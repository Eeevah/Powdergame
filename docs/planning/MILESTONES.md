# Powdergame Milestones

이 문서는 기능 체크리스트가 아니라 **Evidence Gate 계약**이다.

- 현재 실제 상태·SHA·Run ID·다음 행동: [`STATUS.md`](STATUS.md)
- 장기 방향과 순서: [`ROADMAP.md`](ROADMAP.md)
- 실행 증거와 사용자 판정: [`../evidence/`](../evidence/)

이 문서는 Gate의 의미와 완료 조건만 유지한다. 변화가 잦은 현재 상태를 복제하지 않는다.

---

## 1. 상태와 승인

공식 Milestone 상태:

- `PLANNED`
- `IN_PROGRESS`
- `BLOCKED`
- `VALIDATION`
- `ACHIEVED`
- `REGRESSION`

AI, Codex, CI와 benchmark는 `VALIDATION`까지 올릴 수 있다.

> **최종 `ACHIEVED`는 사용자 승인 없이는 선언하지 않는다.**

과거 완료 기록은 삭제하지 않는다. 이후 계약을 깨뜨리면 `REGRESSION`으로 이력을 남긴다.

공통 validation 차원:

- Functional Correctness
- Behavioral Stability / Reproducibility
- Performance
- UX / Product Validation
- Persistence / Compatibility
- Failure / Recovery
- Regression

Bit-perfect determinism은 기본 correctness 기준이 아니다. `DETERMINISM_SPEC.md`를 따른다.

---

# M0 — First World

**Type:** Delivery

## 목적

> **수백만 개의 매우 싼 Local Rule을 GPU에서 병렬 실행해, 작은 규칙들이 서로 영향을 주는 살아 있는 첫 세계를 만들고 사용자가 직접 실험하고 싶어지는지 증명한다.**

M0는 콘텐츠 수 경쟁이 아니다. 현재 Matter와 공통 Rule로 기술적 세계와 첫 제품 경험을 함께 증명한다.

## Reference Configuration

```text
Platform: Windows
Language: Rust
Window/Input: winit
GPU API: wgpu / DX12
Primary GPU: RTX 5090
Reference World: 2048 × 2048
Initial Chunk: 64 × 64
Simulation Target: 60 TPS
Rendering: simulation과 독립
```

Chunk size와 numeric threshold는 evidence에 따라 조정할 수 있다. 변경 시 관련 benchmark와 acceptance contract를 갱신한다.

## Content Baseline

Matter:

- Boundary Block
- Stone
- Sand
- Ice
- Water
- Steam
- Smoke
- Wood
- Oil

공통 현상·Field:

- Temperature
- Phase change
- Combustion / Fire phenomenon
- Pressure
- rupture / vent
- Active / Sleep

Fire는 영구 orange Matter가 아니라 연소 중인 Matter, Heat, Smoke와 semantic event의 결합으로 다룬다.

---

## G0 — Runtime

### Claim

Windows에서 GPU Production Simulation이 실제로 실행되고 Simulation과 Presentation이 분리되어 있다.

### Required evidence

- Rust workspace/build 성공
- winit Windows app 실행
- wgpu DX12 path 확인
- rendering 없이 Simulation tick 가능
- `WorldConfig`로 world size 설정
- 2048×2048 reference world 초기화
- bounded/headless execution hook

### Failure

- production world를 CPU가 authoritative하게 계산
- simulation이 window/render lifecycle에 직접 종속
- world size가 여러 위치에 하드코딩

---

## G1 — World Integrity

### Claim

Powdergame의 Cell identity와 경계 계약이 깨지지 않는다.

### Required evidence

- One Cell = Max One Matter
- EMPTY는 Registry Matter가 아님
- per-cell mixed amount 기본 모델 없음
- editable outer Boundary Block
- outer Block 제거 가능
- 열린 경계 밖 Matter는 Void로 소멸
- invalid Material ID와 out-of-bounds write 없음
- EMPTY Temperature/Pressure/flags 위생

### User validation

경계를 열었을 때 Matter가 자연스럽게 빠져나가고 보이지 않는 벽이 느껴지지 않는지 확인한다.

---

## G2 — Local Movement

### Claim

STATIC / POWDER / LIQUID / GAS가 장거리 scan이나 teleport 없이 local movement만으로 이해 가능한 움직임을 만든다.

### Required evidence

- Stone/Boundary STATIC
- Sand POWDER
- Water/Oil LIQUID
- Steam/Smoke GAS
- behavior별 bounded local stencil
- ordered First-Match movement
- 한 tick에 한 Cell이 먼 빈칸을 탐색하지 않음
- ownership collision에서 single winner와 state integrity
- chunk boundary와 interior가 같은 local 계약 사용

### User validation

Sand, Liquid, Gas가 직관적이며 무의미하게 영원히 떨지 않는지 확인한다.

---

## G3 — Density / Displacement

### Claim

연속 부력 solver 없이 작은 integer Density Rank와 local displacement로 침강·부상·층분리가 성립한다.

### Required evidence

- Density Rank는 Material property
- per-cell density 중복 저장 없음
- movable Matter끼리 local rank 비교
- Sand가 Water에서 침강 가능
- Water/Oil 등 서로 다른 Liquid 층분리
- 필요한 Gas ordering
- equal rank는 swap하지 않음
- STATIC은 일반 density swap 제외
- identity/count 보존

### User validation

> **부력을 계산하지 않는다. 정렬한다.**

이 표현이 실제 플레이에서 충분히 자연스럽고 재미있는지 확인한다.

---

## G4 — Thermal / Phase / Combustion

### Claim

Temperature와 Reaction이 공통 세계 법칙으로 동작하고 한 fixture 전용 트릭이 아니다.

### Required evidence

- f32 Temperature baseline
- 4-neighbor finite propagation
- EMPTY는 hidden thermal medium이 아님
- Material별 cheap conductivity/heat-capacity 성격
- Ice ↔ Water ↔ Steam 양방향 transition과 hysteresis
- Wood/Oil이 공통 combustion grammar에 참여
- finite fuel consumption
- combustion → Heat + Smoke + semantic event
- finite Smoke lifetime
- Temperature/flags가 Matter identity와 함께 이동
- NaN/Infinity runaway 없음

### User validation

정확한 현실 열역학이 아니더라도 뜨거운 곳과 차가운 곳의 원인·결과를 읽을 수 있는지 확인한다.

---

## G5 — Pressure Chain

### Claim

상변화와 공간 제약이 Pressure, 구조 파열, opening과 venting으로 이어진다.

### Required representative chain

```text
Water heated
→ Steam transition / expansion request
→ insufficient space
→ Pressure generated
→ local propagation
→ weak structure stressed
→ rupture threshold exceeded
→ opening created
→ venting / relief
```

추가 요구:

- scalar Pressure baseline
- bounded local propagation
- Liquid/Gas pressure medium 계약
- 이유 없는 arbitrary decay 금지
- opening/container 경계 변화에 반응
- rupture는 공통 Material threshold 사용
- fixture combustion 등 다른 subsystem이 opening 원인을 대신하지 않음
- causal telemetry로 named chain 검증

### User validation

전용 `boiler_explosion()` 없이 작은 Rule chain으로 압력 사고가 납득되는지 확인한다.

---

## G6 — Parallel Integrity

### Claim

일반 상호작용은 `Read Neighbors, Write Self`와 제한된 Claim/Resolve로 GPU 병렬화하며 무거운 global ordering을 요구하지 않는다.

### Required evidence

- 일반 Rule은 자기 Next state만 write
- 다른 Cell 직접 mutation이 일반 authoring path가 아님
- movement/swap/spawn만 ownership Claim/Resolve
- multiple sources → one target에서 single winner
- loser state 보존과 identity/count integrity
- stateless cheap arbitration
- per-cell RNG state 없음
- ordered First-Match rule execution
- ordinary interaction마다 full-world sort/priority resolve를 추가하지 않음
- scratch reuse 경계에서 stale data 없음

### Failure

- 새 Reaction마다 atomic/global synchronization 증가
- 모든 Rule을 모든 Cell이 scan
- race를 approximate determinism으로 포장해 invariant를 깨뜨림

---

## G7 — Active / Sleep

### Claim

Matter 존재량이 아니라 실제 changeable frontier가 simulation work를 결정한다.

### Required evidence

- Chunk activity state
- Matter / Thermal / Pressure / Reaction activity 분리
- 의미 있는 변화가 일정 기간 없으면 Sleep
- 이웃 frontier, edit, phase/reaction 영향이 접근하면 Wake
- stable Water bulk가 존재만으로 movement active하지 않음
- stable Steam/Gas bulk가 존재만으로 active하지 않음
- slowly burning Wood는 실제 변화 때문에 active 유지
- same-Matter no-op swap 제거
- sleeping pass guards가 exact state를 보존
- reset/edit와 wake snapshot race 없음

### User validation

대규모 stable bulk가 gameplay를 깨뜨리지 않으면서 실제 계산량을 줄이는지 확인한다.

### Deferred optimization boundary

Active-list compaction, indirect dispatch와 aggressive sparse dispatch는 correctness Gate가 아니다. G8 또는 이후 product workload가 명확한 blocker를 증명할 때만 별도 가설로 측정한다.

---

## G8 — Performance Evidence

### Claim

M0의 비용을 재현 가능한 실제 workload 숫자로 설명하고 다음 행동이 최적화인지 제품 구현인지 결정할 수 있다.

G8은 최대 TPS 경쟁이 아니다.

### Evidence roles

- **G8-A:** 신뢰 가능한 측정 substrate와 exact-source capture/verifier
- **G8-B:** 대표 workload와 사용자 acceptance
- **G8-C:** 동일 조건의 official performance matrix와 bottleneck decision

Automatic measurement verdict는 해당 실행의 claim이지 사용자 승인·제품 준비 상태·다른 Gate closure가 아니다.

### G8-A — Measurement Substrate

Required evidence:

- production과 동일한 pass ordering
- ordinary production path에 timestamp/readback 강제 없음
- 별도 profiled context
- per-pass raw GPU timestamp
- profiled/unprofiled state equivalence
- timed loop 밖 full raw activity census
- aggregate 독립 재집계
- application-tracked persistent GPU memory
- exact source/input, binary, argv, logs, exit, artifact hash binding
- no-overwrite staged publication과 Receipt-last
- capture와 독립된 verifier
- synchronized diagnostic와 sustained production throughput 구분

User visual disposition은 technical capture/verification과 별도 기록한다.

### G8-B — Benchmark Scenario Suite

Official fixtures:

- Sand Fall
- Water Flow
- Fire / Heat
- Pressure Burst
- Heavy Mixed World

Required evidence:

- deterministic shared staging API
- production `Simulation` reset/stage
- Windowed Gallery에서 paused inspection
- Headless benchmark에서 같은 fixture 사용
- scenario-specific analyzer가 production physics를 변경하지 않음
- movement, conservation/allowed transitions, integrity, lifecycle와 exact reset
- automatic verdict와 user disposition 분리
- rejected/superseded run immutable 보존
- Cell Inspector 등 사용자 comprehension surface
- 다섯 scenario 각각의 explicit user decision

G8-B closure는 G8-C performance result가 아니다.

### G8-C — Official Performance Matrix

Required scenarios: G8-B의 다섯 official workload를 정확히 한 번씩 동일 config로 측정한다.

Required modes:

- Mode A — ordinary-context sustained throughput
- Mode B — separate profiled-context GPU pass/group breakdown
- Mode C — windowed production simulation+render coexistence
- Mode D — separate render GPU timestamp profile

Required metrics:

- sustained TPS / wall time per tick
- GPU tick envelope, pass sum, residual
- Matter / Claim-Resolve / Thermal / Reaction / Pressure / Active-Sleep cost
- active Cell and Chunk census
- tracked persistent GPU bytes
- Render FPS and frame P50/P95/P99
- simulation target/deadline/catch-up/drop accounting
- GPU render P50/P95/mean
- source, binaries, OS/GPU/driver/config/build identity

Measurement integrity:

- timed loop 안 full-world readback 없음
- every window/trial before pristine reset/stage
- common warmup/trial policy
- diagnostic latency와 sustained throughput 구분
- HUD TPS를 official benchmark로 확대 해석 금지
- profiling overhead를 product FPS에 섞지 않음
- canonical window size/format/present mode를 실제 live state로 검증
- historical producer vocabulary와 internal model을 explicit adapter로 분리
- matrix-level Receipt-last/package 하나
- raw에서 전체 matrix를 재구성하는 independent verifier

Decision outputs:

- `PROCEED_TO_G9`
- `OPTIMIZATION_REVIEW_REQUIRED`
- `NEEDS_HUMAN_REVIEW`

Recommendation은 사용자 승인이나 자동 구현 권한이 아니다.

### Numeric target and optimization rule

60 TPS는 M0 reference product target이다. 단일 최대 TPS 숫자만으로 Gate를 닫지 않는다.

G9보다 먼저 최적화를 시작하려면 최소 하나를 official evidence가 증명해야 한다.

- 대표 workload가 60 TPS 또는 product responsiveness를 방해
- 특정 subsystem이 콘텐츠 예산을 심각하게 제한
- memory/world scale이 interaction 구현을 막음
- repeated measurement에서 구조적 bottleneck이 재현

그 경우에도 한 번에 하나의 최적화 가설만 baseline과 비교한다.

---

## G9 — Playable First World / Product Validation

### Claim

사용자가 직접 세계를 만들고, 관찰하고, 수정하고, 다음 실험을 시작하면서 Powdergame의 핵심 재미를 경험한다.

G9는 고정 fixture를 구경하는 마지막 승인 절차가 아니다.

> **M0의 실제 game vertical slice다.**

### G9-A — Sandbox Interaction

Current status: **USER ACCEPTED WITH KNOWN FOLLOW-UP**. Inspector continuity v2 at source `a00e39b2e00bfbd9ac28214c44cd22cc97542bb4` is **USER ACCEPTED**. It holds one honestly labelled previous sample across rapid hover movement, then keeps a fixed Sampling panel until a fresh current-Cell sample atomically replaces it. Canonical no-argument launch opens the Sandbox; explicit Gallery remains available. The known follow-up is the separately gated Thermal Environment/phase program; this acceptance does not authorize G9-B.

Required evidence:

- Matter 선택
- draw / erase
- brush size
- Heat 또는 Temperature 조작
- pause / play / single-step / speed / reset
- pan / zoom
- experiment preset load
- edit command가 GPU-authoritative simulation에 안전하게 반영
- UI가 CPU-side 별도 simulation truth를 만들지 않음
- fixed demo 없이 사용자가 원인을 직접 만들 수 있음

### G9-B — Open Emergence

Current status: **NOT STARTED**.

Entry prerequisite: [`Thermal Transport & Ignition Causality`](THERMAL_TRANSPORT_IGNITION_CAUSALITY.md) has completed TE-0R/TE-0/TE-0A/TE-0B and TE-1 Environment state/occupancy hygiene with Critical/High blocker zero. TE-2 is **USER ACCEPTED WITH KNOWN FOLLOW-UP** at candidate source `0977281...`; the production-physics source remains `fb7e568...`. TE-3D is **ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS**, while TE-3 runtime is **NOT STARTED** and the TE-5 pressure-volume bridge is **DESIGN REQUIRED / NOT STARTED**. G9-B emergence validation does not begin until the TE-3 phase path plus separately authorized TE-5 pressure-volume replacement are atomically activated with their named user evidence.

현재 M0 Matter와 공통 Rule만으로 사용자가 만든 sandbox setup에서 다음 chain이 가능해야 한다.

```text
Sand / Water / Oil movement and layering
Ice ↔ Water ↔ Steam
Wood / Oil combustion
sealed chamber → Pressure → rupture → vent
Heat / Smoke / Pressure → follow-up experiment
```

Required evidence:

- 결과를 scenario-specific script로 작성하지 않음
- 동일 Matter가 여러 실험에 사용 가능
- 예상 밖 결과가 invariant를 지키면 관찰 대상으로 보존
- user edit와 production physics의 causal chain을 읽을 수 있음

### G9-C — Discovery MVP

Current status: **NOT STARTED**.

실제 simulation truth와 semantic event에서 첫 의미 있는 관찰을 기록한다.

최소 후보:

- Phase Change
- Combustion Started / Extinguished
- Pressure Generated
- Rupture / Vent
- Matter Transformation
- 의미 있는 resistance / no-reaction

Policy:

- 현상 단위 기록
- exact threshold/coefficient 비공개
- 남은 exact discovery count 비공개
- 가짜 unlock event 금지
- 정답표가 아니라 플레이어 연구 노트

### G9-D — Honest Presentation

Current status: **NOT STARTED**.

Required evidence:

- Simulation Truth와 Presentation Effect 구조적 분리
- 핵심 combustion, Smoke/Heat, rupture/vent가 raw diagnostic color보다 읽기 쉬움
- read-only state 또는 semantic event에서 presentation input 추출
- 실제로 일어나지 않은 movement/reaction을 표현하지 않음
- simulation resolution이 FX resolution을 강제하지 않음

완성된 art/audio stack 전체는 M0 요구사항이 아니다. 그러나 진단 HUD만으로 제품 재미를 판정하지 않는다.

### G9-E — User Product Validation

Current status: **NOT STARTED**.

사용자 확인 질문:

- 직접 만졌을 때 재미있는가
- 작은 Rule이 예상보다 큰 현상으로 연결되는가
- 현실과 같지 않아도 세계 안에서 납득되는가
- 다음 실험 욕구가 생기는가
- 정확한 수치 없이 원인·결과를 읽을 수 있는가

Strong success signals:

- 지시 없이 두 번째 실험을 시작
- 같은 Matter를 다른 용도로 재사용
- 예상 못했지만 납득 가능한 결과를 발견
- 관찰 후 조건이나 Matter를 자발적으로 추가

> **사용자가 실제 sandbox를 직접 플레이하고 승인해야 M0 = ACHIEVED.**

---

## M0 Closure Order

```text
G8-A Measurement Integrity
→ G8-B Representative Workloads + User Acceptance
→ G8-C Official Matrix + Bottleneck Decision
→ G9-A Sandbox Interaction
→ G9-B Open Emergence
→ G9-C Discovery MVP
→ G9-D Honest Presentation
→ G9-E User Product Approval
→ M0 ACHIEVED
```

G8에서 명확한 blocker가 나오지 않는 한 G9 전에 별도 최적화 Phase를 삽입하지 않는다.

---

## M0에서 하지 않는 것

- Electricity / Radiation / Life / Civilization production systems
- full Rule DSL editor
- Interaction Lab 완성
- exact deterministic replay
- true Navier-Stokes fluid
- strict global energy conservation
- broad GPU/platform compatibility
- Browser/macOS product version
- 수십 개 Material 일괄 구현
- universal mixture/atmosphere solver
- 완성된 production art/audio stack 전체
- evidence 없이 정한 공격적 최대-performance target

M0의 목적은 미래 기능을 많이 구현하는 것이 아니다.

> **빠른 GPU 세계를 플레이어가 실제로 자기 세계로 사용하고 다음 실험을 자발적으로 시작하는지 증명한다.**
