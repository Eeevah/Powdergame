// TE-1 exact Matter flag ownership after any occupancy/identity stage.
struct Params {
    cell_count: u32, threads_x: u32, width: u32, height: u32,
    chunk_size: u32, chunks_x: u32, chunks_y: u32, sleep_enabled: u32,
};
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_next: array<u32>;
@group(0) @binding(2) var<storage, read_write> flags_next: array<u32>;
const OIL: u32 = 5u;
const SMOKE: u32 = 7u;
const WOOD: u32 = 9u;
const COMBUSTION_MASK: u32 = 0x0000FFF3u;
const DECAY_MASK: u32 = 0x0FFF0000u;
@compute @workgroup_size(64)
fn material_flag_hygiene_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) { return; }
    let material = material_next[c];
    if (material == OIL || material == WOOD) {
        flags_next[c] = flags_next[c] & COMBUSTION_MASK;
    } else if (material == SMOKE) {
        flags_next[c] = flags_next[c] & DECAY_MASK;
    } else {
        flags_next[c] = 0u;
    }
}
