// G5-B — phase expansion destination Claim/Resolve.
// Each EMPTY destination reads only its 8-neighborhood and chooses the
// smallest source index whose proposal targets this cell. claim[c]=source+1.

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> proposal: array<u32>;
@group(0) @binding(3) var<storage, read_write> claim: array<u32>;

const EMPTY: u32 = 0u;
const NO_CLAIM: u32 = 0u;
const NO_SOURCE: u32 = 0xFFFFFFFFu;

@compute @workgroup_size(64, 1, 1)
fn expansion_claim_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }
    claim[c] = NO_CLAIM;
    if (material_current[c] != EMPTY) {
        return;
    }

    let x = i32(c % params.width);
    let y = i32(c / params.width);
    var best = NO_SOURCE;
    var dy: i32 = -1;
    while (dy <= 1) {
        var dx: i32 = -1;
        while (dx <= 1) {
            if (!(dx == 0 && dy == 0)) {
                let nx = x + dx;
                let ny = y + dy;
                if (nx >= 0 && ny >= 0 && nx < i32(params.width) && ny < i32(params.height)) {
                    let s = u32(ny) * params.width + u32(nx);
                    if (proposal[s] == c + 1u && s < best) {
                        best = s;
                    }
                }
            }
            dx = dx + 1;
        }
        dy = dy + 1;
    }

    if (best != NO_SOURCE) {
        claim[c] = best + 1u;
    }
}
