# Powdergame Status

이 문서는 현재 실제 상태를 기록한다. 장기 방향은 `ROADMAP.md`, 완료 기준은 `MILESTONES.md`를 따른다.

---

## Human-maintained Status

### Current Milestone

`M0 — First World`

### Current Milestone Status

`IN_PROGRESS` — G0 (Runtime) PASS, G1 (World Integrity) PASS, G2 (Local Movement) PASS / CLOSED, G3 (Density / Displacement) PASS / CLOSED, G4 (Thermal / Phase / Combustion) PASS / CLOSED (G4-A thermal baseline TECHNICAL PASS, G4-B phase transition TECHNICAL PASS, G4-C combustion & finite fuel TECHNICAL PASS, Smoke decay lifecycle G4 integration hardening TECHNICAL PASS, G4 Large 4-Panel Thermal Observatory `--thermal-demo` User Validation APPROVED on 2026-08-16). G5 (Pressure Chain) IN_PROGRESS — G5-A Pressure Field TECHNICAL PASS / FROZEN, G5-B Expansion / Confinement → Pressure TECHNICAL PASS / FROZEN, G5-C Rupture / Opening / Vent implementation & validation in progress.

### Current Phase

**G0 — Runtime: PASS. G1 — World Integrity: PASS. G2 — Local Movement: PASS / CLOSED. G3 — Density / Displacement: PASS / CLOSED. G4 — Thermal / Phase / Combustion: PASS / CLOSED (G4-A thermal baseline TECHNICAL PASS, G4-B phase transition TECHNICAL PASS, G4-C combustion TECHNICAL PASS, Smoke decay G4 integration hardening TECHNICAL PASS, G4 4-Panel Thermal Observatory `--thermal-demo` (320×192) User Validation APPROVED on 2026-08-16). G5 — Pressure Chain: IN_PROGRESS (G5-A Pressure Field TECHNICAL PASS / FROZEN; G5-B Expansion / Confinement → Pressure TECHNICAL PASS / FROZEN; G5-C Rupture / Opening / Vent IN_PROGRESS).**

### Current Summary

2026-08-15 Foundation Design Session에서 결정된 핵심 계약:

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
- spatial ownership collision only — Claim/Resolve
- loose causal phases
- f32 Temperature/Pressure baseline (relative gameplay scalar, not Celsius)
- Ice ↔ Water ↔ Steam
- Wood/Oil combustion
- Pressure / rupture / vent
- Active Chunk / stable bulk sleep
- Minimum Sufficient Physics
- approximate, non-bit-exact determinism
- M0 Evidence Gates G0~G9

2026-08-16 기준 **G0 (Runtime)**, **G1 (World Integrity)**, **G2 (Local Movement)**, **G3 (Density / Displacement)**, **G4 (Thermal / Phase / Combustion)**가 구현·검증·사용자 검증 승인 완료되어 **PASS / CLOSED** 되었다.

G4는 **G4-A (Thermal Baseline)**, **G4-B (Phase Transition)**, **G4-C (Combustion & Finite Fuel)**, 그리고 **Smoke Decay Lifecycle (G4 Integration Hardening)**를 포함하며, 4-Panel Large Thermal Observatory (`--thermal-demo`, 320×192) 및 고해상도 Screen-Space 진단 HUD를 통한 장기 관측(~9104 ticks)에서 물리 보존 법칙과 라이프사이클이 모두 입증되어 사용자의 공식 승인을 받았다.

현재 **G5 — Pressure Chain** 진행 중이다. G5-A scalar Pressure propagation과 G5-B Phase expansion / confinement → Pressure generation은 RTX 5090 / DX12 실기 검증으로 TECHNICAL PASS / FROZEN이며, 다음 sub-gate는 **G5-C — Pressure stress → rupture → opening → venting**이다.

---

### G0 Runtime Evidence (2026-08-16, local run)

```text
Base commit:      6de27451a931cdc3c07cdea012163fb80eab87c6 (main @ G0 시작 시점)
Tested platform:  Windows 11 Pro (hostname DK, build 26200), AMD64
GPU (nvidia-smi): NVIDIA GeForce RTX 5090, driver 596.36, 32 GB VRAM
DirectX:          DX12
Window:           1280x720 windowed, resizable
Surface:          Bgra8UnormSrgb, PresentMode::Fifo (vsync), auto-detected
Reference world:  2048x2048 dense cells, 64x64 chunks (32x32 = 1024 chunks)
World buffers:    material, temperature, pressure, flags (current + next each) = 128 MB VRAM
Scratch buffers:  proposal, claim, marker = 32 MB VRAM
Shader pipeline:  propose -> claim -> commit -> copy next->current
Tick behavior:    Simulation::tick advances world state without full-world CPU readback
Headless smoke:   1 tick executed headless before window creation, marker=1
Rendering:        Fullscreen quad, 2 triangles, linear UV sampling
Result:           60 frames presented, smooth window lifecycle, exit 0, no device lost
```

**G0 — Runtime: PASS.**

---

### G1 World Integrity Evidence (2026-08-16, local run)

```text
World bounds:     2048x2048 dense cells, 64x64 chunks
Coordinate system: (0,0) top-left, (+X right, +Y down), local per-cell addresses
Matter identity:  One Cell = Max One Matter (ADR-0001)
EMPTY invariant:  EMPTY = 0, EMPTY is not Matter, empty cells hold no phantom state
Boundary ring:    Boundary Block (real registered Matter) on all 4 outer edges
Void semantics:   Out-of-bounds coordinates evaluate to Void (read Void, write dropped)
Editable:         Outer Boundary Block can be edited to EMPTY for open boundary tests
```

**G1 — World Integrity: PASS.**

---

### G2 Local Movement Evidence (2026-08-16, local run)

```text
Movement families: STATIC / POWDER / LIQUID / GAS
Local stencil:    First-Match candidate selection (1 cell per tick max, no teleportation)
Ownership:        Claim/Resolve spatial collision resolution (exactly one winner)
Boundary flow:    Open boundaries allow matter to exit into Void (exact matter count conservation)
Chunk boundaries: Pure local stencil; chunk edges behave identically to interior cells
```

**G2 — Local Movement: PASS / CLOSED.** (User validation confirmed).

---

### G3 Density / Displacement Evidence (2026-08-16, local run)

```text
Density model:    Discrete integer Density Rank as Material property (no float buoyancy solver)
Rank order:       Steam (20) < Smoke (30) < Oil (70) < Water (90) < Sand (150)
STATIC:           Stone, Boundary Block have rank None (never density swapped)
Displacement:     Local pairwise write-self swap across vertical/diagonal channels
Conservation:     Matter identity and count strictly conserved across all swaps
```

**G3 — Density / Displacement: PASS / CLOSED.** (User validation confirmed).

---

### G4 Thermal / Phase / Combustion Evidence

#### G4-A Thermal Baseline (TECHNICAL PASS)
- 4-neighbor finite conduction with material-specific thermal conductivity and heat capacity.
- Temperature is a relative gameplay scalar (not Celsius); `0.0` reference temperature.
- EMPTY is not a thermal medium (does not conduct heat; reset to reference T=0.0).
- Temperature values are sanitized (clamped, no NaN/Inf runaway).
- Temperature is carried with Matter identity across movement and density swaps.

#### G4-B Phase Transition (TECHNICAL PASS)
- 1:1 local self-transitions: `Ice (T > -10.0) -> Water`, `Water (T < -20.0) -> Ice`, `Water (T > 60.0) -> Steam`, `Steam (T < 40.0) -> Water`.
- Hysteresis bands (-20.0 <-> -10.0, 40.0 <-> 60.0) prevent ping-pong oscillation.
- Phase-transformed Matter adopts new movement behavior on the very next tick.

#### G4-C Combustion & Finite Fuel (TECHNICAL PASS)
- Generic combustion descriptor: `Wood (ignite 90.0, sustain 55.0, heat +4.0, burn_duration 900)` / `Oil (ignite 75.0, sustain 45.0, heat +5.0, burn_duration 600)`.
- Fire is not Matter: Flame is burning Matter + `FLAG_COMBUSTING` + `FLAG_FLAME_EVENT` + heat.
- Finite fuel lifecycle: `FLAG_FUEL_PROGRESS` increments during active burning; when reaching `burn_duration`, cell self-writes to `EMPTY` (T=0.0, flags=0).
- Burning source requests at most 1 Smoke spawn into an adjacent EMPTY cell per tick.

#### Smoke Finite Lifetime & Generic Decay (G4 Integration Hardening — TECHNICAL PASS)
- Generic `DecayDescriptor { lifetime_ticks: 900, target_material: MATERIAL_EMPTY }` (15s @ 60 TPS).
- `flags[]` bit layout:
  - Bit 0: `FLAG_COMBUSTING` (`1 << 0`)
  - Bit 1: `FLAG_FLAME_EVENT` (`1 << 1`)
  - Bits 4..15 (12 bits): `FLAG_FUEL_PROGRESS` (`0x0FFF << 4`)
  - Bits 16..27 (12 bits): `FLAG_DECAY_AGE` (`0x0FFF << 16`)
  - Bits 28..31 (4 bits): reserved / unrelated test flags.
- Decay pass runs after phase transition and before combustion; smoke age increments deterministically and transforms to EMPTY upon reaching lifetime bound.
- Smoke age moves with Smoke identity across movement, gas density swaps, and chunk boundaries.

---

### G4 Large 4-Panel Thermal Observatory — User Validation Evidence (~9104 Ticks, APPROVED 2026-08-16)

실제 사용자 관찰(~9104 ticks 직접 관측 및 승인)에서 확인된 정밀 물리 검증 증거:

#### A. PHASE HEATING (Top-Left: x 1..157, y 1..93)
- **초기 상태**: `ICE 360`, `WATER 0`, `STEAM 0`
- **장기 관측 데이터**:
  - `tick ~1516`: `ICE 315` / `WATER 41` / `STEAM 4`
  - `tick ~2490`: `ICE 258` / `WATER 97` / `STEAM 5`
  - `tick ~9104`: `ICE 246` / `WATER 106` / `STEAM 8`
  - `First Melt = 360`, `First Steam = 1080`
- **핵심 물리 증거**:
  - `Ice + Water + Steam = 360` (전체 관측 구간에서 오차 0으로 완벽 보존).
  - 열원 전도에 의해 `Ice -> Water -> Steam` 상전이 체인이 순차적으로 발생하며, 1:1 변환에서 Matter count가 완벽히 보존됨.

#### B. PHASE COOLING (Top-Right: x 162..318, y 1..93)
- **초기 상태**: `STEAM 3082`, `WATER 0`, `ICE 0` (Initial Ice = 0)
- **장기 관측 데이터**:
  - `tick ~158`: `STEAM 2007` / `WATER 1072` / `ICE 3`
  - `tick ~834`: `STEAM 50` / `WATER 2980` / `ICE 52`
  - `tick ~1516`: `STEAM 6` / `WATER 3053` / `ICE 23`
  - `tick ~2490`: `STEAM 1` / `WATER 3081` / `ICE 0`
  - `tick ~3327`: `STEAM 185` / `WATER 2897` / `ICE 0`
  - `tick ~6426`: `STEAM 1180` / `WATER 1902` / `ICE 0`
  - `tick ~9104`: `STEAM 1778` / `WATER 1304` / `ICE 0`
  - `First Condense = 20`, `First Freeze = 130`
- **핵심 물리 증거**:
  - `Steam + Water + Ice = 3082` (전체 관측 시점에서 정확히 일치 보존).
  - 초기 냉각 단계: 고온 Steam이 천장/냉각핀(T=-40.0)에 접촉해 Water로 응축 → 중앙 틈새로 낙하 → -100.0 수조에서 Ice로 결빙되는 역방향 체인(`Steam -> Water -> Ice`) 확인.
  - 중장기 재평형 현상 (Emergent Behavior): 냉각 석재 수조/핀이 무한 냉각기가 아닌 유한 열용량 매체(finite thermal reservoir)이므로, 대량의 Steam 열을 흡수하며 서서히 온도가 상승함에 따라 수조의 Ice가 녹고 일부 Water가 다시 증발함. 이는 시뮬레이션 물리 장치 간의 자연스러운 열적 재평형(thermal re-equilibration) 결과임 (현실 이상 열역학 증명으로 과장하지 않음).

#### C. HEAT COMPARISON (Bottom-Left: x 1..157, y 98..190)
- **관측 조건**: 동일 형상의 밀봉 석재 튜브 내 Water (`k=1.0`) vs Oil (`k=0.2`)의 열침투 깊이 비교 (바닥 공통 열원 T=50.0).
- **장기 관측 데이터**:
  - `tick ~6420`:
    - Water: `Mid T ≈ 2.9`, `High T ≈ 0.8`
    - Oil: `Mid T ≈ 0.1`, `High T ≈ 0.0`
  - `tick ~9100`:
    - Water: `Mid T ≈ 4.1`, `High T ≈ 1.7`
    - Oil: `Mid T ≈ 0.4`, `High T ≈ 0.0`
- **핵심 물리 증거**:
  - 열전도율 차이(Water 1.0 vs Oil 0.2)에 의한 공간적 열침투 속도 격차가 Mid T / High T 밴드에서 명확하게 입증됨.

#### D. COMBUSTION / FINITE FUEL (Bottom-Right: x 162..318, y 98..190)
- **초기 상태**: `Wood Start = 648`
- **장기 관측 데이터**:
  - `First Ignite = 160`, `First Empty = 1060`
  - `tick ~1516`: `Wood Left 583`
  - `tick ~2490`: `Wood Left 431`
  - `tick ~3327`: `Wood Left 290`
  - `tick ~4552`: `Wood Left 50`
  - `tick ~6426`: `Wood Left 0`, `Burning 0`
- **핵심 물리 증거**:
  - 열전도 점화 -> 공간적 연소 전선 전파 -> 유한 연료 소비(900 ticks) -> Wood가 EMPTY로 완전 소모되는 전 과정이 장기 실행에서 완벽히 완주됨.

#### E. SMOKE LIFETIME & DISSIPATION LIFECYCLE
- **장기 관측 데이터**:
  - `tick 0`: `Smoke 0`
  - `tick ~834`: `Smoke 1394`
  - `tick ~1516`: `Smoke 2665`
  - `tick ~2490`: `Smoke 3207`
  - `tick ~3327`: `Smoke 3403` (최대 누적 피크)
  - `tick ~4552`: `Smoke 2857` (연료 소모에 따라 생성 감소)
  - `tick ~6426`: `Smoke 0` (연료 소모 종료 후 900 ticks 수명 경과로 100% 완전 자연 소멸)
- **핵심 물리 증거**:
  - `생성(spawn) -> 상승 집적(accumulation) -> 연료 소진 -> 자연 소멸(decay) -> 완전 소진(EMPTY)`의 연기 라이프사이클이 장기 실행에서 확인됨.

---

### Known Artifacts & Deferred Items

1. **GAS Triangular / Wedge Plume Artifact (Deferred)**:
   - "Large GAS masses can form visibly geometric triangular/wedge plumes under the current deterministic local gas stencil. Do not patch with ad-hoc RNG/jitter during G4. Re-evaluate gas spreading after G5 Pressure and later gas-flow polish."
   - 결정론적 1-cell 국소 스텐실에 의한 거시적 형상이며, 물질 보존이나 물리 결함이 아님. G4 단계에서 인위적 RNG/jitter/fake diffusion을 적용하지 않고 G5 압력 체인 연동 후 종합 검토.
2. **Generic Decay Scope (Deferred Expansion)**:
   - 현재 generic `DecayDescriptor { lifetime_ticks, target_material }`는 `SMOKE -> EMPTY` (material=EMPTY, temp=0.0, flags=0) 케이스로 검증됨.
   - Non-EMPTY 감쇄 대상(예: 잔해, 재/소트 생성 등)의 온도/플래그 전이 규칙은 추후 확장 항목으로 분류.
3. **Modern Presentation FX Layer (Deferred)**:
   - 연속 유체 연기, 스크린 스페이스 불꽃, 열기 왜곡(heat haze/refraction), 블룸(bloom/glow), 매끄러운 파티클 트레일 등은 시뮬레이션 진실(cell truth)과 분리된 최종 프레젠테이션 계층으로 추후 구현 (remote docs commits 원칙 보존: Cell simulation != cell-bound presentation).
4. **Ash / Soot (Deferred)**:
   - 연소 잔여물 매커니즘은 후속 마일스톤으로 보류.

---

### G0/G1/G2/G3/G4 Automated Test Evidence Summary

```text
cargo test --workspace 전체 PASS (242 passed, 0 failed, 1 ignored controlled benchmark)
  - Core unit tests: 115 passed
  - GPU combustion & decay tests: 56 passed
  - GPU density displacement tests: 15 passed
  - GPU phase transition tests: 16 passed
  - GPU thermal conduction tests: 13 passed
  - GPU local movement tests: 16 passed (1 perf benchmark ignored)
  - GPU world integrity tests: 7 passed
  - GPU headless smoke test: 1 passed
  - Windows observatory unit tests: 3 passed

Runtime Smoke Test Suite (RTX 5090 / DX12):
  - cargo run -p powdergame-windows -- --smoke-frames 60: PASS
  - cargo run -p powdergame-windows -- --movement-demo --smoke-frames 120: PASS
  - cargo run -p powdergame-windows -- --density-demo --smoke-frames 180: PASS
  - cargo run -p powdergame-windows -- --thermal-demo --smoke-frames 360: PASS
  - cargo run -p powdergame-windows -- --thermal-demo --smoke-frames 1200: PASS
  - cargo run -p powdergame-windows -- --thermal-demo --smoke-frames 3000: PASS (device lost 0, clean exit)

Static Analysis & Formatting:
  - cargo fmt --all -- --check: PASS
  - cargo clippy --workspace --all-targets -- -D warnings: PASS (0 warnings)
  - git diff --check: PASS (0 whitespace/syntax warnings)
```

---

### Product Direction

> **현실을 구현하는 것이 아니라 가상의 살아있는 생태계를 만든다. 핵심은 나만의 세계 창조다.**

현실의 자연현상은 reference이며 Powdergame 세계관에 부합하는 재미와 상호작용이 우선이다.

---

### Next Action

1. G0 (Runtime), G1 (World Integrity), G2 (Local Movement), G3 (Density / Displacement), **G4 (Thermal / Phase / Combustion)**: **ALL PASS / CLOSED** (2026-08-16).
2. 다음 마일스톤 게이트 착수: **G5 — Pressure Chain** (Phase expansion / yield / Pressure / rupture / vent).
3. M0 First World 마일스톤 전체는 G5~G9 완료 시까지 `IN_PROGRESS` 유지.

---

### Blockers

현재 문서/설계 기준으로 known hard blocker 없음.

---

## Approval State

Foundation Design direction: **APPROVED BY USER**

M0 implementation: **IN_PROGRESS** — G0/G1/G2/G3/G4 PASS / CLOSED (G2/G3/G4 User Validation 모두 APPROVED 완료), G5~G9 + 최종 M0 승인 남음

M0 `ACHIEVED`: **NO**

---

## Machine-generated Facts

```text
base_commit_sha: 6de27451a931cdc3c07cdea012163fb80eab87c6
build_id: local-cargo-2026-08-16
platform: Windows
primary_gpu: RTX 5090
world_config: 2048x2048 reference
chunk_config: 64x64 initial
build: passed (cargo build --workspace)
tests: passed (115 core + 56 GPU combustion/decay + 15 GPU density + 16 GPU phase + 1 GPU headless smoke + 16 GPU movement + 13 GPU thermal + 7 GPU world integrity + 3 windows observatory = 242 total, ignored 1 controlled benchmark)
benchmarks: G2 controlled reference-world baseline: median ~0.146 ms/tick (~6838 TPS, RTX 5090/DX12). G3/G4: correctness-first.
m0_status: IN_PROGRESS (G0 complete, G1 complete, G2 PASS/CLOSED, G3 PASS/CLOSED, G4 PASS/CLOSED incl. User Validation APPROVED 2026-08-16, G5 pending)
```
