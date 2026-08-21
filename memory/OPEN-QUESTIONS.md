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

## Q-009 · TE-5B exclusive phase-volume bridge disposition — open 2026-08-21

Owner: user at the ADR-0007 architecture-review boundary.

D-019 authorizes a docs/reference candidate and one independent review; it
does not accept the architecture or authorize runtime. The proposed model is
one exclusive same-tick claim on an in-domain EMPTY Cell from the resulting
Steam's up/up-diagonal/lateral GAS stencil. A winning claim leaves target
Matter/Air unchanged and emits no confinement pressure. No target or a losing
claim completes 1:1 to Steam and emits existing gauge-pressure consequence
`100.0` exactly once.

User disposition remains required for:

- approve or revise the exclusive local volume-relief-token model;
- approve or revise occupancy-only relief for both Atmospheric Empty and
  Vacuum Empty without inspecting derived Air pressure;
- approve or revise the exact `00 none / 01 Matter / 10 relief / 11 invalid`
  encoding under the 30-bit Cell-index bound;
- retain or revise inherited confinement scalar `100.0` in the new atomic G5
  fixture;
- confirm or revise whether early relief, ordinary Steam headspace filling and
  later confinement preserve the intended finite-headspace product meaning.

Even a positive disposition will not close full TE-5 background-pressure and
structure-differential design, product edge mode, Vacuum combustion, separate
atomic runtime authorization or source-bound pass/binding/GPU/performance/user
evidence. ADR-0007 remains Proposed and TE-3/TE-5B runtime remains not started
until a later decision says otherwise.

### 2026-08-21 independent-review update — current candidate blocked

The one-shot pure arbitration/accounting proof passed inside its declared
no-grid model, but independent review found an unresolved High counterexample.
In a sealed one-Cell-wide column, only the top Water is ready at `t0`; each
lower Water reaches the endpoint only after ordinary 1:1 Steam movement brings
the EMPTY vacancy above it. The stagger prevents a same-tick Steam-swap stop,
and the vacancy walks down the column so every completion can win zero pressure
without finite headspace ever becoming unavailable. Same-tick exclusivity does
not consume cross-tick capacity.

Q-009 therefore remains open with ADR-0007 **PROPOSED / DESIGN BLOCKED**. The
next user-owned choice is no longer simple approval of this token. It is which
architecture owns finite-capacity consumption and whether any locked no-new-
state, target-non-mutation or 1:1 constraint may change. Occupancy-only Air
eligibility, the two-bit encoding, pressure `100.0`, edge mode and the final
F05/F11 product meaning remain downstream questions after that blocker is
resolved. No alternate model or runtime work is authorized.

### 2026-08-21 D-020 disposition — token rejected, question superseded

D-020 rejects the exclusive completion token as **REJECTED / DESIGN BLOCKED**
and preserves Q-009 as history. The replacement does not use its two-bit token
or an additive Water-completion impulse. Q-009 is superseded for current design
selection by Q-010.

## Q-010 · TE-5C local Vapor capacity-pressure disposition — open 2026-08-21

Owner: user at the proposed ADR-0008 architecture-review boundary.

D-020 authorizes the final no-new-persistent-state attempt. The locked
candidate derives Vapor demand from accepted phase energy, proportionally
shares each radius-1 EMPTY among adjacent phase Cells, maps compression to a
state-derived gauge-pressure target capped at `100.0`, and treats orthogonal
EMPTY as a gauge-zero vent face. It reuses proposal scratch after Smoke, adds
one projected pass and does not mutate EMPTY Matter or Air.

The exact sharing law, EMPTY vent effect on generic pressure, vacancy-walk and
finite-headspace meaning, open-plume false-pressure control, scratch lifetime,
atomic G5 fixture and scalar response remain subject to the one-shot grid/time
proof and fresh independent review. Any unresolved Critical/High makes TE-5C
DESIGN BLOCKED and requires the next decision to permit persistent
phase-volume state. Runtime, background Air pressure, structure differential,
product edge mode and Vacuum combustion remain separate and not started.

### 2026-08-21 one-shot proof update — TE-5C blocked

The locked proof executed once and returned `DESIGN_BLOCKED`. F01–F13,
50,000 static neighbourhoods, 10,000 multi-tick grids and deterministic replay
passed their modeled checks. The predeclared
`reachable_capacity_no_false_pressure` control failed: a complete
two-Steam/two-EMPTY assignment exists, but the proportional law discards an
excess share at one capped Steam and leaves the other at capacity `0.5` with
target `100`.

Fresh-context review independently reproduced that failure and left five more
High findings open: internal EMPTY capacity/vent conflation, irreversible
phase-pressure provenance in the shared gauge field, unreachable downward
Chebyshev capacity, activity/snapshot/binding infeasibility, and overclaimed
one-shot checks. Final review counts are Critical `0` / High `6`.

Q-010 therefore remains open only for a replacement decision that explicitly
permits persistent phase-volume state. Another stateless token, impulse,
matching substitution, radius change or post-result response curve is not
authorized. ADR-0008 remains Proposed / DESIGN BLOCKED and runtime remains not
started.

### 2026-08-21 D-021 disposition — persistent state authorized, question superseded

D-021 preserves the TE-5C result and explicitly permits one reciprocal
extent-link plus dedicated phase-pressure Current/Next pair. Q-010 is
superseded for current design selection by Q-011; neither the failed sharing
law nor its EMPTY vent rule is carried into the Water path.

## Q-011 · TE-5D persistent Vapor extent-pressure disposition — open 2026-08-21

Owner: user at the proposed ADR-0009 architecture-review boundary.

D-021 authorizes the persistent-state replacement design and one fixed-seed
grid/time/matching proof. User review remains required for the reciprocal
extent representation, exact link encoding, approved matching neighbourhood
and reassignment bound, owner movement/environment transaction, dedicated
phase-pressure response and relaxation, rupture stress combination, memory
cost and atomic G5 fixture.

Any unresolved Critical/High makes TE-5D **DESIGN BLOCKED**. The blocking
receipt must identify whether a repair needs wider matching scope, additional
full-world scratch, another persistent field, relaxation of 1:1 quantity or a
different volume representation. Runtime, background Air pressure, structure
differential, product edge mode and Vacuum combustion remain separate and not
started.

### 2026-08-21 proof and independent-review update — candidate blocked

The one-shot proof returned `DESIGN_BLOCKED` and did not execute the required
all-labeled 6×6 exhaustion. A canonical persistent eight-source alternating
chain has a complete matching but needs an augmenting path deeper than the
frozen six-source bound. Fresh review independently confirmed it and recorded
five more High findings: proof overclaim, receiver-blind target matching,
missing editor reciprocal cleanup, pass/binding/scratch understatement and
undefined phase-pressure movement ownership. Final counts are Critical `0` /
High `6` / Medium `2`.

Q-011 remains open for a user decision on wider matching scope and, if needed,
one full-world frontier/predecessor scratch with exact lifetime and bytes.
Another persistent field, a different volume representation and relaxation of
the 1:1 quantity contract are not required by the current counterexamples.

### 2026-08-21 D-022 disposition — comparative reset authorized, question superseded

D-022 preserves TE-5D's exact blocker and supersedes Q-011 for current model
selection. A larger fixed reassignment depth is forbidden. Wider exact
matching is now compared, without preference, against connected shared-chamber
capacity and a conservative Vapor-volume Environment scalar under Q-012.

## Q-012 · TE-5X pressure-volume model selection — open 2026-08-21

Owner: user at the proposed ADR-0010 model-selection boundary.

D-022 authorizes exactly three docs/reference candidates and one combined,
pre-registered execution. The unresolved user choice is whether any candidate
preserves the common phase quantity, finite-capacity, reversible-pressure,
Air/H hygiene, bounded-work and product-readability contract strongly enough
to become a future atomic TE-3/TE-5 implementation architecture.

The comparison must disclose exact matching convergence/scratch, chamber
connectivity and narrow-neck meaning, conservative-field source/sink and Air
coexistence, 2048² memory/pass/query/binding costs, prior-art identity/license,
proof limitations and fresh comparative-review findings. No candidate is
accepted in advance. Any unresolved Critical/High makes TE-5X **DESIGN
BLOCKED**; otherwise the maximum disposition is **ARCHITECTURE COMPARISON
COMPLETE / USER MODEL SELECTION PENDING**. Runtime remains not started.

### 2026-08-21 one-shot execution update — TE-5X blocked

The only combined process failed at the NetworkX oracle-version guard before
candidate evaluation. A/B/C cases, 50,000 generated states, 10,000 multi-tick
grids and the common fixture matrix all completed zero. The failure JSON parses
and is hashed, but explicitly records incomplete evidence and is not a proof
PASS. Repair/rerun is forbidden inside D-022's one-shot scope.

Q-012 therefore remains open for a new user decision: authorize a new evidence
identity and corrected oracle environment, revise the comparison scope, or
leave all three models unselected. No implementation and no fourth candidate
is authorized. ADR-0010 remains Proposed / DESIGN BLOCKED and runtime remains
not started.

Fresh review closes no model-selection item: Critical `0` / High `11` /
Medium `0`. A, B and C are all ineligible under the frozen criteria; there is
no Recommendation or Retained fallback. Q-012 stays open only for a new direct
user decision, not for continuation under the failed D-022 evidence identity.

### 2026-08-21 D-023 disposition — closed by ontology supersession

D-023 supplies the required new direct decision. It leaves all TE-5X evidence
frozen, accepts no A/B/C candidate, and supersedes the whole-Cell quantity
constraint instead of continuing the failed comparison. Q-012 is closed for
current work; TE-5X remains DESIGN BLOCKED with no recommendation.

## Q-013 · Conservative phase-packet architecture review — open 2026-08-21

Owner: user at the proposed ADR-0011 architecture-review boundary.

D-023 authorizes `TE3Q-PHASE-PACKETS-REFERENCE-V1` and a candidate in which
explicit half-unit Steam packets conserve phase quantity while actual Matter
spawn/merge supplies expansion and contraction. User review must decide
whether half-packet foreground occupancy is acceptable, whether a lone
condensation-ready one-unit Steam packet may remain metastable, which local
merge neighbourhood/order is acceptable, whether the predeclared
phase-pressure coefficients and rupture timing preserve product meaning, and
whether the projected 96 MiB state increment plus future pass/binding cost is
acceptable.

The decision also remains open on runtime-only evidence: quantity-aware
Environment receiver behavior, split/merge collision and movement ownership,
activity/sleep equivalence, readback/editor hygiene, GPU feasibility,
performance, visuals and direct user acceptance. ADR-0011 must remain
**PROPOSED — USER ARCHITECTURE REVIEW PENDING**. Any unresolved Critical/High
finding stops **TE-3Q / TE-5Q DESIGN BLOCKED** and runtime remains not started.

### 2026-08-21 proof and independent-review update — candidate blocked

The one-shot process returned mathematical PASS, but fresh review found
Critical `0` / High `8` / Medium `1`. Named cold-lid/beaker/chunk/sleep/editor
fixtures were reduced or constant, local greedy merge can strand pairable
Steam/1 packets, and the spatial pressure law can reset on Steam/2 movement or
remain at Wood threshold after its source disappears. Q-013 remains open only
for a new direct user decision on architecture revision and a new evidence
identity. ADR-0011 remains Proposed / DESIGN BLOCKED; runtime is not started.
