//! G4-D — Material-Owned Decay: Transient matter with finite lifetime (e.g. Smoke).
//!
//! Any Material may define an optional generic `DecayDescriptor` (`MATERIAL_SPEC` §4):
//! ```text
//! newly spawned / entered → age = 0
//! active tick            → age += 1
//! age >= lifetime_ticks  → cell transforms into target_material (e.g. EMPTY)
//! non-decay material     → never decays (decay age bits cleared)
//! ```
//!
//! Decay age is stored in Matter-owned `flags` bits 16..27 (12 bits, 0..4095)
//! and transports on movement/density edges with the Matter identity.
//! Bits 4..15 store `FUEL_PROGRESS` (12 bits, 0..4095).
//! Bits 0..1 store `COMBUSTING` and `FLAME_EVENT`.
//! Bits 28..31 are reserved for future/unrelated subsystem bits (e.g. `1 << 28`).

use crate::material::{registry_lookup, MATERIAL_EMPTY, MATERIAL_REGISTRY};

/// Smoke baseline lifetime: 900 active simulation ticks ≈ 15 s at 60 TPS (gameplay baseline).
pub const SMOKE_LIFETIME_TICKS: u32 = 900;

/// Bit shift for the decay age field in flags (bits 16..27, 12 bits, u12 = 0..4095).
pub const FLAG_DECAY_AGE_SHIFT: u32 = 16;
/// Mask for the decay age field in flags.
pub const FLAG_DECAY_AGE_MASK: u32 = 0x0FFF << FLAG_DECAY_AGE_SHIFT;

/// Generic decay properties owned by a transient Material.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecayDescriptor {
    /// Lifetime in active simulation ticks before decaying into `target_material`.
    pub lifetime_ticks: u32,
    /// Material to transform into after `lifetime_ticks` (e.g. `MATERIAL_EMPTY`).
    pub target_material: u32,
}

/// Compact per-Material decay descriptor for GPU upload (8 bytes each: 2 × u32).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecayGpuDescriptor {
    pub lifetime_ticks: u32,
    pub target_material: u32,
}

/// Returns the decay descriptor of a registered Matter.
pub fn decay_descriptor(id: u32) -> Option<&'static DecayDescriptor> {
    registry_lookup(id).and_then(|m| m.decay.as_ref())
}

/// Extracts the decay age (u12, 0..4095) from a flags word.
pub fn decay_age(flags: u32) -> u32 {
    (flags & FLAG_DECAY_AGE_MASK) >> FLAG_DECAY_AGE_SHIFT
}

/// Replaces the decay age field in a flags word, preserving all other bits.
pub fn with_decay_age(flags: u32, age: u32) -> u32 {
    (flags & !FLAG_DECAY_AGE_MASK) | ((age & 0x0FFF) << FLAG_DECAY_AGE_SHIFT)
}

/// Mask of decay-owned flag bits.
pub fn decay_flag_mask() -> u32 {
    FLAG_DECAY_AGE_MASK
}

/// Compiles the GPU decay table (16 material slots × 8 bytes = 128 bytes).
pub fn decay_table() -> [DecayGpuDescriptor; 16] {
    let none = DecayGpuDescriptor {
        lifetime_ticks: 0,
        target_material: MATERIAL_EMPTY,
    };
    let mut table = [none; 16];
    for m in MATERIAL_REGISTRY {
        if let Some(desc) = m.decay {
            table[m.id as usize] = DecayGpuDescriptor {
                lifetime_ticks: desc.lifetime_ticks,
                target_material: desc.target_material,
            };
        }
    }
    table
}

/// Result of a decay evaluation for a single cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecayResult {
    pub decayed: bool,
    pub next_material: u32,
    pub next_age: u32,
}

/// Pure reference implementation for decay step of a single cell.
pub fn decay_step(material: u32, flags: u32) -> DecayResult {
    if let Some(desc) = decay_descriptor(material) {
        if desc.lifetime_ticks > 0 {
            let current_age = decay_age(flags);
            let next_age = current_age + 1;
            if next_age >= desc.lifetime_ticks {
                return DecayResult {
                    decayed: true,
                    next_material: desc.target_material,
                    next_age: 0,
                };
            } else {
                return DecayResult {
                    decayed: false,
                    next_material: material,
                    next_age,
                };
            }
        }
    }
    DecayResult {
        decayed: false,
        next_material: material,
        next_age: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{MATERIAL_EMPTY, MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STONE};

    #[test]
    fn smoke_has_finite_lifetime() {
        let desc = decay_descriptor(MATERIAL_SMOKE).expect("Smoke has decay descriptor");
        assert_eq!(desc.lifetime_ticks, SMOKE_LIFETIME_TICKS);
        assert_eq!(desc.lifetime_ticks, 900);
    }

    #[test]
    fn smoke_decay_target_is_empty() {
        let desc = decay_descriptor(MATERIAL_SMOKE).expect("Smoke has decay descriptor");
        assert_eq!(desc.target_material, MATERIAL_EMPTY);
    }

    #[test]
    fn non_smoke_has_no_decay() {
        assert!(decay_descriptor(MATERIAL_EMPTY).is_none());
        assert!(decay_descriptor(MATERIAL_STONE).is_none());
        assert!(decay_descriptor(MATERIAL_SAND).is_none());
    }

    #[test]
    fn age_encode_decode() {
        let initial_flags = 0;
        let with_100 = with_decay_age(initial_flags, 100);
        assert_eq!(decay_age(with_100), 100);

        let with_899 = with_decay_age(with_100, 899);
        assert_eq!(decay_age(with_899), 899);
    }

    #[test]
    fn lifetime_boundary_exact() {
        let flags_898 = with_decay_age(0, 898);
        let res_899 = decay_step(MATERIAL_SMOKE, flags_898);
        assert!(!res_899.decayed);
        assert_eq!(res_899.next_material, MATERIAL_SMOKE);
        assert_eq!(res_899.next_age, 899);

        let flags_899 = with_decay_age(0, 899);
        let res_900 = decay_step(MATERIAL_SMOKE, flags_899);
        assert!(res_900.decayed);
        assert_eq!(res_900.next_material, MATERIAL_EMPTY);
        assert_eq!(res_900.next_age, 0);
    }

    #[test]
    fn unrelated_flags_preserved() {
        let unrelated = 1u32 << 28;
        let flags_with_unrelated = with_decay_age(unrelated, 42);
        assert_eq!(flags_with_unrelated & unrelated, unrelated);
        assert_eq!(decay_age(flags_with_unrelated), 42);

        let cleared = flags_with_unrelated & !decay_flag_mask();
        assert_eq!(cleared & unrelated, unrelated);
        assert_eq!(decay_age(cleared), 0);
    }
}
