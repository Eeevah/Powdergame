// TE-1 phase expansion whose Matter target won but whose Environment receiver
// failed receives the existing blocked-expansion pressure source exactly once.
struct Params {
    cell_count: u32, threads_x: u32, width: u32, height: u32,
    chunk_size: u32, chunks_x: u32, chunks_y: u32, sleep_enabled: u32,
};
struct PhaseDesc {
    below_target: u32, above_target: u32, below_yield: u32, above_yield: u32,
    below_threshold: f32, above_threshold: f32,
    below_blocked_pressure: f32, above_blocked_pressure: f32,
};
struct PhaseEffect { enabled: u32, matter_yield: u32, blocked_pressure: f32 };
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> temperature_current: array<f32>;
@group(0) @binding(3) var<storage, read> phase_table: array<PhaseDesc, 16>;
@group(0) @binding(4) var<storage, read> proposal: array<u32>;
@group(0) @binding(5) var<storage, read> matter_claim: array<u32>;
@group(0) @binding(6) var<storage, read> environment_receiver_claim: array<u32>;
@group(0) @binding(7) var<storage, read_write> pressure_next: array<f32>;
const EMPTY: u32 = 0u;
const NO_PHASE_TARGET: u32 = 0xFFFFFFFFu;
const BLOCKED_EXPANSION: u32 = 0xFFFFFFFFu;
const PRESSURE_MAX: f32 = 1.0e6;

fn selected_effect(mat: u32, t: f32) -> PhaseEffect {
    var effect = PhaseEffect(0u, 1u, 0.0);
    if (mat == EMPTY || mat >= 16u) { return effect; }
    let desc = phase_table[mat];
    if (desc.below_target != NO_PHASE_TARGET && t < desc.below_threshold) {
        return PhaseEffect(1u, desc.below_yield, desc.below_blocked_pressure);
    }
    if (desc.above_target != NO_PHASE_TARGET && t > desc.above_threshold) {
        return PhaseEffect(1u, desc.above_yield, desc.above_blocked_pressure);
    }
    return effect;
}
fn in_domain(x: i32, y: i32) -> bool {
    return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
}
fn has_receiver(target_cell: u32) -> bool {
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
@compute @workgroup_size(64)
fn environment_blocked_expansion_pressure_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) { return; }
    let request = proposal[c];
    if (request == 0u || request == BLOCKED_EXPANSION) { return; }
    let target_cell = request - 1u;
    if (target_cell >= params.cell_count || matter_claim[target_cell] != c + 1u || has_receiver(target_cell)) { return; }
    let effect = selected_effect(material_current[c], temperature_current[c]);
    if (effect.enabled != 0u && effect.matter_yield > 1u) {
        let p = pressure_next[c];
        let base = select(0.0, p, p == p && abs(p) <= 3.402823e38);
        pressure_next[c] = clamp(base + max(effect.blocked_pressure, 0.0), 0.0, PRESSURE_MAX);
    }
}
