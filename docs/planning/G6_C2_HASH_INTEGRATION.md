# G6-C2 — Stateless Edge Hash Production Integration

Status: **DECISION RECORDED / INTEGRATION PENDING**

Decision date: 2026-08-16

## Decision

**ADOPT STATELESS EDGE HASH** for production ownership arbitration.

Evidence basis:

- Frozen correctness baseline: `ea08f6605703bb42593f7e6a1fb5181c99909ca8`
- G6-C1 measured candidate: `3011eeda46dd498a8aff8b2d1a770596db2dcf7d`
- Fixed-index baseline produced 100% / 0% winner bias in mirrored horizontal, vertical, and diagonal contention.
- Stateless edge-hash candidate produced approximately balanced 49–51% winner distributions.
- Candidate movement endpoint agreement, exactly-one ownership, deterministic repeat, and collision fallback all passed.
- RTX 5090 / DX12 claim-only worst-case overhead was approximately `+0.0015 ms` (`+3.86%` claim-only), with sparse contention effectively unchanged. This is small enough to justify removing the severe fixed-index bias.

The decision is consistent with ADR-0004: prefer cheap stateless arbitration when it materially reduces directional bias without meaningful hot-path cost.

## Production integration architecture

Do **not** expand the existing shared 16-byte world `Params` uniform used by all production shaders. That would unnecessarily change the uniform ABI of every pass.

Instead add one dedicated 16-byte arbitration uniform shared only by the three ownership resolver passes:

```text
ArbitrationParams
- tick: u32
- padding: 3 × u32
```

The buffer is created once by `Simulation`, stored as a simulation-owned non-per-cell resource, and updated once at the beginning of every tick from the already-existing CPU `Simulation::tick_count`.

This is not persistent per-cell RNG state and does not add world-state memory proportional to cell count.

Recommended bindings:

- `movement_claim.wgsl`: new arbitration uniform binding after existing bindings
- `expansion_claim.wgsl`: new arbitration uniform binding after existing bindings
- `smoke_claim.wgsl`: new arbitration uniform binding after existing bindings

No proposer or commit pass needs the arbitration tick.

All three resolvers use exactly the same priority primitive:

```wgsl
fn edge_priority(source: u32, target_cell: u32, tick: u32) -> u32 {
    var h: u32 = source ^ (target_cell * 0x9E3779B9u) ^ (tick * 0x85EBCA6Bu);
    h = (h ^ (h >> 16u)) * 0x7FEB352Du;
    h = (h ^ (h >> 15u)) * 0x846CA68Bu;
    h = h ^ (h >> 16u);
    return h;
}
```

Winner rule:

```text
lowest edge_priority wins
hash collision → smaller source index wins
```

For movement, both endpoints must compare incident edges using the identical `(source, target, tick)` key so reciprocal ownership remains guaranteed.

## Tick semantics

- First submitted simulation tick uses seed `0`.
- Subsequent ticks use the current lower 32 bits of `Simulation::tick_count`.
- `u32` wrap is acceptable; this is an arbitration phase seed, not physical world time.
- All ownership subsystems within one simulation tick use exactly the same arbitration seed.

## Frozen scope

G6-C2 must not change:

- movement candidate ordering / local stencil
- density ranks or density-swap semantics
- phase thresholds / expansion yield / confinement pressure
- combustion behavior or smoke candidate ordering
- thermal or pressure physics
- rupture thresholds
- G5 fixtures
- world cell storage
- proposal/claim scratch layout or encoding
- G7 Active/Sleep

The change is limited to the tie-break policy inside existing Claim/Resolve passes plus the minimal tick-uniform plumbing required to supply the stateless seed.

## Required correctness validation

After integration, rerun and preserve all G6-A/B evidence. In addition prove production behavior directly:

1. `movement_claim` still forms reciprocal single ownership edges.
2. multiple sources → one destination still has exactly one winner.
3. density movement/swap contention remains safe.
4. expansion contention still spawns exactly one extra Matter and converts losers to the frozen pressure semantics.
5. smoke contention still spawns exactly one Smoke and preserves source state.
6. scratch reuse remains safe.
7. chunk-boundary contention remains identical to interior contention.
8. same world state + same tick seed is deterministic.
9. changing tick seed can change the valid winner without violating invariants.
10. production mirrored/translated contention no longer exhibits fixed-index 100/0 lock.

The existing Naga G6-A pass-contract test must be updated only for the new read-only arbitration uniform binding and must still report no unauthorized writable storage, no atomics, and no workgroup/global coordination.

## Required performance validation

C1 measured the hash ALU cost but did not include the production per-tick uniform update. Therefore C2 must measure end-to-end production before/after integration on RTX 5090 / DX12.

Use the same controlled release protocol for baseline `3011eeda46dd498a8aff8b2d1a770596db2dcf7d` and integrated HEAD.

Record:

- individual runs
- median full-tick time
- TPS
- absolute delta
- percentage delta

Also rerun the claim-only sparse/heavy C1 benchmark as a sanity check if the harness remains available.

A small low-single-digit full-tick cost is acceptable given the measured elimination of severe directional bias. A surprising regression substantially larger than the C1 estimate must stop closure and be investigated before G6 is marked complete.

## Closure rule

G6 may become `PASS / CLOSED` only after:

- G6-A remains TECHNICAL PASS
- G6-B remains TECHNICAL PASS
- G6-C1 measurement complete
- this G6-C2 ADOPT decision is implemented
- full regression and RTX 5090 / DX12 validation pass
- end-to-end production performance remains acceptable
- final G6 evidence is documented

No additional visual user-validation scene is required unless production behavior becomes visually suspicious; G6 is primarily an invariant/architecture gate.

Do not start G7 until G6 is closed.
