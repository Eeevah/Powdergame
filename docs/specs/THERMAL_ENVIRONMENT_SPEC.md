# Powdergame Thermal Environment Specification

- **Status:** TE-2 passive thermal Environment candidate; user review pending
- **Architecture:** ADR-0005 / D-013 / D-014 / D-015
- **Runtime boundary:** Air transport and unified passive thermal exchange are implemented at source `fb7e568e21012b6067269f4e1b82c36c865023d0`; Air-pressure force and TE-3+ remain disabled

## 1. State model

Foreground occupancy and Environment are separate layers.

```text
Foreground:
  material_id
  matter_temperature
  flags

Mechanical:
  pressure  // existing gameplay gauge overpressure

Environment:
  air_mass_current / air_mass_next
  air_energy_current / air_energy_next
```

Air is not Matter. An occupied Cell has canonical zero Air. An EMPTY Cell can be Atmospheric, low pressure, or Vacuum. Void is not a Cell.

The exact-zero Vacuum boundary is locked. Other coefficients remain
implementation data inside these finite domains:

```text
VACUUM_THRESHOLD = 0
0 < AIR_PRESENT_THRESHOLD <= STANDARD_AIR_MASS <= AIR_MASS_MAX
0 < AIR_ENERGY_MAX
0 < AIR_FLOW_RATE < infinity
0 <= permeability <= 1
0 < AIR_MAX_OUTFLOW_FRACTION <= 1
0 < THERMAL_BASE_STEP < infinity
0 < THERMAL_MAX_MIX_FRACTION <= 1
epsilon < capacity < infinity
0 <= conductance < infinity
```

Only exact `(0, 0)` canonicalizes as Vacuum. Positive finite residual mass and
energy remain conserved low-pressure Air. Rounding guards may reduce a flux,
never clamp a donor negative, and apply equal-and-opposite pair correction
within the declared error tolerance.

## 2. Derived Air state

For non-Vacuum Air:

```text
specific_energy = air_energy / air_mass
T_air_absolute_like = specific_energy / AIR_HEAT_CAPACITY
T_air_celsius_like = T_air_absolute_like - AIR_ZERO_OFFSET
P_air_absolute_like = P_STANDARD
                    * (air_mass / STANDARD_AIR_MASS)
                    * (T_air_absolute_like / AMBIENT_T_ABSOLUTE_LIKE)
```

These are gameplay quantities, not kg, joules, kelvin, or pascals. Derived values must be finite. Vacuum has no transferable Air temperature and has zero Air pressure.

## 3. Hard invariants

### TH-INV-001 — Single foreground occupancy

One Cell has at most one foreground Matter. Air is a Field and does not create mixed foreground occupancy.

### TH-INV-002 — Explicit medium identity

`material_id == EMPTY` does not identify Atmosphere or Vacuum. Environment state must be inspected.

### TH-INV-003 — Passive maximum principle

Without Heat/Cool, combustion, phase release, or an external reservoir, passive exchange cannot move a node outside the current participating-node temperature extrema beyond numerical tolerance.

### TH-INV-004 — Equal-and-opposite face exchange

Matter↔Matter, Matter↔Air and Air↔Air passive thermal exchange uses one face flux computed from one Current snapshot and applied with opposite signs.

### TH-INV-005 — Vacuum canonicalization

Vacuum has exact `(mass, energy) = (0, 0)`, supplies no Air flow, Air conduction, Matter↔Air exchange, or Air pressure. Direct Matter contact conduction remains valid. Radiation is outside the initial program.

### TH-INV-006 — Finite non-negative Environment

Air mass and energy are finite and non-negative. Non-Vacuum specific energy and temperature are finite and inside a gameplay safety range. Safety sanitization is a hard diagnostic, not silent evidence of correctness.

### TH-INV-007 — One settled writer per causal stage

One commit writes each authoritative Next field in a causal stage. A later stage may write it only after explicit settle. Every occupancy-changing commit has exactly one Environment reconcile before the next causal reader.

### TH-INV-008 — No duplicate Air transport

Air-flow stages transport mass and donor specific energy. Unified thermal stages conduct heat. No term is applied in both paths.

### TH-INV-009 — No stale Environment under Matter

Every post-settle occupied Cell has exact zero Air. Every material identity replacement clears incompatible Matter-local progress.

### TH-INV-010 — Honest observability

Telemetry and Inspector language distinguish Matter temperature, Air temperature, Atmosphere/low-pressure/Vacuum, mechanical overpressure, and sample freshness. A held Matter sample must not be relabelled as current Environment state.

### TH-INV-011 — Spawn displacement is lossless or blocked

An EMPTY-to-Matter spawn must move the target Air to a deterministically claimed orthogonal EMPTY receiver. If none exists, the spawn does not commit. TE-1 never deletes physical Air and never invents Air-to-pressure coupling.

## 4. Passive reference formulas

### 4.1 Air outflow and energy advection

For an open face from donor `i` to receiver `j`:

```text
raw_out(i→j) = AIR_FLOW_RATE
             * permeability(i,j)
             * max(P_i - P_j, 0)

scale_i = min(1,
              AIR_MAX_OUTFLOW_FRACTION * mass_i
              / max(sum_raw_out_i, epsilon))

F(i→j) = raw_out(i→j) * scale_i
E_advected(i→j) = F(i→j) * specific_energy_i
```

The commit self-writes all incoming and outgoing terms. Source-free internal faces conserve mass and advected energy within tolerance. Air conduction is not part of this stage.

### 4.2 Unified passive thermal exchange

Each Cell exposes at most one active thermal node: Matter, Air, or none for Vacuum.

```text
lambda_i = min(1,
               THERMAL_MAX_MIX_FRACTION * capacity_i
               / max(THERMAL_BASE_STEP * sum_conductance_i, epsilon))

edge_step_ij = THERMAL_BASE_STEP * min(lambda_i, lambda_j)
Q_ij = edge_step_ij * G_ij * (T_j - T_i)
```

`G_ij` is symmetric. Matter applies `sum(Q)/capacity`; Air applies `sum(Q)` to energy. Vacuum uses zero conductance. Source-free output is a convex combination of current participating temperatures and the energy-like internal sum cancels pairwise.

The thermal deadband is an exact shared work gate:

```text
if abs(T_j - T_i) <= THERMAL_DEADBAND_C:
    Q_ij = 0
else:
    effective_delta = T_j - T_i
```

`THERMAL_DEADBAND_C = 0.01 °C`. The deadband is never subtracted from the
eligible delta. Physics and thermal activity use the identical predicate
`abs(delta) > THERMAL_DEADBAND_C`; `lambda` and
`THERMAL_MAX_MIX_FRACTION` remain the stability bounds.

Explicit sources are Heat/Cool authoring, combustion chemical heat, boundary reservoirs, and later phase latent release. Each is reported separately.

## 5. Occupancy and Environment reconcile

| Occupancy path | Environment result |
|---|---|
| Matter moves into EMPTY | destination parcel moves to vacated source; destination zero |
| Matter↔Matter density swap | both remain occupied; both Air states remain zero |
| Matter exits to Void | vacated source starts Vacuum |
| Matter→Matter self transition | Air remains zero; incompatible progress clears |
| Phase/Smoke spawn | target parcel moves to claimed orthogonal EMPTY receiver; otherwise spawn blocked |
| Rupture, decay, fuel consumption | new EMPTY starts Vacuum; TE-2 flow may refill |
| Sandbox Draw | external authoring removes target Environment and writes Matter with zero Air |
| Sandbox Erase | external authoring seeds the current world's default Environment |
| Preset/reset/staging | one canonical image stages both Environment halves exactly |
| Future Vacuum edit | separate Environment command sets EMPTY mass/energy to zero |

Receiver selection is deterministic, bounded, and arbitrated. A receiver is
not any winning Matter destination in that stage and has headroom for the whole
displaced parcel under `AIR_MASS_MAX` and `AIR_ENERGY_MAX`. Receiver mass and
energy add exactly; partial transfer, clamp, overflow and deletion are
forbidden. Phase failure is an explicit Environment-blocked expansion and
feeds the existing phase-pressure consequence through a mandatory separate
pass. Smoke
generation is rejected for that tick when no receiver is available.

One persistent full-world `u32 environment_receiver_claim` scratch is part of
the correctness baseline. For phase and Smoke, a potential receiver derives
the preferred orthogonal candidate of neighbouring winning Matter targets
from the still-live original Matter claim, chooses the smallest target index,
and writes `target + 1`. Spawn commit checks that identity before changing
Matter. Environment reconcile then reads pre/post Matter, the original claim,
the receiver claim and four Air buffers—exactly eight storage bindings—moves
the complete parcel, and joint settles. Failure commits neither Matter nor
Environment. Structural tests pin this order and scratch live range.

For phase only, the existing expansion-pressure pass runs first. A mandatory
`environment_blocked_expansion_pressure` pass then detects an original winning
target whose receiver claim failed and adds the same blocked-expansion source
to `pressure_next` before pressure settle. Its storage layout is material,
temperature, phase table, proposal, original claim, receiver claim and
read/write pressure Next (seven), plus uniform params. It neither reads nor
deletes Air state. Environment reconcile, identity hygiene and joint settle
complete the transaction. Smoke has no corresponding pressure pass.

Direct staging helpers, benchmark staging, scenario upload, Sandbox presets and reset must all use one canonical Environment-image contract. A bypass writer is a TE-1 blocker.

## 6. Causal pass contract

The initial gate ordering is deliberately staged:

```text
TE-1:
  allocate/init/reset Environment
  ordinary occupancy commit → identity hygiene → Environment reconcile → joint settle
  phase/Smoke claim → receiver claim → receiver-gated Matter commit
    → identity hygiene → Environment reconcile → joint settle
  no Air flow, no Air thermal exchange, no pressure coupling

TE-2 (implemented):
  Air outflow scale → mass/advected-energy commit → settle
  thermal stability scale → unified thermal commit → settle
  activity/wake integration

TE-3:
  Water/Steam progress and energy-like phase accounting

TE-4:
  ignition exposure/dose and chemical source accounting

TE-5:
  background pressure, Vacuum differential, structure coupling
```

Separate reconcile passes preserve the eight-storage-buffer limit. Existing ownership claims may be wrapped, but their live ranges and encodings remain authoritative until a structural test proves reuse. New passes receive explicit profiler identities and groups.

The TE-1 Matter flag ownership map is exact: bits 0–1 and 4–15 belong only to
Oil/Wood combustion; bits 16–27 belong only to Smoke decay; bits 2–3 and 28–31
are reserved and zero until assigned. Movement carries the source word and
then sanitizes it for the resulting Material. Matter→Matter self-transition
clears every bit not owned by the target. New spawns and every Matter→EMPTY
transition write zero. Future phase and ignition progress receives dedicated
state instead of colliding with these bits. A separate material/flags hygiene
pass performs this before joint settle without inflating Environment reconcile.

## 7. Pressure vocabulary

```text
Atmospheric pressure:
  derived from Environment Air mass and energy

Mechanical/gauge overpressure:
  existing pressure[] source and propagation

Vacuum pressure:
  zero

Structure face differential:
  effective pressure on side A minus side B
```

Before TE-5, these are not coupled into new structure forces. At TE-5, standard Atmosphere and ordinary Liquid/Gas with zero mechanical overpressure must share the same reference, so no false force exists. Air pressure and `pressure[]` are never summed independently of occupancy.

## 8. Temperature migration inventory

The following migrated atomically at TE-2 source `fb7e568...` from the former arbitrary scalar to the Celsius-like gameplay scale:

- `TEMPERATURE_REFERENCE` and thermal safety limits;
- Water/Ice/Steam thresholds and hysteresis;
- direct Ice/Steam placement values;
- Heat/Cool authoring deltas;
- Oil/Wood ignition and sustain thresholds;
- combustion heat per tick and cap;
- authored fixture temperatures;
- reset/staging defaults;
- Inspector labels/copy and UI `°C` usage;
- CPU reference rules, WGSL constants, descriptors, tests and evidence expectations.

The migration is one runtime commit with its production fixtures and tests;
historical TE-0/TE-1 evidence retains its original vocabulary.

## 9. Later phase and ignition contracts

TE-3 must account for sensible, phase-offset, and pending latent energy-like state before changing Water/Steam behavior or yield. Exact coefficients and representation remain open until named fixtures pass.

TE-4 replaces one-tick threshold ignition with bounded exposure/dose. A brief threshold spike must decay without ignition; sustained surface exposure may ignite. Combustion heat is an explicit source, so burning Matter may exceed the original source temperature. Oxygen, Ash and final FX remain excluded.

Vacuum combustion support is a user-owned decision for TE-4/TE-5 and must not be inferred from Air amount before it is closed.

## 10. Activity, Inspector and performance

Equilibrium Atmosphere and Vacuum bulk sleep. The TE-2 Environment activity
stage constructs a bilateral runnable face cohort: both
endpoint chunks and the required halo execute a face, or neither does. One
canonical face owner computes flux and both endpoint self-writes consume the
same Current snapshot, so a runnable donor cannot export into an unexecuted
sleeping receiver. Nonzero boundary-reservoir flux persistently wakes only its
edge chunk and halo until equilibrium. Sleep-on/off semantic equivalence,
chunk-seam activation and reservoir-source activation are hard fixtures. Air
work must not make the full world permanently runnable.

TE-2's correctness runtime boundary is sealed/no-flux. Its open-boundary
semantic fixture uses an explicit fixed standard-Atmosphere ghost reservoir
and reports that exchange as an external source/sink. The product's default
world-edge mode remains a user decision before TE-5 integration; TE-2's
reference contract is not ambiguous.

The current Cell Inspector remains a 24-byte, at-most-10-Hz Matter diagnostic.
The TE-2 candidate uses a separate bounded diagnostic sample every 8 ticks and
does not extend the product Inspector payload.

The TE-2 correctness source reaches `268,462,384` tracked bytes without
profiling and `268,463,472` with profiling at 2048². Its 2048² GPU tick P95 is
`2.599712 ms` for equilibrium and `2.304832 ms` for a local frontier; the
equilibrium terminal has zero active Cells/chunks. Full CFD, velocity fields,
coarsening, packing, f16 and optimization remain outside the program.

## 11. Open decisions

These remain explicitly open after TE-2 and do not change its candidate status:

- default world edge reservoir mode, required before TE-5 product integration;
- Vacuum combustion support, required before TE-4/TE-5 closure;
- phase latent coefficients, yield and progress representation, required before TE-3;
- GAS Matter Environment permeability, opened only if TE-F33 demonstrates a product blocker;
- any post-baseline Air-flow cadence/coarsening/packing optimization; the
  implemented correctness baseline is full-resolution and every tick.

## 12. Exclusions

No Oxygen, Ash, new Matter, same-cell gas mixture, humidity, radiation, CFD, final FX, Save/Load, Rewind, G9-B/C/D/E implementation, G7-C optimization, G8 rerun, or G8-C recapture is authorized by this specification.
