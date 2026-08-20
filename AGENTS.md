# Don’t reinvent the wheel

This is a **primary operating principle** for every Powdergame task.

Before designing or implementing anything:

1. Search for reuse in this order: existing Powdergame code, documents, tools, skills, fixtures, and evidence; established algorithms and standards; maintained Rust crates and libraries; relevant open-source simulation engines, powder sandboxes, GPU implementations, UI systems, and prior art.
2. Prefer reuse, composition, adaptation, or a thin wrapper over a from-scratch implementation. Build a new subsystem from the ground up only when no suitable option exists or a concrete Powdergame constraint rules reuse out.
3. Do not import blindly. Check license and provenance, maintenance health, security, API stability, Windows/wgpu/DX12 compatibility, GPU cost, determinism, architecture fit, evidence impact, and rollback cost. Pin a version or commit when reproducibility matters.
4. Reuse the useful mechanism, not accidental baggage. External code must not override Powdergame’s source of truth, GPU-authoritative world, One Cell = Max One Matter, local-rule architecture, product intent, evidence boundaries, or current Gate scope.
5. In implementation plans and final reports, state what was found, what was reused or adapted, what was rejected and why, and what truly had to be created.

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
- A docs-only or memory-only change does not trigger Rust/GPU FULL, a bounded app launch check, scenario candidates, official capture, or user acceptance.
- Reuse a result only within its stated source SHA, command, toolchain/profile, relevant configuration, hardware/backend, artifact identity, and invalidation conditions.
- Do not promote an agent proposal, implementation observation, or pending item into a decision without explicit user confirmation.
- Commit, push, merge, destructive cleanup, and external review retain existing approval rules.

Emergency stop and rollback are defined in `docs/development/BALLAST_MEMORY_CUTOVER.md`. The fastest stop is `BALLAST_DISABLE=1` or removing trust from the Ballast Hook; this disables rule injection without changing Git files.
<!-- END managed: ballast-project-memory -->

## Powdergame terminology guard

Powdergame contains a real registered Matter named **Smoke**, so software-validation terminology must not reuse that word ambiguously.

- In new prompts, reports, checkpoints, policies, and user-facing prose, do **not** use the bare phrase `smoke test`.
- Call the short software check a **bounded launch check** or **application startup check**. It verifies binary startup, GPU/renderer initialization, requested mode loading, a bounded number of frames/ticks, and a clean exit.
- Use **Smoke Matter behavior**, **Smoke generation/decay**, or similarly explicit wording when validating the in-game Matter produced by combustion.
- The legacy CLI option `--smoke-frames` and historical machine fields may remain for compatibility. Describe that option as a bounded launch-frame limit; it does not test Smoke Matter.
- Do not rewrite immutable historical evidence merely to change terminology. Interpret older unqualified software uses according to D-009 and `docs/development/VALIDATION_POLICY.md`.