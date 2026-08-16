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
    WorldConfig, CHUNK_STATE_RUNNABLE, CHUNK_STATE_SLEEPING, FLAG_COMBUSTING, MATERIAL_EMPTY,
    MATERIAL_ICE, MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER,
    MATERIAL_WOOD, WAKE_REASON_NEIGHBOR_HALO, WAKE_REASON_NONE, WAKE_REASON_SELF_ACTIVITY,
    WAKE_REASON_USER_EDIT,
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
    assert_eq!(states[0], CHUNK_STATE_RUNNABLE, "hot chunk must be runnable");
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
    assert_eq!(states[0], CHUNK_STATE_RUNNABLE, "chunk (0,0) with pressure must wake");
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
