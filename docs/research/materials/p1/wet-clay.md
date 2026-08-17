---
title: Wet Clay
type: material
id: wet_clay
aliases:
  - Plastic Clay
family: mineral-manufacture
status: prototype
implementation_state: not_registered
movement_class: STATIC_BASELINE
palette_policy: hidden_until_discovered_debug_spawnable
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

# Wet Clay

> 흐르던 가루가 물을 기억해 형태를 잡고, 불을 기다리는 중간재.

## 개념

Clay가 Water와 접촉해 성형 가능한 상태가 된 staged result다. P1에서는 약한 STATIC으로 시작해 “가루 → 형태 유지”를 값싼 방식으로 표현한다.

## 왜 넣는가

Clay 제조 과정에서 플레이어가 직접 모양을 만들 수 있는 중간 단계를 제공한다. Water가 단순 recipe token이 아니라 취급 성격을 바꾸는 도구가 된다.

## 핵심 동사

```text
HOLD SHAPE
DRY → CLAY
FIRE → BRICK
```

## 플레이어 직관

젖어 있을 때는 원하는 모양을 잡고, 말리면 다시 가루가 되며, 충분히 구우면 영구 구조가 된다고 읽혀야 한다.

## 세계 안의 역할

- staged manufacturing material
- shape-holding intermediate
- reversible-before-firing state

## 대표 인과 사슬

```text
Clay + Water → Wet Clay

Wet Clay + moderate warmth + no Water → Clay

Wet Clay + high Heat → Brick
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| Clay | source/drying result | 젖음으로 생성되고 건조하면 돌아간다. |
| Water | formation/maintenance | 접촉이 Wet Clay 상태를 지지한다. |
| Heat | branch condition | 중간 열은 건조, 높은 열은 Brick 소성. |
| Brick | irreversible output | 소성 뒤 Water로 되돌아가지 않는다. |

## 독립 Material인 이유

movement/structure와 Rule order가 Clay와 다르므로 prototype staged identity로 둔다. 별도 재미가 없으면 Clay의 internal state로 축소할 수 있다.

## Palette / Discovery 정책

- **Palette:** `hidden_until_discovered_debug_spawnable`
- **Player Dictionary:** “Water lets Clay hold a shape before Fire makes it permanent.”
- 정확한 threshold, rule priority, 남은 discovery 개수는 공개하지 않는다.

## 현실 앵커와 게임 추상화

### 현실 앵커

젖은 점토는 성형 가능하고 건조·소성 과정에서 성질이 바뀐다.

### 게임 추상화

응집력 solver 없이 STATIC baseline으로 형태 유지를 근사한다.

### 창작 보강

STATIC이 너무 인공적이면 shared low-mobility LIQUID 대안을 시험한다.

## 구현 개요

- Movement class: weak STATIC baseline
- Rule owner: Wet Clay
- Ordered rules: high Heat → Brick; else warm/no Water → Clay
- State-cost policy: cohesion/wetness field 없음
- Rule Cards: P1-WET-CLAY-001, P1-WET-CLAY-002

수치·threshold·density rank는 이 페이지에서 확정하지 않는다. 최신 Rule Card가 튜닝 source다.

## 실패 모드 / 카운터

보통 Water 접촉만으로 영구 구조재가 되면 안 된다. STATIC이 지나치게 죽은 느낌이면 독립성이 아니라 구현 표현을 재검토한다.

## 미결정 사항

- [ ] STATIC과 저이동성 LIQUID 중 어느 쪽이 더 재미있는가?
- [ ] 건조 시 수축 시각화가 필요한가?

## 관련 문서

- [P1 family index](README.md)
- [Material Wiki](../README.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
