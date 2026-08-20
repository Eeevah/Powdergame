# Phase-Volume Pressure Bridge Validation Contract

- **Status:** Pure reference abstraction PASS / DESIGN BLOCKED; runtime validation NOT STARTED
- **Architecture:** [`ADR-0007`](../architecture/decisions/ADR-0007-phase-volume-pressure-bridge.md)
- **Specification:** [`PHASE_VOLUME_PRESSURE_BRIDGE_SPEC`](../specs/PHASE_VOLUME_PRESSURE_BRIDGE_SPEC.md)
- **Design baseline:** `d7500e219af6f670be05f830b50c232d2bb53077`
- **Authorization source:** `f1ca48cc01a906bfb4a997c72bc2744b81546ccd`
- **Runtime:** TE-3 / TE-5B / full TE-5 NOT STARTED

This contract is written before the one permitted pure-reference execution.
It prevents a passing abstraction from being relabelled as WGSL, GPU, movement,
pressure propagation, rupture, product or user evidence.

## 1. Evidence layers

| Layer | May establish | Current state |
|---|---|---|
| docs/source audit | coherent ownership, encoding, pass/binding/lifetime feasibility | independent review found an unresolved High finite-capacity counterexample |
| pure reference proof | encoding, deterministic abstract arbitration and consequence accounting | passed its only permitted process execution |
| CPU semantic implementation | future Core implementation matches the candidate | not implemented / not run |
| Naga/write-contract | future WGSL parses; mode guards/full writes/bindings are structural | not implemented / not run |
| production GPU fixtures | actual races, ownership, movement, wake, pressure/rupture/vent trace | not implemented / not run |
| profiler/allocation | actual 40/80/1,280 B and zero TE-5B memory delta | projection only / not run |
| product/user observation | finite-headspace meaning, open control and inherited scalar suitability | architecture revision required; runtime not run |

A docs/reference PASS cannot advance a runtime gate.

## 2. Fixed-seed reference proof — predeclared contract

The proof is a newly authored pure Python model outside the repository and
outside production runtime. It imports no Powdergame or external simulation
implementation.

Locked before execution:

| Item | Predeclared value |
|---|---|
| fixed seed | `0x54453542` |
| unique randomized arbitration trials | `100,000` |
| deterministic replays inside the one process | `2` |
| process/script executions | exactly `1` |
| script path | `C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te5b_phase_volume_bridge_reference.py` |
| result path | `C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te5b_phase_volume_bridge_reference_result.json` |
| expected status | `PASS_REFERENCE_MODEL_ONLY` |

The output must be finite standards-compliant JSON, parse with PowerShell
`ConvertFrom-Json`, include every check/count below, and record the smallest
counterexample or `null`. Script SHA-256 is computed before execution; result
SHA-256 is computed after the single process exits. A failed run is preserved
as the only result and blocks the design; the script is not repaired and rerun
under the same task.

### 2.1 Required proof checks

The one process must establish all of these properties in its pure model:

1. `MODE | (index + 1)` round-trips boundary indices `0`, `1`,
   `cell_count - 2` and `cell_count - 1` for representative legal Cell counts,
   including `cell_count = (1 << 30) - 1`;
2. NONE, both blocked words, every targeted boundary word and reserved/invalid
   words do not collide;
3. mode `00` with payload, mode `11`, out-of-range payload and mode-mismatched
   claim all fail closed;
4. each generated source owns at most one request;
5. each target chooses at most one winner using the existing deterministic
   edge-priority/min-source tie-break abstraction;
6. Matter-expansion and volume-relief candidates share one winner list, so a
   mixed target never chooses one winner per mode;
7. rerunning all generated trials from the same seed inside the process yields
   an identical canonical digest and aggregate result;
8. every relief winner leaves its modeled target Matter and Air state exactly
   unchanged;
9. relief pressure-consequence count equals blocked relief requests plus losing
   targeted relief requests;
10. every relief winner receives zero phase-volume pressure;
11. Matter direct failure, Matter Environment-receiver failure and relief
    failure remain separate source-owned outcomes;
12. relief produces zero Environment-receiver requests and zero Environment-
    blocked duplicate consequences;
13. modeled phase-family quantity before/after all completion outcomes is
    identical and no relief outcome creates extra Matter;
14. an input-identity/output-identity completion event emits a request once,
    while subsequent Steam ticks emit none and add no repeated pressure;
15. a modeled reset reproduces the same event/consequence tuple;
16. full overwrite into the next modeled scratch lifetime removes every
    proposal/claim request-mode tag before a later consumer.

Randomized trials must include 1–32 sources, 1–16 EMPTY destinations, both
modes, no-request, blocked and targeted sources, deliberate shared targets,
optional generic Environment-receiver failure and randomized immutable target
Air tuples. The generated model is an arbitration/accounting abstraction; it
does not construct a grid or claim actual GAS movement.

### 2.2 Failure rule

Any failed required check yields status `DESIGN_BLOCKED`, preserves the
lexicographically smallest canonical counterexample and prevents a PASS claim.
No alternate encoding, pressure scalar, extra pass or second run may be
silently substituted.

### 2.3 Explicit limitations

Regardless of PASS, the proof does not establish:

- Rust or WGSL correctness;
- actual storage bindings, full-world writes or GPU race freedom;
- the production GAS movement target or cross-tick availability;
- chunk seams, wake halos or sleep-on/off equivalence;
- scalar pressure propagation, sanitization or saturation;
- Wood rupture, opening, venting or pressure decline;
- pass/query/allocation/performance measurements;
- visual quality, finite-headspace product meaning or user acceptance.
- any grid/vacancy-conservation property, finite-capacity consumption or
  transition from early relief to later confinement.
- actual phase-descriptor slot population, registry-to-descriptor generation,
  the normalization trigger for extreme Ice, or a ready Water `E = Lv`,
  `T = 100` semantic completion/reopen path.
- downstream revalidation of source/target against proposal after
  `expansion_claim`; receiver/spawn trust that producer's full-write output.

### 2.4 One-execution receipt

The predeclared validation contract SHA-256 immediately before execution was
`a05234ab0b920ed56fe1256c341634330a640bf25787595d53eb56f0fbb5c2c2`.
The result path did not exist. The script SHA-256 was fixed, then this command
was executed exactly once:

```powershell
& 'C:\Users\mdkap\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' 'C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te5b_phase_volume_bridge_reference.py' --output 'C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te5b_phase_volume_bridge_reference_result.json'
```

| Item | Result |
|---|---|
| status | `PASS_REFERENCE_MODEL_ONLY` |
| fixed seed | `0x54453542` |
| unique randomized trials | `100,000` |
| deterministic replays inside one process | `2` |
| process/script executions | `1` |
| exit code / wall time | `0` / `19.4945397 s` |
| script SHA-256 | `6fd9276933822db850bd4ec3f9648cf64c45b8905f6b37d17cc88d03cb23a340` |
| embedded/actual script hash match | `true` |
| result SHA-256 | `f53173af05199916b10d287a02c8193e9f86c40c853c019db8491cb86ff56e59` |
| deterministic digest | `001968b462d75865851e159c35167e6ace04c27c46d12a7f77511823ab378d80` |
| failed summary checks | `0` |
| smallest counterexample | `null` |

PowerShell `ConvertFrom-Json` parsed the result. Every summary check was true;
the embedded script hash matched the pre-run/actual hash; and the raw result
contained no NaN or Infinity token.

Coverage/accounting totals from the unique trials:

| Counter | Value |
|---|---:|
| mixed-mode targets | 169,945 |
| relief requests | 661,857 |
| relief winners | 250,245 |
| blocked/losing relief requests | 411,612 |
| relief pressure consequences | 411,612 |
| generic direct consequences | 412,418 |
| generic Environment-blocked consequences | 124,481 |
| phase-family quantity before / after | 661,857 / 661,857 |

The proof also passed boundary `index + 1` round-trips through the largest legal
Cell-count case, abstract invalid/reserved/mode-mismatch rejection, smaller-
source tie break, one request/source, one winner/target, unchanged relief
targets, zero winner pressure, no relief Environment path, identity-labelled
Water and extreme-Ice completion cases, subsequent-Steam non-repeat, reset
replay and modeled scratch full overwrite. Its extreme-Ice check calls a pure
`completion_word("Ice", "Steam", ...)` identity abstraction and checks the
constant consequence. It does not model registry-derived phase-descriptor
slots, actual enthalpy normalization or attempt eligibility.

This receipt does not strengthen Section 2.3's limitations. In particular it
does not prove that production GAS movement consumes capacity, that bindings or
full writes exist in WGSL, or that pressure propagates to rupture and venting.
The model contains abstract targets but no grid or cross-tick occupancy. Its
PASS is therefore compatible with the later High counterexample in which an
EMPTY vacancy simply moves down a sealed Water column and is repeatedly
recounted. The proof was not rerun or retroactively changed.

## 3. Future implementation fixtures

All fixtures are defined before implementation. “Exact pressure 100” uses
initial gauge pressure zero and observes the completion-tick source before
propagation.

### TE5B-F01 — Unique open relief

Predeclare one completion-ready Water source and exactly one GAS-reachable
EMPTY target. The relief claim wins. Require one Steam quantity, no extra
Steam, zero phase-volume source pressure and byte-identical target Matter, Air
mass and Air energy.

### TE5B-F02 — Fully blocked completion

Predeclare completion-ready/vaporization-ready Water with no in-domain EMPTY
target in the five-Cell relief stencil. Require one Steam quantity, exactly
`100.0` source pressure on the completion tick, no extra Matter and no
Environment mutation. Include explicit semantic cases for ready Water at
`E = Lv`, `T = 100` while buried/blocked and after a surface route reopens; the
reference proof did not establish either trigger path.

### TE5B-F03 — Shared relief contention

Predeclare two completion-ready Water sources whose only relief candidate is
the same EMPTY Cell. Require one deterministic relief winner, one loser with
exactly `100.0`, an EMPTY/byte-identical target and phase-family quantity two.

### TE5B-F04 — Open route is consumed by ordinary movement

Predeclare a completion that wins a relief target and isolate the next-tick
world so the resulting Steam's existing GAS proposal selects that same route.
Require ordinary movement, not a bridge move or reservation, to consume it.

### TE5B-F05 — Finite headspace becomes confined

**Current candidate status: UNSATISFIABLE / DESIGN BLOCKER.**

Predeclare a sealed chamber with a Water surface and finite EMPTY headspace.
Early completion wins relief; ordinary Steam GAS movement fills headspace;
later completion is blocked or loses contention; pressure begins only after
local relief is unavailable. Record exact occupancy and event ticks. The
one-Cell-wide vacancy-walk counterexample shows the evaluated 1:1 non-mutating
token cannot guarantee this outcome: movement vacates the source and supplies
the next relief target instead of consuming capacity.

The blocking control must stagger enthalpy: only the top Water is ready at
`t0`; each lower Water is just below `Lv` and reaches `Lv` only on the tick after
ordinary movement brings the vacancy above it. This prevents a lower same-tick
attempt from legitimately stopping on the newly created Steam as a density-
swap outcome. Even with that ordering, every completion wins zero pressure.

### TE5B-F06 — Generic expansion compatibility

Use a synthetic non-family yield-2 rule targeting non-phase Matter. Require the
historical target claim, Environment receiver, target Matter/Environment
transaction and failure consequences to remain valid without violating phase-
energy invariants.

### TE5B-F07 — Mixed-mode target contention

One Matter-expansion and one relief source request the same EMPTY target.
Require exactly one deterministic winner. Every loser receives only its own
mode consequence. No source receives duplicate pressure and no mode gains a
second arbitration domain.

### TE5B-F08 — Environment receiver failure isolation

Make a generic Matter target win while its Environment receiver fails. Require
the existing generic exactly-once pressure. In the paired relief case require
zero receiver request and zero Environment-blocked relief consequence.

### TE5B-F09 — Exactly-once source

Run one confined Water-to-Steam completion through subsequent Steam ticks.
Require one completion-tick pressure consequence only. Reset the fixture and
require the same event/tick/value sequence.

### TE5B-F10 — Open boiler no false rupture

Maintain a visibly open local relief route and a nearby weak wall. Require no
phase-volume source for each winning completion and no rupture attributable to
a phase-volume source that never occurred. Other pressure sources are excluded
or separately labelled.

### TE5B-F11 — Atomic G5 causal fixture

**Current candidate status: UNSATISFIABLE / DESIGN BLOCKER.**

Predeclare a finite-headspace boiler with no scripted explosion and no
combustion-created opening. Required ordered trace:

```text
Water surface heating
-> positive boiling phase energy
-> first 1:1 Steam completion
-> early relief claim
-> ordinary Steam GAS movement into headspace
-> relief capacity exhausted or contended
-> later completion emits pressure 100
-> local gauge-pressure propagation
-> Wood threshold 80 exceeded
-> generic rupture
-> opening
-> ordinary Steam/Liquid movement through opening
-> regional pressure peak declines
```

The fixture must declare exact event names, ticks and regions before capture.
It must report phase-family quantity at every event, show no extra Steam spawn,
use no boiler-specific explosion, use no combustion-created opening and bind
all evidence to the future atomic source. Historical G5 receipts are not
relabelled.

The evaluated token cannot force the `relief capacity exhausted` event without
contention that is incidental to scheduling rather than finite headspace. F11
therefore cannot establish the required product meaning under this candidate.

### TE5B-F12 — Scratch, sleep and reset

Require proposal/claim full writes and invalid-mode failure, a contention case
across a chunk seam, sleep-on/off semantic equivalence, completion/pressure
wake of the required existing halo and exact reset/staging. No mode word may be
observed by a later Smoke/movement lifetime. Include a top-edge Void-first case
that returns `EDGE_DEFERRED` and an earlier legal density-swap case that stops
as blocked; neither may skip to a later lateral EMPTY target.

## 4. Structural and device obligations

Before any runtime candidate, future validation must additionally prove:

- `cell_count < 1 << 30` is enforced before mode use;
- Naga accepts every modified WGSL module;
- structural binding counts equal the specification and never exceed eight;
- every proposal/claim producer fully writes all in-domain Cells before its
  consumer, including sleeping skip paths; `expansion_claim` must reject every
  invalid/mismatched proposal candidate and write either one constructed valid
  winner or zero;
- receiver/spawn validate claim mode/source range and trust the claim writer's
  source/target construction; no test may claim independent proposal/claim
  cross-validation in those consumers without a new binding design;
- receiver, spawn and Environment-blocked pressure have an early relief-mode
  rejection;
- the phase descriptor encodes the `100.0` consequence for every source family
  identity that can normalize through vaporization in one invocation;
- a semantic/structural fixture derives that descriptor from the registry and
  exercises gas-facing extreme Ice through the actual normalization trigger,
  while non-gas-facing extreme Ice cannot initiate through the bridge;
- no WGSL branch compares the source Material specifically to Water to produce
  confinement pressure;
- phase identity/energy and pressure settle in the specified order;
- profiler names/groups expose the unchanged 40-pass graph and 80 queries;
- tracked allocations show TE-5B delta zero;
- external staging/reset writes canonical phase energy and existing reference
  pressure only; proposal/claim require no persistent initialization.

## 5. Evidence and stop rules

The independent reviewer must attack the full user-specified list plus:

- extreme finite Ice-to-Steam completion selecting the wrong consequence;
- invalid mode/payload or mode-mismatched claim producing a side effect;
- a Matter winner whose receiver later fails causing an illicit relief retry;
- same-tick exclusivity being mistaken for a next-tick reservation;
- an existing maxed binding acquiring an undeclared input.

The independent review found an unresolved High finite-capacity counterexample,
so the current task disposition is:

```text
TE-5B: DESIGN BLOCKED
Runtime: NOT STARTED
```

The otherwise-available successful docs-only stop below was not reached:

```text
TE-5B: PHASE-VOLUME BRIDGE DESIGN CANDIDATE
       / INDEPENDENT REVIEW PASS
       / USER ARCHITECTURE REVIEW PENDING
ADR-0007: PROPOSED
Runtime: UNCHANGED / NOT STARTED
```

## 6. Validation expected in this docs/reference task

Required non-runtime checks:

- Wiki verification or documented safe remote fallback;
- exactly one reference process execution;
- result JSON parse and SHA-256 verification;
- Markdown links, fences and index validation;
- strict policy audit and applicable secret scan;
- `git diff --check`;
- docs/memory-only changed-path classification.

Expected execution count zero:

```text
Cargo test/check/clippy
GPU test/run
workspace FULL
release build
application launch
TE-3 runtime candidate
TE-5B runtime candidate
full TE-5 runtime candidate
G8 / G8-C
```
