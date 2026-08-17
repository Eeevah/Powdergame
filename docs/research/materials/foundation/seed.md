---
title: Seed
type: material
id: seed
aliases:
  - Generic Seed
family: ecology
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
  - ecology
  - growth
  - foundation
  - initial-catalog-direction
---

# Seed

> 작고 움직이지만, 알맞은 자리에 멈추면 세계를 자라게 하는 잠든 생명.

## 개념

여러 식물 종의 씨앗을 하나로 묶은 Foundation ecology archetype이다. `Seed`는 이동 가능한 잠재 생명이고 [Plant](plant.md)는 조건을 만나 자리 잡은 성장 결과라는 구분을 만든다.

Seed는 현재 [Material Specification](../../../specs/MATERIAL_SPEC.md)의 **initial catalog direction**이다. 성장 조건과 runtime Rule은 아직 채택되거나 구현되지 않았다.

## 왜 넣는가

Seed가 없으면 Plant는 플레이어가 직접 놓는 STATIC 장식에 머물기 쉽다. Seed는 Powder movement, [Dirt](../p1/dirt.md), [Water](water.md), 시간과 성장 결과를 한 인과 사슬로 연결해 생태가 세계 안에서 시작되는 지점을 제공한다.

## 핵심 동사

```text
FALL / SETTLE
WAIT FOR SUITABLE CONDITIONS
GERMINATE
```

## 플레이어 직관

플레이어는 Seed가 작은 Powder처럼 떨어진 뒤 흙과 물이 있는 곳에서 싹틀 것이라 예상할 수 있다. 아무 표면에서나 즉시 복제되지 않고, 조건이 맞지 않으면 잠든 채 남는다는 제한도 읽혀야 한다.

## 세계 안의 역할

- dormant ecology input
- movable growth potential
- terrain and Water connector
- future biomass-chain source

## 대표 인과 사슬

```text
future candidate:
Seed + suitable Dirt + Water
→ growth eligibility
→ Plant

Plant + suitable environment / time
→ future Tree
→ Wood
→ combustion / decomposition chains
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Dirt](../p1/dirt.md) | future substrate | Seed가 자리 잡을 수 있는 익숙한 토양 후보. |
| [Water](water.md) | future condition | 성장을 허용하되 무한 복제의 무료 연료가 되어서는 안 된다. |
| [Plant](plant.md) | transition result | 잠든 이동 identity와 자리 잡은 성장 identity를 구분한다. |
| [Sand](sand.md) | movement contrast | 둘 다 POWDER일 수 있지만 Seed는 ecology condition과 결과를 가진다. |
| Heat / Combustion | counter | 부적합한 고온은 성장 조건을 막거나 향후 생물 Matter를 파괴한다. |

## 독립 Material인 이유

Seed와 Plant를 하나로 합치면 “이동해 자리를 찾는 잠재 생명”과 “공간을 차지하며 자라는 정착 생명”의 실험이 사라진다. 다만 실제 성장 전이가 없다면 Seed는 Sand의 재색칠이므로 독립 identity를 유지할 근거가 없다.

## Palette / Discovery 정책

- **Palette:** `deferred_until_adoption`
- ecology Rule과 eligibility가 채택되기 전에는 일반 player palette에 노출하지 않는다.
- adoption 뒤에는 실험을 시작하는 source 후보지만, Plant의 정확한 성장 조건은 관찰로 발견하게 한다.
- Dictionary 후보 문장: “씨앗은 떨어질 곳보다 자랄 조건을 찾는다.”

## 현실 앵커와 게임 추상화

### 현실 앵커

씨앗의 발아는 수분, 온도, 기질과 종별 조건에 영향을 받으며, 적합하지 않은 환경에서는 발아하지 않을 수 있다.

### 게임 추상화

종, 휴면 생리, 영양분, 뿌리와 발아율을 추적하지 않는다. local Dirt/Water/Temperature 조건을 읽는 하나의 generic Seed identity로 시작한다.

### 창작 보강

어떤 local 조건이 “suitable”인지, Seed가 Plant로 self-transition하는지 주변 빈 Cell에 growth claim을 만드는지는 Powdergame prototype에서 정할 규칙이다.

## 구현 개요

- Movement class: `POWDER` 방향
- descriptor-level properties: 기존 shared POWDER family를 우선 재사용
- Rule owner: future Seed/Plant material-owned growth rule 후보
- update tier: 조건이 없는 안정 Seed는 Sleep 가능한 slow ecology 후보
- state-cost policy: 모든 Cell에 growth·nutrition progress를 미리 추가하지 않음
- current Rule Card: 없음

현재 Registry에는 Seed identity가 없고 growth subsystem도 없다. 이 페이지는 `candidate / not_registered`인 초기 catalog 방향만 설명한다.

## 실패 모드 / 카운터

Seed가 Dirt·Water·공간 조건 없이 어디서나 Plant를 만들거나, 성장 frontier가 없는 안정 세계를 계속 깨우면 안 된다. 눈에 보이는 발아 조건과 Heat·기질 부족 같은 실패 조건이 필요하다.

## 미결정 사항

- [ ] 최소 성장 조건은 Dirt, Water, Temperature 중 어디까지 포함하는가?
- [ ] Seed Cell은 Plant로 바뀌는가, 빈 이웃에 성장을 요청하는가?
- [ ] 성장에 유한한 substrate 소비가 필요한가?
- [ ] adoption 뒤 Seed를 source palette에 직접 노출할 것인가?

## 관련 문서

- [Foundation family index](README.md)
- [Material Wiki](../README.md)
- [Plant](plant.md)
- [Water](water.md)
- [Dirt](../p1/dirt.md)
- [User Vision](../../../vision/USER_VISION.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Common-Sense Material Candidate Pool](../../derived/COMMON_SENSE_MATERIAL_CANDIDATE_POOL.md)
