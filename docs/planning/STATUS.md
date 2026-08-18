# Powdergame Status

이 문서는 현재 실제 상태를 기록한다. 장기 방향은 `ROADMAP.md`, 완료 기준은 `MILESTONES.md`를 따른다.

---

## Human-maintained Status

### Current Milestone

`M0 — First World`

### Current Milestone Status

`IN_PROGRESS` — G0 (Runtime) PASS, G1 (World Integrity) PASS / CLOSED, G2 (Local Movement) PASS / CLOSED, G3 (Density / Displacement) PASS / CLOSED, G4 (Thermal / Phase / Combustion) PASS / CLOSED (User Validation APPROVED on 2026-08-16), G5 (Pressure Chain) PASS / CLOSED (G5 User Validation APPROVED on 2026-08-16), G6 (Parallel Integrity) PASS / CLOSED (G6 User Validation APPROVED on 2026-08-16), G7 (Active / Sleep) PASS / CLOSED (G7 User Validation APPROVED on 2026-08-17). Current gate: G8 — Performance Evidence (IN_PROGRESS). G8-A v5 official capture와 independent verification은 완료되어 verified evidence candidate가 성립했고, 같은 SHA의 user visual validation은 pending이다. Canonical Recovery는 local integration branch에서 완료·검증했다. G8-B Scenario 1 Sand Fall은 사용자 승인되었고 immutable Harness pilot은 automatic `PASS` / review `APPROVED`다. Scenario 2 Water remediation candidate는 source `5af031f1a04af866127616d4f1b0faa6c85e4d8e`에서 automatic `NEEDS_HUMAN_REVIEW`를 유지한 채 human `ACCEPTED WITH KNOWN FOLLOW-UP`로 승인되었다. 알려진 후속 과제는 M0 local-liquid free-surface의 소수 셀 지속 재배열이며 production-physics defect 증거는 없다. Scenario 3 Fire / Heat는 tested source `1635fdb9f562192123c92846e137b125c684ede9`의 exactly one sealed candidate automatic `PASS`와 독립 검증 불일치 0을 보존한 채 **USER ACCEPTED**로 승인되었다. Scenario 4 Pressure Burst는 source `43e19d0f3b43aa0c15bf31e98f6401ba5f885170`의 immutable candidate에서 automatic `NEEDS_HUMAN_REVIEW`를 유지한 채 **USER ACCEPTED WITH KNOWN FOLLOW-UP**로 승인되었다. **Scenarios 1–4 are USER ACCEPTED**. Cell Inspector v0 tested source `3c342d25099683df53e303d1920cebe1f6578b74`는 **USER ACCEPTED WITH KNOWN FOLLOW-UP**다. Scenario 5 Heavy Mixed World는 **NEXT / IN PROGRESS 진입 가능**하지만 아직 user accepted가 아니며 **G8-B 전체는 NOT CLOSED**이고, G8-C는 별도 pending gate다.

### Current Phase

**G0 — Runtime: PASS. G1 — World Integrity: PASS / CLOSED. G2 — Local Movement: PASS / CLOSED. G3 — Density / Displacement: PASS / CLOSED. G4 — Thermal / Phase / Combustion: PASS / CLOSED. G5 — Pressure Chain: PASS / CLOSED. G6 — Parallel Integrity: PASS / CLOSED. G7 — Active / Sleep: PASS / CLOSED (G7-A USER VALIDATED / FROZEN; G7-B PASS / CLOSED / FROZEN; G7-C DEFERRED OPTIMIZATION). Current Gate: G8 — Performance Evidence (IN_PROGRESS; G8-A VERIFIED V5 EVIDENCE CANDIDATE, USER VISUAL VALIDATION PENDING; G8-B SCENARIOS 1–4 USER ACCEPTED, WATER AND PRESSURE AUTOMATIC NEEDS_HUMAN_REVIEW UNCHANGED / KNOWN FOLLOW-UP, FIRE AUTOMATIC PASS UNCHANGED, CELL INSPECTOR V0 USER ACCEPTED WITH KNOWN FOLLOW-UP, SCENARIO 5 HEAVY MIXED NEXT / IN PROGRESS ENTRY ALLOWED, OVERALL NOT CLOSED; G8-C PENDING).**

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

2026-08-17 기준 **G0 (Runtime)**, **G1 (World Integrity)**, **G2 (Local Movement)**, **G3 (Density / Displacement)**, **G4 (Thermal / Phase / Combustion)**, **G5 (Pressure Chain)**, **G6 (Parallel Integrity)**, **G7 (Active / Sleep)**가 구현·검증·독립 레드팀 재검토·사용자 검증 승인 완료되어 **PASS / CLOSED** 되었다.

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

### G7-A observatory fixture hardening (`fix: harden G7 activity observatory fixture`)

이 라운드는 사용자 장기 관찰(tick ~0/117/411/679/1515/3019)에서 확인된 **HUD 누락 / fast-forward 회귀 / detector correctness mismatch / fixture 결함**을 수정한다. authoritative contract: `docs/planning/G7_A_OBSERVATORY_HARDENING.md` (historical observation은 그대로 보존).

- **Activity HUD root cause/fix**: `Renderer::new()`의 `needs_text_hud`가 `PresentationPalette::Activity`를 포함하지 않아 Activity 모드에서 TextRenderer가 생성되지 않았음 → 포함. 이제 SIM TICK / DIAGNOSTIC SAMPLE / global(Matter·Thermal·Pressure·Reaction Active, Fully Stable, Max Stable Ticks, Sampled Wake Candidates) / panel M/T/P/R / heatmap legend / controls가 실제로 렌더링된다.
- **Fast-forward 회귀 root cause/fix**: `request_fast_forward()`가 `ParallelIntegrity`만 허용 → `Activity`도 **F x1/x4/x16** 지원 (production `Simulation::tick()`을 순차 반복할 뿐, timestep/생략 없음). N은 항상 정확히 1 tick, R은 x1 reset. title/HUD/콘솔/BAT에 F 표기.
- **Stale activity bit root cause/fix**: propose가 `mask | cell_activity[index]`로 OR-merge → 이전 tick의 MATTER/PRESSURE/REACTION bit가 생존 가능. 수정: `cell_activity[index] = mask | (cell_activity[index] & ACTIVITY_THERMAL)` — phase pass의 이번-tick THERMAL transition marker만 보존하고 나머지는 매 tick overwrite. regression: `matter_frontier_clears_when_settled` / `pressure_frontier_clears_when_uniform` / `reaction_frontier_clears_when_extinguished` PASS (frontier 소멸 → bit clear → stable counter 재개).
- **THERMAL detector = frozen G4 정합**: conductivity table을 read-only binding으로 추가 (phase table과 단일 storage buffer로 결합 — DX12 per-stage storage-buffer limit 8 준수). thermal edge는 양쪽 모두 Matter이고 effective conductivity > 0일 때만 THERMAL work — **EMPTY와 Boundary Block(K=0)은 frontier가 아님**. regression: `hot_matter_next_to_empty_does_not_false_report_thermal` / `temperature_difference_across_boundary_block_is_inactive` / `conductive_stone_gradient_reports_thermal_active` / `cross_chunk_thermal_frontier_detected` PASS. `THERMAL_ACTIVITY_EPS` 변경 0, thermal.wgsl 수정 0.
- **PRESSURE detector = frozen G5 정합**: pressure-medium(LIQUID/GAS)↔medium만 비교 — Stone/EMPTY 경계는 frontier 아님. regression: `uniform_pressurized_medium_sealed_by_stone_is_not_pressure_frontier` PASS (기존 `non_medium_cells_do_not_report_pressure_activity`의 잘못된 기대값 정정), `pressure_gradient_reports_pressure_active` / `cross_chunk_pressure_frontier_detected` 유지. `PRESSURE_ACTIVITY_EPS` 변경 0, pressure.wgsl 수정 0.
- **Panel isolation**: 중앙 십자벽(x=127/128, y=127/128)을 Stone → **MATERIAL_BOUNDARY_BLOCK(K=0)** 으로 교체 — 네 실험이 thermal coupling되지 않음. G5/G6 fixture는 변경 없음. 3000-tick 검증에서 A control chunk mask 0 (cross-panel THERMAL contamination 0).
- **Panel B = TRUE stable Steam control**: Stone shell + Steam 모두 T=80, EMPTY interface / 압력 / reaction 없음 → 4 chunk가 mask 0, stable counter 단조 증가 (t=500/1000/3000 전부 확인). **Gas existence != Activity** 증명.
- **Panel C = STABLE DURATION / WAKE CANDIDATE**: Sand source를 upper-right C chunk(cx=1, cy=2)에만 배치, lower-right target chunk(cx=1, cy=3)는 먼저 stable → 생산 movement로 y=192 seam을 넘어 Sand 도착 → stable reset. G7-A에는 actual sleep/dedicated wake propagation이 없으므로 **wake-candidate 관측**으로 명명 (이전 "WAKE PROPAGATION" 표현 제거).
- **Sampled Wake Candidates**: `wake_events` 과장 표현 제거. sentinel `ACTIVITY_NO_PREV_SAMPLE`(u32::MAX)로 **첫 sample은 baseline만 설정**하고 이후 0→nonzero sampled transition만 카운트. R reset도 sampling baseline을 sentinel로 reset.
- **3000-tick actual-fixture test** (ignored, RTX 5090/DX12): `apps/windows/src/main.rs`의 실제 `stage_activity_demo()`를 3000+ production tick. 결과: **C pre-arrival stable=1 → first MATTER arrival tick=27 → reset_to_zero=true**; B 4 chunk late state 전부 mask 0 + stable ≥ 1000; A control chunk mask 0; device loss / panic 0.
- **Automated tests**: activity **29 passed** (23 + 6 hardening). thermal 13 / pressure 8 / phase 16 / wgsl_parse 1 / parallel_integrity 12 / windows 7 (+1 ignored long-run) 전부 PASS. `--activity-demo --smoke-frames 300` exit 0.
- **런처**: `run_g7_activity_demo.bat` (repository root, commit 대상) — `cd /d "%~dp0"` / `RUST_LOG=warn` / incremental release build / build 실패 시 메시지+`pause`+exit 1 / 로컬 `targetelease\powdergame-windows.exe --activity-demo` 직접 실행 / controls 표시. `cargo clean`·절대 경로·다른 checkout exe 금지 준수.
### G7 — Active / Sleep Summary (PASS / CLOSED / FROZEN — 2026-08-17)

- **G7-A (Chunk Activity Observatory)**: **PASS / USER VALIDATED / FROZEN**
  - 64×64 chunk activity detection baseline (`chunk_activity` bitmask, `chunk_changed`, `chunk_stable_ticks`).
  - EPS-guarded detector for Matter, Thermal, Pressure, Reaction frontiers.
  - Dense State, Sparse Work philosophy established.
  - Evidence: `docs/evidence/G7_A_ACTIVITY_BASELINE_2026-08-16.md` / `docs/evidence/G7_A_USER_VALIDATION_2026-08-17.md`

- **G7-B (Actual Sleep / Wake Correctness)**: **PASS / CLOSED / FROZEN**
  - All 14 physical compute passes guarded with `chunk_state` sleeping skip.
  - Dedicated `activity_wake.wgsl` pass evaluates 8-neighbor safety halo, edit-wake, settling duration.
  - Two-phase immutable edit-wake snapshot prevents cross-workgroup race conditions.
  - Sleeping guards carry forward exact state (frozen fuel progress, exact flags, sanitization).
  - Bulk world reset eliminates multi-second UI stalls during interactive reset.
  - Font atlas extended to all required HUD sizes (`12, 13, 14, 15, 16, 17, 18, 24`).
  - Independent Red-Team re-review: **REVIEW PASS**.
  - Windows / RTX 5090 / DX12 User Validation: **APPROVED (2026-08-17)**.
  - Validated Implementation: `638a27febd5f38a117c9f2a1dedac958446c8c20`
  - Evidence Anchor: `eb53b9195218108992e1c7c194e64d00e5163883`
  - Evidence: `docs/evidence/G7_B_SLEEP_WAKE_2026-08-17.md`

- **G7-C (Compaction / Indirect Dispatch)**: **DEFERRED PERFORMANCE OPTIMIZATION CANDIDATE**
  - Compact active lists, indirect dispatch, aggressive sparse dispatch are NOT part of the completed G7 correctness gate.
  - Reconsider only after G8 Performance Evidence demonstrates that full-grid dispatch / dispatch overhead is a meaningful bottleneck.

- **Automated Test Suite (357 discovered, 354 passed, 3 ignored, 0 failed)**:
  - `sleep_wake.rs` 17 passed (A~K scenarios + structural contracts).
  - `activity_demo_reset_exact_equivalence` + `test_font_atlas_contains_all_required_hud_sizes` passed.
  - G7-A 3000-tick actual-fixture long-run validation passed on RTX 5090 / DX12.
  - Full workspace tests: 354 passed, 3 ignored (2 manual perf + 1 long-run), 0 failed.

---

### G8 — Performance Evidence (IN_PROGRESS)

- **G8-A (Measurement Substrate)**: **V5 OFFICIAL CAPTURE + INDEPENDENT VERIFICATION COMPLETE / VERIFIED EVIDENCE CANDIDATE; USER VISUAL VALIDATION PENDING**
  - Headless benchmark harness (`powdergame-benchmark` at `apps/benchmark/`) with strict CLI validation and a dimension-safe calibration fixture.
  - Mode A uses a normal production context; Mode B uses a separate timestamp-enabled context on the same adapter.
  - Exact 17-pass timing, 34 raw queries per tick, timestamp-order validation, envelope/pass-sum/residual reconstruction, and per-tick grouped subsystem statistics.
  - Historical v4 Mode A values: P50 948.9 TPS (1.0538 ms/tick, 2048×2048 reference world, RTX 5090 / DX12). The packet does not bind these values to the later empty-submit fence source bytes.
  - Historical v4 Mode B values: Envelope P50 1.0204 ms, GPU Pass Sum P50 0.7663 ms, Residual P50 0.2542 ms. Mode B is per-tick synchronized and is not a sustained-throughput measurement.
  - Historical v4 aggregate census: 266,016 any-active cells, 219 active chunks, 381 runnable chunks, 643 sleeping chunks. The original three GPU arrays were not preserved, so these counts cannot be independently recounted from v4.
  - Historical v4 application-tracked requested persistent GPU buffer bytes: 184,576,672 with profiler buffers. This is not driver-reported resident VRAM.
  - Historical v4 three-way overhead control: per-tick synchronization +27.00%; profiling increment over synchronized control +1.77%; combined profiled path versus batch +29.25%.
  - v4 artifacts use schema v3: 129 aggregate rows plus 768 raw tick rows contain internally reconstructable timing data, but only commit SHA plus `dirty` state for source provenance.
  - Earlier v2/v3 calibrations are invalidated because pending reset/staging writes were not explicitly submitted before waiting.
  - The claim that v4 was the first empty-submit + completion-wait run is not established by its packet: the collected `main.rs` snapshot is later than the CSVs and no run-time source/diff or binary hash was recorded.
  - Schema v5 preserves exactly one row per raw cell and one row per raw chunk, recomputes the aggregate census from that snapshot, and stages/synchronizes four CSVs before raw-cell/raw-chunk/raw-tick/aggregate no-overwrite publication.
  - Official capture accepts only attached clean source, records source/binary/argv/log/exit/artifact hashes, writes the receipt last, and packages with a ZIP-external `PACKAGE_SHA256.txt`.
  - Receipt absence means incomplete capture. Existing v4 remains historical and is not replaced or corrected in place.
  - Verified v5 source: clean source/upstream `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`; capture `g8a-v5-9abec9e-20260817T032827206Z`; package SHA-256 `4b9f44f66c18235f80d33738d15f3418c65c98e68e254aa01d13fc3a66eb6ec8`.
  - Independent verification: success, 11/11 checks passed, zero findings. Raw evidence contains 4,194,304 cell rows, 1,024 chunk rows, 768 tick rows, and 129 aggregate rows.
  - Official v5 reference calibration: Mode A P50 865.304 TPS / 1.155663 ms per tick; profiled envelope P50 1.018080 ms. These are G8-A calibration facts, not G8-B/G8-C or product validation.
  - Historical provenance: external review activity occurred before the policy changed. It is not used as current status support. Adversarial review is explicit-request-only, and the one-time local report is a non-blocking historical record.
  - No production optimization performed; G7-C not implemented. G8-A calibration facts remain separate from the later G8-B fixture candidate and G8-C matrix.
  - Evidence: `docs/evidence/G8_A_MEASUREMENT_SUBSTRATE_2026-08-17.md`

- **G8-B (Benchmark Scenario Suite)**: **SCENARIOS 1–4 USER ACCEPTED; CELL INSPECTOR V0 USER ACCEPTED WITH KNOWN FOLLOW-UP; SCENARIO 5 HEAVY MIXED WORLD NEXT / IN PROGRESS ENTRY ALLOWED / NOT CLOSED**
  - Candidate and next work line: `feature/m0-g8b-scenario-suite`, based on local canonical recovery commit `ca79bb20b27041758ab4d4a224e491c171189393`; accepted Sand Fall scenario-suite checkpoint `e77d102febb1e3c497c2b669efe0140408bd99d7`. At docs closure it is ff-only aligned with the retained Harness branch.
  - Shared crate/API: `apps/scenarios` / `powdergame-scenarios::{ScenarioId, ScenarioFixture, validate_scenario_config, reset_and_stage_scenario}`.
  - Official fixtures: Sand Fall, Water Flow, Fire / Heat, Pressure Burst, Heavy Mixed World.
  - Scenario 1 Sand Fall: **USER ACCEPTED (2026-08-17)**. 사용자는 Sand가 완전히 정착하고 모든 chunk가 sleep 상태로 수렴하는 것을 성공으로 승인했다. benchmark identity를 위해 activity를 영구 유지하도록 source/geometry/sleep behavior를 retune하지 않는다.
  - Scenario 2 Water Flow: **USER ACCEPTED WITH KNOWN FOLLOW-UP (2026-08-17)**. Sealed run `g8b-water-flow-v0-20260817T110906547252Z-8b808e66`는 automatic `NEEDS_HUMAN_REVIEW`를 유지한다. Side-wall remediation 뒤 outer-basin Water max/final은 `0 / 0`, conservation/movement/cross-chunk/destination/integrity/reset은 pass했고, 마지막 active `64` cells는 Water/EMPTY `51`, Water/Oil `1`, other `12`였다. 알려진 후속 과제는 M0 local-liquid free-surface의 소수 셀 지속 재배열이며 production-physics defect 증거는 없다. 기존 first candidate는 immutable/superseded 상태로 보존한다.
  - Scenario 3 Fire / Heat: **USER ACCEPTED (2026-08-18)**. Tested source `1635fdb9f562192123c92846e137b125c684ede9`; exactly one candidate automatic `PASS`; independent artifact/raw recomputation found zero mismatch. Genuine Wood/Oil combustion, Smoke generation and decay to zero, Ice/Water/Steam phase work, finite fuel consumption, Reaction first/confirmed zero `11,448 / 11,464`, post-Reaction completion `11,644` with restart `0`, a remaining slightly decreasing Thermal tail, invalid/non-finite `0 / 0`, and exact reset `true` were accepted. Production physics change required: **NO**; candidate rerun required: **NO**.
  - Scenario 4 Pressure Burst: **USER ACCEPTED WITH KNOWN FOLLOW-UP (2026-08-18); AUTOMATIC `NEEDS_HUMAN_REVIEW` UNCHANGED**. Clean source/run: `43e19d0f3b43aa0c15bf31e98f6401ba5f885170` / `g8b-pressure-burst-v0-20260818T101046792957Z-17158748`. `pressure_opening_precedes_combustion` passed; seam combustion/flame/fuel progress through opening were `0`; first through opening was tick `7,216`, persistent opening and causal exterior Steam tick `7,232`, and adjacent Pressure `83.301017761` exceeded Wood rupture threshold `80`. Terminal mean/max was `53.460612654 / 145.603897095`, terminal mean declined `56.547848156 → 53.460612654`, runaway was false, invalid/non-finite was `0 / 0`, and exact reset was true. Known follow-up: top-seam-only opening, small persistent vent plume, broad terminal Pressure activity, and G8-C tail/activity workload-cost measurement. No production-physics defect is established. Scenario 5 Heavy Mixed World: **NEXT / IN PROGRESS ENTRY ALLOWED / NOT YET USER ACCEPTED**.
  - Sixth Gallery slot: exact `active-sleep-g7` 256×256×64 regression fixture. It is not a sixth official matrix workload.
  - Shared staging resets production `Simulation`, uploads authored Material/Temperature/Pressure/Flags to Current and Next plus `chunk_edit_wake`, and completes transfer work before measurement or inspection continues.
  - Windows `--benchmark-gallery` / `run_g8_benchmark_gallery.bat` starts paused and provides `1-6`, `SPACE`, `N`, `F`, `R`, `ESC` controls. Scenario selection and reset commit their new attribution only after shared reset/staging succeeds; failure is explicit and suppresses advancement/readback until recovery.
  - Gallery source SHA and dirty state are embedded at build time, so a reused executable cannot silently claim a later checkout as its build source.
  - Headless `--scenario` uses the same staging path before every prewarm, trial, and overhead-control window. Legacy `calibration` retains G8-A v5 identity/output; shared fixtures use scenario-scoped G8-B identity/output without changing CSV column shape.
  - Gallery rendering, HUD, wall-clock TPS, and sampled activity census are out-of-band inspection diagnostics. They are excluded from official timing and do not establish G8-C.
  - Scope exclusions: no production physics/shader/pass-graph change, no new Material, no G9 implementation, no performance optimization.
  - Recorded targeted checks in this implementation round: scenarios library 6/6; scenarios GPU reset 1/1; benchmark package 27/27; Windows Gallery-filtered tests 7/7; changed-package all-target clippy and workspace fmt passed; the final bounded 60-frame release Gallery launch exited cleanly on RTX 5090 / DX12 while remaining paused at tick 0 and reporting build-bound provenance.
  - No official five-scenario performance matrix, benchmark result, or broad demo matrix is claimed by the scenario-suite checkpoint. The later Sand Fall, Water Flow, Fire / Heat, and Pressure Burst Harness evidence is recorded separately below. User acceptance currently applies to Scenarios 1–4 only.
  - Evidence: `docs/evidence/G8_B_BENCHMARK_SCENARIO_GALLERY_2026-08-17.md`

- **G8-B Sand Fall Experiment Harness v0**: **PILOT PASS; HARNESS REVIEW OUTPUT APPROVED; G8-B NOT CLOSED**
  - Development line: `feature/g8b-experiment-harness-v0`, stacked on scenario-suite checkpoint `e77d102`.
  - Experiment source: `9e1fdac44aa14a546c7fe5ad6ceba49e71777eb5`; the later docs-only closure commit is not experiment provenance.
  - One-command entry: `run_experiment.bat sand-fall`; external root: `C:\Users\mdkap\source\Powdergame-artifacts`.
  - The worker uses only shared pristine Sand Fall staging and production ticks, records separate simulation-tick/diagnostic-sample identities, requires three consecutive all-sleep samples, checks 180 post-sleep ticks, and compares a programmatic `R`-equivalent reset to tick 0.
  - The coordinator uses unique/no-overwrite run directories and publishes telemetry, semantic renderer frames, derived screenshots/crops, reports, contact sheet, inert review prompt, packet, and hashes before writing `EXPERIMENT_RECEIPT.json` last. Generated artifacts never belong in Git.
  - Automatic `PASS` is restricted to the seven hard Sand Fall predicates. It does not close G8-B, approve Scenario 2–5, or establish G8-C.
  - Validated run: `g8b-sand-fall-v0-20260817T065311878587Z-3ebd7505`; automatic `PASS`; Harness review output `APPROVED`; first all-sleep simulation tick 1096; diagnostic sample sequence 139; confirmed tick 1112; 180 stable post-sleep ticks; exact reset true.
  - Erratum: preserved v0 field `first_all_sleep_diagnostic_sample_tick=139` is a deprecated-name alias for diagnostic sample sequence, not simulation tick. Correct simulation identities remain first `1096` / confirmed `1112`; artifact and verdict are unchanged.
  - Review Packet SHA-256: `feffc180f81d36558b8139f5436a2a0eed6422617dd9a3207153b1cb62af1323`; Receipt SHA-256: `42bbacf77ca80356996a53fb2d0a56a5aba18215b5b85bda637350522c95e033`.
  - Evidence contract: `docs/evidence/G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md`

- **G8-B Water Flow Experiment Harness v2 remediation**: **USER ACCEPTED WITH KNOWN FOLLOW-UP; AUTOMATIC NEEDS_HUMAN_REVIEW UNCHANGED; G8-B NOT CLOSED**
  - Active line/base: `feature/m0-g8b-scenario-suite` from `d12edbfbcc0fb3fc2ef599cd06b3c46a2293d268`; sealed remediation source SHA `5af031f1a04af866127616d4f1b0faa6c85e4d8e`.
  - Preserved candidate: `g8b-water-flow-v0-20260817T100732645294Z-f7ee7959`; automatic `NEEDS_HUMAN_REVIEW`; human `FIX REQUIRED — fixture_representativeness_issue`; no production-physics defect established.
  - The finite remediated fixture remains Water 15,244 / Oil 2,240 and keeps its 6,216-cell destination. Side walls become `[10,18) × [14,238)` and `[238,246) × [14,238)`; Stone is 8,104 and Empty 38,928.
  - Exactly one fresh candidate was generated after the FULL checkpoint and source seal: Run ID `g8b-water-flow-v0-20260817T110906547252Z-8b808e66`. Existing Run IDs/artifacts remain untouched; unique external create-new/no-overwrite, failure preservation, hashes, and receipt-last publication remain mandatory.
  - Water telemetry/analysis/report/receipt are v2; manifest v1 and frames v0 remain. The tenth predicate `water_outside_outer_basin_cells` requires a maximum of zero outside `[18,238) × [14,230)`. All-sleep and plateau policy are unchanged.
  - FAST recorded: workspace fmt/check PASS; scenarios library 7/7; shared GPU reset 1/1; bounded Water destination/conservation/leak/reset 1/1; Windows Sand/Water experiment 16/16; Python coordinator/analyzer 23/23.
  - Final checkpoint recorded: workspace tests PASS; workspace all-target clippy PASS; diff-check PASS; one 60-frame RTX 5090/DX12 Gallery release smoke PASS.
  - Candidate result: automatic `NEEDS_HUMAN_REVIEW` retained; human `ACCEPTED WITH KNOWN FOLLOW-UP`. Outer-basin Water max/final `0 / 0`; Matter/Water/Oil conservation, movement, cross-chunk, destination, invalid/non-finite integrity, and exact reset passed. Final active cells `64` = Water/EMPTY `51` + Water/Oil `1` + other `12`.
  - Review Packet SHA-256 `83783025ee6bdac8f6dedbf25edfec1dd75040d533c9fc563157cc699b5caec5`; Receipt SHA-256 `96f60b465dbfa4f7a4cacd7f78f475cad9af7c2e6d1754aba4dddc186d497c1b`.
  - Known follow-up: M0 local-liquid free-surface minority-cell persistent rearrangement. No production-physics defect evidence is established. The automatic verdict, all-sleep policy, fixture, liquid physics, Sleep/Wake semantics, Oil, Run ID, and generated artifacts remain unchanged.
  - Sand fixture/pilot/artifacts remain immutable. No production physics, Pressure/Mixed, G8-C, optimization, main merge, or PR is included.
  - Evidence contract: `docs/evidence/G8_B_WATER_FLOW_HARNESS_CANDIDATE_2026-08-17.md`

- **G8-B Fire / Heat Experiment Harness candidate**: **SEALED AUTOMATIC PASS; USER ACCEPTED; G8-B NOT CLOSED**
  - Active line: `feature/m0-g8b-scenario-suite`; sealed experiment source SHA `1635fdb9f562192123c92846e137b125c684ede9`.
  - Exactly one candidate run: `g8b-fire-heat-v0-20260817T133938546075Z-0e6aa901`. Whole-world all-sleep is not required; Reaction termination and the post-Reaction Thermal tail remain distinct.
  - Candidate metrics: Wood `10,926 → 0`; Oil `1,610 → 475`; fuel consumed `12,061`; first genuine combustion/Smoke tick `1 / 1`; first phase work tick `712`; Smoke peak `12,070 @ 7,864`; Reaction peak `12,997 @ 7,808`; Thermal peak `22,577 @ 6,696`.
  - Reaction zero first/confirmed ticks `11,448 / 11,464`; 180-tick post-Reaction window ended at `11,644` with restart samples `0`; Thermal activity `9,783 → 9,773`, sampled minimum `9,768`.
  - Invalid material/non-finite occurrences `0 / 0`; programmatic reset exact equivalence `true`; all twelve Fire predicates passed.
  - Final source sequence: workspace fmt/check and targeted suites PASS; workspace all-target clippy PASS; exactly one completed post-seal workspace test checkpoint PASS; one 60-frame RTX 5090/DX12 Gallery release smoke PASS; one Fire candidate PASS. One earlier pre-remediation FULL attempt was interrupted and is not the canonical checkpoint.
  - Review Packet SHA-256 `2a8e99d14bf0647b71e7ef32e3840655117e93b9f20ad1360af97d62a69eb940`; Receipt SHA-256 `ed17e75f7515d155f8b6e5a41a0aeb751b2876ec573658a6e49eb6dd72108aff`; sibling Audit Bundle SHA-256 `1c1df01dfa9004b9273bc45e4b01d3c784d5c377f98a9417bc0b7594c6a83706`.
  - Independent read-only verification found zero inventory, digest, source/binary, telemetry recomputation, image/contact-sheet, packet/bundle, or receipt-last mismatch. The user separately accepted Scenario 3 on 2026-08-18; the automatic verdict remains unchanged and distinct from that decision.
  - Acceptance closure is docs-only: Fire source, production physics, candidate, Review Packet, Audit Bundle, Receipt, and generated artifacts were neither changed nor rerun. Scenario 3 acceptance does not close G8-B.
  - Evidence contract: `docs/evidence/G8_B_FIRE_HEAT_HARNESS_CANDIDATE_2026-08-17.md`

- **G8-B Pressure Burst causal remediation**: **CLEAN-SOURCE CANDIDATE AUTOMATIC NEEDS_HUMAN_REVIEW UNCHANGED / USER ACCEPTED WITH KNOWN FOLLOW-UP / HEAVY MIXED PENDING — DO NOT START**
  - Immutable combustion-confounded first candidate source/run: `28e4e0bf7736fb95f0a9d7797cc3ab4274b91442` / `g8b-pressure-burst-v0-20260818T014452058676Z-353fb706`. Its automatic verdict remains `NEEDS_HUMAN_REVIEW`; human verdict is `FIX REQUIRED — fixture_causality_confounded_by_combustion`.
  - Valid observations remain Pressure activity, Wood damage, topology-aware persistent opening, exterior Steam, chamber-mean relief, no runaway, Water+Steam conservation, invalid/non-finite `0 / 0`, and exact reset. No production-physics defect is established.
  - Causal blocker: authored top relief Wood was `95 C`, above the `90 C` ignition threshold. Tick `896` still had top Wood `318` and through lanes `0` with Smoke/Reaction `3,682 / 4,000`; tick `904` had Wood `0` and all `48` top lanes open with Smoke/Reaction `3,541 / 3,541`. The complete opening coincided with the 900-active-tick combustion lifetime and did not prove Pressure-caused rupture.
  - The existing Run ID, Receipt, Review Packet, Audit Bundle, source, telemetry, screenshots, and reports are immutable. It is preserved as candidate review evidence and will be superseded only by a fresh unique candidate; it is not retrospectively changed to `PASS`.
  - First remediation scope is fixture-only: remove the authored top-seam heat while preserving geometry, Water/Steam, chamber temperature/Pressure, Stone, both seam dimensions, rupture thresholds, pass order, Sleep/Wake, and production physics. Pressure telemetry/analysis/report/receipt v1 gains subsystem-specific seam combustion/fuel evidence and `pressure_opening_precedes_combustion` before one permitted scratch; manifest/frames remain v0.
  - A new confounded scratch is classified as automatic `FIXTURE_CAUSALITY_CONFOUNDED`, not generic FAIL/NHR. Candidate publication is blocked unless causal classification is pressure-first and every hard predicate is non-failing; the prior v0 candidate's immutable automatic `NEEDS_HUMAN_REVIEW` is not rewritten.
  - Causal remediation scratch: `g8b-pressure-burst-v0-scratch-20260818T100203210143Z-a37e998c`; source HEAD `328757bd8cd5b07cd0ed4c66a592f973dcd66981`; `git_state=dirty`; `run_mode=scratch`; exact `SOURCE_INPUT_MANIFEST.json` SHA-256 `485c1a915afca65757c9994d0036381a92cc40d2134e293f617ead024edc966b`. The exact dirty tracked build-input bytes were sealed for the scratch; it is not clean-source candidate provenance.
  - Scratch result: causal classification `pressure_opening_precedes_combustion`; all ten hard predicates pass; failed hard predicates `[]`; `candidate_blocker=false`. Pressure activity/Wood damage first tick `1`; first through opening and raw outside Steam `7,216`; three-sample confirmation and confirmation-gated causal exterior-Steam milestone `7,232`; first relief `7,233`; no reseal.
  - Causal guard: first seam combustion/fuel progress `None / None`; combusting/flame/fuel sum/fuel max through confirmation all `0`. First-opening adjacent Pressure max `83.301017761` across `73` pressure-medium cells exceeded Wood rupture threshold `80`.
  - Chamber mean/max moved from pre-opening peak `223.582887701 / 319.461242676` to post-opening `61.290115379 / 148.151290894` and terminal `53.460612654 / 145.603897095`. Terminal 64-sample mean slope was `-0.048798238`, runaway false; invalid/non-finite `0 / 0`; exact reset true.
  - Scratch automatic `NEEDS_HUMAN_REVIEW` is review-only: only one seam ruptured, terminal Pressure activity remained high, vent plume persisted, and terminal activity remained. These flags did not block the clean-source candidate. The old candidate remains immutable/human `FIX REQUIRED`.
  - Accepted immutable candidate: source `43e19d0f3b43aa0c15bf31e98f6401ba5f885170`; run `g8b-pressure-burst-v0-20260818T101046792957Z-17158748`; automatic `NEEDS_HUMAN_REVIEW` unchanged; human **USER ACCEPTED WITH KNOWN FOLLOW-UP**.
  - Candidate hashes: Review Packet `f1523a816e5c97ad15fbd53e910553f8bbe59c3c7b78cf7fd6089f7dc143c0eb`; Audit Bundle `6f8e4579afb7b3375e571b1481b46d95e0e8533b4e718e6c5bd3ca4d120e5bc8`; Receipt `c21374d5bef5e3d2233cc488c893829b5285988887bcbd35db00f3ae87f72a64`.
  - Candidate causal result: `pressure_opening_precedes_combustion` PASS; combustion/flame/fuel-progress through persistent opening all `0`; through opening `7,216`; confirmation and causal exterior Steam `7,232`; opening-adjacent Pressure `83.301017761 > 80`; terminal mean/max `53.460612654 / 145.603897095`; terminal mean window `56.547848156 → 53.460612654`; runaway false; invalid/non-finite `0 / 0`; exact reset true.
  - Known follow-up: only the top seam opened, a small vent plume persisted, and broad terminal Pressure activity remained. There is no production-physics defect evidence; G8-C will measure the tail/activity workload cost.
  - Evidence contract: `docs/evidence/G8_B_PRESSURE_BURST_HARNESS_CANDIDATE_2026-08-18.md`

- **Cell Inspector v0**: **USER ACCEPTED WITH KNOWN FOLLOW-UP; G8-B NOT CLOSED**
  - Immutable tested source SHA: `3c342d25099683df53e303d1920cebe1f6578b74`; prior silent-pending docs closure SHA: `6a521c07c23d9817f6abb9d3e1eab47758b5d5c4`. Docs-only acceptance closure는 두 provenance와 구분한다.
  - Compact hover는 canonical Material name을 보이고 `I`로 detailed Inspector를 켜며, v0 detail은 명시적 계약대로 Material `Name (ID)`, Temperature, Pressure, activity, Chunk state, sample simulation tick / diagnostic sequence / freshness를 표시한다.
  - Renderer와 physical-pixel cursor picking은 동일한 CPU `WorldViewport`의 letterbox origin/scale을 공유한다. Material/Temperature/Pressure/flags/Cell activity/Chunk state six field는 하나의 24-byte batch로 수집하며, pending request는 하나, 주기는 10 Hz 이하다. Mouse movement마다 full-world readback은 없다.
  - Cell/Chunk, simulation tick, diagnostic sequence, selection generation, world epoch을 묶은 identity를 사용하고 reset, scenario switch, staging failure, shutdown에서 pending/sample을 무효화한다.
  - Validation: fmt/check PASS; silent-pending Inspector/UI targeted `16/16` PASS; existing viewport tests `7/7` PASS; affected Windows all-target check/clippy PASS; strict policy audit PASS; exactly one 3-frame Gallery smoke PASS. Workspace FULL `0`; scenario candidate `0`.
  - User acceptance grounds: compact Material hover와 `I` detailed Inspector가 잘 동작했고, silent pending은 자연스러웠으며 이전 Cell의 stale 값이나 `Collecting...` placeholder는 보이지 않았다. Material과 Field 상태 이해에 실질적으로 도움이 되었고 reset/scenario switch도 사용자 관찰상 정상이었다.
  - Known follow-up: 새 Cell hover 뒤 정보 표시까지 최대 10 Hz / 100 ms bounded readback에 따른 약간의 지연이 있다. 현재 비차단이며 Heavy Mixed와 G8-C를 막지 않는다. 실제 UX 비용과 readback 비용을 함께 측정하기 전에는 sampling frequency나 architecture를 변경하지 않는다.

---

### Next Action

1. **Heavy Mixed World**: Cell Inspector v0 acceptance가 기록되었으므로 Scenario 5 fixture/Harness는 **NEXT / IN PROGRESS 진입 가능**이다. Heavy Mixed user acceptance 전에는 G8-B를 닫지 않는다.
2. **Scope boundary**: G8-C, G9, optimization, `main`/PR은 별도 사용자 지시 전 시작하지 않는다. Accepted Sand/Water/Fire/Pressure candidate/artifacts와 automatic verdict는 변경하거나 재실행하지 않는다.

---

### Blockers

Canonical Recovery, v5 technical evidence, Sand Fall behavior, Sand Fall Harness pilot, Water Flow user acceptance, Fire / Heat automatic candidate와 user acceptance, Pressure Burst causal candidate와 user acceptance, Cell Inspector v0 user acceptance에는 확인된 기술 blocker가 없다. Water의 local-liquid 재배열, Pressure의 top-seam-only opening/small plume/broad terminal activity, Inspector의 최대 100 ms hover sample delay는 알려진 비차단 후속 과제이며 production-physics defect 증거는 없다. Pressure tail/activity 비용은 G8-C에서 측정한다. G8-B closure에는 Scenario 5 Heavy Mixed World acceptance가 남아 있다. G8-A의 same-SHA visual validation, `main` 승격과 G8-C/G9 진입은 별도 사용자 결정이다.

Canonical Recovery의 source 선택, merge, 검증, 보존 경계는 [Canonical Recovery Evidence](../evidence/CANONICAL_RECOVERY_2026-08-17.md)에 기록한다.

---

## Approval State

Foundation Design direction: **APPROVED BY USER**

M0 implementation: **IN_PROGRESS** — G0/G1/G2/G3/G4/G5/G6/G7 PASS / CLOSED (G2/G3/G4/G5/G6/G7 User Validation APPROVED); G8 Performance Evidence: IN_PROGRESS (G8-A verified v5 evidence candidate; official capture and independent verification complete; same-SHA User Visual Validation PENDING; G8-B Scenarios 1–4 user accepted, Water and Pressure automatic `NEEDS_HUMAN_REVIEW` unchanged with known follow-ups, Cell Inspector v0 user accepted with known follow-up, Scenario 5 Heavy Mixed World next / in-progress entry allowed, overall NOT CLOSED; G8-C PENDING).

M0 `ACHIEVED`: **NO**

---

## Machine-generated Facts

```text
g7_frozen_base_sha: 94babb2667c081b5588489e1b4e710cc6efa68be
g8a_remediation_base_sha: a67abaf959aba0423627f35b79fce7c82d8ec9b5
g8a_remediation_branch: fix/g8a-evidence-remediation-v5
g8a_verified_source_sha: 9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe
g8a_prebranch_binary_patch_sha256: 9993574e552f2cf9523aa2e1c3ee8b0f7ebe6dae422eebf5a8ecbe2737c0cd5b
roadmap_update_ref: origin/agent/update-product-roadmap at a5efc1b4bb54cdd9629de9654c8b5eb0c24397b7
canonical_main_ref: origin/main at 1304b71a15df140a994737becb5f47f421758801
canonical_recovery_branch: integration/canonical-recovery (local only)
canonical_recovery_merge: e5871bdc53093700c44562826860c4d482f31ba5
canonical_recovery_publication: not pushed; no recovery PR; main unchanged
g8b_candidate_branch: feature/m0-g8b-scenario-suite
g8b_candidate_base: ca79bb20b27041758ab4d4a224e491c171189393
g8b_candidate_status: SCENARIOS 1-4 USER ACCEPTED; CELL INSPECTOR V0 USER ACCEPTED WITH KNOWN FOLLOW-UP; SCENARIO 5 HEAVY MIXED WORLD NEXT / NOT CLOSED
g8b_official_scenarios: sand-fall, water-flow, fire-heat, pressure-burst, heavy-mixed-world
g8b_scenario_1_acceptance: sand-fall USER ACCEPTED; complete settling and all chunks sleeping are success; no activity-retuning
g8b_scenario_2_acceptance: water-flow USER ACCEPTED WITH KNOWN FOLLOW-UP; automatic NEEDS_HUMAN_REVIEW unchanged
g8b_scenario_3_candidate: fire-heat source 1635fdb9f562192123c92846e137b125c684ede9; automatic PASS unchanged; USER ACCEPTED
g8b_scenario_4_acceptance: pressure-burst USER ACCEPTED WITH KNOWN FOLLOW-UP; automatic NEEDS_HUMAN_REVIEW unchanged
cell_inspector_v0_tested_source_sha: 3c342d25099683df53e303d1920cebe1f6578b74
cell_inspector_v0_prior_docs_closure_sha: 6a521c07c23d9817f6abb9d3e1eab47758b5d5c4
cell_inspector_v0_human_verdict: USER ACCEPTED WITH KNOWN FOLLOW-UP
cell_inspector_v0_known_follow_up: slight bounded hover delay at no more than 10 Hz / 100 ms; non-blocking; measure UX and readback cost before changing frequency or architecture
g8b_remaining_acceptance: heavy-mixed-world NEXT
g8b_regression_only_scenario: active-sleep-g7 (exact 256x256x64; not official matrix)
g8b_shared_staging: powdergame-scenarios::reset_and_stage_scenario
evidence_writer_schema: calibration=powdergame-g8a-v5; shared fixture=powdergame-g8b-fixture-v1
g8b_harness_experiment_source_sha: 9e1fdac44aa14a546c7fe5ad6ceba49e71777eb5
g8b_harness_docs_closure: later docs-only commit; distinct from experiment source
g8b_harness_branch_retention: retained; ff-only aligned with feature/m0-g8b-scenario-suite at docs closure
g8b_harness_run_id: g8b-sand-fall-v0-20260817T065311878587Z-3ebd7505
g8b_harness_automatic_verdict: PASS
g8b_harness_review_output: HARNESS REVIEW OUTPUT APPROVED
g8b_harness_review_packet_sha256: feffc180f81d36558b8139f5436a2a0eed6422617dd9a3207153b1cb62af1323
g8b_harness_receipt_sha256: 42bbacf77ca80356996a53fb2d0a56a5aba18215b5b85bda637350522c95e033
g8b_water_harness_base_sha: d12edbfbcc0fb3fc2ef599cd06b3c46a2293d268
g8b_water_harness_source_sha: 5af031f1a04af866127616d4f1b0faa6c85e4d8e
g8b_water_harness_schema: manifest v1; telemetry/analysis/report/receipt v2; frames v0
g8b_water_harness_fast: workspace fmt/check PASS; scenarios 7/7; shared GPU reset 1/1; bounded Water GPU 1/1; windows Sand/Water experiment 16/16; Python 23/23; FULL workspace test/clippy/diff PASS; Gallery release smoke PASS
g8b_water_harness_run_id: g8b-water-flow-v0-20260817T110906547252Z-8b808e66
g8b_water_harness_automatic_verdict: NEEDS_HUMAN_REVIEW
g8b_water_harness_human_verdict: ACCEPTED WITH KNOWN FOLLOW-UP
g8b_water_harness_review_packet_sha256: 83783025ee6bdac8f6dedbf25edfec1dd75040d533c9fc563157cc699b5caec5
g8b_water_harness_receipt_sha256: 96f60b465dbfa4f7a4cacd7f78f475cad9af7c2e6d1754aba4dddc186d497c1b
g8b_water_harness_outer_basin_water_max_final: 0 / 0
g8b_water_harness_conservation_movement_cross_destination_integrity_reset: PASS
g8b_water_harness_final_active_partition: total 64; Water/EMPTY 51; Water/Oil 1; other 12
g8b_water_harness_known_follow_up: M0 local-liquid free-surface minority-cell persistent rearrangement; no production-physics defect evidence
g8b_water_harness_runs: first candidate immutable/superseded; remediation candidate immutable/user accepted with known follow-up; automatic verdict unchanged
g8b_fire_harness_source_sha: 1635fdb9f562192123c92846e137b125c684ede9
g8b_fire_harness_run_id: g8b-fire-heat-v0-20260817T133938546075Z-0e6aa901
g8b_fire_harness_automatic_verdict: PASS
g8b_fire_harness_user_acceptance: USER ACCEPTED (2026-08-18); production physics change NO; candidate rerun NO
g8b_fire_harness_review_packet_sha256: 2a8e99d14bf0647b71e7ef32e3840655117e93b9f20ad1360af97d62a69eb940
g8b_fire_harness_receipt_sha256: ed17e75f7515d155f8b6e5a41a0aeb751b2876ec573658a6e49eb6dd72108aff
g8b_fire_harness_audit_bundle_sha256: 1c1df01dfa9004b9273bc45e4b01d3c784d5c377f98a9417bc0b7594c6a83706
g8b_fire_harness_binary_sha256: 0338dfedbfd226f041cfda1b3ee4a81131ba27a2d5b8035abd4e653b552edbb9
g8b_fire_harness_independent_verification: PASS; zero concrete mismatch
g8b_pressure_harness_source_sha: 28e4e0bf7736fb95f0a9d7797cc3ab4274b91442
g8b_pressure_harness_run_id: g8b-pressure-burst-v0-20260818T014452058676Z-353fb706
g8b_pressure_harness_automatic_verdict: NEEDS_HUMAN_REVIEW (unchanged)
g8b_pressure_harness_human_verdict: FIX REQUIRED — fixture_causality_confounded_by_combustion
g8b_pressure_harness_review_packet_sha256: 10c4ef1e376a416a6895fb3fc7681b7bf7d9cb72610bad6407028807be96f3ed
g8b_pressure_harness_receipt_sha256: 2c044806e4049d38cdd8edf967af92f118c8a37321d56367fbc7a334a7339a1b
g8b_pressure_harness_audit_bundle_sha256: 7b51c82171d2d3d800f951967068bdb7e8abf6ce2b97bd0f6b4845d670004471
g8b_pressure_harness_causal_blocker: top relief Wood authored 95 C > ignition 90 C; complete opening at burn-duration boundary; production physics defect not established
g8b_pressure_remediation_scratch_run_id: g8b-pressure-burst-v0-scratch-20260818T100203210143Z-a37e998c
g8b_pressure_remediation_scratch_source: HEAD 328757bd8cd5b07cd0ed4c66a592f973dcd66981; git_state dirty; run_mode scratch
g8b_pressure_remediation_source_input_manifest_sha256: 485c1a915afca65757c9994d0036381a92cc40d2134e293f617ead024edc966b
g8b_pressure_remediation_scratch_verdict: NEEDS_HUMAN_REVIEW (review-only); causal classification pressure_opening_precedes_combustion; hard predicates 10/10 PASS; candidate blocker false
g8b_pressure_accepted_source_sha: 43e19d0f3b43aa0c15bf31e98f6401ba5f885170
g8b_pressure_accepted_run_id: g8b-pressure-burst-v0-20260818T101046792957Z-17158748
g8b_pressure_accepted_automatic_verdict: NEEDS_HUMAN_REVIEW (unchanged)
g8b_pressure_accepted_human_verdict: USER ACCEPTED WITH KNOWN FOLLOW-UP
g8b_pressure_accepted_review_packet_sha256: f1523a816e5c97ad15fbd53e910553f8bbe59c3c7b78cf7fd6089f7dc143c0eb
g8b_pressure_accepted_audit_bundle_sha256: 6f8e4579afb7b3375e571b1481b46d95e0e8533b4e718e6c5bd3ca4d120e5bc8
g8b_pressure_accepted_receipt_sha256: c21374d5bef5e3d2233cc488c893829b5285988887bcbd35db00f3ae87f72a64
official_capture_completion_marker: CAPTURE_RECEIPT.json (must be written last)
official_capture_package_hash: ZIP-external PACKAGE_SHA256.txt
official_capture_id: g8a-v5-9abec9e-20260817T032827206Z
official_capture_package_sha256: 4b9f44f66c18235f80d33738d15f3418c65c98e68e254aa01d13fc3a66eb6ec8
independent_verification: success (11/11 checks passed; zero findings)
build_id: local-cargo-2026-08-17
platform: Windows
primary_gpu: RTX 5090
world_config: 2048x2048 reference
chunk_config: 64x64 initial
build: canonical recovery PRE/POST full workspace, clippy, evidence self-tests, verifier fixtures, and Windows release smoke passed; 385 tests passed, 3 expected ignored, 0 failed in each workspace run
source_freeze_checks: official receipt complete; source/upstream clean and unchanged at 9abec9e; independent verifier passed 11/11 with zero findings
benchmarks: official G8-A v5 P50 = 865.304 TPS / 1.155663 ms per tick and profiled envelope P50 = 1.018080 ms; historical v4 remains unbound historical data
adversarial_review: optional and explicit-request-only; existing G8-A report is historical and non-blocking
m0_status: IN_PROGRESS (G0-G7 PASS/CLOSED User Validation APPROVED; G8 IN_PROGRESS — G8-A verified v5 evidence candidate, same-SHA user visual validation pending; G8-B scenarios 1-4 and Cell Inspector v0 accepted with documented follow-ups, Water and Pressure automatic NEEDS_HUMAN_REVIEW unchanged, Scenario 5 Heavy Mixed World next/in-progress entry allowed, overall not closed; G8-C pending)
```
