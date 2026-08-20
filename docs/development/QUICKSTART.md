# Powdergame Developer Quickstart

Read this first when entering the repo.

Executable, launcher, worktree, and artifact-copy rules are defined in
[`WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md`](WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md).
Use `run_powdergame.bat` for the user application and `run_experiment.bat` for
automated scenario evidence; do not add a Gate-specific executable or launcher.

## Current gate

- M0: IN_PROGRESS
- G0-G7: PASS / CLOSED
- G7-A: USER VALIDATED / FROZEN
- G7-B: PASS / CLOSED / FROZEN
- G8: CLOSED / FROZEN; official Matrix independently verified and recommends `PROCEED_TO_G9`
- G8-A: verified official capture; separate historical visual requirement formally superseded without relabeling the old capture
- G8-B: five official scenarios and Cell Inspector user accepted; **CLOSED / FROZEN**; automatic verdicts and immutable artifacts unchanged
- G8-C: official A/B/C/D Matrix independently reconstructed with mismatch `0`; recommendation `PROCEED_TO_G9`
- G9-A: source `a00e39b2e00bfbd9ac28214c44cd22cc97542bb4`, **REVISED IMPLEMENTATION CANDIDATE / USER RE-REVIEW PENDING** after a second revision-required user review
- Thermal Transport & Ignition Causality: **PLANNED / DESIGN REQUIRED / IMPLEMENTATION NOT STARTED**; G9-B prerequisite
- G9-B/C/D/E and optimization: **NOT STARTED**
- Current work line: `feature/m0-g9-first-playable`; shared `main` promotion requires explicit user direction

## Windows

Typical repo/worktree root:
`C:\Users\mdkap\source\repos\Powdergame*`

Use the gate-specific worktree when present. Never blindly pull/rebase a dirty worktree.

The preserved correction was attached without reset/stash/rebase/pull to `fix/g8a-evidence-remediation-v5` from base `a67abaf959aba0423627f35b79fce7c82d8ec9b5` and sealed at `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`. Keep that source branch frozen. Canonical Recovery uses it as the implementation parent and merges `feature/foundation-material-wiki` separately.

## Run the G9-A First Playable Sandbox candidate

```bat
run_powdergame.bat sandbox
```

The root BAT may also be double-clicked with no arguments; the canonical EXE itself defaults to the same Sandbox. Compatibility alias: `run_powdergame.bat play`. Explicit `run_powdergame.bat gallery` retains G8-B. Controls: left drag selected tool, right drag Erase, middle drag Pan, wheel Zoom, Shift+wheel brush size; `1` Stone, `2` Sand, `3` Water, `4` Wood, `5` Oil, `6` Ice, `7` Steam, `8` Smoke, `9` Boundary Block; `D/E/H/C` tools, `SPACE` pause/play, `N` single step, `F` speed, `R` current preset reset, `L/B` Starter/Blank, `I` Inspector, `ESC` quit. Draw affects EMPTY Cells only. Ice/Steam placement starts at -30°C/80°C; Heat/Cool changes non-EMPTY Matter by +25°C/-25°C and shows presentation-only brush feedback.

## Run the G8-B inspection Gallery

```bat
run_powdergame.bat gallery
```

The Gallery uses one shared scenario source with the headless harness and starts paused at a pristine tick-0 state.

| Key | Action |
|---|---|
| `1`–`5` | Select Sand Fall, Water Flow, Fire / Heat, Pressure Burst, or Heavy Mixed World |
| `6` | Select the exact G7 Active / Sleep regression fixture |
| `SPACE` | Play / pause |
| `N` | Advance exactly one simulation tick while paused |
| `F` | Cycle x1 / x4 / x16 sequential tick multiplier |
| `R` | Pristine reset; return to paused x1 at tick 0 |
| `ESC` | Quit |

Gallery rendering, HUD, wall-clock TPS, and bounded activity-census readback are inspection diagnostics outside official timing. They are not G8-C performance evidence.

Scenario 1 Sand Fall is user accepted. Complete settling followed by all chunks sleeping is its intended successful outcome; do not retune it to manufacture perpetual activity. Scenario 2 Water Flow is user `ACCEPTED WITH KNOWN FOLLOW-UP`; its automatic `NEEDS_HUMAN_REVIEW` and immutable artifacts are unchanged. Scenario 3 Fire / Heat keeps its fixture, production physics, sealed automatic-`PASS` candidate, and artifacts unchanged and is `USER ACCEPTED`. Scenario 4 Pressure Burst is next; Scenario 5 remains pending.

## Validated Sand Fall experiment pilot

The approved pilot used the following entry point from clean experiment source `9e1fdac44aa14a546c7fe5ad6ceba49e71777eb5`. Do not rerun Sand Fall or the Harness pilot for this closure.

```bat
run_experiment.bat sand-fall
```

The recorded Sand command retains its v0 schema and immutable artifacts. The coordinator dispatches `sand-fall`, `water-flow`, or `fire-heat` and writes each unique run beneath `C:\Users\mdkap\source\Powdergame-artifacts`. It refuses dirty/detached source and existing output paths. `EXPERIMENT_RECEIPT.json` is the final write inside the Run directory; its absence means the preserved run is incomplete and must not be repaired or reused. Candidate-only Audit Bundle delivery is a sibling write after that marker.

The run records raw stdout/stderr, telemetry samples/events, worker analysis, 6–10 semantic RGBA frames, derived full/crop PNGs, reports, contact sheet, inert ChatGPT review prompt, review packet, and SHA-256 inventory outside Git. An automatic `PASS` means the seven hard Sand Fall predicates passed for that run; it does not close G8-B or establish Water Flow/G8-C evidence.

The documented pilot completed as run `g8b-sand-fall-v0-20260817T065311878587Z-3ebd7505`, with automatic verdict `PASS` and `HARNESS REVIEW OUTPUT APPROVED`. The later docs-only closure commit records the result but is not the experiment source. G8-B remains **NOT CLOSED**; Water Flow is separately accepted with a known follow-up, Fire / Heat is user accepted with its automatic-`PASS` candidate unchanged, and later scenarios remain pending. See `docs/evidence/G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md`.

## Water Flow Harness candidate

The Water worker reuses pristine `ScenarioId::WaterFlow` staging and production ticks. At 256×256×64, the remediated tick 0 contains Water 15,244, Oil 2,240, Stone 8,104, Boundary Block 1,020, and Empty 38,928 cells. The destination observation mask remains the 6,216 tick-0 `EMPTY` cells inside `[18,238) × [200,230)`; it is diagnostic only. The outer-basin interior is `[18,238) × [14,230)`.

The first candidate and its human classification are immutable. The sealed remediation candidate is also immutable and must not be rerun:

```bat
rem Run ID: g8b-water-flow-v0-20260817T110906547252Z-8b808e66
```

The default Water mode is `candidate`; scratch Run IDs contain `-scratch-`. Both modes use create-new directories, exact schema validation, hashes, and receipt-last publication. The preserved first candidate used Water v1. The remediation uses Water v2 with ten tri-state predicates: the previous nine plus `water_outside_outer_basin_cells`, which passes only when its maximum is zero. A stable plateau may terminate collection but remains `unknown` for `stable_bulk_before_max`, producing `NEEDS_HUMAN_REVIEW` rather than silently claiming `PASS`.

Remediation FAST checks, the single FULL checkpoint, and the 60-frame release smoke passed. Source `5af031f1a04af866127616d4f1b0faa6c85e4d8e`, Run ID `g8b-water-flow-v0-20260817T110906547252Z-8b808e66`, outer-basin maximum/final `0 / 0`, conservation/movement/cross/destination/integrity/reset pass, and final active partition `64 = 51 Water/EMPTY + 1 Water/Oil + 12 other` are recorded. The automatic verdict stays `NEEDS_HUMAN_REVIEW`; the human verdict is `ACCEPTED WITH KNOWN FOLLOW-UP`. See `docs/evidence/G8_B_WATER_FLOW_HARNESS_CANDIDATE_2026-08-17.md`.

## Fire / Heat Harness candidate

The unchanged 256×256×64 fixture contains finite Wood 10,926, Oil 1,610,
Ice 2,240, and Water 1,536. It stages authored hot/combusting seeds but no
scenario-specific outcome. Tick-0 flags are not accepted as production
combustion evidence because 68 flagged cells overlap non-combustible Stone.

After the clean source seal and the single post-seal checkpoint, the candidate
entry point is:

```bat
run_experiment.bat fire-heat
```

The Fire analyzer separates reaction termination from its later thermal tail;
it does not require whole-world all-sleep and does not fail merely because heat
remains after reaction work ends. It records genuine post-tick Wood/Oil
combustion, Smoke, heat propagation, phase inventory change, finite fuel use,
reaction-zero, post-reaction restart/tail, field integrity, and exact reset.
Candidate mode freezes the executed binary in the unique Run directory and
publishes a sibling `AUDIT_BUNDLE.zip` plus SHA-256 sidecar after the receipt.
The ordinary `REVIEW_PACKET.zip` remains a lightweight human-review packet and
is not presented as a complete source/binary forensic bundle.

The single sealed candidate used source
`1635fdb9f562192123c92846e137b125c684ede9` and Run ID
`g8b-fire-heat-v0-20260817T133938546075Z-0e6aa901`. It recorded automatic
`PASS`, zero reaction restarts during the 180-tick post-reaction window, exact
reset, and no independently detected inventory/digest/telemetry/image mismatch.
Review Packet, Receipt, and sibling Audit Bundle SHA-256 are respectively
`2a8e99d14bf0647b71e7ef32e3840655117e93b9f20ad1360af97d62a69eb940`,
`ed17e75f7515d155f8b6e5a41a0aeb751b2876ec573658a6e49eb6dd72108aff`, and
`1c1df01dfa9004b9273bc45e4b01d3c784d5c377f98a9417bc0b7594c6a83706`.
The user accepted Scenario 3 from this immutable evidence without a production
physics change or candidate rerun. This acceptance does not close G8-B.
See `docs/evidence/G8_B_FIRE_HEAT_HARNESS_CANDIDATE_2026-08-17.md`.

## Select a headless fixture

Headless performance runs are manual-only. Do not add them to normal build/test loops.

```bat
cargo run --release -p powdergame-benchmark -- --scenario sand-fall
```

The accepted shared values are `sand-fall`, `water-flow`, `fire-heat`, `pressure-burst`, `heavy-mixed-world`, and `active-sleep-g7`. The first five use the normal 2048×2048×64 benchmark default. `active-sleep-g7` requires its exact frozen configuration:

```bat
cargo run --release -p powdergame-benchmark -- --scenario active-sleep-g7 --width 256 --height 256 --chunk 64
```

Without an explicit `--csv`, shared fixtures write `target/<slug>_report.csv`; the legacy default `calibration` path remains `target/calibration_report.csv`. See `docs/evidence/G8_B_BENCHMARK_SCENARIO_GALLERY_2026-08-17.md` for the implementation and closure boundary.

## Run current G7 demo

```bat
run_powdergame.bat activity
```

Direct:
```bat
cargo run --release -p powdergame-windows -- --activity-demo
```

Controls: `SPACE` play/pause · `N` one tick · `F` x1/x4/x16 · `R` reset · `ESC` quit.

## Validation policy

**FAST — normal iteration**
```bat
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test <targeted tests>
```

**FULL — once per gate/checkpoint round**
```bat
cargo test --workspace -- --test-threads=1
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```
Do not run broad or repetitive bounded launch matrices by default. Run only the smallest application startup check genuinely required by the current change. If user testing later exposes a problem, reproduce and validate only that affected path.

**PERFORMANCE — manual only**
Do not run performance benchmarks during normal build/test loops. G8-B fixture selection exists, but G8-C official matrix measurement remains a separate, not-yet-started step.

The historical G8-A v4 aggregate/raw timing CSVs can be numerically reconstructed, but they are not bound to the later dirty source snapshot or executed binary and do not contain raw census buffers. Do not label them an official baseline.

The verified v5 package for `9abec9e` already has a complete official receipt and independent-verifier record. Do not rerun, replace, or repair that capture. If a future source SHA requires a new auditable capture, use `apps/benchmark/capture-evidence.ps1 -Official` instead of invoking the benchmark binary or `cargo run` directly. Official mode requires an attached clean source SHA, a new Capture ID, and a new empty destination outside the repository. `CAPTURE_RECEIPT.json` remains the final completion marker; without it the capture is incomplete.

```powershell
pwsh -NoProfile -File .\apps\benchmark\capture-evidence.ps1 `
  -Official `
  -DestinationRoot <new-empty-directory-outside-the-repository>
```

The v5 remediation branch stopped after source publication, one official capture, and independent verification. Canonical Recovery is a separate local integration line; its existence does not approve publication, close G8, or select the next product Gate.

The G8-B candidate adds authored fixtures and inspection/selection surfaces only. It does not add production physics, a Material, G9 interaction, or an optimization. Targeted automation and Gallery diagnostics do not close G8-B; user acceptance is still required.

**ADVERSARIAL REVIEW — opt-in only**

Do not automatically request, perform, or file an adversarial review. Only do so when the user explicitly requests it, following `docs/adversarial-reviews/README.md`. Do not send Powdergame code, diffs, artifacts, or review prompts to GPT Pro, Grok, or another external AI reviewer.

## Never forget

- GPU production simulation is authoritative.
- One Cell = Max One Matter; EMPTY is not hidden air.
- Demo/HUD/diagnostics must not silently change physics.
- Gallery diagnostics and rendering must never be reported as official benchmark timing.
- Frozen G0-G6 physics needs explicit justification to change.
- Temperature is a relative gameplay scalar, not Celsius.
- `docs/planning/MATERIAL_CANDIDATES.md` is user-owned; do not touch it.
- AI/CI may reach VALIDATION; user approval is required to close a gate.

For details: `docs/planning/STATUS.md`, `docs/HANDOFF.md`.
