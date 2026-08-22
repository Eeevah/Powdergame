# Ignition Kinetics Specification

Status: **PROPOSED / V2 DESIGN BLOCKED / PRODUCTION NOT ESTABLISHED** under D-029 and ADR-0012. This document
defines the candidate contract; it is not active production behavior.

## State and ownership

`ignition_exposure` is reversible Matter-owned pre-ignition work. The primary
candidate encodes 0..63 with low bits at flags 2..3 and high bits at 28..31.
Encoding outside 0..63 or a descriptor budget outside 1..63 is an invariant
failure, never a clamp. Oil/Wood may own exposure while unlit; burning,
non-combustible Matter and EMPTY own exact zero. Movement/density swap carries
it; identity replacement, Void exit, decay, rupture, fuel consumption, Draw,
Erase, preset and reset clear it in Current and Next.

Fuel progress remains only irreversible consumed-fuel ticks. Extinguishing
preserves fuel and clears/restarts exposure at zero; reignition cannot restore
fuel.

## Dose transaction

For an unlit combustible Cell, only its own finite authoritative temperature
can establish thermal eligibility. At/above the Material ignition threshold,
bounded bucketed thermal work and a bounded count of orthogonal
previous-snapshot flame events accumulate. Below threshold, exposure decreases
by the configured positive decay to zero. A flame does not bypass the target's
threshold, write its identity or inspect same-tick Next flags.

V1 selected nothing and remains immutable blocked history. D-029 selects for
v2 Oil `48/2/50/6/1/2/4` and Wood `60/1/50/5/1/2/4` for
budget/base/width/max/decay/flame/cap. Evidence identity
`TE4-IGNITION-KINETICS-REFERENCE-V2` validates these exact values without a
coefficient search.

## Chemical heat

The Core candidate source is gross
`Q_tick = legacy_delta_T * heat_capacity`: Oil `15`, Wood `8`. Descriptor
compilation serializes the derived delta `Q/C`, so combustion needs no heat-
capacity storage binding. The existing 1200 C cap divides accounting into
gross Q, deposited sensible Q (`C * actual delta_T`) and clipped Q
(`gross - deposited`). Only deposited heat becomes ordinary TE-2 sensible
heat; TE-2 does not add the gross source again. The consumption tick emits
zero, preserving the current order; therefore 599 Oil and 899 Wood emitting
ticks bound **gross** totals at `8,985` and `7,192`, while deposited totals are
temperature-history dependent and no larger. Extinguished and consumed fuel
emit zero. No Oxygen, Air mass, Pressure, Ash or new Matter is involved.

Packed descriptor validation is fail-closed: budget 1..63; positive u8 decay,
base and bucket width; `base<=max<=255`; flame/cap u8 with `cap>=flame`;
duration 1..4095; finite thresholds/Q/capacity/delta; non-negative Q; positive
capacity; reserved bits zero. Non-combustible sentinel entries have all
kinetics and source fields zero. Packing never truncates a larger integer.

## Air-face access

Ignition and burning sustain require an in-domain orthogonal neighbour whose
foreground Material is EMPTY and current Air mass is positive. Exact Vacuum,
Void and occupied GAS Matter do not qualify; positive LowPressure does. Air is
not consumed and no Oxygen quantity or rate scaling exists. Loss of access
extinguishes before Heat, Flame or Smoke emission that tick, preserves fuel,
and leaves exposure zero. An inaccessible unlit Cell decays partial exposure.

## Invariants

- **IG-INV-001** One-to-three threshold ticks do not ignite Oil or Wood.
- **IG-INV-002** Sustained eligibility ignites in an accepted bounded window.
- **IG-INV-003** Greater excess never lowers the thermal rate.
- **IG-INV-004** Cooling monotonically decays exposure to zero.
- **IG-INV-005** Exposure follows its Matter through movement/swaps.
- **IG-INV-006** Replacement and EMPTY hold exact zero exposure.
- **IG-INV-007** Fuel progress has no exposure meaning.
- **IG-INV-008** Previous-tick flame cannot recursively chain in one tick.
- **IG-INV-009** Connectivity alone cannot ignite a whole region.
- **IG-INV-010** Non-combustible Matter never accumulates exposure.
- **IG-INV-011** Chemical heat is finite and emitted once per emitting tick.
- **IG-INV-012** TE-2 never injects chemical heat a second time.
- **IG-INV-013** Physics and activity use identical work predicates.
- **IG-INV-014** Stable unlit/burned-out regions may sleep.
- **IG-INV-015** Descriptor/state values are finite and canonical.
- **IG-INV-016** Every production pass remains at eight storage bindings or less.
- **IG-INV-017** Ignition/sustain require an orthogonal EMPTY face with positive Air mass; no Air is consumed.
- **IG-INV-018** Historical evidence stays source-bound.

The v2 reduced reference established the mathematical/state-transition subset
for 13 fixtures. TE4-F01/F14/F16/F17 and all actual GPU/product/user claims
remain `NOT_ESTABLISHED`; D-028's v1 execution established none of them.

Independent review invalidated the broad `state_transition_result=PASS`
interpretation: several path counters are not mutation-derived, sole-Air-face
loss through same-tick Smoke is unspecified, and F08 has no frozen exact
frontier oracle. These are design/evidence blockers, not runtime bugs.
