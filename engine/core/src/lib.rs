//! Powdergame Simulation Core.
//!
//! GPU/Window-independent contracts and calculations.
//!
//! Architectural boundary (see `docs/architecture/ARCHITECTURE.md`):
//! - `powdergame-core` MUST NOT depend on winit, wgpu, or any windowing.
//! - `powdergame-gpu` MUST NOT depend on Window/Renderer/Input.
//! - only `apps/windows` may combine core + gpu with the platform layer.

pub mod layout;
pub mod world_config;

pub use layout::{
    WorldLayout, FLAGS_ELEM_SIZE, MATERIAL_ELEM_SIZE, PRESSURE_ELEM_SIZE, TEMPERATURE_ELEM_SIZE,
};
pub use world_config::{ConfigError, WorldConfig};

/// `material_id` value for an empty cell.
///
/// `EMPTY` is not a Matter and has no material properties
/// (see ADR-0001 / SIMULATION_SPEC §3.4). Its dense-array slot is `0`.
pub const MATERIAL_EMPTY: u32 = 0;
