# Steam-Load Relaxing Pressure Specification

- **Decision:** D-037
- **Architecture:** [ADR-0014](../architecture/decisions/ADR-0014-post-phase-steam-load-relaxing-pressure.md)
- **Status:** IMPLEMENTED CANDIDATE — USER REVIEW PENDING
- **Historical boundary:** ADR-0013 and TE-5R0 remain blocked and immutable

## 1. Scope

TE-5R1 reuses the existing `pressure_current/pressure_next` pair as a bounded
dynamic-pressure field. It adds no persistent field and no full-world scratch.
It is a local relaxing product approximation, not volume matching, a connected-
component solver, exact compressible flow, or pressure-driven Matter dynamics.

The only active couplings are Air transport and structural rupture. Matter
movement remains the accepted ordinary stencil and does not read pressure.

## 2. Nodes and background

Dynamic-pressure nodes are in-domain EMPTY, Liquid and Gas Cells. Static,
Powder and Void are blocked. A missing dynamic-pressure face is sealed/no-flux.

For canonical EMPTY state only:

```text
P_air = air_energy / 293.15
P_total = P_dynamic + P_air
```

Exact Vacuum has zero mass and energy, hence zero background. Occupied Matter
owns no Air background. Each consumer forms the sum once.

Air and dynamic pressure have sealed domain edges. Matter retains its existing
Void-exit edge. Evidence chambers use explicit in-domain walls.

## 3. Steam target and update

For settled Material and phase energy:

```text
Steam, finite E in [0,480]: target = 100 * E / 480
Water and every other identity: target = 0
```

Invalid Steam energy is an invariant failure at authoring. The shader fails
closed to target zero; it does not clamp invalid state into a valid receipt.
Water contributes no target before identity completion. Water-to-Steam creates
only the Steam state target and never emits a phase-pressure impulse.

After generic expansion consequence settlement, let `q` be the current field:

```text
p_next = clamp(
    q
    + 0.20 * sum(dynamic-neighbour - q)
    + 0.02 * (target - q),
    0,
    1.0e6)
```

Blocked faces contribute no term. An isolated target-zero `q=100` becomes
`98` in that same update and continues `96.04`, not another impulse. Generic
expansion consequences remain owned by their existing mutually exclusive
passes; no currently active Water-family transition uses them.

## 4. Air transport

Air face demand compares total pressure. The scale pass fully overwrites the
existing proposal scratch with donor scale and claim scratch with total
pressure. Commit reads both, recomputes complete receiver headroom from current
Air and all four incoming faces, and applies the donor/receiver minimum.

The transaction remains mass- and energy-bounded. A Vacuum Cell with dynamic
pressure can influence direction but cannot donate mass or manufacture Air.
Proposal is overwritten at thermal stability; claim is overwritten at phase
context. No mode or stale float survives either lifetime.

## 5. Structural rupture

For the four orthogonal neighbouring faces:

```text
stress_x = abs(P_total_left - P_total_right)
stress_y = abs(P_total_up - P_total_down)
stress   = max(stress_x, stress_y)
```

A registered finite rupture threshold is compared to `stress`. Uniform pressure
on both sides does not rupture. Wood retains threshold `80`; Stone and Boundary
Block remain unbreakable. A rupture writes actual EMPTY identity, canonical
temperature/flags/phase/Air pairing, and creates a dynamic-pressure/Air topology
opening for following Ticks.

## 6. Activity and sleep

Base activity fully writes Matter/Thermal/Reaction and never sets
`ACTIVITY_PRESSURE`. `pressure_activity_propose` is the only production setter.
It runs full-world after rupture settlement, duplicates the exact pressure
update, and sets the pressure bit only when:

```text
abs(predicted_next - current) > 0.001
```

Environment activity compares total pressure. Pending diffusion, relaxation,
stale-field removal and Air work therefore wake; exact equilibria can sleep.

## 7. Pass, binding and allocation contract

The candidate graph is 43 passes and 86 timestamp queries. Relevant storage
counts are pressure `6`, Air scale `7`, Air commit `8`, base activity `7`,
Environment activity `6`, pressure activity `5`, and rupture `8`.

The added pass is immediately before activity reduction. Profiler resolve and
readback are 688 bytes each, 1,376 bytes total. Persistent and full-world
scratch allocation delta is exactly zero.

## 8. Authoritative writers

Reset, direct Material/phase/pressure authoring, scenario staging and Sandbox
edits establish both Current and Next halves. Movement, phase, decay,
combustion and rupture retain identity/phase/Environment hygiene. Static and
Powder replacements clear dynamic pressure through authoring and the full
pressure writer. Invalid authoring rejects without partial commit.

## 9. Vent evidence

Relaxation by itself is never called venting. An opening-attributable claim
requires matched opening/no-opening treatments, identical initial phase load,
an actual structural identity change, new pressure-node topology, and:

```text
drop_open > drop_control + 5.0
```

It also records following-Tick Air or legal ordinary Gas use and exact phase-
family quantity. A marker may outline only the actual opened/occupied Cell; it
may not fabricate pressure, motion, rupture or Steam.

## 10. Explicit exclusions

Pressure-driven Matter force, headspace tokens, reservations, matching, CCL,
phase packets, owner fields, Oxygen quantity, Ash, TE-6 and G9-B/C/D/E are not
active. Historical G5 Water expansion receipts remain source-bound and are not
relabelled as TE-5R1 evidence.
