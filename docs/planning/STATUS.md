# Powdergame Status

이 문서는 현재 실제 상태를 기록한다. 장기 방향은 `ROADMAP.md`, 완료 기준은 `MILESTONES.md`를 따른다.

---

## Human-maintained Status

### Current Milestone

`M0 — First World`

### Current Milestone Status

`IN_PROGRESS` — G0 (Runtime) PASS, G1 (World Integrity) PASS, G2 (Local Movement) PASS / CLOSED, G3 (Density / Displacement) PASS / CLOSED, G4 (Thermal / Phase / Combustion) PASS / CLOSED (User Validation APPROVED on 2026-08-16), G5 (Pressure Chain) PASS / CLOSED (G5 User Validation APPROVED on 2026-08-16). Next Gate: G6 — Parallel Integrity.

### Current Phase

**G0 — Runtime: PASS. G1 — World Integrity: PASS. G2 — Local Movement: PASS / CLOSED. G3 — Density / Displacement: PASS / CLOSED. G4 — Thermal / Phase / Combustion: PASS / CLOSED (User Validation APPROVED 2026-08-16). G5 — Pressure Chain: PASS / CLOSED (G5-A Pressure Field TECHNICAL PASS / FROZEN; G5-B Expansion / Confinement → Pressure TECHNICAL PASS / FROZEN; G5-C Rupture / Opening / Vent TECHNICAL PASS / FROZEN; 2×2 Multi-Boiler Stress Lab User Validation APPROVED 2026-08-16). Next Gate: G6 — Parallel Integrity.**

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

2026-08-16 기준 **G0 (Runtime)**, **G1 (World Integrity)**, **G2 (Local Movement)**, **G3 (Density / Displacement)**, **G4 (Thermal / Phase / Combustion)**, **G5 (Pressure Chain)**가 구현·검증·사용자 검증 승인 완료되어 **PASS / CLOSED** 되었다.

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
- Generic combustion descriptor: `Wood (ignite T=90.0, sustain T=55.0, heat +4.0, burn_duration 900)` / `Oil (ignite T=75.0, sustain T=45.0, heat +5.0, burn_duration 600)`.
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

4-Panel Large Thermal Observatory (`--thermal-demo`, 320×192) 및 고해상도 Screen-Space 진단 HUD를 통한 장기 관측(~9104 ticks)에서 물리 보존 법칙과 라이프사이클이 모두 입증되어 사용자의 공식 승인을 받았다.

---

### G5 Pressure Chain Evidence — RTX 5090 / DX12 (PASS / CLOSED, User Validation APPROVED 2026-08-16)

#### G5 Technical Chain Sub-Gates
* **G5-A Pressure Field**: `TECHNICAL PASS / FROZEN` (Scalar spatial `f32` Pressure Field; Liquid/Gas are media; 4-neighbor propagation; no arbitrary decay).
* **G5-B Expansion / Confinement**: `TECHNICAL PASS / FROZEN` (Boiling uses Matter yield=2; blocked expansion becomes confinement Pressure `100.0`).
* **G5-C Rupture / Opening / Vent**: `TECHNICAL PASS / FROZEN` (Wood rupture threshold `80.0`; Stone/Boundary unbreakable; spatial pressure cleared on vent).

```text
Tested implementation SHA: 5187d9980f9067cced1edb0b6a8f79ab56147a0c
Validation worktree: C:\Users\mdkap\source\repos\Powdergame-g5-user-validation
Adapter: NVIDIA GeForce RTX 5090
Vendor: 0x10DE
Backend: wgpu::Backend::Dx12
WGSL parse: 1 passed, 0 failed (rupture.wgsl included)
G5-C rupture & stress lab tests: 7 passed, 0 failed
G5-B expansion regression: 5 passed, 0 failed
G5-A pressure regression: 8 passed, 0 failed
G4-B phase regression: 16 passed, 0 failed
Full GPU integration: 145 passed, 0 failed, 1 ignored (controlled_reference_world_perf)
Core: 130 passed, 0 failed
Windows observatory: 4 passed, 0 failed
Workspace all-target check: 0 errors, 0 warnings
Static Analysis (clippy): 0 warnings (-D warnings)
Formatting (cargo fmt): clean
Git diff check: clean
Original user MATERIAL_CANDIDATES.md: preserved untouched
```

#### G5 Production Simulation Invariant
G5의 모든 물리적 인과 사슬은 특별 explosion 코드 없이 오직 실제 GPU 프로덕션 시뮬레이션 파이프라인(`Phase → Pressure → Rupture → Movement`)에서 발생한다:
- Scripted timer rupture 없음
- Pre-staged Pressure 없음
- Pre-staged Steam 없음
- Radial explosion solver 없음
- Fake vent animation 없음
- Minimum Sufficient Physics 원칙으로 성립.

---

### G5 2×2 Multi-Boiler Stress Lab — Architecture & Final Evidence

```text
┌─────────────────────────────────────────┬─────────────────────────────────────────┐
│ [A] TOP-LEFT: WOOD RELIEF (CANONICAL)   │ [B] TOP-RIGHT: STONE SEALED (CONTROL)   │
│ • 1x Floor Heater (T=150)               │ • 1x Floor Heater (T=150)               │
│ • 1x Upper Heater (T=110)               │ • 1x Upper Heater (T=110)               │
│ • 9-cell Wood Roof Relief Plug (x=60..68│ • 100% Unbreakable Stone Roof           │
├─────────────────────────────────────────┼─────────────────────────────────────────┤
│ [C] BOT-LEFT: WOOD RELIEF (EXTREME)     │ [D] BOT-RIGHT: DELAYED PRESSURE BREACH  │
│ • 3x Floor Heaters (T=220 Overdrive)    │ • 3x Floor Heaters (T=220 Overdrive)    │
│ • 1x Upper Heater (T=130, y=176)        │ • 1x Upper Heater (T=130, y=176)        │
│ • 9-cell Wood Roof Relief Plug (x=60..68│ • Solid Stone Roof + 9-cell Wood        │
│                                         │   Distant Side Seam (y=214..=222, x=242)│
└─────────────────────────────────────────┴─────────────────────────────────────────┘
```

#### 1. Interactive User Observation — FINAL (2026-08-16)

사용자가 직접 2×2 Stress Lab을 관찰하고 최종 승인한 런타임 실측 증거:

* **[A] WOOD RELIEF — CANONICAL**:
  - Peak Pressure: `650.0`
  - First Relief: `Tick 40`
  - Relief Plug Wood: `6/9` cells remaining after opening
  - Sustained Steam vent observed
  - Final state: `RELIEF ACTIVE / VENTING`
  - **판정**: canonical Heat → Steam → confinement → Wood relief rupture → opening → vent chain PASS.

* **[B] STONE SEALED — CONTROL**:
  - Peak Pressure: `650.0`
  - Rupture Event: `NONE`
  - Chamber Integrity: `100% SEALED`
  - Long-run sealed state maintained indefinitely
  - **판정**: 동일 canonical heating 조건에서 Stone control이 rupture하지 않음을 직접 확인.

* **[C] WOOD RELIEF — EXTREME**:
  - Initial Staging: Panel D와 100% 동일한 초기 Matter / Temperature (Water T=58.0, 3x Floor Heaters T=220, Upper Heater T=130).
  - Peak Pressure: `1314.4`
  - First Relief: `Tick 35`
  - Sustained high-output upward vent plume observed
  - A보다 relief가 더 빠름 (`35 < 40`) 및 Peak Pressure가 더 높음 (`1314.4 > 650.0`).
  - **판정**: extreme heating에서도 relief path가 조기 pressure release를 제공함.

* **[D] DELAYED PRESSURE BREACH**:
  - Initial Staging: Panel C와 100% 동일한 초기 Matter / Temperature (유일한 차이는 구조적 relief path 배치).
  - Peak Pressure: `1307.7`
  - First Breach: `Tick 135`
  - Weak Seam Wood: `8/9` cells remaining after first rupture
  - Duct Steam Vent: `Tick 170`
  - Final state: `SIDE WALL BREACH -> VENTING`
  - Relative observation: C First Relief = `35`, D First Breach = `135` $\to$ Difference = `100 simulation ticks` (60 TPS 기준 약 1.67초).
  - D는 C보다 충분히 오래 confinement를 유지한 뒤 pressure가 far-side seam까지 자연 전파되어 rupture했고, 그 이후 실제 opening을 통해 Steam이 duct로 vent함.
  - **판정**: delayed pressure propagation → structural stress → rupture → opening → vent PASS.

#### 2. Automated Contract Fixture Evidence (`two_by_two_multi_boiler_stress_lab_relative_ordering_contract`)

소형 결정론적 GPU 회귀 테스트 픽스처에서의 검증 수치:
- `first_relief(C) = Tick 33` <= `first_relief(A) = Tick 36`
- `rupture(B) == NONE` (100% unbreakable sealed)
- `first_breach(D) = Tick 133` (Separation: `133 - 33 = 100 ticks >= MIN_MEANINGFUL_DELAY (60)`)
- `breach_local_pressure(D) = 80.8 >= 80.0` (Wood rupture threshold 도달)
- `first_vent(D) = Tick 170 > Tick 133` (파열구 개방 후 증기가 배기 덕트로 진입)
- `test_c_d_initial_thermal_matter_symmetry`: $t=0$ 시점 Panel C와 Panel D 챔버 내부의 모든 셀에 대해 Material 및 Temperature 100% 동일성 검증 (`PASS`).

---

### Known Artifacts & Deferred Items

1. **GAS Triangular / Wedge Plume Artifact (Deferred)**:
   - "Large GAS masses can still expose geometric triangle / wedge / column patterns from the current deterministic local GAS stencil. G5 Pressure does not introduce a full gas velocity / turbulence solver. Do not reopen frozen G5 physics with ad-hoc RNG, fake diffusion, or presentation hacks. Revisit during later gas-flow / presentation polish."
   - 이 artifact는 G5 blocker가 아니며, 물리 보존 및 인과율이 완전히 증명되었으므로 후속 프레젠테이션/가스 흐름 폴리시 단계로 분류.
2. **Column-Like Vent Shape & Corner Accumulation (Deferred)**:
   - 좁은 굴뚝 및 밀폐 챔버 모서리에서 증기가 상승하여 집적될 때 국소 충돌 순서에 따른 칼럼 형상 발생. Deterministic local stencil의 자연스러운 거시적 결과로 분류.
3. **Generic Decay Scope (Deferred Expansion)**:
   - 현재 generic `DecayDescriptor { lifetime_ticks, target_material }`는 `SMOKE -> EMPTY` 케이스로 완결 검증됨.
4. **Modern Presentation FX Layer (Deferred)**:
   - 연속 유체 렌더링, 스크린 스페이스 파티클 블룸, 열기 왜곡(heat haze), 매끄러운 연기 트레일 등은 시뮬레이션 진실(cell truth)과 분리된 최종 프레젠테이션 계층으로 추후 구현 (ADR: Cell simulation != cell-bound presentation).
5. **Ash / Soot (Deferred)**:
   - 연소 잔여물 매커니즘은 후속 마일스톤으로 보류.

---

### Automated Test Evidence Summary (Total 279 Tests)

```text
cargo test --workspace -- --test-threads=1 전체 PASS (279 passed, 0 failed, 1 ignored controlled benchmark)
  - Core unit tests: 130 passed
  - GPU combustion & decay tests: 56 passed
  - GPU density displacement tests: 15 passed
  - GPU expansion tests: 5 passed
  - GPU headless smoke test: 1 passed
  - GPU local movement tests: 16 passed (1 perf benchmark ignored)
  - GPU phase transition tests: 16 passed
  - GPU scalar pressure tests: 8 passed
  - GPU rupture & 2x2 multi-boiler stress lab tests: 7 passed
  - GPU thermal conduction tests: 13 passed
  - GPU world integrity tests: 7 passed
  - Windows observatory & pressure lab unit tests: 4 passed
  - WGSL syntax parse tests: 1 passed

Runtime Smoke Test Suite (RTX 5090 / DX12):
  - cargo run -p powdergame-windows -- --smoke-frames 60: PASS
  - cargo run -p powdergame-windows -- --movement-demo --smoke-frames 120: PASS
  - cargo run -p powdergame-windows -- --density-demo --smoke-frames 180: PASS
  - cargo run -p powdergame-windows -- --thermal-demo --smoke-frames 360: PASS
  - cargo run -p powdergame-windows -- --thermal-demo --smoke-frames 3000: PASS
  - cargo run -p powdergame-windows -- --pressure-demo --smoke-frames 200: PASS
  - cargo run -p powdergame-windows -- --pressure-demo --smoke-frames 500: PASS (device lost 0, clean exit)

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

1. **G0 / G1 / G2 / G3 / G4 / G5**: **ALL PASS / CLOSED** (G2/G3/G4/G5 User Validation ALL APPROVED).
2. **Next Gate: G6 — Parallel Integrity** (Active chunk tracking, boundary synchronization, thread safety under full GPU load).

---

### Blockers

현재 문서/설계 기준으로 known hard blocker 없음.

---

## Approval State

Foundation Design direction: **APPROVED BY USER**

M0 implementation: **IN_PROGRESS** — G0/G1/G2/G3/G4/G5 PASS / CLOSED (G2/G3/G4/G5 User Validation APPROVED); Next: G6 — Parallel Integrity.

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
tests: passed (130 core + 56 GPU combustion/decay + 15 GPU density + 5 GPU expansion + 1 GPU headless smoke + 16 GPU movement + 16 GPU phase + 8 GPU pressure + 7 GPU rupture/stress-lab + 13 GPU thermal + 7 GPU world integrity + 4 windows observatory + 1 wgsl parse = 279 total, ignored 1 controlled benchmark)
benchmarks: G2 controlled reference-world baseline: median ~0.146 ms/tick (~6838 TPS, RTX 5090/DX12). G3/G4/G5: correctness-first.
m0_status: IN_PROGRESS (G0 complete, G1 complete, G2 PASS/CLOSED, G3 PASS/CLOSED, G4 PASS/CLOSED, G5 PASS/CLOSED User Validation APPROVED 2026-08-16; G6-G9 pending)
```
