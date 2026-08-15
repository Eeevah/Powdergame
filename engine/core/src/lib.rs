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
pub mod world_config;

pub use domain::{initial_material_ids, Domain};
pub use layout::{
    WorldLayout, FLAGS_ELEM_SIZE, MATERIAL_ELEM_SIZE, PRESSURE_ELEM_SIZE, TEMPERATURE_ELEM_SIZE,
};
pub use material::{
    is_valid_cell_material_value, registry_contains, registry_lookup, MaterialDescriptor,
    MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_REGISTRY, MATERIAL_STONE,
};
pub use world_config::{ConfigError, WorldConfig};
