---
title: Plant
type: material
id: plant
aliases:
  - Generic Plant
family: ecology
status: candidate
implementation_state: not_registered
movement_class: STATIC
palette_policy: discovery_result_after_adoption
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../../vision/USER_VISION.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../specs/SIMULATION_SPEC.md
  - ../../../specs/REACTION_SPEC.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - ../../derived/BLOCK_PALETTE_AND_PG2_GAP_REVIEW.md
  - ../../derived/COMMON_SENSE_MATERIAL_CANDIDATE_POOL.md
  - ../../encyclopedia/01A_FOUNDATION_CATALOG.md
tags:
  - ecology
  - growth
  - foundation-placeholder
  - initial-catalog-direction
---

# Plant

> 씨앗이 환경을 읽고 공간에 남긴, 가장 단순한 살아 있는 구조.

## 개념

여러 식물과 성장 형태를 하나로 압축한 Foundation biology abstraction이다. 정확한 종이 아니라 “조건을 만나 정착하고, 공간을 차지하며, 다른 biomass chain으로 이어지는 생명 Matter”를 대표한다.

Plant는 [Material Specification](../../../specs/MATERIAL_SPEC.md)의 **initial catalog direction**이며 아직 Registry나 growth Rule에 존재하지 않는다. 장기적으로 [Tree / Vine / Moss / Fungus](../../derived/BLOCK_PALETTE_AND_PG2_GAP_REVIEW.md#6-plant-family--one-generic-plant-is-too-narrow)가 서로 다른 동사를 증명하면 구체 identity로 분해될 수 있는 placeholder다.

## 왜 넣는가

Plant는 Water와 terrain이 이동·열만이 아니라 생태 변화를 만들게 한다. [Seed](seed.md), [Dirt](../p1/dirt.md), Water, [Wood](wood.md), Combustion과 future decomposition을 연결해 세계가 스스로 흔적을 축적하는 장기 사슬의 입구가 된다.

## 핵심 동사

```text
GERMINATE / TAKE ROOT
OCCUPY AND GROW
MATURE / BECOME BIOMASS
```

## 플레이어 직관

플레이어는 적합한 흙과 물이 Seed를 Plant로 만들고, 고온이나 불이 성장을 멈춘다고 예상할 수 있다. Plant가 아무 Matter나 먹어 치우는 전염체가 아니라 제한된 생태 조건을 가진다는 점도 관찰 가능해야 한다.

## 세계 안의 역할

- ecology result
- visible growth state
- terrain and Water consumer/connector
- future Wood and decomposition source
- broad Foundation placeholder

## 대표 인과 사슬

```text
future candidate:
Seed + suitable Dirt / Water
→ Plant

Plant + suitable topology / time
→ Tree
→ Wood
→ Heat + Smoke through combustion

future family branches:
Plant abstraction
→ Tree / Vine / Moss / Fungus
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Seed](seed.md) | source | 이동 가능한 잠재 생명이 정착한 결과 후보. |
| [Dirt](../p1/dirt.md) | substrate | ecology prototype에서 성장 자격을 제공할 기본 토양 후보. |
| [Water](water.md) | growth condition | 성장을 잇지만 무한 증식의 무료 조건으로 단순화하지 않는다. |
| [Wood](wood.md) | future mature output | Tree/biomass chain을 Foundation combustion과 연결한다. |
| Heat / Combustion | counter | 고온은 생물 성장을 멈추거나 파괴하는 공통 카운터 후보다. |

## 독립 Material인 이유

Seed는 떨어지고 자리를 찾는 잠재 상태이고 Plant는 정착해 형태를 유지하는 결과다. Wood는 이미 구조재·연료가 된 biomass다. Plant가 성장·성숙·환경 반응 없이 STATIC 초록 블록에 머문다면 독립 Material이 아니라 presentation variant로 축소해야 한다.

## Palette / Discovery 정책

- **Palette:** `discovery_result_after_adoption`
- adoption 뒤에도 처음부터 source button으로 제공하기보다 Seed의 성공적인 발아 결과로 먼저 발견하게 하는 방향이다.
- Tree, Vine, Moss, Fungus의 분화 조건은 해당 behavior가 채택되기 전까지 Dictionary 답안으로 공개하지 않는다.
- Dictionary 후보 문장: “생명은 놓인 곳보다 물과 바닥이 허락한 곳에서 자란다.”

## 현실 앵커와 게임 추상화

### 현실 앵커

식물은 물, 적합한 기질과 환경 조건을 필요로 하며 성장 형태와 생태 역할이 서로 다르다. 고온과 연소는 생물 조직을 손상시킬 수 있다.

### 게임 추상화

광합성, 뿌리, 종별 생장, 영양 순환과 세포 생리를 구현하지 않는다. 첫 단계에서는 local 조건과 staged Material identity로 성장의 원인과 결과만 읽히게 한다.

### 창작 보강

Plant에서 Tree로 성숙하는 조건, growth topology와 Wood 생성 방식은 Powdergame 고유 콘텐츠 규칙이 된다. 아직 어느 것도 채택된 runtime 계약이 아니다.

## 구현 개요

- Movement class: `STATIC` 방향
- descriptor-level properties: growth result에 필요한 최소 thermal/combustion 성격은 prototype 뒤 결정
- Rule owner: future Seed/Plant material-owned growth rule 후보
- update tier: 느린 local ecology frontier 후보
- state-cost policy: universal growth/nutrition/corrosion byte를 모든 Cell에 추가하지 않음
- current Rule Card: 없음

현재 Registry에는 Plant identity가 없고 ecology topology도 구현되지 않았다. `candidate / not_registered` 상태이며 initial catalog direction을 넘는 구현 약속이 아니다.

## 실패 모드 / 카운터

Plant가 조건 없이 빈 공간 전체로 퍼지거나 Stone 같은 비생물 Matter를 임의 변환하면 ecology가 아니라 범용 Virus가 된다. eligible substrate, Water 조건, 공간 제한과 Heat/Combustion 카운터가 필요하다.

Tree, Vine, Moss, Fungus가 색과 모양만 다르면 별도 identity로 분해하지 않는다. 각각 성숙, 표면 추종, 습윤 광물 식민, 유기물 분해라는 관찰 가능한 동사를 증명해야 한다.

## 미결정 사항

- [ ] Foundation Plant가 담당할 최소 성장 topology는 무엇인가?
- [ ] Plant는 언제 Tree identity로 분리되는가?
- [ ] Vine, Moss, Fungus 중 첫 ecology prototype은 무엇인가?
- [ ] 성장에 Dirt/Water를 어떻게 소비하거나 유지 조건으로 사용할 것인가?
- [ ] 발견 뒤 player palette에 직접 노출할 필요가 있는가?

## 관련 문서

- [Foundation family index](README.md)
- [Material Wiki](../README.md)
- [Seed](seed.md)
- [Water](water.md)
- [Wood](wood.md)
- [Dirt](../p1/dirt.md)
- [User Vision](../../../vision/USER_VISION.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Block Palette & Powder Game 2 Gap Review](../../derived/BLOCK_PALETTE_AND_PG2_GAP_REVIEW.md)
- [Common-Sense Material Candidate Pool](../../derived/COMMON_SENSE_MATERIAL_CANDIDATE_POOL.md)
