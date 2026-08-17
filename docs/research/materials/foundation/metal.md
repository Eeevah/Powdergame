---
title: Metal
type: material
id: metal
aliases:
  - Generic Metal
family: engineering-metal
status: candidate
implementation_state: not_registered
movement_class: STATIC
palette_policy: deferred_until_adoption
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
  - metal
  - engineering
  - structure
  - foundation-placeholder
  - initial-catalog-direction
---

# Metal

> 돌보다 열을 잘 이어 주고, 녹고 부식하면서 공학의 차이를 드러내는 첫 구조재.

## 개념

여러 현실 금속을 하나로 뭉친 Foundation engineering abstraction이다. 초기에는 구조·열전달·용융·부식으로 이어질 공통 입구를 제공하고, 나중에는 실제로 다른 동사를 증명한 구체 Metal identity로 분해될 수 있다.

Metal은 [Material Specification](../../../specs/MATERIAL_SPEC.md)의 **initial catalog direction**이다. 현재 Registry에는 generic Metal도 구체 Metal도 없으며, 이 페이지는 구현 약속이나 주기율표 확장 계획이 아니다.

## 왜 넣는가

[Stone](stone.md)만으로는 Heat를 전달하는 공학 구조, 녹는 구조재, 환경을 기록하는 부식 같은 세계 문법이 빈다. Metal은 Temperature, Pressure, [Acid](acid.md)와 future manufacturing을 연결하되, 단순히 Stone보다 모든 수치가 높은 “상위 재료”가 아니어야 한다.

## 핵심 동사

```text
CONDUCT HEAT
HOLD STRUCTURE
MELT / CORRODE UNDER ELIGIBLE CONDITIONS
```

## 플레이어 직관

플레이어는 Metal이 단단하고 열을 빠르게 전달하며 충분히 가열하면 녹을 것이라 예상할 수 있다. 이후에는 모든 금속이 같지 않고, 녹슬거나 산화하거나 매우 무겁거나 액체로 흐르는 차이를 실험으로 발견해야 한다.

## 세계 안의 역할

- engineering structure
- thermal transport baseline
- future phase/manufacture input
- future corrosion family root
- concrete-metal placeholder

## 대표 인과 사슬

```text
future generic direction:
Metal + sufficient Heat
→ molten Metal stage
→ flow / cast / cool

eligible Metal + Acid or corrosion environment
→ material-owned changed state

placeholder decomposition:
Metal
├─ Iron   → rust / structure / melt
├─ Copper → conduct Heat / oxidize in stages
├─ Lead   → sink / lower-melt ballast
└─ Mercury→ dense liquid metal
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Stone](stone.md) | structural contrast | Metal은 별도 thermal/phase/chemistry 동사를 증명해야 한다. |
| Heat / Temperature | phase driver | molten stage와 공학적 열전달을 여는 공통 조건 후보. |
| Pressure | structure test | 버티기만 하는 절대 재료가 아니라 Material별 resistance 차이를 표현할 축. |
| [Acid](acid.md) | future corrosion condition | 모든 Metal에 동일 결과를 강제하지 않고 대상이 자기 Rule을 소유한다. |
| [Lava](lava.md) | high-Heat environment | future melting/casting 실험의 자연 Heat source 후보. |

## 독립 Material인 이유

Stone은 안정된 자연 구조 baseline이고 Metal은 열전달·용융·부식·가공으로 공학 사슬을 만든다. 이런 차이가 없다면 Metal은 회색 Stone variant에 불과하다.

Generic Metal은 Foundation placeholder로 유용하지만 영구적으로 모든 금속을 대신하지 않는다. [Iron, Copper, Lead, Mercury](../../derived/BLOCK_PALETTE_AND_PG2_GAP_REVIEW.md#3-metal-family--generic-metal-should-become-a-foundation-placeholder)는 각각 rust, staged oxidation/thermal conduction, density/lower-melting contrast, liquid-metal movement라는 관찰 가능한 동사를 증명할 때만 별도 identity가 된다.

## Palette / Discovery 정책

- **Palette:** `deferred_until_adoption`
- generic Metal의 descriptor와 concrete-family 경계가 채택되기 전에는 player palette에 노출하지 않는다.
- adoption 뒤 generic source가 필요한지, Iron 같은 concrete source로 바로 시작할지는 사용자 판단으로 남긴다.
- 새로운 Metal은 색·희귀도·더 높은 수치만으로 palette slot을 얻지 않는다.
- Dictionary 후보 문장: “금속은 열을 이어 주지만, 어떤 금속인지는 녹고 흐르고 부식될 때 드러난다.”

## 현실 앵커와 게임 추상화

### 현실 앵커

금속은 일반적으로 열전도·구조·용융·산화에서 중요한 재료지만, 실제 성질은 금속과 합금마다 크게 다르다. 수은처럼 상온에서 액체인 금속도 있어 하나의 현실 물성표로 묶을 수 없다.

### 게임 추상화

합금 조성, 결정 구조, 소성가공과 전기 전도 전체를 구현하지 않는다. compact descriptor와 Material-owned transition으로 플레이에 필요한 차이만 표현한다. Electricity는 현재 Foundation 구현 범위가 아니다.

### 창작 보강

generic Metal을 얼마나 오래 유지할지, molten stage를 variant와 독립 identity 중 무엇으로 둘지, 구체 Metal의 corrosion/phase 결과는 Powdergame 콘텐츠 결정이다.

## 구현 개요

- Movement class: `STATIC` 방향
- descriptor-level properties: thermal conduction, heat capacity, rupture/phase/corrosion 자격 후보
- Rule owner: future concrete Metal이 자기 melting/corrosion Rule을 소유
- update tier: thermal 또는 reaction frontier에서만 활성인 local rule 후보
- state-cost policy: 모든 Cell에 oxidation/corrosion progress를 미리 추가하지 않음
- current Rule Card: 없음

현재 Registry에는 Metal identity, molten Metal, Iron, Copper, Lead, Mercury가 없다. 이 페이지의 상태는 `candidate / not_registered`이며 decomposition 목록은 자동 등록 계획이 아니다.

## 실패 모드 / 카운터

Metal이 Stone보다 단단하고 빠르고 안전한 만능 상위 재료가 되면 선택이 사라진다. 반대로 열전달·용융·부식 차이가 없으면 독립 identity가 없다.

Iron/Copper/Lead/Mercury를 색만 바꾼 STATIC 블록으로 늘리지 않는다. 각 identity는 별도 interaction verb, observable chain과 실패 조건을 증명해야 한다.

## 미결정 사항

- [ ] generic Metal을 adopted catalog에 유지할 것인가, concrete family의 임시 alias로 둘 것인가?
- [ ] 첫 concrete Metal prototype은 Iron과 Copper 중 무엇인가?
- [ ] Lead의 density와 melting contrast가 실제로 읽히는가?
- [ ] Mercury는 generic molten stage와 어떻게 구분할 것인가?
- [ ] molten Metal은 phase result인가 독립 discoverable identity인가?

## 관련 문서

- [Foundation family index](README.md)
- [Material Wiki](../README.md)
- [Stone](stone.md)
- [Acid](acid.md)
- [Lava](lava.md)
- [User Vision](../../../vision/USER_VISION.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Block Palette & Powder Game 2 Gap Review](../../derived/BLOCK_PALETTE_AND_PG2_GAP_REVIEW.md)
- [Common-Sense Material Candidate Pool](../../derived/COMMON_SENSE_MATERIAL_CANDIDATE_POOL.md)
