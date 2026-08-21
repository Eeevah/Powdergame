# ADR-0008 — Local Vapor Capacity Share and Gauge-Pressure Equilibrium

- **Status:** PROPOSED — DESIGN BLOCKED / architecture revision required
- **Date:** 2026-08-21
- **Decision:** D-020
- **Design baseline:** `6a1c83fad702d18f2d24365a4fc747ab74225f5c`
- **Supersedes for design selection:** ADR-0007's exclusive completion token
- **Runtime:** TE-3 / TE-5C / full TE-5 NOT STARTED
- **External implementation copied, translated or vendored:** `0 files / 0 lines`

## Context

ADR-0006 preserves one Water-equivalent Cell and Matter-owned phase enthalpy.
ADR-0007 tried to preserve the frozen G5 pressure chain with a non-mutating
same-tick EMPTY token. Its staggered one-column witness showed that ordinary
1:1 Steam movement only relocates the vacancy; same-tick exclusivity cannot
consume finite headspace across ticks. D-020 rejects that token and authorizes
one final no-new-persistent-state replacement.

## Decision under evaluation

Evaluate **LOCAL VAPOR CAPACITY SHARE + GAUGE-PRESSURE EQUILIBRIUM**.

The candidate derives additional Vapor volume demand from accepted phase
state, recomputes current local EMPTY capacity every tick, and raises the
existing gauge field toward a bounded state target. It creates no completion
token, reservation owner, extra Steam, phase quantity or target mutation.

For one phase-family Cell `i`, with `Lv=480`:

```text
Ice or Water E<=0: r_i = 0
Water 0<E<=Lv:     r_i = E/Lv
Steam:             r_i = E/Lv
```

Values outside accepted identity ranges are invariant failures, not clamped
evidence. Every in-domain radius-1 Chebyshev EMPTY supplies capacity one. For
EMPTY `e`:

```text
D_e = sum adjacent phase j of r_j
a(e,i) = r_i / max(1,D_e)
capacity_i = min(r_i, sum adjacent EMPTY e of a(e,i))
deficit_i = max(0,r_i-capacity_i)
compression_i = deficit_i/r_i when r_i>0, else 0
target_i = 100 * clamp(compression_i/0.5,0,1)
```

The law guarantees each EMPTY contributes aggregate capacity at most one. It
does not claim a maximum matching or globally optimal allocation. Whether its
proportional local allocation underuses reachable capacity is a predeclared
blocking attack, not permission to replace the formula after proof.

## Pressure boundary

The candidate retains `pressure[]` as non-negative gameplay gauge
overpressure. Generic non-family expansion remains a separate event source.
For a pressure-medium Cell:

```text
p_source = max(sanitize(p_current), phase_pressure_target)
p_next = p_source
       + 0.20 * sum(pressure-medium neighbor - p_source)
       + 0.20 * sum(orthogonal EMPTY face 0 - p_source)
```

Static/Powder/Boundary and Void do not exchange pressure. Four participating
faces give a maximum explicit coefficient sum `0.8`. An isolated sealed
pressure medium therefore retains pressure; a real EMPTY opening vents it.
Atmospheric and Vacuum EMPTY both have gauge baseline zero. Derived Air
pressure, structure face differential and background-pressure force are not
added. Extending EMPTY venting to generic gauge pressure changes the future
source contract and requires new evidence; historical G5 receipts are not
rebound.

## TE-3 integration

With a future atomic TE-3/TE-5C source, accepted initiated or completion-ready
Water reaching `E=Lv` completes 1:1 to Steam without a token or immediate
impulse. The resulting Steam contributes `r=1`. Partial positive-E Water
contributes continuously before completion. Buried initiated/ready Water may
complete through this explicit current-state pressure-volume transaction;
canonical buried `E=0` Water still cannot initiate by it. H and family quantity
remain exact.

## Options and rejection boundary

| Option | State | Main property | Disposition |
|---|---|---|---|
| TE-5B completion token | no new state | one winner per tick, no cross-tick capacity | rejected / blocked |
| completion impulse | no new state | false pressure in open boiling | rejected |
| direct two-hop gather | no new pass | same law, high repeated reads | comparison only |
| scratch-reuse capacity sum | one reused scratch lifetime | explicit `D_e`, one new pass | primary layout |
| persistent capacity/volume state | new state | can own history/capacity | forbidden in D-020 attempt; next decision if blocked |

No third token or impulse variant may replace this candidate. A failed locked
proof means the next decision must explicitly permit persistent phase-volume
state.

## Static GPU feasibility

The existing proposal scratch is dead only after the Smoke transaction and its
Environment reconcile. The future order is:

```text
... Smoke proposal/claim/commit/reconcile
-> settle Matter/Environment
-> vapor_capacity_sum fully overwrites proposal as f32 D_e
-> pressure reads neighboring D_e and writes pressure_next
-> pressure settle
-> rupture and settle
-> base activity
-> phase activity reads still-live D_e as its eighth storage binding
-> Environment activity
-> activity reduce
-> next tick movement fully overwrites proposal as u32
```

Projected TE-3D plus TE-5C cost: 41 timestamped passes, 82 queries, two
656-byte profiler buffers (`1,312` B total), no new persistent/full-world
allocation and no new scratch allocation. Proposed storage counts are:

| Pass | RO | RW | Total |
|---|---:|---:|---:|
| `vapor_capacity_sum` | Material + phase energy + chunk state | proposal | 4 |
| pressure with capacity target | Material + phase energy + proposal + pressure + class + chunk state | pressure Next | 7 |
| phase activity with capacity predicate | existing 6 RO + proposal | activity proposal | 8 |

These are static projections. Naga, device, actual allocation, sleep and
performance evidence do not exist.

## Consequences and approval boundary

Potential benefits are current-population accounting, smooth partial demand,
no event replay and causal EMPTY venting. Risks include proportional-allocation
underuse, false pressure in locally open geometry, dense-cloud broad pressure,
generic-pressure vent regression and a pressure target that acts as a floor
until an actual vent/movement route exists.

The locked one-shot grid/time proof failed the predeclared
`reachable_capacity_no_false_pressure` control. With Steam B `(0,1)`, Steam A
`(1,1)`, shared EMPTY `(0,0)` and A-only EMPTY `(2,1)`, a complete assignment
exists. The proportional law gives B only `0.5`, gives A `1.5` before its
per-Cell cap, discards A's excess and creates target `100` at B. Each EMPTY's
aggregate contribution is still exactly one, so VC-INV-003 alone cannot
prevent the false pressure. VC-INV-008 is unsatisfied.

ADR-0008 remains **PROPOSED — DESIGN BLOCKED / architecture revision
required**. D-020 requires the next architecture decision to explicitly permit
persistent phase-volume state. The failed law is preserved; no matching,
redistribution, radius or curve replacement is selected here.

Fresh-context review left Critical `0` / High `6`. In addition to the locked
sharing counterexample, it found that occupancy-only EMPTY conflates finite
headspace with an infinite vent reservoir; the shared gauge field cannot shed
only phase-origin pressure after condensation; radius-1 capacity includes
downward space unavailable to GAS movement; the projected activity pass lacks
a coherent pressure/snapshot/binding path; and the one-shot receipt overstates
several unimplemented checks. The review is preserved in
[`TE5_LOCAL_VAPOR_CAPACITY_PRESSURE_DESIGN`](../../adversarial-reviews/TE5_LOCAL_VAPOR_CAPACITY_PRESSURE_DESIGN.md).

This ADR authorizes no runtime, Rust, WGSL, Cargo, build, launch, TE-4,
G9-B/C/D/E, PR or main merge.
