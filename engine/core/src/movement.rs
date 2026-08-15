//! Movement candidate selection — pure/reference logic.
//!
//! Purpose (`SIMULATION_SPEC` §7/§11, `DEVELOPMENT.md` §4 Step 3):
//! unit tests, algorithm explanation and semantic comparison against the GPU
//! production path. The production 2048×2048 world is never simulated on the
//! CPU; this module only defines the behavior contract.
//!
//! G2 contract:
//! - movement reads **Current** state only,
//! - only **1-cell local** neighbors are considered (no teleport, no scan),
//! - **First-Match**: the first valid candidate wins and searching stops,
//! - destinations are `EMPTY` only (density displacement is G3),
//! - an out-of-domain position is `Void` for **every** stencil candidate —
//!   primary, diagonal or lateral. An open side/top/bottom boundary is a
//!   Void exit, never an invisible wall, never clamped, never treated as an
//!   EMPTY cell.

use crate::material::MovementClass;

/// State of a candidate cell as seen by a mover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    /// In-domain cell with no Matter.
    Empty,
    /// In-domain cell occupied by Matter (or not a valid destination).
    Blocked,
}

/// Where a mover proposes to go this tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveTarget {
    /// Stay in place (no valid candidate / STATIC).
    NoMove,
    /// Move to this in-domain 1-cell neighbor.
    Cell(i64, i64),
    /// Leave the world through an open boundary (source becomes EMPTY).
    Void,
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
/// `lookup(dx, dy)` returns the state of the in-domain cell at `(x+dx, y+dy)`:
/// - `Some(Empty)` — valid destination,
/// - `Some(Blocked)` — occupied, not a destination; try the next candidate,
/// - `None` — out-of-domain (Void). For EVERY stencil candidate this is a
///   `Void` exit: the mover leaves the world there. It is never an invisible
///   wall and never clamped.
pub fn propose_move(
    class: MovementClass,
    x: i64,
    y: i64,
    lookup: impl Fn(i64, i64) -> Option<CellState>,
) -> MoveTarget {
    match class {
        MovementClass::Static => MoveTarget::NoMove,
        MovementClass::Powder => {
            // down → down-diagonal → stop
            match lookup(0, 1) {
                None => return MoveTarget::Void,
                Some(CellState::Empty) => return MoveTarget::Cell(x, y + 1),
                Some(CellState::Blocked) => {}
            }
            try_diagonals(x, y, 1, prefer_left(x, y), &lookup).unwrap_or(MoveTarget::NoMove)
        }
        MovementClass::Liquid => {
            // down → down-diagonal → lateral → stop
            match lookup(0, 1) {
                None => return MoveTarget::Void,
                Some(CellState::Empty) => return MoveTarget::Cell(x, y + 1),
                Some(CellState::Blocked) => {}
            }
            if let Some(target) = try_diagonals(x, y, 1, prefer_left(x, y), &lookup) {
                return target;
            }
            try_lateral(x, y, prefer_left(x, y), &lookup).unwrap_or(MoveTarget::NoMove)
        }
        MovementClass::Gas => {
            // up → up-diagonal → lateral → stop
            match lookup(0, -1) {
                None => return MoveTarget::Void,
                Some(CellState::Empty) => return MoveTarget::Cell(x, y - 1),
                Some(CellState::Blocked) => {}
            }
            if let Some(target) = try_diagonals(x, y, -1, prefer_left(x, y), &lookup) {
                return target;
            }
            try_lateral(x, y, prefer_left(x, y), &lookup).unwrap_or(MoveTarget::NoMove)
        }
    }
}

/// First-match diagonal candidates (±1 in `dy` direction), ordered by parity.
///
/// Out-of-domain candidates are `MoveTarget::Void` exits (open side/top/
/// bottom boundaries are not invisible walls); in-domain occupied cells just
/// fall through to the next candidate. Returns `None` only when every
/// candidate was an in-domain blocked cell.
fn try_diagonals(
    x: i64,
    y: i64,
    dy: i64,
    left_first: bool,
    lookup: &impl Fn(i64, i64) -> Option<CellState>,
) -> Option<MoveTarget> {
    let (l, r) = if left_first {
        ((-1, dy), (1, dy))
    } else {
        ((1, dy), (-1, dy))
    };
    for (dx, ddy) in [l, r] {
        match lookup(dx, ddy) {
            None => return Some(MoveTarget::Void),
            Some(CellState::Empty) => return Some(MoveTarget::Cell(x + dx, y + ddy)),
            Some(CellState::Blocked) => {}
        }
    }
    None
}

/// First-match lateral candidate (one cell), ordered by parity.
///
/// Same Void semantics as [`try_diagonals`]: an out-of-domain lateral is a
/// Void exit, not a wall. Returns `None` only when every candidate was an
/// in-domain blocked cell.
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
            Some(CellState::Blocked) => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::MATERIAL_EMPTY;

    /// Builds a lookup over a small grid; `None` for out-of-bounds.
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
            CellState::Blocked
        })
    }

    fn check(
        class: MovementClass,
        x: i64,
        y: i64,
        grid: &[u32],
        width: i64,
        height: i64,
    ) -> MoveTarget {
        propose_move(class, x, y, |dx, dy| {
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
            check(MovementClass::Static, 4, 4, &grid, 8, 8),
            MoveTarget::NoMove
        );
    }

    #[test]
    fn powder_falls_down() {
        let grid = empty_grid(8, 8);
        assert_eq!(
            check(MovementClass::Powder, 4, 4, &grid, 8, 8),
            MoveTarget::Cell(4, 5)
        );
    }

    #[test]
    fn powder_diagonal_when_down_blocked() {
        // Build: cell below (4,5) = Stone; both diagonals EMPTY.
        let mut g = empty_grid(8, 8);
        g[44] = 2; // stone below (4,5)
        let target = check(MovementClass::Powder, 4, 4, &g, 8, 8);
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
            check(MovementClass::Powder, 4, 4, &g, 8, 8),
            MoveTarget::NoMove
        );
    }

    #[test]
    fn powder_void_when_down_is_out_of_domain() {
        let grid = empty_grid(8, 8);
        // Bottom row: down is outside the world.
        assert_eq!(
            check(MovementClass::Powder, 4, 7, &grid, 8, 8),
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
            check(MovementClass::Powder, 0, 0, &g, 8, 8),
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
            check(MovementClass::Liquid, 0, 0, &g, 8, 8),
            MoveTarget::Void
        );
    }

    #[test]
    fn liquid_falls_down() {
        let grid = empty_grid(8, 8);
        assert_eq!(
            check(MovementClass::Liquid, 4, 4, &grid, 8, 8),
            MoveTarget::Cell(4, 5)
        );
    }

    #[test]
    fn liquid_diagonal_when_down_blocked() {
        let mut g = empty_grid(8, 8);
        g[44] = 2; // stone below (4,5)
        let target = check(MovementClass::Liquid, 4, 4, &g, 8, 8);
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
        let target = check(MovementClass::Liquid, 4, 4, &g, 8, 8);
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
        let target = check(MovementClass::Liquid, 3, 4, &g, 8, 8);
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
            check(MovementClass::Gas, 4, 4, &grid, 8, 8),
            MoveTarget::Cell(4, 3)
        );
    }

    #[test]
    fn gas_up_diagonal_when_up_blocked() {
        let mut g = empty_grid(8, 8);
        g[28] = 2; // stone above (4,3)
        let target = check(MovementClass::Gas, 4, 4, &g, 8, 8);
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
            check(MovementClass::Gas, 4, 0, &grid, 8, 8),
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
        assert_eq!(check(MovementClass::Gas, 0, 1, &g, 8, 8), MoveTarget::Void);
    }

    #[test]
    fn gas_stable_bulk_has_no_meaningless_swap() {
        // 3×3 steam block. The center cell (4,4) sees steam in its whole
        // 1-cell stencil (up, up-diagonals, laterals): no EMPTY/interface,
        // so it must stay — no Gas↔Gas swap.
        let mut g = empty_grid(8, 8);
        for y in 3..=5 {
            for x in 3..=5 {
                g[y * 8 + x] = 6; // steam
            }
        }
        assert_eq!(
            check(MovementClass::Gas, 4, 4, &g, 8, 8),
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
            check(MovementClass::Powder, 4, 4, &g, 8, 8),
            MoveTarget::Cell(3, 5),
            "even parity must prefer the left diagonal"
        );
        assert_eq!(
            check(MovementClass::Powder, 5, 4, &g, 8, 8),
            MoveTarget::Cell(6, 5),
            "odd parity must prefer the right diagonal"
        );
    }
}
