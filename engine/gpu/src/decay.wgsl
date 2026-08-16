// G4-D — Material-Owned Decay Pass.
//
// Reads this cell's Material and flags, applies the generic decay rule
// (Material-owned decay descriptor table — no material-name branches), and
// writes ONLY self slots:
//   material_next[self], temperature_next[self], flags_next[self].
// No Claim/Resolve, no atomics.
//
// Rule:
//   if decay_table[material].lifetime_ticks > 0:
//     age = read_decay_age(flags) + 1
//     if age >= decay_table[material].lifetime_ticks:
//       material_next = decay_table[material].target_material (EMPTY)
//       temperature_next = TEMPERATURE_REFERENCE (0.0)
//       flags_next = 0 (if EMPTY)
//     else:
//       material_next = material
//       temperature_next = temperature
//       flags_next = with_decay_age(flags, age)
//   else:
//     material_next = material
//     temperature_next = temperature
//     flags_next = flags & ~FLAG_DECAY_AGE_MASK (clears stale decay bits on non-decay matter)

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

struct DecayDesc {
    lifetime_ticks: u32,
    target_material: u32,
};

const FLAG_DECAY_AGE_SHIFT: u32 = 16u;
const FLAG_DECAY_AGE_MASK: u32 = 0x0FFFu << 16u;
const TEMPERATURE_REFERENCE: f32 = 0.0;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> flags_current: array<u32>;
@group(0) @binding(3) var<storage, read> temperature_current: array<f32>;
@group(0) @binding(4) var<storage, read> decay_table: array<DecayDesc, 16>;
@group(0) @binding(5) var<storage, read_write> material_next: array<u32>;
@group(0) @binding(6) var<storage, read_write> flags_next: array<u32>;
@group(0) @binding(7) var<storage, read_write> temperature_next: array<f32>;
@group(0) @binding(8) var<storage, read> chunk_state: array<u32>;

fn read_decay_age(f: u32) -> u32 {
    return (f & FLAG_DECAY_AGE_MASK) >> FLAG_DECAY_AGE_SHIFT;
}

fn with_decay_age(f: u32, a: u32) -> u32 {
    return (f & ~FLAG_DECAY_AGE_MASK) | ((a & 0x0FFFu) << FLAG_DECAY_AGE_SHIFT);
}

@compute @workgroup_size(64, 1, 1)
fn decay_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) {
        return;
    }

    let mat = material_current[index];
    let flags = flags_current[index];
    let temp = temperature_current[index];

    if (params.sleep_enabled != 0u) {
        let cx = (index % params.width) / params.chunk_size;
        let cy = (index / params.width) / params.chunk_size;
        if (chunk_state[cy * params.chunks_x + cx] != 0u) {
            material_next[index] = mat;
            temperature_next[index] = temp;
            flags_next[index] = flags & ~FLAG_DECAY_AGE_MASK;
            return;
        }
    }

    if (mat >= 16u) {
        material_next[index] = mat;
        temperature_next[index] = temp;
        flags_next[index] = flags & ~FLAG_DECAY_AGE_MASK;
        return;
    }

    let desc = decay_table[mat];
    if (desc.lifetime_ticks > 0u) {
        let cur_age = read_decay_age(flags);
        let next_age = cur_age + 1u;
        if (next_age >= desc.lifetime_ticks) {
            // Lifetime reached: transform into target material (EMPTY)
            material_next[index] = desc.target_material;
            temperature_next[index] = TEMPERATURE_REFERENCE;
            if (desc.target_material == 0u) {
                flags_next[index] = 0u;
            } else {
                flags_next[index] = flags & ~FLAG_DECAY_AGE_MASK;
            }
            return;
        } else {
            material_next[index] = mat;
            temperature_next[index] = temp;
            flags_next[index] = with_decay_age(flags, next_age);
            return;
        }
    }

    // Non-decay material: preserve material, temp, and clean any stale decay bits
    material_next[index] = mat;
    temperature_next[index] = temp;
    flags_next[index] = flags & ~FLAG_DECAY_AGE_MASK;
}
