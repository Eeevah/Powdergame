//! TE-5R1 local Steam-load relaxing dynamic pressure reference.

use crate::material::{movement_class, MovementClass, MATERIAL_EMPTY, MATERIAL_STEAM};
use crate::phase::LATENT_VAPORIZATION;

pub const PRESSURE_REFERENCE: f32 = 0.0;
pub const PRESSURE_DIFFUSION_RATE: f32 = 0.20;
pub const PRESSURE_RELAXATION_RATE: f32 = 0.02;
pub const FULL_STEAM_PRESSURE: f32 = 100.0;
pub const PRESSURE_MAX: f32 = 1.0e6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PressureNeighbor {
    pub material: u32,
    pub pressure: f32,
}

pub fn is_dynamic_pressure_node(material: u32) -> bool {
    material == MATERIAL_EMPTY
        || matches!(
            movement_class(material),
            Some(MovementClass::Liquid | MovementClass::Gas)
        )
}

/// Historical G5 diagnostic category: foreground Liquid/Gas only. TE-5R1
/// production pressure uses [`is_dynamic_pressure_node`] instead.
pub fn is_pressure_medium(material: u32) -> bool {
    matches!(
        movement_class(material),
        Some(MovementClass::Liquid | MovementClass::Gas)
    )
}

pub fn sanitize_pressure(value: f32) -> f32 {
    if !value.is_finite() {
        PRESSURE_REFERENCE
    } else {
        value.clamp(PRESSURE_REFERENCE, PRESSURE_MAX)
    }
}

/// Returns `None` only for invalid Steam phase state. Every non-Steam
/// identity, including Water, has target zero.
pub fn steam_pressure_target(material: u32, phase_energy: f32) -> Option<f32> {
    if material != MATERIAL_STEAM {
        return Some(PRESSURE_REFERENCE);
    }
    if !phase_energy.is_finite() || !(0.0..=LATENT_VAPORIZATION).contains(&phase_energy) {
        return None;
    }
    Some(FULL_STEAM_PRESSURE * phase_energy / LATENT_VAPORIZATION)
}

/// Read-neighbours/write-self update. Missing and blocked neighbours are
/// no-flux faces. Invalid Steam phase state fails closed to target zero;
/// authoritative staging rejects it before production use.
pub fn pressure_step_with_phase(
    self_material: u32,
    self_phase_energy: f32,
    self_pressure: f32,
    neighbors: [Option<PressureNeighbor>; 4],
) -> f32 {
    if !is_dynamic_pressure_node(self_material) {
        return PRESSURE_REFERENCE;
    }
    let q = sanitize_pressure(self_pressure);
    let mut diffusion = 0.0f32;
    for neighbor in neighbors.into_iter().flatten() {
        if is_dynamic_pressure_node(neighbor.material) {
            diffusion += sanitize_pressure(neighbor.pressure) - q;
        }
    }
    let target = steam_pressure_target(self_material, self_phase_energy).unwrap_or(0.0);
    sanitize_pressure(
        q + PRESSURE_DIFFUSION_RATE * diffusion + PRESSURE_RELAXATION_RATE * (target - q),
    )
}

pub fn pressure_step(
    self_material: u32,
    self_pressure: f32,
    neighbors: [Option<PressureNeighbor>; 4],
) -> f32 {
    pressure_step_with_phase(self_material, 0.0, self_pressure, neighbors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{MATERIAL_STONE, MATERIAL_WATER};

    fn right(material: u32, pressure: f32) -> [Option<PressureNeighbor>; 4] {
        [
            None,
            None,
            Some(PressureNeighbor { material, pressure }),
            None,
        ]
    }

    #[test]
    fn empty_liquid_and_gas_are_nodes_but_stone_is_blocked() {
        assert!(is_dynamic_pressure_node(MATERIAL_EMPTY));
        assert!(is_dynamic_pressure_node(MATERIAL_WATER));
        assert!(is_dynamic_pressure_node(MATERIAL_STEAM));
        assert!(!is_dynamic_pressure_node(MATERIAL_STONE));
    }

    #[test]
    fn only_valid_steam_supplies_a_target() {
        assert_eq!(steam_pressure_target(MATERIAL_WATER, 480.0), Some(0.0));
        assert_eq!(steam_pressure_target(MATERIAL_STEAM, 0.0), Some(0.0));
        assert_eq!(steam_pressure_target(MATERIAL_STEAM, 240.0), Some(50.0));
        assert_eq!(steam_pressure_target(MATERIAL_STEAM, 480.0), Some(100.0));
        assert_eq!(steam_pressure_target(MATERIAL_STEAM, -1.0), None);
        assert_eq!(steam_pressure_target(MATERIAL_STEAM, 481.0), None);
        assert_eq!(steam_pressure_target(MATERIAL_STEAM, f32::NAN), None);
    }

    #[test]
    fn fresh_isolated_generic_impulse_relaxes_from_100_to_98() {
        assert_eq!(
            pressure_step_with_phase(MATERIAL_WATER, 480.0, 100.0, [None; 4]),
            98.0
        );
    }

    #[test]
    fn new_canonical_steam_rises_by_two() {
        assert_eq!(
            pressure_step_with_phase(MATERIAL_STEAM, 480.0, 0.0, [None; 4]),
            2.0
        );
    }

    #[test]
    fn empty_participates_and_static_face_is_no_flux() {
        let from_empty =
            pressure_step_with_phase(MATERIAL_WATER, 0.0, 12.0, right(MATERIAL_EMPTY, 100.0));
        let stone_face =
            pressure_step_with_phase(MATERIAL_WATER, 0.0, 12.0, right(MATERIAL_STONE, 100.0));
        assert!((from_empty - 29.36).abs() < 1.0e-5);
        assert!((stone_face - 11.76).abs() < 1.0e-5);
    }

    #[test]
    fn update_stays_finite_non_negative_and_bounded() {
        let zero = Some(PressureNeighbor {
            material: MATERIAL_EMPTY,
            pressure: 0.0,
        });
        let next = pressure_step_with_phase(MATERIAL_STEAM, 480.0, 1.0e6, [zero; 4]);
        assert!(next.is_finite());
        assert!((0.0..=PRESSURE_MAX).contains(&next));
        assert_eq!(sanitize_pressure(f32::NAN), 0.0);
        assert_eq!(sanitize_pressure(-4.0), 0.0);
    }
}
