# TE-5D Persistent Vapor Extent and Dedicated Phase Pressure

- **Program:** authorized by D-021
- **Candidate:** proposed ADR-0009
- **Runtime:** not started
- **Current stop:** TE-5D DESIGN BLOCKED; user architecture revision required

## Reuse-first boundary

TE-5D reuses accepted TE-3 phase energy, Current/Next settlement, movement
ownership, whole-parcel Environment receiver, TE-2 Air/thermal predicates,
generic gauge pressure, Wood threshold 80, proposal/claim scratch,
activity/wake and profiler contracts. It imports no external implementation
and adds no general fluid solver.

The blocked histories remain evidence of what not to repeat:

- TE-5B same-tick token did not own capacity across ticks;
- TE-5C stateless sharing could underuse a complete assignment and mixed
  capacity with EMPTY venting.

## Locked candidate

One logical Current/Next pair stores reciprocal link plus dedicated phase
pressure. A Steam extent is a real EMPTY Cell with zero Air, blocked to all
non-owner movement and Air/thermal flow. Whole-parcel Air displacement must
succeed before acquisition. Water/Steam quantity stays 1:1 and the extent is
not Matter.

Movement relocates an extent to the zero-Air vacated source for ordinary EMPTY
movement, keeps and re-backlinks it for density swap, and releases it on
condensation or Void exit. Generic and phase pressure remain separate until a
single rupture-stress sum.

## Frozen matching and pressure experiment

- approved targets: the five in-domain EMPTY GAS positions;
- algorithm: scarcity/age/source order plus atomic augmenting paths;
- `MAX_REASSIGNMENT_DEPTH=6`;
- `MATCH_SETTLE_TICKS=6`;
- phase-pressure relaxation `0.10`;
- diffusion `0.025`;
- compressed equilibrium `100`.

The asymmetric two-source fixture must match fully. More importantly, every
complete matching in the approved domain must settle before Wood rupture could
be caused by matching delay. A longer alternating path is a correctness
counterexample, not a tuning opportunity.

## Static feasibility projection

| Item | Projection |
|---|---:|
| added persistent bytes / Cell | 16 across both halves |
| 256² increment | 1,048,576 B |
| 2048² increment | 67,108,864 B |
| TE-3 + TE-5D 2048² no-profiler tracked total | 369,125,680 B |
| projected passes | 62 |
| projected timestamp queries | 124 |

Movement and Environment reconciliation need a fully written phase-volume
context because the existing Environment reconcile is already at the storage
binding ceiling. Air-flow and thermal scale passes can read the link mask;
their commit passes rely on zero transfer and canonical zero-Air state.
Rupture stays at eight storage bindings only if its two material lookup tables
are combined into one descriptor binding. These are future implementation
requirements, not achieved evidence.

## User review checklist

- accept or revise reciprocal EMPTY extent ownership and target Air mutation;
- accept or revise the exact encoding and saturated compressed age;
- accept or revise relocation for EMPTY movement and keep/re-backlink for swap;
- accept or revise the five-position target domain;
- evaluate the hard matching result without post-result retuning;
- accept or revise dedicated phase-pressure coefficients and memory cost;
- review the finite boiler product meaning and atomic TE-3/TE-5D boundary.

Full background-Air pressure, structure differential, product edge mode,
Vacuum combustion, runtime implementation and source-bound performance remain
separate future decisions.

## Stop condition

The one-shot proof and fresh independent review decide whether the candidate
can proceed to user architecture review. Any unresolved Critical/High stops
TE-5D DESIGN BLOCKED and runtime remains not started.

## One-shot outcome

The proof executed once at seed `0x54453544` and returned `DESIGN_BLOCKED`.
The fixed-depth model passed its fresh-start sampled checks, but did not meet
the mandated all-labeled 6×6 enumeration. It also omitted arbitrary existing
persistent matches. In the canonical eight-source alternating chain recorded
by ADR-0009, a complete matching exists but requires an augmenting path beyond
the frozen depth six. Repeated atomic retries make no progress and permit false
rupture-capable phase pressure.

The design is therefore blocked on **wider matching scope**. Another
persistent field, relaxation of 1:1 quantity and a different volume
representation are not required by this witness. An efficient fixed-pass GPU
solution may require an additional full-world frontier/predecessor scratch;
that remains a user-owned architecture choice rather than an implicit change.
