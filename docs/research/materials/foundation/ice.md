---
title: Ice
type: material
id: ice
aliases:
  - Water Ice
family: water-phase
status: adopted
implementation_state: implemented
movement_class: STATIC
palette_policy: foundation_phase_state_pending_player_palette
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../../vision/USER_VISION.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../specs/SIMULATION_SPEC.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - ../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md
  - "https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs#L277-L293"
tags:
  - foundation
  - water-phase
  - temperature
  - implemented
---

# Ice

> 흐르던 물을 멈춰 세우고, 열이 돌아오면 다시 길을 내주는 고체.

## 개념

Ice는 Water의 차가운 고체 상태를 독립 Matter identity로 표현한다. 현실의 모든 얼음 종류를 구분하는 데이터베이스가 아니라, Temperature가 이동 방식과 공간 구조를 바꾼다는 Foundation 문법을 보여주는 Water-family archetype이다.

## 왜 넣는가

Ice가 없으면 냉각은 숫자 변화로만 남고 세계의 형태를 바꾸지 못한다. Ice는 흐르는 Water를 STATIC 구조로 바꾸고, 다시 가열하면 흐름을 돌려놓아 가역적인 phase chain을 눈으로 읽게 한다.

## 핵심 동사

```text
FREEZE
HOLD SHAPE
MELT
```

## 플레이어 직관

차가워진 Water는 움직임을 멈추고 Ice가 되며, 충분한 Heat를 받으면 다시 Water로 녹는다고 예상할 수 있다. 정확한 전환 조건과 내부 수치는 처음부터 공개하지 않는다.

## 세계 안의 역할

- Water-family의 STATIC phase
- 냉각 결과
- 임시 구조와 흐름 차단
- 가열/냉각 인과를 보여주는 관찰 대상
- 향후 급랭 환경의 입력 후보

## 대표 인과 사슬

```text
Water + 충분한 냉각
→ Ice
→ 흐름 정지와 공간 구조 형성
→ Heat 유입
→ Water
→ 다시 local flow
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Water](water.md) | phase source/result | 냉각으로 Ice가 되고 가열로 Water로 돌아간다. |
| [Steam](steam.md) | extended phase family | Water를 사이에 둔 Water-family의 GAS 상태다. |
| Temperature | transition condition | 별도 냉기 Matter 없이 Cell의 thermal state가 변환을 유도한다. |
| [Obsidian](../p1/obsidian.md) | future quench environment | P1에서는 Lava 급랭 조건을 제공할 후보지만 현재 Foundation 구현 규칙은 아니다. |

## 독립 Material인 이유

Ice를 차가운 Stone이나 Water의 시각 효과로만 처리하면 phase transition 뒤 movement class가 바뀌는 세계 규칙을 표현할 수 없다. Water와 오갈 수 있는 STATIC identity라는 점이 핵심 차이다.

## Palette / Discovery 정책

- **Palette:** `foundation_phase_state_pending_player_palette`
- 현재 구현에는 일반 플레이어용 Material palette가 없다.
- Ice는 debug/demo renderer에서 구별되어 보이지만, debug/demo 노출은 플레이어 palette 노출을 뜻하지 않는다.
- **Player Dictionary:** “물은 충분히 차가워지면 흐름을 멈추고, 열을 받으면 다시 흐른다.”
- 정확한 threshold와 남은 discovery 개수는 숨긴다.

## 현실 앵커와 게임 추상화

### 현실 앵커

물은 냉각되면 고체 얼음이 될 수 있고, 열을 받으면 다시 액체로 녹는다. 고체가 되면 액체와 다른 형태 유지와 이동 성격을 보인다.

### 게임 추상화

결정 구조, 염분, 잠열, 균열 전파를 계산하지 않는다. Ice는 공통 STATIC movement family와 Material-owned temperature transition으로 표현하며, 전환 수치는 현실의 섭씨값이 아니라 gameplay 값이다.

### 창작 보강

반복 가능한 local rule을 위해 전환 조건 사이에 안정 구간을 둘 수 있다. 향후 Ice가 Lava 급랭을 돕는 관계도 현실을 참고한 게임용 world grammar이며 현재 P1 후보 규칙과 구분한다.

## 구현 개요

- Canonical Wiki ID: `ice`
- Engine Material ID: `8`
- Movement class: `STATIC`
- 현재 구현: Ice → Water thermal self-transition과 Water → Ice 전환의 target identity
- State-cost policy: 별도 freezing progress나 결정 구조 state 없음
- Code evidence: [Material ID constants](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs#L28-L47), [Ice descriptor](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs#L277-L293)

커밋 `177879c2e2e916066f376a4465c00430a0cdd8ac`에서 identity와 phase descriptor가 구현되어 있음을 확인했다. 이는 개별 Material의 Product Gate 완료나 `validated` 상태를 뜻하지 않는다.

## 실패 모드 / 카운터

Ice와 Water가 경계 조건 근처에서 매 Tick 왕복하거나, Ice가 단지 색이 다른 Stone처럼 남으면 phase identity가 실패한다. Ice를 숨은 냉기 source로 취급하거나 EMPTY를 열 매질로 사용해서도 안 된다.

## 미결정 사항

- [ ] 일반 플레이어 palette에서 Ice를 처음부터 선택 가능하게 할 것인가, Water phase 발견 뒤 노출할 것인가?
- [ ] 향후 Pressure 파쇄나 취성을 추가할 때 Stone과 다른 동사가 충분히 읽히는가?
- [ ] P1 Lava 급랭에서 Ice 접촉을 어떤 cheap local 조건으로 사용할 것인가?

## 관련 문서

- [Foundation Material index](README.md)
- [Material Wiki](../README.md)
- [Water](water.md)
- [Steam](steam.md)
- [Obsidian](../p1/obsidian.md)
- [Authoritative User Vision](../../../vision/USER_VISION.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
