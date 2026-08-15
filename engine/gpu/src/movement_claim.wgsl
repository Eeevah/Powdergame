// G3 movement — claim pass (own WGSL module; no Rust string scanning).
//
// Ownership-changing movements are treated as **edges** (source → target).
// An edge executes only when BOTH endpoints select it:
//   - the source endpoint claims its own edge,
//   - the destination endpoint claims the same edge back.
//
// Each cell keeps ONE incident edge (or none). A cell that is both a source
// and a destination (chains like A→B→C, or mutual swaps) picks exactly one
// incident edge, so no cell can join two ownership-changing moves in one
// tick. Fixed, deterministic arbitration: the edge with the smallest source
// index wins (no per-cell RNG; DETERMINISM_SPEC §4).
//
// Encoding (per-cell u32 scratch — ownership arbitration state, never
// Matter and never density state):
//   claim = (peer_index << 2) | kind
//   kind: 0 = NO_CLAIM, 1 = SOURCE end, 2 = DEST end
//   peer_index: the other endpoint. VOID_PEER (a sentinel that can never be
//   a real index) marks a Void-exit edge, which has no destination endpoint
//   and therefore executes unconditionally.
//
// A source whose proposal lost arbitration simply keeps its own edge claim;
// the destination does not reciprocate, so the commit pass drops it — the
// loser stays valid at its source, nothing is lost or duplicated.

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
};

const NO_MOVE: u32 = 0xFFFFFFFFu;
const VOID_TARGET: u32 = 0xFFFFFFFEu;
const NO_CLAIM: u32 = 0u;
const KIND_SOURCE: u32 = 1u;
const KIND_DEST: u32 = 2u;
const VOID_PEER: u32 = 0x3FFFFFFFu;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> proposal: array<u32>;
@group(0) @binding(2) var<storage, read_write> claim: array<u32>;

@compute
@workgroup_size(64)
fn claim_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }

    // Void-exit edge: the destination endpoint does not exist inside the
    // world, so the source edge executes unconditionally.
    let t = proposal[c];
    if (t == VOID_TARGET) {
        claim[c] = (VOID_PEER << 2u) | KIND_SOURCE;
        return;
    }

    var best: u32 = NO_CLAIM;
    var best_owner: u32 = 0xFFFFFFFFu;

    // This cell's own source edge (c → t).
    if (t != NO_MOVE) {
        best = (t << 2u) | KIND_SOURCE;
        best_owner = c;
    }

    // Edges where this cell is the destination (a 1-cell neighbor proposes
    // this cell). Fixed arbitration: the smallest source index wins.
    let x = i32(c % params.width);
    let y = i32(c / params.width);
    var dy: i32 = -1;
    while (dy <= 1) {
        var dx: i32 = -1;
        while (dx <= 1) {
            if (!(dx == 0 && dy == 0)) {
                let nx = x + dx;
                let ny = y + dy;
                if (nx >= 0 && ny >= 0 && nx < i32(params.width) && ny < i32(params.height)) {
                    let s = u32(ny) * params.width + u32(nx);
                    if (proposal[s] == c && s < best_owner) {
                        best = (s << 2u) | KIND_DEST;
                        best_owner = s;
                    }
                }
            }
            dx = dx + 1;
        }
        dy = dy + 1;
    }

    claim[c] = best;
}
