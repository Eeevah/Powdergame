//! TE-3 source-bound phase-enthalpy fixtures.

use powdergame_core::{
    normalize_phase_enthalpy, phase_enthalpy, vacuum_air_state, PhaseContext, WorldConfig,
    LATENT_FUSION, LATENT_VAPORIZATION, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_ICE,
    MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER,
};
use powdergame_gpu::Simulation;

fn sim() -> Simulation {
    pollster::block_on(Simulation::new(WorldConfig::new(16, 16, 8).unwrap())).unwrap()
}

fn set(s: &Simulation, x: i64, y: i64, m: u32, t: f32) {
    s.world.write_material(&s.context.queue, x, y, m).unwrap();
    s.world
        .write_temperature(&s.context.queue, x, y, t)
        .unwrap();
}

fn seal(s: &Simulation, x: i64, y: i64, material: u32, t: f32) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx != 0 || dy != 0 {
                set(s, x + dx, y + dy, material, t);
            }
        }
    }
}

fn family_count(s: &Simulation) -> usize {
    s.world
        .read_material_all(&s.context.device, &s.context.queue)
        .unwrap()
        .into_iter()
        .filter(|m| [MATERIAL_ICE, MATERIAL_WATER, MATERIAL_STEAM].contains(m))
        .count()
}

fn find_family(s: &Simulation, material: u32) -> Vec<(usize, f32)> {
    let m = s
        .world
        .read_material_all(&s.context.device, &s.context.queue)
        .unwrap();
    let e = s
        .world
        .read_phase_energy_all(&s.context.device, &s.context.queue)
        .unwrap();
    m.into_iter()
        .zip(e)
        .enumerate()
        .filter_map(|(i, (m, e))| (m == material).then_some((i, e)))
        .collect()
}

#[test]
fn te3_f01_one_hundred_closed_cycles_conserve_quantity_and_h() {
    let gas = PhaseContext {
        gas_facing: true,
        ..Default::default()
    };
    let sink = PhaseContext {
        condensation_sink: true,
        ..Default::default()
    };
    for _ in 0..100 {
        let steam = normalize_phase_enthalpy(MATERIAL_WATER, 292.0, 0.0, gas).unwrap();
        assert_eq!(steam.material, MATERIAL_STEAM);
        assert_eq!(steam.phase_energy, LATENT_VAPORIZATION);
        let partial =
            normalize_phase_enthalpy(steam.material, -250.0, steam.phase_energy, sink).unwrap();
        let water =
            normalize_phase_enthalpy(partial.material, -250.0, partial.phase_energy, sink).unwrap();
        assert_eq!(water.material, MATERIAL_WATER);
        assert_eq!(water.phase_energy, 0.0);
        assert!(phase_enthalpy(steam.material, steam.temperature, steam.phase_energy).is_finite());
    }
}

#[test]
fn te3_f02_partial_boiling_reverses() {
    let gas = PhaseContext {
        gas_facing: true,
        ..Default::default()
    };
    let up = normalize_phase_enthalpy(MATERIAL_WATER, 180.0, 0.0, gas).unwrap();
    assert_eq!(up.material, MATERIAL_WATER);
    assert!(up.phase_energy > 0.0);
    let down = normalize_phase_enthalpy(
        MATERIAL_WATER,
        80.0,
        up.phase_energy,
        PhaseContext::default(),
    )
    .unwrap();
    assert!(down.phase_energy < up.phase_energy);
}

#[test]
fn te3_f03_partial_condensation_reverses() {
    let sink = PhaseContext {
        condensation_sink: true,
        ..Default::default()
    };
    let down = normalize_phase_enthalpy(MATERIAL_STEAM, 60.0, 480.0, sink).unwrap();
    assert!(down.phase_energy < 480.0);
    let up = normalize_phase_enthalpy(
        MATERIAL_STEAM,
        200.0,
        down.phase_energy,
        PhaseContext::default(),
    )
    .unwrap();
    assert!(up.phase_energy >= down.phase_energy);
}

#[test]
fn te3_f04_freeze_melt_reversal() {
    let freezing =
        normalize_phase_enthalpy(MATERIAL_WATER, -20.0, 0.0, PhaseContext::default()).unwrap();
    assert!(freezing.phase_energy < 0.0);
    let melting = normalize_phase_enthalpy(
        MATERIAL_WATER,
        20.0,
        freezing.phase_energy,
        PhaseContext::default(),
    )
    .unwrap();
    assert!(melting.phase_energy > freezing.phase_energy);
}

#[test]
fn te3_f05_surface_boil_buried_hold_and_reopen() {
    let mut s = sim();
    seal(&s, 4, 8, MATERIAL_STONE, 300.0);
    set(&s, 4, 7, MATERIAL_EMPTY, 20.0);
    set(&s, 4, 8, MATERIAL_WATER, 300.0);
    seal(&s, 11, 8, MATERIAL_STONE, 300.0);
    set(&s, 11, 8, MATERIAL_WATER, 300.0);
    s.tick().unwrap();
    assert_eq!(find_family(&s, MATERIAL_STEAM).len(), 1);
    assert_eq!(find_family(&s, MATERIAL_WATER).len(), 1);
    set(&s, 11, 7, MATERIAL_EMPTY, 20.0);
    s.tick().unwrap();
    assert_eq!(family_count(&s), 2);
    assert_eq!(find_family(&s, MATERIAL_STEAM).len(), 2);
}

#[test]
fn te3_f05b_f05c_buried_partial_reversal_and_ready_hold() {
    let mut s = sim();
    seal(&s, 8, 8, MATERIAL_STONE, 120.0);
    set(&s, 8, 8, MATERIAL_WATER, 120.0);
    s.world
        .write_phase_energy(&s.context.queue, 8, 8, MATERIAL_WATER, 480.0)
        .unwrap();
    s.tick().unwrap();
    assert_eq!(find_family(&s, MATERIAL_WATER).len(), 1);
    let e = s
        .world
        .read_phase_energy_cell(&s.context.device, &s.context.queue, 8, 8)
        .unwrap();
    assert!(e <= 480.0 && e > 0.0);
}

#[test]
fn te3_f06_cold_lid_condenses_but_zero_k_boundary_does_not() {
    let mut cold = sim();
    seal(&cold, 8, 8, MATERIAL_STONE, 60.0);
    set(&cold, 8, 7, MATERIAL_STONE, 0.0);
    set(&cold, 8, 8, MATERIAL_STEAM, 60.0);
    cold.tick().unwrap();
    assert!(find_family(&cold, MATERIAL_STEAM)[0].1 < 480.0);

    let mut boundary = sim();
    seal(&boundary, 8, 8, MATERIAL_BOUNDARY_BLOCK, 0.0);
    set(&boundary, 8, 8, MATERIAL_STEAM, 60.0);
    boundary.tick().unwrap();
    assert_eq!(find_family(&boundary, MATERIAL_STEAM)[0].1, 480.0);
}

#[test]
fn te3_f07_free_air_seed_is_sparse_and_partial_veto_is_multitick() {
    let mut s = sim();
    for y in 6..10 {
        for x in 4..12 {
            set(&s, x, y, MATERIAL_STEAM, 60.0);
        }
    }
    s.tick().unwrap();
    let first = find_family(&s, MATERIAL_STEAM);
    let partial = first.iter().filter(|(_, e)| *e > 0.0 && *e < 480.0).count();
    assert!(partial > 0 && partial < first.len());
    for _ in 0..3 {
        s.tick().unwrap();
    }
    assert_eq!(family_count(&s), 32);
}

#[test]
fn te3_f07b_isolated_supercooled_steam_is_metastable() {
    let mut s = sim();
    seal(&s, 8, 8, MATERIAL_BOUNDARY_BLOCK, -100.0);
    set(&s, 8, 8, MATERIAL_STEAM, 60.0);
    s.world
        .write_environment_cell_for_test(&s.context.queue, 8, 8, vacuum_air_state())
        .unwrap();
    for _ in 0..4 {
        s.tick().unwrap();
    }
    assert_eq!(find_family(&s, MATERIAL_STEAM)[0].1, 480.0);
}

#[test]
fn te3_f08_real_grid_does_not_create_persistent_checkerboard() {
    let mut s = sim();
    for y in 5..11 {
        for x in 3..13 {
            set(&s, x, y, MATERIAL_STEAM, 60.0);
        }
    }
    let mut peak_edges = 0usize;
    let mut final_edges = 0usize;
    for tick in 0..30 {
        s.tick().unwrap();
        let m = s
            .world
            .read_material_all(&s.context.device, &s.context.queue)
            .unwrap();
        let mut edges = 0;
        for y in 1..15 {
            for x in 1..15 {
                let i = y * 16 + x;
                if m[i] == MATERIAL_WATER {
                    if m[i - 1] == MATERIAL_STEAM {
                        edges += 1;
                    }
                    if m[i - 16] == MATERIAL_STEAM {
                        edges += 1;
                    }
                }
            }
        }
        peak_edges = peak_edges.max(edges);
        if tick == 29 {
            final_edges = edges;
        }
    }
    assert_eq!(family_count(&s), 60);
    assert!(final_edges <= peak_edges);
}

#[test]
fn te3_f09_open_beaker_trace_uses_real_phase_and_movement() {
    let mut s = sim();
    for x in 4..12 {
        set(&s, x, 12, MATERIAL_STONE, 20.0);
    }
    for y in 8..12 {
        set(&s, 4, y, MATERIAL_STONE, 20.0);
        set(&s, 11, y, MATERIAL_STONE, 20.0);
    }
    for x in 5..11 {
        set(&s, x, 11, MATERIAL_WATER, 300.0);
    }
    for _ in 0..4 {
        s.tick().unwrap();
    }
    assert_eq!(family_count(&s), 6);
    assert!(!find_family(&s, MATERIAL_STEAM).is_empty());
}

#[test]
fn te3_f10_sealed_vessel_has_zero_phase_pressure_and_no_extra_matter() {
    let mut s = sim();
    seal(&s, 8, 8, MATERIAL_STONE, 300.0);
    set(&s, 8, 8, MATERIAL_WATER, 300.0);
    let before = family_count(&s);
    s.tick().unwrap();
    assert_eq!(family_count(&s), before);
    assert_eq!(
        s.world
            .read_pressure_cell(&s.context.device, &s.context.queue, 8, 8)
            .unwrap(),
        0.0
    );
}

#[test]
fn te3_f11_reset_and_staging_are_canonical() {
    let mut s = sim();
    set(&s, 8, 8, MATERIAL_ICE, -10.0);
    assert_eq!(
        s.world
            .read_phase_energy_cell(&s.context.device, &s.context.queue, 8, 8)
            .unwrap(),
        -LATENT_FUSION
    );
    set(&s, 8, 8, MATERIAL_STEAM, 120.0);
    assert_eq!(
        s.world
            .read_phase_energy_cell(&s.context.device, &s.context.queue, 8, 8)
            .unwrap(),
        LATENT_VAPORIZATION
    );
    s.reset().unwrap();
    assert!(s
        .world
        .read_phase_energy_all(&s.context.device, &s.context.queue)
        .unwrap()
        .iter()
        .all(|e| *e == 0.0));
}

#[test]
fn te3_f12_phase_energy_moves_with_owner() {
    let mut s = sim();
    set(&s, 8, 6, MATERIAL_WATER, 100.0);
    s.world
        .write_phase_energy(&s.context.queue, 8, 6, MATERIAL_WATER, 200.0)
        .unwrap();
    s.tick().unwrap();
    let waters = find_family(&s, MATERIAL_WATER);
    assert_eq!(waters.len(), 1);
    assert!(waters[0].1 > 0.0);
}

#[test]
fn te3_f13_sleep_on_off_matches_for_equal_ticks() {
    let mut a = sim();
    let mut b = sim();
    a.set_sleep_enabled(false);
    b.set_sleep_enabled(true);
    for s in [&a, &b] {
        set(s, 8, 8, MATERIAL_STEAM, 60.0);
    }
    for _ in 0..4 {
        a.tick().unwrap();
        b.tick().unwrap();
    }
    assert_eq!(
        a.world
            .read_material_all(&a.context.device, &a.context.queue)
            .unwrap(),
        b.world
            .read_material_all(&b.context.device, &b.context.queue)
            .unwrap()
    );
    assert_eq!(
        a.world
            .read_phase_energy_all(&a.context.device, &a.context.queue)
            .unwrap(),
        b.world
            .read_phase_energy_all(&b.context.device, &b.context.queue)
            .unwrap()
    );
}

#[test]
fn te3_f14_cpu_gpu_semantics_agree_for_surface_boiling() {
    let expected = normalize_phase_enthalpy(
        MATERIAL_WATER,
        300.0,
        0.0,
        PhaseContext {
            gas_facing: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut s = sim();
    seal(&s, 8, 8, MATERIAL_STONE, 300.0);
    set(&s, 8, 7, MATERIAL_EMPTY, 20.0);
    set(&s, 8, 8, MATERIAL_WATER, 300.0);
    s.tick().unwrap();
    assert_eq!(find_family(&s, expected.material).len(), 1);
}

#[test]
fn te3_f15_water_never_spawns_or_sources_blocked_pressure() {
    let mut s = sim();
    seal(&s, 8, 8, MATERIAL_STONE, 300.0);
    set(&s, 8, 7, MATERIAL_EMPTY, 20.0);
    set(&s, 8, 8, MATERIAL_WATER, 300.0);
    let before = family_count(&s);
    s.tick().unwrap();
    assert_eq!(family_count(&s), before);
    assert_eq!(find_family(&s, MATERIAL_STEAM).len(), 1);
    assert!(s
        .world
        .read_pressure_all(&s.context.device, &s.context.queue)
        .unwrap()
        .iter()
        .all(|p| *p == 0.0));
}
