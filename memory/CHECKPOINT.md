# Checkpoint — TE-5C replacement design authorized — 2026-08-21 11:39 KST

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Start/current remote: `6a1c83fad702d18f2d24365a4fc747ab74225f5c`
- TE-3D architecture: `d7500e219af6f670be05f830b50c232d2bb53077`
- Runtime source remains unchanged; this task is docs/reference only.

## Current truth

G9-A and TE-2 remain **USER ACCEPTED WITH KNOWN FOLLOW-UP**. TE-3D remains
**ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS** and ADR-0006 remains
**ACCEPTED FOR FUTURE ATOMIC IMPLEMENTATION**.

D-020 rejects the TE-5B token as **REJECTED / DESIGN BLOCKED** and preserves
D-019/ADR-0007 as history. It authorizes TE-5C **LOCAL VAPOR CAPACITY SHARE +
GAUGE-PRESSURE EQUILIBRIUM** as the final no-new-persistent-state attempt.
ADR-0008 is not yet proposed. TE-3/TE-5 runtime, full TE-5, Air force, TE-4
and G9-B/C/D/E are **NOT STARTED**. The historical G5 Water path remains active.

## Locked candidate and audit status

- Vapor demand derives from accepted phase energy with `Lv=480`.
- Radius-1 EMPTY capacity uses the prompt's proportional sharing law without
  target mutation or ownership state.
- Compression maps linearly to a gauge target capped at `100.0`; orthogonal
  EMPTY is a proposed gauge-zero vent face at rate `0.20`.
- Static source audit confirms a proposal-scratch window after Smoke settle
  and before pressure. One full-write capacity-sum pass projects 41 passes / 82
  queries, subject to proof and review.
- Known attack to preserve: proportional per-EMPTY sharing can underuse
  reachable capacity when a multiply-connected phase Cell receives shares
  from several EMPTYs while another phase Cell has only one. The locked proof
  must report, not repair, this case.

## Evidence boundary

- Local Wiki was user-dirty and untouched; remote fallback SHA:
  `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3`.
- No TE-5C proof, ADR-0008 or review exists yet.
- Cargo/test/check/clippy, GPU, FULL, build, launch and runtime counts are zero.
- External implementation copy count: `0 files / 0 lines`.

## Next first action

Draft ADR-0008/spec/validation/planning authorities and the predeclared proof
contract. Execute the grid/time proof exactly once. A required fixture failure
is preserved and stops TE-5C DESIGN BLOCKED; no formula switch is allowed.
