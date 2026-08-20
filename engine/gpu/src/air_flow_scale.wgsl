// TE-2 passive Air transport scales. Reuses proposal/claim as f32 scratch.
struct Params {
    cell_count: u32, threads_x: u32, width: u32, height: u32,
    chunk_size: u32, chunks_x: u32, chunks_y: u32, sleep_enabled: u32,
    boundary_mode: u32, _pad0: u32, _pad1: u32, _pad2: u32,
};
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> air_mass_current: array<f32>;
@group(0) @binding(3) var<storage, read> air_energy_current: array<f32>;
@group(0) @binding(4) var<storage, read> chunk_state: array<u32>;
@group(0) @binding(5) var<storage, read_write> donor_scale: array<f32>;
@group(0) @binding(6) var<storage, read_write> receiver_scale: array<f32>;

const EMPTY: u32 = 0u;
const SEALED: u32 = 0u;
const STANDARD_MASS: f32 = 1.0;
const STANDARD_ENERGY: f32 = 293.15;
const AIR_FLOW_RATE: f32 = 0.125;
const AIR_MAX_OUTFLOW_FRACTION: f32 = 0.25;
const AIR_PRESSURE_DEADBAND: f32 = 0.001;
const AIR_FLOW_SCALE_SAFETY: f32 = 0.999999;
const AIR_MASS_MAX: f32 = 16.0;
const AIR_ENERGY_MAX: f32 = 36370.4;

fn finite(v: f32) -> bool { return v == v && abs(v) <= 3.402823e38; }
fn in_domain(x: i32, y: i32) -> bool { return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height); }
fn index_of(x: i32, y: i32) -> u32 { return u32(y) * params.width + u32(x); }
fn chunk_of(index: u32) -> u32 {
    return ((index / params.width) / params.chunk_size) * params.chunks_x
        + ((index % params.width) / params.chunk_size);
}
fn face_enabled(a: u32, b: u32) -> bool {
    return params.sleep_enabled == 0u || chunk_state[chunk_of(a)] == 0u || chunk_state[chunk_of(b)] == 0u;
}
fn pressure(mass: f32, energy: f32) -> f32 {
    return select(0.0, energy / STANDARD_ENERGY, mass > 0.0);
}
fn raw_out(mass_a: f32, energy_a: f32, mass_b: f32, energy_b: f32) -> f32 {
    return AIR_FLOW_RATE * max(pressure(mass_a, energy_a) - pressure(mass_b, energy_b) - AIR_PRESSURE_DEADBAND, 0.0);
}
fn neighbor_state(index: u32, nx: i32, ny: i32) -> vec2<f32> {
    if (!in_domain(nx, ny)) { return select(vec2<f32>(STANDARD_MASS, STANDARD_ENERGY), vec2<f32>(0.0), params.boundary_mode == SEALED); }
    let n = index_of(nx, ny);
    if (material_current[n] != EMPTY || !face_enabled(index, n)) { return vec2<f32>(air_mass_current[index], air_energy_current[index]); }
    return vec2<f32>(air_mass_current[n], air_energy_current[n]);
}
fn raw_in(index: u32, nx: i32, ny: i32) -> vec2<f32> {
    if (!in_domain(nx, ny)) {
        if (params.boundary_mode == SEALED) { return vec2<f32>(0.0); }
        let mass = raw_out(STANDARD_MASS, STANDARD_ENERGY, air_mass_current[index], air_energy_current[index]);
        return vec2<f32>(mass, mass * STANDARD_ENERGY / STANDARD_MASS);
    }
    let n = index_of(nx, ny);
    if (material_current[n] != EMPTY || !face_enabled(index, n)) { return vec2<f32>(0.0); }
    let mass = raw_out(air_mass_current[n], air_energy_current[n], air_mass_current[index], air_energy_current[index]);
    let specific = select(0.0, air_energy_current[n] / air_mass_current[n], air_mass_current[n] > 0.0);
    return vec2<f32>(mass, mass * specific);
}

@compute @workgroup_size(64, 1, 1)
fn air_flow_scale_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) { return; }
    donor_scale[index] = 0.0;
    receiver_scale[index] = 0.0;
    if (material_current[index] != EMPTY) { return; }
    let mass = air_mass_current[index];
    let energy = air_energy_current[index];
    if (!finite(mass) || !finite(energy) || mass < 0.0 || energy < 0.0) { return; }
    let x = i32(index % params.width);
    let y = i32(index / params.width);
    var sum_out = 0.0;
    let left = neighbor_state(index, x - 1, y);
    let right = neighbor_state(index, x + 1, y);
    let up = neighbor_state(index, x, y - 1);
    let down = neighbor_state(index, x, y + 1);
    sum_out += raw_out(mass, energy, left.x, left.y);
    sum_out += raw_out(mass, energy, right.x, right.y);
    sum_out += raw_out(mass, energy, up.x, up.y);
    sum_out += raw_out(mass, energy, down.x, down.y);
    if (sum_out > 0.0) {
        donor_scale[index] = min(1.0, AIR_MAX_OUTFLOW_FRACTION * mass / sum_out) * AIR_FLOW_SCALE_SAFETY;
    }
    var incoming = vec2<f32>(0.0);
    incoming += raw_in(index, x - 1, y);
    incoming += raw_in(index, x + 1, y);
    incoming += raw_in(index, x, y - 1);
    incoming += raw_in(index, x, y + 1);
    var accept = 1.0;
    if (incoming.x > 0.0) { accept = min(accept, (AIR_MASS_MAX - mass) / incoming.x); }
    if (incoming.y > 0.0) { accept = min(accept, (AIR_ENERGY_MAX - energy) / incoming.y); }
    receiver_scale[index] = select(accept, accept * AIR_FLOW_SCALE_SAFETY, accept < 1.0);
}
