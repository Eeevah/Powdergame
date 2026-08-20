// G4-C — Combustion pass (own WGSL module; no Rust string scanning).
//
// Reads this cell's Material + Temperature + flags, applies the generic
// combustion rule (Material-owned descriptor table — no material-name
// branches), and writes ONLY self slots:
//   material_next[self], temperature_next[self], flags_next[self],
//   proposal[self] (smoke spawn).
// No Claim/Resolve, no atomics (REACTION_SPEC §9/§11).
//
// Rule (matches engine/core/src/combustion.rs):
//   unlit + T >= ignition    → ignite (COMBUSTING + FLAME_EVENT)
//   burning + T >= sustain   → keep burning, add heat_per_tick
//   burning + T  < sustain   → extinguish (COMBUSTING/FLAME_EVENT clear,
//                              fuel progress PRESERVED)
//   burning fuel progress    → +1 per active burning tick; reignition
//                              continues from the remaining fuel
//   progress >= burn_duration → fuel consumed → material_next = EMPTY,
//                              temperature_next = 0, flags_next = 0,
//                              no Smoke spawn this tick
//   non-combustible          → never burns; combustion bits cleared
//                              (including stale fuel progress)
//
// Fire is NOT a Material: flame = Matter + COMBUSTING + heat + FLAME_EVENT
// presentation signal. Only the combustion-owned bits (bool state + u16
// fuel progress in bits 8..23) are set/cleared; unrelated future flag bits
// are preserved.
//
// A burning source requests AT MOST ONE local 1-cell Smoke spawn into an
// in-domain EMPTY cell (up → up-diagonal → lateral, parity ordered). The
// proposal buffer is a request only — the Smoke claim/commit passes
// (separate dispatches after this one) resolve ownership with exactly one
// winner per destination; a source thread never overwrites a neighbor.
//
// Pressure is a spatial field (G5) and is never written here.
//
// Smoke proposal encoding (reuses the movement `proposal` buffer, which is
// fully consumed by the movement claim pass before this pass runs):
//   0            = no spawn
//   index + 1    = spawn Smoke at `index`

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

struct CombDesc {
    is_combustible: u32,
    ignition: f32,
    sustain: f32,
    heat_per_tick: f32,
    burn_duration: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

struct CombTable {
    table: array<CombDesc, 16>,
};

const EMPTY: u32 = 0u;
const FLAG_COMBUSTING: u32 = 1u;
const FLAG_FLAME_EVENT: u32 = 2u;
const FLAG_FUEL_PROGRESS_SHIFT: u32 = 4u;
const FLAG_FUEL_PROGRESS_MASK: u32 = 0x0FFFu << 4u;
const COMBUSTION_MASK: u32 = FLAG_COMBUSTING | FLAG_FLAME_EVENT | FLAG_FUEL_PROGRESS_MASK;
const TEMPERATURE_REFERENCE: f32 = 20.0;
const TEMPERATURE_MIN: f32 = -250.0;
const TEMPERATURE_MAX: f32 = 2000.0;
const COMBUSTION_MAX_TEMPERATURE: f32 = 1200.0;
const NO_SPAWN: u32 = 0u;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> temperature_current: array<f32>;
@group(0) @binding(3) var<storage, read> flags_current: array<u32>;
@group(0) @binding(4) var<uniform> combustion_table: CombTable;
@group(0) @binding(5) var<storage, read_write> temperature_next: array<f32>;
@group(0) @binding(6) var<storage, read_write> flags_next: array<u32>;
@group(0) @binding(7) var<storage, read_write> proposal: array<u32>;
@group(0) @binding(8) var<storage, read_write> material_next: array<u32>;
@group(0) @binding(9) var<storage, read> chunk_state: array<u32>;

fn sanitize(t: f32) -> f32 {
    if (t != t) {
        return TEMPERATURE_REFERENCE;
    }
    if (t > 1.0e20 || t < -1.0e20) {
        return TEMPERATURE_REFERENCE;
    }
    return clamp(t, TEMPERATURE_MIN, TEMPERATURE_MAX);
}

fn fuel_progress(f: u32) -> u32 {
    return (f & FLAG_FUEL_PROGRESS_MASK) >> FLAG_FUEL_PROGRESS_SHIFT;
}

fn with_fuel_progress(f: u32, p: u32) -> u32 {
    return (f & ~FLAG_FUEL_PROGRESS_MASK) | ((p & 0x0FFFu) << FLAG_FUEL_PROGRESS_SHIFT);
}


fn in_domain(x: i32, y: i32) -> bool {
    return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
}

fn index_of(x: i32, y: i32) -> u32 {
    return u32(y) * params.width + u32(x);
}

fn cell_empty(x: i32, y: i32) -> bool {
    if (!in_domain(x, y)) {
        return false;
    }
    return material_current[index_of(x, y)] == EMPTY;
}

// Smoke spawn stencil: up → up-diagonal (parity) → lateral (parity) → none.
// Returns the target index + 1, or NO_SPAWN. Max one local candidate;
// no long-distance scan; Void is never a spawn target.
fn smoke_target(x: i32, y: i32) -> u32 {
    let parity = (u32(x) + u32(y)) & 1u;
    if (cell_empty(x, y - 1)) {
        return index_of(x, y - 1) + 1u;
    }
    if (parity == 0u) {
        if (cell_empty(x - 1, y - 1)) {
            return index_of(x - 1, y - 1) + 1u;
        }
        if (cell_empty(x + 1, y - 1)) {
            return index_of(x + 1, y - 1) + 1u;
        }
    } else {
        if (cell_empty(x + 1, y - 1)) {
            return index_of(x + 1, y - 1) + 1u;
        }
        if (cell_empty(x - 1, y - 1)) {
            return index_of(x - 1, y - 1) + 1u;
        }
    }
    if (parity == 0u) {
        if (cell_empty(x - 1, y)) {
            return index_of(x - 1, y) + 1u;
        }
        if (cell_empty(x + 1, y)) {
            return index_of(x + 1, y) + 1u;
        }
    } else {
        if (cell_empty(x + 1, y)) {
            return index_of(x + 1, y) + 1u;
        }
        if (cell_empty(x - 1, y)) {
            return index_of(x - 1, y) + 1u;
        }
    }
    return NO_SPAWN;
}

@compute @workgroup_size(64, 1, 1)
fn combustion_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) {
        return;
    }

    let mat = material_current[index];
    let flags = flags_current[index];

    if (params.sleep_enabled != 0u) {
        let cx = (index % params.width) / params.chunk_size;
        let cy = (index / params.width) / params.chunk_size;
        if (chunk_state[cy * params.chunks_x + cx] != 0u) {
            material_next[index] = mat;
            temperature_next[index] = sanitize(temperature_current[index]);
            flags_next[index] = flags;
            proposal[index] = NO_SPAWN;
            return;
        }
    }

    if (mat == EMPTY) {
        temperature_next[index] = TEMPERATURE_REFERENCE;
        flags_next[index] = flags & ~COMBUSTION_MASK;
        proposal[index] = NO_SPAWN;
        return;
    }
    if (mat >= 16u) {
        // Unknown ids are never combustible; preserve temperature.
        temperature_next[index] = sanitize(temperature_current[index]);
        flags_next[index] = flags & ~COMBUSTION_MASK;
        proposal[index] = NO_SPAWN;
        return;
    }

    let desc = combustion_table.table[mat];
    let t = sanitize(temperature_current[index]);

    // Non-combustible Matter can never burn, hold the burning bit, or keep
    // stale fuel progress.
    if (desc.is_combustible == 0u) {
        temperature_next[index] = t;
        flags_next[index] = flags & ~COMBUSTION_MASK;
        proposal[index] = NO_SPAWN;
        return;
    }

    var burning = (flags & FLAG_COMBUSTING) != 0u;
    if (!burning && t >= desc.ignition) {
        burning = true;
    }
    if (burning && t < desc.sustain) {
        burning = false;
    }

    // Fuel progress: +1 per ACTIVE burning tick. Preserved when not burning
    // (extinguish keeps the partial progress; reignition continues from it).
    var progress = fuel_progress(flags);
    if (burning) {
        progress = progress + 1u;
    }
    let consumed = burning && progress >= desc.burn_duration;

    if (consumed) {
        // Fuel exhausted this tick: the cell becomes EMPTY (self-write).
        material_next[index] = EMPTY;
        temperature_next[index] = TEMPERATURE_REFERENCE;
        flags_next[index] = 0u;
        proposal[index] = NO_SPAWN;
        return;
    }

    // Add combustion heat, capped at the gameplay bound but never reducing
    // an already-hotter cell.
    var t_out = t;
    if (burning) {
        t_out = min(t + desc.heat_per_tick, max(COMBUSTION_MAX_TEMPERATURE, t));
    }

    var next_flags = flags & ~COMBUSTION_MASK;
    if (burning) {
        next_flags |= FLAG_COMBUSTING | FLAG_FLAME_EVENT;
    }
    next_flags = with_fuel_progress(next_flags, progress);

    temperature_next[index] = sanitize(t_out);
    flags_next[index] = next_flags;
    if (burning) {
        let x = i32(index % params.width);
        let y = i32(index / params.width);
        proposal[index] = smoke_target(x, y);
    } else {
        proposal[index] = NO_SPAWN;
    }
}
