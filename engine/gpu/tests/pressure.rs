//! G5-A — scalar pressure field GPU semantic/invariant tests.
//!
//! These tests require the production Windows + RTX 5090 + DX12 path.
//! GitHub CI can compile them; the reference machine executes them for final
//! technical validation. G5-B expansion generation and G5-C rupture are out
//! of scope here.

use powdergame_core::{
    WorldConfig, MATERIAL_EMPTY, MATERIAL_STONE, MATERIAL_WATER, PRESSURE_REFERENCE,
};
use powdergame_gpu::Simulation;

fn make_sim(config: WorldConfig) -> Simulation {
    pollster::block_on(Simulation::new(config)).expect("DX12 + RTX 5090 simulation init")
}

fn eight_by_eight() -> Simulation {
    make_sim(WorldConfig::new(8, 8, 8).unwrap())
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

fn all_pressure(sim: &Simulation) -> Vec<f32> {
    sim.world
        .read_pressure_all(&sim.context.device, &sim.context.queue)
        .expect("pressure readback")
}

fn set_mat(sim: &Simulation, x: i64, y: i64, material: u32) {
    sim.world
        .write_material(&sim.context.queue, x, y, material)
        .expect("material edit");
}

fn set_pressure(sim: &Simulation, x: i64, y: i64, value: f32) {
    sim.world
        .write_pressure(&sim.context.queue, x, y, value)
        .expect("pressure edit");
}

fn box_water_pair(sim: &Simulation) {
    // Two Water cells at (3,3)/(4,3), all liquid movement exits blocked.
    for (x, y) in [(2, 3), (5, 3), (2, 4), (3, 4), (4, 4), (5, 4)] {
        set_mat(sim, x, y, MATERIAL_STONE);
    }
    set_mat(sim, 3, 3, MATERIAL_WATER);
    set_mat(sim, 4, 3, MATERIAL_WATER);
}

#[test]
fn pressure_propagates_between_adjacent_liquid_cells() {
    let mut sim = eight_by_eight();
    box_water_pair(&sim);
    set_pressure(&sim, 3, 3, 100.0);
    set_pressure(&sim, 4, 3, 0.0);

    sim.tick().expect("tick");

    let left = pressure(&sim, 3, 3);
    let right = pressure(&sim, 4, 3);
    assert!(left < 100.0 && left > 0.0, "left={left}");
    assert!(right > 0.0 && right < left, "right={right}, left={left}");
    assert!(((left + right) - 100.0).abs() < 1.0e-3, "sum={}", left + right);
}

#[test]
fn isolated_pressure_has_no_time_decay() {
    let mut sim = eight_by_eight();
    set_mat(&sim, 3, 3, MATERIAL_WATER);
    for (x, y) in [(2, 3), (4, 3), (2, 4), (3, 4), (4, 4)] {
        set_mat(&sim, x, y, MATERIAL_STONE);
    }
    set_pressure(&sim, 3, 3, 42.0);

    for _ in 0..120 {
        sim.tick().expect("tick");
    }

    let p = pressure(&sim, 3, 3);
    assert!((p - 42.0).abs() < 1.0e-4, "pressure decayed without a sink: {p}");
}

#[test]
fn non_medium_cells_clear_pressure() {
    let mut sim = eight_by_eight();
    set_pressure(&sim, 3, 3, 50.0); // EMPTY
    set_mat(&sim, 4, 3, MATERIAL_STONE);
    set_pressure(&sim, 4, 3, 50.0);

    sim.tick().expect("tick");

    assert_eq!(pressure(&sim, 3, 3), PRESSURE_REFERENCE);
    assert_eq!(pressure(&sim, 4, 3), PRESSURE_REFERENCE);
}

#[test]
fn material_edit_clears_stale_spatial_pressure() {
    let sim = eight_by_eight();
    set_mat(&sim, 3, 3, MATERIAL_WATER);
    set_pressure(&sim, 3, 3, 25.0);
    assert_eq!(pressure(&sim, 3, 3), 25.0);

    set_mat(&sim, 3, 3, MATERIAL_STONE);
    assert_eq!(pressure(&sim, 3, 3), PRESSURE_REFERENCE);
}

#[test]
fn pressure_crosses_chunk_boundary() {
    let mut sim = make_sim(WorldConfig::new(128, 16, 64).unwrap());
    for (x, y) in [(62, 8), (65, 8), (62, 9), (63, 9), (64, 9), (65, 9)] {
        set_mat(&sim, x, y, MATERIAL_STONE);
    }
    set_mat(&sim, 63, 8, MATERIAL_WATER);
    set_mat(&sim, 64, 8, MATERIAL_WATER);
    set_pressure(&sim, 63, 8, 40.0);

    sim.tick().expect("tick");

    assert!(pressure(&sim, 63, 8) < 40.0);
    assert!(pressure(&sim, 64, 8) > 0.0);
}

#[test]
fn void_exit_vents_pressure_with_departing_medium() {
    let mut sim = eight_by_eight();
    set_mat(&sim, 4, 7, MATERIAL_WATER);
    set_pressure(&sim, 4, 7, 80.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 4, 7), MATERIAL_EMPTY);
    assert_eq!(pressure(&sim, 4, 7), PRESSURE_REFERENCE);
}

#[test]
fn pressure_world_stays_finite_and_non_negative() {
    let mut sim = eight_by_eight();
    box_water_pair(&sim);
    set_pressure(&sim, 3, 3, 1.0e6);

    for _ in 0..200 {
        sim.tick().expect("tick");
    }

    for (i, p) in all_pressure(&sim).into_iter().enumerate() {
        assert!(p.is_finite(), "pressure[{i}] non-finite: {p}");
        assert!(p >= 0.0, "pressure[{i}] negative: {p}");
    }
}

#[test]
fn write_pressure_rejects_non_finite() {
    let sim = eight_by_eight();
    let err = sim
        .world
        .write_pressure(&sim.context.queue, 3, 3, f32::NAN)
        .expect_err("NaN must be rejected");
    assert!(format!("{err}").contains("invalid pressure"));
}
