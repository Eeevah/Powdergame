# Checkpoint — TE-2 accepted, TE-3D design authorized — 2026-08-20 22:39 KST

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Session start source: `94b152e85ff6f5481a033d885d38dca0dbc1043a`
- Production TE-2 source: `fb7e568e21012b6067269f4e1b82c36c865023d0`
- Review-remediation source: `097728128343cf89383920c968a010b3dcf8e8c0`
- Working tree: TE-2 docs/memory closure in progress; no runtime source changed

## The story so far

G8 remains **CLOSED / FROZEN**. G9-A and TE-2 are now **USER ACCEPTED WITH
KNOWN FOLLOW-UP**. D-017 records the direct observations: F/N/I work; scene 1
shows Direct > Atmosphere > Vacuum; scene 2 shows sealed Air refill into
connected Vacuum; scene 3 has no external exchange; scene 4 exposes fixed
reservoir exchange; and reset/controls are usable.

TE-2 keeps `LONG_HORIZON_SEALED_AIR_DRIFT_BUDGET` and
`TE2_CANDIDATE_HUD_LABEL_POLISH` as non-blocking follow-ups. No coefficient was
retuned and no further same-tick scene 3/4 comparison is required.

D-017 authorizes docs-only TE-3D work around Hybrid A+C. ADR-0006 remains to be
written as `Proposed`; TE-3 runtime, Air-pressure force, TE-4 and G9-B/C/D/E
remain **NOT STARTED**.

## Valid evidence

- TE-2 production physics remains source `fb7e568...`; review remediation
  remains source `0977281...`.
- Existing focused `10 / 10`, Windows `164 passed / 0 failed / 1 ignored`,
  scene checkpoints, final-source FULL, bounded launch and profiler evidence
  retain only their original source/run scopes.
- This closure is docs/memory-only and ran Cargo/GPU/FULL/build/launch/TE-3/G8
  counts `0`.
- Local personal-infra Wiki remained user-dirty; remote `origin/main`
  `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3` was read without modifying the
  Wiki.

## Decided

- D-017: TE-2 **USER ACCEPTED WITH KNOWN FOLLOW-UP**.
- Hybrid A+C is the required TE-3D candidate to attack, not an accepted
  architecture.
- ADR-0006 must remain `Proposed — user architecture review pending`.
- Runtime, thresholds, Air coefficients, pressure, ignition and later Gates
  remain outside this task.

## Waiting on user

No immediate input is required. The next user decision is accept or revise the
completed TE-3D architecture candidate after independent review and reference
proof.

## Next first action

Draft ADR-0006 and the canonical phase-enthalpy specification against the
audited runtime writer/pass inventory.

## Tried

- Verified clean/upstream-equal start source and preserved the user-dirty Wiki
  via remote-object fallback.
- Reused existing TE-2 evidence without rerunning runtime validation.
- Recorded no new runtime, GPU, build, launch or performance claim.
