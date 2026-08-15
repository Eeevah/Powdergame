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
//! (`docs/development/PERFORMANCE.md` §17, MILESTONES G0).
//!
//! G1: the initial world has an outermost ring of `BOUNDARY_BLOCK` with an
//! `EMPTY` interior. The world stays finite and authoritative on the GPU;
//! CPU-side work here is initialization/staging and small validated edit
//! hooks only (no per-tick full-world CPU simulation).

use wgpu::util::DeviceExt;

use powdergame_core::{
    initial_material_ids, is_valid_cell_material_value, Domain, WorldConfig, WorldLayout,
    MATERIAL_ELEM_SIZE, MATERIAL_EMPTY,
};

use crate::context::GpuError;

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
    pub total_requested_world_bytes: u64,
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
            total_requested_world_bytes: layout.total_world_bytes,
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
            "total world-state bytes: {}",
            self.total_requested_world_bytes
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
        })
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

    /// Minimal world-edit hook: sets one cell's material value.
    ///
    /// Validation before any write:
    /// - `value` must be `EMPTY` or a registered Matter, otherwise
    ///   `InvalidMaterialValue` — unknown IDs never enter the world.
    /// - `(x, y)` must be inside the finite world, otherwise
    ///   `CoordinateOutOfBounds` (Void) — no invisible-wall clamping.
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
        Ok(())
    }
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
