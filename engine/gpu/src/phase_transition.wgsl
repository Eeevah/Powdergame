// G4-B — Phase transition. Temperature-based 1:1 SELF transition
// (REACTION_SPEC §3): the decision depends only on this cell's own
// Material + Temperature, so each invocation writes ONLY
// `material_next[self]`. No Claim/Resolve, no atomics, no neighbor
// writes. Multi-cell expansion / yield / Pressure are G5 (out of scope).
//
// Rules are compiled into the per-Material phase table (Material data →
// generic GPU table; no material-name branches). First-Match: the
// `below` rule is tested before the `above` rule, matching Water's
// ordered rule set (freeze before boil). `NO_PHASE_TARGET` is the safe
// sentinel — it is never confused with EMPTY (0).
//
// Temperature is preserved across the 1:1 transform (latent heat is out
// of scope). EMPTY is not a phase candidate and keeps writing itself.

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
};

struct PhaseDesc {
    below_target: u32,
    above_target: u32,
    below_threshold: f32,
    above_threshold: f32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> temperature_current: array<f32>;
@group(0) @binding(3) var<storage, read> phase_table: array<PhaseDesc, 16>;
@group(0) @binding(4) var<storage, read_write> material_next: array<u32>;

const EMPTY: u32 = 0u;
const NO_PHASE_TARGET: u32 = 0xFFFFFFFFu;
const TEMPERATURE_REFERENCE: f32 = 0.0;

fn sanitize(t: f32) -> f32 {
    if (t != t) {
        return TEMPERATURE_REFERENCE;
    }
    if (t > 1.0e20 || t < -1.0e20) {
        return TEMPERATURE_REFERENCE;
    }
    return t;
}

@compute @workgroup_size(64, 1, 1)
fn phase_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) {
        return;
    }

    let mat = material_current[index];
    if (mat == EMPTY || mat >= 16u) {
        material_next[index] = mat;
        return;
    }

    let desc = phase_table[mat];
    let t = sanitize(temperature_current[index]);
    var next_mat = mat;
    if (desc.below_target != NO_PHASE_TARGET && t < desc.below_threshold) {
        next_mat = desc.below_target;
    } else if (desc.above_target != NO_PHASE_TARGET && t > desc.above_threshold) {
        next_mat = desc.above_target;
    }
    material_next[index] = next_mat;
}
