# Thermal Environment production inventory

- **Audited design baseline:** `94b152e85ff6f5481a033d885d38dca0dbc1043a`
- **Production-physics source:** TE-1 `1a722d239a16bade5772688fa822465d5cef4602`; TE-2 `fb7e568e21012b6067269f4e1b82c36c865023d0`
- **TE-5B design baseline / authorization source:** `d7500e219af6f670be05f830b50c232d2bb53077` / `f1ca48cc01a906bfb4a997c72bc2744b81546ccd`
- **Scope:** implemented TE-1/TE-2/pressure-decoupled TE-3 graph, TE-4I production candidate, plus blocked TE-4D/TE-5/packet history
- **Runtime status:** TE-2 and TE-3 user accepted with known follow-up; TE-3 production physics `41467219819c5d0cb3eab8ae22b652449da20480`; TE-4I final runtime `8d9e8cbe3b6ac651335b5a728ef491abeae4772a` automated PASS/user review pending; Pressure redesign deferred/not started

Sections 1–7 preserve the TE-1 foundation inventory at source `1a722d...`.
Section 8 is the implemented TE-2 34-pass delta. Section 9 began as the D-018
projection and is confirmed by the D-024 40-pass runtime. Later sections
preserve rejected TE-5/packet projections as history. The 30-pass TE-1 table is
not current.

## 1. TE-1 30-pass baseline order

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

At the TE-1 source, production and profiled ticks shared this exact encoding.
The profiler's residual was an envelope residual, not an isolated copy-time
measure. Section 8 records the later TE-2 names, queries and group
reconstruction.

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

### TE-4D audited projection (not implemented)

Live combustion and activity each use eight storage bindings. The combustion
descriptor is serialized as 16 × 32 bytes (512 bytes), with a 20-byte Rust
logical descriptor and 12 WGSL padding bytes. Flags bits 2..3 and 28..31 are
unowned, but the current Oil/Wood hygiene mask `0x0000FFF3` would clear them.
Packed u6 exposure is selected by D-029. Because live combustion and base
activity are already at eight storage bindings, it conservatively adds two
logical passes while reusing proposal scratch: 42 passes/84 queries, two
672-byte profiler buffers (1,344 bytes total), zero new persistent world bytes
and zero new full-world scratch bytes. Dedicated u32 Current/Next remains an
unselected fallback (524,288 bytes at 256² or 32 MiB at 2048²). None is runtime
fact.

The projected order inserts `ignition_exposure_propose` as pass 24 after decay
and all of its hygiene/reconcile work has settled. Existing combustion/Smoke,
pressure, rupture and activity shift by one. It inserts
`ignition_activity_propose` as pass 40 after Environment activity and before
activity reduce, which becomes pass 41. The exposure pass uses five read-only
storage bindings (Material, temperature, flags, Air mass, chunk state), one
proposal read-write binding and two uniforms (params and the existing 512-byte
combustion table): six storage total. The activity pass uses Material,
temperature, flags, Air mass and chunk state read-only plus `cell_activity`
read-write, again six storage, and the same two uniforms.

Proposal lifetime is source-feasible: all earlier phase/expansion consumers
finish; the exposure pass fully overwrites every proposal word as packed
exposure/ignite/Air context; combustion consumes it and then fully overwrites
proposal with the existing Smoke request; Smoke claim/receiver/commit consume
that request before later activity. No stale interpretation crosses a lifetime.

Fresh review means this arithmetic projection is not implementation-ready:
the sole-Air-face/Smoke counterexample may require target protection,
post-commit recheck or changed snapshot semantics. Any choice requires a new
pass/encoding/live-range audit. V2 is DESIGN BLOCKED.

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

## 9. D-018 accepted TE-3D phase-enthalpy projection

This section records the D-018-accepted arithmetic/static design projection
against the 34-pass TE-2 graph, including its locked amendments. It is not an
implemented inventory or a runtime measurement.

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

Initiated boiling remains Water when buried. It may retain, increase or reverse
phase energy, and reaches the derived ready state `Water, E=480` without a new
buffer or flag. Any further added enthalpy is represented as Water sensible
superheat. Cooling consumes that superheat before reducing phase energy.
Identity completion to Steam requires either a gas-facing context or a future
separately accepted TE-5 transaction. The TE-5 route is a contract placeholder
only and adds no pass, binding or allocation to this projection.

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
A generic non-family `matter_yield > 1` descriptor may target only non-phase
Matter in this architecture. Targeting Ice, Water or Steam requires a later
owned destination-phase-energy writer design and separate acceptance.

### 9.3 Binding and table ceilings

| Proposed pass | Storage RO | Storage RW | Total storage | Other binding |
|---|---:|---:|---:|---|
| phase-energy movement reconcile | 5 | 1 | 6 | params uniform |
| phase context propose | 7 | 1 | **8** | params + phase descriptor + existing TE-2 thermal-table uniforms |
| phase thermodynamics | 4 | 4 | **8** | params + phase descriptor + existing TE-2 thermal-table uniforms |
| phase-energy identity hygiene | 4 | 1 | 5 | params uniform |
| phase activity propose | 7 | 1 | **8** | params + phase descriptor + existing TE-2 thermal-table uniforms |
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

The context marker distinguishes gas-facing completion, a real surface sink,
canonical free-air work and an active radius-2 partial veto. A real surface
sink requires identity/temperature eligibility, positive shared TE-2 face
conductance and the exact shared TE-2 predicate that would remove energy from
Steam on that face. A zero-conductivity Boundary is not a sink. Free-air seed
selection and active-partial veto both use `NUCLEATION_RADIUS=2` and the strict
minimum of `(coordinate_hash32, y, x)` in a 5×5 Chebyshev neighbourhood.

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

D-024 implementation at `41467219819c5d0cb3eab8ae22b652449da20480`
confirms these allocation totals, 40 passes, 80 queries and the corrected live
binding rows above.

The locked sink, ready-Water, radius-2 and metastability amendments alter
predicates and state interpretation only. They add no pass, persistent buffer,
storage binding or profiler query beyond the projection above.

### 9.5 Sleep and activity boundary

The old threshold-only phase candidate and phase activity marker are removed
together. A separate phase-activity proposal uses the same initiation, real
surface-sink, radius-2 seed/veto and exact thermal-work predicates as
normalization. Partial progress with no eligible energy-flow face may sleep;
movement, edit or thermal frontiers wake the existing halo. Canonical or
partially condensed supercooled Steam with no eligible sink/work may remain
metastable indefinitely and sleep without losing its identity or phase energy.
Completed or stalled equilibrium bulk must settle to zero activity.
Sleep-on/off equivalence remains a future fixture, not a claim established by
this design inventory.

## 10. Proposed but design-blocked TE-5B phase-volume bridge delta

This section records the D-019-authorized candidate from
[`ADR-0007`](decisions/ADR-0007-phase-volume-pressure-bridge.md) and
[`PHASE_VOLUME_PRESSURE_BRIDGE_SPEC`](../specs/PHASE_VOLUME_PRESSURE_BRIDGE_SPEC.md).
It is a static packing/lifetime feasibility projection against Section 9, not
implementation or device evidence. Independent review found the candidate
semantically unable to consume finite headspace; the zero-allocation result
below is therefore not an implementation recommendation.

### 10.1 Reused allocations and encoding

No new buffer is proposed. After `phase_context_propose`'s context lifetime,
`phase_thermodynamics` fully overwrites existing proposal with a two-bit mode
and a 30-bit `index + 1` payload. `expansion_claim` fully overwrites existing
claim with the winner's mode and `source + 1`. The strict existing
`cell_count < 1 << 30` bound prevents payload/mode overlap.

```text
00 = none
01 = Matter expansion
10 = volume relief
11 = invalid/reserved
```

The mode-aware expansion lifetime ends before combustion fully overwrites
proposal and Smoke arbitration fully overwrites claim. No mode bits enter a
later scratch interpretation.

The one 512-byte projected phase descriptor retains both uniform and storage
views. The candidate assigns its existing family above-consequence pressure
slot `100.0` for every source family identity that can normalize through
vaporization in one invocation. This is a design requirement, not an established
fact: the one-shot proof used identity strings and did not generate the
descriptor or exercise extreme-Ice normalization. Future semantic/structural
evidence would have to close that gap without a Water-name branch, new table or
new binding.

### 10.2 Projected pass order

The Section 9 40-pass graph is unchanged. The detailed expansion window is:

```text
11  phase_context_propose: claim full write
12  phase_thermodynamics: attempt classification + accepted proposal/provisional identity full write
13  expansion_claim: shared-mode claim full write
14  expansion Environment receiver claim: Matter mode only
15  expansion spawn commit: Matter mode only
16  expansion_pressure: generic direct failures + relief failures
17  Environment-blocked expansion pressure: Matter mode only
18  identity/phase-energy hygiene
19  Environment reconcile
    settle Material/temperature/phase energy/pressure/Environment
31  existing gauge-pressure propagation
    settle pressure before rupture
```

A relief winner leaves the EMPTY target and its Air mass/energy byte-identical.
A blocked/losing relief request adds the descriptor's `100.0` once at its
completion source in pass 16. Pass 17 rejects relief before receiver lookup,
so it cannot duplicate that consequence.

### 10.3 Binding ceiling

| Projected mode-aware pass | Storage RO | Storage RW | Total | Added storage |
|---|---:|---:|---:|---:|
| phase thermodynamics | 4 | 4 | **8** | 0 |
| expansion claim | 3 | 1 | 4 | 0 |
| Environment receiver claim | 4 | 1 | 5 | 0 |
| expansion spawn commit | 5 | 3 | **8** | 0 |
| expansion pressure | 7 | 1 | **8** | 0 |
| Environment-blocked pressure | 6 | 1 | 7 | 0 |

Mode decoding and filtering use already-bound proposal/claim words. The claim
writer rejects invalid proposal candidates and fully writes one constructed
winner or zero. Receiver/spawn validate fields in claim but do not bind proposal
again; the source/target relationship is a trust boundary at the claim writer,
not independently revalidated by every consumer. Expansion pressure reuses its
already-bound descriptor. Any future implementation that needs another storage
binding, pass or scratch is a design blocker rather than implicit authority to
enlarge this table.

### 10.4 Cost and evidence boundary

| Resource | Accepted TE-3D | TE-5B delta | Combined projection |
|---|---:|---:|---:|
| timestamped passes | 40 | 0 | 40 |
| timestamp queries | 80 | 0 | 80 |
| profiler storage | 1,280 B | 0 | 1,280 B |
| persistent/full-world storage | Section 9 totals | 0 | unchanged |
| 256² tracked with profiler | 4,722,608 B | 0 | 4,722,608 B |
| 2048² tracked with profiler | 302,018,096 B | 0 | 302,018,096 B |

The bridge reuses the current gauge-pressure propagation and rupture grammar
without changing their meaning. Air pressure, structure differential,
movement, sleep, exact device bindings, allocation and performance remain
future evidence. TE-3 and TE-5B must activate atomically on one later
authorized source; historical G5 evidence is not rebound.

### 10.5 Semantic feasibility blocker

The reused storage graph can encode same-tick exclusivity, but it has no owner
for consumed cross-tick capacity. In a sealed one-Cell-wide column with one
EMPTY Cell above stagger-heated Water, only the top Water is ready at `t0` and
each lower Cell reaches the endpoint after the vacancy arrives above. A winning
1:1 Steam completion later moves into the EMPTY Cell and vacates its source.
That vacancy then becomes the next newly-ready Water's relief target and walks
down the column without a lower simultaneous attempt seeing a Steam swap. The
bridge can therefore settle every completion with zero pressure and never
reach the required finite-headspace confinement event.

Fixing this needs capacity/reservation state, a target/Environment mutation,
additional occupied quantity, or another pressure law. Each changes the locked
candidate or a rejected option. The projected `40` passes, `80` queries and zero
TE-5B memory delta describe only the infeasible candidate; they do not prove a
replacement can preserve those counts. ADR-0007 remains Proposed / DESIGN
BLOCKED and all runtime remains not started.

## 11. Proposed but design-blocked TE-5C capacity-share delta

D-020 rejects Section 10's token and evaluates
[`ADR-0008`](decisions/ADR-0008-local-vapor-capacity-pressure.md). Static source
order permits one new `vapor_capacity_sum` dispatch after the settled Smoke
transaction and before pressure. It fully overwrites proposal as `f32 D_e`;
pressure consumes it, phase activity consumes the same still-live values as an
eighth storage binding, and next-tick movement overwrites proposal as u32.

| Projected pass | Storage RO | Storage RW | Total |
|---|---:|---:|---:|
| capacity sum | 3 | 1 | 4 |
| pressure with target | 6 | 1 | 7 |
| phase activity with capacity | 7 | 1 | 8 |

The projection is 41 passes, 82 queries and two 656-byte profiler buffers
(1,312 B). With no new buffer or table, tracked totals would be 4,722,640 B at
256² and 302,018,128 B at 2048². These are arithmetic, not runtime evidence.

The proposed scratch window is packable in isolation, but fresh review found
the overall activity path is not: the eight-binding phase-activity projection
has no pressure input, base activity ignores the new medium-to-EMPTY vent work,
and rupture changes the Material snapshot after `D_e` was computed. The law is
also semantically blocked. In a two-Steam,
two-EMPTY adjacency graph with a complete assignment, independent
proportional shares give one Steam gross `1.5` then cap/discard `0.5`, while
the other stays at `0.5` and receives target `100`. The one-shot proof failed
that predeclared control. Other open High findings are internal-EMPTY finite-
capacity/zero-vent conflation, irreversible phase-pressure provenance,
unreachable downward capacity and reference-receipt overclaim. Therefore this
section is not an implementation recommendation and no persistent allocation
claim is established for the next required architecture.

## 12. Proposed but design-blocked TE-5D persistent-state delta

D-021 and [`ADR-0009`](decisions/ADR-0009-persistent-vapor-extent.md) permit
one 8-byte-per-Cell Current/Next pair: reciprocal `u32` link plus dedicated
`f32` phase pressure. Both halves add 16 bytes per Cell, exactly 1,048,576 B
at 256² and 67,108,864 B at 2048². Adding that to the accepted TE-3 projection
gives 369,125,680 B at 2048² before profiler or any other allocation change.

The fixed design projects 62 passes and 124 timestamps: a phase-volume
movement context, eighteen bounded reassignment dispatches, Environment
receiver acceptance, reservation commit and phase-pressure update beyond the
40-pass TE-3 graph. Movement propose, Air-flow scale, thermal-stability scale
and Environment activity can read the link within eight storage bindings.
The existing movement Environment commit is already full, so a fully written
context must replace rather than add a raw link binding. Rupture can read the
new pressure only if its two current material tables are combined into one
descriptor binding. All are static projections.

The projection is not an implementation recommendation. A canonical
persistent matching may contain an augmenting path with eight source vertices;
the frozen depth-six atomic algorithm cannot reach its free target. Exact
world-scale convergence would need wider matching scope and either an
unbounded/far larger fixed dispatch budget or a source-proven global search
lifetime. Proposal and claim can represent frontier/predecessor roles after
Smoke, but no fixed pass bound or exact convergence protocol has been proven.
No additional scratch allocation is authorized.

## 13. Proposed but evidence-blocked TE-5X comparison projections

D-022 and [`ADR-0010`](decisions/ADR-0010-pressure-volume-model-selection.md)
compare three non-runtime layouts. Candidate A retains the TE-5D 16-byte-per-
Cell Current/Next state (64 MiB at 2048²), reuses proposal/claim for exact
frontier/predecessor work and has the finite but prohibitive pass bound
`47 + N*(2N+3)`. Candidate B uses one dedicated pressure pair plus two
temporary `vec4<u32>` grouping buffers: 40 bytes per Cell, 160 MiB at 2048²,
with a conservative 188-pass/376-query CCL/radix/reduction projection.
Candidate C uses Vapor-volume and pressure Current/Next pairs, 16 bytes per
Cell (64 MiB), with a projected 46 passes/92 queries but an unresolved
condensation ownership failure. All representative passes are at or below
eight storage bindings; base activity remains separate because it is already
at eight.

These are arithmetic and architecture comparisons only. The one-shot process
failed during NetworkX oracle bootstrap before any candidate, fixture,
generated state or grid ran. No memory was allocated in production, no pass
exists in source and no candidate is recommended. TE-5X is DESIGN BLOCKED by
incomplete comparison evidence.

## 14. Proposed TE-3Q / TE-5Q conservative packet projection

D-023 and [`ADR-0011`](decisions/ADR-0011-conservative-phase-packets.md)
supersede only the whole-Cell quantity constraint. The proposed state adds
`phase_units` and `phase_pressure` Current/Next pairs alongside the accepted
`phase_energy` pair. Each pair is 8 bytes per Cell: 524,288 B at 256² and
33,554,432 B at 2048². The total increment over TE-2 is therefore 1,572,864 B
and 100,663,296 B / 96 MiB respectively.

The static 50-pass/100-query projection adds ten named passes to TE-3D's 40:
unit movement reconcile; a second normalization/identity pass; quantity split
commit; five merge proposal/claim/commit/reconcile passes; phase pressure; and
phase-pressure activity. Proposal/claim is fully overwritten between
expansion, merge and Smoke. The base activity pass remains unchanged because
it already binds eight storage buffers. Representative maxima are eight
storage bindings for enthalpy normalization, identity/proposal, split commit,
merge commits/reconcile and packed-table rupture. No extra full-world scratch
is proposed.

Tracked arithmetic including two 50-pass profiler buffers (1,600 B) is
5,771,504 B at 256² and 369,127,280 B at 2048². These figures are static; no
buffer or pass exists in runtime. The one-shot standard-library reference
reported mathematical PASS, not WGSL/device/memory/performance feasibility.

Fresh review recorded Critical `0` / High `8` / Medium `1`. The 50-pass table
is not source-closed: phase pressure needs a fourth RO `chunk_state` binding,
and movement/relief/merge/writer transactions remain incomplete even though
the eight-storage ceiling is not itself exceeded. No runtime state or pass was
allocated. Final projection disposition is **DESIGN BLOCKED**.

## 15. TE-4D v3 source-feasibility projection

D-030 adds no production code. The projected `ignition_exposure_propose`
occupies future pass 24 and reads the settled post-decay
`COMBUSTION_STAGE_SNAPSHOT`; combustion shifts to 25 and fully overwrites
proposal with Smoke requests. Smoke reconciliation remains before the existing
Matter/temperature/flags/phase-energy/Air settle copies. Pressure/rupture then
settle before base, phase and Environment activity. A projected
`ignition_activity_propose` at 40 therefore reads authoritative post-Smoke
Matter/Air and augments `cell_activity` before reduction at 41.

Both projected TE-4 passes need six storage bindings (uniform tables excluded)
and remain under the eight-storage ceiling. Total projection is 42 passes, 84
timestamp queries and 1,344 profiler query bytes. Proposal scratch is fully
written for exposure context, consumed by combustion, then fully overwritten
for Smoke. New persistent and full-world scratch allocation are both zero.
This is static source feasibility only; no binding, Naga, GPU, wake, profiler
or performance evidence was run.

## 16. TE-4I implemented production inventory

D-032 final source `8d9e8cbe3b6ac651335b5a728ef491abeae4772a`
turns the Section 15 projection into production fact. The exact pass order is
the 42-entry list in
[`TE4_IGNITION_KINETICS`](../planning/TE4_IGNITION_KINETICS.md): exposure is
pass 24, combustion/Smoke is 25..31, pressure/rupture is 32..36, the existing
three activity proposals are 37..39, ignition activity is 40, and reduction is
41. There are 84 timestamp queries and two 672-byte resolve buffers, or 1,344
profiler bytes.

`ignition_exposure_propose` and `ignition_activity_propose` each use five
read-only storage bindings plus one read-write binding, with params and the
existing combustion table as uniforms: six storage bindings each. Combustion
remains at eight. The Section 2 binding table continues to cover every reused
pass; no pass exceeds eight storage bindings.

Proposal lifetime is now verified: pass 24 fully writes u8-bounded
exposure/ignite/Air context into every word; pass 25 consumes it and fully
overwrites every word with a Smoke request or zero; passes 26..28 consume the
Smoke lifetime. Smoke hygiene/reconcile and Current copies settle Matter,
temperature, flags, phase energy and Air before ignition activity reads the
next-stage topology. The combustion upload is still 16 entries x 32 bytes =
512 bytes. New persistent state is zero; new full-world scratch is zero.

Authoritative flag writers and hygiene paths now preserve packed u6 exposure
only with Oil/Wood identity and clear it on movement source replacement,
density transfer source, Void, phase/decay/rupture/consumption replacement,
Draw, Erase, presets, scenarios, reset, and generic identity replacement.
Naga, binding, writer, allocation, profiler, runtime, and final-source FULL
evidence are recorded in
[`THERMAL_ENVIRONMENT_TE_4_IGNITION_KINETICS_2026-08-23`](../evidence/THERMAL_ENVIRONMENT_TE_4_IGNITION_KINETICS_2026-08-23.md).
