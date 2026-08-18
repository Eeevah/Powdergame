//! Pressure Burst Experiment Evidence Harness worker.
//!
//! This worker observes the shared authored Pressure Burst fixture through the
//! ordinary production simulation tick. It owns no staging or physics rules.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use powdergame_core::{
    is_valid_cell_material_value, WorldConfig, ACTIVITY_MATTER, ACTIVITY_PRESSURE,
    ACTIVITY_REACTION, ACTIVITY_THERMAL, CHUNK_STATE_RUNNABLE, CHUNK_STATE_SLEEPING,
    MATERIAL_EMPTY, MATERIAL_STEAM, MATERIAL_WATER, MATERIAL_WOOD,
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

pub const PRESSURE_EXPERIMENT_ID: &str = "g8b-pressure-burst-v0";
const PRESSURE_TELEMETRY_SCHEMA_VERSION: &str = "powdergame-pressure-burst-telemetry-v0";
const PRESSURE_ANALYSIS_SCHEMA_VERSION: &str = "powdergame-pressure-burst-analysis-v0";
const PRESSURE_FRAMES_SCHEMA_VERSION: &str = "powdergame-pressure-burst-frames-v0";
const REQUIRED_MAX_TICKS: u64 = 20_000;
const REQUIRED_DIAGNOSTIC_INTERVAL_TICKS: u64 = 8;
const REQUIRED_PERSISTENT_OPENING_SAMPLES: u32 = 3;
const REQUIRED_POST_OPENING_TICKS: u32 = 180;
const REQUIRED_TERMINAL_WINDOW_SAMPLES: u32 = 64;
const MIN_RAW_FRAMES: usize = 8;
const MAX_RAW_FRAMES: usize = 12;
const DIAGNOSTIC_RING_CAPACITY: usize = 8;

const OUTER_MIN_X: usize = 32;
const OUTER_MAX_X: usize = 224;
const OUTER_MIN_Y: usize = 38;
const OUTER_MAX_Y: usize = 224;
const CAVITY_MIN_X: usize = 40;
const CAVITY_MAX_X: usize = 216;
const CAVITY_MIN_Y: usize = 46;
const CAVITY_MAX_Y: usize = 216;
const TOP_SEAM_MIN_X: usize = 104;
const TOP_SEAM_MAX_X: usize = 152;
const TOP_SEAM_MIN_Y: usize = 38;
const TOP_SEAM_MAX_Y: usize = 46;
const BOTTOM_SEAM_MIN_X: usize = 116;
const BOTTOM_SEAM_MAX_X: usize = 140;
const BOTTOM_SEAM_MIN_Y: usize = 216;
const BOTTOM_SEAM_MAX_Y: usize = 224;
const INITIAL_TOP_SEAM_WOOD: u64 = 384;
const INITIAL_BOTTOM_SEAM_WOOD: u64 = 192;
const INITIAL_TOTAL_SEAM_WOOD: u64 = 576;
const CHAMBER_PRESSURE_CELL_COUNT: u64 = 29_920;

/// Canonical decision value for every Pressure number serialized with `:.9`.
/// Parsing the emitted decimal back to f64 makes Rust predicates and strict
/// JSON consumers operate on the same IEEE-754 value.
fn quantize_json_9(value: f64) -> f64 {
    format!("{value:.9}")
        .parse::<f64>()
        .expect("finite nine-decimal Pressure value must parse")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SampleIdentity {
    sim_tick: u64,
    sample_sequence: u64,
}

#[derive(Clone, Debug)]
struct PressureBaseline {
    matter_count: u64,
    water_count: u64,
    steam_count: u64,
    relief_seam_wood_cells: u64,
    top_relief_seam_wood_cells: u64,
    bottom_relief_seam_wood_cells: u64,
    chamber_pressure_cell_count: u64,
    chamber_mean_pressure: f64,
    chamber_max_pressure: f64,
}

#[derive(Clone, Debug)]
struct PressureSampleMetrics {
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
    total_chunks: u32,
    active_chunks: u32,
    runnable_chunks: u32,
    sleeping_chunks: u32,
    material_counts_by_id: [u64; 10],
    matter_count: u64,
    water_count: u64,
    steam_count: u64,
    relief_seam_wood_cells: u64,
    top_relief_seam_wood_cells: u64,
    bottom_relief_seam_wood_cells: u64,
    relief_seam_open_cells: u64,
    top_relief_seam_open_cells: u64,
    bottom_relief_seam_open_cells: u64,
    steam_in_relief_seam_cells: u64,
    outside_chamber_steam_cells: u64,
    chamber_pressure_cell_count: u64,
    chamber_mean_pressure: f64,
    chamber_max_pressure: f64,
    invalid_material_count: u64,
    nonfinite_temperature_count: u64,
    nonfinite_pressure_count: u64,
    changed_chunks: u32,
    wake_chunks: u32,
    wake_reason_or: u32,
    state_hash: String,
    physical_state_hash: String,
}

impl PressureSampleMetrics {
    fn identity(&self) -> SampleIdentity {
        SampleIdentity {
            sim_tick: self.sim_tick,
            sample_sequence: self.sample_sequence,
        }
    }
}

fn in_top_seam(x: usize, y: usize) -> bool {
    (TOP_SEAM_MIN_X..TOP_SEAM_MAX_X).contains(&x) && (TOP_SEAM_MIN_Y..TOP_SEAM_MAX_Y).contains(&y)
}

fn in_bottom_seam(x: usize, y: usize) -> bool {
    (BOTTOM_SEAM_MIN_X..BOTTOM_SEAM_MAX_X).contains(&x)
        && (BOTTOM_SEAM_MIN_Y..BOTTOM_SEAM_MAX_Y).contains(&y)
}

fn in_outer_chamber(x: usize, y: usize) -> bool {
    (OUTER_MIN_X..OUTER_MAX_X).contains(&x) && (OUTER_MIN_Y..OUTER_MAX_Y).contains(&y)
}

fn in_pressure_cavity(x: usize, y: usize) -> bool {
    (CAVITY_MIN_X..CAVITY_MAX_X).contains(&x) && (CAVITY_MIN_Y..CAVITY_MAX_Y).contains(&y)
}

fn pressure_metrics_from_snapshot(
    snapshot: &GpuSnapshot,
    world: WorldConfig,
    sample_sequence: u64,
    sim_tick: u64,
    phase: &'static str,
    reason: &'static str,
) -> Result<PressureSampleMetrics, String> {
    let expected_cells = u64::from(world.width) * u64::from(world.height);
    if snapshot.material_current.len() as u64 != expected_cells
        || snapshot.temperature_current.len() as u64 != expected_cells
        || snapshot.pressure_current.len() as u64 != expected_cells
        || snapshot.flags_current.len() as u64 != expected_cells
        || snapshot.cell_activity.len() as u64 != expected_cells
    {
        return Err(
            "Pressure GPU snapshot cell-vector lengths do not match WorldConfig".to_string(),
        );
    }
    if snapshot.chunk_activity.len() != snapshot.chunk_state.len()
        || snapshot.chunk_activity.len() != snapshot.chunk_changed.len()
        || snapshot.chunk_activity.len() != snapshot.chunk_wake_reason.len()
    {
        return Err("Pressure GPU snapshot chunk-vector lengths disagree".to_string());
    }

    let width = world.width as usize;
    let mut material_counts_by_id = [0u64; 10];
    let mut matter_count = 0u64;
    let mut invalid_material_count = 0u64;
    let mut top_wood = 0u64;
    let mut bottom_wood = 0u64;
    let mut top_open = 0u64;
    let mut bottom_open = 0u64;
    let mut steam_in_seam = 0u64;
    let mut outside_steam = 0u64;
    let mut chamber_pressure_sum = 0.0f64;
    let mut chamber_pressure_max = 0.0f32;
    let mut chamber_pressure_cells = 0u64;

    for (index, &material) in snapshot.material_current.iter().enumerate() {
        let x = index % width;
        let y = index / width;
        if let Some(slot) = material_counts_by_id.get_mut(material as usize) {
            *slot = slot.saturating_add(1);
        }
        if !is_valid_cell_material_value(material) {
            invalid_material_count = invalid_material_count.saturating_add(1);
        } else if material != MATERIAL_EMPTY {
            matter_count = matter_count.saturating_add(1);
        }

        let top = in_top_seam(x, y);
        let bottom = in_bottom_seam(x, y);
        if top || bottom {
            if material == MATERIAL_WOOD {
                if top {
                    top_wood = top_wood.saturating_add(1);
                } else {
                    bottom_wood = bottom_wood.saturating_add(1);
                }
            } else if top {
                top_open = top_open.saturating_add(1);
            } else {
                bottom_open = bottom_open.saturating_add(1);
            }
            if material == MATERIAL_STEAM {
                steam_in_seam = steam_in_seam.saturating_add(1);
            }
        }
        if material == MATERIAL_STEAM && !in_outer_chamber(x, y) {
            outside_steam = outside_steam.saturating_add(1);
        }
        if in_pressure_cavity(x, y) {
            chamber_pressure_cells = chamber_pressure_cells.saturating_add(1);
            let pressure = f32::from_bits(snapshot.pressure_current[index]);
            if pressure.is_finite() {
                chamber_pressure_sum += f64::from(pressure);
                chamber_pressure_max = chamber_pressure_max.max(pressure);
            }
        }
    }

    let nonfinite_temperature_count = snapshot
        .temperature_current
        .iter()
        .filter(|bits| !f32::from_bits(**bits).is_finite())
        .count() as u64;
    let nonfinite_pressure_count = snapshot
        .pressure_current
        .iter()
        .filter(|bits| !f32::from_bits(**bits).is_finite())
        .count() as u64;
    let chamber_mean_pressure = if chamber_pressure_cells == 0 {
        0.0
    } else {
        chamber_pressure_sum / chamber_pressure_cells as f64
    };
    let chamber_mean_pressure = quantize_json_9(chamber_mean_pressure);
    let chamber_max_pressure = quantize_json_9(f64::from(chamber_pressure_max));
    let water_count = material_counts_by_id[MATERIAL_WATER as usize];
    let steam_count = material_counts_by_id[MATERIAL_STEAM as usize];

    Ok(PressureSampleMetrics {
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
        matter_active_cells: bit_count(&snapshot.cell_activity, ACTIVITY_MATTER),
        thermal_active_cells: bit_count(&snapshot.cell_activity, ACTIVITY_THERMAL),
        pressure_active_cells: bit_count(&snapshot.cell_activity, ACTIVITY_PRESSURE),
        reaction_active_cells: bit_count(&snapshot.cell_activity, ACTIVITY_REACTION),
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
        water_count,
        steam_count,
        relief_seam_wood_cells: top_wood.saturating_add(bottom_wood),
        top_relief_seam_wood_cells: top_wood,
        bottom_relief_seam_wood_cells: bottom_wood,
        relief_seam_open_cells: top_open.saturating_add(bottom_open),
        top_relief_seam_open_cells: top_open,
        bottom_relief_seam_open_cells: bottom_open,
        steam_in_relief_seam_cells: steam_in_seam,
        outside_chamber_steam_cells: outside_steam,
        chamber_pressure_cell_count: chamber_pressure_cells,
        chamber_mean_pressure,
        chamber_max_pressure,
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
            .fold(0u32, |acc, value| acc | value),
        state_hash: authoritative_current_hash(snapshot),
        physical_state_hash: physical_state_hash(snapshot),
    })
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

fn baseline_from_tick0(metrics: &PressureSampleMetrics) -> Result<PressureBaseline, String> {
    if metrics.relief_seam_wood_cells != INITIAL_TOTAL_SEAM_WOOD
        || metrics.top_relief_seam_wood_cells != INITIAL_TOP_SEAM_WOOD
        || metrics.bottom_relief_seam_wood_cells != INITIAL_BOTTOM_SEAM_WOOD
        || metrics.relief_seam_open_cells != 0
    {
        return Err(format!(
            "Pressure tick-0 relief seam mismatch: total/top/bottom/open={}/{}/{}/{} expected 576/384/192/0",
            metrics.relief_seam_wood_cells,
            metrics.top_relief_seam_wood_cells,
            metrics.bottom_relief_seam_wood_cells,
            metrics.relief_seam_open_cells
        ));
    }
    if metrics.chamber_pressure_cell_count != CHAMBER_PRESSURE_CELL_COUNT {
        return Err(format!(
            "Pressure tick-0 cavity count={} expected {CHAMBER_PRESSURE_CELL_COUNT}",
            metrics.chamber_pressure_cell_count
        ));
    }
    Ok(PressureBaseline {
        matter_count: metrics.matter_count,
        water_count: metrics.water_count,
        steam_count: metrics.steam_count,
        relief_seam_wood_cells: metrics.relief_seam_wood_cells,
        top_relief_seam_wood_cells: metrics.top_relief_seam_wood_cells,
        bottom_relief_seam_wood_cells: metrics.bottom_relief_seam_wood_cells,
        chamber_pressure_cell_count: metrics.chamber_pressure_cell_count,
        chamber_mean_pressure: metrics.chamber_mean_pressure,
        chamber_max_pressure: metrics.chamber_max_pressure,
    })
}

struct PressureJsonlWriters {
    samples: BufWriter<File>,
    events: BufWriter<File>,
    event_sequence: u64,
}

impl PressureJsonlWriters {
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
        metrics: &PressureSampleMetrics,
    ) -> Result<(), String> {
        let counts = metrics
            .material_counts_by_id
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        writeln!(
            self.samples,
            concat!(
                "{{\"schema_version\":\"{}\",\"experiment_id\":\"{}\",",
                "\"run_id\":\"{}\",\"scenario\":\"pressure-burst\",",
                "\"source_sha\":\"{}\",\"git_state\":\"{}\",",
                "\"build_profile\":\"{}\",\"binary_sha256\":\"{}\",",
                "\"sample_sequence\":{},\"sim_tick\":{},",
                "\"phase\":\"{}\",\"reason\":\"{}\",",
                "\"world\":{{\"width\":{},\"height\":{},\"chunk_size\":{}}},",
                "\"sleep\":{{\"enabled\":{},\"threshold\":{}}},",
                "\"census\":{{\"total_cells\":{},\"any_active_cells\":{},",
                "\"matter_active_cells\":{},\"thermal_active_cells\":{},",
                "\"pressure_active_cells\":{},\"reaction_active_cells\":{},",
                "\"total_chunks\":{},\"active_chunks\":{},",
                "\"runnable_chunks\":{},\"sleeping_chunks\":{}}},",
                "\"material_counts_by_id\":[{}],\"matter_count\":{},",
                "\"water_count\":{},\"steam_count\":{},",
                "\"relief_seam_wood_cells\":{},",
                "\"top_relief_seam_wood_cells\":{},",
                "\"bottom_relief_seam_wood_cells\":{},",
                "\"relief_seam_open_cells\":{},",
                "\"top_relief_seam_open_cells\":{},",
                "\"bottom_relief_seam_open_cells\":{},",
                "\"steam_in_relief_seam_cells\":{},",
                "\"outside_chamber_steam_cells\":{},",
                "\"chamber_pressure_cell_count\":{},",
                "\"chamber_mean_pressure\":{:.9},",
                "\"chamber_max_pressure\":{:.9},",
                "\"invalid_material_count\":{},",
                "\"nonfinite_temperature_count\":{},",
                "\"nonfinite_pressure_count\":{},\"changed_chunks\":{},",
                "\"wake_chunks\":{},\"wake_reason_or\":{},",
                "\"state_hash\":\"{}\",\"physical_state_hash\":\"{}\"}}"
            ),
            PRESSURE_TELEMETRY_SCHEMA_VERSION,
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
            counts,
            metrics.matter_count,
            metrics.water_count,
            metrics.steam_count,
            metrics.relief_seam_wood_cells,
            metrics.top_relief_seam_wood_cells,
            metrics.bottom_relief_seam_wood_cells,
            metrics.relief_seam_open_cells,
            metrics.top_relief_seam_open_cells,
            metrics.bottom_relief_seam_open_cells,
            metrics.steam_in_relief_seam_cells,
            metrics.outside_chamber_steam_cells,
            metrics.chamber_pressure_cell_count,
            metrics.chamber_mean_pressure,
            metrics.chamber_max_pressure,
            metrics.invalid_material_count,
            metrics.nonfinite_temperature_count,
            metrics.nonfinite_pressure_count,
            metrics.changed_chunks,
            metrics.wake_chunks,
            metrics.wake_reason_or,
            metrics.state_hash,
            metrics.physical_state_hash,
        )
        .map_err(|error| format!("write Pressure samples JSONL failed: {error}"))
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
                "\"run_id\":\"{}\",\"scenario\":\"pressure-burst\",",
                "\"event_sequence\":{},\"event\":\"{}\",",
                "\"sim_tick\":{},\"sample_sequence\":{},\"detail\":\"{}\"}}"
            ),
            PRESSURE_TELEMETRY_SCHEMA_VERSION,
            json_escape(&config.experiment_id),
            json_escape(&config.run_id),
            self.event_sequence,
            json_escape(event),
            sim_tick,
            json_opt_u64(sample_sequence),
            json_escape(detail),
        )
        .map_err(|error| format!("write Pressure events JSONL failed: {error}"))?;
        self.event_sequence = self.event_sequence.saturating_add(1);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        self.samples
            .flush()
            .map_err(|error| format!("flush Pressure samples JSONL failed: {error}"))?;
        self.events
            .flush()
            .map_err(|error| format!("flush Pressure events JSONL failed: {error}"))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OpeningUpdate {
    first_in_streak: bool,
    confirmed: bool,
    streak_broken: bool,
}

#[derive(Clone, Debug)]
struct PersistentOpeningDetector {
    required: u32,
    streak: u32,
    first: Option<SampleIdentity>,
}

impl PersistentOpeningDetector {
    fn new(required: u32) -> Self {
        Self {
            required,
            streak: 0,
            first: None,
        }
    }

    fn observe(&mut self, metrics: &PressureSampleMetrics) -> OpeningUpdate {
        if metrics.relief_seam_open_cells == 0 {
            let streak_broken = self.streak != 0;
            self.streak = 0;
            self.first = None;
            return OpeningUpdate {
                streak_broken,
                ..OpeningUpdate::default()
            };
        }
        let first_in_streak = self.streak == 0;
        if first_in_streak {
            self.first = Some(metrics.identity());
        }
        self.streak = self.streak.saturating_add(1);
        OpeningUpdate {
            first_in_streak,
            confirmed: self.streak >= self.required,
            streak_broken: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ObservationUpdate {
    first_pressure_activity: bool,
    first_wood_damage: bool,
    first_rupture: bool,
    first_steam_in_relief_seam: bool,
    first_exterior_steam: bool,
    first_post_confirmation_reseal: bool,
    new_peak_chamber_mean: bool,
    new_peak_chamber_max: bool,
    new_peak_pressure_activity: bool,
    first_post_opening_relief: bool,
}

impl ObservationUpdate {
    fn needs_frame(self) -> bool {
        self.first_pressure_activity
            || self.first_wood_damage
            || self.first_rupture
            || self.first_exterior_steam
            || self.first_post_confirmation_reseal
            || self.new_peak_chamber_mean
            || self.new_peak_chamber_max
            || self.new_peak_pressure_activity
            || self.first_post_opening_relief
    }
}

#[derive(Clone, Debug)]
struct PressureObservations {
    first_pressure_activity: Option<SampleIdentity>,
    first_wood_damage: Option<SampleIdentity>,
    /// Cold bottom seam loss is the unambiguous pressure-only rupture evidence.
    first_rupture: Option<SampleIdentity>,
    persistent_opening_start: Option<SampleIdentity>,
    persistent_opening_confirmed: Option<SampleIdentity>,
    first_steam_in_relief_seam: Option<SampleIdentity>,
    first_exterior_steam: Option<SampleIdentity>,
    first_post_confirmation_reseal: Option<SampleIdentity>,
    first_post_opening_relief: Option<SampleIdentity>,
    vent_reference_chamber_mean_pressure: Option<f64>,
    vent_reference_chamber_max_pressure: Option<f64>,
    top_relief_seam_ever_opened: bool,
    bottom_relief_seam_ever_opened: bool,
    peak_chamber_mean_pressure: f64,
    peak_chamber_mean: SampleIdentity,
    peak_chamber_max_pressure: f64,
    peak_chamber_max: SampleIdentity,
    peak_pressure_active_cells: u64,
    peak_pressure_activity: SampleIdentity,
    pre_opening_peak_chamber_mean_pressure: f64,
    pre_opening_peak_chamber_max_pressure: f64,
    post_opening_chamber_mean_pressure: Option<f64>,
    post_opening_chamber_max_pressure: Option<f64>,
    outside_chamber_steam_peak: u64,
    invalid_material_occurrences: u64,
    nonfinite_field_occurrences: u64,
    latest: PressureSampleMetrics,
}

impl PressureObservations {
    fn new(tick0: &PressureSampleMetrics) -> Self {
        Self {
            first_pressure_activity: None,
            first_wood_damage: None,
            first_rupture: None,
            persistent_opening_start: None,
            persistent_opening_confirmed: None,
            first_steam_in_relief_seam: None,
            first_exterior_steam: None,
            first_post_confirmation_reseal: None,
            first_post_opening_relief: None,
            vent_reference_chamber_mean_pressure: None,
            vent_reference_chamber_max_pressure: None,
            top_relief_seam_ever_opened: false,
            bottom_relief_seam_ever_opened: false,
            peak_chamber_mean_pressure: tick0.chamber_mean_pressure,
            peak_chamber_mean: tick0.identity(),
            peak_chamber_max_pressure: tick0.chamber_max_pressure,
            peak_chamber_max: tick0.identity(),
            peak_pressure_active_cells: tick0.pressure_active_cells,
            peak_pressure_activity: tick0.identity(),
            pre_opening_peak_chamber_mean_pressure: tick0.chamber_mean_pressure,
            pre_opening_peak_chamber_max_pressure: tick0.chamber_max_pressure,
            post_opening_chamber_mean_pressure: None,
            post_opening_chamber_max_pressure: None,
            outside_chamber_steam_peak: tick0.outside_chamber_steam_cells,
            invalid_material_occurrences: tick0.invalid_material_count,
            nonfinite_field_occurrences: tick0
                .nonfinite_temperature_count
                .saturating_add(tick0.nonfinite_pressure_count),
            latest: tick0.clone(),
        }
    }

    fn observe(
        &mut self,
        metrics: &PressureSampleMetrics,
        baseline: &PressureBaseline,
    ) -> ObservationUpdate {
        self.invalid_material_occurrences = self
            .invalid_material_occurrences
            .saturating_add(metrics.invalid_material_count);
        self.nonfinite_field_occurrences = self
            .nonfinite_field_occurrences
            .saturating_add(metrics.nonfinite_temperature_count)
            .saturating_add(metrics.nonfinite_pressure_count);

        let first_pressure_activity =
            metrics.pressure_active_cells != 0 && self.first_pressure_activity.is_none();
        if first_pressure_activity {
            self.first_pressure_activity = Some(metrics.identity());
        }
        let opening_already_seen = self.persistent_opening_confirmed.is_some();
        if !opening_already_seen {
            self.pre_opening_peak_chamber_mean_pressure = self
                .pre_opening_peak_chamber_mean_pressure
                .max(metrics.chamber_mean_pressure);
            self.pre_opening_peak_chamber_max_pressure = self
                .pre_opening_peak_chamber_max_pressure
                .max(metrics.chamber_max_pressure);
        }
        let first_wood_damage = metrics.relief_seam_wood_cells < baseline.relief_seam_wood_cells
            && self.first_wood_damage.is_none();
        if first_wood_damage {
            self.first_wood_damage = Some(metrics.identity());
        }
        let first_rupture =
            metrics.bottom_relief_seam_open_cells != 0 && self.first_rupture.is_none();
        if first_rupture {
            self.first_rupture = Some(metrics.identity());
        }
        self.top_relief_seam_ever_opened |= metrics.top_relief_seam_open_cells != 0;
        self.bottom_relief_seam_ever_opened |= metrics.bottom_relief_seam_open_cells != 0;

        let confirmed = self.persistent_opening_confirmed.is_some();
        let first_post_confirmation_reseal = confirmed
            && metrics.relief_seam_open_cells == 0
            && self.first_post_confirmation_reseal.is_none();
        if first_post_confirmation_reseal {
            self.first_post_confirmation_reseal = Some(metrics.identity());
        }
        let first_steam_in_relief_seam = confirmed
            && metrics.steam_in_relief_seam_cells != 0
            && self.first_steam_in_relief_seam.is_none();
        if first_steam_in_relief_seam {
            self.first_steam_in_relief_seam = Some(metrics.identity());
        }
        let first_exterior_steam = confirmed
            && self.first_steam_in_relief_seam.is_some()
            && metrics.outside_chamber_steam_cells != 0
            && self.first_exterior_steam.is_none();
        if first_exterior_steam {
            self.first_exterior_steam = Some(metrics.identity());
            self.vent_reference_chamber_mean_pressure = Some(metrics.chamber_mean_pressure);
            self.vent_reference_chamber_max_pressure = Some(metrics.chamber_max_pressure);
        }
        let new_peak_chamber_mean = metrics.chamber_mean_pressure > self.peak_chamber_mean_pressure;
        if new_peak_chamber_mean {
            self.peak_chamber_mean_pressure = metrics.chamber_mean_pressure;
            self.peak_chamber_mean = metrics.identity();
        }
        let new_peak_chamber_max = metrics.chamber_max_pressure > self.peak_chamber_max_pressure;
        if new_peak_chamber_max {
            self.peak_chamber_max_pressure = metrics.chamber_max_pressure;
            self.peak_chamber_max = metrics.identity();
        }
        let new_peak_pressure_activity =
            metrics.pressure_active_cells > self.peak_pressure_active_cells;
        if new_peak_pressure_activity {
            self.peak_pressure_active_cells = metrics.pressure_active_cells;
            self.peak_pressure_activity = metrics.identity();
        }
        self.outside_chamber_steam_peak = self
            .outside_chamber_steam_peak
            .max(metrics.outside_chamber_steam_cells);

        if confirmed && self.post_opening_chamber_mean_pressure.is_none() {
            self.post_opening_chamber_mean_pressure = Some(metrics.chamber_mean_pressure);
            self.post_opening_chamber_max_pressure = Some(metrics.chamber_max_pressure);
        }
        let first_post_opening_relief = matches!(
            (
                self.first_exterior_steam,
                self.vent_reference_chamber_mean_pressure,
                self.vent_reference_chamber_max_pressure,
            ),
            (Some(vent), Some(vent_mean), Some(vent_max))
                if metrics.identity() != vent
                    && metrics.chamber_mean_pressure < vent_mean
                    && metrics.chamber_max_pressure < vent_max
                    && self.first_post_opening_relief.is_none()
        );
        if first_post_opening_relief {
            self.first_post_opening_relief = Some(metrics.identity());
        }
        self.latest = metrics.clone();

        ObservationUpdate {
            first_pressure_activity,
            first_wood_damage,
            first_rupture,
            first_steam_in_relief_seam,
            first_exterior_steam,
            first_post_confirmation_reseal,
            new_peak_chamber_mean,
            new_peak_chamber_max,
            new_peak_pressure_activity,
            first_post_opening_relief,
        }
    }

    fn confirm_opening(&mut self, first: SampleIdentity, confirmed: SampleIdentity) {
        if self.persistent_opening_confirmed.is_none() {
            self.persistent_opening_start = Some(first);
            self.persistent_opening_confirmed = Some(confirmed);
        }
    }
}

#[derive(Clone, Debug)]
struct TerminalTrend {
    sample_count: usize,
    start_sim_tick: Option<u64>,
    end_sim_tick: Option<u64>,
    start_mean_pressure: Option<f64>,
    end_mean_pressure: Option<f64>,
    start_max_pressure: Option<f64>,
    end_max_pressure: Option<f64>,
    minimum_mean_pressure: Option<f64>,
    maximum_mean_pressure: Option<f64>,
    slope_per_sample: Option<f64>,
    /// Kept for schema compatibility: positive chamber-mean transitions.
    positive_step_count: usize,
    positive_max_step_count: usize,
    mean_unbounded_growth: bool,
    max_unbounded_growth: bool,
    unbounded_growth: bool,
}

fn terminal_trend(samples: &VecDeque<PressureSampleMetrics>) -> TerminalTrend {
    let sample_count = samples.len();
    let start = samples.front();
    let end = samples.back();
    let minimum_mean_pressure = samples
        .iter()
        .map(|sample| sample.chamber_mean_pressure)
        .reduce(f64::min);
    let maximum_mean_pressure = samples
        .iter()
        .map(|sample| sample.chamber_mean_pressure)
        .reduce(f64::max);
    let positive_step_count = samples
        .iter()
        .zip(samples.iter().skip(1))
        .filter(|(left, right)| right.chamber_mean_pressure > left.chamber_mean_pressure)
        .count();
    let positive_max_step_count = samples
        .iter()
        .zip(samples.iter().skip(1))
        .filter(|(left, right)| right.chamber_max_pressure > left.chamber_max_pressure)
        .count();
    let slope_per_sample = if sample_count < 2 {
        None
    } else {
        let n = sample_count as f64;
        let mean_x = (n - 1.0) / 2.0;
        let mean_y = samples
            .iter()
            .map(|sample| sample.chamber_mean_pressure)
            .sum::<f64>()
            / n;
        let (numerator, denominator) =
            samples
                .iter()
                .enumerate()
                .fold((0.0f64, 0.0f64), |(num, den), (index, sample)| {
                    let dx = index as f64 - mean_x;
                    (
                        num + dx * (sample.chamber_mean_pressure - mean_y),
                        den + dx * dx,
                    )
                });
        Some(if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        })
    };
    let mean_unbounded_growth = match (start, end) {
        (Some(start), Some(end)) if sample_count >= 2 => {
            end.chamber_mean_pressure > start.chamber_mean_pressure * 1.10 + 1.0
                && positive_step_count.saturating_mul(4)
                    >= sample_count.saturating_sub(1).saturating_mul(3)
        }
        _ => false,
    };
    let max_unbounded_growth = match (start, end) {
        (Some(start), Some(end)) if sample_count >= 2 => {
            end.chamber_max_pressure > start.chamber_max_pressure * 1.10 + 1.0
                && positive_max_step_count.saturating_mul(4)
                    >= sample_count.saturating_sub(1).saturating_mul(3)
        }
        _ => false,
    };
    let unbounded_growth = mean_unbounded_growth || max_unbounded_growth;
    TerminalTrend {
        sample_count,
        start_sim_tick: start.map(|sample| sample.sim_tick),
        end_sim_tick: end.map(|sample| sample.sim_tick),
        start_mean_pressure: start.map(|sample| sample.chamber_mean_pressure),
        end_mean_pressure: end.map(|sample| sample.chamber_mean_pressure),
        start_max_pressure: start.map(|sample| sample.chamber_max_pressure),
        end_max_pressure: end.map(|sample| sample.chamber_max_pressure),
        minimum_mean_pressure,
        maximum_mean_pressure,
        slope_per_sample,
        positive_step_count,
        positive_max_step_count,
        mean_unbounded_growth,
        max_unbounded_growth,
        unbounded_growth,
    }
}

fn remember_terminal_sample(
    samples: &mut VecDeque<PressureSampleMetrics>,
    sample: &PressureSampleMetrics,
    capacity: usize,
) {
    if samples.len() == capacity {
        let _ = samples.pop_front();
    }
    samples.push_back(sample.clone());
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FrameBadge {
    kind: &'static str,
    reason: &'static str,
}

#[derive(Clone, Debug)]
struct PressureFrame {
    badges: Vec<FrameBadge>,
    sim_tick: u64,
    sample_sequence: u64,
    state_hash: String,
    frame: RawFrame,
}

impl PressureFrame {
    fn with_badge(&self, kind: &'static str, reason: &'static str) -> Self {
        Self {
            badges: vec![FrameBadge { kind, reason }],
            sim_tick: self.sim_tick,
            sample_sequence: self.sample_sequence,
            state_hash: self.state_hash.clone(),
            frame: self.frame.clone(),
        }
    }

    fn is_reset(&self) -> bool {
        self.badges.iter().any(|badge| badge.kind == "reset")
    }
}

#[derive(Clone, Debug)]
struct WrittenPressureFrame {
    ordinal: usize,
    relative_path: String,
    width: u32,
    height: u32,
    rgba_bytes: usize,
    badges: Vec<FrameBadge>,
    sim_tick: u64,
    sample_sequence: u64,
    state_hash: String,
}

fn badge_rank(kind: &str) -> usize {
    match kind {
        "tick0" => 0,
        "tick1" => 1,
        "first-pressure-activity" => 2,
        "first-wood-damage" => 3,
        "first-rupture" => 4,
        "persistent-opening" => 5,
        "opening-reseal" => 6,
        "first-exterior-steam" => 7,
        "peak-pressure" => 8,
        "peak-pressure-activity" => 9,
        "post-opening" => 10,
        "terminal" => 11,
        "diagnostic-observation" => 12,
        "reset" => 13,
        _ => usize::MAX,
    }
}

fn capture_pressure_frame(
    renderer: &mut Renderer,
    metrics: &PressureSampleMetrics,
    kind: &'static str,
    reason: &'static str,
) -> Result<PressureFrame, String> {
    let captured = renderer
        .capture_full_frame(None)
        .map_err(|error| format!("capture Pressure {kind} frame failed: {error}"))?;
    Ok(PressureFrame {
        badges: vec![FrameBadge { kind, reason }],
        sim_tick: metrics.sim_tick,
        sample_sequence: metrics.sample_sequence,
        state_hash: metrics.state_hash.clone(),
        frame: RawFrame::try_from(captured)?,
    })
}

fn fold_and_order_frames(mut frames: Vec<PressureFrame>) -> Vec<PressureFrame> {
    frames.sort_by_key(|frame| {
        (
            frame.is_reset(),
            frame.sim_tick,
            frame.sample_sequence,
            frame
                .badges
                .iter()
                .map(|badge| badge_rank(badge.kind))
                .min()
                .unwrap_or(usize::MAX),
        )
    });
    let mut folded: Vec<PressureFrame> = Vec::with_capacity(frames.len());
    for mut frame in frames {
        frame.badges.sort_by_key(|badge| badge_rank(badge.kind));
        frame.badges.dedup_by_key(|badge| badge.kind);
        let can_fold = !frame.is_reset();
        if can_fold {
            if let Some(existing) = folded.iter_mut().find(|existing| {
                !existing.is_reset()
                    && existing.sim_tick == frame.sim_tick
                    && existing.state_hash == frame.state_hash
            }) {
                existing.badges.extend(frame.badges);
                existing.badges.sort_by_key(|badge| badge_rank(badge.kind));
                existing.badges.dedup_by_key(|badge| badge.kind);
                existing.sample_sequence = existing.sample_sequence.min(frame.sample_sequence);
                continue;
            }
        }
        folded.push(frame);
    }
    folded.sort_by_key(|frame| {
        (
            frame.is_reset(),
            frame.sim_tick,
            frame.sample_sequence,
            frame
                .badges
                .iter()
                .map(|badge| badge_rank(badge.kind))
                .min()
                .unwrap_or(usize::MAX),
        )
    });
    folded
}

fn assemble_frames(
    milestone_frames: [Option<PressureFrame>; 12],
    diagnostics: &VecDeque<PressureFrame>,
) -> Result<Vec<PressureFrame>, String> {
    let mut frames = milestone_frames.into_iter().flatten().collect::<Vec<_>>();
    frames = fold_and_order_frames(frames);
    for diagnostic in diagnostics {
        if frames.len() >= MIN_RAW_FRAMES {
            break;
        }
        frames.push(diagnostic.clone());
        frames = fold_and_order_frames(frames);
    }
    if !(MIN_RAW_FRAMES..=MAX_RAW_FRAMES).contains(&frames.len()) {
        return Err(format!(
            "completed Pressure lifecycle produced {} physical frames; required {MIN_RAW_FRAMES}..={MAX_RAW_FRAMES}",
            frames.len()
        ));
    }
    Ok(frames)
}

fn write_pressure_raw_frames(
    raw_frames_dir: &Path,
    frames: Vec<PressureFrame>,
) -> Result<Vec<WrittenPressureFrame>, String> {
    let mut written = Vec::with_capacity(frames.len());
    for (ordinal, frame) in frames.into_iter().enumerate() {
        let primary = frame
            .badges
            .first()
            .map_or("observation", |badge| badge.kind);
        let filename = format!("{ordinal:02}-{primary}.rgba");
        let path = raw_frames_dir.join(&filename);
        write_new(&path, &frame.frame.rgba)?;
        written.push(WrittenPressureFrame {
            ordinal,
            relative_path: format!("work/frames/{filename}"),
            width: frame.frame.width,
            height: frame.frame.height,
            rgba_bytes: frame.frame.rgba.len(),
            badges: frame.badges,
            sim_tick: frame.sim_tick,
            sample_sequence: frame.sample_sequence,
            state_hash: frame.state_hash,
        });
    }
    Ok(written)
}

fn write_frames_json(
    config: &super::ExperimentWorkerConfig,
    path: &Path,
    frames: &[WrittenPressureFrame],
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
                    "{{\"ordinal\":{},\"relative_path\":\"{}\",",
                    "\"width\":{},\"height\":{},\"rgba_bytes\":{},",
                    "\"badges\":[{}],\"sim_tick\":{},\"sample_sequence\":{},",
                    "\"state_hash\":\"{}\"}}"
                ),
                frame.ordinal,
                json_escape(&frame.relative_path),
                frame.width,
                frame.height,
                frame.rgba_bytes,
                badges,
                frame.sim_tick,
                frame.sample_sequence,
                json_escape(&frame.state_hash),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let json = format!(
        concat!(
            "{{\n  \"schema_version\": \"{}\",\n  \"experiment_id\": \"{}\",",
            "\n  \"run_id\": \"{}\",\n  \"scenario\": \"pressure-burst\",",
            "\n  \"binary_sha256\": \"{}\",\n  \"frame_count\": {},",
            "\n  \"pixel_encoding\": \"rgba8-tightly-packed\",",
            "\n  \"frames\": [{}]\n}}\n"
        ),
        PRESSURE_FRAMES_SCHEMA_VERSION,
        json_escape(&config.experiment_id),
        json_escape(&config.run_id),
        json_escape(&config.binary_sha256.to_ascii_lowercase()),
        frames.len(),
        entries,
    );
    write_new(path, json.as_bytes())
}

fn validate_pressure_worker_config(
    simulation: &Simulation,
    config: &super::ExperimentWorkerConfig,
) -> Result<(), String> {
    if config.experiment_id != PRESSURE_EXPERIMENT_ID {
        return Err(format!(
            "Pressure experiment_id must be '{PRESSURE_EXPERIMENT_ID}', got '{}'",
            config.experiment_id
        ));
    }
    if !is_safe_identifier(&config.run_id) {
        return Err("run_id must contain only ASCII letters, digits, '.', '_' or '-'".to_string());
    }
    if !config.run_dir.is_dir() {
        return Err(format!(
            "run_dir must already exist as a unique directory: {}",
            display_path(&config.run_dir)
        ));
    }
    if config.scenario != ScenarioId::PressureBurst {
        return Err(format!(
            "Pressure experiment v0 supports only PressureBurst, got {}",
            config.scenario
        ));
    }
    if simulation.world.config != REQUIRED_WORLD {
        return Err(format!(
            "Pressure experiment v0 requires WorldConfig 256x256x64, got {}x{}x{}",
            simulation.world.config.width,
            simulation.world.config.height,
            simulation.world.config.chunk_size
        ));
    }
    if !simulation.sleep_enabled {
        return Err("Pressure experiment v0 requires simulation sleep to be enabled".to_string());
    }
    if config.max_ticks != REQUIRED_MAX_TICKS {
        return Err(format!("Pressure max_ticks must be {REQUIRED_MAX_TICKS}"));
    }
    if config.diagnostic_interval_ticks != REQUIRED_DIAGNOSTIC_INTERVAL_TICKS {
        return Err(format!(
            "Pressure diagnostic_interval_ticks must be {REQUIRED_DIAGNOSTIC_INTERVAL_TICKS}"
        ));
    }
    if config.consecutive_persistent_opening != REQUIRED_PERSISTENT_OPENING_SAMPLES {
        return Err(format!(
            "consecutive_persistent_opening must be {REQUIRED_PERSISTENT_OPENING_SAMPLES}"
        ));
    }
    if config.post_opening_ticks != REQUIRED_POST_OPENING_TICKS {
        return Err(format!(
            "post_opening_ticks must be {REQUIRED_POST_OPENING_TICKS}"
        ));
    }
    if config.terminal_window_samples != REQUIRED_TERMINAL_WINDOW_SAMPLES {
        return Err(format!(
            "terminal_window_samples must be {REQUIRED_TERMINAL_WINDOW_SAMPLES}"
        ));
    }
    if config.consecutive_all_sleep != 0
        || config.post_sleep_ticks != 0
        || config.consecutive_reaction_zero != 0
        || config.post_reaction_ticks != 0
    {
        return Err("Pressure worker rejects Sand/Water/Fire lifecycle settings".to_string());
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TerminalReason {
    PostOpeningObservationComplete,
    MaxTicks,
}

impl TerminalReason {
    const fn as_str(self) -> &'static str {
        match self {
            Self::PostOpeningObservationComplete => "post-opening-observation-complete",
            Self::MaxTicks => "max-ticks",
        }
    }
}

#[derive(Clone, Debug)]
struct PressurePredicates {
    pressure_activity_observed: PredicateResult,
    relief_seam_damaged: PredicateResult,
    persistent_opening_created: PredicateResult,
    exterior_vent_observed: PredicateResult,
    post_opening_pressure_relieved: PredicateResult,
    terminal_pressure_not_runaway: PredicateResult,
    no_invalid_materials: PredicateResult,
    no_nonfinite_fields: PredicateResult,
    exact_reset: PredicateResult,
}

impl PressurePredicates {
    fn statuses(&self) -> [PredicateStatus; 9] {
        [
            self.pressure_activity_observed.status,
            self.relief_seam_damaged.status,
            self.persistent_opening_created.status,
            self.exterior_vent_observed.status,
            self.post_opening_pressure_relieved.status,
            self.terminal_pressure_not_runaway.status,
            self.no_invalid_materials.status,
            self.no_nonfinite_fields.status,
            self.exact_reset.status,
        ]
    }
}

#[derive(Clone, Debug)]
struct ReviewFlags {
    only_one_relief_seam_ruptured: bool,
    high_terminal_pressure_activity: bool,
    long_pressure_tail: bool,
    persistent_vent_plume: bool,
    terminal_activity_remains: bool,
    reasons: Vec<&'static str>,
}

fn review_flags(
    baseline: &PressureBaseline,
    observations: &PressureObservations,
    trend: &TerminalTrend,
) -> ReviewFlags {
    let latest = &observations.latest;
    let only_one_relief_seam_ruptured =
        observations.top_relief_seam_ever_opened ^ observations.bottom_relief_seam_ever_opened;
    let high_terminal_pressure_activity = latest.pressure_active_cells >= 256
        && latest.pressure_active_cells.saturating_mul(4) > observations.peak_pressure_active_cells;
    let long_pressure_tail = trend
        .end_mean_pressure
        .is_some_and(|value| value * 2.0 > baseline.chamber_mean_pressure);
    let persistent_vent_plume = latest.outside_chamber_steam_cells != 0;
    let terminal_activity_remains = latest.any_active_cells > 0 || latest.active_chunks > 0;
    let mut reasons = Vec::with_capacity(5);
    if only_one_relief_seam_ruptured {
        reasons.push("only_one_relief_seam_ruptured");
    }
    if high_terminal_pressure_activity {
        reasons.push("high_terminal_pressure_activity");
    }
    if long_pressure_tail {
        reasons.push("long_pressure_tail");
    }
    if persistent_vent_plume {
        reasons.push("persistent_vent_plume");
    }
    if terminal_activity_remains {
        reasons.push("terminal_activity_remains");
    }
    ReviewFlags {
        only_one_relief_seam_ruptured,
        high_terminal_pressure_activity,
        long_pressure_tail,
        persistent_vent_plume,
        terminal_activity_remains,
        reasons,
    }
}

fn build_predicates(
    observations: &PressureObservations,
    trend: &TerminalTrend,
    required_terminal_samples: usize,
    exact_reset: bool,
) -> PressurePredicates {
    let pressure_activity_observed = observations.first_pressure_activity.map_or_else(
        || PredicateResult::fail("no sampled production state had Pressure activity"),
        |identity| {
            PredicateResult::pass(format!(
                "Pressure activity first observed at tick {} sample {}",
                identity.sim_tick, identity.sample_sequence
            ))
        },
    );
    let relief_seam_damaged = observations.first_wood_damage.map_or_else(
        || {
            PredicateResult::fail(
                "both authored relief seams retained all Wood through the terminal sample",
            )
        },
        |identity| {
            PredicateResult::pass(format!(
                "an authored relief seam first lost Wood at tick {} sample {}",
                identity.sim_tick, identity.sample_sequence
            ))
        },
    );
    let persistent_opening_created = match (
        observations.persistent_opening_confirmed,
        observations.first_post_confirmation_reseal,
    ) {
        (None, _) => PredicateResult::fail("no opening persisted for three diagnostic samples"),
        (Some(confirmed), Some(reseal)) => PredicateResult::fail(format!(
            "opening confirmed at tick {} sample {} resealed at tick {} sample {}",
            confirmed.sim_tick,
            confirmed.sample_sequence,
            reseal.sim_tick,
            reseal.sample_sequence
        )),
        (Some(identity), None) => PredicateResult::pass(format!(
            "persistent opening confirmed at tick {} sample {} and remained open through the observation window",
            identity.sim_tick, identity.sample_sequence
        )),
    };
    let exterior_vent_observed = observations.first_exterior_steam.map_or_else(
        || {
            PredicateResult::fail(
                "no causal relief-seam Steam transit to exterior Steam was observed after persistent opening confirmation",
            )
        },
        |identity| {
            PredicateResult::pass(format!(
                "exterior Steam first observed at tick {} sample {}",
                identity.sim_tick, identity.sample_sequence
            ))
        },
    );
    let post_opening_pressure_relieved = match (
        observations.persistent_opening_confirmed,
        observations.first_exterior_steam,
        observations.first_post_opening_relief,
        observations.vent_reference_chamber_mean_pressure,
        observations.vent_reference_chamber_max_pressure,
        trend.end_mean_pressure,
        trend.end_max_pressure,
    ) {
        (
            Some(_),
            Some(_),
            Some(relief),
            Some(vent_mean),
            Some(vent_max),
            Some(end_mean),
            Some(end_max),
        )
            if end_mean < observations.pre_opening_peak_chamber_mean_pressure
                && end_max < observations.pre_opening_peak_chamber_max_pressure
                && end_mean < vent_mean
                && end_max < vent_max =>
        {
            PredicateResult::pass(format!(
                "post-vent relief first observed at tick {} sample {}; terminal chamber mean/max {:.9}/{:.9} are below vent reference {:.9}/{:.9} and pre-opening peak {:.9}/{:.9}",
                relief.sim_tick,
                relief.sample_sequence,
                end_mean,
                end_max,
                vent_mean,
                vent_max,
                observations.pre_opening_peak_chamber_mean_pressure,
                observations.pre_opening_peak_chamber_max_pressure
            ))
        }
        (
            Some(_),
            Some(_),
            _,
            Some(vent_mean),
            Some(vent_max),
            Some(end_mean),
            Some(end_max),
        ) => PredicateResult::unknown(format!(
            "opening and causal vent exist, but no sustained post-vent mean/max relief was established: terminal {:.9}/{:.9}, vent reference {:.9}/{:.9}, pre-opening peak {:.9}/{:.9}",
            end_mean,
            end_max,
            vent_mean,
            vent_max,
            observations.pre_opening_peak_chamber_mean_pressure,
            observations.pre_opening_peak_chamber_max_pressure
        )),
        _ => PredicateResult::unknown(
            "post-opening Pressure relief cannot be evaluated without opening, vent, and terminal samples",
        ),
    };
    let terminal_pressure_not_runaway = if trend.sample_count < required_terminal_samples {
        PredicateResult::unknown(format!(
            "terminal Pressure window has {} samples; required {required_terminal_samples}",
            trend.sample_count
        ))
    } else if trend.unbounded_growth {
        PredicateResult::fail(format!(
            "terminal Pressure met runaway rule: mean={:.9}->{:.9} positive_mean_steps={}/{}, max={:.9}->{:.9} positive_max_steps={}/{}",
            trend.start_mean_pressure.unwrap_or(0.0),
            trend.end_mean_pressure.unwrap_or(0.0),
            trend.positive_step_count,
            trend.sample_count.saturating_sub(1),
            trend.start_max_pressure.unwrap_or(0.0),
            trend.end_max_pressure.unwrap_or(0.0),
            trend.positive_max_step_count,
            trend.sample_count.saturating_sub(1)
        ))
    } else {
        PredicateResult::pass(format!(
            "terminal Pressure did not meet runaway rule: mean={:.9}->{:.9} positive_mean_steps={}/{}, max={:.9}->{:.9} positive_max_steps={}/{}",
            trend.start_mean_pressure.unwrap_or(0.0),
            trend.end_mean_pressure.unwrap_or(0.0),
            trend.positive_step_count,
            trend.sample_count.saturating_sub(1),
            trend.start_max_pressure.unwrap_or(0.0),
            trend.end_max_pressure.unwrap_or(0.0),
            trend.positive_max_step_count,
            trend.sample_count.saturating_sub(1)
        ))
    };
    let no_invalid_materials = if observations.invalid_material_occurrences == 0 {
        PredicateResult::pass("invalid material count was zero in every sample")
    } else {
        PredicateResult::fail(format!(
            "sampled invalid material occurrences={}",
            observations.invalid_material_occurrences
        ))
    };
    let no_nonfinite_fields = if observations.nonfinite_field_occurrences == 0 {
        PredicateResult::pass("non-finite temperature/pressure count was zero in every sample")
    } else {
        PredicateResult::fail(format!(
            "sampled non-finite field occurrences={}",
            observations.nonfinite_field_occurrences
        ))
    };
    let exact_reset = if exact_reset {
        PredicateResult::pass("programmatic R-equivalent state exactly matched tick 0")
    } else {
        PredicateResult::fail("programmatic R-equivalent state differed from tick 0")
    };
    PressurePredicates {
        pressure_activity_observed,
        relief_seam_damaged,
        persistent_opening_created,
        exterior_vent_observed,
        post_opening_pressure_relieved,
        terminal_pressure_not_runaway,
        no_invalid_materials,
        no_nonfinite_fields,
        exact_reset,
    }
}

fn pressure_verdict(
    predicates: &PressurePredicates,
    review_flags: &ReviewFlags,
) -> ExperimentVerdict {
    let statuses = predicates.statuses();
    if statuses.contains(&PredicateStatus::Fail) {
        ExperimentVerdict::Fail
    } else if statuses.contains(&PredicateStatus::Unknown) || !review_flags.reasons.is_empty() {
        ExperimentVerdict::NeedsHumanReview
    } else {
        ExperimentVerdict::Pass
    }
}

#[allow(clippy::too_many_arguments)]
fn write_analysis_json(
    config: &super::ExperimentWorkerConfig,
    provenance: &RuntimeProvenance,
    simulation: &Simulation,
    path: &Path,
    baseline: &PressureBaseline,
    observations: &PressureObservations,
    terminal_reason: TerminalReason,
    post_opening_end_tick: Option<u64>,
    trend: &TerminalTrend,
    predicates: &PressurePredicates,
    review_flags: &ReviewFlags,
    verdict: ExperimentVerdict,
    sample_count: u64,
    raw_frame_count: usize,
    exact_reset: bool,
) -> Result<(), String> {
    let predicate_json = |name: &str, value: &PredicateResult| {
        format!(
            "\"{}\":{{\"status\":\"{}\",\"detail\":\"{}\"}}",
            name,
            value.status.as_str(),
            json_escape(&value.detail)
        )
    };
    let predicates_json = [
        predicate_json(
            "pressure_activity_observed",
            &predicates.pressure_activity_observed,
        ),
        predicate_json("relief_seam_damaged", &predicates.relief_seam_damaged),
        predicate_json(
            "persistent_opening_created",
            &predicates.persistent_opening_created,
        ),
        predicate_json("exterior_vent_observed", &predicates.exterior_vent_observed),
        predicate_json(
            "post_opening_pressure_relieved",
            &predicates.post_opening_pressure_relieved,
        ),
        predicate_json(
            "terminal_pressure_not_runaway",
            &predicates.terminal_pressure_not_runaway,
        ),
        predicate_json("no_invalid_materials", &predicates.no_invalid_materials),
        predicate_json("no_nonfinite_fields", &predicates.no_nonfinite_fields),
        predicate_json("exact_reset", &predicates.exact_reset),
    ]
    .join(",");
    let identity_tick = |identity: Option<SampleIdentity>| identity.map(|item| item.sim_tick);
    let identity_sample =
        |identity: Option<SampleIdentity>| identity.map(|item| item.sample_sequence);
    let json_opt_f64 = |value: Option<f64>| {
        value.map_or_else(|| "null".to_string(), |value| format!("{value:.9}"))
    };
    let reasons = review_flags
        .reasons
        .iter()
        .map(|reason| format!("\"{}\"", json_escape(reason)))
        .collect::<Vec<_>>()
        .join(",");
    let latest = &observations.latest;
    let terminal_pressure_relieved =
        predicates.post_opening_pressure_relieved.status == PredicateStatus::Pass;
    let json = format!(
        concat!(
            "{{\n  \"schema_version\": \"{}\",\n  \"experiment_id\": \"{}\",",
            "\n  \"run_id\": \"{}\",\n  \"scenario\": \"pressure-burst\",",
            "\n  \"binary_sha256\": \"{}\",",
            "\n  \"provenance\": {{\"source_sha\":\"{}\",\"git_state\":\"{}\",\"build_profile\":\"{}\"}},",
            "\n  \"world\": {{\"width\":{},\"height\":{},\"chunk_size\":{}}},",
            "\n  \"sleep\": {{\"enabled\":{},\"threshold\":{}}},",
            "\n  \"lifecycle\": {{\"max_ticks\":{},\"diagnostic_interval_ticks\":{},",
            "\"consecutive_persistent_opening_samples\":{},\"post_opening_ticks\":{},",
            "\"terminal_window_samples\":{},\"terminal_reason\":\"{}\",",
            "\"persistent_opening_start_sim_tick\":{},",
            "\"persistent_opening_start_sample_sequence\":{},",
            "\"persistent_opening_confirmed_sim_tick\":{},",
            "\"persistent_opening_confirmed_sample_sequence\":{},",
            "\"post_opening_end_tick\":{},\"sample_count\":{}}},",
            "\n  \"baseline\": {{\"initial_matter_count\":{},",
            "\"initial_water_count\":{},\"initial_steam_count\":{},",
            "\"initial_relief_seam_wood_cells\":{},",
            "\"initial_top_relief_seam_wood_cells\":{},",
            "\"initial_bottom_relief_seam_wood_cells\":{},",
            "\"initial_chamber_pressure_cell_count\":{},",
            "\"initial_chamber_mean_pressure\":{:.9},",
            "\"initial_chamber_max_pressure\":{:.9}}},",
            "\n  \"metrics\": {{\"first_pressure_activity_tick\":{},",
            "\"first_pressure_activity_sample_sequence\":{},",
            "\"first_wood_damage_tick\":{},\"first_wood_damage_sample_sequence\":{},",
            "\"first_rupture_tick\":{},\"first_rupture_sample_sequence\":{},",
            "\"first_persistent_opening_tick\":{},",
            "\"first_persistent_opening_sample_sequence\":{},",
            "\"persistent_opening_confirmed_tick\":{},",
            "\"persistent_opening_confirmed_sample_sequence\":{},",
            "\"first_steam_in_relief_seam_tick\":{},",
            "\"first_steam_in_relief_seam_sample_sequence\":{},",
            "\"first_outside_chamber_steam_tick\":{},",
            "\"first_outside_chamber_steam_sample_sequence\":{},",
            "\"first_post_confirmation_reseal_tick\":{},",
            "\"first_post_confirmation_reseal_sample_sequence\":{},",
            "\"first_post_opening_relief_tick\":{},",
            "\"first_post_opening_relief_sample_sequence\":{},",
            "\"vent_reference_chamber_mean_pressure\":{},",
            "\"vent_reference_chamber_max_pressure\":{},",
            "\"peak_chamber_mean_pressure\":{:.9},",
            "\"peak_chamber_mean_pressure_tick\":{},",
            "\"peak_chamber_mean_pressure_sample_sequence\":{},",
            "\"peak_chamber_max_pressure\":{:.9},",
            "\"peak_chamber_max_pressure_tick\":{},",
            "\"peak_chamber_max_pressure_sample_sequence\":{},",
            "\"peak_pressure_active_cells\":{},\"peak_pressure_active_tick\":{},",
            "\"peak_pressure_active_sample_sequence\":{},",
            "\"pre_opening_peak_chamber_mean_pressure\":{:.9},",
            "\"pre_opening_peak_chamber_max_pressure\":{:.9},",
            "\"post_opening_chamber_mean_pressure\":{},",
            "\"post_opening_chamber_max_pressure\":{},",
            "\"terminal_chamber_mean_pressure\":{},",
            "\"terminal_chamber_max_pressure\":{},",
            "\"terminal_pressure_relieved\":{},",
            "\"final_relief_seam_wood_cells\":{},",
            "\"final_top_relief_seam_wood_cells\":{},",
            "\"final_bottom_relief_seam_wood_cells\":{},",
            "\"final_relief_seam_open_cells\":{},",
            "\"final_top_relief_seam_open_cells\":{},",
            "\"final_bottom_relief_seam_open_cells\":{},",
            "\"final_steam_in_relief_seam_cells\":{},",
            "\"outside_chamber_steam_peak\":{},",
            "\"final_outside_chamber_steam_cells\":{},",
            "\"final_matter_count\":{},\"matter_count_delta\":{},",
            "\"final_water_count\":{},\"water_count_delta\":{},",
            "\"final_steam_count\":{},\"steam_count_delta\":{},",
            "\"final_pressure_active_cells\":{},",
            "\"final_thermal_active_cells\":{},",
            "\"final_reaction_active_cells\":{},",
            "\"invalid_material_occurrences\":{},",
            "\"nonfinite_field_occurrences\":{},",
            "\"reset_exact_equivalence\":{}}},",
            "\n  \"terminal_window\": {{\"sample_count\":{},",
            "\"start_sim_tick\":{},\"end_sim_tick\":{},",
            "\"start_mean_pressure\":{},\"end_mean_pressure\":{},",
            "\"start_max_pressure\":{},\"end_max_pressure\":{},",
            "\"minimum_mean_pressure\":{},\"maximum_mean_pressure\":{},",
            "\"slope_per_sample\":{},\"positive_step_count\":{},",
            "\"positive_max_step_count\":{},",
            "\"mean_unbounded_growth\":{},\"max_unbounded_growth\":{},",
            "\"unbounded_growth\":{}}},",
            "\n  \"review_flags\": {{\"only_one_relief_seam_ruptured\":{},",
            "\"high_terminal_pressure_activity\":{},\"long_pressure_tail\":{},",
            "\"persistent_vent_plume\":{},\"terminal_activity_remains\":{},",
            "\"reasons\":[{}]}},",
            "\n  \"predicates\": {{{}}},",
            "\n  \"verdict\": \"{}\",\n  \"raw_frame_count\": {}\n}}\n"
        ),
        PRESSURE_ANALYSIS_SCHEMA_VERSION,
        json_escape(&config.experiment_id),
        json_escape(&config.run_id),
        json_escape(&config.binary_sha256.to_ascii_lowercase()),
        json_escape(&provenance.source_sha),
        provenance.git_state.as_str(),
        provenance.build_profile,
        simulation.world.config.width,
        simulation.world.config.height,
        simulation.world.config.chunk_size,
        simulation.sleep_enabled,
        simulation.sleep_threshold,
        config.max_ticks,
        config.diagnostic_interval_ticks,
        config.consecutive_persistent_opening,
        config.post_opening_ticks,
        config.terminal_window_samples,
        terminal_reason.as_str(),
        json_opt_u64(identity_tick(observations.persistent_opening_start)),
        json_opt_u64(identity_sample(observations.persistent_opening_start)),
        json_opt_u64(identity_tick(observations.persistent_opening_confirmed)),
        json_opt_u64(identity_sample(observations.persistent_opening_confirmed)),
        json_opt_u64(post_opening_end_tick),
        sample_count,
        baseline.matter_count,
        baseline.water_count,
        baseline.steam_count,
        baseline.relief_seam_wood_cells,
        baseline.top_relief_seam_wood_cells,
        baseline.bottom_relief_seam_wood_cells,
        baseline.chamber_pressure_cell_count,
        baseline.chamber_mean_pressure,
        baseline.chamber_max_pressure,
        json_opt_u64(identity_tick(observations.first_pressure_activity)),
        json_opt_u64(identity_sample(observations.first_pressure_activity)),
        json_opt_u64(identity_tick(observations.first_wood_damage)),
        json_opt_u64(identity_sample(observations.first_wood_damage)),
        json_opt_u64(identity_tick(observations.first_rupture)),
        json_opt_u64(identity_sample(observations.first_rupture)),
        json_opt_u64(identity_tick(observations.persistent_opening_start)),
        json_opt_u64(identity_sample(observations.persistent_opening_start)),
        json_opt_u64(identity_tick(observations.persistent_opening_confirmed)),
        json_opt_u64(identity_sample(observations.persistent_opening_confirmed)),
        json_opt_u64(identity_tick(observations.first_steam_in_relief_seam)),
        json_opt_u64(identity_sample(observations.first_steam_in_relief_seam)),
        json_opt_u64(identity_tick(observations.first_exterior_steam)),
        json_opt_u64(identity_sample(observations.first_exterior_steam)),
        json_opt_u64(identity_tick(observations.first_post_confirmation_reseal)),
        json_opt_u64(identity_sample(observations.first_post_confirmation_reseal)),
        json_opt_u64(identity_tick(observations.first_post_opening_relief)),
        json_opt_u64(identity_sample(observations.first_post_opening_relief)),
        json_opt_f64(observations.vent_reference_chamber_mean_pressure),
        json_opt_f64(observations.vent_reference_chamber_max_pressure),
        observations.peak_chamber_mean_pressure,
        observations.peak_chamber_mean.sim_tick,
        observations.peak_chamber_mean.sample_sequence,
        observations.peak_chamber_max_pressure,
        observations.peak_chamber_max.sim_tick,
        observations.peak_chamber_max.sample_sequence,
        observations.peak_pressure_active_cells,
        observations.peak_pressure_activity.sim_tick,
        observations.peak_pressure_activity.sample_sequence,
        observations.pre_opening_peak_chamber_mean_pressure,
        observations.pre_opening_peak_chamber_max_pressure,
        json_opt_f64(observations.post_opening_chamber_mean_pressure),
        json_opt_f64(observations.post_opening_chamber_max_pressure),
        json_opt_f64(trend.end_mean_pressure),
        json_opt_f64(trend.end_max_pressure),
        terminal_pressure_relieved,
        latest.relief_seam_wood_cells,
        latest.top_relief_seam_wood_cells,
        latest.bottom_relief_seam_wood_cells,
        latest.relief_seam_open_cells,
        latest.top_relief_seam_open_cells,
        latest.bottom_relief_seam_open_cells,
        latest.steam_in_relief_seam_cells,
        observations.outside_chamber_steam_peak,
        latest.outside_chamber_steam_cells,
        latest.matter_count,
        i128::from(latest.matter_count) - i128::from(baseline.matter_count),
        latest.water_count,
        i128::from(latest.water_count) - i128::from(baseline.water_count),
        latest.steam_count,
        i128::from(latest.steam_count) - i128::from(baseline.steam_count),
        latest.pressure_active_cells,
        latest.thermal_active_cells,
        latest.reaction_active_cells,
        observations.invalid_material_occurrences,
        observations.nonfinite_field_occurrences,
        exact_reset,
        trend.sample_count,
        json_opt_u64(trend.start_sim_tick),
        json_opt_u64(trend.end_sim_tick),
        json_opt_f64(trend.start_mean_pressure),
        json_opt_f64(trend.end_mean_pressure),
        json_opt_f64(trend.start_max_pressure),
        json_opt_f64(trend.end_max_pressure),
        json_opt_f64(trend.minimum_mean_pressure),
        json_opt_f64(trend.maximum_mean_pressure),
        json_opt_f64(trend.slope_per_sample),
        trend.positive_step_count,
        trend.positive_max_step_count,
        trend.mean_unbounded_growth,
        trend.max_unbounded_growth,
        trend.unbounded_growth,
        review_flags.only_one_relief_seam_ruptured,
        review_flags.high_terminal_pressure_activity,
        review_flags.long_pressure_tail,
        review_flags.persistent_vent_plume,
        review_flags.terminal_activity_remains,
        reasons,
        predicates_json,
        verdict.as_str(),
        raw_frame_count,
    );
    write_new(path, json.as_bytes())
}

#[allow(clippy::too_many_arguments)]
fn record_observation_updates(
    output: &mut PressureJsonlWriters,
    config: &super::ExperimentWorkerConfig,
    update: ObservationUpdate,
    metrics: &PressureSampleMetrics,
    frame: Option<&PressureFrame>,
    first_pressure_activity_frame: &mut Option<PressureFrame>,
    first_wood_damage_frame: &mut Option<PressureFrame>,
    first_rupture_frame: &mut Option<PressureFrame>,
    first_exterior_steam_frame: &mut Option<PressureFrame>,
    peak_pressure_frame: &mut Option<PressureFrame>,
    peak_pressure_activity_frame: &mut Option<PressureFrame>,
    post_opening_frame: &mut Option<PressureFrame>,
    reseal_frame: &mut Option<PressureFrame>,
) -> Result<(), String> {
    let require_frame = || {
        frame.ok_or_else(|| "Pressure milestone update was recorded without a frame".to_string())
    };
    if update.first_pressure_activity {
        *first_pressure_activity_frame = Some(
            require_frame()?
                .with_badge("first-pressure-activity", "first-sampled-pressure-activity"),
        );
        output.event(
            config,
            "pressure_activity_observed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!("pressure_active_cells={}", metrics.pressure_active_cells),
        )?;
    }
    if update.first_wood_damage {
        *first_wood_damage_frame = Some(
            require_frame()?
                .with_badge("first-wood-damage", "first-authored-relief-seam-wood-loss"),
        );
        output.event(
            config,
            "relief_seam_damage_observed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!(
                "wood_total={};top={};bottom={};open_total={}",
                metrics.relief_seam_wood_cells,
                metrics.top_relief_seam_wood_cells,
                metrics.bottom_relief_seam_wood_cells,
                metrics.relief_seam_open_cells
            ),
        )?;
    }
    if update.first_rupture {
        *first_rupture_frame = Some(require_frame()?.with_badge(
            "first-rupture",
            "cold-bottom-seam-pressure-attributed-opening",
        ));
        output.event(
            config,
            "rupture_observed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!(
                "cold_bottom=true;bottom_wood={};bottom_open={}",
                metrics.bottom_relief_seam_wood_cells, metrics.bottom_relief_seam_open_cells
            ),
        )?;
    }
    if update.first_steam_in_relief_seam {
        output.event(
            config,
            "relief_seam_steam_observed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!(
                "steam_in_relief_seam_cells={}",
                metrics.steam_in_relief_seam_cells
            ),
        )?;
    }
    if update.first_exterior_steam {
        *first_exterior_steam_frame = Some(require_frame()?.with_badge(
            "first-exterior-steam",
            "first-steam-outside-authored-chamber-after-opening",
        ));
        output.event(
            config,
            "exterior_vent_observed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!(
                "outside_chamber_steam_cells={};steam_in_relief_seam_cells={}",
                metrics.outside_chamber_steam_cells, metrics.steam_in_relief_seam_cells
            ),
        )?;
    }
    if update.first_post_confirmation_reseal {
        *reseal_frame = Some(require_frame()?.with_badge(
            "opening-reseal",
            "first-zero-open-cell-sample-after-persistent-confirmation",
        ));
        output.event(
            config,
            "post_confirmation_reseal_observed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            "relief_seam_open_cells=0",
        )?;
    }
    if update.new_peak_chamber_mean {
        output.event(
            config,
            "new_peak_chamber_mean_pressure",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!("chamber_mean_pressure={:.9}", metrics.chamber_mean_pressure),
        )?;
    }
    if update.new_peak_chamber_max {
        *peak_pressure_frame = Some(
            require_frame()?.with_badge("peak-pressure", "highest-observed-chamber-max-pressure"),
        );
        output.event(
            config,
            "new_peak_chamber_max_pressure",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!("chamber_max_pressure={:.9}", metrics.chamber_max_pressure),
        )?;
    }
    if update.new_peak_pressure_activity {
        *peak_pressure_activity_frame = Some(require_frame()?.with_badge(
            "peak-pressure-activity",
            "highest-observed-pressure-active-cells",
        ));
        output.event(
            config,
            "new_peak_pressure_activity",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!("pressure_active_cells={}", metrics.pressure_active_cells),
        )?;
    }
    if update.first_post_opening_relief {
        *post_opening_frame = Some(require_frame()?.with_badge(
            "post-opening",
            "first-post-vent-chamber-mean-and-max-pressure-relief",
        ));
        output.event(
            config,
            "post_opening_pressure_relief_observed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!(
                "chamber_mean_pressure={:.9};chamber_max_pressure={:.9}",
                metrics.chamber_mean_pressure, metrics.chamber_max_pressure
            ),
        )?;
    }
    Ok(())
}

/// Runs the Pressure Burst experiment through the production simulation path.
/// Semantic findings are completed outcomes; operational failures remain Err.
pub fn run_pressure_burst_experiment(
    simulation: &mut Simulation,
    renderer: &mut Renderer,
    provenance: &RuntimeProvenance,
    config: &super::ExperimentWorkerConfig,
) -> Result<ExperimentOutcome, String> {
    validate_pressure_worker_config(simulation, config)?;

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
    let mut output = PressureJsonlWriters::new(&samples_path, &events_path)?;
    output.event(
        config,
        "lifecycle_started",
        simulation.tick_count,
        None,
        "Pressure worker output opened",
    )?;

    reset_and_stage_scenario(simulation, config.scenario)
        .map_err(|error| format!("pristine Pressure Burst reset/stage failed: {error}"))?;
    output.event(
        config,
        "pristine_reset_completed",
        0,
        None,
        "shared Pressure Burst reset/staging completed",
    )?;
    let baseline_sleep_enabled = simulation.sleep_enabled;
    let baseline_sleep_threshold = simulation.sleep_threshold;
    let mut next_sample_sequence = 0u64;
    let tick0_snapshot = capture_gpu_snapshot(simulation)?;
    let tick0_metrics = pressure_metrics_from_snapshot(
        &tick0_snapshot,
        simulation.world.config,
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "initial",
        "tick0",
    )?;
    let baseline = baseline_from_tick0(&tick0_metrics)?;
    output.sample(config, provenance, simulation, &tick0_metrics)?;
    let tick0_frame = capture_pressure_frame(renderer, &tick0_metrics, "tick0", "pristine-reset")?;
    output.event(
        config,
        "tick0_captured",
        tick0_metrics.sim_tick,
        Some(tick0_metrics.sample_sequence),
        &tick0_metrics.state_hash,
    )?;

    let mut observations = PressureObservations::new(&tick0_metrics);
    let mut first_pressure_activity_frame = None;
    let mut first_wood_damage_frame = None;
    let mut first_rupture_frame = None;
    let mut persistent_opening_frame = None;
    let mut first_exterior_steam_frame = None;
    let mut peak_pressure_frame =
        Some(tick0_frame.with_badge("peak-pressure", "highest-observed-chamber-max-pressure"));
    let mut peak_pressure_activity_frame = Some(tick0_frame.with_badge(
        "peak-pressure-activity",
        "highest-observed-pressure-active-cells",
    ));
    let mut post_opening_frame = None;
    let mut reseal_frame = None;
    let mut diagnostics = VecDeque::with_capacity(DIAGNOSTIC_RING_CAPACITY);
    let mut terminal_samples = VecDeque::with_capacity(config.terminal_window_samples as usize);
    let mut opening_detector =
        PersistentOpeningDetector::new(config.consecutive_persistent_opening);

    simulation
        .tick()
        .map_err(|error| format!("Pressure production tick 1 failed: {error}"))?;
    let tick1_snapshot = capture_gpu_snapshot(simulation)?;
    let tick1_metrics = pressure_metrics_from_snapshot(
        &tick1_snapshot,
        simulation.world.config,
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "pressurizing",
        "tick1",
    )?;
    let tick1_opening_update = opening_detector.observe(&tick1_metrics);
    debug_assert!(!tick1_opening_update.confirmed);
    let tick1_update = observations.observe(&tick1_metrics, &baseline);
    output.sample(config, provenance, simulation, &tick1_metrics)?;
    let tick1_frame = capture_pressure_frame(
        renderer,
        &tick1_metrics,
        "tick1",
        "after-one-production-tick",
    )?;
    record_observation_updates(
        &mut output,
        config,
        tick1_update,
        &tick1_metrics,
        Some(&tick1_frame),
        &mut first_pressure_activity_frame,
        &mut first_wood_damage_frame,
        &mut first_rupture_frame,
        &mut first_exterior_steam_frame,
        &mut peak_pressure_frame,
        &mut peak_pressure_activity_frame,
        &mut post_opening_frame,
        &mut reseal_frame,
    )?;
    output.event(
        config,
        "tick1_captured",
        tick1_metrics.sim_tick,
        Some(tick1_metrics.sample_sequence),
        &tick1_metrics.state_hash,
    )?;
    if tick1_opening_update.first_in_streak {
        output.event(
            config,
            "persistent_opening_streak_started",
            tick1_metrics.sim_tick,
            Some(tick1_metrics.sample_sequence),
            &format!(
                "relief_seam_open_cells={}",
                tick1_metrics.relief_seam_open_cells
            ),
        )?;
    }
    remember_terminal_sample(
        &mut terminal_samples,
        &tick1_metrics,
        config.terminal_window_samples as usize,
    );

    let mut terminal_reason = TerminalReason::MaxTicks;
    let mut terminal_metrics: Option<PressureSampleMetrics> = None;
    let mut post_opening_end_tick = None;

    while simulation.tick_count < config.max_ticks {
        simulation.tick().map_err(|error| {
            format!(
                "Pressure production tick {} failed: {error}",
                simulation.tick_count + 1
            )
        })?;
        let sim_tick = simulation.tick_count;
        let early = sim_tick == 2;
        let cadence = sim_tick.is_multiple_of(config.diagnostic_interval_ticks);
        let max_tick = sim_tick == config.max_ticks;
        if !early && !cadence && !max_tick {
            continue;
        }
        let reason = if early {
            "early-diagnostic"
        } else if max_tick {
            "max-tick"
        } else {
            "diagnostic-cadence"
        };
        let snapshot = capture_gpu_snapshot(simulation)?;
        let metrics = pressure_metrics_from_snapshot(
            &snapshot,
            simulation.world.config,
            take_sequence(&mut next_sample_sequence),
            sim_tick,
            "pressurizing",
            reason,
        )?;
        let opening_update = opening_detector.observe(&metrics);
        let confirmed_first = if opening_update.confirmed {
            let first = opening_detector.first.unwrap_or(metrics.identity());
            observations.confirm_opening(first, metrics.identity());
            Some(first)
        } else {
            None
        };
        let update = observations.observe(&metrics, &baseline);
        output.sample(config, provenance, simulation, &metrics)?;
        remember_terminal_sample(
            &mut terminal_samples,
            &metrics,
            config.terminal_window_samples as usize,
        );

        let fallback_capture =
            sim_tick.is_multiple_of(config.diagnostic_interval_ticks.saturating_mul(128));
        let needs_frame = update.needs_frame()
            || opening_update.first_in_streak
            || opening_update.confirmed
            || fallback_capture
            || max_tick;
        let frame = if needs_frame {
            Some(capture_pressure_frame(
                renderer,
                &metrics,
                "diagnostic-observation",
                reason,
            )?)
        } else {
            None
        };
        if opening_update.streak_broken {
            output.event(
                config,
                "persistent_opening_streak_broken",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                "relief seam returned to zero non-Wood cells",
            )?;
        }
        if opening_update.first_in_streak {
            output.event(
                config,
                "persistent_opening_streak_started",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                &format!("relief_seam_open_cells={}", metrics.relief_seam_open_cells),
            )?;
        }
        if opening_update.confirmed {
            let first = confirmed_first.expect("opening confirmation stores first sample");
            persistent_opening_frame = Some(
                frame
                    .as_ref()
                    .expect("opening confirmation requests frame")
                    .with_badge(
                        "persistent-opening",
                        "three-consecutive-diagnostics-with-opening",
                    ),
            );
            output.event(
                config,
                "persistent_opening_confirmed",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                &format!(
                    "required={};first_tick={};first_sample={};open_cells={}",
                    config.consecutive_persistent_opening,
                    first.sim_tick,
                    first.sample_sequence,
                    metrics.relief_seam_open_cells
                ),
            )?;
            output.event(
                config,
                "post_opening_observation_started",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                &format!("production_ticks={}", config.post_opening_ticks),
            )?;
        }
        record_observation_updates(
            &mut output,
            config,
            update,
            &metrics,
            frame.as_ref(),
            &mut first_pressure_activity_frame,
            &mut first_wood_damage_frame,
            &mut first_rupture_frame,
            &mut first_exterior_steam_frame,
            &mut peak_pressure_frame,
            &mut peak_pressure_activity_frame,
            &mut post_opening_frame,
            &mut reseal_frame,
        )?;
        if fallback_capture {
            let fallback = frame
                .as_ref()
                .expect("fallback capture requested")
                .with_badge("diagnostic-observation", "minimum-evidence-observation");
            if diagnostics.len() == DIAGNOSTIC_RING_CAPACITY {
                let _ = diagnostics.pop_front();
            }
            diagnostics.push_back(fallback);
        }
        if opening_update.confirmed {
            terminal_samples.clear();
            remember_terminal_sample(
                &mut terminal_samples,
                &metrics,
                config.terminal_window_samples as usize,
            );
            terminal_metrics = Some(metrics);
            break;
        }
        if max_tick {
            terminal_metrics = Some(metrics);
            terminal_reason = TerminalReason::MaxTicks;
            break;
        }
    }

    if observations.persistent_opening_confirmed.is_some() {
        for offset in 1..=config.post_opening_ticks {
            if simulation.tick_count >= config.max_ticks {
                terminal_reason = TerminalReason::MaxTicks;
                break;
            }
            simulation.tick().map_err(|error| {
                format!(
                    "Pressure post-opening production tick {offset}/{} failed: {error}",
                    config.post_opening_ticks
                )
            })?;
            let snapshot = capture_gpu_snapshot(simulation)?;
            let reason = if offset == config.post_opening_ticks {
                "post-opening-observation-complete"
            } else if simulation.tick_count == config.max_ticks {
                "max-tick"
            } else {
                "post-opening-tick"
            };
            let metrics = pressure_metrics_from_snapshot(
                &snapshot,
                simulation.world.config,
                take_sequence(&mut next_sample_sequence),
                simulation.tick_count,
                "post-opening-observation",
                reason,
            )?;
            let update = observations.observe(&metrics, &baseline);
            output.sample(config, provenance, simulation, &metrics)?;
            remember_terminal_sample(
                &mut terminal_samples,
                &metrics,
                config.terminal_window_samples as usize,
            );
            let fallback_capture = offset.is_multiple_of(32);
            let needs_frame = update.needs_frame()
                || fallback_capture
                || offset == config.post_opening_ticks
                || simulation.tick_count == config.max_ticks;
            let frame = if needs_frame {
                Some(capture_pressure_frame(
                    renderer,
                    &metrics,
                    "diagnostic-observation",
                    reason,
                )?)
            } else {
                None
            };
            record_observation_updates(
                &mut output,
                config,
                update,
                &metrics,
                frame.as_ref(),
                &mut first_pressure_activity_frame,
                &mut first_wood_damage_frame,
                &mut first_rupture_frame,
                &mut first_exterior_steam_frame,
                &mut peak_pressure_frame,
                &mut peak_pressure_activity_frame,
                &mut post_opening_frame,
                &mut reseal_frame,
            )?;
            if fallback_capture {
                let fallback = frame
                    .as_ref()
                    .expect("fallback capture requested")
                    .with_badge("diagnostic-observation", "minimum-evidence-observation");
                if diagnostics.len() == DIAGNOSTIC_RING_CAPACITY {
                    let _ = diagnostics.pop_front();
                }
                diagnostics.push_back(fallback);
            }
            terminal_metrics = Some(metrics.clone());
            if offset == config.post_opening_ticks {
                terminal_reason = TerminalReason::PostOpeningObservationComplete;
                post_opening_end_tick = Some(metrics.sim_tick);
                output.event(
                    config,
                    "post_opening_observation_completed",
                    metrics.sim_tick,
                    Some(metrics.sample_sequence),
                    &format!("production_ticks={}", config.post_opening_ticks),
                )?;
                break;
            }
            if simulation.tick_count == config.max_ticks {
                terminal_reason = TerminalReason::MaxTicks;
                break;
            }
        }
    }

    let terminal_metrics = terminal_metrics.ok_or_else(|| {
        "Pressure lifecycle did not produce a terminal diagnostic sample".to_string()
    })?;
    let terminal_frame = capture_pressure_frame(
        renderer,
        &terminal_metrics,
        "terminal",
        terminal_reason.as_str(),
    )?;
    output.event(
        config,
        "terminal_selected",
        terminal_metrics.sim_tick,
        Some(terminal_metrics.sample_sequence),
        terminal_reason.as_str(),
    )?;

    output.event(
        config,
        "reset_started",
        simulation.tick_count,
        Some(terminal_metrics.sample_sequence),
        "programmatic R-equivalent shared Pressure Burst reset/staging",
    )?;
    reset_and_stage_scenario(simulation, config.scenario)
        .map_err(|error| format!("programmatic Pressure Burst reset failed: {error}"))?;
    let reset_snapshot = capture_gpu_snapshot(simulation)?;
    let reset_metrics = pressure_metrics_from_snapshot(
        &reset_snapshot,
        simulation.world.config,
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "reset",
        "programmatic-r-equivalent",
    )?;
    output.sample(config, provenance, simulation, &reset_metrics)?;
    let reset_frame = capture_pressure_frame(
        renderer,
        &reset_metrics,
        "reset",
        "programmatic-r-equivalent",
    )?;
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

    let trend = terminal_trend(&terminal_samples);
    let predicates = build_predicates(
        &observations,
        &trend,
        config.terminal_window_samples as usize,
        exact_reset,
    );
    let review_flags = review_flags(&baseline, &observations, &trend);
    let verdict = pressure_verdict(&predicates, &review_flags);
    let frames = assemble_frames(
        [
            Some(tick0_frame),
            Some(tick1_frame),
            first_pressure_activity_frame,
            first_wood_damage_frame,
            first_rupture_frame,
            persistent_opening_frame,
            first_exterior_steam_frame,
            peak_pressure_frame,
            peak_pressure_activity_frame,
            reseal_frame.or(post_opening_frame),
            Some(terminal_frame),
            Some(reset_frame),
        ],
        &diagnostics,
    )?;
    let written_frames = write_pressure_raw_frames(&raw_frames_dir, frames)?;
    write_frames_json(config, &frames_path, &written_frames)?;
    write_analysis_json(
        config,
        provenance,
        simulation,
        &analysis_path,
        &baseline,
        &observations,
        terminal_reason,
        post_opening_end_tick,
        &trend,
        &predicates,
        &review_flags,
        verdict,
        next_sample_sequence,
        written_frames.len(),
        exact_reset,
    )?;
    output.event(
        config,
        "worker_completed",
        simulation.tick_count,
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
        post_sleep_end_tick: Some(terminal_metrics.sim_tick),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(world: WorldConfig) -> GpuSnapshot {
        let cells = world.cell_count().expect("cell count") as usize;
        let chunks =
            powdergame_core::chunk_count(world.width, world.height, world.chunk_size) as usize;
        GpuSnapshot {
            material_current: vec![MATERIAL_EMPTY; cells],
            material_next: vec![MATERIAL_EMPTY; cells],
            temperature_current: vec![0.0f32.to_bits(); cells],
            temperature_next: vec![0.0f32.to_bits(); cells],
            pressure_current: vec![0.0f32.to_bits(); cells],
            pressure_next: vec![0.0f32.to_bits(); cells],
            flags_current: vec![0; cells],
            flags_next: vec![0; cells],
            proposal: vec![0; cells],
            claim: vec![0; cells],
            cell_activity: vec![0; cells],
            chunk_activity: vec![0; chunks],
            chunk_changed: vec![0; chunks],
            chunk_stable: vec![0; chunks],
            chunk_edit_wake: vec![0; chunks],
            chunk_state: vec![CHUNK_STATE_SLEEPING; chunks],
            chunk_wake_reason: vec![0; chunks],
            params: vec![0; 8],
            wake_params: vec![0; 8],
            arbitration_params: vec![0; 8],
        }
    }

    fn set_material(
        snapshot: &mut GpuSnapshot,
        world: WorldConfig,
        x: usize,
        y: usize,
        value: u32,
    ) {
        let index = y * world.width as usize + x;
        snapshot.material_current[index] = value;
        snapshot.material_next[index] = value;
    }

    fn authored_metric_snapshot() -> (WorldConfig, GpuSnapshot) {
        let world = WorldConfig::new(256, 256, 64).expect("world");
        let mut snapshot = snapshot(world);
        for y in TOP_SEAM_MIN_Y..TOP_SEAM_MAX_Y {
            for x in TOP_SEAM_MIN_X..TOP_SEAM_MAX_X {
                set_material(&mut snapshot, world, x, y, MATERIAL_WOOD);
            }
        }
        for y in BOTTOM_SEAM_MIN_Y..BOTTOM_SEAM_MAX_Y {
            for x in BOTTOM_SEAM_MIN_X..BOTTOM_SEAM_MAX_X {
                set_material(&mut snapshot, world, x, y, MATERIAL_WOOD);
            }
        }
        for y in CAVITY_MIN_Y..CAVITY_MAX_Y {
            for x in CAVITY_MIN_X..CAVITY_MAX_X {
                let index = y * world.width as usize + x;
                snapshot.pressure_current[index] = 180.0f32.to_bits();
                snapshot.pressure_next[index] = 180.0f32.to_bits();
            }
        }
        (world, snapshot)
    }

    fn metrics(tick: u64, mean: f64, max: f64, pressure_active: u64) -> PressureSampleMetrics {
        PressureSampleMetrics {
            sample_sequence: tick,
            sim_tick: tick,
            phase: "test",
            reason: "test",
            total_cells: 65_536,
            any_active_cells: pressure_active,
            matter_active_cells: 0,
            thermal_active_cells: 0,
            pressure_active_cells: pressure_active,
            reaction_active_cells: 0,
            total_chunks: 16,
            active_chunks: u32::from(pressure_active != 0),
            runnable_chunks: u32::from(pressure_active != 0),
            sleeping_chunks: u32::from(pressure_active == 0) * 16,
            material_counts_by_id: [0; 10],
            matter_count: 100,
            water_count: 60,
            steam_count: 40,
            relief_seam_wood_cells: 576,
            top_relief_seam_wood_cells: 384,
            bottom_relief_seam_wood_cells: 192,
            relief_seam_open_cells: 0,
            top_relief_seam_open_cells: 0,
            bottom_relief_seam_open_cells: 0,
            steam_in_relief_seam_cells: 0,
            outside_chamber_steam_cells: 0,
            chamber_pressure_cell_count: CHAMBER_PRESSURE_CELL_COUNT,
            chamber_mean_pressure: quantize_json_9(mean),
            chamber_max_pressure: quantize_json_9(max),
            invalid_material_count: 0,
            nonfinite_temperature_count: 0,
            nonfinite_pressure_count: 0,
            changed_chunks: 0,
            wake_chunks: 0,
            wake_reason_or: 0,
            state_hash: format!("state-{tick}"),
            physical_state_hash: format!("physical-{tick}"),
        }
    }

    fn baseline(metrics: &PressureSampleMetrics) -> PressureBaseline {
        PressureBaseline {
            matter_count: metrics.matter_count,
            water_count: metrics.water_count,
            steam_count: metrics.steam_count,
            relief_seam_wood_cells: metrics.relief_seam_wood_cells,
            top_relief_seam_wood_cells: metrics.top_relief_seam_wood_cells,
            bottom_relief_seam_wood_cells: metrics.bottom_relief_seam_wood_cells,
            chamber_pressure_cell_count: metrics.chamber_pressure_cell_count,
            chamber_mean_pressure: metrics.chamber_mean_pressure,
            chamber_max_pressure: metrics.chamber_max_pressure,
        }
    }

    #[test]
    fn pressure_geometry_detectors_pin_authored_seams_and_cavity() {
        let (world, snapshot) = authored_metric_snapshot();
        let metrics = pressure_metrics_from_snapshot(&snapshot, world, 0, 0, "test", "test")
            .expect("metrics");
        assert_eq!(metrics.relief_seam_wood_cells, 576);
        assert_eq!(metrics.top_relief_seam_wood_cells, 384);
        assert_eq!(metrics.bottom_relief_seam_wood_cells, 192);
        assert_eq!(metrics.relief_seam_open_cells, 0);
        assert_eq!(metrics.chamber_pressure_cell_count, 29_920);
        assert_eq!(metrics.chamber_mean_pressure, 180.0);
        assert_eq!(metrics.chamber_max_pressure, 180.0);
        baseline_from_tick0(&metrics).expect("authored baseline");
    }

    #[test]
    fn pressure_rupture_opening_and_exterior_steam_detectors_are_geometric() {
        let (world, mut snapshot) = authored_metric_snapshot();
        set_material(
            &mut snapshot,
            world,
            BOTTOM_SEAM_MIN_X,
            BOTTOM_SEAM_MIN_Y,
            MATERIAL_EMPTY,
        );
        set_material(
            &mut snapshot,
            world,
            TOP_SEAM_MIN_X,
            TOP_SEAM_MIN_Y,
            MATERIAL_STEAM,
        );
        set_material(&mut snapshot, world, OUTER_MIN_X - 1, 100, MATERIAL_STEAM);
        set_material(&mut snapshot, world, 100, 100, MATERIAL_STEAM);
        let metrics = pressure_metrics_from_snapshot(&snapshot, world, 1, 8, "test", "test")
            .expect("metrics");
        assert_eq!(metrics.bottom_relief_seam_open_cells, 1);
        assert_eq!(metrics.top_relief_seam_open_cells, 1);
        assert_eq!(metrics.steam_in_relief_seam_cells, 1);
        assert_eq!(metrics.outside_chamber_steam_cells, 1);
    }

    #[test]
    fn pressure_persistent_opening_counts_tick1_tick2_and_first_cadence_sample() {
        let mut detector = PersistentOpeningDetector::new(3);
        let mut opened = metrics(1, 100.0, 120.0, 1);
        opened.relief_seam_open_cells = 1;
        assert!(detector.observe(&opened).first_in_streak);
        opened.sim_tick = 2;
        opened.sample_sequence = 2;
        assert!(!detector.observe(&opened).confirmed);
        opened.sim_tick = 8;
        opened.sample_sequence = 3;
        assert!(detector.observe(&opened).confirmed);
        assert_eq!(
            detector.first,
            Some(SampleIdentity {
                sim_tick: 1,
                sample_sequence: 1,
            })
        );
        let closed = metrics(16, 90.0, 100.0, 1);
        assert!(detector.observe(&closed).streak_broken);
    }

    #[test]
    fn pressure_terminal_trend_distinguishes_relief_from_runaway() {
        let falling = (0..64)
            .map(|index| metrics(index, 180.0 - index as f64, 200.0, 100))
            .collect::<VecDeque<_>>();
        let falling = terminal_trend(&falling);
        assert_eq!(falling.sample_count, 64);
        assert!(!falling.unbounded_growth);
        assert!(falling.slope_per_sample.expect("slope") < 0.0);

        let rising = (0..64)
            .map(|index| metrics(index, 10.0 + index as f64, 100.0, 100))
            .collect::<VecDeque<_>>();
        let rising = terminal_trend(&rising);
        assert!(rising.unbounded_growth);
        assert!(rising.mean_unbounded_growth);
        assert!(!rising.max_unbounded_growth);
        assert_eq!(rising.positive_step_count, 63);

        let max_only_rising = (0..64)
            .map(|index| metrics(index, 100.0, 10.0 + index as f64, 100))
            .collect::<VecDeque<_>>();
        let max_only_rising = terminal_trend(&max_only_rising);
        assert!(max_only_rising.unbounded_growth);
        assert!(!max_only_rising.mean_unbounded_growth);
        assert!(max_only_rising.max_unbounded_growth);
        assert_eq!(max_only_rising.positive_max_step_count, 63);
    }

    #[test]
    fn pressure_decisions_match_serialized_nine_decimal_boundary_values() {
        let rounded_equal_low = quantize_json_9(100.000_000_000_1);
        let rounded_equal_high = quantize_json_9(100.000_000_000_4);
        assert_eq!(rounded_equal_low, 100.0);
        assert_eq!(rounded_equal_high, 100.0);
        assert_eq!(format!("{rounded_equal_high:.9}"), "100.000000000");

        let initial = metrics(0, 100.000_000_000_1, 100.000_000_000_1, 0);
        let initial_baseline = baseline(&initial);
        let mut peak_observations = PressureObservations::new(&initial);
        let rounded_same_peak = metrics(1, 100.000_000_000_4, 100.000_000_000_4, 1);
        let peak_update = peak_observations.observe(&rounded_same_peak, &initial_baseline);
        assert!(!peak_update.new_peak_chamber_mean);
        assert!(!peak_update.new_peak_chamber_max);

        let initial = metrics(0, 180.0, 180.0, 0);
        let initial_baseline = baseline(&initial);
        let mut relief_observations = PressureObservations::new(&initial);
        let mut vent = metrics(8, 100.000_000_000_4, 100.000_000_000_4, 1);
        vent.relief_seam_open_cells = 1;
        vent.bottom_relief_seam_open_cells = 1;
        vent.steam_in_relief_seam_cells = 1;
        vent.outside_chamber_steam_cells = 1;
        relief_observations.confirm_opening(vent.identity(), vent.identity());
        relief_observations.observe(&vent, &initial_baseline);

        let mut sub_serialization_drop = metrics(9, 99.999_999_999_9, 99.999_999_999_9, 1);
        sub_serialization_drop.relief_seam_open_cells = 1;
        sub_serialization_drop.bottom_relief_seam_open_cells = 1;
        let hidden_update = relief_observations.observe(&sub_serialization_drop, &initial_baseline);
        assert_eq!(sub_serialization_drop.chamber_mean_pressure, 100.0);
        assert_eq!(sub_serialization_drop.chamber_max_pressure, 100.0);
        assert!(!hidden_update.first_post_opening_relief);

        let mut serialized_drop = metrics(10, 99.999_999_998_9, 99.999_999_998_9, 1);
        serialized_drop.relief_seam_open_cells = 1;
        serialized_drop.bottom_relief_seam_open_cells = 1;
        let visible_update = relief_observations.observe(&serialized_drop, &initial_baseline);
        assert_eq!(serialized_drop.chamber_mean_pressure, 99.999_999_999);
        assert_eq!(serialized_drop.chamber_max_pressure, 99.999_999_999);
        assert!(visible_update.first_post_opening_relief);

        let mean_boundary = |end: f64| {
            (0..64)
                .map(|index| {
                    let fraction = index as f64 / 63.0;
                    metrics(index, 10.0 + (end - 10.0) * fraction, 100.0, 1)
                })
                .collect::<VecDeque<_>>()
        };
        assert!(!terminal_trend(&mean_boundary(12.000_000_000_4)).mean_unbounded_growth);
        assert!(terminal_trend(&mean_boundary(12.000_000_000_6)).mean_unbounded_growth);

        let max_boundary = |end: f64| {
            (0..64)
                .map(|index| {
                    let fraction = index as f64 / 63.0;
                    metrics(index, 100.0, 10.0 + (end - 10.0) * fraction, 1)
                })
                .collect::<VecDeque<_>>()
        };
        assert!(!terminal_trend(&max_boundary(12.000_000_000_4)).max_unbounded_growth);
        assert!(terminal_trend(&max_boundary(12.000_000_000_6)).max_unbounded_growth);
    }

    #[test]
    fn pressure_vent_requires_confirmed_seam_transit_and_relief_after_vent() {
        let initial = metrics(0, 180.0, 200.0, 0);
        let baseline = baseline(&initial);
        let mut observations = PressureObservations::new(&initial);

        let mut premature = metrics(8, 175.0, 195.0, 10);
        premature.relief_seam_open_cells = 1;
        premature.bottom_relief_seam_open_cells = 1;
        premature.steam_in_relief_seam_cells = 1;
        premature.outside_chamber_steam_cells = 1;
        observations.observe(&premature, &baseline);
        assert!(observations.first_steam_in_relief_seam.is_none());
        assert!(observations.first_exterior_steam.is_none());

        observations.confirm_opening(premature.identity(), premature.identity());
        let mut outside_without_transit = metrics(16, 170.0, 190.0, 10);
        outside_without_transit.relief_seam_open_cells = 1;
        outside_without_transit.bottom_relief_seam_open_cells = 1;
        outside_without_transit.outside_chamber_steam_cells = 1;
        observations.observe(&outside_without_transit, &baseline);
        assert!(observations.first_exterior_steam.is_none());

        let mut in_seam = metrics(24, 165.0, 185.0, 10);
        in_seam.relief_seam_open_cells = 1;
        in_seam.bottom_relief_seam_open_cells = 1;
        in_seam.steam_in_relief_seam_cells = 1;
        observations.observe(&in_seam, &baseline);
        assert_eq!(
            observations.first_steam_in_relief_seam,
            Some(in_seam.identity())
        );
        assert!(observations.first_exterior_steam.is_none());

        let mut vent = metrics(32, 160.0, 180.0, 10);
        vent.relief_seam_open_cells = 1;
        vent.bottom_relief_seam_open_cells = 1;
        vent.outside_chamber_steam_cells = 1;
        observations.observe(&vent, &baseline);
        assert_eq!(observations.first_exterior_steam, Some(vent.identity()));
        assert!(observations.first_post_opening_relief.is_none());

        let mut relief = metrics(40, 159.0, 179.0, 10);
        relief.relief_seam_open_cells = 1;
        relief.bottom_relief_seam_open_cells = 1;
        observations.observe(&relief, &baseline);
        assert_eq!(
            observations.first_post_opening_relief,
            Some(relief.identity())
        );
        let terminal = (0..64)
            .map(|index| {
                metrics(
                    100 + index,
                    150.0 - index as f64 * 0.1,
                    170.0 - index as f64 * 0.1,
                    10,
                )
            })
            .collect::<VecDeque<_>>();
        let predicates = build_predicates(&observations, &terminal_trend(&terminal), 64, true);
        assert_eq!(
            predicates.exterior_vent_observed.status,
            PredicateStatus::Pass
        );
        assert_eq!(
            predicates.post_opening_pressure_relieved.status,
            PredicateStatus::Pass
        );
    }

    #[test]
    fn pressure_post_confirmation_reseal_is_a_hard_failure() {
        let initial = metrics(0, 180.0, 200.0, 0);
        let baseline = baseline(&initial);
        let mut observations = PressureObservations::new(&initial);
        let mut open = metrics(8, 170.0, 190.0, 10);
        open.relief_seam_wood_cells = 575;
        open.bottom_relief_seam_wood_cells = 191;
        open.relief_seam_open_cells = 1;
        open.bottom_relief_seam_open_cells = 1;
        observations.confirm_opening(open.identity(), open.identity());
        observations.observe(&open, &baseline);

        let resealed = metrics(16, 165.0, 185.0, 10);
        observations.observe(&resealed, &baseline);
        assert_eq!(
            observations.first_post_confirmation_reseal,
            Some(resealed.identity())
        );
        let terminal = (0..64)
            .map(|index| metrics(100 + index, 160.0, 180.0, 10))
            .collect::<VecDeque<_>>();
        let predicates = build_predicates(&observations, &terminal_trend(&terminal), 64, true);
        assert_eq!(
            predicates.persistent_opening_created.status,
            PredicateStatus::Fail
        );
    }

    #[test]
    fn pressure_predicates_allow_either_seam_but_flag_one_seam_for_review() {
        let initial = metrics(0, 180.0, 180.0, 0);
        let baseline = baseline(&initial);
        let mut observations = PressureObservations::new(&initial);
        let mut sample = metrics(8, 170.0, 170.0, 10);
        sample.relief_seam_wood_cells = 575;
        sample.top_relief_seam_wood_cells = 383;
        sample.relief_seam_open_cells = 1;
        sample.top_relief_seam_open_cells = 1;
        sample.outside_chamber_steam_cells = 1;
        observations.observe(&sample, &baseline);
        observations.confirm_opening(sample.identity(), sample.identity());
        observations.first_exterior_steam = Some(sample.identity());
        observations.latest = sample.clone();
        let window = (0..64)
            .map(|index| metrics(100 + index, 160.0 - index as f64 * 0.1, 170.0, 10))
            .collect::<VecDeque<_>>();
        let trend = terminal_trend(&window);
        let predicates = build_predicates(&observations, &trend, 64, true);
        assert_eq!(predicates.relief_seam_damaged.status, PredicateStatus::Pass);
        let flags = review_flags(&baseline, &observations, &trend);
        assert!(flags.only_one_relief_seam_ruptured);
        assert_eq!(
            pressure_verdict(&predicates, &flags),
            ExperimentVerdict::NeedsHumanReview
        );
    }

    #[test]
    fn pressure_terminal_activity_is_review_only_and_runnable_alone_does_not_trigger_it() {
        let initial = metrics(0, 180.0, 180.0, 0);
        let baseline = baseline(&initial);
        let mut observations = PressureObservations::new(&initial);
        observations.latest = metrics(100, 0.0, 0.0, 1);
        let terminal = (0..64)
            .map(|index| metrics(100 + index, 0.0, 0.0, 1))
            .collect::<VecDeque<_>>();
        let flags = review_flags(&baseline, &observations, &terminal_trend(&terminal));
        assert!(flags.terminal_activity_remains);
        assert_eq!(flags.reasons, vec!["terminal_activity_remains"]);

        let pass = || PredicateResult::pass("test");
        let predicates = PressurePredicates {
            pressure_activity_observed: pass(),
            relief_seam_damaged: pass(),
            persistent_opening_created: pass(),
            exterior_vent_observed: pass(),
            post_opening_pressure_relieved: pass(),
            terminal_pressure_not_runaway: pass(),
            no_invalid_materials: pass(),
            no_nonfinite_fields: pass(),
            exact_reset: pass(),
        };
        assert_eq!(predicates.statuses(), [PredicateStatus::Pass; 9]);
        assert_eq!(
            pressure_verdict(&predicates, &flags),
            ExperimentVerdict::NeedsHumanReview
        );

        observations.latest = metrics(200, 0.0, 0.0, 0);
        observations.latest.runnable_chunks = 1;
        let runnable_only = review_flags(&baseline, &observations, &terminal_trend(&terminal));
        assert!(!runnable_only.terminal_activity_remains);
        assert!(!runnable_only.reasons.contains(&"terminal_activity_remains"));
    }

    #[test]
    fn pressure_exact_reset_compares_full_gpu_state() {
        let world = WorldConfig::new(256, 256, 64).expect("world");
        let original = snapshot(world);
        assert!(exact_reset_equal(&original, &original.clone()));
        let mut changed = original.clone();
        changed.pressure_next[0] = 1.0f32.to_bits();
        assert!(!exact_reset_equal(&original, &changed));
    }

    fn frame(tick: u64, sequence: u64, hash: &str, kind: &'static str) -> PressureFrame {
        PressureFrame {
            badges: vec![FrameBadge {
                kind,
                reason: "test",
            }],
            sim_tick: tick,
            sample_sequence: sequence,
            state_hash: hash.to_string(),
            frame: RawFrame {
                width: 1,
                height: 1,
                rgba: vec![0, 0, 0, 255],
            },
        }
    }

    #[test]
    fn pressure_frames_fold_same_tick_hash_order_badges_and_keep_reset_last() {
        let frames = fold_and_order_frames(vec![
            frame(0, 9, "baseline", "reset"),
            frame(8, 3, "same", "first-rupture"),
            frame(1, 1, "tick1", "tick1"),
            frame(8, 2, "same", "first-wood-damage"),
            frame(0, 0, "baseline", "tick0"),
        ]);
        assert_eq!(frames.len(), 4);
        assert_eq!(frames[0].badges[0].kind, "tick0");
        assert_eq!(frames[1].badges[0].kind, "tick1");
        assert_eq!(
            frames[2]
                .badges
                .iter()
                .map(|badge| badge.kind)
                .collect::<Vec<_>>(),
            vec!["first-wood-damage", "first-rupture"]
        );
        assert_eq!(frames.last().expect("reset").badges[0].kind, "reset");
        assert!(badge_rank("persistent-opening") < badge_rank("opening-reseal"));
        assert!(badge_rank("opening-reseal") < badge_rank("first-exterior-steam"));
    }
}
