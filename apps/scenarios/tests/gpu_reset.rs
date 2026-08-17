use powdergame_core::{
    fuel_progress, is_valid_cell_material_value, WorldConfig, ACTIVITY_ALL_BITS, ACTIVITY_MATTER,
    ACTIVITY_PRESSURE, ACTIVITY_REACTION, ACTIVITY_THERMAL, CHUNK_STATE_RUNNABLE,
    CHUNK_STATE_SLEEPING, FLAG_FLAME_EVENT, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_OIL,
    MATERIAL_SMOKE, MATERIAL_WATER, MATERIAL_WOOD, PRESSURE_REFERENCE, TEMPERATURE_REFERENCE,
    WAKE_REASON_ALWAYS_ACTIVE, WAKE_REASON_NEIGHBOR_HALO, WAKE_REASON_NONE,
    WAKE_REASON_SELF_ACTIVITY, WAKE_REASON_SETTLING, WAKE_REASON_USER_EDIT,
};
use powdergame_gpu::Simulation;
use powdergame_scenarios::{
    reset_and_stage_scenario, ScenarioFixture, ScenarioId, GALLERY_SCENARIOS,
    WATER_FLOW_OUTER_BASIN_MAX_X_EXCLUSIVE, WATER_FLOW_OUTER_BASIN_MAX_Y_EXCLUSIVE,
    WATER_FLOW_OUTER_BASIN_MIN_X, WATER_FLOW_OUTER_BASIN_MIN_Y,
};

#[derive(Debug, PartialEq)]
struct Snapshot {
    materials: Vec<u32>,
    temperatures: Vec<f32>,
    pressures: Vec<f32>,
    flags: Vec<u32>,
    cell_activity: Vec<u32>,
    chunk_activity: Vec<u32>,
    chunk_changed: Vec<u32>,
    chunk_stable: Vec<u32>,
    chunk_state: Vec<u32>,
    chunk_wake_reason: Vec<u32>,
    chunk_edit_wake: Vec<u32>,
}

fn snapshot(simulation: &Simulation) -> Snapshot {
    let world = &simulation.world;
    let device = &simulation.context.device;
    let queue = &simulation.context.queue;
    Snapshot {
        materials: world.read_material_all(device, queue).expect("materials"),
        temperatures: world
            .read_temperature_all(device, queue)
            .expect("temperatures"),
        pressures: world.read_pressure_all(device, queue).expect("pressures"),
        flags: world.read_flags_all(device, queue).expect("flags"),
        cell_activity: world
            .read_cell_activity_all(device, queue)
            .expect("cell activity"),
        chunk_activity: world
            .read_chunk_activity_all(device, queue)
            .expect("chunk activity"),
        chunk_changed: world
            .read_chunk_changed_all(device, queue)
            .expect("chunk changed"),
        chunk_stable: world
            .read_chunk_stable_all(device, queue)
            .expect("chunk stable"),
        chunk_state: world
            .read_chunk_state_all(device, queue)
            .expect("chunk state"),
        chunk_wake_reason: world
            .read_chunk_wake_reason_all(device, queue)
            .expect("chunk wake reason"),
        chunk_edit_wake: world
            .read_chunk_edit_wake_all(device, queue)
            .expect("chunk edit wake"),
    }
}

fn assert_tick_zero_matches_fixture(
    scenario: ScenarioId,
    fixture: &ScenarioFixture,
    actual: &Snapshot,
) {
    assert_eq!(
        actual.materials,
        fixture.materials(),
        "{scenario} materials"
    );
    assert_eq!(
        actual.temperatures,
        fixture.temperatures(),
        "{scenario} temperatures"
    );
    assert_eq!(
        actual.pressures,
        fixture.pressures(),
        "{scenario} pressures"
    );
    assert_eq!(actual.flags, fixture.flags(), "{scenario} flags");
    assert_eq!(
        actual.chunk_edit_wake,
        fixture.chunk_edit_wake(),
        "{scenario} edit wake"
    );
    assert!(actual.cell_activity.iter().all(|value| *value == 0));
    assert!(actual.chunk_activity.iter().all(|value| *value == 0));
    assert!(actual.chunk_changed.iter().all(|value| *value == 0));
    assert!(actual.chunk_stable.iter().all(|value| *value == 0));
    assert!(actual
        .chunk_state
        .iter()
        .all(|value| *value == CHUNK_STATE_RUNNABLE));
    assert!(actual
        .chunk_wake_reason
        .iter()
        .all(|value| *value == WAKE_REASON_NONE));
}

fn assert_state_integrity(scenario: ScenarioId, config: WorldConfig, state: &Snapshot) {
    let width = config.width as usize;
    let height = config.height as usize;
    assert_eq!(state.materials.len(), width * height);
    for index in 0..state.materials.len() {
        let material = state.materials[index];
        assert!(
            is_valid_cell_material_value(material),
            "{scenario}: invalid material {material} at {index}"
        );
        assert!(
            state.temperatures[index].is_finite(),
            "{scenario}: non-finite temperature at {index}"
        );
        assert!(
            state.pressures[index].is_finite(),
            "{scenario}: non-finite pressure at {index}"
        );
        if material == MATERIAL_EMPTY {
            assert_eq!(
                state.temperatures[index].to_bits(),
                TEMPERATURE_REFERENCE.to_bits(),
                "{scenario}: EMPTY temperature hygiene at {index}"
            );
            assert_eq!(
                state.pressures[index].to_bits(),
                PRESSURE_REFERENCE.to_bits(),
                "{scenario}: EMPTY pressure hygiene at {index}"
            );
            assert_eq!(state.flags[index], 0, "{scenario}: EMPTY flags at {index}");
        }
    }
    for x in 0..width {
        assert_eq!(state.materials[x], MATERIAL_BOUNDARY_BLOCK);
        assert_eq!(
            state.materials[(height - 1) * width + x],
            MATERIAL_BOUNDARY_BLOCK
        );
    }
    for y in 0..height {
        assert_eq!(state.materials[y * width], MATERIAL_BOUNDARY_BLOCK);
        assert_eq!(
            state.materials[y * width + width - 1],
            MATERIAL_BOUNDARY_BLOCK
        );
    }

    assert!(state
        .cell_activity
        .iter()
        .all(|value| value & !ACTIVITY_ALL_BITS == 0));
    assert!(state
        .chunk_activity
        .iter()
        .all(|value| value & !ACTIVITY_ALL_BITS == 0));
    assert!(state.chunk_changed.iter().all(|value| *value <= 1));
    assert!(state
        .chunk_state
        .iter()
        .all(|value| matches!(*value, CHUNK_STATE_RUNNABLE | CHUNK_STATE_SLEEPING)));
    assert!(state.chunk_edit_wake.iter().all(|value| *value <= 1));

    let wake_mask = WAKE_REASON_SELF_ACTIVITY
        | WAKE_REASON_NEIGHBOR_HALO
        | WAKE_REASON_USER_EDIT
        | WAKE_REASON_SETTLING
        | WAKE_REASON_ALWAYS_ACTIVE;
    assert!(state
        .chunk_wake_reason
        .iter()
        .all(|value| value & !wake_mask == 0));
}

fn assert_named_activity(scenario: ScenarioId, state: &Snapshot) {
    let combined = state
        .cell_activity
        .iter()
        .copied()
        .fold(0u32, |mask, value| mask | value);
    let expected = match scenario {
        ScenarioId::SandFall | ScenarioId::WaterFlow => ACTIVITY_MATTER,
        ScenarioId::FireHeat => ACTIVITY_THERMAL | ACTIVITY_REACTION,
        ScenarioId::PressureBurst => ACTIVITY_PRESSURE,
        ScenarioId::HeavyMixedWorld | ScenarioId::ActiveSleepG7 => ACTIVITY_ALL_BITS,
    };
    assert_ne!(
        combined & expected,
        0,
        "{scenario}: bounded tick produced none of the expected activity bits 0x{expected:x}"
    );
}

/// One bounded GPU regression covers all six shared fixtures. This is a
/// staging/reset correctness check, not a performance or long-run smoke matrix.
#[test]
fn all_six_scenarios_reset_exactly_and_survive_one_production_tick() {
    let config = WorldConfig::new(256, 256, 64).expect("test config");
    let mut simulation = match pollster::block_on(Simulation::new(config)) {
        Ok(simulation) => simulation,
        Err(error) => {
            eprintln!("Skipping shared scenario GPU test (GPU unavailable): {error}");
            return;
        }
    };
    simulation.set_sleep_enabled(true);
    simulation.set_sleep_threshold(7);

    for scenario in GALLERY_SCENARIOS {
        let fixture = ScenarioFixture::build(scenario, config).expect("build fixture");
        reset_and_stage_scenario(&mut simulation, scenario).expect("initial staging");
        assert_eq!(simulation.tick_count, 0);
        let baseline = snapshot(&simulation);
        assert_tick_zero_matches_fixture(scenario, &fixture, &baseline);

        simulation.tick().expect("divergence tick 1");
        simulation.tick().expect("divergence tick 2");
        assert_eq!(simulation.tick_count, 2);

        reset_and_stage_scenario(&mut simulation, scenario).expect("reset staging");
        assert_eq!(simulation.tick_count, 0);
        assert!(simulation.sleep_enabled);
        assert_eq!(simulation.sleep_threshold, 7);
        assert_eq!(
            snapshot(&simulation),
            baseline,
            "{scenario}: reset baseline"
        );

        simulation.tick().expect("first tick A");
        assert_eq!(simulation.tick_count, 1);
        let first_tick = snapshot(&simulation);
        assert_state_integrity(scenario, config, &first_tick);
        assert_named_activity(scenario, &first_tick);

        reset_and_stage_scenario(&mut simulation, scenario).expect("second reset staging");
        simulation.tick().expect("first tick B");
        assert_eq!(
            snapshot(&simulation),
            first_tick,
            "{scenario}: tick-1 result after repeated reset"
        );
    }
}

/// A single bounded Water Flow acceptance check. Intermediate observations use
/// material-only readback at the Harness cadence; full state is read only at
/// tick 0, tick 256, and reset.
#[test]
fn water_flow_reaches_destination_with_conserved_finite_matter_and_resets_exactly() {
    const MAX_TICKS: u64 = 256;
    const DESTINATION_MIN_Y: u32 = 200;

    let scenario = ScenarioId::WaterFlow;
    let config = WorldConfig::new(256, 256, 64).expect("test config");
    let mut simulation = match pollster::block_on(Simulation::new(config)) {
        Ok(simulation) => simulation,
        Err(error) => {
            eprintln!("Skipping bounded Water Flow GPU test (GPU unavailable): {error}");
            return;
        }
    };
    simulation.set_sleep_enabled(true);
    simulation.set_sleep_threshold(7);

    let fixture = ScenarioFixture::build(scenario, config).expect("build Water Flow fixture");
    reset_and_stage_scenario(&mut simulation, scenario).expect("initial Water Flow staging");
    assert_eq!(simulation.tick_count, 0);
    let baseline = snapshot(&simulation);
    assert_tick_zero_matches_fixture(scenario, &fixture, &baseline);

    let count_material =
        |materials: &[u32], material| materials.iter().filter(|&&value| value == material).count();
    let count_matter = |materials: &[u32]| {
        materials
            .iter()
            .filter(|&&material| material != MATERIAL_EMPTY)
            .count()
    };
    let baseline_water = count_material(&baseline.materials, MATERIAL_WATER);
    let baseline_oil = count_material(&baseline.materials, MATERIAL_OIL);
    let baseline_matter = count_matter(&baseline.materials);
    assert_eq!(baseline_water, 15_244);
    assert_eq!(baseline_oil, 2_240);

    let width = config.width as usize;
    let destination_empty_mask: Vec<bool> = baseline
        .materials
        .iter()
        .enumerate()
        .map(|(index, &material)| {
            let x = (index % width) as u32;
            let y = (index / width) as u32;
            material == MATERIAL_EMPTY
                && (WATER_FLOW_OUTER_BASIN_MIN_X..WATER_FLOW_OUTER_BASIN_MAX_X_EXCLUSIVE)
                    .contains(&x)
                && (DESTINATION_MIN_Y..WATER_FLOW_OUTER_BASIN_MAX_Y_EXCLUSIVE).contains(&y)
        })
        .collect();
    let count_water_outside_basin = |materials: &[u32]| {
        materials
            .iter()
            .enumerate()
            .filter(|&(_, material)| *material == MATERIAL_WATER)
            .filter(|&(index, _)| {
                let x = (index % width) as u32;
                let y = (index / width) as u32;
                !(WATER_FLOW_OUTER_BASIN_MIN_X..WATER_FLOW_OUTER_BASIN_MAX_X_EXCLUSIVE).contains(&x)
                    || !(WATER_FLOW_OUTER_BASIN_MIN_Y..WATER_FLOW_OUTER_BASIN_MAX_Y_EXCLUSIVE)
                        .contains(&y)
            })
            .count()
    };

    let mut first_destination_arrival_tick = None;
    for expected_tick in 1..=MAX_TICKS {
        simulation.tick().expect("bounded Water Flow tick");
        if expected_tick <= 2 || expected_tick.is_multiple_of(8) {
            let materials = simulation
                .world
                .read_material_all(&simulation.context.device, &simulation.context.queue)
                .expect("Water Flow material-only observation");
            assert_eq!(
                count_material(&materials, MATERIAL_WATER),
                baseline_water,
                "Water count at observation tick {expected_tick}"
            );
            assert_eq!(
                count_material(&materials, MATERIAL_OIL),
                baseline_oil,
                "Oil count at observation tick {expected_tick}"
            );
            assert_eq!(
                count_matter(&materials),
                baseline_matter,
                "non-empty matter count at observation tick {expected_tick}"
            );
            assert_eq!(
                count_water_outside_basin(&materials),
                0,
                "Water outside shared outer basin at observation tick {expected_tick}"
            );

            let destination_water = destination_empty_mask
                .iter()
                .zip(&materials)
                .filter(|(was_empty, material)| **was_empty && **material == MATERIAL_WATER)
                .count();
            if destination_water > 0 && first_destination_arrival_tick.is_none() {
                first_destination_arrival_tick = Some(expected_tick);
            }
        }
    }
    assert_eq!(simulation.tick_count, MAX_TICKS);
    assert!(
        first_destination_arrival_tick.is_some(),
        "Water did not reach a tick-0 EMPTY destination cell in [18,238)x[200,230) at Harness observation cadence within {MAX_TICKS} ticks"
    );

    let after = snapshot(&simulation);
    assert_state_integrity(scenario, config, &after);

    reset_and_stage_scenario(&mut simulation, scenario).expect("Water Flow reset staging");
    assert_eq!(simulation.tick_count, 0);
    assert!(simulation.sleep_enabled);
    assert_eq!(simulation.sleep_threshold, 7);
    let reset = snapshot(&simulation);
    assert_tick_zero_matches_fixture(scenario, &fixture, &reset);
    assert_eq!(reset, baseline, "Water Flow exact pristine reset");
}

/// A small Fire / Heat production-path check. It authenticates that the
/// unchanged authored seed reaches the generic combustion and Smoke paths;
/// long-run reaction termination, phase work, and thermal-tail acceptance
/// remain evidence-Harness responsibilities rather than test thresholds.
#[test]
fn fire_heat_seed_progresses_wood_and_oil_and_generates_smoke_then_resets_exactly() {
    const MAX_TICKS: u64 = 64;

    let scenario = ScenarioId::FireHeat;
    let config = WorldConfig::new(256, 256, 64).expect("test config");
    let mut simulation = match pollster::block_on(Simulation::new(config)) {
        Ok(simulation) => simulation,
        Err(error) => {
            eprintln!("Skipping bounded Fire / Heat GPU test (GPU unavailable): {error}");
            return;
        }
    };
    simulation.set_sleep_enabled(true);
    simulation.set_sleep_threshold(7);

    let fixture = ScenarioFixture::build(scenario, config).expect("build Fire / Heat fixture");
    reset_and_stage_scenario(&mut simulation, scenario).expect("initial Fire / Heat staging");
    let baseline = snapshot(&simulation);
    assert_tick_zero_matches_fixture(scenario, &fixture, &baseline);

    for _ in 0..MAX_TICKS {
        simulation.tick().expect("bounded Fire / Heat tick");
    }
    assert_eq!(simulation.tick_count, MAX_TICKS);
    let after = snapshot(&simulation);
    assert_state_integrity(scenario, config, &after);

    let progressed = |material| {
        after
            .materials
            .iter()
            .zip(&after.flags)
            .filter(|&(cell_material, flags)| {
                *cell_material == material && fuel_progress(*flags) != 0
            })
            .count()
    };
    let flame_events = after
        .materials
        .iter()
        .zip(&after.flags)
        .filter(|&(material, flags)| {
            matches!(*material, MATERIAL_WOOD | MATERIAL_OIL) && *flags & FLAG_FLAME_EVENT != 0
        })
        .count();
    let smoke = after
        .materials
        .iter()
        .filter(|&&material| material == MATERIAL_SMOKE)
        .count();
    assert!(progressed(MATERIAL_WOOD) > 0, "Wood fuel did not progress");
    assert!(progressed(MATERIAL_OIL) > 0, "Oil fuel did not progress");
    assert!(
        flame_events > 0,
        "no production flame event remained observable"
    );
    assert!(smoke > 0, "no Smoke was generated within {MAX_TICKS} ticks");
    assert!(after
        .cell_activity
        .iter()
        .any(|activity| activity & ACTIVITY_REACTION != 0));
    assert!(after
        .cell_activity
        .iter()
        .any(|activity| activity & ACTIVITY_THERMAL != 0));

    reset_and_stage_scenario(&mut simulation, scenario).expect("Fire / Heat reset staging");
    assert_eq!(simulation.tick_count, 0);
    assert!(simulation.sleep_enabled);
    assert_eq!(simulation.sleep_threshold, 7);
    assert_eq!(
        snapshot(&simulation),
        baseline,
        "Fire / Heat exact pristine reset"
    );
}
