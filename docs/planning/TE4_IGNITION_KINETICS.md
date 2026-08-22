# TE-4D Ignition Kinetics Plan

Status: **TE-4I IMPLEMENTATION CANDIDATE / AUTOMATED VALIDATION PASS /
ADR-0012 PROPOSED / USER REVIEW PENDING**.

## Reuse and exact inventory

- Immediate-threshold production behavior and Oil/Wood constants remain live.
- `flags`: combustion 0..1 and 4..15; decay 16..27; candidate u6 2..3 + 28..31.
- Movement commit carries flags but is already 8-storage; identity edits clear
  both flag halves; current combustible hygiene mask would erase candidate bits.
- Combustion: 8 storage + 2 uniform bindings, pass 24 of 40; `proposal` is fully
  overwritten then consumed by Smoke claim/receiver/commit.
- Activity propose: 8 storage + 1 uniform, pass 36; property table binding 8 may
  grow without a new binding.
- Descriptor upload: Rust logical 20 bytes, manual 32-byte stride, WGSL 32
  bytes, 16 entries/512 bytes; 12 bytes padding available.
- Profiler: 40 passes / 80 timestamp queries.
- External copied/translated/vendored implementation: 0 files / 0 lines.

## Candidate layouts

Packed u6 is selected by D-029 and adds zero persistent/scratch bytes. Source
binding limits require two future logical passes, producing a conservative
42-pass/84-query projection. It
requires exact mask, movement, identity, authoring, activity and Inspector
tests. Descriptor padding is proposed as three u32 words: dose budget; packed
base/max/bucket metadata; packed decay/flame/cap metadata. ADR-0012 fixes their
byte offsets and 8-bit subfields. The base activity pass remains at eight
storage bindings and adds the existing combustion table as a second uniform;
the table allocation is reused, not duplicated. Finite/range validation,
serialized table hash and bounded candidate-only exposure diagnostics remain
future implementation work; the normal Inspector contract is not expanded.
Core chemical Q is compiled with the Material heat capacity into the existing
GPU delta-T slot. This avoids a capacity binding and preserves the current cap;
validation must distinguish finite gross Q from deposited and clipped Q.

Dedicated u32 Current/Next adds 524,288 bytes at 256² and 33,554,432 bytes at
2048². Because movement commit and combustion are already at eight storage
bindings, the conservative minimum projection is 42 passes/84 queries using a
movement-reconcile pass and a pre-combustion exposure pass. The latter writes
exposure Next and a fully overwritten ignition request into `proposal`; the
existing combustion pass consumes it before overwriting `proposal` for Smoke.
Identity-hygiene fusion, activity visibility and all binding rows remain
unproven, so this is only a fallback estimate.

## Selected v2 identity

D-029 fixes Oil `48/2/50/6/1/2/4`, Wood `60/1/50/5/1/2/4`, packed u6,
non-Vacuum orthogonal EMPTY Air-face access, Oil/Wood gross Q `15/8`, and the
consume-before-emission final tick. A manifest-bound reference must execute 13
required fixtures while four production fixtures remain `NOT_ESTABLISHED`.

The exactly-once process completed `1/1` with 100,000 sequences and 10,000
grids. F07 events were ticks 20/40/60/80. F08 first/max/completion were
20/5/173. All required path counters were positive. The result is reference-
only; actual TE-2 transport, GPU sleep/wake, CPU/GPU agreement and TE-2/TE-3
regression remain F01/F14/F16/F17 `NOT_ESTABLISHED`.

Fresh review found Critical `0` / unresolved High `3` / Medium `1`. Counter
aggregation does not prove every named transaction, same-tick Smoke may remove
the sole Air face after the early predicate, and F08 lacks a frozen frontier
oracle. The process result is retained without patch/rerun; architecture blocks.

Projected final order is current passes 0..23, ignition exposure at 24,
shifted combustion/Smoke at 25..31, pressure at 32, rupture/hygiene at 33..36,
base/phase/Environment activity at 37..39, ignition activity at 40, and reduce
at 41. Both new passes have six storage bindings plus params/descriptor
uniforms. Proposal is fully written at 24, consumed and overwritten for Smoke
at 25, then consumed by the existing Smoke transaction. Persistent and scratch
world-state deltas are zero; profiler buffers total 1,344 bytes.

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

## Historical v2 decision boundary

V2 ended by requiring a later user decision on sole-Air timing, receipt
provenance and an independent oracle. D-030 supplied that decision without
repairing v2.

## D-030 v3 closure program

D-030 resolves the v2 ambiguity with `COMBUSTION_STAGE_SNAPSHOT`, mutation-
derived audited receipts and a pre-evidence frozen independent exact frontier
oracle. It narrows the coefficient statement to
`USER_SELECTED_AND_VALIDATED / NOT_CLAIMED optimality`. V1/v2 remain immutable
blocked history.

The one v3 process completed successfully with the locked `13/4/0/0/0`
reference/deferred/fail/unexpected/zero-path aggregate and exact F07/F08/F15B
results. The live-source feasibility projection remains the 42-pass list above,
84 queries, 1,344 query bytes, six storage bindings in each new logical pass,
zero persistent bytes and zero scratch bytes. Post-Smoke settle makes the next-
stage Air loss visible to ignition activity before reduction.

Fresh review found Critical `0` / unresolved High `3` / Medium `1` / Low `1`.
F15B hardcodes rather than derives next-stage Air access, the auditor trusts
SUT semantic names/events, and F09 does not derive chemical heat and the final
consumption tick from one lifecycle. Therefore v3 is **DESIGN BLOCKED**. A
future user decision is required before any new identity; v3 may not be
patched or rerun and runtime remains unauthorized.

`LESSON_PROMOTION: NONE` for v3: PG-L035 and the verified Wiki snapshot-
precondition-lifetime workflow already encode the reusable same-stage
invalidation rule. No new reusable failure was found.

## D-031 targeted supplement result

The frozen supplement ran exactly once and completed `1/1`. It published 1,565
full reduced-model before/after records, continuous Oil/Wood lifecycles and
cap controls. Its own auditor accepted 1,527 transitions and rejected 38; the
snapshot self-re-audit reproduced Oil/Wood gross Q `8,985/7,192` and zero-
emission final consumption ticks.

Fresh review found Critical `0` / unresolved High `3` / Medium `2`. The F15B
world is not settled between Smoke/Air mutation and the next Air decision;
semantic classes remain caller-selected; and Air displacement does not prove
receiver topology/claim. Negative-control family coverage and the named third-
party audit are also overstated. The supplement is **BLOCKED**, immutable and
not eligible for rerun. Runtime remains unauthorized.

`LESSON_PROMOTION: NONE` because the Wiki Evidence/fixture integrity and
Snapshot precondition lifetime contracts already cover these failures.

## D-032 implementation-first gate

D-032 preserves all failed synthetic evidence byte-for-byte and authorizes the
42-pass production candidate described above. The remaining questions must be
answered by actual Core transitions, WGSL, Current/Next settle, Smoke receiver
transactions, bounded GPU readback and the canonical candidate. No new Python
reference identity or synthetic semantic auditor is permitted. ADR-0012 stays
Proposed until direct user review.

## D-032 production result and manual review

Final runtime source `8d9e8cbe3b6ac651335b5a728ef491abeae4772a`
implements the projected 42-pass graph, locked coefficients, packed-u6 state,
binary Air-face rule, finite chemical heat, activity/wake integration, four
candidate scenes, and fixed diagnostics. The graph is 84 queries/1,344 bytes;
both new passes use five storage bindings; new persistent and scratch state are
zero. See the
[`source-bound evidence`](../evidence/THERMAL_ENVIRONMENT_TE_4_IGNITION_KINETICS_2026-08-23.md).

Direct user review remains:

1. Short Heat spikes do not ignite Oil or Wood.
2. Sustained Heat ignites Oil sooner than Wood.
3. Greater excess temperature ignites faster.
4. Cooling visibly reduces pending exposure.
5. Flame accelerates without a same-tick recursive chain.
6. Connected fuel burns from a surface/frontier rather than all at once.
7. Atmosphere and positive LowPressure permit ignition.
8. Exact Vacuum does not permit ignition or sustain.
9. In Scene 4, press `N` once and verify the fixed `(209,110)` target row is
   exactly Smoke, Smoke count is `1`, receiver `(209,111)` holds the displaced
   Air, and the candidate-only outline surrounds that real target. Press `N`
   again and verify the source is extinguished with fuel unchanged and no
   second Smoke. Source Air loss alone is not creation evidence.
10. Fuel is finite and the consumption tick emits nothing.
11. No Ash, Oxygen quantity, or Pressure behavior is implied.
12. Reset is exact.

Successful automation leaves ADR-0012 Proposed and TE-4I user review pending.
The Scene 4 observability remediation and its exact state receipt are recorded
in [`TE4 Scene 4 remediation`](../evidence/TE4_SCENE4_SMOKE_OBSERVABILITY_REMEDIATION_2026-08-23.md).
