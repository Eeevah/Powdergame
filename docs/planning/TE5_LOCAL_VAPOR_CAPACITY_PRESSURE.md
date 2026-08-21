# TE-5C Local Vapor Capacity and Gauge-Pressure Equilibrium

- **Authorization:** D-020
- **Status:** DESIGN BLOCKED / one-shot proof failed locked open-capacity control
- **Runtime:** NOT STARTED
- **Baseline:** `6a1c83fad702d18f2d24365a4fc747ab74225f5c`

## Goal and stop boundary

Replace the rejected TE-5B completion token with one current-state law that
can preserve 1:1 phase quantity and the frozen G5 causal meaning without new
persistent/full-world allocation. Maximum successful stop is a proposed
ADR-0008 design candidate pending user architecture review. Any unresolved
Critical/High stops TE-5C DESIGN BLOCKED and requires a later decision that
explicitly permits persistent phase-volume state.

## Reuse-first design

Reused unchanged as contracts: TE-3 phase energy and H, TE-2 Current/Next
thermal result, phase/property tables, existing `pressure[]`, four-neighbour
pressure diffusion, Wood threshold `80`, proposal scratch after Smoke,
activity/wake halo and timestamp profiler. New runtime buffers, velocity,
fragments, reservations, extra Steam and external implementation ports are
excluded.

The primary scratch layout adds one projected `vapor_capacity_sum` dispatch
between Smoke settle and pressure. It fully writes proposal as `f32 D_e`;
pressure and phase activity consume it; next movement overwrites it as u32.
Projected total: 41 passes, 82 queries, 1,312 profiler bytes, allocation delta
zero. Direct two-hop gather is the disclosed fallback comparison, not selected.

## Candidate law and comparisons

The normative formula is in
[`LOCAL_VAPOR_CAPACITY_PRESSURE_SPEC`](../specs/LOCAL_VAPOR_CAPACITY_PRESSURE_SPEC.md).
Compare only for disclosure:

- binary target at any deficit: rejected as discontinuous/false-threshold risk;
- locked linear target: primary;
- smooth nonlinear response: disclosure only.

The formula must not be changed after proof output. Proportional sharing is
also fixed; an underuse failure blocks rather than authorizing redistribution.

## Fixture matrix

| Fixture | Locked meaning |
|---|---|
| F01–F04 | elementary demand/capacity/reference targets |
| F05–F06 | vacancy-walk and finite-headspace crossing |
| F07 | open plume/no false Wood rupture control |
| F08–F09 | partial boiling, condensation and relief |
| F10 | generic G5 event-pressure separation |
| F11 | heat→phase E→1:1 Steam→capacity exhaustion→gauge target→Wood rupture→EMPTY vent→decline |
| F12 | sealed/open/Boundary controls |
| F13 | chunk/sleep/scratch/reset obligations |

F11 forbids extra Steam, token, boiler-specific explosion and combustion-made
opening. Its proof trace is pure-model evidence only; a future atomic source
must create new source-bound evidence.

## User-review choices if the design survives

1. accept/revise radius-1 proportional capacity meaning;
2. accept/revise occupancy-only equivalence of Atmospheric and Vacuum EMPTY;
3. accept/revise the linear compression curve and `100.0` maximum;
4. accept/revise EMPTY venting of the whole gauge field, including generic pressure;
5. confirm finite-headspace and open-plume product meaning;
6. separately authorize any runtime implementation.

Product edge mode, derived-Air/background pressure, structure differential and
Vacuum combustion remain later full TE-5/TE-4 decisions.

## Current disposition

The one-shot result reported the named vacancy-walk, finite-headspace, partial,
generic, pressure/vent and atomic pure-model fixtures as passed, but failed the
predeclared asymmetric reachable-capacity control. Independent review found
that several reported fixture checks do not implement their full named
obligations, so those properties are not established. The proportional rule
can discard a share at one saturated phase Cell while leaving another Cell
spuriously compressed even though a complete local EMPTY assignment exists.

The same review recorded Critical `0` / High `6`: false capacity allocation,
internal-EMPTY vent conflation, irreversible phase-pressure provenance,
unreachable downward capacity, activity/snapshot/binding infeasibility, and
reference-receipt overclaim. Review SHA-256 is
`d0d26585326d79cfe60ab0fd0a334e9537e6bedc8d41059e5e129caa08d2edf2`.

This unresolved design-level counterexample stops **TE-5C DESIGN BLOCKED**.
The response curve, radius and sharing rule were not changed. Per D-020, the
next architecture decision must explicitly permit persistent phase-volume
state; another stateless token/impulse variant is forbidden.
