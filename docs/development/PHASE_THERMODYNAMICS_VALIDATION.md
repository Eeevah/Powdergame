# Phase Thermodynamics Validation Contract

- **Status:** SOURCE-BOUND RUNTIME VALIDATION PASS / USER REVIEW PENDING
- **Architecture:** [`ADR-0006`](../architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md)
- **Specification:** [`PHASE_THERMODYNAMICS_SPEC`](../specs/PHASE_THERMODYNAMICS_SPEC.md)
- **Reference result:** v1 preserved; v2 `PASS_AMENDED_REFERENCE_MATH_ONLY`
- **User acceptance:** Hybrid A+C core and locked amendments accepted

This contract separates pure reference evidence from future Rust/WGSL,
production-pass, GPU, sleep, performance, visual and user evidence. Passing one
layer MUST NOT be relabelled as another.

## 1. Evidence layers

| Layer | What it may establish | Current state |
|---|---|---|
| docs/static audit | coherent state, writer, pass, binding and fixture design | fresh v2 review PASS; unresolved Critical 0 / High 0 |
| pure reference math | piecewise enthalpy formulas, f32 tolerance, coefficient sweep, deterministic seed properties | v1 preserved; v2 passed its only run |
| CPU semantic tests | future Core reference implementation matches this spec | not run / no implementation |
| Naga/write-contract | future WGSL parses and respects write-self/binding rules | not run |
| production GPU fixtures | actual buffers, order, movement, hygiene and sleep semantics | not run |
| profiler/allocation | actual pass/query/bytes and cost | not run; arithmetic projection only |
| product observation | visual timing, nucleation appearance and traffic-jam outcome | not run / user pending |

### 1.1 D-024 runtime receipt

Runtime source `41467219819c5d0cb3eab8ae22b652449da20480` passed Core semantics,
Naga/binding/write contracts, all actual TE3-F01–F15 GPU fixtures, movement and
identity hygiene, sleep/wake, TE-2 regressions, workspace check, warnings-denied
clippy, strict policy audit and the final workspace FULL. One earlier FULL
attempt failed only on stale 34-pass benchmark expectations; a second failed
only on the old pre-32-MiB headless allocation literal. Both were corrected
before the final-source FULL PASS. Release build count is one. The bounded
candidate launch and measurement count is one. Direct user review remains
pending and is not implied by automation.

## 2. Reference proof records

### 2.1 Preserved v1 receipt

The pure tool and result live outside the repository and production runtime.
It was executed exactly once.

```powershell
& 'C:\Users\mdkap\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' 'C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te3_phase_enthalpy_reference.py' --output 'C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te3_phase_enthalpy_reference_result.json'
```

Identity:

| Item | Value |
|---|---|
| fixed seed | `0x54453344` |
| script SHA-256 | `117439a84f1debdc4e4cca6007a4307903bc643cb1811f8c0d979dfecda05561` |
| result SHA-256 | `6c1afe9f3734be51301562ee3363a94726a75c1f64c222c3dc824ed31d19e42e` |
| random enthalpy trials | `50,000` |
| generated finite nucleation regions | `4,096` |
| maximum absolute H error | `1.52587890625e-05` |
| declared tolerance | `max(1e-3, 2e-6 * max(1, |H_before|, |H_after|))` |
| closed heat/cool cycles | `100` |
| final quantity / state | `1`; Water, 20°C, E=0 |
| generated-region seed range | `1..24` |
| chunk-seam comparisons | `4,096` |

The result JSON was parsed with PowerShell `ConvertFrom-Json`; status was
`PASS_REFERENCE_MATH_ONLY` and it contained no NaN/Infinity JSON value.

Proved by this tool:

- Ice/Water and Water/Steam endpoint continuity;
- normalization H preservation within the declared f32 tolerance;
- partial boiling, condensation and melting reversal;
- 100 repeated closed cycles with one Water-equivalent unit;
- finite phase-energy values inside identity ranges;
- excess sensible energy after endpoint completion;
- partial progress retained when initiating context disappears;
- identical accounting with and without surface eligibility;
- at least one deterministic seed in every generated finite canonical-Steam
  snapshot;
- no adjacent same-snapshot seeds, hash-tie failure or chunk-seam discrepancy;
- no extra Matter quantity and proposed Water boil yield 1.

Not proved:

- WGSL race freedom, actual bindings or pass order;
- movement, density swap or identity-writer hygiene;
- sleep/wake equivalence or chunk scheduling;
- temporal nucleation rate, partial-progress veto or moving-seed shadows;
- GPU performance/allocation or visual quality;
- acceptable free-air nucleation appearance;
- user architecture acceptance.

The v1 script/result remain immutable historical evidence. They do not prove
the D-018 real-sink, vaporization-ready or radius-2 amendments and MUST NOT be
relabeled as the v2 receipt.

### 2.2 Predeclared amended v2 proof contract and receipt

Before the one v2 execution, lock these pass conditions:

- K=0 Boundary never qualifies as a surface sink, changes E or creates phase
  activity by itself;
- buried positive-E Water retains, increases and reverses H correctly;
- Water at E=Lv without a completion context remains vaporization-ready Water,
  preserves excess sensible H and cannot bypass the completion gate;
- reopening a gas surface and a contract-only accepted TE-5 placeholder each
  complete 1:1 from the same H;
- isolated canonical and partial supercooled Steam retain finite identity/E/H
  without activity, then resume when a real cooling face returns;
- radius 1, 2 and 3 static/temporal metrics are disclosed, while radius 2 alone
  is normative;
- radius-2 seeds and active partial veto are symmetric across global
  coordinates, chunk seams, fixed hash ties, movement, completion and Void
  release;
- thin and diagonal regions retain a seed under the radius-2 connected-graph
  definition;
- every sampled 30-tick TE3-F08 window satisfies
  `new initiations <= max(4, ceil(peak eligible canonical Steam / 8))`;
- the synthetic generic yield-2 compatibility target is non-phase Matter.

If normative radius 2 fails any condition, the result is `DESIGN_BLOCKED` and
radius 3 MUST NOT be selected silently.

The conditions above were written before execution. The new tool/result use new
filenames outside the repository; the preserved v1 files were not overwritten.
The v2 tool was then executed exactly once:

```powershell
& 'C:\Users\mdkap\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' 'C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te3_phase_enthalpy_v2_reference.py' --output 'C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te3_phase_enthalpy_v2_reference_result.json'
```

Identity and result:

| Item | Value |
|---|---|
| status | `PASS_AMENDED_REFERENCE_MATH_ONLY` |
| fixed seed | `0x54453344` |
| script SHA-256 | `c3624e467638a62ef2b62f96c8b12954ceef70609feeac47da70eca69f84db23` |
| result SHA-256 | `f727101543f4eaa7582def01940e2567dd3b79bc6e585cfad4051160de1d90ea` |
| random enthalpy trials | `50,000` |
| generated finite nucleation regions | `4,096` |
| maximum absolute H error | `1.52587890625e-05` |
| closed heat/cool cycles | `100`; final quantity `1`; Water, 20°C, E=0 |
| generated-region radius-2 seed range | `1..12` |
| chunk-seam comparisons | `4,096` |
| temporal completion/veto release tick | `32` |

The result JSON parsed successfully with PowerShell `ConvertFrom-Json` and
contains no NaN/Infinity JSON value. The normative radius-2 60-tick reference
abstraction recorded 401 total initiations, at most 209 in any tick or sampled
30-tick window, and passed every predeclared window bound. Disclosure-only
radius 1 recorded 1,098 total and a 578 maximum window; radius 3 recorded 222
total and a 120 maximum window. All three disclosed models passed their own
window bounds, so radius 2 remains the preselected rule rather than a fallback.

The v2 receipt exercised pure-state/predicate abstractions for buried initiated
Water, value-derived ready Water, sensible-before-latent cooling,
gas/placeholder completion gates, K=0/no-work rejection, isolated Steam hold,
moving radius-2 partial veto, active-veto release, thin/diagonal/seam/tie
coverage and a declared non-phase synthetic yield-2 target. Completion, Void
and stalled-no-work release share one reference abstraction—the owner is
absent from the thermally runnable active-veto set—so this run does not prove
their distinct runtime writers. The K=0 check models the required predicate,
not `phase_activity` execution or wake/resume. The generic target and mixer
provenance checks pin declared values but do not establish a registry or WGSL
guard. The receipt still does not execute Rust, WGSL, GPU, sleep scheduling,
TE-5 or a product scene; the named future structural/GPU fixtures retain all
of those obligations.

### 2.3 Fresh independent v2 review receipt

The fresh-context reviewer inspected the amended primary authorities, current
production source and the already-existing v2 artifacts without rerunning the
proof. The review is preserved in
[`TE3_PHASE_ENTHALPY_DESIGN.md`](../adversarial-reviews/TE3_PHASE_ENTHALPY_DESIGN.md)
at SHA-256
`c6d63fd84d8057e6cbe201696df0a4914e1a396eaeaf2bc189a5ebcd24a9a31d`.

Disposition:

```text
INDEPENDENT V2 DESIGN REVIEW PASS
UNRESOLVED CRITICAL 0 / HIGH 0
```

All eight v2 High attacks are closed at their named design/reference layer.
Three Medium and two Low items remain explicitly open for future runtime,
structural/device, TE-5, user and registry/provenance evidence. The review
snapshot predates only the mechanical final-disposition/router/memory closure
that records D-018 success; no normative phase rule was changed after its
reviewed hashes.

## 3. Coefficient sweep and locked values

The sweep used the existing capacities/conductances and a deliberately small
bounded target model. It did not fit values to a production run.

### 3.1 Fusion energy

Target: one +25°C Heat input at the 0°C Ice plateau MUST NOT complete melting;
two such inputs MAY complete it.

| Lf | Result | Disposition |
|---:|---|---|
| 40 | one pulse supplies 50 energy and completes | reject: too small |
| **80** | one pulse partial; two pulses can complete | **locked** |
| 120 | two pulses supply only 100 | reject: target misses |
| 160 | two pulses supply only 100 | reject: target misses |

### 3.2 Vaporization energy

Target model: one orthogonal fixed 300°C inert Stone face heats open
gas-facing Water initially at 20°C using existing `0.12` step and
Stone/Water conductance. First Steam target is 45–65 ticks. A single +25°C
Water Heat input supplies 62.5 energy and MUST NOT complete boiling.

| Lv | First Steam tick | Disposition |
|---:|---:|---|
| 240 | 34 | reject: earlier than 45 |
| 360 | 44 | reject: one tick earlier than target |
| **480** | **54** | **locked** |
| 720 | 74 | reject: later than 65 |

The 360/480 boundary is sensitive to the selected target window. D-018 locks
480 as a gameplay choice, not a claim that it is physically correct.

### 3.3 Surface sink

Grid:

```text
CONDENSATION_SURFACE_MAX_C = {70, 80, 90}
CONDENSATION_MIN_DELTA_C   = {5, 10, 20}
```

Targets:

- Steam 94°C / lid 80°C must qualify;
- Steam 86°C / neighbour 80°C (6°C delta) must not;
- Steam 94°C / neighbour 82°C must not because the sink is above 80°C.

Only `(80°C, 10°C)` passes all three. Max 70 rejects the cold lid; delta 5
admits insufficient separation; delta 20 rejects the 14°C cold-lid case; max
90 admits the above-80°C case when its delta passes.

### 3.4 Free-air threshold and route timing

Target model: one Steam/Air face cools toward 20°C using the existing
Matter/Air interface conductance. Nucleation onset target is 50–80 ticks and
completion target is 900–1300 ticks.

| Free-air max | Onset | Completion | Disposition |
|---:|---:|---:|---|
| 60°C | 93 | 1026 | reject: onset too late |
| **70°C** | **63** | **1013** | **locked** |
| 80°C | 39 | 1006 | reject: onset too early |

With the selected constants, the one-face 20°C cold-surface model begins at
tick 5 and completes at tick 501, faster than the free-air route.

Sensitivity boundary:

- these tick values are pure one-Cell envelopes, not a promise about moving
  multi-Cell production scenes;
- extra thermal faces change the rate but not accounting;
- local-minimum seeds in generated multi-Cell shapes reached a maximum fraction
  `2/3` for a very small non-adjacent shape, so visual sparsity is not proven;
- seed initiation is not immediate Water conversion; sustained latent removal
  remains required until E=0.

## 4. Deterministic fixture definitions

All coordinates are inclusive. Unless stated otherwise, use a 128×128 world,
32×32 chunks, sealed outer Environment boundary, sleep disabled for the
reference run, fixed arbitration tick seed, and exact canonical staging in both
state halves. Future implementation may factor shared setup but MUST preserve
these predicates.

### TE3-F01 — Repeated closed cycle, no net quantity gain

- Stage one Water unit at 20°C/E=0 in a bounded isolated fixture.
- Drive Water → Steam → Water at least 100 times through the reference and GPU
  semantic paths.
- Assert phase-family Cell count remains exactly 1 after every leg.
- Assert no phase energy exists at EMPTY or non-phase cells.
- Assert final Water 20°C/E=0 within H tolerance.

Coverage: PH-INV-001, 002, 003, 004, 005, 008, 014, 016.

### TE3-F02 — Partial boiling reversal

- Gas-facing Water begins 100°C/E=0.
- Add exactly `Lv/3`, verify Water 100°C and `0 < E < Lv`.
- Remove the same energy with surface context absent.
- Verify E returns to 0 before Water cools below the plateau, no Steam identity
  appears, and H closes within tolerance.

Coverage: PH-INV-005, 006, 007, 010, 016, 017.

### TE3-F03 — Partial condensation reversal

- Stage canonical Steam at 90°C/E=Lv next to an 80°C eligible sink.
- Verify Steam enters `0 < E < Lv` at 100°C.
- Reheat by the exact removed latent amount after moving away from the sink.
- Verify Steam returns toward E=Lv and no Water appears before E=0.

Coverage: PH-INV-005, 007, 011, 016.

### TE3-F04 — Partial freezing/melting reversal

- Exercise Water 0°C/E=0 toward negative E and reverse to 0 without Ice.
- Exercise Ice 0°C/E=-Lf toward higher E and reverse to -Lf without Water.
- Cross each endpoint separately and assert no threshold ping-pong for 256
  no-input ticks.

Coverage: PH-INV-005, 007, 016, 018.

### TE3-F05 — Surface boiling versus buried Water

- Stage two Water Cells with identical 125°C/E=0 and H.
- Surface Cell at `(32,32)` has orthogonal EMPTY; buried Cell at `(96,96)` has
  four non-GAS occupied neighbours.
- One normalization makes only the surface Cell Water 100°C/E>0.
- Buried Water remains 125°C/E=0 with identical H.
- After exposing one buried face, its preserved H enters the same progress.

Coverage: PH-INV-005, 010, 016.

### TE3-F05B — Buried mid-progress reversal

- Initiate gas-facing Water to `0<E<Lv`, then replace every gas-facing
  neighbour with occupied non-GAS Matter.
- Continue one bounded heating leg and assert E increases while Material stays
  Water; apply the inverse cooling leg and assert E reverses before Water
  cools below 100°C.
- No burial event may erase E, create Steam, add Matter or generate pressure.

Coverage: PH-INV-005, 007, 010, 014, 016, 022.

### TE3-F05C — Buried vaporization-ready hold

- Continue buried positive-E Water until total H exceeds
  `C_water*100+Lv`.
- Require Water/E=Lv and `T=(H-Lv)/C_water > 100°C` for at least 256 no-context
  ticks; quantity and pressure remain exact.
- Cool it: sensible Water superheat reaches 100°C first, then E falls below
  Lv. H closes at every step.

Coverage: PH-INV-001, 004, 005, 007, 014, 016, 022.

### TE3-F05D — Completion after surface reopens

- Starting from F05C, expose one orthogonal EMPTY or GAS Matter face.
- Convert exactly 1:1 to Steam/E=Lv and compute Steam temperature from the same
  H; no expansion proposal, second Steam or blocked pressure is permitted.

Coverage: PH-INV-001, 002, 003, 005, 010, 014, 022.

### TE3-F05E — Completion through future TE-5 transaction placeholder

- Keep the F05C Cell buried and default the placeholder to rejected/false.
- Only an explicit accepted placeholder may complete Water→Steam; absent,
  stale or unrelated values must not bypass it.
- The fixture asserts the input/output H and identity contract only. It MUST
  NOT invent a pressure law, force, buffer or production pass.

Coverage: PH-INV-005, 015, 019, 022.

### TE3-F06 — Cold-lid condensation

- Steam 94°C/E=Lv at `(64,63)` sees non-GAS lid `(64,62)` at 80°C.
- Control Steam has equivalent Air cooling and no sink.
- Surface route begins no later than tick 5 in the pure envelope and completes
  in target 450–650; free-air completes in 900–1300.
- First Water identity must occur at the actual cold-lid region.
- Repeat with lid 82°C and with 6°C delta; neither may initiate by surface.

Coverage: PH-INV-005, 011, 012, 016.

### TE3-F06B — Zero-conductivity cold Boundary

- Stage canonical Steam beside a 20°C Boundary whose compiled conductivity is
  exactly zero; all other faces have no thermal work.
- Require no surface condensation initiation, latent progress, phase activity
  or stalled partial state from that Boundary.
- Identity, E and H remain unchanged except when another named real thermal
  face is added. Adding a positive-conductance cooling face must wake and use
  the exact shared TE-2 work predicate.

Coverage: PH-INV-005, 011, 017, 018, 021, 023.

### TE3-F07 — Free-air nucleation

- Stage canonical Steam below 70°C in deterministic 1×1, 2×2, 8×8, 31×17
  and chunk-seam-crossing regions, plus one-Cell-thin and diagonal shapes.
- CPU/GPU keys use `(hash32,y,x)` and the exact radius-2 5×5 Chebyshev set.
- Connectivity means the graph induced by eligible Cells whose Chebyshev
  distance is at most two. Each finite component has at least one seed; no two
  same-tick seeds have Chebyshev distance at most two; a solid W×H region has
  at most `ceil(W/3)*ceil(H/3)` seeds.
- Force equal primary hashes and verify `(y,x)` tie-break leaves a seed.
- Initiation changes E but not identity in one tick; a multi-Cell region MUST
  NOT become all Water in the initiation tick.
- Advance at least two ticks after initiation. Every thermally runnable partial
  Steam Cell vetoes every radius-2 new free-air seed even though
  normalization put it at the 100°C plateau; no next-tick eligibility cascade
  is permitted.
- Move a partial seed through legal one-Cell GAS movement and assert that E and
  its active radius-2 veto follow the Matter owner. Completion or Void
  exit may release the veto; disappearance merely from the cold canonical set
  may not.
- Remove every thermal-work face from a partial seed: E is retained, the Cell
  may sleep, and it no longer vetoes a neighbour that has its own valid
  energy-removal face. Restoring a face wakes and resumes owned progress.
- CPU/GPU agree on the canonical initiation, thermal-work and active-veto sets.
- Selected one-face envelope onset is 50–80 and completion 900–1300.
- The docs-only v2 sweep discloses radius 1/2/3 metrics. Radius 2 is normative;
  a failure blocks the design rather than selecting radius 3.

Coverage: PH-INV-011, 012, 013, 016, 017, 020, 023, 024.

### TE3-F07B — Thermally isolated supercooled Steam

- Stage both canonical and partial Steam below their condensation thresholds
  in Vacuum and, separately, behind only zero-conductivity faces.
- With no positive-conductance work face, identity and E remain finite and
  unchanged, phase activity is zero and equilibrium may sleep indefinitely.
- Restoring one real cooling face wakes the halo and resumes sink or free-air
  eligibility using the shared TE-2 predicate.

Coverage: PH-INV-004, 005, 011, 016, 017, 018, 021, 023.

### TE3-F08 — No mid-air phase traffic jam

Preselect this geometry and thresholds before implementation:

- world 128×128; vessel interior `x=40..87, y=72..111`;
- named mid-air region `x=24..103, y=8..71`;
- heater is removed at tick 600; settle horizon ends at tick 3000;
- record phase-family quantity, airborne Water count, Water↔Steam orthogonal
  contact-edge count and largest alternating Water/Steam connected component
  every 30 ticks.

Required result:

- total phase-family quantity is exact throughout;
- vessel Water at tick 3000 exceeds vessel Water at tick 600;
- airborne Water at tick 3000 is no more than
  `max(8, floor(0.25 * peak_airborne_water_after_removal))`;
- Water↔Steam edge count at tick 3000 is no more than
  `max(8, floor(0.25 * peak_contact_edges_after_removal))`;
- largest alternating component in the mid-air region is at most 16 Cells at
  tick 3000;
- no two same-tick free-air initiations have Chebyshev distance at most two,
  and no new seed occurs within radius 2 of thermally runnable partial
  condensation; a stalled partial with no matching thermal work may coexist
  with a nearby Cell that has its own valid removal face;
- in every sampled 30-tick window, new free-air initiations are at most
  `max(4, ceil(peak eligible canonical Steam / 8))`;
- the edge-count peak occurs before the final 600-tick window and then
  declines under the 30-tick sampling cadence.

These values are pre-implementation product targets. A miss is evidence for
review, not permission to tune after observing the result.

Coverage: PH-INV-001, 002, 009, 012, 013, 018, 024.

### TE3-F09 — Open beaker causal chain

- Source: Stone heater rectangle `x=52..75, y=96..99`, staged at 300°C.
- Water inventory: vessel interior `x=48..79, y=80..95`; gas-facing top row is
  `y=80`; cold-wall regions are `x=44..47` and `x=80..83`, `y=32..79`, at
  20°C.
- Run bound: 3600 ticks, with first-event tick recorded for each named region.

Require one ordered trace with strict event ordering:

```text
heater Q input
< surface Water sensible rise
< positive boiling E at y=80
< 1:1 Steam identity
< Steam GAS movement into y<80
< Air/cold-wall cooling
< surface or free-air negative condensation progress
< 1:1 Water identity
< Water downward movement
< returned Water inside vessel y>=80
```

Observing both identities without the ordered source/tick/region chain fails.

Coverage: PH-INV-002, 005, 006, 009, 010, 011, 013.

### TE3-F10 — Sealed vessel accounting

- Stage 256 Water-equivalent units in a sealed occupied vessel.
- Heat/cool for 4096 ticks.
- Quantity remains 256; phase proposal/claim/spawn count is zero; Water-boil
  blocked-pressure delta is exact zero.
- Environment external Air exchange is exact zero.
- No assertion is made about sealed vapor force; it remains TE-5.

F10 is a disabled phase-contract fixture, not permission to ship that behavior.
The production/user-testable activation gate additionally requires a separately
authorized same-source TE-5 replacement to preserve the frozen G5 sequence:

```text
Water heat -> Steam expansion -> confinement Pressure -> rupture -> vent
```

Without that replacement the phase implementation may not become the active
Water path or be called a candidate. Historical G5 evidence stays source-bound
and is not a PASS for the replacement.

Coverage: PH-INV-001, 002, 003, 014, 015, 019.

### TE3-F11 — Exact staging/reset

Cover `GpuWorld::new`, direct `write_material`, Sandbox Draw/Erase,
Starter Lab, Blank World, reset, scenario upload, benchmark upload and bounded
test staging. Ice/Water/Steam write `-Lf/0/Lv`; all other/EMPTY write 0; Current
and Next are byte-identical. Heat/Cool changes only temperature. Sandbox uses a
separate pre-field phase edit dispatch with exactly five storage bindings; it
must not extend the seven-storage field edit pass to nine.

Coverage: PH-INV-008, 016.

### TE3-F12 — Movement and identity hygiene

- Move partial and canonical Ice/Water/Steam into EMPTY and through each legal
  density swap; compare ownership edges exactly.
- Void exit zeros the vacated in-domain state.
- Phase-family→non-phase/EMPTY through decay, consumption, rupture and Erase
  zeros E.
- Non-phase→phase staging uses the target canonical value.
- Run each identity hygiene pipeline use and assert no orphan E.

Coverage: PH-INV-008, 009, 016.

### TE3-F13 — Sleep/wake equivalence

- Run sleep disabled and enabled from identical staged bytes for 4096 ticks.
- Include active partial progress, a stalled partial plateau, eligible
  nucleation across a chunk seam, completion, and later equilibrium.
- Material and phase-energy bytes must match; temperatures use H tolerance.
- Changing progress remains runnable; stalled/completed equilibrium reaches
  sleeping state; an edit or neighbour frontier wakes the safety halo.

Coverage: PH-INV-012, 017, 018.

### TE3-F14 — CPU/GPU semantic agreement

For every candidate in a generated table covering starts, exact equalities,
surfaces, sinks, seeds, partial ranges, endpoints and excess:

- eligibility and transition kind match;
- target identity and progress direction match;
- result ranges are valid;
- H error passes the same absolute/relative tolerance;
- coordinate hash and tie-break match exactly.

Coverage: PH-INV-004, 005, 007, 010, 011, 012, 016, 017.

### TE3-F15 — Existing TE-2 regression

On the future runtime source, rerun only the affected TE-2 suites for Air
transport, unified thermal exchange, sealed/reservoir accounting, 30-case
small-delta convergence and Inspector payload/cadence. Historical TE-2 evidence
at `fb7e568...`/`0977281...` remains preserved but is not rebound to the new
runtime source.

Coverage: PH-INV-006 and no TE-2 regression.

## 5. Future structural guards

Before a runtime candidate can exist, targeted static/Naga tests MUST prove:

- exactly two new full-world f32 buffers and zero new full-world scratch;
- phase context has exactly seven storage bindings, reads real Air mass/energy,
  reuses the exact existing 128-byte TE-2 thermal-table uniform/predicate, and
  fully overwrites claim only after TE-2's claim lifetime is dead;
- the context, normalization and activity predicates agree that a surface sink
  has positive conductance and actual TE-2 energy-removal work; a K=0 Boundary
  cannot set sink or phase-activity bits;
- phase thermodynamics has exactly eight storage bindings, reads immutable
  claim/context plus the existing TE-2 thermal-table uniform, and has no direct
  Air, chunk-state or activity binding;
- movement reconcile and identity hygiene stay below 8; phase activity has
  exactly six RO plus one RW storage bindings, ordered Material Current,
  temperature Current, phase energy Current, Air mass Current, Air energy
  Current, chunk state and activity proposal RW, plus the same TE-2 thermal
  table needed to match the context thermal-work predicate;
- proposal is fully overwritten by phase thermodynamics after the TE-2 float
  window and before expansion readers;
- expansion claim fully overwrites claim only after phase thermodynamics has
  consumed every context marker;
- every Ice/Water/Steam proposal is `NO_PROPOSAL`, yield is 1 and blocked
  pressure is 0;
- a synthetic non-family yield-2 descriptor still emits one valid proposal and
  exercises the historical expansion consumer path while targeting non-phase
  Matter;
- every generic non-family `matter_yield>1` descriptor targeting Ice, Water or
  Steam is rejected unless a separately approved destination phase-energy
  ownership/writer contract and fixture are present;
- Water→Steam completion checks a current gas-facing bit; the future TE-5
  accepted-transaction bit is false/unbound in the phase-only graph and cannot
  be synthesized from burial, E=Lv, pressure metadata or stale scratch;
- seed and active-partial-veto loops use the same compile-time Chebyshev radius
  2 and global coordinates across chunk seams;
- the coordinate key exactly matches the internal `edge_priority` arithmetic
  and constants from the named production shaders; the new x/y/tag input
  mapping is pinned without creating a runtime helper;
- Sandbox phase editing is a separate five-storage dispatch and all accepted
  Draw/Erase paths update both halves without exceeding the field-pass ceiling;
- pass names/order are exactly the approved projection or an explicitly
  reviewed replacement;
- profiler arrays cover exactly 40 passes / 80 queries / two 640-byte buffers,
  and grouped summaries cover every pass once;
- phase-energy copies occur at every named joint settle;
- all staging/edit/reset writers touch both halves canonically;
- base activity removes the old threshold-only phase rule and phase activity
  uses the normalization work predicate;
- no Rust string scanner substitutes for Naga parsing;
- no external code or formula enters production;
- production activation of Water yield 1 is impossible unless a separately
  approved same-source pressure-volume replacement and its G5 causal fixture
  are present; disabled phase staging alone is never labelled a candidate;
- historical G5/TE-2 evidence identifiers remain attached to their original
  sources and are not rebound to the atomic activation source.

The amendments add no phase-only buffer, pass, timestamp query or binding.
The projection remains exactly two phase-energy buffers, zero new full-world
scratch, 40 passes, 80 queries and the 7/8/6/7 storage ceilings above. A future
TE-5 bridge may change the atomic graph only through its own separately
approved design and evidence; this contract does not pre-authorize that delta.

## 6. Future validation sequence

After a separate implementation authorization only (the ADR-0006
reference/review gate is now closed):

1. targeted Core normalization/quantity/property tests;
2. Naga parse and structural writer/binding/pass-order tests;
3. targeted GPU F01–F14 fixtures on the smallest worlds;
4. F15 affected TE-2 regression;
5. validation-plan and its required scope;
6. one allocation/profiler measurement only if the disabled phase source is
   final; stop as verified-but-inactive, not a candidate;
7. after separate TE-5 authorization, place the accounted pressure-volume
   replacement on the same source and pass the frozen G5 causal fixture;
8. atomically activate both paths, then run one bounded product candidate launch
   only when user review is requested;
9. user product disposition without rebinding earlier evidence.

No broad smoke matrix is required by default. The smallest affected-path test
is preferred until the validation planner establishes broader risk.

## 7. This docs/design task validation boundary

Required here:

- Wiki sync or recorded safe remote fallback;
- reference proof exactly once;
- Markdown link/fence/index checks;
- result JSON parse;
- docs policy audit and `git diff --check`.

Explicit counts for this task:

```text
Cargo test:             0
Cargo check:            0
Cargo clippy:           0
GPU test/run:           0
workspace FULL:         0
release build:          0
application launch:     0
TE-3 candidate run:     0
G8/G8-C run:            0
```
