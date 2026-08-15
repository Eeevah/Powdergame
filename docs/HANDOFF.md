# Powdergame Handoff

이 문서는 새 사람/AI/Codex 세션이 현재 Powdergame을 이어받을 때 가장 먼저 읽는 실행용 안내다.

---

## 1. Read Order

반드시 다음 순서로 읽는다.

1. `docs/vision/USER_VISION.md`
2. `docs/design-history/2026-08-15-foundation-design-session.md`
3. `docs/architecture/ARCHITECTURE.md`
4. `docs/architecture/decisions/ADR-0001-world-cell-invariants.md`
5. `docs/architecture/decisions/ADR-0002-gpu-authoritative-local-simulation.md`
6. `docs/architecture/decisions/ADR-0003-minimum-sufficient-physics.md`
7. `docs/architecture/decisions/ADR-0004-approximate-determinism-and-arbitration.md`
8. `docs/specs/SIMULATION_SPEC.md`
9. `docs/specs/MATERIAL_SPEC.md`
10. `docs/specs/REACTION_SPEC.md`
11. `docs/specs/DETERMINISM_SPEC.md`
12. `docs/development/PERFORMANCE.md`
13. `docs/development/DEVELOPMENT.md`
14. `docs/development/TESTING.md`
15. `docs/planning/MILESTONES.md`
16. `docs/planning/STATUS.md`

`README.md`, `00_USER_VISION.md`, `01_MASTER_DESIGN_REPORT.md`는 위 문서들과 맞춰 최신화되지만 세부 구현 판단은 위 authoritative 문서를 우선한다.

---

## 2. Current Goal

**M0 — First World**를 구현한다.

현재 M0 상태는 `PLANNED`.

M0의 목적:

> **수백만 개의 매우 싼 Local Rule을 RTX 5090에서 병렬 실행해, 작은 규칙들이 실제로 상호작용하며 살아 있는 첫 Powdergame world를 만든다는 것을 증명한다.**

---

## 3. Non-negotiable Product Principles

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

---

## 4. Current Technical Target

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

## 5. M0 Matter / Systems

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

---

## 6. Suggested First Coding Sequence

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
15. Active/Sleep
16. benchmark harness

Do not start with aggressive packing/f16/indirect dispatch.

---

## 7. Required Benchmarks

- Sand Fall
- Water Flow
- Fire / Heat
- Pressure Burst
- Heavy Mixed World

Record subsystem cost separately.

Do not set arbitrary M0 numeric performance pass/fail before baseline exists.

---

## 8. Important Deferred Item

### Interaction Lab

Future developer tool that takes **already-defined** Material/Rules and runs the actual GPU Simulation against existing Matter/environment combinations to find unexpected interactions/regressions.

It is not a Material generator.

It is currently deferred because the game itself is more important.

---

## 9. What Not to Reintroduce

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

If changing one of these, create explicit evidence and update ADR/SPEC/Design History.

---

## 10. Completion Authority

AI/Codex/CI may implement and gather evidence.

M0 may move to `VALIDATION` when evidence is ready.

> **Only the user can approve final `ACHIEVED`.**
