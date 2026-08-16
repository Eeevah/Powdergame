// G5-B — phase expansion destination Claim/Resolve.
//
// G6-C2: Each EMPTY destination reads only its 8-neighborhood and chooses
// the source index targeting this cell with the lowest edge_priority.
// Ties break deterministically by smaller source index. claim[c]=source+1.

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

struct ArbitrationParams {
    tick: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> proposal: array<u32>;
@group(0) @binding(3) var<storage, read_write> claim: array<u32>;
@group(0) @binding(4) var<uniform> arbitration: ArbitrationParams;
@group(0) @binding(5) var<storage, read> chunk_state: array<u32>;

const EMPTY: u32 = 0u;
const NO_CLAIM: u32 = 0u;
const NO_SOURCE: u32 = 0xFFFFFFFFu;

fn edge_priority(source: u32, target_cell: u32, tick: u32) -> u32 {
    var h: u32 = source ^ (target_cell * 0x9E3779B9u) ^ (tick * 0x85EBCA6Bu);
    h = (h ^ (h >> 16u)) * 0x7FEB352Du;
    h = (h ^ (h >> 15u)) * 0x846CA68Bu;
    h = h ^ (h >> 16u);
    return h;
}

@compute @workgroup_size(64, 1, 1)
fn expansion_claim_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }
    claim[c] = NO_CLAIM;

    if (params.sleep_enabled != 0u) {
        let cx = (c % params.width) / params.chunk_size;
        let cy = (c / params.width) / params.chunk_size;
        if (chunk_state[cy * params.chunks_x + cx] != 0u) {
            return;
        }
    }

    if (material_current[c] != EMPTY) {
        return;
    }

    let x = i32(c % params.width);
    let y = i32(c / params.width);
    var best_source = NO_SOURCE;
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
                    if (proposal[s] == c + 1u) {
                        let p = edge_priority(s, c, arbitration.tick);
                        if (p < best_priority || (p == best_priority && s < best_source)) {
                            best_source = s;
                            best_priority = p;
                        }
                    }
                }
            }
            dx = dx + 1;
        }
        dy = dy + 1;
    }

    if (best_source != NO_SOURCE) {
        claim[c] = best_source + 1u;
    }
}
