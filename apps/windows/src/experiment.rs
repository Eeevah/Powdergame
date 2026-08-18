//! G8-B Experiment Evidence Harness workers.
//!
//! This module deliberately lives in the Windows application: it combines the
//! production simulation tick, the shared deterministic scenario staging path,
//! and the read-only renderer capture path. It does not change simulation
//! physics and it does not publish the final review packet or receipt; the
//! external experiment coordinator owns those steps.

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use powdergame_core::{
    is_valid_cell_material_value, WorldConfig, ACTIVITY_MATTER, ACTIVITY_PRESSURE,
    ACTIVITY_REACTION, ACTIVITY_THERMAL, CHUNK_STATE_RUNNABLE, CHUNK_STATE_SLEEPING,
    MATERIAL_EMPTY, MATERIAL_SAND,
};
use powdergame_gpu::Simulation;
use powdergame_scenarios::{reset_and_stage_scenario, ScenarioId};

use crate::gallery::RuntimeProvenance;
use crate::renderer::{CapturedFrame, Renderer};

mod fire;
mod pressure;
mod water;

pub use fire::{run_fire_heat_experiment, FIRE_EXPERIMENT_ID};
pub use pressure::{run_pressure_burst_experiment, PRESSURE_EXPERIMENT_ID};
pub use water::{run_water_flow_experiment, WATER_EXPERIMENT_ID};

pub const EXPERIMENT_ID: &str = "g8b-sand-fall-v0";
pub const TELEMETRY_SCHEMA_VERSION: &str = "powdergame-experiment-telemetry-v0";
pub const ANALYSIS_SCHEMA_VERSION: &str = "powdergame-experiment-analysis-v0";
pub const FRAMES_SCHEMA_VERSION: &str = "powdergame-experiment-frames-v0";
pub const REQUIRED_ALL_SLEEP_SAMPLES: u32 = 3;
pub const MIN_POST_SLEEP_TICKS: u32 = 120;
pub const MAX_POST_SLEEP_TICKS: u32 = 256;
pub const MIN_RAW_FRAMES: usize = 6;
pub const MAX_RAW_FRAMES: usize = 10;

const REQUIRED_WORLD: WorldConfig = WorldConfig {
    width: 256,
    height: 256,
    chunk_size: 64,
};

/// Validated input supplied by the external experiment coordinator.
#[derive(Clone, Debug)]
pub struct ExperimentWorkerConfig {
    pub experiment_id: String,
    pub run_id: String,
    pub run_dir: PathBuf,
    pub scenario: ScenarioId,
    pub binary_sha256: String,
    pub max_ticks: u64,
    pub diagnostic_interval_ticks: u64,
    pub consecutive_all_sleep: u32,
    pub post_sleep_ticks: u32,
    /// Fire / Heat-only diagnostic confirmation. Zero for sealed Sand/Water contracts.
    pub consecutive_reaction_zero: u32,
    /// Fire / Heat-only production tail. Zero for sealed Sand/Water contracts.
    pub post_reaction_ticks: u32,
    /// Pressure Burst-only opening confirmation. Zero for sealed Sand/Water/Fire contracts.
    pub consecutive_persistent_opening: u32,
    /// Pressure Burst-only production window after opening. Zero for sealed Sand/Water/Fire contracts.
    pub post_opening_ticks: u32,
    /// Pressure Burst-only terminal diagnostic window. Zero for sealed Sand/Water/Fire contracts.
    pub terminal_window_samples: u32,
}

fn pressure_lifecycle_options_present(
    consecutive_persistent_opening: u32,
    post_opening_ticks: u32,
    terminal_window_samples: u32,
) -> bool {
    consecutive_persistent_opening != 0 || post_opening_ticks != 0 || terminal_window_samples != 0
}

/// Hash and authenticate the executable image that the OS actually launched.
///
/// The external coordinator freezes the release EXE in the immutable run
/// directory and passes the frozen copy's digest. This check runs before any
/// EventLoop, renderer, GPU device, or simulation is created.
pub fn verify_current_executable_sha256(expected_sha256: &str) -> Result<(), String> {
    if expected_sha256.len() != 64 || !expected_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(
            "expected binary SHA-256 must contain exactly 64 hexadecimal characters".to_string(),
        );
    }
    let executable = std::env::current_exe()
        .map_err(|error| format!("cannot resolve current executable: {error}"))?;
    let bytes = fs::read(&executable).map_err(|error| {
        format!(
            "cannot read current executable {}: {error}",
            executable.display()
        )
    })?;
    let actual = hex_sha256(&bytes);
    if !actual.eq_ignore_ascii_case(expected_sha256) {
        return Err(format!(
            "current executable SHA-256 mismatch: expected {}, actual {} ({})",
            expected_sha256.to_ascii_lowercase(),
            actual,
            executable.display()
        ));
    }
    Ok(())
}

fn hex_sha256(input: &[u8]) -> String {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const ROUND: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    fn compress(state: &mut [u32; 8], block: &[u8]) {
        let mut words = [0u32; 64];
        for (index, chunk) in block.chunks_exact(4).take(16).enumerate() {
            words[index] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
        for index in 0..64 {
            let sigma1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choose = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(sigma1)
                .wrapping_add(choose)
                .wrapping_add(ROUND[index])
                .wrapping_add(words[index]);
            let sigma0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = sigma0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }

    let mut state = INITIAL;
    for block in input.chunks_exact(64) {
        compress(&mut state, block);
    }
    let remainder = input.chunks_exact(64).remainder();
    let bit_length = (input.len() as u64).wrapping_mul(8);
    let mut tail = Vec::with_capacity(128);
    tail.extend_from_slice(remainder);
    tail.push(0x80);
    while tail.len() % 64 != 56 {
        tail.push(0);
    }
    tail.extend_from_slice(&bit_length.to_be_bytes());
    for block in tail.chunks_exact(64) {
        compress(&mut state, block);
    }

    let mut output = String::with_capacity(64);
    for value in state {
        use std::fmt::Write as _;
        let _ = write!(output, "{value:08x}");
    }
    output
}

/// The semantic verdict produced by a scenario experiment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExperimentVerdict {
    Pass,
    Fail,
    NeedsHuman,
    NeedsHumanReview,
    FixtureCausalityConfounded,
}

impl ExperimentVerdict {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::NeedsHuman => "NEEDS_HUMAN",
            Self::NeedsHumanReview => "NEEDS_HUMAN_REVIEW",
            Self::FixtureCausalityConfounded => "FIXTURE_CAUSALITY_CONFOUNDED",
        }
    }
}

/// Completed worker output. Semantic failure is represented by `verdict`, not
/// by `Err`; `Err` is reserved for an incomplete configuration, GPU, renderer,
/// or filesystem operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExperimentOutcome {
    pub experiment_id: String,
    pub run_id: String,
    pub verdict: ExperimentVerdict,
    pub analysis_path: PathBuf,
    pub frames_path: PathBuf,
    pub samples_path: PathBuf,
    pub events_path: PathBuf,
    pub sample_count: u64,
    pub raw_frame_count: usize,
    pub first_all_sleep_sim_tick: Option<u64>,
    pub first_all_sleep_sample_sequence: Option<u64>,
    pub post_sleep_end_tick: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PredicateStatus {
    Pass,
    Fail,
    Unknown,
}

impl PredicateStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug)]
struct PredicateResult {
    status: PredicateStatus,
    detail: String,
}

impl PredicateResult {
    fn pass(detail: impl Into<String>) -> Self {
        Self {
            status: PredicateStatus::Pass,
            detail: detail.into(),
        }
    }

    fn fail(detail: impl Into<String>) -> Self {
        Self {
            status: PredicateStatus::Fail,
            detail: detail.into(),
        }
    }

    fn unknown(detail: impl Into<String>) -> Self {
        Self {
            status: PredicateStatus::Unknown,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug)]
struct HardPredicates {
    actual_fall: PredicateResult,
    matter_conservation: PredicateResult,
    no_invalid_materials: PredicateResult,
    no_nonfinite_fields: PredicateResult,
    sleep_before_max: PredicateResult,
    post_sleep_stable: PredicateResult,
    exact_reset: PredicateResult,
}

impl HardPredicates {
    fn statuses(&self) -> [PredicateStatus; 7] {
        [
            self.actual_fall.status,
            self.matter_conservation.status,
            self.no_invalid_materials.status,
            self.no_nonfinite_fields.status,
            self.sleep_before_max.status,
            self.post_sleep_stable.status,
            self.exact_reset.status,
        ]
    }

    fn verdict(&self) -> ExperimentVerdict {
        verdict_from_statuses(&self.statuses())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GpuSnapshot {
    material_current: Vec<u32>,
    material_next: Vec<u32>,
    temperature_current: Vec<u32>,
    temperature_next: Vec<u32>,
    pressure_current: Vec<u32>,
    pressure_next: Vec<u32>,
    flags_current: Vec<u32>,
    flags_next: Vec<u32>,
    proposal: Vec<u32>,
    claim: Vec<u32>,
    cell_activity: Vec<u32>,
    chunk_activity: Vec<u32>,
    chunk_changed: Vec<u32>,
    chunk_stable: Vec<u32>,
    chunk_edit_wake: Vec<u32>,
    chunk_state: Vec<u32>,
    chunk_wake_reason: Vec<u32>,
    params: Vec<u32>,
    wake_params: Vec<u32>,
    arbitration_params: Vec<u32>,
}

#[derive(Clone, Debug)]
struct SampleMetrics {
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
    sand_count: u64,
    sand_y_sum: u64,
    sand_min_y: Option<u32>,
    sand_max_y: Option<u32>,
    invalid_material_count: u64,
    nonfinite_temperature_count: u64,
    nonfinite_pressure_count: u64,
    changed_chunks: u32,
    wake_chunks: u32,
    wake_reason_or: u32,
    state_hash: String,
}

impl SampleMetrics {
    fn all_sleep(&self) -> bool {
        all_sleep_counts(
            self.any_active_cells,
            self.active_chunks,
            self.runnable_chunks,
            self.sleeping_chunks,
            self.total_chunks,
        )
    }
}

fn all_sleep_counts(
    any_active_cells: u64,
    active_chunks: u32,
    runnable_chunks: u32,
    sleeping_chunks: u32,
    total_chunks: u32,
) -> bool {
    any_active_cells == 0
        && active_chunks == 0
        && runnable_chunks == 0
        && sleeping_chunks == total_chunks
        && total_chunks != 0
}

#[derive(Clone, Debug)]
struct RawFrame {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl TryFrom<CapturedFrame> for RawFrame {
    type Error = String;

    fn try_from(frame: CapturedFrame) -> Result<Self, Self::Error> {
        let expected = u64::from(frame.width)
            .checked_mul(u64::from(frame.height))
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| "captured frame dimensions overflow byte count".to_string())?;
        if expected != frame.rgba.len() as u64 {
            return Err(format!(
                "captured frame byte count mismatch: {}x{} requires {expected}, got {}",
                frame.width,
                frame.height,
                frame.rgba.len()
            ));
        }
        Ok(Self {
            width: frame.width,
            height: frame.height,
            rgba: frame.rgba,
        })
    }
}

#[derive(Clone, Debug)]
struct SemanticFrame {
    kind: &'static str,
    reason: &'static str,
    sim_tick: u64,
    sample_sequence: u64,
    state_hash: String,
    frame: RawFrame,
}

#[derive(Clone, Debug)]
struct WrittenFrame {
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
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct DetectorUpdate {
    first_in_streak: bool,
    confirmed: bool,
    streak_broken: bool,
}

#[derive(Clone, Debug)]
struct AllSleepDetector {
    required: u32,
    streak: u32,
    first_sim_tick: Option<u64>,
    first_sample_sequence: Option<u64>,
}

impl AllSleepDetector {
    fn new(required: u32) -> Self {
        Self {
            required,
            streak: 0,
            first_sim_tick: None,
            first_sample_sequence: None,
        }
    }

    fn observe(&mut self, all_sleep: bool, sim_tick: u64, sample_sequence: u64) -> DetectorUpdate {
        if !all_sleep {
            let streak_broken = self.streak != 0;
            self.streak = 0;
            self.first_sim_tick = None;
            self.first_sample_sequence = None;
            return DetectorUpdate {
                streak_broken,
                ..DetectorUpdate::default()
            };
        }

        let first_in_streak = self.streak == 0;
        if first_in_streak {
            self.first_sim_tick = Some(sim_tick);
            self.first_sample_sequence = Some(sample_sequence);
        }
        self.streak = self.streak.saturating_add(1);
        DetectorUpdate {
            first_in_streak,
            confirmed: self.streak >= self.required,
            streak_broken: false,
        }
    }
}

struct JsonlWriters {
    samples: BufWriter<File>,
    events: BufWriter<File>,
    event_sequence: u64,
}

impl JsonlWriters {
    fn new(samples_path: &Path, events_path: &Path) -> Result<Self, String> {
        Ok(Self {
            samples: BufWriter::new(create_new_file(samples_path)?),
            events: BufWriter::new(create_new_file(events_path)?),
            event_sequence: 0,
        })
    }

    fn sample(
        &mut self,
        config: &ExperimentWorkerConfig,
        provenance: &RuntimeProvenance,
        simulation: &Simulation,
        metrics: &SampleMetrics,
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
                "\"run_id\":\"{}\",\"source_sha\":\"{}\",",
                "\"git_state\":\"{}\",\"build_profile\":\"{}\",",
                "\"binary_sha256\":\"{}\",",
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
                "\"sand_count\":{},\"sand_y_sum\":{},\"sand_min_y\":{},",
                "\"sand_max_y\":{},\"invalid_material_count\":{},",
                "\"nonfinite_temperature_count\":{},",
                "\"nonfinite_pressure_count\":{},\"changed_chunks\":{},",
                "\"wake_chunks\":{},\"wake_reason_or\":{},",
                "\"state_hash\":\"{}\"}}"
            ),
            TELEMETRY_SCHEMA_VERSION,
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
            metrics.sand_count,
            metrics.sand_y_sum,
            json_opt_u32(metrics.sand_min_y),
            json_opt_u32(metrics.sand_max_y),
            metrics.invalid_material_count,
            metrics.nonfinite_temperature_count,
            metrics.nonfinite_pressure_count,
            metrics.changed_chunks,
            metrics.wake_chunks,
            metrics.wake_reason_or,
            metrics.state_hash,
        )
        .map_err(|error| format!("write {} failed: {error}", display_path(&config.run_dir)))
    }

    fn event(
        &mut self,
        config: &ExperimentWorkerConfig,
        event: &str,
        sim_tick: u64,
        sample_sequence: Option<u64>,
        detail: &str,
    ) -> Result<(), String> {
        writeln!(
            self.events,
            concat!(
                "{{\"schema_version\":\"{}\",\"experiment_id\":\"{}\",",
                "\"run_id\":\"{}\",\"event_sequence\":{},\"event\":\"{}\",",
                "\"sim_tick\":{},\"sample_sequence\":{},\"detail\":\"{}\"}}"
            ),
            TELEMETRY_SCHEMA_VERSION,
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

/// Runs the Sand Fall experiment lifecycle. This path advances the simulation
/// only through the production `Simulation::tick` method.
pub fn run_sand_fall_experiment(
    simulation: &mut Simulation,
    renderer: &mut Renderer,
    provenance: &RuntimeProvenance,
    config: &ExperimentWorkerConfig,
) -> Result<ExperimentOutcome, String> {
    validate_worker_config(simulation, config)?;

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
    let mut output = JsonlWriters::new(&samples_path, &events_path)?;
    output.event(
        config,
        "lifecycle_started",
        simulation.tick_count,
        None,
        "worker output opened",
    )?;

    reset_and_stage_scenario(simulation, ScenarioId::SandFall)
        .map_err(|error| format!("pristine Sand Fall reset/stage failed: {error}"))?;
    output.event(
        config,
        "pristine_reset_completed",
        0,
        None,
        "shared Sand Fall reset/staging completed",
    )?;

    let mut next_sample_sequence = 0u64;
    let tick0_snapshot = capture_gpu_snapshot(simulation)?;
    let tick0_metrics = metrics_from_snapshot(
        &tick0_snapshot,
        simulation.world.config,
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "initial",
        "tick0",
    )?;
    output.sample(config, provenance, simulation, &tick0_metrics)?;
    let tick0_frame = capture_semantic_frame(renderer, "tick0", "pristine-reset", &tick0_metrics)?;
    output.event(
        config,
        "tick0_captured",
        tick0_metrics.sim_tick,
        Some(tick0_metrics.sample_sequence),
        &tick0_metrics.state_hash,
    )?;

    let baseline_matter_count = tick0_metrics.matter_count;
    let baseline_sand_count = tick0_metrics.sand_count;
    let baseline_sand_y_sum = tick0_metrics.sand_y_sum;
    let baseline_sleep_enabled = simulation.sleep_enabled;
    let baseline_sleep_threshold = simulation.sleep_threshold;
    let mut invariant_matter = true;
    let mut invalid_material_total = tick0_metrics.invalid_material_count;
    let mut nonfinite_field_total = tick0_metrics
        .nonfinite_temperature_count
        .saturating_add(tick0_metrics.nonfinite_pressure_count);
    let mut actual_fall_observed = false;

    simulation
        .tick()
        .map_err(|error| format!("production tick 1 failed: {error}"))?;
    let tick1_snapshot = capture_gpu_snapshot(simulation)?;
    let tick1_metrics = metrics_from_snapshot(
        &tick1_snapshot,
        simulation.world.config,
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "settling",
        "tick1",
    )?;
    update_invariants(
        &tick1_metrics,
        baseline_matter_count,
        baseline_sand_count,
        baseline_sand_y_sum,
        &mut invariant_matter,
        &mut invalid_material_total,
        &mut nonfinite_field_total,
        &mut actual_fall_observed,
    );
    output.sample(config, provenance, simulation, &tick1_metrics)?;
    let tick1_frame = capture_semantic_frame(
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
    if actual_fall_observed {
        output.event(
            config,
            "actual_fall_observed",
            tick1_metrics.sim_tick,
            Some(tick1_metrics.sample_sequence),
            "Sand y_sum exceeded pristine baseline",
        )?;
    }

    let mut all_sleep_detector = AllSleepDetector::new(config.consecutive_all_sleep);
    let (mut peak_active_count, mut peak_frame) =
        if tick0_metrics.any_active_cells >= tick1_metrics.any_active_cells {
            (
                tick0_metrics.any_active_cells,
                tick0_frame.clone_with_kind("peak-active", "highest-observed-active-cells"),
            )
        } else {
            (
                tick1_metrics.any_active_cells,
                tick1_frame.clone_with_kind("peak-active", "highest-observed-active-cells"),
            )
        };
    let mut peak_active_chunks = tick0_metrics.active_chunks.max(tick1_metrics.active_chunks);
    let mut first_sleeping_chunk_tick =
        (tick1_metrics.sleeping_chunks > 0).then_some(tick1_metrics.sim_tick);
    let mut first_sleeping_frame = (tick1_metrics.sleeping_chunks > 0).then(|| {
        tick1_frame.clone_with_kind("first-sleeping-chunk", "first-observed-sleeping-chunk")
    });
    if first_sleeping_frame.is_some() {
        output.event(
            config,
            "first_sleeping_chunk_observed",
            tick1_metrics.sim_tick,
            Some(tick1_metrics.sample_sequence),
            &format!("sleeping_chunks={}", tick1_metrics.sleeping_chunks),
        )?;
    }
    let mut latest_matter_count = tick1_metrics.matter_count;
    let mut final_sleeping_chunks = tick1_metrics.sleeping_chunks;
    let mut early_settling_frame: Option<SemanticFrame> = None;
    let mut previous_diagnostic_frame = Some(tick1_frame.clone());
    let mut all_sleep_candidate_frame: Option<SemanticFrame> = None;
    let mut late_candidate_frame: Option<SemanticFrame> = None;
    let mut first_all_sleep_frame: Option<SemanticFrame> = None;
    let mut late_settling_frame: Option<SemanticFrame> = None;
    let mut first_all_sleep_sim_tick = None;
    let mut first_all_sleep_sample_sequence = None;
    let mut confirmed_all_sleep_tick = None;
    let mut last_observed_frame =
        tick1_frame.clone_with_kind("final-observed", "last-pre-reset-observation");
    let mut actual_fall_event_written = actual_fall_observed;

    while simulation.tick_count < config.max_ticks && confirmed_all_sleep_tick.is_none() {
        simulation.tick().map_err(|error| {
            format!(
                "production tick {} failed: {error}",
                simulation.tick_count + 1
            )
        })?;
        let sim_tick = simulation.tick_count;
        let is_early = early_settling_frame.is_none();
        let is_cadence = sim_tick.is_multiple_of(config.diagnostic_interval_ticks);
        let is_max = sim_tick == config.max_ticks;
        if !is_early && !is_cadence && !is_max {
            continue;
        }

        let reason = if is_early {
            "early-settling"
        } else if is_max {
            "max-tick"
        } else {
            "diagnostic-cadence"
        };
        let snapshot = capture_gpu_snapshot(simulation)?;
        let metrics = metrics_from_snapshot(
            &snapshot,
            simulation.world.config,
            take_sequence(&mut next_sample_sequence),
            sim_tick,
            "settling",
            reason,
        )?;
        let previously_fell = actual_fall_observed;
        update_invariants(
            &metrics,
            baseline_matter_count,
            baseline_sand_count,
            baseline_sand_y_sum,
            &mut invariant_matter,
            &mut invalid_material_total,
            &mut nonfinite_field_total,
            &mut actual_fall_observed,
        );
        output.sample(config, provenance, simulation, &metrics)?;
        let frame = capture_semantic_frame(renderer, "diagnostic", reason, &metrics)?;
        latest_matter_count = metrics.matter_count;
        final_sleeping_chunks = metrics.sleeping_chunks;

        if is_early {
            early_settling_frame =
                Some(frame.clone_with_kind("early-settling", "early-settling-observation"));
        }
        if metrics.any_active_cells > peak_active_count {
            peak_active_count = metrics.any_active_cells;
            peak_frame = frame.clone_with_kind("peak-active", "highest-observed-active-cells");
            output.event(
                config,
                "new_peak_active",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                &format!("any_active_cells={peak_active_count}"),
            )?;
        }
        peak_active_chunks = peak_active_chunks.max(metrics.active_chunks);
        if first_sleeping_frame.is_none() && metrics.sleeping_chunks > 0 {
            first_sleeping_chunk_tick = Some(metrics.sim_tick);
            first_sleeping_frame = Some(
                frame.clone_with_kind("first-sleeping-chunk", "first-observed-sleeping-chunk"),
            );
            output.event(
                config,
                "first_sleeping_chunk_observed",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                &format!("sleeping_chunks={}", metrics.sleeping_chunks),
            )?;
        }
        if !previously_fell && actual_fall_observed && !actual_fall_event_written {
            actual_fall_event_written = true;
            output.event(
                config,
                "actual_fall_observed",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                "Sand y_sum exceeded pristine baseline",
            )?;
        }

        let detector_update = all_sleep_detector.observe(
            metrics.all_sleep(),
            metrics.sim_tick,
            metrics.sample_sequence,
        );
        if detector_update.streak_broken {
            all_sleep_candidate_frame = None;
            late_candidate_frame = None;
        }
        if detector_update.first_in_streak {
            late_candidate_frame = previous_diagnostic_frame.as_ref().map(|previous| {
                previous.clone_with_kind("late-settling", "observation-before-all-sleep-streak")
            });
            all_sleep_candidate_frame = Some(frame.clone_with_kind(
                "first-all-sleep",
                "first-observed-all-sleep-in-confirmed-streak",
            ));
            output.event(
                config,
                "all_sleep_observed",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                "first all-sleep sample in current streak",
            )?;
        }
        if detector_update.confirmed {
            first_all_sleep_sim_tick = all_sleep_detector.first_sim_tick;
            first_all_sleep_sample_sequence = all_sleep_detector.first_sample_sequence;
            confirmed_all_sleep_tick = Some(metrics.sim_tick);
            first_all_sleep_frame = all_sleep_candidate_frame.take();
            late_settling_frame = late_candidate_frame.take();
            output.event(
                config,
                "all_sleep_confirmed",
                metrics.sim_tick,
                Some(metrics.sample_sequence),
                &format!(
                    "{} consecutive diagnostic samples; first observed sim_tick={} sample_sequence={}",
                    config.consecutive_all_sleep,
                    first_all_sleep_sim_tick.unwrap_or(metrics.sim_tick),
                    first_all_sleep_sample_sequence.unwrap_or(metrics.sample_sequence)
                ),
            )?;
        }

        previous_diagnostic_frame = Some(frame.clone());
        last_observed_frame = frame.clone_with_kind("final-observed", "last-pre-reset-observation");
    }

    let mut post_sleep_frame = None;
    let mut post_sleep_end_tick = None;
    let mut post_sleep_change_ticks = 0u32;
    let mut post_sleep_wake_ticks = 0u32;
    if confirmed_all_sleep_tick.is_some() {
        let full_sleep_snapshot = capture_gpu_snapshot(simulation)?;
        for offset in 1..=config.post_sleep_ticks {
            simulation.tick().map_err(|error| {
                format!(
                    "post-sleep production tick {offset}/{} failed: {error}",
                    config.post_sleep_ticks
                )
            })?;
            let snapshot = capture_gpu_snapshot(simulation)?;
            let metrics = metrics_from_snapshot(
                &snapshot,
                simulation.world.config,
                take_sequence(&mut next_sample_sequence),
                simulation.tick_count,
                "post-sleep-confirmation",
                "post-sleep-tick",
            )?;
            update_invariants(
                &metrics,
                baseline_matter_count,
                baseline_sand_count,
                baseline_sand_y_sum,
                &mut invariant_matter,
                &mut invalid_material_total,
                &mut nonfinite_field_total,
                &mut actual_fall_observed,
            );
            if !physical_tick_boundary_equal(&full_sleep_snapshot, &snapshot)
                || metrics.changed_chunks != 0
            {
                post_sleep_change_ticks = post_sleep_change_ticks.saturating_add(1);
            }
            if metrics.wake_chunks != 0
                || metrics.any_active_cells != 0
                || metrics.active_chunks != 0
                || metrics.runnable_chunks != 0
                || metrics.sleeping_chunks != metrics.total_chunks
            {
                post_sleep_wake_ticks = post_sleep_wake_ticks.saturating_add(1);
            }
            output.sample(config, provenance, simulation, &metrics)?;
            latest_matter_count = metrics.matter_count;
            final_sleeping_chunks = metrics.sleeping_chunks;
            if offset == config.post_sleep_ticks {
                let frame = capture_semantic_frame(
                    renderer,
                    "post-sleep-confirmation",
                    "post-sleep-confirmation-complete",
                    &metrics,
                )?;
                last_observed_frame =
                    frame.clone_with_kind("final-observed", "last-pre-reset-observation");
                post_sleep_frame = Some(frame);
                post_sleep_end_tick = Some(metrics.sim_tick);
            }
        }
        output.event(
            config,
            "post_sleep_confirmation_completed",
            simulation.tick_count,
            Some(next_sample_sequence.saturating_sub(1)),
            &format!(
                "ticks={}; state_change_ticks={post_sleep_change_ticks}; wake_ticks={post_sleep_wake_ticks}",
                config.post_sleep_ticks
            ),
        )?;
    }

    output.event(
        config,
        "reset_started",
        simulation.tick_count,
        Some(next_sample_sequence.saturating_sub(1)),
        "programmatic R-equivalent shared reset/staging",
    )?;
    reset_and_stage_scenario(simulation, ScenarioId::SandFall)
        .map_err(|error| format!("programmatic R-equivalent reset failed: {error}"))?;
    let reset_snapshot = capture_gpu_snapshot(simulation)?;
    let reset_metrics = metrics_from_snapshot(
        &reset_snapshot,
        simulation.world.config,
        take_sequence(&mut next_sample_sequence),
        simulation.tick_count,
        "reset",
        "programmatic-r-equivalent",
    )?;
    output.sample(config, provenance, simulation, &reset_metrics)?;
    let reset_frame = capture_semantic_frame(
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

    let hard_predicates = build_hard_predicates(
        actual_fall_observed,
        invariant_matter,
        invalid_material_total,
        nonfinite_field_total,
        confirmed_all_sleep_tick,
        config.max_ticks,
        post_sleep_end_tick,
        post_sleep_change_ticks,
        post_sleep_wake_ticks,
        exact_reset,
    );
    let verdict = hard_predicates.verdict();

    let mut semantic_frames = vec![tick0_frame, tick1_frame];
    if let Some(frame) = early_settling_frame {
        semantic_frames.push(frame);
    }
    semantic_frames.push(peak_frame);
    if let Some(frame) = first_sleeping_frame {
        semantic_frames.push(frame);
    }
    if let Some(frame) = late_settling_frame {
        semantic_frames.push(frame);
    }
    if let Some(frame) = first_all_sleep_frame {
        semantic_frames.push(frame);
    }
    if let Some(frame) = post_sleep_frame {
        semantic_frames.push(frame);
    } else {
        semantic_frames.push(last_observed_frame);
    }
    semantic_frames.push(reset_frame);
    if semantic_frames.len() > MAX_RAW_FRAMES {
        semantic_frames.truncate(MAX_RAW_FRAMES);
    }
    if semantic_frames.len() < MIN_RAW_FRAMES {
        return Err(format!(
            "completed lifecycle produced {} semantic frames; required {MIN_RAW_FRAMES}..={MAX_RAW_FRAMES}",
            semantic_frames.len()
        ));
    }

    let written_frames = write_raw_frames(&raw_frames_dir, semantic_frames)?;
    write_frames_json(config, &frames_path, &written_frames)?;
    write_analysis_json(
        config,
        provenance,
        simulation,
        &analysis_path,
        &hard_predicates,
        verdict,
        next_sample_sequence,
        written_frames.len(),
        first_all_sleep_sim_tick,
        first_all_sleep_sample_sequence,
        confirmed_all_sleep_tick,
        post_sleep_end_tick,
        post_sleep_change_ticks,
        post_sleep_wake_ticks,
        baseline_matter_count,
        baseline_sand_count,
        baseline_sand_y_sum,
        peak_active_count,
        peak_active_chunks,
        first_sleeping_chunk_tick,
        final_sleeping_chunks,
        i128::from(latest_matter_count) - i128::from(baseline_matter_count),
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
        first_all_sleep_sim_tick,
        first_all_sleep_sample_sequence,
        post_sleep_end_tick,
    })
}

impl SemanticFrame {
    fn clone_with_kind(&self, kind: &'static str, reason: &'static str) -> Self {
        Self {
            kind,
            reason,
            sim_tick: self.sim_tick,
            sample_sequence: self.sample_sequence,
            state_hash: self.state_hash.clone(),
            frame: self.frame.clone(),
        }
    }
}

fn validate_worker_config(
    simulation: &Simulation,
    config: &ExperimentWorkerConfig,
) -> Result<(), String> {
    if config.experiment_id != EXPERIMENT_ID {
        return Err(format!(
            "experiment_id must be '{EXPERIMENT_ID}', got '{}'",
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
    if config.scenario != ScenarioId::SandFall {
        return Err(format!(
            "experiment v0 supports only SandFall, got {}",
            config.scenario
        ));
    }
    if simulation.world.config != REQUIRED_WORLD {
        return Err(format!(
            "experiment v0 requires WorldConfig 256x256x64, got {}x{}x{}",
            simulation.world.config.width,
            simulation.world.config.height,
            simulation.world.config.chunk_size
        ));
    }
    if !simulation.sleep_enabled {
        return Err("experiment v0 requires simulation sleep to be enabled".to_string());
    }
    if config.max_ticks < 3 {
        return Err("max_ticks must be at least 3".to_string());
    }
    if config.diagnostic_interval_ticks == 0 {
        return Err("diagnostic_interval_ticks must be greater than zero".to_string());
    }
    if config.consecutive_all_sleep != REQUIRED_ALL_SLEEP_SAMPLES {
        return Err(format!(
            "consecutive_all_sleep must be {REQUIRED_ALL_SLEEP_SAMPLES}"
        ));
    }
    if !(MIN_POST_SLEEP_TICKS..=MAX_POST_SLEEP_TICKS).contains(&config.post_sleep_ticks) {
        return Err(format!(
            "post_sleep_ticks must be in {MIN_POST_SLEEP_TICKS}..={MAX_POST_SLEEP_TICKS}"
        ));
    }
    if config.consecutive_reaction_zero != 0
        || config.post_reaction_ticks != 0
        || pressure_lifecycle_options_present(
            config.consecutive_persistent_opening,
            config.post_opening_ticks,
            config.terminal_window_samples,
        )
    {
        return Err("Sand worker rejects Fire/Pressure lifecycle settings".to_string());
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

fn create_worker_directory(path: &Path) -> Result<(), String> {
    fs::create_dir(path).map_err(|error| {
        format!(
            "create-new worker directory {} failed: {error}",
            display_path(path)
        )
    })
}

fn create_new_file(path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("create-new {} failed: {error}", display_path(path)))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = create_new_file(path)?;
    file.write_all(bytes)
        .map_err(|error| format!("write {} failed: {error}", display_path(path)))?;
    file.sync_all()
        .map_err(|error| format!("sync {} failed: {error}", display_path(path)))
}

fn capture_semantic_frame(
    renderer: &mut Renderer,
    kind: &'static str,
    reason: &'static str,
    metrics: &SampleMetrics,
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

fn capture_gpu_snapshot(simulation: &Simulation) -> Result<GpuSnapshot, String> {
    let cell_bytes = simulation.world.layout.material_bytes;
    let chunk_count = powdergame_core::chunk_count(
        simulation.world.config.width,
        simulation.world.config.height,
        simulation.world.config.chunk_size,
    ) as u64;
    let chunk_bytes = chunk_count
        .checked_mul(4)
        .ok_or_else(|| "chunk snapshot byte count overflow".to_string())?;

    let params_bytes = simulation.params.size();
    let wake_params_bytes = simulation.wake_params.size();
    let arbitration_params_bytes = simulation.arbitration_params.size();
    let sources: [(&wgpu::Buffer, u64); 20] = [
        (&simulation.world.material_current, cell_bytes),
        (&simulation.world.material_next, cell_bytes),
        (&simulation.world.temperature_current, cell_bytes),
        (&simulation.world.temperature_next, cell_bytes),
        (&simulation.world.pressure_current, cell_bytes),
        (&simulation.world.pressure_next, cell_bytes),
        (&simulation.world.flags_current, cell_bytes),
        (&simulation.world.flags_next, cell_bytes),
        (&simulation.world.proposal, cell_bytes),
        (&simulation.world.claim, cell_bytes),
        (&simulation.world.cell_activity, cell_bytes),
        (&simulation.world.chunk_activity, chunk_bytes),
        (&simulation.world.chunk_changed_this_tick, chunk_bytes),
        (&simulation.world.chunk_stable_ticks, chunk_bytes),
        (&simulation.world.chunk_edit_wake, chunk_bytes),
        (&simulation.world.chunk_state, chunk_bytes),
        (&simulation.world.chunk_wake_reason, chunk_bytes),
        (&simulation.params, params_bytes),
        (&simulation.wake_params, wake_params_bytes),
        (&simulation.arbitration_params, arbitration_params_bytes),
    ];
    let total_bytes = sources.iter().try_fold(0u64, |sum, (_, size)| {
        sum.checked_add(*size)
            .ok_or_else(|| "combined snapshot byte count overflow".to_string())
    })?;
    let staging = simulation
        .context
        .device
        .create_buffer(&wgpu::BufferDescriptor {
            label: Some("experiment/combined-readback-staging"),
            size: total_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
    let mut encoder =
        simulation
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("experiment/combined-readback-encoder"),
            });
    let mut offset = 0u64;
    for (source, size) in &sources {
        encoder.copy_buffer_to_buffer(source, 0, &staging, offset, *size);
        offset += *size;
    }
    simulation.context.queue.submit([encoder.finish()]);
    simulation
        .context
        .device
        .poll(wgpu::PollType::Wait)
        .map_err(|error| format!("GPU wait before experiment readback failed: {error}"))?;

    let slice = staging.slice(..);
    let (tx, rx) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    simulation
        .context
        .device
        .poll(wgpu::PollType::Wait)
        .map_err(|error| format!("GPU wait during experiment readback failed: {error}"))?;
    rx.recv()
        .map_err(|error| format!("experiment map callback lost: {error}"))?
        .map_err(|error| format!("experiment buffer map failed: {error}"))?;
    let mapped = slice.get_mapped_range();
    let mut cursor = 0usize;
    let cell_len = usize::try_from(cell_bytes).map_err(|_| "cell byte count exceeds usize")?;
    let chunk_len = usize::try_from(chunk_bytes).map_err(|_| "chunk byte count exceeds usize")?;
    let params_len =
        usize::try_from(params_bytes).map_err(|_| "params byte count exceeds usize")?;
    let wake_params_len =
        usize::try_from(wake_params_bytes).map_err(|_| "wake params byte count exceeds usize")?;
    let arbitration_params_len = usize::try_from(arbitration_params_bytes)
        .map_err(|_| "arbitration params byte count exceeds usize")?;
    let mut take = |size: usize| -> Result<Vec<u32>, String> {
        let end = cursor
            .checked_add(size)
            .ok_or_else(|| "snapshot cursor overflow".to_string())?;
        let bytes = mapped
            .get(cursor..end)
            .ok_or_else(|| "snapshot staging buffer shorter than declared layout".to_string())?;
        if bytes.len() % 4 != 0 {
            return Err("snapshot segment is not u32 aligned".to_string());
        }
        cursor = end;
        Ok(bytes
            .chunks_exact(4)
            .map(|chunk| u32::from_ne_bytes(chunk.try_into().expect("four-byte chunk")))
            .collect())
    };
    let snapshot = GpuSnapshot {
        material_current: take(cell_len)?,
        material_next: take(cell_len)?,
        temperature_current: take(cell_len)?,
        temperature_next: take(cell_len)?,
        pressure_current: take(cell_len)?,
        pressure_next: take(cell_len)?,
        flags_current: take(cell_len)?,
        flags_next: take(cell_len)?,
        proposal: take(cell_len)?,
        claim: take(cell_len)?,
        cell_activity: take(cell_len)?,
        chunk_activity: take(chunk_len)?,
        chunk_changed: take(chunk_len)?,
        chunk_stable: take(chunk_len)?,
        chunk_edit_wake: take(chunk_len)?,
        chunk_state: take(chunk_len)?,
        chunk_wake_reason: take(chunk_len)?,
        params: take(params_len)?,
        wake_params: take(wake_params_len)?,
        arbitration_params: take(arbitration_params_len)?,
    };
    if cursor != mapped.len() {
        return Err(format!(
            "snapshot parser consumed {cursor} of {} mapped bytes",
            mapped.len()
        ));
    }
    drop(mapped);
    staging.unmap();
    Ok(snapshot)
}

fn metrics_from_snapshot(
    snapshot: &GpuSnapshot,
    world: WorldConfig,
    sample_sequence: u64,
    sim_tick: u64,
    phase: &'static str,
    reason: &'static str,
) -> Result<SampleMetrics, String> {
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

    let mut material_counts_by_id = [0u64; 10];
    let mut matter_count = 0u64;
    let mut sand_count = 0u64;
    let mut sand_y_sum = 0u64;
    let mut sand_min_y = None;
    let mut sand_max_y = None;
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
        if material == MATERIAL_SAND {
            sand_count = sand_count.saturating_add(1);
            let y = (index as u64 / u64::from(world.width)) as u32;
            sand_y_sum = sand_y_sum.saturating_add(u64::from(y));
            sand_min_y = Some(sand_min_y.map_or(y, |old: u32| old.min(y)));
            sand_max_y = Some(sand_max_y.map_or(y, |old: u32| old.max(y)));
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
    let any_active_cells = snapshot
        .cell_activity
        .iter()
        .filter(|&&value| value != 0)
        .count() as u64;
    let matter_active_cells = bit_count(&snapshot.cell_activity, ACTIVITY_MATTER);
    let thermal_active_cells = bit_count(&snapshot.cell_activity, ACTIVITY_THERMAL);
    let pressure_active_cells = bit_count(&snapshot.cell_activity, ACTIVITY_PRESSURE);
    let reaction_active_cells = bit_count(&snapshot.cell_activity, ACTIVITY_REACTION);
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
    let wake_reason_or = snapshot
        .chunk_wake_reason
        .iter()
        .copied()
        .fold(0u32, |acc, value| acc | value);

    Ok(SampleMetrics {
        sample_sequence,
        sim_tick,
        phase,
        reason,
        total_cells: expected_cells,
        any_active_cells,
        matter_active_cells,
        thermal_active_cells,
        pressure_active_cells,
        reaction_active_cells,
        total_chunks: snapshot.chunk_activity.len() as u32,
        active_chunks,
        runnable_chunks,
        sleeping_chunks,
        material_counts_by_id,
        matter_count,
        sand_count,
        sand_y_sum,
        sand_min_y,
        sand_max_y,
        invalid_material_count,
        nonfinite_temperature_count,
        nonfinite_pressure_count,
        changed_chunks,
        wake_chunks,
        wake_reason_or,
        state_hash: authoritative_current_hash(snapshot),
    })
}

#[allow(clippy::too_many_arguments)]
fn update_invariants(
    metrics: &SampleMetrics,
    baseline_matter_count: u64,
    baseline_sand_count: u64,
    baseline_sand_y_sum: u64,
    invariant_matter: &mut bool,
    invalid_material_total: &mut u64,
    nonfinite_field_total: &mut u64,
    actual_fall_observed: &mut bool,
) {
    *invariant_matter &= metrics.matter_count == baseline_matter_count;
    *invalid_material_total = invalid_material_total.saturating_add(metrics.invalid_material_count);
    *nonfinite_field_total = nonfinite_field_total
        .saturating_add(metrics.nonfinite_temperature_count)
        .saturating_add(metrics.nonfinite_pressure_count);
    *actual_fall_observed |=
        metrics.sand_count == baseline_sand_count && metrics.sand_y_sum > baseline_sand_y_sum;
}

#[allow(clippy::too_many_arguments)]
fn build_hard_predicates(
    actual_fall_observed: bool,
    invariant_matter: bool,
    invalid_material_total: u64,
    nonfinite_field_total: u64,
    confirmed_all_sleep_tick: Option<u64>,
    max_ticks: u64,
    post_sleep_end_tick: Option<u64>,
    post_sleep_change_ticks: u32,
    post_sleep_wake_ticks: u32,
    exact_reset: bool,
) -> HardPredicates {
    let actual_fall = if actual_fall_observed {
        PredicateResult::pass("Sand y_sum increased from the tick-0 baseline")
    } else {
        PredicateResult::fail("no sampled state showed a Sand y_sum increase")
    };
    let matter_conservation = if invariant_matter {
        PredicateResult::pass("registered non-empty Matter count matched tick 0 in every sample")
    } else {
        PredicateResult::fail("registered non-empty Matter count differed from tick 0")
    };
    let no_invalid_materials = if invalid_material_total == 0 {
        PredicateResult::pass("invalid material count was zero in every sample")
    } else {
        PredicateResult::fail(format!(
            "sampled invalid material occurrences={invalid_material_total}"
        ))
    };
    let no_nonfinite_fields = if nonfinite_field_total == 0 {
        PredicateResult::pass("non-finite temperature/pressure count was zero in every sample")
    } else {
        PredicateResult::fail(format!(
            "sampled non-finite temperature/pressure occurrences={nonfinite_field_total}"
        ))
    };
    let sleep_before_max = match confirmed_all_sleep_tick {
        Some(tick) if tick < max_ticks => PredicateResult::pass(format!(
            "three-sample all-sleep confirmation completed at sim tick {tick} before max {max_ticks}"
        )),
        Some(tick) => PredicateResult::fail(format!(
            "all-sleep confirmation completed at sim tick {tick}, not before max {max_ticks}"
        )),
        None => PredicateResult::fail(format!(
            "all-sleep was not confirmed before max tick {max_ticks}"
        )),
    };
    let post_sleep_stable = match post_sleep_end_tick {
        Some(end_tick) if post_sleep_change_ticks == 0 && post_sleep_wake_ticks == 0 => {
            PredicateResult::pass(format!(
                "post-sleep window ended at tick {end_tick} with zero changes and zero wakes"
            ))
        }
        Some(end_tick) => PredicateResult::fail(format!(
            "post-sleep window ended at tick {end_tick}; change_ticks={post_sleep_change_ticks}; wake_ticks={post_sleep_wake_ticks}"
        )),
        None if confirmed_all_sleep_tick.is_none() => PredicateResult::unknown(
            "post-sleep stability could not be measured because full sleep was not confirmed",
        ),
        None => PredicateResult::unknown("post-sleep stability evidence is missing"),
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

    HardPredicates {
        actual_fall,
        matter_conservation,
        no_invalid_materials,
        no_nonfinite_fields,
        sleep_before_max,
        post_sleep_stable,
        exact_reset,
    }
}

fn verdict_from_statuses(statuses: &[PredicateStatus]) -> ExperimentVerdict {
    if statuses.contains(&PredicateStatus::Fail) {
        ExperimentVerdict::Fail
    } else if statuses.contains(&PredicateStatus::Unknown) {
        ExperimentVerdict::NeedsHuman
    } else {
        ExperimentVerdict::Pass
    }
}

fn physical_tick_boundary_equal(left: &GpuSnapshot, right: &GpuSnapshot) -> bool {
    left.material_current == right.material_current
        && left.material_next == right.material_next
        && left.temperature_current == right.temperature_current
        && left.temperature_next == right.temperature_next
        && left.pressure_current == right.pressure_current
        && left.pressure_next == right.pressure_next
        && left.flags_current == right.flags_current
        && left.flags_next == right.flags_next
}

fn exact_reset_equal(left: &GpuSnapshot, right: &GpuSnapshot) -> bool {
    left == right
}

fn authoritative_current_hash(snapshot: &GpuSnapshot) -> String {
    let mut hash = Fnv1a64::new();
    hash.update_u32s(&snapshot.material_current);
    hash.update_u32s(&snapshot.temperature_current);
    hash.update_u32s(&snapshot.pressure_current);
    hash.update_u32s(&snapshot.flags_current);
    format!("fnv1a64:{:016x}", hash.finish())
}

struct Fnv1a64(u64);

impl Fnv1a64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    const fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn update_u32s(&mut self, values: &[u32]) {
        for value in values {
            for byte in value.to_le_bytes() {
                self.0 ^= u64::from(byte);
                self.0 = self.0.wrapping_mul(Self::PRIME);
            }
        }
    }

    const fn finish(self) -> u64 {
        self.0
    }
}

fn bit_count(values: &[u32], bit: u32) -> u64 {
    values.iter().filter(|&&value| value & bit != 0).count() as u64
}

fn take_sequence(next: &mut u64) -> u64 {
    let current = *next;
    *next = next.saturating_add(1);
    current
}

fn write_raw_frames(
    raw_frames_dir: &Path,
    frames: Vec<SemanticFrame>,
) -> Result<Vec<WrittenFrame>, String> {
    let mut written = Vec::with_capacity(frames.len());
    for (ordinal, semantic) in frames.into_iter().enumerate() {
        let filename = frame_filename(ordinal, semantic.kind);
        let path = raw_frames_dir.join(&filename);
        write_new(&path, &semantic.frame.rgba)?;
        written.push(WrittenFrame {
            ordinal,
            kind: semantic.kind,
            relative_path: format!("work/frames/{filename}"),
            width: semantic.frame.width,
            height: semantic.frame.height,
            rgba_bytes: semantic.frame.rgba.len(),
            reason: semantic.reason,
            sim_tick: semantic.sim_tick,
            sample_sequence: semantic.sample_sequence,
            state_hash: semantic.state_hash,
        });
    }
    Ok(written)
}

fn frame_filename(ordinal: usize, kind: &str) -> String {
    format!("{ordinal:02}-{kind}.rgba")
}

fn write_frames_json(
    config: &ExperimentWorkerConfig,
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
            "\n  \"run_id\": \"{}\",\n  \"scenario\": \"sand-fall\",",
            "\n  \"binary_sha256\": \"{}\",",
            "\n  \"frame_count\": {},\n  \"pixel_encoding\": \"rgba8-tightly-packed\",",
            "\n  \"frames\": [{}]\n}}\n"
        ),
        FRAMES_SCHEMA_VERSION,
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
    config: &ExperimentWorkerConfig,
    provenance: &RuntimeProvenance,
    simulation: &Simulation,
    path: &Path,
    predicates: &HardPredicates,
    verdict: ExperimentVerdict,
    sample_count: u64,
    raw_frame_count: usize,
    first_all_sleep_sim_tick: Option<u64>,
    first_all_sleep_sample_sequence: Option<u64>,
    confirmed_all_sleep_tick: Option<u64>,
    post_sleep_end_tick: Option<u64>,
    post_sleep_change_ticks: u32,
    post_sleep_wake_ticks: u32,
    baseline_matter_count: u64,
    baseline_sand_count: u64,
    baseline_sand_y_sum: u64,
    peak_active_count: u64,
    peak_active_chunks: u32,
    first_sleeping_chunk_tick: Option<u64>,
    final_sleeping_chunks: u32,
    matter_count_delta: i128,
    reset_exact_equivalence: bool,
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
        predicate_json("actual_fall", &predicates.actual_fall),
        predicate_json("matter_conservation", &predicates.matter_conservation),
        predicate_json("no_invalid_materials", &predicates.no_invalid_materials),
        predicate_json("no_nonfinite_fields", &predicates.no_nonfinite_fields),
        predicate_json("sleep_before_max", &predicates.sleep_before_max),
        predicate_json("post_sleep_stable", &predicates.post_sleep_stable),
        predicate_json("exact_reset", &predicates.exact_reset),
    ]
    .join(",");
    // Sand analysis v0 preserves `first_all_sleep_diagnostic_sample_tick` for
    // immutable-parser compatibility. Despite its historical name, its value
    // is the diagnostic sample sequence, exactly equal to
    // `first_all_sleep_sample_sequence`; it is never a simulation tick. A
    // future schema must remove or explicitly deprecate the alias and bump its
    // version rather than changing the meaning of published v0 bytes.
    let json = format!(
        concat!(
            "{{\n  \"schema_version\": \"{}\",\n  \"experiment_id\": \"{}\",",
            "\n  \"run_id\": \"{}\",\n  \"scenario\": \"sand-fall\",",
            "\n  \"binary_sha256\": \"{}\",",
            "\n  \"provenance\": {{\"source_sha\":\"{}\",\"git_state\":\"{}\",\"build_profile\":\"{}\"}},",
            "\n  \"world\": {{\"width\":{},\"height\":{},\"chunk_size\":{}}},",
            "\n  \"sleep\": {{\"enabled\":{},\"threshold\":{}}},",
            "\n  \"lifecycle\": {{\"max_ticks\":{},\"diagnostic_interval_ticks\":{},",
            "\"all_sleep_consecutive_samples\":{},\"post_sleep_confirmation_ticks\":{},",
            "\"first_all_sleep_sim_tick\":{},\"first_all_sleep_diagnostic_sample_tick\":{},",
            "\"first_all_sleep_sample_sequence\":{},",
            "\"confirmed_all_sleep_sim_tick\":{},\"post_sleep_end_tick\":{},",
            "\"post_sleep_change_ticks\":{},\"post_sleep_wake_ticks\":{},",
            "\"sample_count\":{}}},",
            "\n  \"baseline\": {{\"matter_count\":{},\"sand_count\":{},\"sand_y_sum\":{}}},",
            "\n  \"metrics\": {{\"peak_active_cells\":{},\"peak_active_chunks\":{},",
            "\"first_sleeping_chunk_tick\":{},\"first_all_sleep_tick\":{},",
            "\"settling_duration\":{},\"post_sleep_state_changes\":{},",
            "\"post_sleep_spontaneous_wakes\":{},\"final_sleeping_chunks\":{},",
            "\"matter_count_delta\":{},\"reset_exact_equivalence\":{}}},",
            "\n  \"predicates\": {{{}}},",
            "\n  \"verdict\": \"{}\",\n  \"raw_frame_count\": {}\n}}\n"
        ),
        ANALYSIS_SCHEMA_VERSION,
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
        config.post_sleep_ticks,
        json_opt_u64(first_all_sleep_sim_tick),
        json_opt_u64(first_all_sleep_sample_sequence),
        json_opt_u64(first_all_sleep_sample_sequence),
        json_opt_u64(confirmed_all_sleep_tick),
        json_opt_u64(post_sleep_end_tick),
        post_sleep_change_ticks,
        post_sleep_wake_ticks,
        sample_count,
        baseline_matter_count,
        baseline_sand_count,
        baseline_sand_y_sum,
        peak_active_count,
        peak_active_chunks,
        json_opt_u64(first_sleeping_chunk_tick),
        json_opt_u64(first_all_sleep_sim_tick),
        json_opt_u64(first_all_sleep_sim_tick),
        post_sleep_change_ticks,
        post_sleep_wake_ticks,
        final_sleeping_chunks,
        matter_count_delta,
        reset_exact_equivalence,
        predicates_json,
        verdict.as_str(),
        raw_frame_count,
    );
    write_new(path, json.as_bytes())
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            value if value <= '\u{1f}' => {
                use std::fmt::Write as _;
                let _ = write!(escaped, "\\u{:04x}", value as u32);
            }
            value => escaped.push(value),
        }
    }
    escaped
}

fn json_opt_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "null".to_string(), |value| value.to_string())
}

fn json_opt_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "null".to_string(), |value| value.to_string())
}

fn is_safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_vectors_and_worker_current_executable_authentication() {
        assert_eq!(
            hex_sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex_sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let executable = std::env::current_exe().expect("test executable path");
        let bytes = fs::read(executable).expect("read test executable");
        let expected = hex_sha256(&bytes);
        verify_current_executable_sha256(&expected).expect("matching executable digest");
        let error = verify_current_executable_sha256(&"0".repeat(64))
            .expect_err("mismatched executable digest must be rejected");
        assert!(error.contains("mismatch"));
    }

    fn empty_snapshot() -> GpuSnapshot {
        GpuSnapshot {
            material_current: vec![0, 3],
            material_next: vec![0, 3],
            temperature_current: vec![0, 0],
            temperature_next: vec![0, 0],
            pressure_current: vec![0, 0],
            pressure_next: vec![0, 0],
            flags_current: vec![0, 0],
            flags_next: vec![0, 0],
            proposal: vec![u32::MAX, u32::MAX],
            claim: vec![0, 0],
            cell_activity: vec![0, 0],
            chunk_activity: vec![0],
            chunk_changed: vec![0],
            chunk_stable: vec![0],
            chunk_edit_wake: vec![1],
            chunk_state: vec![0],
            chunk_wake_reason: vec![0],
            params: vec![2, 64, 2, 1, 1, 2, 1, 1],
            wake_params: vec![2, 1, 1, 8],
            arbitration_params: vec![0, 0, 0, 0],
        }
    }

    #[test]
    fn all_sleep_requires_three_consecutive_samples_and_keeps_first_identity() {
        assert!(all_sleep_counts(0, 0, 0, 16, 16));
        assert!(!all_sleep_counts(1, 0, 0, 16, 16));
        assert!(!all_sleep_counts(0, 1, 0, 16, 16));
        assert!(!all_sleep_counts(0, 0, 1, 15, 16));
        assert!(!all_sleep_counts(0, 0, 0, 0, 0));

        let mut detector = AllSleepDetector::new(3);
        assert_eq!(
            detector.observe(true, 40, 4),
            DetectorUpdate {
                first_in_streak: true,
                confirmed: false,
                streak_broken: false,
            }
        );
        assert!(!detector.observe(true, 50, 5).confirmed);
        assert!(detector.observe(true, 60, 6).confirmed);
        assert_eq!(detector.first_sim_tick, Some(40));
        assert_eq!(detector.first_sample_sequence, Some(4));

        assert!(detector.observe(false, 70, 7).streak_broken);
        assert_eq!(detector.first_sim_tick, None);
        assert!(detector.observe(true, 80, 8).first_in_streak);
        assert_eq!(detector.first_sim_tick, Some(80));
    }

    #[test]
    fn verdict_is_fail_first_then_unknown_then_pass() {
        assert_eq!(
            verdict_from_statuses(&[PredicateStatus::Pass; 7]),
            ExperimentVerdict::Pass
        );
        assert_eq!(
            verdict_from_statuses(&[
                PredicateStatus::Pass,
                PredicateStatus::Unknown,
                PredicateStatus::Pass,
            ]),
            ExperimentVerdict::NeedsHuman
        );
        assert_eq!(
            verdict_from_statuses(&[
                PredicateStatus::Unknown,
                PredicateStatus::Fail,
                PredicateStatus::Pass,
            ]),
            ExperimentVerdict::Fail
        );
    }

    #[test]
    fn exact_reset_compares_world_scratch_activity_and_uniform_state() {
        let pristine = empty_snapshot();
        assert!(exact_reset_equal(&pristine, &pristine.clone()));

        let mut changed_next = pristine.clone();
        changed_next.material_next[1] = 0;
        assert!(!exact_reset_equal(&pristine, &changed_next));

        let mut changed_edit_wake = pristine.clone();
        changed_edit_wake.chunk_edit_wake[0] = 0;
        assert!(!exact_reset_equal(&pristine, &changed_edit_wake));

        let mut changed_proposal = pristine.clone();
        changed_proposal.proposal[0] = 0;
        assert!(!exact_reset_equal(&pristine, &changed_proposal));

        let mut changed_uniform = pristine.clone();
        changed_uniform.wake_params[3] = 9;
        assert!(!exact_reset_equal(&pristine, &changed_uniform));
    }

    #[test]
    fn post_sleep_physical_comparison_includes_current_and_next_but_not_counters() {
        let pristine = empty_snapshot();
        assert!(physical_tick_boundary_equal(&pristine, &pristine));

        let mut changed_next = pristine.clone();
        changed_next.pressure_next[1] = 1.0f32.to_bits();
        assert!(!physical_tick_boundary_equal(&pristine, &changed_next));

        let mut advanced_diagnostic = pristine.clone();
        advanced_diagnostic.chunk_stable[0] = 1;
        assert!(physical_tick_boundary_equal(
            &pristine,
            &advanced_diagnostic
        ));
    }

    #[test]
    fn frame_naming_and_json_escaping_are_stable() {
        assert_eq!(frame_filename(3, "peak-active"), "03-peak-active.rgba");
        assert_eq!(json_escape("a\"b\\c\n"), "a\\\"b\\\\c\\n");
        assert_eq!(json_opt_u64(None), "null");
        assert_eq!(json_opt_u64(Some(42)), "42");
        assert!(is_safe_identifier("expv0-sand-fall_20260817.1"));
        assert!(!is_safe_identifier("../escape"));
        assert!(!is_safe_identifier("with space"));
    }

    #[test]
    fn hard_predicate_serialized_names_are_exactly_seven() {
        let value = build_hard_predicates(true, true, 0, 0, Some(100), 200, Some(280), 0, 0, true);
        assert_eq!(value.statuses(), [PredicateStatus::Pass; 7]);
        assert_eq!(value.verdict(), ExperimentVerdict::Pass);
    }

    #[test]
    fn pressure_lifecycle_option_guard_rejects_each_nonzero_field() {
        assert!(!pressure_lifecycle_options_present(0, 0, 0));
        assert!(pressure_lifecycle_options_present(1, 0, 0));
        assert!(pressure_lifecycle_options_present(0, 1, 0));
        assert!(pressure_lifecycle_options_present(0, 0, 1));
    }
}
