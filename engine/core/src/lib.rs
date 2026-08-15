//! Powdergame Simulation Core.
//!
//! GPU/Window-independent contracts and calculations.
//!
//! Architectural boundary (see `docs/architecture/ARCHITECTURE.md`):
//! - `powdergame-core` MUST NOT depend on winit, wgpu, or any windowing.
//! - `powdergame-gpu` MUST NOT depend on Window/Renderer/Input.
//! - only `apps/windows` may combine core + gpu with the platform layer.

pub mod domain;
pub mod layout;
pub mod material;
pub mod movement;
pub mod world_config;

pub use domain::{initial_material_ids, Domain};
pub use layout::{
    WorldLayout, FLAGS_ELEM_SIZE, MATERIAL_ELEM_SIZE, PRESSURE_ELEM_SIZE, TEMPERATURE_ELEM_SIZE,
};
pub use material::{
    is_valid_cell_material_value, movement_class, movement_class_table, registry_contains,
    registry_lookup, MaterialDescriptor, MovementClass, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY,
    MATERIAL_OIL, MATERIAL_REGISTRY, MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE,
    MATERIAL_WATER,
};
pub use movement::{propose_move, CellState, MoveTarget};
pub use world_config::{ConfigError, WorldConfig};
