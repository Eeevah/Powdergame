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
pub const STANDARD_AIR_PRESSURE: f32 = 1.0;
pub const VACUUM_THRESHOLD: f32 = 0.0;
pub const AIR_PRESENT_THRESHOLD: f32 = 0.5;
pub const AIR_MASS_MAX: f32 = 16.0;
pub const AIR_TEMPERATURE_ABS_MIN: f32 = 1.0;
pub const AIR_TEMPERATURE_ABS_MAX: f32 = 2_273.15;
pub const AIR_ENERGY_MAX: f32 = 36_370.4;
pub const AIR_FLOW_RATE: f32 = 0.125;
pub const AIR_MAX_OUTFLOW_FRACTION: f32 = 0.25;
pub const AIR_PRESSURE_DEADBAND: f32 = 0.001;
pub const AIR_FLOW_SCALE_SAFETY: f32 = 0.999_999;
pub const EMPTY_EMPTY_AIR_PERMEABILITY: f32 = 1.0;
pub const ALL_OTHER_AIR_PERMEABILITY: f32 = 0.0;
pub const ENVIRONMENT_UPDATE_INTERVAL: u32 = 1;
pub const AIR_THERMAL_CONDUCTIVITY: f32 = 0.025;
pub const MATTER_AIR_INTERFACE_CONDUCTANCE: f32 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum EnvironmentBoundaryMode {
    #[default]
    Sealed = 0,
    FixedStandardAtmosphereReservoir = 1,
}

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

pub fn air_pressure_like(state: AirState) -> f32 {
    let Some(absolute_temperature) = air_temperature_absolute_like(state) else {
        return 0.0;
    };
    let pressure = STANDARD_AIR_PRESSURE
        * (state.mass / STANDARD_AIR_MASS)
        * (absolute_temperature / AMBIENT_TEMPERATURE_ABS);
    if validate_air_state(state).is_ok() && pressure.is_finite() {
        pressure
    } else {
        0.0
    }
}

/// Canonical derived pressure vocabulary for TE-2. This is deliberately not
/// coupled to the production Matter/structure pressure field.
pub fn derived_air_pressure(state: AirState) -> f32 {
    air_pressure_like(state)
}

pub fn air_face_permeability(source_material: u32, target_material: u32) -> f32 {
    if source_material == MATERIAL_EMPTY && target_material == MATERIAL_EMPTY {
        EMPTY_EMPTY_AIR_PERMEABILITY
    } else {
        ALL_OTHER_AIR_PERMEABILITY
    }
}

pub fn raw_air_face_outflow(donor: AirState, receiver: AirState, permeability: f32) -> f32 {
    let excess = pressure_excess(donor, receiver);
    let raw = AIR_FLOW_RATE * permeability.max(0.0) * excess;
    if raw.is_finite() {
        raw
    } else {
        0.0
    }
}

pub fn pressure_excess(donor: AirState, receiver: AirState) -> f32 {
    (derived_air_pressure(donor) - derived_air_pressure(receiver) - AIR_PRESSURE_DEADBAND).max(0.0)
}

pub fn raw_directed_air_flow(donor: AirState, receiver: AirState, permeability: f32) -> f32 {
    raw_air_face_outflow(donor, receiver, permeability)
}

pub fn donor_outflow_scale(donor_mass: f32, sum_raw_out: f32) -> f32 {
    if !donor_mass.is_finite() || donor_mass < 0.0 || !sum_raw_out.is_finite() || sum_raw_out <= 0.0
    {
        return 0.0;
    }
    (AIR_MAX_OUTFLOW_FRACTION * donor_mass / sum_raw_out).min(1.0) * AIR_FLOW_SCALE_SAFETY
}

pub fn receiver_accept_scale(
    receiver: AirState,
    sum_raw_in_mass: f32,
    sum_raw_in_energy: f32,
) -> Option<f32> {
    validate_air_state(receiver).ok()?;
    if !sum_raw_in_mass.is_finite()
        || sum_raw_in_mass < 0.0
        || !sum_raw_in_energy.is_finite()
        || sum_raw_in_energy < 0.0
    {
        return None;
    }
    let mass_headroom = AIR_MASS_MAX - receiver.mass;
    let energy_headroom = AIR_ENERGY_MAX - receiver.energy;
    if !mass_headroom.is_finite()
        || mass_headroom < 0.0
        || !energy_headroom.is_finite()
        || energy_headroom < 0.0
    {
        return None;
    }
    let mut scale = 1.0f32;
    if sum_raw_in_mass > 0.0 {
        scale = scale.min(mass_headroom / sum_raw_in_mass);
    }
    if sum_raw_in_energy > 0.0 {
        scale = scale.min(energy_headroom / sum_raw_in_energy);
    }
    if !scale.is_finite() || scale < 0.0 {
        return None;
    }
    Some(if scale < 1.0 {
        scale * AIR_FLOW_SCALE_SAFETY
    } else {
        1.0
    })
}

pub fn canonical_air_face_transfer(
    donor: AirState,
    receiver: AirState,
    donor_scale: f32,
    receiver_scale: f32,
    permeability: f32,
) -> Option<AirState> {
    let specific_energy = air_specific_energy(donor)?;
    let mass = raw_air_face_outflow(donor, receiver, permeability)
        * donor_scale
            .clamp(0.0, 1.0)
            .min(receiver_scale.clamp(0.0, 1.0));
    let energy = mass * specific_energy;
    (mass.is_finite() && energy.is_finite()).then_some(AirState { mass, energy })
}

pub fn canonical_directed_face_flow(
    donor: AirState,
    receiver: AirState,
    donor_scale: f32,
    receiver_scale: f32,
    permeability: f32,
) -> Option<AirState> {
    canonical_air_face_transfer(donor, receiver, donor_scale, receiver_scale, permeability)
}

pub fn advected_energy(mass: f32, donor: AirState) -> Option<f32> {
    let energy = mass * air_specific_energy(donor)?;
    (mass.is_finite() && mass >= 0.0 && energy.is_finite() && energy >= 0.0).then_some(energy)
}

/// Applies already-arbitrated directed transfers to one self-writing cell.
/// Invalid or over-capacity results are rejected rather than clamped.
pub fn air_transport_cell_step(
    current: AirState,
    outgoing: &[AirState],
    incoming: &[AirState],
) -> Result<AirState, EnvironmentError> {
    validate_air_state(current)?;
    let outgoing_mass = outgoing.iter().map(|flow| flow.mass).sum::<f32>();
    let outgoing_energy = outgoing.iter().map(|flow| flow.energy).sum::<f32>();
    let incoming_mass = incoming.iter().map(|flow| flow.mass).sum::<f32>();
    let incoming_energy = incoming.iter().map(|flow| flow.energy).sum::<f32>();
    let next = AirState {
        mass: current.mass - outgoing_mass + incoming_mass,
        energy: current.energy - outgoing_energy + incoming_energy,
    };
    validate_air_state(next)?;
    Ok(next)
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ReservoirFaceAccounting {
    pub air_mass: f64,
    pub advected_energy: f64,
    pub passive_heat: f64,
}

impl ReservoirFaceAccounting {
    pub fn record(&mut self, signed_mass: f32, signed_advected_energy: f32, signed_heat: f32) {
        self.air_mass += signed_mass as f64;
        self.advected_energy += signed_advected_energy as f64;
        self.passive_heat += signed_heat as f64;
    }
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
        assert!((derived_air_pressure(standard) - STANDARD_AIR_PRESSURE).abs() <= 1.0e-6);
        let hot = AirState {
            mass: 1.0,
            energy: 773.15,
        };
        assert!(derived_air_pressure(hot) > derived_air_pressure(standard));
        let half_mass = AirState {
            mass: 0.5,
            energy: 0.5 * STANDARD_AIR_ENERGY,
        };
        assert!((derived_air_pressure(half_mass) - 0.5).abs() <= 1.0e-6);
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

    #[test]
    fn multi_source_receiver_headroom_is_conservative_and_lossless() {
        let donor = AirState {
            mass: AIR_MASS_MAX,
            energy: AIR_MASS_MAX * STANDARD_AIR_ENERGY,
        };
        let receiver = AirState {
            mass: 15.9,
            energy: 15.9 * STANDARD_AIR_ENERGY,
        };
        let raw = raw_directed_air_flow(donor, receiver, EMPTY_EMPTY_AIR_PERMEABILITY);
        let donor_scale = donor_outflow_scale(donor.mass, raw);
        let receiver_scale = receiver_accept_scale(
            receiver,
            raw * 4.0,
            advected_energy(raw * 4.0, donor).unwrap(),
        )
        .unwrap();
        let transfer = canonical_directed_face_flow(
            donor,
            receiver,
            donor_scale,
            receiver_scale,
            EMPTY_EMPTY_AIR_PERMEABILITY,
        )
        .unwrap();
        assert!(transfer.mass <= AIR_MAX_OUTFLOW_FRACTION * donor.mass);

        let receiver_next = air_transport_cell_step(receiver, &[], &[transfer; 4]).unwrap();
        let donor_next = air_transport_cell_step(donor, &[transfer], &[]).unwrap();
        assert!(receiver_next.mass <= AIR_MASS_MAX);
        assert!(receiver_next.energy <= AIR_ENERGY_MAX);
        let before_mass = receiver.mass + donor.mass * 4.0;
        let after_mass = receiver_next.mass + donor_next.mass * 4.0;
        let before_energy = receiver.energy + donor.energy * 4.0;
        let after_energy = receiver_next.energy + donor_next.energy * 4.0;
        assert!((before_mass - after_mass).abs() <= 1.0e-5);
        assert!((before_energy - after_energy).abs() <= 1.0e-3);
    }

    #[test]
    fn reservoir_accounting_is_explicit_and_signed() {
        let mut accounting = ReservoirFaceAccounting::default();
        accounting.record(0.25, 73.2875, -0.5);
        accounting.record(-0.1, -29.315, 0.25);
        assert!((accounting.air_mass - 0.15).abs() <= 1.0e-6);
        assert!((accounting.advected_energy - 43.9725).abs() <= 1.0e-4);
        assert!((accounting.passive_heat + 0.25).abs() <= 1.0e-6);
    }
}
