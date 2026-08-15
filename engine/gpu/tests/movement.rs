//! G2 — Local Movement: GPU semantic/invariant tests.
//!
//! Runs on the actual machine (Windows + RTX 5090 + DX12). Movement is
//! verified through small repeated scenarios on the authoritative GPU world:
//! read Current (diagnostic readback of a few cells), edit via the validated
//! staging hook, tick, read back. No exact pixel checksums are required
//! (DETERMINISM_SPEC §7); the semantic invariants are what matter.
//!
//! G2 scope: local 1-cell stencils only, EMPTY destinations only, no
//! density/displacement (G3), no temperature/pressure (G4).
//!
//! Note: the default 8×8 fixture has 28 boundary-ring blocks, so absolute
//! `matter_count` includes those. Per-material counts are used where the
//! invariant is about a specific Matter.

use powdergame_core::{
    registry_contains, WorldConfig, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_OIL,
    MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER,
};
use powdergame_gpu::{GpuError, Simulation};

fn make_sim(config: WorldConfig) -> Simulation {
    pollster::block_on(Simulation::new(config)).expect("DX12 + RTX 5090 simulation init")
}

/// Default 8×8 fixture: outer ring BOUNDARY_BLOCK, EMPTY interior.
fn eight_by_eight() -> Simulation {
    make_sim(WorldConfig::new(8, 8, 8).unwrap())
}

fn cell(sim: &Simulation, x: i64, y: i64) -> u32 {
    sim.world
        .read_material_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("cell readback")
}

fn set(sim: &Simulation, x: i64, y: i64, id: u32) {
    sim.world
        .write_material(&sim.context.queue, x, y, id)
        .expect("validated edit must succeed")
}

/// Total non-EMPTY cells (includes the boundary ring).
fn matter_count(sim: &Simulation) -> usize {
    sim.world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("full readback")
        .iter()
        .filter(|&&v| v != MATERIAL_EMPTY)
        .count()
}

/// Count of a specific material value across the world.
fn count_material(sim: &Simulation, id: u32) -> usize {
    sim.world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("full readback")
        .iter()
        .filter(|&&v| v == id)
        .count()
}

// ── STATIC ─────────────────────────────────────────────────────────────

#[test]
fn static_materials_never_move() {
    let mut sim = eight_by_eight();
    set(&sim, 1, 1, MATERIAL_STONE);
    set(&sim, 2, 2, MATERIAL_BOUNDARY_BLOCK); // interior-placed boundary block
    for _ in 0..3 {
        sim.tick().expect("tick");
    }
    assert_eq!(cell(&sim, 1, 1), MATERIAL_STONE, "stone must not move");
    assert_eq!(
        cell(&sim, 2, 2),
        MATERIAL_BOUNDARY_BLOCK,
        "boundary block must not move via normal movement"
    );
}

// ── POWDER ─────────────────────────────────────────────────────────────

#[test]
fn sand_falls_exactly_one_cell_per_tick() {
    let mut sim = eight_by_eight();
    set(&sim, 1, 1, MATERIAL_SAND);

    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 1, 2), MATERIAL_SAND, "sand falls down one cell");
    assert_eq!(cell(&sim, 1, 1), MATERIAL_EMPTY, "source becomes EMPTY");

    sim.tick().expect("tick");
    assert_eq!(
        cell(&sim, 1, 3),
        MATERIAL_SAND,
        "second tick: exactly one more cell"
    );
    assert_eq!(cell(&sim, 1, 2), MATERIAL_EMPTY);
    // No teleport: exactly one sand cell exists, it just changed position.
    assert_eq!(
        count_material(&sim, MATERIAL_SAND),
        1,
        "no sand duplicated or lost"
    );
}

#[test]
fn sand_takes_diagonal_when_down_blocked() {
    let mut sim = eight_by_eight();
    set(&sim, 2, 1, MATERIAL_SAND);
    set(&sim, 2, 2, MATERIAL_STONE);

    sim.tick().expect("tick");

    // Down blocked; both down-diagonals EMPTY. Parity (2+1) is odd → the
    // right diagonal is tried first. Either way it is a 1-cell diagonal.
    assert_eq!(
        cell(&sim, 3, 2),
        MATERIAL_SAND,
        "sand slid to a down-diagonal"
    );
    assert_eq!(cell(&sim, 2, 1), MATERIAL_EMPTY);
    assert_eq!(cell(&sim, 2, 2), MATERIAL_STONE, "obstacle untouched");
}

#[test]
fn sand_stops_when_fully_blocked() {
    let mut sim = eight_by_eight();
    set(&sim, 1, 1, MATERIAL_SAND);
    set(&sim, 1, 2, MATERIAL_STONE);
    set(&sim, 2, 2, MATERIAL_STONE);
    // (0,2) is the boundary ring — blocked, not a destination.

    sim.tick().expect("tick");
    sim.tick().expect("tick");
    assert_eq!(
        cell(&sim, 1, 1),
        MATERIAL_SAND,
        "fully blocked sand stays put"
    );
    assert_eq!(
        matter_count(&sim),
        31,
        "28 boundary + 1 sand + 2 stone: nothing lost"
    );
}

// ── LIQUID ─────────────────────────────────────────────────────────────

#[test]
fn water_falls_down_then_flows_laterally_one_cell() {
    // Fall.
    let mut sim = eight_by_eight();
    set(&sim, 1, 1, MATERIAL_WATER);
    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 1, 2), MATERIAL_WATER, "water falls down");

    // Lateral: down and both down-diagonals blocked → 1-cell lateral.
    let mut sim = eight_by_eight();
    set(&sim, 2, 1, MATERIAL_WATER);
    set(&sim, 2, 2, MATERIAL_STONE);
    set(&sim, 1, 2, MATERIAL_STONE);
    set(&sim, 3, 2, MATERIAL_STONE);
    sim.tick().expect("tick");
    let x_at: i64 = if cell(&sim, 1, 1) == MATERIAL_WATER {
        1
    } else if cell(&sim, 3, 1) == MATERIAL_WATER {
        3
    } else {
        panic!("water must have moved laterally by exactly one cell");
    };
    assert_eq!(
        (x_at - 2).abs(),
        1,
        "lateral is exactly one cell, no teleport"
    );
    assert_eq!(cell(&sim, 2, 1), MATERIAL_EMPTY, "source drained");
}

#[test]
fn oil_uses_the_liquid_family() {
    let mut sim = eight_by_eight();
    set(&sim, 1, 1, MATERIAL_OIL);
    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 1, 2), MATERIAL_OIL, "oil falls like a liquid");

    let mut sim = eight_by_eight();
    set(&sim, 2, 1, MATERIAL_OIL);
    set(&sim, 2, 2, MATERIAL_STONE);
    set(&sim, 1, 2, MATERIAL_STONE);
    set(&sim, 3, 2, MATERIAL_STONE);
    sim.tick().expect("tick");
    let laterals = [cell(&sim, 1, 1), cell(&sim, 3, 1)];
    assert!(
        laterals.contains(&MATERIAL_OIL),
        "oil flows laterally when blocked below: {laterals:?}"
    );
}

// ── GAS ────────────────────────────────────────────────────────────────

#[test]
fn steam_and_smoke_rise() {
    let mut sim = eight_by_eight();
    set(&sim, 6, 6, MATERIAL_STEAM);
    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 6, 5), MATERIAL_STEAM, "steam rises up");

    let mut sim = eight_by_eight();
    set(&sim, 6, 6, MATERIAL_SMOKE);
    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 6, 5), MATERIAL_SMOKE, "smoke rises up");
}

#[test]
fn gas_takes_up_diagonal_when_up_blocked() {
    let mut sim = eight_by_eight();
    set(&sim, 5, 6, MATERIAL_STEAM);
    set(&sim, 5, 5, MATERIAL_STONE);
    sim.tick().expect("tick");
    let up_diag = [cell(&sim, 4, 5), cell(&sim, 6, 5)];
    assert!(
        up_diag.contains(&MATERIAL_STEAM),
        "steam must have taken an up-diagonal: {up_diag:?}"
    );
    assert_eq!(cell(&sim, 5, 6), MATERIAL_EMPTY, "source vacated");
}

#[test]
fn gas_stable_bulk_center_does_not_swap() {
    // 3×3 steam block. Interior cells with no EMPTY in their 1-cell stencil
    // must stay put — no meaningless Gas↔Gas swaps.
    let mut sim = eight_by_eight();
    for y in 2..=4 {
        for x in 2..=4 {
            set(&sim, x, y, MATERIAL_STEAM);
        }
    }
    sim.tick().expect("tick");
    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_STEAM,
        "bulk center must not swap with neighbors"
    );
    assert_eq!(
        count_material(&sim, MATERIAL_STEAM),
        9,
        "steam that found interfaces may move, but nothing is duplicated or lost"
    );
}

// ── Ownership contention ───────────────────────────────────────────────

#[test]
fn contention_exactly_one_winner_no_duplication() {
    let mut sim = eight_by_eight();
    // Stones block the up path of both gas cells; both steam sources propose
    // the same EMPTY cell (3,1).
    set(&sim, 1, 1, MATERIAL_STONE);
    set(&sim, 2, 1, MATERIAL_STONE);
    set(&sim, 4, 1, MATERIAL_STONE);
    set(&sim, 2, 2, MATERIAL_STEAM);
    set(&sim, 4, 2, MATERIAL_STEAM);

    let before = matter_count(&sim);
    sim.tick().expect("tick");

    // Destination holds exactly one Matter.
    assert_eq!(
        cell(&sim, 3, 1),
        MATERIAL_STEAM,
        "destination has exactly one winner"
    );
    // Exactly one source won the claim (the other stays valid at its source).
    let winner_moved = cell(&sim, 2, 2) == MATERIAL_EMPTY;
    let loser_stayed = cell(&sim, 4, 2) == MATERIAL_STEAM;
    let winner_moved_alt = cell(&sim, 4, 2) == MATERIAL_EMPTY;
    let loser_stayed_alt = cell(&sim, 2, 2) == MATERIAL_STEAM;
    assert!(
        (winner_moved && loser_stayed) || (winner_moved_alt && loser_stayed_alt),
        "exactly one winner must move; loser stays valid"
    );
    // Matter is conserved and never duplicated.
    assert_eq!(matter_count(&sim), before, "total matter conserved");
    assert_eq!(
        count_material(&sim, MATERIAL_STEAM),
        2,
        "no steam duplicated or destroyed"
    );
}

// ── Chunk boundary ─────────────────────────────────────────────────────

#[test]
fn chunk_boundary_movement_is_plain_local_movement() {
    // 128×16 world: the 64-column chunk boundary sits between x=63 and x=64.
    let config = WorldConfig::new(128, 16, 64).unwrap();

    // Cross right: sand at x=63 slides into x=64.
    let mut sim = make_sim(config);
    set(&sim, 63, 1, MATERIAL_SAND);
    set(&sim, 63, 2, MATERIAL_STONE);
    set(&sim, 62, 2, MATERIAL_STONE);
    sim.tick().expect("tick");
    assert_eq!(
        cell(&sim, 64, 2),
        MATERIAL_SAND,
        "sand crossed the chunk boundary rightward"
    );
    assert_eq!(cell(&sim, 63, 1), MATERIAL_EMPTY);

    // Cross left: sand at x=64 slides into x=63.
    let mut sim = make_sim(WorldConfig::new(128, 16, 64).unwrap());
    set(&sim, 64, 1, MATERIAL_SAND);
    set(&sim, 64, 2, MATERIAL_STONE);
    set(&sim, 65, 2, MATERIAL_STONE);
    sim.tick().expect("tick");
    assert_eq!(
        cell(&sim, 63, 2),
        MATERIAL_SAND,
        "sand crossed the chunk boundary leftward"
    );
    assert_eq!(cell(&sim, 64, 1), MATERIAL_EMPTY);
}

// ── World boundary / Void ──────────────────────────────────────────────

#[test]
fn void_exit_loses_exactly_one_matter() {
    let mut sim = eight_by_eight();
    // Open the bottom boundary ring: no invisible wall may remain there.
    set(&sim, 4, 7, MATERIAL_EMPTY);
    set(&sim, 4, 6, MATERIAL_SAND);

    let before = matter_count(&sim);
    assert_eq!(before, 28, "27 boundary blocks + 1 sand");

    // Tick 1: sand drops into the opened hole.
    sim.tick().expect("tick");
    assert_eq!(
        cell(&sim, 4, 7),
        MATERIAL_SAND,
        "sand entered the opened boundary cell"
    );
    assert_eq!(cell(&sim, 4, 6), MATERIAL_EMPTY);
    assert_eq!(matter_count(&sim), before, "still inside the world");

    // Tick 2: sand falls out of the world → Void. Exactly one Matter is lost,
    // the GPU never writes outside the buffer.
    sim.tick().expect("tick");
    assert_eq!(
        cell(&sim, 4, 7),
        MATERIAL_EMPTY,
        "sand left through the open boundary"
    );
    assert_eq!(
        matter_count(&sim),
        before - 1,
        "exactly one Matter vanished into Void"
    );
}

#[test]
fn liquid_exits_through_open_side_boundary() {
    // Open the LEFT side boundary: the outward stencil candidates (which are
    // out-of-domain) must act as a Void exit, not an invisible wall.
    let mut sim = eight_by_eight();
    set(&sim, 0, 1, MATERIAL_EMPTY); // open the ring cell
    set(&sim, 0, 1, MATERIAL_WATER); // water in the edge cell
    set(&sim, 0, 2, MATERIAL_STONE); // blocks down
    set(&sim, 1, 2, MATERIAL_STONE); // blocks the inward (right) diagonal
    let water_before = count_material(&sim, MATERIAL_WATER);

    // Parity of (0,1) is odd → right (inward) diagonal first (blocked), then
    // the outward (−1,2) diagonal → out-of-domain → Void.
    sim.tick().expect("tick");

    assert_eq!(
        cell(&sim, 0, 1),
        MATERIAL_EMPTY,
        "water left the world through the open side"
    );
    assert_eq!(
        count_material(&sim, MATERIAL_WATER),
        water_before - 1,
        "exactly one water vanished into Void (no duplication, no invisible wall)"
    );
}

#[test]
fn powder_diagonal_void_exit() {
    // Powder at the top-left corner with down blocked: the first-match
    // diagonal (outward, out-of-domain) is a Void exit through the open side.
    let mut sim = eight_by_eight();
    set(&sim, 0, 0, MATERIAL_EMPTY); // open the ring corner
    set(&sim, 0, 0, MATERIAL_SAND);
    set(&sim, 0, 1, MATERIAL_STONE); // blocks down
    let sand_before = count_material(&sim, MATERIAL_SAND);

    // Parity of (0,0) is even → left diagonal first → (−1,1) is OOB → Void.
    sim.tick().expect("tick");

    assert_eq!(
        cell(&sim, 0, 0),
        MATERIAL_EMPTY,
        "sand left through the open side"
    );
    assert_eq!(
        count_material(&sim, MATERIAL_SAND),
        sand_before - 1,
        "exactly one sand vanished into Void"
    );
}

// ── G1 contract regression inside the G2 tick loop ─────────────────────

#[test]
fn g2_tick_preserves_g1_contracts() {
    let mut sim = eight_by_eight();

    // Invalid material values are still rejected by the edit path.
    assert!(matches!(
        sim.world.write_material(&sim.context.queue, 1, 1, 999),
        Err(GpuError::InvalidMaterialValue(999))
    ));
    assert!(matches!(
        sim.world.write_material(&sim.context.queue, 1, 1, u32::MAX),
        Err(GpuError::InvalidMaterialValue(_))
    ));

    // Out-of-bounds coordinates are Void: rejected, never clamped.
    assert!(matches!(
        sim.world
            .read_material_cell(&sim.context.device, &sim.context.queue, -1, 1),
        Err(GpuError::CoordinateOutOfBounds { x: -1, y: 1 })
    ));
    assert!(matches!(
        sim.world
            .write_material(&sim.context.queue, 8, 0, MATERIAL_EMPTY),
        Err(GpuError::CoordinateOutOfBounds { x: 8, y: 0 })
    ));

    // The boundary ring is editable Matter, not an immutable hidden wall.
    sim.world
        .write_material(&sim.context.queue, 0, 0, MATERIAL_EMPTY)
        .expect("boundary erase must succeed");
    assert_eq!(
        cell(&sim, 0, 0),
        MATERIAL_EMPTY,
        "erased boundary cell is EMPTY"
    );

    // EMPTY is still not a registered Matter.
    assert!(!registry_contains(MATERIAL_EMPTY));

    // Headless lifecycle with the movement pipeline works and executes on GPU.
    sim.tick().expect("tick");
    sim.tick().expect("tick");
    assert_eq!(sim.tick_count, 2);
    assert_eq!(
        sim.read_marker().expect("marker"),
        1,
        "movement dispatch executed on GPU"
    );
    // The erased corner stays EMPTY (nothing clamps back into the world).
    assert_eq!(cell(&sim, 0, 0), MATERIAL_EMPTY);
}

// ── Performance ────────────────────────────────────────────────────────

/// Coarse sanity observation (runs in normal `cargo test`; 30 ticks only).
#[test]
fn coarse_reference_world_perf() {
    let mut sim = make_sim(WorldConfig::reference());
    const TICKS: u32 = 30;
    let start = std::time::Instant::now();
    for _ in 0..TICKS {
        sim.tick().expect("reference world tick");
    }
    // Wait for actual GPU execution of the last submitted tick before
    // stopping the timer. `PollType::Wait` blocks until the most recent
    // submission has completed execution (wgpu-types 26 docs), so the
    // measured time includes GPU work, not just CPU submission.
    let _ = sim.context.device.poll(wgpu::PollType::Wait);
    let elapsed = start.elapsed();
    let per_tick_ms = elapsed.as_secs_f64() * 1000.0 / f64::from(TICKS);
    let approx_tps = f64::from(TICKS) / elapsed.as_secs_f64();
    eprintln!(
        "[powdergame-g2][perf-sanity] world={}x{} ticks={TICKS} elapsed={elapsed:?} per_tick={per_tick_ms:.3} ms approx_tps={approx_tps:.1} \
         (coarse sanity observation; not a baseline)",
        sim.world.config.width, sim.world.config.height
    );
    assert_eq!(
        sim.read_marker().expect("marker"),
        1,
        "the 2048×2048 movement pipeline must actually execute on the GPU"
    );
}

/// Controlled idle-machine performance baseline.
///
/// Run explicitly (ignored by default so normal `cargo test` stays fast):
///
/// ```text
/// cargo test --release -p powdergame-gpu --test movement \
///   controlled_reference_world_perf -- --ignored --nocapture
/// ```
///
/// Protocol: create the simulation once, warm up 100 ticks (excluded), then
/// measure 5 runs of 1000 ticks each, waiting for GPU completion
/// (`PollType::Wait`) before each timer stop. The median per-tick time is
/// the official G2 baseline.
///
/// Measurement is: **controlled idle-machine, release, coarse end-to-end
/// wall-clock including GPU completion** — NOT a GPU timestamp benchmark.
#[test]
#[ignore]
fn controlled_reference_world_perf() {
    const WARM_UP_TICKS: u32 = 100;
    const MEASURE_TICKS: u32 = 1000;
    const RUNS: usize = 5;

    let mut sim = make_sim(WorldConfig::reference());

    // Warm-up: exclude initialization and first-submission effects.
    for _ in 0..WARM_UP_TICKS {
        sim.tick().expect("warm-up tick");
    }
    let _ = sim.context.device.poll(wgpu::PollType::Wait);

    let mut per_tick_samples = Vec::with_capacity(RUNS);
    for run in 1..=RUNS {
        let start = std::time::Instant::now();
        for _ in 0..MEASURE_TICKS {
            sim.tick().expect("measured tick");
        }
        let _ = sim.context.device.poll(wgpu::PollType::Wait); // GPU completion
        let elapsed = start.elapsed();
        let per_tick_ms = elapsed.as_secs_f64() * 1000.0 / f64::from(MEASURE_TICKS);
        let approx_tps = f64::from(MEASURE_TICKS) / elapsed.as_secs_f64();
        eprintln!(
            "[powdergame-g2][perf-ctrl] run {run}/{RUNS}: ticks={MEASURE_TICKS} elapsed={elapsed:?} per_tick={per_tick_ms:.4} ms approx_tps={approx_tps:.1}"
        );
        per_tick_samples.push(per_tick_ms);
    }

    per_tick_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_ms = per_tick_samples[RUNS / 2];
    let median_tps = 1000.0 / median_ms;
    eprintln!(
        "[powdergame-g2][perf-ctrl] MEDIAN per_tick={median_ms:.4} ms approx_tps={median_tps:.1} \
         (release, controlled idle-machine, coarse end-to-end wall-clock incl. GPU completion; \
         not a GPU timestamp benchmark)"
    );

    assert_eq!(
        sim.read_marker().expect("marker"),
        1,
        "the 2048×2048 movement pipeline must actually execute on the GPU"
    );
}
