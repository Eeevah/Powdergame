---
title: Steam
type: material
id: steam
aliases:
  - Water Vapor
family: water-phase
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
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - ../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md
  - "https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs#L243-L260"
tags:
  - foundation
  - water-phase
  - gas
  - pressure
  - implemented
---

# Steam

> 물이 열을 받아 떠오르고, 갇히면 세계를 밀어내는 숨.

## 개념

Steam은 가열된 Water가 변한 GAS Matter다. 단순한 흰색 효과가 아니라 local movement, 냉각에 따른 응축, 제한된 공간에서의 expansion과 Pressure chain을 이어 주는 Water-family result identity다.

## 왜 넣는가

Steam이 없으면 Water 가열은 온도 숫자만 바꾸고 다른 시스템으로 이어지지 않는다. Steam은 phase transition을 이동, Pressure, rupture, vent로 연결해 작은 규칙이 큰 현상을 만드는 Foundation의 대표 chain을 만든다.

## 핵심 동사

```text
RISE
EXPAND
PRESSURIZE
CONDENSE
```

## 플레이어 직관

Water를 가열하면 Steam이 생겨 위쪽과 빈 공간으로 움직이며, 빠져나갈 곳이 부족하면 Pressure를 만들 수 있다고 예상할 수 있다. 식으면 다시 Water로 돌아온다.

## 세계 안의 역할

- Water-family의 GAS phase
- phase-expansion result
- Pressure source chain의 매개
- vent와 enclosure 실험용 Matter
- 다른 Gas identity의 movement/density 비교 기준

## 대표 인과 사슬

```text
Water + Heat
→ Steam transition / expansion request
→ 공간 부족
→ Pressure
→ 약한 구조 파열
→ Steam vent
→ 주변 Temperature와 Matter movement 변화
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Water](water.md) | phase source/result | 가열로 생성되고 냉각으로 응축한다. |
| [Ice](ice.md) | extended phase family | Water를 사이에 둔 STATIC phase다. |
| Pressure | generated field | expansion이 막힐 때 다음 인과계를 시작한다. |
| [Wood](wood.md) | rupture/vent structure | 약한 구조를 통한 pressure relief chain을 보여준다. |
| [Stone](stone.md) | resistant control | 같은 압력 환경에서 버티는 구조 대조군이다. |
| [Smoke](smoke.md) | Gas contrast | 둘 다 GAS지만 생성 원인과 lifecycle이 다르다. |
| [Carbon Dioxide](../p1/carbon-dioxide.md) | future Gas contrast | P1 후보 Gas의 density/movement 차이를 설명하는 기준이다. |
| [Obsidian](../p1/obsidian.md) | future quench neighbor | Lava 주변 Water가 자기 규칙으로 Steam이 될 수 있지만 Obsidian 전용 recipe는 아니다. |

## 독립 Material인 이유

Steam을 Water의 시각 효과로만 만들면 GAS movement, spatial ownership, condensation, expansion Pressure를 Cell truth로 표현할 수 없다. Smoke와도 달리 Water로 되돌아가는 phase identity다.

## Palette / Discovery 정책

- **Palette:** `result_identity_pending_player_palette`
- 현재 구현에는 일반 플레이어용 Material palette가 없다.
- Steam은 debug/demo에서 생성·표시되지만, 그 노출은 플레이어가 처음부터 직접 선택할 수 있다는 뜻이 아니다.
- **Player Dictionary:** “물을 가열하면 떠오르는 증기가 생기고, 갇힌 증기는 주변을 밀어낸다.”
- 정확한 transition, yield, Pressure 수치는 공개하지 않는다.

## 현실 앵커와 게임 추상화

### 현실 앵커

물은 가열되면 기체 상태로 바뀔 수 있고 냉각되면 다시 응축한다. 제한된 공간의 증기 생성은 압력을 높여 구조에 힘을 줄 수 있다.

### 게임 추상화

실제 유체역학, 수증기 분압, 잠열, 공기 혼합을 계산하지 않는다. Steam은 한 Cell을 차지하는 GAS Matter이며 local movement와 gameplay Pressure rule로 거동한다. EMPTY는 숨은 공기나 압력 매질이 아니다.

### 창작 보강

phase expansion의 Matter yield와 막힌 expansion을 Pressure로 바꾸는 정책은 읽기 쉬운 chain을 위한 게임 규칙이다. 정확한 전역 질량·에너지 회계보다 local 인과의 일관성을 우선한다.

## 구현 개요

- Canonical Wiki ID: `steam`
- Engine Material ID: `6`
- Movement class: `GAS`
- 현재 구현: Water의 boiling result, Steam → Water thermal transition, GAS movement와 density property, expansion/Pressure chain 참여
- State-cost policy: 별도 per-cell velocity나 공기 조성 없음
- Code evidence: [Material ID constants](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs#L28-L47), [Steam descriptor](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs#L243-L260)

커밋 `177879c2e2e916066f376a4465c00430a0cdd8ac`에서 identity와 phase descriptor가 구현되어 있음을 확인했다. subsystem evidence가 존재하더라도 이 페이지에서 개별 Product Gate나 `validated` 상태를 선언하지 않는다.

## 실패 모드 / 카운터

Steam이 안정된 bulk에서도 영원히 흔들리거나, 좁은 공간을 순간 이동해 빠져나가거나, Pressure 없이 장식 효과로만 보이면 핵심 역할이 무너진다. 냉각된 Steam이 Water로 돌아오지 않거나 EMPTY가 압력을 운반하는 숨은 매질이 되어도 안 된다.

## 미결정 사항

- [ ] 일반 플레이어 palette에서 Steam을 직접 선택하게 할지 Water phase 발견 결과로만 노출할지 결정했는가?
- [ ] Gas plume의 시각적 인공물을 Presentation에서 어떻게 완화할 것인가?
- [ ] 응축과 vent chain을 플레이어가 threshold 공개 없이 충분히 읽을 수 있는가?

## 관련 문서

- [Foundation Material index](README.md)
- [Material Wiki](../README.md)
- [Water](water.md)
- [Ice](ice.md)
- [Smoke](smoke.md)
- [Wood](wood.md)
- [Stone](stone.md)
- [Obsidian](../p1/obsidian.md)
- [Carbon Dioxide](../p1/carbon-dioxide.md)
- [Authoritative User Vision](../../../vision/USER_VISION.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
