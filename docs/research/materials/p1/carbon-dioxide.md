---
title: Carbon Dioxide
type: material
id: carbon_dioxide
aliases:
  - CO2
family: gas
status: prototype
implementation_state: not_registered
movement_class: GAS
palette_policy: debug_only_in_p1_player_exposure_later
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../specs/REACTION_SPEC.md
tags:
  - gas
  - p1
  - interaction
---

# Carbon Dioxide

> 불과 돌의 반응이 남기는 무거운 숨. 위로 달아나는 증기와 달리 낮은 곳을 찾는다.

## 개념

현실의 이산화탄소를 gameplay에 필요한 범위로 단순화한 heavy, nonflammable GAS Matter다. P1에서는 Limestone+Acid 반응의 가시적 출력이 핵심이다.

## 왜 넣는가

화학 반응이 고체 삭제로 끝나지 않고 이동하는 결과를 만들게 한다. Steam/Smoke와 다른 Gas density를 보여주며, 다음 combustion prototype에서 불 억제 역할로 확장될 수 있다.

## 핵심 동사

```text
FORM AS GAS OUTPUT
SETTLE AS HEAVY GAS
FUTURE SUPPRESS COMBUSTION
SOLIDIFY AS DRY ICE LATER
```

## 플레이어 직관

Gas지만 Steam처럼 위로만 빠지지 않고 낮은 곳에 고이며 불을 돕지 않는다고 예상할 수 있다.

## 세계 안의 역할

- reaction output
- heavy Gas exemplar
- future combustion counter
- Dry Ice transition target

## 대표 인과 사슬

```text
Limestone + Acid → CO2

CO2 moves through shared GAS family with heavier density ordering

future: Dry Ice + Heat → CO2

future: CO2 pocket + Fire → combustion weakened
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| Limestone | source | Acid 반응에서 Limestone Cell이 CO2로 전이한다. |
| Acid | reaction partner | Acid는 Water로 중화되는 근사. |
| Steam / Smoke | movement contrast | 같은 GAS family지만 density ordering이 다르다. |
| Fire / Combustion | future counter | P2에서 local suppression modifier 후보. |
| Dry Ice | future phase source | 승화 결과로 연결될 후보. |

## 독립 Material인 이유

P1만 보면 지원 output이지만, heavy Gas와 이후 Fire suppression/Dry Ice chain까지 연결된다. Steam과 다르게 움직이지 못하면 P1 support identity로만 남기거나 P2까지 판단을 보류한다.

## Palette / Discovery 정책

- **Palette:** `debug_only_in_p1_player_exposure_later`
- **Player Dictionary:** “Some mineral reactions release a gas that settles lower than Steam.”
- 정확한 threshold, rule priority, 남은 discovery 개수는 공개하지 않는다.

## 현실 앵커와 게임 추상화

### 현실 앵커

CO2는 상온에서 기체이며 공기보다 무겁고 연소를 지지하지 않는다.

### 게임 추상화

완전한 대기 조성이나 분압을 계산하지 않고 density rank와 local modifier로 표현한다.

### 창작 보강

P1에서는 fire suppression을 의도적으로 범위 밖에 둔다.

## 구현 개요

- Movement class: shared GAS
- Descriptor: heavy_gas, nonflammable
- Rule owner: none in P1; produced by Limestone rule
- State-cost policy: atmosphere composition 없음
- Rule Card source: P1-LIMESTONE-001

수치·threshold·density rank는 이 페이지에서 확정하지 않는다. 최신 Rule Card가 튜닝 source다.

## 실패 모드 / 카운터

Steam과 완전히 같은 움직임이면 gas identity가 읽히지 않는다. P1에서 할 일이 적다고 소화 Rule을 억지로 범위에 넣지 않는다.

## 미결정 사항

- [ ] GAS density ordering만으로 낮은 곳 축적이 충분히 보이는가?
- [ ] P2에서 fire suppression을 어떤 local rule로 표현할 것인가?

## 관련 문서

- [P1 family index](README.md)
- [Material Wiki](../README.md)
- [Foundation: Acid](../foundation/acid.md)
- [Foundation: Water](../foundation/water.md)
- [Foundation: Steam](../foundation/steam.md)
- [Foundation: Smoke](../foundation/smoke.md)
- [Foundation: Wood](../foundation/wood.md)
- [Foundation: Oil](../foundation/oil.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
