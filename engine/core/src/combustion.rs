//! G4-C — Combustion: temperature-based ignition / sustain / heat / Smoke
//! + finite fuel lifecycle.
//!
//! Wood and Oil share ONE generic combustion grammar (`REACTION_SPEC` §11):
//! a Material-owned `CombustionDescriptor` decides
//!
//! ```text
//! unlit + T >= ignition    → ignite (COMBUSTING + FLAME_EVENT)
//! burning + T >= sustain   → keep burning, add heat_per_tick, emit FLAME_EVENT
//! burning + T  < sustain   → extinguish (COMBUSTING/FLAME_EVENT clear)
//! burning fuel progress    → +1 per ACTIVE burning tick; progress survives
//!                            extinguish and continues on reignition
//! progress >= burn_duration → fuel consumed → cell becomes EMPTY
//!                            (material/T/flags self-reset)
//! non-combustible          → never ignites (combustion bits cleared)
//! ```
//!
//! Fuel semantics (`MATERIAL_SPEC` §4 Matter-owned state):
//! - `FUEL_PROGRESS` = accumulated active burn ticks (consumed fuel amount).
//!   It is stored in the per-cell `flags` bits 8..23 (u16) — Matter-owned,
//!   transported on movement edges like `temperature`.
//! - Burning is the ONLY source of progress; extinguish preserves it and
//!   reignition continues from it (extinguish→reignite never restores fuel).
//! - Identity replacement, EMPTY and Void all reset it.
//!
//! Contracts:
//! - Combustion is a **Material property** (descriptor), never per-cell
//!   state. No fuel mass / Ash / burn-age counter beyond the progress bits
//!   (no `No Universal Future State` violation: progress is the consumed
//!   fuel amount, a single u16).
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
use crate::TEMPERATURE_REFERENCE;

/// Persists across ticks: this Matter is actively combusting.
pub const FLAG_COMBUSTING: u32 = 1 << 0;
/// Ephemeral per-tick presentation signal: flame is visible this tick
/// (set on the ignition tick and on every active-combustion tick).
pub const FLAG_FLAME_EVENT: u32 = 1 << 1;

/// Fuel progress bit range (bits 4..15, 12 bits, u12 = 0..4095). Stored in the Matter-owned
/// `flags` field and transported on movement edges like temperature.
pub const FLAG_FUEL_PROGRESS_SHIFT: u32 = 4;
/// Mask of the fuel-progress bits (bits 4..15 inclusive).
pub const FLAG_FUEL_PROGRESS_MASK: u32 = 0x0FFF << FLAG_FUEL_PROGRESS_SHIFT;

/// Gameplay cap on combustion heat (finite, not a physical unit).
pub const COMBUSTION_MAX_TEMPERATURE: f32 = 1200.0;

/// Oil baseline tuning (relative gameplay scalar, not physical units).
pub const COMBUSTION_OIL_IGNITION: f32 = 200.0;
pub const COMBUSTION_OIL_SUSTAIN: f32 = 150.0;
pub const COMBUSTION_OIL_HEAT_PER_TICK: f32 = 6.0;
/// Oil fuel: 600 active burn ticks ≈ 10 s at 60 TPS (gameplay baseline).
pub const COMBUSTION_OIL_BURN_DURATION: u32 = 600;

/// Wood baseline tuning (relative gameplay scalar, not physical units).
pub const COMBUSTION_WOOD_IGNITION: f32 = 300.0;
pub const COMBUSTION_WOOD_SUSTAIN: f32 = 250.0;
pub const COMBUSTION_WOOD_HEAT_PER_TICK: f32 = 4.0;
/// Wood fuel: 900 active burn ticks ≈ 15 s at 60 TPS (gameplay baseline).
pub const COMBUSTION_WOOD_BURN_DURATION: u32 = 900;

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
    /// Fuel life in active burning ticks. When `FUEL_PROGRESS` reaches this
    /// value the Matter is consumed and the cell becomes `EMPTY`.
    pub burn_duration_ticks: u32,
}

/// Compact per-Material descriptor for GPU upload (20 bytes each).
///
/// `is_combustible == 0` is the safe sentinel — a non-combustible Matter
/// can never read thresholds as if it were burning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombustionGpuDescriptor {
    pub is_combustible: u32,
    pub ignition_threshold: f32,
    pub sustain_threshold: f32,
    pub heat_per_tick: f32,
    pub burn_duration_ticks: u32,
}

/// Returns the combustion descriptor of a registered Matter.
///
/// `None` for `EMPTY`, unknown ids and every non-combustible Matter.
pub fn combustion_descriptor(id: u32) -> Option<&'static CombustionDescriptor> {
    registry_lookup(id).and_then(|m| m.combustion.as_ref())
}

/// Extracts the accumulated fuel progress (u16) from a flags word.
pub fn fuel_progress(flags: u32) -> u32 {
    (flags & FLAG_FUEL_PROGRESS_MASK) >> FLAG_FUEL_PROGRESS_SHIFT
}

/// Replaces the fuel-progress field in a flags word, preserving every other
/// bit.
pub fn with_fuel_progress(flags: u32, progress: u32) -> u32 {
    (flags & !FLAG_FUEL_PROGRESS_MASK) | ((progress & 0x0FFF) << FLAG_FUEL_PROGRESS_SHIFT)
}

/// Combustion-owned flag bits (the two bool state bits + the fuel-progress
/// field). The combustion pass only ever sets/clears these bits; all other
/// flags bits belong to future subsystems.
pub fn combustion_flag_mask() -> u32 {
    FLAG_COMBUSTING | FLAG_FLAME_EVENT | FLAG_FUEL_PROGRESS_MASK
}

/// Compiles the GPU combustion table (16 material slots × 20 bytes).
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
        burn_duration_ticks: 0,
    };
    let mut table = [none; 16];
    for m in MATERIAL_REGISTRY {
        if let Some(desc) = m.combustion {
            table[m.id as usize] = CombustionGpuDescriptor {
                is_combustible: 1,
                ignition_threshold: desc.ignition_threshold,
                sustain_threshold: desc.sustain_threshold,
                heat_per_tick: desc.heat_per_tick,
                burn_duration_ticks: desc.burn_duration_ticks,
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
    /// Accumulated fuel progress after this tick's rule (consumed amount in
    /// active burn ticks). Preserved on extinguish; continues on reignition.
    pub fuel_progress: u32,
    /// True when the fuel reached `burn_duration_ticks` this tick — the
    /// cell becomes `EMPTY` (material/T/flags self-reset, no spawn).
    pub consumed: bool,
}

/// Pure reference: applies the G4-C combustion rule to one cell.
///
/// This is a unit/reference helper — the production full-world path is the
/// GPU combustion pass, never a CPU world loop. There is no Oxygen input:
/// ignition depends only on the thermal condition.
///
/// Fuel boundary (matches the GPU pass): the ignition tick is active burn
/// tick 1 (`progress += 1`); when progress reaches `burn_duration_ticks`
/// that tick consumes the fuel (`consumed == true`).
pub fn combustion_step(material_id: u32, temperature: f32, flags: u32) -> CombustionResult {
    let Some(desc) = combustion_descriptor(material_id) else {
        // Non-combustible Matter / EMPTY / unknown: never burns, and the
        // combustion bits (including any stale fuel progress) are cleared.
        return CombustionResult {
            burning: false,
            flame_event: false,
            temperature: sanitize_temperature(temperature),
            fuel_progress: 0,
            consumed: false,
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
    // Fuel progress: +1 per ACTIVE burning tick. Preserved when not burning
    // (extinguish keeps the partial progress; reignition continues from it).
    let mut progress = fuel_progress(flags);
    if burning {
        progress += 1;
    }
    let consumed = burning && progress >= desc.burn_duration_ticks;
    // Cap at the gameplay bound but never reduce an already-hotter cell.
    // A consumed cell resets to the reference temperature (EMPTY is not a
    // thermal medium).
    let temperature = if consumed {
        TEMPERATURE_REFERENCE
    } else if burning {
        (t + desc.heat_per_tick).min(t.max(COMBUSTION_MAX_TEMPERATURE))
    } else {
        t
    };
    CombustionResult {
        burning: burning && !consumed,
        flame_event: burning && !consumed,
        temperature: sanitize_temperature(temperature),
        fuel_progress: progress,
        consumed,
    }
}

/// What `flags_next` should be after the combustion rule: the combustion
/// bits are set/cleared, all unrelated future flag bits are preserved, and
/// a consumed cell resets to `0` (EMPTY has no Matter-owned state).
pub fn combustion_flags_next(flags: u32, result: &CombustionResult) -> u32 {
    if result.consumed {
        return 0;
    }
    let mut next = flags & !combustion_flag_mask();
    if result.burning {
        next |= FLAG_COMBUSTING;
    }
    if result.flame_event {
        next |= FLAG_FLAME_EVENT;
    }
    with_fuel_progress(next, result.fuel_progress)
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
        assert_eq!(wood_gpu.burn_duration_ticks, wood.burn_duration_ticks);
        assert_eq!(oil_gpu.burn_duration_ticks, oil.burn_duration_ticks);
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
        assert!(!combustion_step(MATERIAL_OIL, 199.0, 0).burning);
        assert!(combustion_step(MATERIAL_OIL, 200.0, 0).burning);
        assert!(combustion_step(MATERIAL_OIL, 250.0, 0).burning);
    }

    #[test]
    fn wood_ignition_threshold() {
        assert!(!combustion_step(MATERIAL_WOOD, 299.0, 0).burning);
        assert!(combustion_step(MATERIAL_WOOD, 300.0, 0).burning);
        assert!(combustion_step(MATERIAL_WOOD, 350.0, 0).burning);
    }

    #[test]
    fn burning_above_sustain_continues() {
        let result = combustion_step(MATERIAL_OIL, 175.0, FLAG_COMBUSTING);
        assert!(result.burning);
        assert!(result.flame_event);
    }

    #[test]
    fn burning_below_sustain_extinguishes() {
        let result = combustion_step(MATERIAL_OIL, 149.0, FLAG_COMBUSTING);
        assert!(!result.burning);
        assert!(!result.flame_event);
        // Extinguished matter keeps its (finite) temperature.
        assert!(result.temperature.is_finite());
    }

    #[test]
    fn burning_adds_heat() {
        let result = combustion_step(MATERIAL_OIL, 200.0, FLAG_COMBUSTING);
        assert_eq!(
            result.temperature, 206.0,
            "burning Oil adds heat_per_tick each tick"
        );
        let wood = combustion_step(MATERIAL_WOOD, 300.0, FLAG_COMBUSTING);
        assert_eq!(wood.temperature, 304.0, "burning Wood adds heat_per_tick");
    }

    #[test]
    fn ignition_tick_also_adds_heat() {
        let result = combustion_step(MATERIAL_OIL, 200.0, 0);
        assert!(result.burning);
        assert_eq!(result.temperature, 206.0);
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
        let sealed_hot_wood = combustion_step(MATERIAL_WOOD, 300.0, 0);
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
        assert_eq!(huge.temperature, crate::TEMPERATURE_MAX_C);
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
        let unrelated = 1u32 << 28; // outside the combustion-owned bits
        let result = combustion_step(MATERIAL_OIL, 200.0, FLAG_COMBUSTING | unrelated);
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

    // -----------------------------------------------------------------
    // Finite fuel (G4-C hardening)
    // -----------------------------------------------------------------

    #[test]
    fn wood_has_finite_burn_duration() {
        let wood = combustion_descriptor(MATERIAL_WOOD).expect("Wood combusts");
        assert_eq!(wood.burn_duration_ticks, COMBUSTION_WOOD_BURN_DURATION);
        assert_eq!(
            combustion_table()[MATERIAL_WOOD as usize].burn_duration_ticks,
            900
        );
    }

    #[test]
    fn oil_has_finite_burn_duration() {
        let oil = combustion_descriptor(MATERIAL_OIL).expect("Oil combusts");
        assert_eq!(oil.burn_duration_ticks, COMBUSTION_OIL_BURN_DURATION);
        assert_eq!(
            combustion_table()[MATERIAL_OIL as usize].burn_duration_ticks,
            600
        );
    }

    #[test]
    fn burn_duration_fits_progress_encoding() {
        // The u16 fuel-progress field must hold the full fuel life.
        const {
            assert!(COMBUSTION_WOOD_BURN_DURATION <= 0xFFFF);
        };
        const {
            assert!(COMBUSTION_OIL_BURN_DURATION <= 0xFFFF);
        };
    }

    #[test]
    fn progress_increments_only_while_burning() {
        // Unlit (even hot below sustain or cold) never advances progress.
        let cold_unlit = combustion_step(MATERIAL_WOOD, 0.0, 0);
        assert_eq!(cold_unlit.fuel_progress, 0);
        let below_ignition = combustion_step(MATERIAL_WOOD, 50.0, 0);
        assert_eq!(below_ignition.fuel_progress, 0);
        // Ignition tick is active burn tick 1.
        let ignite = combustion_step(MATERIAL_WOOD, 300.0, 0);
        assert!(ignite.burning);
        assert_eq!(ignite.fuel_progress, 1);
        // Burning continues to advance progress each tick.
        let burning = combustion_step(MATERIAL_WOOD, 300.0, with_fuel_progress(FLAG_COMBUSTING, 5));
        assert_eq!(burning.fuel_progress, 6);
    }

    #[test]
    fn extinguish_preserves_progress() {
        let flags = with_fuel_progress(FLAG_COMBUSTING, 200);
        let result = combustion_step(MATERIAL_WOOD, 20.0, flags); // below sustain
        assert!(!result.burning);
        assert_eq!(result.fuel_progress, 200, "progress survives extinguish");
        // The next flags word keeps the progress bits.
        let next = combustion_flags_next(flags, &result);
        assert_eq!(fuel_progress(next), 200);
        assert_eq!(next & FLAG_COMBUSTING, 0);
    }

    #[test]
    fn reignite_does_not_restore_fuel() {
        // Extinguish at progress 200, then reignite: progress continues at
        // 201 — the fuel is NOT restored to zero.
        let cooled = combustion_step(
            MATERIAL_WOOD,
            20.0,
            with_fuel_progress(FLAG_COMBUSTING, 200),
        );
        assert!(!cooled.burning);
        assert_eq!(cooled.fuel_progress, 200);
        let flags_after_extinguish =
            combustion_flags_next(with_fuel_progress(FLAG_COMBUSTING, 200), &cooled);
        let reignited = combustion_step(MATERIAL_WOOD, 300.0, flags_after_extinguish);
        assert!(reignited.burning);
        assert_eq!(
            reignited.fuel_progress, 201,
            "reignition continues from the remaining fuel"
        );
    }

    #[test]
    fn exact_duration_consumes_fuel() {
        let flags = with_fuel_progress(FLAG_COMBUSTING, 899);
        let result = combustion_step(MATERIAL_WOOD, 300.0, flags);
        assert!(result.consumed, "reaching the burn duration consumes fuel");
        assert!(!result.burning);
        assert_eq!(result.temperature, TEMPERATURE_REFERENCE);
        assert_eq!(
            combustion_flags_next(flags, &result),
            0,
            "consumed cell resets all Matter-owned state"
        );
    }

    #[test]
    fn duration_minus_one_does_not_consume() {
        let flags = with_fuel_progress(FLAG_COMBUSTING, 898);
        let result = combustion_step(MATERIAL_WOOD, 300.0, flags);
        assert!(!result.consumed, "one tick before the duration still burns");
        assert!(result.burning);
        assert_eq!(result.fuel_progress, 899);
    }

    #[test]
    fn nonflammable_has_no_fuel_progress() {
        for id in [MATERIAL_STONE, MATERIAL_SAND, MATERIAL_WATER, MATERIAL_ICE] {
            let result = combustion_step(id, 1000.0, FLAG_COMBUSTING);
            assert!(!result.burning);
            assert_eq!(result.fuel_progress, 0, "material {id} has no fuel");
        }
    }

    #[test]
    fn stale_progress_cleared_on_nonflammable() {
        let stale = FLAG_COMBUSTING | FLAG_FLAME_EVENT | with_fuel_progress(0, 500);
        let result = combustion_step(MATERIAL_STONE, 0.0, stale);
        assert!(!result.consumed);
        let next = combustion_flags_next(stale, &result);
        assert_eq!(
            next & combustion_flag_mask(),
            0,
            "nonflammable Matter cannot keep stale combustion state"
        );
    }

    #[test]
    fn unrelated_flag_bits_preserved() {
        let unrelated = 1u32 << 28;
        let flags = FLAG_COMBUSTING | with_fuel_progress(0, 10) | unrelated;
        let result = combustion_step(MATERIAL_WOOD, 300.0, flags);
        let next = combustion_flags_next(flags, &result);
        assert_ne!(next & unrelated, 0);
        assert_ne!(next & FLAG_COMBUSTING, 0);
        assert_eq!(fuel_progress(next), 11);
    }

    #[test]
    fn progress_encode_decode_boundaries() {
        assert_eq!(fuel_progress(0), 0);
        assert_eq!(fuel_progress(FLAG_FUEL_PROGRESS_MASK), 0x0FFF);
        for p in [0u32, 1, 255, 256, 899, 900, 0x0FFF] {
            let f = with_fuel_progress(0, p);
            assert_eq!(fuel_progress(f), p, "round trip for {p}");
            // Non-fuel bits survive a progress rewrite.
            let base = FLAG_COMBUSTING | FLAG_FLAME_EVENT | (1u32 << 28);
            let f2 = with_fuel_progress(base, p);
            assert_eq!(fuel_progress(f2), p);
            assert_ne!(f2 & FLAG_COMBUSTING, 0);
            assert_ne!(f2 & FLAG_FLAME_EVENT, 0);
            assert_ne!(f2 & (1u32 << 28), 0);
        }
        let overflow = with_fuel_progress(0, 0x1_0000 + 7);
        assert_eq!(fuel_progress(overflow), 7);
    }
}
