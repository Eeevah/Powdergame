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