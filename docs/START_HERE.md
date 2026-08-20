# Powdergame — Start Here

> **내가 규칙을 정한 거대하고 아름다운 세계에서, 작은 규칙들이 내가 예상하지 못한 연쇄를 만드는 것을 보는 게임.**

이 문서는 Powdergame의 가장 짧은 제품 입구다. 기술 구현, Gate, Evidence, SHA를 읽기 전에 **왜 이 게임을 만드는지**를 먼저 복구하기 위해 존재한다.

현재 상태의 세부 수치와 다음 명령은 `planning/STATUS.md`가 정본이다. 이 문서는 자주 바뀌는 상태를 복제하지 않는다.

---

## 1. 플레이어에게 약속하는 경험

Powdergame은 단순히 물질 목록을 제공하는 falling-sand toy가 아니다. 플레이어에게 **세계를 발명할 수 있는 문법**을 제공한다.

플레이어가 느껴야 하는 핵심은 네 가지다.

1. **만지면 바로 반응한다**  
   물질을 놓고, 지우고, 가열하고, 구조를 만들면 세계가 즉시 답한다.

2. **예측할 수 있지만 놀랍다**  
   개별 규칙은 배울 수 있어야 하지만, 여러 규칙이 만나면 예상하지 못한 연쇄가 생겨야 한다.

3. **다음 질문이 계속 생긴다**  
   “이것을 저기에 넣으면 무슨 일이 일어날까?”가 자연스럽게 이어져야 한다.

4. **내가 만든 세계라는 감각이 든다**  
   정답표의 빈칸을 채우는 것이 아니라, 플레이어가 가설을 세우고 자기 연구 기록을 만든다.

---

## 2. 이 게임이 지키는 North Star

### 나만의 세계 창조

현실은 직관과 아이디어의 출발점이지 법전이 아니다. 가상의 Matter와 가상의 법칙도 게임 안에서 이해되고, 반복해서 배울 수 있고, 재미있는 상호작용을 만든다면 유효하다.

### 본능적인 상호작용

콘텐츠 수보다 중요한 것은 Matter와 Field가 서로 영향을 주는 정도다.

```text
Water 가열
→ Steam
→ 공간 부족
→ Pressure
→ Wood 파열
→ opening
→ vent
→ 주변 Matter와 또 다른 반응
```

이 연쇄를 `boiler_explosion()` 같은 전용 결과 코드로 만들지 않고, 작은 공통 규칙들의 결과로 만들려 한다.

### 거대한 스케일

성능은 엔지니어링 점수가 아니라 제품 비전이다.

```text
극도로 싼 Cell
× 수백만 Cell의 GPU 병렬 실행
= 동시에 많은 일이 살아 있는 큰 세계
```

절약한 계산 예산은 더 큰 세계, 더 많은 동시 반응, 더 나은 발견·Rewind·Presentation에 다시 투자한다.

### 결과는 정직하게, 감각은 과장한다

Simulation Truth와 Presentation Effect를 분리한다. Cell은 계산 단위이지 최종 화면의 미술 해상도 제한이 아니다. 세계의 인과는 실제 simulation에서 나오고, 불꽃·연기·열·빛·충격파의 감각은 더 풍부하게 표현할 수 있다.

---

## 3. 지금 보이는 화면은 무엇인가

현재 저장소에는 서로 다른 목적의 사용자 표면이 함께 존재한다. 이름이 비슷해도 역할은 다르다.

| Surface | 역할 | 제품인가? |
|---|---|---:|
| Runtime Baseline | GPU·window·allocation이 살아 있는지 확인하는 기술 기준선 | 아니오 |
| Observatory | Thermal, Pressure, Active/Sleep 같은 subsystem을 관찰 | 아니오 |
| Benchmark Gallery | 고정 fixture의 거동과 대표성을 사람이 검토 | 아니오 |
| Experiment Harness | screenshot·telemetry·report·evidence를 자동 생성 | 아니오 |
| First Playable World | 플레이어가 직접 Matter를 놓고 가설을 시험 | **첫 제품 형태** |
| Final Presentation | Simulation Truth 위에 최종 시각·음향 언어를 구축 | 제품 발전 단계 |

> **Gallery와 Observatory를 잘 만드는 것은 게임을 완성하는 것과 다르다.**

현재 G8의 도구들은 세계 법칙을 검증하기 위한 실험실이다. M0의 최종 관문은 G9에서 실제로 조작 가능한 First Playable World를 만들고 사용자가 직접 플레이하는 것이다.

---

## 4. 절대 잃으면 안 되는 계약

- **One Cell = Max One Matter** — 복잡성은 셀 안의 혼합물이 아니라 수많은 단순 셀의 공간적 상호작용에서 나온다.
- **GPU Production Simulation이 truth** — CPU reference와 AI 설명은 보조 수단이다.
- **Read Neighbors → cheap local rule → Write Self Next** — 소유권 변화만 최소 Claim/Resolve를 사용한다.
- **Game-Consistent Minimum Physics** — 현실 방정식보다 이해 가능하고 재미있는 최소 상태·연산을 선택한다.
- **Non-exact but stable** — bit-perfect replay보다 유효하고 안정적인 결과와 성능이 우선이다.
- **사용자 승인 없는 ACHIEVED 없음** — 자동 PASS는 사용자 경험이나 전체 Gate 종료를 대신하지 않는다.

---

## 5. 작업별 최소 읽기 경로

모든 작업에서 20개 문서를 먼저 읽지 않는다. 작업 종류에 맞는 최소 경로를 읽고, 충돌이 있을 때 권위 문서로 확장한다.

### 제품·UX·콘텐츠 방향

1. `START_HERE.md`
2. `vision/USER_VISION.md`
3. `vision/FIRST_PLAYABLE_WORLD.md`
4. `vision/UI_DIRECTION.md`
5. 필요한 경우 `design-history/*`

### Engine·Simulation 구현

1. `START_HERE.md`
2. 관련 `architecture/decisions/ADR-*`
3. 관련 `specs/*`
4. `architecture/ARCHITECTURE.md`
5. 관련 tests/evidence

Thermal Environment 작업은 ADR-0005,
`specs/THERMAL_ENVIRONMENT_SPEC.md`,
`architecture/THERMAL_ENVIRONMENT_PRODUCTION_INVENTORY.md`,
`development/THERMAL_ENVIRONMENT_VALIDATION.md`,
`planning/THERMAL_ENVIRONMENT_IMPLEMENTATION_GATES.md` 순서로 읽는다.
현재 TE-1 Environment state / occupancy hygiene는 구현됐고, Air transport와
thermal exchange를 시작하는 TE-2는 별도 승인 전 **NOT STARTED**다.

### 현재 Gate를 이어서 개발

1. `development/QUICKSTART.md`
2. `planning/STATUS.md`
3. 해당 Gate의 `evidence/*`
4. `HANDOFF.md`
5. 변경 경로에 해당하는 SPEC/ADR

### 검증·Evidence·성능

1. `development/VALIDATION_POLICY.md`
2. `development/TESTING.md`
3. `development/PERFORMANCE.md`
4. 해당 `evidence/*`
5. `development/WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md`

### Material 연구

1. `vision/USER_VISION.md`
2. `specs/MATERIAL_SPEC.md`
3. `specs/REACTION_SPEC.md`
4. `research/README.md`
5. 관련 `research/materials/*`

---

## 6. 다음 제품 질문

기술 Gate의 최종 목적은 다음 질문에 답하는 것이다.

> **플레이어가 직접 세계를 만졌을 때, “다음에는 무엇을 해볼까?”라는 생각이 자연스럽게 드는가?**

이를 구체화한 첫 5분의 경험은 `vision/FIRST_PLAYABLE_WORLD.md`, UI의 정보 공개와 Inspector 원칙은 `vision/UI_DIRECTION.md`, 시각적 발전 순서는 `planning/PRESENTATION_ROADMAP.md`를 따른다.

---

## 7. 문서 권위

이 문서는 제품 의도를 빠르게 복구하는 입구다. 충돌 시 정본은 다음과 같다.

- 제품 원칙: `vision/USER_VISION.md`
- 승인된 구조 결정: 최신 `architecture/decisions/ADR-*`
- 구현 계약: `specs/*`
- 실제 현재 상태: `planning/STATUS.md`
- 완료 기준: `planning/MILESTONES.md`
- 실행 증거: tests와 `evidence/*`
- 결정 맥락: `design-history/*`

자세한 문서 지도와 유지 규칙은 `README.md`를 따른다.
