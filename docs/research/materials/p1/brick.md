---
title: Brick
type: material
id: brick
aliases:
  - Fired Brick
family: manufactured-structure
status: prototype
implementation_state: not_registered
movement_class: STATIC
palette_policy: unlock_after_discovery
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../specs/REACTION_SPEC.md
tags:
  - manufactured-structure
  - p1
  - interaction
---

# Brick

> 불이 흙에게 남긴 기억. 한 번 구워진 형태는 다시 진흙으로 돌아가지 않는다.

## 개념

Clay 또는 Wet Clay를 충분히 가열해 얻는 manufactured STATIC Matter다. 단순 건축 블록이 아니라 **세계 안에서 자연 원료를 가공해 만든 첫 비가역 재료**다.

## 왜 넣는가

Powdergame의 조합 재미를 메뉴식 레시피가 아니라 실제 Temperature와 상태 전이로 보여준다. 플레이어가 환경을 이용해 물질을 생산하는 첫 명확한 보상이다.

## 핵심 동사

```text
FIRE / MANUFACTURE
HOLD STRUCTURE
RESIST REWETTING
```

## 플레이어 직관

Clay를 강하게 구우면 단단한 벽돌이 되고, 다시 물을 부어도 Clay로 돌아가지 않는다고 이해할 수 있다.

## 세계 안의 역할

- manufactured result
- early structural material
- discovery reward
- construction chain seed

## 대표 인과 사슬

```text
Clay + Heat → Brick

Wet Clay + Heat → Brick

Brick + Water → remains Brick

future: Brick + structure pattern → shelter discovery
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| Clay | input | 직접 소성 가능. |
| Wet Clay | input | 형태를 만든 뒤 소성하는 대표 경로. |
| Water | non-reverse test | 재습윤으로 원료에 돌아가지 않는다. |
| Pressure | structural test | Stone과 다른 rupture/strength class 후보. |

## 독립 Material인 이유

Stone은 자연의 기본 구조재이고 Brick은 플레이어가 Heat로 생산한 구조재다. Origin과 제조 의미에 더해 파괴·열 성격도 달라야 독립성을 유지한다.

## Palette / Discovery 정책

- **Palette:** `unlock_after_discovery`
- **Player Dictionary:** “Enough Heat permanently changes Clay into a structural material.”
- 정확한 threshold, rule priority, 남은 discovery 개수는 공개하지 않는다.

## 현실 앵커와 게임 추상화

### 현실 앵커

점토를 소성하면 물에 다시 풀리지 않는 세라믹성 구조재가 된다.

### 게임 추상화

수축률·기공·소결 품질을 생략하고 임계 온도 전이로 처리한다.

### 창작 보강

정확한 pressure resistance와 thermal class는 gameplay tuning이다.

## 구현 개요

- Movement class: STATIC
- Rule owner: none in P1 after creation
- Produced by Clay/Wet Clay rules
- State-cost policy: no firing-quality state
- Rule Cards: P1-CLAY-002, P1-WET-CLAY-002

수치·threshold·density rank는 이 페이지에서 확정하지 않는다. 최신 Rule Card가 튜닝 source다.

## 실패 모드 / 카운터

Stone과 물성·시각·발견 의미가 모두 같으면 별도 Material일 이유가 없다. 적어도 제조 경로와 구조적 피드백이 분명해야 한다.

## 미결정 사항

- [ ] Stone 대비 rupture/thermal 차이를 얼마나 줄 것인가?
- [ ] 발견 후 palette에 직접 노출할 것인가?

## 관련 문서

- [P1 family index](README.md)
- [Material Wiki](../README.md)
- [Foundation: Water](../foundation/water.md)
- [Foundation: Stone](../foundation/stone.md)
- [P1 Rule Cards](../../derived/P1_GEOLOGY_AND_MANUFACTURE_RULE_CARDS.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
