# Powdergame agent instructions

<!-- BEGIN managed: ballast-project-memory -->
## Active bounded project memory

Powdergame uses Ballast as the **single active session-continuity workflow**. It is a navigation, resumption, and user-decision layer; it does not replace the authoritative documents for product vision, architecture, specifications, milestones, evidence, validation, or runtime facts.

For every new, reset, or compacted session:

1. Read `memory/00-INDEX.md`.
2. Read `memory/CHECKPOINT.md`.
3. Read active entries in `memory/DECISIONS.md`.
4. Open only the canonical project documents linked by the index that are relevant to the current task.
5. Read `docs/development/VALIDATION_POLICY.md` before running validation.
6. Search for reusable evidence from the same source SHA before broad or expensive tests.
7. Update `memory/CHECKPOINT.md` after a substantial unit, before a handoff, or when the next action or evidence validity changes.

Operating boundaries:

- `memory/CHECKPOINT.md` is the sole current session-resume coordinate. Live Git and current runtime observations win if it is stale.
- `memory/DECISIONS.md` records only user-confirmed decisions and append-only supersession history.
- `memory/OPEN-QUESTIONS.md` records active pending choices; it is not proof of project state.
- `docs/HANDOFF.md` is preserved as historical/domain reference and is not maintained as a competing per-session checkpoint after cutover.
- `docs/planning/STATUS.md` remains the milestone/evidence router and changes only when actual project state changes.
- Evidence, ADRs, architecture, specifications, validation policy, milestones, and lessons retain authority in their own domains. Memory links to them and does not copy or replace their proof.
- A docs-only or memory-only change does not trigger Rust/GPU FULL, app smoke, scenario candidates, official capture, or user acceptance.
- Reuse a result only within its stated source SHA, command, toolchain/profile, relevant configuration, hardware/backend, artifact identity, and invalidation conditions.
- Do not promote an agent proposal, implementation observation, or pending item into a decision without explicit user confirmation.
- Commit, push, merge, destructive cleanup, and external review retain existing approval rules.

Emergency stop and rollback are defined in `docs/development/BALLAST_MEMORY_CUTOVER.md`. The fastest stop is `BALLAST_DISABLE=1` or removing trust from the Ballast Hook; this disables rule injection without changing Git files.
<!-- END managed: ballast-project-memory -->