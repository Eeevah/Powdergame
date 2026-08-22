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
//! G4 adds `temperature_current` + `flags_current` as additional read-only
//! bindings used by the ThermalLab palette only: a presentation-only
//! thermal tint over the material color, and a flame-like overlay for
//! `FLAG_COMBUSTING` / `FLAG_FLAME_EVENT` (Fire is NOT Matter; the flame is
//! the burning Wood/Oil itself). G2/G3 modes ignore these buffers entirely.
//!
//! ThermalLab presentation policy (UV rounds 2-4):
//!   - Smoke is NOT a validation focus: it renders near-background dark
//!     gray with NO temperature tint, so it never reads as orange/yellow
//!     Matter and never dominates the screen.
//!   - Steam keeps a pale identity with a very weak tint.
//!   - The HEAT COMPARE zone (panel C, x 213..318) abandons subtle material
//!     tint for the liquids and uses a strong temperature diagnostic
//!     false-color ramp (deep blue → cyan → yellow → orange → near-white)
//!     so the conduction front climbing the sealed Water/Oil/Stone/Sand
//!     tubes is distinguishable by screenshot at tick 0/250/500/1000.
//!     Material identity remains via the tube labels + a weak hue offset.
//!     Diagnostic view, not final art.
//!   - Wood: normal = brown, hot non-burning = red-brown (heat building
//!     before ignition), COMBUSTING = vivid orange/yellow. The ignition
//!     front travelling along the strip reads before Smoke does.
//!   - The combustion overlay is applied LAST and only to Wood/Oil.
//!
//! The world view preserves square cells. [`WorldViewport`] is the one CPU
//! authority for the palette-specific HUD reservation + letterbox rectangle;
//! the shader consumes that rectangle and physical-pixel picking reuses it.
//! Pixels map to cells with integer truncation, so cell edges stay crisp and
//! the world aspect ratio is never distorted.

use std::sync::Arc;

use wgpu::util::DeviceExt;
use wgpu::TextureFormat;

use powdergame_gpu::GpuError;
use winit::{dpi::PhysicalPosition, window::Window};

/// Presentation-only color mode. Material IDs are never remapped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationPalette {
    /// G2 forest demo: Stone reads as forest-green terrain.
    Forest = 0,
    /// G3 density demo: Stone reads as neutral laboratory gray.
    Lab = 1,
    /// G4 thermal lab demo: Stone neutral, + thermal tint + flame overlay.
    ThermalLab = 2,
    /// G6 parallel-integrity lab: neutral Lab-style cell colors with NO
    /// procedural G3 HUD overlay (the G6 HUD is drawn by the screen-space
    /// text renderer so panel titles + readback metrics stay legible).
    Integrity = 3,
    /// G7 activity observatory: Lab-style base colors + per-chunk activity
    /// heatmap overlay (read-only chunk_activity storage binding).
    Activity = 4,
    /// G8-B benchmark Gallery: neutral material identity plus generic,
    /// coordinate-independent temperature, pressure, and reaction tinting.
    Gallery = 5,
    /// G9-A first-playable Sandbox: neutral product colors with the same
    /// camera transform used by rendering, picking, and the Inspector.
    Sandbox = 6,
    /// TE-2 direct-review candidate: ThermalLab colors with wider persistent
    /// diagnostic-card reservations on both sides of the world.
    ThermalEnvironment = 7,
}

/// Physical-pixel rectangle occupied by the rendered world.
///
/// The calculations intentionally use `f32`, matching the values consumed by
/// WGSL. `x`/`y` are inclusive; `right()`/`bottom()` are exclusive. The world
/// cell origin is the top-left of this rectangle, matching fragment indexing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    world_width: u32,
    world_height: u32,
}

impl WorldViewport {
    /// Calculates the exact palette-specific viewport used by the shader.
    /// All dimensions are physical pixels; zero-sized surfaces/worlds have no
    /// pickable viewport.
    pub fn calculate(
        surface_width: u32,
        surface_height: u32,
        world_width: u32,
        world_height: u32,
        palette: PresentationPalette,
    ) -> Option<Self> {
        if surface_width == 0 || surface_height == 0 || world_width == 0 || world_height == 0 {
            return None;
        }

        let surface_width = surface_width as f32;
        let surface_height = surface_height as f32;
        let world_width_f = world_width as f32;
        let world_height_f = world_height as f32;

        // These reservations are the existing presentation contract. Keeping
        // them here means shader drawing and CPU picking cannot drift apart.
        let (available_width, available_height, available_top) = match palette {
            PresentationPalette::Forest => (surface_width, surface_height, 0.0),
            PresentationPalette::Lab => {
                let hud_top = surface_height * 0.10;
                let hud_bottom = surface_height * 0.13;
                (
                    surface_width,
                    (surface_height - hud_top - hud_bottom).max(1.0),
                    hud_top,
                )
            }
            PresentationPalette::ThermalLab => (
                (surface_width - 270.0 * 2.0).max(1.0),
                (surface_height - 140.0).max(1.0),
                65.0,
            ),
            PresentationPalette::ThermalEnvironment => (
                (surface_width - 370.0 * 2.0).max(1.0),
                (surface_height - 140.0).max(1.0),
                65.0,
            ),
            PresentationPalette::Integrity
            | PresentationPalette::Activity
            | PresentationPalette::Gallery => (
                (surface_width - 400.0 * 2.0).max(1.0),
                (surface_height - 140.0).max(1.0),
                60.0,
            ),
            PresentationPalette::Sandbox => (
                (surface_width - 310.0 * 2.0).max(1.0),
                (surface_height - 120.0).max(1.0),
                56.0,
            ),
        };

        let scale = (available_width / world_width_f).min(available_height / world_height_f);
        let width = world_width_f * scale;
        let height = world_height_f * scale;
        let x = (surface_width - width) * 0.5;
        let y = available_top + (available_height - height) * 0.5;

        Some(Self {
            x,
            y,
            width,
            height,
            scale,
            world_width,
            world_height,
        })
    }

    pub fn right(self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(self) -> f32 {
        self.y + self.height
    }

    /// Maps a physical cursor point to a top-left-origin world cell.
    #[allow(dead_code)] // retained as the full-world compatibility picker
    pub fn cell_at(self, cursor: PhysicalPosition<f64>) -> Option<(u32, u32)> {
        if !cursor.x.is_finite() || !cursor.y.is_finite() {
            return None;
        }

        let left = f64::from(self.x);
        let top = f64::from(self.y);
        let right = f64::from(self.right());
        let bottom = f64::from(self.bottom());
        if cursor.x < left || cursor.x >= right || cursor.y < top || cursor.y >= bottom {
            return None;
        }

        let scale = f64::from(self.scale);
        let cell_x = ((cursor.x - left) / scale).floor() as u32;
        let cell_y = ((cursor.y - top) / scale).floor() as u32;
        (cell_x < self.world_width && cell_y < self.world_height).then_some((cell_x, cell_y))
    }
}

/// Finite, clamped world-space camera state. `zoom == 1` is the full-world
/// fitted view; larger values zoom in around `center_*`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldCamera {
    pub center_x: f32,
    pub center_y: f32,
    pub zoom: f32,
}

impl WorldCamera {
    pub fn fitted(world_width: u32, world_height: u32) -> Self {
        Self {
            center_x: world_width as f32 * 0.5,
            center_y: world_height as f32 * 0.5,
            zoom: 1.0,
        }
    }

    fn normalized(self, world_width: u32, world_height: u32) -> Self {
        let mut camera = self;
        if !camera.zoom.is_finite() {
            camera.zoom = 1.0;
        }
        camera.zoom = camera.zoom.clamp(1.0, 16.0);
        if !camera.center_x.is_finite() {
            camera.center_x = world_width as f32 * 0.5;
        }
        if !camera.center_y.is_finite() {
            camera.center_y = world_height as f32 * 0.5;
        }
        let half_w = world_width as f32 / (2.0 * camera.zoom);
        let half_h = world_height as f32 / (2.0 * camera.zoom);
        camera.center_x = camera.center_x.clamp(half_w, world_width as f32 - half_w);
        camera.center_y = camera.center_y.clamp(half_h, world_height as f32 - half_h);
        camera
    }

    fn panned_by_pixels(self, viewport: WorldViewport, delta_x: f32, delta_y: f32) -> Self {
        if !delta_x.is_finite() || !delta_y.is_finite() {
            return self.normalized(viewport.world_width, viewport.world_height);
        }
        let transform = WorldTransform::calculate(viewport, self);
        Self {
            center_x: self.center_x - delta_x / transform.scale,
            center_y: self.center_y - delta_y / transform.scale,
            zoom: self.zoom,
        }
        .normalized(viewport.world_width, viewport.world_height)
    }

    fn zoomed_at(
        self,
        viewport: WorldViewport,
        cursor: PhysicalPosition<f64>,
        factor: f32,
    ) -> Self {
        if !factor.is_finite() || factor <= 0.0 {
            return self.normalized(viewport.world_width, viewport.world_height);
        }
        let before = WorldTransform::calculate(viewport, self);
        let Some(anchor) = before.world_at(cursor) else {
            return self.normalized(viewport.world_width, viewport.world_height);
        };
        let zoom = (self.zoom * factor).clamp(1.0, 16.0);
        let scale = viewport.scale * zoom;
        let cursor_world_offset_x = (cursor.x as f32 - viewport.x) / scale;
        let cursor_world_offset_y = (cursor.y as f32 - viewport.y) / scale;
        let visible_w = viewport.world_width as f32 / zoom;
        let visible_h = viewport.world_height as f32 / zoom;
        Self {
            center_x: anchor.0 - cursor_world_offset_x + visible_w * 0.5,
            center_y: anchor.1 - cursor_world_offset_y + visible_h * 0.5,
            zoom,
        }
        .normalized(viewport.world_width, viewport.world_height)
    }
}

/// Exact camera-aware physical-pixel transform shared by shader and picking.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldTransform {
    pub viewport: WorldViewport,
    pub origin_x: f32,
    pub origin_y: f32,
    pub scale: f32,
    world_width: u32,
    world_height: u32,
}

impl WorldTransform {
    pub(crate) fn calculate(viewport: WorldViewport, camera: WorldCamera) -> Self {
        let camera = camera.normalized(viewport.world_width, viewport.world_height);
        let visible_w = viewport.world_width as f32 / camera.zoom;
        let visible_h = viewport.world_height as f32 / camera.zoom;
        Self {
            viewport,
            origin_x: camera.center_x - visible_w * 0.5,
            origin_y: camera.center_y - visible_h * 0.5,
            scale: viewport.scale * camera.zoom,
            world_width: viewport.world_width,
            world_height: viewport.world_height,
        }
    }

    pub fn cell_at(self, cursor: PhysicalPosition<f64>) -> Option<(u32, u32)> {
        if !cursor.x.is_finite() || !cursor.y.is_finite() {
            return None;
        }
        if cursor.x < f64::from(self.viewport.x)
            || cursor.x >= f64::from(self.viewport.right())
            || cursor.y < f64::from(self.viewport.y)
            || cursor.y >= f64::from(self.viewport.bottom())
        {
            return None;
        }
        let x = (f64::from(self.origin_x)
            + (cursor.x - f64::from(self.viewport.x)) / f64::from(self.scale))
        .floor() as u32;
        let y = (f64::from(self.origin_y)
            + (cursor.y - f64::from(self.viewport.y)) / f64::from(self.scale))
        .floor() as u32;
        (x < self.world_width && y < self.world_height).then_some((x, y))
    }

    fn world_at(self, cursor: PhysicalPosition<f64>) -> Option<(f32, f32)> {
        self.cell_at(cursor)?;
        Some((
            self.origin_x + (cursor.x as f32 - self.viewport.x) / self.scale,
            self.origin_y + (cursor.y as f32 - self.viewport.y) / self.scale,
        ))
    }
}

/// Pure physical-pixel picking entry point. `None` represents modes without a
/// `WorldView` (for example the explicit runtime baseline).
#[allow(dead_code)] // pure compatibility entry point used by viewport regressions
pub fn pick_world_cell(
    viewport: Option<WorldViewport>,
    cursor: PhysicalPosition<f64>,
) -> Option<(u32, u32)> {
    viewport?.cell_at(cursor)
}

/// Read-only view spec for presenting the material world (G2/G3/G4).
///
/// `temperature_buffer` / `flags_buffer` are optional presentation inputs
/// (used by the ThermalLab palette only); they are always bound read-only,
/// so the renderer can never mutate the authoritative simulation state.
pub struct WorldViewSpec<'a> {
    pub material_buffer: &'a wgpu::Buffer,
    pub temperature_buffer: Option<&'a wgpu::Buffer>,
    pub pressure_buffer: Option<&'a wgpu::Buffer>,
    pub flags_buffer: Option<&'a wgpu::Buffer>,
    /// G7-A per-chunk activity masks (presentation read-only; the Activity
    /// palette heatmap overlay). None for other palettes.
    pub chunk_activity_buffer: Option<&'a wgpu::Buffer>,
    /// Chunk edge length (used by the Activity palette; 0 otherwise).
    pub chunk_size: u32,
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
    text_renderer: Option<crate::text_renderer::TextRenderer>,
}

/// Immutable surface facts recorded by the G8-C windowed measurement worker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SurfaceInfo {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub present_mode: wgpu::PresentMode,
}

/// Stable classification for a failed G8-C measured surface acquisition.
/// Normal application rendering continues to use its existing `GpuError`
/// behavior; this type exists only so evidence can count typed frame drops.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeasurementSurfaceFailure {
    pub kind: &'static str,
    pub message: String,
    pub reconfigured: bool,
    pub fatal: bool,
}

/// Result of one explicit G8-C surface presentation attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MeasurementFrameStatus {
    Presented,
    Dropped(MeasurementSurfaceFailure),
}

/// One resolved render-pass timestamp pair. Raw ticks are retained so the
/// independent verifier can reconstruct every duration without trusting the
/// worker's floating-point conversion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderTimestampSample {
    pub start_tick: u64,
    pub end_tick: u64,
    pub duration_ms: f64,
}

/// Mode-D-only timestamp resources. Ordinary renderer construction and
/// [`Renderer::render`] never allocate this type or request TIMESTAMP_QUERY.
pub struct RenderTimestampBatch {
    query_set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
    readback_buffer: wgpu::Buffer,
    capacity: u32,
    submitted: u32,
}

/// CPU-owned RGBA8 pixels captured from the exact renderer draw path.
#[allow(dead_code)] // Used by the automated capture worker when that mode is linked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CaptureChannelOrder {
    Rgba,
    Bgra,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CaptureLayout {
    unpadded_bytes_per_row: u32,
    padded_bytes_per_row: u32,
    buffer_size: u64,
    channel_order: CaptureChannelOrder,
}

fn capture_layout(width: u32, height: u32, format: TextureFormat) -> Result<CaptureLayout, String> {
    if width == 0 || height == 0 {
        return Err(format!(
            "capture dimensions must be nonzero, got {width}x{height}"
        ));
    }
    let channel_order = match format {
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => CaptureChannelOrder::Rgba,
        TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => CaptureChannelOrder::Bgra,
        _ => return Err(format!("unsupported capture texture format: {format:?}")),
    };
    let unpadded_bytes_per_row = width
        .checked_mul(4)
        .ok_or_else(|| format!("capture row byte count overflows for width {width}"))?;
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = unpadded_bytes_per_row
        .checked_add(alignment - 1)
        .ok_or_else(|| format!("capture row alignment overflows for width {width}"))?
        / alignment
        * alignment;
    let buffer_size = u64::from(padded_bytes_per_row)
        .checked_mul(u64::from(height))
        .ok_or_else(|| format!("capture buffer size overflows for {width}x{height}"))?;
    Ok(CaptureLayout {
        unpadded_bytes_per_row,
        padded_bytes_per_row,
        buffer_size,
        channel_order,
    })
}

fn captured_bytes_to_rgba(
    mapped: &[u8],
    width: u32,
    height: u32,
    layout: CaptureLayout,
) -> Result<Vec<u8>, String> {
    let expected_mapped_len = usize::try_from(layout.buffer_size)
        .map_err(|_| "capture buffer is too large for this platform".to_string())?;
    if mapped.len() < expected_mapped_len {
        return Err(format!(
            "capture buffer is truncated: expected at least {expected_mapped_len} bytes, got {}",
            mapped.len()
        ));
    }
    let rgba_len = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(4))
        .and_then(|bytes| usize::try_from(bytes).ok())
        .ok_or_else(|| format!("capture RGBA size overflows for {width}x{height}"))?;
    let row_bytes = usize::try_from(layout.unpadded_bytes_per_row)
        .map_err(|_| "capture row is too large for this platform".to_string())?;
    let padded_row_bytes = usize::try_from(layout.padded_bytes_per_row)
        .map_err(|_| "padded capture row is too large for this platform".to_string())?;
    let mut rgba = Vec::with_capacity(rgba_len);
    for row in 0..height as usize {
        let start = row
            .checked_mul(padded_row_bytes)
            .ok_or_else(|| "capture row offset overflows".to_string())?;
        let end = start
            .checked_add(row_bytes)
            .ok_or_else(|| "capture row end overflows".to_string())?;
        let source = mapped
            .get(start..end)
            .ok_or_else(|| format!("capture row {row} is outside the mapped buffer"))?;
        match layout.channel_order {
            CaptureChannelOrder::Rgba => rgba.extend_from_slice(source),
            CaptureChannelOrder::Bgra => {
                for pixel in source.chunks_exact(4) {
                    rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
                }
            }
        }
    }
    if rgba.len() != rgba_len {
        return Err(format!(
            "capture RGBA length mismatch: expected {rgba_len}, got {}",
            rgba.len()
        ));
    }
    Ok(rgba)
}

/// Clear color for the empty G0 world frame (a dim slate blue).
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.02,
    g: 0.02,
    b: 0.05,
    a: 1.0,
};

/// Params uniform: world/surface/layout metadata + CPU camera transform (64 B).
const WORLD_VIEW_PARAMS_SIZE: u64 = 64;
/// Metrics uniform: 32 u32/f32 values = 128 B.
const METRICS_UNIFORM_SIZE: u64 = 128;

const WORLD_VIEW_SHADER: &str = r#"
struct Params {
    width: u32,
    height: u32,
    surface_w: u32,
    surface_h: u32,
    palette: u32,
    chunk_size: u32,
    chunks_x: u32,
    _pad2: u32,
    viewport_x: f32,
    viewport_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    camera_origin_x: f32,
    camera_origin_y: f32,
    camera_scale: f32,
    _pad3: f32,
};

struct Metrics {
    a_ice: u32,
    a_water: u32,
    a_steam: u32,
    a_first_melt: u32,
    a_first_steam: u32,

    b_steam: u32,
    b_water: u32,
    b_ice: u32,
    b_first_condense: u32,
    b_first_freeze: u32,

    c_w_mid_t: f32,
    c_w_top_t: f32,
    c_w_max_t: f32,
    c_o_mid_t: f32,
    c_o_top_t: f32,
    c_o_max_t: f32,

    c_w_mid_reach: u32,
    c_o_mid_reach: u32,
    c_w_top_reach: u32,
    c_o_top_reach: u32,

    d_wood_start: u32,
    d_wood_left: u32,
    d_burning: u32,
    d_first_ignite: u32,
    d_first_empty: u32,

    current_tick: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
    _pad4: u32,
    _pad5: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> materials: array<u32>;
@group(0) @binding(2) var<storage, read> temperatures: array<f32>;
@group(0) @binding(3) var<storage, read> flags: array<u32>;
@group(0) @binding(4) var<uniform> metrics: Metrics;
@group(0) @binding(5) var<storage, read> chunk_activity: array<u32>;
@group(0) @binding(6) var<storage, read> pressures: array<f32>;

const EMPTY: u32 = 0u;
const BOUNDARY: u32 = 1u;
const STONE: u32 = 2u;
const SAND: u32 = 3u;
const WATER: u32 = 4u;
const OIL: u32 = 5u;
const STEAM: u32 = 6u;
const SMOKE: u32 = 7u;
const ICE: u32 = 8u;
const WOOD: u32 = 9u;
const PALETTE_LAB: u32 = 1u;
const PALETTE_THERMAL: u32 = 2u;
const PALETTE_INTEGRITY: u32 = 3u;
const PALETTE_ACTIVITY: u32 = 4u;
const PALETTE_GALLERY: u32 = 5u;
const PALETTE_SANDBOX: u32 = 6u;
const PALETTE_THERMAL_ENVIRONMENT: u32 = 7u;

const ACT_MATTER: u32 = 1u << 0u;
const ACT_THERMAL: u32 = 1u << 1u;
const ACT_PRESSURE: u32 = 1u << 2u;
const ACT_REACTION: u32 = 1u << 3u;
const FLAG_COMBUSTING: u32 = 1u;
const FLAG_FLAME_EVENT: u32 = 2u;

// 4-Panel Observatory: Panel C (Heat Comparison) is bottom-left (x 20..140, y 100..186).
const HEAT_COMP_X0: u32 = 20u;
const HEAT_COMP_X1: u32 = 140u;
const HEAT_COMP_Y0: u32 = 100u;
const HEAT_COMP_Y1: u32 = 186u;

// 3x5 uppercase glyphs, bit = gy*3+gx. Presentation overlay only.
fn glyph_bits(code: u32) -> u32 {
    switch code {
        case 32u: { return 0u; }
        case 43u: { return 0x05D0u; } // +
        case 45u: { return 0x01C0u; } // -
        case 46u: { return 0x2000u; } // .
        case 48u: { return 0x7B6Fu; } // 0
        case 49u: { return 0x2492u; } // 1
        case 50u: { return 0x4B2Fu; } // 2
        case 51u: { return 0x79E7u; } // 3
        case 52u: { return 0x4F5Cu; } // 4
        case 53u: { return 0x6F2Fu; } // 5
        case 54u: { return 0x7BCEu; } // 6
        case 55u: { return 0x24A7u; } // 7
        case 56u: { return 0x7BEFu; } // 8
        case 57u: { return 0x79EFu; } // 9
        case 58u: { return 0x0410u; } // :
        case 62u: { return 0x18A1u; } // >
        case 65u: { return 0x5BEAu; } // A
        case 66u: { return 0x3AEBu; } // B
        case 67u: { return 0x624Eu; } // C
        case 68u: { return 0x3B6Bu; } // D
        case 69u: { return 0x72CFu; } // E
        case 70u: { return 0x13CFu; } // F
        case 71u: { return 0x6B4Eu; } // G
        case 72u: { return 0x0BE8u; } // H
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
        case 85u: { return 0x0BEDu; } // U
        case 86u: { return 0x3AEBu; } // V
        case 87u: { return 0x5F6Du; } // W
        case 88u: { return 0x5AADu; } // X
        case 89u: { return 0x24ADu; } // Y
        case 90u: { return 0x72AFu; } // Z
        case 94u: { return 0x24BAu; } // ^
        case 118u: { return 0x2E92u; } // v
        default: { return 0u; }
    }
}

// spacing: character pitch in units of `cell` (3 glyph cols + gaps).
fn text_hit(px: f32, py: f32, origin_x: f32, origin_y: f32, cell: f32, spacing: f32, codes: array<u32, 16>, n: u32) -> bool {
    let rel_x = px - origin_x;
    let rel_y = py - origin_y;
    if (rel_x < 0.0 || rel_y < 0.0) { return false; }
    let step = cell * spacing;
    let col = u32(rel_x / step);
    if (col >= n) { return false; }
    let gx = u32((rel_x % step) / cell);
    let gy = u32(rel_y / cell);
    if (gx >= 3u || gy >= 5u) { return false; }
    let bits = glyph_bits(codes[col]);
    return ((bits >> (gy * 3u + gx)) & 1u) == 1u;
}

fn centered_origin(center_x: f32, n: u32, cell: f32, spacing: f32) -> f32 {
    let text_w = f32(n) * cell * spacing - cell;
    return center_x - text_w * 0.5;
}

fn num_hit(px: f32, py: f32, origin_x: f32, origin_y: f32, cell: f32, spacing: f32, val: u32) -> bool {
    var digits = array<u32, 8>();
    var v = val;
    var count = 0u;
    if (v == 0u) {
        digits[0] = 48u;
        count = 1u;
    } else {
        var temp = array<u32, 8>();
        var t_cnt = 0u;
        while (v > 0u && t_cnt < 8u) {
            temp[t_cnt] = 48u + (v % 10u);
            v = v / 10u;
            t_cnt = t_cnt + 1u;
        }
        for (var i = 0u; i < t_cnt; i = i + 1u) {
            digits[i] = temp[t_cnt - 1u - i];
        }
        count = t_cnt;
    }
    var codes = array<u32, 16>();
    for (var i = 0u; i < count; i = i + 1u) {
        codes[i] = digits[i];
    }
    return text_hit(px, py, origin_x, origin_y, cell, spacing, codes, count);
}

fn tick_hit(px: f32, py: f32, origin_x: f32, origin_y: f32, cell: f32, spacing: f32, tick_val: u32) -> bool {
    if (tick_val == 0xFFFFFFFFu) {
        var dash = array<u32, 16>();
        dash[0] = 45u; dash[1] = 45u;
        return text_hit(px, py, origin_x, origin_y, cell, spacing, dash, 2u);
    }
    return num_hit(px, py, origin_x, origin_y, cell, spacing, tick_val);
}

fn temp_hit(px: f32, py: f32, origin_x: f32, origin_y: f32, cell: f32, spacing: f32, temp_val: f32) -> bool {
    var codes = array<u32, 16>();
    var count = 0u;
    var t = temp_val;
    if (t < 0.0) {
        codes[count] = 45u; // '-'
        count = count + 1u;
        t = -t;
    }
    let int_part = u32(clamp(t, 0.0, 999.0));
    let frac_part = u32(clamp(fract(t) * 10.0, 0.0, 9.0));
    if (int_part >= 100u) {
        codes[count] = 48u + (int_part / 100u) % 10u;
        count = count + 1u;
    }
    if (int_part >= 10u) {
        codes[count] = 48u + (int_part / 10u) % 10u;
        count = count + 1u;
    }
    codes[count] = 48u + (int_part % 10u);
    count = count + 1u;
    codes[count] = 46u; // '.'
    count = count + 1u;
    codes[count] = 48u + frac_part;
    count = count + 1u;
    return text_hit(px, py, origin_x, origin_y, cell, spacing, codes, count);
}

// Presentation-only debug palette (material IDs never change).
// Forest: Stone is green terrain/trees. Lab/ThermalLab/Integrity: Stone is neutral.
fn debug_color(id: u32, palette: u32) -> vec4<f32> {
    if (palette == PALETTE_LAB
        || palette == PALETTE_THERMAL
        || palette == PALETTE_INTEGRITY
        || palette == PALETTE_ACTIVITY
        || palette == PALETTE_GALLERY
        || palette == PALETTE_SANDBOX
        || palette == PALETTE_THERMAL_ENVIRONMENT) {
        if (id == EMPTY) { return vec4<f32>(0.05, 0.055, 0.07, 1.0); }
        if (id == BOUNDARY) { return vec4<f32>(0.22, 0.23, 0.25, 1.0); }
        if (id == STONE) { return vec4<f32>(0.46, 0.47, 0.50, 1.0); }
        if (id == SAND) { return vec4<f32>(0.96, 0.82, 0.28, 1.0); }
        if (id == WATER) { return vec4<f32>(0.12, 0.48, 0.96, 1.0); }
        if (id == OIL) { return vec4<f32>(0.66, 0.38, 0.10, 1.0); }
        if (id == STEAM) { return vec4<f32>(0.96, 0.97, 0.99, 1.0); }
        if (id == SMOKE) { return vec4<f32>(0.18, 0.18, 0.20, 1.0); }
        if (id == ICE) { return vec4<f32>(0.72, 0.92, 0.99, 1.0); }
        if (id == WOOD) { return vec4<f32>(0.45, 0.32, 0.18, 1.0); }
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
    if (id == ICE) { return vec4<f32>(0.72, 0.92, 0.99, 1.0); }
    if (id == WOOD) { return vec4<f32>(0.42, 0.30, 0.16, 1.0); }
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}

// ThermalLab base material colors (neutral lab gray for Stone; same identity
// hues as the Lab palette so the material stays recognizable under tint).
fn thermal_base(id: u32) -> vec4<f32> {
    if (id == EMPTY) { return vec4<f32>(0.04, 0.045, 0.06, 1.0); }
    if (id == BOUNDARY) { return vec4<f32>(0.20, 0.21, 0.23, 1.0); }
    if (id == STONE) { return vec4<f32>(0.44, 0.45, 0.48, 1.0); }
    if (id == SAND) { return vec4<f32>(0.88, 0.75, 0.30, 1.0); }
    if (id == WATER) { return vec4<f32>(0.12, 0.46, 0.92, 1.0); }
    if (id == OIL) { return vec4<f32>(0.60, 0.36, 0.10, 1.0); }
    if (id == STEAM) { return vec4<f32>(0.93, 0.95, 0.98, 1.0); }
    if (id == SMOKE) { return vec4<f32>(0.07, 0.07, 0.09, 1.0); }
    if (id == ICE) { return vec4<f32>(0.68, 0.90, 0.99, 1.0); }
    if (id == WOOD) { return vec4<f32>(0.46, 0.33, 0.18, 1.0); }
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}

// Diagnostic temperature ramp shared by Heat Comparison tubes and HUD legend:
// deep blue (cold) → cyan → yellow → orange → near-white (very hot).
fn heat_ramp(ramp_t: f32) -> vec4<f32> {
    var c = vec4<f32>(0.05, 0.10, 0.55, 1.0);
    c = mix(c, vec4<f32>(0.10, 0.78, 0.88, 1.0), smoothstep(0.12, 0.25, ramp_t));
    c = mix(c, vec4<f32>(1.0, 0.88, 0.18, 1.0), smoothstep(0.32, 0.48, ramp_t));
    c = mix(c, vec4<f32>(1.0, 0.45, 0.05, 1.0), smoothstep(0.55, 0.75, ramp_t));
    c = mix(c, vec4<f32>(0.98, 0.96, 0.90, 1.0), smoothstep(0.82, 1.0, ramp_t));
    return c;
}

// ThermalLab cell color (4-panel layout):
//   - Panel C (HEAT COMPARISON, bottom-left x 20..140, y 100..186): strong false-color ramp.
//   - Smoke: near-background dark gray.
//   - Wood: normal brown → hot red-brown → combusting orange/yellow.
fn thermal_lab_color(cx: u32, cy: u32, id: u32, t: f32, f: u32) -> vec4<f32> {
    let t_c = clamp(t, -100.0, 250.0);
    let heat_comp = cx >= HEAT_COMP_X0 && cx <= HEAT_COMP_X1 && cy >= HEAT_COMP_Y0 && cy <= HEAT_COMP_Y1;
    if (heat_comp && (id == WATER || id == OIL)) {
        let ramp_t = clamp(t_c / 60.0, 0.0, 1.0);
        var c = heat_ramp(ramp_t);
        if (id == WATER) { c = mix(c, vec4<f32>(0.20, 0.45, 1.0, 1.0), 0.10); }
        if (id == OIL) { c = mix(c, vec4<f32>(1.0, 0.55, 0.25, 1.0), 0.10); }
        return c;
    }

    var c = thermal_base(id);
    if (id == SMOKE) {
        return c;
    }
    let steam = id == STEAM;
    let cold = clamp(-t_c / 40.0, 0.0, 1.0);
    let warm = clamp(t_c / 90.0, 0.0, 1.0);
    var cold_s = 0.35;
    var warm_s = 0.45;
    if (steam) {
        cold_s = 0.10;
        warm_s = 0.12;
    }
    c = mix(c, vec4<f32>(0.20, 0.45, 1.0, 1.0), cold * cold_s);
    c = mix(c, vec4<f32>(1.0, 0.40, 0.12, 1.0), warm * warm_s);

    if (id == WOOD) {
        let hot_s = clamp((t_c - 40.0) / 60.0, 0.0, 1.0);
        c = mix(c, vec4<f32>(0.58, 0.26, 0.12, 1.0), hot_s * 0.85);
    }
    if (id == WOOD || id == OIL) {
        if ((f & FLAG_COMBUSTING) != 0u) {
            c = mix(c, vec4<f32>(1.0, 0.38, 0.08, 1.0), 0.70);
        }
        if ((f & FLAG_FLAME_EVENT) != 0u) {
            c = mix(c, vec4<f32>(1.0, 0.85, 0.20, 1.0), 0.45);
        }
    }
    return c;
}

// G8-B Gallery presentation is deliberately coordinate-independent. Every
// tint comes from the authoritative per-cell fields bound above; no expected
// outcome or fixture coordinate is encoded in this renderer.
fn gallery_color(id: u32, t: f32, p: f32, f: u32) -> vec4<f32> {
    var c = debug_color(id, PALETTE_GALLERY);
    if (id == EMPTY || id == BOUNDARY) {
        return c;
    }

    let cold = clamp(-t / 50.0, 0.0, 1.0);
    let hot = clamp(t / 180.0, 0.0, 1.0);
    c = mix(c, vec4<f32>(0.18, 0.42, 1.0, 1.0), cold * 0.32);
    c = mix(c, vec4<f32>(1.0, 0.34, 0.10, 1.0), hot * 0.42);

    if (id == WATER || id == OIL || id == STEAM || id == SMOKE) {
        let pressure = clamp(max(p, 0.0) / 20.0, 0.0, 1.0);
        c = mix(c, vec4<f32>(0.75, 0.28, 1.0, 1.0), pressure * 0.55);
    }
    if ((f & FLAG_COMBUSTING) != 0u) {
        c = mix(c, vec4<f32>(1.0, 0.30, 0.04, 1.0), 0.75);
    }
    if ((f & FLAG_FLAME_EVENT) != 0u) {
        c = mix(c, vec4<f32>(1.0, 0.90, 0.20, 1.0), 0.55);
    }
    return c;
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

// G7-A activity heatmap overlay: one dominant presentation color per chunk
// (priority REACTION > PRESSURE > THERMAL > MATTER); zero-activity chunks
// are dimmed. Presentation-only — simulation truth is never altered.
fn activity_overlay(cell_x: u32, cell_y: u32, base: vec4<f32>) -> vec4<f32> {
    let chunk = (cell_y / params.chunk_size) * params.chunks_x + (cell_x / params.chunk_size);
    let mask = chunk_activity[chunk];
    if (mask == 0u) {
        return base * vec4<f32>(0.52, 0.52, 0.58, 1.0);
    }
    var col = base;
    if ((mask & ACT_REACTION) != 0u) {
        col = mix(col, vec4<f32>(1.0, 0.22, 0.22, 1.0), 0.55);
    } else if ((mask & ACT_PRESSURE) != 0u) {
        col = mix(col, vec4<f32>(0.25, 0.5, 1.0, 1.0), 0.55);
    } else if ((mask & ACT_THERMAL) != 0u) {
        col = mix(col, vec4<f32>(1.0, 0.55, 0.12, 1.0), 0.55);
    } else {
        col = mix(col, vec4<f32>(0.3, 0.85, 0.35, 1.0), 0.55);
    }
    return col;
}

@fragment
fn fs_main(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let fw = f32(params.surface_w);
    let fh = f32(params.surface_h);
    let ww = f32(params.width);
    let wh = f32(params.height);
    let lab = params.palette == PALETTE_LAB;
    let thermal = params.palette == PALETTE_THERMAL
        || params.palette == PALETTE_THERMAL_ENVIRONMENT;
    let integrity = params.palette == PALETTE_INTEGRITY;
    let activity = params.palette == PALETTE_ACTIVITY;
    let gallery = params.palette == PALETTE_GALLERY;
    let sandbox = params.palette == PALETTE_SANDBOX;
    let scale = params.camera_scale;
    let off_x = params.viewport_x;
    let off_y = params.viewport_y;
    let px = frag.x;
    let py = frag.y;
    let in_viewport = px >= off_x && px < off_x + params.viewport_width
                   && py >= off_y && py < off_y + params.viewport_height;
    if (in_viewport) {
        let cell_x = min(u32(params.camera_origin_x + (px - off_x) / scale), params.width - 1u);
        let cell_y = min(u32(params.camera_origin_y + (py - off_y) / scale), params.height - 1u);
        let idx = cell_y * params.width + cell_x;
        if (thermal) {
            return thermal_lab_color(
                cell_x, cell_y, materials[idx], temperatures[idx], flags[idx]
            );
        }
        if (gallery) {
            return gallery_color(
                materials[idx], temperatures[idx], pressures[idx], flags[idx]
            );
        }
        let base = debug_color(materials[idx], params.palette);
        if (activity) {
            return activity_overlay(cell_x, cell_y, base);
        }
        return base;
    }
    if (lab) {
        let hud = lab_hud(px, py, fw, fh, off_x, off_y, scale);
        if (hud.a > 0.0) {
            return hud;
        }
        return vec4<f32>(0.09, 0.10, 0.12, 1.0);
    }
    if (thermal) {
        let hud = thermal_lab_hud(px, py, fw, fh, off_x, off_y, scale);
        if (hud.a > 0.0) {
            return hud;
        }
        let border_t = 1.0;
        let on_border = (px >= off_x - border_t && px <= off_x + ww * scale + border_t &&
                         py >= off_y - border_t && py <= off_y + wh * scale + border_t) &&
                        (px < off_x || px > off_x + ww * scale || py < off_y || py > off_y + wh * scale);
        if (on_border) {
            return vec4<f32>(0.24, 0.28, 0.38, 1.0);
        }
        return vec4<f32>(0.07, 0.08, 0.11, 1.0);
    }
    if (integrity || gallery || sandbox) {
        // No procedural G3 HUD here — the G6 HUD is the screen-space text
        // renderer. Just a crisp viewport border over the dark lab backdrop.
        let border_t = 1.0;
        let on_border = (px >= off_x - border_t && px <= off_x + ww * scale + border_t &&
                         py >= off_y - border_t && py <= off_y + wh * scale + border_t) &&
                        (px < off_x || px > off_x + ww * scale || py < off_y || py > off_y + wh * scale);
        if (on_border) {
            return vec4<f32>(0.24, 0.28, 0.38, 1.0);
        }
        return vec4<f32>(0.07, 0.08, 0.11, 1.0);
    }
    return vec4<f32>(0.06, 0.07, 0.10, 1.0);
}

// G3 density-lab HUD (unchanged — approved fixture).
fn lab_hud(px: f32, py: f32, fw: f32, fh: f32, off_x: f32, off_y: f32, scale: f32) -> vec4<f32> {
    let cell = max(2.0, min(3.0, scale * 0.55));
    let spacing = 4.0;
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
    if (text_hit(px, py, centered_origin(fw * 0.5, 15u, cell, spacing), title_y, cell, spacing, title, 15u)) {
        return ink;
    }

    var sw = array<u32, 16>();
    sw[0] = 83u; sw[1] = 65u; sw[2] = 78u; sw[3] = 68u; sw[4] = 32u;
    sw[5] = 43u; sw[6] = 32u; sw[7] = 87u; sw[8] = 65u; sw[9] = 84u;
    sw[10] = 69u; sw[11] = 82u;
    if (text_hit(px, py, centered_origin(c1, 12u, cell, spacing), label_y, cell, spacing, sw, 12u)) {
        return ink;
    }
    var wo = array<u32, 16>();
    wo[0] = 87u; wo[1] = 65u; wo[2] = 84u; wo[3] = 69u; wo[4] = 82u;
    wo[5] = 32u; wo[6] = 43u; wo[7] = 32u; wo[8] = 79u; wo[9] = 73u;
    wo[10] = 76u;
    if (text_hit(px, py, centered_origin(c2, 11u, cell, spacing), label_y, cell, spacing, wo, 11u)) {
        return ink;
    }
    var ss = array<u32, 16>();
    ss[0] = 83u; ss[1] = 84u; ss[2] = 69u; ss[3] = 65u; ss[4] = 77u;
    ss[5] = 32u; ss[6] = 43u; ss[7] = 32u; ss[8] = 83u; ss[9] = 77u;
    ss[10] = 79u; ss[11] = 75u; ss[12] = 69u;
    if (text_hit(px, py, centered_origin(c3, 13u, cell, spacing), label_y, cell, spacing, ss, 13u)) {
        return ink;
    }

    var a0 = array<u32, 16>();
    a0[0] = 83u; a0[1] = 65u; a0[2] = 78u; a0[3] = 68u; a0[4] = 32u;
    a0[5] = 83u; a0[6] = 73u; a0[7] = 78u; a0[8] = 75u; a0[9] = 83u;
    a0[10] = 32u; a0[11] = 118u;
    if (text_hit(px, py, centered_origin(c1, 12u, cell, spacing), cap0, cell, spacing, a0, 12u)) {
        return sand;
    }
    var a1 = array<u32, 16>();
    a1[0] = 87u; a1[1] = 65u; a1[2] = 84u; a1[3] = 69u; a1[4] = 82u;
    a1[5] = 32u; a1[6] = 82u; a1[7] = 73u; a1[8] = 83u; a1[9] = 69u;
    a1[10] = 83u; a1[11] = 32u; a1[12] = 94u;
    if (text_hit(px, py, centered_origin(c1, 13u, cell, spacing), cap1, cell, spacing, a1, 13u)) {
        return water;
    }
    var b0 = array<u32, 16>();
    b0[0] = 87u; b0[1] = 65u; b0[2] = 84u; b0[3] = 69u; b0[4] = 82u;
    b0[5] = 32u; b0[6] = 83u; b0[7] = 73u; b0[8] = 78u; b0[9] = 75u;
    b0[10] = 83u; b0[11] = 32u; b0[12] = 118u;
    if (text_hit(px, py, centered_origin(c2, 13u, cell, spacing), cap0, cell, spacing, b0, 13u)) {
        return water;
    }
    var b1 = array<u32, 16>();
    b1[0] = 79u; b1[1] = 73u; b1[2] = 76u; b1[3] = 32u; b1[4] = 82u;
    b1[5] = 73u; b1[6] = 83u; b1[7] = 69u; b1[8] = 83u; b1[9] = 32u;
    b1[10] = 94u;
    if (text_hit(px, py, centered_origin(c2, 11u, cell, spacing), cap1, cell, spacing, b1, 11u)) {
        return oil;
    }
    var c0 = array<u32, 16>();
    c0[0] = 83u; c0[1] = 84u; c0[2] = 69u; c0[3] = 65u; c0[4] = 77u;
    c0[5] = 32u; c0[6] = 82u; c0[7] = 73u; c0[8] = 83u; c0[9] = 69u;
    c0[10] = 83u; c0[11] = 32u; c0[12] = 94u;
    if (text_hit(px, py, centered_origin(c3, 13u, cell, spacing), cap0, cell, spacing, c0, 13u)) {
        return steam;
    }
    var c1t = array<u32, 16>();
    c1t[0] = 83u; c1t[1] = 77u; c1t[2] = 79u; c1t[3] = 75u; c1t[4] = 69u;
    c1t[5] = 32u; c1t[6] = 83u; c1t[7] = 73u; c1t[8] = 78u; c1t[9] = 75u;
    c1t[10] = 83u; c1t[11] = 32u; c1t[12] = 118u;
    if (text_hit(px, py, centered_origin(c3, 13u, cell, spacing), cap1, cell, spacing, c1t, 13u)) {
        return smoke;
    }
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

// G4 4-Panel Large Thermal Observatory:
// General HUD text and diagnostic metrics are rendered in a dedicated screen-space
// high-resolution text pass (TextRenderer) to ensure maximum legibility and zero viewport clutter.
fn thermal_lab_hud(px: f32, py: f32, fw: f32, fh: f32, off_x: f32, off_y: f32, scale: f32) -> vec4<f32> {
    // Temperature Color Ramp Legend at Window Bottom Center
    let legend_w = 260.0;
    let legend_x0 = fw * 0.5 - legend_w * 0.5;
    let legend_y = fh - 40.0;
    if (py >= legend_y - 4.0 && py <= legend_y + 4.0 && px >= legend_x0 && px <= legend_x0 + legend_w) {
        let lt = clamp((px - legend_x0) / legend_w, 0.0, 1.0);
        return heat_ramp(lt);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

"#;

impl Renderer {
    /// Creates a surface for `window` on the given instance/adapter/device.
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

        let needs_text_hud = world_view
            .as_ref()
            .map(|s| {
                matches!(
                    s.palette,
                    PresentationPalette::ThermalLab
                        | PresentationPalette::ThermalEnvironment
                        | PresentationPalette::Integrity
                        | PresentationPalette::Activity
                        | PresentationPalette::Gallery
                        | PresentationPalette::Sandbox
                )
            })
            .unwrap_or(false);
        let text_renderer = if needs_text_hud {
            Some(crate::text_renderer::TextRenderer::new(
                device, queue, format,
            )?)
        } else {
            None
        };

        let world_view =
            world_view.map(|spec| build_world_view(device, queue, format, &config, spec));

        Ok(Self {
            surface,
            config,
            device: device.clone(),
            queue: queue.clone(),
            world_view,
            text_renderer,
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

    /// Returns the world rectangle currently sent to the shader, in physical
    /// surface pixels. Runtime-baseline mode has no world viewport.
    pub fn world_viewport(&self) -> Option<WorldViewport> {
        Some(self.world_transform()?.viewport)
    }

    /// Returns the exact camera-aware transform currently sent to the shader.
    pub fn world_transform(&self) -> Option<WorldTransform> {
        let world_view = self.world_view.as_ref()?;
        let viewport = WorldViewport::calculate(
            self.config.width,
            self.config.height,
            world_view.world_width,
            world_view.world_height,
            world_view.palette,
        )?;
        Some(WorldTransform::calculate(viewport, world_view.camera))
    }

    /// Picks a top-left-origin world cell from a physical cursor position.
    pub fn world_cell_at(&self, cursor: PhysicalPosition<f64>) -> Option<(u32, u32)> {
        self.world_transform()?.cell_at(cursor)
    }

    pub fn reset_world_camera(&mut self) {
        let Some(world_view) = &mut self.world_view else {
            return;
        };
        world_view.camera = WorldCamera::fitted(world_view.world_width, world_view.world_height);
        write_world_view_params(&self.queue, world_view, &self.config);
    }

    /// Pans the world with a physical-pixel drag. Positive pointer movement
    /// moves the presented world with the pointer.
    pub fn pan_world_camera(&mut self, delta_x: f32, delta_y: f32) {
        if !delta_x.is_finite() || !delta_y.is_finite() {
            return;
        }
        let Some(world_view) = &mut self.world_view else {
            return;
        };
        let Some(viewport) = WorldViewport::calculate(
            self.config.width,
            self.config.height,
            world_view.world_width,
            world_view.world_height,
            world_view.palette,
        ) else {
            return;
        };
        world_view.camera = world_view
            .camera
            .panned_by_pixels(viewport, delta_x, delta_y);
        write_world_view_params(&self.queue, world_view, &self.config);
    }

    /// Cursor-anchored zoom using the same transform as shader and picking.
    pub fn zoom_world_camera_at(&mut self, cursor: PhysicalPosition<f64>, factor: f32) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        let Some(world_view) = &mut self.world_view else {
            return;
        };
        let Some(viewport) = WorldViewport::calculate(
            self.config.width,
            self.config.height,
            world_view.world_width,
            world_view.world_height,
            world_view.palette,
        ) else {
            return;
        };
        world_view.camera = world_view.camera.zoomed_at(viewport, cursor, factor);
        write_world_view_params(&self.queue, world_view, &self.config);
    }

    /// Updates the live observatory metrics uniform buffer for HUD display.
    #[allow(dead_code)]
    pub fn update_metrics(&self, metrics: &crate::observatory::MetricsUniform) {
        if let Some(wv) = &self.world_view {
            self.queue
                .write_buffer(&wv.metrics_buf, 0, &metrics.to_bytes());
        }
    }
}

/// Live HUD overlay data dispatched to the vector text renderer.
pub enum HudData<'a> {
    Thermal(&'a crate::observatory::ObservatoryMetrics, u64),
    ThermalEnvironment(&'a crate::thermal_environment::ThermalEnvironmentHudData),
    PhaseCycle(&'a crate::phase_cycle::PhaseCycleHudData),
    IgnitionKinetics(&'a crate::ignition_kinetics::IgnitionHudData),
    Pressure(&'a crate::observatory::PressureObservatoryMetrics, u64),
    ParallelIntegrity(&'a crate::observatory::IntegrityMetrics, u64),
    Activity(&'a crate::observatory::ActivityMetrics, u64),
    Gallery(&'a crate::gallery::GalleryHudData),
    Sandbox(&'a crate::sandbox::SandboxHudData),
}

impl Renderer {
    /// Acquires the next surface frame, draws the world view (or clear) + text HUD,
    /// and presents.
    pub fn render(&mut self, hud_data: Option<HudData<'_>>) -> Result<(), GpuError> {
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
        self.encode_frame(&mut encoder, &view, hud_data, None);

        self.queue.submit([encoder.finish()]);
        frame.present();
        Ok(())
    }

    /// Presents one HUD-free G8-C measurement frame while retaining the
    /// exact surface-acquisition failure class. Lost/outdated surfaces are
    /// reconfigured for the next attempt; the failed attempt remains a
    /// counted drop and is never reported as presented.
    pub fn render_measurement(&mut self) -> MeasurementFrameStatus {
        let frame = match self.acquire_measurement_frame() {
            Ok(frame) => frame,
            Err(error) => return MeasurementFrameStatus::Dropped(error),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("powdergame-g8c-coexistence-render-encoder"),
            });
        self.encode_frame(&mut encoder, &view, None, None);
        self.queue.submit([encoder.finish()]);
        frame.present();
        MeasurementFrameStatus::Presented
    }

    /// Allocates one bounded render timestamp window. This is intentionally an
    /// explicit diagnostic API: it succeeds only when the caller created the
    /// device with TIMESTAMP_QUERY, and it has no effect on ordinary renders.
    pub fn begin_render_timestamp_batch(
        &self,
        frame_capacity: u32,
    ) -> Result<RenderTimestampBatch, GpuError> {
        if frame_capacity == 0 {
            return Err(GpuError::Other(
                "render timestamp batch capacity must be greater than zero".into(),
            ));
        }
        if !self
            .device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY)
        {
            return Err(GpuError::FeatureNotSupported("TIMESTAMP_QUERY".into()));
        }
        let query_count = frame_capacity.checked_mul(2).ok_or_else(|| {
            GpuError::Other(format!(
                "render timestamp query count overflows for {frame_capacity} frames"
            ))
        })?;
        let byte_size = u64::from(query_count) * 8;
        let query_set = self.device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("powdergame-g8c-render/query-set"),
            ty: wgpu::QueryType::Timestamp,
            count: query_count,
        });
        let resolve_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("powdergame-g8c-render/resolve-buffer"),
            size: byte_size,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("powdergame-g8c-render/readback-buffer"),
            size: byte_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Ok(RenderTimestampBatch {
            query_set,
            resolve_buffer,
            readback_buffer,
            capacity: frame_capacity,
            submitted: 0,
        })
    }

    /// Presents one HUD-free frame with a begin/end timestamp on the render
    /// pass. Query resolve and CPU mapping are deliberately deferred to
    /// [`Renderer::finish_render_timestamp_batch`].
    pub fn render_timestamped(
        &mut self,
        batch: &mut RenderTimestampBatch,
    ) -> Result<MeasurementFrameStatus, GpuError> {
        if batch.submitted >= batch.capacity {
            return Err(GpuError::Other(format!(
                "render timestamp batch capacity {} exhausted",
                batch.capacity
            )));
        }
        let frame = match self.acquire_measurement_frame() {
            Ok(frame) => frame,
            Err(error) => return Ok(MeasurementFrameStatus::Dropped(error)),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("powdergame-g8c-render-encoder"),
            });
        let first_query = batch.submitted * 2;
        self.encode_frame(
            &mut encoder,
            &view,
            None,
            Some((&batch.query_set, first_query)),
        );
        self.queue.submit([encoder.finish()]);
        frame.present();
        batch.submitted += 1;
        Ok(MeasurementFrameStatus::Presented)
    }

    /// Resolves and maps the complete timestamp window once, after every
    /// measured frame has been submitted.
    pub fn finish_render_timestamp_batch(
        &self,
        batch: RenderTimestampBatch,
        timestamp_period_ns: f32,
    ) -> Result<Vec<RenderTimestampSample>, GpuError> {
        if batch.submitted == 0 {
            return Err(GpuError::ReadbackFailed(
                "cannot resolve an empty render timestamp batch".into(),
            ));
        }
        if !timestamp_period_ns.is_finite() || timestamp_period_ns <= 0.0 {
            return Err(GpuError::ReadbackFailed(format!(
                "invalid render timestamp period {timestamp_period_ns:?}"
            )));
        }
        let query_count = batch.submitted * 2;
        let byte_size = u64::from(query_count) * 8;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("powdergame-g8c-render-resolve-encoder"),
            });
        encoder.resolve_query_set(&batch.query_set, 0..query_count, &batch.resolve_buffer, 0);
        encoder.copy_buffer_to_buffer(
            &batch.resolve_buffer,
            0,
            &batch.readback_buffer,
            0,
            byte_size,
        );
        self.queue.submit([encoder.finish()]);

        let slice = batch.readback_buffer.slice(..byte_size);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device.poll(wgpu::PollType::Wait).map_err(|error| {
            GpuError::ReadbackFailed(format!("render timestamp GPU wait failed: {error}"))
        })?;
        receiver
            .recv()
            .map_err(|error| {
                GpuError::ReadbackFailed(format!("render timestamp map callback lost: {error}"))
            })?
            .map_err(|error| GpuError::ReadbackFailed(error.to_string()))?;

        let mapped = slice.get_mapped_range();
        let raw = mapped
            .chunks_exact(8)
            .take(query_count as usize)
            .map(|bytes| u64::from_ne_bytes(bytes.try_into().expect("eight-byte timestamp")))
            .collect::<Vec<_>>();
        drop(mapped);
        batch.readback_buffer.unmap();

        timestamp_samples_from_raw(&raw, timestamp_period_ns).map_err(GpuError::ReadbackFailed)
    }

    /// Draws the complete renderer frame into an offscreen texture and reads
    /// it back as tightly packed, top-to-bottom RGBA8 pixels. The surface
    /// configuration and usage remain unchanged.
    #[allow(dead_code)] // Public integration point for the capture worker.
    pub fn capture_full_frame(
        &mut self,
        hud_data: Option<HudData<'_>>,
    ) -> Result<CapturedFrame, GpuError> {
        let width = self.config.width;
        let height = self.config.height;
        let format = self.config.format;
        let layout = capture_layout(width, height, format).map_err(GpuError::ReadbackFailed)?;
        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("powdergame-full-frame-capture-texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("powdergame-full-frame-capture-readback"),
            size: layout.buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("powdergame-full-frame-capture-encoder"),
            });
        self.encode_frame(&mut encoder, &view, hud_data, None);
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(layout.padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            extent,
        );
        self.queue.submit([encoder.finish()]);

        let slice = readback.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device.poll(wgpu::PollType::Wait).map_err(|error| {
            GpuError::ReadbackFailed(format!("capture GPU wait failed: {error}"))
        })?;
        receiver
            .recv()
            .map_err(|error| {
                GpuError::ReadbackFailed(format!("capture map callback lost: {error}"))
            })?
            .map_err(|error| GpuError::ReadbackFailed(error.to_string()))?;

        let mapped = slice.get_mapped_range();
        let rgba_result = captured_bytes_to_rgba(&mapped, width, height, layout);
        drop(mapped);
        readback.unmap();
        let rgba = rgba_result.map_err(GpuError::ReadbackFailed)?;
        Ok(CapturedFrame {
            width,
            height,
            rgba,
        })
    }

    /// Encodes the one authoritative presentation draw body shared by the
    /// window surface and offscreen full-frame capture.
    fn encode_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        hud_data: Option<HudData<'_>>,
        timestamp: Option<(&wgpu::QuerySet, u32)>,
    ) {
        let timestamp_writes =
            timestamp.map(|(query_set, first)| wgpu::RenderPassTimestampWrites {
                query_set,
                beginning_of_pass_write_index: Some(first),
                end_of_pass_write_index: Some(first + 1),
            });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("powdergame-present-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes,
            occlusion_query_set: None,
        });
        if let Some(wv) = &self.world_view {
            render_pass.set_pipeline(&wv.pipeline);
            render_pass.set_bind_group(0, &wv.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
        if let (Some(tr), Some(hud)) = (&mut self.text_renderer, hud_data) {
            match hud {
                HudData::Thermal(metrics, sim_ticks) => {
                    tr.render_thermal_hud(
                        &self.device,
                        &self.queue,
                        &mut render_pass,
                        self.config.width,
                        self.config.height,
                        metrics,
                        sim_ticks,
                    );
                }
                HudData::ThermalEnvironment(data) => {
                    tr.render_thermal_environment_hud(
                        &self.device,
                        &self.queue,
                        &mut render_pass,
                        self.config.width,
                        self.config.height,
                        data,
                    );
                }
                HudData::PhaseCycle(data) => {
                    tr.render_phase_cycle_hud(
                        &self.device,
                        &self.queue,
                        &mut render_pass,
                        self.config.width,
                        self.config.height,
                        data,
                    );
                }
                HudData::IgnitionKinetics(data) => {
                    tr.render_ignition_kinetics_hud(
                        &self.device,
                        &self.queue,
                        &mut render_pass,
                        self.config.width,
                        self.config.height,
                        data,
                    );
                }
                HudData::Pressure(metrics, sim_ticks) => {
                    tr.render_pressure_hud(
                        &self.device,
                        &self.queue,
                        &mut render_pass,
                        self.config.width,
                        self.config.height,
                        metrics,
                        sim_ticks,
                    );
                }
                HudData::ParallelIntegrity(metrics, sim_ticks) => {
                    tr.render_parallel_integrity_hud(
                        &self.device,
                        &self.queue,
                        &mut render_pass,
                        self.config.width,
                        self.config.height,
                        metrics,
                        sim_ticks,
                    );
                }
                HudData::Activity(metrics, sim_ticks) => {
                    tr.render_activity_hud(
                        &self.device,
                        &self.queue,
                        &mut render_pass,
                        self.config.width,
                        self.config.height,
                        metrics,
                        sim_ticks,
                    );
                }
                HudData::Gallery(data) => {
                    tr.render_gallery_hud(
                        &self.device,
                        &self.queue,
                        &mut render_pass,
                        self.config.width,
                        self.config.height,
                        data,
                    );
                }
                HudData::Sandbox(data) => {
                    tr.render_sandbox_hud(
                        &self.device,
                        &self.queue,
                        &mut render_pass,
                        self.config.width,
                        self.config.height,
                        data,
                    );
                }
            }
        }
    }

    /// The surface format in use (useful for diagnostics).
    pub fn format(&self) -> TextureFormat {
        self.config.format
    }

    /// Returns the actual configured surface contract used by a frame.
    pub fn surface_info(&self) -> SurfaceInfo {
        SurfaceInfo {
            width: self.config.width,
            height: self.config.height,
            format: self.config.format,
            present_mode: self.config.present_mode,
        }
    }

    fn acquire_measurement_frame(
        &mut self,
    ) -> Result<wgpu::SurfaceTexture, MeasurementSurfaceFailure> {
        self.surface.get_current_texture().map_err(|error| {
            let (kind, reconfigure, fatal) = classify_measurement_surface_error(&error);
            if reconfigure {
                self.surface.configure(&self.device, &self.config);
            }
            MeasurementSurfaceFailure {
                kind,
                message: error.to_string(),
                reconfigured: reconfigure,
                fatal,
            }
        })
    }
}

fn classify_measurement_surface_error(error: &wgpu::SurfaceError) -> (&'static str, bool, bool) {
    match error {
        wgpu::SurfaceError::Timeout => ("timeout", false, false),
        wgpu::SurfaceError::Outdated => ("outdated", true, false),
        wgpu::SurfaceError::Lost => ("lost", true, false),
        wgpu::SurfaceError::OutOfMemory => ("out_of_memory", false, true),
        wgpu::SurfaceError::Other => ("other", false, false),
    }
}

fn timestamp_samples_from_raw(
    raw: &[u64],
    timestamp_period_ns: f32,
) -> Result<Vec<RenderTimestampSample>, String> {
    if raw.is_empty() || !raw.len().is_multiple_of(2) {
        return Err(format!(
            "render timestamps require a non-empty even-length sequence, got {} values",
            raw.len()
        ));
    }
    if !timestamp_period_ns.is_finite() || timestamp_period_ns <= 0.0 {
        return Err(format!(
            "invalid render timestamp period {timestamp_period_ns:?}"
        ));
    }
    let mut samples = Vec::with_capacity(raw.len() / 2);
    let mut previous_end = None;
    for (frame, pair) in raw.chunks_exact(2).enumerate() {
        let start_tick = pair[0];
        let end_tick = pair[1];
        if end_tick <= start_tick {
            return Err(format!(
                "render frame {frame} timestamp order invalid: {start_tick}..{end_tick}"
            ));
        }
        if previous_end.is_some_and(|end| start_tick < end) {
            return Err(format!(
                "render frame {frame} starts at {start_tick} before prior end {}",
                previous_end.expect("checked")
            ));
        }
        samples.push(RenderTimestampSample {
            start_tick,
            end_tick,
            duration_ms: (end_tick - start_tick) as f64 * f64::from(timestamp_period_ns)
                / 1_000_000.0,
        });
        previous_end = Some(end_tick);
    }
    Ok(samples)
}

struct WorldView {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    params: wgpu::Buffer,
    #[allow(dead_code)]
    metrics_buf: wgpu::Buffer,
    /// Real or zeroed-dummy chunk-activity buffer (binding 5 is always bound).
    #[allow(dead_code)]
    chunk_activity: wgpu::Buffer,
    world_width: u32,
    world_height: u32,
    chunk_size: u32,
    palette: PresentationPalette,
    camera: WorldCamera,
}

/// Builds the read-only world-view pipeline + bind group.
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

    let storage = wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Storage { read_only: true },
        has_dynamic_offset: false,
        min_binding_size: None,
    };
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
                ty: storage,
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: storage,
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: storage,
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(METRICS_UNIFORM_SIZE),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: storage,
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: storage,
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

    let metrics_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("powdergame-world-view-metrics"),
        size: METRICS_UNIFORM_SIZE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Binding 5 is always bound; the Activity palette provides the real
    // per-chunk buffer, every other palette gets a tiny zeroed dummy (never
    // read because the activity branch is palette-gated).
    let chunk_activity = spec.chunk_activity_buffer.map_or_else(
        || {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("powdergame-world-view-dummy-chunk-activity"),
                contents: &[0u8; 4],
                usage: wgpu::BufferUsages::STORAGE,
            })
        },
        |b| b.clone(),
    );

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
            wgpu::BindGroupEntry {
                binding: 2,
                resource: spec
                    .temperature_buffer
                    .unwrap_or(spec.material_buffer)
                    .as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: spec
                    .flags_buffer
                    .unwrap_or(spec.material_buffer)
                    .as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: metrics_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: chunk_activity.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: spec
                    .pressure_buffer
                    .unwrap_or(spec.material_buffer)
                    .as_entire_binding(),
            },
        ],
    });

    let world_view = WorldView {
        pipeline,
        bind_group,
        params,
        metrics_buf,
        chunk_activity,
        world_width: spec.width,
        world_height: spec.height,
        chunk_size: spec.chunk_size,
        palette: spec.palette,
        camera: WorldCamera::fitted(spec.width, spec.height),
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
    let viewport = WorldViewport::calculate(
        config.width,
        config.height,
        wv.world_width,
        wv.world_height,
        wv.palette,
    )
    .expect("configured surface and WorldConfig dimensions are non-zero");
    let transform = WorldTransform::calculate(viewport, wv.camera);
    let mut data = [0u8; WORLD_VIEW_PARAMS_SIZE as usize];
    data[0..4].copy_from_slice(&wv.world_width.to_ne_bytes());
    data[4..8].copy_from_slice(&wv.world_height.to_ne_bytes());
    data[8..12].copy_from_slice(&config.width.to_ne_bytes());
    data[12..16].copy_from_slice(&config.height.to_ne_bytes());
    data[16..20].copy_from_slice(&(wv.palette as u32).to_ne_bytes());
    data[20..24].copy_from_slice(&wv.chunk_size.to_ne_bytes());
    let chunks_x = if wv.chunk_size == 0 {
        0
    } else {
        wv.world_width.div_ceil(wv.chunk_size)
    };
    data[24..28].copy_from_slice(&chunks_x.to_ne_bytes());
    data[32..36].copy_from_slice(&viewport.x.to_ne_bytes());
    data[36..40].copy_from_slice(&viewport.y.to_ne_bytes());
    data[40..44].copy_from_slice(&viewport.width.to_ne_bytes());
    data[44..48].copy_from_slice(&viewport.height.to_ne_bytes());
    data[48..52].copy_from_slice(&transform.origin_x.to_ne_bytes());
    data[52..56].copy_from_slice(&transform.origin_y.to_ne_bytes());
    data[56..60].copy_from_slice(&transform.scale.to_ne_bytes());
    queue.write_buffer(&wv.params, 0, &data);
}

#[cfg(test)]
mod viewport_tests {
    use super::*;

    fn cell_center(viewport: WorldViewport, x: u32, y: u32) -> PhysicalPosition<f64> {
        PhysicalPosition::new(
            f64::from(viewport.x) + (f64::from(x) + 0.5) * f64::from(viewport.scale),
            f64::from(viewport.y) + (f64::from(y) + 0.5) * f64::from(viewport.scale),
        )
    }

    #[test]
    fn viewport_preserves_existing_layouts_and_reserves_te2_diagnostic_cards() {
        let forest =
            WorldViewport::calculate(1280, 720, 128, 128, PresentationPalette::Forest).unwrap();
        assert_eq!((forest.x, forest.y), (280.0, 0.0));
        assert_eq!(
            (forest.width, forest.height, forest.scale),
            (720.0, 720.0, 5.625)
        );

        let gallery =
            WorldViewport::calculate(1600, 900, 256, 256, PresentationPalette::Gallery).unwrap();
        assert_eq!((gallery.x, gallery.y), (420.0, 60.0));
        assert_eq!(
            (gallery.width, gallery.height, gallery.scale),
            (760.0, 760.0, 2.96875)
        );

        let thermal =
            WorldViewport::calculate(1600, 900, 320, 192, PresentationPalette::ThermalLab).unwrap();
        assert_eq!((thermal.x, thermal.y), (270.0, 127.0));
        assert_eq!(
            (thermal.width, thermal.height, thermal.scale),
            (1060.0, 636.0, 3.3125)
        );

        let te2 =
            WorldViewport::calculate(1600, 900, 256, 192, PresentationPalette::ThermalEnvironment)
                .unwrap();
        assert_eq!((te2.x, te2.y), (370.0, 122.5));
        assert_eq!((te2.width, te2.height, te2.scale), (860.0, 645.0, 3.359375));
    }

    #[test]
    fn wide_and_tall_letterboxes_reject_every_outside_side() {
        let wide =
            WorldViewport::calculate(1280, 720, 128, 128, PresentationPalette::Forest).unwrap();
        assert_eq!(
            wide.cell_at(PhysicalPosition::new(280.0, 0.0)),
            Some((0, 0))
        );
        assert_eq!(wide.cell_at(PhysicalPosition::new(279.99, 360.0)), None);
        assert_eq!(wide.cell_at(PhysicalPosition::new(1000.0, 360.0)), None);

        let tall =
            WorldViewport::calculate(720, 1280, 128, 128, PresentationPalette::Forest).unwrap();
        assert_eq!((tall.x, tall.y), (0.0, 280.0));
        assert_eq!(
            tall.cell_at(PhysicalPosition::new(0.0, 280.0)),
            Some((0, 0))
        );
        assert_eq!(tall.cell_at(PhysicalPosition::new(360.0, 279.99)), None);
        assert_eq!(tall.cell_at(PhysicalPosition::new(360.0, 1000.0)), None);
    }

    #[test]
    fn left_top_are_inclusive_and_right_bottom_are_exclusive() {
        let viewport =
            WorldViewport::calculate(1600, 900, 256, 256, PresentationPalette::Gallery).unwrap();
        assert_eq!(
            viewport.cell_at(PhysicalPosition::new(
                f64::from(viewport.x),
                f64::from(viewport.y)
            )),
            Some((0, 0))
        );
        assert_eq!(
            viewport.cell_at(cell_center(viewport, 255, 255)),
            Some((255, 255))
        );
        assert_eq!(
            viewport.cell_at(PhysicalPosition::new(
                f64::from(viewport.right()),
                f64::from(viewport.y)
            )),
            None
        );
        assert_eq!(
            viewport.cell_at(PhysicalPosition::new(
                f64::from(viewport.x),
                f64::from(viewport.bottom())
            )),
            None
        );
    }

    #[test]
    fn y_axis_increases_down_like_fragment_indexing() {
        let viewport =
            WorldViewport::calculate(1600, 900, 256, 256, PresentationPalette::Gallery).unwrap();
        assert_eq!(
            viewport.cell_at(cell_center(viewport, 31, 0)),
            Some((31, 0))
        );
        assert_eq!(
            viewport.cell_at(cell_center(viewport, 31, 127)),
            Some((31, 127))
        );
        assert_eq!(
            viewport.cell_at(cell_center(viewport, 31, 255)),
            Some((31, 255))
        );
    }

    #[test]
    fn non_square_320x192_world_picks_first_middle_and_last_cells() {
        let viewport =
            WorldViewport::calculate(1600, 900, 320, 192, PresentationPalette::ThermalLab).unwrap();
        for cell in [(0, 0), (159, 95), (319, 191)] {
            assert_eq!(
                viewport.cell_at(cell_center(viewport, cell.0, cell.1)),
                Some(cell)
            );
        }
        assert_eq!(
            viewport.cell_at(PhysicalPosition::new(
                f64::from(viewport.right()),
                f64::from(viewport.bottom()) - 0.01,
            )),
            None
        );
    }

    #[test]
    fn resize_and_dpi_scaled_physical_surfaces_keep_the_requested_cell() {
        for (surface_width, surface_height) in [(1600, 900), (1200, 1000), (2400, 1350)] {
            let viewport = WorldViewport::calculate(
                surface_width,
                surface_height,
                256,
                256,
                PresentationPalette::Gallery,
            )
            .unwrap();
            let cursor = cell_center(viewport, 37, 201);
            assert_eq!(viewport.cell_at(cursor), Some((37, 201)));
            assert_eq!(viewport.cell_at(cursor), viewport.cell_at(cursor));
        }
    }

    #[test]
    fn invalid_or_absent_world_views_are_not_pickable() {
        let cursor = PhysicalPosition::new(0.0, 0.0);
        assert_eq!(pick_world_cell(None, cursor), None);
        assert!(WorldViewport::calculate(0, 900, 256, 256, PresentationPalette::Gallery).is_none());
        assert!(
            WorldViewport::calculate(1600, 0, 256, 256, PresentationPalette::Gallery).is_none()
        );
        assert!(
            WorldViewport::calculate(1600, 900, 0, 256, PresentationPalette::Gallery).is_none()
        );
        assert!(
            WorldViewport::calculate(1600, 900, 256, 0, PresentationPalette::Gallery).is_none()
        );
    }

    #[test]
    fn sandbox_camera_transform_pans_zooms_and_keeps_renderer_picker_shared() {
        let viewport =
            WorldViewport::calculate(1600, 900, 256, 256, PresentationPalette::Sandbox).unwrap();
        assert_eq!(
            (viewport.x, viewport.y, viewport.width, viewport.height),
            (410.0, 56.0, 780.0, 780.0)
        );
        let fitted = WorldTransform::calculate(viewport, WorldCamera::fitted(256, 256));
        assert_eq!(
            fitted.cell_at(PhysicalPosition::new(
                f64::from(viewport.x + 1.0),
                f64::from(viewport.y + 1.0),
            )),
            Some((0, 0))
        );

        let camera = WorldCamera {
            center_x: 96.0,
            center_y: 160.0,
            zoom: 4.0,
        };
        let zoomed = WorldTransform::calculate(viewport, camera);
        assert_eq!((zoomed.origin_x, zoomed.origin_y), (64.0, 128.0));
        assert_eq!(
            zoomed.cell_at(PhysicalPosition::new(
                f64::from(viewport.x + 0.5 * zoomed.scale),
                f64::from(viewport.y + 0.5 * zoomed.scale),
            )),
            Some((64, 128))
        );
        assert_eq!(
            zoomed.cell_at(PhysicalPosition::new(
                f64::from(viewport.right()),
                f64::from(viewport.y),
            )),
            None
        );
    }

    #[test]
    fn sandbox_camera_clamps_nonfinite_and_world_loss() {
        let viewport =
            WorldViewport::calculate(1200, 1000, 256, 256, PresentationPalette::Sandbox).unwrap();
        let transform = WorldTransform::calculate(
            viewport,
            WorldCamera {
                center_x: f32::NAN,
                center_y: f32::INFINITY,
                zoom: f32::NEG_INFINITY,
            },
        );
        assert!(transform.origin_x.is_finite());
        assert!(transform.origin_y.is_finite());
        assert!(transform.scale.is_finite());
        assert_eq!((transform.origin_x, transform.origin_y), (0.0, 0.0));

        let edge = WorldTransform::calculate(
            viewport,
            WorldCamera {
                center_x: -1000.0,
                center_y: 1000.0,
                zoom: 16.0,
            },
        );
        assert_eq!(edge.origin_x, 0.0);
        assert_eq!(edge.origin_y, 240.0);
    }

    #[test]
    fn sandbox_pan_and_cursor_anchored_zoom_are_deterministic_after_resize() {
        for (width, height) in [(1600, 900), (1400, 1000), (2400, 1350)] {
            let viewport =
                WorldViewport::calculate(width, height, 256, 256, PresentationPalette::Sandbox)
                    .unwrap();
            let cursor = PhysicalPosition::new(
                f64::from(viewport.x + viewport.width * 0.63),
                f64::from(viewport.y + viewport.height * 0.41),
            );
            let camera = WorldCamera::fitted(256, 256).zoomed_at(viewport, cursor, 4.0);
            let before = WorldTransform::calculate(viewport, camera)
                .world_at(cursor)
                .unwrap();
            let camera = camera.zoomed_at(viewport, cursor, 1.25);
            let after = WorldTransform::calculate(viewport, camera)
                .world_at(cursor)
                .unwrap();
            assert!((before.0 - after.0).abs() < 0.001);
            assert!((before.1 - after.1).abs() < 0.001);

            let panned = camera.panned_by_pixels(viewport, 25.0, -40.0);
            assert!(panned.center_x.is_finite() && panned.center_y.is_finite());
            assert_ne!(panned, camera);
        }
    }
}

#[cfg(test)]
mod capture_tests {
    use super::*;

    #[test]
    fn renderer_capture_unpads_rgba_rows() {
        let layout = capture_layout(3, 2, TextureFormat::Rgba8Unorm).unwrap();
        assert_eq!(layout.unpadded_bytes_per_row, 12);
        assert_eq!(layout.padded_bytes_per_row, 256);
        assert_eq!(layout.buffer_size, 512);

        let mut mapped = vec![0xEE; layout.buffer_size as usize];
        mapped[..12].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        mapped[256..268].copy_from_slice(&[21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]);

        let rgba = captured_bytes_to_rgba(&mapped, 3, 2, layout).unwrap();
        assert_eq!(
            rgba,
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
                32,
            ]
        );
    }

    #[test]
    fn renderer_capture_swizzles_bgra_to_rgba() {
        let layout = capture_layout(2, 1, TextureFormat::Bgra8UnormSrgb).unwrap();
        let mut mapped = vec![0; layout.buffer_size as usize];
        mapped[..8].copy_from_slice(&[3, 2, 1, 4, 30, 20, 10, 40]);

        assert_eq!(
            captured_bytes_to_rgba(&mapped, 2, 1, layout).unwrap(),
            [1, 2, 3, 4, 10, 20, 30, 40]
        );
    }

    #[test]
    fn renderer_capture_rejects_invalid_layouts_and_truncated_input() {
        assert!(capture_layout(0, 1, TextureFormat::Rgba8Unorm).is_err());
        assert!(capture_layout(1, 0, TextureFormat::Rgba8Unorm).is_err());
        assert!(capture_layout(u32::MAX, 1, TextureFormat::Rgba8Unorm).is_err());
        assert!(capture_layout(1, 1, TextureFormat::R8Unorm).is_err());

        let layout = capture_layout(1, 2, TextureFormat::Rgba8UnormSrgb).unwrap();
        assert!(captured_bytes_to_rgba(&[0; 511], 1, 2, layout).is_err());
    }
}

#[cfg(test)]
mod render_timestamp_tests {
    use super::*;

    #[test]
    fn render_timestamps_preserve_raw_identity_and_period_conversion() {
        let samples = timestamp_samples_from_raw(&[10, 30, 40, 65], 2.0).unwrap();
        assert_eq!(samples.len(), 2);
        assert_eq!((samples[0].start_tick, samples[0].end_tick), (10, 30));
        assert_eq!(samples[0].duration_ms, 0.000_04);
        assert_eq!((samples[1].start_tick, samples[1].end_tick), (40, 65));
        assert_eq!(samples[1].duration_ms, 0.000_05);
    }

    #[test]
    fn render_timestamps_reject_malformed_and_out_of_order_windows() {
        assert!(timestamp_samples_from_raw(&[], 1.0).is_err());
        assert!(timestamp_samples_from_raw(&[1], 1.0).is_err());
        assert!(timestamp_samples_from_raw(&[1, 2], 0.0).is_err());
        assert!(timestamp_samples_from_raw(&[2, 2], 1.0).is_err());
        assert!(timestamp_samples_from_raw(&[4, 8, 7, 9], 1.0).is_err());
    }

    #[test]
    fn measured_surface_errors_have_stable_recovery_and_fatal_classes() {
        assert_eq!(
            classify_measurement_surface_error(&wgpu::SurfaceError::Timeout),
            ("timeout", false, false)
        );
        assert_eq!(
            classify_measurement_surface_error(&wgpu::SurfaceError::Outdated),
            ("outdated", true, false)
        );
        assert_eq!(
            classify_measurement_surface_error(&wgpu::SurfaceError::Lost),
            ("lost", true, false)
        );
        assert_eq!(
            classify_measurement_surface_error(&wgpu::SurfaceError::OutOfMemory),
            ("out_of_memory", false, true)
        );
        assert_eq!(
            classify_measurement_surface_error(&wgpu::SurfaceError::Other),
            ("other", false, false)
        );
    }
}
