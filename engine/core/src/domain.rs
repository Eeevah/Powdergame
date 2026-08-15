//! Finite world domain contract.
//!
//! Out-of-domain coordinates are `Void` — never clamped back into the world.
//! There is no hidden collision wall and no Void cell/array slot
//! (`SIMULATION_SPEC` §4, ADR-0001 "Finite editable boundary").
//!
//! Vocabulary contract:
//! - `EMPTY` = an in-domain cell with no Matter.
//! - `Void`  = outside the world; there is no cell at all.
//!
//! An out-of-domain coordinate must therefore never be reported as `EMPTY`.

use crate::material::{MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY};
use crate::world_config::{ConfigError, WorldConfig};

/// Finite world domain derived from a `WorldConfig`.
///
/// Coordinates are `i64` so negative/outside coordinates are representable
/// and testable without unsigned wraparound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Domain {
    pub width: u32,
    pub height: u32,
}

impl Domain {
    /// Builds the domain from a world configuration.
    pub fn from_config(config: &WorldConfig) -> Self {
        Self {
            width: config.width,
            height: config.height,
        }
    }

    /// Total cell count (overflow-safe).
    pub fn cell_count(&self) -> Result<u64, ConfigError> {
        u64::from(self.width)
            .checked_mul(u64::from(self.height))
            .ok_or(ConfigError::CellCountOverflow)
    }

    /// True if `(x, y)` lies inside the finite world.
    pub fn contains(&self, x: i64, y: i64) -> bool {
        x >= 0
            && y >= 0
            && (x as u64) < u64::from(self.width)
            && (y as u64) < u64::from(self.height)
    }

    /// Cell index for an in-bounds coordinate.
    ///
    /// Out-of-bounds is `None` (Void). It is **never** clamped to an edge
    /// cell, so out-of-world coordinates cannot act as an invisible wall.
    pub fn index(&self, x: i64, y: i64) -> Option<u64> {
        if !self.contains(x, y) {
            return None;
        }
        Some((y as u64) * u64::from(self.width) + (x as u64))
    }

    /// Reconstructs `(x, y)` from a row-major cell index, if in bounds.
    pub fn coords(&self, index: u64) -> Option<(u64, u64)> {
        let count = self.cell_count().ok()?;
        if index >= count {
            return None;
        }
        Some((index % u64::from(self.width), index / u64::from(self.width)))
    }

    /// True if the cell lies on the outermost edge of the world.
    pub fn is_outer_edge(&self, x: i64, y: i64) -> bool {
        if !self.contains(x, y) {
            return false;
        }
        let (x, y) = (x as u64, y as u64);
        x == 0 || y == 0 || x == u64::from(self.width) - 1 || y == u64::from(self.height) - 1
    }
}

/// Initial material value for an **in-bounds** coordinate: outermost ring is
/// `BOUNDARY_BLOCK`, interior is `EMPTY`.
///
/// This is an in-bounds initialization helper only. The caller must pass a
/// coordinate inside `domain`; out-of-domain coordinates are `Void` (no cell
/// exists) and must never be interpreted as `EMPTY`. The only call site is
/// [`initial_material_ids`], which iterates strictly in-bounds coordinates.
fn initial_material_value(domain: &Domain, x: i64, y: i64) -> u32 {
    debug_assert!(
        domain.contains(x, y),
        "initial_material_value requires an in-bounds coordinate; ({x},{y}) is Void"
    );
    if domain.is_outer_edge(x, y) {
        MATERIAL_BOUNDARY_BLOCK
    } else {
        MATERIAL_EMPTY
    }
}

/// Builds the full initial material state for `config` as a dense row-major
/// `u32` array (outermost ring = `BOUNDARY_BLOCK`, interior = `EMPTY`).
///
/// Every produced value corresponds to an in-domain cell. CPU-side
/// construction here is initialization/staging only; the production world
/// stays authoritative on the GPU (`ARCHITECTURE.md` §8).
pub fn initial_material_ids(config: &WorldConfig) -> Result<Vec<u32>, ConfigError> {
    let domain = Domain::from_config(config);
    let count = domain.cell_count()?;
    let mut cells = Vec::with_capacity(count as usize);
    for y in 0..i64::from(config.height) {
        for x in 0..i64::from(config.width) {
            cells.push(initial_material_value(&domain, x, y));
        }
    }
    Ok(cells)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn domain_8() -> Domain {
        Domain {
            width: 8,
            height: 8,
        }
    }

    #[test]
    fn index_is_row_major() {
        let d = domain_8();
        assert_eq!(d.index(0, 0), Some(0));
        assert_eq!(d.index(7, 0), Some(7));
        assert_eq!(d.index(0, 1), Some(8));
        assert_eq!(d.index(4, 4), Some(36));
        assert_eq!(d.index(7, 7), Some(63));
    }

    #[test]
    fn outside_coordinates_are_void() {
        let d = domain_8();
        // left / top / right / bottom
        assert_eq!(d.index(-1, 0), None);
        assert_eq!(d.index(0, -1), None);
        assert_eq!(d.index(8, 0), None);
        assert_eq!(d.index(0, 8), None);
        // extreme coordinates: no overflow, no wraparound
        assert_eq!(d.index(i64::MIN, 0), None);
        assert_eq!(d.index(0, i64::MIN), None);
        assert_eq!(d.index(i64::MAX, i64::MAX), None);
        assert_eq!(d.index(-1, -1), None);
        assert!(!d.contains(-1, 0));
        assert!(!d.contains(8, 8));
    }

    #[test]
    fn outside_is_never_clamped_to_edge() {
        let d = domain_8();
        // Clamping would map -1 -> 0 and 8 -> 7; the contract forbids that.
        assert_ne!(d.index(-1, 0), d.index(0, 0));
        assert_ne!(d.index(8, 0), d.index(7, 0));
        assert_ne!(d.index(0, 8), d.index(0, 7));
    }

    #[test]
    fn oob_is_void_never_empty() {
        // Contract: EMPTY is an in-domain empty cell; Void is outside the
        // world. An out-of-domain coordinate must never be reported as EMPTY
        // by any initialization API.
        let d = domain_8();

        // 1. Out-of-domain coordinates are not cells at all (Void).
        for (x, y) in [(-1, 0), (0, -1), (8, 0), (0, 8), (8, 8), (-1, -1)] {
            assert_eq!(d.index(x, y), None, "({x},{y}) is Void, not a cell");
        }

        // 2. The initialization path only ever produces values for exactly
        //    the in-domain cells — never a value for an out-of-domain cell.
        let config = WorldConfig::new(8, 8, 8).unwrap();
        let cells = initial_material_ids(&config).unwrap();
        assert_eq!(cells.len(), 64, "exactly width*height in-domain cells");
        assert!(
            cells
                .iter()
                .all(|&v| v == MATERIAL_EMPTY || v == MATERIAL_BOUNDARY_BLOCK),
            "every value must be EMPTY or BOUNDARY_BLOCK for an in-domain cell"
        );
    }

    #[test]
    fn outer_edge_classification() {
        let d = domain_8();
        for (x, y) in [
            (0, 0),
            (7, 0),
            (0, 7),
            (7, 7),
            (3, 0),
            (3, 7),
            (0, 3),
            (7, 3),
        ] {
            assert!(d.is_outer_edge(x, y), "({x},{y}) should be outer edge");
        }
        for (x, y) in [(1, 1), (3, 3), (4, 5)] {
            assert!(!d.is_outer_edge(x, y), "({x},{y}) should be interior");
        }
        // Out-of-bounds is never an edge (and never clamps in).
        assert!(!d.is_outer_edge(-1, 0));
        assert!(!d.is_outer_edge(8, 0));
    }

    #[test]
    fn coords_round_trip() {
        let d = domain_8();
        for index in 0..64 {
            let (x, y) = d.coords(index).unwrap();
            assert_eq!(d.index(x as i64, y as i64), Some(index));
        }
        assert_eq!(d.coords(64), None);
    }

    #[test]
    fn one_cell_world_is_all_edge() {
        let d = Domain {
            width: 1,
            height: 1,
        };
        assert!(d.is_outer_edge(0, 0));
        assert_eq!(initial_material_value(&d, 0, 0), MATERIAL_BOUNDARY_BLOCK);
    }

    #[test]
    fn reference_initial_pattern() {
        let config = WorldConfig::reference();
        let d = Domain::from_config(&config);
        assert_eq!(d.cell_count().unwrap(), 4_194_304);
        assert_eq!(
            initial_material_value(&d, 0, 0),
            MATERIAL_BOUNDARY_BLOCK,
            "corner"
        );
        assert_eq!(
            initial_material_value(&d, 0, 1024),
            MATERIAL_BOUNDARY_BLOCK,
            "left edge"
        );
        assert_eq!(
            initial_material_value(&d, 2047, 1024),
            MATERIAL_BOUNDARY_BLOCK,
            "right edge"
        );
        assert_eq!(
            initial_material_value(&d, 1024, 0),
            MATERIAL_BOUNDARY_BLOCK,
            "top edge"
        );
        assert_eq!(
            initial_material_value(&d, 1024, 1024),
            MATERIAL_EMPTY,
            "interior"
        );
    }

    #[test]
    fn eight_by_eight_initial_pattern() {
        let config = WorldConfig::new(8, 8, 8).unwrap();
        let cells = initial_material_ids(&config).unwrap();
        assert_eq!(cells.len(), 64);
        // Row 0 and row 7: all boundary.
        for x in 0..8 {
            assert_eq!(cells[x], MATERIAL_BOUNDARY_BLOCK, "top row x={x}");
            assert_eq!(cells[56 + x], MATERIAL_BOUNDARY_BLOCK, "bottom row x={x}");
        }
        // Interior rows: boundary at x=0 and x=7, EMPTY inside.
        for y in 1..7 {
            assert_eq!(cells[y * 8], MATERIAL_BOUNDARY_BLOCK, "left edge y={y}");
            assert_eq!(
                cells[y * 8 + 7],
                MATERIAL_BOUNDARY_BLOCK,
                "right edge y={y}"
            );
            for x in 1..7 {
                assert_eq!(cells[y * 8 + x], MATERIAL_EMPTY, "interior ({x},{y})");
            }
        }
    }

    #[test]
    fn world_size_is_not_extended_by_boundary() {
        let config = WorldConfig::new(8, 8, 8).unwrap();
        let cells = initial_material_ids(&config).unwrap();
        // The boundary ring lives inside the same finite dimensions.
        assert_eq!(cells.len(), 8 * 8);
        let reference = WorldConfig::reference();
        assert_eq!(initial_material_ids(&reference).unwrap().len(), 4_194_304);
    }
}
