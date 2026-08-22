//! TE-5R1 opposing-face total-pressure structural rupture helpers.

use crate::material::{registry_lookup, MATERIAL_REGISTRY};
use crate::pressure::sanitize_pressure;

pub const WOOD_RUPTURE_THRESHOLD: f32 = 80.0;

pub fn rupture_threshold(material_id: u32) -> Option<f32> {
    registry_lookup(material_id).and_then(|m| m.rupture_threshold)
}

pub fn rupture_threshold_table() -> [f32; 16] {
    let mut table = [0.0f32; 16];
    for material in MATERIAL_REGISTRY {
        if let Some(value) = material.rupture_threshold {
            table[material.id as usize] = value.max(0.0);
        }
    }
    table
}

/// Neighbour order is left, right, up, down. Missing/blocked faces provide
/// zero total pressure.
pub fn pressure_differential(neighbor_total_pressure: [Option<f32>; 4]) -> f32 {
    let sample = |value: Option<f32>| sanitize_pressure(value.unwrap_or(0.0));
    let left = sample(neighbor_total_pressure[0]);
    let right = sample(neighbor_total_pressure[1]);
    let up = sample(neighbor_total_pressure[2]);
    let down = sample(neighbor_total_pressure[3]);
    (left - right).abs().max((up - down).abs())
}

pub fn should_rupture(material_id: u32, neighbor_total_pressure: [Option<f32>; 4]) -> bool {
    let Some(limit) = rupture_threshold(material_id) else {
        return false;
    };
    limit.is_finite() && limit > 0.0 && pressure_differential(neighbor_total_pressure) >= limit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{MATERIAL_BOUNDARY_BLOCK, MATERIAL_STONE, MATERIAL_WOOD};

    #[test]
    fn uniform_pressure_does_not_rupture() {
        assert!(!should_rupture(
            MATERIAL_WOOD,
            [Some(100.0), Some(100.0), Some(20.0), Some(20.0)]
        ));
    }

    #[test]
    fn one_sided_threshold_differential_ruptures() {
        assert!(should_rupture(
            MATERIAL_WOOD,
            [Some(WOOD_RUPTURE_THRESHOLD), Some(0.0), None, None]
        ));
    }

    #[test]
    fn unbreakable_materials_ignore_extreme_differential() {
        for material in [MATERIAL_STONE, MATERIAL_BOUNDARY_BLOCK] {
            assert!(!should_rupture(
                material,
                [Some(1.0e6), Some(0.0), None, None]
            ));
        }
    }

    #[test]
    fn gpu_table_matches_material_descriptor() {
        let table = rupture_threshold_table();
        assert_eq!(table[MATERIAL_WOOD as usize], WOOD_RUPTURE_THRESHOLD);
        assert_eq!(table[MATERIAL_STONE as usize], 0.0);
    }
}
