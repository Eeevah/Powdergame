# Checkpoint — G8-C lifecycle fixed; aggregation blocked by historical CSV vocabulary — 2026-08-19 KST

## Validity

This checkpoint records the completed replacement G8-C pilot attempt. It is a session-resume coordinate, not official performance evidence. The active G8-C writer worktree remains intentionally dirty with reviewed staged and unstaged changes; no other session may reset, unstage, stash, clean, overwrite, or independently continue it.

PR #4 remains Draft/Open/Unmerged. Its memory branch may record this checkpoint, but it must not be integrated into the active product line until the G8-C writer reaches a clean safe stop and the live history is reconciled.

## Repository coordinate

- G8-B closure commit: `18391e6a9fc8f9bc7b2757f3504366f106c05435`
- Legacy launcher retirement commit: `8ee1ae238c324c1db1d7e2882af071fec179a8f1`
- G8-B state: **CLOSED / FROZEN**
- Active G8-C branch: `feature/m0-g8c-official-matrix`
- Active G8-C HEAD / upstream: `8ee1ae238c324c1db1d7e2882af071fec179a8f1` / identical, divergence `0/0`
- G8-C source commit: none; aggregation failure stopped source-seal/commit/push
- Active writer state: original 14 intended paths remain staged; five lifecycle/coordinator remediation files remain unstaged; untracked files `0`
- Original staged patch SHA-256: `eba224c3f39c2a0a40fc47be46bdc5a7863a6062027b1952bbb114535c1d6733`, unchanged after remediation
- Remediation unstaged patch SHA-256: `069338e8922c4b717ead60fb5fdabef0a0ac93739c064e60faa74c56443d2150`
- Worktree count: `3`
- Target cache: preserved; `cargo clean` was not run

## Story so far

The window lifecycle remediation succeeded. `window.inner_size()` is now the final authority: a stale noncanonical event payload is ignored only while the live size remains exactly 1600×900; a genuinely noncanonical or zero live size remains fatal. `Resized` and `ScaleFactorChanged` share the same helper, renderer resize is not invoked, and Mode C/D record structured lifecycle metadata.

Targeted validation passed: Windows lifecycle tests `15/15`, renderer lifecycle/timestamp tests `3/3`, affected checks/clippy, coordinator/verifier tests `47/47`, legacy benchmark verifier fixtures, policy audit, formatting, and staged/unstaged diff checks. Workspace FULL, Gallery smoke, and G8-B candidate reruns remained `0`.

The replacement non-evidence pilot was executed exactly once:

- Pilot ID: `g8c-pilot-8ee1ae238c32-6341f4f59218`
- Isolated build: PASS
- Headless Mode A/B: five scenarios, all exit `0`
- Mode C: five scenarios, each 60 frames, all exit `0`
- Mode D: five scenarios, each 16 frames, all exit `0`
- Lifecycle: all 10 workers initial/last live size 1600×900; one stale 2864×1560 payload per worker, 10 total, safely ignored; fatal live resize `0`; surface/device error `0/0`
- Final aggregation: FAIL
- Official capture / verification / package: `0 / 0 / 0`

The remaining blocker is a coordinator adapter mismatch, not a GPU, renderer, window-lifecycle, fixture, or performance failure. The historical benchmark summary vocabulary emits throughput metric name `wall_per_tick` with unit `ms/tick`; the new coordinator searched for an internal field named `wall_ms_per_tick`. Complete raw measurement files therefore existed, but the coordinator produced an empty headless summary and stopped with `headless summary is incomplete`.

Historical G8-A/G8-B CSV vocabulary must remain unchanged. The narrow correction is to map raw `wall_per_tick` plus strict `ms/tick` unit validation into the internal aggregate field `wall_ms_per_tick`, and to add an actual-schema regression based on the real producer vocabulary.

## Evidence boundary

Original failed pilot:

- ID: `g8c-pilot-8ee1ae238c32-c64090539536`
- Path: `C:\Users\mdkap\source\Powdergame-artifacts\scratch\g8c-pilot-8ee1ae238c32-c64090539536`
- Files / bytes: `53 / 56,911,997`
- Failure: first Sand Fall Mode C lifecycle guard

Replacement failed pilot:

- ID: `g8c-pilot-8ee1ae238c32-6341f4f59218`
- Files / bytes: `98 / 57,021,663`
- All 15 measurement subprocesses exited `0`
- Failure: coordinator CSV metric-name mismatch during final aggregation
- Pilot-only benchmark binary SHA-256: `991c0cca831ab14d3ba47b3b03151ff782541e9a72521a33b2e1984090ec3f64`
- Pilot-only Windows binary SHA-256: `4f164060f198fca6c644a7252d44aab19b61c0de18b6464b144dbc528f9221ad`

Neither pilot has official Matrix identity, official frozen-binary identity, final Receipt, official package, or independent official verification. Mode C/D raw rows are diagnostic only and must not be reported as official performance or bottleneck conclusions.

The task timer's `PASS` means only that UTC/monotonic timing accounting closed consistently. The G8-C task remains `NEEDS_HUMAN_REVIEW — measurement aggregation integrity blocker`.

## Decided

- D-003 — Heavy Mixed World is user accepted with known follow-up; its immutable automatic verdict and evidence remain unchanged.
- D-004 — Ballast is the approved single active Powdergame session-continuity workflow after commit-preserving integration.
- D-005 — Ballast remains selectively reversible; squash merge is forbidden.

## Waiting on the user or in-flight state

- Q-007 — Whether to authorize the narrow historical-CSV adapter correction, one aggregation-only replay over immutable replacement-pilot raw outputs, and—only if that replay passes—one official matrix capture and independent verification.
- Q-002 — Same-SHA G8-A user visual validation remains pending.
- Q-004 — The post-matrix choice among G9, optimization review, or further human review remains blocked until a verified official matrix exists.
- Q-005 — PR #4 integration remains deferred until the active G8-C writer reaches a clean safe stop and the exact live history/checkpoint is refreshed.

## Next first action

Do not start another task and do not disturb the staged/unstaged G8-C worktree.

If the user explicitly authorizes the narrow continuation:

1. continue in the same G8-C writer session and worktree;
2. preserve both failed pilot directories and the staged/unstaged patch identities;
3. change only the coordinator/verifier adapter so raw `wall_per_tick` with exact unit `ms/tick` maps to internal `wall_ms_per_tick`; do not rename or rewrite historical CSV output;
4. add actual-producer-schema and wrong-unit/missing/duplicate metric regressions;
5. exercise downstream aggregation, report, Receipt, package, and verifier with one new aggregation-only scratch replay over copied and hash-bound raw outputs from the completed replacement pilot; do not rerun GPU measurement subprocesses for this adapter check;
6. if and only if the replay passes, seal/commit/push the clean source and run the official matrix exactly once, followed by independent verification and one package;
7. stop without starting G9 or optimization.

If the aggregation-only replay fails, preserve it, do not run official capture, and return a blocker report. A further pilot or official retry requires a new explicit user decision.

## Tried / avoid repeating

- Window lifecycle remediation is proven by all ten Mode C/D workers; do not redesign window sizing again for this blocker.
- Do not change the historical producer metric name `wall_per_tick` or its unit `ms/tick` merely to fit the new coordinator.
- Do not treat complete raw rows as official matrix evidence while aggregation, Receipt, package, and independent verification are incomplete.
- Do not rerun all 15 pilot measurement subprocesses when the current defect is a pure aggregation adapter mismatch; prefer an aggregation-only replay.
- Do not reset, unstage, stash, clean, or recreate the active worktree.
- Do not interpret task-timer `PASS` as G8-C task success.
- Do not merge or squash PR #4 while the G8-C writer is dirty.
- If Ballast Hook injection misbehaves, set `BALLAST_DISABLE=1` or remove Hook trust before changing Git.
