# G7-A — Activity Observatory Hardening Contract

Status: **HARDENING REQUIRED / IN_PROGRESS**  
Scope: **G7-A measurement + user-observation fixture only**  
Starting implementation baseline: `ebebfde45bcec9c7e3170c5c60c6c5e8825137f1`  
G7-B actual sleep/work skipping: **NOT STARTED / OUT OF SCOPE**

## 1. Why this hardening round exists

The user observed the G7-A Activity Observatory on the RTX 5090 for roughly tick 0 / 117 / 411 / 679 / 1515 / 3019.

The long run was useful: the detector is clearly observing real movement/thermal/pressure/reaction phenomena, but the current user-validation fixture cannot yet be accepted because the observation path contains both fixture problems and detector correctness mismatches.

This round is not a visual retune and is not G7-B. It must preserve G0-G6 physics and keep the G7 activity EPS thresholds unchanged. However, a detector that reports work where the frozen subsystem would perform no work is a correctness bug and must be fixed.

## 2. Confirmed defects

### 2.1 Activity HUD is never constructed

`render_activity_hud()` exists and already supports global/panel metrics, but `Renderer::new()` only constructs `TextRenderer` for `ThermalLab | Integrity`. `PresentationPalette::Activity` is omitted.

Fix: include `PresentationPalette::Activity` in the existing `needs_text_hud` path. Reuse the existing text renderer; do not create another HUD system.

The Activity HUD must visibly show:

- `SIM TICK`
- `DIAGNOSTIC SAMPLE`
- global Total Chunks / Matter Active / Thermal Active / Pressure Active / Reaction Active / Fully Stable / Max Stable Ticks
- **Sampled Wake Candidates** (not actual wake events)
- panel A/B/C/D M/T/P/R, stable N/total, max stable ticks, current state
- heatmap legend
- controls including fast-forward

### 2.2 Activity fast-forward regression

`DemoState.fast` and x1/x4/x16 behavior still exist, but `request_fast_forward()` currently returns for every mode except `ParallelIntegrity`. Therefore `F` does nothing in the Activity Observatory.

Fix: Activity must support the same observation-only x1 → x4 → x16 → x1 cycle.

Contracts:

- `F`: cycle x1/x4/x16
- `SPACE`: play/pause
- `N`: exactly one production simulation tick, regardless of fast multiplier
- `R`: reset world/metrics and fast multiplier to x1
- `ESC`: quit
- fast-forward only changes how many sequential production ticks are executed per update opportunity; it must not change `Simulation::tick()` semantics
- Activity HUD, window title, startup console text, and the launch BAT must expose the `F` control

## 3. Detector correctness fixes — thresholds remain frozen

### 3.1 Stale non-thermal activity bits

Current behavior is unsafe for measurement:

- the phase pass clears only the THERMAL bit in `cell_activity`
- the final activity pass writes `mask | cell_activity[index]`

As a result, MATTER / PRESSURE / REACTION bits from a previous tick may survive after their frontier disappears.

This can permanently reset `chunk_stable_ticks` and can make a chunk remain colored after real activity has ended.

Minimal required fix:

- preserve only the current-tick phase-transition marker from the phase pass
- overwrite every other bit from the current detector result each tick

Recommended equivalent form in `activity_propose.wgsl`:

```wgsl
let phase_marker = cell_activity[index] & ACTIVITY_THERMAL;
cell_activity[index] = mask | phase_marker;
```

The phase pass continues to clear/re-set THERMAL for its current tick. Do not turn `cell_activity` into persistent history.

Required regression: create a real MATTER/PRESSURE/REACTION frontier, remove/settle it through normal semantics, and prove the corresponding bit clears and stable ticks resume.

### 3.2 THERMAL activity must mirror frozen G4 heat-exchange participation

Current detector compares temperature against every in-domain neighbor. Frozen G4 thermal physics does not: EMPTY is not a thermal medium, and an edge whose effective conductivity is zero performs no heat exchange. Boundary Block has conductivity 0.

Therefore a temperature difference across EMPTY or a zero-conductivity Boundary Block is **not** thermal work and must not keep a chunk active.

Fix detector semantics, without changing `THERMAL_ACTIVITY_EPS`:

- add the existing conductivity table as a **read-only** activity-detector input
- a neighbor temperature difference is THERMAL frontier only when the actual G4 thermal edge participates (`self` and neighbor are valid Matter and effective conductivity > 0)
- EMPTY does not create a thermal frontier
- Boundary Block (`K=0`) does not create a thermal frontier
- combusting heat source and phase-candidate/actual-transition semantics remain unchanged

Required regressions:

- hot Matter adjacent only to EMPTY does not report THERMAL solely because EMPTY is at reference temperature
- a temperature difference across Boundary Block does not report THERMAL
- a real conductive Stone↔Stone (or equivalent) gradient still reports THERMAL
- the existing cross-chunk conductive thermal frontier still reports both relevant chunks

This is detector correctness alignment to frozen G4, not physics tuning.

### 3.3 PRESSURE activity must mirror frozen G5 pressure-medium exchange

Current detector requires only the self cell to be a Liquid/Gas medium, then compares its pressure against any in-domain neighbor. Frozen G5 pressure propagation exchanges pressure only between pressure media (Liquid/Gas).

Therefore a pressured Water/Steam cell next to Stone/EMPTY must not remain PRESSURE-active solely because the non-medium neighbor has pressure 0.

Fix detector semantics, without changing `PRESSURE_ACTIVITY_EPS`:

- self must be a pressure medium
- neighbor must also be a pressure medium before its pressure difference can create a PRESSURE frontier

Required regressions:

- isolated/uniformly pressured medium sealed by Stone does not report PRESSURE solely at the Stone boundary
- Water↔Water or Gas↔Gas pressure gradient still reports PRESSURE
- the existing cross-chunk pressure-medium gradient still reports both relevant chunks
- update any old test whose expectation encoded the incorrect `Water next to Stone = pressure frontier` assumption

This is detector correctness alignment to frozen G5, not pressure-physics tuning.

## 4. Panel fixture hardening

The Activity Observatory is a 256×256 world with 64×64 chunks, i.e. 4×4 chunks. The 2×2 panels therefore each contain exactly four chunks.

### 4.1 Thermal isolation between A/B/C/D

The current central cross uses Stone. Stone conducts heat, so a long run can couple the experiments.

Use existing `MATERIAL_BOUNDARY_BLOCK` for the G7-A central cross at x=127/128 and y=127/128. This is an existing registered STATIC Matter with thermal conductivity 0; it is not a new hidden wall/material and does not change production material semantics.

Only the **G7-A diagnostic fixture** central divider changes. Do not change G5/G6 fixtures.

The corrected detector must also respect Boundary Block K=0, so a temperature difference across the divider does not falsely report THERMAL activity.

### 4.2 Panel B — true stable Steam/Gas control

Purpose: prove `Gas existence != Activity`.

Stage a sealed bulk with:

- Steam T=80
- Stone chamber shell T=80
- no EMPTY interface inside the chamber
- no staged pressure
- no staged reaction state
- Steam remains on the stable side of its condensation threshold
- no initial temperature gradient on the Steam↔Stone interface

Do not inject fake stability flags or pressure.

Expected production observation after staging:

- the four Panel-B chunks converge immediately/quickly to activity mask 0 according to actual measured behavior
- all four become fully stable
- their `chunk_stable_ticks` continue increasing for the long run
- large Steam existence alone never sets MATTER activity

Do not invent an exact transient tick count before measuring it.

### 4.3 Panel A — stable bulk + real movement frontier

Keep the intended structure: sealed Water bulk plus a separate draining Water column.

The chunk-level evidence must distinguish:

- stable-control chunk(s) containing the sealed bulk → stable ticks increase
- chunk(s) containing the draining/interface frontier → MATTER activity while movement is possible

Use direct chunk-buffer assertions in automated tests. The existing per-panel `stable N/total` HUD may be enough; only add one small stable-control line if actual user readability still needs it. No per-cell tracing system.

### 4.4 Panel C — stable-duration / wake-candidate observation, not wake propagation

G7-A has no sleeping chunk and no dedicated neighbor wake propagation. Do not call this panel `WAKE PROPAGATION`.

Rename/reframe it as:

`STABLE DURATION / WAKE CANDIDATE`

The current geometry is insufficient because the Sand column occupies both right-side C chunks from tick 0. Redesign the geometry so one target chunk is genuinely stable first, then a real Matter frontier enters it later.

Recommended shape:

- keep the Sand source entirely in the upper-right C chunk at tick 0 (world chunk cx=1, cy=2)
- keep the lower-right C chunk (cx=1, cy=3) initially empty/stable except for a distant landing floor
- let Sand fall naturally across the y=192 chunk boundary

Required exact observation:

1. target lower-right C chunk accumulates `stable_ticks > 0` before arrival
2. when the Sand frontier enters, MATTER activity appears
3. target chunk stable counter resets to 0
4. this is recorded as a **wake candidate observation**, not as proof that a sleeping chunk was actually woken

Do not use a timer/script to activate the target.

### 4.5 Panel D

Keep it as a real slow-active-world example: combustion/thermal/pressure may appear, move, and eventually settle naturally.

Do **not** require D to remain active forever. G7-A is measuring truthful work, not forcing an always-active control.

## 5. Wake diagnostic nomenclature and sampling

Current `wake_events` terminology overstates G7-A. Also, the previous-mask array begins at zeros, so the first active diagnostic sample can be counted as a stable→active transition despite no prior observed stable sample.

Required correction:

- user-visible label: **Sampled Wake Candidates** (or equivalently explicit `Sampled Stable→Active Candidates`)
- documentation must say it is derived from diagnostic samples; it is not an exhaustive event stream and not actual G7-B wake execution
- the first sample establishes the baseline and must not increment the candidate count
- only a later sampled transition from previously observed mask 0 → nonzero counts
- reset must reset this sampling baseline

A sentinel previous-mask value or an explicit `previous_sample_valid` flag is acceptable; choose the smallest clear implementation.

## 6. Heatmap contract

Keep the existing palette and priority:

```text
GREEN  = MATTER
ORANGE = THERMAL
BLUE   = PRESSURE
RED    = REACTION
DIM    = mask == 0 / stable

priority: REACTION > PRESSURE > THERMAL > MATTER
```

The HUD must add a short note that a chunk may contain multiple activity bits while the heatmap shows only the dominant-priority color.

Do not change simulation truth or detector bits merely to make the colors prettier.

## 7. Automated validation structure

### 7.1 Fast targeted suite

Keep ordinary tests fast. At minimum run:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test -p powdergame-gpu --test activity -- --test-threads=1
cargo test -p powdergame-gpu --test thermal -- --test-threads=1
cargo test -p powdergame-gpu --test pressure -- --test-threads=1
cargo test -p powdergame-gpu --test phase -- --test-threads=1
cargo test -p powdergame-gpu --test wgsl_parse
cargo test -p powdergame-gpu --test parallel_integrity -- --test-threads=1
cargo test -p powdergame-windows
```

No performance benchmark. Full workspace suite is not required for this fixture-hardening round unless a targeted regression exposes a broader problem.

### 7.2 Actual G7-A fixture long-run validation

Add a dedicated RTX5090/DX12 long-run correctness test for the **actual `stage_activity_demo` geometry**. Do not copy/paste a second approximate fixture that can drift from the user-visible demo.

Preferred implementation: an ignored test in the Windows crate (or a very small shared staging helper) so it invokes the same staging function. Do not create a large fixture framework.

Run it explicitly in release for at least 3000 production `Simulation::tick()` calls.

This test is correctness evidence, not a benchmark.

Required long-run evidence:

- Panel B: all four B chunks are zero-activity at the validated stable period and their stable counters continue monotonically; report exact sampled values at useful checkpoints
- Panel A: no cross-panel THERMAL contamination from B/D; the sealed-control chunk(s) behave independently from the draining frontier
- Panel C: target lower-right chunk is initially stable, then a real Sand frontier crosses into it and resets stable duration with MATTER activity
- central Boundary Block isolates thermal work across the panels
- Panel D is allowed to evolve or settle naturally
- no device loss / panic / invalid state

Do not require B or C behavior by hardcoded fake state. Observe the production buffers.

Suggested explicit command shape:

```text
cargo test --release -p powdergame-windows activity_demo_long_run_3000 -- --ignored --nocapture --test-threads=1
```

Use the actual final test name in evidence.

### 7.3 Runtime smoke

```text
cargo run --release -p powdergame-windows -- --activity-demo --smoke-frames 300
```

Confirm clean exit and that Activity HUD is actually rendered during the run path.

## 8. Required launch BAT — mandatory deliverable

Create at repository root:

`run_g7_activity_demo.bat`

It is mandatory for user re-validation.

Requirements:

- Windows CMD-compatible BAT
- `cd /d "%~dp0"` first, so it always builds/runs the worktree containing the BAT and cannot accidentally launch another Powdergame checkout
- set `RUST_LOG=warn` (or equivalently suppress noisy wgpu/Naga info logs while preserving errors)
- incremental release build: `cargo build --release -p powdergame-windows`
- if build fails, print a clear error, pause, and exit nonzero
- after successful build, run the freshly built local executable directly:
  `target\release\powdergame-windows.exe --activity-demo`
- print controls before launch:
  `SPACE Play/Pause | F Fast x1/x4/x16 | N Step | R Reset | ESC Quit`
- no `cargo clean`
- no hardcoded absolute worktree path

Recommended baseline:

```bat
@echo off
setlocal
cd /d "%~dp0"
set "RUST_LOG=warn"

echo [Powdergame] Building G7 Activity Observatory...
cargo build --release -p powdergame-windows
if errorlevel 1 goto :build_error

echo.
echo Controls: SPACE Play/Pause ^| F Fast x1/x4/x16 ^| N Step ^| R Reset ^| ESC Quit
"%~dp0target\release\powdergame-windows.exe" --activity-demo
exit /b %ERRORLEVEL%

:build_error
echo.
echo [Powdergame] Build failed.
pause
exit /b 1
```

The actual committed BAT may improve messages, but must preserve these safety properties.

## 9. Freeze boundaries

Allowed in this round:

- G7 detector correctness implementation/tests
- Activity-specific simulation binding needed only to read existing material-property tables
- G7 Activity demo staging
- Activity observatory metrics/labels
- renderer TextRenderer initialization for Activity
- Activity fast-forward observation controls
- G7 evidence/status/planning docs
- `run_g7_activity_demo.bat`

Forbidden:

- G7-B actual sleep/work skipping
- indirect dispatch/active-list compaction
- choosing/tuning sleep cutoff
- changing `THERMAL_ACTIVITY_EPS` or `PRESSURE_ACTIVITY_EPS` to make screenshots look good
- changing G0-G6 movement/density/thermal/phase/combustion/pressure/rupture physics semantics
- changing material thermal properties
- new hidden wall/material
- `MATERIAL_CANDIDATES.md` modification/staging

If detector correctness requires a new **read-only** table binding, that is allowed. Physics output must remain unchanged.

## 10. Status and closure

Until the fixes and re-validation complete:

- G7 = `IN_PROGRESS`
- G7-A = `HARDENING / IN_PROGRESS`
- G7-B/C = `PLANNED`

After automated hardening passes, G7-A may return to `VALIDATION candidate`, but it must not be marked PASS/CLOSED/ACHIEVED without the user's fresh visual validation.

Historical long-run observations (~0/117/411/679/1515/3019) must be preserved as the evidence that triggered this hardening round, not rewritten as successful final validation.

## 11. Implementation checkpoint

After all targeted checks and the explicit 3000-tick fixture run pass:

```text
fix: harden G7 activity observatory fixture
```

Normal fast-forward push to `feature/m0-g7-active-sleep` only. No force push.

Final report must include:

1. starting/final SHA
2. Activity HUD root cause + fix
3. fast-forward regression root cause + restored controls
4. stale-bit root cause + regression evidence
5. thermal detector alignment + tests
6. pressure detector alignment + tests
7. Panel B exact stable metrics/checkpoints
8. central isolation method
9. Panel C stable→frontier reset tick/metrics
10. Sampled Wake Candidate first-sample fix
11. 3000-tick actual-fixture result
12. targeted test summaries
13. runtime smoke result
14. production physics diff audit
15. `run_g7_activity_demo.bat` path/content behavior
16. `MATERIAL_CANDIDATES.md` untouched confirmation
17. G7-A = VALIDATION candidate, G7 overall = IN_PROGRESS / NOT CLOSED

Stop after reporting. Do not start G7-B.