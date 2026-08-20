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
    chunk_count, chunks_x, chunks_y, stable_ticks_update, ACTIVITY_ALL_BITS, ACTIVITY_MATTER,
    ACTIVITY_PRESSURE, ACTIVITY_REACTION, ACTIVITY_THERMAL, CHUNK_STATE_RUNNABLE,
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
    air_specific_energy, air_temperature_absolute_like, air_temperature_celsius_like,
    classify_air_state, combine_whole_parcel, environment_image_from_materials,
    parcel_has_full_headroom, standard_air_state, vacuum_air_state, validate_air_state, AirState,
    EmptyEnvironmentSeed, EnvironmentClass, EnvironmentError, EnvironmentImage, AIR_ENERGY_MAX,
    AIR_HEAT_CAPACITY, AIR_MASS_MAX, AIR_PRESENT_THRESHOLD, AIR_TEMPERATURE_ABS_MAX,
    AIR_TEMPERATURE_ABS_MIN, AIR_ZERO_OFFSET, AMBIENT_TEMPERATURE_ABS, AMBIENT_TEMPERATURE_C,
    STANDARD_AIR_ENERGY, STANDARD_AIR_MASS, VACUUM_THRESHOLD,
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
    is_phase_candidate, phase_descriptor_table, select_phase_effect, select_phase_transition,
    PhaseEffect, PhaseGpuDescriptor, PhaseTransition, TemperatureCondition, ICE_MELT_ABOVE,
    MAX_PHASE_MATTER_YIELD, NO_PHASE_TARGET, PHASE_IDENTITY_MATTER_YIELD, STEAM_CONDENSE_BELOW,
    WATER_BOIL_ABOVE, WATER_BOIL_BLOCKED_PRESSURE, WATER_BOIL_MATTER_YIELD, WATER_FREEZE_BELOW,
};
pub use pressure::{
    is_pressure_medium, pressure_step, sanitize_pressure, PressureNeighbor,
    PRESSURE_DIFFUSION_RATE, PRESSURE_MAX, PRESSURE_REFERENCE,
};
pub use rupture::{
    rupture_threshold, rupture_threshold_table, should_rupture, WOOD_RUPTURE_THRESHOLD,
};
pub use thermal::{
    conductivity_table, heat_capacity_table, sanitize_temperature, thermal_properties,
    thermal_step, ThermalNeighbor, ThermalProperties, TEMPERATURE_REFERENCE, THERMAL_DEADBAND,
    THERMAL_MAX_DELTA, THERMAL_MIN_CAPACITY, THERMAL_RATE,
};
pub use world_config::{ConfigError, WorldConfig};
