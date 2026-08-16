//! G5-C — pressure stress / structural rupture CPU reference helpers.
//!
//! Pressure remains a spatial field. Structural Matter does NOT store
//! pressure; instead it reads the pressure in its four orthogonal neighbors
//! and decides whether its own cell ruptures. The production path is WGSL;
//! these functions define the cheap Material-data contract for tests/tools.

use crate::material::{registry_lookup, MATERIAL_REGISTRY};
use crate::pressure::sanitize_pressure;

/// G5-C M0 weak-wall baseline. Relative gameplay pressure scalar, not SI.
/// One fully blocked Water→Steam expansion produces 100 pressure, so Wood
/// ruptures from that event while Stone/Boundary remain reference walls.
pub const WOOD_RUPTURE_THRESHOLD: f32 = 80.0;

/// Looks up a Material-owned rupture threshold. `None` means unbreakable in
/// the current M0 grammar (including Boundary Block and Stone).
pub fn rupture_threshold(material_id: u32) -> Option<f32> {
    registry_lookup(material_id).and_then(|m| m.rupture_threshold)
}

/// Compact GPU table. `0.0` means this Material does not rupture from
/// pressure in G5-C. This is Material data, never a per-cell strength field.
pub fn rupture_threshold_table() -> [f32; 16] {
    let mut table = [0.0f32; 16];
    for material in MATERIAL_REGISTRY {
        if let Some(value) = material.rupture_threshold {
            table[material.id as usize] = value.max(0.0);
        }
    }
    table
}

/// Pure Read-Neighbors → Write-Self rupture decision.
///
/// `neighbor_pressures` are only samples from pressure-medium neighbors;
/// callers pass `None` for EMPTY, Static/Powder or Void. Threshold equality
/// counts as rupture so the descriptor is the minimum pressure strength.
pub fn should_rupture(material_id: u32, neighbor_pressures: [Option<f32>; 4]) -> bool {
    let Some(limit) = rupture_threshold(material_id) else {
        return false;
    };
    if !limit.is_finite() || limit <= 0.0 {
        return false;
    }
    neighbor_pressures
        .into_iter()
        .flatten()
        .map(sanitize_pressure)
        .any(|pressure| pressure >= limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{
        MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_OIL, MATERIAL_SAND,
        MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD,
    };

    #[test]
    fn only_wood_is_weak_structure_in_m0_g5c() {
        assert_eq!(
            rupture_threshold(MATERIAL_WOOD),
            Some(WOOD_RUPTURE_THRESHOLD)
        );
        for id in [
            MATERIAL_EMPTY,
            MATERIAL_BOUNDARY_BLOCK,
            MATERIAL_STONE,
            MATERIAL_SAND,
            MATERIAL_WATER,
            MATERIAL_OIL,
            MATERIAL_STEAM,
            MATERIAL_SMOKE,
            MATERIAL_ICE,
        ] {
            assert_eq!(
                rupture_threshold(id),
                None,
                "material {id} should be unbreakable/non-structural in G5-C baseline"
            );
        }
    }

    #[test]
    fn sub_threshold_pressure_does_not_rupture_wood() {
        assert!(!should_rupture(
            MATERIAL_WOOD,
            [Some(WOOD_RUPTURE_THRESHOLD - 0.01), None, None, None]
        ));
    }

    #[test]
    fn threshold_pressure_ruptures_wood() {
        assert!(should_rupture(
            MATERIAL_WOOD,
            [None, Some(WOOD_RUPTURE_THRESHOLD), None, None]
        ));
    }

    #[test]
    fn unbreakable_material_ignores_extreme_pressure() {
        assert!(!should_rupture(
            MATERIAL_STONE,
            [Some(1.0e6), Some(1.0e6), Some(1.0e6), Some(1.0e6)]
        ));
        assert!(!should_rupture(
            MATERIAL_BOUNDARY_BLOCK,
            [Some(1.0e6), None, None, None]
        ));
    }

    #[test]
    fn gpu_table_matches_material_descriptor() {
        let table = rupture_threshold_table();
        assert_eq!(table[MATERIAL_WOOD as usize], WOOD_RUPTURE_THRESHOLD);
        assert_eq!(table[MATERIAL_STONE as usize], 0.0);
        assert_eq!(table[MATERIAL_BOUNDARY_BLOCK as usize], 0.0);
        assert_eq!(table[MATERIAL_WATER as usize], 0.0);
    }
}
