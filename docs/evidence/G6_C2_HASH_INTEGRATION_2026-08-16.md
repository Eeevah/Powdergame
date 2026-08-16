# Powdergame M0 G6-C2 — Stateless Edge Hash Production Integration Report

Date: 2026-08-16  
Status: **TECHNICAL PASS / READY FOR USER VALIDATION**  
Environment: NVIDIA GeForce RTX 5090 (32GB VRAM), Windows 11 Pro, wgpu 24 (DX12 backend), Rust 1.84+  

---

## 1. Executive Summary

In accordance with the **G6-C2 Hash Adoption Decision** recorded in `docs/planning/G6_C2_HASH_INTEGRATION.md` and ADR-0004, the production ownership arbitration pipeline was upgraded from fixed linear-index preference to a **stateless, tick-seeded edge-hash function** (`edge_priority`).

Key achievements:
1. **Directional Bias Eliminated**: Mirrored horizontal, vertical, diagonal, and rotated contentions no longer lock into 100%/0% artifacts; winners distribute symmetrically (~49–51%) across ticks while maintaining exact reciprocal agreement between endpoints.
2. **Zero Invariant Violations**: 100% adherence to write ownership contracts (Read Neighbors / Write Self, 0 atomics, 0 workgroup barriers, 0 writable storage aliases). Conservation of Matter, energy bounds, and single-winner guarantees verified across 100+ tests.
3. **Bit-Exact Determinism**: Identical world state and tick seed produce identical bitwise GPU simulation outcomes.
4. **End-to-End Performance**: Full-tick execution on the 2048×2048 reference world (4,194,304 cells, 14 dispatch passes + memory transfers) remained identical within hardware noise:
   - **PRE-Integration Median**: `0.8560 ms` (`1,168.2 TPS`)
   - **POST-Integration Median**: `0.8426 ms` (`1,186.8 TPS`)
   - **Overhead**: `0.0%` regression (delta `-0.0134 ms` / `-1.57%`).

---

## 2. Production Integration Architecture

### 2.1 Dedicated Arbitration Uniform
The shared 16-byte `Params` uniform used across all 14 passes was preserved without ABI changes. A dedicated, simulation-owned 16-byte uniform buffer (`ArbitrationParams`) was introduced exclusively for the three claim/resolve passes:

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

No proposer, thermal, phase, combustion, pressure, rupture, or commit shaders received this binding.

### 2.3 CPU Tick Update
At the beginning of each simulation tick, before compute command encoder recording:
```rust
let mut arb_bytes = [0u8; 16];
arb_bytes[..4].copy_from_slice(&(self.tick_count as u32).to_ne_bytes());
self.context.queue.write_buffer(&self.arbitration_params, 0, &arb_bytes);
```

### 2.4 Hash Function & Tie-Breaking
All 3 resolvers execute the identical, stateless permutation primitive:
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

---

## 3. Correctness & Invariant Validation

| Subsystem / Contract | Verification Method | Result |
| :--- | :--- | :--- |
| **Write Ownership Safety** | Naga AST static binding audit of all 14 WGSL modules | **PASS** (0 neighbor writes, 0 atomics, 0 shared memory) |
| **Movement Reciprocity** | Both endpoints compute `edge_priority(s, c, tick)` | **PASS** (100% edge agreement across 4.19M cells) |
| **Many-to-One Contention** | 8-neighbor simultaneous proposals on single target | **PASS** (Exactly 1 winner, non-winners preserved at source) |
| **Density Movement & Swaps** | Sand/Water, Water/Oil, Steam/Smoke columns & chains | **PASS** (Single swap edge executed, conservation holds) |
| **Expansion Contention** | Competing boiling sources targeting single EMPTY cell | **PASS** (1 spawn winner, 1 loser converted to 100.0 pressure) |
| **Smoke Contention** | Competing combustion sources proposing smoke spawn | **PASS** (1 smoke spawned, sources remain intact) |
| **Scratch Buffer Reuse** | Sequential `proposal`/`claim` reuse across 3 subsystems | **PASS** (No cross-phase pollution or stale bits) |
| **Chunk Boundaries** | 64-cell chunk crossing for movement, expansion, smoke | **PASS** (No boundary walls, identical physics across chunks) |
| **Same-Seed Determinism** | 2 independent runs with identical initial state & tick | **PASS** (Bitwise identical state across entire world) |
| **Tick Seed Variation** | 64 ticks evaluated on symmetric horizontal contention | **PASS** (Both Left and Right win across seeds: 33 vs 31) |

---

## 4. Performance Benchmark (RTX 5090 / DX12)

**Test Protocol**: `cargo test --release -p powdergame-gpu --test movement controlled_reference_world_perf -- --ignored --nocapture --test-threads=1`  
- World Size: 2048×2048 (4,194,304 cells)
- Warm-up: 100 ticks (excluded)
- Measurement: 5 runs × 1,000 full-tick dispatches with GPU completion synchronization (`wgpu::PollType::Wait`)

### Full-Tick End-to-End Comparison

| Run # | PRE-Integration Baseline (`be9645d`) | POST-Integration Production (`HEAD`) | Delta |
| :--- | :--- | :--- | :--- |
| Run 1 | 0.8528 ms (1,172.7 TPS) | 0.8496 ms (1,177.1 TPS) | -0.0032 ms |
| Run 2 | 0.8617 ms (1,160.5 TPS) | 0.8426 ms (1,186.8 TPS) | -0.0191 ms |
| Run 3 | 0.8560 ms (1,168.2 TPS) | 0.8375 ms (1,194.0 TPS) | -0.0185 ms |
| Run 4 | 0.8767 ms (1,140.7 TPS) | 0.8443 ms (1,184.4 TPS) | -0.0324 ms |
| Run 5 | 0.8384 ms (1,192.8 TPS) | 0.8221 ms (1,216.4 TPS) | -0.0163 ms |
| **MEDIAN** | **0.8560 ms (1,168.2 TPS)** | **0.8426 ms (1,186.8 TPS)** | **-0.0134 ms (-1.57%)** |

---

## 5. Test & Smoke Run Summary

- **Automated Workspace Tests**: All 130 core tests + 144 GPU integration tests passed (`--test-threads=1`, 0 failures).
- **Code Quality**: `cargo fmt --all -- --check` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean (0 warnings).
- **Runtime Smoke Runs**:
  1. Default 2048×2048 reference world (`--smoke-frames 60`): **PASS** (clean exit)
  2. Movement demo (`--movement-demo --smoke-frames 120`): **PASS** (clean exit)
  3. Density demo (`--density-demo --smoke-frames 180`): **PASS** (clean exit)
  4. Thermal observatory (`--thermal-demo --smoke-frames 360`): **PASS** (clean exit)
  5. Pressure multi-boiler lab (`--pressure-demo --smoke-frames 500`): **PASS** (clean exit)

---

## 6. Conclusion

G6-C2 stateless edge hash production integration is complete, fully verified, and hardened. All parallel integrity criteria (G6-A, G6-B, G6-C1, G6-C2) have achieved **TECHNICAL PASS**. G6 is ready for user validation.
