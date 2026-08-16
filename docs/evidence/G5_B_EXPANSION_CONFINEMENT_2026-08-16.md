# G5-B Expansion / Confinement → Pressure — Validation Evidence

Date: 2026-08-16
Status: **TECHNICAL PASS / FROZEN**

## Tested revision

- Branch: `feature/m0-g5-expansion-confinement`
- Tested HEAD: `2eb9c98eb1dfbc89f60af649b5d02d765cafcc39`
- G5-B implementation commit: `1d6f5c13fe99e88b8da88a25e54561423ce2ff0e`
- Validation worktree: `C:\Users\mdkap\source\repos\Powdergame-g5b-validation`

## Reference hardware

- OS: Windows 11
- Adapter: NVIDIA GeForce RTX 5090
- Vendor: `0x10DE`
- Backend: `wgpu::Backend::Dx12`
- `verify_target_hardware()` enforced the RTX 5090 + DX12 reference target for GPU tests.

## G5-B contract validated

Phase transition effects are Material-rule data, not boiler-specific code:

- `target_material`
- `matter_yield`
- `blocked_pressure`

Baseline rules:

- Water → Ice: yield 1, blocked pressure 0
- Ice → Water: yield 1, blocked pressure 0
- Steam → Water: yield 1, blocked pressure 0
- Water → Steam: yield 2, blocked pressure 100

Causal chain validated:

```text
Water heated above boil threshold
→ Water becomes Steam
→ one extra Steam cell requested locally
→ Proposal → Claim/Resolve → Commit
→ if destination succeeds: extra Steam, no confinement Pressure
→ if blocked or ownership competition loses: no extra Matter, Pressure +100 at source
→ existing G5-A Pressure propagation handles the resulting scalar field
```

No permanent per-cell expansion buffer was added. Existing `proposal[]` / `claim[]` scratch is reused. G5-C rupture/stress/vent logic remains out of scope for this frozen gate.

## GPU-free WGSL regression

Command:

```text
cargo test -p powdergame-gpu --test wgsl_parse -- --nocapture
```

Result: **1 passed, 0 failed**.

## G5-B RTX 5090 GPU tests

Command:

```text
cargo test -p powdergame-gpu --test expansion -- --nocapture --test-threads=1
```

Result: **5 passed, 0 failed**.

Validated tests:

1. `boiling_with_space_spawns_second_steam_without_pressure`
   - additional Steam is created when local space exists;
   - confinement pressure remains 0;
   - temperature/state are preserved.
2. `fully_confined_boiling_generates_pressure_instead_of_extra_matter`
   - no extra Matter is created in a fully blocked chamber;
   - source receives exactly `WATER_BOIL_BLOCKED_PRESSURE = 100.0`.
3. `competing_expansions_have_one_winner_and_loser_becomes_pressure`
   - exactly one deterministic ownership winner;
   - losing source converts unmet expansion to confinement Pressure.
4. `expansion_can_cross_a_64_cell_chunk_boundary`
   - chunk boundaries are not simulation walls.
5. `one_to_one_phase_transition_creates_no_expansion_pressure`
   - yield-1 transitions do not request extra Matter or generate confinement Pressure.

## G5-A regression

```text
cargo test -p powdergame-gpu --test pressure -- --nocapture --test-threads=1
```

Result: **8 passed, 0 failed**.

The frozen G5-A semantics remain intact: scalar spatial pressure, Liquid/Gas media only, no arbitrary time decay, finite/non-negative field, chunk-boundary propagation, Void release, and stale-pressure clearing.

## Phase regression

```text
cargo test -p powdergame-gpu --test phase -- --nocapture --test-threads=1
```

Result: **16 passed, 0 failed**.

Freeze/melt/condense/boil thresholds, hysteresis, temperature preservation, MovementClass adoption, chunk-boundary behavior, and GPU execution remain valid.

## Full GPU regression

```text
cargo test -p powdergame-gpu --tests -- --nocapture --test-threads=1
```

Result: **138 passed, 0 failed, 1 ignored** (`controlled_reference_world_perf`, intentionally ignored).

Suite totals:

- Combustion: 56 passed
- Density: 15 passed
- Expansion: 5 passed
- Headless smoke: 1 passed
- Movement: 16 passed, 1 ignored
- Phase: 16 passed
- Pressure: 8 passed
- Thermal: 13 passed
- WGSL parse: 1 passed
- World integrity: 7 passed

## Core / workspace regression

- `cargo test -p powdergame-core`: **125 passed, 0 failed**
- `cargo check --workspace --all-targets`: **PASS, 0 errors, 0 warnings**
- `git diff --check`: **clean**
- Validation worktree: **clean**
- Original working tree user file `docs/planning/MATERIAL_CANDIDATES.md`: preserved exactly as found (`??` untracked); no reset/clean/overwrite performed.

## Gate conclusion

**G5-B Expansion / Confinement → Pressure is TECHNICAL PASS / FROZEN on the reference RTX 5090 / DX12 machine.**

This evidence advances the G5 chain through:

```text
Water heated
→ Steam transition / expansion request
→ space insufficient
→ Pressure generated
→ local propagation
```

The remaining G5-C gate is:

```text
Pressure
→ weak structure stressed
→ rupture threshold exceeded
→ opening created
→ venting
```
