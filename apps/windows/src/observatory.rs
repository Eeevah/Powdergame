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
    MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_WATER, MATERIAL_WOOD,
};
use powdergame_gpu::Simulation;

/// Diagnostic threshold for detecting warm heat reach in Heat Comparison tubes.
/// (Observation threshold; relative gameplay scalar, not a simulation physics constant).
pub const THERMAL_OBS_WARM_THRESHOLD: f32 = 10.0;

/// Bit flag indicating active combustion on a cell (matches core/GPU shader).
pub const FLAG_COMBUSTING: u32 = 1;

/// Sentinel value indicating no event has occurred yet (e.g. tick record is None).
#[allow(dead_code)]
pub const TICK_NONE: u32 = 0xFFFF_FFFF;

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

/// Asynchronous GPU diagnostic readback collector.
pub struct ObservatoryCollector {
    staging_material: wgpu::Buffer,
    staging_temperature: wgpu::Buffer,
    staging_flags: wgpu::Buffer,
    pending: bool,
    pending_tick: u64,
    receiver: Option<std::sync::mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>>,
    metrics: ObservatoryMetrics,
    initial_wood_set: bool,
    initial_a_ice: u32,
    initial_b_steam: u32,
    last_request_tick: u64,
    cell_bytes: u64,
}

impl ObservatoryCollector {
    /// Creates staging buffers and allocates a new collector.
    pub fn new(simulation: &Simulation) -> Self {
        let device = &simulation.context.device;
        let cell_bytes = simulation.world.layout.material_bytes;

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

        Self {
            staging_material,
            staging_temperature,
            staging_flags,
            pending: false,
            pending_tick: 0,
            receiver: None,
            metrics: ObservatoryMetrics::default(),
            initial_wood_set: false,
            initial_a_ice: 0,
            initial_b_steam: 0,
            last_request_tick: 0,
            cell_bytes,
        }
    }

    /// Resets all metrics and latches (invoked on 'R' key).
    pub fn reset(&mut self) {
        self.metrics = ObservatoryMetrics::default();
        self.initial_wood_set = false;
        self.initial_a_ice = 0;
        self.initial_b_steam = 0;
        self.last_request_tick = 0;
        // Clear pending async states
        if self.pending {
            self.staging_material.unmap();
            self.staging_temperature.unmap();
            self.staging_flags.unmap();
            self.pending = false;
            self.receiver = None;
        }
    }

    /// Current live metrics snapshot.
    pub fn metrics(&self) -> &ObservatoryMetrics {
        &self.metrics
    }

    /// Non-blocking update: checks pending map callbacks and requests next readback if due.
    pub fn update(&mut self, simulation: &Simulation, current_tick: u64) {
        let device = &simulation.context.device;

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

                let width = simulation.world.config.width;
                let height = simulation.world.config.height;
                let cell_count = (width * height) as usize;

                let materials = bytemuck_u32_slice(&mat_slice, cell_count);
                let temperatures = bytemuck_f32_slice(&temp_slice, cell_count);
                let flags = bytemuck_u32_slice(&flag_slice, cell_count);

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

                drop(mat_slice);
                drop(temp_slice);
                drop(flag_slice);

                self.staging_material.unmap();
                self.staging_temperature.unmap();
                self.staging_flags.unmap();

                self.pending = false;
                self.receiver = None;
            }
        }

        // 2. If idle, check if we should request a new diagnostic readback
        // Request every ~10 ticks (or immediately at tick 0/1)
        if !self.pending && (current_tick == 0 || current_tick >= self.last_request_tick + 10) {
            self.request_readback(simulation, current_tick);
        }
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

        queue.submit([encoder.finish()]);

        let (tx, rx) = std::sync::mpsc::channel();
        let tx_temp = tx.clone();
        let tx_flag = tx.clone();

        // Async mapping on all 3 buffers
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

#[cfg(test)]
mod tests {
    use super::*;
    use powdergame_core::MATERIAL_OIL;

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
}
