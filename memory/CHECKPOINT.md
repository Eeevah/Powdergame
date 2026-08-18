# Checkpoint — G8-C pilot stopped at Mode C lifecycle guard — 2026-08-19 KST

## Validity

This checkpoint records the first G8-C pilot outcome. It is a session-resume coordinate, not official performance evidence. The active G8-C writer worktree is intentionally dirty with reviewed staged changes; no other session may reset, stash, clean, overwrite, or independently continue that worktree.

PR #4 remains Draft/Open/Unmerged. Its memory branch may record this checkpoint, but it must not be integrated into the active product line until the G8-C writer reaches a safe stop and the live history is reconciled.

## Repository coordinate

- G8-B closure commit: `18391e6a9fc8f9bc7b2757f3504366f106c05435`
- Legacy launcher retirement commit: `8ee1ae238c324c1db1d7e2882af071fec179a8f1`
- G8-B state: **CLOSED / FROZEN**
- Active G8-C branch: `feature/m0-g8c-official-matrix`
- Active G8-C HEAD / upstream: `8ee1ae238c324c1db1d7e2882af071fec179a8f1` / identical, divergence `0/0`
- G8-C source commit: none; pilot failure stopped source-seal/commit/push
- Active writer state: intended 14 files staged, worktree dirty, forbidden-path changes `0`
- Worktree count: `3`
- Target cache: preserved; `cargo clean` was not run

## Story so far

G8-B is closed and frozen. The G8-C implementation added windowed coexistence measurement, render timestamp profiling, matrix coordination, source/binary sealing, receipt/package publication, and an independent verifier while preserving the existing G8-A Mode A/B contracts and historical schemas.

Targeted validation passed: Windows G8-C tests `9/9`, renderer timestamp/surface tests `3/3`, benchmark tests `27/27`, coordinator/verifier regression `43/43`, the legacy benchmark verifier fixture, affected checks/clippy, policy audit, formatting, and diff checks. Workspace FULL was not run because it was not required by the changed paths.

Exactly one allowed non-evidence pilot was started:

- Pilot ID: `g8c-pilot-8ee1ae238c32-c64090539536`
- Headless Mode A/B: all five scenarios exited `0`
- First Mode C Sand Fall process: exit `1`
- Remaining Mode C and all Mode D runs: not started
- Official capture: `0`
- Receipt, hash inventory, package, report, verifier result: not created

The failure is a window-lifecycle measurement guard issue, not a performance result. Renderer initialization verified the actual 1600×900 surface, but a late initial `Resized(2864×1560)` payload was treated as a real post-init resize and caused a fatal exit. The proposed narrow remediation is to re-read `window.inner_size()` when handling the event: ignore a stale payload only when the live size is still exactly 1600×900; retain a fatal error when the live size is genuinely noncanonical. Arbitrary resolutions and renderer resize are still forbidden.

## Evidence boundary

The failed pilot directory is incomplete scratch evidence only:

- Path: `C:\Users\mdkap\source\Powdergame-artifacts\scratch\g8c-pilot-8ee1ae238c32-c64090539536`
- Files / bytes: `53 / 56,911,997`
- No final Receipt, `HASHES.sha256`, package, matrix report, or independent-verifier result exists
- Pilot-only benchmark binary SHA-256: `cb9f8ba98e2fdfe7f38b56b3e1f3e6c0a8d1c15361293b7484f645307a9d2161`
- Pilot-only Windows binary SHA-256: `2eeda613c26c66e10b247700c0569d269e49507506998edbdeb9caef5364e09f`

These hashes and the successful headless subprocesses are diagnostic facts only. They are not official G8-C evidence, do not support a bottleneck decision, and do not authorize G9 or optimization.

The task timer's `PASS` means only that UTC/monotonic timing accounting closed consistently. The G8-C task itself stopped as `NEEDS_HUMAN_REVIEW — measurement integrity incomplete`.

## Decided

- D-003 — Heavy Mixed World is user accepted with known follow-up; its immutable automatic verdict and evidence remain unchanged.
- D-004 — Ballast is the approved single active Powdergame session-continuity workflow after commit-preserving integration.
- D-005 — Ballast remains selectively reversible; squash merge is forbidden.

## Waiting on the user or in-flight state

- Q-006 — Whether to authorize the narrow stale-resize remediation, one replacement non-evidence pilot, and—only if that pilot passes—one official matrix capture and independent verification.
- Q-002 — Same-SHA G8-A user visual validation remains pending.
- Q-004 — The post-matrix choice among G9, optimization review, or further human review remains blocked until a verified official matrix exists.
- Q-005 — PR #4 integration remains deferred until the active G8-C writer reaches a clean safe stop and the exact live history/checkpoint is refreshed.

## Next first action

Do not start another task and do not disturb the staged G8-C worktree.

If the user explicitly authorizes replacement measurement:

1. continue in the same G8-C writer session and worktree;
2. preserve the failed pilot directory and all staged work;
3. implement only the stale resize-payload/live-size distinction and its targeted regressions;
4. run exactly one replacement non-evidence pilot;
5. if and only if it passes, seal/commit/push the clean source and run the official matrix exactly once, followed by independent verification and one package;
6. stop and report without starting G9 or optimization.

If the replacement pilot fails, preserve it, do not run official capture, and return a blocker report.

## Tried / avoid repeating

- Do not treat the failed pilot as a performance measurement.
- Do not rerun the same pilot without explicit user authorization.
- Do not loosen the canonical 1600×900 requirement or accept arbitrary window sizes merely to make the pilot pass.
- Do not reset, unstage, stash, clean, or recreate the active worktree.
- Do not interpret task-timer `PASS` as G8-C task success.
- Do not merge or squash PR #4 while the G8-C writer is dirty.
- If Ballast Hook injection misbehaves, set `BALLAST_DISABLE=1` or remove Hook trust before changing Git.