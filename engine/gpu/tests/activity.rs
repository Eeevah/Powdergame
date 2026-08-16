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
//!   ACTIVITY_THERMAL   = 1 << 1   temperature gradient or heat source
//!   ACTIVITY_PRESSURE  = 1 << 2   pressure gradient
//!   ACTIVITY_REACTION  = 1 << 3   combustion / decay state progressing

use powdergame_core::{
    WorldConfig, ACTIVITY_MATTER, ACTIVITY_PRESSURE, ACTIVITY_REACTION, ACTIVITY_THERMAL,
    MATERIAL_EMPTY, MATERIAL_SAND, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD,
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
