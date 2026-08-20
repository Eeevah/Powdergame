# Thermal Environment production inventory

- **Audited design baseline:** `94b152e85ff6f5481a033d885d38dca0dbc1043a`
- **Production-physics source:** TE-1 `1a722d239a16bade5772688fa822465d5cef4602`; TE-2 `fb7e568e21012b6067269f4e1b82c36c865023d0`
- **Scope:** implemented TE-1/TE-2 production graph plus the proposed TE-3D writer, binding, memory and scratch delta
- **Runtime status:** TE-2 implemented and user accepted with known follow-up; TE-3D design candidate only; Air-pressure force and TE-3 runtime not started

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

Persistent uniform/table storage is 2,352 B. Profiling adds 1,088 B for 68 raw
timestamp values in both persistent resolve and readback buffers. Transient
diagnostic staging and opaque driver/query-set memory are not included,
matching the current report boundary.

| World | One f32/u32 | Four Air buffers | Receiver scratch | Existing no-profiler | New no-profiler | New with profiler |
|---|---:|---:|---:|---:|---:|---:|
| 256×256, chunk 64 | 262,144 B | 1,048,576 B | 262,144 B | 2,886,144 B | **4,197,040 B** | **4,198,128 B** |
| 2048×2048, chunk 64 | 16,777,216 B | 67,108,864 B | 16,777,216 B | 184,576,128 B | **268,462,384 B** | **268,463,472 B** |

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

## 8. TE-2 implementation-entry note — scratch lifetime resolution

The final TE-1 command graph leaves both ordinary ownership scratch buffers
available for the bounded TE-2 transport window without another full-world
allocation. The movement `proposal` and `claim` consumers end at movement
commit / Environment reconcile pass 5, and the joint Matter/Air copies finish
before any TE-2 reader begins. Phase transition is the next ordinary writer of
`proposal`; expansion claim is the next ordinary writer of `claim`.

TE-2 therefore locks this sequential reinterpretation:

```text
movement propose/claim/commit/hygiene/Environment reconcile
-> joint movement settle
-> proposal as fully-written f32 donor_outflow_scale
-> claim as fully-written f32 receiver_accept_scale
-> Air transport commit consumes both scales
-> Air mass/energy settle
-> proposal overwritten as fully-written f32 thermal_lambda
-> unified thermal commit consumes thermal_lambda
-> Matter temperature/Air energy settle
-> phase transition overwrites proposal with its normal u32 encoding
-> expansion claim overwrites claim with its normal u32 encoding
```

No TE-1 receiver-claim consumer is live in this window. The dedicated
`environment_receiver_claim` remains independent and unchanged for later
phase/Smoke transactions. Structural tests must pin the writer/consumer order,
the `u32 -> f32 -> u32` encodings, full-buffer writes before consumption, the
expected 34 production passes, and the absence of a new full-world scratch.
If the implemented order cannot retain these boundaries, TE-2 stops rather
than allocating another dense scratch buffer.

The implemented graph follows this order exactly at passes 0–33. Air scale,
Air commit, thermal scale and unified thermal commit are passes 6–9;
Environment activity is pass 32. The profiler therefore owns 68 timestamp
queries. Every new pass remains at or below eight storage bindings, proposal
and claim are fully overwritten before their f32 reads, and phase/expansion
restore their ordinary u32 meanings after the TE-2 window.

## 9. Proposed TE-3D phase-enthalpy delta

This section is an arithmetic/static design projection against the 34-pass
TE-2 graph. It is not an implemented inventory or a runtime measurement.

### 9.1 State and ownership

TE-3D proposes exactly two new dense buffers:

| Buffer | Type | Bytes | Owner and settle rule |
|---|---|---:|---|
| `phase_energy_current` | full-world `f32` | `4 * Cell count` | read half; one Water-equivalent quantity's reversible phase enthalpy |
| `phase_energy_next` | full-world `f32` | `4 * Cell count` | write half; settles jointly with the owning Matter identity |

No `phase_quantity`, fragment, mixed-cell or new generic scratch buffer is
proposed. `proposal`, `claim`, `environment_receiver_claim`, `cell_activity`
and Environment Current/Next retain their existing allocations and ownership.

Canonical external staging is Ice `-80`, Water `0`, Steam `480`, all other
Matter/EMPTY `0`. Movement copies or swaps the value with its Matter owner and
zeros a vacated/erased Cell. Phase normalization writes a valid in-family value.
Decay, combustion and rupture gain a phase-energy hygiene dispatch before their
joint settle. Reset, preset, scenario, benchmark and Draw stage both halves
byte-identically; Heat/Cool changes temperature only.

### 9.2 Projected production order

```text
0       activity_wake
1..5    movement propose/claim/commit, flag hygiene, Environment reconcile
6       phase_energy_reconcile_movement
7..10   TE-2 Air scale/commit and thermal scale/commit
11      phase_context_propose (fully writes claim as immutable u32 markers)
12      phase_thermodynamics (replaces phase_transition and fully writes proposal)
13..19  dormant expansion claim/receiver/spawn/pressure, flag hygiene,
        Environment reconcile
20..23  decay, flag hygiene, phase-energy hygiene, Environment reconcile
24..30  combustion/Smoke transaction, flag hygiene, phase-energy hygiene,
        Environment reconcile
31      pressure
32..35  rupture, flag hygiene, phase-energy hygiene, Environment reconcile
36      base activity_propose without the threshold-only phase candidate
37      phase_activity_propose
38      environment_activity_propose
39      activity_reduce
```

The historical generic expansion chain remains present for non-family
`matter_yield > 1`. All Ice/Water/Steam descriptors have `yield = 1` and zero
blocked-pressure metadata. `phase_context_propose` fully writes dead claim with
immutable Air/surface/work markers, and `phase_thermodynamics` consumes them
while fully writing proposal: phase-family Cells receive `NO_PROPOSAL`, while a
generic descriptor retains its historical proposal. Expansion claim then
overwrites claim. The Water/Steam path
therefore cannot create an expansion receiver, a second Steam identity or
blocked-expansion pressure without disabling the generic path. New Steam
pressure-volume force remains TE-5-owned.

### 9.3 Binding and table ceilings

| Proposed pass | Storage RO | Storage RW | Total storage | Other binding |
|---|---:|---:|---:|---|
| phase-energy movement reconcile | 6 | 1 | 7 | params uniform |
| phase context propose | 6 | 1 | 7 | params + phase descriptor + existing TE-2 thermal-table uniforms |
| phase thermodynamics | 4 | 4 | **8** | params + phase descriptor + existing TE-2 thermal-table uniforms |
| phase-energy identity hygiene | 5 | 1 | 6 | params uniform |
| phase activity propose | 6 | 1 | 7 | params + phase descriptor + existing TE-2 thermal-table uniforms |
| Sandbox phase edit (outside tick graph) | 3 | 2 | 5 | params uniform |

The context pass storage order is Material Current, temperature Current, phase
energy Current, Air mass/energy Current, chunk state and claim RW. It is the
only phase-context Air reader and reuses the existing 128-byte TE-2
conductivity/capacity uniform plus its exact work predicate. The
phase-thermodynamics storage order is
Material Current, temperature Current, phase energy Current, immutable
claim/context, Material Next, temperature Next, phase energy Next and proposal.
It has no Air, chunk-state or `cell_activity` binding. The existing
512-byte phase table is re-encoded as a packed 32-byte × 16 descriptor and
bound as a uniform here; capacity/conductivity comes directly from the existing
128-byte TE-2 thermal-table uniform, so the proposal adds no persistent table
allocation.
Sandbox phase editing is a separate non-timestamped dispatch because extending
the current seven-storage field-edit pass with two phase buffers would exceed
the ceiling.

### 9.4 Pass, profiler and allocation projection

The graph projects 40 timestamped passes and 80 queries. Two 640-byte profiler
buffers total 1,280 bytes, `+192` bytes from the current two 544-byte buffers.

| World | TE-2 no profiler | Two phase buffers | TE-3D no profiler | TE-3D with profiler |
|---:|---:|---:|---:|---:|
| 256² | 4,197,040 B | 524,288 B | 4,721,328 B | 4,722,608 B |
| 2048² | 268,462,384 B | 33,554,432 B | 302,016,816 B | 302,018,096 B |

The tracked boundary excludes transient diagnostic staging and opaque driver
or query-set storage, matching the TE-2 report. These figures must be replaced
by exact runtime allocation/profiler evidence only after a separately
authorized implementation exists.

### 9.5 Sleep and activity boundary

The old threshold-only phase candidate and phase activity marker are removed
together. A separate phase-activity proposal uses the same initiation,
surface/sink, nucleation and thermal-work predicates as normalization. Partial
progress with no eligible energy-flow face may sleep; movement, edit or thermal
frontiers wake the existing halo. Completed or stalled equilibrium bulk must
settle to zero activity. Sleep-on/off equivalence remains a future fixture, not
a claim established by this design inventory.
