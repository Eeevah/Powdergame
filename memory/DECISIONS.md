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
