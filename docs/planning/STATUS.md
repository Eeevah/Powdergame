# Powdergame Status

이 문서는 현재 실제 상태를 기록한다. 장기 방향은 `ROADMAP.md`, 완료 기준은 `MILESTONES.md`를 따른다.

---

## Human-maintained Status

### Current Milestone

`M0 — First World`

### Current Milestone Status

`IN_PROGRESS` — G0 (Runtime) PASS, G1 (World Integrity) PASS, G2 (Local Movement) PASS / CLOSED (User Validation 승인 완료 2026-08-16). G3~G9와 최종 M0 사용자 승인 남음.

### Current Phase

**G0 — Runtime: PASS. G1 — World Integrity: PASS. G2 — Local Movement: PASS / CLOSED (자동·기술 검증 + 성능 baseline 기록 + User Validation 사용자 승인 완료).**

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

2026-08-16 기준 **G0 (Runtime)**, **G1 (World Integrity)**, **G2 (Local Movement)**가 구현·검증·승인 완료되었다. G2는 사용자가 개선된 128×128 가상 숲 movement demo를 직접 실행해 ("잘된다") 승인했다.

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

M0 전체는 여전히 `IN_PROGRESS` — `ACHIEVED`가 아니다. G3~G9와 최종 사용자 승인이 남아 있다.

### Product Direction

> **현실을 구현하는 것이 아니라 가상의 재미있는 놀이터를 만든다. 핵심은 나만의 세계 창조다.**

현실의 자연현상은 reference이며 Powdergame 내부의 이해 가능한 논리와 상호작용이 우선한다.

### Next Action

1. G3 — Density / Displacement 준비: Density Rank + local displacement + layer
   separation Evidence Gate. (G2는 EMPTY destination 전용 baseline이며 density
   swap은 G3)
2. G2 기준선(feature/m0-g2-local-movement)에서 G3 branch 생성.
3. G3 구현 + 자동/기술 검증 후 보고.

아직 G3 구현은 시작하지 않는다.

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

---

## Approval State

Foundation Design direction: **APPROVED BY USER**

M0 implementation: **IN_PROGRESS** — G0/G1/G2 PASS (G2는 User Validation 포함), G3~G9 + 최종 M0 승인 남음

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
tests: passed (44 core + 1 GPU headless smoke + 16 GPU movement + 7 GPU world integrity, ignored 1 controlled benchmark)
benchmarks: G2 controlled reference-world baseline (2048x2048 initial world, Boundary ring + EMPTY interior): release, idle machine, 100 warm-up ticks + GPU completion, 1000 measured ticks x 5 runs, GPU completion included — median ~0.146 ms/tick (~6838 TPS, RTX 5090/DX12, coarse end-to-end incl. GPU completion). Reference-scenario baseline only; NOT an active/heavy-matter gameplay benchmark.
m0_status: IN_PROGRESS (G0 complete, G1 complete, G2 PASS/CLOSED incl. user validation 2026-08-16 — G3+ pending)
```
