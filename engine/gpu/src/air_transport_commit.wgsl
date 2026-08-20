// TE-2 conservative passive Air transport. Each invocation writes self only.
struct Params {
    cell_count: u32, threads_x: u32, width: u32, height: u32,
    chunk_size: u32, chunks_x: u32, chunks_y: u32, sleep_enabled: u32,
    boundary_mode: u32, _pad0: u32, _pad1: u32, _pad2: u32,
};
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> air_mass_current: array<f32>;
@group(0) @binding(3) var<storage, read> air_energy_current: array<f32>;
@group(0) @binding(4) var<storage, read> donor_scale: array<f32>;
@group(0) @binding(5) var<storage, read> receiver_scale: array<f32>;
@group(0) @binding(6) var<storage, read_write> air_mass_next: array<f32>;
@group(0) @binding(7) var<storage, read_write> air_energy_next: array<f32>;
@group(0) @binding(8) var<storage, read> chunk_state: array<u32>;
const EMPTY: u32 = 0u;
const SEALED: u32 = 0u;
const STANDARD_MASS: f32 = 1.0;
const STANDARD_ENERGY: f32 = 293.15;
const RATE: f32 = 0.125;
const DEADBAND: f32 = 0.001;
fn in_domain(x: i32, y: i32) -> bool { return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height); }
fn index_of(x: i32, y: i32) -> u32 { return u32(y) * params.width + u32(x); }
fn chunk_of(index: u32) -> u32 { return ((index / params.width) / params.chunk_size) * params.chunks_x + ((index % params.width) / params.chunk_size); }
fn face_enabled(a:u32,b:u32)->bool{return params.sleep_enabled==0u||chunk_state[chunk_of(a)]==0u||chunk_state[chunk_of(b)]==0u;}
fn pressure(mass:f32,energy:f32)->f32{return select(0.0,energy/STANDARD_ENERGY,mass>0.0);}
fn raw(am:f32,ae:f32,bm:f32,be:f32)->f32{return RATE*max(pressure(am,ae)-pressure(bm,be)-DEADBAND,0.0);}
fn transfer(index: u32, nx: i32, ny: i32) -> vec2<f32> {
    let self_mass = air_mass_current[index];
    let self_specific = select(0.0, air_energy_current[index] / self_mass, self_mass > 0.0);
    if (!in_domain(nx, ny)) {
        if (params.boundary_mode == SEALED) { return vec2<f32>(0.0); }
        let outgoing = raw(self_mass, air_energy_current[index], STANDARD_MASS, STANDARD_ENERGY) * donor_scale[index];
        let incoming = raw(STANDARD_MASS, STANDARD_ENERGY, self_mass, air_energy_current[index]) * receiver_scale[index];
        return vec2<f32>(incoming - outgoing, incoming * STANDARD_ENERGY - outgoing * self_specific);
    }
    let n = index_of(nx, ny);
    if (material_current[n] != EMPTY || !face_enabled(index,n)) { return vec2<f32>(0.0); }
    let n_mass = air_mass_current[n];
    let n_specific = select(0.0, air_energy_current[n] / n_mass, n_mass > 0.0);
    let outgoing = raw(self_mass, air_energy_current[index], n_mass, air_energy_current[n]) * min(donor_scale[index], receiver_scale[n]);
    let incoming = raw(n_mass, air_energy_current[n], self_mass, air_energy_current[index]) * min(donor_scale[n], receiver_scale[index]);
    return vec2<f32>(incoming - outgoing, incoming * n_specific - outgoing * self_specific);
}
@compute @workgroup_size(64, 1, 1)
fn air_transport_commit_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) { return; }
    if (material_current[index] != EMPTY) { air_mass_next[index] = 0.0; air_energy_next[index] = 0.0; return; }
    let x = i32(index % params.width); let y = i32(index / params.width);
    var delta = vec2<f32>(0.0);
    delta += transfer(index, x - 1, y); delta += transfer(index, x + 1, y);
    delta += transfer(index, x, y - 1); delta += transfer(index, x, y + 1);
    air_mass_next[index] = air_mass_current[index] + delta.x;
    air_energy_next[index] = air_energy_current[index] + delta.y;
}
