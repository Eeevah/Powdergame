# Local Vapor Capacity and Gauge-Pressure Validation Contract

- **Status:** One-shot grid/time proof DESIGN BLOCKED; runtime not started
- **Architecture:** [`ADR-0008`](../architecture/decisions/ADR-0008-local-vapor-capacity-pressure.md)
- **Specification:** [`LOCAL_VAPOR_CAPACITY_PRESSURE_SPEC`](../specs/LOCAL_VAPOR_CAPACITY_PRESSURE_SPEC.md)
- **Runtime:** NOT STARTED

## 1. Evidence layers

| Layer | May establish | Current state |
|---|---|---|
| source/docs audit | static graph, binding and ownership coherence | complete; six open High findings |
| one-shot pure grid/time proof | exact modeled capacity/pressure properties below | DESIGN BLOCKED; one predeclared check failed |
| independent design review | unresolved architecture counterexamples and receipt fidelity | complete; Critical 0 / High 6; DESIGN BLOCKED |
| Rust/WGSL/Naga/GPU/device | implementation semantics | not authorized / not run |
| product/user | visual and gameplay suitability | not run |

No lower layer is promoted to a higher layer.

## 2. One-shot proof contract

The script is newly authored outside the repository and imports no external or
Powdergame simulation implementation. Lock before execution:

| Item | Predeclared value |
|---|---|
| fixed seed | `0x54453543` |
| static generated neighbourhoods | `50,000` |
| bounded multi-tick generated grids | `10,000` |
| deterministic same-process replays | `2` |
| process executions | exactly `1` |
| static world | 7×7 radius-aware local grids |
| multi-tick worlds | 5..12 wide, 6..16 high, 4..24 ticks |
| chunk seam | logical partition every 4 Cells; global coordinates unchanged |
| pressure bounds | `[0,100]` for phase target; `[0,1e6]` gauge |

Script/result paths:

```text
C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te5c_local_vapor_capacity_reference.py
C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te5c_local_vapor_capacity_reference_result.json
```

The result must be standards-compliant JSON, parse with `ConvertFrom-Json`,
embed the pre-run script SHA-256, contain no NaN/Infinity token and record the
lexicographically smallest counterexample or null. A failure is preserved as
the only run. The script is not patched and rerun.

## 3. Required fixed fixtures

- **TE5C-F01:** one canonical Steam plus one adjacent EMPTY: capacity `1`, target `0`.
- **TE5C-F02:** blocked Steam: capacity `0`, target `100`.
- **TE5C-F03:** two Steam sharing exactly one EMPTY: capacity `0.5` and target `100` each; EMPTY aggregate exactly `1`.
- **TE5C-F04:** two Steam with two disjoint sufficient EMPTYs: both target `0`.
- **TE5C-F05:** staggered ADR-0007 vacancy walk; once the second Steam exists, total demand exceeds the single moving capacity and not all targets return to zero.
- **TE5C-F06:** finite headspace crossing occurs by demand/capacity, not a completion event.
- **TE5C-F07:** open boundary control stays below Wood `80`; dense interior compression is reported separately.
- **TE5C-F08:** partial Water demand grows continuously, then reversal lowers demand/target without quantity change.
- **TE5C-F09:** condensation lowers target; movement or EMPTY vent lowers later gauge pressure.
- **TE5C-F10:** generic expansion impulse stays a separate input and is not duplicated.
- **TE5C-F11:** sealed boiler reaches `80`, ruptures Wood, creates EMPTY and records a lower later pressure peak with quantity exact.
- **TE5C-F12:** sealed pressure stable; EMPTY opening vents; solid Boundary does not.
- **TE5C-F13:** global-coordinate/chunk partition equality, modeled scratch full-write/overwrite and deterministic reset/replay.

### Predeclared proportional-underuse attack

This is a required blocking control, not a post-result addition:

```text
Steam B at (0,1), Steam A at (1,1)
shared EMPTY E1 at (0,0)
A-only EMPTY E2 at (2,1)
all other local Cells occupied
```

There is a complete reachable assignment `B->E1`, `A->E2`. The locked formula
instead gives E1 shares `0.5/0.5` and E2 gives A `1`; A caps at `1`, discarding
its extra `0.5`, while B remains at capacity `0.5` and target `100`. The proof
must test `reachable_capacity_no_false_pressure`. If it fails, VC-INV-008 is
not established and TE-5C is DESIGN BLOCKED; no max-flow, iterative
redistribution or changed radius may be substituted.

## 4. Generated checks

For at least 50,000 static neighbourhoods and 10,000 bounded multi-tick grids:

- each EMPTY aggregate contribution is `<=1`;
- demand/capacity/compression/target are finite and non-negative;
- target is `<=100`;
- quantity is invariant under scripted identity/movement events;
- partial demand/condensation moves target in the expected direction;
- vacancy-walk population accounting never treats a completion event as capacity;
- explicit pressure updates remain finite/non-negative;
- opening does not increase the later regional pressure peak;
- identical seed and global grid with different logical chunk partitions match;
- replay digests match.

Any failed fixed or generated check yields `DESIGN_BLOCKED` and preserves the
smallest counterexample. The old TE-5B PASS is not an input.

## 5. Proof limitations

The proof does not establish WGSL bindings, GPU races, real GAS movement,
actual activity/sleep, pressure propagation implementation, rupture writer,
performance, visual quality or user acceptance. Its scripted movement and
rupture are pure reference events. Historical G5 evidence remains source-bound.

## 6. Future runtime fixtures

If and only if architecture later passes and runtime is separately authorized,
the exact F01–F13 geometries and event/region ordering above become CPU/GPU,
Naga/write-contract, sleep-on/off, profiler/allocation and product fixtures.
The future graph must prove 41 passes, 82 queries, two 656-byte profiler
buffers, storage bindings `<=8`, zero new dense allocation, proposal f32 full
write after Smoke and u32 overwrite before next movement.

## 7. Docs/reference validation boundary

Required here: Wiki fallback, exactly one proof process, JSON parse/hash,
Markdown links/fences/index checks, policy/secret audit, `git diff --check` and
docs/memory-only classification.

```text
Cargo test/check/clippy: 0
GPU test/run:            0
workspace FULL:          0
build/launch:            0
TE-3/TE-5 runtime:       0
G8/G8-C:                 0
```

## 8. One-execution receipt

The result path did not exist. Before execution:

| Item | SHA-256 |
|---|---|
| validation contract | `b2b6f216da2d3cd85bd5f2feafdecf0b47d7d00177359587e5bb21d78c5e67eb` |
| script | `f0b4cb155fcc0785c60ff6ff4c2ee9d18a439ed3ea0941e679140de4188af791` |

Executed exactly once:

```powershell
& 'C:\Users\mdkap\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' 'C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te5c_local_vapor_capacity_reference.py' --output 'C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te5c_local_vapor_capacity_reference_result.json'
```

| Result | Value |
|---|---|
| status | `DESIGN_BLOCKED` |
| process executions | `1` |
| wall time / exit | `47.873 s` / `0` |
| seed | `0x54453543` |
| static / multi-tick | `50,000` / `10,000` |
| deterministic replays | `2`, matching digest |
| digest | `3f01a0cb3033f157ba2371c0c4b52dd8d32daecee638b53e4da61a3337565b76` |
| result SHA-256 | `59b98a3454e13a22742e66559e06cfa9b3552a37e18929fa3b71949afaf1e8e5` |
| failed checks | `1`: `reachable_capacity_no_false_pressure` |

The JSON reports F01–F13, deterministic replay and generated bounds/quantity
as passed. Independent static review found that several reported checks do not
execute their named obligations: F13 is a literal `True`, both partition
digests hash the same object, the opening-peak violation counter is never
updated, F09 does not model condensation, F10 does not exercise coexistence or
EMPTY-vent regression, and F06 aliases F05. Those properties are therefore
**NOT ESTABLISHED**, despite the authentic deterministic bytes. Generated
violations were zero only for the checks the script actually implemented:
per-EMPTY capacity, finite/bounded target and phase quantity. JSON parsed with
`ConvertFrom-Json`, the embedded script hash matched and no NaN/Infinity token
existed.

Smallest counterexample:

```text
B=(0,1), A=(1,1), r_A=r_B=1
E1=(0,0) adjacent A/B; E2=(2,1) adjacent A only
feasible: B->E1, A->E2
locked result: capacity_B=0.5, target_B=100;
               capacity_A=1.0, target_A=0
```

The failed output is preserved. The proof was not rerun and the formula was
not changed. Under D-020, TE-5C is **DESIGN BLOCKED** and the next decision
must explicitly permit persistent phase-volume state.

## 9. Independent-review receipt

Fresh-context review recorded unresolved Critical `0` / High `6` in
[`TE5_LOCAL_VAPOR_CAPACITY_PRESSURE_DESIGN`](../adversarial-reviews/TE5_LOCAL_VAPOR_CAPACITY_PRESSURE_DESIGN.md),
SHA-256
`d0d26585326d79cfe60ab0fd0a334e9537e6bedc8d41059e5e129caa08d2edf2`.
It independently reproduced the proportional-underuse blocker and added five
independent High findings:

- an internal EMPTY cannot simultaneously be finite headspace and an infinite
  gauge-zero vent reservoir;
- `max(current,target)` cannot lower phase-origin pressure after condensation
  without source provenance, while the shared field also contains generic
  pressure;
- Chebyshev radius 1 counts below/down-diagonal EMPTY that ordinary GAS cannot
  reach;
- pressure/vent work, snapshot order and the eight-binding activity layout do
  not provide a coherent sleep predicate;
- the one-shot receipt overstates several checks the script does not perform.

The reviewer did not execute the proof or any runtime command. These findings
do not authorize a formula repair or rerun; they reinforce **DESIGN BLOCKED**.
