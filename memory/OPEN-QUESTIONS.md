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

Known state: The first non-evidence G8-C pilot failed before any official matrix, Receipt, package, or independent-verifier result existed. Therefore there is no verified matrix recommendation yet.

Next check: After Q-006 is resolved and a verified official matrix exists, refresh `memory/CHECKPOINT.md` from exact live Git and evidence, then ask the user for the next explicit product/optimization decision. Do not auto-start G9 or optimization.

Status: open

## Q-005 · When is the Ballast cutover integrated into the active product line? — opened 2026-08-19

Owner: project operator

Blocks: Repository-wide enforcement of the active memory workflow in future Powdergame sessions.

Known state: The user approved adoption. PR #4 contains the isolated pilot and active cutover, but commit-preserving integration is deferred while the G8-C writer owns a dirty staged worktree. Squash is forbidden.

Next check: After the G8-C writer reaches a clean safe stop, exact-fetch the active line, refresh the checkpoint, reconcile the PR against the live history, and integrate PR #4 using a commit-preserving method.

Status: open

## Q-006 · Is one G8-C replacement pilot and conditional official capture authorized? — opened 2026-08-19

Owner: user

Blocks: G8-C official matrix, independent verification, and any later performance/product decision.

Known evidence:

- G8-B is `CLOSED / FROZEN` at closure commit `18391e6a9fc8f9bc7b2757f3504366f106c05435`.
- G8-C branch is `feature/m0-g8c-official-matrix` at clean upstream base `8ee1ae238c324c1db1d7e2882af071fec179a8f1`, with intended 14-file staged work and no sealed G8-C source commit.
- Pilot `g8c-pilot-8ee1ae238c32-c64090539536` completed all five headless Mode A/B subprocesses but failed the first Sand Fall Mode C process.
- The renderer had already confirmed live 1600×900. A late initial `Resized(2864×1560)` payload was misclassified as a real post-init resize.
- No official capture, Receipt, hash inventory, package, matrix report, or verifier result exists.

Proposed bounded authorization:

1. preserve the failed pilot and staged work;
2. distinguish stale resize payload from the current live `window.inner_size()` without allowing arbitrary sizes;
3. run targeted lifecycle regressions;
4. run exactly one replacement non-evidence pilot;
5. only if it passes, seal/commit/push source and run exactly one official capture, one independent verification, and one package;
6. stop without G9 or optimization.

Status: open