# ADR-0003 — Game-Consistent Minimum Sufficient Physics

- **Status:** Accepted
- **Date:** 2026-08-15

## Context

Powdergame is not a scientific simulator. Real-world physics is valuable because players already understand many natural phenomena, but reproducing those equations exactly would make each cell expensive and limit the scale and number of interacting systems.

The user repeatedly emphasized two goals:

1. the world may contain fictional Matter and fictional laws as long as players can understand the cause/effect;
2. every individual operation must be extremely cheap so millions of cells and many systems can run together.

## Decision

Use **Game-Consistent Minimum Sufficient Physics**.

For each physical/gameplay phenomenon, represent only the minimum state and operations needed to produce understandable, useful behavior.

Examples:

```text
Buoyancy
→ integer Density Rank compare + local displacement

Heat
→ meaningful ΔT + cheap conductivity/heat-capacity transfer

Pressure
→ local ΔP + resistance/push/rupture

Electricity (future)
→ conductive? + strength/loss frontier

Radiation (future)
→ intensity + attenuation/blocking

Gameplay Light (future)
→ transmit/absorb/reflect + remaining intensity
```

The representation is chosen by need:

```text
continuous value needed → f32/proper numeric
ordering only           → integer rank
boolean only            → bit
few states              → small enum
```

## Conservation policy

Use cheap local conservation when useful, but do not maintain an exact global energy ledger.

> **로컬에서는 납득 가능하게, 글로벌에서는 회계하지 않는다.**

Game-specific sources/sinks are allowed.

## Consequences

### Positive

- much lower per-cell cost
- more systems can run simultaneously
- fictional Matter and laws are first-class
- simple rules can combine into emergent chains
- optimization can focus on actual bottlenecks

### Tradeoffs

- not scientifically exact
- exact physical conservation is not guaranteed
- some coefficients/ranks are game values rather than SI quantities

These are intentional tradeoffs.

## Rejected alternatives

### Implement real equations by default

Rejected because scientific fidelity is not the product goal and the cost would reduce world scale/interaction density.

### Make every property f32 for uniformity

Rejected because many properties only need ordering, boolean or a few discrete states.

### Over-compress everything from day one

Rejected because some systems such as Temperature/Pressure genuinely benefit from continuous state; representation should be benchmark-driven.

## User provenance

The user explicitly stated that real natural phenomena should be reflected when useful but do not need to be reproduced exactly; the game only needs its own understandable logic and that logic must run at minimal cost.

See `docs/design-history/2026-08-15-foundation-design-session.md`.
