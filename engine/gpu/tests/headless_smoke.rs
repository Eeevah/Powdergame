//! G0 headless GPU smoke test.
//!
//! Runs on the actual machine (reference: Windows + RTX 5090 + DX12).
//! Intentionally NOT ignored: this is the primary G0 runtime evidence.
//! It exercises the full chain without any Window:
//!
//! ```text
//! GPU Instance → DX12 Adapter → Device/Queue → WorldConfig
//! → Simulation/GpuWorld → tick()
//! ```

use powdergame_core::WorldConfig;
use powdergame_gpu::{verify_target_hardware, Simulation};

#[test]
fn headless_simulation_lifecycle_without_window() {
    // Production GPU context: DX12 + high-performance adapter.
    let mut simulation = pollster::block_on(Simulation::new(WorldConfig::reference()))
        .expect("simulation init (DX12 context + dense world)");

    // --- Adapter evidence ---
    assert!(
        matches!(simulation.context.adapter_info.backend, wgpu::Backend::Dx12),
        "G0 requires the DX12 backend, got {:?}",
        simulation.context.adapter_info.backend
    );
    verify_target_hardware(&simulation.context.adapter_info)
        .expect("reference hardware must be NVIDIA RTX 5090");

    // --- World allocation evidence ---
    assert_eq!(simulation.world.layout.cell_count, 4_194_304);
    assert_eq!(simulation.world.layout.material_bytes, 16_777_216);
    assert_eq!(simulation.world.layout.temperature_bytes, 16_777_216);
    assert_eq!(simulation.world.layout.pressure_bytes, 16_777_216);
    assert_eq!(simulation.world.layout.flags_bytes, 16_777_216);
    assert_eq!(simulation.world.layout.total_world_bytes, 134_217_728);
    assert_eq!(
        simulation.world.allocation.total_requested_world_bytes,
        134_217_728
    );

    // --- Headless lifecycle: multiple ticks, no window ---
    for _ in 0..4 {
        simulation.tick().expect("tick must complete without error");
    }
    assert_eq!(simulation.tick_count, 4);

    // --- GPU actually executed (not merely queued) ---
    let marker = simulation.read_marker().expect("GPU readback must succeed");
    assert_eq!(
        marker, 1,
        "the G0 tick shader must have executed on the GPU and set the marker"
    );
}
