---
title: Oil
type: material
id: oil
aliases:
  - Combustible Oil
family: combustible-liquid
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
  - https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/combustion.rs
tags:
  - foundation
  - liquid
  - fuel
  - combustion
  - density
---

# Oil

> 물 위에 길을 만들고, 불이 붙으면 그 길을 따라 열과 연기를 남기는 액체 연료.

## 개념

Oil은 여러 가연성 기름을 하나로 압축한 gameplay archetype이다. **Water와 다른 층을 만드는 Liquid**이면서 **Heat를 만나 finite combustion chain을 시작하는 연료**라는 두 동사를 결합한다.

## 왜 넣는가

Water 하나만으로는 모든 Liquid가 같은 방식으로 보인다. Oil은 같은 movement family 안에서도 density와 combustion descriptor가 다른 결과를 만들 수 있음을 보여주고, 흐름이 곧 연료 경로가 되는 공간 실험을 연다.

## 핵심 동사

```text
FLOW
FLOAT / LAYER
IGNITE
BURN DOWN
PRODUCE HEAT AND SMOKE
```

## 플레이어 직관

Oil은 Water 위에 층을 만들고 통로를 따라 흐른다. 충분한 점화 조건을 만나면 표면과 연결된 연료가 타며 Heat와 Smoke를 남기지만 영원한 불이 되어서는 안 된다.

## 세계 안의 역할

- combustible LIQUID
- density-layering contrast
- mobile fuel path
- combustion exemplar
- Heat/Smoke source

## 대표 인과 사슬

```text
Oil meets Water
→ shared LIQUID movement
→ density ordering
→ Oil layer remains above Water
```

```text
Oil reaches ignition condition
→ finite combustion state
→ Heat + Smoke
→ nearby fuel may ignite
→ exhausted Oil disappears
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Water](water.md) | density contrast | Oil이 Water 위에 층을 만들어 Liquid 차이를 보여준다. |
| [Stone](stone.md) | container/ignition environment | 흐름을 가두고 뜨거운 구조가 점화 조건을 전달할 수 있다. |
| [Sand](sand.md) | density/movement partner | POWDER와 Liquid가 같은 local displacement 계약을 공유한다. |
| [Wood](wood.md) | combustion-family peer | 같은 공용 연소 문법을 서로 다른 fuel identity가 재사용한다. |
| [Smoke](smoke.md) | combustion output | 연소 경로와 소모를 시각적으로 남긴다. |
| [Carbon Dioxide](../p1/carbon-dioxide.md) | future counter | P2에서 combustion suppression 후보이며 Oil의 현재 구현 Rule은 아니다. |

## 독립 Material인 이유

Oil은 Water의 색상 variant가 아니다. 서로 다른 density ordering과 점화·연료 소모·Smoke 생성이라는 기억할 동사를 가진다. Wood와도 같은 combustion grammar를 공유하지만 흐르는 연료라는 공간적 차이가 있다.

## Palette / Discovery 정책

- **Palette:** `foundation_source_pending_player_palette`
- Foundation fuel source로 의도하지만 일반 플레이어 palette UI 노출은 아직 확인되지 않았다.
- 구현 브랜치에는 scenario/presentation/debug 경로에서 직접 생성하고 색·이름·열·연소 상태로 관찰할 수 있다. 이는 player palette button의 증거가 아니다.
- 플레이어 도감은 층분리와 연소 결과를 관찰한 뒤 “흐르는 연료”라는 문장을 제공하고 내부 점화·소모 tuning은 숨긴다.

## 현실 앵커와 게임 추상화

### 현실 앵커

많은 기름은 물과 섞이지 않고 물 위에 뜨며, 가연성 종류는 점화되면 열과 연소 생성물을 낸다.

### 게임 추상화

점도, 유종, 휘발성, 유화, 산소 농도와 복잡한 연소 화학을 계산하지 않는다. 하나의 Oil identity가 LIQUID ordering과 공용 combustion descriptor를 사용한다.

### 창작 보강

정확한 현실 석유 제품 하나가 아니라 “물 위를 흐르는 finite fuel”이라는 합성 archetype이다. 점화 조건과 burn duration은 gameplay tuning이며 여기서 고정하지 않는다.

## 구현 개요

- Movement class: LIQUID
- Descriptor-level properties: movable density, thermal participation, combustible finite-fuel behavior
- Rule owner: shared LIQUID movement와 generic combustion grammar
- Update tier: movement·thermal·combustion frontier가 있을 때 활성
- State-cost policy: 유종·산소량·휘발성 같은 universal per-cell 상태를 추가하지 않음
- Current Rule Card: M0 combustion contract; CO2 suppression은 future P2 후보

### 현재 구현 상태와 증거 경계

구현 commit [`177879c`](https://github.com/Eeevah/Powdergame/commit/177879c2e2e916066f376a4465c00430a0cdd8ac)에서 Registry/GPU descriptor, LIQUID movement와 Water layering, generic finite combustion, Heat/Smoke 결과, scenario/presentation/debug 접근을 확인했다. 따라서 `implemented`다. 일반 player palette 노출과 Oil 단독 Product Gate를 증명하지 않으므로 `validated`로 기록하지 않는다. CO2 suppression은 구현 사실로 올리지 않는다.

## 실패 모드 / 카운터

Oil이 Water 아래로 가라앉거나 둘이 같은 층처럼 보이면 density identity가 실패한다. 점화된 Oil이 연료 소모 없이 영원히 타거나 멀리 떨어진 Oil을 즉시 점화해도 안 된다. 냉각·연료 소모와 국소 접촉이 combustion의 counter이며, 안정된 Oil bulk는 존재만으로 계속 active하지 않아야 한다.

## 미결정 사항

- [ ] 일반 플레이어 palette에서 Oil을 언제, 어떤 위험 표시와 함께 노출할 것인가?
- [ ] 흐르는 연료의 전파가 충분히 읽히면서도 한 번의 점화로 과도하게 번지지 않는가?
- [ ] 미래에 Alcohol 등 다른 liquid fuel이 생길 때 Oil이 유지할 고유 동사는 무엇인가?
- [ ] CO2 suppression을 공용 local modifier로 추가할 때 Oil/Smoke chain이 어떻게 달라져야 하는가?

## 관련 문서

- [Foundation Materials](README.md)
- [Material Wiki](../README.md)
- [Carbon Dioxide](../p1/carbon-dioxide.md)
- [User Vision](../../../vision/USER_VISION.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [Foundation Catalog](../../encyclopedia/01A_FOUNDATION_CATALOG.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
- [Implementation evidence: Material Registry](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs)
- [Implementation evidence: generic combustion](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/combustion.rs)
