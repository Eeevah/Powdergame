---
title: Acid
type: material
id: acid
aliases:
  - Generic Acid
family: reactive-liquid
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
  - ../../encyclopedia/01A_FOUNDATION_CATALOG.md
tags:
  - chemistry
  - reactive-liquid
  - foundation
  - initial-catalog-direction
---

# Acid

> 닿는 것을 모두 지우는 액체가 아니라, 무엇이 반응할지를 드러내는 선택적 용해제.

## 개념

현실의 특정 산 하나를 그대로 재현하지 않고, 선택적 용해·부식·중화라는 행동을 묶은 게임용 reactive liquid archetype이다.

Acid는 현재 [Material Specification](../../../specs/MATERIAL_SPEC.md)에 남아 있는 **initial catalog direction**이다. 개념 페이지가 존재하거나 P1 Rule Card가 Acid를 입력으로 사용한다는 사실은 identity 등록 또는 구현 약속이 아니다.

## 왜 넣는가

Water와 Oil만으로는 액체가 다른 Matter의 정체성을 시험하는 화학 문법이 비어 있다. Acid는 안정된 Stone과 반응성 [Limestone](../p1/limestone.md), 일반 Metal과 부식 가능한 구체 Metal을 구분하고, 단순 삭제보다 다음 결과를 남기는 반응을 열 수 있다.

## 핵심 동사

```text
SELECTIVELY DISSOLVE
NEUTRALIZE
REVEAL GAS / CORRODE ELIGIBLE MATTER
```

## 플레이어 직관

플레이어는 Acid가 위험한 액체라고 예상할 수 있다. 실험 뒤에는 모든 벽을 똑같이 지우는 것이 아니라, 반응 가능한 광물·금속에서 서로 다른 결과와 부산물을 만든다는 점을 발견해야 한다.

## 세계 안의 역할

- chemistry hub
- selective terrain modifier
- reactive-mineral probe
- future corrosion driver
- counter with finite consumption

## 대표 인과 사슬

```text
P1 candidate:
Limestone + Acid
→ Carbon Dioxide + neutralized liquid abstraction

future:
eligible Metal + Acid
→ material-owned corrosion transition
→ changed structural / thermal behavior
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Limestone](../p1/limestone.md) | prototype reactant | P1 연구에서는 기체를 드러내는 선택적 탄산염 반응의 상대다. |
| [Carbon Dioxide](../p1/carbon-dioxide.md) | prototype output | 고체 삭제로 끝나지 않는 이동 가능한 Gas 결과 후보다. |
| [Water](water.md) | neutralized-liquid abstraction | P1 연구의 값싼 pair 결과이며 현실의 완전한 생성물 목록을 뜻하지 않는다. |
| [Metal](metal.md) | future eligible target | 구체 Metal family가 채택된 뒤 material-owned corrosion 후보가 된다. |
| [Stone](stone.md) | baseline contrast | 모든 구조재를 같은 속도로 지우지 않는 선택성을 보여줄 기준이다. |

## 독립 Material인 이유

Water는 이동·상변화의 기준 액체이고 Oil은 층분리·연소의 액체 연료다. Acid는 접촉한 상대의 반응 자격을 시험하고 자신도 중화·소모될 수 있다는 별도 동사를 가진다. 이 선택성과 결과물이 없다면 Acid는 색이 다른 삭제 도구에 불과하다.

## Palette / Discovery 정책

- **Palette:** `deferred_until_adoption`
- adoption과 실제 반응 구현 전에는 일반 플레이어 palette에 노출하지 않는다.
- 향후 Dictionary는 “일부 밝은 암석은 Acid를 만나 무거운 Gas를 내놓는다”처럼 관찰된 현상만 기록한다.
- 정확한 대상 목록, 우선순위, 반응량은 처음부터 답안처럼 공개하지 않는다.

## 현실 앵커와 게임 추상화

### 현실 앵커

산은 물질에 따라 서로 다른 반응을 보이며, 탄산염과의 반응은 이산화탄소 발생으로 관찰될 수 있다. 금속의 산 반응 역시 금속과 환경에 따라 달라진다.

### 게임 추상화

pH, 농도, 이온, 용해된 염과 화학량론을 추적하지 않는다. 각 대상 Material이 local neighbor를 읽고 자기 전이를 결정하는 작은 규칙으로 선택적 반응을 표현한다.

### 창작 보강

P1의 `Limestone Cell + Acid Cell → CO2 Cell + Water Cell`은 gameplay를 위한 cell-level bookkeeping 후보이다. 정확한 소비량, 반응 속도와 어떤 Metal이 eligible한지는 구현 증거와 사용자 판단이 필요한 Powdergame 규칙이다.

## 구현 개요

- Movement class: `LIQUID` 방향
- descriptor-level properties: 이동·밀도·열 성격과 반응 자격은 adoption 뒤 결정
- Rule owner: P1 연구에서는 Acid와 Limestone의 paired self-rule 후보
- update tier: 활성 접촉면에서만 평가하는 local reaction 후보
- state-cost policy: 모든 Cell에 acidity·concentration·capacity 값을 추가하지 않음
- current Rule Card: [P1-ACID-001 candidate](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md#p1-acid-001--carbonate-neutralization-abstraction)

현재 Registry에는 Acid identity가 없고 실행되는 Acid Rule도 없다. Rule Card는 비권위 연구이며 이 페이지의 상태는 `candidate / not_registered`다.

## 실패 모드 / 카운터

Acid가 상대를 가리지 않고 모든 Matter를 즉시 지우거나, 한 Cell이 거대한 벽을 한 Tick에 소모하면 세계 규칙이 아니라 만능 삭제 도구가 된다. 반응 자격, Acid의 중화·소모, 국소 접촉, 필요 시 해당 pair만의 ownership 조정이 카운터가 되어야 한다.

## 미결정 사항

- [ ] Foundation adoption 시 처음 허용할 반응 대상은 무엇인가?
- [ ] Limestone pair의 self-write fan-out가 관찰상 허용 가능한가?
- [ ] 구체 Metal corrosion은 어느 family prototype에서 시작할 것인가?
- [ ] adoption 뒤 source palette와 discovery-only 중 어느 정책이 적절한가?

## 관련 문서

- [Foundation family index](README.md)
- [Material Wiki](../README.md)
- [Limestone](../p1/limestone.md)
- [Carbon Dioxide](../p1/carbon-dioxide.md)
- [P1 family index](../p1/README.md)
- [User Vision](../../../vision/USER_VISION.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
