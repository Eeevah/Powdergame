# Checkpoint — active Ballast cutover approved — 2026-08-19 KST

## Validity

This checkpoint records the approved memory cutover while the G8-C Official Matrix task is already running in another active development context. Do not interrupt, restart, or reinterpret that in-flight task from this file. Before any new Powdergame work after G8-C finishes, refresh this checkpoint from the final report and exact live Git coordinates.

## Repository coordinate

- Memory branch: `agent/ballast-memory-pilot`
- Initial pilot commit: `ba2b6406f6605882c51886b0a50bc64d10990a7f`
- Existing Draft PR: `#4`
- Target line: `feature/m0-g8b-scenario-suite`, followed by the separately authorized G8-C line
- Cutover state: user approved Ballast as the single active session-continuity workflow; commit-preserving integration is intentionally deferred until the current G8-C writer finishes so the branch is not raced
- Live source rule: exact-fetch the active branch and verify its full SHA before relying on any coordinate in memory

## Story so far

The isolated six-file pilot passed its docs-only audit and was useful enough for the user to adopt. Ballast now replaces the old per-session HANDOFF maintenance pattern, not the project's domain documents. `memory/CHECKPOINT.md` becomes the one current resume point; decisions and open questions move through their append-only ledgers; canonical evidence, milestone, architecture, validation, and status documents remain authoritative in their own domains.

The user accepted Heavy Mixed World with a known non-blocking broad terminal Thermal tail. The immutable candidate remains automatic `NEEDS_HUMAN_REVIEW`, all 14 hard predicates remain pass, `candidate_blocker=false`, and no production-physics defect was established. The user then authorized G8-C Official Matrix work. That task is currently running; no G9 or optimization decision has been authorized.

## Valid evidence

- Heavy Mixed immutable candidate:
  - source `07260fffab22e5b4513eb168f0baac36e374ab94`
  - run `g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4`
  - binary SHA-256 `9b84db005942cf60ae9ef133521e9297413d49c93d72e7ae64133e29622f7583`
  - Receipt SHA-256 `2abebdef7f9174e63abfd9c67ce4a48d24b48edde4e6c29fab49022e36a2dbd1`
  - Audit Bundle SHA-256 `bc44c66bd52b5d856decb2317389a455a56ac8ae1f8d67b1bfeb5446cfb5731b`
  - 14/14 hard predicates pass; `candidate_blocker=false`; automatic `NEEDS_HUMAN_REVIEW` only for declining `broad_terminal_tail`
  - human disposition: `USER ACCEPTED WITH KNOWN FOLLOW-UP`
- Earlier accepted Sand, Water, Fire, Pressure, and Cell Inspector results remain valid only under their exact canonical evidence/receipt contracts. Open the linked evidence before reuse; this checkpoint does not reproduce every toolchain/config key.
- Docs-only memory changes do not invalidate those exact historical runs and do not authorize reruns.

## Decided

- D-003 — Heavy Mixed World is user accepted with known follow-up; automatic verdict and immutable evidence remain unchanged.
- D-004 — Ballast is adopted as Powdergame's single active session-continuity workflow, superseding the isolated-pilot operating model.
- D-005 — The cutover is reversible: immediate Hook disable first, then revert the active cutover commit and the pilot commit in reverse order; squash merge is forbidden because it destroys the rollback boundary.

## Waiting on the user or in-flight work

- Q-002 — Same-SHA G8-A user visual validation remains pending.
- Q-004 — G8-C final matrix result and the subsequent choice among G9, optimization review, or further human review.
- Integration action — after G8-C completes, refresh the exact branch/SHA and integrate PR #4 without squashing.

## Next first action

Do not start a new task and do not disturb the running G8-C matrix. When its final report arrives:

1. exact-fetch the live G8-C and G8-B branches;
2. record the final source, matrix ID, receipt/package identities, recommendation, and remaining gate items;
3. refresh this checkpoint;
4. integrate PR #4 with commit boundaries preserved;
5. make the next product/optimization decision only from the verified G8-C report.

## Tried / avoid repeating

- Do not maintain `docs/HANDOFF.md` and this checkpoint as two live session coordinates.
- Do not treat local ahead/behind `0/0` as live remote equality without an exact fetch.
- Do not rerun valid FULL/GPU/candidates merely because memory or docs changed.
- Do not record agent proposals as user decisions.
- Do not merge PR #4 with squash.
- If Hook injection misbehaves, set `BALLAST_DISABLE=1` or remove Hook trust before changing Git.