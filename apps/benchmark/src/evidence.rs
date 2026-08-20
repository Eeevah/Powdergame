//! Durable, self-contained machine-readable G8 benchmark evidence.

use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use powdergame_core::{
    ACTIVITY_MATTER, ACTIVITY_PRESSURE, ACTIVITY_REACTION, ACTIVITY_THERMAL, CHUNK_STATE_RUNNABLE,
    CHUNK_STATE_SLEEPING,
};
use powdergame_gpu::{
    ActivityCensusReport, ActivityCensusSnapshot, AdapterReport, ProfiledTickReport,
    TrackedMemoryReport, PASS_NAMES,
};

use crate::config::{BenchmarkCliConfig, BenchmarkScenario};
#[cfg(test)]
use crate::config::{G8A_EVIDENCE_SCHEMA_VERSION, G8B_EVIDENCE_SCHEMA_VERSION};
use crate::stats::{grouped_values, ProfiledStatistics, StatSummary, GROUP_NAMES};

#[cfg(test)]
pub const EVIDENCE_SCHEMA_VERSION: &str = G8A_EVIDENCE_SCHEMA_VERSION;
#[cfg(test)]
pub const FIXTURE_EVIDENCE_SCHEMA_VERSION: &str = G8B_EVIDENCE_SCHEMA_VERSION;

const GROUP_DEFINITION: &str = "matter_movement=movement_propose+movement_commit+material_flag_hygiene_movement+environment_reconcile_movement;ownership_claim=movement_claim+expansion_claim+expansion_environment_receiver_claim+smoke_claim+smoke_environment_receiver_claim;thermal_conduction=thermal;reaction_phase=phase_transition+expansion_spawn_commit+expansion_pressure+material_flag_hygiene_phase+environment_reconcile_expansion+decay+material_flag_hygiene_decay+environment_reconcile_decay+combustion+smoke_commit+material_flag_hygiene_combustion+environment_reconcile_smoke;pressure_structure=environment_blocked_expansion_pressure+pressure+rupture+material_flag_hygiene_rupture+environment_reconcile_rupture;active_sleep_management=activity_wake+activity_propose+activity_reduce";

#[derive(Debug, Clone)]
pub struct GitProvenance {
    pub commit_sha: String,
    pub state: String,
}

impl GitProvenance {
    pub fn detect() -> Self {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let commit_sha =
            git_stdout(&repo, &["rev-parse", "HEAD"]).unwrap_or_else(|| "unavailable".into());
        let state = match git_stdout(&repo, &["status", "--porcelain", "--untracked-files=all"]) {
            Some(output) if output.trim().is_empty() => "clean",
            Some(_) => "dirty",
            None => "unavailable",
        }
        .to_string();
        Self { commit_sha, state }
    }
}

fn git_stdout(repo: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
}

#[derive(Debug, Clone)]
pub struct RunProvenance {
    pub run_id: String,
    pub git: GitProvenance,
    pub build_profile: &'static str,
}

impl RunProvenance {
    pub fn capture() -> Self {
        Self::capture_with_prefix("g8a")
    }

    pub fn capture_for_scenario(scenario: BenchmarkScenario) -> Self {
        if scenario.is_calibration() {
            Self::capture()
        } else {
            Self::capture_with_prefix(&scenario.run_id_prefix())
        }
    }

    fn capture_with_prefix(prefix: &str) -> Self {
        let epoch_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis());
        Self {
            run_id: format!("{prefix}-{epoch_millis}"),
            git: GitProvenance::detect(),
            build_profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThroughputTrialResult {
    pub trial: u32,
    pub total_ticks: u32,
    pub elapsed_wall_ms: f64,
    pub wall_ms_per_tick: f64,
    pub sustained_tps: f64,
}

#[derive(Debug, Clone)]
pub struct ProfiledTrialResult {
    pub trial: u32,
    pub stats: ProfiledStatistics,
}

#[derive(Debug, Clone)]
pub struct ProfiledSample {
    pub trial: u32,
    pub sample_id: u32,
    pub report: ProfiledTickReport,
}

#[derive(Debug, Clone)]
pub struct OverheadReport {
    pub ticks: u32,
    pub batched_unprofiled_ms: f64,
    pub synchronized_unprofiled_ms: f64,
    pub synchronized_profiled_ms: f64,
    pub synchronization_overhead_pct: f64,
    pub profiling_increment_pct: f64,
    pub total_profiled_path_overhead_pct: f64,
}

pub struct EvidenceBundle<'a> {
    pub config: &'a BenchmarkCliConfig,
    pub provenance: &'a RunProvenance,
    pub production_adapter: &'a AdapterReport,
    pub profiling_adapter: &'a AdapterReport,
    pub profiling_timestamp_period: f32,
    pub production_prewarm_ticks: u64,
    pub profiling_prewarm_ticks: u64,
    pub throughput_trials: &'a [ThroughputTrialResult],
    pub throughput_tps_stats: &'a StatSummary,
    pub throughput_ms_stats: &'a StatSummary,
    pub profiled_trials: &'a [ProfiledTrialResult],
    pub median_profiled_trial: u32,
    pub profiled_samples: &'a [ProfiledSample],
    pub memory: &'a TrackedMemoryReport,
    pub census: &'a ActivityCensusSnapshot,
    pub census_tick: u64,
    pub overhead: &'a OverheadReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidencePaths {
    pub summary: PathBuf,
    pub raw_ticks: PathBuf,
    pub raw_cells: PathBuf,
    pub raw_chunks: PathBuf,
}

const COMMON_HEADER: [&str; 22] = [
    "schema_version",
    "run_id",
    "commit_sha",
    "git_state",
    "adapter_name",
    "vendor_id",
    "device_id",
    "device_type",
    "backend",
    "driver",
    "driver_info",
    "profiling_enabled",
    "timestamp_period_ns",
    "build_profile",
    "width",
    "height",
    "chunk_size",
    "sleep_enabled",
    "sleep_threshold",
    "prewarm_requested_secs",
    "prewarm_ticks",
    "measurement_mode",
];

const SUMMARY_TAIL_HEADER: [&str; 15] = [
    "selection",
    "trial",
    "tick_start",
    "tick_end",
    "metric_type",
    "name",
    "value",
    "count",
    "p50",
    "p95",
    "mean",
    "min",
    "max",
    "unit",
    "method_note",
];

fn common_values(
    bundle: &EvidenceBundle<'_>,
    adapter: &AdapterReport,
    profiling_enabled: bool,
    timestamp_period: Option<f32>,
    prewarm_ticks: u64,
    measurement_mode: &str,
) -> Vec<String> {
    vec![
        bundle.config.scenario.evidence_schema_version().into(),
        bundle.provenance.run_id.clone(),
        bundle.provenance.git.commit_sha.clone(),
        bundle.provenance.git.state.clone(),
        adapter.name.clone(),
        format!("0x{:04X}", adapter.vendor),
        format!("0x{:04X}", adapter.device),
        adapter.device_type.clone(),
        adapter.backend.clone(),
        adapter.driver.clone(),
        adapter.driver_info.clone(),
        profiling_enabled.to_string(),
        timestamp_period.map_or_else(String::new, |value| format!("{value:.9}")),
        bundle.provenance.build_profile.into(),
        bundle.config.width.to_string(),
        bundle.config.height.to_string(),
        bundle.config.chunk_size.to_string(),
        bundle.config.sleep_enabled.to_string(),
        bundle.config.sleep_threshold.to_string(),
        format!("{:.6}", bundle.config.prewarm_secs),
        prewarm_ticks.to_string(),
        measurement_mode.into(),
    ]
}

fn summary_header() -> Vec<String> {
    COMMON_HEADER
        .into_iter()
        .chain(SUMMARY_TAIL_HEADER)
        .map(str::to_string)
        .collect()
}

fn raw_header() -> Vec<String> {
    let mut header: Vec<String> = COMMON_HEADER.into_iter().map(str::to_string).collect();
    header.extend(
        ["trial", "sample_id", "tick_index", "tick_start", "tick_end"]
            .into_iter()
            .map(str::to_string),
    );
    header.extend(
        PASS_NAMES
            .iter()
            .flat_map(|name| [format!("{name}_start_tick"), format!("{name}_end_tick")]),
    );
    header.extend(PASS_NAMES.iter().map(|name| format!("pass_{name}_ms")));
    header.extend(GROUP_NAMES.iter().map(|name| format!("group_{name}_ms")));
    header.extend(
        [
            "gpu_pass_sum_ms",
            "gpu_tick_envelope_ms",
            "residual_ms",
            "timestamp_unit",
            "duration_unit",
            "group_definition",
        ]
        .into_iter()
        .map(str::to_string),
    );
    header
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn write_csv_row<W: Write>(
    writer: &mut W,
    values: &[String],
    expected_columns: usize,
) -> std::io::Result<()> {
    if values.len() != expected_columns {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "CSV schema mismatch: expected {expected_columns} columns, got {}",
                values.len()
            ),
        ));
    }
    let encoded = values
        .iter()
        .map(|value| csv_field(value))
        .collect::<Vec<_>>()
        .join(",");
    writeln!(writer, "{encoded}")
}

struct SummaryRecord<'a> {
    selection: &'a str,
    trial: String,
    tick_start: u64,
    tick_end: u64,
    metric_type: &'a str,
    name: &'a str,
    value: Option<f64>,
    stats: Option<&'a StatSummary>,
    unit: &'a str,
    method_note: &'a str,
}

fn scenario_method_note(scenario: BenchmarkScenario, method_note: &str) -> String {
    if scenario.is_calibration() {
        method_note.to_string()
    } else {
        format!("{method_note}; scenario={}", scenario.slug())
    }
}

fn write_summary_record<W: Write>(
    writer: &mut W,
    common: &[String],
    record: SummaryRecord<'_>,
    scenario: BenchmarkScenario,
) -> std::io::Result<()> {
    let mut row = common.to_vec();
    let stats = record.stats;
    row.extend([
        record.selection.to_string(),
        record.trial,
        record.tick_start.to_string(),
        record.tick_end.to_string(),
        record.metric_type.to_string(),
        record.name.to_string(),
        record
            .value
            .map_or_else(String::new, |value| format!("{value:.9}")),
        stats.map_or_else(String::new, |value| value.count.to_string()),
        stats.map_or_else(String::new, |value| format!("{:.9}", value.p50)),
        stats.map_or_else(String::new, |value| format!("{:.9}", value.p95)),
        stats.map_or_else(String::new, |value| format!("{:.9}", value.mean)),
        stats.map_or_else(String::new, |value| format!("{:.9}", value.min)),
        stats.map_or_else(String::new, |value| format!("{:.9}", value.max)),
        record.unit.to_string(),
        scenario_method_note(scenario, record.method_note),
    ]);
    write_csv_row(
        writer,
        &row,
        COMMON_HEADER.len() + SUMMARY_TAIL_HEADER.len(),
    )
}

fn write_summary<W: Write>(writer: &mut W, bundle: &EvidenceBundle<'_>) -> std::io::Result<()> {
    let header = summary_header();
    write_csv_row(writer, &header, header.len())?;

    let production_common = common_values(
        bundle,
        bundle.production_adapter,
        false,
        None,
        bundle.production_prewarm_ticks,
        "production_throughput",
    );
    let throughput_end = u64::from(bundle.config.throughput_ticks - 1);
    for trial in bundle.throughput_trials {
        let trial_tick_end = u64::from(trial.total_ticks - 1);
        for (name, value, unit) in [
            ("elapsed_wall", trial.elapsed_wall_ms, "ms"),
            ("wall_per_tick", trial.wall_ms_per_tick, "ms/tick"),
            ("sustained_tps", trial.sustained_tps, "ticks/s"),
        ] {
            write_summary_record(
                writer,
                &production_common,
                SummaryRecord {
                    selection: "trial",
                    trial: trial.trial.to_string(),
                    tick_start: 0,
                    tick_end: trial_tick_end,
                    metric_type: "throughput_trial",
                    name,
                    value: Some(value),
                    stats: None,
                    unit,
                    method_note:
                        "batch-submitted; one completion wait after the measured tick window",
                },
                bundle.config.scenario,
            )?;
        }
    }
    for (name, stats, unit) in [
        ("wall_per_tick", bundle.throughput_ms_stats, "ms/tick"),
        ("sustained_tps", bundle.throughput_tps_stats, "ticks/s"),
    ] {
        write_summary_record(
            writer,
            &production_common,
            SummaryRecord {
                selection: "all_trials",
                trial: "all".into(),
                tick_start: 0,
                tick_end: throughput_end,
                metric_type: "throughput_summary",
                name,
                value: None,
                stats: Some(stats),
                unit,
                method_note: "statistics across independent reset/restage trials",
            },
            bundle.config.scenario,
        )?;
    }

    let profiling_common = common_values(
        bundle,
        bundle.profiling_adapter,
        true,
        Some(bundle.profiling_timestamp_period),
        bundle.profiling_prewarm_ticks,
        "isolated_profiled_tick",
    );
    let profile_end = u64::from(bundle.config.profile_ticks - 1);
    for trial in bundle.profiled_trials {
        let selection = if trial.trial == bundle.median_profiled_trial {
            "median_envelope_trial"
        } else {
            "trial"
        };
        for (index, name) in PASS_NAMES.iter().enumerate() {
            write_summary_record(
                writer,
                &profiling_common,
                SummaryRecord {
                    selection,
                    trial: trial.trial.to_string(),
                    tick_start: 0,
                    tick_end: profile_end,
                    metric_type: "pass",
                    name,
                    value: None,
                    stats: Some(&trial.stats.pass_stats[index]),
                    unit: "ms",
                    method_note: "GPU timestamp duration for one production compute pass",
                },
                bundle.config.scenario,
            )?;
        }
        for (group_index, group_name) in GROUP_NAMES.iter().enumerate() {
            write_summary_record(
                writer,
                &profiling_common,
                SummaryRecord {
                    selection,
                    trial: trial.trial.to_string(),
                    tick_start: 0,
                    tick_end: profile_end,
                    metric_type: "grouped_subsystem",
                    name: group_name,
                    value: None,
                    stats: Some(&trial.stats.grouped_stats[group_index]),
                    unit: "ms",
                    method_note:
                        "percentile of per-tick grouped sums; never a sum of pass percentiles",
                },
                bundle.config.scenario,
            )?;
            write_summary_record(
                writer,
                &profiling_common,
                SummaryRecord {
                    selection,
                    trial: trial.trial.to_string(),
                    tick_start: 0,
                    tick_end: profile_end,
                    metric_type: "grouped_envelope_ratio",
                    name: group_name,
                    value: None,
                    stats: Some(&trial.stats.grouped_envelope_pct_stats[group_index]),
                    unit: "percent",
                    method_note: "percentile of each tick's group/envelope ratio",
                },
                bundle.config.scenario,
            )?;
        }
        for (name, stats) in [
            ("gpu_tick_envelope", &trial.stats.envelope_stats),
            ("gpu_pass_sum", &trial.stats.pass_sum_stats),
            ("diagnostic_residual", &trial.stats.residual_stats),
        ] {
            write_summary_record(
                writer,
                &profiling_common,
                SummaryRecord {
                    selection,
                    trial: trial.trial.to_string(),
                    tick_start: 0,
                    tick_end: profile_end,
                    metric_type: "envelope",
                    name,
                    value: None,
                    stats: Some(stats),
                    unit: "ms",
                    method_note: "isolated fully synchronized profiled-tick cadence",
                },
                bundle.config.scenario,
            )?;
        }
    }

    let memory_values = [
        ("world_dense_state", bundle.memory.world_dense_state_bytes),
        ("environment_state", bundle.memory.environment_state_bytes),
        ("movement_scratch", bundle.memory.movement_scratch_bytes),
        (
            "environment_receiver_claim",
            bundle.memory.environment_receiver_claim_bytes,
        ),
        ("activity_scratch", bundle.memory.activity_scratch_bytes),
        (
            "uniforms_and_tables",
            bundle.memory.uniforms_and_tables_bytes,
        ),
        (
            "profiler_resolve_and_readback",
            bundle.memory.profiler_bytes,
        ),
        ("total_tracked", bundle.memory.total_tracked_gpu_bytes),
    ];
    for (name, value) in memory_values {
        write_summary_record(
            writer,
            &profiling_common,
            SummaryRecord {
                selection: "snapshot",
                trial: "n/a".into(),
                tick_start: bundle.census_tick,
                tick_end: bundle.census_tick,
                metric_type: "application_tracked_buffer_allocation",
                name,
                value: Some(value as f64),
                stats: None,
                unit: "bytes",
                method_note: "persistent application-requested buffers; not resident VRAM; transient readbacks and opaque query storage excluded",
            },
            bundle.config.scenario,
        )?;
    }

    let census_values = [
        ("total_cells", bundle.census.report.total_cells),
        ("any_active_cells", bundle.census.report.any_active_cells),
        (
            "matter_active_cells",
            bundle.census.report.matter_active_cells,
        ),
        (
            "thermal_active_cells",
            bundle.census.report.thermal_active_cells,
        ),
        (
            "pressure_active_cells",
            bundle.census.report.pressure_active_cells,
        ),
        (
            "reaction_active_cells",
            bundle.census.report.reaction_active_cells,
        ),
        ("total_chunks", u64::from(bundle.census.report.total_chunks)),
        (
            "active_chunks",
            u64::from(bundle.census.report.active_chunks),
        ),
        (
            "runnable_chunks",
            u64::from(bundle.census.report.runnable_chunks),
        ),
        (
            "sleeping_chunks",
            u64::from(bundle.census.report.sleeping_chunks),
        ),
    ];
    for (name, value) in census_values {
        write_summary_record(
            writer,
            &profiling_common,
            SummaryRecord {
                selection: "snapshot",
                trial: "n/a".into(),
                tick_start: bundle.census_tick,
                tick_end: bundle.census_tick,
                metric_type: "activity_census",
                name,
                value: Some(value as f64),
                stats: None,
                unit: "count",
                method_note: "active overlaps runnable/sleeping; chunk categories are not all mutually exclusive",
            },
            bundle.config.scenario,
        )?;
    }

    let overhead_end = u64::from(bundle.overhead.ticks - 1);
    for (name, value, unit, note) in [
        (
            "batched_unprofiled_elapsed",
            bundle.overhead.batched_unprofiled_ms,
            "ms",
            "unprofiled batch with one completion wait",
        ),
        (
            "synchronized_unprofiled_elapsed",
            bundle.overhead.synchronized_unprofiled_ms,
            "ms",
            "unprofiled tick with a completion wait after every tick",
        ),
        (
            "synchronized_profiled_elapsed",
            bundle.overhead.synchronized_profiled_ms,
            "ms",
            "profiled tick with timestamp resolve/copy/map/readback and per-tick wait",
        ),
        (
            "synchronization_overhead",
            bundle.overhead.synchronization_overhead_pct,
            "percent",
            "synchronized unprofiled relative to batched unprofiled",
        ),
        (
            "profiling_increment",
            bundle.overhead.profiling_increment_pct,
            "percent",
            "synchronized profiled relative to synchronized unprofiled",
        ),
        (
            "observed_profiled_path_overhead",
            bundle.overhead.total_profiled_path_overhead_pct,
            "percent",
            "combined timestamp, resolve, copy, synchronization, map/readback, CPU orchestration, and lost-pipelining delta",
        ),
    ] {
        write_summary_record(
            writer,
            &profiling_common,
            SummaryRecord {
                selection: "matched_control",
                trial: "n/a".into(),
                tick_start: 0,
                tick_end: overhead_end,
                metric_type: "profiling_overhead",
                name,
                value: Some(value),
                stats: None,
                unit,
                method_note: note,
            },
            bundle.config.scenario,
        )?;
    }

    Ok(())
}

fn write_raw<W: Write>(writer: &mut W, bundle: &EvidenceBundle<'_>) -> std::io::Result<()> {
    let header = raw_header();
    write_csv_row(writer, &header, header.len())?;
    let common = common_values(
        bundle,
        bundle.profiling_adapter,
        true,
        Some(bundle.profiling_timestamp_period),
        bundle.profiling_prewarm_ticks,
        "isolated_profiled_tick",
    );

    for sample in bundle.profiled_samples {
        let grouped = grouped_values(&sample.report);
        let mut row = common.clone();
        row.extend([
            sample.trial.to_string(),
            sample.sample_id.to_string(),
            sample.report.tick_index.to_string(),
            sample.report.tick_index.to_string(),
            sample.report.tick_index.to_string(),
        ]);
        row.extend(sample.report.raw_timestamps.iter().map(ToString::to_string));
        row.extend(
            sample
                .report
                .passes
                .iter()
                .map(|pass| format!("{:.9}", pass.duration_ms)),
        );
        row.extend(grouped.iter().map(|value| format!("{value:.9}")));
        row.extend([
            format!("{:.9}", sample.report.gpu_pass_sum_ms),
            format!("{:.9}", sample.report.gpu_tick_envelope_ms),
            format!("{:.9}", sample.report.residual_ms),
            "raw_gpu_tick".into(),
            "milliseconds".into(),
            GROUP_DEFINITION.into(),
        ]);
        write_csv_row(writer, &row, header.len())?;
    }
    Ok(())
}

const RAW_CELL_HEADER: [&str; 7] = [
    "schema_version",
    "run_id",
    "commit_sha",
    "git_state",
    "census_tick",
    "index",
    "activity_mask",
];

const RAW_CHUNK_HEADER: [&str; 8] = [
    "schema_version",
    "run_id",
    "commit_sha",
    "git_state",
    "census_tick",
    "index",
    "activity_mask",
    "chunk_state",
];

fn census_common_values(
    provenance: &RunProvenance,
    census_tick: u64,
    scenario: BenchmarkScenario,
) -> [String; 5] {
    [
        scenario.evidence_schema_version().into(),
        provenance.run_id.clone(),
        provenance.git.commit_sha.clone(),
        provenance.git.state.clone(),
        census_tick.to_string(),
    ]
}

fn write_raw_cells<W: Write>(
    writer: &mut W,
    provenance: &RunProvenance,
    census_tick: u64,
    census: &ActivityCensusSnapshot,
    scenario: BenchmarkScenario,
) -> io::Result<()> {
    let header = RAW_CELL_HEADER.map(str::to_string);
    write_csv_row(writer, &header, header.len())?;
    let common = census_common_values(provenance, census_tick, scenario);
    for (index, &activity_mask) in census.cell_activity.iter().enumerate() {
        let mut row = common.to_vec();
        row.extend([index.to_string(), activity_mask.to_string()]);
        write_csv_row(writer, &row, header.len())?;
    }
    Ok(())
}

fn write_raw_chunks<W: Write>(
    writer: &mut W,
    provenance: &RunProvenance,
    census_tick: u64,
    census: &ActivityCensusSnapshot,
    scenario: BenchmarkScenario,
) -> io::Result<()> {
    if census.chunk_activity.len() != census.chunk_state.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "chunk activity/state length mismatch: {} activity rows, {} state rows",
                census.chunk_activity.len(),
                census.chunk_state.len()
            ),
        ));
    }

    let header = RAW_CHUNK_HEADER.map(str::to_string);
    write_csv_row(writer, &header, header.len())?;
    let common = census_common_values(provenance, census_tick, scenario);
    for (index, (&activity_mask, &chunk_state)) in census
        .chunk_activity
        .iter()
        .zip(&census.chunk_state)
        .enumerate()
    {
        let mut row = common.to_vec();
        row.extend([
            index.to_string(),
            activity_mask.to_string(),
            chunk_state.to_string(),
        ]);
        write_csv_row(writer, &row, header.len())?;
    }
    Ok(())
}

fn recompute_census_report(
    census: &ActivityCensusSnapshot,
) -> Result<ActivityCensusReport, String> {
    if census.chunk_activity.len() != census.chunk_state.len() {
        return Err(format!(
            "chunk activity/state length mismatch: {} activity rows, {} state rows",
            census.chunk_activity.len(),
            census.chunk_state.len()
        ));
    }

    let total_cells = u64::try_from(census.cell_activity.len())
        .map_err(|_| "cell activity row count does not fit u64".to_string())?;
    let total_chunks = u32::try_from(census.chunk_activity.len())
        .map_err(|_| "chunk activity row count does not fit u32".to_string())?;
    let count_cells = |mask: u32| {
        u64::try_from(
            census
                .cell_activity
                .iter()
                .filter(|&&activity| activity & mask != 0)
                .count(),
        )
        .expect("cell count already fits u64")
    };
    let count_chunks = |values: &[u32], expected: Option<u32>| -> Result<u32, String> {
        let count = values
            .iter()
            .filter(|&&value| expected.map_or(value != 0, |expected| value == expected))
            .count();
        u32::try_from(count).map_err(|_| "chunk census count does not fit u32".to_string())
    };

    Ok(ActivityCensusReport {
        total_cells,
        any_active_cells: u64::try_from(
            census
                .cell_activity
                .iter()
                .filter(|&&activity| activity != 0)
                .count(),
        )
        .expect("cell count already fits u64"),
        matter_active_cells: count_cells(ACTIVITY_MATTER),
        thermal_active_cells: count_cells(ACTIVITY_THERMAL),
        pressure_active_cells: count_cells(ACTIVITY_PRESSURE),
        reaction_active_cells: count_cells(ACTIVITY_REACTION),
        total_chunks,
        active_chunks: count_chunks(&census.chunk_activity, None)?,
        runnable_chunks: count_chunks(&census.chunk_state, Some(CHUNK_STATE_RUNNABLE))?,
        sleeping_chunks: count_chunks(&census.chunk_state, Some(CHUNK_STATE_SLEEPING))?,
    })
}

fn validate_census_snapshot(census: &ActivityCensusSnapshot) -> Result<(), String> {
    let recomputed = recompute_census_report(census)?;
    if recomputed == census.report {
        Ok(())
    } else {
        Err(format!(
            "activity census aggregate does not match raw snapshot: aggregate={:?}, recomputed={recomputed:?}",
            census.report
        ))
    }
}

pub fn raw_csv_path(summary_path: &Path) -> PathBuf {
    let parent = summary_path.parent().unwrap_or_else(|| Path::new(""));
    let mut file_name = summary_path
        .file_stem()
        .unwrap_or_else(|| OsStr::new("calibration_report"))
        .to_os_string();
    file_name.push("_raw_ticks.csv");
    parent.join(file_name)
}

pub fn raw_cells_csv_path(summary_path: &Path) -> PathBuf {
    let parent = summary_path.parent().unwrap_or_else(|| Path::new(""));
    let mut file_name = summary_path
        .file_stem()
        .unwrap_or_else(|| OsStr::new("calibration_report"))
        .to_os_string();
    file_name.push("_raw_cells.csv");
    parent.join(file_name)
}

pub fn raw_chunks_csv_path(summary_path: &Path) -> PathBuf {
    let parent = summary_path.parent().unwrap_or_else(|| Path::new(""));
    let mut file_name = summary_path
        .file_stem()
        .unwrap_or_else(|| OsStr::new("calibration_report"))
        .to_os_string();
    file_name.push("_raw_chunks.csv");
    parent.join(file_name)
}

/// Resolves all four output paths and rejects every pre-existing final entry.
/// Call this before expensive GPU work; publication repeats the validation.
pub fn validate_evidence_output_paths(summary_path: &Path) -> Result<EvidencePaths, String> {
    let file_name = summary_path.file_name().ok_or_else(|| {
        format!(
            "summary evidence path must name a file: {}",
            summary_path.display()
        )
    })?;
    let requested_parent = summary_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(requested_parent).map_err(|error| {
        format!(
            "failed to create evidence directory {}: {error}",
            requested_parent.display()
        )
    })?;
    let parent = fs::canonicalize(requested_parent).map_err(|error| {
        format!(
            "failed to resolve evidence directory {}: {error}",
            requested_parent.display()
        )
    })?;
    let summary = parent.join(file_name);
    let paths = EvidencePaths {
        raw_ticks: raw_csv_path(&summary),
        raw_cells: raw_cells_csv_path(&summary),
        raw_chunks: raw_chunks_csv_path(&summary),
        summary,
    };

    for path in [
        &paths.summary,
        &paths.raw_ticks,
        &paths.raw_cells,
        &paths.raw_chunks,
    ] {
        match fs::symlink_metadata(path) {
            Ok(metadata) => {
                let kind = if metadata.file_type().is_symlink() {
                    "symlink"
                } else if metadata.is_dir() {
                    "directory"
                } else if metadata.is_file() {
                    "file"
                } else {
                    "filesystem entry"
                };
                return Err(format!(
                    "refusing to overwrite existing evidence {kind}: {}",
                    path.display()
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "failed to inspect evidence output {}: {error}",
                    path.display()
                ));
            }
        }
    }

    Ok(paths)
}

struct StagingDirectory {
    path: PathBuf,
}

impl StagingDirectory {
    fn create(parent: &Path, stem: &OsStr) -> Result<Self, String> {
        let epoch_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        for attempt in 0..128u32 {
            let mut name = OsString::from(".");
            name.push(stem);
            name.push(format!(
                ".powdergame-evidence-stage-{}-{epoch_nanos}-{attempt}",
                std::process::id()
            ));
            let path = parent.join(name);
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!(
                        "failed to create evidence staging directory {}: {error}",
                        path.display()
                    ));
                }
            }
        }
        Err(format!(
            "failed to allocate a unique evidence staging directory under {}",
            parent.display()
        ))
    }
}

impl Drop for StagingDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_synced_staged_file<F>(path: &Path, label: &str, write: F) -> Result<(), String>
where
    F: FnOnce(&mut BufWriter<File>) -> io::Result<()>,
{
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            format!(
                "failed to create staged {label} evidence {}: {error}",
                path.display()
            )
        })?;
    let mut writer = BufWriter::new(file);
    let result = (|| {
        write(&mut writer)?;
        writer.flush()?;
        writer.get_ref().sync_all()
    })();
    drop(writer);
    result.map_err(|error| {
        format!(
            "failed to write and sync staged {label} evidence {}: {error}",
            path.display()
        )
    })
}

fn publish_staged_file(staged: &Path, final_path: &Path, label: &str) -> Result<(), String> {
    fs::hard_link(staged, final_path).map_err(|error| {
        format!(
            "failed to publish staged {label} evidence {} as {}: {error}",
            staged.display(),
            final_path.display()
        )
    })?;
    if let Err(error) = fs::remove_file(staged) {
        let cleanup = fs::remove_file(final_path);
        return Err(match cleanup {
            Ok(()) => format!(
                "failed to remove staged {label} link {} after publication; published link was removed: {error}",
                staged.display()
            ),
            Err(cleanup_error) => format!(
                "failed to remove staged {label} link {} after publication: {error}; also failed to remove published evidence {}: {cleanup_error}",
                staged.display(),
                final_path.display()
            ),
        });
    }
    Ok(())
}

fn remove_published_files(paths: &[&Path]) -> Option<String> {
    let mut failures = Vec::new();
    for path in paths.iter().rev() {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }
    if failures.is_empty() {
        None
    } else {
        Some(failures.join("; "))
    }
}

fn stage_and_publish_evidence_files<
    SummaryWriter,
    RawTicksWriter,
    RawCellsWriter,
    RawChunksWriter,
>(
    summary_path: &Path,
    write_summary_file: SummaryWriter,
    write_raw_ticks_file: RawTicksWriter,
    write_raw_cells_file: RawCellsWriter,
    write_raw_chunks_file: RawChunksWriter,
) -> Result<EvidencePaths, String>
where
    SummaryWriter: FnOnce(&mut BufWriter<File>) -> io::Result<()>,
    RawTicksWriter: FnOnce(&mut BufWriter<File>) -> io::Result<()>,
    RawCellsWriter: FnOnce(&mut BufWriter<File>) -> io::Result<()>,
    RawChunksWriter: FnOnce(&mut BufWriter<File>) -> io::Result<()>,
{
    stage_and_publish_evidence_files_with_publisher(
        summary_path,
        write_summary_file,
        write_raw_ticks_file,
        write_raw_cells_file,
        write_raw_chunks_file,
        publish_staged_file,
    )
}

fn stage_and_publish_evidence_files_with_publisher<
    SummaryWriter,
    RawTicksWriter,
    RawCellsWriter,
    RawChunksWriter,
    Publisher,
>(
    summary_path: &Path,
    write_summary_file: SummaryWriter,
    write_raw_ticks_file: RawTicksWriter,
    write_raw_cells_file: RawCellsWriter,
    write_raw_chunks_file: RawChunksWriter,
    mut publish: Publisher,
) -> Result<EvidencePaths, String>
where
    SummaryWriter: FnOnce(&mut BufWriter<File>) -> io::Result<()>,
    RawTicksWriter: FnOnce(&mut BufWriter<File>) -> io::Result<()>,
    RawCellsWriter: FnOnce(&mut BufWriter<File>) -> io::Result<()>,
    RawChunksWriter: FnOnce(&mut BufWriter<File>) -> io::Result<()>,
    Publisher: FnMut(&Path, &Path, &str) -> Result<(), String>,
{
    let paths = validate_evidence_output_paths(summary_path)?;
    let parent = paths
        .summary
        .parent()
        .expect("validated evidence path must have an absolute parent");
    let stem = paths
        .summary
        .file_stem()
        .unwrap_or_else(|| OsStr::new("calibration_report"));
    let staging = StagingDirectory::create(parent, stem)?;
    let staged_summary = staging.path.join("aggregate.csv.tmp");
    let staged_raw_ticks = staging.path.join("raw_ticks.csv.tmp");
    let staged_raw_cells = staging.path.join("raw_cells.csv.tmp");
    let staged_raw_chunks = staging.path.join("raw_chunks.csv.tmp");

    write_synced_staged_file(&staged_raw_cells, "raw cell", write_raw_cells_file)?;
    write_synced_staged_file(&staged_raw_chunks, "raw chunk", write_raw_chunks_file)?;
    write_synced_staged_file(&staged_raw_ticks, "raw tick", write_raw_ticks_file)?;
    write_synced_staged_file(&staged_summary, "aggregate", write_summary_file)?;

    let rechecked = validate_evidence_output_paths(&paths.summary)?;
    if rechecked != paths {
        return Err("evidence output paths changed between validation and publication".into());
    }

    let mut published = Vec::with_capacity(4);
    for (staged_path, final_path, label) in [
        (&staged_raw_cells, &paths.raw_cells, "raw cell"),
        (&staged_raw_chunks, &paths.raw_chunks, "raw chunk"),
        (&staged_raw_ticks, &paths.raw_ticks, "raw tick"),
        (&staged_summary, &paths.summary, "aggregate"),
    ] {
        if let Err(error) = publish(staged_path, final_path, label) {
            return Err(match remove_published_files(&published) {
                Some(cleanup_error) => {
                    format!("{error}; failed to clean published evidence: {cleanup_error}")
                }
                None => error,
            });
        }
        published.push(final_path.as_path());
    }

    Ok(paths)
}

/// Writes and syncs all staged files, then publishes raw cells, raw chunks, raw
/// ticks, and the aggregate summary in that order. The individual publications
/// are not a cross-file atomic transaction under process termination or an OS
/// crash. A capture wrapper must publish its receipt after all four files exist.
pub fn write_evidence(bundle: &EvidenceBundle<'_>) -> Result<EvidencePaths, String> {
    validate_census_snapshot(bundle.census)?;
    stage_and_publish_evidence_files(
        &bundle.config.csv_output,
        |writer| write_summary(writer, bundle),
        |writer| write_raw(writer, bundle),
        |writer| {
            write_raw_cells(
                writer,
                bundle.provenance,
                bundle.census_tick,
                bundle.census,
                bundle.config.scenario,
            )
        },
        |writer| {
            write_raw_chunks(
                writer,
                bundle.provenance,
                bundle.census_tick,
                bundle.census,
                bundle.config.scenario,
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::ProfiledStatistics;
    use powdergame_gpu::PassTiming;

    fn temporary_test_directory(label: &str) -> StagingDirectory {
        StagingDirectory::create(&std::env::temp_dir(), OsStr::new(label)).unwrap()
    }

    #[test]
    fn csv_escaping_is_rfc4180_compatible() {
        assert_eq!(csv_field("plain"), "plain");
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("a\"b"), "\"a\"\"b\"");
        assert_eq!(csv_field("a\nb"), "\"a\nb\"");
    }

    #[test]
    fn evidence_headers_pin_required_provenance_and_sample_identity() {
        let summary = summary_header();
        let raw = raw_header();
        for required in [
            "schema_version",
            "commit_sha",
            "git_state",
            "adapter_name",
            "backend",
            "driver",
            "build_profile",
            "width",
            "height",
            "chunk_size",
            "sleep_enabled",
            "sleep_threshold",
            "trial",
            "tick_start",
            "tick_end",
        ] {
            assert!(summary.contains(&required.to_string()));
        }
        for required in [
            "trial",
            "sample_id",
            "tick_index",
            "activity_wake_start_tick",
            "activity_reduce_end_tick",
            "pass_activity_wake_ms",
            "group_matter_movement_ms",
            "gpu_tick_envelope_ms",
            "group_definition",
        ] {
            assert!(raw.contains(&required.to_string()));
        }

        for (index, pass_name) in PASS_NAMES.iter().enumerate() {
            let start = raw
                .iter()
                .position(|column| column == &format!("{pass_name}_start_tick"))
                .unwrap();
            let end = raw
                .iter()
                .position(|column| column == &format!("{pass_name}_end_tick"))
                .unwrap();
            assert_eq!(end, start + 1, "raw pair {index} must remain adjacent");
        }
        for group_name in GROUP_NAMES {
            assert!(GROUP_DEFINITION.contains(group_name));
        }
        for pass_name in PASS_NAMES {
            assert!(GROUP_DEFINITION.contains(pass_name));
        }
    }

    #[test]
    fn scenario_identity_preserves_calibration_and_scopes_shared_fixtures() {
        let calibration = BenchmarkScenario::Calibration;
        let sand: BenchmarkScenario = "sand-fall".parse().unwrap();

        assert_eq!(
            calibration.evidence_schema_version(),
            EVIDENCE_SCHEMA_VERSION
        );
        assert_eq!(
            sand.evidence_schema_version(),
            FIXTURE_EVIDENCE_SCHEMA_VERSION
        );
        assert_eq!(scenario_method_note(calibration, "base note"), "base note");
        assert_eq!(
            scenario_method_note(sand, "base note"),
            "base note; scenario=sand-fall"
        );

        assert!(RunProvenance::capture_for_scenario(calibration)
            .run_id
            .starts_with("g8a-"));
        assert!(RunProvenance::capture_for_scenario(sand)
            .run_id
            .starts_with("g8b-sand-fall-"));
    }

    #[test]
    fn raw_path_is_derived_without_overwriting_summary() {
        assert_eq!(
            raw_csv_path(Path::new("target/calibration_report.csv")),
            PathBuf::from("target/calibration_report_raw_ticks.csv")
        );
        assert_eq!(
            raw_cells_csv_path(Path::new("target/calibration_report.csv")),
            PathBuf::from("target/calibration_report_raw_cells.csv")
        );
        assert_eq!(
            raw_chunks_csv_path(Path::new("target/calibration_report.csv")),
            PathBuf::from("target/calibration_report_raw_chunks.csv")
        );
    }

    #[test]
    fn staged_publication_rejects_raw_path_directory_without_creating_summary() {
        let test_directory = temporary_test_directory("raw-path-directory");
        let summary = test_directory.path.join("report.csv");
        let raw_ticks = raw_csv_path(&summary);
        let raw_cells = raw_cells_csv_path(&summary);
        let raw_chunks = raw_chunks_csv_path(&summary);
        fs::create_dir(&raw_ticks).unwrap();

        let error = stage_and_publish_evidence_files(
            &summary,
            |writer| writer.write_all(b"aggregate"),
            |writer| writer.write_all(b"raw ticks"),
            |writer| writer.write_all(b"raw cells"),
            |writer| writer.write_all(b"raw chunks"),
        )
        .unwrap_err();

        assert!(error.contains("directory"));
        assert!(!summary.exists());
        assert!(raw_ticks.is_dir());
        assert!(!raw_cells.exists());
        assert!(!raw_chunks.exists());
    }

    #[test]
    fn staged_publication_writer_failure_leaves_all_final_paths_absent() {
        let test_directory = temporary_test_directory("staged-writer-failure");
        let summary = test_directory.path.join("report.csv");
        let raw_ticks = raw_csv_path(&summary);
        let raw_cells = raw_cells_csv_path(&summary);
        let raw_chunks = raw_chunks_csv_path(&summary);

        let error = stage_and_publish_evidence_files(
            &summary,
            |writer| writer.write_all(b"aggregate"),
            |_writer| Err(io::Error::other("injected raw tick staging failure")),
            |writer| writer.write_all(b"raw cells"),
            |writer| writer.write_all(b"raw chunks"),
        )
        .unwrap_err();

        assert!(error.contains("injected raw tick staging failure"));
        assert!(!summary.exists());
        assert!(!raw_ticks.exists());
        assert!(!raw_cells.exists());
        assert!(!raw_chunks.exists());
        assert!(fs::read_dir(&test_directory.path).unwrap().next().is_none());
    }

    #[test]
    fn staged_publication_uses_raw_to_aggregate_order_and_cleans_on_failure() {
        let test_directory = temporary_test_directory("publication-order-failure");
        let summary = test_directory.path.join("report.csv");
        let paths = EvidencePaths {
            summary: summary.clone(),
            raw_ticks: raw_csv_path(&summary),
            raw_cells: raw_cells_csv_path(&summary),
            raw_chunks: raw_chunks_csv_path(&summary),
        };
        let mut publication_order = Vec::new();

        let error = stage_and_publish_evidence_files_with_publisher(
            &summary,
            |writer| writer.write_all(b"aggregate"),
            |writer| writer.write_all(b"raw ticks"),
            |writer| writer.write_all(b"raw cells"),
            |writer| writer.write_all(b"raw chunks"),
            |staged, final_path, label| {
                publication_order.push(label.to_string());
                if label == "aggregate" {
                    Err("injected aggregate publication failure".into())
                } else {
                    publish_staged_file(staged, final_path, label)
                }
            },
        )
        .unwrap_err();

        assert!(error.contains("injected aggregate publication failure"));
        assert_eq!(
            publication_order,
            ["raw cell", "raw chunk", "raw tick", "aggregate"]
        );
        for path in [
            &paths.summary,
            &paths.raw_ticks,
            &paths.raw_cells,
            &paths.raw_chunks,
        ] {
            assert!(!path.exists());
        }
        assert!(fs::read_dir(&test_directory.path).unwrap().next().is_none());
    }

    #[test]
    fn staged_publication_rejects_existing_file_without_overwriting_it() {
        let test_directory = temporary_test_directory("existing-output");
        let summary = test_directory.path.join("report.csv");
        let raw_cells = raw_cells_csv_path(&summary);
        fs::write(&raw_cells, b"existing raw cells").unwrap();

        let error = stage_and_publish_evidence_files(
            &summary,
            |writer| writer.write_all(b"aggregate"),
            |writer| writer.write_all(b"raw ticks"),
            |writer| writer.write_all(b"new raw cells"),
            |writer| writer.write_all(b"raw chunks"),
        )
        .unwrap_err();

        assert!(error.contains("refusing to overwrite existing evidence file"));
        assert_eq!(fs::read(raw_cells).unwrap(), b"existing raw cells");
        assert!(!summary.exists());
    }

    fn sample_census_snapshot() -> ActivityCensusSnapshot {
        ActivityCensusSnapshot {
            report: ActivityCensusReport {
                total_cells: 2,
                any_active_cells: 1,
                matter_active_cells: 1,
                thermal_active_cells: 1,
                pressure_active_cells: 0,
                reaction_active_cells: 0,
                total_chunks: 2,
                active_chunks: 1,
                runnable_chunks: 1,
                sleeping_chunks: 1,
            },
            cell_activity: vec![0, ACTIVITY_MATTER | ACTIVITY_THERMAL],
            chunk_activity: vec![ACTIVITY_MATTER | ACTIVITY_THERMAL, 0],
            chunk_state: vec![CHUNK_STATE_RUNNABLE, CHUNK_STATE_SLEEPING],
        }
    }

    #[test]
    fn raw_cell_and_chunk_writers_emit_one_rectangular_row_per_snapshot_entry() {
        let provenance = RunProvenance {
            run_id: "test-run".into(),
            git: GitProvenance {
                commit_sha: "deadbeef".into(),
                state: "dirty".into(),
            },
            build_profile: "test",
        };
        let census = sample_census_snapshot();
        let mut cells = Vec::new();
        let mut chunks = Vec::new();

        write_raw_cells(
            &mut cells,
            &provenance,
            42,
            &census,
            BenchmarkScenario::Calibration,
        )
        .unwrap();
        write_raw_chunks(
            &mut chunks,
            &provenance,
            42,
            &census,
            BenchmarkScenario::Calibration,
        )
        .unwrap();

        assert_eq!(
            String::from_utf8(cells).unwrap(),
            concat!(
                "schema_version,run_id,commit_sha,git_state,census_tick,index,activity_mask\n",
                "powdergame-g8a-v5,test-run,deadbeef,dirty,42,0,0\n",
                "powdergame-g8a-v5,test-run,deadbeef,dirty,42,1,3\n",
            )
        );
        assert_eq!(
            String::from_utf8(chunks).unwrap(),
            concat!(
                "schema_version,run_id,commit_sha,git_state,census_tick,index,activity_mask,chunk_state\n",
                "powdergame-g8a-v5,test-run,deadbeef,dirty,42,0,3,0\n",
                "powdergame-g8a-v5,test-run,deadbeef,dirty,42,1,0,1\n",
            )
        );

        let mut shared_cells = Vec::new();
        write_raw_cells(
            &mut shared_cells,
            &provenance,
            42,
            &census,
            "sand-fall".parse().unwrap(),
        )
        .unwrap();
        assert!(String::from_utf8(shared_cells)
            .unwrap()
            .lines()
            .nth(1)
            .unwrap()
            .starts_with("powdergame-g8b-fixture-v1,"));
    }

    #[test]
    fn census_aggregate_is_recomputed_from_raw_snapshot_and_mismatch_is_rejected() {
        let mut census = sample_census_snapshot();
        assert_eq!(recompute_census_report(&census).unwrap(), census.report);
        validate_census_snapshot(&census).unwrap();

        census.report.any_active_cells += 1;
        let error = validate_census_snapshot(&census).unwrap_err();
        assert!(error.contains("aggregate does not match raw snapshot"));
    }

    #[test]
    fn row_writer_rejects_schema_mismatch() {
        let mut output = Vec::new();
        let error = write_csv_row(&mut output, &["one".into()], 2).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn aggregate_writer_emits_precomputed_per_tick_group_percentile() {
        let zero = StatSummary::from_slice(&[0.0]);
        let pass_stats = std::array::from_fn(|_| StatSummary::from_slice(&[1.0]));
        let grouped_stats = std::array::from_fn(|index| {
            StatSummary::from_slice(&[if index == 0 { 100.0 } else { 1.0 }])
        });
        let grouped_envelope_pct_stats = std::array::from_fn(|_| StatSummary::from_slice(&[1.0]));
        let profiled_trials = [ProfiledTrialResult {
            trial: 1,
            stats: ProfiledStatistics {
                pass_stats,
                grouped_stats,
                grouped_envelope_pct_stats,
                envelope_stats: StatSummary::from_slice(&[200.0]),
                pass_sum_stats: StatSummary::from_slice(&[105.0]),
                residual_stats: StatSummary::from_slice(&[95.0]),
            },
        }];
        let throughput_trials = [ThroughputTrialResult {
            trial: 1,
            total_ticks: 1,
            elapsed_wall_ms: 1.0,
            wall_ms_per_tick: 1.0,
            sustained_tps: 1.0,
        }];
        let raw_timestamps = std::array::from_fn(|index| 1_000 + index as u64 * 100);
        let passes = std::array::from_fn(|index| PassTiming {
            name: PASS_NAMES[index],
            raw_start: raw_timestamps[index * 2],
            raw_end: raw_timestamps[index * 2 + 1],
            duration_ns: 100.0,
            duration_ms: 0.0001,
        });
        let profiled_samples = [ProfiledSample {
            trial: 1,
            sample_id: 0,
            report: ProfiledTickReport {
                tick_index: 7,
                timestamp_period: 1.0,
                passes,
                raw_timestamps,
                gpu_pass_sum_ms: 0.0017,
                gpu_tick_envelope_ms: 0.0033,
                residual_ms: 0.0016,
            },
        }];
        let config = BenchmarkCliConfig {
            throughput_ticks: 1,
            profile_ticks: 1,
            overhead_ticks: 1,
            trials: 1,
            ..BenchmarkCliConfig::default()
        };
        let adapter = AdapterReport {
            name: "adapter".into(),
            vendor: 1,
            device: 2,
            device_type: "DiscreteGpu".into(),
            backend: "Dx12".into(),
            driver: "driver".into(),
            driver_info: String::new(),
        };
        let provenance = RunProvenance {
            run_id: "test-run".into(),
            git: GitProvenance {
                commit_sha: "deadbeef".into(),
                state: "clean".into(),
            },
            build_profile: "test",
        };
        let memory = TrackedMemoryReport {
            world_dense_state_bytes: 1,
            environment_state_bytes: 1,
            movement_scratch_bytes: 1,
            environment_receiver_claim_bytes: 1,
            activity_scratch_bytes: 1,
            uniforms_and_tables_bytes: 1,
            profiler_bytes: 1,
            total_tracked_gpu_bytes: 7,
        };
        let census = ActivityCensusSnapshot {
            report: ActivityCensusReport {
                total_cells: 1,
                any_active_cells: 0,
                matter_active_cells: 0,
                thermal_active_cells: 0,
                pressure_active_cells: 0,
                reaction_active_cells: 0,
                total_chunks: 1,
                active_chunks: 0,
                runnable_chunks: 1,
                sleeping_chunks: 0,
            },
            cell_activity: vec![0],
            chunk_activity: vec![0],
            chunk_state: vec![CHUNK_STATE_RUNNABLE],
        };
        let overhead = OverheadReport {
            ticks: 1,
            batched_unprofiled_ms: 1.0,
            synchronized_unprofiled_ms: 1.0,
            synchronized_profiled_ms: 1.0,
            synchronization_overhead_pct: 0.0,
            profiling_increment_pct: 0.0,
            total_profiled_path_overhead_pct: 0.0,
        };
        let bundle = EvidenceBundle {
            config: &config,
            provenance: &provenance,
            production_adapter: &adapter,
            profiling_adapter: &adapter,
            profiling_timestamp_period: 1.0,
            production_prewarm_ticks: 0,
            profiling_prewarm_ticks: 0,
            throughput_trials: &throughput_trials,
            throughput_tps_stats: &zero,
            throughput_ms_stats: &zero,
            profiled_trials: &profiled_trials,
            median_profiled_trial: 1,
            profiled_samples: &profiled_samples,
            memory: &memory,
            census: &census,
            census_tick: 1,
            overhead: &overhead,
        };

        let mut bytes = Vec::new();
        write_summary(&mut bytes, &bundle).unwrap();
        let csv = String::from_utf8(bytes).unwrap();
        let header: Vec<&str> = csv.lines().next().unwrap().split(',').collect();
        let p50_index = header.iter().position(|column| *column == "p50").unwrap();
        let row: Vec<&str> = csv
            .lines()
            .find(|line| line.contains("grouped_subsystem,matter_movement"))
            .unwrap()
            .split(',')
            .collect();
        assert_eq!(row[p50_index], "100.000000000");

        let mut raw_bytes = Vec::new();
        write_raw(&mut raw_bytes, &bundle).unwrap();
        let raw_csv = String::from_utf8(raw_bytes).unwrap();
        let mut raw_lines = raw_csv.lines();
        let raw_header: Vec<&str> = raw_lines.next().unwrap().split(',').collect();
        let raw_row: Vec<&str> = raw_lines.next().unwrap().split(',').collect();
        assert_eq!(raw_row.len(), raw_header.len());
        let start_index = raw_header
            .iter()
            .position(|column| *column == "activity_wake_start_tick")
            .unwrap();
        let final_index = raw_header
            .iter()
            .position(|column| *column == "activity_reduce_end_tick")
            .unwrap();
        let group_definition_index = raw_header
            .iter()
            .position(|column| *column == "group_definition")
            .unwrap();
        assert_eq!(raw_row[start_index], "1000");
        assert_eq!(raw_row[final_index], "6900");
        assert_eq!(raw_row[group_definition_index], GROUP_DEFINITION);
    }
}
