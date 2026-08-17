# Powdergame Roadmap

이 문서는 Powdergame의 **장기 제품 방향과 작업 순서**를 기록한다.

- 약속된 일정표가 아니다.
- 현재 실제 상태는 `STATUS.md`를 따른다.
- 완료 판정은 `MILESTONES.md`의 Evidence Gate를 따른다.
- 제품 의도와 충돌하면 `vision/USER_VISION.md`가 최우선이다.
- 연구 문서는 후보의 원천이며, 별도 채택 없이 자동으로 Roadmap이나 구현 계약이 되지 않는다.

---

## 1. Product North Star

> **플레이어에게 물질을 주는 게임이 아니라, 우주를 발명할 수 있는 문법을 주는 게임.**

Powdergame의 목표는 기술적으로 정교한 GPU 시뮬레이션 자체가 아니다.

플레이어가 Matter를 놓고, 가열하고, 가두고, 식히고, 태우고, 구조를 만들었을 때 작은 세계 법칙들이 서로 연결되어 개발자가 직접 작성하지 않은 결과를 만들어야 한다.

최상위 제품 질문은 다음이다.

> **“이 세계에 이것을 넣으면 대체 무슨 일이 일어날까?”라는 생각을 계속 하게 만드는가?**

Roadmap의 모든 단계는 이 질문에 더 강한 답을 만들기 위해 존재한다.

---

## 2. Roadmap Decision Order

새 기능이나 최적화의 우선순위는 다음 순서로 판단한다.

1. 플레이어에게 새로운 실험 욕구를 만드는가.
2. 원인과 결과가 세계 안에서 이해 가능하고 재사용 가능한가.
3. 기존 Matter/Field와 새로운 연쇄작용을 만드는가.
4. 작은 Local Rule과 낮은 Cell cost로 구현 가능한가.
5. 실제 RTX 5090 측정에서 필요한가.
6. 사용자가 직접 플레이하고 가치가 있다고 승인했는가.

성능은 제품 비전의 핵심이지만 목적 그 자체가 아니다.

> **절약한 계산 예산은 더 큰 세계, 더 많은 동시 반응, 더 풍부한 실험과 더 좋은 Presentation에 다시 투자한다.**

---

## 3. Current Position — 2026-08-17

현재 Milestone은 **M0 — First World (`IN_PROGRESS`)**다.

현재 증거 상태:

- G0 Runtime — PASS / CLOSED
- G1 World Integrity — PASS / CLOSED
- G2 Local Movement — PASS / CLOSED
- G3 Density / Displacement — PASS / CLOSED
- G4 Thermal / Phase / Combustion — PASS / CLOSED
- G5 Pressure Chain — PASS / CLOSED
- G6 Parallel Integrity — PASS / CLOSED
- G7 Active / Sleep — PASS / CLOSED
- G8 Performance Evidence — IN_PROGRESS
  - G8-A Measurement Substrate — V5 OFFICIAL CAPTURE + INDEPENDENT VERIFICATION COMPLETE / VERIFIED EVIDENCE CANDIDATE; USER VISUAL VALIDATION PENDING
  - G8-B Benchmark Scenario Suite — IMPLEMENTATION CANDIDATE; Sand USER ACCEPTED; Water Harness candidate implemented but runs/user acceptance PENDING; overall NOT CLOSED
  - G8-C Official Matrix Measurement — PENDING
- G9 Product Validation — PENDING

G8-A v5는 clean source `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`에서 2048×2048 reference world와 실제 production pass를 측정했고, official capture와 독립 검증을 완료했다. 현재 상태는 verified evidence candidate이며 같은 SHA의 user visual validation은 pending이다. 기존 v4 원자료는 later source/binary 실행 연결과 raw census가 없으므로 historical data로만 보존한다.

이 결과로 “GPU 세계가 성립하는가”라는 초기 위험은 크게 낮아졌다. 이제 가장 큰 위험은 다음이다.

> **이 엔진이 실제로 플레이어가 만지고, 발견하고, 다시 실험하고 싶어지는 게임이 되는가.**

따라서 G8 이후 기본 경로는 추가 엔진 최적화가 아니라 **Playable First World**다.

---

## 4. Current Repository State — Canonical Recovery Integrated Locally

Canonical Recovery는 기능 Gate가 아니라 분기된 구현·증거선과 연구·문서선을 다시 하나의 검증 가능한 후보선으로 묶는 운영 작업이다.

- verified runtime/evidence parent: `fix/g8a-evidence-remediation-v5` at `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`
- research/Foundation parent: `feature/foundation-material-wiki` at `ccd0d7b00fb99128e8750ef09e5c4cce068bce09`
- local recovery merge: `e5871bdc53093700c44562826860c4d482f31ba5` on `integration/canonical-recovery`
- recovery 결과의 Cargo, apps, engine, WGSL, test tree는 `9abec9e`와 동일하다. Merge commit의 `docs/research` tree는 `ccd0d7b`와 동일하며, 후속 reconciliation은 P1 frontmatter의 비정식 movement 값 두 개만 canonical enum으로 정규화했다.
- PRE/POST full workspace, clippy, evidence-script self-test, verifier fixture, Windows release smoke, document/link 검증을 통과했다.
- 이 branch는 local-only다. recovery branch push, recovery PR 생성, `main` 승격은 수행하지 않았다.
- Draft PR #1은 open/draft 상태로 보존하며, 그 product-first 문서는 더 최신 evidence correction과 함께 `9abec9e`에 이미 포함되어 있으므로 merge/cherry-pick하지 않는다.
- personal-infra-wiki의 위치·운영 계약은 바뀌지 않았으므로 별도 프로젝트 페이지를 임의로 만들지 않았다.

따라서 local canonical candidate는 만들어졌지만 shared canonical `main`은 아직 갱신되지 않았다. 게시와 다음 Gate 선택은 각각 별도 사용자 결정이다. 기존 dirty worktree의 사용자 변경은 reset, stash, discard 또는 overwrite하지 않는다.

---

# M0 — First World Completion

M0는 엔진의 모든 미래 가능성을 구현하는 단계가 아니다.

> **현재 9개 Matter와 Temperature, Combustion, Pressure만으로도 플레이어가 자유롭게 실험하고 작은 Rule의 연쇄작용을 체감할 수 있는 첫 세계를 증명한다.**

---

## G8-B / G8-C — Performance Matrix Closure

### Goal

M0 성능을 하나의 calibration fixture가 아니라 대표 gameplay workload로 설명한다.

### Required Scenarios

- Sand Fall
- Water Flow
- Fire / Heat
- Pressure Burst
- Heavy Mixed World

### Required Evidence

- production sustained simulation throughput
- single-tick GPU timing envelope
- Render FPS
- GPU rendering time
- simulation + rendering 동시 실행 상태
- Matter / Thermal / Pressure / Reaction / Claim-Resolve / Active-Sleep subsystem cost
- active Cell count
- active Chunk count
- tracked GPU memory
- commit / hardware / driver / config / scenario 기록

60 TPS는 reference product target이지만, M0에서 임의의 최대 TPS 숫자 하나만으로 PASS/FAIL을 결정하지 않는다.

### Current G8-B acceptance sequence

- Sand Fall의 complete-settle/all-sleep behavior와 published Harness pilot은 accepted/immutable이다.
- Water Flow는 같은 `feature/m0-g8b-scenario-suite` line에서 finite fixture와 production physics를 변경하지 않은 Harness v1 implementation candidate다. FAST checks만 기록되었고 exact source SHA, FULL/smoke, first scratch run, one candidate run, automatic verdict와 user acceptance는 pending이다.
- Water candidate는 `run_experiment.bat water-flow --mode scratch`로 첫 raw observation을 보존한 뒤 clean source에서 기본 candidate mode를 정확히 한 번 게시한다. unique/no-overwrite/receipt-last 정책을 유지한다.
- Fire / Heat, Pressure Burst, Heavy Mixed World, G8-C는 Water 결과 뒤에도 자동 시작하지 않는다. Water 자동 판정은 G8-B closure가 아니다.

### Optimization Stop Rule

G8 공식 행렬이 다음 중 하나를 증명하지 않는 한 G7-C, active compaction, indirect dispatch 또는 공격적인 packing으로 바로 넘어가지 않는다.

- 대표 workload에서 60 TPS 또는 필요한 render responsiveness를 방해하는 명확한 병목
- 새로운 콘텐츠를 추가할 여유를 심각하게 제한하는 subsystem cost
- 현재 구조가 world scale이나 user interaction을 실제로 막는 증거

병목이 확인되면 한 번에 하나의 최적화 가설만 baseline과 비교한다.

---

## G9 — Playable First World

G9는 마지막에 정답 장면을 보여주는 단일 승인 절차가 아니다.

사용자가 실제로 세계를 만들고, 관찰하고, 다시 바꾸는 **첫 플레이 가능한 vertical slice**다.

### G9-A — Sandbox Interaction

최소 상호작용 도구:

- Matter 선택
- 마우스/포인터로 그리기
- ERASE
- brush size
- Heat 또는 Temperature 조작 도구
- pause / play / single-step / speed control / reset
- pan / zoom
- 실험 preset load
- 실제 GPU Production Simulation에 대한 안전한 edit command path

고정 validation fixture만 관찰하는 것이 아니라 플레이어가 장면의 원인을 직접 만들어야 한다.

### G9-B — Open Emergence

현재 M0 Matter와 공통 Rule만으로 자유 실험을 진행한다.

대표 가능성:

```text
Sand / Water / Oil movement and layering
Ice ↔ Water ↔ Steam
Wood / Oil combustion
sealed chamber → Pressure → rupture → vent
Heat / Smoke / Pressure가 다른 실험으로 이어지는 chain
```

사용자에게 정답과 기대 결과를 먼저 알려주는 observatory가 아니라, 스스로 구조와 조건을 만드는 sandbox에서 검증한다.

### G9-C — Discovery MVP

Doodle God 계열의 발견 감각을 최소 형태로 실제 simulation과 연결한다.

처음 관찰한 의미 있는 현상을 기록한다.

- Phase Change
- Combustion Started / Extinguished
- Pressure Generated
- Rupture / Vent
- Matter Transformation
- 의미 있는 무반응 또는 저항성

정확한 threshold, 계수와 남은 발견 개수는 기본적으로 공개하지 않는다.

> **게임은 현상을 알려주고, 공식은 숨긴다.**

G9의 Discovery는 완성된 도감이 아니라 플레이어의 작은 연구 노트다.

### G9-D — Honest Presentation

최종 art stack 전체를 요구하지 않는다. 하지만 simulation causality를 읽을 수 있는 최소한의 modern feedback은 필요하다.

후보:

- combustion source → smooth flame / glow
- Smoke distribution → softened smoke density
- Temperature → heat haze / emissive response
- rupture / vent → short shockwave or pressure release cue
- 중요한 event의 기본 sound feedback

원칙:

> **결과는 정직하게, 감각은 과장한다.**

실제로 발생하지 않은 이동이나 반응을 보여주어 세계 법칙을 오해하게 만들면 안 된다.

### G9-E — User Product Validation

M0는 사용자가 직접 sandbox를 플레이한 뒤 승인해야 한다.

핵심 성공 신호:

- 지시 없이 두 번째 실험을 시작한다.
- 같은 Matter를 다른 용도로 다시 사용한다.
- 정확한 수치를 몰라도 원인과 결과를 설명할 수 있다.
- 예상하지 못했지만 세계 안에서 납득 가능한 결과를 발견한다.
- 관찰 후 “그럼 이것까지 넣어보면?”이라는 다음 행동이 실제로 나온다.

M0 `ACHIEVED`는 G8 숫자가 좋다는 이유만으로 선언하지 않는다.

---

# Proposed M1 — Interaction Grammar Alpha

M0가 “첫 세계가 재미있는가”를 증명하면 M1은 “콘텐츠를 늘려도 엔진과 게임성이 함께 확장되는가”를 증명한다.

## Product Goal

새 Material이 단일 레시피가 아니라 여러 실험의 출발점이 되게 한다.

## First Prototype Bundles

### 1. Trapped Fuel / Pressure Accident

```text
Methane Clathrate
+ Heat
→ Methane release
→ confined accumulation
→ ignition
→ Heat + Pressure
→ rupture
→ vent
```

기존 Temperature, Transition, Gas, Density, Combustion, Pressure, Rupture 문법을 재사용한다.

전용 `clathrate_explosion` 또는 radial blast solver를 만들지 않는다.

### 2. World Fabrication

```text
Clay + Water condition → workable Clay
workable Clay + Heat → Brick

Sand + extreme Heat → Glass
```

세계가 파괴되는 것뿐 아니라 공간 안에서 재료가 생산되고 구조로 바뀌는 경험을 만든다.

### Optional Third Bundle — Volatile Atmosphere

```text
Dry Ice + Heat → CO2
CO2 density accumulation
CO2 adjacency → combustion suppression
```

앞선 두 bundle이 안정된 뒤 추가한다.

## Engine Goal

복잡한 full Rule DSL editor를 만들지 않고 작은 compiled interaction grammar를 증명한다.

후보 vocabulary:

```text
Condition
- Neighbor Material / compiled tag
- Temperature range
- Pressure range
- State bit
- schedule tier

Effect
- TransformSelf
- Set / Clear State
- Add Heat
- Add Pressure
- Request Spawn
- Emit Semantic Event
```

실행 방향:

```text
material_id
→ that Material's small precompiled rule range
→ Ordered First-Match
→ Write Self Next
→ ownership change only: Claim / Resolve
```

## M1 Evidence Direction

- 새 Material마다 별도 full-world GPU pass가 늘어나지 않음
- Material 이름 branch가 shader에 무제한 증가하지 않음
- 모든 Cell에 universal wetness/capacity/corrosion progress를 추가하지 않음
- 신규 bundle 전후 subsystem cost 비교 가능
- mixed-world integrity 유지
- 각 신규 Matter가 두 개 이상의 유용한 실험에 참여
- 플레이어가 결과의 원인과 용도를 이해

정확한 M1 Evidence Gate는 M0 승인 후 `MILESTONES.md`에 별도로 확정한다.

---

# Proposed M2 — Experimentation Power

플레이어가 세계를 더 빨리 이해하고 조건을 바꾸어 비교할 수 있게 한다.

우선순위:

- recent 10-second Rewind
- 1-second granularity state snapshots
- snapshot에서 다시 simulation 계속
- save experiment state
- fork experiment
- before / after comparison
- thermal / pressure / activity overlays
- causal event inspection
- Discovery notebook 확장

Rewind는 단순 편의 기능이 아니라 핵심 실험 도구다.

bit-exact command replay보다 실제 GPU state snapshot을 기본으로 한다.

---

# Proposed M3 — Evidence-driven Optimization

최적화는 고정된 Phase 1이 아니라 실제 측정이 요구할 때 진행한다.

M0와 M1의 대표 workload를 다시 측정한 뒤 병목만 선택한다.

현재 후보:

1. Activity reduction / management cost
2. intermediate buffer copies / tick residual
3. field-specific activity refinement
4. active compaction / indirect dispatch
5. descriptor and compiled-rule packing
6. chunk size 32 / 64 / 128 comparison
7. shared-memory tile + halo
8. f16 Temperature / Pressure experiment
9. Rewind storage optimization

후보 순서는 약속이 아니다.

각 최적화는:

- baseline 대비 단독 비교
- physics / integrity equivalence
- user-visible world fidelity 유지
- 관리 비용을 포함한 실제 개선

을 증명해야 한다.

---

# Proposed M4 — Matter Families and World Building

평면적인 Material 수 증가 대신 **행동 family와 대표 Matter**로 확장한다.

첫 분해 후보:

```text
METAL
- Iron: structure / rust / melt
- Copper: staged oxidation / strong conduction
- Lead: density / lower-melt / later shielding
- Mercury: liquid metal / extreme density

STONE / MINERAL
- Stone: inert baseline
- Basalt: slow Lava cooling result
- Obsidian: rapid cooling / glass-like brittle result
- Limestone / Calcite: dissolve / deposit / cement chain
- Crystal / Amethyst: nucleate / grow

PLANT / ECOLOGY
- Plant: foundation abstraction
- Vine: climb / spread on support
- Moss: colonize wet mineral surface
- Fungus: consume residue / decompose
- Algae: aquatic growth
```

선정 규칙:

> **새 이름이 아니라 새로운 interaction verb가 있을 때만 별도 Matter identity를 사용한다.**

38개 roster를 한 번에 구현하지 않는다. 각 family는 기존 세계 문법과 연결되는 작은 bundle로 도입하고 사용자 검증을 받는다.

---

# Proposed M5 — More Transferable Fields

Temperature와 Pressure에서 검증한 Minimum Sufficient Physics 패턴을 새 Field로 확장한다.

후보 순서:

- Electricity
- Gameplay Light
- Radiation
- additional force / field systems

각 Field는 하나의 완결된 vertical slice로 시작한다.

예:

```text
Electricity
→ conductive + strength / loss frontier

Gameplay Light
→ transmit / absorb / reflect + intensity

Radiation
→ intensity + attenuation / blocking
```

현실 equation 전체를 먼저 구현하지 않는다.

각 Field는:

- 기존 Matter와 여러 상호작용을 만드는가
- 플레이어가 실험으로 이해할 수 있는가
- 최소 local state와 transfer로 충분한가
- 큰 세계에서 감당 가능한가

를 먼저 증명한다.

---

# Long-term World Layers

장기 개념:

1. Matter
2. Field
3. Agent
4. Concept
5. Meta

가능한 방향:

```text
Matter
→ Energy / Chemistry
→ Life / Ecosystem
→ Machine
→ Information
→ Language
→ Society / Civilization
→ Myth / Belief
→ AI
→ Space / Time
→ World Rules
```

이 목록은 빈 framework를 미리 만들라는 뜻이 아니다.

각 계층은 이전 계층이 실제 플레이에서 재미있고 성능상 지속 가능하다는 증거가 생긴 뒤 추가한다.

---

## Continuous Product Axis — Presentation

Presentation은 마지막 장식 Phase가 아니다.

각 Milestone에서 새 simulation truth를 플레이어가 읽을 수 있게 만드는 최소한의 visual/audio feedback을 함께 설계한다.

```text
Simulation Truth
→ Semantic State / Event
→ Presentation Extraction
→ Modern FX / Audio
```

Cell simulation resolution은 최종 FX resolution이나 retro pixel-art style을 강제하지 않는다.

---

## Continuous Product Axis — Discovery

Discovery는 별도 메뉴 게임이 아니라 실제 sandbox 관찰에서 파생된다.

```text
actual simulation event
→ first meaningful observation
→ player research note
```

정확한 공식과 남은 개수는 숨기고, 플레이어가 관찰한 현상과 관계를 보존한다.

G9에서 MVP를 시작하고 M2 이후 실험 비교·causal inspection과 함께 확장한다.

---

## Research and Candidate Promotion Policy

현재 research corpus는 이미 충분히 넓다.

기본 작업은 더 많은 이름을 수집하는 것이 아니라 기존 후보를 선택하고 실제 세계에서 검증하는 것이다.

Promotion pipeline:

```text
Research Candidate
→ lore-free interaction verb
→ existing real / historical coverage check
→ user selection
→ ADR / SPEC / content definition
→ implementation
→ mixed-world evidence
→ direct play validation
→ VALIDATED
```

Research 문서에 있다는 이유만으로 Material을 구현하지 않는다.

DAN-BALL, Minecraft, Powder Game 2와 다른 작품에서는 이름보다 재사용 가능한 행동 문법을 추출한다.

---

## Future Developer Tool — Interaction Lab

현재는 `DEFERRED`.

완성된 Material/Rule을 actual GPU Production Simulation에 자동 투입해 기존 Matter와 대표 환경에서 상호작용을 탐색하는 개발 도구다.

- Material을 자동 생성하는 도구가 아님
- simulation truth는 실제 엔진 결과
- 예상 밖 chain과 regression 발견이 목적
- 본 게임과 content authoring loop보다 우선하지 않음

M1 이후 Material 수와 regression surface가 실제로 커져 수동 검증이 병목이 될 때 재평가한다.

---

## Non-goals for the Near Term

- Browser/macOS product parity
- broad low-end GPU support
- true infinite world
- exact physical simulation
- exact global energy accounting
- deterministic multiplayer / bit-perfect replay architecture
- full Rule DSL editor
- 수십 개 Material의 일괄 구현
- atmosphere composition / universal mixture solver
- 모든 미래 progress state를 Cell에 미리 추가
- Interaction Lab 완성
- Life / Agent / Civilization을 M0 또는 M1에 조기 도입
- 측정 근거 없는 f16 / packing / indirect dispatch

---

## Current Execution Order

1. **G8-A verified evidence candidate** — official capture와 independent verification은 완료했다. 동일 source SHA의 user visual validation은 pending이다.
2. **Canonical Recovery local integration** — 구현/증거선과 연구/문서선의 병합·검증은 완료했다. recovery branch push, recovery PR, `main` 승격은 pending이다.
3. **User decision** — 다음 실행 범위를 자동 선택하지 않는다. G8-B/G8-C, G9, 또는 M0 이후 P1 검토 중 하나를 별도 승인으로 정한다.
4. **Dependency boundary** — G8-B 다섯 benchmark fixture와 G8-C 공식 multi-trial matrix를 마쳐야 G8 전체를 닫을 수 있다.
5. **Product path** — G9-A/G9-B sandbox input·edit loop와 open emergence 뒤 G9-C/G9-D/G9-E Discovery·Presentation·direct user play approval을 연결한다.
6. M0 승인 후에만 **P1 identity/descriptor 등록**과 **M1 Interaction Grammar Alpha**를 정식 구현·Evidence Gate로 검토한다.

현재 목표는 더 빠른 실험실을 만드는 것이 아니다.

> **이미 만든 빠른 세계를, 플레이어가 자기 세계로 사용할 수 있게 만드는 것.**
