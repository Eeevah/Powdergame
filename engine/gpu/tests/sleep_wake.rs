//! G7-B — Actual Sleep / Wake Correctness test suite.
//!
//! Verifies on the authoritative GPU path (Windows + DX12 + discrete GPU) that:
//! 1. Stable chunks enter CHUNK_STATE_SLEEPING and skip work pass-by-pass.
//! 2. 8-neighbor cross-chunk activity halo wakes chunks before any interaction crosses boundaries.
//! 3. Diagonal seams and corners are guarded against false sleep.
//! 4. User edits wake the target chunk and all 8 neighbors immediately.
//! 5. Combustion, decay, thermal gradients, pressure gradients, and phase transitions never sleep while active.
//! 6. Scratch buffer hygiene is strictly maintained with exact state carry-forward.
//! 7. Sleep ON (Sparse Work) and Sleep OFF (Always Active reference) produce semantically equivalent results.

use powdergame_core::{
    fuel_progress, ignition_exposure, with_fuel_progress, with_ignition_exposure, WorldConfig,
    CHUNK_STATE_RUNNABLE, CHUNK_STATE_SLEEPING, COMBUSTION_WOOD_BURN_DURATION, FLAG_COMBUSTING,
    MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_SAND, MATERIAL_SMOKE,
    MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD, WAKE_REASON_NEIGHBOR_HALO,
    WAKE_REASON_NONE, WAKE_REASON_SELF_ACTIVITY, WAKE_REASON_USER_EDIT,
};
use powdergame_gpu::Simulation;

fn make_sim(config: WorldConfig) -> Simulation {
    pollster::block_on(Simulation::new(config)).expect("DX12 simulation init")
}

fn set_mat(sim: &Simulation, x: i64, y: i64, id: u32) {
    sim.world
        .write_material(&sim.context.queue, x, y, id)
        .expect("material edit");
}

fn set_temp(sim: &Simulation, x: i64, y: i64, t: f32) {
    sim.world
        .write_temperature(&sim.context.queue, x, y, t)
        .expect("temperature edit");
}

fn set_press(sim: &Simulation, x: i64, y: i64, p: f32) {
    sim.world
        .write_pressure(&sim.context.queue, x, y, p)
        .expect("pressure edit");
}

fn set_flag(sim: &Simulation, x: i64, y: i64, f: u32) {
    sim.world
        .write_flags(&sim.context.queue, x, y, f)
        .expect("flags edit");
}

fn fill_box(sim: &Simulation, x0: i64, y0: i64, x1: i64, y1: i64, id: u32) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            set_mat(sim, x, y, id);
        }
    }
}

fn fill_box_temp(sim: &Simulation, x0: i64, y0: i64, x1: i64, y1: i64, t: f32) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            set_temp(sim, x, y, t);
        }
    }
}

fn read_states(sim: &Simulation) -> Vec<u32> {
    sim.world
        .read_chunk_state_all(&sim.context.device, &sim.context.queue)
        .expect("read chunk states")
}

fn read_reasons(sim: &Simulation) -> Vec<u32> {
    sim.world
        .read_chunk_wake_reason_all(&sim.context.device, &sim.context.queue)
        .expect("read chunk wake reasons")
}

fn read_mats(sim: &Simulation) -> Vec<u32> {
    sim.world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("read all materials")
}

fn read_temps(sim: &Simulation) -> Vec<f32> {
    sim.world
        .read_temperature_all(&sim.context.device, &sim.context.queue)
        .expect("read all temperatures")
}

fn read_pressures(sim: &Simulation) -> Vec<f32> {
    sim.world
        .read_pressure_all(&sim.context.device, &sim.context.queue)
        .expect("read all pressures")
}

fn read_flags_vec(sim: &Simulation) -> Vec<u32> {
    sim.world
        .read_flags_all(&sim.context.device, &sim.context.queue)
        .expect("read all flags")
}

fn read_edit_wakes(sim: &Simulation) -> Vec<u32> {
    sim.world
        .read_chunk_edit_wake_all(&sim.context.device, &sim.context.queue)
        .expect("read all edit wakes")
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario A: Stable Water Bulk Sleep
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_scenario_a_stable_water_bulk_sleep() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 32).unwrap());
    sim.set_sleep_threshold(4);

    // Uniform water basin enclosed in stone in a 2x2 chunk world (chunk_size = 32)
    fill_box(&sim, 0, 0, 63, 63, MATERIAL_STONE);
    fill_box(&sim, 1, 1, 62, 31, MATERIAL_EMPTY);
    fill_box(&sim, 1, 32, 62, 59, MATERIAL_WATER);

    // Run ticks to let water settle and stable duration reach threshold
    for _ in 0..12 {
        sim.tick().expect("tick");
    }

    let states = read_states(&sim);
    let reasons = read_reasons(&sim);

    // Both bottom chunks (containing settled water and floor) should be SLEEPING
    // Top chunks (empty air enclosed in stone) should also be SLEEPING
    for (i, &state) in states.iter().enumerate() {
        assert_eq!(
            state, CHUNK_STATE_SLEEPING,
            "chunk {i} should be SLEEPING once settled"
        );
        assert_eq!(
            reasons[i], WAKE_REASON_NONE,
            "chunk {i} should have WAKE_REASON_NONE"
        );
    }

    // Material count must be strictly preserved
    let mats = read_mats(&sim);
    let water_count = mats.iter().filter(|&&m| m == MATERIAL_WATER).count();
    assert_eq!(water_count, 62 * 28, "water count preserved under sleep");
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario B: Stable Steam Bulk Sleep
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_scenario_b_stable_steam_bulk_sleep() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 32).unwrap());
    sim.set_sleep_threshold(4);

    // Confined Steam basin with uniform temperature (150°C) across steam and container
    fill_box(&sim, 0, 0, 63, 63, MATERIAL_STONE);
    fill_box_temp(&sim, 0, 0, 63, 63, 150.0);
    fill_box(&sim, 1, 1, 62, 62, MATERIAL_STEAM);
    fill_box_temp(&sim, 1, 1, 62, 62, 150.0);

    for _ in 0..10 {
        sim.tick().expect("tick");
    }

    let states = read_states(&sim);
    for (i, &state) in states.iter().enumerate() {
        assert_eq!(
            state, CHUNK_STATE_SLEEPING,
            "chunk {i} containing uniform steam should sleep"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario C: Falling Sand Wake Before Impact
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_scenario_c_falling_sand_wakes_before_impact() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 32).unwrap());
    sim.set_sleep_threshold(3);

    // Bottom chunk (y: 32..63) is sleeping stone floor
    fill_box(&sim, 0, 32, 63, 63, MATERIAL_STONE);

    // Settle bottom chunk to sleep
    for _ in 0..6 {
        sim.tick().expect("tick");
    }

    let states_before = read_states(&sim);
    assert_eq!(states_before[2], CHUNK_STATE_SLEEPING); // bottom-left chunk (0, 1)

    // Spawn falling sand at top (x: 16, y: 10) in chunk (0, 0)
    set_mat(&sim, 16, 10, MATERIAL_SAND);

    // Next tick: chunk (0, 0) has active falling sand.
    // The 8-neighbor safety halo MUST wake bottom chunk (0, 1) BEFORE sand crosses seam y=32.
    sim.tick().expect("tick with sand");

    let states_after = read_states(&sim);
    let reasons_after = read_reasons(&sim);

    assert_eq!(
        states_after[0], CHUNK_STATE_RUNNABLE,
        "chunk (0,0) with sand must be runnable"
    );
    assert_eq!(
        states_after[2], CHUNK_STATE_RUNNABLE,
        "chunk (0,1) must wake via safety halo before impact"
    );
    assert!(
        (reasons_after[2] & WAKE_REASON_NEIGHBOR_HALO) != 0,
        "wake reason for neighbor must include NEIGHBOR_HALO"
    );

    // Let sand fall and settle on the floor
    for _ in 0..40 {
        sim.tick().expect("tick");
    }

    let mats = read_mats(&sim);
    // Sand must have rested at (16, 31) right on top of stone at y=32
    assert_eq!(mats[31 * 64 + 16], MATERIAL_SAND);
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario D: Cross-Chunk Thermal Conduction Wake
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_scenario_d_cross_chunk_thermal_conduction_wake() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 32).unwrap());
    sim.set_sleep_threshold(3);

    // Stone wall everywhere at uniform 20°C
    fill_box(&sim, 0, 0, 63, 63, MATERIAL_STONE);
    fill_box_temp(&sim, 0, 0, 63, 63, 20.0);

    // Settle to sleep
    for _ in 0..6 {
        sim.tick().expect("tick");
    }

    for state in read_states(&sim) {
        assert_eq!(state, CHUNK_STATE_SLEEPING);
    }

    // Heat up chunk (0, 0) at x=31 (right against the seam to chunk 1, 0)
    set_temp(&sim, 31, 16, 500.0);

    // Next tick: chunk (0, 0) is active (thermal gradient + user edit).
    // Neighbor chunk (1, 0) must wake via halo and conduct heat.
    sim.tick().expect("tick");

    let states = read_states(&sim);
    assert_eq!(
        states[0], CHUNK_STATE_RUNNABLE,
        "hot chunk must be runnable"
    );
    assert_eq!(
        states[1], CHUNK_STATE_RUNNABLE,
        "adjacent cold chunk must wake via halo"
    );

    // Run more ticks to observe conduction across seam (x=31 -> x=32)
    for _ in 0..10 {
        sim.tick().expect("tick");
    }

    let temps = read_temps(&sim);
    let temp_at_32 = temps[16 * 64 + 32];
    assert!(
        temp_at_32 > 25.0,
        "heat must conduct across chunk seam into adjacent chunk (got {temp_at_32}°C)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario E: Cross-Chunk Pressure Wake
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_scenario_e_cross_chunk_pressure_wake() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 32).unwrap());
    sim.set_sleep_threshold(3);

    // Water enclosed in container
    fill_box(&sim, 0, 0, 63, 63, MATERIAL_STONE);
    fill_box(&sim, 1, 1, 62, 62, MATERIAL_WATER);

    // Settle to sleep
    for _ in 0..8 {
        sim.tick().expect("tick");
    }

    for state in read_states(&sim) {
        assert_eq!(state, CHUNK_STATE_SLEEPING);
    }

    // Inject high pressure impulse into chunk (0, 0) at (30, 16) right next to seam x=31
    set_press(&sim, 30, 16, 50.0);

    sim.tick().expect("tick");

    let states = read_states(&sim);
    assert_eq!(
        states[0], CHUNK_STATE_RUNNABLE,
        "chunk (0,0) with pressure must wake"
    );
    assert_eq!(
        states[1], CHUNK_STATE_RUNNABLE,
        "neighbor chunk (1,0) must wake via halo"
    );

    // Propagate pressure across ticks
    for _ in 0..20 {
        sim.tick().expect("tick");
    }

    let pressures = read_pressures(&sim);
    let p_across = pressures[16 * 64 + 33];
    assert!(
        p_across > 0.0,
        "pressure must propagate across chunk seam to x=33"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario F: Diagonal Seam Movement Across Chunk Corner
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_scenario_f_diagonal_seam_movement_wake() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 32).unwrap());
    sim.set_sleep_threshold(3);

    // Entire world empty, settle to sleep
    for _ in 0..6 {
        sim.tick().expect("tick");
    }

    for state in read_states(&sim) {
        assert_eq!(state, CHUNK_STATE_SLEEPING);
    }

    // Place falling sand diagonally right above the corner (31, 30)
    // Seam intersection is at (31.5, 31.5):
    // Chunk (0,0): top-left [0..31, 0..31]
    // Chunk (1,1): bottom-right [32..63, 32..63]
    set_mat(&sim, 31, 30, MATERIAL_SAND);

    // Solid obstacle below at (31, 31) and left at (30, 31) to force diagonal slide down-right into (32, 31) then (32, 32)
    set_mat(&sim, 31, 31, MATERIAL_STONE);
    set_mat(&sim, 30, 31, MATERIAL_STONE);

    sim.tick().expect("tick 1");

    let states = read_states(&sim);
    let reasons = read_reasons(&sim);

    // Chunk (1, 1) is the diagonal neighbor of (0, 0).
    // The 8-neighbor halo MUST include diagonal neighbors!
    assert_eq!(
        states[3], CHUNK_STATE_RUNNABLE,
        "diagonal chunk (1,1) must wake via 8-neighbor safety halo"
    );
    assert!(
        (reasons[3] & WAKE_REASON_NEIGHBOR_HALO) != 0,
        "wake reason must be NEIGHBOR_HALO"
    );

    // Let sand slide diagonally across corner into chunk (1, 1)
    for _ in 0..5 {
        sim.tick().expect("tick");
    }

    let mats = read_mats(&sim);
    let mut sand_pos = None;
    for (i, &m) in mats.iter().enumerate() {
        if m == MATERIAL_SAND {
            sand_pos = Some((i % 64, i / 64));
        }
    }
    let (sx, sy) = sand_pos.expect("sand should exist in world");
    assert!(
        sx >= 32 || sy >= 32,
        "sand at ({sx}, {sy}) must cross into chunk (1,1) or adjacent chunk across seam"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario G: User Edit Wake (Immediate 9-Chunk Wake)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_scenario_g_user_edit_wake() {
    let mut sim = make_sim(WorldConfig::new(96, 96, 32).unwrap()); // 3x3 chunks
    sim.set_sleep_threshold(3);

    // Settle entire 3x3 grid to sleep
    for _ in 0..6 {
        sim.tick().expect("tick");
    }

    for state in read_states(&sim) {
        assert_eq!(state, CHUNK_STATE_SLEEPING);
    }

    // User edits center cell (48, 48) in center chunk (1, 1) [index 4]
    set_mat(&sim, 48, 48, MATERIAL_SAND);

    // Execute one tick: the wake pass must wake center chunk (USER_EDIT) and ALL 8 neighbors (NEIGHBOR_HALO)
    sim.tick().expect("tick after edit");

    let states = read_states(&sim);
    let reasons = read_reasons(&sim);

    assert_eq!(
        states[4], CHUNK_STATE_RUNNABLE,
        "edited center chunk must be runnable"
    );
    assert!(
        (reasons[4] & (WAKE_REASON_USER_EDIT | WAKE_REASON_SELF_ACTIVITY)) != 0,
        "center chunk wake reason must include USER_EDIT"
    );

    for (i, &state) in states.iter().enumerate() {
        assert_eq!(
            state, CHUNK_STATE_RUNNABLE,
            "chunk {i} (including all 8 neighbors) must wake immediately on edit"
        );
        if i != 4 {
            assert!(
                (reasons[i] & WAKE_REASON_NEIGHBOR_HALO) != 0,
                "neighbor chunk {i} wake reason must include NEIGHBOR_HALO"
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario H: Combustion & Decay Never Sleep While Active
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_scenario_h_combustion_and_decay_never_sleep_while_active() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 32).unwrap());
    sim.set_sleep_threshold(2);

    // Place burning wood in chunk (0, 0)
    set_mat(&sim, 16, 16, MATERIAL_WOOD);
    set_temp(&sim, 16, 16, 400.0);
    set_flag(&sim, 16, 16, FLAG_COMBUSTING);

    // Place decaying smoke in chunk (1, 0)
    set_mat(&sim, 48, 16, MATERIAL_SMOKE);
    set_flag(&sim, 48, 16, 10); // lifetime ticks remaining

    // Tick repeatedly while burning/decaying: chunks must STAY RUNNABLE
    for tick_idx in 0..15 {
        sim.tick().expect("tick");
        let states = read_states(&sim);

        assert_eq!(
            states[0], CHUNK_STATE_RUNNABLE,
            "burning chunk must not sleep on tick {tick_idx}"
        );
        if tick_idx < 10 {
            assert_eq!(
                states[1], CHUNK_STATE_RUNNABLE,
                "decaying smoke chunk must not sleep on tick {tick_idx}"
            );
        }
    }
}

#[test]
fn test_ignition_exposure_decay_runs_to_zero_before_sleep() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 32).unwrap());
    sim.set_sleep_threshold(2);
    fill_box(&sim, 0, 0, 63, 63, MATERIAL_STONE);
    fill_box_temp(&sim, 0, 0, 63, 63, 20.0);
    set_mat(&sim, 16, 16, MATERIAL_WOOD);
    set_flag(&sim, 16, 16, with_ignition_exposure(0, 3));
    let target = 16 * 64 + 16;

    for expected in [2, 1] {
        sim.tick().expect("exposure decay tick");
        assert_eq!(ignition_exposure(read_flags_vec(&sim)[target]), expected);
        assert_eq!(read_states(&sim)[0], CHUNK_STATE_RUNNABLE);
    }
    sim.tick().expect("exposure reaches zero");
    assert_eq!(ignition_exposure(read_flags_vec(&sim)[target]), 0);
    for _ in 0..3 {
        sim.tick().expect("stable zero");
    }
    assert_eq!(
        read_states(&sim)[0],
        CHUNK_STATE_SLEEPING,
        "zero-dose equilibrium may sleep"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario I: Phase Transitions Wake
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_scenario_i_phase_transitions_wake() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 32).unwrap());
    sim.set_sleep_threshold(3);

    // Enclosed Ice block at uniform -5°C (container + ice)
    fill_box(&sim, 0, 0, 63, 63, MATERIAL_STONE);
    fill_box_temp(&sim, 0, 0, 63, 63, -5.0);
    fill_box(&sim, 1, 1, 62, 62, MATERIAL_ICE);
    fill_box_temp(&sim, 1, 1, 62, 62, -5.0);

    // Settle to sleep
    for _ in 0..6 {
        sim.tick().expect("tick");
    }

    for state in read_states(&sim) {
        assert_eq!(state, CHUNK_STATE_SLEEPING);
    }

    // Heat up ice to 50°C (above melting point 0°C)
    fill_box_temp(&sim, 1, 1, 62, 62, 50.0);

    // Next tick: ice melting phase transition MUST trigger and wake chunk
    sim.tick().expect("melt tick");

    let states = read_states(&sim);
    for state in states {
        assert_eq!(state, CHUNK_STATE_RUNNABLE, "melting ice chunk must wake");
    }

    let mats = read_mats(&sim);
    let water_count = mats.iter().filter(|&&m| m == MATERIAL_WATER).count();
    assert!(water_count > 0, "ice must melt into water upon waking");
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario J: Scratch Hygiene & Carry-Forward Verification
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_scenario_j_scratch_hygiene_and_carry_forward() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 32).unwrap());
    sim.set_sleep_threshold(2);

    // Complex stable multi-material pattern in uniform thermal equilibrium:
    // Left half: Stone wall at 20°C
    // Right half: Stone wall at 20°C with distinct flags
    fill_box(&sim, 0, 0, 31, 63, MATERIAL_STONE);
    fill_box_temp(&sim, 0, 0, 31, 63, 20.0);
    fill_box(&sim, 32, 0, 63, 63, MATERIAL_STONE);
    fill_box_temp(&sim, 32, 0, 63, 63, 20.0);

    // Settle to sleep
    for _ in 0..10 {
        sim.tick().expect("tick");
    }

    let mats_baseline = read_mats(&sim);
    let temps_baseline = read_temps(&sim);
    let press_baseline = read_pressures(&sim);
    let flags_baseline = read_flags_vec(&sim);

    // Run 50 ticks while fully sleeping
    for _ in 0..50 {
        sim.tick().expect("tick");
    }

    let mats_after = read_mats(&sim);
    let temps_after = read_temps(&sim);
    let press_after = read_pressures(&sim);
    let flags_after = read_flags_vec(&sim);

    // Exact byte-for-byte carry-forward integrity check
    assert_eq!(
        mats_baseline, mats_after,
        "material buffer must carry forward perfectly during sleep"
    );
    assert_eq!(
        temps_baseline, temps_after,
        "temperature buffer must carry forward perfectly during sleep"
    );
    assert_eq!(
        press_baseline, press_after,
        "pressure buffer must carry forward perfectly during sleep"
    );
    assert_eq!(
        flags_baseline, flags_after,
        "flags buffer must carry forward perfectly during sleep"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario K: Mixed Long-Run Equivalence (Sleep ON vs Sleep OFF)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_scenario_k_mixed_long_run_equivalence() {
    let config = WorldConfig::new(64, 64, 32).unwrap();

    let mut sim_sleep = make_sim(config);
    let mut sim_nosleep = make_sim(config);

    sim_sleep.set_sleep_enabled(true);
    sim_sleep.set_sleep_threshold(4);

    sim_nosleep.set_sleep_enabled(false); // Always Active reference

    let setup = |sim: &Simulation| {
        // Enclosed arena
        fill_box(sim, 0, 0, 63, 63, MATERIAL_EMPTY);
        fill_box(sim, 0, 60, 63, 63, MATERIAL_STONE);
        fill_box(sim, 0, 0, 3, 63, MATERIAL_STONE);
        fill_box(sim, 60, 0, 63, 63, MATERIAL_STONE);

        // Water basin on bottom left
        fill_box(sim, 4, 45, 30, 59, MATERIAL_WATER);
        fill_box_temp(sim, 4, 45, 30, 59, 20.0);

        // Sand heap falling from top right
        fill_box(sim, 35, 10, 45, 20, MATERIAL_SAND);

        // Stone divider with heat source
        fill_box(sim, 31, 35, 33, 59, MATERIAL_STONE);
        fill_box_temp(sim, 31, 35, 33, 59, 80.0);
    };

    setup(&sim_sleep);
    setup(&sim_nosleep);

    // Run both simulations for 60 ticks
    for _ in 0..60 {
        sim_sleep.tick().expect("sleep tick");
        sim_nosleep.tick().expect("no-sleep tick");
    }

    let mats_sleep = read_mats(&sim_sleep);
    let mats_nosleep = read_mats(&sim_nosleep);

    let temps_sleep = read_temps(&sim_sleep);
    let temps_nosleep = read_temps(&sim_nosleep);

    let press_sleep = read_pressures(&sim_sleep);
    let press_nosleep = read_pressures(&sim_nosleep);

    let flags_sleep = read_flags_vec(&sim_sleep);
    let flags_nosleep = read_flags_vec(&sim_nosleep);

    // Semantic Equivalence: Material placement must match exactly
    assert_eq!(
        mats_sleep, mats_nosleep,
        "Sleep ON and Sleep OFF must produce identical material outcomes"
    );

    // Flags must match exactly
    assert_eq!(
        flags_sleep, flags_nosleep,
        "Sleep ON and Sleep OFF must produce identical flags outcomes"
    );

    // Temperatures must match within floating point precision
    for (i, (&t_s, &t_ns)) in temps_sleep.iter().zip(temps_nosleep.iter()).enumerate() {
        assert!(
            (t_s - t_ns).abs() < 1e-3,
            "temperature mismatch at cell {i}: sleep={t_s}, nosleep={t_ns}"
        );
    }

    // Pressures must match within floating point precision
    for (i, (&p_s, &p_ns)) in press_sleep.iter().zip(press_nosleep.iter()).enumerate() {
        assert!(
            (p_s - p_ns).abs() < 1e-3,
            "pressure mismatch at cell {i}: sleep={p_s}, nosleep={p_ns}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5.1: Extinguished Wood Exact Fuel Progress Sleep Freeze
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_extinguished_wood_fuel_progress_survives_sleep_exact() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 32).unwrap());
    sim.set_sleep_threshold(2);

    // Enclose world with stone to keep it completely stable
    fill_box(&sim, 0, 0, 63, 63, MATERIAL_STONE);
    fill_box_temp(&sim, 0, 0, 63, 63, 20.0);

    // Interior Wood cell at (16, 16) with fuel progress = 100, COMBUSTING = OFF
    let target_flags = with_fuel_progress(0, 100);
    set_mat(&sim, 16, 16, MATERIAL_WOOD);
    set_temp(&sim, 16, 16, 20.0);
    set_flag(&sim, 16, 16, target_flags);

    let w = 64;
    let target_idx = 16 * w + 16;

    // Run production ticks until sleeping
    let mut reached_sleep = false;
    for tick in 1..=20 {
        sim.tick().expect("production tick");

        let mats = read_mats(&sim);
        let flags_vec = read_flags_vec(&sim);
        let states = read_states(&sim);

        assert_eq!(
            mats[target_idx], MATERIAL_WOOD,
            "target cell must remain WOOD at tick {tick}"
        );
        let current_p = fuel_progress(flags_vec[target_idx]);
        assert_eq!(
            current_p, 100,
            "fuel progress must be preserved exact=100 at tick {tick}, got {current_p}"
        );

        // Chunk containing (16, 16) is chunk index 0 (cx=0, cy=0 in 64x64/32)
        if states[0] == CHUNK_STATE_SLEEPING {
            assert_eq!(
                flags_vec[target_idx], target_flags,
                "full flags must match target_flags exact on sleep transition"
            );
            reached_sleep = true;
            break;
        }
    }

    assert!(reached_sleep, "chunk 0 must enter CHUNK_STATE_SLEEPING");
}

// ─────────────────────────────────────────────────────────────────────────────
// 5.2: Scenario K Combustion Lifecycle Equivalence
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_scenario_k_combustion_lifecycle_equivalence() {
    let config = WorldConfig::new(64, 64, 32).unwrap();
    let mut sim_sleep = make_sim(config);
    let mut sim_ref = make_sim(config);

    sim_sleep.set_sleep_enabled(true);
    sim_sleep.set_sleep_threshold(2);
    sim_ref.set_sleep_enabled(false);

    let setup = |sim: &Simulation| {
        // Enclose in stone
        fill_box(sim, 0, 0, 63, 63, MATERIAL_STONE);
        fill_box_temp(sim, 0, 0, 63, 63, 20.0);

        // Target Wood at (16, 16) with initial progress = 100, COMBUSTING = OFF
        let target_flags = with_fuel_progress(0, 100);
        set_mat(sim, 16, 16, MATERIAL_WOOD);
        set_temp(sim, 16, 16, 20.0);
        set_flag(sim, 16, 16, target_flags);

        // Smoke spawn blockers around (16, 16)
        for (dx, dy) in [(0, -1), (-1, -1), (1, -1), (-1, 0), (1, 0)] {
            set_mat(sim, 16 + dx, 16 + dy, MATERIAL_BOUNDARY_BLOCK);
            set_temp(sim, 16 + dx, 16 + dy, 20.0);
        }
        // Down is an orthogonal positive-Air face but not a Smoke target.
        set_mat(sim, 16, 17, MATERIAL_EMPTY);
    };

    setup(&sim_sleep);
    setup(&sim_ref);

    let w = 64;
    let target_idx = 16 * w + 16;

    // Run until sleep
    let mut slept = false;
    for _ in 0..10 {
        sim_sleep.tick().expect("sleep sim tick");
        sim_ref.tick().expect("ref sim tick");

        let states_sleep = read_states(&sim_sleep);
        if states_sleep[0] == CHUNK_STATE_SLEEPING {
            slept = true;
            break;
        }
    }
    assert!(slept, "sim_sleep chunk 0 must have entered sleep");

    let flags_s = read_flags_vec(&sim_sleep)[target_idx];
    let flags_r = read_flags_vec(&sim_ref)[target_idx];
    assert_eq!(
        fuel_progress(flags_s),
        100,
        "sleep sim fuel progress must be 100"
    );
    assert_eq!(
        fuel_progress(flags_r),
        100,
        "ref sim fuel progress must be 100"
    );
    assert_eq!(read_mats(&sim_sleep)[target_idx], MATERIAL_WOOD);
    assert_eq!(read_mats(&sim_ref)[target_idx], MATERIAL_WOOD);

    // Re-ignite both at the same tick boundary
    // Keep this lifecycle fixture well clear of the 250 C sustain boundary;
    // TE-2 Air exchange must not turn an exact burn-duration test into a
    // threshold-crossing timing test.
    let ignition_t = 1_000.0;
    set_temp(&sim_sleep, 16, 16, ignition_t);
    set_temp(&sim_ref, 16, 16, ignition_t);
    for (dx, dy) in [(0, -1), (-1, -1), (1, -1), (-1, 0), (1, 0), (0, 1)] {
        set_temp(&sim_sleep, 16 + dx, 16 + dy, ignition_t);
        set_temp(&sim_ref, 16 + dx, 16 + dy, ignition_t);
    }
    let resume_flags = with_ignition_exposure(with_fuel_progress(0, 100), 55);
    set_flag(&sim_sleep, 16, 16, resume_flags);
    set_flag(&sim_ref, 16, 16, resume_flags);

    let remaining = (COMBUSTION_WOOD_BURN_DURATION - 100) as usize;

    // Active burn ticks
    for burn_step in 1..remaining {
        sim_sleep.tick().expect("burn tick sleep");
        sim_ref.tick().expect("burn tick ref");

        let m_s = read_mats(&sim_sleep)[target_idx];
        let m_r = read_mats(&sim_ref)[target_idx];
        let f_s = read_flags_vec(&sim_sleep)[target_idx];
        let f_r = read_flags_vec(&sim_ref)[target_idx];

        assert_eq!(
            m_s, MATERIAL_WOOD,
            "sleep target must still be WOOD at burn step {burn_step}"
        );
        assert_eq!(
            m_r, MATERIAL_WOOD,
            "ref target must still be WOOD at burn step {burn_step}"
        );
        assert_eq!(f_s, f_r, "flags mismatch at burn step {burn_step}");
        assert_eq!(
            fuel_progress(f_s),
            (100 + burn_step) as u32,
            "fuel progress mismatch at step {burn_step}"
        );
    }

    // Final remaining-th tick consumes the last fuel
    sim_sleep.tick().expect("final burn tick sleep");
    sim_ref.tick().expect("final burn tick ref");

    let final_m_s = read_mats(&sim_sleep)[target_idx];
    let final_m_r = read_mats(&sim_ref)[target_idx];
    let final_f_s = read_flags_vec(&sim_sleep)[target_idx];
    let final_f_r = read_flags_vec(&sim_ref)[target_idx];

    assert_eq!(
        final_m_s, MATERIAL_EMPTY,
        "sleep target must become EMPTY on final burn tick"
    );
    assert_eq!(
        final_m_r, MATERIAL_EMPTY,
        "ref target must become EMPTY on final burn tick"
    );
    assert_eq!(
        final_f_s, 0,
        "sleep target flags must be 0 after burning to empty"
    );
    assert_eq!(
        final_f_r, 0,
        "ref target flags must be 0 after burning to empty"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 5.3: Edge Edit Immutable Snapshot
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_edit_wake_edge_snapshot_wakes_expected_halo() {
    let mut sim = make_sim(WorldConfig::new(96, 96, 32).unwrap());
    sim.set_sleep_threshold(2);

    // Fill completely with stone at 20.0
    fill_box(&sim, 0, 0, 95, 95, MATERIAL_STONE);
    fill_box_temp(&sim, 0, 0, 95, 95, 20.0);

    // Tick until all 9 chunks are sleeping
    for _ in 0..10 {
        sim.tick().expect("settling tick");
    }
    let states_before = read_states(&sim);
    assert_eq!(states_before.len(), 9);
    assert!(
        states_before.iter().all(|&s| s == CHUNK_STATE_SLEEPING),
        "all 9 chunks must be sleeping before edit"
    );

    // Edit (31, 48) - in chunk 3 = (cx=0, cy=1), adjacent to vertical seam at x=32
    set_mat(&sim, 31, 48, MATERIAL_STONE);

    // Execute exactly 1 tick
    sim.tick().expect("edit wake tick");

    let states_after = read_states(&sim);
    let reasons = read_reasons(&sim);

    // Expected RUNNABLE: 0, 1, 3, 4, 6, 7
    let expected_runnable = [0, 1, 3, 4, 6, 7];
    for &idx in &expected_runnable {
        assert_eq!(
            states_after[idx], CHUNK_STATE_RUNNABLE,
            "chunk {idx} expected RUNNABLE"
        );
    }

    // Expected SLEEPING: 2, 5, 8
    let expected_sleeping = [2, 5, 8];
    for &idx in &expected_sleeping {
        assert_eq!(
            states_after[idx], CHUNK_STATE_SLEEPING,
            "chunk {idx} expected SLEEPING"
        );
    }

    // Chunk 3 was edited -> WAKE_REASON_USER_EDIT bit must be set
    assert!(
        (reasons[3] & WAKE_REASON_USER_EDIT) != 0,
        "chunk 3 must have USER_EDIT wake reason"
    );

    // Neighbor chunks 0, 1, 4, 6, 7 must have NEIGHBOR_HALO
    for &idx in &[0, 1, 4, 6, 7] {
        assert!(
            (reasons[idx] & WAKE_REASON_NEIGHBOR_HALO) != 0,
            "chunk {idx} must have NEIGHBOR_HALO wake reason"
        );
    }

    // After tick, chunk_edit_wake must be completely consumed (all 0)
    let edit_wakes = read_edit_wakes(&sim);
    assert!(
        edit_wakes.iter().all(|&w| w == 0),
        "all chunk_edit_wake flags must be cleared after tick"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 5.4: Diagonal Corner Edit Immutable Snapshot
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_edit_wake_diagonal_corner_snapshot_wakes_expected_halo() {
    let mut sim = make_sim(WorldConfig::new(96, 96, 32).unwrap());
    sim.set_sleep_threshold(2);

    fill_box(&sim, 0, 0, 95, 95, MATERIAL_STONE);
    fill_box_temp(&sim, 0, 0, 95, 95, 20.0);

    for _ in 0..10 {
        sim.tick().expect("settling tick");
    }
    let states_before = read_states(&sim);
    assert!(states_before.iter().all(|&s| s == CHUNK_STATE_SLEEPING));

    // Edit (31, 31) - corner cell of chunk 0 = (0,0), diagonally adjacent to chunk 4 = (1,1)
    set_mat(&sim, 31, 31, MATERIAL_STONE);

    sim.tick().expect("diagonal edit wake tick");

    let states_after = read_states(&sim);
    let reasons = read_reasons(&sim);

    // Expected RUNNABLE: 0, 1, 3, 4
    for &idx in &[0, 1, 3, 4] {
        assert_eq!(
            states_after[idx], CHUNK_STATE_RUNNABLE,
            "chunk {idx} expected RUNNABLE"
        );
    }

    // Expected SLEEPING: 2, 5, 6, 7, 8
    for &idx in &[2, 5, 6, 7, 8] {
        assert_eq!(
            states_after[idx], CHUNK_STATE_SLEEPING,
            "chunk {idx} expected SLEEPING"
        );
    }

    // Diagonal chunk 4 must have NEIGHBOR_HALO
    assert!(
        (reasons[4] & WAKE_REASON_NEIGHBOR_HALO) != 0,
        "diagonal chunk 4 must wake with NEIGHBOR_HALO"
    );

    let edit_wakes = read_edit_wakes(&sim);
    assert!(
        edit_wakes.iter().all(|&w| w == 0),
        "all chunk_edit_wake flags must be cleared after tick"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 5.5: Structural Race Regression Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_structural_wake_shader_and_simulation_race_contracts() {
    let wake_wgsl = include_str!("../src/activity_wake.wgsl");

    // 1. chunk_edit_wake is read-only in shader
    assert!(
        wake_wgsl.contains("@group(0) @binding(3) var<storage, read> chunk_edit_wake: array<u32>;"),
        "activity_wake.wgsl must bind chunk_edit_wake as read-only"
    );

    // 2. shader contains zero writes to chunk_edit_wake
    assert!(
        !wake_wgsl.contains("chunk_edit_wake[chunk_idx] =")
            && !wake_wgsl.contains("chunk_edit_wake[n_idx] ="),
        "activity_wake.wgsl must not contain writes to chunk_edit_wake"
    );

    let sim_rs = include_str!("../src/simulation.rs");

    // 3. simulation.rs layout binds chunk_edit_wake as Read
    assert!(
        sim_rs.contains(
            "buffer_entry(3, &BindingKind::Read), // chunk_edit_wake immutable wake snapshot"
        ),
        "simulation.rs must bind chunk_edit_wake with BindingKind::Read"
    );

    // 4. tick() ordering: wake pass < clear_buffer < propose pass
    let wake_pos = sim_rs
        .find("powdergame-g7b-activity-wake-pass")
        .expect("wake pass marker");
    let clear_pos = sim_rs
        .find("encoder.clear_buffer(&self.world.chunk_edit_wake, 0, None)")
        .expect("clear_buffer call");
    let propose_pos = sim_rs
        .find("powdergame-g3-propose-pass")
        .expect("propose pass marker");

    assert!(
        wake_pos < clear_pos,
        "wake pass must precede clear_buffer in simulation.rs"
    );
    assert!(
        clear_pos < propose_pos,
        "clear_buffer must precede propose pass in simulation.rs"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 5.6: Decay Sleep Freeze Structural Regression Test
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_structural_decay_sleeping_guard_flags_freeze() {
    let decay_wgsl = include_str!("../src/decay.wgsl");
    let norm_decay = decay_wgsl.replace("\r\n", "\n");

    // Check sleeping guard exists and carries flags exact
    assert!(
        norm_decay.contains("flags_next[index] = flags;\n            return;"),
        "decay.wgsl sleeping guard must carry flags exact"
    );

    // Ensure ~FLAG_DECAY_AGE_MASK is not inside sleeping guard
    let sleep_idx = norm_decay
        .find("if (params.sleep_enabled != 0u)")
        .expect("sleep guard");
    let return_idx = norm_decay[sleep_idx..]
        .find("return;")
        .expect("return in sleep guard");
    let sleep_block = &norm_decay[sleep_idx..sleep_idx + return_idx];
    assert!(
        !sleep_block.contains("~FLAG_DECAY_AGE_MASK"),
        "decay.wgsl sleeping guard must not clear decay age mask"
    );
}
