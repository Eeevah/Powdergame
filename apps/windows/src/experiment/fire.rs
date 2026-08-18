//! Fire / Heat Experiment Evidence Harness worker.
//!
//! This worker observes the shared Fire / Heat fixture through production
//! simulation ticks. It owns no scenario staging or physics behavior.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use powdergame_core::{
    fuel_progress, is_valid_cell_material_value, WorldConfig, ACTIVITY_MATTER, ACTIVITY_PRESSURE,
    ACTIVITY_REACTION, ACTIVITY_THERMAL, CHUNK_STATE_RUNNABLE, CHUNK_STATE_SLEEPING,
    FLAG_COMBUSTING, FLAG_FLAME_EVENT, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_OIL, MATERIAL_SMOKE,
    MATERIAL_STEAM, MATERIAL_WATER, MATERIAL_WOOD, TEMPERATURE_REFERENCE, THERMAL_ACTIVITY_EPS,
};
use powdergame_gpu::Simulation;
use powdergame_scenarios::{reset_and_stage_scenario, ScenarioId};

use crate::gallery::RuntimeProvenance;
use crate::renderer::Renderer;

use super::{
    authoritative_current_hash, bit_count, capture_gpu_snapshot, create_new_file,
    create_worker_directory, display_path, exact_reset_equal, is_safe_identifier, json_escape,
    json_opt_u64, take_sequence, write_new, write_raw_frames, ExperimentOutcome, ExperimentVerdict,
    GpuSnapshot, PredicateResult, PredicateStatus, RawFrame, SemanticFrame, WrittenFrame,
    REQUIRED_WORLD,
};

pub const FIRE_EXPERIMENT_ID: &str = "g8b-fire-heat-v0";
const FIRE_TELEMETRY_SCHEMA_VERSION: &str = "powdergame-fire-heat-telemetry-v0";
const FIRE_ANALYSIS_SCHEMA_VERSION: &str = "powdergame-fire-heat-analysis-v0";
const FIRE_FRAMES_SCHEMA_VERSION: &str = "powdergame-experiment-frames-v0";
const REQUIRED_MAX_TICKS: u64 = 20_000;
const REQUIRED_DIAGNOSTIC_INTERVAL_TICKS: u64 = 8;
const REQUIRED_REACTION_ZERO_SAMPLES: u32 = 3;
const REQUIRED_POST_REACTION_TICKS: u32 = 180;
const MIN_RAW_FRAMES: usize = 8;
const MAX_RAW_FRAMES: usize = 12;
const DIAGNOSTIC_RING_CAPACITY: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SampleIdentity {
    sim_tick: u64,
    sample_sequence: u64,
}

#[derive(Clone, Debug)]
struct FireBaseline {
    initial_temperature_bits: Vec<u32>,
    initially_ambient: Vec<bool>,
    matter_count: u64,
    wood_count: u64,
    oil_count: u64,
    smoke_count: u64,
    ice_count: u64,
    water_count: u64,
    steam_count: u64,
    wood_fuel_progress_sum: u64,
    oil_fuel_progress_sum: u64,
    substantial_fuel_consumption_threshold: u64,
    substantial_fuel_remaining_threshold: u64,
}

impl FireBaseline {
    fn fuel_count(&self) -> u64 {
        self.wood_count.saturating_add(self.oil_count)
    }
}

#[derive(Clone, Debug)]
struct FireSampleMetrics {
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
    wood_count: u64,
    oil_count: u64,
    smoke_count: u64,
    ice_count: u64,
    water_count: u64,
    steam_count: u64,
    combusting_wood_cells: u64,
    combusting_oil_cells: u64,
    flame_event_wood_cells: u64,
    flame_event_oil_cells: u64,
    wood_fuel_progress_sum: u64,
    oil_fuel_progress_sum: u64,
    heat_propagated_cells: u64,
    phase_inventory_changed: bool,
    invalid_material_count: u64,
    nonfinite_temperature_count: u64,
    nonfinite_pressure_count: u64,
    changed_chunks: u32,
    wake_chunks: u32,
    wake_reason_or: u32,
    state_hash: String,
    physical_state_hash: String,
}

impl FireSampleMetrics {
    fn identity(&self) -> SampleIdentity {
        SampleIdentity {
            sim_tick: self.sim_tick,
            sample_sequence: self.sample_sequence,
        }
    }

    fn fuel_count(&self) -> u64 {
        self.wood_count.saturating_add(self.oil_count)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TerminalReason {
    ReactionZero,
    MaxTicks,
}

impl TerminalReason {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ReactionZero => "reaction-zero",
            Self::MaxTicks => "max-ticks",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ReactionZeroUpdate {
    first_in_streak: bool,
    confirmed: bool,
    streak_broken: bool,
}

#[derive(Clone, Debug)]
struct ReactionZeroDetector {
    required: u32,
    streak: u32,
    first: Option<SampleIdentity>,
}

impl ReactionZeroDetector {
    fn new(required: u32) -> Self {
        Self {
            required,
            streak: 0,
            first: None,
        }
    }

    fn observe(&mut self, eligible: bool, metrics: &FireSampleMetrics) -> ReactionZeroUpdate {
        if !eligible || metrics.reaction_active_cells != 0 {
            let streak_broken = self.streak != 0;
            self.streak = 0;
            self.first = None;
            return ReactionZeroUpdate {
                streak_broken,
                ..ReactionZeroUpdate::default()
            };
        }
        let first_in_streak = self.streak == 0;
        if first_in_streak {
            self.first = Some(metrics.identity());
        }
        self.streak = self.streak.saturating_add(1);
        ReactionZeroUpdate {
            first_in_streak,
            confirmed: self.streak >= self.required,
            streak_broken: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ObservationUpdate {
    first_combustion: bool,
    first_smoke: bool,
    first_heat_propagation: bool,
    first_phase_transition: bool,
    substantial_fuel_consumption: bool,
    new_peak_reaction: bool,
    new_peak_thermal: bool,
}

#[derive(Clone, Debug)]
struct FireObservations {
    wood_combustion_observed: bool,
    oil_combustion_observed: bool,
    first_combustion: Option<SampleIdentity>,
    first_smoke: Option<SampleIdentity>,
    first_heat_propagation: Option<SampleIdentity>,
    first_phase_transition: Option<SampleIdentity>,
    substantial_fuel_consumption: Option<SampleIdentity>,
    peak_smoke_count: u64,
    peak_smoke: SampleIdentity,
    peak_reaction_cells: u64,
    peak_reaction: Option<SampleIdentity>,
    peak_thermal_cells: u64,
    peak_thermal: Option<SampleIdentity>,
    max_heat_propagated_cells: u64,
    invalid_material_occurrences: u64,
    nonfinite_field_occurrences: u64,
    latest: FireSampleMetrics,
}

impl FireObservations {
    fn new(tick0: &FireSampleMetrics) -> Self {
        Self {
            wood_combustion_observed: false,
            oil_combustion_observed: false,
            first_combustion: None,
            first_smoke: None,
            first_heat_propagation: None,
            first_phase_transition: None,
            substantial_fuel_consumption: None,
            peak_smoke_count: tick0.smoke_count,
            peak_smoke: tick0.identity(),
            peak_reaction_cells: 0,
            peak_reaction: None,
            peak_thermal_cells: 0,
            peak_thermal: None,
            max_heat_propagated_cells: 0,
            invalid_material_occurrences: tick0.invalid_material_count,
            nonfinite_field_occurrences: tick0
                .nonfinite_temperature_count
                .saturating_add(tick0.nonfinite_pressure_count),
            latest: tick0.clone(),
        }
    }

    fn observe(
        &mut self,
        metrics: &FireSampleMetrics,
        baseline: &FireBaseline,
        allow_combustion_milestone: bool,
    ) -> ObservationUpdate {
        self.invalid_material_occurrences = self
            .invalid_material_occurrences
            .saturating_add(metrics.invalid_material_count);
        self.nonfinite_field_occurrences = self
            .nonfinite_field_occurrences
            .saturating_add(metrics.nonfinite_temperature_count)
            .saturating_add(metrics.nonfinite_pressure_count);

        if allow_combustion_milestone {
            self.wood_combustion_observed |= metrics.flame_event_wood_cells != 0
                || metrics.wood_fuel_progress_sum > baseline.wood_fuel_progress_sum;
            self.oil_combustion_observed |= metrics.flame_event_oil_cells != 0
                || metrics.oil_fuel_progress_sum > baseline.oil_fuel_progress_sum;
        }
        let first_combustion = allow_combustion_milestone
            && self.wood_combustion_observed
            && self.oil_combustion_observed
            && self.first_combustion.is_none();
        if first_combustion {
            self.first_combustion = Some(metrics.identity());
        }
        let first_smoke = metrics.smoke_count > baseline.smoke_count && self.first_smoke.is_none();
        if first_smoke {
            self.first_smoke = Some(metrics.identity());
        }
        let first_heat_propagation =
            metrics.heat_propagated_cells != 0 && self.first_heat_propagation.is_none();
        if first_heat_propagation {
            self.first_heat_propagation = Some(metrics.identity());
        }
        let first_phase_transition =
            metrics.phase_inventory_changed && self.first_phase_transition.is_none();
        if first_phase_transition {
            self.first_phase_transition = Some(metrics.identity());
        }
        let substantial_fuel_consumption = metrics.fuel_count()
            <= baseline.substantial_fuel_remaining_threshold
            && self.substantial_fuel_consumption.is_none();
        if substantial_fuel_consumption {
            self.substantial_fuel_consumption = Some(metrics.identity());
        }
        if metrics.smoke_count > self.peak_smoke_count {
            self.peak_smoke_count = metrics.smoke_count;
            self.peak_smoke = metrics.identity();
        }
        let new_peak_reaction = metrics.reaction_active_cells > self.peak_reaction_cells;
        if new_peak_reaction {
            self.peak_reaction_cells = metrics.reaction_active_cells;
            self.peak_reaction = Some(metrics.identity());
        }
        let new_peak_thermal = metrics.thermal_active_cells > self.peak_thermal_cells;
        if new_peak_thermal {
            self.peak_thermal_cells = metrics.thermal_active_cells;
            self.peak_thermal = Some(metrics.identity());
        }
        self.max_heat_propagated_cells = self
            .max_heat_propagated_cells
            .max(metrics.heat_propagated_cells);
        self.latest = metrics.clone();

        ObservationUpdate {
            first_combustion,
            first_smoke,
            first_heat_propagation,
            first_phase_transition,
            substantial_fuel_consumption,
            new_peak_reaction,
            new_peak_thermal,
        }
    }
}

struct FireJsonlWriters {
    samples: BufWriter<File>,
    events: BufWriter<File>,
    event_sequence: u64,
}

impl FireJsonlWriters {
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
        metrics: &FireSampleMetrics,
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
                "\"run_id\":\"{}\",\"scenario\":\"fire-heat\",",
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
                "\"wood_count\":{},\"oil_count\":{},\"smoke_count\":{},",
                "\"ice_count\":{},\"water_count\":{},\"steam_count\":{},",
                "\"combusting_wood_cells\":{},\"combusting_oil_cells\":{},",
                "\"flame_event_wood_cells\":{},\"flame_event_oil_cells\":{},",
                "\"wood_fuel_progress_sum\":{},\"oil_fuel_progress_sum\":{},",
                "\"heat_propagated_cells\":{},\"phase_inventory_changed\":{},",
                "\"invalid_material_count\":{},",
                "\"nonfinite_temperature_count\":{},",
                "\"nonfinite_pressure_count\":{},\"changed_chunks\":{},",
                "\"wake_chunks\":{},\"wake_reason_or\":{},",
                "\"state_hash\":\"{}\",\"physical_state_hash\":\"{}\"}}"
            ),
            FIRE_TELEMETRY_SCHEMA_VERSION,
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
            metrics.wood_count,
            metrics.oil_count,
            metrics.smoke_count,
            metrics.ice_count,
            metrics.water_count,
            metrics.steam_count,
            metrics.combusting_wood_cells,
            metrics.combusting_oil_cells,
            metrics.flame_event_wood_cells,
            metrics.flame_event_oil_cells,
            metrics.wood_fuel_progress_sum,
            metrics.oil_fuel_progress_sum,
            metrics.heat_propagated_cells,
            metrics.phase_inventory_changed,
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
                "\"run_id\":\"{}\",\"scenario\":\"fire-heat\",",
                "\"event_sequence\":{},\"event\":\"{}\",",
                "\"sim_tick\":{},\"sample_sequence\":{},\"detail\":\"{}\"}}"
            ),
            FIRE_TELEMETRY_SCHEMA_VERSION,
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

fn fire_metrics_from_snapshot(
    snapshot: &GpuSnapshot,
    world: WorldConfig,
    baseline: Option<&FireBaseline>,
    sample_sequence: u64,
    sim_tick: u64,
    phase: &'static str,
    reason: &'static str,
) -> Result<FireSampleMetrics, String> {
    let expected_cells = u64::from(world.width) * u64::from(world.height);
    if snapshot.material_current.len() as u64 != expected_cells
        || snapshot.temperature_current.len() as u64 != expected_cells
        || snapshot.pressure_current.len() as u64 != expected_cells
        || snapshot.flags_current.len() as u64 != expected_cells
        || snapshot.cell_activity.len() as u64 != expected_cells
    {
        return Err("Fire GPU snapshot cell-vector lengths do not match WorldConfig".to_string());
    }
    if snapshot.chunk_activity.len() != snapshot.chunk_state.len()
        || snapshot.chunk_activity.len() != snapshot.chunk_changed.len()
        || snapshot.chunk_activity.len() != snapshot.chunk_wake_reason.len()
    {
        return Err("Fire GPU snapshot chunk-vector lengths disagree".to_string());
    }
    if baseline.is_some_and(|value| {
        value.initial_temperature_bits.len() != expected_cells as usize
            || value.initially_ambient.len() != expected_cells as usize
    }) {
        return Err("Fire baseline temperature vectors do not match WorldConfig".to_string());
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

    for (&material, &flags) in snapshot
        .material_current
        .iter()
        .zip(&snapshot.flags_current)
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
    let heat_propagated_cells = baseline.map_or(0, |value| {
        snapshot
            .temperature_current
            .iter()
            .zip(&value.initial_temperature_bits)
            .zip(&value.initially_ambient)
            .filter(|((current, initial), ambient)| {
                if !**ambient {
                    return false;
                }
                let current = f32::from_bits(**current);
                let initial = f32::from_bits(**initial);
                current.is_finite()
                    && initial.is_finite()
                    && (current - initial).abs() > THERMAL_ACTIVITY_EPS
            })
            .count() as u64
    });
    let wood_count = material_counts_by_id[MATERIAL_WOOD as usize];
    let oil_count = material_counts_by_id[MATERIAL_OIL as usize];
    let smoke_count = material_counts_by_id[MATERIAL_SMOKE as usize];
    let ice_count = material_counts_by_id[MATERIAL_ICE as usize];
    let water_count = material_counts_by_id[MATERIAL_WATER as usize];
    let steam_count = material_counts_by_id[MATERIAL_STEAM as usize];
    let phase_inventory_changed = baseline.is_some_and(|value| {
        (ice_count, water_count, steam_count)
            != (value.ice_count, value.water_count, value.steam_count)
    });

    Ok(FireSampleMetrics {
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
        wood_count,
        oil_count,
        smoke_count,
        ice_count,
        water_count,
        steam_count,
        combusting_wood_cells,
        combusting_oil_cells,
        flame_event_wood_cells,
        flame_event_oil_cells,
        wood_fuel_progress_sum,
        oil_fuel_progress_sum,
        heat_propagated_cells,
        phase_inventory_changed,
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

fn baseline_from_tick0(snapshot: &GpuSnapshot, metrics: &FireSampleMetrics) -> FireBaseline {
    let fuel_count = metrics.fuel_count();
    let substantial_fuel_consumption_threshold = fuel_count.saturating_add(3) / 4;
    FireBaseline {
        initial_temperature_bits: snapshot.temperature_current.clone(),
        initially_ambient: snapshot
            .temperature_current
            .iter()
            .map(|bits| {
                let value = f32::from_bits(*bits);
                value.is_finite() && (value - TEMPERATURE_REFERENCE).abs() <= THERMAL_ACTIVITY_EPS
            })
            .collect(),
        matter_count: metrics.matter_count,
        wood_count: metrics.wood_count,
        oil_count: metrics.oil_count,
        smoke_count: metrics.smoke_count,
        ice_count: metrics.ice_count,
        water_count: metrics.water_count,
        steam_count: metrics.steam_count,
        wood_fuel_progress_sum: metrics.wood_fuel_progress_sum,
        oil_fuel_progress_sum: metrics.oil_fuel_progress_sum,
        substantial_fuel_consumption_threshold,
        substantial_fuel_remaining_threshold: fuel_count
            .saturating_sub(substantial_fuel_consumption_threshold),
    }
}

fn capture_fire_frame(
    renderer: &mut Renderer,
    kind: &'static str,
    reason: &'static str,
    metrics: &FireSampleMetrics,
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
    if ring.len() == DIAGNOSTIC_RING_CAPACITY {
        let _ = ring.pop_front();
    }
    ring.push_back(frame.clone_with_kind("diagnostic-observation", "minimum-evidence-observation"));
}

#[allow(clippy::too_many_arguments)]
fn record_updates(
    output: &mut FireJsonlWriters,
    config: &super::ExperimentWorkerConfig,
    update: ObservationUpdate,
    metrics: &FireSampleMetrics,
    frame: &SemanticFrame,
    first_combustion_frame: &mut Option<SemanticFrame>,
    first_smoke_frame: &mut Option<SemanticFrame>,
    peak_reaction_frame: &mut Option<SemanticFrame>,
    peak_thermal_frame: &mut Option<SemanticFrame>,
    first_phase_frame: &mut Option<SemanticFrame>,
    substantial_fuel_frame: &mut Option<SemanticFrame>,
) -> Result<(), String> {
    if update.first_combustion {
        *first_combustion_frame =
            Some(frame.clone_with_kind("first-combustion", "both-fuels-production-combustion"));
        output.event(
            config,
            "combustion_observed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!(
                "wood_flame={};oil_flame={};wood_progress={};oil_progress={}",
                metrics.flame_event_wood_cells,
                metrics.flame_event_oil_cells,
                metrics.wood_fuel_progress_sum,
                metrics.oil_fuel_progress_sum
            ),
        )?;
    }
    if update.first_smoke {
        *first_smoke_frame = Some(frame.clone_with_kind("first-smoke", "smoke-count-above-tick0"));
        output.event(
            config,
            "smoke_generated",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!("smoke_count={}", metrics.smoke_count),
        )?;
    }
    if update.first_heat_propagation {
        output.event(
            config,
            "heat_propagated",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!("heat_propagated_cells={}", metrics.heat_propagated_cells),
        )?;
    }
    if update.first_phase_transition {
        *first_phase_frame = Some(frame.clone_with_kind(
            "first-phase-transition",
            "phase-inventory-differs-from-tick0",
        ));
        output.event(
            config,
            "phase_transition_observed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!(
                "ice={};water={};steam={}",
                metrics.ice_count, metrics.water_count, metrics.steam_count
            ),
        )?;
    }
    if update.substantial_fuel_consumption {
        *substantial_fuel_frame = Some(frame.clone_with_kind(
            "fuel-substantially-consumed",
            "at-least-25-percent-initial-fuel-consumed",
        ));
        output.event(
            config,
            "fuel_substantially_consumed",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!("fuel_remaining={}", metrics.fuel_count()),
        )?;
    }
    if update.new_peak_reaction {
        *peak_reaction_frame =
            Some(frame.clone_with_kind("peak-reaction", "highest-observed-reaction-cells"));
        output.event(
            config,
            "new_peak_reaction",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!("reaction_active_cells={}", metrics.reaction_active_cells),
        )?;
    }
    if update.new_peak_thermal {
        *peak_thermal_frame =
            Some(frame.clone_with_kind("peak-thermal", "highest-observed-thermal-cells"));
        output.event(
            config,
            "new_peak_thermal",
            metrics.sim_tick,
            Some(metrics.sample_sequence),
            &format!("thermal_active_cells={}", metrics.thermal_active_cells),
        )?;
    }
    Ok(())
}

fn assemble_frames(
    tick0: SemanticFrame,
    tick1: SemanticFrame,
    milestone_frames: [Option<SemanticFrame>; 8],
    terminal: SemanticFrame,
    reset: SemanticFrame,
    diagnostics: &VecDeque<SemanticFrame>,
) -> Result<Vec<SemanticFrame>, String> {
    let mut frames = Vec::with_capacity(MAX_RAW_FRAMES);
    frames.push(tick0);
    frames.push(tick1);
    for frame in milestone_frames.into_iter().flatten() {
        if frames.len() < MAX_RAW_FRAMES.saturating_sub(2) {
            frames.push(frame);
        }
    }
    frames.push(terminal);
    for frame in diagnostics {
        if frames.len() >= MIN_RAW_FRAMES.saturating_sub(1) {
            break;
        }
        frames.push(frame.clone());
    }
    frames.push(reset);
    if !(MIN_RAW_FRAMES..=MAX_RAW_FRAMES).contains(&frames.len()) {
        return Err(format!(
            "completed Fire lifecycle produced {} semantic frames; required {MIN_RAW_FRAMES}..={MAX_RAW_FRAMES}",
            frames.len()
        ));
    }
    Ok(frames)
}

fn validate_fire_worker_config(
    simulation: &Simulation,
    config: &super::ExperimentWorkerConfig,
) -> Result<(), String> {
    if config.experiment_id != FIRE_EXPERIMENT_ID {
        return Err(format!(
            "Fire experiment_id must be '{FIRE_EXPERIMENT_ID}', got '{}'",
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
    if config.scenario != ScenarioId::FireHeat {
        return Err(format!(
            "Fire experiment v0 supports only FireHeat, got {}",
            config.scenario
        ));
    }
    if simulation.world.config != REQUIRED_WORLD {
        return Err(format!(
            "Fire experiment v0 requires WorldConfig 256x256x64, got {}x{}x{}",
            simulation.world.config.width,
            simulation.world.config.height,
            simulation.world.config.chunk_size
        ));
    }
    if !simulation.sleep_enabled {
        return Err("Fire experiment v0 requires simulation sleep to be enabled".to_string());
    }
    if config.max_ticks != REQUIRED_MAX_TICKS {
        return Err(format!("Fire max_ticks must be {REQUIRED_MAX_TICKS}"));
    }
    if config.diagnostic_interval_ticks != REQUIRED_DIAGNOSTIC_INTERVAL_TICKS {
        return Err(format!(
            "Fire diagnostic_interval_ticks must be {REQUIRED_DIAGNOSTIC_INTERVAL_TICKS}"
        ));
    }
    if config.consecutive_reaction_zero != REQUIRED_REACTION_ZERO_SAMPLES {
        return Err(format!(
            "consecutive_reaction_zero must be {REQUIRED_REACTION_ZERO_SAMPLES}"
        ));
    }
    if config.post_reaction_ticks != REQUIRED_POST_REACTION_TICKS {
        return Err(format!(
            "post_reaction_ticks must be {REQUIRED_POST_REACTION_TICKS}"
        ));
    }
    if config.consecutive_all_sleep != 0
        || config.post_sleep_ticks != 0
        || super::pressure_lifecycle_options_present(
            config.consecutive_persistent_opening,
            config.post_opening_ticks,
            config.terminal_window_samples,
        )
    {
        return Err("Fire worker rejects Sand/Water/Pressure lifecycle settings".to_string());
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

#[derive(Clone, Debug)]
struct FirePredicates {
    combustion_observed: PredicateResult,
    smoke_generated: PredicateResult,
    heat_propagated: PredicateResult,
    phase_work_observed: PredicateResult,
    fuel_consumed: PredicateResult,
    reaction_terminated_before_max: PredicateResult,
    post_reaction_no_restart: PredicateResult,
    thermal_tail_observed: PredicateResult,
    thermal_tail_decreased: PredicateResult,
    no_invalid_materials: PredicateResult,
    no_nonfinite_fields: PredicateResult,
    exact_reset: PredicateResult,
}

impl FirePredicates {
    fn statuses(&self) -> [PredicateStatus; 12] {
        [
            self.combustion_observed.status,
            self.smoke_generated.status,
            self.heat_propagated.status,
            self.phase_work_observed.status,
            self.fuel_consumed.status,
            self.reaction_terminated_before_max.status,
            self.post_reaction_no_restart.status,
            self.thermal_tail_observed.status,
            self.thermal_tail_decreased.status,
            self.no_invalid_materials.status,
            self.no_nonfinite_fields.status,
            self.exact_reset.status,
        ]
    }

    fn verdict(&self) -> ExperimentVerdict {
        let statuses = self.statuses();
        if statuses.contains(&PredicateStatus::Fail) {
            ExperimentVerdict::Fail
        } else if statuses.contains(&PredicateStatus::Unknown) {
            ExperimentVerdict::NeedsHumanReview
        } else {
            ExperimentVerdict::Pass
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn build_predicates(
    baseline: &FireBaseline,
    observations: &FireObservations,
    confirmed_reaction_zero: Option<SampleIdentity>,
    max_ticks: u64,
    post_reaction_end_tick: Option<u64>,
    post_reaction_thermal_start: Option<u64>,
    post_reaction_thermal_min: Option<u64>,
    post_reaction_restart_samples: u32,
    exact_reset: bool,
) -> FirePredicates {
    let combustion_observed = observations.first_combustion.map_or_else(
        || PredicateResult::fail("Wood and Oil production combustion was not both observed"),
        |value| {
            PredicateResult::pass(format!(
                "Wood and Oil production combustion observed by tick {} sample {}",
                value.sim_tick, value.sample_sequence
            ))
        },
    );
    let smoke_generated = observations.first_smoke.map_or_else(
        || PredicateResult::fail("Smoke count never exceeded tick 0"),
        |value| PredicateResult::pass(format!("Smoke first observed at tick {}", value.sim_tick)),
    );
    let heat_propagated = observations.first_heat_propagation.map_or_else(
        || PredicateResult::fail("no initially ambient cell changed temperature"),
        |value| {
            PredicateResult::pass(format!(
                "heat entered initially ambient cells at tick {}",
                value.sim_tick
            ))
        },
    );
    let phase_work_observed = observations.first_phase_transition.map_or_else(
        || PredicateResult::fail("Ice/Water/Steam inventory never changed from tick 0"),
        |value| {
            PredicateResult::pass(format!(
                "phase inventory changed at tick {}",
                value.sim_tick
            ))
        },
    );
    let final_fuel = observations.latest.fuel_count();
    let fuel_consumed = if final_fuel < baseline.fuel_count() {
        PredicateResult::pass(format!(
            "finite fuel count decreased by {}",
            baseline.fuel_count() - final_fuel
        ))
    } else {
        PredicateResult::fail("Wood+Oil count did not decrease")
    };
    let reaction_terminated_before_max = confirmed_reaction_zero.map_or_else(
        || {
            PredicateResult::fail(format!(
                "reaction was not terminated before max {max_ticks}"
            ))
        },
        |value| {
            if value.sim_tick < max_ticks {
                PredicateResult::pass(format!(
                    "reaction-zero confirmed at tick {} before max {max_ticks}",
                    value.sim_tick
                ))
            } else {
                PredicateResult::fail(format!(
                    "reaction-zero confirmed at tick {}, not before max {max_ticks}",
                    value.sim_tick
                ))
            }
        },
    );
    let post_reaction_no_restart = match post_reaction_end_tick {
        Some(end) if post_reaction_restart_samples == 0 => PredicateResult::pass(format!(
            "post-reaction window ended at tick {end} without reaction restart"
        )),
        Some(end) => PredicateResult::fail(format!(
            "post-reaction window ended at tick {end} with {post_reaction_restart_samples} restart samples"
        )),
        None => PredicateResult::unknown("post-reaction window was unavailable"),
    };
    let thermal_tail_observed = match post_reaction_thermal_start {
        Some(value) if value > 0 => {
            PredicateResult::pass(format!("thermal tail started with {value} active cells"))
        }
        Some(_) => PredicateResult::unknown("reaction ended without a sampled Thermal tail"),
        None => PredicateResult::unknown("post-reaction Thermal tail was unavailable"),
    };
    let thermal_tail_decreased = match (post_reaction_thermal_start, post_reaction_thermal_min) {
        (Some(start), Some(minimum)) if start > 0 && minimum < start => PredicateResult::pass(
            format!("Thermal tail decreased from {start} to sampled minimum {minimum}"),
        ),
        (Some(start), Some(minimum)) => PredicateResult::unknown(format!(
            "Thermal tail did not show a sampled decrease: start={start}, minimum={minimum}"
        )),
        _ => PredicateResult::unknown("Thermal tail decrease evidence was unavailable"),
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
    FirePredicates {
        combustion_observed,
        smoke_generated,
        heat_propagated,
        phase_work_observed,
        fuel_consumed,
        reaction_terminated_before_max,
        post_reaction_no_restart,
        thermal_tail_observed,
        thermal_tail_decreased,
        no_invalid_materials,
        no_nonfinite_fields,
        exact_reset,
    }
}

fn write_frames_json(
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
            "\n  \"run_id\": \"{}\",\n  \"scenario\": \"fire-heat\",",
            "\n  \"binary_sha256\": \"{}\",\n  \"frame_count\": {},",
            "\n  \"pixel_encoding\": \"rgba8-tightly-packed\",",
            "\n  \"frames\": [{}]\n}}\n"
        ),
        FIRE_FRAMES_SCHEMA_VERSION,
        json_escape(&config.experiment_id),
        json_escape(&config.run_id),
        json_escape(&config.binary_sha256.to_ascii_lowercase()),
        frames.len(),
        entries,
    );
    write_new(path, json.as_bytes())
}

#[allow(clippy::too_many_arguments)]
fn write_analysis_json(
    config: &super::ExperimentWorkerConfig,
    provenance: &RuntimeProvenance,
    simulation: &Simulation,
    path: &Path,
    baseline: &FireBaseline,
    observations: &FireObservations,
    terminal_reason: TerminalReason,
    first_reaction_zero: Option<SampleIdentity>,
    confirmed_reaction_zero: Option<SampleIdentity>,
    post_reaction_end_tick: Option<u64>,
    post_reaction_thermal_start: Option<u64>,
    post_reaction_thermal_end: Option<u64>,
    post_reaction_thermal_min: Option<u64>,
    post_reaction_restart_samples: u32,
    predicates: &FirePredicates,
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
        predicate_json("combustion_observed", &predicates.combustion_observed),
        predicate_json("smoke_generated", &predicates.smoke_generated),
        predicate_json("heat_propagated", &predicates.heat_propagated),
        predicate_json("phase_work_observed", &predicates.phase_work_observed),
        predicate_json("fuel_consumed", &predicates.fuel_consumed),
        predicate_json(
            "reaction_terminated_before_max",
            &predicates.reaction_terminated_before_max,
        ),
        predicate_json(
            "post_reaction_no_restart",
            &predicates.post_reaction_no_restart,
        ),
        predicate_json("thermal_tail_observed", &predicates.thermal_tail_observed),
        predicate_json("thermal_tail_decreased", &predicates.thermal_tail_decreased),
        predicate_json("no_invalid_materials", &predicates.no_invalid_materials),
        predicate_json("no_nonfinite_fields", &predicates.no_nonfinite_fields),
        predicate_json("exact_reset", &predicates.exact_reset),
    ]
    .join(",");
    let identity_tick = |value: Option<SampleIdentity>| value.map(|item| item.sim_tick);
    let identity_sample = |value: Option<SampleIdentity>| value.map(|item| item.sample_sequence);
    let latest = &observations.latest;
    let final_fuel = latest.fuel_count();
    let fuel_consumed = baseline.fuel_count().saturating_sub(final_fuel);
    let thermal_decrease = matches!(
        (post_reaction_thermal_start, post_reaction_thermal_min),
        (Some(start), Some(minimum)) if minimum < start
    );
    let json = format!(
        concat!(
            "{{\n  \"schema_version\": \"{}\",\n  \"experiment_id\": \"{}\",",
            "\n  \"run_id\": \"{}\",\n  \"scenario\": \"fire-heat\",",
            "\n  \"binary_sha256\": \"{}\",",
            "\n  \"provenance\": {{\"source_sha\":\"{}\",\"git_state\":\"{}\",\"build_profile\":\"{}\"}},",
            "\n  \"world\": {{\"width\":{},\"height\":{},\"chunk_size\":{}}},",
            "\n  \"sleep\": {{\"enabled\":{},\"threshold\":{}}},",
            "\n  \"lifecycle\": {{\"max_ticks\":{},\"diagnostic_interval_ticks\":{},",
            "\"consecutive_reaction_zero_samples\":{},\"post_reaction_confirmation_ticks\":{},",
            "\"terminal_reason\":\"{}\",\"first_reaction_zero_sim_tick\":{},",
            "\"first_reaction_zero_sample_sequence\":{},",
            "\"confirmed_reaction_zero_sim_tick\":{},",
            "\"confirmed_reaction_zero_sample_sequence\":{},",
            "\"post_reaction_end_tick\":{},\"post_reaction_restart_samples\":{},",
            "\"sample_count\":{}}},",
            "\n  \"baseline\": {{\"matter_count\":{},\"wood_count\":{},\"oil_count\":{},",
            "\"smoke_count\":{},\"ice_count\":{},\"water_count\":{},\"steam_count\":{},",
            "\"fuel_count\":{},\"wood_fuel_progress_sum\":{},",
            "\"oil_fuel_progress_sum\":{},",
            "\"substantial_fuel_consumption_threshold\":{},",
            "\"substantial_fuel_remaining_threshold\":{}}},",
            "\n  \"metrics\": {{\"first_combustion_tick\":{},",
            "\"first_combustion_sample_sequence\":{},\"first_smoke_tick\":{},",
            "\"first_smoke_sample_sequence\":{},\"first_phase_transition_tick\":{},",
            "\"first_phase_transition_sample_sequence\":{},",
            "\"fuel_substantially_consumed_tick\":{},",
            "\"fuel_substantially_consumed_sample_sequence\":{},",
            "\"peak_smoke_count\":{},\"peak_smoke_tick\":{},",
            "\"peak_smoke_sample_sequence\":{},",
            "\"peak_reaction_cells\":{},\"peak_reaction_tick\":{},",
            "\"peak_reaction_sample_sequence\":{},\"peak_thermal_cells\":{},",
            "\"peak_thermal_tick\":{},\"peak_thermal_sample_sequence\":{},",
            "\"max_heat_propagated_cells\":{},\"reaction_zero_tick\":{},",
            "\"confirmed_reaction_zero_tick\":{},",
            "\"post_reaction_thermal_cells\":{},",
            "\"post_reaction_final_thermal_cells\":{},",
            "\"post_reaction_min_thermal_cells\":{},",
            "\"post_reaction_thermal_decrease\":{},",
            "\"post_reaction_reaction_restart_ticks\":{},",
            "\"post_reaction_restart_samples\":{},",
            "\"final_matter_count\":{},\"final_wood_count\":{},",
            "\"final_oil_count\":{},\"final_smoke_count\":{},",
            "\"final_ice_count\":{},\"final_water_count\":{},",
            "\"final_steam_count\":{},\"wood_count_delta\":{},",
            "\"oil_count_delta\":{},\"fuel_count_delta\":{},",
            "\"fuel_consumed\":{},\"invalid_material_occurrences\":{},",
            "\"nonfinite_field_occurrences\":{},\"reset_exact_equivalence\":{}}},",
            "\n  \"predicates\": {{{}}},",
            "\n  \"verdict\": \"{}\",\n  \"raw_frame_count\": {}\n}}\n"
        ),
        FIRE_ANALYSIS_SCHEMA_VERSION,
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
        config.consecutive_reaction_zero,
        config.post_reaction_ticks,
        terminal_reason.as_str(),
        json_opt_u64(identity_tick(first_reaction_zero)),
        json_opt_u64(identity_sample(first_reaction_zero)),
        json_opt_u64(identity_tick(confirmed_reaction_zero)),
        json_opt_u64(identity_sample(confirmed_reaction_zero)),
        json_opt_u64(post_reaction_end_tick),
        post_reaction_restart_samples,
        sample_count,
        baseline.matter_count,
        baseline.wood_count,
        baseline.oil_count,
        baseline.smoke_count,
        baseline.ice_count,
        baseline.water_count,
        baseline.steam_count,
        baseline.fuel_count(),
        baseline.wood_fuel_progress_sum,
        baseline.oil_fuel_progress_sum,
        baseline.substantial_fuel_consumption_threshold,
        baseline.substantial_fuel_remaining_threshold,
        json_opt_u64(identity_tick(observations.first_combustion)),
        json_opt_u64(identity_sample(observations.first_combustion)),
        json_opt_u64(identity_tick(observations.first_smoke)),
        json_opt_u64(identity_sample(observations.first_smoke)),
        json_opt_u64(identity_tick(observations.first_phase_transition)),
        json_opt_u64(identity_sample(observations.first_phase_transition)),
        json_opt_u64(identity_tick(observations.substantial_fuel_consumption)),
        json_opt_u64(identity_sample(observations.substantial_fuel_consumption)),
        observations.peak_smoke_count,
        observations.peak_smoke.sim_tick,
        observations.peak_smoke.sample_sequence,
        observations.peak_reaction_cells,
        json_opt_u64(identity_tick(observations.peak_reaction)),
        json_opt_u64(identity_sample(observations.peak_reaction)),
        observations.peak_thermal_cells,
        json_opt_u64(identity_tick(observations.peak_thermal)),
        json_opt_u64(identity_sample(observations.peak_thermal)),
        observations.max_heat_propagated_cells,
        json_opt_u64(identity_tick(first_reaction_zero)),
        json_opt_u64(identity_tick(confirmed_reaction_zero)),
        post_reaction_thermal_start.unwrap_or(0),
        post_reaction_thermal_end.unwrap_or(0),
        post_reaction_thermal_min.unwrap_or(0),
        thermal_decrease,
        post_reaction_restart_samples,
        post_reaction_restart_samples,
        latest.matter_count,
        latest.wood_count,
        latest.oil_count,
        latest.smoke_count,
        latest.ice_count,
        latest.water_count,
        latest.steam_count,
        i128::from(latest.wood_count) - i128::from(baseline.wood_count),
        i128::from(latest.oil_count) - i128::from(baseline.oil_count),
        i128::from(final_fuel) - i128::from(baseline.fuel_count()),
        fuel_consumed,
        observations.invalid_material_occurrences,
        observations.nonfinite_field_occurrences,
        exact_reset,
        predicates_json,
        verdict.as_str(),
        raw_frame_count,
    );
    write_new(path, json.as_bytes())
}

/// Runs the Fire / Heat experiment through production simulation ticks.
/// Semantic failures are returned as completed outcomes; operational failures
/// remain `Err` and therefore cannot be mistaken for a receipt-ready run.
pub fn run_fire_heat_experiment(
    simulation: &mut Simulation,
    renderer: &mut Renderer,
    provenance: &RuntimeProvenance,
    config: &super::ExperimentWorkerConfig,
) -> Result<ExperimentOutcome, String> {
    validate_fire_worker_config(simulation, config)?;

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
    let mut output = FireJsonlWriters::new(&samples_path, &events_path)?;
    output.event(
        config,
        "lifecycle_started",
        simulation.tick_count,
        None,
        "Fire worker output opened",
    )?;

    reset_and_stage_scenario(simulation, config.scenario)
        .map_err(|error| format!("pristine Fire / Heat reset/stage failed: {error}"))?;
    output.event(
        config,
        "pristine_reset_completed",
        0,
        None,
        "shared Fire / Heat reset/staging completed",
    )?;
    let baseline_sleep_enabled = simulation.sleep_enabled;
    let baseline_sleep_threshold = simulation.sleep_threshold;
    let mut next_sample_sequence = 0u64;
    let tick0_snapshot = capture_gpu_snapshot(simulation)?;
    let tick0_metrics = fire_metrics_from_snapshot(
        &tick0_snapshot,
        simulation.world.config,
        None,
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "initial",
        "tick0",
    )?;
    let baseline = baseline_from_tick0(&tick0_snapshot, &tick0_metrics);
    output.sample(config, provenance, simulation, &tick0_metrics)?;
    let tick0_frame = capture_fire_frame(renderer, "tick0", "pristine-reset", &tick0_metrics)?;
    output.event(
        config,
        "tick0_captured",
        tick0_metrics.sim_tick,
        Some(tick0_metrics.sample_sequence),
        &tick0_metrics.state_hash,
    )?;

    let mut observations = FireObservations::new(&tick0_metrics);
    let mut first_combustion_frame = None;
    let mut first_smoke_frame = None;
    let mut peak_reaction_frame = None;
    let mut peak_thermal_frame = None;
    let mut first_phase_frame = None;
    let mut substantial_fuel_frame = None;

    simulation
        .tick()
        .map_err(|error| format!("Fire production tick 1 failed: {error}"))?;
    let tick1_snapshot = capture_gpu_snapshot(simulation)?;
    let tick1_metrics = fire_metrics_from_snapshot(
        &tick1_snapshot,
        simulation.world.config,
        Some(&baseline),
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "reacting",
        "tick1",
    )?;
    let tick1_update = observations.observe(&tick1_metrics, &baseline, true);
    output.sample(config, provenance, simulation, &tick1_metrics)?;
    let tick1_frame = capture_fire_frame(
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
    record_updates(
        &mut output,
        config,
        tick1_update,
        &tick1_metrics,
        &tick1_frame,
        &mut first_combustion_frame,
        &mut first_smoke_frame,
        &mut peak_reaction_frame,
        &mut peak_thermal_frame,
        &mut first_phase_frame,
        &mut substantial_fuel_frame,
    )?;

    let mut diagnostics = VecDeque::with_capacity(DIAGNOSTIC_RING_CAPACITY);
    let mut zero_detector = ReactionZeroDetector::new(config.consecutive_reaction_zero);
    let mut reaction_zero_frame = None;
    let terminal_reason;
    let terminal_metrics;
    let terminal_frame;
    let first_reaction_zero;
    let confirmed_reaction_zero;
    loop {
        if simulation.tick_count >= config.max_ticks {
            return Err(
                "Fire lifecycle reached max tick without a max-tick diagnostic".to_string(),
            );
        }
        simulation.tick().map_err(|error| {
            format!(
                "Fire production tick {} failed: {error}",
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
            "early-diagnostic"
        } else if is_max {
            "max-tick"
        } else {
            "diagnostic-cadence"
        };
        let snapshot = capture_gpu_snapshot(simulation)?;
        let metrics = fire_metrics_from_snapshot(
            &snapshot,
            simulation.world.config,
            Some(&baseline),
            take_sequence(&mut next_sample_sequence),
            sim_tick,
            "reacting",
            reason,
        )?;
        let update = observations.observe(&metrics, &baseline, true);
        output.sample(config, provenance, simulation, &metrics)?;
        let frame = capture_fire_frame(renderer, "diagnostic", reason, &metrics)?;
        record_updates(
            &mut output,
            config,
            update,
            &metrics,
            &frame,
            &mut first_combustion_frame,
            &mut first_smoke_frame,
            &mut peak_reaction_frame,
            &mut peak_thermal_frame,
            &mut first_phase_frame,
            &mut substantial_fuel_frame,
        )?;
        remember_diagnostic(&mut diagnostics, &frame);

        let zero_update = zero_detector.observe(observations.first_combustion.is_some(), &metrics);
        if zero_update.streak_broken {
            reaction_zero_frame = None;
            output.event(
                config,
                "reaction_zero_streak_broken",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                "reaction activity became nonzero or combustion evidence was ineligible",
            )?;
        }
        if zero_update.first_in_streak {
            reaction_zero_frame = Some(frame.clone_with_kind(
                "reaction-zero",
                "first-sample-of-confirmed-reaction-zero-streak",
            ));
            output.event(
                config,
                "reaction_zero_streak_started",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                "first zero-reaction diagnostic in current streak",
            )?;
        }
        if zero_update.confirmed {
            let first = zero_detector.first.unwrap_or(metrics.identity());
            let confirmed = metrics.identity();
            output.event(
                config,
                "reaction_zero_confirmed",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                &format!(
                    "{} consecutive diagnostics; first_tick={};first_sample={}",
                    config.consecutive_reaction_zero, first.sim_tick, first.sample_sequence
                ),
            )?;
            terminal_reason = TerminalReason::ReactionZero;
            terminal_metrics = metrics.clone();
            terminal_frame = frame.clone_with_kind("terminal", "reaction-zero-confirmed");
            first_reaction_zero = Some(first);
            confirmed_reaction_zero = Some(confirmed);
            break;
        }
        if is_max {
            terminal_reason = TerminalReason::MaxTicks;
            terminal_metrics = metrics.clone();
            terminal_frame = frame.clone_with_kind("terminal", "max-tick-reached");
            first_reaction_zero = None;
            confirmed_reaction_zero = None;
            break;
        }
    }
    output.event(
        config,
        "terminal_selected",
        terminal_metrics.sim_tick,
        Some(terminal_metrics.sample_sequence),
        terminal_reason.as_str(),
    )?;

    let mut post_reaction_frame = None;
    let mut post_reaction_end_tick = None;
    let mut post_reaction_thermal_start = None;
    let mut post_reaction_thermal_end = None;
    let mut post_reaction_thermal_min = None;
    let mut post_reaction_restart_samples = 0u32;
    if terminal_reason == TerminalReason::ReactionZero {
        post_reaction_thermal_start = Some(terminal_metrics.thermal_active_cells);
        post_reaction_thermal_min = Some(terminal_metrics.thermal_active_cells);
        for offset in 1..=config.post_reaction_ticks {
            simulation.tick().map_err(|error| {
                format!(
                    "post-reaction production tick {offset}/{} failed: {error}",
                    config.post_reaction_ticks
                )
            })?;
            let snapshot = capture_gpu_snapshot(simulation)?;
            let metrics = fire_metrics_from_snapshot(
                &snapshot,
                simulation.world.config,
                Some(&baseline),
                take_sequence(&mut next_sample_sequence),
                simulation.tick_count,
                "post-reaction-confirmation",
                "post-reaction-tick",
            )?;
            let update = observations.observe(&metrics, &baseline, false);
            post_reaction_thermal_min = Some(
                post_reaction_thermal_min
                    .unwrap_or(metrics.thermal_active_cells)
                    .min(metrics.thermal_active_cells),
            );
            if metrics.reaction_active_cells != 0 {
                post_reaction_restart_samples = post_reaction_restart_samples.saturating_add(1);
            }
            output.sample(config, provenance, simulation, &metrics)?;
            let needs_frame = offset == config.post_reaction_ticks
                || update.first_smoke
                || update.first_heat_propagation
                || update.first_phase_transition
                || update.substantial_fuel_consumption
                || update.new_peak_reaction
                || update.new_peak_thermal;
            if needs_frame {
                let frame = capture_fire_frame(
                    renderer,
                    "post-reaction",
                    if offset == config.post_reaction_ticks {
                        "post-reaction-confirmation-complete"
                    } else {
                        "post-reaction-observation"
                    },
                    &metrics,
                )?;
                record_updates(
                    &mut output,
                    config,
                    update,
                    &metrics,
                    &frame,
                    &mut first_combustion_frame,
                    &mut first_smoke_frame,
                    &mut peak_reaction_frame,
                    &mut peak_thermal_frame,
                    &mut first_phase_frame,
                    &mut substantial_fuel_frame,
                )?;
                if offset == config.post_reaction_ticks {
                    post_reaction_frame = Some(frame.clone_with_kind(
                        "post-reaction-tail",
                        "post-reaction-confirmation-complete",
                    ));
                }
            }
            if offset == config.post_reaction_ticks {
                post_reaction_end_tick = Some(metrics.sim_tick);
                post_reaction_thermal_end = Some(metrics.thermal_active_cells);
            }
        }
        output.event(
            config,
            "post_reaction_confirmation_completed",
            simulation.tick_count,
            Some(next_sample_sequence.saturating_sub(1)),
            &format!(
                "ticks={};restart_samples={post_reaction_restart_samples};thermal_start={};thermal_end={}",
                config.post_reaction_ticks,
                post_reaction_thermal_start.unwrap_or(0),
                post_reaction_thermal_end.unwrap_or(0)
            ),
        )?;
    }

    output.event(
        config,
        "reset_started",
        simulation.tick_count,
        Some(next_sample_sequence.saturating_sub(1)),
        "programmatic R-equivalent shared Fire / Heat reset/staging",
    )?;
    reset_and_stage_scenario(simulation, config.scenario)
        .map_err(|error| format!("programmatic Fire / Heat reset failed: {error}"))?;
    let reset_snapshot = capture_gpu_snapshot(simulation)?;
    let reset_metrics = fire_metrics_from_snapshot(
        &reset_snapshot,
        simulation.world.config,
        Some(&baseline),
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "reset",
        "programmatic-r-equivalent",
    )?;
    output.sample(config, provenance, simulation, &reset_metrics)?;
    let reset_frame = capture_fire_frame(
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

    let predicates = build_predicates(
        &baseline,
        &observations,
        confirmed_reaction_zero,
        config.max_ticks,
        post_reaction_end_tick,
        post_reaction_thermal_start,
        post_reaction_thermal_min,
        post_reaction_restart_samples,
        exact_reset,
    );
    let verdict = predicates.verdict();
    let frames = assemble_frames(
        tick0_frame,
        tick1_frame,
        [
            first_combustion_frame,
            first_smoke_frame,
            peak_reaction_frame,
            peak_thermal_frame,
            first_phase_frame,
            substantial_fuel_frame,
            reaction_zero_frame,
            post_reaction_frame,
        ],
        terminal_frame,
        reset_frame,
        &diagnostics,
    )?;
    let written_frames = write_raw_frames(&raw_frames_dir, frames)?;
    write_frames_json(config, &frames_path, &written_frames)?;
    write_analysis_json(
        config,
        provenance,
        simulation,
        &analysis_path,
        &baseline,
        &observations,
        terminal_reason,
        first_reaction_zero,
        confirmed_reaction_zero,
        post_reaction_end_tick,
        post_reaction_thermal_start,
        post_reaction_thermal_end,
        post_reaction_thermal_min,
        post_reaction_restart_samples,
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
        first_all_sleep_sim_tick: None,
        first_all_sleep_sample_sequence: None,
        post_sleep_end_tick: post_reaction_end_tick,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(sim_tick: u64, reaction: u64) -> FireSampleMetrics {
        FireSampleMetrics {
            sample_sequence: sim_tick,
            sim_tick,
            phase: "reacting",
            reason: "test",
            total_cells: 1,
            any_active_cells: reaction,
            matter_active_cells: 0,
            thermal_active_cells: 4,
            pressure_active_cells: 0,
            reaction_active_cells: reaction,
            total_chunks: 1,
            active_chunks: u32::from(reaction != 0),
            runnable_chunks: 1,
            sleeping_chunks: 0,
            material_counts_by_id: [0; 10],
            matter_count: 10,
            wood_count: 4,
            oil_count: 4,
            smoke_count: 0,
            ice_count: 1,
            water_count: 1,
            steam_count: 0,
            combusting_wood_cells: 0,
            combusting_oil_cells: 0,
            flame_event_wood_cells: 0,
            flame_event_oil_cells: 0,
            wood_fuel_progress_sum: 0,
            oil_fuel_progress_sum: 0,
            heat_propagated_cells: 0,
            phase_inventory_changed: false,
            invalid_material_count: 0,
            nonfinite_temperature_count: 0,
            nonfinite_pressure_count: 0,
            changed_chunks: 0,
            wake_chunks: 0,
            wake_reason_or: 0,
            state_hash: format!("state-{sim_tick}"),
            physical_state_hash: format!("physical-{sim_tick}"),
        }
    }

    fn baseline() -> FireBaseline {
        FireBaseline {
            initial_temperature_bits: vec![TEMPERATURE_REFERENCE.to_bits()],
            initially_ambient: vec![true],
            matter_count: 10,
            wood_count: 4,
            oil_count: 4,
            smoke_count: 0,
            ice_count: 1,
            water_count: 1,
            steam_count: 0,
            wood_fuel_progress_sum: 0,
            oil_fuel_progress_sum: 0,
            substantial_fuel_consumption_threshold: 2,
            substantial_fuel_remaining_threshold: 6,
        }
    }

    #[test]
    fn reaction_zero_requires_three_eligible_diagnostics_and_keeps_first() {
        let mut detector = ReactionZeroDetector::new(3);
        assert!(detector.observe(true, &metrics(8, 0)).first_in_streak);
        assert!(!detector.observe(true, &metrics(16, 0)).confirmed);
        assert!(detector.observe(true, &metrics(24, 0)).confirmed);
        assert_eq!(detector.first, Some(metrics(8, 0).identity()));
        assert!(detector.observe(true, &metrics(32, 1)).streak_broken);
        assert_eq!(detector.first, None);
    }

    #[test]
    fn combustion_requires_post_tick_evidence_for_both_fuels() {
        let tick0 = metrics(0, 1);
        let baseline = baseline();
        let mut observations = FireObservations::new(&tick0);
        let mut wood = metrics(1, 1);
        wood.flame_event_wood_cells = 1;
        assert!(
            !observations
                .observe(&wood, &baseline, true)
                .first_combustion
        );
        let mut oil = metrics(2, 1);
        oil.oil_fuel_progress_sum = 1;
        assert!(observations.observe(&oil, &baseline, true).first_combustion);
        assert_eq!(observations.first_combustion, Some(oil.identity()));
    }

    #[test]
    fn substantial_fuel_threshold_is_fixed_at_ceiling_quarter() {
        let mut sample = metrics(0, 0);
        sample.wood_count = 5;
        sample.oil_count = 4;
        let snapshot = GpuSnapshot {
            material_current: vec![MATERIAL_WOOD],
            material_next: vec![MATERIAL_WOOD],
            temperature_current: vec![TEMPERATURE_REFERENCE.to_bits()],
            temperature_next: vec![TEMPERATURE_REFERENCE.to_bits()],
            pressure_current: vec![0.0f32.to_bits()],
            pressure_next: vec![0.0f32.to_bits()],
            flags_current: vec![0],
            flags_next: vec![0],
            proposal: vec![0],
            claim: vec![0],
            cell_activity: vec![0],
            chunk_activity: vec![0],
            chunk_changed: vec![0],
            chunk_stable: vec![0],
            chunk_edit_wake: vec![0],
            chunk_state: vec![0],
            chunk_wake_reason: vec![0],
            params: vec![0],
            wake_params: vec![0],
            arbitration_params: vec![0],
        };
        let value = baseline_from_tick0(&snapshot, &sample);
        assert_eq!(value.substantial_fuel_consumption_threshold, 3);
        assert_eq!(value.substantial_fuel_remaining_threshold, 6);
    }

    #[test]
    fn absent_or_flat_thermal_tail_is_unknown_not_fail() {
        let baseline = baseline();
        let tick0 = metrics(0, 1);
        let observations = FireObservations::new(&tick0);
        let absent = build_predicates(
            &baseline,
            &observations,
            None,
            20_000,
            None,
            None,
            None,
            0,
            true,
        );
        assert_eq!(
            absent.thermal_tail_observed.status,
            PredicateStatus::Unknown
        );
        assert_eq!(
            absent.thermal_tail_decreased.status,
            PredicateStatus::Unknown
        );
        let flat = build_predicates(
            &baseline,
            &observations,
            Some(SampleIdentity {
                sim_tick: 100,
                sample_sequence: 10,
            }),
            20_000,
            Some(280),
            Some(7),
            Some(7),
            0,
            true,
        );
        assert_eq!(flat.thermal_tail_observed.status, PredicateStatus::Pass);
        assert_eq!(flat.thermal_tail_decreased.status, PredicateStatus::Unknown);
    }
}
