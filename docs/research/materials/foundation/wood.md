---
title: Wood
type: material
id: wood
aliases:
  - Timber
family: organic-structure
status: adopted
implementation_state: implemented
movement_class: STATIC
palette_policy: foundation_source_pending_player_palette
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../../vision/USER_VISION.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../specs/SIMULATION_SPEC.md
  - ../../../specs/REACTION_SPEC.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - "https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs#L294-L310"
tags:
  - foundation
  - structure
  - fuel
  - combustion
  - implemented
---

# Wood

> 서 있을 때는 구조물이지만, 열을 만나면 세계를 바꾸는 연료.

## 개념

Wood는 다양한 목재와 건조한 식물성 구조를 하나로 묶은 STATIC organic archetype이다. 구조를 만들고 Pressure를 버티는 동시에, thermal condition이 맞으면 공통 combustion grammar에 참여해 Heat와 Smoke를 만든다.

## 왜 넣는가

Stone만으로는 구조가 모두 영구적인 벽이 된다. Wood는 평소에는 공간을 나누지만 Heat와 Pressure 앞에서는 변하는 약한 구조이므로, construction과 destruction을 같은 causal chain에 연결한다.

## 핵심 동사

```text
BUILD
IGNITE
BURN
RUPTURE
```

## 플레이어 직관

Wood는 움직이지 않는 구조재이자 연료다. 충분히 뜨거워지면 타면서 Heat와 Smoke를 내고, 강한 Pressure를 받으면 Stone보다 먼저 길을 열 수 있다고 예상할 수 있다.

## 세계 안의 역할

- breakable STATIC structure
- finite combustible fuel
- Heat와 Smoke source
- Pressure relief / rupture material
- 향후 ecology와 manufacturing을 잇는 biomass output

## 대표 인과 사슬

```text
Heat reaches Wood
→ combustion begins
→ Heat + Smoke
→ nearby Wood heats and may ignite
→ fuel lifecycle ends
→ opening changes later movement
```

```text
Steam confinement
→ Pressure rises
→ Wood ruptures
→ opening forms
→ Steam vents
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Smoke](smoke.md) | combustion output | burning Wood가 local spawn request를 만든다. |
| [Oil](oil.md) | shared grammar / contrast | 둘 다 타지만 Wood는 STATIC 구조이고 Oil은 LIQUID fuel이다. |
| [Steam](steam.md) | pressure source | confined Steam chain에서 약한 relief structure가 된다. |
| [Stone](stone.md) | structural contrast | Stone은 M0 pressure control, Wood는 rupture 가능한 구조다. |
| [Plant](plant.md) | future source family | ecology가 구현되면 Plant/Tree가 Wood 생산으로 이어질 수 있다. |
| [Dirt](../p1/dirt.md) | future ecology substrate | 향후 growth chain의 terrain input 후보이며 현재 Wood rule은 아니다. |
| Fire / Combustion | state/phenomenon | Fire라는 별도 Matter가 아니라 Wood의 burning state와 Heat가 결과를 만든다. |

## 독립 Material인 이유

Wood는 단순히 갈색 Stone이 아니다. 구조재이면서 finite fuel이고 Pressure에 파괴될 수 있다는 세 동사가 결합된다. Oil과는 같은 combustion grammar를 공유하지만 movement와 공간 역할이 다르다.

## Palette / Discovery 정책

- **Palette:** `foundation_source_pending_player_palette`
- 현재 구현에는 일반 플레이어용 Material palette가 없다.
- Wood는 debug/demo fixture에서 직접 배치되고 보이지만, 이는 player-facing 선택 palette가 존재한다는 증거가 아니다.
- **Player Dictionary:** “나무는 벽이 될 수 있지만, 열을 받으면 연기와 더 많은 열을 남기며 무너진다.”
- ignition, sustain, fuel-life, rupture 수치는 숨긴다.

## 현실 앵커와 게임 추상화

### 현실 앵커

목재는 식물성 구조재이며 조건이 맞으면 연소한다. 종류, 수분, 구조에 따라 열전달과 강도, 연소 양상이 크게 달라진다.

### 게임 추상화

수종, 결 방향, 수분율, 산소 농도, 실제 화학 반응을 계산하지 않는다. Wood는 하나의 STATIC Material descriptor와 generic combustion/rupture property로 표현한다. Oxygen은 현재 hardcoded 필수 조건이 아니다.

### 창작 보강

finite fuel lifecycle 뒤 EMPTY로 전환하고, local Pressure threshold로 구조를 여는 것은 읽기 쉬운 game-consistent abstraction이다. 향후 Charcoal/Ash를 추가하더라도 현재 구현 결과로 소급해 주장하지 않는다.

## 구현 개요

- Canonical Wiki ID: `wood`
- Engine Material ID: `9`
- Movement class: `STATIC`
- 현재 구현: generic finite combustion descriptor, Smoke generation source, structural rupture property
- Rule ownership: Wood가 자기 combustion state를 결정하고 spatial output은 spawn ownership path를 사용한다.
- State-cost policy: universal wetness나 산화 progress 없음
- Code evidence: [Material ID constants](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs#L28-L47), [Wood descriptor](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs#L294-L310)

커밋 `177879c2e2e916066f376a4465c00430a0cdd8ac`에서 identity, combustion, rupture descriptor가 구현되어 있음을 확인했다. 이 근거는 `implementation_state: implemented`를 지지하지만 개별 Product Gate의 `validated` 판정은 아니다.

## 실패 모드 / 카운터

Wood가 permanent flame처럼 영원히 타거나, Oxygen simulation을 암묵적으로 요구하거나, 타는 동안 실제 Heat/Smoke를 만들지 않으면 안 된다. Stone과 같은 강도로 버티거나 모든 Cell에 Wood 전용 progress state를 추가하는 것도 Foundation 비용 원칙에 맞지 않는다.

## 미결정 사항

- [ ] 일반 플레이어 palette에서 Wood를 Foundation source로 즉시 제공할 것인가?
- [ ] 연소 뒤 Charcoal/Ash를 독립 result로 만들 만큼 새로운 동사가 있는가?
- [ ] ecology가 도입될 때 Plant → Tree/Wood를 identity, state, spawn 중 무엇으로 표현할 것인가?

## 관련 문서

- [Foundation Material index](README.md)
- [Material Wiki](../README.md)
- [Smoke](smoke.md)
- [Oil](oil.md)
- [Steam](steam.md)
- [Stone](stone.md)
- [Plant](plant.md)
- [Dirt](../p1/dirt.md)
- [Authoritative User Vision](../../../vision/USER_VISION.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
