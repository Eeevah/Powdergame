# ADR-0002 — GPU-authoritative Local Simulation

- **Status:** Accepted
- **Date:** 2026-08-15

## Context

Powdergame wants a large world with millions of cells and many simultaneous interactions. A CPU-centric ordered update model would fight the core performance goal. At the same time, uncontrolled GPU writes between neighboring cells would create races and state corruption.

The design therefore needs to maximize local independence while preserving spatial ownership invariants.

## Decision

### GPU Production is authoritative

The production world state lives on the GPU. CPU is responsible for commands, configuration, orchestration, small reference simulations, save/inspection support and diagnostics.

### Read Neighbors, Write Self

General Matter and Field rules read local Current state and write only their own Next state.

```text
read self + local neighbors
→ cheap local rule
→ write self next
```

### Spatial ownership changes use minimal resolve

Movement, swap, multi-cell spawn and other ownership changes use:

```text
Propose
→ Claim / Resolve
→ Commit
```

but general reactions do not automatically enter a heavy resolve pipeline.

### Locality

- Matter interaction: maximum default neighborhood = 8-neighbor
- Field propagation baseline = 4-neighbor
- Movement uses behavior-specific stencil and First-Match

### Loose causal phases

Not every causal effect must be visible in the same tick. One-tick delay is acceptable when natural and it avoids unnecessary barriers. State integrity is never delayed.

## Consequences

### Positive

- high GPU parallelism
- fewer multi-writer races
- fewer atomics/global synchronization points
- simpler hot path
- local rules scale better with world size

### Tradeoffs

- some physical cause/effect chains may resolve one tick later
- exact pairwise conservation is not guaranteed
- ownership-changing actions need a dedicated local arbitration path

## Rejected alternatives

### CPU authoritative full-world simulation

Rejected because it undermines the main performance strategy.

### Neighbor-direct writes for general reactions

Rejected because multiple threads can target the same cell and force broader synchronization.

### Strong global phase barriers for all physics

Rejected because same-tick exact causality is not worth the likely synchronization/memory-pass cost.

## User provenance

The user repeatedly emphasized that the game should avoid a CPU processing cells sequentially and instead exploit the fact that most cells only interact with nearby cells. The user explicitly selected `Read Neighbors, Write Self` and loose causal phases.

See `docs/design-history/2026-08-15-foundation-design-session.md`.
