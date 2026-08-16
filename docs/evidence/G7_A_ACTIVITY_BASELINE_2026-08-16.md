# G7-A — Chunk Activity Observatory / Measurement Baseline (2026-08-16)

G7 — Active / Sleep gate, sub-step A.

- Base SHA: `53f7c7e23cae720840fed9dff552a296110e9018` (G6 PASS / CLOSED freeze point)
- Branch: `feature/m0-g7-active-sleep`
- World: dense SoA storage 그대로 (`material` / `temperature` / `pressure` / `flags`). Cell state를 sparse container로 바꾸지 않았다.
- 철학: **Dense State, Sparse Work** — Matter count가 아니라 changeable frontier가 계산 필요성을 결정.

이번 G7-A는 **측정/시각화 baseline**이다. 아직 aggressive sleep optimization, GPU active-list compaction, indirect dispatch, 실제 subsystem skip은 하지 않는다.

**Semantic hardening round (후속)**: phase transition이 실제로 발생한 tick을 activity buffer에 self-mark (phase pass) + detector의 phase-condition 방어적 체크 + PRESSURE activity를 pressure-medium(LIQUID/GAS) cell로 제한 + 문서 문구를 실제 구현과 100% 일치하도록 정정. G7-A는 계속 VALIDATION candidate. commit `fix: harden G7 activity semantics`.

---

## 1. Chunk activity state

GPU-side 64×64 chunk 기준 auxiliary diagnostic state (world.rs 버퍼 3종):

- `chunk_activity: Vec<u32>` — per-chunk bitmask (MATTER / THERMAL / PRESSURE / REACTION)
- `chunk_changed: Vec<u32>` — per-chunk "이번 tick에 meaningful change" (stable-duration reset 원인)
- `chunk_stable: Vec<u32>` — per-chunk consecutive-stable-ticks counter

Bit layout (engine/core/src/activity.rs, shared constants):

```text
ACTIVITY_MATTER     = 1 << 0
ACTIVITY_THERMAL    = 1 << 1
ACTIVITY_PRESSURE   = 1 << 2
ACTIVITY_REACTION   = 1 << 3
```

Epsilon baseline (측정 목적, 추측 고정 아님 — 추후 조정 가능):

```text
THERMAL_ACTIVITY_EPS  = 0.001
PRESSURE_ACTIVITY_EPS = 0.001
```

이 버퍼들은 **diagnostic state**이지 per-cell simulation state가 아니다. Simulation semantics에 영향 없음.

## 2. Activity bit 의미

- **MATTER_ACTIVE**: movable Matter가 EMPTY interface에 인접, density ordering 후보 존재, 또는 movement frontier 생성 가능. *"해당 material이 존재함"과 같지 않다.*
- **THERMAL_ACTIVE**: 4-neighbor temperature gradient > EPS, burning·heat source, **phase rule이 현재 자기 Material + Temperature에서 성립** (detector의 방어적 체크), 또는 **이번 tick에 실제로 발생한 phase transition** (phase pass가 activity buffer에 self-mark).
- **PRESSURE_ACTIVE**: neighbor pressure 차이 > EPS — **pressure medium(LIQUID/GAS) cell에서만 평가** (G5 계약: EMPTY/STATIC/POWDER는 medium이 아니며 pressure field가 매 tick 0으로 정리되므로 pressure frontier를 가질 수 없음).
- **REACTION_ACTIVE**: burning Wood/Oil, decay 진행 Matter, reaction state가 실제 변화 중.

Phase transition은 1:1 write-self이므로 rule이 성립하는 cell은 같은 tick 안에서 반드시 변환된다 (hysteresis band가 변환 후 상태를 안정으로 보장). 따라서 end-of-tick 측정 기준으로는 "대기 중인 phase candidate"가 존재할 수 없고, phase pass가 transition tick을 `cell_activity`에 직접 표시하는 것이 실제 관측 가능한 신호다. detector의 phase-condition 체크는 방어적(defensive)이다 — 향후 semantics가 candidate를 남기는 경우를 대비한다.

## 3. Same-Matter no-op audit

`movement_propose.wgsl` audit 결과:

- NORMAL move는 **EMPTY destination만** 허용 — same-ID Matter는 candidate가 될 수 없음 (EMPTY가 아니면 blocked).
- Density candidate는 rank ordering 필요: `source_rank > dest_rank` (POWDER/LIQUID down/down-diag), `source_rank < dest_rank` (GAS up/up-diag). **equal rank는 절대 swap하지 않음** — Water↔Water / Oil↔Oil / Steam↔Steam 등 same-ID 및 same-rank pair가 무의미한 ownership edge를 생성하는 경로 없음.
- Lateral candidate는 EMPTY-only (density lateral jitter 없음).
- Temperature/Pressure/Flags가 함께 운반되는 실제 state transport가 있는 경우는 Matter identity와 함께 이동하므로 same-material no-op으로 오분류하지 않는다 (flags[]는 Matter-owned contract, G4-C/G6 동결).

새 regression test: `same_matter_noop_does_not_create_false_activity` — stable same-matter bulk가 MATTER_ACTIVE로 오판되지 않는지 확인.

## 4. Stable-duration 측정

`chunk_stable[]`는 "이 chunk가 몇 tick 연속 meaningful change 없이 안정 상태였나"를 관찰한다.

- meaningful change가 없으면 +1 (포화 0xFFFF_FFFF).
- `chunk_changed` 비트가 set되면 0으로 reset.

G7-A에서는 이 값을 **sleep cutoff로 사용하지 않는다.** 1/2/4/8/16/32/64… tick stable 분포를 먼저 관찰한다. Sleep threshold는 임의로 선택하지 않는다.

## 5. Wake reason model

진단 수준으로 정의/계측:

```text
SELF_CHANGED      — 자기 chunk가 meaningful change
NEIGHBOR_ACTIVE   — 이웃 chunk의 frontier 영향 (activity_reduce에서 이웃 chunk bit 참조)
EDIT              — 사용자 edit (향후 G7-B에서 wake trigger)
PHASE_CHANGE / REACTION / THERMAL_FRONT / PRESSURE_FRONT
```

G7-A에서는 aggregate diagnostic만 유지 (persistent per-reason 저장 없음). 모든 reason을 영구 저장하지 않는 쪽으로 싸게 유지.

## 6. GPU passes

진단 pass 2개 (G3부터의 shader-per-pass 구조, string scanner 없음):

- `activity_propose.wgsl` — cell 단위로 4 activity bit를 평가 (EMPTY/STATIC/movable stencil, neighbor temperature/pressure gradient — pressure는 medium만, burning·decay flag, phase condition). `cell_activity`에 **OR-merge** (phase pass가 mid-tick에 설정한 transition 마커를 보존).
- `activity_reduce.wgsl` — 64×64 chunk별 reduction: cell bit OR 합산 → `chunk_activity`, `chunk_changed_this_tick` (= 이 chunk에 이번 tick activity(frontier)가 있었는지, mask != 0 → 1), `chunk_stable` update.
- `phase_transition.wgsl` — physics pass이지만 G7-A 진단을 위해 매 tick 모든 cell의 `cell_activity` THERMAL bit를 clear 후, transition이 실제 발생한 cell에만 다시 set (self-write). physics state에는 영향 없음.

**Chunk 경계 정확한 의미**: `activity_propose`의 cell-level stencil은 **world 좌표로 1-cell neighbor를 읽으므로** chunk seam 반대편의 Matter/field도 정상 감지된다 — seam이 activity detection wall이 아니다. **dedicated chunk-to-chunk wake propagation pass는 아직 없다** (그 기능은 G7-B에서 actual sleep과 함께 구현). `chunk_changed`는 "이번 tick에 activity(frontier) 존재"를 의미하며 **이전/다음 state를 비교하는 dirty tracking이 아니다** — state-delta dirty tracking이 필요하면 G7-B 별도 설계.

`parallel_integrity` write-contract (phase pass의 `cell_activity` read-write 포함) / `wgsl_parse` 테스트에 등록.

## 7. --activity-demo (G7 Activity Observatory)

- World: 256×256 (4×4 chunks of 64), 60 TPS, square-cell aspect-preserving presentation.
- Title: `G7 ACTIVE / SLEEP OBSERVATORY — Stable Bulk vs Active Frontier`
- 2×2 panel:

```text
[A] STABLE WATER BULK     [B] STABLE STEAM / GAS BULK
[C] WAKE PROPAGATION      [D] SLOW ACTIVE WORLD
```

- **A**: 큰 밀폐 Water tank — settling 후 bulk 내부 activity 급감, surface/interface만 유지.
- **B**: 밀폐 Steam/Gas 공간 — 존재만으로 모든 chunk가 영구 active가 아님을 관찰.
- **C**: 좌측 stable bulk + 우측 Sand/Water/heat frontier 접근 — stable duration 증가 → 영향 접근 → neighbor wake candidate 발생 계측 (실제 sleep 없이 "would sleep/wake candidate" 정확 계측).
- **D**: 천천히 타는 Wood + thermal gradient + 소형 pressure area — 주변 stable chunk는 안정, burning/thermal/pressure frontier chunk는 activity 유지 ("안 움직인다" ≠ "계산할 필요 없다" 증명).

HUD (G4~G6 screen-space text renderer 재사용):

```text
SIM TICK / DIAGNOSTIC SAMPLE
Total Chunks / Matter Active / Thermal Active / Pressure Active / Reaction Active
Fully Stable / Max Stable Ticks
```

chunk heatmap overlay (PresentationPalette::Activity): inactive candidate = dark, activity bit별 색 — 진단 가독성 우선, 색을 겹쳐 복잡하게 만들지 않음.

## 8. Automated tests (engine/gpu/tests/activity.rs, 23 passed)

Baseline (15):

- `stable_stone_chunk_reports_inactive`, `stable_water_bulk_reports_no_internal_movement_frontier`
- `same_matter_noop_does_not_create_false_activity` (same-Matter no-op audit regression)
- `water_empty_interface_reports_matter_active`, `density_inversion_reports_active`
- `thermal_gradient_reports_thermal_active`, `pressure_gradient_reports_pressure_active`
- `burning_wood_reports_reaction_active`
- `stable_duration_increments_only_when_no_meaningful_change`, `meaningful_change_resets_stable_duration`
- `neighbor_activity_does_not_falsely_wake_adjacent_stable_chunk`
- `chunk_boundary_frontier_marks_both_relevant_chunks`
- False-sleep hazard: `sand_falling_into_water_wakes_interface`, `thermal_frontier_wakes_cold_steam_candidate`, `ignition_heat_wakes_sleep_candidate_wood`

Semantic hardening (8):

- Phase zero-gradient positive: `uniform_water_above_boil_threshold_reports_thermal_active`, `uniform_steam_below_condense_threshold_reports_thermal_active`, `uniform_water_below_freeze_threshold_reports_thermal_active`, `uniform_ice_above_melt_threshold_reports_thermal_active` — 전 세계 균일 T (ring 포함, gradient 0), sealed chamber → THERMAL은 phase transition marker 때문에만 발생.
- Phase negative: `uniform_water_inside_phase_hysteresis_without_gradient_can_be_inactive` — T=0 hysteresis → activity 0, stable counter 증가.
- Cross-chunk: `cross_chunk_thermal_frontier_detected`, `cross_chunk_pressure_frontier_detected` — seam x=63/64 양쪽 chunk 모두 감지 (cell-level stencil이 world 좌표로 seam을 넘어 읽음).
- Pressure-medium audit: `non_medium_cells_do_not_report_pressure_activity` — Stone-only chunk가 이웃 pressured Water 때문에 PRESSURE로 오보되지 않음.

## 9. Validation (FAST)

```text
cargo fmt --all -- --check                       PASS
cargo check --workspace --all-targets            PASS (warning 0)
cargo test -p powdergame-gpu --test activity -- --test-threads=1   PASS — 23 passed
cargo test -p powdergame-gpu --test phase -- --test-threads=1       PASS — 16 passed (regression)
cargo test -p powdergame-gpu --test wgsl_parse                       PASS — 1 passed
cargo test -p powdergame-gpu --test parallel_integrity -- --test-threads=1   PASS — 12 passed (write contract incl. phase cell_activity)
cargo test -p powdergame-windows                 PASS — 7 passed
--activity-demo --smoke-frames 300               exit 0 (device loss 0)
--smoke-frames 60                                exit 0 (marker=1)
--density-demo --smoke-frames 180                exit 0
```

Instrumentation overhead sanity: 진단 pass들은 매 tick 실행되지만 reference-world smoke는 정상 동작 (marker=1). **공식 performance benchmark는 실행하지 않았다** (G8이 공식 Performance Evidence Gate; 기존 ignored performance tests 유지).

## 10. Limits / next

- 이번 G7-A는 measurement baseline. 아직 실제 work skipping / sleep cutoff / active-list compaction / indirect dispatch 없음.
- `chunk_stable` 값과 분포를 사용자 관찰로 확보한 뒤 G7-B에서 sleep/wake cutoff와 wake reason 기반 correctness를 설계한다.

G7 = IN_PROGRESS. G7-A = VALIDATION candidate. G7-B/C = PLANNED. G7 PASS/CLOSED 아님.
## 11. G7-A observatory fixture hardening (2026-08-17, `fix: harden G7 activity observatory fixture`)

Trigger: 사용자 장기 관찰(tick ~0/117/411/679/1515/3019)에서 HUD 누락, F fast-forward 미동작, detector가 frozen G4/G5가 실제로 수행하지 않는 work를 active로 보고하는 correctness mismatch, fixture 결함이 확인됨. authoritative contract: `docs/planning/G7_A_OBSERVATORY_HARDENING.md`.

### 11.1 Detector correctness fixes (production physics untouched)

- **Stale activity bits (bug)**: propose pass가 `mask | cell_activity[index]`로 OR-merge → 이전 tick의 MATTER/PRESSURE/REACTION bit가 frontier 소멸 후에도 생존, stable counter가 영구 reset될 수 있었음. 수정: `cell_activity[index] = mask | (cell_activity[index] & ACTIVITY_THERMAL)` — phase pass의 이번-tick THERMAL transition marker만 보존하고 나머지 bit는 매 tick overwrite. `cell_activity`는 history buffer가 아님.
- **THERMAL = frozen G4 participation**: conductivity table을 read-only binding으로 추가 (phase table과 단일 `ActivityTables` storage buffer로 결합 — DX12 per-stage storage-buffer limit 8 준수). thermal edge는 양쪽 endpoint가 모두 Matter이고 `min(k_self, k_neighbor) > 0`일 때만 THERMAL work. EMPTY는 thermal medium이 아니고 Boundary Block은 K=0이므로, 이 둘을 건너는 온도차는 frontier가 아님. `THERMAL_ACTIVITY_EPS` 변경 0, thermal.wgsl 수정 0.
- **PRESSURE = frozen G5 medium exchange**: self와 neighbor가 모두 pressure-medium(LIQUID/GAS)일 때만 비교. pressured medium이 Stone/EMPTY와 만나는 경계는 frontier 아님. `PRESSURE_ACTIVITY_EPS` 변경 0, pressure.wgsl 수정 0.

### 11.2 New GPU regressions (activity 23 → 29 passed)

| test | meaning |
|---|---|
| `hot_matter_next_to_empty_does_not_false_report_thermal` | G4: EMPTY는 thermal medium 아님 |
| `temperature_difference_across_boundary_block_is_inactive` | Boundary Block K=0 → frontier 아님 |
| `conductive_stone_gradient_reports_thermal_active` | Stone↔Stone 실제 전도 edge는 THERMAL |
| `matter_frontier_clears_when_settled` | MATTER frontier 소멸 → bit clear + stable 재개 |
| `pressure_frontier_clears_when_uniform` | 압력 확산 후 균일 → PRESSURE clear + stable 재개 |
| `reaction_frontier_clears_when_extinguished` | sustain 이하 냉각 → REACTION clear + stable 재개 |
| `uniform_pressurized_medium_sealed_by_stone_is_not_pressure_frontier` | 기존 잘못된 기대값 정정 (Stone 경계는 frontier 아님) |

### 11.3 Demo fixture hardening

- **Central isolation**: 중앙 십자벽을 Stone → **MATERIAL_BOUNDARY_BLOCK (K=0)**. 네 패널이 열적으로 분리됨 — 3000-tick 검증에서 A control chunk가 mask 0 유지 (cross-panel THERMAL contamination 0).
- **Panel B (TRUE stable Steam control)**: Stone shell + Steam 모두 T=80, no EMPTY interface / no staged pressure / no staged reaction → 네 chunk가 mask 0, stable counter 단조 증가. Gas existence != Activity.
- **Panel C (STABLE DURATION / WAKE CANDIDATE)**: Sand source를 upper-right C chunk(cx=1,cy=2)에만 배치; lower-right target chunk(cx=1,cy=3)는 먼저 stable → 생산 movement로 y=192 seam을 넘어 Sand 도착 → stable reset. wake-candidate 관측 (actual sleep/wake 없음).
- **Panel A**: draining column을 sealed shaft+catch basin에 격리 — C 오염 방지.
- **Sampled Wake Candidates**: 첫 diagnostic sample은 sentinel `ACTIVITY_NO_PREV_SAMPLE`(u32::MAX) baseline만 설정, 이후 0→nonzero sampled transition만 카운트. `wake_events` 과장 표현 제거.

### 11.4 3000-tick actual-fixture validation (RTX 5090 / DX12, release)

`cargo test --release -p powdergame-windows activity_demo_long_run_3000 -- --ignored --nocapture --test-threads=1` → PASS (4.62s).

실제 `stage_activity_demo()` geometry를 3000+ production `Simulation::tick()`으로 실행:

- Panel C: **pre-arrival stable = 1 → first MATTER arrival tick = 27 → reset_to_zero = true**
- Panel B: 4 chunks, sampled late ticks(t=500/1000/3000) 전부 mask 0, stable ≥ 1000 (단조 증가)
- Panel A: sealed control chunks mask 0 (cross-panel THERMAL contamination 0)
- Panel D: 자연 진화 (강제 active 유지 없음)
- device loss 0 / panic 0 / invalid state 0

### 11.5 Validation

```text
cargo fmt --all -- --check              PASS
cargo check --workspace --all-targets   PASS (warning 0)
activity 29 passed | thermal 13 | pressure 8 | phase 16 | wgsl_parse 1 | parallel_integrity 12 | windows 7 (+1 ignored long-run)
cargo test --release -p powdergame-windows activity_demo_long_run_3000 -- --ignored   PASS
--activity-demo --smoke-frames 300      exit 0 (Activity HUD 렌더링 포함)
```

성능 benchmark 실행 안 함 (G8 공식 Gate). production physics diff 0. activity EPS 변경 0.

### 11.6 Launch BAT (mandatory deliverable)

`run_g7_activity_demo.bat` (repository root, committed):

- `cd /d "%~dp0"` — BAT가 있는 worktree에서만 build/run
- `set "RUST_LOG=warn"` — 노이즈 감소
- incremental release build 후 로컬 `targetelease\powdergame-windows.exe --activity-demo` 직접 실행
- build 실패 시 명확한 메시지 + `pause` + `exit /b 1`
- controls 표시: `SPACE Play/Pause | F Fast x1/x4/x16 | N Step | R Reset | ESC Quit`
- `cargo clean` 없음, 절대 경로 없음

G7 = IN_PROGRESS. G7-A = VALIDATION candidate / AWAITING USER RE-VALIDATION. G7-B/C = PLANNED. G7 PASS/CLOSED 아님.
