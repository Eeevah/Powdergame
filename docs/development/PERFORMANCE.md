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

- HEAD commit SHA, clean/dirty state, evidence schema/run ID
- build/config
- Windows version
- GPU/driver
- WorldConfig
- chunk size
- scenario
- measurement mode, synchronization policy, prewarm, trial and tick ranges
- simulation TPS / tick time
- GPU simulation time
- GPU render time
- active cell/chunk counts
- application-tracked requested persistent GPU buffer bytes and explicit exclusions
- driver/OS-reported resident VRAM, when separately available
- raw sample count/path and percentile aggregation method
- Rewind memory if applicable

Dirty-worktree calibration은 local validation에는 사용할 수 있지만 immutable official baseline으로 승격하지 않는다. Official evidence는 결과가 어떤 source state에서 생성되었는지 재현할 수 있어야 한다.

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
- raw per-tick timestamp/pass/group samples and sample identity
- raw activity census values: every `cell_activity`, `chunk_activity`, and `chunk_state` element, census tick, and bit/state definitions needed for an independent recount
- percentile method: grouped percentile은 pass percentile의 합이 아니라 per-tick group sum의 percentile
- timing mode and synchronization policy: batch production throughput과 synchronized profiling을 구분
- setup timing fence: reset/fixture `Queue::write_buffer`는 timer 전에 명시적으로 submit하고 completion wait; `device.poll(Wait)` 단독 사용 금지
- profiling overhead controls: batch unprofiled, per-tick synchronized unprofiled, per-tick synchronized profiled
- application-tracked requested persistent GPU buffer bytes with exact scope/exclusions
- driver/OS-reported resident VRAM when available; tracked bytes와 동일한 값으로 취급하지 않음
- run receipt: exact argv/cwd, raw stdout/stderr, exit code, isolated build log, source-input snapshot and full dirty diff hashes, executed binary hash, artifact hashes, and one matching run ID
- evidence publication: all raw files are fully staged and synchronized before the aggregate summary is exposed; existing evidence paths are not overwritten. This is not a cross-file crash/power-loss atomicity claim.
- review packet delivery: attach the ZIP and its sibling `PACKAGE_SHA256.txt` together; generated target inventories must reject empty paths and assert their row count against the captured path inputs
- Rewind storage

전체 TPS만 보고 어디가 병목인지 모르는 상태를 피한다.

Activity reason census의 Matter / Thermal / Pressure / Reaction category는 서로 겹칠 수 있으므로 partition처럼 합산하지 않는다. Mode B의 GPU envelope/pass breakdown은 per-tick synchronization과 readback을 포함하는 diagnostic path이며 Mode A sustained wall time을 대체하지 않는다.

---

## 19. Benchmark Scenarios

M0부터 반복 가능한 대표 시나리오를 만든다. 아래 다섯 official G8-B scenario는 shared deterministic fixture, Windows inspection Gallery, headless `--scenario` selection까지 구현 candidate가 존재한다. Scenario 1 Sand Fall과 Scenario 2 Water Flow는 사용자 승인되었고, Water의 automatic `NEEDS_HUMAN_REVIEW`는 알려진 M0 liquid free-surface 후속 과제와 함께 그대로 유지된다. Scenario 3 Fire / Heat candidate가 진행 중이며 Scenario 4~5는 **PENDING / NOT YET USER ACCEPTED**다. G8-B 전체 상태는 **USER ACCEPTANCE PENDING / NOT CLOSED**이며, 아직 G8-C official matrix 결과가 아니다.

### Sand Fall

- Powder movement
- collision/arbitration
- **USER ACCEPTED (2026-08-17)**: Sand가 완전히 정착하고 모든 chunk가 sleep에 들어가는 것이 성공이다. 지속 activity를 만들기 위한 source/geometry/sleep retuning은 하지 않는다.

### Water Flow

- Liquid movement
- density displacement
- stable bulk
- **USER ACCEPTED WITH KNOWN FOLLOW-UP (2026-08-17)**: first candidate는 immutable/superseded다. Source `5af031f` remediation run은 automatic `NEEDS_HUMAN_REVIEW`를 유지하며, outer-basin Water `0 / 0`, conservation/movement/destination/reset을 기록했다. 알려진 후속 과제는 M0 local-liquid free-surface 소수 셀의 지속 재배열이며 production-physics defect 증거는 없다.

### Fire / Heat

- Thermal propagation
- combustion
- Smoke
- **HARNESS CANDIDATE IN PROGRESS / NOT YET USER ACCEPTED**: unchanged finite fixture와 production physics에서 combustion, Smoke, phase inventory, reaction termination, separate Thermal tail, exact reset을 관찰한다.

### Pressure Burst

- Steam expansion
- pressure
- rupture/vent
- **PENDING / NOT YET USER ACCEPTED**

### Heavy Mixed World

- 여러 subsystem 동시 active
- worst-case에 가까운 실제 플레이 workload
- **PENDING / NOT YET USER ACCEPTED**

각 scenario는 가능하면 자동으로 초기 상태를 만들 수 있어야 한다.

### Shared staging contract

- `apps/scenarios`의 `powdergame-scenarios` crate가 `ScenarioId`, pure CPU `ScenarioFixture`, `validate_scenario_config`, `reset_and_stage_scenario`를 소유한다.
- official 5종은 동일한 2048×2048×64 headless default에서 자동 staging 가능하다. 256×256 이상의 rectangular config도 허용하지만 official matrix config를 바꾸었다는 뜻은 아니다.
- `reset_and_stage_scenario`는 production `Simulation`을 reset하고 Material/Temperature/Pressure/Flags의 Current와 Next, 그리고 authored `chunk_edit_wake`를 같은 tick-0 image로 staging한 뒤 transfer completion을 기다린다.
- benchmark는 모든 prewarm, production-throughput trial, profiled trial, overhead control 직전에 이 shared reset/stage 경로를 사용한다.
- crate는 production pass graph, shader, physics Rule, Material registry를 변경하지 않는다. fixture-specific code는 authored initial state만 만든다.

### Sixth Gallery regression fixture

`active-sleep-g7`은 official G8-B matrix의 여섯 번째 workload가 아니다. 기존 G7 Activity/Sleep observatory의 256×256×64 geometry와 edit-wake snapshot을 정확히 재사용하는 회귀 fixture이며, 다른 config는 pre-GPU validation에서 거부한다.

### Windows inspection and headless timing separation

- Windows `--benchmark-gallery` / `run_g8_benchmark_gallery.bat`은 1~6 선택, play/pause, one-tick step, x1/x4/x16, pristine reset을 제공하고 항상 paused 상태에서 시작한다.
- Gallery의 rendering, HUD, wall-clock TPS, sampled activity census는 시각적·진단용 surface다. bounded census도 out-of-band readback이며 official timed loop에 들어가지 않는다.
- headless harness는 `--scenario calibration|sand-fall|water-flow|fire-heat|pressure-burst|heavy-mixed-world|active-sleep-g7`을 받는다. Gallery crate/window/renderer를 통과하지 않는다.
- 기본 `calibration`은 기존 `powdergame-g8a-v5`, `g8a-*`, `target/calibration_report.csv` 계약을 유지한다. shared fixture는 같은 CSV column shape에서 `powdergame-g8b-fixture-v1`, `g8b-<slug>-*`, `target/<slug>_report.csv`로 identity와 기본 output을 분리한다.
- 이 구현은 scenario 반복 가능성과 관찰 surface를 제공할 뿐이다. official G8-C throughput/render/coexistence matrix, bottleneck 결정, 숫자 budget은 별도 단계다.

### Scenario Experiment Evidence Harness

`feature/m0-g8b-scenario-suite` checkpoint `e77d102`에서 사용자 승인된 Sand Fall을 대상으로, `feature/g8b-experiment-harness-v0`가 lifecycle evidence runner를 제공한다. 이 runner는 performance benchmark가 아니라 accepted fixture의 실제 낙하, Matter/field integrity, sleep convergence, post-sleep stability, exact reset을 기록하는 out-of-band experiment다.

```bat
run_experiment.bat sand-fall
```

- Sand v0 contract; 256×256×64; production `Simulation::tick`; shared pristine staging
- diagnostic simulation tick과 sample sequence를 별도 기록
- all-sleep 조건을 3회 연속 sample에서 확인하고 이후 180 production tick 동안 change/wake를 매 tick 검사
- renderer와 같은 draw path에서 6–10 semantic full frames를 캡처하고 crop/contact sheet는 full PNG에서만 파생
- `C:\Users\mdkap\source\Powdergame-artifacts\<unique-run-id>` create-new/no-overwrite publication
- raw stdout/stderr, samples/events JSONL, analysis/frame manifests, raw RGBA, PNG, report, prompt, packet, SHA-256 inventory 보존
- `EXPERIMENT_RECEIPT.json` final write; receipt absence = incomplete preserved run; failed ID reuse 금지
- 모든 generated artifact는 Git 밖에 유지

Automatic `PASS`는 actual fall, Matter conservation, zero invalid Material, zero non-finite field, sleep before max, zero post-sleep change/wake, exact reset의 일곱 hard predicate가 모두 참이라는 run-local 판정이다. Scenario 2–5 user acceptance, G8-B closure, official G8-C performance, bottleneck decision을 의미하지 않는다.

Harness pilot은 experiment source `9e1fdac44aa14a546c7fe5ad6ceba49e71777eb5`에서 automatic `PASS`, Harness review output `APPROVED`로 검증되었다. 이는 performance benchmark나 G8-C evidence가 아니며, 이후 docs-only closure commit과 experiment source provenance를 구분한다. 전체 lifecycle, 수치, artifact hash, review 경계는 `docs/evidence/G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md`를 따른다.

#### Water Flow v2 accepted remediation evidence

Water Flow는 같은 coordinator/provenance/screenshot/report/contact-sheet/hash/receipt-last 기반 위에 scenario-specific analyzer만 분리한다. 첫 candidate `g8b-water-flow-v0-20260817T100732645294Z-f7ee7959`는 immutable/superseded이며 automatic `NEEDS_HUMAN_REVIEW`, human `FIX REQUIRED — fixture_representativeness_issue`를 기록했다. Remediation은 좌우 외벽 시작만 `y=90 → 14`로 올리고 Water/Oil·내부 channel·production physics를 보존했다. Source `5af031f`의 run `g8b-water-flow-v0-20260817T110906547252Z-8b808e66`는 automatic `NEEDS_HUMAN_REVIEW`를 유지한 채 human `ACCEPTED WITH KNOWN FOLLOW-UP`로 승인되었다. unique Run ID, create-new/no-overwrite, failed-run preservation, receipt-last 정책을 유지하며 generated artifact는 Git 밖에 둔다.

Remediation telemetry/analysis/report/receipt는 `powdergame-experiment-*-v2`를 사용하고 manifest v1과 shared frame manifest v0를 유지한다. 기존 아홉 predicate에 `water_outside_outer_basin_cells`를 추가한다. 이 predicate는 `[18,238) × [14,230)` 밖의 Water가 모든 non-reset sample에서 0일 때만 pass한다. 모든 열 개 status가 pass일 때만 `PASS`, 하나라도 fail이면 `FAIL`, unknown이 남으면 `NEEDS_HUMAN_REVIEW`다. Eight-sample stable plateau는 terminal observation이 될 수 있지만 finite fixture의 all-sleep proof를 대신해 `stable_bulk_before_max`를 pass로 만들지 않는다. all-sleep 실패 시 마지막 active cell은 cardinal 4-neighbor 기준 Water/Oil interface, Water/EMPTY surface, other로 분류하며 정책 완화에는 사용하지 않는다.

256×256×64의 remediated tick-0 census는 Water 15,244 / Oil 2,240 / Stone 8,104 / Boundary Block 1,020 / Empty 38,928이다. destination은 `[18,238) × [200,230)` 안의 tick-0 EMPTY 6,216-cell mask이며 staging outcome이 아니라 observation region이다. FAST 기록은 workspace fmt/check PASS, scenarios 7/7, shared GPU reset 1/1, bounded Water GPU 1/1, Windows Sand/Water experiment 16/16, Python 23/23다. Single FULL workspace test/clippy/diff checkpoint와 60-frame RTX 5090/DX12 Gallery release smoke도 PASS다. Candidate source SHA는 `5af031f1a04af866127616d4f1b0faa6c85e4d8e`; human acceptance는 automatic verdict를 소급 변경하지 않으며 G8-B를 닫지 않는다.

#### Fire / Heat Harness candidate

Fire / Heat는 같은 coordinator와 receipt-last artifact 구조를 재사용하지만 Sand/Water 판정으로 fall through하지 않는 별도 analyzer를 사용한다. Tick 0의 authored combustion flags에는 Wood 544, Oil 272뿐 아니라 hot-seed/Stone-column overlap 68 cells가 포함되므로 production tick 이후 Wood와 Oil의 flame/fuel-progress signal을 모두 관찰해야 genuine combustion으로 기록한다. Whole-world all-sleep은 요구하지 않는다. Reaction zero를 세 diagnostic sample로 확인한 뒤 180 production ticks의 Thermal tail을 별도로 기록하며, tail이 남아 있다는 사실만으로 실패시키지 않는다. Fire candidate의 exact fixture, lifecycle, predicates, artifact 경계는 `docs/evidence/G8_B_FIRE_HEAT_HARNESS_CANDIDATE_2026-08-17.md`를 따른다.

---

## 20. M0 Performance Gate

M0에는 아직 임의의 숫자 목표를 박지 않는다.

M0 통과를 위해 필요한 것은:

- benchmark harness 존재
- subsystem cost가 분리되어 보임
- 대표 scenario 반복 가능
- bottleneck 파악 가능
- 결과가 HEAD SHA, clean/dirty state, evidence ID, hardware, config와 함께 기록됨
- 공식 candidate는 attached clean source SHA에서만 생성하고, source snapshot, executed binary, command/log/exit code, CSV run ID를 최종 receipt에서 해시로 연결함
- dirty run 또는 receipt 없는 run은 incomplete/non-canonical capture로만 보존함
- aggregate census는 동일 tick의 one-row-per-cell raw cell CSV와 one-row-per-chunk raw chunk CSV로 독립 재집계 가능함
- capture ZIP SHA-256은 ZIP 외부의 sibling `PACKAGE_SHA256.txt`에 기록함
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
