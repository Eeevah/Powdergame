# G8-B — Benchmark Scenario Gallery Implementation Candidate (2026-08-17)

## 1. Status and authority boundary

- **Gate**: G8-B Benchmark Scenario Suite
- **Candidate branch**: `feature/m0-g8b-scenario-suite`
- **Candidate base**: local canonical recovery commit `ca79bb20b27041758ab4d4a224e491c171189393`
- **Candidate source SHA**: assigned only after the implementation/documentation seal
- **Status**: Scenario 1 Sand Fall **USER ACCEPTED**; Scenario 2–5 **UNACCEPTED**; G8-B overall **USER ACCEPTANCE PENDING / NOT CLOSED**
- **G8-C official matrix**: not started

This candidate implements deterministic workload construction, shared production-simulation staging, a Windows inspection Gallery, and headless scenario selection. It does not declare G8-B closed and does not contain official five-scenario performance results.

The change adds no production physics Rule, shader/pass-graph behavior, Material, G9 sandbox interaction, G7-C compaction, or other performance optimization. Targeted automation can validate construction and routing contracts, but only the user can accept the Gallery behavior and close G8-B.

---

## 2. Scenario inventory

| Slot | CLI slug | Role | Acceptance | Intended workload |
|---:|---|---|---|---|
| 1 | `sand-fall` | Official G8-B | **USER ACCEPTED — 2026-08-17** | Dense Powder movement, collision, arbitration, shelves/funnels/catch basin |
| 2 | `water-flow` | Official G8-B | UNACCEPTED | Liquid movement, density displacement, reservoir/channel/basin flow |
| 3 | `fire-heat` | Official G8-B | UNACCEPTED | Thermal propagation, combustion, Smoke, hot/cold phase work |
| 4 | `pressure-burst` | Official G8-B | UNACCEPTED | Steam expansion, Pressure diffusion, rupture/vent relief seam |
| 5 | `heavy-mixed-world` | Official G8-B | UNACCEPTED | Simultaneous movement, density, heat, reaction, Pressure workload |
| 6 | `active-sleep-g7` | G7 regression only | prior G7 regression; not G8-B acceptance | Frozen Activity/Sleep observatory geometry and exact edit-wake snapshot |

Slot 6 is not a sixth official G8-B matrix workload. It must use exactly 256×256 cells with chunk size 64. The five official fixtures share the normal 2048×2048×64 headless default and may be built on other valid rectangular worlds of at least 256×256 for development inspection; that flexibility does not redefine the later official matrix configuration.

### 2.1 Scenario 1 acceptance contract

The user accepted Sand Fall with complete settling and all chunks eventually entering sleep. That terminal state is the intended success signal: the workload demonstrates production Powder movement, collision/arbitration, convergence, and Active/Sleep behavior. Do not add a perpetual source, artificial wake, oscillation, or geometry/threshold retuning merely to keep the benchmark visibly active. Any future change to this accepted interpretation requires a new explicit user decision.

Scenarios 2–5 remain unaccepted, so Scenario 1 approval does not close G8-B. Water Flow inspection, correction, or retuning is explicitly outside the current checkpoint task and must not begin automatically.

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
run_g8_benchmark_gallery.bat
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

---

## 7. Remaining acceptance and closure work

Before G8-B may be described as closed:

1. seal the integrated candidate and record its exact source SHA;
2. preserve the accepted Scenario 1 settling/sleep contract without retuning;
3. receive separate user acceptance or concrete findings for Scenario 2–5;
4. confirm that the remaining slots have distinct, understandable workload identities and correct pristine reset/control behavior;
5. record each remaining decision. Water Flow is not part of the current checkpoint task.

G8-C remains separate. It must establish the official repeated performance matrix, production throughput, profiled GPU timing, rendering cost, simulation/render coexistence, provenance, and bottleneck decision without treating Gallery diagnostics as timed evidence.

**Current result: IMPLEMENTATION CANDIDATE — SCENARIO 1 USER ACCEPTED; SCENARIO 2–5 UNACCEPTED; G8-B OVERALL NOT CLOSED.**
