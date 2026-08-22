//! TE-5R1 actual production-buffer fixtures.
//!
//! These fixtures exercise the production pass graph. They do not implement
//! a second pressure model or substitute a synthetic transaction pipeline.

use powdergame_core::{
    vacuum_air_state, WorldConfig, ACTIVITY_PRESSURE, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY,
    MATERIAL_STEAM, MATERIAL_WATER, MATERIAL_WOOD, PRESSURE_ACTIVITY_EPS,
};
use powdergame_gpu::Simulation;

fn sim() -> Simulation {
    pollster::block_on(Simulation::new(WorldConfig::new(8, 8, 8).unwrap()))
        .expect("DX12 production simulation")
}

fn set_material(sim: &Simulation, x: i64, y: i64, material: u32) {
    sim.world
        .write_material(&sim.context.queue, x, y, material)
        .unwrap();
}

fn set_temperature(sim: &Simulation, x: i64, y: i64, value: f32) {
    sim.world
        .write_temperature(&sim.context.queue, x, y, value)
        .unwrap();
}

fn set_phase(sim: &Simulation, x: i64, y: i64, material: u32, value: f32) {
    sim.world
        .write_phase_energy(&sim.context.queue, x, y, material, value)
        .unwrap();
}

fn set_pressure(sim: &Simulation, x: i64, y: i64, value: f32) {
    sim.world
        .write_pressure(&sim.context.queue, x, y, value)
        .unwrap();
}

fn pressure(sim: &Simulation, x: i64, y: i64) -> f32 {
    sim.world
        .read_pressure_cell(&sim.context.device, &sim.context.queue, x, y)
        .unwrap()
}

fn material(sim: &Simulation, x: i64, y: i64) -> u32 {
    sim.world
        .read_material_cell(&sim.context.device, &sim.context.queue, x, y)
        .unwrap()
}

fn boundary_cage(sim: &Simulation, center: (i64, i64), open_up: bool) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx != 0 || dy != 0 {
                set_material(sim, center.0 + dx, center.1 + dy, MATERIAL_BOUNDARY_BLOCK);
            }
        }
    }
    if open_up {
        set_material(sim, center.0, center.1 - 1, MATERIAL_EMPTY);
    }
}

#[test]
fn f01_f02_air_background_and_vacuum_dynamic_pressure_are_honest() {
    let mut simulation = sim();
    boundary_cage(&simulation, (3, 3), false);
    set_material(&simulation, 3, 3, MATERIAL_EMPTY);
    simulation
        .world
        .write_environment_cell_for_test(&simulation.context.queue, 3, 3, vacuum_air_state())
        .unwrap();
    set_pressure(&simulation, 3, 3, 100.0);

    simulation.tick().unwrap();

    let observed = simulation
        .world
        .read_environment_cells(
            &simulation.context.device,
            &simulation.context.queue,
            &[(3, 3)],
        )
        .unwrap()[0];
    assert_eq!(observed.current, vacuum_air_state());
    assert_eq!(observed.current, observed.next);
    assert!((pressure(&simulation, 3, 3) - 98.0).abs() < 1.0e-4);
}

#[test]
fn f03_f04_water_has_no_target_and_completion_creates_only_bounded_steam_load() {
    let mut partial = sim();
    boundary_cage(&partial, (3, 3), false);
    set_material(&partial, 3, 3, MATERIAL_WATER);
    set_temperature(&partial, 3, 3, 100.0);
    set_phase(&partial, 3, 3, MATERIAL_WATER, 240.0);
    partial.tick().unwrap();
    assert_eq!(material(&partial, 3, 3), MATERIAL_WATER);
    assert_eq!(pressure(&partial, 3, 3), 0.0);

    let mut ready = sim();
    boundary_cage(&ready, (3, 3), true);
    set_material(&ready, 3, 3, MATERIAL_WATER);
    set_temperature(&ready, 3, 3, 300.0);
    set_phase(&ready, 3, 3, MATERIAL_WATER, 480.0);
    ready.tick().unwrap();
    assert_eq!(material(&ready, 3, 3), MATERIAL_STEAM);
    let first = pressure(&ready, 3, 3);
    assert!(first > 0.0 && first <= 2.0 + 1.0e-4, "first={first}");
}

#[test]
fn f05_partial_steam_target_and_f17_target_removal_relax_continuously() {
    let mut full = sim();
    let mut half = sim();
    for simulation in [&full, &half] {
        boundary_cage(simulation, (3, 3), false);
        set_material(simulation, 3, 3, MATERIAL_STEAM);
        set_temperature(simulation, 3, 3, 100.0);
    }
    set_phase(&half, 3, 3, MATERIAL_STEAM, 240.0);
    full.tick().unwrap();
    half.tick().unwrap();
    assert!((pressure(&full, 3, 3) - 2.0).abs() < 1.0e-4);
    assert!((pressure(&half, 3, 3) - 1.0).abs() < 1.0e-4);

    set_phase(&full, 3, 3, MATERIAL_STEAM, 0.0);
    let before = pressure(&full, 3, 3);
    full.tick().unwrap();
    assert!(pressure(&full, 3, 3) < before);
}

#[test]
fn f06_authoritative_pre_pressure_impulse_100_relaxes_to_98_once() {
    let mut simulation = sim();
    boundary_cage(&simulation, (3, 3), false);
    set_material(&simulation, 3, 3, MATERIAL_WATER);
    set_pressure(&simulation, 3, 3, 100.0);

    simulation.tick().unwrap();
    assert!((pressure(&simulation, 3, 3) - 98.0).abs() < 1.0e-4);
    simulation.tick().unwrap();
    assert!((pressure(&simulation, 3, 3) - 96.04).abs() < 1.0e-3);
}

#[test]
fn f07_two_node_nonuniform_equilibrium_has_no_pressure_activity() {
    let mut simulation = sim();
    for y in 2..=4 {
        for x in 2..=5 {
            set_material(&simulation, x, y, MATERIAL_BOUNDARY_BLOCK);
        }
    }
    set_material(&simulation, 3, 3, MATERIAL_STEAM);
    set_temperature(&simulation, 3, 3, 100.0);
    set_material(&simulation, 4, 3, MATERIAL_WATER);
    set_temperature(&simulation, 4, 3, 100.0);
    set_pressure(&simulation, 3, 3, 52.380_95);
    set_pressure(&simulation, 4, 3, 47.619_05);

    simulation.tick().unwrap();

    assert!((pressure(&simulation, 3, 3) - 52.380_95).abs() <= PRESSURE_ACTIVITY_EPS);
    assert!((pressure(&simulation, 4, 3) - 47.619_05).abs() <= PRESSURE_ACTIVITY_EPS);
    let activity = simulation
        .world
        .read_chunk_activity_all(&simulation.context.device, &simulation.context.queue)
        .unwrap()[0];
    assert_eq!(activity & ACTIVITY_PRESSURE, 0);
}

#[test]
fn f18_sleep_enabled_and_disabled_match_for_equal_ticks() {
    let mut sleeping = sim();
    let mut reference = sim();
    sleeping.set_sleep_enabled(true);
    sleeping.set_sleep_threshold(2);
    reference.set_sleep_enabled(false);
    for simulation in [&sleeping, &reference] {
        boundary_cage(simulation, (3, 3), false);
        set_material(simulation, 3, 3, MATERIAL_STEAM);
        set_temperature(simulation, 3, 3, 100.0);
    }
    for _ in 0..256 {
        sleeping.tick().unwrap();
        reference.tick().unwrap();
    }
    let a = sleeping
        .world
        .read_pressure_all(&sleeping.context.device, &sleeping.context.queue)
        .unwrap();
    let b = reference
        .world
        .read_pressure_all(&reference.context.device, &reference.context.queue)
        .unwrap();
    assert_eq!(a, b);
}

#[test]
fn f16_air_background_and_dynamic_pressure_are_added_exactly_once_for_rupture() {
    fn stage(value: f32) -> Simulation {
        let simulation = sim();
        for y in 2..=4 {
            for x in 1..=5 {
                set_material(&simulation, x, y, MATERIAL_BOUNDARY_BLOCK);
            }
        }
        set_material(&simulation, 3, 3, MATERIAL_WOOD);
        set_material(&simulation, 2, 3, MATERIAL_EMPTY);
        set_material(&simulation, 4, 3, MATERIAL_EMPTY);
        simulation
            .world
            .write_environment_cell_for_test(&simulation.context.queue, 4, 3, vacuum_air_state())
            .unwrap();
        set_pressure(&simulation, 2, 3, value);
        simulation
    }

    let mut below_if_added_once = stage(80.5);
    below_if_added_once.tick().unwrap();
    assert_eq!(material(&below_if_added_once, 3, 3), MATERIAL_WOOD);

    let mut above_if_added_once = stage(81.0);
    above_if_added_once.tick().unwrap();
    assert_eq!(material(&above_if_added_once, 3, 3), MATERIAL_EMPTY);
}
