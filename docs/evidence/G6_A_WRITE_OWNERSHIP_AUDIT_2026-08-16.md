# G6-A GPU Write Ownership Audit — RTX 5090 / DX12

- **Date**: 2026-08-16
- **Status**: **TECHNICAL PASS / VALIDATION COMPLETED**
- **Base Commit**: `2112dfbacdefdcb02f4d82496dee374fc8e97f70` (`feat: finalize M0 G5 pressure chain`)
- **GPU Target**: NVIDIA GeForce RTX 5090, wgpu backend Dx12, Windows 11 Pro

---

## 1. Production Pass Ownership & Structural Write Matrix

Every production simulation pass dispatched in `Simulation::tick()` was audited directly against its WGSL shader AST and runtime binding configuration.

| Pass | Entry Point | Role | Neighbor Reads | Writable Storage Bindings | Actual Write Index Rule | Ownership-changing? | Claim/Resolve? | Atomic? | Global Ordering? | Persistent RNG? | Verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|
| **movement_propose** | `propose_main` | OWNERSHIP_PROPOSE | 8-neighbors (material_current) | `proposal`, `marker` | `proposal[self]`, `marker[0]` (if self==0) | Yes (initiates) | No (pre-claim) | None | None | None | **PASS** |
| **movement_claim** | `claim_main` | OWNERSHIP_RESOLVE | 8-neighbors (proposal) | `claim` | `claim[self]` | Yes (arbitrates) | Yes (bidirectional) | None | None | None | **PASS** |
| **movement_commit** | `commit_main` | OWNERSHIP_COMMIT | 1 peer (claim, material_current, temperature_current, flags_current) | `material_next`, `temperature_next`, `flags_next` | `material_next[self]`, `temperature_next[self]`, `flags_next[self]` | Yes (executes) | Yes | None | None | None | **PASS** |
| **thermal** | `thermal_main` | SELF_WRITE | 4-neighbors (material_current, temperature_current) | `temperature_next` | `temperature_next[self]` | No | No | None | None | None | **PASS** |
| **phase_transition** | `phase_main` | SELF_WRITE + OWNERSHIP_PROPOSE | Self only for phase; 8-neighbors (material_current) for expansion candidate | `material_next`, `proposal` | `material_next[self]`, `proposal[self]` | Yes (if yield=2) | No (pre-claim) | None | None | None | **PASS** |
| **expansion_claim** | `expansion_claim_main` | OWNERSHIP_RESOLVE | 8-neighbors (proposal) | `claim` | `claim[self]` | Yes (arbitrates) | Yes | None | None | None | **PASS** |
| **expansion_spawn_commit** | `expansion_spawn_commit_main` | OWNERSHIP_COMMIT | 1 source (material_next, temperature_current) | `material_next`, `temperature_next`, `flags_next` | `material_next[self]`, `temperature_next[self]`, `flags_next[self]` | Yes (executes) | Yes | None | None | None | **PASS** |
| **expansion_pressure** | `expansion_pressure_main` | SELF_WRITE | 1 destination (claim) | `pressure_next` | `pressure_next[self]` | No | No | None | None | None | **PASS** |
| **decay** | `decay_main` | SELF_WRITE | Self only | `material_next`, `flags_next`, `temperature_next` | `material_next[self]`, `flags_next[self]`, `temperature_next[self]` | No | No | None | None | None | **PASS** |
| **combustion** | `combustion_main` | SELF_WRITE + OWNERSHIP_PROPOSE | 8-neighbors (material_current) for smoke candidate | `temperature_next`, `flags_next`, `proposal`, `material_next` | `temperature_next[self]`, `flags_next[self]`, `proposal[self]`, `material_next[self]` | Yes (if burning) | No (pre-claim) | None | None | None | **PASS** |
| **smoke_claim** | `smoke_claim_main` | OWNERSHIP_RESOLVE | 8-neighbors (proposal) | `claim` | `claim[self]` | Yes (arbitrates) | Yes | None | None | None | **PASS** |
| **smoke_commit** | `smoke_commit_main` | OWNERSHIP_COMMIT | 1 source (temperature_next) | `temperature_next`, `material_next` | `temperature_next[self]`, `material_next[self]` | Yes (executes) | Yes | None | None | None | **PASS** |
| **pressure** | `pressure_main` | SELF_WRITE | 4-neighbors (material_current, pressure_current) | `pressure_next` | `pressure_next[self]` | No | No | None | None | None | **PASS** |
| **rupture** | `rupture_main` | SELF_WRITE | 4-neighbors (material_current, pressure_current) | `material_next`, `temperature_next`, `flags_next` | `material_next[self]`, `temperature_next[self]`, `flags_next[self]` | No | No | None | None | None | **PASS** |

---

## 2. Invariant Audit Findings

1. **Direct Neighbor Mutation (SELF_WRITE & General Rules)**:
   - **Finding**: **NONE**.
   - In all 14 shaders, every write statement writes strictly to an index expression representing the invocation's own cell index (`self`, `c`, or `index`).
   - Ordinary local rules (`thermal`, `decay`, `pressure`, `rupture`, `expansion_pressure`) and self-write phases (`phase_transition`, `combustion`) never execute `buffer[neighbor_index] = ...`.

2. **Atomics & Spin-Locks**:
   - **Finding**: **NONE**.
   - No `atomic*` operations, atomic types, or spin-lock mechanisms exist in any production WGSL shader.
   - Diagnostic signaling is handled by single invocation index 0 (`marker[0] = 1u`).

3. **Workgroup Shared Memory & Global Synchronization**:
   - **Finding**: **NONE**.
   - No `var<workgroup>` variables exist in any production shader.
   - Dispatch barriers between sequential causal phases (e.g. `propose` $\to$ `claim` $\to$ `commit`) are standard GPU pipeline barriers and ensure clean sequential ordering without fine-grained global synchronization.

4. **Global Ordering / Full-World Sort**:
   - **Finding**: **NONE**.
   - No sort algorithms, global priority queues, or whole-world coordination are required or used for rule priority.

5. **Per-Cell Persistent RNG State**:
   - **Finding**: **NONE**.
   - Arbitration is 100% stateless and deterministic (smallest source index wins). No per-cell RNG seeds or pseudo-random state buffers exist.

6. **First-Match & Locality**:
   - **Finding**: **VERIFIED**.
   - Movement candidate selection uses local 1-cell stencils only (Powder/Liquid/Gas).
   - Expansion candidate selection uses 8-neighbor local First-Match only.
   - Smoke spawn candidate selection uses 8-neighbor local First-Match only.
   - No long-range searches or teleportation scans exist.

---

## 3. Automated Structural Test Evidence

- **Test Suite**: `engine/gpu/tests/parallel_integrity.rs`
- **Test Function**: `test_all_production_wgsl_write_contracts_and_binding_safety`
- **Mechanism**: Naga AST reflection over all 14 WGSL modules.
- **Result**: **PASS** (all 14 production shaders validated against exact expected read_write storage bindings, verifying 0 unauthorized writable buffers and 0 workgroup shared variables).
