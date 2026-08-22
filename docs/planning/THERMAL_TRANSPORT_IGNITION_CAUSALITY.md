# Thermal Transport & Ignition Causality

Status: **TE-2 USER ACCEPTED WITH KNOWN FOLLOW-UP**; **TE-3 USER ACCEPTED WITH KNOWN FOLLOW-UP**; **TE-4D v1/v2/v3/D-031 BLOCKED / IMMUTABLE**; **TE-4I IMPLEMENTATION CANDIDATE / AUTOMATED VALIDATION PASS / USER REVIEW PENDING / ADR-0012 PROPOSED**; **TE-5 PRESSURE REDESIGN DEFERRED / NOT STARTED**

Gate relationship: **G9-B emergence-validation prerequisite**. This document registers a bounded design project; it does not authorize implementation, retune existing physics, or reopen G8 evidence.

## 1. Direct observation and problem statement

Direct G9-A re-review produced two linked observations:

- heat does not travel through EMPTY space;
- hot Stone can bring adjacent Oil or Wood to its ignition threshold so quickly that ignition appears immediate.

These observations are not, by themselves, a production defect verdict. They
expose a product-causality question. D-013 and ADR-0005 now close the Environment
architecture; the later ignition exposure representation and coefficients
remain gate-owned work rather than an open Air ontology choice.

## 2. Historical TE-1 baseline and current TE-2 boundary

The following list records the TE-1 baseline that motivated TE-2. At source
`fb7e568...`, Air flow and unified passive thermal exchange supersede only the
open-space/direct-contact statements; threshold ignition still remains the
current baseline until TE-4:

- `EMPTY` is absence of foreground Matter. Its separate Environment may be Atmosphere, low pressure, or Vacuum. At TE-2, non-Vacuum Air is a passive thermal transport medium.
- A Matter-temperature value stored at an EMPTY index is not physical Air temperature; EMPTY self Matter temperature resolves to the reference state.
- TE-1 thermal transfer was direct-contact only. TE-2 adds four-face Air flow and unified Matter/Air passive exchange without adding diagonal or line-of-sight transport.
- Each participating Matter uses its conductivity and heat-capacity gameplay scalars; no diagonal, distance or line-of-sight transport is present.
- The pre-TE-4I baseline ignited combustible Matter as soon as its own current temperature reached its Material threshold. D-032 production source `8d9e8cb...` supersedes that behavior with bounded exposure/dose.
- Production has no Oxygen quantity. TE-4I adds only a non-Vacuum orthogonal EMPTY Air-face gate: positive Air mass qualifies but is not consumed or rate-scaled.

TE-3 and later gates must not quietly change the remaining phase/ignition
statements without their named authorization and fixtures.

D-029 names that authorization for a design candidate only. Oil/Wood exposure
uses own authoritative threshold temperature, integrated bucketed dose,
cooling decay and previous-snapshot orthogonal flame events. Air access is a
surface predicate rather than a transported Oxygen quantity. Loss of access
extinguishes before same-tick heat/flame/Smoke, while fuel remains owned by the
Matter. The reference proves this reduced state transaction; TE-2 transport,
WGSL order and product behavior remain production-deferred.

V2 review exposed an unresolved causal edge: Smoke can occupy the sole Air
face later in the same tick after that face authorized emission. A later user
decision must choose start-snapshot semantics, last-face protection or atomic
post-commit cancellation before the projection can be reconsidered.

The first direct TE-2 review classified the original candidate **USER REVIEWED /
REVISION REQUIRED** because F, N, I and the thermal/Air measurements were not
usable enough to evaluate the four scenes. Source
`097728128343cf89383920c968a010b3dcf8e8c0` remediates only candidate controls,
bounded diagnostics and staging; production physics and coefficients remain
the D-015 runtime. Direct Sandbox review separately registered the Water/Steam
checkerboard clumping and closed-cycle quantity defect as TE-3 design input,
not a TE-2 retuning request. The subsequent direct re-review confirmed F/N/I,
all four scene contracts and reset/controls, and recorded TE-2 **USER ACCEPTED
WITH KNOWN FOLLOW-UP**. The tiny long-horizon sealed Air drift budget and HUD
label/truncation polish remain non-blocking later work.

## 3. Historical architecture options — superseded by D-013 / ADR-0005

The following Option A/Option B comparison records the state at D-012. It remains as design history and is not the current selection. D-013 and ADR-0005 select separate `air_mass_current/next` plus `air_energy_current/next`; Air temperature and background pressure are derived. Neither option below is the adopted architecture.

### Option A — implicit ambient carried by the existing temperature field

Allow the existing dense temperature field to carry an ambient value through EMPTY while continuing to treat EMPTY as non-Matter for identity, density, pressure, movement and reactions. A future rule would need to define transport, dissipation, boundary behavior, wake/sleep participation, edit/reset hygiene and the exact point where ambient heat couples into adjacent Matter.

This is the smaller state-layout option but explicitly revises the current “EMPTY is not a hidden thermal medium” contract. It cannot be adopted by implementation accident; specs and semantic tests must make the limited ambient exception explicit.

### Option B — separate ambient temperature field

Keep Matter temperature strictly attached to non-EMPTY Matter and add a distinct ambient-temperature field for open-space transport. A future rule would need to define field allocation, Current/Next ordering, source coupling, diffusion/dissipation, chunk activity, reset/staging, Inspector visibility and memory/performance cost.

This keeps the EMPTY/Matter contract conceptually clean but adds world state, bandwidth and pass cost. It therefore requires measurement before selection.

Neither historical option is selected. The canonical design is linked below.

## 3.1 Canonical design program

- [`ADR-0005`](../architecture/decisions/ADR-0005-atmosphere-vacuum-environment.md)
- [`THERMAL_ENVIRONMENT_SPEC`](../specs/THERMAL_ENVIRONMENT_SPEC.md)
- [`Production Inventory`](../architecture/THERMAL_ENVIRONMENT_PRODUCTION_INVENTORY.md)
- [`Reuse Survey`](../research/2026-08-20-thermal-environment-reuse-survey.md)
- [`Validation Contract`](../development/THERMAL_ENVIRONMENT_VALIDATION.md)
- [`Implementation Gates`](THERMAL_ENVIRONMENT_IMPLEMENTATION_GATES.md)
- [`TE-3 Water / Steam Phase Accounting`](TE3_WATER_STEAM_PHASE_ACCOUNTING.md)
- [`Independent Adversarial Review`](../adversarial-reviews/THERMAL_ENVIRONMENT_TE_0.md)

TE-0 is complete and remains docs-only. Independent review found seven High
design defects; all seven are resolved in the canonical contracts and the
review records Critical/High blocker zero. The reference formula proof passed
within its declared limited domain. TE-1 subsequently implemented only
Environment state/occupancy hygiene at source `1a722d...`; the runtime still
has no Air transport, Air thermal exchange or Air-pressure coupling.

## 4. Ignition exposure or dose requirement

D-028 and [`ADR-0012`](../architecture/decisions/ADR-0012-ignition-exposure-dose.md)
select integrated excess-temperature dose, cooling decay and a bounded
previous-snapshot flame bonus as the design candidate. The frozen one-shot
reference attempt completed zero trials because equal-metric Oil coefficient
candidates disagreed with the preregistered selection identity. Therefore the
candidate is **DESIGN BLOCKED**, not accepted, and TE-4 runtime remains not
started. The detailed fixture/evidence boundary is in
[`IGNITION_KINETICS_VALIDATION`](../development/IGNITION_KINETICS_VALIDATION.md).

D-030 fixes the v3 precondition lifetime as the settled
`COMBUSTION_STAGE_SNAPSHOT`. Same-stage Smoke cannot retroactively revoke an
authorized burn; the following snapshot must extinguish before emission if
Air access is gone. The one v3 reduced reference execution passed mutation-
audited transaction paths and exact independent F07/F08 frontiers. This does
not start or establish TE-4 runtime.

Fresh review rejected the v3 transaction-closed interpretation with three
unresolved High findings: asserted F15B next-snapshot Air, SUT-trusted semantic
receipts and non-lifecycle F09 accounting. The frozen receipt remains narrow
history; TE-4 runtime is still not started.

D-032 then ended synthetic-reference repair and authorized actual production
implementation. Source `8d9e8cbe3b6ac651335b5a728ef491abeae4772a`
implements the locked dose, settled binary Air access, finite chemical heat,
and post-Smoke activity semantics. F01..F17 and final-source FULL pass; the
source-bound receipt is
[`THERMAL_ENVIRONMENT_TE_4_IGNITION_KINETICS_2026-08-23`](../evidence/THERMAL_ENVIRONMENT_TE_4_IGNITION_KINETICS_2026-08-23.md).
ADR-0012 remains Proposed and direct user review is pending.

The project must evaluate a causal ignition gate beyond one-frame threshold crossing. Candidate grammar:

```text
combustible Matter above its ignition threshold
+ sustained local exposure or accumulated thermal dose
→ ignition
```

Open design questions include:

- continuous-above-threshold ticks versus integrated excess-temperature dose;
- whether dose decays when cooling and whether partial exposure persists;
- per-Matter thresholds/dose budgets versus one generic grammar;
- how direct flame contact differs, if at all, from a hot inert neighbor;
- how the state is represented without adding unjustified universal per-cell data;
- how the player and Inspector can read “heating” before ignition without exposing a debug table.

The target is legible causality, not real-world combustion simulation.

## 5. Required semantic fixtures before implementation approval

A later implementation proposal must first define deterministic fixtures for at least:

1. hot Stone separated from Wood/Oil by one EMPTY Cell;
2. direct hot-Stone contact with Wood and Oil;
3. short threshold spike that must not ignite under the selected dose rule;
4. sustained exposure that does ignite at a bounded, explainable time;
5. cooling that reduces or clears pending exposure according to the selected rule;
6. ambient transport near world Boundary and through narrow/open cavities;
7. reset/staging exactness for every new field or state;
8. sleep/wake behavior at a slowly moving thermal frontier;
9. no invalid/non-finite temperature or exposure state;
10. CPU reference and production GPU agreement for the chosen local rule.

These are semantic design fixtures, not G8 candidate reruns and not acceptance evidence until their source/harness contract exists.

## 6. Performance and integration questions

Before choosing an option, measure or bound:

- persistent GPU bytes at 256×256 and 2048×2048;
- additional reads/writes, dispatches and timestamped pass cost;
- active-chunk expansion caused by ambient diffusion or exposure decay;
- interaction with thermal deadband, sleeping chunks and long terminal tails;
- whether ambient work can remain sparse/local rather than waking the whole world;
- Current/Next consistency and failure-safe reset/staging;
- Inspector/readback impact without increasing its existing 24-byte, at-most-10-Hz contract;
- Mode C responsiveness and whether any cost threatens the verified 60-TPS product target.

No optimization implementation is authorized by asking these questions.

## 7. Initial exclusions

The initial project explicitly excludes:

- Oxygen or oxidizer simulation;
- Ash or any new Matter;
- final Fire/Smoke visual effects;
- CFD, velocity fields or physically complete radiation/convection;
- broad pressure redesign;
- Save/Load or Rewind;
- G7-C compaction, indirect dispatch, f16/packing or speculative optimization;
- G8 scenario/candidate or G8-C Matrix reruns.

## 8. Stop and approval boundary

The architecture selection, design contract and TE-1 state/occupancy
foundation are complete. Current state:

- Thermal Environment is **TE-2 USER ACCEPTED WITH KNOWN FOLLOW-UP** at candidate source `0977281...`; the production-physics source remains `fb7e568...` and all prior evidence boundaries remain intact;
- D-024/D-027 supersede the old atomic TE-3/TE-5 constraint: pressure-decoupled TE-3 is **USER ACCEPTED WITH KNOWN FOLLOW-UP** and Water/Steam Pressure redesign remains separately deferred/not started;
- G9-B emergence validation remains blocked on this prerequisite;
- G9-A Inspector continuity is **USER ACCEPTED** and G9-A overall is **USER ACCEPTED WITH KNOWN FOLLOW-UP**; this does not advance G9-B.
