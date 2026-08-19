# Checkpoint — G8-C official matrix verified; product decision pending — 2026-08-19 KST

## Validity

This checkpoint records the verified G8-C Official Performance Matrix. It is the active session-resume coordinate after the G8-C writer reached a clean, upstream-equal safe stop. The official Matrix and its independent verification are exact-source evidence; the earlier pilots and aggregation replay remain non-evidence diagnostics.

No G9 or optimization work has started. The matrix recommendation is an evidence-derived recommendation, not an automatic user decision.

## Repository coordinate

- G8-B closure commit: `18391e6a9fc8f9bc7b2757f3504366f106c05435`
- Legacy launcher retirement commit: `8ee1ae238c324c1db1d7e2882af071fec179a8f1`
- G8-B state: **CLOSED / FROZEN**
- Active G8-C branch: `feature/m0-g8c-official-matrix`
- Sealed G8-C source: `4653d7c2e09e93f80fb81eeb73458d992c86858f`
- Source commit: `fix: complete the G8-C measurement aggregation contract`
- Writer state at final report: clean, local/remote divergence `0/0`
- Worktree count: `3`; target cache preserved; workspace FULL `0`

## Story so far

The first pilot exposed a stale initial window-size event. The bounded lifecycle remediation made live `window.inner_size()` authoritative while keeping genuine noncanonical sizes fatal. The replacement pilot then completed all five Headless A/B, five Mode C, and five Mode D workers, but exposed a historical CSV adapter mismatch. The coordinator expected internal `wall_ms_per_tick`, while the canonical producer emits `wall_per_tick` with unit `ms/tick`.

The user authorized one aggregation-only replay over hash-bound existing raw outputs and one conditional official capture. The explicit adapter now maps the external producer vocabulary to the internal model without changing historical G8-A/G8-B CSV schemas. The replay launched no executable or GPU process and passed report, Receipt, package, and independent-verifier validation. Clean source was then sealed and the official Matrix was captured exactly once.

Official result:

- Matrix ID: `g8c-official-matrix-4653d7c2e09e-64df60ba0d79`
- Official capture / independent verification / package: `1 / 1 / 1`
- Independent verifier: `verified=true`; 230 matrix fields recomputed from raw inputs; mismatch `0`
- Recommendation: **PROCEED_TO_G9**
- G9 and optimization implementation: not started

## Official evidence identity

- Source SHA: `4653d7c2e09e93f80fb81eeb73458d992c86858f`
- Benchmark executable SHA-256: `29131418a091d1657960c8cf1307d533582fa69e140af330b69be530c4394ed5`
- Windows executable SHA-256: `2c1670bff506cc9793da9e3708cafb28b6485d14bc577abbcb5faa04f897c4e5`
- Matrix Receipt SHA-256: `1fbf4599893cc29e99b6033996b42fcdf025aac0b421cb80b95b3e55807455f6`
- Matrix package SHA-256: `92f8b85cc0e34ea6e71a9f6b4fc95b0f70704263a0f798a69a830cce1d40b729`
- Verification result SHA-256: `77c7e1c982296277c451de02c3dca68fa6d7d9a90e9fd5426c4dffa1abd9bb0d`
- Run `HASHES.sha256`: `8ade901cc359c2cdfb750f01fff35f0fae463046757e6cee4ba44100c0b8c260`
- Official run / delivery bytes: `2,495,594,656 / 70,073,360`

These identities support only the authenticated G8-C matrix contract. They do not transfer to another source, build, hardware/backend, config, or altered artifact set.

## Performance conclusion

Across the five accepted official workloads:

- minimum Mode A P50: `931.602 TPS` (Pressure Burst)
- Mode A 60-TPS headroom: at least `15.527x`
- maximum Mode B GPU envelope P95: `1.046784 ms`
- minimum Mode C simulation rate: `59.898580 TPS`
- Mode C deadline misses / catch-up ticks / dropped frames: `0 / 0 / 0`
- maximum Mode C frame P95: `4.2005 ms`
- maximum Mode D GPU render P95: `0.021280 ms`
- persistent tracked GPU allocation per scenario: `184,576,672 bytes` (`~0.172 GiB`, `~0.537%` of RTX 5090 32 GiB)
- repeated largest Mode B grouped P50: Active / Sleep management

The matrix found no current simulation, rendering, memory, or coexistence blocker for the 60-TPS M0 product target. Active / Sleep is the largest measured subsystem group, but the evidence does not justify implementing compaction, indirect dispatch, packing, f16, or another optimization before G9.

## Non-evidence diagnostics retained

- lifecycle-failed pilot: `g8c-pilot-8ee1ae238c32-c64090539536`
- aggregation-failed replacement pilot: `g8c-pilot-8ee1ae238c32-6341f4f59218`
- passing aggregation replay: `g8c-aggregation-replay-20260819T015515996891Z-fc408076b67a`

The replay is explicitly `non_evidence=true` and launched zero measurement subprocesses. All three diagnostic artifacts remain preserved pending a separate retention/prune decision.

## Decided

- D-003 — Heavy Mixed World is user accepted with known follow-up; its immutable automatic verdict and evidence remain unchanged.
- D-004 — Ballast is the approved single active Powdergame session-continuity workflow after commit-preserving integration.
- D-005 — Ballast remains selectively reversible; squash merge is forbidden.

## Waiting on the user / operator

- Q-004 — Choose the next product action from the verified recommendation: authorize G9 planning/implementation, request a narrower human review, or explicitly override the recommendation.
- Q-002 — Same-SHA G8-A user visual durable disposition remains pending. It is a separate closure item and must not be inferred from G8-C.
- Q-005 — Integrate PR #4 into the clean G8-C line with commit boundaries preserved, then record the exact merge-based rollback.

## Next first action

1. Integrate PR #4 into `feature/m0-g8c-official-matrix` with a merge commit or rebase-and-merge; never squash.
2. Record the exact integrated rollback command and retire `docs/HANDOFF.md` as a live checkpoint.
3. Ask the user for the G9 product brief before implementation. Do not start G9 from the machine recommendation alone.
4. Keep optimization deferred unless later product work or a new benchmark establishes a concrete blocker.

## Tried / avoid repeating

- Do not rerun the official Matrix merely because docs or memory change.
- Do not report the two failed pilots or aggregation replay as official performance evidence.
- Do not rename historical producer fields to fit a consumer; keep the external schema and internal model separated by a strict adapter.
- Do not infer user authorization for G9, optimization, G8 closure, main promotion, or G8-A visual acceptance from `PROCEED_TO_G9`.
- Do not maintain `docs/HANDOFF.md` and this checkpoint as parallel live session coordinates after cutover.
- Do not squash PR #4.
- If Ballast Hook injection misbehaves, set `BALLAST_DISABLE=1` or remove Hook trust before changing Git.
