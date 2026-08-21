# Checkpoint — TE-5D persistent-state design authorized — 2026-08-21

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Task baseline: `94e4da7603dafcf4a83c652abe192a435779a127`
- Authorization commit: pending this docs/memory commit
- Wiki read-only fallback: `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3`
- Runtime source is unchanged; this task remained docs/reference only.

## Current truth

G9-A and TE-2 remain **USER ACCEPTED WITH KNOWN FOLLOW-UP**. TE-3D remains
**ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS** and ADR-0006 remains accepted
for future atomic implementation. TE-5B and TE-5C are **REJECTED / DESIGN
BLOCKED**. Their proofs and reviews remain immutable historical evidence.

D-021 authorizes TE-5D **PERSISTENT VAPOR EXTENT + DEDICATED PHASE PRESSURE**
as a docs/reference design program. It explicitly permits one new per-Cell
Current/Next pair while preserving 1:1 phase-family quantity and no extra
Steam. ADR-0009, the locked proof and independent review are not yet complete.
TE-3/TE-5 runtime, full TE-5, TE-4 and G9-B/C/D/E are **NOT STARTED**.

## Validation boundary

- No runtime, Cargo, GPU, build, launch or FULL validation is authorized.
- Exactly one external fixed-seed proof run will be permitted after its script,
  fixtures, bounds and hashes are frozen.
- External copied/translated/vendored implementation remains `0 files / 0
  lines`.

## Next first action

Complete the source/pass/binding audit, write the ADR-0009 authority set,
freeze and execute the proof exactly once, then obtain a fresh-context
independent review. Any unresolved Critical/High stops TE-5D DESIGN BLOCKED.
