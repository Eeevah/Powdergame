//! G5-C — structural rupture / opening / vent GPU integration tests.
//!
//! Requires Windows + RTX 5090 + DX12. G5-C adds only a generic structural
//! self-write rule: finite-strength Matter reads neighboring Liquid/Gas
//! pressure and becomes EMPTY at its descriptor threshold. Venting then
//! emerges from ordinary Matter movement through that opening.

use powdergame_core::{
    WorldConfig, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_STEAM, MATERIAL_STONE,
    MATERIAL_WATER, MATERIAL_WOOD, PRESSURE_REFERENCE, WATER_BOIL_BLOCKED_PRESSURE,
    WOOD_RUPTURE_THRESHOLD,
};
use powdergame_gpu::Simulation;

fn make_sim(config: WorldConfig) -> Simulation {
    pollster::block_on(Simulation::new(config)).expect("DX12 + RTX 5090 simulation init")
}

fn eight_by_eight() -> Simulation {
    make_sim(WorldConfig::new(8, 8, 8).unwrap())
}

fn set(sim: &Simulation, x: i64, y: i64, material: u32) {
    sim.world
        .write_material(&sim.context.queue, x, y, material)
        .expect("material edit");
}

fn set_t(sim: &Simulation, x: i64, y: i64, value: f32) {
    sim.world
        .write_temperature(&sim.context.queue, x, y, value)
        .expect("temperature edit");
}

fn set_p(sim: &Simulation, x: i64, y: i64, value: f32) {
    sim.world
        .write_pressure(&sim.context.queue, x, y, value)
        .expect("pressure edit");
}

fn cell(sim: &Simulation, x: i64, y: i64) -> u32 {
    sim.world
        .read_material_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("material readback")
}

fn pressure(sim: &Simulation, x: i64, y: i64) -> f32 {
    sim.world
        .read_pressure_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("pressure readback")
}

fn block_water_motion_except_top_wall(sim: &Simulation, wall_material: u32) {
    // Water at (3,3). Liquid candidates down/down-diagonal/lateral are Stone;
    // the top cell (3,2) is the structural wall stressed by Pressure.
    set(sim, 3, 2, wall_material);
    for (x, y) in [(2, 3), (4, 3), (2, 4), (3, 4), (4, 4)] {
        set(&sim, x, y, MATERIAL_STONE);
    }
    set(sim, 3, 3, MATERIAL_WATER);
}

#[test]
fn wood_survives_sub_threshold_pressure() {
    let mut sim = eight_by_eight();
    block_water_motion_except_top_wall(&sim, MATERIAL_WOOD);
    set_p(&sim, 3, 3, WOOD_RUPTURE_THRESHOLD - 1.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 2), MATERIAL_WOOD);
    assert!(pressure(&sim, 3, 3) < WOOD_RUPTURE_THRESHOLD);
}

#[test]
fn wood_ruptures_from_threshold_exceeding_neighbor_pressure() {
    let mut sim = eight_by_eight();
    block_water_motion_except_top_wall(&sim, MATERIAL_WOOD);
    set_p(&sim, 3, 3, WATER_BOIL_BLOCKED_PRESSURE);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 2), MATERIAL_EMPTY, "weak wall opened");
    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_WATER,
        "pressure stress alone does not transmute the medium"
    );
}

#[test]
fn stone_and_boundary_remain_reference_unbreakable_walls() {
    // Stone intentionally remains unbreakable in M0 because frozen G5-A
    // pressure fixtures use Stone containment up to PRESSURE_MAX.
    let mut sim = eight_by_eight();
    block_water_motion_except_top_wall(&sim, MATERIAL_STONE);
    set_p(&sim, 3, 3, 1.0e6);
    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 3, 2), MATERIAL_STONE);

    let mut sim = eight_by_eight();
    set(&sim, 3, 1, MATERIAL_WATER);
    for (x, y) in [(2, 1), (4, 1), (2, 2), (3, 2), (4, 2)] {
        set(&sim, x, y, MATERIAL_STONE);
    }
    set_p(&sim, 3, 1, 1.0e6);
    sim.tick().expect("tick");
    assert_eq!(cell(&sim, 3, 0), MATERIAL_BOUNDARY_BLOCK);
}

#[test]
fn rupture_crosses_64_cell_chunk_boundary() {
    let mut sim = make_sim(WorldConfig::new(128, 16, 64).unwrap());
    // Pressure medium on x=63 stresses Wood on x=64 across the chunk edge.
    set(&sim, 63, 8, MATERIAL_WATER);
    set(&sim, 64, 8, MATERIAL_WOOD);
    for (x, y) in [(62, 8), (62, 9), (63, 9), (64, 9)] {
        set(&sim, x, y, MATERIAL_STONE);
    }
    set_p(&sim, 63, 8, WATER_BOIL_BLOCKED_PRESSURE);

    sim.tick().expect("tick");

    assert_eq!(
        cell(&sim, 64, 8),
        MATERIAL_EMPTY,
        "chunk edge is not a stress wall"
    );
}

#[test]
fn blocked_boiling_ruptures_weak_wall_then_vents_on_following_tick() {
    let mut sim = eight_by_eight();
    // One weak top wall; every other 8-neighbor is occupied so G5-B cannot
    // satisfy Water→Steam yield=2. Above the weak wall is ordinary EMPTY.
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 80.0);
    set(&sim, 3, 2, MATERIAL_WOOD);
    for (x, y) in [(2, 2), (4, 2), (2, 3), (4, 3), (2, 4), (3, 4), (4, 4)] {
        set(&sim, x, y, MATERIAL_STONE);
    }

    // Tick 1: hot Water cannot expand, becomes Steam +100 pressure, then
    // the neighboring Wood reads that pressure and ruptures to EMPTY.
    sim.tick().expect("boil + confinement + rupture tick");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_STEAM, "water boiled in place");
    assert_eq!(
        cell(&sim, 3, 2),
        MATERIAL_EMPTY,
        "weak wall opened from pressure"
    );
    let confined = pressure(&sim, 3, 3);
    assert!(
        confined >= WOOD_RUPTURE_THRESHOLD,
        "confinement pressure must exist before vent movement; got {confined}"
    );

    // Tick 2: ordinary GAS movement sees the newly EMPTY opening and moves
    // Steam into it. Because Pressure is spatial (not transported with
    // Matter), the vacated source pressure is cleared by the G5-A pass.
    sim.tick().expect("vent movement tick");
    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_EMPTY,
        "pressurized source volume vented"
    );
    assert_eq!(
        cell(&sim, 3, 2),
        MATERIAL_STEAM,
        "steam moved through the rupture opening"
    );
    assert_eq!(
        pressure(&sim, 3, 3),
        PRESSURE_REFERENCE,
        "vacated spatial pressure released"
    );
}
