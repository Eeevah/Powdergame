// G7-A activity detector — propose pass (own WGSL module; no Rust string
// scanning).
//
// Every cell self-writes its per-cell activity bitmask to `cell_activity`
// (deterministic, no atomics, no workgroup memory — G6 write-ownership
// principles preserved). The bits follow the movement stencil and field
// gradient semantics from `engine/core/src/activity.rs`:
//
//   ACTIVITY_MATTER    (1 << 0): movable Matter whose ordered local
//                       stencil has ANY real candidate — an EMPTY move, a
//                       density-swap-appropriate neighbor, or an
//                       out-of-domain Void exit. Existence is not activity:
//                       a settled cell with Matter on every stencil stage is
//                       inactive. Only 1-cell local candidates are examined
//                       (no long-distance scan).
//   ACTIVITY_THERMAL   (1 << 1): |T - T_neighbor| > eps over the 4-neighbor
//                       field, or an active heat source (COMBUSTING).
//   ACTIVITY_PRESSURE  (1 << 2): |P - P_neighbor| > eps over the 4-neighbor
//                       field (spatial scalar field, never moved on
//                       ownership edges).
//   ACTIVITY_REACTION  (1 << 3): reaction state actively changing
//                       (COMBUSTING flag, or a progressing DECAY_AGE).
//
// EMPTY cells never contribute activity. The epsilons are gameplay
// measurement baselines (not sleep thresholds — G7-B decides those).

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

const EMPTY: u32 = 0u;

const ACTIVITY_MATTER: u32 = 1u << 0u;
const ACTIVITY_THERMAL: u32 = 1u << 1u;
const ACTIVITY_PRESSURE: u32 = 1u << 2u;
const ACTIVITY_REACTION: u32 = 1u << 3u;

const FLAG_COMBUSTING: u32 = 1u << 0u;
const FLAG_DECAY_AGE_SHIFT: u32 = 16u;
const FLAG_DECAY_AGE_MASK: u32 = 0x0FFFu << FLAG_DECAY_AGE_SHIFT;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> temperature_current: array<f32>;
@group(0) @binding(3) var<storage, read> pressure_current: array<f32>;
@group(0) @binding(4) var<storage, read> flags_current: array<u32>;
@group(0) @binding(5) var<storage, read> class_table: array<u32, 16>;
@group(0) @binding(6) var<storage, read> density_table: array<u32, 16>;
@group(0) @binding(7) var<storage, read_write> cell_activity: array<u32>;

// Movement candidate kind mirroring movement_propose: 0 = out of domain
// (Void), 1 = EMPTY, 2 = static/blocked, 3 = movable Matter.
fn candidate_kind(x: i32, y: i32) -> u32 {
    if (x < 0 || y < 0 || x >= i32(params.width) || y >= i32(params.height)) {
        return 0u;
    }
    let mat = material_current[u32(y) * params.width + u32(x)];
    if (mat == EMPTY) {
        return 1u;
    }
    if (density_table[mat] == 0u) {
        return 2u;
    }
    return 3u;
}

fn candidate_rank(x: i32, y: i32) -> u32 {
    let mat = material_current[u32(y) * params.width + u32(x)];
    return density_table[mat];
}

// One vertical stencil stage has a real frontier if the candidate is a
// Void exit, an EMPTY move, or a density-swap-appropriate neighbor
// (lighter_rises mirrors movement's GAS-up vs POWDER/LIQUID-down ordering).
fn vertical_frontier(x: i32, y: i32, dx: i32, dy: i32, src_rank: u32, lighter_rises: bool) -> bool {
    let kind = candidate_kind(x + dx, y + dy);
    if (kind == 0u || kind == 1u) {
        return true;
    }
    if (kind == 3u) {
        let dest_rank = candidate_rank(x + dx, y + dy);
        if (lighter_rises) {
            return src_rank < dest_rank;
        }
        return src_rank > dest_rank;
    }
    return false;
}

// One lateral stage: EMPTY-only (no lateral density swap), Void is a
// frontier.
fn lateral_frontier(x: i32, y: i32, dx: i32) -> bool {
    let kind = candidate_kind(x + dx, y);
    return kind == 0u || kind == 1u;
}

// MATTER activity per movement class. "Any stage has a candidate" is
// equivalent to "movement propose would not have returned NO_MOVE".
fn matter_frontier(x: i32, y: i32, cls: u32, src_rank: u32) -> bool {
    if (cls == 1u) {
        // POWDER: down → down-diagonal.
        if (vertical_frontier(x, y, 0, 1, src_rank, false)) {
            return true;
        }
        return vertical_frontier(x, y, -1, 1, src_rank, false)
            || vertical_frontier(x, y, 1, 1, src_rank, false);
    }
    if (cls == 2u) {
        // LIQUID: down → down-diagonal → lateral.
        if (vertical_frontier(x, y, 0, 1, src_rank, false)) {
            return true;
        }
        if (vertical_frontier(x, y, -1, 1, src_rank, false)
            || vertical_frontier(x, y, 1, 1, src_rank, false)) {
            return true;
        }
        return lateral_frontier(x, y, -1) || lateral_frontier(x, y, 1);
    }
    // GAS: up → up-diagonal → lateral.
    if (vertical_frontier(x, y, 0, -1, src_rank, true)) {
        return true;
    }
    if (vertical_frontier(x, y, -1, -1, src_rank, true)
        || vertical_frontier(x, y, 1, -1, src_rank, true)) {
        return true;
    }
    return lateral_frontier(x, y, -1) || lateral_frontier(x, y, 1);
}

fn in_domain(x: i32, y: i32) -> bool {
    return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
}

fn neighbor_temperature(x: i32, y: i32) -> f32 {
    return temperature_current[u32(y) * params.width + u32(x)];
}

fn neighbor_pressure(x: i32, y: i32) -> f32 {
    return pressure_current[u32(y) * params.width + u32(x)];
}

// 4-neighbor field gradient exists for temperature / pressure.
fn thermal_frontier(x: i32, y: i32, t: f32, flags: u32) -> bool {
    if ((flags & FLAG_COMBUSTING) != 0u) {
        return true; // active heat source
    }
    if (in_domain(x - 1, y) && abs(t - neighbor_temperature(x - 1, y)) > params.thermal_eps) {
        return true;
    }
    if (in_domain(x + 1, y) && abs(t - neighbor_temperature(x + 1, y)) > params.thermal_eps) {
        return true;
    }
    if (in_domain(x, y - 1) && abs(t - neighbor_temperature(x, y - 1)) > params.thermal_eps) {
        return true;
    }
    if (in_domain(x, y + 1) && abs(t - neighbor_temperature(x, y + 1)) > params.thermal_eps) {
        return true;
    }
    return false;
}

fn pressure_frontier(x: i32, y: i32, p: f32) -> bool {
    if (in_domain(x - 1, y) && abs(p - neighbor_pressure(x - 1, y)) > params.pressure_eps) {
        return true;
    }
    if (in_domain(x + 1, y) && abs(p - neighbor_pressure(x + 1, y)) > params.pressure_eps) {
        return true;
    }
    if (in_domain(x, y - 1) && abs(p - neighbor_pressure(x, y - 1)) > params.pressure_eps) {
        return true;
    }
    if (in_domain(x, y + 1) && abs(p - neighbor_pressure(x, y + 1)) > params.pressure_eps) {
        return true;
    }
    return false;
}

@compute
@workgroup_size(64)
fn propose_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) {
        return;
    }

    var mask = 0u;
    let mat = material_current[index];
    if (mat != EMPTY) {
        let x = i32(index % params.width);
        let y = i32(index / params.width);
        let flags = flags_current[index];

        let cls = class_table[mat];
        if (cls != 0u) {
            let src_rank = density_table[mat];
            if (matter_frontier(x, y, cls, src_rank)) {
                mask = mask | ACTIVITY_MATTER;
            }
        }
        if (thermal_frontier(x, y, temperature_current[index], flags)) {
            mask = mask | ACTIVITY_THERMAL;
        }
        if (pressure_frontier(x, y, pressure_current[index])) {
            mask = mask | ACTIVITY_PRESSURE;
        }
        if ((flags & FLAG_COMBUSTING) != 0u || (flags & FLAG_DECAY_AGE_MASK) != 0u) {
            mask = mask | ACTIVITY_REACTION;
        }
    }

    cell_activity[index] = mask;
}
