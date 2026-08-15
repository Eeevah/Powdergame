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
**Status:** PLANNED  
**Purpose:** 첫 GPU 세계가 Powdergame의 핵심 철학을 실제로 증명하는가.

M0의 목적은 콘텐츠를 많이 넣는 것이 아니다.

> **수백만 개의 아주 싼 Local Rule을 RTX 5090에서 병렬로 실행해, Matter들이 실제로 서로 영향을 주는 살아 있는 첫 세계가 성립한다는 것을 증명한다.**

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

M0의 성능을 추측이 아니라 숫자로 설명할 수 있다.

### Required Benchmark Scenarios

- Sand Fall
- Water Flow
- Fire / Heat
- Pressure Burst
- Heavy Mixed World

### Required Metrics

- Render FPS
- simulation tick time
- GPU simulation time
- GPU rendering time
- Matter movement cost
- Temperature cost
- Pressure cost
- Reaction cost
- Claim/Resolve cost
- Active/Sleep management cost
- active Cell count
- active Chunk count
- VRAM usage

### M0 Numeric Target

아직 임의의 numeric pass/fail 기준을 두지 않는다.

M0는 **baseline을 만들고 병목을 볼 수 있는 것**이 gate다.

M1 이후 실제 RTX 5090 결과를 기반으로 performance budget을 설정한다.

---

## G9 — Product Validation

### Claim

기술적으로 움직이는 것뿐 아니라 Powdergame의 핵심 재미가 실제로 보인다.

대표 M0 experiment:

```text
Heat
→ Water
→ Steam
→ expansion
→ Pressure
→ rupture
→ vent
→ nearby Heat/Smoke/Movement
→ follow-up reaction
```

### Required User Questions

- 직접 만졌을 때 재미있는가?
- 단순한 local Rule들이 서로 연결되어 예상보다 큰 현상을 만드는가?
- 결과가 현실과 똑같지 않아도 세계 안에서 말이 되는가?
- “이걸 넣으면 무슨 일이 일어날까?”라는 다음 실험 욕구가 생기는가?
- 성능 최적화가 세계를 죽이는 편법으로 느껴지지 않는가?

### Final Approval

**사용자가 직접 플레이하고 승인해야 M0 = ACHIEVED.**

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
- 숫자로 미리 정한 무리한 performance target

M0의 목적은 이 미래 기능을 많이 구현하는 것이 아니라 **이들을 나중에 같은 저비용 local physics 철학으로 확장할 수 있는 첫 세계를 증명하는 것**이다.
