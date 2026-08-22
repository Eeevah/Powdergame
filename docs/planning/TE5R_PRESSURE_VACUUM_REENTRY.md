# TE-5R0 Pressure / Vacuum Architecture Re-entry

- **Status:** DESIGN BLOCKED / runtime not started
- **Decision:** D-035
- **Candidate:** [ADR-0013](../architecture/decisions/ADR-0013-local-relaxing-phase-load-pressure.md)
- **Cost override:** D-036; Wiki refresh deferred until 2026-09-01

## Objective

Replace exact phase-volume ownership attempts with one local, bounded pressure
model that preserves accepted TE-2 Air and TE-3 quantity/enthalpy. The task
stops at architecture review; it creates no runtime or reference proof.

## Preserved and superseded constraints

Preserved: 1:1 phase quantity, accepted phase energy, no extra Steam, one
foreground Matter per Cell, Air mass/energy authority, exact Vacuum, local
GPU-bounded work, eight storage bindings and source-bound evidence.

Superseded: distinct EMPTY extent ownership, exact zero pressure whenever a
capacity assignment exists, owner-exact pressure recovery, matching/CCL
certification and permanent pressure in a sealed region.

## Exact source audit

Current production has 42 passes/84 queries. The reviewed but blocked 44-pass
projection was:

```text
 0 activity_wake
 1 movement_propose_total_pressure
 2 movement_claim
 3 movement_commit
 4 material_flag_hygiene_movement
 5 environment_reconcile_movement
 6 phase_energy_reconcile_movement
 7 air_flow_scale_total_pressure
 8 air_mass_transport_commit
 9 air_energy_transport_commit
10 thermal_stability
11 unified_thermal
12 phase_context_propose
13 phase_thermodynamics
14 expansion_claim
15 expansion_environment_receiver_claim
16 expansion_spawn_commit
17 expansion_pressure
18 environment_blocked_expansion_pressure
19 material_flag_hygiene_phase
20 environment_reconcile_expansion
21 decay
22 material_flag_hygiene_decay
23 phase_energy_hygiene_decay
24 environment_reconcile_decay
25 ignition_exposure_propose
26 combustion
27 smoke_claim
28 smoke_environment_receiver_claim
29 smoke_commit
30 material_flag_hygiene_combustion
31 phase_energy_hygiene_combustion
32 environment_reconcile_smoke
33 local_relaxing_pressure
34 differential_rupture
35 material_flag_hygiene_rupture
36 phase_energy_hygiene_rupture
37 environment_reconcile_rupture
38 activity_propose
39 phase_activity_propose
40 environment_activity_propose_total_pressure
41 ignition_activity_propose
42 pressure_activity_propose
43 activity_reduce
```

Representative future storage counts:

| Pass | RO | RW | Total | Source-bound reason |
|---|---:|---:|---:|---|
| movement proposal | 6 | 2 | **8** | adds pressure and Air energy; keeps existing scratch/class/density/chunk |
| Air scale | 5 | 2 | **7** | adds dynamic pressure to current material/Air/chunk inputs |
| Air mass commit | 7 | 1 | **8** | separate output avoids ninth binding |
| Air energy commit | 7 | 1 | **8** | same inputs, separate output |
| local pressure | 5 | 1 | **6** | material, phase energy, pressure, class, chunk -> pressure Next |
| differential rupture | 5 | 3 | **8** | drops redundant class table after settled blocked-node zero; adds Air energy |
| base activity | 7 | 1 | **8** | unchanged and still maxed |
| Environment activity | 5 | 1 | **6** | adds dynamic pressure |
| pressure activity | 4 | 1 | **5** | material, phase energy, pressure, class -> shared activity |

Uniform bindings are not storage counts. Future structural tests must confirm
the exact layouts; this table is not WGSL/device evidence.

## Scratch, profiler and memory

Air scale fully overwrites proposal/claim as `f32` donor/receiver scales. Both
new commits consume that lifetime. Thermal stability then fully overwrites
proposal, phase context fully overwrites claim, and later phase/Smoke writers
retain their current full-write boundaries.

No persistent or full-world scratch allocation is added. The extra two passes
raise profiler queries to 88: two 704-byte buffers, 1,408 bytes total. With the
current no-profiler totals, projected tracked memory is:

| World | No profiler | With 44-pass profiler |
|---|---:|---:|
| 256² | 4,721,328 B | 4,722,736 B |
| 2048² | 302,016,816 B | 302,018,224 B |

## Work sequence

- [x] verify baseline and preserve dirty Wiki;
- [x] reread blocked TE-5B/C/D/X/Q histories;
- [x] audit current pressure, Air, phase, movement, rupture and activity paths;
- [x] record D-035 architecture reset and D-036 cost override;
- [x] draft ADR/spec/validation and 44-pass feasibility;
- [x] complete fresh-context adversarial review;
- [x] mark DESIGN BLOCKED for unresolved Critical `0` / High `3` / Medium `3`;
- [ ] run local docs/memory validation;
- [ ] make one coherent `[skip ci]` local commit;
- [ ] inspect `.github/workflows/**` and push at most once only if CI skipping is certain;
- [ ] obtain a new user architecture decision before any revision.

## Independent-review stop

The independent review found three unresolved High source contradictions:

1. the pressure pass cannot recover the pre-transition TE-3 gas-facing snapshot;
2. the documented fresh generic impulse is not present in its projected inputs;
3. unchanged base activity cannot sleep at an exact nonuniform equilibrium.

The projection is therefore not implementable as one coherent contract even
though its numerical binding counts are at most eight. Per D-035, this task
stops **TE-5R0 DESIGN BLOCKED** and does not repair the pass graph, alter the
formula or select another model. Medium risks also remain around vent causality,
moving-source trail feedback and the pressure-only meaning of a sealed edge.

## Deferred work

`WIKI_PROJECT_SNAPSHOT_REFRESH` is **DEFERRED UNTIL 2026-09-01 DUE TO GITHUB
ACTIONS BUDGET CONSERVATION**. No Wiki clone, branch, push, PR, CI poll or merge
occurs in this task. Runtime implementation, coefficient exploration, product
reservoir mode, Oxygen/Ash, TE-6 and G9-B/C/D/E remain not started.

## Required next decision

Before any revision, the user must explicitly decide how to make the Water
surface snapshot, generic impulse transaction and activity ownership one
source-realizable contract. The same decision must retain or supersede the
remaining zero-Air Vacuum, dissipation, movement-feedback and edge semantics.
No implementation checklist is active while the design is blocked.
