---
title: P1 Geology & Irreversible Manufacture
type: material-family-index
id: material-family-p1-geology-manufacture
status: prototype
implementation_state: not_registered
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
tags:
  - geology
  - manufacturing
  - transition
---

# P1 — Geology & Irreversible Manufacture

> 익숙한 흙·돌·용암이 접촉과 온도만으로 새로운 지형과 제작물을 만드는 첫 콘텐츠 묶음.

`P1`은 Roadmap Phase 1이 아니라 **Prototype Bundle 1**이다. M0 범위나 Evidence Gate를 변경하지 않는다.

## 이 family가 증명할 것

```text
Dirt + Water → Mud → drying → Dirt
Clay + Water → Wet Clay → Heat → Brick
Lava + ordinary cooling → Basalt
Lava + rapid quench → Obsidian
Limestone + Acid → CO2 + neutralized liquid abstraction
```

이 묶음은 다음 빈칸을 채운다.

- Water가 지형의 성격을 바꿈
- Heat가 파괴가 아니라 제조를 일으킴
- 냉각 환경이 암석 결과를 바꿈
- Acid가 고체를 단순 삭제하지 않고 Gas를 생성함
- 팔레트에 없는 결과물이 세계 안에서 발견됨

## 물질 지도

```text
Dirt ──Water──> Mud ──drying──> Dirt

Clay ──Water──> Wet Clay
  └────Heat───────────────> Brick
Wet Clay ──Heat───────────> Brick
Wet Clay ──drying─────────> Clay

Lava ──ordinary cooling───> Basalt
Lava ──rapid quench───────> Obsidian
Basalt / Obsidian ──Heat──> Lava

Limestone + Acid
→ CO2 + Water abstraction
```

## Foundation dependencies

P1은 새 범용 물리층을 만들지 않고 [Foundation Material](../foundation/README.md)의 기존·후보 어휘를 조합한다.

- [Water](../foundation/water.md), [Ice](../foundation/ice.md), [Steam](../foundation/steam.md) — 젖음, 건조, 급랭, 상변화 환경
- [Lava](../foundation/lava.md) — Basalt/Obsidian 냉각 분기의 source identity 후보
- [Acid](../foundation/acid.md) — Limestone/CO2 반응의 liquid reactant 후보
- [Stone](../foundation/stone.md), [Sand](../foundation/sand.md) — 구조·Powder baseline과 결과 비교
- [Smoke](../foundation/smoke.md) — CO2가 달라야 하는 기존 Gas 비교점
- [Seed](../foundation/seed.md), [Plant](../foundation/plant.md) — Dirt의 생태 연결은 future scope이며 P1 Rule에는 포함하지 않음

## 페이지

### 자연 원료

- [Dirt](dirt.md) — 물과 생명을 연결하는 기본 토양
- [Clay](clay.md) — 젖음과 소성을 잇는 광물성 가루
- [Limestone](limestone.md) — 산과 반응해 기체를 드러내는 암석

### 중간 상태

- [Mud](mud.md) — 물 때문에 흐르게 된 흙
- [Wet Clay](wet-clay.md) — 형태를 잡고 소성할 수 있는 젖은 점토

### 제조·지질 결과

- [Brick](brick.md) — 열이 만든 비가역 구조재
- [Basalt](basalt.md) — 보통 냉각된 용암의 기록
- [Obsidian](obsidian.md) — 급랭된 용암의 유리질 기록
- [Carbon Dioxide](carbon-dioxide.md) — 석회암 반응에서 드러나는 무거운 기체

## 공통 구현 원칙

- universal wetness/cooling-history/progress 필드를 추가하지 않는다.
- staged Material identity로 느린 변화를 표현한다.
- 일반 규칙은 `Read Neighbors, Write Self`.
- 수치는 최신 [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)에서 관리한다.
- 결과물은 palette button과 분리해 관리한다.
- 각 identity는 prototype evidence를 독립적으로 통과하거나 탈락한다.

## 승격 기준

- Mud가 Water와 다른 장난감이어야 한다.
- Brick이 단순 Stone 재색칠이 아니어야 한다.
- Basalt/Obsidian이 냉각 맥락을 읽히게 해야 한다.
- Limestone 반응이 Gas generation이라는 기억할 장면을 만들어야 한다.
- 안정 상태가 Sleep 가능해야 한다.
