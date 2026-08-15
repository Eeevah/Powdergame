# Powdergame Status

이 문서는 현재 실제 상태를 기록한다. 장기 방향은 `ROADMAP.md`, 완료 기준은 `MILESTONES.md`를 따른다.

---

## Human-maintained Status

### Current Milestone

`M0 — First World`

### Current Milestone Status

`IN_PROGRESS` — G0 (Runtime) 구현/검증 완료. G1 (World Integrity) 대기.

### Current Phase

**G0 — Runtime: 구현 및 로컬 검증 완료 (Windows + RTX 5090 + DX12).**

### Current Summary

2026-08-15 Foundation Design Session에서 확정한 핵심 계약:

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

2026-08-16 기준 **G0 (Runtime)가 구현되고 실제 RTX 5090 머신에서 검증되었다.**

구현 내용:

- Rust workspace (`engine/core`, `engine/gpu`, `apps/windows`)
- `WorldConfig` (기본 2048×2048, chunk 64) + overflow-safe derived buffer layout
- dense GPU world: Current/Next × (material_id u32, temperature f32, pressure f32, flags u32) = 8 buffers
- wgpu/DX12 전용 context (fallback 없음) + high-performance adapter
- headless `Simulation` (Window/Surface/Renderer 없이 init + tick 가능)
- G0 tick: 전 world compute dispatch (runtime plumbing, gameplay rule 없음)
- winit Windows app: window 생성 → DX12 init → world allocation → surface clear → present
- bounded smoke-run 지원 (`--smoke-frames N` / `POWDERGAME_SMOKE_FRAMES`)

### G0 Runtime Evidence (2026-08-16, local run)

```text
Base commit:      6de27451a931cdc3c07cdea012163fb80eab87c6 (main @ G0 시작 시점)
Implementation:   3f67cf0168f4be3735774b6261e592c499b4f5d8
                  — 첫 G0 implementation baseline commit
                  (feat: establish M0 G0 runtime baseline, branch feature/m0-g0-runtime)
Working branch:   feature/m0-g0-runtime
Tested platform:  Windows 11 Pro (hostname DK, build 26200), AMD64
GPU (nvidia-smi): NVIDIA GeForce RTX 5090, driver 596.36, 32 GB VRAM
Adapter (wgpu):   NVIDIA GeForce RTX 5090 (vendor 0x10DE, device 0x2B85)
Backend (wgpu):   Dx12
Device type:      DiscreteGpu
Driver:           32.0.15.9636

WorldConfig:      2048×2048 (chunk 64)
Cell count:       4,194,304
Per buffer:       16,777,216 bytes (× 8 buffers)
Total world:      134,217,728 bytes (128 MiB) — actual wgpu::Buffer allocation OK

Headless test:    cargo test --workspace — 9 passed (8 core + 1 GPU headless smoke)
                  headless_simulation_lifecycle_without_window: PASS
                  (DX12 adapter + RTX 5090 확인, 4 tick, GPU marker readback == 1)
Windows smoke:    powdergame-windows.exe --smoke-frames 60 → exit 0
                  window 생성 OK, tick marker=1, 60 frames present OK, panic/device lost 없음

cargo fmt:        PASS (--check)
cargo build:      PASS (--workspace)
cargo test:       PASS (--workspace)
cargo clippy:     PASS (--workspace --all-targets -- -D warnings)
git diff --check: PASS
```

G0 Evidence Gate 판정 (MILESTONES.md 기준):

```text
Rust workspace/build 성공                          PASS
winit Windows app 실행                             PASS (smoke run exit 0)
wgpu DX12 path 확인                                PASS (wgpu AdapterInfo backend=Dx12)
GPU Simulation Core가 window rendering과 독립 tick   PASS (headless test)
WorldConfig로 world size 설정 가능                  PASS
reference world 2048×2048 초기화 가능               PASS (실제 GPU allocation)
headless/reference execution hook 존재              PASS (Simulation::new/tick/read_marker)

G0 추가 확인:
actual adapter/device info 출력                     PASS
RTX 5090 실제 선택 확인                             PASS (vendor 0x10DE, name match)
dense Current/Next world buffers 실제 GPU allocation PASS (8 × 16 MiB)
Window empty-frame present 성공                      PASS (60 frames)
```

M0 전체는 `ACHIEVED`가 아니다. G1~G9와 사용자 승인이 남아 있다.

### Product Direction

> **현실을 구현하는 것이 아니라 가상의 재미있는 놀이터를 만든다. 핵심은 나만의 세계 창조다.**

현실의 자연현상은 reference이며 Powdergame 내부의 이해 가능한 논리와 상호작용이 우선한다.

### Next Action

G1 — World Integrity:

1. One Cell = Max One Matter invariant test
2. EMPTY가 Matter가 아님을 코드/테스트로 확정
3. editable outer BLOCK + open boundary → Void
4. invalid material id / out-of-bounds 방지

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

M0 implementation: **IN_PROGRESS** — G0 Runtime 로컬 검증 완료, G1 대기

M0 `ACHIEVED`: **NO**

최종 M0 완료는 실제 구현/benchmark/play validation 후 사용자 승인이 필요하다.

---

## Machine-generated Facts

> 이 블록은 향후 script/CI/benchmark 도구가 갱신하도록 설계한다. 자동 생성 pipeline이 아직 없으므로 아래 값은 2026-08-16 로컬 검증에서 사람이 기록한 사실값이다. pipeline이 생기면 이 블록은 자동 갱신으로 전환한다.
>
> `base_commit_sha`는 G0 시작 시점의 main commit이다. 첫 G0 implementation baseline commit
> (`3f67cf0168f4be3735774b6261e592c499b4f5d8`)은 이 블록이 아니라 Human-maintained
> G0 Runtime Evidence 영역에 기록한다. 자동 pipeline이 없으므로 이 블록에
> self-referential current commit SHA를 억지로 넣지 않는다.

```text
base_commit_sha: 6de27451a931cdc3c07cdea012163fb80eab87c6
build_id: local-cargo-2026-08-16
platform: Windows
primary_gpu: RTX 5090
world_config: 2048x2048 reference
chunk_config: 64x64 initial
build: passed (cargo build --workspace)
tests: passed (8 core + 1 GPU headless smoke)
benchmarks: not_started
m0_status: IN_PROGRESS (G0 complete)
```
