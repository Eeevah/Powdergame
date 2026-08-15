//! G4-B — Phase transition: temperature-based 1:1 self transitions.
//!
//! Ice ↔ Water ↔ Steam is a **Self Transition** (`REACTION_SPEC` §3): the
//! decision depends only on the cell's own Material + Temperature, so one
//! invocation writes only `material_next[self]` — no Claim/Resolve, no
//! atomics, no neighbor writes. Multi-cell expansion / yield and Pressure
//! are G5 (out of scope here).
//!
//! Contracts:
//! - Transition rules are **Material data**, pre-ordered First-Match
//!   (`REACTION_SPEC` §6). The GPU consumes a compact compiled table, not
//!   material-name branches in the shader.
//! - Temperature is the G4-A relative gameplay scalar, not Celsius.
//! - Hysteresis bands (−20 ↔ −10 and 40 ↔ 60) prevent tick-to-tick
//!   ping-pong near a threshold.
//! - EMPTY has no phase rule (not a registered Matter); Void has no cell.
//! - 1:1 transform: temperature is preserved (latent heat is out of scope).
//! - No global pair/rule scan; each Material owns only its own small set.

use crate::material::{registry_lookup, MATERIAL_REGISTRY};
use crate::thermal::sanitize_temperature;

/// Reference thresholds (relative gameplay scalar, not physical units).
/// Hysteresis: freeze at −20, melt at −10; condense at 40, boil at 60.
pub const WATER_FREEZE_BELOW: f32 = -20.0;
pub const ICE_MELT_ABOVE: f32 = -10.0;
pub const STEAM_CONDENSE_BELOW: f32 = 40.0;
pub const WATER_BOIL_ABOVE: f32 = 60.0;

/// Sentinel for "no transition target" in the compact GPU table.
/// Distinct from `EMPTY == 0` so "no rule" can never be confused with
/// "becomes EMPTY".
pub const NO_PHASE_TARGET: u32 = u32::MAX;

/// Temperature comparison direction for a phase rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureCondition {
    /// Transition fires when temperature is strictly below the threshold.
    Below,
    /// Transition fires when temperature is strictly above the threshold.
    Above,
}

/// One ordered phase rule owned by a Material.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseTransition {
    pub condition: TemperatureCondition,
    pub threshold: f32,
    pub target_material: u32,
}

/// Compact per-Material descriptor for GPU upload (one per material id).
///
/// Compiled from the Material's ordered rule set: the first `Below` rule
/// fills `below_*`, the first `Above` rule fills `above_*`. The GPU pass
/// checks `below` first, then `above` — which matches the Water rule order
/// (freeze before boil). Materials with no transitions keep the sentinel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseGpuDescriptor {
    pub below_target: u32,
    pub above_target: u32,
    pub below_threshold: f32,
    pub above_threshold: f32,
}

/// Pure reference: selects the phase target for `material_id` at
/// `temperature`, or `None` when the Material has no matching rule.
///
/// This is a unit/reference helper — the production full-world path is the
/// GPU phase pass, never a CPU world loop.
pub fn select_phase_transition(material_id: u32, temperature: f32) -> Option<u32> {
    let rules = registry_lookup(material_id)?.phase_transitions;
    let t = sanitize_temperature(temperature);
    for rule in rules {
        let hit = match rule.condition {
            TemperatureCondition::Below => t < rule.threshold,
            TemperatureCondition::Above => t > rule.threshold,
        };
        if hit {
            return Some(rule.target_material);
        }
    }
    None
}

/// Returns `true` if the Material owns at least one phase rule
/// (never true for EMPTY / unknown ids).
pub fn is_phase_candidate(material_id: u32) -> bool {
    registry_lookup(material_id)
        .map(|m| !m.phase_transitions.is_empty())
        .unwrap_or(false)
}

/// Compiles the GPU phase descriptor table (16 material slots).
///
/// This is a Material property upload — there is no per-cell phase buffer.
pub fn phase_descriptor_table() -> [PhaseGpuDescriptor; 16] {
    let none = PhaseGpuDescriptor {
        below_target: NO_PHASE_TARGET,
        above_target: NO_PHASE_TARGET,
        below_threshold: 0.0,
        above_threshold: 0.0,
    };
    let mut table = [none; 16];
    for m in MATERIAL_REGISTRY {
        let mut desc = none;
        let mut below_seen = false;
        let mut above_seen = false;
        for rule in m.phase_transitions {
            match rule.condition {
                TemperatureCondition::Below if !below_seen => {
                    desc.below_target = rule.target_material;
                    desc.below_threshold = rule.threshold;
                    below_seen = true;
                }
                TemperatureCondition::Above if !above_seen => {
                    desc.above_target = rule.target_material;
                    desc.above_threshold = rule.threshold;
                    above_seen = true;
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
    use crate::material::{
        MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_OIL, MATERIAL_SAND,
        MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER,
    };
    use crate::thermal::{heat_capacity_table, thermal_properties};

    #[test]
    fn water_freezes_below_threshold() {
        assert_eq!(
            select_phase_transition(MATERIAL_WATER, -30.0),
            Some(MATERIAL_ICE)
        );
        assert_eq!(
            select_phase_transition(MATERIAL_WATER, -25.0),
            Some(MATERIAL_ICE)
        );
    }

    #[test]
    fn water_boils_above_threshold() {
        assert_eq!(
            select_phase_transition(MATERIAL_WATER, 70.0),
            Some(MATERIAL_STEAM)
        );
        assert_eq!(
            select_phase_transition(MATERIAL_WATER, 65.0),
            Some(MATERIAL_STEAM)
        );
    }

    #[test]
    fn ice_melts_above_threshold() {
        assert_eq!(
            select_phase_transition(MATERIAL_ICE, 0.0),
            Some(MATERIAL_WATER)
        );
        assert_eq!(
            select_phase_transition(MATERIAL_ICE, -5.0),
            Some(MATERIAL_WATER)
        );
    }

    #[test]
    fn steam_condenses_below_threshold() {
        assert_eq!(
            select_phase_transition(MATERIAL_STEAM, 30.0),
            Some(MATERIAL_WATER)
        );
        assert_eq!(
            select_phase_transition(MATERIAL_STEAM, 0.0),
            Some(MATERIAL_WATER)
        );
    }

    #[test]
    fn neutral_temperatures_are_stable() {
        assert_eq!(select_phase_transition(MATERIAL_WATER, 0.0), None);
        assert_eq!(select_phase_transition(MATERIAL_ICE, -30.0), None);
        assert_eq!(select_phase_transition(MATERIAL_STEAM, 80.0), None);
    }

    #[test]
    fn hysteresis_bands_prevent_ping_pong() {
        // Freeze at -20, melt at -10: -15 is inside the band.
        assert_eq!(select_phase_transition(MATERIAL_WATER, -15.0), None);
        assert_eq!(select_phase_transition(MATERIAL_ICE, -15.0), None);
        // Condense at 40, boil at 60: +50 is inside the band.
        assert_eq!(select_phase_transition(MATERIAL_WATER, 50.0), None);
        assert_eq!(select_phase_transition(MATERIAL_STEAM, 50.0), None);
    }

    #[test]
    fn non_phase_materials_never_transition() {
        for id in [
            MATERIAL_EMPTY,
            MATERIAL_BOUNDARY_BLOCK,
            MATERIAL_STONE,
            MATERIAL_SAND,
            MATERIAL_OIL,
            MATERIAL_SMOKE,
        ] {
            assert!(!is_phase_candidate(id), "material {id} has no phase rules");
            assert_eq!(select_phase_transition(id, -1000.0), None);
            assert_eq!(select_phase_transition(id, 1000.0), None);
        }
    }

    #[test]
    fn unknown_ids_never_transition() {
        for unknown in [9u32, 42, u32::MAX] {
            assert!(!is_phase_candidate(unknown));
            assert_eq!(select_phase_transition(unknown, -50.0), None);
        }
    }

    #[test]
    fn targets_are_registered_matter() {
        let targets = [
            select_phase_transition(MATERIAL_WATER, -30.0),
            select_phase_transition(MATERIAL_WATER, 70.0),
            select_phase_transition(MATERIAL_ICE, 0.0),
            select_phase_transition(MATERIAL_STEAM, 30.0),
        ];
        for target in targets {
            let id = target.expect("each transition must have a target");
            assert!(
                registry_lookup(id).is_some(),
                "phase target {id} must be a registered Matter"
            );
            assert_ne!(id, MATERIAL_EMPTY, "phase targets are never EMPTY");
        }
    }

    #[test]
    fn phase_candidates_are_only_water_ice_steam() {
        assert!(is_phase_candidate(MATERIAL_WATER));
        assert!(is_phase_candidate(MATERIAL_ICE));
        assert!(is_phase_candidate(MATERIAL_STEAM));
        assert!(!is_phase_candidate(MATERIAL_EMPTY));
    }

    #[test]
    fn gpu_descriptor_table_matches_reference() {
        let table = phase_descriptor_table();

        // Water: below → Ice @ -20, above → Steam @ 60.
        assert_eq!(table[MATERIAL_WATER as usize].below_target, MATERIAL_ICE);
        assert_eq!(
            table[MATERIAL_WATER as usize].below_threshold,
            WATER_FREEZE_BELOW
        );
        assert_eq!(table[MATERIAL_WATER as usize].above_target, MATERIAL_STEAM);
        assert_eq!(
            table[MATERIAL_WATER as usize].above_threshold,
            WATER_BOIL_ABOVE
        );

        // Ice: below none, above → Water @ -10.
        assert_eq!(table[MATERIAL_ICE as usize].below_target, NO_PHASE_TARGET);
        assert_eq!(table[MATERIAL_ICE as usize].above_target, MATERIAL_WATER);
        assert_eq!(table[MATERIAL_ICE as usize].above_threshold, ICE_MELT_ABOVE);

        // Steam: below → Water @ 40, above none.
        assert_eq!(table[MATERIAL_STEAM as usize].below_target, MATERIAL_WATER);
        assert_eq!(
            table[MATERIAL_STEAM as usize].below_threshold,
            STEAM_CONDENSE_BELOW
        );
        assert_eq!(table[MATERIAL_STEAM as usize].above_target, NO_PHASE_TARGET);

        // EMPTY and non-phase materials keep the safe sentinel.
        assert_eq!(table[MATERIAL_EMPTY as usize].below_target, NO_PHASE_TARGET);
        assert_eq!(table[MATERIAL_EMPTY as usize].above_target, NO_PHASE_TARGET);
        for id in [
            MATERIAL_BOUNDARY_BLOCK,
            MATERIAL_STONE,
            MATERIAL_SAND,
            MATERIAL_OIL,
            MATERIAL_SMOKE,
        ] {
            assert_eq!(table[id as usize].below_target, NO_PHASE_TARGET);
            assert_eq!(table[id as usize].above_target, NO_PHASE_TARGET);
        }
        for unknown in [9usize, 15] {
            assert_eq!(table[unknown].below_target, NO_PHASE_TARGET);
            assert_eq!(table[unknown].above_target, NO_PHASE_TARGET);
        }
    }

    #[test]
    fn ice_thermal_properties_are_sane() {
        let ice = thermal_properties(MATERIAL_ICE).unwrap();
        assert!(ice.conductivity > 0.0 && ice.conductivity.is_finite());
        assert!(ice.heat_capacity > 0.0 && ice.heat_capacity.is_finite());
        assert!(heat_capacity_table()[MATERIAL_ICE as usize] > 0.0);
    }
}
