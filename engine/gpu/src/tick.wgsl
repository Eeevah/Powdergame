// G2 movement tick shader — multi-pass local movement.
//
// Passes (each is a full-world compute dispatch; pass boundaries act as
// barriers):
//   1. propose  — every source reads Current, picks ONE local 1-cell
//                 destination (First-Match), writes it to `proposal`.
//   2. resolve  — every EMPTY cell that was claimed picks exactly one
//                 winner source (fixed arbitration: smallest source index).
//   3. commit   — every cell writes only its OWN material_next slot:
//                 claimed destinations receive the winner's Matter; sources
//                 that won move (become EMPTY); losers stay; Void dies.
//
// No gameplay rule beyond movement. Destinations are EMPTY only (density
// displacement is G3). ANY out-of-domain stencil candidate (primary,
// diagonal or lateral) is a Void exit — never an invisible wall, never
// clamped, never treated as an EMPTY cell.

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
};

const EMPTY: u32 = 0u;
const NO_MOVE: u32 = 0xFFFFFFFFu;
const VOID_TARGET: u32 = 0xFFFFFFFEu;
const NO_SOURCE: u32 = 0xFFFFFFFFu;

// ── propose ─────────────────────────────────────────────────────────────

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read_write> proposal: array<u32>;
@group(0) @binding(3) var<storage, read_write> marker: array<u32>;
@group(0) @binding(4) var<storage, read> class_table: array<u32, 16>;

// State of a candidate cell: 0 = out of domain (Void), 1 = EMPTY, 2 = blocked.
fn cell_state(x: i32, y: i32) -> u32 {
    if (x < 0 || y < 0 || x >= i32(params.width) || y >= i32(params.height)) {
        return 0u;
    }
    let mat = material_current[u32(y) * params.width + u32(x)];
    if (mat == EMPTY) {
        return 1u;
    }
    return 2u;
}

fn target_index(x: i32, y: i32) -> u32 {
    return u32(y) * params.width + u32(x);
}

fn try_diagonal(x: i32, y: i32, dy: i32, parity: u32) -> u32 {
    // First-match, ordered by cheap stateless parity (no RNG state).
    // Out-of-domain candidates are Void exits (open side/top/bottom
    // boundaries are not invisible walls).
    if (parity == 0u) {
        let s = cell_state(x - 1, y + dy);
        if (s == 0u) {
            return VOID_TARGET;
        }
        if (s == 1u) {
            return target_index(x - 1, y + dy);
        }
        let s2 = cell_state(x + 1, y + dy);
        if (s2 == 0u) {
            return VOID_TARGET;
        }
        if (s2 == 1u) {
            return target_index(x + 1, y + dy);
        }
    } else {
        let s = cell_state(x + 1, y + dy);
        if (s == 0u) {
            return VOID_TARGET;
        }
        if (s == 1u) {
            return target_index(x + 1, y + dy);
        }
        let s2 = cell_state(x - 1, y + dy);
        if (s2 == 0u) {
            return VOID_TARGET;
        }
        if (s2 == 1u) {
            return target_index(x - 1, y + dy);
        }
    }
    return NO_MOVE;
}

fn try_lateral(x: i32, y: i32, parity: u32) -> u32 {
    if (parity == 0u) {
        let s = cell_state(x - 1, y);
        if (s == 0u) {
            return VOID_TARGET;
        }
        if (s == 1u) {
            return target_index(x - 1, y);
        }
        let s2 = cell_state(x + 1, y);
        if (s2 == 0u) {
            return VOID_TARGET;
        }
        if (s2 == 1u) {
            return target_index(x + 1, y);
        }
    } else {
        let s = cell_state(x + 1, y);
        if (s == 0u) {
            return VOID_TARGET;
        }
        if (s == 1u) {
            return target_index(x + 1, y);
        }
        let s2 = cell_state(x - 1, y);
        if (s2 == 0u) {
            return VOID_TARGET;
        }
        if (s2 == 1u) {
            return target_index(x - 1, y);
        }
    }
    return NO_MOVE;
}

// POWDER: down -> down-diagonal -> stop. Down out-of-domain = Void.
fn propose_powder(x: i32, y: i32, parity: u32) -> u32 {
    let down = cell_state(x, y + 1);
    if (down == 0u) {
        return VOID_TARGET;
    }
    if (down == 1u) {
        return target_index(x, y + 1);
    }
    let d = try_diagonal(x, y, 1, parity);
    if (d != NO_MOVE) {
        return d;
    }
    return NO_MOVE;
}

// LIQUID: down -> down-diagonal -> lateral (1 cell only) -> stop.
fn propose_liquid(x: i32, y: i32, parity: u32) -> u32 {
    let down = cell_state(x, y + 1);
    if (down == 0u) {
        return VOID_TARGET;
    }
    if (down == 1u) {
        return target_index(x, y + 1);
    }
    let d = try_diagonal(x, y, 1, parity);
    if (d != NO_MOVE) {
        return d;
    }
    let l = try_lateral(x, y, parity);
    if (l != NO_MOVE) {
        return l;
    }
    return NO_MOVE;
}

// GAS: up -> up-diagonal -> lateral -> stop. No meaningless swap in a
// stable same-Matter bulk: the stencil only finds EMPTY/interface cells.
fn propose_gas(x: i32, y: i32, parity: u32) -> u32 {
    let up = cell_state(x, y - 1);
    if (up == 0u) {
        return VOID_TARGET;
    }
    if (up == 1u) {
        return target_index(x, y - 1);
    }
    let d = try_diagonal(x, y, -1, parity);
    if (d != NO_MOVE) {
        return d;
    }
    let l = try_lateral(x, y, parity);
    if (l != NO_MOVE) {
        return l;
    }
    return NO_MOVE;
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
    if (cls == 1u) {
        proposal[index] = propose_powder(x, y, parity);
    } else if (cls == 2u) {
        proposal[index] = propose_liquid(x, y, parity);
    } else {
        proposal[index] = propose_gas(x, y, parity);
    }
}

// ── resolve ────────────────────────────────────────────────────────────

@group(0) @binding(0) var<uniform> params_r: Params;
@group(0) @binding(1) var<storage, read> material_current_r: array<u32>;
@group(0) @binding(2) var<storage, read> proposal_r: array<u32>;
@group(0) @binding(3) var<storage, read_write> resolve: array<u32>;
@group(0) @binding(4) var<storage, read> class_table_r: array<u32, 16>;

@compute
@workgroup_size(64)
fn resolve_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let d = gid.y * params_r.threads_x + gid.x;
    if (d >= params_r.cell_count) {
        return;
    }
    resolve[d] = NO_SOURCE;
    // Only EMPTY cells can be claimed as destinations this tick.
    if (material_current_r[d] != EMPTY) {
        return;
    }
    let x = i32(d % params_r.width);
    let y = i32(d / params_r.width);

    // Look at the 1-cell neighborhood: did any source propose `d`?
    // Fixed arbitration: the smallest source index wins (deterministic,
    // exactly one winner, no per-cell RNG).
    var winner: u32 = NO_SOURCE;
    var dy: i32 = -1;
    while (dy <= 1) {
        var dx: i32 = -1;
        while (dx <= 1) {
            if (!(dx == 0 && dy == 0)) {
                let nx = x + dx;
                let ny = y + dy;
                if (nx >= 0 && ny >= 0 && nx < i32(params_r.width) && ny < i32(params_r.height)) {
                    let s = u32(ny) * params_r.width + u32(nx);
                    if (proposal_r[s] == d && (winner == NO_SOURCE || s < winner)) {
                        winner = s;
                    }
                }
            }
            dx = dx + 1;
        }
        dy = dy + 1;
    }
    resolve[d] = winner;
}

// ── commit ─────────────────────────────────────────────────────────────

@group(0) @binding(0) var<uniform> params_c: Params;
@group(0) @binding(1) var<storage, read> material_current_c: array<u32>;
@group(0) @binding(2) var<storage, read> proposal_c: array<u32>;
@group(0) @binding(3) var<storage, read> resolve_c: array<u32>;
@group(0) @binding(4) var<storage, read_write> material_next: array<u32>;
@group(0) @binding(5) var<storage, read> class_table_c: array<u32, 16>;

@compute
@workgroup_size(64)
fn commit_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.y * params_c.threads_x + gid.x;
    if (i >= params_c.cell_count) {
        return;
    }

    // Each cell writes only its OWN material_next slot (Read Neighbors,
    // Write Self; ownership changes go through resolve).

    // 1. Claimed destination: receive the winner's Matter.
    let winner_of_i = resolve_c[i];
    if (winner_of_i != NO_SOURCE) {
        material_next[i] = material_current_c[winner_of_i];
        return;
    }

    // 2. Not a destination: apply this cell's own movement proposal.
    let dest = proposal_c[i];
    if (dest == NO_MOVE) {
        material_next[i] = material_current_c[i];
        return;
    }
    if (dest == VOID_TARGET) {
        material_next[i] = EMPTY; // left the world through an open boundary
        return;
    }
    if (resolve_c[dest] == i) {
        material_next[i] = EMPTY; // won the destination claim: moved away
    } else {
        material_next[i] = material_current_c[i]; // lost the claim: stay
    }
}
