# TE-3 Sandbox Phase Truth and One-Click Review Remediation — 2026-08-22

- **Disposition:** DIRECT OBSERVATION CONSISTENT / TE-3 USER ACCEPTED WITH KNOWN FOLLOW-UP (D-027)
- **App candidate source:** `89d2400d677dec7e39cba76234c18d8b2363a496`
- **Production-physics source:** `41467219819c5d0cb3eab8ae22b652449da20480` — unchanged
- **Task baseline:** `f1f1a8532fab3e2d7d541562643a9bebee61000f`
- **Decision:** D-025
- **Prior candidate receipt:** [`TE3_DIRECT_REVIEW_SURFACE_REMEDIATION_2026-08-21`](TE3_DIRECT_REVIEW_SURFACE_REMEDIATION_2026-08-21.md)

## Direct-review disposition

The user directly observed Scenes 2–4 and found their visible behavior
consistent with the staged contracts. This is a partial review disposition,
not full TE-3 acceptance.

- Scene 2: **DIRECT OBSERVATION CONSISTENT**. Equal-H surface/buried Water,
  surface-only first completion, buried hold, tick-24 reveal completion and
  family count two were visible. Later EMPTY fixed probes mean the Steam moved
  away; they do not imply quantity loss.
- Scene 3: **DIRECT OBSERVATION CONSISTENT**. Cold-lid progress preceded sparse
  free-Air progress, K=0 remained unchanged, lid Water later cooled below the
  plateau and no whole lane converted in one tick.
- Scene 4: **DIRECT OBSERVATION CONSISTENT**. Both partial reversals, true
  no-sink hold, restored-face wake and eventual Water were visible.
- Scene 1: superseded for fixture/review by source `e9f4a37...`; direct
  observation later closed consistent under D-026.

The Sandbox observation of a large upper Steam cloud, some condensed falling
Water and Water appearing at exactly `100 C` is interpreted in the actual
sealed finite-energy world. It is not evidence that phase progress failed:
temperature may remain at the phase plateau while latent energy changes, and
the sealed Starter Lab has no external ambient heat sink that guarantees all
Steam will eventually condense.

## Fixed 24-byte Inspector profiles

The persistent staging buffer and cadence did not change: exactly 24 bytes,
six four-byte copies, at no more than 10 Hz.

| Offset | Gallery / technical profile | Sandbox product profile |
|---:|---|---|
| 0 | Material `u32` | Material `u32` |
| 4 | Temperature `f32` | Temperature `f32` |
| 8 | Pressure `f32` | Pressure `f32` |
| 12 | Flags `u32` | Flags `u32` |
| 16 | Cell activity `u32` | Phase energy `f32` |
| 20 | Chunk state `u32` | Chunk state `u32` |

The sample owns a typed profile-field enum, and the request identity includes
the profile. Reset/world epoch, hover selection, request generation and profile
must all match before publication. A profile change cancels pending work and
discards the previous sample, so activity bits cannot be relabelled as phase
energy or vice versa. GPU tests read an actual staged Steam phase energy of
`276.0` through the Sandbox profile; Gallery retains the actual activity mask.

Sandbox phase copy uses the shared TE-3 progress formatter:

- Water `E>0`: `Boiling = E / 480`;
- Steam: `Condensing = (480-E) / 480`;
- Water `E<0`: `Freezing = -E / 80`;
- Ice: `Melting = (E+80) / 80`;
- canonical/non-phase state: canonical or `n/a`.

Only active latent progress adds: `Temperature may remain at the phase plateau
while latent energy changes.`

## Candidate and Sandbox presentation

- Scene 2 fixed coordinates are labelled `Surface source probe`, `Buried
  source probe` and `Buried source after opening`; EMPTY after movement is an
  honest coordinate result, not failed object tracking.
- Scene 3 keeps a real 20 C Stone contact face and now connects it to an
  isolated finite 63-Cell Stone heat-capacity block. No scripted temperature
  maintenance or infinite sink was added.
- Sandbox primary tool labels are `Add Heat` and `Remove Heat`; the ±25 detail
  remains only in the brush feedback.
- Sandbox HUD now states `Environment boundary: SEALED` and `External ambient
  heat sink: NONE`.
- Production thermodynamics, constants, Air coefficients and Environment
  boundary mode are unchanged.

## Targeted production semantics

The actual production `Simulation` path produced this strict sequence:

| Observation | Tick/value |
|---|---|
| Cold-lid Steam first latent progress | tick 5 |
| Cold-lid Steam completes to Water | tick 656, `T=99.867607 C`, `E=0` |
| Additional cooling takes Water below plateau | tick 657, `T=98.475067 C` |
| Free-Air Steam first sparse latent progress | tick 63 |
| Free-Air path completes actual Water | tick 1043 at Cell `(132,102)` |
| True no-sink control | unchanged through tick 31 |
| Cooling face restored | tick 32 resumes latent work |

Every observed partial cold-lid Steam sample remained Steam with positive,
monotonically decreasing E and stayed within `0.05 C` of the 100 C latent
plateau. Completion was accepted within `0.25 C`; the next tick demonstrated
post-latent sensible cooling. The no-sink control is metastable by contract,
not a failed condensation attempt.

## Validation and cost boundary

- Inspector profile/layout and actual phase-energy readback tests: passed.
- Sandbox progress/canonical/plateau presentation tests: passed.
- Candidate Scene 2–4, reset, controls and fixed-probe tests: passed.
- Windows affected suite: `173 passed / 0 failed / 1 unrelated ignored`.
- Formatting, affected all-target check and warnings-denied clippy: passed.
- Strict policy audit and `git diff --check`: passed.
- Workspace FULL: `0`.
- Release build: `1`.
- TE-3 candidate bounded launch check: `1`.
- Source changes after release build: `0`.

The change-impact helper recommended considering FULL because Inspector
readback is app-wide. Policy makes that recommendation non-mandatory for an
app/readback change whose risk is closed by targeted tests; Engine/Core/WGSL,
shared world state and buffer allocation did not change. The D-024 final-source
FULL remains bound only to `4146721...` and is not rebound here.

The only bounded launch command was:

```text
target/release/powdergame-windows.exe --phase-cycle-candidate --smoke-frames 60
```

It initialized RTX 5090 / DX12, loaded Scene 1 and exited cleanly after 60
frames and 14 simulation ticks. This does not prove a complete phase cycle.

## Artifact and local shortcut

- EXE: `target/release/powdergame-windows.exe`
- SHA-256: `B22044D1E96AA9EAAED7A66D37DF76FA502FDFE9762BBD4EB19413A260EE9CA8`
- Size: `10,098,688` bytes
- Local shortcut: `C:\Users\mdkap\Desktop\Powdergame TE-3 Phase Cycle.lnk`
- Shortcut target: canonical EXE above
- Arguments: `--phase-cycle-candidate`
- Working directory: `C:\Users\mdkap\source\repos\Powdergame-g8b`

The shortcut was created through the resolved Windows Desktop known folder,
verified by reading its saved COM properties, and is not tracked by Git. No
second executable or launcher was created.

## Historical Scene 1 direct-review checklist

1. Start the Desktop shortcut and leave Scene 1 selected.
2. Confirm Water count never multiplies and family quantity remains stable.
3. Confirm gas-facing Water accumulates boiling progress and becomes 1:1 Steam.
4. Confirm Steam rises through ordinary GAS movement.
5. Confirm cold surfaces/Air produce visible Water that falls toward the beaker.
6. Inspect active Steam/Water in Sandbox and confirm T can remain near 100 C
   while phase energy/progress changes.
7. Confirm the sealed/no-external-sink disclosure explains residual upper
   Steam honestly.
8. Confirm Pressure coupling remains visibly deferred/not active.

TE-5 Pressure redesign remains deferred/not started. TE-4 and G9-B/C/D/E
remain not started.

## 2026-08-22 Scene 1 direct-review failure and remediation

The user reached tick `21,160` with Water `3,885`, Steam `0`, and the surface
probe at `85.824 C / E=0`. The finite heater had equilibrated below boiling;
this receipt's Scene 1 review route therefore failed. App source `e9f4a37...`
supersedes only that candidate fixture and adds an actual ordered 584-tick
boil/rise/condense/fall check. See
[`TE3_SCENE1_PHASE_CYCLE_REMEDIATION_2026-08-22`](TE3_SCENE1_PHASE_CYCLE_REMEDIATION_2026-08-22.md).

## Final direct disposition

D-027 records the complete TE-3 candidate as **USER ACCEPTED WITH KNOWN
FOLLOW-UP** after Scenes 1–4 were directly observed as consistent. This does
not reactivate the failed Scene 1 fixture in this receipt or change any
source-bound validation result. `WATER_STEAM_PRESSURE_VOLUME_REDESIGN` remains
deferred/not started.
