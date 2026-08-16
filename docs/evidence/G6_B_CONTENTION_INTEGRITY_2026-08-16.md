# G6-B Ownership Contention Integrity — RTX 5090 / DX12

- **Date**: 2026-08-16
- **Status**: **TECHNICAL PASS / VALIDATION COMPLETED**
- **Base Commit**: `2112dfbacdefdcb02f4d82496dee374fc8e97f70` (`feat: finalize M0 G5 pressure chain`)
- **GPU Target**: NVIDIA GeForce RTX 5090, wgpu backend Dx12, Windows 11 Pro

---

## 1. Contention Subsystem Verification

### A. Movement Contention
* **Many Sources $\to$ One EMPTY Destination**:
  * Test: `test_movement_many_sources_one_empty_target_exactly_one_winner`
  * Setup: 3 Sand sources contending for a single EMPTY target in an enclosed funnel.
  * Result: **PASS**. Exactly one Sand landed on the destination; the remaining 2 sources retained their Sand; total Matter count was strictly conserved (3 Sand).
* **Source/Destination Chain (Non-duplication & Non-loss)**:
  * Test: `test_movement_chain_cell_joins_at_most_one_edge`
  * Setup: Vertical column $A(\text{Sand}) \to B(\text{Sand}) \to C(\text{EMPTY})$.
  * Result: **PASS**. Cell $B$ joined at most one ownership edge in tick 1 ($B \to C$). Cell $A$ remained at its location because $B$ was occupied during propose. No duplication or phantom gap occurred.
* **Chunk Boundary Contention**:
  * Test: `test_movement_contention_across_chunk_boundary_single_winner`
  * Setup: Target cell at $(63, 20)$ (chunk boundary $x=63 \leftrightarrow 64$), Source A at $(63, 19)$ [chunk 0], Source B at $(64, 19)$ [chunk 1].
  * Result: **PASS**. Exactly one winner at $(63, 20)$; chunk boundary contention behaves identically to interior contention.
* **Repeated Dense Contention Long-Run**:
  * Test: `test_movement_repeated_contention_long_run_preserves_world_integrity`
  * Setup: 30 Sand + 30 Water particles in a closed hopper ticked for 200 consecutive frames under heavy parallel contention.
  * Result: **PASS**. Exact count conservation (30 Sand, 30 Water), 0 loss, 0 corruption.

### B. Density Contention Evidence (Reused & Verified)
* `density_contention_exactly_one_winner` (`density.rs`): **PASS**
* `density_swap_crosses_chunk_boundary` (`density.rs`): **PASS**
* `equal_rank_water_does_not_jitter` (`density.rs`): **PASS**
* `overlapping_swap_chain_corrupts_nothing` (`density.rs`): **PASS**
* `static_targets_never_swap` (`density.rs`): **PASS**
* Invariant: Normal moves and density swaps never collide because both use the unified bidirectional edge claim protocol (`claim = (peer << 2) | kind`).

### C. Expansion Contention
* **Multiple Boiling Sources $\to$ One EMPTY Destination**:
  * Test: `test_expansion_contention_many_boiling_sources_one_empty_target`
  * Setup: 3 Water sources ($T=75.0$, above boiling point) competing for a single enclosed EMPTY destination.
  * Result: **PASS**. All 3 sources underwent 1:1 phase transition to Steam. Exactly one winner spawned an extra Steam at the destination (total Steam = 4). Losing sources generated confinement pressure ($P \ge 100.0$).
* **Scratch Reuse (`movement` $\to$ `expansion`)**:
  * Test: `test_expansion_scratch_reuse_after_movement`
  * Setup: Active Sand movement in Region A dirties `proposal` and `claim` scratch buffers. In the same tick, boiling Water expansion executes in Region B.
  * Result: **PASS**. Phase transition and expansion claim/commit operate purely on freshly initialized expansion proposals (`proposal[index] = NO_PROPOSAL` at pass start); 0 stale movement data leaked.

### D. Smoke Spawn Contention
* **Multiple Burning Sources $\to$ One EMPTY Destination**:
  * Test: `test_smoke_spawn_contention_multiple_burning_sources_one_empty_target`
  * Setup: 3 burning Wood cells ($T=150.0$, `FLAG_COMBUSTING`) competing for a single EMPTY target.
  * Result: **PASS**. Destination received exactly one Smoke cell; newly spawned Smoke decay age initialized strictly to 0; source Wood cells remained intact and combusting.
* **Scratch Reuse (`movement` $\to$ `expansion` $\to$ `smoke`)**:
  * Test: `test_smoke_scratch_reuse_after_movement_and_expansion`
  * Setup: Simultaneous movement, phase expansion, and combustion in separate regions in the same tick.
  * Result: **PASS**. Smoke destination winner was determined strictly by combustion smoke proposals; 0 interference from previous movement or expansion claims.

### E. Heavy Mixed Integrity Stress
* **Test**: `test_mixed_integrity_stress_long_run`
* **Configuration**: $64 \times 64$ world with 5 distinct simultaneous physical zones (Sand/Water hopper, Oil/Water density column, burning Wood generating Smoke, boiling Water boiler with Wood relief plug, melting Ice) ticked for 300 frames.
* **Audited Invariants**:
  1. All cell material IDs valid ($< 16$).
  2. EMPTY hygiene strictly preserved ($T=0.0$, $\text{flags}=0$, $\text{pressure}=0.0$).
  3. Temperature values finite, non-NaN, non-Inf, bounded in $[-100.0, 2000.0]$.
  4. Pressure values finite, non-NaN, non-Inf, non-negative in $[0.0, 1.0\times 10^6]$.
  5. 0 GPU device lost, clean execution.
* **Result**: **PASS**.

---

## 2. Scratch Buffer Reuse Lifecycle Summary

| Lifecycle Phase | `world.proposal` State | `world.claim` State | Safety Mechanism |
|---|---|---|---|
| **Movement** | Overwritten for all cells by `movement_propose` | Overwritten for all cells by `movement_claim` | Consumed completely by `movement_commit` |
| **Phase / Expansion** | Overwritten for all cells by `phase_transition` (`proposal[index] = NO_PROPOSAL`) | Overwritten for all cells by `expansion_claim` (`claim[c] = NO_CLAIM`) | Full unconditional write per invocation |
| **Combustion / Smoke** | Overwritten for all cells by `combustion` (all branches write `proposal[index]`) | Overwritten for all cells by `smoke_claim` (all branches write `claim[c]`) | Full unconditional write per invocation |

**Conclusion**: Scratch reuse between subsystems is 100% safe by construction. No stale data can survive across pass boundaries.

---

## 3. Current Arbitration Baseline & G6-C Preparation

* **Current Winner Selection Rule**: Smallest source index ($s < \text{best}$).
* **Properties**:
  * 100% stateless (no per-cell RNG seeds or memory overhead).
  * Extremely cheap (single scalar comparison in inner loop).
  * Deterministic and reproducible across executions.
* **Expected Directional Bias**: Source cells with lower linear buffer index ($y \cdot \text{width} + x$) win over higher-index sources.
* **G6-C Forward Reference**:
  * In G6-C, mirrored/translated/rotated contention will be measured against this baseline to evaluate whether a stateless hash-based candidate provides enough visual symmetry improvement to justify any additional GPU overhead.
  * For G6-A/B, the current smallest-index baseline is verified as the frozen correctness baseline.
