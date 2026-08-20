//! G4-B/G5-B — Phase transition with data-driven expansion metadata.
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
//! - Temperature is the TE-2 Celsius-like gameplay scalar.
//! - Hysteresis bands (−2 ↔ 2 and 95 ↔ 100) prevent tick-to-tick
//!   ping-pong near a threshold.
//! - EMPTY has no phase rule (not a registered Matter); Void has no cell.
//! - 1:1 transform: temperature is preserved (latent heat is out of scope).
//! - No global pair/rule scan; each Material owns only its own small set.

use crate::material::{registry_lookup, MATERIAL_REGISTRY};
use crate::thermal::sanitize_temperature;

/// Reference thresholds (relative gameplay scalar, not physical units).
/// Hysteresis: freeze at −20, melt at −10; condense at 40, boil at 60.
pub const WATER_FREEZE_BELOW: f32 = -2.0;
pub const ICE_MELT_ABOVE: f32 = 2.0;
pub const STEAM_CONDENSE_BELOW: f32 = 95.0;
pub const WATER_BOIL_ABOVE: f32 = 100.0;

/// G5-B baseline: ordinary phase rules are identity-yield (1 cell in → 1 out).
pub const PHASE_IDENTITY_MATTER_YIELD: u32 = 1;
/// G5-B minimum sufficient expansion: boiling Water requests one extra Steam cell.
pub const WATER_BOIL_MATTER_YIELD: u32 = 2;
/// Pressure impulse created when the extra boiling yield cannot acquire space.
/// Gameplay scalar, not SI pressure.
pub const WATER_BOIL_BLOCKED_PRESSURE: f32 = 100.0;
/// Current G5-B ownership path supports at most one additional Matter cell.
pub const MAX_PHASE_MATTER_YIELD: u32 = 2;

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
    /// Total Matter cells requested by this transition, including self.
    pub matter_yield: u32,
    /// Pressure added at the source when requested extra yield is unresolved.
    pub blocked_pressure: f32,
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
    pub below_yield: u32,
    pub above_yield: u32,
    pub below_threshold: f32,
    pub above_threshold: f32,
    pub below_blocked_pressure: f32,
    pub above_blocked_pressure: f32,
}

/// Full selected phase effect used by G5-B expansion/confinement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseEffect {
    pub target_material: u32,
    pub matter_yield: u32,
    pub blocked_pressure: f32,
}

/// Selects the first matching Material-owned phase effect.
pub fn select_phase_effect(material_id: u32, temperature: f32) -> Option<PhaseEffect> {
    let rules = registry_lookup(material_id)?.phase_transitions;
    let t = sanitize_temperature(temperature);
    for rule in rules {
        let hit = match rule.condition {
            TemperatureCondition::Below => t < rule.threshold,
            TemperatureCondition::Above => t > rule.threshold,
        };
        if hit {
            return Some(PhaseEffect {
                target_material: rule.target_material,
                matter_yield: rule.matter_yield,
                blocked_pressure: rule.blocked_pressure,
            });
        }
    }
    None
}

/// Pure reference: selects the phase target for `material_id` at
/// `temperature`, or `None` when the Material has no matching rule.
///
/// This is a unit/reference helper — the production full-world path is the
/// GPU phase pass, never a CPU world loop.
pub fn select_phase_transition(material_id: u32, temperature: f32) -> Option<u32> {
    select_phase_effect(material_id, temperature).map(|effect| effect.target_material)
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
        below_yield: PHASE_IDENTITY_MATTER_YIELD,
        above_yield: PHASE_IDENTITY_MATTER_YIELD,
        below_threshold: 0.0,
        above_threshold: 0.0,
        below_blocked_pressure: 0.0,
        above_blocked_pressure: 0.0,
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
                    desc.below_yield = rule.matter_yield;
                    desc.below_threshold = rule.threshold;
                    desc.below_blocked_pressure = rule.blocked_pressure;
                    below_seen = true;
                }
                TemperatureCondition::Above if !above_seen => {
                    desc.above_target = rule.target_material;
                    desc.above_yield = rule.matter_yield;
                    desc.above_threshold = rule.threshold;
                    desc.above_blocked_pressure = rule.blocked_pressure;
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
        MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD,
    };
    use crate::thermal::{heat_capacity_table, thermal_properties};

    #[test]
    fn water_freezes_below_threshold() {
        assert_eq!(
            select_phase_transition(MATERIAL_WATER, -3.0),
            Some(MATERIAL_ICE)
        );
        assert_eq!(
            select_phase_transition(MATERIAL_WATER, -10.0),
            Some(MATERIAL_ICE)
        );
    }

    #[test]
    fn water_boils_above_threshold() {
        assert_eq!(
            select_phase_transition(MATERIAL_WATER, 101.0),
            Some(MATERIAL_STEAM)
        );
        assert_eq!(
            select_phase_transition(MATERIAL_WATER, 120.0),
            Some(MATERIAL_STEAM)
        );
    }

    #[test]
    fn ice_melts_above_threshold() {
        assert_eq!(
            select_phase_transition(MATERIAL_ICE, 3.0),
            Some(MATERIAL_WATER)
        );
        assert_eq!(
            select_phase_transition(MATERIAL_ICE, 20.0),
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
        assert_eq!(select_phase_transition(MATERIAL_WATER, 20.0), None);
        assert_eq!(select_phase_transition(MATERIAL_ICE, 0.0), None);
        assert_eq!(select_phase_transition(MATERIAL_STEAM, 100.0), None);
    }

    #[test]
    fn hysteresis_bands_prevent_ping_pong() {
        // Freeze at -2, melt at 2: 0 is inside the band.
        assert_eq!(select_phase_transition(MATERIAL_WATER, 0.0), None);
        assert_eq!(select_phase_transition(MATERIAL_ICE, 0.0), None);
        // Condense at 95, boil at 100: 97 is inside the band.
        assert_eq!(select_phase_transition(MATERIAL_WATER, 97.0), None);
        assert_eq!(select_phase_transition(MATERIAL_STEAM, 97.0), None);
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
            MATERIAL_WOOD,
        ] {
            assert!(!is_phase_candidate(id), "material {id} has no phase rules");
            assert_eq!(select_phase_transition(id, -1000.0), None);
            assert_eq!(select_phase_transition(id, 1000.0), None);
        }
    }

    #[test]
    fn unknown_ids_never_transition() {
        for unknown in [10u32, 42, u32::MAX] {
            assert!(!is_phase_candidate(unknown));
            assert_eq!(select_phase_transition(unknown, -50.0), None);
        }
    }

    #[test]
    fn targets_are_registered_matter() {
        let targets = [
            select_phase_transition(MATERIAL_WATER, -3.0),
            select_phase_transition(MATERIAL_WATER, 101.0),
            select_phase_transition(MATERIAL_ICE, 3.0),
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
            MATERIAL_WOOD,
        ] {
            assert_eq!(table[id as usize].below_target, NO_PHASE_TARGET);
            assert_eq!(table[id as usize].above_target, NO_PHASE_TARGET);
        }
        for unknown in [10usize, 15] {
            assert_eq!(table[unknown].below_target, NO_PHASE_TARGET);
            assert_eq!(table[unknown].above_target, NO_PHASE_TARGET);
        }
    }

    #[test]
    fn boiling_effect_requests_expansion_and_confinement_pressure() {
        let effect = select_phase_effect(MATERIAL_WATER, 101.0).unwrap();
        assert_eq!(effect.target_material, MATERIAL_STEAM);
        assert_eq!(effect.matter_yield, WATER_BOIL_MATTER_YIELD);
        assert_eq!(effect.blocked_pressure, WATER_BOIL_BLOCKED_PRESSURE);
    }

    #[test]
    fn non_expanding_phase_rules_keep_identity_yield() {
        for (material, t) in [
            (MATERIAL_WATER, -3.0),
            (MATERIAL_ICE, 3.0),
            (MATERIAL_STEAM, 30.0),
        ] {
            let effect = select_phase_effect(material, t).unwrap();
            assert_eq!(effect.matter_yield, PHASE_IDENTITY_MATTER_YIELD);
            assert_eq!(effect.blocked_pressure, 0.0);
        }
    }

    #[test]
    fn phase_descriptor_carries_g5b_metadata() {
        let table = phase_descriptor_table();
        let water = table[MATERIAL_WATER as usize];
        assert_eq!(water.above_yield, WATER_BOIL_MATTER_YIELD);
        assert_eq!(water.above_blocked_pressure, WATER_BOIL_BLOCKED_PRESSURE);
        assert_eq!(water.below_yield, PHASE_IDENTITY_MATTER_YIELD);
        assert_eq!(water.below_blocked_pressure, 0.0);
    }

    #[test]
    fn registered_phase_yields_fit_g5b_single_extra_cell_path() {
        for material in crate::material::MATERIAL_REGISTRY {
            for rule in material.phase_transitions {
                assert!(rule.matter_yield >= 1);
                assert!(rule.matter_yield <= MAX_PHASE_MATTER_YIELD);
                assert!(rule.blocked_pressure.is_finite());
                assert!(rule.blocked_pressure >= 0.0);
                if rule.matter_yield == PHASE_IDENTITY_MATTER_YIELD {
                    assert_eq!(rule.blocked_pressure, 0.0);
                }
            }
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
