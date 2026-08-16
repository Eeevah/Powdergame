//! G8-A Headless Performance Benchmark & Measurement Harness.
//!
//! Features:
//! 1. Mode A: Production Throughput Mode (unprofiled Simulation::tick(), batch-submitted, GPU wait ONCE at end).
//! 2. Mode B: GPU Breakdown Mode (timestamp-profiled Simulation::tick_profiled(), 17 raw passes + tick envelope).
//! 3. Out-of-band Activity Census (cells + chunks).
//! 4. Application-tracked GPU Allocation Memory Report.
//! 5. Statistical Aggregations (P50, P95, Mean, Min, Max across trials).
//! 6. Dual Output: Concise human-readable console summary + machine-readable CSV.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use pollster::block_on;
use powdergame_core::{
    WorldConfig, FLAG_COMBUSTING, MATERIAL_OIL, MATERIAL_SAND, MATERIAL_STONE, MATERIAL_WATER,
    MATERIAL_WOOD,
};
use powdergame_gpu::{
    AdapterReport, GpuContext, GpuProfiler, ProfiledTickReport, Simulation, PASS_COUNT, PASS_NAMES,
};

/// Command-line configuration for the G8 benchmark.
#[derive(Debug, Clone)]
struct BenchmarkCliConfig {
    pub width: u32,
    pub height: u32,
    pub chunk_size: u32,
    pub sleep_enabled: bool,
    pub sleep_threshold: u32,
    pub prewarm_secs: f64,
    pub throughput_ticks: u32,
    pub profile_ticks: u32,
    pub trials: u32,
    pub csv_output: PathBuf,
}

impl Default for BenchmarkCliConfig {
    fn default() -> Self {
        Self {
            width: 2048,
            height: 2048,
            chunk_size: 64,
            sleep_enabled: true,
            sleep_threshold: 16,
            prewarm_secs: 2.0,
            throughput_ticks: 1024,
            profile_ticks: 256,
            trials: 3,
            csv_output: PathBuf::from("target/calibration_report.csv"),
        }
    }
}

fn parse_cli_args() -> BenchmarkCliConfig {
    let mut config = BenchmarkCliConfig::default();
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--width" if i + 1 < args.len() => {
                config.width = args[i + 1].parse().unwrap_or(2048);
                i += 2;
            }
            "--height" if i + 1 < args.len() => {
                config.height = args[i + 1].parse().unwrap_or(2048);
                i += 2;
            }
            "--chunk" if i + 1 < args.len() => {
                config.chunk_size = args[i + 1].parse().unwrap_or(64);
                i += 2;
            }
            "--sleep" if i + 1 < args.len() => {
                config.sleep_enabled = !args[i + 1].eq_ignore_ascii_case("off")
                    && !args[i + 1].eq_ignore_ascii_case("false");
                i += 2;
            }
            "--threshold" if i + 1 < args.len() => {
                config.sleep_threshold = args[i + 1].parse().unwrap_or(16);
                i += 2;
            }
            "--prewarm-secs" if i + 1 < args.len() => {
                config.prewarm_secs = args[i + 1].parse().unwrap_or(2.0);
                i += 2;
            }
            "--throughput-ticks" if i + 1 < args.len() => {
                config.throughput_ticks = args[i + 1].parse().unwrap_or(1024);
                i += 2;
            }
            "--profile-ticks" if i + 1 < args.len() => {
                config.profile_ticks = args[i + 1].parse().unwrap_or(256);
                i += 2;
            }
            "--trials" if i + 1 < args.len() => {
                config.trials = args[i + 1].parse().unwrap_or(3);
                i += 2;
            }
            "--csv" if i + 1 < args.len() => {
                config.csv_output = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }
    config
}

/// Stages a repeatable, rich calibration fixture on the world.
/// Exercises Sand fall, Water flow, Boiling water heater with Steam expansion pressure,
/// Burning Wood combustion + Smoke decay, weak Wood rupture opening, and stable Water bulk.
fn stage_calibration_fixture(sim: &mut Simulation) {
    let w = sim.world.config.width as usize;
    let h = sim.world.config.height as usize;
    let cell_count = w * h;

    let mut materials = powdergame_core::initial_material_ids(&sim.world.config).unwrap();
    let mut temperatures = vec![0.0f32; cell_count];
    let mut flags = vec![0u32; cell_count];

    // Quadrant 1: Falling Sand streams
    for cx in (100..400).step_by(50) {
        for y in 100..500 {
            for x in (cx - 10)..(cx + 10) {
                materials[y * w + x] = MATERIAL_SAND;
            }
        }
    }

    // Quadrant 2: Water & Oil tanks
    for y in 800..1000 {
        for x in 100..400 {
            materials[y * w + x] = MATERIAL_WATER;
        }
        for x in 500..800 {
            materials[y * w + x] = MATERIAL_OIL;
        }
    }

    // Quadrant 3: Boiling water boiler with steam expansion pressure
    for y in 1200..1400 {
        for x in 200..400 {
            materials[y * w + x] = MATERIAL_WATER;
            temperatures[y * w + x] = 120.0;
        }
    }
    // Wood relief walls around boiler
    for x in 190..410 {
        materials[1190 * w + x] = MATERIAL_WOOD;
        materials[1410 * w + x] = MATERIAL_WOOD;
    }
    for y in 1190..1410 {
        materials[y * w + 190] = MATERIAL_WOOD;
        materials[y * w + 410] = MATERIAL_WOOD;
    }

    // Quadrant 4: Burning Wood line + Smoke generation
    for x in 1000..1600 {
        for y in 300..320 {
            let idx = y * w + x;
            materials[idx] = MATERIAL_WOOD;
            if x % 10 == 0 {
                flags[idx] = FLAG_COMBUSTING;
                temperatures[idx] = 500.0;
            }
        }
    }

    // Stable bulk Water in deep basin (for sleep observation)
    for y in 1500..1900 {
        for x in 1000..1900 {
            let idx = y * w + x;
            materials[idx] = MATERIAL_WATER;
            temperatures[idx] = 20.0;
        }
    }
    for x in 990..1910 {
        materials[1900 * w + x] = MATERIAL_STONE;
    }
    for y in 1500..1901 {
        materials[y * w + 990] = MATERIAL_STONE;
        materials[y * w + 1910] = MATERIAL_STONE;
    }

    let q = &sim.context.queue;
    let mut mat_bytes = Vec::with_capacity(cell_count * 4);
    for m in &materials {
        mat_bytes.extend_from_slice(&m.to_ne_bytes());
    }
    let mut temp_bytes = Vec::with_capacity(cell_count * 4);
    for t in &temperatures {
        temp_bytes.extend_from_slice(&t.to_ne_bytes());
    }
    let mut flag_bytes = Vec::with_capacity(cell_count * 4);
    for f in &flags {
        flag_bytes.extend_from_slice(&f.to_ne_bytes());
    }

    q.write_buffer(&sim.world.material_current, 0, &mat_bytes);
    q.write_buffer(&sim.world.material_next, 0, &mat_bytes);
    q.write_buffer(&sim.world.temperature_current, 0, &temp_bytes);
    q.write_buffer(&sim.world.temperature_next, 0, &temp_bytes);
    q.write_buffer(&sim.world.flags_current, 0, &flag_bytes);
    q.write_buffer(&sim.world.flags_next, 0, &flag_bytes);
}

/// Computes percentile (0..100) from a sorted f64 slice.
fn percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (pct / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Statistics for a series of numeric measurements.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct StatSummary {
    pub count: usize,
    pub p50: f64,
    pub p95: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
}

impl StatSummary {
    pub fn from_slice(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self {
                count: 0,
                p50: 0.0,
                p95: 0.0,
                mean: 0.0,
                min: 0.0,
                max: 0.0,
            };
        }
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let sum: f64 = sorted.iter().sum();
        let mean = sum / sorted.len() as f64;
        let p50 = percentile(&sorted, 50.0);
        let p95 = percentile(&sorted, 95.0);
        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        Self {
            count: sorted.len(),
            p50,
            p95,
            mean,
            min,
            max,
        }
    }
}

/// Result of a single production throughput trial.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ThroughputTrialResult {
    pub total_ticks: u32,
    pub elapsed_wall_ms: f64,
    pub wall_ms_per_tick: f64,
    pub sustained_tps: f64,
}

/// Result of a single GPU breakdown trial.
#[derive(Debug, Clone)]
struct ProfiledTrialResult {
    pub pass_stats: [StatSummary; PASS_COUNT],
    pub envelope_stats: StatSummary,
    pub pass_sum_stats: StatSummary,
    pub residual_stats: StatSummary,
}

fn main() {
    let cli = parse_cli_args();

    println!("================================================================================");
    println!("Powdergame G8-A Headless Performance Measurement Substrate");
    println!("================================================================================");

    let world_config = WorldConfig {
        width: cli.width,
        height: cli.height,
        chunk_size: cli.chunk_size,
    };

    println!(
        "World Configuration:  {}x{} (cell count: {})",
        cli.width,
        cli.height,
        cli.width * cli.height
    );
    println!(
        "Chunk Size:           {}x{} (chunks: {}x{} = {})",
        cli.chunk_size,
        cli.chunk_size,
        cli.width / cli.chunk_size,
        cli.height / cli.chunk_size,
        (cli.width / cli.chunk_size) * (cli.height / cli.chunk_size)
    );
    println!(
        "Sleep Optimization:   {} (Threshold: {} ticks)",
        if cli.sleep_enabled { "ON" } else { "OFF" },
        cli.sleep_threshold
    );
    println!(
        "Build Profile:        {}",
        if cfg!(debug_assertions) {
            "DEBUG (unoptimized)"
        } else {
            "RELEASE (opt-level=3)"
        }
    );
    println!("Pre-warm Duration:    {:.1}s", cli.prewarm_secs);
    println!(
        "Throughput Ticks:     {} ticks per trial ({} trials)",
        cli.throughput_ticks, cli.trials
    );
    println!(
        "Profiled Ticks:       {} ticks per trial ({} trials)",
        cli.profile_ticks, cli.trials
    );

    // 1. Initialize GPU context with profiling
    let ctx = match block_on(GpuContext::with_profiling()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("FATAL: Failed to initialize GPU context with TIMESTAMP_QUERY: {e}");
            std::process::exit(1);
        }
    };

    let report = AdapterReport::from_info(&ctx.adapter_info);
    println!("\n--- Adapter & Hardware Info ---");
    println!("{}", report);
    println!("TIMESTAMP_QUERY:      SUPPORTED");
    println!("Timestamp Period:     {:.6} ns/tick", ctx.timestamp_period);

    let mut sim = Simulation::with_context(ctx, world_config).expect("failed to create Simulation");
    sim.sleep_enabled = cli.sleep_enabled;
    sim.sleep_threshold = cli.sleep_threshold;
    sim.update_uniforms();

    let mut profiler = GpuProfiler::new(&sim.context).expect("failed to create GpuProfiler");
    let mem_report = sim.tracked_memory_report(Some(&profiler));

    println!("\n--- Tracked GPU Buffer Allocations ---");
    println!(
        "World Dense State:    {:.2} MB ({} bytes)",
        mem_report.world_dense_state_bytes as f64 / 1_048_576.0,
        mem_report.world_dense_state_bytes
    );
    println!(
        "Movement Scratch:     {:.2} MB ({} bytes)",
        mem_report.movement_scratch_bytes as f64 / 1_048_576.0,
        mem_report.movement_scratch_bytes
    );
    println!(
        "Activity Diagnostics: {:.2} MB ({} bytes)",
        mem_report.activity_scratch_bytes as f64 / 1_048_576.0,
        mem_report.activity_scratch_bytes
    );
    println!(
        "Uniforms & Tables:    {:.2} KB ({} bytes)",
        mem_report.uniforms_and_tables_bytes as f64 / 1024.0,
        mem_report.uniforms_and_tables_bytes
    );
    println!("Profiler Staging:     {} bytes", mem_report.profiler_bytes);
    println!(
        "Total Tracked GPU:    {:.2} MB ({} bytes)",
        mem_report.total_tracked_gpu_bytes as f64 / 1_048_576.0,
        mem_report.total_tracked_gpu_bytes
    );

    // 2. Pre-warm phase
    println!("\n--- Pre-warm Phase ({:.1}s) ---", cli.prewarm_secs);
    stage_calibration_fixture(&mut sim);
    let prewarm_start = Instant::now();
    let mut prewarm_ticks = 0u64;
    while prewarm_start.elapsed().as_secs_f64() < cli.prewarm_secs {
        for _ in 0..128 {
            sim.tick().expect("prewarm tick failed");
            prewarm_ticks += 1;
        }
        let _ = sim.context.device.poll(wgpu::PollType::Wait);
    }
    println!(
        "Pre-warm completed: {} ticks in {:.2}s",
        prewarm_ticks,
        prewarm_start.elapsed().as_secs_f64()
    );

    // 3. Mode A: Production Throughput Measurement
    println!("\n================================================================================");
    println!("MODE A: Production Throughput (Unprofiled, Batch-Submitted, End-Wait Once)");
    println!("================================================================================");

    let mut throughput_results = Vec::new();
    for trial in 1..=cli.trials {
        sim.reset().expect("reset failed");
        stage_calibration_fixture(&mut sim);

        let t_start = Instant::now();
        for _ in 0..cli.throughput_ticks {
            sim.tick().expect("production tick failed");
        }
        let _ = sim.context.device.poll(wgpu::PollType::Wait);
        let elapsed = t_start.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        let ms_per_tick = elapsed_ms / cli.throughput_ticks as f64;
        let tps = (cli.throughput_ticks as f64) / elapsed.as_secs_f64();

        println!(
            "Trial {}/{}: {} ticks in {:.2} ms -> {:.4} ms/tick | {:.1} TPS",
            trial, cli.trials, cli.throughput_ticks, elapsed_ms, ms_per_tick, tps
        );

        throughput_results.push(ThroughputTrialResult {
            total_ticks: cli.throughput_ticks,
            elapsed_wall_ms: elapsed_ms,
            wall_ms_per_tick: ms_per_tick,
            sustained_tps: tps,
        });
    }

    let tps_values: Vec<f64> = throughput_results.iter().map(|r| r.sustained_tps).collect();
    let ms_values: Vec<f64> = throughput_results
        .iter()
        .map(|r| r.wall_ms_per_tick)
        .collect();
    let tps_stats = StatSummary::from_slice(&tps_values);
    let ms_stats = StatSummary::from_slice(&ms_values);

    println!("\nThroughput Summary Across {} Trials:", cli.trials);
    println!(
        "  Sustained TPS:  Median = {:.1} TPS | Mean = {:.1} TPS | Min = {:.1} | Max = {:.1}",
        tps_stats.p50, tps_stats.mean, tps_stats.min, tps_stats.max
    );
    println!(
        "  Wall Time/Tick: Median = {:.4} ms  | Mean = {:.4} ms  | Min = {:.4} | Max = {:.4}",
        ms_stats.p50, ms_stats.mean, ms_stats.min, ms_stats.max
    );

    // 4. Mode B: GPU Breakdown Measurement (Profiled)
    println!("\n================================================================================");
    println!(
        "MODE B: GPU Breakdown (17 Timestamp-Profiled Passes, {} ticks, {} trials)",
        cli.profile_ticks, cli.trials
    );
    println!("================================================================================");

    let mut profiled_trials: Vec<ProfiledTrialResult> = Vec::new();
    let mut all_reports: Vec<ProfiledTickReport> = Vec::new();

    for trial in 1..=cli.trials {
        sim.reset().expect("reset failed");
        stage_calibration_fixture(&mut sim);

        let mut trial_reports: Vec<ProfiledTickReport> = Vec::new();
        for _ in 0..cli.profile_ticks {
            let rep = sim
                .tick_profiled(&mut profiler)
                .expect("profiled tick failed");
            trial_reports.push(rep);
        }

        let envelope_vals: Vec<f64> = trial_reports
            .iter()
            .map(|r| r.gpu_tick_envelope_ms)
            .collect();
        let pass_sum_vals: Vec<f64> = trial_reports.iter().map(|r| r.gpu_pass_sum_ms).collect();
        let residual_vals: Vec<f64> = trial_reports.iter().map(|r| r.residual_ms).collect();

        let mut pass_stats: Vec<StatSummary> = Vec::new();
        for p_idx in 0..PASS_COUNT {
            let p_vals: Vec<f64> = trial_reports
                .iter()
                .map(|r| r.passes[p_idx].duration_ms)
                .collect();
            pass_stats.push(StatSummary::from_slice(&p_vals));
        }

        let trial_res = ProfiledTrialResult {
            pass_stats: pass_stats.try_into().unwrap(),
            envelope_stats: StatSummary::from_slice(&envelope_vals),
            pass_sum_stats: StatSummary::from_slice(&pass_sum_vals),
            residual_stats: StatSummary::from_slice(&residual_vals),
        };

        println!("Trial {}/{}: Envelope Median = {:.4} ms (P95 = {:.4} ms), Pass Sum = {:.4} ms, Residual = {:.4} ms",
            trial, cli.trials, trial_res.envelope_stats.p50, trial_res.envelope_stats.p95, trial_res.pass_sum_stats.p50, trial_res.residual_stats.p50
        );

        profiled_trials.push(trial_res);
        all_reports.extend(trial_reports);
    }

    // Select median trial based on envelope median
    profiled_trials.sort_by(|a, b| {
        a.envelope_stats
            .p50
            .partial_cmp(&b.envelope_stats.p50)
            .unwrap()
    });
    let median_trial = &profiled_trials[profiled_trials.len() / 2];

    println!("\n--- Raw 17 Pass GPU Timing Breakdown (Median Trial Summary) ---");
    println!(
        "{:<3} | {:<24} | {:>9} | {:>9} | {:>9} | {:>8}",
        "#", "Pass Name", "P50 (ms)", "P95 (ms)", "Mean (ms)", "% Envelop"
    );
    println!("----+--------------------------+-----------+-----------+-----------+---------");
    for (i, &name) in PASS_NAMES.iter().enumerate() {
        let st = &median_trial.pass_stats[i];
        let pct = (st.p50 / median_trial.envelope_stats.p50) * 100.0;
        println!(
            "{:<3} | {:<24} | {:>9.4} | {:>9.4} | {:>9.4} | {:>7.2}%",
            i + 1,
            name,
            st.p50,
            st.p95,
            st.mean,
            pct
        );
    }
    println!("----+--------------------------+-----------+-----------+-----------+---------");
    println!(
        "    | {:<24} | {:>9.4} | {:>9.4} | {:>9.4} | {:>7.2}%",
        "GPU Pass Sum",
        median_trial.pass_sum_stats.p50,
        median_trial.pass_sum_stats.p95,
        median_trial.pass_sum_stats.mean,
        (median_trial.pass_sum_stats.p50 / median_trial.envelope_stats.p50) * 100.0
    );
    println!(
        "    | {:<24} | {:>9.4} | {:>9.4} | {:>9.4} | {:>7.2}%",
        "GPU Tick Envelope",
        median_trial.envelope_stats.p50,
        median_trial.envelope_stats.p95,
        median_trial.envelope_stats.mean,
        100.0
    );
    println!(
        "    | {:<24} | {:>9.4} | {:>9.4} | {:>9.4} | {:>7.2}%",
        "Diagnostic Residual",
        median_trial.residual_stats.p50,
        median_trial.residual_stats.p95,
        median_trial.residual_stats.mean,
        (median_trial.residual_stats.p50 / median_trial.envelope_stats.p50) * 100.0
    );

    // Grouped summary
    let matter_p50 = median_trial.pass_stats[1].p50 + median_trial.pass_stats[3].p50;
    let claim_p50 = median_trial.pass_stats[2].p50
        + median_trial.pass_stats[6].p50
        + median_trial.pass_stats[11].p50;
    let thermal_p50 = median_trial.pass_stats[4].p50;
    let reaction_p50 = median_trial.pass_stats[5].p50
        + median_trial.pass_stats[7].p50
        + median_trial.pass_stats[8].p50
        + median_trial.pass_stats[9].p50
        + median_trial.pass_stats[10].p50
        + median_trial.pass_stats[12].p50;
    let pressure_p50 = median_trial.pass_stats[13].p50 + median_trial.pass_stats[14].p50;
    let active_sleep_p50 = median_trial.pass_stats[0].p50
        + median_trial.pass_stats[15].p50
        + median_trial.pass_stats[16].p50;

    println!("\n--- Grouped Subsystem Roll-Up (P50) ---");
    println!(
        "  Matter Movement (propose + commit):              {:>8.4} ms ({:.1}%)",
        matter_p50,
        matter_p50 / median_trial.envelope_stats.p50 * 100.0
    );
    println!(
        "  Ownership / Claim (move + exp + smoke claims):   {:>8.4} ms ({:.1}%)",
        claim_p50,
        claim_p50 / median_trial.envelope_stats.p50 * 100.0
    );
    println!(
        "  Thermal Conduction:                              {:>8.4} ms ({:.1}%)",
        thermal_p50,
        thermal_p50 / median_trial.envelope_stats.p50 * 100.0
    );
    println!(
        "  Reaction & Phase (phase, exp, decay, combustion):{:>8.4} ms ({:.1}%)",
        reaction_p50,
        reaction_p50 / median_trial.envelope_stats.p50 * 100.0
    );
    println!(
        "  Pressure & Rupture:                              {:>8.4} ms ({:.1}%)",
        pressure_p50,
        pressure_p50 / median_trial.envelope_stats.p50 * 100.0
    );
    println!(
        "  Active / Sleep Management (wake, prop, red):     {:>8.4} ms ({:.1}%)",
        active_sleep_p50,
        active_sleep_p50 / median_trial.envelope_stats.p50 * 100.0
    );

    // 5. Activity Census Snapshot
    println!(
        "\n--- Activity Census Snapshot (at tick {}) ---",
        sim.tick_count
    );
    let census = sim.activity_census().expect("activity census failed");
    println!("  Cells Total:       {}", census.total_cells);
    println!(
        "  Cells Any Active:  {} ({:.2}%)",
        census.any_active_cells,
        (census.any_active_cells as f64 / census.total_cells as f64) * 100.0
    );
    println!("  Cells Matter:      {}", census.matter_active_cells);
    println!("  Cells Thermal:     {}", census.thermal_active_cells);
    println!("  Cells Pressure:    {}", census.pressure_active_cells);
    println!("  Cells Reaction:    {}", census.reaction_active_cells);
    println!("  Chunks Total:      {}", census.total_chunks);
    println!(
        "  Chunks Active:     {} ({:.1}%)",
        census.active_chunks,
        (census.active_chunks as f64 / census.total_chunks as f64) * 100.0
    );
    println!(
        "  Chunks Runnable:   {} ({:.1}%)",
        census.runnable_chunks,
        (census.runnable_chunks as f64 / census.total_chunks as f64) * 100.0
    );
    println!(
        "  Chunks Sleeping:   {} ({:.1}%)",
        census.sleeping_chunks,
        (census.sleeping_chunks as f64 / census.total_chunks as f64) * 100.0
    );

    // 6. Profiling Overhead Evaluation
    // Compare unprofiled 256 ticks vs profiled 256 ticks
    println!("\n--- Profiling Overhead Evaluation (256-tick matched run) ---");
    sim.reset().expect("reset failed");
    stage_calibration_fixture(&mut sim);
    let t_unprof_start = Instant::now();
    for _ in 0..256 {
        sim.tick().unwrap();
    }
    let _ = sim.context.device.poll(wgpu::PollType::Wait);
    let unprof_ms = t_unprof_start.elapsed().as_secs_f64() * 1000.0;

    sim.reset().expect("reset failed");
    stage_calibration_fixture(&mut sim);
    let t_prof_start = Instant::now();
    for _ in 0..256 {
        sim.tick_profiled(&mut profiler).unwrap();
    }
    let _ = sim.context.device.poll(wgpu::PollType::Wait);
    let prof_ms = t_prof_start.elapsed().as_secs_f64() * 1000.0;

    let overhead_pct = ((prof_ms - unprof_ms) / unprof_ms) * 100.0;
    println!(
        "  Unprofiled 256 ticks: {:.2} ms ({:.4} ms/tick)",
        unprof_ms,
        unprof_ms / 256.0
    );
    println!(
        "  Profiled 256 ticks:   {:.2} ms ({:.4} ms/tick)",
        prof_ms,
        prof_ms / 256.0
    );
    println!(
        "  Observed Overhead:    {:.2}% (readback + map per tick)",
        overhead_pct
    );

    // 7. Write Structured Machine-Readable CSV Report
    if let Some(parent) = cli.csv_output.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = File::create(&cli.csv_output) {
        writeln!(f, "# Powdergame G8-A Calibration Benchmark Report").unwrap();
        writeln!(f, "metric_type,name,p50_ms,p95_ms,mean_ms,min_ms,max_ms").unwrap();
        for (i, &name) in PASS_NAMES.iter().enumerate() {
            let st = &median_trial.pass_stats[i];
            writeln!(
                f,
                "pass,{},{:.6},{:.6},{:.6},{:.6},{:.6}",
                name, st.p50, st.p95, st.mean, st.min, st.max
            )
            .unwrap();
        }
        writeln!(
            f,
            "envelope,gpu_tick_envelope,{:.6},{:.6},{:.6},{:.6},{:.6}",
            median_trial.envelope_stats.p50,
            median_trial.envelope_stats.p95,
            median_trial.envelope_stats.mean,
            median_trial.envelope_stats.min,
            median_trial.envelope_stats.max
        )
        .unwrap();
        writeln!(
            f,
            "envelope,gpu_pass_sum,{:.6},{:.6},{:.6},{:.6},{:.6}",
            median_trial.pass_sum_stats.p50,
            median_trial.pass_sum_stats.p95,
            median_trial.pass_sum_stats.mean,
            median_trial.pass_sum_stats.min,
            median_trial.pass_sum_stats.max
        )
        .unwrap();
        writeln!(
            f,
            "envelope,residual,{:.6},{:.6},{:.6},{:.6},{:.6}",
            median_trial.residual_stats.p50,
            median_trial.residual_stats.p95,
            median_trial.residual_stats.mean,
            median_trial.residual_stats.min,
            median_trial.residual_stats.max
        )
        .unwrap();
        writeln!(
            f,
            "throughput,wall_ms_per_tick,{:.6},{:.6},{:.6},{:.6},{:.6}",
            ms_stats.p50, ms_stats.p95, ms_stats.mean, ms_stats.min, ms_stats.max
        )
        .unwrap();
        writeln!(
            f,
            "throughput,sustained_tps,{:.2},{:.2},{:.2},{:.2},{:.2}",
            tps_stats.p50, tps_stats.p95, tps_stats.mean, tps_stats.min, tps_stats.max
        )
        .unwrap();
        println!(
            "\nStructured CSV report saved to: {}",
            cli.csv_output.display()
        );
    }

    println!("\n================================================================================");
    println!("G8-A Calibration Complete: Trustworthy Measurement Substrate Established");
    println!("================================================================================");
}
