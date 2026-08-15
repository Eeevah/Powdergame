# Powdergame Research Archive

이 디렉터리는 Powdergame 개발에 참고할 외부 조사·아이디어·후보값을 보존한다.

## 권위

`docs/research/*`는 **구현 계약이 아니다.**

문서가 서로 충돌할 경우 `docs/README.md`의 기존 권위 순서를 따른다. 특히 현재 승인된 `vision/USER_VISION.md`, 최신 ADR, `specs/*`, `planning/MILESTONES.md`가 이 디렉터리보다 우선한다.

Research 문서의 수치·아키타입·규칙·Material 후보는 다음 중 하나로 취급한다.

- `REFERENCE` — 현실/창작물에서 얻은 참고 정보
- `CANDIDATE` — Powdergame에 맞게 검토할 후보
- `DERIVED` — 기존 research를 현재 ADR/SPEC에 맞춰 재가공한 제안
- `ADOPTED` — 별도 ADR/SPEC/승인 문서로 실제 채택된 항목. research 문서 자체만으로는 이 상태가 되지 않는다.

## 사용 원칙

1. 현실 SI 값과 Powdergame 내부 값을 동일시하지 않는다.
2. `density_rank`, thermal class, ignition threshold 등 조사 보고서의 게임값은 자동으로 확정하지 않는다.
3. 현재 M0 범위를 넘어서는 Electricity / Radiation / Light / Biology / Information / Magic / Space-Time / Meta 아이디어는 미래 후보로 보존한다.
4. 유명 창작물의 이름·설정은 `REFERENCE_ONLY`로 보고, 실제 게임 콘텐츠에는 행동 원리만 추출해 독자적인 Matter로 재설계하는 것을 기본으로 한다.
5. 외부 조사에서 현재 ADR/SPEC과 충돌하는 제안은 그대로 구현하지 않고 충돌을 명시한다.
6. Material 하나를 위해 무거운 범용 시스템을 추가하지 않는다. Interaction Yield와 기존 시스템 재사용성을 우선한다.

## 2026-08-16 intake

다섯 개의 주요 조사/후보 자료를 검토 대상으로 받았다.

- 현실 물성 → Powdergame 정규화 중심의 Material & Phenomenon 조사
- 현실 재료 / 기존 창작물 / 오리지널 판타지·SF 재료를 함께 다룬 종합 재료 조사
- 기존 창작물 100종의 행동 문법과 Original Matter 100종을 다룬 가상 물질 동역학 조사
- 기존 후보에 capacity/fatigue/counter/byproduct를 보강하고 VX-001~010 및 OM-101~180까지 확장한 가상물질 통합 설계 조사
- 현실·우주·창작물에서 약 230개 항목을 모은 human-readable `MATERIAL_CANDIDATES.md` 후보 노트

앞선 세 자료의 출처 정보, SHA-256, 현재 계약과의 충돌 및 우선 활용 영역은 `2026-08-16-material-research-intake.md`에 기록한다.

네 번째 확장 자료는 `2026-08-16-expanded-fictional-matter-intake.md`에 별도로 기록한다.

다섯 번째 후보 노트는 원문을 `raw/MATERIAL_CANDIDATES.md`로 보존하고, 현재 SPEC 기준 분석을 `2026-08-16-material-candidates-analysis.md`에 기록한다.

## Derived outputs

원자료를 현재 Powdergame 세계 문법에 맞게 압축한 결과는 `derived/`에 둔다. 이 문서들도 여전히 구현 계약이 아니다.

- `derived/MATERIAL_BEHAVIOR_FAMILIES.md` — 약 230개 후보와 앞선 OM 연구를 24개 행동 family로 압축하고, foundation-compatible / near-term / future / meta-defer를 구분한다.
- `derived/FIRST_EXPANSION_MATERIAL_SHORTLIST.md` — M0를 건드리지 않고 그 이후 첫 확장에 적합한 저비용·고상호작용 Material과 prototype bundle을 정리한다.
- `derived/MATERIAL_PROTOTYPE_BUNDLES.md` — Dry Ice/CO2, Clathrate/Methane, Clay/Brick, Cryofluid/Ablative, Salt/Brine을 실제 causal-chain 검증 묶음으로 정의하고 prerequisites와 pass evidence를 적는다.
- `derived/MATERIAL_SELECTION_FRAMEWORK.md` — **현실의 특징 강한 물질 → 역사적 자연관/연금술 → interaction gap → 필요한 만큼만 오리지널 Matter** 순서로 후보를 줄이는 interaction-first 선정 원칙.
- `derived/CURATED_INTERACTION_CORE.md` — 위 선정 원칙을 실제 후보에 적용해 현실 물질 중심의 수십 종 core와 소수의 original gap-filler를 제안한다.

## Master Encyclopedia

`encyclopedia/`는 위 raw/derived 자료를 다시 모아 **두들갓처럼 읽히는 개발용 통합 사전**으로 만든 corpus다.

- `encyclopedia/README.md` — 사전 구조와 canonicalization 규칙
- `encyclopedia/MASTER_INDEX.md` — 전체 인덱스
- 현재 **530 source-index rows**를 추적한다.
- 이 숫자는 중복, 별칭, skip 후보, IP reference까지 보존한 provenance 숫자이며 최종 Material 수가 아니다.
- Foundation / 현실 재료 / 우주 후보 / OM-001~180 / VX-001~010 / reference quarry / Doodle-God World Primitive / web reality anchor를 한 체계 안에서 검색할 수 있다.
- 설명이 약한 항목은 기존 Powdergame 법칙 → 현실 조사 → 독자 상상 순서로 보강한다.
- 실제 게임 Dictionary는 이 master corpus를 그대로 공개하는 답안지가 아니라 **플레이어가 관찰한 현상만 드러나는 발견 도감**으로 파생해야 한다.

핵심 변환은 다음과 같다.

```text
source name / lore
→ lore-free mechanic
→ behavior family
→ duplicate merge
→ real/historical coverage check
→ only then original gap-filler
→ user review
→ ADR / SPEC / content registration
```

## 권장 변환 파이프라인

```text
Raw research
  ↓
Source / provenance audit
  ↓
Current ADR/SPEC conflict check
  ↓
Candidate extraction
  ├─ high-interaction real Matter
  ├─ historical / philosophical archetype
  ├─ original gap-filler
  └─ future / meta
  ↓
Powdergame normalization
  ↓
Human review / user approval
  ↓
ADR / SPEC / content data
  ↓
IMPLEMENTED
  ↓
Evidence Gate / play validation
  ↓
VALIDATED
```

핵심은 research를 정답으로 취급하지 않고 **좋은 원재료로 사용해 현재 Powdergame 세계 문법에 맞게 다시 만드는 것**이다.