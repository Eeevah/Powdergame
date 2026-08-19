# Open questions

Only unresolved questions active in the latest authoritative status or checkpoint belong here. Close an item by appending a dated disposition and a durable decision/evidence link; do not delete history.

## Q-001 · Does the user accept the immutable Heavy Mixed World candidate? — opened 2026-08-19

Owner: user

Original block: G8-B closure and any G8-C start.

Known evidence: [`docs/evidence/G8_B_HEAVY_MIXED_WORLD_HARNESS_CANDIDATE_2026-08-19.md`](../docs/evidence/G8_B_HEAVY_MIXED_WORLD_HARNESS_CANDIDATE_2026-08-19.md); source `07260fffab22e5b4513eb168f0baac36e374ab94`; Run ID `g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4`; Receipt SHA-256 `2abebdef7f9174e63abfd9c67ce4a48d24b48edde4e6c29fab49022e36a2dbd1`.

### Disposition — closed 2026-08-19

The user accepted Heavy Mixed World with known follow-up. Automatic `NEEDS_HUMAN_REVIEW`, 14/14 hard PASS, `candidate_blocker=false`, immutable artifacts, and the declining non-blocking `broad_terminal_tail` remain unchanged. See D-003.

Status: closed

## Q-002 · Is same-SHA G8-A user visual validation approved? — opened 2026-08-19

Owner: user

Blocks: Final user disposition of the G8-A verified evidence candidate.

Known evidence: [`docs/planning/STATUS.md`](../docs/planning/STATUS.md); [`docs/evidence/G8_A_MEASUREMENT_SUBSTRATE_2026-08-17.md`](../docs/evidence/G8_A_MEASUREMENT_SUBSTRATE_2026-08-17.md); source `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`; capture `g8a-v5-9abec9e-20260817T032827206Z`.

Next check: Obtain an explicit user disposition for the already captured same-SHA visual evidence; do not infer it from technical verification or later G8-C results.

Status: open

## Q-003 · Which gate or publication action is explicitly authorized after Heavy acceptance? — opened 2026-08-19

Owner: user

Original block: Explicit G8-B closure, G8-C, G9, optimization, and shared `main` promotion decisions.

### Disposition — closed 2026-08-19

The user authorized G8-C Official Performance Matrix. This authorization did not extend to G9, optimization implementation, G8 overall closure, or `main` promotion.

Status: closed

## Q-004 · What should follow the verified G8-C matrix? — opened 2026-08-19

Owner: user

Blocks: G9 start, any optimization implementation, G8 overall closure, and later publication choices.

Known evidence:

- sealed source `4653d7c2e09e93f80fb81eeb73458d992c86858f`
- Matrix ID `g8c-official-matrix-4653d7c2e09e-64df60ba0d79`
- Receipt SHA-256 `1fbf4599893cc29e99b6033996b42fcdf025aac0b421cb80b95b3e55807455f6`
- package SHA-256 `92f8b85cc0e34ea6e71a9f6b4fc95b0f70704263a0f798a69a830cce1d40b729`
- independent verification SHA-256 `77c7e1c982296277c451de02c3dca68fa6d7d9a90e9fd5426c4dffa1abd9bb0d`
- verifier result `verified=true`, 230 recomputed matrix fields, mismatch `0`
- matrix recommendation `PROCEED_TO_G9`

The verified matrix found no current 60-TPS simulation, coexistence, rendering, or memory blocker. The recommendation is evidence, not automatic user authorization.

Next check: Ask the user to define and authorize the G9 product brief, or to request a narrower review/override. Do not auto-start G9 or optimization.

Status: open

## Q-005 · When is the Ballast cutover integrated into the active product line? — opened 2026-08-19

Owner: project operator

Blocks: Repository-wide enforcement of the active memory workflow in future Powdergame sessions.

Known state: G8-C reached a clean upstream-equal safe stop at `4653d7c2e09e93f80fb81eeb73458d992c86858f`. PR #4 contains the approved Ballast cutover and current final-matrix checkpoint. Squash is forbidden.

Next check: Retarget PR #4 to `feature/m0-g8c-official-matrix`, confirm CI/mergeability, integrate with commit boundaries preserved, and record the exact merge-based rollback.

Status: open

## Q-006 · Is one G8-C replacement pilot and conditional official capture authorized? — opened 2026-08-19

Owner: user

Original block: G8-C official matrix, independent verification, and any later performance/product decision.

### Disposition — closed 2026-08-19

The user authorized the narrow window-lifecycle remediation, one replacement non-evidence pilot, and conditional official capture. The remediation succeeded and all 15 replacement-pilot measurement subprocesses exited `0`, but final aggregation stopped on the historical CSV vocabulary mismatch. No official capture was run under this authorization.

Status: closed

## Q-007 · Is the narrow historical-CSV adapter fix and conditional official capture authorized? — opened 2026-08-19

Owner: user

Original block: G8-C official matrix, independent verification, and any later performance/product decision.

### Disposition — closed 2026-08-19

The user authorized the strict adapter correction, one aggregation-only replay over hash-bound replacement-pilot raw outputs, and—only if the replay passed—one official matrix capture, independent verification, and package.

Outcome:

- canonical producer vocabulary `wall_per_tick` + `ms/tick` was preserved and mapped explicitly to internal `wall_ms_per_tick`
- actual-producer-schema and missing/duplicate/wrong-unit/alias regressions were added
- aggregation replay `g8c-aggregation-replay-20260819T015515996891Z-fc408076b67a` passed while launching zero executable/GPU/measurement subprocesses and remained `non_evidence=true`
- sealed source `4653d7c2e09e93f80fb81eeb73458d992c86858f` was committed and pushed clean/upstream-equal
- official Matrix `g8c-official-matrix-4653d7c2e09e-64df60ba0d79` ran exactly once
- independent verification passed with 230 fields recomputed and mismatch `0`
- final recommendation: `PROCEED_TO_G9`

Status: closed
