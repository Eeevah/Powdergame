# Checkpoint — TE-4I production candidate ready for direct review — 2026-08-23

## Repository coordinate
- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Start: `a19753ba087309e4f2a4863915d57b67750f1ad2`
- D-032: `ae8c04bc42f85c6a78d5960e08f3f4bcef1a28cd`
- Final runtime: `8d9e8cbe3b6ac651335b5a728ef491abeae4772a`
- Docs/checkpoint commit: pending this closure commit

## Current product state
TE-4D v1/v2/v3 and D-031 remain blocked immutable history. TE-4I is an
implemented production candidate with automated validation PASS. ADR-0012 is
still Proposed; direct user architecture/product review is pending.

## Valid evidence
- Source-bound receipt:
  `docs/evidence/THERMAL_ENVIRONMENT_TE_4_IGNITION_KINETICS_2026-08-23.md`.
- Final-source FULL attempt 3 PASS; attempts 1/2 remain recorded invalid-source
  failures.
- One release build, one bounded launch, and one bounded measurement passed.
- Canonical EXE SHA-256:
  `27D92287931421560027EF4D554DA26BBB50C5DE1565D75E52D1BC406A2A6081`.
- Wiki authority `57d7e2bdbab5b9cbc46a4448fd881e7493e12f74`; dirty local Wiki untouched.

## Locked implementation
- Oil `48/2/50/6/1/2/4`; Wood `60/1/50/5/1/2/4`.
- Packed u6 flag mask `0xF000000C`; combustion mask `0xF000FFFF`.
- Binary `COMBUSTION_STAGE_SNAPSHOT` Air access; no Oxygen quantity.
- Chemical Q Oil/Wood `15/8`; final consumption tick emits zero.
- 42 passes / 84 queries / 1,344 profiler bytes.
- New persistent state `0`; new full-world scratch `0`; max storage bindings `8`.

## Waiting on the user
Use the Desktop shortcut or `run_powdergame.bat ignition-kinetics` and apply
the twelve-item checklist in `docs/planning/TE4_IGNITION_KINETICS.md`. The user
must accept, revise, or reject ADR-0012/TE-4I; automation cannot close Q-017.

## Boundaries
No Oxygen, Ash, Pressure redesign, TE-5/TE-6, G9-B/C/D/E, optimization, PR, or
main merge started. Lesson promotion is NONE because existing PG-L034/PG-L035
and Wiki contracts already cover the reusable failure classes.
