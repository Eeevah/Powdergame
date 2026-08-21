# ADR-0011: Conservative phase packets

- **Status:** PROPOSED — DESIGN BLOCKED / ARCHITECTURE REVISION REQUIRED
- **Decision:** D-023
- **Evidence identity:** `TE3Q-PHASE-PACKETS-REFERENCE-V1`
- **Runtime:** NOT STARTED

## Context

TE-5B, TE-5C, TE-5D and TE-5X are preserved blocked history. Their failures
show that forcing every foreground Steam Cell to represent one whole
Water-equivalent quantity makes visible expansion depend on cross-tick tokens,
capacity allocation, persistent extent matching or a remote conservative
field. D-023 supersedes only that whole-Cell quantity constraint. One Cell is
still at most one foreground Matter and no mixed foreground Matter is added.

## Decision

Adopt as a docs/reference candidate a two-unit conservative representation:

```text
PHASE_UNIT_SCALE = 2
EMPTY/non-phase = 0
Ice/Water        = 2
expanded Steam   = 1
compressed Steam = 2
```

Water-equivalent quantity is `sum(phase_units)/2`. `phase_units` and
`phase_energy` are Matter-owned Current/Next pairs. `phase_energy` is total
Cell latent energy. Canonical values are Ice/2 `-80`, Water/2 `0`, Steam/1
`240`, and Steam/2 `480`. Local enthalpy is
`H = (phase_units/2) * S_material(T) + phase_energy`.

Boiling with a successful existing expansion claim and whole-parcel
Environment receiver transaction produces two real Steam/1 Cells. A blocked,
losing or receiver-failed boil produces one Steam/2 Cell. A Steam/2 Cell later
retries the same local GAS-reachable EMPTY transaction and splits on success.
No quantity is created and the target becomes actual Matter.

Condensation uses deterministic **orthogonal-only** pairing. This is selected
before reference execution because it is the smallest symmetric neighbourhood,
crosses chunk seams naturally, avoids diagonal-only teleport contraction and
needs no cold-surface ranking binding. Proposal order is rotating orthogonal
order by `(tick parity, source coordinate hash)`; the existing stateless claim
priority chooses one winner per packet. Two endpoint-ready Steam/1 packets
merge into one Water/2 packet and one canonical Vacuum EMPTY Cell. A lone
Steam/1 packet remains finite, condensation-ready and explicitly metastable.

A separate spatial `phase_pressure_current/next` pair uses source equilibrium
`100` only for Steam/2 and `0` otherwise. Predeclared coefficients are:

```text
PHASE_PRESSURE_RELAXATION = 0.20
PHASE_PRESSURE_DIFFUSION  = 0.05
PHASE_PRESSURE_EPSILON    = 0.01
```

`0.20 + 4*0.05 = 0.40 <= 1`. Pressure-media Cells update by relaxation plus
orthogonal diffusion; non-media write zero. Values with sub-epsilon update
retain the previous finite value and may sleep until a source, movement,
thermal, edit or neighbour wake. An isolated compressed source crosses Wood's
threshold `80` on update 8, not instantly. Rupture reads sanitized
`generic_pressure + phase_pressure` exactly once. Derived Air pressure is not
included.

## Transaction and ownership consequences

- Movement and density swap carry units and phase energy with Matter.
- Phase pressure is spatial and follows only its own source/diffusion law.
- Void exit removes the exiting units from finite-world inventory explicitly.
- Split receiver failure changes neither target Air nor source packet.
- Merge creates Vacuum at the loser; TE-2 may refill it later, but the merge
  invents no Air.
- Draw/Erase/reset and every bypass writer stage canonical unit/energy pairs.
- Invalid identity/unit/energy combinations fail closed; clamping is not
  evidence.
- Generic non-family expansion and generic gauge pressure remain separate.

## GPU feasibility projection

This is static design arithmetic, not an implementation claim. At 2048² the
three Current/Next pairs—phase units, phase energy and phase pressure—are each
`33,554,432` B, totaling `100,663,296` B / 96 MiB over TE-2. No owner link,
matching state, CCL state or additional full-world scratch is proposed.

The candidate projects 50 timestamped passes and 100 queries. It adds one
unit-movement reconcile, splits phase normalization into two passes, adds one
quantity split commit, five local merge passes, one phase-pressure pass and
one pressure-activity pass to the accepted 40-pass TE-3 projection. Existing
proposal/claim scratch is fully overwritten between expansion, merge and Smoke
lifetimes. Every representative pass is at or below eight storage bindings;
the exact table is normative in the specification. Runtime allocation,
compilation, races, performance and device limits remain unknown.

## Alternatives rejected for this candidate

- Preserve whole-Cell quantity: repeats the blocked global volume-allocation
  problem.
- Half-Water or same-Cell mixture: obscures foreground identity and broadens
  the initial state space.
- Existing movement-neighbourhood merge: diagonal contraction is harder to
  read and expands claim predicates without evidence.
- Cold-surface-ranked merge: adds thermal-neighbour ranking and bindings while
  local cooling eligibility already gates the packets.
- Global matching/CCL: expressly outside D-023 and unnecessary for local pair
  conservation.

## Acceptance boundary

This ADR is not accepted and authorizes no runtime. It may advance only after
the one-shot reference identity completes, a fresh independent review leaves
Critical/High `0`, and the user reviews half-packet occupancy, lone-packet
metastability, merge order, phase-pressure timing, 96 MiB state cost and
visual/product meaning. Otherwise the disposition is **TE-3Q / TE-5Q DESIGN
BLOCKED**.

## Independent-review disposition

The frozen mathematical reference completed, but the fresh review recorded
Critical `0`, High `8`, Medium `1`, Low `0`; ADR-0011 is therefore **DESIGN
BLOCKED**. The one-shot identity remains immutable and cannot be patched or
rerun. Blocking findings cover non-executable/under-modeled named fixtures,
greedy local merge stranding, movable Steam/2 pressure reset, rupture-eligible
stored pressure after source removal, unfrozen f32/generic-pressure transaction
details, incomplete writer/editor/reset closure and a missing phase-pressure
sleep binding. See [`TE3Q_CONSERVATIVE_PHASE_PACKETS_DESIGN`](../../adversarial-reviews/TE3Q_CONSERVATIVE_PHASE_PACKETS_DESIGN.md).

Runtime remains **NOT STARTED**. A future attempt requires a new direct user
decision and new evidence identity; this ADR must not be marked Accepted.
