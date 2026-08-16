//! G7-A — Chunk Activity measurement semantics.
//!
//! Dense State, Sparse Work: world storage stays dense (material /
//! temperature / pressure / flags Current/Next); what changes is the
//! *changeable frontier* that decides which chunks are computationally
//! relevant. This module defines the measurement baseline only — G7-A
//! records and visualizes activity; it does not yet skip any subsystem
//! dispatch (that is G7-B work).
//!
//! Activity is a per-chunk u32 bitmask (one chunk = `chunk_size`² cells).
//! A cell contributes a bit when a *meaningful changeable frontier* exists
//! at that cell this tick:
//!
//! - `ACTIVITY_MATTER`: a movable Matter cell whose ordered local stencil
//!   has any real candidate (EMPTY move, density-swap-appropriate neighbor,
//!   or an out-of-domain Void exit). Existence of Matter is NOT activity.
//! - `ACTIVITY_THERMAL`: a cell with a relevant 4-neighbor temperature
//!   gradient, an active heat source (combusting Matter), a phase rule
//!   currently satisfied on its own Material + Temperature, or a phase
//!   transition that actually fired this tick (the phase pass self-marks
//!   the transition in the activity buffer; the detector additionally
//!   evaluates the phase condition as a defensive check — 1:1 transitions
//!   self-resolve within one tick, so the marker is the observable
//!   signal).
//! - `ACTIVITY_PRESSURE`: a cell with a non-trivial 4-neighbor pressure
//!   gradient, evaluated on pressure-media cells only (LIQUID/GAS per the
//!   G5 contract — EMPTY/STATIC/POWDER have their pressure field zeroed
//!   every tick and never carry a pressure frontier).
//! - `ACTIVITY_REACTION`: a cell whose reaction state is actively changing
//!   (combusting Matter, or Matter with a progressing decay age).
//!
//! Chunk seams: the cell-level stencil reads 1-cell neighbors in world
//! coordinates, so a frontier across a chunk boundary is detected normally;
//! there is no dedicated chunk-to-chunk wake propagation pass yet (G7-B).
//! `chunk_changed_this_tick` means "a frontier was present this tick" (it
//! resets the stable counter) — it does NOT compare previous/next world
//! state; state-delta dirty tracking, if ever needed, is separate G7-B
//! work.
//!
//! The exact epsilons are gameplay measurement baselines, not physical
//! constants; sleep thresholds are NOT chosen here (that decision is left
//! to a future benchmark-driven step, per MILESTONES G7).

/// Matter/interface frontier present at this cell (movement or density
/// candidate exists).
pub const ACTIVITY_MATTER: u32 = 1 << 0;
/// Relevant temperature gradient or heat source present.
pub const ACTIVITY_THERMAL: u32 = 1 << 1;
/// Non-trivial pressure gradient present.
pub const ACTIVITY_PRESSURE: u32 = 1 << 2;
/// Reaction state actively changing (combustion / decay).
pub const ACTIVITY_REACTION: u32 = 1 << 3;

/// All defined activity bits.
pub const ACTIVITY_ALL_BITS: u32 =
    ACTIVITY_MATTER | ACTIVITY_THERMAL | ACTIVITY_PRESSURE | ACTIVITY_REACTION;

/// Smallest absolute temperature difference considered a meaningful
/// thermal frontier (gameplay scalar, not Celsius).
pub const THERMAL_ACTIVITY_EPS: f32 = 0.001;
/// Smallest absolute pressure difference considered a meaningful pressure
/// frontier.
pub const PRESSURE_ACTIVITY_EPS: f32 = 0.001;

/// Number of chunks along X for a world of `width` cells.
pub fn chunks_x(width: u32, chunk_size: u32) -> u32 {
    width.div_ceil(chunk_size)
}

/// Number of chunks along Y for a world of `height` cells.
pub fn chunks_y(height: u32, chunk_size: u32) -> u32 {
    height.div_ceil(chunk_size)
}

/// Total number of chunks in the world.
pub fn chunk_count(width: u32, height: u32, chunk_size: u32) -> u32 {
    chunks_x(width, chunk_size) * chunks_y(height, chunk_size)
}

/// Stable-duration update: a chunk that had no meaningful activity this
/// tick counts one more consecutive stable tick (saturating); any activity
/// resets the counter to zero.
///
/// This is the G7-A measurement of "how long has this chunk had no
/// changeable frontier" — used for observation only, not yet a sleep
/// cutoff.
pub fn stable_ticks_update(chunk_activity: &[u32], prev_stable: &[u32]) -> Vec<u32> {
    chunk_activity
        .iter()
        .zip(prev_stable.iter())
        .map(|(&mask, &stable)| {
            if mask == 0 {
                stable.saturating_add(1)
            } else {
                0
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_constants_are_disjoint() {
        assert_eq!(ACTIVITY_ALL_BITS, 0b1111);
        assert_eq!(ACTIVITY_MATTER & ACTIVITY_THERMAL, 0);
        assert_eq!(ACTIVITY_THERMAL & ACTIVITY_PRESSURE, 0);
        assert_eq!(ACTIVITY_PRESSURE & ACTIVITY_REACTION, 0);
        assert_eq!(ACTIVITY_REACTION & ACTIVITY_MATTER, 0);
    }

    #[test]
    fn chunk_geometry_reference_world() {
        // 2048×2048 chunk 64 → 32×32 = 1024 chunks.
        assert_eq!(chunks_x(2048, 64), 32);
        assert_eq!(chunks_y(2048, 64), 32);
        assert_eq!(chunk_count(2048, 2048, 64), 1024);
    }

    #[test]
    fn chunk_geometry_non_multiple() {
        // 320×192 chunk 64 → 5×3 (partial edge chunks).
        assert_eq!(chunks_x(320, 64), 5);
        assert_eq!(chunks_y(192, 64), 3);
        assert_eq!(chunk_count(320, 192, 64), 15);
    }

    #[test]
    fn stable_ticks_increment_when_inactive() {
        let activity = [0u32, 0, 0];
        let prev = [0u32, 3, 100];
        assert_eq!(stable_ticks_update(&activity, &prev), vec![1, 4, 101]);
    }

    #[test]
    fn stable_ticks_reset_on_activity() {
        let activity = [ACTIVITY_MATTER, 0, ACTIVITY_REACTION];
        let prev = [7u32, 42, 0];
        assert_eq!(stable_ticks_update(&activity, &prev), vec![0, 43, 0]);
    }

    #[test]
    fn stable_ticks_saturate() {
        let activity = [0u32];
        let prev = [u32::MAX];
        assert_eq!(stable_ticks_update(&activity, &prev), vec![u32::MAX]);
    }
}
