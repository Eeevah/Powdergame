//! G7-A — Chunk Activity measurement baseline tests.
//!
//! Verifies on the authoritative GPU path (Windows + RTX 5090 + DX12) that
//! the per-chunk activity detector reflects the changeable frontier — not
//! Matter existence — and that the stable-duration counter behaves as an
//! observation baseline. No work is skipped yet (G7-B); these tests pin the
//! measurement semantics.
//!
//! Bit semantics (engine/core/src/activity.rs):
//!   ACTIVITY_MATTER    = 1 << 0   movement / density frontier exists
//!   ACTIVITY_THERMAL   = 1 << 1   temperature gradient, heat source, phase
//!                                 rule satisfied, or a phase transition
//!                                 actually fired this tick
//!   ACTIVITY_PRESSURE  = 1 << 2   pressure gradient (pressure media only)
//!   ACTIVITY_REACTION  = 1 << 3   combustion / decay state progressing

use powdergame_core::{
    WorldConfig, ACTIVITY_MATTER, ACTIVITY_PRESSURE, ACTIVITY_REACTION, ACTIVITY_THERMAL,
    FLAG_COMBUSTING, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_SAND,
    MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD,
};
use powdergame_gpu::Simulation;

fn make_sim(config: WorldConfig) -> Simulation {
    pollster::block_on(Simulation::new(config)).expect("DX12 + RTX 5090 simulation init")
}

fn set(sim: &Simulation, x: i64, y: i64, id: u32) {
    sim.world
        .write_material(&sim.context.queue, x, y, id)
        .expect("material edit");
}

fn set_t(sim: &Simulation, x: i64, y: i64, t: f32) {
    sim.world
        .write_temperature(&sim.context.queue, x, y, t)
        .expect("temperature edit");
}

fn set_p(sim: &Simulation, x: i64, y: i64, p: f32) {
    sim.world
        .write_pressure(&sim.context.queue, x, y, p)
        .expect("pressure edit");
}

fn chunk_activity(sim: &Simulation) -> Vec<u32> {
    sim.world
        .read_chunk_activity_all(&sim.context.device, &sim.context.queue)
        .expect("chunk activity readback")
}

fn chunk_changed(sim: &Simulation) -> Vec<u32> {
    sim.world
        .read_chunk_changed_all(&sim.context.device, &sim.context.queue)
        .expect("chunk changed readback")
}

fn chunk_stable(sim: &Simulation) -> Vec<u32> {
    sim.world
        .read_chunk_stable_all(&sim.context.device, &sim.context.queue)
        .expect("chunk stable readback")
}

fn count_material(sim: &Simulation, id: u32) -> usize {
    let mats = sim
        .world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("read all materials");
    mats.iter().filter(|&&m| m == id).count()
}

/// Fill a rectangular region (inclusive) with a material.
fn fill_rect(sim: &Simulation, x0: i64, y0: i64, x1: i64, y1: i64, id: u32) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            set(sim, x, y, id);
        }
    }
}

/// Set one temperature on EVERY cell (incl. the boundary ring) so the world
/// is exactly uniform — any activity then cannot come from a gradient.
fn fill_uniform_t(sim: &Simulation, t: f32, w: i64, h: i64) {
    for y in 0..h {
        for x in 0..w {
            set_t(sim, x, y, t);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Baseline: stable / inactive chunks
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn stable_stone_chunk_reports_inactive() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    fill_rect(&sim, 1, 1, 62, 62, MATERIAL_STONE);

    sim.tick().expect("tick 1");
    sim.tick().expect("tick 2");
    sim.tick().expect("tick 3");

    // A chunk of pure STATIC Matter at uniform T/P/0 flags has no frontier.
    assert_eq!(chunk_activity(&sim), vec![0]);
    assert_eq!(chunk_changed(&sim), vec![0]);
    // One stable tick per inactive tick (observation baseline, no cutoff).
    assert_eq!(chunk_stable(&sim), vec![3]);
}

#[test]
fn stable_water_bulk_reports_no_internal_movement_frontier() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());

    // Sealed stone box, completely filled with Water: every Water cell has
    // Water (or STATIC) on every stencil stage — no EMPTY interface, no
    // density inversion. Existence of 1000+ Water cells is NOT activity.
    fill_rect(&sim, 8, 8, 55, 55, MATERIAL_STONE); // box walls (shell)
    fill_rect(&sim, 10, 10, 53, 53, MATERIAL_WATER); // interior water

    sim.tick().expect("tick 1");
    sim.tick().expect("tick 2");
    sim.tick().expect("tick 3");

    assert_eq!(chunk_activity(&sim), vec![0]);
    assert_eq!(chunk_stable(&sim), vec![3]);
    // Nothing was lost or duplicated.
    assert_eq!(count_material(&sim, MATERIAL_WATER), 44 * 44);
}

#[test]
fn same_matter_noop_does_not_create_false_activity() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());

    // A vertical Water stack: identical ranks never swap (G3 equal-rank
    // stability), and occupied cells block movement. No false frontier.
    fill_rect(&sim, 8, 8, 55, 55, MATERIAL_STONE);
    fill_rect(&sim, 31, 20, 33, 45, MATERIAL_WATER);

    sim.tick().expect("tick 1");
    sim.tick().expect("tick 2");

    assert_eq!(chunk_activity(&sim)[0] & ACTIVITY_MATTER, 0);
    assert_eq!(count_material(&sim, MATERIAL_WATER), 3 * 26);
}

// ─────────────────────────────────────────────────────────────────────────────
// Baseline: active frontiers
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn water_empty_interface_reports_matter_active() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());

    // Water pool resting on stone, with EMPTY beside its left column: those
    // Water cells have a real lateral candidate → MATTER frontier.
    fill_rect(&sim, 8, 53, 55, 55, MATERIAL_STONE); // floor
    fill_rect(&sim, 8, 8, 19, 52, MATERIAL_EMPTY); // open left column
    fill_rect(&sim, 20, 40, 43, 52, MATERIAL_WATER); // pool

    sim.tick().expect("tick 1");

    assert_ne!(chunk_activity(&sim)[0] & ACTIVITY_MATTER, 0);
    assert_eq!(chunk_changed(&sim)[0], 1);
    assert_eq!(chunk_stable(&sim)[0], 0);
}

#[test]
fn density_inversion_reports_active() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());

    // Sand above Water: the interface cells are density-swap candidates
    // (150 > 90) → MATTER frontier.
    fill_rect(&sim, 30, 30, 33, 33, MATERIAL_SAND);
    fill_rect(&sim, 30, 34, 33, 37, MATERIAL_WATER);

    sim.tick().expect("tick 1");

    assert_ne!(chunk_activity(&sim)[0] & ACTIVITY_MATTER, 0);
}

#[test]
fn thermal_gradient_reports_thermal_active() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    fill_rect(&sim, 8, 8, 55, 55, MATERIAL_STONE);

    // Hot Stone cell adjacent to cold Stone → 4-neighbor gradient.
    set_t(&sim, 30, 30, 100.0);

    sim.tick().expect("tick 1");

    assert_ne!(chunk_activity(&sim)[0] & ACTIVITY_THERMAL, 0);
}

#[test]
fn pressure_gradient_reports_pressure_active() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());

    // Pressure is a spatial field that exists in Liquid/Gas media only;
    // stage it in a sealed Water body (Stone would clear the field).
    fill_rect(&sim, 8, 8, 55, 55, MATERIAL_STONE);
    fill_rect(&sim, 20, 20, 43, 43, MATERIAL_WATER);
    set_p(&sim, 30, 30, 50.0);

    sim.tick().expect("tick 1");

    assert_ne!(chunk_activity(&sim)[0] & ACTIVITY_PRESSURE, 0);
}

#[test]
fn burning_wood_reports_reaction_active() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());

    // Wood above its ignition threshold (90) with EMPTY surroundings
    // (EMPTY is not a thermal medium, so the fuel keeps its heat).
    fill_rect(&sim, 30, 30, 33, 31, MATERIAL_WOOD);
    for y in 30..=31 {
        for x in 30..=33 {
            set_t(&sim, x, y, 100.0);
        }
    }

    sim.tick().expect("tick 1 (ignition)");

    let mask = chunk_activity(&sim)[0];
    // Combustion is an active reaction state; a burning cell is also a
    // heat source.
    assert_ne!(mask & ACTIVITY_REACTION, 0);
    assert_ne!(mask & ACTIVITY_THERMAL, 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Stable duration semantics
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn stable_duration_increments_only_when_no_meaningful_change() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    fill_rect(&sim, 1, 1, 62, 62, MATERIAL_STONE);

    sim.tick().expect("tick 1");
    assert_eq!(chunk_stable(&sim)[0], 1);
    sim.tick().expect("tick 2");
    assert_eq!(chunk_stable(&sim)[0], 2);
    sim.tick().expect("tick 3");
    assert_eq!(chunk_stable(&sim)[0], 3);
}

#[test]
fn meaningful_change_resets_stable_duration() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    fill_rect(&sim, 1, 1, 62, 62, MATERIAL_STONE);

    sim.tick().expect("tick 1");
    sim.tick().expect("tick 2");
    sim.tick().expect("tick 3");
    assert_eq!(chunk_stable(&sim)[0], 3);

    // A falling Sand column (EMPTY gap below, Stone floor): the chunk keeps
    // a real movement frontier while grains are in flight.
    fill_rect(&sim, 30, 30, 33, 40, MATERIAL_SAND);
    fill_rect(&sim, 30, 41, 33, 54, MATERIAL_EMPTY);

    sim.tick().expect("tick 4 (wake)");

    assert_ne!(chunk_activity(&sim)[0] & ACTIVITY_MATTER, 0);
    assert_eq!(chunk_stable(&sim)[0], 0);
    assert_eq!(chunk_changed(&sim)[0], 1);

    // Still in flight the following tick: activity persists, stable stays 0.
    sim.tick().expect("tick 5");
    assert_ne!(chunk_activity(&sim)[0] & ACTIVITY_MATTER, 0);
    assert_eq!(chunk_stable(&sim)[0], 0);
}

#[test]
fn neighbor_activity_does_not_falsely_wake_adjacent_stable_chunk() {
    // 128×128 → 4 chunks. Chunk 0 (x 0..63) holds a falling Sand column
    // (active); chunk 1 (x 64..127) is pure Stone. Mere adjacency must NOT
    // wake the stable chunk (no neighbor-spillover false wake).
    let mut sim = make_sim(WorldConfig::new(128, 128, 64).unwrap());

    fill_rect(&sim, 64, 1, 126, 126, MATERIAL_STONE); // chunk 1 stable region
    fill_rect(&sim, 30, 20, 33, 60, MATERIAL_SAND); // falling column (chunk 0)

    sim.tick().expect("tick 1");
    sim.tick().expect("tick 2");
    sim.tick().expect("tick 3");

    let acts = chunk_activity(&sim);
    let stables = chunk_stable(&sim);
    // Chunk 0: the sand column has EMPTY below → active.
    assert_ne!(acts[0] & ACTIVITY_MATTER, 0);
    // Chunk 1: pure Stone, no frontier, stable counter growing.
    assert_eq!(acts[1], 0);
    assert!(stables[1] >= 3);
}

// ─────────────────────────────────────────────────────────────────────────────
// Chunk boundary
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn chunk_boundary_frontier_marks_both_relevant_chunks() {
    // A water pool whose active EMPTY-interface columns sit on BOTH sides of
    // the x=63/64 chunk seam → both chunks report MATTER activity.
    let mut sim = make_sim(WorldConfig::new(128, 128, 64).unwrap());

    fill_rect(&sim, 8, 53, 119, 55, MATERIAL_STONE); // floor
                                                     // EMPTY columns on both sides of the pool.
    fill_rect(&sim, 8, 8, 39, 52, MATERIAL_EMPTY);
    fill_rect(&sim, 88, 8, 119, 52, MATERIAL_EMPTY);
    // Pool spans the seam (x 40..87 crosses 63/64).
    fill_rect(&sim, 40, 40, 87, 52, MATERIAL_WATER);

    sim.tick().expect("tick 1");

    let acts = chunk_activity(&sim);
    assert_ne!(acts[0] & ACTIVITY_MATTER, 0); // chunk (0,0): left interface
    assert_ne!(acts[1] & ACTIVITY_MATTER, 0); // chunk (1,0): right interface
}

// ─────────────────────────────────────────────────────────────────────────────
// False-sleep hazard fixtures (wake detection must catch these)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn sand_falling_into_water_wakes_interface() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());

    // Sealed water pool below; Sand column above with EMPTY beneath it.
    fill_rect(&sim, 8, 44, 55, 55, MATERIAL_STONE); // basin floor + walls shell
    fill_rect(&sim, 20, 40, 43, 52, MATERIAL_WATER); // pool
    fill_rect(&sim, 30, 20, 33, 35, MATERIAL_SAND); // sand above, EMPTY below

    sim.tick().expect("tick 1");
    sim.tick().expect("tick 2");
    sim.tick().expect("tick 3");

    // Sand is falling (EMPTY below) and then swaps into Water → MATTER
    // frontier in the chunk throughout.
    assert_ne!(chunk_activity(&sim)[0] & ACTIVITY_MATTER, 0);
    // Swaps conserve Matter: counts unchanged.
    assert_eq!(count_material(&sim, MATERIAL_SAND), 64);
    assert_eq!(count_material(&sim, MATERIAL_WATER), 24 * 13);
}

#[test]
fn thermal_frontier_wakes_cold_steam_candidate() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());

    // Sealed Steam chamber (T=80 uniform, stable above the 40 condense
    // threshold) inside a Stone world, with a Stone reservoir beside it.
    // No EMPTY exists anywhere in the interior (EMPTY resets to T=0 and
    // would create an unavoidable gradient); the boundary ring is included.
    fill_rect(&sim, 1, 1, 62, 62, MATERIAL_STONE); // whole interior
    fill_rect(&sim, 22, 20, 53, 43, MATERIAL_STEAM); // sealed gas chamber
                                                     // Phase 1: uniform 80.0 across the ENTIRE world (incl. the ring).
    for y in 0..=63 {
        for x in 0..=63 {
            set_t(&sim, x, y, 80.0);
        }
    }

    sim.tick().expect("tick 1");
    sim.tick().expect("tick 2");
    // Uniform field, sealed GAS with no EMPTY interface → no frontier.
    assert_eq!(chunk_activity(&sim)[0] & ACTIVITY_THERMAL, 0);
    assert!(chunk_stable(&sim)[0] >= 2);

    // A hot front arrives at the reservoir (edit): the Steam chamber's
    // cells near the shared wall develop a temperature gradient → the
    // stable chunk wakes (THERMAL).
    fill_rect(&sim, 8, 8, 17, 55, MATERIAL_STONE);
    for y in 8..=55 {
        for x in 8..=17 {
            set_t(&sim, x, y, 200.0);
        }
    }

    sim.tick().expect("tick 3 (thermal wake)");

    let mask = chunk_activity(&sim)[0];
    assert_ne!(mask & ACTIVITY_THERMAL, 0);
    assert_eq!(chunk_stable(&sim)[0], 0);
}

#[test]
fn ignition_heat_wakes_sleep_candidate_wood() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());

    // Cold Wood sealed in Stone: STATIC, no frontier, stable.
    fill_rect(&sim, 8, 8, 55, 55, MATERIAL_STONE);
    fill_rect(&sim, 30, 20, 33, 21, MATERIAL_WOOD);

    sim.tick().expect("tick 1");
    sim.tick().expect("tick 2");
    sim.tick().expect("tick 3");
    assert_eq!(chunk_activity(&sim)[0], 0);
    assert_eq!(chunk_stable(&sim)[0], 3);

    // Ignition heat arrives (Wood above its 90 threshold): combustion
    // starts → reaction + thermal frontier, stable counter resets.
    for y in 20..=21 {
        for x in 30..=33 {
            set_t(&sim, x, y, 100.0);
        }
    }

    sim.tick().expect("tick 4 (ignition wake)");

    let mask = chunk_activity(&sim)[0];
    assert_ne!(mask & ACTIVITY_REACTION, 0);
    assert_eq!(chunk_stable(&sim)[0], 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase transition activity (zero-gradient fixtures)
//
// Each positive fixture is a SEALED chamber inside Stone with one uniform
// temperature over the ENTIRE world (ring included), so there is no
// temperature gradient anywhere. Activity can therefore only come from the
// phase rule: the phase pass self-marks the transition tick in the activity
// buffer (THERMAL), so the chunk that performed phase work is never
// observed as stable. The detector also checks the phase condition directly
// (defensive — 1:1 transitions resolve within one tick).
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn uniform_water_above_boil_threshold_reports_thermal_active() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    fill_rect(&sim, 1, 1, 62, 62, MATERIAL_STONE);
    fill_rect(&sim, 20, 20, 43, 43, MATERIAL_WATER); // sealed chamber
    fill_uniform_t(&sim, 70.0, 64, 64); // uniform, above boil (60)

    sim.tick().expect("tick 1 (boil)");

    // Zero gradient; the phase transition itself marks the tick.
    let mask = chunk_activity(&sim)[0];
    assert_ne!(mask & ACTIVITY_THERMAL, 0);
    assert_eq!(chunk_stable(&sim)[0], 0);
}

#[test]
fn uniform_steam_below_condense_threshold_reports_thermal_active() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    fill_rect(&sim, 1, 1, 62, 62, MATERIAL_STONE);
    fill_rect(&sim, 20, 20, 43, 43, MATERIAL_STEAM); // sealed chamber
    fill_uniform_t(&sim, 30.0, 64, 64); // uniform, below condense (40)

    sim.tick().expect("tick 1 (condense)");

    let mask = chunk_activity(&sim)[0];
    assert_ne!(mask & ACTIVITY_THERMAL, 0);
    assert_eq!(chunk_stable(&sim)[0], 0);
}

#[test]
fn uniform_water_below_freeze_threshold_reports_thermal_active() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    fill_rect(&sim, 1, 1, 62, 62, MATERIAL_STONE);
    fill_rect(&sim, 20, 20, 43, 43, MATERIAL_WATER); // sealed chamber
    fill_uniform_t(&sim, -30.0, 64, 64); // uniform, below freeze (-20)

    sim.tick().expect("tick 1 (freeze)");

    let mask = chunk_activity(&sim)[0];
    assert_ne!(mask & ACTIVITY_THERMAL, 0);
    assert_eq!(chunk_stable(&sim)[0], 0);
}

#[test]
fn uniform_ice_above_melt_threshold_reports_thermal_active() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    fill_rect(&sim, 1, 1, 62, 62, MATERIAL_STONE);
    fill_rect(&sim, 20, 20, 43, 43, MATERIAL_ICE); // sealed chamber
    fill_uniform_t(&sim, 0.0, 64, 64); // uniform, above melt (-10)

    sim.tick().expect("tick 1 (melt)");

    let mask = chunk_activity(&sim)[0];
    assert_ne!(mask & ACTIVITY_THERMAL, 0);
    assert_eq!(chunk_stable(&sim)[0], 0);
}

#[test]
fn uniform_water_inside_phase_hysteresis_without_gradient_can_be_inactive() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    fill_rect(&sim, 1, 1, 62, 62, MATERIAL_STONE);
    fill_rect(&sim, 20, 20, 43, 43, MATERIAL_WATER); // sealed chamber
    fill_uniform_t(&sim, 0.0, 64, 64); // uniform reference T, hysteresis band

    sim.tick().expect("tick 1");
    sim.tick().expect("tick 2");
    sim.tick().expect("tick 3");

    // No phase rule fires at T=0, no gradient, sealed → fully inactive and
    // the stable counter grows (hysteresis-safe negative test).
    assert_eq!(chunk_activity(&sim)[0], 0);
    assert_eq!(chunk_stable(&sim)[0], 3);
}

// ─────────────────────────────────────────────────────────────────────────────
// Cross-chunk frontiers (cell-level stencil crosses the seam in world coords)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cross_chunk_thermal_frontier_detected() {
    // 128×128 → 4 chunks; seam at x=63/64. Uniform halves (left 100 / right
    // 0) create a thermal gradient exactly at the seam → BOTH chunks
    // (0,0) and (1,0) report THERMAL.
    let mut sim = make_sim(WorldConfig::new(128, 128, 64).unwrap());
    fill_rect(&sim, 1, 1, 126, 126, MATERIAL_STONE);
    for y in 0..128 {
        for x in 0..64 {
            set_t(&sim, x, y, 100.0);
        }
        for x in 64..128 {
            set_t(&sim, x, y, 0.0);
        }
    }

    sim.tick().expect("tick 1");

    let acts = chunk_activity(&sim);
    assert_ne!(acts[0] & ACTIVITY_THERMAL, 0); // chunk (0,0): seam at x=63
    assert_ne!(acts[1] & ACTIVITY_THERMAL, 0); // chunk (1,0): seam at x=64
}

#[test]
fn cross_chunk_pressure_frontier_detected() {
    // Sealed Water pocket straddling the x=63/64 seam; P=50 staged on the
    // chunk-0 side, P=0 on the chunk-1 side. Pressure (a medium-only field)
    // is detected on BOTH sides of the seam — the chunk boundary is not a
    // pressure wall.
    let mut sim = make_sim(WorldConfig::new(128, 128, 64).unwrap());
    fill_rect(&sim, 1, 1, 126, 126, MATERIAL_STONE);
    fill_rect(&sim, 62, 31, 65, 33, MATERIAL_WATER); // crosses x=63/64
    for y in 31..=33 {
        for x in 62..=63 {
            set_p(&sim, x, y, 50.0);
        }
    }

    sim.tick().expect("tick 1");

    let acts = chunk_activity(&sim);
    assert_ne!(acts[0] & ACTIVITY_PRESSURE, 0); // chunk (0,0): P=50 side
    assert_ne!(acts[1] & ACTIVITY_PRESSURE, 0); // chunk (1,0): P=0 side
}

#[test]
fn uniform_pressurized_medium_sealed_by_stone_is_not_pressure_frontier() {
    // G5 contract: pressure exchanges only between pressure media. A
    // uniformly pressured Water body sealed by Stone has no medium-medium
    // pressure delta, so the Stone boundary is NOT a pressure frontier (the
    // old detector wrongly compared the medium against its non-medium
    // neighbors' zeroed field). The Stone-only neighbor chunk is inactive.
    let mut sim = make_sim(WorldConfig::new(128, 128, 64).unwrap());
    fill_rect(&sim, 1, 1, 126, 126, MATERIAL_STONE);
    fill_rect(&sim, 62, 31, 63, 33, MATERIAL_WATER); // sealed, chunk 0 only
    for y in 31..=33 {
        for x in 62..=63 {
            set_p(&sim, x, y, 50.0);
        }
    }
    // Chunk 1 (x 64..127) stays pure Stone.

    sim.tick().expect("tick 1");

    let acts = chunk_activity(&sim);
    // Uniformly pressured medium: no pressure work at the Stone boundary.
    assert_eq!(acts[0] & ACTIVITY_PRESSURE, 0);
    assert_eq!(acts[1], 0); // Stone-only chunk: nothing at all
}

// ─────────────────────────────────────────────────────────────────────────────
// G7-A hardening: detector alignment to frozen G4/G5 + stale-bit regressions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn hot_matter_next_to_empty_does_not_false_report_thermal() {
    // G4: EMPTY is not a thermal medium. A hot Stone cell surrounded only by
    // EMPTY has no heat-exchange edge, so its temperature difference against
    // the EMPTY reference is NOT thermal work.
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    set(&sim, 32, 32, MATERIAL_STONE);
    set_t(&sim, 32, 32, 100.0);

    sim.tick().expect("tick 1");

    let mask = chunk_activity(&sim)[0];
    assert_eq!(mask & ACTIVITY_THERMAL, 0);
    assert_eq!(mask, 0);
}

#[test]
fn temperature_difference_across_boundary_block_is_inactive() {
    // Boundary Block has conductivity 0 (frozen G4): a temperature
    // difference across it performs no heat exchange, so it is not a
    // thermal frontier.
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    set(&sim, 30, 30, MATERIAL_STONE);
    set_t(&sim, 30, 30, 100.0);
    set(&sim, 31, 30, MATERIAL_BOUNDARY_BLOCK);
    set_t(&sim, 31, 30, 0.0);

    sim.tick().expect("tick 1");

    let mask = chunk_activity(&sim)[0];
    assert_eq!(mask & ACTIVITY_THERMAL, 0);
    assert_eq!(mask, 0);
}

#[test]
fn conductive_stone_gradient_reports_thermal_active() {
    // A real conductive edge (Stone↔Stone, K=0.5 on both sides) with a
    // temperature difference IS thermal work.
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    fill_rect(&sim, 1, 1, 62, 62, MATERIAL_STONE);
    set_t(&sim, 30, 30, 100.0);

    sim.tick().expect("tick 1");

    assert_ne!(chunk_activity(&sim)[0] & ACTIVITY_THERMAL, 0);
}

#[test]
fn matter_frontier_clears_when_settled() {
    // Stale-bit regression: a real MATTER frontier that disappears (Sand
    // column lands on the floor) must clear the MATTER bit and let the
    // stable counter resume.
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    fill_rect(&sim, 28, 53, 32, 55, MATERIAL_STONE); // landing floor
    fill_rect(&sim, 30, 20, 30, 40, MATERIAL_SAND); // column in flight

    sim.tick().expect("tick 1");
    assert_ne!(chunk_activity(&sim)[0] & ACTIVITY_MATTER, 0);

    let mut settled = false;
    for _ in 0..200 {
        sim.tick().expect("settle tick");
        if chunk_activity(&sim)[0] & ACTIVITY_MATTER == 0 {
            settled = true;
            break;
        }
    }
    assert!(settled, "sand column must settle and clear MATTER activity");
    let stable_after_settle = chunk_stable(&sim)[0];
    sim.tick().expect("tick after settle");
    assert_eq!(chunk_activity(&sim)[0] & ACTIVITY_MATTER, 0);
    assert!(chunk_stable(&sim)[0] > stable_after_settle);
}

#[test]
fn pressure_frontier_clears_when_uniform() {
    // Stale-bit regression: a staged pressure gradient diffuses to uniform
    // inside the medium through normal G5 propagation; PRESSURE must clear
    // and the stable counter resume.
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    fill_rect(&sim, 1, 1, 62, 62, MATERIAL_STONE);
    fill_rect(&sim, 26, 26, 37, 37, MATERIAL_WATER); // sealed pocket
    for y in 26..=37 {
        for x in 26..=37 {
            let p = if x <= 31 { 50.0 } else { 0.0 };
            set_p(&sim, x, y, p);
        }
    }

    sim.tick().expect("tick 1");
    assert_ne!(chunk_activity(&sim)[0] & ACTIVITY_PRESSURE, 0);

    let mut uniform = false;
    for _ in 0..2000 {
        sim.tick().expect("diffuse tick");
        if chunk_activity(&sim)[0] & ACTIVITY_PRESSURE == 0 {
            uniform = true;
            break;
        }
    }
    assert!(
        uniform,
        "pressure must diffuse to uniform and clear PRESSURE"
    );
    let stable_after = chunk_stable(&sim)[0];
    sim.tick().expect("tick after uniform");
    assert_eq!(chunk_activity(&sim)[0] & ACTIVITY_PRESSURE, 0);
    assert!(chunk_stable(&sim)[0] > stable_after);
}

#[test]
fn reaction_frontier_clears_when_extinguished() {
    // Stale-bit regression: a real REACTION frontier that ends (burning
    // Matter cooled below its sustain threshold extinguishes through normal
    // combustion semantics) must clear the REACTION bit. With a uniform
    // temperature field the chunk returns to stable and the counter resumes.
    let mut sim = make_sim(WorldConfig::new(64, 64, 64).unwrap());
    fill_rect(&sim, 1, 1, 62, 62, MATERIAL_STONE);
    // Uniform cold field (no gradient anywhere).
    fill_uniform_t(&sim, 45.0, 64, 64);
    // Staged burning Wood already below its sustain threshold (55): the next
    // combustion pass extinguishes it.
    for y in 30..=31 {
        for x in 30..=33 {
            set(&sim, x, y, MATERIAL_WOOD);
            set_t(&sim, x, y, 45.0);
            sim.world
                .write_flags(&sim.context.queue, x, y, FLAG_COMBUSTING)
                .expect("flags edit");
        }
    }

    sim.tick().expect("tick 1 (extinguish)");
    let mask = chunk_activity(&sim)[0];
    assert_eq!(mask & ACTIVITY_REACTION, 0);
    // Uniform field + STATIC matter: no other frontier remains.
    assert_eq!(mask, 0);
    assert_eq!(chunk_stable(&sim)[0], 1);

    sim.tick().expect("tick 2");
    assert_eq!(chunk_activity(&sim)[0], 0);
    assert_eq!(chunk_stable(&sim)[0], 2);
}
