# TE-3 Water/Steam Phase-Cycle Runtime Evidence — 2026-08-21

- **Disposition:** WATER/STEAM PHASE-CYCLE CANDIDATE / USER REVIEW PENDING
- **Runtime source:** `41467219819c5d0cb3eab8ae22b652449da20480`
- **Decision:** D-024
- **Architecture:** [`ADR-0006`](../architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md)
- **Validation contract:** [`PHASE_THERMODYNAMICS_VALIDATION`](../development/PHASE_THERMODYNAMICS_VALIDATION.md)

## Scope and evidence boundary

This source implements the pressure-decoupled ADR-0006 phase model. One
Ice/Water/Steam Cell is one Water-equivalent quantity and every family
transition is 1:1. Water boiling creates no expansion proposal, no second Steam
Cell and no blocked-expansion pressure. Generic gauge pressure and rupture are
still present. Historical G5 Water-expansion receipts remain valid only for
their historical source; this candidate does not claim that causal chain.

Only `phase_energy_current` and `phase_energy_next` were added as persistent
TE-3 state. Their 2048-squared increment is exactly 33,554,432 bytes. No phase
unit, packet, quantity, pressure, volume, owner link, persistent reservation or
new full-world scratch exists. External implementation ingress is 0 files / 0
lines.

## Runtime graph and ownership

The production graph is 40 timestamped passes and 80 timestamp queries:

```text
00 activity_wake
01 movement_propose
02 movement_claim
03 movement_commit
04 material_flag_hygiene_movement
05 environment_reconcile_movement
06 phase_energy_reconcile_movement
07 air_flow_scale
08 air_transport_commit
09 thermal_stability_scale
10 unified_thermal_commit
11 phase_context_propose
12 phase_thermodynamics
13 expansion_claim
14 expansion_environment_receiver_claim
15 expansion_spawn_commit
16 expansion_pressure
17 environment_blocked_expansion_pressure
18 material_flag_hygiene_phase
19 environment_reconcile_expansion
20 decay
21 material_flag_hygiene_decay
22 phase_energy_hygiene_decay
23 environment_reconcile_decay
24 combustion
25 smoke_claim
26 smoke_environment_receiver_claim
27 smoke_commit
28 material_flag_hygiene_combustion
29 phase_energy_hygiene_combustion
30 environment_reconcile_smoke
31 pressure
32 rupture
33 material_flag_hygiene_rupture
34 phase_energy_hygiene_rupture
35 environment_reconcile_rupture
36 activity_propose
37 phase_activity_propose
38 environment_activity_propose
39 activity_reduce
```

Storage ceilings are: movement reconcile `5 RO + 1 RW = 6`, context `7 + 1 =
8`, thermodynamics `4 + 4 = 8`, phase hygiene `4 + 1 = 5`, phase activity `7 +
1 = 8`, and the separate Sandbox phase edit dispatch `3 + 2 = 5`. TE-2
finishes its scratch consumers before phase context fully overwrites claim;
phase consumes context and fully writes family `NO_PROPOSAL`; expansion claim
then overwrites claim. Later Smoke producers fully overwrite their lifetime.

Tracked 2048-squared allocations are 302,016,816 bytes without profiler and
302,018,096 with the 1,280-byte profiler pair.

## Semantics and fixtures

The local coordinate is `H = S_material(T) + phase_energy`, with `Lf = 80`,
`Lv = 480`, melt `0 C`, boil `100 C`, surface condensation at no more than `80
C` and at least `10 C` colder, and free-Air nucleation at no more than `70 C`
with radius two. TE-2 transfers Q first; phase repartitions the resulting local
H. The D-018 real-sink work predicate, buried ready-Water hold/reversal,
no-sink metastable Steam and shared physics/activity predicates are retained.

Actual runtime/reference state fixtures TE3-F01 through TE3-F15 pass. This
includes 100 closed cycles, partial reversals, freeze/melt, buried ready hold
and reopen completion, K=0 Boundary control, multi-tick radius-two nucleation,
the predeclared traffic-jam grid, open-beaker causal ordering, quantity-only
sealed vessel accounting, staging/reset, movement/identity hygiene,
sleep/wake, CPU/GPU agreement, TE-2 regression, and the explicit no-second-
Steam/no-proposal/no-pressure Water check. Closed-cycle quantity gain and
Water phase-pressure source are both zero.

## Validation receipt

Targeted results on the final implementation line include Core phase tests
`5/5`, TE-3 GPU fixtures `17/17`, Naga/binding/write contracts `4/4`, Windows
tests `164 pass / 1 historical ignored`, activity `29`, combustion `58`,
environment `9`, profiler `5`, pressure `8`, sleep/wake `17`, thermal `13` and
world-integrity `7`, all passing. Workspace all-target check, warnings-denied
clippy, formatting, strict policy audit and diff check pass.

Two pre-final FULL attempts exposed stale expectations rather than runtime
semantic failures: the first retained the old 34-pass benchmark work count;
the second retained the pre-32-MiB headless allocation literal. Both literals
were corrected and the final-source canonical FULL ran exactly once at
`41467219819c5d0cb3eab8ae22b652449da20480`, with zero failures.

Release build count is 1. The one bounded launch and measurement command was:

```text
run_powdergame.bat phase-cycle --smoke-frames 60
```

It exited cleanly on RTX 5090 / DX12 after 60 frames and 14 simulation ticks,
reporting 53.50 wall TPS, sample tick 8, family 3885, Water 3885, Steam 0 and
Ice 0. This short launch ended before visible Steam appeared; it is a bounded
launch/performance observation, not the phase-transition proof. TE3-F09 is the
automated causal evidence and direct visual user review remains pending.

The normal product Inspector remains a 24-byte, at-most-10-Hz interface. The
candidate diagnostics use their dedicated bounded cadence and do not change
that product contract.

## Artifact and deferred work

- EXE: `target/release/powdergame-windows.exe`
- SHA-256: `99745D13A7F5D7323EB5961A3A462A965C446C10CDA4CA9AF04495B0537C87BE`
- Size: 10,094,592 bytes
- Candidate routes: `run_powdergame.bat phase-cycle` and alias
  `run_powdergame.bat te3`
- No-argument Sandbox: unchanged

The candidate starts paused, contains four phase-cycle review scenes and
states `Pressure coupling: DEFERRED / NOT ACTIVE IN THIS TE-3 CANDIDATE`.
`WATER_STEAM_PRESSURE_VOLUME_REDESIGN` is deferred/not started. TE-4,
G9-B/C/D/E, official capture, PR, main merge and user acceptance claim counts
are zero.
