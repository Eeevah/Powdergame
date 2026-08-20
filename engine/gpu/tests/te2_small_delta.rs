//! Production-GPU semantic regression for the TE-2 thermal deadband gate.

use powdergame_core::{
    AirState, WorldConfig, ACTIVITY_THERMAL, AIR_ZERO_OFFSET, CHUNK_STATE_SLEEPING,
    MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_STONE, THERMAL_C_STONE, THERMAL_DEADBAND_C,
};
use powdergame_gpu::Simulation;

#[derive(Clone, Copy, Debug)]
enum PairKind {
    MatterMatter,
    AirAir,
    MatterAir,
}

#[derive(Clone, Copy, Debug)]
struct Pair {
    kind: PairKind,
    baseline: f32,
    requested_delta: f32,
    left: (i64, i64),
    right: (i64, i64),
}

fn cell_index(x: i64, y: i64) -> usize {
    y as usize * 64 + x as usize
}

fn set_air_temperature(sim: &Simulation, x: i64, y: i64, temperature_c: f32) {
    sim.world
        .write_environment_cell_for_test(
            &sim.context.queue,
            x,
            y,
            AirState {
                mass: 1.0,
                energy: temperature_c + AIR_ZERO_OFFSET,
            },
        )
        .unwrap();
}

fn stage_pair(sim: &Simulation, pair: Pair) {
    let (x, y) = pair.left;
    for wall_y in (y - 1)..=(y + 1) {
        for wall_x in (x - 1)..=(x + 2) {
            if (wall_x, wall_y) != pair.left && (wall_x, wall_y) != pair.right {
                sim.world
                    .write_material(&sim.context.queue, wall_x, wall_y, MATERIAL_BOUNDARY_BLOCK)
                    .unwrap();
            }
        }
    }

    let hot = pair.baseline + pair.requested_delta;
    match pair.kind {
        PairKind::MatterMatter => {
            for ((cx, cy), temperature) in [(pair.left, pair.baseline), (pair.right, hot)] {
                sim.world
                    .write_material(&sim.context.queue, cx, cy, MATERIAL_STONE)
                    .unwrap();
                sim.world
                    .write_temperature(&sim.context.queue, cx, cy, temperature)
                    .unwrap();
            }
        }
        PairKind::AirAir => {
            for ((cx, cy), temperature) in [(pair.left, pair.baseline), (pair.right, hot)] {
                sim.world
                    .write_material(&sim.context.queue, cx, cy, MATERIAL_EMPTY)
                    .unwrap();
                set_air_temperature(sim, cx, cy, temperature);
            }
        }
        PairKind::MatterAir => {
            sim.world
                .write_material(&sim.context.queue, pair.left.0, pair.left.1, MATERIAL_STONE)
                .unwrap();
            sim.world
                .write_temperature(&sim.context.queue, pair.left.0, pair.left.1, pair.baseline)
                .unwrap();
            sim.world
                .write_material(
                    &sim.context.queue,
                    pair.right.0,
                    pair.right.1,
                    MATERIAL_EMPTY,
                )
                .unwrap();
            set_air_temperature(sim, pair.right.0, pair.right.1, hot);
        }
    }
}

fn stage_fixture(sim: &Simulation) -> Vec<Pair> {
    let mut pairs = Vec::new();
    let kinds = [
        PairKind::MatterMatter,
        PairKind::AirAir,
        PairKind::MatterAir,
    ];
    let baselines = [20.0f32, 500.0];
    let deltas = [1.0f32, 0.1, 0.02, 0.011, 0.009];
    let mut ordinal = 0usize;
    for kind in kinds {
        for baseline in baselines {
            for requested_delta in deltas {
                let column = ordinal % 5;
                let row = ordinal / 5;
                let pair = Pair {
                    kind,
                    baseline,
                    requested_delta,
                    left: (3 + column as i64 * 12, 3 + row as i64 * 10),
                    right: (4 + column as i64 * 12, 3 + row as i64 * 10),
                };
                stage_pair(sim, pair);
                pairs.push(pair);
                ordinal += 1;
            }
        }
    }
    pairs
}

fn temperatures(sim: &Simulation, pairs: &[Pair]) -> Vec<(f32, f32)> {
    let matter = sim
        .world
        .read_temperature_all(&sim.context.device, &sim.context.queue)
        .unwrap();
    let air_coordinates = pairs
        .iter()
        .flat_map(|pair| [pair.left, pair.right])
        .collect::<Vec<_>>();
    let air = sim
        .world
        .read_environment_cells(&sim.context.device, &sim.context.queue, &air_coordinates)
        .unwrap();
    pairs
        .iter()
        .enumerate()
        .map(|(pair_index, pair)| {
            let value = |endpoint: usize, coordinate: (i64, i64)| match (pair.kind, endpoint) {
                (PairKind::MatterMatter, _) | (PairKind::MatterAir, 0) => {
                    matter[cell_index(coordinate.0, coordinate.1)]
                }
                _ => {
                    let state = air[pair_index * 2 + endpoint].current;
                    state.energy / state.mass - AIR_ZERO_OFFSET
                }
            };
            (value(0, pair.left), value(1, pair.right))
        })
        .collect()
}

fn energy_like(pair: Pair, temperatures: (f32, f32)) -> f32 {
    match pair.kind {
        PairKind::MatterMatter => THERMAL_C_STONE * (temperatures.0 + temperatures.1),
        PairKind::AirAir => temperatures.0 + temperatures.1,
        PairKind::MatterAir => THERMAL_C_STONE * temperatures.0 + temperatures.1,
    }
}

#[test]
fn small_delta_thermal_convergence_matches_the_cpu_semantic_gate() {
    let config = WorldConfig::new(64, 64, 64).unwrap();
    let mut awake = pollster::block_on(Simulation::new(config)).unwrap();
    let mut sleeping = pollster::block_on(Simulation::new(config)).unwrap();
    awake.set_sleep_enabled(false);
    sleeping.set_sleep_enabled(true);
    sleeping.set_sleep_threshold(4);
    let pairs = stage_fixture(&awake);
    let sleeping_pairs = stage_fixture(&sleeping);
    assert_eq!(pairs.len(), sleeping_pairs.len());

    let initial = temperatures(&awake, &pairs);
    let initial_energy = pairs
        .iter()
        .copied()
        .zip(initial.iter().copied())
        .map(|(pair, values)| energy_like(pair, values))
        .collect::<Vec<_>>();
    awake.tick().unwrap();
    sleeping.tick().unwrap();
    let first = temperatures(&awake, &pairs);
    let activity = awake
        .world
        .read_cell_activity_all(&awake.context.device, &awake.context.queue)
        .unwrap();

    for (index, pair) in pairs.iter().enumerate() {
        let initial_delta = initial[index].1 - initial[index].0;
        let should_work = initial_delta.abs() > THERMAL_DEADBAND_C;
        assert_eq!(first[index] != initial[index], should_work, "{pair:?}");
        let pair_activity = activity[cell_index(pair.left.0, pair.left.1)]
            | activity[cell_index(pair.right.0, pair.right.1)];
        assert_eq!(
            pair_activity & ACTIVITY_THERMAL != 0,
            should_work,
            "{pair:?}"
        );
        assert!(first[index].0 >= initial[index].0 - 1.0e-4, "{pair:?}");
        assert!(first[index].1 <= initial[index].1 + 1.0e-4, "{pair:?}");
        assert!(first[index].0 <= first[index].1 + 1.0e-4, "{pair:?}");
    }

    let mut previous = first;
    let mut converged = vec![false; pairs.len()];
    for _ in 0..256 {
        for _ in 0..16 {
            awake.tick().unwrap();
            sleeping.tick().unwrap();
        }
        let current = temperatures(&awake, &pairs);
        let sleeping_current = temperatures(&sleeping, &pairs);
        let sleeping_state = sleeping
            .world
            .read_chunk_state_all(&sleeping.context.device, &sleeping.context.queue)
            .unwrap();
        if sleeping_state == vec![CHUNK_STATE_SLEEPING] {
            assert!(
                sleeping_current
                    .iter()
                    .all(|values| (values.1 - values.0).abs() <= THERMAL_DEADBAND_C + 1.0e-4),
                "sleep must not precede thermal equilibrium"
            );
        }
        let current_activity = awake
            .world
            .read_cell_activity_all(&awake.context.device, &awake.context.queue)
            .unwrap();
        for (index, pair) in pairs.iter().enumerate() {
            assert!(current[index].0 + 1.0e-4 >= previous[index].0, "{pair:?}");
            assert!(current[index].1 <= previous[index].1 + 1.0e-4, "{pair:?}");
            assert!(current[index].0 <= current[index].1 + 1.0e-4, "{pair:?}");
            assert!(
                (energy_like(*pair, current[index]) - initial_energy[index]).abs() <= 2.0e-2,
                "energy-like drift for {pair:?}: initial={} current={}",
                initial_energy[index],
                energy_like(*pair, current[index])
            );
            let delta = (current[index].1 - current[index].0).abs();
            let active = (current_activity[cell_index(pair.left.0, pair.left.1)]
                | current_activity[cell_index(pair.right.0, pair.right.1)])
                & ACTIVITY_THERMAL
                != 0;
            assert_eq!(active, delta > THERMAL_DEADBAND_C, "{pair:?}");
            converged[index] |= delta <= THERMAL_DEADBAND_C;
        }
        previous = current;
        if converged.iter().all(|value| *value) {
            break;
        }
    }
    let unconverged = pairs
        .iter()
        .zip(&previous)
        .zip(&converged)
        .filter_map(|((pair, values), converged)| {
            (!converged).then_some((*pair, values.1 - values.0))
        })
        .collect::<Vec<_>>();
    assert!(unconverged.is_empty(), "unconverged pairs: {unconverged:?}");

    let sleeping_final = temperatures(&sleeping, &pairs);
    for (awake_pair, sleeping_pair) in previous.iter().zip(sleeping_final) {
        assert!((awake_pair.1 - awake_pair.0).abs() <= THERMAL_DEADBAND_C + 1.0e-4);
        assert!((sleeping_pair.1 - sleeping_pair.0).abs() <= THERMAL_DEADBAND_C + 1.0e-4);
    }
}
