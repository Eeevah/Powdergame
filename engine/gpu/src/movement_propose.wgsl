// G3 movement — propose pass (own WGSL module; no Rust string scanning).
//
// Every source Matter reads Current and picks ONE local 1-cell destination
// (First-Match), writing it to `proposal`:
//   - EMPTY destination          → normal move (target index)
//   - STATIC target (rank 0)     → blocked, next candidate
//   - movable + rank ordering OK → local density SWAP candidate (target index;
//                                  the commit exchanges both cells' Matter)
//   - movable + ordering wrong   → blocked (equal ranks never swap)
//   - out-of-domain candidate    → VOID_TARGET for EVERY stencil stage
//                                  (primary, diagonal, lateral) — an open
//                                  side/top/bottom boundary is a Void exit,
//                                  never an invisible wall, never clamped.
//
// Only the gravity-aligned vertical stages may swap:
//   POWDER/LIQUID: down, down-diagonal (source_rank > dest_rank → sinks)
//   GAS:           up,    up-diagonal (source_rank < dest_rank → rises)
// Lateral candidates are EMPTY-only (no sideways density jitter).
//
// Density is a Material table lookup (`density_table[id]`), never per-cell
// state. `class_table[id]` keeps the movement family. The diagnostic marker
// is written by the single index-0 invocation (no atomic, no multi-writer).

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
};

const EMPTY: u32 = 0u;
const NO_MOVE: u32 = 0xFFFFFFFFu;
const VOID_TARGET: u32 = 0xFFFFFFFEu;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read_write> proposal: array<u32>;
@group(0) @binding(3) var<storage, read_write> marker: array<u32>;
@group(0) @binding(4) var<storage, read> class_table: array<u32, 16>;
@group(0) @binding(5) var<storage, read> density_table: array<u32, 16>;

// Candidate kind: 0 = out of domain (Void), 1 = EMPTY, 2 = static/blocked,
// 3 = movable Matter (rank known via candidate_rank).
fn candidate_kind(x: i32, y: i32) -> u32 {
    if (x < 0 || y < 0 || x >= i32(params.width) || y >= i32(params.height)) {
        return 0u;
    }
    let mat = material_current[u32(y) * params.width + u32(x)];
    if (mat == EMPTY) {
        return 1u;
    }
    if (density_table[mat] == 0u) {
        return 2u;
    }
    return 3u;
}

fn candidate_rank(x: i32, y: i32) -> u32 {
    let mat = material_current[u32(y) * params.width + u32(x)];
    return density_table[mat];
}

fn target_index(x: i32, y: i32) -> u32 {
    return u32(y) * params.width + u32(x);
}

// One vertical stencil candidate. `lighter_rises` selects the ordering:
//   false (POWDER/LIQUID down): source_rank > dest_rank → swap (sinks)
//   true  (GAS up):             source_rank < dest_rank → swap (rises)
// Returns a target index, VOID_TARGET, or NO_MOVE (try the next candidate).
fn eval_vertical(x: i32, y: i32, dx: i32, dy: i32, src_rank: u32, lighter_rises: bool) -> u32 {
    let kind = candidate_kind(x + dx, y + dy);
    if (kind == 0u) {
        return VOID_TARGET;
    }
    if (kind == 1u) {
        return target_index(x + dx, y + dy);
    }
    if (kind == 3u) {
        let dest_rank = candidate_rank(x + dx, y + dy);
        if (lighter_rises) {
            if (src_rank < dest_rank) {
                return target_index(x + dx, y + dy);
            }
        } else {
            if (src_rank > dest_rank) {
                return target_index(x + dx, y + dy);
            }
        }
    }
    return NO_MOVE;
}

// One lateral candidate: EMPTY-only (no lateral density swap).
fn eval_lateral(x: i32, y: i32, dx: i32) -> u32 {
    let kind = candidate_kind(x + dx, y);
    if (kind == 0u) {
        return VOID_TARGET;
    }
    if (kind == 1u) {
        return target_index(x + dx, y);
    }
    return NO_MOVE;
}

fn try_diagonals(x: i32, y: i32, dy: i32, src_rank: u32, lighter_rises: bool, parity: u32) -> u32 {
    if (parity == 0u) {
        let a = eval_vertical(x, y, -1, dy, src_rank, lighter_rises);
        if (a != NO_MOVE) {
            return a;
        }
        return eval_vertical(x, y, 1, dy, src_rank, lighter_rises);
    }
    let a = eval_vertical(x, y, 1, dy, src_rank, lighter_rises);
    if (a != NO_MOVE) {
        return a;
    }
    return eval_vertical(x, y, -1, dy, src_rank, lighter_rises);
}

fn try_lateral(x: i32, y: i32, parity: u32) -> u32 {
    if (parity == 0u) {
        let a = eval_lateral(x, y, -1);
        if (a != NO_MOVE) {
            return a;
        }
        return eval_lateral(x, y, 1);
    }
    let a = eval_lateral(x, y, 1);
    if (a != NO_MOVE) {
        return a;
    }
    return eval_lateral(x, y, -1);
}

// POWDER: down → down-diagonal → stop.
fn propose_powder(x: i32, y: i32, src_rank: u32, parity: u32) -> u32 {
    let a = eval_vertical(x, y, 0, 1, src_rank, false);
    if (a != NO_MOVE) {
        return a;
    }
    return try_diagonals(x, y, 1, src_rank, false, parity);
}

// LIQUID: down → down-diagonal → lateral → stop.
fn propose_liquid(x: i32, y: i32, src_rank: u32, parity: u32) -> u32 {
    let a = eval_vertical(x, y, 0, 1, src_rank, false);
    if (a != NO_MOVE) {
        return a;
    }
    let d = try_diagonals(x, y, 1, src_rank, false, parity);
    if (d != NO_MOVE) {
        return d;
    }
    return try_lateral(x, y, parity);
}

// GAS: up → up-diagonal → lateral → stop.
fn propose_gas(x: i32, y: i32, src_rank: u32, parity: u32) -> u32 {
    let a = eval_vertical(x, y, 0, -1, src_rank, true);
    if (a != NO_MOVE) {
        return a;
    }
    let d = try_diagonals(x, y, -1, src_rank, true, parity);
    if (d != NO_MOVE) {
        return d;
    }
    return try_lateral(x, y, parity);
}

@compute
@workgroup_size(64)
fn propose_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) {
        return;
    }
    // Diagnostic: single invocation proves the dispatch executed (no atomic).
    if (index == 0u) {
        marker[0] = 1u;
    }

    let mat = material_current[index];
    if (mat == EMPTY) {
        proposal[index] = NO_MOVE;
        return;
    }
    let cls = class_table[mat];
    if (cls == 0u) {
        proposal[index] = NO_MOVE; // STATIC (and unknown ids)
        return;
    }
    let x = i32(index % params.width);
    let y = i32(index / params.width);
    let parity = (u32(x) + u32(y)) & 1u;
    let src_rank = density_table[mat];
    if (cls == 1u) {
        proposal[index] = propose_powder(x, y, src_rank, parity);
    } else if (cls == 2u) {
        proposal[index] = propose_liquid(x, y, src_rank, parity);
    } else {
        proposal[index] = propose_gas(x, y, src_rank, parity);
    }
}
