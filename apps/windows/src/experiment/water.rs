//! Water Flow Experiment Evidence Harness worker.
//!
//! Sand Fall keeps its v0 lifecycle and serializers in the parent module. This
//! module reuses only the scenario-neutral GPU snapshot, renderer capture,
//! create-new filesystem, frame publication, and reset-comparison helpers.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use powdergame_core::{
    is_valid_cell_material_value, WorldConfig, ACTIVITY_MATTER, ACTIVITY_PRESSURE,
    ACTIVITY_REACTION, ACTIVITY_THERMAL, CHUNK_STATE_RUNNABLE, CHUNK_STATE_SLEEPING,
    MATERIAL_EMPTY, MATERIAL_OIL, MATERIAL_WATER,
};
use powdergame_gpu::Simulation;
use powdergame_scenarios::{reset_and_stage_scenario, ScenarioId};

use crate::gallery::RuntimeProvenance;
use crate::renderer::Renderer;

use super::{
    authoritative_current_hash, bit_count, capture_gpu_snapshot, create_new_file,
    create_worker_directory, display_path, exact_reset_equal, is_safe_identifier, json_escape,
    json_opt_u32, json_opt_u64, physical_tick_boundary_equal, take_sequence, write_new,
    write_raw_frames, AllSleepDetector, ExperimentOutcome, ExperimentVerdict, GpuSnapshot,
    PredicateResult, PredicateStatus, RawFrame, SemanticFrame, WrittenFrame,
    REQUIRED_ALL_SLEEP_SAMPLES, REQUIRED_WORLD,
};

pub const WATER_EXPERIMENT_ID: &str = "g8b-water-flow-v0";
const WATER_TELEMETRY_SCHEMA_VERSION: &str = "powdergame-experiment-telemetry-v1";
const WATER_ANALYSIS_SCHEMA_VERSION: &str = "powdergame-experiment-analysis-v1";
const WATER_FRAMES_SCHEMA_VERSION: &str = "powdergame-experiment-frames-v0";
const REQUIRED_STABLE_PLATEAU_SAMPLES: u32 = 8;
const REQUIRED_MAX_TICKS: u64 = 20_000;
const REQUIRED_DIAGNOSTIC_INTERVAL_TICKS: u64 = 8;
const REQUIRED_POST_SETTLE_TICKS: u32 = 180;
const WATER_MIN_RAW_FRAMES: usize = 8;
const WATER_MAX_RAW_FRAMES: usize = 12;
const DIAGNOSTIC_RING_CAPACITY: usize = 12;
const BOTTOM_CHUNK_ROW: u32 = 3;
const DESTINATION_MIN_X: u32 = 18;
const DESTINATION_MAX_X_EXCLUSIVE: u32 = 238;
const DESTINATION_MIN_Y: u32 = 200;
const DESTINATION_MAX_Y_EXCLUSIVE: u32 = 230;

#[derive(Clone, Debug)]
struct WaterBaseline {
    initial_water_mask: Vec<bool>,
    destination_empty_mask: Vec<bool>,
    matter_count: u64,
    water_count: u64,
    oil_count: u64,
    water_y_sum: u64,
    oil_y_sum: u64,
    water_occupied_chunks: u32,
    oil_occupied_chunks: u32,
    bottom_chunk_row_water_cells: u64,
    destination_water_cells: u64,
    destination_spread_x: u32,
}

#[derive(Clone, Debug)]
struct WaterSampleMetrics {
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
    oil_count: u64,
    water_y_sum: u64,
    water_min_y: Option<u32>,
    water_max_y: Option<u32>,
    oil_y_sum: u64,
    oil_min_y: Option<u32>,
    oil_max_y: Option<u32>,
    water_occupied_chunks: u32,
    oil_occupied_chunks: u32,
    water_outside_initial_mask: u64,
    initial_water_cells_vacated: u64,
    bottom_chunk_row_water_cells: u64,
    destination_water_cells: u64,
    destination_spread_x: u32,
    invalid_material_count: u64,
    nonfinite_temperature_count: u64,
    nonfinite_pressure_count: u64,
    changed_chunks: u32,
    wake_chunks: u32,
    wake_reason_or: u32,
    state_hash: String,
    physical_state_hash: String,
}

impl WaterSampleMetrics {
    fn all_sleep(&self) -> bool {
        super::all_sleep_counts(
            self.any_active_cells,
            self.active_chunks,
            self.runnable_chunks,
            self.sleeping_chunks,
            self.total_chunks,
        )
    }

    fn movement_observed(&self) -> bool {
        self.water_outside_initial_mask != 0 && self.initial_water_cells_vacated != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TerminalReason {
    AllSleep,
    StablePlateau,
    MaxTicks,
}

impl TerminalReason {
    const fn as_str(self) -> &'static str {
        match self {
            Self::AllSleep => "all-sleep",
            Self::StablePlateau => "stable-plateau",
            Self::MaxTicks => "max-ticks",
        }
    }

    const fn has_post_settle_window(self) -> bool {
        !matches!(self, Self::MaxTicks)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct PlateauUpdate {
    first_in_streak: bool,
    confirmed: bool,
    streak_broken: bool,
}

#[derive(Clone, Debug)]
struct StablePlateauDetector {
    required: u32,
    streak: u32,
    state_hash: Option<String>,
    first_sim_tick: Option<u64>,
    first_sample_sequence: Option<u64>,
}

impl StablePlateauDetector {
    fn new(required: u32) -> Self {
        Self {
            required,
            streak: 0,
            state_hash: None,
            first_sim_tick: None,
            first_sample_sequence: None,
        }
    }

    fn observe(&mut self, metrics: &WaterSampleMetrics) -> PlateauUpdate {
        if metrics.changed_chunks != 0 || metrics.wake_chunks != 0 {
            return self.break_streak();
        }

        let same_hash = self.state_hash.as_deref() == Some(metrics.state_hash.as_str());
        let streak_broken = self.streak != 0 && !same_hash;
        if streak_broken {
            let _ = self.break_streak();
        }
        let first_in_streak = self.streak == 0;
        if first_in_streak {
            self.state_hash = Some(metrics.state_hash.clone());
            self.first_sim_tick = Some(metrics.sim_tick);
            self.first_sample_sequence = Some(metrics.sample_sequence);
        }
        self.streak = self.streak.saturating_add(1);
        PlateauUpdate {
            first_in_streak,
            confirmed: self.streak >= self.required,
            streak_broken,
        }
    }

    fn break_streak(&mut self) -> PlateauUpdate {
        let streak_broken = self.streak != 0;
        self.streak = 0;
        self.state_hash = None;
        self.first_sim_tick = None;
        self.first_sample_sequence = None;
        PlateauUpdate {
            streak_broken,
            ..PlateauUpdate::default()
        }
    }
}

#[derive(Clone, Debug)]
struct WaterPredicates {
    actual_water_movement: PredicateResult,
    cross_chunk_flow: PredicateResult,
    destination_arrival: PredicateResult,
    water_conservation: PredicateResult,
    no_invalid_materials: PredicateResult,
    no_nonfinite_fields: PredicateResult,
    stable_bulk_before_max: PredicateResult,
    post_settle_stable: PredicateResult,
    exact_reset: PredicateResult,
}

impl WaterPredicates {
    fn statuses(&self) -> [PredicateStatus; 9] {
        [
            self.actual_water_movement.status,
            self.cross_chunk_flow.status,
            self.destination_arrival.status,
            self.water_conservation.status,
            self.no_invalid_materials.status,
            self.no_nonfinite_fields.status,
            self.stable_bulk_before_max.status,
            self.post_settle_stable.status,
            self.exact_reset.status,
        ]
    }

    fn verdict(&self) -> ExperimentVerdict {
        water_verdict_from_statuses(&self.statuses())
    }
}

fn water_verdict_from_statuses(statuses: &[PredicateStatus]) -> ExperimentVerdict {
    if statuses.contains(&PredicateStatus::Fail) {
        ExperimentVerdict::Fail
    } else if statuses.contains(&PredicateStatus::Unknown) {
        ExperimentVerdict::NeedsHumanReview
    } else {
        ExperimentVerdict::Pass
    }
}

struct WaterJsonlWriters {
    samples: BufWriter<File>,
    events: BufWriter<File>,
    event_sequence: u64,
}

impl WaterJsonlWriters {
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
        metrics: &WaterSampleMetrics,
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
                "\"run_id\":\"{}\",\"scenario\":\"water-flow\",",
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
                "\"water_count\":{},\"oil_count\":{},",
                "\"water_y_sum\":{},\"water_min_y\":{},\"water_max_y\":{},",
                "\"oil_y_sum\":{},\"oil_min_y\":{},\"oil_max_y\":{},",
                "\"water_occupied_chunks\":{},\"oil_occupied_chunks\":{},",
                "\"water_outside_initial_mask\":{},",
                "\"initial_water_cells_vacated\":{},",
                "\"bottom_chunk_row_water_cells\":{},",
                "\"destination_water_cells\":{},\"destination_spread_x\":{},",
                "\"invalid_material_count\":{},",
                "\"nonfinite_temperature_count\":{},",
                "\"nonfinite_pressure_count\":{},\"changed_chunks\":{},",
                "\"wake_chunks\":{},\"wake_reason_or\":{},",
                "\"state_hash\":\"{}\",\"physical_state_hash\":\"{}\"}}"
            ),
            WATER_TELEMETRY_SCHEMA_VERSION,
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
            metrics.oil_count,
            metrics.water_y_sum,
            json_opt_u32(metrics.water_min_y),
            json_opt_u32(metrics.water_max_y),
            metrics.oil_y_sum,
            json_opt_u32(metrics.oil_min_y),
            json_opt_u32(metrics.oil_max_y),
            metrics.water_occupied_chunks,
            metrics.oil_occupied_chunks,
            metrics.water_outside_initial_mask,
            metrics.initial_water_cells_vacated,
            metrics.bottom_chunk_row_water_cells,
            metrics.destination_water_cells,
            metrics.destination_spread_x,
            metrics.invalid_material_count,
            metrics.nonfinite_temperature_count,
            metrics.nonfinite_pressure_count,
            metrics.changed_chunks,
            metrics.wake_chunks,
            metrics.wake_reason_or,
            metrics.state_hash,
            metrics.physical_state_hash,
        )
        .map_err(|error| format!("write {} failed: {error}", display_path(&config.run_dir)))
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
                "\"run_id\":\"{}\",\"scenario\":\"water-flow\",",
                "\"event_sequence\":{},\"event\":\"{}\",",
                "\"sim_tick\":{},\"sample_sequence\":{},\"detail\":\"{}\"}}"
            ),
            WATER_TELEMETRY_SCHEMA_VERSION,
            json_escape(&config.experiment_id),
            json_escape(&config.run_id),
            self.event_sequence,
            json_escape(event),
            sim_tick,
            json_opt_u64(sample_sequence),
            json_escape(detail),
        )
        .map_err(|error| format!("write events JSONL failed: {error}"))?;
        self.event_sequence = self.event_sequence.saturating_add(1);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        self.samples
            .flush()
            .map_err(|error| format!("flush samples JSONL failed: {error}"))?;
        self.events
            .flush()
            .map_err(|error| format!("flush events JSONL failed: {error}"))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SampleIdentity {
    sim_tick: u64,
    sample_sequence: u64,
}

impl SampleIdentity {
    fn from_metrics(metrics: &WaterSampleMetrics) -> Self {
        Self {
            sim_tick: metrics.sim_tick,
            sample_sequence: metrics.sample_sequence,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ObservationUpdate {
    first_movement: bool,
    first_cross: bool,
    first_destination: bool,
    new_peak: bool,
    new_max_spread: bool,
    first_sleeping: bool,
}

#[derive(Clone, Debug)]
struct WaterObservations {
    counts_conserved: bool,
    invalid_material_total: u64,
    nonfinite_field_total: u64,
    first_movement: Option<SampleIdentity>,
    first_cross: Option<SampleIdentity>,
    first_destination: Option<SampleIdentity>,
    first_sleeping: Option<SampleIdentity>,
    peak_active_cells: u64,
    peak_active_chunks: u32,
    peak_active: SampleIdentity,
    max_bottom_chunk_row_water_cells: u64,
    max_destination_water_cells: u64,
    max_destination_spread_x: u32,
    max_destination_spread: Option<SampleIdentity>,
    latest: WaterSampleMetrics,
}

impl WaterObservations {
    fn new(tick0: &WaterSampleMetrics) -> Self {
        Self {
            counts_conserved: true,
            invalid_material_total: tick0.invalid_material_count,
            nonfinite_field_total: tick0
                .nonfinite_temperature_count
                .saturating_add(tick0.nonfinite_pressure_count),
            first_movement: None,
            first_cross: None,
            first_destination: None,
            first_sleeping: (tick0.sleeping_chunks != 0)
                .then(|| SampleIdentity::from_metrics(tick0)),
            peak_active_cells: tick0.any_active_cells,
            peak_active_chunks: tick0.active_chunks,
            peak_active: SampleIdentity::from_metrics(tick0),
            max_bottom_chunk_row_water_cells: tick0.bottom_chunk_row_water_cells,
            max_destination_water_cells: tick0.destination_water_cells,
            max_destination_spread_x: tick0.destination_spread_x,
            max_destination_spread: (tick0.destination_spread_x != 0)
                .then(|| SampleIdentity::from_metrics(tick0)),
            latest: tick0.clone(),
        }
    }

    fn observe(
        &mut self,
        metrics: &WaterSampleMetrics,
        baseline: &WaterBaseline,
        allow_milestones: bool,
    ) -> ObservationUpdate {
        self.counts_conserved &= metrics.matter_count == baseline.matter_count
            && metrics.water_count == baseline.water_count
            && metrics.oil_count == baseline.oil_count;
        self.invalid_material_total = self
            .invalid_material_total
            .saturating_add(metrics.invalid_material_count);
        self.nonfinite_field_total = self
            .nonfinite_field_total
            .saturating_add(metrics.nonfinite_temperature_count)
            .saturating_add(metrics.nonfinite_pressure_count);

        let identity = SampleIdentity::from_metrics(metrics);
        let first_movement =
            allow_milestones && metrics.movement_observed() && self.first_movement.is_none();
        if first_movement {
            self.first_movement = Some(identity);
        }
        let first_cross = allow_milestones
            && metrics.bottom_chunk_row_water_cells != 0
            && self.first_cross.is_none();
        if first_cross {
            self.first_cross = Some(identity);
        }
        let first_destination = allow_milestones
            && metrics.destination_water_cells != 0
            && self.first_destination.is_none();
        if first_destination {
            self.first_destination = Some(identity);
        }
        let first_sleeping = metrics.sleeping_chunks != 0 && self.first_sleeping.is_none();
        if first_sleeping {
            self.first_sleeping = Some(identity);
        }
        let new_peak = allow_milestones && metrics.any_active_cells > self.peak_active_cells;
        if new_peak {
            self.peak_active_cells = metrics.any_active_cells;
            self.peak_active = identity;
        }
        if allow_milestones {
            self.peak_active_chunks = self.peak_active_chunks.max(metrics.active_chunks);
        }
        if allow_milestones {
            self.max_bottom_chunk_row_water_cells = self
                .max_bottom_chunk_row_water_cells
                .max(metrics.bottom_chunk_row_water_cells);
            self.max_destination_water_cells = self
                .max_destination_water_cells
                .max(metrics.destination_water_cells);
        }
        let new_max_spread =
            allow_milestones && metrics.destination_spread_x > self.max_destination_spread_x;
        if new_max_spread {
            self.max_destination_spread_x = metrics.destination_spread_x;
            self.max_destination_spread = Some(identity);
        }
        self.latest = metrics.clone();

        ObservationUpdate {
            first_movement,
            first_cross,
            first_destination,
            new_peak,
            new_max_spread,
            first_sleeping,
        }
    }
}

fn baseline_from_tick0(
    snapshot: &GpuSnapshot,
    metrics: &WaterSampleMetrics,
    world: WorldConfig,
) -> WaterBaseline {
    WaterBaseline {
        initial_water_mask: snapshot
            .material_current
            .iter()
            .map(|&material| material == MATERIAL_WATER)
            .collect(),
        destination_empty_mask: snapshot
            .material_current
            .iter()
            .enumerate()
            .map(|(index, &material)| {
                let index = index as u64;
                let x = (index % u64::from(world.width)) as u32;
                let y = (index / u64::from(world.width)) as u32;
                material == MATERIAL_EMPTY
                    && (DESTINATION_MIN_X..DESTINATION_MAX_X_EXCLUSIVE).contains(&x)
                    && (DESTINATION_MIN_Y..DESTINATION_MAX_Y_EXCLUSIVE).contains(&y)
            })
            .collect(),
        matter_count: metrics.matter_count,
        water_count: metrics.water_count,
        oil_count: metrics.oil_count,
        water_y_sum: metrics.water_y_sum,
        oil_y_sum: metrics.oil_y_sum,
        water_occupied_chunks: metrics.water_occupied_chunks,
        oil_occupied_chunks: metrics.oil_occupied_chunks,
        bottom_chunk_row_water_cells: metrics.bottom_chunk_row_water_cells,
        destination_water_cells: metrics.destination_water_cells,
        destination_spread_x: metrics.destination_spread_x,
    }
}

fn water_metrics_from_snapshot(
    snapshot: &GpuSnapshot,
    world: WorldConfig,
    baseline: Option<&WaterBaseline>,
    sample_sequence: u64,
    sim_tick: u64,
    phase: &'static str,
    reason: &'static str,
) -> Result<WaterSampleMetrics, String> {
    let expected_cells = u64::from(world.width) * u64::from(world.height);
    if snapshot.material_current.len() as u64 != expected_cells
        || snapshot.temperature_current.len() as u64 != expected_cells
        || snapshot.pressure_current.len() as u64 != expected_cells
        || snapshot.flags_current.len() as u64 != expected_cells
        || snapshot.cell_activity.len() as u64 != expected_cells
    {
        return Err("GPU snapshot cell-vector lengths do not match WorldConfig".to_string());
    }
    if snapshot.chunk_activity.len() != snapshot.chunk_state.len()
        || snapshot.chunk_activity.len() != snapshot.chunk_changed.len()
        || snapshot.chunk_activity.len() != snapshot.chunk_wake_reason.len()
    {
        return Err("GPU snapshot chunk-vector lengths disagree".to_string());
    }
    if baseline.is_some_and(|value| {
        value.initial_water_mask.len() != expected_cells as usize
            || value.destination_empty_mask.len() != expected_cells as usize
    }) {
        return Err("Water baseline mask length does not match WorldConfig".to_string());
    }

    let chunks_x = world.width / world.chunk_size;
    let expected_chunks =
        usize::try_from(u64::from(chunks_x) * u64::from(world.height / world.chunk_size))
            .map_err(|_| "chunk count exceeds usize".to_string())?;
    if snapshot.chunk_activity.len() != expected_chunks {
        return Err("GPU snapshot chunk count does not match WorldConfig".to_string());
    }

    let mut material_counts_by_id = [0u64; 10];
    let mut matter_count = 0u64;
    let mut water_count = 0u64;
    let mut oil_count = 0u64;
    let mut water_y_sum = 0u64;
    let mut water_min_y = None;
    let mut water_max_y = None;
    let mut oil_y_sum = 0u64;
    let mut oil_min_y = None;
    let mut oil_max_y = None;
    let mut water_chunks = vec![false; expected_chunks];
    let mut oil_chunks = vec![false; expected_chunks];
    let mut water_outside_initial_mask = 0u64;
    let mut bottom_chunk_row_water_cells = 0u64;
    let mut destination_water_cells = 0u64;
    let mut destination_min_x = None;
    let mut destination_max_x = None;
    let mut invalid_material_count = 0u64;

    for (index, &material) in snapshot.material_current.iter().enumerate() {
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

        let index_u64 = index as u64;
        let x = (index_u64 % u64::from(world.width)) as u32;
        let y = (index_u64 / u64::from(world.width)) as u32;
        let chunk_index = usize::try_from(
            u64::from(y / world.chunk_size) * u64::from(chunks_x) + u64::from(x / world.chunk_size),
        )
        .map_err(|_| "chunk index exceeds usize".to_string())?;

        if material == MATERIAL_WATER {
            water_count = water_count.saturating_add(1);
            water_y_sum = water_y_sum.saturating_add(u64::from(y));
            water_min_y = Some(water_min_y.map_or(y, |old: u32| old.min(y)));
            water_max_y = Some(water_max_y.map_or(y, |old: u32| old.max(y)));
            water_chunks[chunk_index] = true;
            if baseline.is_some_and(|value| !value.initial_water_mask[index]) {
                water_outside_initial_mask = water_outside_initial_mask.saturating_add(1);
            }
            if y / world.chunk_size == BOTTOM_CHUNK_ROW {
                bottom_chunk_row_water_cells = bottom_chunk_row_water_cells.saturating_add(1);
            }
            let is_destination_cell = baseline.map_or_else(
                || {
                    (DESTINATION_MIN_X..DESTINATION_MAX_X_EXCLUSIVE).contains(&x)
                        && (DESTINATION_MIN_Y..DESTINATION_MAX_Y_EXCLUSIVE).contains(&y)
                },
                |value| value.destination_empty_mask[index],
            );
            if is_destination_cell {
                destination_water_cells = destination_water_cells.saturating_add(1);
                destination_min_x = Some(destination_min_x.map_or(x, |old: u32| old.min(x)));
                destination_max_x = Some(destination_max_x.map_or(x, |old: u32| old.max(x)));
            }
        } else if material == MATERIAL_OIL {
            oil_count = oil_count.saturating_add(1);
            oil_y_sum = oil_y_sum.saturating_add(u64::from(y));
            oil_min_y = Some(oil_min_y.map_or(y, |old: u32| old.min(y)));
            oil_max_y = Some(oil_max_y.map_or(y, |old: u32| old.max(y)));
            oil_chunks[chunk_index] = true;
        }
    }

    let initial_water_cells_vacated = baseline.map_or(0, |value| {
        value
            .initial_water_mask
            .iter()
            .zip(&snapshot.material_current)
            .filter(|(initial_water, material)| **initial_water && **material != MATERIAL_WATER)
            .count() as u64
    });
    let destination_spread_x = match (destination_min_x, destination_max_x) {
        (Some(minimum), Some(maximum)) => maximum.saturating_sub(minimum).saturating_add(1),
        _ => 0,
    };
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
    let any_active_cells = snapshot
        .cell_activity
        .iter()
        .filter(|&&value| value != 0)
        .count() as u64;
    let active_chunks = snapshot
        .chunk_activity
        .iter()
        .filter(|&&value| value != 0)
        .count() as u32;
    let runnable_chunks = snapshot
        .chunk_state
        .iter()
        .filter(|&&value| value == CHUNK_STATE_RUNNABLE)
        .count() as u32;
    let sleeping_chunks = snapshot
        .chunk_state
        .iter()
        .filter(|&&value| value == CHUNK_STATE_SLEEPING)
        .count() as u32;
    let changed_chunks = snapshot
        .chunk_changed
        .iter()
        .filter(|&&value| value != 0)
        .count() as u32;
    let wake_chunks = snapshot
        .chunk_wake_reason
        .iter()
        .filter(|&&value| value != 0)
        .count() as u32;

    Ok(WaterSampleMetrics {
        sample_sequence,
        sim_tick,
        phase,
        reason,
        total_cells: expected_cells,
        any_active_cells,
        matter_active_cells: bit_count(&snapshot.cell_activity, ACTIVITY_MATTER),
        thermal_active_cells: bit_count(&snapshot.cell_activity, ACTIVITY_THERMAL),
        pressure_active_cells: bit_count(&snapshot.cell_activity, ACTIVITY_PRESSURE),
        reaction_active_cells: bit_count(&snapshot.cell_activity, ACTIVITY_REACTION),
        total_chunks: snapshot.chunk_activity.len() as u32,
        active_chunks,
        runnable_chunks,
        sleeping_chunks,
        material_counts_by_id,
        matter_count,
        water_count,
        oil_count,
        water_y_sum,
        water_min_y,
        water_max_y,
        oil_y_sum,
        oil_min_y,
        oil_max_y,
        water_occupied_chunks: water_chunks.iter().filter(|&&occupied| occupied).count() as u32,
        oil_occupied_chunks: oil_chunks.iter().filter(|&&occupied| occupied).count() as u32,
        water_outside_initial_mask,
        initial_water_cells_vacated,
        bottom_chunk_row_water_cells,
        destination_water_cells,
        destination_spread_x,
        invalid_material_count,
        nonfinite_temperature_count,
        nonfinite_pressure_count,
        changed_chunks,
        wake_chunks,
        wake_reason_or: snapshot
            .chunk_wake_reason
            .iter()
            .copied()
            .fold(0u32, |acc, value| acc | value),
        state_hash: authoritative_current_hash(snapshot),
        physical_state_hash: physical_tick_boundary_hash(snapshot),
    })
}

fn physical_tick_boundary_hash(snapshot: &GpuSnapshot) -> String {
    let mut hash = super::Fnv1a64::new();
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

fn capture_water_frame(
    renderer: &mut Renderer,
    kind: &'static str,
    reason: &'static str,
    metrics: &WaterSampleMetrics,
) -> Result<SemanticFrame, String> {
    let captured = renderer
        .capture_full_frame(None)
        .map_err(|error| format!("capture {kind} frame failed: {error}"))?;
    Ok(SemanticFrame {
        kind,
        reason,
        sim_tick: metrics.sim_tick,
        sample_sequence: metrics.sample_sequence,
        state_hash: metrics.state_hash.clone(),
        frame: RawFrame::try_from(captured)?,
    })
}

fn remember_diagnostic(ring: &mut VecDeque<SemanticFrame>, frame: &SemanticFrame) {
    if ring.iter().any(|existing| {
        existing.sim_tick == frame.sim_tick && existing.sample_sequence == frame.sample_sequence
    }) {
        return;
    }
    if ring.len() == DIAGNOSTIC_RING_CAPACITY {
        let _ = ring.pop_front();
    }
    ring.push_back(frame.clone_with_kind("diagnostic-observation", "minimum-evidence-observation"));
}

fn same_identity(left: &SemanticFrame, right: &SemanticFrame) -> bool {
    left.sim_tick == right.sim_tick && left.sample_sequence == right.sample_sequence
}

fn push_required_frame(frames: &mut Vec<SemanticFrame>, frame: SemanticFrame) {
    if let Some(existing) = frames
        .iter_mut()
        .find(|existing| same_identity(existing, &frame))
    {
        *existing = frame;
    } else if frames.len() < WATER_MAX_RAW_FRAMES {
        frames.push(frame);
    }
}

fn push_optional_frame(frames: &mut Vec<SemanticFrame>, frame: Option<SemanticFrame>) {
    let Some(frame) = frame else {
        return;
    };
    if frames.len() >= WATER_MAX_RAW_FRAMES
        || frames
            .iter()
            .any(|existing| same_identity(existing, &frame))
        || frames
            .iter()
            .any(|existing| existing.frame.rgba == frame.frame.rgba)
    {
        return;
    }
    frames.push(frame);
}

fn push_named_alias_frame(frames: &mut Vec<SemanticFrame>, frame: Option<SemanticFrame>) {
    let Some(frame) = frame else {
        return;
    };
    if frames.len() >= WATER_MAX_RAW_FRAMES
        || frames.iter().any(|existing| existing.kind == frame.kind)
    {
        return;
    }
    // `peak-active` is the sole sanctioned semantic alias. Its identity is
    // independently bound to the analysis peak, so retaining it is honest
    // even when another named frame was captured from the same sample.
    frames.push(frame);
}

#[allow(clippy::too_many_arguments)]
fn assemble_semantic_frames(
    tick0: SemanticFrame,
    tick1: SemanticFrame,
    first_movement: Option<SemanticFrame>,
    peak_active: Option<SemanticFrame>,
    cross_chunk: Option<SemanticFrame>,
    destination: Option<SemanticFrame>,
    max_spread: Option<SemanticFrame>,
    first_sleeping: Option<SemanticFrame>,
    late: Option<SemanticFrame>,
    terminal: SemanticFrame,
    post_settle: Option<SemanticFrame>,
    reset: SemanticFrame,
    verdict: ExperimentVerdict,
    diagnostics: &VecDeque<SemanticFrame>,
) -> Result<Vec<SemanticFrame>, String> {
    let mut frames = Vec::with_capacity(WATER_MAX_RAW_FRAMES);
    push_required_frame(&mut frames, tick0);
    push_required_frame(&mut frames, tick1);
    push_optional_frame(&mut frames, first_movement);
    push_optional_frame(&mut frames, cross_chunk);
    push_optional_frame(&mut frames, destination);
    push_optional_frame(&mut frames, max_spread);
    push_optional_frame(&mut frames, first_sleeping);
    if let Some(late) = late {
        push_required_frame(&mut frames, late);
    }
    push_required_frame(&mut frames, terminal);
    if let Some(post_settle) = post_settle {
        push_required_frame(&mut frames, post_settle);
    }
    // Peak is a requested analysis-bound semantic role and the sole allowed
    // alias. Insert it after required frames so a late/terminal collision does
    // not silently replace the peak role.
    push_named_alias_frame(&mut frames, peak_active);

    let target_before_reset = WATER_MIN_RAW_FRAMES.saturating_sub(1);
    if verdict != ExperimentVerdict::Pass {
        for frame in diagnostics {
            if frames.len() >= target_before_reset {
                break;
            }
            if frames.iter().any(|existing| same_identity(existing, frame))
                || frames
                    .iter()
                    .any(|existing| existing.frame.rgba == frame.frame.rgba)
            {
                continue;
            }
            frames.push(frame.clone());
        }
        // A semantically broken/static run can legitimately render identical
        // pixels at distinct diagnostic samples. Keep those distinct
        // identities rather than inventing a missing movement milestone.
        for frame in diagnostics {
            if frames.len() >= target_before_reset {
                break;
            }
            if !frames.iter().any(|existing| same_identity(existing, frame)) {
                frames.push(frame.clone());
            }
        }
    }
    push_required_frame(&mut frames, reset);

    if !(WATER_MIN_RAW_FRAMES..=WATER_MAX_RAW_FRAMES).contains(&frames.len()) {
        return Err(format!(
            "completed Water lifecycle produced {} distinct semantic frames; required {WATER_MIN_RAW_FRAMES}..={WATER_MAX_RAW_FRAMES}",
            frames.len()
        ));
    }
    Ok(frames)
}

fn validate_water_worker_config(
    simulation: &Simulation,
    config: &super::ExperimentWorkerConfig,
) -> Result<(), String> {
    if config.experiment_id != WATER_EXPERIMENT_ID {
        return Err(format!(
            "Water experiment_id must be '{WATER_EXPERIMENT_ID}', got '{}'",
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
    if config.scenario != ScenarioId::WaterFlow {
        return Err(format!(
            "Water experiment v1 supports only WaterFlow, got {}",
            config.scenario
        ));
    }
    if simulation.world.config != REQUIRED_WORLD {
        return Err(format!(
            "Water experiment v1 requires WorldConfig 256x256x64, got {}x{}x{}",
            simulation.world.config.width,
            simulation.world.config.height,
            simulation.world.config.chunk_size
        ));
    }
    if !simulation.sleep_enabled {
        return Err("Water experiment v1 requires simulation sleep to be enabled".to_string());
    }
    if config.max_ticks != REQUIRED_MAX_TICKS {
        return Err(format!("Water max_ticks must be {REQUIRED_MAX_TICKS}"));
    }
    if config.diagnostic_interval_ticks != REQUIRED_DIAGNOSTIC_INTERVAL_TICKS {
        return Err(format!(
            "Water diagnostic_interval_ticks must be {REQUIRED_DIAGNOSTIC_INTERVAL_TICKS}"
        ));
    }
    if config.consecutive_all_sleep != REQUIRED_ALL_SLEEP_SAMPLES {
        return Err(format!(
            "consecutive_all_sleep must be {REQUIRED_ALL_SLEEP_SAMPLES}"
        ));
    }
    if config.post_sleep_ticks != REQUIRED_POST_SETTLE_TICKS {
        return Err(format!(
            "Water post_sleep_ticks must be {REQUIRED_POST_SETTLE_TICKS}"
        ));
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

#[allow(clippy::too_many_arguments)]
fn build_water_predicates(
    observations: &WaterObservations,
    terminal_reason: TerminalReason,
    first_all_sleep_tick: Option<u64>,
    confirmed_all_sleep_tick: Option<u64>,
    max_ticks: u64,
    post_settle_end_tick: Option<u64>,
    post_settle_change_ticks: u32,
    post_settle_wake_ticks: u32,
    exact_reset: bool,
) -> WaterPredicates {
    let actual_water_movement = match observations.first_movement {
        Some(identity) => PredicateResult::pass(format!(
            "Water left the tick-0 mask and vacated tick-0 Water cells at sim tick {} sample {}",
            identity.sim_tick, identity.sample_sequence
        )),
        None => PredicateResult::unknown(
            "no flowing sample showed both Water outside the tick-0 mask and a vacated tick-0 Water cell",
        ),
    };
    let cross_chunk_flow = match observations.first_cross {
        Some(identity) => PredicateResult::pass(format!(
            "Water entered bottom chunk row cy=3 at sim tick {} sample {}",
            identity.sim_tick, identity.sample_sequence
        )),
        None => PredicateResult::unknown("no flowing sample contained Water in chunk row cy=3"),
    };
    let destination_arrival = match observations.first_destination {
        Some(identity) => PredicateResult::pass(format!(
            "Water entered destination [18,238)x[200,230) at sim tick {} sample {}",
            identity.sim_tick, identity.sample_sequence
        )),
        None => PredicateResult::unknown(
            "no flowing sample contained Water in destination [18,238)x[200,230)",
        ),
    };
    let water_conservation = if observations.counts_conserved {
        PredicateResult::pass(
            "Matter, Water, and Oil counts matched tick 0 in every non-reset sample",
        )
    } else {
        PredicateResult::fail(
            "Matter, Water, or Oil count differed from tick 0 in a non-reset sample",
        )
    };
    let no_invalid_materials = if observations.invalid_material_total == 0 {
        PredicateResult::pass("invalid material count was zero in every non-reset sample")
    } else {
        PredicateResult::fail(format!(
            "sampled invalid material occurrences={}",
            observations.invalid_material_total
        ))
    };
    let no_nonfinite_fields = if observations.nonfinite_field_total == 0 {
        PredicateResult::pass(
            "non-finite temperature/pressure count was zero in every non-reset sample",
        )
    } else {
        PredicateResult::fail(format!(
            "sampled non-finite temperature/pressure occurrences={}",
            observations.nonfinite_field_total
        ))
    };
    let stable_bulk_before_max = match (
        terminal_reason,
        first_all_sleep_tick,
        confirmed_all_sleep_tick,
    ) {
        (TerminalReason::AllSleep, Some(first_tick), Some(confirmed_tick))
            if confirmed_tick < max_ticks =>
        {
            PredicateResult::pass(format!(
                "three-sample all-sleep confirmed at sim tick {confirmed_tick} before max {max_ticks}; streak began at {first_tick}"
            ))
        }
        (TerminalReason::AllSleep, Some(first_tick), Some(confirmed_tick)) => {
            PredicateResult::unknown(format!(
                "all-sleep confirmed at sim tick {confirmed_tick}, not before max {max_ticks}; streak began at {first_tick}"
            ))
        }
        (TerminalReason::AllSleep, _, _) => PredicateResult::unknown(
            "all-sleep terminal was selected without complete streak identities",
        ),
        (TerminalReason::StablePlateau, _, _) => PredicateResult::unknown(
            "authoritative state plateaued without three-sample all-sleep confirmation",
        ),
        (TerminalReason::MaxTicks, _, _) => PredicateResult::unknown(format!(
            "neither all-sleep nor an eight-diagnostic stable plateau was confirmed by max tick {max_ticks}"
        )),
    };
    let post_settle_stable = match post_settle_end_tick {
        Some(end_tick) if post_settle_change_ticks == 0 && post_settle_wake_ticks == 0 => {
            PredicateResult::pass(format!(
                "post-settle window ended at tick {end_tick} with zero changes and zero wakes"
            ))
        }
        Some(end_tick) => PredicateResult::fail(format!(
            "post-settle window ended at tick {end_tick}; change_ticks={post_settle_change_ticks}; wake_ticks={post_settle_wake_ticks}"
        )),
        None => PredicateResult::unknown(
            "post-settle stability was not measured because no all-sleep or plateau terminal candidate was confirmed",
        ),
    };
    let exact_reset = if exact_reset {
        PredicateResult::pass(
            "programmatic R-equivalent world, scratch, uniforms, tick and sleep settings matched tick 0",
        )
    } else {
        PredicateResult::fail(
            "programmatic R-equivalent world, scratch, uniforms, tick or sleep settings differed from tick 0",
        )
    };

    WaterPredicates {
        actual_water_movement,
        cross_chunk_flow,
        destination_arrival,
        water_conservation,
        no_invalid_materials,
        no_nonfinite_fields,
        stable_bulk_before_max,
        post_settle_stable,
        exact_reset,
    }
}

#[derive(Clone, Debug)]
struct WaterLifecycle {
    terminal_reason: TerminalReason,
    first_all_sleep: Option<SampleIdentity>,
    confirmed_all_sleep_sim_tick: Option<u64>,
    first_stable_plateau: Option<SampleIdentity>,
    confirmed_stable_plateau_sim_tick: Option<u64>,
    terminal: SampleIdentity,
    post_settle_end_tick: Option<u64>,
    post_settle_change_ticks: u32,
    post_settle_wake_ticks: u32,
}

fn write_water_frames_json(
    config: &super::ExperimentWorkerConfig,
    path: &Path,
    frames: &[WrittenFrame],
) -> Result<(), String> {
    let entries = frames
        .iter()
        .map(|frame| {
            format!(
                concat!(
                    "{{\"ordinal\":{},\"kind\":\"{}\",\"relative_path\":\"{}\",",
                    "\"width\":{},\"height\":{},\"rgba_bytes\":{},\"reason\":\"{}\",",
                    "\"sim_tick\":{},\"sample_sequence\":{},\"state_hash\":\"{}\"}}"
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
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let json = format!(
        concat!(
            "{{\n  \"schema_version\": \"{}\",\n  \"experiment_id\": \"{}\",",
            "\n  \"run_id\": \"{}\",\n  \"scenario\": \"water-flow\",",
            "\n  \"binary_sha256\": \"{}\",",
            "\n  \"frame_count\": {},\n  \"pixel_encoding\": \"rgba8-tightly-packed\",",
            "\n  \"frames\": [{}]\n}}\n"
        ),
        WATER_FRAMES_SCHEMA_VERSION,
        json_escape(&config.experiment_id),
        json_escape(&config.run_id),
        json_escape(&config.binary_sha256.to_ascii_lowercase()),
        frames.len(),
        entries,
    );
    write_new(path, json.as_bytes())
}

#[allow(clippy::too_many_arguments)]
fn write_water_analysis_json(
    config: &super::ExperimentWorkerConfig,
    provenance: &RuntimeProvenance,
    simulation: &Simulation,
    path: &Path,
    baseline: &WaterBaseline,
    observations: &WaterObservations,
    lifecycle: &WaterLifecycle,
    predicates: &WaterPredicates,
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
        predicate_json("actual_water_movement", &predicates.actual_water_movement),
        predicate_json("cross_chunk_flow", &predicates.cross_chunk_flow),
        predicate_json("destination_arrival", &predicates.destination_arrival),
        predicate_json("water_conservation", &predicates.water_conservation),
        predicate_json("no_invalid_materials", &predicates.no_invalid_materials),
        predicate_json("no_nonfinite_fields", &predicates.no_nonfinite_fields),
        predicate_json("stable_bulk_before_max", &predicates.stable_bulk_before_max),
        predicate_json("post_settle_stable", &predicates.post_settle_stable),
        predicate_json("exact_reset", &predicates.exact_reset),
    ]
    .join(",");

    let first_all_sleep_tick = lifecycle.first_all_sleep.map(|value| value.sim_tick);
    let first_all_sleep_sample = lifecycle.first_all_sleep.map(|value| value.sample_sequence);
    let first_plateau_tick = lifecycle.first_stable_plateau.map(|value| value.sim_tick);
    let first_plateau_sample = lifecycle
        .first_stable_plateau
        .map(|value| value.sample_sequence);
    let first_movement_tick = observations.first_movement.map(|value| value.sim_tick);
    let first_movement_sample = observations
        .first_movement
        .map(|value| value.sample_sequence);
    let first_cross_tick = observations.first_cross.map(|value| value.sim_tick);
    let first_cross_sample = observations.first_cross.map(|value| value.sample_sequence);
    let first_destination_tick = observations.first_destination.map(|value| value.sim_tick);
    let first_destination_sample = observations
        .first_destination
        .map(|value| value.sample_sequence);
    let first_sleeping_tick = observations.first_sleeping.map(|value| value.sim_tick);
    let first_sleeping_sample = observations
        .first_sleeping
        .map(|value| value.sample_sequence);
    let max_spread_tick = observations
        .max_destination_spread
        .map(|value| value.sim_tick);
    let max_spread_sample = observations
        .max_destination_spread
        .map(|value| value.sample_sequence);
    let latest = &observations.latest;

    let json = format!(
        concat!(
            "{{\n  \"schema_version\": \"{}\",\n  \"experiment_id\": \"{}\",",
            "\n  \"run_id\": \"{}\",\n  \"scenario\": \"water-flow\",",
            "\n  \"binary_sha256\": \"{}\",",
            "\n  \"provenance\": {{\"source_sha\":\"{}\",\"git_state\":\"{}\",\"build_profile\":\"{}\"}},",
            "\n  \"world\": {{\"width\":{},\"height\":{},\"chunk_size\":{}}},",
            "\n  \"sleep\": {{\"enabled\":{},\"threshold\":{}}},",
            "\n  \"lifecycle\": {{\"max_ticks\":{},\"diagnostic_interval_ticks\":{},",
            "\"all_sleep_consecutive_samples\":{},\"stable_plateau_consecutive_samples\":{},",
            "\"post_settle_confirmation_ticks\":{},\"terminal_reason\":\"{}\",",
            "\"first_all_sleep_sim_tick\":{},\"first_all_sleep_sample_sequence\":{},",
            "\"confirmed_all_sleep_sim_tick\":{},",
            "\"first_stable_plateau_sim_tick\":{},",
            "\"first_stable_plateau_sample_sequence\":{},",
            "\"confirmed_stable_plateau_sim_tick\":{},",
            "\"terminal_sim_tick\":{},\"terminal_sample_sequence\":{},",
            "\"post_settle_end_tick\":{},\"post_settle_change_ticks\":{},",
            "\"post_settle_wake_ticks\":{},\"sample_count\":{}}},",
            "\n  \"baseline\": {{\"matter_count\":{},\"water_count\":{},\"oil_count\":{},",
            "\"water_y_sum\":{},\"oil_y_sum\":{},",
            "\"water_occupied_chunks\":{},\"oil_occupied_chunks\":{},",
            "\"bottom_chunk_row_water_cells\":{},",
            "\"destination_water_cells\":{},\"destination_spread_x\":{}}},",
            "\n  \"metrics\": {{\"peak_active_cells\":{},\"peak_active_chunks\":{},",
            "\"peak_active_sim_tick\":{},\"peak_active_sample_sequence\":{},",
            "\"first_water_movement_tick\":{},",
            "\"first_water_movement_sample_sequence\":{},",
            "\"first_cross_chunk_flow_tick\":{},",
            "\"first_cross_chunk_flow_sample_sequence\":{},",
            "\"first_destination_arrival_tick\":{},",
            "\"first_destination_arrival_sample_sequence\":{},",
            "\"first_sleeping_chunk_tick\":{},",
            "\"first_sleeping_chunk_sample_sequence\":{},",
            "\"max_bottom_chunk_row_water_cells\":{},",
            "\"max_destination_water_cells\":{},",
            "\"max_destination_spread_x\":{},",
            "\"max_destination_spread_tick\":{},",
            "\"max_destination_spread_sample_sequence\":{},",
            "\"final_matter_count\":{},\"final_water_count\":{},",
            "\"final_oil_count\":{},\"final_water_occupied_chunks\":{},",
            "\"final_oil_occupied_chunks\":{},\"final_sleeping_chunks\":{},",
            "\"matter_count_delta\":{},\"water_count_delta\":{},\"oil_count_delta\":{},",
            "\"post_settle_state_changes\":{},",
            "\"post_settle_spontaneous_wakes\":{},",
            "\"reset_exact_equivalence\":{}}},",
            "\n  \"predicates\": {{{}}},",
            "\n  \"verdict\": \"{}\",\n  \"raw_frame_count\": {}\n}}\n"
        ),
        WATER_ANALYSIS_SCHEMA_VERSION,
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
        config.consecutive_all_sleep,
        REQUIRED_STABLE_PLATEAU_SAMPLES,
        config.post_sleep_ticks,
        lifecycle.terminal_reason.as_str(),
        json_opt_u64(first_all_sleep_tick),
        json_opt_u64(first_all_sleep_sample),
        json_opt_u64(lifecycle.confirmed_all_sleep_sim_tick),
        json_opt_u64(first_plateau_tick),
        json_opt_u64(first_plateau_sample),
        json_opt_u64(lifecycle.confirmed_stable_plateau_sim_tick),
        lifecycle.terminal.sim_tick,
        lifecycle.terminal.sample_sequence,
        json_opt_u64(lifecycle.post_settle_end_tick),
        lifecycle.post_settle_change_ticks,
        lifecycle.post_settle_wake_ticks,
        sample_count,
        baseline.matter_count,
        baseline.water_count,
        baseline.oil_count,
        baseline.water_y_sum,
        baseline.oil_y_sum,
        baseline.water_occupied_chunks,
        baseline.oil_occupied_chunks,
        baseline.bottom_chunk_row_water_cells,
        baseline.destination_water_cells,
        baseline.destination_spread_x,
        observations.peak_active_cells,
        observations.peak_active_chunks,
        observations.peak_active.sim_tick,
        observations.peak_active.sample_sequence,
        json_opt_u64(first_movement_tick),
        json_opt_u64(first_movement_sample),
        json_opt_u64(first_cross_tick),
        json_opt_u64(first_cross_sample),
        json_opt_u64(first_destination_tick),
        json_opt_u64(first_destination_sample),
        json_opt_u64(first_sleeping_tick),
        json_opt_u64(first_sleeping_sample),
        observations.max_bottom_chunk_row_water_cells,
        observations.max_destination_water_cells,
        observations.max_destination_spread_x,
        json_opt_u64(max_spread_tick),
        json_opt_u64(max_spread_sample),
        latest.matter_count,
        latest.water_count,
        latest.oil_count,
        latest.water_occupied_chunks,
        latest.oil_occupied_chunks,
        latest.sleeping_chunks,
        i128::from(latest.matter_count) - i128::from(baseline.matter_count),
        i128::from(latest.water_count) - i128::from(baseline.water_count),
        i128::from(latest.oil_count) - i128::from(baseline.oil_count),
        lifecycle.post_settle_change_ticks,
        lifecycle.post_settle_wake_ticks,
        exact_reset,
        predicates_json,
        verdict.as_str(),
        raw_frame_count,
    );
    write_new(path, json.as_bytes())
}

#[allow(clippy::too_many_arguments)]
fn record_observation_update(
    output: &mut WaterJsonlWriters,
    config: &super::ExperimentWorkerConfig,
    update: ObservationUpdate,
    metrics: &WaterSampleMetrics,
    frame: &SemanticFrame,
    first_movement_frame: &mut Option<SemanticFrame>,
    cross_chunk_frame: &mut Option<SemanticFrame>,
    destination_frame: &mut Option<SemanticFrame>,
    peak_active_frame: &mut Option<SemanticFrame>,
    max_spread_frame: &mut Option<SemanticFrame>,
    first_sleeping_frame: &mut Option<SemanticFrame>,
) -> Result<(), String> {
    if update.first_movement {
        *first_movement_frame =
            Some(frame.clone_with_kind("first-movement", "first-observed-water-mask-movement"));
        output.event(
            config,
            "water_movement_observed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!(
                "water_outside_initial_mask={}; initial_water_cells_vacated={}",
                metrics.water_outside_initial_mask, metrics.initial_water_cells_vacated
            ),
        )?;
    }
    if update.first_cross {
        *cross_chunk_frame =
            Some(frame.clone_with_kind("cross-chunk-flow", "first-water-in-bottom-chunk-row"));
        output.event(
            config,
            "cross_chunk_flow_observed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!(
                "bottom_chunk_row_water_cells={}",
                metrics.bottom_chunk_row_water_cells
            ),
        )?;
    }
    if update.first_destination {
        *destination_frame =
            Some(frame.clone_with_kind("destination-arrival", "first-water-in-destination-basin"));
        output.event(
            config,
            "destination_arrival_observed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!(
                "destination_water_cells={}; destination_spread_x={}",
                metrics.destination_water_cells, metrics.destination_spread_x
            ),
        )?;
    }
    if update.new_peak {
        *peak_active_frame =
            Some(frame.clone_with_kind("peak-active", "highest-observed-active-cells"));
        output.event(
            config,
            "new_peak_active",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!("any_active_cells={}", metrics.any_active_cells),
        )?;
    }
    if update.new_max_spread {
        *max_spread_frame =
            Some(frame.clone_with_kind("max-destination-spread", "new-maximum-destination-spread"));
        output.event(
            config,
            "new_max_destination_spread",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!("destination_spread_x={}", metrics.destination_spread_x),
        )?;
    }
    if update.first_sleeping {
        *first_sleeping_frame =
            Some(frame.clone_with_kind("first-sleeping-chunk", "first-observed-sleeping-chunk"));
        output.event(
            config,
            "first_sleeping_chunk_observed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!("sleeping_chunks={}", metrics.sleeping_chunks),
        )?;
    }
    Ok(())
}

fn terminal_detail(reason: TerminalReason, observations: &WaterObservations) -> String {
    let mut missing = Vec::new();
    if observations.first_movement.is_none() {
        missing.push("actual_water_movement");
    }
    if observations.first_cross.is_none() {
        missing.push("cross_chunk_flow");
    }
    if observations.first_destination.is_none() {
        missing.push("destination_arrival");
    }
    format!(
        "reason={}; missing_milestones={}",
        reason.as_str(),
        if missing.is_empty() {
            "none".to_string()
        } else {
            missing.join(",")
        }
    )
}

/// Runs the Water Flow experiment lifecycle through production simulation
/// ticks. Semantic FAIL/NEEDS_HUMAN_REVIEW outcomes still return `Ok`; `Err`
/// is reserved for incomplete configuration, GPU, renderer, or filesystem
/// operations.
pub fn run_water_flow_experiment(
    simulation: &mut Simulation,
    renderer: &mut Renderer,
    provenance: &RuntimeProvenance,
    config: &super::ExperimentWorkerConfig,
) -> Result<ExperimentOutcome, String> {
    validate_water_worker_config(simulation, config)?;

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
    let mut output = WaterJsonlWriters::new(&samples_path, &events_path)?;
    output.event(
        config,
        "lifecycle_started",
        simulation.tick_count,
        None,
        "Water worker output opened",
    )?;

    reset_and_stage_scenario(simulation, config.scenario)
        .map_err(|error| format!("pristine Water Flow reset/stage failed: {error}"))?;
    output.event(
        config,
        "pristine_reset_completed",
        0,
        None,
        "shared Water Flow reset/staging completed",
    )?;

    let baseline_sleep_enabled = simulation.sleep_enabled;
    let baseline_sleep_threshold = simulation.sleep_threshold;
    let mut next_sample_sequence = 0u64;
    let tick0_snapshot = capture_gpu_snapshot(simulation)?;
    let tick0_metrics = water_metrics_from_snapshot(
        &tick0_snapshot,
        simulation.world.config,
        None,
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "initial",
        "tick0",
    )?;
    let baseline = baseline_from_tick0(&tick0_snapshot, &tick0_metrics, simulation.world.config);
    output.sample(config, provenance, simulation, &tick0_metrics)?;
    let tick0_frame = capture_water_frame(renderer, "tick0", "pristine-reset", &tick0_metrics)?;
    output.event(
        config,
        "tick0_captured",
        tick0_metrics.sim_tick,
        Some(tick0_metrics.sample_sequence),
        &tick0_metrics.state_hash,
    )?;

    let mut observations = WaterObservations::new(&tick0_metrics);
    let mut first_movement_frame = None;
    let mut cross_chunk_frame = None;
    let mut destination_frame = None;
    let mut peak_active_frame =
        Some(tick0_frame.clone_with_kind("peak-active", "highest-observed-active-cells"));
    let mut max_spread_frame = (tick0_metrics.destination_spread_x != 0).then(|| {
        tick0_frame.clone_with_kind("max-destination-spread", "new-maximum-destination-spread")
    });
    let mut first_sleeping_frame = (tick0_metrics.sleeping_chunks != 0).then(|| {
        tick0_frame.clone_with_kind("first-sleeping-chunk", "first-observed-sleeping-chunk")
    });
    if tick0_metrics.sleeping_chunks != 0 {
        output.event(
            config,
            "first_sleeping_chunk_observed",
            tick0_metrics.sim_tick,
            Some(tick0_metrics.sample_sequence),
            &format!("sleeping_chunks={}", tick0_metrics.sleeping_chunks),
        )?;
    }

    simulation
        .tick()
        .map_err(|error| format!("production tick 1 failed: {error}"))?;
    let tick1_snapshot = capture_gpu_snapshot(simulation)?;
    let tick1_metrics = water_metrics_from_snapshot(
        &tick1_snapshot,
        simulation.world.config,
        Some(&baseline),
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "flowing",
        "tick1",
    )?;
    let tick1_update = observations.observe(&tick1_metrics, &baseline, true);
    output.sample(config, provenance, simulation, &tick1_metrics)?;
    let tick1_frame = capture_water_frame(
        renderer,
        "tick1",
        "after-one-production-tick",
        &tick1_metrics,
    )?;
    output.event(
        config,
        "tick1_captured",
        tick1_metrics.sim_tick,
        Some(tick1_metrics.sample_sequence),
        &tick1_metrics.state_hash,
    )?;
    record_observation_update(
        &mut output,
        config,
        tick1_update,
        &tick1_metrics,
        &tick1_frame,
        &mut first_movement_frame,
        &mut cross_chunk_frame,
        &mut destination_frame,
        &mut peak_active_frame,
        &mut max_spread_frame,
        &mut first_sleeping_frame,
    )?;

    let mut diagnostics = VecDeque::with_capacity(DIAGNOSTIC_RING_CAPACITY);
    let mut all_sleep_detector = AllSleepDetector::new(config.consecutive_all_sleep);
    let mut plateau_detector = StablePlateauDetector::new(REQUIRED_STABLE_PLATEAU_SAMPLES);
    let mut previous_diagnostic_frame: Option<SemanticFrame> = None;
    let late_frame;
    let mut first_all_sleep = None;
    let mut confirmed_all_sleep_sim_tick = None;
    let mut first_stable_plateau = None;
    let mut confirmed_stable_plateau_sim_tick = None;
    let terminal_reason;
    let terminal_snapshot;
    let terminal_metrics;
    let terminal_frame;

    loop {
        if simulation.tick_count >= config.max_ticks {
            return Err(
                "Water lifecycle reached max tick without a max-tick diagnostic".to_string(),
            );
        }
        simulation.tick().map_err(|error| {
            format!(
                "Water production tick {} failed: {error}",
                simulation.tick_count + 1
            )
        })?;
        let sim_tick = simulation.tick_count;
        let is_early = sim_tick == 2;
        let is_cadence = sim_tick.is_multiple_of(config.diagnostic_interval_ticks);
        let is_max = sim_tick == config.max_ticks;
        if !is_early && !is_cadence && !is_max {
            continue;
        }

        let reason = if is_early {
            "early-flow"
        } else if is_max {
            "max-tick"
        } else {
            "diagnostic-cadence"
        };
        let snapshot = capture_gpu_snapshot(simulation)?;
        let metrics = water_metrics_from_snapshot(
            &snapshot,
            simulation.world.config,
            Some(&baseline),
            take_sequence(&mut next_sample_sequence),
            sim_tick,
            "flowing",
            reason,
        )?;
        let update = observations.observe(&metrics, &baseline, true);
        output.sample(config, provenance, simulation, &metrics)?;
        let frame = capture_water_frame(renderer, "diagnostic", reason, &metrics)?;
        record_observation_update(
            &mut output,
            config,
            update,
            &metrics,
            &frame,
            &mut first_movement_frame,
            &mut cross_chunk_frame,
            &mut destination_frame,
            &mut peak_active_frame,
            &mut max_spread_frame,
            &mut first_sleeping_frame,
        )?;
        remember_diagnostic(&mut diagnostics, &frame);

        let sleep_update = all_sleep_detector.observe(
            metrics.all_sleep(),
            metrics.sim_tick,
            metrics.sample_sequence,
        );
        if sleep_update.streak_broken {
            output.event(
                config,
                "all_sleep_streak_broken",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                "diagnostic no longer satisfied the exact all-sleep census",
            )?;
        }
        if sleep_update.first_in_streak {
            output.event(
                config,
                "all_sleep_observed",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                "first all-sleep sample in current diagnostic streak",
            )?;
        }

        let mut selected_reason = None;
        if sleep_update.confirmed {
            let first = SampleIdentity {
                sim_tick: all_sleep_detector
                    .first_sim_tick
                    .unwrap_or(metrics.sim_tick),
                sample_sequence: all_sleep_detector
                    .first_sample_sequence
                    .unwrap_or(metrics.sample_sequence),
            };
            first_all_sleep = Some(first);
            confirmed_all_sleep_sim_tick = Some(metrics.sim_tick);
            output.event(
                config,
                "all_sleep_confirmed",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                &format!(
                    "{} consecutive diagnostics; first sim_tick={} sample_sequence={}",
                    config.consecutive_all_sleep, first.sim_tick, first.sample_sequence
                ),
            )?;
            selected_reason = Some(TerminalReason::AllSleep);
        }

        // Plateau telemetry is independent of terminal precedence. An
        // all-sleep confirmation can coincide with the eighth unchanged
        // diagnostic, in which case both confirmations are recorded and the
        // stronger all-sleep terminal remains selected.
        let plateau_update = plateau_detector.observe(&metrics);
        if plateau_update.streak_broken {
            output.event(
                config,
                "stable_plateau_streak_broken",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                "authoritative hash changed or changed/wake chunks became nonzero",
            )?;
        }
        if plateau_update.first_in_streak {
            output.event(
                config,
                "stable_plateau_observed",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                "first unchanged authoritative diagnostic with zero changed/wake chunks",
            )?;
        }
        if plateau_update.confirmed {
            let first = SampleIdentity {
                sim_tick: plateau_detector.first_sim_tick.unwrap_or(metrics.sim_tick),
                sample_sequence: plateau_detector
                    .first_sample_sequence
                    .unwrap_or(metrics.sample_sequence),
            };
            first_stable_plateau = Some(first);
            confirmed_stable_plateau_sim_tick = Some(metrics.sim_tick);
            output.event(
                config,
                "stable_plateau_confirmed",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                &format!(
                    "{REQUIRED_STABLE_PLATEAU_SAMPLES} consecutive diagnostics; first sim_tick={} sample_sequence={}",
                    first.sim_tick, first.sample_sequence
                ),
            )?;
            if selected_reason.is_none() {
                selected_reason = Some(TerminalReason::StablePlateau);
            }
        }
        if selected_reason.is_none() && is_max {
            selected_reason = Some(TerminalReason::MaxTicks);
        }

        if let Some(reason) = selected_reason {
            late_frame = previous_diagnostic_frame.as_ref().map(|previous| {
                previous.clone_with_kind("late", "observation-before-terminal-diagnostic")
            });
            terminal_reason = reason;
            terminal_snapshot = snapshot;
            terminal_metrics = metrics.clone();
            terminal_frame = frame.clone_with_kind(
                "terminal",
                match reason {
                    TerminalReason::AllSleep => "all-sleep-confirmed",
                    TerminalReason::StablePlateau => "stable-plateau-confirmed",
                    TerminalReason::MaxTicks => "max-tick-reached",
                },
            );
            output.event(
                config,
                "terminal_selected",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                &terminal_detail(reason, &observations),
            )?;
            break;
        }
        previous_diagnostic_frame = Some(frame);
    }

    let mut post_settle_frame = None;
    let mut post_settle_end_tick = None;
    let mut post_settle_change_ticks = 0u32;
    let mut post_settle_wake_ticks = 0u32;
    if terminal_reason.has_post_settle_window() {
        for offset in 1..=config.post_sleep_ticks {
            simulation.tick().map_err(|error| {
                format!(
                    "post-settle production tick {offset}/{} failed: {error}",
                    config.post_sleep_ticks
                )
            })?;
            let snapshot = capture_gpu_snapshot(simulation)?;
            let metrics = water_metrics_from_snapshot(
                &snapshot,
                simulation.world.config,
                Some(&baseline),
                take_sequence(&mut next_sample_sequence),
                simulation.tick_count,
                "post-settle-confirmation",
                "post-settle-tick",
            )?;
            let update = observations.observe(&metrics, &baseline, false);
            if !physical_tick_boundary_equal(&terminal_snapshot, &snapshot)
                || metrics.changed_chunks != 0
            {
                post_settle_change_ticks = post_settle_change_ticks.saturating_add(1);
            }
            let all_sleep_wake = terminal_reason == TerminalReason::AllSleep
                && (metrics.any_active_cells != 0
                    || metrics.active_chunks != 0
                    || metrics.runnable_chunks != 0
                    || metrics.sleeping_chunks != metrics.total_chunks);
            if metrics.wake_chunks != 0 || all_sleep_wake {
                post_settle_wake_ticks = post_settle_wake_ticks.saturating_add(1);
            }
            output.sample(config, provenance, simulation, &metrics)?;

            let needs_frame = offset == config.post_sleep_ticks || update.first_sleeping;
            if needs_frame {
                let frame = capture_water_frame(
                    renderer,
                    "post-settle",
                    if offset == config.post_sleep_ticks {
                        "post-settle-confirmation-complete"
                    } else {
                        "post-settle-observation"
                    },
                    &metrics,
                )?;
                if update.first_sleeping {
                    first_sleeping_frame =
                        Some(frame.clone_with_kind(
                            "first-sleeping-chunk",
                            "first-observed-sleeping-chunk",
                        ));
                    output.event(
                        config,
                        "first_sleeping_chunk_observed",
                        metrics.sim_tick,
                        Some(metrics.sample_sequence),
                        &format!("sleeping_chunks={}", metrics.sleeping_chunks),
                    )?;
                }
                if offset == config.post_sleep_ticks {
                    post_settle_frame = Some(frame);
                    post_settle_end_tick = Some(metrics.sim_tick);
                }
            }
        }
        output.event(
            config,
            "post_settle_confirmation_completed",
            simulation.tick_count,
            Some(next_sample_sequence.saturating_sub(1)),
            &format!(
                "ticks={}; state_change_ticks={post_settle_change_ticks}; wake_ticks={post_settle_wake_ticks}",
                config.post_sleep_ticks
            ),
        )?;
    }

    output.event(
        config,
        "reset_started",
        simulation.tick_count,
        Some(next_sample_sequence.saturating_sub(1)),
        "programmatic R-equivalent shared Water reset/staging",
    )?;
    reset_and_stage_scenario(simulation, config.scenario)
        .map_err(|error| format!("programmatic Water reset failed: {error}"))?;
    let reset_snapshot = capture_gpu_snapshot(simulation)?;
    let reset_metrics = water_metrics_from_snapshot(
        &reset_snapshot,
        simulation.world.config,
        Some(&baseline),
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "reset",
        "programmatic-r-equivalent",
    )?;
    output.sample(config, provenance, simulation, &reset_metrics)?;
    let reset_frame = capture_water_frame(
        renderer,
        "reset",
        "programmatic-r-equivalent",
        &reset_metrics,
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

    let lifecycle = WaterLifecycle {
        terminal_reason,
        first_all_sleep,
        confirmed_all_sleep_sim_tick,
        first_stable_plateau,
        confirmed_stable_plateau_sim_tick,
        terminal: SampleIdentity::from_metrics(&terminal_metrics),
        post_settle_end_tick,
        post_settle_change_ticks,
        post_settle_wake_ticks,
    };
    let predicates = build_water_predicates(
        &observations,
        terminal_reason,
        first_all_sleep.map(|value| value.sim_tick),
        confirmed_all_sleep_sim_tick,
        config.max_ticks,
        post_settle_end_tick,
        post_settle_change_ticks,
        post_settle_wake_ticks,
        exact_reset,
    );
    let verdict = predicates.verdict();

    let semantic_frames = assemble_semantic_frames(
        tick0_frame,
        tick1_frame,
        first_movement_frame,
        peak_active_frame,
        cross_chunk_frame,
        destination_frame,
        max_spread_frame,
        first_sleeping_frame,
        late_frame,
        terminal_frame,
        post_settle_frame,
        reset_frame,
        verdict,
        &diagnostics,
    )?;
    let written_frames = write_raw_frames(&raw_frames_dir, semantic_frames)?;
    write_water_frames_json(config, &frames_path, &written_frames)?;
    write_water_analysis_json(
        config,
        provenance,
        simulation,
        &analysis_path,
        &baseline,
        &observations,
        &lifecycle,
        &predicates,
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
        first_all_sleep_sim_tick: first_all_sleep.map(|value| value.sim_tick),
        first_all_sleep_sample_sequence: first_all_sleep.map(|value| value.sample_sequence),
        post_sleep_end_tick: post_settle_end_tick,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use powdergame_core::MATERIAL_STONE;

    fn snapshot_with_materials(materials: Vec<u32>) -> GpuSnapshot {
        let cell_count = materials.len();
        let chunk_count = 16;
        GpuSnapshot {
            material_current: materials.clone(),
            material_next: materials,
            temperature_current: vec![20.0f32.to_bits(); cell_count],
            temperature_next: vec![20.0f32.to_bits(); cell_count],
            pressure_current: vec![0.0f32.to_bits(); cell_count],
            pressure_next: vec![0.0f32.to_bits(); cell_count],
            flags_current: vec![0; cell_count],
            flags_next: vec![0; cell_count],
            proposal: vec![u32::MAX; cell_count],
            claim: vec![0; cell_count],
            cell_activity: vec![0; cell_count],
            chunk_activity: vec![0; chunk_count],
            chunk_changed: vec![0; chunk_count],
            chunk_stable: vec![0; chunk_count],
            chunk_edit_wake: vec![0; chunk_count],
            chunk_state: vec![CHUNK_STATE_SLEEPING; chunk_count],
            chunk_wake_reason: vec![0; chunk_count],
            params: vec![0; 8],
            wake_params: vec![0; 4],
            arbitration_params: vec![0; 4],
        }
    }

    fn cell(x: u32, y: u32) -> usize {
        (y * REQUIRED_WORLD.width + x) as usize
    }

    fn water_baseline_and_moved() -> (WaterBaseline, WaterSampleMetrics, WaterSampleMetrics) {
        let cell_count = (REQUIRED_WORLD.width * REQUIRED_WORLD.height) as usize;
        let mut initial = vec![MATERIAL_EMPTY; cell_count];
        initial[cell(10, 10)] = MATERIAL_WATER;
        initial[cell(20, 20)] = MATERIAL_WATER;
        initial[cell(30, 30)] = MATERIAL_OIL;
        let initial_snapshot = snapshot_with_materials(initial);
        let initial_metrics = water_metrics_from_snapshot(
            &initial_snapshot,
            REQUIRED_WORLD,
            None,
            0,
            0,
            "initial",
            "tick0",
        )
        .expect("initial metrics");
        let baseline = baseline_from_tick0(&initial_snapshot, &initial_metrics, REQUIRED_WORLD);

        let mut moved = initial_snapshot.material_current.clone();
        moved[cell(10, 10)] = MATERIAL_EMPTY;
        moved[cell(20, 20)] = MATERIAL_EMPTY;
        moved[cell(18, 200)] = MATERIAL_WATER;
        moved[cell(100, 205)] = MATERIAL_WATER;
        let moved_snapshot = snapshot_with_materials(moved);
        let moved_metrics = water_metrics_from_snapshot(
            &moved_snapshot,
            REQUIRED_WORLD,
            Some(&baseline),
            1,
            8,
            "flowing",
            "diagnostic-cadence",
        )
        .expect("moved metrics");
        (baseline, initial_metrics, moved_metrics)
    }

    #[test]
    fn water_metrics_recompute_mask_cross_destination_and_spread() {
        let (baseline, initial, moved) = water_baseline_and_moved();
        assert_eq!(baseline.water_count, 2);
        assert_eq!(baseline.oil_count, 1);
        assert_eq!(initial.destination_water_cells, 0);
        assert_eq!(moved.water_count, baseline.water_count);
        assert_eq!(moved.oil_count, baseline.oil_count);
        assert_eq!(moved.water_outside_initial_mask, 2);
        assert_eq!(moved.initial_water_cells_vacated, 2);
        assert_eq!(moved.bottom_chunk_row_water_cells, 2);
        assert_eq!(moved.destination_water_cells, 2);
        assert_eq!(moved.destination_spread_x, 83);
        assert!(moved.movement_observed());
    }

    #[test]
    fn destination_metric_excludes_cells_that_were_not_empty_at_tick0() {
        let cell_count = (REQUIRED_WORLD.width * REQUIRED_WORLD.height) as usize;
        let mut initial = vec![MATERIAL_EMPTY; cell_count];
        initial[cell(124, 205)] = MATERIAL_STONE;
        let initial_snapshot = snapshot_with_materials(initial);
        let initial_metrics = water_metrics_from_snapshot(
            &initial_snapshot,
            REQUIRED_WORLD,
            None,
            0,
            0,
            "initial",
            "tick0",
        )
        .expect("initial metrics");
        let baseline = baseline_from_tick0(&initial_snapshot, &initial_metrics, REQUIRED_WORLD);

        let mut changed = initial_snapshot.material_current.clone();
        changed[cell(124, 205)] = MATERIAL_WATER;
        let changed_metrics = water_metrics_from_snapshot(
            &snapshot_with_materials(changed),
            REQUIRED_WORLD,
            Some(&baseline),
            1,
            8,
            "flowing",
            "diagnostic-cadence",
        )
        .expect("changed metrics");

        assert_eq!(changed_metrics.destination_water_cells, 0);
        assert_eq!(changed_metrics.destination_spread_x, 0);
    }

    #[test]
    fn physical_hash_covers_next_buffers_beyond_authoritative_current_hash() {
        let cell_count = (REQUIRED_WORLD.width * REQUIRED_WORLD.height) as usize;
        let snapshot = snapshot_with_materials(vec![MATERIAL_EMPTY; cell_count]);
        let mut changed_next = snapshot.clone();
        changed_next.material_next[cell(10, 10)] = MATERIAL_WATER;
        assert_eq!(
            authoritative_current_hash(&snapshot),
            authoritative_current_hash(&changed_next)
        );
        assert_ne!(
            physical_tick_boundary_hash(&snapshot),
            physical_tick_boundary_hash(&changed_next)
        );
    }

    #[test]
    fn stable_plateau_requires_eight_identical_zero_change_diagnostics() {
        let (_, _, mut metrics) = water_baseline_and_moved();
        metrics.state_hash = "fnv1a64:0123456789abcdef".to_string();
        let mut detector = StablePlateauDetector::new(REQUIRED_STABLE_PLATEAU_SAMPLES);
        for sequence in 0..7 {
            metrics.sample_sequence = sequence;
            metrics.sim_tick = sequence * 8;
            let update = detector.observe(&metrics);
            assert!(!update.confirmed);
        }
        metrics.sample_sequence = 7;
        metrics.sim_tick = 56;
        assert!(detector.observe(&metrics).confirmed);
        assert_eq!(detector.first_sim_tick, Some(0));
        assert_eq!(detector.first_sample_sequence, Some(0));

        metrics.state_hash = "fnv1a64:fedcba9876543210".to_string();
        let update = detector.observe(&metrics);
        assert!(update.streak_broken);
        assert!(update.first_in_streak);
        assert!(!update.confirmed);
        metrics.changed_chunks = 1;
        assert!(detector.observe(&metrics).streak_broken);
    }

    #[test]
    fn water_predicates_are_unknown_for_missing_signals_and_fail_first_for_hard_data() {
        let (baseline, initial, moved) = water_baseline_and_moved();
        let mut observations = WaterObservations::new(&initial);
        let update = observations.observe(&moved, &baseline, true);
        assert!(update.first_movement);
        assert!(update.first_cross);
        assert!(update.first_destination);
        let predicates = build_water_predicates(
            &observations,
            TerminalReason::AllSleep,
            Some(8),
            Some(24),
            100,
            Some(188),
            0,
            0,
            true,
        );
        assert_eq!(predicates.statuses(), [PredicateStatus::Pass; 9]);
        assert_eq!(predicates.verdict(), ExperimentVerdict::Pass);

        let missing = WaterObservations::new(&initial);
        let predicates = build_water_predicates(
            &missing,
            TerminalReason::StablePlateau,
            None,
            None,
            100,
            Some(188),
            0,
            0,
            true,
        );
        assert_eq!(predicates.verdict(), ExperimentVerdict::NeedsHumanReview);

        let mut hard_failure = observations;
        hard_failure.counts_conserved = false;
        let predicates = build_water_predicates(
            &hard_failure,
            TerminalReason::AllSleep,
            Some(8),
            Some(24),
            100,
            Some(188),
            0,
            0,
            true,
        );
        assert_eq!(predicates.verdict(), ExperimentVerdict::Fail);
    }

    fn semantic_frame(
        sim_tick: u64,
        sample_sequence: u64,
        kind: &'static str,
        pixel: u8,
    ) -> SemanticFrame {
        SemanticFrame {
            kind,
            reason: "test",
            sim_tick,
            sample_sequence,
            state_hash: format!("fnv1a64:{sim_tick:016x}"),
            frame: RawFrame {
                width: 1,
                height: 1,
                rgba: vec![pixel, pixel, pixel, 255],
            },
        }
    }

    #[test]
    fn frame_shortfall_uses_honest_diagnostics_without_inventing_milestones() {
        let mut diagnostics = VecDeque::new();
        for sequence in 2..=7 {
            diagnostics.push_back(
                semantic_frame(sequence, sequence, "diagnostic-observation", 2)
                    .clone_with_kind("diagnostic-observation", "minimum-evidence-observation"),
            );
        }
        let frames = assemble_semantic_frames(
            semantic_frame(0, 0, "tick0", 0),
            semantic_frame(1, 1, "tick1", 1),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            semantic_frame(8, 8, "terminal", 2),
            None,
            semantic_frame(0, 9, "reset", 0),
            ExperimentVerdict::NeedsHumanReview,
            &diagnostics,
        )
        .expect("minimum evidence frame set");
        assert_eq!(frames.len(), WATER_MIN_RAW_FRAMES);
        assert!(frames
            .iter()
            .any(|frame| frame.kind == "diagnostic-observation"));
        assert!(frames.iter().any(|frame| frame.kind == "reset"));
        assert!(!frames.iter().any(|frame| frame.kind == "first-movement"));
    }

    #[test]
    fn peak_alias_is_retained_but_diagnostic_fallback_identities_stay_distinct() {
        let tick0 = semantic_frame(0, 0, "tick0", 0);
        let peak = tick0.clone_with_kind("peak-active", "highest-observed-active-cells");
        let diagnostics = VecDeque::from([semantic_frame(2, 2, "diagnostic-observation", 2)
            .clone_with_kind("diagnostic-observation", "minimum-evidence-observation")]);
        let frames = assemble_semantic_frames(
            tick0,
            semantic_frame(1, 1, "tick1", 1),
            None,
            Some(peak),
            None,
            None,
            None,
            None,
            Some(semantic_frame(3, 3, "late", 3)),
            semantic_frame(4, 4, "terminal", 4),
            Some(semantic_frame(184, 5, "post-settle", 4)),
            semantic_frame(0, 6, "reset", 0),
            ExperimentVerdict::NeedsHumanReview,
            &diagnostics,
        )
        .expect("peak alias frame set");
        assert_eq!(frames.len(), WATER_MIN_RAW_FRAMES);
        assert_eq!(
            frames
                .iter()
                .filter(|frame| frame.sample_sequence == 0)
                .count(),
            2
        );
        assert!(frames.iter().any(|frame| frame.kind == "peak-active"));
        assert_eq!(
            frames
                .iter()
                .filter(|frame| frame.kind == "diagnostic-observation")
                .count(),
            1
        );
    }
}
