# Archived Checkpoint — G8-C verified, docs closed, G9 brief pending

Archived: 2026-08-19 KST  
Historical only. Replaced after the user approved the G9 product brief and closed the remaining G8 visual-disposition question.

## Validity

This was Powdergame's session-resume coordinate after G8-C official verification, Ballast integration and the G8 documentation closure. It is retained to preserve the decision boundary that existed before G9 scope approval.

## Repository coordinate at archive time

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

No current 60-TPS simulation, rendering, coexistence or persistent-memory blocker was established. Active / Sleep was the largest grouped subsystem, but optimization remained deferred until actual product work or a new measurement established a concrete blocker.

## Pending decisions at archive time

- G8-A same-SHA visual durable disposition remained separate and pending.
- Define and authorize the G9 product brief, request a narrower review, or override the Matrix recommendation.

## Then-next action

Requirements first; no implementation. Ask the user to decide:

1. initial sandbox state;
2. initial Matter palette visibility;
3. editor MVP tools and controls;
4. Discovery timing and feedback;
5. Save/Load or Rewind scope;
6. manual acceptance session;
7. G8-A visual disposition.

## Avoid repeating

- Do not treat `PROCEED_TO_G9` as automatic user authorization.
- Do not keep `docs/HANDOFF.md` and the checkpoint as parallel live coordinates.
- Do not copy volatile status into Milestones.
- Do not rerun exact-source official evidence after docs/memory changes.
- Do not optimize Active / Sleep merely because it is the largest group.