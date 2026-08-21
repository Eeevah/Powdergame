# Checkpoint — pressure-decoupled TE-3 runtime authorized — 2026-08-21

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Task baseline: `b5a3405bffba5f757f966e44d0cae0077f57d285`
- Wiki read-only fallback: `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3`
- Preflight local/remote: exact match, left/right `0/0`, clean before D-024 memory edits

## Current truth

D-024 directly rejects ADR-0011 as the active representation, preserves all
TE-5B/C/D/X and packet blockers as history, and supersedes the atomic TE-3/TE-5
activation requirement. Standalone one-Cell/one-quantity ADR-0006 TE-3 runtime
implementation is authorized. Water boiling yield is 1, blocked pressure is 0,
and Water never enters generic expansion. TE-5 Pressure redesign is DEFERRED /
NOT STARTED.

## Scope and boundary

- Add only `phase_energy_current` and `phase_energy_next` persistent Cell state:
  exactly 32 MiB at 2048 squared.
- Preserve D-018 sink, burial, metastability and nucleation amendments.
- Reuse TE-2 transfer, Current/Next settle, scratch, activity, profiler,
  Sandbox controls, canonical EXE and launcher.
- Do not add packets, units, volume/pressure state, matching, CCL or a pressure-
  volume model.
- Historical G5 Water pressure evidence stays source-bound.

## Validation boundary

- No runtime validation has yet run for D-024.
- Run the validation plan first, then targeted tests.
- Freeze and commit the runtime source before exactly one canonical FULL, one
  release build, one bounded candidate launch and one bounded measurement.
- G8/G8-C, TE-4, G9-B/C/D/E and official capture remain zero.

## Next first action

Run `tools/dev.ps1 validation-plan` with base
`b5a3405bffba5f757f966e44d0cae0077f57d285`, then audit live source pass,
binding, writer and candidate-control contracts before implementation.
