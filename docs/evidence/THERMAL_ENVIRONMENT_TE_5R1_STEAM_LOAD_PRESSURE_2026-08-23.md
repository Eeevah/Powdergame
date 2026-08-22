# TE-5R1 Steam-Load Relaxing Pressure — Runtime Evidence

- **Date:** 2026-08-23
- **Decision:** D-037
- **Baseline:** `12b49dc07c8d875de55a048013a01090d38345a9`
- **Final runtime source:** `1ee28ac2003d3e2804dfce5fbf0fa25e583e3030`
- **Architecture:** [ADR-0014](../architecture/decisions/ADR-0014-post-phase-steam-load-relaxing-pressure.md)
- **Specification:** [Steam-load relaxing pressure](../specs/STEAM_LOAD_RELAXING_PRESSURE_SPEC.md)
- **Verdict:** **IMPLEMENTATION CANDIDATE / AUTOMATED VALIDATION PASS / USER REVIEW PENDING**
- **External implementation copied, translated or vendored:** `0 files / 0 lines`

## Scope and evidence boundary

This receipt binds the TE-5R1 implementation and local validation to the final
runtime source above. It does not accept ADR-0014, claim direct user review,
rebind historical G5 Water-yield-two evidence, add Matter pressure force, or
start TE-6/G9-B/C/D/E.

TE-5R0/ADR-0013 remains blocked immutable history. Water phase completion is
still 1:1, emits no expansion proposal and creates no blocked-pressure
impulse. Only the settled Steam identity and its accepted phase energy supply
the TE-5R1 load target.

## Implemented contract

- Dynamic-pressure nodes are EMPTY and Liquid/Gas Matter. Static, Powder and
  Void faces are no-flux.
- Standard EMPTY Air contributes background pressure `1`; exact Vacuum
  contributes `0`.
- Steam target is `100 * phase_energy / 480`; Water and all other identities
  target `0`. Invalid Steam phase state fails closed and authoritative staging
  rejects it.
- Each tick applies diffusion `0.20` and target relaxation `0.02` after the
  existing generic pressure consequence settlement. A target-zero `100`
  becomes `98`, then `96.04`; no second impulse is generated.
- Air transport and rupture read dynamic plus Air background exactly once.
  Matter movement has no pressure binding.
- Wood rupture reads the maximum left/right or up/down total-pressure
  differential. Uniform opposing pressure does not rupture.
- A dedicated full-world pressure-activity proposal is the sole producer of
  the pressure activity bit and compares the exact production update.

## Pass, binding and allocation result

- Production graph: `43` passes, `86` timestamp queries.
- Profiler timestamp storage: two `688`-byte buffers, `1,376` bytes total.
- Maximum changed-pass storage bindings: `8`.
- Pressure: `6`; Air scale: `7`; Air commit: `8`; base activity: `7`;
  Environment activity: `6`; pressure activity: `5`; rupture: `8`.
- Persistent buffers added: `0`.
- Full-world scratch allocations added: `0`.
- Compute passes added: `1` (`pressure_activity_propose`).
- Candidate 256x192 world state: `2,949,120` bytes; activity scratch:
  `196,896` bytes.
- Smoke-complete proposal/claim scratch lifetimes remain fully overwritten
  before Air scale/commit reuse.

## Production fixture result

The complete F01–F21 mapping is in the
[validation contract](../development/STEAM_LOAD_RELAXING_PRESSURE_VALIDATION.md).
All active production assertions passed:

- F01/F02: canonical Atmosphere background `1`, exact Vacuum `0`, no Air
  manufacture from dynamic pressure.
- F03/F04: Water target `0`; actual 1:1 Water completion creates one Steam and
  a first load rise no greater than `2`.
- F05/F17: partial Steam target and target removal relax continuously without
  a permanent field.
- F06: authoritative pre-pressure `100` settles to `98`, then `96.04`, with no
  repeated source. No active M0 non-family yield-two descriptor is claimed.
- F07: the two-node target equilibrium is approximately
  `(52.38095, 47.61905)` and produces no pressure activity.
- F08–F14: sparse load remains below Wood threshold; dense load crosses it;
  uniform opposing faces survive; a one-sided differential creates real EMPTY;
  the matched opening drops pressure by more than the sealed control plus `5`;
  following ticks use real Air and ordinary Gas movement.
- F15–F18: a departing source leaves only a bounded relaxing trail; Air plus
  dynamic pressure is counted exactly once; sleep on/off states agree.
- F19/F20: Current/Next authoring and accepted TE-2/TE-3/TE-4 regressions pass.
- F21 actual candidate chain opened Wood at tick `88`, preserved phase-family
  count `24`, and changed opening Air from `0` to `0.004166` on following
  ticks. No token, extra Steam spawn, fake vent or boiler-specific explosion
  is present.

The TE-3 candidate's free-Air lane required an integration repair after total-
pressure Air transport became active: it now owns a finite K=0-enclosed,
dense 0 C Air reservoir. Production ticks show free-Air nucleation at tick
`40`, Water completion at tick `358`, cold-lid partial progress at tick `5`
and lid completion at tick `656`. The K=0 Boundary control remains unchanged.

## Validation receipt

Targeted Core, pressure, rupture, Environment, phase, combustion, activity,
sleep/wake, Naga/binding/write-contract, profiler, candidate controls and
diagnostic suites passed. Notable complete suites include phase `17/17`,
combustion `64/64`, candidate Scene tests `7/7`, and Windows `190 passed / 1
manual long-run ignored`.

Final-source canonical FULL:

```text
command: cargo test --workspace -- --test-threads=1
source:  1ee28ac2003d3e2804dfce5fbf0fa25e583e3030
result:  PASS
successful final-source FULL count: 1
```

Four earlier source candidates failed discovery FULL runs and were not reused
as PASS evidence:

1. `4988551...`: benchmark evidence group omitted the new pressure-activity
   pass and retained a stale final tick value.
2. `34cc3f7...`: a thermal-only activity fixture staged canonical Steam below
   its new pressure equilibrium.
3. `163a54b...`: two active tests still attempted the historical Water-yield-
   two Environment expansion path.
4. `735922d...`: the stable-Steam sleep fixture likewise staged a real pressure
   disequilibrium. A subsequent Windows preflight also exposed the historical
   pressure-medium diagnostic alias and finite free-Air Scene 3 sink issues.

Each failure was resolved before the final source was frozen. The successful
FULL was then run once on the final source.

Other final checks:

- `cargo check --workspace --all-targets`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- strict development-policy audit: PASS.
- changed Markdown links/fences: PASS.
- secret-pattern scan: PASS.
- `git diff --check`: PASS.
- G8/G8-C: `0`; official capture: `0`; user acceptance claim: `0`.

## Canonical candidate artifact

```text
path:    target/release/powdergame-windows.exe
size:    10,187,776 bytes
SHA-256: 1FE11C518C30F71347442F77BD24D8FADEE9CE4956D6FB1008B222C472040F5D
```

Exactly one release build, one bounded launch and one bounded measurement ran:

```text
run_powdergame.bat pressure-vacuum --smoke-frames 60
frames=60 ticks=15 wall_tps=57.25 sample_tick=8
family=170 water=0 steam=170 wood=0 rows=4
```

The run exited cleanly on NVIDIA GeForce RTX 5090 / DX12. This is a bounded
candidate launch and measurement, not proof of all four visual scenes.

## Direct review checklist

1. Scene 1: the large sparse chamber stays visibly milder than the small dense
   chamber; the sparse control does not falsely rupture Wood.
2. Scene 2: condensation/removal of Steam load reduces dynamic pressure more
   than its matched no-condensation control.
3. Scene 3: uniform opposing total pressure leaves Wood intact, while the
   one-sided differential creates a real EMPTY opening.
4. Scene 4: Water heats and completes 1:1 to Steam, dense Steam load raises
   pressure, Wood ruptures, and Air/Steam use the real opening.
5. Fixed rows show Material, phase energy, Steam target, dynamic/background/
   total pressure, predicted delta, differential, Air, tick and freshness.
6. Reset is exact and no stale row crosses scene generation.
7. Labels remain explicit: local relaxing approximation, no Matter pressure
   force, no Oxygen quantity, Void Matter edge and explicitly sealed fixtures.

Until that review is supplied, TE-5R1 remains **IMPLEMENTATION CANDIDATE /
AUTOMATED VALIDATION PASS / USER REVIEW PENDING** and ADR-0014 remains
**PROPOSED**.
