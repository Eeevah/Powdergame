# Phase-Volume Pressure Bridge Specification

- **Status:** Candidate specification — DESIGN BLOCKED / architecture revision required
- **ADR:** [`ADR-0007`](../architecture/decisions/ADR-0007-phase-volume-pressure-bridge.md)
- **Authorization:** D-019
- **Depends on:** accepted [`ADR-0006`](../architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md)
- **Design baseline:** `d7500e219af6f670be05f830b50c232d2bb53077`
- **Runtime:** NOT STARTED

This document records the normative TE-5B candidate contract that was examined.
The candidate is not implementation-ready: Section 3.1 gives an unresolved
High finite-capacity counterexample. It describes no existing implementation
and grants no runtime authority.

## 1. Scope

TE-5B supplies one local pressure-volume completion transaction for a
phase-family identity change that crosses the Water-to-Steam endpoint. It:

- preserves one foreground phase-family Cell as one Water-equivalent quantity;
- decides whether one same-tick EMPTY movement opportunity is exclusively
  available;
- emits zero confinement pressure for the winner;
- emits one `100.0` gauge-pressure impulse for a blocked or losing completion;
- reuses the existing proposal/claim, Environment receiver, pressure and
  rupture grammar.

TE-5B does not define a volume fraction, velocity, compressible-fluid solver,
Air-pressure force, Atmosphere/Vacuum structure force, product edge mode,
Vacuum combustion, rupture change, ignition rule or additional Matter.

## 2. Required terms

**Phase-family quantity** is one in-domain Ice, Water or Steam foreground Cell.
Identity changes inside the family do not change its count.

**Completion attempt** exists when trial H reaches/crosses the vaporization
endpoint and either input Water already has initiated positive E (including
ready `E = Lv`) or the current phase context is gas-facing. Buried canonical
`Water E = 0` and non-gas-facing Ice cannot initiate boiling through TE-5B.

**Accepted completion** is a non-edge-deferred attempt for which the TE-5B
function returns a valid targeted or blocked relief word. The word and
provisional Steam/phase-energy Next state are written together before claim and
consequence resolution. Joint settle later commits the identity.

**Completion event** is that accepted provisional result becoming settled
Steam after its relief/pressure consequence is resolved. Remaining Steam is
not a new event.

**Relief target** is one in-domain EMPTY Cell selected from the EMPTY-only
resulting-Steam GAS movement stencil.

**Relief token** is the exclusive same-tick claim on that target. It is an
availability result, not occupancy, a reservation or a movement commit.

**Matter expansion** is the historical generic non-family `matter_yield > 1`
transaction that can spawn target Matter after an Environment receiver wins.

**Confinement impulse** is the gameplay scalar
`WATER_VAPORIZATION_CONFINEMENT_PRESSURE = 100.0` added once at a failed
completion source before existing pressure propagation.

## 3. Relief target function

The target function reads only the immutable in-domain Matter occupancy used
by `phase_thermodynamics`. Let source coordinates be `(x, y)` and:

```text
prefer_left = ((x + y) & 1) == 0
```

Candidate order is:

```text
( 0, -1)                                      // up
(-1, -1), (+1, -1) if prefer_left else reverse
(-1,  0), (+1,  0) if prefer_left else reverse
```

The selector replays production GAS First-Match control flow but can return
only an EMPTY Cell as a relief token. Each stage is handled immediately:

1. out-of-domain returns `EDGE_DEFERRED`; it is not a relief target, and the
   selector does not continue to a later in-domain Cell;
2. EMPTY returns `TARGET(index)`;
3. occupied up/up-diagonal Matter that would accept the resulting Steam's
   legal upward density swap returns `BLOCKED`; the swap is not free volume and
   the selector does not continue past the ordinary First-Match;
4. occupied non-swappable vertical/diagonal Matter continues to the next stage;
5. occupied lateral Matter continues because lateral GAS movement never swaps;
6. exhausting all five in-domain stages returns `BLOCKED`.

Downward coordinates do not exist in the function, and no Cell outside the
five-position stencil is a target. Occupied GAS and density-swap destinations
never become relief targets. Atmospheric Empty and Vacuum Empty both qualify
because eligibility is occupancy/movement based. Air mass, Air energy and
derived Air pressure are not inputs.

`TARGET(index)` becomes the unique source request. `BLOCKED` becomes a blocked
relief request. `EDGE_DEFERRED` writes `REQUEST_NONE`, does not accept the
completion, preserves Water/phase H under ADR-0006 ready-Water rules and emits
no pressure. This deliberately leaves product world-edge pressure/open-
reservoir meaning to full TE-5 while inheriting current Void First-Match enough
to avoid a false lateral target or false confinement impulse. A later explicit
edge decision must revisit this deferral.

Swap eligibility is derived from the current registry's movement class/rank
and packed into an existing phase-descriptor trait word. The phase pass already
binds that 512-byte descriptor; no density-table binding or shadow allocation
is added.

For every `TARGET`, if local occupancy is unchanged, ordinary resulting-Steam
GAS movement reaches that same EMPTY candidate before any other outcome. The
token does not guarantee later occupancy.

### 3.1 Unresolved finite-capacity counterexample

The bridge requires finite EMPTY headspace to become unavailable after ordinary
Steam movement. The evaluated representation cannot enforce that requirement.
In a sealed one-Cell-wide column containing one EMPTY Cell above stagger-heated
Water, only the top Water is ready at `t0`; each lower Cell is just below the
endpoint and reaches it on the tick after the vacancy arrives above. The first
Water wins the token with zero pressure. Ordinary Steam movement then occupies
the EMPTY Cell but vacates the source. During that tick's thermal/phase sequence
the next Water reaches the endpoint and uses the new up-EMPTY target. This
stagger avoids a simultaneous lower attempt seeing an earlier legal Steam
density swap. The vacancy repeats downward until every Water has completed,
with no blocked/losing request and no phase-volume pressure.

The target's required byte-identical Matter/Air state means a win consumes no
resource in the completion tick. The later 1:1 occupancy-conserving movement
only relocates the vacancy. Consequently same-tick PV-INV-002 does not imply
cross-tick finite-volume consumption. A solution needs capacity/reservation
state, a target/Environment transaction, additional occupied quantity, or a
different pressure law; none is authorized by this candidate. No alternate
model is selected here.

## 4. Word encoding

All request and claim words are `u32`:

```text
REQUEST_MODE_MASK          = 0xC0000000
REQUEST_INDEX_MASK         = 0x3FFFFFFF

REQUEST_NONE               = 0x00000000
REQUEST_MATTER_EXPANSION   = 0x40000000
REQUEST_VOLUME_RELIEF      = 0x80000000
REQUEST_INVALID_RESERVED   = 0xC0000000
```

The existing world invariant is strict:

```text
0 < cell_count < (1 << 30)
0 <= cell_index < cell_count
1 <= cell_index + 1 <= 0x3fffffff
```

Encodings:

```text
targeted proposal = mode | (target_index + 1)
blocked proposal  = mode | 0
winning claim     = mode | (source_index + 1)
```

### 4.1 Valid proposal decoder

A proposal is valid exactly when one of these holds:

1. word is exactly `REQUEST_NONE`; or
2. mode is Matter expansion or volume relief and payload is zero; or
3. mode is Matter expansion or volume relief, payload is nonzero and
   `payload - 1 < cell_count`.

Mode `00` with nonzero payload, mode `11` with any payload and an out-of-range
payload are invalid. `expansion_claim` admits none of them and fully writes
zero for a destination with no valid candidate. Source-side pressure logic also
rejects them. Receiver and spawn consumers do not independently bind and
re-read proposal; they consume the trusted full-write output of
`expansion_claim`. Invalid proposals therefore do not invent a target, Matter,
Environment mutation or pressure consequence.

### 4.2 Valid claim decoder

A claim is valid exactly when it is zero, or its mode is Matter expansion or
volume relief and its nonzero payload decodes to a source inside the world.
A nonzero claim produced by `expansion_claim` matches the winning source
proposal's mode and decoded target by construction. That producer rejects
invalid/mismatched candidates and otherwise fully writes the one winner.
Downstream receiver/spawn consumers validate the claim mode and decoded source
range but, under the projected binding counts, do not independently re-bind the
proposal to prove the source/target relationship again. This producer-to-
consumer trust boundary is structural and must be tested at the claim writer;
arbitrary post-writer claim corruption is not promised to fail closed without
an additional binding/design decision.

### 4.3 Round-trip and sentinel separation

For every valid Cell index and either request mode:

```text
decode_index(encode(mode, index)) == index
```

No targeted word equals `REQUEST_NONE`, either blocked-mode word or a reserved
mode word. Payload `0x3fffffff` remains available for the largest legal
`index + 1`; mode bits never overlap it.

## 5. Producer contract

`phase_thermodynamics` fully writes one proposal word for every dispatched
Cell after computing trial H, the initiation/continuity predicate and the
First-Match classifier.

It writes:

- `REQUEST_VOLUME_RELIEF | payload` only for an accepted completion attempt;
- `REQUEST_MATTER_EXPANSION | payload` only for an eligible generic non-family
  extra-Matter event;
- `REQUEST_NONE` for every other Cell, including Steam that merely remains
  Steam, a phase-family non-vaporization transition and a sleeping/context-skip
  self-copy.

A source emits at most one request. A completion attempt cannot also emit a
Matter-expansion request. Phase-family descriptors have yield 1 and never use
the generic extra-Matter path.

For an eligible completion attempt, `TARGET` writes
`REQUEST_VOLUME_RELIEF | (target + 1)` and `BLOCKED` writes
`REQUEST_VOLUME_RELIEF`. That valid word is the explicit TE-5B acceptance
token required by ADR-0006. The same invocation writes provisional 1:1 Steam
identity and phase energy Next. Claim and pressure passes resolve its outcome
before the joint settle commits it.

`EDGE_DEFERRED`, an endpoint that fails initiation/continuity, and every
non-attempt write `REQUEST_NONE` and retain the ADR-0006 Water representation.
Thus no code needs a post-completion request and no unaccepted output Steam is
settled. Arbitration selects only the accepted completion's pressure-volume
consequence; it never reverts a valid accepted attempt.

## 6. Shared claim arbitration

`expansion_claim` fully writes claim for every destination Cell. A Cell can
grant a claim only when its immutable Matter identity is EMPTY.

The destination examines the existing local source neighborhood and admits
only valid targeted Matter-expansion or relief proposals whose decoded target
is this destination. All admitted sources enter one candidate list independent
of mode. The winner is the minimum tuple:

```text
(edge_priority(source_index, target_index, tick), source_index)
```

The destination writes:

```text
winning_source_mode | (winning_source_index + 1)
```

or zero when no valid candidate exists. One destination therefore grants at
most one claim per tick, and one source has at most one request. Mode is not a
separate ownership domain.

## 7. Mode-specific consumers

### 7.1 Environment receiver claim

The expansion Environment receiver producer fully initializes its scratch and
accepts a target only when the winning claim mode is
`REQUEST_MATTER_EXPANSION`. It ignores every relief and invalid claim.

Relief never asks an Air receiver to absorb displaced target Air. Target Air
mass and Air energy are untouched.

### 7.2 Matter spawn commit

The expansion spawn pass commits target Matter only for a valid Matter-
expansion claim with the historical valid Environment receiver. It ignores
relief and invalid claims before decoding/reading a source.

A winning relief claim leaves target Material, temperature, flags, phase
energy, Air mass and Air energy byte-identical. It creates no extra Steam or
other Matter.

### 7.3 Direct expansion/relief pressure

The mode-aware `expansion_pressure` pass writes source pressure from the
already-settled input gauge pressure plus at most one direct consequence:

| Source request | Direct result |
|---|---|
| Matter mode, blocked | add that rule's non-negative blocked pressure |
| Matter mode, targeted claim lost | add that rule's non-negative blocked pressure |
| Matter mode, targeted claim won | add zero here |
| Relief mode, blocked | add `100.0` |
| Relief mode, targeted claim lost or mismatched | add `100.0` |
| Relief mode, matching claim won | add zero |
| none or invalid | add zero |

The requested impulse is exactly `100.0`; storage remains subject only to the
existing finite sanitization and pressure clamp. TE5B fixtures start from zero
gauge pressure where the stored source is exactly `100.0`.

Relief consequence metadata is read from the phase descriptor rather than a
Water-name branch. Every source phase-family descriptor from which pure
normalization can cross the vaporization endpoint in one invocation after the
attempt predicate passes carries the same `100.0` above-completion consequence.
This includes Ice under extreme finite H only with current gas-facing context.
The accepted request mode proves that an eligible endpoint attempt entered the
transaction; the metadata alone never emits pressure.

### 7.4 Environment-blocked pressure

`environment_blocked_expansion_pressure` accepts only a valid Matter-
expansion proposal whose matching destination claim won and whose Environment
receiver failed. It adds the generic blocked consequence once.

It ignores relief at its first mode guard. It cannot add a second relief
impulse, even when the relief target is EMPTY and has no Environment receiver.

### 7.5 Environment reconcile

Only a successful Matter spawn can displace target Environment. A relief
winner leaves target occupancy unchanged and has no receiver, so Environment
reconcile performs no relief mutation. No Air mass or energy is created,
deleted, moved or combined for relief.

## 8. Mixed-mode outcomes

When one Matter-expansion source and one relief source target the same EMPTY
Cell, the shared destination chooses exactly one winner.

- If relief wins, it receives zero pressure; the Matter source receives its
  ordinary losing consequence. No Environment receiver or spawn is created for
  the relief winner.
- If Matter expansion wins and its receiver succeeds, its historical spawn
  transaction proceeds; the relief source receives `100.0` once.
- If Matter expansion wins but its receiver fails, the Matter source receives
  its Environment-blocked consequence once and the relief loser receives
  `100.0` once. The target may remain EMPTY, but there is no same-tick second-
  chance arbitration.

Consequences are source-owned and mode-owned. They are not charged to the
destination and cannot be duplicated by two passes for one source event.

## 9. Phase-family integration

ADR-0006 rules remain unchanged:

- Ice/Water/Steam identity is 1:1;
- phase-family quantity is unchanged through completion;
- positive partial Water phase energy is Matter-owned;
- burial does not erase progress;
- Water at `E = Lv` may remain vaporization-ready and retain excess H as Water
  sensible superheat when completion context is absent;
- a current gas-facing context or this explicit bridge transaction is required
  for Water-to-Steam completion.

With TE-5B in the future atomic source, an initiated/ready or current gas-facing
endpoint attempt first obtains `TARGET`, `BLOCKED` or `EDGE_DEFERRED`.
`TARGET` and `BLOCKED` are explicit accepted transactions: they write a valid
word plus provisional Steam, resolve zero/`100.0`, and commit together at
settle. `EDGE_DEFERRED` retains Water and emits neither request nor pressure.
The bridge does not add or discard H, change `Lf`/`Lv`, modify the phase-energy
range or create an extra destination phase-energy owner.

The attempt predicate is computed from input identity/E, trial H and the
current gas-facing context before output identity is settled. It is not
reconstructed later from temperature. This both lets buried initiated/ready
Water reach the blocked consequence and prevents canonical buried Water or
non-gas-facing extreme Ice from bypassing the initiation/completion gate.

## 10. Generic expansion compatibility

The historical generic path remains available only to a non-family descriptor
with `matter_yield > 1` and a non-phase target. Its selected directional rule
owns its target, Environment receiver, spawn and pressure metadata.

TE-5B does not authorize a generic destination of Ice, Water or Steam. Such a
target still requires a separately approved phase-energy owner/writer design.

Mode-aware decoding must not turn existing generic blocked requests into
relief, drop generic Environment-receiver failure pressure or create a second
winner domain.

## 11. Gauge-pressure boundary

The bridge writes only the existing scalar `pressure[]` consequence at the
completion source. Its semantics remain:

- non-negative gameplay gauge overpressure;
- finite and bounded by existing sanitization/clamp;
- propagated locally only through current Liquid/Gas pressure media;
- reset/cleared on non-pressure Cells under existing rules;
- consumed by existing generic rupture thresholds.

Derived Air pressure is neither read nor combined. Atmospheric Empty and
Vacuum Empty have identical geometric relief eligibility. Structure face
differential, outside background pressure, Vacuum pressure and world-edge
reservoir behavior remain full TE-5 work.

## 12. Pass order and scratch lifetimes

The accepted TE-3 projection remains 40 timestamped passes:

```text
0       activity_wake
1..5    movement propose/claim/commit, flag hygiene, Environment reconcile
6       phase_energy_reconcile_movement
7..10   TE-2 Air scale/commit and thermal scale/commit
11      phase_context_propose: claim full write as context
12      phase_thermodynamics: attempt classification; accepted request + provisional identity full write
13      expansion_claim: claim full write with winner mode
14      expansion_environment_receiver_claim: Matter mode only
15      expansion_spawn_commit: Matter mode only
16      expansion_pressure: direct generic and relief failures
17      environment_blocked_expansion_pressure: Matter mode only
18..19  identity/phase-energy hygiene and Environment reconcile
settle  Material, temperature, phase energy, pressure and Environment
20..30  decay and combustion/Smoke transactions plus hygiene/reconcile
31      pressure propagation
settle  pressure
32..35  rupture, hygiene and Environment reconcile
36      base activity_propose
37      phase_activity_propose
38      environment_activity_propose
39      activity_reduce
```

No consumer may retain a reference to proposal/claim meaning after its listed
lifetime. Later combustion fully overwrites proposal; later Smoke arbitration
fully overwrites claim. Reserved mode bits therefore cannot leak into a Smoke,
movement or next-tick interpretation.

Exact live ranges are:

- phase context in claim: written at 11, consumed at 12, dead before the full
  claim write at 13;
- expansion proposal mode: written at 12, consumed by claim/direct pressure
  through 16 and by Matter-only Environment-blocked pressure through 17, then
  dead before the later combustion proposal full write;
- expansion claim mode: written at 13, consumed by receiver/spawn/direct and
  Environment-blocked pressure through 17, then dead before the later Smoke
  claim full write;
- `environment_receiver_claim`: fully initialized at 14, consumed only by the
  Matter-mode spawn, Environment-blocked consequence and Environment reconcile
  through 19, then dead and fully initialized again before its Smoke lifetime.

Pressure consequence settles before pass 31; propagated pressure settles
before rupture. Phase energy settles with the identity owner. There is no
TE-5B-only settle, copy or dispatch.

## 13. Binding ceilings

| Pass | Storage bindings | Limit result |
|---|---:|---|
| `phase_thermodynamics` | `4 RO + 4 RW = 8` | at ceiling; no new binding |
| `expansion_claim` | `3 RO + 1 RW = 4` | mode arithmetic only |
| Environment receiver claim | `4 RO + 1 RW = 5` | mode guard only |
| expansion spawn commit | `5 RO + 3 RW = 8` | at ceiling; mode guard only |
| expansion pressure | `7 RO + 1 RW = 8` | at ceiling; reuses descriptor input |
| Environment-blocked pressure | `6 RO + 1 RW = 7` | mode guard only |

The existing params/arbitration/descriptor uniforms do not add storage
bindings. The phase descriptor is one existing projected 512-byte allocation
with the already-planned uniform/storage views. The family consequence scalar
uses an existing pressure-metadata slot. A packed movement trait derived from
the current registry lets the phase pass distinguish non-swappable occupancy
from an earlier legal Steam density swap while replaying GAS First-Match. No
density-table binding or shadow allocation is allowed.

If implementation discovers that any row cannot be expressed at these counts
or with these lifetimes, TE-5B is blocked pending a new design decision. It
must not silently add a pass, scratch buffer or binding.

## 14. Activity, sleep, reset and staging

An accepted phase completion attempt and a nonzero pressure consequence are
work and must leave the source and required existing halo observable to
subsequent pressure, rupture and activity logic. A sleeping chunk cannot enter
the transaction while skipped; skip paths copy self and write `REQUEST_NONE`.
`EDGE_DEFERRED` is stable ready-Water state and may sleep under the existing
phase rules until movement/edit/context makes an in-domain transaction possible.

Sleep-on and sleep-off execution must produce equivalent Material, temperature,
phase energy, pressure, Air and mode-consumer outcomes for the same executed
ticks. Exact wake propagation remains future implementation evidence.

Proposal and claim are scratch. Reset and staging need no persistent mode
initialization because every relevant producer fully overwrites before use.
World reset must still stage both phase-energy halves canonically and pressure
at its existing reference value. Repeating TE5B-F09 after reset must reproduce
the same one completion/one consequence trace.

## 15. Cost projection

Relative to accepted TE-3D:

| Resource | TE-3D projection | TE-5B delta | Combined projection |
|---|---:|---:|---:|
| timestamped passes | 40 | 0 | 40 |
| timestamp queries | 80 | 0 | 80 |
| profiler buffers | 1,280 B | 0 | 1,280 B |
| persistent/full-world buffers | two phase-energy halves | 0 | unchanged |
| 256² tracked bytes with profiler | 4,722,608 B | 0 | 4,722,608 B |
| 2048² tracked bytes with profiler | 302,018,096 B | 0 | 302,018,096 B |

These are design arithmetic, not measured runtime allocation or performance.

## 16. Required invariants

- **PV-INV-001 — Unit quantity:** Water-to-Steam remains 1:1; no relief outcome changes phase-family quantity.
- **PV-INV-002 — Exclusive target:** One EMPTY target grants at most one volume-relief claim per tick.
- **PV-INV-003 — Non-mutating winner:** A winning relief claim creates no Matter and changes no Environment state.
- **PV-INV-004 — Failed consequence:** A blocked or losing relief request generates pressure `100.0` exactly once.
- **PV-INV-005 — Winning consequence:** A winning relief request generates zero phase-volume pressure.
- **PV-INV-006 — Movement reachability:** Every relief target is the first EMPTY outcome reached by existing GAS First-Match control flow; Void ends in edge deferral and an earlier legal density swap cannot be skipped.
- **PV-INV-007 — Shared ownership:** Matter-expansion and relief requests contend in one deterministic ownership domain.
- **PV-INV-008 — Receiver/spawn isolation:** Environment receiver and spawn passes ignore relief mode.
- **PV-INV-009 — Environment-pressure isolation:** Environment-blocked pressure ignores relief mode.
- **PV-INV-010 — Scratch hygiene:** No proposal/claim mode bit survives into another scratch lifetime.
- **PV-INV-011 — No new state:** No new persistent or full-world scratch state is required.
- **PV-INV-012 — Gauge meaning:** Existing gauge-pressure meaning is unchanged.
- **PV-INV-013 — No false open source:** Open boiling does not receive a confinement source when its relief claim wins; a current Void-first edge attempt is deferred without pressure rather than mapped to a false lateral target.
- **PV-INV-014 — Frozen consequence:** Confined or contended boiling preserves the named G5 pressure consequence `100.0`.
- **PV-INV-015 — Data-driven source:** Pressure generation is descriptor/accepted-attempt driven, not a Water-name WGSL branch.
- **PV-INV-016 — Atomic G5 continuity:** The atomic TE-3/TE-5B source preserves the full G5 causal chain with new source-bound evidence.
- **PV-INV-017 — Full TE-5 exclusion:** Background-pressure and structure-differential coupling remain not started.
- **PV-INV-018 — Finite-capacity consumption (UNSATISFIED):** A finite sealed EMPTY headspace must eventually stop granting relief under continued 1:1 boiling. The evaluated non-mutating token fails this invariant because ordinary movement relocates rather than consumes the vacancy.

## 17. Evidence boundary

The one-shot pure reference proof established only encoding, deterministic
arbitration, abstract consequence accounting, target non-mutation, quantity and
scratch-lifetime properties in its modeled domain. It did not construct a grid
or model vacancy conservation, cross-tick route reuse or finite-capacity
exhaustion. It therefore cannot clear PV-INV-018 or establish WGSL bindings,
GPU races, actual movement, sleep, pressure propagation, rupture, venting,
performance, visual quality or user acceptance.

Those layers and TE5B-F01 through F12 are defined in
[`PHASE_VOLUME_PRESSURE_BRIDGE_VALIDATION.md`](../development/PHASE_VOLUME_PRESSURE_BRIDGE_VALIDATION.md).
