# Session log

Terse append-only continuity audit; not a transcript and not runtime evidence.

## 2026-08-19 · Initial isolated Ballast pilot

- Synchronized `personal-infra-wiki` at `318276eebfbf913638d72f5d218ead2450361a01`; Ballast installation verify passed.
- Created isolated `agent/ballast-memory-pilot` from G8-B line at `e43078737712862c9cc6ccdc4b7e56475bafc6ce` without reading or modifying active worktree WIP.
- Added AGENTS and the initial bounded `memory/` map in commit `ba2b6406f6605882c51886b0a50bc64d10990a7f`; opened Draft PR #4.
- Docs-only audit passed. No app, Rust/GPU/FULL, smoke, candidate or capture ran.

## 2026-08-19 · Ballast adopted with reversible cutover

- User adopted Ballast as Powdergame's single active session-continuity workflow.
- Active cutover commit: `8d21756f3dfa5c6a743f0aa03108153bb4b206df`.
- Powdergame PR #4 was converted from pilot to reversible cutover; squash forbidden.
- `personal-infra-wiki` PR #39 merged after CI PASS, recording workflow, decision and rollback troubleshooting.
- Immediate disable: `BALLAST_DISABLE=1` or Hook untrust.
- Runtime/evidence work remained untouched.

## 2026-08-19 · G8-B closed and first G8-C pilot stopped

- G8-B closed/frozen at `18391e6a9fc8f9bc7b2757f3504366f106c05435`; legacy launchers retired at `8ee1ae238c324c1db1d7e2882af071fec179a8f1`.
- G8-C implementation added Mode C coexistence, Mode D render timestamps, matrix coordination, source/binary sealing and independent verification while preserving G8-A/B producer contracts.
- First pilot `g8c-pilot-8ee1ae238c32-c64090539536`: all five headless A/B processes passed; first Sand Mode C failed because a late stale `Resized(2864×1560)` payload was mistaken for a live noncanonical resize.
- No official Receipt/package/report/verifier existed; no performance conclusion was made.

## 2026-08-19 · Replacement pilot fixed lifecycle and exposed aggregation contract

- Live `window.inner_size()` became final authority; stale payloads are ignored only while live size remains exactly 1600×900. Genuine noncanonical/zero live size remains fatal.
- Replacement pilot `g8c-pilot-8ee1ae238c32-6341f4f59218`: five A/B, five C and five D processes all exited `0`; ten stale initial payloads ignored; fatal live resize/surface/device errors all `0`.
- Final aggregation stopped because historical producer uses `wall_per_tick` + `ms/tick` while the coordinator searched for internal `wall_ms_per_tick`.
- No official Matrix was published from the pilot.

## 2026-08-19 · Aggregation replay and official Matrix completed

- Strict adapter preserved external producer vocabulary and maps it explicitly to internal model. Actual producer fixtures and alias/unit/missing/duplicate regressions were added.
- Aggregation replay `g8c-aggregation-replay-20260819T015515996891Z-fc408076b67a` launched zero executable/GPU/measurement process and passed downstream publication/verifier while remaining `non_evidence=true`.
- Sealed source `4653d7c2e09e93f80fb81eeb73458d992c86858f` was committed and pushed clean/upstream-equal.
- Official Matrix `g8c-official-matrix-4653d7c2e09e-64df60ba0d79` ran exactly once; build plus 15 measurement processes exited `0`; independent verifier recomputed 230 fields with mismatch `0`.
- Minimum Mode A P50 `931.602 TPS`; maximum Mode B P95 `1.046784 ms`; minimum Mode C simulation `59.898580 TPS`; zero deadline misses/catch-up/drops; maximum Mode C frame P95 `4.2005 ms`; maximum Mode D render P95 `0.021280 ms`; tracked persistent bytes `184,576,672` per scenario.
- Recommendation: `PROCEED_TO_G9`. No G9, optimization, G8 closure or main promotion started.

## 2026-08-19 · Ballast integrated and G8-C docs closed

- Final Matrix checkpoint committed to memory branch as `4f5e910f6a4f27548f7f0b41f21e69b80996ec93`; policy CI passed.
- PR #4 was retargeted to clean `feature/m0-g8c-official-matrix` at `4653d7c2...`, marked ready and merged with merge commit `6b5f0201f882f212f9916521aec689261d97b4a6`. It was not squashed; product/evidence history is first parent.
- G8-C authoritative docs closure: `51699d1a73be6f484a8436720463d1aa6c037de9`.
- `docs/HANDOFF.md` retired as live checkpoint; current resume path is memory index/checkpoint/decisions.
- G8-A visual durable disposition and G9 product brief remain user-owned decisions. No runtime evidence was rerun for these docs/memory changes.

## 2026-08-19 · G8 closed and G9 product brief approved

- User accepted the recommended post-Matrix choices.
- Product/evidence commit `78d8e9325bc224e0ec193af75bacc945eccc0a7d` formally superseded the separate old G8-A visual requirement without retroactively marking it `PASS`, recorded G8 `CLOSED / FROZEN`, and added the canonical G9 product brief.
- G9 starts with an editable Starter Lab plus immediate New Blank World; all M0 Matter is visible from the beginning.
- G9-A scope is the bounded editor/control set with Cell Inspector reuse. Discovery follows after the editor works and records phenomena without unlocking Matter.
- Save/Load and Rewind are deferred from the first acceptance slice; Rewind remains part of the longer experiment-tool direction.
- The first direct product-validation session is approximately 10–15 minutes and unguided. A voluntary second experiment and causal explanation are primary strong signals.
- G9-A is authorized; optimization, new Matter, main promotion and M0 `ACHIEVED` are not.
- Docs/memory-only closure: no Rust/GPU/FULL, app smoke, candidate, official capture or evidence rerun.

## 2026-08-19 · Decision-ledger append-only correction

- The first G9 memory checkpoint draft accidentally rewrote D-005's pre-integration rollback wording instead of appending a superseding entry.
- The original D-005 text was restored byte-for-meaning, and D-008 now records the current merge-based rollback after PR #4 integration.
- Checkpoint references were updated to distinguish D-005 historical scope from D-008 current scope.
- This was a memory-only correction; no product decision, runtime evidence, test, smoke or capture changed.

## 2026-08-19 · G9-A First Playable implementation candidate

- Source `f9a7087249bf6ffa0b6d47ad7568ba1798f591a3` added an explicit Sandbox mode to the canonical Windows EXE; no-argument launch remains Gallery and experiment/G8-C routing remains isolated.
- Starter Lab and New Blank World use production GPU staging. All nine M0 Matter, Draw/Erase/Heat/Cool, four brush sizes, Pause/Play/Step/speed/Reset, Pan/Zoom and Cell Inspector are available without Discovery or unlock gates.
- Bounded/coalesced edit commands update Current/Next together before ticks, clean field/flags state, wake touched/halo chunks, and keep CPU free of world truth. Renderer, picking and Inspector share one camera transform.
- Targeted Windows validation, affected check/clippy, strict policy audit, exact scenario reset and one 3-frame release Sandbox smoke passed. Workspace FULL was not required and ran `0` times.
- Candidate status is **IMPLEMENTATION CANDIDATE / USER ACCEPTANCE PENDING**. G9-B/C/D/E, Discovery, Save/Load, Rewind, optimization, main merge and M0 closure remain not started.

## 2026-08-19 · Smoke terminology disambiguated

- User clarified that Codex's frequent short software checks are not measurements of combustion-generated Smoke Matter.
- D-009 reserves `Smoke` for the in-game Matter and requires `bounded launch check` or `application startup check` for software startup/mode/bounded-exit validation.
- `--smoke-frames` remains a legacy compatibility option and is described as a bounded launch-frame limit.
- Historical immutable evidence and prior log wording were not rewritten; new prompts, reports, checkpoints and policy prose use the unambiguous terms.
- This terminology-only change did not run Rust/GPU/FULL, a bounded launch check, candidate, capture or user acceptance.

## 2026-08-19 · G9-A canonical launch corrected from direct feedback

- The first candidate required an explicit `sandbox` argument, so double-clicking the root BAT still opened the frozen G8 Gallery and failed the practical user-test handoff.
- User feedback established D-010: canonical BAT/EXE no-argument launch now opens Starter Lab Sandbox; explicit `gallery`/`normal` preserves G8-B.
- Source `0d03dafbb4bc6375adc10c8b819db6c0bc232db9` passed default and explicit-route tests, affected check/clippy, strict launcher audit and a no-mode 3-frame release bounded launch check. The rebuilt canonical EXE SHA-256 is `9e809342074c313c79a1080a89b9aa6e84e0e39238b4c8d9aa1368ad8bc72f3c`.
- Candidate remains **USER ACCEPTANCE PENDING**; no later G9 slice or optimization started.

## 2026-08-20 · G9-A direct-review remediation candidate

- Direct review classified the first candidate **USER REVIEWED / REVISION REQUIRED** and established D-011 with exactly five authorized changes.
- Source `b363c078fdc1d7e8b54fa6be328b7a0c5b908f06` makes Draw EMPTY-only, gives direct Ice/Steam placement -30°C/80°C defaults, groups all nine visible Matter as Core/Generated/Advanced, adds a 150 ms identity-only Inspector grace, and adds camera-aligned presentation-only Heat/Cool feedback.
- GPU authority, Current/Next ordering, touched/halo wake, Inspector 24-byte/10-Hz sampling, Gallery/worker/G8-C routing and all G8 evidence remain unchanged.
- Validation passed: fmt; Windows suite `150 passed / 0 failed / 1 ignored`; affected check/clippy; strict audit; and one canonical release Sandbox 3-frame bounded launch check on RTX 5090/DX12. The EXE SHA-256 is `26512598746c21858a81c85a2e4f8f2635e2e1deed6c1ebff661bc2810a126d1`. FULL ran `0` times.
- Status is **REVISED IMPLEMENTATION CANDIDATE / USER RE-REVIEW PENDING**. G9-B/C/D/E, Ash, Discovery, Save/Load, Rewind, optimization, main promotion, PR creation and M0 closure remain not started.

## 2026-08-20 · G9-A Inspector continuity v2 and thermal project registration

- Direct re-review confirmed the previous Draw/Ice/Steam/palette/Heat-Cool revisions were present but classified G9-A **USER RE-REVIEWED / REVISION REQUIRED** because rapid Cell hover still flickered.
- D-012 records continuity v2 and registers Thermal Transport & Ignition Causality as planned/design-required/not-started before G9-B.
- Source `a00e39b2e00bfbd9ac28214c44cd22cc97542bb4` separates requested hover from one presented sample, preserves the original Cell/Material/tick/freshness through second/third rapid hovers, keeps one Ready/Held/Sampling panel geometry, hides compact stale identity and atomically replaces with a fresh current-Cell sample. Reset/preset/epoch/failure invalidation remains immediate; readback stays 24 bytes at most 10 Hz.
- Focused validation passed: Inspector `11/11`, Inspector UI `3/3`, affected check/clippy, strict audit and exactly one 3-frame release Sandbox bounded launch check. EXE SHA-256 `5062f0cb0ac9f23828765ce6c2fe2c2137caaa2f055c1c4fcfd9fb0cf7f177d5`; FULL `0`.
- No engine/Core/WGSL/shared Simulation/fixture change and no G8 rerun occurred. Thermal implementation and G9-B/C/D/E remain not started.

## 2026-08-20 · Thermal Environment TE-0 design lock

- D-013 selected a separate full-resolution Air mass/energy Environment Field,
  exact Atmosphere/Vacuum/EMPTY/Void meanings and Celsius-like product anchors;
  current runtime values and behavior remain unchanged.
- TE-0R surveyed internal reuse and exact external prior art with copied
  external code count `0`. The design package SHA-256 is
  `3f5abcc282881a190a0881499b61f8d00cf782f305354f6b533740ac0d0b9c84`.
- Production inventory pinned 17 existing passes, the eight-storage ceiling,
  writer/scratch lifetimes, every known occupancy path and a correctness memory
  baseline of `268,462,208` bytes at 2048² including one receiver scratch.
- Independent TE-0A review initially found seven High defects. The canonical
  design resolved all seven; final recheck was Critical `0`, High `0`, blocker
  `0`.
- The reference formula proof ran exactly once with seed `1413820466` and
  40,000 randomized trials; maximum error was
  `7.275957614183426e-12`, result `PASS_REFERENCE_MATH_ONLY`.
- Docs source commit `2591dd5196752ca0caa4a69029dd04a9eee76744` records ADR-0005, spec, validation, gates,
  inventory, research and adversarial review. TE-1 is READY / NOT STARTED.
  G9-A remains USER RE-REVIEW PENDING; G9-B/C/D/E remain not started.
- No Rust/WGSL/Cargo/runtime/fixture change, Cargo test, GPU/FULL/build, bounded
  launch, candidate or capture ran.

## 2026-08-20 · Thermal Environment TE-1 state and occupancy foundation

- D-014 authorized and source `1a722d239a16bade5772688fa822465d5cef4602`
  implemented four Air Current/Next buffers, one receiver scratch, canonical
  Atmosphere/Vacuum staging, occupancy reconciliation, whole-parcel
  receiver-gated phase/Smoke spawn, exactly-once blocked phase pressure and
  exact Matter flag identity hygiene.
- The production profiler now names 30 passes. Tracked no-profiler totals are
  4,196,864 B at 256² and 268,462,208 B at 2048²; the Inspector remains a
  24-byte Matter sample at no more than 10 Hz.
- Targeted Core/GPU/staging/Sandbox/profiler tests, all-target check,
  warnings-denied clippy and strict audit passed. The first FULL attempt found
  a pre-existing Heavy Mixed census assertion block misplaced inside a
  Fire/Heat fixture test; only that stale test block was removed, and the
  final-source workspace FULL passed. Attempts 2, final-SHA valid pass 1.
- One release build and one three-frame Sandbox bounded launch passed on RTX
  5090/DX12. EXE SHA-256
  `8c3f0050eef67cfca04e970c071276ce8ae856a7a1a65e58ff63a0deecb34ea6`.
- Air transport, Air thermal exchange, Air pressure coupling, TE-2 and
  G9-B/C/D/E were not started. G8/G8-C/candidate/official runs remained 0.

## 2026-08-20 · Thermal Environment TE-2 passive transport candidate

- D-015/source `fb7e568e21012b6067269f4e1b82c36c865023d0` implemented full-resolution
  Air flow/advection, unified passive thermal exchange, activity/wake and the
  atomic Celsius-like gameplay migration.
- The user-supplied Chinese-community research added the 30-case CPU/GPU
  `SMALL_DELTA_THERMAL_CONVERGENCE` contract. Deadband is a work gate, copied
  external simulation code remains `0 files / 0 lines`.
- Targeted suites, all-target check, warnings-denied clippy, strict audit and
  exactly one final-source serial workspace FULL passed. One locked release
  build and one 60-frame TE-2 bounded launch passed.
- One-shot performance: 2048² equilibrium/frontier GPU tick P95
  `2.599712/2.304832 ms`; equilibrium terminal activity `0` Cells/chunks.
- Completion: **PASSIVE THERMAL ENVIRONMENT CANDIDATE / USER REVIEW PENDING**.
  Air-pressure force, TE-3 and G9-B/C/D/E remain not started.

## 2026-08-20 · TE-2 direct-review remediation and TE-3 blocker registration

- Live Git preflight found clean local/remote source
  `869690b7a282eec203d10df3502bc3451db03779`; it superseded the older expected
  handover SHA without a merge.
- Direct disposition recorded Inspector continuity v2 **USER ACCEPTED**, G9-A
  **USER ACCEPTED WITH KNOWN FOLLOW-UP**, and original TE-2 **USER REVIEWED /
  REVISION REQUIRED**. D-016 preserves D-015 evidence and authorizes only
  candidate controls, bounded diagnostics and staging.
- Candidate source `097728128343cf89383920c968a010b3dcf8e8c0` enables F
  x1/x4/x16, ordered paused N with immediate generation-safe sampling,
  candidate-only `TE-2 DIAGNOSTICS [I]`, persistent actual-value summaries and
  legible four-scene staging. Engine/Core, production compute WGSL, TE-2
  coefficients and phase/movement/ignition runtime are unchanged.
- Scene-1 production checkpoints at ticks `0/1/8/60/300` show target
  temperatures `20/28.4/69.755394/129.528992/146.758209 °C` Direct,
  `20/20/20.128620/24.413996/46.056732 °C` Atmosphere and constant `20 °C`
  Vacuum. Scene 2 bounded accounting, scene 3 sealed-zero exchange and scene 4
  explicit reservoir exchange are visible and tested.
- The initial focused candidate run exposed three over-bound accounting reads
  (`7/10` passed); batching to the existing 64-Cell limit fixed them and the
  rerun passed `10/10`. Windows `164/164` passed with one ignored test; fmt,
  affected check/clippy, strict audit and diff check passed. FULL, G8/G8-C and
  TE-3 runtime counts were `0`.
- One actual 60-frame TE-2 bounded launch passed on RTX 5090/DX12. The canonical
  BAT's mandatory build recompiled after the earlier explicit release build, so
  the honest release-build invocation count is `2`, not the requested `1`.
  Final EXE SHA-256 is
  `283fa6c603eb47d3906a14302b183ee8509d9571039bf135e69d922d091d0f00`, size
  `10,034,176` bytes.
- Current phase audit registers `1 Water -> up to 2 Steam -> up to 2 Water`,
  preserved temperature and absent latent/progress accounting as a required
  TE-3 design blocker. No representation was selected and no phase runtime was
  changed. TE-2 is **REVISED PASSIVE THERMAL ENVIRONMENT CANDIDATE / USER
  RE-REVIEW PENDING**; TE-3 remains **DESIGN REQUIRED / NOT STARTED**.

## 2026-08-20 · TE-2 direct user-acceptance closure

- D-017 records G9-A and TE-2 **USER ACCEPTED WITH KNOWN FOLLOW-UP** after the
  user confirmed F/N/I, Direct > Atmosphere > Vacuum, sealed Vacuum refill,
  sealed no-external-exchange redistribution, fixed-reservoir accounting and
  usable reset/controls.
- All automated verdicts, production/remediation identities, performance
  receipts and limitations remain unchanged; this docs-only closure ran no
  Cargo, GPU, FULL, build, launch, TE-3 candidate or G8/G8-C validation.
- `LONG_HORIZON_SEALED_AIR_DRIFT_BUDGET` and
  `TE2_CANDIDATE_HUD_LABEL_POLISH` remain non-blocking later work. Another
  same-tick scene 3/4 comparison is not required.
- D-017 authorizes only docs/research/adversarial/reference work around the
  proposed Hybrid A+C TE-3D candidate. ADR-0006 is not accepted, TE-3 runtime
  is not started, and Air-pressure force, TE-4 and G9-B/C/D/E remain excluded.
