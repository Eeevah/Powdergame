// G3 movement — commit pass (own WGSL module; no Rust string scanning).
//
// Each invocation writes ONLY its own `material_next` slot (Read Neighbors,
// Write Self; ADR-0002). A selected, mutually-agreed edge exchanges Matter
// with the peer:
//   - source endpoint:    material_next[source] = material_current[dest]
//     (dest is EMPTY → normal move; dest has Matter → swap)
//   - destination endpoint: material_next[dest] = material_current[source]
//   - Void edge:          material_next[source] = EMPTY (leaves the world;
//                         nothing is written outside the buffer)
//   - unmatched edge / NO_CLAIM: keep material_current (loser stays valid,
//     no duplication, no unexplained loss)
//
// Claim encoding (see movement_claim.wgsl):
//   claim = (peer_index << 2) | kind   (0 = NO_CLAIM, 1 = SOURCE, 2 = DEST)

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
};

const EMPTY: u32 = 0u;
const NO_CLAIM: u32 = 0u;
const KIND_SOURCE: u32 = 1u;
const KIND_DEST: u32 = 2u;
const VOID_PEER: u32 = 0x3FFFFFFFu;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> claim: array<u32>;
@group(0) @binding(3) var<storage, read_write> material_next: array<u32>;

@compute
@workgroup_size(64)
fn commit_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }

    let my_claim = claim[c];
    let kind = my_claim & 3u;
    let peer = my_claim >> 2u;

    if (kind == NO_CLAIM) {
        material_next[c] = material_current[c];
        return;
    }

    if (kind == KIND_SOURCE) {
        if (peer == VOID_PEER) {
            // Void exit: Matter leaves the world through an open boundary.
            material_next[c] = EMPTY;
            return;
        }
        // The destination must reciprocate this exact edge.
        let peer_claim = claim[peer];
        if ((peer_claim & 3u) == KIND_DEST && (peer_claim >> 2u) == c) {
            material_next[c] = material_current[peer];
        } else {
            material_next[c] = material_current[c]; // unmatched: stay
        }
        return;
    }

    // KIND_DEST: the source must reciprocate this exact edge.
    let peer_claim = claim[peer];
    if ((peer_claim & 3u) == KIND_SOURCE && (peer_claim >> 2u) == c) {
        material_next[c] = material_current[peer];
    } else {
        material_next[c] = material_current[c]; // unmatched: stay
    }
}
