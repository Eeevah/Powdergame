# Powdergame Status

이 문서는 현재 실제 상태를 기록한다. 장기 방향은 `ROADMAP.md`, 완료 기준은 `MILESTONES.md`를 따른다.

---

## Human-maintained Status

### Current Milestone

`M0 — First World`

### Current Milestone Status

`IN_PROGRESS` — G0 (Runtime) PASS, G1 (World Integrity) PASS, G2 (Local Movement) PASS / CLOSED, G3 (Density / Displacement) PASS / CLOSED, G4-A (Thermal Baseline) TECHNICAL PASS, G4-B (Phase Transition) TECHNICAL PASS, G4-C (Combustion) TECHNICAL PASS. G4 전체는 아직 PASS/CLOSED가 아니다 (G4 User Validation 미실행). G4~G9와 최종 M0 사용자 승인 남음.

### Current Phase

**G0 — Runtime: PASS. G1 — World Integrity: PASS. G2 — Local Movement: PASS / CLOSED. G3 — Density / Displacement: PASS / CLOSED. G4 — Thermal / Phase / Combustion: IN_PROGRESS (G4-A thermal baseline TECHNICAL PASS, G4-B phase transition TECHNICAL PASS, G4-C combustion TECHNICAL PASS, G4 User Validation PENDING / NOT YET RUN).**

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

2026-08-16 기준 **G0 (Runtime)**, **G1 (World Integrity)**, **G2 (Local Movement)**, **G3 (Density / Displacement)**가 구현·검증·승인 완료되었고, **G4-A (Thermal Baseline)**, **G4-B (Phase Transition)**, **G4-C (Combustion)**는 기술 검증 완료 상태(TECHNICAL PASS)다. G2는 사용자가 개선된 128×128 가상 숲 movement demo를 직접 실행해 ("잘된다") 승인했고, G3는 laboratory `--density-demo`를 직접 실행해 약 300 ticks 관찰 후 승인했다.

G4 전체는 여전히 IN_PROGRESS다. 현재 branch `feature/m0-g4-thermal-phase-combustion`(base: `4053fe0a51ecdf59e5515eb58e2079e87c78c740` — G3 PASS/CLOSED)에는 G4-A Thermal Baseline + thermal ownership hardening, G4-B Ice↔Water↔Steam Phase Transition, G4-C Wood/Oil Combustion이 구현되어 있다. G4 통합 User Validation demo, G5 Pressure logic, phase expansion은 아직 구현되지 않았다.

### G0 Runtime Evidence (2026-08-16, local run)

```text
Base commit:      6de27451a931cdc3c07cdea012163fb80eab87c6 (main @ G0 시작 시점)
Implementation:   3f67cf0168f4be3735774b6261e592c499b4f5d8
                  — 첫 G0 implementation baseline commit
                  (feat: establish M0 G0 runtime baseline, branch feature/m0-g0-runtime)
                  + cd2c501777c23cff9573be8abc5b13cf10df4ed1 (fix: harden G0 runtime evidence)
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

Headless test:    headless_simulation_lifecycle_without_window: PASS
                  (DX12 adapter + RTX 5090 확인, 4 tick, GPU marker readback == 1)
Windows smoke:    powdergame-windows.exe --smoke-frames 60 → exit 0
                  window 생성 OK, tick marker=1, 60 frames present OK
```

G0 Evidence Gate 판정:

```text
Rust workspace/build 성공                          PASS
winit Windows app 실행                             PASS
wgpu DX12 path 확인                                PASS
GPU Simulation Core가 window rendering과 독립 tick   PASS
WorldConfig로 world size 설정 가능                  PASS
reference world 2048×2048 초기화 가능               PASS
headless/reference execution hook 존재              PASS
actual adapter/device info 출력 / RTX 5090 확인     PASS
dense Current/Next 실제 GPU allocation              PASS
Window empty-frame present                          PASS
```

### G1 World Integrity Evidence (2026-08-16, local run)

```text
Branch:           feature/m0-g1-world-integrity (base: cd2c501777c23cff9573be8abc5b13cf10df4ed1)
Material ID:      EMPTY = 0 (absence, not Matter)
                  BOUNDARY_BLOCK = 1 (registered)
                  STONE = 2 (registered)
Registry:         EMPTY is NOT an entry (registry_contains(EMPTY) == false,
                  registry_lookup(EMPTY) == None). Void has no material ID.
Domain:           finite; out-of-bounds coordinate → Void (None), never clamped
Initial world:    outermost ring = BOUNDARY_BLOCK, interior = EMPTY
                  (Current/Next 일관, temperature/pressure/flags = 0)
Edit hook:        write_material(x, y, value) — validated coordinate + ID
                  boundary cell BOUNDARY_BLOCK → EMPTY 성공 (erase 증명)
                  interior EMPTY → STONE 성공 (registered Matter 배치)
                  unknown ID (999) → InvalidMaterialValue 거부
                  OOB (-1,0)/(8,0)/(0,8)/(0,-1) → CoordinateOutOfBounds 거부

GPU tests (world_integrity.rs, 실제 RTX 5090/DX12):
  reference_world_boundary_initialization            PASS
  small_world_boundary_pattern_matches_expected      PASS (8×8 BBBBBBBB 패턴)
  boundary_block_is_editable_to_empty                PASS
  stone_is_a_registered_matter_distinct_from_empty   PASS
  invalid_material_edit_is_rejected                  PASS
  out_of_bounds_is_void_and_never_a_buffer_index     PASS
  g0_tick_regression_on_boundary_world               PASS

Core tests (24): EMPTY==0, EMPTY 미등록, BOUNDARY/STONE 등록, ID unique,
  unknown ID 거부, coordinate→index, OOB→Void, no-clamp, edge 분류,
  reference/8×8 초기 패턴, world size 비확장.

cargo fmt / build / test / clippy(-D warnings) / git diff --check: 모두 PASS
Windows smoke (--smoke-frames 60): PASS — G0 regression 없음
```

G1 Evidence Gate 판정 (MILESTONES.md 기준):

```text
One Cell = Max One Matter invariant test              PASS (구조: material_id 단일 slot + 테스트)
EMPTY가 Material Registry Matter가 아님               PASS
per-cell mixed amount 없음                            PASS
editable outer BLOCK                                  PASS
outer BLOCK 제거 가능                                 PASS
열린 boundary 밖 Matter가 Void로 소멸                  PASS (domain contract; G2에서 실제 소멸 적용)
invalid material id / out-of-bounds 없음               PASS
```

### G2 Local Movement Evidence (2026-08-16, local run)

```text
Branch:           feature/m0-g2-local-movement (base: eb4c77f82c3663ad0ece7a7291db0668e1acb50a)
MovementClass:    STATIC = 0 (Boundary Block, Stone)
                  POWDER = 1 (Sand)
                  LIQUID = 2 (Water, Oil)
                  GAS    = 3 (Steam, Smoke)
Registry:         SAND=3, WATER=4, OIL=5, STEAM=6, SMOKE=7 추가 (기존 ID 유지)
                  EMPTY는 여전히 registry entry 아님, Void material ID 없음
Stencils:         STATIC: no move
                  POWDER: down → down-diagonal → stop
                  LIQUID: down → down-diagonal → lateral(1 cell) → stop
                  GAS:    up → up-diagonal → lateral(1 cell) → stop
                  First-Match, 1-cell local only (scan/teleport 없음)
                  parity 기반 좌/우 stateless ordering (RNG state 없음)
                  모든 stencil candidate(primary/diagonal/lateral)의 OOB는
                  Void exit — open side/top/bottom은 invisible wall 아님
Pipeline:         propose (Current read, 1 destination) → resolve (EMPTY cell당
                  winner exactly one, fixed min-source arbitration)
                  → commit (각 cell이 자기 material_next slot만 write)
                  → GPU material_next → material_current copy
                  CPU는 매 tick full world를 계산/복사하지 않음

GPU tests (movement.rs, 실제 RTX 5090/DX12 — 16개 + ignored benchmark 1개):
  static_materials_never_move                        PASS
  sand_falls_exactly_one_cell_per_tick               PASS
  sand_takes_diagonal_when_down_blocked              PASS
  sand_stops_when_fully_blocked                      PASS
  water_falls_down_then_flows_laterally_one_cell     PASS
  oil_uses_the_liquid_family                         PASS
  steam_and_smoke_rise                               PASS (steam T=80 stable staging)
  gas_takes_up_diagonal_when_up_blocked              PASS
  gas_stable_bulk_center_does_not_swap               PASS (Gas↔Gas 무의미 swap 없음)
  contention_exactly_one_winner_no_duplication       PASS (winner exactly one,
                                                        loser valid, matter conserved)
  chunk_boundary_movement_is_plain_local_movement    PASS (63↔64 경계 양방향)
  void_exit_loses_exactly_one_matter                 PASS (open boundary로 실제 소멸,
                                                        OOB memory access 없음)
  liquid_exits_through_open_side_boundary            PASS (side opening Void exit)
  powder_diagonal_void_exit                          PASS (diagonal OOB Void exit)
  g2_tick_preserves_g1_contracts                     PASS (invalid ID/OOB 거부,
                                                        boundary erase, EMPTY 미등록)
  coarse_reference_world_perf                        PASS (sanity observation only)
  controlled_reference_world_perf                    PASS (ignored; release benchmark,
                                                        see Performance below)

Performance — Controlled reference-world baseline (2048×2048, RTX 5090, DX12):
  scenario:       2048×2048 initial reference world (outer Boundary Block
                  ring + EMPTY interior) — reference 초기 상태 기준
  build:          release, idle machine
  protocol:       simulation 1회 생성 → warm-up 100 ticks (제외)
                  → device.poll(PollType::Wait) (warm-up GPU work와 측정
                  interval 분리) → 측정 1000 ticks × 5 runs → 각 run 마지막
                  submission 후 device.poll(PollType::Wait) → timer 종료 → median
  runs:           0.1506 / 0.1503 / 0.1462 / 0.1442 / 0.1445 ms/tick
                  (≈ 6641 / 6655 / 6838 / 6935 / 6919 TPS)
  MEDIAN:         0.1462 ms/tick ≈ 6838 TPS
  의미:           reference 초기 world에 대한 full G2 propose → resolve →
                  commit → Next→Current pipeline의 baseline cost를 측정한 것.
                  active/heavy-matter gameplay benchmark가 아니다. 60 TPS
                  목표 대비 여유(~114×)는 이 reference scenario 한정 결과다.
  명시:           controlled idle-machine, release, coarse end-to-end
                  wall-clock including GPU completion — GPU timestamp
                  benchmark 아님. GPU timestamp framework는 G2 범위 밖.
  실행 방법:      cargo test --release -p powdergame-gpu --test movement \
                    controlled_reference_world_perf -- --ignored --nocapture

Movement demo (User Validation fixture) — 128×128 가상 숲 (승인 완료):
  실행:           cargo run -p powdergame-windows -- --movement-demo
                  또는 상위 폴더의 run_powdergame.bat
  world:          128×128 (chunk 64) — cell이 충분히 커서 pixel movement
                  를 육안 관찰 가능. reference 2048×2048은 변경 없음
  scene:          stylized 가상 숲 — Stone을 녹색 지형/나무로 표현
                  zone은 좌→우 SAND | WATER | OIL | STEAM | SMOKE 순,
                  stone tree-trunk divider로 분리 + 우하단 별도 Void funnel
                  - SAND:  언덕+나무 위로 모래 쏟아짐 → ledge/ground로
                           계단식 낙하·퇴적 (bump, tree canopy에 쌓임)
                  - WATER: cliff 위 물이 양쪽으로 흘러 폭포 → mid ledge에
                           떨어졌다가 basin에 고임 (나무가 선 연못)
                  - OIL:   stone bowl에 oil이 떨어져 고이는 LIQUID pool
                  - STEAM: geyser basin → slab 밑을 돌아 canopy gap으로 상승
                           (G4-B: Steam은 T=80 stable로 staging)
                  - SMOKE: pit에서 canopy gap으로 상승
                  - Void:  platform+funnel의 모래가 열린 boundary hole로
                           빠져 실제 소멸 (matter count 감소)
  view:           square-cell aspect-preserving (letterbox, scale=min ratio,
                  crisp cell edge) — 1280×720에서 ~720×720 정사각 표시
  initial state:  PAUSED — staging 직후 원본 scene을 먼저 볼 수 있음
  controls:       SPACE play/pause | N step (paused) | R reset | ESC quit
  demo TPS:       15 TPS (render FPS와 분리 — 관찰용, Simulation::tick
                  semantics와 60 TPS target은 변경 없음)
  title:          zone 순서 + [PAUSED]/[PLAY 15 TPS] + tick count 표시
  renderer:       read-only (material_current storage read) — presentation은
                  simulation state를 수정하지 않음
  bounded run:    --movement-demo --smoke-frames 120 → exit 0, device lost 없음

G2 User Validation:
  PASS — 사용자가 개선된 movement demo를 직접 실행해 Sand / Water / Oil /
  Steam / Smoke movement와 관찰 fixture(PAUSED/SPACE/N/R, 15 TPS,
  square-cell view)가 정상 동작함을 승인 ("잘된다", 2026-08-16).

G0/G1 regression:
  cargo test --workspace 전체 PASS (G0 headless + G1 world integrity 유지)
  Windows smoke (--smoke-frames 60): PASS — RTX 5090/Dx12, 2048×2048, 60 frames

cargo fmt / build / test / clippy(-D warnings) / git diff --check: 모두 PASS
```

G2 Evidence Gate 판정 (MILESTONES.md 기준):

```text
STATIC / POWDER / LIQUID / GAS movement family        PASS (registry + GPU tests)
behavior별 local stencil / First-Match                PASS
1 tick에 local neighbor 밖 teleport 금지               PASS (1-cell 이동 고정 테스트)
long-distance empty-cell scan 금지                    PASS
ownership winner exactly one                         PASS
loser stays valid / no duplication / no unexplained loss  PASS
열린 boundary → Void 실제 소멸                         PASS (bottom/side/diagonal, matter count 정확히 -1)
OOB GPU memory access 없음                            PASS
G0/G1 regression 없음                                 PASS
Performance observation                          REFERENCE BASELINE RECORDED —
                                                      controlled idle-machine, release,
                                                      median ~0.146 ms/tick (≈6838 TPS)
                                                      @2048×2048 reference initial world
                                                      (Boundary ring + EMPTY interior),
                                                      coarse end-to-end incl. GPU completion.
                                                      active/heavy-matter gameplay throughput
                                                      validation은 향후 별도 scenario/
                                                      benchmark 대상으로 남김
User Validation (movement demo)                       PASS — 사용자 승인 완료 (2026-08-16)
```

**G2 — Local Movement: PASS / CLOSED.**

### G3 Density / Displacement Evidence (2026-08-16, local run)

```text
Branch:           feature/m0-g3-density-displacement (base: 686ed5a08effcab002912ddca271aac6f4010d56)
철학:             "부력을 계산하지 않는다. 정렬한다." — buoyancy solver 없음,
                  small integer Density Rank + local displacement 만으로
                  침강/층분리/기체 ordering/STATIC exclusion/equal-rank 안정성 증명
Density Rank:     Material property (density_rank: Option<u32>) — per-cell 저장 금지
                  Steam = 20   Smoke = 30   Oil = 70
                  Water = 90   Sand = 150
                  Boundary Block = None (STATIC)   Stone = None (STATIC)
                  EMPTY = None (registry Matter 아님)
                  값은 현실 단위가 아니라 A>B/A==B/A<B 비교용 gameplay ordering.
                  Material ID는 Identity, Density는 Property — ID를 density 순서로
                  재배치하지 않음
GPU lookup:       density_table[material_id] -> u32 (0 = displacement 없음/STATIC/EMPTY sentinel,
                  20/30/70/90/150 = movable rank) — Material property upload이지
                  per-cell state 아님. packing/u8 최적화 없음, 가독성 우선
Density semantics (First-Match 유지, stencil ordering은 G2 그대로):
                  B == EMPTY          → G2 normal movement
                  B == STATIC         → blocked (다음 candidate)
                  B movable + equal rank → density swap 없음 (blocked)
                  B movable + ordering OK → local SWAP candidate
                  B movable + ordering inappropriate → blocked
                  OOB                 → G2 Void exit 의미 유지
                  POWDER/LIQUID down/down-diagonal: source_rank > dest_rank → SWAP
                  GAS up/up-diagonal:              source_rank < dest_rank → SWAP
                  lateral: EMPTY-only (density lateral displacement 없음 — 수직 정렬이
                  핵심이며 동일 높이 lateral density swap은 무의미한 jitter 방지)
Shader 구조:      G2 audit risk였던 string brace scanner 제거 — pass별 명시적 WGSL
                  source로 분리 (movement_propose.wgsl / movement_claim.wgsl /
                  movement_commit.wgsl). WGSL include/code generator 없음.
                  refactor 직후 기존 G2 tests 전부 PASS 확인 후 G3 logic 추가
Ownership:        G3 swap은 source+destination 양쪽 endpoint가 동일 movement edge를
                  선택해야 실행 (edge selected at source AND at destination).
                  per-cell claim buffer (ownership arbitration scratch state —
                  density state 아님). fixed min-source arbitration 유지.
                  한 Cell이 한 Tick에 두 개의 ownership-changing edge에 동시 참여 불가
Commit:           write-self 유지 — 각 invocation은 material_next[self]만 write.
                  swap S<->D: S invocation → next[S]=current[D], D invocation →
                  next[D]=current[S]. neighbor direct overwrite 없음.
                  unmatched/경쟁 패배 edge → 양쪽 모두 상태 유지 (corruption보다
                  conservative no-move)

GPU tests (density.rs, 실제 RTX 5090/DX12 — 15개):
  sand_swaps_with_water_below                        PASS
  sand_sinks_through_water_column                    PASS
  water_swaps_with_oil_below                         PASS
  oil_above_water_does_not_swap                      PASS
  mixed_water_oil_channel_separates_into_layers      PASS
  equal_rank_water_does_not_jitter                   PASS
  static_targets_never_swap                          PASS
  steam_swaps_up_through_smoke_when_blocked_above    PASS (steam T=80 stable staging)
  stable_gas_ordering_does_not_swap                  PASS (steam T=80 stable staging)
  gas_channel_orders_steam_above_smoke               PASS (steam T=120, 12 ticks —
                                                        밀봉 채널에서 응축 방지)
  overlapping_swap_chain_corrupts_nothing            PASS
  density_contention_exactly_one_winner              PASS
  density_swap_crosses_chunk_boundary                PASS
  sand_sinks_then_exits_through_open_boundary        PASS
  density_pipeline_executes_on_gpu                   PASS

Core pure tests (movement.rs/material.rs): density rank/ordering/STATIC/equal-rank/
  lateral-rejection 전부 PASS — G2 movement/stencil/Void pure tests 전부 유지

Density demo (User Validation fixture) — 128×128 laboratory tanks (승인 완료):
  실행:           cargo run -p powdergame-windows -- --density-demo
                  또는 상위 폴더의 run_powdergame.bat
  world/view:     G2와 동일한 128×128 square-cell / PAUSED / 15 TPS /
                  SPACE·N·R·ESC 관찰 구조 재사용 (새 UI framework 없음)
  scene:          좌→우 SAND+WATER | WATER+OIL | STEAM+SMOKE laboratory tanks
                  - Tank 1: 하단 Water pool 위 큰 Sand block — PLAY 시 침강
                  - Tank 2: inverted (Water 위 / Oil 아래) → 층분리
                  - Tank 3: sealed (Smoke 위 / Steam 아래) → gas ordering
                            (G4-B: Steam은 T=80 stable로 staging)
  bounded run:    --density-demo --smoke-frames 180 → exit 0, device lost 없음

G0/G1/G2 regression:
  cargo test --workspace 전체 PASS (56 core + 15 density + 1 G0 headless +
  16 movement + 7 integrity = 95, ignored 1 controlled benchmark)
  Windows smoke (--smoke-frames 60): PASS — RTX 5090/Dx12, 2048×2048
  --movement-demo --smoke-frames 120: PASS — G2 forest demo regression 없음

cargo fmt / build / test / clippy(-D warnings) / git diff --check: 모두 PASS
```

G3 Evidence Gate 판정 (MILESTONES.md 기준):

```text
Density는 Material property (per-cell buffer 없음)      PASS (density_table lookup,
                                                          current/next density buffer 없음)
buoyancy float/SI solver 없음                            PASS ("sort, not buoyancy")
Sand in Water 침강                                       PASS (local swap, count 보존)
Water/Oil 층분리                                         PASS (inversion + stable + multi-cell)
Gas density ordering example                             PASS (Steam 20 / Smoke 30)
STATIC exclusion                                         PASS (Stone/Boundary는 rank None)
equal-rank 안정성                                        PASS (same rank → no swap)
lateral density jitter 없음                              PASS (lateral은 EMPTY-only)
long-distance density scan 없음                          PASS
swap이 neighbor Next를 직접 overwrite하지 않음            PASS (write-self commit)
overlapping swap/move에서 duplicate/loss 없음            PASS (edge 양단 agreement,
                                                          count conserved)
Void material ID 없음 / EMPTY 미등록 유지                 PASS
G4 code 없음                                             PASS (temperature/ignition/
                                                          combustion/phase 전무)
G0/G1/G2 regression 없음                                 PASS
User Validation (density demo)                           PASS — 사용자 승인 완료 (2026-08-16)
```

**G3 — Density / Displacement: PASS / CLOSED.**

M0 전체는 여전히 `IN_PROGRESS` — `ACHIEVED`가 아니다. G4~G9와 최종 사용자 승인이 남아 있다.

### G4-A Thermal Baseline (technical sub-step — TECHNICAL PASS, not G4 CLOSED)

```text
Branch:           feature/m0-g4-thermal-phase-combustion (base: 4053fe0a51ecdf59e5515eb58e2079e87c78c740 — G3 PASS/CLOSED)
Scope:            Temperature f32 4-neighbor conduction ONLY
Out of scope:     Ice/Water/Steam phase (G4-B), Wood/Oil combustion (G4-C),
                  Fire, Pressure, thermal demo, G4 User Validation
Reference T:      0.0 (relative hot/cold scalar; not Celsius)
State:            per-cell temperature_current / temperature_next
Material props:   thermal_conductivity, heat_capacity (cheap gameplay scalars)
                  Boundary k=0 (outer ring은 숨은 heat sink 아님)
EMPTY:            not a thermal medium; EMPTY self writes 0.0; no conduction
                  through EMPTY/Void; vacated / Void-exit cells are T=0
Update:           Read 4-neighbors → write self only
                  T' = T + clamp(RATE * Σ min(k_self,k_n)*(T_n-T) / C, ±MAX)
                  deadband |ΔT| < 1e-4 → skip; NaN/Inf → 0.0
Ownership:        no separate thermal Claim/Resolve. Movement commit
                  transports T with Matter on the same G3 edge
                  (stay / move / swap / void / unmatched).
                  density swap: 각 Matter가 자기 T를 가져감 (열이 좌표에 남지 않음)
                  Void exit: Matter와 열이 함께 world 밖으로 (T=0)
                  경쟁 패배/blocked: T 유지
GPU:              propose → claim → commit(mat+T) → copy both Current →
                  thermal.wgsl → temperature Next→Current
Phase/combustion: not started (G4-B / G4-C 미구현 at G4-A)
G4 전체 상태:      NOT PASS, NOT CLOSED (IN_PROGRESS)
```

G4-A 기술 검증 (2026-08-16, local run — 실제 RTX 5090/DX12):

```text
Core pure tests (thermal.rs + material.rs — 8개):
  hot_neighbor_heats_cold_self / cold_neighbor_cools_hot_self      PASS
  equal_temperature_is_stable / empty_neighbor_does_not_conduct    PASS
  conductivity_difference_changes_transfer (Water > Oil)           PASS
  output_is_always_finite (clamp + NaN/Inf → 0)                    PASS
  empty_self_has_no_thermal_state / tables_cover_registered_matter PASS

GPU tests (thermal.rs — 13개, 실제 RTX 5090/DX12):
  two_cell_hot_cold_propagation                    PASS (hot cools, cold heats, ordering 유지)
  four_neighbor_propagation                        PASS (4방향 > 1방향 전달)
  empty_gap_blocks_heat                            PASS (EMPTY는 thermal medium 아님)
  stone_and_water_exchange_heat                    PASS (이종 재질 heat exchange)
  thermal_crosses_chunk_boundary                   PASS (x=63↔64 경계 전달 — chunk wall 아님)
  repeated_ticks_stay_finite (200 ticks)           PASS
  no_nan_or_infinity_in_world                      PASS
  write_temperature_rejects_non_finite             PASS (NaN edit 거부)
  empty_cell_temperature_stays_at_reference        PASS
  hot_matter_carries_temperature_when_moving       PASS (이동 시 열이 destination으로)
  density_swap_carries_each_matter_temperature     PASS (swap 후 Sand가 여전히 더 뜨거움)
  void_exit_removes_temperature                    PASS (Void exit 시 T=0, ghost heat 없음)
  blocked_or_losing_move_keeps_temperature         PASS (blocked/경쟁 패배 시 T 유지,
                                                     winner가 hot state를 운반 — G4-B 이후
                                                     loser Steam은 T=80 stable로 staging)
```

**G4-A Thermal Baseline: TECHNICAL PASS.**

### G4-B Phase Transition (technical sub-step — TECHNICAL PASS, not G4 CLOSED)

```text
Branch:           feature/m0-g4-thermal-phase-combustion (G4-A 위에 추가)
Scope:            Temperature-based 1:1 SELF transitions only
                  Ice → Water, Water → Ice, Water → Steam, Steam → Water
Out of scope:     combustion, Fire, Wood, ignition, Smoke spawn (G4-C),
                  phase expansion / 1:N spawn, blocked expansion, Pressure,
                  latent heat, exact energy conservation, thermal demo
Material:         ICE = 8 추가 (기존 ID 변경 없음)
                  movement_class = STATIC, density_rank = None
                  thermal_conductivity = 0.60, heat_capacity = 2.0 (gameplay)
Data model:       MaterialDescriptor.phase_transitions: &'static [PhaseTransition]
                  (condition Below/Above + threshold + target_material)
                  — Material-owned small ordered rule set (REACTION_SPEC §6),
                  shader에 material-name branch 없음. GPU는 compiled
                  PhaseGpuDescriptor table (below_target/above_target/
                  below_threshold/above_threshold, NO_PHASE_TARGET=0xFFFF_FFFF
                  sentinel — EMPTY=0과 혼동 없음)만 사용
Thresholds:       Water freeze below -20 / boil above 60
                  Ice melt above -10 / Steam condense below 40
                  (relative gameplay scalar, not Celsius; hysteresis bands
                  -20↔-10과 40↔60에서 ping-pong 방지)
GPU pass:         phase_transition.wgsl — read material_current +
                  temperature_current + phase_table → write material_next[self]
                  only. Claim/Resolve/atomic 없음 (1:1 self transform)
Tick order:       movement (Matter + T 수송) → thermal conduction → phase
                  (새 위치에서 settle된 Temperature 기준으로 phase 선택)
Temperature:      1:1 transform에서 보존 (latent heat는 scope 밖)
EMPTY:            phase rule 없음, table sentinel, T==0 invariant 유지
Matter count:     1:1 — cell/matter count 변화 없음, spawn/duplicate 없음
```

G4-B 기술 검증 (2026-08-16, local run — 실제 RTX 5090/DX12):

```text
Core pure tests (phase.rs + material.rs — 13개):
  water_freezes_below_threshold / water_boils_above_threshold         PASS
  ice_melts_above_threshold / steam_condenses_below_threshold         PASS
  neutral_temperatures_are_stable / hysteresis_bands_prevent_ping_pong PASS
  non_phase_materials_never_transition (EMPTY/Stone/Sand/Oil/Smoke/Wood) PASS
  unknown_ids_never_transition / targets_are_registered_matter        PASS
  phase_candidates_are_only_water_ice_steam                           PASS
  gpu_descriptor_table_matches_reference                              PASS
  ice_thermal_properties_are_sane                                     PASS
  material.rs: ICE 등록/STATIC/density None/unique ID/valid values     PASS

GPU tests (phase.rs — 16개, 실제 RTX 5090/DX12):
  water_freezes_to_ice                                PASS (T=-30 → Ice, 1:1 count 보존)
  ice_melts_to_water                                  PASS (T=-5 → Water, T 보존)
  water_boils_to_steam                                PASS (T=70 → Steam, 1:1 count 보존)
  steam_condenses_to_water                            PASS (T=30 → Water)
  neutral_water_is_stable                             PASS
  hysteresis_prevents_ping_pong                       PASS (water -15, ice -15,
                                                        water +50, steam +50 모두 유지)
  non_phase_materials_never_transform_on_gpu          PASS (Sand/Oil/Smoke/Stone
                                                        극온에서도 불변)
  phase_preserves_temperature                         PASS (boil 후 T가 reference로
                                                        reset되지 않음)
  thermal_heating_triggers_boiling                    PASS (hot stone reservoir →
                                                        온도장이 Water를 boil threshold
                                                        넘김 → Steam 자발 생성)
  thermal_cooling_triggers_freezing                   PASS (cold reservoir → Ice 자발 생성)
  hot_water_moves_then_boils_at_destination           PASS (1 tick chain: movement →
                                                        T 수송 → no conduction → phase,
                                                        old cell EMPTY/T=0, new cell STEAM/T≈80)
  melted_ice_uses_water_movement_next_tick            PASS (tick1: STATIC Ice가 제자리에서
                                                        Water로 melt(phase는 movement 뒤라
                                                        이동 안 함) → tick2: LIQUID identity가
                                                        아래로 이동, T 동반, old cell EMPTY/T=0,
                                                        matter 보존)
  boiled_water_uses_steam_movement_next_tick          PASS (tick1: Water T=80이 밀봉 상태로
                                                        제자리에서 Steam으로 boil → tick2:
                                                        GAS identity가 위로 이동, hot T 동반,
                                                        source EMPTY/T=0, matter 보존)
                                                        → phase-changed Matter adopts new
                                                        movement behavior on following tick
                                                        (phase는 단순 ID repaint가 아님)
  ice_is_static_and_never_density_swaps               PASS (Ice는 STATIC, density None)
  phase_transition_crosses_chunk_boundary             PASS (x=63↔64 경계를 넘어
                                                        freezing — chunk는 phase wall 아님)
  phase_pipeline_executes_on_gpu                      PASS (marker=1)

기존 Steam fixture 갱신 (thermal semantics에 맞는 stable staging — movement/density
intent 변경 없음):
  movement.rs: steam 4개 테스트 Steam T=80
  density.rs: steam 3개 테스트 Steam T=80 (long sealed ordering만 T=120 + 12 ticks)
  thermal.rs: 경쟁 패자 Steam T=10 → 80 (winner 90 > loser 80 유지)
  main.rs demos: movement-demo steam zone + density-demo tank 3 Steam T=80

G0/G1/G2/G3/G4-A regression:
  cargo test --workspace 전체 PASS (78 core + 15 density + 16 phase + 1 G0 headless
  + 16 movement + 13 thermal + 7 integrity = 146, ignored 1 controlled benchmark)
  Windows smoke (--smoke-frames 60): PASS — RTX 5090/Dx12, 2048×2048
  --movement-demo --smoke-frames 120: PASS
  --density-demo --smoke-frames 180: PASS

cargo fmt / build / test / clippy(-D warnings) / git diff --check: 모두 PASS
```

**G4-B Phase Transition: TECHNICAL PASS.** (G4 전체는 여전히 IN_PROGRESS — G4-C 이후 G4 User Validation 남음.)

### G4-C Combustion (technical sub-step — TECHNICAL PASS, not G4 CLOSED)

```text
Branch:           feature/m0-g4-thermal-phase-combustion (G4-B 위에 추가)
Scope:            Wood/Oil combustion ENGINE + tests ONLY — generic ignition
                  / sustain / heat / Smoke request / presentation event
Out of scope:     Oxygen simulation, stoichiometry, Ash, finite fuel mass,
                  burn-age counter, realistic flame chemistry, Pressure /
                  rupture / vent, phase expansion (G5), G4 demo, Active/Sleep,
                  Reaction DSL editor, performance optimization
Material:         WOOD = 9 추가 (기존 ID 변경 없음)
                  movement_class = STATIC, density_rank = None
                  thermal_conductivity = 0.15, heat_capacity = 2.0 (gameplay)
                  Oil은 combustion descriptor 추가 (기존 ID/property 유지)
Data model:       MaterialDescriptor.combustion: Option<CombustionDescriptor>
                  { ignition_threshold, sustain_threshold, heat_per_tick }
                  — Wood/Oil share ONE generic grammar (REACTION_SPEC §11),
                  shader에 material-name branch 없음. GPU는 compiled
                  CombustionGpuDescriptor table (is_combustible + 3 floats,
                  16 slots × 16 bytes)만 사용
Tuning (baseline):Oil  ignite 75 / sustain 45 / heat +5 per tick
                  Wood ignite 90 / sustain 55 / heat +4 per tick
                  (relative gameplay scalar, not SI; retunable)
Fire is NOT Matter: flame = Matter + FLAG_COMBUSTING + heat +
                  FLAG_FLAME_EVENT presentation signal (permanent orange
                  Fire ID 없음)
Flags:            FLAG_COMBUSTING (1<<0) persistent Matter-owned state,
                  FLAG_FLAME_EVENT (1<<1) per-tick presentation pulse
                  — combustion은 자기 bit만 set/clear, 미래 subsystem bit 보존
Ownership:        flags는 Matter-owned state — movement commit이 temperature와
                  같은 edge를 따라 수송 (stay/move/swap/void/unmatched 전부).
                  flags[] contract: occupying Matter에 부착된 state bits 전용
                  field (EMPTY flags=0). Pressure 같은 spatial/cell-owned state는
                  flags에 넣지 않고 별도 field 사용 — movement edge에서 미수송.
Edit invariant:   write_material이 identity 교체 시 Current/Next flags를 0으로
                  reset — stale COMBUSTING이 새 identity에 남지 않음
Tick order:       movement (Matter + T + flags) → thermal → phase →
                  combustion (self-write heat/flags + Smoke request) →
                  smoke claim (destination winner exactly one) →
                  smoke commit (destination self-write Smoke + hot T)
Smoke spawn:      max 1 local 1-cell candidate per burning source per tick
                  (up → up-diagonal → lateral, parity ordered; in-domain
                  EMPTY only; blocked → no spawn; Void는 spawn target 아님).
                  Smoke proposal은 기존 movement proposal/claim scratch를
                  sequential pass 안전 재사용. new Smoke T = burning source T
                  (finite). source Wood/Oil는 spawn으로 사라지지 않음.
```

G4-C 기술 검증 (2026-08-16, local run — 실제 RTX 5090/DX12):

```text
Core pure tests (combustion.rs + material.rs — 19개):
  Wood/Oil 같은 generic descriptor 구조 / nonflammable(Stone/Water/Sand/
  Ice/Steam/Smoke/Boundary/EMPTY) combustion None                     PASS
  Oil 75/ Wood 90 ignition threshold / sustain 45/55 continuation      PASS
  burning 아래 sustain → extinguish / burning → heat_per_tick 증가     PASS
  ignition tick도 heat 추가 / nonflammable hot → never ignites         PASS
  no Oxygen concept in pure rule (signature에 산소 입력 없음)           PASS
  outputs always finite / cap은 더 뜨거운 cell을 줄이지 않음            PASS
  combustion flags는 unrelated bit 보존 (bit mask)                    PASS
  smoke stencil ordering (up → diag parity → lateral parity → none)    PASS
  combustion table은 combustible만 1 (sentinel is_combustible=0)       PASS
  material.rs: WOOD 등록/STATIC/density None/thermal/combustion        PASS

GPU tests (combustion.rs — 25개, 실제 RTX 5090/DX12):
  hot_oil_ignites                                 PASS (T=80 → COMBUSTING + FLAME_EVENT, heat 추가)
  hot_wood_ignites                               PASS (T=95 → COMBUSTING)
  cold_oil_does_not_ignite / cold_wood_does_not_ignite  PASS
  burning_adds_heat                              PASS (T 증가)
  cooling_below_sustain_extinguishes             PASS (COMBUSTING/FLAME_EVENT 해제)
  nonflammable_hot_material_does_not_combust     PASS (Stone T=100, stale bit도 무시)
  no_oxygen_requirement                          PASS (완전 밀폐 stone chamber에서 점화,
                                                     smoke spawn도 차단)
  flame_event_emitted_on_ignition                PASS (presentation signal)
  combustion_flag_bits_are_preserved             PASS (unrelated bit 보존)
  burning_oil_carries_flags_when_moving          PASS (move edge로 flags 수송,
                                                     vacated source flags=0)
  burning_matter_swap_carries_flags              PASS (density swap으로 Oil의
                                                     COMBUSTING 이동, Smoke가 훔치지 않음)
  burning_matter_void_exit_clears_flags          PASS (EMPTY/T=0/flags=0, OOB write 없음)
  blocked_or_losing_burning_matter_keeps_flags   PASS (blocked 시 flags 유지)
  burning_wood_spawns_smoke / burning_oil_spawns_smoke  PASS (같은 generic path,
                                                     source 유지, one cell per request,
                                                     hot Smoke T = source T)
  smoke_spawn_contention_exactly_one             PASS (두 source → 같은 target,
                                                     winner exactly one, Wood 2 유지)
  smoke_spawn_crosses_chunk_boundary             PASS (x=63→64 spawn — chunk wall 아님)
  thermal_heating_triggers_ignition              PASS (hot reservoir 전도만으로
                                                     Wood가 ignition threshold 통과)
  edit_replaces_material_and_clears_flags        PASS (burning Wood → Stone,
                                                     flags reset)
  burning_source_keeps_heat_and_flags_while_spawning_smoke  PASS (1 tick 안에 combustion
                                                     self effects(heat + COMBUSTING +
                                                     FLAME_EVENT)와 Smoke spawn 동시 검증 —
                                                     후속 smoke commit이 source heat/flags를
                                                     clobber하지 않음)
  spawned_smoke_does_not_inherit_combustion_flags PASS (Smoke는 source T는 파생하지만
                                                     COMBUSTING/FLAME_EVENT는 복제하지 않음)
  unrelated_flag_bit_survives_combustion         PASS (TEST_UNRELATED_FLAG(1<<10) 보존 —
                                                     flags word 전체 덮어쓰기 없음)
  nonflammable_material_clears_stale_combustion_bits  PASS (Water에 stale combustion bits
                                                     설정 → tick 후 clear, T 증가 없음,
                                                     unrelated bit 보존)
  flame_event_is_set_on_active_ticks_and_cleared_on_extinguish  PASS (ephemeral pulse vs
                                                     persistent COMBUSTING 구분)

Integration hardening evidence:
  - combustion self effects → smoke proposal/claim/commit 후속 pass가 앞선
    combustion 결과를 clobber하지 않음 (heat + flags 보존)
  - spawned Smoke는 combustion flags를 상속하지 않음 (identity/state 복제 없음)
  - combustion shader는 flags word 전체를 덮지 않고 자기 bit만 조작
  - nonflammable Matter는 stale combustion state를 clear
  - FLAME_EVENT는 per-tick presentation pulse (ephemeral), COMBUSTING은 persistent

G0/G1/G2/G3/G4-A/G4-B regression:
  cargo test --workspace 전체 PASS (97 core + 25 combustion + 15 density +
  16 phase + 1 G0 headless + 16 movement + 13 thermal + 7 integrity
  = 190, ignored 1 controlled benchmark)
  Windows smoke (--smoke-frames 60): PASS — RTX 5090/Dx12, 2048×2048, marker=1
  --movement-demo --smoke-frames 120: PASS — G2 forest demo regression 없음
  --density-demo --smoke-frames 180: PASS — G3 lab demo regression 없음

cargo fmt / build / test / clippy(-D warnings) / git diff --check: 모두 PASS
공식 performance benchmark: 측정 안 함 (correctness-first; 추가 GPU passes의
잠재 cost는 risk 항목으로 기록)
```

G4-C Evidence Gate 판정 (MILESTONES.md G4 Required Evidence 기준):

```text
Temperature f32 baseline                                PASS (G4-A)
4-neighbor thermal propagation baseline                 PASS (G4-A)
EMPTY가 숨은 thermal medium 아님                        PASS (G4-A)
Material별 cheap conductivity/heat-capacity              PASS (G4-A)
Ice ↔ Water ↔ Steam                                     PASS (G4-B)
heating/cooling 양방향 transition                       PASS (G4-B)
Wood/Oil 공통 combustion grammar                        PASS (descriptor + GPU)
combustion → Heat + Smoke + presentation event          PASS (heat + spawn + FLAME_EVENT)
Oxygen이 하드코딩 필수 조건 아님                          PASS (no_oxygen GPU test + pure signature)
NaN/Infinity runaway 없음                                PASS (sanitize + cap + finite tests)
G4 User Validation                                       PENDING — G4 통합 demo 후 사용자 확인 필요
```

**G4-C Combustion: TECHNICAL PASS.** (G4 전체는 여전히 IN_PROGRESS — G4 User Validation PENDING / NOT YET RUN. G4 통합 demo는 아직 만들지 않았다.)

### Product Direction

> **현실을 구현하는 것이 아니라 가상의 재미있는 놀이터를 만든다. 핵심은 나만의 세계 창조다.**

현실의 자연현상은 reference이며 Powdergame 내부의 이해 가능한 논리와 상호작용이 우선한다.

### Next Action

1. G4-A thermal baseline + G4-B phase transition + G4-C combustion: **TECHNICAL PASS** (2026-08-16).
2. 다음 단계: **G4-A+B+C 통합 thermal/phase/combustion User Validation demo** (기본 60 TPS) → G4 User Validation.
3. G4 User Validation 후 G4 전체 검토 (필요 시 gameplay tuning).
4. 그 다음 **G5 — Pressure Chain** 준비 (phase expansion / yield / Pressure / rupture / vent).
5. G4 User Validation 전까지 G4를 PASS/CLOSED로 올리지 않는다.

공식 G4 performance benchmark는 측정하지 않는다 (correctness-first).

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
- GPU timestamp query benchmark framework (G2 baseline은 coarse wall-clock로 기록)
- active/heavy-matter world gameplay performance benchmark (G2 reference-world baseline과 별도 대상)
- G3 controlled performance baseline (idle-machine 여부 사용자 확인 후 별도 측정 — correctness-first)
- G4 combustion 추가 GPU passes의 성능 비용 측정 (G4 통합 demo 이후 별도 검토 — correctness-first)

---

## Approval State

Foundation Design direction: **APPROVED BY USER**

M0 implementation: **IN_PROGRESS** — G0/G1/G2/G3 PASS (G2·G3는 User Validation 포함), G4-A/B/C TECHNICAL PASS (G4 전체는 CLOSED 아님 — G4 User Validation 미실행), G4~G9 + 최종 M0 승인 남음

M0 `ACHIEVED`: **NO**

최종 M0 완료는 실제 구현/benchmark/play validation 후 사용자가 승인해야 한다.

---

## Machine-generated Facts

> 이 블록은 향후 script/CI/benchmark 도구가 갱신하도록 설계한다. 자동 생성 pipeline이 아직 없으므로 아래 값은 2026-08-16 로컬 검증에서 사람이 기록한 사실값이다. pipeline이 생기면 이 블록은 자동 갱신으로 전환한다.
>
> `base_commit_sha`는 G0 시작 시점의 main commit이다. implementation commit들은 이 블록이 아니라
> Human-maintained evidence 영역에 기록한다 (self-referential current commit SHA는 넣지 않는다).

```text
base_commit_sha: 6de27451a931cdc3c07cdea012163fb80eab87c6
build_id: local-cargo-2026-08-16
platform: Windows
primary_gpu: RTX 5090
world_config: 2048x2048 reference
chunk_config: 64x64 initial
build: passed (cargo build --workspace)
tests: passed (97 core incl. 8 G4-A thermal + 12 G4-B phase + 1 G4-B material (ICE) + 17 G4-C combustion + 2 G4-C material (WOOD/Oil) + 25 GPU combustion + 15 GPU density + 16 GPU phase + 1 GPU headless smoke + 16 GPU movement + 13 GPU thermal + 7 GPU world integrity = 190 total, ignored 1 controlled benchmark)
benchmarks: G2 controlled reference-world baseline (2048x2048 initial world, Boundary ring + EMPTY interior): release, idle machine, 100 warm-up ticks + GPU completion, 1000 measured ticks x 5 runs, GPU completion included — median ~0.146 ms/tick (~6838 TPS, RTX 5090/DX12, coarse end-to-end incl. GPU completion). Reference-scenario baseline only; NOT an active/heavy-matter gameplay benchmark. G3 controlled baseline: deferred (idle-machine 확인 후 별도 측정). G4: correctness-first, no official benchmark (additional GPU passes' cost deferred).
m0_status: IN_PROGRESS (G0 complete, G1 complete, G2 PASS/CLOSED incl. user validation 2026-08-16, G3 PASS/CLOSED incl. user validation 2026-08-16, G4-A thermal baseline TECHNICAL PASS, G4-B phase transition TECHNICAL PASS incl. next-tick movement adoption evidence, G4-C combustion TECHNICAL PASS incl. flags ownership + Smoke spawn ownership + integration hardening (spawn clobber-free, flags hygiene, FLAME_EVENT ephemerality) evidence — G4 not CLOSED, G4 User Validation not yet run, G4+ pending)
```
