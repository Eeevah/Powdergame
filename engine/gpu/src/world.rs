//! Dense GPU world state.
//!
//! G1 baseline: eight dense logical buffers — for each of Current and Next:
//!
//! ```text
//! material_id[] : u32
//! temperature[] : f32
//! pressure[]    : f32
//! flags[]       : u32
//! ```
//!
//! No f16, packing, compaction, indirect dispatch or subtile masks
//! (`docs/development/PERFORMANCE.md` §17, MILESTONES G0). Density is a
//! Material table property (G3), NEVER a per-cell buffer — there is no
//! `density_current[]`/`density_next[]`.
//!
//! G1: the initial world has an outermost ring of `BOUNDARY_BLOCK` with an
//! `EMPTY` interior. The world stays finite and authoritative on the GPU;
//! CPU-side work here is initialization/staging and small validated edit
//! hooks only (no per-tick full-world CPU simulation).
//!
//! G2/G3: two auxiliary per-cell `u32` buffers support the movement
//! pipeline — `proposal` (each source's chosen destination) and `claim`
//! (each cell's single selected ownership edge, with reciprocal agreement
//! between both endpoints). They hold movement arbitration scratch state,
//! never Matter and never density state. G4-C reuses both buffers for the
//! smoke spawn proposal/claim after movement ownership has fully settled
//! (sequential passes, safe reuse).
//!
//! G4-C: `flags` holds Matter-owned combustion bits. Replacing a cell's
//! Material identity through the edit hook resets the cell's flags on both
//! Current and Next, so a stale `COMBUSTING` bit can never survive an
//! identity change.

use wgpu::util::DeviceExt;

use powdergame_core::{
    chunk_count, environment_image_from_materials, initial_material_ids,
    is_valid_cell_material_value, standard_air_state, vacuum_air_state, Domain,
    EmptyEnvironmentSeed, EnvironmentImage, WorldConfig, WorldLayout, FLAGS_ELEM_SIZE,
    MATERIAL_ELEM_SIZE, MATERIAL_EMPTY, PRESSURE_ELEM_SIZE, PRESSURE_REFERENCE,
    TEMPERATURE_ELEM_SIZE, TEMPERATURE_REFERENCE,
};

use crate::context::GpuError;

const MAX_ENVIRONMENT_TEST_READBACK_CELLS: usize = 64;

/// One bounded TE-1 test observation. Product diagnostics do not expose Air
/// until a later gate explicitly expands the Inspector contract.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvironmentCellSnapshot {
    pub x: i64,
    pub y: i64,
    pub current: powdergame_core::AirState,
    pub next: powdergame_core::AirState,
}

/// Storage usage shared by every world buffer.
fn world_usage() -> wgpu::BufferUsages {
    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST
}

/// Allocation evidence for the reference world (diagnostics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllocationReport {
    pub config: WorldConfig,
    pub cell_count: u64,
    pub material_current_bytes: u64,
    pub material_next_bytes: u64,
    pub temperature_current_bytes: u64,
    pub temperature_next_bytes: u64,
    pub pressure_current_bytes: u64,
    pub pressure_next_bytes: u64,
    pub flags_current_bytes: u64,
    pub flags_next_bytes: u64,
    pub air_mass_current_bytes: u64,
    pub air_mass_next_bytes: u64,
    pub air_energy_current_bytes: u64,
    pub air_energy_next_bytes: u64,
    pub environment_receiver_claim_bytes: u64,
    pub total_requested_world_bytes: u64,
    /// G7-A/B activity diagnostics and sleep state scratch (per-cell flags + 6 per-chunk u32 buffers).
    pub activity_scratch_bytes: u64,
}

impl AllocationReport {
    pub fn from_layout(config: WorldConfig, layout: &WorldLayout) -> Self {
        // Current and Next halves share the same per-field byte size.
        Self {
            config,
            cell_count: layout.cell_count,
            material_current_bytes: layout.material_bytes,
            material_next_bytes: layout.material_bytes,
            temperature_current_bytes: layout.temperature_bytes,
            temperature_next_bytes: layout.temperature_bytes,
            pressure_current_bytes: layout.pressure_bytes,
            pressure_next_bytes: layout.pressure_bytes,
            flags_current_bytes: layout.flags_bytes,
            flags_next_bytes: layout.flags_bytes,
            air_mass_current_bytes: layout.material_bytes,
            air_mass_next_bytes: layout.material_bytes,
            air_energy_current_bytes: layout.material_bytes,
            air_energy_next_bytes: layout.material_bytes,
            environment_receiver_claim_bytes: layout.material_bytes,
            total_requested_world_bytes: layout.total_world_bytes + 5 * layout.material_bytes,
            activity_scratch_bytes: layout.material_bytes
                + 6 * (chunk_count(config.width, config.height, config.chunk_size) as u64) * 4,
        }
    }
}

impl std::fmt::Display for AllocationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "WorldConfig:             {}x{} (chunk {})",
            self.config.width, self.config.height, self.config.chunk_size
        );
        let _ = writeln!(out, "Cell count:              {}", self.cell_count);
        let _ = writeln!(
            out,
            "material current bytes:  {}",
            self.material_current_bytes
        );
        let _ = writeln!(out, "material next bytes:     {}", self.material_next_bytes);
        let _ = writeln!(
            out,
            "temperature cur bytes:   {}",
            self.temperature_current_bytes
        );
        let _ = writeln!(
            out,
            "temperature next bytes:  {}",
            self.temperature_next_bytes
        );
        let _ = writeln!(
            out,
            "pressure current bytes:  {}",
            self.pressure_current_bytes
        );
        let _ = writeln!(out, "pressure next bytes:     {}", self.pressure_next_bytes);
        let _ = writeln!(out, "flags current bytes:     {}", self.flags_current_bytes);
        let _ = writeln!(out, "flags next bytes:        {}", self.flags_next_bytes);
        let _ = writeln!(
            out,
            "Air mass current bytes:  {}",
            self.air_mass_current_bytes
        );
        let _ = writeln!(out, "Air mass next bytes:     {}", self.air_mass_next_bytes);
        let _ = writeln!(
            out,
            "Air energy cur bytes:    {}",
            self.air_energy_current_bytes
        );
        let _ = writeln!(
            out,
            "Air energy next bytes:   {}",
            self.air_energy_next_bytes
        );
        let _ = writeln!(
            out,
            "Environment claim bytes: {}",
            self.environment_receiver_claim_bytes
        );
        let _ = writeln!(
            out,
            "total world-state bytes: {}",
            self.total_requested_world_bytes
        );
        let _ = writeln!(
            out,
            "G7 activity scratch bytes: {}",
            self.activity_scratch_bytes
        );
        f.write_str(out.trim_end())
    }
}

/// Dense Current/Next world buffers allocated on the GPU.
pub struct GpuWorld {
    pub config: WorldConfig,
    pub domain: Domain,
    pub layout: WorldLayout,
    pub allocation: AllocationReport,

    pub material_current: wgpu::Buffer,
    pub material_next: wgpu::Buffer,
    pub temperature_current: wgpu::Buffer,
    pub temperature_next: wgpu::Buffer,
    pub pressure_current: wgpu::Buffer,
    pub pressure_next: wgpu::Buffer,
    pub flags_current: wgpu::Buffer,
    pub flags_next: wgpu::Buffer,

    /// TE-1 Environment Air state. These buffers remain full-resolution and
    /// GPU-authoritative; Air is not a Matter ID.
    pub air_mass_current: wgpu::Buffer,
    pub air_mass_next: wgpu::Buffer,
    pub air_energy_current: wgpu::Buffer,
    pub air_energy_next: wgpu::Buffer,
    /// TE-1 receiver claim scratch. Encoding: 0 = none, target index + 1 = claim.
    pub environment_receiver_claim: wgpu::Buffer,

    /// Per-cell movement proposal (destination index, NO_MOVE or VOID_TARGET).
    /// G4-C reuses this buffer for smoke spawn proposals after movement
    /// ownership has fully settled.
    pub proposal: wgpu::Buffer,
    /// Per-cell ownership edge claim (reciprocal agreement for moves/swaps).
    /// G4-C reuses this buffer for smoke spawn claims (sequential passes).
    pub claim: wgpu::Buffer,

    /// G7-A per-cell activity flags (diagnostic measurement scratch;
    /// every cell is rewritten each tick by the activity propose pass).
    pub cell_activity: wgpu::Buffer,
    /// G7-A per-chunk activity mask (OR of the chunk's cell flags).
    pub chunk_activity: wgpu::Buffer,
    /// G7-A per-chunk "had any frontier this tick" diagnostic.
    pub chunk_changed_this_tick: wgpu::Buffer,
    /// G7-A per-chunk consecutive stable ticks (observation baseline).
    pub chunk_stable_ticks: wgpu::Buffer,
    /// G7-B per-chunk user/external edit wake trigger.
    pub chunk_edit_wake: wgpu::Buffer,
    /// G7-B per-chunk run/sleep state (0 = RUNNABLE, 1 = SLEEPING).
    pub chunk_state: wgpu::Buffer,
    /// G7-B per-chunk wake reason diagnostic bitmask.
    pub chunk_wake_reason: wgpu::Buffer,
}

/// Creates a zero-initialized buffer of `size` bytes.
fn create_zeroed_buffer(
    device: &wgpu::Device,
    label: &str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> Result<wgpu::Buffer, GpuError> {
    if size == 0 {
        return Err(GpuError::BufferCreateFailed(format!(
            "buffer {label} has zero size"
        )));
    }
    Ok(
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &vec![0u8; size as usize],
            usage,
        }),
    )
}

impl GpuWorld {
    /// Allocates the full dense Current/Next world on `device` and uploads
    /// the initial material state (outermost ring `BOUNDARY_BLOCK`, interior
    /// `EMPTY`).
    ///
    /// The initial material data is built on the CPU as initialization
    /// staging only; the authoritative world state lives on the GPU.
    pub fn new(device: &wgpu::Device, config: WorldConfig) -> Result<Self, GpuError> {
        let layout = config
            .layout()
            .map_err(|e| GpuError::Other(format!("invalid world config: {e}")))?;
        if layout.cell_count >= u32::MAX as u64 {
            return Err(GpuError::Other(format!(
                "world cell count {} cannot use TE-1 receiver claim target+1 encoding",
                layout.cell_count
            )));
        }
        let domain = Domain::from_config(&config);

        // Initial material state (staging): ring of BOUNDARY_BLOCK, EMPTY interior.
        let initial_ids = initial_material_ids(&config)
            .map_err(|e| GpuError::Other(format!("initial world build failed: {e}")))?;
        let mut material_bytes: Vec<u8> = Vec::with_capacity(initial_ids.len() * 4);
        for id in &initial_ids {
            material_bytes.extend_from_slice(&id.to_ne_bytes());
        }

        // Note: wgpu 26 `create_buffer_init` returns the buffer directly.
        let material_current = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("world/material/current"),
            contents: &material_bytes,
            usage: world_usage(),
        });
        let material_next = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("world/material/next"),
            contents: &material_bytes,
            usage: world_usage(),
        });

        let temperature_current = create_zeroed_buffer(
            device,
            "world/temperature/current",
            layout.temperature_bytes,
            world_usage(),
        )?;
        let temperature_next = create_zeroed_buffer(
            device,
            "world/temperature/next",
            layout.temperature_bytes,
            world_usage(),
        )?;
        let pressure_current = create_zeroed_buffer(
            device,
            "world/pressure/current",
            layout.pressure_bytes,
            world_usage(),
        )?;
        let pressure_next = create_zeroed_buffer(
            device,
            "world/pressure/next",
            layout.pressure_bytes,
            world_usage(),
        )?;
        let flags_current = create_zeroed_buffer(
            device,
            "world/flags/current",
            layout.flags_bytes,
            world_usage(),
        )?;
        let flags_next = create_zeroed_buffer(
            device,
            "world/flags/next",
            layout.flags_bytes,
            world_usage(),
        )?;

        let environment = environment_image_from_materials(
            &initial_ids,
            EmptyEnvironmentSeed::StandardAtmosphere,
        )
        .map_err(|error| GpuError::Other(format!("initial Environment build failed: {error}")))?;
        let air_mass_bytes = f32_bytes(&environment.air_mass);
        let air_energy_bytes = f32_bytes(&environment.air_energy);
        let air_mass_current = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("world/environment/air-mass/current"),
            contents: &air_mass_bytes,
            usage: world_usage(),
        });
        let air_mass_next = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("world/environment/air-mass/next"),
            contents: &air_mass_bytes,
            usage: world_usage(),
        });
        let air_energy_current = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("world/environment/air-energy/current"),
            contents: &air_energy_bytes,
            usage: world_usage(),
        });
        let air_energy_next = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("world/environment/air-energy/next"),
            contents: &air_energy_bytes,
            usage: world_usage(),
        });
        let environment_receiver_claim = create_zeroed_buffer(
            device,
            "world/environment/receiver-claim",
            layout.material_bytes,
            world_usage(),
        )?;

        // Movement arbitration scratch (never Matter, never density state).
        // Every entry is rewritten by the movement passes each tick.
        let proposal = create_zeroed_buffer(
            device,
            "world/proposal",
            layout.material_bytes,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        )?;
        let claim = create_zeroed_buffer(
            device,
            "world/claim",
            layout.material_bytes,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        )?;

        // G7-A activity diagnostics (measurement baseline; no dispatch is
        // skipped yet — G7-B decides that from these outputs).
        let activity_usage = wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST;
        let cell_activity = create_zeroed_buffer(
            device,
            "world/activity/cell-activity",
            layout.material_bytes,
            activity_usage,
        )?;
        let chunk_bytes = chunk_count(config.width, config.height, config.chunk_size) as u64 * 4;
        let chunk_activity = create_zeroed_buffer(
            device,
            "world/activity/chunk-activity",
            chunk_bytes,
            activity_usage,
        )?;
        let chunk_changed_this_tick = create_zeroed_buffer(
            device,
            "world/activity/chunk-changed",
            chunk_bytes,
            activity_usage,
        )?;
        let chunk_stable_ticks = create_zeroed_buffer(
            device,
            "world/activity/chunk-stable",
            chunk_bytes,
            activity_usage,
        )?;
        let chunk_edit_wake = create_zeroed_buffer(
            device,
            "world/activity/chunk-edit-wake",
            chunk_bytes,
            activity_usage,
        )?;
        let chunk_state = create_zeroed_buffer(
            device,
            "world/activity/chunk-state",
            chunk_bytes,
            activity_usage,
        )?;
        let chunk_wake_reason = create_zeroed_buffer(
            device,
            "world/activity/chunk-wake-reason",
            chunk_bytes,
            activity_usage,
        )?;

        let allocation = AllocationReport::from_layout(config, &layout);

        Ok(Self {
            config,
            domain,
            layout,
            allocation,
            material_current,
            material_next,
            temperature_current,
            temperature_next,
            pressure_current,
            pressure_next,
            flags_current,
            flags_next,
            air_mass_current,
            air_mass_next,
            air_energy_current,
            air_energy_next,
            environment_receiver_claim,
            proposal,
            claim,
            cell_activity,
            chunk_activity,
            chunk_changed_this_tick,
            chunk_stable_ticks,
            chunk_edit_wake,
            chunk_state,
            chunk_wake_reason,
        })
    }

    /// Resets all dense GPU world state and scratch buffers to the pristine
    /// initial state (outermost ring BOUNDARY_BLOCK, interior EMPTY, zero temperatures,
    /// zero pressures, zero flags, cleared proposal/claim scratch buffers, zeroed activity/diagnostics).
    ///
    /// Uses bulk uploads via `queue.write_buffer` instead of per-cell edits, eliminating
    /// multi-second pipeline stalls during demo reset.
    pub fn reset(&self, queue: &wgpu::Queue) -> Result<(), GpuError> {
        let initial_ids = initial_material_ids(&self.config)
            .map_err(|e| GpuError::Other(format!("initial world build failed: {e}")))?;
        let mut material_bytes: Vec<u8> = Vec::with_capacity(initial_ids.len() * 4);
        for id in &initial_ids {
            material_bytes.extend_from_slice(&id.to_ne_bytes());
        }

        queue.write_buffer(&self.material_current, 0, &material_bytes);
        queue.write_buffer(&self.material_next, 0, &material_bytes);

        let zero_cells = vec![0u8; self.layout.material_bytes as usize];
        queue.write_buffer(&self.temperature_current, 0, &zero_cells);
        queue.write_buffer(&self.temperature_next, 0, &zero_cells);
        queue.write_buffer(&self.pressure_current, 0, &zero_cells);
        queue.write_buffer(&self.pressure_next, 0, &zero_cells);
        queue.write_buffer(&self.flags_current, 0, &zero_cells);
        queue.write_buffer(&self.flags_next, 0, &zero_cells);
        self.stage_environment_for_materials(
            queue,
            &initial_ids,
            EmptyEnvironmentSeed::StandardAtmosphere,
        )?;
        queue.write_buffer(&self.environment_receiver_claim, 0, &zero_cells);
        queue.write_buffer(&self.proposal, 0, &zero_cells);
        queue.write_buffer(&self.claim, 0, &zero_cells);
        queue.write_buffer(&self.cell_activity, 0, &zero_cells);

        let chunk_bytes = (chunk_count(
            self.config.width,
            self.config.height,
            self.config.chunk_size,
        ) * 4) as usize;
        let zero_chunks = vec![0u8; chunk_bytes];
        queue.write_buffer(&self.chunk_activity, 0, &zero_chunks);
        queue.write_buffer(&self.chunk_changed_this_tick, 0, &zero_chunks);
        queue.write_buffer(&self.chunk_stable_ticks, 0, &zero_chunks);
        queue.write_buffer(&self.chunk_edit_wake, 0, &zero_chunks);
        queue.write_buffer(&self.chunk_state, 0, &zero_chunks);
        queue.write_buffer(&self.chunk_wake_reason, 0, &zero_chunks);

        Ok(())
    }

    /// Reads a whole u32 buffer (test/diagnostic helper).
    fn read_u32_buffer(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer: &wgpu::Buffer,
        count: u64,
    ) -> Result<Vec<u32>, GpuError> {
        let bytes = read_back_bytes(device, queue, buffer, 0, count * 4)?;
        let mut values = Vec::with_capacity(bytes.len() / 4);
        for chunk in bytes.chunks_exact(4) {
            values.push(u32::from_ne_bytes(chunk.try_into().unwrap()));
        }
        Ok(values)
    }

    /// Canonically stages both Environment halves from a Material image.
    pub fn stage_environment_for_materials(
        &self,
        queue: &wgpu::Queue,
        materials: &[u32],
        empty_seed: EmptyEnvironmentSeed,
    ) -> Result<(), GpuError> {
        if materials.len() as u64 != self.layout.cell_count {
            return Err(GpuError::Other(format!(
                "Environment staging Material length {} does not match cell count {}",
                materials.len(),
                self.layout.cell_count
            )));
        }
        let image = environment_image_from_materials(materials, empty_seed)
            .map_err(|error| GpuError::Other(format!("Environment staging failed: {error}")))?;
        self.stage_environment_image(queue, &image)
    }

    fn stage_environment_image(
        &self,
        queue: &wgpu::Queue,
        image: &EnvironmentImage,
    ) -> Result<(), GpuError> {
        if image.air_mass.len() as u64 != self.layout.cell_count
            || image.air_energy.len() as u64 != self.layout.cell_count
        {
            return Err(GpuError::Other(
                "Environment image length does not match world".into(),
            ));
        }
        let mass = f32_bytes(&image.air_mass);
        let energy = f32_bytes(&image.air_energy);
        queue.write_buffer(&self.air_mass_current, 0, &mass);
        queue.write_buffer(&self.air_mass_next, 0, &mass);
        queue.write_buffer(&self.air_energy_current, 0, &energy);
        queue.write_buffer(&self.air_energy_next, 0, &energy);
        Ok(())
    }

    /// Bounded test-only Environment observation for selected Cells.
    ///
    /// This deliberately refuses full-world and pointer-driven use. TE-1 does
    /// not expand the product Inspector payload or sampling cadence.
    pub fn read_environment_cells(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cells: &[(i64, i64)],
    ) -> Result<Vec<EnvironmentCellSnapshot>, GpuError> {
        if cells.len() > MAX_ENVIRONMENT_TEST_READBACK_CELLS {
            return Err(GpuError::ReadbackFailed(format!(
                "Environment test readback requested {} Cells; maximum is {}",
                cells.len(),
                MAX_ENVIRONMENT_TEST_READBACK_CELLS
            )));
        }
        let mut observations = Vec::with_capacity(cells.len());
        for &(x, y) in cells {
            let index = self
                .domain
                .index(x, y)
                .ok_or(GpuError::CoordinateOutOfBounds { x, y })?;
            let offset = index * 4;
            let read = |buffer: &wgpu::Buffer| -> Result<f32, GpuError> {
                let bytes = read_back_bytes(device, queue, buffer, offset, 4)?;
                Ok(f32::from_ne_bytes(bytes[..4].try_into().unwrap()))
            };
            observations.push(EnvironmentCellSnapshot {
                x,
                y,
                current: powdergame_core::AirState {
                    mass: read(&self.air_mass_current)?,
                    energy: read(&self.air_energy_current)?,
                },
                next: powdergame_core::AirState {
                    mass: read(&self.air_mass_next)?,
                    energy: read(&self.air_energy_next)?,
                },
            });
        }
        Ok(observations)
    }

    /// Test-only bounded Cell Environment staging. Production authoring uses
    /// the occupancy-aware Material/Sandbox paths.
    pub fn write_environment_cell_for_test(
        &self,
        queue: &wgpu::Queue,
        x: i64,
        y: i64,
        state: powdergame_core::AirState,
    ) -> Result<(), GpuError> {
        powdergame_core::validate_air_state(state)
            .map_err(|error| GpuError::Other(format!("invalid Air state: {error}")))?;
        let index = self
            .domain
            .index(x, y)
            .ok_or(GpuError::CoordinateOutOfBounds { x, y })?;
        let offset = index * 4;
        queue.write_buffer(&self.air_mass_current, offset, &state.mass.to_ne_bytes());
        queue.write_buffer(&self.air_mass_next, offset, &state.mass.to_ne_bytes());
        queue.write_buffer(
            &self.air_energy_current,
            offset,
            &state.energy.to_ne_bytes(),
        );
        queue.write_buffer(&self.air_energy_next, offset, &state.energy.to_ne_bytes());
        Ok(())
    }

    /// Reads all per-cell activity flags (G7-A test helper).
    pub fn read_cell_activity_all(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u32>, GpuError> {
        self.read_u32_buffer(device, queue, &self.cell_activity, self.layout.cell_count)
    }

    /// Reads the per-chunk activity masks (G7-A test helper).
    pub fn read_chunk_activity_all(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u32>, GpuError> {
        let count = chunk_count(
            self.config.width,
            self.config.height,
            self.config.chunk_size,
        ) as u64;
        self.read_u32_buffer(device, queue, &self.chunk_activity, count)
    }

    /// Reads the per-chunk "changed this tick" diagnostics (G7-A test helper).
    pub fn read_chunk_changed_all(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u32>, GpuError> {
        let count = chunk_count(
            self.config.width,
            self.config.height,
            self.config.chunk_size,
        ) as u64;
        self.read_u32_buffer(device, queue, &self.chunk_changed_this_tick, count)
    }

    /// Reads the per-chunk stable-ticks counters (G7-A test helper).
    pub fn read_chunk_stable_all(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u32>, GpuError> {
        let count = chunk_count(
            self.config.width,
            self.config.height,
            self.config.chunk_size,
        ) as u64;
        self.read_u32_buffer(device, queue, &self.chunk_stable_ticks, count)
    }

    /// Reads the per-chunk run/sleep state (G7-B test helper, 0 = RUNNABLE, 1 = SLEEPING).
    pub fn read_chunk_state_all(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u32>, GpuError> {
        let count = chunk_count(
            self.config.width,
            self.config.height,
            self.config.chunk_size,
        ) as u64;
        self.read_u32_buffer(device, queue, &self.chunk_state, count)
    }

    /// Reads the per-chunk wake reason bitmasks (G7-B test helper).
    pub fn read_chunk_wake_reason_all(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u32>, GpuError> {
        let count = chunk_count(
            self.config.width,
            self.config.height,
            self.config.chunk_size,
        ) as u64;
        self.read_u32_buffer(device, queue, &self.chunk_wake_reason, count)
    }

    /// Reads the per-chunk edit-wake flags (G7-B test helper).
    pub fn read_chunk_edit_wake_all(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u32>, GpuError> {
        let count = chunk_count(
            self.config.width,
            self.config.height,
            self.config.chunk_size,
        ) as u64;
        self.read_u32_buffer(device, queue, &self.chunk_edit_wake, count)
    }

    /// Reads a single cell's material value (diagnostic/test helper).
    ///
    /// Out-of-bounds coordinates fail with `CoordinateOutOfBounds` (Void) —
    /// they are never clamped or turned into a buffer index.
    pub fn read_material_cell(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        x: i64,
        y: i64,
    ) -> Result<u32, GpuError> {
        let index = self
            .domain
            .index(x, y)
            .ok_or(GpuError::CoordinateOutOfBounds { x, y })?;
        let offset = index * MATERIAL_ELEM_SIZE;
        let bytes = read_back_bytes(
            device,
            queue,
            &self.material_current,
            offset,
            MATERIAL_ELEM_SIZE,
        )?;
        Ok(u32::from_ne_bytes(bytes[..4].try_into().unwrap()))
    }

    /// Reads the entire material Current buffer (test helper for small worlds).
    pub fn read_material_all(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u32>, GpuError> {
        let bytes = read_back_bytes(
            device,
            queue,
            &self.material_current,
            0,
            self.layout.material_bytes,
        )?;
        let mut cells = Vec::with_capacity(bytes.len() / 4);
        for chunk in bytes.chunks_exact(4) {
            cells.push(u32::from_ne_bytes(chunk.try_into().unwrap()));
        }
        Ok(cells)
    }

    /// Marks the chunk containing `(x, y)` with an edit wake trigger, resetting
    /// its stable ticks. The GPU `activity_wake` pass automatically propagates the
    /// wake to all 8 neighbor chunks via its safety halo evaluation.
    pub fn mark_edit_wake_for_cell(&self, queue: &wgpu::Queue, x: i64, y: i64) {
        if x < 0 || y < 0 || x >= self.config.width as i64 || y >= self.config.height as i64 {
            return;
        }
        let cx = (x as u32) / self.config.chunk_size;
        let cy = (y as u32) / self.config.chunk_size;
        let c_x = powdergame_core::chunks_x(self.config.width, self.config.chunk_size);
        let n_idx = cy * c_x + cx;
        let off = (n_idx as u64) * 4;
        let one = 1u32.to_ne_bytes();
        let zero = 0u32.to_ne_bytes();
        queue.write_buffer(&self.chunk_edit_wake, off, &one);
        queue.write_buffer(&self.chunk_stable_ticks, off, &zero);
    }

    /// Minimal world-edit hook: sets one cell's material value.
    ///
    /// Validation before any write:
    /// - `value` must be `EMPTY` or a registered Matter, otherwise
    ///   `InvalidMaterialValue` — unknown IDs never enter the world.
    /// - `(x, y)` must be inside the finite world, otherwise
    ///   `CoordinateOutOfBounds` (Void) — no invisible-wall clamping.
    ///
    /// Replacing a cell's Material identity resets its Matter-owned flags
    /// on both Current and Next (a new identity never inherits a stale
    /// `COMBUSTING` state, `MATERIAL_SPEC` §4). Writing `EMPTY` also
    /// resets temperature to the reference.
    ///
    /// Writes both Current and Next so the two halves stay consistent at
    /// rest. This is an edit/command hook, not a per-tick CPU simulation.
    pub fn write_material(
        &self,
        queue: &wgpu::Queue,
        x: i64,
        y: i64,
        value: u32,
    ) -> Result<(), GpuError> {
        if !is_valid_cell_material_value(value) {
            return Err(GpuError::InvalidMaterialValue(value));
        }
        let index = self
            .domain
            .index(x, y)
            .ok_or(GpuError::CoordinateOutOfBounds { x, y })?;
        let offset = index * MATERIAL_ELEM_SIZE;
        let bytes = value.to_ne_bytes();
        queue.write_buffer(&self.material_current, offset, &bytes);
        queue.write_buffer(&self.material_next, offset, &bytes);
        // Matter-owned flags never survive an identity replacement.
        let zero_flags = 0u32.to_ne_bytes();
        let f_off = index * FLAGS_ELEM_SIZE;
        queue.write_buffer(&self.flags_current, f_off, &zero_flags);
        queue.write_buffer(&self.flags_next, f_off, &zero_flags);
        // Pressure is spatial, not Matter-owned. An explicit authoring
        // identity replacement must never inherit stale field state.
        let zero_pressure = PRESSURE_REFERENCE.to_ne_bytes();
        let p_off = index * PRESSURE_ELEM_SIZE;
        queue.write_buffer(&self.pressure_current, p_off, &zero_pressure);
        queue.write_buffer(&self.pressure_next, p_off, &zero_pressure);
        if value == MATERIAL_EMPTY {
            // EMPTY is not a thermal medium and must not keep leftover heat.
            let zero = TEMPERATURE_REFERENCE.to_ne_bytes();
            let t_off = index * TEMPERATURE_ELEM_SIZE;
            queue.write_buffer(&self.temperature_current, t_off, &zero);
            queue.write_buffer(&self.temperature_next, t_off, &zero);
        }
        let air = if value == MATERIAL_EMPTY {
            standard_air_state()
        } else {
            vacuum_air_state()
        };
        let air_offset = index * 4;
        for buffer in [&self.air_mass_current, &self.air_mass_next] {
            queue.write_buffer(buffer, air_offset, &air.mass.to_ne_bytes());
        }
        for buffer in [&self.air_energy_current, &self.air_energy_next] {
            queue.write_buffer(buffer, air_offset, &air.energy.to_ne_bytes());
        }
        self.mark_edit_wake_for_cell(queue, x, y);
        Ok(())
    }

    /// Diagnostic/test hook: sets one cell's flags on Current and Next.
    ///
    /// Used to stage Matter-owned state (e.g. `FLAG_COMBUSTING`) in
    /// fixtures. Coordinate-validated; no material coupling by design.
    pub fn write_flags(
        &self,
        queue: &wgpu::Queue,
        x: i64,
        y: i64,
        value: u32,
    ) -> Result<(), GpuError> {
        let index = self
            .domain
            .index(x, y)
            .ok_or(GpuError::CoordinateOutOfBounds { x, y })?;
        let offset = index * FLAGS_ELEM_SIZE;
        let bytes = value.to_ne_bytes();
        queue.write_buffer(&self.flags_current, offset, &bytes);
        queue.write_buffer(&self.flags_next, offset, &bytes);
        self.mark_edit_wake_for_cell(queue, x, y);
        Ok(())
    }

    /// Reads a single cell's flags (diagnostic/test helper).
    pub fn read_flags_cell(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        x: i64,
        y: i64,
    ) -> Result<u32, GpuError> {
        let index = self
            .domain
            .index(x, y)
            .ok_or(GpuError::CoordinateOutOfBounds { x, y })?;
        let offset = index * FLAGS_ELEM_SIZE;
        let bytes = read_back_bytes(device, queue, &self.flags_current, offset, FLAGS_ELEM_SIZE)?;
        Ok(u32::from_ne_bytes(bytes[..4].try_into().unwrap()))
    }

    /// Reads the entire flags Current buffer (test helper).
    pub fn read_flags_all(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u32>, GpuError> {
        let bytes = read_back_bytes(
            device,
            queue,
            &self.flags_current,
            0,
            self.layout.flags_bytes,
        )?;
        let mut cells = Vec::with_capacity(bytes.len() / 4);
        for chunk in bytes.chunks_exact(4) {
            cells.push(u32::from_ne_bytes(chunk.try_into().unwrap()));
        }
        Ok(cells)
    }

    /// Reads a single cell's temperature (diagnostic/test helper).
    pub fn read_temperature_cell(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        x: i64,
        y: i64,
    ) -> Result<f32, GpuError> {
        let index = self
            .domain
            .index(x, y)
            .ok_or(GpuError::CoordinateOutOfBounds { x, y })?;
        let offset = index * TEMPERATURE_ELEM_SIZE;
        let bytes = read_back_bytes(
            device,
            queue,
            &self.temperature_current,
            offset,
            TEMPERATURE_ELEM_SIZE,
        )?;
        Ok(f32::from_ne_bytes(bytes[..4].try_into().unwrap()))
    }

    /// Reads the entire temperature Current buffer (test helper).
    pub fn read_temperature_all(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<f32>, GpuError> {
        let bytes = read_back_bytes(
            device,
            queue,
            &self.temperature_current,
            0,
            self.layout.temperature_bytes,
        )?;
        let mut cells = Vec::with_capacity(bytes.len() / 4);
        for chunk in bytes.chunks_exact(4) {
            cells.push(f32::from_ne_bytes(chunk.try_into().unwrap()));
        }
        Ok(cells)
    }

    /// Edit hook: sets one cell's temperature on Current and Next.
    ///
    /// Non-finite values are rejected. This does not change material.
    pub fn write_temperature(
        &self,
        queue: &wgpu::Queue,
        x: i64,
        y: i64,
        value: f32,
    ) -> Result<(), GpuError> {
        if !value.is_finite() {
            return Err(GpuError::InvalidTemperature(value));
        }
        let index = self
            .domain
            .index(x, y)
            .ok_or(GpuError::CoordinateOutOfBounds { x, y })?;
        let offset = index * TEMPERATURE_ELEM_SIZE;
        let bytes = value.to_ne_bytes();
        queue.write_buffer(&self.temperature_current, offset, &bytes);
        queue.write_buffer(&self.temperature_next, offset, &bytes);
        self.mark_edit_wake_for_cell(queue, x, y);
        Ok(())
    }

    /// Reads one cell's scalar pressure (diagnostic/test helper).
    pub fn read_pressure_cell(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        x: i64,
        y: i64,
    ) -> Result<f32, GpuError> {
        let index = self
            .domain
            .index(x, y)
            .ok_or(GpuError::CoordinateOutOfBounds { x, y })?;
        let offset = index * PRESSURE_ELEM_SIZE;
        let bytes = read_back_bytes(
            device,
            queue,
            &self.pressure_current,
            offset,
            PRESSURE_ELEM_SIZE,
        )?;
        Ok(f32::from_ne_bytes(bytes[..4].try_into().unwrap()))
    }

    /// Reads the entire scalar pressure Current buffer (test helper).
    pub fn read_pressure_all(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<f32>, GpuError> {
        let bytes = read_back_bytes(
            device,
            queue,
            &self.pressure_current,
            0,
            self.layout.pressure_bytes,
        )?;
        let mut cells = Vec::with_capacity(bytes.len() / 4);
        for chunk in bytes.chunks_exact(4) {
            cells.push(f32::from_ne_bytes(chunk.try_into().unwrap()));
        }
        Ok(cells)
    }

    /// Edit/test hook: sets scalar pressure on Current and Next.
    /// Non-finite values are rejected; the simulation pass later clears
    /// pressure from cells that are not Liquid/Gas pressure media.
    pub fn write_pressure(
        &self,
        queue: &wgpu::Queue,
        x: i64,
        y: i64,
        value: f32,
    ) -> Result<(), GpuError> {
        if !value.is_finite() {
            return Err(GpuError::InvalidPressure(value));
        }
        let index = self
            .domain
            .index(x, y)
            .ok_or(GpuError::CoordinateOutOfBounds { x, y })?;
        let offset = index * PRESSURE_ELEM_SIZE;
        let bytes = value.to_ne_bytes();
        queue.write_buffer(&self.pressure_current, offset, &bytes);
        queue.write_buffer(&self.pressure_next, offset, &bytes);
        self.mark_edit_wake_for_cell(queue, x, y);
        Ok(())
    }
}

fn f32_bytes(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(values));
    for value in values {
        bytes.extend_from_slice(&value.to_ne_bytes());
    }
    bytes
}

/// Copies `size` bytes out of `source` at `offset` and maps them back to CPU.
fn read_back_bytes(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source: &wgpu::Buffer,
    offset: u64,
    size: u64,
) -> Result<Vec<u8>, GpuError> {
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("world/readback-staging"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("world-readback-encoder"),
    });
    encoder.copy_buffer_to_buffer(source, offset, &staging, 0, size);
    queue.submit([encoder.finish()]);

    let _ = device.poll(wgpu::PollType::Wait);

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    let _ = device.poll(wgpu::PollType::Wait);

    rx.recv()
        .map_err(|e| GpuError::ReadbackFailed(format!("map callback lost: {e}")))?
        .map_err(|e| GpuError::ReadbackFailed(e.to_string()))?;

    let mapped = slice.get_mapped_range();
    let data = mapped.to_vec();
    drop(mapped);
    staging.unmap();
    Ok(data)
}

/// `material_id` value for an empty cell, re-exported for shader-side
/// baseline plumbing. `EMPTY` is not Matter (ADR-0001).
pub const MATERIAL_EMPTY_ID: u32 = MATERIAL_EMPTY;
