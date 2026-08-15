//! G5-A pressure field baseline — CPU reference rule.
//!
//! Pressure is a spatial per-cell `f32` field (`SIMULATION_SPEC` §15), not
//! Matter-owned state. The baseline is deliberately small:
//! - scalar pressure only (no pressure velocity vector),
//! - 4-neighbor local propagation,
//! - only LIQUID/GAS Matter acts as a pressure medium,
//! - EMPTY/Void and STATIC/POWDER do not secretly transmit pressure,
//! - no arbitrary time decay: an isolated pressured medium retains pressure,
//! - finite/non-negative sanitization prevents NaN/Infinity runaway.
//!
//! G5-B will generate pressure from blocked phase expansion. G5-C will use
//! pressure gradients to influence Matter and stress/rupture structures.

use crate::material::{movement_class, MovementClass};

/// Neutral pressure for cells that cannot host the field.
pub const PRESSURE_REFERENCE: f32 = 0.0;

/// Explicit 4-neighbor diffusion coefficient. Must stay <= 0.25 for the
/// symmetric four-neighbor explicit update to avoid overshoot.
pub const PRESSURE_DIFFUSION_RATE: f32 = 0.20;

/// Gameplay safety clamp, not a physical unit.
pub const PRESSURE_MAX: f32 = 1.0e6;

/// One orthogonal pressure sample. `None` represents Void/out-of-domain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PressureNeighbor {
    pub material: u32,
    pub pressure: f32,
}

/// Pressure propagates only through actual Liquid/Gas Matter in G5-A.
pub fn is_pressure_medium(material: u32) -> bool {
    matches!(
        movement_class(material),
        Some(MovementClass::Liquid | MovementClass::Gas)
    )
}

/// Collapses invalid pressure to the neutral value and bounds valid values.
pub fn sanitize_pressure(value: f32) -> f32 {
    if !value.is_finite() {
        PRESSURE_REFERENCE
    } else {
        value.clamp(PRESSURE_REFERENCE, PRESSURE_MAX)
    }
}

/// One Read-Neighbors / Write-Self pressure update.
///
/// Only pressure-media neighbors participate. There is no implicit loss term;
/// a sealed isolated Liquid/Gas cell therefore keeps its pressure exactly.
pub fn pressure_step(
    self_material: u32,
    self_pressure: f32,
    neighbors: [Option<PressureNeighbor>; 4],
) -> f32 {
    if !is_pressure_medium(self_material) {
        return PRESSURE_REFERENCE;
    }

    let self_p = sanitize_pressure(self_pressure);
    let mut acc = 0.0f32;
    for neighbor in neighbors.into_iter().flatten() {
        if !is_pressure_medium(neighbor.material) {
            continue;
        }
        let neighbor_p = sanitize_pressure(neighbor.pressure);
        acc += neighbor_p - self_p;
    }

    sanitize_pressure(self_p + PRESSURE_DIFFUSION_RATE * acc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{MATERIAL_EMPTY, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER};

    fn right(material: u32, pressure: f32) -> [Option<PressureNeighbor>; 4] {
        [
            None,
            None,
            Some(PressureNeighbor { material, pressure }),
            None,
        ]
    }

    #[test]
    fn only_liquid_and_gas_are_pressure_media() {
        assert!(is_pressure_medium(MATERIAL_WATER));
        assert!(is_pressure_medium(MATERIAL_STEAM));
        assert!(!is_pressure_medium(MATERIAL_STONE));
        assert!(!is_pressure_medium(MATERIAL_EMPTY));
    }

    #[test]
    fn pressure_moves_down_gradient_without_spontaneous_loss() {
        let hot = pressure_step(MATERIAL_WATER, 100.0, right(MATERIAL_WATER, 0.0));
        let cold = pressure_step(MATERIAL_WATER, 0.0, right(MATERIAL_WATER, 100.0));
        assert!((hot - 80.0).abs() < 1.0e-5, "hot={hot}");
        assert!((cold - 20.0).abs() < 1.0e-5, "cold={cold}");
        assert!(((hot + cold) - 100.0).abs() < 1.0e-5);
    }

    #[test]
    fn isolated_pressure_does_not_decay_with_time() {
        let next = pressure_step(MATERIAL_STEAM, 42.0, [None, None, None, None]);
        assert_eq!(next, 42.0);
    }

    #[test]
    fn empty_and_static_do_not_transmit_pressure() {
        let through_empty = pressure_step(MATERIAL_WATER, 12.0, right(MATERIAL_EMPTY, 100.0));
        let through_stone = pressure_step(MATERIAL_WATER, 12.0, right(MATERIAL_STONE, 100.0));
        assert_eq!(through_empty, 12.0);
        assert_eq!(through_stone, 12.0);
        assert_eq!(pressure_step(MATERIAL_EMPTY, 99.0, [None; 4]), PRESSURE_REFERENCE);
        assert_eq!(pressure_step(MATERIAL_STONE, 99.0, [None; 4]), PRESSURE_REFERENCE);
    }

    #[test]
    fn four_neighbor_update_is_stable() {
        let zero = Some(PressureNeighbor {
            material: MATERIAL_WATER,
            pressure: 0.0,
        });
        let next = pressure_step(MATERIAL_WATER, 100.0, [zero; 4]);
        assert!((next - 20.0).abs() < 1.0e-5, "next={next}");
    }

    #[test]
    fn invalid_values_are_sanitized() {
        assert_eq!(sanitize_pressure(f32::NAN), PRESSURE_REFERENCE);
        assert_eq!(sanitize_pressure(f32::INFINITY), PRESSURE_REFERENCE);
        assert_eq!(sanitize_pressure(f32::NEG_INFINITY), PRESSURE_REFERENCE);
        assert_eq!(sanitize_pressure(-4.0), PRESSURE_REFERENCE);
        assert_eq!(sanitize_pressure(PRESSURE_MAX * 2.0), PRESSURE_MAX);
    }
}
