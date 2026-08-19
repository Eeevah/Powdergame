# Open questions

This is an append-only register of user-owned pending choices and dated dispositions. It is not proof of runtime state; open the linked canonical evidence for exact claims.

## Q-001 · Heavy Mixed user acceptance — closed 2026-08-19

Disposition: `USER ACCEPTED WITH KNOWN FOLLOW-UP`. Automatic `NEEDS_HUMAN_REVIEW`, 14/14 hard PASS, `candidate_blocker=false`, immutable artifacts and declining `broad_terminal_tail` remain unchanged. See D-003 and the Heavy evidence record.

## Q-002 · G8-A same-SHA visual durable disposition — open

Owner: user

Known evidence:

- source `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`
- capture `g8a-v5-9abec9e-20260817T032827206Z`
- official capture and independent verification complete

Decision required: approve, explicitly defer, or formally supersede the separate same-SHA visual-disposition requirement. Do not infer this decision from G8-C.

## Q-003 · Authorize G8-C after Heavy acceptance — closed 2026-08-19

Disposition: user authorized G8-C Official Performance Matrix only. This did not authorize G9, optimization, G8 closure or `main` promotion.

## Q-004 · What follows the verified G8-C Matrix? — open

Owner: user

Known official evidence:

- sealed source `4653d7c2e09e93f80fb81eeb73458d992c86858f`
- Matrix `g8c-official-matrix-4653d7c2e09e-64df60ba0d79`
- Receipt `1fbf4599893cc29e99b6033996b42fcdf025aac0b421cb80b95b3e55807455f6`
- package `92f8b85cc0e34ea6e71a9f6b4fc95b0f70704263a0f798a69a830cce1d40b729`
- verification `77c7e1c982296277c451de02c3dca68fa6d7d9a90e9fd5426c4dffa1abd9bb0d`
- verifier: 230 recomputed fields, mismatch `0`
- recommendation: `PROCEED_TO_G9`

Decision required: approve a G9 product brief, request a narrower human review, or override the recommendation. The evidence found no current simulation/rendering/coexistence/persistent-memory blocker for the 60-TPS M0 target, but it cannot make the product decision for the user.

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
