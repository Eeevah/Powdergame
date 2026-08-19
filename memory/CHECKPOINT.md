# Checkpoint — G9-A First Playable candidate awaits user review — 2026-08-19 13:57 KST

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- HEAD at checkpoint update: `0d03dafbb4bc6375adc10c8b819db6c0bc232db9`
- Tested source: `0d03dafbb4bc6375adc10c8b819db6c0bc232db9`
- Working tree: expected memory-only checkpoint/archive/session-log closure; live Git wins if this note is stale

## The story so far

G8 is closed/frozen and its independently verified G8-C Matrix recommends `PROCEED_TO_G9`; optimization remains deferred. The approved G9-A slice is now implemented inside the canonical Windows EXE as an explicit Sandbox mode. Starter Lab and New Blank World are editable production GPU worlds. The candidate exposes all nine M0 Matter, Draw/Erase/Heat/Cool, four brush sizes, Pause/Play/Step/x1/x4/x16/Reset, Pan/Zoom and the existing bounded Cell Inspector.

The edit path batches and coalesces bounded cell commands, applies both Current and Next at one pre-tick boundary, performs exact field/flags hygiene, and wakes touched chunks plus their clipped neighbor halo. Rendering, physical-pixel picking and Inspector hover share one camera transform. After direct candidate feedback, no-argument BAT/EXE launch opens Sandbox; Gallery, runtime, experiment workers and G8-C remain explicitly routed.

Edit-core source `f9a7087...` passed the Windows unit suite (`149 passed`, `1 ignored`) and exact shared scenario reset. Launch source `0d03daf...` passed default/explicit-route tests, affected check/clippy, strict launcher/policy audit, and one no-mode release 3-frame Sandbox smoke on RTX 5090/DX12. This is implementation integrity, not user acceptance.

## Valid evidence

- G9-A edit-core validation — valid from source `f9a7087249bf6ffa0b6d47ad7568ba1798f591a3` through launch-only descendant `0d03dafbb4bc6375adc10c8b819db6c0bc232db9` while Cargo graph, engine/Core, fixtures and shared Simulation layout remain unchanged.
- Canonical no-mode smoke — `target/release/powdergame-windows.exe`, SHA-256 `9e809342074c313c79a1080a89b9aa6e84e0e39238b4c8d9aa1368ad8bc72f3c`, 3 frames, exit `0`; valid only as startup/default-routing evidence for source `0d03daf...`, not UX approval.
- G8-C official Matrix `g8c-official-matrix-4653d7c2e09e-64df60ba0d79` — remains valid under its sealed source/artifact/verifier identities; G9 docs do not invalidate it.

## Decided

- D-006 — G9 Product Brief approved.
- D-007 — old G8-A visual requirement superseded; G8 closed/frozen.
- D-008 — current Ballast rollback is newest-first Ballast-only reverts, then `git revert -m 1 6b5f0201...`.
- D-009 — canonical no-argument BAT/EXE launch opens the G9-A Sandbox; Gallery is explicit.
- Other G9-A implementation proposals remain candidate details, not new user-approved product decisions.

## Waiting on the user

Run the candidate for approximately 10–15 minutes and accept, revise or reject the editor/sandbox experience. G9-B/C/D/E remain blocked on that decision.

## Next first action

From `C:\Users\mdkap\source\repos\Powdergame-g8b`, double-click `run_powdergame.bat` and work through the manual G9-A acceptance checklist in `docs/vision/FIRST_PLAYABLE_WORLD.md`.

## Tried

- One nine-storage-buffer edit bind group exceeded the actual adapter limit of eight; field writes and flags hygiene were split into ordered passes within one submission, preserving Current/Next authority.
- Applying a patch to the root BAT converted line endings to LF and broke `cmd.exe` label routing; the launcher was restored to CRLF and the full policy probe passed.
- One timer wrapper invocation used ambiguous `--` syntax and stopped before build/EXE launch; explicit `RemainingArgs` then recorded the only actual Sandbox smoke.
