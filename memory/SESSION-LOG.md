# Session log

This is a terse append-only audit trail, not a transcript.

## 2026-08-19 · Initial isolated Ballast pilot

- Wiki/profile: synchronized `personal-infra-wiki` at `318276eebfbf913638d72f5d218ead2450361a01`; local `HEAD == origin/main`; `install-ballast-codex.ps1 -Verify` passed; no reinstall was performed.
- Repository coordinate: exact fetch of `origin/feature/m0-g8b-scenario-suite` produced Base SHA `e43078737712862c9cc6ccdc4b7e56475bafc6ce`; isolated branch `agent/ballast-memory-pilot` was created from that commit.
- Isolation: the existing `Powdergame-g8b` worktree was observed at `feature/m0-g8b-scenario-suite` / `e43078737712862c9cc6ccdc4b7e56475bafc6ce` with an empty `git status --short`; only permitted Git metadata was read, and no uncommitted contents were inspected or copied.
- Initialized: `AGENTS.md`, `memory/00-INDEX.md`, `memory/DECISIONS.md`, `memory/OPEN-QUESTIONS.md`, `memory/SESSION-LOG.md`, and `memory/CHECKPOINT.md`; no other project file was changed.
- Validation: six-path allowlist passed; 21 intended targets and 28 relative Markdown links resolved; stale-status/SHA review passed; guarded `pwsh -NoProfile -File tools/dev.ps1 audit` returned PASS/exit 0 with only the two declared legacy-launcher warnings; `git diff --check` passed. No repository-owned link/stale checker exists, so those two checks were targeted manual reviews.
- Publication outcome: commit `ba2b6406f6605882c51886b0a50bc64d10990a7f`, Draft PR #4 against `feature/m0-g8b-scenario-suite`, left unmerged for user evaluation.
- Runtime: no Rust/GPU FULL, application execution, smoke, experiment/fixture candidate, official capture, performance run, or user acceptance was executed.

## 2026-08-19 · User adoption and reversible cutover staged

- User decision: Heavy Mixed accepted with known follow-up; Ballast adopted as Powdergame's single active session-continuity workflow with selective rollback.
- Memory commit: `8d21756f3dfa5c6a743f0aa03108153bb4b206df` (`docs: adopt Ballast as primary project memory`).
- PR #4 was retitled and rewritten as a reversible cutover; squash merge forbidden; initial pilot and active cutover commits preserved separately.
- Wiki decision/workflow/troubleshooting were merged through personal-infra-wiki PR #39 after CI PASS. Immediate disable is `BALLAST_DISABLE=1` or Hook untrust; project rollback reverts the active cutover commit then `ba2b640...`.
- Integration into the product line was intentionally deferred while the separately authorized G8-C writer was active.
- Runtime: no Powdergame build, test, smoke, candidate, capture, or evidence mutation was performed by the memory cutover work.

## 2026-08-19 · G8-C first pilot stopped before official capture

- Canonical product result: G8-B closed/frozen at `18391e6a9fc8f9bc7b2757f3504366f106c05435`; legacy launchers retired at `8ee1ae238c324c1db1d7e2882af071fec179a8f1`.
- Active writer: `feature/m0-g8c-official-matrix` at upstream-equal base `8ee1ae238c324c1db1d7e2882af071fec179a8f1`, with 14 intended files staged and no sealed G8-C source commit.
- Validation before pilot passed: Windows G8-C `9/9`, renderer timestamp/surface `3/3`, benchmark `27/27`, coordinator/verifier `43/43`, legacy verifier fixture, fmt/check/clippy/audit/diff. FULL remained `0`.
- Pilot: `g8c-pilot-8ee1ae238c32-c64090539536`; all five headless Mode A/B processes passed; first Sand Fall Mode C process exited `1`; remaining Mode C/D and all official publication stages were skipped.
- Failure: renderer initialization confirmed live 1600×900, then a late initial `Resized(2864×1560)` event payload was treated as a real resize. Proposed remediation rechecks live `window.inner_size()` and ignores only stale payloads while keeping genuine noncanonical live sizes fatal.
- Evidence boundary: incomplete scratch only, no Receipt, hash inventory, package, matrix report, or verifier result; no performance/bottleneck conclusion exists.
- Next: Q-006 asks the user whether to authorize one narrow remediation, one replacement pilot, and conditional official capture. PR #4 remains unmerged while the writer is dirty.