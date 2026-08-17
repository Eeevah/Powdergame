# Powdergame Handoff

이 문서는 새 사람/AI/Codex 세션이 현재 Powdergame을 이어받을 때 가장 먼저 읽는 실행용 안내다.

---

## 1. Read Order

처음 작업하는 에이전트는 코드 수정 전에 **`docs/development/QUICKSTART.md`와 `docs/planning/STATUS.md`를 먼저 읽는다.**

적대적 리뷰는 기본 종료 절차가 아니다. 사용자가 명시적으로 요청한 경우에만 `docs/adversarial-reviews/README.md`를 따라 수행·보고하며, 기존 보고서는 비차단 작업 이력으로만 취급한다. Powdergame 코드·diff·artifact·review prompt를 GPT Pro, Grok 또는 다른 외부 AI reviewer에게 보내지 않는다.

그 다음 반드시 다음 순서로 읽는다.

1. `docs/development/QUICKSTART.md`
2. `docs/vision/USER_VISION.md`
3. `docs/design-history/2026-08-15-foundation-design-session.md`
4. `docs/architecture/ARCHITECTURE.md`
5. `docs/architecture/decisions/ADR-0001-world-cell-invariants.md`
6. `docs/architecture/decisions/ADR-0002-gpu-authoritative-local-simulation.md`
7. `docs/architecture/decisions/ADR-0003-minimum-sufficient-physics.md`
8. `docs/architecture/decisions/ADR-0004-approximate-determinism-and-arbitration.md`
9. `docs/specs/SIMULATION_SPEC.md`
10. `docs/specs/MATERIAL_SPEC.md`
11. `docs/specs/REACTION_SPEC.md`
12. `docs/specs/DETERMINISM_SPEC.md`
13. `docs/development/PERFORMANCE.md`
14. `docs/development/DEVELOPMENT.md`
15. `docs/development/TESTING.md`
16. `docs/planning/ROADMAP.md`
17. `docs/planning/MILESTONES.md`
18. `docs/planning/STATUS.md`
19. `docs/evidence/G8_B_BENCHMARK_SCENARIO_GALLERY_2026-08-17.md`
20. `docs/evidence/G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md`

`README.md`, `00_USER_VISION.md`, `01_MASTER_DESIGN_REPORT.md`는 위 문서들과 맞춰 최신화되지만 세부 구현 판단은 위 authoritative 문서를 우선한다.

---

## 2. Current Goal

**M0 — First World**를 구현한다.

현재 M0 상태는 `IN_PROGRESS`다.

- G0-G7: PASS / CLOSED
- G8: Performance Evidence — IN_PROGRESS
  - G8-A Measurement Substrate: V5 OFFICIAL CAPTURE + INDEPENDENT VERIFICATION COMPLETE / VERIFIED EVIDENCE CANDIDATE; USER VISUAL VALIDATION PENDING
  - G8-B Benchmark Scenario Suite: IMPLEMENTATION CANDIDATE; Scenario 1 Sand Fall USER ACCEPTED; Scenario 2 Water Flow Harness implementation candidate / NOT USER ACCEPTED; Scenario 3–5 PENDING; overall USER ACCEPTANCE PENDING / NOT CLOSED
  - G8-B Sand Fall Experiment Harness v0: experiment source `9e1fdac` pilot PASS / `HARNESS REVIEW OUTPUT APPROVED`; later docs-only closure is separate; G8-B overall NOT CLOSED
  - G8-B Water Flow Harness v1: implemented from base `b884abc` on the shared scenario-suite line; FAST checks recorded; source seal/FULL/smoke/scratch/candidate run/verdict pending
  - G8-C Official Matrix: PENDING
- G9: Playable First World / Product Validation — PENDING

최신 세부 상태는 반드시 `docs/planning/STATUS.md`를 따른다.

M0의 목적:

> **수백만 개의 매우 싼 Local Rule을 RTX 5090에서 병렬 실행해, 작은 규칙들이 실제로 상호작용하며 살아 있는 첫 Powdergame world를 만든다는 것을 증명한다.**

중요한 현재 해석:

> **G8이 명확한 성능 blocker를 증명하지 않는 한, 다음 기본 경로는 추가 최적화가 아니라 G9 Playable First World다.**

G9에서 사용자가 직접 Matter를 놓고, 지우고, 가열하고, 구조를 만들고, 발견을 기록하고, 다음 실험을 시작할 수 있어야 M0가 닫힌다.

---

## 3. Current Canonical Recovery State

현재 local integration branch는 `integration/canonical-recovery`다.

- 검증 구현 기준: `fix/g8a-evidence-remediation-v5` at `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`
- 최신 research/Foundation Wiki: `feature/foundation-material-wiki` at `ccd0d7b00fb99128e8750ef09e5c4cce068bce09`
- Foundation base: `origin/main` at `1304b71a15df140a994737becb5f47f421758801`
- recovery merge: `e5871bdc53093700c44562826860c4d482f31ba5`
- G8-A official capture: `g8a-v5-9abec9e-20260817T032827206Z`
- independent verification: 11/11 checks passed, zero findings

G8-A의 clean source publish, official capture, independent verification은 끝났다. 같은 SHA의 user visual validation은 아직 durable approval record가 없어 pending이다.

Canonical Recovery는 local integration branch에서 구현선과 research/Material Wiki를 결합했다. 이 branch는 push되지 않았고 recovery PR도 생성되지 않았으며 `main`도 갱신되지 않았다. Draft PR #1은 open/draft 상태로 보존한다.

사용자 지시에 따라 G8-B 구현 candidate는 `feature/m0-g8b-scenario-suite`에서 시작했고 Scenario 1 Sand Fall checkpoint는 `e77d102febb1e3c497c2b669efe0140408bd99d7`로 고정되었다. Sand Fall Experiment Harness v0는 그 checkpoint 위의 `feature/g8b-experiment-harness-v0` experiment source `9e1fdac44aa14a546c7fe5ad6ceba49e71777eb5`에서 pilot automatic `PASS`와 `HARNESS REVIEW OUTPUT APPROVED`를 기록했다. 이후 docs-only closure commit은 이 experiment source provenance와 별도이며, `feature/m0-g8b-scenario-suite`가 그 closure까지 ff-only로 전진해 보존된 Harness branch와 같은 지점을 가리킨다. 다음 G8-B 작업선은 `feature/m0-g8b-scenario-suite` 하나다. Scenario 1 및 Harness 승인은 Scenario 2~5, G8-B closure, G8-C, G9, P1 identity/descriptor 등록, 새 Material, 최적화 또는 `main` 승격을 자동 승인하지 않는다.

Scenario 2 Water Flow 작업은 같은 `feature/m0-g8b-scenario-suite`의 required base `b884abcfbab8e104bdf34e2e8d19635b157c1638`에서 시작했다. Harness candidate는 구현되어 FAST fmt/check, scenarios 7/7, Windows experiment 16/16, Python 19/19, GPU reset 1/1을 기록했다. 기존 finite Water fixture, Sand fixture/pilot/artifacts, production physics는 변경하지 않았다. 아직 clean source SHA, FULL workspace checkpoint, Windows release smoke, first scratch run, one candidate run, artifact receipt와 automatic verdict가 없으므로 Water는 **NOT USER ACCEPTED**이며 G8-B는 **NOT CLOSED**다.

---

## 4. Non-negotiable Product Principles

### World fantasy

현실을 정확히 재현하는 과학 simulator가 아니다.

현실은 직관과 아이디어의 출발점이고, 가상의 Matter/가상의 법칙도 게임 안에서 이해되고 재미있으면 허용한다.

### Cell identity

```text
One Cell = Max One Matter
```

per-cell mixture/amount 모델을 기본으로 만들지 않는다.

### Performance thesis

```text
cell 하나는 극도로 싸게
×
수백만 cell GPU 병렬
=
복잡한 emergent world
```

성능은 목적 그 자체가 아니다. 절약한 예산은 더 큰 세계, 더 많은 동시 반응, 발견, Rewind와 Presentation에 다시 투자한다.

### GPU execution thesis

```text
Read Neighbors
→ cheap local rule
→ Write Self Next
```

ownership change만 Claim/Resolve.

### Minimum Sufficient Physics

실제 equation보다 gameplay에 필요한 최소 state/operation.

### Approximate determinism

bit-perfect replay보다 stable valid behavior와 성능이 우선.

### Product validation

고정 observatory가 계약대로 움직이는 것과 사용자가 자유롭게 놀고 싶어지는 것은 다른 증거다.

M0의 최종 증거는 실제 sandbox play다.

---

## 5. Current Technical Target

```text
Windows
Rust
winit
wgpu
DX12
RTX 5090 primary target
```

Reference world:

```text
2048 × 2048
initial chunk 64 × 64
60 simulation TPS target
```

---

## 6. M0 Matter / Systems

Matter:

- Boundary Block
- Stone
- Sand
- Ice
- Water
- Steam
- Smoke
- Wood
- Oil

Systems:

- Static / Powder / Liquid / Gas local movement
- Density Rank displacement
- Temperature
- Ice ↔ Water ↔ Steam
- Combustion
- Pressure
- rupture / vent
- Active/Sleep

Do not expand M0 with Electricity/Life/Civilization/etc before the current gates are proven.

G9는 신규 Matter 수를 늘리는 단계가 아니다. 현재 세트로 먼저 실제 sandbox 재미를 검증한다.

---

## 7. Implementation Sequence

완료된 순서:

1. Rust workspace
2. Windows/winit app
3. wgpu DX12 setup
4. Simulation Core separated from rendering
5. `WorldConfig`
6. dense Current/Next world buffers
7. outer BLOCK / EMPTY / Void boundary
8. Stone + Sand
9. ownership collision/arbitration
10. Water + Density Rank
11. Steam/Smoke
12. Temperature / Ice-Water-Steam
13. Combustion
14. Pressure/rupture
15. Active/Sleep — G7 Completed / Frozen
16. Measurement substrate — v5 clean source, official capture, and independent verification complete; verified evidence candidate
17. Canonical Recovery — verified runtime/evidence line + latest research/Foundation Material Wiki merged into a tested local integration branch
18. G8-B scenario-suite checkpoint `e77d102` — five official shared fixtures + exact G7 regression fixture, Windows Gallery, headless scenario selection

최근 완료와 현재 이후 순서:

19. Sand Fall Experiment Harness v0 pilot + receipt-last artifact validation — **PASS** at experiment source `9e1fdac`
20. Harness Contact Sheet/keyframe review — **APPROVED**; compact per-tile metric captions are a non-blocking future improvement
21. G8-B Windows Gallery user acceptance — Scenario 1 Sand Fall **ACCEPTED**; Scenario 2~5 **PENDING**; overall **NOT CLOSED**
22. Water Flow Harness v1 implementation candidate — unchanged finite fixture/physics; FAST checks recorded
23. clean scratch source seal → first immutable scratch run and classification
24. 필요한 source 수정이 끝나면 다시 seal → one FULL checkpoint/smoke → exactly one candidate run, then stop
25. 같은 source SHA의 G8-A user visual validation
26. Fire / Heat, Pressure Burst, Heavy Mixed World 및 G8-C는 별도 사용자 지시 전 시작 금지
27. 사용자 결정 B: G9 Playable First World 진행
28. 사용자 결정 C: M0 승인 이후에만 P1 identity/descriptor 등록 검토
29. M0 승인 후 M1 Interaction Grammar Alpha 설계 확정

Do not start with aggressive packing/f16/indirect dispatch.

Do not optimize compact active lists / indirect dispatch before G8 measurement identifies them as a real blocker.

---

## 8. Required G8 Benchmarks

- Sand Fall — **USER ACCEPTED**; complete settling and all chunks sleeping are success; do not retune for perpetual activity
- Water Flow — **HARNESS IMPLEMENTATION CANDIDATE / NOT YET USER ACCEPTED; RUNS PENDING**
- Fire / Heat — **PENDING / NOT YET USER ACCEPTED**
- Pressure Burst — **PENDING / NOT YET USER ACCEPTED**
- Heavy Mixed World — **PENDING / NOT YET USER ACCEPTED**

이 다섯 fixture는 `powdergame-scenarios`의 `ScenarioId`와 `reset_and_stage_scenario`를 Windows Gallery와 headless benchmark가 공유한다. `active-sleep-g7`은 exact 256×256×64 G7 회귀 fixture이며 official G8-B workload가 아니다.

Windows inspection:

```bat
run_g8_benchmark_gallery.bat
```

Gallery는 paused 상태로 시작한다. `1-6` scenario, `SPACE` play/pause, `N` one tick, `F` x1/x4/x16, `R` pristine reset, `ESC` quit을 사용한다.

Headless selection:

```bat
cargo run --release -p powdergame-benchmark -- --scenario sand-fall
```

Gallery rendering, HUD, wall-clock TPS, bounded activity census는 inspection diagnostics이며 official timed benchmark에 포함하지 않는다. headless harness만 각 prewarm/trial/overhead window 전에 shared reset/stage를 수행한다.

Record subsystem cost separately during G8-C, not from Gallery diagnostics.

Include rendering and simulation+rendering coexistence evidence; calibration-only headless TPS is not the entire product performance result.

Do not set arbitrary M0 maximum-TPS pass/fail before the official matrix exists.

Current boundary: fixture/staging/selection implementation candidate exists and Scenario 1 is accepted. Scenario 2 Water Flow now has an unsealed Harness candidate, but no scratch/candidate evidence or user acceptance; Scenario 3~5 remain pending and **G8-B is NOT CLOSED**. Do not retune accepted Sand Fall or the untuned Water fixture before its first run. No physics/Material/G9/optimization addition belongs to this candidate.

### Sand Fall Experiment Harness v0

Recorded pilot entry point (do not rerun for this closure):

```bat
run_experiment.bat sand-fall
```

The shared coordinator dispatches the immutable Sand v0 contract and the new Water v1 contract, writing every unique run directly below `C:\Users\mdkap\source\Powdergame-artifacts`. It preserves raw stdout/stderr, telemetry samples/events, worker analysis/frame manifests, semantic RGBA frames, derived full/crop PNGs, reports, contact sheet, inert review prompt, review packet, and hashes. `EXPERIMENT_RECEIPT.json` is written last with no filesystem write afterward; no receipt means incomplete, and a failed Run ID is never repaired or reused. Generated artifacts never enter Git.

The lifecycle records tick 0, tick 1, peak active, first sleeping chunk, late settling, first observed all sleep in a confirmed three-sample streak, 180 post-sleep ticks, and programmatic `R`-equivalent exact reset. Simulation tick and diagnostic sample sequence remain distinct. Automatic `PASS` requires all seven hard Sand Fall predicates, but does not close G8-B or establish Water Flow/G8-C evidence.

Current Harness state: validated pilot **PASS** at experiment source `9e1fdac44aa14a546c7fe5ad6ceba49e71777eb5`; Harness review output **APPROVED**. The later docs-only closure commit records this result but is not the experiment source. G8-B remains **NOT CLOSED** because Water Flow, Fire / Heat, Pressure Burst, and Heavy Mixed World are pending. Follow `docs/evidence/G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md` for the authoritative run record.

### Water Flow Experiment Harness v1 candidate

```bat
run_experiment.bat water-flow --mode scratch
run_experiment.bat water-flow
```

Water uses the same shared pristine staging and production physics. The 256×256×64 tick-0 fixture remains Water 15,244 / Oil 2,240 / Stone 6,888 / Boundary Block 1,020 / Empty 40,144, with a diagnostic destination mask of 6,216 tick-0 EMPTY cells inside `[18,238) × [200,230)`. Neither the mask nor the analyzer stages a result.

Water manifest/telemetry/analysis/report/receipt use v1 schemas; the frame manifest remains shared v0. Its nine predicates are movement, cross-chunk flow, destination arrival, Water conservation, invalid-ID integrity, finite-field integrity, stable bulk before max, post-settle stability, and exact reset. `scratch` and default `candidate` have distinct Run IDs and the same create-new/no-overwrite/receipt-last contract. A plateau can yield `NEEDS_HUMAN_REVIEW`; it is not silently promoted to finite-fixture all-sleep `PASS`.

Current Water state: implementation and FAST checks exist, but candidate source SHA, FULL checkpoint, Windows release smoke, first scratch run, candidate run, Contact Sheet/packet/receipt hashes and automatic verdict are **PENDING**. Do not alter the fixture based on expectation before preserving and classifying the first scratch evidence. Water remains **NOT USER ACCEPTED**. Follow `docs/evidence/G8_B_WATER_FLOW_HARNESS_CANDIDATE_2026-08-17.md`.

---

## 9. Required G9 Product Slice

### Sandbox interaction

- Matter selection
- draw / erase
- brush size
- Heat or Temperature tool
- pause / play / step / speed / reset
- pan / zoom
- preset load

### Discovery MVP

Record meaningful first observations from actual simulation state/events.

- phase change
- combustion
- pressure generation
- rupture / vent
- transformation

Hide exact threshold and remaining discovery count.

### Presentation

Simulation Truth and Presentation remain separated.

At minimum, the player must be able to read key combustion, smoke/heat and rupture/vent events more clearly than raw diagnostic colors alone. Presentation must not invent gameplay results.

### User approval

The strongest success signal is not “the expected boiler ruptured.”

It is:

> **the user voluntarily starts another experiment.**

---

## 10. Important Deferred Item

### Interaction Lab

Future developer tool that takes **already-defined** Material/Rules and runs the actual GPU Simulation against existing Matter/environment combinations to find unexpected interactions/regressions.

It is not a Material generator.

It is currently deferred because the game itself is more important.

Reconsider after M1 when Material count and regression surface make manual validation a real bottleneck.

---

## 11. What Not to Reintroduce

Do not silently revert to older research assumptions such as:

- Browser-first product path
- macOS parity requirement
- broad GPU fallback complexity
- ONI-style multi-Matter/mass Cell
- giant WorldPrimitive object per Cell
- strict global energy bookkeeping
- bit-perfect seeded replay requirement
- per-cell universal progress fields for every future phenomenon
- Gas/Liquid that stay Active forever simply because they exist
- full-world heavy Rule resolve passes for ordinary interactions
- optimization as an automatic phase regardless of benchmark evidence
- dozens of new Matter before the current sandbox is fun

If changing one of these, create explicit evidence and update ADR/SPEC/Design History.

---

## 12. Completion Authority

AI/Codex/CI may implement and gather evidence.

M0 may move to `VALIDATION` when G8 and G9 evidence is ready.

> **Only the user can approve final `ACHIEVED`.**
