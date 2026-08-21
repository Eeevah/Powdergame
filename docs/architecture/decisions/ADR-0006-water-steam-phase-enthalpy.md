# ADR-0006 — Water/Steam Phase Enthalpy

- **Status:** ACCEPTED FOR FUTURE ATOMIC IMPLEMENTATION
- **Date:** 2026-08-20; amended 2026-08-21
- **Decision owner:** user at the TE-3D architecture-review boundary
- **Decision:** D-018
- **Design source:** docs-only work descended from TE-2 closure `fd97e8b...`
- **Runtime status:** TE-3 NOT STARTED
- **Atomic bridge attempts:** [`ADR-0007`](ADR-0007-phase-volume-pressure-bridge.md) token REJECTED / DESIGN BLOCKED; [`ADR-0008`](ADR-0008-local-vapor-capacity-pressure.md) local capacity law PROPOSED / DESIGN BLOCKED
- **Supersedes at implementation only:** current Water `matter_yield = 2` and
  blocked-expansion-pressure behavior; no runtime source is changed here

## Context

The current phase path can perform:

```text
1 Water -> source Steam + optional extra Steam -> up to 2 Water
```

Temperature is preserved through each identity change, and no latent progress
is stored. A closed boil/condense cycle can therefore gain Water-equivalent
foreground Matter. Direct Sandbox observation also found persistent airborne
Water/Steam checkerboard clumps. Threshold retuning cannot repair either
accounting defect.

TE-2 is separately **USER ACCEPTED WITH KNOWN FOLLOW-UP**. Its passive thermal
pass already transfers one conservative local quantity `Q`; TE-3 must consume
that result locally, not transfer latent heat to a neighbour a second time.

## Decision drivers

The initial correctness architecture must:

1. conserve one Water-equivalent quantity per Ice/Water/Steam foreground Cell;
2. preserve finite local enthalpy through partial transitions and reversals;
3. keep partial state attached to Matter through move and density swap;
4. gate boiling by a real gas-facing surface and condensation by a real sink or
   bounded deterministic free-air nucleation;
5. fit the DX12 eight-storage-buffer ceiling;
6. add no full-world scratch beyond the two required Current/Next state halves;
7. preserve TE-2, defer Air-pressure force to TE-5, and avoid ignition/TE-4;
8. make reset, staging, authoring, sleep and profiling obligations explicit
   before implementation.

## Options considered

| Option | Quantity | Reversal state | Expansion | Result |
|---|---|---|---|---|
| A — one Matter Cell plus Environment/pressure | Correct 1:1 foreground quantity | Missing by itself | GAS movement now; pressure later | Useful quantity half, insufficient alone |
| B — primary Steam plus owner-linked fragment | Can be correct if ownership never breaks | Requires extra fragment lifecycle | Extra occupied volume is explicit | Rejected: movement, swap, contraction, orphan and scratch complexity |
| C — separate phase quantity/progress | Can model arbitrary sub-cell quantity | Can represent progress | Could drive volume | Rejected as the initial quantity model: duplicates the chosen one-Cell unit and invites same-cell mixed Matter |
| Existing yield-2 path | Incorrect closed-cycle quantity | None | Extra independent Steam | Rejected: creates unowned Water-equivalent Matter |
| **Hybrid A+C** | **One foreground Cell = one unit** | **Dedicated phase enthalpy** | **GAS dispersion now; TE-5 pressure later** | **Accepted with locked amendments** |

## Accepted core and locked amendments

Adopt for future atomic implementation:

**HYBRID A+C — 1:1 WATER-EQUIVALENT QUANTITY WITH DEDICATED PHASE ENTHALPY**

D-018 accepts the quantity, enthalpy, constants, memory cost, no-sink
metastability and atomic-activation core. The v1 text is not accepted
unchanged. This ADR becomes implementation authority only after its real-sink,
buried-completion, radius-2 nucleation, generic-target and hash-provenance
amendments pass the named reference proof and fresh independent review. The
one-shot amended reference proof passed, and the fresh v2 review closed with
unresolved Critical `0` / High `0`. The amended architecture is therefore
accepted as future atomic-implementation authority. This does not authorize
runtime work.

### Quantity and occupancy

- One Ice, Water or Steam foreground Cell is one Water-equivalent unit.
- Ice ↔ Water ↔ Steam identity transitions are 1:1.
- Future Water boiling uses `matter_yield = 1` and creates no extra independent
  Steam Cell.
- Closed phase-family Cell count changes only through explicit Void exit,
  destructive authoring, or a separately named future reaction.
- There is no `phase_quantity`, expansion-fragment owner, or mixed-Matter Cell.
- Visible Steam expansion comes from the conserved Steam Cells' GAS movement
  and spatial dispersion. Physically exact gas volume is not claimed.

### Persistent state

Add exactly two dense `f32` buffers:

```text
phase_energy_current
phase_energy_next
```

Phase energy is Matter-owned. Current/Next is required because movement and
normalization read one immutable world snapshot while writing self-owned next
state. A single read/write buffer would make ownership edges order-dependent.

Exact state increment:

| World | Two f32 buffers |
|---|---:|
| 256×256 | 524,288 B |
| 2048×2048 | 33,554,432 B (32 MiB) |

No packing, `f16`, flag reuse, quantity buffer or new full-world scratch is part
of the candidate.

### Canonical latent meaning

Let `Lf > 0` be fusion energy and `Lv > 0` vaporization energy:

| Material | Canonical `phase_energy` | Valid range | Partial meaning |
|---|---:|---:|---|
| Ice | `-Lf` | `[-Lf, 0]` | value above `-Lf` is partial melting |
| Water | `0` | `[-Lf, Lv]` | negative is partial freezing; positive is partial boiling |
| Steam | `+Lv` | `[0, Lv]` | value below `Lv` is partial condensation |
| Other Matter / EMPTY | `0` | exactly `0` | none |

The source-side identity remains until its latent endpoint. State outside the
range is an invariant failure, never evidence produced by clamping.

### Local enthalpy

With existing gameplay capacities `C_ice = 2.0`, `C_water = 2.5`,
`C_steam = 0.8`, and anchors `T_melt = 0°C`, `T_boil = 100°C`:

```text
S_ice(T)   = C_ice   * (T - 0)
S_water(T) = C_water * (T - 0)
S_steam(T) = C_water * (100 - 0) + C_steam * (T - 100)

H = S_material(T) + phase_energy
```

Identity endpoints are continuous representations of the same `H`: Ice
partial energy `0` equals Water at 0°C/0; Water partial freezing `-Lf` equals
Ice at 0°C/`-Lf`; Water boiling `Lv` equals Steam at 100°C/`Lv`; Steam
condensation `0` equals Water at 100°C/0.

The proposed f32 acceptance is:

```text
abs(H_before - H_after)
<= max(1e-3, 2e-6 * max(1, abs(H_before), abs(H_after)))
```

TE-2 first writes the trial temperature after transferring `Q`. Phase
normalization then preserves the resulting local `H` while repartitioning it
between sensible temperature, latent energy and identity. It performs no
neighbour write and no second latent transfer.

### Plateaus and reversal

- Existing strict initiation hysteresis remains: Ice `T > 2`, Water `T < -2`
  or `T > 100`, Steam `T < 95`.
- Melting/freezing uses the 0°C plateau; boiling/condensation uses the 100°C
  plateau.
- Partial state continues or reverses from energy flow even if its initiating
  surface later disappears.
- Ice/Water and Steam/Water identity changes occur only at their canonical
  endpoints. Water reaching `Lv` is additionally completion-gated as specified
  below.
- Energy beyond completion becomes sensible heat in the target Material.
- Positive Water E is Matter-owned after initiation. Burial does not erase it;
  subsequent heating may increase it and cooling reverses it.
- Water→Steam completion requires either a current gas-facing surface or a
  separately designed TE-5 confinement/pressure-volume transaction that
  explicitly accepts the conversion. No other context may bypass this gate.
- If buried Water reaches `E=Lv` without an accepted completion context, it
  remains Water. `E` stays `Lv`, and excess H becomes Water sensible superheat
  above 100°C. This value-derived **vaporization-ready Water** adds no state,
  flag or Matter identity.
- On cooling vaporization-ready Water, sensible superheat falls to 100°C
  before E reverses below `Lv`. When a valid context later appears, conversion
  to Steam computes target temperature from the same H.
- Canonical Steam with no positive-conductance energy-removal face, or partial
  Steam with no runnable thermal-work face in either direction, may remain
  metastable indefinitely and sleep. Restoring a real cooling or heating-work
  face wakes eligibility and normalizes the preserved H; no spontaneous
  condensation is added.

### Surface and nucleation

Water may initiate boiling only when an orthogonal neighbour is EMPTY
(Atmosphere or Vacuum) or registered GAS Matter.

Steam may initiate surface condensation only when an orthogonal condensed
phase-family or non-EMPTY/non-GAS Matter neighbour satisfies all of:

```text
sink_temperature <= 80°C
sink_temperature <= steam_temperature - 10°C
shared_TE2_face_conductance > 0
shared_TE2_work_predicate permits energy to leave Steam through this face
```

Boundary with zero conductivity is not a sink. Atmosphere and Vacuum remain
non-surface routes. The context and activity passes use the exact shared TE-2
conductance/interface/deadband predicate; no phase-only approximation is
permitted.

Free-air initiation requires canonical Steam below 70°C and the lexicographic
key `(coordinate_hash32, y, x)` to be the strict minimum among eligible Steam
in its 5×5 Chebyshev neighbourhood. `NUCLEATION_RADIUS=2`. Eligibility also
requires a TE-2 face that can actually remove energy. Any partially condensing
Steam within the same radius with matching thermal work vetoes a new seed
while runnable; otherwise the first seed would leave the cold set at the 100°C
plateau and permit a next-tick temporal cascade. The 32-bit mixer and tie-break
are specified in
[`PHASE_THERMODYNAMICS_SPEC.md`](../../specs/PHASE_THERMODYNAMICS_SPEC.md).
Connectivity for this proof is the graph induced by eligible Cells at
Chebyshev distance at most two; its global-minimum key is a seed. With no
existing partial veto, this provides a seed in every finite such component,
makes same-tick seeds impossible within distance two, handles hash ties, and
crosses chunk seams in world coordinates. If a partial veto exists, its radius
already contains owned condensation progress and need not create another seed.
Once partial condensation starts, Matter-owned phase energy—not the moving
coordinate seed—owns progress and blocks new free-air initiation within the
same radius while thermal work can advance or reverse it. Stalled progress
retains E and may sleep but does not reserve its neighbours forever.

For TE3-F08, every sampled 30-tick window is precommitted to:

```text
new free-air initiations
<= max(4, ceil(peak eligible canonical Steam / 8))
```

The local-minimum rule does not make Water immediately: each seed first
accumulates sustained latent-energy removal. Radius 1/2/3 are disclosed in the
v2 reference sweep, but radius 2 is normative and may not silently fall back
to radius 3.

The coordinate mixer exactly reuses Powdergame's internal `edge_priority`
pattern from `engine/gpu/src/movement_claim.wgsl`,
`expansion_claim.wgsl` and `smoke_claim.wgsl`: input multipliers
`0x9E3779B9`/`0x85EBCA6B` and finalizer multipliers
`0x7FEB352D`/`0x846CA68B`. Mapping `x` to `source`, `y` to `target_cell` and
the fixed TE-3 namespace tag `0x54453344` to `tick` is newly authored for this
design; the arithmetic and constants are not. No runtime helper is created and
external code/formulas copied remain zero.

## Locked coefficients

One fixed-seed pure reference sweep selected:

| Constant | Locked value | Bounded target/result |
|---|---:|---|
| `Lf` | `80` | one +25°C Heat pulse at the melt plateau does not complete; two can |
| `Lv` | `480` | 300°C Stone/open Water first Steam target 45–65; result 54 ticks |
| `CONDENSATION_SURFACE_MAX_C` | `80°C` | admits 80°C lid; rejects 82°C sink |
| `CONDENSATION_MIN_DELTA_C` | `10°C` | rejects a 6°C delta; admits a 14°C delta |
| `FREE_AIR_NUCLEATION_MAX_C` | `70°C` | onset target 50–80; result 63 ticks |
| `NUCLEATION_RADIUS` | `2` | strict minimum and active veto in a 5×5 Chebyshev neighbourhood |
| cold-surface completion | derived | target 450–650; result 501 ticks |
| free-air completion | derived | target 900–1300; result 1013 ticks |

Rejected grid values and exact reasons are recorded in
[`PHASE_THERMODYNAMICS_VALIDATION.md`](../../development/PHASE_THERMODYNAMICS_VALIDATION.md).
These are user-accepted gameplay constants, not physical properties and not
runtime values until separately authorized atomic implementation.

## Pressure boundary

Water boiling requests no extra destination. Therefore it does not enter the
extra-yield claim/spawn path and receives no blocked-expansion pressure. Air
mass and phase energy are not converted into pressure. Sealed vapor/background
pressure and structure force remain TE-5 work.

That semantic change would otherwise regress the frozen G5 product chain
`Water heat -> Steam expansion -> confinement Pressure -> rupture -> vent`.
Therefore **activation is atomic**: no production/user-testable source may
activate Water `yield = 1` / blocked pressure `0` until a separately authorized,
accounted TE-5 pressure-volume replacement preserves that chain on the same
source. TE-3 implementation may be developed only behind a disabled/non-
production path before then; it may not replace the current Water path or claim
candidate status. This ADR specifies the dependency, not the TE-5 law.

Historical G5 and TE-2 evidence remains valid only for its recorded source. It
MUST NOT be rebound to the future atomic source. User acceptance of ADR-0006
accepts this sequencing constraint, not a temporary product regression.

D-019 subsequently authorized the docs/reference-only design of that narrow
replacement. Proposed
[`ADR-0007`](ADR-0007-phase-volume-pressure-bridge.md) defines an exclusive
EMPTY movement-opportunity token and an exactly-once existing-gauge-pressure
consequence. It does not amend the phase quantity, enthalpy, completion gate or
atomic-activation rule in this accepted ADR, and it is not implementation
authority. Independent review found that its non-mutating token plus 1:1
occupancy merely moves an EMPTY vacancy through a sealed Water column and
cannot guarantee finite-headspace exhaustion. ADR-0007 is therefore design-
blocked pending a revised capacity-ownership model. Until that model receives a
later explicit user disposition and separate runtime authorization, the
current production Water path remains active and TE-3/TE-5B runtime remains not
started.

D-020 rejected the token and authorized
[`ADR-0008`](ADR-0008-local-vapor-capacity-pressure.md), a final
no-new-persistent-state attempt that recomputed Vapor demand and radius-1 EMPTY
capacity each tick. Its one-shot proof failed a predeclared asymmetric
open-capacity control: proportional shares were capped at one already-satisfied
Steam Cell rather than reallocated to another Steam Cell, producing false
target `100` despite a complete two-Steam/two-EMPTY assignment. TE-5C is
design-blocked. The next decision must explicitly permit persistent
phase-volume state; neither failure changes this ADR's phase quantity, H or
atomic-activation rule.

The historical expansion pipelines remain available for generic non-family
phase rules. After TE-2's float scratch lifetime, `phase_context_propose` fully
overwrites claim with immutable phase-context markers and
`phase_thermodynamics` consumes those markers while fully overwriting proposal:
Ice/Water/Steam emit `NO_PROPOSAL`, yield 1 and pressure 0;
a synthetic or future non-family descriptor retains the historical accounted
`yield > 1` proposal only when its target is non-phase Matter. A generic
non-family `matter_yield > 1` rule MUST NOT target Ice, Water or Steam unless a
later separately approved ownership/writer design writes canonical phase
energy for every destination. Thus the current chain is dormant for
Water/Steam without silently deleting safe generic expansion semantics. A new
quantity/fragment ownership model still requires a new design decision.

## GPU feasibility projection

The candidate uses five pipeline types:

1. `phase_energy_reconcile_movement` after movement ownership;
2. `phase_context_propose`, reusing claim to freeze Air/surface/work context;
3. `phase_thermodynamics` in place of `phase_transition`;
4. `phase_energy_hygiene_identity`, reused after decay, combustion and rupture;
5. `phase_activity_propose` between base and Environment activity proposal.

`phase_context_propose` uses seven storage bindings: Material Current,
temperature Current, phase energy Current, Air mass/energy Current, chunk state
and claim RW. This is where Atmosphere versus Vacuum and matching TE-2 thermal
work are resolved using the existing 128-byte TE-2 thermal-table uniform and
the exact shared node/conductance/interface/deadband predicate.
`phase_thermodynamics` then uses exactly eight storage
bindings: Material Current, temperature Current, phase energy Current, immutable
claim/context, Material Next, temperature Next, phase energy Next and proposal.
The re-encoded 512-byte phase descriptor/surface table, existing 128-byte TE-2
thermal table and params are uniform bindings. Capacity/conductivity is read
from that shared table rather than duplicated. Activity is separate. No
existing maxed movement pass gains bindings.
Sandbox Draw/Erase use a
separate five-storage pre-field phase edit dispatch because the current field
edit pass already has seven storage bindings; that edit dispatch is outside the
timestamped production-tick graph.

With conservative dormant expansion passes retained, the projection is 40
compute passes and 80 timestamp queries, versus 34/68 now. Profiler
resolve/readback storage becomes 1,280 B, an increase of 192 B. Existing small
tables are re-encoded in place, so projected tracked totals are:

| World | No profiler | With projected profiler |
|---|---:|---:|
| 256×256 | 4,721,328 B | 4,722,608 B |
| 2048×2048 | 302,016,816 B | 302,018,096 B |

These are design arithmetic, not measured allocation or performance evidence.

## Consequences

Positive:

- closed phase cycles cannot duplicate foreground quantity;
- latent progress and reversal are explicit and Matter-owned;
- TE-2 `Q` remains single-accounted;
- the eight-storage ceiling and existing ownership strategy survive;
- pressure and ignition remain at their named later Gates.

Costs and residual risks:

- persistent correctness state grows by 32 MiB at 2048²;
- six additional timestamped dispatches are projected;
- the phase path cannot become a production/user-testable candidate by itself;
  separately authorized TE-5 pressure-volume work must activate it atomically
  while preserving the frozen G5 chain;
- buried vaporization-ready Water may store superheat until a completion
  context exists, and isolated Steam may remain metastable by design;
- deterministic radius-2 local minima may still leave visible spatial
  regularity even when the predeclared initiation-rate bound passes;
- user review must judge reference timing and future visual appearance;
- implementation still needs Naga/write-contract, pass-order, movement,
  sleep-on/off, fixture and performance evidence.

## Reuse and non-reuse

Reused: existing Material registry/capacities, strict phase starts,
Current/Next ownership, movement claim, TE-1 identity hygiene pattern, TE-2
thermal work predicate, activity halo, profiler inventory and canonical
staging APIs.

Genuinely new: two phase-energy buffers, enthalpy normalization,
vaporization-ready Water semantics, real-sink metadata, deterministic radius-2
nucleation key, a claim-backed phase-context pass and phase-specific activity
proposal. The context pass adds no allocation.

Rejected: extra independent Steam, owner fragments, a phase-quantity buffer,
threshold-only conversion, random deletion/spreading, fake droplets, output
clamps and latent neighbour writes. External simulation code/formulas copied,
translated or vendored remain `0 files / 0 lines`.

## Approval boundary

D-018 accepts the architecture core with the locked amendments above. The
amended radius-2 reference proof passed its only run and the fresh independent
v2 review closed with no unresolved Critical/High finding. This ADR is
therefore **ACCEPTED FOR FUTURE ATOMIC IMPLEMENTATION** and TE-3D is
**ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS**. That status authorizes no
runtime task by itself. TE-3 runtime is **NOT STARTED**; TE-5B and TE-5C are
**DESIGN BLOCKED** with ADR-0007/ADR-0008 preserved as Proposed history; all
pressure-volume runtime, Air-pressure force, full TE-5, TE-4 and G9-B/C/D/E
remain **NOT STARTED**.
