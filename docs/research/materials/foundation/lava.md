---
title: Lava
type: material
id: lava
aliases:
  - Molten Rock
family: volcanic
status: candidate
implementation_state: not_registered
movement_class: LIQUID
palette_policy: deferred_until_adoption
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../../vision/USER_VISION.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../specs/SIMULATION_SPEC.md
  - ../../../specs/REACTION_SPEC.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - ../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md
  - ../../derived/COMMON_SENSE_MATERIAL_CANDIDATE_POOL.md
  - ../../encyclopedia/01A_FOUNDATION_CATALOG.md
tags:
  - volcanic
  - thermal
  - liquid
  - foundation
  - initial-catalog-direction
---

# Lava

> 흐르는 돌이면서, 닿는 환경이 어떤 돌을 남길지 선택하게 하는 열의 강.

## 개념

현실의 다양한 용융 암석을 하나의 gameplay archetype으로 압축한 hot LIQUID Matter 후보다. Heat source, 흐르는 지형 위험, 냉각 결과의 source identity를 함께 맡는다.

Lava는 [Material Specification](../../../specs/MATERIAL_SPEC.md)의 **initial catalog direction**이다. P1 Rule Card가 Basalt/Obsidian 분기를 구체화했지만 현재 Registry에는 Lava가 없고 어떤 volcanic Rule도 실행되지 않는다.

## 왜 넣는가

Lava는 Temperature를 단순 색상이나 피해가 아니라 지질 변환의 원인으로 만든다. 같은 source가 보통 냉각과 급랭에서 다른 결과를 내게 해 [Basalt](../p1/basalt.md), [Obsidian](../p1/obsidian.md), [Water](water.md), [Steam](steam.md), Pressure를 한 실험으로 잇는다.

## 핵심 동사

```text
FLOW AS HOT ROCK
TRANSFER HEAT
SOLIDIFY BY COOLING CONTEXT
REMELT
```

## 플레이어 직관

플레이어는 Lava가 천천히 흐르며 주변을 가열하고, 식으면 돌이 된다고 예상할 수 있다. Water나 Ice로 갑자기 식힐 때는 보통 냉각과 다른 유리질 결과가 생긴다는 점이 숨은 발견이다.

## 세계 안의 역할

- volcanic source
- mobile Heat carrier
- geological transformation hub
- Water/Steam/Pressure connector
- Basalt/Obsidian discovery source

## 대표 인과 사슬

```text
P1 candidate:
Lava + ordinary local cooling
→ Basalt

Lava + rapid Water / Ice / local-temperature quench
→ Obsidian

Water near hot Lava
→ Water's own Steam transition
→ blocked expansion may create Pressure

Basalt / Obsidian + extreme Heat
→ Lava
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Water](water.md) / [Ice](ice.md) | quench environment | 전용 pair recipe가 아니라 급격한 local cooling 조건 후보다. |
| [Basalt](../p1/basalt.md) | ordinary-cooling result | 보통 냉각 맥락을 기록하는 volcanic rock 후보. |
| [Obsidian](../p1/obsidian.md) | rapid-quench result | 급랭 맥락과 glasslike/brittle 성격을 기록할 후보. |
| [Steam](steam.md) | neighboring phase result | Water가 자기 phase Rule로 바뀌어 후속 Pressure chain을 열 수 있다. |
| [Stone](stone.md) | baseline contrast | 모든 냉각 결과를 generic Stone 하나로 압축하지 않는 이유를 보여준다. |

## 독립 Material인 이유

Water는 기준 액체이고 Oil은 가연성 액체다. Lava는 이동 자체가 Heat와 지질 전이를 운반하며, 냉각 조건에 따라 STATIC result family를 선택한다. 이 분기가 읽히지 않으면 Lava는 뜨거운 색의 Liquid에 그친다.

## Palette / Discovery 정책

- **Palette:** `deferred_until_adoption`
- identity와 thermal/phase Rule이 채택되기 전에는 player palette에 노출하지 않는다.
- adoption 뒤 Lava는 source 후보가 될 수 있지만 Basalt/Obsidian은 세계에서 냉각 결과를 관찰한 뒤 발견·노출하는 방향이다.
- Dictionary 후보 문장: “용암은 식는 속도와 주변에 따라 서로 다른 돌을 남긴다.”

## 현실 앵커와 게임 추상화

### 현실 앵커

용융 암석은 냉각해 화산암을 만들며, 조성과 냉각 조건은 결과 조직에 영향을 준다. 급랭된 규산질 용융물은 volcanic glass를 만들 수 있다.

### 게임 추상화

조성, 결정 성장, 점도와 연속 냉각 이력을 저장하지 않는다. 현재 local Temperature 차이와 Water/Ice 접촉을 cheap proxy로 사용해 ordered transition을 선택하는 방향이다.

### 창작 보강

Basalt/Obsidian을 가르는 정확한 조건, remelt 경계, 구조·열 차이는 gameplay tuning이다. P1의 두 결과는 현실 지질학 전체가 아니라 플레이어가 읽을 수 있는 분기다.

## 구현 개요

- Movement class: `LIQUID` 방향
- descriptor-level properties: hot liquid의 이동·열·density 성격은 adoption 뒤 결정
- Rule owner: P1 연구에서는 Lava가 ordered solidification self-rules를 소유
- update tier: active thermal frontier에서 평가하는 phase transition 후보
- state-cost policy: 모든 Cell에 cooling-history 값을 추가하지 않음
- current Rule Cards: [P1-LAVA-001 / P1-LAVA-002 candidates](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md#p1-lava-001--rapid-quench)

현재 Registry에는 Lava identity가 없고 rapid-quench, ordinary-solidification, remelting Rule도 없다. Rule Cards는 비권위 연구이며 상태는 `candidate / not_registered`다.

## 실패 모드 / 카운터

보통 냉각과 급랭이 같은 결과를 만들거나, 멀리 있는 한 개의 차가운 Cell이 큰 Lava 영역을 바꾸면 분기가 읽히지 않는다. 별도 `Lava + Water recipe`를 하드코딩해 Water의 phase Rule과 중복시키는 것도 피한다.

안정된 냉각 결과가 계속 thermal work를 만들거나 Obsidian이 Basalt의 재색칠에 그치면 해당 identity를 merge/demote해야 한다.

## 미결정 사항

- [ ] local quench proxy가 cooling context를 충분히 읽히게 하는가?
- [ ] Basalt와 Obsidian은 어떤 구조·열 차이를 가져야 하는가?
- [ ] Lava의 기본 이동성이 기존 LIQUID family만으로 충분한가?
- [ ] adoption 뒤 player source palette 노출 시점은 언제인가?

## 관련 문서

- [Foundation family index](README.md)
- [Material Wiki](../README.md)
- [Water](water.md)
- [Ice](ice.md)
- [Steam](steam.md)
- [Basalt](../p1/basalt.md)
- [Obsidian](../p1/obsidian.md)
- [P1 family index](../p1/README.md)
- [User Vision](../../../vision/USER_VISION.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
