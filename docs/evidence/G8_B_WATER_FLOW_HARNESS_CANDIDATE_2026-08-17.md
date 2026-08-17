# G8-B Scenario 2 — Water Flow Harness Candidate

Date: 2026-08-17
Status: **IMPLEMENTATION CANDIDATE — RUNS / VERDICT / USER ACCEPTANCE PENDING**
Branch: `feature/m0-g8b-scenario-suite`
Required starting SHA: `b884abcfbab8e104bdf34e2e8d19635b157c1638`
Candidate source SHA: **PENDING SOURCE SEAL**

## 1. Scope and frozen baseline

This work prepares the existing shared Water Flow fixture for one automated
Harness candidate and later direct user inspection. The first scratch run and
the candidate must observe the fixture exactly as it existed at the starting
SHA. They must not tune the fixture for appearance or alter production physics
before that evidence is preserved and classified.

The already accepted Sand Fall fixture, its approval record, and its generated
artifacts remain frozen. Fire / Heat, Pressure Burst, Heavy Mixed World, G8-C,
main integration, and PR creation are outside this candidate.

Current execution state:

| Item | State |
|---|---|
| Water Harness implementation | IMPLEMENTED CANDIDATE; unsealed |
| Targeted Rust/Python/Harness tests | FAST PASS recorded |
| Sand Harness regression | FAST PASS recorded; published Sand run unchanged |
| Full workspace checkpoint | PENDING |
| Windows release smoke | PENDING |
| First Water Flow scratch run | PENDING |
| Water Flow candidate run | PENDING |
| Automatic verdict | PENDING |
| User acceptance | PENDING |

No automated or user verdict is recorded in this document.

## 2. Audited finite fixture

`powdergame-scenarios::ScenarioId::WaterFlow` is a finite tick-0 image. It has
no recurring Water source and no scenario-specific production rule. For the
Harness world (`256x256`, chunk size `64`), the authored half-open rectangles
are:

| Role | Rectangle `[x0,x1) × [y0,y1)` | Material |
|---|---|---|
| basin floor | `[10,246) × [230,238)` | Stone |
| left wall | `[10,18) × [90,238)` | Stone |
| right wall | `[238,246) × [90,238)` | Stone |
| left reservoir | `[18,112) × [22,112)` | Water |
| right reservoir | `[144,238) × [34,130)` | Water |
| density pocket | `[164,220) × [72,112)` | Oil, overwriting Water |
| upper shelf | `[72,164) × [154,160)` | Stone |
| lower-left shelf | `[18,74) × [188,194)` | Stone |
| lower-right shelf | `[182,238) × [194,200)` | Stone |
| central channel wall | `[112,124) × [110,202)` | Stone |
| lower divider | `[124,136) × [188,230)` | Stone |

The exact tick-0 material census derived from that fill order is:

| Material | Cells |
|---|---:|
| Water | 15,244 |
| Oil | 2,240 |
| Stone | 6,888 |
| Boundary Block | 1,020 |
| Empty | 40,144 |

All authored Temperature values are the reference temperature, all Pressure
values are the reference pressure, Flags are zero, and `chunk_edit_wake` is
zero. A source-level fixture test pins the full authored material image, these
counts, the reference fields, and the zero state. The fixture builder and all
production physics remain unchanged.

## 3. Observation regions

The fixture description calls for reservoirs draining through staggered
channels into a basin. The Harness may observe that behavior but must not stage
an outcome.

The destination observation mask is defined as cells that are `EMPTY` at tick
0 inside `[18,238) × [200,230)`. The mask has 6,216 cells and contains zero
Water and zero Oil initially. It lies below the final shelves, inside the side
walls, and above the floor. This mask is diagnostic metadata only; it does not
change fixture construction.

The bottom chunk row (`cy=3`, `y=192..255`) contains zero Water at tick 0.
Water observed there later is therefore a fixture-derived cross-chunk flow
signal. The candidate should preserve the observed simulation tick and the
diagnostic sample identity separately.

The Oil pocket is an existing density-displacement stimulus. Oil count and
vertical distribution may be reported as secondary raw observations, but the
Harness must not manufacture a scenario-specific result.

## 4. Shared staging identity

Gallery and headless benchmark construction already share the same scenario
crate and reset path:

- `powdergame_scenarios::ScenarioFixture::build(ScenarioId::WaterFlow, ...)`
- `powdergame_scenarios::reset_and_stage_scenario`
- Windows Gallery slot `2`
- headless benchmark `--scenario water-flow`

The shared reset validates the fixture before mutation, resets the production
`Simulation`, uploads the complete Material, Temperature, Pressure, and Flags
images to both Current and Next buffers, restores edit-wake state, submits the
transfer, and waits for completion. The Experiment worker reuses that path.
Gallery rendering, screenshots, HUD, and readback remain outside official timed
benchmark work.

## 5. Candidate collection contract

The implemented analyzer collects:

- actual Water occupancy movement;
- sampled peak active cells and active chunks;
- first cross-chunk flow observation;
- first destination-mask arrival;
- exact Water count conservation;
- first sleeping chunk;
- final all-sleep or a stable plateau candidate;
- post-settle physical state changes and wake observations;
- invalid Material IDs and non-finite Temperature/Pressure values;
- programmatic reset exact equivalence.

Required semantic frames are tick 0, tick 1, first movement, sampled peak
activity, first cross-chunk flow, first destination arrival, maximum spread,
first sleeping chunk, late settling, terminal state, post-settle confirmation,
and reset. It retains 8–12 representative frames and permits semantic aliases
when two reasons share one sample. Contact Sheet tiles show Active cells,
Runnable chunks, Sleeping chunks, and State hash.

### 5.1 Scenario contract and modes

One entry point dispatches the scenario-specific analyzer without duplicating
the common coordinator:

```bat
run_experiment.bat water-flow --mode scratch
run_experiment.bat water-flow
```

`scratch` is the required first observation mode and places `-scratch-` in the
Run ID. The default is `candidate`, used exactly once after the source seal and
FULL checkpoint. Sand remains candidate-only and preserves its v0 schemas and
published artifacts.

Water uses these schema identities:

- `powdergame-experiment-manifest-v1`
- `powdergame-experiment-telemetry-v1`
- `powdergame-experiment-analysis-v1`
- shared `powdergame-experiment-frames-v0`
- `powdergame-experiment-report-v1`
- `powdergame-experiment-receipt-v1`

Its nine predicates are:

1. `actual_water_movement`
2. `cross_chunk_flow`
3. `destination_arrival`
4. `water_conservation`
5. `no_invalid_materials`
6. `no_nonfinite_fields`
7. `stable_bulk_before_max`
8. `post_settle_stable`
9. `exact_reset`

All nine must be `pass` for automatic `PASS`. A `fail` produces `FAIL`; any
remaining `unknown` produces `NEEDS_HUMAN_REVIEW`. Three diagnostic all-sleep
samples can satisfy stable bulk. Eight identical authoritative-state samples
may select a stable plateau terminal, but a plateau does not silently convert
the finite-fixture all-sleep predicate to pass.

The common Harness remains responsible for the external artifact root
`C:\Users\mdkap\source\Powdergame-artifacts`, clean attached source
provenance, unique Run ID, create-new/no-overwrite files, exact schema/event/
frame validation, hashes, review packet, and receipt-last publication. A worker
or validation failure is preserved without a receipt; its Run ID is never
repaired or reused. `EXPERIMENT_RECEIPT.json` is the final filesystem write and
the only structural completion marker. Generated artifacts must not be
committed to Git.

## 6. Recorded FAST checks and pending checkpoint

| Check | Recorded result |
|---|---:|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| scenario library | 7 passed / 0 failed |
| Windows experiment tests | 16 passed / 0 failed |
| Python coordinator/analyzer tests | 19 passed / 0 failed |
| shared GPU reset integration | 1 passed / 0 failed |

These are implementation checks, not Water evidence or user acceptance. The
one FULL workspace checkpoint, workspace clippy, scoped/full diff check,
Windows Gallery release smoke, scratch run, candidate run, source SHA, artifact
hashes and automatic verdict remain pending. No Water artifact has been
generated or committed.

## 7. Candidate verdict and anomaly classification

The generated report will use `PASS`, `FAIL`, or `NEEDS_HUMAN_REVIEW`. A first
unexpected result must be preserved before any fixture change and mapped, with
its raw observations, to one candidate category:

- `actual_physics_defect`;
- `fixture_representativeness_issue`;
- `expected_local_movement_artifact`;
- `presentation_or_capture_issue`;
- `insufficient_evidence`.

These labels organize follow-up inspection. This pre-run document does not
assign one and does not declare Water Flow or G8-B accepted or closed.
