# Independent TE-5B Phase-Volume Pressure Bridge Design Review

- **Date:** 2026-08-21
- **Reviewer role:** fresh-context independent adversarial reviewer; not a
  primary-author participant
- **Requested output-name exception:** this non-date filename was explicitly
  required by the review task
- **Reviewed branch:** `feature/m0-g9-first-playable`
- **Source baseline:** `d7500e219af6f670be05f830b50c232d2bb53077`
- **D-019 authorization commit / HEAD:** `f1ca48cc01a906bfb4a997c72bc2744b81546ccd`
- **Current unresolved:** Critical **0**, High **1**, Medium **3**, Low **1**
- **Runtime/proof execution by this reviewer:** **0**
- **Verdict:** **TE-5B DESIGN BLOCKED**

## 1. Task and reviewed scope

This review attacks the D-019-authorized docs/reference design for the
Water-to-Steam phase-volume pressure bridge. It does not review an
implementation: the source baseline still contains the historical G5
`matter_yield = 2` Water path, and TE-3/TE-5B runtime is not started.

Primary authority:

- [ADR-0007](../architecture/decisions/ADR-0007-phase-volume-pressure-bridge.md)
- [bridge specification](../specs/PHASE_VOLUME_PRESSURE_BRIDGE_SPEC.md)
- [validation contract](../development/PHASE_VOLUME_PRESSURE_BRIDGE_VALIDATION.md)
- [TE-5B plan](../planning/TE5_PHASE_VOLUME_PRESSURE_BRIDGE.md)
- [production inventory](../architecture/THERMAL_ENVIRONMENT_PRODUCTION_INVENTORY.md)

Dependency and gate authority:

- [ADR-0006](../architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md)
- [phase thermodynamics specification](../specs/PHASE_THERMODYNAMICS_SPEC.md)
- [thermal implementation gates](../planning/THERMAL_ENVIRONMENT_IMPLEMENTATION_GATES.md)
- [MILESTONES](../planning/MILESTONES.md)
- [STATUS](../planning/STATUS.md)
- [G5 user validation](../planning/G5_USER_VALIDATION.md)
- [G5-B plan](../planning/G5_B_EXPANSION_CONFINEMENT.md)
- [G5-C plan](../planning/G5_C_RUPTURE_VENT.md)
- historical G5-A/B/C evidence, read only and retained under its original
  source identity

The static source comparison covered the requested phase, pressure, rupture,
Environment, movement, claim, receiver, spawn, pressure-consequence, reconcile
and pass-order paths, including:

- `engine/core/src/{phase,pressure,rupture,environment}.rs`;
- `engine/gpu/src/{movement_propose,phase_transition,expansion_claim,
  environment_receiver_claim,expansion_spawn_commit,expansion_pressure,
  environment_blocked_expansion_pressure,pressure,rupture}.wgsl`;
- `engine/gpu/src/simulation.rs`;
- the relevant `engine/gpu/tests/{phase,expansion,environment,pressure,
  rupture,parallel_integrity,wgsl_parse,sleep_wake}.rs` contracts.

No source/test command was executed.

## 2. Frozen provenance

The primary snapshot was frozen only after the author incorporated the
reviewer's completion-order, GAS First-Match and staggered-readiness
counterexamples. I independently recomputed these SHA-256 values:

| Authority | SHA-256 |
|---|---|
| ADR-0007 | `63e602b079e761a3f809e517447863c26feaf2a12f894ec1404ad080f1fd6739` |
| bridge specification | `f5f12dfbaf26b96ab2260ecdf82a7753b5fc0f695e07679e2487d3a84fc772da` |
| validation contract | `62fb9724ee126e162bbb2bff76bd46bd84c770553325073883ff46202348d430` |
| TE-5B plan | `af3147fb6134ff90c48f4b550506c9112f557fe7cdf97407417ec647f2471472` |
| production inventory | `33178319a1e6a20ce1d562494f5e80285645d4f22d1c5853fad098e5d81d04fa` |
| ADR-0006 | `47badd386783dbf2f16829f7b75b9d97636485a7a70cb1eeca299797cc55b6b2` |
| phase thermodynamics specification | `ebf45eb758e43df2a805bbb38fd5e8652df688ac85197515774e83ce02417a65` |
| thermal implementation gates | `f273f3cf28c52edcc152af710b4fb03571ed1efe2a53bb7a58447ad48e6d2ad4` |
| external reference script | `6fd9276933822db850bd4ec3f9648cf64c45b8905f6b37d17cc88d03cb23a340` |
| external reference result | `f53173af05199916b10d287a02c8193e9f86c40c853c019db8491cb86ff56e59` |

HEAD is the D-019 authorization commit and its parent is the named source
baseline. `git diff --name-only d7500e2... -- engine apps Cargo.toml
Cargo.lock` was empty. The pre-review dirty scope was therefore docs/memory
only. The following canonical manifest records every dirty file before this
review file was added:

```text
M 012cbf0d77d683dd3815a1d78c13530440859d2fef20e5c8b10d0f17aac33ba4 docs/README.md
M 33178319a1e6a20ce1d562494f5e80285645d4f22d1c5853fad098e5d81d04fa docs/architecture/THERMAL_ENVIRONMENT_PRODUCTION_INVENTORY.md
M 47badd386783dbf2f16829f7b75b9d97636485a7a70cb1eeca299797cc55b6b2 docs/architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md
M 1074823d845f3c294e158ffa5d10112ae99cfbdb366d122b09c39b454eb99184 docs/planning/MILESTONES.md
M e457db721b422e8b7c18cc1f30bcc5b3fc3ef98de3bb676ecc8d4e468fd67f88 docs/planning/STATUS.md
M c15dcfbe91f05c10125c8750443c3d6f92354f3c20e17331080030dbbeec47b5 docs/planning/TE3_WATER_STEAM_PHASE_ACCOUNTING.md
M f273f3cf28c52edcc152af710b4fb03571ed1efe2a53bb7a58447ad48e6d2ad4 docs/planning/THERMAL_ENVIRONMENT_IMPLEMENTATION_GATES.md
M ebf45eb758e43df2a805bbb38fd5e8652df688ac85197515774e83ce02417a65 docs/specs/PHASE_THERMODYNAMICS_SPEC.md
M 83dd38c18041ca529e93df2a5b058214512e2e19a7f83ef11870d47c20e28e29 memory/00-INDEX.md
M 33b411ebb022353cca38d98b94a65a5c655e8386921edaf628994af3f9655f10 memory/OPEN-QUESTIONS.md
? 63e602b079e761a3f809e517447863c26feaf2a12f894ec1404ad080f1fd6739 docs/architecture/decisions/ADR-0007-phase-volume-pressure-bridge.md
? 62fb9724ee126e162bbb2bff76bd46bd84c770553325073883ff46202348d430 docs/development/PHASE_VOLUME_PRESSURE_BRIDGE_VALIDATION.md
? af3147fb6134ff90c48f4b550506c9112f557fe7cdf97407417ec647f2471472 docs/planning/TE5_PHASE_VOLUME_PRESSURE_BRIDGE.md
? f5f12dfbaf26b96ab2260ecdf82a7753b5fc0f695e07679e2487d3a84fc772da docs/specs/PHASE_VOLUME_PRESSURE_BRIDGE_SPEC.md
? bd011c2911b0dc51785a574c684823985db01670a1fea92486c1e4f1cc9f102a memory/checkpoints/20260821-0219-te5b-authorized-before-design-blocker.md
```

The manifest is sorted in the order shown and encoded as
`status + space + sha256 + space + path + LF`, with aggregate SHA-256
`421837987618c0089e6613e88e0747b9b694b0eb874e6c7e81b07c349181ce2d`.
This review file is the only reviewer-authored write and is intentionally not
part of that reviewed-input manifest.

The external result was read and parsed, not regenerated. It reports
`PASS_REFERENCE_MODEL_ONLY`, process executions `1`, failure count `0`,
smallest counterexample `null`, and deterministic digest
`001968b462d75865851e159c35167e6ace04c27c46d12a7f77511823ab378d80`.
That receipt is evidence only for its declared abstract model.

## 3. Finding counts

| Severity | Recorded | Open at verdict | Resolved in reviewed snapshot |
|---|---:|---:|---:|
| Critical | 0 | 0 | 0 |
| High | 3 | **1** | 2 |
| Medium | 4 | **3** | 1 |
| Low | 1 | **1** | 0 |
| Info | 1 | **1 evidence obligation** | 0 |

The verdict is controlled by unresolved Critical/High, not by the abstract
proof's PASS. Current unresolved Critical is **0** and High is **1**.

## 4. Findings

### TE5B-H-001 — Same-tick exclusivity does not consume finite headspace

- **Severity:** High
- **Status:** **OPEN — DESIGN BLOCKER**
- **Counterexample:** Use a sealed one-Cell-wide column with one EMPTY Cell
  above Water. At `t0`, only the top Water is at `E = Lv`; each lower Water
  is initiated but just below the endpoint. The top completion wins the
  up-EMPTY relief claim, settles one Steam, and receives zero pressure. At
  `t1`, ordinary GAS movement moves that Steam into the EMPTY Cell and leaves
  its old source EMPTY. During that tick's thermal/phase sequence, the next
  Water reaches `E = Lv`, targets the newly vacated source Cell, wins, and
  also receives zero pressure. Repeat the staged endpoint arrival down the
  column. Staggering is important: it avoids a simultaneous lower request that
  would stop on an occupied legal Steam density swap. The vacancy walks down
  the column; occupied capacity never increases, yet every completion can win
  zero pressure.
- **Impact:** The candidate cannot guarantee “early relief, then finite
  headspace exhausted, then confinement pressure.” PV-INV-018 is unsatisfied,
  TE5B-F05 and TE5B-F11 are unsatisfiable as causal requirements, and the
  required atomic G5 pressure/rupture chain is not guaranteed. A same-tick
  shared winner prevents duplicate ownership only inside one arbitration
  epoch; it is not a cross-tick volume ledger or reservation.
- **Authority basis:** ADR-0006 locks 1:1 quantity; ADR-0007 and the bridge
  specification lock a non-mutating target and explicitly deny next-tick
  reservation; production GAS movement into EMPTY vacates its source; MILESTONES
  and F05/F11 require finite relief to become confinement rather than merely
  scheduling contention.
- **Resolution:** None inside the current candidate. Persistent capacity or
  reservation state, target/Environment mutation, additional occupied phase
  quantity, or a different pressure law changes a locked premise or reopens a
  rejected option. The frozen authorities correctly mark ADR-0007 Proposed,
  PV-INV-018 unsatisfied, F05/F11 unsatisfiable, and TE-5B DESIGN BLOCKED.

### TE5B-H-002 — Completion acceptance was circular for buried ready Water

- **Severity:** High
- **Status:** **RESOLVED IN FROZEN SNAPSHOT**
- **Counterexample attacked:** Under the earlier ordering, a buried Water Cell
  already at `E = Lv` with every relief stage occupied needed a TE-5
  acceptance to become Steam, while TE-5B emitted its blocked request only
  after observing an already-completed Water-to-Steam event. It could neither
  complete and emit `100.0` nor remain within ADR-0006 without circular
  authorization.
- **Impact if retained:** TE5B-F02/F05/F11 would be unreachable for the core
  confined case, or the implementation would settle unaccepted Steam before
  its pressure consequence.
- **Authority basis:** ADR-0006's explicit completion gate and preservation of
  initiated/buried phase progress; the bridge's exactly-once pressure
  transaction.
- **Resolution:** The frozen design classifies an eligible endpoint
  **attempt** first. Already initiated positive-E/ready Water may enter while
  buried; a current gas-facing normalization may initiate/cross; canonical
  buried `E = 0` Water and non-gas-facing extreme Ice may not. A valid
  targeted or blocked request and provisional Steam Next are written together,
  claim/consequence resolves, and identity/phase/pressure settle jointly.
  `EDGE_DEFERRED` writes NONE and retains ready Water. This closes the design
  circularity; implementation evidence remains future work.

### TE5B-H-003 — Relief selection diverged from production GAS First-Match

- **Severity:** High
- **Status:** **RESOLVED IN FROZEN SNAPSHOT**
- **Counterexample attacked:** At the top edge, production Steam's first
  up-stage returns Void and stops, while an EMPTY-only scan that skipped Void
  could choose a lateral EMPTY Cell. Similarly, an earlier legal upward
  density swap is production's first movement target; skipping it to a later
  lateral EMPTY fabricates relief. Two sources sharing that fabricated target
  could then create a losing `100.0` consequence even though their actual
  first movement outcomes differed.
- **Impact if retained:** False open routes, false contention pressure, and
  open-boil rupture would violate movement reachability and the prompt's Void
  exclusion.
- **Authority basis:** `movement_propose.wgsl` GAS order is up, parity
  up-diagonals, parity lateral with First-Match termination; vertical stages
  allow density swaps and every out-of-domain stage returns Void.
- **Resolution:** The frozen selector replays those stages: EMPTY targets and
  stops; occupied non-swappable stages continue; an earlier legal upward Steam
  swap stops as `BLOCKED`; Void stops as `EDGE_DEFERRED` without checking a
  later lateral Cell. Registry-derived Steam swap/rank data is packed into an
  existing descriptor trait, and F12 names both structural cases. No new
  table/binding is promised.

### TE5B-M-001 — Occupancy-only relief can label compressed Air as free volume

- **Severity:** Medium
- **Status:** **OPEN — USER ARCHITECTURE DECISION**
- **Counterexample:** Two geometrically identical EMPTY target Cells, one
  Vacuum and one Atmospheric with high derived Air pressure/energy, both grant
  identical zero-pressure relief because Air state is not consulted.
- **Impact:** When full TE-5 later gives derived Air pressure mechanical
  meaning, this bridge can classify a strongly compressed EMPTY Cell as the
  same volume opportunity as Vacuum. That may need a compatibility rule or a
  revised eligibility predicate.
- **Authority basis:** ADR-0007 and the bridge specification explicitly make
  Atmospheric and Vacuum EMPTY identical for relief and exclude derived-Air
  pressure from the gauge-pressure bridge.
- **Resolution:** Deliberately open and correctly scoped as a user choice/full
  TE-5 dependency. It is not escalated to High because the frozen candidate
  does not claim background-pressure or structure-force completeness.

### TE5B-M-002 — The inherited `100.0` impulse is a sharp gameplay threshold

- **Severity:** Medium
- **Status:** **OPEN — USER ARCHITECTURE DECISION**
- **Counterexample:** At zero source gauge pressure, one blocked or losing
  completion requests `100.0`, already above Wood's current rupture threshold
  `80.0`. A same-target contention can therefore create a rupture-capable
  source even when other local headspace exists. Near the pressure clamp, the
  requested impulse is still exactly `100.0` but the stored delta can be
  smaller.
- **Impact:** The inherited scalar controls abrupt failure and geometry-
  dependent propagation, so the new 1:1 atomic fixture must establish that the
  old gameplay value still produces the intended causal/readability result.
- **Authority basis:** G5-B freezes the historical consequence; ADR-0007
  intentionally preserves it; the bridge specification distinguishes the
  requested impulse from sanitization/clamp and requires exact-`100.0`
  fixtures to start at zero.
- **Resolution:** No silent retune occurred. Retain/revise is one of the five
  required user decisions and later source-bound evidence remains mandatory.

### TE5B-M-003 — World-edge completion semantics remain deliberately absent

- **Severity:** Medium
- **Status:** **OPEN — EXPLICITLY DEFERRED**
- **Counterexample:** A vaporization-ready Water Cell on the top row has Void
  as resulting Steam's first up-stage and an EMPTY lateral neighbour. TE-5B
  returns `EDGE_DEFERRED`, retains Water/H, emits no request and no pressure,
  even though an already-existing Steam Cell would ordinarily exit through
  Void.
- **Impact:** Boundary boiling may remain indefinitely ready rather than
  vaporizing or venting. This avoids false lateral relief but is not a product
  world-edge pressure/reservoir rule.
- **Authority basis:** The prompt excludes Void as a relief candidate;
  production movement stops on Void; ADR-0007 explicitly defers product edge
  meaning to full TE-5.
- **Resolution:** Coherent for the narrow bridge, but product edge semantics
  remain an explicit later decision and must not be inferred from this design.

### TE5B-M-004 — The conservation trace initially allowed same-tick swap stop

- **Severity:** Medium
- **Status:** **RESOLVED IN FROZEN SNAPSHOT**
- **Counterexample attacked:** If every Water below the one EMPTY Cell were
  already completion-ready at `t0`, the lower Cells would attempt in the same
  phase pass. Their first up-stage is occupied Water and is a legal
  resulting-Steam density swap, so they classify `BLOCKED` and can emit
  pressure immediately. That initial diagram did not demonstrate all-zero-
  pressure vacancy reuse.
- **Impact if retained:** The central High's published witness would not
  follow its claimed tick trace, even though a valid staggered witness exists.
- **Authority basis:** The amended First-Match/swap rule and the single
  phase-pass transaction.
- **Resolution:** All frozen primary authorities now predeclare only the top
  Water ready at `t0`; each lower Water reaches the endpoint after the
  vacancy arrives during that tick's thermal/phase sequence. The corrected
  witness in TE5B-H-001 is valid and the High remains open.

### TE5B-L-001 — “Byte-identical loser target” is ambiguous in mixed-mode arbitration

- **Severity:** Low
- **Status:** **OPEN — DOCUMENTATION CLARIFICATION**
- **Counterexample:** ADR-0007's relief transaction table labels a targeted
  relief loser's target “byte-identical.” If the relief request loses to a
  Matter-expansion request whose Environment receiver succeeds, the shared
  target is intentionally replaced by the Matter winner's historical spawn.
  It is unchanged **by relief**, but not byte-identical after the whole mixed-
  mode transaction.
- **Impact:** A fixture author could incorrectly demand unchanged target bytes
  for a Matter-wins mixed-mode case.
- **Authority basis:** The bridge specification's mixed-mode section correctly
  permits the Matter winner's spawn and gives the relief loser `100.0` once.
- **Resolution:** Normative outcome is recoverable and ownership is not
  ambiguous in the specification; ADR wording should eventually say
  “no mutation attributable to relief” or scope that row to relief-only
  contention.

### TE5B-I-001 — The reference PASS cannot validate geometry or production wiring

- **Severity:** Info
- **Status:** **OPEN EVIDENCE OBLIGATION, NOT A DESIGN HIGH**
- **Counterexample to overclaim:** A future implementation could use the wrong
  descriptor slot for gas-facing extreme Ice, skip the ready-Water surface-
  reopen trigger, mishandle Void/swap geometry, or omit a sleeping-path full
  write while the existing Python JSON still reports PASS. The script uses
  abstract requests and an identity-labelled
  `completion_word("Ice", "Steam", ...)`; it does not construct a grid,
  registry descriptor, normalization invocation or GPU pass.
- **Impact:** The receipt cannot establish WGSL validity, producer/consumer
  bindings, sleep equivalence, pressure propagation, rupture/venting,
  performance or user acceptance.
- **Authority basis:** The validation contract now states these limitations,
  requires structural/semantic fixtures for descriptor generation, ready
  Water/extreme Ice, claim-writer rejection, full writes, bindings and F01–F12,
  and does not claim downstream proposal revalidation that the consumers do
  not bind.
- **Resolution:** Correctly bounded. No missing future runtime evidence is
  promoted to another design High.

## 5. Required attack coverage

| # | Adversarial failure mode | Result |
|---:|---|---|
| 1 | non-exclusive relief reintroduced | Closed for one tick by the shared claim; cross-tick failure is H-001 |
| 2 | one EMPTY relieves multiple sources | One winner in an epoch; sequential reuse remains H-001 |
| 3 | target is not actual Steam GAS EMPTY route | H-003 resolved by production-order replay |
| 4 | down/occupied/swap/Void misclassification | Down excluded; occupied continue; swap blocked-stop; Void edge-deferred |
| 5 | later Steam tick repeats pressure | Steam emits NONE; event is attempt-derived |
| 6 | max Cell index/sentinel/mode collision | Strict `cell_count < 1 << 30` leaves low-30-bit `index+1` disjoint from modes |
| 7 | invalid/reserved/mismatched side effect | Claim writer rejects/fully writes zero; downstream trusts that structural boundary and validates bound claim fields |
| 8 | Environment receiver consumes relief | Early mode rejection required |
| 9 | spawn creates relief Matter | Early mode rejection required; winner target unchanged by relief |
| 10 | Environment-blocked pass double-charges relief | Matter-mode only |
| 11 | generic expansion is damaged | Separate Matter mode and F06 preserve receiver/spawn/failure grammar |
| 12 | mixed modes obtain two winners | One candidate list/one claim; L-001 is wording only |
| 13 | finite headspace never becomes confinement | **H-001 open** |
| 14 | open boil creates false pressure/rupture | Unique winner has zero source; Void false-lateral case resolved; same-target contention is deliberate |
| 15 | G5 fixture depends on combustion/script | F11 forbids boiler explosion, combustion opening and pre-staged pressure, but is blocked by H-001 |
| 16 | scalar `100` silently retuned | Not retuned; M-002 remains user-owned |
| 17 | derived Air pressure leaks into bridge | Explicitly excluded; M-001 remains user-owned |
| 18 | full TE-5/Air force leaks into TE-5B | Background/face force, Vacuum pressure and combustion remain deferred |
| 19 | extra pass/scratch/binding appears | Static 40-pass/80-query/8-storage projection is coherent; production evidence remains I-001 |
| 20 | historical evidence rebound/external implementation imported | Historical receipts stay source-bound; source diff is zero; no external implementation imported |

Additional required attacks:

- gas-facing extreme Ice and Water `E = Lv, T = 100` reopen are now explicit
  future semantic/structural fixtures; the proof does not establish them;
- a Matter winner whose receiver fails never reopens arbitration for the
  relief loser;
- the token is explicitly not a next-tick reservation;
- phase context, proposal, claim and receiver scratch live ranges end before
  full overwrite, and all projected storage counts are at or below eight;
- phase/pressure settles before propagation and rupture;
- sleeping/context-skip paths must write NONE, with future structural and
  sleep-on/off evidence still required;
- the pure proof is not represented as grid, geometry, WGSL or GPU evidence.

## 6. Verification performed and intentionally omitted

Performed read-only/static work:

- read project Ballast authority, D-018/D-019, Q-009 and current checkpoint;
- recomputed every frozen primary/proof hash above;
- parsed the existing result JSON and inspected the external script without
  executing it;
- compared HEAD/source baseline and confirmed no source/Cargo diff;
- inspected source writers, consumers, binding layouts, pass order, pressure
  settle/propagation/rupture order and relevant test contracts;
- checked the strict max-index arithmetic and the shared-mode decoder;
- checked that the primary authorities record H-001 and do not rebind
  historical G5 evidence.

Intentionally omitted:

- Rust execution: **0**
- WGSL/Naga execution: **0**
- Cargo/build/clippy/test execution: **0**
- launch/GPU/runtime/device execution: **0**
- proof/reference-script reruns: **0**
- fixture, performance or visual execution: **0**
- Wiki edits: **0**

Only file reads, searches, Git inspection, SHA-256 calculation and JSON parsing
were used. The external proof script/result were never modified.

## 7. Remaining risks and exactly five user decisions

The five candidate decisions that remain visible are:

1. **Exclusive token / finite-capacity owner:** revise or replace the
   exclusive local token and state which no-new-state, target-non-mutation or
   1:1 constraint, if any, may change so finite capacity is actually consumed.
2. **Occupancy-only Air relief:** approve or revise equal treatment of
   Atmospheric EMPTY and Vacuum EMPTY without derived-Air pressure input.
3. **Encoding:** approve or revise
   `00 none / 01 Matter / 10 relief / 11 invalid` under the 30-bit Cell-index
   bound.
4. **Confinement scalar:** retain or revise the inherited `100.0` impulse in
   the future atomic source/fixture.
5. **Finite-headspace product meaning:** confirm or revise the required
   early-relief → ordinary movement → later-confinement → pressure → rupture
   F05/F11 causal behavior.

Separate later decisions/evidence are not hidden inside those five: product
world-edge mode, full TE-5 background/structure coupling, Vacuum combustion,
runtime authorization, source-bound WGSL/GPU/performance evidence and user
acceptance remain outstanding.

## 8. Final verdict

Unresolved Critical: **0**

Unresolved High: **1**

The same-tick non-mutating token cannot own or consume finite headspace across
ticks under 1:1 occupancy, and the corrected staggered-heating trace defeats
F05/F11 without relying on missing runtime evidence.

**TE-5B DESIGN BLOCKED**
