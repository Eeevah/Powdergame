# Ignition Kinetics Validation Contract

Status: **PREDECLARED / REFERENCE EXECUTION INCOMPLETE**.

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
