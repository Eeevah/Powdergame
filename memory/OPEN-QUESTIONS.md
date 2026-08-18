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

The user authorized G8-C Official Performance Matrix and that task is currently running. This authorization does not extend to G9, optimization implementation, G8 overall closure, or `main` promotion.

Status: closed

## Q-004 · What should follow the verified G8-C matrix? — opened 2026-08-19

Owner: user

Blocks: G9 start, optimization review/implementation, G8 closure, and later publication choices.

Known inputs: The in-flight G8-C task is required to report one of `PROCEED_TO_G9`, `OPTIMIZATION_REVIEW_REQUIRED`, or `NEEDS_HUMAN_REVIEW`, together with source, matrix, receipt/package, and independent-verifier identities.

Next check: When the final G8-C report arrives, refresh `memory/CHECKPOINT.md` from exact live Git and evidence, then ask the user for the next explicit product/optimization decision. Do not auto-start G9 or optimization.

Status: open

## Q-005 · When is the Ballast cutover integrated into the active product line? — opened 2026-08-19

Owner: project operator

Blocks: Repository-wide enforcement of the active memory workflow in future Powdergame sessions.

Known state: The user approved adoption. PR #4 contains the isolated pilot and active cutover, but commit-preserving integration is deferred while the G8-C writer is active so the target branch is not raced.

Next check: After G8-C finishes, exact-fetch the active line, refresh the checkpoint, and integrate PR #4 using rebase-and-merge or a merge commit. Squash is forbidden.

Status: open