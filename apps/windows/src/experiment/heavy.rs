//! Heavy Mixed World experiment analysis.
//!
//! This worker only observes the shared authored fixture while the production
//! simulation runs. It does not stage cells itself and owns no physics rule.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use powdergame_core::{
    decay_age, fuel_progress, is_valid_cell_material_value, WorldConfig, ACTIVITY_MATTER,
    ACTIVITY_PRESSURE, ACTIVITY_REACTION, ACTIVITY_THERMAL, CHUNK_STATE_RUNNABLE,
    CHUNK_STATE_SLEEPING, FLAG_COMBUSTING, FLAG_FLAME_EVENT, MATERIAL_BOUNDARY_BLOCK,
    MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_OIL, MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STEAM,
    MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD, WAKE_REASON_USER_EDIT, WOOD_RUPTURE_THRESHOLD,
};
use powdergame_gpu::Simulation;
use powdergame_scenarios::{reset_and_stage_scenario, ScenarioId};

use crate::gallery::RuntimeProvenance;
use crate::renderer::Renderer;

use super::{
    authoritative_current_hash, bit_count, capture_gpu_snapshot, create_new_file,
    create_worker_directory, display_path, exact_reset_equal, is_safe_identifier, json_escape,
    json_opt_u64, take_sequence, write_new, ExperimentOutcome, ExperimentVerdict, Fnv1a64,
    GpuSnapshot, PredicateResult, PredicateStatus, RawFrame, REQUIRED_WORLD,
};

pub const HEAVY_EXPERIMENT_ID: &str = "g8b-heavy-mixed-v0";
pub const HEAVY_TELEMETRY_SCHEMA_VERSION: &str = "powdergame-heavy-mixed-telemetry-v0";
pub const HEAVY_ANALYSIS_SCHEMA_VERSION: &str = "powdergame-heavy-mixed-analysis-v0";
pub const HEAVY_FRAMES_SCHEMA_VERSION: &str = "powdergame-heavy-mixed-frames-v0";
const REQUIRED_MAX_TICKS: u64 = 20_000;
const REQUIRED_DIAGNOSTIC_INTERVAL_TICKS: u64 = 8;
const TERMINAL_WINDOW_SAMPLES: usize = 64;
const MEANINGFUL_MULTI_SYSTEM_SAMPLES: u64 = 3;
const MIN_RAW_FRAMES: usize = 10;
const MAX_RAW_FRAMES: usize = 14;

const RELIEF_MIN_X: usize = 162;
const RELIEF_MAX_X: usize = 190;
const RELIEF_MIN_Y: usize = 140;
const RELIEF_MAX_Y: usize = 148;
const EXTERIOR_STEAM_MIN_Y: usize = 132;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SampleIdentity {
    sim_tick: u64,
    sample_sequence: u64,
}

#[derive(Clone, Debug)]
struct HeavyBaseline {
    materials: Vec<u32>,
    material_counts_by_id: [u64; 10],
    matter_count: u64,
    sand_count: u64,
    water_count: u64,
    oil_count: u64,
    wood_count: u64,
    ice_count: u64,
    steam_count: u64,
    smoke_count: u64,
    phase_pool_count: u64,
    fuel_count: u64,
    relief_seam_wood_count: u64,
    exterior_steam_cells: u64,
    wood_fuel_progress_sum: u64,
    oil_fuel_progress_sum: u64,
    density_ordered_pairs: u64,
    state_hash: String,
    physical_state_hash: String,
}

#[derive(Clone, Debug)]
struct HeavySampleMetrics {
    sample_sequence: u64,
    sim_tick: u64,
    phase: &'static str,
    reason: &'static str,
    total_cells: u64,
    any_active_cells: u64,
    matter_active_cells: u64,
    thermal_active_cells: u64,
    pressure_active_cells: u64,
    reaction_active_cells: u64,
    subsystem_active_count: u32,
    total_chunks: u32,
    active_chunks: u32,
    runnable_chunks: u32,
    sleeping_chunks: u32,
    material_counts_by_id: [u64; 10],
    matter_count: u64,
    sand_count: u64,
    water_count: u64,
    oil_count: u64,
    wood_count: u64,
    ice_count: u64,
    steam_count: u64,
    smoke_count: u64,
    sand_position_changed_cells: u64,
    liquid_position_changed_cells: u64,
    water_oil_interface_edges: u64,
    density_ordered_pairs: u64,
    combusting_wood_cells: u64,
    combusting_oil_cells: u64,
    flame_event_wood_cells: u64,
    flame_event_oil_cells: u64,
    wood_fuel_progress_sum: u64,
    oil_fuel_progress_sum: u64,
    dynamic_combustion_work: bool,
    new_smoke_cells: u64,
    phase_inventory_changed: bool,
    relief_seam_wood_count: u64,
    relief_seam_combusting_cells: u64,
    relief_seam_flame_event_cells: u64,
    relief_seam_fuel_progress_sum: u64,
    relief_seam_adjacent_pressure_medium_cells: u64,
    relief_seam_max_adjacent_pressure: f64,
    relief_open_lanes: u32,
    exterior_steam_cells: u64,
    temperature_min: f64,
    temperature_max: f64,
    pressure_min: f64,
    pressure_max: f64,
    phase_pool_count: u64,
    fuel_count: u64,
    material_count_deltas_by_id: [i64; 10],
    gross_inventory_delta_cells: u64,
    explained_material_delta_cells: u64,
    unexplained_material_delta_cells: u64,
    inventory_accounted: bool,
    invalid_material_count: u64,
    nonfinite_temperature_count: u64,
    nonfinite_pressure_count: u64,
    changed_chunks: u32,
    wake_chunks: u32,
    wake_reason_or: u32,
    wake_anomaly_chunks: u32,
    state_hash: String,
    physical_state_hash: String,
}

impl HeavySampleMetrics {
    fn identity(&self) -> SampleIdentity {
        SampleIdentity {
            sim_tick: self.sim_tick,
            sample_sequence: self.sample_sequence,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct EvidenceContext {
    phase: bool,
    combustion: bool,
    pressure: bool,
    rupture: bool,
}

fn in_relief_seam(x: usize, y: usize) -> bool {
    (RELIEF_MIN_X..RELIEF_MAX_X).contains(&x) && (RELIEF_MIN_Y..RELIEF_MAX_Y).contains(&y)
}

fn is_relief_passable(material: u32) -> bool {
    matches!(material, MATERIAL_EMPTY | MATERIAL_STEAM | MATERIAL_SMOKE)
}

fn relief_open_lanes(materials: &[u32], world: WorldConfig) -> u32 {
    let width = world.width as usize;
    (RELIEF_MIN_X..RELIEF_MAX_X)
        .filter(|&x| {
            (RELIEF_MIN_Y..RELIEF_MAX_Y).all(|y| is_relief_passable(materials[y * width + x]))
        })
        .count() as u32
}

fn exterior_steam_cells(materials: &[u32], world: WorldConfig) -> u64 {
    let width = world.width as usize;
    (EXTERIOR_STEAM_MIN_Y..RELIEF_MIN_Y)
        .flat_map(|y| (RELIEF_MIN_X..RELIEF_MAX_X).map(move |x| y * width + x))
        .filter(|&index| materials[index] == MATERIAL_STEAM)
        .count() as u64
}

fn finite_bounds(values: &[u32]) -> (f64, f64, u64) {
    let mut minimum = f32::INFINITY;
    let mut maximum = f32::NEG_INFINITY;
    let mut nonfinite = 0u64;
    for &bits in values {
        let value = f32::from_bits(bits);
        if value.is_finite() {
            minimum = minimum.min(value);
            maximum = maximum.max(value);
        } else {
            nonfinite = nonfinite.saturating_add(1);
        }
    }
    if minimum == f32::INFINITY {
        (0.0, 0.0, nonfinite)
    } else {
        (f64::from(minimum), f64::from(maximum), nonfinite)
    }
}

fn gross_inventory_delta(deltas: &[i64; 10]) -> u64 {
    deltas.iter().map(|value| value.unsigned_abs()).sum::<u64>() / 2
}

fn apply_inventory_accounting(
    metrics: &mut HeavySampleMetrics,
    baseline: &HeavyBaseline,
    evidence: EvidenceContext,
) {
    let deltas = metrics.material_count_deltas_by_id;
    let mut unexplained = deltas[MATERIAL_BOUNDARY_BLOCK as usize]
        .unsigned_abs()
        .saturating_add(deltas[MATERIAL_STONE as usize].unsigned_abs())
        .saturating_add(deltas[MATERIAL_SAND as usize].unsigned_abs());
    let phase_delta = i128::from(metrics.phase_pool_count) - i128::from(baseline.phase_pool_count);
    if phase_delta < 0 {
        unexplained = unexplained.saturating_add((-phase_delta) as u64);
    } else if phase_delta > 0 && !evidence.phase {
        unexplained = unexplained.saturating_add(phase_delta as u64);
    }
    let oil_delta = deltas[MATERIAL_OIL as usize];
    let wood_delta = deltas[MATERIAL_WOOD as usize];
    if oil_delta > 0 {
        unexplained = unexplained.saturating_add(oil_delta as u64);
    } else if oil_delta < 0 && !evidence.combustion {
        unexplained = unexplained.saturating_add((-oil_delta) as u64);
    }
    if wood_delta > 0 {
        unexplained = unexplained.saturating_add(wood_delta as u64);
    } else if wood_delta < 0 && !(evidence.combustion || evidence.pressure && evidence.rupture) {
        unexplained = unexplained.saturating_add((-wood_delta) as u64);
    }
    let smoke_delta = deltas[MATERIAL_SMOKE as usize];
    if smoke_delta > 0 && !evidence.combustion {
        unexplained = unexplained.saturating_add(smoke_delta as u64);
    }
    let registered_total = metrics.material_counts_by_id.iter().sum::<u64>();
    if registered_total != metrics.total_cells {
        unexplained = unexplained.saturating_add(registered_total.abs_diff(metrics.total_cells));
    }
    metrics.unexplained_material_delta_cells = unexplained;
    metrics.explained_material_delta_cells = metrics
        .gross_inventory_delta_cells
        .saturating_sub(unexplained.min(metrics.gross_inventory_delta_cells));
    metrics.inventory_accounted = unexplained == 0 && metrics.invalid_material_count == 0;
}

#[allow(clippy::too_many_arguments)]
fn heavy_metrics_from_snapshot(
    snapshot: &GpuSnapshot,
    world: WorldConfig,
    baseline: Option<&HeavyBaseline>,
    sample_sequence: u64,
    sim_tick: u64,
    phase: &'static str,
    reason: &'static str,
) -> Result<HeavySampleMetrics, String> {
    let width = world.width as usize;
    let expected_cells = u64::from(world.width) * u64::from(world.height);
    if snapshot.material_current.len() as u64 != expected_cells
        || snapshot.temperature_current.len() as u64 != expected_cells
        || snapshot.pressure_current.len() as u64 != expected_cells
        || snapshot.flags_current.len() as u64 != expected_cells
        || snapshot.cell_activity.len() as u64 != expected_cells
    {
        return Err("Heavy GPU snapshot cell-vector lengths do not match WorldConfig".to_string());
    }
    if snapshot.chunk_activity.len() != snapshot.chunk_state.len()
        || snapshot.chunk_activity.len() != snapshot.chunk_changed.len()
        || snapshot.chunk_activity.len() != snapshot.chunk_wake_reason.len()
    {
        return Err("Heavy GPU snapshot chunk-vector lengths disagree".to_string());
    }
    if baseline.is_some_and(|value| value.materials.len() as u64 != expected_cells) {
        return Err("Heavy baseline material vector does not match WorldConfig".to_string());
    }

    let mut material_counts_by_id = [0u64; 10];
    let mut matter_count = 0u64;
    let mut invalid_material_count = 0u64;
    let mut combusting_wood_cells = 0u64;
    let mut combusting_oil_cells = 0u64;
    let mut flame_event_wood_cells = 0u64;
    let mut flame_event_oil_cells = 0u64;
    let mut wood_fuel_progress_sum = 0u64;
    let mut oil_fuel_progress_sum = 0u64;
    let mut relief_seam_wood_count = 0u64;
    let mut relief_seam_combusting_cells = 0u64;
    let mut relief_seam_flame_event_cells = 0u64;
    let mut relief_seam_fuel_progress_sum = 0u64;
    let mut new_smoke_cells = 0u64;
    for (index, (&material, &flags)) in snapshot
        .material_current
        .iter()
        .zip(&snapshot.flags_current)
        .enumerate()
    {
        if let Some(slot) = material_counts_by_id.get_mut(material as usize) {
            *slot = slot.saturating_add(1);
        }
        if !is_valid_cell_material_value(material) {
            invalid_material_count = invalid_material_count.saturating_add(1);
            continue;
        }
        if material != MATERIAL_EMPTY {
            matter_count = matter_count.saturating_add(1);
        }
        if material == MATERIAL_WOOD {
            combusting_wood_cells =
                combusting_wood_cells.saturating_add(u64::from(flags & FLAG_COMBUSTING != 0));
            flame_event_wood_cells =
                flame_event_wood_cells.saturating_add(u64::from(flags & FLAG_FLAME_EVENT != 0));
            wood_fuel_progress_sum =
                wood_fuel_progress_sum.saturating_add(u64::from(fuel_progress(flags)));
        } else if material == MATERIAL_OIL {
            combusting_oil_cells =
                combusting_oil_cells.saturating_add(u64::from(flags & FLAG_COMBUSTING != 0));
            flame_event_oil_cells =
                flame_event_oil_cells.saturating_add(u64::from(flags & FLAG_FLAME_EVENT != 0));
            oil_fuel_progress_sum =
                oil_fuel_progress_sum.saturating_add(u64::from(fuel_progress(flags)));
        }
        let x = index % width;
        let y = index / width;
        if material == MATERIAL_WOOD && in_relief_seam(x, y) {
            relief_seam_wood_count = relief_seam_wood_count.saturating_add(1);
            relief_seam_combusting_cells = relief_seam_combusting_cells
                .saturating_add(u64::from(flags & FLAG_COMBUSTING != 0));
            relief_seam_flame_event_cells = relief_seam_flame_event_cells
                .saturating_add(u64::from(flags & FLAG_FLAME_EVENT != 0));
            relief_seam_fuel_progress_sum =
                relief_seam_fuel_progress_sum.saturating_add(u64::from(fuel_progress(flags)));
        }
        if sim_tick != 0
            && material == MATERIAL_SMOKE
            && decay_age(flags) == 0
            && baseline.is_some_and(|value| value.materials[index] != MATERIAL_SMOKE)
        {
            new_smoke_cells = new_smoke_cells.saturating_add(1);
        }
    }

    let sand_count = material_counts_by_id[MATERIAL_SAND as usize];
    let water_count = material_counts_by_id[MATERIAL_WATER as usize];
    let oil_count = material_counts_by_id[MATERIAL_OIL as usize];
    let wood_count = material_counts_by_id[MATERIAL_WOOD as usize];
    let ice_count = material_counts_by_id[MATERIAL_ICE as usize];
    let steam_count = material_counts_by_id[MATERIAL_STEAM as usize];
    let smoke_count = material_counts_by_id[MATERIAL_SMOKE as usize];
    let phase_pool_count = water_count
        .saturating_add(ice_count)
        .saturating_add(steam_count);
    let fuel_count = wood_count.saturating_add(oil_count);
    let mut sand_position_changed_cells = 0u64;
    let mut liquid_position_changed_cells = 0u64;
    if let Some(value) = baseline {
        for (&initial, &current) in value.materials.iter().zip(&snapshot.material_current) {
            sand_position_changed_cells = sand_position_changed_cells.saturating_add(u64::from(
                (initial == MATERIAL_SAND) != (current == MATERIAL_SAND),
            ));
            let relevant = matches!(initial, MATERIAL_WATER | MATERIAL_OIL)
                || matches!(current, MATERIAL_WATER | MATERIAL_OIL);
            liquid_position_changed_cells = liquid_position_changed_cells
                .saturating_add(u64::from(relevant && initial != current));
        }
    }
    let mut water_oil_interface_edges = 0u64;
    let mut density_ordered_pairs = 0u64;
    for y in 0..world.height as usize {
        for x in 0..width {
            let material = snapshot.material_current[y * width + x];
            if x + 1 < width {
                let right = snapshot.material_current[y * width + x + 1];
                water_oil_interface_edges =
                    water_oil_interface_edges.saturating_add(u64::from(matches!(
                        (material, right),
                        (MATERIAL_WATER, MATERIAL_OIL) | (MATERIAL_OIL, MATERIAL_WATER)
                    )));
            }
            if y + 1 < world.height as usize {
                let below = snapshot.material_current[(y + 1) * width + x];
                water_oil_interface_edges =
                    water_oil_interface_edges.saturating_add(u64::from(matches!(
                        (material, below),
                        (MATERIAL_WATER, MATERIAL_OIL) | (MATERIAL_OIL, MATERIAL_WATER)
                    )));
                density_ordered_pairs = density_ordered_pairs.saturating_add(u64::from(
                    material == MATERIAL_OIL && below == MATERIAL_WATER,
                ));
            }
        }
    }
    let mut relief_seam_adjacent_pressure_medium_cells = 0u64;
    let mut relief_seam_max_adjacent_pressure = 0.0f32;
    for y in RELIEF_MIN_Y..RELIEF_MAX_Y {
        for x in RELIEF_MIN_X..RELIEF_MAX_X {
            for (nx, ny) in [
                (x.wrapping_sub(1), y),
                (x.saturating_add(1), y),
                (x, y.wrapping_sub(1)),
                (x, y.saturating_add(1)),
            ] {
                if nx >= width || ny >= world.height as usize {
                    continue;
                }
                let index = ny * width + nx;
                if matches!(
                    snapshot.material_current[index],
                    MATERIAL_WATER | MATERIAL_OIL | MATERIAL_STEAM | MATERIAL_SMOKE
                ) {
                    let pressure = f32::from_bits(snapshot.pressure_current[index]);
                    if pressure.is_finite() {
                        relief_seam_adjacent_pressure_medium_cells =
                            relief_seam_adjacent_pressure_medium_cells.saturating_add(1);
                        relief_seam_max_adjacent_pressure =
                            relief_seam_max_adjacent_pressure.max(pressure.max(0.0));
                    }
                }
            }
        }
    }
    let matter_active_cells = bit_count(&snapshot.cell_activity, ACTIVITY_MATTER);
    let thermal_active_cells = bit_count(&snapshot.cell_activity, ACTIVITY_THERMAL);
    let pressure_active_cells = bit_count(&snapshot.cell_activity, ACTIVITY_PRESSURE);
    let reaction_active_cells = bit_count(&snapshot.cell_activity, ACTIVITY_REACTION);
    let subsystem_active_count = [
        matter_active_cells,
        thermal_active_cells,
        pressure_active_cells,
        reaction_active_cells,
    ]
    .into_iter()
    .filter(|&value| value != 0)
    .count() as u32;
    let (temperature_min, temperature_max, nonfinite_temperature_count) =
        finite_bounds(&snapshot.temperature_current);
    let (pressure_min, pressure_max, nonfinite_pressure_count) =
        finite_bounds(&snapshot.pressure_current);
    let material_count_deltas_by_id = std::array::from_fn(|index| {
        i64::try_from(material_counts_by_id[index]).unwrap_or(i64::MAX)
            - baseline
                .map(|value| i64::try_from(value.material_counts_by_id[index]).unwrap_or(i64::MAX))
                .unwrap_or_else(|| i64::try_from(material_counts_by_id[index]).unwrap_or(i64::MAX))
    });
    let gross_inventory_delta_cells = gross_inventory_delta(&material_count_deltas_by_id);
    let dynamic_combustion_work = sim_tick != 0
        && baseline.is_some_and(|value| {
            flame_event_wood_cells.saturating_add(flame_event_oil_cells) != 0
                || wood_fuel_progress_sum > value.wood_fuel_progress_sum
                || oil_fuel_progress_sum > value.oil_fuel_progress_sum
        });
    let wake_anomaly_chunks = if matches!(phase, "initial" | "reset") {
        0
    } else {
        snapshot
            .chunk_wake_reason
            .iter()
            .filter(|&&reason| reason & WAKE_REASON_USER_EDIT != 0 || reason & !0x1f != 0)
            .count() as u32
    };
    let mut metrics = HeavySampleMetrics {
        sample_sequence,
        sim_tick,
        phase,
        reason,
        total_cells: expected_cells,
        any_active_cells: snapshot
            .cell_activity
            .iter()
            .filter(|&&value| value != 0)
            .count() as u64,
        matter_active_cells,
        thermal_active_cells,
        pressure_active_cells,
        reaction_active_cells,
        subsystem_active_count,
        total_chunks: snapshot.chunk_activity.len() as u32,
        active_chunks: snapshot
            .chunk_activity
            .iter()
            .filter(|&&value| value != 0)
            .count() as u32,
        runnable_chunks: snapshot
            .chunk_state
            .iter()
            .filter(|&&value| value == CHUNK_STATE_RUNNABLE)
            .count() as u32,
        sleeping_chunks: snapshot
            .chunk_state
            .iter()
            .filter(|&&value| value == CHUNK_STATE_SLEEPING)
            .count() as u32,
        material_counts_by_id,
        matter_count,
        sand_count,
        water_count,
        oil_count,
        wood_count,
        ice_count,
        steam_count,
        smoke_count,
        sand_position_changed_cells,
        liquid_position_changed_cells,
        water_oil_interface_edges,
        density_ordered_pairs,
        combusting_wood_cells,
        combusting_oil_cells,
        flame_event_wood_cells,
        flame_event_oil_cells,
        wood_fuel_progress_sum,
        oil_fuel_progress_sum,
        dynamic_combustion_work,
        new_smoke_cells,
        phase_inventory_changed: baseline.is_some_and(|value| {
            (ice_count, water_count, steam_count)
                != (value.ice_count, value.water_count, value.steam_count)
        }),
        relief_seam_wood_count,
        relief_seam_combusting_cells,
        relief_seam_flame_event_cells,
        relief_seam_fuel_progress_sum,
        relief_seam_adjacent_pressure_medium_cells,
        relief_seam_max_adjacent_pressure: f64::from(relief_seam_max_adjacent_pressure),
        relief_open_lanes: relief_open_lanes(&snapshot.material_current, world),
        exterior_steam_cells: exterior_steam_cells(&snapshot.material_current, world),
        temperature_min,
        temperature_max,
        pressure_min,
        pressure_max,
        phase_pool_count,
        fuel_count,
        material_count_deltas_by_id,
        gross_inventory_delta_cells,
        explained_material_delta_cells: 0,
        unexplained_material_delta_cells: 0,
        inventory_accounted: true,
        invalid_material_count,
        nonfinite_temperature_count,
        nonfinite_pressure_count,
        changed_chunks: snapshot
            .chunk_changed
            .iter()
            .filter(|&&value| value != 0)
            .count() as u32,
        wake_chunks: snapshot
            .chunk_wake_reason
            .iter()
            .filter(|&&value| value != 0)
            .count() as u32,
        wake_reason_or: snapshot
            .chunk_wake_reason
            .iter()
            .copied()
            .fold(0, |acc, value| acc | value),
        wake_anomaly_chunks,
        state_hash: authoritative_current_hash(snapshot),
        physical_state_hash: physical_state_hash(snapshot),
    };
    if let Some(value) = baseline {
        apply_inventory_accounting(&mut metrics, value, EvidenceContext::default());
    }
    Ok(metrics)
}

fn physical_state_hash(snapshot: &GpuSnapshot) -> String {
    let mut hash = Fnv1a64::new();
    hash.update_u32s(&snapshot.material_current);
    hash.update_u32s(&snapshot.material_next);
    hash.update_u32s(&snapshot.temperature_current);
    hash.update_u32s(&snapshot.temperature_next);
    hash.update_u32s(&snapshot.pressure_current);
    hash.update_u32s(&snapshot.pressure_next);
    hash.update_u32s(&snapshot.flags_current);
    hash.update_u32s(&snapshot.flags_next);
    format!("fnv1a64:{:016x}", hash.finish())
}

fn baseline_from_tick0(snapshot: &GpuSnapshot, metrics: &HeavySampleMetrics) -> HeavyBaseline {
    HeavyBaseline {
        materials: snapshot.material_current.clone(),
        material_counts_by_id: metrics.material_counts_by_id,
        matter_count: metrics.matter_count,
        sand_count: metrics.sand_count,
        water_count: metrics.water_count,
        oil_count: metrics.oil_count,
        wood_count: metrics.wood_count,
        ice_count: metrics.ice_count,
        steam_count: metrics.steam_count,
        smoke_count: metrics.smoke_count,
        phase_pool_count: metrics.phase_pool_count,
        fuel_count: metrics.fuel_count,
        relief_seam_wood_count: metrics.relief_seam_wood_count,
        exterior_steam_cells: metrics.exterior_steam_cells,
        wood_fuel_progress_sum: metrics.wood_fuel_progress_sum,
        oil_fuel_progress_sum: metrics.oil_fuel_progress_sum,
        density_ordered_pairs: metrics.density_ordered_pairs,
        state_hash: metrics.state_hash.clone(),
        physical_state_hash: metrics.physical_state_hash.clone(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ObservationUpdate {
    first_movement: bool,
    first_density: bool,
    first_thermal: bool,
    first_phase: bool,
    first_combustion: bool,
    first_smoke: bool,
    first_pressure: bool,
    first_relief_damage: bool,
    first_rupture: bool,
    first_opening: bool,
    first_vent: bool,
    first_three: bool,
    first_all: bool,
    new_peak_active: bool,
    new_peak_concurrency: bool,
}

#[derive(Clone, Debug, Default)]
struct SubsystemSummary {
    peak_cells: u64,
    peak: Option<SampleIdentity>,
    active_sample_count: u64,
    first: Option<SampleIdentity>,
    last: Option<SampleIdentity>,
    cumulative_active_cells: u64,
}

impl SubsystemSummary {
    fn observe(&mut self, cells: u64, identity: SampleIdentity) {
        self.cumulative_active_cells = self.cumulative_active_cells.saturating_add(cells);
        if cells > self.peak_cells {
            self.peak_cells = cells;
            self.peak = Some(identity);
        }
        if cells != 0 {
            self.active_sample_count = self.active_sample_count.saturating_add(1);
            self.first.get_or_insert(identity);
            self.last = Some(identity);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MultiSystemWindow {
    sample_count: u64,
    start: Option<SampleIdentity>,
    end: Option<SampleIdentity>,
}

impl MultiSystemWindow {
    fn tick_span(self) -> u64 {
        match (self.start, self.end) {
            (Some(start), Some(end)) => end.sim_tick.saturating_sub(start.sim_tick),
            _ => 0,
        }
    }
}

#[derive(Clone, Debug)]
struct HeavyObservations {
    first_movement: Option<SampleIdentity>,
    first_density: Option<SampleIdentity>,
    first_thermal: Option<SampleIdentity>,
    first_phase: Option<SampleIdentity>,
    first_combustion: Option<SampleIdentity>,
    first_smoke: Option<SampleIdentity>,
    first_pressure: Option<SampleIdentity>,
    first_relief_damage: Option<SampleIdentity>,
    first_rupture: Option<SampleIdentity>,
    first_opening: Option<SampleIdentity>,
    first_vent: Option<SampleIdentity>,
    first_three: Option<SampleIdentity>,
    first_all: Option<SampleIdentity>,
    peak_active_cells: u64,
    peak_active: Option<SampleIdentity>,
    peak_concurrent_subsystems: u32,
    peak_concurrency: Option<SampleIdentity>,
    matter: SubsystemSummary,
    thermal: SubsystemSummary,
    pressure: SubsystemSummary,
    reaction: SubsystemSummary,
    current_multi_window: MultiSystemWindow,
    longest_multi_window: MultiSystemWindow,
    smoke_peak: u64,
    smoke_peak_identity: Option<SampleIdentity>,
    invalid_material_occurrences: u64,
    nonfinite_field_occurrences: u64,
    unexplained_material_occurrences: u64,
    wake_anomaly_occurrences: u64,
    zero_activity_before_overlap_samples: u64,
    seam_combustion_seen: bool,
    latest: HeavySampleMetrics,
}

impl HeavyObservations {
    fn new(tick0: HeavySampleMetrics) -> Self {
        Self {
            first_movement: None,
            first_density: None,
            first_thermal: None,
            first_phase: None,
            first_combustion: None,
            first_smoke: None,
            first_pressure: None,
            first_relief_damage: None,
            first_rupture: None,
            first_opening: None,
            first_vent: None,
            first_three: None,
            first_all: None,
            peak_active_cells: 0,
            peak_active: None,
            peak_concurrent_subsystems: 0,
            peak_concurrency: None,
            matter: SubsystemSummary::default(),
            thermal: SubsystemSummary::default(),
            pressure: SubsystemSummary::default(),
            reaction: SubsystemSummary::default(),
            current_multi_window: MultiSystemWindow::default(),
            longest_multi_window: MultiSystemWindow::default(),
            smoke_peak: tick0.smoke_count,
            smoke_peak_identity: Some(tick0.identity()),
            invalid_material_occurrences: 0,
            nonfinite_field_occurrences: 0,
            unexplained_material_occurrences: 0,
            wake_anomaly_occurrences: 0,
            zero_activity_before_overlap_samples: 0,
            seam_combustion_seen: false,
            latest: tick0,
        }
    }

    fn evidence(&self) -> EvidenceContext {
        EvidenceContext {
            phase: self.first_phase.is_some(),
            combustion: self.first_combustion.is_some(),
            pressure: self.first_pressure.is_some(),
            rupture: self.first_rupture.is_some(),
        }
    }

    fn observe(
        &mut self,
        metrics: &mut HeavySampleMetrics,
        baseline: &HeavyBaseline,
        production_sample: bool,
    ) -> ObservationUpdate {
        let identity = metrics.identity();
        let first_movement = metrics.sand_position_changed_cells != 0
            && self.first_movement.get_or_insert(identity) == &identity;
        let first_density = metrics.sim_tick != 0
            && baseline.density_ordered_pairs == 0
            && metrics.density_ordered_pairs != 0
            && metrics.liquid_position_changed_cells != 0
            && metrics.water_oil_interface_edges != 0
            && self.first_density.get_or_insert(identity) == &identity;
        let first_thermal = metrics.thermal_active_cells != 0
            && self.first_thermal.get_or_insert(identity) == &identity;
        let first_phase = metrics.phase_inventory_changed
            && self.first_phase.get_or_insert(identity) == &identity;
        let first_combustion = metrics.dynamic_combustion_work
            && self.first_combustion.get_or_insert(identity) == &identity;
        let first_smoke =
            metrics.new_smoke_cells != 0 && self.first_smoke.get_or_insert(identity) == &identity;
        let first_pressure = metrics.pressure_active_cells != 0
            && self.first_pressure.get_or_insert(identity) == &identity;
        self.seam_combustion_seen |= metrics.relief_seam_combusting_cells != 0
            || metrics.relief_seam_flame_event_cells != 0
            || metrics.relief_seam_fuel_progress_sum != 0;
        let first_relief_damage = metrics.relief_seam_wood_count < baseline.relief_seam_wood_count
            && self.first_relief_damage.get_or_insert(identity) == &identity;
        let first_rupture = metrics.relief_seam_wood_count < baseline.relief_seam_wood_count
            && self.first_pressure.is_some()
            && !self.seam_combustion_seen
            && metrics.relief_seam_max_adjacent_pressure >= f64::from(WOOD_RUPTURE_THRESHOLD)
            && self.first_rupture.get_or_insert(identity) == &identity;
        let first_opening = metrics.relief_open_lanes != 0
            && self.first_opening.get_or_insert(identity) == &identity;
        let first_vent = metrics.exterior_steam_cells > baseline.exterior_steam_cells
            && self.first_vent.get_or_insert(identity) == &identity;
        let first_three = metrics.subsystem_active_count >= 3
            && self.first_three.get_or_insert(identity) == &identity;
        let first_all = metrics.subsystem_active_count == 4
            && self.first_all.get_or_insert(identity) == &identity;

        let new_peak_active = metrics.any_active_cells > self.peak_active_cells;
        if new_peak_active {
            self.peak_active_cells = metrics.any_active_cells;
            self.peak_active = Some(identity);
        }
        let new_peak_concurrency = metrics.subsystem_active_count > self.peak_concurrent_subsystems;
        if new_peak_concurrency {
            self.peak_concurrent_subsystems = metrics.subsystem_active_count;
            self.peak_concurrency = Some(identity);
        }
        self.matter.observe(metrics.matter_active_cells, identity);
        self.thermal.observe(metrics.thermal_active_cells, identity);
        self.pressure
            .observe(metrics.pressure_active_cells, identity);
        self.reaction
            .observe(metrics.reaction_active_cells, identity);

        if metrics.subsystem_active_count >= 3 {
            if self.current_multi_window.sample_count == 0 {
                self.current_multi_window.start = Some(identity);
            }
            self.current_multi_window.sample_count =
                self.current_multi_window.sample_count.saturating_add(1);
            self.current_multi_window.end = Some(identity);
            if self.current_multi_window.sample_count > self.longest_multi_window.sample_count {
                self.longest_multi_window = self.current_multi_window;
            }
        } else {
            self.current_multi_window = MultiSystemWindow::default();
        }
        if metrics.smoke_count > self.smoke_peak {
            self.smoke_peak = metrics.smoke_count;
            self.smoke_peak_identity = Some(identity);
        }

        apply_inventory_accounting(metrics, baseline, self.evidence());
        self.invalid_material_occurrences = self
            .invalid_material_occurrences
            .saturating_add(metrics.invalid_material_count);
        self.nonfinite_field_occurrences = self.nonfinite_field_occurrences.saturating_add(
            metrics
                .nonfinite_temperature_count
                .saturating_add(metrics.nonfinite_pressure_count),
        );
        self.unexplained_material_occurrences = self
            .unexplained_material_occurrences
            .saturating_add(metrics.unexplained_material_delta_cells);
        self.wake_anomaly_occurrences = self
            .wake_anomaly_occurrences
            .saturating_add(u64::from(metrics.wake_anomaly_chunks));
        if production_sample && metrics.any_active_cells == 0 && self.first_three.is_none() {
            self.zero_activity_before_overlap_samples =
                self.zero_activity_before_overlap_samples.saturating_add(1);
        }
        self.latest = metrics.clone();

        ObservationUpdate {
            first_movement,
            first_density,
            first_thermal,
            first_phase,
            first_combustion,
            first_smoke,
            first_pressure,
            first_relief_damage,
            first_rupture,
            first_opening,
            first_vent,
            first_three,
            first_all,
            new_peak_active,
            new_peak_concurrency,
        }
    }
}

struct HeavyJsonlWriters {
    samples: BufWriter<File>,
    events: BufWriter<File>,
    event_sequence: u64,
}

impl HeavyJsonlWriters {
    fn new(samples_path: &Path, events_path: &Path) -> Result<Self, String> {
        Ok(Self {
            samples: BufWriter::new(create_new_file(samples_path)?),
            events: BufWriter::new(create_new_file(events_path)?),
            event_sequence: 0,
        })
    }

    fn sample(
        &mut self,
        config: &super::ExperimentWorkerConfig,
        provenance: &RuntimeProvenance,
        simulation: &Simulation,
        metrics: &HeavySampleMetrics,
    ) -> Result<(), String> {
        let counts = metrics
            .material_counts_by_id
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let deltas = metrics
            .material_count_deltas_by_id
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        writeln!(
            self.samples,
            concat!(
                "{{\"schema_version\":\"{}\",\"experiment_id\":\"{}\",",
                "\"run_id\":\"{}\",\"scenario\":\"heavy-mixed\",",
                "\"source_sha\":\"{}\",\"git_state\":\"{}\",",
                "\"build_profile\":\"{}\",\"binary_sha256\":\"{}\",",
                "\"sample_sequence\":{},\"sim_tick\":{},\"phase\":\"{}\",\"reason\":\"{}\",",
                "\"world\":{{\"width\":{},\"height\":{},\"chunk_size\":{}}},",
                "\"sleep\":{{\"enabled\":{},\"threshold\":{}}},",
                "\"census\":{{\"total_cells\":{},\"any_active_cells\":{},",
                "\"matter_active_cells\":{},\"thermal_active_cells\":{},",
                "\"pressure_active_cells\":{},\"reaction_active_cells\":{},",
                "\"total_chunks\":{},\"active_chunks\":{},",
                "\"runnable_chunks\":{},\"sleeping_chunks\":{}}},",
                "\"subsystem_active_count\":{},\"material_counts_by_id\":[{}],",
                "\"matter_count\":{},\"sand_count\":{},\"water_count\":{},",
                "\"oil_count\":{},\"wood_count\":{},\"ice_count\":{},",
                "\"steam_count\":{},\"smoke_count\":{},",
                "\"sand_position_changed_cells\":{},\"liquid_position_changed_cells\":{},",
                "\"water_oil_interface_edges\":{},\"density_ordered_pairs\":{},",
                "\"combusting_wood_cells\":{},\"combusting_oil_cells\":{},",
                "\"flame_event_wood_cells\":{},\"flame_event_oil_cells\":{},",
                "\"wood_fuel_progress_sum\":{},\"oil_fuel_progress_sum\":{},",
                "\"dynamic_combustion_work\":{},\"new_smoke_cells\":{},",
                "\"phase_inventory_changed\":{},\"relief_seam_wood_count\":{},",
                "\"relief_seam_combusting_cells\":{},\"relief_seam_flame_event_cells\":{},",
                "\"relief_seam_fuel_progress_sum\":{},",
                "\"relief_seam_adjacent_pressure_medium_cells\":{},",
                "\"relief_seam_max_adjacent_pressure\":{},",
                "\"relief_open_lanes\":{},\"exterior_steam_cells\":{},",
                "\"temperature_min\":{},\"temperature_max\":{},",
                "\"pressure_min\":{},\"pressure_max\":{},",
                "\"phase_pool_count\":{},\"fuel_count\":{},",
                "\"material_count_deltas_by_id\":[{}],",
                "\"gross_inventory_delta_cells\":{},\"explained_material_delta_cells\":{},",
                "\"unexplained_material_delta_cells\":{},\"inventory_accounted\":{},",
                "\"invalid_material_count\":{},\"nonfinite_temperature_count\":{},",
                "\"nonfinite_pressure_count\":{},\"changed_chunks\":{},",
                "\"wake_chunks\":{},\"wake_reason_or\":{},\"wake_anomaly_chunks\":{},",
                "\"state_hash\":\"{}\",\"physical_state_hash\":\"{}\"}}"
            ),
            HEAVY_TELEMETRY_SCHEMA_VERSION,
            json_escape(&config.experiment_id),
            json_escape(&config.run_id),
            json_escape(&provenance.source_sha),
            provenance.git_state.as_str(),
            provenance.build_profile,
            json_escape(&config.binary_sha256.to_ascii_lowercase()),
            metrics.sample_sequence,
            metrics.sim_tick,
            metrics.phase,
            metrics.reason,
            simulation.world.config.width,
            simulation.world.config.height,
            simulation.world.config.chunk_size,
            simulation.sleep_enabled,
            simulation.sleep_threshold,
            metrics.total_cells,
            metrics.any_active_cells,
            metrics.matter_active_cells,
            metrics.thermal_active_cells,
            metrics.pressure_active_cells,
            metrics.reaction_active_cells,
            metrics.total_chunks,
            metrics.active_chunks,
            metrics.runnable_chunks,
            metrics.sleeping_chunks,
            metrics.subsystem_active_count,
            counts,
            metrics.matter_count,
            metrics.sand_count,
            metrics.water_count,
            metrics.oil_count,
            metrics.wood_count,
            metrics.ice_count,
            metrics.steam_count,
            metrics.smoke_count,
            metrics.sand_position_changed_cells,
            metrics.liquid_position_changed_cells,
            metrics.water_oil_interface_edges,
            metrics.density_ordered_pairs,
            metrics.combusting_wood_cells,
            metrics.combusting_oil_cells,
            metrics.flame_event_wood_cells,
            metrics.flame_event_oil_cells,
            metrics.wood_fuel_progress_sum,
            metrics.oil_fuel_progress_sum,
            metrics.dynamic_combustion_work,
            metrics.new_smoke_cells,
            metrics.phase_inventory_changed,
            metrics.relief_seam_wood_count,
            metrics.relief_seam_combusting_cells,
            metrics.relief_seam_flame_event_cells,
            metrics.relief_seam_fuel_progress_sum,
            metrics.relief_seam_adjacent_pressure_medium_cells,
            metrics.relief_seam_max_adjacent_pressure,
            metrics.relief_open_lanes,
            metrics.exterior_steam_cells,
            metrics.temperature_min,
            metrics.temperature_max,
            metrics.pressure_min,
            metrics.pressure_max,
            metrics.phase_pool_count,
            metrics.fuel_count,
            deltas,
            metrics.gross_inventory_delta_cells,
            metrics.explained_material_delta_cells,
            metrics.unexplained_material_delta_cells,
            metrics.inventory_accounted,
            metrics.invalid_material_count,
            metrics.nonfinite_temperature_count,
            metrics.nonfinite_pressure_count,
            metrics.changed_chunks,
            metrics.wake_chunks,
            metrics.wake_reason_or,
            metrics.wake_anomaly_chunks,
            metrics.state_hash,
            metrics.physical_state_hash,
        )
        .map_err(|error| format!("write Heavy samples JSONL failed: {error}"))
    }

    fn event(
        &mut self,
        config: &super::ExperimentWorkerConfig,
        event: &str,
        sim_tick: u64,
        sample_sequence: Option<u64>,
        detail: &str,
    ) -> Result<(), String> {
        writeln!(
            self.events,
            concat!(
                "{{\"schema_version\":\"{}\",\"experiment_id\":\"{}\",",
                "\"run_id\":\"{}\",\"scenario\":\"heavy-mixed\",",
                "\"event_sequence\":{},\"event\":\"{}\",\"sim_tick\":{},",
                "\"sample_sequence\":{},\"detail\":\"{}\"}}"
            ),
            HEAVY_TELEMETRY_SCHEMA_VERSION,
            json_escape(&config.experiment_id),
            json_escape(&config.run_id),
            self.event_sequence,
            json_escape(event),
            sim_tick,
            json_opt_u64(sample_sequence),
            json_escape(detail),
        )
        .map_err(|error| format!("write Heavy events JSONL failed: {error}"))?;
        self.event_sequence = self.event_sequence.saturating_add(1);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        self.samples
            .flush()
            .map_err(|error| format!("flush Heavy samples JSONL failed: {error}"))?;
        self.events
            .flush()
            .map_err(|error| format!("flush Heavy events JSONL failed: {error}"))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FrameBadge {
    kind: &'static str,
    reason: &'static str,
}

#[derive(Clone, Debug)]
struct FrameCaptionMetrics {
    active_cells: u64,
    subsystem_active_count: u32,
    matter_active_cells: u64,
    thermal_active_cells: u64,
    pressure_active_cells: u64,
    reaction_active_cells: u64,
    sand_count: u64,
    water_count: u64,
    oil_count: u64,
    wood_count: u64,
    ice_count: u64,
    steam_count: u64,
    smoke_count: u64,
}

#[derive(Clone, Debug)]
struct HeavyFrame {
    sim_tick: u64,
    sample_sequence: u64,
    state_hash: String,
    badges: Vec<FrameBadge>,
    caption: FrameCaptionMetrics,
    frame: RawFrame,
}

impl HeavyFrame {
    fn clone_with_badges(&self, badges: Vec<FrameBadge>) -> Self {
        Self {
            sim_tick: self.sim_tick,
            sample_sequence: self.sample_sequence,
            state_hash: self.state_hash.clone(),
            badges,
            caption: self.caption.clone(),
            frame: self.frame.clone(),
        }
    }
}

#[derive(Clone, Debug)]
struct WrittenHeavyFrame {
    ordinal: usize,
    kind: &'static str,
    relative_path: String,
    width: u32,
    height: u32,
    rgba_bytes: usize,
    reason: &'static str,
    sim_tick: u64,
    sample_sequence: u64,
    state_hash: String,
    badges: Vec<FrameBadge>,
    caption: FrameCaptionMetrics,
}

fn badge_rank(kind: &str) -> usize {
    match kind {
        "tick0" => 0,
        "tick1" => 1,
        "first-movement" => 2,
        "first-density" => 3,
        "first-phase" => 4,
        "first-combustion" => 5,
        "first-smoke" => 6,
        "first-pressure" => 7,
        "first-rupture" => 8,
        "first-vent" => 9,
        "peak-concurrency" => 10,
        "peak-active" => 11,
        "representative" => 12,
        "mid-run" => 13,
        "late-run" => 14,
        "terminal" => 15,
        "reset" => 16,
        _ => 17,
    }
}

fn capture_heavy_frame(
    renderer: &mut Renderer,
    metrics: &HeavySampleMetrics,
    badges: Vec<FrameBadge>,
) -> Result<HeavyFrame, String> {
    let captured = renderer
        .capture_full_frame(None)
        .map_err(|error| format!("capture Heavy frame failed: {error}"))?;
    Ok(HeavyFrame {
        sim_tick: metrics.sim_tick,
        sample_sequence: metrics.sample_sequence,
        state_hash: metrics.state_hash.clone(),
        badges,
        caption: FrameCaptionMetrics {
            active_cells: metrics.any_active_cells,
            subsystem_active_count: metrics.subsystem_active_count,
            matter_active_cells: metrics.matter_active_cells,
            thermal_active_cells: metrics.thermal_active_cells,
            pressure_active_cells: metrics.pressure_active_cells,
            reaction_active_cells: metrics.reaction_active_cells,
            sand_count: metrics.sand_count,
            water_count: metrics.water_count,
            oil_count: metrics.oil_count,
            wood_count: metrics.wood_count,
            ice_count: metrics.ice_count,
            steam_count: metrics.steam_count,
            smoke_count: metrics.smoke_count,
        },
        frame: RawFrame::try_from(captured)?,
    })
}

fn fold_and_order_frames(mut frames: Vec<HeavyFrame>) -> Vec<HeavyFrame> {
    for frame in &mut frames {
        frame.badges.sort_by_key(|badge| badge_rank(badge.kind));
        frame
            .badges
            .dedup_by(|left, right| left.kind == right.kind && left.reason == right.reason);
    }
    frames.sort_by(|left, right| {
        let left_reset = left.badges.iter().any(|badge| badge.kind == "reset");
        let right_reset = right.badges.iter().any(|badge| badge.kind == "reset");
        left_reset
            .cmp(&right_reset)
            .then_with(|| left.sim_tick.cmp(&right.sim_tick))
            .then_with(|| left.sample_sequence.cmp(&right.sample_sequence))
            .then_with(|| {
                left.badges
                    .first()
                    .map_or(usize::MAX, |badge| badge_rank(badge.kind))
                    .cmp(
                        &right
                            .badges
                            .first()
                            .map_or(usize::MAX, |badge| badge_rank(badge.kind)),
                    )
            })
    });
    let mut folded: Vec<HeavyFrame> = Vec::with_capacity(frames.len());
    for mut frame in frames {
        let frame_is_reset = frame.badges.iter().any(|badge| badge.kind == "reset");
        if let Some(existing) = folded.iter_mut().find(|existing| {
            let existing_is_reset = existing.badges.iter().any(|badge| badge.kind == "reset");
            existing.sim_tick == frame.sim_tick
                && existing.state_hash == frame.state_hash
                && existing_is_reset == frame_is_reset
        }) {
            existing.badges.append(&mut frame.badges);
            existing.badges.sort_by_key(|badge| badge_rank(badge.kind));
            existing
                .badges
                .dedup_by(|left, right| left.kind == right.kind && left.reason == right.reason);
        } else {
            folded.push(frame);
        }
    }
    while folded.len() > MAX_RAW_FRAMES {
        if let Some(index) = folded.iter().position(|frame| {
            frame
                .badges
                .iter()
                .all(|badge| badge.kind == "representative")
        }) {
            folded.remove(index);
        } else {
            let removable = (1..folded.len().saturating_sub(2)).rev().find(|&index| {
                !folded[index]
                    .badges
                    .iter()
                    .any(|badge| matches!(badge.kind, "tick0" | "tick1" | "terminal" | "reset"))
            });
            if let Some(index) = removable {
                folded.remove(index);
            } else {
                break;
            }
        }
    }
    let mut seen_non_reset_hashes = std::collections::HashSet::new();
    let mut index = 0usize;
    while index < folded.len() {
        let is_reset = folded[index]
            .badges
            .iter()
            .any(|badge| badge.kind == "reset");
        let generic_only = folded[index]
            .badges
            .iter()
            .all(|badge| matches!(badge.kind, "representative" | "mid-run" | "late-run"));
        let duplicate = !is_reset && seen_non_reset_hashes.contains(&folded[index].state_hash);
        if generic_only && duplicate && folded.len() > MIN_RAW_FRAMES {
            folded.remove(index);
            continue;
        }
        if !is_reset {
            seen_non_reset_hashes.insert(folded[index].state_hash.clone());
        }
        index += 1;
    }
    folded.sort_by(|left, right| {
        let left_reset = left.badges.iter().any(|badge| badge.kind == "reset");
        let right_reset = right.badges.iter().any(|badge| badge.kind == "reset");
        left_reset
            .cmp(&right_reset)
            .then_with(|| left.sim_tick.cmp(&right.sim_tick))
            .then_with(|| left.sample_sequence.cmp(&right.sample_sequence))
    });
    folded
}

fn write_heavy_raw_frames(
    raw_frames_dir: &Path,
    frames: Vec<HeavyFrame>,
) -> Result<Vec<WrittenHeavyFrame>, String> {
    let mut written = Vec::with_capacity(frames.len());
    for (ordinal, frame) in frames.into_iter().enumerate() {
        let primary = frame
            .badges
            .first()
            .ok_or_else(|| "Heavy frame has no badge".to_string())?;
        let filename = format!("{ordinal:02}-{}.rgba", primary.kind);
        write_new(&raw_frames_dir.join(&filename), &frame.frame.rgba)?;
        written.push(WrittenHeavyFrame {
            ordinal,
            kind: primary.kind,
            relative_path: format!("work/frames/{filename}"),
            width: frame.frame.width,
            height: frame.frame.height,
            rgba_bytes: frame.frame.rgba.len(),
            reason: primary.reason,
            sim_tick: frame.sim_tick,
            sample_sequence: frame.sample_sequence,
            state_hash: frame.state_hash,
            badges: frame.badges,
            caption: frame.caption,
        });
    }
    Ok(written)
}

fn write_frames_json(
    config: &super::ExperimentWorkerConfig,
    path: &Path,
    frames: &[WrittenHeavyFrame],
) -> Result<(), String> {
    let entries = frames
        .iter()
        .map(|frame| {
            let badges = frame
                .badges
                .iter()
                .map(|badge| {
                    format!(
                        "{{\"kind\":\"{}\",\"reason\":\"{}\"}}",
                        json_escape(badge.kind),
                        json_escape(badge.reason)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                concat!(
                    "{{\"ordinal\":{},\"kind\":\"{}\",\"relative_path\":\"{}\",",
                    "\"width\":{},\"height\":{},\"rgba_bytes\":{},\"reason\":\"{}\",",
                    "\"sim_tick\":{},\"sample_sequence\":{},\"state_hash\":\"{}\",",
                    "\"badges\":[{}],\"caption_metrics\":{{\"active_cells\":{},",
                    "\"subsystem_active_count\":{},\"matter_active_cells\":{},",
                    "\"thermal_active_cells\":{},\"pressure_active_cells\":{},",
                    "\"reaction_active_cells\":{},\"sand_count\":{},\"water_count\":{},",
                    "\"oil_count\":{},\"wood_count\":{},\"ice_count\":{},",
                    "\"steam_count\":{},\"smoke_count\":{}}}}}"
                ),
                frame.ordinal,
                json_escape(frame.kind),
                json_escape(&frame.relative_path),
                frame.width,
                frame.height,
                frame.rgba_bytes,
                json_escape(frame.reason),
                frame.sim_tick,
                frame.sample_sequence,
                json_escape(&frame.state_hash),
                badges,
                frame.caption.active_cells,
                frame.caption.subsystem_active_count,
                frame.caption.matter_active_cells,
                frame.caption.thermal_active_cells,
                frame.caption.pressure_active_cells,
                frame.caption.reaction_active_cells,
                frame.caption.sand_count,
                frame.caption.water_count,
                frame.caption.oil_count,
                frame.caption.wood_count,
                frame.caption.ice_count,
                frame.caption.steam_count,
                frame.caption.smoke_count,
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let json = format!(
        concat!(
            "{{\n  \"schema_version\": \"{}\",\n  \"experiment_id\": \"{}\",",
            "\n  \"run_id\": \"{}\",\n  \"scenario\": \"heavy-mixed\",",
            "\n  \"binary_sha256\": \"{}\",\n  \"frame_count\": {},",
            "\n  \"pixel_encoding\": \"rgba8-tightly-packed\",",
            "\n  \"frames\": [{}]\n}}\n"
        ),
        HEAVY_FRAMES_SCHEMA_VERSION,
        json_escape(&config.experiment_id),
        json_escape(&config.run_id),
        json_escape(&config.binary_sha256.to_ascii_lowercase()),
        frames.len(),
        entries,
    );
    write_new(path, json.as_bytes())
}

#[derive(Clone, Debug)]
struct TerminalTrend {
    sample_count: usize,
    start_sim_tick: Option<u64>,
    end_sim_tick: Option<u64>,
    start_temperature_max: Option<f64>,
    end_temperature_max: Option<f64>,
    temperature_positive_steps: usize,
    temperature_runaway: bool,
    start_pressure_max: Option<f64>,
    end_pressure_max: Option<f64>,
    pressure_positive_steps: usize,
    pressure_runaway: bool,
    unbounded_growth: bool,
}

fn terminal_trend(samples: &VecDeque<HeavySampleMetrics>) -> TerminalTrend {
    let sample_count = samples.len();
    let start = samples.front();
    let end = samples.back();
    let temperature_positive_steps = samples
        .iter()
        .zip(samples.iter().skip(1))
        .filter(|(left, right)| right.temperature_max > left.temperature_max)
        .count();
    let pressure_positive_steps = samples
        .iter()
        .zip(samples.iter().skip(1))
        .filter(|(left, right)| right.pressure_max > left.pressure_max)
        .count();
    let sustained = |positive_steps: usize| {
        positive_steps.saturating_mul(4) >= sample_count.saturating_sub(1).saturating_mul(3)
    };
    let temperature_runaway = match (start, end) {
        (Some(start), Some(end)) if sample_count >= 2 => {
            end.temperature_max > start.temperature_max * 1.10 + 1.0
                && sustained(temperature_positive_steps)
        }
        _ => false,
    };
    let pressure_runaway = match (start, end) {
        (Some(start), Some(end)) if sample_count >= 2 => {
            end.pressure_max > start.pressure_max * 1.10 + 1.0 && sustained(pressure_positive_steps)
        }
        _ => false,
    };
    TerminalTrend {
        sample_count,
        start_sim_tick: start.map(|sample| sample.sim_tick),
        end_sim_tick: end.map(|sample| sample.sim_tick),
        start_temperature_max: start.map(|sample| sample.temperature_max),
        end_temperature_max: end.map(|sample| sample.temperature_max),
        temperature_positive_steps,
        temperature_runaway,
        start_pressure_max: start.map(|sample| sample.pressure_max),
        end_pressure_max: end.map(|sample| sample.pressure_max),
        pressure_positive_steps,
        pressure_runaway,
        unbounded_growth: temperature_runaway || pressure_runaway,
    }
}

#[derive(Clone, Debug)]
struct HeavyPredicates {
    matter_movement_observed: PredicateResult,
    density_displacement_observed: PredicateResult,
    thermal_activity_observed: PredicateResult,
    phase_work_observed: PredicateResult,
    combustion_observed: PredicateResult,
    smoke_work_observed: PredicateResult,
    pressure_activity_observed: PredicateResult,
    meaningful_multi_system_overlap: PredicateResult,
    inventory_accounted: PredicateResult,
    no_invalid_materials: PredicateResult,
    no_nonfinite_fields: PredicateResult,
    no_wake_anomalies: PredicateResult,
    no_unbounded_runaway: PredicateResult,
    exact_reset: PredicateResult,
}

impl HeavyPredicates {
    fn statuses(&self) -> [PredicateStatus; 14] {
        [
            self.matter_movement_observed.status,
            self.density_displacement_observed.status,
            self.thermal_activity_observed.status,
            self.phase_work_observed.status,
            self.combustion_observed.status,
            self.smoke_work_observed.status,
            self.pressure_activity_observed.status,
            self.meaningful_multi_system_overlap.status,
            self.inventory_accounted.status,
            self.no_invalid_materials.status,
            self.no_nonfinite_fields.status,
            self.no_wake_anomalies.status,
            self.no_unbounded_runaway.status,
            self.exact_reset.status,
        ]
    }
}

#[derive(Clone, Debug)]
struct ReviewFlags {
    dominant_subsystem: bool,
    dominant_subsystem_name: &'static str,
    dominant_subsystem_share: f64,
    broad_terminal_tail: bool,
    long_thermal_pressure_tail: bool,
    reasons: Vec<&'static str>,
}

fn build_predicates(
    observations: &HeavyObservations,
    trend: &TerminalTrend,
    exact_reset: bool,
) -> HeavyPredicates {
    let observed = |identity: Option<SampleIdentity>, label: &str| {
        identity.map_or_else(
            || PredicateResult::fail(format!("{label} was not observed before max tick")),
            |value| {
                PredicateResult::pass(format!(
                    "{label} first observed at tick {} sample {}",
                    value.sim_tick, value.sample_sequence
                ))
            },
        )
    };
    let meaningful_multi_system_overlap =
        if observations.longest_multi_window.sample_count >= MEANINGFUL_MULTI_SYSTEM_SAMPLES {
            PredicateResult::pass(format!(
                "{} consecutive sampled records had >=3 active subsystems (ticks {}..{}, span={})",
                observations.longest_multi_window.sample_count,
                observations
                    .longest_multi_window
                    .start
                    .map_or(0, |value| value.sim_tick),
                observations
                    .longest_multi_window
                    .end
                    .map_or(0, |value| value.sim_tick),
                observations.longest_multi_window.tick_span(),
            ))
        } else {
            PredicateResult::fail(format!(
                "longest >=3 subsystem sampled window was {} records; required {}",
                observations.longest_multi_window.sample_count, MEANINGFUL_MULTI_SYSTEM_SAMPLES
            ))
        };
    let inventory_accounted = if observations.unexplained_material_occurrences == 0 {
        PredicateResult::pass("all sampled inventory deltas fit the allowed transition model")
    } else {
        PredicateResult::fail(format!(
            "unexplained sampled Material delta occurrences={}",
            observations.unexplained_material_occurrences
        ))
    };
    let no_invalid_materials = if observations.invalid_material_occurrences == 0 {
        PredicateResult::pass("invalid Material ID count was zero in every sample")
    } else {
        PredicateResult::fail(format!(
            "sampled invalid Material ID occurrences={}",
            observations.invalid_material_occurrences
        ))
    };
    let no_nonfinite_fields = if observations.nonfinite_field_occurrences == 0 {
        PredicateResult::pass("non-finite Temperature/Pressure count was zero in every sample")
    } else {
        PredicateResult::fail(format!(
            "sampled non-finite field occurrences={}",
            observations.nonfinite_field_occurrences
        ))
    };
    let no_wake_anomalies = if observations.wake_anomaly_occurrences == 0 {
        PredicateResult::pass("no USER_EDIT or unknown wake-reason bits occurred during production")
    } else {
        PredicateResult::fail(format!(
            "unexpected production wake-reason chunk occurrences={}",
            observations.wake_anomaly_occurrences
        ))
    };
    let no_unbounded_runaway = if trend.sample_count < TERMINAL_WINDOW_SAMPLES {
        PredicateResult::fail(format!(
            "terminal trend contains {} samples; required {}",
            trend.sample_count, TERMINAL_WINDOW_SAMPLES
        ))
    } else if trend.unbounded_growth {
        PredicateResult::fail(format!(
            "terminal runaway rule matched: temperature={} pressure={}",
            trend.temperature_runaway, trend.pressure_runaway
        ))
    } else {
        PredicateResult::pass(format!(
            "terminal {}-sample Temperature/Pressure maxima did not meet runaway rule",
            trend.sample_count
        ))
    };
    let exact_reset = if exact_reset {
        PredicateResult::pass("programmatic R-equivalent state exactly matched pristine tick 0")
    } else {
        PredicateResult::fail("programmatic R-equivalent state differed from pristine tick 0")
    };
    HeavyPredicates {
        matter_movement_observed: observed(observations.first_movement, "Matter movement"),
        density_displacement_observed: observed(
            observations.first_density,
            "Water/Oil density-displacement evidence",
        ),
        thermal_activity_observed: observed(observations.first_thermal, "Thermal activity"),
        phase_work_observed: observed(observations.first_phase, "phase inventory work"),
        combustion_observed: observed(observations.first_combustion, "post-tick combustion work"),
        smoke_work_observed: observed(
            observations.first_smoke,
            "new decay-age-zero Smoke generation",
        ),
        pressure_activity_observed: observed(observations.first_pressure, "Pressure activity"),
        meaningful_multi_system_overlap,
        inventory_accounted,
        no_invalid_materials,
        no_nonfinite_fields,
        no_wake_anomalies,
        no_unbounded_runaway,
        exact_reset,
    }
}

fn review_flags(observations: &HeavyObservations) -> ReviewFlags {
    let candidates = [
        ("matter", observations.matter.cumulative_active_cells),
        ("thermal", observations.thermal.cumulative_active_cells),
        ("pressure", observations.pressure.cumulative_active_cells),
        ("reaction", observations.reaction.cumulative_active_cells),
    ];
    let total = candidates.iter().map(|(_, value)| *value).sum::<u64>();
    let (dominant_subsystem_name, dominant_cells) = candidates
        .into_iter()
        .max_by_key(|(_, value)| *value)
        .unwrap_or(("none", 0));
    let dominant_subsystem_share = if total == 0 {
        0.0
    } else {
        dominant_cells as f64 / total as f64
    };
    let dominant_subsystem = dominant_subsystem_share >= 0.90;
    let broad_terminal_tail = observations.latest.any_active_cells
        >= observations.latest.total_cells.saturating_add(9) / 10;
    let long_thermal_pressure_tail = observations.latest.thermal_active_cells != 0
        && observations.latest.pressure_active_cells != 0;
    let mut reasons = Vec::new();
    if dominant_subsystem {
        reasons.push("dominant_subsystem");
    }
    if broad_terminal_tail {
        reasons.push("broad_terminal_tail");
    }
    if long_thermal_pressure_tail {
        reasons.push("long_thermal_pressure_tail");
    }
    ReviewFlags {
        dominant_subsystem,
        dominant_subsystem_name,
        dominant_subsystem_share,
        broad_terminal_tail,
        long_thermal_pressure_tail,
        reasons,
    }
}

fn heavy_verdict(predicates: &HeavyPredicates, review: &ReviewFlags) -> ExperimentVerdict {
    if predicates.statuses().contains(&PredicateStatus::Fail) {
        ExperimentVerdict::Fail
    } else if predicates.statuses().contains(&PredicateStatus::Unknown)
        || !review.reasons.is_empty()
    {
        ExperimentVerdict::NeedsHumanReview
    } else {
        ExperimentVerdict::Pass
    }
}

fn identity_fields(prefix: &str, identity: Option<SampleIdentity>) -> String {
    format!(
        "\"{prefix}_tick\":{},\"{prefix}_sample\":{}",
        json_opt_u64(identity.map(|value| value.sim_tick)),
        json_opt_u64(identity.map(|value| value.sample_sequence)),
    )
}

fn subsystem_json(summary: &SubsystemSummary) -> String {
    format!(
        concat!(
            "{{\"peak_cells\":{},\"peak_tick\":{},\"peak_sample\":{},",
            "\"active_sample_count\":{},\"first_tick\":{},\"first_sample\":{},",
            "\"last_tick\":{},\"last_sample\":{},\"cumulative_active_cells\":{}}}"
        ),
        summary.peak_cells,
        json_opt_u64(summary.peak.map(|value| value.sim_tick)),
        json_opt_u64(summary.peak.map(|value| value.sample_sequence)),
        summary.active_sample_count,
        json_opt_u64(summary.first.map(|value| value.sim_tick)),
        json_opt_u64(summary.first.map(|value| value.sample_sequence)),
        json_opt_u64(summary.last.map(|value| value.sim_tick)),
        json_opt_u64(summary.last.map(|value| value.sample_sequence)),
        summary.cumulative_active_cells,
    )
}

fn predicate_json(name: &str, predicate: &PredicateResult) -> String {
    format!(
        "\"{}\":{{\"status\":\"{}\",\"detail\":\"{}\"}}",
        json_escape(name),
        predicate.status.as_str(),
        json_escape(&predicate.detail),
    )
}

#[allow(clippy::too_many_arguments)]
fn write_analysis_json(
    config: &super::ExperimentWorkerConfig,
    provenance: &RuntimeProvenance,
    simulation: &Simulation,
    path: &Path,
    baseline: &HeavyBaseline,
    observations: &HeavyObservations,
    trend: &TerminalTrend,
    terminal: &HeavySampleMetrics,
    reset: &HeavySampleMetrics,
    predicates: &HeavyPredicates,
    review: &ReviewFlags,
    verdict: ExperimentVerdict,
    sample_count: u64,
    raw_frame_count: usize,
    exact_reset: bool,
) -> Result<(), String> {
    let predicate_entries = [
        predicate_json(
            "matter_movement_observed",
            &predicates.matter_movement_observed,
        ),
        predicate_json(
            "density_displacement_observed",
            &predicates.density_displacement_observed,
        ),
        predicate_json(
            "thermal_activity_observed",
            &predicates.thermal_activity_observed,
        ),
        predicate_json("phase_work_observed", &predicates.phase_work_observed),
        predicate_json("combustion_observed", &predicates.combustion_observed),
        predicate_json("smoke_work_observed", &predicates.smoke_work_observed),
        predicate_json(
            "pressure_activity_observed",
            &predicates.pressure_activity_observed,
        ),
        predicate_json(
            "meaningful_multi_system_overlap",
            &predicates.meaningful_multi_system_overlap,
        ),
        predicate_json("inventory_accounted", &predicates.inventory_accounted),
        predicate_json("no_invalid_materials", &predicates.no_invalid_materials),
        predicate_json("no_nonfinite_fields", &predicates.no_nonfinite_fields),
        predicate_json("no_wake_anomalies", &predicates.no_wake_anomalies),
        predicate_json("no_unbounded_runaway", &predicates.no_unbounded_runaway),
        predicate_json("exact_reset", &predicates.exact_reset),
    ]
    .join(",");
    let baseline_counts = baseline
        .material_counts_by_id
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let final_counts = terminal
        .material_counts_by_id
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let final_deltas = terminal
        .material_count_deltas_by_id
        .iter()
        .map(i64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let review_reasons = review
        .reasons
        .iter()
        .map(|reason| format!("\"{}\"", json_escape(reason)))
        .collect::<Vec<_>>()
        .join(",");
    let firsts = [
        identity_fields("first_movement", observations.first_movement),
        identity_fields("first_density_displacement", observations.first_density),
        identity_fields("first_thermal_activity", observations.first_thermal),
        identity_fields("first_phase_transition", observations.first_phase),
        identity_fields("first_combustion_work", observations.first_combustion),
        identity_fields("first_smoke_generation", observations.first_smoke),
        identity_fields("first_pressure_activity", observations.first_pressure),
        identity_fields("first_relief_damage", observations.first_relief_damage),
        identity_fields("first_rupture", observations.first_rupture),
        identity_fields("first_opening", observations.first_opening),
        identity_fields("first_vent", observations.first_vent),
        identity_fields("first_three_subsystems", observations.first_three),
        identity_fields("first_all_intended_subsystems", observations.first_all),
    ]
    .join(",");
    let matter_delta = i128::from(terminal.matter_count) - i128::from(baseline.matter_count);
    let sand_delta = i128::from(terminal.sand_count) - i128::from(baseline.sand_count);
    let water_delta = i128::from(terminal.water_count) - i128::from(baseline.water_count);
    let oil_delta = i128::from(terminal.oil_count) - i128::from(baseline.oil_count);
    let wood_delta = i128::from(terminal.wood_count) - i128::from(baseline.wood_count);
    let ice_delta = i128::from(terminal.ice_count) - i128::from(baseline.ice_count);
    let steam_delta = i128::from(terminal.steam_count) - i128::from(baseline.steam_count);
    let smoke_delta = i128::from(terminal.smoke_count) - i128::from(baseline.smoke_count);
    let phase_pool_delta =
        i128::from(terminal.phase_pool_count) - i128::from(baseline.phase_pool_count);
    let fuel_delta = i128::from(terminal.fuel_count) - i128::from(baseline.fuel_count);
    let json = format!(
        concat!(
            "{{\n  \"schema_version\": \"{}\",\n  \"experiment_id\": \"{}\",",
            "\n  \"run_id\": \"{}\",\n  \"scenario\": \"heavy-mixed\",",
            "\n  \"source_sha\": \"{}\",\n  \"git_state\": \"{}\",",
            "\n  \"build_profile\": \"{}\",\n  \"binary_sha256\": \"{}\",",
            "\n  \"world\": {{\"width\": {}, \"height\": {}, \"chunk_size\": {}}},",
            "\n  \"sleep\": {{\"enabled\": {}, \"threshold\": {}}},",
            "\n  \"lifecycle\": {{\"terminal_reason\": \"max-ticks\",",
            "\"terminal_tick\": {}, \"terminal_sample\": {},",
            "\"required_max_ticks\": {}, \"diagnostic_interval_ticks\": {},",
            "\"terminal_window_samples\": {}}},",
            "\n  \"baseline\": {{\"material_counts_by_id\": [{}],",
            "\"matter_count\": {},\"sand_count\": {},\"water_count\": {},",
            "\"oil_count\": {},\"wood_count\": {},\"ice_count\": {},",
            "\"steam_count\": {},\"smoke_count\": {},\"phase_pool_count\": {},",
            "\"fuel_count\": {},\"relief_seam_wood_count\": {},",
            "\"exterior_steam_cells\": {},\"density_ordered_pairs\": {}}},",
            "\n  \"metrics\": {{{},",
            "\"peak_active_cells\": {},\"peak_active_tick\": {},\"peak_active_sample\": {},",
            "\"peak_concurrent_subsystem_count\": {},\"peak_concurrency_tick\": {},",
            "\"peak_concurrency_sample\": {},",
            "\"longest_three_plus_window_samples\": {},",
            "\"longest_three_plus_window_start_tick\": {},",
            "\"longest_three_plus_window_start_sample\": {},",
            "\"longest_three_plus_window_end_tick\": {},",
            "\"longest_three_plus_window_end_sample\": {},",
            "\"longest_three_plus_window_tick_span\": {},",
            "\"subsystems\":{{\"matter\":{},\"thermal\":{},\"pressure\":{},\"reaction\":{}}},",
            "\"initial_material_counts_by_id\":[{}],\"final_material_counts_by_id\":[{}],",
            "\"final_material_count_deltas_by_id\":[{}],",
            "\"initial_matter\":{},\"final_matter\":{},\"matter_delta\":{},",
            "\"initial_sand\":{},\"final_sand\":{},\"sand_delta\":{},",
            "\"initial_water\":{},\"final_water\":{},\"water_delta\":{},",
            "\"initial_oil\":{},\"final_oil\":{},\"oil_delta\":{},",
            "\"initial_wood\":{},\"final_wood\":{},\"wood_delta\":{},",
            "\"initial_ice\":{},\"final_ice\":{},\"ice_delta\":{},",
            "\"initial_steam\":{},\"final_steam\":{},\"steam_delta\":{},",
            "\"initial_smoke\":{},\"smoke_peak\":{},\"smoke_peak_tick\":{},",
            "\"smoke_peak_sample\":{},\"final_smoke\":{},\"smoke_delta\":{},",
            "\"initial_phase_pool\":{},\"final_phase_pool\":{},\"phase_pool_delta\":{},",
            "\"initial_fuel\":{},\"final_fuel\":{},\"fuel_delta\":{},",
            "\"gross_inventory_delta_cells\":{},\"explained_material_delta_cells\":{},",
            "\"unexplained_material_delta_cells\":{},",
            "\"unexplained_material_delta_occurrences\":{},",
            "\"terminal_activity\":{{\"any_active_cells\":{},\"active_chunks\":{},",
            "\"runnable_chunks\":{},\"sleeping_chunks\":{},",
            "\"matter_active_cells\":{},\"thermal_active_cells\":{},",
            "\"pressure_active_cells\":{},\"reaction_active_cells\":{},",
            "\"subsystem_active_count\":{}}},",
            "\"terminal_bounds\":{{\"temperature_min\":{},\"temperature_max\":{},",
            "\"pressure_min\":{},\"pressure_max\":{}}},",
            "\"relief_seam_wood_final\":{},\"relief_open_lanes_final\":{},",
            "\"exterior_steam_final\":{},",
            "\"invalid_material_occurrences\":{},\"nonfinite_field_occurrences\":{},",
            "\"wake_anomaly_occurrences\":{},\"zero_activity_before_overlap_samples\":{},",
            "\"reset_exact_equivalence\":{},\"tick0_state_hash\":\"{}\",",
            "\"reset_state_hash\":\"{}\",\"tick0_physical_state_hash\":\"{}\",",
            "\"reset_physical_state_hash\":\"{}\"}},",
            "\n  \"terminal_trend\": {{\"sample_count\":{},\"start_sim_tick\":{},",
            "\"end_sim_tick\":{},\"start_temperature_max\":{},",
            "\"end_temperature_max\":{},\"temperature_positive_steps\":{},",
            "\"temperature_runaway\":{},\"start_pressure_max\":{},",
            "\"end_pressure_max\":{},\"pressure_positive_steps\":{},",
            "\"pressure_runaway\":{},\"unbounded_growth\":{}}},",
            "\n  \"predicates\": {{{}}},",
            "\n  \"review_flags\": {{\"dominant_subsystem\":{},",
            "\"dominant_subsystem_name\":\"{}\",\"dominant_subsystem_share\":{},",
            "\"broad_terminal_tail\":{},\"long_thermal_pressure_tail\":{},",
            "\"reasons\":[{}]}},",
            "\n  \"verdict\": \"{}\",\n  \"sample_count\": {},",
            "\n  \"raw_frame_count\": {}\n}}\n"
        ),
        HEAVY_ANALYSIS_SCHEMA_VERSION,
        json_escape(&config.experiment_id),
        json_escape(&config.run_id),
        json_escape(&provenance.source_sha),
        provenance.git_state.as_str(),
        provenance.build_profile,
        json_escape(&config.binary_sha256.to_ascii_lowercase()),
        simulation.world.config.width,
        simulation.world.config.height,
        simulation.world.config.chunk_size,
        simulation.sleep_enabled,
        simulation.sleep_threshold,
        terminal.sim_tick,
        terminal.sample_sequence,
        config.max_ticks,
        config.diagnostic_interval_ticks,
        TERMINAL_WINDOW_SAMPLES,
        baseline_counts,
        baseline.matter_count,
        baseline.sand_count,
        baseline.water_count,
        baseline.oil_count,
        baseline.wood_count,
        baseline.ice_count,
        baseline.steam_count,
        baseline.smoke_count,
        baseline.phase_pool_count,
        baseline.fuel_count,
        baseline.relief_seam_wood_count,
        baseline.exterior_steam_cells,
        baseline.density_ordered_pairs,
        firsts,
        observations.peak_active_cells,
        json_opt_u64(observations.peak_active.map(|value| value.sim_tick)),
        json_opt_u64(observations.peak_active.map(|value| value.sample_sequence)),
        observations.peak_concurrent_subsystems,
        json_opt_u64(observations.peak_concurrency.map(|value| value.sim_tick)),
        json_opt_u64(
            observations
                .peak_concurrency
                .map(|value| value.sample_sequence)
        ),
        observations.longest_multi_window.sample_count,
        json_opt_u64(
            observations
                .longest_multi_window
                .start
                .map(|value| value.sim_tick)
        ),
        json_opt_u64(
            observations
                .longest_multi_window
                .start
                .map(|value| value.sample_sequence)
        ),
        json_opt_u64(
            observations
                .longest_multi_window
                .end
                .map(|value| value.sim_tick)
        ),
        json_opt_u64(
            observations
                .longest_multi_window
                .end
                .map(|value| value.sample_sequence)
        ),
        observations.longest_multi_window.tick_span(),
        subsystem_json(&observations.matter),
        subsystem_json(&observations.thermal),
        subsystem_json(&observations.pressure),
        subsystem_json(&observations.reaction),
        baseline_counts,
        final_counts,
        final_deltas,
        baseline.matter_count,
        terminal.matter_count,
        matter_delta,
        baseline.sand_count,
        terminal.sand_count,
        sand_delta,
        baseline.water_count,
        terminal.water_count,
        water_delta,
        baseline.oil_count,
        terminal.oil_count,
        oil_delta,
        baseline.wood_count,
        terminal.wood_count,
        wood_delta,
        baseline.ice_count,
        terminal.ice_count,
        ice_delta,
        baseline.steam_count,
        terminal.steam_count,
        steam_delta,
        baseline.smoke_count,
        observations.smoke_peak,
        json_opt_u64(observations.smoke_peak_identity.map(|value| value.sim_tick)),
        json_opt_u64(
            observations
                .smoke_peak_identity
                .map(|value| value.sample_sequence)
        ),
        terminal.smoke_count,
        smoke_delta,
        baseline.phase_pool_count,
        terminal.phase_pool_count,
        phase_pool_delta,
        baseline.fuel_count,
        terminal.fuel_count,
        fuel_delta,
        terminal.gross_inventory_delta_cells,
        terminal.explained_material_delta_cells,
        terminal.unexplained_material_delta_cells,
        observations.unexplained_material_occurrences,
        terminal.any_active_cells,
        terminal.active_chunks,
        terminal.runnable_chunks,
        terminal.sleeping_chunks,
        terminal.matter_active_cells,
        terminal.thermal_active_cells,
        terminal.pressure_active_cells,
        terminal.reaction_active_cells,
        terminal.subsystem_active_count,
        terminal.temperature_min,
        terminal.temperature_max,
        terminal.pressure_min,
        terminal.pressure_max,
        terminal.relief_seam_wood_count,
        terminal.relief_open_lanes,
        terminal.exterior_steam_cells,
        observations.invalid_material_occurrences,
        observations.nonfinite_field_occurrences,
        observations.wake_anomaly_occurrences,
        observations.zero_activity_before_overlap_samples,
        exact_reset,
        json_escape(&baseline.state_hash),
        json_escape(&reset.state_hash),
        json_escape(&baseline.physical_state_hash),
        json_escape(&reset.physical_state_hash),
        trend.sample_count,
        json_opt_u64(trend.start_sim_tick),
        json_opt_u64(trend.end_sim_tick),
        trend.start_temperature_max.unwrap_or(0.0),
        trend.end_temperature_max.unwrap_or(0.0),
        trend.temperature_positive_steps,
        trend.temperature_runaway,
        trend.start_pressure_max.unwrap_or(0.0),
        trend.end_pressure_max.unwrap_or(0.0),
        trend.pressure_positive_steps,
        trend.pressure_runaway,
        trend.unbounded_growth,
        predicate_entries,
        review.dominant_subsystem,
        review.dominant_subsystem_name,
        review.dominant_subsystem_share,
        review.broad_terminal_tail,
        review.long_thermal_pressure_tail,
        review_reasons,
        verdict.as_str(),
        sample_count,
        raw_frame_count,
    );
    write_new(path, json.as_bytes())
}

fn event_detail(metrics: &HeavySampleMetrics) -> String {
    format!(
        "active={};subsystems={};matter={};thermal={};pressure={};reaction={};state_hash={}",
        metrics.any_active_cells,
        metrics.subsystem_active_count,
        metrics.matter_active_cells,
        metrics.thermal_active_cells,
        metrics.pressure_active_cells,
        metrics.reaction_active_cells,
        metrics.state_hash,
    )
}

fn record_observation_events(
    output: &mut HeavyJsonlWriters,
    config: &super::ExperimentWorkerConfig,
    update: ObservationUpdate,
    metrics: &HeavySampleMetrics,
) -> Result<Vec<FrameBadge>, String> {
    let mut badges = Vec::new();
    let mut event = |condition: bool,
                     event_name: &'static str,
                     badge_kind: Option<&'static str>,
                     reason: &'static str,
                     detail: String|
     -> Result<(), String> {
        if condition {
            output.event(
                config,
                event_name,
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                &detail,
            )?;
            if let Some(kind) = badge_kind {
                badges.push(FrameBadge { kind, reason });
            }
        }
        Ok(())
    };
    event(
        update.first_movement,
        "first_movement_observed",
        Some("first-movement"),
        "Sand-position-change",
        format!(
            "sand_position_changed_cells={}",
            metrics.sand_position_changed_cells
        ),
    )?;
    event(
        update.first_density,
        "first_density_displacement_observed",
        Some("first-density"),
        "ordered-Water-Oil-displacement",
        format!(
            "density_ordered_pairs={};interface_edges={};liquid_position_changed_cells={}",
            metrics.density_ordered_pairs,
            metrics.water_oil_interface_edges,
            metrics.liquid_position_changed_cells
        ),
    )?;
    event(
        update.first_thermal,
        "first_thermal_activity_observed",
        None,
        "Thermal-activity",
        format!("thermal_active_cells={}", metrics.thermal_active_cells),
    )?;
    event(
        update.first_phase,
        "first_phase_transition_observed",
        Some("first-phase"),
        "phase-inventory-change",
        format!(
            "ice={};water={};steam={};phase_pool={}",
            metrics.ice_count, metrics.water_count, metrics.steam_count, metrics.phase_pool_count
        ),
    )?;
    event(
        update.first_combustion,
        "first_combustion_work_observed",
        Some("first-combustion"),
        "post-tick-combustion-work",
        format!(
            "flame_events={};wood_fuel_progress={};oil_fuel_progress={}",
            metrics
                .flame_event_wood_cells
                .saturating_add(metrics.flame_event_oil_cells),
            metrics.wood_fuel_progress_sum,
            metrics.oil_fuel_progress_sum
        ),
    )?;
    event(
        update.first_smoke,
        "first_smoke_generation_observed",
        Some("first-smoke"),
        "new-decay-age-zero-Smoke",
        format!("new_smoke_cells={}", metrics.new_smoke_cells),
    )?;
    event(
        update.first_pressure,
        "first_pressure_activity_observed",
        Some("first-pressure"),
        "Pressure-activity",
        format!("pressure_active_cells={}", metrics.pressure_active_cells),
    )?;
    event(
        update.first_relief_damage,
        "first_relief_damage_observed",
        None,
        "relief-seam-Wood-loss",
        format!(
            "seam_wood={};seam_combusting={};adjacent_pressure_max={}",
            metrics.relief_seam_wood_count,
            metrics.relief_seam_combusting_cells,
            metrics.relief_seam_max_adjacent_pressure
        ),
    )?;
    event(
        update.first_rupture,
        "first_rupture_observed",
        Some("first-rupture"),
        "pressure-threshold-noncombusting-relief-damage",
        format!(
            "seam_wood={};adjacent_pressure_max={};threshold={}",
            metrics.relief_seam_wood_count,
            metrics.relief_seam_max_adjacent_pressure,
            WOOD_RUPTURE_THRESHOLD
        ),
    )?;
    event(
        update.first_opening,
        "first_opening_observed",
        None,
        "through-relief-opening",
        format!("relief_open_lanes={}", metrics.relief_open_lanes),
    )?;
    event(
        update.first_vent,
        "first_vent_observed",
        Some("first-vent"),
        "exterior-Steam-above-relief",
        format!("exterior_steam_cells={}", metrics.exterior_steam_cells),
    )?;
    event(
        update.first_three,
        "first_three_subsystems_observed",
        None,
        ">=3-subsystems-active",
        event_detail(metrics),
    )?;
    event(
        update.first_all,
        "first_all_intended_subsystems_observed",
        None,
        "all-four-subsystems-active",
        event_detail(metrics),
    )?;
    event(
        update.new_peak_active,
        "new_peak_active",
        Some("peak-active"),
        "new-peak-active-cells",
        format!("active_cells={}", metrics.any_active_cells),
    )?;
    event(
        update.new_peak_concurrency,
        "new_peak_concurrency",
        Some("peak-concurrency"),
        "new-peak-subsystem-concurrency",
        format!("subsystem_active_count={}", metrics.subsystem_active_count),
    )?;
    Ok(badges)
}

fn validate_heavy_worker_config(
    simulation: &Simulation,
    config: &super::ExperimentWorkerConfig,
) -> Result<(), String> {
    if config.experiment_id != HEAVY_EXPERIMENT_ID {
        return Err(format!(
            "Heavy experiment_id must be '{HEAVY_EXPERIMENT_ID}', got '{}'",
            config.experiment_id
        ));
    }
    if !is_safe_identifier(&config.run_id) {
        return Err("run_id must contain only ASCII letters, digits, '.', '_' or '-'".to_string());
    }
    if !config.run_id.starts_with(HEAVY_EXPERIMENT_ID) {
        return Err(format!(
            "Heavy run_id must start with '{HEAVY_EXPERIMENT_ID}'"
        ));
    }
    if !config.run_dir.is_dir() {
        return Err(format!(
            "run_dir must already exist as a unique directory: {}",
            display_path(&config.run_dir)
        ));
    }
    if config.run_dir.file_name().and_then(|value| value.to_str()) != Some(&config.run_id) {
        return Err("run_dir leaf must exactly match run_id".to_string());
    }
    if config.scenario != ScenarioId::HeavyMixedWorld {
        return Err(format!(
            "Heavy experiment v0 supports only HeavyMixedWorld, got {}",
            config.scenario
        ));
    }
    if simulation.world.config != REQUIRED_WORLD {
        return Err(format!(
            "Heavy experiment v0 requires WorldConfig 256x256x64, got {}x{}x{}",
            simulation.world.config.width,
            simulation.world.config.height,
            simulation.world.config.chunk_size
        ));
    }
    if !simulation.sleep_enabled {
        return Err("Heavy experiment v0 requires simulation sleep to be enabled".to_string());
    }
    if config.max_ticks != REQUIRED_MAX_TICKS {
        return Err(format!("Heavy max_ticks must be {REQUIRED_MAX_TICKS}"));
    }
    if config.diagnostic_interval_ticks != REQUIRED_DIAGNOSTIC_INTERVAL_TICKS {
        return Err(format!(
            "Heavy diagnostic_interval_ticks must be {REQUIRED_DIAGNOSTIC_INTERVAL_TICKS}"
        ));
    }
    if config.consecutive_all_sleep != 0
        || config.post_sleep_ticks != 0
        || config.consecutive_reaction_zero != 0
        || config.post_reaction_ticks != 0
        || super::pressure_lifecycle_options_present(
            config.consecutive_persistent_opening,
            config.post_opening_ticks,
            config.terminal_window_samples,
        )
    {
        return Err("Heavy worker rejects Sand/Water/Fire/Pressure lifecycle settings".to_string());
    }
    if config.binary_sha256.len() != 64
        || !config
            .binary_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("binary_sha256 must contain exactly 64 hexadecimal characters".to_string());
    }
    Ok(())
}

/// Runs the unchanged Heavy Mixed World fixture through production ticks and
/// records scenario-specific evidence. Semantic failure is an outcome;
/// configuration, GPU, renderer, and filesystem failures remain `Err`.
pub fn run_heavy_mixed_experiment(
    simulation: &mut Simulation,
    renderer: &mut Renderer,
    provenance: &RuntimeProvenance,
    config: &super::ExperimentWorkerConfig,
) -> Result<ExperimentOutcome, String> {
    validate_heavy_worker_config(simulation, config)?;

    let telemetry_dir = config.run_dir.join("telemetry");
    let work_dir = config.run_dir.join("work");
    let raw_frames_dir = work_dir.join("frames");
    create_worker_directory(&telemetry_dir)?;
    create_worker_directory(&work_dir)?;
    create_worker_directory(&raw_frames_dir)?;
    let samples_path = telemetry_dir.join("samples.jsonl");
    let events_path = telemetry_dir.join("events.jsonl");
    let analysis_path = work_dir.join("analysis.json");
    let frames_path = work_dir.join("frames.json");
    let mut output = HeavyJsonlWriters::new(&samples_path, &events_path)?;
    output.event(
        config,
        "lifecycle_started",
        simulation.tick_count,
        None,
        "Heavy Mixed worker output opened",
    )?;

    reset_and_stage_scenario(simulation, config.scenario)
        .map_err(|error| format!("pristine Heavy Mixed reset/stage failed: {error}"))?;
    output.event(
        config,
        "pristine_reset_completed",
        0,
        None,
        "shared Heavy Mixed reset/staging completed",
    )?;
    let baseline_sleep_enabled = simulation.sleep_enabled;
    let baseline_sleep_threshold = simulation.sleep_threshold;
    let mut next_sample_sequence = 0u64;
    let tick0_snapshot = capture_gpu_snapshot(simulation)?;
    let mut tick0_metrics = heavy_metrics_from_snapshot(
        &tick0_snapshot,
        simulation.world.config,
        None,
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "initial",
        "tick0",
    )?;
    let baseline = baseline_from_tick0(&tick0_snapshot, &tick0_metrics);
    let mut observations = HeavyObservations::new(tick0_metrics.clone());
    let _ = observations.observe(&mut tick0_metrics, &baseline, false);
    output.sample(config, provenance, simulation, &tick0_metrics)?;
    let tick0_frame = capture_heavy_frame(
        renderer,
        &tick0_metrics,
        vec![FrameBadge {
            kind: "tick0",
            reason: "pristine-reset",
        }],
    )?;
    output.event(
        config,
        "tick0_captured",
        0,
        Some(tick0_metrics.sample_sequence),
        &tick0_metrics.state_hash,
    )?;

    let mut frames = vec![tick0_frame];
    let mut peak_active_frame = None;
    let mut peak_concurrency_frame = None;
    let mut terminal_window = VecDeque::with_capacity(TERMINAL_WINDOW_SAMPLES);
    let representative_targets = [2u64, 2_500, 5_000, 7_500, 10_000, 12_500, 15_000, 17_500];
    let mut next_representative = 0usize;
    let terminal_metrics;

    loop {
        if simulation.tick_count >= config.max_ticks {
            return Err("Heavy lifecycle reached max tick without max-tick diagnostic".to_string());
        }
        simulation.tick().map_err(|error| {
            format!(
                "Heavy production tick {} failed: {error}",
                simulation.tick_count + 1
            )
        })?;
        let sim_tick = simulation.tick_count;
        let is_tick1 = sim_tick == 1;
        let is_early = sim_tick == 2;
        let is_cadence = sim_tick.is_multiple_of(config.diagnostic_interval_ticks);
        let is_max = sim_tick == config.max_ticks;
        if !is_tick1 && !is_early && !is_cadence && !is_max {
            continue;
        }
        let reason = if is_tick1 {
            "tick1"
        } else if is_early {
            "early-diagnostic"
        } else if is_max {
            "max-tick"
        } else {
            "diagnostic-cadence"
        };
        let snapshot = capture_gpu_snapshot(simulation)?;
        let mut metrics = heavy_metrics_from_snapshot(
            &snapshot,
            simulation.world.config,
            Some(&baseline),
            take_sequence(&mut next_sample_sequence),
            sim_tick,
            "mixed",
            reason,
        )?;
        let update = observations.observe(&mut metrics, &baseline, true);
        output.sample(config, provenance, simulation, &metrics)?;
        terminal_window.push_back(metrics.clone());
        if terminal_window.len() > TERMINAL_WINDOW_SAMPLES {
            terminal_window.pop_front();
        }
        let mut badges = record_observation_events(&mut output, config, update, &metrics)?;
        if is_tick1 {
            badges.push(FrameBadge {
                kind: "tick1",
                reason: "after-one-production-tick",
            });
            output.event(
                config,
                "tick1_captured",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                &metrics.state_hash,
            )?;
        }
        if next_representative < representative_targets.len()
            && sim_tick >= representative_targets[next_representative]
        {
            let target = representative_targets[next_representative];
            let (kind, representative_reason) = if target == config.max_ticks / 2 {
                ("mid-run", "representative-mid-run")
            } else if target == config.max_ticks.saturating_mul(3) / 4 {
                ("late-run", "representative-late-run")
            } else {
                ("representative", "scheduled-mixed-state")
            };
            badges.push(FrameBadge {
                kind,
                reason: representative_reason,
            });
            next_representative += 1;
        }
        if is_max {
            badges.push(FrameBadge {
                kind: "terminal",
                reason: "max-tick-reached",
            });
        }
        let needs_peak_active = update.new_peak_active;
        let needs_peak_concurrency = update.new_peak_concurrency;
        if !badges.is_empty() {
            let frame = capture_heavy_frame(renderer, &metrics, badges)?;
            if needs_peak_active {
                peak_active_frame = Some(frame.clone_with_badges(vec![FrameBadge {
                    kind: "peak-active",
                    reason: "maximum-observed-active-cells",
                }]));
            }
            if needs_peak_concurrency {
                peak_concurrency_frame = Some(frame.clone_with_badges(vec![FrameBadge {
                    kind: "peak-concurrency",
                    reason: "maximum-observed-subsystem-concurrency",
                }]));
            }
            let milestone_badges = frame
                .badges
                .iter()
                .filter(|badge| !matches!(badge.kind, "peak-active" | "peak-concurrency"))
                .cloned()
                .collect::<Vec<_>>();
            if !milestone_badges.is_empty() {
                frames.push(frame.clone_with_badges(milestone_badges));
            }
        }
        if is_max {
            terminal_metrics = metrics;
            break;
        }
    }
    output.event(
        config,
        "terminal_selected",
        terminal_metrics.sim_tick,
        Some(terminal_metrics.sample_sequence),
        "max-ticks",
    )?;

    output.event(
        config,
        "reset_started",
        terminal_metrics.sim_tick,
        Some(terminal_metrics.sample_sequence),
        "programmatic R-equivalent shared Heavy Mixed reset/staging",
    )?;
    reset_and_stage_scenario(simulation, config.scenario)
        .map_err(|error| format!("programmatic Heavy Mixed reset failed: {error}"))?;
    let reset_snapshot = capture_gpu_snapshot(simulation)?;
    let mut reset_metrics = heavy_metrics_from_snapshot(
        &reset_snapshot,
        simulation.world.config,
        Some(&baseline),
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "reset",
        "programmatic-r-equivalent",
    )?;
    apply_inventory_accounting(&mut reset_metrics, &baseline, EvidenceContext::default());
    output.sample(config, provenance, simulation, &reset_metrics)?;
    let exact_reset = exact_reset_equal(&tick0_snapshot, &reset_snapshot)
        && simulation.tick_count == tick0_metrics.sim_tick
        && simulation.sleep_enabled == baseline_sleep_enabled
        && simulation.sleep_threshold == baseline_sleep_threshold;
    output.event(
        config,
        "reset_comparison_completed",
        reset_metrics.sim_tick,
        Some(reset_metrics.sample_sequence),
        if exact_reset {
            "exact=true"
        } else {
            "exact=false"
        },
    )?;
    frames.push(capture_heavy_frame(
        renderer,
        &reset_metrics,
        vec![FrameBadge {
            kind: "reset",
            reason: "programmatic-r-equivalent",
        }],
    )?);
    if let Some(frame) = peak_active_frame {
        frames.push(frame);
    }
    if let Some(frame) = peak_concurrency_frame {
        frames.push(frame);
    }

    let trend = terminal_trend(&terminal_window);
    let predicates = build_predicates(&observations, &trend, exact_reset);
    let review = review_flags(&observations);
    let verdict = heavy_verdict(&predicates, &review);
    let frames = fold_and_order_frames(frames);
    if !(MIN_RAW_FRAMES..=MAX_RAW_FRAMES).contains(&frames.len()) {
        return Err(format!(
            "completed Heavy lifecycle produced {} frames; required {MIN_RAW_FRAMES}..={MAX_RAW_FRAMES}",
            frames.len()
        ));
    }
    let written_frames = write_heavy_raw_frames(&raw_frames_dir, frames)?;
    write_frames_json(config, &frames_path, &written_frames)?;
    write_analysis_json(
        config,
        provenance,
        simulation,
        &analysis_path,
        &baseline,
        &observations,
        &trend,
        &terminal_metrics,
        &reset_metrics,
        &predicates,
        &review,
        verdict,
        next_sample_sequence,
        written_frames.len(),
        exact_reset,
    )?;
    output.event(
        config,
        "worker_completed",
        reset_metrics.sim_tick,
        Some(reset_metrics.sample_sequence),
        verdict.as_str(),
    )?;
    output.flush()?;

    Ok(ExperimentOutcome {
        experiment_id: config.experiment_id.clone(),
        run_id: config.run_id.clone(),
        verdict,
        analysis_path,
        frames_path,
        samples_path,
        events_path,
        sample_count: next_sample_sequence,
        raw_frame_count: written_frames.len(),
        first_all_sleep_sim_tick: None,
        first_all_sleep_sample_sequence: None,
        post_sleep_end_tick: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_snapshot() -> GpuSnapshot {
        let cells = (REQUIRED_WORLD.width * REQUIRED_WORLD.height) as usize;
        let chunks = 16usize;
        GpuSnapshot {
            material_current: vec![MATERIAL_EMPTY; cells],
            material_next: vec![MATERIAL_EMPTY; cells],
            temperature_current: vec![0.0f32.to_bits(); cells],
            temperature_next: vec![0.0f32.to_bits(); cells],
            pressure_current: vec![0.0f32.to_bits(); cells],
            pressure_next: vec![0.0f32.to_bits(); cells],
            flags_current: vec![0; cells],
            flags_next: vec![0; cells],
            proposal: vec![u32::MAX; cells],
            claim: vec![0; cells],
            cell_activity: vec![0; cells],
            chunk_activity: vec![0; chunks],
            chunk_changed: vec![0; chunks],
            chunk_stable: vec![0; chunks],
            chunk_edit_wake: vec![0; chunks],
            chunk_state: vec![CHUNK_STATE_RUNNABLE; chunks],
            chunk_wake_reason: vec![0; chunks],
            params: vec![0; 8],
            wake_params: vec![0; 4],
            arbitration_params: vec![0; 4],
        }
    }

    fn index(x: usize, y: usize) -> usize {
        y * REQUIRED_WORLD.width as usize + x
    }

    fn sample(
        snapshot: &GpuSnapshot,
        baseline: Option<&HeavyBaseline>,
        sequence: u64,
        tick: u64,
    ) -> HeavySampleMetrics {
        heavy_metrics_from_snapshot(
            snapshot,
            REQUIRED_WORLD,
            baseline,
            sequence,
            tick,
            if tick == 0 { "initial" } else { "mixed" },
            if tick == 0 {
                "tick0"
            } else {
                "diagnostic-cadence"
            },
        )
        .expect("valid synthetic Heavy sample")
    }

    fn frame(kind: &'static str, tick: u64, sequence: u64, hash: &str) -> HeavyFrame {
        HeavyFrame {
            sim_tick: tick,
            sample_sequence: sequence,
            state_hash: hash.to_string(),
            badges: vec![FrameBadge {
                kind,
                reason: "test",
            }],
            caption: FrameCaptionMetrics {
                active_cells: 0,
                subsystem_active_count: 0,
                matter_active_cells: 0,
                thermal_active_cells: 0,
                pressure_active_cells: 0,
                reaction_active_cells: 0,
                sand_count: 0,
                water_count: 0,
                oil_count: 0,
                wood_count: 0,
                ice_count: 0,
                steam_count: 0,
                smoke_count: 0,
            },
            frame: RawFrame {
                width: 1,
                height: 1,
                rgba: vec![0, 0, 0, 255],
            },
        }
    }

    #[test]
    fn relief_passability_accepts_only_empty_and_gases() {
        for material in [MATERIAL_EMPTY, MATERIAL_STEAM, MATERIAL_SMOKE] {
            assert!(is_relief_passable(material));
        }
        for material in [
            MATERIAL_BOUNDARY_BLOCK,
            MATERIAL_STONE,
            MATERIAL_SAND,
            MATERIAL_WATER,
            MATERIAL_OIL,
            MATERIAL_ICE,
            MATERIAL_WOOD,
            u32::MAX,
        ] {
            assert!(!is_relief_passable(material));
        }
    }

    #[test]
    fn density_requires_actual_post_tick_liquid_displacement_and_interface() {
        let mut tick0 = empty_snapshot();
        tick0.material_current[index(10, 10)] = MATERIAL_WATER;
        tick0.material_current[index(20, 20)] = MATERIAL_OIL;
        let tick0_metrics = sample(&tick0, None, 0, 0);
        assert_eq!(tick0_metrics.density_ordered_pairs, 0);
        let baseline = baseline_from_tick0(&tick0, &tick0_metrics);
        let mut observations = HeavyObservations::new(tick0_metrics.clone());
        let mut initial = tick0_metrics;
        observations.observe(&mut initial, &baseline, false);

        let mut static_order = tick0.clone();
        static_order.material_current[index(30, 30)] = MATERIAL_OIL;
        static_order.material_current[index(30, 31)] = MATERIAL_WATER;
        let mut metrics = sample(&static_order, Some(&baseline), 1, 0);
        assert!(
            !observations
                .observe(&mut metrics, &baseline, true)
                .first_density
        );

        let mut metrics = sample(&static_order, Some(&baseline), 2, 8);
        assert!(metrics.liquid_position_changed_cells > 0);
        assert!(metrics.water_oil_interface_edges > 0);
        assert!(
            observations
                .observe(&mut metrics, &baseline, true)
                .first_density
        );
    }

    #[test]
    fn authored_smoke_and_combustion_do_not_count_but_post_tick_work_does() {
        let mut tick0 = empty_snapshot();
        tick0.material_current[index(10, 10)] = MATERIAL_SMOKE;
        tick0.material_current[index(20, 20)] = MATERIAL_WOOD;
        tick0.flags_current[index(20, 20)] = FLAG_COMBUSTING;
        let tick0_metrics = sample(&tick0, None, 0, 0);
        let baseline = baseline_from_tick0(&tick0, &tick0_metrics);
        assert_eq!(tick0_metrics.new_smoke_cells, 0);
        assert!(!tick0_metrics.dynamic_combustion_work);

        let mut tick1 = tick0.clone();
        tick1.flags_current[index(10, 10)] = 1 << 24;
        tick1.material_current[index(20, 19)] = MATERIAL_SMOKE;
        tick1.flags_current[index(20, 19)] = 0;
        tick1.flags_current[index(20, 20)] = FLAG_COMBUSTING | FLAG_FLAME_EVENT | (1 << 8);
        let metrics = sample(&tick1, Some(&baseline), 1, 1);
        assert_eq!(metrics.new_smoke_cells, 1);
        assert!(metrics.dynamic_combustion_work);
    }

    #[test]
    fn causal_rupture_requires_threshold_pressure_and_no_seam_combustion() {
        let mut tick0 = empty_snapshot();
        for y in RELIEF_MIN_Y..RELIEF_MAX_Y {
            for x in RELIEF_MIN_X..RELIEF_MAX_X {
                tick0.material_current[index(x, y)] = MATERIAL_WOOD;
            }
        }
        tick0.material_current[index(RELIEF_MIN_X, RELIEF_MAX_Y)] = MATERIAL_WATER;
        let tick0_metrics = sample(&tick0, None, 0, 0);
        let baseline = baseline_from_tick0(&tick0, &tick0_metrics);

        let mut causal = tick0.clone();
        causal.material_current[index(RELIEF_MIN_X, RELIEF_MAX_Y - 1)] = MATERIAL_EMPTY;
        causal.pressure_current[index(RELIEF_MIN_X, RELIEF_MAX_Y)] =
            WOOD_RUPTURE_THRESHOLD.to_bits();
        causal.cell_activity[index(RELIEF_MIN_X, RELIEF_MAX_Y)] = ACTIVITY_PRESSURE;
        let mut causal_metrics = sample(&causal, Some(&baseline), 1, 8);
        let mut causal_observations = HeavyObservations::new(tick0_metrics.clone());
        let update = causal_observations.observe(&mut causal_metrics, &baseline, true);
        assert!(update.first_relief_damage);
        assert!(update.first_rupture);

        let mut confounded = causal;
        confounded.flags_current[index(RELIEF_MIN_X + 1, RELIEF_MAX_Y - 1)] =
            FLAG_COMBUSTING | FLAG_FLAME_EVENT | (1 << 8);
        let mut confounded_metrics = sample(&confounded, Some(&baseline), 1, 8);
        let mut confounded_observations = HeavyObservations::new(tick0_metrics);
        let update = confounded_observations.observe(&mut confounded_metrics, &baseline, true);
        assert!(update.first_relief_damage);
        assert!(!update.first_rupture);
    }

    #[test]
    fn three_subsystem_window_is_consecutive_sample_based() {
        let tick0 = empty_snapshot();
        let tick0_metrics = sample(&tick0, None, 0, 0);
        let baseline = baseline_from_tick0(&tick0, &tick0_metrics);
        let mut observations = HeavyObservations::new(tick0_metrics);
        for sequence in 1..=3 {
            let mut active = tick0.clone();
            active.cell_activity[0] = ACTIVITY_MATTER | ACTIVITY_THERMAL | ACTIVITY_PRESSURE;
            let mut metrics = sample(&active, Some(&baseline), sequence, sequence * 8);
            observations.observe(&mut metrics, &baseline, true);
        }
        assert_eq!(observations.longest_multi_window.sample_count, 3);
        assert_eq!(observations.longest_multi_window.tick_span(), 16);
        let mut broken = sample(&tick0, Some(&baseline), 4, 32);
        observations.observe(&mut broken, &baseline, true);
        assert_eq!(observations.current_multi_window.sample_count, 0);
        assert_eq!(observations.longest_multi_window.sample_count, 3);
    }

    #[test]
    fn inventory_accounting_separates_allowed_phase_work_from_sand_corruption() {
        let mut tick0 = empty_snapshot();
        tick0.material_current[index(1, 1)] = MATERIAL_SAND;
        tick0.material_current[index(2, 2)] = MATERIAL_WATER;
        let tick0_metrics = sample(&tick0, None, 0, 0);
        let baseline = baseline_from_tick0(&tick0, &tick0_metrics);

        let mut allowed = tick0.clone();
        allowed.material_current[index(2, 2)] = MATERIAL_STEAM;
        allowed.material_current[index(2, 3)] = MATERIAL_STEAM;
        let mut allowed_metrics = sample(&allowed, Some(&baseline), 1, 8);
        apply_inventory_accounting(
            &mut allowed_metrics,
            &baseline,
            EvidenceContext {
                phase: true,
                ..EvidenceContext::default()
            },
        );
        assert!(allowed_metrics.inventory_accounted);
        assert_eq!(allowed_metrics.unexplained_material_delta_cells, 0);

        let mut corrupted = allowed;
        corrupted.material_current[index(1, 1)] = MATERIAL_EMPTY;
        let mut corrupted_metrics = sample(&corrupted, Some(&baseline), 2, 16);
        apply_inventory_accounting(
            &mut corrupted_metrics,
            &baseline,
            EvidenceContext {
                phase: true,
                ..EvidenceContext::default()
            },
        );
        assert!(!corrupted_metrics.inventory_accounted);
        assert!(corrupted_metrics.unexplained_material_delta_cells > 0);
    }

    #[test]
    fn wake_anomaly_excludes_staging_but_rejects_production_user_edit_or_unknown_bits() {
        let mut snapshot = empty_snapshot();
        snapshot.chunk_wake_reason[0] = WAKE_REASON_USER_EDIT;
        snapshot.chunk_wake_reason[1] = 1 << 12;
        assert_eq!(sample(&snapshot, None, 0, 0).wake_anomaly_chunks, 0);
        let baseline_metrics = sample(&empty_snapshot(), None, 0, 0);
        let baseline = baseline_from_tick0(&empty_snapshot(), &baseline_metrics);
        assert_eq!(
            sample(&snapshot, Some(&baseline), 1, 8).wake_anomaly_chunks,
            2
        );
    }

    #[test]
    fn terminal_runaway_requires_ten_percent_plus_one_and_three_quarters_positive_steps() {
        let snapshot = empty_snapshot();
        let baseline_metrics = sample(&snapshot, None, 0, 0);
        let baseline = baseline_from_tick0(&snapshot, &baseline_metrics);
        let mut samples = VecDeque::new();
        for index in 0..TERMINAL_WINDOW_SAMPLES {
            let mut metrics = sample(&snapshot, Some(&baseline), index as u64, index as u64 * 8);
            metrics.temperature_max = 10.0 + index as f64;
            metrics.pressure_max = 10.0;
            samples.push_back(metrics);
        }
        let trend = terminal_trend(&samples);
        assert!(trend.temperature_runaway);
        assert!(trend.unbounded_growth);

        for (index, metrics) in samples.iter_mut().enumerate() {
            metrics.temperature_max = 10.0 + f64::from((index % 2) as u32);
        }
        assert!(!terminal_trend(&samples).temperature_runaway);
    }

    #[test]
    fn same_tick_badges_fold_but_exact_reset_stays_distinct_and_last() {
        let folded = fold_and_order_frames(vec![
            frame("first-movement", 8, 2, "h1"),
            frame("first-pressure", 8, 2, "h1"),
            frame("reset", 0, 99, "h0"),
            frame("tick0", 0, 0, "h0"),
        ]);
        assert_eq!(folded.len(), 3);
        assert_eq!(folded[1].badges.len(), 2);
        assert_eq!(folded.last().unwrap().badges[0].kind, "reset");
        assert_eq!(folded[0].badges[0].kind, "tick0");
    }

    #[test]
    fn generic_near_duplicates_are_removed_deterministically_without_crossing_floor() {
        let folded = fold_and_order_frames(vec![
            frame("tick0", 0, 0, "h0"),
            frame("tick1", 1, 1, "h1"),
            frame("representative", 2, 2, "h1"),
            frame("mid-run", 10_000, 3, "h2"),
            frame("late-run", 15_000, 4, "h2"),
            frame("first-phase", 16_000, 5, "h3"),
            frame("first-pressure", 16_100, 6, "h4"),
            frame("first-smoke", 16_200, 7, "h5"),
            frame("first-density", 16_300, 8, "h6"),
            frame("peak-active", 16_400, 9, "h7"),
            frame("terminal", 20_000, 10, "h8"),
            frame("reset", 0, 11, "h0"),
        ]);
        assert_eq!(folded.len(), MIN_RAW_FRAMES);
        assert!(!folded.iter().any(|entry| {
            entry
                .badges
                .iter()
                .any(|badge| matches!(badge.kind, "representative" | "late-run"))
        }));
        assert!(folded
            .iter()
            .any(|entry| entry.badges.iter().any(|badge| badge.kind == "mid-run")));
        assert_eq!(folded.last().unwrap().badges[0].kind, "reset");

        let at_floor = fold_and_order_frames(vec![
            frame("tick0", 0, 0, "h0"),
            frame("tick1", 1, 1, "h1"),
            frame("representative", 2, 2, "h1"),
            frame("first-phase", 3, 3, "h2"),
            frame("first-pressure", 4, 4, "h3"),
            frame("first-smoke", 5, 5, "h4"),
            frame("first-density", 6, 6, "h5"),
            frame("peak-active", 7, 7, "h6"),
            frame("terminal", 20_000, 8, "h7"),
            frame("reset", 0, 9, "h0"),
        ]);
        assert_eq!(at_floor.len(), MIN_RAW_FRAMES);
        assert!(at_floor
            .iter()
            .any(|entry| entry.badges[0].kind == "representative"));
    }

    #[test]
    fn verdict_is_fail_first_then_review_then_pass() {
        let pass = || PredicateResult::pass("ok");
        let predicates = HeavyPredicates {
            matter_movement_observed: pass(),
            density_displacement_observed: pass(),
            thermal_activity_observed: pass(),
            phase_work_observed: pass(),
            combustion_observed: pass(),
            smoke_work_observed: pass(),
            pressure_activity_observed: pass(),
            meaningful_multi_system_overlap: pass(),
            inventory_accounted: pass(),
            no_invalid_materials: pass(),
            no_nonfinite_fields: pass(),
            no_wake_anomalies: pass(),
            no_unbounded_runaway: pass(),
            exact_reset: pass(),
        };
        let review = ReviewFlags {
            dominant_subsystem: false,
            dominant_subsystem_name: "matter",
            dominant_subsystem_share: 0.4,
            broad_terminal_tail: false,
            long_thermal_pressure_tail: false,
            reasons: vec![],
        };
        assert_eq!(heavy_verdict(&predicates, &review), ExperimentVerdict::Pass);
        let mut review_needed = review.clone();
        review_needed.reasons.push("broad_terminal_tail");
        assert_eq!(
            heavy_verdict(&predicates, &review_needed),
            ExperimentVerdict::NeedsHumanReview
        );
        let mut failed = predicates;
        failed.no_wake_anomalies = PredicateResult::fail("wake");
        assert_eq!(
            heavy_verdict(&failed, &review_needed),
            ExperimentVerdict::Fail
        );
    }
}
