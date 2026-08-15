//! Minimal Material identity and registry.
//!
//! Contract (`ADR-0001`, `MATERIAL_SPEC` §2/§3, `SIMULATION_SPEC` §3):
//! - `material_id` is identity. It is never a property ordering.
//! - `EMPTY` is a valid *absence* value for a cell but is **not** a
//!   registered Matter and has no descriptor.
//! - `Void` is not a Material ID and has no array slot.
//! - G1 registers only the minimum identities needed to prove identity and
//!   integrity: Boundary Block and Stone. Movement/physics descriptors
//!   arrive with their own Gates.

/// Absence of Matter in a cell. `EMPTY` is not Matter (ADR-0001).
pub const MATERIAL_EMPTY: u32 = 0;
/// Editable outer boundary Block — a real, registered Matter.
pub const MATERIAL_BOUNDARY_BLOCK: u32 = 1;
/// Stone — a registered Matter. Movement/physics arrive in later Gates.
pub const MATERIAL_STONE: u32 = 2;

/// Minimum descriptor for a registered Matter identity.
///
/// Deliberately minimal for G1: no movement class, density, thermal
/// properties, tags or reaction rules yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialDescriptor {
    pub id: u32,
    pub name: &'static str,
}

/// The registered Matter catalog.
///
/// `EMPTY` intentionally has **no entry** here: `registry_contains(EMPTY)`
/// is `false` and `registry_lookup(EMPTY)` is `None`.
pub const MATERIAL_REGISTRY: &[MaterialDescriptor] = &[
    MaterialDescriptor {
        id: MATERIAL_BOUNDARY_BLOCK,
        name: "Boundary Block",
    },
    MaterialDescriptor {
        id: MATERIAL_STONE,
        name: "Stone",
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
        assert_eq!(MATERIAL_REGISTRY.len(), 2);
    }

    #[test]
    fn boundary_block_is_registered() {
        assert!(registry_contains(MATERIAL_BOUNDARY_BLOCK));
        let d = registry_lookup(MATERIAL_BOUNDARY_BLOCK).expect("boundary block registered");
        assert_eq!(d.id, MATERIAL_BOUNDARY_BLOCK);
        assert_eq!(d.name, "Boundary Block");
    }

    #[test]
    fn stone_is_registered() {
        assert!(registry_contains(MATERIAL_STONE));
        let d = registry_lookup(MATERIAL_STONE).expect("stone registered");
        assert_eq!(d.id, MATERIAL_STONE);
        assert_eq!(d.name, "Stone");
    }

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<u32> = MATERIAL_REGISTRY.iter().map(|m| m.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), MATERIAL_REGISTRY.len());
    }

    #[test]
    fn unknown_ids_are_rejected() {
        for unknown in [3u32, 42, u32::MAX] {
            assert!(!registry_contains(unknown));
            assert_eq!(registry_lookup(unknown), None);
            assert!(!is_valid_cell_material_value(unknown));
        }
    }

    #[test]
    fn valid_cell_values() {
        assert!(is_valid_cell_material_value(MATERIAL_EMPTY));
        assert!(is_valid_cell_material_value(MATERIAL_BOUNDARY_BLOCK));
        assert!(is_valid_cell_material_value(MATERIAL_STONE));
    }
}
