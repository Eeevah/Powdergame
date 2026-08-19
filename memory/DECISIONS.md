# Decision ledger

Entries are append-only. Record only choices explicitly confirmed by the user or clearly adopted by an authoritative Powdergame document. A changed decision gets a new sequential ID that supersedes the old entry; never erase or silently rewrite history.

## D-001 · Use a bounded isolated Ballast memory pilot — 2026-08-19 (source: user Stage 6 instruction)

Decision: Opt the Powdergame pilot branch into a six-file memory layer in a separate sibling worktree. The layer indexes and reconnects existing Powdergame authority; it does not copy or supersede architecture, status, evidence, validation, or handoff documents, and it does not import uncommitted work from another worktree.

Reason: Evaluate durable session return and evidence reuse without disturbing ongoing Powdergame development or creating a competing source of truth.

Scope: `agent/ballast-memory-pilot`, based on `feature/m0-g8b-scenario-suite` at `e43078737712862c9cc6ccdc4b7e56475bafc6ce`.

Evidence: User Stage 6 instruction; independently verified Ballast workflow from Wiki commit `318276eebfbf913638d72f5d218ead2450361a01`. Implementation outputs: `AGENTS.md` and `memory/00-INDEX.md`.

Invalidated by: Superseded by D-004 after the user accepted the pilot as the active workflow.

## D-002 · Keep memory-only changes docs-only and reuse valid exact-source evidence — 2026-08-19

Decision: Changes limited to agent/memory documents use docs-only validation. They do not trigger Rust/GPU FULL, application smoke, experiment/fixture candidates, official capture, or user acceptance. Existing results may be reused only for their exact source SHA, command, toolchain/profile, relevant configuration, hardware/backend, artifact identity, and invalidation conditions.

Reason: Project memory changes navigation and durable context, not runtime source, fixtures, harnesses, or evidence artifacts.

Scope: Ballast project memory and later docs-only updates that leave cited evidence inputs unchanged.

Evidence: `docs/development/VALIDATION_POLICY.md`; adopted lessons PG-L001 and PG-L005; user safety boundary.

Invalidated by: A runtime source, fixture, test/capture implementation, relevant configuration, toolchain/profile, claim-relevant environment, or authenticated artifact change; incomplete/failed evidence; explicit user rerun request; or a later user supersession.

## D-003 · Accept Heavy Mixed World with a known follow-up — 2026-08-19

Decision: Record Heavy Mixed World as `USER ACCEPTED WITH KNOWN FOLLOW-UP`. Preserve automatic `NEEDS_HUMAN_REVIEW`, all 14 hard-predicate passes, `candidate_blocker=false`, the immutable source/run/artifacts, and the declining `broad_terminal_tail` as a non-blocking observation. Do not classify it as a production-physics defect and do not rerun or retune the candidate.

Reason: Matter movement, density displacement, phase work, combustion/Smoke, Pressure activity, meaningful multi-system overlap, inventory accounting, no invalid/non-finite/wake anomaly, no runaway, and exact reset all passed. The remaining broad terminal activity is a declining Thermal tail whose workload cost belongs in G8-C.

Scope: Heavy Mixed candidate source `07260fffab22e5b4513eb168f0baac36e374ab94`, run `g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4`, Receipt SHA-256 `2abebdef7f9174e63abfd9c67ce4a48d24b48edde4e6c29fab49022e36a2dbd1`.

Evidence: User visual review and the canonical Heavy Mixed evidence record.

Invalidated by: A later explicit user supersession or authenticated evidence showing the accepted candidate identity/result was incorrect. A new source or new run is a different claim, not an automatic invalidation of this historical disposition.

## D-004 · Adopt Ballast as Powdergame's primary project memory — 2026-08-19

Decision: Supersede the isolated-pilot operating model. Ballast becomes the single active session-continuity workflow for Powdergame:

1. `memory/00-INDEX.md`
2. `memory/CHECKPOINT.md`
3. active entries in `memory/DECISIONS.md`
4. only the task-relevant canonical project documents linked by the index

`memory/CHECKPOINT.md` is the sole current resume coordinate. `docs/HANDOFF.md` is preserved as historical/domain reference and is not maintained in parallel as a per-session checkpoint. `docs/planning/STATUS.md`, evidence, ADRs, architecture, specs, milestones, validation policy, and lessons remain authoritative within their domains.

Reason: Maintaining multiple live session-memory paths created repeated context reconstruction and stale-state propagation. The isolated pilot passed its bounded docs-only validation and the user chose the single-workflow cutover.

Scope: Powdergame after commit-preserving integration of PR #4. It does not apply retroactively to rewrite immutable evidence or completed run identities.

Evidence: Explicit user adoption after the pilot; PR #4; `docs/development/BALLAST_MEMORY_CUTOVER.md`.

Invalidated by: Explicit user rollback or supersession, or a rollback trigger in D-005 that the user decides is severe enough to revert the cutover.

## D-005 · Preserve a reversible Ballast cutover — 2026-08-19

Decision: Keep the pilot initialization and active cutover as separate commits and forbid squash merge. The fastest incident response is `BALLAST_DISABLE=1` or removing Hook trust. Project rollback then reverts the active cutover commit first and `ba2b6406f6605882c51886b0a50bc64d10990a7f` second. Existing domain documents are preserved, so rollback restores the previous resume model rather than reconstructing it from memory.

Reason: Powdergame needs one active memory workflow during normal development without making the adoption irreversible or coupling it to product/evidence decisions.

Scope: Project AGENTS/memory/cutover documents and their merge history. Heavy Mixed acceptance and other canonical product decisions must remain outside the rollback unit.

Evidence: User-approved rollback requirement; `docs/development/BALLAST_MEMORY_CUTOVER.md`; Wiki Ballast workflow.

Invalidated by: A later explicit user decision that adopts a different rollback mechanism after the commit history has been migrated safely.