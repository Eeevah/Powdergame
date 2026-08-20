# Checkpoint — TE-1 Environment foundation implemented; TE-2 not started — 2026-08-20 16:41 KST

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- HEAD: runtime source `1a722d239a16bade5772688fa822465d5cef4602`; the immediately following docs/memory closure contains this checkpoint, so live Git wins
- Working tree: expected TE-1 docs/memory closure files until the closure commit; final target is clean and upstream-equal

## The story so far

G8 remains closed/frozen. G9-A continuity v2 remains **REVISED IMPLEMENTATION
CANDIDATE / USER RE-REVIEW PENDING**; no acceptance verdict was inferred.

D-014 implements TE-1 only. The GPU world now owns four full-resolution Air
mass/energy Current/Next buffers and one u32 receiver-claim scratch. One
canonical staging/reset contract distinguishes Atmosphere and Vacuum, while
movement, phase, Smoke, decay, fuel consumption and rupture reconcile
Environment at explicit joint-settle boundaries. Phase/Smoke spawns are
receiver-gated whole-parcel transactions; a failed phase receiver receives the
existing blocked-expansion pressure source exactly once. Matter flag ownership
is sanitized independently. The profiler covers all 30 passes and allocation
reports pin 4,196,864 B at 256² and 268,462,208 B at 2048² without profiling.

Air does not move between EMPTY Cells, exchange heat, or apply atmospheric
pressure to Matter. TE-2 and G9-B/C/D/E are not started.

## Valid evidence

- Runtime source `1a722d239a16bade5772688fa822465d5cef4602` — valid while runtime/test bytes, RTX 5090/DX12 environment and locked commands remain unchanged. Final-source workspace FULL, warnings-denied clippy, all-target check, strict audit and one Sandbox bounded launch passed.
- `docs/evidence/THERMAL_ENVIRONMENT_TE_1_FOUNDATION_2026-08-20.md` — exact TE-1 scope, constants, pass/allocation data and limitations.
- Release EXE `target/release/powdergame-windows.exe` — SHA-256 `8c3f0050eef67cfca04e970c071276ce8ae856a7a1a65e58ff63a0deecb34ea6`, size 9,936,896 B, valid only for runtime source `1a722d...`.
- TE-0.2 reference proof remains formula-only evidence and does not prove TE-2 physics.

## Decided

- D-007 — G8 is closed/frozen.
- D-012 — G9-A still awaits continuity v2 user re-review.
- D-013 — Air is separate mass/energy Environment, not Matter.
- D-014 — TE-1 state/occupancy foundation is implemented; TE-2 is not authorized.

## Waiting on the user

- Re-review G9-A continuity v2 on the new TE-1 descendant executable.
- Decide whether to authorize TE-2 as a separate task.
- Q-008 later-gate choices remain open and were not silently selected.

## Next first action

Run `run_powdergame.bat sandbox` for the pending G9-A continuity v2 re-review;
do not start TE-2 or G9-B/C/D/E without a new explicit authorization.

## Tried

- A no-scratch receiver design was rejected during TE-0 because the original Matter claim remains live; TE-1 uses exactly one dedicated u32 scratch.
- The first final FULL attempt exposed a pre-existing Heavy Mixed census block misplaced in the Fire/Heat fixture test. The stale test block was removed without retuning production fixtures, invalidating that attempt; the final source then passed the full workspace.
- The local personal-infra Wiki checkout remained user-dirty. Remote `origin/main` object `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3` was used as authority and no Wiki file was modified.
