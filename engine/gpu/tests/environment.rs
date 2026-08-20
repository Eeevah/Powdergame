//! TE-1 Environment state, staging, occupancy, and allocation invariants.

use powdergame_core::{
    standard_air_state, vacuum_air_state, with_decay_age, AirState, WorldConfig, WorldLayout,
    MATERIAL_EMPTY, MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STONE, MATERIAL_WATER,
    STANDARD_AIR_ENERGY,
};
use powdergame_gpu::{world::AllocationReport, Simulation, PASS_COUNT, PASS_NAMES};

fn eight_by_eight() -> Simulation {
    pollster::block_on(Simulation::new(WorldConfig::new(8, 8, 8).unwrap()))
        .expect("DX12 + RTX 5090 simulation init")
}

fn observe(sim: &Simulation, cells: &[(i64, i64)]) -> Vec<powdergame_gpu::EnvironmentCellSnapshot> {
    sim.world
        .read_environment_cells(&sim.context.device, &sim.context.queue, cells)
        .expect("bounded Environment readback")
}

#[test]
fn allocation_and_profiler_contracts_are_exact() {
    let small = WorldConfig::new(256, 256, 64).unwrap();
    let small_layout = WorldLayout::new(256 * 256).unwrap();
    let small_report = AllocationReport::from_layout(small, &small_layout);
    assert_eq!(small_report.air_mass_current_bytes, 262_144);
    assert_eq!(small_report.air_mass_next_bytes, 262_144);
    assert_eq!(small_report.air_energy_current_bytes, 262_144);
    assert_eq!(small_report.air_energy_next_bytes, 262_144);
    assert_eq!(small_report.environment_receiver_claim_bytes, 262_144);
    assert_eq!(small_report.total_requested_world_bytes, 3_407_872);

    let reference = WorldConfig::reference();
    let reference_layout = WorldLayout::new(2048 * 2048).unwrap();
    let reference_report = AllocationReport::from_layout(reference, &reference_layout);
    assert_eq!(reference_report.total_requested_world_bytes, 218_103_808);

    assert_eq!(PASS_COUNT, 30);
    assert_eq!(PASS_NAMES.len(), PASS_COUNT);
    assert_eq!(PASS_NAMES[0], "activity_wake");
    assert_eq!(PASS_NAMES[29], "activity_reduce");
    for required in [
        "environment_reconcile_movement",
        "expansion_environment_receiver_claim",
        "environment_blocked_expansion_pressure",
        "environment_reconcile_expansion",
        "smoke_environment_receiver_claim",
        "environment_reconcile_smoke",
    ] {
        assert!(
            PASS_NAMES.contains(&required),
            "missing profiler pass {required}"
        );
    }
}

#[test]
fn tracked_no_profiler_total_is_exact_at_256_squared() {
    let sim = pollster::block_on(Simulation::new(WorldConfig::new(256, 256, 64).unwrap()))
        .expect("DX12 + RTX 5090 simulation init");
    let report = sim.tracked_memory_report(None);
    assert_eq!(report.environment_state_bytes, 1_048_576);
    assert_eq!(report.environment_receiver_claim_bytes, 262_144);
    assert_eq!(report.total_tracked_gpu_bytes, 4_196_864);
}

#[test]
fn canonical_staging_edits_and_reset_keep_both_halves_exact() {
    let mut sim = eight_by_eight();
    let initial = observe(&sim, &[(0, 0), (3, 3)]);
    assert_eq!(
        initial[0].current,
        AirState {
            mass: 0.0,
            energy: 0.0
        }
    );
    assert_eq!(initial[0].current, initial[0].next);
    assert_eq!(initial[1].current, standard_air_state());
    assert_eq!(initial[1].current, initial[1].next);

    sim.world
        .write_material(&sim.context.queue, 3, 3, MATERIAL_STONE)
        .unwrap();
    let drawn = observe(&sim, &[(3, 3)])[0];
    assert_eq!(
        drawn.current,
        AirState {
            mass: 0.0,
            energy: 0.0
        }
    );
    assert_eq!(drawn.current, drawn.next);

    sim.world
        .write_material(&sim.context.queue, 3, 3, MATERIAL_EMPTY)
        .unwrap();
    let erased = observe(&sim, &[(3, 3)])[0];
    assert_eq!(erased.current, standard_air_state());
    assert_eq!(erased.current, erased.next);

    let residual = AirState {
        mass: 0.25,
        energy: STANDARD_AIR_ENERGY * 0.25,
    };
    sim.world
        .write_environment_cell_for_test(&sim.context.queue, 3, 3, residual)
        .unwrap();
    sim.reset().unwrap();
    let reset = observe(&sim, &[(0, 0), (3, 3)]);
    assert_eq!(
        reset[0].current,
        AirState {
            mass: 0.0,
            energy: 0.0
        }
    );
    assert_eq!(reset[1].current, standard_air_state());
    assert!(reset.iter().all(|cell| cell.current == cell.next));
}

#[test]
fn unchanged_tick_has_no_air_flow_or_thermal_exchange() {
    let mut sim = eight_by_eight();
    let residual = AirState {
        mass: 0.25,
        energy: STANDARD_AIR_ENERGY * 0.25,
    };
    sim.world
        .write_environment_cell_for_test(&sim.context.queue, 3, 3, residual)
        .unwrap();
    sim.world
        .write_material(&sim.context.queue, 4, 3, MATERIAL_STONE)
        .unwrap();
    sim.world
        .write_temperature(&sim.context.queue, 4, 3, 500.0)
        .unwrap();
    let before = observe(&sim, &[(3, 3), (3, 4)]);

    sim.tick().unwrap();

    let after = observe(&sim, &[(3, 3), (3, 4)]);
    assert_eq!(before, after, "TE-1 must not flow or thermally mix Air");
}

#[test]
fn movement_exchanges_the_exact_destination_parcel() {
    let mut sim = eight_by_eight();
    let parcel = AirState {
        mass: 0.25,
        energy: STANDARD_AIR_ENERGY * 0.25,
    };
    sim.world
        .write_material(&sim.context.queue, 3, 2, MATERIAL_SAND)
        .unwrap();
    sim.world
        .write_environment_cell_for_test(&sim.context.queue, 3, 3, parcel)
        .unwrap();

    sim.tick().unwrap();

    assert_eq!(
        sim.world
            .read_material_cell(&sim.context.device, &sim.context.queue, 3, 2)
            .unwrap(),
        MATERIAL_EMPTY
    );
    assert_eq!(
        sim.world
            .read_material_cell(&sim.context.device, &sim.context.queue, 3, 3)
            .unwrap(),
        MATERIAL_SAND
    );
    let cells = observe(&sim, &[(3, 2), (3, 3)]);
    assert_eq!(cells[0].current, parcel);
    assert_eq!(cells[0].current, cells[0].next);
    assert_eq!(
        cells[1].current,
        AirState {
            mass: 0.0,
            energy: 0.0
        }
    );
    assert_eq!(cells[1].current, cells[1].next);
}

#[test]
fn movement_volume_exchange_covers_vacuum_density_void_chunk_edges_and_sleep_modes() {
    // Matter entering Vacuum leaves exact Vacuum behind at its vacated source.
    let mut vacuum = eight_by_eight();
    vacuum
        .world
        .write_material(&vacuum.context.queue, 3, 2, MATERIAL_SAND)
        .unwrap();
    vacuum
        .world
        .write_environment_cell_for_test(&vacuum.context.queue, 3, 3, vacuum_air_state())
        .unwrap();
    vacuum.tick().unwrap();
    let vacuum_cells = observe(&vacuum, &[(3, 2), (3, 3)]);
    assert_eq!(vacuum_cells[0].current, vacuum_air_state());
    assert_eq!(vacuum_cells[1].current, vacuum_air_state());
    assert!(vacuum_cells.iter().all(|cell| cell.current == cell.next));

    // A Matter-for-Matter density swap never creates same-cell Air.
    let mut density = eight_by_eight();
    for y in 1..=6 {
        density
            .world
            .write_material(&density.context.queue, 2, y, MATERIAL_STONE)
            .unwrap();
        density
            .world
            .write_material(&density.context.queue, 4, y, MATERIAL_STONE)
            .unwrap();
    }
    density
        .world
        .write_material(&density.context.queue, 3, 5, MATERIAL_SAND)
        .unwrap();
    density
        .world
        .write_material(&density.context.queue, 3, 6, MATERIAL_WATER)
        .unwrap();
    density.tick().unwrap();
    assert_eq!(
        density
            .world
            .read_material_cell(&density.context.device, &density.context.queue, 3, 6)
            .unwrap(),
        MATERIAL_SAND
    );
    assert_eq!(
        density
            .world
            .read_material_cell(&density.context.device, &density.context.queue, 3, 5)
            .unwrap(),
        MATERIAL_WATER
    );
    let density_cells = observe(&density, &[(3, 5), (3, 6)]);
    assert!(density_cells
        .iter()
        .all(|cell| cell.current == vacuum_air_state() && cell.current == cell.next));

    // A Void exit exposes Vacuum rather than spontaneously refilling Atmosphere.
    let mut void_exit = eight_by_eight();
    void_exit
        .world
        .write_material(&void_exit.context.queue, 4, 7, MATERIAL_EMPTY)
        .unwrap();
    void_exit
        .world
        .write_material(&void_exit.context.queue, 4, 6, MATERIAL_SAND)
        .unwrap();
    void_exit.tick().unwrap();
    void_exit.tick().unwrap();
    let void_cell = observe(&void_exit, &[(4, 7)])[0];
    assert_eq!(void_cell.current, vacuum_air_state());
    assert_eq!(void_cell.current, void_cell.next);

    // Crossing a chunk boundary transfers the exact destination parcel.
    let mut boundary =
        pollster::block_on(Simulation::new(WorldConfig::new(128, 16, 64).unwrap())).unwrap();
    let edge_parcel = AirState {
        mass: 0.75,
        energy: STANDARD_AIR_ENERGY * 0.75,
    };
    boundary
        .world
        .write_material(&boundary.context.queue, 63, 1, MATERIAL_SAND)
        .unwrap();
    boundary
        .world
        .write_material(&boundary.context.queue, 63, 2, MATERIAL_STONE)
        .unwrap();
    boundary
        .world
        .write_material(&boundary.context.queue, 62, 2, MATERIAL_STONE)
        .unwrap();
    boundary
        .world
        .write_environment_cell_for_test(&boundary.context.queue, 64, 2, edge_parcel)
        .unwrap();
    boundary.tick().unwrap();
    let boundary_cells = observe(&boundary, &[(63, 1), (64, 2)]);
    assert_eq!(boundary_cells[0].current, edge_parcel);
    assert_eq!(boundary_cells[1].current, vacuum_air_state());

    // Sleep ON and OFF use the same occupancy-linked Environment result.
    let config = WorldConfig::new(16, 8, 8).unwrap();
    let mut sleeping = pollster::block_on(Simulation::new(config)).unwrap();
    let mut awake = pollster::block_on(Simulation::new(config)).unwrap();
    sleeping.set_sleep_enabled(true);
    sleeping.set_sleep_threshold(4);
    awake.set_sleep_enabled(false);
    for sim in [&sleeping, &awake] {
        sim.world
            .write_material(&sim.context.queue, 7, 2, MATERIAL_SAND)
            .unwrap();
        sim.world
            .write_environment_cell_for_test(&sim.context.queue, 7, 3, edge_parcel)
            .unwrap();
    }
    sleeping.tick().unwrap();
    awake.tick().unwrap();
    assert_eq!(
        observe(&sleeping, &[(7, 2), (7, 3)]),
        observe(&awake, &[(7, 2), (7, 3)])
    );
}

#[test]
fn physical_matter_removal_exposes_vacuum_in_both_halves() {
    let mut sim = eight_by_eight();
    sim.world
        .write_material(&sim.context.queue, 3, 3, MATERIAL_SMOKE)
        .unwrap();
    sim.world
        .write_flags(&sim.context.queue, 3, 3, with_decay_age(0, 899))
        .unwrap();
    for (x, y) in [
        (2, 2),
        (3, 2),
        (4, 2),
        (2, 3),
        (4, 3),
        (2, 4),
        (3, 4),
        (4, 4),
    ] {
        sim.world
            .write_material(&sim.context.queue, x, y, MATERIAL_STONE)
            .unwrap();
    }

    sim.tick().unwrap();

    assert_eq!(
        sim.world
            .read_material_cell(&sim.context.device, &sim.context.queue, 3, 3)
            .unwrap(),
        MATERIAL_EMPTY
    );
    let removed = observe(&sim, &[(3, 3)])[0];
    assert_eq!(removed.current, vacuum_air_state());
    assert_eq!(removed.current, removed.next);
}

#[test]
fn environment_observation_is_strictly_bounded() {
    let sim = eight_by_eight();
    let cells = vec![(1, 1); 65];
    let error = sim
        .world
        .read_environment_cells(&sim.context.device, &sim.context.queue, &cells)
        .unwrap_err();
    assert!(error.to_string().contains("maximum is 64"));
}
