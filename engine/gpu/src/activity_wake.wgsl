// G7-B — Chunk Run/Sleep State & Safety Halo Evaluation pass.
//
// One invocation per chunk (workgroup_size(1,1,1); no atomics, no workgroup memory).
// Evaluates whether each chunk is RUNNABLE or SLEEPING for the upcoming tick:
//
// Wake conditions:
//   1. Sleep mode disabled (Always-Active reference mode) → RUNNABLE (reason: ALWAYS_ACTIVE)
//   2. Self activity != 0 (MATTER, THERMAL, PRESSURE, REACTION) → RUNNABLE (reason: SELF_ACTIVITY)
//   3. User edit trigger (chunk_edit_wake[chunk] != 0) → RUNNABLE (reason: USER_EDIT)
//   4. Neighbor halo: any of the 8 neighbors (dx: -1..1, dy: -1..1) has activity != 0
//      or edit wake != 0 → RUNNABLE (reason: NEIGHBOR_HALO)
//   5. Settling period: consecutive stable ticks < sleep_threshold → RUNNABLE (reason: SETTLING)
//
// If none of the wake conditions hold:
//   chunk_state[chunk] = CHUNK_STATE_SLEEPING (1)
//   chunk_wake_reason[chunk] = WAKE_REASON_NONE (0)
//
// During wake dispatch, chunk_edit_wake is an immutable snapshot (read-only).
// The edit wake buffer is cleared after the wake pass completes by the command encoder.

struct Params {
    chunks_x: u32,
    chunks_y: u32,
    sleep_enabled: u32,
    sleep_threshold: u32,
};

const CHUNK_STATE_RUNNABLE: u32 = 0u;
const CHUNK_STATE_SLEEPING: u32 = 1u;

const WAKE_REASON_NONE: u32 = 0u;
const WAKE_REASON_SELF_ACTIVITY: u32 = 1u;       // 1 << 0
const WAKE_REASON_NEIGHBOR_HALO: u32 = 2u;       // 1 << 1
const WAKE_REASON_USER_EDIT: u32 = 4u;           // 1 << 2
const WAKE_REASON_SETTLING: u32 = 8u;            // 1 << 3
const WAKE_REASON_ALWAYS_ACTIVE: u32 = 16u;      // 1 << 4

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> chunk_activity: array<u32>;
@group(0) @binding(2) var<storage, read> chunk_stable_ticks: array<u32>;
@group(0) @binding(3) var<storage, read> chunk_edit_wake: array<u32>;
@group(0) @binding(4) var<storage, read_write> chunk_state: array<u32>;
@group(0) @binding(5) var<storage, read_write> chunk_wake_reason: array<u32>;

@compute
@workgroup_size(1, 1, 1)
fn wake_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cx = gid.x;
    let cy = gid.y;
    if (cx >= params.chunks_x || cy >= params.chunks_y) {
        return;
    }
    let chunk_idx = cy * params.chunks_x + cx;

    if (params.sleep_enabled == 0u) {
        chunk_state[chunk_idx] = CHUNK_STATE_RUNNABLE;
        chunk_wake_reason[chunk_idx] = WAKE_REASON_ALWAYS_ACTIVE;
        return;
    }

    let edit_wake = chunk_edit_wake[chunk_idx];
    let self_act = chunk_activity[chunk_idx];
    let stable = chunk_stable_ticks[chunk_idx];

    var neighbor_act = false;
    var neighbor_edit = false;

    let icx = i32(cx);
    let icy = i32(cy);
    let n_chunks_x = i32(params.chunks_x);
    let n_chunks_y = i32(params.chunks_y);

    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            if (dx == 0 && dy == 0) {
                continue;
            }
            let nx = icx + dx;
            let ny = icy + dy;
            if (nx >= 0 && ny >= 0 && nx < n_chunks_x && ny < n_chunks_y) {
                let n_idx = u32(ny * n_chunks_x + nx);
                if (chunk_activity[n_idx] != 0u) {
                    neighbor_act = true;
                }
                if (chunk_edit_wake[n_idx] != 0u) {
                    neighbor_edit = true;
                }
            }
        }
    }

    var reasons = WAKE_REASON_NONE;
    if (self_act != 0u) {
        reasons = reasons | WAKE_REASON_SELF_ACTIVITY;
    }
    if (neighbor_act || neighbor_edit) {
        reasons = reasons | WAKE_REASON_NEIGHBOR_HALO;
    }
    if (edit_wake != 0u) {
        reasons = reasons | WAKE_REASON_USER_EDIT;
    }
    if (stable < params.sleep_threshold) {
        reasons = reasons | WAKE_REASON_SETTLING;
    }

    if (reasons != WAKE_REASON_NONE) {
        chunk_state[chunk_idx] = CHUNK_STATE_RUNNABLE;
        chunk_wake_reason[chunk_idx] = reasons;
    } else {
        chunk_state[chunk_idx] = CHUNK_STATE_SLEEPING;
        chunk_wake_reason[chunk_idx] = WAKE_REASON_NONE;
    }
}
