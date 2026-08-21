# TE-3Q / TE-5Q Conservative Phase Packets — Fresh Independent Adversarial Review

- **Review scope:** frozen docs/reference candidate only
- **Branch:** `feature/m0-g9-first-playable`
- **Reviewed HEAD:** `3a427974f45dd416190849d4b68528437b879d64`
- **Evidence identity:** `TE3Q-PHASE-PACKETS-REFERENCE-V1`
- **Independent disposition:** **TE-3Q / TE-5Q DESIGN BLOCKED / ADR-0011 PROPOSED / RUNTIME NOT STARTED**
- **Unresolved counts:** Critical **0** / High **8** / Medium **1** / Low **0**

## 1. Review boundary and method

This is a fresh-context, adversarial static review of the frozen ADR,
specification, validation contract, planning page, prior-art survey, production
inventory, external proof source and result JSON. The current Rust/WGSL source
was read only to test production-integration feasibility.

No Rust, WGSL, Cargo, build, application launch, runtime, GPU/device, candidate,
or proof execution was performed. The external proof was only read, SHA-256
hashed and its existing JSON result parsed. TE-5X is preserved failed history;
its script, receipt and candidate evaluations were not reused as evidence.

The frozen script/result pair is authentic for the program that ran. This
review does not allege hash drift or result tampering. It finds that the program
does not execute several obligations its validation document says are modeled,
and that the frozen architecture still has production-semantic counterexamples.

## 2. Severity summary

| Severity | Open | Meaning |
|---|---:|---|
| Critical | 0 | No repository-corruption, evidence-tampering or unavoidable quantity-creation defect was found in the frozen bytes. |
| High | 8 | Any one blocks the D-023 success stop. The one-shot proof identity cannot be patched or rerun to close them. |
| Medium | 1 | Product meaning remains unresolved even after correctness repair. |
| Low | 0 | No separate low-severity item is recorded. |

## 3. High findings

### H-01 — The frozen fixture matrix violates its own “callable model path; label-only forbidden” rule

The validation document says every PQ-F01..F14 row is an executable modeled
assertion and expressly forbids a label-only `MODELED_PASS`. The script does not
meet that contract:

- **PQ-F08** constructs four already-separated adjacent pairs, performs one
  pairing call and then checks that no adjacent pair remains. There is no cold
  lid, positive-conductance condensation sink, competing proposal graph,
  checkerboard, chunk seam, multi-tick collection or adversarial greedy order.
- **PQ-F09** does not build an open beaker or advance a tick. It creates a Water
  object, directly splits it, discards the resulting packet states, constructs
  two new cooled Steam objects, merges them, and returns the literal strings
  `heat/split/rise/cool/merge/fall`. Rise, fall, cooling, Environment transfer,
  movement and spatial conservation are not executed.
- **PQ-F11** swaps two local variables once. It never enters the movement,
  density arbitration, chunk partition or sleep paths, then returns
  `chunk_sizes=[4,8]` and `sleep_equivalent=True` as constants.
- **PQ-F13** validates four single `Cell` objects and rejects only Water/1. It
  has no Current/Next halves and does not execute Draw, Erase, reset, preset,
  scenario, benchmark or Inspector staging.
- **PQ-F14** returns two Boolean constants and inspects no historical evidence,
  source identity, runtime receipt or obligation registry.

This directly contradicts the frozen fixture contract. The reported `14 / 14
PASS` is a true report of those Python functions returning, but it is not proof
that the named assertions ran.

**Required resolution:** Keep this script/result immutable. A later explicitly
authorized evidence identity must replace the label/synthetic fixtures with
executable state machines and fail if a named path was not exercised. D-023's
current one-shot evidence condition is not met.

### H-02 — The 100,000/10,000 campaigns and deterministic replay exercise an under-specified model, not the required accounting boundary

The scale counters are real: the source contains 100,000 algebra iterations,
10,000 grid iterations, and `main` calls the complete `execute(SEED)` twice and
compares the canonical digest. The parsed JSON matches those counts. Coverage,
however, omits required state and can hide loss:

- every random grid starts all EMPTY Cells as canonical Vacuum, so nonzero Air
  is absent;
- grid split calls `split_packet` and overwrites the target without executing
  `receive_air` or selecting/validating an Environment receiver;
- the random grid never performs an out-of-domain move; `void_units` remains a
  constant zero;
- there is no chunk partition, sleep state, wake halo, receiver scratch,
  Current/Next settle, proposal/claim endpoint arbitration or destructive edit;
- the so-called density swap is an unconditional tuple exchange between
  phase-family Cells and does not apply the production class/rank/claim rules;
- replay proves determinism of this same reduced Python model, not equivalence
  to the named production contract.

Consequently the automatic criteria “split and merge Air accounting is exact,”
“all grids conserve except logged Void/destructive events,” and PQ-F11's
ownership/sleep/chunk assertion were not exercised by the campaigns.

**Required resolution:** A new authorized proof must include nonzero Atmosphere
Air, actual receiver selection and failure, merge-created Vacuum, explicit Void
and destructive logs, Current/Next ownership, production-equivalent edge claim,
chunk seams and sleep-on/off state equivalence. Merely increasing trial counts
or replaying the same reduced model cannot close this finding.

### H-03 — Orthogonal one-shot contraction can irreversibly strand an even quantity, and the proof uses the wrong arbitration algorithm

The architecture permits each ready Steam/1 to propose one orthogonal partner,
then commits winning pairs irreversibly to Water/2 + Vacuum. A four-packet line
is a counterexample class: if the middle edge wins first, the two endpoint
packets become non-adjacent after the middle Water/Vacuum commit. Under a cold
lid or other motion-blocked geometry they can never meet again, even though the
component began with an even packet count. This is stronger than the explicitly
accepted single-packet metastability; a locally greedy commit manufactured two
stranded half packets from a fully pairable component.

The Python `deterministic_pairs` implementation does a host-sequential global
sort followed by a mutable `used` set. Current production arbitration instead
uses parallel per-endpoint reciprocal edge claims: an edge commits only when
both endpoints select it. These algorithms are not equivalent, and the frozen
specification does not pin which reciprocal encoding, priority comparison and
mutual-winner condition the five merge passes implement. PQ-F08 avoids all
competition, so it cannot distinguish them or detect greedy residue.

**Required resolution:** Define a GPU-realizable endpoint-exclusive algorithm
and a liveness boundary for even local components, then prove adversarial paths,
rings, dense rectangles, checkerboards and chunk-seam graphs. If irreversible
local pairing is intentionally allowed to strand multiple half packets, that
is an architecture/product amendment, not the currently claimed F08 result.
Any extra iteration/pass/scratch must update the exact 50-pass authority before
another proof is frozen.

### H-04 — Movable Steam/2 can reset spatial phase pressure and bypass the intended compression-to-rupture chain

`phase_pressure` is explicitly spatial and is not carried with Matter. Current
production order moves GAS Matter before any proposed phase-pressure update.
Steam/2 retains ordinary Steam GAS movement. Therefore a compressed Steam/2
packet can move into a fresh EMPTY Cell, carrying units and phase energy while
leaving its spatial pressure behind. The old source becomes non-media and is
zeroed; the new location begins from the EMPTY Cell's prior pressure, normally
zero, and rises only to 20. Repeating through available Cells can restart at 20
each tick instead of reaching Wood's threshold on residence update 8.

This is especially relevant when split fails for lack of an Environment
receiver: ordinary GAS movement can walk the same EMPTY vacancy while vacating
its previous source. The fixture's sealed boiler never moves the compressed
packet, and PQ-F11 does not run movement, so the proof cannot see the bypass.
Quantity remains numerically conserved, but the causal confinement/rupture
contract is lost.

**Required resolution:** Freeze one coherent rule: prevent Steam/2 movement,
move the causal pressure/source-age with Steam/2, update compression before
movement with an equivalent invariant, or adopt another explicitly reviewed
causal representation. Then prove a moving compressed packet, vacancy walking,
density interaction, chunk crossings and sleep-on/off equivalence.

### H-05 — Condensation or relief can still rupture Wood after the pressure source is gone

For an isolated Cell at `phase_pressure=100`, split or condensation changes the
equilibrium source to zero. The frozen law still produces
`100 + 0.20*(0-100) = 80` on the next phase-pressure pass because non-media
neighbors add no diffusion term. Rupture then reads the combined pressure and
the current Wood contract ruptures at `>= 80`. Thus a Cell can remove its
compressed-Steam source and nevertheless rupture adjacent Wood in the same
settle window.

PQ-F04 starts relief at 85 with an adjacent pressure-medium packet, while
PQ-F06 starts condensation at 90. Neither covers the exact 100-to-80 threshold
edge. The automatic statement that split/condensation produces a safe lower
later pressure is insufficient: lower can still be rupture-eligible.

**Required resolution:** Define the causal order and post-source-removal rule
so a successful relief/condensation cannot create a false threshold hit, or
explicitly accept delayed stored stress as product semantics. Add exact 100,
neighbor-medium/non-medium, generic-pressure-present and same-tick rupture
fixtures under the next authorized evidence identity.

### H-06 — Split/merge H ownership and generic-pressure exclusion are assertions, not a frozen production transaction

The Python split gives one packet `E/2` and the other `E-E/2`, which gives a
clear residual owner under exact `Fraction` arithmetic. The frozen spec does
not pin that asymmetric residual rule, f32 evaluation order, sanitization or
which endpoint owns the residual in WGSL. Its tolerance constants are not used
by the proof; equality is exact rational equality. A future implementation in
which both endpoint writers independently evaluate `E*0.5` can lose or create
an f32 residual while still producing valid units.

The current expansion pipeline also derives generic blocked pressure from the
Water phase descriptor whenever `matter_yield > 1` loses or blocks. The new
documents say packet compression must not also receive generic blocked
pressure, but do not freeze the descriptor/mode encoding that prevents both
the historical generic pass and the new phase-pressure source from applying.
Similarly, the separate split commit must distinguish claim loss from receiver
failure and update source/target units and E exactly once after the existing
spawn without duplicating the source parcel. The representative binding counts
do not name those exact bindings or endpoint writer responsibilities.

**Required resolution:** Freeze a bit-level transaction table for Water/2 and
Steam/2 split outcomes, including source/target writer, residual-E owner,
f32 operation order, claim/receiver predicates, invalid-value fail-close and
zero generic-pressure consequence for the phase-packet mode. Prove each branch
against production-equivalent proposal, claim and receiver encodings.

### H-07 — Required writer, reset, editor and evidence paths are not source-closed, and PQ-F13 cannot establish fail-closed state

The current `GpuWorld`, reset, scenario staging, Sandbox edit, experiment
snapshot/hash, Inspector and renderer know nothing about phase units, phase
energy or phase pressure. This is expected for `RUNTIME NOT STARTED`, but the
architecture claims an exact integration projection while only stating that
all bypass writers “must” canonicalize.

The current Sandbox field edit already binds commands plus Material,
temperature and generic-pressure Current/Next pairs. The accepted phase-energy
design required a separate non-timestamped edit dispatch; adding units and
phase pressure requires an exact additional pre-field ordering and binding
plan. None is listed. Scenario fixtures, reset bulk uploads, direct world write
helpers, experiment physical-boundary equality/hash and Inspector sampling also
need explicit inclusion or an explicit exclusion with a replacement audit.
PQ-F13 has no Current/Next data structure and cannot test any of these paths.

Invalid combinations are said to “fail closed,” but the closed result is not
defined: reject a command, preserve previous state, canonicalize to EMPTY, or
surface a fatal invariant. Those choices have different quantity and authoring
semantics.

**Required resolution:** Add a complete authoritative-writer matrix with
Current/Next values, ordering, binding counts, reset/scenario/direct-write
coverage, snapshot/hash/readback coverage and one precise invalid-state action.
The production-tick count may remain 50 only if all out-of-tick dispatches are
named and cannot race the next tick.

### H-08 — The phase-pressure pass cannot both honor sleep and match its normative binding row

The normative table gives phase pressure `3 RO + 1 RW`. The pass needs at
least Material identity, `phase_units_current`, `phase_pressure_current` and
`chunk_state` as storage reads to distinguish Steam/2 from Water/2 and skip a
sleeping chunk, plus `phase_pressure_next` RW. A packed Material-property table
can be uniform, but it cannot remove any of those four per-world inputs.

If `chunk_state` is omitted to preserve `3+1`, phase pressure mutates sleeping
chunks while movement, phase, rupture and other production passes skip them.
That can change or erase pressure without contemporaneous rupture and breaks
the sleep-on/off equivalence requirement. If `chunk_state` is included, the
normative binding row and source projection are wrong. The proof has no chunk
or sleep model and PQ-F11's `sleep_equivalent=True` is a literal constant.
The phase-pressure activity predicate is also not defined as the same
epsilon-gated update predicate, so it can either leak activity forever or let
unfinished pressure work sleep.

The numerical pass arithmetic is internally consistent—40 + 10 = 50—and all
listed rows are at or below eight. This missing required input means the exact
table as frozen is not yet source-realizable, even though the device ceiling
itself is not exceeded by adding a fourth RO binding.

**Required resolution:** Correct the normative binding row, define sleep skip/
copy behavior and pin phase-pressure activity to the identical source,
diffusion and epsilon work predicate. Prove sleeping/runnable equivalence,
neighbor-halo wake, source movement, source removal and sub-epsilon settle.

## 4. Medium finding

### M-01 — Steam/1 and Steam/2 have no distinct collision or visual meaning in the current product surface

Both packets use the same Steam Material ID, movement class, density rank and
renderer identity. Equal-rank Steam does not density-swap, hot Steam/1 packets
do not combine on collision, and only condensation-ready orthogonal pairs can
merge. Steam/1 and Steam/2 therefore look identical while representing
different quantity and while only Steam/2 sources phase pressure. The current
Inspector payload does not expose units or phase pressure.

This is not promoted to High because ADR-0011 already leaves visual/product
meaning to user architecture review. It remains a required user decision and
will need a deliberate renderer/Inspector choice before runtime acceptance.

## 5. Resolved checks and non-findings

1. **Frozen authority integrity:** all six repository authority files and both
   external proof files match their declared SHA-256 values.
2. **Receipt parse:** the result is valid JSON, embeds the matching script hash,
   reports process execution `1`, status `PASS`, 100,000 algebra trials, 10,000
   grids and an in-process replay digest.
3. **Standard-library boundary:** the proof imports only Python standard-library
   modules. No external implementation dependency is present.
4. **No mixed foreground Matter:** the candidate keeps one Material ID per Cell;
   units are Matter-owned scalar state, not a second co-resident Matter.
5. **No external code ingress:** the reviewed diff/proof contains no copied,
   translated or vendored simulation implementation; observed count remains
   `0 files / 0 lines`.
6. **Integer quantity algebra inside the reduced model:** split/merge units are
   exact integers and the script's rational enthalpy equations are internally
   consistent for the states it constructs. This does not resolve H-01/H-02/H-06.
7. **96 MiB arithmetic:** three Current/Next 4-byte full-world pairs at 2048²
   are `3 * 2 * 4 * 4,194,304 = 100,663,296 B = 96 MiB`. At 256² the increment
   is 1,572,864 B. The stated 50-pass profiler buffers add 1,600 B total, so the
   documented tracked totals are arithmetically consistent with the current
   inventory boundary.
8. **Pass-count arithmetic:** the named delta is ten passes over the accepted
   40-pass TE-3 projection. This verifies addition only, not H-07/H-08's exact
   source feasibility.
9. **Generic field separation as intent:** the documents consistently state
   that generic pressure is not erased and rupture combines it with phase
   pressure once. H-06 concerns the missing production-mode encoding, not the
   stated intent.

## 6. Mandatory attack matrix

| Required attack | Result |
|---|---|
| spawn quantity replication | **Blocked:** H-06; existing spawn plus separate split commit lacks frozen endpoint writer table. |
| split H rounding/loss | **Blocked:** H-06; rational proof has a residual owner, normative f32 transaction does not. |
| local merge starvation/greediness/checkerboard | **Blocked:** H-01/H-03. |
| half-Steam collision/visual meaning | **Open Medium:** M-01 and user architecture review. |
| Steam/2 movement compression bypass | **Blocked:** H-04. |
| spatial pressure after split/condensation | **Blocked:** H-05. |
| generic pressure erasure/double-count | Erasure intent is separated; **double-count implementation remains blocked:** H-06. |
| Environment receiver/merge Air loss | **Blocked evidence:** H-02/H-07; nonzero Air and production merge reconcile are not modeled. |
| movement/density/chunk/Void unit loss | **Blocked evidence:** H-01/H-02. |
| staging/editor invalid combinations | **Blocked:** H-01/H-07. |
| activity sleep/leak | **Blocked:** H-01/H-08. |
| exact 50 / <=8 / scratch lifetimes | Arithmetic/scratch window is plausible; **exact source closure blocked:** H-03/H-06/H-07/H-08. |
| 96 MiB arithmetic | **Pass (arithmetic only).** |
| no mixed Matter / external code | **Pass for reviewed bytes.** |
| F08/F09/F11/F14 actual execution | **Fail:** H-01; functions return, named behaviors do not execute. |
| 100k/10k/deterministic replay | Counts and replay call **pass**; obligation coverage **fails:** H-02. |
| current simulation/WGSL integration | **Blocked:** H-03 through H-08. |

## 7. Hash snapshot

| Frozen authority | SHA-256 | Review result |
|---|---|---|
| `docs/architecture/decisions/ADR-0011-conservative-phase-packets.md` | `136c314ca0ed564b49a99a98b68cc6402811824ac78458eee27d99fe95fcd5f6` | exact match |
| `docs/specs/CONSERVATIVE_PHASE_PACKETS_SPEC.md` | `06f2ec7038603cdeb6c0159937543cc4a87f6a0318701dcac3950fffa959ec09` | exact match |
| `docs/development/CONSERVATIVE_PHASE_PACKETS_VALIDATION.md` | `8d7fc72e8acb5f7c96b2baf65b8deca1c5a658b4bc342b02d249f21f4ad9a64b` | exact match |
| `docs/planning/TE3Q_CONSERVATIVE_PHASE_PACKETS.md` | `3b907a1bf2f18ec2cd6caecea1850f520e85384e6929caac83fd25148f2e0fca` | exact match |
| `docs/research/2026-08-21-conservative-phase-quantity-prior-art.md` | `d1c0f7666d4e3076bc391c4efa6d750671b68542cc28ee4a90295576b5d8afa5` | exact match |
| `docs/architecture/THERMAL_ENVIRONMENT_PRODUCTION_INVENTORY.md` | `c29e9fd8f91e11aea844091bb62ff6f16452e05b0f7616972a1816cf4ddef5b7` | exact match |
| external `te3q_phase_packets_reference_v1.py` | `c938c6e3ce7074abc6d5144c708f85a17be349bb84f962238e568c17d55ed03c` | exact match; read only, not executed |
| external `te3q_phase_packets_reference_v1_result.json` | `a0181d4ca0ed63eb92cac5cd04098ff438546903c8dc6853e8b0b5d5ab208ed7` | exact match; JSON parse succeeded |

## 8. Evidence limitations

- Static source inspection cannot establish WGSL compilation, Naga acceptance,
  bind-group creation, shader races, f32 device behavior, allocation success,
  GPU timing, performance, visual readability or user acceptance.
- No runtime or proof command was authorized or run, so this review creates no
  implementation receipt and does not alter the frozen proof history.
- The existing JSON proves what the frozen Python program evaluated. It cannot
  be widened after execution to cover behaviors absent from that program.
- The working tree already contained the frozen candidate documents. This
  review changes only this adversarial-review file and does not rewrite their
  hashes, the checkpoint, decisions, status, inventory or Wiki.

## 9. Final disposition

Unresolved High findings are nonzero. Under D-023 the only permitted outcome is:

> **TE-3Q / TE-5Q DESIGN BLOCKED / ADR-0011 PROPOSED / RUNTIME NOT STARTED**

The `MATHEMATICAL_REFERENCE_PASS` receipt remains immutable but is insufficient
for architecture advancement. Do not accept ADR-0011, begin Rust/WGSL work,
reuse TE-5X evidence, patch/rerun this proof identity, or silently add passes,
scratch, pressure semantics or another model. Closing the blockers requires a
new explicit user authorization, revised frozen authority and a new evidence
identity.
