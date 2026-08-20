# Checkpoint — revised G9-A candidate awaits user re-review — 2026-08-20 09:31 KST

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- HEAD: `b363c078fdc1d7e8b54fa6be328b7a0c5b908f06`
- Working tree: expected docs and Ballast closure for the tested source; live Git wins if this note is stale

## The story so far

G8 remains closed/frozen and optimization remains deferred. The first G9-A Sandbox was directly reviewed and classified **USER REVIEWED / REVISION REQUIRED**. D-011 limits remediation to five details. Source `b363c078fdc1d7e8b54fa6be328b7a0c5b908f06` implements exactly those details: Draw writes only to Cells whose Current and Next are EMPTY; direct Ice/Steam placement initializes both buffers at -30°C/80°C; all nine Matter stay available under Core/Generated/Advanced palette groups; Inspector may show only the explicitly identified prior Cell/Material for 150 ms while fresh fields are pending; and Heat/Cool shows camera-aligned preview/application feedback without adding world truth or readback.

The production GPU edit boundary remains one bounded/coalesced pre-tick submission with Current/Next hygiene and touched/halo wake. The 24-byte Inspector cadence stays at most 10 Hz. Gallery, runtime, experiment workers and G8-C routes remain isolated. G9-B/C/D/E, Ash, Discovery, Save/Load, Rewind, optimization, main promotion, PR creation and M0 closure have not started.

## Valid evidence

- Revised G9-A Windows validation — source `b363c078fdc1d7e8b54fa6be328b7a0c5b908f06`; valid while engine/Core, fixtures, production WGSL, Cargo graph and shared Simulation layout remain unchanged: fmt PASS; Windows unit suite `150 passed / 0 failed / 1 ignored`; affected all-target check PASS; affected clippy with denied warnings PASS; strict policy audit PASS.
- Revised Sandbox bounded launch check — canonical `target/release/powdergame-windows.exe`, SHA-256 `26512598746c21858a81c85a2e4f8f2635e2e1deed6c1ebff661bc2810a126d1`, 9,876,480 bytes; `run_powdergame.bat sandbox --smoke-frames 3`; RTX 5090 / DX12; Starter Lab staged paused; 3 frames; exit `0`. This proves startup/routing only, not user acceptance.
- G8-C official Matrix `g8c-official-matrix-4653d7c2e09e-64df60ba0d79` remains valid under its sealed identity; this G9-A app-local revision does not mutate that evidence.

## Decided

- D-006 — G9 Product Brief approved.
- D-007 — G8 is closed/frozen.
- D-009 — use `bounded launch check` for short software validation; reserve `Smoke` for the Matter.
- D-010 — canonical no-argument BAT/EXE launch opens Sandbox; Gallery is explicit.
- D-011 — first G9-A candidate requires exactly five interaction revisions and returns only to user re-review pending.

## Waiting on the user

Re-review the five revised interactions and accept, revise or reject G9-A. G9-B/C/D/E remain blocked on that decision.

## Next first action

Double-click `C:\Users\mdkap\source\repos\Powdergame-g8b\run_powdergame.bat`, then verify EMPTY-only Draw, Ice/Steam placement, palette grouping, Inspector grace and Heat/Cool feedback.

## Tried

- The user launched an older release executable and saw the G8 surface; the current BAT now rebuilds the canonical release executable and the revised source completed a bounded Sandbox launch check.
- Workspace FULL was classified recommended rather than required and was not run because no engine/Core/fixture/WGSL/Cargo/shared Simulation state path changed.
