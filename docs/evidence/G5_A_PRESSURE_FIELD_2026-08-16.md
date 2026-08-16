# G5-A Pressure Field — RTX 5090 / DX12 Technical Evidence

**Date:** 2026-08-16  
**Gate:** G5-A — Scalar Pressure Field baseline  
**Validated commit:** `c8fcb5e1c8106f6c67f57eba1c31bd256de14818`  
**Branch at validation:** `feature/m0-g5-pressure-field`  
**Result:** **TECHNICAL PASS / FROZEN**

This record preserves the user's actual local hardware validation. It does not mark the whole G5 Pressure Chain as ACHIEVED; G5-B expansion/confinement and G5-C rupture/vent remain separate work.

## Target Hardware

- Adapter: NVIDIA GeForce RTX 5090 (`0x10DE`)
- Backend: `wgpu::Backend::Dx12`
- `verify_target_hardware()` enforced the intended adapter/backend during GPU test initialization.

## GPU-free WGSL Parser Regression

Command:

```powershell
cargo test -p powdergame-gpu --test wgsl_parse -- --nocapture
```

Result:

```text
running 1 test
test all_production_wgsl_parses_without_a_gpu ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All production WGSL, including `pressure.wgsl`, parsed successfully.

## G5-A Pressure GPU Contract

Command:

```powershell
cargo test -p powdergame-gpu --test pressure -- --nocapture --test-threads=1
```

Result: **8 passed; 0 failed; 0 ignored**.

Validated behaviors:

1. `isolated_pressure_has_no_time_decay` — sealed isolated pressure retained for 120 ticks; no arbitrary time decay.
2. `material_edit_clears_stale_spatial_pressure` — explicit Matter identity replacement clears stale spatial pressure.
3. `non_medium_cells_clear_pressure` — EMPTY and Stone/non-medium cells resolve to pressure reference `0.0`.
4. `pressure_crosses_chunk_boundary` — pressure propagates across the x=63/64 64-cell chunk boundary.
5. `pressure_propagates_between_adjacent_liquid_cells` — adjacent liquid pressure diffuses through the 4-neighbor rule without spontaneous pair loss.
6. `pressure_world_stays_finite_and_non_negative` — 200-tick run remained finite, non-negative, and free of NaN/Inf runaway.
7. `void_exit_vents_pressure_with_departing_medium` — pressure is removed when its hosting Matter exits the finite domain into Void.
8. `write_pressure_rejects_non_finite` — non-finite authored pressure is rejected.

## Full GPU Integration Regression

Command:

```powershell
cargo test -p powdergame-gpu --tests -- --nocapture --test-threads=1
```

Observed result:

```text
Combustion:       56 passed
Density:          15 passed
Headless smoke:    1 passed
Movement:         16 passed, 1 ignored (controlled_reference_world_perf)
Phase transition: 16 passed
Pressure:          8 passed
Thermal:          13 passed
WGSL parse:        1 passed
World integrity:   7 passed
--------------------------------
GPU total:       133 passed; 0 failed; 1 ignored
```

## Core / Build Regression

```text
cargo test -p powdergame-core
→ 121 passed; 0 failed; 0 ignored

cargo check --workspace --all-targets
→ PASS

git diff --check
→ clean
```

## Working-tree Preservation

The user's existing in-progress document `docs/planning/MATERIAL_CANDIDATES.md` was preserved. No code change or extra validation commit was created by the local validation run.

## Gate Decision

G5-A satisfies its technical sub-gate on the production target:

- scalar `f32` Pressure field
- Current/Next GPU field lifecycle
- 4-neighbor local propagation
- Read Neighbors / Write Self
- Liquid/Gas as actual pressure media
- EMPTY/Void/Static/Powder are not hidden pressure media
- no arbitrary time decay
- pressure can leave when hosting Matter vents into Void
- finite/non-negative long-run behavior
- production RTX 5090 + DX12 execution verified

**Decision: G5-A = TECHNICAL PASS / FROZEN at validated commit `c8fcb5e1c8106f6c67f57eba1c31bd256de14818`.**

Next sub-gate: **G5-B — Phase Expansion / Confinement → Pressure Generation**.
