// G4-C — Smoke spawn claim pass (own WGSL module; no Rust string
// scanning).
//
// Smoke generation acquires ownership of another EMPTY cell, so the
// destination cannot be directly overwritten by the source thread
// (REACTION_SPEC §10 — Spatial Ownership Effect). Each destination
// invocation claims at most ONE winning source.
//
// G6-C2: Stateless edge-hash arbitration replaces fixed-index preference.
// Both endpoints evaluate `edge_priority(source, target_cell, tick)`.
// Lowest priority wins; ties break deterministically by smaller source index.
//
// Reuses the movement `claim` buffer (fully consumed by the movement
// commit pass before the combustion pass rewrote `proposal`). Encoding:
//   claim = (winner_source_index << 2) | SMOKE_DEST_KIND (3)
//   claim = 0 (NO_CLAIM) for non-destinations.
// Kind 3 is unambiguous with the movement kinds (0/1/2), which are already
// resolved and consumed by the time this pass runs.

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
};

struct ArbitrationParams {
    tick: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

const EMPTY: u32 = 0u;
const NO_CLAIM: u32 = 0u;
const SMOKE_DEST_KIND: u32 = 3u;
const NO_SOURCE: u32 = 0xFFFFFFFFu;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> proposal: array<u32>;
@group(0) @binding(3) var<storage, read_write> claim: array<u32>;
@group(0) @binding(4) var<uniform> arbitration: ArbitrationParams;

fn edge_priority(source: u32, target_cell: u32, tick: u32) -> u32 {
    var h: u32 = source ^ (target_cell * 0x9E3779B9u) ^ (tick * 0x85EBCA6Bu);
    h = (h ^ (h >> 16u)) * 0x7FEB352Du;
    h = (h ^ (h >> 15u)) * 0x846CA68Bu;
    h = h ^ (h >> 16u);
    return h;
}

@compute @workgroup_size(64, 1, 1)
fn smoke_claim_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }

    // Only EMPTY cells can receive a Smoke spawn.
    if (material_current[c] != EMPTY) {
        claim[c] = NO_CLAIM;
        return;
    }

    // A burning source proposes at most one 1-cell target, so every source
    // of this destination is within the 8-neighborhood. Smoke proposals are
    // encoded as `target_index + 1` (0 = no spawn).
    let x = i32(c % params.width);
    let y = i32(c / params.width);
    var best_source: u32 = NO_SOURCE;
    var best_priority: u32 = 0xFFFFFFFFu;
    var dy: i32 = -1;
    while (dy <= 1) {
        var dx: i32 = -1;
        while (dx <= 1) {
            if (!(dx == 0 && dy == 0)) {
                let nx = x + dx;
                let ny = y + dy;
                if (nx >= 0 && ny >= 0 && nx < i32(params.width) && ny < i32(params.height)) {
                    let s = u32(ny) * params.width + u32(nx);
                    let p = proposal[s];
                    if (p != 0u && (p - 1u) == c) {
                        let priority = edge_priority(s, c, arbitration.tick);
                        if (priority < best_priority
                            || (priority == best_priority && s < best_source))
                        {
                            best_source = s;
                            best_priority = priority;
                        }
                    }
                }
            }
            dx = dx + 1;
        }
        dy = dy + 1;
    }

    if (best_source == NO_SOURCE) {
        claim[c] = NO_CLAIM;
    } else {
        claim[c] = (best_source << 2u) | SMOKE_DEST_KIND;
    }
}
