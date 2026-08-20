//! TE-2 production-GPU Air transport, headroom, and boundary regressions.

use powdergame_core::{
    standard_air_state, vacuum_air_state, AirState, EnvironmentBoundaryMode, WorldConfig,
    ACTIVITY_ENVIRONMENT, AIR_MASS_MAX, AIR_MAX_OUTFLOW_FRACTION, AIR_ZERO_OFFSET,
    CHUNK_STATE_SLEEPING, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY,
};
use powdergame_gpu::Simulation;

fn simulation() -> Simulation {
    pollster::block_on(Simulation::new(WorldConfig::new(8, 8, 8).unwrap())).unwrap()
}

fn all_cells() -> Vec<(i64, i64)> {
    (0..8).flat_map(|y| (0..8).map(move |x| (x, y))).collect()
}

fn totals(sim: &Simulation) -> (f64, f64) {
    sim.world
        .read_environment_cells(&sim.context.device, &sim.context.queue, &all_cells())
        .unwrap()
        .iter()
        .fold((0.0, 0.0), |(mass, energy), cell| {
            (
                mass + cell.current.mass as f64,
                energy + cell.current.energy as f64,
            )
        })
}

fn wall(sim: &Simulation, cells: &[(i64, i64)]) {
    for &(x, y) in cells {
        sim.world
            .write_material(&sim.context.queue, x, y, MATERIAL_BOUNDARY_BLOCK)
            .unwrap();
    }
}

#[test]
fn multi_source_receiver_headroom_is_bounded_conservative_and_deterministic() {
    let mut first = simulation();
    let mut second = simulation();
    for sim in [&first, &second] {
        wall(
            sim,
            &[
                (2, 3),
                (2, 4),
                (2, 5),
                (3, 2),
                (4, 2),
                (5, 2),
                (6, 3),
                (6, 4),
                (6, 5),
                (3, 6),
                (4, 6),
                (5, 6),
                (3, 3),
                (5, 3),
                (3, 5),
                (5, 5),
            ],
        );
        sim.world
            .write_environment_cell_for_test(
                &sim.context.queue,
                4,
                4,
                AirState {
                    mass: 15.9,
                    energy: 15.9 * (2_000.0 + AIR_ZERO_OFFSET),
                },
            )
            .unwrap();
        for (x, y) in [(3, 4), (5, 4), (4, 3), (4, 5)] {
            sim.world
                .write_environment_cell_for_test(
                    &sim.context.queue,
                    x,
                    y,
                    AirState {
                        mass: AIR_MASS_MAX,
                        energy: AIR_MASS_MAX * (2_000.0 + AIR_ZERO_OFFSET),
                    },
                )
                .unwrap();
        }
    }
    let before = totals(&first);
    first.tick().unwrap();
    second.tick().unwrap();
    let after = totals(&first);
    assert!((before.0 - after.0).abs() <= 1.0e-4);
    assert!(
        (before.1 - after.1).abs() <= 1.0e-2,
        "energy drift before={} after={} delta={}",
        before.1,
        after.1,
        after.1 - before.1
    );

    let cells = first
        .world
        .read_environment_cells(
            &first.context.device,
            &first.context.queue,
            &[(4, 4), (3, 4), (5, 4), (4, 3), (4, 5)],
        )
        .unwrap();
    assert!(cells[0].current.mass <= AIR_MASS_MAX);
    assert!(cells[0].current.energy <= 36_370.4);
    for donor in &cells[1..] {
        assert!(AIR_MASS_MAX - donor.current.mass <= AIR_MAX_OUTFLOW_FRACTION * AIR_MASS_MAX);
        assert!(donor.current.mass >= 0.0 && donor.current.energy >= 0.0);
    }
    let other = second
        .world
        .read_environment_cells(
            &second.context.device,
            &second.context.queue,
            &[(4, 4), (3, 4), (5, 4), (4, 3), (4, 5)],
        )
        .unwrap();
    assert_eq!(cells, other);
}

#[test]
fn donor_specific_energy_advects_without_a_hidden_heat_source() {
    let mut sim = simulation();
    wall(
        &sim,
        &[
            (2, 2),
            (3, 2),
            (4, 2),
            (2, 3),
            (4, 3),
            (2, 4),
            (3, 4),
            (4, 4),
            (2, 5),
            (3, 5),
            (4, 5),
        ],
    );
    let donor = AirState {
        mass: 1.0,
        energy: 500.0 + AIR_ZERO_OFFSET,
    };
    sim.world
        .write_environment_cell_for_test(&sim.context.queue, 3, 3, donor)
        .unwrap();
    sim.world
        .write_environment_cell_for_test(&sim.context.queue, 3, 4, vacuum_air_state())
        .unwrap();
    // Open exactly the second endpoint after installing the surrounding wall.
    sim.world
        .write_material(&sim.context.queue, 3, 4, MATERIAL_EMPTY)
        .unwrap();
    sim.world
        .write_environment_cell_for_test(&sim.context.queue, 3, 4, vacuum_air_state())
        .unwrap();
    let before = totals(&sim);
    sim.tick().unwrap();
    let after = totals(&sim);
    let pair = sim
        .world
        .read_environment_cells(&sim.context.device, &sim.context.queue, &[(3, 3), (3, 4)])
        .unwrap();
    assert!(pair[1].current.mass > 0.0);
    let receiver_temperature = pair[1].current.energy / pair[1].current.mass - AIR_ZERO_OFFSET;
    assert!(
        (receiver_temperature - 500.0).abs() <= 1.0e-3,
        "receiver={receiver_temperature} pair={pair:?}"
    );
    assert!((before.0 - after.0).abs() <= 1.0e-4);
    assert!((before.1 - after.1).abs() <= 1.0e-2);
}

#[test]
fn sealed_is_default_and_fixture_reservoir_is_explicit() {
    let mut sealed = simulation();
    let mut reservoir = simulation();
    assert_eq!(
        sealed.environment_boundary_mode,
        EnvironmentBoundaryMode::Sealed
    );
    reservoir
        .set_environment_boundary_mode(EnvironmentBoundaryMode::FixedStandardAtmosphereReservoir);
    for sim in [&sealed, &reservoir] {
        wall(sim, &[(1, 0), (0, 1)]);
        sim.world
            .write_material(&sim.context.queue, 0, 0, MATERIAL_EMPTY)
            .unwrap();
        sim.world
            .write_environment_cell_for_test(&sim.context.queue, 0, 0, vacuum_air_state())
            .unwrap();
    }
    sealed.tick().unwrap();
    reservoir.tick().unwrap();
    let sealed_edge = sealed
        .world
        .read_environment_cells(&sealed.context.device, &sealed.context.queue, &[(0, 0)])
        .unwrap()[0];
    let open_edge = reservoir
        .world
        .read_environment_cells(
            &reservoir.context.device,
            &reservoir.context.queue,
            &[(0, 0)],
        )
        .unwrap()[0];
    assert_eq!(sealed_edge.current, vacuum_air_state());
    assert!(open_edge.current.mass > 0.0);
    assert!(open_edge.current.mass < standard_air_state().mass);
}

#[test]
fn chunk_seam_air_flow_is_bilateral_and_sleep_equivalent() {
    let config = WorldConfig::new(128, 8, 64).unwrap();
    let mut awake = pollster::block_on(Simulation::new(config)).unwrap();
    let mut sleeping = pollster::block_on(Simulation::new(config)).unwrap();
    awake.set_sleep_enabled(false);
    sleeping.set_sleep_enabled(true);
    sleeping.set_sleep_threshold(4);
    for sim in [&awake, &sleeping] {
        wall(
            sim,
            &[
                (62, 2),
                (63, 2),
                (64, 2),
                (65, 2),
                (62, 4),
                (63, 4),
                (64, 4),
                (65, 4),
                (62, 3),
                (65, 3),
            ],
        );
        sim.world
            .write_environment_cell_for_test(
                &sim.context.queue,
                63,
                3,
                AirState {
                    mass: 1.0,
                    energy: 773.15,
                },
            )
            .unwrap();
        sim.world
            .write_environment_cell_for_test(&sim.context.queue, 64, 3, vacuum_air_state())
            .unwrap();
    }

    awake.tick().unwrap();
    sleeping.tick().unwrap();
    let coordinates = [(63, 3), (64, 3)];
    assert_eq!(
        awake
            .world
            .read_environment_cells(&awake.context.device, &awake.context.queue, &coordinates)
            .unwrap(),
        sleeping
            .world
            .read_environment_cells(
                &sleeping.context.device,
                &sleeping.context.queue,
                &coordinates,
            )
            .unwrap()
    );
    let activity = sleeping
        .world
        .read_chunk_activity_all(&sleeping.context.device, &sleeping.context.queue)
        .unwrap();
    assert_ne!(activity[0] & ACTIVITY_ENVIRONMENT, 0);
    assert_ne!(activity[1] & ACTIVITY_ENVIRONMENT, 0);
}

#[test]
fn equilibrium_atmosphere_becomes_sleep_eligible() {
    let mut sim =
        pollster::block_on(Simulation::new(WorldConfig::new(128, 8, 64).unwrap())).unwrap();
    sim.set_sleep_enabled(true);
    sim.set_sleep_threshold(4);
    for _ in 0..16 {
        sim.tick().unwrap();
    }
    let states = sim
        .world
        .read_chunk_state_all(&sim.context.device, &sim.context.queue)
        .unwrap();
    assert_eq!(states, vec![CHUNK_STATE_SLEEPING; 2]);
}
