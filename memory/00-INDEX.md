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
| [`docs/planning/TE3_WATER_STEAM_PHASE_ACCOUNTING.md`](../docs/planning/TE3_WATER_STEAM_PHASE_ACCOUNTING.md) | Registered TE-3 closed-cycle and mid-air phase-accounting blocker | Before any Water/Steam phase design or runtime proposal |

## Authority and evidence boundaries

- Memory is a pointer and decision layer, not an evidence package.
- A receipt proves only its exact source/run contract. It does not transfer runtime provenance to a memory or docs commit.
- `docs/planning/STATUS.md` changes only when milestone/evidence state changes; routine session coordinates belong only in `memory/CHECKPOINT.md`.
- `docs/HANDOFF.md` is no longer updated after every session. It remains available for preserved recovery/domain context.
- Do not store credentials, authentication material, generated artifacts, copied evidence, or raw telemetry in `memory/`.
- Emergency disable and rollback are documented in `docs/development/BALLAST_MEMORY_CUTOVER.md`.
