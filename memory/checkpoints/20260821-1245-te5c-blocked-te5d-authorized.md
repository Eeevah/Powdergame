# Checkpoint — TE-5C blocked; persistent-state decision next — 2026-08-21

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Task baseline: `6a1c83fad702d18f2d24365a4fc747ab74225f5c`
- Authorization commit: `cceca63fc7597aa58d6053a15118f761964366e0`
- Wiki read-only fallback: `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3`
- Runtime source is unchanged; this task remained docs/reference only.

## Current truth

G9-A and TE-2 remain **USER ACCEPTED WITH KNOWN FOLLOW-UP**. TE-3D remains
**ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS**; ADR-0006 remains accepted
for a future atomic implementation. TE-5B is **REJECTED / DESIGN BLOCKED**.

D-020 authorized TE-5C as the final no-new-persistent-state attempt. The
locked grid/time proof returned `DESIGN_BLOCKED`, and fresh review ended at
Critical `0` / High `6`. ADR-0008 remains **PROPOSED — DESIGN BLOCKED /
ARCHITECTURE REVISION REQUIRED**. TE-3/TE-5 runtime, full TE-5, Air force,
TE-4 and G9-B/C/D/E are **NOT STARTED**.

## Blocking evidence

- Exactly one proof process ran at fixed seed `0x54453543`, with 50,000 static
  neighbourhoods, 10,000 bounded multi-tick grids and two deterministic
  replays.
- Script SHA-256:
  `f0b4cb155fcc0785c60ff6ff4c2ee9d18a439ed3ea0941e679140de4188af791`.
- Result SHA-256:
  `59b98a3454e13a22742e66559e06cfa9b3552a37e18929fa3b71949afaf1e8e5`.
- The smallest witness has two unit-demand Steam and two EMPTY Cells. A full
  assignment exists, but proportional sharing caps and discards one excess
  share, leaving the other Steam at capacity `0.5` and false target `100`.
- Fresh review also found internal EMPTY capacity/vent conflation,
  irreversible phase-pressure provenance, unreachable downward Chebyshev
  capacity, activity/snapshot/binding infeasibility and overclaimed proof
  checks. Review SHA-256:
  `d0d26585326d79cfe60ab0fd0a334e9537e6bedc8d41059e5e129caa08d2edf2`.

The proof bytes are preserved and were not rerun or patched. No formula,
radius, curve, token or impulse substitute was selected.

## Validation boundary

- Markdown links/fences, strict development policy, secret scan,
  docs/memory-only classification and `git diff --check` passed before final
  commit.
- Cargo/test/check/clippy, GPU/device, FULL, build, launch and runtime counts:
  zero.
- External copied/translated/vendored implementation: `0 files / 0 lines`.

## Next first action

Obtain an explicit user architecture decision that permits persistent
phase-volume state before designing another replacement. Do not attempt a
third stateless token/impulse or silently change the failed TE-5C law.
