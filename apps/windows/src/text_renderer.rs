//! High-resolution screen-space text renderer for the Powdergame Windows runtime.
//!
//! This module decouples HUD typography from the WGSL cell shader, rendering
//! crisp, readable vector-rasterized text (Consolas / Segoe UI / Arial) in screen
//! space atop the simulation viewport without obscuring physics observation.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use fontdue::{Font, FontSettings};
use powdergame_gpu::GpuError;

use crate::observatory::{ObservatoryMetrics, PressureObservatoryMetrics};

/// Single vertex for the text / UI quad batcher.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

/// Uniform buffer payload providing screen dimensions for NDC projection.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct ScreenUniform {
    screen_width: f32,
    screen_height: f32,
    _pad0: f32,
    _pad1: f32,
}

/// Metric info for a rasterized character in the atlas.
#[derive(Clone, Copy, Debug)]
struct GlyphMetric {
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    width: f32,
    height: f32,
    offset_x: f32,
    offset_y: f32,
    advance_x: f32,
}

/// Texture atlas containing rasterized font glyphs at various sizes.
pub(crate) struct FontAtlas {
    glyphs: HashMap<(char, u32), GlyphMetric>,
    solid_white_uv: [f32; 2],
}

impl FontAtlas {
    fn build(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(Self, wgpu::Texture, wgpu::TextureView), GpuError> {
        let font_bytes = Self::load_system_font();
        let font = Font::from_bytes(font_bytes, FontSettings::default())
            .map_err(|e| GpuError::Other(format!("failed to parse font: {e}")))?;

        let atlas_w = 1024u32;
        let atlas_h = 1024u32;
        let mut pixels = vec![0u8; (atlas_w * atlas_h) as usize];

        // Reserve top-left 4x4 pixels as solid white (255) for rendering UI rectangles/lines
        for y in 0..4 {
            for x in 0..4 {
                pixels[(y * atlas_w + x) as usize] = 255;
            }
        }
        let solid_white_uv = [2.0 / atlas_w as f32, 2.0 / atlas_h as f32];

        let mut cur_x = 8u32;
        let mut cur_y = 8u32;
        let mut row_h = 0u32;
        let mut glyphs = HashMap::new();

        // Rasterize standard sizes: 13, 15, 18, 24 px
        let sizes = [13u32, 15u32, 18u32, 24u32];

        for &sz in &sizes {
            let px_f32 = sz as f32;
            for ch in (32u8..=126u8).map(|b| b as char) {
                let (metrics, bitmap) = font.rasterize(ch, px_f32);
                let gw = metrics.width as u32;
                let gh = metrics.height as u32;

                if cur_x + gw + 2 >= atlas_w {
                    cur_x = 8;
                    cur_y += row_h + 4;
                    row_h = 0;
                }
                if cur_y + gh + 2 >= atlas_h {
                    eprintln!("[powdergame] warning: font atlas full at char {ch} sz {sz}");
                    break;
                }

                // Copy bitmap into atlas
                for by in 0..gh {
                    for bx in 0..gw {
                        let src_idx = (by * gw + bx) as usize;
                        let dst_idx = ((cur_y + by) * atlas_w + (cur_x + bx)) as usize;
                        pixels[dst_idx] = bitmap[src_idx];
                    }
                }

                let uv_min = [cur_x as f32 / atlas_w as f32, cur_y as f32 / atlas_h as f32];
                let uv_max = [
                    (cur_x + gw) as f32 / atlas_w as f32,
                    (cur_y + gh) as f32 / atlas_h as f32,
                ];

                glyphs.insert(
                    (ch, sz),
                    GlyphMetric {
                        uv_min,
                        uv_max,
                        width: metrics.width as f32,
                        height: metrics.height as f32,
                        offset_x: metrics.xmin as f32,
                        offset_y: metrics.ymin as f32,
                        advance_x: metrics.advance_width,
                    },
                );

                cur_x += gw + 2;
                if gh > row_h {
                    row_h = gh;
                }
            }
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("font_atlas_texture"),
            size: wgpu::Extent3d {
                width: atlas_w,
                height: atlas_h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(atlas_w),
                rows_per_image: Some(atlas_h),
            },
            wgpu::Extent3d {
                width: atlas_w,
                height: atlas_h,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok((
            Self {
                glyphs,
                solid_white_uv,
            },
            texture,
            view,
        ))
    }

    /// Loads a standard Windows system font with fallback options.
    fn load_system_font() -> &'static [u8] {
        static CONSOLAS: &[u8] = include_bytes!(r"C:\Windows\Fonts\consola.ttf");
        CONSOLAS
    }
}

/// Dynamic batch builder for screen-space text and UI cards.
pub struct UiBatch {
    vertices: Vec<TextVertex>,
    indices: Vec<u32>,
}

impl UiBatch {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(4096),
            indices: Vec::with_capacity(6144),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    /// Draws a solid or translucent rectangle.
    pub fn draw_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        white_uv: [f32; 2],
    ) {
        let base = self.vertices.len() as u32;
        self.vertices.push(TextVertex {
            position: [x, y],
            uv: white_uv,
            color,
        });
        self.vertices.push(TextVertex {
            position: [x + w, y],
            uv: white_uv,
            color,
        });
        self.vertices.push(TextVertex {
            position: [x + w, y + h],
            uv: white_uv,
            color,
        });
        self.vertices.push(TextVertex {
            position: [x, y + h],
            uv: white_uv,
            color,
        });

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Draws a card outline / border.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_outline(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        thickness: f32,
        color: [f32; 4],
        white_uv: [f32; 2],
    ) {
        self.draw_rect(x, y, w, thickness, color, white_uv);
        self.draw_rect(x, y + h - thickness, w, thickness, color, white_uv);
        self.draw_rect(
            x,
            y + thickness,
            thickness,
            h - 2.0 * thickness,
            color,
            white_uv,
        );
        self.draw_rect(
            x + w - thickness,
            y + thickness,
            thickness,
            h - 2.0 * thickness,
            color,
            white_uv,
        );
    }

    /// Draws left-aligned text at `(x, y)` where `y` is the top-left baseline offset.
    pub fn draw_text(
        &mut self,
        atlas: &FontAtlas,
        x: f32,
        y: f32,
        size_px: u32,
        text: &str,
        color: [f32; 4],
    ) -> f32 {
        let mut cur_x = x;
        for ch in text.chars() {
            if let Some(metric) = atlas.glyphs.get(&(ch, size_px)) {
                if metric.width > 0.0 && metric.height > 0.0 {
                    let gx = cur_x + metric.offset_x;
                    let gy = y + (size_px as f32 - metric.offset_y - metric.height);

                    let base = self.vertices.len() as u32;
                    self.vertices.push(TextVertex {
                        position: [gx, gy],
                        uv: metric.uv_min,
                        color,
                    });
                    self.vertices.push(TextVertex {
                        position: [gx + metric.width, gy],
                        uv: [metric.uv_max[0], metric.uv_min[1]],
                        color,
                    });
                    self.vertices.push(TextVertex {
                        position: [gx + metric.width, gy + metric.height],
                        uv: metric.uv_max,
                        color,
                    });
                    self.vertices.push(TextVertex {
                        position: [gx, gy + metric.height],
                        uv: [metric.uv_min[0], metric.uv_max[1]],
                        color,
                    });

                    self.indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base,
                        base + 2,
                        base + 3,
                    ]);
                }
                cur_x += metric.advance_x;
            } else if ch == ' ' {
                cur_x += size_px as f32 * 0.55;
            }
        }
        cur_x - x
    }

    /// Draws right-aligned text ending at `right_x`.
    pub fn draw_text_right(
        &mut self,
        atlas: &FontAtlas,
        right_x: f32,
        y: f32,
        size_px: u32,
        text: &str,
        color: [f32; 4],
    ) {
        let w = self.measure_text(atlas, size_px, text);
        self.draw_text(atlas, right_x - w, y, size_px, text, color);
    }

    /// Measures the total advance width of a text string.
    pub fn measure_text(&self, atlas: &FontAtlas, size_px: u32, text: &str) -> f32 {
        let mut w = 0.0f32;
        for ch in text.chars() {
            if let Some(metric) = atlas.glyphs.get(&(ch, size_px)) {
                w += metric.advance_x;
            } else if ch == ' ' {
                w += size_px as f32 * 0.55;
            }
        }
        w
    }
}

/// Standalone screen-space vector text & UI renderer.
pub struct TextRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    screen_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
    atlas: FontAtlas,
    _texture: wgpu::Texture,
    batch: UiBatch,
}

impl TextRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, GpuError> {
        let (atlas, texture, view) = FontAtlas::build(device, queue)?;

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("text_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let screen_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("text_screen_uniform"),
            size: std::mem::size_of::<ScreenUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("text_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: screen_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text_shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
struct ScreenUniform {
    screen_width: f32,
    screen_height: f32,
    _pad0: f32,
    _pad1: f32,
};

@group(0) @binding(0) var<uniform> screen: ScreenUniform;
@group(0) @binding(1) var font_tex: texture_2d<f32>;
@group(0) @binding(2) var font_smp: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let ndc_x = (in.position.x / screen.screen_width) * 2.0 - 1.0;
    let ndc_y = 1.0 - (in.position.y / screen.screen_height) * 2.0;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(font_tex, font_smp, in.uv).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
"#
                .into(),
            ),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 2,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let init_v_cap = 8192;
        let init_i_cap = 12288;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("text_vertex_buffer"),
            size: (init_v_cap * std::mem::size_of::<TextVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("text_index_buffer"),
            size: (init_i_cap * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            pipeline,
            bind_group,
            screen_buffer,
            vertex_buffer,
            index_buffer,
            vertex_capacity: init_v_cap,
            index_capacity: init_i_cap,
            atlas,
            _texture: texture,
            batch: UiBatch::new(),
        })
    }

    /// Renders the complete, high-resolution diagnostic HUD overlay for the Thermal Observatory.
    #[allow(clippy::too_many_arguments)]
    pub fn render_thermal_hud(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        surface_w: u32,
        surface_h: u32,
        metrics: &ObservatoryMetrics,
        sim_ticks: u64,
    ) {
        let sw = surface_w as f32;
        let sh = surface_h as f32;

        self.batch.clear();
        let white_uv = self.atlas.solid_white_uv;

        // Colors
        let col_title = [0.95, 0.96, 0.98, 1.0];
        let col_header = [0.85, 0.90, 0.96, 1.0];
        let col_label = [0.65, 0.70, 0.78, 1.0];
        let col_val_white = [0.98, 0.98, 0.98, 1.0];
        let col_ice_cyan = [0.45, 0.88, 1.0, 1.0];
        let col_warm_yellow = [1.0, 0.82, 0.35, 1.0];
        let col_flame_orange = [1.0, 0.55, 0.20, 1.0];
        let col_fuel_green = [0.40, 0.92, 0.55, 1.0];
        let col_dim = [0.42, 0.46, 0.54, 1.0];

        let col_card_bg = [0.07, 0.09, 0.13, 0.88];
        let col_card_border = [0.18, 0.22, 0.30, 1.0];
        let col_card_divider = [0.14, 0.17, 0.24, 1.0];

        // ─── 1. Top Global Banner ───
        self.batch.draw_text(
            &self.atlas,
            24.0,
            18.0,
            24,
            "G4 THERMAL OBSERVATORY",
            col_title,
        );

        let sim_text = format!("SIM TICK: {:>6}", sim_ticks);
        let sample_text = format!("METRICS SAMPLE: {:>6}", metrics.current_tick);
        let full_tick_str = format!("{}   |   {}", sim_text, sample_text);
        self.batch
            .draw_text_right(&self.atlas, sw - 24.0, 24.0, 15, &full_tick_str, col_header);

        // Calculate layout coordinates
        let sidebar_w = 265.0f32;
        let card_w = sidebar_w - 20.0;
        let left_x = 20.0;
        let right_x = sw - sidebar_w + 10.0;

        let card_top_y = 70.0;
        let card_h = 360.0;
        let card_bot_y = card_top_y + card_h + 20.0;

        // ─── 2. Panel A Card (Top-Left: PHASE HEATING) ───
        self.batch
            .draw_rect(left_x, card_top_y, card_w, card_h, col_card_bg, white_uv);
        self.batch.draw_outline(
            left_x,
            card_top_y,
            card_w,
            card_h,
            1.0,
            col_card_border,
            white_uv,
        );

        let mut y = card_top_y + 14.0;
        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            18,
            "PHASE HEATING",
            col_header,
        );
        y += 26.0;
        self.batch.draw_rect(
            left_x + 16.0,
            y,
            card_w - 32.0,
            1.0,
            col_card_divider,
            white_uv,
        );
        y += 14.0;

        let opt_tick = |v: Option<u64>| match v {
            Some(t) => format!("{t:>6}"),
            None => "    --".to_string(),
        };

        // Panel A Counts
        self.batch
            .draw_text(&self.atlas, left_x + 16.0, y, 15, "Ice Cells:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            &format!("{:>6}", metrics.a_ice_count),
            col_ice_cyan,
        );
        y += 24.0;

        self.batch
            .draw_text(&self.atlas, left_x + 16.0, y, 15, "Water Cells:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            &format!("{:>6}", metrics.a_water_count),
            col_val_white,
        );
        y += 24.0;

        self.batch
            .draw_text(&self.atlas, left_x + 16.0, y, 15, "Steam Cells:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            &format!("{:>6}", metrics.a_steam_count),
            col_warm_yellow,
        );
        y += 28.0;

        self.batch.draw_rect(
            left_x + 16.0,
            y,
            card_w - 32.0,
            1.0,
            col_card_divider,
            white_uv,
        );
        y += 16.0;

        // Panel A First-Events
        self.batch
            .draw_text(&self.atlas, left_x + 16.0, y, 15, "First Melt:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            &opt_tick(metrics.a_first_melt),
            if metrics.a_first_melt.is_some() {
                col_ice_cyan
            } else {
                col_dim
            },
        );
        y += 24.0;

        self.batch
            .draw_text(&self.atlas, left_x + 16.0, y, 15, "First Steam:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            &opt_tick(metrics.a_first_steam),
            if metrics.a_first_steam.is_some() {
                col_warm_yellow
            } else {
                col_dim
            },
        );

        // ─── 3. Panel B Card (Top-Right: PHASE COOLING) ───
        self.batch
            .draw_rect(right_x, card_top_y, card_w, card_h, col_card_bg, white_uv);
        self.batch.draw_outline(
            right_x,
            card_top_y,
            card_w,
            card_h,
            1.0,
            col_card_border,
            white_uv,
        );

        let mut y = card_top_y + 14.0;
        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            18,
            "PHASE COOLING",
            col_header,
        );
        y += 26.0;
        self.batch.draw_rect(
            right_x + 16.0,
            y,
            card_w - 32.0,
            1.0,
            col_card_divider,
            white_uv,
        );
        y += 14.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "Steam Cells:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &format!("{:>6}", metrics.b_steam_count),
            col_warm_yellow,
        );
        y += 24.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "Water Cells:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &format!("{:>6}", metrics.b_water_count),
            col_val_white,
        );
        y += 24.0;

        self.batch
            .draw_text(&self.atlas, right_x + 16.0, y, 15, "Ice Cells:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &format!("{:>6}", metrics.b_ice_count),
            col_ice_cyan,
        );
        y += 28.0;

        self.batch.draw_rect(
            right_x + 16.0,
            y,
            card_w - 32.0,
            1.0,
            col_card_divider,
            white_uv,
        );
        y += 16.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "First Condense:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &opt_tick(metrics.b_first_condense),
            if metrics.b_first_condense.is_some() {
                col_val_white
            } else {
                col_dim
            },
        );
        y += 24.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "First Freeze:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &opt_tick(metrics.b_first_freeze),
            if metrics.b_first_freeze.is_some() {
                col_ice_cyan
            } else {
                col_dim
            },
        );

        // ─── 4. Panel C Card (Bottom-Left: HEAT COMPARISON) ───
        self.batch
            .draw_rect(left_x, card_bot_y, card_w, card_h, col_card_bg, white_uv);
        self.batch.draw_outline(
            left_x,
            card_bot_y,
            card_w,
            card_h,
            1.0,
            col_card_border,
            white_uv,
        );

        let mut y = card_bot_y + 14.0;
        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            18,
            "HEAT COMPARISON",
            col_header,
        );
        y += 26.0;
        self.batch.draw_rect(
            left_x + 16.0,
            y,
            card_w - 32.0,
            1.0,
            col_card_divider,
            white_uv,
        );
        y += 12.0;

        // Water Tube sub-block
        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            15,
            "[WATER TUBE]",
            col_ice_cyan,
        );
        y += 20.0;
        self.batch
            .draw_text(&self.atlas, left_x + 20.0, y, 13, "Low T:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            13,
            &format!("{:.1}", metrics.c_w_low_t),
            col_val_white,
        );
        y += 18.0;
        self.batch
            .draw_text(&self.atlas, left_x + 20.0, y, 13, "Mid T:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            13,
            &format!("{:.1}", metrics.c_w_mid_t),
            col_val_white,
        );
        y += 18.0;
        self.batch
            .draw_text(&self.atlas, left_x + 20.0, y, 13, "High T:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            13,
            &format!("{:.1}", metrics.c_w_high_t),
            col_val_white,
        );
        y += 22.0;

        self.batch.draw_rect(
            left_x + 16.0,
            y,
            card_w - 32.0,
            1.0,
            col_card_divider,
            white_uv,
        );
        y += 12.0;

        // Oil Tube sub-block
        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            15,
            "[OIL TUBE]",
            col_warm_yellow,
        );
        y += 20.0;
        self.batch
            .draw_text(&self.atlas, left_x + 20.0, y, 13, "Low T:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            13,
            &format!("{:.1}", metrics.c_o_low_t),
            col_val_white,
        );
        y += 18.0;
        self.batch
            .draw_text(&self.atlas, left_x + 20.0, y, 13, "Mid T:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            13,
            &format!("{:.1}", metrics.c_o_mid_t),
            col_val_white,
        );
        y += 18.0;
        self.batch
            .draw_text(&self.atlas, left_x + 20.0, y, 13, "High T:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            13,
            &format!("{:.1}", metrics.c_o_high_t),
            col_val_white,
        );

        // ─── 5. Panel D Card (Bottom-Right: COMBUSTION) ───
        self.batch
            .draw_rect(right_x, card_bot_y, card_w, card_h, col_card_bg, white_uv);
        self.batch.draw_outline(
            right_x,
            card_bot_y,
            card_w,
            card_h,
            1.0,
            col_card_border,
            white_uv,
        );

        let mut y = card_bot_y + 14.0;
        self.batch
            .draw_text(&self.atlas, right_x + 16.0, y, 18, "COMBUSTION", col_header);
        y += 26.0;
        self.batch.draw_rect(
            right_x + 16.0,
            y,
            card_w - 32.0,
            1.0,
            col_card_divider,
            white_uv,
        );
        y += 14.0;

        self.batch
            .draw_text(&self.atlas, right_x + 16.0, y, 15, "Wood Start:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &format!("{:>6}", metrics.d_wood_start),
            col_val_white,
        );
        y += 20.0;

        self.batch
            .draw_text(&self.atlas, right_x + 16.0, y, 15, "Wood Left:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &format!("{:>6}", metrics.d_wood_left),
            col_val_white,
        );
        y += 20.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "Burning Cells:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &format!("{:>6}", metrics.d_burning),
            col_flame_orange,
        );
        y += 20.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "Smoke Cells:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &format!("{:>6}", metrics.d_smoke_count),
            col_ice_cyan,
        );
        y += 24.0;

        self.batch.draw_rect(
            right_x + 16.0,
            y,
            card_w - 32.0,
            1.0,
            col_card_divider,
            white_uv,
        );
        y += 14.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "First Ignite:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &opt_tick(metrics.d_first_ignite),
            if metrics.d_first_ignite.is_some() {
                col_flame_orange
            } else {
                col_dim
            },
        );
        y += 20.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "First Empty:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &opt_tick(metrics.d_first_empty),
            if metrics.d_first_empty.is_some() {
                col_fuel_green
            } else {
                col_dim
            },
        );

        // ─── 6. Bottom Controls Bar ───
        let bot_bar_y = sh - 38.0;
        self.batch.draw_text(&self.atlas, 24.0, bot_bar_y, 15, "SPACE Play / Pause (60 TPS)   |   N Single Step   |   R Reset World & Metrics   |   ESC Quit", col_label);

        // ─── Upload and Draw ───
        if self.batch.vertices.is_empty() {
            return;
        }

        let screen_data = ScreenUniform {
            screen_width: sw,
            screen_height: sh,
            _pad0: 0.0,
            _pad1: 0.0,
        };
        queue.write_buffer(&self.screen_buffer, 0, bytemuck::bytes_of(&screen_data));

        // Reallocate vertex/index buffers if needed
        if self.batch.vertices.len() > self.vertex_capacity {
            self.vertex_capacity = (self.batch.vertices.len() * 3) / 2;
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("text_vertex_buffer"),
                size: (self.vertex_capacity * std::mem::size_of::<TextVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if self.batch.indices.len() > self.index_capacity {
            self.index_capacity = (self.batch.indices.len() * 3) / 2;
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("text_index_buffer"),
                size: (self.index_capacity * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.batch.vertices),
        );
        queue.write_buffer(
            &self.index_buffer,
            0,
            bytemuck::cast_slice(&self.batch.indices),
        );

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.batch.indices.len() as u32, 0, 0..1);
    }

    /// Renders the diagnostic HUD overlay for the G5 2x2 Multi-Boiler Pressure Lab.
    #[allow(clippy::too_many_arguments)]
    pub fn render_pressure_hud(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        surface_w: u32,
        surface_h: u32,
        metrics: &PressureObservatoryMetrics,
        sim_ticks: u64,
    ) {
        let sw = surface_w as f32;
        let sh = surface_h as f32;

        self.batch.clear();
        let white_uv = self.atlas.solid_white_uv;

        // Color palette
        let col_title = [0.95, 0.96, 0.98, 1.0];
        let col_header = [0.85, 0.90, 0.96, 1.0];
        let col_label = [0.65, 0.70, 0.78, 1.0];
        let col_val_white = [0.98, 0.98, 0.98, 1.0];
        let col_relief_green = [0.35, 0.95, 0.60, 1.0];
        let col_warn_orange = [1.0, 0.65, 0.20, 1.0];
        let col_breach_red = [1.0, 0.35, 0.30, 1.0];
        let col_stone_cyan = [0.45, 0.85, 1.0, 1.0];
        let col_dim = [0.45, 0.48, 0.56, 1.0];

        let col_card_bg = [0.07, 0.09, 0.13, 0.90];
        let col_card_border = [0.18, 0.22, 0.30, 1.0];
        let col_card_divider = [0.14, 0.17, 0.24, 1.0];

        // 1. Top Global Banner
        self.batch.draw_text(
            &self.atlas,
            24.0,
            16.0,
            24,
            "G5 PRESSURE CHAIN | 2x2 MULTI-BOILER STRESS LAB",
            col_title,
        );

        let sim_text = format!("SIM TICK: {:>6}", sim_ticks);
        let sample_text = format!("DIAGNOSTIC SAMPLE: {:>6}", metrics.current_tick);
        let full_tick_str = format!("{}   |   {}", sim_text, sample_text);
        self.batch
            .draw_text_right(&self.atlas, sw - 24.0, 22.0, 15, &full_tick_str, col_header);

        // Sidebar dimensions
        let sidebar_w = 340.0f32;
        let card_w = sidebar_w - 20.0;
        let left_x = 20.0;
        let right_x = sw - sidebar_w + 10.0;

        let card_top_y = 65.0;
        let card_h = 350.0;
        let card_bot_y = card_top_y + card_h + 20.0;

        let opt_tick = |v: Option<u64>| match v {
            Some(t) => format!("Tick {t:>5}"),
            None => "  PENDING".to_string(),
        };

        // 2. Panel A Card (Top-Left: WOOD RELIEF CANONICAL STANDARD)
        self.batch
            .draw_rect(left_x, card_top_y, card_w, card_h, col_card_bg, white_uv);
        self.batch.draw_outline(
            left_x,
            card_top_y,
            card_w,
            card_h,
            1.0,
            col_card_border,
            white_uv,
        );

        let mut y = card_top_y + 14.0;
        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            18,
            "[A] WOOD RELIEF (CANONICAL)",
            col_header,
        );
        y += 24.0;
        self.batch.draw_rect(
            left_x + 16.0,
            y,
            card_w - 32.0,
            1.0,
            col_card_divider,
            white_uv,
        );
        y += 12.0;

        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            13,
            "Heaters: Floor T=150 + Upper T=110",
            col_dim,
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            15,
            "Peak Pressure:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            &format!("{:.1}", metrics.tl_peak_pressure),
            col_val_white,
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            15,
            "First Relief:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            &opt_tick(metrics.tl_relief_tick),
            if metrics.tl_relief_tick.is_some() {
                col_relief_green
            } else {
                col_dim
            },
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            15,
            "Relief Plug Wood:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            &format!("{}/9 cells", metrics.tl_wood_remaining),
            col_val_white,
        );
        y += 22.0;

        self.batch
            .draw_text(&self.atlas, left_x + 16.0, y, 15, "Steam Cells:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            &format!("{}", metrics.tl_steam_count),
            col_stone_cyan,
        );
        y += 26.0;

        let status_a = if metrics.tl_relief_tick.is_some() {
            "[RELIEF ACTIVE / VENTING]"
        } else if metrics.tl_steam_count > 0 {
            "[PRESSURE BUILDING]"
        } else {
            "[HEATING WATER]"
        };
        self.batch
            .draw_text(&self.atlas, left_x + 16.0, y, 15, "State:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            status_a,
            if metrics.tl_relief_tick.is_some() {
                col_relief_green
            } else {
                col_warn_orange
            },
        );

        // 3. Panel B Card (Top-Right: STONE SEALED STANDARD CONTROL)
        self.batch
            .draw_rect(right_x, card_top_y, card_w, card_h, col_card_bg, white_uv);
        self.batch.draw_outline(
            right_x,
            card_top_y,
            card_w,
            card_h,
            1.0,
            col_card_border,
            white_uv,
        );

        let mut y = card_top_y + 14.0;
        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            18,
            "[B] STONE SEALED (CONTROL)",
            col_header,
        );
        y += 24.0;
        self.batch.draw_rect(
            right_x + 16.0,
            y,
            card_w - 32.0,
            1.0,
            col_card_divider,
            white_uv,
        );
        y += 12.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            13,
            "Heaters: Floor T=150 + Upper T=110",
            col_dim,
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "Peak Pressure:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &format!("{:.1}", metrics.tr_peak_pressure),
            col_val_white,
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "Rupture Event:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            "NONE (UNBREAKABLE)",
            col_stone_cyan,
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "Chamber Integrity:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            "100% SEALED",
            col_stone_cyan,
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "Steam Cells:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &format!("{}", metrics.tr_steam_count),
            col_stone_cyan,
        );
        y += 26.0;

        self.batch
            .draw_text(&self.atlas, right_x + 16.0, y, 15, "State:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            "[PERMANENT CONFINEMENT]",
            col_stone_cyan,
        );

        // 4. Panel C Card (Bottom-Left: WOOD RELIEF EXTREME OVERDRIVE)
        self.batch
            .draw_rect(left_x, card_bot_y, card_w, card_h, col_card_bg, white_uv);
        self.batch.draw_outline(
            left_x,
            card_bot_y,
            card_w,
            card_h,
            1.0,
            col_card_border,
            white_uv,
        );

        let mut y = card_bot_y + 14.0;
        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            18,
            "[C] WOOD RELIEF (EXTREME)",
            col_warn_orange,
        );
        y += 24.0;
        self.batch.draw_rect(
            left_x + 16.0,
            y,
            card_w - 32.0,
            1.0,
            col_card_divider,
            white_uv,
        );
        y += 12.0;

        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            13,
            "Heaters: 3x Floor T=220 + Upper T=130",
            col_warn_orange,
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            15,
            "Peak Pressure:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            &format!("{:.1}", metrics.bl_peak_pressure),
            col_val_white,
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            15,
            "First Relief:",
            col_label,
        );
        let bl_relief_str = match metrics.bl_relief_tick {
            Some(t) => format!("Tick {t:>5} (FAST)"),
            None => "  PENDING".to_string(),
        };
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            &bl_relief_str,
            if metrics.bl_relief_tick.is_some() {
                col_relief_green
            } else {
                col_dim
            },
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            left_x + 16.0,
            y,
            15,
            "Relief Plug Wood:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            &format!("{}/9 cells", metrics.bl_wood_remaining),
            col_val_white,
        );
        y += 22.0;

        self.batch
            .draw_text(&self.atlas, left_x + 16.0, y, 15, "Steam Cells:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            &format!("{}", metrics.bl_steam_count),
            col_warn_orange,
        );
        y += 26.0;

        let status_c = if metrics.bl_relief_tick.is_some() {
            "[OVERDRIVE VENT PLUME]"
        } else if metrics.bl_steam_count > 0 {
            "[RAPID ESCALATION]"
        } else {
            "[SUPERHEATING]"
        };
        self.batch
            .draw_text(&self.atlas, left_x + 16.0, y, 15, "State:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            left_x + card_w - 16.0,
            y,
            15,
            status_c,
            if metrics.bl_relief_tick.is_some() {
                col_relief_green
            } else {
                col_warn_orange
            },
        );

        // 5. Panel D Card (Bottom-Right: STONE SEALED EXTREME -> CATASTROPHIC BREACH)
        self.batch
            .draw_rect(right_x, card_bot_y, card_w, card_h, col_card_bg, white_uv);
        self.batch.draw_outline(
            right_x,
            card_bot_y,
            card_w,
            card_h,
            1.0,
            col_card_border,
            white_uv,
        );

        let mut y = card_bot_y + 14.0;
        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            18,
            "[D] DELAYED PRESSURE BREACH",
            col_breach_red,
        );
        y += 24.0;
        self.batch.draw_rect(
            right_x + 16.0,
            y,
            card_w - 32.0,
            1.0,
            col_card_divider,
            white_uv,
        );
        y += 12.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            13,
            "Heaters: 3x Floor T=220 + Upper T=130",
            col_dim,
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "Peak Pressure:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &format!("{:.1}", metrics.br_peak_pressure),
            if metrics.br_peak_pressure > 80.0 {
                col_breach_red
            } else {
                col_val_white
            },
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "First Breach:",
            col_label,
        );
        let br_rupture_str = match metrics.br_rupture_tick {
            Some(t) => format!("Tick {t:>5} (DELAYED)"),
            None => "  ACCUMULATING".to_string(),
        };
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &br_rupture_str,
            if metrics.br_rupture_tick.is_some() {
                col_breach_red
            } else {
                col_warn_orange
            },
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "Weak Seam Wood:",
            col_label,
        );
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &format!("{}/9 cells", metrics.br_weak_seam_remaining),
            if metrics.br_weak_seam_remaining < 9 {
                col_breach_red
            } else {
                col_val_white
            },
        );
        y += 22.0;

        self.batch.draw_text(
            &self.atlas,
            right_x + 16.0,
            y,
            15,
            "Duct Steam Vent:",
            col_label,
        );
        let vent_str = match metrics.br_first_vent_tick {
            Some(t) => format!("Tick {t:>5} ({} cells)", metrics.br_exterior_steam_count),
            None => "  AWAITING BREACH".to_string(),
        };
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            &vent_str,
            if metrics.br_first_vent_tick.is_some() {
                col_breach_red
            } else {
                col_dim
            },
        );
        y += 26.0;

        let status_d = if metrics.br_first_vent_tick.is_some() {
            "SIDE WALL BREACH -> VENTING"
        } else if metrics.br_rupture_tick.is_some() {
            "[RUPTURE OPENED]"
        } else if metrics.br_current_pressure > 80.0 {
            "[CRITICAL OVERPRESSURE]"
        } else if metrics.br_steam_count > 0 {
            "[PRESSURE PROPAGATING]"
        } else {
            "[SUPERHEATING]"
        };
        self.batch
            .draw_text(&self.atlas, right_x + 16.0, y, 15, "State:", col_label);
        self.batch.draw_text_right(
            &self.atlas,
            right_x + card_w - 16.0,
            y,
            15,
            status_d,
            if metrics.br_rupture_tick.is_some() {
                col_breach_red
            } else {
                col_warn_orange
            },
        );

        // 6. Bottom Controls Bar
        let bot_bar_y = sh - 32.0;
        self.batch.draw_text(&self.atlas, 24.0, bot_bar_y, 15, "SPACE Play / Pause (60 TPS)   |   N Single Step   |   R Reset World & Metrics   |   ESC Quit", col_label);

        // Upload and Draw
        if self.batch.vertices.is_empty() {
            return;
        }

        let screen_data = ScreenUniform {
            screen_width: sw,
            screen_height: sh,
            _pad0: 0.0,
            _pad1: 0.0,
        };
        queue.write_buffer(&self.screen_buffer, 0, bytemuck::bytes_of(&screen_data));

        if self.batch.vertices.len() > self.vertex_capacity {
            self.vertex_capacity = (self.batch.vertices.len() * 3) / 2;
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("text_vertex_buffer"),
                size: (self.vertex_capacity * std::mem::size_of::<TextVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if self.batch.indices.len() > self.index_capacity {
            self.index_capacity = (self.batch.indices.len() * 3) / 2;
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("text_index_buffer"),
                size: (self.index_capacity * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.batch.vertices),
        );
        queue.write_buffer(
            &self.index_buffer,
            0,
            bytemuck::cast_slice(&self.batch.indices),
        );

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.batch.indices.len() as u32, 0, 0..1);
    }
}
