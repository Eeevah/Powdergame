//! TE-4I bounded ignition exposure and finite chemical heat.
//!
//! Oil and Wood share one descriptor-driven grammar. Ignition is not an
//! instantaneous temperature threshold: an unlit fuel accumulates a packed
//! six-bit exposure dose only while it has an orthogonal, in-domain EMPTY
//! neighbour with positive Air mass at the combustion-stage snapshot.
//! Burning also requires that binary Air-access predicate. Air is neither
//! consumed nor interpreted as an Oxygen quantity.
//!
//! Matter-owned `flags` layout:
//! - bit 0: `COMBUSTING`;
//! - bit 1: current-tick `FLAME_EVENT`;
//! - bits 2..3 and 28..31: packed ignition exposure (u6);
//! - bits 4..15: finite fuel progress (u12);
//! - bits 16..27: decay ownership (not combustion-owned).

use crate::material::{registry_lookup, MATERIAL_REGISTRY};
use crate::thermal::sanitize_temperature;
use crate::TEMPERATURE_REFERENCE;

pub const FLAG_COMBUSTING: u32 = 1 << 0;
pub const FLAG_FLAME_EVENT: u32 = 1 << 1;
pub const FLAG_IGNITION_EXPOSURE_LOW_MASK: u32 = 0x0000_000C;
pub const FLAG_IGNITION_EXPOSURE_HIGH_MASK: u32 = 0xF000_0000;
pub const FLAG_IGNITION_EXPOSURE_MASK: u32 =
    FLAG_IGNITION_EXPOSURE_LOW_MASK | FLAG_IGNITION_EXPOSURE_HIGH_MASK;
pub const FLAG_FUEL_PROGRESS_SHIFT: u32 = 4;
pub const FLAG_FUEL_PROGRESS_MASK: u32 = 0x0FFF << FLAG_FUEL_PROGRESS_SHIFT;
pub const COMBUSTION_FLAG_MASK: u32 = 0xF000_FFFF;

pub const IGNITION_CONTEXT_EXPOSURE_MASK: u32 = 0x3F;
pub const IGNITION_CONTEXT_IGNITE: u32 = 1 << 6;
pub const IGNITION_CONTEXT_AIR_ACCESS: u32 = 1 << 7;
pub const IGNITION_CONTEXT_MASK: u32 = 0xFF;

pub const COMBUSTION_MAX_TEMPERATURE: f32 = 1200.0;

pub const COMBUSTION_OIL_IGNITION: f32 = 200.0;
pub const COMBUSTION_OIL_SUSTAIN: f32 = 150.0;
pub const COMBUSTION_OIL_CHEMICAL_Q_PER_TICK: f32 = 15.0;
pub const COMBUSTION_OIL_BURN_DURATION: u32 = 600;
pub const COMBUSTION_OIL_IGNITION_BUDGET: u32 = 48;
pub const COMBUSTION_OIL_THERMAL_BASE_RATE: u32 = 2;
pub const COMBUSTION_OIL_THERMAL_BUCKET_WIDTH_C: u32 = 50;
pub const COMBUSTION_OIL_THERMAL_MAX_RATE: u32 = 6;
pub const COMBUSTION_OIL_COOLING_DECAY: u32 = 1;
pub const COMBUSTION_OIL_FLAME_BONUS: u32 = 2;
pub const COMBUSTION_OIL_FLAME_BONUS_CAP: u32 = 4;

pub const COMBUSTION_WOOD_IGNITION: f32 = 300.0;
pub const COMBUSTION_WOOD_SUSTAIN: f32 = 250.0;
pub const COMBUSTION_WOOD_CHEMICAL_Q_PER_TICK: f32 = 8.0;
pub const COMBUSTION_WOOD_BURN_DURATION: u32 = 900;
pub const COMBUSTION_WOOD_IGNITION_BUDGET: u32 = 60;
pub const COMBUSTION_WOOD_THERMAL_BASE_RATE: u32 = 1;
pub const COMBUSTION_WOOD_THERMAL_BUCKET_WIDTH_C: u32 = 50;
pub const COMBUSTION_WOOD_THERMAL_MAX_RATE: u32 = 5;
pub const COMBUSTION_WOOD_COOLING_DECAY: u32 = 1;
pub const COMBUSTION_WOOD_FLAME_BONUS: u32 = 2;
pub const COMBUSTION_WOOD_FLAME_BONUS_CAP: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombustionDescriptor {
    pub ignition_threshold: f32,
    pub sustain_threshold: f32,
    /// Authoritative finite chemical energy released on one emitting tick.
    pub chemical_q_per_tick: f32,
    pub burn_duration_ticks: u32,
    pub ignition_budget: u32,
    pub thermal_base_rate: u32,
    pub thermal_bucket_width_c: u32,
    pub thermal_max_rate: u32,
    pub cooling_decay: u32,
    pub flame_bonus: u32,
    pub flame_bonus_cap: u32,
}

/// Exact 32-byte Rust/WGSL upload record.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombustionGpuDescriptor {
    pub is_combustible: u32,
    pub ignition_threshold: f32,
    pub sustain_threshold: f32,
    pub chemical_delta_t: f32,
    pub burn_duration_ticks: u32,
    /// low byte budget, next byte cooling decay
    pub budget_decay: u32,
    /// low byte base, next byte max, next byte bucket width in C
    pub thermal_rates: u32,
    /// low byte flame bonus, next byte flame bonus cap
    pub flame_rates: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IgnitionContext {
    pub next_exposure: u32,
    pub ignite: bool,
    pub air_access: bool,
    pub thermal_rate: u32,
    pub flame_rate: u32,
    pub previous_flame_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombustionResult {
    pub burning: bool,
    pub flame_event: bool,
    pub temperature: f32,
    pub exposure: u32,
    pub fuel_progress: u32,
    pub consumed: bool,
    pub gross_chemical_q: f32,
    pub deposited_chemical_q: f32,
    pub clipped_chemical_q: f32,
}

pub fn combustion_descriptor(id: u32) -> Option<&'static CombustionDescriptor> {
    registry_lookup(id).and_then(|material| material.combustion.as_ref())
}

pub fn fuel_progress(flags: u32) -> u32 {
    (flags & FLAG_FUEL_PROGRESS_MASK) >> FLAG_FUEL_PROGRESS_SHIFT
}

pub fn with_fuel_progress(flags: u32, progress: u32) -> u32 {
    assert!(
        progress <= 0x0FFF,
        "fuel progress must fit the owned u12 field"
    );
    (flags & !FLAG_FUEL_PROGRESS_MASK) | (progress << FLAG_FUEL_PROGRESS_SHIFT)
}

pub fn ignition_exposure(flags: u32) -> u32 {
    ((flags >> 2) & 0x3) | (((flags >> 28) & 0xF) << 2)
}

pub fn with_ignition_exposure(flags: u32, exposure: u32) -> u32 {
    assert!(
        exposure <= 63,
        "ignition exposure must fit the owned u6 field"
    );
    let low = (exposure & 0x3) << 2;
    let high = ((exposure >> 2) & 0xF) << 28;
    (flags & !FLAG_IGNITION_EXPOSURE_MASK) | low | high
}

pub fn combustion_flag_mask() -> u32 {
    COMBUSTION_FLAG_MASK
}

fn pack_u8(value: u32, field: &str) -> u32 {
    assert!(value <= u8::MAX as u32, "{field} does not fit u8");
    value
}

fn compile_gpu_descriptor(
    desc: CombustionDescriptor,
    heat_capacity: f32,
) -> CombustionGpuDescriptor {
    assert!(heat_capacity.is_finite() && heat_capacity > 0.0);
    assert!(desc.chemical_q_per_tick.is_finite() && desc.chemical_q_per_tick >= 0.0);
    assert!(desc.ignition_threshold.is_finite() && desc.sustain_threshold.is_finite());
    assert!((1..=63).contains(&desc.ignition_budget));
    assert!(desc.thermal_bucket_width_c > 0);
    let budget_decay = pack_u8(desc.ignition_budget, "ignition budget")
        | (pack_u8(desc.cooling_decay, "cooling decay") << 8);
    let thermal_rates = pack_u8(desc.thermal_base_rate, "thermal base rate")
        | (pack_u8(desc.thermal_max_rate, "thermal max rate") << 8)
        | (pack_u8(desc.thermal_bucket_width_c, "thermal bucket width") << 16);
    let flame_rates = pack_u8(desc.flame_bonus, "flame bonus")
        | (pack_u8(desc.flame_bonus_cap, "flame bonus cap") << 8);
    CombustionGpuDescriptor {
        is_combustible: 1,
        ignition_threshold: desc.ignition_threshold,
        sustain_threshold: desc.sustain_threshold,
        chemical_delta_t: desc.chemical_q_per_tick / heat_capacity,
        burn_duration_ticks: desc.burn_duration_ticks,
        budget_decay,
        thermal_rates,
        flame_rates,
    }
}

pub fn combustion_table() -> [CombustionGpuDescriptor; 16] {
    let none = CombustionGpuDescriptor {
        is_combustible: 0,
        ignition_threshold: 0.0,
        sustain_threshold: 0.0,
        chemical_delta_t: 0.0,
        burn_duration_ticks: 0,
        budget_decay: 0,
        thermal_rates: 0,
        flame_rates: 0,
    };
    let mut table = [none; 16];
    for material in MATERIAL_REGISTRY {
        if let Some(desc) = material.combustion {
            table[material.id as usize] = compile_gpu_descriptor(desc, material.heat_capacity);
        }
    }
    table
}

/// Exact bytes uploaded to the 16 x 32-byte WGSL uniform table.
pub fn combustion_table_bytes() -> [u8; 512] {
    let mut bytes = [0u8; 512];
    for (index, descriptor) in combustion_table().iter().enumerate() {
        let offset = index * 32;
        for (field, value) in [
            (0, descriptor.is_combustible.to_ne_bytes()),
            (4, descriptor.ignition_threshold.to_ne_bytes()),
            (8, descriptor.sustain_threshold.to_ne_bytes()),
            (12, descriptor.chemical_delta_t.to_ne_bytes()),
            (16, descriptor.burn_duration_ticks.to_ne_bytes()),
            (20, descriptor.budget_decay.to_ne_bytes()),
            (24, descriptor.thermal_rates.to_ne_bytes()),
            (28, descriptor.flame_rates.to_ne_bytes()),
        ] {
            bytes[offset + field..offset + field + 4].copy_from_slice(&value);
        }
    }
    bytes
}

pub fn ignition_context(
    material_id: u32,
    temperature: f32,
    flags: u32,
    air_access: bool,
    previous_flame_count: u32,
) -> IgnitionContext {
    let Some(desc) = combustion_descriptor(material_id) else {
        return IgnitionContext {
            next_exposure: 0,
            ignite: false,
            air_access: false,
            thermal_rate: 0,
            flame_rate: 0,
            previous_flame_count,
        };
    };
    if flags & FLAG_COMBUSTING != 0 {
        return IgnitionContext {
            next_exposure: 0,
            ignite: false,
            air_access,
            thermal_rate: 0,
            flame_rate: 0,
            previous_flame_count,
        };
    }
    let t = sanitize_temperature(temperature);
    let thermal_eligible = air_access && t >= desc.ignition_threshold;
    let thermal_rate = if thermal_eligible {
        let excess = (t - desc.ignition_threshold).max(0.0);
        let buckets = (excess / desc.thermal_bucket_width_c as f32).floor() as u32;
        desc.thermal_max_rate.min(desc.thermal_base_rate + buckets)
    } else {
        0
    };
    let flame_rate = if thermal_eligible {
        desc.flame_bonus_cap
            .min(previous_flame_count.saturating_mul(desc.flame_bonus))
    } else {
        0
    };
    let previous = ignition_exposure(flags);
    let next_exposure = if thermal_eligible {
        desc.ignition_budget
            .min(previous + thermal_rate + flame_rate)
    } else {
        previous.saturating_sub(desc.cooling_decay)
    };
    let ignite = next_exposure >= desc.ignition_budget;
    IgnitionContext {
        next_exposure: if ignite { 0 } else { next_exposure },
        ignite,
        air_access,
        thermal_rate,
        flame_rate,
        previous_flame_count,
    }
}

pub fn encode_ignition_context(context: IgnitionContext) -> u32 {
    assert!(context.next_exposure <= 63);
    context.next_exposure
        | if context.ignite {
            IGNITION_CONTEXT_IGNITE
        } else {
            0
        }
        | if context.air_access {
            IGNITION_CONTEXT_AIR_ACCESS
        } else {
            0
        }
}

pub fn decode_ignition_context(encoded: u32) -> IgnitionContext {
    assert_eq!(
        encoded & !IGNITION_CONTEXT_MASK,
        0,
        "reserved context bits set"
    );
    IgnitionContext {
        next_exposure: encoded & IGNITION_CONTEXT_EXPOSURE_MASK,
        ignite: encoded & IGNITION_CONTEXT_IGNITE != 0,
        air_access: encoded & IGNITION_CONTEXT_AIR_ACCESS != 0,
        thermal_rate: 0,
        flame_rate: 0,
        previous_flame_count: 0,
    }
}

/// Pure combustion-stage reference. `context` must come from
/// `ignition_context` over the same production snapshot.
pub fn combustion_step(
    material_id: u32,
    temperature: f32,
    flags: u32,
    context: IgnitionContext,
) -> CombustionResult {
    let Some(desc) = combustion_descriptor(material_id) else {
        return CombustionResult {
            burning: false,
            flame_event: false,
            temperature: sanitize_temperature(temperature),
            exposure: 0,
            fuel_progress: 0,
            consumed: false,
            gross_chemical_q: 0.0,
            deposited_chemical_q: 0.0,
            clipped_chemical_q: 0.0,
        };
    };
    let t = sanitize_temperature(temperature);
    let was_burning = flags & FLAG_COMBUSTING != 0;
    let mut burning = if was_burning {
        context.air_access
    } else {
        context.ignite
    };
    if burning && t < desc.sustain_threshold {
        burning = false;
    }
    let old_progress = fuel_progress(flags);
    let candidate_progress = if burning {
        old_progress + 1
    } else {
        old_progress
    };
    let consumed = burning && candidate_progress >= desc.burn_duration_ticks;
    let emitting = burning && !consumed;
    let capacity = registry_lookup(material_id)
        .expect("combustible descriptor must belong to a registered material")
        .heat_capacity;
    let gross = if emitting {
        desc.chemical_q_per_tick
    } else {
        0.0
    };
    let temperature_out = if consumed {
        TEMPERATURE_REFERENCE
    } else if emitting {
        let delta_t = desc.chemical_q_per_tick / capacity;
        (t + delta_t).min(t.max(COMBUSTION_MAX_TEMPERATURE))
    } else {
        t
    };
    let deposited = if emitting {
        (capacity * (temperature_out - t)).clamp(0.0, gross)
    } else {
        0.0
    };
    let clipped = (gross - deposited).max(0.0);
    CombustionResult {
        burning: emitting,
        flame_event: emitting,
        temperature: sanitize_temperature(temperature_out),
        exposure: if emitting || consumed {
            0
        } else {
            context.next_exposure
        },
        fuel_progress: candidate_progress,
        consumed,
        gross_chemical_q: gross,
        deposited_chemical_q: deposited,
        clipped_chemical_q: clipped,
    }
}

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
    next = with_fuel_progress(next, result.fuel_progress);
    with_ignition_exposure(next, result.exposure)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmokeSpawnDirection {
    Up,
    UpLeft,
    UpRight,
    Left,
    Right,
}

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
    let diagonals = if parity == 0 {
        [
            (up_left, SmokeSpawnDirection::UpLeft),
            (up_right, SmokeSpawnDirection::UpRight),
        ]
    } else {
        [
            (up_right, SmokeSpawnDirection::UpRight),
            (up_left, SmokeSpawnDirection::UpLeft),
        ]
    };
    for (available, direction) in diagonals {
        if available {
            return Some(direction);
        }
    }
    let laterals = if parity == 0 {
        [
            (left, SmokeSpawnDirection::Left),
            (right, SmokeSpawnDirection::Right),
        ]
    } else {
        [
            (right, SmokeSpawnDirection::Right),
            (left, SmokeSpawnDirection::Left),
        ]
    };
    for (available, direction) in laterals {
        if available {
            return Some(direction);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{
        MATERIAL_EMPTY, MATERIAL_OIL, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD,
    };

    fn context(
        material: u32,
        temperature: f32,
        flags: u32,
        air: bool,
        flames: u32,
    ) -> IgnitionContext {
        ignition_context(material, temperature, flags, air, flames)
    }

    #[test]
    fn exposure_u6_round_trips_without_touching_decay_or_fuel() {
        let unrelated = 0x0FFF_0000 | FLAG_COMBUSTING | with_fuel_progress(0, 777);
        for exposure in 0..=63 {
            let flags = with_ignition_exposure(unrelated, exposure);
            assert_eq!(ignition_exposure(flags), exposure);
            assert_eq!(flags & 0x0FFF_0000, unrelated & 0x0FFF_0000);
            assert_eq!(fuel_progress(flags), 777);
        }
        assert_eq!(combustion_flag_mask(), 0xF000_FFFF);
    }

    #[test]
    #[should_panic(expected = "ignition exposure must fit")]
    fn invalid_exposure_is_rejected_not_truncated() {
        let _ = with_ignition_exposure(0, 64);
    }

    #[test]
    fn locked_threshold_timings_and_temperature_buckets_match() {
        for (material, temperature, expected_tick) in [
            (MATERIAL_OIL, 200.0, 24),
            (MATERIAL_OIL, 300.0, 12),
            (MATERIAL_WOOD, 300.0, 60),
            (MATERIAL_WOOD, 400.0, 20),
        ] {
            let mut flags = 0;
            for tick in 1..=expected_tick {
                let c = context(material, temperature, flags, true, 0);
                assert_eq!(c.ignite, tick == expected_tick, "tick {tick}");
                if c.ignite {
                    let result = combustion_step(material, temperature, flags, c);
                    assert!(result.burning);
                    assert_eq!(result.fuel_progress, 1);
                } else {
                    flags = with_ignition_exposure(flags, c.next_exposure);
                }
            }
        }
    }

    #[test]
    fn flame_bonus_requires_air_and_own_threshold() {
        let oil = context(MATERIAL_OIL, 200.0, 0, true, 1);
        assert_eq!((oil.thermal_rate, oil.flame_rate), (2, 2));
        let below = context(MATERIAL_OIL, 199.0, 0, true, 4);
        assert_eq!((below.thermal_rate, below.flame_rate), (0, 0));
        let vacuum = context(MATERIAL_OIL, 400.0, 0, false, 4);
        assert_eq!((vacuum.thermal_rate, vacuum.flame_rate), (0, 0));
    }

    #[test]
    fn cooling_decay_is_monotonic_and_reheating_keeps_surviving_dose() {
        let flags = with_ignition_exposure(0, 10);
        assert_eq!(context(MATERIAL_OIL, 20.0, flags, true, 0).next_exposure, 9);
        assert_eq!(
            context(MATERIAL_OIL, 300.0, flags, true, 0).next_exposure,
            14
        );
        assert_eq!(
            context(MATERIAL_OIL, 400.0, flags, false, 4).next_exposure,
            9
        );
    }

    #[test]
    fn context_encoding_uses_only_low_byte() {
        for exposure in [0, 1, 47, 63] {
            let value = IgnitionContext {
                next_exposure: exposure,
                ignite: exposure == 0,
                air_access: true,
                thermal_rate: 0,
                flame_rate: 0,
                previous_flame_count: 0,
            };
            let encoded = encode_ignition_context(value);
            assert_eq!(encoded & !IGNITION_CONTEXT_MASK, 0);
            let decoded = decode_ignition_context(encoded);
            assert_eq!(
                (decoded.next_exposure, decoded.ignite, decoded.air_access),
                (exposure, value.ignite, true)
            );
        }
    }

    #[test]
    fn chemical_q_compiles_from_live_heat_capacity() {
        let table = combustion_table();
        assert_eq!(table[MATERIAL_OIL as usize].chemical_delta_t, 6.0);
        assert_eq!(table[MATERIAL_WOOD as usize].chemical_delta_t, 4.0);
        assert_eq!(std::mem::size_of::<CombustionGpuDescriptor>(), 32);
        assert_eq!(std::mem::align_of::<CombustionGpuDescriptor>(), 4);
    }

    #[cfg(target_endian = "little")]
    #[test]
    fn gpu_descriptor_offsets_and_bytes_are_exact() {
        assert_eq!(
            std::mem::offset_of!(CombustionGpuDescriptor, is_combustible),
            0
        );
        assert_eq!(
            std::mem::offset_of!(CombustionGpuDescriptor, ignition_threshold),
            4
        );
        assert_eq!(
            std::mem::offset_of!(CombustionGpuDescriptor, sustain_threshold),
            8
        );
        assert_eq!(
            std::mem::offset_of!(CombustionGpuDescriptor, chemical_delta_t),
            12
        );
        assert_eq!(
            std::mem::offset_of!(CombustionGpuDescriptor, burn_duration_ticks),
            16
        );
        assert_eq!(
            std::mem::offset_of!(CombustionGpuDescriptor, budget_decay),
            20
        );
        assert_eq!(
            std::mem::offset_of!(CombustionGpuDescriptor, thermal_rates),
            24
        );
        assert_eq!(
            std::mem::offset_of!(CombustionGpuDescriptor, flame_rates),
            28
        );
        let bytes = combustion_table_bytes();
        assert_eq!(
            &bytes[MATERIAL_OIL as usize * 32..MATERIAL_OIL as usize * 32 + 32],
            &[
                1, 0, 0, 0, 0, 0, 72, 67, 0, 0, 22, 67, 0, 0, 192, 64, 88, 2, 0, 0, 48, 1, 0, 0, 2,
                6, 50, 0, 2, 4, 0, 0,
            ]
        );
        assert_eq!(
            &bytes[MATERIAL_WOOD as usize * 32..MATERIAL_WOOD as usize * 32 + 32],
            &[
                1, 0, 0, 0, 0, 0, 150, 67, 0, 0, 122, 67, 0, 0, 128, 64, 132, 3, 0, 0, 60, 1, 0, 0,
                1, 5, 50, 0, 2, 4, 0, 0,
            ]
        );
        let hash = bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01B3)
        });
        assert_eq!(
            hash, 0x86b6_ad52_6f7e_38f2,
            "descriptor table byte fixture changed"
        );
    }

    #[test]
    fn consume_before_emission_closes_locked_lifecycle_totals() {
        for (material, duration, expected) in
            [(MATERIAL_OIL, 600, 8_985.0), (MATERIAL_WOOD, 900, 7_192.0)]
        {
            let mut flags = FLAG_COMBUSTING;
            let mut total = 0.0;
            for tick in 1..=duration {
                let c = context(material, 1000.0, flags, true, 0);
                let result = combustion_step(material, 1000.0, flags, c);
                total += result.gross_chemical_q;
                assert_eq!(result.consumed, tick == duration);
                if tick == duration {
                    assert!(!result.flame_event);
                    assert_eq!(result.gross_chemical_q, 0.0);
                } else {
                    flags = combustion_flags_next(flags, &result);
                }
            }
            assert_eq!(total, expected);
        }
    }

    #[test]
    fn chemical_q_deposited_plus_clipped_equals_gross() {
        let c = context(MATERIAL_OIL, 1199.0, FLAG_COMBUSTING, true, 0);
        let result = combustion_step(MATERIAL_OIL, 1199.0, FLAG_COMBUSTING, c);
        assert_eq!(
            (
                result.gross_chemical_q,
                result.deposited_chemical_q,
                result.clipped_chemical_q
            ),
            (15.0, 2.5, 12.5)
        );
    }

    #[test]
    fn burning_without_air_extinguishes_before_emission_and_preserves_fuel() {
        let flags = with_fuel_progress(FLAG_COMBUSTING, 17);
        let c = context(MATERIAL_WOOD, 500.0, flags, false, 4);
        let result = combustion_step(MATERIAL_WOOD, 500.0, flags, c);
        assert!(!result.burning);
        assert_eq!(
            (
                result.fuel_progress,
                result.gross_chemical_q,
                result.exposure
            ),
            (17, 0.0, 0)
        );
    }

    #[test]
    fn non_combustible_and_empty_clear_owned_state() {
        let stale = COMBUSTION_FLAG_MASK;
        for material in [MATERIAL_EMPTY, MATERIAL_STONE, MATERIAL_WATER] {
            let c = context(material, 1000.0, stale, true, 4);
            let result = combustion_step(material, 1000.0, stale, c);
            assert_eq!(
                combustion_flags_next(stale, &result) & COMBUSTION_FLAG_MASK,
                0
            );
        }
    }

    #[test]
    fn smoke_stencil_keeps_existing_order() {
        assert_eq!(
            pick_smoke_spawn(true, true, true, true, true, 0),
            Some(SmokeSpawnDirection::Up)
        );
        assert_eq!(
            pick_smoke_spawn(false, true, true, false, false, 1),
            Some(SmokeSpawnDirection::UpRight)
        );
        assert_eq!(pick_smoke_spawn(false, false, false, false, false, 0), None);
    }
}
