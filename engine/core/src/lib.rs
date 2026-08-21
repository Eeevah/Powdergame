//! Powdergame Simulation Core.
//!
//! GPU/Window-independent contracts and calculations.
//!
//! Architectural boundary (see `docs/architecture/ARCHITECTURE.md`):
//! - `powdergame-core` MUST NOT depend on winit, wgpu, or any windowing.
//! - `powdergame-gpu` MUST NOT depend on Window/Renderer/Input.
//! - only `apps/windows` may combine core + gpu with the platform layer.

pub mod activity;
pub mod combustion;
pub mod decay;
pub mod domain;
pub mod environment;
pub mod layout;
pub mod material;
pub mod movement;
pub mod phase;
pub mod pressure;
pub mod rupture;
pub mod thermal;
pub mod world_config;

pub use activity::{
    chunk_count, chunks_x, chunks_y, stable_ticks_update, ACTIVITY_ALL_BITS, ACTIVITY_ENVIRONMENT,
    ACTIVITY_MATTER, ACTIVITY_PRESSURE, ACTIVITY_REACTION, ACTIVITY_THERMAL, CHUNK_STATE_RUNNABLE,
    CHUNK_STATE_SLEEPING, DEFAULT_SLEEP_THRESHOLD_TICKS, PRESSURE_ACTIVITY_EPS,
    THERMAL_ACTIVITY_EPS, WAKE_REASON_ALWAYS_ACTIVE, WAKE_REASON_NEIGHBOR_HALO, WAKE_REASON_NONE,
    WAKE_REASON_SELF_ACTIVITY, WAKE_REASON_SETTLING, WAKE_REASON_USER_EDIT,
};
pub use combustion::{
    combustion_descriptor, combustion_flag_mask, combustion_flags_next, combustion_step,
    combustion_table, fuel_progress, pick_smoke_spawn, with_fuel_progress, CombustionDescriptor,
    CombustionGpuDescriptor, CombustionResult, SmokeSpawnDirection, COMBUSTION_MAX_TEMPERATURE,
    COMBUSTION_OIL_BURN_DURATION, COMBUSTION_OIL_HEAT_PER_TICK, COMBUSTION_OIL_IGNITION,
    COMBUSTION_OIL_SUSTAIN, COMBUSTION_WOOD_BURN_DURATION, COMBUSTION_WOOD_HEAT_PER_TICK,
    COMBUSTION_WOOD_IGNITION, COMBUSTION_WOOD_SUSTAIN, FLAG_COMBUSTING, FLAG_FLAME_EVENT,
    FLAG_FUEL_PROGRESS_MASK, FLAG_FUEL_PROGRESS_SHIFT,
};
pub use decay::{
    decay_age, decay_descriptor, decay_flag_mask, decay_step, decay_table, with_decay_age,
    DecayDescriptor, DecayGpuDescriptor, DecayResult, FLAG_DECAY_AGE_MASK, FLAG_DECAY_AGE_SHIFT,
    SMOKE_LIFETIME_TICKS,
};

pub use domain::{initial_material_ids, Domain};
pub use environment::{
    advected_energy, air_face_permeability, air_pressure_like, air_specific_energy,
    air_temperature_absolute_like, air_temperature_celsius_like, air_transport_cell_step,
    canonical_air_face_transfer, canonical_directed_face_flow, classify_air_state,
    combine_whole_parcel, derived_air_pressure, donor_outflow_scale,
    environment_image_from_materials, parcel_has_full_headroom, pressure_excess,
    raw_air_face_outflow, raw_directed_air_flow, receiver_accept_scale, standard_air_state,
    vacuum_air_state, validate_air_state, AirState, EmptyEnvironmentSeed, EnvironmentBoundaryMode,
    EnvironmentClass, EnvironmentError, EnvironmentImage, ReservoirFaceAccounting, AIR_ENERGY_MAX,
    AIR_FLOW_RATE, AIR_FLOW_SCALE_SAFETY, AIR_HEAT_CAPACITY, AIR_MASS_MAX,
    AIR_MAX_OUTFLOW_FRACTION, AIR_PRESENT_THRESHOLD, AIR_PRESSURE_DEADBAND,
    AIR_TEMPERATURE_ABS_MAX, AIR_TEMPERATURE_ABS_MIN, AIR_THERMAL_CONDUCTIVITY, AIR_ZERO_OFFSET,
    ALL_OTHER_AIR_PERMEABILITY, AMBIENT_TEMPERATURE_ABS, AMBIENT_TEMPERATURE_C,
    EMPTY_EMPTY_AIR_PERMEABILITY, ENVIRONMENT_UPDATE_INTERVAL, MATTER_AIR_INTERFACE_CONDUCTANCE,
    STANDARD_AIR_ENERGY, STANDARD_AIR_MASS, STANDARD_AIR_PRESSURE, VACUUM_THRESHOLD,
};
pub use layout::{
    WorldLayout, FLAGS_ELEM_SIZE, MATERIAL_ELEM_SIZE, PRESSURE_ELEM_SIZE, TEMPERATURE_ELEM_SIZE,
};
pub use material::{
    density_rank, density_table, is_valid_cell_material_value, movement_class,
    movement_class_table, registry_contains, registry_lookup, MaterialDescriptor, MovementClass,
    DENSITY_RANK_OIL, DENSITY_RANK_SAND, DENSITY_RANK_SMOKE, DENSITY_RANK_STEAM,
    DENSITY_RANK_WATER, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_OIL,
    MATERIAL_REGISTRY, MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE,
    MATERIAL_WATER, MATERIAL_WOOD, THERMAL_C_GAS, THERMAL_C_ICE, THERMAL_C_LIQUID, THERMAL_C_SAND,
    THERMAL_C_STONE, THERMAL_C_WOOD, THERMAL_K_ICE, THERMAL_K_OIL, THERMAL_K_STONE,
    THERMAL_K_WATER, THERMAL_K_WOOD,
};
pub use movement::{
    density_displacement_allowed, prefer_left, propose_move, CellState, DensityDirection,
    MoveTarget,
};
pub use phase::{
    canonical_phase_energy, is_phase_candidate, normalize_phase_enthalpy, phase_descriptor_table,
    phase_enthalpy, select_phase_effect, select_phase_transition, sensible_enthalpy,
    valid_phase_energy, PhaseContext, PhaseEffect, PhaseGpuDescriptor, PhaseNormalization,
    PhaseTransition, PhaseTransitionKind, TemperatureCondition, CONDENSATION_MIN_DELTA_C,
    CONDENSATION_SURFACE_MAX_C, FREE_AIR_NUCLEATION_MAX_C, ICE_MELT_ABOVE, LATENT_FUSION,
    LATENT_VAPORIZATION, MAX_PHASE_MATTER_YIELD, NO_PHASE_TARGET, NUCLEATION_RADIUS,
    PHASE_H_ABS_TOL, PHASE_H_REL_TOL, PHASE_IDENTITY_MATTER_YIELD, STEAM_CONDENSE_BELOW, T_BOIL,
    T_MELT, WATER_BOIL_ABOVE, WATER_BOIL_BLOCKED_PRESSURE, WATER_BOIL_MATTER_YIELD,
    WATER_FREEZE_BELOW,
};
pub use pressure::{
    is_pressure_medium, pressure_step, sanitize_pressure, PressureNeighbor,
    PRESSURE_DIFFUSION_RATE, PRESSURE_MAX, PRESSURE_REFERENCE,
};
pub use rupture::{
    rupture_threshold, rupture_threshold_table, should_rupture, WOOD_RUPTURE_THRESHOLD,
};
pub use thermal::{
    canonical_thermal_face_flux, conductivity_table, energy_like_total, heat_capacity_table,
    passive_thermal_cell_step, sanitize_temperature, thermal_face_conductance,
    thermal_node_for_cell, thermal_properties, thermal_stability_scale, thermal_step,
    thermal_work_exists, ThermalNeighbor, ThermalNode, ThermalProperties, TEMPERATURE_MAX_C,
    TEMPERATURE_MIN_C, TEMPERATURE_REFERENCE, TEMPERATURE_REFERENCE_C, THERMAL_BASE_STEP,
    THERMAL_DEADBAND, THERMAL_DEADBAND_C, THERMAL_MAX_DELTA, THERMAL_MAX_MIX_FRACTION,
    THERMAL_MIN_CAPACITY, THERMAL_RATE,
};
pub use world_config::{ConfigError, WorldConfig};
