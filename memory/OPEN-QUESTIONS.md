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

Current disposition: no item blocks the TE-2 user-review candidate. Product
edge mode, Vacuum combustion, latent phase/yield and optional GAS permeability
remain open at their named later gates. The implemented sealed correctness edge
and explicit fixture reservoir do not close the product edge choice.

## Current pending user-owned choices

No product-scope question blocks G9-A continuity re-review or TE-1 entry.

Future user decisions occur at the next evidence boundary:

- accept, revise or reject the G9-A user-testable editor/sandbox candidate;
- close Q-008 items only at their named Thermal Environment gate;
- authorize later G9-B/C/D progression after that candidate;
- approve shared `main` promotion;
- give the final G9/M0 product disposition after direct play.
