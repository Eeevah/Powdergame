# Ignition Kinetics Validation Contract

Status: **V2 PROCESS COMPLETED / REVIEW INVALIDATED BROAD PASS / DESIGN BLOCKED**.

## Fixture matrix

| Fixture | Required future execution |
|---|---|
| TE4-F01 | One-Air-gap Stone heating via actual TE-2; exposure starts only at own threshold. |
| TE4-F02 | Direct Stone contact; separate bounded Oil/Wood onset. |
| TE4-F03 | 1/2/3-tick threshold spikes cool without ignition and decay. |
| TE4-F04 | Sustained source records exact onset/ignition, never first-tick. |
| TE4-F05 | Partial dose survives brief cooling, resumes, and clears after long cooling. |
| TE4-F06 | Matched-temperature previous-flame route is faster but finite. |
| TE4-F07 | A newly ignited line Cell cannot affect the next until a later tick. |
| TE4-F08 | 2-D connected fuel ignites locally; no region-wide event. |
| TE4-F09 | Actual burn ticks report gross/deposited/clipped Q separately, cap behavior, extinguish/consumption zero and no TE-2 double source. |
| TE4-F10 | Extinguish/reignite preserves fuel, restarts exposure at zero. |
| TE4-F11 | Movement/swap transfers exposure and fuel without residue/duplication. |
| TE4-F12 | Decay/rupture/consumption/Void clear exposure and preserve unrelated flags. |
| TE4-F13 | Draw/Erase/preset/reset produce canonical Current/Next; invalid staging is atomic. |
| TE4-F14 | Thermal/flame frontier wake, exposure-decay runnability, zero sleep and sleep equivalence. |
| TE4-F15 | Both Vacuum policies documented; only a user-selected one executes. |
| TE4-F16 | CPU/GPU equality for eligibility, delta, ignition tick, decay, flags and Q. |
| TE4-F17 | TE-2/TE-3 unchanged; no Pressure or historical-evidence rebind. |

`NO_REGION_WIDE_INSTANT_COMBUSTION` maps to TE4-F07 and TE4-F08.

## Frozen D-028 attempt

- Script: `../reference/te4_ignition_kinetics_reference.py`
- Script SHA-256: `886fe5b7d1f59c2d53856f079067936fcc60bb8b4a6d742fd934256696470f82`
- Seed: `0x54453444`
- Planned minimums: 100,000 single-Cell sequences, 10,000 grids, 17 fixtures
- Attempts/completions: `1 / 0`
- Executed sequences/grids/fixtures: `0 / 0 / 0`
- Failure: `AssertionError: frozen selected coefficient mismatch: Oil.bucket_width`
- Failure receipt: `../reference/te4_ignition_kinetics_failure.json`
- Receipt SHA-256: `6342bad5cce21cd5dff03dfb4c5e4aadb39c181d50dad81431d5eed92b62c1bb`

The smallest reproduction is a coefficient-metric tie: Oil
`48/2/25/4/1/2` and `48/2/50/6/1/2` both yield
threshold/high/flame/half-decay `24/12/12/24`; lexicographic ordering selects
the former before the frozen expected latter. The result JSON was never
created. Mathematical, representation, coefficient and fixture results are
all `NOT_ESTABLISHED`; GPU/product/user results remain unknown/pending.

## Future validation boundary

A new user-authorized evidence identity must separate coefficient identity
from equal-metric tie order during preflight, then run the full matrix once.
Production validation later requires Core tests, actual WGSL/Naga/binding and
writer tests, GPU fixtures, sleep/wake equivalence, targeted TE-2/TE-3
regression, profiler/allocation evidence and direct product review. None ran in
this docs-only task.

## D-029 manifest-bound v2 receipt

The immutable v1 files and `1/0` receipt above were neither modified nor
rerun. D-029 created the distinct identity
`TE4-IGNITION-KINETICS-REFERENCE-V2`.

- Manifest: `../reference/te4_ignition_kinetics_v2_manifest.json`
- Manifest SHA-256: `9b763c1c7efa0ee9f9d444ef19dc5daed3833aafb546612b01d4e9db48d253ba`
- Script: `../reference/te4_ignition_kinetics_v2_reference.py`
- Script SHA-256: `c01e28690fa7b2a6b2c9f24e5af07f776db031f7ca3d4c9cfe27c7b4be79a769`
- Result: `../reference/te4_ignition_kinetics_v2_result.json`
- Result payload SHA-256: `717f4ef7f339a12a4f135c7ccbf31d0d10d41763a66fc513ac8d42422dcc132c`
- Result file SHA-256: `24ebd7974969087de09ee2353d80696be96fbdb34c65894f51e9dbbf3918f151`
- Seed: `0x54453444`; attempts/completions: `1/1`
- Coverage: 100,000 single-Cell sequences, 10,000 bounded grids, one matching in-process replay
- Fixtures: 13 `REFERENCE_REQUIRED` PASS; exactly four `PRODUCTION_DEFERRED` `NOT_ESTABLISHED`; failures 0; unexpected `NOT_ESTABLISHED` 0

Preflight used Python 3.12.13 with syntax compilation, import validation,
manifest parse/hash, fixture/class/path listing, exact coefficients/profiles and
output-destination inspection. It executed no randomized campaign or evidence
fixture. The evidence command was:

```powershell
& 'C:\Users\mdkap\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' -B `
  docs/reference/te4_ignition_kinetics_v2_reference.py `
  --manifest docs/reference/te4_ignition_kinetics_v2_manifest.json `
  --result docs/reference/te4_ignition_kinetics_v2_result.json `
  --failure docs/reference/te4_ignition_kinetics_v2_failure.json
```

F07 produced exactly `(20,1)`, `(40,2)`, `(60,3)`, `(80,4)`. F08 produced
first ignition tick 20, maximum five new ignitions in one tick and completion
tick 173; deterministic digest
`072d8e25d36e8120f9fa99fcac6eae3b91885538f57affa40d6d2e4f7124f423`.
F09 recorded Oil `599 / 8,985` and Wood `899 / 7,192` emitting-tick/gross-Q
totals, consume-tick zero, and finite deposited+clipped closure. F15 accepted
Atmosphere and positive LowPressure, rejected exact Vacuum and occupied Steam,
emitted zero on access loss, preserved Air and resumed dose after access
returned.

Every declared required path counter is positive. The named ownership fixtures
execute actual state transactions: partial/reversal history, burning/fuel/Q,
extinguish/reignite, move/swap, five replacement paths, authoring/reset and the
Air policy matrix. The result deliberately says `PASS_REFERENCE_MODEL_ONLY`;
GPU, product and user status remain `NOT_ESTABLISHED`, `NOT_ESTABLISHED` and
`PENDING`.

Fresh review does not alter or rerun the receipt, but rejects its broad
fixture/state-transition disposition. Several counters are positive constants
rather than mutation-derived proof; F08 compares a post-run digest only with
the same implementation's replay; and F15 omits Smoke occupying the sole Air
face later in the same tick. The reported 13 PASS remains historical process
output, not accepted design evidence. See the v2 adversarial review.
