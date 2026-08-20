# Thermal Environment production inventory

- **Audited source:** `f5c7ac8e76867f769cdf19d7f420432d8fef4509`
- **Scope:** existing production pass graph, writers, bindings, memory, scratch and occupancy paths
- **Runtime changes:** none

## 1. Current pass order

| # | Pass | Current/read inputs | Next/scratch outputs | Settle boundary | Ownership/activity effect |
|---:|---|---|---|---|---|
| 0 | activity wake | chunk activity/stable/edit-wake | chunk state/wake reason | edit-wake cleared after all chunks observe it | next-tick execution mask |
| 1 | movement propose | material, class/density, chunk state | proposal, marker | none | candidate only |
| 2 | movement claim | proposal, arbitration, chunk state | claim | none | winner only |
| 3 | movement commit | material/temperature/flags Current, claim | corresponding Next | all three copied | move, density swap, Void exit |
| 4 | thermal | material/temperature Current, properties | temperature Next | temperature copied | field only |
| 5 | phase transition | material/temperature Current, phase table | material Next, proposal, activity marker | deferred | self identity and expansion request |
| 6 | expansion claim | material Current, proposal | claim | none | target winner |
| 7 | expansion spawn | material/temperature Current, claim | material/temperature/flags Next | deferred | EMPTY→Matter |
| 8 | expansion pressure | phase/proposal/claim, pressure Current | pressure Next | phase fields and pressure copied | blocked expansion source |
| 9 | decay | material/temperature/flags Current | corresponding Next | copied | current Smoke→EMPTY path |
| 10 | combustion | material/temperature/flags Current | temperature/flags, proposal, consumption material | deferred | fuel→EMPTY or Smoke request |
| 11 | Smoke claim | material Current, proposal | claim | none | target winner |
| 12 | Smoke commit | material Current, claim, temperature Next | material/temperature Next | material/temperature/flags copied | EMPTY→Smoke |
| 13 | pressure | material/pressure Current, class | pressure Next | copied | field only |
| 14 | rupture | material/pressure Current, properties | material/temperature/flags Next | copied | Matter→EMPTY |
| 15 | activity propose | settled fields and tables | cell activity | none | final frontier detection |
| 16 | activity reduce | cell activity | chunk activity/changed/stable | tick end | feeds pass 0 next tick |

Production and profiled ticks share this exact encoding. The profiler's residual is an envelope residual, not an isolated copy-time measure. New Environment passes require explicit names, queries and group reconstruction.

## 2. Binding inventory

| Pass | Storage RO | Storage RW | Total storage | Uniform |
|---|---:|---:|---:|---:|
| activity wake | 3 | 2 | 5 | 1 |
| movement propose | 4 | 2 | 6 | 1 |
| movement claim | 2 | 1 | 3 | 2 |
| movement commit | 5 | 3 | **8** | 1 |
| thermal | 5 | 1 | 6 | 1 |
| phase transition | 4 | 3 | 7 | 1 |
| expansion claim | 3 | 1 | 4 | 2 |
| expansion spawn | 4 | 3 | 7 | 1 |
| expansion pressure | 7 | 1 | **8** | 1 |
| decay | 5 | 3 | **8** | 1 |
| combustion | 4 | 4 | **8** | 2 |
| Smoke claim | 3 | 1 | 4 | 2 |
| Smoke commit | 3 | 2 | 5 | 1 |
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
- `proposal`: movement propose, then phase, then combustion; each overwrites after the previous live range.
- `claim`: movement claim, then expansion claim, then Smoke claim.
- `cell_activity`: phase transition marker and final activity proposal.
- chunk activity/stability: activity reduce.
- chunk state/wake reason: activity wake.
- edit wake: CPU/edit staging writes; the encoder clears only after the immutable wake snapshot is consumed.

TE-1 adds Air writers only through one reconcile commit per causal stage and canonical staging/reset. It does not add an Air writer to the existing Matter passes.

## 4. Scratch lifetime

`proposal` and `claim` are each one full-cell `u32` buffer and are deliberately reused:

```text
proposal: movement 1→2; phase 5→6/8; combustion 10→11
claim:    movement 2→3; expansion 6→7/8; Smoke 11→12
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

Phase order is exact:

```text
phase proposal
→ original expansion claim
→ Environment receiver claim
→ receiver-gated expansion spawn
→ existing expansion pressure
→ environment-blocked expansion pressure
→ Environment reconcile
→ Matter flag hygiene
→ joint Matter/Air/pressure settle
```

The new blocked-pressure pass uses seven storage bindings: material Current,
temperature Current, phase table, proposal, original claim, receiver claim and
read/write `pressure_next`, plus uniform params. It detects only an original
winning Matter target whose receiver claim failed and applies the same existing
blocked-expansion source. It runs after the existing eight-storage pressure
pass and before pressure copy, so it adds no ninth binding and does not double
ordinary loser/blocked outcomes. Smoke uses the same receiver transaction
without this pressure consequence.

## 5. Exact memory budget

Persistent uniform/table storage is 2,176 B. Profiling adds 544 B. Transient diagnostic staging and opaque driver/query-set memory are not included, matching the current report boundary.

| World | One f32/u32 | Four Air buffers | Receiver scratch | Existing no-profiler | New no-profiler | New with profiler |
|---|---:|---:|---:|---:|---:|---:|
| 256×256, chunk 64 | 262,144 B | 1,048,576 B | 262,144 B | 2,886,144 B | **4,196,864 B** | 4,197,408 B |
| 2048×2048, chunk 64 | 16,777,216 B | 67,108,864 B | 16,777,216 B | 184,576,128 B | **268,462,208 B** | 268,462,752 B |

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

TE-1 remains **NOT STARTED** and must preserve these constraints:

- separate reconcile and joint settle after every occupancy stage;
- eight-storage-buffer ceiling;
- canonical world/scenario/benchmark/Sandbox staging;
- exact allocation and profiler reporting;
- no `cell_activity` float scratch;
- no Air flow/thermal/pressure physics;
- no Inspector payload/cadence expansion;
- no historical G8 schema/evidence rewrite.
