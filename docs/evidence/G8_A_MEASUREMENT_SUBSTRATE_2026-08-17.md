# G8-A — Performance Measurement Substrate Evidence (2026-08-17)

G8 — Performance Evidence gate, sub-step A (Measurement Substrate).

- **Frozen Baseline SHA**: `94babb2667c081b5588489e1b4e710cc6efa68be`
- **Target Branch**: `feature/m0-g8-performance-evidence`
- **Primary Hardware**: NVIDIA GeForce RTX 5090 (Vendor: `0x10DE`, Device: `0x2B85`, Driver: `32.0.15.9636`)
- **Backend**: DirectX 12 (`wgpu` DX12 backend)
- **Status**: `G8-A SUBSTRATE COMPLETE / TRUSTWORTHY MEASUREMENT ESTABLISHED`
- **Core Principle**: Observational profiling only — observe the exact production pipeline without altering or perturbing simulation semantics.
- **Production Physics & Optimization Policy**: **NO PRODUCTION OPTIMIZATION PERFORMED**. G7-C (compact active lists, indirect dispatch) is **NOT IMPLEMENTED**. G8 official five-scenario matrix is **NOT STARTED**.

---

## 1. Profiling & Measurement Architecture

### 1.1 Observational Pipeline Non-Perturbation
Production `Simulation::tick()` and timestamp-profiled `Simulation::tick_profiled(&mut profiler)` share the exact same internal orchestrator (`tick_internal`), guaranteeing:
1. **Pass Ordering**: Identical 17 compute passes executed in identical causal sequence.
2. **Buffer Copies**: Identical intermediate Current/Next buffer copies and uniform updates.
3. **Zero Profiling Overhead on Production Path**: When unprofiled, `timestamp_writes` is `None`, with zero query sets bound and zero staging buffer copies.
4. **Byte-Exact Equivalence**: Matching 50-tick runs from identical active fixtures produce byte-exact equality across Material, Flags, Temperature, and Pressure (`test_profiled_vs_unprofiled_simulation_state_exact_equivalence`).

### 1.2 GPU Timestamp Query Implementation
- **Capability Check**: Adapter features are queried for `wgpu::Features::TIMESTAMP_QUERY`.
- **Feature Isolation**: `TIMESTAMP_QUERY` is requested only when profiling is explicitly enabled via `GpuContext::with_profiling()` or `ContextOptions { enable_profiling: true }`. Normal production runtime does not require it.
- **Timestamp Period**: On NVIDIA GeForce RTX 5090, `queue.get_timestamp_period()` reports `1.000000 ns/tick`.
- **Per-Pass Timestamp Writes**: `wgpu::ComputePassTimestampWrites` records start and end timestamps at pass boundaries:
  - Total passes: 17
  - Total queries per tick: 34 (`2 * 17`)
  - Pass $i$ begins at query $2i$ and ends at query $2i + 1$.
- **Query Resolution**: At the end of the profiled command encoder, `resolve_query_set(0..34)` writes to a 272-byte GPU resolve buffer, copied to a staging readback buffer.

### 1.3 The 17 Canonical Simulation Passes
The profiling substrate measures all 17 passes in execution order:

| # | Pass Name | Subsystem Category | Shader Label |
|---|---|---|---|
| 1 | `activity_wake` | Active / Sleep Management | `powdergame-g7b-activity-wake-pass` |
| 2 | `movement_propose` | Matter Movement | `powdergame-g3-propose-pass` |
| 3 | `movement_claim` | Ownership / Claim | `powdergame-g3-claim-pass` |
| 4 | `movement_commit` | Matter Movement | `powdergame-g3-commit-pass` |
| 5 | `thermal` | Thermal Conduction | `powdergame-g4a-thermal-pass` |
| 6 | `phase_transition` | Reaction / Phase | `powdergame-g4b-g5b-phase-pass` |
| 7 | `expansion_claim` | Ownership / Claim | `powdergame-g5b-expansion-claim-pass` |
| 8 | `expansion_spawn_commit` | Reaction / Phase | `powdergame-g5b-expansion-spawn-commit-pass` |
| 9 | `expansion_pressure` | Reaction / Phase | `powdergame-g5b-expansion-pressure-pass` |
| 10 | `decay` | Reaction / Phase | `powdergame-g4d-decay-pass` |
| 11 | `combustion` | Reaction / Phase | `powdergame-g4c-combustion-pass` |
| 12 | `smoke_claim` | Ownership / Claim | `powdergame-g4c-smoke-claim-pass` |
| 13 | `smoke_commit` | Reaction / Phase | `powdergame-g4c-smoke-commit-pass` |
| 14 | `pressure` | Pressure / Structure | `powdergame-g5a-pressure-pass` |
| 15 | `rupture` | Pressure / Structure | `powdergame-g5c-rupture-pass` |
| 16 | `activity_propose` | Active / Sleep Management | `powdergame-g7a-activity-propose-pass` |
| 17 | `activity_reduce` | Active / Sleep Management | `powdergame-g7a-activity-reduce-pass` |

---

## 2. Measurement Modes & Methodology

### 2.1 Mode A: Production Sustained Throughput
- **Execution**: Uses ordinary unprofiled `Simulation::tick()`.
- **Submission Pattern**: 1024 ticks submitted in batch without per-tick CPU/GPU synchronizations.
- **End Synchronization**: Calls `device.poll(wgpu::PollType::Wait)` ONCE at the end of the 1024-tick batch.
- **Rationale**: Waiting after every tick forces artificial CPU-GPU pipeline bubbles, measuring driver synchronization latency rather than GPU pipeline throughput. Batch submission measures true sustained production throughput.
- **Metrics**: Total elapsed wall time (ms), wall ms/tick, sustained TPS.

### 2.2 Mode B: GPU Breakdown
- **Execution**: Uses timestamp-profiled `Simulation::tick_profiled(&mut profiler)`.
- **Timings**: Collects all 17 individual pass durations, plus:
  - `gpu_tick_envelope_ms`: Query 0 (`activity_wake` start) to Query 33 (`activity_reduce` end).
  - `gpu_pass_sum_ms`: Sum of 17 pass durations.
  - `residual_ms`: `gpu_tick_envelope_ms - gpu_pass_sum_ms` (diagnostic residual including intermediate buffer copy times and scheduling overhead).

### 2.3 Non-Timed Activity Census
- Out-of-band diagnostic query returning:
  - Cell metrics: total cells, any active, Matter active, Thermal active, Pressure active, Reaction active.
  - Chunk metrics: total chunks, active chunks, runnable chunks, sleeping chunks.
- **Inviolable Rule**: Census readbacks are NEVER executed inside timed simulation loops.

### 2.4 Application-Tracked Memory Accounting
Reports exact tracked GPU buffer allocation sizes across all world, scratch, activity, uniform, and profiler allocations (`tracked_gpu_allocation_bytes`). Does not claim to represent physical OS driver-reported resident VRAM.

---

## 3. Reference Calibration Run Results (RTX 5090 / DX12)

Configuration: 2048×2048 reference world (4,194,304 cells), 64×64 chunks (1,024 chunks), Sleep Optimization ON (Threshold: 16 ticks), Release Profile (opt-level=3).

### 3.1 Mode A: Production Sustained Throughput (1024 ticks × 3 trials)

| Trial | Batch Ticks | Total Wall Time (ms) | Wall Time / Tick (ms) | Sustained TPS |
|---|---|---|---|---|
| Trial 1 | 1024 | 1176.97 ms | 1.1494 ms | 870.0 TPS |
| Trial 2 | 1024 | 1198.16 ms | 1.1701 ms | 854.6 TPS |
| Trial 3 | 1024 | 1175.06 ms | 1.1475 ms | 871.4 TPS |

**Summary Across 3 Trials**:
- **Sustained TPS**: **Median = 870.0 TPS** (Mean = 865.4 TPS, Min = 854.6, Max = 871.4)
- **Wall Time / Tick**: **Median = 1.1494 ms** (Mean = 1.1557 ms, Min = 1.1475, Max = 1.1701)

### 3.2 Mode B: GPU Breakdown (256 ticks × 3 trials, Median Trial Summary)

| # | Pass Name | P50 (ms) | P95 (ms) | Mean (ms) | % of Envelope |
|---|---|---|---|---|---|
| 1 | `activity_wake` | 0.0043 ms | 0.0047 ms | 0.0041 ms | 0.42% |
| 2 | `movement_propose` | 0.0327 ms | 0.0329 ms | 0.0328 ms | 3.20% |
| 3 | `movement_claim` | 0.0330 ms | 0.0337 ms | 0.0330 ms | 3.23% |
| 4 | `movement_commit` | 0.0395 ms | 0.0420 ms | 0.0398 ms | 3.86% |
| 5 | `thermal` | 0.0318 ms | 0.0321 ms | 0.0318 ms | 3.11% |
| 6 | `phase_transition` | 0.0333 ms | 0.0336 ms | 0.0342 ms | 3.26% |
| 7 | `expansion_claim` | 0.0332 ms | 0.0348 ms | 0.0340 ms | 3.24% |
| 8 | `expansion_spawn_commit` | 0.0326 ms | 0.0327 ms | 0.0332 ms | 3.19% |
| 9 | `expansion_pressure` | 0.0302 ms | 0.0303 ms | 0.0310 ms | 2.96% |
| 10 | `decay` | 0.0432 ms | 0.0444 ms | 0.0446 ms | 4.23% |
| 11 | `combustion` | 0.0423 ms | 0.0443 ms | 0.0426 ms | 4.14% |
| 12 | `smoke_claim` | 0.0331 ms | 0.0352 ms | 0.0332 ms | 3.24% |
| 13 | `smoke_commit` | 0.0313 ms | 0.0320 ms | 0.0313 ms | 3.06% |
| 14 | `pressure` | 0.0315 ms | 0.0324 ms | 0.0323 ms | 3.09% |
| 15 | `rupture` | 0.0318 ms | 0.0320 ms | 0.0318 ms | 3.12% |
| 16 | `activity_propose` | 0.0377 ms | 0.0388 ms | 0.0393 ms | 3.69% |
| 17 | `activity_reduce` | 0.2472 ms | 0.2480 ms | 0.2473 ms | 24.20% |
| **—** | **GPU Pass Sum** | **0.7688 ms** | **0.7740 ms** | **0.7764 ms** | **75.25%** |
| **—** | **GPU Tick Envelope** | **1.0217 ms** | **1.0307 ms** | **1.0295 ms** | **100.00%** |
| **—** | **Diagnostic Residual** | **0.2528 ms** | **0.2593 ms** | **0.2532 ms** | **24.75%** |

### 3.3 Grouped Subsystem Roll-Up (P50)

- **Matter Movement** (`propose` + `commit`): **0.0722 ms** (7.1%)
- **Ownership / Claim** (`movement_claim` + `expansion_claim` + `smoke_claim`): **0.0992 ms** (9.7%)
- **Thermal Conduction** (`thermal`): **0.0318 ms** (3.1%)
- **Reaction & Phase** (`phase` + `expansion_spawn` + `expansion_pressure` + `decay` + `combustion` + `smoke_commit`): **0.2129 ms** (20.8%)
- **Pressure & Rupture** (`pressure` + `rupture`): **0.0634 ms** (6.2%)
- **Active / Sleep Management** (`wake` + `activity_propose` + `activity_reduce`): **0.2892 ms** (28.3%)

### 3.4 Out-of-Band Activity Census Snapshot (at Tick 256)

- **Cells Total**: 4,194,304
- **Cells Any Active**: 266,016 (6.34%)
- **Cells Matter Active**: 220,275
- **Cells Thermal Active**: 79,795
- **Cells Pressure Active**: 1,898
- **Cells Reaction Active**: 66,504
- **Chunks Total**: 1,024
- **Chunks Active**: 219 (21.4%)
- **Chunks Runnable**: 381 (37.2%)
- **Chunks Sleeping**: 643 (62.8%)

### 3.5 Application-Tracked GPU Buffer Allocation Memory Report

- **World Dense State (8 buffers)**: 128.00 MB (134,217,728 bytes)
- **Movement Arbitration Scratch (2 buffers)**: 32.00 MB (33,554,432 bytes)
- **Activity Diagnostics (cell + 6 chunk buffers)**: 16.02 MB (16,801,792 bytes)
- **Uniforms & Tables**: 1.08 KB (1,104 bytes)
- **Profiler Resolve & Readback Staging**: 544 bytes
- **Total Application-Tracked GPU Memory**: **176.03 MB** (184,575,600 bytes)

### 3.6 Profiling Overhead Evaluation (256-Tick Matched Run)

- **Unprofiled 256 ticks**: 298.86 ms (1.1674 ms/tick)
- **Profiled 256 ticks**: 393.19 ms (1.5359 ms/tick)
- **Observed Overhead**: **31.56%**
- **Root Cause**: Per-tick synchronous GPU-to-CPU timestamp query buffer map/readback. This overhead is strictly isolated to Mode B and NEVER affects Mode A production throughput.

---

## 4. Automated Regression Verification

The targeted test suite `engine/gpu/tests/profiler.rs` verifies all architectural and observational invariants:

| Test Name | Purpose | Result |
|---|---|---|
| `test_ordinary_simulation_tick_does_not_require_profiling_feature` | Ordinary `Simulation::tick()` functions without `TIMESTAMP_QUERY` | **PASSED** |
| `test_profiled_simulation_tick_produces_17_valid_pass_timings` | Profiler returns 17 valid, non-negative, finite timings and valid envelope | **PASSED** |
| `test_profiled_vs_unprofiled_simulation_state_exact_equivalence` | 50-tick run produces byte-exact Material, Flags, Temp, Pressure match | **PASSED** |
| `test_activity_census_reports_accurate_cell_and_chunk_metrics` | Census correctly tracks pristine vs single-cell falling sand activation | **PASSED** |
| `test_tracked_gpu_allocation_report_structure` | Tracked memory arithmetic matches exact reference byte counts | **PASSED** |

Full workspace verification (`cargo test --workspace -- --test-threads=1`):
- **Discovered**: 362 tests
- **Passed**: 359 passed
- **Ignored**: 3 (2 manual performance benchmarks, 1 3000-tick DX12 stress lab)
- **Failed**: 0 failed

Static Analysis:
- `cargo fmt --all -- --check`: **PASS**
- `cargo clippy --workspace --all-targets -- -D warnings`: **PASS (0 warnings)**
- `git diff --check`: **PASS**

---

## 5. Gate Declarations & Scope Boundaries

- **G8-A Substrate**: `COMPLETE / TRUSTWORTHY MEASUREMENT ESTABLISHED`
- **G8 Final PASS**: `NO` (G8 is an in-progress milestone; G8-A establishes measurement substrate only).
- **G7-C (Compaction / Indirect Dispatch)**: `NOT IMPLEMENTED`
- **Production Physics Optimization**: `NONE PERFORMED`
- **G8 Official Five-Scenario Matrix**: `NOT STARTED` (Pending G8-B).
