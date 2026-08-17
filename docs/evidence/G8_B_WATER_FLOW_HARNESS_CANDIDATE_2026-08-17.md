# G8-B Scenario 2 — Water Flow Harness Candidate and Fixture Remediation

Date: 2026-08-17
Status: **REMEDIATION CANDIDATE USER ACCEPTED WITH KNOWN FOLLOW-UP — AUTOMATIC NEEDS_HUMAN_REVIEW / G8-B NOT CLOSED**
Branch: `feature/m0-g8b-scenario-suite`
Remediation starting SHA: `d12edbfbcc0fb3fc2ef599cd06b3c46a2293d268`
Candidate source SHA: `5af031f1a04af866127616d4f1b0faa6c85e4d8e`

## 1. Scope and frozen baseline

The first clean-source Water candidate is preserved as immutable evidence at
Run ID `g8b-water-flow-v0-20260817T100732645294Z-f7ee7959`. Its automatic
verdict was `NEEDS_HUMAN_REVIEW`. Direct review classified the result as
`FIX REQUIRED — fixture_representativeness_issue`, with
`expected_local_movement_artifact` secondary; no production-physics defect was
established. Water moved, crossed a chunk boundary, reached the destination,
preserved Water/Oil/Matter, retained valid finite fields, and reset exactly.

The diagnosed fixture issue was geometric: both outer walls began at `y=90`,
below the Water reservoir tops at `y=22` and `y=34`. Water could therefore pass
over a wall and spread along the world bottom outside the intended basin. The
remediation changes only those two wall tops and adds direct leakage evidence.
The rejected candidate and every generated artifact remain unmodified.

- Artifact root: `C:\Users\mdkap\source\Powdergame-artifacts\g8b-water-flow-v0-20260817T100732645294Z-f7ee7959`
- Receipt SHA-256: `443ee6d2a56a9af6ff883977b02a5eccb5040f200a8fbf218d13ffb76849db1a`
- Review Packet SHA-256: `2aca0476cbaa47c9f486b8785e2e049a814b7ebefde33d44700c121ec3e83cc3`

The already accepted Sand Fall fixture, its approval record, and its generated
artifacts remain frozen. Fire / Heat, Pressure Burst, Heavy Mixed World, G8-C,
main integration, and PR creation are outside this candidate.

Current execution state:

| Item | State |
|---|---|
| First Water candidate | IMMUTABLE / SUPERSEDED; automatic `NEEDS_HUMAN_REVIEW` |
| First-candidate human review | `FIX REQUIRED — fixture_representativeness_issue` |
| Fixture remediation | SEALED at `5af031f1a04af866127616d4f1b0faa6c85e4d8e` |
| Targeted Rust/Python/Harness tests | FAST PASS recorded |
| Sand Harness regression | FAST PASS recorded; published Sand run unchanged |
| Full workspace checkpoint | PASS |
| Windows release smoke | PASS; 60 frames, RTX 5090 / DX12 |
| Remediation candidate run | `g8b-water-flow-v0-20260817T110906547252Z-8b808e66` |
| Remediation automatic verdict | `NEEDS_HUMAN_REVIEW` — unchanged |
| User acceptance | `ACCEPTED WITH KNOWN FOLLOW-UP` |

The human acceptance does not rewrite the automatic verdict to `PASS`, relax
the all-sleep contract, or modify the immutable candidate artifacts.

## 2. Audited finite fixture

`powdergame-scenarios::ScenarioId::WaterFlow` is a finite tick-0 image. It has
no recurring Water source and no scenario-specific production rule. For the
Harness world (`256x256`, chunk size `64`), the authored half-open rectangles
are:

| Role | Rectangle `[x0,x1) × [y0,y1)` | Material |
|---|---|---|
| basin floor | `[10,246) × [230,238)` | Stone |
| left wall | `[10,18) × [14,238)` | Stone |
| right wall | `[238,246) × [14,238)` | Stone |
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
| Stone | 8,104 |
| Boundary Block | 1,020 |
| Empty | 38,928 |

All authored Temperature values are the reference temperature, all Pressure
values are the reference pressure, Flags are zero, and `chunk_edit_wake` is
zero. Water remains exactly 15,244 cells and Oil remains exactly 2,240 cells.
A source-level fixture test pins the full remediated image, counts, fields,
zero state, sealed wall faces, and observation masks. The floor, internal
channel, central geometry, destination basin, Water rectangles, and Oil pocket
remain unchanged. Production movement/density shaders, Sleep/Wake semantics,
pass graph, and Material descriptors are not changed.

## 3. Observation regions

The fixture description calls for reservoirs draining through staggered
channels into a basin. The Harness may observe that behavior but must not stage
an outcome.

The destination observation mask is defined as cells that are `EMPTY` at tick
0 inside `[18,238) × [200,230)`. The mask has 6,216 cells and contains zero
Water and zero Oil initially. It lies below the final shelves, inside the side
walls, and above the floor. This mask is diagnostic metadata only; it does not
change fixture construction.

The outer-basin interior is the half-open region `[18,238) × [14,230)`.
`water_outside_outer_basin_cells` counts current Water cells outside that
region at every diagnostic sample. The remediation candidate hard predicate
passes only when the maximum observed value is exactly zero; any nonzero value
is a failure even if Water later returns inside.

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
- Water outside the remediated outer basin, including peak and final counts;
- first sleeping chunk;
- final all-sleep or a stable plateau candidate;
- post-settle physical state changes and wake observations;
- invalid Material IDs and non-finite Temperature/Pressure values;
- programmatic reset exact equivalence.

Each sample also partitions every active cell using in-bounds cardinal
neighbors. Water/Oil interfaces take priority, Water/Empty surfaces are second,
and all remaining active cells are `other`. The three counters sum exactly to
`any_active_cells`. This supports the required terminal classification if
outside leakage is zero but all-sleep still does not occur; it does not relax
the all-sleep predicate or promote a plateau to `PASS`.

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
run_experiment.bat water-flow
```

The default is `candidate`, used exactly once with a new Run ID after the
remediation source seal and FULL checkpoint. The already published first Water
candidate is not repaired, overwritten, or rerun under its old ID. Sand remains
candidate-only and preserves its v0 schemas and published artifacts.

Water uses these schema identities:

- `powdergame-experiment-manifest-v1`
- `powdergame-experiment-telemetry-v2`
- `powdergame-experiment-analysis-v2`
- shared `powdergame-experiment-frames-v0`
- `powdergame-experiment-report-v2`
- `powdergame-experiment-receipt-v2`

Its ten predicates are:

1. `actual_water_movement`
2. `cross_chunk_flow`
3. `destination_arrival`
4. `water_conservation`
5. `water_outside_outer_basin_cells`
6. `no_invalid_materials`
7. `no_nonfinite_fields`
8. `stable_bulk_before_max`
9. `post_settle_stable`
10. `exact_reset`

All ten must be `pass` for automatic `PASS`. A `fail` produces `FAIL`; any
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

## 6. Remediation checks and source seal

| Check | Recorded result |
|---|---:|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| scenario library | 7 passed / 0 failed |
| shared GPU reset integration | 1 passed / 0 failed |
| bounded Water destination/conservation/leak/reset GPU test | 1 passed / 0 failed |
| Windows Sand/Water experiment tests | 16 passed / 0 failed |
| Python coordinator/analyzer tests | 23 passed / 0 failed |
| `cargo test --workspace -- --test-threads=1` | PASS; 3 explicitly ignored manual tests |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `git diff --check` | PASS; line-ending advisories only |
| `cargo run --locked --release -p powdergame-windows -- --benchmark-gallery --smoke-frames 60` | PASS; paused tick 0; exit 0 |

These implementation checks preceded the clean source seal at
`5af031f1a04af866127616d4f1b0faa6c85e4d8e`. Exactly one remediation candidate
was then generated with a fresh Run ID. Generated artifacts remain outside Git
and are not modified by this approval record.

## 7. Candidate verdict and anomaly classification

The generated report will use `PASS`, `FAIL`, or `NEEDS_HUMAN_REVIEW`. The
first candidate is already preserved and classified. Any remediation result is
also preserved before further change and mapped, with raw observations, to one
candidate category:

- `actual_physics_defect`;
- `fixture_representativeness_issue`;
- `expected_local_movement_artifact`;
- `presentation_or_capture_issue`;
- `insufficient_evidence`.

If outer-basin leakage is zero but all-sleep still fails, the run remains
`NEEDS_HUMAN_REVIEW`; its final active-cell partition is reported without
altering production physics or the all-sleep policy.

### 7.1 Sealed remediation candidate result

- Source SHA: `5af031f1a04af866127616d4f1b0faa6c85e4d8e`
- Run ID: `g8b-water-flow-v0-20260817T110906547252Z-8b808e66`
- Automatic verdict: `NEEDS_HUMAN_REVIEW`
- Human verdict: `ACCEPTED WITH KNOWN FOLLOW-UP`
- Review Packet SHA-256: `83783025ee6bdac8f6dedbf25edfec1dd75040d533c9fc563157cc699b5caec5`
- Receipt SHA-256: `96f60b465dbfa4f7a4cacd7f78f475cad9af7c2e6d1754aba4dddc186d497c1b`
- Outer-basin Water maximum/final: `0 / 0`
- Matter/Water/Oil conservation: `PASS`
- Movement / cross-chunk flow / destination arrival: `PASS / PASS / PASS`
- Invalid Material IDs / non-finite Temperature / non-finite Pressure: `0 / 0 / 0`
- Exact reset: `PASS`
- Final active cells: `64` — Water/EMPTY interface `51`, Water/Oil interface
  `1`, other `12`

The accepted known follow-up is minority-cell persistent rearrangement at the
M0 local-liquid free surface. This candidate establishes no evidence of a
production-physics defect. The automatic `NEEDS_HUMAN_REVIEW` remains the
artifact verdict, all-sleep remains unchanged, and the candidate Run ID and
artifacts are immutable. Water Flow is user accepted with that known follow-up;
G8-B remains **NOT CLOSED** because Scenarios 3–5 still require acceptance.
