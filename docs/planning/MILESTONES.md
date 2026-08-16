# Powdergame Milestones

이 문서는 기능 체크리스트가 아니라 **Evidence Gate** 계약이다.

Milestone은 “코드가 존재한다”가 아니라 **실제로 원하는 세계가 성립한다는 증거가 있다**는 뜻이다.

---

## 1. Milestone Types

Milestone은 번호 체계를 하나로 유지한다.

```text
M0
M1
M2
...
```

각 Milestone은 type을 가진다.

- `Delivery` — 실제 기능/제품 단위 증명
- `Investigation` — 중요한 기술 가설/대안을 측정하고 결론 내림

번호를 Delivery/Investigation별로 따로 나누지 않는다.

---

## 2. Status

공식 상태:

- `PLANNED`
- `IN_PROGRESS`
- `BLOCKED`
- `VALIDATION`
- `ACHIEVED`
- `REGRESSION`

### 의미

`PLANNED`  
아직 시작하지 않음.

`IN_PROGRESS`  
구현/조사 진행 중.

`BLOCKED`  
명확한 blocker가 있음.

`VALIDATION`  
자동/기술 검증은 충분히 진행되어 사용자 확인을 기다리거나 최종 증거를 확인하는 단계.

`ACHIEVED`  
Evidence Gate를 통과했고 **사용자가 완료를 승인**함.

`REGRESSION`  
과거에는 통과했지만 이후 변경으로 계약을 만족하지 못하는 상태.

과거 ACHIEVED 기록은 삭제하지 않는다. Regression으로 이력을 남긴다.

---

## 3. User Approval Rule

AI, Codex, CI, benchmark는 Milestone을 `VALIDATION`까지 올릴 수 있다.

> **최종 `ACHIEVED`는 사용자 승인 없이는 선언하지 않는다.**

특히 Product/UX Milestone은 자동 테스트만으로 완료되지 않는다.

---

## 4. Common Validation Dimensions

각 Milestone은 필요한 항목에 대해 증거를 남긴다.

- Functional Correctness
- Reproducibility / Behavioral Stability
- Performance
- UX / Product Validation
- Persistence / Compatibility
- Failure / Fallback
- Regression

해당되지 않는 항목은 `N/A`로 둘 수 있으나 이유를 적는다.

Bit-perfect determinism을 기본 correctness 기준으로 사용하지 않는다. `DETERMINISM_SPEC.md`를 따른다.

---

# M0 — First World

**Type:** Delivery  
**Status:** IN_PROGRESS  
**Purpose:** 첫 GPU 세계가 Powdergame의 핵심 철학을 실제로 증명하는가.

M0의 목적은 콘텐츠를 많이 넣는 것이 아니다.

> **수백만 개의 아주 싼 Local Rule을 RTX 5090에서 병렬로 실행해, Matter들이 실제로 서로 영향을 주는 살아 있는 첫 세계가 성립한다는 것을 증명한다.**

## Current Gate State — 2026-08-17

현재 실제 세부 상태는 `STATUS.md`가 최종 기준이다.

이 문서 갱신 시점의 요약:

- G0-G7: PASS / CLOSED
- G8: IN_PROGRESS
  - G8-A Measurement Substrate: COMPLETE
  - G8-B Benchmark Scenario Suite: NEXT
  - G8-C Official Matrix Measurement: PENDING
- G9: PENDING
- M0 `ACHIEVED`: NO

---

## M0 Reference Configuration

```text
Platform: Windows
Language: Rust
Window/Input: winit
GPU API: wgpu
Backend: DX12
Primary GPU: RTX 5090
Reference World: 2048 × 2048 = 4,194,304 cells
Initial Chunk: 64 × 64
Chunk Count: 32 × 32 = 1024
Simulation Target: 60 TPS baseline
Rendering: independent
```

Chunk size와 세부 numeric threshold는 benchmark에 따라 조정 가능하다.

---

## M0 Content Baseline

### Matter

- Boundary Block
- Stone
- Sand
- Ice
- Water
- Steam
- Smoke
- Wood
- Oil

### Phenomenon / Field / State

- Temperature
- Combustion / Fire phenomenon
- Pressure

Fire는 단순 permanent orange Matter가 아니라 combustion/energy phenomenon으로 검증한다.

---

## G0 — Runtime

### Claim

Windows에서 GPU Production Simulation이 실제로 실행되며 Simulation과 Presentation이 분리되어 있다.

### Required Evidence

- Rust workspace/build 성공
- winit Windows app 실행
- wgpu DX12 path 확인
- GPU Simulation Core가 window rendering과 독립적으로 tick 가능
- `WorldConfig`로 world size 설정 가능
- reference world 2048×2048 초기화 가능
- headless/reference execution hook 존재

### Failure

- Simulation이 rendering code에 직접 묶여 headless 실행 불가
- world size가 여러 곳에 하드코딩
- CPU가 production world 전체를 authoritative하게 계산

---

## G1 — World Integrity

### Claim

Powdergame의 핵심 Cell identity가 깨지지 않는다.

### Required Evidence

- One Cell = Max One Matter invariant test
- EMPTY가 Material Registry Matter가 아님
- per-cell mixed amount 없음
- editable outer BLOCK
- outer BLOCK 제거 가능
- 열린 boundary 밖 Matter가 Void로 소멸
- invalid material id / out-of-bounds 없음

### User Validation

경계를 지웠을 때 Matter가 자연스럽게 밖으로 빠져나가고 보이지 않는 벽이 느껴지지 않는지 확인.

---

## G2 — Local Movement

### Claim

STATIC / POWDER / LIQUID / GAS가 장거리 scan 없이 local movement만으로 충분히 이해 가능한 움직임을 만든다.

### Required Evidence

- Stone/Block STATIC
- Sand POWDER
- Water/Oil LIQUID
- Steam/Smoke GAS
- behavior별 local stencil
- First-Match movement
- 한 Tick에 Liquid가 먼 Cell을 search/teleport하지 않음
- movement competition에서 state corruption 없음

### User Validation

- Sand가 Powder Game답게 떨어지는가
- Water가 여러 Tick의 local movement로 충분히 자연스럽게 퍼지는가
- Gas가 이동성은 높지만 무의미하게 영원히 떨지 않는가

---

## G3 — Density / Displacement

### Claim

실제 부력 solver 없이 작은 integer Density Rank + local displacement만으로 유용한 부력/침강/층분리가 만들어진다.

### Required Evidence

- Density가 Material property로 존재
- per-cell density 중복 저장 없음
- movable Matter끼리 rank 비교 가능
- Sand가 Water와의 local displacement에서 침강 가능
- 서로 다른 Liquid가 rank ordering에 따라 층분리 가능
- 서로 다른 Gas가 필요한 경우 density ordering을 가질 수 있음
- 같은 rank는 density swap을 만들지 않음
- STATIC은 일반 density swap 제외

### User Validation

> **“부력을 계산하지 않는다. 정렬한다.”** 방식이 플레이할 때 충분히 자연스럽고 재미있는지 확인.

---

## G4 — Thermal / Phase / Combustion

### Claim

Temperature가 공통 세계 법칙으로 동작하고 한 물질 전용 트릭이 아니다.

### Required Evidence

- Temperature f32 baseline
- 4-neighbor thermal propagation baseline
- EMPTY가 숨은 thermal medium이 아님
- Material별 cheap conductivity/heat-capacity 성격 표현 가능
- Ice ↔ Water ↔ Steam
- heating/cooling 양방향 transition
- Wood/Oil이 공통 combustion grammar에 참여
- combustion → Heat + Smoke + presentation event 가능
- Oxygen이 하드코딩된 필수 조건이 아님
- NaN/Infinity runaway 없음

### User Validation

정확한 현실 열역학이 아니라도 뜨거운 곳/차가운 곳의 인과가 상식적으로 이해되는가.

---

## G5 — Pressure Chain

### Claim

상변화와 공간 제약이 Pressure라는 다음 시스템으로 자연스럽게 연결된다.

### Required Evidence

최소 대표 chain:

```text
Water heated
→ Steam transition / expansion request
→ space insufficient
→ Pressure generated
→ local propagation
→ movable Matter influenced OR weak structure stressed
→ rupture threshold exceeded
→ opening created
→ venting
```

추가 요구:

- scalar pressure baseline
- 4-neighbor propagation baseline
- 별도 per-cell pressure velocity vector 필수 아님
- Pressure가 시간이 지나서 이유 없이 0으로 사라지는 구조가 아님
- opened boundary/container에서 해소 가능

### User Validation

“보일러 폭발” 전용 코드 없이 작은 Rule chain으로 실제로 터지는 현상이 납득되는가.

---

## G6 — Parallel Integrity

### Claim

일반 상호작용은 GPU 병렬화 친화적인 `Read Neighbors, Write Self`로 처리되며 무거운 global ordering을 요구하지 않는다.

### Required Evidence

- 일반 local Rule은 self Next만 write
- 다른 Cell 직접 write가 일반 authoring path가 아님
- movement/swap/spawn만 ownership Claim/Resolve 사용
- multiple source → one target에서 single winner
- stateless cheap arbitration candidate 구현
- per-cell RNG state 없음
- ordered first-match rule execution
- Rule 우선순위를 위해 full-world extra pass를 기본적으로 사용하지 않음

### Failure

- reaction 하나 추가할 때마다 atomic/global sync가 늘어남
- 모든 Rule을 모든 Cell이 scan
- race를 비결정성이라는 이유로 방치해 invariant가 깨짐

---

## G7 — Active / Sleep

### Claim

물질의 존재량이 아니라 실제 변화 가능한 영역이 simulation work를 결정하는 방향이 성립한다.

### Required Evidence

- Chunk activity state 존재
- 의미 있는 변화가 일정 기간 없으면 Sleep 가능
- 이웃 영향이 접근하면 Wake
- field/system별 active 상태 분리 가능하거나 최소한 구조적으로 측정 가능
- 천천히 타는 Wood는 실제 변화가 있으므로 Active 유지
- stable Water bulk가 존재만으로 영원히 movement active하지 않음
- stable Steam/Gas bulk가 존재만으로 영원히 active하지 않음
- same Matter ↔ same Matter 무의미한 swap 제거

### User Validation

대규모 안정 Liquid/Gas가 gameplay를 깨뜨리지 않고 실제 계산량을 줄이는가.

---

## G8 — Performance Evidence

### Claim

M0의 성능을 추측이 아니라 재현 가능한 숫자와 병목 결론으로 설명할 수 있다.

G8은 최대 TPS 경쟁이 아니다.

> **어떤 실제 gameplay workload가 얼마의 비용을 만들며, 다음 작업이 최적화인지 제품 구현인지 판단할 수 있어야 한다.**

### G8-A — Measurement Substrate

Required evidence:

- production pipeline과 동일한 pass ordering 측정
- ordinary production path에 profiling feature/readback 의존성 없음
- per-pass GPU timestamp 측정
- profiled / unprofiled state equivalence
- timed loop 밖 activity census
- application-tracked GPU memory accounting

### G8-B — Benchmark Scenario Suite

Required repeatable fixtures:

- Sand Fall
- Water Flow
- Fire / Heat
- Pressure Burst
- Heavy Mixed World

각 fixture는 동일 config에서 자동 staging 가능하고, scenario-specific benchmark code가 production physics를 변경하지 않아야 한다.

### G8-C — Official Matrix Measurement

Required metrics:

- production sustained simulation throughput
- simulation tick wall time
- GPU tick envelope
- Render FPS
- GPU rendering time
- simulation + rendering 동시 실행 상태
- Matter movement cost
- Temperature cost
- Pressure cost
- Reaction cost
- Claim/Resolve cost
- Active/Sleep management cost
- active Cell count
- active Chunk count
- tracked GPU memory
- commit SHA / Windows / GPU / driver / WorldConfig / chunk size / build mode

### Measurement Integrity

- timed loop 안에서 full-world readback 금지
- scenario마다 동일한 warm-up/trial 정책 사용
- single-tick latency와 sustained batch throughput을 혼동하지 않음
- debug HUD의 wall-clock TPS를 공식 GPU benchmark로 확대 해석하지 않음
- profiling overhead를 production cost로 보고하지 않음

### M0 Numeric Target

60 TPS는 reference product target이다.

그러나 M0는 임의의 최대 TPS 숫자 하나만으로 PASS/FAIL을 결정하지 않는다.

M0의 G8 Gate는 다음을 요구한다.

- representative baseline 확보
- subsystem cost 분리
- 병목 식별 가능
- 결과가 재현 가능
- 최적화가 지금 필요한지 결론 가능

### Optimization Decision Rule

다음 증거가 없으면 G7-C compaction / indirect dispatch 또는 공격적인 packing을 G9보다 먼저 구현하지 않는다.

- 대표 workload에서 60 TPS 또는 필요한 render responsiveness를 막는 병목
- 콘텐츠 추가 예산을 심각하게 제한하는 subsystem cost
- world scale 또는 user interaction 구현을 막는 구조적 비용

병목이 확인되면 한 번에 하나의 최적화 가설만 baseline과 비교한다.

---

## G9 — Playable First World / Product Validation

### Claim

기술적으로 움직이는 것을 넘어, 사용자가 직접 세계를 만들고 관찰하고 다시 실험하면서 Powdergame의 핵심 재미를 경험할 수 있다.

G9는 고정된 정답 fixture를 구경하는 마지막 승인 절차가 아니다.

> **M0의 실제 게임 vertical slice다.**

### G9-A — Sandbox Interaction

Required evidence:

- Matter 선택
- draw / erase
- brush size
- Heat 또는 Temperature 조작 도구
- pause / play / single-step / speed control / reset
- pan / zoom
- experiment preset load
- edit command가 GPU authoritative simulation에 안전하게 반영
- input/presentation code가 physics를 직접 우회하거나 임의 변경하지 않음

Failure:

- 사용자가 고정 demo를 재생/정지하는 것만 가능
- test staging 함수 없이는 세계를 만들 수 없음
- UI가 CPU-side 별도 simulation truth를 만듦

### G9-B — Open Emergence

Required evidence:

현재 M0 Matter와 공통 Rule만 사용해 사용자가 직접 만든 sandbox setup에서 다음 종류의 chain이 발생할 수 있다.

```text
Sand / Water / Oil movement and layering
Ice ↔ Water ↔ Steam
Wood / Oil combustion
sealed chamber → Pressure → rupture → vent
Heat / Smoke / Pressure → follow-up change
```

- 결과 전체를 scenario-specific script로 작성하지 않음
- `boiler_explosion()` 같은 전용 정답 기능 없이 공통 Rule이 결합
- 동일 Matter가 한 가지 정답 장면이 아니라 여러 실험에 사용 가능
- 예상 밖 결과가 invariant를 지키고 세계 규칙상 납득 가능하면 관찰 대상으로 보존

### G9-C — Discovery MVP

Required evidence:

실제 simulation truth에서 나온 semantic state/event를 사용해 첫 의미 있는 관찰을 기록한다.

최소 후보:

- Phase Change
- Combustion Started / Extinguished
- Pressure Generated
- Rupture / Vent
- Matter Transformation
- 의미 있는 resistance / no-reaction observation

Discovery policy:

- 현상 단위 기록
- 정확한 threshold / coefficient 비공개
- 남은 exact discovery count 비공개
- “아직 발견하지 못한 성질이 있다” 정도의 hint 허용
- reset/preset이 runtime truth와 다른 가짜 발견을 만들지 않음

> **사전은 정답표가 아니라 플레이어가 발견한 세계의 연구 노트다.**

### G9-D — Honest Presentation

Required evidence:

- Simulation Truth와 Presentation Effect가 구조적으로 분리
- combustion, Smoke/Temperature 또는 rupture/vent 중 핵심 현상을 raw diagnostic color보다 읽기 쉬운 modern feedback으로 표현
- read-only state 또는 semantic event에서 presentation input 추출
- 실제로 일어나지 않은 movement/reaction을 보여주지 않음
- cell simulation resolution이 final FX resolution을 강제하지 않음

최종 art stack 전체, 모든 sound, 완성된 post-processing은 M0 요구사항이 아니다.

그러나 현재 진단 HUD와 pixel palette만으로 제품 재미를 판정하지 않는다.

### G9-E — User Product Validation

Required user questions:

- 직접 만졌을 때 재미있는가?
- 단순한 local Rule들이 서로 연결되어 예상보다 큰 현상을 만드는가?
- 결과가 현실과 똑같지 않아도 세계 안에서 말이 되는가?
- “이걸 넣으면 무슨 일이 일어날까?”라는 다음 실험 욕구가 생기는가?
- 성능 최적화가 세계를 죽이는 편법으로 느껴지지 않는가?
- 고정 설명 없이도 원인과 결과를 어느 정도 읽을 수 있는가?

Strong success signals:

- 사용자가 지시 없이 두 번째 실험을 시작
- 같은 Matter를 다른 용도로 다시 사용
- 정확한 수치를 몰라도 결과의 원인을 설명
- 예상하지 못했지만 납득 가능한 결과를 발견
- 관찰 후 새로운 조건이나 Matter를 자발적으로 추가

### Final Approval

**사용자가 실제 sandbox를 직접 플레이하고 승인해야 M0 = ACHIEVED.**

자동 테스트, benchmark, AI review와 고정 observatory 승인은 G9-E의 사용자 제품 승인을 대체하지 않는다.

---

## M0 Closure Order

```text
G8-B Benchmark Fixtures
→ G8-C Official Matrix and Bottleneck Decision
→ G9-A Sandbox Interaction
→ G9-B Open Emergence
→ G9-C Discovery MVP
→ G9-D Honest Presentation
→ G9-E User Approval
→ M0 ACHIEVED
```

G8에서 명확한 blocker가 나오지 않는 한 G9 전에 별도 최적화 Phase를 삽입하지 않는다.

---

## M0에서 명시적으로 하지 않는 것

다음은 M0 통과 조건이 아니다.

- Electricity production system
- Radiation
- Gameplay Light physics
- Life / Agent
- Civilization / Concept / Meta
- full Rule DSL editor
- Interaction Lab 완성
- exact deterministic replay
- true Navier-Stokes fluid
- strict global energy conservation
- broad GPU compatibility
- Browser/macOS product version
- 수십 개 Material의 일괄 구현
- atmosphere composition / universal mixture solver
- 완성된 production art/audio stack 전체
- 숫자로 미리 정한 무리한 최대 performance target

M0의 목적은 이 미래 기능을 많이 구현하는 것이 아니다.

> **이미 증명한 빠른 GPU 세계를 플레이어가 실제로 자기 세계로 사용할 수 있게 만들고, 다음 실험 욕구가 생기는지 확인하는 것.**
