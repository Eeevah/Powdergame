//! One-shot TE-2 correctness-baseline timing capture.
//!
//! This ignored test is intentionally outside normal validation. It profiles
//! the production 34-pass tick graph after the final source is frozen and
//! writes one create-new CSV selected by `TE2_PERFORMANCE_OUTPUT`.

use std::{
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};

use pollster::block_on;
use powdergame_core::{vacuum_air_state, WorldConfig, MATERIAL_STONE};
use powdergame_gpu::{GpuContext, GpuProfiler, ProfiledTickReport, Simulation};

const PROFILE_TICKS: usize = 32;
const PREWARM_TICKS: usize = 8;

fn percentile(mut values: Vec<f64>, p: f64) -> f64 {
    values.sort_by(f64::total_cmp);
    let index = (p * (values.len() - 1) as f64).round() as usize;
    values[index]
}

fn pass_values(reports: &[ProfiledTickReport], index: usize) -> Vec<f64> {
    reports
        .iter()
        .map(|report| report.passes[index].duration_ms)
        .collect()
}

fn stage_local_frontier(simulation: &Simulation) {
    let x = i64::from(simulation.world.config.width / 2);
    let y = i64::from(simulation.world.config.height / 2);
    simulation
        .world
        .write_environment_cell_for_test(&simulation.context.queue, x, y, vacuum_air_state())
        .unwrap();
    simulation
        .world
        .write_material(&simulation.context.queue, x + 2, y, MATERIAL_STONE)
        .unwrap();
    simulation
        .world
        .write_temperature(&simulation.context.queue, x + 2, y, 300.0)
        .unwrap();
}

fn capture_row(label: &str, config: WorldConfig, local_frontier: bool) -> String {
    let context = block_on(GpuContext::with_profiling()).expect("profiling context");
    let mut simulation = Simulation::with_context(context, config).expect("simulation");
    simulation.set_sleep_enabled(true);
    simulation.set_sleep_threshold(16);
    if local_frontier {
        stage_local_frontier(&simulation);
    }
    for _ in 0..PREWARM_TICKS {
        simulation.tick().expect("prewarm tick");
    }
    let mut profiler = GpuProfiler::new(&simulation.context).expect("profiler");
    let start = Instant::now();
    let mut reports = Vec::with_capacity(PROFILE_TICKS);
    for _ in 0..PROFILE_TICKS {
        reports.push(
            simulation
                .tick_profiled(&mut profiler)
                .expect("profiled tick"),
        );
    }
    let wall = start.elapsed();
    let census = simulation.activity_census().expect("terminal census");
    let memory = simulation.tracked_memory_report(Some(&profiler));
    let envelopes: Vec<_> = reports
        .iter()
        .map(|report| report.gpu_tick_envelope_ms)
        .collect();
    let pass = |index| {
        let values = pass_values(&reports, index);
        (percentile(values.clone(), 0.50), percentile(values, 0.95))
    };
    let air_scale = pass(6);
    let air_commit = pass(7);
    let thermal_scale = pass(8);
    let thermal_commit = pass(9);
    let environment_activity = pass(32);
    format!(
        "{label},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{},{},{},{}",
        config.width,
        config.height,
        PROFILE_TICKS,
        percentile(envelopes.clone(), 0.50),
        percentile(envelopes, 0.95),
        wall.as_secs_f64() * 1000.0 / PROFILE_TICKS as f64,
        air_scale.0,
        air_scale.1,
        air_commit.0,
        air_commit.1,
        thermal_scale.0,
        thermal_scale.1,
        thermal_commit.0,
        thermal_commit.1,
        environment_activity.0,
        environment_activity.1,
        census.any_active_cells,
        census.active_chunks,
        census.sleeping_chunks,
        memory.total_tracked_gpu_bytes,
        simulation.tick_count,
    )
}

#[test]
#[ignore = "one bounded final-source TE-2 performance capture only"]
fn capture_te2_correctness_baseline_once() {
    let output = PathBuf::from(
        std::env::var_os("TE2_PERFORMANCE_OUTPUT")
            .expect("TE2_PERFORMANCE_OUTPUT must name a create-new CSV"),
    );
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .expect("performance output must not already exist");
    writeln!(
        file,
        "scenario,width,height,profile_ticks,gpu_tick_p50_ms,gpu_tick_p95_ms,synchronized_wall_ms_per_tick,air_flow_scale_p50_ms,air_flow_scale_p95_ms,air_transport_p50_ms,air_transport_p95_ms,thermal_scale_p50_ms,thermal_scale_p95_ms,unified_thermal_p50_ms,unified_thermal_p95_ms,environment_activity_p50_ms,environment_activity_p95_ms,terminal_active_cells,terminal_active_chunks,terminal_sleeping_chunks,tracked_gpu_bytes,terminal_tick"
    )
    .unwrap();
    for (label, config, local) in [
        (
            "candidate-256-local",
            WorldConfig::new(256, 256, 64).unwrap(),
            true,
        ),
        (
            "equilibrium-2048",
            WorldConfig::new(2048, 2048, 64).unwrap(),
            false,
        ),
        (
            "frontier-2048",
            WorldConfig::new(2048, 2048, 64).unwrap(),
            true,
        ),
    ] {
        writeln!(file, "{}", capture_row(label, config, local)).unwrap();
        file.flush().unwrap();
    }
    std::thread::sleep(Duration::from_millis(1));
}
