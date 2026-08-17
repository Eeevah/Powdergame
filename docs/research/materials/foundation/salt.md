---
title: Salt
type: material
id: salt
aliases:
  - Soluble Salt
family: soluble-mineral
status: candidate
implementation_state: not_registered
movement_class: POWDER
palette_policy: deferred_until_adoption
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../../vision/USER_VISION.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../specs/SIMULATION_SPEC.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - ../../derived/COMMON_SENSE_MATERIAL_CANDIDATE_POOL.md
  - ../../encyclopedia/01A_FOUNDATION_CATALOG.md
tags:
  - chemistry
  - solution
  - powder
  - foundation
  - initial-catalog-direction
---

# Salt

> 물속에서 사라져도, 물이 다음에 할 일을 바꾸어 놓는 결정 가루.

## 개념

현실의 여러 수용성 염 가운데 Powdergame에 필요한 `dissolve → solution changes behavior` 문법을 대표하는 generic crystalline Powder다. 정확한 화합물 하나보다 [Water](water.md)를 future Brine state로 바꾸는 상식적 archetype에 가깝다.

Salt는 현재 [Material Specification](../../../specs/MATERIAL_SPEC.md)의 **initial catalog direction**이며, 아직 identity·Brine·dissolution Rule 어느 것도 Registry에 등록되지 않았다.

## 왜 넣는가

Salt는 Powder가 단지 쌓이는 데서 끝나지 않고 Liquid의 정체성과 이후 상변화·부식 조건을 바꾸게 한다. [Ice](ice.md), Water, future Metal corrosion을 하나의 값싼 solution family로 연결한다.

## 핵심 동사

```text
DISSOLVE
CREATE A SOLUTION STATE
ALTER FREEZING / CORROSION CONDITIONS
```

## 플레이어 직관

플레이어는 Salt가 Water에 닿으면 눈에 보이는 결정은 줄어들지만 물의 성질은 남아서 달라진다고 예상할 수 있다. 단순히 EMPTY가 되는 대신 Brine 같은 관찰 가능한 staged identity가 결과를 이어야 한다.

## 세계 안의 역할

- soluble mineral input
- solution-state source
- Water/Ice interaction hub
- future corrosion modifier
- historical/alchemical foundation archetype

## 대표 인과 사슬

```text
future candidate:
Salt + Water
→ Brine

Brine + cold
→ altered freezing behavior

Brine + eligible Metal / environment
→ accelerated corrosion candidate
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Water](water.md) | dissolution medium | paired local rules로 Brine staged identity를 만들 후보. |
| [Ice](ice.md) | phase contrast | Brine의 동결 성격 차이가 실제로 읽혀야 solution identity가 의미를 얻는다. |
| [Metal](metal.md) | future modifier target | 구체 Metal corrosion prototype에서 환경 조건을 바꿀 후보. |
| [Sand](sand.md) | movement contrast | 둘 다 POWDER지만 Salt는 용액 상태와 후속 조건을 만든다. |
| Brine | future result | 농도 float 대신 discrete Material identity로 시작할 후보. |

## 독립 Material인 이유

Sand는 이동·퇴적이 주된 Powder이고 Salt는 Water에 녹아 Liquid의 다음 행동을 바꾼다. Salt가 단순히 사라지고 Water가 그대로라면 독립 Material이 아니라 시각 효과에 그친다.

## Palette / Discovery 정책

- **Palette:** `deferred_until_adoption`
- Salt/Brine Rule과 observable difference가 채택되기 전에는 player palette에 노출하지 않는다.
- adoption 뒤 Salt는 source 후보지만 Brine의 freezing/corrosion 효과는 실험으로 발견하게 한다.
- Dictionary 후보 문장: “소금은 물속에서 보이지 않게 되어도 물의 성질을 바꾼다.”

## 현실 앵커와 게임 추상화

### 현실 앵커

수용성 염은 물에 녹아 용액을 만들고, 염수는 순수한 물과 다른 동결·부식 환경을 형성할 수 있다.

### 게임 추상화

용해도, 농도, 이온, 포화도와 각 염의 차이를 연속값으로 계산하지 않는다. Salt와 Water의 local transition으로 discrete Brine identity를 만드는 방향을 우선한다.

### 창작 보강

Brine의 정확한 동결 변화, 부식 가속 대상, Salt 소비 균형과 재결정 조건은 gameplay prototype에서 정할 규칙이다.

## 구현 개요

- Movement class: `POWDER` 방향
- descriptor-level properties: shared POWDER family와 solution 결과 descriptor 후보
- Rule owner: future Water/Salt paired self-rule
- update tier: 접촉면에서만 활성인 local dissolution 후보
- state-cost policy: 모든 Liquid Cell에 dissolved-salt 농도나 mixture amount를 추가하지 않음
- current Rule Card: 없음

현재 Registry에는 Salt 또는 Brine identity가 없고 dissolution Rule도 없다. 이 페이지는 `candidate / not_registered`인 initial catalog direction을 설명한다.

## 실패 모드 / 카운터

Salt가 Water의 변화 없이 사라지거나, 한 결정이 무한한 Water 전체를 즉시 Brine으로 바꾸면 인과와 소비 경계가 읽히지 않는다. local contact, 유한한 전이 범위와 분명한 Brine 결과가 필요하다.

## 미결정 사항

- [ ] Brine을 독립 discoverable Material로 채택할 것인가?
- [ ] Salt/Water paired self-rule의 소비 균형은 어떻게 보일 것인가?
- [ ] 첫 prototype은 altered freezing과 corrosion 중 무엇을 증명할 것인가?
- [ ] 재결정/증발 chain은 언제 다룰 것인가?

## 관련 문서

- [Foundation family index](README.md)
- [Material Wiki](../README.md)
- [Water](water.md)
- [Ice](ice.md)
- [Metal](metal.md)
- [User Vision](../../../vision/USER_VISION.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Common-Sense Material Candidate Pool](../../derived/COMMON_SENSE_MATERIAL_CANDIDATE_POOL.md)
