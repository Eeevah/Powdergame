//! Powdergame GPU Production Simulation.
//!
//! Owns the production GPU world (dense Current/Next buffers) and a headless
//! simulation lifecycle that does not require a Window, Surface or Renderer.
//!
//! Architectural boundary (see `docs/architecture/ARCHITECTURE.md`):
//! - this crate MUST NOT depend on winit, Window, Surface or Input.
//! - production world state lives on the GPU; CPU only orchestrates.

pub mod context;
pub mod simulation;
pub mod world;

pub use context::{
    describe_adapter_info, verify_target_hardware, AdapterReport, GpuContext, GpuError,
};
pub use simulation::Simulation;
pub use world::GpuWorld;
