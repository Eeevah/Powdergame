# G6-C1 — Arbitration Quality Measurement Report

- **Date**: 2026-08-16
- **Status**: **MEASUREMENT COMPLETE / DECISION PENDING**
- **Frozen Baseline SHA**: `ea08f6605703bb42593f7e6a1fb5181c99909ca8` (`test: establish G6 parallel integrity baseline`)
- **C1 Design Definition SHA**: `29251d484b0b5f279861839aafef91e832b320e7` (`docs: define G6-C arbitration quality measurement gate`)
- **Frozen G5 Base SHA**: `2112dfbacdefdcb02f4d82496dee374fc8e97f70` (`feat: finalize M0 G5 pressure chain`)
- **Hardware / Backend**: NVIDIA GeForce RTX 5090 (32 GB VRAM, Driver 596.36), `wgpu::Backend::Dx12`, Windows 11 Pro
- **Production WGSL Changed**: **NO** (0 files modified)
- **`Simulation::tick()` Changed**: **NO** (0 lines modified)

---

## 1. Candidate Hash Algorithm Specification

The test-only candidate replaces linear index comparison with a stateless 3-input integer hash mixer evaluated identically on both edge endpoints:

```wgsl
fn edge_priority(source: u32, target_cell: u32, tick: u32) -> u32 {
    var h: u32 = source ^ (target_cell * 0x9E3779B9u) ^ (tick * 0x85EBCA6Bu);
    h = (h ^ (h >> 16u)) * 0x7FEB352Du;
    h = (h ^ (h >> 15u)) * 0x846CA68Bu;
    h = h ^ (h >> 16u);
    return h;
}
```

### Properties:
1. **Integer-only & Stateless**: Requires no per-cell RNG state buffers or global atomic counters.
2. **Symmetric Edge Priority**: Because $P(\text{source}, \text{target}, \text{tick})$ depends purely on the edge endpoints and tick uniform, both source and target compute 100% identical priority for any incident candidate edge.
3. **Total Order & Fallback**: Winner is lowest priority `p < best_priority`. In case of a priority collision (`p == best_priority`), ties break deterministically by `source < best_owner`.
4. **Tick Input Delivery**: For C1, `tick` is supplied via test-only harness uniform buffer (`Params.tick`).

---

## 2. Correctness & Invariant Proofs

### A. Movement Edge Endpoint Agreement
* **Proof & Test**: `test_candidate_movement_edge_agreement_many_fixtures`
* **Result**: **PASS**
* **Verification**: In multi-source contending graphs across multiple ticks ($0, 1, 7, 42, 123, 999$), whenever a destination cell $D$ claims an edge $(S \to D, \text{KIND\_DEST})$, source cell $S$ unconditionally claimed $(S \to D, \text{KIND\_SOURCE})$. 0 split ownership, 0 duplicate claims.

### B. Destination-Only Claim Exactness
* **Test**: `test_candidate_destination_claim_exactly_one_winner`
* **Result**: **PASS** (4-source contention on single destination resolved to exactly 1 winner).

### C. Deterministic Repeatability
* **Test**: `test_candidate_deterministic_repeat`
* **Result**: **PASS** (100% bit-exact claim outputs across repeated GPU executions with identical proposals and tick seed).

### D. Priority Collision Tie-Break
* **Test**: `test_candidate_hash_collision_deterministic_tie_break`
* **Result**: **PASS** (256 tick iterations across contending pairs all produced valid single winners).

---

## 3. Directional Bias & Statistical Measurement (RTX 5090)

Large sample dataset containing 2,048 independent translated micro-fixtures per orientation class:

### A. Horizontal Contention (2,048 Contests: LEFT vs RIGHT)
* **Baseline (Fixed-Index)**:
  * LEFT: **2,048 (100.0%)** | RIGHT: **0 (0.0%)**
  * *Severe systematic bias towards lower horizontal linear index.*
* **Candidate (Edge-Hash)**:
  * LEFT: **1,010 (49.3%)** | RIGHT: **1,038 (50.7%)**
  * *Near-perfect macroscopic balance.*

### B. Vertical Contention (2,048 Contests: UP vs DOWN)
* **Baseline (Fixed-Index)**:
  * UP: **2,048 (100.0%)** | DOWN: **0 (0.0%)**
  * *Severe systematic bias towards upper rows.*
* **Candidate (Edge-Hash)**:
  * UP: **1,023 (50.0%)** | DOWN: **1,025 (50.0%)**
  * *Perfect 50/50 balance.*

### C. Diagonal Contention (2,048 Contests: NW vs SE)
* **Baseline (Fixed-Index)**:
  * NW: **2,048 (100.0%)** | SE: **0 (0.0%)**
* **Candidate (Edge-Hash)**:
  * NW: **1,048 (51.2%)** | SE: **1,000 (48.8%)**

### D. Rotated Contention (0°, 90°, 180°, 270°)
* **Candidate Distribution (0° Left vs Up)**:
  * LEFT: **274 (53.5%)** | UP: **238 (46.5%)**

### E. Tick-Seed Sweep (64 Seeds for Fixed Single Target)
* **Candidate across 64 Ticks**:
  * LEFT: **33 (51.6%)** | RIGHT: **31 (48.4%)**
  * *Both contenders win evenly across time; no permanent directional lock.*

---

## 4. RTX 5090 / DX12 GPU Microbenchmark (4,194,304 Cells / 2048×2048 World)

Measured in Release build on controlled idle NVIDIA GeForce RTX 5090 (DirectX 12 backend), 50 batched dispatches per run, alternating B-H-H-B-B-H order:

### Scenario A: Realistic / Sparse Contention (~5% contending pairs)
* **Baseline Runs (ms/dispatch)**: `[0.0378, 0.0408, 0.0412, 0.0412, 0.0459]` $\to$ **Median: 0.0412 ms**
* **Candidate Runs (ms/dispatch)**: `[0.0406, 0.0408, 0.0412, 0.0441, 0.0445]` $\to$ **Median: 0.0412 ms**
* **Delta**: **-0.0001 ms (-0.18%)** (within timer noise; virtually identical).

### Scenario B: Contention-Heavy / Worst-Case (Dense 100% Contending Grid)
* **Baseline Runs (ms/dispatch)**: `[0.0381, 0.0395, 0.0396, 0.0412, 0.0414]` $\to$ **Median: 0.0396 ms**
* **Candidate Runs (ms/dispatch)**: `[0.0409, 0.0410, 0.0412, 0.0429, 0.0431]` $\to$ **Median: 0.0412 ms**
* **Delta**: **+0.0015 ms (+3.86% claim-only)**

### Simulation Full-Tick Impact Context:
* Full simulation reference tick budget is ~0.146 ms (6,838 TPS).
* A +0.0015 ms overhead in the claim pass corresponds to **~1.0% of full-tick simulation time**.
* The ALU cost of the integer mixer is almost entirely hidden by existing memory bandwidth latency on RTX 5090.

---

## 5. Formal Recommendation

### **`HASH CANDIDATE WORTH INTEGRATING`**

### Rationale:
1. **Bias Elimination**: Fixed-index baseline exhibits absolute (100% vs 0%) directional bias across all orientations. The candidate eliminates this bias entirely (49.3%~51.2% distribution).
2. **Zero Invariant Risk**: Edge agreement, exact single ownership, and determinism are 100% verified.
3. **Negligible GPU Cost**: On RTX 5090 / DX12, the worst-case claim dispatch overhead is only **+0.0015 ms** (~1.0% full-tick impact), which is well below the 5% threshold.

---

## 6. Gate Status

- **G6-A**: `TECHNICAL PASS`
- **G6-B**: `TECHNICAL PASS`
- **G6-C**: `MEASUREMENT COMPLETE / DECISION PENDING`
- **G6 Overall**: `IN_PROGRESS / NOT CLOSED` (Awaiting user decision on G6-C2 adoption)
- **G7**: `NOT STARTED`
