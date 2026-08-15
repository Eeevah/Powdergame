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
pub mod thermal;
pub mod world_config;

pub use domain::{initial_material_ids, Domain};
pub use layout::{
    WorldLayout, FLAGS_ELEM_SIZE, MATERIAL_ELEM_SIZE, PRESSURE_ELEM_SIZE, TEMPERATURE_ELEM_SIZE,
};
pub use material::{
    density_rank, density_table, is_valid_cell_material_value, movement_class,
    movement_class_table, registry_contains, registry_lookup, MaterialDescriptor, MovementClass,
    DENSITY_RANK_OIL, DENSITY_RANK_SAND, DENSITY_RANK_SMOKE, DENSITY_RANK_STEAM,
    DENSITY_RANK_WATER, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_OIL, MATERIAL_REGISTRY,
    MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, THERMAL_C_GAS,
    THERMAL_C_LIQUID, THERMAL_C_SAND, THERMAL_C_STONE, THERMAL_K_OIL, THERMAL_K_STONE,
    THERMAL_K_WATER,
};
pub use movement::{
    density_displacement_allowed, prefer_left, propose_move, CellState, DensityDirection,
    MoveTarget,
};
pub use thermal::{
    conductivity_table, heat_capacity_table, sanitize_temperature, thermal_properties,
    thermal_step, ThermalNeighbor, ThermalProperties, TEMPERATURE_REFERENCE, THERMAL_DEADBAND,
    THERMAL_MAX_DELTA, THERMAL_MIN_CAPACITY, THERMAL_RATE,
};
pub use world_config::{ConfigError, WorldConfig};
