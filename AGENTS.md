# Powdergame agent instructions

<!-- BEGIN managed: ballast-project-memory -->
## Ballast project memory pilot

This repository has opted into a bounded, file-based memory pilot. The pilot is a navigation and return layer; it does not replace Powdergame's architecture, status, evidence, validation, or handoff documents.

For a new or reset session:

1. Read `memory/00-INDEX.md`.
2. Read `memory/CHECKPOINT.md`.
3. Read active entries in `memory/DECISIONS.md`.
4. Follow the authoritative Powdergame documents linked by the index.
5. Read `docs/development/VALIDATION_POLICY.md` before running any validation.
6. Search for reusable evidence from the same source SHA before broad or expensive tests.
7. Update `memory/CHECKPOINT.md` when a substantial unit ends or the session winds down.

If `memory/HANDOFF.md` is ever created, treat it as single-use: read and absorb it after opening the index and before relying on the checkpoint, then delete it after acting on or registering its instruction.

- Current source and live runtime observations remain authoritative for observed facts. `memory/` does not supersede existing architecture, status, evidence, validation, or `docs/HANDOFF.md` authority.
- A docs-only or agent-memory change does not trigger Rust/GPU FULL, app smoke, experiment candidates, fixture candidates, official capture, or other runtime reruns.
- Record only user-confirmed decisions. Do not silently promote agent proposals, implementation observations, filenames, or pending items into decisions.
- Reuse a result only within its stated source SHA, command, toolchain/profile, relevant configuration, and hardware/backend validity conditions.
- Commit, push, merge, destructive cleanup, and external review retain their existing approval rules.
<!-- END managed: ballast-project-memory -->
