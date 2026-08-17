---
title: Material Name
type: material
id: canonical_material_id
aliases: []
family: family-name
status: candidate
implementation_state: not_registered
movement_class: STATIC
palette_policy: hidden
updated: YYYY-MM-DD
last_verified: YYYY-MM-DD
sources: []
tags: []
---

# Material Name

> 한 문장으로 읽히는 정체성.

## 개념

이 Material이 어떤 종류의 존재인지 설명한다.

- 현실 물질
- 현실 물질 family의 게임 abstraction
- 역사적/연금술적 archetype
- Powdergame-original gap filler
- staged result / manufactured result

## 왜 넣는가

이 물질이 없을 때 세계에 어떤 상호작용 빈칸이 생기는지 적는다.

## 핵심 동사

```text
PRIMARY_VERB
SECONDARY_VERB
```

## 플레이어 직관

처음 보는 플레이어가 예상할 행동과, 실험 뒤에 발견할 숨은 행동을 구분한다.

## 세계 안의 역할

- terrain
- fuel
- structure
- reaction hub
- byproduct
- ecology
- manufacturing
- counter
- transport
- other

## 대표 인과 사슬

```text
Input / condition
→ transition
→ field or material output
→ follow-up interaction
```

## 상호작용 관계

| 상대 | 관계 | 결과/의미 |
|---|---|---|
| Material | input / output / counter | 설명 |

## 독립 Material인 이유

기존 Material의 variant나 단순 색 차이로 합치지 않는 이유를 적는다.

## Palette / Discovery 정책

- 처음부터 선택 가능한가
- 결과를 관찰한 뒤 해금되는가
- debug palette에서만 직접 생성 가능한가
- 플레이어 사전에 어떤 문장으로 기록되는가

## 현실 앵커와 게임 추상화

### 현실 앵커

자료가 실제로 지지하는 성질.

### 게임 추상화

정확한 현실 재현을 포기하거나 단순화한 부분.

### 창작 보강

자료가 부족해 Powdergame 고유 규칙으로 만든 부분. 없으면 `없음`.

## 구현 개요

- Movement class:
- descriptor-level properties:
- Rule owner:
- update tier:
- state-cost policy:
- current Rule Card:

수치와 threshold는 여기서 확정하지 않고 최신 Rule Card/SPEC에 연결한다.

## 실패 모드 / 카운터

강한 행동이 어디서 멈추거나 무효화되는지 적는다.

## 미결정 사항

- [ ] 질문
- [ ] prototype에서 확인할 항목

## 관련 문서

- [Material Wiki](README.md)
- 관련 family/prototype index
- 관련 Rule Card
- 관련 SPEC
