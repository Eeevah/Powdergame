---
title: Clay
type: material
id: clay
aliases:
  - Dry Clay
family: mineral-manufacture
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
  - mineral-manufacture
  - p1
  - interaction
---

# Clay

> 젖으면 형태를 얻고, 불을 만나면 다시 흙으로 돌아가지 않는 재료가 된다.

## 개념

현실의 점토광물군을 하나의 게임 archetype으로 압축한 광물성 Powder다. 핵심은 화학 조성이 아니라 **물로 성형되고 열로 소성되는 원료**라는 점이다.

## 왜 넣는가

자연 Matter가 플레이어가 만든 구조재로 바뀌는 첫 제조 문법을 연다. 제작 메뉴 없이 세계의 Water와 Heat가 작업대와 가마 역할을 한다.

## 핵심 동사

```text
WET → SHAPE-HOLDING
FIRE → BRICK
DRY / REWET
```

## 플레이어 직관

흙과 비슷한 가루지만 물을 섞으면 형태를 잡을 수 있고, 강하게 구우면 단단해진다고 예상할 수 있다.

## 세계 안의 역할

- mineral powder
- manufacturing input
- Heat/Water bridge
- early construction economy

## 대표 인과 사슬

```text
Clay + Water → Wet Clay

Clay + sufficient Heat → Brick

Wet Clay + sufficient Heat → Brick
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| Water | plasticization input | Clay 자신이 Wet Clay로 전이한다. |
| Wet Clay | staged result | 형태를 유지하는 젖은 중간재. |
| Heat | manufacturing driver | 충분한 열에서 Brick으로 비가역 전이. |
| Brick | manufactured output | 자연 원료가 만든 구조재. |

## 독립 Material인 이유

Dirt도 물에 젖지만 Mud로 흘러간다. Clay는 Water를 만나 오히려 형태를 잡고 Heat에서 Brick이 되는 제조 동사가 있어 별도 identity를 얻는다.

## Palette / Discovery 정책

- **Palette:** `visible_after_adoption`
- **Player Dictionary:** “Enough Heat permanently changes Clay into a structural material.”
- 정확한 threshold, rule priority, 남은 discovery 개수는 공개하지 않는다.

## 현실 앵커와 게임 추상화

### 현실 앵커

점토는 물과 함께 성형 가능해지고 소성으로 세라믹성 재료가 된다.

### 게임 추상화

가소성, 수축, 소결 진행도를 계산하지 않고 Clay → Wet Clay → Brick 단계로 표현한다.

### 창작 보강

정확한 소성 온도와 직접 dry-Clay firing 허용은 게임 설계다.

## 구현 개요

- Movement class: POWDER
- Rule owner: Clay
- Ordered rules: high Heat → Brick; else Water → Wet Clay
- State-cost policy: firing/wetness progress 없음
- Rule Cards: P1-CLAY-001, P1-CLAY-002

수치·threshold·density rank는 이 페이지에서 확정하지 않는다. 최신 Rule Card가 튜닝 source다.

## 실패 모드 / 카운터

Wet Clay와 Brick이 단순히 Clay의 색만 바뀐 모습이면 안 된다. Water contact가 high-Heat firing보다 우선해서 소성이 막혀서도 안 된다.

## 미결정 사항

- [ ] Dry Clay 직접 소성을 유지할 것인가?
- [ ] Clay와 Dirt의 Powder 움직임 차이가 필요한가?

## 관련 문서

- [P1 family index](README.md)
- [Material Wiki](../README.md)
- [Foundation: Water](../foundation/water.md)
- [Foundation: Sand](../foundation/sand.md)
- [Foundation: Stone](../foundation/stone.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
