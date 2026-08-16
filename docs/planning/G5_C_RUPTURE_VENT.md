# G5-C Pressure Stress → Rupture → Opening → Vent

Status: **TECHNICAL PASS / FROZEN**

Validation date: 2026-08-16

## Frozen implementation

- Tested implementation SHA: `5187d9980f9067cced1edb0b6a8f79ab56147a0c`
- Branch: `feature/m0-g5-rupture-vent`
- Reference hardware: Windows 11 / NVIDIA GeForce RTX 5090 / `wgpu::Backend::Dx12`
- Vendor ID: `0x10DE`

## Contract

```text
Pressure in Liquid/Gas
→ adjacent weak structural Matter reads local Pressure
→ Material rupture threshold exceeded
→ structure self-writes EMPTY
→ opening created
→ ordinary Matter movement uses the opening on following ticks
→ pressured Matter vents
→ vacated spatial Pressure is released
```

Pressure remains a spatial scalar Field. Static structures do not become Pressure media and do not store pressure. Rupture follows Read Neighbors → Write Self and has no boiler-specific explosion code.

M0 structural baseline:

- Wood: `rupture_threshold = 80.0`
- Stone: unbreakable reference wall
- Boundary Block: unbreakable
- Fully blocked Water → Steam expansion: `blocked_pressure = 100.0`

Stone remains intentionally unbreakable because the frozen G5-A containment tests use Stone at pressure up to `1.0e6`.

## RTX 5090 / DX12 validation

### WGSL parser

`cargo test -p powdergame-gpu --test wgsl_parse -- --nocapture`

- 1 passed, 0 failed
- `rupture.wgsl` included in the production shader parse set

### G5-C rupture suite

`cargo test -p powdergame-gpu --test rupture -- --nocapture --test-threads=1`

**5 passed, 0 failed**

1. `wood_survives_sub_threshold_pressure`
   - Wood survives pressure 79.0 below threshold 80.0.
2. `wood_ruptures_from_threshold_exceeding_neighbor_pressure`
   - Neighbor Water pressure 100.0 ruptures Wood to EMPTY while Water identity remains unchanged.
3. `stone_and_boundary_remain_reference_unbreakable_walls`
   - Stone and Boundary survive pressure `1.0e6`; frozen G5-A containment contract preserved.
4. `rupture_crosses_64_cell_chunk_boundary`
   - Pressure at x=63 ruptures Wood at x=64; chunk edges are not stress walls.
5. `blocked_boiling_ruptures_weak_wall_then_vents_on_following_tick`
   - Tick 1: hot confined Water boils → expansion fails → +100 Pressure → adjacent Wood ruptures to EMPTY.
   - Tick 2: ordinary GAS movement moves Steam through the new opening; the vacated source Pressure becomes `PRESSURE_REFERENCE (0.0)`.
   - No dedicated boiler/vent/explosion special case is involved.

## Frozen-regression evidence

- G5-B expansion suite: **5 passed, 0 failed**
- G5-A pressure suite: **8 passed, 0 failed**
- G4-B phase suite: **16 passed, 0 failed**
- Full GPU integration: **143 passed, 0 failed, 1 ignored** (`controlled_reference_world_perf`)
- Core unit tests: **130 passed, 0 failed**
- `cargo check --workspace --all-targets`: **0 errors, 0 warnings**
- `git diff --check`: **clean**

Validation worktree was clean and the original repository's user-owned `docs/planning/MATERIAL_CANDIDATES.md` remained untouched/untracked exactly as before validation.

## Gate decision

**G5-C TECHNICAL PASS / FROZEN.**

Together, G5-A Pressure Field, G5-B Expansion / Confinement → Pressure, and G5-C Pressure Stress → Rupture → Opening → Vent now form a technically validated pressure-chain implementation.

This does **not** by itself close G5 as a user-facing milestone. Final G5 closure still requires a visible boiler-chain user validation demonstrating that the emergent sequence reads convincingly on screen without special-case explosion logic.
