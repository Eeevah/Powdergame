//! TE-5R1 — total-pressure differential rupture GPU integration tests.
//!
//! Requires Windows + RTX 5090 + DX12. Finite-strength Matter reads opposing
//! total-pressure faces and becomes EMPTY at its descriptor threshold.

use powdergame_core::{
    vacuum_air_state, WorldConfig, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_STEAM,
    MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD, PRESSURE_REFERENCE, WOOD_RUPTURE_THRESHOLD,
};
use powdergame_gpu::Simulation;

fn make_sim(config: WorldConfig) -> Simulation {
    pollster::block_on(Simulation::new(config)).expect("DX12 + RTX 5090 simulation init")
}

fn eight_by_eight() -> Simulation {
    make_sim(WorldConfig::new(8, 8, 8).unwrap())
}

fn two_hundred_fifty_six() -> Simulation {
    make_sim(WorldConfig::new(256, 256, 64).unwrap())
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
        set(sim, x, y, MATERIAL_STONE);
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
    set_p(&sim, 3, 3, WOOD_RUPTURE_THRESHOLD + 20.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 2), MATERIAL_EMPTY, "weak wall opened");
    assert_eq!(
        cell(&sim, 3, 3),
        MATERIAL_WATER,
        "pressure stress alone does not transmute the medium"
    );
    let opened = sim
        .world
        .read_environment_cells(&sim.context.device, &sim.context.queue, &[(3, 2)])
        .unwrap()[0];
    assert_eq!(opened.current, vacuum_air_state());
    assert_eq!(opened.current, opened.next);
}

#[test]
fn wood_survives_uniform_pressure_on_opposing_faces() {
    let mut sim = eight_by_eight();
    set(&sim, 3, 2, MATERIAL_WOOD);
    set(&sim, 3, 1, MATERIAL_WATER);
    set(&sim, 3, 3, MATERIAL_WATER);
    for (x, y) in [
        (2, 0),
        (3, 0),
        (4, 0),
        (2, 1),
        (4, 1),
        (2, 3),
        (4, 3),
        (2, 4),
        (3, 4),
        (4, 4),
    ] {
        set(&sim, x, y, MATERIAL_STONE);
    }
    set_p(&sim, 3, 1, WOOD_RUPTURE_THRESHOLD + 20.0);
    set_p(&sim, 3, 3, WOOD_RUPTURE_THRESHOLD + 20.0);

    sim.tick().expect("uniform-pressure tick");

    assert_eq!(cell(&sim, 3, 2), MATERIAL_WOOD);
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
    for (x, y) in [
        (62, 7),
        (63, 7),
        (64, 7),
        (62, 8),
        (62, 9),
        (63, 9),
        (64, 9),
    ] {
        set(&sim, x, y, MATERIAL_STONE);
    }
    set_p(&sim, 63, 8, WOOD_RUPTURE_THRESHOLD + 20.0);

    sim.tick().expect("tick");

    assert_eq!(
        cell(&sim, 64, 8),
        MATERIAL_EMPTY,
        "chunk edge is not a stress wall"
    );
}

#[test]
#[ignore = "historical G5 Water-yield-2 pressure chain; D-024 active Water boiling has zero pressure"]
fn blocked_boiling_ruptures_weak_wall_then_vents_on_following_tick() {
    let mut sim = eight_by_eight();
    // One weak top wall; every other 8-neighbor is occupied so G5-B cannot
    // satisfy Water→Steam yield=2. Above the weak wall is ordinary EMPTY.
    set(&sim, 3, 3, MATERIAL_WATER);
    set_t(&sim, 3, 3, 120.0);
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

/// Unified staging configuration for boiler stress experiment chambers.
struct BoilerStagingConfig {
    x0: i64,
    x1: i64,
    roof_y: i64,
    bottom_y: i64,
    floor_heater_rows: i64,
    floor_heater_temp: f32,
    upper_heater_temp: f32,
    water_temp: f32,
    roof_relief: Option<(i64, i64)>, // (plug_left, plug_right)
    side_seam: Option<(i64, i64)>,   // (seam_top, seam_bottom) on right wall x1
    chimney_rails: bool,
    exhaust_duct: bool,
}

fn stage_test_boiler(sim: &Simulation, cfg: &BoilerStagingConfig, stone: u32) {
    // 1. Left Wall (Stone)
    for y in cfg.roof_y..=cfg.bottom_y {
        set(sim, cfg.x0, y, stone);
    }

    // 2. Right Wall (Stone or Weak Seam)
    for y in cfg.roof_y..=(cfg.bottom_y - cfg.floor_heater_rows) {
        if let Some((s_top, s_bot)) = cfg.side_seam {
            if y >= s_top && y <= s_bot {
                set(sim, cfg.x1, y, MATERIAL_WOOD);
                set_t(sim, cfg.x1, y, 20.0);
                continue;
            }
        }
        set(sim, cfg.x1, y, stone);
    }

    // 3. Floor Heaters
    for y in (cfg.bottom_y - cfg.floor_heater_rows + 1)..=cfg.bottom_y {
        for x in cfg.x0..=cfg.x1 {
            set(sim, x, y, stone);
            set_t(sim, x, y, cfg.floor_heater_temp);
        }
    }

    // 4. Roof (Stone or Roof Relief Plug)
    for x in (cfg.x0 + 1)..cfg.x1 {
        let is_plug = if let Some((p_l, p_r)) = cfg.roof_relief {
            x >= p_l && x <= p_r
        } else {
            false
        };
        let mat = if is_plug { MATERIAL_WOOD } else { stone };
        set(sim, x, cfg.roof_y, mat);
        set_t(sim, x, cfg.roof_y, 20.0);
    }

    // 5. Interior Water Fill
    for y in (cfg.roof_y + 1)..(cfg.bottom_y - cfg.floor_heater_rows + 1) {
        for x in (cfg.x0 + 1)..cfg.x1 {
            set(sim, x, y, MATERIAL_WATER);
            set_t(sim, x, y, cfg.water_temp);
        }
    }

    // 6. Upper Heater Plate (centered in chamber, 6 cells below roof)
    let center_x = (cfg.x0 + cfg.x1) / 2;
    let heater_y = cfg.roof_y + 6;
    for x in (center_x - 6)..=(center_x + 6) {
        set(sim, x, heater_y, stone);
        set_t(sim, x, heater_y, cfg.upper_heater_temp);
    }

    // 7. Optional Top Chimney Rails
    if cfg.chimney_rails {
        let chimney_top = if cfg.roof_y < 100 { 8i64 } else { 130i64 };
        for y in chimney_top..cfg.roof_y {
            set(sim, center_x - 6, y, stone);
            set(sim, center_x + 6, y, stone);
        }
    }

    // 8. Optional Side Exhaust Duct
    if cfg.exhaust_duct {
        if let Some((s_top, s_bot)) = cfg.side_seam {
            for y in (s_top - 4)..=(s_bot + 4) {
                for x in (cfg.x1 + 1)..=(cfg.x1 + 10) {
                    if y == s_top - 4 || y == s_bot + 4 {
                        set(sim, x, y, stone);
                    } else {
                        set(sim, x, y, MATERIAL_EMPTY);
                    }
                }
            }
        }
    }
}

fn stage_test_2x2_world(sim: &Simulation) {
    let stone = MATERIAL_STONE;

    // Central dividers
    for y in 4..=250 {
        set(sim, 126, y, stone);
        set(sim, 127, y, stone);
        set(sim, 128, y, stone);
        set(sim, 129, y, stone);
    }
    for x in 4..=251 {
        for y in 118..=124 {
            set(sim, x, y, stone);
        }
    }
    // Panel A: Top-Left (Standard Wood Relief)
    stage_test_boiler(
        sim,
        &BoilerStagingConfig {
            x0: 14,
            x1: 114,
            roof_y: 44,
            bottom_y: 108,
            floor_heater_rows: 1,
            floor_heater_temp: 190.0,
            upper_heater_temp: 150.0,
            water_temp: 98.0,
            roof_relief: Some((60, 68)),
            side_seam: None,
            chimney_rails: true,
            exhaust_duct: false,
        },
        stone,
    );

    // Panel B: Top-Right (Standard Stone Sealed Control)
    stage_test_boiler(
        sim,
        &BoilerStagingConfig {
            x0: 142,
            x1: 242,
            roof_y: 44,
            bottom_y: 108,
            floor_heater_rows: 1,
            floor_heater_temp: 190.0,
            upper_heater_temp: 150.0,
            water_temp: 98.0,
            roof_relief: None,
            side_seam: None,
            chimney_rails: true,
            exhaust_duct: false,
        },
        stone,
    );

    // Panel C: Bottom-Left (Extreme Wood Relief Overdrive)
    stage_test_boiler(
        sim,
        &BoilerStagingConfig {
            x0: 14,
            x1: 114,
            roof_y: 170,
            bottom_y: 236,
            floor_heater_rows: 3,
            floor_heater_temp: 500.0,
            upper_heater_temp: 500.0,
            water_temp: 98.0,
            roof_relief: Some((60, 68)),
            side_seam: None,
            chimney_rails: true,
            exhaust_duct: false,
        },
        stone,
    );

    // Panel D: Bottom-Right (Stone Sealed Extreme -> Delayed Pressure Breach)
    stage_test_boiler(
        sim,
        &BoilerStagingConfig {
            x0: 142,
            x1: 242,
            roof_y: 170,
            bottom_y: 236,
            floor_heater_rows: 3,
            floor_heater_temp: 500.0,
            upper_heater_temp: 500.0,
            water_temp: 98.0,
            roof_relief: None,
            side_seam: Some((214, 222)),
            chimney_rails: false,
            exhaust_duct: true,
        },
        stone,
    );
}

#[test]
fn test_c_d_initial_thermal_matter_symmetry() {
    let sim = two_hundred_fifty_six();
    stage_test_2x2_world(&sim);

    let mats = sim
        .world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("material readback");
    let temps = sim
        .world
        .read_temperature_all(&sim.context.device, &sim.context.queue)
        .expect("temperature readback");
    let w = 256;

    // Verify that inside the chamber bounds (width 100, height 66),
    // Panel C (14..114, 170..236) and Panel D (142..242, 170..236)
    // have 100% identical material and initial temperature for every internal cell.
    for dy in 1..66 {
        for dx in 1..100 {
            let c_x = 14 + dx;
            let d_x = 142 + dx;
            let y = 170 + dy;

            let c_mat = mats[(y * w + c_x) as usize];
            let d_mat = mats[(y * w + d_x) as usize];
            assert_eq!(
                c_mat, d_mat,
                "Internal chamber material mismatch at relative ({dx}, {dy}): C({c_x},{y})={c_mat} vs D({d_x},{y})={d_mat}"
            );

            let c_temp = temps[(y * w + c_x) as usize];
            let d_temp = temps[(y * w + d_x) as usize];
            assert!(
                (c_temp - d_temp).abs() < 1e-4,
                "Internal chamber temperature mismatch at relative ({dx}, {dy}): C={c_temp:.2} vs D={d_temp:.2}"
            );
        }
    }
}

#[test]
#[ignore = "historical G5 Water-yield-2 multi-boiler receipt; source-bound and not claimed by D-024"]
fn two_by_two_multi_boiler_stress_lab_relative_ordering_contract() {
    let mut sim = two_hundred_fifty_six();
    stage_test_2x2_world(&sim);

    let mut first_relief_a: Option<u64> = None;
    let mut first_relief_c: Option<u64> = None;
    let mut first_breach_d: Option<u64> = None;
    let mut first_vent_d: Option<u64> = None;
    let mut breach_d_pressure: f32 = 0.0;
    let mut breach_d_cell: (u32, u32) = (0, 0);

    let w = 256;

    // TE-2 Air thermal transport can cool the first escaping parcel; retain
    // enough bounded time for a later production Steam parcel to reach the
    // exterior duct after the pressure-driven breach.
    for tick in 1..=600 {
        sim.tick().expect("multi boiler lab tick");

        let mats = sim
            .world
            .read_material_all(&sim.context.device, &sim.context.queue)
            .expect("mats readback");

        // Panel A: Wood plug at y=44, x=60..68 (9 cells)
        let mut a_wood = 0;
        for x in 60..=68 {
            if mats[(44 * w + x) as usize] == MATERIAL_WOOD {
                a_wood += 1;
            }
        }
        if first_relief_a.is_none() && a_wood < 9 {
            first_relief_a = Some(tick);
            println!("[TEST] Panel A first relief at tick {tick} (wood remaining: {a_wood}/9)");
        }

        // Panel C: Wood plug at y=170, x=60..68 (9 cells)
        let mut c_wood = 0;
        for x in 60..=68 {
            if mats[(170 * w + x) as usize] == MATERIAL_WOOD {
                c_wood += 1;
            }
        }
        if first_relief_c.is_none() && c_wood < 9 {
            first_relief_c = Some(tick);
            println!("[TEST] Panel C first relief at tick {tick} (wood remaining: {c_wood}/9)");
        }

        // Panel D: Weak seam at x=242, y=214..222 (9 cells)
        let mut d_wood = 0;
        for y in 214..=222 {
            if mats[(y * w + 242) as usize] == MATERIAL_WOOD {
                d_wood += 1;
            } else if first_breach_d.is_none() {
                // Record breach cell and local neighbor pressure
                breach_d_cell = (242, y as u32);
                let pressures = sim
                    .world
                    .read_pressure_all(&sim.context.device, &sim.context.queue)
                    .expect("pressures readback");
                breach_d_pressure = pressures[(y * w + 241) as usize];
            }
        }
        if first_breach_d.is_none() && d_wood < 9 {
            first_breach_d = Some(tick);
            println!(
                "[TEST] Panel D first breach at tick {tick} (wood remaining: {d_wood}/9, cell: {:?}, local p: {:.1})",
                breach_d_cell, breach_d_pressure
            );
        }

        // Panel D Exterior Duct Venting (x=243..=254, y=210..=226)
        let mut ext_vented_fluid = 0u32;
        for y in 210..=226 {
            for x in 243..=254 {
                if matches!(mats[(y * w + x) as usize], MATERIAL_STEAM | MATERIAL_WATER) {
                    ext_vented_fluid += 1;
                }
            }
        }
        if first_vent_d.is_none() && ext_vented_fluid > 0 {
            first_vent_d = Some(tick);
            println!("[TEST] Panel D first exterior vent at tick {tick} (exterior fluid: {ext_vented_fluid} cells)");
        }
    }

    // 1. Panel A (Top-Left Standard Relief): Wood relief plug must open
    assert!(first_relief_a.is_some(), "Panel A relief plug must open");

    // 2. Panel B (Top-Right Stone Control): Stone roof must remain unbroken
    let mats = sim
        .world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("final mats readback");
    for x in 143..=241 {
        assert_eq!(
            mats[(44 * w + x) as usize],
            MATERIAL_STONE,
            "Panel B roof must remain 100% stone"
        );
    }

    // 3. Panel C (Bottom-Left Extreme Relief): Wood relief plug must open
    assert!(
        first_relief_c.is_some(),
        "Panel C extreme relief plug must open"
    );

    // 4. Panel D (Bottom-Right Extreme Breach): Weak seam must breach
    assert!(
        first_breach_d.is_some(),
        "Panel D weak seam must breach under accumulated overpressure"
    );

    let t_a = first_relief_a.unwrap();
    let t_c = first_relief_c.unwrap();
    let t_d = first_breach_d.unwrap();

    println!("[TEST] Summary: t_A = {t_a}, t_C = {t_c}, t_D = {t_d}");

    // Contract 1: C (Extreme Overdrive Relief) opens earlier or equal to A (Standard Relief)
    assert!(
        t_c <= t_a,
        "Extreme relief (tick {t_c}) must be earlier or equal to standard relief (tick {t_a})"
    );

    // Contract 2: D (Delayed Pressure Breach) delay separation contract
    // This is a demo readability / experiment-separation contract, not a simulation physics constant.
    const MIN_MEANINGFUL_DELAY: u64 = 60;
    assert!(
        t_d >= t_c + MIN_MEANINGFUL_DELAY,
        "Panel D delayed breach (tick {t_d}) must occur after fast relief (tick {t_c}) by at least {MIN_MEANINGFUL_DELAY} ticks"
    );

    // Contract 3: D breach occurred due to local pressure exceeding Wood threshold (80.0)
    assert!(
        breach_d_pressure >= 80.0,
        "Panel D breach-time local neighbor pressure ({breach_d_pressure:.1}) must reach or exceed Wood rupture threshold (80.0)"
    );

    // Contract 4: Steam venting into exterior exhaust duct must occur upon or after breach
    assert!(
        first_vent_d.is_some() && first_vent_d.unwrap() >= t_d,
        "Panel D exterior venting (tick {:?}) must occur upon or after breach (tick {t_d})",
        first_vent_d
    );
}
