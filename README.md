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

`integration/canonical-recovery`는 이 검증 구현선과 최신 research/Foundation Material Wiki를 하나의 tested local integration line으로 결합했다. recovery branch push/recovery PR/`main` 승격은 하지 않았으며, G8-B/G8-C, G9, M0 이후 P1 중 어느 작업을 진행할지는 사용자가 별도로 결정한다.

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
