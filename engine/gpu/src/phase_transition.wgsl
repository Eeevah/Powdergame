// G4-B + G5-B — Phase self-transition plus expansion proposal.
//
// The phase identity transform remains Write Self. If the selected
// Material-owned rule requests matter_yield=2, the same invocation also
// writes only proposal[self], choosing at most one local EMPTY target.
// Destination ownership is resolved by the following expansion claim pass.

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
};

struct PhaseDesc {
    below_target: u32,
    above_target: u32,
    below_yield: u32,
    above_yield: u32,
    below_threshold: f32,
    above_threshold: f32,
    below_blocked_pressure: f32,
    above_blocked_pressure: f32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> temperature_current: array<f32>;
@group(0) @binding(3) var<storage, read> phase_table: array<PhaseDesc, 16>;
@group(0) @binding(4) var<storage, read_write> material_next: array<u32>;
@group(0) @binding(5) var<storage, read_write> proposal: array<u32>;

const EMPTY: u32 = 0u;
const NO_PHASE_TARGET: u32 = 0xFFFFFFFFu;
const NO_PROPOSAL: u32 = 0u;
const BLOCKED_EXPANSION: u32 = 0xFFFFFFFFu;
const TEMPERATURE_REFERENCE: f32 = 0.0;

fn sanitize_temperature(t: f32) -> f32 {
    if (t != t) {
        return TEMPERATURE_REFERENCE;
    }
    if (t > 1.0e20 || t < -1.0e20) {
        return TEMPERATURE_REFERENCE;
    }
    return t;
}

fn in_domain(x: i32, y: i32) -> bool {
    return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
}

fn candidate(x: i32, y: i32) -> u32 {
    if (!in_domain(x, y)) {
        return NO_PROPOSAL;
    }
    let idx = u32(y) * params.width + u32(x);
    if (material_current[idx] == EMPTY) {
        return idx + 1u;
    }
    return NO_PROPOSAL;
}

// Local 8-neighbor First-Match. Upward cells are preferred so boiling
// expansion composes naturally with the following GAS movement without
// any long-range scan or special boiler code.
fn find_expansion_target(index: u32) -> u32 {
    let x = i32(index % params.width);
    let y = i32(index / params.width);
    var p = candidate(x, y - 1);
    if (p != NO_PROPOSAL) { return p; }
    p = candidate(x - 1, y - 1);
    if (p != NO_PROPOSAL) { return p; }
    p = candidate(x + 1, y - 1);
    if (p != NO_PROPOSAL) { return p; }
    p = candidate(x - 1, y);
    if (p != NO_PROPOSAL) { return p; }
    p = candidate(x + 1, y);
    if (p != NO_PROPOSAL) { return p; }
    p = candidate(x - 1, y + 1);
    if (p != NO_PROPOSAL) { return p; }
    p = candidate(x + 1, y + 1);
    if (p != NO_PROPOSAL) { return p; }
    p = candidate(x, y + 1);
    if (p != NO_PROPOSAL) { return p; }
    return BLOCKED_EXPANSION;
}

@compute @workgroup_size(64, 1, 1)
fn phase_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) {
        return;
    }

    proposal[index] = NO_PROPOSAL;
    let mat = material_current[index];
    if (mat == EMPTY || mat >= 16u) {
        material_next[index] = mat;
        return;
    }

    let desc = phase_table[mat];
    let t = sanitize_temperature(temperature_current[index]);
    var next_mat = mat;
    var matter_yield = 1u;
    if (desc.below_target != NO_PHASE_TARGET && t < desc.below_threshold) {
        next_mat = desc.below_target;
        matter_yield = desc.below_yield;
    } else if (desc.above_target != NO_PHASE_TARGET && t > desc.above_threshold) {
        next_mat = desc.above_target;
        matter_yield = desc.above_yield;
    }

    material_next[index] = next_mat;
    if (next_mat != mat && matter_yield > 1u) {
        // G5-B baseline supports one additional cell (yield=2). Unknown
        // larger yields fail closed into confinement pressure rather than
        // silently writing multiple neighbors.
        if (matter_yield == 2u) {
            proposal[index] = find_expansion_target(index);
        } else {
            proposal[index] = BLOCKED_EXPANSION;
        }
    }
}
