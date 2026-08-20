// TE-1 movement Volume Exchange. Reads Current Matter/Air and the settled
// Matter result, then self-writes both Air Next buffers.
struct Params {
    cell_count: u32, threads_x: u32, width: u32, height: u32,
    chunk_size: u32, chunks_x: u32, chunks_y: u32, sleep_enabled: u32,
};
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> material_next: array<u32>;
@group(0) @binding(3) var<storage, read> claim: array<u32>;
@group(0) @binding(4) var<storage, read> air_mass_current: array<f32>;
@group(0) @binding(5) var<storage, read> air_energy_current: array<f32>;
@group(0) @binding(6) var<storage, read_write> air_mass_next: array<f32>;
@group(0) @binding(7) var<storage, read_write> air_energy_next: array<f32>;
const EMPTY: u32 = 0u;
const KIND_SOURCE: u32 = 1u;
const KIND_DEST: u32 = 2u;
const VOID_PEER: u32 = 0x3FFFFFFFu;

fn write_air(c: u32, mass: f32, energy: f32) {
    air_mass_next[c] = mass; air_energy_next[c] = energy;
}
@compute @workgroup_size(64)
fn movement_environment_reconcile_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) { return; }
    if (material_next[c] != EMPTY) { write_air(c, 0.0, 0.0); return; }
    if (material_current[c] == EMPTY) {
        write_air(c, air_mass_current[c], air_energy_current[c]); return;
    }
    let my_claim = claim[c];
    if ((my_claim & 3u) == KIND_SOURCE) {
        let peer = my_claim >> 2u;
        if (peer != VOID_PEER && peer < params.cell_count
            && material_current[peer] == EMPTY) {
            let peer_claim = claim[peer];
            if ((peer_claim & 3u) == KIND_DEST && (peer_claim >> 2u) == c) {
                write_air(c, air_mass_current[peer], air_energy_current[peer]); return;
            }
        }
    }
    write_air(c, 0.0, 0.0);
}
