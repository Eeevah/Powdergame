//! Thermal Environment state and occupancy helpers (TE-1).
//!
//! Air is a full-resolution Environment field, never a foreground Matter.
//! This module deliberately contains only state validation, classification,
//! whole-parcel accounting, and canonical staging. Inter-cell Air flow,
//! thermal exchange, and pressure coupling begin in later gates.

use crate::{is_valid_cell_material_value, MATERIAL_EMPTY};

pub const STANDARD_AIR_MASS: f32 = 1.0;
pub const AIR_HEAT_CAPACITY: f32 = 1.0;
pub const AIR_ZERO_OFFSET: f32 = 273.15;
pub const AMBIENT_TEMPERATURE_C: f32 = 20.0;
pub const AMBIENT_TEMPERATURE_ABS: f32 = 293.15;
pub const STANDARD_AIR_ENERGY: f32 = 293.15;
pub const VACUUM_THRESHOLD: f32 = 0.0;
pub const AIR_PRESENT_THRESHOLD: f32 = 0.5;
pub const AIR_MASS_MAX: f32 = 16.0;
pub const AIR_TEMPERATURE_ABS_MIN: f32 = 1.0;
pub const AIR_TEMPERATURE_ABS_MAX: f32 = 2_273.15;
pub const AIR_ENERGY_MAX: f32 = 36_370.4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AirState {
    pub mass: f32,
    pub energy: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentClass {
    Vacuum,
    LowPressure,
    Atmosphere,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyEnvironmentSeed {
    StandardAtmosphere,
    Vacuum,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnvironmentImage {
    pub air_mass: Vec<f32>,
    pub air_energy: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentError {
    NonFinite,
    Negative,
    VacuumPairMismatch,
    MassOutOfRange,
    EnergyOutOfRange,
    SpecificEnergyOutOfRange,
    InvalidMaterial { index: usize, value: u32 },
}

impl std::fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonFinite => write!(f, "Air mass and energy must be finite"),
            Self::Negative => write!(f, "Air mass and energy must be non-negative"),
            Self::VacuumPairMismatch => write!(f, "Vacuum must be exact (0, 0)"),
            Self::MassOutOfRange => write!(f, "Air mass exceeds AIR_MASS_MAX"),
            Self::EnergyOutOfRange => write!(f, "Air energy exceeds AIR_ENERGY_MAX"),
            Self::SpecificEnergyOutOfRange => {
                write!(
                    f,
                    "Air specific energy is outside the gameplay safety range"
                )
            }
            Self::InvalidMaterial { index, value } => {
                write!(f, "invalid Material value {value} at image index {index}")
            }
        }
    }
}

impl std::error::Error for EnvironmentError {}

pub const fn vacuum_air_state() -> AirState {
    AirState {
        mass: 0.0,
        energy: 0.0,
    }
}

pub const fn standard_air_state() -> AirState {
    AirState {
        mass: STANDARD_AIR_MASS,
        energy: STANDARD_AIR_ENERGY,
    }
}

pub fn validate_air_state(state: AirState) -> Result<(), EnvironmentError> {
    if !state.mass.is_finite() || !state.energy.is_finite() {
        return Err(EnvironmentError::NonFinite);
    }
    if state.mass < 0.0 || state.energy < 0.0 {
        return Err(EnvironmentError::Negative);
    }
    if state.mass == VACUUM_THRESHOLD || state.energy == 0.0 {
        return if state.mass == 0.0 && state.energy == 0.0 {
            Ok(())
        } else {
            Err(EnvironmentError::VacuumPairMismatch)
        };
    }
    if state.mass > AIR_MASS_MAX {
        return Err(EnvironmentError::MassOutOfRange);
    }
    if state.energy > AIR_ENERGY_MAX {
        return Err(EnvironmentError::EnergyOutOfRange);
    }
    let specific = air_specific_energy(state).ok_or(EnvironmentError::SpecificEnergyOutOfRange)?;
    if !(AIR_TEMPERATURE_ABS_MIN..=AIR_TEMPERATURE_ABS_MAX).contains(&specific) {
        return Err(EnvironmentError::SpecificEnergyOutOfRange);
    }
    Ok(())
}

pub fn classify_air_state(state: AirState) -> Result<EnvironmentClass, EnvironmentError> {
    validate_air_state(state)?;
    if state.mass == 0.0 {
        Ok(EnvironmentClass::Vacuum)
    } else if state.mass < AIR_PRESENT_THRESHOLD {
        Ok(EnvironmentClass::LowPressure)
    } else {
        Ok(EnvironmentClass::Atmosphere)
    }
}

pub fn air_specific_energy(state: AirState) -> Option<f32> {
    if state.mass <= 0.0 || !state.mass.is_finite() || !state.energy.is_finite() {
        return None;
    }
    let value = state.energy / state.mass;
    value.is_finite().then_some(value)
}

pub fn air_temperature_absolute_like(state: AirState) -> Option<f32> {
    air_specific_energy(state).map(|specific| specific / AIR_HEAT_CAPACITY)
}

pub fn air_temperature_celsius_like(state: AirState) -> Option<f32> {
    air_temperature_absolute_like(state).map(|temperature| temperature - AIR_ZERO_OFFSET)
}

pub fn parcel_has_full_headroom(receiver: AirState, parcel: AirState) -> bool {
    validate_air_state(receiver).is_ok()
        && validate_air_state(parcel).is_ok()
        && receiver.mass + parcel.mass <= AIR_MASS_MAX
        && receiver.energy + parcel.energy <= AIR_ENERGY_MAX
        && (receiver.mass + parcel.mass).is_finite()
        && (receiver.energy + parcel.energy).is_finite()
}

pub fn combine_whole_parcel(receiver: AirState, parcel: AirState) -> Option<AirState> {
    if !parcel_has_full_headroom(receiver, parcel) {
        return None;
    }
    let combined = AirState {
        mass: receiver.mass + parcel.mass,
        energy: receiver.energy + parcel.energy,
    };
    validate_air_state(combined).is_ok().then_some(combined)
}

pub fn environment_image_from_materials(
    materials: &[u32],
    empty_seed: EmptyEnvironmentSeed,
) -> Result<EnvironmentImage, EnvironmentError> {
    let empty = match empty_seed {
        EmptyEnvironmentSeed::StandardAtmosphere => standard_air_state(),
        EmptyEnvironmentSeed::Vacuum => vacuum_air_state(),
    };
    let mut air_mass = Vec::with_capacity(materials.len());
    let mut air_energy = Vec::with_capacity(materials.len());
    for (index, &material) in materials.iter().enumerate() {
        if !is_valid_cell_material_value(material) {
            return Err(EnvironmentError::InvalidMaterial {
                index,
                value: material,
            });
        }
        let state = if material == MATERIAL_EMPTY {
            empty
        } else {
            vacuum_air_state()
        };
        air_mass.push(state.mass);
        air_energy.push(state.energy);
    }
    Ok(EnvironmentImage {
        air_mass,
        air_energy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MATERIAL_BOUNDARY_BLOCK, MATERIAL_WATER};

    #[test]
    fn standard_air_and_vacuum_are_exact_and_classified() {
        let standard = standard_air_state();
        assert_eq!(validate_air_state(standard), Ok(()));
        assert_eq!(
            classify_air_state(standard),
            Ok(EnvironmentClass::Atmosphere)
        );
        assert_eq!(air_temperature_celsius_like(standard), Some(20.0));
        assert_eq!(
            classify_air_state(vacuum_air_state()),
            Ok(EnvironmentClass::Vacuum)
        );
    }

    #[test]
    fn positive_residual_is_low_pressure_not_deleted() {
        let residual = AirState {
            mass: 0.25,
            energy: 0.25 * AMBIENT_TEMPERATURE_ABS,
        };
        assert_eq!(
            classify_air_state(residual),
            Ok(EnvironmentClass::LowPressure)
        );
    }

    #[test]
    fn invalid_pairs_and_ranges_are_rejected_without_clamping() {
        for invalid in [
            AirState {
                mass: 0.0,
                energy: 1.0,
            },
            AirState {
                mass: 1.0,
                energy: 0.0,
            },
            AirState {
                mass: -1.0,
                energy: 1.0,
            },
            AirState {
                mass: f32::NAN,
                energy: 1.0,
            },
            AirState {
                mass: AIR_MASS_MAX + 1.0,
                energy: STANDARD_AIR_ENERGY,
            },
            AirState {
                mass: 1.0,
                energy: AIR_ENERGY_MAX + 1.0,
            },
            AirState {
                mass: 1.0,
                energy: AIR_TEMPERATURE_ABS_MIN / 2.0,
            },
        ] {
            assert!(validate_air_state(invalid).is_err(), "accepted {invalid:?}");
        }
    }

    #[test]
    fn whole_parcel_combination_is_lossless_or_rejected() {
        let receiver = standard_air_state();
        let parcel = AirState {
            mass: 2.0,
            energy: 586.3,
        };
        let combined = combine_whole_parcel(receiver, parcel).unwrap();
        assert_eq!(combined.mass, receiver.mass + parcel.mass);
        assert_eq!(combined.energy, receiver.energy + parcel.energy);
        assert!(combine_whole_parcel(
            AirState {
                mass: AIR_MASS_MAX,
                energy: AIR_ENERGY_MAX
            },
            standard_air_state()
        )
        .is_none());

        let one_ulp_over_mass_max = f32::from_bits(AIR_MASS_MAX.to_bits() + 1);
        assert!(combine_whole_parcel(
            AirState {
                mass: one_ulp_over_mass_max,
                energy: AIR_ENERGY_MAX,
            },
            vacuum_air_state(),
        )
        .is_none());
    }

    #[test]
    fn canonical_image_zeros_occupied_cells_and_seeds_empty_cells() {
        let materials = [MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_WATER];
        let atmosphere =
            environment_image_from_materials(&materials, EmptyEnvironmentSeed::StandardAtmosphere)
                .unwrap();
        assert_eq!(atmosphere.air_mass, vec![0.0, 1.0, 0.0]);
        assert_eq!(atmosphere.air_energy, vec![0.0, 293.15, 0.0]);

        let vacuum =
            environment_image_from_materials(&materials, EmptyEnvironmentSeed::Vacuum).unwrap();
        assert_eq!(vacuum.air_mass, vec![0.0; 3]);
        assert_eq!(vacuum.air_energy, vec![0.0; 3]);
    }
}
