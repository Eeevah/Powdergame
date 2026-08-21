//! TE-3 pressure-decoupled Water/Steam/Ice phase enthalpy.
//!
//! One foreground phase-family Cell is one Water-equivalent quantity.  Latent
//! progress is Matter-owned and all identity transitions are 1:1.  This module
//! is the CPU semantic reference for the production GPU normalization pass.

use crate::material::{registry_lookup, MATERIAL_REGISTRY};
use crate::thermal::sanitize_temperature;

pub const T_MELT: f32 = 0.0;
pub const T_BOIL: f32 = 100.0;
pub const LATENT_FUSION: f32 = 80.0;
pub const LATENT_VAPORIZATION: f32 = 480.0;
pub const CONDENSATION_SURFACE_MAX_C: f32 = 80.0;
pub const CONDENSATION_MIN_DELTA_C: f32 = 10.0;
pub const FREE_AIR_NUCLEATION_MAX_C: f32 = 70.0;
pub const NUCLEATION_RADIUS: i32 = 2;
pub const PHASE_H_ABS_TOL: f32 = 1.0e-3;
pub const PHASE_H_REL_TOL: f32 = 2.0e-6;

pub const WATER_FREEZE_BELOW: f32 = -2.0;
pub const ICE_MELT_ABOVE: f32 = 2.0;
pub const STEAM_CONDENSE_BELOW: f32 = 95.0;
pub const WATER_BOIL_ABOVE: f32 = 100.0;

pub const PHASE_IDENTITY_MATTER_YIELD: u32 = 1;
pub const WATER_BOIL_MATTER_YIELD: u32 = 1;
pub const WATER_BOIL_BLOCKED_PRESSURE: f32 = 0.0;
pub const MAX_PHASE_MATTER_YIELD: u32 = 1;
pub const NO_PHASE_TARGET: u32 = u32::MAX;

const C_ICE: f32 = 2.0;
const C_WATER: f32 = 2.5;
const C_STEAM: f32 = 0.8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureCondition {
    Below,
    Above,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseTransition {
    pub condition: TemperatureCondition,
    pub threshold: f32,
    pub target_material: u32,
    pub matter_yield: u32,
    pub blocked_pressure: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseGpuDescriptor {
    pub below_target: u32,
    pub above_target: u32,
    pub below_yield: u32,
    pub above_yield: u32,
    pub below_threshold: f32,
    pub above_threshold: f32,
    pub below_blocked_pressure: f32,
    pub above_blocked_pressure: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseEffect {
    pub target_material: u32,
    pub matter_yield: u32,
    pub blocked_pressure: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhaseContext {
    pub gas_facing: bool,
    pub condensation_sink: bool,
    pub free_air_seed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseTransitionKind {
    None,
    Melt,
    Freeze,
    Boil,
    Condense,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseNormalization {
    pub material: u32,
    pub temperature: f32,
    pub phase_energy: f32,
    pub transition: PhaseTransitionKind,
}

pub fn canonical_phase_energy(material: u32) -> f32 {
    match material {
        crate::material::MATERIAL_ICE => -LATENT_FUSION,
        crate::material::MATERIAL_STEAM => LATENT_VAPORIZATION,
        _ => 0.0,
    }
}

pub fn valid_phase_energy(material: u32, energy: f32) -> bool {
    if !energy.is_finite() {
        return false;
    }
    match material {
        crate::material::MATERIAL_ICE => (-LATENT_FUSION..=0.0).contains(&energy),
        crate::material::MATERIAL_WATER => (-LATENT_FUSION..=LATENT_VAPORIZATION).contains(&energy),
        crate::material::MATERIAL_STEAM => (0.0..=LATENT_VAPORIZATION).contains(&energy),
        _ => energy == 0.0,
    }
}

pub fn sensible_enthalpy(material: u32, temperature: f32) -> f32 {
    let t = sanitize_temperature(temperature);
    match material {
        crate::material::MATERIAL_ICE => C_ICE * t,
        crate::material::MATERIAL_WATER => C_WATER * t,
        crate::material::MATERIAL_STEAM => C_WATER * T_BOIL + C_STEAM * (t - T_BOIL),
        _ => 0.0,
    }
}

pub fn phase_enthalpy(material: u32, temperature: f32, phase_energy: f32) -> f32 {
    sensible_enthalpy(material, temperature) + phase_energy
}

fn water_from_enthalpy(h: f32, gas_facing: bool, allow_freeze: bool) -> PhaseNormalization {
    use crate::material::{MATERIAL_ICE, MATERIAL_STEAM, MATERIAL_WATER};
    if allow_freeze && h <= -LATENT_FUSION {
        return PhaseNormalization {
            material: MATERIAL_ICE,
            temperature: sanitize_temperature((h + LATENT_FUSION) / C_ICE),
            phase_energy: -LATENT_FUSION,
            transition: PhaseTransitionKind::Freeze,
        };
    }
    if allow_freeze && h < 0.0 {
        return PhaseNormalization {
            material: MATERIAL_WATER,
            temperature: T_MELT,
            phase_energy: h,
            transition: PhaseTransitionKind::None,
        };
    }
    let boiling_start_h = C_WATER * T_BOIL;
    let steam_start_h = boiling_start_h + LATENT_VAPORIZATION;
    if gas_facing && h >= steam_start_h {
        return PhaseNormalization {
            material: MATERIAL_STEAM,
            temperature: sanitize_temperature(T_BOIL + (h - steam_start_h) / C_STEAM),
            phase_energy: LATENT_VAPORIZATION,
            transition: PhaseTransitionKind::Boil,
        };
    }
    if gas_facing && h > boiling_start_h {
        return PhaseNormalization {
            material: MATERIAL_WATER,
            temperature: T_BOIL,
            phase_energy: h - boiling_start_h,
            transition: PhaseTransitionKind::None,
        };
    }
    PhaseNormalization {
        material: MATERIAL_WATER,
        temperature: sanitize_temperature(h / C_WATER),
        phase_energy: 0.0,
        transition: PhaseTransitionKind::None,
    }
}

/// Repartitions one already-transferred local enthalpy state.  Invalid phase
/// energy is rejected; callers must not clamp corrupt authoritative state.
pub fn normalize_phase_enthalpy(
    material: u32,
    trial_temperature: f32,
    phase_energy: f32,
    context: PhaseContext,
) -> Result<PhaseNormalization, &'static str> {
    use crate::material::{MATERIAL_ICE, MATERIAL_STEAM, MATERIAL_WATER};
    if !valid_phase_energy(material, phase_energy) {
        return Err("invalid phase-energy state");
    }
    let t = sanitize_temperature(trial_temperature);
    let h = phase_enthalpy(material, t, phase_energy);
    let result = match material {
        MATERIAL_ICE => {
            let initiated = phase_energy > -LATENT_FUSION || t > ICE_MELT_ABOVE;
            if !initiated {
                PhaseNormalization {
                    material,
                    temperature: t,
                    phase_energy,
                    transition: PhaseTransitionKind::None,
                }
            } else if h < -LATENT_FUSION {
                PhaseNormalization {
                    material,
                    temperature: sanitize_temperature((h + LATENT_FUSION) / C_ICE),
                    phase_energy: -LATENT_FUSION,
                    transition: PhaseTransitionKind::None,
                }
            } else if h < 0.0 {
                PhaseNormalization {
                    material,
                    temperature: T_MELT,
                    phase_energy: h,
                    transition: PhaseTransitionKind::None,
                }
            } else {
                let mut out = water_from_enthalpy(h, context.gas_facing, false);
                if out.material == MATERIAL_WATER {
                    out.transition = PhaseTransitionKind::Melt;
                }
                out
            }
        }
        MATERIAL_WATER if phase_energy < 0.0 => water_from_enthalpy(h, false, true),
        MATERIAL_WATER if phase_energy > 0.0 => {
            let boiling_start_h = C_WATER * T_BOIL;
            let steam_start_h = boiling_start_h + LATENT_VAPORIZATION;
            if h < boiling_start_h {
                PhaseNormalization {
                    material,
                    temperature: sanitize_temperature(h / C_WATER),
                    phase_energy: 0.0,
                    transition: PhaseTransitionKind::None,
                }
            } else if h < steam_start_h {
                PhaseNormalization {
                    material,
                    temperature: T_BOIL,
                    phase_energy: h - boiling_start_h,
                    transition: PhaseTransitionKind::None,
                }
            } else if context.gas_facing {
                PhaseNormalization {
                    material: MATERIAL_STEAM,
                    temperature: sanitize_temperature(T_BOIL + (h - steam_start_h) / C_STEAM),
                    phase_energy: LATENT_VAPORIZATION,
                    transition: PhaseTransitionKind::Boil,
                }
            } else {
                PhaseNormalization {
                    material,
                    temperature: sanitize_temperature((h - LATENT_VAPORIZATION) / C_WATER),
                    phase_energy: LATENT_VAPORIZATION,
                    transition: PhaseTransitionKind::None,
                }
            }
        }
        MATERIAL_WATER => {
            let freeze = t < WATER_FREEZE_BELOW;
            let boil = t > WATER_BOIL_ABOVE && context.gas_facing;
            if freeze {
                water_from_enthalpy(h, false, true)
            } else if boil {
                water_from_enthalpy(h, true, false)
            } else {
                PhaseNormalization {
                    material,
                    temperature: t,
                    phase_energy: 0.0,
                    transition: PhaseTransitionKind::None,
                }
            }
        }
        MATERIAL_STEAM => {
            let may_condense = phase_energy < LATENT_VAPORIZATION
                || ((context.condensation_sink || context.free_air_seed)
                    && t < STEAM_CONDENSE_BELOW);
            let boiling_start_h = C_WATER * T_BOIL;
            let steam_start_h = boiling_start_h + LATENT_VAPORIZATION;
            if !may_condense || h >= steam_start_h {
                PhaseNormalization {
                    material,
                    temperature: sanitize_temperature(
                        T_BOIL + (h - LATENT_VAPORIZATION - boiling_start_h) / C_STEAM,
                    ),
                    phase_energy: LATENT_VAPORIZATION,
                    transition: PhaseTransitionKind::None,
                }
            } else if h > boiling_start_h {
                PhaseNormalization {
                    material,
                    temperature: T_BOIL,
                    phase_energy: h - boiling_start_h,
                    transition: PhaseTransitionKind::None,
                }
            } else {
                let mut out = water_from_enthalpy(h, false, true);
                if out.material == MATERIAL_WATER {
                    out.transition = PhaseTransitionKind::Condense;
                }
                out
            }
        }
        _ => PhaseNormalization {
            material,
            temperature: t,
            phase_energy: 0.0,
            transition: PhaseTransitionKind::None,
        },
    };
    Ok(result)
}

pub fn select_phase_effect(material_id: u32, temperature: f32) -> Option<PhaseEffect> {
    let rules = registry_lookup(material_id)?.phase_transitions;
    let t = sanitize_temperature(temperature);
    rules.iter().find_map(|rule| {
        let hit = match rule.condition {
            TemperatureCondition::Below => t < rule.threshold,
            TemperatureCondition::Above => t > rule.threshold,
        };
        hit.then_some(PhaseEffect {
            target_material: rule.target_material,
            matter_yield: rule.matter_yield,
            blocked_pressure: rule.blocked_pressure,
        })
    })
}

pub fn select_phase_transition(material_id: u32, temperature: f32) -> Option<u32> {
    select_phase_effect(material_id, temperature).map(|effect| effect.target_material)
}

pub fn is_phase_candidate(material_id: u32) -> bool {
    registry_lookup(material_id)
        .map(|m| !m.phase_transitions.is_empty())
        .unwrap_or(false)
}

pub fn phase_descriptor_table() -> [PhaseGpuDescriptor; 16] {
    let none = PhaseGpuDescriptor {
        below_target: NO_PHASE_TARGET,
        above_target: NO_PHASE_TARGET,
        below_yield: PHASE_IDENTITY_MATTER_YIELD,
        above_yield: PHASE_IDENTITY_MATTER_YIELD,
        below_threshold: 0.0,
        above_threshold: 0.0,
        below_blocked_pressure: 0.0,
        above_blocked_pressure: 0.0,
    };
    let mut table = [none; 16];
    for m in MATERIAL_REGISTRY {
        let mut desc = none;
        for rule in m.phase_transitions {
            match rule.condition {
                TemperatureCondition::Below if desc.below_target == NO_PHASE_TARGET => {
                    desc.below_target = rule.target_material;
                    desc.below_yield = rule.matter_yield;
                    desc.below_threshold = rule.threshold;
                    desc.below_blocked_pressure = rule.blocked_pressure;
                }
                TemperatureCondition::Above if desc.above_target == NO_PHASE_TARGET => {
                    desc.above_target = rule.target_material;
                    desc.above_yield = rule.matter_yield;
                    desc.above_threshold = rule.threshold;
                    desc.above_blocked_pressure = rule.blocked_pressure;
                }
                _ => {}
            }
        }
        table[m.id as usize] = desc;
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{MATERIAL_ICE, MATERIAL_STEAM, MATERIAL_WATER};

    #[test]
    fn canonical_values_and_water_pressure_are_locked() {
        assert_eq!(canonical_phase_energy(MATERIAL_ICE), -80.0);
        assert_eq!(canonical_phase_energy(MATERIAL_WATER), 0.0);
        assert_eq!(canonical_phase_energy(MATERIAL_STEAM), 480.0);
        let water = phase_descriptor_table()[MATERIAL_WATER as usize];
        assert_eq!(water.above_yield, 1);
        assert_eq!(water.above_blocked_pressure, 0.0);
    }

    #[test]
    fn boiling_is_surface_gated_and_one_to_one() {
        let buried =
            normalize_phase_enthalpy(MATERIAL_WATER, 300.0, 0.0, PhaseContext::default()).unwrap();
        assert_eq!(buried.material, MATERIAL_WATER);
        assert_eq!(buried.phase_energy, 0.0);
        let open = normalize_phase_enthalpy(
            MATERIAL_WATER,
            300.0,
            0.0,
            PhaseContext {
                gas_facing: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(open.material, MATERIAL_STEAM);
        assert_eq!(open.phase_energy, LATENT_VAPORIZATION);
        let before = phase_enthalpy(MATERIAL_WATER, 300.0, 0.0);
        let after = phase_enthalpy(open.material, open.temperature, open.phase_energy);
        assert!((before - after).abs() <= PHASE_H_ABS_TOL);
    }

    #[test]
    fn buried_ready_water_preserves_and_reverses_progress() {
        let ready = normalize_phase_enthalpy(
            MATERIAL_WATER,
            120.0,
            LATENT_VAPORIZATION,
            PhaseContext::default(),
        )
        .unwrap();
        assert_eq!(ready.material, MATERIAL_WATER);
        assert_eq!(ready.phase_energy, LATENT_VAPORIZATION);
        let reverse = normalize_phase_enthalpy(
            MATERIAL_WATER,
            90.0,
            LATENT_VAPORIZATION,
            PhaseContext::default(),
        )
        .unwrap();
        assert!(reverse.phase_energy < LATENT_VAPORIZATION);
        assert_eq!(reverse.temperature, T_BOIL);
    }

    #[test]
    fn condensation_needs_sink_or_seed_but_partial_is_owned() {
        let stalled = normalize_phase_enthalpy(
            MATERIAL_STEAM,
            60.0,
            LATENT_VAPORIZATION,
            PhaseContext::default(),
        )
        .unwrap();
        assert_eq!(stalled.material, MATERIAL_STEAM);
        assert_eq!(stalled.phase_energy, LATENT_VAPORIZATION);
        let seeded = normalize_phase_enthalpy(
            MATERIAL_STEAM,
            60.0,
            LATENT_VAPORIZATION,
            PhaseContext {
                free_air_seed: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(seeded.phase_energy < LATENT_VAPORIZATION);
        let partial =
            normalize_phase_enthalpy(MATERIAL_STEAM, 90.0, 200.0, PhaseContext::default()).unwrap();
        assert!(partial.phase_energy < 200.0);
    }

    #[test]
    fn invalid_authoritative_energy_is_rejected() {
        assert!(normalize_phase_enthalpy(
            MATERIAL_WATER,
            20.0,
            LATENT_VAPORIZATION + 1.0,
            PhaseContext::default()
        )
        .is_err());
        assert!(
            normalize_phase_enthalpy(MATERIAL_STEAM, 20.0, f32::NAN, PhaseContext::default())
                .is_err()
        );
    }
}
