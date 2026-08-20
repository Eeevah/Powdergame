// G4-A thermal baseline. Read Neighbors → Write Self.
// EMPTY / unknown / Void contribute no heat. No claim/resolve.
// Constants must match engine/core/src/thermal.rs.

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
@group(0) @binding(3) var<storage, read_write> temperature_next: array<f32>;
@group(0) @binding(4) var<storage, read> conductivity_table: array<f32>;
@group(0) @binding(5) var<storage, read> capacity_table: array<f32>;
@group(0) @binding(6) var<storage, read> chunk_state: array<u32>;

const EMPTY: u32 = 0u;
const TABLE_LEN: u32 = 16u;
const TEMPERATURE_REFERENCE: f32 = 20.0;
const THERMAL_DEADBAND: f32 = 0.01;
const THERMAL_RATE: f32 = 0.12;
const THERMAL_MAX_DELTA: f32 = 25.0;
const THERMAL_MIN_CAPACITY: f32 = 0.25;

fn sanitize(t: f32) -> f32 {
    if (t != t) {
        return TEMPERATURE_REFERENCE;
    }
    if (t > 1.0e20 || t < -1.0e20) {
        return TEMPERATURE_REFERENCE;
    }
    return t;
}

fn in_domain(x: i32, y: i32) -> bool {
    return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
}

fn index_of(x: i32, y: i32) -> u32 {
    return u32(y) * params.width + u32(x);
}

fn lookup_k(id: u32) -> f32 {
    if (id == EMPTY || id >= TABLE_LEN) {
        return 0.0;
    }
    return max(conductivity_table[id], 0.0);
}

fn lookup_c(id: u32) -> f32 {
    if (id == EMPTY || id >= TABLE_LEN) {
        return 0.0;
    }
    return capacity_table[id];
}

fn is_matter(id: u32) -> bool {
    return id != EMPTY && id < TABLE_LEN && lookup_c(id) > 0.0;
}

fn accumulate(self_t: f32, k_self: f32, nx: i32, ny: i32) -> f32 {
    if (!in_domain(nx, ny)) {
        return 0.0;
    }
    let nidx = index_of(nx, ny);
    let nid = material_current[nidx];
    if (!is_matter(nid)) {
        return 0.0;
    }
    let n_t = sanitize(temperature_current[nidx]);
    let delta = n_t - self_t;
    if (abs(delta) <= THERMAL_DEADBAND) {
        return 0.0;
    }
    let k_eff = min(k_self, lookup_k(nid));
    return k_eff * delta;
}

@compute @workgroup_size(64, 1, 1)
fn thermal_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) {
        return;
    }

    if (params.sleep_enabled != 0u) {
        let cx = (index % params.width) / params.chunk_size;
        let cy = (index / params.width) / params.chunk_size;
        if (chunk_state[cy * params.chunks_x + cx] != 0u) {
            if (!is_matter(material_current[index])) {
                temperature_next[index] = TEMPERATURE_REFERENCE;
            } else {
                temperature_next[index] = sanitize(temperature_current[index]);
            }
            return;
        }
    }

    let mat = material_current[index];
    if (!is_matter(mat)) {
        temperature_next[index] = TEMPERATURE_REFERENCE;
        return;
    }

    let self_t = sanitize(temperature_current[index]);
    let k_self = lookup_k(mat);
    let capacity = max(lookup_c(mat), THERMAL_MIN_CAPACITY);
    let x = i32(index % params.width);
    let y = i32(index / params.width);

    var acc = 0.0;
    acc += accumulate(self_t, k_self, x, y - 1);
    acc += accumulate(self_t, k_self, x, y + 1);
    acc += accumulate(self_t, k_self, x - 1, y);
    acc += accumulate(self_t, k_self, x + 1, y);

    var change = THERMAL_RATE * acc / capacity;
    change = clamp(change, -THERMAL_MAX_DELTA, THERMAL_MAX_DELTA);
    temperature_next[index] = sanitize(self_t + change);
}
