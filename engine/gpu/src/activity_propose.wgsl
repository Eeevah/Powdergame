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
//                       field, an active heat source (COMBUSTING), a
//                       phase-transition condition currently satisfied on
//                       the cell's own Material+T (defensive: 1:1
//                       transitions self-resolve within the same tick, so
//                       the phase pass also marks the actual transition
//                       tick directly in this buffer), or a phase
//                       transition that actually fired this tick.
//   ACTIVITY_PRESSURE  (1 << 2): |P - P_neighbor| > eps over the 4-neighbor
//                       field, evaluated ONLY on pressure-media cells
//                       (LIQUID/GAS — G5 contract; EMPTY/STATIC/POWDER are
//                       not pressure media and their field is zeroed each
//                       tick, so they never carry a pressure frontier).
//                       Pressure is a spatial scalar field, never moved on
//                       ownership edges.
//   ACTIVITY_REACTION  (1 << 3): reaction state actively changing
//                       (COMBUSTING flag, or a progressing DECAY_AGE).
//
// EMPTY cells never contribute activity. The epsilons are gameplay
// measurement baselines (not sleep thresholds — G7-B decides those).
//
// Chunk seams: this pass reads 1-cell neighbors in world coordinates, so
// a frontier on the far side of a chunk boundary is detected normally — a
// chunk seam is not a detection wall. There is deliberately NO
// dedicated chunk-to-chunk wake propagation pass in G7-A (that arrives
// with actual sleep in G7-B); here the boundary is crossed by the ordinary
// cell-level stencil.
//
// chunk_changed_this_tick means "a frontier (activity) was present in this
// chunk this tick" — it resets the stable counter. It does NOT compare
// previous/next world state; real state-delta dirty tracking, if needed,
// is a separate G7-B design.

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

struct PhaseDesc {
    below_target: u32,
    above_target: u32,
    below_yield: u32,
    above_yield: u32,
    below_threshold: f32,
    above_threshold: f32,
    below_blocked_pressure: f32,
    above_blocked_pressure: f32,
};

const EMPTY: u32 = 0u;
const TABLE_LEN: u32 = 16u;
const NO_PHASE_TARGET: u32 = 0xFFFFFFFFu;
const CLASS_LIQUID: u32 = 2u;
const CLASS_GAS: u32 = 3u;

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
// G4-B phase descriptor table (Material property, shared with the phase
// pass — the activity detector reads it to flag cells whose own
// Material + Temperature currently satisfy a phase rule).
@group(0) @binding(8) var<storage, read> phase_table: array<PhaseDesc, TABLE_LEN>;

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

// G5 contract: only LIQUID/GAS are pressure media. EMPTY/STATIC/POWDER
// cells have their pressure field zeroed every tick and never carry a
// pressure frontier, so PRESSURE activity is evaluated on media only
// (avoids false positives on non-medium cells at a field boundary while
// never missing real pressure work).
fn is_pressure_medium(mat: u32) -> bool {
    if (mat == EMPTY || mat >= TABLE_LEN) {
        return false;
    }
    let cls = class_table[mat];
    return cls == CLASS_LIQUID || cls == CLASS_GAS;
}

fn pressure_frontier(x: i32, y: i32, mat: u32, p: f32) -> bool {
    if (!is_pressure_medium(mat)) {
        return false;
    }
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

// A cell whose own Material + Temperature satisfies a phase rule has
// pending phase work — it must never be observed as stable. In the current
// 1:1 write-self semantics such a cell always transforms within the same
// tick (hysteresis keeps the post-transition state stable), so this check
// is defensive; the phase pass itself marks the actual transition tick in
// `cell_activity` (THERMAL), which is the observable signal.
fn phase_candidate(mat: u32, t: f32) -> bool {
    if (mat == EMPTY || mat >= TABLE_LEN) {
        return false;
    }
    let desc = phase_table[mat];
    if (desc.below_target != NO_PHASE_TARGET && t < desc.below_threshold) {
        return true;
    }
    if (desc.above_target != NO_PHASE_TARGET && t > desc.above_threshold) {
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
        let t = temperature_current[index];

        let cls = class_table[mat];
        if (cls != 0u) {
            let src_rank = density_table[mat];
            if (matter_frontier(x, y, cls, src_rank)) {
                mask = mask | ACTIVITY_MATTER;
            }
        }
        if (thermal_frontier(x, y, t, flags) || phase_candidate(mat, t)) {
            mask = mask | ACTIVITY_THERMAL;
        }
        if (pressure_frontier(x, y, mat, pressure_current[index])) {
            mask = mask | ACTIVITY_PRESSURE;
        }
        if ((flags & FLAG_COMBUSTING) != 0u || (flags & FLAG_DECAY_AGE_MASK) != 0u) {
            mask = mask | ACTIVITY_REACTION;
        }
    }

    // OR-merge: the phase pass self-marked this tick's transition in
    // `cell_activity` (THERMAL); preserve it instead of clobbering. Stale
    // bits cannot accumulate because the phase pass clears THERMAL for
    // every cell at the start of each tick and this is the only other
    // writer.
    cell_activity[index] = mask | cell_activity[index];
}
