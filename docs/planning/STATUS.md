# Powdergame Status

이 문서는 현재 실제 상태를 기록한다. 장기 방향은 `ROADMAP.md`, 완료 기준은 `MILESTONES.md`를 따른다.

---

## Human-maintained Status

### Current Milestone

`M0 — First World`

### Current Milestone Status

`IN_PROGRESS` — G0 (Runtime) PASS, G1 (World Integrity) PASS, G2 (Local Movement) PASS / CLOSED (User Validation 승인 완료 2026-08-16), G3 (Density / Displacement) PASS / CLOSED (User Validation 승인 완료 2026-08-16). G4~G9와 최종 M0 사용자 승인 남음.

### Current Phase

**G0 — Runtime: PASS. G1 — World Integrity: PASS. G2 — Local Movement: PASS / CLOSED (자동·기술 검증 + 성능 baseline 기록 + User Validation 사용자 승인 완료). G3 — Density / Displacement: PASS / CLOSED (자동·기술 검증 + User Validation 사용자 승인 완료).**

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

2026-08-16 기준 **G0 (Runtime)**, **G1 (World Integrity)**, **G2 (Local Movement)**, **G3 (Density / Displacement)**가 구현·검증·승인 완료되었다. G2는 사용자가 개선된 128×128 가상 숲 movement demo를 직접 실행해 ("잘된다") 승인했다. G3는 사용자가 개선된 laboratory `--density-demo`를 직접 실행해 약 300 ticks를 관찰한 뒤 Sand/Water 침강, Water/Oil 층분리, Steam/Smoke 정렬이 관찰 가능한 수준으로 동작함을 승인했다.

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
  steam_and_smoke_rise                               PASS
  gas_takes_up_diagonal_when_up_blocked              PASS
  gas_stable_bulk_center_does_not_swap               PASS (Gas↔Gas 무의미 swap 없음)
  contention_exactly_one_winner_no_duplication       PASS (winner exactly one,
                                                        loser valid, matter conserved)
  chunk_boundary_movement_is_plain_local_movement    PASS (63↔64 경계 양방향)
  void_exit_loses_exactly_one_matter                 PASS (open boundary로 실제 소멸,
                                                        OOB memory access 없음)
  liquid_exits_through_open_side_boundary            PASS (side opening Void exit,
                                                        water count 정확히 -1)
  powder_diagonal_void_exit                          PASS (diagonal OOB Void exit,
                                                        sand count 정확히 -1)
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
  sand_swaps_with_water_below                        PASS (Sand directly above Water →
                                                        local swap, Sand 아래/Water 위,
                                                        count 보존)
  sand_sinks_through_water_column                    PASS (여러 tick 후 Sand가 Water
                                                        아래로 진행 — pixel checksum 없음)
  water_below_oil_inversion                          PASS (Water above Oil → swap → 정렬)
  oil_above_water_is_stable                          PASS (density 이유만으로 swap 없음)
  multi_cell_layer_separation_in_basin               PASS (inverted/mixed → Water가
                                                        Oil보다 아래 semantic ordering)
  equal_rank_water_water_no_swap                     PASS (same rank density swap 없음,
                                                        쓸데없는 흔들림 없음)
  static_exclusion_stone_and_boundary                PASS (Sand vs Stone / Water vs
                                                        Boundary → density swap 없음)
  gas_rank_steam_rises_above_smoke                   PASS (Steam 20 below Smoke 30 →
                                                        lighter가 위로 정렬)
  gas_stable_ordering_no_swap                        PASS (반대 stable ordering에서는
                                                        density swap 없음)
  overlapping_swap_and_move_no_corruption            PASS (swap+move 겹침 → no duplicate,
                                                        no unexplained loss, per-material
                                                        count conserved)
  overlapping_swap_pair_no_corruption                PASS (swap+swap 겹침 → 동일)
  contention_no_corruption                           PASS (multiple sources overlapping
                                                        density candidates)
  chunk_boundary_density_displacement                PASS (y=63/64 chunk 경계 Sand/Water
                                                        displacement — chunk는 density
                                                        wall 아님)
  void_regression_bottom_side_diagonal               PASS (G2 Void semantics 유지)
  g3_tick_preserves_g2_contracts                     PASS (invalid ID/OOB 거부, EMPTY
                                                        미등록, boundary erase 유지)

Core pure tests (movement.rs/material.rs, 56개 total 중 G3 관련):
  sand_rank_gt_water / water_rank_gt_oil / steam_rank_lt_smoke   PASS
  EMPTY no rank / Stone·Boundary no movable density             PASS
  equal rank → no displacement / STATIC target → no displacement PASS
  sand downward into water allowed / water downward into oil allowed PASS
  oil downward into water rejected / steam upward into smoke allowed PASS
  lateral density swap rejected                                  PASS
  G2 movement/stencil/Void pure tests 전부 유지                   PASS

Density demo (User Validation fixture) — 128×128 laboratory tanks (승인 완료):
  실행:           cargo run -p powdergame-windows -- --density-demo
                  또는 상위 폴더의 run_powdergame.bat
  world/view:     G2와 동일한 128×128 square-cell / PAUSED / 15 TPS /
                  SPACE·N·R·ESC 관찰 구조 재사용 (새 UI framework 없음)
  scene:          G2 forest/tree divider를 쓰지 않는 별도 laboratory/tank 장면
                  좌→우 SAND+WATER | WATER+OIL | STEAM+SMOKE
                  - Tank 1: 하단 Water pool 위 큰 Sand block — PLAY 시 침강
                  - Tank 2: inverted (Water 위 / Oil 아래) → 층분리
                  - Tank 3: sealed (Smoke 위 / Steam 아래) → gas ordering
  presentation:   G3-only lab palette (Stone = gray). G2 forest green palette와
                  movement-demo 장면은 변경 없음
  title:          Powdergame G3 Density Demo | SAND+WATER | WATER+OIL |
                  STEAM+SMOKE | [PAUSED]/[PLAY 15 TPS] + tick count
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

G3 User Validation:
  PASS — 사용자가 개선된 laboratory `--density-demo`를 직접 실행해 약 300 ticks
  진행 화면을 확인하고, Sand/Water 침강 · Water/Oil 층분리 · Steam/Smoke 정렬이
  관찰 가능한 수준으로 동작함을 승인 (2026-08-16).

M0 전체는 여전히 `IN_PROGRESS` — `ACHIEVED`가 아니다. G4~G9와 최종 사용자 승인이 남아 있다.

### Product Direction

> **현실을 구현하는 것이 아니라 가상의 재미있는 놀이터를 만든다. 핵심은 나만의 세계 창조다.**

현실의 자연현상은 reference이며 Powdergame 내부의 이해 가능한 논리와 상호작용이 우선한다.

### Next Action

1. G4 — Temperature / Phase / Combustion 준비 (이 G3 기준선에서 별도 branch 생성).
2. G4 구현은 그 branch에서만 시작한다. 이 G3 기준점은 변경하지 않는다.

아직 G4 구현은 시작하지 않는다.

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

---

## Approval State

Foundation Design direction: **APPROVED BY USER**

M0 implementation: **IN_PROGRESS** — G0/G1/G2/G3 PASS (G2·G3는 User Validation 포함), G4~G9 + 최종 M0 승인 남음

M0 `ACHIEVED`: **NO**

최종 M0 완료는 실제 구현/benchmark/play validation 후 사용자 승인이 필요하다.

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
tests: passed (56 core + 15 GPU density + 1 GPU headless smoke + 16 GPU movement + 7 GPU world integrity, ignored 1 controlled benchmark)
benchmarks: G2 controlled reference-world baseline (2048x2048 initial world, Boundary ring + EMPTY interior): release, idle machine, 100 warm-up ticks + GPU completion, 1000 measured ticks x 5 runs, GPU completion included — median ~0.146 ms/tick (~6838 TPS, RTX 5090/DX12, coarse end-to-end incl. GPU completion). Reference-scenario baseline only; NOT an active/heavy-matter gameplay benchmark. G3 controlled baseline: deferred (idle-machine 확인 후 별도 측정).
m0_status: IN_PROGRESS (G0 complete, G1 complete, G2 PASS/CLOSED incl. user validation 2026-08-16, G3 PASS/CLOSED incl. user validation 2026-08-16, G4+ pending)
```
