//! Headless GPU Simulation lifecycle.
//!
//! `Simulation` owns the production GPU context and dense world, and can be
//! created and ticked without any Window, Surface or Renderer
//! (`docs/architecture/ARCHITECTURE.md` §8, MILESTONES G0).
//!
//! G2 `tick()` runs the local-movement pipeline over the full world:
//! propose → resolve → commit, then a GPU-side `material_next →
//! material_current` copy. The GPU remains the authoritative simulation
//! path; the CPU only orchestrates and stages edits. No gameplay beyond
//! movement (density/thermal are later Gates).

use powdergame_core::{movement_class_table, WorldConfig};

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
/// bytes. Movement classes live in a separate storage buffer (no uniform
/// alignment concerns).
const PARAMS_SIZE: u64 = 16;
/// Movement-class table buffer size (16 u32 entries).
const CLASS_TABLE_SIZE: u64 = 64;
/// Size of the diagnostic marker buffer (one `u32` + padding).
const MARKER_SIZE: u64 = 16;

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
    resolve_pipeline: wgpu::ComputePipeline,
    commit_pipeline: wgpu::ComputePipeline,
    propose_bind_group: wgpu::BindGroup,
    resolve_bind_group: wgpu::BindGroup,
    commit_bind_group: wgpu::BindGroup,
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

        // Per-pass shader modules: each contains only its own entry point and
        // its own bindings, so the bind group layouts line up 1:1.
        let shader_propose = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g2-movement-propose"),
                source: wgpu::ShaderSource::Wgsl(
                    entry_point_source(include_str!("tick.wgsl"), "propose_main").into(),
                ),
            });
        let shader_resolve = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g2-movement-resolve"),
                source: wgpu::ShaderSource::Wgsl(
                    entry_point_source(include_str!("tick.wgsl"), "resolve_main").into(),
                ),
            });
        let shader_commit = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("powdergame-g2-movement-commit"),
                source: wgpu::ShaderSource::Wgsl(
                    entry_point_source(include_str!("tick.wgsl"), "commit_main").into(),
                ),
            });

        // Bind group layouts: uniform (0) + storage bindings.
        let propose_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g2-propose-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current
                        buffer_entry(2, &BindingKind::ReadWrite), // proposal
                        buffer_entry(3, &BindingKind::ReadWrite), // marker
                        buffer_entry(4, &BindingKind::Read), // class_table
                    ],
                });
        let resolve_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g2-resolve-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current_r
                        buffer_entry(2, &BindingKind::Read), // proposal_r
                        buffer_entry(3, &BindingKind::ReadWrite), // resolve
                        buffer_entry(4, &BindingKind::Read), // class_table_r
                    ],
                });
        let commit_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("powdergame-g2-commit-bgl"),
                    entries: &[
                        buffer_entry(0, &BindingKind::Uniform),
                        buffer_entry(1, &BindingKind::Read), // material_current_c
                        buffer_entry(2, &BindingKind::Read), // proposal_c
                        buffer_entry(3, &BindingKind::Read), // resolve_c
                        buffer_entry(4, &BindingKind::ReadWrite), // material_next
                        buffer_entry(5, &BindingKind::Read), // class_table_c
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
            "powdergame-g2-propose",
            &propose_layout,
            &shader_propose,
            "propose_main",
        );
        let resolve_pipeline = make_pipeline(
            "powdergame-g2-resolve",
            &resolve_layout,
            &shader_resolve,
            "resolve_main",
        );
        let commit_pipeline = make_pipeline(
            "powdergame-g2-commit",
            &commit_layout,
            &shader_commit,
            "commit_main",
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
            label: Some("g2/movement/params"),
            size: PARAMS_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context.queue.write_buffer(&params, 0, &params_data);

        // Movement-class table (read-only storage; EMPTY/unknown map to 0).
        let mut class_data = [0u8; CLASS_TABLE_SIZE as usize];
        for (i, class) in movement_class_table().iter().enumerate() {
            let off = i * 4;
            class_data[off..off + 4].copy_from_slice(&class.to_ne_bytes());
        }
        let class_table = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g2/movement/class-table"),
            size: CLASS_TABLE_SIZE,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context.queue.write_buffer(&class_table, 0, &class_data);

        // Diagnostic marker: 16 bytes, one u32 used.
        let marker = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("g2/movement/marker"),
            size: MARKER_SIZE,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let propose_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g2-propose-bg"),
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
                ],
            });
        let resolve_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g2-resolve-bg"),
                layout: &resolve_layout,
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
                        resource: world.resolve.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: class_table.as_entire_binding(),
                    },
                ],
            });
        let commit_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("powdergame-g2-commit-bg"),
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
                        resource: world.proposal.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: world.resolve.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: world.material_next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: class_table.as_entire_binding(),
                    },
                ],
            });

        Ok(Self {
            context,
            world,
            propose_pipeline,
            resolve_pipeline,
            commit_pipeline,
            propose_bind_group,
            resolve_bind_group,
            commit_bind_group,
            marker,
            tick_count: 0,
        })
    }

    /// Submits one G2 movement tick: propose → resolve → commit + Next→Current
    /// copy, all on the GPU. The CPU never simulates or copies the full world.
    pub fn tick(&mut self) -> Result<(), GpuError> {
        let cell_count = self.world.layout.cell_count;
        let dispatch_y = u32::try_from(cell_count.div_ceil(THREADS_X))
            .map_err(|_| GpuError::Other("dispatch height overflow".into()))?;

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("powdergame-g2-tick-encoder"),
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
                label: Some("powdergame-g2-propose-pass"),
                timestamp_writes: None,
            });
            dispatch(&mut pass, &self.propose_pipeline, &self.propose_bind_group);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g2-resolve-pass"),
                timestamp_writes: None,
            });
            dispatch(&mut pass, &self.resolve_pipeline, &self.resolve_bind_group);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("powdergame-g2-commit-pass"),
                timestamp_writes: None,
            });
            dispatch(&mut pass, &self.commit_pipeline, &self.commit_bind_group);
        }

        // Commit Next → Current on the GPU (readable baseline; the CPU does
        // not copy the full world).
        encoder.copy_buffer_to_buffer(
            &self.world.material_next,
            0,
            &self.world.material_current,
            0,
            self.world.layout.material_bytes,
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
            label: Some("g2/movement/marker-staging"),
            size: MARKER_SIZE,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("powdergame-g2-readback-encoder"),
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

/// Helper functions used only by the propose pass (they read the propose
/// bindings). They are stripped from the resolve/commit modules.
const PROPOSE_HELPERS: &[&str] = &[
    "cell_state",
    "target_index",
    "try_diagonal",
    "try_lateral",
    "propose_powder",
    "propose_liquid",
    "propose_gas",
];

/// Builds a shader module source containing only the given entry point and
/// only the bindings/helpers that entry point uses.
fn entry_point_source(source: &str, keep: &str) -> String {
    // Pass 1: drop binding declarations that the kept entry point does not use.
    let mut filtered = String::with_capacity(source.len());
    for line in source.lines() {
        let trimmed = line.trim_start();
        let is_binding_decl =
            trimmed.starts_with("@group(0) @binding(") && trimmed.contains(" var<");
        let drop = if is_binding_decl {
            let name = binding_name(trimmed);
            !keep_bindings(keep).contains(&name)
        } else {
            false
        };
        if !drop {
            filtered.push_str(line);
            filtered.push('\n');
        }
    }

    // Pass 2: strip the propose-only helpers from non-propose modules.
    let filtered = if keep == "propose_main" {
        filtered
    } else {
        strip_fns(&filtered, PROPOSE_HELPERS)
    };

    // Pass 3: strip the other compute entry points (with their attributes).
    strip_other_entry_points(&filtered, keep)
}

/// Bindings kept per pass (names declared in `tick.wgsl`).
fn keep_bindings(keep: &str) -> &'static [&'static str] {
    match keep {
        "propose_main" => &[
            "params",
            "material_current",
            "proposal",
            "marker",
            "class_table",
        ],
        "resolve_main" => &[
            "params_r",
            "material_current_r",
            "proposal_r",
            "resolve",
            "class_table_r",
        ],
        "commit_main" => &[
            "params_c",
            "material_current_c",
            "proposal_c",
            "resolve_c",
            "material_next",
            "class_table_c",
        ],
        _ => &[],
    }
}

/// Extracts the variable name from a binding declaration line like
/// `@group(0) @binding(4) var<storage, read> class_table: array<u32, 16>;`.
fn binding_name(line: &str) -> &str {
    let after_gt = line.find('>').map(|p| p + 1).unwrap_or(0);
    let rest = &line[after_gt..];
    let name_end = rest.find(':').unwrap_or(rest.len());
    rest[..name_end].trim()
}

/// Removes every top-level function in `remove` from `source`, preserving all
/// other text. Assumes no nested function definitions and no `fn ` tokens in
/// comments/strings (true for `tick.wgsl`).
fn strip_fns(source: &str, remove: &[&str]) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(pos) = rest.find("fn ") {
        out.push_str(&rest[..pos]);
        let after = &rest[pos..];
        let ident = &after[3..];
        let name_len = ident
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(ident.len());
        let name = &ident[..name_len];
        let open = after.find('{').unwrap_or(after.len());
        let body = &after[open + 1..];
        let depth = brace_span(body);
        let end = open + 1 + depth;
        if !remove.contains(&name) {
            out.push_str(&after[..end]);
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// Removes every `@compute fn` except `keep` from `source`, preserving the
/// struct/const/helper declarations.
fn strip_other_entry_points(source: &str, keep: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(pos) = rest.find("@compute") {
        out.push_str(&rest[..pos]);
        let after = &rest[pos..];
        // Find the `fn <name>` of this entry point.
        let fn_start = after.find("fn ").map(|p| p + 3).unwrap_or(after.len());
        let name_end = after[fn_start..]
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .map(|p| fn_start + p)
            .unwrap_or(after.len());
        let name = &after[fn_start..name_end];
        // Find the balanced-brace span of the function body.
        let open = after.find('{').unwrap_or(after.len());
        let body = &after[open + 1..];
        let depth = brace_span(body);
        let end = open + 1 + depth;
        let keep_fn = name == keep;
        if !keep_fn {
            // Skip this entry point entirely (attributes included).
            rest = &after[end..];
            continue;
        }
        out.push_str(&after[..end]);
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// Returns the byte span of `body` covered by the block that opened right
/// before it: one past the matching close brace. The opening brace already
/// contributes depth 1, so a `}` is only terminal once depth returns to 0.
fn brace_span(body: &str) -> usize {
    let mut depth = 1i32;
    for (i, c) in body.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return i + 1; // one past the matching close brace
                }
            }
            _ => {}
        }
    }
    body.len()
}
