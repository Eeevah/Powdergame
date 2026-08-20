// G4-C — Smoke spawn commit pass (own WGSL module; no Rust string
// scanning).
//
// Each invocation writes ONLY its own `material_next[self]` (and, for a
// spawn destination, its own `temperature_next[self]`). No neighbor writes.
//
//   claim kind == SMOKE_DEST (3): this cell won a Smoke spawn from
//     `source = claim >> 2`. Write MATERIAL_SMOKE and carry the burning
//     source's post-combustion temperature (new Smoke T = burning source T,
//     a cheap finite thermal derivation). flags_next was already cleared
//     for this EMPTY cell by the combustion pass (Smoke is not burning).
//   otherwise: preserve material (the phase result already settled into
//     material_current), EXCEPT cells the combustion pass consumed this
//     tick — consumed fuel wrote EMPTY to material_next[self], which must
//     not be clobbered back to the pre-combustion material.
//
// The source's temperature_next was finalized by the combustion pass
// before this pass runs (sequential dispatches), and no two destinations
// can claim the same source (each source proposes one target), so element
// `source` is only ever read here.

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
const SMOKE_MATERIAL: u32 = 7u;
const NO_CLAIM: u32 = 0u;
const SMOKE_DEST_KIND: u32 = 3u;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> claim: array<u32>;
@group(0) @binding(3) var<storage, read_write> temperature_next: array<f32>;
@group(0) @binding(4) var<storage, read_write> material_next: array<u32>;
@group(0) @binding(5) var<storage, read> chunk_state: array<u32>;
@group(0) @binding(6) var<storage, read> environment_receiver_claim: array<u32>;

fn in_domain(x: i32, y: i32) -> bool {
    return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
}

fn has_environment_receiver(target_cell: u32) -> bool {
    let x = i32(target_cell % params.width);
    let y = i32(target_cell / params.width);
    let offsets = array<vec2<i32>, 4>(vec2<i32>(0,-1), vec2<i32>(1,0), vec2<i32>(0,1), vec2<i32>(-1,0));
    var i = 0u;
    while (i < 4u) {
        let p = vec2<i32>(x, y) + offsets[i];
        if (in_domain(p.x, p.y)) {
            let receiver = u32(p.y) * params.width + u32(p.x);
            if (environment_receiver_claim[receiver] == target_cell + 1u) { return true; }
        }
        i += 1u;
    }
    return false;
}

@compute @workgroup_size(64, 1, 1)
fn smoke_commit_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }

    if (params.sleep_enabled != 0u) {
        let cx = (c % params.width) / params.chunk_size;
        let cy = (c / params.width) / params.chunk_size;
        if (chunk_state[cy * params.chunks_x + cx] != 0u) {
            return;
        }
    }

    let my_claim = claim[c];
    if ((my_claim & 3u) == SMOKE_DEST_KIND && has_environment_receiver(c)) {
        let source = my_claim >> 2u;
        material_next[c] = SMOKE_MATERIAL;
        temperature_next[c] = temperature_next[source];
        return;
    }

    // Consumed fuel: the combustion pass already wrote EMPTY (and reset
    // temperature/flags) into this cell's next slots this tick — never
    // resurrect it. A phase result is never EMPTY for a Matter cell, so
    // (material_current != EMPTY && material_next == EMPTY) uniquely
    // identifies a fuel-consumed cell.
    if (material_current[c] != EMPTY && material_next[c] == EMPTY) {
        return;
    }

    // Non-destination: the phase result (or the unchanged source Matter)
    // is preserved. temperature_next / flags_next are untouched.
    material_next[c] = material_current[c];
}
