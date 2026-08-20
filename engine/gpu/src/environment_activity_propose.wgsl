struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
    chunk_size: u32,
    chunks_x: u32,
    chunks_y: u32,
    sleep_enabled: u32,
    boundary_mode: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};
struct ThermalTable { values: array<vec4<f32>, 8> };

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> temperature_current: array<f32>;
@group(0) @binding(3) var<storage, read> air_mass_current: array<f32>;
@group(0) @binding(4) var<storage, read> air_energy_current: array<f32>;
@group(0) @binding(5) var<uniform> thermal_table: ThermalTable;
@group(0) @binding(6) var<storage, read_write> cell_activity: array<u32>;

const EMPTY: u32 = 0u;
const TABLE_LEN: u32 = 16u;
const ACTIVITY_ENVIRONMENT: u32 = 16u;
const ACTIVITY_THERMAL: u32 = 2u;
const AIR_PRESSURE_DEADBAND: f32 = 0.001;
const THERMAL_DEADBAND_C: f32 = 0.01;
const AIR_ZERO_OFFSET: f32 = 273.15;
const AIR_THERMAL_CONDUCTIVITY: f32 = 0.025;
const MATTER_AIR_INTERFACE_CONDUCTANCE: f32 = 0.05;
const SEALED: u32 = 0u;
const STANDARD_AIR_ENERGY: f32 = 293.15;

fn inside(x: i32, y: i32) -> bool {
    return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
}

fn index_of(x: i32, y: i32) -> u32 {
    return u32(y) * params.width + u32(x);
}

fn properties(material: u32) -> vec2<f32> {
    let packed = thermal_table.values[material / 2u];
    return select(packed.xy, packed.zw, (material & 1u) != 0u);
}

fn has_thermal_node(index: u32) -> bool {
    let material = material_current[index];
    return (material != EMPTY && material < TABLE_LEN && properties(material).y > 0.0)
        || (material == EMPTY && air_mass_current[index] > 0.0);
}

fn temperature_c(index: u32) -> f32 {
    if (material_current[index] != EMPTY) {
        return temperature_current[index];
    }
    return air_energy_current[index] / air_mass_current[index] - AIR_ZERO_OFFSET;
}

fn air_pressure(index: u32) -> f32 {
    return select(0.0, air_energy_current[index] / STANDARD_AIR_ENERGY, air_mass_current[index] > 0.0);
}

fn thermal_conductance(a: u32, b: u32) -> f32 {
    if (!has_thermal_node(a) || !has_thermal_node(b)) {
        return 0.0;
    }
    let material_a = material_current[a];
    let material_b = material_current[b];
    if (material_a == EMPTY && material_b == EMPTY) {
        return AIR_THERMAL_CONDUCTIVITY;
    }
    if (material_a == EMPTY) {
        return min(properties(material_b).x, MATTER_AIR_INTERFACE_CONDUCTANCE);
    }
    if (material_b == EMPTY) {
        return min(properties(material_a).x, MATTER_AIR_INTERFACE_CONDUCTANCE);
    }
    return min(properties(material_a).x, properties(material_b).x);
}

fn inspect_face(index: u32, nx: i32, ny: i32) -> u32 {
    if (!inside(nx, ny)) {
        if (params.boundary_mode == SEALED || material_current[index] != EMPTY) {
            return 0u;
        }
        var boundary_bits = 0u;
        if (abs(air_pressure(index) - 1.0) > AIR_PRESSURE_DEADBAND) {
            boundary_bits |= ACTIVITY_ENVIRONMENT;
        }
        if (air_mass_current[index] > 0.0
            && abs(temperature_c(index) - 20.0) > THERMAL_DEADBAND_C) {
            boundary_bits |= ACTIVITY_THERMAL;
        }
        return boundary_bits;
    }
    let neighbor = index_of(nx, ny);
    var bits = 0u;
    if (material_current[index] == EMPTY
        && material_current[neighbor] == EMPTY
        && abs(air_pressure(index) - air_pressure(neighbor)) > AIR_PRESSURE_DEADBAND) {
        bits |= ACTIVITY_ENVIRONMENT;
    }
    let conductance = thermal_conductance(index, neighbor);
    if (conductance > 0.0
        && abs(temperature_c(index) - temperature_c(neighbor)) > THERMAL_DEADBAND_C) {
        bits |= ACTIVITY_THERMAL;
    }
    return bits;
}

@compute @workgroup_size(64, 1, 1)
fn environment_activity_propose_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) {
        return;
    }
    let x = i32(index % params.width);
    let y = i32(index / params.width);
    cell_activity[index] |= inspect_face(index, x - 1, y)
        | inspect_face(index, x + 1, y)
        | inspect_face(index, x, y - 1)
        | inspect_face(index, x, y + 1);
}
