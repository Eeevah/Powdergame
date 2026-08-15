// G0 tick shader — runtime plumbing only.
//
// No gameplay semantics: this pass copies `material_id` from the Current
// half to the Next half over the full world and sets a diagnostic marker.
// It proves the compute pipeline, world buffers and full-world dispatch
// work headlessly. Gameplay rules start in G1+.
//
// The world is dispatched as a 2D grid (WORKGROUPS_X x ceil(cell_count /
// THREADS_X)) because DX12 limits a single dispatch dimension to 65535 and
// the reference world needs 65536 workgroups of 64 threads.

struct Params {
    cell_count: u32,
    threads_x: u32, // WORKGROUPS_X * workgroup_size
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read_write> material_next: array<u32>;
@group(0) @binding(3) var<storage, read_write> marker: array<u32>;

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) {
        return;
    }
    material_next[index] = material_current[index];
    // Diagnostic only: prove this dispatch actually executed on the GPU.
    marker[0] = 1u;
}
