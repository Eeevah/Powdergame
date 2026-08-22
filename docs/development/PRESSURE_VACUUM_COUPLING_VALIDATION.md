# Pressure / Vacuum Coupling Validation Contract

- **Status:** DESIGN BLOCKED by independent review; no runtime/reference execution
- **Authority:** D-035, proposed [ADR-0013](../architecture/decisions/ADR-0013-local-relaxing-phase-load-pressure.md)
- **Specification:** [PRESSURE_VACUUM_COUPLING_SPEC](../specs/PRESSURE_VACUUM_COUPLING_SPEC.md)
- **Runtime:** TE-5 NOT STARTED

## 1. Evidence boundary

This task supplies architecture, exact static source feasibility and an
independent review only. It runs no Python reference, coefficient campaign,
Rust, WGSL parser, GPU, FULL, build or application. Analytical stability and
binding arithmetic are not runtime PASS.

Future evidence must distinguish:

| Layer | Required claim |
|---|---|
| Core semantics | finite update, target, total-pressure and stress formulas |
| structural WGSL | parse, full writes, binding counts, scratch lifetimes and exact pass order |
| production GPU | actual Current/Next state, Air/Matter/phase/rupture transactions and sleep |
| profiler/allocation | a future revised graph must re-establish pass/query/binding totals and zero world-state delta |
| product/user | readable boil/load/pressure/rupture/vent behavior and acceptable local hot spots |

No old G5 or blocked TE-5 receipt transfers to the future source.

## 2. Future fixture contracts

Every fixture must stage real authoritative Current/Next state, execute actual
production ticks and report exact milestone ticks, region extrema, conserved
quantities, Air exchange and source identities. A label or precomputed literal
is `NOT_ESTABLISHED`.

### TE5R-F01 — Uniform standard Atmosphere

Fill a sealed EMPTY region with canonical Air `(mass=1, energy=293.15)` and
dynamic zero. Require total pressure one everywhere, zero Air transfer, zero
Matter bias, zero wall face differential and no rupture for the bounded sleep
horizon.

### TE5R-F02 — Connected Vacuum refill

Join standard Atmosphere to exact Vacuum through an EMPTY face. Require actual
TE-2 mass/energy transport, conserved closed-domain totals, positive Vacuum-side
Air after the first accepted transfer and monotone reduction of the total-
pressure face demand within tolerance. Dynamic pressure creates no Air.

### TE5R-F03 — Heated sealed Air

Heat a sealed fixed-mass Air Cell/region through production thermal transfer.
Require increased `air_energy/293.15`, a corresponding total-pressure gradient
and no dynamic-pressure source when phase/generic sources are absent.

### TE5R-F04 — Fixed Atmosphere reservoir

Use the explicit fixture-only boundary. Require boundary Air and dynamic values
remain standard/zero and every external mass, energy and pressure exchange be
reported with sign and cumulative total. The same geometry under sealed mode
must exchange zero externally.

### TE5R-F05 — One Steam load in a large chamber

Place one canonical Steam Cell in a predeclared large connected pressure-node
region with no generic impulse. Require source target 100, component-average
equilibrium near `100/N`, finite local peak recorded honestly and every Wood
face differential below 80 for the locked horizon. Failure is an open-space
false-rupture blocker, not permission to retune after observation.

### TE5R-F06 — Half-filled sealed vapor chamber

Use equal counts of canonical Steam target-100 nodes and zero-target EMPTY
nodes in one sealed component. Require component average approach 50 within a
predeclared tolerance/horizon and quantity/Air invariants throughout.

### TE5R-F07 — Fully vapor-loaded sealed chamber

Use a connected component whose every node is canonical Steam. Require dynamic
pressure approach 100, cross Wood threshold 80 at the analytically predicted
bounded interval and produce no extra Steam or event impulse.

### TE5R-F08 — Water/Steam continuity

Stage gas-facing Water through partial positive `E`, `E=Lv`, and 1:1 Steam.
Require targets `100*E/Lv`, exactly 100 on both sides of identity completion,
zero pressure impulse and exact H/family quantity.

### TE5R-F09 — Buried ready-Water

Stage Water at `E=Lv` with four non-gas orthogonal neighbours. Require target
zero and no dynamic rise attributable to phase load. Open one real gas face;
the accepted phase path may complete and the source becomes target 100 without
hidden identity scripting.

### TE5R-F10 — Condensation relief

Cool Steam through partial condensation to Water using accepted TE-3 sinks.
Require target fall continuously with `E`, stored dynamic pressure decline by
the same update law, exact phase H/quantity and no owner recovery/matching.

### TE5R-F11 — Opening increases connected volume

Reach a settled sealed loaded region, rupture/open one real wall Cell, and
record the connected node count, regional average and peak before/after.
Require finite-rate—not instant—decline and no external mass exchange under the
default sealed edge.

### TE5R-F12 — Narrow-neck transmission

Connect loaded and unloaded chambers through a one-Cell neck. Require pressure
cross the neck only by one-face-per-tick diffusion, with predeclared near/far
milestones. Distant capacity may not alter the source before the causal
frontier reaches it; permanent false confinement is also a failure.

### TE5R-F13 — Uniform two-sided wall

Place Wood between equal total pressure on opposing faces, including one case
with nonzero Air background and dynamic pressure. Require both axis
differentials zero within tolerance and no rupture.

### TE5R-F14 — One-sided differential rupture

Drive one face above the Wood threshold while the opposite face remains zero.
Require descriptor-driven rupture, exact opening tick and no material-name
branch or combustion-created opening.

### TE5R-F15 — Following-tick vent use

After F14, require the rupture-created EMPTY Cell to join pressure topology and
ordinary Air/Gas/Liquid paths on following ticks. No same-tick rollback,
teleport or special vent movement is allowed.

### TE5R-F16 — Total pressure exactly once

Construct independently nonzero Air background and dynamic pressure. Compare
Air raw demand, Liquid/Gas candidate ranking and structure face stress against
`P_air+P_dynamic`; reject background-only, dynamic-only or any `2*` term.

### TE5R-F17 — Source removal / no stale permanent pressure

Remove a phase load by real condensation or identity exit. Require target zero,
continued wake while the update exceeds epsilon, bounded convergence toward
zero and eventual sleep. No owner/link cleanup may appear.

### TE5R-F18 — Writer and hygiene matrix

Exercise movement, density swap, phase, decay, combustion, rupture, Draw,
Erase, preset, scenario, benchmark staging and reset. Phase energy follows
Matter; dynamic pressure remains spatial; invalid staging rejects without
partial commit; canonical EMPTY/Air and both pressure halves are exact.

### TE5R-F19 — Chunk and sleep equivalence

Place diffusion, Air-flow, movement-bias and relaxation frontiers across a
64-Cell seam. Compare sleep on/off for equal executed ticks, require existing
halo wake, and require equilibrium bulk eventually sleep without unfinished
relaxation.

### TE5R-F20 — TE-2/3/4 regression

Re-run accepted Air transport/thermal, phase-cycle and ignition transaction
fixtures on the future source. Require unchanged phase quantity/H, exact Air
accounting, positive-Air ignition policy, fuel/Q lifetimes and no Vacuum
combustion. New pressure effects must be explicitly isolated where expected.

### TE5R-F21 — Replacement product chain

Predeclare one user-testable causal scene:

```text
Water heating -> positive phase E -> 1:1 Steam -> phase-load pressure
-> finite-rate propagation -> one-sided Wood differential >=80 -> rupture
-> real EMPTY opening -> following-tick Air/Steam/Liquid use -> lower later peak
```

Require strict tick/region/source order, exact family quantity, no extra Steam,
no boiler-specific explosion, no combustion opening, total pressure exactly
once and a clearly labelled derived-background/dynamic trace. This is new
source evidence; historical G5 receipts cannot satisfy it.

## 3. Structural gates

Before runtime review, future implementation must prove:

- every WGSL module parses and every writer self-writes its authoritative output;
- the revised named production passes and timestamps match the source;
- all storage-binding counts match the TE-5R plan and are `<=8`;
- Air donor/receiver scratch is consumed by both split commits before reuse;
- pressure settles before differential rupture;
- movement/Air read the same previous settled total-pressure snapshot;
- pressure and pressure-activity predicates are semantically identical;
- blocked nodes and reset/editor paths clear both pressure halves;
- no new persistent or full-world scratch buffer is allocated;
- tracked totals are exact at 256² and 2048²;
- no Water-family expansion proposal or pressure impulse reappears;
- no external implementation text/formula enters production.

## 4. Failure and user-review gates

Any of these blocks the implementation candidate:

- open control reaches Wood threshold from one sparse phase load;
- background/dynamic double count;
- a ninth storage binding or hidden allocation;
- sleep before relaxation/transport completes;
- Vacuum manufactures Air or supports combustion;
- uniform opposing pressure ruptures;
- narrow-neck response becomes instantaneous component equilibrium;
- invalid state is clamped into a PASS;
- TE-2/3/4 accepted behavior regresses;
- old G5 evidence is rebound.

Even a fully passing automated candidate requires direct user review of F05,
F11/F12, F21, pressure-biased Matter motion and the visibility/meaning of
dynamic pressure in Vacuum.

## 5. Current task receipt

```text
reference process: 0
Cargo test/check/clippy: 0
WGSL/Naga/GPU/device: 0
workspace FULL: 0
release build/application launch: 0
TE-5 runtime: 0
```

Only local docs/policy/link/fence/index/secret/diff/path checks are permitted.

The fresh-context source review is the only design evidence executed. Its
result is Critical `0`, High `3`, Medium `3`, hence **TE-5R0 DESIGN BLOCKED**.
The blocking witnesses are:

- unavailable pre-transition phase-context data at the projected pressure pass;
- unavailable/transactionally inconsistent fresh generic-impulse input;
- unchanged base activity permanently waking a two-node exact equilibrium.

The 44-pass arithmetic remains a frozen failed projection, not a structural
gate PASS. F01–F21 were predeclared but not executed. A later replacement or
revision requires a new explicit user decision and fresh evidence identity.
