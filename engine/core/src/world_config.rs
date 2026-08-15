//! Central world configuration.
//!
//! The reference world is `2048 x 2048` with an initial `64 x 64` chunk, but
//! world size must be configurable and never hardcoded into subsystems
//! (`docs/architecture/ARCHITECTURE.md` §4, `docs/specs/SIMULATION_SPEC.md` §2).

use crate::layout::WorldLayout;

/// Configuration for a finite dense world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldConfig {
    /// World width in cells.
    pub width: u32,
    /// World height in cells.
    pub height: u32,
    /// Initial chunk size in cells per side (benchmark baseline, not an invariant).
    pub chunk_size: u32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            width: 2048,
            height: 2048,
            chunk_size: 64,
        }
    }
}

impl WorldConfig {
    /// Returns the reference 2048 x 2048 world with a 64 x 64 chunk.
    pub fn reference() -> Self {
        Self::default()
    }

    /// Validates and returns a new configuration.
    pub fn new(width: u32, height: u32, chunk_size: u32) -> Result<Self, ConfigError> {
        let config = Self {
            width,
            height,
            chunk_size,
        };
        config.validate()?;
        Ok(config)
    }

    /// Validates this configuration.
    ///
    /// Checks positive dimensions/chunk, overflow-safe cell count and
    /// overflow-safe buffer byte sizes for every dense world buffer.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.width == 0 {
            return Err(ConfigError::ZeroWidth);
        }
        if self.height == 0 {
            return Err(ConfigError::ZeroHeight);
        }
        if self.chunk_size == 0 {
            return Err(ConfigError::ZeroChunkSize);
        }
        // Overflow-safe cell count.
        let _ = self.cell_count()?;
        // Overflow-safe buffer byte sizes for the dense SoA layout.
        let _ = self.layout()?;
        Ok(())
    }

    /// Total number of cells, computed with checked arithmetic.
    pub fn cell_count(&self) -> Result<u64, ConfigError> {
        let width = u64::from(self.width);
        let height = u64::from(self.height);
        width
            .checked_mul(height)
            .ok_or(ConfigError::CellCountOverflow)
    }

    /// Derived overflow-checked buffer layout for this world.
    pub fn layout(&self) -> Result<WorldLayout, ConfigError> {
        WorldLayout::new(self.cell_count()?).map_err(ConfigError::from)
    }
}

/// Configuration/layout validation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    /// `width` must be greater than zero.
    ZeroWidth,
    /// `height` must be greater than zero.
    ZeroHeight,
    /// `chunk_size` must be greater than zero.
    ZeroChunkSize,
    /// Cell count (`width * height`) overflows `u64`.
    CellCountOverflow,
    /// A dense buffer byte size overflows `u64`.
    BufferSizeOverflow,
}

impl From<crate::layout::LayoutError> for ConfigError {
    fn from(error: crate::layout::LayoutError) -> Self {
        match error {
            crate::layout::LayoutError::BufferSizeOverflow => ConfigError::BufferSizeOverflow,
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ZeroWidth => write!(f, "world width must be > 0"),
            ConfigError::ZeroHeight => write!(f, "world height must be > 0"),
            ConfigError::ZeroChunkSize => write!(f, "chunk_size must be > 0"),
            ConfigError::CellCountOverflow => write!(f, "cell count overflows u64"),
            ConfigError::BufferSizeOverflow => write!(f, "world buffer byte size overflows u64"),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_reference_world() {
        let config = WorldConfig::default();
        assert_eq!(config.width, 2048);
        assert_eq!(config.height, 2048);
        assert_eq!(config.chunk_size, 64);
        assert_eq!(config.cell_count().unwrap(), 4_194_304);
    }

    #[test]
    fn reference_config_is_valid() {
        assert!(WorldConfig::reference().validate().is_ok());
    }

    #[test]
    fn invalid_zero_values_are_rejected() {
        assert_eq!(
            WorldConfig::new(0, 2048, 64).unwrap_err(),
            ConfigError::ZeroWidth
        );
        assert_eq!(
            WorldConfig::new(2048, 0, 64).unwrap_err(),
            ConfigError::ZeroHeight
        );
        assert_eq!(
            WorldConfig::new(2048, 2048, 0).unwrap_err(),
            ConfigError::ZeroChunkSize
        );
        assert_eq!(
            WorldConfig::new(0, 0, 0).unwrap_err(),
            ConfigError::ZeroWidth
        );
    }

    #[test]
    fn cell_count_matches_reference() {
        assert_eq!(WorldConfig::reference().cell_count().unwrap(), 4_194_304);
        assert_eq!(
            WorldConfig::new(64, 64, 64).unwrap().cell_count().unwrap(),
            4096
        );
    }

    #[test]
    fn cell_count_overflow_safe_for_extreme_dimensions() {
        // Constructed directly (bypassing new()) because validate() already
        // rejects this config: the buffer byte size overflows u64 even though
        // the cell count itself still fits.
        let config = WorldConfig {
            width: u32::MAX,
            height: u32::MAX,
            chunk_size: 64,
        };
        // Cell count (u32::MAX x u32::MAX) still fits in u64.
        assert_eq!(
            config.cell_count().unwrap(),
            (u64::from(u32::MAX)) * (u64::from(u32::MAX))
        );
        // The derived buffer byte size must overflow-check and fail.
        assert_eq!(
            config.validate().unwrap_err(),
            ConfigError::BufferSizeOverflow
        );
    }
}
