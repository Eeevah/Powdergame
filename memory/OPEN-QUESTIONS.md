# Open questions

This is an append-only register of user-owned pending choices and dated dispositions. It is not proof of runtime state; open the linked canonical evidence for exact claims.

## Q-001 · Heavy Mixed user acceptance — closed 2026-08-19

Disposition: `USER ACCEPTED WITH KNOWN FOLLOW-UP`. Automatic `NEEDS_HUMAN_REVIEW`, 14/14 hard PASS, `candidate_blocker=false`, immutable artifacts and declining `broad_terminal_tail` remain unchanged. See D-003 and the Heavy evidence record.

## Q-002 · G8-A same-SHA visual durable disposition — closed 2026-08-19

Original owner: user

Original known evidence:

- source `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`
- capture `g8a-v5-9abec9e-20260817T032827206Z`
- official capture and independent verification complete

Disposition:

The user selected formal supersession. G8-A v5 remains verified technical evidence. The separate old same-SHA visual requirement is **SUPERSEDED**, not retroactively marked `PASS`, by the later direct G8-B Gallery/Cell Inspector user approvals and the independently verified G8-C windowed Matrix. The old capture is not replayed or rebound.

See D-007 and `docs/evidence/G8_PERFORMANCE_GATE_USER_CLOSURE_2026-08-19.md`.

## Q-003 · Authorize G8-C after Heavy acceptance — closed 2026-08-19

Disposition: user authorized G8-C Official Performance Matrix only. This did not authorize G9, optimization, G8 closure or `main` promotion.

## Q-004 · What follows the verified G8-C Matrix? — closed 2026-08-19

Original owner: user

Known official evidence:

- sealed source `4653d7c2e09e93f80fb81eeb73458d992c86858f`
- Matrix `g8c-official-matrix-4653d7c2e09e-64df60ba0d79`
- Receipt `1fbf4599893cc29e99b6033996b42fcdf025aac0b421cb80b95b3e55807455f6`
- package `92f8b85cc0e34ea6e71a9f6b4fc95b0f70704263a0f798a69a830cce1d40b729`
- verification `77c7e1c982296277c451de02c3dca68fa6d7d9a90e9fd5426c4dffa1abd9bb0d`
- verifier: 230 recomputed fields, mismatch `0`
- recommendation: `PROCEED_TO_G9`

Disposition:

The user approved the recommended G9 product brief in D-006. G9-A is the next implementation step; optimization remains deferred. The first task stops at a user-testable editor/sandbox candidate before Discovery or other expansion.

## Q-005 · Integrate Ballast into the active product line — closed 2026-08-19

Disposition:

- PR #4 was retargeted to clean `feature/m0-g8c-official-matrix` at source `4653d7c2e09e93f80fb81eeb73458d992c86858f`.
- Development Policy Audit passed.
- PR #4 was merged with a merge commit, not squash.
- Integration merge: `6b5f0201f882f212f9916521aec689261d97b4a6`.
- Product/evidence history remains the first parent.
- Preferred rollback after reverting later Ballast-only commits is `git revert -m 1 6b5f0201f882f212f9916521aec689261d97b4a6`.

## Q-006 · Window-lifecycle replacement pilot authorization — closed 2026-08-19

Disposition: user authorized the narrow lifecycle fix, one replacement pilot and conditional official capture. Lifecycle passed; official capture did not run under this authorization because aggregation found the historical CSV vocabulary mismatch.

## Q-007 · Historical CSV adapter and official capture authorization — closed 2026-08-19

Disposition:

- external `wall_per_tick` + `ms/tick` remains canonical;
- explicit adapter maps to internal `wall_ms_per_tick`;
- aggregation replay passed with zero launched processes and stayed non-evidence;
- sealed source `4653d7c2e09e93f80fb81eeb73458d992c86858f` was pushed clean/upstream-equal;
- official Matrix, independent verification and package each ran exactly once;
- recommendation `PROCEED_TO_G9`.

## Q-008 · Thermal Environment later-gate choices — open 2026-08-20

Owner: user at the named evidence boundary.

D-013 and ADR-0005 close the Environment ontology: Air is a separate
mass/energy Field, Atmosphere and Vacuum are distinct from foreground EMPTY and
Void, and the correctness baseline uses full-resolution state. Do not reopen
those choices as implementation questions.

The remaining choices are deliberately later-gate decisions:

- product default world-edge reservoir mode before TE-5 integration; TE-2
  correctness uses sealed edges and an explicit fixture-only ghost reservoir;
- whether combustion is supported in Vacuum before TE-4/TE-5 closure;
- phase latent coefficients, yield and reversal representation before TE-3;
- GAS Matter Environment permeability only if TE-F33 demonstrates a product
  blocker in the no-same-cell-mixture baseline;
- any post-baseline Air-flow cadence/coarsening optimization. D-015/TE-2 has
  now measured and preserved the full-resolution every-tick correctness
  baseline; no optimization was inferred.

Current disposition: TE-2 is **USER ACCEPTED WITH KNOWN FOLLOW-UP**. Product
edge mode, Vacuum combustion, latent phase/yield and optional GAS permeability
remain open at their named later gates. The implemented sealed correctness edge
and explicit fixture reservoir do not close the product edge choice.

The direct review registered a concrete TE-3 blocker: the available-space
round trip is `1 Water -> up to 2 Steam -> up to 2 Water`, so a closed cycle can
gain Water-equivalent Cells. The future design must compare 1:1 Matter plus
Environment/pressure expansion, a primary Steam Cell plus explicit bounded
expansion-fragment/contraction state, and a dedicated bounded phase-quantity
representation. D-017 now authorizes a docs-only Hybrid A+C proposal with 1:1
Water-equivalent foreground quantity and dedicated phase enthalpy, but does not
accept that architecture or authorize runtime. See
`docs/planning/TE3_WATER_STEAM_PHASE_ACCOUNTING.md`, D-016 and D-017.

TE-3D candidate update — 2026-08-20:

- proposed ADR-0006 selects Hybrid A+C: one Water-equivalent foreground Cell,
  Water/Steam `yield = 1`, and two dedicated phase-energy halves;
- user approval or revision of that representation remains open;
- proposed `L_f = 80` and `L_v = 480` remain user-review choices;
- proposed surface thresholds, deterministic free-air nucleation and their
  visual result remain user-review choices;
- the approximately 32 MiB phase-buffer cost at 2048² remains a user-review
  choice;
- the atomic activation constraint—no active Water yield-1 path until a
  separately authorized TE-5 replacement preserves the frozen G5
  boil/confinement/rupture/vent chain on the same source—remains a user-review
  choice;
- the static pass/binding inventory found no implementation-feasibility
  blocker, but no runtime implementation or evidence exists;
- the independent review passed with zero unresolved Critical/High findings,
  while retaining the following lower-severity design risks for user review or
  pre-implementation closure:
  - no numeric product bound yet exists for temporal/lattice initiation rate;
  - whether already-started boiling may complete after burial is not a settled
    product rule;
  - Steam with neither a sink nor runnable thermal work may remain metastably
    supercooled;
  - a cold zero-conductivity Boundary can satisfy the geometric sink predicate
    even though it cannot perform thermal work;
  - coefficient timing windows were selected inside the same reference design
    rather than by an independent preregistration;
  - a future generic non-family yield-2 rule needs an explicit target-family
    phase-energy ownership restriction; and
  - the fixed hash mixer constants need clearer provenance before runtime use;
- product world-edge mode and Vacuum combustion remain open at their original
  TE-5 and TE-4/TE-5 boundaries; this proposal does not close them.

The candidate is **PHASE-ENTHALPY DESIGN CANDIDATE / USER ARCHITECTURE REVIEW
PENDING**. Reference math and adversarial review cannot substitute for the
user's architecture disposition.

TE-2 direct re-review is closed **USER ACCEPTED WITH KNOWN FOLLOW-UP**. Preserve
`LONG_HORIZON_SEALED_AIR_DRIFT_BUDGET` and
`TE2_CANDIDATE_HUD_LABEL_POLISH` as non-blocking follow-ups. Do not reopen the
TE-2 Air architecture, Celsius-like temperature or acceptance.

## Current pending user-owned choices

G9-A and TE-2 are user accepted with known follow-up. TE-3 architecture review
and later runtime evidence remain prerequisites to G9-B; they do not reopen
G9-A or TE-2.

Future user decisions occur at the next evidence boundary:

- accept or revise the proposed Hybrid A+C representation, latent coefficients,
  condensation/nucleation appearance, memory cost and pressure boundary at its
  named review gate;
- close Q-008 items only at their named Thermal Environment gate;
- authorize later G9-B/C/D progression after that candidate;
- approve shared `main` promotion;
- give the final G9/M0 product disposition after direct play.

## D-018 TE-3D architecture disposition — closed 2026-08-21

D-018 closes the Hybrid A+C representation, 1:1 yield, two phase-energy
halves, latent constants, real-sink predicate, buried ready-Water semantics,
radius-2 nucleation/veto and 30-tick bound, 32 MiB 2048² cost, isolated-Steam
metastability, generic phase-target restriction, hash provenance and atomic
same-source G5/TE-5 activation constraint. The amended proof and independent
v2 review passed with unresolved Critical `0` / High `0`.

This does not close Q-008 as a whole. Product edge mode, Vacuum combustion,
the actual TE-5 pressure-volume law, separate runtime implementation authority,
source-bound GPU/device/G5 evidence and future product/user observation remain
open at their named gates. TE-3 runtime and the TE-5 bridge remain **NOT
STARTED**.

Current pending user-owned choices after D-018 are the TE-5 pressure-volume
design and later atomic TE-3/TE-5 implementation authorization, the remaining
Q-008 gate-owned items, later G9 progression, shared `main` promotion and the
final G9/M0 product disposition. The superseded pending list above remains as
append-only history rather than being rewritten.
