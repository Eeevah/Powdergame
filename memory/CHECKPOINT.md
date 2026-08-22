# Checkpoint — TE-5R1 automated candidate ready for direct review — 2026-08-23

## Repository coordinate
- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Start/remote baseline: `12b49dc07c8d875de55a048013a01090d38345a9`
- Final runtime source: `1ee28ac2003d3e2804dfce5fbf0fa25e583e3030`
- Docs/checkpoint closure: this coherent docs/memory commit; resolve with `git rev-parse HEAD`
- Verified Wiki `origin/main`: `b9a36c7712cda2ac5332e3083e0e5ff5b018fa91`
- User-dirty Wiki preserved: `M wiki/workflows/index.md`; `?? wiki/workflows/codex-lm-studio.md`

## The story so far
G9-A, TE-2, TE-3 and TE-4I remain **USER ACCEPTED WITH KNOWN FOLLOW-UP**.
TE-5R0/ADR-0013 remains blocked immutable history. D-037 authorized the
source-realizable TE-5R1 Steam-load relaxing pressure replacement. Its fresh
review closed Critical/High at zero, runtime source `1ee28ac...` passed the
complete local automated gate, and ADR-0014 remains **PROPOSED**.

TE-5R1 is **IMPLEMENTATION CANDIDATE / AUTOMATED VALIDATION PASS / USER REVIEW
PENDING**. No Matter pressure force, Oxygen quantity, new persistent buffer,
full-world scratch allocation, TE-6 or G9-B/C/D/E work was added.

## Valid evidence
- Runtime receipt:
  `docs/evidence/THERMAL_ENVIRONMENT_TE_5R1_STEAM_LOAD_PRESSURE_2026-08-23.md`.
- Fresh review:
  `docs/adversarial-reviews/TE5R1_STEAM_LOAD_RELAXING_PRESSURE_SOURCE_GATE.md`.
- Final source has 43 passes/86 queries, at most eight storage bindings, one
  new pressure-activity pass and zero new persistent/full-world state.
- Active F01–F21 assertions pass. F21 opens Wood at tick 88, preserves family
  count 24, and real Air uses the opening on following ticks.
- Final-source FULL passed exactly once on `1ee28ac...`. Earlier source
  candidates produced four discovery failures that were fixed and are listed
  in the receipt.
- Release build `1/1`; bounded launch `1/1`; bounded measurement `1/1`.
- Canonical EXE SHA-256:
  `1FE11C518C30F71347442F77BD24D8FADEE9CE4956D6FB1008B222C472040F5D`,
  size `10,187,776` bytes.

## Decided
- Steam alone supplies `100 * phase_energy / 480`; Water target is zero.
- Dynamic pressure is a dissipative local field; Air transport and rupture
  read total pressure exactly once; Matter movement does not read pressure.
- Dedicated pressure activity is the sole exact-update pressure-bit owner.
- Historical Water-yield-two fixtures/evidence remain source-bound and are not
  rebound to this candidate.
- Wiki refresh and PG-L037 promotion remain deferred until 2026-09-01. No Wiki
  remote work occurred.

## Waiting on the user
Q-019 is open. Directly review the four `pressure-vacuum` scenes and decide
whether to accept, revise or reject ADR-0014/TE-5R1. Automated evidence cannot
make that decision.

## Next first action
Run `run_powdergame.bat pressure-vacuum`, inspect Scenes 1–4 and use the fixed
diagnostics. Record only the user's direct disposition; do not start TE-6 or
G9-B/C/D/E.

## Tried
- Verified and preserved the dirty Wiki checkout; no Wiki remote operation.
- Implemented Steam-load pressure, total-pressure Air/rupture coupling,
  opposing-face rupture and sole-owner pressure activity.
- Integrated canonical candidate routes `pressure-vacuum` and `te5`.
- Repaired final-source integration findings in benchmark evidence grouping,
  equilibrium sleep fixtures, historical expansion classification, historical
  pressure diagnostics and the finite free-Air TE-3 candidate sink.
- Passed targeted validation, one final-source FULL, one release build and one
  bounded launch/measurement. G8/G8-C and official capture counts remain zero.
