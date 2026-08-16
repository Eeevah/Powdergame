// G5-A scalar pressure baseline. Read Neighbors → Write Self.
// Constants must match engine/core/src/pressure.rs.
// EMPTY/Void and STATIC/POWDER are not hidden pressure media.

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
@group(0) @binding(2) var<storage, read> pressure_current: array<f32>;
@group(0) @binding(3) var<storage, read_write> pressure_next: array<f32>;
@group(0) @binding(4) var<storage, read> movement_class_table: array<u32>;
@group(0) @binding(5) var<storage, read> chunk_state: array<u32>;

const EMPTY: u32 = 0u;
const TABLE_LEN: u32 = 16u;
const CLASS_LIQUID: u32 = 2u;
const CLASS_GAS: u32 = 3u;
const PRESSURE_REFERENCE: f32 = 0.0;
const PRESSURE_DIFFUSION_RATE: f32 = 0.20;
const PRESSURE_MAX: f32 = 1.0e6;

fn sanitize_pressure(value: f32) -> f32 {
    if (value != value) {
        return PRESSURE_REFERENCE;
    }
    if (value > 1.0e20 || value < -1.0e20) {
        return PRESSURE_REFERENCE;
    }
    return clamp(value, PRESSURE_REFERENCE, PRESSURE_MAX);
}

fn in_domain(x: i32, y: i32) -> bool {
    return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
}

fn index_of(x: i32, y: i32) -> u32 {
    return u32(y) * params.width + u32(x);
}

fn is_pressure_medium(material: u32) -> bool {
    if (material == EMPTY || material >= TABLE_LEN) {
        return false;
    }
    let movement_class = movement_class_table[material];
    return movement_class == CLASS_LIQUID || movement_class == CLASS_GAS;
}

fn accumulate(self_p: f32, nx: i32, ny: i32) -> f32 {
    if (!in_domain(nx, ny)) {
        return 0.0;
    }
    let nidx = index_of(nx, ny);
    if (!is_pressure_medium(material_current[nidx])) {
        return 0.0;
    }
    return sanitize_pressure(pressure_current[nidx]) - self_p;
}

@compute @workgroup_size(64, 1, 1)
fn pressure_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) {
        return;
    }

    if (params.sleep_enabled != 0u) {
        let cx = (index % params.width) / params.chunk_size;
        let cy = (index / params.width) / params.chunk_size;
        if (chunk_state[cy * params.chunks_x + cx] != 0u) {
            if (!is_pressure_medium(material_current[index])) {
                pressure_next[index] = PRESSURE_REFERENCE;
            } else {
                pressure_next[index] = sanitize_pressure(pressure_current[index]);
            }
            return;
        }
    }

    if (!is_pressure_medium(material_current[index])) {
        pressure_next[index] = PRESSURE_REFERENCE;
        return;
    }

    let self_p = sanitize_pressure(pressure_current[index]);
    let x = i32(index % params.width);
    let y = i32(index / params.width);

    var acc = 0.0;
    acc += accumulate(self_p, x, y - 1);
    acc += accumulate(self_p, x, y + 1);
    acc += accumulate(self_p, x - 1, y);
    acc += accumulate(self_p, x + 1, y);

    pressure_next[index] = sanitize_pressure(self_p + PRESSURE_DIFFUSION_RATE * acc);
}
