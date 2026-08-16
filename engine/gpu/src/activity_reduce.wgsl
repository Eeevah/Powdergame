// G7-A activity — chunk reduction pass (own WGSL module).
//
// One invocation per chunk (workgroup_size(1,1,1); no atomics, no
// workgroup memory) OR-reduces the per-cell activity flags of its
// chunk_size² cells into `chunk_activity[chunk]`, then self-writes the
// tick diagnostics:
//
//   chunk_activity[chunk]        — OR of all cell activity bits
//   chunk_changed_this_tick[...] — 1 if the chunk has any activity this
//                                  tick, else 0 (measurement of "frontier
//                                  present this tick")
//   chunk_stable_ticks[...]      — consecutive ticks with zero activity
//                                  (saturating); any activity resets to 0.
//
// Stable ticks are an OBSERVATION baseline only — no sleep cutoff is
// applied (G7-B decides thresholds from measurements).

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
    chunk_size: u32,
    chunks_x: u32,
    chunks_y: u32,
    thermal_eps: f32,
    pressure_eps: f32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

const SATURATION: u32 = 4294967295u;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> cell_activity: array<u32>;
@group(0) @binding(2) var<storage, read_write> chunk_activity: array<u32>;
@group(0) @binding(3) var<storage, read_write> chunk_changed_this_tick: array<u32>;
@group(0) @binding(4) var<storage, read_write> chunk_stable_ticks: array<u32>;

@compute
@workgroup_size(1, 1, 1)
fn reduce_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cx = gid.x;
    let cy = gid.y;
    if (cx >= params.chunks_x || cy >= params.chunks_y) {
        return;
    }
    let chunk_idx = cy * params.chunks_x + cx;

    var mask = 0u;
    for (var ly = 0u; ly < params.chunk_size; ly++) {
        for (var lx = 0u; lx < params.chunk_size; lx++) {
            let gx = cx * params.chunk_size + lx;
            let gy = cy * params.chunk_size + ly;
            if (gx >= params.width || gy >= params.height) {
                continue;
            }
            let idx = gy * params.width + gx;
            mask = mask | cell_activity[idx];
        }
    }

    chunk_activity[chunk_idx] = mask;
    chunk_changed_this_tick[chunk_idx] = select(0u, 1u, mask != 0u);

    let old = chunk_stable_ticks[chunk_idx];
    if (mask == 0u) {
        if (old >= SATURATION) {
            chunk_stable_ticks[chunk_idx] = SATURATION;
        } else {
            chunk_stable_ticks[chunk_idx] = old + 1u;
        }
    } else {
        chunk_stable_ticks[chunk_idx] = 0u;
    }
}
