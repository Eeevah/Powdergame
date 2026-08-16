//! Live diagnostic metrics for the G4 Thermal Observatory.
//!
//! This module provides non-blocking diagnostic readback from the authoritative
//! GPU world buffers (Current: material, temperature, flags) to compute live
//! numeric metrics for each of the four observatory panels:
//!   - Panel A (PHASE HEATING): live Ice / Water / Steam cell counts, first melt & first steam ticks
//!   - Panel B (PHASE COOLING): live Steam / Water / Ice cell counts, first condense & first freeze ticks
//!   - Panel C (HEAT COMPARISON): Water vs Oil mid/top/max temperatures and reach ticks
//!   - Panel D (COMBUSTION): Wood remaining, currently burning, first ignite & first empty ticks
//!
//! Readbacks are asynchronous and decoupled from the production simulation tick
//! loop. Simulation truth is never mutated by diagnostic instrumentation.

use powdergame_core::{
    chunk_count, chunks_x, chunks_y, ACTIVITY_MATTER, ACTIVITY_PRESSURE, ACTIVITY_REACTION,
    ACTIVITY_THERMAL, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STEAM,
    MATERIAL_WATER, MATERIAL_WOOD,
};
use powdergame_gpu::Simulation;

/// Diagnostic threshold for detecting warm heat reach in Heat Comparison tubes.
/// (Observation threshold; relative gameplay scalar, not a simulation physics constant).
pub const THERMAL_OBS_WARM_THRESHOLD: f32 = 10.0;

/// Bit flag indicating active combustion on a cell (matches core/GPU shader).
pub const FLAG_COMBUSTING: u32 = 1;

/// Decay-age bit range for Material-owned decay (matches `decay.wgsl`).
pub const FLAG_DECAY_AGE_SHIFT: u32 = 16;
pub const FLAG_DECAY_AGE_MASK: u32 = 0x0FFF;

// A claim-losing expansion source receives the Material-owned blocked
// pressure impulse (`WATER_BOIL_BLOCKED_PRESSURE`, 100); the claim winner
// receives no impulse and only carries pressure diffused in from its losing
// neighbors during the same tick. The loser count in `IntegrityMetrics` is
// derived as "sources whose pressure exceeds the minimum source pressure" —
// the winner is always the minimum — without any hardcoded expectation of
// WHICH source won the hash arbitration.

/// Sentinel value indicating no event has occurred yet (e.g. tick record is None).
#[allow(dead_code)]
pub const TICK_NONE: u32 = 0xFFFF_FFFF;

// ── G6 Panel bounds (world 256×256, stone dividers at x 127..128 / y 127..128) ──
pub const G6_A_X_MIN: u32 = 1;
pub const G6_A_X_MAX: u32 = 126;
pub const G6_A_Y_MIN: u32 = 1;
pub const G6_A_Y_MAX: u32 = 126;
pub const G6_B_X_MIN: u32 = 129;
pub const G6_B_X_MAX: u32 = 254;
pub const G6_B_Y_MIN: u32 = 1;
pub const G6_B_Y_MAX: u32 = 126;
pub const G6_C_X_MIN: u32 = 1;
pub const G6_C_X_MAX: u32 = 126;
pub const G6_C_Y_MIN: u32 = 129;
pub const G6_C_Y_MAX: u32 = 254;
pub const G6_D_X_MIN: u32 = 129;
pub const G6_D_X_MAX: u32 = 254;
pub const G6_D_Y_MIN: u32 = 129;
pub const G6_D_Y_MAX: u32 = 254;

// ── G6 Panel C one-tick ownership fixture (staged by `stage_parallel_integrity_demo`;
//    these constants MUST match the staging geometry so the readback evaluator
//    interprets the first tick correctly). ──
/// Shared EMPTY destination the three boiling Water sources all propose.
pub const EXP_TARGET: (u32, u32) = (22, 185);
/// The three boiling Water sources (must all become Steam on tick 1).
pub const EXP_SOURCES: [(u32, u32); 3] = [(21, 186), (22, 186), (23, 186)];
/// Stone rect (x0, y0, x1, y1) surrounding the expansion fixture — every
/// source neighbor except the shared target is Stone, so the target is the
/// only valid expansion candidate.
pub const EXP_REGION: (u32, u32, u32, u32) = (18, 183, 26, 190);
/// Shared EMPTY Smoke destination the three burning Wood sources all propose.
pub const SMOKE_TARGET: (u32, u32) = (100, 185);
/// The three Wood sources (ignite on tick 1, all preserved).
pub const SMOKE_SOURCES: [(u32, u32); 3] = [(99, 186), (100, 186), (101, 186)];
/// Stone rect around the smoke fixture (all non-target neighbors Stone).
pub const SMOKE_REGION: (u32, u32, u32, u32) = (96, 183, 104, 190);
/// Small movement fixture: Sand at `MOVE_SRC`, EMPTY at `MOVE_DST` — proves the
/// movement pass ran in the SAME tick as expansion + smoke (scratch reuse).
pub const MOVE_SRC: (u32, u32) = (60, 180);
pub const MOVE_DST: (u32, u32) = (60, 181);
pub const MOVE_REGION: (u32, u32, u32, u32) = (58, 178, 62, 183);

/// Bounding boxes for the four 4-panel observatory chambers (world size 320×192).
pub const PANEL_A_X_MIN: u32 = 1;
pub const PANEL_A_X_MAX: u32 = 157;
pub const PANEL_A_Y_MIN: u32 = 1;
pub const PANEL_A_Y_MAX: u32 = 93;

pub const PANEL_B_X_MIN: u32 = 162;
pub const PANEL_B_X_MAX: u32 = 318;
pub const PANEL_B_Y_MIN: u32 = 1;
pub const PANEL_B_Y_MAX: u32 = 93;

pub const PANEL_C_X_MIN: u32 = 1;
pub const PANEL_C_X_MAX: u32 = 157;
pub const PANEL_C_Y_MIN: u32 = 98;
pub const PANEL_C_Y_MAX: u32 = 190;

pub const PANEL_D_X_MIN: u32 = 162;
pub const PANEL_D_X_MAX: u32 = 318;
pub const PANEL_D_Y_MIN: u32 = 98;
pub const PANEL_D_Y_MAX: u32 = 190;

// Sub-regions for Panel C (Water vs Oil tubes)
pub const TUBE_WATER_X_MIN: u32 = 25;
pub const TUBE_WATER_X_MAX: u32 = 65;
pub const TUBE_OIL_X_MIN: u32 = 90;
pub const TUBE_OIL_X_MAX: u32 = 130;
pub const TUBE_INTERIOR_Y_MIN: u32 = 112;
pub const TUBE_INTERIOR_Y_MAX: u32 = 174;
// 25% height band (LOW probe, 14 cells from heat source)
pub const TUBE_LOW_Y_MIN: u32 = 157;
pub const TUBE_LOW_Y_MAX: u32 = 161;
// 50% height band (MID probe, 30 cells from heat source)
pub const TUBE_MID_Y_MIN: u32 = 141;
pub const TUBE_MID_Y_MAX: u32 = 145;
// 75% height band (HIGH probe, 46 cells from heat source)
pub const TUBE_HIGH_Y_MIN: u32 = 125;
pub const TUBE_HIGH_Y_MAX: u32 = 129;

// Sub-region for Panel D (Initial Wood footprint)
pub const WOOD_FOOTPRINT_X_MIN: u32 = 200;
pub const WOOD_FOOTPRINT_X_MAX: u32 = 280;
pub const WOOD_FOOTPRINT_Y_MIN: u32 = 146;
pub const WOOD_FOOTPRINT_Y_MAX: u32 = 153;

/// Live diagnostic snapshot for the 4-panel Thermal Observatory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObservatoryMetrics {
    // Panel A: PHASE HEATING
    pub a_ice_count: u32,
    pub a_water_count: u32,
    pub a_steam_count: u32,
    pub a_first_melt: Option<u64>,
    pub a_first_steam: Option<u64>,

    // Panel B: PHASE COOLING
    pub b_steam_count: u32,
    pub b_water_count: u32,
    pub b_ice_count: u32,
    pub b_first_condense: Option<u64>,
    pub b_first_freeze: Option<u64>,

    // Panel C: HEAT COMPARISON (Water vs Oil penetration probes)
    pub c_w_low_t: f32,
    pub c_w_mid_t: f32,
    pub c_w_high_t: f32,
    pub c_o_low_t: f32,
    pub c_o_mid_t: f32,
    pub c_o_high_t: f32,
    pub c_w_low_reach: Option<u64>,
    pub c_o_low_reach: Option<u64>,
    pub c_w_mid_reach: Option<u64>,
    pub c_o_mid_reach: Option<u64>,
    pub c_w_high_reach: Option<u64>,
    pub c_o_high_reach: Option<u64>,

    // Panel D: COMBUSTION
    pub d_wood_start: u32,
    pub d_wood_left: u32,
    pub d_burning: u32,
    pub d_smoke_count: u32,
    pub d_first_ignite: Option<u64>,
    pub d_first_empty: Option<u64>,

    pub current_tick: u64,
}

/// Live diagnostic metrics for the G5 Pressure Multi-Boiler Stress Lab (2×2 layout).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PressureObservatoryMetrics {
    // Top-Left: WOOD RELIEF (CANONICAL STANDARD)
    pub tl_peak_pressure: f32,
    pub tl_current_pressure: f32,
    pub tl_relief_tick: Option<u64>,
    pub tl_wood_remaining: u32,
    pub tl_steam_count: u32,

    // Top-Right: STONE SEALED (CANONICAL STANDARD CONTROL)
    pub tr_peak_pressure: f32,
    pub tr_current_pressure: f32,
    pub tr_is_sealed: bool,
    pub tr_steam_count: u32,

    // Bottom-Left: WOOD RELIEF (EXTREME OVERDRIVE)
    pub bl_peak_pressure: f32,
    pub bl_current_pressure: f32,
    pub bl_relief_tick: Option<u64>,
    pub bl_wood_remaining: u32,
    pub bl_steam_count: u32,

    // Bottom-Right: STONE SEALED (DELAYED PRESSURE BREACH)
    pub br_peak_pressure: f32,
    pub br_current_pressure: f32,
    pub br_rupture_tick: Option<u64>,
    pub br_weak_seam_remaining: u32,
    pub br_breach_cell: Option<(u32, u32)>,
    pub br_breach_local_pressure: f32,
    pub br_exterior_steam_count: u32,
    pub br_first_vent_tick: Option<u64>,
    pub br_steam_count: u32,

    pub current_tick: u64,
}

impl Default for PressureObservatoryMetrics {
    fn default() -> Self {
        Self {
            tl_peak_pressure: 0.0,
            tl_current_pressure: 0.0,
            tl_relief_tick: None,
            tl_wood_remaining: 9,
            tl_steam_count: 0,

            tr_peak_pressure: 0.0,
            tr_current_pressure: 0.0,
            tr_is_sealed: true,
            tr_steam_count: 0,

            bl_peak_pressure: 0.0,
            bl_current_pressure: 0.0,
            bl_relief_tick: None,
            bl_wood_remaining: 9,
            bl_steam_count: 0,

            br_peak_pressure: 0.0,
            br_current_pressure: 0.0,
            br_rupture_tick: None,
            br_weak_seam_remaining: 9,
            br_breach_cell: None,
            br_breach_local_pressure: 0.0,
            br_exterior_steam_count: 0,
            br_first_vent_tick: None,
            br_steam_count: 0,

            current_tick: 0,
        }
    }
}

impl Default for ObservatoryMetrics {
    fn default() -> Self {
        Self {
            a_ice_count: 0,
            a_water_count: 0,
            a_steam_count: 0,
            a_first_melt: None,
            a_first_steam: None,

            b_steam_count: 0,
            b_water_count: 0,
            b_ice_count: 0,
            b_first_condense: None,
            b_first_freeze: None,

            c_w_low_t: 0.0,
            c_w_mid_t: 0.0,
            c_w_high_t: 0.0,
            c_o_low_t: 0.0,
            c_o_mid_t: 0.0,
            c_o_high_t: 0.0,
            c_w_low_reach: None,
            c_o_low_reach: None,
            c_w_mid_reach: None,
            c_o_mid_reach: None,
            c_w_high_reach: None,
            c_o_high_reach: None,

            d_wood_start: 0,
            d_wood_left: 0,
            d_burning: 0,
            d_smoke_count: 0,
            d_first_ignite: None,
            d_first_empty: None,

            current_tick: 0,
        }
    }
}

/// Packed 128-byte uniform buffer payload sent to WGSL for procedural HUD rendering.
#[allow(dead_code)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetricsUniform {
    // Panel A (5 u32): ice, water, steam, first_melt, first_steam
    pub a_ice: u32,
    pub a_water: u32,
    pub a_steam: u32,
    pub a_first_melt: u32,
    pub a_first_steam: u32,

    // Panel B (5 u32): steam, water, ice, first_condense, first_freeze
    pub b_steam: u32,
    pub b_water: u32,
    pub b_ice: u32,
    pub b_first_condense: u32,
    pub b_first_freeze: u32,

    // Panel C temps (6 f32): w_low, w_mid, w_high, o_low, o_mid, o_high
    pub c_w_low_t: f32,
    pub c_w_mid_t: f32,
    pub c_w_high_t: f32,
    pub c_o_low_t: f32,
    pub c_o_mid_t: f32,
    pub c_o_high_t: f32,

    // Panel C reach ticks (6 u32): w_low, o_low, w_mid, o_mid, w_high, o_high
    pub c_w_low_reach: u32,
    pub c_o_low_reach: u32,
    pub c_w_mid_reach: u32,
    pub c_o_mid_reach: u32,
    pub c_w_high_reach: u32,
    pub c_o_high_reach: u32,

    // Panel D (6 u32): wood_start, wood_left, burning, smoke_count, first_ignite, first_empty
    pub d_wood_start: u32,
    pub d_wood_left: u32,
    pub d_burning: u32,
    pub d_smoke_count: u32,
    pub d_first_ignite: u32,
    pub d_first_empty: u32,

    // Tick count & padding to 32 u32 words = 128 bytes
    pub current_tick: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub _pad2: u32,
}

impl MetricsUniform {
    pub fn to_bytes(self) -> [u8; 128] {
        let mut data = [0u8; 128];
        data[0..4].copy_from_slice(&self.a_ice.to_ne_bytes());
        data[4..8].copy_from_slice(&self.a_water.to_ne_bytes());
        data[8..12].copy_from_slice(&self.a_steam.to_ne_bytes());
        data[12..16].copy_from_slice(&self.a_first_melt.to_ne_bytes());
        data[16..20].copy_from_slice(&self.a_first_steam.to_ne_bytes());

        data[20..24].copy_from_slice(&self.b_steam.to_ne_bytes());
        data[24..28].copy_from_slice(&self.b_water.to_ne_bytes());
        data[28..32].copy_from_slice(&self.b_ice.to_ne_bytes());
        data[32..36].copy_from_slice(&self.b_first_condense.to_ne_bytes());
        data[36..40].copy_from_slice(&self.b_first_freeze.to_ne_bytes());

        data[40..44].copy_from_slice(&self.c_w_low_t.to_ne_bytes());
        data[44..48].copy_from_slice(&self.c_w_mid_t.to_ne_bytes());
        data[48..52].copy_from_slice(&self.c_w_high_t.to_ne_bytes());
        data[52..56].copy_from_slice(&self.c_o_low_t.to_ne_bytes());
        data[56..60].copy_from_slice(&self.c_o_mid_t.to_ne_bytes());
        data[60..64].copy_from_slice(&self.c_o_high_t.to_ne_bytes());

        data[64..68].copy_from_slice(&self.c_w_low_reach.to_ne_bytes());
        data[68..72].copy_from_slice(&self.c_o_low_reach.to_ne_bytes());
        data[72..76].copy_from_slice(&self.c_w_mid_reach.to_ne_bytes());
        data[76..80].copy_from_slice(&self.c_o_mid_reach.to_ne_bytes());
        data[80..84].copy_from_slice(&self.c_w_high_reach.to_ne_bytes());
        data[84..88].copy_from_slice(&self.c_o_high_reach.to_ne_bytes());

        data[88..92].copy_from_slice(&self.d_wood_start.to_ne_bytes());
        data[92..96].copy_from_slice(&self.d_wood_left.to_ne_bytes());
        data[96..100].copy_from_slice(&self.d_burning.to_ne_bytes());
        data[100..104].copy_from_slice(&self.d_smoke_count.to_ne_bytes());
        data[104..108].copy_from_slice(&self.d_first_ignite.to_ne_bytes());
        data[108..112].copy_from_slice(&self.d_first_empty.to_ne_bytes());

        data[112..116].copy_from_slice(&self.current_tick.to_ne_bytes());
        data[116..120].copy_from_slice(&self._pad0.to_ne_bytes());
        data[120..124].copy_from_slice(&self._pad1.to_ne_bytes());
        data[124..128].copy_from_slice(&self._pad2.to_ne_bytes());
        data
    }
}

impl From<&ObservatoryMetrics> for MetricsUniform {
    fn from(m: &ObservatoryMetrics) -> Self {
        let opt = |v: Option<u64>| v.map(|t| t as u32).unwrap_or(TICK_NONE);
        Self {
            a_ice: m.a_ice_count,
            a_water: m.a_water_count,
            a_steam: m.a_steam_count,
            a_first_melt: opt(m.a_first_melt),
            a_first_steam: opt(m.a_first_steam),

            b_steam: m.b_steam_count,
            b_water: m.b_water_count,
            b_ice: m.b_ice_count,
            b_first_condense: opt(m.b_first_condense),
            b_first_freeze: opt(m.b_first_freeze),

            c_w_low_t: m.c_w_low_t,
            c_w_mid_t: m.c_w_mid_t,
            c_w_high_t: m.c_w_high_t,
            c_o_low_t: m.c_o_low_t,
            c_o_mid_t: m.c_o_mid_t,
            c_o_high_t: m.c_o_high_t,

            c_w_low_reach: opt(m.c_w_low_reach),
            c_o_low_reach: opt(m.c_o_low_reach),
            c_w_mid_reach: opt(m.c_w_mid_reach),
            c_o_mid_reach: opt(m.c_o_mid_reach),
            c_w_high_reach: opt(m.c_w_high_reach),
            c_o_high_reach: opt(m.c_o_high_reach),

            d_wood_start: m.d_wood_start,
            d_wood_left: m.d_wood_left,
            d_burning: m.d_burning,
            d_smoke_count: m.d_smoke_count,
            d_first_ignite: opt(m.d_first_ignite),
            d_first_empty: opt(m.d_first_empty),

            current_tick: m.current_tick as u32,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        }
    }
}

/// Pure CPU analysis of dense world buffers to update metrics.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_observatory_state(
    materials: &[u32],
    temperatures: &[f32],
    flags: &[u32],
    width: u32,
    height: u32,
    tick: u64,
    metrics: &mut ObservatoryMetrics,
    initial_a_ice: &mut u32,
    initial_b_steam: &mut u32,
    initial_wood_set: &mut bool,
) {
    metrics.current_tick = tick;

    // Reset current frame counters
    let mut a_ice = 0u32;
    let mut a_water = 0u32;
    let mut a_steam = 0u32;

    let mut b_steam = 0u32;
    let mut b_water = 0u32;
    let mut b_ice = 0u32;

    let mut c_w_low_sum = 0.0f32;
    let mut c_w_low_cnt = 0u32;
    let mut c_w_mid_sum = 0.0f32;
    let mut c_w_mid_cnt = 0u32;
    let mut c_w_high_sum = 0.0f32;
    let mut c_w_high_cnt = 0u32;

    let mut c_o_low_sum = 0.0f32;
    let mut c_o_low_cnt = 0u32;
    let mut c_o_mid_sum = 0.0f32;
    let mut c_o_mid_cnt = 0u32;
    let mut c_o_high_sum = 0.0f32;
    let mut c_o_high_cnt = 0u32;

    let mut d_wood_left = 0u32;
    let mut d_burning = 0u32;
    let mut d_smoke_count = 0u32;
    let mut d_empty_in_footprint = false;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx >= materials.len() {
                continue;
            }
            let mat = materials[idx];
            let temp = temperatures[idx];
            let flag = flags[idx];

            // ─── Panel A (PHASE HEATING) ───
            if (PANEL_A_X_MIN..=PANEL_A_X_MAX).contains(&x)
                && (PANEL_A_Y_MIN..=PANEL_A_Y_MAX).contains(&y)
            {
                if mat == MATERIAL_ICE {
                    a_ice += 1;
                } else if mat == MATERIAL_WATER {
                    a_water += 1;
                } else if mat == MATERIAL_STEAM {
                    a_steam += 1;
                }
            }

            // ─── Panel B (PHASE COOLING) ───
            if (PANEL_B_X_MIN..=PANEL_B_X_MAX).contains(&x)
                && (PANEL_B_Y_MIN..=PANEL_B_Y_MAX).contains(&y)
            {
                if mat == MATERIAL_STEAM {
                    b_steam += 1;
                } else if mat == MATERIAL_WATER {
                    b_water += 1;
                } else if mat == MATERIAL_ICE {
                    b_ice += 1;
                }
            }

            // ─── Panel C (HEAT COMPARISON) ───
            if (PANEL_C_X_MIN..=PANEL_C_X_MAX).contains(&x)
                && (PANEL_C_Y_MIN..=PANEL_C_Y_MAX).contains(&y)
            {
                // Water tube
                if (TUBE_WATER_X_MIN..=TUBE_WATER_X_MAX).contains(&x)
                    && (TUBE_INTERIOR_Y_MIN..=TUBE_INTERIOR_Y_MAX).contains(&y)
                {
                    if (TUBE_LOW_Y_MIN..=TUBE_LOW_Y_MAX).contains(&y) {
                        c_w_low_sum += temp;
                        c_w_low_cnt += 1;
                    }
                    if (TUBE_MID_Y_MIN..=TUBE_MID_Y_MAX).contains(&y) {
                        c_w_mid_sum += temp;
                        c_w_mid_cnt += 1;
                    }
                    if (TUBE_HIGH_Y_MIN..=TUBE_HIGH_Y_MAX).contains(&y) {
                        c_w_high_sum += temp;
                        c_w_high_cnt += 1;
                    }
                }

                // Oil tube
                if (TUBE_OIL_X_MIN..=TUBE_OIL_X_MAX).contains(&x)
                    && (TUBE_INTERIOR_Y_MIN..=TUBE_INTERIOR_Y_MAX).contains(&y)
                {
                    if (TUBE_LOW_Y_MIN..=TUBE_LOW_Y_MAX).contains(&y) {
                        c_o_low_sum += temp;
                        c_o_low_cnt += 1;
                    }
                    if (TUBE_MID_Y_MIN..=TUBE_MID_Y_MAX).contains(&y) {
                        c_o_mid_sum += temp;
                        c_o_mid_cnt += 1;
                    }
                    if (TUBE_HIGH_Y_MIN..=TUBE_HIGH_Y_MAX).contains(&y) {
                        c_o_high_sum += temp;
                        c_o_high_cnt += 1;
                    }
                }
            }

            // ─── Panel D (COMBUSTION) ───
            if (PANEL_D_X_MIN..=PANEL_D_X_MAX).contains(&x)
                && (PANEL_D_Y_MIN..=PANEL_D_Y_MAX).contains(&y)
            {
                if mat == MATERIAL_WOOD {
                    d_wood_left += 1;
                }
                if mat == MATERIAL_SMOKE {
                    d_smoke_count += 1;
                }
                if (flag & FLAG_COMBUSTING) != 0 {
                    d_burning += 1;
                }
                if (WOOD_FOOTPRINT_X_MIN..=WOOD_FOOTPRINT_X_MAX).contains(&x)
                    && (WOOD_FOOTPRINT_Y_MIN..=WOOD_FOOTPRINT_Y_MAX).contains(&y)
                    && mat == MATERIAL_EMPTY
                {
                    d_empty_in_footprint = true;
                }
            }
        }
    }

    // Set initial counts if at tick 0 or not yet set
    if tick == 0 || !*initial_wood_set {
        *initial_a_ice = a_ice;
        *initial_b_steam = b_steam;
        metrics.d_wood_start = d_wood_left;
        *initial_wood_set = true;
    }

    // Update Panel A metrics & event latches
    metrics.a_ice_count = a_ice;
    metrics.a_water_count = a_water;
    metrics.a_steam_count = a_steam;
    if metrics.a_first_melt.is_none() && (a_ice < *initial_a_ice || a_water > 0) && tick > 0 {
        metrics.a_first_melt = Some(tick);
    }
    if metrics.a_first_steam.is_none() && a_steam > 0 && tick > 0 {
        metrics.a_first_steam = Some(tick);
    }

    // Update Panel B metrics & event latches
    metrics.b_steam_count = b_steam;
    metrics.b_water_count = b_water;
    metrics.b_ice_count = b_ice;
    if metrics.b_first_condense.is_none() && (b_water > 0 || b_steam < *initial_b_steam) && tick > 0
    {
        metrics.b_first_condense = Some(tick);
    }
    if metrics.b_first_freeze.is_none() && b_ice > 0 && tick > 0 {
        metrics.b_first_freeze = Some(tick);
    }

    // Update Panel C metrics & reach latches
    metrics.c_w_low_t = if c_w_low_cnt > 0 {
        c_w_low_sum / c_w_low_cnt as f32
    } else {
        0.0
    };
    metrics.c_w_mid_t = if c_w_mid_cnt > 0 {
        c_w_mid_sum / c_w_mid_cnt as f32
    } else {
        0.0
    };
    metrics.c_w_high_t = if c_w_high_cnt > 0 {
        c_w_high_sum / c_w_high_cnt as f32
    } else {
        0.0
    };

    metrics.c_o_low_t = if c_o_low_cnt > 0 {
        c_o_low_sum / c_o_low_cnt as f32
    } else {
        0.0
    };
    metrics.c_o_mid_t = if c_o_mid_cnt > 0 {
        c_o_mid_sum / c_o_mid_cnt as f32
    } else {
        0.0
    };
    metrics.c_o_high_t = if c_o_high_cnt > 0 {
        c_o_high_sum / c_o_high_cnt as f32
    } else {
        0.0
    };

    if metrics.c_w_low_reach.is_none()
        && metrics.c_w_low_t >= THERMAL_OBS_WARM_THRESHOLD
        && tick > 0
    {
        metrics.c_w_low_reach = Some(tick);
    }
    if metrics.c_o_low_reach.is_none()
        && metrics.c_o_low_t >= THERMAL_OBS_WARM_THRESHOLD
        && tick > 0
    {
        metrics.c_o_low_reach = Some(tick);
    }

    if metrics.c_w_mid_reach.is_none()
        && metrics.c_w_mid_t >= THERMAL_OBS_WARM_THRESHOLD
        && tick > 0
    {
        metrics.c_w_mid_reach = Some(tick);
    }
    if metrics.c_o_mid_reach.is_none()
        && metrics.c_o_mid_t >= THERMAL_OBS_WARM_THRESHOLD
        && tick > 0
    {
        metrics.c_o_mid_reach = Some(tick);
    }

    if metrics.c_w_high_reach.is_none()
        && metrics.c_w_high_t >= THERMAL_OBS_WARM_THRESHOLD
        && tick > 0
    {
        metrics.c_w_high_reach = Some(tick);
    }
    if metrics.c_o_high_reach.is_none()
        && metrics.c_o_high_t >= THERMAL_OBS_WARM_THRESHOLD
        && tick > 0
    {
        metrics.c_o_high_reach = Some(tick);
    }

    // Update Panel D metrics & latches
    metrics.d_wood_left = d_wood_left;
    metrics.d_burning = d_burning;
    metrics.d_smoke_count = d_smoke_count;
    if metrics.d_first_ignite.is_none() && d_burning > 0 && tick > 0 {
        metrics.d_first_ignite = Some(tick);
    }
    if metrics.d_first_empty.is_none()
        && (d_wood_left < metrics.d_wood_start || d_empty_in_footprint)
        && (metrics.d_first_ignite.is_some() || d_burning > 0)
        && tick > 0
    {
        metrics.d_first_empty = Some(tick);
    }
}

/// Pure CPU analysis of dense world buffers for the 2×2 G5 Pressure Multi-Boiler Stress Lab.
pub fn evaluate_pressure_observatory_state(
    materials: &[u32],
    _temperatures: &[f32],
    pressures: &[f32],
    width: u32,
    height: u32,
    tick: u64,
    metrics: &mut PressureObservatoryMetrics,
) {
    metrics.current_tick = tick;

    let mut tl_wood = 0u32;
    let mut tl_steam = 0u32;
    let mut tl_max_p = 0.0f32;

    let mut tr_steam = 0u32;
    let mut tr_max_p = 0.0f32;

    let mut bl_wood = 0u32;
    let mut bl_steam = 0u32;
    let mut bl_max_p = 0.0f32;

    let mut br_seam_wood = 0u32;
    let mut br_steam = 0u32;
    let mut br_max_p = 0.0f32;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx >= materials.len() {
                continue;
            }
            let mat = materials[idx];
            let p = if idx < pressures.len() {
                pressures[idx]
            } else {
                0.0
            };

            // Panel A: Top-Left (x 14..114, y 8..114)
            if (14..=114).contains(&x) && (8..=114).contains(&y) {
                if (45..=107).contains(&y) && p > tl_max_p {
                    tl_max_p = p;
                }
                if (60..=68).contains(&x) && y == 44 && mat == MATERIAL_WOOD {
                    tl_wood += 1;
                }
                if mat == MATERIAL_STEAM {
                    tl_steam += 1;
                }
            }

            // Panel B: Top-Right (x 142..242, y 8..114)
            if (142..=242).contains(&x) && (8..=114).contains(&y) {
                if (45..=107).contains(&y) && p > tr_max_p {
                    tr_max_p = p;
                }
                if mat == MATERIAL_STEAM {
                    tr_steam += 1;
                }
            }

            // Panel C: Bottom-Left (x 14..114, y 130..244)
            if (14..=114).contains(&x) && (130..=244).contains(&y) {
                if (171..=233).contains(&y) && p > bl_max_p {
                    bl_max_p = p;
                }
                if (60..=68).contains(&x) && y == 170 && mat == MATERIAL_WOOD {
                    bl_wood += 1;
                }
                if mat == MATERIAL_STEAM {
                    bl_steam += 1;
                }
            }

            // Panel D: Bottom-Right (x 142..254, y 130..244)
            if (142..=254).contains(&x) && (130..=244).contains(&y) {
                if (171..=233).contains(&y) && p > br_max_p {
                    br_max_p = p;
                }
                if x == 242 && (214..=222).contains(&y) && mat == MATERIAL_WOOD {
                    br_seam_wood += 1;
                }
                if mat == MATERIAL_STEAM {
                    br_steam += 1;
                }
            }
        }
    }

    if tl_max_p > metrics.tl_peak_pressure {
        metrics.tl_peak_pressure = tl_max_p;
    }
    metrics.tl_current_pressure = tl_max_p;
    metrics.tl_wood_remaining = tl_wood;
    metrics.tl_steam_count = tl_steam;
    if metrics.tl_relief_tick.is_none() && tl_wood < 9 && tick > 0 {
        metrics.tl_relief_tick = Some(tick);
    }

    if tr_max_p > metrics.tr_peak_pressure {
        metrics.tr_peak_pressure = tr_max_p;
    }
    metrics.tr_current_pressure = tr_max_p;
    metrics.tr_steam_count = tr_steam;

    if bl_max_p > metrics.bl_peak_pressure {
        metrics.bl_peak_pressure = bl_max_p;
    }
    metrics.bl_current_pressure = bl_max_p;
    metrics.bl_wood_remaining = bl_wood;
    metrics.bl_steam_count = bl_steam;
    if metrics.bl_relief_tick.is_none() && bl_wood < 9 && tick > 0 {
        metrics.bl_relief_tick = Some(tick);
    }

    if br_max_p > metrics.br_peak_pressure {
        metrics.br_peak_pressure = br_max_p;
    }
    metrics.br_current_pressure = br_max_p;
    metrics.br_weak_seam_remaining = br_seam_wood;
    metrics.br_steam_count = br_steam;

    // Scan exterior duct for vented steam
    let mut exterior_steam = 0u32;
    for y in 210..=226 {
        for x in 243..=254 {
            let idx = (y * width + x) as usize;
            if idx < materials.len() && materials[idx] == MATERIAL_STEAM {
                exterior_steam += 1;
            }
        }
    }
    metrics.br_exterior_steam_count = exterior_steam;

    if metrics.br_rupture_tick.is_none() && br_seam_wood < 9 && tick > 0 {
        metrics.br_rupture_tick = Some(tick);

        // Find the breach coordinate and local neighbor pressure
        for y in 214..=222 {
            let idx = (y * width + 242) as usize;
            if idx < materials.len() && materials[idx] != MATERIAL_WOOD {
                metrics.br_breach_cell = Some((242, y));
                let left_idx = (y * width + 241) as usize;
                if left_idx < pressures.len() {
                    metrics.br_breach_local_pressure = pressures[left_idx];
                }
                break;
            }
        }
    }

    if metrics.br_first_vent_tick.is_none() && exterior_steam > 0 && tick > 0 {
        metrics.br_first_vent_tick = Some(tick);
    }
}

/// Asynchronous GPU diagnostic readback collector.
pub struct ObservatoryCollector {
    staging_material: wgpu::Buffer,
    staging_temperature: wgpu::Buffer,
    staging_flags: wgpu::Buffer,
    staging_pressure: wgpu::Buffer,
    pending: bool,
    pending_tick: u64,
    receiver: Option<std::sync::mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>>,
    metrics: ObservatoryMetrics,
    pressure_metrics: PressureObservatoryMetrics,
    integrity_metrics: IntegrityMetrics,
    /// G7-A chunk-activity observation metrics (activity demo mode).
    activity_metrics: ActivityMetrics,
    /// Previous sample's per-chunk masks (stable→active wake detection).
    activity_prev_masks: Vec<u32>,
    initial_wood_set: bool,
    initial_a_ice: u32,
    initial_b_steam: u32,
    last_request_tick: u64,
    cell_bytes: u64,
    c_latch_reported: bool,
    /// G6 parallel-integrity instrument: blocking one-shot snapshots for the
    /// pristine tick-0 baseline and the exact tick-1 ownership latch.
    g6_mode: bool,
    /// G7-A activity demo mode: chunk-activity readbacks + aggregation.
    activity_mode: bool,
    chunk_bytes: u64,
    staging_chunk_activity: wgpu::Buffer,
    staging_chunk_changed: wgpu::Buffer,
    staging_chunk_stable: wgpu::Buffer,
    initial_latched: bool,
    c_staging_material: wgpu::Buffer,
    c_staging_temperature: wgpu::Buffer,
    c_staging_flags: wgpu::Buffer,
    c_staging_pressure: wgpu::Buffer,
}

impl ObservatoryCollector {
    /// Creates staging buffers and allocates a new collector. `g6_mode` enables
    /// the G6 parallel-integrity one-tick instrument (blocking tick-0 / tick-1
    /// snapshots + real readback ownership metrics); `activity_mode` enables
    /// the G7-A chunk-activity observation readbacks (activity demo).
    pub fn new(simulation: &Simulation, g6_mode: bool, activity_mode: bool) -> Self {
        let device = &simulation.context.device;
        let cell_bytes = simulation.world.layout.material_bytes;
        let n_chunks = chunk_count(
            simulation.world.config.width,
            simulation.world.config.height,
            simulation.world.config.chunk_size,
        );
        let chunk_bytes = (n_chunks as u64) * 4;

        let staging_material = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("observatory/staging/material"),
            size: cell_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let staging_temperature = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("observatory/staging/temperature"),
            size: cell_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let staging_flags = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("observatory/staging/flags"),
            size: cell_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let staging_pressure = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("observatory/staging/pressure"),
            size: cell_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        // G6 instrument staging set (blocking snapshots, separate from the
        // async pipeline so a slow in-flight map never delays the latch).
        let mk_staging = |label: &str| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: cell_bytes,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        };
        let c_staging_material = mk_staging("observatory/g6-staging/material");
        let c_staging_temperature = mk_staging("observatory/g6-staging/temperature");
        let c_staging_flags = mk_staging("observatory/g6-staging/flags");
        let c_staging_pressure = mk_staging("observatory/g6-staging/pressure");

        // G7-A activity chunk staging (small: 16 u32 for a 4×4-chunk world).
        let mk_chunk_staging = |label: &str| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: chunk_bytes,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        };
        let staging_chunk_activity =
            mk_chunk_staging("observatory/activity/staging/chunk-activity");
        let staging_chunk_changed = mk_chunk_staging("observatory/activity/staging/chunk-changed");
        let staging_chunk_stable = mk_chunk_staging("observatory/activity/staging/chunk-stable");

        let mut activity_prev_masks = Vec::with_capacity(n_chunks as usize);
        activity_prev_masks.resize(n_chunks as usize, ACTIVITY_NO_PREV_SAMPLE);

        Self {
            staging_material,
            staging_temperature,
            staging_flags,
            staging_pressure,
            pending: false,
            pending_tick: 0,
            receiver: None,
            metrics: ObservatoryMetrics::default(),
            pressure_metrics: PressureObservatoryMetrics::default(),
            integrity_metrics: IntegrityMetrics::default(),
            activity_metrics: ActivityMetrics::default(),
            activity_prev_masks,
            initial_wood_set: false,
            initial_a_ice: 0,
            initial_b_steam: 0,
            last_request_tick: 0,
            cell_bytes,
            c_latch_reported: false,
            g6_mode,
            activity_mode,
            chunk_bytes,
            staging_chunk_activity,
            staging_chunk_changed,
            staging_chunk_stable,
            initial_latched: false,
            c_staging_material,
            c_staging_temperature,
            c_staging_flags,
            c_staging_pressure,
        }
    }

    /// Resets all metrics and latches (invoked on 'R' key).
    pub fn reset(&mut self) {
        self.metrics = ObservatoryMetrics::default();
        self.pressure_metrics = PressureObservatoryMetrics::default();
        self.integrity_metrics = IntegrityMetrics::default();
        self.activity_metrics = ActivityMetrics::default();
        for m in &mut self.activity_prev_masks {
            *m = ACTIVITY_NO_PREV_SAMPLE;
        }
        self.initial_wood_set = false;
        self.initial_a_ice = 0;
        self.initial_b_steam = 0;
        self.last_request_tick = 0;
        self.c_latch_reported = false;
        self.initial_latched = false;
        // Clear pending async states
        if self.pending {
            self.staging_material.unmap();
            self.staging_temperature.unmap();
            self.staging_flags.unmap();
            self.staging_pressure.unmap();
            self.pending = false;
            self.receiver = None;
        }
    }

    /// Current live metrics snapshot.
    pub fn metrics(&self) -> &ObservatoryMetrics {
        &self.metrics
    }

    /// Current live pressure demo metrics snapshot.
    pub fn pressure_metrics(&self) -> &PressureObservatoryMetrics {
        &self.pressure_metrics
    }

    /// Current live parallel integrity metrics snapshot.
    pub fn integrity_metrics(&self) -> &IntegrityMetrics {
        &self.integrity_metrics
    }

    /// Current live G7-A chunk-activity metrics snapshot.
    pub fn activity_metrics(&self) -> &ActivityMetrics {
        &self.activity_metrics
    }

    /// Non-blocking update: checks pending map callbacks and requests next
    /// readback if due. `fast` is the demo fast-forward multiplier (1/4/16) —
    /// diagnostic sampling stays cheap during fast-forward.
    ///
    /// G6 instrument snapshots are blocking one-shots taken at exact ticks so
    /// the latched values are never smeared by async readback latency:
    ///   - tick 0 (pristine staged scene): Panel A/B conservation baseline
    ///   - tick 1 (first post-tick state): Panel C ownership latch
    pub fn update(&mut self, simulation: &Simulation, current_tick: u64, fast: u32) {
        let device = &simulation.context.device;

        if self.g6_mode && fast == 1 && !self.initial_latched && current_tick == 0 {
            self.snapshot_integrity_sync(simulation, 0);
            self.initial_latched = true;
        }
        // The exact tick-1 ownership latch is taken by
        // `latch_first_tick_if_g6` (called from the demo loop right after the
        // first tick) so the accumulator can never skip past tick 1.

        // 1. Check if previous map request finished
        if self.pending {
            let ready = if let Some(rx) = &self.receiver {
                match rx.try_recv() {
                    Ok(Ok(())) => true,
                    Ok(Err(_)) => {
                        self.pending = false;
                        self.receiver = None;
                        false
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        // Poll device non-blocking
                        let _ = device.poll(wgpu::PollType::Poll);
                        false
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.pending = false;
                        self.receiver = None;
                        false
                    }
                }
            } else {
                false
            };

            if ready {
                // Read mapped slices
                let mat_slice = self.staging_material.slice(..).get_mapped_range();
                let temp_slice = self.staging_temperature.slice(..).get_mapped_range();
                let flag_slice = self.staging_flags.slice(..).get_mapped_range();
                let press_slice = self.staging_pressure.slice(..).get_mapped_range();

                let width = simulation.world.config.width;
                let height = simulation.world.config.height;
                let cell_count = (width * height) as usize;

                let materials = bytemuck_u32_slice(&mat_slice, cell_count);
                let temperatures = bytemuck_f32_slice(&temp_slice, cell_count);
                let flags = bytemuck_u32_slice(&flag_slice, cell_count);
                let pressures = bytemuck_f32_slice(&press_slice, cell_count);

                evaluate_observatory_state(
                    materials,
                    temperatures,
                    flags,
                    width,
                    height,
                    self.pending_tick,
                    &mut self.metrics,
                    &mut self.initial_a_ice,
                    &mut self.initial_b_steam,
                    &mut self.initial_wood_set,
                );

                evaluate_pressure_observatory_state(
                    materials,
                    temperatures,
                    pressures,
                    width,
                    height,
                    self.pending_tick,
                    &mut self.pressure_metrics,
                );

                evaluate_integrity_state(
                    materials,
                    temperatures,
                    flags,
                    pressures,
                    width,
                    height,
                    self.pending_tick,
                    &mut self.integrity_metrics,
                );

                if self.activity_mode {
                    // G7-A chunk-activity readbacks (small per-chunk arrays).
                    let act_slice = self.staging_chunk_activity.slice(..).get_mapped_range();
                    let chg_slice = self.staging_chunk_changed.slice(..).get_mapped_range();
                    let stb_slice = self.staging_chunk_stable.slice(..).get_mapped_range();
                    let chunks_x = chunks_x(
                        simulation.world.config.width,
                        simulation.world.config.chunk_size,
                    );
                    let chunks_y = chunks_y(
                        simulation.world.config.height,
                        simulation.world.config.chunk_size,
                    );
                    let n_chunks = (chunks_x * chunks_y) as usize;
                    let chunk_act = bytemuck_u32_slice(&act_slice, n_chunks);
                    let chunk_stb = bytemuck_u32_slice(&stb_slice, n_chunks);
                    evaluate_activity_state(
                        chunk_act,
                        chunk_stb,
                        chunks_x,
                        chunks_y,
                        self.pending_tick,
                        &mut self.activity_metrics,
                        &mut self.activity_prev_masks,
                    );
                    let _ = chg_slice;
                    drop(act_slice);
                    drop(chg_slice);
                    drop(stb_slice);
                    self.staging_chunk_activity.unmap();
                    self.staging_chunk_changed.unmap();
                    self.staging_chunk_stable.unmap();
                }

                drop(mat_slice);
                drop(temp_slice);
                drop(flag_slice);
                drop(press_slice);

                self.staging_material.unmap();
                self.staging_temperature.unmap();
                self.staging_flags.unmap();
                self.staging_pressure.unmap();

                self.pending = false;
                self.receiver = None;

                self.report_c_latch();
            }
        }

        // 2. If idle, check if we should request a new diagnostic readback.
        //    Tick 0 (pristine staged scene) is sampled unless the G6 instrument
        //    already captured it synchronously. Afterwards the cadence widens
        //    with the fast-forward multiplier so readbacks never throttle the
        //    sim loop. (The G6 Panel C latch is taken by the synchronous tick-1
        //    snapshot above, so no async special-case is needed.)
        let cadence = if fast >= 16 {
            30
        } else if fast >= 4 {
            12
        } else {
            5
        };
        let need_tick0 = current_tick == 0 && !(self.g6_mode && self.initial_latched);
        if !self.pending && (need_tick0 || current_tick >= self.last_request_tick + cadence) {
            self.request_readback(simulation, current_tick);
        }
    }

    /// G6 instrument: called by the demo loop immediately after every tick.
    /// The exact tick-1 snapshot (first post-tick state) is taken right here —
    /// before any later tick can run — so the ownership latch is never
    /// smeared by frame accumulation or async readback latency.
    pub fn latch_first_tick_if_g6(&mut self, simulation: &Simulation, tick: u64, fast: u32) {
        if self.g6_mode && fast == 1 && !self.integrity_metrics.c_latched && tick == 1 {
            self.snapshot_integrity_sync(simulation, 1);
        }
    }

    /// Blocking one-shot GPU snapshot for the G6 instrument: copies the
    /// authoritative Current buffers and evaluates the integrity metrics at
    /// exactly `tick`. Used for the pristine tick-0 baseline and the tick-1
    /// ownership latch so async readback latency never smears the latched
    /// values (one-shot, a few ms each).
    fn snapshot_integrity_sync(&mut self, simulation: &Simulation, tick: u64) {
        let device = &simulation.context.device;
        let queue = &simulation.context.queue;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("observatory/g6-instrument-copy-encoder"),
        });
        encoder.copy_buffer_to_buffer(
            &simulation.world.material_current,
            0,
            &self.c_staging_material,
            0,
            self.cell_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &simulation.world.temperature_current,
            0,
            &self.c_staging_temperature,
            0,
            self.cell_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &simulation.world.flags_current,
            0,
            &self.c_staging_flags,
            0,
            self.cell_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &simulation.world.pressure_current,
            0,
            &self.c_staging_pressure,
            0,
            self.cell_bytes,
        );
        queue.submit([encoder.finish()]);

        for b in [
            &self.c_staging_material,
            &self.c_staging_temperature,
            &self.c_staging_flags,
            &self.c_staging_pressure,
        ] {
            b.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        }
        let _ = device.poll(wgpu::PollType::Wait); // blocking map completion

        let mat_slice = self.c_staging_material.slice(..).get_mapped_range();
        let temp_slice = self.c_staging_temperature.slice(..).get_mapped_range();
        let flag_slice = self.c_staging_flags.slice(..).get_mapped_range();
        let press_slice = self.c_staging_pressure.slice(..).get_mapped_range();

        let width = simulation.world.config.width;
        let height = simulation.world.config.height;
        let cell_count = (width * height) as usize;
        let materials = bytemuck_u32_slice(&mat_slice, cell_count);
        let temperatures = bytemuck_f32_slice(&temp_slice, cell_count);
        let flags = bytemuck_u32_slice(&flag_slice, cell_count);
        let pressures = bytemuck_f32_slice(&press_slice, cell_count);

        evaluate_integrity_state(
            materials,
            temperatures,
            flags,
            pressures,
            width,
            height,
            tick,
            &mut self.integrity_metrics,
        );

        drop(mat_slice);
        drop(temp_slice);
        drop(flag_slice);
        drop(press_slice);
        self.c_staging_material.unmap();
        self.c_staging_temperature.unmap();
        self.c_staging_flags.unmap();
        self.c_staging_pressure.unmap();

        self.report_c_latch();
    }

    /// Prints the latched G6-C ownership evidence once (actual GPU readback
    /// numbers, never hardcoded expectations).
    fn report_c_latch(&mut self) {
        if !self.g6_mode || !self.integrity_metrics.c_latched || self.c_latch_reported {
            return;
        }
        self.c_latch_reported = true;
        let m = &self.integrity_metrics;
        println!(
            "[powdergame][G6-C] latch @tick {}: expansion candidates={} winners={} \
             steam_sources={}/3 pressure_losers={} target={}; \
             smoke candidates={} winners={} wood_preserved={}/3 smoke_age={} target={}; \
             movement_done={} scratch_reuse={} result={}",
            m.tick,
            m.c_exp_candidates,
            m.c_exp_winners,
            m.c_exp_steam_sources,
            m.c_exp_pressure_losers,
            if m.c_exp_target_steam { "STEAM" } else { "?" },
            m.c_smoke_candidates,
            m.c_smoke_winners,
            m.c_smoke_wood_preserved,
            m.c_smoke_age,
            if m.c_smoke_target_smoke { "SMOKE" } else { "?" },
            m.c_move_done,
            m.c_scratch_reuse,
            m.c_result,
        );
    }

    fn request_readback(&mut self, simulation: &Simulation, current_tick: u64) {
        let device = &simulation.context.device;
        let queue = &simulation.context.queue;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("observatory/diagnostic-copy-encoder"),
        });

        encoder.copy_buffer_to_buffer(
            &simulation.world.material_current,
            0,
            &self.staging_material,
            0,
            self.cell_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &simulation.world.temperature_current,
            0,
            &self.staging_temperature,
            0,
            self.cell_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &simulation.world.flags_current,
            0,
            &self.staging_flags,
            0,
            self.cell_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &simulation.world.pressure_current,
            0,
            &self.staging_pressure,
            0,
            self.cell_bytes,
        );
        if self.activity_mode {
            encoder.copy_buffer_to_buffer(
                &simulation.world.chunk_activity,
                0,
                &self.staging_chunk_activity,
                0,
                self.chunk_bytes,
            );
            encoder.copy_buffer_to_buffer(
                &simulation.world.chunk_changed_this_tick,
                0,
                &self.staging_chunk_changed,
                0,
                self.chunk_bytes,
            );
            encoder.copy_buffer_to_buffer(
                &simulation.world.chunk_stable_ticks,
                0,
                &self.staging_chunk_stable,
                0,
                self.chunk_bytes,
            );
        }

        queue.submit([encoder.finish()]);

        let (tx, rx) = std::sync::mpsc::channel();
        let tx_temp = tx.clone();
        let tx_flag = tx.clone();
        let tx_press = tx.clone();
        let tx_act = tx.clone();
        let tx_chg = tx.clone();
        let tx_stb = tx.clone();

        // Async mapping on all 4 world buffers
        self.staging_material
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |res| {
                let _ = tx.send(res);
            });
        self.staging_temperature
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |res| {
                let _ = tx_temp.send(res);
            });
        self.staging_flags
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |res| {
                let _ = tx_flag.send(res);
            });
        self.staging_pressure
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |res| {
                let _ = tx_press.send(res);
            });
        if self.activity_mode {
            self.staging_chunk_activity
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |res| {
                    let _ = tx_act.send(res);
                });
            self.staging_chunk_changed
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |res| {
                    let _ = tx_chg.send(res);
                });
            self.staging_chunk_stable
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |res| {
                    let _ = tx_stb.send(res);
                });
        }

        self.pending = true;
        self.pending_tick = current_tick;
        self.last_request_tick = current_tick;
        self.receiver = Some(rx);
    }
}

fn bytemuck_u32_slice(bytes: &[u8], count: usize) -> &[u32] {
    let raw_ptr = bytes.as_ptr() as *const u32;
    unsafe { std::slice::from_raw_parts(raw_ptr, count.min(bytes.len() / 4)) }
}

fn bytemuck_f32_slice(bytes: &[u8], count: usize) -> &[f32] {
    let raw_ptr = bytes.as_ptr() as *const f32;
    unsafe { std::slice::from_raw_parts(raw_ptr, count.min(bytes.len() / 4)) }
}

/// Live diagnostic metrics for the G6 Parallel Integrity Lab (2×2 layout).
///
/// Every value is computed from a real GPU readback of the authoritative
/// Current buffers (material / temperature / flags / pressure) — nothing is
/// a hardcoded expectation. Panel C is a one-tick ownership instrument whose
/// first-tick result is latched and preserved for the rest of the session.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct IntegrityMetrics {
    pub tick: u64,

    // Panel A — MOVEMENT CONTENTION (closed fixture: conservation required)
    pub a_matter_count: u32,
    pub a_initial_matter: u32,
    pub a_matter_delta: i32,
    pub a_invalid: u32,

    // Panel B — CHUNK BOUNDARY (closed fixture: conservation required)
    pub b_cross_chunk_matter: u32,
    pub b_initial_matter: u32,
    pub b_matter_delta: i32,
    pub b_invalid_material: u32,
    /// Matter currently occupying the seam columns/rows (x 191..192, y 63..64)
    /// — live evidence that the chunk boundary is not a wall.
    pub b_crossings: u32,

    // Panel C — EXPANSION + SMOKE OWNERSHIP (one-tick instrument, latched)
    pub c_latched: bool,
    pub c_exp_candidates: u32,
    pub c_exp_winners: u32,
    pub c_exp_steam_sources: u32,
    pub c_exp_pressure_losers: u32,
    pub c_exp_target_steam: bool,
    pub c_smoke_candidates: u32,
    pub c_smoke_winners: u32,
    pub c_smoke_wood_preserved: u32,
    pub c_smoke_age: u32,
    pub c_smoke_target_smoke: bool,
    pub c_move_done: bool,
    pub c_scratch_reuse: bool,
    pub c_result: bool,

    // Panel D — HEAVY MIXED STRESS (integrity violations, D region only)
    pub d_total_matter: u32,
    pub d_invalid_material_ids: u32,
    pub d_nan_inf_temperature: u32,
    pub d_nan_inf_pressure: u32,
    pub d_negative_pressure: u32,
    pub d_empty_temp_violations: u32,
    pub d_empty_flag_violations: u32,
    pub d_empty_pressure_violations: u32,
}

/// G7-A per-panel chunk-activity counters (one quadrant of the 4×4-chunk
/// demo world; chunk regions, not cell scans).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActivityPanelMetrics {
    pub total_chunks: u32,
    pub matter_active: u32,
    pub thermal_active: u32,
    pub pressure_active: u32,
    pub reaction_active: u32,
    pub fully_stable: u32,
    pub max_stable_ticks: u32,
}

/// Sentinel marking "no previous sample yet" in the per-chunk previous-mask
/// baseline: the first diagnostic sample only ESTABLISHES the baseline and
/// must never be counted as a stable→active transition.
pub const ACTIVITY_NO_PREV_SAMPLE: u32 = u32::MAX;

/// G7-A global chunk-activity observation metrics, computed from real GPU
/// readback of `chunk_activity` / `chunk_stable_ticks` (never hardcoded).
///
/// - `fully_stable`: chunks whose mask is 0 this sample (no frontier).
/// - `max_stable_ticks`: longest consecutive zero-activity run observed.
/// - `sampled_wake_candidates`: chunks that transitioned stable→active
///   between diagnostic samples (derived from samples only — NOT an
///   exhaustive event stream and NOT actual G7-B wake execution). The first
///   sample only establishes the baseline and never increments this count.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActivityMetrics {
    pub sample_tick: u64,
    pub total_chunks: u32,
    pub matter_active: u32,
    pub thermal_active: u32,
    pub pressure_active: u32,
    pub reaction_active: u32,
    pub fully_stable: u32,
    pub max_stable_ticks: u32,
    pub sampled_wake_candidates: u32,
    pub panels: [ActivityPanelMetrics; 4],
}

/// G7-A demo panel names (world 256×256, chunk 64 → 4×4 chunks).
pub const ACTIVITY_PANEL_NAMES: [&str; 4] = [
    "A STABLE WATER BULK",
    "B STABLE STEAM / GAS BULK",
    "C STABLE DURATION / WAKE CANDIDATE",
    "D SLOW ACTIVE WORLD",
];

/// Aggregates per-chunk activity/stability into global + 4-quadrant panel
/// metrics and counts stable→active wake transitions against the previous
/// sample's masks.
#[allow(clippy::too_many_arguments)] // diagnostic pure fn: 5 slices/dims + metrics + prev
pub fn evaluate_activity_state(
    chunk_activity: &[u32],
    chunk_stable: &[u32],
    chunks_x: u32,
    chunks_y: u32,
    tick: u64,
    metrics: &mut ActivityMetrics,
    prev_masks: &mut [u32],
) {
    metrics.sample_tick = tick;
    metrics.total_chunks = chunks_x * chunks_y;
    metrics.matter_active = 0;
    metrics.thermal_active = 0;
    metrics.pressure_active = 0;
    metrics.reaction_active = 0;
    metrics.fully_stable = 0;
    metrics.max_stable_ticks = 0;
    metrics.panels = [
        ActivityPanelMetrics::default(),
        ActivityPanelMetrics::default(),
        ActivityPanelMetrics::default(),
        ActivityPanelMetrics::default(),
    ];

    let half_x = chunks_x / 2;
    let half_y = chunks_y / 2;

    for cy in 0..chunks_y {
        for cx in 0..chunks_x {
            let idx = (cy * chunks_x + cx) as usize;
            let mask = chunk_activity.get(idx).copied().unwrap_or(0);
            let stable = chunk_stable.get(idx).copied().unwrap_or(0);

            let panel = if cy < half_y {
                if cx < half_x {
                    0
                } else {
                    1
                }
            } else if cx < half_x {
                2
            } else {
                3
            };
            let p = &mut metrics.panels[panel];
            p.total_chunks += 1;
            p.matter_active += u32::from(mask & ACTIVITY_MATTER != 0);
            p.thermal_active += u32::from(mask & ACTIVITY_THERMAL != 0);
            p.pressure_active += u32::from(mask & ACTIVITY_PRESSURE != 0);
            p.reaction_active += u32::from(mask & ACTIVITY_REACTION != 0);
            if mask == 0 {
                p.fully_stable += 1;
            }
            p.max_stable_ticks = p.max_stable_ticks.max(stable);

            metrics.matter_active += u32::from(mask & ACTIVITY_MATTER != 0);
            metrics.thermal_active += u32::from(mask & ACTIVITY_THERMAL != 0);
            metrics.pressure_active += u32::from(mask & ACTIVITY_PRESSURE != 0);
            metrics.reaction_active += u32::from(mask & ACTIVITY_REACTION != 0);
            if mask == 0 {
                metrics.fully_stable += 1;
            }
            metrics.max_stable_ticks = metrics.max_stable_ticks.max(stable);

            // Stable → active between samples = a wake candidate observation.
            // The FIRST sample (sentinel prev) only establishes the baseline;
            // only a later sampled 0 → nonzero transition counts.
            if let Some(prev) = prev_masks.get_mut(idx) {
                if *prev != ACTIVITY_NO_PREV_SAMPLE && *prev == 0 && mask != 0 {
                    metrics.sampled_wake_candidates += 1;
                }
                *prev = mask;
            }
        }
    }
}

/// Evaluates the G6 integrity diagnostics from one GPU readback snapshot.
///
/// `tick` is the simulation tick at which the snapshot was captured
/// (`pending_tick`). Panel A/B initials are latched on the tick-0 snapshot
/// (the pristine staged scene); Panel C is latched on the FIRST snapshot with
/// `tick >= 1` and then preserved.
#[allow(clippy::too_many_arguments)] // 8 args: 4 state slices + dims + tick + metrics (diagnostic pure fn)
pub fn evaluate_integrity_state(
    materials: &[u32],
    temperatures: &[f32],
    flags: &[u32],
    pressures: &[f32],
    width: u32,
    height: u32,
    tick: u64,
    metrics: &mut IntegrityMetrics,
) {
    metrics.tick = tick;

    let mut a_matter = 0u32;
    let mut a_invalid = 0u32;
    let mut b_matter = 0u32;
    let mut b_invalid = 0u32;
    let mut b_crossings = 0u32;
    let mut c_invalid = 0u32;
    let mut c_nan_inf_t = 0u32;
    let mut c_nan_inf_p = 0u32;
    let mut c_empty_t = 0u32;
    let mut c_empty_f = 0u32;
    let mut c_empty_p = 0u32;
    let mut d_matter = 0u32;
    let mut d_invalid = 0u32;
    let mut d_nan_inf_t = 0u32;
    let mut d_nan_inf_p = 0u32;
    let mut d_neg_p = 0u32;
    let mut d_empty_t = 0u32;
    let mut d_empty_f = 0u32;
    let mut d_empty_p = 0u32;

    let mut exp_steam_sources = 0u32;
    let mut exp_source_pressures = [0.0f32; 3];
    let mut exp_target_steam = false;
    let mut smoke_candidates = 0u32;
    let mut smoke_wood_preserved = 0u32;
    let mut smoke_age = 0u32;
    let mut smoke_target_smoke = false;
    let mut move_src_empty = false;
    let mut move_dst_sand = false;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if idx >= materials.len() {
                continue;
            }
            let mat = materials[idx];
            let temp = temperatures.get(idx).copied().unwrap_or(0.0);
            let f = flags.get(idx).copied().unwrap_or(0);
            let p = pressures.get(idx).copied().unwrap_or(0.0);

            // Registered Matter ids are 2..=9 (EMPTY 0 and Boundary 1 are not
            // countable Matter); anything above 9 is corruption, not Matter.
            let is_matter = (2..=9).contains(&mat);
            let is_invalid = mat > 9;

            let in_a =
                (G6_A_X_MIN..=G6_A_X_MAX).contains(&x) && (G6_A_Y_MIN..=G6_A_Y_MAX).contains(&y);
            let in_b =
                (G6_B_X_MIN..=G6_B_X_MAX).contains(&x) && (G6_B_Y_MIN..=G6_B_Y_MAX).contains(&y);
            let in_c =
                (G6_C_X_MIN..=G6_C_X_MAX).contains(&x) && (G6_C_Y_MIN..=G6_C_Y_MAX).contains(&y);
            let in_d =
                (G6_D_X_MIN..=G6_D_X_MAX).contains(&x) && (G6_D_Y_MIN..=G6_D_Y_MAX).contains(&y);

            if in_a {
                if is_matter {
                    a_matter += 1;
                }
                if is_invalid {
                    a_invalid += 1;
                }
            }
            if in_b {
                if is_matter {
                    b_matter += 1;
                    // Live seam activity at the 64×64 chunk boundary (x 191/192
                    // seam columns, y 63/64 seam rows within panel B).
                    if x == 191 || x == 192 || y == 63 || y == 64 {
                        b_crossings += 1;
                    }
                }
                if is_invalid {
                    b_invalid += 1;
                }
            }
            if in_c {
                if is_invalid {
                    c_invalid += 1;
                }
                if temp.is_nan() || temp.is_infinite() {
                    c_nan_inf_t += 1;
                }
                if p.is_nan() || p.is_infinite() {
                    c_nan_inf_p += 1;
                }
                if mat == MATERIAL_EMPTY {
                    if temp != 0.0 {
                        c_empty_t += 1;
                    }
                    if f != 0 {
                        c_empty_f += 1;
                    }
                    if p != 0.0 {
                        c_empty_p += 1;
                    }
                }
                // One-tick instrument: read the fixture cells directly.
                if let Some(si) = EXP_SOURCES.iter().position(|&s| s == (x, y)) {
                    if mat == MATERIAL_STEAM {
                        exp_steam_sources += 1;
                    }
                    exp_source_pressures[si] = p;
                }
                if (x, y) == EXP_TARGET {
                    exp_target_steam = mat == MATERIAL_STEAM;
                }
                if SMOKE_SOURCES.contains(&(x, y)) {
                    if (f & FLAG_COMBUSTING) != 0 {
                        smoke_candidates += 1;
                    }
                    if mat == MATERIAL_WOOD {
                        smoke_wood_preserved += 1;
                    }
                }
                if (x, y) == SMOKE_TARGET {
                    smoke_target_smoke = mat == MATERIAL_SMOKE;
                    smoke_age = (f & FLAG_DECAY_AGE_MASK) >> FLAG_DECAY_AGE_SHIFT;
                }
                if (x, y) == MOVE_SRC {
                    move_src_empty = mat == MATERIAL_EMPTY;
                }
                if (x, y) == MOVE_DST {
                    move_dst_sand = mat == MATERIAL_SAND;
                }
            }
            if in_d {
                if is_matter {
                    d_matter += 1;
                }
                if is_invalid {
                    d_invalid += 1;
                }
                if temp.is_nan() || temp.is_infinite() {
                    d_nan_inf_t += 1;
                }
                if p.is_nan() || p.is_infinite() {
                    d_nan_inf_p += 1;
                }
                if p < 0.0 {
                    d_neg_p += 1;
                }
                if mat == MATERIAL_EMPTY {
                    if temp != 0.0 {
                        d_empty_t += 1;
                    }
                    if f != 0 {
                        d_empty_f += 1;
                    }
                    if p != 0.0 {
                        d_empty_p += 1;
                    }
                }
            }
        }
    }

    // Latch the pristine staged scene (tick 0) as the closed-fixture baseline.
    if tick == 0 {
        metrics.a_initial_matter = a_matter;
        metrics.b_initial_matter = b_matter;
    }

    let move_done = move_src_empty && move_dst_sand;

    // Confinement losers = sources whose pressure exceeds the minimum source
    // pressure (the claim winner, which carries only diffused pressure).
    let min_exp_pressure = exp_source_pressures
        .iter()
        .copied()
        .fold(f32::INFINITY, f32::min);
    let exp_pressure_losers = if min_exp_pressure.is_finite() {
        exp_source_pressures
            .iter()
            .filter(|&&p| p > min_exp_pressure)
            .count() as u32
    } else {
        0
    };

    // Live A/B/D numbers.
    metrics.a_matter_count = a_matter;
    metrics.a_invalid = a_invalid;
    metrics.a_matter_delta = (a_matter as i64 - metrics.a_initial_matter as i64) as i32;

    metrics.b_cross_chunk_matter = b_matter;
    metrics.b_invalid_material = b_invalid;
    metrics.b_crossings = b_crossings;
    metrics.b_matter_delta = (b_matter as i64 - metrics.b_initial_matter as i64) as i32;

    metrics.d_total_matter = d_matter;
    metrics.d_invalid_material_ids = d_invalid;
    metrics.d_nan_inf_temperature = d_nan_inf_t;
    metrics.d_nan_inf_pressure = d_nan_inf_p;
    metrics.d_negative_pressure = d_neg_p;
    metrics.d_empty_temp_violations = d_empty_t;
    metrics.d_empty_flag_violations = d_empty_f;
    metrics.d_empty_pressure_violations = d_empty_p;

    // Panel C: latch the FIRST post-tick snapshot (tick >= 1) and preserve it.
    if !metrics.c_latched && tick >= 1 {
        let exp_winners = u32::from(exp_target_steam);
        let smoke_winners = u32::from(smoke_target_smoke);
        let hygiene_ok = c_invalid == 0
            && c_nan_inf_t == 0
            && c_nan_inf_p == 0
            && c_empty_t == 0
            && c_empty_f == 0
            && c_empty_p == 0;
        let scratch_reuse = move_done
            && exp_target_steam
            && smoke_target_smoke
            && smoke_wood_preserved == SMOKE_SOURCES.len() as u32
            && hygiene_ok;
        let result = exp_steam_sources == EXP_SOURCES.len() as u32
            && exp_winners == 1
            && exp_pressure_losers >= 2
            && smoke_candidates == SMOKE_SOURCES.len() as u32
            && smoke_wood_preserved == SMOKE_SOURCES.len() as u32
            && smoke_winners == 1
            && smoke_age == 0
            && move_done
            && scratch_reuse;

        metrics.c_exp_candidates = exp_steam_sources;
        metrics.c_exp_winners = exp_winners;
        metrics.c_exp_steam_sources = exp_steam_sources;
        metrics.c_exp_pressure_losers = exp_pressure_losers;
        metrics.c_exp_target_steam = exp_target_steam;
        metrics.c_smoke_candidates = smoke_candidates;
        metrics.c_smoke_winners = smoke_winners;
        metrics.c_smoke_wood_preserved = smoke_wood_preserved;
        metrics.c_smoke_age = smoke_age;
        metrics.c_smoke_target_smoke = smoke_target_smoke;
        metrics.c_move_done = move_done;
        metrics.c_scratch_reuse = scratch_reuse;
        metrics.c_result = result;
        metrics.c_latched = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use powdergame_core::{MATERIAL_OIL, MATERIAL_STONE};

    #[test]
    fn test_panel_classification_and_metrics_accumulation() {
        let width = 320u32;
        let height = 192u32;
        let cell_count = (width * height) as usize;

        let mut materials = vec![MATERIAL_EMPTY; cell_count];
        let mut temperatures = vec![0.0f32; cell_count];
        let mut flags = vec![0u32; cell_count];

        // Panel A: Place 10 Ice cells
        for x in 30..40 {
            let idx = (50 * width + x) as usize;
            materials[idx] = MATERIAL_ICE;
        }

        // Panel B: Place 20 Steam cells
        for x in 200..220 {
            let idx = (20 * width + x) as usize;
            materials[idx] = MATERIAL_STEAM;
            temperatures[idx] = 80.0;
        }

        // Panel C: Water tube mid band (temp = 25.0), Oil tube mid band (temp = 5.0)
        for x in TUBE_WATER_X_MIN..=TUBE_WATER_X_MAX {
            for y in TUBE_MID_Y_MIN..=TUBE_MID_Y_MAX {
                let idx = (y * width + x) as usize;
                materials[idx] = MATERIAL_WATER;
                temperatures[idx] = 25.0;
            }
        }
        for x in TUBE_OIL_X_MIN..=TUBE_OIL_X_MAX {
            for y in TUBE_MID_Y_MIN..=TUBE_MID_Y_MAX {
                let idx = (y * width + x) as usize;
                materials[idx] = MATERIAL_OIL;
                temperatures[idx] = 5.0;
            }
        }

        // Panel D: Place 50 Wood cells, 5 of which are combusting
        for x in 200..250 {
            let idx = (150 * width + x) as usize;
            materials[idx] = MATERIAL_WOOD;
            if x < 205 {
                flags[idx] = FLAG_COMBUSTING;
            }
        }

        let mut metrics = ObservatoryMetrics::default();
        let mut initial_a_ice = 0;
        let mut initial_b_steam = 0;
        let mut initial_wood_set = false;

        // Tick 0 baseline evaluation
        evaluate_observatory_state(
            &materials,
            &temperatures,
            &flags,
            width,
            height,
            0,
            &mut metrics,
            &mut initial_a_ice,
            &mut initial_b_steam,
            &mut initial_wood_set,
        );

        assert_eq!(metrics.a_ice_count, 10);
        assert_eq!(metrics.a_water_count, 0);
        assert_eq!(metrics.a_steam_count, 0);
        assert_eq!(metrics.a_first_melt, None);

        assert_eq!(metrics.b_steam_count, 20);
        assert_eq!(metrics.b_water_count, 0);
        assert_eq!(metrics.b_ice_count, 0);
        assert_eq!(metrics.b_first_condense, None);

        assert_eq!(metrics.c_w_mid_t, 25.0);
        assert_eq!(metrics.c_o_mid_t, 5.0);
        assert_eq!(metrics.c_w_mid_reach, None); // Not latched at tick 0

        assert_eq!(metrics.d_wood_start, 50);
        assert_eq!(metrics.d_wood_left, 50);
        assert_eq!(metrics.d_burning, 5);
        assert_eq!(metrics.d_first_ignite, None); // Not latched at tick 0
    }

    #[test]
    fn test_first_event_tick_latching_and_permanence() {
        let width = 320u32;
        let height = 192u32;
        let cell_count = (width * height) as usize;

        let mut materials = vec![MATERIAL_EMPTY; cell_count];
        let temperatures = vec![0.0f32; cell_count];
        let mut flags = vec![0u32; cell_count];

        // Initial setup
        materials[(50 * width + 30) as usize] = MATERIAL_ICE;
        materials[(20 * width + 200) as usize] = MATERIAL_STEAM;
        materials[(150 * width + 200) as usize] = MATERIAL_WOOD;
        flags[(150 * width + 200) as usize] = FLAG_COMBUSTING;

        let mut metrics = ObservatoryMetrics::default();
        let mut initial_a_ice = 0;
        let mut initial_b_steam = 0;
        let mut initial_wood_set = false;

        // Tick 0 baseline
        evaluate_observatory_state(
            &materials,
            &temperatures,
            &flags,
            width,
            height,
            0,
            &mut metrics,
            &mut initial_a_ice,
            &mut initial_b_steam,
            &mut initial_wood_set,
        );

        // Tick 10: Ice melts to water, Steam condenses, Wood ignites
        materials[(50 * width + 30) as usize] = MATERIAL_WATER; // melted
        materials[(20 * width + 200) as usize] = MATERIAL_WATER; // condensed
        evaluate_observatory_state(
            &materials,
            &temperatures,
            &flags,
            width,
            height,
            10,
            &mut metrics,
            &mut initial_a_ice,
            &mut initial_b_steam,
            &mut initial_wood_set,
        );

        assert_eq!(metrics.a_first_melt, Some(10));
        assert_eq!(metrics.b_first_condense, Some(10));
        assert_eq!(metrics.d_first_ignite, Some(10));

        // Tick 50: Next frame should preserve original tick 10 latches!
        evaluate_observatory_state(
            &materials,
            &temperatures,
            &flags,
            width,
            height,
            50,
            &mut metrics,
            &mut initial_a_ice,
            &mut initial_b_steam,
            &mut initial_wood_set,
        );

        assert_eq!(metrics.a_first_melt, Some(10));
        assert_eq!(metrics.b_first_condense, Some(10));
        assert_eq!(metrics.d_first_ignite, Some(10));
    }

    #[test]
    fn test_heat_comparison_warm_reach_threshold() {
        let width = 320u32;
        let height = 192u32;
        let cell_count = (width * height) as usize;

        let mut materials = vec![MATERIAL_EMPTY; cell_count];
        let mut temperatures = vec![0.0f32; cell_count];
        let flags = vec![0u32; cell_count];

        let mut metrics = ObservatoryMetrics::default();
        let mut initial_a_ice = 0;
        let mut initial_b_steam = 0;
        let mut initial_wood_set = false;

        // Below threshold at tick 5 (temp = 8.0 < 10.0)
        for x in TUBE_WATER_X_MIN..=TUBE_WATER_X_MAX {
            for y in TUBE_MID_Y_MIN..=TUBE_MID_Y_MAX {
                let idx = (y * width + x) as usize;
                materials[idx] = MATERIAL_WATER;
                temperatures[idx] = 8.0;
            }
        }
        evaluate_observatory_state(
            &materials,
            &temperatures,
            &flags,
            width,
            height,
            5,
            &mut metrics,
            &mut initial_a_ice,
            &mut initial_b_steam,
            &mut initial_wood_set,
        );
        assert_eq!(metrics.c_w_mid_reach, None);

        // Crosses threshold at tick 25 (temp = 12.5 >= 10.0)
        for x in TUBE_WATER_X_MIN..=TUBE_WATER_X_MAX {
            for y in TUBE_MID_Y_MIN..=TUBE_MID_Y_MAX {
                let idx = (y * width + x) as usize;
                temperatures[idx] = 12.5;
            }
        }
        evaluate_observatory_state(
            &materials,
            &temperatures,
            &flags,
            width,
            height,
            25,
            &mut metrics,
            &mut initial_a_ice,
            &mut initial_b_steam,
            &mut initial_wood_set,
        );
        assert_eq!(metrics.c_w_mid_reach, Some(25));
    }

    #[test]
    fn test_pressure_observatory_evaluation() {
        let width = 256u32;
        let height = 256u32;
        let cell_count = (width * height) as usize;

        let mut materials = vec![MATERIAL_EMPTY; cell_count];
        let temperatures = vec![58.0f32; cell_count];
        let mut pressures = vec![0.0f32; cell_count];

        // Panel A (Top-Left): Place Wood plug (9 cells at y=44, x=60..68) and high pressure
        for x in 60..=68 {
            let idx = (44 * width + x) as usize;
            materials[idx] = MATERIAL_WOOD;
        }
        for x in 30..=40 {
            for y in 50..=60 {
                let idx = (y * width + x) as usize;
                materials[idx] = MATERIAL_WATER;
                pressures[idx] = 85.0;
            }
        }

        // Panel C (Bottom-Left): 9 Wood cells, 120.0 pressure
        for x in 60..=68 {
            let idx = (170 * width + x) as usize;
            materials[idx] = MATERIAL_WOOD;
        }
        for x in 30..=40 {
            for y in 180..=190 {
                let idx = (y * width + x) as usize;
                materials[idx] = MATERIAL_WATER;
                pressures[idx] = 120.0;
            }
        }

        // Panel D (Bottom-Right): 9 Wood cells in weak seam, 250.0 pressure
        for y in 214..=222 {
            let idx = (y * width + 242) as usize;
            materials[idx] = MATERIAL_WOOD;
        }
        for x in 160..=180 {
            for y in 180..=200 {
                let idx = (y * width + x) as usize;
                materials[idx] = MATERIAL_WATER;
                pressures[idx] = 250.0;
            }
        }

        let mut metrics = PressureObservatoryMetrics::default();

        evaluate_pressure_observatory_state(
            &materials,
            &temperatures,
            &pressures,
            width,
            height,
            0,
            &mut metrics,
        );

        assert_eq!(metrics.tl_wood_remaining, 9);
        assert_eq!(metrics.tl_peak_pressure, 85.0);
        assert_eq!(metrics.tl_relief_tick, None);

        assert_eq!(metrics.bl_wood_remaining, 9);
        assert_eq!(metrics.bl_peak_pressure, 120.0);
        assert_eq!(metrics.bl_relief_tick, None);

        assert_eq!(metrics.br_weak_seam_remaining, 9);
        assert_eq!(metrics.br_peak_pressure, 250.0);
        assert_eq!(metrics.br_rupture_tick, None);
        assert_eq!(metrics.br_first_vent_tick, None);

        // Tick 40: Bottom-Left relief opens (wood -> empty)
        for x in 60..=68 {
            let idx = (170 * width + x) as usize;
            materials[idx] = MATERIAL_EMPTY;
        }
        evaluate_pressure_observatory_state(
            &materials,
            &temperatures,
            &pressures,
            width,
            height,
            40,
            &mut metrics,
        );
        assert_eq!(metrics.bl_wood_remaining, 0);
        assert_eq!(metrics.bl_relief_tick, Some(40));

        // Tick 80: Bottom-Right seam ruptures (wood -> empty at y=222) and steam vents
        let breach_idx = (222 * width + 242) as usize;
        materials[breach_idx] = MATERIAL_EMPTY;
        let p_left_idx = (222 * width + 241) as usize;
        pressures[p_left_idx] = 84.5;
        // Exterior duct steam
        let ext_idx = (220 * width + 245) as usize;
        materials[ext_idx] = MATERIAL_STEAM;

        evaluate_pressure_observatory_state(
            &materials,
            &temperatures,
            &pressures,
            width,
            height,
            80,
            &mut metrics,
        );
        assert_eq!(metrics.br_weak_seam_remaining, 8);
        assert_eq!(metrics.br_rupture_tick, Some(80));
        assert_eq!(metrics.br_breach_cell, Some((242, 222)));
        assert_eq!(metrics.br_breach_local_pressure, 84.5);
        assert_eq!(metrics.br_first_vent_tick, Some(80));
        assert_eq!(metrics.br_exterior_steam_count, 1);
    }

    #[test]
    fn test_integrity_a_b_conservation_and_initial_latch() {
        let width = 256u32;
        let height = 256u32;
        let cell_count = (width * height) as usize;

        let mut materials = vec![MATERIAL_EMPTY; cell_count];
        let temperatures = vec![0.0f32; cell_count];
        let flags = vec![0u32; cell_count];
        let pressures = vec![0.0f32; cell_count];

        // Panel A: 20 Sand, Panel B: 15 Water, one invalid id in each.
        for x in 10..30 {
            let idx = (50 * width + x) as usize;
            materials[idx] = MATERIAL_SAND;
        }
        for x in 140..155 {
            let idx = (50 * width + x) as usize;
            materials[idx] = MATERIAL_WATER;
        }
        materials[(60 * width + 5) as usize] = 42; // invalid in A
        materials[(60 * width + 200) as usize] = 42; // invalid in B

        let mut metrics = IntegrityMetrics::default();

        evaluate_integrity_state(
            &materials,
            &temperatures,
            &flags,
            &pressures,
            width,
            height,
            0,
            &mut metrics,
        );
        assert_eq!(metrics.a_initial_matter, 20);
        assert_eq!(metrics.b_initial_matter, 15);
        assert_eq!(metrics.a_invalid, 1);
        assert_eq!(metrics.b_invalid_material, 1);
        assert_eq!(metrics.a_matter_delta, 0);
        assert_eq!(metrics.b_matter_delta, 0);
        assert!(!metrics.c_latched, "C must not latch at tick 0");

        // Tick 5: one A cell left for B (crossed) — conservation per panel
        // would show -1/+1, but A/B are independent closed panels here, so
        // simulate genuine loss to prove the delta is readback-derived.
        materials[(50 * width + 10) as usize] = MATERIAL_EMPTY;
        evaluate_integrity_state(
            &materials,
            &temperatures,
            &flags,
            &pressures,
            width,
            height,
            5,
            &mut metrics,
        );
        assert_eq!(metrics.a_matter_count, 19);
        assert_eq!(metrics.a_matter_delta, -1);
    }

    #[test]
    fn test_integrity_d_hygiene_violations() {
        let width = 256u32;
        let height = 256u32;
        let cell_count = (width * height) as usize;

        let mut materials = vec![MATERIAL_EMPTY; cell_count];
        let mut temperatures = vec![0.0f32; cell_count];
        let mut flags = vec![0u32; cell_count];
        let mut pressures = vec![0.0f32; cell_count];

        // D region (bottom-right) violations:
        //   invalid id, NaN temperature, infinite pressure, negative pressure,
        //   EMPTY with T!=0, EMPTY with flags!=0, EMPTY with pressure!=0
        // NaN/Inf/negative fixtures sit on STONE so they do not double-count
        // into the EMPTY-hygiene categories.
        let d_x = 200u32;
        let d_y = 200u32;
        materials[(d_y * width + d_x) as usize] = 99; // invalid
        let nan_t = (210 * width + 150) as usize;
        materials[nan_t] = MATERIAL_STONE;
        temperatures[nan_t] = f32::NAN;
        let inf_p = (210 * width + 160) as usize;
        materials[inf_p] = MATERIAL_STONE;
        pressures[inf_p] = f32::INFINITY;
        let neg_p = (210 * width + 170) as usize;
        materials[neg_p] = MATERIAL_STONE;
        pressures[neg_p] = -5.0;
        let e1 = (220 * width + 150) as usize;
        temperatures[e1] = 3.0; // EMPTY with T != 0
        let e2 = (220 * width + 160) as usize;
        flags[e2] = 0x1234; // EMPTY with flags != 0
        let e3 = (220 * width + 170) as usize;
        pressures[e3] = 2.0; // EMPTY with pressure != 0

        let mut metrics = IntegrityMetrics::default();
        evaluate_integrity_state(
            &materials,
            &temperatures,
            &flags,
            &pressures,
            width,
            height,
            0,
            &mut metrics,
        );
        assert_eq!(metrics.d_invalid_material_ids, 1);
        assert_eq!(metrics.d_nan_inf_temperature, 1);
        assert_eq!(metrics.d_nan_inf_pressure, 1);
        assert_eq!(metrics.d_negative_pressure, 1);
        assert_eq!(metrics.d_empty_temp_violations, 1);
        assert_eq!(metrics.d_empty_flag_violations, 1);
        assert_eq!(metrics.d_empty_pressure_violations, 1);
    }

    #[test]
    fn test_integrity_c_one_tick_latch_and_permanence() {
        let width = 256u32;
        let height = 256u32;
        let cell_count = (width * height) as usize;

        let mut materials = vec![MATERIAL_EMPTY; cell_count];
        let mut temperatures = vec![0.0f32; cell_count];
        let mut flags = vec![0u32; cell_count];
        let mut pressures = vec![0.0f32; cell_count];

        let set_mat = |m: &mut Vec<u32>, x: u32, y: u32, v: u32| {
            m[(y * width + x) as usize] = v;
        };

        // Tick-0 fixture state (pre-tick): sources still Water / Wood.
        for &(sx, sy) in &EXP_SOURCES {
            set_mat(&mut materials, sx, sy, MATERIAL_WATER);
            temperatures[(sy * width + sx) as usize] = 100.0;
        }
        set_mat(&mut materials, EXP_TARGET.0, EXP_TARGET.1, MATERIAL_EMPTY);
        for &(sx, sy) in &SMOKE_SOURCES {
            set_mat(&mut materials, sx, sy, MATERIAL_WOOD);
            temperatures[(sy * width + sx) as usize] = 100.0;
        }
        set_mat(
            &mut materials,
            SMOKE_TARGET.0,
            SMOKE_TARGET.1,
            MATERIAL_EMPTY,
        );
        set_mat(&mut materials, MOVE_SRC.0, MOVE_SRC.1, MATERIAL_SAND);
        set_mat(&mut materials, MOVE_DST.0, MOVE_DST.1, MATERIAL_EMPTY);

        let mut metrics = IntegrityMetrics::default();
        evaluate_integrity_state(
            &materials,
            &temperatures,
            &flags,
            &pressures,
            width,
            height,
            0,
            &mut metrics,
        );
        assert!(!metrics.c_latched);

        // Tick-1 expected post-tick state:
        //   - 3 sources -> Steam, winner spawned Steam at EXP_TARGET
        //   - 2 losers carry confinement pressure (100)
        //   - 3 Woods burning (COMBUSTING), winner spawned Smoke at SMOKE_TARGET (age 0)
        //   - Sand moved MOVE_SRC -> MOVE_DST
        for &(sx, sy) in &EXP_SOURCES {
            set_mat(&mut materials, sx, sy, MATERIAL_STEAM);
        }
        set_mat(&mut materials, EXP_TARGET.0, EXP_TARGET.1, MATERIAL_STEAM);
        pressures[(EXP_SOURCES[0].1 * width + EXP_SOURCES[0].0) as usize] = 100.0;
        pressures[(EXP_SOURCES[2].1 * width + EXP_SOURCES[2].0) as usize] = 100.0;
        for &(sx, sy) in &SMOKE_SOURCES {
            set_mat(&mut materials, sx, sy, MATERIAL_WOOD);
            flags[(sy * width + sx) as usize] = FLAG_COMBUSTING;
        }
        set_mat(
            &mut materials,
            SMOKE_TARGET.0,
            SMOKE_TARGET.1,
            MATERIAL_SMOKE,
        );
        set_mat(&mut materials, MOVE_SRC.0, MOVE_SRC.1, MATERIAL_EMPTY);
        set_mat(&mut materials, MOVE_DST.0, MOVE_DST.1, MATERIAL_SAND);

        evaluate_integrity_state(
            &materials,
            &temperatures,
            &flags,
            &pressures,
            width,
            height,
            1,
            &mut metrics,
        );
        assert!(metrics.c_latched);
        assert_eq!(metrics.c_exp_candidates, 3);
        assert_eq!(metrics.c_exp_winners, 1);
        assert_eq!(metrics.c_exp_steam_sources, 3);
        assert_eq!(metrics.c_exp_pressure_losers, 2);
        assert!(metrics.c_exp_target_steam);
        assert_eq!(metrics.c_smoke_candidates, 3);
        assert_eq!(metrics.c_smoke_winners, 1);
        assert_eq!(metrics.c_smoke_wood_preserved, 3);
        assert_eq!(metrics.c_smoke_age, 0);
        assert!(metrics.c_smoke_target_smoke);
        assert!(metrics.c_move_done);
        assert!(metrics.c_scratch_reuse);
        assert!(metrics.c_result);

        // The latch must be preserved on later snapshots even if the live
        // fixture has moved on (Steam rose, Smoke decayed).
        set_mat(&mut materials, EXP_TARGET.0, EXP_TARGET.1, MATERIAL_EMPTY);
        set_mat(
            &mut materials,
            SMOKE_TARGET.0,
            SMOKE_TARGET.1,
            MATERIAL_EMPTY,
        );
        evaluate_integrity_state(
            &materials,
            &temperatures,
            &flags,
            &pressures,
            width,
            height,
            50,
            &mut metrics,
        );
        assert!(metrics.c_latched);
        assert_eq!(metrics.c_exp_winners, 1);
        assert_eq!(metrics.c_smoke_winners, 1);
        assert_eq!(metrics.c_exp_steam_sources, 3);
        assert_eq!(metrics.c_smoke_wood_preserved, 3);
    }
}
