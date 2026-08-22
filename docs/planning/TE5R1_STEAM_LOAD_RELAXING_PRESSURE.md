# TE-5R1 Steam-Load Relaxing Pressure — Source-Realizability Gate

- **Decision:** D-037
- **ADR:** [ADR-0014](../architecture/decisions/ADR-0014-post-phase-steam-load-relaxing-pressure.md)
- **Source baseline:** `12b49dc07c8d875de55a048013a01090d38345a9`
- **Gate state:** FRESH REVIEW PASS (Critical 0 / High 0); implementation candidate in validation
- **Required pass condition:** unresolved Critical `0`, High `0`

## Exact production order that the candidate reuses

The current 42-pass source settles generic expansion pressure before the local
pressure pass:

```text
12 phase_thermodynamics
13 expansion_claim
14 expansion_environment_receiver_claim
15 expansion_spawn_commit
16 expansion_pressure
17 environment_blocked_expansion_pressure
18 material_flag_hygiene_phase
19 environment_reconcile_expansion
   copy pressure_next -> pressure_current
20..31 decay/combustion and identity/Environment hygiene (no pressure writer)
32 pressure
   copy pressure_next -> pressure_current
33 rupture
34..36 rupture hygiene/reconcile
   settle Matter/phase/Air Current
37..40 existing activity proposers
41 activity_reduce
```

R1 inserts only `pressure_activity_propose` at 41 and moves reduction to 42.
The resulting graph is 43 passes and 86 timestamp queries.

## Source-bound value and lifetime table

| Value | Authoritative writer | First valid point | Overwrite/settle point | Consumer | Clear/set owner | Storage count |
|---|---|---|---|---|---|---:|
| settled Material | movement/phase/decay/combustion/rupture production writers | after each paired hygiene settle | next identity transaction | pressure, Air, rupture, activity | identity writers; authoring writes both halves | pressure 6; rupture 8 |
| settled phase energy | TE-3 phase and identity hygiene | post-phase settle; re-settled after later identity paths | next phase/identity transaction | Steam target in pressure and pressure activity | TE-3 writers; invalid authoring rejected | pressure 6; activity 5 |
| generic pressure impulse | `expansion_pressure`, or exclusively Environment-blocked writer | pass 16/17 `pressure_next` | expansion copy to Current, then local pressure pass | pressure update as settled `q` | first writer fully writes; second only adds winner/receiver failure | writers 8/7; pressure 6 |
| dynamic pressure Current | reset/editor or previous pressure settle | tick start and post-expansion settle | pressure pass writes Next; copy settles | Air scale, pressure, rupture, Environment/pressure activity, diagnostics | reset/editor both halves; pressure full writes every Cell | scale 7; pressure 6; rupture 8 |
| Steam target | pure function of settled Material + phase energy | pressure/pressure-activity invocation | never stored | local update and exact-update activity | no buffer; invalid Steam state fails closed | 6 / 5 |
| derived Air background | pure function of canonical EMPTY Air energy | each total-pressure consumer | never stored persistently | Air scale/commit scratch, rupture, Environment activity | Environment writers maintain EMPTY/Matter pairing | scale 7; rupture 8; env activity 6 |
| Air donor scale | Air scale pass writes proposal scratch as `f32` | end pass 7 | thermal stability overwrites proposal at pass 9 | Air commit pass 8 | Air scale fully writes every Cell; non-donor zero | 7 then 8 |
| Air total pressure | Air scale pass writes claim scratch as `f32` | end pass 7 | phase context overwrites claim at pass 11 | Air commit receiver-scale recomputation | Air scale fully writes every Cell | 7 then 8 |
| Air receiver scale | pure gather in Air commit from current Air + total-pressure scratch | each transfer evaluation | never stored | donor/receiver min for actual transfer | commit formula only | commit remains 8 |
| Air mass/energy Next | Air commit | end pass 8 | copy to Current immediately after pass 8 | later thermal/phase and following Tick Air | commit self-writes all Cells | 8 |
| total pressure at rupture face | pure `dynamic + EMPTY air_energy/293.15` | rupture invocation after pressure settle | never stored | opposing-face differential | rupture adds each term once | 8 |
| base cell activity | base activity proposer | pass 37 | full overwrite next Tick pass 37 | later OR proposers and reduction | base fully writes only Matter/Thermal/Reaction | 7 |
| Environment activity | Environment activity proposer | pass 39 | base full-write next Tick | reduction | Environment pass ORs Thermal/Environment from exact Air-work predicate using total pressure | 6 |
| pressure activity | dedicated pressure activity proposer | pass 41 after settled rupture | base full-write next Tick | reduction/wake next Tick | sole `ACTIVITY_PRESSURE` setter; no sleeping skip | 5 |
| chunk activity/state | reduce then next Tick wake | pass 42 / next pass 0 | next reduction/wake | all existing sleep-aware passes | unchanged reduce/wake ownership | unchanged |

No consumer reads phase-context scratch for pressure. No movement pass reads
dynamic or total pressure. No consumer adds Air background or dynamic pressure
more than once.

## Binding-realizability details

The Air commit cannot add a ninth input. R1 therefore changes only its existing
scratch meaning: proposal is donor scale and claim is total pressure. The Air
scale pass computes both from Current state. Commit recomputes receiver
capacity for self and neighbouring receivers from the total-pressure scratch
and current Air, preserving the existing mass and energy caps. `chunk_state`
stays bound, so disabled sleeping/sleeping faces do not silently transport.

Rupture also stays at eight: it removes movement class and reads Air energy in
that slot. This is safe only because the immediately settled pressure pass
fully writes blocked Static/Powder nodes to zero. EMPTY and Liquid/Gas are the
only dynamic nodes; rupture may therefore sample settled dynamic pressure
without reclassifying it.

## Current/Next and authoring hygiene

World creation/reset write pressure zero to both halves, canonical phase energy
to both halves, and paired canonical Air state. `write_material`, `write_phase_energy`,
`write_pressure`, Environment test staging and Sandbox Draw/Erase write both
halves; identity replacement clears pressure and establishes canonical Air and
phase energy. Movement, phase, decay, combustion and rupture retain their
paired identity/phase/Environment hygiene and settle copies. R1 adds no new
state requiring an authoring path.

## Field-specific edge and causal fixtures

Air and dynamic-pressure missing faces are sealed/no-flux. Matter keeps its
existing Void target. Every sealed F fixture uses an explicit in-domain wall
ring. Every vent result uses an opening/no-opening pair, predeclared margin,
identity/topology receipt, following-Tick Air and ordinary-Gas movement, and
quantity accounting.

## Gate attacks required of the fresh reviewer

- any hidden phase-context or pre-transition dependency;
- impulse double-write or wrong settle order;
- an earlier pressure-bit setter that cannot be cleared;
- Air donor/receiver conservation lost by scratch reinterpretation;
- a ninth storage binding or stale scratch consumer;
- total pressure added twice or omitted from an Air-work predicate;
- rupture background requiring Air mass or a class input that is unavailable;
- sleeping chunks skipping pending pressure/Air work;
- reset/editor half-state drift;
- pressure-dependent Matter movement sneaking back in;
- an edge or vent claim relying on domain-edge Matter conservation;
- any new persistent/full-world state or test-only replacement simulator.

Any unresolved Critical/High finding blocks implementation. The primary author
must not repair the design during that review or silently select another model.

## Gate result

The independent [source gate review](../adversarial-reviews/TE5R1_STEAM_LOAD_RELAXING_PRESSURE_SOURCE_GATE.md)
reported unresolved Critical `0`, High `0`, Medium `0`, Low `0`. Implementation
was therefore authorized under D-037 without changing this table. The candidate
uses exactly 43 passes / 86 queries, adds no persistent or full-world scratch
allocation, and is governed by the [runtime specification](../specs/STEAM_LOAD_RELAXING_PRESSURE_SPEC.md)
and [validation matrix](../development/STEAM_LOAD_RELAXING_PRESSURE_VALIDATION.md).
