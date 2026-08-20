# Checkpoint — continuity v2 awaits user re-review; thermal design is planned — 2026-08-20 10:06 KST

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- HEAD: `a00e39b2e00bfbd9ac28214c44cd22cc97542bb4`
- Working tree: expected docs and Ballast closure for the tested source; live Git wins if this note is stale

## The story so far

G8 remains closed/frozen and optimization remains deferred. The five-revision G9-A build was directly re-reviewed: Draw/Ice/Steam/palette/Heat-Cool changes were present, but rapid Cell movement still caused Inspector on/off flicker. D-012 records **USER RE-REVIEWED / REVISION REQUIRED** without claiming acceptance. Source `a00e39b2e00bfbd9ac28214c44cd22cc97542bb4` implements Inspector continuity v2 only.

Requested hover and presented sample are now separate. One previous sample retains its original Cell, Material, sample tick, diagnostic sequence and freshness across second/third rapid hovers for a single 150 ms deadline. It is labelled `Previous sample`; compact tooltip remains hidden until a fresh current-Cell sample atomically replaces the panel. After timeout, the same-size detailed shell shows Sampling. Reset, preset, world epoch and readback failure clear the held sample immediately. The collector remains one persistent 24-byte batch at most 10 Hz.

Thermal Transport & Ignition Causality is registered as **PLANNED / DESIGN REQUIRED / IMPLEMENTATION NOT STARTED** and a G9-B prerequisite. Current EMPTY/no-hidden-Air, four-neighbor direct-contact transport and immediate threshold ignition are unchanged. Option A existing-field ambient, Option B separate ambient field, and ignition exposure/dose remain open design choices.

## Valid evidence

- Inspector continuity v2 validation — source `a00e39b2e00bfbd9ac28214c44cd22cc97542bb4`; valid while engine/Core, production WGSL, fixtures, Cargo graph and shared Simulation state remain unchanged: Inspector tests `11/11`, Inspector UI tests `3/3`, fmt, affected all-target check, denied-warning clippy, strict policy audit and diff-check PASS.
- Canonical release bounded launch check — source `a00e39b...`; `target/release/powdergame-windows.exe`; SHA-256 `5062f0cb0ac9f23828765ce6c2fe2c2137caaa2f055c1c4fcfd9fb0cf7f177d5`; 9,875,456 bytes; `run_powdergame.bat sandbox --smoke-frames 3`; RTX 5090 / DX12; exit `0`. This proves startup/routing only, not user acceptance.
- Prior source `b363c078...` full Windows suite evidence remains useful for unchanged Sandbox/editor/routing paths; continuity v2 changed only Inspector and text UI, which received the focused replacement checks above.
- G8-C official Matrix `g8c-official-matrix-4653d7c2e09e-64df60ba0d79` remains valid under its sealed identity; no G8 candidate or Matrix was rerun.

## Decided

- D-007 — G8 is closed/frozen.
- D-010 — canonical no-argument BAT/EXE launch opens Sandbox; Gallery is explicit.
- D-011 — first review required five bounded G9-A revisions.
- D-012 — re-review still requires Inspector continuity v2; thermal causality is a planned G9-B prerequisite, not an implementation authorization.

## Waiting on the user

Re-review Inspector continuity v2 during rapid movement. Thermal architecture selection remains a separate future decision. G9-B/C/D/E remain blocked and not started.

## Next first action

Double-click `C:\Users\mdkap\source\repos\Powdergame-g8b\run_powdergame.bat`, enable detailed Inspector with `I`, and sweep rapidly across Cells to verify stable Ready/Previous sample/Sampling transitions.

## Tried

- Identity-only grace derived only from `latest_sample` was lost on the second hover change and shrank the panel; continuity v2 replaces that coupling with a persistent presented sample and one fixed panel geometry.
- Workspace FULL was not run because the change stayed in app-local Inspector/text presentation; engine/Core/WGSL/shared Simulation/fixture paths were untouched.
