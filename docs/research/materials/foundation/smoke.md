---
title: Smoke
type: material
id: smoke
aliases: []
family: combustion-byproduct
status: adopted
implementation_state: implemented
movement_class: GAS
palette_policy: result_identity_pending_player_palette
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../../vision/USER_VISION.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../specs/SIMULATION_SPEC.md
  - ../../../specs/REACTION_SPEC.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - "https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs#L261-L275"
tags:
  - foundation
  - gas
  - combustion
  - byproduct
  - implemented
---

# Smoke

> 불이 지나간 자리를 잠시 떠다니다가 사라지는 어두운 흔적.

## 개념

Smoke는 combustion이 만들어 내는 유한한 GAS byproduct다. Fire 자체도 Presentation 효과도 아니며, 실제 Cell을 차지해 이동하고 다른 Gas와 공간을 경쟁하다가 lifecycle이 끝나면 사라지는 Matter다.

## 왜 넣는가

Smoke가 없으면 연소는 Heat와 색 변화로만 끝나 공간에 남는 결과가 없다. Smoke는 combustion을 다음 movement와 density 실험으로 연결하고, world가 반응의 결과를 잠시 기억하게 한다.

## 핵심 동사

```text
SPAWN FROM COMBUSTION
RISE / DRIFT
OCCUPY SPACE
DECAY
```

## 플레이어 직관

[Wood](wood.md)나 [Oil](oil.md)이 타면 Smoke가 생겨 주변 빈 공간으로 떠오르고, 영원히 남지 않고 차츰 사라진다고 예상할 수 있다. Smoke와 flame은 서로 다른 현상이다.

## 세계 안의 역할

- combustion output
- transient GAS Matter
- 공간 점유와 Gas ordering 관찰 대상
- 연소 위치와 진행 방향의 가시적 흔적
- 향후 filtration, capture, residue system의 입력 후보

## 대표 인과 사슬

```text
Wood / Oil + ignition condition
→ combustion state
→ Heat + Smoke spawn request + flame presentation event
→ Smoke local GAS movement
→ finite decay
→ EMPTY
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Wood](wood.md) | source | 공통 combustion grammar가 Smoke 생성 요청을 낸다. |
| [Oil](oil.md) | source | Wood와 다른 Matter이지만 같은 연소 문법을 사용한다. |
| [Steam](steam.md) | Gas contrast | 같은 GAS family라도 source, phase 관계, lifecycle이 다르다. |
| Fire / Combustion | producing phenomenon | Fire는 Matter가 아니며 burning state와 Heat가 Smoke를 만든다. |
| EMPTY | decay target / spawn space | 생성은 빈 Cell ownership을 얻어야 하며 lifecycle 종료 뒤 EMPTY로 돌아간다. |
| [Carbon Dioxide](../p1/carbon-dioxide.md) | future Gas contrast | P1의 heavier Gas 후보와 movement/density 차이를 비교한다. |

## 독립 Material인 이유

Smoke를 flame overlay로만 그리면 공간 점유, GAS movement, spawn contention, decay를 simulation truth로 다룰 수 없다. Steam과 색만 다른 Gas가 아니라 combustion에서 생기고 Water로 응축하지 않는 transient identity다.

## Palette / Discovery 정책

- **Palette:** `result_identity_pending_player_palette`
- 현재 구현에는 일반 플레이어용 Material palette가 없다.
- Smoke는 debug/demo renderer에서 구별되지만, 연소 결과를 보는 것과 플레이어 palette에서 직접 생성하는 것은 별도 결정이다.
- **Player Dictionary:** “타는 물질은 열뿐 아니라, 잠시 공간을 차지하는 연기를 남긴다.”
- decay 시간, spawn 순서, density 수치는 공개하지 않는다.

## 현실 앵커와 게임 추상화

### 현실 앵커

현실의 연기는 연소가 만든 입자와 기체가 섞인 복합 plume이며, 주변 흐름과 온도에 따라 이동하고 확산된다.

### 게임 추상화

입자 크기, 공기 조성, 난류, 독성 농도를 계산하지 않는다. Smoke는 한 Cell을 차지하는 공통 GAS movement identity와 유한 decay descriptor로 표현한다. 숨은 Air가 이동을 운반하지 않는다.

### 창작 보강

Smoke가 일정 lifecycle 뒤 곧바로 EMPTY로 변하는 것은 world cleanup과 읽기 쉬운 transient behavior를 위한 게임 규칙이다. 향후 Ash/Soot를 추가하더라도 현재 Smoke에 자동으로 고체 잔여물을 약속하지 않는다.

## 구현 개요

- Canonical Wiki ID: `smoke`
- Engine Material ID: `7`
- Movement class: `GAS`
- 현재 구현: Wood/Oil combustion의 spawn result, GAS movement와 density property, generic finite decay to EMPTY
- Rule ownership: combustion source가 spawn을 요청하고 destination ownership pass가 생성 위치를 해결한다.
- State-cost policy: 별도 plume vector나 농도 field 없음
- Code evidence: [Material ID constants](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs#L28-L47), [Smoke descriptor](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs#L261-L275)

커밋 `177879c2e2e916066f376a4465c00430a0cdd8ac`에서 identity와 decay descriptor가 구현되어 있음을 확인했다. 이는 개별 Material이 `validated`되었거나 player-facing content가 완성되었다는 뜻이 아니다.

## 실패 모드 / 카운터

Smoke가 영구히 쌓여 world를 막거나, source 없이 생성되거나, 여러 source가 같은 Cell에 중복 생성하면 안 된다. Steam과 행동·lifecycle 차이가 보이지 않거나 flame처럼 실제 위치와 무관하게 그려져도 정체성이 약해진다.

## 미결정 사항

- [ ] Smoke는 일반 플레이어 palette에서 직접 선택 가능한가, combustion 결과로만 발견되는가?
- [ ] 향후 Ash/Soot가 필요할 때 Smoke decay와 어떤 관계로 분리할 것인가?
- [ ] filtration/capture가 추가되기 전에도 Steam과의 차이가 충분히 읽히는가?

## 관련 문서

- [Foundation Material index](README.md)
- [Material Wiki](../README.md)
- [Wood](wood.md)
- [Oil](oil.md)
- [Steam](steam.md)
- [Carbon Dioxide](../p1/carbon-dioxide.md)
- [Authoritative User Vision](../../../vision/USER_VISION.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
