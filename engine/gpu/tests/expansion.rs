//! G5-B — phase expansion / confinement → Pressure GPU integration tests.
//!
//! Requires production Windows + RTX 5090 + DX12 through `Simulation::new`.

use powdergame_core::{
    standard_air_state, AirState, WorldConfig, AIR_ENERGY_MAX, AIR_MASS_MAX, MATERIAL_EMPTY,
    MATERIAL_ICE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, STANDARD_AIR_ENERGY,
    WATER_BOIL_BLOCKED_PRESSURE,
};
use powdergame_gpu::Simulation;

fn make_sim(config: WorldConfig) -> Simulation {
    pollster::block_on(Simulation::new(config)).expect("DX12 + RTX 5090 simulation init")
}

fn eight_by_eight() -> Simulation {
    make_sim(WorldConfig::new(8, 8, 8).unwrap())
}

fn set_mat(sim: &Simulation, x: i64, y: i64, material: u32) {
    sim.world
        .write_material(&sim.context.queue, x, y, material)
        .expect("material edit");
}

fn set_t(sim: &Simulation, x: i64, y: i64, value: f32) {
    sim.world
        .write_temperature(&sim.context.queue, x, y, value)
        .expect("temperature edit");
}

fn cell(sim: &Simulation, x: i64, y: i64) -> u32 {
    sim.world
        .read_material_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("material readback")
}

fn temp(sim: &Simulation, x: i64, y: i64) -> f32 {
    sim.world
        .read_temperature_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("temperature readback")
}

fn pressure(sim: &Simulation, x: i64, y: i64) -> f32 {
    sim.world
        .read_pressure_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("pressure readback")
}

fn air(sim: &Simulation, cells: &[(i64, i64)]) -> Vec<powdergame_gpu::EnvironmentCellSnapshot> {
    sim.world
        .read_environment_cells(&sim.context.device, &sim.context.queue, cells)
        .expect("bounded Environment readback")
}

fn clear_region(sim: &Simulation, x0: i64, y0: i64, x1: i64, y1: i64) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            set_mat(sim, x, y, MATERIAL_EMPTY);
        }
    }
}

fn seal_eight(sim: &Simulation, x: i64, y: i64) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx != 0 || dy != 0 {
                set_mat(sim, x + dx, y + dy, MATERIAL_STONE);
            }
        }
    }
}

#[test]
fn boiling_with_space_spawns_second_steam_without_pressure() {
    let mut sim = eight_by_eight();
    clear_region(&sim, 1, 1, 6, 6);
    seal_eight(&sim, 3, 3);
    set_mat(&sim, 3, 2, MATERIAL_EMPTY); // first expansion candidate (up)
    set_mat(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 1000.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_STEAM);
    assert_eq!(cell(&sim, 3, 2), MATERIAL_STEAM);
    assert_eq!(pressure(&sim, 3, 3), 0.0);
    assert_eq!(pressure(&sim, 3, 2), 0.0);
    let source_t = temp(&sim, 3, 3);
    let spawn_t = temp(&sim, 3, 2);
    assert!(source_t > 60.0);
    assert!(
        (source_t - spawn_t).abs() < 1.0e-3,
        "source={source_t} spawn={spawn_t}"
    );
    let environment = air(&sim, &[(3, 3), (3, 2), (3, 1)]);
    assert_eq!(
        environment[0].current,
        AirState {
            mass: 0.0,
            energy: 0.0
        }
    );
    assert_eq!(
        environment[1].current,
        AirState {
            mass: 0.0,
            energy: 0.0
        }
    );
    assert_eq!(
        environment[2].current,
        AirState {
            mass: 2.0,
            energy: STANDARD_AIR_ENERGY * 2.0,
        },
        "the target parcel moves whole into the deterministic receiver"
    );
    assert!(environment.iter().all(|cell| cell.current == cell.next));
}

#[test]
fn environment_blocked_spawn_keeps_target_air_and_adds_pressure_exactly_once() {
    let mut sim = eight_by_eight();
    clear_region(&sim, 1, 1, 6, 6);
    seal_eight(&sim, 3, 3);
    set_mat(&sim, 3, 2, MATERIAL_EMPTY); // Matter target exists.
    set_mat(&sim, 3, 1, MATERIAL_STONE); // No orthogonal receiver remains.
    set_mat(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 1000.0);
    let before_target = air(&sim, &[(3, 2)])[0];
    assert_eq!(before_target.current, standard_air_state());

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_STEAM);
    assert_eq!(cell(&sim, 3, 2), MATERIAL_EMPTY);
    let after_target = air(&sim, &[(3, 2)])[0];
    assert_eq!(after_target.current, standard_air_state());
    assert_eq!(after_target.current, after_target.next);
    assert!(
        (pressure(&sim, 3, 3) - WATER_BOIL_BLOCKED_PRESSURE).abs() < 1.0e-3,
        "Environment-blocked expansion must reuse the existing pressure consequence exactly once"
    );
}

#[test]
fn receiver_without_whole_parcel_headroom_blocks_without_clamping_or_deletion() {
    let mut sim = eight_by_eight();
    clear_region(&sim, 1, 1, 6, 6);
    seal_eight(&sim, 3, 3);
    set_mat(&sim, 3, 2, MATERIAL_EMPTY);
    set_mat(&sim, 3, 1, MATERIAL_EMPTY);
    set_mat(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 1000.0);
    let full = AirState {
        mass: AIR_MASS_MAX,
        energy: AIR_ENERGY_MAX,
    };
    sim.world
        .write_environment_cell_for_test(&sim.context.queue, 3, 1, full)
        .unwrap();

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 2), MATERIAL_EMPTY);
    let environment = air(&sim, &[(3, 2), (3, 1)]);
    assert_eq!(environment[0].current, standard_air_state());
    assert_eq!(environment[1].current, full);
    assert!(environment.iter().all(|cell| cell.current == cell.next));
    assert!(
        (pressure(&sim, 3, 3) - WATER_BOIL_BLOCKED_PRESSURE).abs() < 1.0e-3,
        "headroom rejection uses the existing pressure consequence"
    );
}

#[test]
fn fully_confined_boiling_generates_pressure_instead_of_extra_matter() {
    let mut sim = eight_by_eight();
    clear_region(&sim, 1, 1, 6, 6);
    seal_eight(&sim, 3, 3);
    set_mat(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 1000.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_STEAM);
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx != 0 || dy != 0 {
                assert_ne!(cell(&sim, 3 + dx, 3 + dy), MATERIAL_STEAM);
            }
        }
    }
    let p = pressure(&sim, 3, 3);
    assert!(
        (p - WATER_BOIL_BLOCKED_PRESSURE).abs() < 1.0e-3,
        "blocked pressure={p}"
    );
}

#[test]
fn competing_expansions_have_one_winner_and_loser_becomes_pressure() {
    let mut sim = eight_by_eight();
    clear_region(&sim, 1, 1, 6, 6);

    // Two Water cells can only expand into shared up-diagonal (4,3).
    for (x, y) in [
        (3, 3),
        (2, 3), // A: up, up-left blocked; up-right is target
        (5, 3), // B: up blocked; up-left is target
        (2, 4),
        (4, 4),
        (6, 4),
        (2, 5),
        (3, 5),
        (4, 5),
        (5, 5),
        (6, 5),
    ] {
        set_mat(&sim, x, y, MATERIAL_STONE);
    }
    set_mat(&sim, 4, 3, MATERIAL_EMPTY);
    set_mat(&sim, 3, 4, MATERIAL_WATER); // contender A
    set_mat(&sim, 5, 4, MATERIAL_WATER); // contender B
    set_t(&sim, 3, 4, 1000.0);
    set_t(&sim, 5, 4, 1000.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 4), MATERIAL_STEAM);
    assert_eq!(cell(&sim, 5, 4), MATERIAL_STEAM);
    assert_eq!(
        cell(&sim, 4, 3),
        MATERIAL_STEAM,
        "exactly one destination winner"
    );
    let p_a = pressure(&sim, 3, 4);
    let p_b = pressure(&sim, 5, 4);
    let a_won = p_a.abs() < 1.0e-3 && (p_b - WATER_BOIL_BLOCKED_PRESSURE).abs() < 1.0e-3;
    let b_won = p_b.abs() < 1.0e-3 && (p_a - WATER_BOIL_BLOCKED_PRESSURE).abs() < 1.0e-3;
    assert!(
        a_won || b_won,
        "exactly one winner (p=0) and one loser (p={WATER_BOIL_BLOCKED_PRESSURE}); got p_a={p_a}, p_b={p_b}"
    );
}

#[test]
fn expansion_can_cross_a_64_cell_chunk_boundary() {
    let mut sim = make_sim(WorldConfig::new(16, 128, 64).unwrap());
    clear_region(&sim, 5, 61, 11, 67);
    seal_eight(&sim, 8, 64);
    set_mat(&sim, 8, 63, MATERIAL_EMPTY); // up target lies in previous y chunk
    set_mat(&sim, 8, 64, MATERIAL_WATER);
    set_t(&sim, 8, 64, 1000.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 8, 64), MATERIAL_STEAM);
    assert_eq!(cell(&sim, 8, 63), MATERIAL_STEAM);
    assert_eq!(pressure(&sim, 8, 64), 0.0);
}

#[test]
fn one_to_one_phase_transition_creates_no_expansion_pressure() {
    let mut sim = eight_by_eight();
    clear_region(&sim, 1, 1, 6, 6);
    set_mat(&sim, 3, 3, MATERIAL_ICE);
    set_t(&sim, 3, 3, 100.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_WATER);
    assert_eq!(pressure(&sim, 3, 3), 0.0);
    let materials = sim
        .world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("material readback");
    assert_eq!(
        materials.iter().filter(|&&m| m == MATERIAL_WATER).count(),
        1
    );
}
