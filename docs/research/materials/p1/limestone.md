---
title: Limestone
type: material
id: limestone
aliases:
  - Calcite Rock
  - Carbonate Rock
family: reactive-rock
status: prototype
implementation_state: not_registered
movement_class: STATIC
palette_policy: visible_after_adoption
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../specs/REACTION_SPEC.md
tags:
  - reactive-rock
  - p1
  - interaction
---

# Limestone

> 물은 천천히 만들 수 있고, 산은 거품과 기체로 빠르게 지워버리는 밝은 돌.

## 개념

석회암·방해석 계열을 하나의 게임용 carbonate rock archetype으로 묶은 reactive STATIC Matter다.

## 왜 넣는가

Stone family에 `DISSOLVE / RELEASE GAS / future DEPOSIT / construction precursor`라는 동사를 추가한다. Acid가 단순 삭제액이 아니라 눈에 보이는 Gas 반응을 만들게 한다.

## 핵심 동사

```text
REACT WITH ACID
RELEASE GAS
FUTURE PRECIPITATE / PROCESS
```

## 플레이어 직관

밝은 돌에 Acid를 떨어뜨리면 거품과 무거운 Gas가 생기고, 돌과 Acid가 둘 다 소모된다고 이해할 수 있어야 한다.

## 세계 안의 역할

- reactive geology
- Acid target
- CO2 source
- future cave/construction chain

## 대표 인과 사슬

```text
Limestone + Acid → CO2 + Water abstraction

future: mineral-bearing Water → Calcite deposit

future: Limestone → Lime/Cement family
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| Acid | reactant/counter | Limestone 자신은 CO2로, Acid는 Water로 전이한다. |
| CO2 | output | 반응을 눈에 보이게 만드는 Gas 결과. |
| Water | future formation medium | 침전/동굴 성장 family의 후속 후보. |
| Cement / Concrete | future manufacturing | 두 번째 건축 제조 chain의 원료 후보. |

## 독립 Material인 이유

Stone은 안정 baseline이고 Limestone은 Acid에 반응하고 Gas를 만든다. 이 반응이 기억할 장면을 만들면 독립 identity가 확실하다.

## Palette / Discovery 정책

- **Palette:** `visible_after_adoption`
- **Player Dictionary:** “Some pale rocks release a heavy gas when Acid reaches them.”
- 정확한 threshold, rule priority, 남은 discovery 개수는 공개하지 않는다.

## 현실 앵커와 게임 추상화

### 현실 앵커

탄산염 암석은 산과 반응해 이산화탄소가 발생할 수 있다.

### 게임 추상화

용해 이온과 염을 별도 Matter로 추적하지 않고 Limestone Cell + Acid Cell을 CO2 Cell + Water Cell로 근사한다.

### 창작 보강

정확한 반응량·속도·중화 생성물은 P1 gameplay abstraction이다.

## 구현 개요

- Movement class: STATIC
- Rule owner: Limestone and paired Acid self-rules
- Reaction phase: Special Reaction
- State-cost policy: dissolution progress/acid capacity 없음
- Rule Cards: P1-LIMESTONE-001, P1-ACID-001

수치·threshold·density rank는 이 페이지에서 확정하지 않는다. 최신 Rule Card가 튜닝 source다.

## 실패 모드 / 카운터

하나의 Acid Cell이 한 Tick에 거대한 벽을 지우면 안 된다. self-write 근사가 심각하면 이 pair만 Claim/Resolve로 승격한다.

## 미결정 사항

- [ ] self-write fan-out가 허용 가능한가?
- [ ] CO2 생성량이 반응을 충분히 읽히게 하는가?

## 관련 문서

- [P1 family index](README.md)
- [Material Wiki](../README.md)
- [Foundation: Stone](../foundation/stone.md)
- [Foundation: Acid](../foundation/acid.md)
- [Foundation: Water](../foundation/water.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
