# Thermal Environment production inventory

- **Audited source:** `1a722d239a16bade5772688fa822465d5cef4602`
- **Scope:** implemented TE-1 production pass graph, writers, bindings, memory, scratch and occupancy paths
- **Runtime status:** TE-1 implemented; Air transport/thermal exchange/pressure coupling not started

## 1. Current pass order

| # | Pass | Current/read inputs | Next/scratch outputs | Settle boundary | Ownership/activity effect |
|---:|---|---|---|---|---|
| 0 | activity wake | chunk activity/stable/edit-wake | chunk state/wake reason | edit-wake cleared after all chunks observe it | next-tick execution mask |
| 1 | movement propose | material, class/density, chunk state | proposal, marker | none | candidate only |
| 2 | movement claim | proposal, arbitration, chunk state | claim | none | winner only |
| 3 | movement commit | material/temperature/flags Current, claim | corresponding Next | deferred | move, density swap, Void exit |
| 4 | movement flag hygiene | material/flags Next | sanitized flags Next | deferred | target identity ownership |
| 5 | movement Environment reconcile | Matter Current/Next, claim, Air Current | Air Next | joint Matter/temperature/flags/Air copy | exact Volume Exchange |
| 6 | thermal | material/temperature Current, properties | temperature Next | temperature copied | Matter thermal field only |
| 7 | phase transition | material/temperature Current, phase table | material Next, proposal, activity marker | deferred | self identity and expansion request |
| 8 | expansion claim | material Current, proposal | claim | none | target winner |
| 9 | expansion Environment receiver claim | material/claim/Air Current | receiver claim | scratch retained | target+1 arbitration |
| 10 | receiver-gated expansion spawn | material/temperature Current, claims | material/temperature/flags Next | deferred | EMPTY→Matter only with receiver |
| 11 | expansion pressure | phase/proposal/claim, pressure Current | pressure Next | deferred | ordinary blocked expansion source |
| 12 | Environment-blocked expansion pressure | phase/proposal/both claims, pressure Next | pressure Next | deferred | failed receiver source exactly once |
| 13 | phase flag hygiene | material/flags Next | sanitized flags Next | deferred | target identity ownership |
| 14 | expansion Environment reconcile | Matter Current/Next, both claims, Air Current | Air Next | joint Matter/temperature/flags/Air/pressure copy | whole parcel transfer |
| 15 | decay | material/temperature/flags Current | corresponding Next | deferred | Smoke identity/removal |
| 16 | decay flag hygiene | material/flags Next | sanitized flags Next | deferred | target identity ownership |
| 17 | decay Environment reconcile | Matter Current/Next, Air Current | Air Next | joint Matter/temperature/flags/Air copy | removal creates Vacuum |
| 18 | combustion | material/temperature/flags Current | temperature/flags, proposal, consumption material | deferred | fuel→EMPTY or Smoke request |
| 19 | Smoke claim | material Current, proposal | claim | none | target winner |
| 20 | Smoke Environment receiver claim | material/claim/Air Current | receiver claim | scratch retained | target+1 arbitration |
| 21 | receiver-gated Smoke commit | material Current, claim, temperature Next | material/temperature Next | deferred | optional EMPTY→Smoke |
| 22 | combustion flag hygiene | material/flags Next | sanitized flags Next | deferred | target identity ownership |
| 23 | Smoke Environment reconcile | Matter Current/Next, both claims, Air Current | Air Next | joint Matter/temperature/flags/Air copy | whole parcel transfer/removal Vacuum |
| 24 | pressure | material/pressure Current, class | pressure Next | copied | existing mechanical field only |
| 25 | rupture | material/pressure Current, properties | material/temperature/flags Next | deferred | Matter→EMPTY |
| 26 | rupture flag hygiene | material/flags Next | sanitized flags Next | deferred | EMPTY flags zero |
| 27 | rupture Environment reconcile | Matter Current/Next, Air Current | Air Next | joint Matter/temperature/flags/Air copy | removal creates Vacuum |
| 28 | activity propose | settled fields and tables | cell activity | none | final frontier detection |
| 29 | activity reduce | cell activity | chunk activity/changed/stable | tick end | feeds pass 0 next tick |

Production and profiled ticks share this exact encoding. The profiler's residual is an envelope residual, not an isolated copy-time measure. New Environment passes require explicit names, queries and group reconstruction.

## 2. Binding inventory

| Pass | Storage RO | Storage RW | Total storage | Uniform |
|---|---:|---:|---:|---:|
| activity wake | 3 | 2 | 5 | 1 |
| movement propose | 4 | 2 | 6 | 1 |
| movement claim | 2 | 1 | 3 | 2 |
| movement commit | 5 | 3 | **8** | 1 |
| material flag hygiene (each use) | 1 | 1 | 2 | 1 |
| Environment reconcile movement | 5 | 2 | 7 | 1 |
| thermal | 5 | 1 | 6 | 1 |
| phase transition | 4 | 3 | 7 | 1 |
| expansion claim | 3 | 1 | 4 | 2 |
| Environment receiver claim (phase/Smoke) | 4 | 1 | 5 | 2 |
| expansion spawn | 5 | 3 | **8** | 1 |
| expansion pressure | 7 | 1 | **8** | 1 |
| Environment-blocked expansion pressure | 6 | 1 | 7 | 1 |
| Environment reconcile spawn (phase/Smoke) | 6 | 2 | **8** | 1 |
| decay | 5 | 3 | **8** | 1 |
| Environment reconcile identity (decay/rupture) | 4 | 2 | 6 | 1 |
| combustion | 4 | 4 | **8** | 2 |
| Smoke claim | 3 | 1 | 4 | 2 |
| Smoke commit | 4 | 2 | 6 | 1 |
| pressure | 4 | 1 | 5 | 1 |
| rupture | 5 | 3 | **8** | 1 |
| activity propose | 7 | 1 | **8** | 1 |
| activity reduce | 1 | 3 | 4 | 1 |

The device requests wgpu default limits. The real RTX 5090/DX12 path rejected a nine-storage-buffer Sandbox edit group; the durable ceiling is eight. Combustion already moves its descriptor table to uniform storage, and activity combines phase/conductivity tables to remain within the ceiling.

A spawn Environment reconcile uses exactly eight storage bindings: pre/post
material, the original Matter claim, one new receiver claim, two Air Current
and two Air Next buffers. Chunk state belongs in the preceding receiver stage,
not this reconcile. Any extra table or event requires a split. Existing maxed
passes must not receive Environment bindings.

## 3. Authoritative writers

- `material_next`: movement commit, phase, expansion spawn, decay, combustion consumption, Smoke commit, rupture, Sandbox edit.
- `temperature_next`: movement commit, thermal, expansion spawn, decay, combustion, Smoke commit, rupture, Sandbox edit.
- `pressure_next`: expansion pressure, pressure propagation, Sandbox edit.
- `flags_next`: movement commit, expansion spawn, decay, combustion, rupture, Sandbox edit.
- Air mass/energy Next: movement, phase, decay, Smoke/combustion and rupture
  Environment reconcile passes, plus canonical staging and Sandbox edit.
- `proposal`: movement propose, then phase, then combustion; each overwrites after the previous live range.
- `claim`: movement claim, then expansion claim, then Smoke claim.
- `cell_activity`: phase transition marker and final activity proposal.
- chunk activity/stability: activity reduce.
- chunk state/wake reason: activity wake.
- edit wake: CPU/edit staging writes; the encoder clears only after the immutable wake snapshot is consumed.

TE-1 Air writers exist only in one reconcile per causal stage and canonical
staging/reset/edit. Existing Matter passes do not write Air.

## 4. Scratch lifetime

`proposal` and `claim` are each one full-cell `u32` buffer and are deliberately reused:

```text
proposal: movement 1→2; phase 7→8/11/12; combustion 18→19
claim:    movement 2→3/5; expansion 8→10/11/12/14; Smoke 19→21/23
```

One sequential f32 scale may wrap an existing scratch only after its ownership consumer/reconcile completes and before the next writer. Reuse requires structural lifetime tests. `cell_activity` remains live from the phase marker through final activity reduction and is not eligible.

Spawn receiver arbitration is new. The Matter source→target claim remains live
through spawn and blocked-pressure consumers and cannot be overwritten. TE-1
therefore adds one full-world `u32 environment_receiver_claim`. A potential
receiver scans adjacent winning Matter targets, rejects every cell that is
itself a winning Matter destination, applies whole-parcel mass/energy headroom,
and chooses the smallest target index. Matter commit is gated on the matching
receiver identity; Environment reconcile uses both claims. This is a paired
commit/block transaction, not cleanup after Matter has committed.

The implemented phase order is exact:

```text
phase proposal
→ original expansion claim
→ clear Environment receiver scratch
→ Environment receiver claim
→ receiver-gated expansion spawn
→ existing expansion pressure
→ environment-blocked expansion pressure
→ Matter flag hygiene
→ Environment reconcile
→ joint Matter/Air/pressure settle
```

Smoke repeats the same clear/receiver/commit/hygiene/reconcile pattern at
passes 19–23. The scratch clear is an encoder command immediately before each
receiver pass and never overlaps the preceding receiver consumer.

The new blocked-pressure pass uses seven storage bindings: material Current,
temperature Current, phase table, proposal, original claim, receiver claim and
read/write `pressure_next`, plus uniform params. It detects only an original
winning Matter target whose receiver claim failed and applies the same existing
blocked-expansion source. It runs after the existing eight-storage pressure
pass and before pressure copy, so it adds no ninth binding and does not double
ordinary loser/blocked outcomes. Smoke uses the same receiver transaction
without this pressure consequence.

## 5. Exact memory budget

Persistent uniform/table storage is 2,176 B. Profiling adds 960 B for 60 raw
timestamp values in both persistent resolve and readback buffers. Transient
diagnostic staging and opaque driver/query-set memory are not included,
matching the current report boundary.

| World | One f32/u32 | Four Air buffers | Receiver scratch | Existing no-profiler | New no-profiler | New with profiler |
|---|---:|---:|---:|---:|---:|---:|
| 256×256, chunk 64 | 262,144 B | 1,048,576 B | 262,144 B | 2,886,144 B | **4,196,864 B** | **4,197,824 B** |
| 2048×2048, chunk 64 | 16,777,216 B | 67,108,864 B | 16,777,216 B | 184,576,128 B | **268,462,208 B** | **268,463,168 B** |

The 2048² Environment increment is exactly 64 MiB and the receiver scratch
adds 16 MiB. The tracked correctness total is about 256.026 MiB. The live
source/target claim is still required by spawn and blocked-pressure consumers,
which is the explicit non-reuse proof. No second new full-world scratch is
authorized.

## 6. Occupancy-changing path inventory

| Path | Environment reconcile contract |
|---|---|
| normal movement into EMPTY | move destination Air parcel to vacated source; zero destination |
| density swap | both cells remain occupied; Air stays zero |
| Void exit | vacated source becomes Vacuum; no fictitious Void state |
| phase self transition | Matter→Matter, Air zero, incompatible progress cleared |
| phase expansion spawn | move target Air to claimed orthogonal EMPTY receiver; otherwise existing blocked-expansion outcome |
| Smoke spawn | same receiver rule; reject optional spawn if no receiver |
| rupture | new EMPTY starts Vacuum |
| decay | EMPTY target starts Vacuum; Matter target remains Air zero |
| fuel consumption | new EMPTY starts Vacuum |
| Sandbox Draw | external authoring removes target Environment and zeros Air |
| Sandbox Erase | external authoring seeds current world default Environment |
| preset/reset | stage both Air halves from one canonical image |
| future Vacuum edit | separate Environment operation sets EMPTY mass/energy zero |

Additional bypass writers are in scope: `GpuWorld::write_material`, benchmark calibration staging, shared scenario uploads and initial `GpuWorld::new`. Each must call the same canonical Environment staging/reconcile helper. The inventory is incomplete if any of those paths can place Matter over stale Air.

## 7. TE-1 feasibility and constraints

No hardware, memory or pass-order impossibility was found. Spawn displacement
is closed at the design level by the additional receiver-claim scratch,
receiver-gated Matter commit, eight-binding reconcile, whole-parcel headroom
and joint settle. An Environment-blocked phase expansion receives the existing
phase-pressure consequence in the pinned seven-storage pass above; Air mass
and energy are never converted to pressure in TE-1.

TE-1 is **IMPLEMENTED** and its closure preserves these constraints:

- separate reconcile and joint settle after every occupancy stage;
- eight-storage-buffer ceiling;
- canonical world/scenario/benchmark/Sandbox staging;
- exact allocation and profiler reporting;
- no `cell_activity` float scratch;
- no Air flow/thermal/pressure physics;
- no Inspector payload/cadence expansion;
- no historical G8 schema/evidence rewrite.
