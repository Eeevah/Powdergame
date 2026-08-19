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

## D-006 · Approve the G9 First Playable product brief — 2026-08-19

Decision: Follow the official G8-C recommendation and authorize the bounded G9 product brief in `docs/planning/G9_PRODUCT_BRIEF_2026-08-19.md`.

Approved scope:

1. start in an editable Starter Lab and expose `New Blank World` immediately;
2. show and allow all current M0 Matter from the beginning; Discovery records phenomena and does not unlock Matter;
3. implement the first G9-A editor set: Matter selection, Draw, Erase, brush size, Heat, Cool, Pause/Play, Single Step, x1/x4/x16, Reset, Pan, Zoom, preset load and the existing Cell Inspector;
4. add a phenomenon-level Research Note after the editor core works, within the same G9 milestone;
5. defer Save/Load and Rewind from the first acceptance slice while preserving Rewind as a later core experiment tool;
6. use an approximately 10–15 minute unguided user session for final product validation, with a voluntary second experiment and causal explanation as primary strong signals;
7. implement G9-A first and stop at a user-testable candidate before expanding the scope.

Reason: G8-C found no simulation, rendering, coexistence or persistent-memory blocker. The project's remaining risk is product interaction and the desire to run another experiment, not more pre-emptive optimization.

Scope: G9 product direction and the first implementation line `feature/m0-g9-first-playable`. This does not authorize new Matter, unlock progression, Save/Load, Rewind, final FX, optimization, `main` promotion or M0 `ACHIEVED`.

Evidence: Explicit user response “추천대로 진행”; G8-C official Matrix `g8c-official-matrix-4653d7c2e09e-64df60ba0d79`; `docs/vision/FIRST_PLAYABLE_WORLD.md`; `docs/vision/UI_DIRECTION.md`; canonical product brief.

Invalidated by: A later explicit user change to the G9 product brief or direct product testing that shows the approved slice is materially wrong.

## D-007 · Supersede the separate G8-A visual requirement and close G8 — 2026-08-19

Decision: Preserve G8-A v5 as verified technical evidence, but formally supersede its separate same-SHA user visual durable requirement. Do not call that old visual requirement a retroactive `PASS`. Use the later direct G8-B Gallery/Cell Inspector user approvals plus independently verified G8-C windowed evidence as the broader observation/product record, and close G8 as `CLOSED / FROZEN`.

Reason: The old requirement concerned an additional durable visual disposition for one historical calibration capture. The user has since directly reviewed richer production scenarios and the Cell Inspector, while G8-C independently verified simulation/render coexistence on the five accepted workloads. Replaying the older visual session would not add a product decision worth delaying G9.

Scope: G8-A source `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`, capture `g8a-v5-9abec9e-20260817T032827206Z`, later G8-B user dispositions and G8-C Matrix source `4653d7c2e09e93f80fb81eeb73458d992c86858f`. Canonical closure: `docs/evidence/G8_PERFORMANCE_GATE_USER_CLOSURE_2026-08-19.md`.

Evidence: Explicit user selection of recommendation C; G8-A official capture/verification; G8-B user acceptance records; G8-C official independent verification.

Invalidated by: A later explicit user decision that reopens the old visual requirement or authenticated evidence that invalidates one of the recorded identities. A future source is a new claim and does not rewrite this historical closure.

## D-008 · Use merge-based rollback after Ballast integration — 2026-08-19

Decision: Supersede D-005's pre-integration two-commit rollback sequence now that PR #4 has been integrated. Immediate disable remains `BALLAST_DISABLE=1` or Hook untrust. Project rollback reverts Ballast-only commits newest-first, then reverts merge commit `6b5f0201f882f212f9916521aec689261d97b4a6` with `git revert -m 1`.

Reason: The merge commit preserves the G8-C product/evidence line as first parent and the complete Ballast history as second parent. Reverting that merge cleanly removes the project opt-in while preserving product/evidence commits.

Scope: Powdergame's integrated Ballast history after merge `6b5f0201f882f212f9916521aec689261d97b4a6`. Later Ballast-only checkpoint commits must be reverted newest-first before the merge revert.

Evidence: User-approved reversible integration; Powdergame PR #4 merge; `personal-infra-wiki` decision and rollback troubleshooting updated through Wiki PR #40.

Invalidated by: A later explicit user decision that adopts another audited rollback mechanism.