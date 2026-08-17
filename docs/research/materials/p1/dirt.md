---
title: Dirt
type: material
id: dirt
aliases:
  - Soil
family: soil
status: prototype
implementation_state: not_registered
movement_class: POWDER
palette_policy: visible_after_adoption
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../specs/REACTION_SPEC.md
tags:
  - soil
  - p1
  - interaction
---

# Dirt

> 평범한 흙이지만, 물과 생명을 만나면 세계의 표면을 바꾸는 출발점.

## 개념

현실의 다양한 토양을 하나의 게임용 archetype으로 압축한 기본 지형 Matter다. 정확한 토성·유기물 함량을 재현하지 않고, **물을 만나 흐름성이 생기고 생명의 기질이 되는 흙**을 대표한다.

## 왜 넣는가

현재 기반에는 Stone과 Sand는 있지만, Water·Seed·Plant를 동시에 연결하는 익숙한 토양이 없다. Dirt는 지형, 수분, 성장, 건조를 한 family로 묶는 가장 상식적인 허브다.

## 핵심 동사

```text
SUPPORT GROWTH
WET → FLOW
DRY → LOOSE SOIL
```

## 플레이어 직관

처음에는 “식물이 자라는 흙”으로 이해된다. Water를 부으면 사라지는 것이 아니라 Mud로 변해 흐른다는 점이 첫 발견이다.

## 세계 안의 역할

- terrain
- growth substrate
- Water interaction hub
- manufacturing/ecology precursor

## 대표 인과 사슬

```text
Dirt + Water → Mud

Dirt + Seed + suitable Water → future growth substrate

Mud + warmth + no Water → Dirt
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| Water | input | Dirt 자신이 Mud로 전이한다. |
| Mud | result/reverse | 젖은 상태의 독립 staged identity. |
| Seed / Plant | future substrate | 생태 prototype에서 성장 조건을 제공할 후보. |
| Heat | indirect counter | Mud 건조를 통해 Dirt를 회복시킨다. |

## 독립 Material인 이유

Sand는 주로 낙하·퇴적의 가루이고, Dirt는 Water와 생명을 연결한다. 이 차이가 없다면 합쳐야 하지만 P1은 `soil substrate + wetting` 동사를 검증한다.

## Palette / Discovery 정책

- **Palette:** `visible_after_adoption`
- **Player Dictionary:** “Water can make loose earth flow.”
- 정확한 threshold, rule priority, 남은 discovery 개수는 공개하지 않는다.

## 현실 앵커와 게임 추상화

### 현실 앵커

토양은 물 함량에 따라 거동이 크게 달라지고 생물 성장의 기질이 된다.

### 게임 추상화

토양 종류와 연속 수분량은 생략하고 Dirt ↔ Mud 두 단계로 표현한다.

### 창작 보강

없음. 단, 정확한 건조 속도와 이동성은 게임 튜닝이다.

## 구현 개요

- Movement class: POWDER
- Rule owner: Dirt
- Water contact rule: Dirt → Mud
- State-cost policy: 별도 wetness 값 없음
- Rule Card: P1-DIRT-001

수치·threshold·density rank는 이 페이지에서 확정하지 않는다. 최신 Rule Card가 튜닝 source다.

## 실패 모드 / 카운터

Water가 멀리 있다는 이유만으로 전체 지형이 즉시 Mud가 되면 안 된다. 접촉 기반이며 안정된 Dirt는 Sleep 가능해야 한다.

## 미결정 사항

- [ ] Mud가 충분히 다른 움직임을 보이는가?
- [ ] Dirt를 초기 palette에 노출할 시점은 언제인가?

## 관련 문서

- [P1 family index](README.md)
- [Material Wiki](../README.md)
- [Foundation: Stone](../foundation/stone.md)
- [Foundation: Sand](../foundation/sand.md)
- [Foundation: Water](../foundation/water.md)
- [Foundation: Seed](../foundation/seed.md)
- [Foundation: Plant](../foundation/plant.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
