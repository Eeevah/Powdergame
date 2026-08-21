//! G8-A Observational Profiling & Measurement Substrate Integration Tests.
//!
//! Verifies:
//! - A. Ordinary Simulation::tick remains available and does not require TIMESTAMP_QUERY.
//! - B. A profiled simulation tick executes the exact same production pass sequence.
//! - C. Matching profiled vs unprofiled simulations from identical state (Material, Flags, Temperature, Pressure exact).
//! - D. Timestamp results (all ordered positive production pass records and a valid envelope).
//! - E. Activity census works out-of-band and does not perturb world state.
//! - F. Tracked memory report accounts for all world, scratch, activity, uniform, and profiler allocations.

use pollster::block_on;
use powdergame_core::{WorldConfig, FLAG_COMBUSTING, MATERIAL_SAND, MATERIAL_WATER, MATERIAL_WOOD};
use powdergame_gpu::{ContextOptions, GpuContext, GpuProfiler, Simulation, PASS_COUNT, PASS_NAMES};

#[test]
fn test_ordinary_simulation_tick_does_not_require_profiling_feature() {
    let ctx = match block_on(GpuContext::new_with_options(ContextOptions {
        enable_profiling: false,
    })) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test (GPU unavailable): {e}");
            return;
        }
    };

    assert!(!ctx.profiling_enabled);

    let config = WorldConfig {
        width: 128,
        height: 128,
        chunk_size: 64,
    };
    let mut sim = Simulation::with_context(ctx, config).expect("failed to create Simulation");

    // Ordinary unprofiled tick runs cleanly without error
    sim.tick().expect("ordinary tick must succeed");
    assert_eq!(sim.tick_count, 1);
}

#[test]
fn test_profiled_simulation_tick_produces_all_valid_pass_timings() {
    let ctx = match block_on(GpuContext::with_profiling()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test (GPU profiling unavailable): {e}");
            return;
        }
    };

    assert!(ctx.profiling_enabled);
    assert!(ctx.timestamp_period > 0.0);

    let mut profiler = GpuProfiler::new(&ctx).expect("failed to create GpuProfiler");
    let config = WorldConfig {
        width: 128,
        height: 128,
        chunk_size: 64,
    };
    let mut sim = Simulation::with_context(ctx, config).expect("failed to create Simulation");

    // Stage a small active fixture to ensure passes do work
    sim.world
        .write_material(&sim.context.queue, 64, 60, MATERIAL_SAND)
        .unwrap();
    sim.world
        .write_material(&sim.context.queue, 64, 64, MATERIAL_WATER)
        .unwrap();
    sim.world
        .write_material(&sim.context.queue, 64, 68, MATERIAL_WOOD)
        .unwrap();
    sim.world
        .write_flags(&sim.context.queue, 64, 68, FLAG_COMBUSTING)
        .unwrap();
    sim.world
        .write_temperature(&sim.context.queue, 64, 68, 500.0)
        .unwrap();

    let first = sim
        .tick_profiled(&mut profiler)
        .expect("tick_profiled must succeed");
    let report = sim
        .tick_profiled(&mut profiler)
        .expect("repeated tick_profiled must succeed");

    assert_eq!(first.tick_index, 0);
    assert_eq!(report.tick_index, 1);
    assert_ne!(
        first.raw_timestamps, report.raw_timestamps,
        "each profiled tick must overwrite the reused query results"
    );
    assert!(
        report.raw_timestamps[0] >= first.raw_timestamps[PASS_COUNT * 2 - 1],
        "the second resolved query set must follow the first on the GPU timeline"
    );
    assert_eq!(report.passes.len(), PASS_COUNT);
    assert_eq!(PASS_COUNT, 40);

    for (i, pass) in report.passes.iter().enumerate() {
        assert_eq!(pass.name, PASS_NAMES[i]);
        assert_eq!(pass.raw_start, report.raw_timestamps[i * 2]);
        assert_eq!(pass.raw_end, report.raw_timestamps[i * 2 + 1]);
        assert!(
            pass.raw_end > pass.raw_start,
            "pass must have positive duration"
        );
        if i > 0 {
            assert!(pass.raw_start >= report.passes[i - 1].raw_end);
        }
        assert!(pass.duration_ms > 0.0, "pass duration must be positive");
        assert!(pass.duration_ms.is_finite(), "pass duration must be finite");
    }

    assert!(report.gpu_tick_envelope_ms > 0.0);
    assert!(report.gpu_pass_sum_ms > 0.0);
    assert!(report.residual_ms >= 0.0);
    assert!(report.residual_ms.is_finite());
    assert!(report.gpu_pass_sum_ms <= report.gpu_tick_envelope_ms);
    assert!(
        (report.gpu_pass_sum_ms + report.residual_ms - report.gpu_tick_envelope_ms).abs() < 1.0e-12
    );

    let grouped = report.grouped_summary();
    let grouped_sum = grouped.matter_movement_ms
        + grouped.ownership_claim_ms
        + grouped.thermal_ms
        + grouped.reaction_phase_ms
        + grouped.pressure_structure_ms
        + grouped.active_sleep_ms;
    assert!((grouped_sum - report.gpu_pass_sum_ms).abs() < 1.0e-12);
}

#[test]
fn test_profiled_vs_unprofiled_simulation_state_exact_equivalence() {
    let ctx_unprof = match block_on(GpuContext::new()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test (GPU unprofiled unavailable): {e}");
            return;
        }
    };
    let ctx_prof = match block_on(GpuContext::with_profiling()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test (GPU profiled unavailable): {e}");
            return;
        }
    };

    let config = WorldConfig {
        width: 128,
        height: 128,
        chunk_size: 64,
    };

    let mut sim_unprof =
        Simulation::with_context(ctx_unprof, config).expect("failed to create unprofiled sim");
    let mut sim_prof =
        Simulation::with_context(ctx_prof, config).expect("failed to create profiled sim");
    let mut profiler = GpuProfiler::new(&sim_prof.context).expect("failed to create profiler");

    // Stage identical rich active fixture in both simulations
    for y in 20..40 {
        for x in 20..40 {
            sim_unprof
                .world
                .write_material(&sim_unprof.context.queue, x, y, MATERIAL_SAND)
                .unwrap();
            sim_prof
                .world
                .write_material(&sim_prof.context.queue, x, y, MATERIAL_SAND)
                .unwrap();
        }
    }
    for y in 60..80 {
        for x in 60..80 {
            sim_unprof
                .world
                .write_material(&sim_unprof.context.queue, x, y, MATERIAL_WATER)
                .unwrap();
            sim_prof
                .world
                .write_material(&sim_prof.context.queue, x, y, MATERIAL_WATER)
                .unwrap();
        }
    }
    for x in 40..60 {
        sim_unprof
            .world
            .write_material(&sim_unprof.context.queue, x, 100, MATERIAL_WOOD)
            .unwrap();
        sim_prof
            .world
            .write_material(&sim_prof.context.queue, x, 100, MATERIAL_WOOD)
            .unwrap();
    }
    sim_unprof
        .world
        .write_flags(&sim_unprof.context.queue, 40, 100, FLAG_COMBUSTING)
        .unwrap();
    sim_prof
        .world
        .write_flags(&sim_prof.context.queue, 40, 100, FLAG_COMBUSTING)
        .unwrap();
    sim_unprof
        .world
        .write_temperature(&sim_unprof.context.queue, 40, 100, 500.0)
        .unwrap();
    sim_prof
        .world
        .write_temperature(&sim_prof.context.queue, 40, 100, 500.0)
        .unwrap();

    // Run 50 ticks: unprofiled via sim.tick(), profiled via sim.tick_profiled()
    for _ in 0..50 {
        sim_unprof.tick().unwrap();
        let _ = sim_prof.tick_profiled(&mut profiler).unwrap();
    }

    assert_eq!(sim_unprof.tick_count, sim_prof.tick_count);

    // Read back and assert byte-exact state equality
    let mat_unprof = sim_unprof
        .world
        .read_material_all(&sim_unprof.context.device, &sim_unprof.context.queue)
        .unwrap();
    let mat_prof = sim_prof
        .world
        .read_material_all(&sim_prof.context.device, &sim_prof.context.queue)
        .unwrap();
    assert_eq!(
        mat_unprof, mat_prof,
        "Material must match byte-exact between profiled and unprofiled"
    );

    let flags_unprof = sim_unprof
        .world
        .read_flags_all(&sim_unprof.context.device, &sim_unprof.context.queue)
        .unwrap();
    let flags_prof = sim_prof
        .world
        .read_flags_all(&sim_prof.context.device, &sim_prof.context.queue)
        .unwrap();
    assert_eq!(
        flags_unprof, flags_prof,
        "Flags must match byte-exact between profiled and unprofiled"
    );

    let temp_unprof = sim_unprof
        .world
        .read_temperature_all(&sim_unprof.context.device, &sim_unprof.context.queue)
        .unwrap();
    let temp_prof = sim_prof
        .world
        .read_temperature_all(&sim_prof.context.device, &sim_prof.context.queue)
        .unwrap();
    assert_eq!(
        temp_unprof, temp_prof,
        "Temperature must match exact between profiled and unprofiled"
    );

    let press_unprof = sim_unprof
        .world
        .read_pressure_all(&sim_unprof.context.device, &sim_unprof.context.queue)
        .unwrap();
    let press_prof = sim_prof
        .world
        .read_pressure_all(&sim_prof.context.device, &sim_prof.context.queue)
        .unwrap();
    assert_eq!(
        press_unprof, press_prof,
        "Pressure must match exact between profiled and unprofiled"
    );

    let selected_cells = [(1, 1), (20, 20), (40, 40), (64, 64), (100, 100)];
    let environment_unprof = sim_unprof
        .world
        .read_environment_cells(
            &sim_unprof.context.device,
            &sim_unprof.context.queue,
            &selected_cells,
        )
        .unwrap();
    let environment_prof = sim_prof
        .world
        .read_environment_cells(
            &sim_prof.context.device,
            &sim_prof.context.queue,
            &selected_cells,
        )
        .unwrap();
    assert_eq!(
        environment_unprof, environment_prof,
        "Environment must match exact between profiled and unprofiled"
    );
}

#[test]
fn test_activity_census_reports_accurate_cell_and_chunk_metrics() {
    let ctx = match block_on(GpuContext::new()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test (GPU unavailable): {e}");
            return;
        }
    };

    let config = WorldConfig {
        width: 128,
        height: 128,
        chunk_size: 64, // 2x2 = 4 chunks
    };
    let mut sim = Simulation::with_context(ctx, config).expect("failed to create sim");

    // In pristine empty world, initial tick produces 0 active cells
    sim.tick().unwrap();
    let census = sim.activity_census().expect("census must succeed");

    assert_eq!(census.total_cells, 128 * 128);
    assert_eq!(census.total_chunks, 4);
    assert_eq!(census.any_active_cells, 0);
    assert_eq!(census.active_chunks, 0);

    // Place a single falling Sand cell at (32, 32) in chunk (0, 0)
    sim.world
        .write_material(&sim.context.queue, 32, 32, MATERIAL_SAND)
        .unwrap();
    sim.tick().unwrap();

    let census_active = sim.activity_census().expect("census must succeed");
    assert!(
        census_active.any_active_cells > 0,
        "falling sand must register active cells"
    );
    assert!(
        census_active.matter_active_cells > 0,
        "falling sand must register matter active cells"
    );
    assert!(
        census_active.active_chunks > 0,
        "falling sand must activate at least chunk (0,0)"
    );
}

#[test]
fn test_tracked_gpu_allocation_report_structure() {
    let ctx = match block_on(GpuContext::with_profiling()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test (GPU unavailable): {e}");
            return;
        }
    };

    let profiler = GpuProfiler::new(&ctx).expect("failed to create profiler");
    let config = WorldConfig {
        width: 2048,
        height: 2048,
        chunk_size: 64,
    };
    let sim = Simulation::with_context(ctx, config).expect("failed to create sim");

    let unprofiled = sim.tracked_memory_report(None);
    assert_eq!(unprofiled.total_tracked_gpu_bytes, 302_016_816);
    let mem = sim.tracked_memory_report(Some(&profiler));

    // 2048x2048 = 4,194,304 cells
    // 10 dense world arrays include the TE-3 phase-energy Current/Next pair.
    assert_eq!(mem.world_dense_state_bytes, 4_194_304 * 4 * 10);

    // TE-1 persistent Air: mass/energy Current+Next = 4 f32 arrays.
    assert_eq!(mem.environment_state_bytes, 4_194_304 * 4 * 4);

    // 2 scratch arrays * 4 bytes/cell = 8 bytes/cell = 33,554,432 bytes (32 MB)
    assert_eq!(mem.movement_scratch_bytes, 4_194_304 * 4 * 2);

    // TE-1 reuses exactly one full-resolution receiver-claim scratch.
    assert_eq!(mem.environment_receiver_claim_bytes, 4_194_304 * 4);

    // Activity scratch: cell_activity (16 MB) + 6 chunk buffers (6 * 1024 * 4 = 24,576 bytes)
    assert_eq!(mem.activity_scratch_bytes, 4_194_304 * 4 + 1024 * 4 * 6);

    // Profiler: 80 timestamps * 8 bytes * resolve+readback = 1,280 bytes.
    assert_eq!(mem.profiler_bytes, 1_280);

    // Exact persistent inventory: TE-2 adds the Environment and wake uniforms
    // plus one combined thermal table without adding full-resolution scratch.
    // This assertion must fail if tracked_memory_report omits an allocation.
    assert_eq!(mem.uniforms_and_tables_bytes, 2_352);

    assert_eq!(mem.total_tracked_gpu_bytes, 302_018_096);

    assert_eq!(
        mem.total_tracked_gpu_bytes,
        mem.world_dense_state_bytes
            + mem.environment_state_bytes
            + mem.movement_scratch_bytes
            + mem.environment_receiver_claim_bytes
            + mem.activity_scratch_bytes
            + mem.uniforms_and_tables_bytes
            + mem.profiler_bytes
    );
}
