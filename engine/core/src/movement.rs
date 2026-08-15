//! Movement candidate selection — pure/reference logic.
//!
//! Purpose (`SIMULATION_SPEC` §7/§11/§12, `DEVELOPMENT.md` §4 Step 3):
//! unit tests, algorithm explanation and semantic comparison against the GPU
//! production path. The production 2048×2048 world is never simulated on the
//! CPU; this module only defines the behavior contract.
//!
//! G2 contract:
//! - movement reads **Current** state only,
//! - only **1-cell local** neighbors are considered (no teleport, no scan),
//! - **First-Match**: the first valid candidate wins and searching stops,
//! - an out-of-domain position is `Void` for **every** stencil candidate —
//!   primary, diagonal or lateral. An open side/top/bottom boundary is a
//!   Void exit, never an invisible wall, never clamped, never treated as an
//!   EMPTY cell.
//!
//! G3 adds local density displacement (`SIMULATION_SPEC` §12):
//! - `EMPTY` destination → normal move,
//! - STATIC target (no density rank) → blocked,
//! - movable target + appropriate rank ordering → **local swap** candidate,
//! - movable target + inappropriate ordering → blocked (equal ranks never
//!   swap),
//! - only the gravity-aligned vertical stages (down/down-diagonal for
//!   POWDER/LIQUID, up/up-diagonal for GAS) may swap; **lateral** candidates
//!   stay EMPTY-only so liquids/gases do not jitter sideways on density.

use crate::material::{density_rank, MovementClass};

/// State of a candidate cell as seen by a mover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    /// In-domain cell with no Matter.
    Empty,
    /// In-domain cell occupied by Matter. `Some(rank)` = movable density
    /// rank (a possible swap target); `None` = STATIC — never swapped.
    Matter(Option<u32>),
}

/// Where a mover proposes to go this tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveTarget {
    /// Stay in place (no valid candidate / STATIC).
    NoMove,
    /// Move to this in-domain 1-cell neighbor (destination is EMPTY).
    Cell(i64, i64),
    /// Local density swap with this in-domain neighbor (both cells exchange
    /// Matter this tick).
    Swap(i64, i64),
    /// Leave the world through an open boundary (source becomes EMPTY).
    Void,
}

/// Direction of a vertical density-displacement candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityDirection {
    /// Gravity-aligned: heavier sinks (source rank > destination rank).
    Downward,
    /// Gas-aligned: lighter rises (source rank < destination rank).
    Upward,
}

/// Whether a density displacement is appropriate for this rank pair.
///
/// Only strict ordering swaps: equal ranks never swap, STATIC targets never
/// swap. Lateral candidates never call this (they are EMPTY-only).
pub fn density_displacement_allowed(
    source_rank: u32,
    dest_rank: u32,
    direction: DensityDirection,
) -> bool {
    match direction {
        DensityDirection::Downward => source_rank > dest_rank,
        DensityDirection::Upward => source_rank < dest_rank,
    }
}

/// Cheap stateless left/right ordering: `true` = prefer the left candidate.
///
/// Used so symmetric candidates do not always resolve to the same fixed
/// side across the whole world, without any RNG state (DETERMINISM_SPEC §4,
/// SIMULATION_SPEC §10).
pub fn prefer_left(x: i64, y: i64) -> bool {
    (x + y) & 1 == 0
}

/// Selects the first valid 1-cell local destination for `class` at `(x, y)`.
///
/// `source_material` provides the mover's density rank. `lookup(dx, dy)`
/// returns the state of the in-domain cell at `(x+dx, y+dy)`:
/// - `Some(Empty)` — valid normal-move destination,
/// - `Some(Matter(_))` — occupied; a density swap on vertical stages, or
///   blocked otherwise; try the next candidate,
/// - `None` — out-of-domain (Void). For EVERY stencil candidate this is a
///   `Void` exit: the mover leaves the world there. It is never an invisible
///   wall and never clamped.
pub fn propose_move(
    class: MovementClass,
    source_material: u32,
    x: i64,
    y: i64,
    lookup: impl Fn(i64, i64) -> Option<CellState>,
) -> MoveTarget {
    let source_rank = density_rank(source_material);
    match class {
        MovementClass::Static => MoveTarget::NoMove,
        MovementClass::Powder => {
            // down → down-diagonal → stop
            if let Some(t) =
                vertical_candidate(x, y, 0, 1, source_rank, DensityDirection::Downward, &lookup)
            {
                return t;
            }
            try_diagonals(
                x,
                y,
                1,
                source_rank,
                DensityDirection::Downward,
                prefer_left(x, y),
                &lookup,
            )
            .unwrap_or(MoveTarget::NoMove)
        }
        MovementClass::Liquid => {
            // down → down-diagonal → lateral → stop
            if let Some(t) =
                vertical_candidate(x, y, 0, 1, source_rank, DensityDirection::Downward, &lookup)
            {
                return t;
            }
            if let Some(target) = try_diagonals(
                x,
                y,
                1,
                source_rank,
                DensityDirection::Downward,
                prefer_left(x, y),
                &lookup,
            ) {
                return target;
            }
            try_lateral(x, y, prefer_left(x, y), &lookup).unwrap_or(MoveTarget::NoMove)
        }
        MovementClass::Gas => {
            // up → up-diagonal → lateral → stop
            if let Some(t) =
                vertical_candidate(x, y, 0, -1, source_rank, DensityDirection::Upward, &lookup)
            {
                return t;
            }
            if let Some(target) = try_diagonals(
                x,
                y,
                -1,
                source_rank,
                DensityDirection::Upward,
                prefer_left(x, y),
                &lookup,
            ) {
                return target;
            }
            try_lateral(x, y, prefer_left(x, y), &lookup).unwrap_or(MoveTarget::NoMove)
        }
    }
}

/// Evaluates one vertical candidate (`dx`, `dy`): Void exit, EMPTY normal
/// move, or appropriate density swap. `None` means "try the next candidate".
fn vertical_candidate(
    x: i64,
    y: i64,
    dx: i64,
    dy: i64,
    source_rank: Option<u32>,
    direction: DensityDirection,
    lookup: &impl Fn(i64, i64) -> Option<CellState>,
) -> Option<MoveTarget> {
    match lookup(dx, dy) {
        None => Some(MoveTarget::Void),
        Some(CellState::Empty) => Some(MoveTarget::Cell(x + dx, y + dy)),
        Some(CellState::Matter(dest_rank)) => {
            match (source_rank, dest_rank) {
                (Some(s), Some(d)) if density_displacement_allowed(s, d, direction) => {
                    Some(MoveTarget::Swap(x + dx, y + dy))
                }
                // STATIC target or inappropriate ordering: next candidate.
                _ => None,
            }
        }
    }
}

/// First-match diagonal candidates (±1 in `dy` direction), ordered by parity.
///
/// Out-of-domain candidates are `MoveTarget::Void` exits (open side/top/
/// bottom boundaries are not invisible walls); in-domain occupied cells are
/// density-swap candidates on the vertical stage or just fall through to the
/// next candidate. Returns `None` only when every candidate was an in-domain
/// non-swappable cell.
fn try_diagonals(
    x: i64,
    y: i64,
    dy: i64,
    source_rank: Option<u32>,
    direction: DensityDirection,
    left_first: bool,
    lookup: &impl Fn(i64, i64) -> Option<CellState>,
) -> Option<MoveTarget> {
    let (l, r) = if left_first {
        ((-1, dy), (1, dy))
    } else {
        ((1, dy), (-1, dy))
    };
    for (dx, ddy) in [l, r] {
        if let Some(t) = vertical_candidate(x, y, dx, ddy, source_rank, direction, lookup) {
            return Some(t);
        }
    }
    None
}

/// First-match lateral candidate (one cell), ordered by parity.
///
/// Laterals are EMPTY-only in G3 (no lateral density swap — no sideways
/// jitter). Same Void semantics as [`try_diagonals`]: an out-of-domain
/// lateral is a Void exit, not a wall. Returns `None` only when every
/// candidate was an in-domain blocked cell.
fn try_lateral(
    x: i64,
    y: i64,
    left_first: bool,
    lookup: &impl Fn(i64, i64) -> Option<CellState>,
) -> Option<MoveTarget> {
    let (l, r) = if left_first { (-1, 1) } else { (1, -1) };
    for dx in [l, r] {
        match lookup(dx, 0) {
            None => return Some(MoveTarget::Void),
            Some(CellState::Empty) => return Some(MoveTarget::Cell(x + dx, y)),
            Some(CellState::Matter(_)) => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{
        DENSITY_RANK_OIL, DENSITY_RANK_SAND, DENSITY_RANK_SMOKE, DENSITY_RANK_STEAM,
        DENSITY_RANK_WATER, MATERIAL_EMPTY, MATERIAL_OIL, MATERIAL_SAND, MATERIAL_SMOKE,
        MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER,
    };

    /// Builds a lookup over a small grid; `None` for out-of-bounds.
    /// Occupied cells carry their material's density rank (`None` for
    /// STATIC) so density semantics can be exercised.
    fn lookup_from(
        grid: &[u32],
        width: i64,
        height: i64,
        x: i64,
        y: i64,
        dx: i64,
        dy: i64,
    ) -> Option<CellState> {
        let nx = x + dx;
        let ny = y + dy;
        if nx < 0 || ny < 0 || nx >= width || ny >= height {
            return None;
        }
        let idx = (ny * width + nx) as usize;
        Some(if grid[idx] == MATERIAL_EMPTY {
            CellState::Empty
        } else {
            CellState::Matter(density_rank(grid[idx]))
        })
    }

    fn check(
        class: MovementClass,
        source_material: u32,
        x: i64,
        y: i64,
        grid: &[u32],
        width: i64,
        height: i64,
    ) -> MoveTarget {
        propose_move(class, source_material, x, y, |dx, dy| {
            lookup_from(grid, width, height, x, y, dx, dy)
        })
    }

    fn empty_grid(w: i64, h: i64) -> Vec<u32> {
        vec![MATERIAL_EMPTY; (w * h) as usize]
    }

    #[test]
    fn static_never_moves() {
        let grid = empty_grid(8, 8);
        assert_eq!(
            check(MovementClass::Static, MATERIAL_STONE, 4, 4, &grid, 8, 8),
            MoveTarget::NoMove
        );
    }

    #[test]
    fn powder_falls_down() {
        let grid = empty_grid(8, 8);
        assert_eq!(
            check(MovementClass::Powder, MATERIAL_SAND, 4, 4, &grid, 8, 8),
            MoveTarget::Cell(4, 5)
        );
    }

    #[test]
    fn powder_diagonal_when_down_blocked() {
        // Build: cell below (4,5) = Stone; both diagonals EMPTY.
        let mut g = empty_grid(8, 8);
        g[44] = 2; // stone below (4,5)
        let target = check(MovementClass::Powder, MATERIAL_SAND, 4, 4, &g, 8, 8);
        match target {
            MoveTarget::Cell(x, y) => {
                assert_eq!(y, 5);
                assert!(x == 3 || x == 5);
            }
            other => panic!("expected diagonal move, got {other:?}"),
        }
    }

    #[test]
    fn powder_stops_when_fully_blocked() {
        let mut g = empty_grid(8, 8);
        for (dx, dy) in [(0i64, 1i64), (-1i64, 1i64), (1i64, 1i64)] {
            g[((4 + dy) * 8 + (4 + dx)) as usize] = 2; // stone
        }
        assert_eq!(
            check(MovementClass::Powder, MATERIAL_SAND, 4, 4, &g, 8, 8),
            MoveTarget::NoMove
        );
    }

    #[test]
    fn powder_void_when_down_is_out_of_domain() {
        let grid = empty_grid(8, 8);
        // Bottom row: down is outside the world.
        assert_eq!(
            check(MovementClass::Powder, MATERIAL_SAND, 4, 7, &grid, 8, 8),
            MoveTarget::Void
        );
    }

    #[test]
    fn powder_diagonal_oob_is_void() {
        // Powder at the left edge with down blocked: the first-match
        // diagonal (outward) is out-of-domain → Void exit through the open
        // side. The side boundary is not an invisible wall.
        let mut g = empty_grid(8, 8);
        g[8] = 2; // (0,1) stone — blocks down
                  // Parity of (0,0) is even → left diagonal first → (−1,1) is OOB.
        assert_eq!(
            check(MovementClass::Powder, MATERIAL_SAND, 0, 0, &g, 8, 8),
            MoveTarget::Void
        );
    }

    #[test]
    fn liquid_side_diagonal_oob_is_void() {
        // Liquid at the left edge with down and the inward diagonal blocked:
        // the outward diagonal is out-of-domain → Void exit.
        let mut g = empty_grid(8, 8);
        g[8] = 2; // (0,1) stone — blocks down
        g[9] = 2; // (1,1) stone — blocks the inward diagonal
                  // Parity of (0,0) is even → left (outward) diagonal first → OOB.
        assert_eq!(
            check(MovementClass::Liquid, MATERIAL_WATER, 0, 0, &g, 8, 8),
            MoveTarget::Void
        );
    }

    #[test]
    fn liquid_falls_down() {
        let grid = empty_grid(8, 8);
        assert_eq!(
            check(MovementClass::Liquid, MATERIAL_WATER, 4, 4, &grid, 8, 8),
            MoveTarget::Cell(4, 5)
        );
    }

    #[test]
    fn liquid_diagonal_when_down_blocked() {
        let mut g = empty_grid(8, 8);
        g[44] = 2; // stone below (4,5)
        let target = check(MovementClass::Liquid, MATERIAL_WATER, 4, 4, &g, 8, 8);
        match target {
            MoveTarget::Cell(x, y) => {
                assert_eq!(y, 5);
                assert!(x == 3 || x == 5);
            }
            other => panic!("expected diagonal move, got {other:?}"),
        }
    }

    #[test]
    fn liquid_lateral_when_down_and_diagonal_blocked() {
        let mut g = empty_grid(8, 8);
        g[44] = 2; // stone below (4,5)
        g[43] = 2; // down-left stone (3,5)
        g[45] = 2; // down-right stone (5,5)
                   // Laterals are EMPTY.
        let target = check(MovementClass::Liquid, MATERIAL_WATER, 4, 4, &g, 8, 8);
        match target {
            MoveTarget::Cell(x, y) => {
                assert_eq!(y, 4, "lateral move stays on the same row");
                assert!(x == 3 || x == 5, "lateral is 1 cell, got x={x}");
            }
            other => panic!("expected lateral move, got {other:?}"),
        }
    }

    #[test]
    fn liquid_teleports_nothing_and_scans_nothing() {
        // The source liquid at (3,4) is fully boxed in except for a distant
        // EMPTY at (7,4) on the same row. The 1-cell stencil never reaches
        // it: down, both diagonals and both laterals are blocked → NoMove.
        let mut g = empty_grid(8, 8);
        for x in 0..8 {
            if x != 7 {
                g[4 * 8 + x] = 2; // whole row blocked except far right (7)
            }
        }
        g[43] = 2; // down (3,5)
        g[42] = 2; // down-left (2,5)
        g[44] = 2; // down-right (4,5)
        let target = check(MovementClass::Liquid, MATERIAL_WATER, 3, 4, &g, 8, 8);
        assert_eq!(
            target,
            MoveTarget::NoMove,
            "no teleport / scan across the row"
        );
    }

    #[test]
    fn gas_rises_up() {
        let grid = empty_grid(8, 8);
        assert_eq!(
            check(MovementClass::Gas, MATERIAL_STEAM, 4, 4, &grid, 8, 8),
            MoveTarget::Cell(4, 3)
        );
    }

    #[test]
    fn gas_up_diagonal_when_up_blocked() {
        let mut g = empty_grid(8, 8);
        g[28] = 2; // stone above (4,3)
        let target = check(MovementClass::Gas, MATERIAL_STEAM, 4, 4, &g, 8, 8);
        match target {
            MoveTarget::Cell(x, y) => {
                assert_eq!(y, 3);
                assert!(x == 3 || x == 5);
            }
            other => panic!("expected up-diagonal move, got {other:?}"),
        }
    }

    #[test]
    fn gas_void_when_up_is_out_of_domain() {
        let grid = empty_grid(8, 8);
        assert_eq!(
            check(MovementClass::Gas, MATERIAL_STEAM, 4, 0, &grid, 8, 8),
            MoveTarget::Void
        );
    }

    #[test]
    fn gas_up_diagonal_oob_is_void() {
        // Gas at the left edge with up and the inward up-diagonal blocked:
        // the outward up-diagonal is out-of-domain → Void exit.
        let mut g = empty_grid(8, 8);
        g[0] = 2; // (0,0) stone — blocks up
        g[1] = 2; // (1,0) stone — blocks the inward up-diagonal
                  // Parity of (0,1) is odd → right (inward) up-diagonal first, then the
                  // outward (−1,0) one → OOB → Void.
        assert_eq!(
            check(MovementClass::Gas, MATERIAL_STEAM, 0, 1, &g, 8, 8),
            MoveTarget::Void
        );
    }

    #[test]
    fn gas_stable_bulk_has_no_meaningless_swap() {
        // 3×3 steam block. The center cell (4,4) sees steam in its whole
        // 1-cell stencil (up, up-diagonals, laterals): no EMPTY/interface,
        // equal ranks never swap → it must stay.
        let mut g = empty_grid(8, 8);
        for y in 3..=5 {
            for x in 3..=5 {
                g[y * 8 + x] = 6; // steam
            }
        }
        assert_eq!(
            check(MovementClass::Gas, MATERIAL_STEAM, 4, 4, &g, 8, 8),
            MoveTarget::NoMove,
            "bulk interior must not swap with neighbors"
        );
    }

    #[test]
    fn parity_flips_diagonal_preference() {
        // Both sources have their primary (down) direction blocked, so the
        // first-match diagonal is the deciding stage: even parity prefers
        // the left diagonal, odd parity the right one (source-relative).
        let mut g = empty_grid(8, 8);
        g[44] = 2; // (4,5) stone — down of source A
        g[45] = 2; // (5,5) stone — down of source B
        assert_eq!(
            check(MovementClass::Powder, MATERIAL_SAND, 4, 4, &g, 8, 8),
            MoveTarget::Cell(3, 5),
            "even parity must prefer the left diagonal"
        );
        assert_eq!(
            check(MovementClass::Powder, MATERIAL_SAND, 5, 4, &g, 8, 8),
            MoveTarget::Cell(6, 5),
            "odd parity must prefer the right diagonal"
        );
    }

    // ── G3 density displacement ─────────────────────────────────────────

    #[test]
    fn sand_downward_into_water_is_a_swap() {
        let mut g = empty_grid(8, 8);
        g[3 * 8 + 3] = MATERIAL_SAND; // sand source (3,3)
        g[4 * 8 + 3] = MATERIAL_WATER; // water below (3,4)
        assert_eq!(
            check(MovementClass::Powder, MATERIAL_SAND, 3, 3, &g, 8, 8),
            MoveTarget::Swap(3, 4),
            "sand (150) above water (90) must swap downward"
        );
    }

    #[test]
    fn water_downward_into_oil_is_a_swap() {
        let mut g = empty_grid(8, 8);
        g[3 * 8 + 3] = MATERIAL_WATER; // water source (3,3)
        g[4 * 8 + 3] = MATERIAL_OIL; // oil below (3,4)
        assert_eq!(
            check(MovementClass::Liquid, MATERIAL_WATER, 3, 3, &g, 8, 8),
            MoveTarget::Swap(3, 4),
            "water (90) above oil (70) must swap downward"
        );
    }

    #[test]
    fn oil_downward_into_water_is_rejected() {
        let mut g = empty_grid(8, 8);
        g[3 * 8 + 3] = MATERIAL_OIL; // oil source (3,3)
        g[4 * 8 + 3] = MATERIAL_WATER; // water below
                                       // Oil (70) > Water (90) is false → no swap; diagonals EMPTY
                                       // → it slides diagonally, never directly swaps.
        let target = check(MovementClass::Liquid, MATERIAL_OIL, 3, 3, &g, 8, 8);
        match target {
            MoveTarget::Swap(_, _) => panic!("oil must not swap with denser water below"),
            MoveTarget::Cell(2, 4) | MoveTarget::Cell(4, 4) => {}
            other => panic!("expected diagonal slide, got {other:?}"),
        }
    }

    #[test]
    fn steam_upward_into_smoke_is_a_swap() {
        let mut g = empty_grid(8, 8);
        g[3 * 8 + 3] = MATERIAL_SMOKE; // smoke above (3,3)
        g[4 * 8 + 3] = MATERIAL_STEAM; // steam source below (3,4)
        assert_eq!(
            check(MovementClass::Gas, MATERIAL_STEAM, 3, 4, &g, 8, 8),
            MoveTarget::Swap(3, 3),
            "steam (20) below smoke (30) must swap upward"
        );
    }

    #[test]
    fn equal_ranks_never_swap() {
        // Water above water: same rank → no density swap. Fully boxed so the
        // only possible outcome is NoMove (no diagonal/lateral escape).
        let mut g = empty_grid(8, 8);
        g[3 * 8 + 3] = MATERIAL_WATER; // water source (3,3)
        g[4 * 8 + 3] = MATERIAL_WATER; // water below (3,4)
        g[4 * 8 + 2] = 2; // stone down-left
        g[4 * 8 + 4] = 2; // stone down-right
        g[3 * 8 + 2] = 2; // stone left
        g[3 * 8 + 4] = 2; // stone right
        assert_eq!(
            check(MovementClass::Liquid, MATERIAL_WATER, 3, 3, &g, 8, 8),
            MoveTarget::NoMove,
            "equal rank must never swap"
        );
    }

    #[test]
    fn static_targets_never_swap() {
        let mut g = empty_grid(8, 8);
        g[3 * 8 + 3] = MATERIAL_SAND; // sand source (3,3)
        g[4 * 8 + 3] = 2; // stone below
        g[4 * 8 + 2] = 2; // stone down-left
        g[4 * 8 + 4] = 2; // stone down-right
        assert_eq!(
            check(MovementClass::Powder, MATERIAL_SAND, 3, 3, &g, 8, 8),
            MoveTarget::NoMove,
            "STATIC targets are never density-swap targets"
        );
    }

    #[test]
    fn lateral_density_swap_is_rejected() {
        // Oil beside water on the same row, down and diagonals blocked:
        // lateral candidates must stay EMPTY-only — no sideways density swap.
        let mut g = empty_grid(8, 8);
        g[3 * 8 + 3] = MATERIAL_OIL; // oil source (3,3)
        g[3 * 8 + 2] = MATERIAL_WATER; // water left (2,3) — same row
        g[4 * 8 + 3] = 2; // stone down
        g[4 * 8 + 2] = 2; // stone down-left
        g[4 * 8 + 4] = 2; // stone down-right
        let target = check(MovementClass::Liquid, MATERIAL_OIL, 3, 3, &g, 8, 8);
        match target {
            MoveTarget::Swap(_, _) => panic!("lateral density swap is forbidden in G3"),
            MoveTarget::Cell(x, 3) => assert_eq!(x, 4, "oil flows into the empty right cell"),
            other => panic!("expected lateral move, got {other:?}"),
        }
    }

    #[test]
    fn g3_rank_ordering_helpers() {
        assert!(density_displacement_allowed(
            DENSITY_RANK_SAND,
            DENSITY_RANK_WATER,
            DensityDirection::Downward
        ));
        assert!(!density_displacement_allowed(
            DENSITY_RANK_WATER,
            DENSITY_RANK_SAND,
            DensityDirection::Downward
        ));
        assert!(density_displacement_allowed(
            DENSITY_RANK_WATER,
            DENSITY_RANK_OIL,
            DensityDirection::Downward
        ));
        assert!(!density_displacement_allowed(
            DENSITY_RANK_OIL,
            DENSITY_RANK_WATER,
            DensityDirection::Downward
        ));
        assert!(density_displacement_allowed(
            DENSITY_RANK_STEAM,
            DENSITY_RANK_SMOKE,
            DensityDirection::Upward
        ));
        assert!(!density_displacement_allowed(
            DENSITY_RANK_SMOKE,
            DENSITY_RANK_STEAM,
            DensityDirection::Upward
        ));
        assert!(!density_displacement_allowed(
            DENSITY_RANK_WATER,
            DENSITY_RANK_WATER,
            DensityDirection::Downward
        ));
    }
}
