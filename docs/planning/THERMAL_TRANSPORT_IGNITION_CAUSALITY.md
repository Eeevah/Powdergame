# Thermal Transport & Ignition Causality

Status: **TE-2 REVISED PASSIVE THERMAL ENVIRONMENT CANDIDATE / USER RE-REVIEW PENDING**; **TE-3 DESIGN REQUIRED / NOT STARTED**

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
- Combustible Matter ignites as soon as its own current temperature reaches its Material ignition threshold. Oil and Wood share this generic threshold grammar and currently have no exposure-time or accumulated-dose requirement.
- There is no Oxygen requirement. TE-2 Air flow/heat exchange does not silently add combustion support or ignition dose.

TE-3 and later gates must not quietly change the remaining phase/ignition
statements without their named authorization and fixtures.

Direct TE-2 review classified the original candidate **USER REVIEWED /
REVISION REQUIRED** because F, N, I and the thermal/Air measurements were not
usable enough to evaluate the four scenes. Source
`097728128343cf89383920c968a010b3dcf8e8c0` remediates only candidate controls,
bounded diagnostics and staging; production physics and coefficients remain
the D-015 runtime. Direct Sandbox review separately registered the Water/Steam
checkerboard clumping and closed-cycle quantity defect as TE-3 design input,
not a TE-2 retuning request.

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

- Thermal Environment is **TE-2 REVISED PASSIVE THERMAL ENVIRONMENT CANDIDATE / USER RE-REVIEW PENDING** at candidate source `0977281...`; the production-physics source remains `fb7e568...`;
- TE-3 is **DESIGN REQUIRED / NOT STARTED**; its phase-accounting blocker is registered, but no representation or runtime change is selected;
- G9-B emergence validation remains blocked on this prerequisite;
- G9-A Inspector continuity is **USER ACCEPTED** and G9-A overall is **USER ACCEPTED WITH KNOWN FOLLOW-UP**; this does not advance G9-B.
