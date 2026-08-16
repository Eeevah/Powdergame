# G7-B — Actual Sleep / Wake Correctness Evidence (2026-08-17)

G7 — Active / Sleep gate, sub-step B.

- **Original Reviewed Candidate**: `3139ffc1524c94957d2e611ae98a296bcc988676`
- **Red-Team Correctness Fix**: `2e90869b8ddc3d44618a2c264c1f9e099c7d5cc6`
- **Status**: `G7-B implementation complete / PASS candidate (Independent re-review + user validation pending)`
- **Philosophy**: **Dense State, Sparse Work** — dense SoA storage 유지 (`material` / `temperature` / `pressure` / `flags`), Cell state를 sparse container로 바꾸지 않음.
- **Core Principle**: Stable bulk chunk에서 실제 simulation work를 생략하고, 필요한 순간에는 절대로 늦지 않게 Wake하며, Sleep ON과 기존 Always-Active 결과가 의미적으로 완전히 같음을 증명한다.

---

## 1. Architectural Design & Invariants

### 1.1 Dense State, Sparse Work
- Dense world arrays (`material_current`, `material_next`, `temperature_current`, `temperature_next`, `pressure_current`, `pressure_next`, `flags_current`, `flags_next`) remain uniformly allocated and authoritative on GPU.
- Chunks (64×64 cells) evaluate their activity bitmask and consecutive stable ticks.
- When a chunk meets the sleep threshold and has 0 self-activity, 0 neighbor activity, and 0 user edit triggers, it transitions to `CHUNK_STATE_SLEEPING` (1).
- In sleeping chunks, all 14 simulation compute pipelines skip physical stencil evaluations and cleanly carry forward existing state:
  ```wgsl
  if (params.sleep_enabled != 0u) {
      let cx = (index % params.width) / params.chunk_size;
      let cy = (index / params.width) / params.chunk_size;
      if (chunk_state[cy * params.chunks_x + cx] != 0u) {
          material_next[index] = mat;
          temperature_next[index] = temp;
          flags_next[index] = flags;
          return;
      }
  }
  ```

### 1.2 The 14 Guarded Subsystem Passes
Every simulation compute shader carries the standard uniform layout (`params: Params` with `sleep_enabled: u32`) and reads `chunk_state: array<u32>`:
1. `movement_propose.wgsl` (binding 6: `chunk_state`)
2. `movement_claim.wgsl` (binding 4: `chunk_state`)
3. `movement_commit.wgsl` (binding 8: `chunk_state`)
4. `thermal.wgsl` (binding 6: `chunk_state`)
5. `phase_transition.wgsl` (binding 7: `chunk_state`)
6. `expansion_claim.wgsl` (binding 5: `chunk_state`)
7. `expansion_spawn_commit.wgsl` (binding 7: `chunk_state`)
8. `expansion_pressure.wgsl` (binding 8: `chunk_state`)
9. `decay.wgsl` (binding 8: `chunk_state`)
10. `combustion.wgsl` (binding 9: `chunk_state`; 512B uniform combustion table allows 8 storage buffers on DX12)
11. `smoke_claim.wgsl` (binding 5: `chunk_state`)
12. `smoke_commit.wgsl` (binding 5: `chunk_state`)
13. `pressure.wgsl` (binding 5: `chunk_state`)
14. `rupture.wgsl` (binding 8: `chunk_state`)

### 1.3 GPU Wake Pass & Two-Phase Edit Snapshot
Before any simulation pass dispatches, the dedicated `activity_wake_pass` executes on GPU:
- Dispatches `(chunks_x, chunks_y, 1)` workgroups with 1 thread per chunk.
- Evaluates:
  1. `sleep_enabled == 0` → `CHUNK_STATE_RUNNABLE` (reason: `WAKE_REASON_ALWAYS_ACTIVE`)
  2. `chunk_activity[self] != 0` → `CHUNK_STATE_RUNNABLE` (reason: `WAKE_REASON_SELF_ACTIVITY`)
  3. `chunk_edit_wake[self] != 0` → `CHUNK_STATE_RUNNABLE` (reason: `WAKE_REASON_USER_EDIT`)
  4. **8-Neighbor Safety Halo**: `any(neighbor_activity != 0 || neighbor_edit != 0)` → `CHUNK_STATE_RUNNABLE` (reason: `WAKE_REASON_NEIGHBOR_HALO`)
  5. `stable_ticks < sleep_threshold` → `CHUNK_STATE_RUNNABLE` (reason: `WAKE_REASON_SETTLING`)
- If none of the above hold → `CHUNK_STATE_SLEEPING` (reason: `WAKE_REASON_NONE`).
- **Two-Phase Edit Wake Contract**:
  ```text
  wake dispatch reads one immutable chunk_edit_wake snapshot
  → wake pass ends
  → command encoder clears the complete edit-wake buffer
  → physics begins
  ```
  `chunk_edit_wake` is bound read-only in the shader to guarantee workgroups observe an immutable snapshot across chunk boundaries without write-race hazards.

---

## 2. Red-Team Correctness Fixes & Hardening

### 2.1 F1 — Combustion Sleeping Fuel-Progress Freeze
- **Defect**: The sleeping guard in `combustion.wgsl` previously wrote `flags_next[index] = flags & ~COMBUSTION_MASK;`, which cleared `FLAG_FUEL_PROGRESS_MASK`. Extinguished, partially-consumed fuel (e.g. Wood burned for 100 ticks) that fell asleep had its progress reset to 0, restoring consumed fuel when reignited.
- **Fix**: Replaced with exact carry-forward:
  ```wgsl
  material_next[index] = mat;
  temperature_next[index] = sanitize(temperature_current[index]);
  flags_next[index] = flags;
  proposal[index] = NO_SPAWN;
  return;
  ```
  Guarantees: material exact, temperature safe sanitization preserved, full flags exact carry-forward, no fuel progress change, no smoke proposal, no burn progression.

### 2.2 F2 — Edit Wake Halo Immutable Snapshot & Two-Phase Clear
- **Defect**: The wake shader previously bound `chunk_edit_wake` as `read_write` and cleared `chunk_edit_wake[chunk_idx] = 0u` inside the workgroup. Across separate workgroups, one chunk could clear its edit wake before an adjacent or diagonal neighbor read it, risking a missed halo wake.
- **Fix**:
  1. `activity_wake.wgsl`: `chunk_edit_wake` bound as `read` (immutable snapshot), in-shader writes completely removed.
  2. `simulation.rs`: Layout changed to `BindingKind::Read`.
  3. `simulation.rs` `tick()`: Two-phase clear `encoder.clear_buffer(&self.world.chunk_edit_wake, 0, None)` placed immediately after the wake pass and before the first physics pass (`propose_pass`).

### 2.3 Hardening — Decay Pure Sleep Freeze
- **Hardening**: `decay.wgsl` sleeping guard previously cleared decay age bits via `flags & ~FLAG_DECAY_AGE_MASK`. Updated to exact carry-forward `flags_next[index] = flags;` to strictly preserve all flag bits during sleep. Production reaction activity semantics remain unchanged.

### 2.4 Scratch Ownership Hygiene
- Red-Team confirmed proposal/claim scratch hygiene is strictly maintained without stale proposal/claim resurrection. No scratch buffer architecture redesign was needed.

---

## 3. Automated Test Suite (`sleep_wake.rs`) — All 17 Scenarios Passed

The dedicated integration test suite `engine/gpu/tests/sleep_wake.rs` now verifies all 17 scenarios and structural contracts:

| Scenario | Name | Test Purpose | Result |
|---|---|---|---|
| **A** | `test_scenario_a_stable_water_bulk_sleep` | Deep stable Water in Stone container settles to sleep after threshold ticks | **PASSED** |
| **B** | `test_scenario_b_stable_steam_bulk_sleep` | Hot Steam (80°C) in hot Stone container (80°C) with no gradient enters sleep | **PASSED** |
| **C** | `test_scenario_c_falling_sand_wakes_before_impact` | Sand falling toward sleeping target chunk wakes target chunk via 8-neighbor halo before crossing seam | **PASSED** |
| **D** | `test_scenario_d_cross_chunk_thermal_conduction_wake` | Hot source in chunk 0 wakes adjacent sleeping chunk 1 via halo before heat conducts across boundary | **PASSED** |
| **E** | `test_scenario_e_cross_chunk_pressure_wake` | Pressure impulse in chunk 0 wakes adjacent sleeping chunk 1 and diffuses across seam | **PASSED** |
| **F** | `test_scenario_f_diagonal_seam_movement_wake` | Sand sliding diagonally across chunk corner (0,0) → (1,1) wakes diagonal chunk via 8-neighbor halo | **PASSED** |
| **G** | `test_scenario_g_user_edit_wake` | Single cell edit in center chunk (1,1) immediately wakes center chunk and ALL 8 neighbor chunks | **PASSED** |
| **H** | `test_scenario_h_combustion_and_decay_never_sleep_while_active` | Burning Wood and decaying Smoke chunks are wake-locked and never enter sleep while reacting | **PASSED** |
| **I** | `test_scenario_i_phase_transitions_wake` | Heating stable sleeping Ice above 0°C immediately wakes chunks and executes melt transition to Water | **PASSED** |
| **J** | `test_scenario_j_scratch_hygiene_and_carry_forward` | 50 consecutive ticks in 100% sleeping world preserves material, temperature, pressure, flags byte-for-byte | **PASSED** |
| **K-1** | `test_scenario_k_mixed_long_run_equivalence` | Complex world run for 100 ticks with Sleep ON vs Sleep OFF produces identical material state | **PASSED** |
| **K-2** | `test_scenario_k_combustion_lifecycle_equivalence` | Full lifecycle: partial fuel (progress=100) → extinguish → sleep → reignite → exact remaining consumption | **PASSED** |
| **5.1** | `test_extinguished_wood_fuel_progress_survives_sleep_exact` | Extinguished Wood with fuel progress=100 preserves exact flags through stable transition into sleep | **PASSED** |
| **5.3** | `test_edit_wake_edge_snapshot_wakes_expected_halo` | User edit at seam (31,48) in 3x3 sleeping world wakes exact 6 chunks (0,1,3,4,6,7) and clears edit buffer | **PASSED** |
| **5.4** | `test_edit_wake_diagonal_corner_snapshot_wakes_expected_halo` | User edit at corner (31,31) in 3x3 sleeping world wakes diagonal chunk 4 on same tick via immutable halo | **PASSED** |
| **5.5** | `test_structural_wake_shader_and_simulation_race_contracts` | Structural check: shader binding 3 read-only, 0 shader writes, simulation Read binding, clear before propose | **PASSED** |
| **5.6** | `test_structural_decay_sleeping_guard_flags_freeze` | Structural check: decay.wgsl sleeping guard carries exact flags without clearing decay age mask | **PASSED** |

Execution log summary:
```text
running 17 tests
test test_edit_wake_diagonal_corner_snapshot_wakes_expected_halo ... ok
test test_edit_wake_edge_snapshot_wakes_expected_halo ... ok
test test_extinguished_wood_fuel_progress_survives_sleep_exact ... ok
test test_scenario_a_stable_water_bulk_sleep ... ok
test test_scenario_b_stable_steam_bulk_sleep ... ok
test test_scenario_c_falling_sand_wakes_before_impact ... ok
test test_scenario_d_cross_chunk_thermal_conduction_wake ... ok
test test_scenario_e_cross_chunk_pressure_wake ... ok
test test_scenario_f_diagonal_seam_movement_wake ... ok
test test_scenario_g_user_edit_wake ... ok
test test_scenario_h_combustion_and_decay_never_sleep_while_active ... ok
test test_scenario_i_phase_transitions_wake ... ok
test test_scenario_j_scratch_hygiene_and_carry_forward ... ok
test test_scenario_k_combustion_lifecycle_equivalence ... ok
test test_scenario_k_mixed_long_run_equivalence ... ok
test test_structural_decay_sleeping_guard_flags_freeze ... ok
test test_structural_wake_shader_and_simulation_race_contracts ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 38.09s
```

---

## 4. Full Workspace Verification

```text
Full Test Suite (`cargo test --workspace -- --test-threads=1`):
  - powdergame-core unit tests: 136 passed
  - GPU activity tests: 29 passed
  - GPU arbitration quality tests: 6 passed
  - GPU combustion & decay tests: 56 passed
  - GPU density tests: 15 passed
  - GPU expansion tests: 5 passed
  - GPU headless smoke test: 1 passed
  - GPU movement tests: 15 passed (2 ignored perf)
  - GPU parallel integrity tests: 12 passed
  - GPU phase transition tests: 16 passed
  - GPU pressure tests: 8 passed
  - GPU rupture & 2x2 multi-boiler stress lab tests: 7 passed
  - GPU sleep/wake correctness tests: 17 passed
  - GPU thermal conduction tests: 13 passed
  - GPU WGSL syntax parse tests: 1 passed
  - GPU world integrity tests: 7 passed
  - Windows observatory unit tests: 8 passed (1 ignored 3000-tick)
  - Total: 355 discovered, 352 passed, 3 ignored, 0 failed

Clippy Validation (`cargo clippy --workspace --all-targets -- -D warnings`):
  - 0 warnings, 0 errors

Performance Benchmarks:
  - NOT RUN (frozen production baseline; manual perf tests ignored)

Git Workspace Integrity (`git diff --check`):
  - 0 whitespace or formatting errors
```

---

## 5. Observatory HUD & Controls

The `--activity-demo` fixture features live G7-B Sleep/Wake diagnostics:
- **`S` Key**: Toggles Sleep Optimization `[ON]` / `[OFF (Always-Active Reference)]`.
- **`[` / `]` Keys**: Decreases / increases the sleep settling threshold (1~64 ticks).
- **HUD Global Card**:
  - Live counts for `Runnable Chunks` and `Sleeping Chunks` (with %).
  - Wake reason breakdown: `Self Activity`, `Neighbor Halo (8)`, `User Edit`, `Settling / Always`.
  - Guarded cell-passes skipped metric: `~X cell-passes / tick`.
- **Panel Status**:
  - `FULL BULK SLEEP (SPARSE SKIP)` in Bright Green when panel is fully sleeping.
  - `REACTIVE (WAKE LOCKED)` in Red during combustion/decay.
  - `MOVING FRONTIER (WAKE HALO)` in Green during frontier motion.
  - `THERMAL FRONT / PRESSURE FRONT (WAKE PROP)` during gradient conduction/diffusion.

---

## 6. Gate Status

- **G7-B Status**: `G7-B implementation complete / PASS candidate`
- **Independent Re-Review**: `PENDING`
- **User Validation**: `PENDING`
- **Final PASS / CLOSED / FROZEN declared**: `NO`
