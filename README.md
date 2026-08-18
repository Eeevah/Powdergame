# Powdergame

> **상상력을 시뮬레이션하는 세계 창조 샌드박스**
>
> **플레이어에게 물질을 주는 게임이 아니라, 우주를 발명할 수 있는 문법을 주는 게임.**

Powdergame은 Doodle God의 **조합·발견·세계 창조**와 DAN-BALL Powder Game의 **즉각적인 공간 상호작용·창발성**을 하나의 실제 샌드박스 세계 안에서 결합하는 프로젝트다.

핵심은 현실을 그대로 복제하는 것이 아니다.

> **현실의 자연현상은 참고자료다. Powdergame 안에서 원인과 결과가 이해되고, 서로 영향을 주며, 재미있는 연쇄작용을 만든다면 가상의 물질과 가상의 물리도 완전히 유효하다.**

## 현재 단계

**M0 — First World 구현 진행 중**

G0-G7은 닫혔고 G8 Performance Evidence가 진행 중이다. G8-A v5는 clean source `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`의 official capture와 독립 검증을 완료한 verified evidence candidate다. 같은 SHA의 user visual validation은 아직 pending이며, 기존 v4 timing CSV는 source/binary 실행 연결과 raw census가 없는 historical data로만 보존한다.

`integration/canonical-recovery`는 이 검증 구현선과 최신 research/Foundation Material Wiki를 하나의 tested local integration line으로 결합했다. 그 위의 `feature/m0-g8b-scenario-suite` checkpoint `e77d102`에서 G8-B의 다섯 official fixture와 여섯 번째 G7 Active/Sleep 회귀 fixture를 같은 shared staging API로 제공하는 구현 candidate가 만들어졌다. Scenario 1 Sand Fall은 사용자가 승인했고, 그 immutable Harness pilot은 experiment source `9e1fdac44aa14a546c7fe5ad6ceba49e71777eb5`에서 automatic verdict **PASS**와 Harness review output **APPROVED**를 기록했다. Scenario 2 Water Flow는 source `5af031f1a04af866127616d4f1b0faa6c85e4d8e`의 remediation candidate에서 automatic **NEEDS_HUMAN_REVIEW**를 유지한 채 human **ACCEPTED WITH KNOWN FOLLOW-UP**로 승인되었다. 알려진 후속 과제는 M0 local-liquid free-surface의 소수 셀 지속 재배열이며 production-physics defect 증거는 없다. Scenario 3 Fire / Heat는 tested source `1635fdb9f562192123c92846e137b125c684ede9`의 run `g8b-fire-heat-v0-20260817T133938546075Z-0e6aa901`에서 automatic **PASS**와 독립 재계산 불일치 0을 기록했고 **USER ACCEPTED**로 승인되었다. Scenario 4 Pressure Burst는 clean source `43e19d0f3b43aa0c15bf31e98f6401ba5f885170`의 run `g8b-pressure-burst-v0-20260818T101046792957Z-17158748`에서 automatic **NEEDS_HUMAN_REVIEW**를 유지한 채 human **USER ACCEPTED WITH KNOWN FOLLOW-UP**로 승인되었다. Pressure-caused opening이 combustion보다 먼저 발생했고 Pressure relief/integrity/reset 계약이 통과했다. 알려진 후속 과제는 top-seam-only opening, small persistent vent plume, broad terminal Pressure activity와 G8-C workload-cost 측정이며 production-physics defect 증거는 없다. **Scenarios 1–4 are USER ACCEPTED**; Scenario 5 Heavy Mixed World가 **NEXT**이므로 **G8-B 전체는 NOT CLOSED**이고 G8-C official matrix는 별도 pending gate다.

## 현재 공식 개발 경로

```text
Platform:      Windows
Language:      Rust
Window/Input:  winit
GPU API:       wgpu
Backend:       DX12
Primary GPU:   NVIDIA RTX 5090
World:         finite, chunked dense grid
Simulation:    GPU authoritative
Target:        60 simulation TPS baseline
```

현재는 Browser/macOS/범용 GPU 호환을 위해 구조와 성능을 희생하지 않는다. 이 프로젝트는 우선 사용자의 Windows + RTX 5090 환경에서 **큰 세계와 많은 상호작용을 가능한 한 싸게 병렬 실행하는 것**을 최우선으로 한다.

## G8-B Benchmark Scenario Gallery

Windows inspection Gallery:

```bat
run_g8_benchmark_gallery.bat
```

Gallery slot `1`~`5`는 Sand Fall, Water Flow, Fire / Heat, Pressure Burst, Heavy Mixed World의 official G8-B fixture다. Slot `6`은 official matrix workload가 아니라 기존 G7 Active/Sleep geometry와 edit-wake 의미를 보존하는 회귀 fixture다. Gallery는 paused 상태로 시작하며 `1-6` scenario 선택, `SPACE` play/pause, `N` one tick, `F` x1/x4/x16, `R` pristine reset, `ESC` quit을 제공한다.

Scenario 1 Sand Fall의 승인 계약은 완전 정착과 모든 chunk의 sleep 수렴을 성공으로 보는 것이다. 계속 움직이는 화면을 만들기 위해 source/geometry/sleep behavior를 retune하지 않는다. Scenario 2 Water Flow는 외벽 remediation 뒤 basin 밖 Water `0 / 0`, conservation/movement/destination/reset을 기록했고 automatic `NEEDS_HUMAN_REVIEW`를 바꾸지 않은 채 알려진 local-liquid 후속 과제와 함께 사용자 승인되었다. Scenario 3 Fire / Heat는 genuine combustion, Smoke 생성·소멸, Ice/Water/Steam phase work, finite fuel consumption, Reaction 종료, 완만히 감소하는 Thermal tail, field integrity, exact reset을 확인해 automatic `PASS`와 별도로 사용자 승인되었다. Scenario 4 Pressure Burst는 기존 hot-Wood candidate의 combustion confound를 immutable하게 보존하고, cold-seam clean-source candidate에서 `pressure_opening_precedes_combustion`, Pressure relief, field integrity, exact reset을 확인했다. Automatic `NEEDS_HUMAN_REVIEW`를 바꾸지 않은 채 top-seam-only opening, persistent plume, broad terminal activity를 known follow-up으로 사용자 승인했다. Scenario 5 Heavy Mixed World가 next이고 G8-B는 닫히지 않았다.

같은 fixture를 headless harness에서 선택할 수 있다.

```bat
cargo run --release -p powdergame-benchmark -- --scenario sand-fall
```

Windows Gallery의 렌더링, HUD, wall-clock TPS, bounded diagnostic census/readback은 사람이 fixture를 관찰하기 위한 정보다. 이 값은 official benchmark timing이 아니며 G8-C evidence로 사용하지 않는다. 세부 계약과 현재 미완료 항목은 [`docs/evidence/G8_B_BENCHMARK_SCENARIO_GALLERY_2026-08-17.md`](docs/evidence/G8_B_BENCHMARK_SCENARIO_GALLERY_2026-08-17.md)를 따른다.

## Scenario Experiment Harness

승인된 Sand Fall의 낙하 → 정착 → all-sleep → post-sleep 안정 → exact reset lifecycle, 승인된 Water Flow의 movement → cross-chunk → destination → settle/reset lifecycle, 그리고 승인된 Fire / Heat의 finite fuel → combustion/Smoke/phase → reaction-zero → thermal-tail → reset lifecycle을 같은 one-command coordinator에서 scenario별 analyzer로 분리한다.

```bat
run_experiment.bat sand-fall
run_experiment.bat water-flow --mode scratch
run_experiment.bat water-flow
run_experiment.bat fire-heat --mode scratch
run_experiment.bat fire-heat
```

각 run은 `C:\Users\mdkap\source\Powdergame-artifacts\<unique-run-id>`에만 생성된다. 기존 경로를 덮어쓰지 않으며 `EXPERIMENT_RECEIPT.json`이 Run 디렉터리의 마지막 publication marker인 run만 structurally complete하다. Candidate-only sibling Audit Bundle과 sidecar는 receipt 이후 Run 디렉터리 밖에 생성된다. 로그, telemetry JSONL, raw RGBA, full/crop PNG, report, contact sheet, local review prompt, review packet, hash manifest는 모두 저장소 밖에 남고 Git에 추가하지 않는다.

Sand v0의 일곱 hard predicate와 이미 게시된 pilot/artifact는 변경하지 않는다. Preserved Water candidates도 immutable이며 automatic verdict를 소급 변경하지 않는다. Fire / Heat는 whole-world all-sleep을 요구하지 않고 genuine post-tick Wood/Oil combustion, Smoke, propagated heat, phase inventory change, finite fuel consumption, reaction termination, post-reaction thermal tail, field integrity, and exact reset을 별도 계약으로 기록한다. Sand 계약은 [`docs/evidence/G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md`](docs/evidence/G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md), Water 계약은 [`docs/evidence/G8_B_WATER_FLOW_HARNESS_CANDIDATE_2026-08-17.md`](docs/evidence/G8_B_WATER_FLOW_HARNESS_CANDIDATE_2026-08-17.md), Fire 계약은 [`docs/evidence/G8_B_FIRE_HEAT_HARNESS_CANDIDATE_2026-08-17.md`](docs/evidence/G8_B_FIRE_HEAT_HARNESS_CANDIDATE_2026-08-17.md)를 따른다.

## 핵심 엔진 철학

### One Cell = Max One Matter

한 Cell에는 Matter가 최대 하나만 존재한다. 셀 내부에 Water/Oil/Air의 비율을 저장하는 복합 혼합 모델을 기본으로 하지 않는다.

### Minimum Sufficient Physics

현실의 정밀 공식을 그대로 풀기보다, 원하는 게임 현상을 만드는 **최소 상태 + 최소 local operation**을 사용한다.

예:

```text
부력      → Density Rank 비교 + local displacement
열        → ΔT + cheap conductivity/heat-capacity model
압력      → local ΔP + push/rupture
연소      → ignition condition + Heat/Smoke
전기(향후) → conductive + strength/loss frontier
방사선    → intensity + attenuation
빛        → transmit / absorb / reflect
```

> **부력을 계산하지 않는다. 정렬한다.**

### Read Neighbors, Write Self

일반 interaction은 주변 Cell을 읽고 자기 Next state만 쓴다. 다른 Cell의 소유권을 바꾸는 movement/swap/spawn에만 최소 Claim/Resolve를 사용한다.

### Dense State, Sparse Work

셀 데이터 구조는 단순하게 유지하되 실제로 변화할 가능성이 있는 Chunk/Field/frontier만 계산한다.

- 안정된 Stone bulk → Sleep
- 안정된 Water/Gas bulk → 존재만으로 영원히 Active하지 않음
- 천천히 타는 Wood → 실제 변화 중이므로 Active
- 변화가 접근하면 이웃 Chunk/subsystem을 Wake

> **물질의 양보다 변화 가능한 영역이 계산량을 결정하게 만든다.**

## M0 — First World

M0는 많은 콘텐츠를 넣는 단계가 아니라 다음을 실제 RTX 5090에서 증명하는 단계다.

- 2048×2048 reference world
- 64×64 initial chunk
- Static / Powder / Liquid / Gas local movement
- Density Rank 기반 침강·부력·층분리
- Ice ↔ Water ↔ Steam
- Temperature
- Wood/Oil combustion
- Steam expansion → Pressure → push/rupture/vent
- Active/Sleep
- GPU local parallelism
- subsystem별 performance measurement
- 실제 플레이에서 단순 Rule의 연쇄작용이 재미있는지 사용자 검증

## 문서

문서 권위와 전체 구조는 [`docs/README.md`](docs/README.md)를 먼저 본다.

핵심 문서:

- [`docs/vision/USER_VISION.md`](docs/vision/USER_VISION.md) — 현재 최상위 제품 비전
- [`docs/architecture/ARCHITECTURE.md`](docs/architecture/ARCHITECTURE.md) — 현재 시스템 구조
- [`docs/specs/SIMULATION_SPEC.md`](docs/specs/SIMULATION_SPEC.md) — 시뮬레이션 구현 계약
- [`docs/specs/MATERIAL_SPEC.md`](docs/specs/MATERIAL_SPEC.md) — Material 구조/물성 계약
- [`docs/specs/REACTION_SPEC.md`](docs/specs/REACTION_SPEC.md) — Reaction/Rule 계약
- [`docs/development/PERFORMANCE.md`](docs/development/PERFORMANCE.md) — 성능 철학과 benchmark 기준
- [`docs/planning/ROADMAP.md`](docs/planning/ROADMAP.md) — 장기 제품 방향과 현재 작업 순서
- [`docs/planning/MILESTONES.md`](docs/planning/MILESTONES.md) — Evidence Gate
- [`docs/planning/STATUS.md`](docs/planning/STATUS.md) — 현재 실제 상태와 다음 작업
- [`docs/design-history/2026-08-15-foundation-design-session.md`](docs/design-history/2026-08-15-foundation-design-session.md) — 질문·선택·사용자 코멘트까지 포함한 설계 provenance

## 우리가 만들지 않으려는 것

- 원소 숫자만 많은 얕은 falling-sand 게임
- 메뉴에서 `A + B = C`만 반복하는 조합 게임
- 현실 정확성을 위해 재미·상상력·성능을 희생하는 과학 시뮬레이터
- 한 Cell에 수많은 미래 상태를 미리 넣어 셀 하나를 비싸게 만드는 구조
- CPU가 수백만 Cell을 순서대로 해석하는 구조
- 카메라 밖이라는 이유로 simulation fidelity를 낮추는 spatial LOD
- bit-perfect replay를 위해 GPU 병렬성과 성능을 크게 희생하는 구조

## 최상위 질문

> **“이 세계에 이것을 넣으면 대체 무슨 일이 일어날까?”라는 생각을 계속 하게 만드는가?**

그 질문이 계속 생기고, 세계가 작은 규칙의 조합으로 예상하지 못한 답을 내놓는다면 Powdergame은 올바른 방향에 있다.
