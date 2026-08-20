// TE-1 self identity/decay/consumption/rupture occupancy hygiene.
struct Params {
    cell_count: u32, threads_x: u32, width: u32, height: u32,
    chunk_size: u32, chunks_x: u32, chunks_y: u32, sleep_enabled: u32,
};
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> material_next: array<u32>;
@group(0) @binding(3) var<storage, read> air_mass_current: array<f32>;
@group(0) @binding(4) var<storage, read> air_energy_current: array<f32>;
@group(0) @binding(5) var<storage, read_write> air_mass_next: array<f32>;
@group(0) @binding(6) var<storage, read_write> air_energy_next: array<f32>;
const EMPTY: u32 = 0u;
@compute @workgroup_size(64)
fn identity_environment_reconcile_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) { return; }
    if (material_next[c] != EMPTY) {
        air_mass_next[c] = 0.0; air_energy_next[c] = 0.0;
    } else if (material_current[c] != EMPTY) {
        // Physical Matter removal exposes Vacuum; TE-2 may refill it later.
        air_mass_next[c] = 0.0; air_energy_next[c] = 0.0;
    } else {
        air_mass_next[c] = air_mass_current[c];
        air_energy_next[c] = air_energy_current[c];
    }
}
