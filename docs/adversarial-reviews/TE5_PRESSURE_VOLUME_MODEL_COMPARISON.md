# TE-5X Pressure-Volume Model Comparison — Fresh-Context Adversarial Review

- **Review date:** 2026-08-21
- **Branch:** `feature/m0-g9-first-playable`
- **Observed HEAD:** `0666d6676029502d340319b8239f4386c2cfa69a`
- **Program authority:** D-022
- **Review mode:** docs/source/external-artifact static inspection only
- **Runtime/build/GPU/proof execution:** prohibited and not performed
- **Runtime status:** TE-3/TE-5 NOT STARTED

## 1. Frozen authority and artifact identity

All six final primary authority bytes were verified directly. The survey hash
reflects the final link-only correction from `air_transport_scale.wgsl` to the
existing `air_flow_scale.wgsl`; its design content is unchanged:

| Authority | SHA-256 |
|---|---|
| `docs/architecture/decisions/ADR-0010-pressure-volume-model-selection.md` | `ac50ca02c95440a8fbb52119794c58e977b567d7a4ee5acc4ba09ac4a3486378` |
| `docs/planning/TE5_PRESSURE_VOLUME_ARCHITECTURE_RESET.md` | `03a1eb668f3ba789ad2a01d0d279e35e0cc1e38cf20e2f7be650f1cae3c73e63` |
| `docs/development/PRESSURE_VOLUME_MODEL_COMPARISON_VALIDATION.md` | `98279a937a8b5ef3a1fdbf49c7dfb0303c271a7e3ff60c1653a12545851ff154` |
| `docs/research/2026-08-21-pressure-volume-algorithm-survey.md` | `157c0f36734a7cf56d7b644b4a4a42a203a22c3c7a4505a3260a6c1d5db11556` |
| `docs/specs/PHASE_THERMODYNAMICS_SPEC.md` | `af60519582261be334c7d4a419c12a45ee74e103edf8b854c2fcd38542cb623b` |
| `docs/architecture/THERMAL_ENVIRONMENT_PRODUCTION_INVENTORY.md` | `29c9768bd2a4b655fa243c0929070f7725bbc5404b729d907c154b6694171929` |

The frozen external comparison artifacts also matched their recorded receipt:

| Artifact | SHA-256 |
|---|---|
| `te5x_pressure_volume_comparison.py` | `0079246918a91faa606d531cb76591af0363dfb3a66d4b88882fc04e33efd8d5` |
| `te5x_pressure_volume_comparison_result.json` | `097f340c265d9e43a23e281a776905add97e6b05c18dedd79d48807558efc116` |

The current worktree was already dirty with the frozen TE-5X authority set and
unrelated documentation changes when this review began. This review changed
only this file and did not treat the observed HEAD as a clean source seal.

## 2. Evidence-integrity verdict

The only proof process failed at the NetworkX version guard before seed
consumption or candidate evaluation. The JSON is a separately written failure
receipt, not script-emitted comparison evidence. Its counts are internally
consistent with that boundary:

| Count | Recorded value |
|---|---:|
| proof processes attempted | 1 |
| completed proof processes | 0 |
| Candidate A evaluations | 0 |
| Candidate B evaluations | 0 |
| Candidate C evaluations | 0 |
| generated states | 0 / 50,000 |
| multi-tick grids | 0 / 10,000 |
| deterministic replay | NOT_RUN |
| fixture matrix | NOT_RUN |
| proof reruns during this review | 0 |
| Rust/Cargo/build/GPU/runtime executions during this review | 0 |

The failure receipt preserves the failed attempt honestly and its script hash
matches the frozen bytes. It cannot establish candidate mathematics,
eligibility, fixture results, selection or ranking. Historical TE-5B/C/D proof
receipts remain source-bound and do not fill any of these zero counts.

## 3. Severity findings

### TE5X-H-001 — The required comparison evidence does not exist

- **Severity:** High
- **Status:** **OPEN — PROGRAM STOP**
- The one allowed process completed no candidate evaluation and no fixture.
  D-022 requires the combined pre-registered comparison before an eligibility
  or recommendation claim. No static preference in ADR-0010 can substitute for
  that missing execution.
- **Impact:** A, B and C are all unqualified. The provisional B/A/C ordering is
  void, exactly as the frozen authorities say.
- **Required resolution:** a new user-authorized evidence identity, not a patch
  or rerun under this task. This review neither requests nor performs it.

### TE5X-H-002 — The frozen fixture matrix is author-favoring and non-executable as a fixture oracle

- **Severity:** High
- **Status:** **OPEN — EVIDENCE-DESIGN BLOCKER**
- `fixture_matrix()` initially labels every candidate on every PVX-F01–F15 row
  `MODELED_PASS`, then changes a few strings. It does not call fixture-specific
  assertions or consume candidate receipts to establish those labels.
- Candidate A's function checks cardinality and long chains, Candidate B's
  function checks random partition equivalence plus hand-written scalar traces,
  and Candidate C's function checks one transport/sink witness. None executes
  movement, editing, reset, Air receivers, rupture mutation, activity/sleep,
  binding layouts or 2048² pass convergence. Yet the matrix would have emitted
  favorable `MODELED_PASS` labels for many such rows if bootstrap had succeeded.
- The response-curve selection also rejects the quadratic option because it
  delays the named consequence, without a predeclared product-time requirement
  that independently establishes why earlier rupture is correct. This favors
  the anticipated B result rather than testing causal pressure behavior.
- **Impact:** even a successful run of the frozen bytes would not support the
  advertised all-candidate F01–F15 comparison or production eligibility.
- **Required resolution:** executable per-candidate fixture oracles and neutral
  response criteria under a new evidence identity.

### TE5X-H-003 — Candidate A substitutes a prohibitive world-size bound for a production algorithm

- **Severity:** High
- **Status:** **OPEN — CANDIDATE A INELIGIBLE**
- The formula `47 + N*(2N+3)` is finite algebra, but the authority does not give
  a mechanically exact bounded WGSL protocol for layered frontier construction,
  vertex-disjoint deterministic selection, predecessor storage, atomic
  multi-edge/Air flips, graph invalidation or certification from arbitrary
  retained state. ADR-0010 itself says that a future protocol must replace this
  projection before selection.
- At 2048² the stated upper bound is about 35.2 trillion passes. During that
  interval newly confined sources are forced to pressure zero and existing
  pressure is merely held. Movement can change the graph before certification
  and restart or invalidate the search. The result is solver delay controlling
  physical pressure rather than a playable confinement response.
- **Impact:** Candidate A violates D-022's bounded/specified production-work and
  solver-delay selection boundary even though it no longer uses depth six.
- **Required resolution:** an exact production protocol with a product-bounded
  certificate latency and complete pass/scratch/binding accounting. Raising or
  disguising another fixed depth is not a repair.

### TE5X-H-004 — Candidate A inherits the unresolved persistent-extent integration blockers

- **Severity:** High
- **Status:** **OPEN — CANDIDATE A INELIGIBLE**
- Exact maximum cardinality repairs TE5D-H-001 only. It does not resolve the
  preserved TE-5D findings for Environment receiver feasibility, reciprocal
  Draw/Erase cleanup, simultaneous owner/target edits, movement/density-swap
  ownership, phase-pressure movement semantics, or every producer that sees a
  reserved target as ordinary `EMPTY`.
- Current `activity_propose` already occupies eight storage bindings and
  classifies Material `EMPTY` as movement capacity without any extent link.
  Movement, Smoke and other EMPTY consumers require either link-aware rewrites
  or separate passes. The two representative eight-binding rows do not name or
  count that consumer audit, link repair, edit arbitration or wake behavior.
- **Impact:** reserved Air can be stranded, a target can be consumed twice,
  pressure can be erased on movement, and chunks can remain falsely active.
- **Required resolution:** close the inherited H-003–H-006 contracts with exact
  pass identities and state ownership; maximum matching alone is insufficient.

### TE5X-H-005 — Candidate B makes distant capacity causal through a zero-conductance abstraction

- **Severity:** High
- **Status:** **OPEN — CANDIDATE B INELIGIBLE**
- A one-cell neck immediately merges component free capacity irrespective of
  neck width, distance or transport time. The universal `0.10` relaxation then
  lowers pressure at the same rate whether the new capacity is adjacent or at
  the far end of a huge chamber. F08 checks only that `100 -> 90`, so it bakes
  in this assumption instead of discriminating a narrow-neck response.
- A newly connected distant EMPTY region can therefore suppress new pressure
  and begin clearing old stress everywhere before phase volume or Air has any
  causal path through the neck. Conversely, a one-cell disconnection changes
  targets globally in one recomputation.
- **Impact:** B's primary-ranked causal/user-readable meaning is unresolved;
  lower post-opening pressure is not proof that the neck caused a physically
  or visually acceptable relief rate.
- **Required resolution:** a predeclared conductance/propagation meaning or an
  explicit user acceptance of instantaneous component equilibrium, followed by
  fixtures varying neck width and distance. This task has neither.

### TE5X-H-006 — Candidate B's CCL convergence contract is not mechanically exact

- **Severity:** High
- **Status:** **OPEN — CANDIDATE B INELIGIBLE**
- The surveyed Shiloach–Vishkin and cuGraph sources establish that parallel CCL
  exists; they do not prove that the unnamed clean-room dense-grid hooking and
  pointer-jumping variant converges in `4*ceil(log2 N)+2` WGSL passes under the
  specified deterministic update rules.
- The proposed fallback is only described as `N-1` label propagation. Its
  dispatch/query total, activation condition, state transition, profiler
  identity and interaction with sleep are absent from the published 188/376
  totals. “Fail closed to the fallback/evidence gate” is not one production
  behavior.
- **Impact:** the exact convergence and bounded-work requirement in PVX-F15 is
  unsupported. The proof, even if it had run, compares CPU BFS with CPU
  union/find and cannot close this GPU contract.
- **Required resolution:** freeze and mechanically prove one exact GPU CCL
  variant, including the fallback path if retained, then recalculate all costs.

### TE5X-H-007 — Candidate B's exact component reduction cannot fit the stated record model

- **Severity:** High
- **Status:** **OPEN — CANDIDATE B INELIGIBLE**
- A partial Water Cell is excluded from the graph and may border up to four
  distinct components. The contract assigns fractions to each component in
  proportion to that component's EMPTY count. A single per-Cell tuple
  `(label, Cell index, EMPTY count, demand bits)` can emit only one label, not
  up to four contributions.
- Determining distinct adjacent representatives and their component EMPTY
  counts also requires labels and reduced counts before the fractional demand
  records can be formed. The two `N`-record `vec4<u32>` buffers, 32 radix passes
  and 22 reduction levels do not state whether they perform four emission
  sweeps, store up to `4N` records, deduplicate repeated adjacent labels, or run
  a second sort/reduction cycle. Histogram/offset/control scratch and each
  overwrite lifetime are likewise absent.
- **Impact:** the 160 MiB, 188-pass, 376-query and representative binding claims
  are not exact; quantity is algebraically promised not duplicated, but its
  executable contribution representation is missing.
- **Required resolution:** an exact record multiplicity, deduplication and
  reduction schedule with bytes, pass identities, bindings and float order.

### TE5X-H-008 — Candidate B has no exact pressure ownership across movement, split or merge

- **Severity:** High
- **Status:** **OPEN — CANDIDATE B INELIGIBLE**
- The design adds a per-Cell phase-pressure Current/Next pair but describes a
  component target. It does not say whether pressure is Matter-owned, spatial,
  component-uniform or a per-Cell relaxation state when Steam moves, two
  chambers merge, one chamber splits, an EMPTY becomes Matter, or a Cell exits
  to Void.
- Reassigning source demand to newly adjacent components is stateless, while
  old pressure is persistent. Without a transfer/merge/split rule, one movement
  can leave old stress behind and create a new target elsewhere, duplicate a
  transient pressure consequence, or erase it through cleanup.
- Draw/Erase/reset staging is asserted but no writer order or canonical values
  are included in the 188 passes. Rupture must also consume phase and generic
  gauge stress exactly once, which the comparison does not schedule.
- **Impact:** conservation, reversibility, movement hygiene and effective
  stress are indeterminate.
- **Required resolution:** choose pressure ownership and specify every
  occupancy-changing writer plus merge/split and rupture ordering.

### TE5X-H-009 — All candidates leave activity, sleep and terminal convergence unspecified

- **Severity:** High
- **Status:** **OPEN — CROSS-CANDIDATE BLOCKER**
- “Separate activity and wake halo” is a label, not a detector contract. The
  current base activity pass already binds eight storage buffers, so none of
  the new state can simply be observed there.
- A can remain unfinished for an enormous number of passes or be invalidated
  by movement. B's `p += 0.10*(target-p)` is asymptotic, but no settle epsilon,
  canonical zero, wake propagation, component-change marker or terminal-tail
  rule is frozen. C's scalar diffusion has the same asymptotic/sleep issue in
  addition to its sink failure.
- **Impact:** sleep-on/off and chunk-partition equivalence in F14 are not
  established; premature sleep loses work while exact inequality can keep the
  world awake forever. The extra activity pass also makes the published pass
  counts incomplete until its settle/reduction/wake passes are named.
- **Required resolution:** exact activity predicates, epsilon/canonicalization,
  wake halo, reduction, settle and profiler entries for each candidate.

### TE5X-H-010 — Candidate C cannot conserve its claimed volume through condensation

- **Severity:** High
- **Status:** **OPEN — CANDIDATE C INELIGIBLE**
- Once positive volume diffuses away, a local negative phase-demand delta can
  exceed the scalar remaining at the condensing Cell. Negative debt violates
  the non-negative field, clipping leaves orphan volume, owner/debt state adds
  unexplained persistence, and connected-region withdrawal changes the model
  into B.
- Steam movement not sourcing the field twice avoids one duplication path, but
  no exact movement, density swap, Void, edge, Draw/Erase or reset rule removes
  the existing remote scalar with the conserved phase event.
- **Impact:** C fails quantity/phase-volume conservation and reversibility. The
  logical witness is sound as static analysis, but the frozen process executed
  it zero times and therefore supplies no proof receipt.
- **Required resolution:** none within Candidate C's locked representation; it
  requires ownership/debt or component withdrawal and is no longer C.

### TE5X-H-011 — World-edge relief and background occupancy are outside every candidate's closed contract

- **Severity:** High
- **Status:** **OPEN — CROSS-CANDIDATE BLOCKER**
- A matches only an in-domain unowned EMPTY target with a feasible Environment
  receiver, so an out-of-domain Void edge cannot be owned as relief. B counts
  only in-domain EMPTY/GAS graph nodes. C transports only through accessible
  in-domain nodes. None defines whether phase volume vents, remains, or raises
  pressure at the product world edge.
- B deliberately gives Atmosphere and Vacuum identical capacity while leaving
  background Air pressure deferred. That avoids directly deleting Air, but it
  also allows the same atmospheric EMPTY to count as free Vapor capacity
  without an Air displacement/compression transaction. The product consequence
  is unverified, and no candidate fixture measures Air mass/energy or accepted
  local H across the complete heat→phase→movement→rupture→relief chain.
- **Impact:** an open boundary can be falsely confined or scalar volume can
  leak; atmospheric capacity can overlap derived Air without a causal volume
  transaction. F07/F10/F11 do not execute these paths.
- **Required resolution:** freeze edge mode and background occupancy semantics,
  then add exact Air/H/phase-volume conservation fixtures. Deferral is not an
  eligibility proof.

## 4. Resolved attacks and non-findings

| Attack | Resolution |
|---|---|
| Fixed-depth substitution | A does not merely raise depth six, but H-003 finds its replacement protocol still production-ineligible. |
| Candidate A pre-certificate false rupture | No false rupture is created because pressure is held/zero; instead H-003 finds unbounded practical suppression of real confinement response. |
| Candidate B quantity duplication | The algebraic weights sum to one, but H-007 leaves the executable multi-component record representation unresolved. No direct extra foreground Steam is specified. |
| Direct phase-energy loss in formulas | The common `r(E)` derives from accepted bounded phase energy and does not itself mutate H. End-to-end Air/H preservation is still untested under H-011. |
| External implementation ingress | **CLOSED for this review.** Surveyed external work is reference-only; no copied/translated/vendored production implementation was found (`0 files / 0 lines`). NetworkX is only the failed external oracle dependency. |
| Historical proof reuse | **CLOSED for this review.** The authorities explicitly preserve TE-5B/C/D as blocked, source-bound history. No old proof is counted toward TE-5X's zero evaluations. |
| Proof-result authenticity | **CLOSED narrowly.** The failure receipt and frozen script hashes match and the receipt does not claim candidate completion. It remains incomplete evidence under H-001. |

## 5. Candidate comparison after attack

| Candidate | Unresolved High blockers | Eligibility | Comparative disposition |
|---|---|---|---|
| A exact persistent-extent maximum matching | H-001, H-002, H-003, H-004, H-009, H-011 | Ineligible | No Retained fallback |
| B shared connected gas-chamber capacity | H-001, H-002, H-005, H-006, H-007, H-008, H-009, H-011 | Ineligible | No Recommendation |
| C conservative Vapor-volume scalar | H-001, H-002, H-009, H-010, H-011 | Ineligible | No evidence-ranked Rejection; statically representation-blocked |

No fourth candidate is synthesized. TE-5B, TE-5C and fixed-depth TE-5D remain
rejected historical designs; this review does not revive them or transfer their
proofs.

## 6. Remaining risks and validation boundary

- Current source contains none of the A/B/C runtime state or passes. All byte,
  pass, query and binding figures are static projections.
- No WGSL compilation, race analysis, device-limit validation, allocation,
  performance, movement, edit/reset, Air receiver, rupture, activity/sleep,
  rendering, product or user evidence exists for TE-5X.
- The failed proof environment identity is incomplete: the imported
  `networkx` namespace had no version/file identity, so the intended 3.6.1
  oracle was not authenticated by the process.
- The observed dirty worktree is not a clean source seal. Exact primary hashes
  make this review reproducible for the named documents, but do not create a
  runtime or repository commit provenance claim.
- Any future comparison must be a separately authorized evidence identity and
  must not rewrite this one-shot failure into a completed result.

## 7. Verification performed and omitted

Performed:

- Ballast checkpoint/decision recall for D-019 through D-022;
- current branch, HEAD and dirty-state inspection;
- direct reads of ADR-0007/0008/0009 and their blocked reviews;
- exact SHA-256 verification of all six frozen primary authorities and both
  external artifacts;
- static inspection of the frozen script/result, current pass inventory,
  binding ceiling, scratch lifetimes, movement, activity and sleep constraints.

Intentionally omitted:

```text
proof/reference process execution: 0
proof rerun or patch: 0
Rust/Cargo/test/check/clippy: 0
WGSL/Naga/GPU/device: 0
build/launch/runtime: 0
candidate/fixture/performance: 0
commit/push: 0
other repository writes: 0
```

## 8. Final disposition

Unresolved Critical: **0**

Unresolved High: **11**

Unresolved Medium: **0**

**TE-5X DESIGN BLOCKED**

There is **no Recommendation**, **no Retained fallback**, and no evidence-ranked
candidate ordering. ADR-0010 remains **PROPOSED / DESIGN BLOCKED**, the one-shot
comparison remains **INCOMPLETE_EVIDENCE**, and TE-3/TE-5 runtime remains **NOT
STARTED**. The exact stop reasons are: zero candidate evidence; a non-executable,
author-favoring fixture matrix; A's unspecified/prohibitive production solver
and inherited extent integration gaps; B's noncausal chamber meaning and
incomplete CCL/reduction/state-cost contract; C's condensation sink; and
cross-candidate activity, edge, Air and end-to-end conservation gaps.

The on-disk SHA-256 of this completed review is reported in the completion
receipt after the final bytes are written; embedding a conventional hash in
the file it hashes would change that hash.
