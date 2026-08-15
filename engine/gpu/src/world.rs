//! Dense GPU world state.
//!
//! G0 baseline: eight dense logical buffers — for each of Current and Next:
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

use wgpu::util::DeviceExt;

use powdergame_core::{WorldConfig, WorldLayout, MATERIAL_EMPTY};

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
    /// Allocates the full dense Current/Next world on `device`.
    ///
    /// The world state is zero-initialized: every cell is `EMPTY`
    /// (`material_id == MATERIAL_EMPTY`), temperature/pressure/flags start at
    /// zero. This is baseline plumbing only, not EMPTY gameplay semantics.
    pub fn new(device: &wgpu::Device, config: WorldConfig) -> Result<Self, GpuError> {
        let layout = config
            .layout()
            .map_err(|e| GpuError::Other(format!("invalid world config: {e}")))?;

        let mk =
            |label: &str, bytes: u64| create_zeroed_buffer(device, label, bytes, world_usage());

        let material_current = mk("world/material/current", layout.material_bytes)?;
        let material_next = mk("world/material/next", layout.material_bytes)?;
        let temperature_current = mk("world/temperature/current", layout.temperature_bytes)?;
        let temperature_next = mk("world/temperature/next", layout.temperature_bytes)?;
        let pressure_current = mk("world/pressure/current", layout.pressure_bytes)?;
        let pressure_next = mk("world/pressure/next", layout.pressure_bytes)?;
        let flags_current = mk("world/flags/current", layout.flags_bytes)?;
        let flags_next = mk("world/flags/next", layout.flags_bytes)?;

        let allocation = AllocationReport::from_layout(config, &layout);

        Ok(Self {
            config,
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
}

/// `material_id` value for an empty cell, re-exported for shader-side
/// baseline plumbing. `EMPTY` is not Matter (ADR-0001).
pub const MATERIAL_EMPTY_ID: u32 = MATERIAL_EMPTY;
