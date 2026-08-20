# Thermal Environment TE-1 foundation evidence — 2026-08-20

## Identity and scope

- Branch: `feature/m0-g9-first-playable`
- Start: `5d580a9d2a3844fd0ed288ee539fff86e80913c2`
- TE-0 architecture source: `2591dd5196752ca0caa4a69029dd04a9eee76744`
- Runtime source: `1a722d239a16bade5772688fa822465d5cef4602`
- Result: **ENVIRONMENT STATE / OCCUPANCY HYGIENE IMPLEMENTED**
- Air transport / Air thermal exchange / Air pressure coupling: **NOT STARTED**
- TE-2 and G9-B/C/D/E: **NOT STARTED**

No external simulation source was copied, translated or vendored: **0 files /
0 lines**. Existing Current/Next buffers, claim arbitration, staging, profiler,
Naga structural validation, bounded readback and wgpu 26 APIs were extended.
`proptest` was not added: the existing deterministic Core/GPU fixture system
covered the locked domains without a new dependency or routine-time cost.

## Locked state

```text
STANDARD_AIR_MASS       1.0
AIR_HEAT_CAPACITY       1.0
AIR_ZERO_OFFSET         273.15
AMBIENT_TEMPERATURE_C   20.0
AMBIENT_TEMPERATURE_ABS 293.15
STANDARD_AIR_ENERGY     293.15
VACUUM_THRESHOLD        0.0
AIR_PRESENT_THRESHOLD   0.5
AIR_MASS_MAX            16.0
AIR_TEMPERATURE_ABS_MIN 1.0
AIR_TEMPERATURE_ABS_MAX 2273.15
AIR_ENERGY_MAX          36370.4
```

Exact `(0,0)` is Vacuum. Positive finite Air is never silently rounded away.
Invalid paired, non-finite, negative, over-limit and invalid specific-energy
states are rejected rather than clamped into evidence.

## GPU state, pass graph and allocation

Added four persistent full-resolution f32 buffers:

- `air_mass_current`, `air_mass_next`
- `air_energy_current`, `air_energy_next`

Added one reusable full-resolution u32 scratch:

- `environment_receiver_claim`, encoded as `target_cell + 1`; zero means none

No second Environment scratch was added. All production buffers use STORAGE
and the existing required COPY source/destination usages; bounded observation
maps a separate short-lived staging buffer only.

The production tick is now 30 explicitly profiled passes. New work consists of
five flag-hygiene placements, movement/phase/decay/Smoke/rupture Environment
reconciles, two uses of the receiver-claim pass, and one exactly-once
Environment-blocked expansion-pressure pass. Every bind group remains at or
below eight storage buffers. Profiling uses 60 raw timestamp queries; every
pass belongs to exactly one non-overlapping group.

| World | Four Air buffers | receiver scratch | tracked no profiler | tracked profiled |
|---|---:|---:|---:|---:|
| 256² | 1,048,576 B | 262,144 B | 4,196,864 B | 4,197,824 B |
| 2048² | 67,108,864 B | 16,777,216 B | 268,462,208 B | 268,463,168 B |

## Occupancy and transaction results

- Initial/reset/direct/scenario/benchmark/Starter Lab/Blank World staging uses
  one canonical material-to-Environment image and writes identical halves.
- Movement transfers the destination Air parcel to the vacated source;
  density swaps keep both occupied Cells at zero Air; Void exits expose Vacuum.
- Matter identity replacement keeps Air zero and clears every flag not owned
  by the target. Oil/Wood own bits 0–1 and 4–15; Smoke owns bits 16–27;
  bits 2–3 and 28–31 are zero.
- Phase and Smoke spawns retain the original Matter claim, deterministically
  claim at most one orthogonal EMPTY receiver, exclude winning Matter targets,
  require whole-parcel headroom and commit Matter/Air as one gated transaction.
- A failed phase receiver leaves target and Air unchanged and receives the
  existing blocked-expansion pressure consequence exactly once. A failed
  Smoke receiver rejects only that optional spawn and adds no pressure.
- Decay, fuel consumption and rupture expose Vacuum. Sandbox accepted Draw
  zeros both Air halves, Erase seeds standard Atmosphere, and Heat/Cool does
  not modify Environment.

## Validation

Targeted validation passed for Core Environment math, canonical staging/reset,
Naga/write/binding structure, Environment GPU semantics, phase/Smoke receiver
transactions, Sandbox edits/reset, shared scenario/benchmark staging,
Inspector 24-byte/10-Hz regression, 30-pass profiler reconstruction and exact
allocation totals. Workspace all-target check, warnings-denied clippy, strict
development-policy audit and diff checks passed.

The first workspace FULL attempt was invalidated after it found a pre-existing
Heavy Mixed census assertion block misplaced inside the Fire/Heat fixture test.
Only that stale test block was removed; production fixture bytes were not
retuned. Scenario tests then passed `10/10`, and the final runtime SHA passed
the canonical serial workspace FULL. Recorded FULL attempts: **2**; final-SHA
valid successful FULL: **1**.

One locked release build passed. One Sandbox bounded launch check using
`run_powdergame.bat sandbox --smoke-frames 3` passed on RTX 5090 / DX12,
Starter Lab staged at tick 0 and exited cleanly after three frames.

- EXE: `target/release/powdergame-windows.exe`
- size: `9,936,896` bytes
- SHA-256: `8c3f0050eef67cfca04e970c071276ce8ae856a7a1a65e58ff63a0deecb34ea6`
- G8/G8-C/candidate/official capture count: `0`

## Evidence boundary

This record proves TE-1 state and occupancy contracts at the named source. It
does not prove Air movement, open-space heat transport, Matter↔Air or Air↔Air
exchange, atmospheric pressure force, phase/ignition retuning, performance,
fun, or user acceptance. G9-A remains **REVISED IMPLEMENTATION CANDIDATE /
USER RE-REVIEW PENDING**. Historical G8/G8-C evidence remains bound to its
sealed historical source.
