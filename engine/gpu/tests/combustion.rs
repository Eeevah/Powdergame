//! G4-C — Combustion: GPU semantic/invariant tests (Windows + RTX 5090 +
//! DX12).
//!
//! Wood and Oil share ONE generic combustion grammar (`REACTION_SPEC` §11):
//! Material-owned descriptor → ignition / sustain / heat / Smoke request /
//! flame presentation event. No Oxygen requirement, no Fire Material —
//! flame is Matter + `COMBUSTING` + heat + `FLAME_EVENT`.
//!
//! Tick causal order (per tick): movement (Matter carries Temperature AND
//! combustion flags on the ownership edge) → thermal → phase → combustion →
//! smoke claim/commit. These tests prove the chain is actually connected:
//! hot conduction ignites, ignition adds heat, burning spawns Smoke with
//! ownership (winner exactly one), flags follow Matter through
//! move/swap/Void, and Pressure is never transported.
//!
//! `flags[]` is a Matter-owned field: EMPTY cells always have flags == 0,
//! combustion touches only its own bits, and the smoke spawn pass never
//! inherits combustion identity/state into the new Smoke.

use powdergame_core::{
    combustion_flag_mask, decay_age, decay_flag_mask, fuel_progress, vacuum_air_state,
    with_decay_age, with_fuel_progress, AirState, WorldConfig, FLAG_COMBUSTING, FLAG_FLAME_EVENT,
    MATERIAL_EMPTY, MATERIAL_OIL, MATERIAL_REGISTRY, MATERIAL_SMOKE, MATERIAL_STEAM,
    MATERIAL_STONE, MATERIAL_WOOD, STANDARD_AIR_ENERGY, TEMPERATURE_REFERENCE,
};
use powdergame_gpu::Simulation;

/// A future-subsystem flag bit that combustion must never touch. Chosen
/// OUTSIDE the combustion-owned bits (0..1 bools, 8..23 fuel progress) so
/// it cannot collide with the fuel-progress field.
const TEST_UNRELATED_FLAG: u32 = 1 << 28;

/// Gameplay fuel life in active burn ticks (must match the core baseline).
const COMBUSTION_WOOD_BURN_TICKS: u32 = 900;
const COMBUSTION_OIL_BURN_TICKS: u32 = 600;

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

fn set_flags(sim: &Simulation, x: i64, y: i64, flags: u32) {
    sim.world
        .write_flags(&sim.context.queue, x, y, flags)
        .expect("validated flags edit must succeed");
}

fn flags(sim: &Simulation, x: i64, y: i64) -> u32 {
    sim.world
        .read_flags_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("flags readback")
}

fn temp(sim: &Simulation, x: i64, y: i64) -> f32 {
    sim.world
        .read_temperature_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("temperature readback")
}

fn air(sim: &Simulation, cells: &[(i64, i64)]) -> Vec<powdergame_gpu::EnvironmentCellSnapshot> {
    sim.world
        .read_environment_cells(&sim.context.device, &sim.context.queue, cells)
        .expect("bounded Environment readback")
}

fn count_material(sim: &Simulation, id: u32) -> usize {
    sim.world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("full readback")
        .iter()
        .filter(|&&v| v == id)
        .count()
}

/// Fills all 8 neighbors of (x, y) with Stone (interior fixture only).
fn seal_eight(sim: &Simulation, x: i64, y: i64) {
    for dx in -1..=1 {
        for dy in -1..=1 {
            if dx != 0 || dy != 0 {
                set(sim, x + dx, y + dy, MATERIAL_STONE);
            }
        }
    }
}

/// Ticks `stride` ticks at a time and checks `pred` before each batch (and
/// after the final one), up to `max_ticks` total. Returns the tick index at
/// which the predicate first held (or `None`). GPU readbacks are cheap
/// relative to long simulations, so polling every `stride` ticks is fine.
fn tick_until(
    sim: &mut Simulation,
    max_ticks: u64,
    stride: u64,
    mut pred: impl FnMut(&Simulation) -> bool,
) -> Option<u64> {
    let mut t = 0u64;
    while t < max_ticks {
        if pred(sim) {
            return Some(t);
        }
        for _ in 0..stride {
            sim.tick().expect("tick");
        }
        t += stride;
    }
    if pred(sim) {
        Some(t)
    } else {
        None
    }
}

// ---------------------------------------------------------------------
// Ignition / heat / extinguish
// ---------------------------------------------------------------------

#[test]
fn hot_oil_ignites() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_OIL);
    set_t(&sim, 3, 3, 80.0); // >= Oil ignition 75
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    assert_ne!(
        flags(&sim, 3, 3) & FLAG_COMBUSTING,
        0,
        "hot Oil must ignite"
    );
    assert_ne!(
        flags(&sim, 3, 3) & FLAG_FLAME_EVENT,
        0,
        "ignition emits a flame presentation event"
    );
    assert!(
        temp(&sim, 3, 3) > 80.0,
        "ignition tick also adds combustion heat"
    );
    assert_eq!(count_material(&sim, MATERIAL_OIL), 1);
}

#[test]
fn hot_wood_ignites() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 95.0); // >= Wood ignition 90
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    assert_ne!(
        flags(&sim, 3, 3) & FLAG_COMBUSTING,
        0,
        "hot Wood must ignite"
    );
    assert_eq!(count_material(&sim, MATERIAL_WOOD), 1);
}

#[test]
fn cold_oil_does_not_ignite() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_OIL);
    set_t(&sim, 3, 3, 40.0); // below ignition 75
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    assert_eq!(
        flags(&sim, 3, 3) & FLAG_COMBUSTING,
        0,
        "cold Oil stays unlit"
    );
    assert_eq!(flags(&sim, 3, 3) & FLAG_FLAME_EVENT, 0);
}

#[test]
fn cold_wood_does_not_ignite() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 50.0); // below Wood ignition 90
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    assert_eq!(
        flags(&sim, 3, 3) & FLAG_COMBUSTING,
        0,
        "cold Wood stays unlit"
    );
}

#[test]
fn burning_adds_heat() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_OIL);
    set_t(&sim, 3, 3, 80.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    assert_ne!(flags(&sim, 3, 3) & FLAG_COMBUSTING, 0, "still burning");
    assert!(
        temp(&sim, 3, 3) > 80.0,
        "burning adds heat_per_tick; got {}",
        temp(&sim, 3, 3)
    );
}

#[test]
fn cooling_below_sustain_extinguishes() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_OIL);
    set_t(&sim, 3, 3, 40.0); // below Oil sustain 45
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    assert_eq!(
        flags(&sim, 3, 3) & FLAG_COMBUSTING,
        0,
        "burning below sustain must extinguish"
    );
    assert_eq!(flags(&sim, 3, 3) & FLAG_FLAME_EVENT, 0);
}

#[test]
fn nonflammable_hot_material_does_not_combust() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_STONE);
    set_t(&sim, 3, 3, 100.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING); // even a stale bit cannot burn Stone
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    assert_eq!(flags(&sim, 3, 3) & FLAG_COMBUSTING, 0, "Stone never burns");
}

#[test]
fn no_oxygen_requirement() {
    // Fully sealed stone chamber: no air concept exists, yet a hot Wood
    // ignites from its own thermal state alone (REACTION_SPEC §11).
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 95.0);
    seal_eight(&sim, 3, 3); // complete enclosure, smoke also blocked

    sim.tick().expect("tick");

    assert_ne!(
        flags(&sim, 3, 3) & FLAG_COMBUSTING,
        0,
        "sealed Wood ignites"
    );
    assert_eq!(
        count_material(&sim, MATERIAL_SMOKE),
        0,
        "no Smoke spawn into a sealed chamber"
    );
}

#[test]
fn flame_event_emitted_on_ignition() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_OIL);
    set_t(&sim, 3, 3, 80.0);
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    assert_ne!(
        flags(&sim, 3, 3) & FLAG_FLAME_EVENT,
        0,
        "ignition emits the presentation signal"
    );
}

#[test]
fn combustion_flag_ownership_clears_reserved_bits() {
    // Bit 28 is reserved. TE-1 makes Matter flag ownership exact rather than
    // preserving unknown state across an identity stage.
    let unrelated = 1u32 << 28;
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_OIL);
    set_t(&sim, 3, 3, 80.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING | unrelated);
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    let f = flags(&sim, 3, 3);
    assert_eq!(f & unrelated, 0, "reserved bits are cleared");
    assert_ne!(f & FLAG_COMBUSTING, 0);
}

// ---------------------------------------------------------------------
// Flags ownership (movement / swap / Void / blocked)
// ---------------------------------------------------------------------

#[test]
fn burning_oil_carries_flags_when_moving() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 4, MATERIAL_OIL);
    set_t(&sim, 3, 4, 80.0);
    set_flags(&sim, 3, 4, FLAG_COMBUSTING);
    // (2,4) remains EMPTY as the Air receiver for Smoke at the vacated source.
    set(&sim, 4, 4, MATERIAL_STONE);
    set(&sim, 3, 3, MATERIAL_STONE);
    set(&sim, 2, 5, MATERIAL_STONE);
    set(&sim, 4, 5, MATERIAL_STONE);
    set(&sim, 3, 6, MATERIAL_STONE);
    // (3,5) stays EMPTY → Oil falls one cell.

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 5), MATERIAL_OIL, "Oil falls into EMPTY below");
    assert_ne!(
        flags(&sim, 3, 5) & FLAG_COMBUSTING,
        0,
        "COMBUSTING travels with Matter on the move edge"
    );
    // The vacated cell receives the Smoke spawn (not a stale Oil flag).
    assert_eq!(cell(&sim, 3, 4), MATERIAL_SMOKE);
    assert_eq!(flags(&sim, 3, 4), 0, "vacated source flags are zero");
    assert!(
        temp(&sim, 3, 4) > 75.0,
        "spawned Smoke carries the burning source's hot temperature"
    );
}

#[test]
fn burning_matter_swap_carries_flags() {
    let mut sim = eight_by_eight();
    // Burning Oil above Smoke: density ordering allows the swap (70 > 30).
    set(&sim, 3, 4, MATERIAL_OIL);
    set_t(&sim, 3, 4, 80.0);
    set_flags(&sim, 3, 4, FLAG_COMBUSTING);
    set(&sim, 3, 5, MATERIAL_SMOKE);
    set_t(&sim, 3, 5, 0.0);
    // Seal every other stencil candidate.
    set(&sim, 3, 3, MATERIAL_STONE);
    // (2,4) stays EMPTY as the orthogonal Air receiver for the Smoke target.
    set(&sim, 4, 4, MATERIAL_STONE);
    set(&sim, 2, 5, MATERIAL_STONE);
    set(&sim, 4, 5, MATERIAL_STONE);
    set(&sim, 3, 6, MATERIAL_STONE);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 4), MATERIAL_SMOKE, "Smoke displaced upward");
    assert_eq!(cell(&sim, 3, 5), MATERIAL_OIL, "Oil sank into Smoke");
    assert_ne!(
        flags(&sim, 3, 5) & FLAG_COMBUSTING,
        0,
        "COMBUSTING follows the Oil through the density swap"
    );
    assert_eq!(
        flags(&sim, 3, 4) & (FLAG_COMBUSTING | FLAG_FLAME_EVENT),
        0,
        "Smoke never inherits Oil's combustion flags"
    );
    assert_eq!(
        decay_age(flags(&sim, 3, 4)),
        1,
        "Smoke has its own decay age 1"
    );
}

#[test]
fn burning_matter_void_exit_clears_flags() {
    let mut sim = eight_by_eight();
    // Open the bottom boundary and drop a burning Oil onto it.
    set(&sim, 3, 7, MATERIAL_EMPTY);
    set(&sim, 3, 7, MATERIAL_OIL);
    set_t(&sim, 3, 7, 80.0);
    set_flags(&sim, 3, 7, FLAG_COMBUSTING);
    set(&sim, 3, 6, MATERIAL_STONE);
    set(&sim, 2, 7, MATERIAL_STONE);
    set(&sim, 4, 7, MATERIAL_STONE);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 7), MATERIAL_EMPTY, "Oil exits through Void");
    assert_eq!(temp(&sim, 3, 7), TEMPERATURE_REFERENCE);
    assert_eq!(
        flags(&sim, 3, 7),
        0,
        "Void exit clears all Matter-owned state"
    );
    assert_eq!(count_material(&sim, MATERIAL_OIL), 0);
    assert_eq!(
        count_material(&sim, MATERIAL_SMOKE),
        0,
        "no spawn after Void exit"
    );
}

#[test]
fn blocked_or_losing_burning_matter_keeps_flags() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_OIL);
    set_t(&sim, 3, 3, 80.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);
    seal_eight(&sim, 3, 3); // fully blocked → no move

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_OIL, "blocked Matter stays");
    assert_ne!(
        flags(&sim, 3, 3) & FLAG_COMBUSTING,
        0,
        "blocked burning Matter keeps its combustion state"
    );
}

// ---------------------------------------------------------------------
// Smoke spawn
// ---------------------------------------------------------------------

#[test]
fn burning_wood_spawns_smoke() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 85.0); // >= sustain 55 → already burning below ignition
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);
    set(&sim, 2, 3, MATERIAL_STONE);
    set(&sim, 4, 3, MATERIAL_STONE);
    set(&sim, 3, 4, MATERIAL_STONE);
    // (3,2) stays EMPTY → Smoke spawns above.

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 2), MATERIAL_SMOKE, "Smoke spawns above Wood");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_WOOD, "the source Wood remains");
    assert_ne!(flags(&sim, 3, 3) & FLAG_COMBUSTING, 0);
    assert!(
        temp(&sim, 3, 2) > 80.0,
        "new Smoke derives its temperature from the burning source"
    );
    assert_eq!(
        count_material(&sim, MATERIAL_SMOKE),
        1,
        "one cell per request"
    );
    let environment = air(&sim, &[(3, 2), (3, 1), (2, 2), (4, 2)]);
    assert_eq!(
        environment[0].current,
        AirState {
            mass: 0.0,
            energy: 0.0
        }
    );
    let mut receiver_masses: Vec<f32> = environment[1..]
        .iter()
        .map(|cell| cell.current.mass)
        .collect();
    receiver_masses.sort_by(f32::total_cmp);
    assert_eq!(receiver_masses, vec![1.0, 1.0, 2.0]);
    assert!(environment[1..].iter().all(|cell| {
        cell.current.energy == STANDARD_AIR_ENERGY * cell.current.mass && cell.current == cell.next
    }));
}

#[test]
fn smoke_without_environment_receiver_is_rejected_without_touching_target_air() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 85.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);
    for (x, y) in [(3, 1), (2, 2), (4, 2), (2, 3), (4, 3), (3, 4)] {
        set(&sim, x, y, MATERIAL_STONE);
    }
    let before = air(&sim, &[(3, 2)])[0];

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 2), MATERIAL_EMPTY);
    let after = air(&sim, &[(3, 2)])[0];
    assert_eq!(before, after, "rejected Smoke must not consume or move Air");
}

#[test]
fn burning_oil_spawns_smoke() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_OIL);
    set_t(&sim, 3, 3, 80.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);
    // Seal every movement candidate so the Oil cannot slide away.
    set(&sim, 2, 3, MATERIAL_STONE);
    set(&sim, 4, 3, MATERIAL_STONE);
    set(&sim, 3, 4, MATERIAL_STONE);
    set(&sim, 2, 4, MATERIAL_STONE);
    set(&sim, 4, 4, MATERIAL_STONE);
    // (3,2) stays EMPTY → Smoke spawns above.

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 2), MATERIAL_SMOKE);
    assert_eq!(cell(&sim, 3, 3), MATERIAL_OIL, "the source Oil remains");
    assert_eq!(count_material(&sim, MATERIAL_SMOKE), 1);
}

#[test]
fn smoke_spawn_contention_exactly_one() {
    let mut sim = eight_by_eight();
    // Two burning Woods propose the SAME EMPTY cell (3,3):
    //   A at (3,4) proposes straight up; B at (4,4) has its up blocked by
    //   Stone so its parity-first up-diagonal also lands on (3,3).
    set(&sim, 3, 4, MATERIAL_WOOD);
    set_t(&sim, 3, 4, 85.0);
    set_flags(&sim, 3, 4, FLAG_COMBUSTING);
    set(&sim, 4, 4, MATERIAL_WOOD);
    set_t(&sim, 4, 4, 85.0);
    set_flags(&sim, 4, 4, FLAG_COMBUSTING);
    set(&sim, 4, 3, MATERIAL_STONE); // B's up is blocked
    set(&sim, 5, 3, MATERIAL_STONE); // B's up-diagonal-right is blocked
    set(&sim, 2, 3, MATERIAL_STONE); // A's up-diagonal-left (sealed)

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_SMOKE, "exactly one Smoke wins");
    assert_eq!(
        count_material(&sim, MATERIAL_SMOKE),
        1,
        "no duplicate spawn"
    );
    assert_eq!(
        count_material(&sim, MATERIAL_WOOD),
        2,
        "both fuel sources remain valid"
    );
    assert_ne!(flags(&sim, 3, 4) & FLAG_COMBUSTING, 0);
    assert_ne!(flags(&sim, 4, 4) & FLAG_COMBUSTING, 0);
}

#[test]
fn smoke_spawn_crosses_chunk_boundary() {
    // 128×16: the 64-column chunk boundary sits between x=63 and x=64.
    // Chunks are not combustion/spawn walls.
    let mut sim = make_sim(WorldConfig::new(128, 16, 64).unwrap());
    set(&sim, 63, 6, MATERIAL_WOOD);
    set_t(&sim, 63, 6, 85.0);
    set_flags(&sim, 63, 6, FLAG_COMBUSTING);
    set(&sim, 63, 5, MATERIAL_STONE); // up blocked
    set(&sim, 62, 5, MATERIAL_STONE); // up-diag-left blocked
    set(&sim, 64, 5, MATERIAL_STONE); // up-diag-right blocked
    set(&sim, 62, 6, MATERIAL_STONE); // lateral-left blocked
    set(&sim, 63, 7, MATERIAL_STONE); // below
                                      // (64,6) stays EMPTY → parity-ordered lateral reaches across x=63→64.

    sim.tick().expect("tick");

    assert_eq!(
        cell(&sim, 64, 6),
        MATERIAL_SMOKE,
        "Smoke spawn crosses the 64-column chunk boundary"
    );
    assert_eq!(count_material(&sim, MATERIAL_SMOKE), 1);
    assert_eq!(cell(&sim, 63, 6), MATERIAL_WOOD);
}

// ---------------------------------------------------------------------
// Integration hardening (G4-C final)
// ---------------------------------------------------------------------

#[test]
fn burning_source_keeps_heat_and_flags_while_spawning_smoke() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 85.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);
    set(&sim, 2, 3, MATERIAL_STONE);
    set(&sim, 4, 3, MATERIAL_STONE);
    set(&sim, 3, 4, MATERIAL_STONE);
    // (3,2) stays EMPTY → Smoke spawns above in the same tick.

    sim.tick().expect("tick");

    // Source: identity, flags and heat all survive the smoke spawn pass.
    assert_eq!(cell(&sim, 3, 3), MATERIAL_WOOD, "source identity preserved");
    let src_flags = flags(&sim, 3, 3);
    assert_ne!(src_flags & FLAG_COMBUSTING, 0, "source keeps COMBUSTING");
    assert_ne!(src_flags & FLAG_FLAME_EVENT, 0, "source keeps FLAME_EVENT");
    let src_temp = temp(&sim, 3, 3);
    assert!(
        src_temp > 85.0,
        "source heat is NOT rolled back by the smoke spawn; got {src_temp}"
    );

    // Destination: Smoke with the combustion-after source thermal state.
    assert_eq!(cell(&sim, 3, 2), MATERIAL_SMOKE);
    let smoke_temp = temp(&sim, 3, 2);
    assert!(smoke_temp.is_finite(), "Smoke temperature is finite");
    assert!(
        smoke_temp > 80.0,
        "Smoke carries a hot derived temperature; got {smoke_temp}"
    );
    // The Smoke temperature is exactly the source's post-combustion state
    // (smoke commit reads temperature_next[source] after heat addition).
    assert_eq!(smoke_temp, src_temp, "Smoke T == post-combustion source T");
}

#[test]
fn spawned_smoke_does_not_inherit_combustion_flags() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 85.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);
    set(&sim, 2, 3, MATERIAL_STONE);
    set(&sim, 4, 3, MATERIAL_STONE);
    set(&sim, 3, 4, MATERIAL_STONE);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 2), MATERIAL_SMOKE);
    assert_eq!(
        flags(&sim, 3, 2),
        0,
        "new Smoke inherits temperature but NEVER combustion identity/state"
    );
}

#[test]
fn reserved_flag_bit_is_cleared_by_combustion_hygiene() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 85.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING | TEST_UNRELATED_FLAG);
    set(&sim, 2, 3, MATERIAL_STONE);
    set(&sim, 4, 3, MATERIAL_STONE);
    set(&sim, 3, 4, MATERIAL_STONE);

    sim.tick().expect("tick");

    let f = flags(&sim, 3, 3);
    assert_eq!(f & TEST_UNRELATED_FLAG, 0, "reserved bit is cleared");
    assert_ne!(f & FLAG_COMBUSTING, 0, "combustion bits behave normally");
    assert_ne!(f & FLAG_FLAME_EVENT, 0);
}

#[test]
fn nonflammable_material_clears_stale_combustion_bits() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_STONE);
    set_t(&sim, 3, 3, 0.0);
    // Stale fire state + an unrelated future bit.
    set_flags(
        &sim,
        3,
        3,
        FLAG_COMBUSTING | FLAG_FLAME_EVENT | TEST_UNRELATED_FLAG,
    );
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_STONE, "material unchanged");
    let f = flags(&sim, 3, 3);
    assert_eq!(
        f & (FLAG_COMBUSTING | FLAG_FLAME_EVENT),
        0,
        "nonflammable Matter cannot keep stale combustion bits"
    );
    assert_eq!(f & TEST_UNRELATED_FLAG, 0, "reserved bit is cleared");
    assert_eq!(
        temp(&sim, 3, 3),
        TEMPERATURE_REFERENCE,
        "nonflammable combustion adds no heat"
    );
}

#[test]
fn flame_event_is_set_on_active_ticks_and_cleared_on_extinguish() {
    // Active combustion: FLAME_EVENT is re-emitted every burning tick.
    let mut active = eight_by_eight();
    set(&active, 3, 3, MATERIAL_OIL);
    set_t(&active, 3, 3, 80.0);
    set_flags(&active, 3, 3, FLAG_COMBUSTING);
    seal_eight(&active, 3, 3);
    active.tick().expect("tick");
    active.tick().expect("tick");
    assert_ne!(
        flags(&active, 3, 3) & FLAG_FLAME_EVENT,
        0,
        "active combustion keeps emitting FLAME_EVENT"
    );
    assert_ne!(flags(&active, 3, 3) & FLAG_COMBUSTING, 0);

    // Extinguishing tick: both persistent state and the pulse clear.
    let mut dying = eight_by_eight();
    set(&dying, 3, 3, MATERIAL_OIL);
    set_t(&dying, 3, 3, 46.0); // just above sustain 45; stone cooling drops below
    set_flags(&dying, 3, 3, FLAG_COMBUSTING | FLAG_FLAME_EVENT);
    seal_eight(&dying, 3, 3);
    dying.tick().expect("tick");
    assert_eq!(
        flags(&dying, 3, 3) & (FLAG_COMBUSTING | FLAG_FLAME_EVENT),
        0,
        "extinguish clears both the persistent state and the pulse"
    );
}

// ---------------------------------------------------------------------
// Integrated chains + edit invariant
// ---------------------------------------------------------------------

#[test]
fn thermal_heating_triggers_ignition() {
    // No CPU direct threshold write: the Temperature field conducts from a
    // hot Stone reservoir until Wood crosses its ignition threshold.
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 0.0);
    // A hot 3×3 stone block around the Wood (8 neighbors, all 300).
    for dx in -1..=1 {
        for dy in -1..=1 {
            if dx != 0 || dy != 0 {
                set(&sim, 3 + dx, 3 + dy, MATERIAL_STONE);
                set_t(&sim, 3 + dx, 3 + dy, 300.0);
            }
        }
    }

    for _ in 0..30 {
        sim.tick().expect("tick");
    }

    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_WOOD,
        "Wood survives the heating (STATIC, no phase)"
    );
    assert_ne!(
        flags(&sim, 3, 3) & FLAG_COMBUSTING,
        0,
        "conduction across ticks ignited the Wood (thermal → combustion chain)"
    );
}

#[test]
fn edit_replaces_material_and_clears_flags() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 85.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);
    seal_eight(&sim, 3, 3);

    // Replace the burning Wood with Stone: the new identity must not
    // inherit a stale COMBUSTING state.
    set(&sim, 3, 3, MATERIAL_STONE);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_STONE);
    assert_eq!(
        flags(&sim, 3, 3) & FLAG_COMBUSTING,
        0,
        "identity replacement clears Matter-owned combustion flags"
    );
}

// ---------------------------------------------------------------------
// Finite fuel lifecycle (G4-C hardening)
// ---------------------------------------------------------------------

#[test]
fn wood_eventually_burns_to_empty() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 85.0); // >= sustain 55, keeps burning
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);
    seal_eight(&sim, 3, 3); // fully enclosed: no move, no smoke

    for _ in 0..=COMBUSTION_WOOD_BURN_TICKS {
        sim.tick().expect("tick");
    }

    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_EMPTY,
        "Wood with finite fuel is consumed after its burn duration"
    );
    assert_eq!(count_material(&sim, MATERIAL_WOOD), 0);
    assert_eq!(
        flags(&sim, 3, 3),
        0,
        "consumed cell resets Matter-owned state"
    );
    assert_eq!(temp(&sim, 3, 3), TEMPERATURE_REFERENCE);
}

#[test]
fn oil_eventually_burns_to_empty() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_OIL);
    set_t(&sim, 3, 3, 80.0); // >= sustain 45
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);
    seal_eight(&sim, 3, 3);

    for _ in 0..=COMBUSTION_OIL_BURN_TICKS {
        sim.tick().expect("tick");
    }

    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_EMPTY,
        "Oil with finite fuel is consumed after its burn duration"
    );
    assert_eq!(count_material(&sim, MATERIAL_OIL), 0);
    assert_eq!(flags(&sim, 3, 3), 0);
    assert_eq!(temp(&sim, 3, 3), TEMPERATURE_REFERENCE);
}

#[test]
fn exact_duration_boundary_is_not_off_by_one() {
    // progress 898 (one below the 899 boundary): still burning after 1 tick.
    let mut before = eight_by_eight();
    set(&before, 3, 3, MATERIAL_WOOD);
    set_t(&before, 3, 3, 85.0);
    set_flags(&before, 3, 3, with_fuel_progress(FLAG_COMBUSTING, 898));
    seal_eight(&before, 3, 3);
    before.tick().expect("tick");
    assert_eq!(
        cell(&before, 3, 3),
        MATERIAL_WOOD,
        "still burns at progress 899"
    );
    assert_eq!(fuel_progress(flags(&before, 3, 3)), 899);

    // progress 899: the next tick reaches the burn duration and consumes.
    let mut exact = eight_by_eight();
    set(&exact, 3, 3, MATERIAL_WOOD);
    set_t(&exact, 3, 3, 85.0);
    set_flags(&exact, 3, 3, with_fuel_progress(FLAG_COMBUSTING, 899));
    seal_eight(&exact, 3, 3);
    exact.tick().expect("tick");
    assert_eq!(
        cell(&exact, 3, 3),
        MATERIAL_EMPTY,
        "reaching the burn duration consumes the fuel exactly on the boundary"
    );
    assert_eq!(flags(&exact, 3, 3), 0);
    let consumed_air = air(&exact, &[(3, 3)])[0];
    assert_eq!(consumed_air.current, vacuum_air_state());
    assert_eq!(consumed_air.current, consumed_air.next);
}

#[test]
fn extinguished_wood_keeps_partial_fuel_progress() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 20.0); // below sustain 55 → extinguish
    set_flags(&sim, 3, 3, with_fuel_progress(FLAG_COMBUSTING, 200));
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    let f = flags(&sim, 3, 3);
    assert_eq!(f & FLAG_COMBUSTING, 0, "extinguished");
    assert_eq!(f & FLAG_FLAME_EVENT, 0);
    assert_eq!(
        fuel_progress(f),
        200,
        "extinguish preserves the consumed fuel amount"
    );
}

#[test]
fn reignited_wood_continues_from_partial_progress() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 20.0); // extinguish first
    set_flags(&sim, 3, 3, with_fuel_progress(FLAG_COMBUSTING, 200));
    seal_eight(&sim, 3, 3);
    sim.tick().expect("tick");
    assert_eq!(fuel_progress(flags(&sim, 3, 3)), 200);
    assert_eq!(flags(&sim, 3, 3) & FLAG_COMBUSTING, 0);

    // Reheat past the ignition threshold: reignition continues from the
    // remaining fuel (200 → 201), never restoring fuel.
    set_t(&sim, 3, 3, 95.0);
    sim.tick().expect("tick");

    let f = flags(&sim, 3, 3);
    assert_ne!(f & FLAG_COMBUSTING, 0, "reignited");
    assert_eq!(
        fuel_progress(f),
        201,
        "reignition continues from the partial progress"
    );
}

#[test]
fn burning_oil_carries_fuel_progress_when_moving() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 4, MATERIAL_OIL);
    set_t(&sim, 3, 4, 80.0);
    set_flags(&sim, 3, 4, with_fuel_progress(FLAG_COMBUSTING, 50));
    set(&sim, 2, 4, MATERIAL_STONE);
    set(&sim, 4, 4, MATERIAL_STONE);
    set(&sim, 3, 3, MATERIAL_STONE);
    set(&sim, 2, 5, MATERIAL_STONE);
    set(&sim, 4, 5, MATERIAL_STONE);
    set(&sim, 3, 6, MATERIAL_STONE);
    // (3,5) stays EMPTY → Oil falls one cell.

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 5), MATERIAL_OIL, "Oil falls into EMPTY below");
    let f = flags(&sim, 3, 5);
    assert_ne!(f & FLAG_COMBUSTING, 0);
    assert_eq!(
        fuel_progress(f),
        51,
        "fuel progress travels with Matter and keeps counting while burning"
    );
}

#[test]
fn density_swap_carries_fuel_progress_with_identity() {
    let mut sim = eight_by_eight();
    // Burning Oil (rank 70) above Smoke (rank 30): local density swap.
    set(&sim, 3, 4, MATERIAL_OIL);
    set_t(&sim, 3, 4, 80.0);
    set_flags(&sim, 3, 4, with_fuel_progress(FLAG_COMBUSTING, 50));
    set(&sim, 3, 5, MATERIAL_SMOKE);
    set_t(&sim, 3, 5, 0.0);
    set(&sim, 3, 3, MATERIAL_STONE);
    set(&sim, 2, 4, MATERIAL_STONE);
    set(&sim, 4, 4, MATERIAL_STONE);
    set(&sim, 2, 5, MATERIAL_STONE);
    set(&sim, 4, 5, MATERIAL_STONE);
    set(&sim, 3, 6, MATERIAL_STONE);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 5), MATERIAL_OIL, "Oil sank below Smoke");
    let f = flags(&sim, 3, 5);
    assert_ne!(f & FLAG_COMBUSTING, 0);
    assert_eq!(
        fuel_progress(f),
        51,
        "fuel progress follows the identity through the swap"
    );
    assert_eq!(
        fuel_progress(flags(&sim, 3, 4)),
        0,
        "Smoke never inherits fuel progress"
    );
    assert_eq!(flags(&sim, 3, 4) & FLAG_COMBUSTING, 0);
    assert_eq!(
        decay_age(flags(&sim, 3, 4)),
        1,
        "Smoke has its own decay age 1"
    );
}

#[test]
fn void_exit_clears_fuel_progress() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 7, MATERIAL_EMPTY); // open the bottom boundary
    set(&sim, 3, 7, MATERIAL_OIL);
    set_t(&sim, 3, 7, 80.0);
    set_flags(&sim, 3, 7, with_fuel_progress(FLAG_COMBUSTING, 50));
    set(&sim, 3, 6, MATERIAL_STONE);
    set(&sim, 2, 7, MATERIAL_STONE);
    set(&sim, 4, 7, MATERIAL_STONE);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 7), MATERIAL_EMPTY, "Oil exits through Void");
    assert_eq!(flags(&sim, 3, 7), 0, "Void exit clears fuel progress");
    assert_eq!(temp(&sim, 3, 7), TEMPERATURE_REFERENCE);
    assert_eq!(count_material(&sim, MATERIAL_OIL), 0);
}

#[test]
fn edit_replacement_clears_fuel_progress() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 85.0);
    set_flags(&sim, 3, 3, with_fuel_progress(FLAG_COMBUSTING, 300));
    seal_eight(&sim, 3, 3);

    // Identity replacement resets the full Matter-owned flags word.
    set(&sim, 3, 3, MATERIAL_STONE);
    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_STONE);
    assert_eq!(fuel_progress(flags(&sim, 3, 3)), 0);
    assert_eq!(flags(&sim, 3, 3) & FLAG_COMBUSTING, 0);
}

#[test]
fn nonflammable_stale_progress_is_removed() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_STONE);
    set_t(&sim, 3, 3, 0.0);
    // Even a deliberately stale fuel-progress field cannot survive on a
    // non-combustible Matter.
    set_flags(
        &sim,
        3,
        3,
        FLAG_COMBUSTING | FLAG_FLAME_EVENT | with_fuel_progress(0, 400),
    );
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    assert_eq!(
        flags(&sim, 3, 3) & combustion_flag_mask(),
        0,
        "nonflammable Matter drops stale combustion state including progress"
    );
    assert_eq!(cell(&sim, 3, 3), MATERIAL_STONE);
}

#[test]
fn reserved_flags_are_cleared_during_progress_updates() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 85.0);
    set_flags(
        &sim,
        3,
        3,
        with_fuel_progress(FLAG_COMBUSTING, 10) | TEST_UNRELATED_FLAG,
    );
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    let f = flags(&sim, 3, 3);
    assert_eq!(fuel_progress(f), 11, "progress advances normally");
    assert_eq!(f & TEST_UNRELATED_FLAG, 0, "reserved bit is cleared");
    assert_ne!(f & FLAG_COMBUSTING, 0);
}

#[test]
fn smoke_generation_stops_after_fuel_is_consumed() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 85.0);
    set_flags(&sim, 3, 3, with_fuel_progress(FLAG_COMBUSTING, 898));
    set(&sim, 2, 3, MATERIAL_STONE);
    set(&sim, 4, 3, MATERIAL_STONE);
    set(&sim, 3, 4, MATERIAL_STONE);
    // (3,2) stays EMPTY → Smoke spawns above on the still-burning tick.

    sim.tick().expect("tick"); // progress 899: burns + spawns Smoke
    assert_eq!(count_material(&sim, MATERIAL_SMOKE), 1);

    sim.tick().expect("tick"); // progress 900: consumed → no new spawn

    assert_eq!(cell(&sim, 3, 3), MATERIAL_EMPTY, "fuel consumed");
    assert_eq!(
        count_material(&sim, MATERIAL_SMOKE),
        1,
        "no new Smoke after the source is consumed"
    );
    assert_eq!(flags(&sim, 3, 3), 0);
}

#[test]
fn fuel_consumption_does_not_delete_neighbor_matter() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 85.0);
    set_flags(&sim, 3, 3, with_fuel_progress(FLAG_COMBUSTING, 899));
    seal_eight(&sim, 3, 3); // 8 Stone neighbors
    assert_eq!(count_material(&sim, MATERIAL_STONE), 8);

    sim.tick().expect("tick"); // consumption tick

    assert_eq!(cell(&sim, 3, 3), MATERIAL_EMPTY);
    assert_eq!(
        count_material(&sim, MATERIAL_STONE),
        8,
        "consuming a fuel cell never deletes neighbor Matter"
    );
    assert_eq!(count_material(&sim, MATERIAL_WOOD), 0);
}

#[test]
fn wood_ignition_front_propagates_then_leaves_empty_cells() {
    // 16×16: a 7-cell Wood strip, hot Stone only at the left end. The
    // ignition front travels along the strip; the earliest cells are
    // consumed to EMPTY while later cells are still burning.
    let mut sim = make_sim(WorldConfig::new(16, 16, 8).unwrap());
    for x in 3..=9 {
        set(&sim, x, 8, MATERIAL_WOOD);
    }
    for sx in 1..=2 {
        for sy in 7..=9 {
            set(&sim, sx, sy, MATERIAL_STONE);
            set_t(&sim, sx, sy, 400.0);
        }
    }

    // Phase 1: the front advances to the far end of the strip.
    let front = tick_until(&mut sim, 2000, 25, |s| {
        (3..=9).any(|x| cell(s, x, 8) == MATERIAL_WOOD && flags(s, x, 8) & FLAG_COMBUSTING != 0)
            && cell(s, 9, 8) != MATERIAL_EMPTY
    });
    assert!(
        front.is_some(),
        "ignition front must propagate along the strip"
    );

    // Phase 2: with continued burning, the earliest cell is consumed (no longer Wood).
    let consumed = tick_until(&mut sim, 3000, 50, |s| {
        cell(s, 3, 8) != MATERIAL_WOOD && count_material(s, MATERIAL_WOOD) > 0
    });
    assert!(
        consumed.is_some(),
        "earliest Wood cells must eventually burn away while the front continues"
    );

    // Invariants after the long run: finite temperatures, no stale
    // combustion state on EMPTY.
    let mats = sim
        .world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("material readback");
    let temps = sim
        .world
        .read_temperature_all(&sim.context.device, &sim.context.queue)
        .expect("temperature readback");
    let fls = sim
        .world
        .read_flags_all(&sim.context.device, &sim.context.queue)
        .expect("flags readback");
    for i in 0..mats.len() {
        assert!(
            temps[i].is_finite(),
            "cell {i} temperature must stay finite"
        );
        if mats[i] == MATERIAL_EMPTY {
            assert_eq!(fls[i], 0, "EMPTY cell {i} must not keep combustion state");
            assert_eq!(temps[i], TEMPERATURE_REFERENCE);
        }
    }
}

#[test]
fn wood_chain_crosses_chunk_boundary() {
    // 128×16 (chunk 64): the 64-column chunk boundary sits between x=63 and
    // x=64. A Wood chain spans it; hot Stone only on the left end. The
    // ignition front must cross the boundary — chunks are never walls.
    let mut sim = make_sim(WorldConfig::new(128, 16, 64).unwrap());
    for x in 60..=66 {
        set(&sim, x, 6, MATERIAL_WOOD);
    }
    for sx in 55..=59 {
        for sy in 5..=7 {
            set(&sim, sx, sy, MATERIAL_STONE);
            set_t(&sim, sx, sy, 400.0);
        }
    }

    let crossed = tick_until(&mut sim, 2500, 25, |s| {
        (64..=66).any(|x| cell(s, x, 6) == MATERIAL_WOOD && flags(s, x, 6) & FLAG_COMBUSTING != 0)
    });
    assert!(
        crossed.is_some(),
        "ignition front must cross the x=63/64 chunk boundary"
    );

    // The chain keeps burning to the far side and the left end is consumed.
    let consumed = tick_until(&mut sim, 3000, 50, |s| {
        cell(s, 60, 6) == MATERIAL_EMPTY && (64..=66).any(|x| cell(s, x, 6) == MATERIAL_WOOD)
    });
    assert!(
        consumed.is_some(),
        "chain consumes fuel on both sides of the chunk boundary"
    );
}

#[test]
fn long_run_combustion_remains_finite() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 85.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);
    seal_eight(&sim, 3, 3);

    for _ in 0..200 {
        sim.tick().expect("tick");
    }

    let temps = sim
        .world
        .read_temperature_all(&sim.context.device, &sim.context.queue)
        .expect("temperature readback");
    for (i, t) in temps.iter().enumerate() {
        assert!(t.is_finite(), "cell {i} temperature must stay finite");
    }
    let mats = sim
        .world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("material readback");
    let fls = sim
        .world
        .read_flags_all(&sim.context.device, &sim.context.queue)
        .expect("flags readback");
    for i in 0..mats.len() {
        if mats[i] == MATERIAL_EMPTY {
            assert_eq!(fls[i], 0, "EMPTY cell {i} must not keep combustion state");
        }
    }
    // The enclosed Wood is still burning (200 < 900 ticks) with finite heat.
    assert_ne!(flags(&sim, 3, 3) & FLAG_COMBUSTING, 0);
    assert!(temp(&sim, 3, 3).is_finite());
}

// ---------------------------------------------------------------------
// G4-D Smoke Finite Lifetime & Material-Owned Decay Tests
// ---------------------------------------------------------------------

#[test]
fn fresh_smoke_does_not_disappear_immediately() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_SMOKE);
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_SMOKE,
        "smoke remains alive at tick 1"
    );
    assert_eq!(decay_age(flags(&sim, 3, 3)), 1, "age advances to 1");
}

#[test]
fn smoke_survives_until_lifetime_minus_one() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_SMOKE);
    set_flags(&sim, 3, 3, with_decay_age(0, 898));
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick"); // age becomes 899 < 900

    assert_eq!(cell(&sim, 3, 3), MATERIAL_SMOKE, "smoke survives age 899");
    assert_eq!(decay_age(flags(&sim, 3, 3)), 899);
}

#[test]
fn smoke_becomes_empty_at_exact_lifetime() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_SMOKE);
    set_flags(&sim, 3, 3, with_decay_age(0, 899));
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick"); // age reaches 900 -> decay to EMPTY

    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_EMPTY,
        "smoke decays to EMPTY at age 900"
    );
    assert_eq!(flags(&sim, 3, 3), 0, "EMPTY has flags == 0");
    assert_eq!(temp(&sim, 3, 3), TEMPERATURE_REFERENCE);
}

#[test]
fn moving_smoke_carries_age() {
    // Smoke at (3, 5), rises to (3, 4) in movement pass, then age increments in decay pass.
    let mut sim = eight_by_eight();
    set(&sim, 3, 5, MATERIAL_SMOKE);
    set_flags(&sim, 3, 5, with_decay_age(0, 100));
    set(&sim, 2, 5, MATERIAL_STONE);
    set(&sim, 4, 5, MATERIAL_STONE);
    set(&sim, 3, 6, MATERIAL_STONE);
    // (3, 4) is EMPTY above

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 5), MATERIAL_EMPTY, "smoke vacated (3,5)");
    assert_eq!(cell(&sim, 3, 4), MATERIAL_SMOKE, "smoke rose to (3,4)");
    assert_eq!(
        decay_age(flags(&sim, 3, 4)),
        101,
        "age carried and incremented to 101"
    );
}

#[test]
fn density_gas_movement_does_not_reset_age() {
    // Steam (rank 20) above Smoke (rank 30) -> Steam rises, Smoke sinks/swaps in gas density channel.
    let mut sim = eight_by_eight();
    for y in 2..=5 {
        set(&sim, 2, y, MATERIAL_STONE);
        set(&sim, 4, y, MATERIAL_STONE);
    }
    set(&sim, 3, 2, MATERIAL_STONE); // top
    set(&sim, 3, 5, MATERIAL_STONE); // bottom
    set(&sim, 3, 3, MATERIAL_STEAM);
    set(&sim, 3, 4, MATERIAL_SMOKE);
    set_flags(&sim, 3, 4, with_decay_age(0, 250));

    sim.tick().expect("tick");

    // After tick, density/gas swap or rise occurs and Smoke keeps its age.
    let smoke_pos = if cell(&sim, 3, 3) == MATERIAL_SMOKE {
        (3, 3)
    } else if cell(&sim, 3, 4) == MATERIAL_SMOKE {
        (3, 4)
    } else {
        panic!("Smoke must still exist in chamber");
    };
    assert_eq!(
        decay_age(flags(&sim, smoke_pos.0, smoke_pos.1)),
        251,
        "age preserved across gas interaction"
    );
}

#[test]
fn void_exit_clears_smoke_age() {
    // Smoke moving through open boundary to Void:
    let config = WorldConfig::new(8, 8, 8).unwrap();
    let mut sim = make_sim(config);
    // Open top boundary at (3, 0)
    set(&sim, 3, 0, MATERIAL_EMPTY);
    set(&sim, 3, 1, MATERIAL_SMOKE);
    set_flags(&sim, 3, 1, with_decay_age(0, 500));

    sim.tick().expect("tick");

    // Smoke rises out of domain to Void; (3,1) is EMPTY with flags 0
    assert_eq!(cell(&sim, 3, 1), MATERIAL_EMPTY);
    assert_eq!(flags(&sim, 3, 1), 0);
}

#[test]
fn stale_smoke_age_cleared_on_non_smoke() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_STONE);
    set_flags(&sim, 3, 3, with_decay_age(0, 777));
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_STONE);
    assert_eq!(
        decay_age(flags(&sim, 3, 3)),
        0,
        "non-decay matter cleans stale decay bits"
    );
}

#[test]
fn smoke_spawn_starts_with_zero_age() {
    // Burning wood spawns fresh Smoke
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 95.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);
    set(&sim, 2, 3, MATERIAL_STONE);
    set(&sim, 4, 3, MATERIAL_STONE);
    set(&sim, 3, 4, MATERIAL_STONE);
    // (3, 2) is EMPTY above

    sim.tick().expect("tick"); // Wood burns and spawns Smoke at (3,2) in smoke_commit

    assert_eq!(cell(&sim, 3, 2), MATERIAL_SMOKE, "smoke spawned");
    assert_eq!(
        decay_age(flags(&sim, 3, 2)),
        0,
        "newly spawned smoke has age 0"
    );
}

#[test]
fn combustion_can_spawn_smoke_that_later_dissipates() {
    let mut sim = eight_by_eight();
    // Seal the sides and bottom. (3,1) remains EMPTY for the first tick as
    // the mandatory orthogonal Air receiver for the Smoke target at (3,2).
    for y in 1..=4 {
        set(&sim, 2, y, MATERIAL_STONE);
        set(&sim, 4, y, MATERIAL_STONE);
    }
    set(&sim, 3, 4, MATERIAL_STONE);
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 95.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING);

    sim.tick().expect("tick 1: smoke spawns at (3,2)");
    assert_eq!(cell(&sim, 3, 2), MATERIAL_SMOKE);

    // Close the receiver after the transaction so this fixture can isolate
    // the spawned Smoke lifetime without allowing upward movement.
    set(&sim, 3, 1, MATERIAL_STONE);

    // Put near expiration age on the spawned smoke
    set_flags(&sim, 3, 2, with_decay_age(flags(&sim, 3, 2), 898));
    // Extinguish the wood so no more smoke spawns
    set_t(&sim, 3, 3, 0.0);
    set_flags(&sim, 3, 3, 0);

    sim.tick().expect("tick 2: age 899");
    assert_eq!(cell(&sim, 3, 2), MATERIAL_SMOKE);

    sim.tick().expect("tick 3: age 900 -> EMPTY");
    assert_eq!(
        cell(&sim, 3, 2),
        MATERIAL_EMPTY,
        "spawned smoke decayed to EMPTY"
    );
    assert_eq!(flags(&sim, 3, 2), 0);
}

#[test]
fn sealed_chamber_smoke_eventually_decreases_after_fire_stops() {
    let mut sim = eight_by_eight();
    // Enclosed 2-cell chamber: (3,3) and (3,2)
    for y in 1..=4 {
        set(&sim, 2, y, MATERIAL_STONE);
        set(&sim, 4, y, MATERIAL_STONE);
    }
    set(&sim, 3, 1, MATERIAL_STONE); // ceiling
    set(&sim, 3, 4, MATERIAL_STONE); // floor
    set(&sim, 3, 3, MATERIAL_SMOKE);
    set(&sim, 3, 2, MATERIAL_SMOKE);
    set_flags(&sim, 3, 3, with_decay_age(0, 850));
    set_flags(&sim, 3, 2, with_decay_age(0, 890));

    assert_eq!(count_material(&sim, MATERIAL_SMOKE), 2);

    // Run 15 ticks: (3,2) reaches 905 (>900) -> decays to EMPTY; (3,3) reaches 865 (<900) -> still alive
    for _ in 0..15 {
        sim.tick().expect("tick");
    }

    assert_eq!(
        count_material(&sim, MATERIAL_SMOKE),
        1,
        "older smoke decayed, younger remains"
    );

    // Run 40 more ticks: (3,3) reaches 905 (>900) -> also decays to EMPTY
    for _ in 0..40 {
        sim.tick().expect("tick");
    }

    assert_eq!(
        count_material(&sim, MATERIAL_SMOKE),
        0,
        "all smoke in sealed room decayed to EMPTY"
    );
}

#[test]
fn smoke_decay_does_not_delete_neighbor_matter() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_SMOKE);
    set_flags(&sim, 3, 3, with_decay_age(0, 899));
    seal_eight(&sim, 3, 3); // 8 Stone neighbors surrounding (3,3)
    assert_eq!(count_material(&sim, MATERIAL_STONE), 8);

    sim.tick().expect("decay tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_EMPTY);
    assert_eq!(
        count_material(&sim, MATERIAL_STONE),
        8,
        "8 stone neighbors untouched"
    );
}

#[test]
fn long_run_smoke_has_no_stale_flags_on_empty() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_SMOKE);
    set_flags(&sim, 3, 3, with_decay_age(0, 800));
    seal_eight(&sim, 3, 3);

    for _ in 0..150 {
        sim.tick().expect("tick");
    }

    assert_eq!(cell(&sim, 3, 3), MATERIAL_EMPTY);
    let flags_all = sim
        .world
        .read_flags_all(&sim.context.device, &sim.context.queue)
        .expect("readback");
    for (i, &f) in flags_all.iter().enumerate() {
        assert_eq!(f, 0, "cell {i} flags must be 0 after smoke decay");
    }
}

#[test]
fn smoke_age_crosses_chunk_boundary_with_identity() {
    // 128x128 world, chunk size 64. Smoke at x=63, rising or moving to x=64.
    let config = WorldConfig::new(128, 128, 64).unwrap();
    let mut sim = make_sim(config);
    // Seal below and sides, open above across chunk boundary
    set(&sim, 63, 10, MATERIAL_SMOKE);
    set_flags(&sim, 63, 10, with_decay_age(0, 300));
    set(&sim, 63, 11, MATERIAL_STONE);
    set(&sim, 62, 10, MATERIAL_STONE);
    set(&sim, 64, 10, MATERIAL_STONE);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 63, 10), MATERIAL_EMPTY);
    assert_eq!(cell(&sim, 63, 9), MATERIAL_SMOKE);
    assert_eq!(
        decay_age(flags(&sim, 63, 9)),
        301,
        "age preserved across movement near chunk boundary"
    );
}

#[test]
fn decayed_smoke_not_resurrected_by_subsequent_passes() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_SMOKE);
    set_flags(&sim, 3, 3, with_decay_age(0, 899));
    seal_eight(&sim, 3, 3);

    // On this tick: movement (blocked) -> thermal -> phase -> decay (transforms to EMPTY) -> combustion/smoke_commit
    sim.tick().expect("decay tick");

    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_EMPTY,
        "smoke decayed to EMPTY in decay pass"
    );
    assert_eq!(flags(&sim, 3, 3), 0);
    assert_eq!(temp(&sim, 3, 3), TEMPERATURE_REFERENCE);

    // Run several more ticks with no nearby burning source: remains EMPTY
    for _ in 0..10 {
        sim.tick().expect("subsequent tick");
        assert_eq!(cell(&sim, 3, 3), MATERIAL_EMPTY);
        assert_eq!(flags(&sim, 3, 3), 0);
    }
}

#[test]
fn reserved_flag_bits_are_cleared_by_decay_hygiene() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_SMOKE);
    // Combine decay age with TEST_UNRELATED_FLAG (bit 28)
    set_flags(&sim, 3, 3, with_decay_age(TEST_UNRELATED_FLAG, 100));
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_SMOKE);
    let f = flags(&sim, 3, 3);
    assert_eq!(decay_age(f), 101, "age incremented");
    assert_eq!(f & TEST_UNRELATED_FLAG, 0, "reserved flag bit is cleared");
}

#[test]
fn every_registered_m0_material_obeys_exact_flag_ownership() {
    let mut sim = make_sim(WorldConfig::new(32, 20, 4).unwrap());
    let reserved = 1u32 << 28;
    let centers = [
        (3, 3),
        (9, 3),
        (15, 3),
        (21, 3),
        (27, 3),
        (3, 11),
        (9, 11),
        (15, 11),
        (21, 11),
        (27, 11),
    ];
    assert_eq!(MATERIAL_REGISTRY.len() + 1, centers.len());

    for (material, &(x, y)) in (0u32..=9).zip(&centers) {
        seal_eight(&sim, x, y);
        set(&sim, x, y, material);
        let staged_flags = match material {
            MATERIAL_OIL | MATERIAL_WOOD => with_fuel_progress(FLAG_COMBUSTING | reserved, 10),
            MATERIAL_SMOKE => with_decay_age(reserved, 10),
            _ => u32::MAX,
        };
        set_flags(&sim, x, y, staged_flags);
        if material == MATERIAL_OIL {
            set_t(&sim, x, y, 80.0);
        } else if material == MATERIAL_WOOD {
            set_t(&sim, x, y, 85.0);
        } else if material == MATERIAL_STEAM {
            set_t(&sim, x, y, 80.0);
        }
    }

    sim.tick().expect("tick");

    for &(x, y) in &centers {
        let material = cell(&sim, x, y);
        let actual = flags(&sim, x, y);
        let allowed = match material {
            MATERIAL_OIL | MATERIAL_WOOD => combustion_flag_mask(),
            MATERIAL_SMOKE => decay_flag_mask(),
            _ => 0,
        };
        assert_eq!(
            actual & !allowed,
            0,
            "Material {material} retained flags outside ownership mask 0x{allowed:08x}"
        );
    }
}
