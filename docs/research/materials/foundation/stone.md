---
title: Stone
type: material
id: stone
aliases:
  - Rock
family: foundation-terrain
status: adopted
implementation_state: implemented
movement_class: STATIC
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
  - ../../derived/BLOCK_PALETTE_AND_PG2_GAP_REVIEW.md
  - https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs
  - https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/apps/windows/src/main.rs
tags:
  - foundation
  - terrain
  - structure
  - static
---

# Stone

> 움직이지 않기 때문에 다른 모든 것이 어디로 움직일지 결정하는 물질.

## 개념

Stone은 여러 암석을 하나로 압축한 Foundation 구조재 archetype이다. 특정 광물 조성보다 **고정된 지형, 흐름을 가르는 벽, 열과 압력 실험의 안정 기준**이라는 역할을 우선한다.

## 왜 넣는가

Powder, Liquid, Gas의 움직임은 움직이지 않는 기준면이 있을 때 읽힌다. Stone이 없으면 그릇, 경사면, 동굴, 압력 챔버와 열전달 경로를 같은 세계 문법으로 만들 수 없다.

## 핵심 동사

```text
SUPPORT
BLOCK
CONTAIN
CONDUCT HEAT
```

## 플레이어 직관

Stone 위에는 다른 Matter가 쌓이고, Stone 벽은 흐름을 돌리며, 뜨거운 Stone은 인접한 세계에 열을 전달한다. 일반 movement로 무너지지는 않지만 모든 암석 반응을 대표하지도 않는다.

## 세계 안의 역할

- static terrain baseline
- structure and container
- movement obstacle
- thermal path/control
- pressure-control material

## 대표 인과 사슬

```text
Falling or flowing Matter
→ meets Stone
→ cannot occupy the same Cell
→ piles up or finds another local path
```

```text
Heat source
→ conductive Stone path
→ nearby Matter changes temperature
→ phase or combustion rule may become eligible
```

```text
Pressure builds in a Stone chamber
→ Stone remains the stable control
→ a deliberately weaker part becomes the rupture path
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Sand](sand.md) | support/contrast | Stone은 고정되고 Sand는 쌓이며 무너진다. |
| [Water](water.md) | container/thermal partner | 흐름을 가두고 열전달·압력 실험의 형상을 만든다. |
| [Oil](oil.md) | container/ignition support | 흐름을 제한하고 뜨거운 구조가 점화 환경이 될 수 있다. |
| [Boundary Block](boundary-block.md) | semantic contrast | Stone은 세계 안의 Matter이고 Boundary Block은 domain primitive다. |
| [Brick](../p1/brick.md) | manufactured contrast | Brick은 제조 경로와 별도 구조 성격으로 Stone과 구분되어야 한다. |
| [Basalt](../p1/basalt.md) / [Obsidian](../p1/obsidian.md) / [Limestone](../p1/limestone.md) | future family decomposition | 구체적인 생성·반응 동사를 가진 암석 identity다. |

## 독립 Material인 이유

Stone은 개별 암석을 대체하는 최종 분류가 아니라, Foundation 단계에서 구조와 지형 문법을 제공하는 유용한 abstraction이다. 이후 구체 암석은 생성 경로나 반응이 실제로 다를 때만 별도 identity를 얻는다.

## Palette / Discovery 정책

- **Palette:** `foundation_source_pending_player_palette`
- 세계를 만들기 위한 Foundation source로 의도하지만 일반 플레이어 palette UI의 실제 노출은 아직 확인되지 않았다.
- 구현 브랜치에는 scenario/presentation/debug 접근과 시각적 식별이 있다. 이는 개발·검증 접근의 증거이지 player palette button의 증거가 아니다.
- 플레이어 도감은 “움직이지 않는 구조가 다른 Matter의 경로를 만든다”는 관찰에서 시작한다.

## 현실 앵커와 게임 추상화

### 현실 앵커

암석은 지형과 구조를 이루고, 종류에 따라 열과 압력에 다른 방식으로 반응한다.

### 게임 추상화

조성, 공극, 균열, 풍화와 암석 종류를 하나의 연속 모델로 계산하지 않는다. Stone은 STATIC 구조 baseline만 맡는다.

### 창작 보강

서로 다른 암석을 하나의 즉시 이해 가능한 Foundation identity로 묶은 것이 게임적 보강이다. 구체 암석의 고유 동사는 별도 Material로 남긴다.

## 구현 개요

- Movement class: STATIC
- Descriptor-level properties: thermal participation과 구조적 control 역할
- Rule owner: 일반 movement rule 없음; 관련 field/rule이 Stone을 descriptor로 읽음
- Update tier: 변화 가능한 thermal/pressure 경계에서만 관련 work
- State-cost policy: 암석 조성이나 균열 상태를 모든 Stone Cell에 추가하지 않음
- Current Rule Card: M0 Simulation/Material 계약과 scenario evidence

### 현재 구현 상태와 증거 경계

구현 commit [`177879c`](https://github.com/Eeevah/Powdergame/commit/177879c2e2e916066f376a4465c00430a0cdd8ac)에서 Registry와 GPU descriptor, STATIC behavior, thermal·pressure scenario의 구조/control, renderer와 debug 접근을 확인했다. 따라서 `implemented`다. 개별 Stone Product Gate나 일반 palette 노출을 입증하는 자료로 보지 않으므로 `validated`로 기록하지 않는다.

## 실패 모드 / 카운터

Stone이 일반 density swap으로 움직이거나 모든 압력·열 변화의 절대 불변벽이 되면 Foundation 역할을 벗어난다. 반대로 Boundary Block과 동일하게 취급하면 편집 가능한 구조재와 world topology의 차이가 사라진다. 구체 암석의 동사를 모두 흡수해 Basalt, Obsidian, Limestone을 이름만 다른 재색칠로 만드는 것도 실패다.

## 미결정 사항

- [ ] 일반 플레이어 palette에서 Stone을 언제, 어떤 범주로 노출할 것인가?
- [ ] Foundation Stone의 파괴 가능성과 pressure control 역할을 어디까지 유지할 것인가?
- [ ] Basalt/Limestone 등 concrete rock family가 늘어날 때 Stone의 고유 사용처는 무엇으로 남길 것인가?

## 관련 문서

- [Foundation Materials](README.md)
- [Material Wiki](../README.md)
- [Brick](../p1/brick.md)
- [Basalt](../p1/basalt.md)
- [Obsidian](../p1/obsidian.md)
- [Limestone](../p1/limestone.md)
- [User Vision](../../../vision/USER_VISION.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [Foundation Catalog](../../encyclopedia/01A_FOUNDATION_CATALOG.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Block Palette & PG2 Gap Review](../../derived/BLOCK_PALETTE_AND_PG2_GAP_REVIEW.md)
- [Implementation evidence: Material Registry](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs)
- [Implementation evidence: Windows scenarios](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/apps/windows/src/main.rs)
