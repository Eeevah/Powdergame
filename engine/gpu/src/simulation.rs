//! Headless GPU Simulation lifecycle.
//!
//! `Simulation` owns the production GPU context and dense world, and can be
//! created and ticked without any Window, Surface or Renderer
//! (`docs/architecture/ARCHITECTURE.md` §8, MILESTONES G0).
//!
//! G2 `tick()` runs the local-movement pipeline over the full world:
//! propose → resolve → commit, then a GPU-side `material_next →
//! material_current` copy. G3 generalizes ownership to bidirectional edge
//! claims (propose → claim → commit) so normal moves AND density swaps go
//! through the same safe path, with a per-cell scratch `claim` buffer. The
//! GPU remains the authoritative simulation path; the CPU only orchestrates
//! and stages edits.
//!
//! G4-A: movement commit transports temperature with Matter (same
//! ownership edge, write-self). After both Current buffers are updated,
//! a 4-neighbor thermal pass conducts into `temperature_next`. EMPTY is
//! not a thermal medium.
//!
//! G4-B: after conduction, a phase transition pass (self-write only,
//! Material-owned temperature rules, Ice ↔ Water ↔ Steam) transforms
//! `material_next` from the settled `material_current` + temperature.
//!
//! G4-C: movement commit also transports Matter-owned combustion flags on
//! the same ownership edge. After phase, a combustion pass (self-write)
//! updates temperature/flags from the generic Material combustion table and
//! requests at most one local Smoke spawn per burning source; a smoke
//! claim/commit pair (reusing the movement `proposal`/`claim` scratch,
//! safe because the passes are sequential) spawns Smoke with exactly one
//! winner per destination. Pressure is a spatial field (G5) and is never
//! transported on movement edges.
//!
//! G5-A adds scalar pressure propagation after Matter/phase/combustion settle:
//! Liquid/Gas cells exchange pressure with 4-neighbor Liquid/Gas cells via
//! Read Neighbors / Write Self. EMPTY/Static/Powder do not transmit it.
//!
//! Causal order per tick: movement (Matter carries Temperature + flags) →
//! thermal conduction → phase transition → combustion → smoke spawn → pressure.
//! Blocked expansion generation and rupture remain G5-B/G5-C.

use powdergame_core::{
    chunk_count, chunks_x, chunks_y, conductivity_table, decay_table, density_table,
    heat_capacity_table, movement_class_table, phase_descriptor_table, rupture_threshold_table,
    EnvironmentBoundaryMode, WorldConfig, ACTIVITY_MATTER, ACTIVITY_PRESSURE, ACTIVITY_REACTION,
    ACTIVITY_THERMAL, CHUNK_STATE_RUNNABLE, CHUNK_STATE_SLEEPING, DEFAULT_SLEEP_THRESHOLD_TICKS,
    PRESSURE_ACTIVITY_EPS, THERMAL_ACTIVITY_EPS,
};

use crate::context::{GpuContext, GpuError};
use crate::profiler::{GpuProfiler, ProfiledTickReport, QUERY_COUNT};
use crate::world::GpuWorld;

/// Workgroup size of the movement shaders.
const WORKGROUP_SIZE: u32 = 64;
/// Workgroups along X per dispatch row. Kept well below the DX12 limit of
/// 65535 while still covering the reference world in two rows.
const WORKGROUPS_X: u32 = 256;
/// Total threads per dispatch row (`WORKGROUPS_X * WORKGROUP_SIZE`).
const THREADS_X: u64 = (WORKGROUPS_X as u64) * (WORKGROUP_SIZE as u64);
/// Params uniform size: cell_count, threads_x, width, height, chunk_size, chunks_x, chunks_y, sleep_enabled (8 u32) = 32 bytes.
const PARAMS_SIZE: u64 = 32;
/// TE-2 Environment params extend common geometry with boundary mode and pad.
const ENVIRONMENT_PARAMS_SIZE: u64 = 48;
/// Arbitration uniform size: tick (u32) + 3 pad u32 = 16 bytes.
const ARBITRATION_PARAMS_SIZE: u64 = 16;
/// Material table buffer size (16 u32 entries each).
const TABLE_SIZE: u64 = 64;
/// 16 pairs of conductivity/capacity f32 values.
const THERMAL_TABLE_SIZE: u64 = 128;
/// Phase descriptor table: 16 descriptors × 32 bytes (G5-B yield/confinement metadata).
const PHASE_TABLE_SIZE: u64 = 512;
/// Combustion descriptor uniform table: 16 descriptors × 32 bytes = 512 bytes.
const COMBUSTION_TABLE_SIZE: u64 = 512;
/// Decay descriptor table: 16 descriptors × 8 bytes.
const DECAY_TABLE_SIZE: u64 = 128;
/// Size of the diagnostic marker buffer (one `u32` + padding).
const MARKER_SIZE: u64 = 16;
/// G7-A activity params uniform: cell_count, threads_x, width, height,
/// chunk_size, chunks_x, chunks_y, thermal_eps, pressure_eps + 3 pad = 48 B.
const ACTIVITY_PARAMS_SIZE: u64 = 48;
/// G7-B activity wake params uniform: chunks_x, chunks_y, sleep_enabled, sleep_threshold = 16 B.
const WAKE_PARAMS_SIZE: u64 = 16;
/// Persistent simulation-owned uniforms, marker, and material/property tables.
///
/// Keep this inventory beside the allocation-size constants so the G8 tracked
/// memory report cannot silently omit a buffer category. It covers all 14
/// persistent buffers allocated below: params, wake params, arbitration params,
/// activity params, marker, five 64-byte material tables, phase descriptors,
/// the combined activity table, decay descriptors, and combustion descriptors.
const TRACKED_UNIFORMS_AND_TABLES_SIZE: u64 = PARAMS_SIZE
    + ENVIRONMENT_PARAMS_SIZE
    + WAKE_PARAMS_SIZE
    + ARBITRATION_PARAMS_SIZE
    + ACTIVITY_PARAMS_SIZE
    + MARKER_SIZE
    + TABLE_SIZE // movement class
    + TABLE_SIZE // rupture threshold
    + TABLE_SIZE // density
    + TABLE_SIZE // conductivity
    + TABLE_SIZE // heat capacity
    + THERMAL_TABLE_SIZE // combined TE-2 thermal table
    + PHASE_TABLE_SIZE
    + (PHASE_TABLE_SIZE + TABLE_SIZE) // activity phase + conductivity table
    + DECAY_TABLE_SIZE
    + COMBUSTION_TABLE_SIZE;
/// Upper bound for cell indices so the claim encoding `(peer << 2) | kind`
/// can never overflow or collide with sentinels.
const MAX_CELL_COUNT: u64 = 1 << 30;

/// Buffer binding kind used to build the movement bind group layouts.
enum BindingKind {
    Uniform,
    Read,
    ReadWrite,
}

/// Builds a compute-stage uniform binding entry with an explicit minimum
/// size.
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

/// Builds a compute-stage buffer binding entry.
fn buffer_entry(binding: u32, kind: &BindingKind) -> wgpu::BindGroupLayoutEntry {
    let ty = match kind {
        BindingKind::Uniform => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        BindingKind::Read => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        BindingKind::ReadWrite => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
    };
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty,
        count: None,
    }
}

/// Headless GPU simulation: context + dense world + movement pipeline.
pub struct Simulation {
    pub context: GpuContext,
    pub world: GpuWorld,

    propose_pipeline: wgpu::ComputePipeline,
    claim_pipeline: wgpu::ComputePipeline,
    commit_pipeline: wgpu::ComputePipeline,
    material_flag_hygiene_pipeline: wgpu::ComputePipeline,
    movement_environment_reconcile_pipeline: wgpu::ComputePipeline,
    phase_energy_reconcile_movement_pipeline: wgpu::ComputePipeline,
    air_flow_scale_pipeline: wgpu::ComputePipeline,
    air_transport_pipeline: wgpu::ComputePipeline,
    thermal_stability_pipeline: wgpu::ComputePipeline,
    unified_thermal_pipeline: wgpu::ComputePipeline,
    phase_context_pipeline: wgpu::ComputePipeline,
    phase_pipeline: wgpu::ComputePipeline,
    phase_energy_hygiene_pipeline: wgpu::ComputePipeline,
    expansion_claim_pipeline: wgpu::ComputePipeline,
    expansion_environment_receiver_claim_pipeline: wgpu::ComputePipeline,
    expansion_spawn_commit_pipeline: wgpu::ComputePipeline,
    expansion_pressure_pipeline: wgpu::ComputePipeline,
    environment_blocked_expansion_pressure_pipeline: wgpu::ComputePipeline,
    expansion_environment_reconcile_pipeline: wgpu::ComputePipeline,
    identity_environment_reconcile_pipeline: wgpu::ComputePipeline,
    decay_pipeline: wgpu::ComputePipeline,
    ignition_exposure_propose_pipeline: wgpu::ComputePipeline,
    combustion_pipeline: wgpu::ComputePipeline,
    smoke_claim_pipeline: wgpu::ComputePipeline,
    smoke_environment_receiver_claim_pipeline: wgpu::ComputePipeline,
    smoke_commit_pipeline: wgpu::ComputePipeline,
    smoke_environment_reconcile_pipeline: wgpu::ComputePipeline,
    pressure_pipeline: wgpu::ComputePipeline,
    rupture_pipeline: wgpu::ComputePipeline,
    activity_propose_pipeline: wgpu::ComputePipeline,
    phase_activity_propose_pipeline: wgpu::ComputePipeline,
    environment_activity_propose_pipeline: wgpu::ComputePipeline,
    ignition_activity_propose_pipeline: wgpu::ComputePipeline,
    activity_reduce_pipeline: wgpu::ComputePipeline,
    activity_wake_pipeline: wgpu::ComputePipeline,
    propose_bind_group: wgpu::BindGroup,
    claim_bind_group: wgpu::BindGroup,
    commit_bind_group: wgpu::BindGroup,
    material_flag_hygiene_bind_group: wgpu::BindGroup,
    movement_environment_reconcile_bind_group: wgpu::BindGroup,
    phase_energy_reconcile_movement_bind_group: wgpu::BindGroup,
    air_flow_scale_bind_group: wgpu::BindGroup,
    air_transport_bind_group: wgpu::BindGroup,
    thermal_stability_bind_group: wgpu::BindGroup,
    unified_thermal_bind_group: wgpu::BindGroup,
    phase_context_bind_group: wgpu::BindGroup,
    phase_bind_group: wgpu::BindGroup,
    phase_energy_hygiene_bind_group: wgpu::BindGroup,
    expansion_claim_bind_group: wgpu::BindGroup,
    environment_receiver_claim_bind_group: wgpu::BindGroup,
    expansion_spawn_commit_bind_group: wgpu::BindGroup,
    expansion_pressure_bind_group: wgpu::BindGroup,
    environment_blocked_expansion_pressure_bind_group: wgpu::BindGroup,
    environment_spawn_reconcile_bind_group: wgpu::BindGroup,
    identity_environment_reconcile_bind_group: wgpu::BindGroup,
    decay_bind_group: wgpu::BindGroup,
    ignition_exposure_propose_bind_group: wgpu::BindGroup,
    combustion_bind_group: wgpu::BindGroup,

    smoke_claim_bind_group: wgpu::BindGroup,
    smoke_commit_bind_group: wgpu::BindGroup,
    pressure_bind_group: wgpu::BindGroup,
    rupture_bind_group: wgpu::BindGroup,
    activity_propose_bind_group: wgpu::BindGroup,
    phase_activity_propose_bind_group: wgpu::BindGroup,
    environment_activity_propose_bind_group: wgpu::BindGroup,
    ignition_activity_propose_bind_group: wgpu::BindGroup,
    activity_reduce_bind_group: wgpu::BindGroup,
    activity_wake_bind_group: wgpu::BindGroup,
    pub params: wgpu::Buffer,
    pub environment_params: wgpu::Buffer,
    pub wake_params: wgpu::Buffer,
    pub arbitration_params: wgpu::Buffer,
    marker: wgpu::Buffer,

    pub sleep_enabled: bool,
    pub sleep_threshold: u32,
    pub environment_boundary_mode: EnvironmentBoundaryMode,
    /// Number of ticks submitted since creation.
    pub tick_count: u64,
}

impl Simulation {
    /// Creates a new DX12 context, allocates the dense world and builds the
    /// movement pipeline. Headless: no window required.
    pub async fn new(config: WorldConfig) -> Result<Self, GpuError> {
        let context = GpuContext::new().await?;
        Self::with_context(context, config)
    }

    /// Builds a simulation on an existing GPU context.
    pub fn with_context(context: GpuContext, config: WorldConfig) -> Result<Self, GpuError> {
        let world = GpuWorld::new(&context.device, config)?;
        if world.layout.cell_count >= MAX_CELL_COUNT {
            return Err(GpuError::Other(format!(
                "world cell count {} exceeds the claim-encoding bound",
                world.layout.cell_count
            )));
        }

        // One explicit shader module per pass (no Rust brace/string scanner):
        // each module declares only its own bindings, so the layouts line up
        // 1:1 and G3 debugging stays simple.
        let shader_propose = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g3-movement-propose"),
                source: wgpu::ShaderSource::Wgsl(include_str!("movement_propose.wgsl").into()),
            });
        let shader_claim = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g3-movement-claim"),
                source: wgpu::ShaderSource::Wgsl(include_str!("movement_claim.wgsl").into()),
            });
        let shader_commit = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g3-movement-commit"),
                source: wgpu::ShaderSource::Wgsl(include_str!("movement_commit.wgsl").into()),
            });
        let shader_material_flag_hygiene =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te1-material-flag-hygiene"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("material_flag_hygiene.wgsl").into(),
                    ),
                });
        let shader_movement_environment_reconcile =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te1-movement-environment-reconcile"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("environment_reconcile_movement.wgsl").into(),
                    ),
                });
        let shader_phase_energy_reconcile_movement =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te3-phase-energy-reconcile-movement"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("phase_energy_reconcile_movement.wgsl").into(),
                    ),
                });
        let shader_air_flow_scale =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te2-air-flow-scale"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("air_flow_scale.wgsl").into()),
                });
        let shader_air_transport =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te2-air-transport"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("air_transport_commit.wgsl").into(),
                    ),
                });
        let shader_thermal_stability =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te2-thermal-stability"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("thermal_stability_scale.wgsl").into(),
                    ),
                });
        let shader_unified_thermal =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te2-unified-thermal"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("unified_thermal_commit.wgsl").into(),
                    ),
                });
        let shader_phase_context =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te3-phase-context"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("phase_context_propose.wgsl").into(),
                    ),
                });
        let shader_phase = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g4b-g5b-phase"),
                source: wgpu::ShaderSource::Wgsl(include_str!("phase_transition.wgsl").into()),
            });
        let shader_phase_energy_hygiene =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te3-phase-energy-hygiene"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("phase_energy_hygiene_identity.wgsl").into(),
                    ),
                });
        let shader_expansion_claim =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-g5b-expansion-claim"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("expansion_claim.wgsl").into()),
                });
        let shader_environment_receiver_claim =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te1-environment-receiver-claim"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("environment_receiver_claim.wgsl").into(),
                    ),
                });
        let shader_expansion_spawn_commit =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-g5b-expansion-spawn-commit"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("expansion_spawn_commit.wgsl").into(),
                    ),
                });
        let shader_expansion_pressure =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-g5b-expansion-pressure"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("expansion_pressure.wgsl").into(),
                    ),
                });
        let shader_environment_blocked_expansion_pressure =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te1-environment-blocked-expansion-pressure"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("environment_blocked_expansion_pressure.wgsl").into(),
                    ),
                });
        let shader_environment_spawn_reconcile =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te1-environment-spawn-reconcile"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("environment_reconcile_spawn.wgsl").into(),
                    ),
                });
        let shader_identity_environment_reconcile =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te1-identity-environment-reconcile"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("environment_reconcile_identity.wgsl").into(),
                    ),
                });
        let shader_decay = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g4d-decay"),
                source: wgpu::ShaderSource::Wgsl(include_str!("decay.wgsl").into()),
            });
        let shader_ignition_exposure_propose =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te4i-ignition-exposure-propose"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("ignition_exposure_propose.wgsl").into(),
                    ),
                });
        let shader_combustion = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g4c-combustion"),
                source: wgpu::ShaderSource::Wgsl(include_str!("combustion.wgsl").into()),
            });
        let shader_smoke_claim =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-g4c-smoke-claim"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("smoke_claim.wgsl").into()),
                });
        let shader_smoke_commit =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-g4c-smoke-commit"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("smoke_commit.wgsl").into()),
                });

        let shader_pressure = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g5a-pressure"),
                source: wgpu::ShaderSource::Wgsl(include_str!("pressure.wgsl").into()),
            });

        let shader_rupture = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g5c-rupture"),
                source: wgpu::ShaderSource::Wgsl(include_str!("rupture.wgsl").into()),
            });

        let shader_activity_propose =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-g7a-activity-propose"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("activity_propose.wgsl").into()),
                });
        let shader_phase_activity_propose =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te3-phase-activity-propose"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("phase_activity_propose.wgsl").into(),
                    ),
                });
        let shader_activity_reduce =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-g7a-activity-reduce"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("activity_reduce.wgsl").into()),
                });
        let shader_environment_activity_propose =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te2-environment-activity-propose"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("environment_activity_propose.wgsl").into(),
                    ),
                });
        let shader_ignition_activity_propose =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-te4i-ignition-activity-propose"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("ignition_activity_propose.wgsl").into(),
                    ),
                });
        let shader_activity_wake =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("powdergame-g7b-activity-wake"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("activity_wake.wgsl").into()),
                });

        // Bind group layouts.
        let propose_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g3-propose-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::ReadWrite), // proposal
                        buffer_entry(3, &BindingKind::ReadWrite), // marker
                        buffer_entry(4, &BindingKind::Read), // class_table
                        buffer_entry(5, &BindingKind::Read), // density_table
                        buffer_entry(6, &BindingKind::Read), // chunk_state
                    ],
                });
        let claim_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g3-claim-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // proposal
                        buffer_entry(2, &BindingKind::ReadWrite), // claim
                        buffer_entry(3, &BindingKind::Uniform), // arbitration
                        buffer_entry(4, &BindingKind::Read), // chunk_state
                    ],
                });
        let commit_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g3-commit-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // claim
                        buffer_entry(3, &BindingKind::ReadWrite), // material_next
                        buffer_entry(4, &BindingKind::Read), // temperature_current
                        buffer_entry(5, &BindingKind::ReadWrite), // temperature_next
                        buffer_entry(6, &BindingKind::Read), // flags_current
                        buffer_entry(7, &BindingKind::ReadWrite), // flags_next
                        buffer_entry(8, &BindingKind::Read), // chunk_state
                    ],
                });
        let material_flag_hygiene_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te1-material-flag-hygiene-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::ReadWrite),
                    ],
                });
        let movement_environment_reconcile_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te1-movement-environment-reconcile-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::Read),
                        buffer_entry(5, &BindingKind::Read),
                        buffer_entry(6, &BindingKind::ReadWrite),
                        buffer_entry(7, &BindingKind::ReadWrite),
                    ],
                });
        let phase_energy_reconcile_movement_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te3-phase-energy-reconcile-movement-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::Read),
                        buffer_entry(5, &BindingKind::ReadWrite),
                        buffer_entry(6, &BindingKind::Read),
                    ],
                });
        let air_flow_scale_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te2-air-flow-scale-bgl"),
                    entries: &[
                        uniform_entry(0, ENVIRONMENT_PARAMS_SIZE),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::Read),
                        buffer_entry(5, &BindingKind::ReadWrite),
                        buffer_entry(6, &BindingKind::ReadWrite),
                    ],
                });
        let air_transport_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te2-air-transport-bgl"),
                    entries: &[
                        uniform_entry(0, ENVIRONMENT_PARAMS_SIZE),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::Read),
                        buffer_entry(5, &BindingKind::Read),
                        buffer_entry(6, &BindingKind::ReadWrite),
                        buffer_entry(7, &BindingKind::ReadWrite),
                        buffer_entry(8, &BindingKind::Read),
                    ],
                });
        let thermal_stability_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te2-thermal-stability-bgl"),
                    entries: &[
                        uniform_entry(0, ENVIRONMENT_PARAMS_SIZE),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::Read),
                        uniform_entry(5, THERMAL_TABLE_SIZE),
                        buffer_entry(6, &BindingKind::Read),
                        buffer_entry(7, &BindingKind::ReadWrite),
                    ],
                });
        let unified_thermal_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te2-unified-thermal-bgl"),
                    entries: &[
                        uniform_entry(0, ENVIRONMENT_PARAMS_SIZE),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::Read),
                        uniform_entry(5, THERMAL_TABLE_SIZE),
                        buffer_entry(6, &BindingKind::Read),
                        buffer_entry(7, &BindingKind::ReadWrite),
                        buffer_entry(8, &BindingKind::ReadWrite),
                        buffer_entry(9, &BindingKind::Read),
                    ],
                });
        let phase_context_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te3-phase-context-bgl"),
                    entries: &[
                        uniform_entry(0, ENVIRONMENT_PARAMS_SIZE),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::Read),
                        buffer_entry(5, &BindingKind::Read),
                        buffer_entry(6, &BindingKind::Read),
                        buffer_entry(7, &BindingKind::ReadWrite),
                        buffer_entry(8, &BindingKind::Read),
                        uniform_entry(9, THERMAL_TABLE_SIZE),
                    ],
                });
        let phase_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te3-phase-thermodynamics-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::Read),
                        buffer_entry(5, &BindingKind::ReadWrite),
                        buffer_entry(6, &BindingKind::ReadWrite),
                        buffer_entry(7, &BindingKind::ReadWrite),
                        buffer_entry(8, &BindingKind::ReadWrite),
                    ],
                });
        let phase_energy_hygiene_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te3-phase-energy-hygiene-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::ReadWrite),
                    ],
                });
        let expansion_claim_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g5b-expansion-claim-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // proposal
                        buffer_entry(3, &BindingKind::ReadWrite), // claim
                        buffer_entry(4, &BindingKind::Uniform), // arbitration
                        buffer_entry(5, &BindingKind::Read), // chunk_state
                    ],
                });
        let environment_receiver_claim_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te1-environment-receiver-claim-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::Read),
                        buffer_entry(5, &BindingKind::ReadWrite),
                        buffer_entry(6, &BindingKind::Uniform),
                    ],
                });
        let expansion_spawn_commit_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g5b-expansion-spawn-commit-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // temperature_current
                        buffer_entry(3, &BindingKind::Read), // claim
                        buffer_entry(4, &BindingKind::ReadWrite), // material_next
                        buffer_entry(5, &BindingKind::ReadWrite), // temperature_next
                        buffer_entry(6, &BindingKind::ReadWrite), // flags_next
                        buffer_entry(7, &BindingKind::Read), // chunk_state
                        buffer_entry(8, &BindingKind::Read), // Environment receiver claim
                    ],
                });
        let expansion_pressure_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g5b-expansion-pressure-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // temperature_current
                        buffer_entry(3, &BindingKind::Read), // phase_table
                        buffer_entry(4, &BindingKind::Read), // proposal
                        buffer_entry(5, &BindingKind::Read), // claim
                        buffer_entry(6, &BindingKind::Read), // pressure_current
                        buffer_entry(7, &BindingKind::ReadWrite), // pressure_next
                        buffer_entry(8, &BindingKind::Read), // chunk_state
                    ],
                });
        let environment_blocked_expansion_pressure_layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("powdergame-te1-environment-blocked-expansion-pressure-bgl"),
                entries: &[
                    buffer_entry(0, &BindingKind::Uniform),
                    buffer_entry(1, &BindingKind::Read),
                    buffer_entry(2, &BindingKind::Read),
                    buffer_entry(3, &BindingKind::Read),
                    buffer_entry(4, &BindingKind::Read),
                    buffer_entry(5, &BindingKind::Read),
                    buffer_entry(6, &BindingKind::Read),
                    buffer_entry(7, &BindingKind::ReadWrite),
                ],
            });
        let environment_spawn_reconcile_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te1-environment-spawn-reconcile-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::Read),
                        buffer_entry(5, &BindingKind::Read),
                        buffer_entry(6, &BindingKind::Read),
                        buffer_entry(7, &BindingKind::ReadWrite),
                        buffer_entry(8, &BindingKind::ReadWrite),
                    ],
                });
        let identity_environment_reconcile_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te1-identity-environment-reconcile-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::Read),
                        buffer_entry(5, &BindingKind::ReadWrite),
                        buffer_entry(6, &BindingKind::ReadWrite),
                    ],
                });
        let decay_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g4d-decay-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // flags_current
                        buffer_entry(3, &BindingKind::Read), // temperature_current
                        buffer_entry(4, &BindingKind::Read), // decay_table
                        buffer_entry(5, &BindingKind::ReadWrite), // material_next
                        buffer_entry(6, &BindingKind::ReadWrite), // flags_next
                        buffer_entry(7, &BindingKind::ReadWrite), // temperature_next
                        buffer_entry(8, &BindingKind::Read), // chunk_state
                    ],
                });
        let ignition_exposure_propose_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te4i-ignition-exposure-propose-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // temperature_current
                        buffer_entry(3, &BindingKind::Read), // flags_current
                        buffer_entry(4, &BindingKind::Read), // air_mass_current
                        uniform_entry(5, COMBUSTION_TABLE_SIZE),
                        buffer_entry(6, &BindingKind::ReadWrite), // proposal context
                    ],
                });
        let combustion_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g4c-combustion-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // temperature_current
                        buffer_entry(3, &BindingKind::Read), // flags_current
                        uniform_entry(4, 512), // combustion_table (uniform to respect DX12 8 storage-buffer limit)
                        buffer_entry(5, &BindingKind::ReadWrite), // temperature_next
                        buffer_entry(6, &BindingKind::ReadWrite), // flags_next
                        buffer_entry(7, &BindingKind::ReadWrite), // proposal (smoke request)
                        buffer_entry(8, &BindingKind::ReadWrite), // material_next (consumed fuel)
                        buffer_entry(9, &BindingKind::Read), // chunk_state
                    ],
                });
        let smoke_claim_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g4c-smoke-claim-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // proposal (smoke request)
                        buffer_entry(3, &BindingKind::ReadWrite), // claim (smoke winner)
                        buffer_entry(4, &BindingKind::Uniform), // arbitration
                        buffer_entry(5, &BindingKind::Read), // chunk_state
                    ],
                });
        let smoke_commit_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g4c-smoke-commit-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // claim (smoke winner)
                        buffer_entry(3, &BindingKind::ReadWrite), // temperature_next
                        buffer_entry(4, &BindingKind::ReadWrite), // material_next
                        buffer_entry(5, &BindingKind::Read), // chunk_state
                        buffer_entry(6, &BindingKind::Read), // Environment receiver claim
                    ],
                });

        let pressure_layout =
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
                        buffer_entry(5, &BindingKind::Read), // chunk_state
                    ],
                });

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
                        buffer_entry(8, &BindingKind::Read), // chunk_state
                    ],
                });

        // G7-A activity passes (measurement baseline; G6 write-ownership
        // preserved: per-cell self-write, per-chunk self-write, no atomics).
        let activity_propose_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g7a-activity-propose-bgl"),
                    entries: &[
                        uniform_entry(0, ACTIVITY_PARAMS_SIZE),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // temperature_current
                        buffer_entry(3, &BindingKind::Read), // pressure_current
                        buffer_entry(4, &BindingKind::Read), // flags_current
                        buffer_entry(5, &BindingKind::Read), // class table
                        buffer_entry(6, &BindingKind::Read), // density table
                        buffer_entry(7, &BindingKind::ReadWrite), // cell_activity
                        buffer_entry(8, &BindingKind::Read), // phase + conductivity tables
                    ],
                });
        let phase_activity_propose_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te3-phase-activity-propose-bgl"),
                    entries: &[
                        uniform_entry(0, ENVIRONMENT_PARAMS_SIZE),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::Read),
                        buffer_entry(5, &BindingKind::Read),
                        buffer_entry(6, &BindingKind::Read),
                        buffer_entry(7, &BindingKind::Read),
                        buffer_entry(8, &BindingKind::ReadWrite),
                        uniform_entry(9, THERMAL_TABLE_SIZE),
                    ],
                });
        let activity_reduce_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g7a-activity-reduce-bgl"),
                    entries: &[
                        uniform_entry(0, ACTIVITY_PARAMS_SIZE),
                        buffer_entry(1, &BindingKind::Read), // cell_activity
                        buffer_entry(2, &BindingKind::ReadWrite), // chunk_activity
                        buffer_entry(3, &BindingKind::ReadWrite), // chunk_changed
                        buffer_entry(4, &BindingKind::ReadWrite), // chunk_stable
                    ],
                });
        let environment_activity_propose_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te2-environment-activity-propose-bgl"),
                    entries: &[
                        uniform_entry(0, ENVIRONMENT_PARAMS_SIZE),
                        buffer_entry(1, &BindingKind::Read),
                        buffer_entry(2, &BindingKind::Read),
                        buffer_entry(3, &BindingKind::Read),
                        buffer_entry(4, &BindingKind::Read),
                        uniform_entry(5, THERMAL_TABLE_SIZE),
                        buffer_entry(6, &BindingKind::ReadWrite),
                    ],
                });
        let ignition_activity_propose_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-te4i-ignition-activity-propose-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // temperature_current
                        buffer_entry(3, &BindingKind::Read), // flags_current
                        buffer_entry(4, &BindingKind::Read), // air_mass_current
                        uniform_entry(5, COMBUSTION_TABLE_SIZE),
                        buffer_entry(6, &BindingKind::ReadWrite), // cell_activity
                    ],
                });

        // G7-B activity wake pass: 1 thread per chunk evaluates self activity,
        // 8-neighbor activity halo, edit wakes, settling threshold, and sleep enable.
        let activity_wake_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g7b-activity-wake-bgl"),
                    entries: &[
                        uniform_entry(0, WAKE_PARAMS_SIZE),
                        buffer_entry(1, &BindingKind::Read), // chunk_activity
                        buffer_entry(2, &BindingKind::Read), // chunk_stable_ticks
                        buffer_entry(3, &BindingKind::Read), // chunk_edit_wake immutable wake snapshot
                        buffer_entry(4, &BindingKind::ReadWrite), // chunk_state
                        buffer_entry(5, &BindingKind::ReadWrite), // chunk_wake_reason
                    ],
                });

        let make_pipeline = |label: &str,
                             layout: &wgpu::BindGroupLayout,
                             module: &wgpu::ShaderModule,
                             entry: &str| {
            let pipeline_layout =
                context
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some(label),
                        bind_group_layouts: &[layout],
                        push_constant_ranges: &[],
                    });
            context
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(label),
                    layout: Some(&pipeline_layout),
                    module,
                    entry_point: Some(entry),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                })
        };

        let propose_pipeline = make_pipeline(
            "powdergame-g3-propose",
            &propose_layout,
            &shader_propose,
            "propose_main",
        );
        let claim_pipeline = make_pipeline(
            "powdergame-g3-claim",
            &claim_layout,
            &shader_claim,
            "claim_main",
        );
        let commit_pipeline = make_pipeline(
            "powdergame-g3-commit",
            &commit_layout,
            &shader_commit,
            "commit_main",
        );
        let material_flag_hygiene_pipeline = make_pipeline(
            "powdergame-te1-material-flag-hygiene",
            &material_flag_hygiene_layout,
            &shader_material_flag_hygiene,
            "material_flag_hygiene_main",
        );
        let movement_environment_reconcile_pipeline = make_pipeline(
            "powdergame-te1-movement-environment-reconcile",
            &movement_environment_reconcile_layout,
            &shader_movement_environment_reconcile,
            "movement_environment_reconcile_main",
        );
        let phase_energy_reconcile_movement_pipeline = make_pipeline(
            "powdergame-te3-phase-energy-reconcile-movement",
            &phase_energy_reconcile_movement_layout,
            &shader_phase_energy_reconcile_movement,
            "phase_energy_reconcile_movement_main",
        );
        let air_flow_scale_pipeline = make_pipeline(
            "powdergame-te2-air-flow-scale",
            &air_flow_scale_layout,
            &shader_air_flow_scale,
            "air_flow_scale_main",
        );
        let air_transport_pipeline = make_pipeline(
            "powdergame-te2-air-transport",
            &air_transport_layout,
            &shader_air_transport,
            "air_transport_commit_main",
        );
        let thermal_stability_pipeline = make_pipeline(
            "powdergame-te2-thermal-stability",
            &thermal_stability_layout,
            &shader_thermal_stability,
            "thermal_stability_scale_main",
        );
        let unified_thermal_pipeline = make_pipeline(
            "powdergame-te2-unified-thermal",
            &unified_thermal_layout,
            &shader_unified_thermal,
            "unified_thermal_commit_main",
        );
        let phase_context_pipeline = make_pipeline(
            "powdergame-te3-phase-context",
            &phase_context_layout,
            &shader_phase_context,
            "phase_context_propose_main",
        );
        let phase_pipeline = make_pipeline(
            "powdergame-te3-phase-thermodynamics",
            &phase_layout,
            &shader_phase,
            "phase_main",
        );
        let phase_energy_hygiene_pipeline = make_pipeline(
            "powdergame-te3-phase-energy-hygiene",
            &phase_energy_hygiene_layout,
            &shader_phase_energy_hygiene,
            "phase_energy_hygiene_identity_main",
        );
        let expansion_claim_pipeline = make_pipeline(
            "powdergame-g5b-expansion-claim",
            &expansion_claim_layout,
            &shader_expansion_claim,
            "expansion_claim_main",
        );
        let expansion_environment_receiver_claim_pipeline = make_pipeline(
            "powdergame-te1-expansion-environment-receiver-claim",
            &environment_receiver_claim_layout,
            &shader_environment_receiver_claim,
            "expansion_receiver_claim_main",
        );
        let expansion_spawn_commit_pipeline = make_pipeline(
            "powdergame-g5b-expansion-spawn-commit",
            &expansion_spawn_commit_layout,
            &shader_expansion_spawn_commit,
            "expansion_spawn_commit_main",
        );
        let expansion_pressure_pipeline = make_pipeline(
            "powdergame-g5b-expansion-pressure",
            &expansion_pressure_layout,
            &shader_expansion_pressure,
            "expansion_pressure_main",
        );
        let environment_blocked_expansion_pressure_pipeline = make_pipeline(
            "powdergame-te1-environment-blocked-expansion-pressure",
            &environment_blocked_expansion_pressure_layout,
            &shader_environment_blocked_expansion_pressure,
            "environment_blocked_expansion_pressure_main",
        );
        let expansion_environment_reconcile_pipeline = make_pipeline(
            "powdergame-te1-expansion-environment-reconcile",
            &environment_spawn_reconcile_layout,
            &shader_environment_spawn_reconcile,
            "expansion_environment_reconcile_main",
        );
        let identity_environment_reconcile_pipeline = make_pipeline(
            "powdergame-te1-identity-environment-reconcile",
            &identity_environment_reconcile_layout,
            &shader_identity_environment_reconcile,
            "identity_environment_reconcile_main",
        );
        let decay_pipeline = make_pipeline(
            "powdergame-g4d-decay",
            &decay_layout,
            &shader_decay,
            "decay_main",
        );
        let combustion_pipeline = make_pipeline(
            "powdergame-g4c-combustion",
            &combustion_layout,
            &shader_combustion,
            "combustion_main",
        );
        let ignition_exposure_propose_pipeline = make_pipeline(
            "powdergame-te4i-ignition-exposure-propose",
            &ignition_exposure_propose_layout,
            &shader_ignition_exposure_propose,
            "ignition_exposure_propose_main",
        );
        let smoke_claim_pipeline = make_pipeline(
            "powdergame-g4c-smoke-claim",
            &smoke_claim_layout,
            &shader_smoke_claim,
            "smoke_claim_main",
        );
        let smoke_environment_receiver_claim_pipeline = make_pipeline(
            "powdergame-te1-smoke-environment-receiver-claim",
            &environment_receiver_claim_layout,
            &shader_environment_receiver_claim,
            "smoke_receiver_claim_main",
        );
        let smoke_commit_pipeline = make_pipeline(
            "powdergame-g4c-smoke-commit",
            &smoke_commit_layout,
            &shader_smoke_commit,
            "smoke_commit_main",
        );
        let smoke_environment_reconcile_pipeline = make_pipeline(
            "powdergame-te1-smoke-environment-reconcile",
            &environment_spawn_reconcile_layout,
            &shader_environment_spawn_reconcile,
            "smoke_environment_reconcile_main",
        );

        let pressure_pipeline = make_pipeline(
            "powdergame-g5a-pressure",
            &pressure_layout,
            &shader_pressure,
            "pressure_main",
        );
        let rupture_pipeline = make_pipeline(
            "powdergame-g5c-rupture",
            &rupture_layout,
            &shader_rupture,
            "rupture_main",
        );
        let activity_propose_pipeline = make_pipeline(
            "powdergame-g7a-activity-propose",
            &activity_propose_layout,
            &shader_activity_propose,
            "propose_main",
        );
        let phase_activity_propose_pipeline = make_pipeline(
            "powdergame-te3-phase-activity-propose",
            &phase_activity_propose_layout,
            &shader_phase_activity_propose,
            "phase_activity_propose_main",
        );
        let activity_reduce_pipeline = make_pipeline(
            "powdergame-g7a-activity-reduce",
            &activity_reduce_layout,
            &shader_activity_reduce,
            "reduce_main",
        );
        let environment_activity_propose_pipeline = make_pipeline(
            "powdergame-te2-environment-activity-propose",
            &environment_activity_propose_layout,
            &shader_environment_activity_propose,
            "environment_activity_propose_main",
        );
        let ignition_activity_propose_pipeline = make_pipeline(
            "powdergame-te4i-ignition-activity-propose",
            &ignition_activity_propose_layout,
            &shader_ignition_activity_propose,
            "ignition_activity_propose_main",
        );
        let activity_wake_pipeline = make_pipeline(
            "powdergame-g7b-activity-wake",
            &activity_wake_layout,
            &shader_activity_wake,
            "wake_main",
        );

        // Params uniform: cell_count, threads_x, width, height, chunk_size, chunks_x, chunks_y, sleep_enabled.
        let cell_count_u32 = u32::try_from(world.layout.cell_count).map_err(|_| {
            GpuError::Other(format!(
                "world cell count {} does not fit in u32 for dispatch",
                world.layout.cell_count
            ))
        })?;
        let threads_x_u32 =
            u32::try_from(THREADS_X).map_err(|_| GpuError::Other("threads_x overflow".into()))?;
        let chunks_x_u32 = chunks_x(world.config.width, world.config.chunk_size);
        let chunks_y_u32 = chunks_y(world.config.height, world.config.chunk_size);

        let params = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g3/movement/params"),
            size: PARAMS_SIZE,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let environment_params = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("te2/environment/params"),
            size: ENVIRONMENT_PARAMS_SIZE,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let wake_params = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g7b/activity/wake-params"),
            size: WAKE_PARAMS_SIZE,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sleep_enabled = true;
        let sleep_threshold = DEFAULT_SLEEP_THRESHOLD_TICKS;
        let environment_boundary_mode = EnvironmentBoundaryMode::Sealed;

        let mut params_data = [0u8; PARAMS_SIZE as usize];
        params_data[..4].copy_from_slice(&cell_count_u32.to_ne_bytes());
        params_data[4..8].copy_from_slice(&threads_x_u32.to_ne_bytes());
        params_data[8..12].copy_from_slice(&world.config.width.to_ne_bytes());
        params_data[12..16].copy_from_slice(&world.config.height.to_ne_bytes());
        params_data[16..20].copy_from_slice(&world.config.chunk_size.to_ne_bytes());
        params_data[20..24].copy_from_slice(&chunks_x_u32.to_ne_bytes());
        params_data[24..28].copy_from_slice(&chunks_y_u32.to_ne_bytes());
        params_data[28..32]
            .copy_from_slice(&(if sleep_enabled { 1u32 } else { 0u32 }).to_ne_bytes());
        context.queue.write_buffer(&params, 0, &params_data);
        let mut environment_params_data = [0u8; ENVIRONMENT_PARAMS_SIZE as usize];
        environment_params_data[..32].copy_from_slice(&params_data);
        environment_params_data[32..36]
            .copy_from_slice(&(environment_boundary_mode as u32).to_ne_bytes());
        context
            .queue
            .write_buffer(&environment_params, 0, &environment_params_data);

        let mut wake_data = [0u8; WAKE_PARAMS_SIZE as usize];
        wake_data[..4].copy_from_slice(&chunks_x_u32.to_ne_bytes());
        wake_data[4..8].copy_from_slice(&chunks_y_u32.to_ne_bytes());
        wake_data[8..12].copy_from_slice(&(if sleep_enabled { 1u32 } else { 0u32 }).to_ne_bytes());
        wake_data[12..16].copy_from_slice(&sleep_threshold.to_ne_bytes());
        context.queue.write_buffer(&wake_params, 0, &wake_data);

        // G7-A activity params (48 B): cell_count, threads_x, width, height,
        // chunk_size, chunks_x, chunks_y, thermal_eps, pressure_eps + pads.
        let mut activity_params_data = [0u8; ACTIVITY_PARAMS_SIZE as usize];
        activity_params_data[..4].copy_from_slice(&cell_count_u32.to_ne_bytes());
        activity_params_data[4..8].copy_from_slice(&threads_x_u32.to_ne_bytes());
        activity_params_data[8..12].copy_from_slice(&world.config.width.to_ne_bytes());
        activity_params_data[12..16].copy_from_slice(&world.config.height.to_ne_bytes());
        activity_params_data[16..20].copy_from_slice(&world.config.chunk_size.to_ne_bytes());
        activity_params_data[20..24].copy_from_slice(&chunks_x_u32.to_ne_bytes());
        activity_params_data[24..28].copy_from_slice(&chunks_y_u32.to_ne_bytes());
        activity_params_data[28..32].copy_from_slice(&THERMAL_ACTIVITY_EPS.to_ne_bytes());
        activity_params_data[32..36].copy_from_slice(&PRESSURE_ACTIVITY_EPS.to_ne_bytes());
        let activity_params = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g7a/activity/params"),
            size: ACTIVITY_PARAMS_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context
            .queue
            .write_buffer(&activity_params, 0, &activity_params_data);

        // Movement-class table (read-only storage; EMPTY/unknown map to 0).
        let mut class_data = [0u8; TABLE_SIZE as usize];
        for (i, class) in movement_class_table().iter().enumerate() {
            let off = i * 4;
            class_data[off..off + 4].copy_from_slice(&class.to_ne_bytes());
        }
        let class_table = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g3/movement/class-table"),
            size: TABLE_SIZE,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context.queue.write_buffer(&class_table, 0, &class_data);

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
        context
            .queue
            .write_buffer(&rupture_table_buf, 0, &rupture_data);

        // Density-rank table (read-only storage; 0 = no movable density).
        // This is a Material property upload — there are no per-cell
        // density buffers (SIMULATION_SPEC §12).
        let mut density_data = [0u8; TABLE_SIZE as usize];
        for (i, rank) in density_table().iter().enumerate() {
            let off = i * 4;
            density_data[off..off + 4].copy_from_slice(&rank.to_ne_bytes());
        }
        let density_table_buf = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g3/movement/density-table"),
            size: TABLE_SIZE,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context
            .queue
            .write_buffer(&density_table_buf, 0, &density_data);

        // G4-A thermal property tables (Material cheap scalars, not per-cell).
        let mut conductivity_data = [0u8; TABLE_SIZE as usize];
        for (i, value) in conductivity_table().iter().enumerate() {
            let off = i * 4;
            conductivity_data[off..off + 4].copy_from_slice(&value.to_ne_bytes());
        }
        let conductivity_table_buf = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g4a/thermal/conductivity-table"),
            size: TABLE_SIZE,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context
            .queue
            .write_buffer(&conductivity_table_buf, 0, &conductivity_data);

        let mut capacity_data = [0u8; TABLE_SIZE as usize];
        for (i, value) in heat_capacity_table().iter().enumerate() {
            let off = i * 4;
            capacity_data[off..off + 4].copy_from_slice(&value.to_ne_bytes());
        }
        let capacity_table_buf = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g4a/thermal/capacity-table"),
            size: TABLE_SIZE,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context
            .queue
            .write_buffer(&capacity_table_buf, 0, &capacity_data);

        let mut thermal_data = [0u8; THERMAL_TABLE_SIZE as usize];
        let conductivity = conductivity_table();
        let capacities = heat_capacity_table();
        for index in 0..16usize {
            let offset = index * 8;
            thermal_data[offset..offset + 4].copy_from_slice(&conductivity[index].to_ne_bytes());
            thermal_data[offset + 4..offset + 8].copy_from_slice(&capacities[index].to_ne_bytes());
        }
        let thermal_table_buf = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("te2/thermal/combined-table"),
            size: THERMAL_TABLE_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context
            .queue
            .write_buffer(&thermal_table_buf, 0, &thermal_data);

        // G4-B/G5-B phase descriptor table (16 × 32 bytes; Material data,
        // not per-cell state): targets + matter yield + thresholds +
        // confinement pressure. No per-cell expansion buffer is added.
        let mut phase_data = [0u8; PHASE_TABLE_SIZE as usize];
        for (i, desc) in phase_descriptor_table().iter().enumerate() {
            let off = i * 32;
            phase_data[off..off + 4].copy_from_slice(&desc.below_target.to_ne_bytes());
            phase_data[off + 4..off + 8].copy_from_slice(&desc.above_target.to_ne_bytes());
            phase_data[off + 8..off + 12].copy_from_slice(&desc.below_yield.to_ne_bytes());
            phase_data[off + 12..off + 16].copy_from_slice(&desc.above_yield.to_ne_bytes());
            phase_data[off + 16..off + 20].copy_from_slice(&desc.below_threshold.to_ne_bytes());
            phase_data[off + 20..off + 24].copy_from_slice(&desc.above_threshold.to_ne_bytes());
            phase_data[off + 24..off + 28]
                .copy_from_slice(&desc.below_blocked_pressure.to_ne_bytes());
            phase_data[off + 28..off + 32]
                .copy_from_slice(&desc.above_blocked_pressure.to_ne_bytes());
        }
        let phase_table_buf = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g4b/phase/table"),
            size: PHASE_TABLE_SIZE,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context.queue.write_buffer(&phase_table_buf, 0, &phase_data);

        // G7-A combined detector tables: the phase descriptors (512 B)
        // followed by the conductivity table (64 B) in ONE storage buffer,
        // so the activity propose pass stays within the DX12 per-stage
        // storage-buffer limit (8). Both halves are the same read-only
        // Material-property tables the physics passes already use.
        let activity_tables_buf = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g7a/activity/tables"),
            size: PHASE_TABLE_SIZE + TABLE_SIZE,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context
            .queue
            .write_buffer(&activity_tables_buf, 0, &phase_data);
        context
            .queue
            .write_buffer(&activity_tables_buf, PHASE_TABLE_SIZE, &conductivity_data);

        // G4-D decay descriptor table (16 × 8 bytes; Material data,
        // not per-cell state). Generic: Smoke/transient matter share one grammar.
        let mut decay_data = [0u8; DECAY_TABLE_SIZE as usize];
        for (i, desc) in decay_table().iter().enumerate() {
            let off = i * 8;
            decay_data[off..off + 4].copy_from_slice(&desc.lifetime_ticks.to_ne_bytes());
            decay_data[off + 4..off + 8].copy_from_slice(&desc.target_material.to_ne_bytes());
        }
        let decay_table_buf = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g4d/decay/table"),
            size: DECAY_TABLE_SIZE,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context.queue.write_buffer(&decay_table_buf, 0, &decay_data);

        // G4-C combustion descriptor table (16 × 32 bytes aligned uniform buffer;
        // Material data, not per-cell state). Generic: Wood/Oil share one grammar.
        let combustion_data = powdergame_core::combustion_table_bytes();
        let combustion_table_buf = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g4c/combustion/table"),
            size: COMBUSTION_TABLE_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context
            .queue
            .write_buffer(&combustion_table_buf, 0, &combustion_data);

        // Diagnostic marker: 16 bytes, one u32 used.
        let marker = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g3/movement/marker"),
            size: MARKER_SIZE,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // G6-C2: Global simulation-owned arbitration parameter buffer (16 bytes, tick: u32).
        let arbitration_params = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g6/arbitration/params"),
            size: ARBITRATION_PARAMS_SIZE,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let propose_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g3-propose-bg"),
                layout: &propose_layout,
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
                        resource: world.proposal.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: marker.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: class_table.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: density_table_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: world.chunk_state.as_entire_binding(),
                    },
                ],
            });
        let claim_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g3-claim-bg"),
                layout: &claim_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: world.proposal.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: world.claim.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: arbitration_params.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: world.chunk_state.as_entire_binding(),
                    },
                ],
            });
        let commit_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g3-commit-bg"),
                layout: &commit_layout,
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
                        resource: world.claim.as_entire_binding(),
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
                        resource: world.flags_current.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: world.flags_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: world.chunk_state.as_entire_binding(),
                    },
                ],
            });
        let material_flag_hygiene_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te1-material-flag-hygiene-bg"),
                    layout: &material_flag_hygiene_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: world.material_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: world.flags_next.as_entire_binding(),
                        },
                    ],
                });
        let movement_environment_reconcile_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te1-movement-environment-reconcile-bg"),
                    layout: &movement_environment_reconcile_layout,
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
                            resource: world.material_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.air_mass_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.air_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.air_mass_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: world.air_energy_next.as_entire_binding(),
                        },
                    ],
                });
        let phase_energy_reconcile_movement_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te3-phase-energy-reconcile-movement-bg"),
                    layout: &phase_energy_reconcile_movement_layout,
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
                            resource: world.claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.phase_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.material_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.phase_energy_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.chunk_state.as_entire_binding(),
                        },
                    ],
                });
        let air_flow_scale_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te2-air-flow-scale-bg"),
                    layout: &air_flow_scale_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: environment_params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: world.material_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: world.air_mass_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.air_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.chunk_state.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.proposal.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.claim.as_entire_binding(),
                        },
                    ],
                });
        let air_transport_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te2-air-transport-bg"),
                    layout: &air_transport_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: environment_params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: world.material_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: world.air_mass_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.air_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.proposal.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.air_mass_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: world.air_energy_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: world.chunk_state.as_entire_binding(),
                        },
                    ],
                });
        let thermal_stability_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te2-thermal-stability-bg"),
                    layout: &thermal_stability_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: environment_params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: world.material_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: world.temperature_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.air_mass_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.air_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: thermal_table_buf.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.chunk_state.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: world.proposal.as_entire_binding(),
                        },
                    ],
                });
        let unified_thermal_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te2-unified-thermal-bg"),
                    layout: &unified_thermal_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: environment_params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: world.material_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: world.temperature_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.air_mass_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.air_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: thermal_table_buf.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.proposal.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: world.temperature_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: world.air_energy_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 9,
                            resource: world.chunk_state.as_entire_binding(),
                        },
                    ],
                });
        let phase_context_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te3-phase-context-bg"),
                    layout: &phase_context_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: environment_params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: world.material_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: world.temperature_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.phase_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.air_mass_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.air_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.chunk_state.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: world.claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: class_table.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 9,
                            resource: thermal_table_buf.as_entire_binding(),
                        },
                    ],
                });
        let phase_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g4b-phase-bg"),
                layout: &phase_layout,
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
                        resource: world.temperature_current.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: world.phase_energy_current.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: world.claim.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: world.material_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: world.temperature_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: world.phase_energy_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: world.proposal.as_entire_binding(),
                    },
                ],
            });
        let phase_energy_hygiene_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te3-phase-energy-hygiene-bg"),
                    layout: &phase_energy_hygiene_layout,
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
                            resource: world.material_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.phase_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.phase_energy_next.as_entire_binding(),
                        },
                    ],
                });
        let expansion_claim_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-g5b-expansion-claim-bg"),
                    layout: &expansion_claim_layout,
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
                            resource: world.proposal.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: arbitration_params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.chunk_state.as_entire_binding(),
                        },
                    ],
                });
        let environment_receiver_claim_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te1-environment-receiver-claim-bg"),
                    layout: &environment_receiver_claim_layout,
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
                            resource: world.claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.air_mass_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.air_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.environment_receiver_claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: arbitration_params.as_entire_binding(),
                        },
                    ],
                });
        let expansion_spawn_commit_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-g5b-expansion-spawn-commit-bg"),
                    layout: &expansion_spawn_commit_layout,
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
                            resource: world.temperature_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.material_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.temperature_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.flags_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: world.chunk_state.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: world.environment_receiver_claim.as_entire_binding(),
                        },
                    ],
                });
        let expansion_pressure_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-g5b-expansion-pressure-bg"),
                    layout: &expansion_pressure_layout,
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
                            resource: world.temperature_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: phase_table_buf.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.proposal.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.pressure_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: world.pressure_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: world.chunk_state.as_entire_binding(),
                        },
                    ],
                });
        let environment_blocked_expansion_pressure_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te1-environment-blocked-expansion-pressure-bg"),
                    layout: &environment_blocked_expansion_pressure_layout,
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
                            resource: world.temperature_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: phase_table_buf.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.proposal.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.environment_receiver_claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: world.pressure_next.as_entire_binding(),
                        },
                    ],
                });
        let environment_spawn_reconcile_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te1-environment-spawn-reconcile-bg"),
                    layout: &environment_spawn_reconcile_layout,
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
                            resource: world.material_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.environment_receiver_claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.air_mass_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.air_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: world.air_mass_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: world.air_energy_next.as_entire_binding(),
                        },
                    ],
                });
        let identity_environment_reconcile_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te1-identity-environment-reconcile-bg"),
                    layout: &identity_environment_reconcile_layout,
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
                            resource: world.material_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.air_mass_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.air_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.air_mass_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.air_energy_next.as_entire_binding(),
                        },
                    ],
                });
        let decay_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g4d-decay-bg"),
                layout: &decay_layout,
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
                        resource: world.flags_current.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: world.temperature_current.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: decay_table_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: world.material_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: world.flags_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: world.temperature_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: world.chunk_state.as_entire_binding(),
                    },
                ],
            });
        let combustion_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g4c-combustion-bg"),
                layout: &combustion_layout,
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
                        resource: world.temperature_current.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: world.flags_current.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: combustion_table_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: world.temperature_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: world.flags_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: world.proposal.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: world.material_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: world.chunk_state.as_entire_binding(),
                    },
                ],
            });
        let ignition_exposure_propose_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te4i-ignition-exposure-propose-bg"),
                    layout: &ignition_exposure_propose_layout,
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
                            resource: world.temperature_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.flags_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.air_mass_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: combustion_table_buf.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.proposal.as_entire_binding(),
                        },
                    ],
                });
        let smoke_claim_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g4c-smoke-claim-bg"),
                layout: &smoke_claim_layout,
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
                        resource: world.proposal.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: world.claim.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: arbitration_params.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: world.chunk_state.as_entire_binding(),
                    },
                ],
            });
        let smoke_commit_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-g4c-smoke-commit-bg"),
                    layout: &smoke_commit_layout,
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
                            resource: world.claim.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.temperature_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.material_next.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.chunk_state.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.environment_receiver_claim.as_entire_binding(),
                        },
                    ],
                });

        let pressure_bind_group = context
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
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: world.chunk_state.as_entire_binding(),
                    },
                ],
            });

        let rupture_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g5c-rupture-bg"),
                layout: &rupture_layout,
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
                        resource: rupture_table_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: class_table.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: world.material_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: world.temperature_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: world.flags_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: world.chunk_state.as_entire_binding(),
                    },
                ],
            });

        let activity_propose_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-g7a-activity-propose-bg"),
                    layout: &activity_propose_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: activity_params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: world.material_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: world.temperature_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.pressure_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.flags_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: class_table.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: density_table_buf.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: world.cell_activity.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: activity_tables_buf.as_entire_binding(),
                        },
                    ],
                });
        let phase_activity_propose_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te3-phase-activity-propose-bg"),
                    layout: &phase_activity_propose_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: environment_params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: world.material_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: world.temperature_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.phase_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.air_mass_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.air_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.chunk_state.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: class_table.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: world.cell_activity.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 9,
                            resource: thermal_table_buf.as_entire_binding(),
                        },
                    ],
                });
        let environment_activity_propose_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te2-environment-activity-propose-bg"),
                    layout: &environment_activity_propose_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: environment_params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: world.material_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: world.temperature_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.air_mass_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.air_energy_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: thermal_table_buf.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.cell_activity.as_entire_binding(),
                        },
                    ],
                });
        let ignition_activity_propose_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-te4i-ignition-activity-propose-bg"),
                    layout: &ignition_activity_propose_layout,
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
                            resource: world.temperature_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.flags_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.air_mass_current.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: combustion_table_buf.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: world.cell_activity.as_entire_binding(),
                        },
                    ],
                });
        let activity_reduce_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-g7a-activity-reduce-bg"),
                    layout: &activity_reduce_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: activity_params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: world.cell_activity.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: world.chunk_activity.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.chunk_changed_this_tick.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.chunk_stable_ticks.as_entire_binding(),
                        },
                    ],
                });

        let activity_wake_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("powdergame-g7b-activity-wake-bg"),
                    layout: &activity_wake_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wake_params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: world.chunk_activity.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: world.chunk_stable_ticks.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: world.chunk_edit_wake.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: world.chunk_state.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: world.chunk_wake_reason.as_entire_binding(),
                        },
                    ],
                });

        Ok(Self {
            context,
            world,
            propose_pipeline,
            claim_pipeline,
            commit_pipeline,
            material_flag_hygiene_pipeline,
            movement_environment_reconcile_pipeline,
            phase_energy_reconcile_movement_pipeline,
            air_flow_scale_pipeline,
            air_transport_pipeline,
            thermal_stability_pipeline,
            unified_thermal_pipeline,
            phase_context_pipeline,
            phase_pipeline,
            phase_energy_hygiene_pipeline,
            expansion_claim_pipeline,
            expansion_environment_receiver_claim_pipeline,
            expansion_spawn_commit_pipeline,
            expansion_pressure_pipeline,
            environment_blocked_expansion_pressure_pipeline,
            expansion_environment_reconcile_pipeline,
            identity_environment_reconcile_pipeline,
            decay_pipeline,
            ignition_exposure_propose_pipeline,
            combustion_pipeline,
            smoke_claim_pipeline,
            smoke_environment_receiver_claim_pipeline,
            smoke_commit_pipeline,
            smoke_environment_reconcile_pipeline,
            pressure_pipeline,
            rupture_pipeline,
            activity_propose_pipeline,
            phase_activity_propose_pipeline,
            environment_activity_propose_pipeline,
            ignition_activity_propose_pipeline,
            activity_reduce_pipeline,
            activity_wake_pipeline,
            propose_bind_group,
            claim_bind_group,
            commit_bind_group,
            material_flag_hygiene_bind_group,
            movement_environment_reconcile_bind_group,
            phase_energy_reconcile_movement_bind_group,
            air_flow_scale_bind_group,
            air_transport_bind_group,
            thermal_stability_bind_group,
            unified_thermal_bind_group,
            phase_context_bind_group,
            phase_bind_group,
            phase_energy_hygiene_bind_group,
            expansion_claim_bind_group,
            environment_receiver_claim_bind_group,
            expansion_spawn_commit_bind_group,
            expansion_pressure_bind_group,
            environment_blocked_expansion_pressure_bind_group,
            environment_spawn_reconcile_bind_group,
            identity_environment_reconcile_bind_group,
            decay_bind_group,
            ignition_exposure_propose_bind_group,
            combustion_bind_group,
            smoke_claim_bind_group,
            smoke_commit_bind_group,
            pressure_bind_group,
            rupture_bind_group,
            activity_propose_bind_group,
            phase_activity_propose_bind_group,
            environment_activity_propose_bind_group,
            ignition_activity_propose_bind_group,
            activity_reduce_bind_group,
            activity_wake_bind_group,
            params,
            environment_params,
            wake_params,
            arbitration_params,
            marker,
            sleep_enabled,
            sleep_threshold,
            environment_boundary_mode,
            tick_count: 0,
        })
    }

    /// Toggles chunk-level simulation sleep optimization.
    pub fn set_sleep_enabled(&mut self, enabled: bool) {
        self.sleep_enabled = enabled;
        self.update_uniforms();
    }

    /// Sets the number of consecutive stable ticks required before a chunk may sleep.
    pub fn set_sleep_threshold(&mut self, threshold: u32) {
        self.sleep_threshold = threshold;
        self.update_uniforms();
    }

    pub fn set_environment_boundary_mode(&mut self, mode: EnvironmentBoundaryMode) {
        self.environment_boundary_mode = mode;
        self.update_uniforms();
    }

    /// Resets the simulation world state, diagnostics scratch, and tick counter to tick 0.
    /// Preserves sleep optimization settings (`sleep_enabled`, `sleep_threshold`).
    pub fn reset(&mut self) -> Result<(), GpuError> {
        self.world.reset(&self.context.queue)?;
        self.tick_count = 0;
        let arb_bytes = [0u8; 16];
        self.context
            .queue
            .write_buffer(&self.arbitration_params, 0, &arb_bytes);
        self.update_uniforms();
        Ok(())
    }

    /// Updates the parameter and wake uniforms on the GPU.
    pub fn update_uniforms(&self) {
        let cell_count_u32 = self.world.layout.cell_count as u32;
        let threads_x_u32 = THREADS_X as u32;
        let chunks_x_u32 = chunks_x(self.world.config.width, self.world.config.chunk_size);
        let chunks_y_u32 = chunks_y(self.world.config.height, self.world.config.chunk_size);
        let sleep_enabled_u32 = if self.sleep_enabled { 1u32 } else { 0u32 };

        let mut params_data = [0u8; PARAMS_SIZE as usize];
        params_data[..4].copy_from_slice(&cell_count_u32.to_ne_bytes());
        params_data[4..8].copy_from_slice(&threads_x_u32.to_ne_bytes());
        params_data[8..12].copy_from_slice(&self.world.config.width.to_ne_bytes());
        params_data[12..16].copy_from_slice(&self.world.config.height.to_ne_bytes());
        params_data[16..20].copy_from_slice(&self.world.config.chunk_size.to_ne_bytes());
        params_data[20..24].copy_from_slice(&chunks_x_u32.to_ne_bytes());
        params_data[24..28].copy_from_slice(&chunks_y_u32.to_ne_bytes());
        params_data[28..32].copy_from_slice(&sleep_enabled_u32.to_ne_bytes());
        self.context
            .queue
            .write_buffer(&self.params, 0, &params_data);
        let mut environment_params_data = [0u8; ENVIRONMENT_PARAMS_SIZE as usize];
        environment_params_data[..32].copy_from_slice(&params_data);
        environment_params_data[32..36]
            .copy_from_slice(&(self.environment_boundary_mode as u32).to_ne_bytes());
        self.context
            .queue
            .write_buffer(&self.environment_params, 0, &environment_params_data);

        let mut wake_data = [0u8; WAKE_PARAMS_SIZE as usize];
        wake_data[..4].copy_from_slice(&chunks_x_u32.to_ne_bytes());
        wake_data[4..8].copy_from_slice(&chunks_y_u32.to_ne_bytes());
        wake_data[8..12].copy_from_slice(&sleep_enabled_u32.to_ne_bytes());
        wake_data[12..16].copy_from_slice(&self.sleep_threshold.to_ne_bytes());
        self.context
            .queue
            .write_buffer(&self.wake_params, 0, &wake_data);
    }

    /// Submits one tick on the GPU in production unprofiled mode (no CPU full-world copy).
    pub fn tick(&mut self) -> Result<(), GpuError> {
        self.tick_internal(None)
    }

    /// Submits one tick on the GPU in timestamp-profiled mode and reads every production pass timing.
    pub fn tick_profiled(
        &mut self,
        profiler: &mut GpuProfiler,
    ) -> Result<ProfiledTickReport, GpuError> {
        self.tick_internal(Some(profiler))?;
        profiler.readback_report(
            &self.context.device,
            self.tick_count - 1,
            self.context.timestamp_period,
        )
    }

    /// Shared internal tick implementation guaranteeing identical pass ordering, dispatch,
    /// copies, and semantics between production and profiled modes.
    ///
    /// Pipeline order:
    /// ```text
    /// 0. activity wake
    /// 1..6. movement ownership, Matter/Environment/phase-energy reconciliation
    /// 7..10. TE-2 Air transport and unified thermal transfer
    /// 11..19. phase context/thermodynamics, generic expansion transaction, hygiene
    /// 20..23. decay and Matter/phase/Environment hygiene
    /// 24..30. combustion/Smoke transaction and Matter/phase/Environment hygiene
    /// 31. pressure
    /// 32..35. rupture and Matter/phase/Environment hygiene
    /// 36..39. Matter/phase/Environment activity proposal and reduction
    /// ```
    fn tick_internal(&mut self, profiler: Option<&mut GpuProfiler>) -> Result<(), GpuError> {
        let cell_count = self.world.layout.cell_count;
        let dispatch_y = u32::try_from(cell_count.div_ceil(THREADS_X))
            .map_err(|_| GpuError::Other("dispatch height overflow".into()))?;

        // G6-C2: Update arbitration uniform once at tick start (low 32 bits of tick_count).
        let mut arb_bytes = [0u8; 16];
        arb_bytes[..4].copy_from_slice(&(self.tick_count as u32).to_ne_bytes());
        self.context
            .queue
            .write_buffer(&self.arbitration_params, 0, &arb_bytes);

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("powdergame-simulation-tick-encoder"),
                });

        let query_set_ref = profiler.as_ref().map(|p| &p.query_set);
        let make_timestamp_writes = |pass_idx: u32| {
            query_set_ref.map(|qs| wgpu::ComputePassTimestampWrites {
                query_set: qs,
                beginning_of_pass_write_index: Some(pass_idx * 2),
                end_of_pass_write_index: Some(pass_idx * 2 + 1),
            })
        };

        let dispatch = |pass: &mut wgpu::ComputePass<'_>,
                        pipeline: &wgpu::ComputePipeline,
                        bg: &wgpu::BindGroup| {
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bg, &[]);
            pass.dispatch_workgroups(WORKGROUPS_X, dispatch_y, 1);
        };

        // Pass 0: activity_wake
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g7b-activity-wake-pass"),
                timestamp_writes: make_timestamp_writes(0),
            });
            pass.set_pipeline(&self.activity_wake_pipeline);
            pass.set_bind_group(0, &self.activity_wake_bind_group, &[]);
            let a_chunks_x = chunks_x(self.world.config.width, self.world.config.chunk_size);
            let a_chunks_y = chunks_y(self.world.config.height, self.world.config.chunk_size);
            pass.dispatch_workgroups(a_chunks_x, a_chunks_y, 1);
        }

        // G7-B two-phase edit wake semantics:
        // every chunk observes the same immutable edit snapshot during the wake
        // dispatch; only after that pass ends do we consume the one-tick triggers.
        encoder.clear_buffer(&self.world.chunk_edit_wake, 0, None);

        // Pass 1: movement_propose
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g3-propose-pass"),
                timestamp_writes: make_timestamp_writes(1),
            });
            dispatch(&mut pass, &self.propose_pipeline, &self.propose_bind_group);
        }
        // Pass 2: movement_claim
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g3-claim-pass"),
                timestamp_writes: make_timestamp_writes(2),
            });
            dispatch(&mut pass, &self.claim_pipeline, &self.claim_bind_group);
        }
        // Pass 3: movement_commit
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g3-commit-pass"),
                timestamp_writes: make_timestamp_writes(3),
            });
            dispatch(&mut pass, &self.commit_pipeline, &self.commit_bind_group);
        }
        // Pass 4: exact Matter flag ownership after movement.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-material-flag-hygiene-movement-pass"),
                timestamp_writes: make_timestamp_writes(4),
            });
            dispatch(
                &mut pass,
                &self.material_flag_hygiene_pipeline,
                &self.material_flag_hygiene_bind_group,
            );
        }
        // Pass 5: movement Volume Exchange.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-movement-environment-reconcile-pass"),
                timestamp_writes: make_timestamp_writes(5),
            });
            dispatch(
                &mut pass,
                &self.movement_environment_reconcile_pipeline,
                &self.movement_environment_reconcile_bind_group,
            );
        }

        // Pass 6: Matter-owned phase energy follows the accepted movement edge.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te3-phase-energy-reconcile-movement-pass"),
                timestamp_writes: make_timestamp_writes(6),
            });
            dispatch(
                &mut pass,
                &self.phase_energy_reconcile_movement_pipeline,
                &self.phase_energy_reconcile_movement_bind_group,
            );
        }

        // Movement ownership is settled on Current before conduction.
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
        encoder.copy_buffer_to_buffer(
            &self.world.phase_energy_next,
            0,
            &self.world.phase_energy_current,
            0,
            self.world.layout.temperature_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.air_mass_next,
            0,
            &self.world.air_mass_current,
            0,
            self.world.layout.material_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.air_energy_next,
            0,
            &self.world.air_energy_current,
            0,
            self.world.layout.material_bytes,
        );

        // Pass 7: donor/receiver Air scales overwrite movement scratch.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te2-air-flow-scale-pass"),
                timestamp_writes: make_timestamp_writes(7),
            });
            dispatch(
                &mut pass,
                &self.air_flow_scale_pipeline,
                &self.air_flow_scale_bind_group,
            );
        }
        // Pass 8: conservative Air mass/energy transport.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te2-air-transport-pass"),
                timestamp_writes: make_timestamp_writes(8),
            });
            dispatch(
                &mut pass,
                &self.air_transport_pipeline,
                &self.air_transport_bind_group,
            );
        }
        encoder.copy_buffer_to_buffer(
            &self.world.air_mass_next,
            0,
            &self.world.air_mass_current,
            0,
            self.world.layout.material_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.air_energy_next,
            0,
            &self.world.air_energy_current,
            0,
            self.world.layout.material_bytes,
        );

        // Pass 9: thermal stability lambda overwrites proposal scratch.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te2-thermal-stability-pass"),
                timestamp_writes: make_timestamp_writes(9),
            });
            dispatch(
                &mut pass,
                &self.thermal_stability_pipeline,
                &self.thermal_stability_bind_group,
            );
        }
        // Pass 10: unified Matter/Air passive thermal exchange.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te2-unified-thermal-pass"),
                timestamp_writes: make_timestamp_writes(10),
            });
            dispatch(
                &mut pass,
                &self.unified_thermal_pipeline,
                &self.unified_thermal_bind_group,
            );
        }
        encoder.copy_buffer_to_buffer(
            &self.world.temperature_next,
            0,
            &self.world.temperature_current,
            0,
            self.world.layout.temperature_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.air_energy_next,
            0,
            &self.world.air_energy_current,
            0,
            self.world.layout.material_bytes,
        );

        // Pass 11: immutable Matter/Air phase context overwrites dead claim scratch.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te3-phase-context-pass"),
                timestamp_writes: make_timestamp_writes(11),
            });
            dispatch(
                &mut pass,
                &self.phase_context_pipeline,
                &self.phase_context_bind_group,
            );
        }

        // Pass 12: local phase enthalpy normalization.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te3-phase-thermodynamics-pass"),
                timestamp_writes: make_timestamp_writes(12),
            });
            dispatch(&mut pass, &self.phase_pipeline, &self.phase_bind_group);
        }
        // Pass 13: expansion_claim
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g5b-expansion-claim-pass"),
                timestamp_writes: make_timestamp_writes(13),
            });
            dispatch(
                &mut pass,
                &self.expansion_claim_pipeline,
                &self.expansion_claim_bind_group,
            );
        }
        // Pass 14: Environment receiver claim (scratch is fully rewritten).
        encoder.clear_buffer(&self.world.environment_receiver_claim, 0, None);
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-expansion-environment-receiver-claim-pass"),
                timestamp_writes: make_timestamp_writes(14),
            });
            dispatch(
                &mut pass,
                &self.expansion_environment_receiver_claim_pipeline,
                &self.environment_receiver_claim_bind_group,
            );
        }
        // Pass 15: expansion_spawn_commit (receiver gated)
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g5b-expansion-spawn-commit-pass"),
                timestamp_writes: make_timestamp_writes(15),
            });
            dispatch(
                &mut pass,
                &self.expansion_spawn_commit_pipeline,
                &self.expansion_spawn_commit_bind_group,
            );
        }
        // Pass 16: existing expansion_pressure
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g5b-expansion-pressure-pass"),
                timestamp_writes: make_timestamp_writes(16),
            });
            dispatch(
                &mut pass,
                &self.expansion_pressure_pipeline,
                &self.expansion_pressure_bind_group,
            );
        }
        // Pass 17: original Matter winner whose Environment receiver failed.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-environment-blocked-expansion-pressure-pass"),
                timestamp_writes: make_timestamp_writes(17),
            });
            dispatch(
                &mut pass,
                &self.environment_blocked_expansion_pressure_pipeline,
                &self.environment_blocked_expansion_pressure_bind_group,
            );
        }
        // Pass 18: phase/spawn identity flag hygiene.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-material-flag-hygiene-phase-pass"),
                timestamp_writes: make_timestamp_writes(18),
            });
            dispatch(
                &mut pass,
                &self.material_flag_hygiene_pipeline,
                &self.material_flag_hygiene_bind_group,
            );
        }
        // Pass 19: paired phase Environment reconcile.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-expansion-environment-reconcile-pass"),
                timestamp_writes: make_timestamp_writes(19),
            });
            dispatch(
                &mut pass,
                &self.expansion_environment_reconcile_pipeline,
                &self.environment_spawn_reconcile_bind_group,
            );
        }

        // Expansion copies
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
        encoder.copy_buffer_to_buffer(
            &self.world.phase_energy_next,
            0,
            &self.world.phase_energy_current,
            0,
            self.world.layout.temperature_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.pressure_next,
            0,
            &self.world.pressure_current,
            0,
            self.world.layout.pressure_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.air_mass_next,
            0,
            &self.world.air_mass_current,
            0,
            self.world.layout.material_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.air_energy_next,
            0,
            &self.world.air_energy_current,
            0,
            self.world.layout.material_bytes,
        );

        // Pass 20: decay
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g4d-decay-pass"),
                timestamp_writes: make_timestamp_writes(20),
            });
            dispatch(&mut pass, &self.decay_pipeline, &self.decay_bind_group);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-material-flag-hygiene-decay-pass"),
                timestamp_writes: make_timestamp_writes(21),
            });
            dispatch(
                &mut pass,
                &self.material_flag_hygiene_pipeline,
                &self.material_flag_hygiene_bind_group,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te3-phase-energy-hygiene-decay-pass"),
                timestamp_writes: make_timestamp_writes(22),
            });
            dispatch(
                &mut pass,
                &self.phase_energy_hygiene_pipeline,
                &self.phase_energy_hygiene_bind_group,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-environment-reconcile-decay-pass"),
                timestamp_writes: make_timestamp_writes(23),
            });
            dispatch(
                &mut pass,
                &self.identity_environment_reconcile_pipeline,
                &self.identity_environment_reconcile_bind_group,
            );
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
        encoder.copy_buffer_to_buffer(
            &self.world.phase_energy_next,
            0,
            &self.world.phase_energy_current,
            0,
            self.world.layout.temperature_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.air_mass_next,
            0,
            &self.world.air_mass_current,
            0,
            self.world.layout.material_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.air_energy_next,
            0,
            &self.world.air_energy_current,
            0,
            self.world.layout.material_bytes,
        );

        // Pass 24: ignition context from the settled combustion-stage snapshot.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te4i-ignition-exposure-propose-pass"),
                timestamp_writes: make_timestamp_writes(24),
            });
            dispatch(
                &mut pass,
                &self.ignition_exposure_propose_pipeline,
                &self.ignition_exposure_propose_bind_group,
            );
        }
        // Pass 25: combustion consumes and fully overwrites the context.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g4c-combustion-pass"),
                timestamp_writes: make_timestamp_writes(25),
            });
            dispatch(
                &mut pass,
                &self.combustion_pipeline,
                &self.combustion_bind_group,
            );
        }
        // Pass 26: smoke_claim
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g4c-smoke-claim-pass"),
                timestamp_writes: make_timestamp_writes(26),
            });
            dispatch(
                &mut pass,
                &self.smoke_claim_pipeline,
                &self.smoke_claim_bind_group,
            );
        }
        // Pass 27: Smoke Environment receiver claim.
        encoder.clear_buffer(&self.world.environment_receiver_claim, 0, None);
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-smoke-environment-receiver-claim-pass"),
                timestamp_writes: make_timestamp_writes(27),
            });
            dispatch(
                &mut pass,
                &self.smoke_environment_receiver_claim_pipeline,
                &self.environment_receiver_claim_bind_group,
            );
        }
        // Pass 28: smoke_commit (receiver gated)
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g4c-smoke-commit-pass"),
                timestamp_writes: make_timestamp_writes(28),
            });
            dispatch(
                &mut pass,
                &self.smoke_commit_pipeline,
                &self.smoke_commit_bind_group,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-material-flag-hygiene-combustion-pass"),
                timestamp_writes: make_timestamp_writes(29),
            });
            dispatch(
                &mut pass,
                &self.material_flag_hygiene_pipeline,
                &self.material_flag_hygiene_bind_group,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te3-phase-energy-hygiene-combustion-pass"),
                timestamp_writes: make_timestamp_writes(30),
            });
            dispatch(
                &mut pass,
                &self.phase_energy_hygiene_pipeline,
                &self.phase_energy_hygiene_bind_group,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-environment-reconcile-smoke-pass"),
                timestamp_writes: make_timestamp_writes(31),
            });
            dispatch(
                &mut pass,
                &self.smoke_environment_reconcile_pipeline,
                &self.environment_spawn_reconcile_bind_group,
            );
        }

        // Combustion copies
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
        encoder.copy_buffer_to_buffer(
            &self.world.phase_energy_next,
            0,
            &self.world.phase_energy_current,
            0,
            self.world.layout.temperature_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.air_mass_next,
            0,
            &self.world.air_mass_current,
            0,
            self.world.layout.material_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.air_energy_next,
            0,
            &self.world.air_energy_current,
            0,
            self.world.layout.material_bytes,
        );

        // Pass 32: pressure
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g5a-pressure-pass"),
                timestamp_writes: make_timestamp_writes(32),
            });
            dispatch(
                &mut pass,
                &self.pressure_pipeline,
                &self.pressure_bind_group,
            );
        }
        encoder.copy_buffer_to_buffer(
            &self.world.pressure_next,
            0,
            &self.world.pressure_current,
            0,
            self.world.layout.pressure_bytes,
        );

        // Pass 33: rupture
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g5c-rupture-pass"),
                timestamp_writes: make_timestamp_writes(33),
            });
            dispatch(&mut pass, &self.rupture_pipeline, &self.rupture_bind_group);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-material-flag-hygiene-rupture-pass"),
                timestamp_writes: make_timestamp_writes(34),
            });
            dispatch(
                &mut pass,
                &self.material_flag_hygiene_pipeline,
                &self.material_flag_hygiene_bind_group,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te3-phase-energy-hygiene-rupture-pass"),
                timestamp_writes: make_timestamp_writes(35),
            });
            dispatch(
                &mut pass,
                &self.phase_energy_hygiene_pipeline,
                &self.phase_energy_hygiene_bind_group,
            );
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te1-environment-reconcile-rupture-pass"),
                timestamp_writes: make_timestamp_writes(36),
            });
            dispatch(
                &mut pass,
                &self.identity_environment_reconcile_pipeline,
                &self.identity_environment_reconcile_bind_group,
            );
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
        encoder.copy_buffer_to_buffer(
            &self.world.phase_energy_next,
            0,
            &self.world.phase_energy_current,
            0,
            self.world.layout.temperature_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.air_mass_next,
            0,
            &self.world.air_mass_current,
            0,
            self.world.layout.material_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.world.air_energy_next,
            0,
            &self.world.air_energy_current,
            0,
            self.world.layout.material_bytes,
        );

        // Pass 37: base activity_propose
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g7a-activity-propose-pass"),
                timestamp_writes: make_timestamp_writes(37),
            });
            dispatch(
                &mut pass,
                &self.activity_propose_pipeline,
                &self.activity_propose_bind_group,
            );
        }
        // Pass 38: phase activity augments the same per-cell flags.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te3-phase-activity-propose-pass"),
                timestamp_writes: make_timestamp_writes(38),
            });
            dispatch(
                &mut pass,
                &self.phase_activity_propose_pipeline,
                &self.phase_activity_propose_bind_group,
            );
        }
        // Pass 39: Environment activity augments the same per-cell flags.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te2-environment-activity-propose-pass"),
                timestamp_writes: make_timestamp_writes(39),
            });
            dispatch(
                &mut pass,
                &self.environment_activity_propose_pipeline,
                &self.environment_activity_propose_bind_group,
            );
        }
        // Pass 40: ignition activity augments the same flags over the full world.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-te4i-ignition-activity-propose-pass"),
                timestamp_writes: make_timestamp_writes(40),
            });
            dispatch(
                &mut pass,
                &self.ignition_activity_propose_pipeline,
                &self.ignition_activity_propose_bind_group,
            );
        }
        // Pass 41: activity_reduce
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g7a-activity-reduce-pass"),
                timestamp_writes: make_timestamp_writes(41),
            });
            pass.set_pipeline(&self.activity_reduce_pipeline);
            pass.set_bind_group(0, &self.activity_reduce_bind_group, &[]);
            let a_chunks_x = chunks_x(self.world.config.width, self.world.config.chunk_size);
            let a_chunks_y = chunks_y(self.world.config.height, self.world.config.chunk_size);
            pass.dispatch_workgroups(a_chunks_x, a_chunks_y, 1);
        }

        // If profiling, resolve timestamps to resolve buffer and copy to readback staging buffer.
        if let Some(prof) = profiler {
            encoder.resolve_query_set(&prof.query_set, 0..QUERY_COUNT, &prof.resolve_buffer, 0);
            encoder.copy_buffer_to_buffer(
                &prof.resolve_buffer,
                0,
                &prof.readback_buffer,
                0,
                (QUERY_COUNT as u64) * 8,
            );
        }

        self.context.queue.submit([encoder.finish()]);
        self.tick_count += 1;
        Ok(())
    }

    /// Performs an out-of-band non-timed activity census on the world.
    /// Reads `cell_activity`, `chunk_activity`, and `chunk_state` from the GPU.
    ///
    /// Note: this must NEVER be called inside timed simulation loops.
    pub fn activity_census(&self) -> Result<ActivityCensusReport, GpuError> {
        self.activity_census_snapshot()
            .map(|snapshot| snapshot.report)
    }

    /// Performs an out-of-band non-timed activity census and preserves the
    /// exact raw GPU readbacks used to produce its report.
    ///
    /// Note: this must NEVER be called inside timed simulation loops.
    pub fn activity_census_snapshot(&self) -> Result<ActivityCensusSnapshot, GpuError> {
        let dev = &self.context.device;
        let q = &self.context.queue;

        let cell_activity = self.world.read_cell_activity_all(dev, q)?;
        let chunk_activity = self.world.read_chunk_activity_all(dev, q)?;
        let chunk_state = self.world.read_chunk_state_all(dev, q)?;

        let expected_cells = usize::try_from(self.world.layout.cell_count).map_err(|_| {
            GpuError::ReadbackFailed(format!(
                "activity census cell count {} does not fit usize",
                self.world.layout.cell_count
            ))
        })?;
        let expected_chunks = chunk_count(
            self.world.config.width,
            self.world.config.height,
            self.world.config.chunk_size,
        ) as usize;

        if cell_activity.len() != expected_cells
            || chunk_activity.len() != expected_chunks
            || chunk_state.len() != expected_chunks
        {
            return Err(GpuError::ReadbackFailed(format!(
                "activity census length mismatch: cell_activity={} expected={}, chunk_activity={} expected={}, chunk_state={} expected={}",
                cell_activity.len(),
                expected_cells,
                chunk_activity.len(),
                expected_chunks,
                chunk_state.len(),
                expected_chunks,
            )));
        }

        let report = count_activity_census(&cell_activity, &chunk_activity, &chunk_state);

        Ok(ActivityCensusSnapshot {
            report,
            cell_activity,
            chunk_activity,
            chunk_state,
        })
    }

    /// Computes the exact persistent application-tracked GPU buffer bytes.
    ///
    /// This is an allocation inventory, not driver-resident VRAM. Transient
    /// readback buffers created by diagnostic helpers are intentionally excluded.
    /// When supplied, `profiler` contributes its persistent resolve and mapped
    /// readback buffers; opaque driver/query-set storage is not reported.
    pub fn tracked_memory_report(&self, profiler: Option<&GpuProfiler>) -> TrackedMemoryReport {
        let world_dense_state_bytes = self.world.layout.material_bytes * 2
            + self.world.layout.temperature_bytes * 2
            + self.world.layout.temperature_bytes * 2
            + self.world.layout.pressure_bytes * 2
            + self.world.layout.flags_bytes * 2;
        let environment_state_bytes = self.world.layout.material_bytes * 4;
        let movement_scratch_bytes = self.world.layout.material_bytes * 2;
        let environment_receiver_claim_bytes = self.world.layout.material_bytes;
        let chunk_bytes = chunk_count(
            self.world.config.width,
            self.world.config.height,
            self.world.config.chunk_size,
        ) as u64
            * 4;
        let activity_scratch_bytes = self.world.layout.material_bytes + chunk_bytes * 6;
        let uniforms_and_tables_bytes = TRACKED_UNIFORMS_AND_TABLES_SIZE;
        let profiler_bytes = profiler.map_or(0, |p| p.tracked_gpu_allocation_bytes());
        let total_tracked_gpu_bytes = world_dense_state_bytes
            + environment_state_bytes
            + movement_scratch_bytes
            + environment_receiver_claim_bytes
            + activity_scratch_bytes
            + uniforms_and_tables_bytes
            + profiler_bytes;

        TrackedMemoryReport {
            world_dense_state_bytes,
            environment_state_bytes,
            movement_scratch_bytes,
            environment_receiver_claim_bytes,
            activity_scratch_bytes,
            uniforms_and_tables_bytes,
            profiler_bytes,
            total_tracked_gpu_bytes,
        }
    }

    /// Waits for GPU work and reads the diagnostic marker.
    ///
    /// The marker is set to `1` by the propose pass (single invocation),
    /// proving that the dispatch actually executed on the GPU.
    pub fn read_marker(&self) -> Result<u32, GpuError> {
        let staging = self.context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g3/movement/marker-staging"),
            size: MARKER_SIZE,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("powdergame-g3-readback-encoder"),
                });
        encoder.copy_buffer_to_buffer(&self.marker, 0, &staging, 0, MARKER_SIZE);
        self.context.queue.submit([encoder.finish()]);

        let _ = self.context.device.poll(wgpu::PollType::Wait);

        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        let _ = self.context.device.poll(wgpu::PollType::Wait);

        rx.recv()
            .map_err(|e| GpuError::ReadbackFailed(format!("map callback lost: {e}")))?
            .map_err(|e| GpuError::ReadbackFailed(e.to_string()))?;

        let mapped = slice.get_mapped_range();
        let value = u32::from_ne_bytes(mapped[..4].try_into().unwrap());
        drop(mapped);
        staging.unmap();
        Ok(value)
    }
}

/// Detailed non-timed activity census of the active world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityCensusReport {
    pub total_cells: u64,
    pub any_active_cells: u64,
    pub matter_active_cells: u64,
    pub thermal_active_cells: u64,
    pub pressure_active_cells: u64,
    pub reaction_active_cells: u64,
    pub total_chunks: u32,
    pub active_chunks: u32,
    pub runnable_chunks: u32,
    pub sleeping_chunks: u32,
}

/// Exact raw GPU readbacks and the activity report derived from them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityCensusSnapshot {
    pub report: ActivityCensusReport,
    pub cell_activity: Vec<u32>,
    pub chunk_activity: Vec<u32>,
    pub chunk_state: Vec<u32>,
}

fn count_activity_census(
    cell_activity: &[u32],
    chunk_activity: &[u32],
    chunk_state: &[u32],
) -> ActivityCensusReport {
    debug_assert_eq!(chunk_activity.len(), chunk_state.len());

    let mut any_active_cells = 0u64;
    let mut matter_active_cells = 0u64;
    let mut thermal_active_cells = 0u64;
    let mut pressure_active_cells = 0u64;
    let mut reaction_active_cells = 0u64;

    for &activity in cell_activity {
        if activity != 0 {
            any_active_cells += 1;
        }
        if (activity & ACTIVITY_MATTER) != 0 {
            matter_active_cells += 1;
        }
        if (activity & ACTIVITY_THERMAL) != 0 {
            thermal_active_cells += 1;
        }
        if (activity & ACTIVITY_PRESSURE) != 0 {
            pressure_active_cells += 1;
        }
        if (activity & ACTIVITY_REACTION) != 0 {
            reaction_active_cells += 1;
        }
    }

    let active_chunks = chunk_activity
        .iter()
        .filter(|&&activity| activity != 0)
        .count() as u32;
    let runnable_chunks = chunk_state
        .iter()
        .filter(|&&state| state == CHUNK_STATE_RUNNABLE)
        .count() as u32;
    let sleeping_chunks = chunk_state
        .iter()
        .filter(|&&state| state == CHUNK_STATE_SLEEPING)
        .count() as u32;

    ActivityCensusReport {
        total_cells: cell_activity.len() as u64,
        any_active_cells,
        matter_active_cells,
        thermal_active_cells,
        pressure_active_cells,
        reaction_active_cells,
        total_chunks: chunk_activity.len() as u32,
        active_chunks,
        runnable_chunks,
        sleeping_chunks,
    }
}

/// Detailed persistent application-tracked GPU buffer allocation report.
///
/// These requested buffer sizes are not a measurement of driver-resident VRAM.
/// Diagnostic readback helpers allocate short-lived staging buffers that are
/// intentionally outside this persistent inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedMemoryReport {
    /// 8 logical dense world state buffers: material, temperature, pressure, flags (current + next each).
    pub world_dense_state_bytes: u64,
    /// TE-1 persistent Environment state: Air mass/energy (current + next each).
    pub environment_state_bytes: u64,
    /// Movement arbitration scratch buffers: proposal + claim.
    pub movement_scratch_bytes: u64,
    /// TE-1 reusable phase/Smoke Environment receiver arbitration scratch.
    pub environment_receiver_claim_bytes: u64,
    /// G7 activity diagnostic buffers: cell_activity + 6 chunk buffers.
    pub activity_scratch_bytes: u64,
    /// All persistent uniforms, simulation tables, and debug markers.
    pub uniforms_and_tables_bytes: u64,
    /// Persistent profiler resolve + readback buffers (0 when profiler absent).
    /// Opaque query-set/driver storage is not application-countable here.
    pub profiler_bytes: u64,
    /// Sum of all application-tracked GPU buffer allocations.
    pub total_tracked_gpu_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_census_counts_raw_bitmasks_and_chunk_states() {
        let report = count_activity_census(
            &[
                0,
                ACTIVITY_MATTER,
                ACTIVITY_THERMAL | ACTIVITY_PRESSURE,
                ACTIVITY_MATTER | ACTIVITY_REACTION,
            ],
            &[0, ACTIVITY_MATTER, ACTIVITY_THERMAL | ACTIVITY_REACTION],
            &[
                CHUNK_STATE_RUNNABLE,
                CHUNK_STATE_SLEEPING,
                CHUNK_STATE_RUNNABLE,
            ],
        );

        assert_eq!(
            report,
            ActivityCensusReport {
                total_cells: 4,
                any_active_cells: 3,
                matter_active_cells: 2,
                thermal_active_cells: 1,
                pressure_active_cells: 1,
                reaction_active_cells: 1,
                total_chunks: 3,
                active_chunks: 2,
                runnable_chunks: 2,
                sleeping_chunks: 1,
            }
        );
    }
}
