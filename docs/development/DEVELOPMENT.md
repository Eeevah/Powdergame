# Powdergame Development

이 문서는 현재 Foundation Design을 실제 구현으로 옮기는 개발 원칙을 정의한다.

---

## 1. Current Stage

현재 단계는 **M0 — First World 구현 직전**이다.

문서/설계는 충분히 구체화되었으며 다음 단계는 실제 GPU baseline을 만드는 것이다.

더 많은 미래 물리/콘텐츠를 설계만 하며 범위를 늘리지 않는다.

---

## 2. Development Priority

개발 우선순위:

1. correctness/invariant가 명확한 단순 baseline
2. 실제 GPU 측정
3. gameplay observation
4. bottleneck-specific optimization
5. 그 다음 content/system 확장

처음부터 모든 최적화 후보를 동시에 넣지 않는다.

---

## 3. Current Technical Direction

```text
Windows
Rust
winit
wgpu
DX12
RTX 5090 primary target
```

Simulation Core는 Presentation/Platform과 분리한다.

개념적 repository 방향:

```text
engine/
apps/windows/
tests/
benches/
examples/
tools/
assets/
docs/
```

정확한 crate 이름은 구현하면서 정할 수 있지만 architectural boundary를 깨지 않는다.

---

## 4. M0 Implementation Sequence

### Step 1 — Workspace / Runtime skeleton

- Rust workspace
- Windows app
- winit loop
- wgpu DX12 device
- render/simulation timing 분리
- `WorldConfig`
- headless simulation entry point

### Step 2 — Dense GPU world baseline

M0 baseline:

```text
material_id[]
temperature[]
pressure[]
minimal_flags[]
```

- Current/Next world state
- finite boundary
- outer BLOCK
- EMPTY
- no premature packing

### Step 3 — Stone + Sand

먼저 다음을 증명한다.

- STATIC
- POWDER
- local stencil
- First-Match
- ownership proposal/resolve/commit
- One Cell = One Matter
- cheap arbitration baseline

### Step 4 — Water + Oil

- LIQUID movement
- Density Rank
- local displacement
- layer formation
- stable bulk behavior

여기서 “부력을 계산하지 않는다. 정렬한다.”가 실제 화면에서 충분히 자연스러운지 본다.

### Step 5 — Steam + Smoke

- GAS movement family
- local high-mobility behavior
- Gas가 반드시 매 Tick 흔들리지 않음
- stable bulk sleep 가능성

### Step 6 — Temperature / Phase

- f32 temperature baseline
- 4-neighbor propagation
- cheap thermal material properties
- Ice ↔ Water ↔ Steam
- EMPTY가 thermal medium 아님

### Step 7 — Combustion

- Wood
- Oil
- common ignition/combustion grammar
- Heat
- Smoke
- flame presentation event

### Step 8 — Pressure

- phase expansion request
- space shortage
- pressure generation
- local pressure propagation
- push/resistance
- rupture
- vent

### Step 9 — Active / Sleep

- chunk activity
- short stable period → sleep
- neighbor influence → wake
- stable Water/Gas bulk test
- slowly burning Wood stays active

### Step 10 — Benchmark baseline

- Sand Fall
- Water Flow
- Fire/Heat
- Pressure Burst
- Heavy Mixed World

subsystem cost를 분리해서 기록.

---

## 5. Baseline before Fast Path

처음 구현은 읽을 수 있어야 한다.

```text
f32
simple descriptor
clear passes
simple current/next
minimal synchronization
```

이후 성능 후보는 한 번에 하나씩 비교한다.

예:

1. Active Chunk
2. Field-specific activity
3. stable frontier reduction
4. active compaction/indirect dispatch
5. shared memory tile
6. descriptor packing
7. f16 experiment

한꺼번에 여러 변경을 넣어 원인을 모르게 하지 않는다.

---

## 6. Content Development

Material/Rule은 data-driven 방향으로 간다.

초기에는:

```text
Material Registry
+ Engine-defined physics primitives
+ Material-owned interaction rules
```

Rule DSL은 미래 확장 경로다. M0에서 editor/DSL 전체를 만들지 않는다.

Material 추가 시:

- identity
- movement class
- 필요한 최소 property
- transition
- interaction rules

정도만 정의하고 일반 움직임/열/압력은 공통 system을 사용한다.

---

## 7. AI-assisted Authoring

AI는 개발 단계에서 다음을 도울 수 있다.

- Material idea 정리
- property/rule 파일 작성
- 테스트 fixture 제안
- existing rule conflict review
- discovery description 작성

하지만 runtime 게임이 LLM에게 반응을 묻는 구조는 아니다.

Runtime은 이미 정의된 cheap rule만 실행한다.

---

## 8. Deferred Interaction Lab

Interaction Lab은 미래 Developer Tool이다.

완성된 Material/Rule을 actual GPU Simulation에 넣고 기존 세계와 대표 환경에서 자동 실험하여 예상 밖 interaction/regression을 찾는 도구다.

현재 구현하지 않는다.

Simulation Core를 Lab 때문에 복잡하게 만들지 말고 다음 hook 정도만 자연스럽게 유지한다.

- initial world/state injection
- headless GPU run
- tick control
- event/state observation

---

## 9. Decision Change Policy

현재 문서는 구현 전 Foundation Q&A를 기준으로 적극 수정되었다.

앞으로 구현 후 중요한 결정이 바뀌면:

- SPEC current contract 갱신
- ADR 새 결정 또는 supersede 기록
- Design History context 남김
- MILESTONE/STATUS 영향 갱신

과거 결정 이유를 조용히 삭제하지 않는다.

---

## 10. Performance Change Rule

최적화는 다음 질문에 답할 수 있어야 한다.

1. 어떤 benchmark에서 병목인가?
2. baseline cost는 얼마인가?
3. 변경 후 cost는 얼마인가?
4. gameplay/invariant가 유지되는가?
5. code/management complexity 증가가 가치 있는가?

`빠를 것 같다`만으로 production architecture를 복잡하게 만들지 않는다.

---

## 11. Definition of Done

개별 task의 `done`과 Milestone `ACHIEVED`를 구분한다.

Task가 구현/테스트되어도 M0 전체는 `MILESTONES.md`의 G0~G9를 통과해야 한다.

최종 Milestone 완료는 사용자가 실제 결과를 보고 승인해야 한다.
