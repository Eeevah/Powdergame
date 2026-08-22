# ADR-0013 — Local Relaxing Phase-Load Pressure

- **Status:** PROPOSED — DESIGN BLOCKED / ARCHITECTURE REVISION REQUIRED
- **Date:** 2026-08-23
- **Decision:** D-035
- **Design baseline:** `769e687c04406016fe9d66c8496269b459f06d83`
- **Runtime:** TE-5 NOT STARTED
- **External implementation copied, translated or vendored:** `0 files / 0 lines`

## Context

TE-2, TE-3 and TE-4 are accepted production lines. TE-3 intentionally keeps
one Ice/Water/Steam Cell as one Water-equivalent quantity and creates no extra
Steam or phase-pressure impulse. The old G5 Water-yield-2 pressure receipt is
historical and source-bound.

TE-5B through TE-5Q tried to preserve exact volume ownership or exact capacity
certification. Their counterexamples are immutable: cross-tick vacancy reuse,
discarded reachable capacity, bounded matching depth, unbounded or incomplete
global solvers, conservative-field inverse ownership, and packet merge/
movement/pressure ambiguity. D-035 does not repair or select any of them. It
changes the product model: pressure is a finite-rate local potential driven by
current phase load, not a proof that every Steam quantity owns a distinct
EMPTY extent.

## Decision

Adopt for architecture review the candidate:

**LOCAL RELAXING PHASE-LOAD PRESSURE**

Architecture class:

```text
DERIVED AIR BACKGROUND
+ DISSIPATIVE LOCAL DYNAMIC PRESSURE
```

No new persistent field is added. Existing `pressure_current/next` becomes a
bounded, non-negative, spatial dynamic mechanical pressure. Air background is
derived when needed from authoritative Air energy:

```text
P_air = air_energy_current / STANDARD_AIR_ENERGY
        for canonical positive Air
P_air = 0 for exact Vacuum
P_total = P_air + P_dynamic
```

The two terms are never precombined into another buffer and may appear only
once in Air transport, Matter movement bias and structure stress.

## Topology and phase load

Dynamic-pressure nodes are in-domain EMPTY Cells—including Atmosphere,
LowPressure and exact Vacuum—and Liquid/Gas Matter. Static, Powder and Void
are blocked. Boundary Block and Stone therefore block connectivity. Vacuum
may carry dynamic pressure propagated from a connected source without claiming
that Air mass exists there; canonical isolated Vacuum starts at and remains
zero.

With accepted `Lv = 480`:

```text
Steam: r = phase_energy / Lv
Water: r = phase_energy / Lv only while the accepted current gas-facing
       boiling context is runnable
Ice, buried/non-runnable Water, other Matter and EMPTY: r = 0
P_phase_target = 100 * r
```

`0 <= r <= 1` is an invariant. Invalid phase state fails validation and is not
clamped into evidence. Gas-facing Water at `E=Lv` and canonical Steam at
`E=Lv` both target 100, so identity completion has no pressure discontinuity
or impulse. Buried ready-Water targets zero until the accepted surface context
returns.

## Local update and equilibrium meaning

The initial architecture constants are:

```text
D = PRESSURE_DIFFUSION_RATE = 0.20
R = PHASE_RELAXATION_RATE   = 0.02
P_FULL_VAPOR                = 100
PRESSURE_MAX                = existing 1.0e6
GENERIC_IMPULSE_MAX         = 100
```

For a pressure node:

```text
p_next = clamp(
    p_current
    + D * sum_pressure_neighbors(neighbor_p - p_current)
    + R * (P_phase_target - p_current)
    + bounded_generic_impulse,
    0,
    PRESSURE_MAX)
```

Blocked and invalid nodes write zero. Missing/out-of-domain faces are no-flux.
`4D + R = 0.82 <= 1`, and every smaller participating degree has a
non-negative retained coefficient. With no impulse or reservoir, summing the
unclamped equilibrium equations over a sealed connected component cancels
every symmetric diffusion edge, giving `average(p)=average(target)`. Thus a
half-vapor-loaded component approaches about 50, every node fully loaded
approaches 100, and one loaded Cell among `N` nodes has component average
about `100/N`. Local values need not equal that average; finite-rate local hot
spots are an explicit product-review risk.

When load disappears, target zero makes stored dynamic pressure relax toward
zero. This supersedes the historical no-decay G5 field meaning. Dissipation is
part of the candidate model, not evidence that a physical vent occurred.

## Edge, force and stress decisions

The product/default edge is **SEALED / NO-FLUX**. Void is not a hidden standard
Atmosphere. A fixed standard-Atmosphere reservoir exists only as a future
fixture boundary: it clamps Air to standard state and dynamic pressure to zero
while explicitly accounting external mass, energy and pressure exchange. A
product reservoir mode needs another decision. Vacuum combustion remains
disabled under the accepted TE-4 positive-Air-face rule.

Tick `N` movement and Air transport read `P_total_current`. Air transport uses
the accepted conservative donor/receiver transaction with its raw face demand
computed from the total-pressure drop. Matter pressure response adds no
velocity or new target: within each already-legal multi-candidate stage of the
existing Liquid/Gas stencil, the candidate with the largest positive total-
pressure drop above the existing Air pressure deadband wins; parity remains
the exact tie-break. The ordinary vertical singleton and legality/density rules
remain unchanged.

Pressure is spatial and does not move with Matter. Movement leaves the old
Cell's dynamic value in place and enters the destination's prior value. After
movement, thermal and phase settle, the phase source is evaluated at the
identity's new Cell; the pressure pass then diffuses/relaxes the spatial field.
This is intentional finite-rate wake behavior, not owner loss.

Pressure settles before rupture. A structure reads total pressure on four
faces and uses:

```text
stress_x = abs(P_left - P_right)
stress_y = abs(P_up - P_down)
stress   = max(stress_x, stress_y)
```

Blocked or out-of-domain faces contribute zero under the current finite-world
fixture convention. Equal pressure on both sides cannot rupture. Rupture
creates a real EMPTY pressure node; Air and Matter use the opening on following
ticks. There is no same-tick rollback.

## Generic impulse boundary

Current Water/Ice/Steam descriptors remain yield 1 with blocked pressure zero.
The existing generic expansion failure writers remain the only event-source
entry and add one descriptor-owned impulse at most once. Future descriptors
must provide a finite value in `[0,100]`; the local pressure pass then subjects
that stored value to the same diffusion and relaxation. No event is
reconstructed from later identity, and Environment-receiver failure cannot add
the same consequence twice.

Historical G5-A/B/C evidence is not rebound. A future TE-5 source must produce
new boil/load/pressure/rupture/vent evidence.

## Source-feasibility decision

Live source at the baseline has 42 timestamped passes, 84 queries, two
672-byte profiler buffers, and no spare binding in Air transport commit,
movement commit, base activity or rupture. The candidate remains feasible by
changing pass ownership rather than adding a ninth binding:

- `movement_propose` adds dynamic pressure and Air energy: 8 storage bindings;
- Air scale adds dynamic pressure: 7;
- the maxed Air commit splits into mass and energy self-writers: 8 each;
- local pressure adds phase energy: 6;
- rupture drops the redundant movement-class read, relies on the just-settled
  zero-on-blocked-node invariant, and adds Air energy: 8;
- Environment activity adds dynamic pressure: 6;
- a new dedicated pressure-activity proposal uses 5 bindings and augments the
  shared activity mask after the existing proposals.

The reviewed projection was **44 passes / 88 queries**, two 704-byte profiler
buffers (**1,408 B**), and no new persistent or full-world scratch allocation.
At 256²/2048² the projected tracked totals with profiler are respectively
`4,722,736 B` and `302,018,224 B`; no-profiler totals remain `4,721,328 B` and
`302,016,816 B`. Proposal/claim continue as Air donor/receiver scratch and are
fully consumed before thermal/phase/Smoke lifetimes overwrite them.

This arithmetic does not establish semantic source feasibility. The independent
review found that the proposed pass inputs cannot preserve the required
pre-transition Water surface snapshot or distinguish a fresh generic impulse,
and that the unchanged base activity pass keeps an exact nonuniform equilibrium
awake. It establishes no WGSL, device, allocation, race, performance, sleep or
product result.

## Consequences

Benefits:

- preserves accepted 1:1 phase quantity without volume ownership machinery;
- treats Atmosphere and Vacuum through one local pressure topology while
  retaining exact Air/Vacuum ontology;
- produces reversible load pressure and finite-rate opening response;
- keeps all work local, bounded and below the eight-storage ceiling;
- reuses the existing pressure pair and accepted activity/profiler mechanisms.

Costs and risks:

- local pressure is an approximation, not a capacity or compressible-fluid
  proof;
- a Vacuum Cell may hold pressure potential with zero Air mass;
- open-space local hot spots and the `D/R/100` product meaning remain untested;
- dissipation can lower pressure without a vent and may erase unrelated
  generic impulses faster than historical G5;
- pressure-biased cellular movement has no momentum and may create feedback or
  oscillation;
- fixed-reservoir and product-edge choices beyond sealed/no-flux remain open.

## Approval boundary

The fresh-context review found unresolved Critical `0`, High `3`, Medium `3`:

1. the projected local-pressure pass cannot access the required pre-transition
   TE-3 gas-facing snapshot;
2. its documented additive generic impulse is unavailable from the projected
   inputs and disagrees with the existing pre-settle transaction;
3. the unchanged base activity pass permanently wakes an exact nonuniform
   relaxing-pressure equilibrium.

ADR-0013 therefore remains **PROPOSED — DESIGN BLOCKED / ARCHITECTURE REVISION
REQUIRED**. The exact witnesses are preserved in the
[independent review](../../adversarial-reviews/TE5R_PRESSURE_VACUUM_REENTRY_DESIGN.md).
This task does not synthesize a replacement. It authorizes no Rust, WGSL,
Cargo, buffer allocation, proof script, GPU/FULL, build, launch or runtime.
