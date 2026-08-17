---
title: Sand
type: material
id: sand
aliases: []
family: granular-terrain
status: adopted
implementation_state: implemented
movement_class: POWDER
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
  - powder
  - terrain
  - movement
---

# Sand

> 떨어지고 쌓이며, 작은 빈틈 하나로 지형의 모양을 바꾸는 가장 단순한 가루.

## 개념

Sand는 알갱이 집합의 거동을 Cell 단위 POWDER movement로 압축한 Foundation Matter다. 현실의 특정 모래 조성보다 **낙하, 대각선 미끄러짐, 퇴적, movable Matter 사이의 정렬**을 보여주는 기준 가루다.

## 왜 넣는가

Powdergame에는 고정된 구조와 흐르는 액체 사이의 문법이 필요하다. Sand는 지형이 무너지면서도 Liquid처럼 퍼지지 않는다는 차이를 가장 즉시 보여주며, movement와 density interaction의 기본 실험 도구가 된다.

## 핵심 동사

```text
FALL
PILE
SLIDE
SETTLE
```

## 플레이어 직관

Sand는 빈 공간을 향해 아래로 떨어지고 막히면 비스듬히 미끄러져 더미를 만든다. Water나 Oil 위에 놓으면 같은 위치를 차지하지 않고 국소적인 순서 경쟁을 거쳐 가라앉는 방향이 읽혀야 한다.

## 세계 안의 역할

- baseline POWDER
- erodible-looking terrain
- movement/density test material
- pile and slope builder
- future Glass precursor

## 대표 인과 사슬

```text
Sand above EMPTY
→ falls through local movement
→ blocked grains choose available diagonal paths
→ stable pile forms
```

```text
Sand above Water or Oil
→ local density displacement becomes eligible
→ movable Matter exchanges ownership safely
→ heavier grains tend to settle below liquid
```

```text
future: Sand + sufficient Heat
→ Glass manufacturing candidate
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Stone](stone.md) | support/obstacle | Stone의 형상에 따라 Sand가 쌓이고 미끄러진다. |
| [Water](water.md) | density/movement partner | Sand가 액체 아래로 정렬되는 Foundation 실험을 만든다. |
| [Oil](oil.md) | density/movement partner | 서로 다른 Liquid와 같은 POWDER 문법이 재사용되는지 보여준다. |
| [Glass](glass.md) | future manufactured result | 열을 통한 유리화 방향이며 현재 구현을 의미하지 않는다. |
| [Mud](../p1/mud.md) | movement contrast | Sand는 POWDER, Mud는 느린 Liquid identity 후보로 구분된다. |
| [Clay](../p1/clay.md) | family contrast | 둘 다 가루로 시작할 수 있지만 Clay는 젖음과 소성 동사를 가진다. |

## 독립 Material인 이유

Stone은 움직이지 않고 Water는 연속적으로 흐른다. Sand는 지지점이 사라지면 무너지면서도 더미를 만드는 POWDER 동사 때문에 독립적인 Foundation identity다.

## Palette / Discovery 정책

- **Palette:** `foundation_source_pending_player_palette`
- Foundation source로 직접 배치할 의도지만 일반 플레이어 palette UI 노출은 아직 확인되지 않았다.
- 구현 브랜치의 scenario, renderer, observatory/debug 경로는 Sand 생성과 식별을 지원한다. 이것을 player palette 노출 증거로 해석하지 않는다.
- 플레이어 도감은 “가루는 흐르는 것이 아니라 떨어지고 쌓인다”는 관찰을 우선한다.

## 현실 앵커와 게임 추상화

### 현실 앵커

마른 모래는 중력 아래에서 낙하하고 경사를 만들며, 물보다 무거운 알갱이는 침전하는 경향이 있다.

### 게임 추상화

입자 크기, 마찰계수, 수분, 압밀과 연속체 역학을 계산하지 않는다. 몇 개의 local destination과 density ordering으로 기억할 행동만 표현한다.

### 창작 보강

특정 모래 종류를 임의로 확정하지 않는다. local Cell arbitration으로 입자 더미를 만드는 부분이 Powdergame식 실행 abstraction이다.

## 구현 개요

- Movement class: POWDER
- Descriptor-level properties: movable density ordering과 thermal participation
- Rule owner: shared POWDER movement와 density displacement
- Update tier: 실제 movement frontier가 있을 때 활성
- State-cost policy: grain 크기·수분·마찰을 per-cell 상태로 추가하지 않음
- Current Rule Card: M0 movement/density 계약; Glass 전이는 아직 없음

### 현재 구현 상태와 증거 경계

구현 commit [`177879c`](https://github.com/Eeevah/Powdergame/commit/177879c2e2e916066f376a4465c00430a0cdd8ac)에서 Registry/GPU descriptor, POWDER movement, density interaction, chunk wake scenario, renderer와 debug 관찰 경로를 확인했다. 따라서 `implemented`다. pile의 최종 플레이 품질이나 일반 palette 노출까지 입증하지 않으므로 `validated`로 올리지 않는다. Glass 전이도 현재 구현으로 선언하지 않는다.

## 실패 모드 / 카운터

막힌 Sand가 Liquid처럼 멀리 옆으로 퍼지거나 안정된 더미가 영원히 활동하면 identity와 성능 원칙이 모두 무너진다. STATIC Matter와 일반 density swap을 하거나 ownership을 복제해서도 안 된다. 미래 Glass chain을 문서만으로 구현된 것처럼 보이게 하는 것도 실패다.

## 미결정 사항

- [ ] pile의 경사와 미끄러짐이 플레이어에게 충분히 자연스럽게 읽히는가?
- [ ] 일반 플레이어 palette에서 Sand를 언제 노출할 것인가?
- [ ] Glass 제조를 추가할 때 별도 universal 상태 없이 어떤 local condition을 사용할 것인가?

## 관련 문서

- [Foundation Materials](README.md)
- [Material Wiki](../README.md)
- [Mud](../p1/mud.md)
- [Clay](../p1/clay.md)
- [User Vision](../../../vision/USER_VISION.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [Foundation Catalog](../../encyclopedia/01A_FOUNDATION_CATALOG.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
- [Implementation evidence: Material Registry](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs)
- [Implementation evidence: Windows scenarios](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/apps/windows/src/main.rs)
