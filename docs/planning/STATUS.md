# Powdergame Status

이 문서는 현재 실제 상태를 기록한다. 장기 방향은 `ROADMAP.md`, 완료 기준은 `MILESTONES.md`를 따른다.

---

## Human-maintained Status

### Current Milestone

`M0 — First World`

### Current Milestone Status

`PLANNED`

### Current Phase

**Foundation Design completed enough to begin implementation.**

### Current Summary

2026-08-15 Foundation Design Session에서 다음 핵심 계약을 확정했다.

- Windows + Rust + winit + wgpu/DX12
- RTX 5090 primary performance target
- finite 2048×2048 reference world
- initial 64×64 chunks
- GPU Production authoritative
- One Cell = Max One Matter
- EMPTY is not Matter
- editable outer BLOCK / Void outside domain
- Static / Powder / Liquid / Gas
- Density Rank local displacement
- Read Neighbors / Write Self
- spatial ownership collision only → Claim/Resolve
- loose causal phases
- f32 Temperature/Pressure baseline
- Ice ↔ Water ↔ Steam
- Wood/Oil combustion
- Pressure / rupture / vent
- Active Chunk / stable bulk sleep
- Minimum Sufficient Physics
- approximate, non-bit-exact determinism
- M0 Evidence Gates G0~G9

### Product Direction

> **현실을 구현하는 것이 아니라 가상의 재미있는 놀이터를 만든다. 핵심은 나만의 세계 창조다.**

현실의 자연현상은 reference이며 Powdergame 내부의 이해 가능한 논리와 상호작용이 우선한다.

### Next Action

M0 implementation을 시작한다.

첫 순서:

1. Rust workspace / Windows runtime skeleton
2. wgpu DX12 device + simulation/render separation
3. `WorldConfig` + 2048×2048 GPU world
4. Stone + Sand baseline
5. first benchmark evidence

### Blockers

현재 문서/설계 기준으로 known hard blocker 없음.

### Deferred

- Interaction Lab 상세 구현
- Electricity
- Radiation
- Gameplay Light physics
- Agent/Concept/Meta layers
- Browser/macOS product support
- broad GPU compatibility

---

## Approval State

Foundation Design direction: **APPROVED BY USER**

M0 implementation: **NOT STARTED / NOT VALIDATED**

M0 `ACHIEVED`: **NO**

최종 M0 완료는 실제 구현/benchmark/play validation 후 사용자 승인이 필요하다.

---

## Machine-generated Facts

> 이 블록은 향후 script/CI/benchmark 도구가 갱신하도록 설계한다. 현재는 implementation 전이므로 사실값이 없다. 자동화가 생긴 뒤 사람은 이 블록을 수동 수정하지 않는 것을 원칙으로 한다.

```text
commit_sha: pending
build_id: pending
platform: Windows
primary_gpu: RTX 5090
world_config: 2048x2048 reference
chunk_config: 64x64 initial
build: not_started
tests: not_started
benchmarks: not_started
m0_status: PLANNED
```
