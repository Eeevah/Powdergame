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

## Q-004 · What should follow a verified G8-C matrix? — opened 2026-08-19

Owner: user

Blocks: G9 start, optimization review/implementation, G8 closure, and later publication choices.

Known state: Both G8-C non-evidence pilots stopped before any official matrix, final Receipt, official package, or independent official verifier result existed. Therefore there is no verified matrix recommendation yet.

Next check: After Q-007 is resolved and a verified official matrix exists, refresh `memory/CHECKPOINT.md` from exact live Git and evidence, then ask the user for the next explicit product/optimization decision. Do not auto-start G9 or optimization.

Status: open

## Q-005 · When is the Ballast cutover integrated into the active product line? — opened 2026-08-19

Owner: project operator

Blocks: Repository-wide enforcement of the active memory workflow in future Powdergame sessions.

Known state: The user approved adoption. PR #4 contains the isolated pilot and active cutover, but commit-preserving integration is deferred while the G8-C writer owns a dirty staged/unstaged worktree. Squash is forbidden.

Next check: After the G8-C writer reaches a clean safe stop, exact-fetch the active line, refresh the checkpoint, reconcile the PR against the live history, and integrate PR #4 using a commit-preserving method.

Status: open

## Q-006 · Is one G8-C replacement pilot and conditional official capture authorized? — opened 2026-08-19

Owner: user

Original block: G8-C official matrix, independent verification, and any later performance/product decision.

### Disposition — closed 2026-08-19

The user authorized the narrow window-lifecycle remediation, one replacement non-evidence pilot, and conditional official capture. The remediation succeeded: all ten Mode C/D workers stayed live at 1600×900, ten stale 2864×1560 event payloads were ignored safely, and fatal live resize/surface/device errors were zero. The one replacement pilot ran all 15 measurement subprocesses successfully but failed final aggregation because the coordinator searched for `wall_ms_per_tick` while the historical producer emits metric name `wall_per_tick` with unit `ms/tick`. No official capture was run. The bounded authorization was consumed without producing official evidence.

Status: closed

## Q-007 · Is the narrow historical-CSV adapter fix and conditional official capture authorized? — opened 2026-08-19

Owner: user

Blocks: G8-C official matrix, independent verification, and any later performance/product decision.

Known evidence:

- G8-C remains on `feature/m0-g8c-official-matrix` at upstream-equal HEAD `8ee1ae238c324c1db1d7e2882af071fec179a8f1`; no sealed G8-C source commit exists.
- The original 14 intended paths remain staged with patch SHA-256 `eba224c3f39c2a0a40fc47be46bdc5a7863a6062027b1952bbb114535c1d6733`.
- Five lifecycle/coordinator remediation files remain unstaged with patch SHA-256 `069338e8922c4b717ead60fb5fdabef0a0ac93739c064e60faa74c56443d2150`.
- Replacement pilot `g8c-pilot-8ee1ae238c32-6341f4f59218` completed all five Headless A/B, five Mode C, and five Mode D subprocesses with exit `0`.
- The historical benchmark summary producer emits throughput metric `wall_per_tick` and unit `ms/tick`; the new coordinator expected internal name `wall_ms_per_tick` and stopped with `headless summary is incomplete`.
- No official Matrix ID, final Receipt, official package, official binary hashes, or independent official verification exists.

Proposed bounded authorization:

1. preserve both failed pilots and the staged/unstaged worktree state;
2. map raw `wall_per_tick` with exact unit `ms/tick` to internal `wall_ms_per_tick` without changing historical CSV vocabulary;
3. add actual-producer-schema plus missing/duplicate/wrong-unit regressions;
4. run one aggregation-only scratch replay over copied and hash-bound raw outputs from the completed replacement pilot, exercising report/Receipt/package/verifier without rerunning GPU measurement subprocesses;
5. only if that replay passes, seal/commit/push source and run exactly one official matrix capture, one independent verification, and one package;
6. stop without G9 or optimization; any replay or official failure requires a new explicit decision.

Status: open
