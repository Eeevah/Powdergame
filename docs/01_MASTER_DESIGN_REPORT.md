# Powdergame Master Design Report — Foundation Synthesis

> [!IMPORTANT]
> 이 문서는 2026-08-15 Foundation Design Session의 사용자 선택과 보완을 기준으로 기존 초기 설계 보고서를 전면 재작성한 종합 설계 문서다.
>
> 현재 구현 계약은 `vision/USER_VISION.md`, `architecture/decisions/`, `specs/`, `development/`, `planning/MILESTONES.md`가 더 구체적인 경우 그 문서를 따른다.
>
> 질문·선택지·사용자 코멘트·중간에 변경된 결정의 provenance는 `design-history/2026-08-15-foundation-design-session.md`에 보존한다.

---

# 1. Executive Summary

Powdergame의 목표는 단순히 원소 수가 많은 falling-sand 게임을 만드는 것이 아니다.

이 프로젝트는:

- **Doodle God의 조합·발견·세계 창조의 감각**
- **DAN-BALL Powder Game의 직접적인 공간 조작·국소 상호작용·창발성**

을 하나의 실제 simulation world에 연결한다.

핵심 제품 문장:

> **플레이어에게 물질을 주는 게임이 아니라, 우주를 발명할 수 있는 문법을 주는 게임.**

플레이어는 Matter를 뿌리고 정답 레시피를 맞히는 사람이 아니라, 자기 가설을 세계에 던지고 그 세계가 자기 규칙으로 답하는 것을 관찰하는 창조자가 된다.

Powdergame의 핵심 재미는 다음 루프다.

```text
호기심
→ 직접 배치/실험
→ 반응 발생
→ 예상 밖의 연쇄
→ 관찰
→ 원인 이해
→ 다시 이용
→ 더 큰 구조 제작
→ 새로운 의문
```

---

# 2. 현실은 참고자료이지 법전이 아니다

Powdergame은 과학 시뮬레이터가 아니다.

현실의:

- 열
- 압력
- 부력
- 연소
- 산화
- 전기
- 빛
- 방사선
- 유체

같은 현상은 플레이어가 이미 직관을 갖고 있으므로 훌륭한 참고자료다.

하지만 현실을 정확히 재현하는 것은 목표가 아니다.

## 2.1 User Principle

> **현실을 구현하는 것이 아니라 가상의 재미있는 놀이터를 만든다. 핵심은 나만의 세계 창조다. 현실 고증보다 게임 안에서 이해 가능한 논리와 상호작용이 중요하다.**

따라서:

- 현실에 없는 Matter를 만들 수 있다.
- 현실에 없는 상변화를 만들 수 있다.
- 현실과 다른 연소 조건을 만들 수 있다.
- 특정 가상 Matter가 열을 흡수하고 압력으로 전기를 방출해도 된다.
- 현실에서 반응하지 않는 Matter가 Powdergame에서는 특정 조건에 반응해도 된다.

중요한 것은:

1. 플레이어가 인과를 이해할 수 있는가
2. 반복해서 이용할 수 있는가
3. 다른 시스템과 재미있는 연쇄를 만드는가
4. 같은 조건에서 세계가 상식적으로 일관되는가
5. 충분히 싸게 계산할 수 있는가

이다.

### 가상 Matter 예시

```text
Vibranium
- STATIC
- Density Rank 매우 높음
- Pressure Resistance 극단적으로 높음
- Thermal Response 낮음
- 대부분의 Reaction에 inert
```

현실에 존재하지 않아도 플레이어가 “이건 거의 반응하지 않는 초강력 방벽이구나”라고 학습할 수 있다면 좋은 Material이다.

---

# 3. 복잡성은 원소 수보다 관계에서 나온다

`1,000 Materials` 자체가 목표가 아니다.

같은 Material이라도:

- Temperature
- Pressure
- 주변 Matter
- Density
- combustion state
- future Electricity
- Radiation
- Gameplay Light

등과 관계를 맺을 때 깊이가 생긴다.

Powdergame은 한 Cell 안에 정보를 끝없이 쌓아서 복잡성을 만드는 대신:

> **작은 Cell들이 공간적으로 만나며 관계의 수를 늘린다.**

이를 통해 개발자가 직접 작성하지 않은 연쇄가 발생해야 한다.

예:

```text
Laser
→ Metal absorbs Light
→ Temperature 상승
→ Metal transition
→ Molten Metal 이동
→ Water 접촉
→ Steam
→ Pressure
→ Wall rupture
```

`Laser + Wall = Explosion`이라는 특별 Rule 없이도 이런 chain이 생기는 것이 목표다.

---

# 4. Core World Identity

## 4.1 One Cell = Max One Matter

Powdergame의 가장 강한 world invariant다.

한 Cell에는 Matter가 최대 하나만 존재한다.

```text
[Water]
```

은 가능하지만:

```text
[Water 40% + Oil 30% + Air 30%]
```

같은 내부 혼합 Cell은 기본 world model이 아니다.

이것은 단순한 기술 제한이 아니라 Powder Game의 정체성이다.

## 4.2 Unit Cell Quantity

Matter가 Cell에 있으면 한 단위다.

per-cell:

```text
0.3 Water
2.7 Sand mass
```

같은 amount는 기본적으로 두지 않는다.

Density나 Heat Capacity는 Matter의 property이지 Cell 안의 양이 아니다.

## 4.3 EMPTY is not Matter

`EMPTY`는 Matter가 없다는 뜻이다.

- Air가 아님
- Vacuum Material이 아님
- thermal medium이 아님
- pressure medium이 아님

Air/Oxygen/Gas가 필요하면 실제 Matter로 추가한다.

## 4.4 Fields are not additional Matter

Cell은 개념적으로:

```text
material_id
Temperature
Pressure
minimal flags/state
```

를 가질 수 있다.

Temperature/Pressure가 있다고 한 Cell에 Matter가 두 개 존재하는 것이 아니다.

---

# 5. World Size and Boundary

## 5.1 Finite World

현재 reference world:

```text
2048 × 2048
= 4,194,304 Cells
```

Infinite world가 아니다.

World size는 `WorldConfig`로 관리한다.

## 5.2 Initial Chunk

```text
64 × 64 Cells
32 × 32 Chunks
= 1024 Chunks
```

64는 초기 benchmark baseline이다. 영구 invariant는 아니다.

## 5.3 Editable Outer BLOCK

DAN-BALL Powder Game의 감각을 따른다.

- outer BLOCK을 지울 수 있다.
- 뒤에 invisible wall을 두지 않는다.
- boundary 밖으로 빠진 Matter는 Void로 소멸한다.
- boundary를 지운다고 world가 확장되지는 않는다.

---

# 6. Platform and Architecture

현재 제품 경로는 범용 multi-platform이 아니다.

```text
Platform: Windows
Language: Rust
Window/Input: winit
GPU API: wgpu
Backend: DX12
Primary Performance Target: NVIDIA RTX 5090
```

현재는 Browser/macOS/다른 GPU fallback을 위해 architecture를 복잡하게 만들지 않는다.

## 6.1 System Boundaries

```text
Simulation Core
    ↓
Game Runtime
    ↓
Presentation / Platform
```

### Simulation Core

- Matter/Field rule execution
- GPU Production Simulation
- headless path
- small CPU Reference

### Game Runtime

- world lifecycle
- commands/config
- save/load orchestration
- event/diagnostic bridge

### Presentation

- Windows rendering/input
- visual effects
- sound
- overlays

Presentation은 Simulation을 읽을 수 있지만 gameplay state를 임의로 수정하지 않는다.

---

# 7. GPU Production is the World Truth

Production simulation은 GPU가 authoritative하다.

```text
CPU
→ input / config / command / orchestration

GPU
→ world simulation
→ authoritative state
```

CPU↔GPU로 전체 world를 매 Tick 복사하지 않는다.

CPU Reference는:

- 작은 test world
- 이해 가능한 reference
- algorithm debug
- semantic comparison

용이다.

GPU Production과 pixel-perfect하게 같을 필요가 없다.

---

# 8. Determinism Policy

Powdergame은 bit-perfect deterministic replay를 목표로 하지 않는다.

> **No intentional randomness; no performance sacrifice for bit-perfect replay.**

GPU 병렬 실행/float approximation 때문에 미세한 결과 차이가 나는 것은 허용한다.

목표:

> **Non-exact but stable.**

허용:

- 미세한 pile shape 차이
- valid winner 차이
- float approximation

금지:

- 한 Cell에 두 Matter
- Matter corruption
- race에 의한 부당한 duplication/loss
- NaN/Infinity runaway
- out-of-bounds

Rewind는 exact deterministic replay 대신 실제 state snapshot을 사용한다.

---

# 9. Core GPU Interaction Pattern

## 9.1 Read Neighbors, Write Self

일반 interaction의 기본:

```text
Read Current Self
+ Read Needed Neighbors
→ Cheap Local Rule
→ Write Self Next
```

예:

```text
Metal Cell
→ neighbor Acid 확인
→ 자기 자신 → Corroded Metal
```

Acid thread가 Metal을 직접 수정하지 않는다.

## 9.2 Ownership Changes only use Resolve

다음은 예외다.

- movement
- swap
- multi-cell spawn
- phase expansion
- multiple sources to same target

이때만:

```text
Propose
→ Claim / Resolve
→ Commit
```

을 사용한다.

> **논리 충돌 때문에 무거운 Resolve를 만들지 않는다. 실제 Cell ownership 충돌만 Resolve한다.**

---

# 10. Locality

## 10.1 Matter Interaction

일반 Reaction 최대 범위:

```text
8-neighbor
```

## 10.2 Field Propagation

기본:

```text
4-neighbor
```

Temperature, Pressure, 향후 Electricity/diffusive Radiation은 4-neighbor baseline부터 검증한다.

## 10.3 Movement

behavior별 필요한 방향만 읽는다.

First-Match:

```text
below?
→ possible: use and stop
→ blocked: next candidate
```

> **알 필요 없는 데이터는 읽지 않는다.**

---

# 11. Movement Families

M0에는 네 family가 있다.

## STATIC

일반 gravity/density movement 없음.

Pressure rupture나 special Rule은 영향을 줄 수 있다.

## POWDER

```text
down
→ down-diagonal
```

예: Sand.

## LIQUID

```text
down
→ down-diagonal
→ lateral
```

한 Tick에 먼 빈 Cell을 탐색하지 않는다.

## GAS

높은 mobility를 갖지만 **항상 움직여야 하는 Matter가 아니다.**

위/대각/측면의 local movement를 사용할 수 있으나 안정된 bulk에서 의미 없는 이동을 계속 계산하지 않는다.

---

# 12. Density: Buoyancy without a Buoyancy Solver

Density는 실제 kg/m³가 아니라 작은 integer rank다.

필요한 관계:

```text
A > B
A == B
A < B
```

예시:

```text
Steam       20
Oil         70
Water       90
Sand        150
MoltenMetal 220
```

실제 값은 gameplay data다.

## 12.1 Local Displacement

A가 아래 Cell B를 볼 때:

```text
B == EMPTY
→ normal move

B STATIC/non-movable
→ stop

B movable
→ compare Density Rank
→ swap candidate if ordering favors it
```

이 반복으로:

- Sand sinks in Water
- Oil floats over Water
- heavy/light Gas stratifies
- fictional powder can float

같은 현상을 만든다.

핵심 문장:

> **부력을 계산하지 않는다. 정렬한다.**

Density는 per-cell에 반복 저장하지 않고 Material property로 둔다.

---

# 13. Minimum Sufficient Physics

이 프로젝트의 가장 중요한 엔진 철학이다.

> **현실 공식을 재현하지 않고, 플레이어가 현상을 이해하고 이용하는 데 필요한 최소 상태와 최소 local operation만 계산한다.**

Representation rule:

```text
continuous value needed → f32 / proper numeric
ordering only           → integer rank
boolean only            → bit
few states              → small enum
```

## 13.1 Why

큰 world에서:

- Temperature
- Pressure
- movement
- reaction
- combustion
- future electricity/radiation/light

이 모두 동시에 작동하려면 한 Cell의 비용이 극도로 작아야 한다.

> **싸구려 Rule을 수백만 개 동시에 돌려서 비싼 세계를 만든다.**

---

# 14. Temperature

Temperature는 단순 rank만으로는 충분하지 않다.

M0 baseline:

```text
f32 Temperature
4-neighbor propagation
```

### Minimum thermal properties

필요하면 Material에:

- conductivity
- heat capacity
- transition threshold
- ignition condition

을 둔다.

현실의 정확한 thermodynamics는 목표가 아니다.

개념적으로:

```text
ΔT = self - neighbor

meaningful difference 없음
→ no work / equilibrium

meaningful difference 있음
→ cheap transfer
→ self next temperature
```

아주 작은 ΔT를 영원히 계산하지 않기 위한 thermal deadband는 benchmark/gameplay 검증 후보다.

f16은 baseline이 아니다. 실제 병목이 확인된 뒤 experiment한다.

---

# 15. Phase Transition

M0 대표:

```text
Ice ↔ Water ↔ Steam
```

정확한 현실 0°C/100°C를 따르는 것이 계약은 아니다.

Transition graph는 game data다.

모든 Material이 현실적인 Solid/Liquid/Gas 세 상태를 가져야 하는 것도 아니다.

가상 transition 예:

```text
Cryosteel
→ High Heat
→ Brittle Cryosteel
```

## 15.1 Transition Yield

상변화는 1:1일 필요가 없다.

```text
1 Water
→ multiple Steam placement requests
```

공간 부족 시 unresolved expansion을 Pressure로 연결할 수 있다.

---

# 16. Pressure

Pressure는 정밀 compressible-fluid solver가 아니다.

M0 baseline:

```text
f32 scalar pressure
4-neighbor local propagation
```

별도 per-cell velocity vector는 처음에 두지 않는다.

방향은 local ΔP에서 유도한다.

대표 causal chain:

```text
Water heated
→ Steam expansion
→ insufficient room
→ Pressure generated
→ local propagation
→ movable Matter pushed
→ resistant Matter holds
→ pressure > rupture threshold
→ Wall rupture
→ vent
```

밀폐된 공간에서 Pressure가 시간이 지났다는 이유만으로 그냥 0으로 사라져서는 안 된다.

---

# 17. Fire / Combustion

Fire는 M0에서 permanent orange Matter가 아니다.

```text
Fuel Matter
+ sufficient thermal condition
→ combustion
→ Heat
→ Smoke
→ flame presentation event
```

Wood와 Oil은 같은 공통 grammar를 쓰는 서로 다른 Material 예시다.

Oxygen은 현실에 필요하다는 이유만으로 하드코딩하지 않는다.

나중에 oxidizer manipulation이 재미를 만들 때 system으로 추가한다.

---

# 18. Reaction Architecture

Material은 자기 interaction rule 목록을 소유한다.

예:

```text
Oil
- Hot neighbor → combustion
- Acid neighbor → special reaction

Metal
- Acid neighbor → corrosion
```

거대한 global pair database에 모든 관계를 강제로 정규화하지 않는다.

## 18.1 Ordered First-Match

Material별 Rule은 load/compile 시 미리 정렬한다.

runtime:

```text
rule 1 matches?
→ yes: select, stop
→ no: next
```

모든 candidate를 모아 다시 sorting/resolution하지 않는다.

## 18.2 Coarse Category Order

세계 전체에는 소수의 coarse category만 둔다.

예:

```text
Critical / Destroy
Phase Transition
Special Reaction
Combustion
State Change
```

정확한 최종 order는 구현/benchmark에서 조정 가능하다.

숫자 priority jungle을 만들지 않는다.

---

# 19. Loose Causal Phases

한 Tick 안에서 모든 원인을 즉시 연결하기 위해 full-world barrier를 반복하지 않는다.

예:

```text
Tick N
Wood temperature rises

Tick N+1
Ignition observes new temperature
→ combustion starts
```

60 TPS에서 자연스럽다면 허용한다.

> **물리적 인과는 조금 늦어도 된다. 상태 무결성은 늦으면 안 된다.**

---

# 20. Active / Sleep Architecture

성능은 Matter 수가 아니라 실제 변화 가능한 영역에 비례하도록 만든다.

## 20.1 Active Chunk

Chunk는 subsystem별 activity를 가질 수 있다.

```text
Matter Active
Thermal Active
Pressure Active
Reaction Active
```

## 20.2 Short Stable Period → Sleep

Chunk는 몇 Tick 동안 의미 있는 변화가 없으면 Sleep 후보가 된다.

정확한 Tick 수는 benchmark한다.

## 20.3 Slow ≠ Sleeping

천천히 타는 Wood는 여전히 변화 중이다.

따라서 관련 Thermal/Combustion/Reaction은 Active다.

## 20.4 Stable Bulk

다음은 world state를 바꾸지 않는다.

```text
Water ↔ Water
Steam ↔ Steam
```

따라서 안정된 Liquid/Gas bulk 내부는 Sleep할 수 있다.

실제 work는:

- EMPTY interface
- different Matter interface
- density inversion
- Temperature gradient
- Pressure gradient
- active reaction frontier

에 집중한다.

핵심:

> **물질의 양이 아니라 변화 가능한 영역이 계산량을 결정하게 한다.**

---

# 21. Slow Rules

산화/부식/성장/노화처럼 느린 Rule은 60Hz로 모든 Cell을 검사하지 않는다.

후보 tier:

```text
FAST
MEDIUM
SLOW
VERY_SLOW
```

좌표 기반 분산 schedule 등으로 load를 시간축에 분산할 수 있다.

그러나:

```text
매 Tick 4.19M thread launch
→ 대부분 '내 차례 아님' 하고 exit
```

같은 가짜 최적화는 피한다.

## 21.1 No Universal Progress Field

초기에는 느린 변화를 위해 모든 Cell에:

```text
oxidation_progress
wetness_progress
growth_progress
```

를 넣는 아이디어가 있었지만 사용자 비용 검토 후 폐기했다.

기본은:

```text
Copper
→ Weathered Copper
→ Oxidized Copper
```

같은 Material transition이다.

정말 continuous state가 필요한 특정 gameplay가 생길 때만 별도 state를 추가한다.

---

# 22. Approximate Conservation

정확한 글로벌 mass/energy 회계를 하지 않는다.

싸게 가능하면 local transfer는 대략 보존한다.

```text
A loses heat
B gains similar heat
```

하지만 game world Rule은 energy source/sink를 만들 수 있다.

```text
Magic Crystal → Heat
Void Matter → Energy disappears
Explosion → Heat + Pressure source
```

핵심:

> **로컬에서는 납득 가능하게, 글로벌에서는 회계하지 않는다.**

---

# 23. Future Physics Extension Pattern

M0에는 넣지 않지만 같은 철학으로 확장할 수 있다.

## Electricity

```text
conductive?
+ electrical strength
+ material loss/resistance
→ local frontier propagation
```

전원이 제거되면 strength는 전달/손실로 줄어들 수 있다.

## Radiation

```text
intensity
→ blocking / attenuation
→ remaining intensity
```

## Gameplay Light

Presentation Light와 분리한다.

Gameplay interaction이 필요할 때만:

```text
intensity
+ transparent / absorb / reflect
→ next beam state
```

## Explosion

복잡한 별도 폭발 solver보다:

```text
inject Heat
inject Pressure
emit Presentation Event
```

를 하고 기존 systems가 결과를 만든다.

---

# 24. Simulation vs Presentation

원칙:

> **결과는 정직하게, 감각은 과장한다.**

## Simulation Truth

- Matter movement
- transition
- Temperature
- Pressure
- combustion
- rupture
- gameplay electricity/light/radiation 등

## Presentation Effects

- glow
- heat haze
- shockwave visual
- debris
- distortion
- sound
- camera impulse

Presentation이 실제로 일어나지 않은 simulation outcome을 거짓으로 보여 플레이어의 이해를 깨뜨리면 안 된다.

---

# 25. Discovery System

Discovery는 정답표가 아니다.

## 25.1 Phenomenon-level discovery

A와 B 사이에서 처음 관찰한:

- Temperature increase
- Pressure generation
- phase change
- transformation
- combustion

같은 **현상**을 기록한다.

정확한 threshold/계수는 기본적으로 숨긴다.

## 25.2 Hidden knowledge

사전은:

> 아직 발견하지 못한 성질이 있다.

정도는 알려줄 수 있다.

하지만:

```text
4 / 17 discovered
```

같은 exact remaining count는 기본적으로 보여주지 않는다.

> **사전은 정답표가 아니라 플레이어가 발견한 세계의 연구 노트다.**

---

# 26. Rewind

Rewind는 단순 undo가 아니라 experiment tool이다.

현재 방향:

- recent 10 seconds
- 1-second granularity
- up to 10 snapshots
- 과거 상태로 복귀
- 조건을 바꾸고 다시 simulation 가능

GPU simulation은 bit-exact deterministic하지 않으므로 actual state snapshot을 사용한다.

full snapshots이 충분히 싸다면 단순한 방식을 사용할 수 있고, 아니면 keyframe + changed-chunk delta를 benchmark한다.

---

# 27. Interaction Lab — Deferred Developer Tool

Interaction Lab은 Material 생성기가 아니다.

사용자가 강조한 역할:

```text
Already-defined Material + Rules
→ Actual GPU Production Simulation
→ Existing Materials + representative environments
→ Observe real interactions
→ Find unknown / unexpected / regression behavior
```

기본 탐색 방향:

- new Matter vs existing Matter pair
- representative Temperature/Pressure/open/confined conditions

실제 truth는 GPU Simulation 결과다.

하지만 Lab이 너무 큰 별도 프로젝트가 될 수 있기 때문에 현재는 **DEFERRED**.

본 게임보다 우선하지 않는다.

M0에는 headless simulation/state injection/observation hook 정도만 자연스럽게 유지한다.

---

# 28. DAN-BALL as an Idea Mine

DAN-BALL Powder Game 1/2뿐 아니라 전체 작품군을 장기적인 idea mine으로 참고한다.

목적은 과거 기능을 그대로 복사하는 것이 아니다.

검토할 것:

- 어떤 mechanic이 재미있었는가
- 당시 구조/하드웨어 제약은 무엇이었는가
- 현대 RTX 5090 GPU simulation에서 더 크게 만들 수 있는가
- 현재 Powdergame의 Matter/Field/Discovery와 어떻게 결합할 수 있는가

연구 후보는 자동으로 Roadmap에 들어가지 않는다.

별도 candidate 검토 후 채택한다.

---

# 29. Long-term World Layers

장기적인 개념 구조:

1. Matter
2. Field
3. Agent
4. Concept
5. Meta

가능한 장기 trajectory:

```text
Matter
→ Energy / Chemistry
→ Life / Ecosystem
→ Machine
→ Information
→ Language
→ Society / Civilization
→ Belief / Myth
→ AI
→ Space / Time
→ World Rules
```

하지만 이것은 M0에서 framework를 미리 다 만들라는 뜻이 아니다.

현재는 Matter + Field만 구현한다.

Agent/Concept/Meta는 이전 계층이 실제로 재미있고 빠르게 동작한 뒤 추가한다.

---

# 30. M0 — First World

M0는 콘텐츠량을 증명하는 단계가 아니다.

핵심 질문:

> **2048×2048 world에서 수백만 개의 매우 싼 local rule을 RTX 5090에서 병렬 실행해 작은 규칙들이 실제로 살아 있는 세계를 만드는가?**

M0 Matter:

- Boundary Block
- Stone
- Sand
- Ice
- Water
- Steam
- Smoke
- Wood
- Oil

M0 systems:

- Static / Powder / Liquid / Gas
- Density Rank
- Temperature
- Ice ↔ Water ↔ Steam
- Combustion
- Pressure
- rupture / vent
- Active / Sleep

Evidence Gates:

```text
G0 Runtime
G1 World Integrity
G2 Local Movement
G3 Density
G4 Thermal / Reaction
G5 Pressure
G6 Parallel Integrity
G7 Sleeping
G8 Performance Evidence
G9 Product Validation
```

최종 `ACHIEVED`는 사용자가 직접 플레이하고 승인해야 한다.

---

# 31. Performance Strategy

M0에서 숫자 performance target을 억지로 정하지 않는다.

먼저 baseline을 만든다.

Required scenarios:

- Sand Fall
- Water Flow
- Fire / Heat
- Pressure Burst
- Heavy Mixed World

Required metrics:

- Render FPS
- simulation tick time
- GPU simulation time
- GPU rendering time
- Matter cost
- Thermal cost
- Pressure cost
- Reaction cost
- Resolve cost
- Active/Sleep management cost
- active Cell count
- active Chunk count
- VRAM usage

병목을 본 뒤 해당 subsystem만 최적화한다.

Candidate optimizations:

1. Active Chunk skipping
2. field-specific Active Set
3. stable frontier optimization
4. active compaction / indirect dispatch
5. shared-memory tile
6. descriptor packing
7. chunk size comparison
8. f16 experiment
9. subtile mask if needed

> **계산을 줄이려고 더 비싼 관리 시스템을 만들지 않는다.**

---

# 32. Documentation and Evidence

현재 저장소는 설계의 결론뿐 아니라 provenance를 보존한다.

- `vision/USER_VISION.md` — 무엇을 만들 것인가
- `architecture/decisions/` — 왜 이 구조를 선택했는가
- `specs/` — 현재 구현 계약
- `development/` — 구현/테스트/성능 철학
- `planning/MILESTONES.md` — 무엇을 증명해야 완료인가
- `design-history/2026-08-15-foundation-design-session.md` — 질문, 선택지, 사용자 선택, 추가 코멘트, superseded decision

문서화 원칙:

> **요약하지 않는다. 정리한다.**

사용자의 선택/교정/의도를 잃지 않는다.

---

# 33. Final Thesis

Powdergame은 현실 전체를 정확히 계산하려는 게임이 아니다.

우리는:

```text
작은 Matter
+ 작은 Field
+ 작은 local Rule
+ 작은 integer/bit/f32 state
+ massive GPU parallelism
```

을 이용해:

```text
열
→ 상변화
→ 압력
→ 파열
→ 이동
→ 연소
→ 또 다른 반응
```

같은 큰 현상을 만든다.

핵심 설계는 다음 두 문장으로 압축된다.

> **Game-Consistent Minimum Sufficient Physics.**
>
> **셀 하나는 극도로 싸게, 세계 전체는 믿을 수 없을 만큼 풍부하게.**

그리고 최종 제품 질문은 여전히 이것이다.

> **“이 세계에 이것을 넣으면 대체 무슨 일이 일어날까?”라는 생각을 계속 하게 만드는가?**

그 질문이 계속 생긴다면 Powdergame은 올바른 방향에 있다.
