---
title: Glass
type: material
id: glass
aliases:
  - Ordinary Glass
family: glass
status: candidate
implementation_state: not_registered
movement_class: STATIC
palette_policy: discovery_result_after_adoption
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../../vision/USER_VISION.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../derived/MATERIAL_PROTOTYPE_BUNDLES.md
  - ../../derived/BLOCK_PALETTE_AND_PG2_GAP_REVIEW.md
  - ../../derived/COMMON_SENSE_MATERIAL_CANDIDATE_POOL.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
tags:
  - foundation-candidate
  - glass
  - structure
  - manufacturing
---

# Glass

> 모래가 극한의 열을 지나, 빛과 충격에 새로운 선택을 남기는 구조.

## 개념

Glass는 ordinary silicate glass를 출발점으로 삼는 투명하고 취성 있는 STATIC structure 후보다. 현재는 초기 catalog direction이며, 구현된 Material identity나 승인된 Sand transition이 아니다.

## 왜 넣는가

Glass는 [Sand](sand.md)와 Heat를 manufacturing chain으로 연결해 플레이어가 공간 안에서 원료를 가공한 결과를 얻도록 할 수 있다. 다만 Stone과 색만 다른 벽이라면 독립 Material로 둘 이유가 없으므로, 구조·열·향후 gameplay Light 중 적어도 하나에서 읽히는 고유 동사가 필요하다.

## 핵심 동사

```text
FUTURE FORM FROM HEATED SAND
TRANSMIT / REVEAL
FRACTURE
```

## 플레이어 직관

Sand에 매우 강한 Heat를 가하면 Glass 같은 고체 결과가 생길 수 있고, 일반 Stone보다 깨지기 쉽지만 빛이나 내부를 통과시킬 수 있다고 예상할 수 있다. 이 기대는 아직 prototype과 구현으로 증명되지 않았다.

## 세계 안의 역할

- future manufactured result
- transparent/brittle structure candidate
- Sand와 Heat를 잇는 irreversible discovery reward
- 향후 gameplay Light와 enclosure를 잇는 소재
- glass family의 ordinary baseline 후보

## 대표 인과 사슬

```text
future candidate only:

Sand + extreme Heat
→ Glass
→ transparent or brittle structure
→ Pressure / thermal shock / future Light interaction
```

현재 코드에는 이 transition이나 Glass rule이 없다.

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Sand](sand.md) | future source | extreme Heat를 통한 manufacturing transition 후보다. |
| Temperature / Heat | future condition | 실제 threshold와 cooling policy는 미정이다. |
| Pressure | future counter | brittle fracture가 Stone과의 차이를 만들 수 있다. |
| [Stone](stone.md) | structural contrast | 불투명하고 일반적인 구조재와 다른 동사가 필요하다. |
| [Obsidian](../p1/obsidian.md) | glass-family contrast | Obsidian은 Lava 급랭 결과인 volcanic glass 후보로 생성 경로와 역할이 다르다. |
| [Lava](lava.md) | family context | Lava의 급랭 결과는 ordinary Glass가 아니라 Basalt/Obsidian P1 branch에서 다룬다. |
| Gameplay Light | future system | Light가 simulation truth가 된 뒤 transmit/absorb 성격을 검토한다. |

## 독립 Material인 이유

Ordinary Glass는 Sand-derived manufacturing result, transparency, brittle structure가 함께 읽힐 때 독립 identity가 된다. 단지 Stone보다 투명하거나 HP가 다른 정도라면 Presentation variant로 축소하거나 채택을 보류해야 한다.

## Palette / Discovery 정책

- **Palette:** `discovery_result_after_adoption`
- Glass는 현재 Registry에 없고 debug/demo renderer에도 Material identity로 연결되어 있지 않다.
- 일반 플레이어용 Material palette 자체가 아직 없으며, candidate 문서가 player exposure를 승인하지 않는다.
- 채택된다면 처음부터 source palette에 놓기보다 Sand 가공 결과를 관찰한 뒤 discovery result로 노출하는 방향이다.
- **Player Dictionary candidate:** “모래는 극심한 열을 지나면, 보이지만 쉽게 깨지는 벽이 될 수 있다.”
- 정확한 threshold, fracture 조건, future optical rule은 공개 이전에 먼저 구현·검증해야 한다.

## 현실 앵커와 게임 추상화

### 현실 앵커

보통의 유리는 주로 규산염 기반 원료를 녹이고 냉각해 만드는 비정질 고체이며, 투명성과 취성을 함께 가질 수 있다. 조성과 가공 방식에 따라 열적·광학적·기계적 성질은 크게 달라진다.

### 게임 추상화

정확한 조성, 점도 곡선, annealing, 응력 분포를 계산하지 않는다. 채택된다면 Sand와 Heat를 사용하는 local transition과 작은 구조/광학 descriptor로 충분한 행동만 표현한다.

### 창작 보강

`Sand + extreme Heat → Glass`를 단일하고 읽기 쉬운 world grammar로 사용하는 것은 여러 현실 공정을 압축한 게임 규칙이다. 현실의 Glass가 하지 않는 반응도 Powdergame 세계에서 일관되고 재미있다면 미래 후보가 될 수 있으나, 현재 페이지에서 약속하지 않는다.

## 구현 개요

- Canonical Wiki ID: `glass`
- Engine Material ID: 없음
- Movement class: `STATIC` candidate
- Registration state: 초기 catalog direction, `not_registered`
- Current Rule: 없음
- Future candidate: Sand의 high-temperature transition result
- State-cost policy: optical field, fracture progress, annealing state를 미리 추가하지 않음

`MATERIAL_SPEC.md`의 초기 catalog 목록과 Vision의 예시는 구현 증거가 아니다. 실제 Registry identity, debug color, transition, fixture, test가 생기기 전에는 `registered`, `implemented`, `validated`로 올리지 않는다.

## 실패 모드 / 카운터

Glass가 투명한 Stone에 그치거나, gameplay Light가 없는데 시각 효과만으로 독립 identity를 정당화하면 안 된다. Sand 한 Cell이 멀리 떨어진 Heat로 즉시 바뀌거나, 모든 Glass variant를 색과 강도 차이만으로 늘리는 것도 피한다.

## 미결정 사항

- [ ] Stone과 구별되는 첫 구현 동사는 brittle fracture, thermal shock, transparency 중 무엇인가?
- [ ] gameplay Light 이전에도 Glass가 충분한 interaction value를 가지는가?
- [ ] Sand → Glass transition은 어떤 local Heat/cooling 조건과 fixture로 읽히게 할 것인가?
- [ ] ordinary Glass와 P1 [Obsidian](../p1/obsidian.md)의 구조·열·생성 경로 차이를 어떻게 검증할 것인가?
- [ ] 채택 뒤 player discovery와 palette unlock을 어떤 현상 event에 연결할 것인가?

## 관련 문서

- [Foundation Material index](README.md)
- [Material Wiki](../README.md)
- [Sand](sand.md)
- [Stone](stone.md)
- [Lava](lava.md)
- [Obsidian](../p1/obsidian.md)
- [Authoritative User Vision](../../../vision/USER_VISION.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [Material Prototype Bundles](../../derived/MATERIAL_PROTOTYPE_BUNDLES.md)
- [Block Palette and Powder Game 2 Gap Review](../../derived/BLOCK_PALETTE_AND_PG2_GAP_REVIEW.md)
- [Common-Sense Material Candidate Pool](../../derived/COMMON_SENSE_MATERIAL_CANDIDATE_POOL.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
