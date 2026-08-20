// G3/G4-A/G4-C movement — commit pass (own WGSL module; no Rust string
// scanning).
//
// Each invocation writes ONLY its own `material_next`, `temperature_next`
// and `flags_next` slots (Read Neighbors, Write Self; ADR-0002).
// Temperature and Matter-owned combustion flags (G4-C) are the state of
// the occupying Matter, so they travel on the same ownership edge as the
// material:
//   - stay / unmatched: keep self material + self temp + self flags
//   - agreed edge:      take peer material + peer temp + peer flags
//                       (EMPTY dest → normal move: dest gets source state,
//                        source becomes EMPTY / T=0 / flags=0;
//                        dest has Matter → swap both identities)
//   - Void edge:        source becomes EMPTY / T=0 / flags=0 (no OOB write)
//   - EMPTY next:       temperature is the reference 0.0 and flags are 0
//
// flags[] contract: flags[] is a Matter-owned field holding state bits
// attached to the occupying Matter (EMPTY flags are always 0). Spatial /
// cell-owned state (e.g. Pressure, G5) must NOT be packed into flags —
// future spatial booleans use a separate field, and Pressure itself is
// deliberately NOT transported here; only material / temperature / flags
// are Matter-owned.
//
// Claim encoding (see movement_claim.wgsl):
//   claim = (peer_index << 2) | kind   (0 = NO_CLAIM, 1 = SOURCE, 2 = DEST)

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
    chunk_size: u32,
    chunks_x: u32,
    chunks_y: u32,
    sleep_enabled: u32,
};

const EMPTY: u32 = 0u;
const NO_CLAIM: u32 = 0u;
const KIND_SOURCE: u32 = 1u;
const KIND_DEST: u32 = 2u;
const VOID_PEER: u32 = 0x3FFFFFFFu;
const TEMPERATURE_REFERENCE: f32 = 20.0;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> claim: array<u32>;
@group(0) @binding(3) var<storage, read_write> material_next: array<u32>;
@group(0) @binding(4) var<storage, read> temperature_current: array<f32>;
@group(0) @binding(5) var<storage, read_write> temperature_next: array<f32>;
@group(0) @binding(6) var<storage, read> flags_current: array<u32>;
@group(0) @binding(7) var<storage, read_write> flags_next: array<u32>;
@group(0) @binding(8) var<storage, read> chunk_state: array<u32>;

fn commit_occupancy(c: u32, mat: u32, temp: f32, flags: u32) {
    material_next[c] = mat;
    if (mat == EMPTY) {
        temperature_next[c] = TEMPERATURE_REFERENCE;
        flags_next[c] = 0u;
    } else {
        temperature_next[c] = temp;
        flags_next[c] = flags;
    }
}

@compute
@workgroup_size(64)
fn commit_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }

    if (params.sleep_enabled != 0u) {
        let cx = (c % params.width) / params.chunk_size;
        let cy = (c / params.width) / params.chunk_size;
        if (chunk_state[cy * params.chunks_x + cx] != 0u) {
            commit_occupancy(
                c,
                material_current[c],
                temperature_current[c],
                flags_current[c],
            );
            return;
        }
    }

    let my_claim = claim[c];
    let kind = my_claim & 3u;
    let peer = my_claim >> 2u;

    if (kind == NO_CLAIM) {
        commit_occupancy(
            c,
            material_current[c],
            temperature_current[c],
            flags_current[c],
        );
        return;
    }

    if (kind == KIND_SOURCE) {
        if (peer == VOID_PEER) {
            // Void exit: Matter, its heat and its combustion state leave
            // the world.
            commit_occupancy(c, EMPTY, TEMPERATURE_REFERENCE, 0u);
            return;
        }
        let peer_claim = claim[peer];
        if ((peer_claim & 3u) == KIND_DEST && (peer_claim >> 2u) == c) {
            commit_occupancy(
                c,
                material_current[peer],
                temperature_current[peer],
                flags_current[peer],
            );
        } else {
            commit_occupancy(
                c,
                material_current[c],
                temperature_current[c],
                flags_current[c],
            );
        }
        return;
    }

    let peer_claim = claim[peer];
    if ((peer_claim & 3u) == KIND_SOURCE && (peer_claim >> 2u) == c) {
        commit_occupancy(
            c,
            material_current[peer],
            temperature_current[peer],
            flags_current[peer],
        );
    } else {
        commit_occupancy(
            c,
            material_current[c],
            temperature_current[c],
            flags_current[c],
        );
    }
}
