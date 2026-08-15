# Powdergame Roadmap

이 문서는 장기 방향을 기록한다. **약속된 일정표가 아니다.** 실제 완료 기준은 `MILESTONES.md`의 Evidence Gate를 따른다.

---

## 현재 위치

Foundation Design은 충분히 구체화되었다.

현재 다음 단계는:

> **M0 — First World 구현과 실제 RTX 5090 baseline 확보**

이다.

---

## Phase 0 — First World

목표:

- finite 2048×2048 reference world
- GPU-authoritative simulation
- Static / Powder / Liquid / Gas
- Density Rank local displacement
- Temperature / Ice-Water-Steam
- Combustion
- Pressure / rupture / vent
- Active/Sleep
- benchmark harness
- user play validation

상세 Gate는 `MILESTONES.md`의 M0를 따른다.

---

## Phase 1 — Baseline-driven Optimization

M0 결과를 보고 실제 병목에 따라 결정한다.

후보:

- Active Chunk refinement
- field-specific Active Set
- stable frontier optimization
- active compaction / indirect dispatch
- shared-memory tile
- descriptor packing
- chunk size benchmark
- f16 experiment if justified
- Rewind storage optimization

이 목록은 구현 순서 약속이 아니다. **병목인 항목만 우선한다.**

---

## Phase 2 — Richer Matter Grammar

M0의 공통 Rule 구조가 검증된 후 Material/Reaction 다양성을 늘린다.

후보:

- Acid / corrosion
- Lava / Glass / Metal transformations
- Salt / solution-like spatial interactions
- Seed / Plant
- fictional Matter families
- slow rules such as oxidation/weathering
- richer phase transition yield

현실 화학 DB를 만드는 것이 아니라 Powdergame 내부에서 배우고 이용할 수 있는 관계를 늘린다.

---

## Phase 3 — Discovery as World Research

Doodle God 계열의 발견 감각을 실제 simulation과 연결한다.

방향:

- 현상 단위 discovery
- 숨은 정확한 threshold 비공개
- 남은 exact discovery count 비공개
- “아직 발견하지 못한 성질이 있다” 정도의 hint
- 발견 사전 = 플레이어의 연구 노트
- reaction/causal observation UI

---

## Phase 4 — More Transferable Physics

M0 Temperature/Pressure 패턴을 확장할 수 있다.

후보:

- Electricity
- Radiation
- Gameplay Light
- additional force/field systems

원칙은 동일하다.

> **Minimum Sufficient Physics.**

각 system은 현실 equation을 그대로 구현하지 않고 gameplay에 필요한 최소 local state/transfer로 시작한다.

---

## Phase 5 — Experimentation Power

플레이어가 세계를 더 빠르게 이해하고 실험할 수 있는 능력.

- Rewind 고도화
- World Fork
- overlays
- causal inspection
- compare experiments
- save experiment states

Rewind는 core experiment tool로 본다.

---

## Future Developer Tool — Interaction Lab

현재는 `DEFERRED`.

완성된 Material/Rule을 actual GPU Simulation에 자동 투입해 기존 Matter/대표 환경과 상호작용을 탐색하는 개발 도구.

본 게임보다 우선하지 않는다.

---

## Long-term World Layers

장기 개념:

1. Matter
2. Field
3. Agent
4. Concept
5. Meta

가능한 방향:

```text
Matter
→ Energy / Chemistry
→ Life / Ecosystem
→ Machine
→ Information
→ Language
→ Society / Civilization
→ Myth / Belief
→ AI
→ Space / Time
→ World Rules
```

하지만 이 목록은 지금부터 빈 framework를 구현하라는 뜻이 아니다.

각 계층은 이전 계층의 실제 재미와 성능이 검증된 후 추가한다.

---

## DAN-BALL Idea Research

DAN-BALL 전체 작품군을 idea mine으로 참고한다.

별도 연구에서:

- 재미있는 mechanic
- 당시 구조/제약
- 현대 GPU에서의 재해석 가능성
- Powdergame의 기존 system과 연결 가능성

을 검토한다.

연구 후보가 자동으로 ROADMAP/MILESTONE이 되지는 않는다. 실제 채택 결정이 필요하다.

---

## Non-goals for the near term

- Browser/macOS product parity
- broad low-end GPU support
- true infinite world
- exact physical simulation
- exact global energy accounting
- deterministic multiplayer/replay architecture
- 모든 미래 system을 한 번에 구현

현재 목표는 **첫 세계가 정말 재미있고 빠르게 살아 움직이는지** 증명하는 것이다.
