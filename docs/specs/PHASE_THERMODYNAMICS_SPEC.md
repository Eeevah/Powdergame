# Phase Thermodynamics Specification

- **Status:** ACCEPTED FOR FUTURE ATOMIC IMPLEMENTATION
- **ADR:** [`ADR-0006`](../architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md)
- **Decision:** D-018
- **Runtime:** NOT STARTED
- **Proposed completion bridge:** [`PHASE_VOLUME_PRESSURE_BRIDGE_SPEC`](PHASE_VOLUME_PRESSURE_BRIDGE_SPEC.md), D-019 design only
- **Normative architecture:** Hybrid A+C — 1:1 Water-equivalent quantity with dedicated phase enthalpy

This specification defines the user-accepted core plus locked amendments.
The amended reference proof passed its only run and the fresh independent v2
review closed with unresolved Critical `0` / High `0`; `MUST`, `MUST NOT`,
`SHOULD` and `MAY` are therefore future atomic-implementation authority. No
Rust or WGSL implementation is authorized by this file.

## 1. Scope and exclusions

In scope:

- Ice/Water/Steam quantity, sensible energy and latent progress;
- melting, freezing, boiling, condensation and reversal;
- boiling surface, condensation sink and free-air nucleation predicates;
- movement, identity, staging, reset and activity ownership;
- proposed GPU writers, bindings, settles and validation fixtures.

Excluded:

- Air-pressure or structure force (TE-5);
- ignition, Oxygen, Ash or combustion kinetics (TE-4);
- physically exact gas volume, CFD or velocity;
- new Matter, mixed-Matter Cells, owner fragments or sub-cell quantity;
- threshold or TE-2 Air/thermal coefficient retuning;
- packing, `f16`, optimization or runtime implementation.

Activation constraint: the frozen G5 expansion/confinement product chain may
not regress. Water `yield = 1` and blocked pressure `0` MUST remain disabled in
production until a separately authorized TE-5 pressure-volume replacement is
ready on the same source. This document neither designs nor authorizes that
TE-5 law.

## 2. Ontology and quantity

`EMPTY` remains absence of foreground Matter. Atmosphere and Vacuum remain
Environment states under ADR-0005. Ice, Water and Steam remain registered
foreground Matter.

Each phase-family foreground Cell is exactly one Water-equivalent quantity
unit. Every Ice ↔ Water ↔ Steam transition is 1:1. Water boiling has future
`matter_yield = 1`, creates no independent second Steam, claims no expansion
receiver and creates no blocked-expansion pressure.

The quantity count may change only through:

- a phase-family Cell exiting finite world into Void;
- destructive external authoring such as Erase;
- a separately named and approved future reaction.

GAS movement disperses conserved Steam Cells. TE-5 may later derive background
pressure, but neither Environment Air nor phase energy is converted into
foreground quantity or pressure in TE-3.

Consequently, a phase-only implementation may exist only as disabled staging.
It is not a user-testable candidate and cannot supersede the current production
Water rule until the atomic activation constraint above is satisfied.

## 3. State

The candidate adds exactly:

```text
phase_energy_current : f32[Cell]
phase_energy_next    : f32[Cell]
```

It adds no other persistent full-world phase state and no new full-world
scratch. Phase energy is Matter-owned, not spatial. It follows the same
ownership edge as Material and temperature.

Proposed gameplay constants:

```text
T_MELT                         = 0.0 C
T_BOIL                         = 100.0 C
Lf                             = 80.0
Lv                             = 480.0
CONDENSATION_SURFACE_MAX_C     = 80.0 C
CONDENSATION_MIN_DELTA_C       = 10.0 C
FREE_AIR_NUCLEATION_MAX_C      = 70.0 C
NUCLEATION_RADIUS              = 2 Cells
PHASE_H_ABS_TOL                = 1.0e-3
PHASE_H_REL_TOL                = 2.0e-6
```

Existing capacities remain:

```text
C_ice   = 2.0
C_water = 2.5
C_steam = 0.8
```

### 3.1 Canonical values and valid ranges

| Foreground identity | Canonical phase energy | Inclusive valid range |
|---|---:|---:|
| Ice | `-Lf` | `[-Lf, 0]` |
| Water | `0` | `[-Lf, Lv]` |
| Steam | `Lv` | `[0, Lv]` |
| other Matter | `0` | exactly `0` |
| EMPTY | `0` | exactly `0` |

Interpretation:

- Ice above `-Lf`: partial melting;
- Water below `0`: partial freezing;
- Water above `0`: partial boiling;
- Steam below `Lv`: partial condensation.

All phase-energy values MUST be finite. A value outside the identity's range is
an invariant failure. Tests MUST NOT clamp such a value and then claim valid
evidence.

### 3.2 Source-side identity

Partial progress keeps the source-side identity until the exact endpoint:

```text
Ice E reaches 0       -> Water, E = 0
Water E reaches -Lf   -> Ice,   E = -Lf
Water E reaches Lv    -> Steam, E = Lv only with accepted completion context
Steam E reaches 0     -> Water, E = 0
```

Water at `E=Lv` without a current gas-facing surface or a future accepted TE-5
completion transaction remains Water. If `T>=100°C`, `(Water,E=Lv,T)` is the
value-derived vaporization-ready state; it adds no buffer, flag or identity.

Interior latent states are intentionally hysteretic: total `H` alone does not
select Ice versus freezing Water or boiling Water versus condensing Steam.
Current identity records which endpoint has not yet completed.

## 4. Local enthalpy

Using Celsius-like gameplay temperature:

```text
S_ice(T)   = C_ice   * (T - T_MELT)
S_water(T) = C_water * (T - T_MELT)
S_steam(T) = C_water * (T_BOIL - T_MELT)
             + C_steam * (T - T_BOIL)

H = S_material(T) + phase_energy
```

The Steam sensible anchor includes Water's sensible rise from 0°C to 100°C.
This is required because the capacities differ.

Endpoint identities represent equal enthalpy:

| Endpoint | Left representation | Right representation | H |
|---|---|---|---:|
| melt complete | Ice, 0°C, E=0 | Water, 0°C, E=0 | `0` |
| freeze complete | Water, 0°C, E=-Lf | Ice, 0°C, E=-Lf | `-Lf` |
| boil complete | Water, 100°C, E=Lv | Steam, 100°C, E=Lv | `C_water*100 + Lv` |
| condense complete | Steam, 100°C, E=0 | Water, 100°C, E=0 | `C_water*100` |

For every normalization:

```text
error = abs(H_before - H_after)
tolerance = max(
    PHASE_H_ABS_TOL,
    PHASE_H_REL_TOL * max(1, abs(H_before), abs(H_after))
)
error <= tolerance
```

## 5. Causal order and no double counting

One phase step is:

```text
TE-2 reads Current neighbours and transfers Q
-> TE-2 writes trial temperature_next
-> trial temperature settles to temperature_current
-> phase_context_propose reads settled Matter/T/E plus Air and fully writes
   immutable context markers into the now-dead claim scratch
-> phase_thermodynamics reads local material/T/E plus context markers
-> phase_thermodynamics writes local material/T/E Next
-> joint phase settle
```

`phase_context_propose` writes flags only, not energy. `phase_thermodynamics`
MUST NOT add heat to a neighbour. It only repartitions the already transferred
local `Q`. A latent debit/credit applied to a neighbour would count the same
transfer twice.

## 6. Pure normalization contract

The proposed pure operation is:

```text
normalize_phase_enthalpy(
    material,
    trial_temperature,
    phase_energy,
    local_surface_and_work_context,
    completion_context
) -> {
    material_next,
    temperature_next,
    phase_energy_next,
    transition_kind
}
```

It reads one immutable local/neighbor snapshot and writes self only. Neighbor
order MUST NOT affect the result. A bounded piecewise evaluation may cross both
phase endpoints when an extreme finite input contains enough energy; all excess
remains sensible in the final identity.

### 6.1 Existing strict initiation starts

No threshold is retuned:

| Source | Initiation condition |
|---|---|
| Ice melt | `T > 2°C` |
| Water freeze | `T < -2°C` |
| Water boil | `T > 100°C` and gas-facing |
| Steam surface condense | `T < 95°C` and eligible sink |
| Steam free-air condense | `T < 70°C` and deterministic seed |

At exact equality, initiation does not occur. Existing partial progress ignores
the initiation gate and follows actual energy work in either direction. It
does not ignore the separate Water→Steam completion gate. A partial Steam Cell
with no runnable thermal work retains its identity, E and H and may sleep.

### 6.2 Fusion plateau

Ice melting:

1. Compute the trial `H`.
2. Once initiated, use 0°C and increase Ice phase energy toward `0`.
3. At `0`, change identity 1:1 to Water/E=0.
4. Any higher `H` becomes Water sensible temperature and may continue into
   boiling only if its independent surface gate is satisfied.

Water freezing is symmetric:

1. Once initiated, use 0°C and decrease Water phase energy toward `-Lf`.
2. At `-Lf`, change identity 1:1 to Ice/E=`-Lf`.
3. Any lower `H` cools Ice below 0°C.

Reheating partially freezing Water restores E toward `0` before Water warms
above the plateau. Cooling partially melting Ice restores E toward `-Lf`
before Ice cools below the plateau.

### 6.3 Vaporization plateau and completion gate

Gas-facing Water boiling initiation:

1. Compute trial `H` after TE-2 transfer.
2. Initiation from canonical Water requires `T>100°C` and a gas-facing
   neighbour.
3. Once initiated, positive E is Matter-owned. Later burial does not erase or
   pause accounting: heating may increase E and cooling reverses it.
4. While `C_water*100 < H < C_water*100+Lv`, represent the state as Water at
   100°C with `E=H-C_water*100`.
5. Water→Steam completion is permitted only when either:
   - a current gas-facing surface exists; or
   - a separately designed TE-5 confinement/pressure-volume transaction has
     explicitly accepted this conversion.
6. If `H>=C_water*100+Lv` without either context, remain Water with `E=Lv` and
   `T=(H-Lv)/C_water`. This is vaporization-ready Water; no clamp, deletion,
   extra Matter or fake pressure occurs.
7. When completion becomes permitted, convert 1:1 to Steam/E=`Lv` and compute
   `T=100+(H-(C_water*100+Lv))/C_steam` from the same H.

The TE-5 boolean above is a contract-only placeholder. The phase-only design
binds no TE-5 state and changes no pass/binding count. A later separately
approved TE-5 design must define the causal transaction and how its accepted
result reaches the atomic source; absence of that design is false, never an
implicit completion permission.

D-019 supplies a proposed, not-yet-user-approved definition in
[`PHASE_VOLUME_PRESSURE_BRIDGE_SPEC`](PHASE_VOLUME_PRESSURE_BRIDGE_SPEC.md):
an eligible endpoint attempt—already initiated positive-E/ready Water, or a
current gas-facing crossing—must obtain a valid targeted/blocked relief word
before its provisional Steam Next state can settle. Claim and pressure outcome
resolve before joint identity/phase/pressure settle. Buried canonical Water and
non-gas-facing extreme Ice cannot initiate through this transaction; a
Void-first attempt is explicitly deferred as ready Water. This overlay does
not change H/E or grant runtime authority. Its finite-headspace capacity model
is currently design-blocked, so the placeholder remains inactive and the
current production Water path remains.

Eligible Steam condensation is symmetric:

1. Once initiated, use 100°C and decrease Steam phase energy toward `0`.
2. At `0`, change identity 1:1 to Water/E=0.
3. Any lower `H` cools Water below 100°C and may continue to freezing only at
   the existing strict freeze start.

Cooling vaporization-ready Water first removes sensible superheat until 100°C,
then reduces positive E before Water cools below the plateau. Reheating
partially condensing Steam restores E toward `Lv` before Steam superheats when
real thermal work exists. Losing a surface or moving away from the nucleation
coordinate MUST NOT discard already-owned progress; a no-work partial Steam
state may remain metastable and sleep without changing E.

### 6.4 Buried Water and ungated Steam

Canonical Water above 100°C that never initiated remains Water/E=0 and stores
its full sensible H. Positive-E buried Water follows §6.3 and may reach the
vaporization-ready Water representation. Reopening a gas surface normalizes
the same H and may complete 1:1.

Canonical Steam with no positive-conductance energy-removal face, or partial
Steam with no runnable thermal-work face in either direction, remains Steam
indefinitely, retains finite E/H and may sleep. Examples include Steam in
Vacuum and Steam enclosed only by zero-conductivity faces. No spontaneous
magic condensation is permitted. Restoring a real cooling or heating-work face
wakes eligibility; threshold-only identity change or energy deletion remains
forbidden.

## 7. Surface predicates

All neighbour tests are orthogonal unless the nucleation section defines its
wider 5×5 Chebyshev neighbourhood. World coordinates cross chunk seams
normally.

### 7.1 Gas-facing Water

Water is gas-facing when any orthogonal neighbour is:

- `EMPTY`, regardless of Atmosphere versus Vacuum; or
- registered Matter whose compiled movement class is GAS.

Thus Water/Atmosphere, Water/Vacuum, Water/Steam and Water/Smoke qualify.
Liquid/static/powder burial does not.

### 7.2 Condensation sink

A Steam Cell has a surface sink when an orthogonal neighbour is either:

- phase-family condensed Matter (Ice or Water via compiled phase traits); or
- non-EMPTY, non-GAS Matter;

and its actual Matter temperature satisfies both:

```text
neighbor_T <= CONDENSATION_SURFACE_MAX_C
neighbor_T <= steam_T - CONDENSATION_MIN_DELTA_C
```

The same face MUST also have strictly positive shared TE-2 conductance and the
exact shared TE-2 node/interface/deadband work predicate MUST say energy can
leave Steam through that face. Boundary with conductivity zero is therefore
not an eligible sink even when cold. Atmosphere and Vacuum remain non-surface
routes. `phase_context_propose`, normalization and `phase_activity_propose`
MUST consume the same predicate result; no duplicate phase approximation is
allowed.

Atmosphere/Vacuum alone is not a surface sink. A hot Stone wall is not a sink.
The table supplies movement class and phase traits; the shader MUST NOT grow a
list of material-name branches.

## 8. Free-air nucleation

Free-air **initiation eligibility** applies only to canonical Steam (`E == Lv`)
that:

- is below `FREE_AIR_NUCLEATION_MAX_C`;
- lacks an eligible surface sink; and
- has at least one face for which the shared TE-2 thermal-work predicate can
  remove energy from the Steam Cell.

A partial condensing Steam Cell (`0 < E < Lv`) with matching thermal work is an
**active owned-progress veto** for every Cell at Chebyshev distance at most
`NUCLEATION_RADIUS=2`, regardless of its plateau temperature. An
initiation-eligible Cell becomes a new seed only when:

1. no Steam Cell within radius 2 has thermally runnable partial condensation
   progress; and
2. its immutable coordinate key is strictly smaller than every other
   initiation-eligible canonical Steam key in its 5×5 Chebyshev neighbourhood.

The active partial veto is required because normalization raises an initiating Cell
to the 100°C plateau. Without the veto, that Cell would leave the cold
eligibility set after one tick and a neighbour could become a fresh seed every
following tick. Progress remains a veto after movement while its matching
thermal-work predicate remains true because E follows the Matter owner. A
stalled partial Cell retains E and may sleep, but does not permanently reserve
its neighbours; another Cell may seed only if it has its own energy-removal
face. Completion to Water may then create a real surface front; loss through
Void/destructive editing may allow a replacement seed.

The 32-bit mixer exactly reuses the internal `edge_priority` arithmetic from
`engine/gpu/src/movement_claim.wgsl`, `expansion_claim.wgsl` and
`smoke_claim.wgsl`:

```text
h = u32(x)
    ^ (u32(y) * 0x9E3779B9)
    ^ (0x54453344 * 0x85EBCA6B)
h = (h ^ (h >> 16)) * 0x7FEB352D
h = (h ^ (h >> 15)) * 0x846CA68B
h = h ^ (h >> 16)

key = (h, y, x)  // lexicographic total order
```

The four constants and finalizer sequence are exact internal reuse. Only the
coordinate-to-existing-input mapping (`source=x`, `target_cell=y`, fixed
`tick=0x54453344`) is newly authored. The TE-3 tag is a namespace input, not a
new mixer constant. No external code/formula was consulted or copied and no
runtime helper is added by this docs task.

The `(y,x)` suffix resolves every 32-bit hash tie. Therefore, for a frozen
snapshot with no partial veto, where a component is induced by eligible Cells
whose Chebyshev distance is at most two:

- every finite initiation-eligible connected component has at least its global-minimum
  seed;
- two same-tick seeds cannot have Chebyshev distance at most two;
- a multi-Cell component cannot convert wholly from one initiation decision;
- chunk partitioning cannot change the answer;
- shifting/moving a cloud may change which canonical Cell is a seed;
- once E drops below `Lv`, progress moves with Matter, no longer depends on the
  coordinate key and vetoes radius-2 new free-air initiation while thermally
  runnable;
- completion or Void release removes the active veto, while a stalled no-work
  partial keeps E but does not reserve space.

TE3-F08 additionally requires in every sampled 30-tick window:

```text
new_free_air_initiations
<= max(4, ceil(peak_eligible_canonical_steam / 8))
```

Radius 1 and radius 3 are disclosure comparisons only. Radius 2 is normative;
a radius-2 hard-property or 30-tick-bound failure blocks the design and MUST
NOT silently select radius 3.

The seed only initiates latent progress. It does not create fake presentation
or immediate Water. A real Water identity appears only at E=0 after sustained
energy removal. Static seed sparsity is insufficient evidence: the future
validation MUST also bound temporal initiation rate, moving partial shadows and
post-completion surface-front behavior.

## 9. Movement and identity ownership

| Path | Required `phase_energy_next` |
|---|---|
| no movement, same identity | preserve source value |
| Matter moves into EMPTY | destination receives source value; vacated EMPTY gets 0 |
| density swap | each Matter receives its owning peer's value |
| Void exit | vacated in-domain EMPTY gets 0; Void has no state |
| phase self transition | normalized partial or canonical endpoint value |
| phase-family → non-phase/EMPTY | 0 |
| non-phase → Ice/Water/Steam external staging | `-Lf / 0 / Lv` |
| decay, fuel consumption, rupture, Erase to EMPTY | 0 |
| Draw Ice/Water/Steam | `-Lf / 0 / Lv` in both halves |
| Draw other Matter | 0 in both halves |
| Heat/Cool | no direct phase-energy write |
| reset/preset/scenario/benchmark | both halves byte-identical and canonical |

Every writer that can place phase-family identity MUST also write canonical or
owned phase energy. A bypass writer is an implementation blocker. Phase energy
MUST NOT be packed into combustion/Smoke flags.

The existing Sandbox field edit pass already owns seven storage bindings.
Phase energy MUST therefore use a separate, non-timestamped pre-field edit
dispatch rather than extending that pass beyond the eight-storage limit. Its
storage bindings are commands RO, Material Current/Next RO and phase energy
Current/Next RW (`3 RO + 2 RW = 5`). It observes pre-edit occupancy just like
the flag/Environment edit passes: accepted Draw writes the target canonical
value, Erase writes zero and Heat/Cool preserves both halves. This edit dispatch
does not enter the 40-pass production-tick profiler count.

## 10. Proposed pass graph

The existing 34-pass TE-2 graph is the source anchor. The conservative
candidate retains the historical expansion transaction as a deterministic
no-op, adds six timestamped dispatch uses, and projects 40 passes:

```text
0       activity_wake
1..5    movement propose/claim/commit, flag hygiene, Environment reconcile
6       phase_energy_reconcile_movement
7..10   TE-2 Air scale/commit and thermal scale/commit
11      phase_context_propose (fully writes claim as immutable u32 markers)
12      phase_thermodynamics (replaces phase_transition and fully writes proposal)
13..19  dormant expansion claim/receiver/spawn/pressure, flag hygiene,
        Environment reconcile
20..23  decay, flag hygiene, phase-energy hygiene, Environment reconcile
24..30  combustion/Smoke transaction, flag hygiene, phase-energy hygiene,
        Environment reconcile
31      pressure
32..35  rupture, flag hygiene, phase-energy hygiene, Environment reconcile
36      base activity_propose without old threshold-only phase candidate
37      phase_activity_propose
38      environment_activity_propose
39      activity_reduce
```

Joint phase-energy copies occur with the corresponding Matter settles after
movement, phase, decay, combustion and rupture.

### 10.1 Binding ceilings

| Future pass | Storage RO | Storage RW | Total | Other bindings |
|---|---:|---:|---:|---|
| phase-energy movement reconcile | 6 | 1 | 7 | params uniform |
| phase context propose | 6 | 1 | 7 | params + phase descriptor + existing TE-2 thermal-table uniforms |
| phase thermodynamics | 4 | 4 | **8** | params + phase descriptor + existing TE-2 thermal-table uniforms |
| phase-energy identity hygiene | 5 | 1 | 6 | params uniform |
| phase activity propose | 6 | 1 | 7 | params + phase descriptor + existing TE-2 thermal-table uniforms |
| Sandbox phase edit (outside tick graph) | 3 | 2 | 5 | params uniform |

The exact phase-context storage order is Material Current, temperature Current,
phase energy Current, Air mass Current, Air energy Current, chunk state and
claim RW. It fully overwrites one `u32` marker per Cell after claim's TE-2
receiver-scale lifetime is dead. Markers encode skip/runnable, gas-facing,
real positive-conductance surface-sink, canonical free-air energy-removal and
active radius-2 partial-veto facts. In the phase-only graph the completion bit
is exactly gas-facing; a future TE-5 accepted-transaction source requires its
own separately reviewed predecessor contract rather than an assumed bit.
This pass is the only phase-context reader of Air, so Atmosphere and Vacuum are
not guessed from `Material == EMPTY`. It binds the existing 128-byte TE-2
conductivity/capacity uniform and uses the exact TE-2 node, conductance,
interface and deadband work predicate; it does not duplicate a divergent
phase-only approximation or allocate another table. `phase_activity_propose`
binds the same uniform and predicate.

The exact phase-activity storage order is Material Current, temperature
Current, phase energy Current, Air mass Current, Air energy Current, chunk
state and activity proposal RW. It therefore has six RO plus one RW storage
binding, recomputes the current thermal-work predicate after intervening
identity writers, and does not read the earlier claim/context snapshot.

The exact phase-thermodynamics storage order is:

1. Material Current RO
2. temperature Current RO
3. phase energy Current RO
4. claim/context marker RO
5. Material Next RW
6. temperature Next RW
7. phase energy Next RW
8. proposal RW

It has no Air, chunk-state or `cell_activity` binding. A `CONTEXT_SKIP` marker
replaces its chunk-state check; every invocation still copies self and fully
overwrites proposal, so sleeping Cells cannot expose stale proposal data. The
current 512-byte phase table is
re-encoded as a compact 32-byte × 16 descriptor containing targets, yields,
starts, pressure metadata and compiled phase/surface traits, and is bound here
as a uniform. Capacity and conductivity come directly from the existing TE-2
thermal-table uniform rather than a duplicate. The same phase buffer may retain a
storage-buffer view for historical expansion readers. No new persistent table
allocation is projected. Existing eight-storage movement commit is not
enlarged.

The 32-byte descriptor layout is two packed rule headers (`target`, bounded
yield and enabled/direction bits), two `f32` thresholds, two `f32` blocked-
pressure values and two packed phase/surface trait words. The buffer
usage must include both `UNIFORM` and `STORAGE` if historical expansion readers
retain their storage view; this is one allocation, not a shadow table.

### 10.2 Dormant expansion safety

`phase_context_propose` first fully overwrites claim after its TE-2 float use.
`phase_thermodynamics` consumes that immutable snapshot and fully overwrites
proposal for every Cell. Ice/Water/Steam write `NO_PROPOSAL`, have yield 1 and zero
blocked-pressure metadata. A non-family descriptor retains the historical
generic transition/proposal semantics, including an accounted `yield > 1`,
only when the target is non-phase Matter;
this prevents the phase-energy change from silently disabling the generic
expansion path. Expansion claim then fully overwrites claim after the phase
consumer and before later claim readers.

On the current registry the expansion chain is dormant because the only phase
family has yield 1. It MUST create no Matter, pressure or Environment receiver
claim for Ice/Water/Steam. A synthetic non-family yield-2 structural fixture
must target non-phase Matter and still emit/consume one valid proposal. A
generic non-family `matter_yield>1` rule MUST NOT target Ice, Water or Steam
unless a later separately approved ownership/writer design writes canonical
phase energy for every destination. Larger or new ownership models remain a
separate design decision.

### 10.3 Proposed TE-5B mode overlay

D-019's proposed bridge reuses the same Section 10 expansion window without an
additional pass. In that future combined design, `phase_thermodynamics` still
fully overwrites proposal. An eligible vaporization endpoint attempt first
replays resulting-Steam GAS First-Match: an in-domain EMPTY outcome writes
`REQUEST_VOLUME_RELIEF` with its target, an earlier legal density swap or no
EMPTY writes blocked payload zero, and a Void-first outcome writes
`REQUEST_NONE` while retaining ready Water. Accepted attempts write
provisional Steam and settle only after their claim/consequence transaction.
Other phase-family Cells write `REQUEST_NONE`.

The shared two-bit/30-bit encoding, mixed-mode claim domain, mode-specific
Environment filters and exactly-once pressure consequences are normative only
in
[`PHASE_VOLUME_PRESSURE_BRIDGE_SPEC`](PHASE_VOLUME_PRESSURE_BRIDGE_SPEC.md).
They do not alter the accepted phase-energy ownership or its 40-pass projection:
the TE-5B delta is zero passes, zero queries and zero persistent/full-world
bytes. The descriptor's existing family consequence-pressure slot provides
the data-driven `100.0` value when a finite extreme Ice source with a current
gas-facing context normalizes through Water to Steam in one invocation; no
Water-name branch is permitted. A packed existing trait word supplies the
registry-derived Steam swap stop without another binding.

This subsection is a dependency cross-reference, not acceptance of ADR-0007.
If its proof or independent review leaves a Critical/High issue, the overlay is
blocked while this D-018 phase architecture remains accepted but inactive.

### 10.4 D-020 TE-5C replacement outcome

D-020 rejects the TE-5B token overlay. Proposed
[`LOCAL_VAPOR_CAPACITY_PRESSURE_SPEC`](LOCAL_VAPOR_CAPACITY_PRESSURE_SPEC.md)
instead lets initiated/ready Water complete 1:1 and derives a later pressure
target from the current phase-energy population and EMPTY capacity. It
projected one capacity-sum pass after Smoke, for 41 passes and 82 queries.

The locked proof failed its predeclared open-capacity control because the
proportional per-EMPTY allocation can discard capacity at a saturated Cell and
leave another Cell falsely compressed. TE-5C is therefore **DESIGN BLOCKED**
and inactive. This accepted phase specification remains unchanged; the next
pressure-volume decision must explicitly permit persistent state.

## 11. Activity and sleep

The old phase pass activity marker and threshold-only phase candidate are
removed together. `phase_activity_propose` uses the same starts, surface/sink,
nucleation and thermal-work predicates as normalization.

It sets `ACTIVITY_THERMAL` when any remaining phase work can change state:

- an initiation predicate is currently eligible, including a real
  positive-conductance sink rather than a geometric-only cold face;
- partial latent state has an adjacent TE-2 thermal face whose shared
  deadband predicate permits energy flow;
- a stored superheated/supercooled state has just become eligible;
- vaporization-ready Water has gained a valid completion context.

A partial plateau with no eligible energy flow is stalled state, not active
progress, and MAY sleep. A neighbour edit/movement/thermal frontier wakes the
existing safety halo before work resumes. Completed or stalled equilibrium
bulk MUST be able to sleep. Sleep-on/off modes MUST produce equivalent
Material, temperature and phase energy for the same executed ticks.

An identity transition that leaves no remaining work need not retain a
one-tick diagnostic marker; its resulting identity is the observable state.
No extra event buffer is authorized.

## 12. Tracked allocation and profiler projection

The two buffers add 524,288 B at 256² and 33,554,432 B at 2048². The 40-pass
projection uses 80 timestamps and two 640-byte profiler buffers, 1,280 B total.

| World | Current TE-2 no profiler | TE-3D projected no profiler | TE-3D projected with profiler |
|---|---:|---:|---:|
| 256² | 4,197,040 B | 4,721,328 B | 4,722,608 B |
| 2048² | 268,462,384 B | 302,016,816 B | 302,018,096 B |

These numbers exclude transient diagnostic staging and opaque driver/query-set
storage, matching the current tracked-report boundary. They are arithmetic
projections, not runtime measurements.

## 13. Required invariants

- **PH-INV-001 — Unit quantity:** One phase-family foreground Cell equals one Water-equivalent quantity unit.
- **PH-INV-002 — Closed-cycle count:** Closed Ice/Water/Steam transitions do not change Water-equivalent Cell count.
- **PH-INV-003 — No unowned yield:** No Ice/Water/Steam descriptor requests unowned `matter_yield > 1`; a generic non-family proposal remains explicitly owned by the historical expansion transaction and targets non-phase Matter only.
- **PH-INV-004 — Finite enthalpy state:** Temperature plus phase energy represents one finite local enthalpy state.
- **PH-INV-005 — H preservation:** Phase normalization preserves local H within tolerance.
- **PH-INV-006 — Q exactly once:** Latent heat is never applied twice to a neighbour.
- **PH-INV-007 — Reversible progress:** Partial progress is reversible and never silently reset.
- **PH-INV-008 — Exact zero outside family:** Non-phase Matter and EMPTY have exact phase energy 0.
- **PH-INV-009 — Matter ownership:** Movement carries phase energy with Matter identity.
- **PH-INV-010 — Surface boiling:** Boiling initiation is surface-gated.
- **PH-INV-011 — Condensation gate:** Condensation initiation is sink- or nucleation-gated.
- **PH-INV-012 — Bounded nucleation:** Free-air nucleation is deterministic, sparse and bounded across both space and successive ticks; thermally runnable partial progress vetoes radius-2 new seeds without making stalled progress a permanent reservation.
- **PH-INV-013 — No traffic jam:** No persistent mid-air Water/Steam checkerboard traffic jam.
- **PH-INV-014 — No fake boil pressure:** No Water-boiling blocked-expansion pressure is generated in TE-3.
- **PH-INV-015 — TE-5 boundary:** Air-pressure force remains unimplemented until TE-5.
- **PH-INV-016 — Valid ranges:** Every phase-energy state is finite and inside the target Material's valid range.
- **PH-INV-017 — Matching work:** Phase progress and activity use matching work predicates.
- **PH-INV-018 — Sleep:** Equilibrium phase bulk can sleep.
- **PH-INV-019 — Atomic G5 continuity:** Production activation of Water yield 1 is atomic with a separately approved pressure-volume replacement; no released source loses the frozen expansion/confinement chain.
- **PH-INV-020 — Context snapshot:** Phase eligibility uses one fully written claim-backed Matter/Air context snapshot; Atmosphere/Vacuum is never inferred from EMPTY and no context marker races a claim/proposal writer.
- **PH-INV-021 — Real condensation sink:** Surface condensation initiation requires the exact positive-conductance TE-2 energy-removal face predicate; a cold K=0 Boundary is not a sink or phase-activity source.
- **PH-INV-022 — Completion permission:** Water→Steam completion requires a current gas-facing surface or an explicit accepted future TE-5 transaction; vaporization-ready Water preserves H without fake pressure.
- **PH-INV-023 — No-work metastability:** Canonical or partial Steam without runnable thermal work may retain finite identity/E/H indefinitely and sleep; restoring a real face wakes it.
- **PH-INV-024 — Radius-2 nucleation:** Seed competition and active partial veto use the same Chebyshev radius 2, preserve the predeclared 30-tick bound and never silently substitute another radius.
- **PH-INV-025 — Generic target hygiene:** A generic non-family yield greater than one cannot target Ice/Water/Steam without a separately approved destination phase-energy ownership/writer design.

## 14. Evidence boundary

The fixed-seed reference proof establishes only the pure math subset. It does
not prove WGSL ownership, bindings, pass order, movement, sleep, performance,
visual quality or user acceptance. Those remain implementation/user gates in
[`PHASE_THERMODYNAMICS_VALIDATION.md`](../development/PHASE_THERMODYNAMICS_VALIDATION.md).
