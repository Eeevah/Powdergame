//! G4-B — Phase transition: GPU semantic/invariant tests.
//!
//! Runs on the actual machine (Windows + RTX 5090 + DX12). Ice ↔ Water ↔
//! Steam are temperature-based **self transitions** (`REACTION_SPEC` §3):
//! the phase pass reads this cell's Material + Temperature and writes only
//! `material_next[self]`. No Claim/Resolve, no neighbor writes.
//!
//! Tick causal order (per tick): movement (Matter carries its Temperature
//! on the ownership edge) → thermal conduction → phase transition. So the
//! integrated tests prove that the Temperature field and the phase rules are
//! actually connected, not two separate features.
//!
//! Thresholds (relative gameplay scalar, not Celsius), with hysteresis:
//!   Water freezes below -20, Ice melts above -10,
//!   Steam condenses below 40, Water boils above 60.
//! Temperature is preserved across the source transform (latent heat is out
//! of scope). G5-B extends boiling with a data-driven extra Steam request;
//! sealed fixtures still isolate the original phase identity contract.
//!
//! The "next tick" tests prove the phase is NOT a pure ID repaint: a
//! phase-changed Matter adopts the new Material descriptor's MovementClass
//! on the following tick (melted Ice falls as Water, boiled Water rises as
//! Steam).

use powdergame_core::{
    WorldConfig, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_OIL, MATERIAL_SAND, MATERIAL_SMOKE,
    MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, TEMPERATURE_REFERENCE,
};
use powdergame_gpu::Simulation;

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

fn temp(sim: &Simulation, x: i64, y: i64) -> f32 {
    sim.world
        .read_temperature_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("temperature readback")
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

/// Seals `(x, y)` with a 3×3 Stone ring so NO stencil candidate is
/// available — used to isolate a cell from movement during phase tests.
fn box_seal(sim: &Simulation, x: i64, y: i64) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx != 0 || dy != 0 {
                set(sim, x + dx, y + dy, MATERIAL_STONE);
            }
        }
    }
}

/// `box_seal` where every ring Stone is staged at temperature `t`, so the
/// sealed cell is also thermally isolated (no conduction drift).
fn box_seal_at(sim: &Simulation, x: i64, y: i64, t: f32) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx != 0 || dy != 0 {
                set(sim, x + dx, y + dy, MATERIAL_STONE);
                set_t(sim, x + dx, y + dy, t);
            }
        }
    }
}

// ── Direct phase transitions (one tick) ────────────────────────────────

#[test]
fn water_freezes_to_ice() {
    let mut sim = eight_by_eight();
    box_seal(&sim, 3, 3);
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, -30.0);
    let before = matter_count(&sim);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_ICE, "water froze to ice");
    assert_eq!(count_material(&sim, MATERIAL_ICE), 1, "exactly one ice");
    assert_eq!(count_material(&sim, MATERIAL_WATER), 0, "water consumed");
    assert_eq!(matter_count(&sim), before, "1:1 transform conserves Matter");
    let t = temp(&sim, 3, 3);
    assert!(t.is_finite(), "ice temperature finite");
    assert!(t < -20.0, "ice stays below the melt threshold; got {t}");
}

#[test]
fn ice_melts_to_water() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_ICE); // isolated: no conduction
    set_t(&sim, 3, 3, -5.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_WATER, "ice melted to water");
    assert_eq!(count_material(&sim, MATERIAL_WATER), 1);
    assert_eq!(count_material(&sim, MATERIAL_ICE), 0);
    let t = temp(&sim, 3, 3);
    assert!(
        (t - (-5.0)).abs() < 1.0e-3,
        "temperature preserved across melt; got {t}"
    );
}

#[test]
fn water_boils_to_steam() {
    let mut sim = eight_by_eight();
    box_seal(&sim, 3, 3);
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 70.0);
    let before = matter_count(&sim);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_STEAM, "water boiled to steam");
    assert_eq!(count_material(&sim, MATERIAL_STEAM), 1, "exactly one steam");
    assert_eq!(count_material(&sim, MATERIAL_WATER), 0, "water consumed");
    assert_eq!(matter_count(&sim), before, "1:1 transform conserves Matter");
    let t = temp(&sim, 3, 3);
    assert!(t.is_finite());
    assert!(t > 60.0, "steam keeps its heat; got {t}");
}

#[test]
fn steam_condenses_to_water() {
    let mut sim = eight_by_eight();
    box_seal(&sim, 3, 3);
    set(&sim, 3, 3, MATERIAL_STEAM);
    set_t(&sim, 3, 3, 30.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_WATER, "steam condensed to water");
    assert_eq!(count_material(&sim, MATERIAL_WATER), 1);
    assert_eq!(count_material(&sim, MATERIAL_STEAM), 0);
}

// ── Stability / hysteresis ──────────────────────────────────────────────

#[test]
fn neutral_water_is_stable() {
    let mut sim = eight_by_eight();
    box_seal_at(&sim, 3, 3, 0.0);
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 0.0);

    for _ in 0..5 {
        sim.tick().expect("tick");
    }

    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_WATER,
        "neutral water stays water"
    );
    assert_eq!(count_material(&sim, MATERIAL_WATER), 1);
}

#[test]
fn hysteresis_prevents_ping_pong() {
    // Water at -15: inside the freeze(-20)/melt(-10) band → stays Water.
    let mut sim = eight_by_eight();
    box_seal_at(&sim, 3, 3, -15.0);
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, -15.0);
    for _ in 0..5 {
        sim.tick().expect("tick");
    }
    assert_eq!(cell(&sim, 3, 3), MATERIAL_WATER, "water -15 stays water");

    // Ice at -15: inside the band → stays Ice.
    let mut sim = eight_by_eight();
    set(&sim, 5, 3, MATERIAL_ICE); // isolated: no conduction drift
    set_t(&sim, 5, 3, -15.0);
    for _ in 0..5 {
        sim.tick().expect("tick");
    }
    assert_eq!(cell(&sim, 5, 3), MATERIAL_ICE, "ice -15 stays ice");

    // Water at +50: inside the condense(40)/boil(60) band → stays Water.
    let mut sim = eight_by_eight();
    box_seal_at(&sim, 3, 3, 50.0);
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 50.0);
    for _ in 0..5 {
        sim.tick().expect("tick");
    }
    assert_eq!(cell(&sim, 3, 3), MATERIAL_WATER, "water +50 stays water");

    // Steam at +50: inside the band → stays Steam.
    let mut sim = eight_by_eight();
    box_seal_at(&sim, 3, 3, 50.0);
    set(&sim, 3, 3, MATERIAL_STEAM);
    set_t(&sim, 3, 3, 50.0);
    for _ in 0..5 {
        sim.tick().expect("tick");
    }
    assert_eq!(cell(&sim, 3, 3), MATERIAL_STEAM, "steam +50 stays steam");
}

#[test]
fn non_phase_materials_never_transform_on_gpu() {
    let mut sim = eight_by_eight();
    // Extreme temperatures, sealed so nothing moves: phase rules are the
    // only thing that could change these materials — and they have none.
    set(&sim, 1, 6, MATERIAL_SAND);
    set_t(&sim, 1, 6, 500.0);
    set(&sim, 2, 6, MATERIAL_OIL);
    set_t(&sim, 2, 6, -500.0);
    set(&sim, 3, 5, MATERIAL_STONE);
    set(&sim, 2, 5, MATERIAL_STONE);
    set(&sim, 4, 5, MATERIAL_STONE);
    set(&sim, 3, 6, MATERIAL_SMOKE);
    set_t(&sim, 3, 6, 500.0);
    set(&sim, 4, 6, MATERIAL_STONE);
    set_t(&sim, 4, 6, -500.0);

    for _ in 0..3 {
        sim.tick().expect("tick");
    }

    assert_eq!(cell(&sim, 1, 6), MATERIAL_SAND, "sand has no phase rule");
    assert_eq!(cell(&sim, 2, 6), MATERIAL_OIL, "oil has no phase rule");
    assert_eq!(cell(&sim, 3, 6), MATERIAL_SMOKE, "smoke has no phase rule");
    assert_eq!(cell(&sim, 4, 6), MATERIAL_STONE, "stone has no phase rule");
}

// ── Temperature preservation ────────────────────────────────────────────

#[test]
fn phase_preserves_temperature() {
    // Sealed Water T=70 boils to Steam in one tick. The transform must NOT
    // reset the temperature to the reference: it keeps a hot value
    // (conduction with the 0 K seal stones cools it only a little).
    let mut sim = eight_by_eight();
    box_seal_at(&sim, 3, 3, 0.0);
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 70.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_STEAM);
    let t = temp(&sim, 3, 3);
    assert!(
        t > 60.0,
        "temperature must survive the 1:1 transform; got {t}"
    );
    assert!(
        t <= 70.0,
        "conduction may cool a little but must never raise the source"
    );
}

// ── Integrated causal chains ────────────────────────────────────────────

#[test]
fn thermal_heating_triggers_boiling() {
    // Sealed Water T=0 next to a very hot Stone reservoir. Over ticks the
    // temperature field (never a CPU material edit) pushes the Water past
    // the boil threshold → Steam appears by itself.
    let mut sim = eight_by_eight();
    box_seal(&sim, 3, 3);
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 0.0);
    // Hot reservoir: the left seal stones.
    set_t(&sim, 2, 2, 400.0);
    set_t(&sim, 2, 3, 400.0);
    set_t(&sim, 2, 4, 400.0);

    for _ in 0..30 {
        sim.tick().expect("tick");
    }

    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_STEAM,
        "thermal propagation must trigger boiling by itself"
    );
    assert_eq!(count_material(&sim, MATERIAL_STEAM), 1);
    let t = temp(&sim, 3, 3);
    assert!(
        t > 40.0,
        "steam must stay warm enough to remain steam; got {t}"
    );
}

#[test]
fn thermal_cooling_triggers_freezing() {
    // Sealed Water T=0 next to a very cold Stone reservoir → freezes to Ice.
    let mut sim = eight_by_eight();
    box_seal(&sim, 3, 3);
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 0.0);
    set_t(&sim, 2, 2, -100.0);
    set_t(&sim, 2, 3, -100.0);
    set_t(&sim, 2, 4, -100.0);

    for _ in 0..40 {
        sim.tick().expect("tick");
    }

    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_ICE,
        "thermal propagation must trigger freezing by itself"
    );
    assert_eq!(count_material(&sim, MATERIAL_ICE), 1);
    let t = temp(&sim, 3, 3);
    assert!(t < -20.0, "ice must stay cold; got {t}");
}

#[test]
fn hot_water_moves_then_boils_at_destination() {
    // Full causal chain in ONE tick:
    //   movement ownership carries hot Water down one cell
    //   → (isolated, no conduction) temperature stays 80
    //   → phase turns it into Steam at the NEW location.
    // The old cell ends EMPTY / T=0.
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 80.0);
    let before = matter_count(&sim);

    sim.tick().expect("tick");

    assert_eq!(
        cell(&sim, 3, 4),
        MATERIAL_STEAM,
        "water moved down, then boiled at the destination"
    );
    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_STEAM,
        "G5-B expansion reuses the newly vacated source cell"
    );
    assert_eq!(count_material(&sim, MATERIAL_STEAM), 2);
    assert_eq!(count_material(&sim, MATERIAL_WATER), 0);
    assert_eq!(matter_count(&sim), before + 1);
    let dest_t = temp(&sim, 3, 4);
    let spawn_t = temp(&sim, 3, 3);
    assert!(
        (dest_t - 80.0).abs() < 1.0e-3,
        "the hot state must be carried to the new cell; got {dest_t}"
    );
    assert!((spawn_t - dest_t).abs() < 1.0e-3);
}

// ── Ice movement semantics ──────────────────────────────────────────────

#[test]
fn ice_is_static_and_never_density_swaps() {
    // Ice above Water on the sealed floor: Ice is STATIC with no density
    // rank, so it neither falls nor swaps. Cold staging keeps it Ice.
    let mut sim = eight_by_eight();
    set(&sim, 2, 6, MATERIAL_STONE); // seal the water cell below
    set(&sim, 4, 6, MATERIAL_STONE);
    set(&sim, 3, 6, MATERIAL_WATER);
    set(&sim, 3, 5, MATERIAL_ICE);
    set_t(&sim, 3, 5, -30.0);

    for _ in 0..5 {
        sim.tick().expect("tick");
    }

    assert_eq!(cell(&sim, 3, 5), MATERIAL_ICE, "ice never moves");
    assert_eq!(cell(&sim, 3, 6), MATERIAL_WATER, "water below untouched");
    assert_eq!(count_material(&sim, MATERIAL_ICE), 1);
    let t = temp(&sim, 3, 5);
    assert!(t < -10.0, "ice stays below the melt threshold; got {t}");
}

// ── Phase → MovementClass adoption on the next tick ─────────────────────

#[test]
fn melted_ice_uses_water_movement_next_tick() {
    // Ice T=-5 with EMPTY below. Tick 1: phase runs AFTER movement, so the
    // STATIC Ice melts in place to Water (it must not move while Ice).
    // Tick 2: the new Water identity uses the LIQUID MovementClass and
    // falls one cell, carrying its temperature.
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_ICE);
    set_t(&sim, 3, 3, -5.0);
    let before = matter_count(&sim);

    // Tick 1: melt in place (Ice is STATIC before the phase pass).
    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_WATER, "ice melted in place");
    assert_eq!(count_material(&sim, MATERIAL_ICE), 0);
    assert_eq!(count_material(&sim, MATERIAL_WATER), 1);
    let melt_t = temp(&sim, 3, 3);
    assert!(
        (melt_t - (-5.0)).abs() < 1.0e-3,
        "melted water keeps the temperature; got {melt_t}"
    );

    // Tick 2: the LIQUID identity actually moves down.
    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_EMPTY, "source vacated on tick 2");
    assert_eq!(
        temp(&sim, 3, 3),
        TEMPERATURE_REFERENCE,
        "no ghost heat at the source"
    );
    assert_eq!(
        cell(&sim, 3, 4),
        MATERIAL_WATER,
        "melted water uses LIQUID movement the next tick"
    );
    let dest_t = temp(&sim, 3, 4);
    assert!(
        (dest_t - (-5.0)).abs() < 1.0e-3,
        "temperature travels with the moved water; got {dest_t}"
    );
    assert_eq!(count_material(&sim, MATERIAL_WATER), 1);
    assert_eq!(
        matter_count(&sim),
        before,
        "matter conserved across both ticks"
    );
}

#[test]
fn boiled_water_uses_steam_movement_next_tick() {
    // Water T=80, up EMPTY, but down / down-diagonals / laterals blocked by
    // a Stone fixture so it cannot slide away on tick 1. Tick 1: Water
    // boils in place to Steam. Tick 2: the Steam identity uses the GAS
    // MovementClass and rises one cell, carrying its heat.
    let mut sim = eight_by_eight();
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 80.0);
    // Seal all eight neighbors on tick 1 so G5-B expansion is deliberately
    // confined; after boiling we open only (3,2) to test GAS movement.
    box_seal(&sim, 3, 3);
    let before = matter_count(&sim);

    // Tick 1: water stays put (all LIQUID stencil candidates blocked), then
    // boils in place to Steam.
    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_STEAM, "water boiled in place");
    assert_eq!(count_material(&sim, MATERIAL_STEAM), 1);
    assert_eq!(count_material(&sim, MATERIAL_WATER), 0);
    let boil_t = temp(&sim, 3, 3);
    assert!(
        boil_t > 60.0,
        "steam keeps its heat after boiling; got {boil_t}"
    );

    // Open one cell only after the blocked boiling tick; this keeps the
    // historical MovementClass adoption test independent of G5-B spawn.
    set(&sim, 3, 2, MATERIAL_EMPTY);
    let after_open = matter_count(&sim);

    // Tick 2: the GAS identity actually rises one cell.
    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_EMPTY, "source vacated on tick 2");
    assert_eq!(
        temp(&sim, 3, 3),
        TEMPERATURE_REFERENCE,
        "no ghost heat at the source"
    );
    assert_eq!(
        cell(&sim, 3, 2),
        MATERIAL_STEAM,
        "boiled steam uses GAS movement the next tick"
    );
    let dest_t = temp(&sim, 3, 2);
    assert!(
        dest_t > 40.0,
        "hot temperature travels with the risen steam; got {dest_t}"
    );
    assert_eq!(count_material(&sim, MATERIAL_STEAM), 1);
    assert_eq!(
        matter_count(&sim),
        after_open,
        "opening the fixture changes the authored Stone count, but movement conserves Matter"
    );
    assert_eq!(after_open + 1, before);
}

// ── Chunk boundary ──────────────────────────────────────────────────────

#[test]
fn phase_transition_crosses_chunk_boundary() {
    // 128×16: the 64-column chunk boundary sits between x=63 and x=64.
    // A cold Stone reservoir at x=63 pulls Water at x=64 below the freeze
    // threshold across the chunk edge → Ice appears on the far side.
    // Chunks are not phase walls.
    let mut sim = make_sim(WorldConfig::new(128, 16, 64).unwrap());
    // Cold reservoir (2 cells) + a cold cell below so the reservoir does
    // not drain into a warm seal stone.
    set(&sim, 63, 8, MATERIAL_STONE);
    set_t(&sim, 63, 8, -300.0);
    set(&sim, 62, 8, MATERIAL_STONE);
    set_t(&sim, 62, 8, -300.0);
    set(&sim, 63, 9, MATERIAL_STONE);
    set_t(&sim, 63, 9, -300.0);
    // Water on the far side of the chunk boundary, sealed on ALL stencil
    // candidates (orthogonal AND down-diagonals) so it cannot slide away
    // before freezing.
    set(&sim, 64, 8, MATERIAL_WATER);
    set_t(&sim, 64, 8, 0.0);
    set(&sim, 65, 8, MATERIAL_STONE);
    set(&sim, 64, 7, MATERIAL_STONE);
    set(&sim, 64, 9, MATERIAL_STONE);
    set(&sim, 65, 9, MATERIAL_STONE);

    for _ in 0..40 {
        sim.tick().expect("tick");
    }

    assert_eq!(
        cell(&sim, 64, 8),
        MATERIAL_ICE,
        "water froze into ice across the chunk boundary"
    );
    assert_eq!(count_material(&sim, MATERIAL_ICE), 1);
}

// ── Pipeline executes on the GPU ────────────────────────────────────────

#[test]
fn phase_pipeline_executes_on_gpu() {
    let mut sim = eight_by_eight();
    box_seal(&sim, 3, 3);
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, -30.0);
    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_ICE);
    assert_eq!(
        sim.read_marker().expect("marker"),
        1,
        "the tick dispatch (incl. phase pass) executed on the GPU"
    );
}
