# Powdergame Performance

이 문서는 Powdergame의 성능 철학과 benchmark 기준을 정의한다.

---

## 1. 성능은 제품 기능이다

Powdergame에서 성능은 단순히 FPS 숫자를 높이는 기술 작업이 아니다.

큰 세계에서 많은 Matter와 Field가 동시에 상호작용해야 게임의 핵심 판타지가 성립한다.

> **거대한 세계를 만드는 비결은 셀 하나를 똑똑하게 만드는 것이 아니라, 셀 하나를 극도로 싸게 만들고 수백만 개를 GPU에서 병렬로 돌리는 것이다.**

RTX 5090의 계산 예산은 한 Cell의 정밀도를 끝없이 높이는 데 쓰지 않는다.

절약한 예산은:

- 더 큰 world
- 더 많은 active Matter
- 더 많은 동시 reaction
- Temperature / Pressure / 미래의 전기·방사선·빛
- richer presentation
- Rewind
- 실험 가능성

에 다시 사용한다.

---

## 2. Primary Performance Target

현재 공식 성능 기준은 사용자의 Windows 개발 PC 한 대다.

핵심 기준 GPU:

```text
NVIDIA RTX 5090 32GB
```

현재 단계에서는 다른 GPU/플랫폼에 맞춘 automatic fallback, generic slow path, compatibility abstraction을 위해 production hot path를 복잡하게 만들지 않는다.

benchmark 기록에는 최소 다음을 포함한다.

- commit SHA
- build/config
- Windows version
- GPU/driver
- WorldConfig
- chunk size
- scenario
- simulation TPS / tick time
- GPU simulation time
- GPU render time
- active cell/chunk counts
- VRAM usage
- Rewind memory if applicable

---

## 3. Minimum Sufficient Physics

Powdergame의 최상위 성능 설계 원칙이다.

> **현실 공식을 그대로 풀지 않는다. 원하는 게임 현상을 만들기 위해 필요한 최소 상태와 최소 local operation만 사용한다.**

예:

| 현상 | 최소 표현 방향 |
|---|---|
| 부력/침강 | integer Density Rank 비교 + local displacement |
| 열 | `ΔT` + 최소 conductivity/heat capacity |
| 압력 | local `ΔP` + push/resistance/rupture |
| 연소 | ignition condition + Heat/Smoke |
| 전기(향후) | conductive + strength/loss frontier |
| 방사선(향후) | intensity + attenuation/blocking |
| Gameplay Light(향후) | transmit/absorb/reflect + intensity |
| 구조 파열 | pressure/force > resistance threshold |

가능하면 hot path는:

```text
bit test
→ integer compare
→ small add/subtract
→ local state change
```

수준에서 끝낸다.

정밀한 연속값이 실제 재미에 필요하면 f32를 사용한다. f32 자체를 피하는 것이 목표가 아니다.

---

## 4. Precision보다 Work를 먼저 줄인다

잘못된 우선순위:

```text
모든 셀을 계속 계산
→ 비싸다
→ 숫자 정밀도를 먼저 낮춘다
```

기본 우선순위:

```text
불필요한 work 제거
→ 불필요한 memory access 제거
→ 불필요한 pass/barrier 제거
→ 그래도 병목이면 representation 최적화
```

> **Precision은 희생하지 않고 Work를 먼저 줄인다.**

Temperature/Pressure M0 baseline은 f32다.

f16은 반드시 독립 benchmark 후 결정한다.

---

## 5. Dense State, Sparse Work

World storage는 GPU에 유리한 Dense SoA baseline을 유지한다.

```text
material_id[]
temperature[]
pressure[]
minimal_flags[]
```

하지만 계산은 dense할 필요가 없다.

> **Dense State, Sparse Work.**

각 subsystem은 변화 가능한 region만 처리하는 방향으로 진화한다.

---

## 6. Active Chunk

초기 기준 Chunk는 64×64.

system별 activity를 분리할 수 있다.

```text
Matter Active
Thermal Active
Pressure Active
Reaction Active
```

예:

```text
Stone bulk
→ all relevant movement/reaction sleep

Sand falling
→ Matter Active
→ Pressure/Thermal may sleep

Wood slowly burning
→ Thermal Active
→ Combustion/Reaction Active

Stable hot Metal cooling
→ Thermal Active
→ movement may sleep
```

Chunk는 짧은 안정 기간 후 Sleep하고 이웃 영향이 접근하면 Wake한다.

Sleep threshold 숫자는 사전에 추측으로 고정하지 않고 benchmark한다.

---

## 7. Stable Bulk / Active Frontier

Liquid/Gas 최적화의 핵심.

같은 Matter끼리 무의미하게 자리만 바꾸어 world state가 같다면 계산하지 않는다.

```text
Water ↔ Water
Steam ↔ Steam
```

안정된 거대한 Water/Gas bulk 내부는 Sleep할 수 있다.

계산 가치가 높은 곳:

- Matter ↔ EMPTY interface
- different Matter interface
- density inversion
- temperature gradient
- pressure gradient
- phase/reaction frontier

> **Matter count가 아니라 changeable frontier가 비용을 결정하도록 한다.**

예:

```text
1,000,000 stable Water cells
→ 대부분 Sleep 가능

50,000 cells in active explosion/boiling front
→ 상당수가 Active
```

후자가 더 비쌀 수 있는 구조가 목표다.

---

## 8. Locality

일반 Matter reaction은 최대 8-neighbor.

Field propagation은 4-neighbor baseline.

Movement는 필요한 방향만 First-Match.

멀리 빈 Cell을 탐색하지 않는다.

예:

```text
Liquid
below?
→ yes: stop searching
→ no: diagonal
→ no: lateral
```

> **알 필요 없는 데이터는 읽지도 않는다.**

---

## 9. Read Neighbors, Write Self

일반 interaction thread는 주변 Current state를 읽고 자기 Next state만 쓴다.

이를 통해:

- multi-writer race 감소
- atomic 감소
- global ordering 감소
- GPU massive parallelism 증가

를 노린다.

Movement/swap/spawn처럼 실제 ownership이 바뀌는 경우에만 Claim/Resolve를 사용한다.

> **논리 충돌 때문에 무거운 Resolve를 만들지 않는다. 상태 소유권 충돌만 Resolve한다.**

---

## 10. Ordered First-Match

Material의 Rule은 load/compile 단계에서 미리 정렬한다.

runtime:

```text
rule 1 matches?
→ yes: select and stop
→ no: rule 2
```

모든 rule을 검사하고 candidate list를 만든 뒤 다시 priority sort하는 구조를 피한다.

Movement에도 First-Match를 적용한다.

---

## 11. Cheap Arbitration

같은 target을 여러 Matter가 원할 때:

```text
coordinate + tick
→ cheap stateless hash
→ winner
```

같은 저비용 방식이 기본 후보다.

추가 RNG memory state는 없다.

Fixed Direction보다 방향 bias를 줄이면서도 비용을 낮게 유지하는 것이 목표다.

실제 RTX 5090에서:

- fixed direction
- cheap hash
- 필요 시 tick-alternating direction

을 benchmark할 수 있다.

---

## 12. Slow Rule Scheduling

산화/성장/부식/노화처럼 플레이어가 60Hz로 볼 필요 없는 Rule은 저빈도 tier로 실행한다.

예:

```text
FAST
MEDIUM
SLOW
VERY_SLOW
```

고정 좌표 분산 schedule로 load를 시간축에 분산할 수 있다.

하지만 모든 셀을 매 Tick launch한 뒤 15/16 thread가 바로 종료하는 가짜 최적화를 목표로 하지 않는다.

가능하면:

```text
relevant active chunk
→ relevant rate tier
→ scheduled subset
→ real work
```

를 사용한다.

다만 queue/mask 관리비가 실제 Rule 계산보다 비싸면 단순한 방법을 사용한다.

> **계산을 줄이려고 더 비싼 관리 시스템을 만들지 않는다.**

---

## 13. No Universal Progress State

모든 셀에 미래 기능을 위해:

```text
oxidation_progress
wetness_progress
growth_progress
...
```

를 넣지 않는다.

느린 변화는 가능한 한:

```text
Copper
→ Weathered Copper
→ Oxidized Copper
```

같은 Material transition으로 표현한다.

정말 gameplay에 연속값이 필요한 시스템만 나중에 비용을 측정한 뒤 별도 state를 추가한다.

> **콘텐츠 수가 늘어도 기본 Cell cost는 가능한 일정해야 한다.**

---

## 14. Compact Material Properties

Density 등 Material 공통 물성을 per-cell에 반복 저장하지 않는다.

```text
Cell
→ material_id
→ compact Material descriptor
```

시스템별로 필요한 property만 읽을 수 있도록 descriptor packing/SoA를 검토한다.

예:

```text
MovementDescriptor
- density rank
- movement class
- mobility flags

ThermalDescriptor
- thermal participation
- conductivity
- heat capacity class
```

정확한 8/16/32-bit packing은 baseline 이후 benchmark로 결정한다.

### LUT 주의

`A > B` 같은 작은 integer comparison을 없애려고 큰 lookup memory read를 추가하면 오히려 느려질 수 있다.

계산을 줄이는 것보다 memory access가 증가하지 않는지를 먼저 본다.

---

## 15. GPU Pass / Barrier Budget

Rule correctness를 이유로 다음 구조를 쉽게 추가하지 않는다.

```text
full-world reaction scan
→ barrier
full-world priority resolve
→ barrier
full-world apply
```

가능하면 local phase 하나에서 필요한 결과를 계산한다.

추가 pass/barrier는:

- invariant를 지키기 위해 필요하거나
- 실제 benchmark에서 이득이 증명될 때

만 정당화한다.

---

## 16. Candidate Optimizations

M0 baseline 이후 각각 독립적으로 측정한다.

권장 실험 순서:

1. Active Chunk skipping
2. Field-specific Active Set
3. Stable Bulk / Frontier reduction
4. Active chunk compaction / indirect dispatch
5. shared-memory tile + halo
6. Material descriptor packing
7. rule specialization
8. chunk size 32/64/128 comparison
9. f16 Temperature/Pressure experiment if actually relevant
10. subtile active mask only if chunk-level granularity is insufficient

한꺼번에 여러 최적화를 넣어 원인을 알 수 없게 하지 않는다.

---

## 17. Baseline First

M0 최초 구현은 최대한 읽기 쉬워야 한다.

```text
f32 fields
clear SoA layout
simple GPU compute
minimal synchronization
baseline metrics
```

이 baseline은 generic fallback 제품 경로가 아니라 **비교 가능한 기준**이다.

각 최적화는 baseline 대비 실제 개선을 증명해야 한다.

---

## 18. Required Performance Metrics

최소 기록:

- Render FPS
- Simulation tick time
- GPU simulation time
- GPU rendering time
- active Cell count
- active Chunk count
- subsystem timing: Matter / Thermal / Pressure / Reaction / Resolve / activity management
- VRAM usage
- Rewind storage

전체 TPS만 보고 어디가 병목인지 모르는 상태를 피한다.

---

## 19. Benchmark Scenarios

M0부터 반복 가능한 대표 시나리오를 만든다.

### Sand Fall

- Powder movement
- collision/arbitration

### Water Flow

- Liquid movement
- density displacement
- stable bulk

### Fire / Heat

- Thermal propagation
- combustion
- Smoke

### Pressure Burst

- Steam expansion
- pressure
- rupture/vent

### Heavy Mixed World

- 여러 subsystem 동시 active
- worst-case에 가까운 실제 플레이 workload

각 scenario는 가능하면 자동으로 초기 상태를 만들 수 있어야 한다.

---

## 20. M0 Performance Gate

M0에는 아직 임의의 숫자 목표를 박지 않는다.

M0 통과를 위해 필요한 것은:

- benchmark harness 존재
- subsystem cost가 분리되어 보임
- 대표 scenario 반복 가능
- bottleneck 파악 가능
- 결과가 commit/hardware/config와 함께 기록됨
- 2048×2048 reference world를 실제로 측정함

M0 baseline을 얻은 뒤 M1부터 숫자 performance budget을 설정한다.

---

## 21. 최종 성능 원칙

Powdergame의 성능 최적화 질문은 항상 다음 순서로 한다.

1. 이 계산이 게임 결과에 정말 필요한가?
2. 계산하지 않고 같은 gameplay 의미를 만들 수 있는가?
3. local comparison/rank/bit로 표현할 수 있는가?
4. 안정된 영역을 Sleep할 수 있는가?
5. 필요한 데이터만 읽는가?
6. 병렬화 가능한가?
7. 실제 benchmark에서 병목인가?
8. 병목이라면 더 복잡한 최적화가 실제로 이기는가?

> **싸구려 규칙을 수백만 개 동시에 돌려서 비싼 세계를 만든다.**
