# Checkpoint — Thermal Environment design locked; TE-1 ready and not started — 2026-08-20 13:10 KST

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- HEAD: `2591dd5196752ca0caa4a69029dd04a9eee76744` docs architecture commit; live Git wins if this checkpoint is stale
- Working tree: expected Ballast closure files only until the memory commit; final target is clean and upstream-equal

## The story so far

G8 is closed/frozen. G9-A continuity v2 remains **REVISED IMPLEMENTATION
CANDIDATE / USER RE-REVIEW PENDING** at runtime source
`a00e39b2e00bfbd9ac28214c44cd22cc97542bb4`; no user acceptance was inferred.

D-013 closes the Thermal Environment ontology before code. Foreground EMPTY,
Atmosphere, Vacuum and Void are distinct. Air is a full-resolution mass/energy
Environment Field, not Matter. The correctness baseline has four Air
Current/Next buffers plus one `u32` receiver-claim scratch. Occupancy changes
use bounded reconcile/joint settle and the eight-storage ceiling. The
Celsius-like product anchors migrate atomically in a later gate.

TE-0R reuse research, TE-0 canonical design, TE-0A independent review and
TE-0B reference math are complete. Independent review first found seven High
defects; the final recheck confirmed Critical 0 / High 0 / blocker 0 after the
design locked transactional spawn displacement, whole-parcel headroom,
exact-zero Vacuum, coefficient domains, bilateral sleep/wake cohorts, flag
hygiene and a sealed TE-2 reference edge. Production runtime is unchanged.

## Valid evidence

- TE-0 docs source `2591dd5196752ca0caa4a69029dd04a9eee76744` — valid while the ADR/spec/inventory/review files
  remain unchanged; strict policy audit, Markdown/link/index/JSON/secret checks
  and diff-check passed.
- TE-0.2 reference proof — package SHA-256
  `3f5abcc282881a190a0881499b61f8d00cf782f305354f6b533740ac0d0b9c84`,
  seed `1413820466`, 40,000 randomized trials, maximum numerical error
  `7.275957614183426e-12`, result `PASS_REFERENCE_MATH_ONLY`. It does not prove
  WGSL, GPU ordering/races, bindings, sleep, performance, fun or acceptance.
- G9-A continuity v2 runtime evidence remains attached to source
  `a00e39b2e00bfbd9ac28214c44cd22cc97542bb4`; this docs/memory work does not
  invalidate or extend it.

## Decided

- D-007 — G8 is closed/frozen.
- D-010 — canonical no-argument launch opens Sandbox; Gallery stays explicit.
- D-012 — G9-A re-review still requires continuity v2; no acceptance verdict.
- D-013 — separate foreground Matter from atmospheric/vacuum Environment and
  adopt Air mass/energy plus reuse-first design.

## Waiting on the user

- Re-review G9-A continuity v2 and accept, revise or reject it.
- Authorize TE-1 as a separate implementation task if desired.
- Q-008 decisions remain later-gate owned and do not block TE-1.

## Next first action

On an explicit TE-1 task, verify the clean pushed docs source, then open
`docs/planning/THERMAL_ENVIRONMENT_IMPLEMENTATION_GATES.md` and execute only
the TE-1 entry checklist; do not start TE-2.

## Tried

- The package's proposed no-scratch spawn receiver was rejected because the
  original ownership claim remains live. The locked design pays one explicit
  `u32` scratch and uses a paired commit/block transaction.
- Independent review initially rejected TE-1 with seven High findings. A first
  revision left receiver-failure phase pressure ambiguous; the final revision
  pinned a mandatory seven-storage pass, settle order and exactly-once fixture.
- The local personal-infra Wiki checkout had user changes and could not
  fast-forward safely. Remote `origin/main` object `b8c22c1...` was fetched and
  read as authority; no Wiki file was modified.
- No Rust, WGSL, Cargo, fixture, runtime, build, GPU, FULL, bounded launch,
  candidate or capture command ran in TE-0.
