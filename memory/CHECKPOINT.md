# Checkpoint — revised TE-2 candidate awaiting direct re-review — 2026-08-20

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Start source: `869690b7a282eec203d10df3502bc3451db03779`
- Production TE-2 source: `fb7e568e21012b6067269f4e1b82c36c865023d0`
- Candidate remediation source: `097728128343cf89383920c968a010b3dcf8e8c0`
- Final target after the docs/memory commit: clean and upstream-equal

## Current truth

G8 remains **CLOSED / FROZEN**. Inspector continuity v2 is **USER ACCEPTED**
and G9-A overall is **USER ACCEPTED WITH KNOWN FOLLOW-UP**.

Direct review classified the original TE-2 candidate **USER REVIEWED /
REVISION REQUIRED** because F did not operate, N did not expose a fresh
one-tick result, I was unavailable, and temperature/Air measurements were not
usable. D-016 preserves D-015's automated/runtime/performance evidence and
authorizes only candidate controls, bounded diagnostics and staging.

Source `0977281...` fixes F x1/x4/x16 routing, ordered paused N with forced fresh
sampling, explicit Sampling/Fresh/Failed generation-safe diagnostics,
candidate-only `TE-2 DIAGNOSTICS [I]`, persistent actual-value summaries and
legible fixed-bounded scenes. It changes no Engine/Core physics, production
compute WGSL, TE-2 coefficient, phase, movement or ignition rule.

TE-2 is **REVISED PASSIVE THERMAL ENVIRONMENT CANDIDATE / USER RE-REVIEW
PENDING**, not accepted. TE-3 is **DESIGN REQUIRED / NOT STARTED** because the
current available-space round trip is `1 Water -> up to 2 Steam -> up to 2
Water`. Air-pressure force, TE-3 runtime, TE-4 and G9-B/C/D/E are **NOT
STARTED**.

## Valid evidence

- Focused candidate rerun `10 / 10`; Windows package suite `164 passed / 0
  failed / 1 ignored`; formatter, affected check, warnings-denied clippy,
  strict policy audit and diff check passed.
- Scene-1 production checkpoints at ticks `0/1/8/60/300` establish Direct >
  Atmosphere > Vacuum Air-mediated warming by the bounded checkpoint.
- Workspace FULL `0`; G8/G8-C `0`; TE-3 runtime `0`.
- One actual 60-frame TE-2 bounded launch passed on RTX 5090/DX12. The
  canonical BAT's mandatory build caused two release-build invocations overall;
  this deviation is recorded rather than relabelled.
- Canonical EXE SHA-256
  `283fa6c603eb47d3906a14302b183ee8509d9571039bf135e69d922d091d0f00`,
  size `10,034,176` bytes.
- `docs/evidence/THERMAL_ENVIRONMENT_TE_2_DIRECT_REVIEW_REMEDIATION_2026-08-20.md`.
- `docs/planning/TE3_WATER_STEAM_PHASE_ACCOUNTING.md`.

## Waiting on the user

Directly re-review the revised four-scene TE-2 candidate using the checklist in
the remediation evidence and record accept, revise or reject for TE-2 only.

## Next first action

Run `run_powdergame.bat thermal-environment` for direct TE-2 re-review. Do not
start Air-pressure force, TE-3 runtime, TE-4 or G9-B/C/D/E.

## Preserved boundary

The local personal-infra Wiki checkout remained user-dirty and no Wiki file was
modified. The requested The Powder Toy provenance identity was already correct
at `2e47966b84b0d2f1750af0f82643791803537ea5`, so no duplicate edit was made.
External simulation code copied/translated/vendored remains `0 files / 0
lines`.
