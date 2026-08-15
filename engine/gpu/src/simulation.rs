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
//! not a thermal medium. Phase / combustion are not implemented.

use powdergame_core::{
    conductivity_table, density_table, heat_capacity_table, movement_class_table, WorldConfig,
};

use crate::context::{GpuContext, GpuError};
use crate::world::GpuWorld;

/// Workgroup size of the movement shaders.
const WORKGROUP_SIZE: u32 = 64;
/// Workgroups along X per dispatch row. Kept well below the DX12 limit of
/// 65535 while still covering the reference world in two rows.
const WORKGROUPS_X: u32 = 256;
/// Total threads per dispatch row (`WORKGROUPS_X * WORKGROUP_SIZE`).
const THREADS_X: u64 = (WORKGROUPS_X as u64) * (WORKGROUP_SIZE as u64);
/// Params uniform size: cell_count, threads_x, width, height (4 u32) = 16
/// bytes. Material tables live in separate storage buffers (no uniform
/// alignment concerns).
const PARAMS_SIZE: u64 = 16;
/// Material table buffer size (16 u32 entries each).
const TABLE_SIZE: u64 = 64;
/// Size of the diagnostic marker buffer (one `u32` + padding).
const MARKER_SIZE: u64 = 16;
/// Upper bound for cell indices so the claim encoding `(peer << 2) | kind`
/// can never overflow or collide with sentinels.
const MAX_CELL_COUNT: u64 = 1 << 30;

/// Buffer binding kind used to build the movement bind group layouts.
enum BindingKind {
    Uniform,
    Read,
    ReadWrite,
}

/// Builds a compute-stage buffer binding entry.
fn buffer_entry(binding: u32, kind: &BindingKind) -> wgpu::BindGroupLayoutEntry {
    let ty = match kind {
        BindingKind::Uniform => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(PARAMS_SIZE),
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
    thermal_pipeline: wgpu::ComputePipeline,
    propose_bind_group: wgpu::BindGroup,
    claim_bind_group: wgpu::BindGroup,
    commit_bind_group: wgpu::BindGroup,
    thermal_bind_group: wgpu::BindGroup,
    marker: wgpu::Buffer,

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
        let shader_thermal = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g4a-thermal"),
                source: wgpu::ShaderSource::Wgsl(include_str!("thermal.wgsl").into()),
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
                    ],
                });
        let thermal_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g4a-thermal-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::Read), // temperature_current
                        buffer_entry(3, &BindingKind::ReadWrite), // temperature_next
                        buffer_entry(4, &BindingKind::Read), // conductivity_table
                        buffer_entry(5, &BindingKind::Read), // capacity_table
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
        let thermal_pipeline = make_pipeline(
            "powdergame-g4a-thermal",
            &thermal_layout,
            &shader_thermal,
            "thermal_main",
        );

        // Params uniform: cell_count, threads_x, width, height.
        let cell_count_u32 = u32::try_from(world.layout.cell_count).map_err(|_| {
            GpuError::Other(format!(
                "world cell count {} does not fit in u32 for dispatch",
                world.layout.cell_count
            ))
        })?;
        let threads_x_u32 =
            u32::try_from(THREADS_X).map_err(|_| GpuError::Other("threads_x overflow".into()))?;
        let mut params_data = [0u8; PARAMS_SIZE as usize];
        params_data[..4].copy_from_slice(&cell_count_u32.to_ne_bytes());
        params_data[4..8].copy_from_slice(&threads_x_u32.to_ne_bytes());
        params_data[8..12].copy_from_slice(&world.config.width.to_ne_bytes());
        params_data[12..16].copy_from_slice(&world.config.height.to_ne_bytes());

        let params = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g3/movement/params"),
            size: PARAMS_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context.queue.write_buffer(&params, 0, &params_data);

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

        // Diagnostic marker: 16 bytes, one u32 used.
        let marker = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g3/movement/marker"),
            size: MARKER_SIZE,
            usage: wgpu::BufferUsages::STORAGE
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
                ],
            });
        let thermal_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g4a-thermal-bg"),
                layout: &thermal_layout,
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
                        resource: world.temperature_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: conductivity_table_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: capacity_table_buf.as_entire_binding(),
                    },
                ],
            });

        Ok(Self {
            context,
            world,
            propose_pipeline,
            claim_pipeline,
            commit_pipeline,
            thermal_pipeline,
            propose_bind_group,
            claim_bind_group,
            commit_bind_group,
            thermal_bind_group,
            marker,
            tick_count: 0,
        })
    }

    /// Submits one tick on the GPU (no CPU full-world copy):
    /// propose → claim → commit (material_next + temperature_next) →
    /// material Next→Current → temperature Next→Current →
    /// thermal conduction (write-self) → temperature Next→Current.
    pub fn tick(&mut self) -> Result<(), GpuError> {
        let cell_count = self.world.layout.cell_count;
        let dispatch_y = u32::try_from(cell_count.div_ceil(THREADS_X))
            .map_err(|_| GpuError::Other("dispatch height overflow".into()))?;

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("powdergame-g3-tick-encoder"),
                });

        let dispatch = |pass: &mut wgpu::ComputePass<'_>,
                        pipeline: &wgpu::ComputePipeline,
                        bg: &wgpu::BindGroup| {
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bg, &[]);
            pass.dispatch_workgroups(WORKGROUPS_X, dispatch_y, 1);
        };

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g3-propose-pass"),
                timestamp_writes: None,
            });
            dispatch(&mut pass, &self.propose_pipeline, &self.propose_bind_group);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g3-claim-pass"),
                timestamp_writes: None,
            });
            dispatch(&mut pass, &self.claim_pipeline, &self.claim_bind_group);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g3-commit-pass"),
                timestamp_writes: None,
            });
            dispatch(&mut pass, &self.commit_pipeline, &self.commit_bind_group);
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

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g4a-thermal-pass"),
                timestamp_writes: None,
            });
            dispatch(&mut pass, &self.thermal_pipeline, &self.thermal_bind_group);
        }
        encoder.copy_buffer_to_buffer(
            &self.world.temperature_next,
            0,
            &self.world.temperature_current,
            0,
            self.world.layout.temperature_bytes,
        );

        self.context.queue.submit([encoder.finish()]);
        self.tick_count += 1;
        Ok(())
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
