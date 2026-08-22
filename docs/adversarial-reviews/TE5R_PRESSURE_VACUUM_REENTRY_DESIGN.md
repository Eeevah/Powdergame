# Independent TE-5R0 Pressure / Vacuum Re-entry Design Review

- **Date:** 2026-08-23
- **Reviewer role:** fresh-context independent adversarial reviewer; not a
  primary-author participant
- **Reviewed branch:** `feature/m0-g9-first-playable`
- **Source baseline / HEAD:**
  `769e687c04406016fe9d66c8496269b459f06d83`
- **Current unresolved:** Critical **0**, High **3**, Medium **3**, Low **0**
- **Runtime/reference execution by this reviewer:** **0**
- **Verdict:** **TE-5R0 DESIGN BLOCKED**

## 1. Scope and provenance

This review attacks the D-035-authorized local relaxing phase-load pressure
candidate. It reviewed:

- [ADR-0013](../architecture/decisions/ADR-0013-local-relaxing-phase-load-pressure.md);
- [Pressure / Vacuum coupling specification](../specs/PRESSURE_VACUUM_COUPLING_SPEC.md);
- [validation contract](../development/PRESSURE_VACUUM_COUPLING_VALIDATION.md);
- [TE-5R0 re-entry plan](../planning/TE5R_PRESSURE_VACUUM_REENTRY.md);
- [production inventory](../architecture/THERMAL_ENVIRONMENT_PRODUCTION_INVENTORY.md)
  section 17;
- D-035, D-036, the current checkpoint, and immutable blocked TE-5 history only
  where needed to preserve the authorization and stop boundaries.

The live-source comparison covered the current pressure, Air transport,
movement, phase context/transition, generic expansion pressure, combustion,
rupture, activity/sleep, tick ordering, profiler and allocation paths under
`engine/core`, `engine/gpu` and `apps/windows`. No production source differs
from HEAD in `engine`, `apps`, `Cargo.toml` or `Cargo.lock`.

Reviewed primary-document SHA-256 identities:

| Input | SHA-256 |
|---|---|
| ADR-0013 | `14d2581c95574c2be16aa8a411785b92dac423fa58dab70a58c37d98db6b41f9` |
| coupling specification | `b02ee016c33432ef1f5f0441f9e7ad11b8d4cb350b91b4f68c525bf8d041d834` |
| validation contract | `00fa7f7737862f346a5bc950bd4359e618be2f4710c53398ec137d32511e4188` |
| TE-5R0 plan | `68ba1193e1db2708cb264b2e2b1dd6e40a8ad202a857584e31ac0acf2c44568e` |
| production inventory | `2442d3277c36b68b58b5333793b2e88864216a4b87b874454bcd97e79716dd4a` |

These are dirty docs/memory-only design inputs. This review file is the only
reviewer-authored write. It is intentionally excluded from the input list.

## 2. Finding counts

| Severity | Open at verdict | Resolved in reviewed snapshot |
|---|---:|---:|
| Critical | **0** | 0 |
| High | **3** | 0 |
| Medium | **3** | 0 |
| Low | **0** | 0 |

The D-035 stop rule is controlled by unresolved Critical/High findings.

## 3. Findings

### TE5R-H-001 — The projected pressure pass cannot use the required TE-3 phase-context snapshot

- **Severity:** High
- **Status:** **OPEN — DESIGN BLOCKER**
- **Smallest concrete counterexample:** Enclose a partially vaporized Water
  Cell (`0 < E < Lv`) so its only gas-facing neighbour in the immutable TE-3
  phase-context snapshot is a valid partial Steam Cell. Stage the Steam so the
  same phase pass condenses it to Water. TE-3 correctly sets the first Water's
  gas-facing bit from the pre-transition snapshot. After phase settle, however,
  a recomputation from settled Material sees only Water/non-gas neighbours and
  returns buried. The converse is also possible when a neighbouring Water
  becomes Steam in the phase pass.
- **Source contradiction:** The specification requires the Water pressure
  source to use "the same material/class snapshot as phase context." Production
  writes that snapshot into reused `claim` scratch in
  `phase_context_propose.wgsl`, consumes it in `phase_transition.wgsl`, and
  overwrites the same lifetime in expansion and Smoke claim passes before the
  proposed local pressure position. The projected local-pressure bindings are
  only Material, phase energy, pressure, class, chunk and pressure output; they
  contain neither the preserved phase-context bit nor its pre-transition
  Material snapshot.
- **Impact:** Buried ready-Water and Water/Steam continuity can depend on which
  later snapshot an implementation silently chooses. PV-INV-002/003/004,
  F08/F09 and the stated six-binding source-feasibility projection are not one
  implementable contract.
- **Resolution:** None in the reviewed snapshot. The primary documents must be
  corrected and re-reviewed; this review does not synthesize a replacement.

### TE5R-H-002 — Fresh generic impulse is not an input to the documented local update

- **Severity:** High
- **Status:** **OPEN — DESIGN BLOCKER**
- **Smallest concrete counterexample:** Start an isolated eligible pressure
  node at dynamic zero with phase target zero and let a future valid generic
  expansion failure emit impulse `100`. The existing expansion writer stores
  `pressure_next = pressure_current + 100`, and that buffer is copied to
  `pressure_current` before the later pressure pass. With degree zero, the
  projected local pass can only compute `(1-R)*100 = 98`. The normative
  equation, which adds `bounded_generic_impulse` after diffusion/relaxation,
  computes `100`.
- **Source contradiction:** ADR-0013 and the specification put an explicit
  additive impulse in the local equation and say the event writes before the
  pressure update. The 44-pass plan gives the local pressure pass no proposal,
  claim, descriptor, event bit or pre-impulse pressure input. It therefore
  cannot distinguish fresh impulse from older stored dynamic pressure. With
  neighbours, the mismatch also redistributes/attenuates the fresh impulse in
  the same tick instead of applying the documented local term.
- **Impact:** The generic consequence, its exactly-once value, activity and
  rupture tick are under-specified. The six-binding pressure projection does
  not implement the normative formula, and a future implementation could pass
  one interpretation while violating the other.
- **Resolution:** None in the reviewed snapshot. The primary documents must
  define one source-realizable transaction and re-enter review.

### TE5R-H-003 — “Base activity unchanged” prevents exact-update sleep

- **Severity:** High
- **Status:** **OPEN — DESIGN BLOCKER**
- **Smallest concrete counterexample:** Use a sealed two-node component with
  one canonical Steam node targeting `100` and one buried Water node targeting
  `0`; surround it with Static Matter so neither identity has movement work.
  With `D=0.20` and `R=0.02`, the unclamped equilibrium is approximately
  `(52.381, 47.619)`. Both exact local updates are zero, but their pressure
  difference is about `4.762`, far above the current pressure activity epsilon
  `0.001`.
- **Source contradiction:** The live maxed base activity pass still marks any
  Liquid/Gas pair whose stored pressure differs by more than epsilon. The plan
  says this pass is unchanged and then adds a dedicated exact-update pressure
  activity pass. Activity proposals OR into the same mask, so the new pass
  cannot clear the legacy bit. The two-node equilibrium therefore remains
  active forever even though the proposed update says there is no work.
- **Impact:** PV-INV-019, F19 eventual sleep/equivalence and the claimed
  pressure-activity semantics are unsatisfied. This is a static source-graph
  contradiction inside the exact 44-pass projection, not missing runtime
  evidence.
- **Resolution:** None in the reviewed snapshot. The source-feasibility table
  and activity ownership need correction and fresh review.

### TE5R-M-001 — Dissipation can satisfy the relief story without proving the opening caused relief

- **Severity:** Medium
- **Status:** **OPEN — VALIDATION AMBIGUITY**
- **Counterexample:** Remove or reduce a phase target after a peak but do not
  open a wall. The `R` term lowers stored pressure anyway. F21's ordered chain
  can then observe "opening" followed by a lower later peak even if the opening
  contributed no measurable relief; F10 likewise calls target-driven decay
  condensation relief.
- **Impact:** The accepted product distinction between an actual vent/topology
  change and deliberate sealed-field dissipation can be hidden by the same
  monotone trace. F11 is stronger because it starts from a settled load, but
  F21 does not require a matched no-opening control or an opening-attributable
  delta.
- **Resolution:** No blocking replacement is proposed here. The validation
  contract needs an explicit causal discriminator before implementation
  evidence can claim vent relief.

### TE5R-M-002 — Moving phase sources can self-propel down their own spatial trail without a bound

- **Severity:** Medium
- **Status:** **OPEN — PRODUCT/VALIDATION RISK**
- **Counterexample:** A Steam source in Cell A builds dynamic pressure there.
  In a legal multi-candidate Gas stage it selects lower-pressure Cell B. Stored
  pressure remains at A while the phase target relocates to B. On the next tick
  the source can again prefer a lower-pressure Cell C, leaving a dissipating
  trail. Reversing parity or encountering a wall can turn the same delayed
  feedback into oscillatory left/right choices.
- **Impact:** This follows the stated spatial-pressure/source-relocation rule
  but can look like pressure-generated momentum despite the explicit
  no-velocity model. F18 checks ownership hygiene and user review is named, but
  no fixture gives an acceptable displacement, oscillation or trail bound.
- **Resolution:** Remains a required direct product decision/evidence risk. It
  is not escalated to High because the candidate openly labels feedback and
  oscillation as a cost rather than claiming their absence.

### TE5R-M-003 — “Sealed edge” is pressure/Air-only while production Matter still exits to Void

- **Severity:** Medium
- **Status:** **OPEN — EDGE SEMANTICS AMBIGUOUS**
- **Counterexample:** Put Steam on the top row with its up stage out of domain.
  Production movement returns `VOID_TARGET`, the claim executes
  unconditionally, and commit deletes the Matter and its phase energy. The
  proposed pressure edge simultaneously treats that missing face as no-flux;
  its spatial dynamic value remains and merely relaxes.
- **Impact:** ADR/spec prose calls the product/default edge sealed/no-flux and
  fixture accounting discusses zero external exchange, while the preserved
  ordinary movement semantics retain an open Matter sink. An edge-adjacent
  phase-load fixture can therefore lose Water-equivalent quantity externally
  without a reservoir exchange record even though pressure is "sealed."
- **Resolution:** Clarify the field-by-field boundary contract and cover it in
  validation. This is Medium because the pressure-only interpretation is
  recoverable from context, but it is not stated tightly enough for evidence.

## 4. Required attack coverage

| Required attack | Review result |
|---|---|
| open-space local hot spots / false rupture | The convex update bounds a phase-only field by target extrema, and F05 names the sparse-load Wood threshold. Coefficients and product result remain unrun; no additional static blocker beyond the open product gate was found. |
| dynamic pressure in zero-Air Vacuum | Internally explicit: it cannot manufacture donor mass, but it can temporarily block refill or bias neighbouring Air. This remains a required user architecture decision. |
| Water surface continuity / buried ready-Water | **Blocked by H-001.** |
| component-average derivation | The unclamped sealed equilibrium sum is algebraically correct for symmetric edges; it does not establish local peaks, clamps or sleep. |
| narrow-neck finite-rate meaning | A one-face explicit stencil is consistent with the claimed causal frontier. F12 remains future runtime evidence. |
| dissipation versus vents/condensation | **M-001.** |
| background/dynamic and generic double counts | F16 covers total-pressure consumers; the generic impulse transaction itself is **blocked by H-002**. |
| uniform-pressure wall stress | The opposing-face difference formula is immune to equal left/right and up/down pressure. No static contradiction found. |
| pressure-force feedback | **M-002.** |
| Air-flow circular dependency | Movement and Air intentionally read the previous settled pressure snapshot; Air background then changes on a delayed loop. No same-pass read/write cycle was found, but Vacuum potential remains a product risk. |
| Matter movement / spatial pressure / source relocation | Ownership is stated consistently; behavioral acceptance is missing as **M-002**. |
| reset/editor stale pressure | Live reset, direct authoring and Sandbox Draw/Erase paths clear both pressure halves. F18 correctly keeps every staging path as future evidence. |
| sleep before relaxation | **Blocked by H-003.** |
| exact 44-pass / binding / scratch feasibility | Arithmetic `42 + split Air commit + pressure activity = 44` and `88` queries is consistent. Semantic feasibility is not closed because H-001/H-002 lack required inputs and H-003 contradicts the unchanged base pass. |
| edge/reservoir | **M-003.** Fixture-only standard reservoir is otherwise explicitly non-product. |
| Vacuum combustion | Exact Vacuum still lacks a positive Air face; no TE-4 policy bypass is proposed. F20 must prove this on future source. |
| G5 evidence rebound | Primary documents correctly forbid rebound. |
| TE-2/3/4 regression | F20 names the accepted regression surface; no future source exists, so none is established. |
| hidden token/matching/capacity | No token, reservation, matching, CCL, packet, owner field, ninth binding or full-world allocation is promised. The three Highs are source-contract mismatches, not a demand to restore those rejected models. |

## 5. Checks performed and omitted

Performed, read-only:

- confirmed branch and source HEAD;
- confirmed no dirty production delta under `engine`, `apps`, Cargo manifests;
- hashed the five primary design inputs;
- inspected D-035/D-036 and current Ballast checkpoint/index;
- traced live pass ordering, Current/Next copies and scratch reuse;
- counted the projected pass/query delta and representative bindings;
- inspected pressure, Air, movement/Void, phase context/transition, generic
  impulse, rupture, activity/sleep, reset/editor and profiler/allocation paths;
- checked each required adversarial category with a smallest static witness
  where a contradiction was found.

Deliberately omitted under the task boundary:

```text
reference/Python proof or coefficient campaign: 0
Rust test/check/clippy: 0
WGSL parse/Naga: 0
GPU/device/runtime/FULL: 0
build/application launch: 0
remote/network/GitHub/Wiki operation: 0
production source or primary-document edit: 0
```

## 6. Remaining risks and exact verdict

The zero-Air Vacuum meaning, dissipative sealed field, coefficient/product
meaning, local hot spots, delayed Air coupling, pressure-biased Matter motion,
reservoir mode and direct readability remain user/runtime risks even after the
three source-contract blockers are corrected. Nothing in this docs-only review
establishes runtime, performance, allocation, sleep, rupture or product
acceptance.

Unresolved Critical: **0**. Unresolved High: **3**. Under D-035, the exact
verdict is:

**TE-5R0 DESIGN BLOCKED**

ADR-0013 remains **PROPOSED — ARCHITECTURE RE-ENTRY CANDIDATE / USER REVIEW
PENDING**. The primary ADR/spec/validation/plan and production-inventory
projection require correction before another fresh-context review. No runtime
implementation is authorized, and this review does not invent a replacement.
