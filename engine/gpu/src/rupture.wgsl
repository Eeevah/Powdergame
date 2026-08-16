// G5-C — generic structural rupture from neighboring scalar Pressure.
//
// A structure never becomes a Pressure medium. It reads the four orthogonal
// neighbor cells and, if any Liquid/Gas pressure reaches its Material-owned
// rupture threshold, writes only its own Matter cell to EMPTY. The opening
// then participates in ordinary movement on following ticks; no special
// explosion/vent code is required.

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> pressure_current: array<f32>;
@group(0) @binding(3) var<storage, read> rupture_threshold_table: array<f32>;
@group(0) @binding(4) var<storage, read> movement_class_table: array<u32>;
@group(0) @binding(5) var<storage, read_write> material_next: array<u32>;
@group(0) @binding(6) var<storage, read_write> temperature_next: array<f32>;
@group(0) @binding(7) var<storage, read_write> flags_next: array<u32>;

const EMPTY: u32 = 0u;
const TABLE_LEN: u32 = 16u;
const CLASS_LIQUID: u32 = 2u;
const CLASS_GAS: u32 = 3u;
const PRESSURE_REFERENCE: f32 = 0.0;
const PRESSURE_MAX: f32 = 1.0e6;
const TEMPERATURE_REFERENCE: f32 = 0.0;

fn sanitize_pressure(value: f32) -> f32 {
    if (value != value || value > 1.0e20 || value < -1.0e20) {
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
    let movement_kind = movement_class_table[material];
    return movement_kind == CLASS_LIQUID || movement_kind == CLASS_GAS;
}

fn neighbor_pressure(x: i32, y: i32) -> f32 {
    if (!in_domain(x, y)) {
        return PRESSURE_REFERENCE;
    }
    let n = index_of(x, y);
    if (!is_pressure_medium(material_current[n])) {
        return PRESSURE_REFERENCE;
    }
    return sanitize_pressure(pressure_current[n]);
}

@compute @workgroup_size(64, 1, 1)
fn rupture_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) {
        return;
    }

    let material = material_current[index];
    if (material == EMPTY || material >= TABLE_LEN) {
        material_next[index] = material;
        return;
    }

    let rupture_limit = rupture_threshold_table[material];
    if (!(rupture_limit > 0.0)) {
        material_next[index] = material;
        return;
    }

    let x = i32(index % params.width);
    let y = i32(index / params.width);
    var local_stress = PRESSURE_REFERENCE;
    local_stress = max(local_stress, neighbor_pressure(x, y - 1));
    local_stress = max(local_stress, neighbor_pressure(x, y + 1));
    local_stress = max(local_stress, neighbor_pressure(x - 1, y));
    local_stress = max(local_stress, neighbor_pressure(x + 1, y));

    if (local_stress >= rupture_limit) {
        material_next[index] = EMPTY;
        temperature_next[index] = TEMPERATURE_REFERENCE;
        flags_next[index] = 0u;
    } else {
        material_next[index] = material;
    }
}
