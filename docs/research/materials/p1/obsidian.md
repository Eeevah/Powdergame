---
title: Obsidian
type: material
id: obsidian
aliases:
  - Volcanic Glass
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

# Obsidian

> 용암이 너무 빨리 식어서 돌보다 유리에 가까워진 검은 상처.

## 개념

Lava가 Water/Ice 또는 큰 국소 온도차로 급랭될 때 생기는 volcanic glass archetype이다.

## 왜 넣는가

`Lava + Water`를 단순 pair recipe로 만들지 않고, Temperature와 현재 이웃 조건이 다른 결과를 선택하게 한다. 플레이어가 냉각 환경을 설계하도록 만든다.

## 핵심 동사

```text
RAPID QUENCH
FORM GLASSLIKE ROCK
REMELT / BRITTLE
```

## 플레이어 직관

뜨거운 Lava를 갑자기 식히면 검고 유리질인 고체가 생기며, 단단하지만 충격에는 깨질 수 있다고 예상할 수 있다.

## 세계 안의 역할

- rapid-cooling result
- geological discovery
- brittle structure
- cooling experiment reward

## 대표 인과 사슬

```text
Lava + rapid local quench → Obsidian

Water near Lava → Water may become Steam through its own rule

Obsidian + extreme Heat → Lava
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| Lava | source/remelt result | 급랭 조건에서 생성되고 극열에서 돌아간다. |
| Water / Ice | quench environment | 전용 recipe가 아니라 급랭 조건을 제공한다. |
| Basalt | contrast | 같은 Lava의 일반 냉각 결과. |
| Pressure | future counter | 유리질 취성을 보여줄 수 있는 파괴 조건. |

## 독립 Material인 이유

Basalt와 동일한 구조재라면 필요 없다. 급랭이라는 생성 경로와 glasslike/brittle 성격이 읽혀야 한다.

## Palette / Discovery 정책

- **Palette:** `unlock_after_discovery`
- **Player Dictionary:** “Rapid cooling makes Lava choose a glass-like path.”
- 정확한 threshold, rule priority, 남은 discovery 개수는 공개하지 않는다.

## 현실 앵커와 게임 추상화

### 현실 앵커

흑요석은 용융된 규산질 물질이 빠르게 식어 결정화가 충분히 진행되지 못한 화산유리다.

### 게임 추상화

과거 냉각 속도를 저장하지 않고 Water/Ice 접촉 또는 local ΔT를 cheap proxy로 사용한다.

### 창작 보강

한 임계조건으로 급랭을 판정하는 것은 게임용 근사다.

## 구현 개요

- Movement class: STATIC
- Rule owner: Obsidian for remelting
- Produced by Lava rapid-quench rule before ordinary solidification
- State-cost policy: cooling history 없음
- Rule Cards: P1-LAVA-001, P1-OBSIDIAN-001

수치·threshold·density rank는 이 페이지에서 확정하지 않는다. 최신 Rule Card가 튜닝 source다.

## 실패 모드 / 카운터

한 개의 차가운 Cell이 멀리 있는 Lava 전체를 Obsidian으로 바꾸면 안 된다. Basalt와 실제 결과 비율이 구분되어야 한다.

## 미결정 사항

- [ ] P1에서 brittle pressure class만으로 충분한가?
- [ ] 열충격 파쇄는 어느 prototype에서 추가할 것인가?

## 관련 문서

- [P1 family index](README.md)
- [Material Wiki](../README.md)
- [Foundation: Lava](../foundation/lava.md)
- [Foundation: Water](../foundation/water.md)
- [Foundation: Ice](../foundation/ice.md)
- [Foundation: Steam](../foundation/steam.md)
- [Foundation: Glass](../foundation/glass.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
