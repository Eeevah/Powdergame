# Thermal Environment TE-2 Passive Transport Evidence — 2026-08-20

## Disposition

- Runtime source: `fb7e568e21012b6067269f4e1b82c36c865023d0`
- State: **PASSIVE THERMAL ENVIRONMENT CANDIDATE / USER REVIEW PENDING**
- TE-3 and G9-B/C/D/E: **NOT STARTED**
- G9-A: **REVISED IMPLEMENTATION CANDIDATE / USER RE-REVIEW PENDING**
- Workspace FULL: exactly one successful Cargo execution at the final runtime source
- Release bounded launch: exactly one, 60 frames, exit `0`
- G8/G8-C reruns: `0`

Automated evidence establishes the TE-2 correctness and bounded-performance
contract. It does not establish product acceptance or authorize TE-3.

## Implemented contract

TE-2 adds full-resolution, every-tick pressure-derived Air mass flow,
donor-specific-energy advection, unified Matter↔Matter / Air↔Air / Matter↔Air
passive thermal exchange, bilateral activity/wake, sealed production edges and
an explicit fixed-reservoir semantic fixture. Air remains a separate
Environment field and does not become Matter. Atmospheric derived pressure is
still not coupled to Matter or structure force.

The atomic Celsius-like gameplay migration is included in the same runtime
source: room reference `20 °C`, Water/Ice anchors around `0 °C`, Water/Steam
anchors around `100 °C`, finite safety range `-250..2000 °C`, phase hysteresis,
stable Sandbox placement defaults and migrated combustion thresholds/heat.

The thermal deadband is a work/no-work gate, not a subtractive flux term:

```text
abs(delta) <= 0.01 °C  => face flux 0
abs(delta) >  0.01 °C  => effective_delta = delta
```

Physics and thermal activity use that identical predicate. Stability remains
bounded by the existing `lambda` and `THERMAL_MAX_MIX_FRACTION` rules.

## SMALL_DELTA_THERMAL_CONVERGENCE

CPU reference and production GPU fixtures cover 30 combinations:

- Matter↔Matter, Air↔Air and Matter↔Air;
- baselines near `20 °C` and `500 °C`;
- initial deltas `1.0`, `0.1`, `0.02`, `0.011` and `0.009 °C`.

For every `delta > 0.01 °C`, the first eligible tick performs nonzero stored
work, temperatures converge monotonically without extrema overshoot or hot/cold
ordering reversal, thermal activity remains present while work exists, and the
bounded run enters the deadband within 4096 ticks. `0.009 °C` permits no work
from the first tick. Both absolute-temperature baselines have the same dead
zone. Sleep-on/off reaches the same semantic result, source-free energy-like
accounting and the maximum principle hold, and CPU/GPU are compared by semantic
predicate plus tolerance rather than exact bit equality.

The implementation contains a directed next-representable-f32 safeguard only
when a genuine nonzero eligible flux would otherwise round to no stored state
change. It does not clamp output or manufacture activity without a state
transition.

## State, passes and memory

- Production pass count: `34`
- Timestamp query count: `68`
- New dense TE-2 scratch: `0`; existing proposal/claim scratch is reused only
  after its previous lifetime ends
- Persistent tracked bytes without profiler: 256×256 `4,197,040`; 2048×2048
  `268,462,384`
- Persistent tracked bytes with profiler: 256×256 `4,198,128`; 2048×2048
  `268,463,472`
- Product Inspector remains `24` bytes and at most `10 Hz`
- Candidate sampling cadence is every 8 simulation ticks, at most `7.5 Hz`

## Validation

Passed at the final runtime source:

- Core `144 / 144`; benchmark `28 / 28`; scenario fixture/reset `10 / 10` and
  `3 / 3`; Windows `156 / 156`
- TE-2 candidate `6 / 6`; transport `5 / 5`; small-delta production GPU `1 / 1`
- GPU integration suites serial, workspace check, warnings-denied clippy,
  formatter, strict policy audit and diff check: PASS
- final-source workspace FULL serial: PASS
- locked release build and canonical release EXE 60-frame bounded candidate
  launch: PASS / exit `0`

Release executable:

- Path: `target/release/powdergame-windows.exe`
- SHA-256: `e1f7e9b3428fbd40f8a3030cb302d8691a28383b494336d6f822be79b9f66512`
- Size: `10,000,896` bytes

## One-shot performance measurement

Artifact: `C:\Users\mdkap\source\Powdergame-artifacts\te2-performance-fb7e568-20260820.csv`

- SHA-256: `f67c058ba0bf41cee0d108f66c9e4599ecf03b06cbc46714ff866dea7c4b5658`
- Size: `934` bytes
- Profiled ticks per row: `32`, after an 8-tick warm-up

| Row | GPU tick P50 / P95 | synchronized wall ms/tick | terminal active Cells / chunks | terminal sleeping chunks |
|---|---:|---:|---:|---:|
| 256² local candidate | 0.456000 / 0.459136 ms | 0.968659 | 157 / 4 | 0 |
| 2048² equilibrium | 2.287168 / 2.599712 ms | 3.001841 | 0 / 0 | 1024 |
| 2048² local frontier | 2.292256 / 2.304832 ms | 3.158381 | 157 / 4 | 1008 |

The equilibrium row reaches zero active Cells/chunks, and the local frontier
does not create a broad active tail. All profiled GPU P95 values are below the
16.667 ms 60-TPS frame budget. This is a narrow synchronized headless pass-cost
measurement, not a Mode C window-responsiveness capture; user-visible scene
quality and interaction responsiveness remain part of direct review.

## Candidate surface and limits

The canonical `powdergame-windows.exe` adds explicit
`--thermal-environment-candidate`, routed by `run_powdergame.bat
thermal-environment` with compatibility alias `te2`. No-argument Sandbox and
all existing explicit modes remain unchanged. The four bounded scenes expose
Atmosphere versus Vacuum transfer, connected Vacuum refill, sealed cooling and
fixed-reservoir cooling.

The following remain open and are not silently selected: product edge mode,
Vacuum combustion support, phase latent/yield representation, GAS permeability
unless TE-F33 proves a blocker, and any future cadence/packing optimization.
Air-pressure force, TE-3, TE-4, G9-B/C/D/E, Oxygen, Ash, new Matter, CFD and
optimization are not started.
