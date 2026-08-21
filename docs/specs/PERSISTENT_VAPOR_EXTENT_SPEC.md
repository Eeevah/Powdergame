# Persistent Vapor Extent Specification

- **Authority:** proposed ADR-0009 and D-021
- **Status:** proposed design blocked by matching hard gate; runtime not started
- **Applies atomically with:** accepted ADR-0006 TE-3 phase state

## 1. Canonical state

There is exactly one new logical Current/Next pair of 8-byte Cells:
`{ link: u32, phase_pressure: f32 }`. No other persistent phase-volume field is
part of this candidate. State outside the rules below is invalid.

| Material/state | Link | Phase pressure |
|---|---|---:|
| non-phase, Ice, Water | NONE | finite non-negative propagated value only on existing pressure media; otherwise 0 |
| reserved Steam | SOURCE_RESERVED(target) | finite non-negative |
| its EMPTY extent | TARGET_RESERVED(owner) | exactly 0 |
| compressed Steam | SOURCE_COMPRESSED(saturated age) | finite non-negative |

`SOURCE_RESERVED(t)` is valid iff Cell `t` is in domain, EMPTY, exact-zero Air,
and contains `TARGET_RESERVED(source)`. The converse is also required. One
source and one target participate in at most one pair.

## 2. Acquisition transaction

Candidates are the in-domain, PV_NONE, Material EMPTY subset of up,
parity-ordered up diagonals and parity-ordered lateral GAS positions. Each
request has at most one proposed target per matching round. A target has at
most one winner.

Before link commit, the target Air parcel must be accepted by the existing
whole-parcel Environment receiver grammar. Atmospheric Air moves without
loss; Vacuum's zero parcel is valid. No receiver or insufficient headroom
causes a byte-identical failure. A commit writes both reciprocal links and
exact zero target Air. No partial transfer, clamp, deletion or target Matter
change is permitted.

## 3. Matching

The evaluated algorithm uses deterministic target-scarcity/source-age/source-
index ordering and atomic augmenting paths limited to six source vertices.
Six settle ticks are allowed. A failed bounded search writes no partial path.
Compressed age saturates at `0x3fffffff`.

Hard acceptance rule: for every approved-domain state with a complete
reservation matching, the algorithm must complete it before any participating
source reaches phase pressure 80, or provide a verifiable no-matching proof.
This is a correctness gate, not a coefficient-tuning goal.

## 4. Phase integration

Water at `E=Lv` may complete 1:1 through this explicit transaction. It becomes
reserved Steam when acquisition succeeds and compressed Steam otherwise. No
extra Matter is created and H is neither discarded nor transferred to the
extent. Partial Water remains link NONE. Steam condensation releases its own
extent, clears the source link and changes equilibrium source to zero.

## 5. Movement settle

All link and Environment consequences settle before the public Current/Next
swap:

- owner→own extent: owner moves to target; vacated source is the new target;
- owner→unreserved EMPTY: destination becomes owner, source becomes new
  extent, old extent becomes PV_NONE, and destination Air moves as one parcel
  to old extent;
- density swap: extent stays spatially fixed and its backlink changes to the
  owner's destination index;
- Void exit: the reciprocal target is released;
- non-owner movement and Draw treat TARGET_RESERVED as occupied.

Only an exact reciprocal owner may release a target. A mismatched backlink
fails closed and raises a validation fault; it never releases another source's
capacity.

## 6. Air and thermal rules

TARGET_RESERVED is a zero-Air exclusion mask. Air donor, receiver, conduction
node and Matter↔Air face predicates all return false. The Air-flow scale and
thermal-stability scale passes read the link; existing commit passes need no
additional binding because the target is canonical zero-Air and its computed
scale is zero. Environment activity reads the link so excluded faces do not
create permanent false activity. Release changes the Cell to PV_NONE Vacuum;
ordinary TE-2 rules then apply.

## 7. Dedicated phase pressure

Use relaxation `0.10`, diffusion `0.025`, and equilibrium 100 only at
compressed Steam. Eligible diffusion neighbours are orthogonal Liquid/Gas
pressure media. EMPTY and non-pressure Matter write zero. Every result is
finite, non-negative, and source-only fixtures stay at or below 100.

Generic pressure is neither read as equilibrium nor modified by this update.
Rupture sanitizes and adds generic plus phase pressure once. Derived Air
pressure is excluded.

## 8. Activity and settle order

The future semantic order is:

```text
movement eligibility using link Current
→ Matter ownership settle
→ phase-volume movement context full write
→ movement Environment/link reconcile
→ TE-2 Air/thermal work with reserved-target mask
→ TE-3 phase completion
→ Smoke proposal/claim consumers finish
→ matching proposal/claim/reassignment rounds
→ Environment receiver acceptance
→ reciprocal reservation commit
→ phase-pressure update
→ rupture effective stress
→ phase/pressure/activity census
→ joint Current/Next settle
```

Proposal, claim and Environment receiver scratch are fully overwritten at
each named lifetime boundary. Matching or phase-pressure work wakes the owning
chunk plus the required neighbour halo. A chunk may sleep only when no link
repair, request, pressure delta, phase work or Environment refill face exists.

## 9. Invariants

- **PVX-INV-001:** phase-family quantity remains 1:1.
- **PVX-INV-002:** one Steam owns at most one extent.
- **PVX-INV-003:** one extent has exactly one owner.
- **PVX-INV-004:** every reserved pair is reciprocal.
- **PVX-INV-005:** an extent consumes capacity across ticks.
- **PVX-INV-006:** vacancy movement cannot reset consumed capacity.
- **PVX-INV-007:** a reserved extent contains and admits no Air.
- **PVX-INV-008:** acquisition preserves the complete Air parcel or fails unchanged.
- **PVX-INV-009:** movement reconciles ownership and Air exactly.
- **PVX-INV-010:** condensation and Void exit leave no orphan.
- **PVX-INV-011:** sufficient matching cannot cause rupture-capable false pressure.
- **PVX-INV-012:** only compressed Steam has equilibrium source 100.
- **PVX-INV-013:** phase pressure declines after source relief.
- **PVX-INV-014:** generic pressure is never erased by phase relaxation.
- **PVX-INV-015:** rupture reads the bounded component sum once.
- **PVX-INV-016:** derived Air pressure is not counted.
- **PVX-INV-017:** no unproved full-world scratch is assumed.
- **PVX-INV-018:** reset, edit and every settle are finite and canonical.
- **PVX-INV-019:** equilibrium can sleep.
- **PVX-INV-020:** historical evidence remains source-bound.

## 10. Resource contract

State increment is 16 bytes per Cell across both halves: 1,048,576 bytes at
256² and 67,108,864 bytes at 2048². The fixed candidate projects 62 passes and
124 timestamp queries. Every proposed pass is limited to eight storage
bindings. Exact implementation bindings and performance remain future
source-bound gates.

## 11. Blocking persistent-state matching witness

The frozen depth-six algorithm is not complete over canonical persistent
states. Let `U0..U6` already own `V0..V6`, let `U7` be compressed, give `U7`
only `V0`, and give each `Ui` edges to `Vi` and `V(i+1)`. `V7` is free. A
complete reassignment exists, but the augmenting path visits eight sources.
The atomic depth-six search fails unchanged on every retry. This is a direct
PVX-INV-011 violation; TE-5D is DESIGN BLOCKED pending wider matching scope.
