from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise SystemExit(f"expected exactly one anchor in {path}: {old[:80]!r}, found {text.count(old)}")
    p.write_text(text.replace(old, new, 1), encoding="utf-8")


def write_new(path: str, content: str) -> None:
    p = Path(path)
    if p.exists():
        raise SystemExit(f"refusing to overwrite existing file: {path}")
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(content, encoding="utf-8")


rupture_rs = r'''//! G5-C — pressure stress / structural rupture CPU reference helpers.
//!
//! Pressure remains a spatial field. Structural Matter does NOT store
//! pressure; instead it reads the pressure in its four orthogonal neighbors
//! and decides whether its own cell ruptures. The production path is WGSL;
//! these functions define the cheap Material-data contract for tests/tools.

use crate::material::{registry_lookup, MATERIAL_REGISTRY};
use crate::pressure::sanitize_pressure;

/// G5-C M0 weak-wall baseline. Relative gameplay pressure scalar, not SI.
/// One fully blocked Water→Steam expansion produces 100 pressure, so Wood
/// ruptures from that event while Stone/Boundary remain reference walls.
pub const WOOD_RUPTURE_THRESHOLD: f32 = 80.0;

/// Looks up a Material-owned rupture threshold. `None` means unbreakable in
/// the current M0 grammar (including Boundary Block and Stone).
pub fn rupture_threshold(material_id: u32) -> Option<f32> {
    registry_lookup(material_id).and_then(|m| m.rupture_threshold)
}

/// Compact GPU table. `0.0` means this Material does not rupture from
/// pressure in G5-C. This is Material data, never a per-cell strength field.
pub fn rupture_threshold_table() -> [f32; 16] {
    let mut table = [0.0f32; 16];
    for material in MATERIAL_REGISTRY {
        if let Some(value) = material.rupture_threshold {
            table[material.id as usize] = value.max(0.0);
        }
    }
    table
}

/// Pure Read-Neighbors → Write-Self rupture decision.
///
/// `neighbor_pressures` are only samples from pressure-medium neighbors;
/// callers pass `None` for EMPTY, Static/Powder or Void. Threshold equality
/// counts as rupture so the descriptor is the minimum pressure strength.
pub fn should_rupture(material_id: u32, neighbor_pressures: [Option<f32>; 4]) -> bool {
    let Some(limit) = rupture_threshold(material_id) else {
        return false;
    };
    if !limit.is_finite() || limit <= 0.0 {
        return false;
    }
    neighbor_pressures
        .into_iter()
        .flatten()
        .map(sanitize_pressure)
        .any(|pressure| pressure >= limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{
        MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_OIL, MATERIAL_SAND,
        MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD,
    };

    #[test]
    fn only_wood_is_weak_structure_in_m0_g5c() {
        assert_eq!(rupture_threshold(MATERIAL_WOOD), Some(WOOD_RUPTURE_THRESHOLD));
        for id in [
            MATERIAL_EMPTY,
            MATERIAL_BOUNDARY_BLOCK,
            MATERIAL_STONE,
            MATERIAL_SAND,
            MATERIAL_WATER,
            MATERIAL_OIL,
            MATERIAL_STEAM,
            MATERIAL_SMOKE,
            MATERIAL_ICE,
        ] {
            assert_eq!(rupture_threshold(id), None, "material {id} should be unbreakable/non-structural in G5-C baseline");
        }
    }

    #[test]
    fn sub_threshold_pressure_does_not_rupture_wood() {
        assert!(!should_rupture(
            MATERIAL_WOOD,
            [Some(WOOD_RUPTURE_THRESHOLD - 0.01), None, None, None]
        ));
    }

    #[test]
    fn threshold_pressure_ruptures_wood() {
        assert!(should_rupture(
            MATERIAL_WOOD,
            [None, Some(WOOD_RUPTURE_THRESHOLD), None, None]
        ));
    }

    #[test]
    fn unbreakable_material_ignores_extreme_pressure() {
        assert!(!should_rupture(
            MATERIAL_STONE,
            [Some(1.0e6), Some(1.0e6), Some(1.0e6), Some(1.0e6)]
        ));
        assert!(!should_rupture(
            MATERIAL_BOUNDARY_BLOCK,
            [Some(1.0e6), None, None, None]
        ));
    }

    #[test]
    fn gpu_table_matches_material_descriptor() {
        let table = rupture_threshold_table();
        assert_eq!(table[MATERIAL_WOOD as usize], WOOD_RUPTURE_THRESHOLD);
        assert_eq!(table[MATERIAL_STONE as usize], 0.0);
        assert_eq!(table[MATERIAL_BOUNDARY_BLOCK as usize], 0.0);
        assert_eq!(table[MATERIAL_WATER as usize], 0.0);
    }
}
'''

rupture_wgsl = r'''// G5-C — generic structural rupture from neighboring scalar Pressure.
//
// A structure never becomes a Pressure medium. It reads the four orthogonal
// neighbor cells and, if any Liquid/Gas pressure reaches its Material-owned
// rupture threshold, writes only its own Matter cell to EMPTY. The opening
// then participates in ordinary movement on following ticks; no special
// explosion/vent code is required.

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> pressure_current: array<f32>;
@group(0) @binding(3) var<storage, read> rupture_threshold_table: array<f32>;
@group(0) @binding(4) var<storage, read> movement_class_table: array<u32>;
@group(0) @binding(5) var<storage, read_write> material_next: array<u32>;
@group(0) @binding(6) var<storage, read_write> temperature_next: array<f32>;
@group(0) @binding(7) var<storage, read_write> flags_next: array<u32>;

const EMPTY: u32 = 0u;
const TABLE_LEN: u32 = 16u;
const CLASS_LIQUID: u32 = 2u;
const CLASS_GAS: u32 = 3u;
const PRESSURE_REFERENCE: f32 = 0.0;
const PRESSURE_MAX: f32 = 1.0e6;
const TEMPERATURE_REFERENCE: f32 = 0.0;

fn sanitize_pressure(value: f32) -> f32 {
    if (value != value || value > 1.0e20 || value < -1.0e20) {
        return PRESSURE_REFERENCE;
    }
    return clamp(value, PRESSURE_REFERENCE, PRESSURE_MAX);
}

fn in_domain(x: i32, y: i32) -> bool {
    return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
}

fn index_of(x: i32, y: i32) -> u32 {
    return u32(y) * params.width + u32(x);
}

fn is_pressure_medium(material: u32) -> bool {
    if (material == EMPTY || material >= TABLE_LEN) {
        return false;
    }
    let movement_kind = movement_class_table[material];
    return movement_kind == CLASS_LIQUID || movement_kind == CLASS_GAS;
}

fn neighbor_pressure(x: i32, y: i32) -> f32 {
    if (!in_domain(x, y)) {
        return PRESSURE_REFERENCE;
    }
    let n = index_of(x, y);
    if (!is_pressure_medium(material_current[n])) {
        return PRESSURE_REFERENCE;
    }
    return sanitize_pressure(pressure_current[n]);
}

@compute @workgroup_size(64, 1, 1)
fn rupture_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) {
        return;
    }

    let material = material_current[index];
    if (material == EMPTY || material >= TABLE_LEN) {
        material_next[index] = material;
        return;
    }

    let rupture_limit = rupture_threshold_table[material];
    if (!(rupture_limit > 0.0)) {
        material_next[index] = material;
        return;
    }

    let x = i32(index % params.width);
    let y = i32(index / params.width);
    var local_stress = PRESSURE_REFERENCE;
    local_stress = max(local_stress, neighbor_pressure(x, y - 1));
    local_stress = max(local_stress, neighbor_pressure(x, y + 1));
    local_stress = max(local_stress, neighbor_pressure(x - 1, y));
    local_stress = max(local_stress, neighbor_pressure(x + 1, y));

    if (local_stress >= rupture_limit) {
        material_next[index] = EMPTY;
        temperature_next[index] = TEMPERATURE_REFERENCE;
        flags_next[index] = 0u;
    } else {
        material_next[index] = material;
    }
}
'''

rupture_tests = r'''//! G5-C — structural rupture / opening / vent GPU integration tests.
//!
//! Requires Windows + RTX 5090 + DX12. G5-C adds only a generic structural
//! self-write rule: finite-strength Matter reads neighboring Liquid/Gas
//! pressure and becomes EMPTY at its descriptor threshold. Venting then
//! emerges from ordinary Matter movement through that opening.

use powdergame_core::{
    WorldConfig, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_STEAM, MATERIAL_STONE,
    MATERIAL_WATER, MATERIAL_WOOD, PRESSURE_REFERENCE, WATER_BOIL_BLOCKED_PRESSURE,
    WOOD_RUPTURE_THRESHOLD,
};
use powdergame_gpu::Simulation;

fn make_sim(config: WorldConfig) -> Simulation {
    pollster::block_on(Simulation::new(config)).expect("DX12 + RTX 5090 simulation init")
}

fn eight_by_eight() -> Simulation {
    make_sim(WorldConfig::new(8, 8, 8).unwrap())
}

fn set(sim: &Simulation, x: i64, y: i64, material: u32) {
    sim.world
        .write_material(&sim.context.queue, x, y, material)
        .expect("material edit");
}

fn set_t(sim: &Simulation, x: i64, y: i64, value: f32) {
    sim.world
        .write_temperature(&sim.context.queue, x, y, value)
        .expect("temperature edit");
}

fn set_p(sim: &Simulation, x: i64, y: i64, value: f32) {
    sim.world
        .write_pressure(&sim.context.queue, x, y, value)
        .expect("pressure edit");
}

fn cell(sim: &Simulation, x: i64, y: i64) -> u32 {
    sim.world
        .read_material_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("material readback")
}

fn pressure(sim: &Simulation, x: i64, y: i64) -> f32 {
    sim.world
        .read_pressure_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("pressure readback")
}

fn block_water_motion_except_top_wall(sim: &Simulation, wall_material: u32) {
    // Water at (3,3). Liquid candidates down/down-diagonal/lateral are Stone;
    // the top cell (3,2) is the structural wall stressed by Pressure.
    set(sim, 3, 2, wall_material);
    for (x, y) in [(2, 3), (4, 3), (2, 4), (3, 4), (4, 4)] {
        set(sim, x, y, MATERIAL_STONE);
    }
    set(sim, 3, 3, MATERIAL_WATER);
}

#[test]
fn wood_survives_sub_threshold_pressure() {
    let mut sim = eight_by_eight();
    block_water_motion_except_top_wall(&sim, MATERIAL_WOOD);
    set_p(&sim, 3, 3, WOOD_RUPTURE_THRESHOLD - 1.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 2), MATERIAL_WOOD);
    assert!(pressure(&sim, 3, 3) < WOOD_RUPTURE_THRESHOLD);
}

#[test]
fn wood_ruptures_from_threshold_exceeding_neighbor_pressure() {
    let mut sim = eight_by_eight();
    block_water_motion_except_top_wall(&sim, MATERIAL_WOOD);
    set_p(&sim, 3, 3, WATER_BOIL_BLOCKED_PRESSURE);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 2), MATERIAL_EMPTY, "weak wall opened");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_WATER, "pressure stress alone does not transmute the medium");
}

#[test]
fn stone_and_boundary_remain_reference_unbreakable_walls() {
    // Stone intentionally remains unbreakable in M0 because frozen G5-A
    // pressure fixtures use Stone containment up to PRESSURE_MAX.
    let mut sim = eight_by_eight();
    block_water_motion_except_top_wall(&sim, MATERIAL_STONE);
    set_p(&sim, 3, 3, 1.0e6);
    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 3, 2), MATERIAL_STONE);

    let mut sim = eight_by_eight();
    set(sim, 3, 1, MATERIAL_WATER);
    for (x, y) in [(2, 1), (4, 1), (2, 2), (3, 2), (4, 2)] {
        set(sim, x, y, MATERIAL_STONE);
    }
    set_p(&sim, 3, 1, 1.0e6);
    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 3, 0), MATERIAL_BOUNDARY_BLOCK);
}

#[test]
fn rupture_crosses_64_cell_chunk_boundary() {
    let mut sim = make_sim(WorldConfig::new(128, 16, 64).unwrap());
    // Pressure medium on x=63 stresses Wood on x=64 across the chunk edge.
    set(&sim, 63, 8, MATERIAL_WATER);
    set(&sim, 64, 8, MATERIAL_WOOD);
    for (x, y) in [(62, 8), (62, 9), (63, 9), (64, 9)] {
        set(&sim, x, y, MATERIAL_STONE);
    }
    set_p(&sim, 63, 8, WATER_BOIL_BLOCKED_PRESSURE);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 64, 8), MATERIAL_EMPTY, "chunk edge is not a stress wall");
}

#[test]
fn blocked_boiling_ruptures_weak_wall_then_vents_on_following_tick() {
    let mut sim = eight_by_eight();
    // One weak top wall; every other 8-neighbor is occupied so G5-B cannot
    // satisfy Water→Steam yield=2. Above the weak wall is ordinary EMPTY.
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 80.0);
    set(&sim, 3, 2, MATERIAL_WOOD);
    for (x, y) in [
        (2, 2), (4, 2),
        (2, 3), (4, 3),
        (2, 4), (3, 4), (4, 4),
    ] {
        set(&sim, x, y, MATERIAL_STONE);
    }

    // Tick 1: hot Water cannot expand, becomes Steam +100 pressure, then
    // the neighboring Wood reads that pressure and ruptures to EMPTY.
    sim.tick().expect("boil + confinement + rupture tick");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_STEAM, "water boiled in place");
    assert_eq!(cell(&sim, 3, 2), MATERIAL_EMPTY, "weak wall opened from pressure");
    let confined = pressure(&sim, 3, 3);
    assert!(
        confined >= WOOD_RUPTURE_THRESHOLD,
        "confinement pressure must exist before vent movement; got {confined}"
    );

    // Tick 2: ordinary GAS movement sees the newly EMPTY opening and moves
    // Steam into it. Because Pressure is spatial (not transported with
    // Matter), the vacated source pressure is cleared by the G5-A pass.
    sim.tick().expect("vent movement tick");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_EMPTY, "pressurized source volume vented");
    assert_eq!(cell(&sim, 3, 2), MATERIAL_STEAM, "steam moved through the rupture opening");
    assert_eq!(pressure(&sim, 3, 3), PRESSURE_REFERENCE, "vacated spatial pressure released");
}
'''

write_new("engine/core/src/rupture.rs", rupture_rs)
write_new("engine/gpu/src/rupture.wgsl", rupture_wgsl)
write_new("engine/gpu/tests/rupture.rs", rupture_tests)

# Core module/export wiring.
replace_once(
    "engine/core/src/lib.rs",
    "pub mod pressure;\npub mod thermal;",
    "pub mod pressure;\npub mod rupture;\npub mod thermal;",
)
replace_once(
    "engine/core/src/lib.rs",
    "pub use pressure::{\n    is_pressure_medium, pressure_step, sanitize_pressure, PressureNeighbor,\n    PRESSURE_DIFFUSION_RATE, PRESSURE_MAX, PRESSURE_REFERENCE,\n};\n",
    "pub use pressure::{\n    is_pressure_medium, pressure_step, sanitize_pressure, PressureNeighbor,\n    PRESSURE_DIFFUSION_RATE, PRESSURE_MAX, PRESSURE_REFERENCE,\n};\npub use rupture::{\n    rupture_threshold, rupture_threshold_table, should_rupture, WOOD_RUPTURE_THRESHOLD,\n};\n",
)

# Material-owned structural strength. `None` means unbreakable/non-structural.
material_path = Path("engine/core/src/material.rs")
material = material_path.read_text(encoding="utf-8")
field_anchor = "    pub decay: Option<DecayDescriptor>,\n}"
if material.count(field_anchor) != 1:
    raise SystemExit("MaterialDescriptor decay field anchor mismatch")
material = material.replace(
    field_anchor,
    "    pub decay: Option<DecayDescriptor>,\n"
    "    /// Generic G5-C structural rupture threshold in gameplay Pressure units.\n"
    "    /// `None` means Pressure cannot rupture this Matter in the M0 baseline.\n"
    "    pub rupture_threshold: Option<f32>,\n}",
    1,
)

thresholds = {
    "BOUNDARY_BLOCK": "None",
    "STONE": "None",
    "SAND": "None",
    "WATER": "None",
    "OIL": "None",
    "STEAM": "None",
    "SMOKE": "None",
    "ICE": "None",
    "WOOD": "Some(crate::rupture::WOOD_RUPTURE_THRESHOLD)",
}
for name, value in thresholds.items():
    needle = f"        id: MATERIAL_{name},"
    start = material.find(needle)
    if start < 0:
        raise SystemExit(f"material block not found: {name}")
    next_block = material.find("\n    MaterialDescriptor {", start + len(needle))
    end = next_block if next_block >= 0 else material.find("\n];", start)
    if end < 0:
        raise SystemExit(f"material block end not found: {name}")
    segment = material[start:end]
    if "rupture_threshold:" in segment:
        raise SystemExit(f"rupture threshold already present: {name}")
    close = segment.rfind("\n    },")
    if close < 0:
        raise SystemExit(f"descriptor close not found: {name}")
    segment = segment[:close] + f"\n        rupture_threshold: {value}," + segment[close:]
    material = material[:start] + segment + material[end:]
material_path.write_text(material, encoding="utf-8")

# GPU simulation wiring.
sim = "engine/gpu/src/simulation.rs"
replace_once(
    sim,
    "    movement_class_table, phase_descriptor_table, WorldConfig,\n",
    "    movement_class_table, phase_descriptor_table, rupture_threshold_table, WorldConfig,\n",
)
replace_once(sim, "    pressure_pipeline: wgpu::ComputePipeline,\n", "    pressure_pipeline: wgpu::ComputePipeline,\n    rupture_pipeline: wgpu::ComputePipeline,\n")
replace_once(sim, "    pressure_bind_group: wgpu::BindGroup,\n", "    pressure_bind_group: wgpu::BindGroup,\n    rupture_bind_group: wgpu::BindGroup,\n")

pressure_shader_anchor = '''        let shader_pressure = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g5a-pressure"),
                source: wgpu::ShaderSource::Wgsl(include_str!("pressure.wgsl").into()),
            });
'''
replace_once(
    sim,
    pressure_shader_anchor,
    pressure_shader_anchor + '''
        let shader_rupture = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g5c-rupture"),
                source: wgpu::ShaderSource::Wgsl(include_str!("rupture.wgsl").into()),
            });
''',
)

pressure_layout_anchor = '''        let pressure_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g5a-pressure-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // pressure_current
                        buffer_entry(3, &BindingKind::ReadWrite), // pressure_next
                        buffer_entry(4, &BindingKind::Read), // movement_class_table
                    ],
                });
'''
replace_once(
    sim,
    pressure_layout_anchor,
    pressure_layout_anchor + '''
        let rupture_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g5c-rupture-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // pressure_current
                        buffer_entry(3, &BindingKind::Read), // rupture threshold table
                        buffer_entry(4, &BindingKind::Read), // movement class table
                        buffer_entry(5, &BindingKind::ReadWrite), // material_next
                        buffer_entry(6, &BindingKind::ReadWrite), // temperature_next
                        buffer_entry(7, &BindingKind::ReadWrite), // flags_next
                    ],
                });
''',
)

pressure_pipeline_anchor = '''        let pressure_pipeline = make_pipeline(
            "powdergame-g5a-pressure",
            &pressure_layout,
            &shader_pressure,
            "pressure_main",
        );
'''
replace_once(
    sim,
    pressure_pipeline_anchor,
    pressure_pipeline_anchor + '''        let rupture_pipeline = make_pipeline(
            "powdergame-g5c-rupture",
            &rupture_layout,
            &shader_rupture,
            "rupture_main",
        );
''',
)

class_table_anchor = '''        context.queue.write_buffer(&class_table, 0, &class_data);
'''
replace_once(
    sim,
    class_table_anchor,
    class_table_anchor + '''
        // G5-C Material-owned structural rupture thresholds (0 = unbreakable).
        let mut rupture_data = [0u8; TABLE_SIZE as usize];
        for (i, value) in rupture_threshold_table().iter().enumerate() {
            let off = i * 4;
            rupture_data[off..off + 4].copy_from_slice(&value.to_ne_bytes());
        }
        let rupture_table_buf = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g5c/rupture/threshold-table"),
            size: TABLE_SIZE,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context.queue.write_buffer(&rupture_table_buf, 0, &rupture_data);
''',
)

pressure_bg_anchor = '''        let pressure_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g5a-pressure-bg"),
                layout: &pressure_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: world.material_current.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: world.pressure_current.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: world.pressure_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: class_table.as_entire_binding(),
                    },
                ],
            });
'''
replace_once(
    sim,
    pressure_bg_anchor,
    pressure_bg_anchor + '''
        let rupture_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g5c-rupture-bg"),
                layout: &rupture_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: params.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: world.material_current.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: world.pressure_current.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: rupture_table_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: class_table.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 5, resource: world.material_next.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 6, resource: world.temperature_next.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 7, resource: world.flags_next.as_entire_binding() },
                ],
            });
''',
)

replace_once(sim, "            pressure_pipeline,\n            propose_bind_group,", "            pressure_pipeline,\n            rupture_pipeline,\n            propose_bind_group,")
replace_once(sim, "            pressure_bind_group,\n\n            marker,", "            pressure_bind_group,\n            rupture_bind_group,\n\n            marker,")

# Tick documentation and G5-C pass after pressure propagation.
replace_once(
    sim,
    "    /// → scalar pressure 4-neighbor propagation → copy pressure Next→Current\n    /// ```",
    "    /// → scalar pressure 4-neighbor propagation → copy pressure Next→Current\n    /// → structural rupture (neighbor Pressure → self EMPTY) → opening\n    /// ```",
)
replace_once(
    sim,
    "    /// structural stress/rupture remains G5-C.\n",
    "    /// G5-C structural rupture runs after pressure propagation; ordinary\n    /// movement through the resulting EMPTY opening provides venting.\n",
)

pressure_tick_anchor = '''        encoder.copy_buffer_to_buffer(
            &self.world.pressure_next,
            0,
            &self.world.pressure_current,
            0,
            self.world.layout.pressure_bytes,
        );

        self.context.queue.submit([encoder.finish()]);
'''
replace_once(
    sim,
    pressure_tick_anchor,
    '''        encoder.copy_buffer_to_buffer(
            &self.world.pressure_next,
            0,
            &self.world.pressure_current,
            0,
            self.world.layout.pressure_bytes,
        );

        // G5-C: weak structural Matter reads settled neighboring Pressure
        // and may self-write to EMPTY. The new opening becomes authoritative
        // before the next tick's ordinary movement pass.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g5c-rupture-pass"),
                timestamp_writes: None,
            });
            dispatch(&mut pass, &self.rupture_pipeline, &self.rupture_bind_group);
        }
        encoder.copy_buffer_to_buffer(
            &self.world.material_next,
            0,
            &self.world.material_current,
            0,
            self.world.layout.material_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.temperature_next,
            0,
            &self.world.temperature_current,
            0,
            self.world.layout.temperature_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.flags_next,
            0,
            &self.world.flags_current,
            0,
            self.world.layout.flags_bytes,
        );

        self.context.queue.submit([encoder.finish()]);
''',
)

# WGSL parser regression includes the new production shader.
wgsl = "engine/gpu/tests/wgsl_parse.rs"
replace_once(
    wgsl,
    '        ("pressure.wgsl", include_str!("../src/pressure.wgsl")),\n',
    '        ("pressure.wgsl", include_str!("../src/pressure.wgsl")),\n        ("rupture.wgsl", include_str!("../src/rupture.wgsl")),\n',
)

# STATUS: keep G5 overall IN_PROGRESS, record frozen sub-gates and current G5-C.
status = Path("docs/planning/STATUS.md")
text = status.read_text(encoding="utf-8")
old_status = "`IN_PROGRESS` — G0 (Runtime) PASS, G1 (World Integrity) PASS, G2 (Local Movement) PASS / CLOSED, G3 (Density / Displacement) PASS / CLOSED, G4 (Thermal / Phase / Combustion) PASS / CLOSED (G4-A thermal baseline TECHNICAL PASS, G4-B phase transition TECHNICAL PASS, G4-C combustion & finite fuel TECHNICAL PASS, Smoke decay lifecycle G4 integration hardening TECHNICAL PASS, G4 Large 4-Panel Thermal Observatory `--thermal-demo` User Validation APPROVED on 2026-08-16)."
new_status = old_status + " G5 (Pressure Chain) IN_PROGRESS — G5-A Pressure Field TECHNICAL PASS / FROZEN, G5-B Expansion / Confinement → Pressure TECHNICAL PASS / FROZEN, G5-C Rupture / Opening / Vent implementation & validation in progress."
if text.count(old_status) != 1:
    raise SystemExit("STATUS current milestone anchor mismatch")
text = text.replace(old_status, new_status, 1)
old_phase = "**G0 — Runtime: PASS. G1 — World Integrity: PASS. G2 — Local Movement: PASS / CLOSED. G3 — Density / Displacement: PASS / CLOSED. G4 — Thermal / Phase / Combustion: PASS / CLOSED (G4-A thermal baseline TECHNICAL PASS, G4-B phase transition TECHNICAL PASS, G4-C combustion TECHNICAL PASS, Smoke decay G4 integration hardening TECHNICAL PASS, G4 4-Panel Thermal Observatory `--thermal-demo` (320×192) User Validation APPROVED on 2026-08-16).**"
new_phase = old_phase[:-2] + " G5 — Pressure Chain: IN_PROGRESS (G5-A Pressure Field TECHNICAL PASS / FROZEN; G5-B Expansion / Confinement → Pressure TECHNICAL PASS / FROZEN; G5-C Rupture / Opening / Vent IN_PROGRESS).**"
if text.count(old_phase) != 1:
    raise SystemExit("STATUS phase anchor mismatch")
text = text.replace(old_phase, new_phase, 1)
old_next = "다음 단계는 **G5 — Pressure Chain** (Phase expansion / yield / Pressure / rupture / vent)이다."
new_next = "현재 **G5 — Pressure Chain** 진행 중이다. G5-A scalar Pressure propagation과 G5-B Phase expansion / confinement → Pressure generation은 RTX 5090 / DX12 실기 검증으로 TECHNICAL PASS / FROZEN이며, 다음 sub-gate는 **G5-C — Pressure stress → rupture → opening → venting**이다."
if text.count(old_next) != 1:
    raise SystemExit("STATUS next-stage anchor mismatch")
text = text.replace(old_next, new_next, 1)
status.write_text(text, encoding="utf-8")

print("G5-C rupture/opening/vent implementation applied")
