//! G4-C — Combustion: temperature-based ignition / sustain / heat / Smoke.
//!
//! Wood and Oil share ONE generic combustion grammar (`REACTION_SPEC` §11):
//! a Material-owned `CombustionDescriptor` decides
//!
//! ```text
//! unlit + T >= ignition  → ignite (COMBUSTING + FLAME_EVENT)
//! burning + T >= sustain → keep burning, add heat_per_tick, emit FLAME_EVENT
//! burning + T  < sustain → extinguish
//! non-combustible        → never ignites (combustion bits cleared)
//! ```
//!
//! Contracts:
//! - Combustion is a **Material property** (descriptor), never per-cell
//!   state. The per-cell `flags` field stores only the combustion **bits**
//!   (Matter-owned state, `MATERIAL_SPEC` §4). No fuel mass / burn-age /
//!   Ash (finite fuel depletion is deferred; `No Universal Future State`).
//! - Fire is NOT a Material: flame is Matter + COMBUSTING + heat + a
//!   presentation signal (`FLAG_FLAME_EVENT`), never a permanent orange ID.
//! - No Oxygen requirement — ignition needs only the thermal condition
//!   (`REACTION_SPEC` §11).
//! - Only the combustion-owned bits are touched; unrelated future flag
//!   bits are preserved.
//! - Heat output is finite and gameplay-capped; exact energy conservation
//!   is not a G4 requirement.
//! - Pressure is a spatial field (G5) and is NEVER transported with Matter
//!   on movement edges — only temperature and flags are Matter-owned.

use crate::material::{registry_lookup, MATERIAL_REGISTRY};
use crate::thermal::sanitize_temperature;

/// Persists across ticks: this Matter is actively combusting.
pub const FLAG_COMBUSTING: u32 = 1 << 0;
/// Ephemeral per-tick presentation signal: flame is visible this tick
/// (set on the ignition tick and on every active-combustion tick).
pub const FLAG_FLAME_EVENT: u32 = 1 << 1;

/// Gameplay cap on combustion heat (finite, not a physical unit).
pub const COMBUSTION_MAX_TEMPERATURE: f32 = 1000.0;

/// Oil baseline tuning (relative gameplay scalar, not physical units).
pub const COMBUSTION_OIL_IGNITION: f32 = 75.0;
pub const COMBUSTION_OIL_SUSTAIN: f32 = 45.0;
pub const COMBUSTION_OIL_HEAT_PER_TICK: f32 = 5.0;

/// Wood baseline tuning (relative gameplay scalar, not physical units).
pub const COMBUSTION_WOOD_IGNITION: f32 = 90.0;
pub const COMBUSTION_WOOD_SUSTAIN: f32 = 55.0;
pub const COMBUSTION_WOOD_HEAT_PER_TICK: f32 = 4.0;

/// Generic combustion properties owned by a combustible Material.
///
/// `None` on `MaterialDescriptor.combustion` means this Matter never
/// combusts (`EMPTY`, STATIC Matter, Sand, Water, Ice, Steam, Smoke).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombustionDescriptor {
    /// Temperature at which an unlit Matter ignites.
    pub ignition_threshold: f32,
    /// Temperature below which a burning Matter extinguishes.
    pub sustain_threshold: f32,
    /// Per-tick temperature added while burning (gameplay heat).
    pub heat_per_tick: f32,
}

/// Compact per-Material descriptor for GPU upload (16 bytes each).
///
/// `is_combustible == 0` is the safe sentinel — a non-combustible Matter
/// can never read thresholds as if it were burning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombustionGpuDescriptor {
    pub is_combustible: u32,
    pub ignition_threshold: f32,
    pub sustain_threshold: f32,
    pub heat_per_tick: f32,
}

/// Returns the combustion descriptor of a registered Matter.
///
/// `None` for `EMPTY`, unknown ids and every non-combustible Matter.
pub fn combustion_descriptor(id: u32) -> Option<&'static CombustionDescriptor> {
    registry_lookup(id).and_then(|m| m.combustion.as_ref())
}

/// Combustion-owned flag bits. The combustion pass only ever sets/clears
/// these bits; all other flags bits belong to future subsystems.
pub fn combustion_flag_mask() -> u32 {
    FLAG_COMBUSTING | FLAG_FLAME_EVENT
}

/// Compiles the GPU combustion table (16 material slots × 16 bytes).
///
/// Material data → compact generic table; the shader contains no
/// material-name branches. This is a Material property upload, not
/// per-cell state.
pub fn combustion_table() -> [CombustionGpuDescriptor; 16] {
    let none = CombustionGpuDescriptor {
        is_combustible: 0,
        ignition_threshold: 0.0,
        sustain_threshold: 0.0,
        heat_per_tick: 0.0,
    };
    let mut table = [none; 16];
    for m in MATERIAL_REGISTRY {
        if let Some(desc) = m.combustion {
            table[m.id as usize] = CombustionGpuDescriptor {
                is_combustible: 1,
                ignition_threshold: desc.ignition_threshold,
                sustain_threshold: desc.sustain_threshold,
                heat_per_tick: desc.heat_per_tick,
            };
        }
    }
    table
}

/// Result of one combustion update for a single cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombustionResult {
    /// Whether the cell is combusting after this tick's rule.
    pub burning: bool,
    /// Whether a flame presentation signal is emitted this tick.
    pub flame_event: bool,
    /// The cell's temperature after this tick's rule (always finite).
    pub temperature: f32,
}

/// Pure reference: applies the G4-C combustion rule to one cell.
///
/// This is a unit/reference helper — the production full-world path is the
/// GPU combustion pass, never a CPU world loop. There is no Oxygen input:
/// ignition depends only on the thermal condition.
pub fn combustion_step(material_id: u32, temperature: f32, flags: u32) -> CombustionResult {
    let Some(desc) = combustion_descriptor(material_id) else {
        // Non-combustible Matter / EMPTY / unknown: never burns, and the
        // combustion bits are cleared (the pass preserves unrelated bits).
        return CombustionResult {
            burning: false,
            flame_event: false,
            temperature: sanitize_temperature(temperature),
        };
    };
    let t = sanitize_temperature(temperature);
    let mut burning = flags & FLAG_COMBUSTING != 0;
    if !burning && t >= desc.ignition_threshold {
        burning = true;
    }
    if burning && t < desc.sustain_threshold {
        burning = false;
    }
    // Cap at the gameplay bound but never reduce an already-hotter cell.
    let temperature = if burning {
        (t + desc.heat_per_tick).min(t.max(COMBUSTION_MAX_TEMPERATURE))
    } else {
        t
    };
    CombustionResult {
        burning,
        flame_event: burning,
        temperature: sanitize_temperature(temperature),
    }
}

/// What `flags_next` should be after the combustion rule: the combustion
/// bits are set/cleared, all unrelated future flag bits are preserved.
pub fn combustion_flags_next(flags: u32, result: &CombustionResult) -> u32 {
    let mut next = flags & !combustion_flag_mask();
    if result.burning {
        next |= FLAG_COMBUSTING;
    }
    if result.flame_event {
        next |= FLAG_FLAME_EVENT;
    }
    next
}

/// Local smoke spawn direction (max one 1-cell candidate per source).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmokeSpawnDirection {
    Up,
    UpLeft,
    UpRight,
    Left,
    Right,
}

/// Pure reference for the smoke spawn stencil (no long-distance scan).
///
/// Each boolean tells whether that local neighbor is an in-domain EMPTY
/// cell. Ordered First-Match (`REACTION_SPEC` §6):
///   up → up-diagonal (parity ordered) → lateral (parity ordered) → none
pub fn pick_smoke_spawn(
    up: bool,
    up_left: bool,
    up_right: bool,
    left: bool,
    right: bool,
    parity: u32,
) -> Option<SmokeSpawnDirection> {
    if up {
        return Some(SmokeSpawnDirection::Up);
    }
    if parity == 0 {
        if up_left {
            return Some(SmokeSpawnDirection::UpLeft);
        }
        if up_right {
            return Some(SmokeSpawnDirection::UpRight);
        }
    } else {
        if up_right {
            return Some(SmokeSpawnDirection::UpRight);
        }
        if up_left {
            return Some(SmokeSpawnDirection::UpLeft);
        }
    }
    if parity == 0 {
        if left {
            return Some(SmokeSpawnDirection::Left);
        }
        if right {
            return Some(SmokeSpawnDirection::Right);
        }
    } else {
        if right {
            return Some(SmokeSpawnDirection::Right);
        }
        if left {
            return Some(SmokeSpawnDirection::Left);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{
        MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_OIL, MATERIAL_SAND,
        MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD,
    };

    #[test]
    fn wood_and_oil_share_the_same_generic_structure() {
        // Both return the exact same descriptor type (`CombustionDescriptor`)
        // from the same generic Material property — no per-Material
        // special casing in the rule. Structurally comparable fields:
        let wood = combustion_descriptor(MATERIAL_WOOD).expect("Wood combusts");
        let oil = combustion_descriptor(MATERIAL_OIL).expect("Oil combusts");
        let wood_gpu = combustion_table()[MATERIAL_WOOD as usize];
        let oil_gpu = combustion_table()[MATERIAL_OIL as usize];
        assert_eq!(wood_gpu.is_combustible, oil_gpu.is_combustible);
        assert!(wood.ignition_threshold.is_finite());
        assert!(oil.ignition_threshold.is_finite());
    }

    #[test]
    fn nonflammable_materials_have_no_combustion() {
        for id in [
            MATERIAL_EMPTY,
            MATERIAL_BOUNDARY_BLOCK,
            MATERIAL_STONE,
            MATERIAL_SAND,
            MATERIAL_WATER,
            MATERIAL_STEAM,
            MATERIAL_SMOKE,
            MATERIAL_ICE,
        ] {
            assert_eq!(
                combustion_descriptor(id),
                None,
                "material {id} must not combust"
            );
        }
        assert_eq!(combustion_descriptor(42), None);
    }

    #[test]
    fn oil_ignition_threshold() {
        // below ignition → off, at/above → ignite
        assert!(!combustion_step(MATERIAL_OIL, 74.0, 0).burning);
        assert!(combustion_step(MATERIAL_OIL, 75.0, 0).burning);
        assert!(combustion_step(MATERIAL_OIL, 100.0, 0).burning);
    }

    #[test]
    fn wood_ignition_threshold() {
        assert!(!combustion_step(MATERIAL_WOOD, 89.0, 0).burning);
        assert!(combustion_step(MATERIAL_WOOD, 90.0, 0).burning);
        assert!(combustion_step(MATERIAL_WOOD, 120.0, 0).burning);
    }

    #[test]
    fn burning_above_sustain_continues() {
        let result = combustion_step(MATERIAL_OIL, 50.0, FLAG_COMBUSTING);
        assert!(result.burning);
        assert!(result.flame_event);
    }

    #[test]
    fn burning_below_sustain_extinguishes() {
        let result = combustion_step(MATERIAL_OIL, 44.0, FLAG_COMBUSTING);
        assert!(!result.burning);
        assert!(!result.flame_event);
        // Extinguished matter keeps its (finite) temperature.
        assert!(result.temperature.is_finite());
    }

    #[test]
    fn burning_adds_heat() {
        let result = combustion_step(MATERIAL_OIL, 80.0, FLAG_COMBUSTING);
        assert_eq!(
            result.temperature, 85.0,
            "burning Oil adds heat_per_tick each tick"
        );
        let wood = combustion_step(MATERIAL_WOOD, 95.0, FLAG_COMBUSTING);
        assert_eq!(wood.temperature, 99.0, "burning Wood adds heat_per_tick");
    }

    #[test]
    fn ignition_tick_also_adds_heat() {
        let result = combustion_step(MATERIAL_OIL, 75.0, 0);
        assert!(result.burning);
        assert_eq!(result.temperature, 80.0);
    }

    #[test]
    fn nonflammable_hot_material_never_ignites() {
        for id in [MATERIAL_STONE, MATERIAL_SAND, MATERIAL_WATER, MATERIAL_ICE] {
            let result = combustion_step(id, 1000.0, 0);
            assert!(!result.burning, "material {id} must never ignite");
        }
    }

    #[test]
    fn no_oxygen_parameter_or_concept() {
        // The pure rule's signature has no Oxygen input: ignition depends
        // only on the thermal condition (REACTION_SPEC §11).
        let sealed_hot_wood = combustion_step(MATERIAL_WOOD, 100.0, 0);
        assert!(sealed_hot_wood.burning);
    }

    #[test]
    fn outputs_are_always_finite() {
        let nan = combustion_step(MATERIAL_OIL, f32::NAN, FLAG_COMBUSTING);
        assert!(nan.temperature.is_finite());
        let inf = combustion_step(MATERIAL_WOOD, f32::INFINITY, FLAG_COMBUSTING);
        assert!(inf.temperature.is_finite());
        // Pre-existing heat above the gameplay cap is never reduced (the
        // cap only bounds combustion growth), but must stay finite.
        let huge = combustion_step(MATERIAL_OIL, 1.0e30, FLAG_COMBUSTING);
        assert!(huge.temperature.is_finite());
        assert_eq!(huge.temperature, 1.0e30);
        let grown = combustion_step(MATERIAL_OIL, 900.0, FLAG_COMBUSTING);
        assert!(grown.temperature <= COMBUSTION_MAX_TEMPERATURE);
    }

    #[test]
    fn combustion_cap_never_reduces_hotter_cells() {
        let result = combustion_step(MATERIAL_WOOD, 2000.0, FLAG_COMBUSTING);
        assert_eq!(
            result.temperature, 2000.0,
            "cap must not cool a hotter cell"
        );
    }

    #[test]
    fn combustion_flags_preserve_unrelated_bits() {
        let unrelated = 1u32 << 8;
        let result = combustion_step(MATERIAL_OIL, 80.0, FLAG_COMBUSTING | unrelated);
        let next = combustion_flags_next(FLAG_COMBUSTING | unrelated, &result);
        assert_ne!(next & unrelated, 0, "unrelated bits must survive");
        assert_ne!(next & FLAG_COMBUSTING, 0);
        assert_ne!(next & FLAG_FLAME_EVENT, 0);

        let extinguished = combustion_step(MATERIAL_OIL, 30.0, FLAG_COMBUSTING | unrelated);
        let next = combustion_flags_next(FLAG_COMBUSTING | unrelated, &extinguished);
        assert_eq!(next & FLAG_COMBUSTING, 0);
        assert_eq!(next & FLAG_FLAME_EVENT, 0);
        assert_ne!(next & unrelated, 0);
    }

    #[test]
    fn smoke_stencil_orders_up_first() {
        assert_eq!(
            pick_smoke_spawn(true, true, true, true, true, 0),
            Some(SmokeSpawnDirection::Up)
        );
    }

    #[test]
    fn smoke_stencil_parity_orders_diagonals() {
        // parity 0: up-left first; parity 1: up-right first.
        assert_eq!(
            pick_smoke_spawn(false, true, true, false, false, 0),
            Some(SmokeSpawnDirection::UpLeft)
        );
        assert_eq!(
            pick_smoke_spawn(false, true, true, false, false, 1),
            Some(SmokeSpawnDirection::UpRight)
        );
    }

    #[test]
    fn smoke_stencil_falls_back_to_lateral_then_none() {
        assert_eq!(
            pick_smoke_spawn(false, false, false, true, false, 0),
            Some(SmokeSpawnDirection::Left)
        );
        assert_eq!(
            pick_smoke_spawn(false, false, false, false, true, 0),
            Some(SmokeSpawnDirection::Right)
        );
        assert_eq!(pick_smoke_spawn(false, false, false, false, false, 0), None);
    }

    #[test]
    fn combustion_table_maps_only_combustibles() {
        let table = combustion_table();
        assert_eq!(table[MATERIAL_WOOD as usize].is_combustible, 1);
        assert_eq!(table[MATERIAL_OIL as usize].is_combustible, 1);
        for id in [
            MATERIAL_EMPTY,
            MATERIAL_BOUNDARY_BLOCK,
            MATERIAL_STONE,
            MATERIAL_SAND,
            MATERIAL_WATER,
            MATERIAL_STEAM,
            MATERIAL_SMOKE,
            MATERIAL_ICE,
        ] {
            assert_eq!(table[id as usize].is_combustible, 0);
        }
        for unknown in [10usize, 15] {
            assert_eq!(table[unknown].is_combustible, 0);
        }
        // Sanity: Oil ignites before Wood in gameplay baseline.
        let oil = table[MATERIAL_OIL as usize];
        let wood = table[MATERIAL_WOOD as usize];
        assert!(oil.ignition_threshold < wood.ignition_threshold);
    }
}
