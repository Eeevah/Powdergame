---
title: Boundary Block
type: material
id: boundary_block
aliases:
  - World Boundary
family: world-boundary
status: adopted
implementation_state: implemented
movement_class: STATIC
palette_policy: world_boundary_only
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../../vision/USER_VISION.md
  - ../../../specs/SIMULATION_SPEC.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../architecture/decisions/ADR-0001-world-cell-invariants.md
  - ../../encyclopedia/01A_FOUNDATION_CATALOG.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs
  - https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/domain.rs
tags:
  - foundation
  - boundary
  - engine-primitive
---

# Boundary Block

> 움직이지 않는 세계의 가장자리. 무엇이 존재할 수 있는지 먼저 정한다.

## 개념

Boundary Block은 편집 가능한 유한 세계의 바깥선과 격리벽을 표현하는 `ENGINE_PRIMITIVE`다. 일반 암석을 흉내 낸 재료가 아니라, Matter가 세계 밖으로 새거나 서로 분리된 실험 영역이 섞이지 않게 하는 명시적 경계다.

## 왜 넣는가

유한 격자에는 플레이어가 예측할 수 있는 끝이 필요하다. Boundary Block이 없으면 바깥 영역, `EMPTY`, 일반 구조재의 의미가 뒤섞이고 world-boundary 처리가 개별 Rule에 흩어진다.

## 핵심 동사

```text
ENCLOSE
DIVIDE
BLOCK
```

## 플레이어 직관

보이는 경계는 움직이거나 타거나 녹는 재료가 아니라 세계의 틀로 읽혀야 한다. 일반 Stone과 닮아 보여도 편집 가능한 구조재로 오해하게 해서는 안 된다.

## 세계 안의 역할

- finite-world boundary
- simulation-domain guard
- fixture isolation
- non-reactive control surface

## 대표 인과 사슬

```text
World configuration
→ outer cells become Boundary Block
→ ordinary Matter cannot occupy or cross those cells
→ finite world remains well-defined
```

```text
Separated test regions
→ Boundary Block divider
→ movement and thermal influence remain local
→ each causal chain can be observed independently
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| [Stone](stone.md) | contrast | Stone은 세계 안의 구조재이고 Boundary Block은 세계의 구조 자체다. |
| `EMPTY` | semantic boundary | Boundary Block은 부재 상태나 숨은 Air가 아니다. |
| movable Matter | blocker | 일반 movement나 density swap의 목적지가 되지 않는다. |
| Temperature | isolation | 일반 열전달 매질처럼 사용하지 않는다. |
| Pressure | containment boundary | 일반 파괴 대상이 아닌 domain control로 남는다. |

## 독립 Material인 이유

Stone에 같은 책임을 주면 플레이어가 만든 벽과 엔진이 보장하는 세계 경계를 구분할 수 없다. Boundary Block은 반응 가능한 Matter family가 아니라 topology와 invariant를 지키는 별도 identity다.

## Palette / Discovery 정책

- **Palette:** `world_boundary_only`
- 일반 플레이어 palette button으로 노출할 대상이 아니다.
- 구현 브랜치에는 scenario 구성, presentation, debug 경로에서의 생성·식별 수단이 있지만 일반 플레이어 palette UI가 있다는 증거는 없다.
- 플레이어에게는 “이 선 바깥은 편집 가능한 세계가 아니다”라는 세계 규칙으로 전달한다.

## 현실 앵커와 게임 추상화

### 현실 앵커

용기 벽과 실험 장치의 격벽처럼 영역을 둘러싸고 서로 다른 환경을 분리하는 구조가 직관의 출발점이다.

### 게임 추상화

현실의 특정 재료나 두께를 재현하지 않고 한 Cell의 절대적 domain boundary로 표현한다.

### 창작 보강

세계의 topology를 등록된 identity로 보이게 만든 Powdergame 고유의 엔진 primitive다.

## 구현 개요

- Movement class: STATIC
- Descriptor-level properties: 일반 movement·density swap·열전달에서 경계로 취급
- Rule owner: world/domain initialization과 boundary handling
- Update tier: 일반 Material reaction 대상이 아님
- State-cost policy: 별도 per-cell boundary 상태를 추가하지 않고 identity로 표현
- Current Rule Card: 없음. world invariant와 Simulation/Material SPEC이 계약이다.

### 현재 구현 상태와 증거 경계

구현 commit [`177879c`](https://github.com/Eeevah/Powdergame/commit/177879c2e2e916066f376a4465c00430a0cdd8ac)에서 Registry, domain 초기화, GPU descriptor와 scenario/presentation/debug 경로를 확인했다. 따라서 `implemented`로 기록한다. 다만 독립적인 player Product Gate를 통과했다는 직접 증거로 확장하지 않으므로 `validated`로 올리지 않는다. 일반 플레이어 palette 노출도 확인되지 않았다.

## 실패 모드 / 카운터

Boundary Block이 움직이거나 일반 반응으로 변하거나 density swap에 참여하면 world invariant가 무너진다. Stone과 구별되지 않아 플레이어가 일반 건축재로 이해하거나, `EMPTY`/Air처럼 취급하는 것도 실패다.

## 미결정 사항

- [ ] 미래 world editor에서 Boundary Block을 어느 수준까지 보이거나 편집 가능하게 할 것인가?
- [ ] Stone과 혼동하지 않으면서도 시각적으로 과도하게 튀지 않는 표현은 무엇인가?
- [ ] Void로 열린 경계와 닫힌 Boundary Block을 플레이어에게 어떻게 구분해 설명할 것인가?

## 관련 문서

- [Foundation Materials](README.md)
- [Material Wiki](../README.md)
- [P1 family index](../p1/README.md)
- [User Vision](../../../vision/USER_VISION.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [World Cell Invariants ADR](../../../architecture/decisions/ADR-0001-world-cell-invariants.md)
- [Foundation Catalog](../../encyclopedia/01A_FOUNDATION_CATALOG.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Implementation evidence: Material Registry](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs)
- [Implementation evidence: domain boundary](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/domain.rs)
