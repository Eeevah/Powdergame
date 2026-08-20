//! TE-2 unified passive thermal exchange — CPU reference rules.
//!
//! Temperature is a per-cell `f32` field (`SIMULATION_SPEC` §13). It is not
//! a Material property. Material only supplies cheap gameplay conductivity
//! and heat-capacity used by the local 4-neighbor transfer.
//!
//! Contracts:
//! - EMPTY has no Matter node; valid positive Air may provide an Environment node.
//! - Read Neighbors → Write Self. The caller writes only `self` next T.
//! - No ownership claim/resolve (this is a local field update).
//! - Gameplay temperatures use a Celsius-like scale with a 20 °C reference.
//! - NaN / Infinity collapse to the reference and finite values are bounded.
//! - Canonical face flux is equal and opposite; source-free energy-like totals
//!   are conserved within floating-point tolerance.

use crate::material::{registry_lookup, MATERIAL_EMPTY, MATERIAL_REGISTRY};
use crate::{air_temperature_celsius_like, AirState, AIR_HEAT_CAPACITY};

/// Simulation reference temperature on the Celsius-like gameplay scale.
pub const TEMPERATURE_REFERENCE_C: f32 = 20.0;
/// Compatibility name used by existing staging and evidence code.
pub const TEMPERATURE_REFERENCE: f32 = TEMPERATURE_REFERENCE_C;
pub const TEMPERATURE_MIN_C: f32 = -250.0;
pub const TEMPERATURE_MAX_C: f32 = 2_000.0;

/// Differences smaller than this are treated as equilibrium (no transfer).
pub const THERMAL_DEADBAND_C: f32 = 0.01;
pub const THERMAL_DEADBAND: f32 = THERMAL_DEADBAND_C;

/// Global transfer rate. Chosen so 4 high-k neighbors cannot overshoot
/// into runaway under explicit Euler.
pub const THERMAL_BASE_STEP: f32 = 0.12;
pub const THERMAL_RATE: f32 = THERMAL_BASE_STEP;

/// Absolute per-tick temperature change clamp.
pub const THERMAL_MAX_MIX_FRACTION: f32 = 0.25;
/// Retained only as an API compatibility constant for authored tools. The
/// production TE-2 exchange uses stability scaling, not a per-cell clamp.
pub const THERMAL_MAX_DELTA: f32 = 25.0;

/// Floor on heat capacity so a zero/tiny C cannot explode the update.
pub const THERMAL_MIN_CAPACITY: f32 = 0.25;

/// Gameplay thermal properties for one registered Matter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermalProperties {
    pub conductivity: f32,
    pub heat_capacity: f32,
}

/// Looks up cheap thermal properties. `EMPTY` and unknown ids have none.
pub fn thermal_properties(id: u32) -> Option<ThermalProperties> {
    registry_lookup(id).map(|m| ThermalProperties {
        conductivity: m.thermal_conductivity,
        heat_capacity: m.heat_capacity,
    })
}

/// Compact per-ID conductivity table for GPU upload (`0` = no conduction).
pub fn conductivity_table() -> [f32; 16] {
    let mut table = [0.0f32; 16];
    for m in MATERIAL_REGISTRY {
        table[m.id as usize] = m.thermal_conductivity;
    }
    table
}

/// Compact per-ID heat-capacity table for GPU upload.
pub fn heat_capacity_table() -> [f32; 16] {
    let mut table = [0.0f32; 16];
    for m in MATERIAL_REGISTRY {
        table[m.id as usize] = m.heat_capacity;
    }
    table
}

/// A 4-neighbor sample. `None` is Void / out of domain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermalNeighbor {
    pub material: u32,
    pub temperature: f32,
}

/// Collapses non-finite values to the reference temperature.
pub fn sanitize_temperature(t: f32) -> f32 {
    if t.is_finite() {
        t.clamp(TEMPERATURE_MIN_C, TEMPERATURE_MAX_C)
    } else {
        TEMPERATURE_REFERENCE
    }
}

/// The one thermal node exposed by a cell in the TE-2 model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThermalNode {
    Matter {
        temperature_c: f32,
        conductivity: f32,
        capacity: f32,
    },
    Air {
        temperature_c: f32,
        capacity: f32,
    },
}

impl ThermalNode {
    pub fn temperature_c(self) -> f32 {
        match self {
            Self::Matter { temperature_c, .. } | Self::Air { temperature_c, .. } => temperature_c,
        }
    }

    pub fn capacity(self) -> f32 {
        match self {
            Self::Matter { capacity, .. } | Self::Air { capacity, .. } => capacity,
        }
    }
}

/// Selects exactly one active thermal node. Occupied Matter owns its
/// temperature; EMPTY may expose valid positive Air; Vacuum exposes none.
pub fn thermal_node_for_cell(
    material: u32,
    matter_temperature_c: f32,
    air: AirState,
) -> Option<ThermalNode> {
    if material != MATERIAL_EMPTY {
        let props = thermal_properties(material)?;
        return Some(ThermalNode::Matter {
            temperature_c: sanitize_temperature(matter_temperature_c),
            conductivity: props.conductivity.max(0.0),
            capacity: props.heat_capacity.max(THERMAL_MIN_CAPACITY),
        });
    }
    let temperature_c = air_temperature_celsius_like(air)?;
    let capacity = air.mass * AIR_HEAT_CAPACITY;
    (air.mass > 0.0 && capacity.is_finite() && capacity > 0.0).then_some(ThermalNode::Air {
        temperature_c: sanitize_temperature(temperature_c),
        capacity,
    })
}

/// Conductance of a canonical thermal face.
pub fn thermal_face_conductance(a: ThermalNode, b: ThermalNode) -> f32 {
    use crate::{AIR_THERMAL_CONDUCTIVITY, MATTER_AIR_INTERFACE_CONDUCTANCE};
    match (a, b) {
        (
            ThermalNode::Matter {
                conductivity: ka, ..
            },
            ThermalNode::Matter {
                conductivity: kb, ..
            },
        ) => ka.min(kb).max(0.0),
        (ThermalNode::Air { .. }, ThermalNode::Air { .. }) => AIR_THERMAL_CONDUCTIVITY,
        (ThermalNode::Matter { conductivity, .. }, ThermalNode::Air { .. })
        | (ThermalNode::Air { .. }, ThermalNode::Matter { conductivity, .. }) => {
            conductivity.clamp(0.0, MATTER_AIR_INTERFACE_CONDUCTANCE)
        }
    }
}

/// Per-node explicit stability factor used by every adjacent canonical face.
pub fn thermal_stability_scale(node: ThermalNode, conductance_sum: f32) -> f32 {
    if !conductance_sum.is_finite() || conductance_sum <= 0.0 {
        return 0.0;
    }
    (THERMAL_MAX_MIX_FRACTION * node.capacity() / (THERMAL_BASE_STEP * conductance_sum))
        .clamp(0.0, 1.0)
}

/// Shared TE-2 physics/activity predicate.
pub fn thermal_work_exists(delta_c: f32) -> bool {
    delta_c.is_finite() && delta_c.abs() > THERMAL_DEADBAND_C
}

/// Signed energy-like flux from the low-index endpoint to the high-index
/// endpoint. Both endpoints must derive this from the same Current snapshot.
pub fn canonical_thermal_face_flux(
    low: ThermalNode,
    high: ThermalNode,
    lambda_low: f32,
    lambda_high: f32,
) -> f32 {
    let conductance = thermal_face_conductance(low, high);
    let delta = high.temperature_c() - low.temperature_c();
    // The deadband is a work/no-work gate, never a quantity subtracted from
    // a real temperature difference. This avoids an asymptotic active tail.
    let effective_delta = if thermal_work_exists(delta) {
        delta
    } else {
        0.0
    };
    let flux = THERMAL_BASE_STEP
        * lambda_low.clamp(0.0, 1.0).min(lambda_high.clamp(0.0, 1.0))
        * conductance
        * effective_delta;
    if flux.is_finite() {
        flux
    } else {
        0.0
    }
}

/// Applies the signed sum of incoming energy-like flux to one node.
pub fn passive_thermal_cell_step(node: ThermalNode, incoming_energy: f32) -> f32 {
    sanitize_temperature(node.temperature_c() + incoming_energy / node.capacity())
}

/// Sums energy-like values used by conservation fixtures.
pub fn energy_like_total(nodes: &[ThermalNode]) -> f64 {
    nodes
        .iter()
        .map(|node| node.temperature_c() as f64 * node.capacity() as f64)
        .sum()
}

/// One explicit-Euler self update from the 4 orthogonal neighbors.
///
/// `neighbors` order is not significant. EMPTY / unknown / Void contribute
/// nothing. The result is always finite.
pub fn thermal_step(
    self_material: u32,
    self_temperature: f32,
    neighbors: [Option<ThermalNeighbor>; 4],
) -> f32 {
    let Some(props) = thermal_properties(self_material) else {
        return TEMPERATURE_REFERENCE;
    };
    let self_t = sanitize_temperature(self_temperature);
    let capacity = props.heat_capacity.max(THERMAL_MIN_CAPACITY);
    let k_self = props.conductivity.max(0.0);

    let mut acc = 0.0f32;
    for neighbor in neighbors.into_iter().flatten() {
        if neighbor.material == MATERIAL_EMPTY {
            continue;
        }
        let Some(n_props) = thermal_properties(neighbor.material) else {
            continue;
        };
        let n_t = sanitize_temperature(neighbor.temperature);
        let delta = n_t - self_t;
        if !thermal_work_exists(delta) {
            continue;
        }
        let k_eff = k_self.min(n_props.conductivity.max(0.0));
        acc += k_eff * delta;
    }

    let change = (THERMAL_RATE * acc / capacity).clamp(-THERMAL_MAX_DELTA, THERMAL_MAX_DELTA);
    sanitize_temperature(self_t + change)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{
        MATERIAL_EMPTY, MATERIAL_OIL, MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STEAM,
        MATERIAL_STONE, MATERIAL_WATER,
    };

    fn only_right(material: u32, temperature: f32) -> [Option<ThermalNeighbor>; 4] {
        [
            None,
            None,
            Some(ThermalNeighbor {
                material,
                temperature,
            }),
            None,
        ]
    }

    #[test]
    fn hot_neighbor_heats_cold_self() {
        let next = thermal_step(MATERIAL_STONE, 0.0, only_right(MATERIAL_STONE, 10.0));
        assert!(next > 0.0, "cold stone must heat; got {next}");
        assert!(next < 10.0, "must not jump past the neighbor; got {next}");
    }

    #[test]
    fn cold_neighbor_cools_hot_self() {
        let next = thermal_step(MATERIAL_STONE, 10.0, only_right(MATERIAL_STONE, 0.0));
        assert!(next < 10.0, "hot stone must cool; got {next}");
        assert!(next > 0.0, "must not jump past the neighbor; got {next}");
    }

    #[test]
    fn equal_temperature_is_stable() {
        let next = thermal_step(MATERIAL_STONE, 4.0, only_right(MATERIAL_STONE, 4.0));
        assert_eq!(next, 4.0);
    }

    #[test]
    fn empty_neighbor_does_not_conduct() {
        let next = thermal_step(MATERIAL_STONE, 3.0, only_right(MATERIAL_EMPTY, 100.0));
        assert_eq!(next, 3.0, "EMPTY is not a thermal medium");
    }

    #[test]
    fn conductivity_difference_changes_transfer() {
        // Water and Oil share heat capacity; Water conducts more.
        let water = thermal_step(MATERIAL_WATER, 0.0, only_right(MATERIAL_WATER, 10.0));
        let oil = thermal_step(MATERIAL_OIL, 0.0, only_right(MATERIAL_OIL, 10.0));
        assert!(
            water > oil,
            "higher conductivity must transfer more (water {water} vs oil {oil})"
        );
        assert!(water > 0.0 && oil > 0.0);
    }

    #[test]
    fn output_is_always_finite() {
        let huge = thermal_step(MATERIAL_STONE, 1.0e10, only_right(MATERIAL_STONE, -1.0e10));
        assert!(
            huge.is_finite(),
            "clamped step must stay finite; got {huge}"
        );

        let nan = thermal_step(
            MATERIAL_STONE,
            f32::NAN,
            only_right(MATERIAL_STONE, f32::INFINITY),
        );
        assert!(nan.is_finite());
        assert_eq!(nan, TEMPERATURE_REFERENCE);

        assert_eq!(sanitize_temperature(f32::NAN), TEMPERATURE_REFERENCE);
        assert_eq!(sanitize_temperature(f32::INFINITY), TEMPERATURE_REFERENCE);
        assert_eq!(
            sanitize_temperature(f32::NEG_INFINITY),
            TEMPERATURE_REFERENCE
        );
    }

    #[test]
    fn empty_self_has_no_thermal_state() {
        let next = thermal_step(MATERIAL_EMPTY, 8.0, only_right(MATERIAL_STONE, 20.0));
        assert_eq!(next, TEMPERATURE_REFERENCE);
    }

    #[test]
    fn tables_cover_registered_matter() {
        let k = conductivity_table();
        let c = heat_capacity_table();
        assert_eq!(k[MATERIAL_EMPTY as usize], 0.0);
        assert!(k[MATERIAL_WATER as usize] > k[MATERIAL_OIL as usize]);
        assert!(c[MATERIAL_WATER as usize] > 0.0);
        assert!(c[MATERIAL_SAND as usize] > 0.0);
        assert!(c[MATERIAL_STEAM as usize] > 0.0);
        assert!(c[MATERIAL_SMOKE as usize] > 0.0);
        assert!(k[MATERIAL_STONE as usize] > 0.0);
    }

    #[derive(Clone, Copy)]
    enum PairKind {
        MatterMatter,
        AirAir,
        MatterAir,
    }

    fn node(kind: PairKind, left: bool, temperature_c: f32) -> ThermalNode {
        match (kind, left) {
            (PairKind::MatterMatter, _) | (PairKind::MatterAir, true) => ThermalNode::Matter {
                temperature_c,
                conductivity: 0.25,
                capacity: 1.0,
            },
            (PairKind::AirAir, _) | (PairKind::MatterAir, false) => ThermalNode::Air {
                temperature_c,
                capacity: 1.0,
            },
        }
    }

    #[test]
    fn small_delta_thermal_convergence_is_monotone_and_scale_independent() {
        for kind in [
            PairKind::MatterMatter,
            PairKind::AirAir,
            PairKind::MatterAir,
        ] {
            for baseline in [20.0f32, 500.0] {
                for initial_delta in [1.0f32, 0.1, 0.02, 0.011, 0.009] {
                    let mut cold = baseline;
                    let mut hot = baseline + initial_delta;
                    let initial_total =
                        energy_like_total(&[node(kind, true, cold), node(kind, false, hot)]);
                    let should_work = initial_delta > THERMAL_DEADBAND_C;
                    let mut converged = !should_work;
                    for tick in 0..4096 {
                        let low = node(kind, true, cold);
                        let high = node(kind, false, hot);
                        let conductance = thermal_face_conductance(low, high);
                        let flux = canonical_thermal_face_flux(
                            low,
                            high,
                            thermal_stability_scale(low, conductance),
                            thermal_stability_scale(high, conductance),
                        );
                        if tick == 0 {
                            assert_eq!(flux != 0.0, should_work);
                            assert_eq!(thermal_work_exists(hot - cold), should_work);
                        }
                        let previous_cold = cold;
                        let previous_hot = hot;
                        cold = passive_thermal_cell_step(low, flux);
                        hot = passive_thermal_cell_step(high, -flux);
                        assert!(cold >= previous_cold && hot <= previous_hot);
                        assert!(cold <= hot, "hot/cold ordering reversed");
                        assert!(cold >= baseline && hot <= baseline + initial_delta);
                        let total =
                            energy_like_total(&[node(kind, true, cold), node(kind, false, hot)]);
                        assert!((total - initial_total).abs() <= 2.0e-4);
                        if !thermal_work_exists(hot - cold) {
                            converged = true;
                            break;
                        }
                    }
                    assert!(
                        converged,
                        "delta {initial_delta} at {baseline} did not converge"
                    );
                }
            }
        }
    }
}
