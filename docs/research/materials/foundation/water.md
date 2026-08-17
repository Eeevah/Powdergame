---
title: Water
type: material
id: water
aliases: []
family: water-phase
status: adopted
implementation_state: implemented
movement_class: LIQUID
palette_policy: foundation_source_pending_player_palette
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../../vision/USER_VISION.md
  - ../../../specs/SIMULATION_SPEC.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../specs/REACTION_SPEC.md
  - ../../encyclopedia/01A_FOUNDATION_CATALOG.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - ../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md
  - https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs
  - https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/apps/windows/src/main.rs
tags:
  - foundation
  - liquid
  - phase-transition
  - pressure
  - interaction-hub
---

# Water

> 형태를 갖지 않지만, 세계의 형태를 가장 많이 바꾸는 액체.

## 개념

Water는 흐름, liquid density ordering, 열전달, 상변화와 압력을 연결하는 Foundation 기준 액체다. 현실의 모든 수용액을 대표하지 않고 **Ice와 Steam 사이에서 상태를 바꾸며 다른 Matter의 행동을 드러내는 물**에 집중한다.

## 왜 넣는가

Water는 서로 떨어진 시스템을 하나의 causal chain으로 묶는다. 지형을 따라 흐르고 Oil과 층을 만들며, 열을 받으면 Steam이 되어 Pressure를 만들고, 차가워지면 Ice가 된다. 이후 Dirt·Clay·Lava·Limestone prototype의 공통 환경이기도 하다.

## 핵심 동사

```text
FLOW
LAYER
FREEZE
BOIL / CONDENSE
TRANSFER HEAT
```

## 플레이어 직관

Water는 아래와 옆의 빈 공간을 찾아 흐르고, Oil 아래에 층을 만들며, 차갑거나 뜨거운 환경에서 Ice 또는 Steam으로 이어져야 한다. 같은 물이 지형·열·압력 실험에 반복해서 쓰이는 것이 핵심이다.

## 세계 안의 역할

- baseline LIQUID
- phase-transition hub
- thermal transport medium
- pressure source through Steam
- terrain/manufacturing interaction input
- density-ordering reference

## 대표 인과 사슬

```text
Ice + Heat
→ Water
→ more Heat
→ Steam
→ confinement
→ Pressure and possible vent/rupture
```

```text
Water meets Oil
→ shared LIQUID movement
→ density ordering
→ visible layers form
```

```text
future P1: Water contacts Dirt or Clay
→ staged material transition
→ Mud or Wet Clay
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Ice](ice.md) | phase predecessor/result | 냉각과 가열로 오가는 고체 상태. |
| [Steam](steam.md) | phase result/predecessor | 가열·응축과 압력 chain을 연결한다. |
| [Oil](oil.md) | density contrast | 두 Liquid가 서로 다른 층을 만든다. |
| [Sand](sand.md) | displacement partner | Sand가 Water 아래로 정렬되는 movement 실험을 만든다. |
| [Stone](stone.md) | container/thermal path | 흐름과 압력을 가두고 열을 전달하는 구조다. |
| [Dirt](../p1/dirt.md) / [Mud](../p1/mud.md) | future wetting chain | P1에서 토양의 movement identity를 바꿀 입력이다. |
| [Clay](../p1/clay.md) / [Wet Clay](../p1/wet-clay.md) | future manufacturing input | 형태 잡기와 소성 전 단계를 여는 입력이다. |
| [Obsidian](../p1/obsidian.md) | future quench environment | Lava의 급랭 조건을 제공할 후보이며 현재 구현은 아니다. |
| [Limestone](../p1/limestone.md) | future reaction result/medium | Acid 반응의 중화 결과 abstraction과 장기 침전 chain에 연결된다. |

## 독립 Material인 이유

Water는 단순한 “파란 Liquid”가 아니다. Ice/Steam phase chain, Oil과의 ordering, Pressure 생성, 미래 지형·제조 반응을 함께 연결하는 Foundation interaction hub다.

## Palette / Discovery 정책

- **Palette:** `foundation_source_pending_player_palette`
- 세계 실험의 Foundation source로 의도하지만 일반 플레이어 palette UI에서의 실제 노출은 확인되지 않았다.
- 구현 브랜치에는 scenario/presentation/debug 경로에서 직접 생성하고 이름·색·계측으로 구별할 수 있다. 이는 개발 접근이며 player palette evidence가 아니다.
- 도감은 관찰된 흐름과 phase chain을 먼저 공개하고, 숨은 threshold나 Rule priority는 공개하지 않는다.

## 현실 앵커와 게임 추상화

### 현실 앵커

물은 중력 아래 흐르고 고체·액체·기체 상태를 오가며, 가열된 밀폐 환경에서는 기체 생성이 압력 문제로 이어질 수 있다.

### 게임 추상화

유체 연속 방정식, 습도, 용존 물질, 미세한 증발과 실제 열역학을 풀지 않는다. 한 Cell의 단일 Matter identity와 local movement/transition으로 필요한 현상만 표현한다.

### 창작 보강

상변화와 압력의 정확한 조건·속도는 Powdergame의 gameplay tuning이다. Water가 모든 물질을 자동으로 적시거나 녹이는 universal solvent라는 규칙은 추가하지 않는다.

## 구현 개요

- Movement class: LIQUID
- Descriptor-level properties: movable density, thermal participation, phase transition, pressure-medium behavior
- Rule owner: shared LIQUID movement와 Water phase transition
- Update tier: movement·thermal·phase·pressure frontier가 있을 때 관련 work
- State-cost policy: humidity·salinity·wetness를 Water나 모든 Cell에 미리 추가하지 않음
- Current Rule Card: M0 phase/pressure contracts; P1 wetting/quench/reaction rules는 아직 구현 범위 밖

### 현재 구현 상태와 증거 경계

구현 commit [`177879c`](https://github.com/Eeevah/Powdergame/commit/177879c2e2e916066f376a4465c00430a0cdd8ac)에서 Registry/GPU descriptor, LIQUID movement와 density ordering, Ice/Steam phase transition, thermal·pressure·sleep/wake scenario, renderer와 observatory/debug 접근을 확인했다. 따라서 `implemented`다. 일반 player palette와 Water 단독 Product Gate의 증거는 아니므로 `validated`로 올리지 않는다. 위 P1 반응은 링크된 연구이며 현재 구현 사실이 아니다.

## 실패 모드 / 카운터

안정된 Water bulk가 존재만으로 영원히 active하거나, Oil과의 ordering이 읽히지 않거나, phase transition이 invalid ownership을 만들면 실패다. Water 접촉만으로 모든 Matter가 보편적 wetness 상태를 얻어서도 안 된다. Stone과 Boundary가 없는 곳에서 원격 반응을 일으키는 “만능 반응 키”가 되지 않도록 local contact와 Material-owned Rule이 counter가 된다.

## 미결정 사항

- [ ] 일반 플레이어 palette에서 Water를 어떤 초기 도구 집합으로 노출할 것인가?
- [ ] P1 wetting Rule에서 Water가 소모되는지, 환경으로 남는지 prototype별 ownership을 어떻게 정할 것인가?
- [ ] phase와 pressure feedback이 복잡한 장면에서도 플레이어에게 같은 causal chain으로 읽히는가?

## 관련 문서

- [Foundation Materials](README.md)
- [Material Wiki](../README.md)
- [Dirt](../p1/dirt.md)
- [Mud](../p1/mud.md)
- [Clay](../p1/clay.md)
- [Wet Clay](../p1/wet-clay.md)
- [Obsidian](../p1/obsidian.md)
- [Limestone](../p1/limestone.md)
- [User Vision](../../../vision/USER_VISION.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [Foundation Catalog](../../encyclopedia/01A_FOUNDATION_CATALOG.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
- [Implementation evidence: Material Registry](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs)
- [Implementation evidence: Windows scenarios](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/apps/windows/src/main.rs)
