# G8-B — Benchmark Scenario Gallery Implementation Candidate and User Acceptance Closure (2026-08-17 to 2026-08-19)

## 1. Status and authority boundary

- **Gate**: G8-B Benchmark Scenario Suite
- **Candidate branch**: `feature/m0-g8b-scenario-suite`
- **Candidate base**: local canonical recovery commit `ca79bb20b27041758ab4d4a224e491c171189393`
- **Candidate source SHA**: assigned only after the implementation/documentation seal
- **Water Harness base**: `b884abcfbab8e104bdf34e2e8d19635b157c1638`
- **Water remediation base**: `d12edbfbcc0fb3fc2ef599cd06b3c46a2293d268`
- **Status**: all five official scenarios and Cell Inspector v0 **USER ACCEPTED**; Water, Pressure, and Heavy automatic `NEEDS_HUMAN_REVIEW` remain unchanged with known follow-ups; Fire automatic `PASS` remains unchanged; G8-B **CLOSED / FROZEN**
- **G8-C official matrix**: **NEXT / AUTHORIZED**, not yet executed

This candidate implements deterministic workload construction, shared production-simulation staging, a Windows inspection Gallery, and headless scenario selection. Separate immutable Harness evidence and user decisions later accepted all five official scenarios, so G8-B is now **CLOSED / FROZEN**. This record still does not contain official five-scenario performance results; G8-C remains a separate authorized next stage.

The change adds no production physics Rule, shader/pass-graph behavior, Material, G9 sandbox interaction, G7-C compaction, or other performance optimization. Targeted automation can validate construction and routing contracts, but only the user can accept the Gallery behavior and close G8-B.

---

## 2. Scenario inventory

| Slot | CLI slug | Role | Acceptance | Intended workload |
|---:|---|---|---|---|
| 1 | `sand-fall` | Official G8-B | **USER ACCEPTED — 2026-08-17** | Dense Powder movement, collision, arbitration, shelves/funnels/catch basin |
| 2 | `water-flow` | Official G8-B | **USER ACCEPTED WITH KNOWN FOLLOW-UP — 2026-08-17** | Liquid movement, density displacement, reservoir/channel/basin flow |
| 3 | `fire-heat` | Official G8-B | **USER ACCEPTED — 2026-08-18; AUTOMATIC PASS UNCHANGED** | Thermal propagation, combustion, Smoke, hot/cold phase work |
| 4 | `pressure-burst` | Official G8-B | **USER ACCEPTED WITH KNOWN FOLLOW-UP — 2026-08-18; AUTOMATIC NEEDS_HUMAN_REVIEW UNCHANGED** | Steam expansion, Pressure diffusion, rupture/vent relief seam |
| 5 | `heavy-mixed-world` | Official G8-B | **USER ACCEPTED WITH KNOWN FOLLOW-UP — 2026-08-19; AUTOMATIC NEEDS_HUMAN_REVIEW UNCHANGED** | Simultaneous movement, density, heat, reaction, Pressure workload |
| 6 | `active-sleep-g7` | G7 regression only | prior G7 regression; not G8-B acceptance | Frozen Activity/Sleep observatory geometry and exact edit-wake snapshot |

Slot 6 is not a sixth official G8-B matrix workload. It must use exactly 256×256 cells with chunk size 64. The five official fixtures share the normal 2048×2048×64 headless default and may be built on other valid rectangular worlds of at least 256×256 for development inspection; that flexibility does not redefine the later official matrix configuration.

### 2.1 Scenario 1 acceptance contract

The user accepted Sand Fall with complete settling and all chunks eventually entering sleep. That terminal state is the intended success signal: the workload demonstrates production Powder movement, collision/arbitration, convergence, and Active/Sleep behavior. Do not add a perpetual source, artificial wake, oscillation, or geometry/threshold retuning merely to keep the benchmark visibly active. Any future change to this accepted interpretation requires a new explicit user decision.

The first Water candidate is preserved at Run ID `g8b-water-flow-v0-20260817T100732645294Z-f7ee7959`; automatic `NEEDS_HUMAN_REVIEW` was followed by human `FIX REQUIRED — fixture_representativeness_issue`. The source `5af031f` remediation run `g8b-water-flow-v0-20260817T110906547252Z-8b808e66` keeps automatic `NEEDS_HUMAN_REVIEW` and is human `ACCEPTED WITH KNOWN FOLLOW-UP`. The correction is fixture-only and does not establish a production-physics defect. Fire / Heat was accepted from its unchanged-fixture automatic-`PASS` candidate. Pressure Burst was accepted from its clean cold-seam causal candidate with automatic `NEEDS_HUMAN_REVIEW` unchanged. Heavy Mixed World source/run `07260fffab22e5b4513eb168f0baac36e374ab94` / `g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4` was accepted with the same unchanged automatic verdict, 14/14 hard pass, and `candidate_blocker=false`. These separate decisions close and freeze G8-B; they do not retroactively alter any automatic verdict or historical artifact.

### 2.2 Scenario 2 candidate boundary

Water Flow uses a finite 256×256×64 tick-0 fixture. Direct review found that its outer walls started at `y=90`, below Water starting at `y=22` and `y=34`, allowing an exterior-bottom bypass. The remediation changes only the side walls from `[10,18) × [90,238)` / `[238,246) × [90,238)` to `[10,18) × [14,238)` / `[238,246) × [14,238)`. Water remains 15,244, Oil 2,240, Boundary Block 1,020, and the 6,216-cell destination mask remains unchanged; Stone becomes 8,104 and Empty 38,928. Internal channel, central geometry, basin floor, Water/Oil staging, shared reset, and production physics are unchanged.

The `run_experiment.bat water-flow` remediation candidate was generated once with a fresh Run ID after the FULL checkpoint and source seal. Both Water runs remain immutable. Unique/create-new/no-overwrite storage, hashes, and final `EXPERIMENT_RECEIPT.json` publication remain unchanged. Water telemetry/analysis/report/receipt use v2 while manifest remains v1 and frames remain v0. The tenth hard predicate, `water_outside_outer_basin_cells`, requires a maximum of zero outside `[18,238) × [14,230)`. The accepted remediation observed max/final `0 / 0`; all-sleep and plateau verdict policy remain unchanged, and automatic `NEEDS_HUMAN_REVIEW` was not rewritten.

---

## 3. Shared fixture and staging architecture

The workspace crate `apps/scenarios` (`powdergame-scenarios`) is the single source for Windows and headless fixture identity and construction.

Public contract:

- `ScenarioId` — stable number, slug, name, description, official-vs-regression identity
- `ScenarioFixture::build` — deterministic CPU tick-0 image
- `validate_scenario_config` — validation before dense allocation or GPU initialization
- `reset_and_stage_scenario` — production `Simulation` reset and GPU staging

Each fixture owns authored Material, Temperature, Pressure, Flags, and `chunk_edit_wake` arrays. Staging writes the complete authoritative image to both Current and Next buffers, restores the authored edit-wake snapshot, submits pending transfer work, and waits for completion. Repeating the same scenario/config therefore begins each inspected or measured window at the same pristine tick-0 state.

The headless harness calls this exact shared reset/stage function before:

1. Mode A prewarm,
2. every Mode A throughput trial,
3. Mode B prewarm,
4. every Mode B profiled trial,
5. batched-unprofiled overhead control,
6. synchronized-unprofiled overhead control,
7. synchronized-profiled overhead control.

The legacy `calibration` selection remains on its original G8-A fixture path. No Gallery module, window, renderer, HUD, or Gallery diagnostic readback is linked into the benchmark execution path.

---

## 4. Windows inspection Gallery

Launcher:

```bat
run_powdergame.bat gallery
```

Direct entry:

```bat
cargo run --release -p powdergame-windows -- --benchmark-gallery
```

The Gallery uses a 256×256 world with 64×64 chunks so all six shared fixtures, including the exact G7 regression, can be selected in one inspection surface. It always starts paused.

`apps/windows/build.rs` embeds the source HEAD and dirty state that produced the executable; the HUD and console do not infer build provenance from a later checkout at launch time. Scenario selection and `R` are transactional at the presentation-state boundary: the previous scenario/tick/sample attribution remains committed until shared reset/staging succeeds. A reset failure is shown explicitly, marks SIM TICK unavailable, and suppresses play, step, and diagnostic sampling until a new reset request succeeds.

| Control | Contract |
|---|---|
| `1`–`6` | Select scenario; pristine shared reset; paused; x1; simulation tick 0; diagnostic sample cleared |
| `SPACE` | Play / pause |
| `N` | Exactly one production simulation tick while paused |
| `F` | Cycle x1 / x4 / x16 sequential tick multiplier |
| `R` | Pristine reset of the current scenario; paused; x1; tick 0 |
| `ESC` | Quit |

The HUD and console expose source SHA, Git state, build profile, scenario identity, WorldConfig, sleep settings, simulation tick, and the most recent diagnostic sample identity. The activity census is bounded/rate-limited and labels its source tick separately.

Rendering, HUD generation, window event handling, wall-clock TPS, and diagnostic census/readback are inspection behavior. They are not official timed benchmark data and must not be copied into G8-C as simulation throughput, GPU pass timing, render timing, or coexistence evidence.

---

## 5. Headless selection and evidence identity

Accepted selection values:

```text
calibration
sand-fall
water-flow
fire-heat
pressure-burst
heavy-mixed-world
active-sleep-g7
```

Example:

```bat
cargo run --release -p powdergame-benchmark -- --scenario sand-fall
```

Exact G7 regression example:

```bat
cargo run --release -p powdergame-benchmark -- --scenario active-sleep-g7 --width 256 --height 256 --chunk 64
```

Identity contract:

| Selection | Schema | Run ID | Default aggregate path |
|---|---|---|---|
| `calibration` | `powdergame-g8a-v5` | `g8a-*` | `target/calibration_report.csv` |
| shared fixture | `powdergame-g8b-fixture-v1` | `g8b-<slug>-*` | `target/<slug>_report.csv` |

An explicit `--csv` overrides the default regardless of argument order. Aggregate, raw tick, raw cell, and raw chunk CSVs retain the existing column shape. Shared-fixture aggregate method notes append `scenario=<slug>`; calibration notes remain unchanged. These identities prevent a shared workload from being mislabeled as the legacy calibration and prevent different default scenarios from colliding on one aggregate filename.

This is a G8-B fixture-selection identity, not a new official-capture declaration. No G8-C capture or performance conclusion is recorded here.

---

## 6. Recorded targeted checks

The following checks were reported during this implementation round:

| Command | Recorded result | Scope |
|---|---|---|
| `cargo test -p powdergame-scenarios --lib` | 6 passed, 0 failed | scenario metadata, deterministic payloads, validation, subsystem content |
| `cargo test -p powdergame-scenarios --test gpu_reset -- --test-threads=1` | 1 passed, 0 failed | actual-GPU reset/restage equivalence |
| `cargo clippy -p powdergame-scenarios --all-targets -- -D warnings` | exit 0 | shared crate targets |
| `cargo test -p powdergame-benchmark` | 27 passed, 0 failed | CLI/config, staging routing, evidence identity, existing benchmark unit contracts |
| `cargo test -p powdergame-windows gallery -- --test-threads=1` | 7 passed, 0 failed | Gallery state, diagnostics, build provenance vocabulary, transactional selection/reset, CLI/control contracts |
| `cargo check -p powdergame-windows` | exit 0 | Windows package compile check |
| `cargo clippy -p powdergame-windows --all-targets -- -D warnings` | exit 0 | Windows package targets |
| `cargo fmt --all -- --check` | exit 0 | integrated workspace formatting |
| `cargo clippy -p powdergame-scenarios -p powdergame-benchmark -p powdergame-windows --all-targets -- -D warnings` | exit 0 | integrated changed-package targets |
| `cargo run --locked --release -p powdergame-windows -- --benchmark-gallery --smoke-frames 60` | exit 0 | one bounded RTX 5090 / DX12 Gallery presentation launch; paused tick 0; 60 frames |

These are targeted implementation checks, not user acceptance and not G8-C measurement. No broad demo smoke matrix, long headless performance run, five-scenario official matrix, generated evidence capture, or external adversarial review was run for this candidate.

Water remediation FAST checks passed: workspace fmt/check, scenarios library 7/7, shared all-six GPU reset 1/1, bounded Water destination/conservation/leak/reset GPU test 1/1, Windows Sand/Water experiment 16/16, and Python coordinator/analyzer 23/23. The single FULL workspace test/clippy/diff checkpoint and one 60-frame RTX 5090/DX12 Gallery release smoke also passed. Source `5af031f` candidate `g8b-water-flow-v0-20260817T110906547252Z-8b808e66` retained automatic `NEEDS_HUMAN_REVIEW` and received human `ACCEPTED WITH KNOWN FOLLOW-UP`; packet/receipt hashes remain recorded in the Water evidence document.

---

## 7. User acceptance closure

The closure requirements were satisfied without rewriting automatic verdicts or historical artifacts:

1. Sand Fall: **USER ACCEPTED**;
2. Water Flow: **USER ACCEPTED WITH KNOWN FOLLOW-UP**, automatic `NEEDS_HUMAN_REVIEW` unchanged;
3. Fire / Heat: **USER ACCEPTED**, automatic `PASS` unchanged;
4. Pressure Burst: **USER ACCEPTED WITH KNOWN FOLLOW-UP**, automatic `NEEDS_HUMAN_REVIEW` unchanged;
5. Heavy Mixed World: **USER ACCEPTED WITH KNOWN FOLLOW-UP**, automatic `NEEDS_HUMAN_REVIEW` unchanged, 14/14 hard predicates PASS, `candidate_blocker=false`;
6. Cell Inspector v0: **USER ACCEPTED WITH KNOWN FOLLOW-UP**.

Heavy acceptance is bound to source `07260fffab22e5b4513eb168f0baac36e374ab94` and run `g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4`. Matter movement, Water/Oil density displacement, phase work, combustion and new Smoke, Pressure activity, four-subsystem concurrency at tick `8`, `1,986` samples of `>=3` subsystem overlap over ticks `1..15,872`, peak active `40,301 @ 3,528`, zero unexplained inventory/invalid/non-finite/wake anomaly, no runaway, and exact reset support the human decision. The terminal broad Thermal tail is large but monotonically decreases through the terminal window while Pressure and Reaction have ended. It is a known G8-C workload-cost follow-up, not a correctness failure. There is no production-physics defect evidence; fixture remediation and candidate rerun are not required.

The preserved raw `first_vent*` vocabulary means first exterior Steam above relief. It is not opening-gated causal vent proof and is not a Heavy hard predicate or acceptance ground.

G8-C remains separate. It is now **NEXT / AUTHORIZED** and must establish the official repeated performance matrix, production throughput, profiled GPU timing, rendering cost, simulation/render coexistence, provenance, and bottleneck decision without treating Gallery diagnostics as timed evidence. G8-A user visual durable closure remains separate, G8 overall remains `IN_PROGRESS`, and G9 remains `PENDING`.

**Current result: ALL FIVE OFFICIAL SCENARIOS + CELL INSPECTOR V0 USER ACCEPTED; G8-B CLOSED / FROZEN; G8-C NEXT / AUTHORIZED; G8 OVERALL IN_PROGRESS; G9 PENDING.**
