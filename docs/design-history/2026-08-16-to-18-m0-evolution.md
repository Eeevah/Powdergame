# M0 Evolution — 2026-08-16 to 2026-08-18

> Foundation 설계 이후 실제 구현·사용자 관찰·적대적 리뷰를 거치며 제품 해석과 개발 방식이 어떻게 바뀌었는지 보존한다.

이 문서는 작업 로그 전체를 복제하지 않는다. 사용자의 교정이 장기 계약으로 바뀐 지점, 선택하지 않은 대안, 현재 정본 문서만 남긴다.

표기:

- **DIRECT** — 사용자의 명시적 발언이나 승인
- **VERIFIED** — code/test/evidence로 확인된 사실
- **INTERPRETATION** — 여러 기록을 합친 제품 해석
- **SUPERSEDED** — 당시에는 유효했으나 이후 결정으로 대체

현재 상태와 SHA는 `planning/STATUS.md`, 세부 증거는 `evidence/*`가 정본이다.

---

## 1. 고정 Demo만 봐도 되는가? — DIRECT / APPROVED

### 문제

G0–G7 동안 사용자는 중간중간 실행 파일로 실제 장면을 보고 물리가 의도대로 읽히는지 확인했다. 자동 테스트와 Evidence가 강해지면서 사람이 더 이상 직접 볼 필요가 없는지 의문이 생겼다.

### 사용자 방향

큰 단계마다 실제 장면을 보아야 하며, 자동 결과와 함께 screenshot/리뷰를 확인해야 한다.

### 최종 계약

- 자동 검증은 correctness와 무결성을 담당한다.
- Harness는 관찰해야 할 장면을 압축한다.
- 사용자는 모든 tick을 수동으로 찍지 않고 Contact Sheet를 먼저 본다.
- 애매하거나 제품적으로 중요한 부분만 Gallery에서 직접 본다.
- 자동 verdict는 사용자 승인을 대신하지 않는다.

### 반영

- Experiment Harness
- `development/VALIDATION_POLICY.md`
- `development/LESSONS_LEDGER.md`
- `evidence/G8_B_*_HARNESS_*`

---

## 2. 수동 screenshot 반복 → Experiment Evidence Harness — DIRECT / APPROVED

### 문제

매 scenario마다 다음을 반복했다.

```text
BAT 실행
→ tick을 기다림
→ 주요 시점 screenshot
→ HUD 숫자 옮겨 적기
→ 파일 정리
→ 사용자/ChatGPT 검토
```

### 사용자 교정

실험 목표, 주요 screenshot, telemetry, 결과 분석과 추가 관찰 추천을 자동으로 생성해야 한다. 앞으로 실험 수가 늘어날 때 수동 방식은 유지 불가능하다.

### 채택 결정

Harness는 Matter를 만드는 도구가 아니다.

```text
이미 설계된 fixture/Rule
→ 실제 GPU Production Simulation
→ screenshot + telemetry
→ deterministic predicate
→ Contact Sheet / Report / Review Packet
→ 사용자 판단
```

### 선택하지 않은 대안

- v0에서 AI API를 runtime에 내장
- OCR로 HUD 숫자 재인식
- 모든 Material 조합을 자동 생성·탐색
- Interaction Lab 전체를 G8 전에 구현

### 반영

- `apps/windows/src/experiment*`
- `tools/experiment/*`
- external artifact root
- receipt-last / no-overwrite / immutable candidate

---

## 3. Sand 정지는 결함이 아니라 성공 — DIRECT / APPROVED

### 관찰

Sand Fall은 약 1,600 tick 전후에 거시적으로 멈추고 최종적으로 모든 chunk가 Sleep에 들어갔다.

### 사용자 해석

Sand가 쌓이고 멈추는 것이 실험 목표이므로 완전 정지는 성공이다.

### 제품 의미

Benchmark 장면을 계속 움직여 보이게 만들기 위해 physics나 fixture를 왜곡하지 않는다.

```text
Active fall
→ settling tail
→ complete sleep
```

각 구간의 의미를 분리한다.

### 반영

- Sand acceptance evidence
- automatic all-sleep lifecycle
- `LESSONS_LEDGER.md`

---

## 4. Water의 영구 미세 움직임 — DIRECT / APPROVED WITH FOLLOW-UP

### 관찰

외부 유출 fixture 문제를 고친 뒤 Water는 거시적으로 정착했지만 Water/Empty 자유 표면의 소수 Cell이 계속 local movement를 만들었다.

### 사용자/리뷰 판정

- conservation, destination, reset, integrity는 통과
- production physics defect 증거 없음
- automatic `NEEDS_HUMAN_REVIEW`는 그대로 유지
- human `ACCEPTED WITH KNOWN FOLLOW-UP`

### 제품 의미

자동 기준을 억지로 완화하거나 all-sleep을 강제하지 않는다. 작은 M0 liquid artifact는 기록하고, 실제 비용은 G8-C에서 측정한다.

---

## 5. Fire: automatic PASS와 사용자 승인은 별개 — DIRECT / APPROVED

### 결과

Fire / Heat candidate는 finite fuel, Smoke, phase work, Reaction 종료, Thermal tail, reset을 자동 PASS로 기록했다.

### 사용자 승인 경계

자동 PASS는 Fire fixture에 대한 기술 주장이다. 사용자가 Contact Sheet와 causal chain을 본 뒤 별도로 승인했다.

### 제품 의미

```text
automatic verdict
≠ human acceptance
≠ G8-B closure
≠ product readiness
```

이 구분은 이후 모든 scenario와 Milestone에 유지한다.

---

## 6. Pressure: 결과 milestone만으로 인과를 증명할 수 없음 — VERIFIED / ADOPTED

### 첫 문제

초기 detector는 Wood seam 일부가 사라진 것을 완전 opening으로 오인했다. through-lane topology 검출로 수정했다.

### 두 번째 문제

정확한 opening tick을 찾자 top Wood가 ignition temperature에서 시작했고 burn duration과 opening tick이 겹친다는 confound가 드러났다.

### 사용자/리뷰 결정

Pressure fixture가 증명해야 하는 named chain은 다음이다.

```text
Pressure
→ structural stress
→ rupture
→ through opening
→ vent
→ Pressure relief
```

다른 subsystem인 combustion이 같은 opening을 만들 수 있으면 결과가 보여도 acceptance evidence가 아니다.

### remediation

- top seam authored temperature만 ignition 아래로 변경
- geometry/Pressure/production physics 유지
- seam combustion/fuel telemetry 추가
- `pressure_opening_precedes_combustion` predicate 추가

### 누적 교훈

Scenario acceptance는 milestone 존재뿐 아니라 **이름 붙인 causal chain과 confound 배제**를 요구한다.

---

## 7. 실행파일과 launcher 증식 — DIRECT / APPROVED

### 문제

Gate마다 worktree, target, BAT, EXE가 늘어나 사용자가 어떤 파일을 실행해야 하는지 찾기 어려워졌다.

### 사용자 결정

사용자가 찾는 앱 실행파일은 하나로 유지한다.

```text
<active-worktree>/target/release/powdergame-windows.exe
```

정식 진입점:

```text
run_powdergame.bat
run_experiment.bat   # 자동 실험 예외
```

새 Scenario와 Observatory는 같은 앱의 mode/CLI로 추가한다.

### forensic 예외

Evidence 계약이 요구하는 frozen binary는 Run/Audit Bundle 안에 보존할 수 있지만 사용자 앱이 아니다.

### 반영

- `development/WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md`
- `config/development-policy.json`
- `tools/dev.ps1 audit`

---

## 8. 무인자 실행의 빈 G0 화면 — DIRECT / PRODUCT CORRECTION

### 관찰

사용자가 canonical launcher를 더블클릭했는데 staged world가 아니라 단색 G0 runtime baseline만 보였다.

### 해석

기술적으로 정상인 diagnostic baseline이 사용자 기본 진입점이 된 UX 결함이다.

### 결정

- 무인자 user-facing default는 의미 있는 Gallery 또는 이후 First Playable이어야 한다.
- G0 empty runtime은 explicit diagnostic flag로만 접근한다.
- smoke가 통과했다는 사실은 실제 no-argument user path가 유용하다는 증거가 아니다.

### 제품 의미

> 기술 기준선과 제품 기본값을 분리한다.

---

## 9. 색만으로 Matter를 이해할 수 없음 — DIRECT / PRODUCT REQUIREMENT

### 관찰

Scenario가 복잡해지면서 사용자는 화면을 보아도 어떤 Matter인지, Temperature/Pressure가 어떤지 구분하기 어려워졌다.

### 사용자 요청

- hover하면 간단한 Material 이름
- 선택적으로 켜는 detail
- Temperature 등 현재 상태

### 채택 방향

Cell Inspector v0:

```text
Hover → Material name
I → detailed current Cell state
```

Inspector는 latest diagnostic sample을 사용하고 sample tick을 정직하게 표시한다. mouse move마다 full GPU readback을 만들지 않는다.

### 반영

- `vision/UI_DIRECTION.md`
- `vision/FIRST_PLAYABLE_WORLD.md`
- `planning/PRESENTATION_ROADMAP.md`

---

## 10. 모든 변경에 FULL을 돌리는 관행 — SUPERSEDED

### 실측

Warm build는 약 0.3초였지만 serial workspace GPU/integration test는 약 310.84초였다.

### 과거 관행

```text
FAST
→ FULL
→ smoke
→ candidate
```

을 변경 위험과 무관하게 반복했다.

### 새 계약

변경 영향에 따라 검증한다.

- docs-only: docs/audit/diff
- harness/CLI: targeted + minimal smoke + candidate
- fixture-only: fixture pin + bounded GPU + candidate
- engine/WGSL/layout/Cargo/shared reset: FULL 포함

같은 source SHA의 성공 검증은 역할이 같다면 재사용한다.

### 반영

- `development/VALIDATION_POLICY.md`
- `config/development-policy.json`
- `tools/dev.ps1 validation-plan`

---

## 11. 실수를 누적하는 구조 — DIRECT / APPROVED

### 사용자 방향

반복되는 실수와 시행착오에서 교훈을 뽑고, 다음 작업에서 자연스럽게 더 잘하게 만들어야 한다.

### 채택 구조

```text
Observe
→ Classify
→ Fix one
→ Verify
→ Promote
→ Sweep
→ Retire
```

승격 조건:

- 같은 문제 2회 이상
- 한 번에 15분 이상 손실
- source/provenance/artifact 위험
- 사용자가 standing rule로 지정

### 반영

- append-only `development/LESSONS_LEDGER.md`
- machine-readable policy
- CI audit
- session timing

---

## 12. 현재 제품 해석

### 현재 G8 도구

- 세계 규칙과 workload를 검증하는 실험실
- 실제 제품 UI가 아님
- final presentation이 아님

### M0의 진짜 남은 관문

G9 First Playable World에서 사용자가 직접:

- Matter를 놓고
- 지우고
- 가열하고
- 구조를 만들고
- Inspector로 관찰하고
- 다음 실험을 시작해야 한다.

### North Star 재확인

> **내가 규칙을 정한 거대하고 아름다운 세계에서, 작은 규칙들이 내가 예상하지 못한 연쇄를 만드는 것을 보고 싶다.**

---

## 13. 현재 정본 반영 위치

- 빠른 제품 입구: `../START_HERE.md`
- 최상위 비전: `../vision/USER_VISION.md`
- 첫 플레이 경험: `../vision/FIRST_PLAYABLE_WORLD.md`
- UI/Inspector: `../vision/UI_DIRECTION.md`
- Presentation 순서: `../planning/PRESENTATION_ROADMAP.md`
- 현재 상태: `../planning/STATUS.md`
- 개발 학습: `../development/DEVELOPMENT_LEARNING_LOOP.md`
- 누적 교훈: `../development/LESSONS_LEDGER.md`
- 검증 비용: `../development/VALIDATION_POLICY.md`
- 실행파일/폴더: `../development/WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md`
