---
title: Basalt
type: material
id: basalt
aliases:
  - Volcanic Rock
family: volcanic-rock
status: prototype
implementation_state: not_registered
movement_class: STATIC
palette_policy: unlock_after_discovery
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../specs/REACTION_SPEC.md
tags:
  - volcanic-rock
  - p1
  - interaction
---

# Basalt

> 용암이 서두르지 않고 식었을 때 남기는 가장 평범한 화산암.

## 개념

Lava가 일반적인 국소 냉각 조건에서 굳은 결과를 대표하는 현실 기반 volcanic rock archetype이다.

## 왜 넣는가

현재 Lava가 식으면 모두 같은 Stone이 되는 문제를 해결하고, **냉각 맥락이 지질 결과에 기록되는 세계**를 만든다. Obsidian과 대비되어 냉각 조건을 실험하게 한다.

## 핵심 동사

```text
SOLIDIFY NORMALLY
RECORD COOLING CONTEXT
REMELT
```

## 플레이어 직관

Lava를 그냥 식히면 검은 화산암이 되고, 극단적으로 다시 데우면 Lava로 돌아간다고 예상할 수 있다.

## 세계 안의 역할

- geological result
- volcanic structure
- thermal history marker
- Obsidian contrast

## 대표 인과 사슬

```text
Lava + ordinary cooling → Basalt

Basalt + extreme Heat → Lava

Basalt + Water/Acid → future weathering candidates
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| Lava | source/remelt result | 일반 냉각에서 생성되고 극열에서 돌아간다. |
| Obsidian | contrast | 급랭 결과와 일반 냉각 결과를 구분한다. |
| Heat | reverse driver | 재용융 threshold에서 Lava 전이. |
| Pressure | structure | 상대적으로 단단한 rock class 후보. |

## 독립 Material인 이유

Stone과 완전히 같다면 결과물 이름만 늘어난다. Lava의 일반 냉각 경로와 Obsidian 대비가 실제로 읽힐 때 독립성을 얻는다.

## Palette / Discovery 정책

- **Palette:** `unlock_after_discovery`
- **Player Dictionary:** “Lava normally cools into dark volcanic rock.”
- 정확한 threshold, rule priority, 남은 discovery 개수는 공개하지 않는다.

## 현실 앵커와 게임 추상화

### 현실 앵커

현무암은 화산성 용융물이 냉각되어 만들어지는 대표 암석이다.

### 게임 추상화

결정 크기·조성·냉각 속도 연속값을 생략하고 ordered rule의 기본 결과로 표현한다.

### 창작 보강

정확한 냉각 threshold와 remelt threshold는 게임 수치다.

## 구현 개요

- Movement class: STATIC
- Rule owner: Basalt for remelting
- Produced by Lava ordinary-solidification rule
- State-cost policy: cooling history 없음
- Rule Cards: P1-LAVA-002, P1-BASALT-001

수치·threshold·density rank는 이 페이지에서 확정하지 않는다. 최신 Rule Card가 튜닝 source다.

## 실패 모드 / 카운터

Obsidian과 생성 비율이 구별되지 않거나 Stone의 재색칠처럼 보이면 demote/merge한다.

## 미결정 사항

- [ ] Stone보다 어떤 구조·열 차이를 줄 것인가?
- [ ] 재용융이 플레이에서 충분히 읽히는가?

## 관련 문서

- [P1 family index](README.md)
- [Material Wiki](../README.md)
- [Foundation: Lava](../foundation/lava.md)
- [Foundation: Stone](../foundation/stone.md)
- [Foundation: Water](../foundation/water.md)
- [Foundation: Acid](../foundation/acid.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
