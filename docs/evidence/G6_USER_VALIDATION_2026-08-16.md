# G6 — Parallel Integrity: User Validation (2026-08-16)

Status: **APPROVED** — the user directly observed the G6 Parallel Integrity Lab
(`--parallel-integrity-demo`, 256×256 world, RTX 5090 / DX12) and approved the
final result, including long-run observation via the fast-forward control.

G6 = **PASS / CLOSED**.

This document records the final interactive evidence only. It is **not** a
performance benchmark: the ~960 TPS figure below is the G6 demo fast-forward
observation rate, not a G8 official performance baseline.

---

## 1. Observation session

- World: 256×256 (4×4 chunks of 64), stone dividers x 127..128 / y 127..128.
- Observed ticks: **0**, **~161**, **~501**, **~1016**, **36724 (FAST x16)**.
- Final screen state at tick ≈ 36724:
  - SIM TICK ≈ 36724
  - DIAGNOSTIC SAMPLE ≈ 36708
  - FAST x16
  - observed sim rate ≈ 960 TPS (demo fast-forward observation — see note above)

---

## 2. Panel A — Movement Contention (closed fixture)

Final observed counters (held from early ticks through ~36724):

| Metric | Value |
|---|---|
| Matter Count (live) | 562 |
| Initial Matter | 562 |
| Count Delta | +0 |
| Winner exactly one/destination | PASS |
| Losers Valid | YES (DELTA 0) |
| Invalid Material IDs | 0 |
| State | INTEGRITY OK |

Evidence claims:
- Closed-fixture Matter conservation maintained.
- Exactly-one winner per contended destination.
- Loser Matter never lost; no duplicate/unexplained disappearance observed.
- No invalid Material IDs.

---

## 3. Panel B — Chunk Boundary (closed fixture)

Final observed counters:

| Metric | Value |
|---|---|
| Boundary Matter (live) | 1712 |
| Initial Matter | 1712 |
| Count Delta | +0 |
| Invalid Material IDs | 0 |
| State | INTEGRITY OK |

User direct observation: *"경계선에서도 없어지지 않아"* (matter does not
disappear at the seam). Crossings Observed is a **live per-sample diagnostic**
(representative readings ~36 / ~30 / ~28), not a cumulative monotonic counter —
it demonstrates actual seam activity at the 64×64 chunk boundary
(x 191/192, y 63/64).

Core evidence: actual seam activity exists, closed-fixture Matter Δ = 0,
invalid IDs = 0, long-run boundary corruption visually absent.

---

## 4. Panel C — Expansion + Smoke Ownership (one-tick instrument)

The previous C-panel observability problem was fixed by the final hardening:
a blocking one-shot snapshot taken at exactly tick 1 latches the first
post-tick state (async readback latency cannot smear it).

Latched real GPU evidence (readback of `material_current`,
`temperature_current`, `flags_current`, `pressure_current` — **not** dummy
expected constants):

| Expansion | Value | Smoke | Value |
|---|---|---|---|
| Candidates | 3 | Candidates | 3 |
| Winners | 1 | Winners | 1 |
| Steam Sources | 3/3 | Wood Preserved | 3/3 |
| Pressure Losers | 2 | New Smoke Age | 0 |
| Target | STEAM | Target | SMOKE |
| Movement Ran (1 cell) | YES | | |
| Scratch Reuse | PASS | | |
| Result | PASS | | |

Proven semantics:
- 3 boiling Water sources → one shared EMPTY destination: exactly one
  expansion spawn winner; losing sources keep a valid phase result and
  receive the Material-owned confinement pressure impulse.
- 3 burning Wood sources → one shared EMPTY Smoke target: exactly one Smoke
  winner; all 3 Wood sources preserved; new Smoke decay age starts at 0.
- movement → expansion → smoke scratch reuse: no stale proposal/claim
  pollution in the same tick.

---

## 5. Panel D — Heavy Mixed Long-Run Stress (integrity violations)

Final counters at tick ≈ 36724 (FAST x16):

| Metric | Value |
|---|---|
| Invalid Material IDs | 0 |
| NaN/Inf Temperature | 0 |
| NaN/Inf Pressure | 0 |
| Negative Pressure | 0 |
| EMPTY Temp Violations | 0 |
| EMPTY Flag Violations | 0 |
| EMPTY Pressure Violations | 0 |
| State | ALL INTEGRITY OK |

Matter live changed over the run (e.g., 6096 → 5696). This is **not** a
failure and is **not** reported as lost matter: Panel D intentionally contains
phase expansion, combustion → EMPTY fuel consumption, Smoke spawn, and Smoke
decay — i.e., intended Matter creation/destruction. D is therefore **not** a
raw Matter-count conservation fixture.

Responsibility split:
- **A / B** — closed fixtures: conservation + exactly-one-winner + no
  duplicate/loss evidence.
- **D** — heterogeneous long-run state-integrity evidence (hygiene violations
  all zero at ~36724 ticks).

---

## 6. Fast-forward user validation

F cycles x1 → x4 → x16 → x1. Confirmed by direct user observation; at x16 the
demo runs smoothly on the RTX 5090 and sustained long-run stress observation
to tick 36724.

Contract (unchanged):
- Physics timestep unchanged.
- `Simulation::tick()` run sequentially `fast` times per beat — identical
  ticks, just more per update opportunity.
- N = exactly one tick (multiplier-independent).
- R = world + metrics reset, fast back to 1x.
- Diagnostic readback cadence relaxes in fast mode (5 → 12 → 30 ticks).

FAST mode does not change simulation semantics.

---

## 7. Audit preservation (G6-A / G6-B / G6-C)

### G6-A — GPU Write Ownership Audit: TECHNICAL PASS / FROZEN
Production passes classified as SELF_WRITE / OWNERSHIP_PROPOSE /
OWNERSHIP_RESOLVE / OWNERSHIP_COMMIT. General rule: **Read Neighbors → Write
Self**; ownership-changing operations use **Propose → Claim/Resolve →
Commit**. Thermal, pressure, rupture, decay, and combustion state updates are
self-writes; movement, expansion, and smoke spawn use the ownership pipeline.
No neighbor direct-write, no atomics/global ordering, no per-cell persistent
RNG state.

### G6-B — Ownership Contention Integrity: TECHNICAL PASS / FROZEN
Exactly-one-winner per contended destination; losing cells remain valid;
Matter conserved; one cell joins at most one ownership edge per tick.

### G6-C1 — Arbitration Quality Measurement: COMPLETE / FROZEN

### G6-C2 — Stateless Edge Hash Production Integration: TECHNICAL PASS / FROZEN
Edge-hash arbitration is the closure baseline. It is stateless, cheap, has no
per-cell RNG state, is deterministic for the same tick/state, preserves the
one-cell-at-most-one-edge invariant, and passed translated/mirrored quality
and ownership tests. RTX 5090 full-tick regression 0.0% (historical evidence —
**not re-run** in this closure).

---

## 8. Performance-test policy (this closure)

- Manual performance benchmark: **not run**.
- `coarse_reference_world_perf`: `#[ignore]`.
- `controlled_reference_world_perf`: `#[ignore]`.
- Normal `cargo test` never auto-runs a performance benchmark.
- G6-C2 historical performance measurement preserved as prior evidence,
  not re-run.
