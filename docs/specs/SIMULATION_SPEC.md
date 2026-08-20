# Powdergame Simulation Specification

이 문서는 현재 구현이 따라야 하는 Simulation Core 계약을 정의한다.

---

## 1. 목표

Powdergame의 Simulation은 현실을 정밀하게 복제하는 solver가 아니다.

목표는:

- 큰 finite world
- 수백만 Cell
- 수많은 Matter/Field 상호작용
- 실시간에 가까운 반응
- 아주 낮은 per-cell 비용
- GPU 병렬성
- 이해 가능한 인과관계
- 재미있는 emergent chain

을 동시에 달성하는 것이다.

---

## 2. Reference Configuration

M0 기준:

```text
World Size: 2048 × 2048
Cells: 4,194,304
Initial Chunk: 64 × 64
Chunks: 32 × 32 = 1024
Simulation Target: 60 TPS
Primary Hardware: Windows + RTX 5090
Production Backend: wgpu / DX12
```

World size는 `WorldConfig`로 관리한다.

---

## 3. Cell Model

### 3.1 Single Matter Occupancy

각 Cell은 한 Tick에 Matter를 최대 하나만 가진다.

```text
material_id[cell]
```

셀 내부 혼합물은 기본 모델이 아니다.

### 3.2 Unit Matter Quantity

Matter가 있으면 한 단위다.

`0.2 Water`, `30% Oil` 같은 per-cell amount는 기본 셀 모델에 넣지 않는다.

### 3.3 Core State Direction

M0 baseline은 개념적으로 다음 Dense SoA를 사용한다.

```text
material_id[]
temperature[]
pressure[]
minimal_flags[]
```

정확한 bit width/packing은 benchmark 후 결정할 수 있다.

Temperature/Pressure는 Matter가 아니다.

### 3.4 EMPTY

`EMPTY`는 Matter가 아니다.

- 물성이 없다.
- Density가 없다.
- 현재 production runtime에서는 열 또는 압력을 전달하는 숨은 매질로 동작하지 않는다.
- ADR-0005는 향후 ordinary EMPTY 공간의 Atmosphere와 Vacuum을 구분하는 별도 `air_mass`/`air_energy` Environment Field를 채택했다.
- 그 Air는 Material ID, Density 대상, palette Matter 또는 occupied Cell 아래의 두 번째 Matter가 아니다.
- Steam과 Smoke는 계속 explicit GAS Matter다.

TE-0은 runtime을 바꾸지 않는다. Environment가 TE-1 이후 구현되기 전까지 Dense Field 배열의 기존 값은 `material_id == EMPTY` Cell에서 물리적 Air로 사용하지 않는다. 이후 계약은 `THERMAL_ENVIRONMENT_SPEC.md`를 따른다.

---

## 4. Boundary

- World는 finite하다.
- 외곽 BLOCK은 실제 Matter/구조로 표현하고 ERASE 가능하다.
- 외곽 BLOCK 뒤에 보이지 않는 collision wall을 추가하지 않는다.
- Matter가 simulation domain 밖으로 이동하면 Void로 소멸한다.
- boundary를 지운다고 World가 확장되지는 않는다.

---

## 5. Production Authority

GPU Production Simulation이 실제 게임의 authoritative simulation이다.

CPU Reference는:

- 작은 테스트
- algorithm explanation
- debug
- semantic comparison

용도다.

CPU/GPU exact equality를 요구하지 않는다.

---

## 6. Tick Philosophy

### 6.1 60 TPS target

기본 목표는 60 simulation TPS다.

Rendering은 독립적으로 더 높은 FPS를 낼 수 있다.

### 6.2 Loose Causal Phases

모든 인과를 같은 Tick에서 즉시 소비시키기 위해 강한 barrier를 연속 사용하지 않는다.

가능하면 subsystem은 Tick 시작의 Current state를 기반으로 독립 계산하고, 필요한 무결성/인과 barrier만 둔다.

예:

```text
Tick N
Wood temperature increases

Tick N+1
Ignition rule observes new temperature
→ combustion starts
```

이 정도 1 Tick 지연은 자연스럽다면 허용한다.

### 6.3 Global Slowdown under overload

심한 workload에서 60 TPS를 유지하지 못할 경우 카메라 밖 physics를 줄이지 않는다.

Simulation 전체가 느려질 수 있다. Rendering은 가능한 부드럽게 유지한다.

카메라가 simulation 결과를 바꾸지 않는다.

---

## 7. Local Neighborhood

### Matter interaction

일반 Matter interaction은 최대 8-neighbor.

```text
NW N NE
 W X E
SW S SE
```

### Field propagation baseline

Temperature/Pressure 등의 기본 propagation은 4-neighbor부터 시작한다.

```text
   N
   ↑
W ←X→ E
   ↓
   S
```

4-neighbor가 게임적으로 부족하다는 증거가 생기면 8-neighbor 등 대안을 benchmark한다.

### Movement

Movement는 behavior-specific stencil을 사용하고 필요한 방향만 First-Match로 읽는다.

---

## 8. Read Neighbors, Write Self

일반 Rule의 기본 병렬 패턴:

```text
READ
- self current state
- required neighbors current state

COMPUTE
- cheap local rule

WRITE
- self next state only
```

다른 Cell을 직접 수정하는 일반 Rule은 피한다.

예:

```text
Dirt cell
→ neighbor Water 발견
→ Dirt 자신이 Wet Dirt로 변환
```

Water가 Dirt를 직접 write하지 않는다.

이 원칙은 Matter뿐 아니라 Temperature/Pressure 같은 local Field update에도 기본 적용한다.

---

## 9. Spatial Ownership Change

다음은 self-write만으로 충분하지 않다.

- movement into another Cell
- two Matter swap
- multiple source → same destination
- multi-cell spawn
- phase expansion yield > 1

이 경우에만 최소한의:

```text
Propose
→ Claim / Resolve
→ Commit
```

을 사용한다.

Resolve는 Rule 우선순위를 위해 모든 계산에 추가하는 것이 아니라 **One Cell = One Matter**를 보존하기 위해 필요한 곳에만 쓴다.

---

## 10. Local Arbitration

여러 Matter가 같은 destination을 원할 때 값비싼 global ordering을 만들지 않는다.

기본 방향:

```text
stateless cheap arbitration
= coordinate + tick 기반 작은 integer hash
```

목적은 게임에 RNG 기능을 넣는 것이 아니라 고정 방향 편향을 줄이는 것이다.

조건:

- per-cell RNG state 없음
- CPU ordering 없음
- actual collision이 있을 때만 사용 가능
- 구현은 fixed direction baseline과 benchmark 가능

---

## 11. Movement Families

M0 기본 family:

- STATIC
- POWDER
- LIQUID
- GAS

### STATIC

일반 gravity/density displacement로 움직이지 않는다.

Pressure rupture, 특수 Rule 등의 별도 시스템은 영향을 줄 수 있다.

### POWDER

예시 우선순위:

```text
down
→ down-diagonal
→ stop
```

가능한 위치를 찾으면 즉시 종료한다.

### LIQUID

예시:

```text
down
→ down-diagonal
→ lateral
```

한 Tick에 먼 빈 공간을 scan하지 않는다.

여러 Tick의 local movement가 전체 흐름을 만든다.

### GAS

높은 mobility를 가진 Matter다.

기본적으로 위쪽/대각/측면 등 Gas behavior stencil을 사용하지만, **Gas는 반드시 매 Tick 위치를 바꾸는 입자가 아니다.**

상태적으로 의미 있는 movement/interface/gradient가 없으면 stable bulk로 Sleep할 수 있다.

---

## 12. Density / Buoyancy

부력 solver를 구현하지 않는다.

Density는 작은 integer rank로 취급한다.

필요한 핵심 관계:

```text
A > B
A == B
A < B
```

### Local Density Displacement

움직이는 Matter A가 destination B를 볼 때:

```text
B == EMPTY
→ normal move

B non-movable
→ stop

B movable
→ density rank comparison
→ appropriate local swap candidate
```

이를 반복하여 침강/부력/층분리를 만든다.

> **부력을 계산하지 않는다. 정렬한다.**

Density는 per-cell 값이 아니라 Material property다.

---

## 13. Temperature

### Baseline state

`f32` temperature를 baseline으로 한다.

f16은 독립적인 optimization experiment다.

### Physical meaning

Temperature는 Matter의 thermal state를 표현한다.

현재 runtime에서는 EMPTY를 통해 자동으로 열이 전달되지 않는다. ADR-0005가 채택한 future Environment thermal path는 Matter temperature와 별도 Air mass/energy를 사용하며, 구현 전 계약은 `THERMAL_ENVIRONMENT_SPEC.md`에 있다.

### Cheap transfer philosophy

Temperature는 Density처럼 단순 rank만으로 충분하지 않다.

최소 모델은 다음을 고려할 수 있다.

- direction from `ΔT`
- cheap conductivity property
- cheap heat-capacity property

정밀 현실 열역학을 목표로 하지 않는다.

개념:

```text
ΔT가 의미 없음
→ no work / equilibrium

ΔT가 의미 있음
→ cheap local transfer
→ self next temperature
```

### Thermal deadband

아주 작은 차이를 영원히 계산하지 않도록 gameplay에 영향이 없는 수준의 `thermal_deadband`를 둘 수 있다.

이 값은 benchmark와 visual/gameplay validation으로 정한다.

---

## 14. Phase Transition

Material은 온도/압력/주변 조건에 따라 다른 Matter로 변할 수 있다.

M0 대표:

```text
Ice ↔ Water ↔ Steam
```

현실의 정확한 0°C/100°C를 따라야 하는 계약은 없다.

### Transition Yield

상변화는 반드시 1:1이 아니다.

예:

```text
1 Water
→ up to N Steam cells
```

정확한 yield는 data-driven content rule이다.

공간이 부족하면:

- 일부만 생성
- 나머지를 defer
- unresolved expansion을 Pressure로 전환

등의 정책을 Rule/System으로 정의할 수 있다.

M0에서 최소 chain은 `blocked phase expansion → pressure`를 증명한다.

---

## 15. Pressure

M0 Pressure는 정밀한 compressible fluid solver가 아니다.

### Baseline

- scalar `pressure[]`
- f32 baseline
- 4-neighbor local propagation
- direction은 별도 velocity vector를 저장하지 않고 local pressure difference에서 유도

### Core causal chain

```text
phase expansion blocked
→ pressure generated
→ local pressure difference propagates
→ movable Matter may be pushed
→ resistant Matter holds
→ pressure > rupture threshold
→ structure ruptures
→ opening allows venting
```

### Environment pressure boundary

현재 runtime에서 EMPTY 자체는 pressure medium이 아니며, 기존 `pressure[]`는 Liquid/Gas와 phase-confinement/rupture를 위한 gameplay gauge overpressure다.

ADR-0005는 future Environment Air의 absolute-like background pressure를 별도 derived state로 정의한다. 두 pressure를 occupancy와 무관하게 더하지 않는다. Atmosphere/Vacuum/structure face differential coupling은 TE-5 전까지 production semantics를 변경하지 않는다.

### Approximate behavior

정확한 mass/energy conservation보다 안정적이고 상식적인 결과를 우선한다.

---

## 16. Combustion / Fire

Fire는 M0에서 단순한 주황색 permanent Matter로 정의하지 않는다.

기본 개념:

```text
Fuel Matter
+ sufficient Temperature / ignition condition
→ combustion state/phenomenon
→ Heat
→ Smoke
→ visual flame
```

Wood와 Oil은 M0에서 같은 공통 combustion grammar를 사용하는 서로 다른 Matter 예시다.

Oxygen은 현실에 필요하다는 이유만으로 M0 필수 조건에 넣지 않는다.

나중에 Oxygen/oxidizer manipulation이 실제 재미를 만든다면 일반화된 oxidizer rule로 추가할 수 있다.

---

## 17. Slow Rules

산화, 부식, 성장, 노화 등 느린 현상을 60Hz로 모두 검사하지 않는다.

공통 update-rate tier를 둘 수 있다.

```text
FAST
MEDIUM
SLOW
VERY_SLOW
```

느린 Rule은 좌표 기반 고정 분산 스케줄 등으로 시간축에 work를 분산한다.

### 중요한 금지사항

매 Tick 모든 셀을 dispatch한 뒤 `이번 tick에 내 차례인가?`를 확인하고 대부분 종료하는 방식은 목표가 아니다.

가능하면:

```text
relevant active chunks
→ relevant rate tier
→ scheduled subset
→ actual rule evaluation
```

으로 실제 work 자체를 줄인다.

다만 관리 시스템이 절약보다 비싸지면 단순한 방식이 우선이다.

---

## 18. No Universal Progress Field

산화/젖음/부식 등의 미래 기능을 위해 모든 Cell에 universal progress 값을 미리 넣지 않는다.

느린 단계 변화는 기본적으로:

```text
Copper
→ Weathered Copper
→ Oxidized Copper
```

처럼 Material transition으로 표현할 수 있다.

정말 연속적인 축적량이 gameplay에 필요한 특정 시스템이 등장했을 때만 별도 state 설계를 검토한다.

> 콘텐츠 수가 증가해도 기본 Cell cost가 비례해서 증가하지 않아야 한다.

---

## 19. Active / Sleep

### Principle

Sleep은 physics fidelity를 낮추는 LOD가 아니다.

> **계산해도 상태가 바뀌지 않는 work만 제거한다.**

### Chunk sleep

Chunk가 짧은 안정 기간 동안 의미 있는 변화가 없으면 Sleep 후보가 된다.

정확한 Tick threshold는 benchmark로 정한다.

### Wake

이웃에서 다음과 같은 영향이 접근하면 관련 subsystem을 Wake한다.

- Matter movement/interface
- Temperature change
- Pressure change
- Reaction condition
- combustion/phase event

### Slow but active

변화가 작다고 Sleep하지 않는다.

천천히 타는 Wood는 실제 변화가 지속되므로 관련 Thermal/Combustion/Reaction subsystem이 Active다.

---

## 20. Stable Liquid/Gas Bulk

동일한 안정 Matter끼리 위치를 바꿔도 world state가 같다면 그 movement를 계산할 이유가 없다.

예:

```text
Steam ↔ Steam
Water ↔ Water
```

안정된 bulk 내부는 Sleep할 수 있다.

실제 work는 가능한 한 다음에 집중한다.

- EMPTY interface
- different-Matter interface
- density inversion
- temperature gradient
- pressure gradient
- active reaction frontier

즉:

> **Matter count가 아니라 changeable frontier가 simulation cost를 결정하게 한다.**

Subtile active mask는 미래 benchmark 후보일 뿐 M0 기본 요구가 아니다.

---

## 21. Approximate Conservation

전 세계 energy/mass bookkeeping을 정확히 맞추지 않는다.

싸게 가능한 local conservation은 사용할 수 있다.

예:

```text
A loses heat
B gains similar heat
```

그러나:

- Magic material이 Heat를 생성
- Explosion이 Heat/Pressure source를 생성
- Void가 Energy를 소멸

하는 게임 고유 Rule도 허용한다.

> **로컬에서는 납득 가능하게, 글로벌에서는 회계하지 않는다.**

---

## 22. Future Transferable Systems

M0 구현 범위는 아니지만 현재 물리 철학을 그대로 확장할 수 있다.

### Electricity

```text
conductive?
+ local electrical strength
+ material loss/resistance
→ frontier propagation
→ strength decays unless source replenishes
```

### Radiation

```text
intensity
→ blocking/attenuation rank
→ remaining intensity propagated locally
```

### Gameplay Light

Presentation light와 Simulation light를 분리한다.

Gameplay light가 필요한 경우:

```text
intensity
+ transparent/absorb/reflect properties
→ next beam/local state
```

### Explosion

복잡한 전용 폭발 solver보다:

```text
Explosion event
→ inject Heat
→ inject Pressure
→ Presentation event
```

후 기존 Temperature/Pressure/Rupture system이 결과를 만드는 방향을 선호한다.

---

## 23. Invariants

GPU의 비정확한 실행 순서나 float 근사는 허용하지만 다음은 허용하지 않는다.

- 한 Cell에 Matter 두 개
- state corruption
- 설명되지 않는 중복/소멸 bug
- NaN/Infinity runaway
- out-of-bounds memory access
- race로 algorithm이 근본적으로 깨지는 상태

이 Invariant는 성능 최적화보다 우선한다.
