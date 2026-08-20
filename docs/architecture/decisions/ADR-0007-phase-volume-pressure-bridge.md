# ADR-0007 — Phase-Volume Pressure Bridge

- **Status:** PROPOSED — DESIGN BLOCKED / user architecture revision required
- **Date:** 2026-08-21
- **Decision owner:** user at the TE-5B architecture-review boundary
- **Authorization:** D-019
- **Design baseline:** `d7500e219af6f670be05f830b50c232d2bb53077`
- **Authorization source:** `f1ca48cc01a906bfb4a997c72bc2744b81546ccd`
- **Runtime status:** TE-3 NOT STARTED; TE-5B NOT STARTED; full TE-5 NOT STARTED
- **External implementation copied, translated or vendored:** `0 files / 0 lines`

## Context

ADR-0006 accepts one Water-equivalent phase-family Cell, 1:1
Ice/Water/Steam identity changes and Matter-owned phase enthalpy for future
atomic implementation. That design deliberately removes the historical extra
Steam spawn and its blocked-expansion pressure from the Water path. Activating
it alone would regress the frozen G5 causal chain:

```text
Water heating
-> Steam expansion / insufficient space
-> gauge Pressure
-> local propagation
-> weak-wall rupture
-> opening and venting
```

The missing bridge is not a general fluid or background-pressure model. It is
one narrow completion transaction that distinguishes a locally open
Water-to-Steam completion from a confined or contended one while preserving
the accepted 1:1 quantity model.

The current source already has the parts required to represent that
transaction: a deterministic proposal/claim owner domain, the bound
`cell_count < 1 << 30`, separate Environment receiver arbitration, an
exactly-once failed-expansion consequence pass, scalar gauge pressure,
pressure propagation and threshold-based rupture. The accepted TE-3 projection
also places the proposal/claim lifetime after TE-2 scratch use. Reuse is the
default; new persistent volume state is not justified.

## Decision drivers

The bridge candidate must:

1. preserve one phase-family Cell as one quantity and every phase transition as
   1:1;
2. produce no confinement pressure for a genuinely open local Steam movement
   route;
3. let one EMPTY Cell relieve at most one same-tick completion;
4. generate the frozen gameplay consequence `100.0` exactly once for a blocked
   or losing completion;
5. leave a winning relief target's Matter and Air byte-identical;
6. preserve historical generic non-family expansion, including its separate
   Environment receiver failure consequence;
7. retain current gauge-pressure meaning and defer derived-Air/background and
   structure-face coupling to full TE-5;
8. fit the existing projected 40-pass TE-3 graph and the DX12 eight-storage
   ceiling without a new buffer, full-world scratch or production pass;
9. remain data-driven, including a one-invocation Ice-to-Steam normalization
   that crosses the vaporization endpoint;
10. keep historical TE-1/TE-2/G5 evidence bound to its original source.

## Options considered

| Option | Open boiling | Same-target contention | State/cost | Disposition |
|---|---|---|---|---|
| A — pressure on every Water-to-Steam completion | False pressure even with clear headspace; can rupture a weak wall before Steam moves | Not relevant | Small | Rejected |
| B — any EMPTY means relief | Avoids some false pressure | Multiple sources can count the same EMPTY Cell | Small but non-exclusive | Rejected |
| C — volume fraction, fragments or dedicated full-world volume state | Can model more volume detail | Could be explicit | New ownership, buffers and solver surface | Rejected for this bridge |
| **D — exclusive local volume-relief token** | **Open completion wins zero-pressure availability** | **One deterministic winner per target** | **Reuses proposal/claim and pressure** | **Evaluated primary candidate; DESIGN BLOCKED** |

No other model is silently selected. A future general volume or compressible-
fluid model would require a separate decision and evidence.

## Evaluated primary candidate

The D-019-authorized candidate evaluated was:

**EXCLUSIVE LOCAL VOLUME-RELIEF TOKEN + EXISTING GAUGE PRESSURE**

When normalization reaches the vaporization endpoint, it may enter a TE-5B
**completion attempt** only if boiling was already initiated (`Water E > 0`,
including vaporization-ready `E = Lv`) or the current context is gas-facing and
this invocation initiates/crosses the endpoint. Buried canonical `Water E = 0`
does not start boiling, and non-gas-facing extreme Ice cannot bypass initiation.

Entering the TE-5B transaction is the explicit completion acceptance required
by ADR-0006. The acceptance function always returns one valid mode-tagged
request: either one EMPTY Cell that the resulting Steam could ordinarily
attempt to enter on its next movement tick, or a blocked request when no target
exists. `phase_thermodynamics` writes the accepted request and a provisional
1:1 Steam/phase-energy Next result together. The identity is not settled yet.
All Matter-expansion and volume-relief requests then contend in the same
deterministic destination claim domain.

A winning relief request gives the provisional 1:1 completion zero phase-
volume pressure. A blocked request or claim loser gives it `100.0` at the
source exactly once. Only after claim and consequence writers finish does the
joint Material/phase-energy/pressure settle commit Steam. Thus neither outcome
waits for a post-completion request, and neither bypasses the accepted
completion gate. The target is never occupied, reserved or mutated by relief.

This token is a same-tick exclusive availability transaction, not a future
movement reservation. Ordinary GAS movement on the following tick remains the
only mechanism that moves Steam into headspace. That distinction is locally
coherent but makes the candidate unable to represent finite headspace
consumption under the locked 1:1 occupancy model, as shown below.

## Blocking finite-headspace counterexample

The independent review found a conservation counterexample that the candidate
cannot close without changing one of its locked constraints. Consider a sealed
one-Cell-wide column with one EMPTY Cell above a stagger-heated Water column.
At tick `t0` only the top Water is ready; each lower Water is predeclared just
below the endpoint and reaches it only after the vacancy arrives above it:

```text
Stone cap
EMPTY
Water ready at E = Lv
Water below endpoint; reaches E = Lv at t1 thermal/phase
Water farther below; reaches E = Lv at t2 thermal/phase
Stone walls/floor
```

The top Water wins the EMPTY token and settles 1:1 to Steam with zero pressure.
On the following ordinary GAS movement tick, Steam enters that EMPTY Cell and
leaves its source EMPTY. The next Water reaches `E = Lv` during that tick's
thermal/phase sequence, sees the newly vacated source as its up-EMPTY target,
wins zero pressure, and repeats the process. The stagger prevents a lower
same-tick attempt from being classified as an earlier legal Steam density swap.
The vacancy walks down the Water column. Every Water Cell can become one Steam
Cell without any blocked or losing request, even though the chamber never
gained physical capacity. If movement does not consume the route, the unchanged
token target can instead be counted again by another completion on a later tick.

Same-tick exclusivity therefore prevents duplicate ownership only within one
arbitration epoch; it does not consume a cross-tick volume resource. The
candidate cannot guarantee TE5B-F05's transition from early relief to later
confinement or TE5B-F11's pressure/rupture causal chain. The one-shot pure proof
did not model a grid, vacancy conservation or cross-tick movement and therefore
does not refute this counterexample.

Closing the counterexample requires at least one unapproved architecture
change: persistent capacity/reservation state, target or Environment mutation,
additional phase volume/Matter, or a different pressure law. Those choices are
respectively outside this bridge's no-new-state/non-mutating/1:1 constraints or
return to rejected option A/C territory. No replacement is silently selected.
ADR-0007 therefore remains Proposed but is **DESIGN BLOCKED**. TE-3 and TE-5B
runtime remain not started.

## Relief eligibility and selection

The source examines the post-TE-2, pre-phase immutable occupancy snapshot. A
relief candidate is an in-domain `EMPTY` Cell in the EMPTY-only subset of the
existing GAS movement stencil, in exactly this order:

1. up;
2. the two up-diagonals in the current deterministic parity order;
3. the two lateral Cells in the current deterministic parity order.

The same world-coordinate parity used by production GAS proposal selects
left-first versus right-first ordering; no bridge-specific random order is
added. Selection follows the production GAS First-Match stops rather than
blindly skipping every non-EMPTY result:

- in-domain EMPTY becomes the single target and stops;
- occupied non-swappable up/up-diagonal Matter is skipped;
- an earlier legal upward Steam density swap stops as `BLOCKED` because it is
  movement but not free volume;
- occupied lateral Matter is skipped because lateral never swaps;
- out-of-domain Void stops as `EDGE_DEFERRED`; it is not skipped to a later
  lateral Cell and is never a relief target;
- exhausting in-domain stages becomes `BLOCKED`.

Excluded:

- downward Cells;
- any density-swap target;
- occupied GAS or other occupied Matter;
- out-of-domain Void;
- long-distance search or a pathfinder.

Atmospheric Empty and Vacuum Empty both qualify geometrically. Eligibility
does not read, derive or threshold Air pressure. The target's Air mass and Air
energy are not receiver resources and remain byte-identical for both winning
and losing relief outcomes. High-pressure-Air refinement is full TE-5 work.

`EDGE_DEFERRED` does not accept or settle the completion, writes no request,
keeps ADR-0006 ready Water/H and emits no pressure. This preserves the prompt's
Void exclusion and leaves product edge pressure/reservoir meaning undecided
without creating the false top-row case where Steam would exit upward while a
lateral token loses. Swap eligibility is compiled from the current movement
class/rank registry into an existing phase-descriptor trait word; no new table,
binding or pass is introduced.

## Shared request and claim encoding

The source audit found no collision under the current strict
`cell_count < 1 << 30` bound. The candidate therefore retains the proposed
encoding:

```text
REQUEST_MODE_MASK          = 0xC0000000
REQUEST_INDEX_MASK         = 0x3FFFFFFF

REQUEST_NONE               = 0x00000000
REQUEST_MATTER_EXPANSION   = 0x40000000
REQUEST_VOLUME_RELIEF      = 0x80000000
REQUEST_INVALID_RESERVED   = 0xC0000000

targeted proposal = MODE | (target_index + 1)
blocked proposal  = MODE | 0
winning claim     = MODE | (source_index + 1)
```

For every valid world index, `index + 1` is in `1..0x3fffffff`; payload zero is
therefore unambiguously blocked/absent within a nonzero mode. `REQUEST_NONE` is
the only valid all-zero word. Mode `00` with a nonzero payload, mode `11` with
any payload, an out-of-range decoded index, or a claim whose mode/source does
not match the source proposal fails closed.

`expansion_claim` considers valid targeted requests of either mode and chooses
one source using the existing `edge_priority(source, target, tick)` and smaller-
source tie break. It writes the winning source's mode into the claim. Thus a
Matter-expansion and relief request for the same EMPTY Cell produce one winner,
not one winner per mode.

Every producer fully overwrites its whole logical scratch lifetime. In the
projected graph:

1. the final TE-2 claim consumer ends;
2. `phase_context_propose` fully writes claim as context;
3. `phase_thermodynamics` reads context and fully writes mode-aware proposal;
4. `expansion_claim` fully writes mode-aware claim;
5. expansion consumers finish;
6. the later combustion/Smoke producer fully overwrites proposal and its claim
   producer fully overwrites claim before those later consumers.

No mode word is interpreted across a scratch-lifetime boundary.

## Completion and consequence transaction

The relief producer is attempt-derived, not reconstructed later from
temperature. A valid attempt requires endpoint H plus the initiation/continuity
predicate above. The TE-5B acceptance function returns a valid relief word
before a Steam result can settle; `phase_thermodynamics` writes that word and
provisional non-Steam→Steam Next identity in the same invocation. An
already-Steam Cell emits none, so a pressure source cannot repeat merely
because Steam remains present.

The transaction is:

| Relief result | Source identity/quantity | Target Matter/Air | Source pressure |
|---|---|---|---:|
| valid request wins | complete 1:1 to Steam; phase-family count unchanged | byte-identical EMPTY / byte-identical Air | `+0` |
| no GAS-reachable EMPTY target | complete 1:1 to Steam; phase-family count unchanged | none | `+100.0` once |
| targeted request loses | complete 1:1 to Steam; phase-family count unchanged | byte-identical | `+100.0` once |
| Void is the ordinary first match | remain ready Water; completion not accepted | none | `+0`; edge decision deferred |

`WATER_VAPORIZATION_CONFINEMENT_PRESSURE = 100.0` is inherited from the
frozen G5-B gameplay consequence. It is intentionally above the current Wood
rupture threshold `80.0`. Both are gameplay scalars, not SI pressure.

The consequence remains data-driven. The packed phase descriptor's existing
above-consequence pressure slot is `100.0` for every phase-family source
identity whose single normalization invocation can cross the Water-to-Steam
endpoint after the attempt predicate passes. This includes an extreme but
finite Ice input only when its current context is gas-facing; without that
context it cannot initiate boiling and must not emit a relief word. For
phase-family descriptors this slot is completion
consequence metadata, not authorization for generic extra Matter. The relief
mode selects it; WGSL must not branch on the Water Material name. A non-family
Matter-expansion request continues to select its own direction/rule metadata.

## Generic Matter-expansion compatibility

`REQUEST_MATTER_EXPANSION` preserves the historical transaction:

1. destination claim;
2. Environment receiver arbitration;
3. target Matter spawn;
4. Environment displacement/reconcile;
5. direct blocked or losing pressure;
6. winning-target but failed-Environment-receiver pressure exactly once.

Only Matter-expansion mode may request an Environment receiver, spawn target
Matter or cause target Environment displacement. Receiver and spawn passes
decode mode and ignore relief. Because relief never changes target Matter and
never receives Environment, Environment reconcile has no relief mutation to
apply. `environment_blocked_expansion_pressure` accepts Matter-expansion mode
only and ignores relief, preventing a second relief consequence.

A generic non-family `matter_yield > 1` rule may target only non-phase Matter,
as locked by D-018. TE-5B neither deletes that path nor designs phase energy for
an extra destination.

## Pressure boundary

TE-5B does not change `pressure[]`. It remains non-negative gameplay gauge
overpressure, propagated by the current local scalar rule through Liquid/Gas
pressure media, cleared on non-pressure Cells by existing semantics and
consumed by current rupture thresholds.

The bridge does not define or compute:

```text
effective pressure = derived Air pressure + gauge pressure
```

It does not add Atmosphere background-pressure force, Vacuum pressure,
structure face differential, a product world-edge pressure mode or Vacuum
combustion. Those remain full TE-5 or TE-4 decisions.

## Pass, binding and memory feasibility

The bridge changes predicates/encodings in the already projected TE-3 chain;
it adds no pass. The normative projected graph remains 40 timestamped compute
passes, 80 timestamp queries and two 640-byte profiler buffers (1,280 bytes).

Relevant storage counts remain at or below eight:

| Projected pass | Storage RO | Storage RW | Total | TE-5B change |
|---|---:|---:|---:|---|
| `phase_thermodynamics` | 4 | 4 | **8** | writes accepted attempt word plus provisional Steam Next |
| `expansion_claim` | 3 | 1 | 4 | decodes both modes; writes winning mode/source |
| expansion Environment receiver claim | 4 | 1 | 5 | filters Matter mode |
| `expansion_spawn_commit` | 5 | 3 | **8** | filters Matter mode |
| `expansion_pressure` | 7 | 1 | **8** | mode-specific direct failure consequence |
| Environment-blocked expansion pressure | 6 | 1 | 7 | filters Matter mode |

No new binding, persistent buffer, full-world scratch buffer, lookup-table
allocation or production dispatch is required. The same 512-byte phase
descriptor allocation supplies data-driven consequence metadata through its
existing views.

The phase/expansion writer order is fixed:

```text
phase_context_propose
-> phase_thermodynamics / accepted-attempt proposal + provisional identity full write
-> expansion_claim / claim full write
-> Matter-only Environment receiver claim
-> Matter-only expansion spawn
-> expansion_pressure (generic direct failures + relief failures)
-> Matter-only Environment-blocked pressure
-> identity/phase-energy hygiene + Environment reconcile
-> Material/temperature/phase-energy/pressure/Environment settle commits accepted completion
-> later scalar pressure propagation
-> pressure settle
-> rupture
```

The completion identity, its phase energy and its pressure consequence become
visible before later pressure propagation and rupture. A completion or pressure
event must keep the existing required activity halo awake; exact sleep-on/off
equivalence remains a future structural/GPU fixture. Profiler grouping remains
the existing phase/expansion/pressure grouping; no TE-5B-only group is added.

TE-5B memory delta relative to the accepted TE-3 projection is exactly zero.
The projected totals remain 4,722,608 bytes at 256² and 302,018,096 bytes at
2048² with profiler buffers, subject to future runtime measurement.

## Atomic activation boundary

This bridge is the explicit TE-5 completion transaction required by ADR-0006,
but this proposed ADR is not runtime authority. The verified future TE-3
phase-energy implementation and verified TE-5B bridge must activate together
on one source. That source must reproduce the full G5 causal fixture with new
source-bound evidence before it becomes production/user-testable. Neither half
may ship alone.

Historical G5-A/B/C, TE-1 and TE-2 receipts remain valid only at their recorded
sources. They are inputs to the design, not evidence for the future atomic
source, and must not be relabelled or rebound.

## Consequences

Locally established properties of the evaluated candidate:

- open boiling does not receive a fabricated pressure impulse;
- same-tick local headspace is exclusive even though the target remains EMPTY;
- 1:1 phase quantity and phase enthalpy remain unchanged;
- the historical G5 pressure magnitude and generic expansion transaction are
  preserved;
- no new persistent state, scratch, pass or solver is introduced.

Costs and residual risks:

- occupancy-only relief intentionally ignores Air pressure, so it is only the
  narrow bridge and not a full confinement model;
- the token is not a next-tick reservation; unrelated Matter movement can
  change occupancy before Steam moves;
- finite-headspace capacity is not represented: ordinary 1:1 movement moves an
  EMPTY vacancy instead of consuming it, so F05/F11 are unsatisfiable for the
  current candidate;
- a Void-first endpoint remains ready Water until context changes or full TE-5
  decides product edge semantics; TE-5B does not label Void as free volume;
- the inherited `100.0` scalar can create an abrupt threshold crossing and
  remains a user-review choice for the atomic fixture;
- mode filtering, full writes, bindings, sleep, GPU races and performance are
  design obligations, not established runtime facts.

## Reuse and non-reuse

Reused: current EMPTY-only portions of the GAS movement stencil, deterministic
parity, `edge_priority`, one-target claim ownership, the 30-bit Cell-index
bound, proposal/claim scratch, Environment receiver separation, scalar gauge
pressure, pressure propagation, rupture threshold and accepted TE-3 pass
lifetime.

New only at the design level: two mode bits, the volume-relief transaction,
mode filters and the explicit finite-headspace fixtures. No general fluid
solver, velocity, volume fraction, fragment system or universal progress state
is introduced.

The Powder Toy and Chinese-community materials remain clean-room fixture and
design references. No external source code, translated implementation or
external formula enters the repository.

## Approval boundary

ADR-0007 remains **PROPOSED — DESIGN BLOCKED / user architecture revision
required**. The current exclusive-token model cannot be approved as the
finite-headspace bridge without first resolving the counterexample. User review
must explicitly revise or disposition:

1. the missing capacity-consumption ownership model, including which locked
   no-state/non-mutation/1:1 constraint, if any, may change;
2. the exclusive local volume-relief-token model after that revision;
3. occupancy-only relief for both Atmospheric Empty and Vacuum Empty;
4. the exact two-bit mode encoding;
5. inherited confinement pressure `100.0` in the new atomic fixture;
6. whether the finite-headspace causal fixture preserves intended product
   meaning.

Full TE-5 background-pressure/structure-differential design, product edge mode,
Vacuum combustion and runtime pass/binding/performance evidence remain open.
This ADR does not authorize Rust, WGSL, Cargo, build, launch or runtime work.
