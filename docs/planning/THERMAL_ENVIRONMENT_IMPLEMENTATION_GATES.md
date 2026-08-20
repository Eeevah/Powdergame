# Thermal Environment Implementation Gates

- **Status:** TE-2 user accepted with known follow-up; TE-3D design authorized, runtime not started
- **Architecture:** D-013 / D-014 / D-015 / D-016 / D-017 / ADR-0005
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
design blockers. D-017 authorizes docs-only TE-3D research, a proposed Hybrid
A+C architecture, independent adversarial review and one pure reference proof.
It does not authorize runtime work or mark the architecture accepted.

Stop: `WATER/STEAM CYCLE USER-TESTABLE CANDIDATE / TE-4 NOT STARTED`.

## TE-4 — Ignition kinetics

Add bounded exposure/dose, decay, surface-first Oil/Wood ignition, explicit flame bonus and chemical heat accounting. Oxygen, Ash, new Matter and final FX remain excluded. Vacuum combustion support requires a user decision.

Stop: `IGNITION CAUSALITY USER-TESTABLE CANDIDATE / TE-5 NOT STARTED`.

## TE-5 — Pressure and Vacuum coupling

After user decisions on edge reservoir and Vacuum combustion, integrate derived background pressure, Atmosphere refill/Vacuum vent, heated sealed Air, face differential and existing gauge overpressure without double counting. Revisit blocked spawn displacement only with a new accounted contract.

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
TE-3D  DESIGN PROGRAM AUTHORIZED / USER ARCHITECTURE REVIEW TO FOLLOW
Air-pressure force / TE-3 runtime   NOT STARTED
```
