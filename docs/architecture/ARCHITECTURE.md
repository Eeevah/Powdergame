# Powdergame Architecture

이 문서는 현재 합의된 Powdergame의 구조를 설명한다. 과거 연구 보고서의 플랫폼/결정성/월드 모델 제안과 충돌할 경우 최신 ADR과 SPEC이 우선한다.

---

## 1. Architectural Boundary

Powdergame은 세 개의 큰 경계로 나눈다.

```text
Simulation Core
    ↓
Game Runtime
    ↓
Presentation / FX / Platform
```

### Simulation Core

- Matter/Field authoritative rules
- GPU Production Simulation
- 작은 CPU Reference implementation
- headless 실행 가능
- rendering/input/audio와 분리

### Game Runtime

- World lifecycle
- player command 전달
- save/load orchestration
- simulation tick orchestration
- discovery/event 전달
- developer diagnostics hook

### Presentation / FX / Platform

- Windows window/input
- rendering
- visual effects
- audio
- overlays/debug UI
- simulation state/event의 presentation extraction
- simulation grid와 독립적인 high-resolution GPU FX 허용

Presentation은 Simulation state를 읽을 수 있지만 gameplay state를 임의로 수정하지 않는다.

**Cell simulation resolution은 presentation resolution 계약이 아니다.** Simulation이 discrete grid로 동작해도 최종 FX는 screen resolution, continuous coordinates, interpolation, procedural animation, post-processing을 사용할 수 있다.

---

## 2. Current Platform Target

현재 공식 타깃은 다음과 같다.

```text
Windows
Rust
winit
wgpu
DX12
RTX 5090 primary performance target
```

현재는 범용 GPU 호환성, Browser, macOS 지원을 위해 구조/성능을 희생하지 않는다.

Baseline GPU 구현은 개발·benchmark·debug 비교 기준으로 남길 수 있지만 runtime fallback compatibility path를 복잡하게 만들지 않는다.

Production path는 RTX 5090에서 실제로 검증된 가장 좋은 구현을 사용한다.

---

## 3. Repository Direction

개념적 workspace 구조:

```text
Powdergame/
├─ engine/
├─ apps/
│  └─ windows/
├─ tests/
├─ benches/
├─ examples/
├─ tools/
├─ assets/
└─ docs/
```

정확한 Rust crate 이름은 구현 시작 시 조정 가능하다. 중요한 것은 Simulation Core가 UI와 분리되고 headless 경로를 갖는 것이다.

---

## 4. World Model

### Reference World

초기 개발 기준:

```text
2048 × 2048
= 4,194,304 cells
```

월드는 finite하다. infinite world가 아니다.

크기는 `WorldConfig`에서 설정하며 엔진 전역에 상수로 하드코딩하지 않는다.

### Initial Chunk

```text
64 × 64 cells
32 × 32 chunks
= 1024 chunks
```

64는 초기 benchmark 기준일 뿐 영구적인 invariant가 아니다. 이후 32/64/128 등의 후보를 실제 측정으로 비교할 수 있다.

---

## 5. Core Cell Invariant

### Single Matter Occupancy

한 Cell은 동시에 Matter를 최대 하나만 가진다.

```text
Cell
├─ material_id
├─ temperature
├─ pressure
└─ minimal flags/state
```

Temperature/Pressure/state는 별도의 Matter가 아니다.

### Unit Quantity

Matter가 있으면 한 단위다. per-cell 0.3 Water 같은 질량/혼합량을 기본 모델로 두지 않는다.

### EMPTY

`EMPTY`는 Material Registry의 Matter가 아니다.

- Cell에 Matter가 없음을 나타낸다.
- EMPTY가 Air/Vacuum이라는 숨은 물질로 행동하지 않는다.
- EMPTY 자체는 열/압력 매질로 취급하지 않는다.
- Air/Oxygen/Gas가 필요하면 실제 Matter로 추가한다.

---

## 6. Boundary

DAN-BALL Powder Game의 감각을 참고해 outer boundary는 editable BLOCK으로 다룬다.

- Outer BLOCK을 ERASE할 수 있다.
- 보이지 않는 추가 벽을 두지 않는다.
- finite domain 밖으로 이동한 Matter는 Void로 소멸한다.
- 경계 제거는 world 확장을 의미하지 않는다.

---

## 7. World Layers

장기 개념 계약:

1. Matter
2. Field
3. Agent
4. Concept
5. Meta

현재 M0에서 구현하는 것은 Matter와 Field뿐이다.

미래 계층을 위한 빈 abstraction을 미리 코드에 넣지 않는다.

---

## 8. GPU Production Authority

실제 게임의 authoritative simulation state는 GPU에 둔다.

```text
CPU
→ commands / config / orchestration

GPU
→ production world simulation
→ production world state
```

CPU와 GPU 사이에 전체 월드를 매 Tick 복사하지 않는다.

CPU는 입력/명령/config를 전달하고 GPU는 simulation을 수행한다. CPU로 가져오는 것은 필요한 event, diagnostics, save/inspection 데이터 정도다.

### G9-A interactive edit boundary

The Sandbox keeps GPU production authority. Pointer input is rasterized into bounded, deterministic cell commands on CPU, but it never becomes a CPU copy of world truth. Once per redraw, before any normal simulation tick, duplicate cells coalesce with last-write-wins semantics and one GPU submission applies two compute passes to both Current and Next buffers. The split keeps each bind group within the hardware storage-buffer limit while preserving an atomic command boundary.

Draw is occupancy-safe and writes only when both Current and Next are EMPTY; a rejected Draw leaves all existing fields and flags unchanged. Accepted Draw writes both buffers with canonical field hygiene, including direct placement temperatures of -30°C for Ice and 80°C for Steam. Erase writes exact EMPTY hygiene. Heat/Cool preserve non-EMPTY identity and clamp finite temperature. Touched chunks plus a clipped eight-neighbor chunk halo are made runnable and edit-woken. Reset or preset change drops pending commands before staging the pristine product preset. The Inspector remains a separate bounded 24-byte, at-most-10-Hz observation path and is invalidated across edits/preset epochs. Its consumer separates requested hover from one presented sample: rapid hover changes keep the original sample identity under one 150 ms hold, then use a fixed Sampling panel until a current request completes. Selection generation prevents held or late samples from being relabelled as the new Cell; reset/preset/epoch/failure clears presentation immediately.

Sandbox rendering, physical-pixel cursor picking, Inspector hover and Heat/Cool brush feedback share one finite/clamped `WorldTransform`. Thermal feedback stores only tool/cell/diameter/time and never becomes world authority or a readback path. Camera pan/zoom changes presentation only and never changes `WorldConfig` or simulation coordinates.

### CPU Reference

CPU Reference는:

- 작은 테스트 세계
- 알고리즘 이해
- debugging
- 의미 비교

를 위한 읽기 쉬운 구현이다.

GPU Production과 bit-exact하게 일치할 필요가 없으며 GPU의 oracle이 아니다.

---

## 9. Simulation Timing

목표:

```text
Simulation: 60 TPS target
Rendering: independent / as fast as possible
```

Simulation이 극단적인 workload에서 60 TPS를 유지하지 못하면 카메라 밖의 물리를 낮은 빈도로 돌리는 방식으로 결과를 왜곡하지 않는다.

대신 전체 simulation time이 같이 느려질 수 있다. Presentation FPS는 가능한 유지하여 smooth slow-motion처럼 보이게 한다.

카메라 위치가 세계 법칙을 변경하면 안 된다.

---

## 10. Loose Causal Phases

한 Tick에 모든 물리적 인과관계를 즉시 이어 붙이기 위해 강한 barrier를 반복하지 않는다.

```text
Tick N Current World
 ├─ Temperature 후보
 ├─ Pressure 후보
 ├─ Movement 후보
 └─ Reaction 후보

Resolve / Commit where required

Tick N+1
→ 새 상태에 따른 다음 인과
```

플레이어가 자연스럽게 느낀다면 1 Tick 늦게 이어지는 현상은 허용한다.

> **물리적 인과는 조금 늦어도 된다. 상태 무결성은 늦으면 안 된다.**

One Cell = One Matter, 동일 목적지 경쟁, multi-cell 생성 등의 무결성 문제는 반드시 올바르게 해결한다.

---

## 11. Read Neighbors, Write Self

일반 상호작용의 핵심 병렬화 규칙:

> **Read Neighbors, Write Self.**

각 Cell은:

- 자기 Current state를 읽고
- 필요한 local neighbor를 읽고
- 자기에게 적용되는 rule을 평가하고
- 자기 Next state만 쓴다.

예:

```text
Metal Cell
→ 주변 Acid 확인
→ 자기 자신을 Corroded Metal로 변환
```

Acid thread가 Metal cell을 직접 쓰지 않는다.

다른 Cell에 쓰는 것은 이동/자리 교환/다중 생성처럼 공간 소유권 변경이 본질인 경우로 제한한다.

---

## 12. Locality

### Matter Interaction

일반적인 Matter reaction의 최대 local neighborhood는 8-neighbor다.

```text
NW  N  NE
 W [X] E
SW  S  SE
```

### Field Propagation

기본 Field propagation은 4-neighbor에서 시작한다.

```text
   N
   ↑
W ←X→ E
   ↓
   S
```

Temperature, Pressure, Electricity propagation, diffusion-style Radiation 등은 우선 가장 싼 4-neighbor baseline으로 검증한다.

필요가 확인되면 더 비싼 stencil을 benchmark한다.

### Movement

Movement는 behavior별 필요한 방향만 First-Match로 읽는다.

---

## 13. Movement / Claim / Resolve

일반 local reaction은 self-write로 처리한다.

공간 소유권이 바뀌는 경우:

- Matter movement
- two-cell swap
- 여러 source가 같은 destination을 원하는 경우
- Steam expansion 등 multi-cell creation

에는 최소한의:

```text
Propose
→ Claim / Resolve
→ Commit
```

을 사용한다.

모든 Rule을 무거운 Resolve 파이프라인으로 보내지 않는다.

---

## 14. Active Work Architecture

성능 원칙:

> **물질의 양이 아니라 변화 가능한 영역이 계산량을 결정하게 한다.**

Chunk에는 시스템별 활동 상태를 둘 수 있다.

```text
Matter Active
Thermal Active
Pressure Active
Reaction Active
```

예:

- 움직이지 않는 Stone bulk → Sleep
- 천천히 타는 Wood → Combustion/Thermal/Reaction Active
- 안정된 Water bulk 내부 → Movement Sleep 가능
- 안정된 Steam room → 단순히 Gas가 있다는 이유로 계속 Active하지 않음

Chunk는 짧은 안정 기간 후 Sleep하고, 이웃 영향이 접근하면 관련 subsystem이 Wake한다.

Subtile mask, active compaction, indirect dispatch 등의 더 복잡한 최적화는 Active Chunk baseline이 부족하다는 benchmark 증거가 있을 때만 추가한다.

---

## 15. Simulation vs Presentation / Modern FX

Gameplay 결과는 Simulation의 책임이다. **Presentation은 그 결과를 표현하는 층이지 Simulation grid의 미술적 복사본이 아니다.**

핵심 계약:

> **Cell-based simulation does not imply cell-bound presentation.**

> **Simulation grid resolution does not define final FX resolution.**

지속적인 visual effect는 read-only simulation state를 읽을 수 있고, 순간적인 효과는 semantic simulation event를 받을 수 있다.

예:

```text
Simulation State / Event
- Temperature field
- Pressure field
- COMBUSTING state
- Smoke Matter distribution
- PressureBurst
- MaterialRuptured
- CombustionStarted
- PhaseExpanded
```

Presentation은 이 state/event를 직접 화면의 cell-sized 색으로만 복사할 필요가 없다. 필요하면 별도의 extraction/smoothing 단계를 거쳐 full-resolution FX input으로 변환할 수 있다.

권장 장기 구조:

```text
GPU Simulation Core
    ↓
Authoritative Matter / Fields / Flags / Semantic Events
    ↓
Presentation Extraction
    ↓
Modern FX Layer
    ↓
Final Renderer / Composite
```

### Presentation Extraction

Presentation Extraction은 simulation truth를 렌더링 친화적인 입력으로 바꾸는 비권위적 단계다.

가능한 예:

- Temperature field sampling / interpolation
- Smoke distribution → visual density source
- COMBUSTING cells → flame emitter regions
- PressureBurst event → shockwave origin/radius input
- high-frequency rendering을 위한 temporal interpolation

이 단계는 gameplay truth를 새로 만들지 않는다.

### Modern FX Layer

Modern FX Layer는 simulation cell보다 높은 해상도와 연속 좌표에서 동작할 수 있다.

허용되는 예:

- heat haze / refraction / shimmering
- screen-space distortion
- glow / bloom / emissive response
- smooth non-cell-bound flame geometry
- sparks / embers / trails
- high-resolution particles
- continuous smoke / mist / vapor presentation
- procedural/temporal noise
- shockwave / pressure-wave distortion
- light scattering style effects
- post-processing
- camera impulse
- audio coupling

이 효과들은 128×128 debug fixture나 2048×2048 production simulation cell과 1:1 대응할 필요가 없다.

### Fire

Simulation의 Fire/Combustion truth는 예를 들어:

```text
Wood/Oil Matter
+ COMBUSTING flag
+ Temperature
+ semantic event
```

로 존재할 수 있다.

최종 Fire visual은 `MATERIAL_FIRE`라는 orange Matter를 요구하지 않으며, Wood/Oil pixel을 단순히 orange로 칠하는 것에 제한되지도 않는다. Presentation은 이를 emitter/source로 사용해 smooth flame, glow, bloom, distortion, sparks 등을 만들 수 있다.

### Smoke

`MATERIAL_SMOKE`가 실제 gameplay Matter로 존재해도 최종 smoke가 `1 Smoke cell = 1 gray square`일 필요는 없다.

```text
Smoke Matter distribution
→ Presentation Extraction
→ smooth density / procedural detail / particles
→ final visual smoke
```

Simulation Smoke는 gameplay truth를, Presentation Smoke는 시각적 품질을 담당한다.

### Heat

Temperature는 simulation field다. 최종 열 표현은:

```text
Temperature
→ sampled/smoothed presentation field
→ heat haze / refraction / distortion / glow
```

처럼 주변 화면 자체를 울렁이게 만들 수 있다.

### Authority Direction

기본 방향은 단방향이다.

```text
Simulation
→ Presentation Extraction
→ FX
```

FX texture/particle이 authoritative Temperature, combustion, Matter movement를 직접 결정하지 않는다. gameplay feedback이 필요한 경우에는 명시적인 Game Runtime command/rule 경계를 통해 별도 설계한다.

### Current M0/G4 Rendering Is Diagnostic

현재 M0/G4에서 사용하는 material palette, temperature tint, combustion coloring, pixel Smoke는 **validation/debug visualization**이다.

이 화면은 다음을 확인하기 위한 계측 도구다.

- 물질이 실제로 이동하는가
- 열이 전달되는가
- phase transition이 발생하는가
- ignition/combustion causal chain이 동작하는가

**현재 ThermalLab rendering을 Powdergame의 최종 art direction이나 최종 Fire/Smoke/Heat FX 품질 기준으로 해석하지 않는다.** Modern FX 구현은 simulation causality가 안정된 뒤 별도 presentation 단계로 의도적으로 미룬다.

---

## 16. Rewind Direction

Rewind는 실험 도구다.

현재 방향:

- 최근 10초
- 1초 granularity
- 최대 10 snapshots
- frame-by-frame reverse simulation이 아님
- 과거 상태로 돌아간 뒤 조건을 바꾸고 다시 실험 가능

GPU Production이 bit-exact deterministic하지 않으므로 과거를 명령 재실행만으로 복구하지 않는다. 실제 state snapshot을 보존한다.

keyframe + changed-chunk delta는 후보이며, 5090/32GB 환경에서 full snapshot이 충분히 싸다면 더 단순한 방식을 선택할 수 있다.

---

## 17. Deferred Developer Tool — Interaction Lab

Interaction Lab은 미래 개발자 도구 후보다.

목적:

- 완성된 새로운 Material/Rule을 입력
- 실제 GPU Production Simulation을 headless로 실행
- 기존 Matter와 대표 온도/압력/공간 조건에서 자동 실험
- 예상 밖 interaction, regression, chain reaction을 관찰

중요:

- Material을 만드는 도구가 아니다.
- 게임 runtime 기능이 아니다.
- 현재 본 게임보다 우선순위가 낮으므로 구현 보류.
- M0 architecture를 이 도구 때문에 복잡하게 만들지 않는다.

Simulation Core가 headless로 실행 가능하고 초기 상태 주입/틱 실행/결과 관찰이 가능하면 충분하다.
