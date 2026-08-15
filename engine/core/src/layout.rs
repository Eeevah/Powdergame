//! Derived, overflow-checked buffer layout for the dense SoA world state.
//!
//! G0 baseline keeps four dense logical fields per world half (Current and
//! Next), each a plain `u32`/`f32` array — no packing, no f16, no bit tricks
//! (`docs/planning/MILESTONES.md` G0, `docs/development/PERFORMANCE.md` §17).
//!
//! Buffer byte sizes are always derived from here instead of being hardcoded
//! in subsystems.

/// Element byte size of the `material_id` field (`u32`).
pub const MATERIAL_ELEM_SIZE: u64 = 4;
/// Element byte size of the `temperature` field (`f32`).
pub const TEMPERATURE_ELEM_SIZE: u64 = 4;
/// Element byte size of the `pressure` field (`f32`).
pub const PRESSURE_ELEM_SIZE: u64 = 4;
/// Element byte size of the `flags` field (`u32`).
pub const FLAGS_ELEM_SIZE: u64 = 4;

/// Overflow-checked byte layout of one world state (one half: Current or Next).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldLayout {
    /// Total number of cells.
    pub cell_count: u64,
    /// Bytes of a `material_id` buffer for this world.
    pub material_bytes: u64,
    /// Bytes of a `temperature` buffer for this world.
    pub temperature_bytes: u64,
    /// Bytes of a `pressure` buffer for this world.
    pub pressure_bytes: u64,
    /// Bytes of a `flags` buffer for this world.
    pub flags_bytes: u64,
    /// Bytes of one world state half (Current or Next).
    pub half_bytes: u64,
    /// Total bytes of the full Current + Next world state.
    pub total_world_bytes: u64,
}

impl WorldLayout {
    /// Builds the layout with checked arithmetic for `cell_count` cells.
    pub fn new(cell_count: u64) -> Result<Self, LayoutError> {
        let material_bytes = cell_count
            .checked_mul(MATERIAL_ELEM_SIZE)
            .ok_or(LayoutError::BufferSizeOverflow)?;
        let temperature_bytes = cell_count
            .checked_mul(TEMPERATURE_ELEM_SIZE)
            .ok_or(LayoutError::BufferSizeOverflow)?;
        let pressure_bytes = cell_count
            .checked_mul(PRESSURE_ELEM_SIZE)
            .ok_or(LayoutError::BufferSizeOverflow)?;
        let flags_bytes = cell_count
            .checked_mul(FLAGS_ELEM_SIZE)
            .ok_or(LayoutError::BufferSizeOverflow)?;

        let half_bytes = material_bytes
            .checked_add(temperature_bytes)
            .and_then(|v| v.checked_add(pressure_bytes))
            .and_then(|v| v.checked_add(flags_bytes))
            .ok_or(LayoutError::BufferSizeOverflow)?;

        let total_world_bytes = half_bytes
            .checked_mul(2)
            .ok_or(LayoutError::BufferSizeOverflow)?;

        Ok(Self {
            cell_count,
            material_bytes,
            temperature_bytes,
            pressure_bytes,
            flags_bytes,
            half_bytes,
            total_world_bytes,
        })
    }
}

/// Buffer layout overflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutError {
    /// A dense buffer byte size overflows `u64`.
    BufferSizeOverflow,
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutError::BufferSizeOverflow => {
                write!(f, "world buffer byte size overflows u64")
            }
        }
    }
}

impl std::error::Error for LayoutError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_world_buffer_bytes() {
        let layout = WorldLayout::new(4_194_304).unwrap();
        assert_eq!(layout.material_bytes, 16_777_216);
        assert_eq!(layout.temperature_bytes, 16_777_216);
        assert_eq!(layout.pressure_bytes, 16_777_216);
        assert_eq!(layout.flags_bytes, 16_777_216);
        assert_eq!(layout.half_bytes, 67_108_864);
        assert_eq!(layout.total_world_bytes, 134_217_728);
    }

    #[test]
    fn buffer_byte_size_overflow_safe() {
        // 5e18 cells * 4 bytes already exceeds u64::MAX for one buffer.
        let layout = WorldLayout::new(5_000_000_000_000_000_000);
        assert!(layout.is_err());
        assert_eq!(layout.unwrap_err(), LayoutError::BufferSizeOverflow);
    }

    #[test]
    fn small_world_layout() {
        let layout = WorldLayout::new(4096).unwrap();
        assert_eq!(layout.material_bytes, 16_384);
        assert_eq!(layout.total_world_bytes, 131_072);
    }
}
