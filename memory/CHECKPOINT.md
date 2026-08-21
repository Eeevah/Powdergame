# Checkpoint — revised TE-3 review candidate ready — 2026-08-21 18:17 KST

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- HEAD before docs closure: `c2f4f2bb16b00801a72ff6e4a54726cc69674bad`
- Working tree: expected docs/memory closure files only
- Wiki remote fallback: `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3`; local Wiki remains user-dirty and untouched

## The story so far

D-024 production physics remains unchanged at `4146721...`. Direct review
found that candidate Scenes 2–4 did not reliably exercise their labels and the
HUD sampled one often-empty Cell. Candidate-only source `c2f4f2b...` now stages
honest surface/buried/reveal, lid/free-Air/K=0, reversal/no-sink controls and
three generation-safe fixed diagnostic rows per scene. No Engine/Core/WGSL,
phase rule, Air coefficient or pressure code changed.

## Valid evidence

- `docs/evidence/THERMAL_ENVIRONMENT_TE_3_PHASE_CYCLE_2026-08-21.md` — F01–F15 and final-source FULL remain valid while production physics stays at `41467219819c5d0cb3eab8ae22b652449da20480`.
- `docs/evidence/TE3_DIRECT_REVIEW_SURFACE_REMEDIATION_2026-08-21.md` — valid for candidate source `c2f4f2bb16b00801a72ff6e4a54726cc69674bad`, current Scene 2–4 geometry, diagnostic schema and RTX 5090/DX12 artifact.
- Windows binary suite: 170 passed / 1 unrelated ignored; phase semantic tests: 6 passed. Valid while the three `apps/windows` candidate files are unchanged.
- Canonical EXE SHA-256 `F15B8B1198443935CB233A0FA526256563F400A0775ECC246542BB195938F966`, 10,095,104 bytes.

## Decided

- D-024 — one-Cell/one-quantity pressure-decoupled TE-3 remains active; Water phase pressure is disabled.
- TE-3 production physics remains `4146721...`; this remediation is candidate presentation/staging only.
- Workspace FULL count is 0 for this remediation; the prior production FULL is not rebound.

## Waiting on the user

Direct review of the revised four-scene candidate. No acceptance is claimed.

## Next first action

Run `run_powdergame.bat phase-cycle` and review the fixed Scene 2–4 rows and causal checkpoints.

## Tried

- Four orthogonal blockers alone allowed legal diagonal movement; the final controls close the full relevant movement stencil.
- A broad free-Air Steam island vacated the fixed centre; the final nine-Cell row cools through downward Air faces that GAS movement never targets.
- Eight-tick-only observation could not prove sparse onset; the final semantic test samples each tick through the first free-Air partial event.
