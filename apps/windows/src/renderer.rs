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

/// Presentation-only color mode. Material IDs are never remapped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationPalette {
    /// G2 forest demo: Stone reads as forest-green terrain.
    Forest = 0,
    /// G3 density demo: Stone reads as neutral laboratory gray.
    Lab = 1,
}

/// Read-only view spec for presenting the material world (G2 / G3).
pub struct WorldViewSpec<'a> {
    pub material_buffer: &'a wgpu::Buffer,
    pub width: u32,
    pub height: u32,
    pub palette: PresentationPalette,
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
    palette: u32,
}

/// Clear color for the empty G0 world frame (a dim slate blue).
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.02,
    g: 0.02,
    b: 0.05,
    a: 1.0,
};

/// Params uniform: world size + surface size + palette id (8 u32 = 32 B).
const WORLD_VIEW_PARAMS_SIZE: u64 = 32;

const WORLD_VIEW_SHADER: &str = r#"
struct Params {
    width: u32,
    height: u32,
    surface_w: u32,
    surface_h: u32,
    palette: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
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
const PALETTE_LAB: u32 = 1u;

// 3x5 uppercase glyphs, bit = gy*3+gx. Presentation overlay only.
fn glyph_bits(code: u32) -> u32 {
    switch code {
        case 32u: { return 0u; }
        case 43u: { return 0x05D0u; } // +
        case 51u: { return 0x79E7u; } // 3
        case 65u: { return 0x5BEAu; } // A
        case 67u: { return 0x624Eu; } // C
        case 68u: { return 0x3B6Bu; } // D
        case 69u: { return 0x72CFu; } // E
        case 71u: { return 0x6B4Eu; } // G
        case 73u: { return 0x7497u; } // I
        case 75u: { return 0x5AEDu; } // K
        case 76u: { return 0x7249u; } // L
        case 77u: { return 0x5B7Du; } // M
        case 78u: { return 0x5B5Du; } // N
        case 79u: { return 0x7B6Fu; } // O
        case 80u: { return 0x12EBu; } // P
        case 82u: { return 0x5AEBu; } // R
        case 83u: { return 0x388Eu; } // S
        case 84u: { return 0x2497u; } // T
        case 87u: { return 0x5F6Du; } // W
        case 89u: { return 0x24ADu; } // Y
        case 94u: { return 0x24BAu; } // ^
        case 118u: { return 0x2E92u; } // v
        default: { return 0u; }
    }
}

fn text_hit(px: f32, py: f32, origin_x: f32, origin_y: f32, cell: f32, codes: array<u32, 16>, n: u32) -> bool {
    let rel_x = px - origin_x;
    let rel_y = py - origin_y;
    if (rel_x < 0.0 || rel_y < 0.0) { return false; }
    let step = cell * 4.0;
    let col = u32(rel_x / step);
    if (col >= n) { return false; }
    let gx = u32((rel_x % step) / cell);
    let gy = u32(rel_y / cell);
    if (gx >= 3u || gy >= 5u) { return false; }
    let bits = glyph_bits(codes[col]);
    return ((bits >> (gy * 3u + gx)) & 1u) == 1u;
}

fn centered_origin(center_x: f32, n: u32, cell: f32) -> f32 {
    let text_w = f32(n) * cell * 4.0 - cell;
    return center_x - text_w * 0.5;
}

// Presentation-only debug palette (material IDs never change).
// Forest: Stone is green terrain/trees. Lab: Stone is neutral chamber gray.
fn debug_color(id: u32, palette: u32) -> vec4<f32> {
    if (palette == PALETTE_LAB) {
        if (id == EMPTY) { return vec4<f32>(0.05, 0.055, 0.07, 1.0); }
        if (id == BOUNDARY) { return vec4<f32>(0.22, 0.23, 0.25, 1.0); }
        if (id == STONE) { return vec4<f32>(0.46, 0.47, 0.50, 1.0); }
        if (id == SAND) { return vec4<f32>(0.96, 0.82, 0.28, 1.0); }
        if (id == WATER) { return vec4<f32>(0.12, 0.48, 0.96, 1.0); }
        if (id == OIL) { return vec4<f32>(0.66, 0.38, 0.10, 1.0); }
        if (id == STEAM) { return vec4<f32>(0.96, 0.97, 0.99, 1.0); }
        if (id == SMOKE) { return vec4<f32>(0.18, 0.18, 0.20, 1.0); }
        return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    }
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
// Lab palette reserves thin HUD bands above/below the world for labels.
@fragment
fn fs_main(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let fw = f32(params.surface_w);
    let fh = f32(params.surface_h);
    let ww = f32(params.width);
    let wh = f32(params.height);
    let lab = params.palette == PALETTE_LAB;
    var scale = min(fw / ww, fh / wh);
    var off_x = (fw - ww * scale) * 0.5;
    var off_y = (fh - wh * scale) * 0.5;
    if (lab) {
        let hud_top = fh * 0.10;
        let hud_bot = fh * 0.13;
        let avail_h = max(fh - hud_top - hud_bot, 1.0);
        scale = min(fw / ww, avail_h / wh);
        off_x = (fw - ww * scale) * 0.5;
        off_y = hud_top + (avail_h - wh * scale) * 0.5;
    }
    let px = frag.x;
    let py = frag.y;
    let in_viewport = px >= off_x && px < off_x + ww * scale
                   && py >= off_y && py < off_y + wh * scale;
    if (in_viewport) {
        let cell_x = min(u32((px - off_x) / scale), params.width - 1u);
        let cell_y = min(u32((py - off_y) / scale), params.height - 1u);
        let idx = cell_y * params.width + cell_x;
        return debug_color(materials[idx], params.palette);
    }
    if (lab) {
        let hud = lab_hud(px, py, fw, fh, off_x, off_y, scale);
        if (hud.a > 0.0) {
            return hud;
        }
        return vec4<f32>(0.09, 0.10, 0.12, 1.0);
    }
    return vec4<f32>(0.06, 0.07, 0.10, 1.0); // letterbox background
}

fn lab_hud(px: f32, py: f32, fw: f32, fh: f32, off_x: f32, off_y: f32, scale: f32) -> vec4<f32> {
    let cell = max(2.0, min(3.0, scale * 0.55));
    let c1 = off_x + 21.5 * scale;
    let c2 = off_x + 63.5 * scale;
    let c3 = off_x + 105.5 * scale;
    let title_y = fh * 0.018;
    let label_y = max(title_y + cell * 6.5, off_y - cell * 7.0);
    let cap0 = off_y + 128.0 * scale + cell * 1.2;
    let cap1 = cap0 + cell * 6.2;
    let sand = vec4<f32>(0.96, 0.82, 0.28, 1.0);
    let water = vec4<f32>(0.45, 0.70, 1.0, 1.0);
    let oil = vec4<f32>(0.78, 0.52, 0.22, 1.0);
    let steam = vec4<f32>(0.94, 0.95, 0.97, 1.0);
    let smoke = vec4<f32>(0.55, 0.55, 0.58, 1.0);
    let ink = vec4<f32>(0.90, 0.91, 0.93, 1.0);

    var title = array<u32, 16>();
    title[0] = 71u; title[1] = 51u; title[2] = 32u; title[3] = 68u;
    title[4] = 69u; title[5] = 78u; title[6] = 83u; title[7] = 73u;
    title[8] = 84u; title[9] = 89u; title[10] = 32u; title[11] = 68u;
    title[12] = 69u; title[13] = 77u; title[14] = 79u;
    if (text_hit(px, py, centered_origin(fw * 0.5, 15u, cell), title_y, cell, title, 15u)) {
        return ink;
    }

    var sw = array<u32, 16>();
    sw[0] = 83u; sw[1] = 65u; sw[2] = 78u; sw[3] = 68u; sw[4] = 32u;
    sw[5] = 43u; sw[6] = 32u; sw[7] = 87u; sw[8] = 65u; sw[9] = 84u;
    sw[10] = 69u; sw[11] = 82u;
    if (text_hit(px, py, centered_origin(c1, 12u, cell), label_y, cell, sw, 12u)) {
        return ink;
    }
    var wo = array<u32, 16>();
    wo[0] = 87u; wo[1] = 65u; wo[2] = 84u; wo[3] = 69u; wo[4] = 82u;
    wo[5] = 32u; wo[6] = 43u; wo[7] = 32u; wo[8] = 79u; wo[9] = 73u;
    wo[10] = 76u;
    if (text_hit(px, py, centered_origin(c2, 11u, cell), label_y, cell, wo, 11u)) {
        return ink;
    }
    var ss = array<u32, 16>();
    ss[0] = 83u; ss[1] = 84u; ss[2] = 69u; ss[3] = 65u; ss[4] = 77u;
    ss[5] = 32u; ss[6] = 43u; ss[7] = 32u; ss[8] = 83u; ss[9] = 77u;
    ss[10] = 79u; ss[11] = 75u; ss[12] = 69u;
    if (text_hit(px, py, centered_origin(c3, 13u, cell), label_y, cell, ss, 13u)) {
        return ink;
    }

    var a0 = array<u32, 16>();
    a0[0] = 83u; a0[1] = 65u; a0[2] = 78u; a0[3] = 68u; a0[4] = 32u;
    a0[5] = 83u; a0[6] = 73u; a0[7] = 78u; a0[8] = 75u; a0[9] = 83u;
    a0[10] = 32u; a0[11] = 118u;
    if (text_hit(px, py, centered_origin(c1, 12u, cell), cap0, cell, a0, 12u)) {
        return sand;
    }
    var a1 = array<u32, 16>();
    a1[0] = 87u; a1[1] = 65u; a1[2] = 84u; a1[3] = 69u; a1[4] = 82u;
    a1[5] = 32u; a1[6] = 82u; a1[7] = 73u; a1[8] = 83u; a1[9] = 69u;
    a1[10] = 83u; a1[11] = 32u; a1[12] = 94u;
    if (text_hit(px, py, centered_origin(c1, 13u, cell), cap1, cell, a1, 13u)) {
        return water;
    }
    var b0 = array<u32, 16>();
    b0[0] = 87u; b0[1] = 65u; b0[2] = 84u; b0[3] = 69u; b0[4] = 82u;
    b0[5] = 32u; b0[6] = 83u; b0[7] = 73u; b0[8] = 78u; b0[9] = 75u;
    b0[10] = 83u; b0[11] = 32u; b0[12] = 118u;
    if (text_hit(px, py, centered_origin(c2, 13u, cell), cap0, cell, b0, 13u)) {
        return water;
    }
    var b1 = array<u32, 16>();
    b1[0] = 79u; b1[1] = 73u; b1[2] = 76u; b1[3] = 32u; b1[4] = 82u;
    b1[5] = 73u; b1[6] = 83u; b1[7] = 69u; b1[8] = 83u; b1[9] = 32u;
    b1[10] = 94u;
    if (text_hit(px, py, centered_origin(c2, 11u, cell), cap1, cell, b1, 11u)) {
        return oil;
    }
    var c0 = array<u32, 16>();
    c0[0] = 83u; c0[1] = 84u; c0[2] = 69u; c0[3] = 65u; c0[4] = 77u;
    c0[5] = 32u; c0[6] = 82u; c0[7] = 73u; c0[8] = 83u; c0[9] = 69u;
    c0[10] = 83u; c0[11] = 32u; c0[12] = 94u;
    if (text_hit(px, py, centered_origin(c3, 13u, cell), cap0, cell, c0, 13u)) {
        return steam;
    }
    var c1t = array<u32, 16>();
    c1t[0] = 83u; c1t[1] = 77u; c1t[2] = 79u; c1t[3] = 75u; c1t[4] = 69u;
    c1t[5] = 32u; c1t[6] = 83u; c1t[7] = 73u; c1t[8] = 78u; c1t[9] = 75u;
    c1t[10] = 83u; c1t[11] = 32u; c1t[12] = 118u;
    if (text_hit(px, py, centered_origin(c3, 13u, cell), cap1, cell, c1t, 13u)) {
        return smoke;
    }
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
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
        palette: spec.palette as u32,
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
    data[16..20].copy_from_slice(&wv.palette.to_ne_bytes());
    queue.write_buffer(&wv.params, 0, &data);
}
