# Powdergame M0 G6-C2 — Stateless Edge Hash Production Integration Report

Date: 2026-08-16  
Status: **TECHNICAL PASS / READY FOR USER VALIDATION**  
Environment: NVIDIA GeForce RTX 5090 (32GB VRAM), Windows 11 Pro, workspace `wgpu = "26"` (DX12 backend), Rust 1.84+  

---

## 1. Executive Summary

In accordance with the **G6-C2 Hash Adoption Decision** recorded in `docs/planning/G6_C2_HASH_INTEGRATION.md` and ADR-0004, the production ownership arbitration pipeline was upgraded from fixed linear-index preference to a **stateless, tick-seeded edge-hash function** (`edge_priority`).

Key achievements:
1. **Directional Bias Eliminated by the Measured Candidate**: G6-C1 mirrored horizontal, vertical, and diagonal contention measurements changed from the fixed baseline's 100%/0% preference to approximately 49–51% distributions. After production integration, the 64-seed symmetric contention regression produced Left 33 / Right 31, confirming the fixed-index permanent lock is gone in the production path.
2. **Zero Ownership Invariant Violations Observed**: The production write-ownership contracts remain intact (Read Neighbors / Write Self or bounded Claim/Resolve). The structural audit found no unauthorized writable storage bindings or workgroup-shared state, and contention regressions preserved single-winner ownership and Matter conservation where applicable.
3. **Deterministic Ownership Result for Identical Seed**: `test_production_same_seed_determinism` verifies that two independently constructed simulations with the same initial fixture and arbitration seed produce the same full **Material ownership buffer** after the production tick. This test does not claim bitwise equality of Temperature, Pressure, or Flags buffers.
4. **End-to-End Performance**: Full-tick execution on the 2048×2048 reference world (4,194,304 cells, 14 production simulation dispatches plus buffer copies) remained within measurement noise:
   - **PRE-Integration Median**: `0.8560 ms` (`1,168.2 TPS`)
   - **POST-Integration Median**: `0.8426 ms` (`1,186.8 TPS`)
   - **Observed delta**: `-0.0134 ms` / `-1.57%`; therefore **no measured performance regression**. The negative delta is treated as run-to-run hardware/measurement variation, not as a claimed optimization speedup.

---

## 2. Production Integration Architecture

### 2.1 Dedicated Arbitration Uniform
The shared 16-byte `Params` uniform used across the production passes was preserved without ABI changes. A dedicated, simulation-owned 16-byte uniform buffer (`ArbitrationParams`) was introduced exclusively for the three claim/resolve passes:

```wgsl
struct ArbitrationParams {
    tick: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};
```

### 2.2 Pipeline Bindings
Binding layouts were updated strictly on the 3 claim shaders:
- `movement_claim.wgsl`: `@group(0) @binding(3) var<uniform> arbitration: ArbitrationParams;`
- `expansion_claim.wgsl`: `@group(0) @binding(4) var<uniform> arbitration: ArbitrationParams;`
- `smoke_claim.wgsl`: `@group(0) @binding(4) var<uniform> arbitration: ArbitrationParams;`

No proposer, thermal, phase, combustion, pressure, rupture, decay, or commit shader received this arbitration binding.

### 2.3 CPU Tick Update
At the beginning of each simulation tick, before compute command encoder recording:

```rust
let mut arb_bytes = [0u8; 16];
arb_bytes[..4].copy_from_slice(&(self.tick_count as u32).to_ne_bytes());
self.context.queue.write_buffer(&self.arbitration_params, 0, &arb_bytes);
```

The same low 32 bits of `tick_count` are therefore visible to movement, expansion, and smoke arbitration during that simulation tick. `u32` wrap is acceptable because this value is a stateless tie-break seed, not authoritative elapsed time.

### 2.4 Hash Function & Tie-Breaking
All 3 resolvers execute the identical stateless integer mixer:

```wgsl
fn edge_priority(source: u32, target_cell: u32, tick: u32) -> u32 {
    var h: u32 = source ^ (target_cell * 0x9E3779B9u) ^ (tick * 0x85EBCA6Bu);
    h = (h ^ (h >> 16u)) * 0x7FEB352Du;
    h = (h ^ (h >> 15u)) * 0x846CA68Bu;
    h = h ^ (h >> 16u);
    return h;
}
```

Winner condition: `p < best_priority || (p == best_priority && source < best_owner)`.

For movement, both endpoints evaluate the same **ordered directed edge identity** `(source, target, tick)`, so the priority is endpoint-consistent. The function is not required to be invariant under swapping `source` and `target`.

---

## 3. Correctness & Invariant Validation

| Subsystem / Contract | Verification Method | Result |
| :--- | :--- | :--- |
| **Write Ownership Safety** | Naga structural storage-binding audit across all 14 production WGSL modules plus manual/behavioral G6 evidence | **PASS** — no unauthorized writable storage binding; no workgroup-shared variable detected |
| **Movement Edge Consistency** | G6-C1 candidate endpoint-agreement test plus production many-to-one/chain/chunk-boundary regressions after integration | **PASS** — no split ownership observed |
| **Many-to-One Contention** | Simultaneous local proposals to one target | **PASS** — exactly 1 winner, non-winners preserved at source |
| **Density Movement & Swaps** | Existing density contention, chain, STATIC, equal-rank, and chunk-boundary regressions | **PASS** — single ownership edge and conservation preserved |
| **Expansion Contention** | Multiple boiling sources targeting one EMPTY cell | **PASS** — exactly 1 extra-Matter spawn winner; non-winning expansion requests follow frozen blocked-pressure semantics |
| **Smoke Contention** | Competing combustion sources proposing Smoke spawn | **PASS** — exactly 1 Smoke destination winner; source state remains valid |
| **Scratch Buffer Reuse** | Sequential `proposal`/`claim` reuse across movement → expansion → combustion/smoke | **PASS** — no stale winner state observed in dedicated regressions |
| **Chunk Boundaries** | Movement/density and other existing cross-chunk local-interaction regressions | **PASS** — ownership rules do not introduce a special chunk wall |
| **Same-Seed Production Ownership Determinism** | `test_production_same_seed_determinism` | **PASS** — identical full Material buffers for the tested fixture/seed |
| **Tick Seed Variation** | 64 seeds on symmetric production horizontal contention | **PASS** — Left 33 / Right 31; both contenders win across seeds |

The project remains intentionally **approximately deterministic**, not globally bit-exact deterministic. G6-C2 only requires that the adopted stateless arbitration produce reproducible ownership decisions for identical edge/tick inputs while preserving hard invariants.

---

## 4. Performance Benchmark (RTX 5090 / DX12)

**Test Protocol**: `cargo test --release -p powdergame-gpu --test movement controlled_reference_world_perf -- --ignored --nocapture --test-threads=1`  
- World Size: 2048×2048 (4,194,304 cells)
- Warm-up: 100 ticks (excluded)
- Measurement: 5 runs × 1,000 full-tick iterations with GPU completion synchronization (`wgpu::PollType::Wait`)

### Full-Tick End-to-End Comparison

| Run # | PRE-Integration Baseline (`be9645d`) | POST-Integration Production (`74a40ef`) | Delta |
| :--- | :--- | :--- | :--- |
| Run 1 | 0.8528 ms (1,172.7 TPS) | 0.8496 ms (1,177.1 TPS) | -0.0032 ms |
| Run 2 | 0.8617 ms (1,160.5 TPS) | 0.8426 ms (1,186.8 TPS) | -0.0191 ms |
| Run 3 | 0.8560 ms (1,168.2 TPS) | 0.8375 ms (1,194.0 TPS) | -0.0185 ms |
| Run 4 | 0.8767 ms (1,140.7 TPS) | 0.8443 ms (1,184.4 TPS) | -0.0324 ms |
| Run 5 | 0.8384 ms (1,192.8 TPS) | 0.8221 ms (1,216.4 TPS) | -0.0163 ms |
| **MEDIAN** | **0.8560 ms (1,168.2 TPS)** | **0.8426 ms (1,186.8 TPS)** | **-0.0134 ms (-1.57%)** |

Interpretation: the integrated production hash shows **no measurable end-to-end regression** in this reference-world protocol. The negative delta is not treated as a guaranteed speed improvement.

For context, G6-C1's contention-heavy claim-only microbenchmark measured approximately `+0.0015 ms` (`+3.86%`) for the hash resolver versus fixed-index arbitration in that isolated kernel. The C2 full-tick measurement above is the source of truth for the integrated reference-world path.

---

## 5. Test & Smoke Run Summary

Validated at production integration SHA `74a40ef37cf708ba4430e7f7ca7dd10842db7408`:

- **Workspace**: `297 passed, 0 failed, 1 ignored` (`--test-threads=1`)
  - `powdergame-core`: **130 passed**
  - `powdergame-gpu`: **163 passed**, **1 ignored** controlled benchmark
  - `powdergame-windows`: **4 passed**
- **GPU Parallel Integrity**: `parallel_integrity.rs` — **12 passed**
- **GPU Arbitration Quality**: `arbitration_quality.rs` — **6 passed**
- **Code Quality**:
  - `cargo fmt --all -- --check`: **PASS**
  - `cargo clippy --workspace --all-targets -- -D warnings`: **PASS** (0 warnings)
  - `git diff --check`: **PASS**
- **Runtime Smoke Runs**:
  1. Default 2048×2048 reference world (`--smoke-frames 60`): **PASS**
  2. Movement demo (`--movement-demo --smoke-frames 120`): **PASS**
  3. Density demo (`--density-demo --smoke-frames 180`): **PASS**
  4. Thermal observatory (`--thermal-demo --smoke-frames 360`): **PASS**
  5. Pressure multi-boiler lab (`--pressure-demo --smoke-frames 500`): **PASS**

The STATUS test breakdown independently sums to the same workspace result:

```text
Core 130 + GPU 163 + Windows 4 = 297 passed
```

---

## 6. Scope / Freeze Audit

Production changes for G6-C2 are limited to:

- `engine/gpu/src/movement_claim.wgsl`
- `engine/gpu/src/expansion_claim.wgsl`
- `engine/gpu/src/smoke_claim.wgsl`
- `engine/gpu/src/simulation.rs`
- G6 tests/docs

No density ranks, thermal constants, phase thresholds/yields, combustion constants, pressure propagation, rupture thresholds, or G5 boiler-fixture physics were changed by the C2 integration.

`docs/planning/MATERIAL_CANDIDATES.md` remained untouched.

---

## 7. Conclusion

G6-C2 stateless edge-hash production integration is complete and technically validated on RTX 5090 / DX12. G6-A, G6-B, G6-C1, and G6-C2 all have sufficient technical evidence for **TECHNICAL PASS**.

G6 overall remains **VALIDATION READY / AWAITING USER APPROVAL** until the user explicitly approves closure. G7 must not begin before that approval is recorded.
