# Thermal Environment Implementation Gates

- **Status:** TE-2 and pressure-decoupled TE-3 accepted with known follow-up; TE-5B/C/D/X and packet designs blocked; Pressure redesign deferred
- **Architecture:** D-013 through D-022 / ADR-0005 / accepted ADR-0006 / proposed blocked ADR-0007 through ADR-0010
- **Rule:** no task may silently include the physics of a later gate

## TE-0R — Reuse and prior-art survey

Deliver internal reuse inventory, exact external source/license/version decisions and copied-code count. External code is not imported. **Completion:** `REUSE SURVEY COMPLETE`.

## TE-0 — Architecture lock

Deliver ADR-0005, canonical spec, production pass/binding/state inventory, memory budget, occupancy-path inventory, validation contract, supersession links and implementation gates. Runtime source stays unchanged. **Completion:** `DESIGN LOCK COMPLETE`.

## TE-0A — Independent adversarial review

Attack ontology, occupancy hygiene, conservation, duplicate transport, pressure double count, wall sealing, sleep/wake, progress ownership, Inspector honesty, license ingress and gate leakage. Critical/High findings must be resolved in the design or TE-1 remains blocked. **Completion:** `ADVERSARIAL REVIEW COMPLETE / CRITICAL-HIGH BLOCKER 0`.

## TE-0B — Reference-math proof

Run the approved fixed-seed candidate proof once outside the repository. Record trials, error bounds and limitations. Do not copy it into production. **Completion:** `REFERENCE FORMULA PROOF PASS`.

## TE-1 — Environment state and occupancy foundation

Allowed scope:

- four full-resolution Environment Current/Next buffers;
- one full-resolution `u32` Environment receiver-claim scratch, and no second
  new full-world scratch without a new measured decision;
- initialization, reset, staging and exact allocation report;
- Atmosphere/Vacuum semantic helpers;
- separate Environment reconcile after every occupancy-changing stage;
- deterministic local spawn receiver claim and blocked-spawn behavior;
- bounded whole-parcel headroom and paired Matter/Environment commit;
- mandatory seven-storage Environment-blocked phase-pressure consequence
  between the existing expansion-pressure pass and pressure settle;
- exact Matter flag ownership and identity hygiene before joint settle;
- joint settle and exact hygiene;
- bounded test-only readback.

Explicitly disabled:

- inter-cell Air flow;
- Matter↔Air or Air↔Air thermal exchange;
- phase/combustion retune;
- Air-pressure/structure coupling;
- Vacuum tool/product UI;
- G9-B validation.

Completed TE-1 checklist at source `1a722d239a16bade5772688fa822465d5cef4602`:

- [x] start from the exact clean/pushed TE-0 docs source;
- [x] preserve the eight-storage-buffer ceiling with separate passes;
- [x] declare every new writer and settle boundary;
- [x] use one canonical staging/reset Environment image API;
- [x] include movement, density swap, Void, phase self transition, phase spawn, Smoke spawn, rupture, decay, fuel consumption, Draw, Erase, preset/reset, direct test write, scenario and benchmark staging;
- [x] prove receiver arbitration and the no-receiver blocked outcome;
- [x] pin receiver scratch encoding/live range, same-stage Matter-target
  exclusion, whole-parcel headroom and paired rollback;
- [x] pin exactly-once Environment-blocked expansion-pressure accounting and
  its seven-storage layout/order before pressure settle;
- [x] pin coefficient domains and exact-zero Vacuum with no residual deletion;
- [x] pin Matter flags ownership and separate hygiene-pass bindings;
- [x] extend profiler identities and allocation report;
- [x] extend Naga parse/write-contract and reset tests;
- [x] leave the 24-byte/10-Hz Inspector contract unchanged;
- [x] run `validation-plan` and complete the required final-source FULL;
- [x] do not start TE-2.

Stop: `ENVIRONMENT STATE/OCCUPANCY HYGIENE IMPLEMENTED / AIR TRANSPORT NOT STARTED`.

## TE-2 — Passive Air transport and unified thermal exchange

Add pressure-derived mass flow, donor-energy advection, Air conduction exactly
once, Matter↔Air surface exchange, bilateral face cohorts, activity/wake and
source-free stability. The correctness runtime edge is sealed/no-flux; an
explicit fixed standard-Atmosphere ghost reservoir is fixture-only and reports
external exchange. Disable phase changes, combustion changes and new pressure
coupling in semantic fixtures.

Stop after user-observable Air-gap/Vacuum/open/sealed candidate: `PASSIVE THERMAL ENVIRONMENT CANDIDATE / TE-3 NOT STARTED`.

Completed at source `fb7e568e21012b6067269f4e1b82c36c865023d0`:

- [x] full-resolution, every-tick pressure-derived Air flow with bounded donor outflow;
- [x] donor-specific-energy advection and separate unified passive conduction;
- [x] Matter↔Matter, Air↔Air and Matter↔Air exchange from one Current snapshot;
- [x] bilateral activity/wake and equilibrium bulk sleep;
- [x] sealed production edge plus explicit accounted fixed-reservoir fixture;
- [x] Celsius-like gameplay migration in the same runtime source;
- [x] deadband as a shared work/no-work gate, not a subtractive flux;
- [x] 30-case CPU/GPU `SMALL_DELTA_THERMAL_CONVERGENCE` regression;
- [x] named transport, source-free, reset, pass/binding and profiler guards;
- [x] final-source FULL, locked release build, one bounded candidate launch and one performance measurement;
- [x] no Air-pressure force, TE-3, G9-B/C/D/E or optimization.

The first direct review classified TE-2 **USER REVIEWED / REVISION REQUIRED**
because the candidate did not expose its existing controls or measurements
well enough to evaluate those claims. Candidate-only remediation source
`097728128343cf89383920c968a010b3dcf8e8c0` fixes F/N/I, makes bounded samples
persistent and honest, enlarges scene 1 staging, and exposes scene 2-4
accounting without changing production physics or TE-2 coefficients. The user
subsequently confirmed F/N/I, all four scene contracts and reset/controls, and
recorded TE-2 **USER ACCEPTED WITH KNOWN FOLLOW-UP**. Preserve
`LONG_HORIZON_SEALED_AIR_DRIFT_BUDGET` and
`TE2_CANDIDATE_HUD_LABEL_POLISH` as non-blocking later work; no additional
same-tick scene 3/4 comparison is required.

## TE-3 — Water/Steam thermal cycle

Close phase progress representation, latent coefficients, yield, reversal accounting, surface boiling, Steam cooling, free/surface condensation and hysteresis. Do not change ignition, Oxygen or final FX.

The direct Sandbox observation and current 1 Water -> up to 2 Steam -> up to 2
Water round-trip audit are registered in
[`TE3_WATER_STEAM_PHASE_ACCOUNTING.md`](TE3_WATER_STEAM_PHASE_ACCOUNTING.md).
Closed-cycle quantity, expansion/contraction, latent reversal, surface boiling,
cold-surface condensation, nucleation, and the mid-air phase traffic jam are
design blockers. D-018 accepts Hybrid A+C with locked amendments after one new
pure reference proof and fresh independent v2 review. Those conditions passed;
this records architecture acceptance only and does not authorize runtime work.

### TE-3D — docs-only phase-enthalpy design lock

- [x] retain exactly one Water-equivalent quantity per Water/Steam Matter Cell;
- [x] compare one-Cell, owned-fragment, dedicated-state and existing yield-2 options;
- [x] propose Hybrid A+C with `yield = 1` and two Current/Next phase-energy buffers;
- [x] define sensible/latent enthalpy, partial progress, reversal and canonical ranges;
- [x] define cold-surface condensation and bounded deterministic free-air nucleation;
- [x] inventory all future writers, movement, reset, editor, sleep/activity and hygiene paths;
- [x] project 40 passes, 80 queries, 1,280 profiler bytes and exact tracked allocations;
- [x] define deterministic future fixtures and pass one pure reference-math proof;
- [x] complete an independent adversarial review with no unresolved Critical/High finding;
- [x] require a real positive-conductance TE-2 energy-removal sink and reject K=0 Boundary;
- [x] lock buried initiated Water, value-derived ready Water and explicit completion permission;
- [x] lock radius-2 seed/veto plus the predeclared 30-tick initiation bound;
- [x] restrict generic yield-greater-than-one phase targets and record internal mixer provenance;
- [x] preserve TE-1/TE-2 evidence boundaries and copy zero external implementation formulas;
- [x] keep pressure-volume force, ignition and all runtime work outside this gate.
- [x] preserve the historical atomic constraint until a direct user decision supersedes it; D-024 now supersedes that constraint without rebinding G5 evidence.

Accepted design authorities:

- [`ADR-0006`](../architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md)
- [`PHASE_THERMODYNAMICS_SPEC`](../specs/PHASE_THERMODYNAMICS_SPEC.md)
- [`PHASE_THERMODYNAMICS_VALIDATION`](../development/PHASE_THERMODYNAMICS_VALIDATION.md)
- [`TE3_PHASE_ENTHALPY_DESIGN`](../adversarial-reviews/TE3_PHASE_ENTHALPY_DESIGN.md)

TE-3D remains `ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS`. D-024 activates
ADR-0006 as a standalone pressure-decoupled candidate at
`41467219819c5d0cb3eab8ae22b652449da20480`: 1:1 family identity, no second
Steam, family `NO_PROPOSAL`, zero Water blocked-expansion pressure, 40 passes,
80 queries and 32 MiB phase state at 2048 squared. The source-bound receipt is
[`THERMAL_ENVIRONMENT_TE_3_PHASE_CYCLE_2026-08-21`](../evidence/THERMAL_ENVIRONMENT_TE_3_PHASE_CYCLE_2026-08-21.md).
Direct user review is pending. Full TE-5 pressure redesign is deferred and not
started.

### TE-5B — docs-only phase-volume bridge design

This narrow prerequisite supplies ADR-0006's explicit completion transaction;
it is not the full TE-5 background-pressure/structure-force gate.

- [x] compare unconditional pressure, non-exclusive EMPTY, new volume state and exclusive-token options;
- [x] evaluate the exclusive local volume-relief token as the named primary candidate;
- [x] classify an eligible endpoint attempt before settle and return targeted, blocked or edge-deferred acceptance;
- [x] replay current GAS First-Match stops so Void defers and an earlier legal density swap cannot be skipped to a lateral EMPTY;
- [x] define one shared Matter-expansion/relief claim domain under the 30-bit Cell-index bound;
- [x] define winner zero-pressure and blocked/loser `100.0` exactly-once consequences;
- [x] isolate relief from Environment receiver, spawn, displacement and Environment-blocked pressure;
- [x] preserve the 40-pass/80-query projection with no new persistent/full-world state;
- [x] define TE5B-F01 through F12, including the atomic G5 and open-control fixtures;
- [x] pass the predeclared fixed-seed pure arbitration/accounting proof exactly once;
- [ ] close the finite-capacity High: a non-mutating 1:1 token moves/reuses an EMPTY vacancy instead of consuming headspace;
- [ ] obtain user revision of the blocked architecture.

Candidate authorities:

- [`ADR-0007`](../architecture/decisions/ADR-0007-phase-volume-pressure-bridge.md)
- [`PHASE_VOLUME_PRESSURE_BRIDGE_SPEC`](../specs/PHASE_VOLUME_PRESSURE_BRIDGE_SPEC.md)
- [`PHASE_VOLUME_PRESSURE_BRIDGE_VALIDATION`](../development/PHASE_VOLUME_PRESSURE_BRIDGE_VALIDATION.md)
- [`TE5_PHASE_VOLUME_PRESSURE_BRIDGE`](TE5_PHASE_VOLUME_PRESSURE_BRIDGE.md)

The successful docs-only stop was not reached. Independent review found that a
sealed one-Cell-wide, stagger-heated Water column can pass one EMPTY vacancy
downward through ordinary 1:1 Steam movement, so finite headspace never has to
become blocked and F05/F11 cannot be guaranteed. Current stop: `TE-5B DESIGN BLOCKED / ADR-0007
PROPOSED / USER ARCHITECTURE REVISION REQUIRED`. A future replacement requires
user disposition and separate authorization before any runtime task; verified
TE-3 and the eventual bridge must still activate together with new G5 evidence.

### TE-5C — docs-only local Vapor capacity-pressure replacement

D-020 rejects the TE-5B token and authorizes the final attempt without new
persistent phase-volume state.

- [x] derive continuous demand from accepted phase energy;
- [x] define radius-1 per-EMPTY proportional capacity and linear target `0..100`;
- [x] define orthogonal EMPTY gauge-zero venting at rate `0.20`;
- [x] audit proposal reuse after Smoke and project 41 passes / 82 queries;
- [x] predeclare F01–F13 plus the asymmetric reachable-capacity control;
- [x] run the fixed-seed 50,000-static / 10,000-multi-tick proof once;
- [ ] satisfy VC-INV-008: sufficient reachable local capacity must not create false pressure;
- [ ] close independent-review Critical/High blockers.

The result reported the vacancy-walk, bounds, quantity, partial, generic,
pressure/vent and atomic pure-model checks as passed but failed the predeclared
asymmetric control. Independent review found several reported checks did not
execute their named obligations. One Steam adjacent to both EMPTYs absorbed
`1.5` gross share then
capped at one; another Steam adjacent only to the shared EMPTY retained
capacity `0.5` and false target `100`, although a complete assignment existed.

Fresh review left Critical `0` / High `6`, adding internal-EMPTY capacity/vent
conflation, irreversible phase-pressure provenance, unreachable downward
capacity, activity/snapshot/binding infeasibility and receipt overclaim.

Current stop: `TE-5C DESIGN BLOCKED / ADR-0008 PROPOSED / RUNTIME NOT
STARTED`. Per D-020, the next design decision must explicitly permit persistent
phase-volume state. No formula substitution or another stateless token/impulse
attempt is authorized.

### TE-5D — docs-only persistent Vapor extent replacement

D-021 permits one reciprocal extent-link plus dedicated phase-pressure
Current/Next pair.

- [x] define exact link encoding, reciprocal invariants and target zero-Air ownership;
- [x] define whole-parcel receiver acquisition and byte-identical failure;
- [x] define owner movement, other-EMPTY relocation, density swap, condensation and Void release;
- [x] freeze depth-six matching, six settle ticks and the five-position GAS target domain;
- [x] freeze phase-pressure relaxation `0.10`, diffusion `0.025` and equilibrium `100`;
- [x] audit the 64 MiB 2048² state delta and fixed 62-pass / 124-query projection;
- [x] run one 50,000-graph / 10,000-grid proof process;
- [ ] satisfy the all-labeled 6×6 proof obligation;
- [ ] satisfy PVX-INV-011 for arbitrary canonical persistent matchings;
- [x] complete fresh independent review;
- [ ] close independent-review High blockers (`6` open).

The frozen candidate fails on an eight-source alternating chain whose complete
matching requires an augmenting path deeper than six. Retrying atomically does
not change the links, so the unmatched source can cross Wood threshold despite
available capacity. Current stop: `TE-5D DESIGN BLOCKED / ADR-0009 PROPOSED /
RUNTIME NOT STARTED`. Required repair is wider matching scope; an efficient
fixed GPU graph may also require user authorization for a full-world search
scratch. Fresh review ended at Critical `0` / High `6` / Medium `2`, SHA-256
`73adaf56bea1589d425d89ba9430a7f50f3d0b9cf50f5b8fdc2155f263968ed6`.

### TE-5X — docs-only three-model architecture reset

D-022 compares exact persistent matching, connected shared-chamber capacity
and a conservative Vapor-volume Environment scalar under one fixture matrix.

- [x] preserve TE-5B/C/D blocked history and forbid another fixed depth;
- [x] record primary-source algorithm/library identities and copied-code zero;
- [x] freeze A/B/C formulas, costs, fixtures, seed and selection criteria;
- [x] start exactly one combined reference process;
- [ ] complete any candidate evaluation (`0` completed);
- [ ] complete 50,000 generated states and 10,000 grids (`0` completed);
- [ ] obtain evidence-supported eligibility/ranking;
- [x] complete fresh comparative review;
- [ ] close fresh comparative-review High blockers (`11` open).

The only process exited at the NetworkX 3.6.1 version guard because the
temporary path resolved a namespace module without `__version__`. The task's
one-shot rule forbids repair/rerun. Current stop: `TE-5X DESIGN BLOCKED /
ADR-0010 PROPOSED / COMPARISON EVIDENCE INCOMPLETE / CRITICAL 0 / HIGH 11 /
RUNTIME NOT STARTED`. Review SHA-256:
`c424c8336d3b34784f6a3ffbb37421ceca8888608c198da45793774b49ffb579`.

### TE-3Q / TE-5Q — conservative phase packets

D-023 preserves all blocked TE-5 candidates and supersedes only the whole-Cell
quantity constraint.

- [x] freeze explicit units, quantity-scaled H, orthogonal merge and spatial pressure law;
- [x] syntax/import/fixture-list check the standard-library-only proof before freeze;
- [x] execute `TE3Q-PHASE-PACKETS-REFERENCE-V1` exactly once;
- [x] complete 100,000 algebra trials, 10,000 grids and deterministic replay;
- [x] record mathematical PASS and GPU/visual/product unknown boundaries;
- [x] complete fresh-context independent review;
- [ ] close every independent-review High finding (`8` open);
- [ ] obtain user architecture acceptance of ADR-0011;
- [ ] authorize and produce source-bound runtime/G5 evidence.

Frozen script/result SHA-256 are
`c938c6e3ce7074abc6d5144c708f85a17be349bb84f962238e568c17d55ed03c` /
`a0181d4ca0ed63eb92cac5cd04098ff438546903c8dc6853e8b0b5d5ab208ed7`.
Current stop is **TE-3Q / TE-5Q DESIGN BLOCKED / ADR-0011 PROPOSED / CRITICAL
0 / HIGH 8 / RUNTIME NOT STARTED**. The mathematical receipt remains narrow;
the old G5 evidence is not rebound to this future source.

## TE-4 — Ignition kinetics

Add bounded exposure/dose, decay, surface-first Oil/Wood ignition, explicit flame bonus and chemical heat accounting. Oxygen, Ash, new Matter and final FX remain excluded. D-029 selects non-Vacuum orthogonal EMPTY Air-face access without Air consumption or Oxygen semantics.

D-028 produced ADR-0012 and a frozen v1 reference identity, but the only process
stopped before executing any sequence/grid/fixture because the coefficient
sweep's equal-metric tie selected a different Oil tuple than the preregistered
tuple. V1 stop remains `TE-4D DESIGN BLOCKED / ADR-0012 PROPOSED / RUNTIME NOT
STARTED`. Its fresh review ended at Critical `0` /
unresolved High `2` (zero completion and named-fixture aggregation/execution).
V1 remains immutable. D-029's separate v2 identity fixes exact coefficients,
packed u6, Air policy, chemical Q and final-tick order. Its manifest-bound
reference completed `1/1`: 13 reference fixtures PASS and exactly four
production fixtures remain `NOT_ESTABLISHED`. Runtime implementation still
requires a new architecture decision because fresh review found unresolved
High `3`: non-transactional path counters, sole-Air-face loss through same-tick
Smoke, and an unfrozen F08 frontier oracle. TE-4D v2 is DESIGN BLOCKED.

## TE-5 — Pressure and Vacuum coupling

After user decisions on edge reservoir and Vacuum combustion, integrate derived background pressure, Atmosphere refill/Vacuum vent, heated sealed Air, face differential and existing gauge overpressure without double counting. Revisit blocked spawn displacement only with a new accounted contract. On the same source, validate the replacement G5 boil/confinement/rupture/vent chain and atomically activate the verified TE-3 Water yield-1 path. Neither half may ship alone.

Stop: `WATER/STEAM + PRESSURE USER-TESTABLE CANDIDATE / TE-6 NOT STARTED`.

## TE-6 — Product integration and G9-B readiness

Integrate approved Environment semantics with Starter Lab/Blank World, editor hygiene and honest Inspector presentation. Measure 256² product and 2048² reference performance on the new correctness source. Do not begin G9-C, M0 closure, main promotion or speculative optimization.

## Current stop

```text
TE-0R  COMPLETE
TE-0   COMPLETE
TE-0A  COMPLETE / seven High findings resolved in the design / blocker 0
TE-0B  PASS_REFERENCE_MATH_ONLY
TE-1   ENVIRONMENT STATE / OCCUPANCY HYGIENE IMPLEMENTED
TE-2   USER ACCEPTED WITH KNOWN FOLLOW-UP
Air transport / unified passive thermal exchange   IMPLEMENTED
TE-3D  ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS
ADR-0006  ACCEPTED / IMPLEMENTED AS PRESSURE-DECOUPLED CANDIDATE
TE-3 runtime  USER ACCEPTED WITH KNOWN FOLLOW-UP
TE-5B phase-volume bridge design  DESIGN BLOCKED / FINITE-CAPACITY HIGH OPEN
ADR-0007  PROPOSED / USER ARCHITECTURE REVISION REQUIRED
TE-5B runtime  NOT STARTED
TE-5C local capacity-pressure design  DESIGN BLOCKED / CRITICAL 0 / HIGH 6
ADR-0008  PROPOSED / ARCHITECTURE REVISION REQUIRED
TE-5C runtime  NOT STARTED
TE-5D persistent extent design  DESIGN BLOCKED / CRITICAL 0 / HIGH 6
ADR-0009  PROPOSED / ARCHITECTURE REVISION REQUIRED
TE-5D runtime  NOT STARTED
TE-5X architecture comparison  DESIGN BLOCKED / CRITICAL 0 / HIGH 11
ADR-0010  PROPOSED / NO MODEL RECOMMENDED
TE-5X runtime  NOT STARTED
TE-3Q / TE-5Q packet design  DESIGN BLOCKED / CRITICAL 0 / HIGH 8
ADR-0011  REJECTED / DESIGN BLOCKED HISTORY
TE-3Q / TE-5Q runtime  NOT STARTED
Full TE-5 Pressure redesign  DEFERRED / NOT STARTED
Air-pressure force  NOT STARTED
TE-4D v1/v2  DESIGN BLOCKED / IMMUTABLE
TE-4D v3  DESIGN BLOCKED / CRITICAL 0 / HIGH 3 / IMMUTABLE
TE-4D D-031 supplement  BLOCKED / CRITICAL 0 / HIGH 3 / ADR-0012 PROPOSED
TE-4 runtime  NOT STARTED
G9-B/C/D/E  NOT STARTED
```

### D-030 v3 transaction/oracle closure

The distinct v3 identity completed once with 13 reference PASS, four expected
production `NOT_ESTABLISHED`, zero failures and zero audited required paths.
F07/F08 matched the frozen independent complete event oracle and F15B
established the two-stage sole-Air self-Smoke reference transaction. The live
source audit supports 42 passes, 84 queries, 1,344 profiler bytes, at most
eight storage bindings and no new persistent or scratch state. Runtime gates
remain unchecked; ADR-0012 remains Proposed and user review is required.

Fresh review blocks v3 despite the completed process receipt. The three High
findings are a hardcoded F15B next snapshot, an auditor that trusts semantic
labels from the system under test, and F09 arithmetic not driven by a complete
fuel/heat lifecycle. No runtime gate advances.

### D-031 targeted transaction supplement

The distinct supplement completed `1/1` and preserved all v1/v2/v3 artifacts.
Its reduced lifecycle and snapshot receipts are valid only in their stated
scope. Fresh review found Critical `0` / High `3` / Medium `2`: F15B has no
Matter/Air settle before the next topology decision, transaction class remains
caller-selected, and Air receiver topology/claim is absent. It therefore does
not compose with v3 into architecture completion. ADR-0012 stays Proposed and
no runtime gate advances.
