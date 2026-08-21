# Independent TE-5D Persistent Vapor Extent Design Review

- **Date:** 2026-08-21
- **Reviewer role:** fresh-context independent adversarial reviewer; not a
  primary-author participant
- **Reviewed branch:** `feature/m0-g9-first-playable`
- **Program authority:** D-021
- **Review-time HEAD:** `d0efba3f9ab39ef9499ee63d26fcba968b258d05`
- **Current unresolved:** Critical **0**, High **6**, Medium **2**
- **Runtime/proof execution by this reviewer:** **0**
- **Verdict:** **TE-5D DESIGN BLOCKED / ADR-0009 PROPOSED / ARCHITECTURE
  REVISION REQUIRED / RUNTIME NOT STARTED**

## 1. Scope and evidence boundary

This review attacks the D-021-authorized docs/reference candidate
**PERSISTENT VAPOR EXTENT + DEDICATED PHASE PRESSURE**. It directly read live
Git state, D-021, ADR-0005/0006, blocked ADR-0007/0008, the TE-3 and TE-5B/C
specifications, validations and independent reviews, the four frozen TE-5D
authorities, the external proof script/result, and the actual production
movement, Environment, Air/thermal, phase, Smoke, generic expansion, pressure,
rupture, activity, profiler, allocation, staging, reset and Sandbox edit paths.

The repository was already user-dirty. This review did not modify an existing
file, runtime source, proof artifact, authority, memory record, Wiki page or
status router. This review file is the only reviewer-authored write.

The external script and result were read and SHA-256 hashed. They were not
executed, regenerated or modified. No Cargo, Rust, WGSL, Naga, GPU, build,
launch, fixture, performance or product command was run.

## 2. Frozen input identity

| Input | SHA-256 |
|---|---|
| ADR-0009 | `5b728123949968e6373b37e42bfa3240f1a31d9297312bc5e98e08b2f193fce3` |
| persistent extent specification | `1d0ed0c417f0e8945ea697be013425c938a41a8dcf2354c41bb43ba2b58e98ac` |
| persistent extent validation | `8dcb9345cdd087cd80f0f8029fd3e957ebf27e3858e2f1ddba9e999b9c7ac241` |
| TE-5D plan | `767b4a377c85ccf00f835abc96c267af85f21c4624826c71f85cf06c963f135e` |
| external proof script | `06d0cea8500fcc3a2ffa4010d0dab70770a3fb2fd8a94f0bf47846cd980dedb9` |
| external proof result | `853379af86ee536166cb752bffbf45cefe5eec93bc10038a023679d507d7a29a` |

The parsed result reports `DESIGN_BLOCKED`, seed `0x54453544`, 50,000 random
graphs, 10,000 abstract grids, 682 directly enumerated graphs through 3x3, 12
structured 6x6 families, and zero of the required `2^36` labeled 6x6 graphs.
Its only serialized blocker is
`literal_exhaustive_all_labeled_6x6_not_executed`.

## 3. Finding counts

| Severity | Open | Resolved in reviewed snapshot |
|---|---:|---:|
| Critical | **0** | 0 |
| High | **6** | 0 |
| Medium | **2** | 0 |

Any one High invokes D-021's stop rule. H-001 independently reproduces the
primary matching blocker. H-002 through H-006 are independent evidence,
transaction, resource or state-semantics blockers.

## 4. Findings

### TE5D-H-001 — Depth six cannot repair a legal persistent complete matching

- **Severity:** High
- **Status:** **OPEN — PRIMARY DESIGN BLOCKER INDEPENDENTLY REPRODUCED**
- **Witness:** Let `U0..U6` already own `V0..V6`; `U7` is compressed and
  unmatched. Give `U7` only `V0`, and each `Ui` edges to `Vi` and `V(i+1)` for
  `i=0..6`. `V7` is free. The complete assignment is `U7->V0` and
  `Ui->V(i+1)`. Its only augmenting path visits the eight source vertices
  `U7,U0,...,U6`.
- **Independent result:** `MAX_REASSIGNMENT_DEPTH=6` cannot reach `V7`.
  Because a failed search is atomic, it preserves the same matching. Age-based
  retry changes source order but not the unique path length. Every retry
  therefore fails unchanged until the unmatched source's isolated recurrence
  `p[n+1]=p[n]+0.1*(100-p[n])` crosses 80 on tick 16.
- **Impact:** A complete extent assignment exists, yet the design can produce
  rupture-capable false phase pressure. PVX-INV-011 and the hard acceptance
  rule fail.
- **Required repair class:** **wider matching scope** is mandatory. A bounded
  parallel implementation may additionally need **full-world
  frontier/predecessor scratch** or a different pass bound. This witness does
  not require another persistent field, a different volume representation or
  relaxation of 1:1 quantity.

### TE5D-H-002 — The one-shot proof does not execute its persistent claim or most named fixtures

- **Severity:** High
- **Status:** **OPEN — EVIDENCE INTEGRITY BLOCKER**
- **Observed static facts:** `bounded_candidate()` creates empty
  `source_target`/`target_owner` arrays on every call. The long-witness loop
  calls it afresh every tick, so it never supplies the already-persistent
  `Ui->Vi` initial state required by H-001. Its history alternates between
  seven and eight fresh matches; the serialized final `candidate_size` and
  `exact_size` are both 8, and
  `complete_matching_hard_product_condition` is `true`.
- The result consequently contains no bounded-depth blocker. It is blocked
  only because all-labeled 6x6 coverage visited zero graphs. The persistent
  eight-source witness is valid static post-receipt reasoning, not an output
  produced by the frozen proof.
- `grid_trial()` tracks only integer counts (`water_count`, `steam_count`,
  `reserved`, `compressed`), one scalar `air_total`, one generic-pressure
  scalar and one phase-pressure scalar. It has no grid occupancy, reciprocal
  link arrays, Air receiver graph, movement edge, density swap, condensation
  transaction, Void edge, edit/reset path, activity/chunk partition or
  rupture-neighbour evaluation. The named TE5D-F01 through F16 geometries and
  event orders do not appear in the script.
- **Impact:** Authentic hashes and deterministic bytes do not establish the
  validation contract's claims for reciprocal movement, Air refill exclusion,
  convergence/diffusion, exactly-once rupture stress, generic separation,
  edit/reset, activity or staging. These are **NOT ESTABLISHED**, not PASS.
- **Disposition:** Preserve the one-shot artifact. Do not patch, rerun or
  reinterpret it as persistent-state proof. A revised architecture needs a
  separately authorized proof contract.

### TE5D-H-003 — Matching ignores whether target Air can actually be displaced

- **Severity:** High
- **Status:** **OPEN — TRANSACTION/MATCHING BLOCKER**
- **Witness:** A compressed Steam has two approved EMPTY targets. The first
  deterministic target contains Air but all of its orthogonal receiver Cells
  are occupied, already claimed or lack whole-parcel headroom. The second is
  Vacuum, or has an eligible whole-parcel receiver. A complete *committable*
  reservation exists through the second target.
- **Conflict:** Sections 2-3 of the specification build matching candidates
  from in-domain, unreserved Material `EMPTY` only. Environment receiver
  acceptance happens after matching. On failure the transaction is
  byte-identical and the source compresses, but no rule removes the failed
  target edge, advances to the next target, or incorporates receiver
  feasibility in the next deterministic retry. The same unusable target can
  win forever.
- **Impact:** Sufficient usable capacity can still generate false pressure,
  independently of augmenting-path depth. TE5D-F10 proves only that one failed
  receiver compresses; it does not establish fallback to another feasible
  matching. PVX-INV-008 and PVX-INV-011 are not jointly satisfied.
- **Required repair class:** widen the matching domain to a single atomic
  **reservation-plus-Environment transaction**, with receiver ownership in
  the edge feasibility/commit proof. This likely changes matching scratch and
  pass counts; a coefficient change cannot repair it.

### TE5D-H-004 — Editor mutation has no reciprocal remote-cleanup transaction

- **Severity:** High
- **Status:** **OPEN — ORPHAN/AIR ACCOUNTING BLOCKER**
- **Witness A:** Erase a `TARGET_RESERVED` Cell. It is already Material EMPTY,
  so the production Sandbox Erase path writes standard Air into both Air
  halves. The TE-5D text blocks Draw, but does not block Erase or define an
  owner update. The target remains linked yet is no longer exact-zero Air.
- **Witness B:** Erase or overwrite the owning Steam. The current bounded edit
  dispatch writes only the commanded Cell's Material/temperature/pressure,
  flags and Environment. It cannot atomically clear a remote reciprocal
  target. `fail closed and report validation fault` leaves an orphan reserved
  capacity and is not the canonical edit hygiene promised by PVX-INV-018.
- **Production anchor:** `apps/windows/src/sandbox.rs` runs separate
  Environment, flags and field passes. Its field pass is already at seven
  storage bindings including the command buffer, and its Environment pass is
  also at seven including commands; neither binds a link or remote-owner
  transaction.
- **Impact:** Ordinary authoring can create nonreciprocal links, inject Air
  into an extent, leak capacity and strand pressure. Reset-to-zero is easy;
  arbitrary edit cleanup is not equivalent to reset and is absent from the
  62-pass/tick and editor projections.
- **Required repair class:** a separately specified link-aware edit
  arbitration/cleanup pass, or another ownership representation that permits
  deterministic reciprocal release. It must cover simultaneous brush edits
  of owner and target without two writers.

### TE5D-H-005 — The fixed pass/binding/no-scratch projection omits required link-aware consumers

- **Severity:** High
- **Status:** **OPEN — RESOURCE/ACTIVITY BLOCKER**
- **Observed production constraints:** a reserved target is still Material
  EMPTY. Current movement, generic expansion, Smoke targeting and base
  activity therefore see it as free unless their claim/eligibility paths read
  the link. `activity_propose.wgsl` already has eight storage bindings:
  Material, temperature, generic pressure, flags, class, density, activity RW
  and combined phase/conductivity tables. Adding the link exceeds the observed
  ceiling unless tables/passes are redesigned. Its movement frontier would
  otherwise keep chunks active by repeatedly seeing the disguised EMPTY.
- `combustion.wgsl` also has eight storage buffers and chooses Smoke targets
  from `material_current == EMPTY`; masking can be moved to a later claim pass,
  but that producer/consumer contract and its activity consequence are not in
  the authority set. Generic expansion and every other EMPTY consumer require
  the same audit. Air/thermal scale masking alone is insufficient.
- The projected 22-pass delta counts a movement context, 18 matching rounds,
  one Environment receiver, one commit and one phase-pressure update. It does
  not name the link-aware editor cleanup from H-004, a reciprocal validation/
  repair path, the base-activity binding redesign, or the wider search work
  required by H-001. The future proof contract itself says exact bindings and
  implementation remain future gates, while the ADR simultaneously claims
  62 passes, 124 queries and no new full-world scratch.
- **Impact:** PVX-INV-017, PVX-INV-018 and PVX-INV-019 are not supported by a
  coherent projected graph. A target may be consumed by an unmasked Matter
  writer or create permanent false activity; fixing it changes the published
  pass/binding/scratch/memory boundary.
- **Required repair class:** at minimum **wider matching scope plus an explicit
  pass/binding rewrite**. A full-world frontier/predecessor allocation must be
  counted if selected. The 62/124 and 64-MiB-only totals must not be carried
  forward unchanged.

### TE5D-H-006 — Phase-pressure ownership on movement is undefined and can erase stress

- **Severity:** High
- **Status:** **OPEN — STATE SEMANTICS BLOCKER**
- **Witness:** A compressed or formerly compressed Steam with positive
  `phase_pressure` moves into its own extent, another EMPTY or through a
  density swap. The movement rules say phase energy and the *source-side link*
  follow Steam identity, but do not say whether `phase_pressure` follows the
  Steam or remains a spatial field like generic `pressure[]`.
- If phase pressure remains spatial, owner-to-own-extent makes the old owner
  Cell the new `TARGET_RESERVED`, which is required to have exact phase
  pressure zero. The positive value is therefore deleted immediately rather
  than relaxing after relief. The destination owner receives the old target's
  zero. Ordinary movement can erase rupture stress.
- If phase pressure follows Matter, every movement/density-swap path must
  transport it with owner identity, while diffusion still treats it as a
  spatial gauge field. That ownership rule, its reciprocal target zeroing,
  and its interaction with the movement context are not specified or counted.
- **Impact:** PVX-INV-009, PVX-INV-012 through PVX-INV-015 and the promised
  pressure delay/clear behavior are indeterminate. The proof uses one scalar
  and cannot distinguish the two semantics.
- **Required repair class:** explicitly choose and prove Matter-owned or
  spatial phase-pressure movement semantics, then recalculate the movement
  pass/binding contract. This does not by itself require relaxing 1:1.

### TE5D-M-001 — Reserved EMPTY is absent from product and Environment accounting contracts

- **Severity:** Medium
- **Status:** **OPEN**
- D-021 permits the auxiliary extent, so its hidden occupancy is not by itself
  a One Cell = Max One Matter violation. However existing readback, scenario
  accounting and EMPTY/Air summaries classify by Material and Air arrays. A
  `TARGET_RESERVED` Cell is Material EMPTY with zero Air, and is therefore
  indistinguishable from ordinary Vacuum unless every accounting consumer
  also reads the link.
- The authority set defines physics masking but no presentation/readback,
  inventory, scenario or telemetry classification. Future validation could
  therefore report a Vacuum or free-EMPTY count that includes consumed extent
  capacity.
- **Required closure:** name extent count, ordinary Vacuum count, atmospheric
  EMPTY count and reciprocal-orphan count separately in structural fixtures
  and product diagnostics. This is an accounting obligation, not permission
  to register extent as Matter.

### TE5D-M-002 — Matching-settle and pressure-update tick boundaries are not exact

- **Severity:** Medium
- **Status:** **OPEN**
- The design allows six settle ticks and says six-tick matching delay stays
  below Wood threshold, but does not state whether a newly compressed source
  performs its first `p_eq=100` update on the completion tick, whether a match
  on settle tick six changes equilibrium before or after that tick's pressure
  update, or which link half rupture reads.
- The difference is an off-by-one in pressure history and in the exact
  `matching -> pressure -> rupture -> joint settle` snapshot. It does not
  rescue H-001, whose source eventually crosses in every ordering, but it is
  required for reproducible F02/F04/F11/F12 evidence.

## 5. Required attack coverage summary

| Attack | Result |
|---|---|
| reciprocal divergence | H-004; fail-closed fault is not canonical cleanup |
| Air displacement/refill | H-003/H-004; feasible fallback and Erase refill are unresolved |
| movement/density swap/orphan | H-004/H-006 |
| persistent initial matching | **H-001 reproduced; proof does not model it, H-002** |
| convergence | coefficient convexity is plausible only for a fixed graph; moved ownership is undefined, H-006 |
| phase-pressure delay/clear | isolated recurrence crosses at tick 16; relief ordering and movement clear are unresolved, H-006/M-002 |
| generic pressure separation | separate storage is specified; proof does not exercise the production path, H-002 |
| exactly-once effective stress | projected combined-table rupture can fit eight storage bindings, but proof and snapshot semantics are absent, H-002/M-002 |
| rupture eight bindings | arithmetically feasible only after combining threshold/class tables; no runtime evidence |
| disguised Matter / reserved EMPTY | authorized auxiliary state, but all free-EMPTY consumers and accounting require link awareness, H-005/M-001 |
| Environment accounting | H-003/H-004/M-001 |
| global solver/scratch/memory/pass understatement | **H-001/H-005** |
| external-code boundary | no copied/vendored implementation or external formula was found; proof is pure reference, not production provenance |
| 1:1 phase quantity | no counterexample found; none of the blockers requires relaxing it |
| generic non-family target hygiene | ADR-0006's prohibition remains necessary; TE-5D grants no bypass |

## 6. Proof disposition and limitations

The external result's `DESIGN_BLOCKED` verdict is authentic for its exact
bytes, but its directly demonstrated blocker is only the unmet 6x6 exhaustive
coverage obligation. The result does **not** demonstrate the persistent
eight-source failure; H-001 is an independent static counterexample discovered
outside the executed model. The script/result must not be described as having
executed F01-F16 or arbitrary persistent reciprocal state.

The useful positive evidence is narrow: deterministic fresh-start matching on
the enumerated/sampled graphs, simple aggregate quantity/Air count arithmetic,
the isolated scalar pressure recurrence, coefficient arithmetic and unchanged
generic-pressure scalar. It is not WGSL, GPU, binding, race, full-grid
movement, receiver-feasibility, activity, edit, staging, profiler, rendering,
performance or user-acceptance evidence.

## 7. Verification performed and omitted

Performed:

- selective Ballast recall and direct authority/source reads;
- live Git HEAD/status inspection;
- SHA-256 verification of all six frozen TE-5D/proof inputs;
- standards-compliant JSON inspection;
- independent graph/path reproduction of H-001;
- static producer/consumer, pass-order, binding, activity/sleep, movement,
  Environment, edit/reset, pressure and rupture inspection.

Intentionally omitted:

```text
proof process execution: 0
Rust/Cargo/test/check/clippy: 0
WGSL/Naga/GPU/device: 0
workspace FULL: 0
build/launch/runtime: 0
fixture/performance/product: 0
Wiki edits: 0
other repository writes: 0
```

## 8. Final disposition and blocker classification

Unresolved Critical: **0**

Unresolved High: **6**

Unresolved Medium: **2**

Exact disposition:

**TE-5D DESIGN BLOCKED / ADR-0009 PROPOSED / ARCHITECTURE REVISION REQUIRED /
RUNTIME NOT STARTED.**

Mandatory blocker classification:

| Candidate repair class | Required by this review? |
|---|---|
| wider matching scope | **Yes — H-001** |
| full-world frontier/predecessor scratch | **Potentially yes** for a bounded GPU realization; must be explicitly authorized and counted |
| another persistent field | **Not established as necessary** |
| different volume representation | **Not established as necessary** |
| relaxation of 1:1 phase-family quantity | **No; preserve 1:1** |

Before any architecture pass, a replacement must also integrate Environment
receiver feasibility into matching, define reciprocal editor cleanup and phase
pressure movement ownership, and replace the incomplete 62-pass/no-scratch
projection with exact pass, binding, scratch, profiler and allocation tables.
No runtime implementation, proof rerun, coefficient retune, evidence rebinding,
G5 gate advancement or revival of ADR-0007/0008 is authorized by this review.
