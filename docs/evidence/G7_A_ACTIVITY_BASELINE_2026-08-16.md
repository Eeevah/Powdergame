# G7-A — Chunk Activity Observatory / Measurement Baseline (2026-08-16)

G7 — Active / Sleep gate, sub-step A.

- Base SHA: `53f7c7e23cae720840fed9dff552a296110e9018` (G6 PASS / CLOSED freeze point)
- Branch: `feature/m0-g7-active-sleep`
- World: dense SoA storage 그대로 (`material` / `temperature` / `pressure` / `flags`). Cell state를 sparse container로 바꾸지 않았다.
- 철학: **Dense State, Sparse Work** — Matter count가 아니라 changeable frontier가 계산 필요성을 결정.

이번 G7-A는 **측정/시각화 baseline**이다. 아직 aggressive sleep optimization, GPU active-list compaction, indirect dispatch, 실제 subsystem skip은 하지 않는다.

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
- **THERMAL_ACTIVE**: 4-neighbor temperature gradient > EPS, 또는 phase threshold 근처 변화 가능 / burning·heat source.
- **PRESSURE_ACTIVE**: neighbor pressure 차이 > EPS 또는 confinement/frontier.
- **REACTION_ACTIVE**: burning Wood/Oil, decay 진행 Matter, phase/reaction state가 실제 변화 중.

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

2개 명시적 진단 pass (G3부터의 shader-per-pass 구조, string scanner 없음):

- `activity_propose.wgsl` — cell 단위로 4 activity bit를 평가 (EMPTY/STATIC/movable, neighbor temperature/pressure gradient, burning·decay flag) → `cell_activity` scratch.
- `activity_reduce.wgsl` — 64×64 chunk별 reduction: cell bit 합산 → `chunk_activity`, `chunk_changed` (이전 tick과 비교), `chunk_stable` update. chunk 경계에서 이웃 chunk의 activity가 wake-candidate로 반영되도록 seam 처리를 포함 (chunk이 activity wall이 되지 않음).

tick 끝에 실행 (시뮬레이션 semantics에 영향 없음, read-only 진단). `parallel_integrity` write-contract / `wgsl_parse` 테스트에 두 shader 등록.

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

## 8. Automated tests (engine/gpu/tests/activity.rs, 15 passed)

- `stable_stone_chunk_reports_inactive`
- `stable_water_bulk_eventually_reports_no_internal_movement_frontier`
- `water_empty_interface_reports_matter_active`
- `density_inversion_reports_active`
- `burning_wood_reports_reaction_active`
- `thermal_gradient_reports_thermal_active`
- `pressure_gradient_reports_pressure_active`
- `neighbor_influence_produces_wake_candidate`
- `stable_duration_increments_only_when_no_meaningful_change`
- `meaningful_change_resets_stable_duration`
- `chunk_boundary_frontier_marks_both_relevant_chunks`
- `same_matter_noop_does_not_create_false_activity`
- + thermal/pressure frontier wake-candidate 및 stable-duration 경계 케이스

False-sleep hazard fixture 포함: stable Water candidate에 Sand 접근 → wake candidate / stable Steam에 thermal frontier 접근 → wake candidate / ignition heat 접근 → reaction·thermal wake candidate (future sleep correctness 근거).

## 9. Validation (FAST)

```text
cargo fmt --all -- --check                       PASS
cargo check --workspace --all-targets            PASS (warning 0)
cargo test -p powdergame-gpu --test activity -- --test-threads=1   PASS — 15 passed
cargo test -p powdergame-windows                 PASS — 7 passed
--activity-demo --smoke-frames 300               exit 0 (device loss 0)
--smoke-frames 60                                exit 0 (marker=1)
--density-demo --smoke-frames 180                exit 0
```

Instrumentation overhead sanity: 2개 진단 pass는 매 tick 실행되지만 reference-world smoke는 정상 동작 (marker=1). **공식 performance benchmark는 실행하지 않았다** (G8이 공식 Performance Evidence Gate; 기존 ignored performance tests 유지).

## 10. Limits / next

- 이번 G7-A는 measurement baseline. 아직 실제 work skipping / sleep cutoff / active-list compaction / indirect dispatch 없음.
- `chunk_stable` 값과 분포를 사용자 관찰로 확보한 뒤 G7-B에서 sleep/wake cutoff와 wake reason 기반 correctness를 설계한다.

G7 = IN_PROGRESS. G7-A = VALIDATION candidate. G7-B/C = PLANNED. G7 PASS/CLOSED 아님.
