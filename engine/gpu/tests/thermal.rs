//! G4-A — Thermal baseline: GPU semantic/invariant tests.
//!
//! Runs on the actual machine (Windows + RTX 5090 + DX12). Temperature is a
//! per-cell f32 field that belongs to the occupying Matter: movement commit
//! transports it on the same ownership edge. 4-neighbor conduction is
//! write-self only and runs after ownership is settled. EMPTY is not a
//! hidden thermal medium. Phase / combustion are out of scope.
//!
//! `0.0` is the simulation reference temperature (relative hot/cold scalar).
//! Exact global energy conservation is not required.

use powdergame_core::{
    WorldConfig, MATERIAL_EMPTY, MATERIAL_SAND, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER,
    TEMPERATURE_REFERENCE,
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

fn set_mat(sim: &Simulation, x: i64, y: i64, id: u32) {
    sim.world
        .write_material(&sim.context.queue, x, y, id)
        .expect("validated material edit");
}

fn set_t(sim: &Simulation, x: i64, y: i64, t: f32) {
    sim.world
        .write_temperature(&sim.context.queue, x, y, t)
        .expect("validated temperature edit");
}

fn temp(sim: &Simulation, x: i64, y: i64) -> f32 {
    sim.world
        .read_temperature_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("temperature readback")
}

fn all_temps(sim: &Simulation) -> Vec<f32> {
    sim.world
        .read_temperature_all(&sim.context.device, &sim.context.queue)
        .expect("full temperature readback")
}

fn assert_finite_world(sim: &Simulation) {
    for (i, t) in all_temps(sim).iter().enumerate() {
        assert!(t.is_finite(), "temperature[{i}] is not finite: {t}");
    }
}

#[test]
fn two_cell_hot_cold_propagation() {
    let mut sim = eight_by_eight();
    set_mat(&sim, 3, 3, MATERIAL_STONE);
    set_mat(&sim, 4, 3, MATERIAL_STONE);
    set_t(&sim, 3, 3, 10.0);
    set_t(&sim, 4, 3, 0.0);

    sim.tick().expect("tick");

    let hot = temp(&sim, 3, 3);
    let cold = temp(&sim, 4, 3);
    assert!(hot < 10.0, "hot stone must cool; got {hot}");
    assert!(cold > 0.0, "cold stone must heat; got {cold}");
    assert!(hot > cold, "ordering must remain hot > cold after one tick");
    assert_finite_world(&sim);
}

#[test]
fn four_neighbor_propagation() {
    let mut sim = eight_by_eight();
    set_mat(&sim, 3, 3, MATERIAL_STONE);
    set_t(&sim, 3, 3, 0.0);
    for (x, y) in [(3, 2), (3, 4), (2, 3), (4, 3)] {
        set_mat(&sim, x, y, MATERIAL_STONE);
        set_t(&sim, x, y, 20.0);
    }

    let mut one = eight_by_eight();
    set_mat(&one, 3, 3, MATERIAL_STONE);
    set_t(&one, 3, 3, 0.0);
    set_mat(&one, 4, 3, MATERIAL_STONE);
    set_t(&one, 4, 3, 20.0);

    sim.tick().expect("tick");
    one.tick().expect("tick");

    let four = temp(&sim, 3, 3);
    let single = temp(&one, 3, 3);
    assert!(
        four > 0.0,
        "center must heat from four neighbors; got {four}"
    );
    assert!(
        four > single,
        "four neighbors must transfer more than one ({four} vs {single})"
    );
}

#[test]
fn empty_gap_blocks_heat() {
    let mut sim = eight_by_eight();
    set_mat(&sim, 2, 3, MATERIAL_STONE);
    set_mat(&sim, 4, 3, MATERIAL_STONE);
    set_t(&sim, 2, 3, 20.0);
    set_t(&sim, 4, 3, 0.0);
    // (3,3) stays EMPTY — the gap must not conduct.

    for _ in 0..40 {
        sim.tick().expect("tick");
    }

    assert!(
        temp(&sim, 4, 3).abs() < 1.0e-5,
        "heat must not cross EMPTY; far stone is {}",
        temp(&sim, 4, 3)
    );
    assert_eq!(temp(&sim, 3, 3), TEMPERATURE_REFERENCE);
    assert!(temp(&sim, 2, 3) > 0.0, "hot stone keeps its own heat");
}

#[test]
fn stone_and_water_exchange_heat() {
    let mut sim = eight_by_eight();
    // Sit the pair on the boundary floor so Water cannot fall or diagonal-flow.
    set_mat(&sim, 3, 6, MATERIAL_STONE);
    set_mat(&sim, 4, 6, MATERIAL_WATER);
    set_mat(&sim, 5, 6, MATERIAL_STONE);
    set_t(&sim, 3, 6, 20.0);
    set_t(&sim, 4, 6, 0.0);

    sim.tick().expect("tick");

    assert!(
        temp(&sim, 4, 6) > 0.0,
        "water must heat from neighboring stone; got {}",
        temp(&sim, 4, 6)
    );
    assert!(
        temp(&sim, 3, 6) < 20.0,
        "stone must cool into water; got {}",
        temp(&sim, 3, 6)
    );
}

#[test]
fn thermal_crosses_chunk_boundary() {
    // 128×16: the 64-column chunk edge sits between x=63 and x=64.
    let mut sim = make_sim(WorldConfig::new(128, 16, 64).unwrap());
    set_mat(&sim, 63, 8, MATERIAL_STONE);
    set_mat(&sim, 64, 8, MATERIAL_STONE);
    set_t(&sim, 63, 8, 20.0);
    set_t(&sim, 64, 8, 0.0);

    sim.tick().expect("tick");

    let left = temp(&sim, 63, 8);
    let right = temp(&sim, 64, 8);
    assert!(left < 20.0, "hot side across the chunk edge must cool");
    assert!(right > 0.0, "cold side across the chunk edge must heat");
    assert!(left > right);
}

#[test]
fn repeated_ticks_stay_finite() {
    let mut sim = eight_by_eight();
    set_mat(&sim, 3, 3, MATERIAL_STONE);
    set_mat(&sim, 4, 3, MATERIAL_STONE);
    set_t(&sim, 3, 3, 1000.0);
    set_t(&sim, 4, 3, -50.0);

    for _ in 0..200 {
        sim.tick().expect("tick");
    }

    assert_finite_world(&sim);
    let a = temp(&sim, 3, 3);
    let b = temp(&sim, 4, 3);
    assert!(a.is_finite() && b.is_finite());
    assert!(
        (a - b).abs() < 5.0,
        "pair should be approaching; {a} vs {b}"
    );
}

#[test]
fn no_nan_or_infinity_in_world() {
    let mut sim = eight_by_eight();
    set_mat(&sim, 2, 2, MATERIAL_STONE);
    set_mat(&sim, 3, 2, MATERIAL_WATER);
    set_mat(&sim, 3, 3, MATERIAL_STONE);
    set_t(&sim, 2, 2, 40.0);
    set_t(&sim, 3, 2, -10.0);

    for _ in 0..20 {
        sim.tick().expect("tick");
    }

    for t in all_temps(&sim) {
        assert!(t.is_finite(), "found non-finite temperature {t}");
        assert!(!t.is_nan());
    }
}

#[test]
fn write_temperature_rejects_non_finite() {
    let sim = eight_by_eight();
    let err = sim
        .world
        .write_temperature(&sim.context.queue, 3, 3, f32::NAN)
        .expect_err("NaN must be rejected");
    assert!(format!("{err}").contains("invalid temperature"));
}

#[test]
fn empty_cell_temperature_stays_at_reference() {
    let mut sim = eight_by_eight();
    set_t(&sim, 3, 3, 7.0);
    sim.tick().expect("tick");
    assert_eq!(temp(&sim, 3, 3), TEMPERATURE_REFERENCE);
}

// ── Movement temperature transport ──────────────────────────────────────

#[test]
fn hot_matter_carries_temperature_when_moving() {
    // Isolated hot Sand above EMPTY: no neighboring Matter, so the same-tick
    // thermal pass cannot conduct. Heat must arrive at the destination.
    let mut sim = eight_by_eight();
    set_mat(&sim, 3, 3, MATERIAL_SAND);
    set_t(&sim, 3, 3, 100.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 4), MATERIAL_SAND, "sand fell one cell");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_EMPTY, "source vacated");
    let dest_t = temp(&sim, 3, 4);
    assert!(
        (dest_t - 100.0).abs() < 1.0e-3,
        "destination must carry the hot state; got {dest_t}"
    );
    assert_eq!(
        temp(&sim, 3, 3),
        TEMPERATURE_REFERENCE,
        "vacated cell must not keep ghost heat"
    );
}

#[test]
fn density_swap_carries_each_matter_temperature() {
    // Sand T=100 above Water T=0. After the swap, Sand's cell must still be
    // hotter than Water's cell. Same-tick conduction may mix a little, but
    // inverted thermal identity (heat left on the coordinate) is FAIL.
    let mut sim = eight_by_eight();
    set_mat(&sim, 3, 3, MATERIAL_SAND);
    set_mat(&sim, 3, 4, MATERIAL_WATER);
    set_t(&sim, 3, 3, 100.0);
    set_t(&sim, 3, 4, 0.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 4), MATERIAL_SAND, "sand sank");
    assert_eq!(cell(&sim, 3, 3), MATERIAL_WATER, "water displaced up");
    let sand_t = temp(&sim, 3, 4);
    let water_t = temp(&sim, 3, 3);
    assert!(
        sand_t > water_t,
        "Sand must remain hotter than Water after swap (sand {sand_t} vs water {water_t})"
    );
    assert!(
        sand_t > 40.0,
        "Sand should still be clearly hot; got {sand_t}"
    );
}

#[test]
fn void_exit_removes_temperature() {
    let mut sim = eight_by_eight();
    // Bottom-ring Sand: down is OOB → Void. Heat must leave with the Matter.
    set_mat(&sim, 4, 7, MATERIAL_SAND);
    set_t(&sim, 4, 7, 100.0);

    sim.tick().expect("tick");

    assert_eq!(
        cell(&sim, 4, 7),
        MATERIAL_EMPTY,
        "void exit vacated the cell"
    );
    assert_eq!(
        temp(&sim, 4, 7),
        TEMPERATURE_REFERENCE,
        "void exit must not leave heat in the world"
    );
    for t in all_temps(&sim) {
        assert!(t.is_finite());
    }
}

#[test]
fn blocked_or_losing_move_keeps_temperature() {
    // Fully blocked Sand: down and both diagonals are Stone. It must stay
    // put and keep a clearly hot temperature (neighbors may conduct a bit).
    let mut sim = eight_by_eight();
    set_mat(&sim, 3, 3, MATERIAL_SAND);
    set_mat(&sim, 3, 4, MATERIAL_STONE);
    set_mat(&sim, 2, 4, MATERIAL_STONE);
    set_mat(&sim, 4, 4, MATERIAL_STONE);
    set_t(&sim, 3, 3, 80.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 3), MATERIAL_SAND, "blocked sand stays");
    assert!(
        temp(&sim, 3, 3) > 40.0,
        "blocked sand must keep its heat; got {}",
        temp(&sim, 3, 3)
    );

    // Contention: two Steam cells propose the same EMPTY (3,1).
    // min-source wins — (2,2) has the lower index — and carries T=90.
    // The loser stays at (4,2) with its own cooler temperature.
    let mut sim = eight_by_eight();
    set_mat(&sim, 1, 1, MATERIAL_STONE);
    set_mat(&sim, 2, 1, MATERIAL_STONE);
    set_mat(&sim, 4, 1, MATERIAL_STONE);
    set_mat(&sim, 2, 2, MATERIAL_STEAM);
    set_mat(&sim, 4, 2, MATERIAL_STEAM);
    set_t(&sim, 2, 2, 90.0);
    set_t(&sim, 4, 2, 10.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 3, 1), MATERIAL_STEAM, "exactly one winner");
    assert_eq!(
        cell(&sim, 2, 2),
        MATERIAL_EMPTY,
        "min-source winner vacated"
    );
    assert_eq!(cell(&sim, 4, 2), MATERIAL_STEAM, "loser stays valid");
    assert_eq!(
        temp(&sim, 2, 2),
        TEMPERATURE_REFERENCE,
        "winner source must not keep ghost heat"
    );
    let dest_t = temp(&sim, 3, 1);
    let loser_t = temp(&sim, 4, 2);
    assert!(
        dest_t > loser_t,
        "winner must carry the hotter state (dest {dest_t} vs loser {loser_t})"
    );
    assert!(loser_t > 0.0, "loser must keep its own temperature");
}
