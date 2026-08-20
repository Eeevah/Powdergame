# TE-3D Phase-Enthalpy Independent Adversarial Review

- **Review role:** fresh-context independent reviewer; not the design author
- **Review date:** 2026-08-20
- **Final reviewed snapshot:** 2026-08-20T23:23:02.5070215+09:00
- **Runtime/build status:** no Rust, WGSL, Cargo, build, launch, GPU, or test execution; no runtime source changed
- **Allowed write used:** this report only
- **Final disposition:** **INDEPENDENT DESIGN REVIEW PASS / USER ARCHITECTURE REVIEW PENDING**

## 1. Method and scope

This review attacked the proposed Hybrid A+C design as a paper architecture,
not as an implementation. I independently:

1. read the repository instructions, opted-in Ballast state and D-017
   authorization before reviewing the design;
2. read the Wiki entry points and relevant Ballast workflow page read-only;
3. traced the current production phase, movement, Air/thermal, expansion,
   activity, staging and editor paths directly in source;
4. recomputed the enthalpy endpoint equalities, pass/query/profiler arithmetic,
   storage-binding ceilings and proposal/claim live ranges;
5. constructed counterexamples for quantity expansion, TE-2 Q accounting,
   capacity changes, partial-state ownership, authoring/reset, buried progress,
   ungated supercooling, temporal nucleation, chunk seams, sleep/wake and
   generic yield greater than one;
6. inspected the one-shot reference script and JSON as already-existing
   artifacts without rerunning them; and
7. preserved findings discovered during review even when the design author
   amended the primary documents and fully resolved them.

This report can pass the design while leaving Medium or Low product/clarity
risks OPEN. It cannot accept ADR-0006 for the user, authorize implementation,
mark the TE-3D checklist complete, activate a phase-only runtime, design TE-5,
or rebind historical evidence.

The Wiki was not modified. No other repository file was modified by this
reviewer.

## 2. Exact review baseline

### 2.1 Worktree and source identity

- Branch: feature/m0-g9-first-playable, one commit ahead of its upstream.
- HEAD: fd97e8b89f277e1205c8b5bcd970002bfd87e7c4
  (docs: accept TE-2 with known follow-ups).
- Worktree: dirty in documentation/memory and with three untracked TE-3D
  primary documents before this report was created.
- Audited production-physics source:
  fb7e568e21012b6067269f4e1b82c36c865023d0.
- TE-2 review-remediation source:
  097728128343cf89383920c968a010b3dcf8e8c0.
- TE-3 blocker/design-start source:
  94b152e85ff6f5481a033d885d38dca0dbc1043a.
- A read-only git comparison found no difference from fb7e568... and no
  worktree difference from HEAD in the reviewed source set:
  engine/core/src, engine/gpu/src, apps/windows/src/sandbox.rs,
  apps/scenarios/src, Cargo.toml and Cargo.lock.

The directly reviewed current source set was:

- engine/core/src/material.rs
- engine/core/src/phase.rs
- engine/core/src/thermal.rs
- engine/gpu/src/simulation.rs
- engine/gpu/src/world.rs
- engine/gpu/src/phase_transition.wgsl
- engine/gpu/src/expansion_claim.wgsl
- engine/gpu/src/expansion_spawn_commit.wgsl
- engine/gpu/src/expansion_pressure.wgsl
- engine/gpu/src/environment_blocked_expansion_pressure.wgsl
- engine/gpu/src/movement_commit.wgsl
- engine/gpu/src/environment_reconcile_movement.wgsl
- engine/gpu/src/thermal_stability_scale.wgsl
- engine/gpu/src/unified_thermal_commit.wgsl
- engine/gpu/src/activity_propose.wgsl
- engine/gpu/src/environment_activity_propose.wgsl
- engine/gpu/src/activity_wake.wgsl
- engine/gpu/src/activity_reduce.wgsl
- apps/windows/src/sandbox.rs
- apps/scenarios/src/stage.rs

Current source still defines Water boil yield 2 and blocked pressure 100
(engine/core/src/phase.rs:30-38 and engine/core/src/material.rs:200-224).
The current phase shader writes proposal for yield greater than one
(engine/gpu/src/phase_transition.wgsl:120-159), and the current 34-pass order is
visible in engine/gpu/src/simulation.rs:2640-3264. Those facts were used as
the source anchor; they were not treated as a TE-3 implementation.

### 2.2 Primary and supporting repository documents

The mutable design snapshot is fixed by these SHA-256 values:

| SHA-256 | Exact reviewed file |
|---|---|
| 959ba983f9482e57d49a6dd099cdff13ca087b36b8092e182b032f138c534611 | docs/planning/TE3_WATER_STEAM_PHASE_ACCOUNTING.md |
| 1b9da3449686f619388569e04bc791b3f1e9337ee9ba48fb6cf50a50865fd309 | docs/architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md |
| 147d67a691a60abca9a501c78031a4cbc81509ca6a4c1dceb4c89f7176ec1d5f | docs/specs/PHASE_THERMODYNAMICS_SPEC.md |
| 5895b673570d24c46f24e334b9dd5185f313fbfc88ae63e06130526166e20715 | docs/development/PHASE_THERMODYNAMICS_VALIDATION.md |
| cf670bccf1cd341cb6cde799251d8f2ae7caa21e527ffb6fd2393546c232b769 | docs/architecture/THERMAL_ENVIRONMENT_PRODUCTION_INVENTORY.md |
| fbb1697a1e87bb631c8741a69bfbb99fa1720e1cfd99725c859f6dbf260bef16 | docs/planning/THERMAL_ENVIRONMENT_IMPLEMENTATION_GATES.md |
| 164c79d5442cad2b25fe5a012daf85010d212bd49ac5137bad006e0cba9bd04e | docs/planning/MILESTONES.md |
| de74655b1d25ee4c479567c32991d21bd19e466d5eab5bf1d19abdbb5e666caa | docs/planning/STATUS.md |
| 655338d9b59c59592bfed7498d4ce4a7fd46e6fe8dd6530e4f9ad0bc51f4ef61 | docs/planning/THERMAL_TRANSPORT_IGNITION_CAUSALITY.md |
| ced98de479ded690c1910e7b505ffc9262375024ce91d38b4fe44a52d9a40c78 | docs/development/VALIDATION_POLICY.md |
| ddaac892c3723a107b5212d64c359e4faabc43d41cb8ac7a6a352cb779438445 | docs/research/2026-08-20-thermal-environment-reuse-survey.md |
| 2638582ab80fcd57bd7bd8a1c37253543b6e88de504117e777df84e8e415d5bf | docs/adversarial-reviews/README.md |
| b93c1eae8edc0b36bbfbacb2cec55d47d887bb133607cfcd7a7c32c76c28b3ff | AGENTS.md |
| b9cd2e2e8cf484577a4727afb7ef61aaa1f5020b3fad50d498c74bd06fca7840 | memory/00-INDEX.md |
| fb6ee8568b907f6ff6ae97cba56e31a6bb49dd334b373ded61a0efdbd1b1a016 | memory/CHECKPOINT.md |
| 783398fe6eaf8bd7e2b698217c61a1b5ccf7bed683eb7ff2507a35f7f96b7ae9 | memory/DECISIONS.md |

### 2.3 Read-only external guidance and proof artifacts

| SHA-256 | Exact reviewed file |
|---|---|
| 38cd11ef1fd663bd9b3b6deb079dfac7acfd7ed0b7f282d0146dc2b775cbc745 | C:\Users\mdkap\Knowledge\personal-infra-wiki\wiki\index.md |
| 58b7c8773b50ed06a168b64e5028d5fe3932bdcf129ac5a42dccc2a6052aa5f0 | C:\Users\mdkap\Knowledge\personal-infra-wiki\wiki\projects\index.md |
| 198d7e7c33f72836312c6791b3b1154fe93900f7303b7ebe08596a6a78d71d7c | C:\Users\mdkap\Knowledge\personal-infra-wiki\wiki\workflows\codex-ballast-memory.md |
| ea7c1614125d437afab7a2308f36ecb91111619fda5b1bbc3bd7ecc899f7221a | C:\Users\mdkap\.agents\skills\ballast-recall\SKILL.md |
| 117439a84f1debdc4e4cca6007a4307903bc643cb1811f8c0d979dfecda05561 | C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te3_phase_enthalpy_reference.py |
| 6c1afe9f3734be51301562ee3363a94726a75c1f64c222c3dc824ed31d19e42e | C:\Users\mdkap\.codex\visualizations\2026\08\20\01a01f0c-f992-74f0-a89d-f2ff2792ada8\te3_phase_enthalpy_reference_result.json |

The Wiki project index did not contain a dedicated Powdergame project page.
The reference artifacts were inspected, not executed.

## 3. Findings

### TE3D-001 — One-cell vapor cannot preserve the frozen G5 chain by itself

- **Severity:** High
- **Claim:** A 1:1 Water-to-Steam foreground mapping fixes quantity duplication,
  but GAS dispersion alone is not a substitute for the already-frozen
  boil/confinement/Pressure/rupture/vent product chain.
- **Counterexample / attack:** In a sealed occupied vessel, Water can become one
  Steam Cell with no extra receiver and exactly zero blocked-expansion pressure.
  If that path replaced the current Water rule before a pressure-volume
  replacement existed, the representative G5 chain in
  docs/planning/MILESTONES.md:220-253 would regress even though foreground
  quantity became correct.
- **Violated or defended invariant:** PH-INV-001/002 quantity conservation was
  defended, but frozen G5 continuity was initially violated. The final design
  adds PH-INV-019 and makes the two obligations coexist.
- **Required resolution:** Completed. ADR-0006:211-239, the specification at
  lines 31-60 and 566, validation F10 and its structural guards, and the gate
  sequence at THERMAL_ENVIRONMENT_IMPLEMENTATION_GATES.md:144-169 now require
  Water yield 1 to remain disabled/non-production until a separately authorized
  same-source TE-5 replacement passes the G5 causal fixture. The TE-3
  implementation stop is verified-but-inactive; the current G5 Water path stays
  active.
- **Residual risk:** The replacement pressure-volume law is intentionally not
  designed here. No Water/Steam plus Pressure user candidate exists until that
  separate work is authorized, implemented and validated.
- **Status:** RESOLVED

### TE3D-002 — The original maxed phase pass could not observe real Air work

- **Severity:** High
- **Claim:** The free-air seed and partial-veto predicates require real TE-2
  thermal work. A phase pass bound only to Matter/T/E/chunk state cannot
  distinguish Atmosphere from Vacuum or evaluate Air temperature.
- **Counterexample / attack:** Atmosphere and Vacuum are both Material EMPTY,
  while EMPTY Matter temperature is the reference value. The actual Air node is
  determined by air_mass_current and air_energy_current
  (engine/core/src/thermal.rs:121-142 and
  engine/gpu/src/environment_activity_propose.wgsl:50-82). Without those
  buffers, cold Steam next to Vacuum can be misclassified as having an
  energy-removal face, or a real Air face can be missed. Adding both buffers to
  the already-eight-storage phase pass would exceed the DX12 ceiling.
- **Violated or defended invariant:** PH-INV-017 matching work, PH-INV-020
  context snapshot, TE-1 Atmosphere/Vacuum ontology, and the eight-storage
  ceiling.
- **Required resolution:** Completed. The final 40-pass design inserts
  phase_context_propose after TE-2 scratch use. It binds M/T/E, Air mass/energy,
  chunk state and claim RW (6 RO + 1 RW), reuses the existing 128-byte TE-2
  conductivity/capacity uniform and fully writes immutable context markers.
  phase_thermodynamics then uses context as its fourth RO binding and remains
  exactly 4 RO + 4 RW. See PHASE_THERMODYNAMICS_SPEC.md:405-484 and the
  structural guards at PHASE_THERMODYNAMICS_VALIDATION.md:393-428.
- **Residual risk:** This remains a design projection. Naga, binding-layout,
  full-write and CPU/GPU predicate evidence must be produced on the eventual
  implementation source.
- **Status:** RESOLVED

### TE3D-003 — Static local minima alone permit a next-tick seed cascade

- **Severity:** High
- **Claim:** A same-snapshot local-minimum proof does not bound temporal
  nucleation once the first seed leaves the below-70 canonical set.
- **Counterexample / attack:** A seed normalizes to a 100-degree partial Steam
  plateau. Without a partial-progress veto it is no longer a cold canonical
  competitor on the next tick, so an adjacent canonical Steam Cell can become
  the next local minimum. Repeating that sequence can walk a seed through a
  cloud one tick at a time despite passing every static no-adjacent-seed test.
- **Violated or defended invariant:** PH-INV-012 bounded nucleation and
  PH-INV-013 no persistent traffic jam.
- **Required resolution:** Completed. The specification at lines 312-373 now
  makes thermally runnable partial Steam an eight-neighbour veto, attaches the
  veto to Matter-owned E through movement, releases a stalled no-work partial,
  and explicitly rejects static sparsity as temporal proof. F07
  (PHASE_THERMODYNAMICS_VALIDATION.md:225-250) covers two or more ticks,
  movement, completion/Void release, stalled progress and CPU/GPU context sets.
  F08 lines 252-284 now uses the same active-partial qualifier.
- **Residual risk:** The product-scale temporal rate and visible texture remain
  open in TE3D-009; this finding resolves the concrete next-tick correctness
  hole, not future visual acceptance.
- **Status:** RESOLVED

### TE3D-004 — Independent-review evidence was marked complete before it existed

- **Severity:** High
- **Claim:** Marking the independent-review checklist item complete while this
  review was still in progress would be an evidence-integrity failure.
- **Counterexample / attack:** A design could appear to have zero independent
  blockers even though no independent report or disposition existed, allowing
  the design gate to consume its own unverified claim.
- **Violated or defended invariant:** D-017's independent-review condition and
  the archive rule that missing evidence must not be converted into PASS.
- **Required resolution:** Completed during review. The final snapshot keeps
  the item unchecked at
  THERMAL_ENVIRONMENT_IMPLEMENTATION_GATES.md:122-135. This report does not
  itself authorize the design author to close any later gate.
- **Residual risk:** Any later checklist update must point to this exact review
  snapshot and must not imply user acceptance of ADR-0006.
- **Status:** RESOLVED

### TE3D-005 — TE-2 Q double counting and capacity discontinuity

- **Severity:** Medium
- **Claim:** Adding latent state after TE-2 could accidentally debit a neighbour
  twice, and different Ice/Water/Steam heat capacities could create an
  enthalpy jump at identity change.
- **Counterexample / attack:** If phase normalization emitted a second
  neighbour heat write after TE-2 already transferred Q, pair energy would not
  close. If Steam sensible energy were simply C_steam times temperature, Water
  at 100/E=Lv and Steam at 100/E=Lv would represent different H.
- **Violated or defended invariant:** PH-INV-005 H preservation and PH-INV-006 Q
  exactly once.
- **Required resolution:** Already present and sufficient. ADR-0006:112-140 and
  PHASE_THERMODYNAMICS_SPEC.md:133-187 anchor Steam sensible energy to Water's
  0-to-100 rise, show equal endpoint H values, and restrict normalization to a
  local repartition of the already-settled TE-2 result. The heat-capacity
  derivative changes at an endpoint, but H does not jump.
- **Residual risk:** Only the future f32 implementation and repeated-cycle
  fixtures can establish that the formulas were encoded without branch or
  rounding mistakes.
- **Status:** RESOLVED

### TE3D-006 — Partial state can be orphaned by movement or external writers

- **Severity:** Medium
- **Claim:** A spatial E buffer would corrupt identity if movement, density
  swap, Void exit, replacement, Draw, Erase or reset updated Material without
  the matching E ownership edge.
- **Counterexample / attack:** Moving partial Water while leaving E at the
  source creates canonical-looking Water with missing progress at the
  destination and latent energy attached to EMPTY at the source. A rejected
  EMPTY-only Draw must also not overwrite the occupied Cell's E.
- **Violated or defended invariant:** PH-INV-007/008/009/016 and the
  Current/Next authoring contract.
- **Required resolution:** The final design is sufficient at design level.
  PHASE_THERMODYNAMICS_SPEC.md:375-403 enumerates stay, move, density swap, Void,
  identity replacement, Draw, Erase, Heat/Cool and reset. Sandbox phase editing
  is a separate pre-field 3-RO/2-RW dispatch rather than turning the existing
  seven-storage field pass into a nine-storage pass. F11/F12 and structural
  guards cover all named staging and hygiene paths.
- **Residual risk:** No WGSL writer, editor dispatch or reset implementation
  exists. One bypass writer on the future source reopens this finding.
- **Status:** RESOLVED

### TE3D-007 — Scratch reuse and generic yield greater than one can collide

- **Severity:** Medium
- **Claim:** TE-2 reinterprets proposal and claim as f32 scratch, while phase
  and expansion need them as u32 ownership buffers. A partial overwrite or
  wrong pass order can feed float bits to an expansion consumer or erase the
  generic yield path.
- **Counterexample / attack:** If context overwrites claim before Air transport
  consumes receiver scale, TE-2 is corrupted. If expansion claim overwrites
  context before phase consumes it, phase eligibility races. If phase writes
  NO_PROPOSAL only for family Cells and leaves other slots stale, a historical
  generic transition can consume arbitrary TE-2 bits.
- **Violated or defended invariant:** PH-INV-003, PH-INV-014, PH-INV-020, scratch
  full-write ownership and generic expansion compatibility.
- **Required resolution:** Completed in the final projection. The ordered live
  range is movement u32 -> TE-2 f32 -> context claim u32 -> phase consume ->
  expansion-claim u32, while proposal is fully rewritten by phase after the
  thermal-lambda consumer. Every Cell is written, family descriptors emit
  NO_PROPOSAL, and a synthetic non-family yield-2 fixture must reach the
  historical consumer. See PHASE_THERMODYNAMICS_SPEC.md:405-507 and
  THERMAL_ENVIRONMENT_PRODUCTION_INVENTORY.md:270-340.
- **Residual risk:** TE3D-014 identifies one future target-family edge not
  material to the current registry. Actual full-write and live-range proof
  remains a structural implementation gate.
- **Status:** RESOLVED

### TE3D-008 — Partial progress can make the world permanently awake or sleep through work

- **Severity:** Medium
- **Claim:** Marking every interior partial E active forever defeats G7 sleep;
  allowing every partial state to sleep can suppress valid progress and
  temporal vetoes.
- **Counterexample / attack:** A partial plateau with no positive-conductance
  thermal face never changes and should sleep. The same partial Cell next to a
  valid removal face must remain runnable, and movement/edit/frontier changes
  must wake it.
- **Violated or defended invariant:** PH-INV-012/017/018 and sleep-on/off
  semantic equivalence.
- **Required resolution:** The final specification at lines 457-461 and
  509-530 gives phase_activity_propose the exact M/T/E/Air/chunk/activity
  storage order, the same phase and existing TE-2 thermal uniforms, and a fresh
  predicate independent of the earlier context snapshot. Stalled progress may
  sleep; runnable work remains active. F07 and F13 cover stall, restore, halo
  wake and sleep-on/off equivalence.
- **Residual risk:** The existing safety halo is relied on but not executed in
  this docs-only review. Any implementation that uses chunk state to skip the
  final activity detector would reopen the deadlock.
- **Status:** RESOLVED

### TE3D-009 — Temporal/lattice appearance has no numeric initiation-rate acceptance bound

- **Severity:** Medium
- **Claim:** Hash ties and chunk seams are specified, but deterministic local
  minima do not by themselves establish acceptable cloud-scale appearance.
- **Counterexample / attack:** The reference generator observed a seed fraction
  as high as 2/3 in a small non-adjacent shape. A fixed coordinate hash can make
  stable lattice texture visible, and post-completion fronts can produce new
  initiation patterns over time. F08 records the maximum new initiations in a
  30-tick window but does not give that metric a pass/fail ceiling.
- **Violated or defended invariant:** PH-INV-012/013 are structurally defended;
  the unresolved part is their product-visible temporal interpretation.
- **Required resolution:** Before a user-testable Water/Steam plus Pressure
  candidate, the user must either accept the recorded temporal/visual pattern
  or approve a predeclared numeric initiation-rate/texture bound. Do not choose
  the bound after viewing a production result.
- **Residual risk:** Passing the single 128x128 F08 geometry may not generalize
  to thin, diagonal, moving or much larger clouds.
- **Status:** OPEN

### TE3D-010 — Already-started boiling may complete after the surface disappears

- **Severity:** Medium
- **Claim:** The design surface-gates boiling initiation, not completion.
  Already-owned positive Water E continues or reverses with energy flow after
  burial.
- **Counterexample / attack:** Start gas-facing Water, accumulate 0 < E < Lv,
  close the gas face through movement or editing, then continue heating through
  occupied neighbours. ADR-0006:148-149 permits E to reach Lv and create Steam
  while buried. F02 tests reversal after context loss and F05 tests canonical
  buried Water, but neither tests buried continuation through completion.
- **Violated or defended invariant:** PH-INV-007 is defended; PH-INV-010's
  initiation wording is obeyed. The open question is whether that behavior is
  the intended product meaning of surface boiling and how the later atomic
  pressure path handles the buried completion.
- **Required resolution:** Add a named buried-mid-progress completion/reversal
  fixture and obtain explicit user disposition. If completion must also be
  surface-gated, revise the normalization rule without discarding E.
- **Residual risk:** The future TE-5 law may make the same case acceptable by
  producing confinement response, but that law is outside this design and
  cannot be assumed.
- **Status:** OPEN

### TE3D-011 — Thermally isolated supercooled Steam may never nucleate

- **Severity:** Medium
- **Claim:** Canonical Steam below the temperature threshold still requires a
  valid sink or a free-air seed with a real energy-removal face.
- **Counterexample / attack:** Canonical Steam staged below 70 degrees in
  Vacuum, or surrounded only by zero-conductance faces, can preserve a large
  sensible supercooling deficit but has no currently removable-energy face. It
  remains Steam indefinitely and may sleep. This is explicitly allowed by
  ADR-0006:154-155 and PHASE_THERMODYNAMICS_SPEC.md:268-277.
- **Violated or defended invariant:** No accounting invariant is violated; the
  risk is the product interpretation of PH-INV-011/012 and metastable
  no-nucleation behavior.
- **Required resolution:** User architecture review must explicitly accept this
  no-sink/no-work metastability or request a separately bounded spontaneous
  nucleation rule. Include an isolated-supercooled control in future product
  observation.
- **Residual risk:** A spontaneous rule would reintroduce temporal-cascade and
  permanent-activity risks and therefore requires a new adversarial pass.
- **Status:** OPEN

### TE3D-012 — A zero-conductivity Boundary qualifies as a cold surface sink

- **Severity:** Medium
- **Claim:** The surface-sink predicate accepts any non-EMPTY, non-GAS Matter
  satisfying the temperature tests, while current Boundary conductivity is
  exactly zero.
- **Counterexample / attack:** A 20-degree Boundary next to 94-degree canonical
  Steam satisfies PHASE_THERMODYNAMICS_SPEC.md:294-310, even though
  engine/core/src/material.rs:61 makes the TE-2 conductance zero. Stored
  supercooling can start partial condensation, after which no heat crosses that
  face and progress stalls. F06's generic cold lid does not explicitly pin the
  zero-conductivity Boundary case.
- **Violated or defended invariant:** H accounting and sleep are defensible,
  but the phrase cold-surface sink may imply an energy sink that does not
  exist.
- **Required resolution:** State explicitly whether a surface is only a
  nucleation substrate or must also have positive TE-2 conductance. Add a
  Boundary-K=0 fixture; if real removal is required, include the shared
  conductance predicate in surface eligibility.
- **Residual risk:** Tightening the predicate changes wall-condensation
  appearance and must be judged with the future product fixture.
- **Status:** OPEN

### TE3D-013 — Coefficient selection is sensitive to self-declared timing windows

- **Severity:** Medium
- **Claim:** The reference sweep is useful as a proposal generator, not as an
  independent fit or production validation.
- **Counterexample / attack:** Lv=360 is rejected at tick 44 only because the
  selected lower bound is 45, while Lv=480 is accepted at tick 54. The target
  window and sweep live in the same one-shot artifact set; there is no earlier
  immutable receipt proving the window was fixed independently.
- **Violated or defended invariant:** No conservation invariant is violated.
  Evidence-integrity risk is limited because
  PHASE_THERMODYNAMICS_VALIDATION.md:78-154 calls the values proposed gameplay
  constants, discloses the sensitive boundary and denies a production fit.
- **Required resolution:** Treat Lf/Lv and timing windows as explicit user
  architecture choices. Preserve the current targets before implementation;
  if any coefficient or window changes, create a new source-bound sweep receipt
  and do not relabel the old result.
- **Residual risk:** Multi-face and moving production scenes can differ
  materially from the one-cell envelope even when accounting remains correct.
- **Status:** OPEN

### TE3D-014 — Future generic yield-2 targets need an explicit phase-energy restriction

- **Severity:** Low
- **Claim:** The design preserves a synthetic non-family yield-2 expansion
  path, but does not explicitly say whether its target may be Ice, Water or
  Steam.
- **Counterexample / attack:** A future non-family descriptor that yield-2
  transitions into Steam can let expansion_spawn_commit place a second Steam
  identity at the destination. The destination invocation was EMPTY during the
  phase pass and therefore wrote E=0, while canonical Steam requires E=Lv.
  The phase-energy hygiene list has no post-expansion destination repair
  because the current family path is dormant.
- **Violated or defended invariant:** PH-INV-008/016 and the rule that every
  writer placing phase-family identity also writes canonical or owned E.
- **Required resolution:** Make the synthetic compatibility fixture target a
  non-family identity and normatively forbid a generic yield-greater-than-one
  target from entering the phase family without a new ownership/writer design,
  or add an accounted destination E write and fixture. Pin the packed rule
  header bit layout when implementation begins.
- **Residual risk:** The current registry contains no such non-family phase
  descriptor, so this is not a current design blocker; any new descriptor is
  already separately gated.
- **Status:** OPEN

### TE3D-015 — The pure reference proof does not prove movement or chunk scheduling

- **Severity:** Low
- **Claim:** The reference script's movement copy and chunk partition checks
  are insufficient to establish actual GPU ownership, halo or seam behavior.
- **Counterexample / attack:** Reassigning a Python tuple or reconstructing the
  same global coordinate set after partitioning cannot expose a WGSL binding,
  settle-order or sleeping-chunk seam bug.
- **Violated or defended invariant:** Evidence-layer integrity.
- **Required resolution:** Already satisfied at design level. The validation
  document at lines 53-76 limits the proof to math/static seed properties and
  explicitly lists movement, chunk scheduling, temporal veto, sleep, GPU and
  visual quality as not proved. F07, F12, F13 and structural guards assign those
  obligations to future implementation evidence.
- **Residual risk:** The future evidence must remain attached to its exact
  implementation source; the existing JSON can never satisfy those gates.
- **Status:** RESOLVED

### TE3D-016 — New mixer-constant provenance is asserted, not independently auditable

- **Severity:** Low
- **Claim:** The ADR states zero external implementation code/formulas were
  copied, but a negative provenance claim cannot be established by repository
  search alone.
- **Counterexample / attack:** The finalizer constants 0x7FEB352D and
  0x846CA68B already exist in internal arbitration shaders, but the coordinate
  multipliers 0x9E3779B1 and 0x85EBCA77 are new exact values in the reviewed
  design. The reviewed files contain no origin note for those two choices.
- **Violated or defended invariant:** The zero-external-copy boundary and
  future license/provenance hygiene; this is not a finding that copyright was
  infringed.
- **Required resolution:** Record whether the new constants were independently
  selected or derived from an internal project pattern. If any external source
  was consulted, record its license/provenance and avoid copying implementation
  text. Keep the future no-external-code structural guard.
- **Residual risk:** Small numeric constants and simple affine enthalpy
  equations do not by themselves establish copying, so the remaining risk is
  provenance clarity rather than a correctness blocker.
- **Status:** OPEN

### TE3D-017 — Historical evidence and TE-4/TE-5 scope could leak into the candidate

- **Severity:** Info
- **Claim:** A new runtime source cannot inherit TE-1/TE-2/G5 proof merely from
  ancestry, and an atomic dependency on TE-5 must not silently authorize or
  design TE-5 or TE-4.
- **Counterexample / attack:** Calling the future atomic source TE-2-passed
  without rerunning affected suites, or treating ADR-0006 acceptance as TE-5
  authorization, would overstate evidence and scope.
- **Violated or defended invariant:** Source-bound evidence, D-017 authorization
  and named-gate separation.
- **Required resolution:** The final documents are sufficient.
  PHASE_THERMODYNAMICS_VALIDATION.md:381-391 requires affected TE-2 regression;
  lines 429 onward forbid historical evidence rebound and sequence separate
  TE-5 authorization. ADR-0006:218-229 specifies only the atomic dependency,
  not the TE-5 law. Ignition/TE-4 remains excluded.
- **Residual risk:** The future combined source must produce new source-bound
  TE-2 and G5 receipts; ancestry is not a substitute.
- **Status:** RESOLVED

### TE3D-018 — The old Water blocked-pressure route must be either active or impossible, never stale

- **Severity:** Info
- **Claim:** During disabled TE-3 staging, the current yield-2 G5 path must
  remain active. After atomic activation, Water-family proposal/claim/spawn and
  blocked pressure must be impossible while generic expansion remains valid.
- **Counterexample / attack:** A stale Water proposal left from TE-2 float bits
  or an incompletely updated descriptor could invoke the historical pressure
  path even when Water yield is nominally one. Conversely, disabling the
  current path before TE-5 would regress G5.
- **Violated or defended invariant:** PH-INV-003/014/019/020.
- **Required resolution:** The final design explicitly separates the two
  states. Before atomic activation, the current source remains unchanged. On
  the future phase path, context and phase fully overwrite claim/proposal,
  family descriptors are yield 1/pressure 0/NO_PROPOSAL, expansion claim fully
  overwrites context, and the synthetic non-family path is retained. F10 and
  structural guards test exact-zero Water pressure while the atomic gate
  prevents that disabled behavior from shipping alone.
- **Residual risk:** Only implementation/source-bound structural evidence can
  establish that no stale route survives.
- **Status:** RESOLVED

## 4. Severity and status summary

| Severity | Total | OPEN | RESOLVED | ACCEPTED RISK |
|---|---:|---:|---:|---:|
| Critical | 0 | 0 | 0 | 0 |
| High | 4 | 0 | 4 | 0 |
| Medium | 9 | 5 | 4 | 0 |
| Low | 3 | 2 | 1 | 0 |
| Info | 2 | 0 | 2 | 0 |
| **All** | **18** | **7** | **11** | **0** |

- **Unresolved Critical:** 0
- **Unresolved High:** 0
- **Most important unresolved lower-severity risks:** temporal/lattice product
  appearance without a numeric initiation-rate bound (TE3D-009), buried
  mid-progress completion (TE3D-010), no-work supercooled metastability
  (TE3D-011), and the zero-conductivity surface-sink interpretation
  (TE3D-012).

## 5. Verification performed and intentionally omitted

Performed:

- direct document/source inspection;
- read-only branch, commit, status and selected-source diff inspection;
- SHA-256 snapshot of every primary/supporting document and reference artifact;
- manual enthalpy endpoint/capacity/Q audit;
- manual pass, query, profiler-byte, storage-binding and allocation arithmetic;
- manual proposal/claim/context live-range audit against the current command
  graph; and
- manual review of the already-existing reference script and JSON.

Intentionally not performed:

- Cargo test/check/clippy, Rust tests or any other test runner;
- WGSL/Naga compilation or shader execution;
- GPU/runtime/scenario execution;
- build or launch;
- Python/reference-proof rerun;
- performance/allocation measurement;
- Sandbox or product visual review; and
- any edit to Rust, WGSL, Cargo, build, launch, test, Wiki, memory, planning,
  ADR, specification, validation or inventory files.

The omitted layers remain exactly where the validation contract places them.

## 6. Final disposition

All Critical/High attacks are either absent or explicitly resolved in the
final hashed design snapshot. The architecture is coherent enough to present
to the user for the named architecture decision. The seven OPEN Medium/Low
findings are real residual risks and must not be hidden, but none requires
rejecting the docs-only candidate before user review.

**INDEPENDENT DESIGN REVIEW PASS / USER ARCHITECTURE REVIEW PENDING**

ADR-0006 remains Proposed. TE-3 runtime remains not started. A future TE-3
implementation may reach only verified-but-inactive status until a separately
authorized TE-5 replacement is ready on the same source and the frozen G5 chain
passes. This report does not authorize implementation, activation, gate
closure, commit, push, merge, release or user acceptance.
