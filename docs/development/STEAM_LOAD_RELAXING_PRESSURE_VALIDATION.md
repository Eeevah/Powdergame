# Steam-Load Relaxing Pressure Validation

- **Decision:** D-037
- **Specification:** [Steam-load relaxing pressure](../specs/STEAM_LOAD_RELAXING_PRESSURE_SPEC.md)
- **Status:** LOCAL AUTOMATED CANDIDATE VALIDATION
- **User acceptance:** not claimed

## Evidence rule

Only production Core functions, buffers, shaders and pass ordering qualify.
There is no Python/Rust reference simulator. Direct authoring is used only to
establish a fixture's declared initial state; it does not substitute a phase,
Air, rupture or movement transaction.

## Fixture matrix

| Fixture | Production assertion | Primary automated location |
|---|---|---|
| F01 | standard Atmosphere remains the canonical `P_air=1` equilibrium | `te2_transport::equilibrium_atmosphere_becomes_sleep_eligible`; candidate helper check |
| F02 | exact Vacuum is background zero; dynamic pressure cannot manufacture Air | `te5r1::f01_f02_*` |
| F03 | partial/gas-facing/buried Water target is zero | Core pressure tests; `te5r1::f03_f04_*` |
| F04 | actual Water completion makes one Steam and first target rise is at most `2` | `te5r1::f03_f04_*` |
| F05 | partial Steam target falls continuously | `te5r1::f05_*` |
| F06 | authoritative settled `100` becomes `98`, then `96.04`, with no repeat impulse | `te5r1::f06_*`; source writer-order contract |
| F07 | two-node `100/0` targets hold `(52.38095,47.61905)` and clear pressure activity | `te5r1::f07_*` |
| F08 | one Steam in a large chamber stays below Wood threshold | candidate Scene 1 semantic test |
| F09 | dense small chamber reaches Wood threshold | candidate Scene 4 chain test |
| F10 | uniform opposing pressure leaves Wood intact | `rupture::wood_survives_uniform_*` |
| F11 | one-sided differential creates actual EMPTY | `rupture::wood_ruptures_*`; candidate Scene 3 |
| F12 | opening treatment drops more than sealed control plus `5` | candidate matched Scene 4 test |
| F13 | Air enters/uses the new opening on following Ticks | candidate Scene 4 chain test |
| F14 | Steam uses ordinary Gas movement, with no pressure movement binding | candidate Scene 4 chain; WGSL contract search |
| F15 | departing Matter leaves a bounded dissipating spatial trail; quantity exact | `pressure::void_exit_*`; candidate chain |
| F16 | Air background and dynamic pressure are added exactly once | `te5r1::f16_*` |
| F17 | lowering/removing Steam target leaves no permanent field | `te5r1::f05_*`; pressure stale relaxation |
| F18 | sleep on/off states match for equal executed Ticks | `te5r1::f18_*` |
| F19 | reset/authoring/identity paths keep Current/Next hygiene | Environment/world/phase/combustion writer suites |
| F20 | accepted TE-2, TE-3 and TE-4 behavior remains unchanged | complete affected Environment, phase and combustion suites |
| F21 | Water heat -> Steam -> pressure -> rupture -> opening -> Air/Steam use | candidate Scene 4 chain test |

F06 does not claim that an active M0 material currently has a non-family
yield-two descriptor. It proves the production post-settlement pressure update
from the authoritative `pressure_current` transaction boundary, while source
write-contract tests prove the dormant generic writers remain mutually
exclusive and settle before pressure. Historical Water-yield-two tests remain
ignored and source-bound.

## Static and GPU gates

- all production WGSL parses through Naga;
- every changed pass declares at most eight storage buffers;
- base activity contains no pressure-bit producer;
- dedicated pressure activity contains the only pressure-bit OR and no chunk
  state input;
- movement shaders have no pressure input;
- proposal/claim full-write and scratch live ranges pass the production write-
  contract test;
- profiler names, pass count, query bytes and total tracked allocation are
  exact.

## Candidate contract

Routes `pressure-vacuum` and `te5` select the same canonical executable and
`--pressure-vacuum-candidate`. It starts paused and preserves SPACE/N/F/R/I,
1-4 and ESC. Every fixed row reports Material, phase energy, Steam target,
dynamic/background/total pressure, predicted delta, opposing-face differential,
Air mass/energy, sample Tick and freshness.

Required labels are always visible:

```text
Pressure model: LOCAL RELAXING APPROXIMATION
Matter pressure force: NOT ACTIVE
Oxygen quantity: NOT PRESENT
World Matter edge: VOID EXIT
Fixture chamber: EXPLICITLY SEALED
```

## Final validation sequence

Targeted suites precede formatting, workspace all-target check, warnings-denied
clippy, strict audit and diff checks. Freeze the runtime source before running
the canonical FULL exactly once. Then build the canonical release executable
and perform exactly one 60-frame bounded candidate launch, whose final line is
the one bounded semantic/performance measurement. No G8/G8-C, official capture,
Wiki remote, PR or user-acceptance action is part of this validation.
