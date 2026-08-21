# Powdergame Documentation

> **먼저 `START_HERE.md`를 읽는다.**
>
> 이 문서 집합은 현재 계약뿐 아니라 사용자의 의도, 선택한 대안, 검증 결과와 superseded 결정을 보존한다. 그러나 모든 문서를 처음부터 읽는 것이 목표는 아니다. 작업 종류에 맞는 최소 경로로 들어가고, 충돌이 있을 때 권위 문서로 확장한다.

---

## 1. 가장 짧은 입구

### 제품 의도를 이해하려면

1. [`START_HERE.md`](START_HERE.md)
2. [`vision/USER_VISION.md`](vision/USER_VISION.md)
3. [`vision/FIRST_PLAYABLE_WORLD.md`](vision/FIRST_PLAYABLE_WORLD.md)
4. [`vision/UI_DIRECTION.md`](vision/UI_DIRECTION.md)

### 현재 작업을 이어받으려면

1. [`development/QUICKSTART.md`](development/QUICKSTART.md)
2. [`planning/STATUS.md`](planning/STATUS.md)
3. 해당 Gate의 [`evidence/`](evidence/)
4. [`HANDOFF.md`](HANDOFF.md)

### Engine을 수정하려면

1. 관련 [`architecture/decisions/ADR-*`](architecture/decisions/)
2. 관련 [`specs/*`](specs/)
3. [`architecture/ARCHITECTURE.md`](architecture/ARCHITECTURE.md)
4. 관련 tests/evidence

### 검증·성능·Evidence를 수정하려면

1. [`development/VALIDATION_POLICY.md`](development/VALIDATION_POLICY.md)
2. [`development/TESTING.md`](development/TESTING.md)
3. [`development/PERFORMANCE.md`](development/PERFORMANCE.md)
4. [`development/WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md`](development/WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md)
5. 해당 [`evidence/*`](evidence/)

`START_HERE.md`에는 작업 종류별 최소 읽기 경로가 더 자세히 정리되어 있다.

---

## 2. 문서 권위 순서

문서가 충돌하면 다음 순서를 따른다.

1. **제품 원칙** — `vision/USER_VISION.md`
2. **승인된 구조 결정** — 최신 `architecture/decisions/ADR-*`
3. **현재 구현 계약** — `specs/*`
4. **실행으로 확인된 진실** — code, tests, `evidence/*`, `architecture/ARCHITECTURE.md`
5. **완료 기준** — `planning/MILESTONES.md`
6. **현재 상태와 다음 작업** — `planning/STATUS.md`
7. **장기 방향과 순서** — `planning/ROADMAP.md`
8. **실행 인수인계** — `HANDOFF.md`
9. **개념 연구** — `research/materials/*`, `research/derived/*`, `research/encyclopedia/*`
10. **원문 연구** — `research/raw/*`
11. **결정 맥락** — `design-history/*`, `01_MASTER_DESIGN_REPORT.md`, `00_USER_VISION.md`
12. 초기 prototype/experiment

`development/*`는 개발·테스트·artifact 운영 절차를 고정하지만 제품 Vision, SPEC, 실제 실행 결과를 덮어쓰지 않는다.

### Q&A와 사용자 교정의 지위

사용자가 직접 선택하거나 가정을 수정한 기록은 초기 연구 가설보다 강하다. 제품 의도를 해석할 때는 `design-history/*`를 확인한다. 구현 코드는 최신 ADR/SPEC을 따르며, 결정이 바뀌면 과거 문서를 조용히 다시 쓰지 않고 새 기록이 supersede 관계를 남긴다.

---

## 3. Why / What / Now

문서를 세 질문으로 구분한다.

### Why — 왜 이 게임을 만드는가

- `START_HERE.md`
- `vision/USER_VISION.md`
- `vision/FIRST_PLAYABLE_WORLD.md`
- `vision/UI_DIRECTION.md`
- `design-history/*`

### What — 현재 무엇을 구현해야 하는가

- `architecture/decisions/ADR-*`
- `specs/*`
- `architecture/ARCHITECTURE.md`
- `planning/MILESTONES.md`

### Now — 지금 실제로 어디까지 왔는가

- `planning/STATUS.md`
- 해당 `evidence/*`
- `HANDOFF.md`

> **현재 branch, SHA, Run ID, test count는 Why 문서에 복제하지 않는다.**

자주 바뀌는 상태는 `STATUS.md`, 상세 증거는 `evidence/*`에만 둔다. README와 Vision에는 안정적인 제품 원칙과 링크만 남긴다.

---

## 4. Surface taxonomy

현재 프로젝트에는 여러 실행 화면이 있지만 목적은 다르다.

| Surface | 역할 | 정본 문서 |
|---|---|---|
| Runtime Baseline | GPU/window/allocation 기술 기준선 | architecture / G0 evidence |
| Observatory | subsystem 관찰 | 해당 Gate evidence |
| Benchmark Gallery | 고정 workload 사람이 검토 | G8-B Gallery evidence |
| Experiment Harness | 자동 screenshot·telemetry·report | Harness evidence |
| First Playable World | 실제 플레이어 조작과 실험 | `vision/FIRST_PLAYABLE_WORLD.md` |
| Final Presentation | 최종 시각·음향 언어 | `planning/PRESENTATION_ROADMAP.md` |

Gallery와 Observatory는 제품을 검증하는 실험실이지 최종 게임 UI가 아니다.

---

## 5. 디렉터리 지도

```text
docs/
├─ START_HERE.md
├─ README.md
├─ HANDOFF.md
├─ 00_USER_VISION.md
├─ 01_MASTER_DESIGN_REPORT.md
├─ vision/
│  ├─ USER_VISION.md
│  ├─ FIRST_PLAYABLE_WORLD.md
│  └─ UI_DIRECTION.md
├─ design-history/
│  ├─ 2026-08-15-foundation-design-session.md
│  └─ 2026-08-16-to-18-m0-evolution.md
├─ planning/
│  ├─ ROADMAP.md
│  ├─ PRESENTATION_ROADMAP.md
│  ├─ MILESTONES.md
│  ├─ TE3_WATER_STEAM_PHASE_ACCOUNTING.md
│  ├─ TE5_PHASE_VOLUME_PRESSURE_BRIDGE.md
│  ├─ TE5_LOCAL_VAPOR_CAPACITY_PRESSURE.md
│  ├─ TE5_PERSISTENT_VAPOR_EXTENT.md
│  └─ STATUS.md
├─ architecture/
│  ├─ ARCHITECTURE.md
│  ├─ THERMAL_ENVIRONMENT_PRODUCTION_INVENTORY.md
│  └─ decisions/ADR-*.md
├─ specs/
│  ├─ SIMULATION_SPEC.md
│  ├─ MATERIAL_SPEC.md
│  ├─ REACTION_SPEC.md
│  ├─ DETERMINISM_SPEC.md
│  ├─ THERMAL_ENVIRONMENT_SPEC.md
│  ├─ PHASE_THERMODYNAMICS_SPEC.md
│  ├─ PHASE_VOLUME_PRESSURE_BRIDGE_SPEC.md
│  ├─ LOCAL_VAPOR_CAPACITY_PRESSURE_SPEC.md
│  └─ PERSISTENT_VAPOR_EXTENT_SPEC.md
├─ development/
│  ├─ QUICKSTART.md
│  ├─ DEVELOPMENT.md
│  ├─ TESTING.md
│  ├─ VALIDATION_POLICY.md
│  ├─ THERMAL_ENVIRONMENT_VALIDATION.md
│  ├─ PHASE_THERMODYNAMICS_VALIDATION.md
│  ├─ PHASE_VOLUME_PRESSURE_BRIDGE_VALIDATION.md
│  ├─ LOCAL_VAPOR_CAPACITY_PRESSURE_VALIDATION.md
│  ├─ PERSISTENT_VAPOR_EXTENT_VALIDATION.md
│  ├─ PERFORMANCE.md
│  ├─ DEVELOPMENT_LEARNING_LOOP.md
│  ├─ LESSONS_LEDGER.md
│  └─ WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md
├─ evidence/
├─ adversarial-reviews/
└─ research/
   ├─ README.md
   ├─ raw/
   ├─ derived/
   ├─ encyclopedia/
   └─ materials/
```

미래의 Life, Agent, Civilization, Magic 등의 권위 문서는 필요가 실제로 생길 때 추가한다. 빈 abstraction이나 빈 SPEC을 미리 확장하지 않는다.

---

## 6. 문서별 역할

### Vision

- `USER_VISION.md`: 최상위 제품 원칙
- `FIRST_PLAYABLE_WORLD.md`: 첫 5분의 플레이 경험과 G9 입력
- `UI_DIRECTION.md`: Player Comprehension, Cell Inspector, debug/product UI 경계

기술적 편의로 Vision을 거꾸로 축소하지 않는다.

### Design History

결정의 결론뿐 아니라 다음을 보존한다.

- 질문과 선택지
- 사용자의 실제 선택
- 사용자가 수정한 가정
- 버린 대안과 이유
- benchmark 뒤로 미룬 항목
- superseded 관계
- 최종 반영 위치

중요한 사용자 발언은 의미를 바꾸지 않고 `DIRECT`, `User Principle`, `User Commentary`처럼 강도를 표시한다.

### ADR

구조 선택의 이유와 대안을 보존한다. 승인된 ADR을 조용히 다시 쓰지 않는다. 방향 변경은 새 ADR이 supersede한다.

### Specs

현재 구현자가 따라야 할 구체적인 계약이다. 과거 대화 없이도 구현할 수 있을 정도로 명확해야 한다.

### Planning

- `ROADMAP.md`: 장기 제품 방향과 증거 기반 작업 순서
- `PRESENTATION_ROADMAP.md`: Simulation Truth 위에 시각·음향 감각을 쌓는 순서
- `MILESTONES.md`: Evidence Gate와 사용자 승인 경계
- `STATUS.md`: 현재 실제 상태와 바로 다음 작업

### Development

개발 비용과 품질을 동시에 관리한다.

- 변경 영향 기반 validation
- targeted/FULL/bounded launch check/candidate 역할 분리
- append-only Lessons Ledger
- 단일 사용자 앱 EXE와 launcher
- worktree/artifact WIP limit
- task timing

### Evidence

`evidence/*`는 실행 결과, provenance, 자동 verdict, 사용자 승인과 scope boundary를 기록한다.

- 자동 PASS는 사용자 승인이 아니다.
- 한 scenario 결과는 다른 scenario나 G8-C를 승인하지 않는다.
- Review Packet은 human review용이며 forensic Audit Bundle과 역할이 다르다.
- historical/rejected/superseded artifact를 소급 수정하지 않는다.
- G8은 verified G8-C Matrix와 user dispositions를 포함해 **CLOSED / FROZEN**이다. G9-A Sandbox와 [`Thermal Environment`](planning/THERMAL_TRANSPORT_IGNITION_CAUSALITY.md) TE-2는 각각 **USER ACCEPTED WITH KNOWN FOLLOW-UP**이다. TE-3D는 [`ADR-0006`](architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md)의 **ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS**이며 ADR-0006은 **ACCEPTED FOR FUTURE ATOMIC IMPLEMENTATION**이다. D-019 [`TE-5B`](planning/TE5_PHASE_VOLUME_PRESSURE_BRIDGE.md), D-020 [`TE-5C`](planning/TE5_LOCAL_VAPOR_CAPACITY_PRESSURE.md), D-021 [`TE-5D`](planning/TE5_PERSISTENT_VAPOR_EXTENT.md)는 각각 REJECTED / DESIGN BLOCKED다. D-022 [`TE-5X`](planning/TE5_PRESSURE_VOLUME_ARCHITECTURE_RESET.md)는 세 모델을 동결했지만 유일한 통합 process가 후보 평가 전 oracle bootstrap에서 종료됐고 fresh review도 Critical 0 / High 11로 세 모델 모두를 부적격 판정했다. ADR-0010은 **PROPOSED / DESIGN BLOCKED**이고 추천 모델이 없다. 모든 TE-3/TE-5 runtime은 **NOT STARTED**이고 기존 증거는 source-bound다.

현재 진행 세부 사항은 `STATUS.md`와 해당 evidence 문서에서만 확인한다.

### Research

- `raw/`: 출처와 원문
- `derived/`: 현재 세계 문법으로 재가공한 후보
- `encyclopedia/`: 넓은 아이디어 corpus
- `materials/`: Material 정체성·의도·상호작용·Discovery 후보

Material Wiki는 개념 상태와 구현 상태를 구분한다. 수치와 threshold의 정본은 Rule Card/SPEC이다.

---

## 7. 문서 유지 규칙

> **요약하지 않는다. 정리한다.**

### 반복해도 되는 것

제품 North Star와 절대 원칙은 짧게 반복해도 된다.

- 나만의 세계 창조
- 본능적인 상호작용
- 거대한 스케일
- One Cell = Max One Matter
- Game-Consistent Minimum Physics
- 결과는 정직하게, 감각은 과장

### 한 곳에만 둘 것

- current branch/SHA
- Run ID와 artifact hash
- test count
- Gate 세부 상태
- next exact command

### 오래된 결론

- 삭제하거나 현재 결론처럼 섞지 않는다.
- historical, rejected, superseded, deferred를 명시한다.
- 새 결정이 어느 문서를 대체하는지 기록한다.

### 새 문서 생성 기준

새 문서는 다음 중 하나일 때만 만든다.

- 기존 문서와 다른 명확한 권위 역할이 있음
- 사용자 경험이나 구현 계약의 중요한 빈틈을 채움
- 반복되는 실수를 machine guard로 승격함
- 결정 provenance를 보존해야 함

작업 일지나 raw session log는 Git 밖의 artifact root에 둔다.

---

## 8. 현재 문서 개선의 목적

이 구조의 목표는 문서를 줄이는 것 자체가 아니다.

> 새 사람이나 AI/Codex가 **게임이 주려는 감정 → 현재 계약 → 지금 할 일** 순서로 빠르게 복구하도록 한다.

기술 Evidence가 제품 의도를 묻지 않게 하고, 제품 Vision이 실행 가능한 계약 없이 추상적인 슬로건에 머물지 않게 한다.
