# Decision ledger

Entries are append-only. Record only choices explicitly confirmed by the user or clearly adopted by an authoritative Powdergame document. A changed decision gets a new sequential ID that supersedes the old entry; never erase or silently rewrite history.

## D-001 · Use a bounded isolated Ballast memory pilot — 2026-08-19 (source: user Stage 6 instruction)

Decision: Opt the Powdergame pilot branch into a six-file memory layer in a separate sibling worktree. The layer indexes and reconnects existing Powdergame authority; it does not copy or supersede architecture, status, evidence, validation, or handoff documents, and it does not import uncommitted work from another worktree.

Reason: Evaluate durable session return and evidence reuse without disturbing ongoing Powdergame development or creating a competing source of truth.

Scope: `agent/ballast-memory-pilot`, based on `feature/m0-g8b-scenario-suite` at `e43078737712862c9cc6ccdc4b7e56475bafc6ce`.

Evidence: User Stage 6 instruction; independently verified Ballast workflow from Wiki commit `318276eebfbf913638d72f5d218ead2450361a01`. Implementation outputs: `AGENTS.md` and `memory/00-INDEX.md`.

Invalidated by: Explicit user supersession, rejection of the pilot, or replacement by a separately approved memory contract.

## D-002 · Keep the pilot docs-only and reuse valid exact-source evidence — 2026-08-19 (source: user Stage 6 instruction and adopted validation policy)

Decision: Changes limited to the pilot agent/memory documents use docs-only validation. They do not trigger Rust/GPU FULL, application smoke, experiment/fixture candidates, official capture, or user acceptance. Existing results may be reused only for their exact source SHA, command, toolchain/profile, relevant configuration, and hardware/backend conditions.

Reason: The pilot changes navigation and durable context, not runtime source, fixtures, harnesses, or evidence artifacts.

Scope: This Ballast pilot and later docs-only updates that leave the cited evidence inputs unchanged.

Evidence: `docs/development/VALIDATION_POLICY.md`; `docs/development/LESSONS_LEDGER.md` entries PG-L001 and PG-L005; user Stage 6 safety boundary.

Invalidated by: A runtime source, fixture, test/capture implementation, relevant configuration, toolchain/profile, or claim-relevant environment change; incomplete/failed evidence; explicit user rerun request; or user supersession of this decision.
