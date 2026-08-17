//! G8 headless performance measurement harness.
//!
//! The default calibration path preserves the G8-A v5 evidence contract. G8-B
//! fixture selections use the shared scenario crate without entering gallery
//! rendering or readback code.
//!
//! Mode A uses a normal production context and batch-submitted `tick()` calls.
//! Mode B uses a separately created profiling context and isolated synchronized
//! `tick_profiled()` calls. Durable evidence is emitted as aggregate, raw tick,
//! raw cell-census, and raw chunk-census CSV files.

mod config;
mod evidence;
mod fixture;
mod stats;

use std::time::Instant;

use config::{parse_cli_args, BenchmarkCliConfig, BenchmarkScenario};
use evidence::{
    validate_evidence_output_paths, write_evidence, EvidenceBundle, OverheadReport, ProfiledSample,
    ProfiledTrialResult, RunProvenance, ThroughputTrialResult,
};
use fixture::{stage_calibration_fixture, validate_calibration_fixture_config};
use pollster::block_on;
use powdergame_core::{chunks_x, chunks_y, WorldConfig};
use powdergame_gpu::{AdapterReport, GpuContext, GpuProfiler, Simulation, PASS_NAMES};
use powdergame_scenarios::{reset_and_stage_scenario, ScenarioId};
use stats::{summarize_profiled_reports, StatSummary, GROUP_LABELS};

fn main() {
    if let Err(error) = run() {
        eprintln!("FATAL: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = parse_cli_args()?;
    let world_config = cli.world_config()?;
    if cli.scenario.is_calibration() {
        validate_calibration_fixture_config(&world_config).map_err(|error| error.to_string())?;
    }
    validate_evidence_output_paths(&cli.csv_output)?;
    let provenance = RunProvenance::capture_for_scenario(cli.scenario);

    print_header(&cli, &world_config, &provenance);

    // Mode A: normal production device, no profiling feature requested.
    println!("\n--- Initializing Mode A production context ---");
    let production_context = block_on(GpuContext::new())
        .map_err(|error| format!("failed to initialize production GPU context: {error}"))?;
    if production_context.profiling_enabled {
        return Err("Mode A unexpectedly enabled timestamp profiling".into());
    }
    let production_adapter = AdapterReport::from_info(&production_context.adapter_info);
    println!("{production_adapter}");
    println!("TIMESTAMP_QUERY requested: NO");

    let mut production_sim = Simulation::with_context(production_context, world_config)
        .map_err(|error| format!("failed to create production Simulation: {error}"))?;
    configure_simulation(&mut production_sim, &cli);

    println!(
        "\n--- Mode A Pre-warm ({:.1}s requested) ---",
        cli.prewarm_secs
    );
    let production_prewarm_ticks = prewarm(&mut production_sim, cli.prewarm_secs, cli.scenario)?;
    println!("Mode A pre-warm completed: {production_prewarm_ticks} ticks");

    println!("\n================================================================================");
    println!("MODE A: Production Throughput (normal device, batch submit, one end wait)");
    println!("================================================================================");
    let throughput_trials = measure_throughput(&mut production_sim, &cli)?;
    let throughput_tps_values: Vec<f64> = throughput_trials
        .iter()
        .map(|trial| trial.sustained_tps)
        .collect();
    let throughput_ms_values: Vec<f64> = throughput_trials
        .iter()
        .map(|trial| trial.wall_ms_per_tick)
        .collect();
    let throughput_tps_stats = StatSummary::from_slice(&throughput_tps_values);
    let throughput_ms_stats = StatSummary::from_slice(&throughput_ms_values);
    println!(
        "Summary: TPS P50 {:.1}, mean {:.1}, min {:.1}, max {:.1}",
        throughput_tps_stats.p50,
        throughput_tps_stats.mean,
        throughput_tps_stats.min,
        throughput_tps_stats.max
    );
    println!(
        "         wall ms/tick P50 {:.4}, mean {:.4}, min {:.4}, max {:.4}",
        throughput_ms_stats.p50,
        throughput_ms_stats.mean,
        throughput_ms_stats.min,
        throughput_ms_stats.max
    );

    // Release the production device before allocating the profiling world.
    drop(production_sim);

    // Mode B: separate timestamp-enabled device and simulation.
    println!("\n--- Initializing Mode B profiling context ---");
    let profiling_context = block_on(GpuContext::with_profiling())
        .map_err(|error| format!("failed to initialize profiling GPU context: {error}"))?;
    if !profiling_context.profiling_enabled {
        return Err("Mode B did not enable timestamp profiling".into());
    }
    let profiling_adapter = AdapterReport::from_info(&profiling_context.adapter_info);
    let profiling_timestamp_period = profiling_context.timestamp_period;
    println!("{profiling_adapter}");
    println!("TIMESTAMP_QUERY requested: YES");
    println!("Timestamp period: {profiling_timestamp_period:.9} ns/tick");
    verify_same_adapter(&production_adapter, &profiling_adapter)?;

    let mut profiling_sim = Simulation::with_context(profiling_context, world_config)
        .map_err(|error| format!("failed to create profiling Simulation: {error}"))?;
    configure_simulation(&mut profiling_sim, &cli);
    let mut profiler = GpuProfiler::new(&profiling_sim.context)
        .map_err(|error| format!("failed to create GpuProfiler: {error}"))?;
    let memory = profiling_sim.tracked_memory_report(Some(&profiler));
    print_memory_report(&memory);

    println!(
        "\n--- Mode B Pre-warm ({:.1}s requested) ---",
        cli.prewarm_secs
    );
    let profiling_prewarm_ticks = prewarm(&mut profiling_sim, cli.prewarm_secs, cli.scenario)?;
    println!("Mode B pre-warm completed: {profiling_prewarm_ticks} ordinary ticks");

    println!("\n================================================================================");
    println!(
        "MODE B: GPU Breakdown (isolated synchronized profiled ticks; {} ticks x {} trials)",
        cli.profile_ticks, cli.trials
    );
    println!("================================================================================");
    let (profiled_trials, profiled_samples) =
        measure_profiled(&mut profiling_sim, &mut profiler, &cli)?;
    let median_profiled_trial = median_profiled_trial_id(&profiled_trials)?;
    let median_trial = profiled_trials
        .iter()
        .find(|trial| trial.trial == median_profiled_trial)
        .ok_or_else(|| "median profiled trial disappeared".to_string())?;
    print_profiled_summary(median_trial);

    // Census is deliberately outside every timed loop.
    let census_tick = profiling_sim.tick_count;
    let census = profiling_sim
        .activity_census_snapshot()
        .map_err(|error| format!("activity census failed: {error}"))?;
    print_census(&census.report, census_tick);

    let overhead = measure_profiled_path_overhead(
        &mut profiling_sim,
        &mut profiler,
        cli.overhead_ticks,
        cli.scenario,
    )?;
    print_overhead(&overhead);

    let evidence = EvidenceBundle {
        config: &cli,
        provenance: &provenance,
        production_adapter: &production_adapter,
        profiling_adapter: &profiling_adapter,
        profiling_timestamp_period,
        production_prewarm_ticks,
        profiling_prewarm_ticks,
        throughput_trials: &throughput_trials,
        throughput_tps_stats: &throughput_tps_stats,
        throughput_ms_stats: &throughput_ms_stats,
        profiled_trials: &profiled_trials,
        median_profiled_trial,
        profiled_samples: &profiled_samples,
        memory: &memory,
        census: &census,
        census_tick,
        overhead: &overhead,
    };
    let evidence_paths = write_evidence(&evidence)?;
    println!(
        "\nAggregate evidence:   {}",
        evidence_paths.summary.display()
    );
    println!(
        "Raw tick evidence:    {}",
        evidence_paths.raw_ticks.display()
    );
    println!(
        "Raw cell evidence:    {}",
        evidence_paths.raw_cells.display()
    );
    println!(
        "Raw chunk evidence:   {}",
        evidence_paths.raw_chunks.display()
    );

    println!("\n================================================================================");
    if cli.scenario.is_calibration() {
        println!(
            "G8-A correction-candidate artifact set written; bind it to a capture receipt before use"
        );
    } else {
        println!("G8-B fixture measurement artifact set written; this is not a G8-C result");
    }
    println!("================================================================================");
    Ok(())
}

fn print_header(cli: &BenchmarkCliConfig, world: &WorldConfig, provenance: &RunProvenance) {
    let chunk_columns = chunks_x(world.width, world.chunk_size);
    let chunk_rows = chunks_y(world.height, world.chunk_size);
    let cell_count = u64::from(world.width) * u64::from(world.height);
    println!("================================================================================");
    if cli.scenario.is_calibration() {
        println!("Powdergame G8-A Headless Performance Measurement Substrate");
    } else {
        println!("Powdergame G8-B Headless Benchmark Fixture");
    }
    println!("================================================================================");
    println!(
        "Evidence schema:       {}",
        cli.scenario.evidence_schema_version()
    );
    if !cli.scenario.is_calibration() {
        if let Some(number) = cli.scenario.number() {
            println!("Scenario number:       {number}");
        }
        println!("Scenario:              {}", cli.scenario.slug());
        println!("Scenario name:         {}", cli.scenario.name());
        println!("Scenario description:  {}", cli.scenario.description());
        println!("Scenario source:       powdergame-scenarios shared staging API");
        println!("Execution surface:     headless simulation only; no gallery rendering/readback");
    }
    println!("Run ID:                {}", provenance.run_id);
    println!("Commit SHA:            {}", provenance.git.commit_sha);
    println!("Git state:             {}", provenance.git.state);
    println!(
        "World:                 {}x{} ({cell_count} cells)",
        world.width, world.height
    );
    println!(
        "Chunks:                {}x{} ({} total, size {})",
        chunk_columns,
        chunk_rows,
        u64::from(chunk_columns) * u64::from(chunk_rows),
        world.chunk_size
    );
    println!(
        "Sleep:                 {} (threshold {})",
        if cli.sleep_enabled { "ON" } else { "OFF" },
        cli.sleep_threshold
    );
    println!("Build profile:         {}", provenance.build_profile);
    println!(
        "Throughput window:     {} ticks x {} trials",
        cli.throughput_ticks, cli.trials
    );
    println!(
        "Profile window:        {} ticks x {} trials",
        cli.profile_ticks, cli.trials
    );
    println!("Overhead control:      {} ticks", cli.overhead_ticks);
}

fn configure_simulation(sim: &mut Simulation, cli: &BenchmarkCliConfig) {
    sim.sleep_enabled = cli.sleep_enabled;
    sim.sleep_threshold = cli.sleep_threshold;
    sim.update_uniforms();
}

fn reset_and_stage(sim: &mut Simulation, scenario: BenchmarkScenario) -> Result<(), String> {
    match shared_staging_scenario(scenario) {
        None => {
            sim.reset()
                .map_err(|error| format!("simulation reset failed: {error}"))?;
            stage_calibration_fixture(sim).map_err(|error| error.to_string())
        }
        Some(scenario) => {
            reset_and_stage_scenario(sim, scenario).map_err(|error| error.to_string())
        }
    }
}

const fn shared_staging_scenario(scenario: BenchmarkScenario) -> Option<ScenarioId> {
    match scenario {
        BenchmarkScenario::Calibration => None,
        BenchmarkScenario::Shared(scenario) => Some(scenario),
    }
}

fn wait_for_gpu(sim: &Simulation, label: &str) -> Result<(), String> {
    sim.context
        .device
        .poll(wgpu::PollType::Wait)
        .map(|_| ())
        .map_err(|error| format!("GPU wait failed during {label}: {error}"))
}

fn reset_stage_and_wait(
    sim: &mut Simulation,
    label: &str,
    scenario: BenchmarkScenario,
) -> Result<(), String> {
    reset_and_stage(sim, scenario)?;
    // Queue writes are scheduled until the next submission; flush them before
    // the wait so fixture uploads cannot enter the measured tick window.
    sim.context.queue.submit([]);
    wait_for_gpu(sim, label)
}

fn prewarm(
    sim: &mut Simulation,
    requested_seconds: f64,
    scenario: BenchmarkScenario,
) -> Result<u64, String> {
    reset_stage_and_wait(sim, "pre-warm fixture staging", scenario)?;
    let start = Instant::now();
    let mut ticks = 0u64;
    while start.elapsed().as_secs_f64() < requested_seconds {
        for _ in 0..128 {
            sim.tick()
                .map_err(|error| format!("pre-warm tick failed: {error}"))?;
            ticks += 1;
        }
        wait_for_gpu(sim, "pre-warm")?;
    }
    Ok(ticks)
}

fn measure_throughput(
    sim: &mut Simulation,
    cli: &BenchmarkCliConfig,
) -> Result<Vec<ThroughputTrialResult>, String> {
    let mut results = Vec::with_capacity(cli.trials as usize);
    for trial in 1..=cli.trials {
        reset_stage_and_wait(
            sim,
            &format!("Mode A trial {trial} fixture staging"),
            cli.scenario,
        )?;
        let start = Instant::now();
        for _ in 0..cli.throughput_ticks {
            sim.tick()
                .map_err(|error| format!("Mode A trial {trial} tick failed: {error}"))?;
        }
        wait_for_gpu(sim, &format!("Mode A trial {trial}"))?;
        let elapsed = start.elapsed();
        let elapsed_wall_ms = elapsed.as_secs_f64() * 1000.0;
        let wall_ms_per_tick = elapsed_wall_ms / f64::from(cli.throughput_ticks);
        let sustained_tps = f64::from(cli.throughput_ticks) / elapsed.as_secs_f64();
        println!(
            "Trial {trial}/{}: {} ticks in {:.2} ms -> {:.4} ms/tick | {:.1} TPS",
            cli.trials, cli.throughput_ticks, elapsed_wall_ms, wall_ms_per_tick, sustained_tps
        );
        results.push(ThroughputTrialResult {
            trial,
            total_ticks: cli.throughput_ticks,
            elapsed_wall_ms,
            wall_ms_per_tick,
            sustained_tps,
        });
    }
    Ok(results)
}

fn measure_profiled(
    sim: &mut Simulation,
    profiler: &mut GpuProfiler,
    cli: &BenchmarkCliConfig,
) -> Result<(Vec<ProfiledTrialResult>, Vec<ProfiledSample>), String> {
    let mut trials = Vec::with_capacity(cli.trials as usize);
    let mut samples = Vec::with_capacity((cli.trials * cli.profile_ticks) as usize);
    for trial in 1..=cli.trials {
        reset_stage_and_wait(
            sim,
            &format!("Mode B trial {trial} fixture staging"),
            cli.scenario,
        )?;
        let mut reports = Vec::with_capacity(cli.profile_ticks as usize);
        for sample_id in 0..cli.profile_ticks {
            let report = sim.tick_profiled(profiler).map_err(|error| {
                format!("Mode B trial {trial} sample {sample_id} failed: {error}")
            })?;
            samples.push(ProfiledSample {
                trial,
                sample_id,
                report: report.clone(),
            });
            reports.push(report);
        }
        let stats = summarize_profiled_reports(&reports);
        println!(
            "Trial {trial}/{}: envelope P50 {:.4} ms (P95 {:.4}), pass sum P50 {:.4}, residual P50 {:.4}",
            cli.trials,
            stats.envelope_stats.p50,
            stats.envelope_stats.p95,
            stats.pass_sum_stats.p50,
            stats.residual_stats.p50
        );
        trials.push(ProfiledTrialResult { trial, stats });
    }
    Ok((trials, samples))
}

fn median_profiled_trial_id(trials: &[ProfiledTrialResult]) -> Result<u32, String> {
    if trials.is_empty() {
        return Err("cannot select a median from zero profiled trials".into());
    }
    let mut ordered: Vec<&ProfiledTrialResult> = trials.iter().collect();
    ordered.sort_by(|left, right| {
        left.stats
            .envelope_stats
            .p50
            .total_cmp(&right.stats.envelope_stats.p50)
    });
    Ok(ordered[ordered.len() / 2].trial)
}

fn print_profiled_summary(trial: &ProfiledTrialResult) {
    println!(
        "\n--- 17-Pass GPU Timing (median-envelope trial {}) ---",
        trial.trial
    );
    println!(
        "{:<3} | {:<24} | {:>9} | {:>9} | {:>9} | {:>12}",
        "#", "Pass", "P50 ms", "P95 ms", "Mean ms", "P50/Env P50"
    );
    for (index, name) in PASS_NAMES.iter().enumerate() {
        let stats = &trial.stats.pass_stats[index];
        let ratio_of_p50s = stats.p50 / trial.stats.envelope_stats.p50 * 100.0;
        println!(
            "{:<3} | {:<24} | {:>9.4} | {:>9.4} | {:>9.4} | {:>11.2}%",
            index + 1,
            name,
            stats.p50,
            stats.p95,
            stats.mean,
            ratio_of_p50s
        );
    }
    println!(
        "Envelope P50 {:.4} ms | Pass Sum P50 {:.4} ms | Residual P50 {:.4} ms",
        trial.stats.envelope_stats.p50,
        trial.stats.pass_sum_stats.p50,
        trial.stats.residual_stats.p50
    );

    println!("\n--- Grouped Subsystem Statistics (percentiles of per-tick sums) ---");
    for (group_index, group_label) in GROUP_LABELS.iter().enumerate() {
        println!(
            "  {:<58} P50 {:>8.4} ms | P95 {:>8.4} ms | per-tick envelope ratio P50 {:>6.2}%",
            group_label,
            trial.stats.grouped_stats[group_index].p50,
            trial.stats.grouped_stats[group_index].p95,
            trial.stats.grouped_envelope_pct_stats[group_index].p50
        );
    }
}

fn print_memory_report(memory: &powdergame_gpu::TrackedMemoryReport) {
    println!("\n--- Persistent Application-Tracked GPU Buffer Allocations ---");
    println!(
        "World dense state:          {} bytes",
        memory.world_dense_state_bytes
    );
    println!(
        "Movement scratch:           {} bytes",
        memory.movement_scratch_bytes
    );
    println!(
        "Activity scratch:           {} bytes",
        memory.activity_scratch_bytes
    );
    println!(
        "Uniforms and tables:        {} bytes",
        memory.uniforms_and_tables_bytes
    );
    println!(
        "Profiler resolve/readback:  {} bytes",
        memory.profiler_bytes
    );
    println!(
        "Total tracked buffers:      {} bytes",
        memory.total_tracked_gpu_bytes
    );
    println!("Scope: persistent requested buffers, not resident VRAM; transient diagnostic readbacks and opaque query/driver storage excluded");
}

fn print_census(census: &powdergame_gpu::ActivityCensusReport, tick: u64) {
    println!("\n--- Out-of-Band Activity Census (tick {tick}) ---");
    println!(
        "Cells total / any active: {} / {}",
        census.total_cells, census.any_active_cells
    );
    println!(
        "Cell bits: Matter {} | Thermal {} | Pressure {} | Reaction {}",
        census.matter_active_cells,
        census.thermal_active_cells,
        census.pressure_active_cells,
        census.reaction_active_cells
    );
    println!(
        "Chunks: total {} | active {} | runnable {} | sleeping {}",
        census.total_chunks, census.active_chunks, census.runnable_chunks, census.sleeping_chunks
    );
    println!("Note: active is activity-pass output; runnable/sleeping are wake-pass output. Active overlaps those state categories, so all displayed percentages are not expected to sum to 100%.");
}

fn measure_profiled_path_overhead(
    sim: &mut Simulation,
    profiler: &mut GpuProfiler,
    ticks: u32,
    scenario: BenchmarkScenario,
) -> Result<OverheadReport, String> {
    println!("\n--- Profiling Cadence Controls ({ticks} ticks each) ---");

    reset_stage_and_wait(sim, "batched overhead control fixture staging", scenario)?;
    let start = Instant::now();
    for _ in 0..ticks {
        sim.tick()
            .map_err(|error| format!("batched overhead control tick failed: {error}"))?;
    }
    wait_for_gpu(sim, "batched overhead control")?;
    let batched_unprofiled_ms = start.elapsed().as_secs_f64() * 1000.0;

    reset_stage_and_wait(
        sim,
        "synchronized overhead control fixture staging",
        scenario,
    )?;
    let start = Instant::now();
    for _ in 0..ticks {
        sim.tick()
            .map_err(|error| format!("synchronized overhead control tick failed: {error}"))?;
        wait_for_gpu(sim, "per-tick synchronized unprofiled control")?;
    }
    let synchronized_unprofiled_ms = start.elapsed().as_secs_f64() * 1000.0;

    reset_stage_and_wait(sim, "profiled overhead control fixture staging", scenario)?;
    let start = Instant::now();
    for _ in 0..ticks {
        sim.tick_profiled(profiler)
            .map_err(|error| format!("profiled overhead tick failed: {error}"))?;
    }
    let synchronized_profiled_ms = start.elapsed().as_secs_f64() * 1000.0;

    let percent_delta = |baseline: f64, measured: f64| (measured / baseline - 1.0) * 100.0;
    Ok(OverheadReport {
        ticks,
        batched_unprofiled_ms,
        synchronized_unprofiled_ms,
        synchronized_profiled_ms,
        synchronization_overhead_pct: percent_delta(
            batched_unprofiled_ms,
            synchronized_unprofiled_ms,
        ),
        profiling_increment_pct: percent_delta(
            synchronized_unprofiled_ms,
            synchronized_profiled_ms,
        ),
        total_profiled_path_overhead_pct: percent_delta(
            batched_unprofiled_ms,
            synchronized_profiled_ms,
        ),
    })
}

fn print_overhead(overhead: &OverheadReport) {
    println!(
        "Batched unprofiled:       {:.2} ms",
        overhead.batched_unprofiled_ms
    );
    println!(
        "Synchronized unprofiled: {:.2} ms",
        overhead.synchronized_unprofiled_ms
    );
    println!(
        "Synchronized profiled:   {:.2} ms",
        overhead.synchronized_profiled_ms
    );
    println!(
        "Synchronization overhead control: {:.2}%",
        overhead.synchronization_overhead_pct
    );
    println!(
        "Profiling increment over synchronized control: {:.2}%",
        overhead.profiling_increment_pct
    );
    println!(
        "Observed profiled-path overhead vs batch: {:.2}%",
        overhead.total_profiled_path_overhead_pct
    );
    println!("The total is a combined-path delta: timestamp writes, resolve/copy, per-tick synchronization, map/readback, CPU orchestration, and lost pipelining. It is not attributed to one mechanism.");
    println!("Mode B envelopes describe isolated fully synchronized profiled ticks and are not a one-to-one replacement for sustained Mode A wall time.");
}

fn verify_same_adapter(
    production: &AdapterReport,
    profiling: &AdapterReport,
) -> Result<(), String> {
    if production.name == profiling.name
        && production.vendor == profiling.vendor
        && production.device == profiling.device
        && production.backend == profiling.backend
    {
        Ok(())
    } else {
        Err(format!(
            "Mode A and Mode B selected different adapters: production={} ({}/{}) profiling={} ({}/{})",
            production.name,
            production.vendor,
            production.device,
            profiling.name,
            profiling.vendor,
            profiling.device
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_stays_legacy_and_all_scenario_slugs_use_shared_staging() {
        assert_eq!(
            shared_staging_scenario(BenchmarkScenario::Calibration),
            None
        );

        for slug in [
            "sand-fall",
            "water-flow",
            "fire-heat",
            "pressure-burst",
            "heavy-mixed-world",
            "active-sleep-g7",
        ] {
            let scenario: BenchmarkScenario = slug.parse().unwrap();
            assert_eq!(
                shared_staging_scenario(scenario).map(ScenarioId::slug),
                Some(slug)
            );
        }
    }
}
