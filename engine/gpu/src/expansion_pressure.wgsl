// G5-B — unresolved phase expansion becomes scalar Pressure at source.
// Successful claims add no pressure. Blocked requests and claim losers
// receive the Material-owned blocked_pressure impulse. Write Self only.

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

struct PhaseEffect {
    enabled: u32,
    matter_yield: u32,
    blocked_pressure: f32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> temperature_current: array<f32>;
@group(0) @binding(3) var<storage, read> phase_table: array<PhaseDesc, 16>;
@group(0) @binding(4) var<storage, read> proposal: array<u32>;
@group(0) @binding(5) var<storage, read> claim: array<u32>;
@group(0) @binding(6) var<storage, read> pressure_current: array<f32>;
@group(0) @binding(7) var<storage, read_write> pressure_next: array<f32>;

const EMPTY: u32 = 0u;
const NO_PHASE_TARGET: u32 = 0xFFFFFFFFu;
const NO_PROPOSAL: u32 = 0u;
const BLOCKED_EXPANSION: u32 = 0xFFFFFFFFu;
const PRESSURE_REFERENCE: f32 = 0.0;
const PRESSURE_MAX: f32 = 1.0e6;

fn sanitize_temperature(t: f32) -> f32 {
    if (t != t || t > 1.0e20 || t < -1.0e20) {
        return 0.0;
    }
    return t;
}

fn sanitize_pressure(p: f32) -> f32 {
    if (p != p || p > 1.0e20 || p < -1.0e20) {
        return PRESSURE_REFERENCE;
    }
    return clamp(p, PRESSURE_REFERENCE, PRESSURE_MAX);
}

fn selected_effect(mat: u32, t: f32) -> PhaseEffect {
    var effect = PhaseEffect(0u, 1u, 0.0);
    if (mat == EMPTY || mat >= 16u) {
        return effect;
    }
    let desc = phase_table[mat];
    if (desc.below_target != NO_PHASE_TARGET && t < desc.below_threshold) {
        effect.enabled = 1u;
        effect.matter_yield = desc.below_yield;
        effect.blocked_pressure = desc.below_blocked_pressure;
    } else if (desc.above_target != NO_PHASE_TARGET && t > desc.above_threshold) {
        effect.enabled = 1u;
        effect.matter_yield = desc.above_yield;
        effect.blocked_pressure = desc.above_blocked_pressure;
    }
    return effect;
}

@compute @workgroup_size(64, 1, 1)
fn expansion_pressure_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }

    let p0 = sanitize_pressure(pressure_current[c]);
    let effect = selected_effect(material_current[c], sanitize_temperature(temperature_current[c]));
    var impulse = 0.0;

    if (effect.enabled != 0u && effect.matter_yield > 1u) {
        let request = proposal[c];
        var succeeded = false;
        if (request != NO_PROPOSAL && request != BLOCKED_EXPANSION) {
            let destination = request - 1u;
            if (destination < params.cell_count && claim[destination] == c + 1u) {
                succeeded = true;
            }
        }
        if (!succeeded) {
            impulse = max(effect.blocked_pressure, 0.0);
        }
    }

    pressure_next[c] = sanitize_pressure(p0 + impulse);
}
