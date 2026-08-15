//! Headless GPU Simulation lifecycle.
//!
//! `Simulation` owns the production GPU context and dense world, and can be
//! created and ticked without any Window, Surface or Renderer
//! (`docs/architecture/ARCHITECTURE.md` §8, MILESTONES G0).
//!
//! G0 `tick()` executes a minimal compute dispatch over the full world as
//! runtime plumbing to prove the lifecycle: it copies `material_id` from the
//! Current half to the Next half (no gameplay rule) and sets a diagnostic
//! marker so a caller can verify the dispatch actually executed on the GPU.

use powdergame_core::WorldConfig;

use crate::context::{GpuContext, GpuError};
use crate::world::GpuWorld;

/// Workgroup size of the G0 tick shader.
const WORKGROUP_SIZE: u32 = 64;
/// Workgroups along X per dispatch row. Kept well below the DX12 limit of
/// 65535 while still covering the reference world in two rows.
const WORKGROUPS_X: u32 = 256;
/// Total threads per dispatch row (`WORKGROUPS_X * WORKGROUP_SIZE`).
const THREADS_X: u64 = (WORKGROUPS_X as u64) * (WORKGROUP_SIZE as u64);
/// Size of the params uniform (two `u32` + padding, 16 bytes).
const PARAMS_SIZE: u64 = 16;
/// Size of the diagnostic marker buffer (one `u32` + padding).
const MARKER_SIZE: u64 = 16;

/// Headless GPU simulation: context + dense world + tick pipeline.
pub struct Simulation {
    pub context: GpuContext,
    pub world: GpuWorld,

    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    marker: wgpu::Buffer,

    /// Number of ticks submitted since creation.
    pub tick_count: u64,
}

impl Simulation {
    /// Creates a new DX12 context, allocates the dense world and builds the
    /// G0 tick pipeline. Headless: no window required.
    pub async fn new(config: WorldConfig) -> Result<Self, GpuError> {
        let context = GpuContext::new().await?;
        Self::with_context(context, config)
    }

    /// Builds a simulation on an existing GPU context.
    pub fn with_context(context: GpuContext, config: WorldConfig) -> Result<Self, GpuError> {
        let world = GpuWorld::new(&context.device, config)?;

        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g0-tick"),
                source: wgpu::ShaderSource::Wgsl(include_str!("tick.wgsl").into()),
            });

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g0-tick-bgl"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(PARAMS_SIZE),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("powdergame-g0-tick-pl"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("powdergame-g0-tick-pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        // Params uniform: cell_count, threads_x, padding. Lives on via the
        // bind group; no need for a struct field.
        let cell_count_u32 = u32::try_from(world.layout.cell_count).map_err(|_| {
            GpuError::Other(format!(
                "world cell count {} does not fit in u32 for dispatch",
                world.layout.cell_count
            ))
        })?;
        let threads_x_u32 = u32::try_from(THREADS_X)
            .map_err(|_| GpuError::Other("threads_x does not fit in u32".into()))?;
        let mut params_data = [0u8; PARAMS_SIZE as usize];
        params_data[..4].copy_from_slice(&cell_count_u32.to_ne_bytes());
        params_data[4..8].copy_from_slice(&threads_x_u32.to_ne_bytes());

        let params = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g0/tick/params"),
            size: PARAMS_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context.queue.write_buffer(&params, 0, &params_data);

        // Diagnostic marker: 16 bytes, one u32 used.
        let marker = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g0/tick/marker"),
            size: MARKER_SIZE,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g0-tick-bg"),
                layout: &bind_group_layout,
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
                        resource: marker.as_entire_binding(),
                    },
                ],
            });

        Ok(Self {
            context,
            world,
            pipeline,
            bind_group,
            marker,
            tick_count: 0,
        })
    }

    /// Submits one G0 tick: a full-world compute dispatch.
    ///
    /// G0 executes no gameplay rule. This proves the GPU context, world
    /// buffers and dispatch plumbing all work without a window.
    pub fn tick(&mut self) -> Result<(), GpuError> {
        let cell_count = self.world.layout.cell_count;
        // 2D grid: `WORKGROUPS_X` groups along X, rows along Y. The tail row
        // is guarded by the bounds check in the shader.
        let dispatch_y = u32::try_from(cell_count.div_ceil(THREADS_X))
            .map_err(|_| GpuError::Other("dispatch height overflow".into()))?;

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("powdergame-g0-tick-encoder"),
                });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g0-tick-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(WORKGROUPS_X, dispatch_y, 1);
        }

        self.context.queue.submit([encoder.finish()]);
        self.tick_count += 1;
        Ok(())
    }

    /// Waits for GPU work and reads the diagnostic marker.
    ///
    /// The marker is set to `1` by the G0 tick shader, proving that the
    /// dispatch actually executed on the GPU (not merely queued).
    pub fn read_marker(&self) -> Result<u32, GpuError> {
        let staging = self.context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g0/tick/marker-staging"),
            size: MARKER_SIZE,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("powdergame-g0-readback-encoder"),
                });
        encoder.copy_buffer_to_buffer(&self.marker, 0, &staging, 0, MARKER_SIZE);
        self.context.queue.submit([encoder.finish()]);

        // Wait for the copy to finish before mapping.
        let _ = self.context.device.poll(wgpu::PollType::Wait);

        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        // Wait for the map to complete (callback is driven by poll).
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
