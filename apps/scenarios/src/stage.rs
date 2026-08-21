use powdergame_core::EmptyEnvironmentSeed;
use powdergame_gpu::Simulation;

use crate::{ScenarioError, ScenarioFixture, ScenarioId};

/// Stages one scenario into a pristine or already-reset simulation.
///
/// This helper writes the complete authoritative tick-0 image to both Current
/// and Next buffers and restores the authored edit-wake snapshot. It does not
/// reset the tick counter or other scratch buffers; repeated trials should use
/// [`reset_and_stage_scenario`] instead.
pub fn stage_scenario(simulation: &Simulation, scenario: ScenarioId) -> Result<(), ScenarioError> {
    let fixture = ScenarioFixture::build(scenario, simulation.world.config)?;
    upload_fixture(simulation, &fixture)?;
    flush_and_wait(simulation)
}

/// Resets all simulation state and stages a deterministic scenario tick-0
/// image. Sleep settings are preserved by `Simulation::reset`.
///
/// The empty submission and completion wait keep reset/staging transfers out
/// of subsequent benchmark timing windows.
pub fn reset_and_stage_scenario(
    simulation: &mut Simulation,
    scenario: ScenarioId,
) -> Result<(), ScenarioError> {
    // Build and validate first so a rejected scenario/config leaves the
    // caller's live simulation and tick counter untouched.
    let fixture = ScenarioFixture::build(scenario, simulation.world.config)?;
    simulation
        .reset()
        .map_err(|error| ScenarioError::Gpu(error.to_string()))?;
    upload_fixture(simulation, &fixture)?;
    flush_and_wait(simulation)
}

fn upload_fixture(simulation: &Simulation, fixture: &ScenarioFixture) -> Result<(), ScenarioError> {
    if simulation.world.config != fixture.config {
        return Err(ScenarioError::InvalidFixture(format!(
            "fixture config {:?} does not match simulation config {:?}",
            fixture.config, simulation.world.config
        )));
    }
    fixture.validate()?;

    let queue = &simulation.context.queue;
    write_u32_pair(
        queue,
        &simulation.world.material_current,
        &simulation.world.material_next,
        &fixture.materials,
    );
    write_f32_pair(
        queue,
        &simulation.world.temperature_current,
        &simulation.world.temperature_next,
        &fixture.temperatures,
    );
    write_f32_pair(
        queue,
        &simulation.world.pressure_current,
        &simulation.world.pressure_next,
        &fixture.pressures,
    );
    write_u32_pair(
        queue,
        &simulation.world.flags_current,
        &simulation.world.flags_next,
        &fixture.flags,
    );
    simulation
        .world
        .stage_phase_energy_for_materials(queue, &fixture.materials)
        .map_err(|error| ScenarioError::Gpu(error.to_string()))?;
    simulation
        .world
        .stage_environment_for_materials(
            queue,
            &fixture.materials,
            EmptyEnvironmentSeed::StandardAtmosphere,
        )
        .map_err(|error| ScenarioError::Gpu(error.to_string()))?;

    let edit_wake_bytes = u32_bytes(&fixture.chunk_edit_wake);
    queue.write_buffer(&simulation.world.chunk_edit_wake, 0, &edit_wake_bytes);
    Ok(())
}

fn write_u32_pair(
    queue: &wgpu::Queue,
    current: &wgpu::Buffer,
    next: &wgpu::Buffer,
    values: &[u32],
) {
    let bytes = u32_bytes(values);
    queue.write_buffer(current, 0, &bytes);
    queue.write_buffer(next, 0, &bytes);
}

fn write_f32_pair(
    queue: &wgpu::Queue,
    current: &wgpu::Buffer,
    next: &wgpu::Buffer,
    values: &[f32],
) {
    let bytes = f32_bytes(values);
    queue.write_buffer(current, 0, &bytes);
    queue.write_buffer(next, 0, &bytes);
}

fn u32_bytes(values: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(values));
    for value in values {
        bytes.extend_from_slice(&value.to_ne_bytes());
    }
    bytes
}

fn f32_bytes(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(values));
    for value in values {
        bytes.extend_from_slice(&value.to_ne_bytes());
    }
    bytes
}

fn flush_and_wait(simulation: &Simulation) -> Result<(), ScenarioError> {
    simulation.context.queue.submit([]);
    simulation
        .context
        .device
        .poll(wgpu::PollType::Wait)
        .map(|_| ())
        .map_err(|error| ScenarioError::Gpu(format!("GPU wait failed: {error}")))
}
