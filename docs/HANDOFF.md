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
16. `docs/development/WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md`
17. `docs/planning/ROADMAP.md`
18. `docs/planning/MILESTONES.md`
19. `docs/planning/STATUS.md`
20. `docs/evidence/G8_B_BENCHMARK_SCENARIO_GALLERY_2026-08-17.md`
21. `docs/evidence/G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md`
22. `docs/evidence/G8_B_WATER_FLOW_HARNESS_CANDIDATE_2026-08-17.md`
23. `docs/evidence/G8_B_FIRE_HEAT_HARNESS_CANDIDATE_2026-08-17.md`
24. `docs/evidence/G8_B_PRESSURE_BURST_HARNESS_CANDIDATE_2026-08-18.md`
25. `docs/evidence/G8_B_HEAVY_MIXED_WORLD_HARNESS_CANDIDATE_2026-08-19.md`

`README.md`, `00_USER_VISION.md`, `01_MASTER_DESIGN_REPORT.md`는 위 문서들과 맞춰 최신화되지만 세부 구현 판단은 위 authoritative 문서를 우선한다.

---

## 2. Current Goal

**M0 — First World**를 구현한다.

현재 M0 상태는 `IN_PROGRESS`다.

- G0-G7: PASS / CLOSED
- G8: Performance Evidence — IN_PROGRESS
  - G8-A Measurement Substrate: V5 OFFICIAL CAPTURE + INDEPENDENT VERIFICATION COMPLETE / VERIFIED EVIDENCE CANDIDATE; USER VISUAL VALIDATION PENDING
  - G8-B Benchmark Scenario Suite: **CLOSED / FROZEN**; all five official scenarios USER ACCEPTED; Water, Pressure, and Heavy automatic NEEDS_HUMAN_REVIEW unchanged / known follow-up; Fire automatic PASS unchanged; Cell Inspector v0 tested source `3c342d2` USER ACCEPTED WITH KNOWN FOLLOW-UP
  - G8-B Sand Fall Experiment Harness v0: experiment source `9e1fdac` pilot PASS / `HARNESS REVIEW OUTPUT APPROVED`; later docs-only closure is separate
  - G8-B Water Flow: first candidate at `d12edbf` preserved as `NEEDS_HUMAN_REVIEW` / human `FIX REQUIRED`; source `5af031f` v2 remediation run automatic `NEEDS_HUMAN_REVIEW` / human `ACCEPTED WITH KNOWN FOLLOW-UP`
  - G8-B Fire / Heat: USER ACCEPTED; unchanged fixture; source `1635fdb`; one sealed candidate automatic PASS; independent verification found zero mismatch; no physics change or candidate rerun
  - G8-B Pressure Burst: source `43e19d0`; immutable run `g8b-pressure-burst-v0-20260818T101046792957Z-17158748`; automatic NEEDS_HUMAN_REVIEW unchanged; human USER ACCEPTED WITH KNOWN FOLLOW-UP; no production-physics defect evidence
  - G8-B Heavy Mixed World: source `07260fff`; immutable run `g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4`; automatic NEEDS_HUMAN_REVIEW unchanged solely for review-only `broad_terminal_tail`; 14/14 hard predicates PASS; candidate blocker false; human USER ACCEPTED WITH KNOWN FOLLOW-UP; no physics defect, fixture remediation, or candidate rerun required
  - G8-C Official Matrix: **NEXT / AUTHORIZED**; not yet executed and G8 overall remains IN_PROGRESS
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

사용자 지시에 따라 G8-B 구현 candidate는 `feature/m0-g8b-scenario-suite`에서 시작했고 Scenario 1 Sand Fall checkpoint는 `e77d102febb1e3c497c2b669efe0140408bd99d7`로 고정되었다. Sand Fall Experiment Harness v0는 그 checkpoint 위의 `feature/g8b-experiment-harness-v0` experiment source `9e1fdac44aa14a546c7fe5ad6ceba49e71777eb5`에서 pilot automatic `PASS`와 `HARNESS REVIEW OUTPUT APPROVED`를 기록했다. 이후 docs-only closure commit은 이 experiment source provenance와 별도이며, `feature/m0-g8b-scenario-suite`가 그 closure까지 ff-only로 전진해 보존된 Harness branch와 같은 지점을 가리킨다. 다섯 official scenario와 Cell Inspector v0의 사용자 승인 뒤 G8-B는 **CLOSED / FROZEN**으로 봉합되었다. 이는 G8-A user visual durable closure, G8-C 결과, G8 전체 closure, G9, P1 identity/descriptor 등록, 새 Material, 최적화 또는 `main` 승격을 자동 승인하지 않는다.

Scenario 2 Water Flow의 first candidate `g8b-water-flow-v0-20260817T100732645294Z-f7ee7959`는 source `d12edbfbcc0fb3fc2ef599cd06b3c46a2293d268`에서 automatic `NEEDS_HUMAN_REVIEW`를 기록했고, human review는 `FIX REQUIRED — fixture_representativeness_issue`로 판정했다. 기존 run/artifact는 immutable/superseded다. Remediation은 같은 branch에서 좌우 외벽 시작 높이만 `y=90 → 14`로 연장하고 zero-leakage hard predicate/terminal active-cell 분류를 추가했다. Source `5af031f1a04af866127616d4f1b0faa6c85e4d8e`의 run `g8b-water-flow-v0-20260817T110906547252Z-8b808e66`는 automatic `NEEDS_HUMAN_REVIEW`를 유지한 채 human `ACCEPTED WITH KNOWN FOLLOW-UP`로 승인되었다. Water/Oil, internal channel, production physics, Sand fixture/pilot/artifacts, all-sleep/plateau policy는 변경하지 않는다. Fire / Heat도 별도 user acceptance를 완료했다. Pressure Burst는 source `43e19d0f3b43aa0c15bf31e98f6401ba5f885170`의 run `g8b-pressure-burst-v0-20260818T101046792957Z-17158748`에서 automatic `NEEDS_HUMAN_REVIEW`를 유지한 채 human `USER ACCEPTED WITH KNOWN FOLLOW-UP`로 승인되었다. Cell Inspector v0는 tested source `3c342d25099683df53e303d1920cebe1f6578b74`에서 **USER ACCEPTED WITH KNOWN FOLLOW-UP**다. Heavy Mixed World는 clean source `07260fffab22e5b4513eb168f0baac36e374ab94`의 candidate run `g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4`에서 14/14 hard predicate와 `candidate_blocker=false`를 보존한 채 human **USER ACCEPTED WITH KNOWN FOLLOW-UP**로 승인되었다. Automatic `NEEDS_HUMAN_REVIEW`는 `broad_terminal_tail` 때문에 변경하지 않는다. G8-B는 **CLOSED / FROZEN**이며 G8-C는 **NEXT / AUTHORIZED**다.

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
21. G8-B Windows Gallery user acceptance — all five official scenarios **ACCEPTED**; Water, Pressure, and Heavy automatic verdicts unchanged / known follow-up; Fire automatic `PASS` unchanged; overall **CLOSED / FROZEN**
22. Water Flow first candidate — immutable/superseded; automatic `NEEDS_HUMAN_REVIEW`; human `FIX REQUIRED — fixture_representativeness_issue`
23. Water fixture-only remediation — source `5af031f`, side-wall top `y=90 → 14`, zero leakage, production physics unchanged; human **ACCEPTED WITH KNOWN FOLLOW-UP**
24. Fire / Heat unchanged-fixture Harness candidate — source `1635fdb`; FAST/clippy, one FULL, one Gallery smoke, one candidate, and independent verification complete; automatic `PASS`; **USER ACCEPTED** with no physics change or candidate rerun
25. Cell Inspector v0 — tested source `3c342d25099683df53e303d1920cebe1f6578b74` **USER ACCEPTED WITH KNOWN FOLLOW-UP**; prior silent-pending docs closure `6a521c07c23d9817f6abb9d3e1eab47758b5d5c4`
26. 같은 source SHA의 G8-A user visual validation
27. Heavy Mixed World — source `07260fff`, run `g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4`, automatic `NEEDS_HUMAN_REVIEW` unchanged solely for `broad_terminal_tail`, 14/14 hard pass, `candidate_blocker=false`; human **USER ACCEPTED WITH KNOWN FOLLOW-UP**
28. G8-B **CLOSED / FROZEN**; G8-C Official Matrix **IMPLEMENTATION CANDIDATE / CAPTURE PENDING**
29. G8-C non-evidence pilot 한 번, clean source seal, official matrix 한 번, independent verification/package
30. 사용자 결정 B: G9 Playable First World 진행
31. 사용자 결정 C: M0 승인 이후에만 P1 identity/descriptor 등록 검토
32. M0 승인 후 M1 Interaction Grammar Alpha 설계 확정

Do not start with aggressive packing/f16/indirect dispatch.

Do not optimize compact active lists / indirect dispatch before G8 measurement identifies them as a real blocker.

---

## 8. Required G8 Benchmarks

- Sand Fall — **USER ACCEPTED**; complete settling and all chunks sleeping are success; do not retune for perpetual activity
- Water Flow — **USER ACCEPTED WITH KNOWN FOLLOW-UP; AUTOMATIC NEEDS_HUMAN_REVIEW UNCHANGED**
- Fire / Heat — **USER ACCEPTED; SEALED CANDIDATE AUTOMATIC PASS UNCHANGED**
- Pressure Burst — **USER ACCEPTED WITH KNOWN FOLLOW-UP; AUTOMATIC NEEDS_HUMAN_REVIEW UNCHANGED**
- Heavy Mixed World — **USER ACCEPTED WITH KNOWN FOLLOW-UP; AUTOMATIC NEEDS_HUMAN_REVIEW UNCHANGED; 14-OF-14 HARD PASS**

이 다섯 fixture는 `powdergame-scenarios`의 `ScenarioId`와 `reset_and_stage_scenario`를 Windows Gallery와 headless benchmark가 공유한다. `active-sleep-g7`은 exact 256×256×64 G7 회귀 fixture이며 official G8-B workload가 아니다.

Windows inspection:

```bat
run_powdergame.bat gallery
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

G8-C implementation candidate는 `pwsh -NoProfile -File tools/dev.ps1 g8c-matrix pilot|official`로만 실행한다. Mode A는 existing ordinary-context sustained throughput, Mode B는 separate profiling-context 17-pass/6-group breakdown, Mode C는 HUD/Inspector/text/readback을 끈 1600×900 Fifo coexistence, Mode D는 별도 timestamp-enabled render profiling이다. Outer schemas는 matrix/headless/coexistence/render-profile v1이며 G8-A/G8-B inner output을 바꾸지 않는다. Official Run은 exact source bytes + canonical Git archive + 두 역할별 frozen binary + raw A/B/C/D + reports + hashes + receipt-last를 한 matrix root에 묶고, sibling ZIP/package hash/independent verification result 하나만 만든다. Screenshot/contact sheet/video와 scenario별 bundle은 만들지 않는다.

현재는 source/CLI/reset/accounting/percentile/receipt/package/verifier targeted checks가 통과한 **implementation candidate**일 뿐이다. Pilot, clean source seal, isolated build, official capture와 independent verification 전에는 수치나 recommendation을 현재 성능 사실로 기록하지 않는다. G8-A same-SHA user visual durable closure는 별도 pending이며 G9 또는 optimization은 자동 시작하지 않는다.

Current boundary: all five official scenarios and Cell Inspector v0 are user accepted, so G8-B is **CLOSED / FROZEN**. Water keeps its immutable automatic `NEEDS_HUMAN_REVIEW` and known M0 liquid free-surface follow-up. Scenario 3 Fire / Heat keeps its sealed unchanged-fixture automatic-`PASS` candidate and immutable artifacts; no physics change or rerun was required. Scenario 4 Pressure Burst preserves the rejected/confounded first candidate and dirty-source causal scratch, while the accepted immutable clean-source candidate is run `g8b-pressure-burst-v0-20260818T101046792957Z-17158748` at source `43e19d0f3b43aa0c15bf31e98f6401ba5f885170`. Its automatic `NEEDS_HUMAN_REVIEW` is unchanged; human verdict is `USER ACCEPTED WITH KNOWN FOLLOW-UP`. `pressure_opening_precedes_combustion` passed, opening-time combustion/flame/fuel progress was zero, adjacent Pressure exceeded the rupture threshold, terminal Pressure trended downward, invalid/non-finite was `0 / 0`, and reset was exact. Known follow-up is top-seam-only opening, a small persistent plume, broad terminal Pressure activity, and G8-C workload-cost measurement; no production-physics defect is established. Cell Inspector v0 is **USER ACCEPTED WITH KNOWN FOLLOW-UP** at tested source `3c342d25099683df53e303d1920cebe1f6578b74`; its slight bounded hover delay is non-blocking. Scenario 5 Heavy Mixed World has an immutable clean-source candidate at source/run `07260fffab22e5b4513eb168f0baac36e374ab94` / `g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4`. Its 14 hard predicates pass and `candidate_blocker=false`; automatic `NEEDS_HUMAN_REVIEW` remains solely for review-only `broad_terminal_tail`, while human verdict is **USER ACCEPTED WITH KNOWN FOLLOW-UP**. Terminal Thermal activity is large but decreases monotonically through the terminal window; Pressure and Reaction have ended, so this is not a correctness failure. G8-C is **NEXT / AUTHORIZED** to measure actual workload cost. Do not retune accepted scenarios or change production physics. No physics/Material/G9/optimization addition belongs to this closure.

### Sand Fall Experiment Harness v0

Recorded pilot entry point (do not rerun for this closure):

```bat
run_experiment.bat sand-fall
```

The shared coordinator dispatches immutable Sand, Water, Fire / Heat, Pressure Burst, and Heavy Mixed World as distinct contracts, writing every unique run directly below `C:\Users\mdkap\source\Powdergame-artifacts`. Already generated evidence remains immutable. The coordinator preserves raw stdout/stderr, telemetry samples/events, worker analysis/frame manifests, semantic RGBA frames, derived full/crop PNGs, reports, contact sheet, inert review prompt, review packet, and hashes. `EXPERIMENT_RECEIPT.json` is the final write inside the Run directory; no receipt means incomplete, and a failed Run ID is never repaired or reused. Candidate-only Audit Bundle/sidecar delivery occurs afterward as siblings outside the Run directory. Generated artifacts never enter Git.

The lifecycle records tick 0, tick 1, peak active, first sleeping chunk, late settling, first observed all sleep in a confirmed three-sample streak, 180 post-sleep ticks, and programmatic `R`-equivalent exact reset. Simulation tick and diagnostic sample sequence remain distinct. Automatic `PASS` requires all seven hard Sand Fall predicates, but did not close G8-B by itself and does not establish Water Flow/G8-C evidence.

Current Harness state: validated pilot **PASS** at experiment source `9e1fdac44aa14a546c7fe5ad6ceba49e71777eb5`; Harness review output **APPROVED**. The later docs-only closure commit records this result but is not the experiment source. This individual Harness result did not close G8-B by itself; the suite later closed only after all five separate user decisions. Follow `docs/evidence/G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md` for the authoritative run record.

### Water Flow Experiment Harness v2 accepted remediation evidence

```bat
run_experiment.bat water-flow
```

Water uses the same shared pristine staging and production physics. The remediated 256×256×64 tick-0 fixture is Water 15,244 / Oil 2,240 / Stone 8,104 / Boundary Block 1,020 / Empty 38,928, with the unchanged 6,216-cell destination mask inside `[18,238) × [200,230)`. Only side walls changed from `y=[90,238)` to `y=[14,238)`; internal geometry and liquids are unchanged.

Water telemetry/analysis/report/receipt use v2; manifest remains v1 and frames remain shared v0. Its ten predicates are the prior nine plus `water_outside_outer_basin_cells`, which passes only if Water outside `[18,238) × [14,230)` remains zero in every non-reset sample. Unique/create-new/no-overwrite/receipt-last rules are unchanged. A plateau can yield `NEEDS_HUMAN_REVIEW`; it is not silently promoted to finite-fixture all-sleep `PASS`.

Current Water state: first candidate and human finding are preserved. Remediation FAST, single FULL checkpoint, clippy/diff, and 60-frame RTX 5090/DX12 Gallery smoke passed. Source `5af031f1a04af866127616d4f1b0faa6c85e4d8e` run `g8b-water-flow-v0-20260817T110906547252Z-8b808e66` recorded outer-basin Water max/final `0 / 0`, conservation/movement/cross-chunk/destination/integrity/reset pass, and final active cells `64` = Water/EMPTY `51`, Water/Oil `1`, other `12`. Automatic `NEEDS_HUMAN_REVIEW` is immutable; human verdict is `ACCEPTED WITH KNOWN FOLLOW-UP`. Follow `docs/evidence/G8_B_WATER_FLOW_HARNESS_CANDIDATE_2026-08-17.md`.

### Fire / Heat Experiment Harness candidate

```bat
run_experiment.bat fire-heat
```

Fire / Heat reuses the coordinator/provenance/no-overwrite/receipt-last structure but has a distinct analyzer and verdict contract. It stages the unchanged finite fixture, advances only production `Simulation::tick()`, and distinguishes authored tick-0 flags from genuine post-tick Wood/Oil combustion. Whole-world all-sleep is not required. Three reaction-zero diagnostic samples begin a separate 180-tick post-reaction Thermal-tail window; a remaining tail is not itself failure. Source `1635fdb9f562192123c92846e137b125c684ede9` completed the single workspace checkpoint, one Gallery smoke, and exactly one candidate run `g8b-fire-heat-v0-20260817T133938546075Z-0e6aa901`. The automatic verdict is `PASS`; independent raw/artifact verification found zero mismatch. The user accepted the genuine combustion, Smoke decay, phase work, finite fuel use, Reaction zero/confirmation at `11,448 / 11,464`, post-Reaction completion at `11,644` with restart `0`, slightly decreasing Thermal tail, invalid/non-finite `0 / 0`, and exact reset. No production physics change or candidate rerun was required. This individual acceptance did not close G8-B by itself; the suite later closed after Heavy Mixed approval. Follow `docs/evidence/G8_B_FIRE_HEAT_HARNESS_CANDIDATE_2026-08-17.md`.

### Cell Inspector v0 user acceptance

Immutable tested source `3c342d25099683df53e303d1920cebe1f6578b74` is **USER ACCEPTED WITH KNOWN FOLLOW-UP**. The prior silent-pending docs closure `6a521c07c23d9817f6abb9d3e1eab47758b5d5c4` and this docs-only acceptance closure remain distinct from tested source provenance. Compact hover shows the canonical Material name; `I` toggles detail with Material `Name (ID)`, Temperature, Pressure, activity, Chunk state, sample simulation tick / diagnostic sequence / freshness.

Renderer and physical-pixel cursor picking share one CPU-authoritative `WorldViewport`. Material, Temperature, Pressure, flags, Cell activity, and Chunk state are copied as a single 24-byte batch with at most one pending request at no more than 10 Hz; there is no mouse-driven full-world readback. Cell/Chunk, simulation tick, diagnostic sequence, selection generation, and world epoch bind sample identity. Reset, scenario switch, staging failure, and shutdown invalidate stale pending/completed samples.

The user accepted compact Material hover, the `I` detail panel, natural silent pending with no stale previous-Cell values or `Collecting...` placeholder, practical Material/Field comprehension, and normal reset/scenario-switch behavior. The known follow-up is a slight delay after hovering a new Cell, bounded by the unchanged no-more-than-10-Hz / 100-ms readback design. It is non-blocking for Heavy Mixed and G8-C; do not increase sampling frequency or change architecture until actual UX and readback costs are measured together. Heavy Mixed World is now also user accepted with a known follow-up, and G8-B is **CLOSED / FROZEN**. The source of truth for the UI contract is `docs/vision/UI_DIRECTION.md`.

### Heavy Mixed World Harness candidate

Clean source `07260fffab22e5b4513eb168f0baac36e374ab94` produced the immutable candidate run `g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4`. Current coordinator revalidation passed for `2,504` samples, `319` events, `14` folded frames, `58` Run hash entries, the source-input seal, frozen binary, report/receipt summary, and candidate Audit Bundle. All 14 hard predicates pass and `candidate_blocker=false`. Automatic `NEEDS_HUMAN_REVIEW` remains unchanged solely for `broad_terminal_tail`; human verdict is **USER ACCEPTED WITH KNOWN FOLLOW-UP**. There is no production-physics defect evidence, and fixture remediation and candidate rerun are not required.

First movement/density/phase ticks are `1 / 8 / 1`; first dynamic combustion/new Smoke/Pressure ticks are `1 / 1 / 8`; all four intended subsystems first overlap at tick `8`. Peak concurrency is `4 @ 8`, peak active cells are `40,301 @ 3,528`, and the longest `>=3` subsystem sampled window is `1,986` samples over ticks `1..15,872`. Inventory unexplained, invalid, non-finite, and wake-anomaly occurrences are all `0`; terminal Temperature and Pressure maxima decline over the last 64 samples; exact reset is true. Terminal active cells are `25,833` (`39.418030%`), which triggers the only review flag.

The raw `first_vent_tick=3,920`, `first_vent_observed`, and `first-vent` badge are legacy raw-kind vocabulary for first exterior Steam above relief. They are not opening-gated and are not proof of causal venting. The first complete relief lane follows at tick `3,960`. Preserve this caveat in human review and docs; Heavy acceptance does not use vent as a hard predicate. Binary / Review Packet / Receipt / Audit Bundle SHA-256 are `9b84db005942cf60ae9ef133521e9297413d49c93d72e7ae64133e29622f7583` / `ca2fe05a1497f8417dd732de23c1a260569adfcccd6bd0f16bad180f2a8d1144` / `2abebdef7f9174e63abfd9c67ce4a48d24b48edde4e6c29fab49022e36a2dbd1` / `bc44c66bd52b5d856decb2317389a455a56ac8ae1f8d67b1bfeb5446cfb5731b`. Exact artifact sizes, provenance, optional-milestone semantics, and acceptance boundary are fixed in `docs/evidence/G8_B_HEAVY_MIXED_WORLD_HARNESS_CANDIDATE_2026-08-19.md`. G8-B is **CLOSED / FROZEN**; G8-C is **NEXT / AUTHORIZED** but not yet executed.

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
