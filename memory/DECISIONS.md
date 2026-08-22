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

## D-009 · Reserve Smoke for the in-game Matter and rename software checks in prose — 2026-08-19

Decision: In new Powdergame prompts, reports, checkpoints, policy text and user-facing explanations, do not use the bare phrase `smoke test` for software validation. Call the short binary startup/mode/bounded-exit validation a **bounded launch check** or **application startup check**. Reserve **Smoke** for the registered in-game Matter and use explicit phrases such as `Smoke generation/decay validation` when testing it.

The legacy CLI option `--smoke-frames` and historical machine/evidence vocabulary may remain for compatibility and provenance. Current prose must describe that option as a bounded launch-frame limit and must never imply that it measures Smoke Matter.

Reason: Powdergame contains actual combustion-generated Smoke. Reusing the same word for Codex's frequent short application checks caused a plausible category error between software validation and game behavior.

Scope: Powdergame project memory, AGENTS instructions, active validation policy, new prompts/reports and future user-facing documentation. Immutable historical evidence is not rewritten solely for terminology.

Evidence: Explicit user clarification that Codex's frequent software checks are not measurements of in-game Smoke.

Invalidated by: A later explicit user terminology decision.

## D-010 · Make Sandbox the canonical no-argument G9-A candidate — 2026-08-19 (source: direct user candidate feedback)

Decision: Double-clicking `run_powdergame.bat` and launching the canonical `powdergame-windows.exe` without arguments must open the G9-A Starter Lab Sandbox. Keep `sandbox` and `play` as explicit aliases, and preserve the frozen G8-B Gallery through explicit `gallery` / `normal` / `--benchmark-gallery` routes.

Reason: The argument-only candidate technically existed but the actual user-facing BAT still opened the G8 Gallery, so it did not deliver the requested immediately runnable first-playable executable experience.

Scope: G9-A launch UX on `feature/m0-g9-first-playable`. This refines D-006 without authorizing G9-B/C/D/E, Discovery, Save/Load, Rewind, optimization, main promotion or M0 closure.

Evidence: Direct user feedback “run bat 실행했는데 g8이랑 똑같잖아” and “실행파일을 만들어놓으라고 했잖아”; source `0d03dafbb4bc6375adc10c8b819db6c0bc232db9`; no-mode 3-frame release bounded launch check on the canonical EXE.

Invalidated by: A later explicit user decision selecting a different canonical default after direct product review.

## D-011 · Revise the first G9-A candidate after direct user review — 2026-08-20 (source: direct user review handover)

Decision: Record G9-A as **USER REVIEWED / REVISION REQUIRED** and revise only five user-identified interaction details: Draw writes only to EMPTY cells; direct Ice and Steam placement uses stable phase-safe default temperatures; the always-available nine-Matter palette separates Generated and Advanced entries from the core choices; the Inspector may retain only explicitly identified prior Cell/Material identity for a bounded 100–200 ms grace period while fresh data is pending and must not reuse old field values as current; Heat/Cool gain presentation-only brush/application feedback derived from the existing tool command and camera transform.

Reason: The canonical Sandbox executable was directly reviewed and the editor core worked, but these five details made placement, palette comprehension, Inspector continuity and thermal-tool feedback less clear than required for acceptance.

Scope: G9-A remediation on `feature/m0-g9-first-playable`. Completion returns to **REVISED IMPLEMENTATION CANDIDATE / USER RE-REVIEW PENDING**. G9-B/C/D/E, Ash, Discovery, Save/Load, Rewind, optimization, `main` promotion, PR creation and M0 closure remain not started or unauthorized.

Evidence: Direct user handover on 2026-08-20; last validated runtime implementation source `0d03dafbb4bc6375adc10c8b819db6c0bc232db9`.

Invalidated by: A later explicit user re-review disposition or a superseding interaction contract.

## D-012 · Require Inspector continuity v2 and register thermal causality work — 2026-08-20 (source: direct user re-review)

Decision: Record the revised G9-A candidate as **USER RE-REVIEWED / REVISION REQUIRED** because Cell Inspector continuity still visibly flickers during rapid hover movement. Replace the identity-only grace with a bounded presented-sample hold: requested hover and presented sample remain separate; the held sample keeps its original Cell, Material, simulation tick and freshness identity; compact cursor copy appears only for a fresh current-Cell sample; the detailed panel keeps fixed geometry and becomes a stable Sampling state after timeout. Separately register **Thermal Transport & Ignition Causality** as **PLANNED / DESIGN REQUIRED / IMPLEMENTATION NOT STARTED** and a prerequisite to G9-B emergence validation.

Reason: Direct re-review confirmed that Draw, Ice/Steam defaults, palette grouping and Heat/Cool feedback were present, but Inspector on/off flicker remained. The same review observed that heat does not cross EMPTY and that hot Stone can cause near-immediate adjacent Oil/Wood ignition, requiring a dedicated design project rather than an unreviewed engine tweak.

Scope: Inspector continuity only in the G9-A Windows presentation/readback consumer, plus docs/memory registration of thermal design work. Do not change engine/Core, production WGSL, shared Simulation state, thermal behavior or fixtures in this task. G9-B/C/D/E, Ash, Discovery, Save/Load, Rewind, optimization, main promotion, PR creation and M0 closure remain not started or unauthorized.

Evidence: Direct user re-review on 2026-08-20; reported runtime source `b363c078fdc1d7e8b54fa6be328b7a0c5b908f06`; reported docs closure `ae49e7c60c29a7c0478215e36fab5010c11d9b3c`.

Invalidated by: A later explicit user disposition after continuity v2 re-review, or an approved thermal architecture decision.

## D-013 · Separate foreground Matter from atmospheric and vacuum Environment — 2026-08-20 (source: direct user TE-0R / TE-0 design-lock authorization)

Decision: Authorize the Thermal Environment foundation design program while leaving production implementation not started. Preserve `EMPTY` as no foreground Matter, but distinguish Atmospheric Empty, Vacuum and out-of-domain Void through a separate Environment Field. Air is not a Material ID, Registry entry, palette Matter, density-displacement Matter, or same-cell mixture. The canonical future state is `air_mass_current/next` plus `air_energy_current/next`; Air temperature and background pressure are derived. Occupied Matter Cells keep Air state canonical zero. Adopt Celsius-like gameplay anchors of about 20 for room temperature, 0 for Water/Ice and 100 for Water/Steam, while requiring one atomic later migration of all current thermal constants and UI labels. Use reuse-first research and import no external code blindly. TE-0 must close design, independent review and reference math before TE-1 runtime work.

Reason: Direct review showed that the no-hidden-Air runtime cannot transport heat through ordinary open space and that threshold-only ignition is not sufficiently legible. A separate mass/energy Environment distinguishes Atmosphere from Vacuum, transports energy with Air amount, preserves One Cell = Max One Matter, and creates a common-rule foundation for later open-space cooling, condensation, sealed heating and Vacuum experiments without inventing an Air Matter or CFD solver.

Scope: Thermal Environment TE-0R/TE-0/TE-0A/TE-0B on `feature/m0-g9-first-playable`. This closes the architecture choice that D-012 intentionally left open and supersedes the Option A/Option B selection state in `docs/planning/THERMAL_TRANSPORT_IGNITION_CAUSALITY.md`; it does not rewrite D-012. G9-A remains **REVISED IMPLEMENTATION CANDIDATE / USER RE-REVIEW PENDING**. G9-B/C/D/E, production Thermal Environment, Vacuum tooling, Oxygen, Ash, CFD, optimization, main promotion and M0 closure remain not started or unauthorized.

Evidence: User-provided TE-0 design-lock prompt; `docs/architecture/decisions/ADR-0005-atmosphere-vacuum-environment.md`; verified design package SHA-256 `3f5abcc282881a190a0881499b61f8d00cf782f305354f6b533740ac0d0b9c84`; live production inventory at source `f5c7ac8e76867f769cdf19d7f420432d8fef4509`.

Invalidated by: A later explicit user decision selecting a different Environment ontology or abandoning the Thermal Environment program. Later coefficient, edge, combustion-support, phase and permeability decisions refine this architecture only through their own sequential decision entries.

## D-014 · Authorize and implement the TE-1 Environment state/occupancy foundation — 2026-08-20 (source: direct user TE-1 implementation authorization)

Decision: Implement only TE-1 from the locked D-013/ADR-0005 architecture: four full-resolution Air mass/energy Current/Next buffers, one reusable u32 receiver-claim scratch, exact Atmosphere/Vacuum initialization and reset, occupancy-linked Volume Exchange, deterministic receiver-gated phase/Smoke spawn, exactly-once Environment-blocked phase pressure, Matter flag identity hygiene, bounded test observation, and explicit allocation/profiler accounting. Do not implement inter-cell Air transport, Matter↔Air or Air↔Air thermal exchange, Air-pressure coupling, TE-2, later thermal/ignition gates, G9-B/C/D/E, optimization, Vacuum tooling, Oxygen or Ash.

Reason: The user explicitly authorized the bounded TE-1 foundation after TE-0R/TE-0/TE-0A/TE-0B closed its architecture and blockers. Preserving whole Air parcels and transactional occupancy changes creates the required state foundation without silently starting the later physics.

Scope: Runtime/test source `1a722d239a16bade5772688fa822465d5cef4602` on `feature/m0-g9-first-playable`; canonical TE-1 docs and evidence. G9-A remains **REVISED IMPLEMENTATION CANDIDATE / USER RE-REVIEW PENDING** and Q-008 remains open for its named later gates.

Evidence: `docs/evidence/THERMAL_ENVIRONMENT_TE_1_FOUNDATION_2026-08-20.md`; final-source serial workspace FULL PASS; release Sandbox bounded launch PASS; exact allocation/profiler/receiver transaction tests.

Invalidated by: A later explicit user decision changing the Environment ontology or a reproducible invariant failure at the named runtime source. This decision does not authorize TE-2.

## D-015 · Implement the TE-2 passive thermal Environment candidate — 2026-08-20 (source: direct user TE-2 authorization)

Decision: Implement TE-2 only on the locked D-013/D-014/ADR-0005 foundation:
full-resolution every-tick pressure-derived Air flow, donor-energy advection,
unified Matter↔Matter/Air↔Air/Matter↔Air passive exchange, bilateral
activity/wake, sealed correctness edges, an explicit fixed-reservoir fixture,
and one atomic Celsius-like gameplay migration. The thermal deadband is a
shared work/no-work gate: `abs(delta) <= 0.01 °C` performs no work; otherwise
the full delta enters the existing stability-bounded formula. It is never
subtracted from the eligible delta.

Reason: TE-1 established lossless Environment ownership but intentionally left
open space thermally inert. TE-2 supplies the minimum passive transport needed
to distinguish Atmosphere from Vacuum while preserving GPU authority,
source-free accounting, One Cell = Max One Matter and the eight-storage limit.
The user-supplied Chinese-community research added the bounded
`SMALL_DELTA_THERMAL_CONVERGENCE` regression before the final source freeze;
no external simulation code was copied.

Scope: Runtime/test source `fb7e568e21012b6067269f4e1b82c36c865023d0`
on `feature/m0-g9-first-playable`; canonical TE-2 docs and evidence. Completion
is **PASSIVE THERMAL ENVIRONMENT CANDIDATE / USER REVIEW PENDING**. Air-pressure
force, TE-3/TE-4, G9-B/C/D/E, Oxygen, Ash, new Matter, CFD and optimization
remain not started or unauthorized. D-013 and D-014 are not rewritten.

Evidence: `docs/evidence/THERMAL_ENVIRONMENT_TE_2_PASSIVE_TRANSPORT_2026-08-20.md`;
final-source serial workspace FULL; locked release bounded launch; 30-case
CPU/GPU small-delta convergence; one-shot 256²/2048² profiler measurement.

Invalidated by: A later explicit user decision changing TE-2 semantics, a
reproducible invariant failure at the named source, or a runtime source change
to the transport/deadband/activity contract. It does not authorize TE-3.

## D-016 · Accept G9-A continuity, require observable TE-2 review controls, and register the TE-3 phase blocker — 2026-08-20 (source: direct user review handover)

Decision: Record G9-A Inspector continuity v2 as **USER ACCEPTED** and G9-A
overall as **USER ACCEPTED WITH KNOWN FOLLOW-UP**. Record TE-2 as **USER
REVIEWED / REVISION REQUIRED** because F did not operate, N showed no fresh
tick-1 result, I was unavailable, temperature/Air measurements were not usable,
and scene 1 still looked unchanged after roughly 3000 ticks. Preserve D-015's
automated correctness, runtime-source, performance, sealed-edge/reservoir and
source-free evidence. Authorize only candidate controls, bounded diagnostics,
honest forced sampling and presentation/staging remediation; completion returns
TE-2 to **REVISED PASSIVE THERMAL ENVIRONMENT CANDIDATE / USER RE-REVIEW
PENDING**, not acceptance.

Separately register the observed Water/Steam mid-air checkerboard clumping and
the audited `1 Water -> up to 2 Steam -> up to 2 Water` closed-cycle quantity
gain as a required TE-3 design blocker. TE-3 must compare the named accounting
representations without selecting or implementing one in this task. It remains
**DESIGN REQUIRED / NOT STARTED**.

Reason: The TE-2 production rules could not be evaluated through the original
candidate surface even though their automated evidence remained valid. In
Sandbox, Air transport/cooling and Steam-to-Water causality were visible, but
the unpaired phase expansion/contraction accounting produced unnatural
blue/white mid-air clumps and can increase Water-equivalent Cell count through
a closed cycle.

Scope: Candidate remediation source
`097728128343cf89383920c968a010b3dcf8e8c0` on
`feature/m0-g9-first-playable`; TE-3 planning registration only. D-012 through
D-015 remain unchanged. Engine/Core, production compute WGSL, TE-2 coefficients,
phase/movement/ignition rules, Air-pressure force, TE-3 runtime, TE-4,
G9-B/C/D/E, optimization, main promotion and PR creation are excluded.

Evidence: Direct user disposition and observations; TE-2 runtime source
`fb7e568e21012b6067269f4e1b82c36c865023d0`; candidate remediation tests and
the canonical TE-3 planning record.

Invalidated by: A later explicit G9-A or TE-2 user disposition, authenticated
evidence that the recorded source/result is incorrect, or an explicit TE-3
architecture decision at its named gate. A future source is a new claim and
does not rewrite this historical review.

## D-017 · Accept TE-2 with known follow-ups and authorize TE-3D design review — 2026-08-20 (source: direct user acceptance and design authorization)

→ superseded in TE-3 architecture-disposition scope by D-018 (2026-08-21).
The TE-2 acceptance and its two known follow-ups remain active.

Decision: Preserve G9-A overall as **USER ACCEPTED WITH KNOWN FOLLOW-UP** and
record TE-2 as **USER ACCEPTED WITH KNOWN FOLLOW-UP**. The accepted direct
observations are that F, N and the candidate-only I diagnostics operate
correctly; scene 1 communicates Direct contact > Atmosphere gap > Vacuum gap;
scene 2 visibly and numerically shows sealed Atmosphere spreading into
connected Vacuum; scene 3 shows sealed thermal redistribution with no external
exchange; scene 4 explicitly shows fixed-reservoir Air mass, advected-energy
and passive-heat exchange; and reset plus candidate controls are understandable
and usable. The user authorizes moving to the next Gate. No additional
same-tick scene 3/4 comparison is required.

Keep two non-blocking follow-ups: `LONG_HORIZON_SEALED_AIR_DRIFT_BUDGET` for a
later bounded numerical budget after a tiny cumulative Air mass/energy drift
was observed over tens of thousands of sealed-corridor ticks, and
`TE2_CANDIDATE_HUD_LABEL_POLISH` to identify chamber/corridor totals as Air-only
and improve long boundary/description truncation. Neither authorizes coefficient
retuning, optimization or a TE-2 physics revision.

Authorize the TE-3D architecture research, design, independent adversarial
review and one pure reference-math proof around the named
**HYBRID A+C — 1:1 WATER-EQUIVALENT QUANTITY WITH DEDICATED PHASE ENTHALPY**
candidate. This is authorization to produce a **PHASE-ENTHALPY DESIGN
CANDIDATE / USER ARCHITECTURE REVIEW PENDING**, not acceptance of ADR-0006 and
not runtime authorization. ADR-0006 must remain `Proposed`; any unresolved
Critical or High counterexample stops TE-3D as **DESIGN BLOCKED**.

Reason: The revised candidate makes the already evidenced TE-2 physics directly
observable and usable, so the user can accept it without changing its
production source. The separate Water/Steam quantity-gain and mid-air traffic
jam remain TE-3 design problems. A 1:1 foreground quantity plus dedicated
Current/Next latent state is now the explicit candidate to attack before any
implementation decision.

Scope: TE-2 production physics
`fb7e568e21012b6067269f4e1b82c36c865023d0`, review-remediation source
`097728128343cf89383920c968a010b3dcf8e8c0`, and docs/memory-only TE-3D design
work on `feature/m0-g9-first-playable`. Preserve all TE-2 automated verdicts,
runtime identities, performance evidence and evidence boundaries. Rust, WGSL,
Cargo, phase/latent runtime, threshold or Air-coefficient retuning,
Air-pressure force, ignition, TE-4, G9-B/C/D/E, optimization, release/build/run,
`main`, PR and tag work remain excluded.

Evidence: Direct user observations and disposition; existing immutable TE-2
production/remediation evidence at their recorded sources. This docs decision
ran no Cargo, GPU, FULL, build, launch, TE-3 candidate or G8/G8-C validation.

Invalidated by: A later explicit user disposition, authenticated evidence that
the named TE-2 source/result is incorrect, or a later user acceptance/revision
of the proposed TE-3 architecture. A future runtime source is a new claim and
does not rewrite this acceptance.

## D-018 · Accept Hybrid A+C with locked TE-3D amendments — 2026-08-21 (source: direct user architecture disposition)

Supersession note: D-024 supersedes only this decision's atomic TE-3/TE-5
activation requirement. The accepted one-Cell/one-quantity phase-energy model
and its locked thermodynamic amendments remain active.

Decision: Accept the Hybrid A+C core for future atomic implementation: one
Ice/Water/Steam Cell equals one Water-equivalent quantity; every family phase
transition is 1:1; `phase_energy_current/next` are the only added persistent
phase state; there is no phase-quantity buffer, expansion fragment or same-cell
mixed Matter; phase normalization repartitions local H only after TE-2 Q and
never reapplies latent energy to a neighbour. Lock `Lf=80`, `Lv=480`,
`CONDENSATION_SURFACE_MAX_C=80`, `CONDENSATION_MIN_DELTA_C=10`,
`FREE_AIR_NUCLEATION_MAX_C=70`, the 32 MiB state increment at 2048² and atomic
activation with a separately approved same-source TE-5 pressure-volume
replacement. Explicitly accept that Steam with no positive-conductance
energy-removal face may remain metastable indefinitely and sleep.

The v1 ADR text is not accepted unchanged. Before ADR-0006 becomes future
implementation authority, lock these amendments: a condensation surface must
be a real positive-conductance TE-2 energy sink; buried partial boiling may
retain/increase/reverse E but Water→Steam completion requires a current
gas-facing surface or an explicit future TE-5 acceptance transaction; E=Lv
without either context is vaporization-ready Water that stores excess H as
Water superheat; free-air seed and active-partial veto radius is exactly two
Cells with the predeclared TE3-F08 30-tick initiation bound; generic non-family
`matter_yield>1` may not target Ice/Water/Steam without a later approved
destination phase-energy ownership/writer design; and the coordinate key must
reuse the existing internal arbitration finalizer with explicit provenance.

Reason: The core quantity, enthalpy, coefficient, memory and atomic-activation
choices are accepted, while the independent v1 review identified real but
closeable ambiguities around false sinks, buried completion, temporal
nucleation density, generic expansion targets and hash provenance. The locked
amendments close those issues without adding runtime state or importing a TE-5
pressure law.

Scope: Docs/reference-only TE-3D v2 work on
`feature/m0-g9-first-playable` from source
`b05b44207ecba1442b67dd1e80b1025590c08d60`. It does not authorize Rust,
WGSL, Cargo, phase runtime, TE-5 runtime, Air-pressure force, build, launch,
TE-4, G9-B/C/D/E, optimization, PR, main merge or release work. Runtime
activation remains atomic and separately authorized.

Evidence: Direct user TE-3D v2 disposition; amended ADR/spec/validation and
production inventory; a new fixed-seed radius-1/2/3 reference receipt; and a
fresh-context independent design review. If the new radius-2 proof or review
has an unresolved Critical/High finding, the docs task stops DESIGN BLOCKED
without silently selecting radius 3.

Invalidated by: A later explicit user supersession, a failed locked radius-2
hard property, an unresolved Critical/High amended-design counterexample, or
authenticated future implementation evidence that contradicts an invariant.
Such a failure blocks implementation authority; it does not silently restore
the rejected yield-2 quantity model or rewrite the preserved TE-2 acceptance.

### D-018 closure receipt — 2026-08-21

The amended fixed-seed proof passed its only run with script/result SHA-256
`c3624e467638a62ef2b62f96c8b12954ceef70609feeac47da70eca69f84db23` /
`f727101543f4eaa7582def01940e2567dd3b79bc6e585cfad4051160de1d90ea`.
The fresh independent v2 review SHA-256 is
`c6d63fd84d8057e6cbe201696df0a4914e1a396eaeaf2bc189a5ebcd24a9a31d`
with unresolved Critical `0` / High `0`. D-018's closure conditions are met:
TE-3D is **ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS** and ADR-0006 is
**ACCEPTED FOR FUTURE ATOMIC IMPLEMENTATION**. This receipt authorizes no
runtime work; TE-3 is **NOT STARTED** and the TE-5 bridge is **DESIGN REQUIRED
/ NOT STARTED**.

## D-019 · Authorize the TE-5B phase-volume bridge design program — 2026-08-21 (source: direct user TE-5B design authorization)

→ superseded in TE-5 pressure-volume design-selection scope by D-020
(2026-08-21). D-019 and ADR-0007 remain the immutable authorization and
blocked-candidate history.

Decision: Preserve TE-3D as **ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS**
and ADR-0006 as **ACCEPTED FOR FUTURE ATOMIC IMPLEMENTATION**. Authorize a
docs/reference-only TE-5B design program around the named primary candidate
**EXCLUSIVE LOCAL VOLUME-RELIEF TOKEN + EXISTING GAUGE PRESSURE**. The program
must compare the unconditional-impulse, non-exclusive-EMPTY, new-state and
exclusive-token options; reuse the existing GAS movement stencil,
proposal/claim ownership domain, Environment-receiver separation and gauge
pressure/rupture grammar; produce ADR-0007/spec/validation/planning/inventory,
one fixed-seed pure proof and a fresh-context adversarial review; and stop at
**PHASE-VOLUME BRIDGE DESIGN CANDIDATE / USER ARCHITECTURE REVIEW PENDING** if
unresolved Critical/High findings are zero. ADR-0007 remains **PROPOSED**.

Reason: D-018 requires an explicit, separately approved pressure-volume
transaction before the 1:1 Water/Steam path can activate without regressing
the frozen G5 boil/confinement/pressure/rupture/vent chain. The narrow bridge
must distinguish open relief from confinement and preserve same-tick exclusive
ownership without inventing a general pressure solver or extra phase quantity.

Scope: Docs, memory, research, pure reference proof and independent design
review on `feature/m0-g9-first-playable`, starting from
`d7500e219af6f670be05f830b50c232d2bb53077`. TE-3 runtime, TE-5B runtime, full
TE-5 background-pressure/structure coupling, Air force, product edge mode,
Vacuum combustion, rupture changes, TE-4, G9-B/C/D/E, optimization, build,
launch, PR and `main` merge remain unauthorized and **NOT STARTED**.

Evidence: Direct user authorization in the TE-5B Phase-Volume Relief /
Confinement Bridge task; D-018/ADR-0006 atomic-activation requirement; current
G5-B proposal/claim/consequence and G5-C gauge-pressure/rupture contracts at
the named start source. This entry records authorization only and is not proof
that ADR-0007 or the candidate has passed review.

Invalidated by: A later explicit user supersession; an unresolved Critical or
High design counterexample, which stops TE-5B as **DESIGN BLOCKED**; or future
source-bound implementation evidence that contradicts a locked invariant.

## D-020 · Reject TE-5B and authorize TE-5C local Vapor capacity-pressure design — 2026-08-21 (source: direct user replacement-design authorization)

→ superseded in TE-5 pressure-volume design-selection scope by D-021
(2026-08-21). D-020 and ADR-0008 remain the immutable authorization and
blocked-candidate history.

Decision: Reject the D-019 exclusive completion-token model as **REJECTED /
DESIGN BLOCKED** because same-tick exclusivity does not conserve finite
capacity across ticks. Preserve D-019 and proposed ADR-0007 as historical
blocked authority. Authorize the docs/reference-only TE-5C program **LOCAL
VAPOR CAPACITY SHARE + GAUGE-PRESSURE EQUILIBRIUM**. It must preserve one
phase-family Cell as one quantity, 1:1 transitions, no extra Steam, no target
Matter/Air mutation and no new persistent/full-world allocation. It replaces
completion-event pressure with a state-derived Vapor-volume pressure target.

Locked evaluation candidate: `Lv=480`; additional Vapor demand from accepted
phase energy; Chebyshev-radius-1 EMPTY capacity; the stated per-EMPTY
proportional sharing law; pressure target capped at `100.0` with full pressure
at compression `0.5`; existing diffusion `0.20`; EMPTY vent rate `0.20`; and
reuse of proposal as fully overwritten `f32` capacity-sum scratch after Smoke.
The formula may not be silently replaced after evidence is observed. Produce
ADR-0008/spec/validation/planning/inventory, one predeclared grid/time proof
executed exactly once and a fresh-context independent review.

Reason: TE-5B owned availability only inside one arbitration epoch; ordinary
1:1 Steam movement relocated the EMPTY vacancy and allowed every staggered
completion to receive zero pressure. A current-state population/capacity law
is the final authorized attempt to represent volume response without new
persistent phase-volume state.

Scope: Docs, memory and one external pure reference script/result only on
`feature/m0-g9-first-playable`, starting from
`6a1c83fad702d18f2d24365a4fc747ab74225f5c`. Rust, WGSL, Cargo, runtime,
build, launch, background-Air force, structure differential, TE-4,
G9-B/C/D/E, optimization, PR and `main` merge remain unauthorized and **NOT
STARTED**. Historical evidence stays source-bound.

Stop rule: This entry is authorization, not proof. Any Critical/High failure
in the locked proof or review stops **TE-5C DESIGN BLOCKED**. The next decision
must then explicitly permit persistent phase-volume state; no third stateless
token/impulse variant is allowed. With zero unresolved Critical/High findings,
the maximum stop is **LOCAL VAPOR CAPACITY-PRESSURE DESIGN CANDIDATE /
INDEPENDENT REVIEW PASS / USER ARCHITECTURE REVIEW PENDING**, ADR-0008 remains
**PROPOSED**, and runtime remains unchanged.

Invalidated by: A later explicit user supersession, failed locked grid/time
property, unresolved Critical/High review finding, or source-bound future
evidence contradicting an invariant. Failure does not revive ADR-0007 or
rewrite D-018.

## D-021 · Authorize TE-5D persistent Vapor extent and dedicated phase pressure — 2026-08-21 (source: direct user replacement-design authorization)

Decision: Preserve TE-5B and TE-5C as **REJECTED / DESIGN BLOCKED**. Together
their failures establish that same-tick ownership and stateless local sharing
do not provide cross-tick finite-capacity ownership. Authorize a
docs/reference-only TE-5D design program named **PERSISTENT VAPOR EXTENT +
DEDICATED PHASE PRESSURE**. This decision explicitly relaxes the prior
no-persistent-phase-volume-state rule and permits one Current/Next per-Cell
pair containing a reciprocal extent link and a dedicated phase-pressure
scalar. It also permits the reserved EMPTY extent target to carry canonical
zero Air while reserved.

Locked preservation boundary: one Ice/Water/Steam Cell remains one
Water-equivalent quantity; all family transitions remain 1:1; no extra Steam,
same-Cell mixed Matter, `phase_quantity` field, general fluid/velocity solver,
owner-linked fragment collection or external implementation port is allowed.
Generic gauge `pressure[]` remains separate. The design must define exact
link ownership, movement/condensation/rupture/editor/reset hygiene, bounded
matching behavior, dedicated phase-pressure equilibrium and atomic TE-3/TE-5D
activation. It must produce ADR-0009/spec/validation/planning/inventory, one
predeclared fixed-seed grid/time/matching proof executed exactly once and a
fresh-context independent review.

Reason: TE-5B let one EMPTY vacancy be reused across ticks; TE-5C could
double-count and discard useful capacity and conflated capacity with venting.
A reciprocal persistent extent is the first authorized candidate that can
retain exclusive phase-volume ownership across ticks, while a dedicated
phase-pressure field can relax or clear without losing provenance to the
historical generic gauge field.

Scope: Docs, memory and one external pure reference script/result only on
`feature/m0-g9-first-playable`, starting from
`94e4da7603dafcf4a83c652abe192a435779a127`. Rust, WGSL, Cargo, runtime
allocation, build, launch, full background-Air force, structure differential,
TE-4, G9-B/C/D/E, optimization, PR and `main` merge remain unauthorized and
**NOT STARTED**. Historical evidence remains source-bound.

Stop rule: This entry authorizes evaluation, not acceptance. Any unresolved
Critical/High proof or review finding stops **TE-5D DESIGN BLOCKED** and must
name whether the required repair is wider matching scope, more full-world
scratch, another persistent field, relaxation of 1:1 quantity or a different
volume representation. With zero unresolved Critical/High findings, the
maximum stop is **PERSISTENT VAPOR EXTENT-PRESSURE DESIGN CANDIDATE /
INDEPENDENT REVIEW PASS / USER ARCHITECTURE REVIEW PENDING**. ADR-0009 remains
**PROPOSED** and runtime remains unchanged.

Invalidated by: A later explicit user supersession, failed locked
grid/time/matching property, unresolved Critical/High review finding or
source-bound future implementation evidence contradicting an invariant.
Failure does not revive either stateless candidate or rewrite D-018.

### D-021 evaluation receipt — 2026-08-21

The frozen proof ran in one process and returned `DESIGN_BLOCKED`; script and
result SHA-256 are
`06d0cea8500fcc3a2ffa4010d0dab70770a3fb2fd8a94f0bf47846cd980dedb9` /
`853379af86ee536166cb752bffbf45cefe5eec93bc10038a023679d507d7a29a`.
It did not satisfy the all-labeled 6×6 obligation. Static analysis and fresh
independent review found that a legal persistent eight-source alternating
chain has a complete matching beyond the frozen depth six. Review SHA-256 is
`73adaf56bea1589d425d89ba9430a7f50f3d0b9cf50f5b8fdc2155f263968ed6`;
unresolved counts are Critical `0`, High `6`, Medium `2`.

TE-5D is therefore **DESIGN BLOCKED** and ADR-0009 remains **PROPOSED /
ARCHITECTURE REVISION REQUIRED**. The required repair is wider matching scope;
a bounded GPU realization may additionally need an explicitly authorized
full-world frontier/predecessor scratch. This result does not require another
persistent field, a different volume representation or relaxation of 1:1
quantity. Runtime remains not started.

## D-022 · Authorize the TE-5 pressure-volume architecture comparison — 2026-08-21 (source: direct user model-selection authorization)

Decision: Preserve D-019, D-020 and D-021 and their source-bound receipts as
immutable blocked-design history. TE-5B and TE-5C remain **REJECTED / DESIGN
BLOCKED**. TE-5D fixed-depth reassignment remains **REJECTED / DESIGN
BLOCKED**. Those failures do not disprove persistent phase-volume ownership,
the one-Cell/one-Water-equivalent ontology or 1:1 phase-family transitions;
they disprove the fixed-depth matching implementation contract. Raising depth
six to another constant is not an architecture repair.

Authorize a docs/reference-only **TE-5X PRESSURE-VOLUME ARCHITECTURE
COMPARISON PROGRAM** over exactly three candidates: exact persistent-extent
maximum matching; shared connected gas-chamber capacity; and a conservative
Vapor-volume Environment scalar. The study must apply the same predeclared
product contract and fixtures, execute one combined pre-registered reference
comparison exactly once, survey maintained primary prior art without importing
code, and obtain a fresh-context comparative review. No runtime model is
accepted by this decision.

Locked selection boundary: any candidate causing false rupture with sufficient
capacity, quantity/Air/phase-volume loss, irreversible pressure,
solver-delay-as-confinement, unbounded or unspecified production work, a pass
above eight storage bindings, unexplained persistent state, stale
movement/edit/reset state, historical evidence rebound or external code ingress
is ineligible. Eligible candidates rank by causal meaning, conservation,
deterministic GPU feasibility, bounded work, memory and then complexity. The
study may report exactly one Recommended, one Retained fallback and one
Rejected only if fresh review leaves no unresolved Critical/High finding.

Scope: Docs, memory and one external pure comparison script/result only on
`feature/m0-g9-first-playable`, starting from
`f5b146571f2cb95b89d56d8831b68ddbeb75f395`. Rust, WGSL, Cargo, runtime
allocation, build, launch, TE-3/TE-5 runtime, TE-4, G9-B/C/D/E, optimization,
PR, tag and `main` merge remain unauthorized and **NOT STARTED**. External
copied/translated/vendored implementation remains `0 files / 0 lines`.

Stop rule: This entry is comparison authorization, not acceptance. With zero
unresolved Critical/High findings, the maximum stop is **TE-5X ARCHITECTURE
COMPARISON COMPLETE / USER MODEL SELECTION PENDING** and ADR-0010 remains
**PROPOSED**. If no model is eligible, or comparative review leaves any
Critical/High finding, stop **TE-5X DESIGN BLOCKED**, name the exact reason and
do not synthesize a fourth candidate after the reference execution.

Invalidated by: A later explicit user supersession, a failed locked reference
property, unresolved Critical/High comparative finding or source-bound future
evidence contradicting a common invariant. Failure does not revive TE-5B/C/D,
rewrite D-018 or authorize runtime.

### D-022 evaluation receipt — 2026-08-21

The pre-registered combined process was started exactly once and exited before
any A/B/C evaluation. Its NetworkX 3.6.1 guard raised `AttributeError` because
the temporary `networkx` path resolved as a namespace module without
`__version__`. Candidate evaluations, generated states, grids and deterministic
replay are all zero/not run. The task's one-shot rule forbids a repair rerun.

Frozen script SHA-256 is
`0079246918a91faa606d531cb76591af0363dfb3a66d4b88882fc04e33efd8d5`.
The parseable post-exit failure receipt SHA-256 is
`097f340c265d9e43a23e281a776905add97e6b05c18dedd79d48807558efc116`;
it is not script-emitted proof evidence. Because eligibility and ranking were
not exercised, the provisional B/A/C ordering is void, no candidate is
Recommended/Retained fallback/Rejected by evidence, and **TE-5X is DESIGN
BLOCKED**. ADR-0010 remains **PROPOSED / COMPARISON EVIDENCE INCOMPLETE** and
runtime remains not started.

Fresh comparative review independently verified the zero-count evidence
boundary and recorded Critical `0`, High `11`, Medium `0`; review SHA-256 is
`c424c8336d3b34784f6a3ffbb37421ceca8888608c198da45793774b49ffb579`.
It found the frozen fixture matrix non-executable/author-favoring, A's
production matching and inherited extent integration incomplete, B's
narrow-neck/component/reduction/pressure ownership incomplete, C's
condensation sink blocked, and all candidates' activity/edge/Air/H contracts
open. Final D-022 stop remains **TE-5X DESIGN BLOCKED / NO RECOMMENDATION / NO
RETAINED FALLBACK / RUNTIME NOT STARTED**.

## D-023 · Supersede whole-Cell phase quantity and authorize conservative phase packets — 2026-08-21 (source: direct user architecture authorization)

Supersession note: D-024 rejects this candidate as the active phase
representation. Its design, proof receipt and blockers remain preserved
history.

Decision: Preserve D-019 through D-022 and ADR-0007 through ADR-0010 as
immutable blocked-design history. TE-5X remains **DESIGN BLOCKED** with proof
process attempted/completed `1 / 0`, A/B/C evaluations `0 / 0 / 0`, and no
Recommendation or Retained fallback. Its frozen script and failure receipt
must not be patched, rerun, continued or reused as the new evidence identity.

This decision explicitly supersedes D-018's constraint that one
Ice/Water/Steam Cell equals one whole Water-equivalent quantity, and D-022's
preservation of that ontology. D-018 remains accepted in every other locked
respect unless the new candidate explicitly names a necessary amendment:
foreground occupancy is still at most one Matter per Cell, phase-family
quantity and enthalpy remain conservative, same-Cell mixed foreground Matter
is forbidden, and TE-2 thermal ownership and the Atmosphere/Vacuum ontology
remain unchanged.

Authorize the docs/reference-only **TE-3Q / TE-5Q CONSERVATIVE PHASE PACKET
DESIGN PROGRAM**. The primary candidate adds `phase_units_current/next` with
`PHASE_UNIT_SCALE = 2`: EMPTY and non-phase Matter have zero units, Ice and
Water have two, normal expanded Steam has one, and compressed Steam has two.
The finite-world sum of units is conserved except explicit Void exit or
destructive authoring. Water boiling may split two units into two spatial
one-unit Steam packets through the existing expansion/Environment-receiver
transaction; blocked expansion remains one two-unit compressed Steam packet.
Two local endpoint-ready one-unit Steam packets may merge into one two-unit
Water packet and one canonical Vacuum EMPTY Cell. A dedicated spatial
Current/Next phase-pressure component derives only from compressed Steam and
remains separate from generic gauge pressure.

The new evidence identity is `TE3Q-PHASE-PACKETS-REFERENCE-V1`. It must use
Python standard library only, be syntax/import/fixture-list checked before
freezing, and execute exactly once after its seed, coefficients, merge order,
geometries, bounds and tolerances are fixed. The study must include at least
100,000 split/merge algebra trials and 10,000 bounded multi-tick grids, preserve
quantity/H/Environment accounting, audit the projected 96 MiB Current/Next
state increment and every future pass/binding/scratch lifetime, and obtain a
fresh-context independent review. External copied/translated/vendored
implementation remains `0 files / 0 lines`.

Scope: Docs, memory, one new external pure reference script/result and
read-only prior-art research only on `feature/m0-g9-first-playable`, starting
from `df6801272aa7505f34c7484ec406916516604c56`. Rust, WGSL, Cargo, runtime
buffer allocation, TE-3/TE-5 runtime, build, launch, TE-4, G9-B/C/D/E,
optimization, PR, tag and `main` merge remain unauthorized and **NOT STARTED**.

Stop rule: This entry authorizes evaluation, not acceptance. With zero
unresolved Critical/High findings, the maximum stop is **CONSERVATIVE PHASE
PACKET DESIGN CANDIDATE / INDEPENDENT REVIEW PASS / USER ARCHITECTURE REVIEW
PENDING** and ADR-0011 remains **PROPOSED**. Any unresolved Critical/High proof
or review finding stops **TE-3Q / TE-5Q DESIGN BLOCKED**. Do not synthesize
another pressure-volume model or begin implementation.

Invalidated by: A later explicit user supersession, a failed locked
grid/time/conservation property, an unresolved Critical/High independent-review
finding, or future source-bound implementation evidence contradicting an
invariant. Failure does not revive TE-5B/C/D/X or rewrite their evidence.

### D-023 evaluation receipt — 2026-08-21

The new standard-library script passed syntax/import/fixture-list prechecks and
was frozen at SHA-256
`c938c6e3ce7074abc6d5144c708f85a17be349bb84f962238e568c17d55ed03c`.
Its only model process completed with mathematical PASS: 100,000 algebra
trials, 10,000 bounded grids, 14 fixture functions and in-process deterministic
replay. Result SHA-256 is
`a0181d4ca0ed63eb92cac5cd04098ff438546903c8dc6853e8b0b5d5ab208ed7`.

Fresh independent review then found that the frozen model did not execute
several named obligations and recorded Critical `0`, High `8`, Medium `1`, Low
`0`. Review SHA-256 is
`40ff5a240851048d77d2afa27004856c69bdb67b6fae6d6b3398df57c6913146`.
Blocking findings cover reduced/constant fixtures, local greedy merge
stranding, movable Steam/2 resetting spatial pressure, rupture-eligible
pressure after source removal, unfrozen f32/generic-pressure transaction
details, incomplete writer/edit/reset closure and phase-pressure sleep binding.

Final D-023 stop is **TE-3Q / TE-5Q DESIGN BLOCKED / ADR-0011 PROPOSED /
ARCHITECTURE REVISION REQUIRED / RUNTIME NOT STARTED**. The script and result
are immutable and cannot be patched or rerun. Another evidence identity or
architecture revision requires a new direct user decision.

## D-024 · Authorize pressure-decoupled standalone TE-3 phase enthalpy — 2026-08-21 (source: direct user architecture authorization)

Decision: Reject ADR-0011's phase-packet model as the active phase
representation and preserve ADR-0007 through ADR-0011, TE-5B/C/D/X and the
packet attempts as blocked history. Their concrete counterexamples and frozen
evidence identities are not rewritten or rebound.

Supersession: D-024 supersedes D-018 only where it required atomic TE-3/TE-5
activation, and supersedes D-023's phase-unit representation. Restore ADR-0006
as the active TE-3 quantity contract: one Ice/Water/Steam foreground Cell is
one Water-equivalent quantity; every family transition is 1:1; Water boiling
has `matter_yield = 1`, `blocked_pressure = 0`, creates no second Steam and
does not enter the expansion proposal/spawn/blocked-pressure path.

Runtime authorization: Implement standalone TE-3 with exactly
`phase_energy_current` and `phase_energy_next` as persistent f32 Cell fields,
using accepted local-enthalpy repartition after TE-2 Q transfer. Preserve the
D-018 real-sink/work predicate, buried-ready-Water behavior, no-sink Steam
metastability, radius-two free-air nucleation and matching activity rules.
Produce one source-bound user-testable phase-cycle candidate, complete the
specified targeted validation, run exactly one final-source canonical FULL,
one release build, one bounded launch check and one bounded measurement, then
record docs/memory closure and push only the named feature branch.

Evidence boundary: Historical G5 Water expansion/pressure/rupture receipts
remain valid only for their historical source. The active TE-3 source makes no
claim to the old Water heat -> extra Steam -> blocked expansion pressure ->
rupture chain and does not remove the generic pressure or rupture systems.
Register `WATER_STEAM_PRESSURE_VOLUME_REDESIGN` as a known deferred product
follow-up. TE-5 Pressure redesign is **DEFERRED / NOT STARTED**.

Excluded: phase units or packets, phase quantity/pressure fields, owner links,
volume tokens, matching, CCL, Vapor Environment fields, Air-pressure force,
TE-4, G9-B/C/D/E, optimization, PR, tag and main merge. External simulation
code copied, translated or vendored remains `0 files / 0 lines`.

Stop rule: Success may reach **TE-3 WATER/STEAM PHASE-CYCLE CANDIDATE / USER
REVIEW PENDING** only with source-bound runtime evidence and zero quantity gain
or Water phase-pressure source. An unresolved invariant, H conservation,
identity/Air partial-commit, GPU integration or validation blocker stops
**TE-3 BLOCKED** with its smallest reproduction. Do not invent a pressure-
volume replacement inside this task.

## D-025 · Record partial direct TE-3 review and require Sandbox phase truth — 2026-08-22 (source: direct user observation and remediation authorization)

Decision: Preserve D-024 and the production-physics source
`41467219819c5d0cb3eab8ae22b652449da20480`. Record Scenes 2, 3 and 4 as
**DIRECT OBSERVATION CONSISTENT**, without promoting the whole TE-3 candidate
to user accepted. Scene 1 remains **USER REVIEW PENDING**.

Observed Scene 2: equal-H surface and buried Water began at `T=100`, `E=480`;
the first eligible tick completed only the surface Cell to Steam; the buried
Cell remained Water at `E=480`; the tick-24 opening permitted its later 1:1
Steam completion; family quantity remained two. A later EMPTY fixed probe is
ordinary Steam movement away from that coordinate, not quantity loss.

Observed Scene 3: the cold-lid path began latent condensation before the
free-Air path; free-Air condensation began later through sparse nucleation;
the K=0 Boundary path remained unchanged; lid Water later cooled below
`100 C`; and no whole lane converted in one tick. Observed Scene 4: cooling
reduced partial-boiling energy, heating initially increased partial-
condensation energy, no-sink Steam remained unchanged until a real cooling
face was restored, and the restored sink resumed condensation and eventually
created Water.

Sandbox clarification: a large Steam cloud near the top of the sealed Starter
Lab, some condensed falling Water, and Water appearing at the `100 C` phase
plateau are observations consistent with the current finite-energy sealed
world, but the normal Sandbox Inspector cannot yet expose the authoritative
phase energy needed to review that state honestly. This is **CLARIFICATION /
PRODUCT OBSERVABILITY REQUIRED**, not a thermodynamics retuning request.

Authorization: keep the Inspector staging payload at exactly 24 bytes and at
no more than 10 Hz. Reuse its fifth four-byte slot as authoritative
`phase_energy` only in the Sandbox profile while Gallery/technical modes retain
Cell activity. Add truthful phase progress/plateau copy, disclose the sealed
boundary and absence of an external ambient heat sink, rename the primary
thermal tools to `Add Heat` and `Remove Heat`, polish fixed probe labels, and
create one local Desktop shortcut to the canonical EXE after the final release
build. The shortcut is local-only and is not committed.

Scope: `apps/windows` Inspector/Sandbox/candidate presentation, targeted
tests, docs and memory only on `feature/m0-g9-first-playable`. Engine/Core,
production WGSL, phase constants/predicates, Air/thermal coefficients, default
Environment boundary mode, Pressure, TE-4, G9-B/C/D/E, optimization, another
EXE/BAT, PR and main merge remain unauthorized. External copied, translated or
vendored implementation remains `0 files / 0 lines`.

Stop rule: Successful remediation stops at **REVISED WATER/STEAM PHASE-CYCLE
CANDIDATE / USER REVIEW IN PROGRESS** with Scene 1 still pending. If targeted
production semantics fail the latent-plateau, no-sink or free-Air controls,
stop and report the smallest reproduction without retuning thresholds.

## D-026 · Record the remediated Scene 1 direct observation — 2026-08-22 (source: direct user review)

Decision: Record the `e9f4a37...` Scene 1 replacement as **DIRECT OBSERVATION
CONSISTENT**. The user directly confirmed that Steam is created, rises, then
condenses and the resulting Water falls, matching the ordered production
phase/thermal/movement chain.

Boundary: This confirms the Scene 1 remediation and closes Q-015. It does not
invent a whole-candidate `USER ACCEPTED` disposition that the user did not
state. D-024's one-Cell/one-quantity model, deferred Water/Steam pressure
redesign, historical evidence boundaries, and D-025's Scenes 2–4 observations
remain unchanged.

Scope: Candidate source `e9f4a3744ea3bdab0fd70f0f78aa27cb7e9fa448`,
artifact SHA-256
`6F2EF0BF49FC39AF550B2CF958DCC5A2F551AAE65ACD9F1735D208519E8E1C0E`,
and the direct user observation in this task. TE-5, TE-4, and G9-B/C/D/E
remain not started.

Invalidated by: a later explicit user supersession or source-bound evidence
that contradicts the recorded observations or 24-byte profile contract.

## D-027 · Accept TE-3 with known follow-up — 2026-08-22 (source: direct user disposition in this task)

Decision: Close the pressure-decoupled TE-3 Water/Steam phase-cycle candidate
as **USER ACCEPTED WITH KNOWN FOLLOW-UP**. Preserve D-024's one-Cell/one-
Water-equivalent quantity contract, D-025's Scenes 2–4 direct observations and
D-026's Scene 1 direct observation. The accepted product line creates no extra
Steam quantity and no Water phase-pressure source.

Known follow-up: `WATER_STEAM_PRESSURE_VOLUME_REDESIGN` remains **DEFERRED /
NOT STARTED**. Acceptance does not select or revive any blocked TE-5B/C/D/X or
phase-packet model and does not authorize TE-5, TE-4, G9-B/C/D/E, optimization,
Powdergame main promotion or a new runtime task.

Evidence: production physics
`41467219819c5d0cb3eab8ae22b652449da20480`; app/Scene 1 source
`e9f4a3744ea3bdab0fd70f0f78aa27cb7e9fa448`; source-bound TE3-F01–F15/FULL
receipt; D-025/D-026 direct observations; canonical artifact SHA-256
`6F2EF0BF49FC39AF550B2CF958DCC5A2F551AAE65ACD9F1735D208519E8E1C0E`.
This closure is docs/memory-only and reuses those receipts without rerunning
Cargo, GPU, FULL, build, launch or candidate validation.

Invalidated by: a later explicit user supersession or authenticated,
source-bound evidence contradicting the accepted quantity, phase, review or
artifact claims. A future pressure redesign is a new gated claim and does not
rewrite this acceptance.

## D-028 · Authorize the TE-4D ignition-kinetics design program — 2026-08-22 (source: direct user architecture authorization)

Decision: Preserve G9-A, TE-2 and TE-3 as **USER ACCEPTED WITH KNOWN
FOLLOW-UP** and preserve `WATER_STEAM_PRESSURE_VOLUME_REDESIGN` as **KNOWN
FOLLOW-UP / DEFERRED / NOT STARTED**. Authorize a docs/reference-only TE-4D
design program for integrated thermal ignition dose, cooling decay,
previous-snapshot orthogonal `FLAME_EVENT` bonus and finite energy-like
chemical heat accounting.

Required reuse boundary: Start from the current generic combustion descriptor,
Matter-owned fuel progress, combustion flags, TE-2 authoritative temperature,
Current/Next ownership, movement hygiene, Smoke proposal/claim transaction,
activity/wake and profiler contracts. Compare the existing unowned flag bits
against a dedicated Current/Next exposure pair, but do not add runtime state,
passes or bindings in this task. Historical G4/G8/G5/TE receipts remain bound
to their original sources.

Evidence authorization: Predeclare coefficient windows and fixture geometry,
preflight and freeze one standard-library reference model, execute that frozen
model exactly once, and subject the proposed design to a fresh-context
independent adversarial review. Six-bit packed exposure is eligible only if it
passes the locked timing, ownership and no-wrap contracts. Any unresolved
Critical or High finding stops **TE-4D DESIGN BLOCKED**.

User-owned boundary: Vacuum combustion policy remains undecided. This decision
does not authorize Oxygen/Air-mass consumption, Ash, flame/Smoke FX, Pressure
redesign, Rust or WGSL implementation, Cargo changes, runtime allocation,
build, launch, TE-5/TE-6, G9-B/C/D/E, Powdergame PR or `main` merge. External
simulation implementation copied, translated or vendored remains `0 files / 0
lines`.

Stop rule: With a passing frozen reference result and zero unresolved
Critical/High review findings, the maximum result is **TE-4D IGNITION KINETICS
DESIGN CANDIDATE / INDEPENDENT REVIEW PASS / USER ARCHITECTURE REVIEW
PENDING**, with ADR-0012 **PROPOSED** and TE-4 runtime **NOT STARTED**. A later
explicit user decision is required before implementation or Vacuum-policy
selection.

## D-029 · Authorize TE-4D v2 evidence repair and select the candidate identity — 2026-08-22 (source: direct user architecture decision)

Decision: Preserve TE-4D v1 as **DESIGN BLOCKED** and preserve its frozen
script and failure receipt unchanged. Its only process completed no model work
because coefficient identity and secondary tie policy were not fixed; its
named-fixture aggregation could also overclaim unexecuted paths. Authorize a
new docs/reference-only evidence identity,
`TE4-IGNITION-KINETICS-REFERENCE-V2`, without continuing or repairing v1.

Select integrated excess-temperature dose, cooling decay and previous-
snapshot orthogonal flame bonus. Fix Oil as `48/2/50/6/1/2/4` and Wood as
`60/1/50/5/1/2/4` for budget/base/bucket-width/max/decay/flame-bonus/flame-
cap. Evidence validates these exact tuples and declared secondary objectives;
it performs no coefficient search. Select canonical packed u6 exposure in
flags bits 2..3 and 28..31. Dedicated Current/Next exposure remains an
unselected fallback and cannot replace a failed packed candidate.

Select `NON_VACUUM_AIR_FACE_REQUIRED`: Air access requires an in-domain
orthogonal neighbour that is EMPTY with positive current Air mass. Vacuum,
Void and occupied GAS Matter do not qualify. Air is not consumed or rate-
scaled and no Oxygen quantity exists. Ignition and sustain require access.
Loss while burning extinguishes before Heat, Flame or Smoke emission that
tick, preserves fuel and clears exposure; an inaccessible unlit Cell decays
exposure.

Select Oil/Wood gross chemical Q `15/8`, derived GPU delta `Q/C`, finite
gross/deposited/clipped accounting under the existing 1200 C cap, and consume-
before-emission on duration tick 600/900. Emitting ticks remain 599/899 and
maximum gross totals are `8,985/7,192`; the consumption tick emits zero.

The source-audited future projection is two new logical passes, 42 passes/84
queries, 1,344 profiler bytes, zero new persistent world bytes and zero new
full-world scratch bytes. One manifest-bound standard-library reference
execution and one fresh-context independent review are required. Any evidence
failure or unresolved Critical/High finding stops **TE-4D v2 DESIGN BLOCKED**.

Stop boundary: at most **TE-4D v2 REVISED IGNITION KINETICS DESIGN CANDIDATE /
INDEPENDENT REVIEW PASS / USER ARCHITECTURE REVIEW PENDING**. ADR-0012 remains
Proposed and TE-4 runtime remains **NOT STARTED**. Oxygen, Ash/new Matter,
production FX, Pressure redesign, TE-5/TE-6, G9-B/C/D/E, optimization, main
merge and PR remain unauthorized.

Invalidated by: later explicit user supersession, a failed frozen v2 execution,
an unresolved Critical/High v2 review finding, or a source audit disproving
the 42-pass/no-new-state projection.

## D-030 · Authorize TE-4D v3 transaction and oracle closure — 2026-08-22 (source: direct user architecture decision)

Decision: Preserve TE-4D v1 and v2 as **DESIGN BLOCKED / IMMUTABLE**. Their
scripts, manifests, result/failure receipts and hashes are historical narrow
reference records only and may not be patched, rerun or relabelled as design
approval. Authorize the distinct docs/reference-only identity
`TE4-IGNITION-KINETICS-REFERENCE-V3` to close transaction receipts and the F08
oracle without beginning runtime work.

Retain integrated excess-temperature dose, cooling decay, previous-snapshot
orthogonal `FLAME_EVENT` bonus, packed u6 exposure, exact Oil
`48/2/50/6/1/2/4` and Wood `60/1/50/5/1/2/4` identities, chemical-Q cap
accounting, consume-before-emission on the final fuel tick, and the positive
non-Vacuum Air-face requirement. These tuples are **USER_SELECTED_AND_VALIDATED**;
global or unique coefficient optimality is **NOT_CLAIMED**.

Select the exact precondition lifetime `COMBUSTION_STAGE_SNAPSHOT`: the settled
production state after movement, TE-2, TE-3 and decay hygiene/reconcile, and
immediately before future ignition exposure and combustion. This snapshot
authorizes the current stage's exposure, ignition, sustain, Heat,
`FLAME_EVENT`, Smoke proposal and fuel increment. A downstream same-stage
Smoke commit may occupy the sole qualifying Air face without rollback. At the
next snapshot, absence of a qualifying face extinguishes before emission,
produces no Heat/Flame/Smoke or fuel increment, and preserves prior fuel
progress. Air is not consumed and this is not Oxygen simulation.

V3 transaction evidence must use distinct mutation entrypoints that return
only resulting state and optional semantic event IDs. A separate auditor must
derive every required path receipt from immutable before/after state and an
ownership specification. Counter provenance is
`AUDITED_BEFORE_AFTER_STATE_DIFF`; a required path without an audited mutation
cannot pass. The exact F07/F08 event lists must be compared against a frozen,
standard-library, independently structured oracle generated before evidence
freeze. Self-replay establishes determinism only.

The source-audited projection remains two future logical passes: 42 passes,
84 timestamp queries, 1,344 profiler bytes, zero new persistent bytes and zero
new full-world scratch bytes. Post-Smoke settle copies Matter and Air into the
authoritative current buffers before activity proposals, so a future ignition
activity proposal can observe lost Air access and keep the next-stage
extinguish runnable without another pass or state. Each projected pass remains
within eight storage bindings.

Evidence authorization: preflight and freeze the independent oracle,
manifest and v3 reference; execute the v3 identity exactly once; then obtain a
fresh-context review by a reviewer who authored none of those artifacts. Any
failed campaign, exact-oracle mismatch, zero audited required path, or
unresolved Critical/High finding stops **TE-4D v3 DESIGN BLOCKED**.

Stop boundary: at most **TE-4D v3 TRANSACTION-CLOSED IGNITION DESIGN CANDIDATE /
INDEPENDENT REVIEW PASS / USER ARCHITECTURE REVIEW PENDING**. ADR-0012 remains
Proposed and TE-4 runtime remains **NOT STARTED**. Oxygen, Ash/new Matter,
production FX, Pressure redesign, Rust/WGSL implementation, runtime state or
passes, Cargo, build, launch, TE-5/TE-6, G9-B/C/D/E, optimization, Powdergame
PR and `main` merge remain unauthorized.

## D-031 · Authorize the targeted TE-4 ignition transaction supplement — 2026-08-22 (source: direct user architecture decision)

Decision: Preserve TE-4D v1/v2/v3 as **DESIGN BLOCKED / IMMUTABLE** and do
not patch, rerun or relabel any of their files. V3 remains usable only for its
exact user-selected coefficient identity/rate profile, packed-u6 arithmetic,
independent exact F07/F08 match, deterministic replay and static 42-pass
feasibility projection. It does not establish topology-derived F15B,
independent transaction semantics, the complete fuel lifecycle or re-auditable
mutation receipts.

Authorize the distinct docs/reference-only identity
`TE4-IGNITION-TRANSACTION-SUPPLEMENT-V1`. This is a bounded composition
supplement for only v3 H-001/H-002/H-003/M-001, not a v4 broad proof. It may
execute exactly once after a non-executing freeze and may not rerun v3's
100,000 sequences, 10,000 grids, F07/F08, coefficient selection, oracle
generation or GPU feasibility work.

The supplement must derive both Air-access snapshots from one shared pure
orthogonal topology predicate; audit semantics independently from immutable
before/after state and manifest specifications without trusting SUT names or
events; reject at least one invalid mutation for every transaction class; run
Oil/Wood through one continuous duration-driven fuel/heat state machine; and
publish canonical hashed before/after JSONL snapshots sufficient for a third
party to recompute the important totals without executing the SUT. Counter
provenance is `INDEPENDENT_SPEC_BEFORE_AFTER_AUDIT`.

Evidence authorization: freeze manifest/script/schema/tolerances, execute the
supplement exactly once, independently re-audit its published snapshots, then
obtain a fresh-context review by a reviewer who authored none of its artifacts.
Any unresolved Critical/High finding stops **TE-4D TRANSACTION SUPPLEMENT
BLOCKED**. Zero unresolved Critical/High may compose with the explicitly narrow
v3 results to reach **TE-4D ARCHITECTURE EVIDENCE COMPLETE / USER ARCHITECTURE
REVIEW PENDING**. ADR-0012 remains Proposed; runtime is not authorized.

Scope excludes Rust/WGSL/Cargo, production passes/buffers, build/launch,
coefficient/model changes, Oxygen, Ash/new Matter, presentation, Pressure
redesign, TE-5/TE-6, G9-B/C/D/E, optimization, PR and `main` merge. External
implementation copied, translated or vendored remains `0 files / 0 lines`.
