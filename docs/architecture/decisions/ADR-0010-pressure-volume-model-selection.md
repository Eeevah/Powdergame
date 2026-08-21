# ADR-0010: Pressure-volume model selection

- **Status:** Proposed — DESIGN BLOCKED / comparison evidence incomplete
- **Decision owner:** user
- **Program authority:** D-022
- **Runtime:** not started
- **Preserves:** ADR-0007, ADR-0008 and ADR-0009 as blocked history

## Context

TE-5B's same-tick token did not conserve capacity across ticks. TE-5C's local
proportional sharing discarded usable capacity and conflated capacity with
venting. TE-5D's persistent ontology was not disproved, but its fixed-depth-six
matching contract produced false pressure on a legal longer augmenting path.
Increasing the constant is forbidden as an architecture repair.

D-022 therefore compares exactly three representations under one common
contract. This ADR records the comparison candidate; it accepts none.

## Common contract

Every candidate preserves one phase-family Cell as one Water-equivalent
quantity, 1:1 transitions, accepted TE-3 local H/phase energy, no extra Steam,
finite reversible pressure, no open-boiling false rupture, sealed-headspace
pressure, rupture-created relief, condensation relief, exact reset/staging and
separate derived-Air/gauge meanings. Historical TE-2/G5 evidence stays bound to
its original source.

The phase-vapor fraction is common:

```text
Ice or Water(E <= 0): r = 0
Water(0 < E <= Lv):   r = E / Lv
Steam:                r = E / Lv
Lv = 480
```

Out-of-range phase energy is an invariant failure, never silently clamped.

## Candidate A — exact persistent-extent maximum matching

Candidate A retains ADR-0009's reciprocal source/EMPTY extent and dedicated
phase-pressure Current/Next state, but deletes every fixed reassignment depth.
An edge exists only when the target is in the accepted Steam movement domain,
is unowned EMPTY and its TE-1 Environment receiver transaction is feasible.
Movement, density swap, condensation, Void, Draw/Erase/reset and reciprocal
cleanup remain mandatory.

Each matching epoch starts by validating retained reciprocal pairs, removes
invalid pairs transactionally, and computes maximum cardinality from that
arbitrary legal starting matching. Layered alternating-frontier search may use
proposal and claim as fully overwritten `u32` frontier/predecessor storage
after Smoke. A deterministic lowest-index choice selects vertex-disjoint paths;
one atomic flip either commits every edge and Air transaction or none. The
epoch is certified only when no augmenting path remains.

Solver progress is not confinement. New phase pressure is held at zero, and
existing phase pressure is held rather than increased, until certification.
Only sources proven unmatched after certification receive target 100. A
complete matching therefore never reaches Wood threshold because work is
unfinished.

The production-work contract is finite but variable. For world Cell count
`N`, at most `N` augmenting phases are permitted; each phase has at most
`2N-1` frontier layers plus deterministic selection/flip/validation. The hard
dispatch bound is:

```text
A_delta_passes <= 7 + N * (2N + 3)
A_total_passes <= 47 + N * (2N + 3)
timestamp queries <= 2 * A_total_passes
```

Crossing the bound without a certificate is a validation failure and produces
no new physical pressure. This is not a real-time claim: at 2048² the bound is
prohibitively large and future source-bound evidence must replace it with a
mechanically exact bounded protocol before Candidate A could be selected for
runtime.

## Candidate B — shared connected gas-chamber capacity

Candidate B has no per-Steam extent. The four-neighbour gas-accessible graph
contains in-domain EMPTY and registered GAS Matter. Static, Powder and Liquid
block connectivity. A partial Water Cell is not a graph node; its `r` is split
across distinct orthogonally adjacent components in proportion to their EMPTY
counts. If all adjacent counts are zero, all demand is assigned to the
lowest representative. The weights sum exactly one, so demand is not
duplicated. Gas phase demand belongs to its own component.

For component `C`:

```text
free(C)        = eligible EMPTY count
demand(C)      = assigned phase-vapor fraction sum
compression(C) = max(0, demand(C) - free(C)) / max(demand(C), epsilon)
target(C)      = 100 * compression(C)
```

The predeclared response comparison rejects binary-any-deficit because it is
discontinuous and a smooth quadratic because it delays the named Wood-scale
consequence. The linear bounded response above is the comparison formula.

Connectivity is an equilibrium chamber meaning: a narrow neck can expose
distant capacity to the target calculation in one recomputation. Actual phase
pressure does not collapse in one tick:

```text
p_next = p_current + 0.10 * (target(C) - p_current)
```

Pressure remains dedicated and finite; ordinary gauge pressure remains
separate until rupture stress combines them once. Atmosphere and Vacuum differ
only in later background-pressure work, not chamber capacity.

The conservative clean-room GPU projection uses hooking/pointer-jumping CCL
with a fixed `4*ceil(log2 N)+2` pass envelope, stable radix grouping of
`(component_label, Cell_index)`, a deterministic Cell-index-order segmented
sum, broadcast, phase-pressure update, separate activity and settle. An exact
fallback label-propagation certificate has the larger `N-1` bound; exceeding
the logarithmic envelope makes the optimized implementation fail closed to the
fallback/evidence gate, not guess a label.

Two existing `u32` proposal/claim buffers carry label Current/Next after Smoke.
Deterministic grouping adds two temporary full-world `vec4<u32>` buffers; the
tuple stores label, Cell index, EMPTY count and `f32` demand bits without
quantization. Dedicated phase pressure adds one Current/Next `f32` pair.
Candidate B does not alias Air and does not mutate EMPTY.

At `N=2048²`, a static projection is 90 CCL passes, 32 stable radix passes, 22
segmented-reduction levels and 4 update/activity/settle passes: delta 148,
total 188 passes and 376 timestamp queries. Every pass is a separate named
profiler identity. This is a conservative architecture bound, not measured
performance.

## Candidate C — conservative Vapor-volume Environment field

Candidate C adds `vapor_volume_current/next` and dedicated phase pressure.
Positive phase-demand delta sources non-negative volume at a gas-accessible
node; TE-2-style donor scaling and Current/Next movement conserves it through
EMPTY/GAS nodes with capacity one. Steam motion does not source it again.

The required inverse is not closed. Once volume has moved away, a later local
negative phase-demand delta can exceed the scalar at the condensing Cell.
Allowing a negative debt violates the field contract; clipping leaves orphan
volume; storing an owner/debt creates unaccounted persistent state; withdrawing
from the connected region becomes Candidate B. Candidate C is therefore the
pre-run rejection candidate if the combined proof reproduces this witness.

## Comparative resource projection at 2048²

Accepted TE-3 remains 40 passes/80 queries before the candidate delta.

| Candidate | Added persistent state | Added temporary full-world scratch | 2048² added bytes | Pass/query contract | Max storage bindings |
|---|---:|---:|---:|---|---:|
| A exact matching | reciprocal link + phase pressure Current/Next: 16 B/Cell | frontier/predecessor reuse proposal/claim | 67,108,864 B | variable finite; formula above | 8 |
| B shared chamber | phase pressure Current/Next: 8 B/Cell | two `vec4<u32>` grouping buffers: 32 B/Cell; labels reuse proposal/claim | 167,772,160 B | projected 188 / 376 | 8 |
| C conservative field | Vapor volume + phase pressure Current/Next: 16 B/Cell | TE-2-style scale reuses authorized scratch pattern | 67,108,864 B | projected 46 / 92 | 8 |

At 256² the same increments are A `1,048,576` B, B `2,621,440` B and C
`1,048,576` B. B's individual 64 MiB `vec4` buffer is below wgpu 26's default
128 MiB storage-binding limit. No allocation exists in runtime.

Representative maximum binding layouts are:

| Pass | Storage RO | Storage RW | Total |
|---|---:|---:|---:|
| A frontier/flip | 5 | 3 | 8 |
| A receiver/reciprocal commit | 5 | 3 | 8 |
| B hook/shortcut | 3 | 2 | 5 |
| B radix/group/reduce | 4 | 2 | 6 |
| B pressure broadcast/update | 5 | 2 | 7 |
| B separate phase-volume activity | 4 | 1 | 5 |
| C donor scale/commit | 5 | 3 | 8 |

Base activity is not enlarged because it already binds eight storage buffers.
Each candidate requiring phase-volume work uses a separately counted activity
pass and wake halo. Sleeping is permitted only after no solver frontier,
transport delta, pressure relaxation, cleanup or phase work remains.

## Predeclared selection rule

The ineligibility list and rank order are exactly D-022 and the task contract.
Before execution, the provisional comparison expected B to rank first if its
component arithmetic and finite-rate relief pass; A remains an exact but costly
fallback; C is rejected if the condensation witness holds. The one-shot result
and independent review may invalidate that ranking. Any unresolved
Critical/High removes all recommendation and makes TE-5X DESIGN BLOCKED.

The one permitted execution stopped at the NetworkX version guard before any
candidate ran. A `networkx` namespace module was found without `__version__`;
candidate evaluations and generated cases are all zero. The provisional rank
is therefore void. No candidate is Recommended, Retained fallback or Rejected
by evidence in this task. See the
[validation receipt](../../development/PRESSURE_VOLUME_MODEL_COMPARISON_VALIDATION.md).

## Consequences and deferrals

ADR-0010 is Proposed / DESIGN BLOCKED. It does not accept a model, allocate buffers, change
runtime or authorize TE-3/TE-5. Background Air pressure, structure
differential, product world edge, Vacuum combustion, TE-4 and later G9 gates
remain separate. The next authorized action after comparison is user model
direction on a new evidence identity or architecture scope, not implementation.
