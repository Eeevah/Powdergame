# Checkpoint — TE-4I accepted; TE-5 architecture re-entry is next — 2026-08-23

## Repository coordinate
- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Acceptance-task baseline: `0b5c80e93e6693d86d7be6f2d74819da6143c358`
- Preserved production physics: `8d9e8cbe3b6ac651335b5a728ef491abeae4772a`
- Scene 4 observability source: `a7622bf2106a11e731a46018a4afe30d236b9304`
- Acceptance decision/docs: `68ed1305a017ac946cf3795eba79175de6ae9a38`
- Wiki base: `bf3d2b1d585492f470b92f09342935a497093379`
- Wiki lesson commit: `fe29fde421425491087d65e09c5557d1fe8b9cc1`
- Wiki PR: `https://github.com/Eeevah/personal-infra-wiki/pull/49`
- Wiki main merge: `b9a36c7712cda2ac5332e3083e0e5ff5b018fa91`

## Current product state
Scenes 1–4 are **DIRECT OBSERVATION CONSISTENT**. TE-4I is **USER ACCEPTED WITH
KNOWN FOLLOW-UP**. ADR-0012 is **ACCEPTED FOR THE CURRENT TE-4
IGNITION-KINETICS IMPLEMENTATION** and Q-017 is closed under D-034. Production
physics and observability sources are unchanged. TE-4D v1/v2/v3 and D-031
remain blocked immutable history. PG-L036 is promoted to Wiki `main`.

## Valid evidence
- Source-bound production evidence:
  `docs/evidence/THERMAL_ENVIRONMENT_TE_4_IGNITION_KINETICS_2026-08-23.md`.
- Scene 4 remediation receipt:
  `docs/evidence/TE4_SCENE4_SMOKE_OBSERVABILITY_REMEDIATION_2026-08-23.md`.
- Scene 4 user sequence: Tick `0` READY; Tick `1` exact Smoke target and whole
  Air receiver; Tick `2` next-snapshot extinguish/no second Smoke; Tick `1184`
  decay to EMPTY, Air recovery and target marker absent.
- Existing final-source FULL and F01..F17 evidence remain source-bound and
  valid. This docs/memory-only closure ran Cargo/GPU/FULL/build/launch zero
  times.
- Powdergame strict audit, Markdown link/fence/index, secret, scope and diff
  checks passed.
- Wiki full validation passed: 41 tests, inventory, secret, health, project
  document contracts and overlay. PR CI passed all six check families on both
  push and PR runs before ordinary merge.
- User-dirty local Wiki remains exactly `M wiki/workflows/index.md` and
  `?? wiki/workflows/codex-lm-studio.md`.

## Accepted implementation boundary
- Packed u6 exposure owns flags bits 2..3 and 28..31.
- Oil is `48/2/50/6/1/2/4`; Wood is `60/1/50/5/1/2/4`.
- `COMBUSTION_STAGE_SNAPSHOT` is the precondition lifetime.
- Positive Atmosphere/LowPressure EMPTY Air faces qualify; exact Vacuum and
  occupied GAS do not. Air is not Oxygen and is not consumed.
- Oil emits 599 ticks and consumes on Tick 600; Wood emits 899 and consumes on
  Tick 900. Final consumption emits no Heat/Flame/Smoke/Q.
- Same-Tick Smoke may remove the last Air face without rollback; the next
  snapshot extinguishes. Target/receiver settle is authoritative.
- No new persistent or full-world scratch state exists.

## Exactly one next action
Authorize **TE-5 Pressure/Vacuum architecture re-entry**. Status remains
**DESIGN RE-ENTRY REQUIRED / NOT STARTED**. Begin by reading all immutable
TE-5B/C/D/X/Q counterexamples and explicitly deciding which architecture
constraints may change; do not resume any blocked model automatically.

## Boundaries
Binary Air-policy refinement requires a new decision. Oxygen quantity and Ash
remain absent. Final flame/glow/burn/Smoke-source presentation belongs to
G9-D. Pressure coupling and TE-5 implementation have not started. TE-6 and
G9-B/C/D/E remain not started. No Powdergame PR or `main` merge occurred.
