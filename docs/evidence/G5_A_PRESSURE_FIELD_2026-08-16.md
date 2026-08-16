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

## Validation Summary

```text
WGSL parser regression: 1 passed; 0 failed
G5-A pressure GPU tests: 8 passed; 0 failed
Full GPU integration: 133 passed; 0 failed; 1 ignored
Core tests: 121 passed; 0 failed
cargo check --workspace --all-targets: PASS
git diff --check: clean
```

Validated G5-A behaviors:

- scalar `f32` Pressure field
- Current/Next GPU field lifecycle
- 4-neighbor local propagation
- Read Neighbors / Write Self
- Liquid/Gas are actual pressure media
- EMPTY/Void/Static/Powder are not hidden pressure media
- sealed pressure has no arbitrary time decay
- chunk-boundary propagation works
- pressure leaves when its hosting Matter vents into Void
- finite/non-negative long-run behavior
- non-finite authored pressure is rejected
- production RTX 5090 + DX12 execution verified

The user's existing `docs/planning/MATERIAL_CANDIDATES.md` work was preserved by the local validation run.

**Decision: G5-A = TECHNICAL PASS / FROZEN at validated commit `c8fcb5e1c8106f6c67f57eba1c31bd256de14818`.**

Next sub-gate: **G5-B — Phase Expansion / Confinement → Pressure Generation**.
