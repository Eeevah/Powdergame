# TE-4I ignition-kinetics implementation evidence — 2026-08-23

## Disposition and provenance

- Start source: `a19753ba087309e4f2a4863915d57b67750f1ad2`
- D-032 authorization: `ae8c04bc42f85c6a78d5960e08f3f4bcef1a28cd`
- Final runtime source: `8d9e8cbe3b6ac651335b5a728ef491abeae4772a`
- Runtime commits: `864159c4bd6cc6f655b36cbaa948ffa6ec4f0ec4`,
  `532bd86eb11a86a49a4b11735e11037170ee23ed`, and
  `8d9e8cbe3b6ac651335b5a728ef491abeae4772a`
- Wiki authority: verified `origin/main`
  `57d7e2bdbab5b9cbc46a4448fd881e7493e12f74`; the user-dirty local checkout
  was not modified.
- Status: **TE-4I IGNITION KINETICS IMPLEMENTATION CANDIDATE / AUTOMATED
  VALIDATION PASS / USER REVIEW PENDING**.
- ADR-0012 remains **PROPOSED / IMPLEMENTATION EVIDENCE AVAILABLE / USER
  ARCHITECTURE REVIEW PENDING**.

The TE-4D v1/v2/v3 and D-031 supplement receipts remain immutable blocked
history. None was patched, rerun, or rebound to this production source.

## Implemented contract

Oil uses `48/2/50/6/1/2/4`; Wood uses `60/1/50/5/1/2/4` for
budget/base/bucket-width/max/decay/flame-bonus/flame-cap. Exposure is a
canonical u6 in flag bits `2..3` and `28..31`, with masks `0x0000000C`,
`0xF0000000`, and `0xF000000C`. The complete combustion-owned mask is
`0xF000FFFF`; decay retains bits `16..27`.

At the settled `COMBUSTION_STAGE_SNAPSHOT`, an orthogonal in-domain EMPTY
neighbour with positive Air mass grants binary access. Atmosphere and positive
LowPressure qualify; exact Vacuum and occupied GAS Matter do not. Air is not
consumed or interpreted as Oxygen. Previous-current `FLAME_EVENT` contributes
only while the target itself is thermally eligible, so no same-stage recursive
chain exists.

Core owns finite `chemical_q_per_tick` values Oil `15` and Wood `8`. The
512-byte `16 x 32` GPU table carries the derived temperature deltas `6` and `4`
with exact offsets and fail-closed packing. Consume-before-emission leaves Oil
ticks `1..599` and Wood ticks `1..899` emitting, for gross totals `8,985` and
`7,192`; ticks `600/900` consume with zero Heat, Flame, Smoke, and Q.

## Pass, binding, scratch, and settle result

The final graph has 42 timestamped passes and 84 queries. Two profiler resolve
buffers total 1,344 bytes. New persistent state and new full-world scratch are
both zero bytes; the existing proposal scratch and 512-byte combustion table
are reused. The two new passes each bind four read-only storage buffers and one
read-write storage buffer, plus two uniforms, for five storage bindings. All
production passes remain at or below the established eight-storage ceiling.

```text
 0 activity_wake
 1 movement_propose
 2 movement_claim
 3 movement_commit
 4 material_flag_hygiene_movement
 5 environment_reconcile_movement
 6 phase_energy_reconcile_movement
 7 air_flow_scale
 8 air_transport_commit
 9 thermal_stability_scale
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
24 ignition_exposure_propose
25 combustion
26 smoke_claim
27 smoke_environment_receiver_claim
28 smoke_commit
29 material_flag_hygiene_combustion
30 phase_energy_hygiene_combustion
31 environment_reconcile_smoke
32 pressure
33 rupture
34 material_flag_hygiene_rupture
35 phase_energy_hygiene_rupture
36 environment_reconcile_rupture
37 activity_propose
38 phase_activity_propose
39 environment_activity_propose
40 ignition_activity_propose
41 activity_reduce
```

Pass 24 fully writes proposal as exposure/ignite/Air context. Pass 25 consumes
that context and fully overwrites proposal as a Smoke request. Passes 26..28
consume the Smoke lifetime. Smoke hygiene, phase-energy hygiene, Environment
reconcile, and Current copies settle Matter, temperature, flags, phase energy,
and Air before pass 40 observes the next-stage topology.

## Production fixture receipt

Actual Core transitions, production WGSL, real Simulation ticks, bounded GPU
readback, and candidate tests cover TE4I-F01 through F17:

- F01/F02: Oil exposure is `2/4/6` after three threshold ticks and then
  `5/4/3` after Air removal; ignition ticks are Oil `24` at threshold and `12`
  at +100 C, Wood `60` and `20`. No first-tick ignition occurs.
- F03/F05: previous-tick flame changes Wood exposure `57 -> ignition`; an
  adjacent Cell at `58` becomes `59`, not burning, until the following tick.
- F04/F15: partial exposure survives short cooling, decays to zero under long
  cooling, remains runnable until zero, then may sleep; sleep-on/off lifecycle
  state is equivalent for equal executed ticks.
- F06: the connected candidate surface/front fixture progresses from local
  production Heat/Flame and does not flash on its first tick.
- F07: Atmosphere mass `1.0` and LowPressure mass `0.1` are eligible and remain
  positive; exact Vacuum, Steam-occupied, and Smoke-occupied controls are not.
- F08: at tick N the burning Wood advances fuel `10 -> 11`, emits one Smoke,
  moves the sole Air parcel mass `1.0` losslessly to the real receiver, and
  settles Current equal to Next. At N+1 the now inaccessible source clears
  burning/flame, keeps fuel `11`, adds no chemical heat, and emits no second
  Smoke.
- F09/F10: actual bounded lifecycles consume Oil at 600 and Wood at 900 with
  zero final-tick emission, exact gross totals `8,985/7,192`, finite
  deposited/clipped closure, and canonical EMPTY state.
- F11: extinguish preserves fuel; re-eligibility rebuilds dose and reignition
  continues from prior finite fuel progress rather than restoring it.
- F12/F13: movement and density swap transport exposure/fuel without residue
  or duplication; Void exit, replacement, decay, rupture, consumption, and
  non-combustible hygiene clear combustion-owned state.
- F14: Draw/Erase, staging, presets, scenario upload, reset, and candidate reset
  establish canonical Current/Next state; the FireHeat seed now has a real Air
  surface and progresses through production semantics.
- F16: Core rate/context/descriptor fixtures and the bounded GPU matrix agree
  on eligibility, exposure, ignition, extinguish, flags, fuel, and temperature.
- F17: TE-2 Air/thermal and accepted TE-3 phase suites pass; no Water pressure,
  TE-5, or TE-6 behavior was added.

## Validation attempts

Targeted suites passed: Core `117`; GPU combustion `64`; activity `29`;
sleep/wake `18`; phase `17`; thermal `13`; TE-2 small-delta `1`; TE-2
transport `5`; Environment `9`; parallel `9` with `3` ignored; profiler `5`;
WGSL `5`; scenarios `10` plus GPU reset `3`; Windows candidate/product `182`
with `1` ignored. Formatting, all-target workspace check, clippy with warnings
denied, strict policy audit, and diff checks passed.

Canonical FULL attempts are reported without hiding invalid sources:

1. `864159c...`: failed two benchmark evidence-group tests because the two new
   pass names were absent from benchmark grouping.
2. `532bd86...`: failed one FireHeat GPU-reset fixture because its hot Wood seed
   was buried and correctly lacked a qualifying Air face.
3. final source `8d9e8cb...`: PASS. Benchmark `28`, Core `117`, GPU and scenario
   suites, Windows `182 passed / 1 ignored`, and doc tests all passed.

Therefore FULL attempts are `3`; successful FULL on the final runtime source is
`1`. G8/G8-C, TE-5, TE-6, and official capture counts are zero.

## Candidate and artifact

The canonical EXE is
`target/release/powdergame-windows.exe`, size `10,141,696` bytes, SHA-256
`27D92287931421560027EF4D554DA26BBB50C5DE1565D75E52D1BC406A2A6081`.
Exactly one release build, one TE-4 candidate bounded launch check, and one
bounded measurement ran via:

```powershell
run_powdergame.bat ignition-kinetics --smoke-frames 60
```

It completed 60 frames and 15 simulation ticks on RTX 5090/DX12, reported
wall TPS `57.31`, four candidate rows at sample tick `8`, and exited cleanly.
The frame limit does not prove user-visible combustion behavior. Equivalent
routes are `run_powdergame.bat ignition-kinetics` and
`run_powdergame.bat te4`.

The verified local shortcut is
`C:\Users\mdkap\Desktop\Powdergame TE-4 Ignition Kinetics.lnk`, targeting the
canonical EXE with `--ignition-kinetics-candidate` and working directory
`C:\Users\mdkap\source\repos\Powdergame-g8b`.

The four production scenes cover spike/sustained Heat, previous-flame versus
inert Heat, a connected surface frontier, and Atmosphere/LowPressure/Vacuum/
self-Smoke. Candidate diagnostics are fixed candidate-only rows and do not
change the normal Inspector contract.

## Boundary and next review

No Oxygen quantity, Ash, Pressure coupling, final Fire/Smoke presentation,
TE-5, TE-6, or G9-B/C/D/E work is claimed. Automated evidence does not accept
ADR-0012. Direct user review must still judge the twelve candidate behaviors
listed in the TE-4 plan and either accept, revise, or reject the architecture.

`LESSON_PROMOTION: NONE` — the implementation defects were caught by the
required final-source validation and repaired without revealing a new reusable
failure class beyond PG-L034/PG-L035 and the existing Wiki evidence/snapshot
contracts.
