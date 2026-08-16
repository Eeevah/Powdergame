# G6 — Parallel Integrity

Status: **IN PROGRESS**

Base / G5 frozen point:

- `2112dfbacdefdcb02f4d82496dee374fc8e97f70`
- `feat: finalize M0 G5 pressure chain`

G0–G5 gameplay semantics are frozen for this gate. G6 proves that the existing production GPU rule pipeline preserves cell ownership and world invariants under parallel execution; it is not a gameplay-physics tuning milestone.

## Official claim

General interactions remain GPU-parallel-friendly:

```text
Read Neighbors → Write Self
```

Cross-cell ownership changes are exceptional operations and use a bounded local ownership protocol:

```text
Propose → Resolve/Claim → Commit
```

G6 must not introduce global rule ordering, persistent per-cell RNG state, general-rule atomics, or direct neighbor mutation as the normal authoring path.

---

## Gate split

### G6-A — GPU Write Ownership Audit

Prove the write contract of every production simulation pass.

Required evidence:

1. Build a complete production-pass inventory from `Simulation::tick()`.
2. Classify each pass as one or more of:
   - `SELF_WRITE`
   - `OWNERSHIP_PROPOSE`
   - `OWNERSHIP_RESOLVE`
   - `OWNERSHIP_COMMIT`
3. Verify writable storage bindings and the index written by each invocation.
4. Verify ordinary local rules do not mutate neighbor cells directly.
5. Audit production WGSL for:
   - atomics
   - workgroup/global coordination used for rule ordering
   - spin locks
   - full-world sort/priority queues
   - persistent per-cell RNG state
6. Confirm local candidate selection remains ordered first-match rather than a long-range/global scan.
7. Audit proposal/claim scratch reuse boundaries.

Expected baseline to verify against source, not to assume blindly:

| Pass | Expected role | Expected write ownership |
|---|---|---|
| movement_propose | OWNERSHIP_PROPOSE | `proposal[self]` |
| movement_claim | OWNERSHIP_RESOLVE | `claim[self]` |
| movement_commit | OWNERSHIP_COMMIT | Matter-owned `Next[self]`, reading peer ownership state |
| thermal | SELF_WRITE | `temperature_next[self]` |
| phase_transition | SELF_WRITE + OWNERSHIP_PROPOSE | `material_next[self]`, `proposal[self]` |
| expansion_claim | OWNERSHIP_RESOLVE | `claim[self]` |
| expansion_spawn_commit | OWNERSHIP_COMMIT | winning destination writes its own `Next[self]` state |
| expansion_pressure | SELF_WRITE | `pressure_next[self]` |
| decay | SELF_WRITE | Matter/flags/temperature `Next[self]` |
| combustion | SELF_WRITE + OWNERSHIP_PROPOSE | self state + `proposal[self]` |
| smoke_claim | OWNERSHIP_RESOLVE | `claim[self]` |
| smoke_commit | OWNERSHIP_COMMIT | winning destination writes its own state |
| pressure | SELF_WRITE | `pressure_next[self]` |
| rupture | SELF_WRITE | structure reads neighbor Pressure, writes itself EMPTY |

If source inspection contradicts this table, the source wins and the discrepancy must be documented.

### G6-A validation strategy

Prefer structural evidence over fragile text matching.

Order of preference:

1. Reuse the repository's existing WGSL/naga parsing capability if it can cheaply expose storage access information.
2. Add a small explicit pass-contract manifest plus WGSL parse validation and behavioral tests.
3. Use targeted source assertions only as supplemental diagnostics.

Do **not** build a large general-purpose shader static analyzer just for G6.

A `SELF_WRITE` pass fails G6-A if an invocation can directly write `buffer[neighbor_index]` as part of its ordinary rule path.

GPU dispatch barriers / sequential causal phases are allowed. Multiple dispatches are not themselves a G6 failure.

---

### G6-B — Ownership Contention Integrity

Prove that the existing ownership mechanisms preserve world invariants under contention and scratch-buffer reuse.

#### Movement contention

Required behavioral evidence:

- many sources → one EMPTY destination: exactly one winner
- loser sources remain valid
- Matter count conserved in a closed fixture
- source/destination chains: a cell joins at most one ownership edge
- chunk-boundary contention behaves like interior contention
- repeated dense contention over hundreds/thousands of ticks does not corrupt ownership

#### Density-swap contention

Reuse existing G3 evidence when it proves the same contract; add tests only for missing G6 cases.

Required properties:

- normal movement and density swap cannot both own the same cell in one commit
- exactly one winning edge
- loser remains valid
- no three-way corruption
- equal-rank produces no unnecessary swap ownership
- STATIC is excluded

#### Expansion contention

For multiple boiling sources competing for one EMPTY destination:

- destination receives exactly one spawned Matter
- exactly one winner
- source phase transitions remain valid
- losing expansion follows the frozen confinement-pressure semantics
- no duplicate Matter
- chunk-boundary case included

#### Smoke-spawn contention

For multiple burning sources competing for one EMPTY destination:

- exactly one Smoke destination winner
- source Wood/Oil remains valid
- destination flags are valid
- Smoke decay age begins at zero
- unrelated Matter untouched
- stale movement/expansion proposal or claim data cannot win the smoke subsystem

#### Scratch reuse contract

The tick currently reuses `proposal` / `claim` scratch sequentially across ownership subsystems.

Audit these boundaries explicitly:

```text
movement
→ phase expansion
→ combustion smoke spawn
```

For every boundary, prove that all state relevant to winner selection is overwritten or made ineligible before the next subsystem consumes the scratch buffers.

If stale data can affect a winner, apply the smallest correctness fix and add a regression test. Do not redesign arbitration in G6-B.

#### Heavy mixed integrity stress

Add one small headless mixed-world GPU fixture containing several of:

- Sand
- Water
- Oil
- Steam
- Smoke
- hot/cold Matter
- burning Wood/Oil
- a phase-transition region
- a pressure/confinement region

After hundreds of ticks verify:

- all material IDs valid
- EMPTY hygiene preserved
- finite Temperature
- finite Pressure
- no ownership corruption / unexplained Matter loss in closed sub-fixtures
- no GPU/device failure

This is an integrity stress, not a G8 performance benchmark.

---

### G6-C — Arbitration Quality Decision

**Not part of the first G6-A/B implementation round.**

Current arbitration is expected to be a stateless, cheap deterministic baseline, generally favoring a small source index. That is acceptable for G6-A/B if correctness holds.

G6-C asks a separate question:

> Is the fixed-index baseline visibly or statistically biased enough that a different stateless arbitration candidate is worth its GPU cost?

Before changing arbitration, measure:

- mirrored contention
- translated contention
- rotated contention
- winner distribution
- GPU cost

Then compare the frozen baseline with at most one cheap stateless candidate, such as a coordinate/tick hash, without adding persistent per-cell RNG state.

G6-C may conclude **KEEP BASELINE**. A new hash scheme is not a mandatory deliverable.

G6 cannot become PASS/CLOSED until the G6-C decision is recorded and required evidence is complete.

---

## Non-goals / frozen scope

Do not change during G6-A/B unless a direct correctness bug requires the smallest possible fix:

- movement feel / GAS behavior
- density ranks
- phase thresholds or yield
- thermal constants
- combustion constants
- Pressure propagation/tuning
- rupture thresholds
- rendering effects
- Active Chunk / Sleep / Wake (G7)
- performance optimization work (G8)

Do not modify or stage `docs/planning/MATERIAL_CANDIDATES.md`.

---

## Validation baseline

Host-independent checks:

```text
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace -- --test-threads=1
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

RTX 5090 / DX12 validation:

- new `parallel_integrity` GPU tests
- movement
- density
- expansion
- combustion
- phase
- pressure
- rupture
- thermal
- world_integrity

Runtime smoke:

```text
--smoke-frames 60
--movement-demo --smoke-frames 120
--density-demo --smoke-frames 180
--thermal-demo --smoke-frames 360
--pressure-demo --smoke-frames 500
```

Record exact per-test-binary summaries instead of inventing a synthetic workspace total if Cargo does not print one.

---

## Evidence documents

G6-A/B should produce:

- `docs/evidence/G6_A_WRITE_OWNERSHIP_AUDIT_2026-08-16.md`
- G6-B behavioral evidence in the same document or a companion `G6_B_CONTENTION_INTEGRITY_2026-08-16.md`

The G6-A pass matrix should include:

```text
Pass
Role
Neighbor reads
Writable outputs
Index/ownership rule
Ownership-changing?
Claim/Resolve?
Atomic?
Global ordering?
Verdict
```

---

## Status policy

During the first round:

```text
G6 = IN_PROGRESS
G6-A = VALIDATION CANDIDATE / TECHNICAL PASS CANDIDATE
G6-B = IN_PROGRESS or VALIDATION CANDIDATE
G6-C = NOT STARTED
```

Do not mark G6 `PASS / CLOSED` during G6-A/B.

A checkpoint commit/push after clean G6-A/B validation is allowed. No force-push and no merge to `main` as part of this round.
