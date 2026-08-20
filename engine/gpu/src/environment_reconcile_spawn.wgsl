// TE-1 paired spawn Environment commit. Exactly eight storage bindings.
struct Params {
    cell_count: u32, threads_x: u32, width: u32, height: u32,
    chunk_size: u32, chunks_x: u32, chunks_y: u32, sleep_enabled: u32,
};
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> material_next: array<u32>;
@group(0) @binding(3) var<storage, read> matter_claim: array<u32>;
@group(0) @binding(4) var<storage, read> environment_receiver_claim: array<u32>;
@group(0) @binding(5) var<storage, read> air_mass_current: array<f32>;
@group(0) @binding(6) var<storage, read> air_energy_current: array<f32>;
@group(0) @binding(7) var<storage, read_write> air_mass_next: array<f32>;
@group(0) @binding(8) var<storage, read_write> air_energy_next: array<f32>;
const EMPTY: u32 = 0u;
const SMOKE_DEST_KIND: u32 = 3u;

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
fn in_domain(x: i32, y: i32) -> bool {
    return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
}
fn claim_matches(target_cell: u32, smoke: bool) -> bool {
    let claim = matter_claim[target_cell];
    if (smoke) { return (claim & 3u) == SMOKE_DEST_KIND; }
    return claim != 0u;
}
fn has_receiver(target_cell: u32) -> bool {
    let x = i32(target_cell % params.width);
    let y = i32(target_cell / params.width);
    var direction = 0u;
    while (direction < 4u) {
        let rx = x + direction_x(direction);
        let ry = y + direction_y(direction);
        if (in_domain(rx, ry)) {
            let receiver = u32(ry) * params.width + u32(rx);
            if (environment_receiver_claim[receiver] == target_cell + 1u) { return true; }
        }
        direction += 1u;
    }
    return false;
}
fn reconcile(c: u32, smoke: bool) {
    if (material_next[c] != EMPTY) {
        air_mass_next[c] = 0.0; air_energy_next[c] = 0.0; return;
    }
    let receiver_value = environment_receiver_claim[c];
    if (receiver_value != 0u) {
        let target_cell = receiver_value - 1u;
        if (target_cell < params.cell_count && material_current[target_cell] == EMPTY
            && material_next[target_cell] != EMPTY && claim_matches(target_cell, smoke)
            && has_receiver(target_cell)) {
            air_mass_next[c] = air_mass_current[c] + air_mass_current[target_cell];
            air_energy_next[c] = air_energy_current[c] + air_energy_current[target_cell];
            return;
        }
    }
    if (material_current[c] == EMPTY) {
        air_mass_next[c] = air_mass_current[c];
        air_energy_next[c] = air_energy_current[c];
    } else {
        air_mass_next[c] = 0.0; air_energy_next[c] = 0.0;
    }
}
@compute @workgroup_size(64)
fn expansion_environment_reconcile_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c < params.cell_count) { reconcile(c, false); }
}
@compute @workgroup_size(64)
fn smoke_environment_reconcile_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c < params.cell_count) { reconcile(c, true); }
}
