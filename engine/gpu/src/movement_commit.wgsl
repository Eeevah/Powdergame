// G3/G4-A movement — commit pass (own WGSL module; no Rust string scanning).
//
// Each invocation writes ONLY its own `material_next` and
// `temperature_next` slots (Read Neighbors, Write Self; ADR-0002).
// Temperature is the thermal state of the occupying Matter, so it travels
// on the same ownership edge as the material:
//   - stay / unmatched: keep self material + self temp
//   - agreed edge:      take peer material + peer temp
//                       (EMPTY dest → normal move: dest gets source T,
//                        source becomes EMPTY / T=0;
//                        dest has Matter → swap both identities)
//   - Void edge:        source becomes EMPTY / T=0 (no OOB write)
//   - EMPTY next:       temperature is always the reference 0.0
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
const TEMPERATURE_REFERENCE: f32 = 0.0;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> claim: array<u32>;
@group(0) @binding(3) var<storage, read_write> material_next: array<u32>;
@group(0) @binding(4) var<storage, read> temperature_current: array<f32>;
@group(0) @binding(5) var<storage, read_write> temperature_next: array<f32>;

fn commit_occupancy(c: u32, mat: u32, temp: f32) {
    material_next[c] = mat;
    if (mat == EMPTY) {
        temperature_next[c] = TEMPERATURE_REFERENCE;
    } else {
        temperature_next[c] = temp;
    }
}

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
        commit_occupancy(c, material_current[c], temperature_current[c]);
        return;
    }

    if (kind == KIND_SOURCE) {
        if (peer == VOID_PEER) {
            // Void exit: Matter and its heat leave the world.
            commit_occupancy(c, EMPTY, TEMPERATURE_REFERENCE);
            return;
        }
        let peer_claim = claim[peer];
        if ((peer_claim & 3u) == KIND_DEST && (peer_claim >> 2u) == c) {
            commit_occupancy(c, material_current[peer], temperature_current[peer]);
        } else {
            commit_occupancy(c, material_current[c], temperature_current[c]);
        }
        return;
    }

    let peer_claim = claim[peer];
    if ((peer_claim & 3u) == KIND_SOURCE && (peer_claim >> 2u) == c) {
        commit_occupancy(c, material_current[peer], temperature_current[peer]);
    } else {
        commit_occupancy(c, material_current[c], temperature_current[c]);
    }
}
