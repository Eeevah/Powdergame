//! Material identity and registry.
//!
//! Contract (`ADR-0001`, `MATERIAL_SPEC` §2/§3, `SIMULATION_SPEC` §3):
//! - `material_id` is identity. It is never a property ordering.
//! - `EMPTY` is a valid *absence* value for a cell but is **not** a
//!   registered Matter and has no descriptor.
//! - `Void` is not a Material ID and has no array slot.
//!
//! G2 adds the minimum Matter identities needed for local movement and the
//! `MovementClass` descriptor. G3 adds `density_rank` — a small gameplay
//! ordering, NOT a physical constant and never per-cell state
//! (`SIMULATION_SPEC` §12, `MATERIAL_SPEC` §5). G4-A adds cheap
//! `thermal_conductivity` / `heat_capacity` (gameplay scalars, not SI).
//! G4-B adds `phase_transitions` — the Material's own small ordered rule
//! set for temperature-based 1:1 self transitions (Ice ↔ Water ↔ Steam).
//! G4-C adds `combustion` — the Material's generic ignition/sustain/heat/
//! fuel-life descriptor (Wood and Oil share one grammar, `REACTION_SPEC`
//! §11; finite fuel = `burn_duration_ticks`).

use crate::combustion::{
    CombustionDescriptor, COMBUSTION_OIL_BURN_DURATION, COMBUSTION_OIL_HEAT_PER_TICK,
    COMBUSTION_OIL_IGNITION, COMBUSTION_OIL_SUSTAIN, COMBUSTION_WOOD_BURN_DURATION,
    COMBUSTION_WOOD_HEAT_PER_TICK, COMBUSTION_WOOD_IGNITION, COMBUSTION_WOOD_SUSTAIN,
};
use crate::decay::{DecayDescriptor, SMOKE_LIFETIME_TICKS};
use crate::phase::{PhaseTransition, TemperatureCondition};

/// Absence of Matter in a cell. `EMPTY` is not Matter (ADR-0001).
pub const MATERIAL_EMPTY: u32 = 0;
/// Editable outer boundary Block — a real, registered Matter.
pub const MATERIAL_BOUNDARY_BLOCK: u32 = 1;
/// Stone — STATIC registered Matter.
pub const MATERIAL_STONE: u32 = 2;
/// Sand — POWDER registered Matter.
pub const MATERIAL_SAND: u32 = 3;
/// Water — LIQUID registered Matter.
pub const MATERIAL_WATER: u32 = 4;
/// Oil — LIQUID registered Matter.
pub const MATERIAL_OIL: u32 = 5;
/// Steam — GAS registered Matter.
pub const MATERIAL_STEAM: u32 = 6;
/// Smoke — GAS registered Matter.
pub const MATERIAL_SMOKE: u32 = 7;
/// Ice — STATIC registered Matter (G4-B phase transition target).
pub const MATERIAL_ICE: u32 = 8;
/// Wood — STATIC registered Matter (G4-C combustible).
pub const MATERIAL_WOOD: u32 = 9;

// G3 baseline density ranks (gameplay ordering, `MATERIAL_SPEC` §5):
// heavier sinks below lighter. These are not physical units and may be
// retuned by gameplay validation.
pub const DENSITY_RANK_STEAM: u32 = 20;
pub const DENSITY_RANK_SMOKE: u32 = 30;
pub const DENSITY_RANK_OIL: u32 = 70;
pub const DENSITY_RANK_WATER: u32 = 90;
pub const DENSITY_RANK_SAND: u32 = 150;

// G4-A cheap thermal scalars (not SI). Heat capacity is shared by the
// two liquids so a conductivity-only contrast is testable. Boundary
// conductivity is 0 so the outer ring is not a hidden heat sink.
pub const THERMAL_K_BOUNDARY: f32 = 0.0;
pub const THERMAL_K_STONE: f32 = 0.50;
pub const THERMAL_K_SAND: f32 = 0.30;
pub const THERMAL_K_WATER: f32 = 1.00;
pub const THERMAL_K_OIL: f32 = 0.20;
pub const THERMAL_K_STEAM: f32 = 0.10;
pub const THERMAL_K_SMOKE: f32 = 0.10;
pub const THERMAL_K_ICE: f32 = 0.60;
pub const THERMAL_K_WOOD: f32 = 0.15;
pub const THERMAL_C_BOUNDARY: f32 = 2.0;
pub const THERMAL_C_STONE: f32 = 2.0;
pub const THERMAL_C_SAND: f32 = 1.5;
pub const THERMAL_C_LIQUID: f32 = 2.5;
pub const THERMAL_C_GAS: f32 = 0.8;
pub const THERMAL_C_ICE: f32 = 2.0;
pub const THERMAL_C_WOOD: f32 = 2.0;

/// Movement behavior family (G2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementClass {
    /// No normal movement (Boundary Block, Stone, Ice, Wood).
    Static,
    /// Falls down / down-diagonal (Sand).
    Powder,
    /// Down / down-diagonal / lateral (Water, Oil).
    Liquid,
    /// Up / up-diagonal / lateral (Steam, Smoke).
    Gas,
}

impl MovementClass {
    /// Compact u32 encoding used in GPU params (0 = none/static ... 3 = gas).
    pub const fn as_u32(self) -> u32 {
        match self {
            MovementClass::Static => 0,
            MovementClass::Powder => 1,
            MovementClass::Liquid => 2,
            MovementClass::Gas => 3,
        }
    }

    /// Decodes the compact u32 encoding.
    pub const fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(MovementClass::Static),
            1 => Some(MovementClass::Powder),
            2 => Some(MovementClass::Liquid),
            3 => Some(MovementClass::Gas),
            _ => None,
        }
    }
}

/// Minimum descriptor for a registered Matter identity.
///
/// G2 adds `movement_class`; G3 adds `density_rank`; G4-A adds cheap
/// thermal scalars; G4-B adds the Material's own `phase_transitions`
/// (ordered First-Match, `REACTION_SPEC` §6); G4-C adds the generic
/// `combustion` descriptor (Wood/Oil share one grammar, finite fuel).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialDescriptor {
    pub id: u32,
    pub name: &'static str,
    pub movement_class: MovementClass,
    /// Small gameplay density ordering for local displacement.
    ///
    /// `None`: no movable density — `EMPTY`, STATIC Matter and unknown ids
    /// are never density-displacement targets. `Some(rank)`: movable Matter;
    /// only `>`/`==`/`<` comparisons are meaningful.
    pub density_rank: Option<u32>,
    /// Gameplay thermal conductivity. `0` means this Matter does not
    /// exchange heat (the outer Boundary ring).
    pub thermal_conductivity: f32,
    /// Gameplay heat capacity. Higher → slower temperature change.
    pub heat_capacity: f32,
    /// Temperature-based 1:1 self transitions, pre-ordered First-Match.
    ///
    /// Empty means this Matter has no phase transition (EMPTY, Stone, Sand,
    /// Oil, Smoke, Wood, Boundary). This is Material data — never per-cell
    /// state.
    pub phase_transitions: &'static [PhaseTransition],
    /// Generic combustion properties (ignition/sustain/heat + fuel life).
    ///
    /// `None` means this Matter never combusts. This is Material data —
    /// the per-cell `flags` field stores only the combustion bits (bool
    /// state + u12 fuel progress).
    pub combustion: Option<CombustionDescriptor>,
    /// Generic decay properties (finite lifetime + target material).
    ///
    /// `None` means this Matter never decays. This is Material data —
    /// the per-cell `flags` field stores only the decay age bits (u12).
    pub decay: Option<DecayDescriptor>,
}

/// The registered Matter catalog.
///
/// `EMPTY` intentionally has **no entry** here: `registry_contains(EMPTY)`
/// is `false` and `registry_lookup(EMPTY)` is `None`.
pub const MATERIAL_REGISTRY: &[MaterialDescriptor] = &[
    MaterialDescriptor {
        id: MATERIAL_BOUNDARY_BLOCK,
        name: "Boundary Block",
        movement_class: MovementClass::Static,
        density_rank: None,
        thermal_conductivity: THERMAL_K_BOUNDARY,
        heat_capacity: THERMAL_C_BOUNDARY,
        phase_transitions: &[],
        combustion: None,
        decay: None,
    },
    MaterialDescriptor {
        id: MATERIAL_STONE,
        name: "Stone",
        movement_class: MovementClass::Static,
        density_rank: None,
        thermal_conductivity: THERMAL_K_STONE,
        heat_capacity: THERMAL_C_STONE,
        phase_transitions: &[],
        combustion: None,
        decay: None,
    },
    MaterialDescriptor {
        id: MATERIAL_SAND,
        name: "Sand",
        movement_class: MovementClass::Powder,
        density_rank: Some(DENSITY_RANK_SAND),
        thermal_conductivity: THERMAL_K_SAND,
        heat_capacity: THERMAL_C_SAND,
        phase_transitions: &[],
        combustion: None,
        decay: None,
    },
    MaterialDescriptor {
        id: MATERIAL_WATER,
        name: "Water",
        movement_class: MovementClass::Liquid,
        density_rank: Some(DENSITY_RANK_WATER),
        thermal_conductivity: THERMAL_K_WATER,
        heat_capacity: THERMAL_C_LIQUID,
        phase_transitions: &[
            PhaseTransition {
                condition: TemperatureCondition::Below,
                threshold: crate::phase::WATER_FREEZE_BELOW,
                target_material: MATERIAL_ICE,
                matter_yield: crate::phase::PHASE_IDENTITY_MATTER_YIELD,
                blocked_pressure: 0.0,
            },
            PhaseTransition {
                condition: TemperatureCondition::Above,
                threshold: crate::phase::WATER_BOIL_ABOVE,
                target_material: MATERIAL_STEAM,
                matter_yield: crate::phase::WATER_BOIL_MATTER_YIELD,
                blocked_pressure: crate::phase::WATER_BOIL_BLOCKED_PRESSURE,
            },
        ],
        combustion: None,
        decay: None,
    },
    MaterialDescriptor {
        id: MATERIAL_OIL,
        name: "Oil",
        movement_class: MovementClass::Liquid,
        density_rank: Some(DENSITY_RANK_OIL),
        thermal_conductivity: THERMAL_K_OIL,
        heat_capacity: THERMAL_C_LIQUID,
        phase_transitions: &[],
        combustion: Some(CombustionDescriptor {
            ignition_threshold: COMBUSTION_OIL_IGNITION,
            sustain_threshold: COMBUSTION_OIL_SUSTAIN,
            heat_per_tick: COMBUSTION_OIL_HEAT_PER_TICK,
            burn_duration_ticks: COMBUSTION_OIL_BURN_DURATION,
        }),
        decay: None,
    },
    MaterialDescriptor {
        id: MATERIAL_STEAM,
        name: "Steam",
        movement_class: MovementClass::Gas,
        density_rank: Some(DENSITY_RANK_STEAM),
        thermal_conductivity: THERMAL_K_STEAM,
        heat_capacity: THERMAL_C_GAS,
        phase_transitions: &[PhaseTransition {
            condition: TemperatureCondition::Below,
            threshold: crate::phase::STEAM_CONDENSE_BELOW,
            target_material: MATERIAL_WATER,
            matter_yield: crate::phase::PHASE_IDENTITY_MATTER_YIELD,
            blocked_pressure: 0.0,
        }],
        combustion: None,
        decay: None,
    },
    MaterialDescriptor {
        id: MATERIAL_SMOKE,
        name: "Smoke",
        movement_class: MovementClass::Gas,
        density_rank: Some(DENSITY_RANK_SMOKE),
        thermal_conductivity: THERMAL_K_SMOKE,
        heat_capacity: THERMAL_C_GAS,
        phase_transitions: &[],
        combustion: None,
        decay: Some(DecayDescriptor {
            lifetime_ticks: SMOKE_LIFETIME_TICKS,
            target_material: MATERIAL_EMPTY,
        }),
    },
    MaterialDescriptor {
        id: MATERIAL_ICE,
        name: "Ice",
        movement_class: MovementClass::Static,
        density_rank: None,
        thermal_conductivity: THERMAL_K_ICE,
        heat_capacity: THERMAL_C_ICE,
        phase_transitions: &[PhaseTransition {
            condition: TemperatureCondition::Above,
            threshold: crate::phase::ICE_MELT_ABOVE,
            target_material: MATERIAL_WATER,
            matter_yield: crate::phase::PHASE_IDENTITY_MATTER_YIELD,
            blocked_pressure: 0.0,
        }],
        combustion: None,
        decay: None,
    },
    MaterialDescriptor {
        id: MATERIAL_WOOD,
        name: "Wood",
        movement_class: MovementClass::Static,
        density_rank: None,
        thermal_conductivity: THERMAL_K_WOOD,
        heat_capacity: THERMAL_C_WOOD,
        phase_transitions: &[],
        combustion: Some(CombustionDescriptor {
            ignition_threshold: COMBUSTION_WOOD_IGNITION,
            sustain_threshold: COMBUSTION_WOOD_SUSTAIN,
            heat_per_tick: COMBUSTION_WOOD_HEAT_PER_TICK,
            burn_duration_ticks: COMBUSTION_WOOD_BURN_DURATION,
        }),
        decay: None,
    },
];

/// Returns `true` if `id` is a registered Matter.
///
/// `EMPTY` is never registered.
pub fn registry_contains(id: u32) -> bool {
    MATERIAL_REGISTRY.iter().any(|m| m.id == id)
}

/// Looks up a registered Matter descriptor. `EMPTY` yields `None`.
pub fn registry_lookup(id: u32) -> Option<&'static MaterialDescriptor> {
    MATERIAL_REGISTRY.iter().find(|m| m.id == id)
}

/// Returns the movement family of a registered Matter. `EMPTY` is `None`.
pub fn movement_class(id: u32) -> Option<MovementClass> {
    registry_lookup(id).map(|m| m.movement_class)
}

/// Returns the density rank of a registered Matter.
///
/// `None` for `EMPTY`, STATIC Matter and unknown ids — those are never
/// density-displacement targets.
pub fn density_rank(id: u32) -> Option<u32> {
    registry_lookup(id).and_then(|m| m.density_rank)
}

/// Compact per-ID movement-class table for GPU upload.
///
/// `table[id]` is `MovementClass::as_u32()`. `EMPTY` (and any unknown id)
/// maps to `0` (no movement). Sized generously for future material ids;
/// shaders only read entries for valid ids.
pub fn movement_class_table() -> [u32; 16] {
    let mut table = [MovementClass::Static.as_u32(); 16];
    for m in MATERIAL_REGISTRY {
        table[m.id as usize] = m.movement_class.as_u32();
    }
    table
}

/// Compact per-ID density-rank table for GPU upload.
///
/// `table[id]` is the gameplay density rank, or `0` for no movable density
/// (`EMPTY`, STATIC, unknown ids). This is a Material property upload, not
/// per-cell state — no `density_current[]`/`density_next[]` buffers exist.
pub fn density_table() -> [u32; 16] {
    let mut table = [0u32; 16];
    for m in MATERIAL_REGISTRY {
        if let Some(rank) = m.density_rank {
            table[m.id as usize] = rank;
        }
    }
    table
}

/// Returns `true` if `value` may be stored as a cell's material value.
///
/// `EMPTY` is a valid absence value; registered Matter IDs are valid Matter
/// values. Any other `u32` is rejected — unknown IDs must not enter the
/// world through normal authoring/edit paths.
pub fn is_valid_cell_material_value(value: u32) -> bool {
    value == MATERIAL_EMPTY || registry_contains(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_zero_contract() {
        assert_eq!(MATERIAL_EMPTY, 0);
    }

    #[test]
    fn empty_is_not_registered_matter() {
        assert!(!registry_contains(MATERIAL_EMPTY));
        assert_eq!(registry_lookup(MATERIAL_EMPTY), None);
        assert_eq!(movement_class(MATERIAL_EMPTY), None);
        assert_eq!(density_rank(MATERIAL_EMPTY), None);
        assert_eq!(MATERIAL_REGISTRY.len(), 9);
    }

    #[test]
    fn all_g2_materials_are_registered() {
        let expected = [
            (MATERIAL_BOUNDARY_BLOCK, "Boundary Block"),
            (MATERIAL_STONE, "Stone"),
            (MATERIAL_SAND, "Sand"),
            (MATERIAL_WATER, "Water"),
            (MATERIAL_OIL, "Oil"),
            (MATERIAL_STEAM, "Steam"),
            (MATERIAL_SMOKE, "Smoke"),
            (MATERIAL_ICE, "Ice"),
            (MATERIAL_WOOD, "Wood"),
        ];
        for (id, name) in expected {
            let d = registry_lookup(id).unwrap_or_else(|| panic!("{name} must be registered"));
            assert_eq!(d.id, id);
            assert_eq!(d.name, name);
        }
    }

    #[test]
    fn movement_classes_are_mapped() {
        assert_eq!(
            movement_class(MATERIAL_BOUNDARY_BLOCK),
            Some(MovementClass::Static)
        );
        assert_eq!(movement_class(MATERIAL_STONE), Some(MovementClass::Static));
        assert_eq!(movement_class(MATERIAL_SAND), Some(MovementClass::Powder));
        assert_eq!(movement_class(MATERIAL_WATER), Some(MovementClass::Liquid));
        assert_eq!(movement_class(MATERIAL_OIL), Some(MovementClass::Liquid));
        assert_eq!(movement_class(MATERIAL_STEAM), Some(MovementClass::Gas));
        assert_eq!(movement_class(MATERIAL_SMOKE), Some(MovementClass::Gas));
        assert_eq!(movement_class(MATERIAL_ICE), Some(MovementClass::Static));
        assert_eq!(movement_class(MATERIAL_WOOD), Some(MovementClass::Static));
    }

    #[test]
    fn movement_class_encoding_round_trip() {
        for class in [
            MovementClass::Static,
            MovementClass::Powder,
            MovementClass::Liquid,
            MovementClass::Gas,
        ] {
            assert_eq!(MovementClass::from_u32(class.as_u32()), Some(class));
        }
        assert_eq!(MovementClass::from_u32(99), None);
    }

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<u32> = MATERIAL_REGISTRY.iter().map(|m| m.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), MATERIAL_REGISTRY.len());
        // G1 ID contract is preserved: no renumbering.
        assert_eq!(MATERIAL_EMPTY, 0);
        assert_eq!(MATERIAL_BOUNDARY_BLOCK, 1);
        assert_eq!(MATERIAL_STONE, 2);
    }

    #[test]
    fn unknown_ids_are_rejected() {
        for unknown in [10u32, 42, u32::MAX] {
            assert!(!registry_contains(unknown));
            assert_eq!(registry_lookup(unknown), None);
            assert!(!is_valid_cell_material_value(unknown));
            assert_eq!(density_rank(unknown), None);
        }
    }

    #[test]
    fn valid_cell_values() {
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
            MATERIAL_WOOD,
        ] {
            assert!(is_valid_cell_material_value(id), "id {id}");
        }
    }

    #[test]
    fn movement_class_table_maps_ids() {
        let table = movement_class_table();
        assert_eq!(table[MATERIAL_EMPTY as usize], 0); // EMPTY has no movement
        assert_eq!(table[MATERIAL_BOUNDARY_BLOCK as usize], 0); // static
        assert_eq!(table[MATERIAL_STONE as usize], 0); // static
        assert_eq!(table[MATERIAL_SAND as usize], 1); // powder
        assert_eq!(table[MATERIAL_WATER as usize], 2); // liquid
        assert_eq!(table[MATERIAL_OIL as usize], 2); // liquid
        assert_eq!(table[MATERIAL_STEAM as usize], 3); // gas
        assert_eq!(table[MATERIAL_SMOKE as usize], 3); // gas
        assert_eq!(table[MATERIAL_ICE as usize], 0); // static
        assert_eq!(table[MATERIAL_WOOD as usize], 0); // static
    }

    #[test]
    fn g3_density_ranks_are_assigned() {
        assert_eq!(density_rank(MATERIAL_SAND), Some(150));
        assert_eq!(density_rank(MATERIAL_WATER), Some(90));
        assert_eq!(density_rank(MATERIAL_OIL), Some(70));
        assert_eq!(density_rank(MATERIAL_STEAM), Some(20));
        assert_eq!(density_rank(MATERIAL_SMOKE), Some(30));
    }

    #[test]
    fn static_and_empty_have_no_density_rank() {
        assert_eq!(density_rank(MATERIAL_BOUNDARY_BLOCK), None);
        assert_eq!(density_rank(MATERIAL_STONE), None);
        assert_eq!(density_rank(MATERIAL_ICE), None);
        assert_eq!(density_rank(MATERIAL_WOOD), None);
        assert_eq!(density_rank(MATERIAL_EMPTY), None);
    }

    #[test]
    fn density_table_maps_ids_only() {
        let table = density_table();
        assert_eq!(table[MATERIAL_EMPTY as usize], 0);
        assert_eq!(table[MATERIAL_BOUNDARY_BLOCK as usize], 0);
        assert_eq!(table[MATERIAL_STONE as usize], 0);
        assert_eq!(table[MATERIAL_SAND as usize], 150);
        assert_eq!(table[MATERIAL_WATER as usize], 90);
        assert_eq!(table[MATERIAL_OIL as usize], 70);
        assert_eq!(table[MATERIAL_STEAM as usize], 20);
        assert_eq!(table[MATERIAL_SMOKE as usize], 30);
        assert_eq!(table[MATERIAL_ICE as usize], 0);
        assert_eq!(table[MATERIAL_WOOD as usize], 0);
        for unknown in [10usize, 15] {
            assert_eq!(table[unknown], 0);
        }
    }

    #[test]
    fn g4a_thermal_scalars_are_assigned() {
        let water = registry_lookup(MATERIAL_WATER).unwrap();
        let oil = registry_lookup(MATERIAL_OIL).unwrap();
        let stone = registry_lookup(MATERIAL_STONE).unwrap();
        let boundary = registry_lookup(MATERIAL_BOUNDARY_BLOCK).unwrap();
        assert_eq!(water.thermal_conductivity, THERMAL_K_WATER);
        assert_eq!(oil.thermal_conductivity, THERMAL_K_OIL);
        assert!(water.thermal_conductivity > oil.thermal_conductivity);
        assert_eq!(water.heat_capacity, oil.heat_capacity);
        assert!(stone.thermal_conductivity > 0.0);
        assert_eq!(boundary.thermal_conductivity, THERMAL_K_BOUNDARY);
    }

    #[test]
    fn ice_is_registered_static_with_thermal() {
        let ice = registry_lookup(MATERIAL_ICE).unwrap();
        assert_eq!(ice.name, "Ice");
        assert_eq!(ice.movement_class, MovementClass::Static);
        assert_eq!(ice.density_rank, None);
        assert_eq!(ice.thermal_conductivity, THERMAL_K_ICE);
        assert_eq!(ice.heat_capacity, THERMAL_C_ICE);
        assert!(ice.thermal_conductivity.is_finite());
        assert!(ice.heat_capacity.is_finite());
        assert_eq!(ice.phase_transitions.len(), 1);
        assert_eq!(ice.combustion, None);
    }

    #[test]
    fn wood_is_registered_static_with_combustion() {
        let wood = registry_lookup(MATERIAL_WOOD).unwrap();
        assert_eq!(wood.name, "Wood");
        assert_eq!(wood.movement_class, MovementClass::Static);
        assert_eq!(wood.density_rank, None);
        assert_eq!(wood.thermal_conductivity, THERMAL_K_WOOD);
        assert_eq!(wood.heat_capacity, THERMAL_C_WOOD);
        assert!(wood.thermal_conductivity.is_finite());
        assert!(wood.heat_capacity.is_finite());
        assert!(wood.phase_transitions.is_empty());
        let combustion = wood.combustion.expect("Wood must be combustible");
        assert_eq!(combustion.ignition_threshold, COMBUSTION_WOOD_IGNITION);
        assert_eq!(combustion.sustain_threshold, COMBUSTION_WOOD_SUSTAIN);
        assert_eq!(combustion.heat_per_tick, COMBUSTION_WOOD_HEAT_PER_TICK);
        assert_eq!(
            combustion.burn_duration_ticks,
            COMBUSTION_WOOD_BURN_DURATION
        );
    }

    #[test]
    fn oil_has_combustion_descriptor() {
        let oil = registry_lookup(MATERIAL_OIL).unwrap();
        let combustion = oil.combustion.expect("Oil must be combustible");
        assert_eq!(combustion.ignition_threshold, COMBUSTION_OIL_IGNITION);
        assert_eq!(combustion.sustain_threshold, COMBUSTION_OIL_SUSTAIN);
        assert_eq!(combustion.heat_per_tick, COMBUSTION_OIL_HEAT_PER_TICK);
        assert_eq!(combustion.burn_duration_ticks, COMBUSTION_OIL_BURN_DURATION);
    }

    #[test]
    fn g3_rank_ordering() {
        // Gameplay ordering used by local displacement: heavier sinks below.
        let ranks = [
            (MATERIAL_STEAM, DENSITY_RANK_STEAM),
            (MATERIAL_SMOKE, DENSITY_RANK_SMOKE),
            (MATERIAL_OIL, DENSITY_RANK_OIL),
            (MATERIAL_WATER, DENSITY_RANK_WATER),
            (MATERIAL_SAND, DENSITY_RANK_SAND),
        ];
        for w in ranks.windows(2) {
            assert!(
                w[0].1 < w[1].1,
                "{} (rank {}) must be lighter than {} (rank {})",
                w[0].0,
                w[0].1,
                w[1].0,
                w[1].1
            );
        }
    }
}
