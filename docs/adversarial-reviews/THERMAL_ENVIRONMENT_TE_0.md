# Thermal Environment TE-0 independent adversarial review

- **Review gate:** TE-0A
- **Reviewed source:** `f5c7ac8e76867f769cdf19d7f420432d8fef4509`
- **Method:** independent fresh-context static attack of the design package,
  live pass/binding inventory and drafted ADR/spec
- **Initial verdict:** Critical 0 / High 7 / blocker 7
- **Final verdict after design revision:** Critical 0 / High 7 resolved /
  residual non-blocking risks 5 / blocker 0
- **Runtime implementation:** not started

The reviewer did not author the design. Its first pass rejected TE-1 entry.
Each High finding below was then resolved in the canonical ADR/spec/inventory;
the resolution is an implementation contract, not runtime evidence.

## Findings

### TE0A-H01 — Spawn transaction was not encoded

- **Severity:** High
- **Claim attacked:** a deterministic adjacent receiver alone makes phase and
  Smoke displacement lossless.
- **Counterexample:** the original Matter source→target claim is still needed
  by spawn and expansion-pressure. Reusing it can lose ownership identity; a
  receiver that is also a concurrent Matter destination can leave Air beneath
  Matter; committing Matter before receiver failure creates a partial state.
- **Affected invariants:** TH-INV-007, TH-INV-009, TH-INV-011.
- **Resolution:** the correctness baseline now includes one independent
  full-world `u32 environment_receiver_claim`. The receiver pass excludes all
  same-stage Matter destinations and arbitrates smallest target index. Matter
  spawn is receiver-gated. A separate eight-storage Environment reconcile uses
  both claims, moves the entire parcel, and joint-settles Matter and Air. A
  failure commits neither. The original claim remains live; no unproved reuse
  is claimed. Phase additionally runs a mandatory seven-storage
  Environment-blocked expansion-pressure pass after the existing pressure pass
  and before settle; it applies the existing source exactly once when a Matter
  winner had no receiver. This closes the pass-order ambiguity without treating
  Air state as pressure.
- **Residual risk:** WGSL race freedom and exact command ordering require the
  TE-1 structural/GPU tests.
- **Status:** Resolved in design; TE-1 machine guard required.

### TE0A-H02 — Receiver accumulation had no headroom

- **Severity:** High
- **Claim attacked:** repeated displaced parcels can always add to a receiver.
- **Counterexample:** TE-1 disables flow, so repeated phase/Smoke displacement
  can overflow one deterministic EMPTY receiver or force later clamping.
- **Affected invariants:** TH-INV-006, TH-INV-011.
- **Resolution:** finite `AIR_MASS_MAX` and `AIR_ENERGY_MAX` are required.
  Candidate selection accepts only a receiver with headroom for the complete
  parcel. Otherwise the spawn blocks. Partial transfer, clamp and deletion are
  forbidden.
- **Residual risk:** exact gameplay maxima need TE-1 bounded-value selection.
- **Status:** Resolved in design.

### TE0A-H03 — Vacuum threshold could delete conserved Air

- **Severity:** High
- **Claim attacked:** canonicalizing every sub-threshold parcel to zero is
  compatible with source-free conservation.
- **Counterexample:** a small parcel can split into two sub-threshold parcels
  and disappear; across the full world the aggregate loss is not bounded.
- **Affected invariants:** TH-INV-004, TH-INV-005, TH-INV-006, TH-INV-008.
- **Resolution:** the correctness baseline locks `VACUUM_THRESHOLD = 0`.
  Positive finite residual state remains low-pressure Air. Rounding may reduce
  a proposed flux but cannot delete stored state. Any future nonzero cutoff
  requires conservative routing or a separately measured source/sink budget.
- **Residual risk:** floating-point pair tolerance remains a TE-2 evidence item.
- **Status:** Resolved in design.

### TE0A-H04 — Formula coefficient domains were open

- **Severity:** High
- **Claim attacked:** the written formulas themselves guarantee non-negative
  donor state and passive convexity.
- **Counterexample:** an outflow or thermal mix fraction greater than one
  defeats those proofs.
- **Affected invariants:** TH-INV-003, TH-INV-006.
- **Resolution:** the spec locks finite positive rates/capacities,
  non-negative finite conductance, permeability in `[0,1]`, and outflow/mix
  fractions in `(0,1]`. Post-rounding guards reduce flux and preserve paired
  balance; they never repair a negative result by silently clamping state.
- **Residual risk:** the reference script did not sample the full locked domain;
  TE-2 property tests and semantic fixtures remain required.
- **Status:** Resolved in design.

### TE0A-H05 — Sleep could make face transport one-sided

- **Severity:** High
- **Claim attacked:** existing chunk sleep can be reused without a transport
  execution contract.
- **Counterexample:** a runnable donor can export across a face whose sleeping
  receiver never imports, and an edge reservoir may never wake a sleeping edge.
- **Affected invariants:** TH-INV-004, TH-INV-007 and sleep-on/off equivalence.
- **Resolution:** TE-2 must build a bilateral runnable face cohort before
  transport. Both endpoint chunks and halo execute a face, or neither does; one
  canonical face owner supplies both self-writes from the same Current state.
  Nonzero reservoir flux persistently wakes only the edge chunk and halo.
- **Residual risk:** runnable-frontier cost and sleeping convergence are
  measured only after correctness.
- **Status:** Resolved in design; TE-2 machine guard required.

### TE0A-H06 — Matter progress hygiene did not fit reconcile

- **Severity:** High
- **Claim attacked:** an Air-only reconcile can also guarantee all Matter-local
  flags are valid after identity changes.
- **Counterexample:** reconcile has no flags binding; current phase identity
  replacement does not define all combustion/decay/reserved-bit ownership.
- **Affected invariants:** TH-INV-007, TH-INV-009.
- **Resolution:** Environment reconcile no longer owns this claim. TE-1 locks
  combustion bits 0–1/4–15 to Oil/Wood, decay bits 16–27 to Smoke, and reserved
  bits 2–3/28–31 to zero. A separate within-limit identity-hygiene pass carries
  movement state, sanitizes Matter→Matter, and zeros spawn/EMPTY transitions
  before joint settle. Future latent/ignition progress uses dedicated state.
- **Residual risk:** current preserved-unrelated-bit tests must be deliberately
  superseded at the TE-1 source boundary, not silently weakened.
- **Status:** Resolved in design.

### TE0A-H07 — TE-2 reservoir gate contradicted an open product choice

- **Severity:** High
- **Claim attacked:** TE-2 can require a boundary reservoir while edge policy
  remains undecided until TE-5.
- **Counterexample:** transport, conservation and wake results change with a
  sealed versus replenishing edge.
- **Affected invariant:** explicit source accounting and gate isolation.
- **Resolution:** TE-2 correctness runtime uses sealed/no-flux edges. Its open
  boundary fixture uses an explicit fixed standard-Atmosphere ghost reservoir
  and reports exchange as an external source/sink. Only the product default
  edge mode remains user-owned before TE-5.
- **Residual risk:** product edge choice still affects later gameplay.
- **Status:** Resolved in design.

## Other attacks

- **Air as second Matter:** rejected by the occupancy model; resolved.
- **EMPTY / Atmosphere / Vacuum / Void overlap:** exact predicates are distinct;
  Atmosphere presentation must use `AIR_PRESENT_THRESHOLD` while any positive
  sub-threshold state remains low-pressure Air.
- **GAS curtain:** a continuous Steam/Smoke row is intentionally Air-impermeable
  in the no-mixture baseline. TE-F33 decides whether later GAS permeability is
  needed; no silent leak is allowed.
- **One-cell wall:** Environment permeability is zero; wall Matter conduction
  may still transfer heat.
- **Duplicate transport:** advection carries donor specific energy exactly once;
  unified thermal exchange conducts exactly once.
- **Pressure:** Air, Vacuum and existing gauge pressure remain separated through
  TE-4. TE-5 cannot start until one effective face-pressure formula is locked.
- **Phase cancellation / latent state:** correctly blocked to TE-3; TE-F20 and
  TE-F34 require exact reversal and yield accounting.
- **Inspector honesty:** TE-1 remains the existing Matter-only 24-byte sample.
  TE-6 must atomically bind any Environment extension to world epoch, selection
  generation, Cell and simulation tick.
- **License ingress:** external code copied/translated/vendored is zero. GPL
  Powder Toy code remains fixture/formula provenance only.
- **Gate leakage:** TE-1 contains state/occupancy hygiene and the unavoidable
  receiver-gated spawn outcome only. Air flow, thermal exchange, phase retune,
  ignition and Air-pressure coupling remain disabled.

## Residual register

The remaining risks are later-gate questions, not TE-1 architecture blockers:

1. product default edge reservoir mode before TE-5;
2. Vacuum combustion policy before TE-4/TE-5;
3. phase latent coefficients, yield and reversal before TE-3;
4. GAS Matter Environment permeability only if TE-F33 proves a blocker;
5. TE-2 update cadence/resolution after the full-resolution baseline.

Final gate result:

```text
TE-0A: ADVERSARIAL REVIEW COMPLETE
Critical: 0
High: 7 found / 7 resolved in canonical design
Critical/High blocker: 0
TE-1: READY / NOT STARTED
```
