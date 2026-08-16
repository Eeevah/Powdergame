# G6-C — Arbitration Quality Decision

Status: **DESIGN READY / MEASUREMENT NOT STARTED**

Prerequisites:

- G6-A GPU Write Ownership Audit — **TECHNICAL PASS**
- G6-B Ownership Contention Integrity — **TECHNICAL PASS**
- Frozen correctness baseline: `ea08f6605703bb42593f7e6a1fb5181c99909ca8`
- G5 frozen base: `2112dfbacdefdcb02f4d82496dee374fc8e97f70`

G6-C does not ask whether the current arbitration is correct. G6-A/B already established correctness. G6-C asks whether the current fixed-index tie-breaker has enough directional/index bias that a cheap stateless alternative is worth its measured RTX 5090 cost.

The gate is deliberately split into measurement and decision/integration so the frozen correctness baseline is not modified before evidence exists.

---

## 1. Verified baseline

All three current contention resolvers use the same basic winner policy:

- `movement_claim.wgsl`: smallest source index wins among incident ownership edges.
- `expansion_claim.wgsl`: smallest source index wins for an EMPTY destination.
- `smoke_claim.wgsl`: smallest source index wins for an EMPTY destination.

Properties already proven in G6-A/B:

- stateless
- deterministic
- no per-cell RNG state
- no atomics
- no global ordering
- exactly-one ownership under contention

Known tradeoff:

```text
source_index = y * width + x
```

therefore geometrically lower-index contenders have a deterministic advantage. This is a quality/fairness question, not a correctness failure.

---

## 2. G6-C split

### G6-C1 — Measurement

Measure the frozen fixed-index baseline against exactly one test-only stateless hash candidate.

**Production claim shaders must not be changed during C1.**

C1 output is evidence only. It must finish with one of:

- `HASH CANDIDATE WORTH INTEGRATING`
- `KEEP BASELINE LIKELY`
- `BORDERLINE — DECISION REQUIRED`

It does not close G6.

### G6-C2 — Decision / Integration

After C1 results are reviewed, record exactly one decision:

- `KEEP FIXED-INDEX BASELINE`, or
- `ADOPT STATELESS EDGE HASH`.

Only if the hash is adopted may production claim shaders change. Any adopted implementation must then rerun G6-A/B regressions, full workspace tests and RTX 5090 smoke/performance validation before G6 can close.

---

## 3. Candidate contract

The comparison candidate must remain:

- local
- stateless
- integer-only
- deterministic for identical inputs
- no per-cell RNG buffer
- no atomics
- no global counter/order/sort
- no extra ownership pass
- total-ordering capable through a deterministic tie-break

Preferred priority key:

```text
priority = hash(source_index, target_index, arbitration_tick)
```

with `source_index` as the final deterministic tie-break if two priorities collide.

Important movement constraint:

Movement ownership is an **edge** protocol. Both endpoints must independently select the same edge. Therefore the candidate priority must be a pure function of the same edge identity on both endpoints:

```text
edge_priority(source, target, tick)
```

Do not use an endpoint-local random value or a destination-only value that could make source and destination choose different edges.

For C1, the tick input may live entirely in a test-only benchmark harness. Do not add a production per-tick uniform or production buffer solely to perform the comparison.

---

## 4. Measurement harness

Create a dedicated test/benchmark harness, preferably:

```text
engine/gpu/tests/arbitration_quality.rs
```

The harness should run custom baseline and candidate claim pipelines using synthetic proposal buffers, without changing `Simulation::tick()` or production WGSL files.

Reuse the real claim semantics as closely as practical:

- movement edge arbitration semantics
- destination-only expansion/smoke arbitration semantics
- same local 8-neighbor contender radius
- same reference indexing convention

Avoid benchmarking a trivial CPU hash and calling that GPU evidence.

---

## 5. Bias experiments

### A. Mirrored pair contention

Construct many independent symmetric two-source → one-target micro-fixtures.

For each geometric pair, label contenders by physical role rather than source index:

- left vs right
- up vs down
- NW vs SE
- NE vs SW

Collect winner counts over many translated placements.

The fixed-index baseline should expose its deterministic index preference. The candidate should substantially reduce persistent physical-side preference.

### B. Rotated contention

Rotate equivalent contention geometry through 90° / 180° / 270° where the protocol allows equivalent proposals.

Report whether the same physical direction is systematically privileged.

### C. Translation distribution

Repeat identical local geometry at many world coordinates.

Baseline and candidate should be reported separately.

A stateless coordinate/hash candidate is allowed to choose different winners at different translations; that is the intended decorrelation property.

### D. Tick-seed distribution

For the hash candidate only, hold geometry fixed and vary the test-only arbitration tick seed.

Verify:

- both symmetric contenders can win across seeds
- identical `(source,target,tick)` input is exactly reproducible
- no persistent state is required

### E. Collision handling

Deliberately or statistically exercise equal-priority collisions if practical.

Final comparison must remain a total order through deterministic source-index fallback.

---

## 6. Minimum sample quality

Use enough independent samples that single-fixture anecdotes cannot decide the gate.

Recommended minimum:

- at least 2,048 independent translated pair contests per orientation
- at least 4 symmetric orientation classes where semantically valid
- at least 64 tick seeds for fixed-geometry candidate checks

Record raw counts as well as percentages.

Do not require mathematically perfect 50/50 output. This is a cheap gameplay arbitration function, not a cryptographic RNG.

A useful candidate should, however, eliminate the baseline's near-absolute physical-side preference across large symmetric samples.

---

## 7. Correctness requirements for the candidate harness

Before considering performance or fairness, candidate measurement must prove:

- exactly one winner per contested target/edge
- no ambiguous double ownership
- movement endpoints agree on the same winning edge
- deterministic repeat for the same input/seed
- no out-of-range source/target encoding

If the candidate fails any of these, reject it immediately regardless of bias score.

---

## 8. GPU cost measurement

Bias improvement alone is insufficient. Compare actual RTX 5090 / DX12 cost.

### Claim-only stress benchmark

Use synthetic proposal buffers on a reference-sized workload where contention is intentionally common, so the hash path actually executes.

Measure baseline vs candidate with identical:

- world/cell count
- proposal pattern
- dispatch dimensions
- warm-up
- number of dispatches
- GPU completion wait policy

Prefer batched repeated dispatches per measured run to reduce CPU submit/timer noise. GPU timestamps may be used if the existing stack supports them cleanly; do not build a large profiling subsystem only for G6-C.

Recommended protocol:

- release build
- controlled idle machine
- warm-up ≥ 100 dispatches
- ≥ 5 measured runs
- many repeated claim dispatches per run
- report each run and median

Measure at least:

1. **Sparse/realistic proposal pattern**
2. **Contention-heavy/worst-case proposal pattern**

Report:

```text
baseline median claim time
candidate median claim time
absolute delta
percentage delta
```

### End-to-end sanity

C1 does not modify production, so production full-tick performance remains the frozen baseline. If C2 adopts the hash, rerun a full end-to-end controlled reference-world measurement after integration.

---

## 9. Decision guidance

C1 must return data, not force a predetermined answer.

Use these as guidance rather than hidden physics constants:

### Strong adopt signal

- correctness PASS
- baseline shows severe symmetric-direction bias
- candidate materially balances translated/mirrored/rotated samples
- worst-case claim-only overhead is modest

### Strong keep signal

- candidate does not materially improve bias, or
- correctness/edge agreement becomes harder or fragile, or
- measured GPU overhead is large enough to conflict with Powdergame's performance-first design.

### Borderline

If fairness improvement is clear but measured cost is non-trivial, report the exact tradeoff and stop for decision. Do not silently choose by preference.

As a rough interpretation aid for the later decision:

- tiny low-single-digit end-to-end-equivalent cost: generally acceptable if bias improvement is strong
- clearly >5% simulation-hot-path cost: strong reason to keep the baseline for M0 unless visual bias is demonstrably harmful

Do not manufacture an end-to-end percentage from claim-only timings without stating how it was derived.

---

## 10. Scope protection

C1 must not change:

- `movement_claim.wgsl`
- `expansion_claim.wgsl`
- `smoke_claim.wgsl`
- any other production WGSL
- `Simulation::tick()`
- G0–G5 gameplay physics
- phase/thermal/pressure/combustion constants
- G5 fixtures
- G7 Active/Sleep

Test-only shader strings/files and benchmark support are allowed.

Do not modify or stage `docs/planning/MATERIAL_CANDIDATES.md`.

---

## 11. C1 evidence output

Create:

```text
docs/evidence/G6_C1_ARBITRATION_MEASUREMENT_2026-08-16.md
```

Required contents:

1. frozen baseline SHA
2. exact test-only candidate algorithm
3. proof of movement edge endpoint agreement
4. mirrored winner counts
5. rotated winner counts
6. translated winner counts
7. tick-seed winner counts
8. deterministic-repeat result
9. baseline claim timings
10. candidate claim timings
11. percentage overhead
12. RTX 5090 / DX12 identity
13. production files changed: `NO`
14. recommendation: `HASH CANDIDATE WORTH INTEGRATING`, `KEEP BASELINE LIKELY`, or `BORDERLINE`

G6-C remains `MEASUREMENT COMPLETE / DECISION PENDING` after C1.

---

## 12. G6 closure rule

G6 may become `PASS / CLOSED` only after:

1. G6-A TECHNICAL PASS
2. G6-B TECHNICAL PASS
3. G6-C1 measurement complete
4. G6-C2 decision recorded
5. if hash adopted: production integration + full regression/performance validation PASS
6. user approval of the final G6 decision/evidence

G7 must not start before G6 is closed.
