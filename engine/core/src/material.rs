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
//! Phase / combustion / ignition properties are still not here.

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
pub const THERMAL_C_BOUNDARY: f32 = 2.0;
pub const THERMAL_C_STONE: f32 = 2.0;
pub const THERMAL_C_SAND: f32 = 1.5;
pub const THERMAL_C_LIQUID: f32 = 2.5;
pub const THERMAL_C_GAS: f32 = 0.8;

/// Movement behavior family (G2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementClass {
    /// No normal movement (Boundary Block, Stone).
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
/// thermal scalars. No phase / combustion / ignition fields yet.
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
    },
    MaterialDescriptor {
        id: MATERIAL_STONE,
        name: "Stone",
        movement_class: MovementClass::Static,
        density_rank: None,
        thermal_conductivity: THERMAL_K_STONE,
        heat_capacity: THERMAL_C_STONE,
    },
    MaterialDescriptor {
        id: MATERIAL_SAND,
        name: "Sand",
        movement_class: MovementClass::Powder,
        density_rank: Some(DENSITY_RANK_SAND),
        thermal_conductivity: THERMAL_K_SAND,
        heat_capacity: THERMAL_C_SAND,
    },
    MaterialDescriptor {
        id: MATERIAL_WATER,
        name: "Water",
        movement_class: MovementClass::Liquid,
        density_rank: Some(DENSITY_RANK_WATER),
        thermal_conductivity: THERMAL_K_WATER,
        heat_capacity: THERMAL_C_LIQUID,
    },
    MaterialDescriptor {
        id: MATERIAL_OIL,
        name: "Oil",
        movement_class: MovementClass::Liquid,
        density_rank: Some(DENSITY_RANK_OIL),
        thermal_conductivity: THERMAL_K_OIL,
        heat_capacity: THERMAL_C_LIQUID,
    },
    MaterialDescriptor {
        id: MATERIAL_STEAM,
        name: "Steam",
        movement_class: MovementClass::Gas,
        density_rank: Some(DENSITY_RANK_STEAM),
        thermal_conductivity: THERMAL_K_STEAM,
        heat_capacity: THERMAL_C_GAS,
    },
    MaterialDescriptor {
        id: MATERIAL_SMOKE,
        name: "Smoke",
        movement_class: MovementClass::Gas,
        density_rank: Some(DENSITY_RANK_SMOKE),
        thermal_conductivity: THERMAL_K_SMOKE,
        heat_capacity: THERMAL_C_GAS,
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
        assert_eq!(MATERIAL_REGISTRY.len(), 7);
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
        for unknown in [8u32, 42, u32::MAX] {
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
        for unknown in [8usize, 15] {
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
