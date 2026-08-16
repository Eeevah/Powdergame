# Powdergame Handoff

이 문서는 새 사람/AI/Codex 세션이 현재 Powdergame을 이어받을 때 가장 먼저 읽는 실행용 안내다.

---

## 1. Read Order

처음 작업하는 에이전트는 코드 수정 전에 **`docs/development/QUICKSTART.md`와 `docs/planning/STATUS.md`를 먼저 읽는다.**

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

`README.md`, `00_USER_VISION.md`, `01_MASTER_DESIGN_REPORT.md`는 위 문서들과 맞춰 최신화되지만 세부 구현 판단은 위 authoritative 문서를 우선한다.

---

## 2. Current Goal

**M0 — First World**를 구현한다.

현재 M0 상태는 `IN_PROGRESS`다.

- G0-G7: PASS / CLOSED
- G8: Performance Evidence — IN_PROGRESS
  - G8-A Measurement Substrate: COMPLETE
  - G8-B Benchmark Scenario Suite: NEXT
  - G8-C Official Matrix: PENDING
- G9: Playable First World / Product Validation — PENDING

최신 세부 상태는 반드시 `docs/planning/STATUS.md`를 따른다.

M0의 목적:

> **수백만 개의 매우 싼 Local Rule을 RTX 5090에서 병렬 실행해, 작은 규칙들이 실제로 상호작용하며 살아 있는 첫 Powdergame world를 만든다는 것을 증명한다.**

중요한 현재 해석:

> **G8이 명확한 성능 blocker를 증명하지 않는 한, 다음 기본 경로는 추가 최적화가 아니라 G9 Playable First World다.**

G9에서 사용자가 직접 Matter를 놓고, 지우고, 가열하고, 구조를 만들고, 발견을 기록하고, 다음 실험을 시작할 수 있어야 M0가 닫힌다.

---

## 3. Immediate Repository Prerequisite

현재 `main`의 연구/문서 진행선과 `feature/m0-g8-performance-evidence`의 구현/증거 진행선이 분기되어 있다.

다음 제품 작업 전에 깨끗한 integration branch에서 결합하고, 검증 후 `main`을 하나의 buildable canonical line으로 만든다.

- dirty worktree의 사용자 변경을 자동 reset/stash/discard하지 않음
- 구현 상태는 G8 진행선의 검증된 코드/증거를 기준으로 판단
- 최신 연구 corpus는 `main`의 문서를 보존
- README / Vision / Roadmap / Milestones / Status / Handoff 일치
- canonical SHA 확정 후 `personal-infra-wiki`에 Powdergame 등록

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
16. Measurement substrate — G8-A Complete

현재 이후 순서:

17. G8-B — five official benchmark fixtures
18. G8-C — official matrix, render+simulation measurement, bottleneck conclusion
19. G9-A — sandbox Matter selection / draw / erase / Heat / camera / time controls
20. G9-B — user-created open emergence validation
21. G9-C — phenomenon-level Discovery MVP
22. G9-D — minimum honest modern Presentation feedback
23. G9-E — direct user play approval
24. M0 승인 후 M1 Interaction Grammar Alpha 설계 확정

Do not start with aggressive packing/f16/indirect dispatch.

Do not optimize compact active lists / indirect dispatch before G8 measurement identifies them as a real blocker.

---

## 8. Required G8 Benchmarks

- Sand Fall
- Water Flow
- Fire / Heat
- Pressure Burst
- Heavy Mixed World

Record subsystem cost separately.

Include rendering and simulation+rendering coexistence evidence; calibration-only headless TPS is not the entire product performance result.

Do not set arbitrary M0 maximum-TPS pass/fail before the official matrix exists.

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
