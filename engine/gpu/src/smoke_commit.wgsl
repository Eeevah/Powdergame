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
//   otherwise: preserve material (the source stays Wood/Oil; phase result
//     already settled into material_current).
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

@compute @workgroup_size(64, 1, 1)
fn smoke_commit_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }

    let my_claim = claim[c];
    if ((my_claim & 3u) == SMOKE_DEST_KIND) {
        let source = my_claim >> 2u;
        material_next[c] = SMOKE_MATERIAL;
        temperature_next[c] = temperature_next[source];
        return;
    }

    // Non-destination: the phase result (or the unchanged source Matter)
    // is preserved. temperature_next / flags_next are untouched.
    material_next[c] = material_current[c];
}
