// TE-1 deterministic Environment receiver arbitration for phase/Smoke spawns.
// Encoding: environment_receiver_claim[receiver] = target + 1, 0 = none.

struct Params {
    cell_count: u32, threads_x: u32, width: u32, height: u32,
    chunk_size: u32, chunks_x: u32, chunks_y: u32, sleep_enabled: u32,
};
struct ArbitrationParams { tick: u32, _pad0: u32, _pad1: u32, _pad2: u32 };

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> matter_claim: array<u32>;
@group(0) @binding(3) var<storage, read> air_mass_current: array<f32>;
@group(0) @binding(4) var<storage, read> air_energy_current: array<f32>;
@group(0) @binding(5) var<storage, read_write> environment_receiver_claim: array<u32>;
@group(0) @binding(6) var<uniform> arbitration: ArbitrationParams;

const EMPTY: u32 = 0u;
const SMOKE_DEST_KIND: u32 = 3u;
const AIR_MASS_MAX: f32 = 16.0;
const AIR_ENERGY_MAX: f32 = 36370.4;
const AIR_TEMPERATURE_MIN: f32 = 1.0;
const AIR_TEMPERATURE_MAX: f32 = 2273.15;

fn in_domain(x: i32, y: i32) -> bool {
    return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
}
fn index_of(x: i32, y: i32) -> u32 { return u32(y) * params.width + u32(x); }
fn is_target(c: u32, smoke: bool) -> bool {
    if (material_current[c] != EMPTY) { return false; }
    let claim = matter_claim[c];
    if (smoke) { return (claim & 3u) == SMOKE_DEST_KIND; }
    return claim != 0u;
}
fn finite_nonnegative(v: f32) -> bool {
    return v == v && v >= 0.0 && v <= 3.402823e38;
}
fn valid_air(mass: f32, energy: f32) -> bool {
    if (!finite_nonnegative(mass) || !finite_nonnegative(energy)) { return false; }
    if (mass == 0.0 || energy == 0.0) { return mass == 0.0 && energy == 0.0; }
    if (mass > AIR_MASS_MAX || energy > AIR_ENERGY_MAX) { return false; }
    let temperature = energy / mass;
    return temperature == temperature
        && temperature >= AIR_TEMPERATURE_MIN
        && temperature <= AIR_TEMPERATURE_MAX;
}
fn receiver_eligible(c: u32, target_cell: u32, smoke: bool) -> bool {
    if (material_current[c] != EMPTY || is_target(c, smoke)) { return false; }
    let rm = air_mass_current[c];
    let re = air_energy_current[c];
    let pm = air_mass_current[target_cell];
    let pe = air_energy_current[target_cell];
    let combined_mass = rm + pm;
    let combined_energy = re + pe;
    return valid_air(rm, re) && valid_air(pm, pe)
        && combined_mass <= AIR_MASS_MAX && combined_energy <= AIR_ENERGY_MAX
        && valid_air(combined_mass, combined_energy);
}
fn direction_x(direction: u32) -> i32 {
    if (direction == 1u) { return 1; }
    if (direction == 3u) { return -1; }
    return 0;
}
fn direction_y(direction: u32) -> i32 {
    if (direction == 0u) { return -1; }
    if (direction == 2u) { return 1; }
    return 0;
}
fn preferred_receiver(target_cell: u32, smoke: bool) -> u32 {
    let tx = i32(target_cell % params.width);
    let ty = i32(target_cell / params.width);
    var h = target_cell ^ (arbitration.tick * 0x9E3779B9u);
    h = (h ^ (h >> 16u)) * 0x7FEB352Du;
    let first = h & 3u;
    var step = 0u;
    while (step < 4u) {
        let direction = (first + step) & 3u;
        let x = tx + direction_x(direction);
        let y = ty + direction_y(direction);
        if (in_domain(x, y)) {
            let candidate = index_of(x, y);
            if (receiver_eligible(candidate, target_cell, smoke)) { return candidate + 1u; }
        }
        step += 1u;
    }
    return 0u;
}
fn receiver_claim(c: u32, smoke: bool) {
    environment_receiver_claim[c] = 0u;
    if (material_current[c] != EMPTY || is_target(c, smoke)) { return; }
    let x = i32(c % params.width);
    let y = i32(c / params.width);
    var best = 0xFFFFFFFFu;
    var direction = 0u;
    while (direction < 4u) {
        let tx = x + direction_x(direction);
        let ty = y + direction_y(direction);
        if (in_domain(tx, ty)) {
            let target_cell = index_of(tx, ty);
            if (is_target(target_cell, smoke)
                && preferred_receiver(target_cell, smoke) == c + 1u
                && target_cell < best) {
                best = target_cell;
            }
        }
        direction += 1u;
    }
    if (best != 0xFFFFFFFFu) { environment_receiver_claim[c] = best + 1u; }
}

@compute @workgroup_size(64)
fn expansion_receiver_claim_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c < params.cell_count) { receiver_claim(c, false); }
}

@compute @workgroup_size(64)
fn smoke_receiver_claim_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c < params.cell_count) { receiver_claim(c, true); }
}
