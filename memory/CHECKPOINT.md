# Checkpoint — G8-C verified, docs closed, G9 brief pending — 2026-08-19 KST

## Validity

This is Powdergame's current session-resume coordinate after G8-C official verification, Ballast integration and the G8 documentation closure. It is not a substitute for official evidence or live Git. Before code work, exact-fetch the active branch and treat contradictory live state as authoritative.

## Repository coordinate

- Active branch: `feature/m0-g8c-official-matrix`
- G8-C sealed runtime source: `4653d7c2e09e93f80fb81eeb73458d992c86858f`
- Ballast integration merge: `6b5f0201f882f212f9916521aec689261d97b4a6`
- G8-C evidence/status closure: `51699d1a73be6f484a8436720463d1aa6c037de9`
- Integrated Ballast checkpoint activation: `dd2f897f29773a44af7ce23fe3e5cf8d07f8110b`
- Milestone contract/status separation: `00e8860dfbf0a05482aa8128dfc683292b4364e8`
- G8-B closure: `18391e6a9fc8f9bc7b2757f3504366f106c05435`
- G8-B state: **CLOSED / FROZEN**
- G8-C state: **OFFICIAL CAPTURE COMPLETE / INDEPENDENT VERIFICATION PASS**
- Matrix recommendation: **PROCEED_TO_G9**
- G9 / optimization / main promotion: **NOT STARTED**

`docs/planning/STATUS.md` now contains only live state and next decisions. `docs/planning/MILESTONES.md` contains Gate contracts without copied volatile status. `docs/HANDOFF.md` is retired as a live checkpoint.

## Story so far

All five G8-B scenarios and Cell Inspector v0 were user accepted. G8-C measured those workloads through headless production throughput, synchronized GPU breakdown, windowed simulation/render coexistence and separate GPU render timing.

Two non-evidence pilots found measurement-tool defects rather than engine-performance defects:

1. a stale initial window-size payload was mistaken for a live resize;
2. the coordinator searched for internal `wall_ms_per_tick` instead of canonical external `wall_per_tick` + `ms/tick`.

Both contracts were corrected without changing historical producers, engine physics, production WGSL or fixtures. A zero-subprocess aggregation replay passed, clean source was sealed, and the official Matrix plus independent verifier each ran exactly once.

## Official G8-C evidence

- Matrix ID: `g8c-official-matrix-4653d7c2e09e-64df60ba0d79`
- Source: `4653d7c2e09e93f80fb81eeb73458d992c86858f`
- Benchmark EXE SHA-256: `29131418a091d1657960c8cf1307d533582fa69e140af330b69be530c4394ed5`
- Windows EXE SHA-256: `2c1670bff506cc9793da9e3708cafb28b6485d14bc577abbcb5faa04f897c4e5`
- Receipt SHA-256: `1fbf4599893cc29e99b6033996b42fcdf025aac0b421cb80b95b3e55807455f6`
- Package SHA-256: `92f8b85cc0e34ea6e71a9f6b4fc95b0f70704263a0f798a69a830cce1d40b729`
- Verification SHA-256: `77c7e1c982296277c451de02c3dca68fa6d7d9a90e9fd5426c4dffa1abd9bb0d`
- Independent reconstruction: 230 fields, mismatch `0`

Performance boundary:

- minimum Mode A P50 `931.602 TPS`
- minimum 60-TPS headroom `15.527×`
- maximum Mode B P95 `1.046784 ms`
- minimum Mode C simulation `59.898580 TPS`
- Mode C deadline misses / catch-up / dropped frames `0 / 0 / 0`
- maximum Mode C frame P95 `4.2005 ms`
- maximum Mode D render P95 `0.021280 ms`
- persistent tracked GPU allocation `184,576,672 bytes` per scenario

No current 60-TPS simulation, rendering, coexistence or persistent-memory blocker was established. Active / Sleep is the largest grouped subsystem, but optimization remains deferred until actual product work or a new measurement establishes a concrete blocker.

Canonical closure: `docs/evidence/G8_C_OFFICIAL_MATRIX_2026-08-19.md`.

## Non-evidence diagnostics

Preserve pending a separate retention decision:

- `g8c-pilot-8ee1ae238c32-c64090539536` — lifecycle failed
- `g8c-pilot-8ee1ae238c32-6341f4f59218` — aggregation failed after all 15 workers passed
- `g8c-aggregation-replay-20260819T015515996891Z-fc408076b67a` — passing parser/publication replay, `non_evidence=true`, zero launched processes

Do not report these as official performance evidence and do not rerun the official Matrix because memory/docs changed.

## Decided

- D-003 — Heavy Mixed is accepted with known follow-up; immutable automatic verdict and evidence remain unchanged.
- D-004 — Ballast is Powdergame's single active session-continuity workflow.
- D-005 — Ballast is selectively reversible; immediate Hook disable first, then merge-based project rollback; product/evidence history is preserved.

## Waiting on the user

- Q-002 — G8-A same-SHA visual durable disposition remains separate and pending.
- Q-004 — Define and authorize the G9 product brief, request a narrower review, or explicitly override the Matrix recommendation.

## Next first action

**Requirements first; no implementation yet.** Ask the user to decide:

1. initial sandbox state: blank world, guided starter world, or both;
2. initial Matter palette visibility;
3. G9-A editor MVP tools and default controls;
4. Discovery MVP timing and visible feedback;
5. whether save/load or Rewind belongs in the first slice or is deferred;
6. the exact manual acceptance session that closes G9/M0;
7. how to dispose of the separate G8-A visual requirement.

Then create a bounded G9 implementation plan and branch. Do not start optimization, new Matter, main promotion or runtime work before that product brief is accepted.

## Avoid repeating

- Do not treat `PROCEED_TO_G9` as automatic user authorization.
- Do not keep `docs/HANDOFF.md` and this checkpoint as parallel live coordinates.
- Do not copy volatile status into Milestones again.
- Do not rerun exact-source official evidence after docs/memory changes.
- Do not optimize Active / Sleep merely because it is the largest group; require a product blocker.
- Do not promote agent proposals into `memory/DECISIONS.md`.
- If Ballast misbehaves, set `BALLAST_DISABLE=1` or untrust the Hook before changing Git.
