# Checkpoint — TE-2 passive thermal candidate; user review pending — 2026-08-20

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Runtime source: `fb7e568e21012b6067269f4e1b82c36c865023d0`
- Final target after the docs commit: clean and upstream-equal

## Current truth

G8 remains **CLOSED / FROZEN**. G9-A remains **REVISED IMPLEMENTATION
CANDIDATE / USER RE-REVIEW PENDING**; no acceptance was inferred.

D-015 implements TE-2 full-resolution Air flow, donor-energy advection,
unified passive thermal exchange, activity/wake and the atomic Celsius-like
temperature migration. CPU reference and production GPU both pin the 30-case
`SMALL_DELTA_THERMAL_CONVERGENCE` contract. The deadband is a shared `> 0.01
°C` work gate, not a subtractive flux.

TE-2 is **PASSIVE THERMAL ENVIRONMENT CANDIDATE / USER REVIEW PENDING**.
Air-pressure force, TE-3/TE-4 and G9-B/C/D/E are **NOT STARTED**.

## Valid evidence

- Runtime source `fb7e568e21012b6067269f4e1b82c36c865023d0`.
- Final-source targeted tests, warnings-denied clippy, strict audit and exactly
  one serial workspace FULL: PASS.
- Release EXE SHA-256
  `e1f7e9b3428fbd40f8a3030cb302d8691a28383b494336d6f822be79b9f66512`,
  size `10,000,896` B; exactly one 60-frame TE-2 bounded launch passed.
- One-shot performance CSV SHA-256
  `f67c058ba0bf41cee0d108f66c9e4599ecf03b06cbc46714ff866dea7c4b5658`.
  2048² equilibrium/frontier GPU tick P95 is `2.599712/2.304832 ms`; the
  equilibrium terminal has zero active Cells/chunks.
- `docs/evidence/THERMAL_ENVIRONMENT_TE_2_PASSIVE_TRANSPORT_2026-08-20.md`.

## Waiting on the user

- Review the four-scene TE-2 candidate via `run_powdergame.bat
  thermal-environment`.
- Separately re-review G9-A Inspector continuity v2 in Sandbox.
- Decide later-gate Q-008 items only at their named boundaries.

## Next first action

Do not start TE-3, TE-4 or G9-B/C/D/E. Await direct TE-2 and G9-A user
dispositions.

## Preserved boundary

The local personal-infra Wiki checkout remained user-dirty. Remote
`origin/main` object `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3` was used as authority and no
Wiki file was modified. External simulation code copied/translated/vendored
for the Chinese-community research intake: `0 files / 0 lines`.
