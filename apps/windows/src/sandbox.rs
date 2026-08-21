//! G9-A first-playable Sandbox product surface.
//!
//! The CPU owns only interaction commands and deterministic preset staging.
//! The dense Current/Next buffers remain the authoritative world. Interactive
//! edits are coalesced once per redraw and committed by one bounded GPU
//! dispatch before any production tick for that redraw.

use std::{
    collections::{BTreeMap, BTreeSet},
    time::{Duration, Instant},
};

use bytemuck::{Pod, Zeroable};
use powdergame_core::{
    initial_material_ids, is_valid_cell_material_value, registry_lookup, WorldConfig,
    MATERIAL_BOUNDARY_BLOCK, MATERIAL_ICE, MATERIAL_OIL, MATERIAL_SAND, MATERIAL_SMOKE,
    MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD, TEMPERATURE_REFERENCE,
};
use powdergame_gpu::{GpuError, Simulation};

use crate::inspector::{InspectorHudData, ScreenRect};
use crate::renderer::WorldTransform;

pub(crate) const SANDBOX_WORLD_WIDTH: u32 = 256;
pub(crate) const SANDBOX_WORLD_HEIGHT: u32 = 256;
pub(crate) const SANDBOX_CHUNK_SIZE: u32 = 64;
pub(crate) const SANDBOX_TPS: u32 = 60;
pub(crate) const SANDBOX_TITLE: &str = "Powdergame G9-A First Playable Sandbox";
pub(crate) const HEAT_DELTA: f32 = 25.0;
pub(crate) const COOL_DELTA: f32 = -25.0;
pub(crate) const ICE_PLACEMENT_TEMPERATURE: f32 = -10.0;
pub(crate) const STEAM_PLACEMENT_TEMPERATURE: f32 = 120.0;
pub(crate) const THERMAL_APPLICATION_FEEDBACK_HOLD: Duration = Duration::from_millis(180);
pub(crate) const MAX_PENDING_EDIT_CELLS: usize = 32_768;
const EDIT_COMMAND_CAPACITY: usize = MAX_PENDING_EDIT_CELLS;
const EDIT_COMMAND_BYTES: u64 = 16;
const EDIT_PARAMS_BYTES: u64 = 16;
const EDIT_WORKGROUP_SIZE: u32 = 64;

pub(crate) const BRUSH_DIAMETERS: [u32; 4] = [1, 3, 5, 9];

/// Product presets are intentionally separate from the official G8 ScenarioId.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SandboxPreset {
    StarterLab,
    BlankWorld,
}

impl SandboxPreset {
    pub(crate) const fn display_name(self) -> &'static str {
        match self {
            Self::StarterLab => "Starter Lab",
            Self::BlankWorld => "New Blank World",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SandboxTool {
    Draw,
    Erase,
    Heat,
    Cool,
}

impl SandboxTool {
    pub(crate) const fn display_name(self) -> &'static str {
        match self {
            Self::Draw => "Matter Draw",
            Self::Erase => "Erase",
            Self::Heat => "Heat +25",
            Self::Cool => "Cool -25",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SandboxPaletteGroup {
    Core,
    Generated,
    Advanced,
}

impl SandboxPaletteGroup {
    pub(crate) const fn display_name(self) -> &'static str {
        match self {
            Self::Core => "CORE",
            Self::Generated => "GENERATED",
            Self::Advanced => "ADVANCED",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SandboxPaletteEntry {
    pub material_id: u32,
    pub group: SandboxPaletteGroup,
}

/// All M0 Matter remains immediately available. Phase/reaction products and
/// the editable world boundary are visually separated from the core choices.
pub(crate) const SANDBOX_PALETTE: [SandboxPaletteEntry; 9] = [
    SandboxPaletteEntry {
        material_id: MATERIAL_STONE,
        group: SandboxPaletteGroup::Core,
    },
    SandboxPaletteEntry {
        material_id: MATERIAL_SAND,
        group: SandboxPaletteGroup::Core,
    },
    SandboxPaletteEntry {
        material_id: MATERIAL_WATER,
        group: SandboxPaletteGroup::Core,
    },
    SandboxPaletteEntry {
        material_id: MATERIAL_WOOD,
        group: SandboxPaletteGroup::Core,
    },
    SandboxPaletteEntry {
        material_id: MATERIAL_OIL,
        group: SandboxPaletteGroup::Core,
    },
    SandboxPaletteEntry {
        material_id: MATERIAL_ICE,
        group: SandboxPaletteGroup::Generated,
    },
    SandboxPaletteEntry {
        material_id: MATERIAL_STEAM,
        group: SandboxPaletteGroup::Generated,
    },
    SandboxPaletteEntry {
        material_id: MATERIAL_SMOKE,
        group: SandboxPaletteGroup::Generated,
    },
    SandboxPaletteEntry {
        material_id: MATERIAL_BOUNDARY_BLOCK,
        group: SandboxPaletteGroup::Advanced,
    },
];

/// Palette order is a product decision; names always come from the canonical registry.
pub(crate) const SANDBOX_PALETTE_IDS: [u32; 9] = [
    MATERIAL_STONE,
    MATERIAL_SAND,
    MATERIAL_WATER,
    MATERIAL_WOOD,
    MATERIAL_OIL,
    MATERIAL_ICE,
    MATERIAL_STEAM,
    MATERIAL_SMOKE,
    MATERIAL_BOUNDARY_BLOCK,
];

pub(crate) const SANDBOX_PALETTE_GROUP_LABEL_Y: [f32; 3] = [140.0, 324.0, 446.0];
pub(crate) const SANDBOX_PALETTE_ROW_Y: [f32; 9] = [
    162.0, 193.0, 224.0, 255.0, 286.0, 346.0, 377.0, 408.0, 468.0,
];
pub(crate) const SANDBOX_PRESET_TITLE_Y: f32 = 507.0;
pub(crate) const SANDBOX_PRESET_FIRST_ROW_Y: f32 = 534.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct SandboxCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SandboxEditKind {
    Draw(u32),
    Erase,
    Heat(f32),
    Cool(f32),
}

impl SandboxEditKind {
    fn validate(self) -> Result<(), String> {
        match self {
            Self::Draw(material_id) if is_valid_cell_material_value(material_id) => Ok(()),
            Self::Draw(material_id) => Err(format!("invalid Sandbox Material ID {material_id}")),
            Self::Erase => Ok(()),
            Self::Heat(delta) if delta.is_finite() && delta > 0.0 => Ok(()),
            Self::Cool(delta) if delta.is_finite() && delta < 0.0 => Ok(()),
            Self::Heat(delta) | Self::Cool(delta) => {
                Err(format!("invalid Sandbox temperature delta {delta:?}"))
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct GpuEditCommand {
    cell_index: u32,
    operation: u32,
    value_bits: u32,
    placement_temperature_bits: u32,
}

impl GpuEditCommand {
    fn from_edit(cell_index: u32, edit: SandboxEditKind) -> Self {
        let (operation, value_bits, placement_temperature_bits) = match edit {
            SandboxEditKind::Draw(material_id) => {
                (0, material_id, placement_temperature(material_id).to_bits())
            }
            SandboxEditKind::Erase => (1, 0, 0),
            SandboxEditKind::Heat(delta) => (2, delta.to_bits(), 0),
            SandboxEditKind::Cool(delta) => (3, delta.to_bits(), 0),
        };
        Self {
            cell_index,
            operation,
            value_bits,
            placement_temperature_bits,
        }
    }
}

pub(crate) const fn placement_temperature(material_id: u32) -> f32 {
    match material_id {
        MATERIAL_ICE => ICE_PLACEMENT_TEMPERATURE,
        MATERIAL_STEAM => STEAM_PLACEMENT_TEMPERATURE,
        _ => TEMPERATURE_REFERENCE,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SandboxPresetImage {
    pub materials: Vec<u32>,
}

/// Creates the deterministic pristine material image for one product preset.
pub(crate) fn preset_image(
    preset: SandboxPreset,
    config: WorldConfig,
) -> Result<SandboxPresetImage, String> {
    if config.width != SANDBOX_WORLD_WIDTH
        || config.height != SANDBOX_WORLD_HEIGHT
        || config.chunk_size != SANDBOX_CHUNK_SIZE
    {
        return Err(format!(
            "Sandbox presets require {}x{} chunk {}, got {}x{} chunk {}",
            SANDBOX_WORLD_WIDTH,
            SANDBOX_WORLD_HEIGHT,
            SANDBOX_CHUNK_SIZE,
            config.width,
            config.height,
            config.chunk_size
        ));
    }
    let mut materials = initial_material_ids(&config).map_err(|error| error.to_string())?;
    if preset == SandboxPreset::BlankWorld {
        return Ok(SandboxPresetImage { materials });
    }

    let set = |materials: &mut [u32], x: u32, y: u32, id: u32| {
        let index = (y * config.width + x) as usize;
        materials[index] = id;
    };
    let fill = |materials: &mut [u32], x0: u32, x1: u32, y0: u32, y1: u32, id: u32| {
        for y in y0..y1 {
            for x in x0..x1 {
                set(materials, x, y, id);
            }
        }
    };

    // A single open workshop, not a grid of benchmark panels. The large
    // upper half and central aisle stay empty for free construction.
    fill(&mut materials, 12, 244, 232, 244, MATERIAL_STONE); // foundation

    // Left shallow basin: Water and Sand can mix/fall immediately when played.
    fill(&mut materials, 26, 30, 176, 232, MATERIAL_STONE);
    fill(&mut materials, 102, 106, 176, 232, MATERIAL_STONE);
    fill(&mut materials, 26, 106, 226, 232, MATERIAL_STONE);
    fill(&mut materials, 31, 67, 204, 226, MATERIAL_WATER);
    fill(&mut materials, 72, 98, 190, 226, MATERIAL_SAND);

    // Neutral Wood bridge/workpiece over the open center. It has no authored
    // heat, fuel progress, pressure, or flags; Heat can ignite it naturally.
    fill(&mut materials, 108, 174, 198, 204, MATERIAL_WOOD);
    fill(&mut materials, 112, 118, 204, 232, MATERIAL_WOOD);
    fill(&mut materials, 164, 170, 204, 232, MATERIAL_WOOD);

    // Right open cup with Oil and a nearby Ice block for density/thermal work.
    fill(&mut materials, 180, 184, 184, 232, MATERIAL_STONE);
    fill(&mut materials, 230, 234, 184, 232, MATERIAL_STONE);
    fill(&mut materials, 180, 234, 226, 232, MATERIAL_STONE);
    fill(&mut materials, 185, 207, 205, 226, MATERIAL_OIL);
    fill(&mut materials, 208, 229, 198, 226, MATERIAL_ICE);

    Ok(SandboxPresetImage { materials })
}

pub(crate) fn stage_preset(
    simulation: &mut Simulation,
    preset: SandboxPreset,
) -> Result<(), GpuError> {
    simulation.reset()?;
    let image = preset_image(preset, simulation.world.config)
        .map_err(|error| GpuError::Other(format!("Sandbox preset build failed: {error}")))?;
    let mut bytes = Vec::with_capacity(image.materials.len() * 4);
    for material in &image.materials {
        bytes.extend_from_slice(&material.to_ne_bytes());
    }
    simulation
        .context
        .queue
        .write_buffer(&simulation.world.material_current, 0, &bytes);
    simulation
        .context
        .queue
        .write_buffer(&simulation.world.material_next, 0, &bytes);
    simulation
        .world
        .stage_phase_energy_for_materials(&simulation.context.queue, &image.materials)?;
    simulation.world.stage_environment_for_materials(
        &simulation.context.queue,
        &image.materials,
        powdergame_core::EmptyEnvironmentSeed::StandardAtmosphere,
    )?;
    Ok(())
}

/// Returns a deterministic circular brush stamp clipped to world bounds.
pub(crate) fn brush_cells(
    center: SandboxCell,
    diameter: u32,
    width: u32,
    height: u32,
) -> Vec<SandboxCell> {
    if width == 0 || height == 0 || diameter == 0 {
        return Vec::new();
    }
    let radius = (diameter / 2) as i64;
    let radius_sq = radius * radius;
    let mut cells = Vec::new();
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy > radius_sq {
                continue;
            }
            let x = i64::from(center.x) + dx;
            let y = i64::from(center.y) + dy;
            if x >= 0 && y >= 0 && x < i64::from(width) && y < i64::from(height) {
                cells.push(SandboxCell {
                    x: x as u32,
                    y: y as u32,
                });
            }
        }
    }
    cells.sort_unstable();
    cells.dedup();
    cells
}

/// Integer Bresenham centers used to interpolate long pointer drags.
pub(crate) fn interpolated_centers(from: SandboxCell, to: SandboxCell) -> Vec<SandboxCell> {
    let mut x0 = i64::from(from.x);
    let mut y0 = i64::from(from.y);
    let x1 = i64::from(to.x);
    let y1 = i64::from(to.y);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    let mut cells = Vec::new();
    loop {
        cells.push(SandboxCell {
            x: x0 as u32,
            y: y0 as u32,
        });
        if x0 == x1 && y0 == y1 {
            break;
        }
        let doubled = 2 * error;
        if doubled >= dy {
            error += dy;
            x0 += sx;
        }
        if doubled <= dx {
            error += dx;
            y0 += sy;
        }
    }
    cells
}

fn stroke_cells(
    from: SandboxCell,
    to: SandboxCell,
    diameter: u32,
    width: u32,
    height: u32,
) -> Vec<SandboxCell> {
    let mut cells = BTreeSet::new();
    for center in interpolated_centers(from, to) {
        cells.extend(brush_cells(center, diameter, width, height));
    }
    cells.into_iter().collect()
}

pub(crate) struct SandboxEditController {
    field_pipeline: wgpu::ComputePipeline,
    field_bind_group: wgpu::BindGroup,
    flag_pipeline: wgpu::ComputePipeline,
    flag_bind_group: wgpu::BindGroup,
    environment_pipeline: wgpu::ComputePipeline,
    environment_bind_group: wgpu::BindGroup,
    phase_pipeline: wgpu::ComputePipeline,
    phase_bind_group: wgpu::BindGroup,
    params_buffer: wgpu::Buffer,
    command_buffer: wgpu::Buffer,
    pending: BTreeMap<u32, SandboxEditKind>,
    width: u32,
    height: u32,
    chunk_size: u32,
}

impl SandboxEditController {
    pub(crate) fn new(simulation: &Simulation) -> Result<Self, GpuError> {
        let device = &simulation.context.device;
        let field_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("powdergame-g9a-sandbox-edit-field-shader"),
            source: wgpu::ShaderSource::Wgsl(SANDBOX_EDIT_FIELD_SHADER.into()),
        });
        let flag_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("powdergame-g9a-sandbox-edit-flag-shader"),
            source: wgpu::ShaderSource::Wgsl(SANDBOX_EDIT_FLAG_SHADER.into()),
        });
        let environment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("powdergame-te1-sandbox-edit-environment-shader"),
            source: wgpu::ShaderSource::Wgsl(SANDBOX_EDIT_ENVIRONMENT_SHADER.into()),
        });
        let phase_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("powdergame-te3-sandbox-edit-phase-shader"),
            source: wgpu::ShaderSource::Wgsl(SANDBOX_EDIT_PHASE_SHADER.into()),
        });
        let field_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("powdergame-g9a-sandbox-edit-field-bgl"),
            entries: &[
                uniform_entry(0, EDIT_PARAMS_BYTES),
                storage_entry(1, true),
                storage_entry(2, false),
                storage_entry(3, false),
                storage_entry(4, false),
                storage_entry(5, false),
                storage_entry(6, false),
                storage_entry(7, false),
            ],
        });
        let flag_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("powdergame-g9a-sandbox-edit-flag-bgl"),
            entries: &[
                uniform_entry(0, EDIT_PARAMS_BYTES),
                storage_entry(1, true),
                storage_entry(2, false),
                storage_entry(3, false),
                storage_entry(4, true),
                storage_entry(5, true),
            ],
        });
        let environment_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("powdergame-te1-sandbox-edit-environment-bgl"),
                entries: &[
                    uniform_entry(0, EDIT_PARAMS_BYTES),
                    storage_entry(1, true),
                    storage_entry(2, true),
                    storage_entry(3, true),
                    storage_entry(4, false),
                    storage_entry(5, false),
                    storage_entry(6, false),
                    storage_entry(7, false),
                ],
            });
        let phase_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("powdergame-te3-sandbox-edit-phase-bgl"),
            entries: &[
                uniform_entry(0, EDIT_PARAMS_BYTES),
                storage_entry(1, true),
                storage_entry(2, true),
                storage_entry(3, true),
                storage_entry(4, false),
                storage_entry(5, false),
            ],
        });
        let field_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("powdergame-g9a-sandbox-edit-field-pl"),
                bind_group_layouts: &[&field_layout],
                push_constant_ranges: &[],
            });
        let flag_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("powdergame-g9a-sandbox-edit-flag-pl"),
            bind_group_layouts: &[&flag_layout],
            push_constant_ranges: &[],
        });
        let environment_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("powdergame-te1-sandbox-edit-environment-pl"),
                bind_group_layouts: &[&environment_layout],
                push_constant_ranges: &[],
            });
        let phase_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("powdergame-te3-sandbox-edit-phase-pl"),
                bind_group_layouts: &[&phase_layout],
                push_constant_ranges: &[],
            });
        let field_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("powdergame-g9a-sandbox-edit-field-pipeline"),
            layout: Some(&field_pipeline_layout),
            module: &field_shader,
            entry_point: Some("apply_fields"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let flag_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("powdergame-g9a-sandbox-edit-flag-pipeline"),
            layout: Some(&flag_pipeline_layout),
            module: &flag_shader,
            entry_point: Some("apply_flags"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let environment_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("powdergame-te1-sandbox-edit-environment-pipeline"),
                layout: Some(&environment_pipeline_layout),
                module: &environment_shader,
                entry_point: Some("apply_environment"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        let phase_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("powdergame-te3-sandbox-edit-phase-pipeline"),
            layout: Some(&phase_pipeline_layout),
            module: &phase_shader,
            entry_point: Some("apply_phase_energy"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("powdergame-g9a-sandbox-edit-params"),
            size: EDIT_PARAMS_BYTES,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let command_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("powdergame-g9a-sandbox-edit-commands"),
            size: EDIT_COMMAND_BYTES * EDIT_COMMAND_CAPACITY as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let world = &simulation.world;
        let field_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("powdergame-g9a-sandbox-edit-field-bg"),
            layout: &field_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: command_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: world.material_current.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: world.material_next.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: world.temperature_current.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: world.temperature_next.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: world.pressure_current.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: world.pressure_next.as_entire_binding(),
                },
            ],
        });
        let flag_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("powdergame-g9a-sandbox-edit-flag-bg"),
            layout: &flag_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: command_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: world.flags_current.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: world.flags_next.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: world.material_current.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: world.material_next.as_entire_binding(),
                },
            ],
        });
        let environment_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("powdergame-te1-sandbox-edit-environment-bg"),
            layout: &environment_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: command_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: world.material_current.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: world.material_next.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: world.air_mass_current.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: world.air_mass_next.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: world.air_energy_current.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: world.air_energy_next.as_entire_binding(),
                },
            ],
        });
        let phase_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("powdergame-te3-sandbox-edit-phase-bg"),
            layout: &phase_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: command_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: world.material_current.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: world.material_next.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: world.phase_energy_current.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: world.phase_energy_next.as_entire_binding(),
                },
            ],
        });
        Ok(Self {
            field_pipeline,
            field_bind_group,
            flag_pipeline,
            flag_bind_group,
            environment_pipeline,
            environment_bind_group,
            phase_pipeline,
            phase_bind_group,
            params_buffer,
            command_buffer,
            pending: BTreeMap::new(),
            width: world.config.width,
            height: world.config.height,
            chunk_size: world.config.chunk_size,
        })
    }

    pub(crate) fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub(crate) fn clear_pending(&mut self) {
        self.pending.clear();
    }

    pub(crate) fn queue_stroke(
        &mut self,
        from: SandboxCell,
        to: SandboxCell,
        diameter: u32,
        edit: SandboxEditKind,
    ) -> Result<usize, String> {
        edit.validate()?;
        if !BRUSH_DIAMETERS.contains(&diameter) {
            return Err(format!("unsupported Sandbox brush diameter {diameter}"));
        }
        let cells = stroke_cells(from, to, diameter, self.width, self.height);
        let additional = cells
            .iter()
            .filter(|cell| {
                let index = cell.y * self.width + cell.x;
                !self.pending.contains_key(&index)
            })
            .count();
        if self.pending.len() + additional > MAX_PENDING_EDIT_CELLS {
            return Err(format!(
                "Sandbox edit batch would exceed the {MAX_PENDING_EDIT_CELLS}-cell bound"
            ));
        }
        for cell in cells {
            self.pending.insert(cell.y * self.width + cell.x, edit);
        }
        Ok(self.pending.len())
    }

    /// Applies one prevalidated, deduplicated batch with exactly one GPU submit.
    pub(crate) fn apply_pending(&mut self, simulation: &Simulation) -> Result<usize, GpuError> {
        if self.pending.is_empty() {
            return Ok(0);
        }
        let commands = self
            .pending
            .iter()
            .map(|(&index, &edit)| GpuEditCommand::from_edit(index, edit))
            .collect::<Vec<_>>();
        let count = commands.len();
        debug_assert!(count <= EDIT_COMMAND_CAPACITY);

        let params = [count as u32, self.width, self.height, self.chunk_size];
        let queue = &simulation.context.queue;
        queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&params));
        queue.write_buffer(&self.command_buffer, 0, bytemuck::cast_slice(&commands));

        // Wake each touched chunk plus its 8-neighbor safety halo. Writes are
        // deduplicated and precede the edit dispatch in queue order.
        let chunks_x = self.width.div_ceil(self.chunk_size);
        let chunks_y = self.height.div_ceil(self.chunk_size);
        let mut wake_chunks = BTreeSet::new();
        for &index in self.pending.keys() {
            let x = (index % self.width) / self.chunk_size;
            let y = (index / self.width) / self.chunk_size;
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    let nx = i64::from(x) + dx;
                    let ny = i64::from(y) + dy;
                    if nx >= 0 && ny >= 0 && nx < i64::from(chunks_x) && ny < i64::from(chunks_y) {
                        wake_chunks.insert(ny as u32 * chunks_x + nx as u32);
                    }
                }
            }
        }
        let one = 1u32.to_ne_bytes();
        let zero = 0u32.to_ne_bytes();
        for chunk in wake_chunks {
            let offset = u64::from(chunk) * 4;
            queue.write_buffer(&simulation.world.chunk_edit_wake, offset, &one);
            queue.write_buffer(&simulation.world.chunk_stable_ticks, offset, &zero);
            queue.write_buffer(&simulation.world.chunk_state, offset, &zero);
        }

        let mut encoder =
            simulation
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("powdergame-g9a-sandbox-edit-encoder"),
                });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-sandbox-edit-environment-pass"),
                timestamp_writes: None,
            });
            // Environment observes pre-edit occupancy so a rejected EMPTY-only
            // Draw cannot clear Air under an existing Matter cell.
            pass.set_pipeline(&self.environment_pipeline);
            pass.set_bind_group(0, &self.environment_bind_group, &[]);
            pass.dispatch_workgroups((count as u32).div_ceil(EDIT_WORKGROUP_SIZE), 1, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te3-sandbox-edit-phase-pass"),
                timestamp_writes: None,
            });
            // Phase energy observes the same pre-edit occupancy as Environment
            // and flags, so a rejected EMPTY-only Draw cannot alter owner state.
            pass.set_pipeline(&self.phase_pipeline);
            pass.set_bind_group(0, &self.phase_bind_group, &[]);
            pass.dispatch_workgroups((count as u32).div_ceil(EDIT_WORKGROUP_SIZE), 1, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g9a-sandbox-edit-flags-pass"),
                timestamp_writes: None,
            });
            // Flags inspect the pre-edit Material identity so a rejected
            // EMPTY-only Draw cannot erase state owned by an occupied cell.
            pass.set_pipeline(&self.flag_pipeline);
            pass.set_bind_group(0, &self.flag_bind_group, &[]);
            pass.dispatch_workgroups((count as u32).div_ceil(EDIT_WORKGROUP_SIZE), 1, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g9a-sandbox-edit-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.field_pipeline);
            pass.set_bind_group(0, &self.field_bind_group, &[]);
            pass.dispatch_workgroups((count as u32).div_ceil(EDIT_WORKGROUP_SIZE), 1, 1);
        }
        queue.submit([encoder.finish()]);
        self.pending.clear();
        Ok(count)
    }
}

fn uniform_entry(binding: u32, min_size: u64) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(min_size),
        },
        count: None,
    }
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SandboxHudData {
    pub preset: SandboxPreset,
    pub tool: SandboxTool,
    pub selected_material_id: u32,
    pub brush_diameter: u32,
    pub playing: bool,
    pub speed: u32,
    pub simulation_tick: u64,
    pub pending_edits: usize,
    pub thermal_feedback: Option<SandboxThermalFeedback>,
    pub inspector: Option<InspectorHudData>,
    pub inspector_cursor: Option<[f32; 2]>,
    pub world_viewport: Option<ScreenRect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SandboxThermalFeedback {
    pub tool: SandboxTool,
    pub rect: ScreenRect,
    pub state: SandboxThermalFeedbackState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SandboxThermalFeedbackState {
    Preview,
    Applying,
    CommittedPulse,
}

pub(crate) fn thermal_brush_feedback(
    transform: WorldTransform,
    center: SandboxCell,
    diameter: u32,
    tool: SandboxTool,
    state: SandboxThermalFeedbackState,
) -> Option<SandboxThermalFeedback> {
    if !matches!(tool, SandboxTool::Heat | SandboxTool::Cool)
        || !BRUSH_DIAMETERS.contains(&diameter)
    {
        return None;
    }
    let cells = brush_cells(center, diameter, SANDBOX_WORLD_WIDTH, SANDBOX_WORLD_HEIGHT);
    let min_x = cells.iter().map(|cell| cell.x).min()? as f32;
    let min_y = cells.iter().map(|cell| cell.y).min()? as f32;
    let max_x = (cells.iter().map(|cell| cell.x).max()? + 1) as f32;
    let max_y = (cells.iter().map(|cell| cell.y).max()? + 1) as f32;
    let viewport = transform.viewport;
    let x0 = (viewport.x + (min_x - transform.origin_x) * transform.scale).max(viewport.x);
    let y0 = (viewport.y + (min_y - transform.origin_y) * transform.scale).max(viewport.y);
    let x1 = (viewport.x + (max_x - transform.origin_x) * transform.scale).min(viewport.right());
    let y1 = (viewport.y + (max_y - transform.origin_y) * transform.scale).min(viewport.bottom());
    (x1 > x0 && y1 > y0).then_some(SandboxThermalFeedback {
        tool,
        rect: ScreenRect {
            x: x0,
            y: y0,
            width: x1 - x0,
            height: y1 - y0,
        },
        state,
    })
}

#[derive(Clone, Copy, Debug)]
struct SandboxThermalApplication {
    cell: SandboxCell,
    diameter: u32,
    tool: SandboxTool,
    applied_at: Instant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SandboxHudAction {
    SelectMaterial(u32),
    LoadPreset(SandboxPreset),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SandboxKeyAction {
    SelectMaterial(u32),
    SelectTool(SandboxTool),
    LoadPreset(SandboxPreset),
}

pub(crate) fn sandbox_key_action(character: &str) -> Option<SandboxKeyAction> {
    if character.len() == 1
        && matches!(
            character,
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
        )
    {
        return Some(SandboxKeyAction::SelectMaterial(
            SANDBOX_PALETTE_IDS[(character.as_bytes()[0] - b'1') as usize],
        ));
    }
    if character.eq_ignore_ascii_case("d") {
        return Some(SandboxKeyAction::SelectTool(SandboxTool::Draw));
    }
    if character.eq_ignore_ascii_case("e") {
        return Some(SandboxKeyAction::SelectTool(SandboxTool::Erase));
    }
    if character.eq_ignore_ascii_case("h") {
        return Some(SandboxKeyAction::SelectTool(SandboxTool::Heat));
    }
    if character.eq_ignore_ascii_case("c") {
        return Some(SandboxKeyAction::SelectTool(SandboxTool::Cool));
    }
    if character.eq_ignore_ascii_case("l") {
        return Some(SandboxKeyAction::LoadPreset(SandboxPreset::StarterLab));
    }
    if character.eq_ignore_ascii_case("b") {
        return Some(SandboxKeyAction::LoadPreset(SandboxPreset::BlankWorld));
    }
    None
}

/// Pointer hit testing for the two deliberately clickable HUD regions. The
/// remaining essential controls are discoverable through the persistent key
/// and pointer hints and do not overlap the world viewport.
pub(crate) fn sandbox_hud_action_at(
    surface_width: u32,
    surface_height: u32,
    cursor: [f32; 2],
) -> Option<SandboxHudAction> {
    if surface_width < 700
        || surface_height < 420
        || !cursor[0].is_finite()
        || !cursor[1].is_finite()
    {
        return None;
    }
    let card_width = 292.0;
    let right = surface_width as f32 - card_width - 14.0;
    for (index, material_id) in SANDBOX_PALETTE_IDS.iter().copied().enumerate() {
        let y = SANDBOX_PALETTE_ROW_Y[index] - 4.0;
        if cursor[0] >= right + 9.0
            && cursor[0] < right + card_width - 9.0
            && cursor[1] >= y
            && cursor[1] < y + 27.0
        {
            return Some(SandboxHudAction::SelectMaterial(material_id));
        }
    }
    let preset_y = SANDBOX_PRESET_FIRST_ROW_Y - 4.0;
    if cursor[0] >= right + 9.0 && cursor[0] < right + card_width - 9.0 {
        if cursor[1] >= preset_y && cursor[1] < preset_y + 26.0 {
            return Some(SandboxHudAction::LoadPreset(SandboxPreset::StarterLab));
        }
        if cursor[1] >= preset_y + 28.0 && cursor[1] < preset_y + 54.0 {
            return Some(SandboxHudAction::LoadPreset(SandboxPreset::BlankWorld));
        }
    }
    None
}

pub(crate) struct SandboxRuntime {
    pub preset: SandboxPreset,
    pub tool: SandboxTool,
    pub selected_material_id: u32,
    pub brush_index: usize,
    pub primary_down: bool,
    pub erase_down: bool,
    pub pan_down: bool,
    pub shift_down: bool,
    pub last_edit_cell: Option<SandboxCell>,
    pub last_cursor: Option<[f64; 2]>,
    pub pending_preset: Option<SandboxPreset>,
    pub edits: SandboxEditController,
    pending_thermal_application: Option<(SandboxCell, u32, SandboxTool)>,
    last_thermal_application: Option<SandboxThermalApplication>,
}

impl SandboxRuntime {
    pub(crate) fn new(simulation: &Simulation) -> Result<Self, GpuError> {
        Ok(Self {
            preset: SandboxPreset::StarterLab,
            tool: SandboxTool::Draw,
            selected_material_id: MATERIAL_SAND,
            brush_index: 1,
            primary_down: false,
            erase_down: false,
            pan_down: false,
            shift_down: false,
            last_edit_cell: None,
            last_cursor: None,
            pending_preset: None,
            edits: SandboxEditController::new(simulation)?,
            pending_thermal_application: None,
            last_thermal_application: None,
        })
    }

    pub(crate) fn brush_diameter(&self) -> u32 {
        BRUSH_DIAMETERS[self.brush_index]
    }

    pub(crate) fn cycle_brush(&mut self, direction: i32) {
        let len = BRUSH_DIAMETERS.len() as i32;
        self.brush_index = (self.brush_index as i32 + direction).rem_euclid(len) as usize;
    }

    pub(crate) fn selected_edit(&self, force_erase: bool) -> SandboxEditKind {
        if force_erase {
            return SandboxEditKind::Erase;
        }
        match self.tool {
            SandboxTool::Draw => SandboxEditKind::Draw(self.selected_material_id),
            SandboxTool::Erase => SandboxEditKind::Erase,
            SandboxTool::Heat => SandboxEditKind::Heat(HEAT_DELTA),
            SandboxTool::Cool => SandboxEditKind::Cool(COOL_DELTA),
        }
    }

    pub(crate) fn cancel_pointer_gestures(&mut self) {
        self.primary_down = false;
        self.erase_down = false;
        self.pan_down = false;
        self.last_edit_cell = None;
        self.last_cursor = None;
    }

    pub(crate) fn note_queued_thermal_application(
        &mut self,
        cell: SandboxCell,
        diameter: u32,
        edit: SandboxEditKind,
    ) {
        let tool = match edit {
            SandboxEditKind::Heat(_) => SandboxTool::Heat,
            SandboxEditKind::Cool(_) => SandboxTool::Cool,
            SandboxEditKind::Draw(_) | SandboxEditKind::Erase => return,
        };
        self.pending_thermal_application = Some((cell, diameter, tool));
    }

    pub(crate) fn commit_thermal_application(&mut self, now: Instant) {
        self.last_thermal_application =
            self.pending_thermal_application
                .take()
                .map(|(cell, diameter, tool)| SandboxThermalApplication {
                    cell,
                    diameter,
                    tool,
                    applied_at: now,
                });
    }

    pub(crate) fn clear_thermal_feedback(&mut self) {
        self.pending_thermal_application = None;
        self.last_thermal_application = None;
    }

    pub(crate) fn recent_thermal_application(
        &self,
        now: Instant,
    ) -> Option<(SandboxCell, u32, SandboxTool)> {
        self.last_thermal_application
            .filter(|application| {
                now.saturating_duration_since(application.applied_at)
                    < THERMAL_APPLICATION_FEEDBACK_HOLD
            })
            .map(|application| (application.cell, application.diameter, application.tool))
    }

    pub(crate) fn request_preset(&mut self, preset: SandboxPreset) {
        self.pending_preset = Some(preset);
        self.edits.clear_pending();
        self.clear_thermal_feedback();
        self.cancel_pointer_gestures();
    }
}

impl SandboxHudData {
    pub(crate) fn material_name(&self) -> &'static str {
        registry_lookup(self.selected_material_id)
            .map(|descriptor| descriptor.name)
            .unwrap_or("Invalid Material")
    }
}

const SANDBOX_EDIT_FIELD_SHADER: &str = r#"
struct Params {
    count: u32,
    width: u32,
    height: u32,
    chunk_size: u32,
};

struct EditCommand {
    cell_index: u32,
    operation: u32,
    value_bits: u32,
    placement_temperature_bits: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> commands: array<EditCommand>;
@group(0) @binding(2) var<storage, read_write> material_current: array<u32>;
@group(0) @binding(3) var<storage, read_write> material_next: array<u32>;
@group(0) @binding(4) var<storage, read_write> temperature_current: array<f32>;
@group(0) @binding(5) var<storage, read_write> temperature_next: array<f32>;
@group(0) @binding(6) var<storage, read_write> pressure_current: array<f32>;
@group(0) @binding(7) var<storage, read_write> pressure_next: array<f32>;

const EMPTY: u32 = 0u;
const TEMPERATURE_REFERENCE: f32 = 20.0;
const PRESSURE_REFERENCE: f32 = 0.0;
const TEMPERATURE_MIN: f32 = -250.0;
const TEMPERATURE_MAX: f32 = 2000.0;

@compute @workgroup_size(64)
fn apply_fields(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= params.count) { return; }
    let command = commands[gid.x];
    let cell_count = params.width * params.height;
    if (command.cell_index >= cell_count) { return; }
    let index = command.cell_index;

    if (command.operation == 0u) {
        if (material_current[index] != EMPTY || material_next[index] != EMPTY) { return; }
        let placement_temperature = bitcast<f32>(command.placement_temperature_bits);
        material_current[index] = command.value_bits;
        material_next[index] = command.value_bits;
        temperature_current[index] = placement_temperature;
        temperature_next[index] = placement_temperature;
        pressure_current[index] = PRESSURE_REFERENCE;
        pressure_next[index] = PRESSURE_REFERENCE;
        return;
    }
    if (command.operation == 1u) {
        material_current[index] = EMPTY;
        material_next[index] = EMPTY;
        temperature_current[index] = TEMPERATURE_REFERENCE;
        temperature_next[index] = TEMPERATURE_REFERENCE;
        pressure_current[index] = PRESSURE_REFERENCE;
        pressure_next[index] = PRESSURE_REFERENCE;
        return;
    }
    if (material_current[index] == EMPTY) { return; }
    let delta = bitcast<f32>(command.value_bits);
    let current = temperature_current[index];
    let current_is_finite = current == current && abs(current) <= 3.402823e38;
    let finite_current = select(TEMPERATURE_REFERENCE, current, current_is_finite);
    let updated = clamp(finite_current + delta, TEMPERATURE_MIN, TEMPERATURE_MAX);
    temperature_current[index] = updated;
    temperature_next[index] = updated;
}
"#;

const SANDBOX_EDIT_PHASE_SHADER: &str = r#"
struct Params {
    count: u32,
    width: u32,
    height: u32,
    chunk_size: u32,
};

struct EditCommand {
    cell_index: u32,
    operation: u32,
    value_bits: u32,
    placement_temperature_bits: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> commands: array<EditCommand>;
@group(0) @binding(2) var<storage, read> material_current: array<u32>;
@group(0) @binding(3) var<storage, read> material_next: array<u32>;
@group(0) @binding(4) var<storage, read_write> phase_energy_current: array<f32>;
@group(0) @binding(5) var<storage, read_write> phase_energy_next: array<f32>;

const EMPTY: u32 = 0u;
const WATER: u32 = 4u;
const STEAM: u32 = 6u;
const ICE: u32 = 8u;

fn canonical_phase_energy(material: u32) -> f32 {
    if (material == ICE) { return -80.0; }
    if (material == STEAM) { return 480.0; }
    return 0.0;
}

@compute @workgroup_size(64)
fn apply_phase_energy(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= params.count) { return; }
    let command = commands[gid.x];
    let cell_count = params.width * params.height;
    if (command.cell_index >= cell_count) { return; }
    let index = command.cell_index;

    if (command.operation == 0u) {
        if (material_current[index] != EMPTY || material_next[index] != EMPTY) { return; }
        let energy = canonical_phase_energy(command.value_bits);
        phase_energy_current[index] = energy;
        phase_energy_next[index] = energy;
    } else if (command.operation == 1u) {
        phase_energy_current[index] = 0.0;
        phase_energy_next[index] = 0.0;
    }
}
"#;

const SANDBOX_EDIT_FLAG_SHADER: &str = r#"
struct Params {
    count: u32,
    width: u32,
    height: u32,
    chunk_size: u32,
};

struct EditCommand {
    cell_index: u32,
    operation: u32,
    value_bits: u32,
    placement_temperature_bits: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> commands: array<EditCommand>;
@group(0) @binding(2) var<storage, read_write> flags_current: array<u32>;
@group(0) @binding(3) var<storage, read_write> flags_next: array<u32>;
@group(0) @binding(4) var<storage, read> material_current: array<u32>;
@group(0) @binding(5) var<storage, read> material_next: array<u32>;

const EMPTY: u32 = 0u;

@compute @workgroup_size(64)
fn apply_flags(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= params.count) { return; }
    let command = commands[gid.x];
    let cell_count = params.width * params.height;
    if (command.cell_index >= cell_count) { return; }
    if (command.operation == 0u
        && material_current[command.cell_index] == EMPTY
        && material_next[command.cell_index] == EMPTY) {
        flags_current[command.cell_index] = 0u;
        flags_next[command.cell_index] = 0u;
    } else if (command.operation == 1u) {
        flags_current[command.cell_index] = 0u;
        flags_next[command.cell_index] = 0u;
    }
}
"#;

const SANDBOX_EDIT_ENVIRONMENT_SHADER: &str = r#"
struct Params {
    count: u32,
    width: u32,
    height: u32,
    chunk_size: u32,
};

struct EditCommand {
    cell_index: u32,
    operation: u32,
    value_bits: u32,
    placement_temperature_bits: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> commands: array<EditCommand>;
@group(0) @binding(2) var<storage, read> material_current: array<u32>;
@group(0) @binding(3) var<storage, read> material_next: array<u32>;
@group(0) @binding(4) var<storage, read_write> air_mass_current: array<f32>;
@group(0) @binding(5) var<storage, read_write> air_mass_next: array<f32>;
@group(0) @binding(6) var<storage, read_write> air_energy_current: array<f32>;
@group(0) @binding(7) var<storage, read_write> air_energy_next: array<f32>;

const EMPTY: u32 = 0u;
const STANDARD_AIR_MASS: f32 = 1.0;
const STANDARD_AIR_ENERGY: f32 = 293.15;

@compute @workgroup_size(64)
fn apply_environment(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= params.count) { return; }
    let command = commands[gid.x];
    let cell_count = params.width * params.height;
    if (command.cell_index >= cell_count) { return; }
    let index = command.cell_index;

    if (command.operation == 0u) {
        if (material_current[index] != EMPTY || material_next[index] != EMPTY) { return; }
        air_mass_current[index] = 0.0;
        air_mass_next[index] = 0.0;
        air_energy_current[index] = 0.0;
        air_energy_next[index] = 0.0;
        return;
    }
    if (command.operation == 1u) {
        air_mass_current[index] = STANDARD_AIR_MASS;
        air_mass_next[index] = STANDARD_AIR_MASS;
        air_energy_current[index] = STANDARD_AIR_ENERGY;
        air_energy_next[index] = STANDARD_AIR_ENERGY;
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{PresentationPalette, WorldCamera, WorldViewport};
    use powdergame_core::{
        registry_contains, MATERIAL_EMPTY, MATERIAL_REGISTRY, PRESSURE_MAX, TEMPERATURE_MAX_C,
        TEMPERATURE_MIN_C, TEMPERATURE_REFERENCE,
    };

    fn config() -> WorldConfig {
        WorldConfig::new(
            SANDBOX_WORLD_WIDTH,
            SANDBOX_WORLD_HEIGHT,
            SANDBOX_CHUNK_SIZE,
        )
        .unwrap()
    }

    #[test]
    fn product_presets_are_deterministic_valid_and_not_scenario_ids() {
        let starter_a = preset_image(SandboxPreset::StarterLab, config()).unwrap();
        let starter_b = preset_image(SandboxPreset::StarterLab, config()).unwrap();
        let blank = preset_image(SandboxPreset::BlankWorld, config()).unwrap();
        assert_eq!(starter_a, starter_b);
        assert_ne!(starter_a, blank);
        assert!(starter_a
            .materials
            .iter()
            .all(|&id| id == 0 || registry_contains(id)));
        assert!(blank
            .materials
            .iter()
            .all(|&id| id == 0 || registry_contains(id)));
        assert_eq!(
            starter_a.materials.len(),
            (SANDBOX_WORLD_WIDTH * SANDBOX_WORLD_HEIGHT) as usize
        );
        assert_eq!(blank.materials.len(), starter_a.materials.len());
        assert_eq!(SandboxPreset::StarterLab.display_name(), "Starter Lab");
        assert_eq!(SandboxPreset::BlankWorld.display_name(), "New Blank World");
    }

    #[test]
    fn blank_world_is_only_boundary_and_empty() {
        let image = preset_image(SandboxPreset::BlankWorld, config()).unwrap();
        for y in 0..SANDBOX_WORLD_HEIGHT {
            for x in 0..SANDBOX_WORLD_WIDTH {
                let id = image.materials[(y * SANDBOX_WORLD_WIDTH + x) as usize];
                let edge = x == 0
                    || y == 0
                    || x + 1 == SANDBOX_WORLD_WIDTH
                    || y + 1 == SANDBOX_WORLD_HEIGHT;
                assert_eq!(
                    id,
                    if edge {
                        MATERIAL_BOUNDARY_BLOCK
                    } else {
                        MATERIAL_EMPTY
                    }
                );
            }
        }
    }

    #[test]
    fn starter_lab_exact_geometry_is_open_and_pristine() {
        let image = preset_image(SandboxPreset::StarterLab, config()).unwrap();
        let at = |x, y| image.materials[(y * SANDBOX_WORLD_WIDTH + x) as usize];
        assert_eq!(at(20, 235), MATERIAL_STONE);
        assert_eq!(at(40, 210), MATERIAL_WATER);
        assert_eq!(at(80, 200), MATERIAL_SAND);
        assert_eq!(at(130, 200), MATERIAL_WOOD);
        assert_eq!(at(190, 214), MATERIAL_OIL);
        assert_eq!(at(215, 214), MATERIAL_ICE);
        assert_eq!(at(128, 96), MATERIAL_EMPTY);
        let empty = image
            .materials
            .iter()
            .filter(|&&id| id == MATERIAL_EMPTY)
            .count();
        assert!(
            empty > image.materials.len() * 3 / 4,
            "world remains construction-first"
        );
    }

    #[test]
    fn palette_is_complete_valid_unique_and_uses_registry_names() {
        let unique = SANDBOX_PALETTE_IDS.into_iter().collect::<BTreeSet<_>>();
        assert_eq!(unique.len(), MATERIAL_REGISTRY.len());
        assert_eq!(
            SANDBOX_PALETTE.map(|entry| entry.material_id),
            SANDBOX_PALETTE_IDS
        );
        assert_eq!(
            SANDBOX_PALETTE.map(|entry| entry.group),
            [
                SandboxPaletteGroup::Core,
                SandboxPaletteGroup::Core,
                SandboxPaletteGroup::Core,
                SandboxPaletteGroup::Core,
                SandboxPaletteGroup::Core,
                SandboxPaletteGroup::Generated,
                SandboxPaletteGroup::Generated,
                SandboxPaletteGroup::Generated,
                SandboxPaletteGroup::Advanced,
            ]
        );
        for id in SANDBOX_PALETTE_IDS {
            assert!(registry_lookup(id).is_some());
        }
    }

    #[test]
    fn palette_and_preset_pointer_regions_are_stable() {
        assert_eq!(
            sandbox_hud_action_at(1600, 900, [1305.0, 168.0]),
            Some(SandboxHudAction::SelectMaterial(MATERIAL_STONE))
        );
        assert_eq!(
            sandbox_hud_action_at(1600, 900, [1305.0, 474.0]),
            Some(SandboxHudAction::SelectMaterial(MATERIAL_BOUNDARY_BLOCK))
        );
        assert_eq!(
            sandbox_hud_action_at(1600, 900, [1305.0, 540.0]),
            Some(SandboxHudAction::LoadPreset(SandboxPreset::StarterLab))
        );
        assert_eq!(sandbox_hud_action_at(1600, 900, [800.0, 450.0]), None);
    }

    #[test]
    fn sandbox_key_bindings_are_conflict_free_and_exact() {
        for (key, action) in [
            ("1", SandboxKeyAction::SelectMaterial(MATERIAL_STONE)),
            (
                "9",
                SandboxKeyAction::SelectMaterial(MATERIAL_BOUNDARY_BLOCK),
            ),
            ("d", SandboxKeyAction::SelectTool(SandboxTool::Draw)),
            ("E", SandboxKeyAction::SelectTool(SandboxTool::Erase)),
            ("h", SandboxKeyAction::SelectTool(SandboxTool::Heat)),
            ("C", SandboxKeyAction::SelectTool(SandboxTool::Cool)),
            ("l", SandboxKeyAction::LoadPreset(SandboxPreset::StarterLab)),
            ("B", SandboxKeyAction::LoadPreset(SandboxPreset::BlankWorld)),
        ] {
            assert_eq!(sandbox_key_action(key), Some(action));
        }
        for reserved in ["i", "n", "f", "r", " ", "s"] {
            assert_eq!(sandbox_key_action(reserved), None, "reserved={reserved}");
        }
    }

    #[test]
    fn brush_geometry_is_deterministic_clipped_and_duplicate_free() {
        assert_eq!(
            brush_cells(SandboxCell { x: 8, y: 8 }, 1, 16, 16),
            vec![SandboxCell { x: 8, y: 8 }]
        );
        let corner = brush_cells(SandboxCell { x: 0, y: 0 }, 9, 16, 16);
        assert!(corner.iter().all(|cell| cell.x < 16 && cell.y < 16));
        assert_eq!(
            corner.iter().copied().collect::<BTreeSet<_>>().len(),
            corner.len()
        );
        assert_eq!(corner, brush_cells(SandboxCell { x: 0, y: 0 }, 9, 16, 16));
    }

    #[test]
    fn thermal_feedback_uses_the_same_camera_transform_and_never_changes_draw_tools() {
        let viewport = WorldViewport::calculate(
            1600,
            900,
            SANDBOX_WORLD_WIDTH,
            SANDBOX_WORLD_HEIGHT,
            PresentationPalette::Sandbox,
        )
        .unwrap();
        let transform = WorldTransform::calculate(
            viewport,
            WorldCamera::fitted(SANDBOX_WORLD_WIDTH, SANDBOX_WORLD_HEIGHT),
        );
        let heat = thermal_brush_feedback(
            transform,
            SandboxCell { x: 128, y: 128 },
            5,
            SandboxTool::Heat,
            SandboxThermalFeedbackState::Applying,
        )
        .unwrap();
        assert_eq!(heat.tool, SandboxTool::Heat);
        assert_eq!(heat.state, SandboxThermalFeedbackState::Applying);
        assert!((heat.rect.width - transform.scale * 5.0).abs() < 0.001);
        assert!((heat.rect.height - transform.scale * 5.0).abs() < 0.001);
        assert!(heat.rect.x >= viewport.x && heat.rect.right() <= viewport.right());
        assert!(heat.rect.y >= viewport.y && heat.rect.bottom() <= viewport.bottom());

        let clipped = thermal_brush_feedback(
            transform,
            SandboxCell { x: 0, y: 0 },
            9,
            SandboxTool::Cool,
            SandboxThermalFeedbackState::Preview,
        )
        .unwrap();
        assert_eq!(clipped.rect.x, viewport.x);
        assert_eq!(clipped.rect.y, viewport.y);
        assert_eq!(clipped.state, SandboxThermalFeedbackState::Preview);
        assert!(thermal_brush_feedback(
            transform,
            SandboxCell { x: 1, y: 1 },
            3,
            SandboxTool::Draw,
            SandboxThermalFeedbackState::Applying,
        )
        .is_none());
    }

    #[test]
    fn long_drag_interpolation_has_no_center_gaps() {
        let centers =
            interpolated_centers(SandboxCell { x: 2, y: 3 }, SandboxCell { x: 30, y: 17 });
        assert_eq!(centers.first(), Some(&SandboxCell { x: 2, y: 3 }));
        assert_eq!(centers.last(), Some(&SandboxCell { x: 30, y: 17 }));
        for pair in centers.windows(2) {
            assert!(pair[0].x.abs_diff(pair[1].x) <= 1);
            assert!(pair[0].y.abs_diff(pair[1].y) <= 1);
        }
    }

    #[test]
    fn external_and_internal_temperature_contracts_match_shader_constants() {
        assert_eq!(TEMPERATURE_REFERENCE, 20.0);
        assert_eq!(TEMPERATURE_MIN_C, -250.0);
        assert_eq!(TEMPERATURE_MAX_C, 2000.0);
        assert_eq!(HEAT_DELTA, 25.0);
        assert_eq!(COOL_DELTA, -25.0);
        assert_eq!(ICE_PLACEMENT_TEMPERATURE, -10.0);
        assert_eq!(STEAM_PLACEMENT_TEMPERATURE, 120.0);
        assert_eq!(
            THERMAL_APPLICATION_FEEDBACK_HOLD,
            Duration::from_millis(180)
        );
        assert!((-250.0f32).is_finite() && 1_000.0f32.is_finite());
        assert!(PRESSURE_MAX.is_finite());
    }

    fn read_word(simulation: &Simulation, buffer: &wgpu::Buffer, cell_index: u64) -> [u8; 4] {
        let device = &simulation.context.device;
        let queue = &simulation.context.queue;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sandbox-test-readback"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("sandbox-test-readback-encoder"),
        });
        encoder.copy_buffer_to_buffer(buffer, cell_index * 4, &staging, 0, 4);
        queue.submit([encoder.finish()]);
        let slice = staging.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        device.poll(wgpu::PollType::Wait).unwrap();
        receiver.recv().unwrap().unwrap();
        let mapped = slice.get_mapped_range();
        let word = mapped[..4].try_into().unwrap();
        drop(mapped);
        staging.unmap();
        word
    }

    #[test]
    fn gpu_edit_batch_preserves_current_next_hygiene_identity_and_wake_halo() {
        let config = WorldConfig::new(16, 16, 8).unwrap();
        let mut simulation = pollster::block_on(Simulation::new(config)).unwrap();
        let mut runtime = SandboxRuntime::new(&simulation).unwrap();
        let cell = SandboxCell { x: 5, y: 5 };
        let index = u64::from(cell.y * config.width + cell.x);
        let pulse_at = Instant::now();
        runtime.note_queued_thermal_application(cell, 3, SandboxEditKind::Heat(HEAT_DELTA));
        runtime.commit_thermal_application(pulse_at);
        assert_eq!(
            runtime.recent_thermal_application(pulse_at + Duration::from_millis(179)),
            Some((cell, 3, SandboxTool::Heat))
        );
        assert_eq!(
            runtime.recent_thermal_application(pulse_at + Duration::from_millis(180)),
            None
        );
        runtime.clear_thermal_feedback();

        simulation
            .world
            .write_material(&simulation.context.queue, 5, 5, MATERIAL_WOOD)
            .unwrap();
        simulation
            .world
            .write_temperature(&simulation.context.queue, 5, 5, 400.0)
            .unwrap();
        simulation
            .world
            .write_pressure(&simulation.context.queue, 5, 5, 77.0)
            .unwrap();
        simulation
            .world
            .write_flags(&simulation.context.queue, 5, 5, u32::MAX)
            .unwrap();
        runtime
            .edits
            .queue_stroke(cell, cell, 1, SandboxEditKind::Draw(MATERIAL_STONE))
            .unwrap();
        assert_eq!(runtime.edits.apply_pending(&simulation).unwrap(), 1);

        for buffer in [
            &simulation.world.material_current,
            &simulation.world.material_next,
        ] {
            assert_eq!(
                u32::from_ne_bytes(read_word(&simulation, buffer, index)),
                MATERIAL_WOOD,
                "Draw is EMPTY-only and cannot overwrite Matter"
            );
        }
        for buffer in [
            &simulation.world.temperature_current,
            &simulation.world.temperature_next,
        ] {
            assert_eq!(
                f32::from_ne_bytes(read_word(&simulation, buffer, index)),
                400.0
            );
        }
        for buffer in [
            &simulation.world.pressure_current,
            &simulation.world.pressure_next,
        ] {
            assert_eq!(
                f32::from_ne_bytes(read_word(&simulation, buffer, index)),
                77.0
            );
        }
        for buffer in [
            &simulation.world.flags_current,
            &simulation.world.flags_next,
        ] {
            assert_eq!(
                u32::from_ne_bytes(read_word(&simulation, buffer, index)),
                u32::MAX,
                "a rejected Draw cannot erase occupied-cell flags"
            );
        }
        let rejected_air = simulation
            .world
            .read_environment_cells(
                &simulation.context.device,
                &simulation.context.queue,
                &[(5, 5)],
            )
            .unwrap()[0];
        assert_eq!(rejected_air.current.mass, 0.0);
        assert_eq!(rejected_air.current, rejected_air.next);

        runtime
            .edits
            .queue_stroke(cell, cell, 1, SandboxEditKind::Erase)
            .unwrap();
        runtime.edits.apply_pending(&simulation).unwrap();
        runtime
            .edits
            .queue_stroke(cell, cell, 1, SandboxEditKind::Draw(MATERIAL_STONE))
            .unwrap();
        runtime.edits.apply_pending(&simulation).unwrap();

        for buffer in [
            &simulation.world.material_current,
            &simulation.world.material_next,
        ] {
            assert_eq!(
                u32::from_ne_bytes(read_word(&simulation, buffer, index)),
                MATERIAL_STONE
            );
        }
        for buffer in [
            &simulation.world.temperature_current,
            &simulation.world.temperature_next,
        ] {
            assert_eq!(
                f32::from_ne_bytes(read_word(&simulation, buffer, index)),
                TEMPERATURE_REFERENCE
            );
        }
        for buffer in [
            &simulation.world.pressure_current,
            &simulation.world.pressure_next,
        ] {
            assert_eq!(
                f32::from_ne_bytes(read_word(&simulation, buffer, index)),
                0.0
            );
        }
        for buffer in [
            &simulation.world.flags_current,
            &simulation.world.flags_next,
        ] {
            assert_eq!(u32::from_ne_bytes(read_word(&simulation, buffer, index)), 0);
        }
        let drawn_air = simulation
            .world
            .read_environment_cells(
                &simulation.context.device,
                &simulation.context.queue,
                &[(5, 5)],
            )
            .unwrap()[0];
        assert_eq!(drawn_air.current.mass, 0.0);
        assert_eq!(drawn_air.current.energy, 0.0);
        assert_eq!(drawn_air.current, drawn_air.next);
        assert_eq!(
            simulation
                .world
                .read_chunk_edit_wake_all(&simulation.context.device, &simulation.context.queue)
                .unwrap(),
            vec![1, 1, 1, 1],
            "the touched chunk and its clipped 8-neighbor halo are runnable"
        );

        for (material, expected_temperature) in [
            (MATERIAL_ICE, ICE_PLACEMENT_TEMPERATURE),
            (MATERIAL_STEAM, STEAM_PLACEMENT_TEMPERATURE),
        ] {
            runtime
                .edits
                .queue_stroke(cell, cell, 1, SandboxEditKind::Erase)
                .unwrap();
            runtime.edits.apply_pending(&simulation).unwrap();
            runtime
                .edits
                .queue_stroke(cell, cell, 1, SandboxEditKind::Draw(material))
                .unwrap();
            runtime.edits.apply_pending(&simulation).unwrap();
            for buffer in [
                &simulation.world.temperature_current,
                &simulation.world.temperature_next,
            ] {
                assert_eq!(
                    f32::from_ne_bytes(read_word(&simulation, buffer, index)),
                    expected_temperature,
                    "direct phase-Matter placement must start in its stable band"
                );
            }
        }
        runtime
            .edits
            .queue_stroke(cell, cell, 1, SandboxEditKind::Erase)
            .unwrap();
        runtime.edits.apply_pending(&simulation).unwrap();
        runtime
            .edits
            .queue_stroke(cell, cell, 1, SandboxEditKind::Draw(MATERIAL_STONE))
            .unwrap();
        runtime.edits.apply_pending(&simulation).unwrap();

        runtime
            .edits
            .queue_stroke(cell, cell, 1, SandboxEditKind::Heat(HEAT_DELTA))
            .unwrap();
        runtime.edits.apply_pending(&simulation).unwrap();
        assert_eq!(
            simulation
                .world
                .read_material_cell(&simulation.context.device, &simulation.context.queue, 5, 5)
                .unwrap(),
            MATERIAL_STONE
        );
        assert_eq!(
            simulation
                .world
                .read_temperature_cell(&simulation.context.device, &simulation.context.queue, 5, 5)
                .unwrap(),
            TEMPERATURE_REFERENCE + HEAT_DELTA
        );
        simulation.tick().unwrap();
        let after_tick_temperature = simulation
            .world
            .read_temperature_cell(&simulation.context.device, &simulation.context.queue, 5, 5)
            .unwrap();
        assert!(after_tick_temperature.is_finite());
        assert_eq!(
            simulation.tick_count, 1,
            "the committed edit precedes exactly the next normal production tick"
        );
        assert!(simulation
            .world
            .read_material_all(&simulation.context.device, &simulation.context.queue)
            .unwrap()
            .into_iter()
            .all(is_valid_cell_material_value));
        assert!(simulation
            .world
            .read_temperature_all(&simulation.context.device, &simulation.context.queue)
            .unwrap()
            .into_iter()
            .all(f32::is_finite));
        assert!(simulation
            .world
            .read_pressure_all(&simulation.context.device, &simulation.context.queue)
            .unwrap()
            .into_iter()
            .all(f32::is_finite));

        runtime
            .edits
            .queue_stroke(cell, cell, 1, SandboxEditKind::Erase)
            .unwrap();
        runtime.edits.apply_pending(&simulation).unwrap();
        runtime
            .edits
            .queue_stroke(cell, cell, 1, SandboxEditKind::Heat(HEAT_DELTA))
            .unwrap();
        runtime.edits.apply_pending(&simulation).unwrap();
        assert_eq!(
            simulation
                .world
                .read_temperature_cell(&simulation.context.device, &simulation.context.queue, 5, 5)
                .unwrap(),
            TEMPERATURE_REFERENCE,
            "EMPTY is never turned into a hidden thermal medium"
        );
        let empty_air = simulation
            .world
            .read_environment_cells(
                &simulation.context.device,
                &simulation.context.queue,
                &[(5, 5)],
            )
            .unwrap()[0];
        assert_eq!(empty_air.current, powdergame_core::standard_air_state());
        assert_eq!(empty_air.current, empty_air.next);

        runtime
            .edits
            .queue_stroke(cell, cell, 1, SandboxEditKind::Draw(MATERIAL_SAND))
            .unwrap();
        runtime
            .edits
            .queue_stroke(cell, cell, 1, SandboxEditKind::Draw(MATERIAL_WATER))
            .unwrap();
        assert_eq!(
            runtime.edits.pending_count(),
            1,
            "rapid edits coalesce by cell"
        );
        runtime.request_preset(SandboxPreset::BlankWorld);
        assert_eq!(
            runtime.edits.pending_count(),
            0,
            "preset/reset cancels pending edits"
        );
        assert_eq!(runtime.pending_preset, Some(SandboxPreset::BlankWorld));
        assert_eq!(
            simulation.tick_count, 1,
            "edit batching itself never advances simulation"
        );

        let before = runtime.edits.pending_count();
        assert!(runtime
            .edits
            .queue_stroke(cell, cell, 1, SandboxEditKind::Draw(u32::MAX))
            .is_err());
        assert_eq!(
            runtime.edits.pending_count(),
            before,
            "failed validation is atomic"
        );
    }

    #[test]
    fn gpu_preset_switch_and_reset_restore_exact_pristine_images() {
        let config = config();
        let mut simulation = pollster::block_on(Simulation::new(config)).unwrap();
        for preset in [SandboxPreset::StarterLab, SandboxPreset::BlankWorld] {
            stage_preset(&mut simulation, preset).unwrap();
            let expected = preset_image(preset, config).unwrap();
            assert_eq!(
                simulation
                    .world
                    .read_material_all(&simulation.context.device, &simulation.context.queue)
                    .unwrap(),
                expected.materials
            );
            assert!(simulation
                .world
                .read_temperature_all(&simulation.context.device, &simulation.context.queue)
                .unwrap()
                .into_iter()
                .all(|value| value == TEMPERATURE_REFERENCE));
            assert!(simulation
                .world
                .read_pressure_all(&simulation.context.device, &simulation.context.queue)
                .unwrap()
                .into_iter()
                .all(|value| value == 0.0));
            assert!(simulation
                .world
                .read_flags_all(&simulation.context.device, &simulation.context.queue)
                .unwrap()
                .into_iter()
                .all(|value| value == 0));
            let observations = simulation
                .world
                .read_environment_cells(
                    &simulation.context.device,
                    &simulation.context.queue,
                    &[(0, 0), (128, 96), (20, 235)],
                )
                .unwrap();
            assert_eq!(observations[0].current.mass, 0.0);
            assert_eq!(
                observations[1].current,
                powdergame_core::standard_air_state()
            );
            let expected_floor_air = if expected.materials
                [(235 * SANDBOX_WORLD_WIDTH + 20) as usize]
                == MATERIAL_EMPTY
            {
                powdergame_core::standard_air_state()
            } else {
                powdergame_core::vacuum_air_state()
            };
            assert_eq!(observations[2].current, expected_floor_air);
            assert!(observations.iter().all(|cell| cell.current == cell.next));
            assert_eq!(simulation.tick_count, 0);
        }
    }
}
