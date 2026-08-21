# Pressure-Volume Model Comparison Validation

- **Status:** one-shot execution failed before candidate evaluation; DESIGN BLOCKED
- **Authority:** D-022 and proposed ADR-0010
- **Runtime evidence:** none
- **Historical proofs:** TE-5B/C/D remain read-only and do not transfer

## Frozen one-shot execution

The external script is outside production runtime and repository history:

```text
C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te5x_pressure_volume_comparison.py
```

Pre-execution SHA-256:
`0079246918a91faa606d531cb76591af0363dfb3a66d4b88882fc04e33efd8d5`.

The exact command is:

```powershell
$env:PYTHONPATH = "$env:TEMP\te5x-networkx-3.6.1"
& 'C:\Users\mdkap\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' `
  'C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te5x_pressure_volume_comparison.py'
```

It was run exactly once. The script refuses to overwrite an existing result.
NetworkX 3.6.1 is installed only in the temporary proof environment and is not
a repository or runtime dependency.

## One-shot failure receipt

The only process exited during the frozen oracle-version guard, before seed
consumption or any A/B/C evaluation:

```text
AttributeError: module 'networkx' has no attribute '__version__'
```

The temporary package path resolved as a namespace module with no `__file__`
or version attribute. Later directory inspection was access-denied. The
failure receipt is:

```text
C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te5x_pressure_volume_comparison_result.json
```

- proof processes: `1`
- completed proof processes: `0`
- candidate evaluations: A `0`, B `0`, C `0`
- generated states: `0 / 50,000`
- multi-tick grids: `0 / 10,000`
- deterministic replay: `NOT_RUN`
- script SHA-256:
  `0079246918a91faa606d531cb76591af0363dfb3a66d4b88882fc04e33efd8d5`
- failure-result SHA-256:
  `097f340c265d9e43a23e281a776905add97e6b05c18dedd79d48807558efc116`
- disposition: `INCOMPLETE_EVIDENCE`

The JSON parses successfully but is explicitly a post-exit failure receipt,
not script-emitted proof evidence. The one-shot contract forbids patching and
rerunning it in this task.

## Pre-registered constants and coverage

| Item | Frozen value |
|---|---:|
| algorithm schema | `TE5X-PREGISTER-1` |
| seed | `0x54453558` |
| matching states | 20,000 |
| chamber states | 15,000 |
| conservative-field states | 15,000 |
| total generated states | 50,000 |
| bounded multi-tick grids | 10,000 |
| grid horizon | 48 |
| augmenting chains | 8, 16, 64, 96 sources |
| `Lv` | 480 |
| pressure maximum | 100 |
| Wood threshold observation | 80 |
| phase-pressure relaxation | 0.10 |
| scalar transport rate | 0.20 |
| float tolerance | `1e-9` |
| exact CPU oracle | NetworkX `hopcroft_karp_matching` 3.6.1 |

The response candidates were compared before execution: binary-any-deficit,
linear compression and smooth quadratic. Linear compression is frozen because
binary is discontinuous and quadratic delays the named consequence. No curve
may be substituted after output.

## Candidate checks

Candidate A must compare 5,000 small generated graphs with the NetworkX
Hopcroft–Karp oracle, complete all 20,000 cases, accept arbitrary legal retained
matchings, traverse chains longer than 6/8/16/64, produce the exact unmatched
count, remain deterministic and keep pre-certificate pressure zero.

Candidate B must compare BFS and union/find component partitions, compute
finite target in `[0,100]`, prevent vacancy-walk capacity reset, reach Wood
threshold in a sealed trace, keep a narrow-neck opening finite-rate and produce
a lower post-opening pressure peak. Its CPU model does not prove the projected
GPU CCL/radix/reduction pipeline.

Candidate C must conserve source-free scalar transport within tolerance and
then test the frozen condensation witness: source one unit, diffuse it away,
request a one-unit local sink. Negative result, clipping orphan, added debt or
component-wide withdrawal all make Candidate C ineligible.

The execution emits no single overall PASS. It reports mathematical status,
product/visual unknowns, GPU-feasibility unknowns, rejection and incomplete
evidence separately. The provisional A/B/C ranking is void if fresh review
leaves any unresolved Critical/High finding.

## Common fixture matrix

| ID | Geometry/event and required observation |
|---|---|
| PVX-F01 | one Steam and one open capacity; no false pressure |
| PVX-F02 | fully confined Steam; bounded target rises |
| PVX-F03 | staggered TE-5B vacancy column; capacity never resets |
| PVX-F04 | TE-5C asymmetric two-Steam/two-EMPTY; available capacity is not discarded |
| PVX-F05 | augmenting chains 8/16/64/96; sufficient capacity never creates rupture pressure |
| PVX-F06 | finite headspace; target begins at actual capacity exhaustion |
| PVX-F07 | large open beaker; no false rupture and mobile Steam semantics |
| PVX-F08 | two chambers joined by one-cell neck at tick 0 after stored pressure 100; first post-open pressure must be 90, not 0 |
| PVX-F09 | condensation lowers demand and pressure; C must remove transported volume exactly or fail |
| PVX-F10 | heat→phase E→1:1 Steam→capacity exhausted→pressure→Wood rupture→opening→decline; no extra Steam/combustion |
| PVX-F11 | Atmosphere/Vacuum share narrow capacity meaning; background pressure remains deferred |
| PVX-F12 | EMPTY movement, density swap and Void; no orphan link/field/pressure |
| PVX-F13 | Draw, Erase, reset and staging canonical cleanup |
| PVX-F14 | chunk partition and sleep-on/off equivalence; no terminal tail |
| PVX-F15 | exact 2048² bytes, passes, queries, bindings and convergence contract |

The reference process directly models the mathematical/time cores. Movement,
editor, reset, chunk, sleep, Air receiver, actual rupture mutation, GPU binding
and 2048² cost entries are marked architecture analysis/arithmetic, never
runtime PASS.

## Selection criteria

A candidate is ineligible for false rupture with sufficient capacity,
quantity/Air/phase-volume loss, irreversible pressure, solver-delay pressure,
unbounded or unspecified production work, a pass above eight storage bindings,
unexplained persistent state, stale movement/edit/reset state, evidence rebound
or external code ingress.

Eligible candidates rank by causal/user-readable meaning, conservation and
reversibility, deterministic GPU feasibility, bounded work, memory and finally
complexity. Memory never overrides semantic failure. No fourth candidate may
be created after execution.

## Evidence boundary and stop

This one process cannot establish WGSL compilation, bindings, races, device
behavior, performance, activity/sleep, editor/reset integration, actual Matter
movement, Air conservation, rupture/vent mutation, visuals or user acceptance.
Cargo/GPU/FULL/build/launch/runtime counts remain zero.

The required evidence was not produced. No candidate can be declared eligible,
ranked or recommended. TE-5X is therefore **DESIGN BLOCKED** independent of any
later review finding. A future user decision would have to authorize a new
comparison execution identity; this task cannot repair or rerun the frozen
process. Runtime remains not started.
