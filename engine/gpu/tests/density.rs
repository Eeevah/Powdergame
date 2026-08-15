//! G3 — Density / Displacement: GPU semantic/invariant tests.
//!
//! Runs on the actual machine (Windows + RTX 5090 + DX12). G3 replaces the
//! G2 EMPTY-only movement with local density displacement: a heavier Matter
//! sinks through a lighter one via **local swaps** on vertical stencils only
//! (down/down-diagonal for POWDER/LIQUID, up/up-diagonal for GAS); lateral
//! candidates stay EMPTY-only. "Buoyancy is not computed, it is sorted"
//! (SIMULATION_SPEC §12).
//!
//! Ownership: every move/swap is an edge claimed by BOTH endpoints (source
//! claims its edge, the destination reciprocates). Overlapping edges resolve
//! by fixed min-source arbitration — one cell never joins two ownership
//! changes, nothing is duplicated or lost (Void exits excepted).
//!
//! No exact pixel checksums are required (DETERMINISM_SPEC §7); semantic
//! invariants are what matter. Density is a Material table property — there
//! are no per-cell density buffers.
//!
//! G4-B note: Steam now has a condensation phase rule, so Steam fixtures
//! place Steam at a stable hot temperature (above the 40.0 condensation
//! threshold). The density intent itself is unchanged.

use powdergame_core::{
    WorldConfig, MATERIAL_EMPTY, MATERIAL_OIL, MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STEAM,
    MATERIAL_STONE, MATERIAL_WATER,
};
use powdergame_gpu::Simulation;

/// Stable hot temperature for Steam fixtures (above condensation 40.0).
const STEAM_STABLE_T: f32 = 80.0;
/// Hotter Steam for the long sealed-channel ordering test: it must survive
/// many ticks of conduction with cold Smoke/Stone and never condense.
const STEAM_VERY_HOT_T: f32 = 120.0;

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
        .expect("validated edit must succeed");
}

fn set_t(sim: &Simulation, x: i64, y: i64, t: f32) {
    sim.world
        .write_temperature(&sim.context.queue, x, y, t)
        .expect("validated temperature edit must succeed");
}

/// Seals a 1-cell-wide vertical channel at column `cx` with stone walls on
/// both sides (y 1..=6), so nothing inside can escape sideways.
fn seal_channel(sim: &Simulation, cx: i64) {
    for y in 1..=6 {
        set(sim, cx - 1, y, MATERIAL_STONE);
        set(sim, cx + 1, y, MATERIAL_STONE);
    }
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

/// Total non-EMPTY cells (includes the boundary ring).
fn matter_count(sim: &Simulation) -> usize {
    sim.world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("full readback")
        .iter()
        .filter(|&&v| v != MATERIAL_EMPTY)
        .count()
}

// ── A. Sand / Water swap ────────────────────────────────────────────────

#[test]
fn sand_swaps_with_water_below() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_SAND);
    set(&sim, 3, 4, MATERIAL_WATER);
    let before = matter_count(&sim);

    sim.tick().expect("tick");

    // Sand (150) sinks through Water (90): they exchange places in ONE tick.
    assert_eq!(cell(&sim, 3, 4), MATERIAL_SAND, "sand sank");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_WATER, "water displaced upward");
    assert_eq!(count_material(&sim, MATERIAL_SAND), 1);
    assert_eq!(count_material(&sim, MATERIAL_WATER), 1);
    assert_eq!(matter_count(&sim), before, "swap conserves Matter");
}

// ── B. Sand sinking through a water column ──────────────────────────────

#[test]
fn sand_sinks_through_water_column() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_SAND);
    for y in 4..=6 {
        set(&sim, 3, y, MATERIAL_WATER);
    }

    for _ in 0..6 {
        sim.tick().expect("tick");
    }

    // Sand reached the bottom of the water column (y=6, above the ring).
    assert_eq!(cell(&sim, 3, 6), MATERIAL_SAND, "sand sank to the bottom");
    assert_eq!(
        count_material(&sim, MATERIAL_SAND),
        1,
        "no sand duplicated/lost"
    );
    assert_eq!(
        count_material(&sim, MATERIAL_WATER),
        3,
        "water conserved through displacement"
    );
}

// ── C. Water / Oil inversion swap ───────────────────────────────────────

#[test]
fn water_swaps_with_oil_below() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WATER);
    set(&sim, 3, 4, MATERIAL_OIL);
    let before = matter_count(&sim);

    sim.tick().expect("tick");

    // Water (90) sinks through Oil (70): water ends below, oil above.
    assert_eq!(cell(&sim, 3, 4), MATERIAL_WATER, "water sank below oil");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_OIL, "oil rose above water");
    assert_eq!(matter_count(&sim), before);
}

// ── D. Stable oil above water (no swap) ─────────────────────────────────

#[test]
fn oil_above_water_does_not_swap() {
    // Sealed channel: oil directly above water at the bottom of the column.
    // This is the stable ordering — nothing may move on density alone.
    let mut sim = eight_by_eight();
    seal_channel(&sim, 3);
    set(&sim, 3, 5, MATERIAL_OIL);
    set(&sim, 3, 6, MATERIAL_WATER);

    for _ in 0..5 {
        sim.tick().expect("tick");
    }

    assert_eq!(cell(&sim, 3, 5), MATERIAL_OIL, "oil stays above water");
    assert_eq!(cell(&sim, 3, 6), MATERIAL_WATER, "water stays below oil");
}

// ── E. Multi-cell layer separation ──────────────────────────────────────

#[test]
fn mixed_water_oil_channel_separates_into_layers() {
    // Sealed channel: intentionally mixed/inverted (water above oil).
    let mut sim = eight_by_eight();
    seal_channel(&sim, 3);
    set(&sim, 3, 2, MATERIAL_OIL);
    set(&sim, 3, 3, MATERIAL_WATER);
    set(&sim, 3, 4, MATERIAL_OIL);
    set(&sim, 3, 5, MATERIAL_WATER);

    for _ in 0..40 {
        sim.tick().expect("tick");
    }

    // Semantic ordering: every water cell below every oil cell in the column.
    let all = sim
        .world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("readback");
    let mut oil_ys = Vec::new();
    let mut water_ys = Vec::new();
    for y in 1..=6 {
        let v = all[(y * 8 + 3) as usize];
        if v == MATERIAL_OIL {
            oil_ys.push(y);
        } else if v == MATERIAL_WATER {
            water_ys.push(y);
        }
    }
    assert_eq!(oil_ys.len(), 2, "oil conserved");
    assert_eq!(water_ys.len(), 2, "water conserved");
    let max_oil = oil_ys.iter().max().unwrap();
    let min_water = water_ys.iter().min().unwrap();
    assert!(
        max_oil < min_water,
        "oil must float above water (oil max y {max_oil} < water min y {min_water})"
    );
}

// ── F. Equal rank never swaps ───────────────────────────────────────────

#[test]
fn equal_rank_water_does_not_jitter() {
    // Sealed channel with two stacked water cells at the bottom: same rank,
    // nothing to fall into, no interface — they must never swap or jitter.
    let mut sim = eight_by_eight();
    seal_channel(&sim, 3);
    set(&sim, 3, 5, MATERIAL_WATER);
    set(&sim, 3, 6, MATERIAL_WATER);

    for _ in 0..5 {
        sim.tick().expect("tick");
    }

    assert_eq!(cell(&sim, 3, 5), MATERIAL_WATER);
    assert_eq!(cell(&sim, 3, 6), MATERIAL_WATER);
    assert_eq!(
        count_material(&sim, MATERIAL_WATER),
        2,
        "no pointless motion"
    );
}

// ── G. STATIC exclusion ─────────────────────────────────────────────────

#[test]
fn static_targets_never_swap() {
    let mut sim = eight_by_eight();
    seal_channel(&sim, 3);
    set(&sim, 3, 4, MATERIAL_SAND);
    set(&sim, 3, 5, MATERIAL_STONE);
    set(&sim, 3, 6, MATERIAL_STONE);

    for _ in 0..5 {
        sim.tick().expect("tick");
    }

    assert_eq!(cell(&sim, 3, 4), MATERIAL_SAND, "sand blocked by stone");
    assert_eq!(cell(&sim, 3, 5), MATERIAL_STONE, "stone never displaced");
}

// ── H. Gas density ordering ─────────────────────────────────────────────

#[test]
fn steam_swaps_up_through_smoke_when_blocked_above() {
    // Steam below Smoke in a sealed channel. The smoke's up is blocked, so
    // it cannot escape: the lighter steam swaps upward through it.
    let mut sim = eight_by_eight();
    seal_channel(&sim, 3);
    set(&sim, 3, 2, MATERIAL_STONE); // block smoke's escape upward
    set(&sim, 3, 3, MATERIAL_SMOKE);
    set(&sim, 3, 4, MATERIAL_STEAM);
    set_t(&sim, 3, 4, STEAM_STABLE_T); // G4-B: Steam must stay Steam
    let before = matter_count(&sim);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_STEAM, "steam rose through smoke");
    assert_eq!(cell(&sim, 3, 4), MATERIAL_SMOKE, "smoke sank below steam");
    assert_eq!(matter_count(&sim), before);
}

#[test]
fn stable_gas_ordering_does_not_swap() {
    // Steam above Smoke (lighter above heavier) is stable: no swap.
    let mut sim = eight_by_eight();
    seal_channel(&sim, 3);
    set(&sim, 3, 2, MATERIAL_STONE);
    set(&sim, 3, 3, MATERIAL_STEAM);
    set_t(&sim, 3, 3, STEAM_STABLE_T);
    set(&sim, 3, 4, MATERIAL_SMOKE);

    for _ in 0..5 {
        sim.tick().expect("tick");
    }

    assert_eq!(cell(&sim, 3, 3), MATERIAL_STEAM, "steam stays on top");
    assert_eq!(cell(&sim, 3, 4), MATERIAL_SMOKE, "smoke stays below");
}

#[test]
fn gas_channel_orders_steam_above_smoke() {
    // Inverted gas arrangement in a sealed channel: smoke above, steam
    // below. Over ticks the lighter steam works upward past the heavier
    // smoke until the column is ordered (steam on top).
    let mut sim = eight_by_eight();
    seal_channel(&sim, 3);
    set(&sim, 3, 3, MATERIAL_SMOKE);
    set(&sim, 3, 4, MATERIAL_SMOKE);
    set(&sim, 3, 5, MATERIAL_STEAM);
    set_t(&sim, 3, 5, STEAM_VERY_HOT_T); // G4-B: hot enough to never condense
    set(&sim, 3, 6, MATERIAL_STEAM);
    set_t(&sim, 3, 6, STEAM_VERY_HOT_T);

    // 12 ticks: the ordering completes in ~5 ticks; the shorter run keeps
    // the hot Steam well above the condensation threshold despite
    // conduction with the cold Smoke/Stone (no phase interference).
    for _ in 0..12 {
        sim.tick().expect("tick");
    }

    let all = sim
        .world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("readback");
    let mut steam_ys = Vec::new();
    let mut smoke_ys = Vec::new();
    for y in 1..=6 {
        let v = all[(y * 8 + 3) as usize];
        if v == MATERIAL_STEAM {
            steam_ys.push(y);
        } else if v == MATERIAL_SMOKE {
            smoke_ys.push(y);
        }
    }
    assert_eq!(steam_ys.len(), 2, "steam conserved");
    assert_eq!(smoke_ys.len(), 2, "smoke conserved");
    let max_steam = steam_ys.iter().max().unwrap();
    let min_smoke = smoke_ys.iter().min().unwrap();
    assert!(
        max_steam < min_smoke,
        "lighter steam must end above heavier smoke (steam max y {max_steam} < smoke min y {min_smoke})"
    );
}

// ── I. Overlapping edges (chain) ────────────────────────────────────────

#[test]
fn overlapping_swap_chain_corrupts_nothing() {
    // Chain: sand→water swap candidate, water→oil swap candidate.
    // Water is the destination of one edge AND the source of another; it
    // must join exactly one. Min-source arbitration picks the upper edge
    // (owner (3,2) < (3,3)); the lower edge conservatively fails.
    let mut sim = eight_by_eight();
    set(&sim, 3, 2, MATERIAL_SAND);
    set(&sim, 3, 3, MATERIAL_WATER);
    set(&sim, 3, 4, MATERIAL_OIL);
    let before = matter_count(&sim);

    sim.tick().expect("tick");

    // Exactly one cell = one Matter, counts conserved, no duplication/loss.
    assert_eq!(cell(&sim, 3, 2), MATERIAL_WATER, "upper edge executed");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_SAND, "upper edge executed");
    assert_eq!(
        cell(&sim, 3, 4),
        MATERIAL_OIL,
        "lower edge conservatively failed"
    );
    assert_eq!(count_material(&sim, MATERIAL_SAND), 1);
    assert_eq!(count_material(&sim, MATERIAL_WATER), 1);
    assert_eq!(count_material(&sim, MATERIAL_OIL), 1);
    assert_eq!(matter_count(&sim), before);
}

// ── J. Contention: two sources, one destination ─────────────────────────

#[test]
fn density_contention_exactly_one_winner() {
    // Two sands both propose sinking into the same water cell (3,3).
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WATER);
    set(&sim, 3, 2, MATERIAL_SAND);
    set(&sim, 4, 2, MATERIAL_SAND);
    set(&sim, 4, 3, MATERIAL_STONE); // forces the second sand's diagonal → water
    let before = matter_count(&sim);

    sim.tick().expect("tick");

    // Winner exactly one (min source index (3,2)); the loser stays valid.
    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_SAND,
        "destination has exactly one winner"
    );
    let winner_moved = cell(&sim, 3, 2) == MATERIAL_WATER;
    let loser_stayed = cell(&sim, 4, 2) == MATERIAL_SAND;
    assert!(
        winner_moved && loser_stayed,
        "one source wins the swap, the other stays put"
    );
    assert_eq!(
        count_material(&sim, MATERIAL_SAND),
        2,
        "no duplication/loss"
    );
    assert_eq!(count_material(&sim, MATERIAL_WATER), 1);
    assert_eq!(matter_count(&sim), before);
}

// ── K. Chunk boundary ───────────────────────────────────────────────────

#[test]
fn density_swap_crosses_chunk_boundary() {
    // 128×16 world: the 64-column chunk boundary sits between x=63 and x=64.
    // Sand at x=63 sinks diagonally into water at x=64 — a density swap that
    // crosses the chunk boundary (and the claim pass reads the neighbor's
    // proposal across it). Chunks must never behave as density walls.
    let mut sim = make_sim(WorldConfig::new(128, 16, 64).unwrap());
    set(&sim, 63, 2, MATERIAL_SAND);
    set(&sim, 63, 3, MATERIAL_STONE); // block sand's straight down
    set(&sim, 64, 3, MATERIAL_WATER);
    let before = matter_count(&sim);

    sim.tick().expect("tick");

    assert_eq!(
        cell(&sim, 64, 3),
        MATERIAL_SAND,
        "sand sank across the boundary"
    );
    assert_eq!(cell(&sim, 63, 2), MATERIAL_WATER, "water displaced upward");
    assert_eq!(matter_count(&sim), before);
}

// ── L. Void regression with density ─────────────────────────────────────

#[test]
fn sand_sinks_then_exits_through_open_boundary() {
    // Open the bottom ring; water in the hole column, sand above the water.
    // The sand sinks through the water, drops into the hole, then leaves
    // through the open boundary into Void — exactly one sand lost, water
    // follows, no OOB write.
    let mut sim = eight_by_eight();
    set(&sim, 4, 7, MATERIAL_EMPTY); // open the ring cell
    set(&sim, 3, 6, MATERIAL_STONE); // keep the water column straight
    set(&sim, 5, 6, MATERIAL_STONE);
    set(&sim, 4, 6, MATERIAL_WATER);
    set(&sim, 4, 5, MATERIAL_SAND);
    let sand_before = count_material(&sim, MATERIAL_SAND);
    let water_before = count_material(&sim, MATERIAL_WATER);

    for _ in 0..6 {
        sim.tick().expect("tick");
    }

    assert_eq!(
        count_material(&sim, MATERIAL_SAND),
        sand_before - 1,
        "exactly one sand vanished into Void"
    );
    assert_eq!(
        count_material(&sim, MATERIAL_WATER),
        water_before - 1,
        "exactly one water followed into Void"
    );
    assert_eq!(cell(&sim, 4, 7), MATERIAL_EMPTY, "hole stays open");
}

// ── Marker / pipeline executes on the GPU ───────────────────────────────

#[test]
fn density_pipeline_executes_on_gpu() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_SAND);
    set(&sim, 3, 4, MATERIAL_WATER);
    sim.tick().expect("tick");
    assert_eq!(
        sim.read_marker().expect("marker"),
        1,
        "G3 propose dispatch executed on the GPU"
    );
}
