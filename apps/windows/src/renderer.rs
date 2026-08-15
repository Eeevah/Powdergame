//! Minimal presentation layer for the Windows app.
//!
//! The Renderer owns the surface + frame path only. It is NOT the
//! authoritative owner of simulation state
//! (`docs/architecture/ARCHITECTURE.md` §15, MILESTONES G0).
//!
//! G2 adds an optional read-only world view: the `material_current` GPU
//! buffer is bound to the fragment shader as read-only storage and drawn
//! through a fullscreen triangle with per-material debug colors.
//! Presentation never mutates the authoritative simulation state.
//!
//! The world view preserves square cells: it letterboxes the world into the
//! surface with `scale = min(surface_w / world_w, surface_h / world_h)` and
//! maps pixels to cells with integer truncation, so cell edges stay crisp
//! and the world aspect ratio is never distorted.

use std::sync::Arc;

use wgpu::TextureFormat;

use powdergame_gpu::GpuError;
use winit::window::Window;

/// Read-only view spec for presenting the material world (G2).
pub struct WorldViewSpec<'a> {
    pub material_buffer: &'a wgpu::Buffer,
    pub width: u32,
    pub height: u32,
}

/// Window surface renderer: acquire → (world view or clear) → present.
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    world_view: Option<WorldView>,
}

struct WorldView {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    params: wgpu::Buffer,
    world_width: u32,
    world_height: u32,
}

/// Clear color for the empty G0 world frame (a dim slate blue).
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.02,
    g: 0.02,
    b: 0.05,
    a: 1.0,
};

/// Params uniform: world width/height + surface width/height (4 u32 = 16 B).
const WORLD_VIEW_PARAMS_SIZE: u64 = 16;

const WORLD_VIEW_SHADER: &str = r#"
struct Params {
    width: u32,
    height: u32,
    surface_w: u32,
    surface_h: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> materials: array<u32>;

const EMPTY: u32 = 0u;
const BOUNDARY: u32 = 1u;
const STONE: u32 = 2u;
const SAND: u32 = 3u;
const WATER: u32 = 4u;
const OIL: u32 = 5u;
const STEAM: u32 = 6u;
const SMOKE: u32 = 7u;

// Presentation-only debug palette (material IDs never change). Stone reads
// as forest-green terrain/trees so the demo world looks like a stylized
// virtual forest; everything else stays clearly distinguishable.
fn debug_color(id: u32) -> vec4<f32> {
    if (id == EMPTY) { return vec4<f32>(0.03, 0.03, 0.06, 1.0); }
    if (id == BOUNDARY) { return vec4<f32>(0.55, 0.57, 0.60, 1.0); }
    if (id == STONE) { return vec4<f32>(0.28, 0.42, 0.24, 1.0); }
    if (id == SAND) { return vec4<f32>(0.88, 0.75, 0.38, 1.0); }
    if (id == WATER) { return vec4<f32>(0.15, 0.42, 0.85, 1.0); }
    if (id == OIL) { return vec4<f32>(0.48, 0.27, 0.07, 1.0); }
    if (id == STEAM) { return vec4<f32>(0.85, 0.88, 0.92, 1.0); }
    if (id == SMOKE) { return vec4<f32>(0.32, 0.32, 0.34, 1.0); }
    return vec4<f32>(1.0, 0.0, 1.0, 1.0); // unknown → magenta (must never appear)
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(pos[vi], 0.0, 1.0);
}

// Square-cell, aspect-preserving view: the world is letterboxed into the
// surface at scale = min(surface/world) and each pixel maps to exactly one
// cell via integer truncation (crisp cell edges, no stretching).
@fragment
fn fs_main(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let fw = f32(params.surface_w);
    let fh = f32(params.surface_h);
    let ww = f32(params.width);
    let wh = f32(params.height);
    let scale = min(fw / ww, fh / wh);
    let off_x = (fw - ww * scale) * 0.5;
    let off_y = (fh - wh * scale) * 0.5;
    let px = frag.x;
    let py = frag.y;
    let in_viewport = px >= off_x && px < off_x + ww * scale
                   && py >= off_y && py < off_y + wh * scale;
    if (!in_viewport) {
        return vec4<f32>(0.06, 0.07, 0.10, 1.0); // letterbox background
    }
    let cell_x = min(u32((px - off_x) / scale), params.width - 1u);
    let cell_y = min(u32((py - off_y) / scale), params.height - 1u);
    let idx = cell_y * params.width + cell_x;
    return debug_color(materials[idx]);
}
"#;

impl Renderer {
    /// Creates a surface for `window` on the given instance/adapter/device.
    ///
    /// Pass `world_view` to present the material world with debug colors
    /// (read-only). With `None` the frame is a plain clear (G0 smoke mode).
    pub fn new(
        instance: &wgpu::Instance,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: Arc<Window>,
        world_view: Option<WorldViewSpec<'_>>,
    ) -> Result<Self, GpuError> {
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| GpuError::SurfaceCreateFailed(e.to_string()))?;

        let capabilities = surface.get_capabilities(adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(device, &config);

        let world_view =
            world_view.map(|spec| build_world_view(device, queue, format, &config, spec));

        Ok(Self {
            surface,
            config,
            device: device.clone(),
            queue: queue.clone(),
            world_view,
        })
    }

    /// Reconfigures the surface after a window resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
        if let Some(wv) = &self.world_view {
            write_world_view_params(&self.queue, wv, &self.config);
        }
    }

    /// Acquires a frame, draws it (world view or clear), and presents it.
    pub fn render(&mut self) -> Result<(), GpuError> {
        let frame = self
            .surface
            .get_current_texture()
            .map_err(|e| GpuError::SurfaceFrameAcquireFailed(e.to_string()))?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("powdergame-render-encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("powdergame-present-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if let Some(wv) = &self.world_view {
                render_pass.set_pipeline(&wv.pipeline);
                render_pass.set_bind_group(0, &wv.bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }
            // In wgpu 26 the render pass ends implicitly when dropped.
            drop(render_pass);
        }

        self.queue.submit([encoder.finish()]);
        frame.present();
        Ok(())
    }

    /// The surface format in use (useful for diagnostics).
    pub fn format(&self) -> TextureFormat {
        self.config.format
    }
}

/// Builds the read-only world-view pipeline + bind group.
///
/// The bind group holds the params uniform and the world's material buffer;
/// the material buffer is bound as read-only storage, so presentation can
/// never mutate the authoritative simulation state.
fn build_world_view(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    config: &wgpu::SurfaceConfiguration,
    spec: WorldViewSpec<'_>,
) -> WorldView {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("powdergame-world-view-shader"),
        source: wgpu::ShaderSource::Wgsl(WORLD_VIEW_SHADER.into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("powdergame-world-view-bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(WORLD_VIEW_PARAMS_SIZE),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("powdergame-world-view-pl"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("powdergame-world-view-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    });

    let params = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("powdergame-world-view-params"),
        size: WORLD_VIEW_PARAMS_SIZE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("powdergame-world-view-bg"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: spec.material_buffer.as_entire_binding(),
            },
        ],
    });

    let world_view = WorldView {
        pipeline,
        bind_group,
        params,
        world_width: spec.width,
        world_height: spec.height,
    };
    write_world_view_params(queue, &world_view, config);
    world_view
}

/// Writes the world-view params uniform from the current surface size.
fn write_world_view_params(
    queue: &wgpu::Queue,
    wv: &WorldView,
    config: &wgpu::SurfaceConfiguration,
) {
    let mut data = [0u8; WORLD_VIEW_PARAMS_SIZE as usize];
    data[0..4].copy_from_slice(&wv.world_width.to_ne_bytes());
    data[4..8].copy_from_slice(&wv.world_height.to_ne_bytes());
    data[8..12].copy_from_slice(&config.width.to_ne_bytes());
    data[12..16].copy_from_slice(&config.height.to_ne_bytes());
    queue.write_buffer(&wv.params, 0, &data);
}
