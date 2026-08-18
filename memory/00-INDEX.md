# Powdergame memory index

This directory is the bounded Ballast pilot map. It reconnects a session to Powdergame's existing sources of truth without copying or replacing them. Current source and live runtime observations remain authoritative for observed facts; the documents below retain authority in their stated domains.

## Resume order

1. Read this index.
2. If `memory/HANDOFF.md` exists, consume its single-use instruction before relying on the checkpoint.
3. Read `memory/CHECKPOINT.md`.
4. Read active entries in `memory/DECISIONS.md`.
5. Open only the project documents and evidence relevant to the current task.

## File map

| Path | Purpose | Authority | When to read | When to update |
|---|---|---|---|---|
| [`memory/CHECKPOINT.md`](CHECKPOINT.md) | Thirty-second return point and one next action | Current Ballast session state; live Git wins if coordinates drift | Every resume | After a substantial unit, before handoff, or when validity/next action changes |
| [`memory/DECISIONS.md`](DECISIONS.md) | User-confirmed decisions and supersession history | Adopted decisions within their recorded scope | Every resume; before reopening a decision | Append when the user confirms or supersedes a decision; never silently rewrite |
| [`memory/OPEN-QUESTIONS.md`](OPEN-QUESTIONS.md) | Active unresolved questions and closing links | Register of pending questions, not proof of project state | When a task depends on a pending choice | Add only sourced active questions; close by appending durable evidence |
| [`memory/SESSION-LOG.md`](SESSION-LOG.md) | Terse pilot audit trail | Historical session record | When reconstructing what this pilot changed or verified | Append a compact entry after a meaningful session |
| [`docs/HANDOFF.md`](../docs/HANDOFF.md) | Powdergame execution handoff and recovery narrative | Authoritative handoff; use current status links rather than older snapshots | Before substantive Powdergame work | When the canonical recovery/current execution handoff changes |
| [`docs/planning/STATUS.md`](../docs/planning/STATUS.md) | Current milestone, gate, verdict, evidence router, and next action | Final authority for current project state | Every substantive resume | When current state, acceptance, blockers, or next action changes |
| [`docs/development/VALIDATION_POLICY.md`](../docs/development/VALIDATION_POLICY.md) | Change-impact validation and same-SHA reuse rules | Validation authority | Before any validation or evidence reuse | When validation classes, reuse keys, or invalidation rules change |
| [`docs/development/DEVELOPMENT_LEARNING_LOOP.md`](../docs/development/DEVELOPMENT_LEARNING_LOOP.md) | Observe-to-promote workflow and stale-reference sweep | Process authority | When recording or promoting a recurring lesson | When the learning/promotion workflow changes |
| [`docs/development/LESSONS_LEDGER.md`](../docs/development/LESSONS_LEDGER.md) | Append-only adopted Powdergame lessons | Durable adopted lessons; not a replacement for decisions | Before repeating a known failure or changing policy | Append only after a lesson meets the promotion rule |
| [`docs/development/QUICKSTART.md`](../docs/development/QUICKSTART.md) | Developer entry points and commands | Operational guide; `STATUS.md` wins for newer state | Before using a launcher or development entry point | When supported entry points or its current summary changes |
| [`docs/development/TESTING.md`](../docs/development/TESTING.md) | Test roles and evidence philosophy | Testing-scope authority; timing is governed by validation policy | When choosing what a test can prove | When test contracts or evidence roles change |
| [`docs/development/WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md`](../docs/development/WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md) | Worktree, launcher, executable, and artifact rules | Operational policy | Before creating/retiring worktrees, binaries, launchers, or artifacts | When those operating contracts change |
| [`docs/planning/ROADMAP.md`](../docs/planning/ROADMAP.md) | Long-term direction and ordering | Direction authority; not current-state authority | When choosing a future product path | When the adopted direction changes |
| [`docs/planning/MILESTONES.md`](../docs/planning/MILESTONES.md) | Evidence Gate and user-approval criteria | Gate-completion authority | Before claiming a gate or milestone complete | When gate contracts or closure order changes |
| [`docs/evidence/G8_A_MEASUREMENT_SUBSTRATE_2026-08-17.md`](../docs/evidence/G8_A_MEASUREMENT_SUBSTRATE_2026-08-17.md) | G8-A v5 capture and verifier record | Exact-source technical evidence, not user acceptance | When using G8-A calibration claims | Append/supersede only through the evidence process; never rewrite artifacts |
| [`docs/evidence/G8_B_BENCHMARK_SCENARIO_GALLERY_2026-08-17.md`](../docs/evidence/G8_B_BENCHMARK_SCENARIO_GALLERY_2026-08-17.md) | Shared fixture/Gallery architecture and its earlier closure snapshot | Architecture/implementation evidence; its status prose is historical where `STATUS.md` is newer | When working on shared staging or Gallery contracts | Through a new evidence closure when those contracts change |
| [`docs/evidence/G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md`](../docs/evidence/G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md) | Sand immutable pilot receipt and review approval | Exact-source/run evidence | When using Sand pilot claims | Never mutate the run; add a new evidence record for a changed claim |
| [`docs/evidence/G8_B_WATER_FLOW_HARNESS_CANDIDATE_2026-08-17.md`](../docs/evidence/G8_B_WATER_FLOW_HARNESS_CANDIDATE_2026-08-17.md) | Water immutable candidates and accepted remediation | Exact-source/run evidence and recorded human disposition | When using Water acceptance or known-follow-up claims | Preserve old runs; add a new record for new evidence |
| [`docs/evidence/G8_B_FIRE_HEAT_HARNESS_CANDIDATE_2026-08-17.md`](../docs/evidence/G8_B_FIRE_HEAT_HARNESS_CANDIDATE_2026-08-17.md) | Fire sealed candidate, verification, and acceptance | Exact-source/run evidence and recorded human disposition | When using Fire claims | Preserve immutable artifacts; add a new record for changed evidence |
| [`docs/evidence/G8_B_PRESSURE_BURST_HARNESS_CANDIDATE_2026-08-18.md`](../docs/evidence/G8_B_PRESSURE_BURST_HARNESS_CANDIDATE_2026-08-18.md) | Pressure causal remediation, immutable candidate, and acceptance | Exact-source/run evidence and recorded human disposition | When using Pressure claims | Preserve historical/rejected/accepted runs; add new evidence rather than rewrite |
| [`docs/evidence/G8_B_HEAVY_MIXED_WORLD_HARNESS_CANDIDATE_2026-08-19.md`](../docs/evidence/G8_B_HEAVY_MIXED_WORLD_HARNESS_CANDIDATE_2026-08-19.md) | Current Heavy Mixed immutable candidate and manual-review contract | Exact-source/run machine evidence; user acceptance remains pending | First for the current G8-B next action | Preserve the candidate; update project state only after explicit user disposition |

## Authority notes

- `docs/planning/STATUS.md` is the current evidence router. Older status prose in Quickstart, Roadmap, Gallery, or scenario closure records remains historical unless brought current there.
- A receipt proves only its exact source/run contract. It does not silently transfer to this pilot's docs commit or close a user-approval gate.
- Do not store credentials, raw authentication material, generated artifacts, or copied evidence in `memory/`.
