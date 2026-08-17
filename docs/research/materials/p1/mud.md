---
title: Mud
type: material
id: mud
aliases:
  - Wet Soil
family: soil
status: prototype
implementation_state: not_registered
movement_class: LIQUID
palette_policy: hidden_until_discovered_debug_spawnable
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

# Mud

> 물은 흙을 없애지 않는다. 흙이 흐르게 만든다.

## 개념

Dirt가 Water와 접촉해 이동 성격이 바뀐 staged result다. 현실의 모든 진흙을 하나의 혼합물로 재현하지 않고, **무겁고 느리게 흐르는 젖은 토양**을 대표한다.

## 왜 넣는가

Dirt + Water가 단순 색 변화로 끝나지 않고 Movement family 자체를 바꾸게 한다. 플레이어는 지형 붕괴, 수로 막힘, 침전, 건조를 한 결과물로 실험할 수 있다.

## 핵심 동사

```text
FLOW SLOWLY
SINK THROUGH WATER
DRY → DIRT
```

## 플레이어 직관

Water보다 탁하고 느리며, 가만히 두거나 데우면 다시 흙이 될 것으로 예상할 수 있어야 한다.

## 세계 안의 역할

- staged result
- slow terrain liquid
- sedimentation toy
- reversible environmental state

## 대표 인과 사슬

```text
Dirt + Water → Mud

Mud settles below Water

Mud + warmth + no Water → Dirt
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| Dirt | source/reverse | Dirt의 젖은 결과이며 건조하면 돌아간다. |
| Water | formation environment | 접촉이 유지되면 젖은 상태를 지지한다. |
| Heat | drying driver | 충분히 따뜻하고 Water와 떨어지면 Dirt로 전이한다. |
| Sand | movement contrast | Sand는 가루, Mud는 느린 액체로 다르게 움직여야 한다. |

## 독립 Material인 이유

per-cell wetness가 아니라 Material identity로 움직임과 규칙이 달라지므로 staged Material이 필요하다. 다만 Water와 구별되지 않으면 Dirt의 상태/variant로 병합한다.

## Palette / Discovery 정책

- **Palette:** `hidden_until_discovered_debug_spawnable`
- **Player Dictionary:** “Water can make loose earth flow.”
- 정확한 threshold, rule priority, 남은 discovery 개수는 공개하지 않는다.

## 현실 앵커와 게임 추상화

### 현실 앵커

토양과 물의 혼합은 흐름성과 침강·건조 거동을 만든다.

### 게임 추상화

점도·함수율·입도 분포를 계산하지 않고 저이동성 LIQUID descriptor와 느린 건조 Rule로 표현한다.

### 창작 보강

건조 시 증기/질량 보존을 강제하지 않는 것은 게임 추상화다.

## 구현 개요

- Movement class: shared LIQUID with LOW mobility seed
- Rule owner: Mud
- Drying rule: warm + no Water → Dirt
- State-cost policy: 수분 progress 없음
- Rule Card: P1-MUD-001

수치·threshold·density rank는 이 페이지에서 확정하지 않는다. 최신 Rule Card가 튜닝 source다.

## 실패 모드 / 카운터

Water처럼 빠르게 퍼지면 독립성이 없다. 건조와 젖음이 매 Tick 왕복하는 flicker도 금지한다.

## 미결정 사항

- [ ] 공통 mobility tier만으로 충분히 진흙처럼 보이는가?
- [ ] 건조 hysteresis가 필요한가?

## 관련 문서

- [P1 family index](README.md)
- [Material Wiki](../README.md)
- [Foundation: Water](../foundation/water.md)
- [Foundation: Sand](../foundation/sand.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
