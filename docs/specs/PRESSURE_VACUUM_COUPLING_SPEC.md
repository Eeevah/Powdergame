# Pressure / Vacuum Coupling Specification

- **Status:** Proposed contract — DESIGN BLOCKED / runtime not started
- **Authority:** D-035 and proposed [ADR-0013](../architecture/decisions/ADR-0013-local-relaxing-phase-load-pressure.md)
- **Baseline:** `769e687c04406016fe9d66c8496269b459f06d83`

## 1. Scope and non-goals

This specification defines derived Air background pressure, the reused local
dynamic-pressure field, phase-load sourcing, total-pressure consumers,
settlement, activity and future evidence. It does not define exact volume
ownership, a fluid velocity solver, tokens, reservations, matching, CCL,
phase packets, Oxygen, Ash, Vacuum combustion or a new persistent field.

## 2. Authoritative state and validity

Persistent state is unchanged:

```text
air_mass_current/next
air_energy_current/next
phase_energy_current/next
pressure_current/next
```

`pressure` is spatial dynamic mechanical pressure. It never follows Matter
ownership and is never an Air mass or energy value. Valid dynamic pressure is
finite and in `[0,PRESSURE_MAX]`. Invalid authoritative phase or Environment
state is a failed invariant, not clamped evidence. Arithmetic sanitization may
prevent propagation of NaN/Infinity but must increment/report a validation
fault in future fixtures.

For canonical EMPTY Environment:

```text
positive Air: P_air = air_energy_current / 293.15
exact Vacuum (air_mass=0, air_energy=0): P_air = 0
P_total = P_air + sanitize(P_dynamic)
```

Non-EMPTY Matter has canonical zero Air, hence `P_air=0`. Every consumer uses
one shared semantic helper/equivalent expression and adds each term once.

## 3. Dynamic-pressure graph

A Cell is a node iff it is in-domain and either:

- Material EMPTY, regardless of Atmosphere/LowPressure/Vacuum; or
- registered Liquid or Gas Matter.

Static, Powder and out-of-domain Void are blocked. Pressure edges are the four
orthogonal node-node faces. Default out-of-domain behavior is sealed/no-flux.
A blocked Cell writes dynamic zero after the pressure update. Canonical
isolated Vacuum initialized at zero remains zero; connected Vacuum may receive
dynamic pressure without acquiring Air.

## 4. Phase load

Constants:

```text
Lv = 480
P_FULL_VAPOR = 100
```

The source fraction is:

```text
Steam: phase_energy / Lv
Water: phase_energy / Lv iff phase_energy > 0 and the exact current TE-3
       gas-facing predicate is true
otherwise: 0
```

The accepted gas-facing predicate is orthogonal in-domain EMPTY or registered
Gas Matter, using the same material/class snapshot as phase context. It does
not inspect derived Air pressure. Buried ready-Water is zero-source. For every
source, `0 <= r <= 1` must already hold. `P_phase_target=100*r`.

Water at `E=Lv` and gas-facing and Steam at canonical `E=Lv` both target 100.
Identity transition adds no impulse. Movement relocates the phase source with
Matter/phase energy on the already-settled ownership edge; it does not relocate
stored dynamic pressure.

## 5. Dynamic update

Constants:

```text
PRESSURE_DIFFUSION_RATE = 0.20
PHASE_RELAXATION_RATE   = 0.02
GENERIC_IMPULSE_MAX     = 100
PRESSURE_MAX            = 1.0e6
```

For degree `k` and node neighbours `n`:

```text
p_next = clamp(
  (1-kD-R)*p_current + D*sum(p_n) + R*P_phase_target
  + bounded_generic_impulse,
  0, PRESSURE_MAX)
```

`k<=4`, so the retained coefficient is at least `0.18`. Symmetric internal
diffusion conserves component sum before relaxation/impulse/clamp. In an
unclamped sealed equilibrium without impulse, summing all Cell equations gives
`sum(p)=sum(target)`. This is a component-average identity, not a claim of
uniform local pressure.

Target removal causes exponential local relaxation plus diffusion. A sealed
isolated nonzero node is deliberately dissipative; historical “pressure never
decays” semantics are superseded for this future source.

## 6. Generic impulses

Only the existing generic non-family expansion direct-failure and
Environment-receiver-failure writers may add an impulse. Exactly one of those
branches owns the consequence. The source descriptor must be finite in
`[0,100]`. Water/Ice/Steam always emit no expansion proposal and have zero
blocked pressure. The event writes once before the pressure update; later
identity does not reconstruct it. Reset, Draw, Erase and canonical staging
write both pressure halves to zero unless an explicitly validated fixture
stages pressure.

## 7. Total-pressure consumers

### 7.1 Air transport

For an eligible EMPTY face `a->b`:

```text
raw_mass = 0.125 * max(P_total(a)-P_total(b)-0.001, 0)
```

Existing donor mass/energy and receiver headroom scaling remain authoritative.
Dynamic pressure can request flow but cannot manufacture donor mass: a Vacuum
donor has zero available mass and transfers zero. Mass and donor-specific
energy settle through separate future self-write commits so both passes remain
within eight storage bindings.

### 7.2 Liquid/Gas movement

Pressure adds no target and never overrides legality. The existing ordered
stencil stages and density-swap rules remain. When one stage contains multiple
legal destinations, rank them by greatest positive
`P_total(source)-P_total(destination)` above `0.001`; use existing parity order
for equal/sub-deadband drops. Singleton vertical stages remain singleton.
Powder and Static movement semantics do not read pressure.

### 7.3 Structure stress

After dynamic-pressure settle, each structural Cell samples total pressure on
its four orthogonal faces. A blocked or out-of-domain face samples zero for the
current finite-world contract. Stress is:

```text
max(abs(P_left-P_right), abs(P_up-P_down))
```

Only descriptor-owned positive rupture thresholds act. Uniform opposing
pressure gives zero stress. Rupture writes its own Matter Cell to EMPTY and
performs existing temperature/flag/phase/Environment hygiene. It does not
delete neighbouring dynamic pressure; the new EMPTY joins the graph and later
Air/Matter ticks may use it.

## 8. Tick and scratch ownership

Normative future order:

1. activity wake;
2. movement proposal/claim/commit reads `P_total_current`;
3. movement Matter/Environment/phase settle;
4. Air scale plus separate mass and energy commits read the same
   `P_total_current`; Air settles;
5. unified thermal and phase settle;
6. generic expansion/decay/combustion transactions settle;
7. local dynamic-pressure update reads settled Material/phase and old spatial
   pressure, then pressure settles;
8. rupture reads settled total pressure and settles its opening;
9. base/phase/Environment/ignition/pressure activity proposals reduce;
10. following Tick movement and Air respond to the new total pressure.

No same-Tick rollback exists. Proposal/claim are Air donor/receiver scales at
the Air lifetime, are consumed by both split commits, and are fully overwritten
before thermal, phase and Smoke lifetimes.

## 9. Activity and sleep

Pressure work is active if the exact local update would change dynamic pressure
by more than `PRESSURE_ACTIVITY_EPS`, including target relaxation or diffusion.
Environment work uses total-pressure face demand. Matter work uses the existing
legal-candidate predicate; pressure only changes ranking. A source/load edit,
Matter movement, phase change, generic impulse or rupture wakes the existing
chunk halo. A nonzero uniform equilibrium may sleep only when its target is
equal, every eligible neighbour is equal and Air/Matter have no runnable total-
pressure work. Removing load wakes relaxation until the update falls within
epsilon. Sleep-on/off must match for equal executed ticks.

## 10. Edge and reservoir

The product/default edge is sealed/no-flux. Fixed standard Atmosphere is a
future fixture-only boundary with Air `(1,293.15)`, dynamic zero and explicit
external mass/energy/pressure-exchange counters. It is not an implicit Void
rule and is not a product option without another decision.

## 11. Invariants

- **PV-INV-001:** phase-family quantity remains 1:1; no extra Steam exists.
- **PV-INV-002:** phase load derives only from accepted phase energy/context.
- **PV-INV-003:** buried/non-runnable Water sources zero pressure.
- **PV-INV-004:** Water/Steam target is continuous at completion.
- **PV-INV-005:** dynamic pressure is spatial, finite and non-negative.
- **PV-INV-006:** Air background is derived, not stored.
- **PV-INV-007:** exact Vacuum has zero Air background.
- **PV-INV-008:** background and dynamic terms are each added exactly once.
- **PV-INV-009:** only EMPTY/Liquid/Gas form dynamic-pressure nodes.
- **PV-INV-010:** `4D+R=0.82<=1` and every retained coefficient is non-negative.
- **PV-INV-011:** sealed equilibrium component average equals target average
  before clamp/impulse.
- **PV-INV-012:** removing phase load permits pressure to relax to zero.
- **PV-INV-013:** generic impulse has one writer and maximum 100.
- **PV-INV-014:** uniform opposing total pressure cannot rupture structure.
- **PV-INV-015:** one-sided differential may rupture only through descriptor data.
- **PV-INV-016:** movement creates no pressure owner transfer or rollback.
- **PV-INV-017:** Vacuum combustion stays disabled.
- **PV-INV-018:** sealed edge is no-flux; reservoir exchange is explicit.
- **PV-INV-019:** pressure activity and update use identical predicates.
- **PV-INV-020:** no new persistent/full-world scratch allocation is required.
- **PV-INV-021:** every future pass has at most eight storage bindings.
- **PV-INV-022:** historical G5 evidence is not rebound.

## 12. Runtime boundary

This specification is architecture only. None of these rules is implemented
or runtime-validated. Independent source review found three unresolved High
contradictions: the phase-load rule requires a pre-transition context snapshot
that is no longer live at the proposed pass; the generic impulse term has no
projected input and disagrees with the existing transaction order; and the
unchanged base activity predicate prevents an exact nonuniform equilibrium
from sleeping. Therefore **TE-5R0 DESIGN BLOCKED**. The candidate is preserved
as reviewed; no replacement rule is introduced here. See the
[review](../adversarial-reviews/TE5R_PRESSURE_VACUUM_REENTRY_DESIGN.md) and
[validation boundary](../development/PRESSURE_VACUUM_COUPLING_VALIDATION.md).
