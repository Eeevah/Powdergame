# Checkpoint — TE-4I Scene 4 revised candidate awaits user re-review — 2026-08-23

## Repository coordinate
- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Start/docs baseline: `4ae570d104268f267ce59cc2c1f8816c803721bb`
- Preserved production physics: `8d9e8cbe3b6ac651335b5a728ef491abeae4772a`
- Scene 4 observability source: `a7622bf2106a11e731a46018a4afe30d236b9304`
- Docs/checkpoint commit: pending this closure commit

## Current product state
Scene 4's original review surface was revision-required because source-side
fuel/Air/burning changes did not visibly prove Smoke creation. Exact candidate
geometry now proves target `(209,110)` is authoritative Smoke at Tick 1 and
receiver `(209,111)` receives the displaced Air. Production physics is
unchanged. TE-4I is **REVISED IMPLEMENTATION CANDIDATE / SCENE 4 USER
RE-REVIEW PENDING**. ADR-0012 remains Proposed and Q-017 remains open.

## Valid evidence
- Original production evidence:
  `docs/evidence/THERMAL_ENVIRONMENT_TE_4_IGNITION_KINETICS_2026-08-23.md`.
- Scene 4 remediation receipt:
  `docs/evidence/TE4_SCENE4_SMOKE_OBSERVABILITY_REMEDIATION_2026-08-23.md`.
- Windows package `183 passed / 1 ignored`; production F08 and targeted
  Environment receiver/settle controls passed.
- Production runtime FULL remains the existing valid PASS at `8d9e8cb...`;
  remediation FULL count is zero because no Engine/Core/WGSL changed.
- Canonical EXE SHA-256:
  `EABC00C3F803800EEFA9DD935DD15F3AEDEB507EEBD638364088E2BD324D6297`;
  size `10,147,328` bytes.
- Wiki authority `bf3d2b1d585492f470b92f09342935a497093379`; dirty local Wiki untouched.

## Revised Scene 4 surface
- Fixed source, `Self-Smoke target`, and `Air receiver` rows.
- Persistent Smoke count, source fuel, target Material, and receiver Air.
- `READY`, `EMITTED`, `EXTINGUISHED`, `DECAYED` causal states.
- Camera focus keeps `(208,110) -> (209,110) -> (209,111)` left of the card.
- Outline exists only while sampled target Material is Smoke.
- No fake/duplicate/enlarged Smoke and no production Inspector change.

## Exactly one next action
The user re-reviews Scene 4: open the existing TE-4 shortcut, press `4`, press
`N` once to observe `EMITTED`/Smoke count one/target outline/receiver Air, then
press `N` again to observe `EXTINGUISHED` with fuel one and no second Smoke.
Only the user may then accept, revise, or reject TE-4I/ADR-0012.

## Boundaries
No coefficient, ignition predicate, Engine/Core/WGSL, Oxygen, Ash, Pressure,
TE-5/TE-6, G9-B/C/D/E, optimization, PR, or main work started. No user
acceptance is claimed.
