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
//! The world view preserves square cells: it letterboxes the world into the
//! surface with `scale = min(surface_w / world_w, surface_h / world_h)` and
//! maps pixels to cells with integer truncation, so cell edges stay crisp
//! and the world aspect ratio is never distorted.

use std::sync::Arc;

use wgpu::util::DeviceExt;
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

/// Clear color for the empty G0 world frame (a dim slate blue).
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.02,
    g: 0.02,
    b: 0.05,
    a: 1.0,
};

/// Params uniform: world size + surface size + palette id (8 u32 = 32 B).
const WORLD_VIEW_PARAMS_SIZE: u64 = 32;
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
        || palette == PALETTE_GALLERY) {
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
    let thermal = params.palette == PALETTE_THERMAL;
    let integrity = params.palette == PALETTE_INTEGRITY;
    let activity = params.palette == PALETTE_ACTIVITY;
    let gallery = params.palette == PALETTE_GALLERY;
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
    } else if (thermal) {
        let sidebar_w = 270.0;
        let avail_w = max(fw - sidebar_w * 2.0, 1.0);
        let avail_h = max(fh - 140.0, 1.0);
        scale = min(avail_w / ww, avail_h / wh);
        off_x = (fw - ww * scale) * 0.5;
        off_y = 65.0 + (avail_h - wh * scale) * 0.5;
    } else if (integrity || gallery) {
        // G6: leave the left/right HUD cards (340 px) and the top banner /
        // bottom controls bar clear of the world view.
        let sidebar_w = 400.0;
        let avail_w = max(fw - sidebar_w * 2.0, 1.0);
        let avail_h = max(fh - 140.0, 1.0);
        scale = min(avail_w / ww, avail_h / wh);
        off_x = (fw - ww * scale) * 0.5;
        off_y = 60.0 + (avail_h - wh * scale) * 0.5;
    } else if (activity) {
        // G7: same letterboxing as G6 — banner + cards clear of the world.
        let sidebar_w = 400.0;
        let avail_w = max(fw - sidebar_w * 2.0, 1.0);
        let avail_h = max(fh - 140.0, 1.0);
        scale = min(avail_w / ww, avail_h / wh);
        off_x = (fw - ww * scale) * 0.5;
        off_y = 60.0 + (avail_h - wh * scale) * 0.5;
    }
    let px = frag.x;
    let py = frag.y;
    let in_viewport = px >= off_x && px < off_x + ww * scale
                   && py >= off_y && py < off_y + wh * scale;
    if (in_viewport) {
        let cell_x = min(u32((px - off_x) / scale), params.width - 1u);
        let cell_y = min(u32((py - off_y) / scale), params.height - 1u);
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
    if (integrity || gallery) {
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
                        | PresentationPalette::Integrity
                        | PresentationPalette::Activity
                        | PresentationPalette::Gallery
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
    Pressure(&'a crate::observatory::PressureObservatoryMetrics, u64),
    ParallelIntegrity(&'a crate::observatory::IntegrityMetrics, u64),
    Activity(&'a crate::observatory::ActivityMetrics, u64),
    Gallery(&'a crate::gallery::GalleryHudData),
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
                }
            }
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
    palette: u32,
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
    data[20..24].copy_from_slice(&wv.chunk_size.to_ne_bytes());
    let chunks_x = if wv.chunk_size == 0 {
        0
    } else {
        wv.world_width.div_ceil(wv.chunk_size)
    };
    data[24..28].copy_from_slice(&chunks_x.to_ne_bytes());
    queue.write_buffer(&wv.params, 0, &data);
}
