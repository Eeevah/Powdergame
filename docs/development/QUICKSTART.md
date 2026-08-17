# Powdergame Developer Quickstart

Read this first when entering the repo.

## Current gate

- M0: IN_PROGRESS
- G0-G7: PASS / CLOSED
- G7-A: USER VALIDATED / FROZEN
- G7-B: PASS / CLOSED / FROZEN
- G8: Performance Evidence (IN_PROGRESS; historical v4 remains unbound historical data)
- G8-A: v5 official capture + independent verification complete / verified evidence candidate; same-SHA user visual validation pending
- G8-B: five-scenario shared fixture + Windows Gallery + headless selection implementation candidate; Scenario 1 Sand Fall USER ACCEPTED, Scenario 2–5 UNACCEPTED; **overall USER ACCEPTANCE PENDING / NOT CLOSED**
- G8-C: official matrix measurement not started
- Current repository line: `feature/m0-g8b-scenario-suite`, based on tested local `integration/canonical-recovery`; publication/promotion and Gate closure require explicit user direction

## Windows

Typical repo/worktree root:
`C:\Users\mdkap\source\repos\Powdergame*`

Use the gate-specific worktree when present. Never blindly pull/rebase a dirty worktree.

The preserved correction was attached without reset/stash/rebase/pull to `fix/g8a-evidence-remediation-v5` from base `a67abaf959aba0423627f35b79fce7c82d8ec9b5` and sealed at `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`. Keep that source branch frozen. Canonical Recovery uses it as the implementation parent and merges `feature/foundation-material-wiki` separately.

## Run the G8-B inspection Gallery

```bat
run_g8_benchmark_gallery.bat
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

Scenario 1 Sand Fall is user accepted. Complete settling followed by all chunks sleeping is its intended successful outcome; do not retune it to manufacture perpetual activity. Scenario 2–5 remain unaccepted. Water Flow inspection or modification is outside the current checkpoint task.

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
run_g7_activity_demo.bat
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
Do not run broad or repetitive demo smoke matrices by default. Run only the smallest smoke that is genuinely required by the current change. If user testing later exposes a problem, reproduce and validate only that affected path.

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
