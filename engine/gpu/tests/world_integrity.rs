//! G1 — World Integrity: GPU-side evidence.
//!
//! Runs on the actual machine (reference: Windows + RTX 5090 + DX12).
//! Verifies boundary initialization, editable outer BLOCK, Void semantics,
//! invalid-ID rejection and the G0 tick regression — all on the GPU world.

use powdergame_core::{
    initial_material_ids, WorldConfig, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_STONE,
};
use powdergame_gpu::{GpuError, Simulation};

fn make_simulation(config: WorldConfig) -> Simulation {
    pollster::block_on(Simulation::new(config)).expect("simulation init (DX12)")
}

#[test]
fn reference_world_boundary_initialization() {
    let sim = make_simulation(WorldConfig::reference());

    // Corner / edge cells are BOUNDARY_BLOCK; interior is EMPTY.
    let cases = [
        ((0i64, 0i64), MATERIAL_BOUNDARY_BLOCK), // corner
        ((2047, 0), MATERIAL_BOUNDARY_BLOCK),    // top-right corner
        ((0, 2047), MATERIAL_BOUNDARY_BLOCK),    // bottom-left corner
        ((2047, 2047), MATERIAL_BOUNDARY_BLOCK), // bottom-right corner
        ((0, 1024), MATERIAL_BOUNDARY_BLOCK),    // left edge
        ((2047, 1024), MATERIAL_BOUNDARY_BLOCK), // right edge
        ((1024, 0), MATERIAL_BOUNDARY_BLOCK),    // top edge
        ((1024, 2047), MATERIAL_BOUNDARY_BLOCK), // bottom edge
        ((1024, 1024), MATERIAL_EMPTY),          // interior
        ((512, 512), MATERIAL_EMPTY),            // interior
    ];
    for ((x, y), expected) in cases {
        let value = sim
            .world
            .read_material_cell(&sim.context.device, &sim.context.queue, x, y)
            .unwrap_or_else(|e| panic!("read ({x},{y}) failed: {e}"));
        assert_eq!(value, expected, "cell ({x},{y})");
    }
}

#[test]
fn small_world_boundary_pattern_matches_expected() {
    let config = WorldConfig::new(8, 8, 8).unwrap();
    let sim = make_simulation(config);
    let expected = initial_material_ids(&config).unwrap();

    let actual = sim
        .world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("read all material");
    assert_eq!(actual.len(), 64);
    assert_eq!(actual, expected, "8x8 boundary ring must match");

    // Spot check the row-major layout explicitly.
    assert_eq!(actual[0], MATERIAL_BOUNDARY_BLOCK); // (0,0)
    assert_eq!(actual[7], MATERIAL_BOUNDARY_BLOCK); // (7,0)
    assert_eq!(actual[9], MATERIAL_EMPTY); // (1,1)
    assert_eq!(actual[56], MATERIAL_BOUNDARY_BLOCK); // (0,7)
    assert_eq!(actual[63], MATERIAL_BOUNDARY_BLOCK); // (7,7)
}

#[test]
fn boundary_block_is_editable_to_empty() {
    let sim = make_simulation(WorldConfig::new(8, 8, 8).unwrap());

    // Erase a boundary cell: BOUNDARY_BLOCK -> EMPTY.
    sim.world
        .write_material(&sim.context.queue, 0, 0, MATERIAL_EMPTY)
        .expect("erase boundary cell");
    let value = sim
        .world
        .read_material_cell(&sim.context.device, &sim.context.queue, 0, 0)
        .expect("read back erased cell");
    assert_eq!(value, MATERIAL_EMPTY, "boundary cell must become EMPTY");

    // The world is finite: dimensions did not change when the block was erased.
    assert_eq!(sim.world.config.width, 8);
    assert_eq!(sim.world.config.height, 8);
    assert_eq!(sim.world.domain.cell_count().unwrap(), 64);
    // Other boundary cells are untouched.
    let corner = sim
        .world
        .read_material_cell(&sim.context.device, &sim.context.queue, 7, 7)
        .expect("read other corner");
    assert_eq!(corner, MATERIAL_BOUNDARY_BLOCK);
}

#[test]
fn stone_is_a_registered_matter_distinct_from_empty() {
    let sim = make_simulation(WorldConfig::new(8, 8, 8).unwrap());

    // Place a registered Matter in the interior.
    sim.world
        .write_material(&sim.context.queue, 4, 4, MATERIAL_STONE)
        .expect("place stone");
    let value = sim
        .world
        .read_material_cell(&sim.context.device, &sim.context.queue, 4, 4)
        .expect("read back stone");
    assert_eq!(value, MATERIAL_STONE);
    assert_ne!(value, MATERIAL_EMPTY);
    assert!(
        powdergame_core::registry_contains(value),
        "stone must be a registered Matter"
    );
}

#[test]
fn invalid_material_edit_is_rejected() {
    let sim = make_simulation(WorldConfig::new(8, 8, 8).unwrap());

    // Unknown ID must never enter the world through the edit path.
    let err = sim
        .world
        .write_material(&sim.context.queue, 4, 4, 999)
        .expect_err("unknown material id must be rejected");
    assert!(
        matches!(err, GpuError::InvalidMaterialValue(999)),
        "got {err:?}"
    );

    // The cell was not modified.
    let value = sim
        .world
        .read_material_cell(&sim.context.device, &sim.context.queue, 4, 4)
        .expect("read cell");
    assert_eq!(value, MATERIAL_EMPTY);
}

#[test]
fn out_of_bounds_is_void_and_never_a_buffer_index() {
    let sim = make_simulation(WorldConfig::new(8, 8, 8).unwrap());

    // Writes outside the domain are rejected (Void), not clamped to an edge.
    for (x, y) in [(-1i64, 0i64), (8, 0), (0, 8), (0, -1)] {
        let err = sim
            .world
            .write_material(&sim.context.queue, x, y, MATERIAL_STONE)
            .expect_err("out-of-bounds write must be rejected");
        assert!(
            matches!(err, GpuError::CoordinateOutOfBounds { .. }),
            "write ({x},{y}) got {err:?}"
        );
    }

    // Reads outside the domain are rejected the same way.
    let err = sim
        .world
        .read_material_cell(&sim.context.device, &sim.context.queue, -1, 0)
        .expect_err("out-of-bounds read must be rejected");
    assert!(
        matches!(err, GpuError::CoordinateOutOfBounds { .. }),
        "read (-1,0) got {err:?}"
    );

    // No invisible wall: the edge cell next to the outside coordinate is
    // still a real boundary block, and nothing was clamped onto it.
    let edge = sim
        .world
        .read_material_cell(&sim.context.device, &sim.context.queue, 0, 0)
        .expect("read edge cell");
    assert_eq!(edge, MATERIAL_BOUNDARY_BLOCK);
}

#[test]
fn g0_tick_regression_on_boundary_world() {
    let mut sim = make_simulation(WorldConfig::reference());

    for _ in 0..3 {
        sim.tick().expect("tick on boundary world");
    }
    assert_eq!(sim.tick_count, 3);
    let marker = sim.read_marker().expect("read marker");
    assert_eq!(marker, 1, "G0 tick dispatch must still execute on the GPU");
}
