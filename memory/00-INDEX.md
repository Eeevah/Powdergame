# Powdergame active project memory

This directory is Powdergame's single active Ballast session-continuity map. It reconnects a session to the existing sources of truth without copying or replacing them. Current source and live runtime observations remain authoritative for observed facts.

## Resume order

1. Read this index.
2. If `memory/HANDOFF.md` exists, consume its single-use instruction and register the result before continuing.
3. Read `memory/CHECKPOINT.md`.
4. Read active entries in `memory/DECISIONS.md`.
5. Open only the canonical project documents relevant to the current task.
6. Before validation, read `docs/development/VALIDATION_POLICY.md` and check for reusable same-SHA evidence.

## Active memory files

| Path | Purpose | Authority | Update rule |
|---|---|---|---|
| [`memory/CHECKPOINT.md`](CHECKPOINT.md) | Thirty-second return point and exactly one next action | Current session coordinate; live Git wins if stale | Update after a substantial unit, before handoff, or when validity/next action changes |
| [`memory/DECISIONS.md`](DECISIONS.md) | User-confirmed decisions and supersession history | Adopted decisions within recorded scope | Append only; supersede with a new ID |
| [`memory/OPEN-QUESTIONS.md`](OPEN-QUESTIONS.md) | Active unresolved choices and dated dispositions | Pending-question register, not project-state proof | Append openings and closures; never erase history |
| [`memory/SESSION-LOG.md`](SESSION-LOG.md) | Compact continuity audit | Historical session record | Append after a meaningful session or cutover |
| [`memory/checkpoints/`](checkpoints/) | Archived previous checkpoints | Historical only | Add an archive when replacing a checkpoint with materially different scope |
| [`memory/checkpoints/20260819-1357-g8-closed-g9a-next.md`](checkpoints/20260819-1357-g8-closed-g9a-next.md) | Last pre-implementation G9-A return point | Historical only | Immutable archive of the superseded checkpoint |
| [`memory/checkpoints/20260820-0931-g9a-first-candidate-reviewed.md`](checkpoints/20260820-0931-g9a-first-candidate-reviewed.md) | First G9-A candidate return point before direct-review remediation | Historical only | Immutable archive of the superseded checkpoint |
| [`memory/checkpoints/20260820-1006-g9a-revised-candidate-rereview.md`](checkpoints/20260820-1006-g9a-revised-candidate-rereview.md) | Five-revision G9-A return point before continuity v2 | Historical only | Immutable archive of the superseded checkpoint |
| [`memory/checkpoints/20260820-1255-g9a-continuity-v2-thermal-planned.md`](checkpoints/20260820-1255-g9a-continuity-v2-thermal-planned.md) | Continuity v2 return point before D-013 and TE-0 design lock | Historical only | Immutable archive of the superseded checkpoint |
| [`memory/checkpoints/20260820-1641-te1-foundation-implemented.md`](checkpoints/20260820-1641-te1-foundation-implemented.md) | TE-0 design-lock return point before TE-1 runtime implementation | Historical only | Immutable archive of the superseded checkpoint |
| [`memory/checkpoints/20260820-2130-te2-passive-candidate-before-direct-remediation.md`](checkpoints/20260820-2130-te2-passive-candidate-before-direct-remediation.md) | Original TE-2 candidate return point before direct-review remediation | Historical only | Immutable archive of the superseded checkpoint |
| [`memory/checkpoints/20260820-2239-te2-revised-candidate-rereview.md`](checkpoints/20260820-2239-te2-revised-candidate-rereview.md) | Revised TE-2 candidate return point before direct user acceptance | Historical only | Immutable archive of the superseded checkpoint |
| [`memory/checkpoints/20260820-2328-te2-accepted-te3d-authorized.md`](checkpoints/20260820-2328-te2-accepted-te3d-authorized.md) | TE-2 accepted and TE-3D design-authorized return point before the design candidate | Historical only | Immutable archive of the superseded checkpoint |
| [`memory/checkpoints/20260821-0111-te3d-v1-review-pending.md`](checkpoints/20260821-0111-te3d-v1-review-pending.md) | TE-3D v1 independent-review return point before D-018 locked-amendment closure | Historical only | Immutable archive of the superseded checkpoint |
| [`memory/checkpoints/20260821-0153-te3d-accepted-te5b-next.md`](checkpoints/20260821-0153-te3d-accepted-te5b-next.md) | D-018 architecture-accepted return point before TE-5B design authorization | Historical only | Immutable archive of the superseded checkpoint |
| [`memory/checkpoints/20260821-0219-te5b-authorized-before-design-blocker.md`](checkpoints/20260821-0219-te5b-authorized-before-design-blocker.md) | D-019 authorization return point before independent review exposed the finite-capacity blocker | Historical only | Immutable archive of the superseded checkpoint |
| [`memory/checkpoints/20260821-1139-te5b-blocked-te5c-authorized.md`](checkpoints/20260821-1139-te5b-blocked-te5c-authorized.md) | TE-5B blocked return point before D-020 authorized TE-5C | Historical only | Immutable archive of the superseded checkpoint |
| [`memory/checkpoints/20260821-1245-te5c-blocked-te5d-authorized.md`](checkpoints/20260821-1245-te5c-blocked-te5d-authorized.md) | TE-5C blocked return point before D-021 authorized TE-5D persistent state | Historical only | Immutable archive of the superseded checkpoint |

## Canonical project authorities

| Path | Role | When to open |
|---|---|---|
| [`docs/START_HERE.md`](../docs/START_HERE.md) | Product intent and surface taxonomy | When recovering why the game exists or reviewing product direction |
| [`docs/planning/STATUS.md`](../docs/planning/STATUS.md) | Current milestone, gate, verdict, evidence router | When actual project state or next gate matters |
| [`docs/planning/G9_PRODUCT_BRIEF_2026-08-19.md`](../docs/planning/G9_PRODUCT_BRIEF_2026-08-19.md) | User-approved first playable scope, sequencing and exclusions | Before any G9 implementation or product-scope change |
| [`docs/planning/MILESTONES.md`](../docs/planning/MILESTONES.md) | Gate completion contracts | Before claiming closure |
| [`docs/planning/ROADMAP.md`](../docs/planning/ROADMAP.md) | Adopted long-term direction and ordering | When choosing a later product path |
| [`docs/development/VALIDATION_POLICY.md`](../docs/development/VALIDATION_POLICY.md) | Change-impact validation and same-SHA reuse | Before any validation or evidence reuse |
| [`docs/development/LESSONS_LEDGER.md`](../docs/development/LESSONS_LEDGER.md) | Promoted technical/process lessons | Before repeating a known failure or changing policy |
| [`docs/development/WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md`](../docs/development/WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md) | Worktree, launcher, executable, artifact limits | Before creating or retiring those resources |
| [`docs/development/BALLAST_MEMORY_CUTOVER.md`](../docs/development/BALLAST_MEMORY_CUTOVER.md) | Adoption, immediate disable, rollback, and merge contract | Before changing or removing Ballast |
| [`docs/HANDOFF.md`](../docs/HANDOFF.md) | Preserved historical/domain handoff reference | Only when older recovery history or domain context is needed; not a competing session checkpoint |
| [`docs/evidence/`](../docs/evidence/) | Immutable run/source/artifact evidence and user dispositions | When relying on a specific measurement or acceptance claim |
| [`docs/architecture/`](../docs/architecture/) and [`docs/specs/`](../docs/specs/) | Architecture and implementation contracts | Before code/engine changes |
| [`docs/planning/TE3_WATER_STEAM_PHASE_ACCOUNTING.md`](../docs/planning/TE3_WATER_STEAM_PHASE_ACCOUNTING.md) | TE-3D accepted-architecture entry point and original closed-cycle/mid-air blocker | Before any Water/Steam phase design or runtime proposal |
| [`docs/architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md`](../docs/architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md) | D-018-accepted Hybrid A+C decision and locked consequences | Before any separately authorized atomic TE-3/TE-5 implementation work |
| [`docs/specs/PHASE_THERMODYNAMICS_SPEC.md`](../docs/specs/PHASE_THERMODYNAMICS_SPEC.md) | Accepted phase-energy math, ownership, pass and invariant contract | Before reviewing or implementing TE-3 |
| [`docs/development/PHASE_THERMODYNAMICS_VALIDATION.md`](../docs/development/PHASE_THERMODYNAMICS_VALIDATION.md) | Evidence ladder, fixture contract and one-shot reference receipt | Before making any TE-3 validation claim |
| [`docs/adversarial-reviews/TE3_PHASE_ENTHALPY_DESIGN.md`](../docs/adversarial-reviews/TE3_PHASE_ENTHALPY_DESIGN.md) | Preserved v1 and fresh v2 independent design attacks/dispositions | Before future implementation or any reassessment of ADR-0006 |
| [`docs/planning/TE5_PHASE_VOLUME_PRESSURE_BRIDGE.md`](../docs/planning/TE5_PHASE_VOLUME_PRESSURE_BRIDGE.md) | D-019 TE-5B option audit, reference receipt and finite-capacity DESIGN BLOCKED router | Before any revised phase-volume bridge decision or runtime proposal |
| [`docs/architecture/decisions/ADR-0007-phase-volume-pressure-bridge.md`](../docs/architecture/decisions/ADR-0007-phase-volume-pressure-bridge.md) | Proposed exclusive token and unresolved vacancy-conservation counterexample | Before revising or replacing the atomic TE-3/TE-5B bridge |
| [`docs/specs/PHASE_VOLUME_PRESSURE_BRIDGE_SPEC.md`](../docs/specs/PHASE_VOLUME_PRESSURE_BRIDGE_SPEC.md) | Evaluated mode/arbitration contract plus unsatisfied finite-capacity invariant | Before any TE-5B redesign or structural fixture |
| [`docs/development/PHASE_VOLUME_PRESSURE_BRIDGE_VALIDATION.md`](../docs/development/PHASE_VOLUME_PRESSURE_BRIDGE_VALIDATION.md) | One-shot pure proof boundary and currently unsatisfiable F05/F11 fixtures | Before making any TE-5B validation claim |
| [`docs/adversarial-reviews/TE5_PHASE_VOLUME_PRESSURE_BRIDGE_DESIGN.md`](../docs/adversarial-reviews/TE5_PHASE_VOLUME_PRESSURE_BRIDGE_DESIGN.md) | Fresh independent review; Critical 0 / High 1 and TE-5B DESIGN BLOCKED | Before revising the capacity model or interpreting the reference PASS |
| [`docs/planning/TE5_LOCAL_VAPOR_CAPACITY_PRESSURE.md`](../docs/planning/TE5_LOCAL_VAPOR_CAPACITY_PRESSURE.md) | D-020 TE-5C plan and one-shot open-capacity DESIGN BLOCKED router | Before any persistent phase-volume replacement decision |
| [`docs/architecture/decisions/ADR-0008-local-vapor-capacity-pressure.md`](../docs/architecture/decisions/ADR-0008-local-vapor-capacity-pressure.md) | Proposed proportional capacity/equilibrium law and blocking underuse counterexample | Before revising volume ownership |
| [`docs/specs/LOCAL_VAPOR_CAPACITY_PRESSURE_SPEC.md`](../docs/specs/LOCAL_VAPOR_CAPACITY_PRESSURE_SPEC.md) | Exact failed capacity/pressure/vent candidate contract | Before interpreting the TE-5C result |
| [`docs/development/LOCAL_VAPOR_CAPACITY_PRESSURE_VALIDATION.md`](../docs/development/LOCAL_VAPOR_CAPACITY_PRESSURE_VALIDATION.md) | Predeclared one-shot grid/time proof and DESIGN BLOCKED receipt | Before making any TE-5C evidence claim |
| [`docs/adversarial-reviews/TE5_LOCAL_VAPOR_CAPACITY_PRESSURE_DESIGN.md`](../docs/adversarial-reviews/TE5_LOCAL_VAPOR_CAPACITY_PRESSURE_DESIGN.md) | Fresh review; Critical 0 / High 6 and TE-5C DESIGN BLOCKED | Before any persistent-state replacement decision or interpretation of the one-shot receipt |
| [`docs/planning/TE5_PERSISTENT_VAPOR_EXTENT.md`](../docs/planning/TE5_PERSISTENT_VAPOR_EXTENT.md) | D-021 TE-5D plan and wider-matching DESIGN BLOCKED router | Before any wider matching or search-scratch decision |
| [`docs/architecture/decisions/ADR-0009-persistent-vapor-extent.md`](../docs/architecture/decisions/ADR-0009-persistent-vapor-extent.md) | Proposed reciprocal extent/dedicated pressure candidate and depth-six blocker | Before revising persistent phase-volume ownership |
| [`docs/specs/PERSISTENT_VAPOR_EXTENT_SPEC.md`](../docs/specs/PERSISTENT_VAPOR_EXTENT_SPEC.md) | Exact failed link, movement, matching and phase-pressure candidate | Before interpreting the TE-5D result |
| [`docs/development/PERSISTENT_VAPOR_EXTENT_VALIDATION.md`](../docs/development/PERSISTENT_VAPOR_EXTENT_VALIDATION.md) | Frozen one-shot proof contract and DESIGN BLOCKED receipt | Before making any TE-5D evidence claim |
| [`docs/adversarial-reviews/TE5_PERSISTENT_VAPOR_EXTENT_DESIGN.md`](../docs/adversarial-reviews/TE5_PERSISTENT_VAPOR_EXTENT_DESIGN.md) | Fresh review; Critical 0 / High 6 / Medium 2 and TE-5D DESIGN BLOCKED | Before a wider matching replacement decision |

## Authority and evidence boundaries

- Memory is a pointer and decision layer, not an evidence package.
- A receipt proves only its exact source/run contract. It does not transfer runtime provenance to a memory or docs commit.
- `docs/planning/STATUS.md` changes only when milestone/evidence state changes; routine session coordinates belong only in `memory/CHECKPOINT.md`.
- `docs/HANDOFF.md` is no longer updated after every session. It remains available for preserved recovery/domain context.
- Do not store credentials, authentication material, generated artifacts, copied evidence, or raw telemetry in `memory/`.
- Emergency disable and rollback are documented in `docs/development/BALLAST_MEMORY_CUTOVER.md`.
