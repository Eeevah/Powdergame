# Powdergame Status

이 문서는 현재 실제 상태를 기록한다. 장기 방향은 `ROADMAP.md`, 완료 기준은 `MILESTONES.md`를 따른다.

---

## Human-maintained Status

### Current Milestone

`M0 — First World`

### Current Milestone Status

`IN_PROGRESS` — G0 (Runtime) PASS, G1 (World Integrity) PASS, G2 (Local Movement) PASS / CLOSED, G3 (Density / Displacement) PASS / CLOSED, G4 (Thermal / Phase / Combustion) PASS / CLOSED (User Validation APPROVED on 2026-08-16), G5 (Pressure Chain) PASS / CLOSED (G5 User Validation APPROVED on 2026-08-16), G6 (Parallel Integrity) PASS / CLOSED (G6 User Validation APPROVED on 2026-08-16; G6-A TECHNICAL PASS / FROZEN, G6-B TECHNICAL PASS / FROZEN, G6-C1 COMPLETE / FROZEN, G6-C2 TECHNICAL PASS / FROZEN). G7 (Active/Sleep) IN_PROGRESS — G7-A (Chunk Activity Observatory / Measurement Baseline) VALIDATION candidate; G7-B/C PLANNED.

### Current Phase

**G0 — Runtime: PASS. G1 — World Integrity: PASS. G2 — Local Movement: PASS / CLOSED. G3 — Density / Displacement: PASS / CLOSED. G4 — Thermal / Phase / Combustion: PASS / CLOSED (User Validation APPROVED 2026-08-16). G5 — Pressure Chain: PASS / CLOSED (2×2 Multi-Boiler Stress Lab User Validation APPROVED 2026-08-16). G6 — Parallel Integrity: PASS / CLOSED (G6-A TECHNICAL PASS / FROZEN; G6-B TECHNICAL PASS / FROZEN; G6-C1 COMPLETE / FROZEN; G6-C2 TECHNICAL PASS / FROZEN; G6 User Validation APPROVED 2026-08-16). G7 — Active/Sleep: IN_PROGRESS (G7-A Chunk Activity Observatory / Measurement Baseline: VALIDATION candidate — measurement/visualization baseline만, 실제 sleep/work-skip 없음; G7-B/C PLANNED).**

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

2026-08-16 기준 **G0 (Runtime)**, **G1 (World Integrity)**, **G2 (Local Movement)**, **G3 (Density / Displacement)**, **G4 (Thermal / Phase / Combustion)**, **G5 (Pressure Chain)**, **G6 (Parallel Integrity)**가 구현·검증·사용자 검증 승인 완료되어 **PASS / CLOSED** 되었다.

**G6 — Parallel Integrity** 최종 상태:
- **G6-A (GPU Write Ownership Audit)**: **TECHNICAL PASS / FROZEN**
- **G6-B (Ownership Contention Integrity)**: **TECHNICAL PASS / FROZEN**
- **G6-C1 (Arbitration Quality Measurement)**: **COMPLETE / FROZEN**
- **G6-C2 (Stateless Edge Hash Production Integration)**: **TECHNICAL PASS / FROZEN** (프로덕션 edge-hash 도입 완료, 0 atomics / 0 writable storage aliases 유지, 방향 편향 제거, RTX 5090 성능 오버헤드 0.0% 검증 완료).
- **G6 User Validation**: **APPROVED — 2026-08-16** (사용자가 G6 Parallel Integrity Lab을 tick 0 / ~161 / ~501 / ~1016 / 36724(FAST x16)까지 직접 관찰 후 최종 승인)
- **G6 전체**: **PASS / CLOSED**

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

2×2 Multi-Boiler Stress Lab (`--pressure-demo`)의 대칭 실험(Panel C vs D) 및 장기 관측을 통해 사용자 직접 승인을 획득하여 `PASS / CLOSED` 되었다.

---

### G6 Parallel Integrity Technical Evidence — RTX 5090 / DX12 (2026-08-16)

#### G6-A GPU Write Ownership Audit — TECHNICAL PASS
- **Production Pass Inventory (14 Shaders)**: `movement_propose`, `movement_claim`, `movement_commit`, `thermal`, `phase_transition`, `expansion_claim`, `expansion_spawn_commit`, `expansion_pressure`, `decay`, `combustion`, `smoke_claim`, `smoke_commit`, `pressure`, `rupture`.
- **Structural Invariant Audit**:
  - `SELF_WRITE` 위반: **0건 (NONE)**.
  - 직접 neighbor 변조(Direct neighbor mutation): **0건 (NONE)**.
  - 아토믹(`atomic*`), 스핀락, `var<workgroup>` 공유 메모리: **0건 (NONE)**.
  - 전역 정렬(Global sort), 우선순위 큐: **0건 (NONE)**.
  - 셀당 영속 RNG 상태: **0건 (NONE)**.
  - 후보 선택 국소성(First-Match Locality): 1-cell 국소 스텐실 및 8-이웃 국소 후보만 사용.
- **Naga AST 구조적 검증 테스트**: `test_all_production_wgsl_write_contracts_and_binding_safety` (`PASS`).

#### G6-B Ownership Contention Integrity — TECHNICAL PASS
- **Movement Contention**:
  - 다중 소스 $\to$ 단일 EMPTY 대상 충돌 시 정확히 1개 소스만 승리, 패자 소스는 보존, 전체 물질수 보존 (`test_movement_many_sources_one_empty_target_exactly_one_winner`).
  - 이동 체인 $A \to B \to C$에서 단일 셀은 틱당 최대 1개 에지만 참여, 복제/소실 없음 (`test_movement_chain_cell_joins_at_most_one_edge`).
  - 64-cell 청크 경계 충돌이 내부 충돌과 완전히 동일하게 거동 (`test_movement_contention_across_chunk_boundary_single_winner`).
  - 200틱 밀집 반복 충돌 환경에서 완전 보존 (`test_movement_repeated_contention_long_run_preserves_world_integrity`).
- **Expansion Contention**:
  - 다중 비등 소스 $\to$ 단일 EMPTY 확장 대상 충돌 시 정확히 1개 증기만 추가 생성, 패자는 가둠 압력($P \ge 100.0$) 생성, 소스 상전이 정상 완결 (`test_expansion_contention_many_boiling_sources_one_empty_target`).
- **Smoke Spawn Contention**:
  - 다중 연소 목재 $\to$ 단일 EMPTY 연기 대상 충돌 시 정확히 1개 연기 생성, 생성된 연기 나이는 0부터 시작, 소스 목재 보존 (`test_smoke_spawn_contention_multiple_burning_sources_one_empty_target`).
- **Scratch Buffer Reuse Boundary**:
  - `movement` $\to$ `expansion` 및 `expansion` $\to$ `combustion/smoke` 순차 재사용 경계에서 모든 invocation이 `proposal` 및 `claim` 슬롯을 무조건 완전 overwrite (`test_expansion_scratch_reuse_after_movement`, `test_smoke_scratch_reuse_after_movement_and_expansion`). 이전 서브시스템 잔존 데이터 유출 0건.
- **Heavy Mixed Integrity Stress**:
  - 5개 상이한 물리 구역(Sand/Water hopper, Oil/Water density, Burning Wood/Smoke, Boiling Boiler, Melting Ice)을 포함하는 64×64 월드에서 300틱 연속 병렬 스트레스 실행 (`test_mixed_integrity_stress_long_run`). 모든 셀 ID 유효, EMPTY 위생 완벽 유지, $T/P$ 유한성 확인, 디바이스 손실 0건 (`PASS`).

#### G6-C1 Arbitration Quality Measurement — MEASUREMENT COMPLETE / FROZEN
- **측정 대상**: Frozen Fixed-Index Baseline vs Test-Only Stateless Edge-Hash Candidate (`edge_priority(source, target_cell, tick)`).
- **편향(Bias) 비교 결과 (2,048 Contests per orientation)**:
  - 수평 충돌 (Left vs Right): Baseline 100% Left vs 0% Right $\to$ Candidate **49.3% Left vs 50.7% Right** (편향 완전 해소).
  - 수직 충돌 (Up vs Down): Baseline 100% Up vs 0% Down $\to$ Candidate **50.0% Up vs 50.0% Down** (완전 균등).
  - 대각 충돌 (NW vs SE): Baseline 100% NW vs 0% SE $\to$ Candidate **51.2% NW vs 48.8% SE**.
  - 회전(0° Left vs Up): Candidate **53.5% Left vs 46.5% Up**.
  - 틱 시드(64 ticks): Candidate **51.6% Left vs 48.4% Right** (영구 고정 없음).
  - 에지 상호 합의(Edge Agreement): 100% 상호 일치 (분할 소유권 0건).
  - 결정론적 재현성: 100% bit-exact.
- **RTX 5090 / DX12 성능 마이크로벤치마크 (2048×2048 = 4,194,304 셀)**:
  - Sparse (현실적 5% 충돌): Baseline 0.0412 ms vs Candidate 0.0412 ms (**Delta: -0.18%**, 오차 범위 내 동일).
  - Heavy (100% 밀집 충돌): Baseline 0.0396 ms vs Candidate 0.0412 ms (**Delta: +0.0015 ms / +3.86% claim-only**, 전체 틱 기준 **~1.0%** 수준).
- **공식 권고**: **`HASH CANDIDATE WORTH INTEGRATING`** (G6-C2 채택 검토 단계로 진입 대기).

---

### Known Artifacts & Deferred Items

1. **GAS Triangular / Wedge Plume Artifact (Deferred)**:
   - 결정론적 1-cell 국소 GAS 스텐실 및 굴뚝 가이드에 따른 자연스러운 거시적 형상이며, 물리 보존 및 인과율에 결함 없음. 후속 프레젠테이션/가스 흐름 폴리시 단계로 분류.
2. **Column-Like Vent Shape & Corner Accumulation (Deferred)**:
   - 좁은 굴뚝 및 밀폐 챔버 모서리에서 증기가 상승하여 집적될 때 국소 충돌 순서에 따른 칼럼 형상 발생.
3. **Generic Decay Scope (Deferred Expansion)**:
   - 현재 generic `DecayDescriptor { lifetime_ticks, target_material }`는 `SMOKE -> EMPTY` 케이스로 완결 검증됨.
4. **Modern Presentation FX Layer (Deferred)**:
   - 연속 유체 렌더링, 스크린 스페이스 파티클 블룸, 열기 왜곡(heat haze), 매끄러운 연기 트레일 등은 시뮬레이션 진실(cell truth)과 분리된 최종 프레젠테이션 계층으로 추후 구현.
5. **Ash / Soot (Deferred)**:
   - 연소 잔여물 매커니즘은 후속 마일스톤으로 보류.

---

### Automated Test Evidence Summary (Total 299 Tests)

```text
cargo test --workspace -- --test-threads=1 전체 PASS (299 passed, 0 failed, 2 ignored performance benchmarks)
  - Core unit tests: 130 passed
  - GPU arbitration quality tests (G6-C1): 6 passed
  - GPU combustion & decay tests: 56 passed
  - GPU density displacement tests: 15 passed
  - GPU expansion tests: 5 passed
  - GPU headless smoke test: 1 passed
  - GPU local movement tests: 15 passed (2 perf benchmarks ignored — coarse + controlled, manual runs only)
  - GPU parallel integrity tests (G6-A/B/C2): 12 passed
  - GPU phase transition tests: 16 passed
  - GPU scalar pressure tests: 8 passed
  - GPU rupture & 2x2 multi-boiler stress lab tests: 7 passed
  - GPU thermal conduction tests: 13 passed
  - GPU world integrity tests: 7 passed
  - Windows observatory & pressure lab unit tests: 7 passed
  - WGSL syntax parse tests: 1 passed

Runtime Smoke Test Suite (RTX 5090 / DX12):
  - cargo run -p powdergame-windows -- --smoke-frames 60: PASS
  - cargo run -p powdergame-windows -- --movement-demo --smoke-frames 120: PASS
  - cargo run -p powdergame-windows -- --density-demo --smoke-frames 180: PASS
  - cargo run -p powdergame-windows -- --thermal-demo --smoke-frames 360: PASS
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

#### G6 Observation Hardening — `--parallel-integrity-demo` (2026-08-16)

사용자 1차 관찰(tick 0/62/347/552/1015/2193) 후 관찰 fixture를 수정했다.

- **C 패널 재설계 — one-tick ownership instrument**: 좌=EXPANSION CONTENTION (3 boiling Water sources → 공유 EMPTY 1개, 다른 후보 전부 Stone 차단), 중앙=movement fixture (Sand 1셀 낙하), 우=SMOKE CONTENTION (3 burning Wood sources → 공유 EMPTY Smoke target 1개). 첫 tick 후 **실제 GPU readback**으로 latch: `candidates=3 / winners=1 / steam_sources=3/3 / pressure_losers=2 / target=STEAM` (expansion), `candidates=3 / winners=1 / wood_preserved=3/3 / smoke_age=0 / target=SMOKE` (smoke), `movement_done=true / scratch_reuse=true / result=PASS` (`[powdergame][G6-C] latch @tick 1` 로 stdout 증거). latch는 첫 post-tick blocking snapshot으로 정확히 tick 1 상태를 보존 (async readback latency로 smear되지 않음).
- **A/B/D HUD — 실제 readback 기반**: A(closed) initial/live/delta + invalid IDs, B(closed) 동일 + seam crossings observed(x 191/192, y 63/64), D는 **count-delta를 loss로 오표기하지 않고** integrity violations만 표시 (invalid IDs, NaN/Inf T/P, negative P, EMPTY T/flags/P hygiene). 더미/하드코딩 PASS 제거.
- **G3 legacy overlay 제거**: `PresentationPalette::Integrity` 신설 — Lab-style cell colors + screen-space font HUD만 (procedural G3 lab text 없음).
- **Fast-forward**: F key 1x/4x/16x 순환 (G6 전용), N은 항상 정확히 1 tick (multiplier 무관), R은 world+metrics reset + 1x 복귀. title/HUD에 `FAST xN` + measured sim TPS 표시. readback cadence는 fast multiplier에 따라 5→12→30 tick으로 확대.
- **Test loop 최적화**: `coarse_reference_world_perf`(2048×2048 + PollType::Wait)를 `#[ignore = "manual performance sanity..."]`로 전환 — 반복 개발 validation에서 제외. `controlled_reference_world_perf`는 기존대로 ignored 유지.
- **Validation policy 명문화** (`docs/development/DEVELOPMENT.md` §11): FAST ITERATION(fmt+check+targeted tests) / FULL CHECKPOINT(기능 라운드 종료 1회: `cargo test --workspace -- --test-threads=1` + clippy + smoke matrix) / PERFORMANCE(명시 요청 또는 G8 Gate에서만).
- FAST validation PASS: fmt, `cargo check --workspace --all-targets`, `parallel_integrity` 12 passed, `powdergame-windows` 7 passed, `--parallel-integrity-demo --smoke-frames 300` exit 0 + G0/G2/G3/G4 smoke regressions exit 0. **Physics/engine 파일 변경 0건.**
- **G6 최종 User Validation — APPROVED (2026-08-16)**: 사용자가 G6 Parallel Integrity Lab을 tick 0 / ~161 / ~501 / ~1016 / **36724 (FAST x16)** 까지 직접 관찰하고 최종 결과에 만족하여 진행/승인 명시.
  - **Panel A — Movement Contention**: Matter live=562 / initial=562 / Δ=+0, winner exactly-one PASS, losers valid (DELTA 0), invalid IDs=0 — tick 36724 장기 실행에서도 유지.
  - **Panel B — Chunk Boundary**: Matter live=1712 / initial=1712 / Δ=+0, invalid IDs=0 — 사용자 직접 관찰 "경계선에서도 없어지지 않아". crossings observed는 시점별 live diagnostic (대표 ~36 / ~30 / ~28, 누적 monotonic counter로 과장하지 않음).
  - **Panel C — Expansion + Smoke Ownership**: one-tick instrument latched 실 GPU readback — Expansion candidates=3 / winners=1 / steam_sources=3/3 / pressure_losers=2 / target=STEAM; Smoke candidates=3 / winners=1 / wood_preserved=3/3 / smoke_age=0 / target=SMOKE; movement ran (1 cell) / scratch reuse / result=PASS.
  - **Panel D — Heavy Mixed Long-Run Stress**: FAST x16 tick≈36724 — invalid IDs=0, NaN/Inf T=0, NaN/Inf P=0, negative P=0, EMPTY T/flags/P hygiene violations=0 → **ALL INTEGRITY OK**. Matter live 6096→5696 변화는 의도된 생성/소멸(expansion / combustion→EMPTY / smoke spawn/decay)로 failure가 아님 — D는 count-conservation fixture가 아니라 heterogeneous long-run state-integrity fixture (A/B가 closed conservation 담당).
  - **FAST x16**: RTX 5090에서 원활, tick 36724까지 장기 stress 관찰 가능. observed sim rate ≈ 960 TPS는 **G6 demo fast-forward 동작 관찰값** — G8 공식 performance benchmark가 아니며 성능 claim으로 확대하지 않음.
  - 최종 화면: SIM TICK ≈ 36724 / DIAGNOSTIC SAMPLE ≈ 36708 / FAST x16.

## G7 — Active / Sleep

### G7-A — Chunk Activity Observatory / Measurement Baseline (TECHNICAL 구현 완료, VALIDATION candidate)

- **철학**: Dense State, Sparse Work — dense SoA storage 유지, Cell state를 sparse container로 바꾸지 않음. Matter count가 아니라 changeable frontier가 계산 필요성을 결정.
- **Chunk activity state**: GPU-side 64×64 chunk 기준 diagnostic 버퍼 3종 (`chunk_activity` bitmask / `chunk_changed` / `chunk_stable`). bit: `ACTIVITY_MATTER=1<<0`, `ACTIVITY_THERMAL=1<<1`, `ACTIVITY_PRESSURE=1<<2`, `ACTIVITY_REACTION=1<<3` (engine/core/src/activity.rs). 진단 state이지 per-cell simulation state 아님.
- **Same-Matter no-op audit**: movement_propose — NORMAL move는 EMPTY만 허용, density는 rank ordering 필수 (equal rank는 절대 swap 안 함), lateral은 EMPTY-only → Water↔Water/Oil↔Oil 등 무의미한 same-ID ownership edge 경로 없음. regression test `same_matter_noop_does_not_create_false_activity` 추가.
- **Stable-duration**: `chunk_stable`는 consecutive-stable-ticks 관찰용 (sleep cutoff로 사용하지 않음; threshold 임의 선택 금지). meaningful change 시 `chunk_changed`로 reset.
- **Wake reason model**: SELF_CHANGED / NEIGHBOR_ACTIVE / EDIT / PHASE_CHANGE / REACTION / THERMAL_FRONT / PRESSURE_FRONT — 진단 aggregate 수준.
- **GPU passes**: `activity_propose.wgsl`(cell bit 평가) + `activity_reduce.wgsl`(chunk reduction, seam 처리 포함 — chunk가 activity wall이 되지 않음). tick 끝에 read-only 진단. parallel_integrity write-contract / wgsl_parse 등록.
- **--activity-demo**: 256×256 (4×4 chunks), 60 TPS. 2×2 panel — [A] STABLE WATER BULK / [B] STABLE STEAM/GAS BULK / [C] WAKE PROPAGATION / [D] SLOW ACTIVE WORLD. G4~G6 screen-space HUD (SIM TICK / DIAGNOSTIC SAMPLE / Total Chunks / Matter·Thermal·Pressure·Reaction Active / Fully Stable / Max Stable Ticks) + chunk activity heatmap overlay.
- **Automated tests**: engine/gpu/tests/activity.rs **15 passed** (baseline + false-sleep hazard fixture: stable Water에 Sand 접근, stable Steam에 thermal frontier 접근, ignition heat 접근 등).
- **FAST validation PASS**: fmt / `cargo check --workspace --all-targets` (warning 0) / activity 15 passed / windows 7 passed / `--activity-demo --smoke-frames 300` exit 0 / `--smoke-frames 60` (marker=1) / `--density-demo --smoke-frames 180` exit 0. **성능 benchmark 실행 안 함** (G8이 공식 Performance Gate).

### G7-A semantic hardening (후속, `fix: harden G7 activity semantics`)

- **코드 vs 문서 불일치 감사 완료**: (A) THERMAL activity에 phase candidate 의미 추가 구현, (B) evidence 문서의 "reduce shader가 neighbor chunk activity를 참조하는 seam 처리" 문구 정정 — 실제 구조는 **cell-level stencil이 world 좌표로 seam 반대편을 읽는 것**이고, dedicated chunk-to-chunk wake propagation은 **없음** (G7-B에서 actual sleep과 함께 구현).
- **Phase false-sleep hazard 보강**: 1:1 write-self phase는 rule이 성립하면 같은 tick에 변환되고 hysteresis가 변환 후 상태를 안정으로 보장하므로 end-of-tick 상태에 "대기 중인 phase candidate"는 원리상 존재하지 않는다. 실제 관측 신호로 **phase pass가 transition tick을 `cell_activity`에 THERMAL self-marker로 기록** (propose는 OR-merge, 매 tick clear) 하고, detector는 phase table을 바인딩해 **phase-condition 방어적 체크**도 수행. physics semantics 변경 0.
- **Phase zero-gradient tests (GPU, 전 세계 균일 T — gradient 0)**: `uniform_water_above_boil_threshold_reports_thermal_active` / `uniform_steam_below_condense_threshold_reports_thermal_active` / `uniform_water_below_freeze_threshold_reports_thermal_active` / `uniform_ice_above_melt_threshold_reports_thermal_active` PASS — THERMAL이 phase transition 때문에만 발생. negative: `uniform_water_inside_phase_hysteresis_without_gradient_can_be_inactive` PASS (activity 0, stable 증가).
- **Cross-chunk**: `cross_chunk_thermal_frontier_detected` / `cross_chunk_pressure_frontier_detected` — seam x=63/64 양쪽 chunk 모두 감지.
- **Pressure-medium audit**: PRESSURE activity를 pressure-medium(LIQUID/GAS) cell로 제한 (G5 계약 정합; EMPTY/STATIC/POWDER는 field가 매 tick 0). `non_medium_cells_do_not_report_pressure_activity` PASS — Stone-only chunk가 이웃 pressured Water 때문에 오보되지 않음. false-negative 위험 없음 (pressure work는 항상 medium에서 발생).
- **`chunk_changed` 의미 명시**: "이번 tick에 activity(frontier) 존재" = stable counter reset 원인. 이전/다음 state 비교 dirty tracking이 아님 (필요 시 G7-B 별도 설계).
- **Automated tests**: activity **23 passed** (15 baseline + 8 hardening). phase 16 / wgsl_parse 1 / parallel_integrity 12 (write contract에 phase `cell_activity` read-write 추가) / windows 7 전부 PASS. FAST validation + `--activity-demo --smoke-frames 300` / `--smoke-frames 60` / `--density-demo --smoke-frames 180` exit 0. **성능 benchmark 실행 안 함.**
- **한계**: 실제 work skipping / sleep cutoff / active-list compaction / indirect dispatch 없음 — measurement baseline.
- Evidence: `docs/evidence/G7_A_ACTIVITY_BASELINE_2026-08-16.md`

### Next Action

1. **G7-A — Chunk Activity Observatory**: **VALIDATION candidate** — 사용자가 `--activity-demo`를 직접 보고 A/B/C/D panel과 stable-duration 분포를 확인 (G7-A는 measurement baseline; G7 PASS/CLOSED 아님)
2. **G7-B — Sleep/Wake correctness** (wake reason 기반, false-sleep 방지, stable-duration 관찰 결과를 바탕으로 cutoff 설계)
3. **G7-C — active-list compaction / indirect dispatch** (성능 측정은 G8 공식 Gate에서)

---

### Blockers

현재 문서/설계 기준으로 known hard blocker 없음.

---

## Approval State

Foundation Design direction: **APPROVED BY USER**

M0 implementation: **IN_PROGRESS** — G0/G1/G2/G3/G4/G5/G6 PASS / CLOSED (G2/G3/G4/G5/G6 User Validation APPROVED 2026-08-16); G7 Active/Sleep **IN_PROGRESS** (G7-A VALIDATION candidate).

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
tests: passed (136 core + 6 GPU arbitration + 56 GPU combustion/decay + 15 GPU density + 5 GPU expansion + 1 GPU headless smoke + 15 GPU movement + 12 GPU parallel integrity + 16 GPU phase + 8 GPU pressure + 7 GPU rupture/stress-lab + 13 GPU thermal + 7 GPU world integrity + 7 windows observatory + 1 wgsl parse + 23 GPU activity = 328 total, ignored 2 performance benchmarks [coarse + controlled]; full-workspace re-collection은 다음 FULL CHECKPOINT에서)
benchmarks: G6-C2 full-tick 2048x2048: median 0.8426 ms/tick (1186.8 TPS, RTX 5090/DX12). G2 controlled baseline: median ~0.146 ms/tick.
m0_status: IN_PROGRESS (G0-G6 PASS/CLOSED User Validation APPROVED 2026-08-16; G7 IN_PROGRESS — G7-A VALIDATION candidate; G8/G9 pending)
```
