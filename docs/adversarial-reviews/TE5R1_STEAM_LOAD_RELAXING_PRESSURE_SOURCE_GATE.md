# Independent TE-5R1 Steam-Load Relaxing Pressure Source Gate

- **Date:** 2026-08-23
- **Reviewer role:** fresh-context independent adversarial reviewer; not a
  primary-author participant
- **Reviewed branch:** `feature/m0-g9-first-playable`
- **Production source baseline / HEAD:**
  `12b49dc07c8d875de55a048013a01090d38345a9`
- **Current unresolved:** Critical **0**, High **0**, Medium **0**, Low **0**
- **Runtime/reference execution by this reviewer:** **0**
- **Verdict:** **TE-5R1 SOURCE-REALIZABILITY PASS — IMPLEMENTATION AUTHORIZED
  UNDER D-037**

## 1. Scope and provenance

This review attacks the D-037-authorized post-phase Steam-load candidate before
any TE-5 runtime edit. It read D-037, [ADR-0014](../architecture/decisions/ADR-0014-post-phase-steam-load-relaxing-pressure.md),
the [TE-5R1 source gate](../planning/TE5R1_STEAM_LOAD_RELAXING_PRESSURE.md),
blocked [ADR-0013](../architecture/decisions/ADR-0013-local-relaxing-phase-load-pressure.md),
and the preserved [R0 independent review](TE5R_PRESSURE_VACUUM_REENTRY_DESIGN.md).

The production trace covered `engine/core` and `engine/gpu` phase,
expansion-pressure, pressure, Air scale/commit, movement, rupture,
Matter/phase/Environment hygiene, activity, sleep, pass/copy, profiler,
allocation and world authoring paths, plus scenario and Sandbox staging under
`apps`. HEAD exactly matches the source baseline. Dirty inputs are limited to
D-037 and the two new design documents; no production, Cargo or runtime file
differs from HEAD.

Reviewed input identities:

| Input | SHA-256 |
|---|---|
| ADR-0014 | `355499887b594b21bd4a2e3e05565a4031fc0665c30ef123a1b9404404dcc19c` |
| TE-5R1 source gate | `5ec305eae5297c9bcf91e2aeedcc86bb246f22a04b972532a369bf971b8faa01` |
| blocked ADR-0013 | `27678ec830c020c4af4d7dc855ddbc8b809c535633aee27210d6cbdecc3b37be` |
| R0 independent review | `dc5bacd26ad5631211974be4e5ef65cbcd4d79bb2ee2a6e1c2a591124f6416ae` |

This file is the reviewer's only write and is excluded from that input list.

## 2. Finding counts

| Severity | Open at verdict | Resolved by the reviewed R1 contract |
|---|---:|---:|
| Critical | **0** | 0 |
| High | **0** | **5** |
| Medium | **0** | **3** |
| Low | **0** | 0 |

D-037 blocks implementation only for an unresolved Critical/High, a ninth
storage binding, required new persistent/full-world state, or another named
invalidation. None was found in this source-bound projection.

## 3. Exact source-table row disposition

Every row in the TE-5R1 value/lifetime table was checked against production
source rather than accepted from binding arithmetic alone.

| Value row | Independent source disposition | Smallest attempted source counterexample |
|---|---|---|
| settled Material | **REALIZABLE.** Phase writes Next at pass 12; expansion hygiene/reconcile settles it at pass 19. Decay, combustion/Smoke and rupture each have their own later settle before pressure/activity consumers. | A pass-12 Water-to-Steam identity is not visible through `material_current` until the expansion copy, but pressure is pass 32 and therefore reads the settled identity. |
| settled phase energy | **REALIZABLE.** Phase writes Next; the pass-19 copy settles it; later identity hygiene and copies re-establish the Material-owned value before pressure. Rupture hygiene settles it again before pressure activity. | A rupture-created EMPTY cannot leave Steam energy visible to pass 41 because pass 35 clears the replacement and the post-rupture copy precedes activity. |
| generic pressure impulse | **REALIZABLE / EXACTLY ONCE.** `expansion_pressure` fully writes `pressure_next`; the Environment-blocked pass adds only for a winning Matter receiver with no Environment receiver. The expansion copy settles that result before local pressure. | A winning Matter claim with failed Environment receiver is treated as success by the first writer and receives one addition from the second; a lost/blocked claim is handled by the first and rejected by the second. No path adds twice. |
| dynamic pressure Current | **REALIZABLE.** Both halves exist already. Expansion settles Current before pass 32; pass 32 fully writes every Cell, including sleeping and blocked branches, and its copy settles before rupture/activity. | Author pressure into a sleeping Static/Powder Cell: the pressure pass sleep branch still writes zero, so rupture cannot observe stale blocked-node pressure. |
| Steam target | **REALIZABLE.** Material plus `phase_energy_current` are both available within the six-storage pressure projection and five-storage pressure-activity projection. No phase-context or pre-transition value is required. | A partially loaded Water next to a changing gas face always targets zero under D-037; the R0 unavailable-snapshot witness no longer changes the answer. |
| derived Air background | **REALIZABLE.** Material, Air energy and dynamic pressure coexist at all three consumers. Exact Vacuum's canonical energy is zero; Matter's paired Environment state is zero. | Zero-mass Vacuum with dynamic pressure contributes the dynamic term but cannot donate Air because donor capacity is still mass-bounded. No Air mass binding is needed by rupture. |
| Air donor scale | **REALIZABLE.** Pass 7 can fully write proposal scratch as `f32` for every Cell, including zero for non-donors, before pass 8 consumes it and pass 9 overwrites it. | A non-EMPTY Cell with stale proposal bits receives explicit zero rather than leaking an old movement destination into Air commit. |
| Air total pressure | **REALIZABLE.** The same pass can fully write claim scratch as `f32`; pass 8 consumes it, passes 9-10 do not read claim, and phase context overwrites it only at pass 11. | No stale receiver-scale consumer remains after commit is changed; expansion does not consume claim until phase context and phase have established the later scratch lifetime. |
| Air receiver scale | **REALIZABLE WITHOUT STORAGE.** Commit can gather each receiver's current mass/energy and the receiver's four donor `P_total` values, recompute raw incoming mass/energy, and derive the accepted mass/energy scale. Radius-two reads add no binding and read only immutable Current/scratch. | For four simultaneous hot donors into a nearly-full receiver, actual incoming on every face uses the same `min(donor_scale, receiver_scale)` and is no greater than the recomputed receiver scale times the unscaled aggregate; both mass and energy caps hold. |
| Air mass/energy Next | **REALIZABLE / CONSERVATIVE.** Internal-face raw demand, donor scale, receiver scale and donor specific energy are identical in the two self-writer evaluations, so one Cell's outgoing pair is the neighbour's incoming pair. Current commit already self-writes all Cells and immediately settles both buffers. | Two EMPTY Cells with unequal `P_total`, mass and specific energy use one directed raw flow and the same two scales on both invocations; no one-sided scale or ninth input appears. Sealed missing faces transfer zero. |
| total pressure at rupture face | **REALIZABLE AT EIGHT STORAGE.** After pass-32 settle, Static/Powder pressure is guaranteed zero. Rupture can therefore replace movement class with Air energy and compute `pressure + EMPTY-only air_energy/293.15` from Material, pressure and Air energy, then compare opposing faces. | A uniform nonzero field on both sides of Wood gives zero opposing-face stress; a blocked Cell carrying authored stale pressure is zeroed even if its chunk sleeps. |
| base cell activity | **REALIZABLE AT SEVEN STORAGE.** Removing pressure input and `pressure_frontier` leaves Material, temperature, flags, class, density, activity tables and cell activity. The existing final assignment remains the full-write clear for all prior bits. | The R0 two-node nonuniform equilibrium cannot retain the old pressure bit because pass 37 assigns a newly built non-pressure mask rather than OR-ing it. |
| Environment activity | **REALIZABLE AT SIX STORAGE.** Adding dynamic pressure to the existing Material/temperature/Air mass/Air energy/cell-activity set permits the Air-work predicate to compare `P_total`; the pass remains full-world and ORs only its owned bits. | Two EMPTY Air Cells with equal Air background but unequal dynamic pressure cannot sleep as "no Air work"; the new pressure input exposes the same total-pressure drop used by scale/commit. |
| pressure activity | **REALIZABLE AT FIVE STORAGE / SOLE SETTER.** Material, phase energy, pressure, class and cell activity are sufficient to duplicate node, target, neighbour, sanitization, clamp and epsilon rules. No chunk binding means the pass cannot skip a sleeping Cell. | At the former `(52.381,47.619)` nonuniform equilibrium both predicted updates are within epsilon, so no pressure bit is set; perturbing either value beyond the exact-update epsilon sets it and wakes the chunk on the following tick. |
| chunk activity/state | **UNCHANGED AND SUFFICIENT.** The new proposer precedes the existing reduction; reduction writes chunk activity/state diagnostics, and next Tick's wake pass consumes them with the existing neighbour halo. | A sleeping chunk with pending target relaxation skips one pressure update, but the full-world pass detects the pending update in that Tick and makes it runnable on the next; it cannot sleep forever with unreported pressure work. |

## 4. Resolved inherited findings

### TE5R1-R-001 — R0 pre-transition phase-context dependency removed

- **Severity:** High in R0
- **Status:** **RESOLVED BY D-037 / NOT OPEN**
- **Smallest source counterexample retested:** partially vaporized Water whose
  only gas-facing neighbour changes identity during the same phase pass.
- **Resolution:** R1 makes every Water target zero and derives a nonzero target
  only from settled Steam plus settled phase energy. Those inputs exist after
  the pass-19 and later identity copies, so neither reused claim scratch nor a
  preserved pre-transition snapshot is required.

### TE5R1-R-002 — R0 fresh-impulse transaction contradiction removed

- **Severity:** High in R0
- **Status:** **RESOLVED BY D-037 / NOT OPEN**
- **Smallest source counterexample retested:** isolated target-zero node receives
  a generic impulse of `100` before the local pass.
- **Resolution:** R1 normatively reads the already-settled `q=100` and applies
  the ordinary same-Tick relaxation, yielding `98`. It no longer claims a
  separate additive event after the settle and needs no event bit or scratch.

### TE5R1-R-003 — R0 overlapping pressure-activity ownership removed

- **Severity:** High in R0
- **Status:** **RESOLVED BY D-037 / NOT OPEN**
- **Smallest source counterexample retested:** sealed Steam/Water two-node exact
  equilibrium with a nonzero stored-pressure gradient.
- **Resolution:** Base activity removes its pressure input and producer but
  keeps the full assignment that clears the old bit. No other current shader
  writes `ACTIVITY_PRESSURE`; the later exact-update proposer is therefore the
  sole setter and can leave the equilibrium asleep.

### TE5R1-R-004 — Air scratch reinterpretation preserves donor and receiver bounds

- **Severity:** High if unresolved
- **Status:** **RESOLVED IN THE SOURCE PROJECTION / NOT OPEN**
- **Smallest source counterexample retested:** four donors simultaneously target
  a receiver with little remaining mass and energy capacity.
- **Resolution:** Proposal stores donor scale, claim stores immutable total
  pressure, and commit recomputes each needed receiver scale from current Air
  and the complete four-face raw-in aggregate. Actual transfer uses the same
  donor/receiver minimum as the accepted transaction. This is conservative,
  cap-preserving, race-free and remains at eight storage buffers.

### TE5R1-R-005 — Rupture total pressure does not require a ninth input

- **Severity:** High if unresolved
- **Status:** **RESOLVED IN THE SOURCE PROJECTION / NOT OPEN**
- **Smallest source counterexample retested:** a sleeping blocked Cell contains
  authored nonzero pressure next to a breakable structure.
- **Resolution:** The immediately preceding pressure pass writes blocked nodes
  to zero even on its sleeping branch. Rupture can drop movement class, bind
  Air energy, use Material for EMPTY-only background, and remain at exactly
  eight storage buffers.

## 5. Required attack coverage and non-blocking history

| Required attack | Review result |
|---|---|
| hidden phase/pre-transition dependency | None remains; **R-001**. |
| impulse double-write or wrong settle order | Existing mutually exclusive writers and the expansion copy implement **R-002**. |
| activity clear/set sole ownership | Base assignment clears; only the new pass sets pressure; **R-003**. |
| Air donor/receiver conservation | Source-realizable and cap-preserving; **R-004**. |
| ninth binding or stale scratch consumer | None. Peak storage remains eight; proposal ends at pass 9 and claim at pass 11. |
| `P_total` exactly once | Air scale writes the one combined scratch; commit consumes that value without re-adding background. Rupture and Environment activity each form dynamic plus EMPTY-Air background once locally. |
| rupture eight-storage feasibility | Closed by **R-005**. Opposing-face differential rejects uniform pressure. |
| sleeping pending pressure/Air work | Both Environment and pressure activity are full-world. Pressure activity intentionally wakes for the following Tick. |
| movement pressure dependency | Production movement currently has no pressure binding; R1 adds none. The R0 self-propelling-pressure risk is removed, while ordinary Gas movement and spatial pressure trails remain explicit. |
| reset/editor half-state | Reset, `write_material`, `write_phase_energy`, `write_pressure`, scenario pair staging and Sandbox Draw/Erase cover both halves. R1 adds no authorable state. |
| field-specific edge | R1 explicitly separates sealed Air/dynamic missing faces from existing Matter `VOID_TARGET`; sealed evidence uses an in-domain wall ring and cannot rely on domain-edge Matter conservation. This resolves R0 M-003. |
| vent causality | Opening/no-opening matched controls, a predeclared margin, topology identity, following-Tick Air, ordinary Gas movement and quantity accounting resolve R0 M-001 at the design boundary. |
| new state/reference model | No token, owner, packet, CCL, matching, persistent/full-world buffer or test-only replacement simulator is present or required. |
| profiler/pass graph | `42 + pressure_activity_propose = 43` passes and `86` queries. Each profiler buffer is `86*8 = 688` bytes; resolve plus readback is 1,376 bytes. No world allocation changes. |

## 6. Checks performed and omitted

Performed, read-only except for this review file:

- confirmed branch, exact source HEAD and dirty-path boundary;
- hashed the four reviewed design/history inputs;
- traced all 42 current passes, intervening Current/Next copies and the exact
  insertion point for pass 41/42;
- inspected production WGSL and Rust layouts/bind groups for every table row;
- independently derived the donor/receiver conservation and cap inequalities;
- searched every WGSL pressure-bit writer and every movement pressure binding;
- inspected profiler array/query/allocation coupling;
- inspected reset, direct authoring, scenario staging and Sandbox Draw/Erase.

Deliberately omitted because this is a pre-implementation source gate:

```text
reference/coefficient execution: 0
Rust test/check/clippy: 0
WGSL parse/Naga: 0
GPU/device/runtime/FULL: 0
build/bounded launch/candidate: 0
remote/network/GitHub/Wiki operation: 0
production source or primary-design edit: 0
```

## 7. Verdict and boundary

Unresolved Critical: **0**. Unresolved High: **0**. Unresolved Medium: **0**.
Unresolved Low: **0**.

The exact D-037 verdict is:

**TE-5R1 SOURCE-REALIZABILITY PASS — IMPLEMENTATION AUTHORIZED UNDER D-037**

This closes only the mandatory source gate. It establishes no compiled WGSL,
GPU behavior, conservation measurement, allocation observation, performance,
sleep result, rupture/vent fixture, final-source FULL result, release artifact
or user acceptance. ADR-0014 remains **PROPOSED**; implementation must preserve
the reviewed row identities, exactly 43 passes / 86 queries, the eight-storage
ceiling, no new persistent/full-world state and D-037's validation/user-review
boundary.

`LESSON_PROMOTION: NONE` — this review found no new failed architecture or
process lesson; it confirmed the source-bound correction already required by
the preserved R0 finding and D-037.
