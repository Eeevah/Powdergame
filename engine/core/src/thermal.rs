//! G4-A thermal baseline — CPU reference rule.
//!
//! Temperature is a per-cell `f32` field (`SIMULATION_SPEC` §13). It is not
//! a Material property. Material only supplies cheap gameplay conductivity
//! and heat-capacity used by the local 4-neighbor transfer.
//!
//! Contracts:
//! - EMPTY is not a hidden thermal medium: no heat flows through EMPTY/Void.
//! - Read Neighbors → Write Self. The caller writes only `self` next T.
//! - No ownership claim/resolve (this is a local field update).
//! - `0.0` is the simulation reference temperature (not Celsius).
//! - NaN / Infinity collapse to the reference. Per-tick delta is clamped.
//! - Exact global energy conservation is NOT required.

use crate::material::{registry_lookup, MATERIAL_EMPTY, MATERIAL_REGISTRY};

/// Simulation reference temperature. The initial world is filled with this
/// value; it is a relative hot/cold scalar, not a physical unit.
pub const TEMPERATURE_REFERENCE: f32 = 0.0;

/// Differences smaller than this are treated as equilibrium (no transfer).
pub const THERMAL_DEADBAND: f32 = 1.0e-4;

/// Global transfer rate. Chosen so 4 high-k neighbors cannot overshoot
/// into runaway under explicit Euler.
pub const THERMAL_RATE: f32 = 0.12;

/// Absolute per-tick temperature change clamp.
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
        t
    } else {
        TEMPERATURE_REFERENCE
    }
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
        if delta.abs() < THERMAL_DEADBAND {
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
}
