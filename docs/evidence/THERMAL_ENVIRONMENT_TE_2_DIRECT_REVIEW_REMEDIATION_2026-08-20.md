# TE-2 Direct-Review Remediation Candidate

- **Status:** REVISED PASSIVE THERMAL ENVIRONMENT CANDIDATE / USER RE-REVIEW PENDING
- **Started from:** `869690b7a282eec203d10df3502bc3451db03779`
- **Production-physics source:** `fb7e568e21012b6067269f4e1b82c36c865023d0`
- **Remediation source:** `097728128343cf89383920c968a010b3dcf8e8c0`
- **Branch:** `feature/m0-g9-first-playable`
- **Direct disposition:** G9-A Inspector continuity USER ACCEPTED; G9-A USER ACCEPTED WITH KNOWN FOLLOW-UP; original TE-2 candidate USER REVIEWED / REVISION REQUIRED

This record covers candidate controls, bounded diagnostics, presentation and
scene staging only. It does not replace the production TE-2 correctness or
performance evidence in
[`THERMAL_ENVIRONMENT_TE_2_PASSIVE_TRANSPORT_2026-08-20.md`](THERMAL_ENVIRONMENT_TE_2_PASSIVE_TRANSPORT_2026-08-20.md).
Engine/Core physics, production compute WGSL, TE-2 coefficients, movement,
phase, ignition and Air-pressure force are unchanged. External simulation code
copied, translated or vendored remains `0 files / 0 lines`.

## Review defect and remediation

The F-key defect was an allow-list omission: the shared fast-forward request
path advertised x1/x4/x16 but did not include `DemoMode::ThermalEnvironment`.
The candidate now uses that same path and cycles `x1 -> x4 -> x16 -> x1`;
window title and HUD use the committed speed, and reset restores x1. Existing
Sandbox, Gallery, G6 and G7 routing remains unchanged.

While paused, every accepted N request is queued and committed in order. Each
request performs exactly one production tick and immediately forces one bounded
TE-2 sample. The window title tick, HUD simulation tick and sample tick agree,
and `STEP APPLIED | TICK n | SAMPLE FORCED` is derived from the committed tick.
N while playing remains ignored. Normal playing sampling remains every eight
simulation ticks, at most 7.5 Hz.

The diagnostic state is explicit `Sampling`, `Fresh` or `Failed`. Requests carry
scene/reset generation, sequence and simulation tick, so late results are
rejected and failures cannot display old rows as current. Reset and scene
selection force a fresh tick-0 sample.

The candidate-only `TE-2 DIAGNOSTICS [I]` panel uses the already collected
fixed bounded rows. I collapses or expands the full row set without hiding the
persistent scene summary. Both states retain fixed panel geometry. The normal
product Cell Inspector remains a 24-byte payload sampled no faster than every
100 ms; its Sandbox/Gallery behavior was not expanded.

## Observable scene contract

Scene 1 now uses large labelled Stone source/target blocks with three distinct
paths: direct Matter contact, a real one-Cell Atmosphere gap, and a real
one-Cell Vacuum gap. Transform-aligned markers and actual sampled values drive
the target thermometers; no presentation value or fake heat motion is
invented.

The production GPU semantic checkpoints at x1 were:

| Tick | Direct target °C | Atmosphere target °C | Vacuum target °C |
|---:|---:|---:|---:|
| 0 | 20.000000 | 20.000000 | 20.000000 |
| 1 | 28.400000 | 20.000000 | 20.000000 |
| 8 | 69.755394 | 20.128620 | 20.000000 |
| 60 | 129.528992 | 24.413996 | 20.000000 |
| 300 | 146.758209 | 46.056732 | 20.000000 |

By the bounded checkpoint, Direct warming is greater than Atmosphere-gap
warming, which is greater than Vacuum-gap Air-mediated warming. The Vacuum gap
sample remains class `Vacuum`, Air mass `0`, and its target remains at the
20 °C initial value in this staging.

- Scene 2 shows four fixed corridor positions plus bounded total Air mass and
  energy. After 64 production ticks, connected Vacuum positions contain
  nonnegative Air while corridor mass and energy remain within the pinned
  tolerances.
- Scene 3 shows Hot Stone and chamber Air temperature, labels `Sealed`, and
  reports zero external mass, advected-energy and passive-heat exchange.
- Scene 4 uses the same comparison geometry, labels the fixture-only Fixed
  Standard Atmosphere Reservoir, and displays cumulative external Air mass,
  advected-energy and passive-heat exchange.
- Every scene reset restores tick 0, its exact staged signature, rows,
  accounting and zeroed cumulative exchange.

## Validation receipts

The first focused TE-2 candidate run passed `7 / 10`; three accounting tests
exposed an attempted 160-Cell readback above the existing 64-Cell bound. The
candidate was corrected to issue fixed batches of at most 64 Cells. The focused
rerun then passed `10 / 10`. The scene-1 semantic test was repeated alone only
to print the numeric checkpoint receipt above; it passed.

- candidate controls and sampling: PASS, including F routing, paused/playing N,
  ordered rapid steps, tick-1 freshness, eight-tick cadence, reset/scene
  generation and explicit failure;
- TE-2 HUD/text/viewport: PASS, including persistent summary, fixed detail
  geometry, transform-aligned markers and actual sampled values;
- Sandbox/Gallery Inspector regression: PASS; payload `24` bytes, cadence
  `>= 100 ms` unchanged;
- Windows package suite: `164 passed / 0 failed / 1 ignored`;
- formatter, affected all-target check, warnings-denied affected clippy,
  development-policy audit and `git diff --check`: PASS;
- validation planner: workspace FULL recommended but not required; FULL `0`;
- G8/G8-C runs `0`; TE-3 runtime runs/changes `0`.

The requested release build was first run directly and passed. The exact
canonical BAT launch later performed its mandatory locked release build again
and recompiled the unchanged Windows package. The honest total is therefore
`2` release-build invocations, not the requested `1`. There was exactly `1`
actual GUI launch. An earlier command was rejected by the BAT argument guard
before any build or EXE launch; the guard was fixed and pinned by the launcher
audit.

The **TE-2 candidate bounded launch check**
`run_powdergame.bat thermal-environment --smoke-frames 60` passed on NVIDIA
GeForce RTX 5090 / DX12 and exited cleanly after 60 frames.

Canonical executable after that launch:

- path: `target/release/powdergame-windows.exe`
- SHA-256: `283fa6c603eb47d3906a14302b183ee8509d9571039bf135e69d922d091d0f00`
- size: `10,034,176` bytes

## Manual TE-2 re-review checklist

1. Launch `run_powdergame.bat thermal-environment`; confirm scene 1 starts
   paused with a nonblank persistent summary and a `Fresh` tick-0 sample.
2. Press F four times and confirm title plus HUD follow
   `x1 -> x4 -> x16 -> x1`; press R and confirm x1, tick 0 and a fresh sample.
3. While paused, press N once. Confirm title tick, simulation tick and sample
   tick are all 1, `STEP APPLIED` is visible, and Direct target temperature has
   changed. Press N rapidly three more times and confirm ordered one-tick
   advances. While playing, confirm N is ignored.
4. Press I repeatedly. Confirm `TE-2 DIAGNOSTICS [I]` expands/collapses without
   moving its panel or hiding scene 1 source/target, boundary and Air/Vacuum gap
   summary values.
5. In scene 1 at x1, observe the actual numeric targets through ticks 8, 60 and
   300. Confirm Direct warms first/most, Atmosphere later/slower, and the staged
   Vacuum target has no Air-mediated warming; confirm Atmosphere and Vacuum are
   visually distinct and the markers align with the labelled lanes.
6. In scene 2, confirm all four fixed positions, corridor Air mass/energy
   totals and visible refill into connected Vacuum; no sample may show negative
   Air mass or disappear into a blank/stale card.
7. In scene 3, confirm Hot Stone and chamber Air temperatures, `Sealed`, and no
   external exchange. In scene 4, confirm the same geometry, the fixture-only
   reservoir label and cumulative mass/advected-energy/passive-heat exchange.
8. Press R in every scene and confirm tick 0, original values and zeroed
   cumulative exchange. Change scenes or reset during sampling and confirm no
   old scene row is presented as current; an actual readback failure must say
   `Failed` rather than retain old values.
9. Record only a TE-2 verdict: accept, revise or reject. Do not infer TE-3
   runtime approval or G9-B advancement.

Until that review occurs, TE-2 is not user accepted.
