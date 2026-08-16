// G5-B — winning expansion destination commits one extra Matter cell.
// Reads the source's already-computed phase result and writes only self.

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
    chunk_size: u32,
    chunks_x: u32,
    chunks_y: u32,
    sleep_enabled: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> temperature_current: array<f32>;
@group(0) @binding(3) var<storage, read> claim: array<u32>;
@group(0) @binding(4) var<storage, read_write> material_next: array<u32>;
@group(0) @binding(5) var<storage, read_write> temperature_next: array<f32>;
@group(0) @binding(6) var<storage, read_write> flags_next: array<u32>;
@group(0) @binding(7) var<storage, read> chunk_state: array<u32>;

const EMPTY: u32 = 0u;

@compute @workgroup_size(64, 1, 1)
fn expansion_spawn_commit_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }

    if (params.sleep_enabled != 0u) {
        let cx = (c % params.width) / params.chunk_size;
        let cy = (c / params.width) / params.chunk_size;
        if (chunk_state[cy * params.chunks_x + cx] != 0u) {
            return;
        }
    }
    let winner = claim[c];
    if (winner == 0u || material_current[c] != EMPTY) {
        return;
    }
    let source = winner - 1u;
    material_next[c] = material_next[source];
    temperature_next[c] = temperature_current[source];
    flags_next[c] = 0u;
}
