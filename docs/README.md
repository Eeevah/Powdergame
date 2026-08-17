# Powdergame Documentation

> 이 디렉터리는 Powdergame의 제품 의도, 설계 결정, 구현 계약, 검증 기준과 결정 역사를 보존한다.
>
> **중요:** 이 프로젝트의 문서는 단순 요약본이 아니다. 현재의 정답뿐 아니라 왜 그 결정을 했는지, 사용자가 어떤 선택을 했고 어떤 전제를 수정했는지까지 추적할 수 있어야 한다.

## 문서 권위 순서

문서가 서로 충돌할 경우 다음 순서를 따른다.

1. `vision/USER_VISION.md` — 사용자가 원하는 게임의 최상위 제품 원칙
2. 최신 `architecture/decisions/ADR-*` — 명시적으로 승인된 구조적 결정과 변경 이력
3. `specs/*` — 현재 구현이 따라야 하는 구체적인 시뮬레이션/물질/반응/결정성 계약
4. 실제 검증 구현, 테스트, `evidence/*`, `architecture/ARCHITECTURE.md` — 실행 결과로 확인된 현재 시스템 진실
5. `planning/MILESTONES.md` — 무엇을 증명해야 완료인지 정의하는 Evidence Gate
6. `planning/STATUS.md` — 현재 실제 상태와 바로 다음 작업
7. `planning/ROADMAP.md` — 장기 제품 방향과 작업 순서
8. `HANDOFF.md` — 현재 canonical line을 이어받기 위한 실행 안내
9. `research/materials/*` — Material Wiki. 개념 상태와 구현 상태를 분리하며 별도 승인 전에는 구현 계약이 아님
10. `research/derived/*`, `research/encyclopedia/*` — 현재 세계 문법으로 재가공한 후보와 개발용 corpus
11. `research/raw/*` — 출처와 원문 보존
12. `01_MASTER_DESIGN_REPORT.md`, `design-history/*`, `00_USER_VISION.md` — 종합 맥락, 결정 provenance, 기존 경로 호환 요약
13. 초기 프로토타입/실험 코드

`development/*`는 개발·테스트·성능 측정 절차를 고정하지만, 위 권위 문서와 실제 검증 결과를 덮어쓰지 않는다.

### 현재 Q&A의 지위

2026-08-15 Foundation Design Session에서 사용자가 직접 선택·교정한 내용은 구현 전 단계의 초기 연구 가설보다 강한 설계 증거로 취급한다.

따라서 기존 README, `00_USER_VISION.md`, `01_MASTER_DESIGN_REPORT.md`도 현재 Q&A에 맞게 갱신했다. 과거 버전은 Git history가 보존한다.

`design-history/*`에는 superseded 결정도 포함될 수 있으므로 구현 코드가 직접 따라야 할 계약은 최신 ADR/SPEC이다. 다만 **왜 그 계약이 그렇게 되었는지, 사용자의 실제 의도가 무엇인지 해석해야 할 때는 Design History를 반드시 참고한다.**

## 디렉터리 구조

```text
docs/
├─ README.md
├─ 00_USER_VISION.md                  # 기존 경로 호환 / 최신 비전 요약
├─ 01_MASTER_DESIGN_REPORT.md         # 현재 Foundation 종합 설계 보고서
├─ vision/
│  └─ USER_VISION.md                  # 현재 최상위 제품 비전
├─ design-history/
│  └─ 2026-08-15-foundation-design-session.md
├─ planning/
│  ├─ ROADMAP.md
│  ├─ MILESTONES.md
│  └─ STATUS.md
├─ adversarial-reviews/
│  ├─ README.md                       # 선택적 적대적 리뷰 및 보존 규칙
│  └─ YYYY-MM-DD_<GATE_OR_SCOPE>.md   # 명시적으로 요청된 리뷰 기록
├─ evidence/
│  ├─ G5_*.md / G6_*.md / G7_*.md    # 이전 Gate evidence와 사용자 승인 기록
│  ├─ G8_A_MEASUREMENT_SUBSTRATE_2026-08-17.md
│  ├─ G8_B_BENCHMARK_SCENARIO_GALLERY_2026-08-17.md
│  └─ G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md
├─ architecture/
│  ├─ ARCHITECTURE.md
│  └─ decisions/
│     ├─ ADR-0001-world-cell-invariants.md
│     ├─ ADR-0002-gpu-authoritative-local-simulation.md
│     ├─ ADR-0003-minimum-sufficient-physics.md
│     └─ ADR-0004-approximate-determinism-and-arbitration.md
├─ specs/
│  ├─ SIMULATION_SPEC.md
│  ├─ MATERIAL_SPEC.md
│  ├─ REACTION_SPEC.md
│  └─ DETERMINISM_SPEC.md
├─ development/
│  ├─ DEVELOPMENT.md
│  ├─ TESTING.md
│  └─ PERFORMANCE.md
├─ research/
│  ├─ README.md                        # research authority/index
│  ├─ raw/                             # 원문 보존
│  ├─ derived/                         # 현재 세계 문법으로 재가공한 후보
│  ├─ encyclopedia/                    # 넓은 아이디어 corpus
│  └─ materials/                       # 물질별 개념 Wiki
│     ├─ README.md
│     ├─ _TEMPLATE.md
│     ├─ foundation/                   # 기본 16종 Material 개념/family Wiki
│     └─ p1/                           # 첫 geology/manufacture prototype family
└─ HANDOFF.md
```

미래의 Life, Agent, Civilization, Magic 등의 권위 문서는 필요해질 때 추가한다. 아직 구현하지 않는 계층을 빈 코드/빈 SPEC으로 미리 확장하지 않는다. Research에는 장기 후보를 보존할 수 있지만, 존재만으로 구현 범위가 되지는 않는다.

## 문서 역할

### Vision

`vision/USER_VISION.md`는 **무엇을 만들고 싶은가**를 정의한다. 기술적 편의 때문에 이 문서를 거꾸로 축소하지 않는다.

### Master Design Report

`01_MASTER_DESIGN_REPORT.md`는 Foundation 설계를 한 번에 읽을 수 있도록 통합한 종합 보고서다. 넓은 맥락을 제공하지만 세부 구현에서 SPEC/ADR과 충돌하면 더 구체적이고 최신인 SPEC/ADR이 우선한다.

### Design History

`design-history/*`는 **어떻게 그 결론에 도달했는가**를 보존한다.

가능한 경우 다음을 남긴다.

- Assistant가 던진 설계 질문
- 제시된 주요 선택지
- 당시 추천안
- 사용자가 실제 선택한 답
- 사용자가 추가한 조건/반례/교정
- 논의를 통해 바뀐 최종안
- 선택하지 않은 대안과 이유
- 성능 측정 후 재검토하기로 한 항목
- superseded된 이전 결정
- 최종적으로 어느 SPEC/ADR에 반영되었는지

중요한 사용자 발언은 `User Principle` 또는 `User Commentary`로 원래 의미를 최대한 유지해 기록한다.

### ADR

ADR은 **왜 이 구조를 선택했는가**를 보존한다. 승인된 과거 ADR은 조용히 다시 쓰지 않는다. 구현 이후 방향이 바뀌면 새 ADR이 이전 ADR을 supersede한다.

### Specs

SPEC은 **현재 구현 계약**이다. 구현자가 과거 대화를 읽지 않아도 코드를 작성할 수 있을 정도로 구체적이어야 한다.

### Planning

- `ROADMAP.md`: 장기 제품 방향과 증거 기반 작업 순서. 일정표는 아니다.
- `MILESTONES.md`: Evidence Gate. 기능 체크리스트가 아니라 증명 계약.
- `STATUS.md`: 지금 실제로 어디까지 되었고 바로 다음 작업이 무엇인가.

### Development

개발/테스트/성능 문서는 구현 절차와 측정 철학을 고정한다. 특히 성능 최적화는 추측이 아니라 실제 benchmark 증거를 기반으로 한다.

### Evidence

`evidence/*`는 Gate별 구현·측정·사용자 수용 경계를 기록한다. 현재 G8-B Gallery 문서는 다섯 official workload와 별도 G7 Active/Sleep 회귀 fixture의 shared staging/Windows Gallery/headless selection 구현 candidate를 설명한다. Sand Fall Harness v0 문서는 승인된 Scenario 1 lifecycle을 저장소 밖의 immutable run으로 기록하는 runner, telemetry, verdict, artifact, receipt-last 계약을 설명하며 pilot/final validation은 pending으로 유지한다. Scenario 1 Sand Fall은 사용자 승인되었고 Scenario 2~5는 미승인이라 전체 기록은 **USER ACCEPTANCE PENDING / NOT CLOSED** 상태다. Gallery diagnostics, Harness 자동 `PASS`, targeted test가 G8-C official timing 또는 남은 사용자 승인을 대신하지 않는다.

### Adversarial Reviews

`adversarial-reviews/*`는 이미 작성된 적대적 검토를 비차단 이력으로 보존한다. 적대적 리뷰는 기본 절차가 아니며 사용자가 명시적으로 요청한 경우에만 수행·기록한다. 외부 AI reviewer에게 프로젝트 내용을 보내지 않으며, 보고서 자체는 commit/push/PR/release 또는 gate closure 권한을 부여하지 않는다.

### Research

`research/*`는 넓은 조사자료와 콘텐츠 후보를 보존하고, 현재 ADR/SPEC에 맞춰 단계적으로 좁힌다.

- `raw/`: 출처와 원문을 가능한 한 보존
- `derived/`: behavior family, shortlist, interaction graph, prototype Rule Card
- `encyclopedia/`: 현실·역사·창작 소재를 폭넓게 추적하는 개발용 corpus
- `materials/`: 물질마다 **어떤 개념인지, 왜 넣는지, 무엇과 상호작용하는지, 현실과 게임 추상화의 경계가 무엇인지** 관리하는 개념 Wiki

Material Wiki는 개념 상태와 구현 상태를 분리한다. 숫자·threshold는 최신 Rule Card/SPEC에 두고, 개별 페이지는 정체성·의도·관계·Discovery와 미결정 사항을 보존한다.

## 핵심 문서화 원칙

> **요약하지 않는다. 정리한다.**

대화의 중복 표현과 말버릇은 다듬어도, 다음 정보는 임의로 버리지 않는다.

- 왜 그런 결정을 했는가
- 어떤 대안이 있었는가
- 사용자가 무엇을 직접 선택했는가
- 사용자가 어떤 가정을 수정했는가
- 어떤 부분은 아직 benchmark가 필요해 확정하지 않았는가
- 어떤 결정이 나중에 superseded되었는가

이 문서 집합의 목표는 새로운 사람이나 AI/Codex 세션이 읽었을 때 **결론뿐 아니라 사용자의 의도와 설계의 경계까지 복구할 수 있게 하는 것**이다.
