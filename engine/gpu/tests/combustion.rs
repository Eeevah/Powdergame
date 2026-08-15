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
    WorldConfig, FLAG_COMBUSTING, FLAG_FLAME_EVENT, MATERIAL_EMPTY, MATERIAL_OIL, MATERIAL_SMOKE,
    MATERIAL_STONE, MATERIAL_WOOD, TEMPERATURE_REFERENCE,
};
use powdergame_gpu::Simulation;

/// A future-subsystem flag bit that combustion must never touch.
const TEST_UNRELATED_FLAG: u32 = 1 << 10;

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
fn combustion_flag_bits_are_preserved() {
    let unrelated = 1u32 << 8;
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_OIL);
    set_t(&sim, 3, 3, 80.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING | unrelated);
    seal_eight(&sim, 3, 3);

    sim.tick().expect("tick");

    let f = flags(&sim, 3, 3);
    assert_ne!(f & unrelated, 0, "unrelated future flag bits survive");
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
    set(&sim, 2, 4, MATERIAL_STONE);
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
    set(&sim, 2, 4, MATERIAL_STONE);
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
    assert_eq!(flags(&sim, 3, 4), 0, "Smoke never inherits Oil's flags");
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
fn unrelated_flag_bit_survives_combustion() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WOOD);
    set_t(&sim, 3, 3, 85.0);
    set_flags(&sim, 3, 3, FLAG_COMBUSTING | TEST_UNRELATED_FLAG);
    set(&sim, 2, 3, MATERIAL_STONE);
    set(&sim, 4, 3, MATERIAL_STONE);
    set(&sim, 3, 4, MATERIAL_STONE);

    sim.tick().expect("tick");

    let f = flags(&sim, 3, 3);
    assert_ne!(f & TEST_UNRELATED_FLAG, 0, "unrelated bit is preserved");
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
    assert_ne!(f & TEST_UNRELATED_FLAG, 0, "unrelated bit survives");
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
