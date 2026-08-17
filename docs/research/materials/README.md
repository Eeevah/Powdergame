---
title: Powdergame Material Wiki
type: material-index
id: material-wiki-index
status: active
confidence: derived
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../specs/MATERIAL_SPEC.md
  - ../../specs/REACTION_SPEC.md
  - ../../specs/SIMULATION_SPEC.md
  - ../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
tags:
  - material
  - encyclopedia
  - interaction
  - discovery
---

# Powdergame Material Wiki

이 디렉터리는 Powdergame의 각 Material을 **하나의 세계 개념이자 플레이 가능한 상호작용 도구**로 설명한다.

수치표만 모으는 데이터베이스가 아니다. 각 페이지는 다음 질문에 답해야 한다.

> 이 물질은 무엇인가?  
> 왜 이 게임에 필요한가?  
> 다른 물질과 무엇이 다른가?  
> 무엇과 만나 어떤 세계 변화를 만드는가?  
> 어떤 조건에서 멈추거나 실패하는가?  
> 플레이어는 무엇을 관찰해 그 성질을 발견하는가?

## 권위

이 디렉터리는 기본적으로 `DERIVED / CANDIDATE` 문서다.

문서 충돌 시 권위 순서는 다음과 같다.

1. `docs/vision/USER_VISION.md`
2. 최신 ADR
3. `docs/specs/*`
4. `docs/planning/MILESTONES.md`
5. 이 Material Wiki
6. 넓은 research/encyclopedia 원자료

Material Wiki 페이지가 존재한다는 이유만으로 해당 Material이 등록·구현·검증된 것은 아니다.

## 상태 체계

개념 상태와 구현 상태를 분리한다.

### `status`

- `reference` — 아이디어/현실 자료 참고
- `candidate` — 게임 후보
- `prototype` — Rule Card와 fixture가 정의된 후보
- `adopted` — 승인된 SPEC/ADR/content contract에 반영
- `deprecated` — 다른 항목에 병합되거나 폐기

### `implementation_state`

- `not_registered`
- `registered`
- `implemented`
- `validated`

예를 들어 `status: prototype`이면서 `implementation_state: not_registered`일 수 있다.

## 페이지 계약

각 Material 페이지에는 가능한 한 다음이 있어야 한다.

- 한 문장 정체성
- 개념과 현실/역사적 앵커
- 넣은 이유
- 플레이어가 기억할 핵심 동사
- 세계 안의 역할
- Movement / Layer
- 대표 인과 사슬
- 관련 Material과의 링크
- palette 노출 정책
- 현실 고증과 게임 추상화의 경계
- Discovery 문장
- 구현 상태
- 미결정 사항
- 근거 문서

수치·threshold·density rank는 개념 페이지에 확정값처럼 복제하지 않는다. 최신 Rule Card 또는 SPEC을 참조한다.

## ID와 파일명

- canonical Material ID는 `snake_case`
- 파일명은 `kebab-case.md`
- 별칭은 frontmatter `aliases`에 기록
- 같은 개념의 작은 변형은 별도 페이지보다 variant/state로 우선 관리
- 실제로 다른 행동 동사가 생길 때만 독립 Material 페이지로 승격

## 링크 규칙

GitHub에서 읽히도록 일반 상대 링크를 사용한다.

```text
[Dirt](p1/dirt.md)
[Mud](p1/mud.md)
```

각 페이지는 최소한 다음을 연결한다.

- 직접 전이 전/후 Material
- 가장 중요한 상호작용 상대
- 소속 prototype/family index
- 현재 Rule Card
- 관련 SPEC

## 관리 원칙

1. **이름보다 동사:** 색이나 희귀도만 다르면 독립 Material로 만들지 않는다.
2. **개념과 튜닝 분리:** “왜 존재하는가”와 숫자값을 같은 문서에서 고정하지 않는다.
3. **결과물도 설명:** Mud, Wet Clay, Brick처럼 팔레트에 처음부터 보이지 않는 결과도 세계 개념이면 페이지를 가진다.
4. **현실과 게임을 구분:** 현실 자료, 게임 추상화, 창작 설정을 명시적으로 나눈다.
5. **발견 중심:** 플레이어용 도감은 이 개발 위키 전체를 공개하지 않고 관찰한 현상만 파생한다.
6. **출처 보존:** research에서 가져온 아이디어와 현재 프로젝트 결정의 출처를 남긴다.
7. **중복 병합:** 페이지를 늘리기 전에 alias/variant/result로 충분한지 확인한다.
8. **근거 없는 승격 금지:** 구현과 play evidence 없이 `adopted`나 `validated`로 올리지 않는다.

## 현재 컬렉션

Foundation은 현재 세계의 기본 어휘와 초기 catalog 방향을, P1은 그 어휘를 확장하는 첫 prototype 묶음을 관리한다.

### Foundation — 16

- [Foundation index](foundation/README.md)

#### M0 baseline

- [Boundary Block](foundation/boundary-block.md)
- [Stone](foundation/stone.md)
- [Sand](foundation/sand.md)
- [Ice](foundation/ice.md)
- [Water](foundation/water.md)
- [Steam](foundation/steam.md)
- [Smoke](foundation/smoke.md)
- [Wood](foundation/wood.md)
- [Oil](foundation/oil.md)

#### Existing catalog direction

- [Acid](foundation/acid.md)
- [Seed](foundation/seed.md)
- [Plant](foundation/plant.md)
- [Salt](foundation/salt.md)
- [Lava](foundation/lava.md)
- [Metal](foundation/metal.md)
- [Glass](foundation/glass.md)

Foundation의 9개 M0 identity는 별도 구현 브랜치의 실제 Registry를 확인해 `implemented`로 기록했지만, 개별 Product Gate 증거 없이 `validated`로 올리지 않는다. 나머지 7개는 문서화와 무관하게 `candidate / not_registered`다.

### P1 — Geology & Irreversible Manufacture

- [P1 index](p1/README.md)
- [Dirt](p1/dirt.md)
- [Mud](p1/mud.md)
- [Clay](p1/clay.md)
- [Wet Clay](p1/wet-clay.md)
- [Brick](p1/brick.md)
- [Basalt](p1/basalt.md)
- [Obsidian](p1/obsidian.md)
- [Limestone](p1/limestone.md)
- [Carbon Dioxide](p1/carbon-dioxide.md)

## 새 항목 작성

새 페이지는 [_TEMPLATE.md](_TEMPLATE.md)를 복사해 시작한다.

승격 흐름:

```text
research source
→ candidate page
→ interaction graph
→ Rule Card / fixture
→ implementation evidence
→ user review
→ adopted content contract
→ implemented
→ validated
```
