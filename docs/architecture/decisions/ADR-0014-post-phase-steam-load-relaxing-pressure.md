# ADR-0014 — Post-Phase Steam-Load Relaxing Pressure

- **Status:** PROPOSED — IMPLEMENTATION CANDIDATE / USER REVIEW PENDING
- **Date:** 2026-08-23
- **Decision:** D-037
- **Source baseline:** `12b49dc07c8d875de55a048013a01090d38345a9`
- **Runtime state:** TE-5R1 implementation candidate; automated closure pending
- **External implementation copied, translated or vendored:** `0 files / 0 lines`

## Context

TE-5R0 and ADR-0013 are immutable blocked history. Their three High source
contradictions were a phase-context value unavailable at the pressure pass, an
unobservable fresh-impulse term, and overlapping pressure-activity ownership.
D-037 changes those contracts instead of adding state or repairing ADR-0013.

The product model remains a local bounded approximation. It is not exact
volume assignment, matching, a connected-component solver, or compressible
CFD. Accepted TE-2 Air, TE-3 one-Cell/one-quantity phase enthalpy and TE-4
ignition remain authoritative.

## Decision

Adopt for source review and, only after Critical `0` / High `0`, production
implementation:

```text
DERIVED AIR BACKGROUND
+ STEAM-ONLY DISSIPATIVE DYNAMIC PRESSURE
```

Existing `pressure_current/next` is the only dynamic-pressure state. No new
persistent buffer or full-world scratch is permitted.

### Nodes and pressure terms

Dynamic-pressure nodes are in-domain EMPTY Cells and Liquid/Gas Matter.
Static, Powder and Void are blocked. Missing faces are no-flux for dynamic
pressure.

For canonical EMPTY Air only:

```text
P_air = air_energy_current / 293.15
P_air = 0 for exact Vacuum
P_total = P_dynamic + P_air
```

Matter Cells own no Air, so their background term is zero. Consumers add each
term once. Vacuum can carry dynamic pressure but cannot donate Air mass.

### Post-phase Steam target

With accepted `Lv = 480`:

```text
Steam with finite phase_energy in [0,480]:
    P_target = 100 * phase_energy / 480
all other identities, including every Water state:
    P_target = 0
```

Invalid Steam phase energy is an invariant failure. Authoritative staging
rejects it; production target evaluation fails closed to zero rather than
clamping invalid state into evidence. The target reads settled post-phase
Material and phase energy. It never reads phase-context scratch. A
Water-to-Steam identity change creates a target, not an impulse.

### Generic impulse and local update

Generic expansion failure retains its existing mutually exclusive transaction.
`expansion_pressure` writes a blocked/losing request; the Environment-blocked
writer writes only a Matter winner whose Environment receiver failed. The
expansion settle copies their `pressure_next` result to `pressure_current`.
Decay and combustion do not alter pressure before the local pass.

For settled `q = pressure_current`:

```text
p_next = clamp(
    q
    + 0.20 * sum(node_neighbor_q - q)
    + 0.02 * (P_target - q),
    0,
    1.0e6)
```

An isolated fresh generic impulse `100` with target zero becomes `98` in that
same Tick. This is normative. No event bit, proposal, claim or pre-impulse
pressure is needed by the local pass.

### Coupling and boundary scope

`P_total` affects only Air transport and structural rupture in this slice.
Matter movement keeps its accepted legality, density and parity rules and does
not read pressure.

Air transport computes face demand from `P_total`. The scale pass fully writes
existing proposal/claim scratch as donor scale and total pressure. The maxed
commit reads those values, recomputes the same receiver-capacity scale from
current Air and neighbouring total-pressure scratch, and retains donor-mass,
receiver-mass and receiver-energy bounds without a ninth binding or split.

Rupture samples total pressure on opposing faces:

```text
stress_x = abs(P_total_left - P_total_right)
stress_y = abs(P_total_up - P_total_down)
stress   = max(stress_x, stress_y)
```

Settled blocked nodes already hold dynamic zero, so rupture drops the movement-
class input and uses that slot for canonical EMPTY Air energy. Equal pressure
on both sides does not rupture.

Air and dynamic pressure use sealed/no-flux domain edges. Matter keeps its
existing Void exit. Sealed fixtures must use an explicit in-domain Stone or
Boundary Block ring; no fixture may call the whole world uniformly sealed.

### Activity ownership

Base `activity_propose` fully writes Matter/Thermal/Reaction bits and never
sets `ACTIVITY_PRESSURE`. A new full-world `pressure_activity_propose`, after
settled rupture/hygiene, is the sole pressure-bit setter. It evaluates the same
node, target, neighbour, sanitization and clamp rule as the pressure pass and
sets the bit only when:

```text
abs(predicted_next - pressure_current) > 0.001
```

It never skips sleeping chunks. Environment activity compares `P_total` so
Air work caused by dynamic pressure cannot sleep early. Base full-write clears
the previous Tick's pressure bit before later proposers OR their owned bits.

## Source-realizable graph

The reviewed projection is 43 passes / 86 timestamp queries. The only added
pass is `pressure_activity_propose` immediately before activity reduction.
Pressure update remains pass 32 and rupture pass 33; pressure activity is pass
41 and reduction pass 42.

| Pass | Storage inputs/outputs | Count | Ownership reason |
|---|---|---:|---|
| pressure update | Material, phase energy, pressure Current/Next, class, chunk | **6** | settled Steam target plus existing sleep contract |
| Air flow scale | Material, Air mass, Air energy, chunk, pressure, donor scratch, total-pressure scratch | **7** | both scratch outputs fully written |
| Air commit | Material, Air mass, Air energy, donor scratch, total-pressure scratch, Air mass Next, Air energy Next, chunk | **8** | receiver scale recomputed; no split |
| base activity | Material, temperature, flags, class, density, activity table, cell activity | **7** | pressure input and producer removed |
| Environment activity | Material, temperature, Air mass, Air energy, pressure, cell activity | **6** | total-pressure Air-work predicate |
| pressure activity | Material, phase energy, pressure, class, cell activity | **5** | no chunk skip; sole pressure-bit setter |
| rupture | Material, pressure, Air energy, rupture table, Material Next, temperature Next, flags Next, chunk | **8** | class read replaced by Air energy |

Uniforms are not storage-buffer bindings. Proposal/claim remain allocated
exactly once and their Air lifetime ends before thermal stability and phase
context overwrite them. Profiler memory grows only from 84 to 86 queries:
two 688-byte timestamp buffers, 1,376 bytes total. No world allocation changes.

The complete value/writer/lifetime table and exact pass list are in the
[TE-5R1 source gate](../../planning/TE5R1_STEAM_LOAD_RELAXING_PRESSURE.md).

## Vent evidence boundary

Relaxation alone is not vent evidence. Every relief claim compares matched
opening and no-opening fixtures with the same target and initial pressure and
requires a predeclared `drop_A > drop_B + margin`. It also requires a real
structural identity change, new pressure-node topology, following-Tick Air
flow, legal ordinary Steam movement and exact phase-family quantity.

## Consequences and exclusions

Benefits are a source-local Steam load, bounded reversible pressure, no Water
context lifetime, exact activity ownership, no pressure-driven Matter feedback
and no new state. Costs remain deliberate dissipation, local hot spots, delayed
Air coupling and a spatial trail when Steam moves.

This ADR does not authorize or define pressure-biased Matter movement, tokens,
matching, CCL, packets, owner fields, Oxygen, Ash, TE-6, G9-B/C/D/E, product
reservoir mode or a second reference simulator. Historical G5 evidence remains
source-bound and is not rebound.

## Approval boundary

The fresh-context source review reported unresolved Critical `0` / High `0`
and authorized implementation. The implementation preserves the reviewed
43-pass/86-query graph and eight-binding ceiling; its fixture contract is in
the [validation document](../../development/STEAM_LOAD_RELAXING_PRESSURE_VALIDATION.md).
ADR-0014 remains **PROPOSED**. Automated implementation success produces a
user-review candidate; only direct user review may accept this ADR.
