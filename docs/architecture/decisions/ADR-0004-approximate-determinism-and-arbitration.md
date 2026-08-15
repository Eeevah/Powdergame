# ADR-0004 — Approximate Determinism and Cheap Arbitration

- **Status:** Accepted
- **Date:** 2026-08-15

## Context

A GPU falling-sand simulation can spend substantial complexity and synchronization to reproduce exactly the same microscopic outcome on every run. Powdergame's user explicitly values world scale, interaction density and speed more than bit-perfect replay.

At the same time, uncontrolled races are not acceptable. The design must distinguish harmless non-exactness from corrupted state.

## Decision

### Do not require bit-perfect replay

GPU floating-point approximation, parallel execution order and local valid-winner differences are acceptable.

> **Non-exact but stable.**

### Do not add intentional randomness by default

Variation that arises from GPU execution is not a reason to introduce a large RNG system.

### Use cheap local arbitration for spatial competition

When multiple Matter cells want the same destination, use a low-cost local tie-breaker such as a stateless coordinate+tick integer hash.

No per-cell RNG state is required.

### Preserve hard invariants

Non-exactness never excuses:

- multi-occupancy
- unexplained duplicate ownership
- invalid material state
- NaN/Infinity runaway
- memory safety errors

## Consequences

### Positive

- avoids expensive global ordering
- maintains high GPU parallelism
- avoids random-state memory traffic
- preserves the only determinism that matters for gameplay: understandable stable laws

### Tradeoffs

- microscopic pile/flow shapes may differ
- exact command replay cannot be the only Rewind mechanism
- tests must use invariants/semantic ranges rather than checksum-only validation

## Rewind consequence

Rewind stores actual world snapshots rather than relying only on replaying commands through a deterministic simulation.

## Rejected alternatives

### Full deterministic global ordering

Rejected because the likely barrier/ordering cost does not serve the product goal.

### Per-cell persistent RNG

Rejected as unnecessary state/memory traffic for basic arbitration.

### Fixed direction only

Kept as a benchmark baseline but not preferred because visible directional bias may accumulate. If cheap hash proves unexpectedly expensive on the reference GPU, this decision may be revisited with measured evidence.

## User provenance

The user explicitly selected the cheap stateless arbitration option after asking for a performance comparison with fixed directional priority. The user also repeatedly accepted approximate behavior as long as it remains intuitive and state-safe.

See `docs/design-history/2026-08-15-foundation-design-session.md`.
