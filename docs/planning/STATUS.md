# Powdergame Status

이 문서는 **현재 실제 상태와 다음 행동만** 기록한다. 장기 방향은 [`ROADMAP.md`](ROADMAP.md), Gate 완료 계약은 [`MILESTONES.md`](MILESTONES.md), 상세 실행 증거는 [`../evidence/`](../evidence/)를 따른다. 과거 상태 이력은 Git history와 각 evidence 문서에 보존한다.

---

## 현재 한눈에 보기

| 구간 | 상태 |
|---|---|
| M0 — First World | **IN_PROGRESS** |
| G0–G7 | **PASS / CLOSED / FROZEN** |
| G8-A Measurement Substrate | **V5 OFFICIAL CAPTURE + INDEPENDENT VERIFICATION COMPLETE / SEPARATE VISUAL REQUIREMENT FORMALLY SUPERSEDED** |
| G8-B Benchmark Scenario Suite | **CLOSED / FROZEN** — 다섯 official scenario와 Cell Inspector v0 사용자 승인 완료 |
| G8-C Official Performance Matrix | **OFFICIAL CAPTURE COMPLETE / INDEPENDENT VERIFICATION PASS / `PROCEED_TO_G9`** |
| G8 Performance Evidence | **CLOSED / FROZEN** |
| G9-A First Playable Sandbox | **REVISED IMPLEMENTATION CANDIDATE / USER RE-REVIEW PENDING** — continuity v2 source `a00e39b2e00bfbd9ac28214c44cd22cc97542bb4`; latest review was revision-required |
| Thermal Environment / Ignition Causality | **TE-1 ENVIRONMENT STATE / OCCUPANCY HYGIENE IMPLEMENTED / TE-2 NOT STARTED** — Air transport, Air thermal exchange and Air pressure coupling remain disabled; G9-B prerequisite |
| G9-B/C/D/E | **NOT STARTED** |
| 최적화 구현 | **DEFERRED / NOT STARTED** |

## 현재 작업선

- G8-C sealed runtime source: `4653d7c2e09e93f80fb81eeb73458d992c86858f`
- Ballast integration merge: `6b5f0201f882f212f9916521aec689261d97b4a6`
- G8-B closure: `18391e6a9fc8f9bc7b2757f3504366f106c05435`
- Legacy launcher retirement: `8ee1ae238c324c1db1d7e2882af071fec179a8f1`
- Current implementation line: `feature/m0-g9-first-playable`
- G9-A continuity v2 tested source: `a00e39b2e00bfbd9ac28214c44cd22cc97542bb4`
- TE-1 runtime source: `1a722d239a16bade5772688fa822465d5cef4602`
- Shared `main`: 이 상태로 승격되지 않음

---

## G8 최종 상태

### G8-A

Preserved technical evidence:

- source: `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`
- capture: `g8a-v5-9abec9e-20260817T032827206Z`
- official package and independent verification complete
- independent verification: `11 / 11`, findings `0`

사용자는 별도 same-SHA visual durable requirement를 **FORMALLY SUPERSEDED**로 닫았다. 이는 G8-A capture를 소급 visual `PASS`로 바꾸는 것이 아니다. 이후의 직접 G8-B Gallery/Cell Inspector 사용자 승인과 independently verified G8-C windowed evidence가 더 넓은 제품·관찰 근거를 제공하므로 옛 visual session을 재실행하지 않는다.

Canonical closure: [`G8_PERFORMANCE_GATE_USER_CLOSURE_2026-08-19.md`](../evidence/G8_PERFORMANCE_GATE_USER_CLOSURE_2026-08-19.md)

### G8-B

| Scenario / Tool | Human disposition | Automatic disposition |
|---|---|---|
| Sand Fall | **USER ACCEPTED** | `PASS` |
| Water Flow | **USER ACCEPTED WITH KNOWN FOLLOW-UP** | `NEEDS_HUMAN_REVIEW` 유지 |
| Fire / Heat | **USER ACCEPTED** | `PASS` 유지 |
| Pressure Burst | **USER ACCEPTED WITH KNOWN FOLLOW-UP** | `NEEDS_HUMAN_REVIEW` 유지 |
| Heavy Mixed World | **USER ACCEPTED WITH KNOWN FOLLOW-UP** | `NEEDS_HUMAN_REVIEW` 유지; 14/14 hard PASS; blocker false |
| Cell Inspector v0 | **USER ACCEPTED WITH KNOWN FOLLOW-UP** | 최대 10 Hz / 100 ms hover delay 비차단 |

G8-B는 **CLOSED / FROZEN**이다. 이전 rejected/superseded candidate와 automatic verdict는 소급 변경하지 않는다. Water의 소수 free-surface 재배열, Pressure의 top-seam-only opening·작은 plume·넓은 terminal activity, Heavy의 감소 중인 broad Thermal tail은 G8-C workload에 포함되어 측정됐고 production-physics defect로 판정되지 않았다.

### G8-C

Evidence identity:

- Matrix ID: `g8c-official-matrix-4653d7c2e09e-64df60ba0d79`
- source: `4653d7c2e09e93f80fb81eeb73458d992c86858f`
- Receipt SHA-256: `1fbf4599893cc29e99b6033996b42fcdf025aac0b421cb80b95b3e55807455f6`
- package SHA-256: `92f8b85cc0e34ea6e71a9f6b4fc95b0f70704263a0f798a69a830cce1d40b729`
- verification SHA-256: `77c7e1c982296277c451de02c3dca68fa6d7d9a90e9fd5426c4dffa1abd9bb0d`
- verifier: `230` fields recomputed, mismatch `0`

Performance boundary:

- minimum Mode A P50: **931.602 TPS**
- minimum 60-TPS headroom: **15.527×**
- maximum Mode B P95: **1.046784 ms**
- minimum Mode C simulation: **59.898580 TPS**
- Mode C missed deadlines / catch-up / dropped frames: **0 / 0 / 0**
- maximum Mode C frame P95: **4.2005 ms**
- maximum Mode D render P95: **0.021280 ms**
- tracked persistent GPU bytes: **184,576,672 / scenario**

Recommendation: **`PROCEED_TO_G9`**. No current 60-TPS simulation, rendering, coexistence or persistent-memory blocker justifies optimization before the first playable sandbox.

Full evidence: [`G8_C_OFFICIAL_MATRIX_2026-08-19.md`](../evidence/G8_C_OFFICIAL_MATRIX_2026-08-19.md)

### G8 result

G8-A technical evidence is verified and its separate visual requirement has a durable user supersession. G8-B and G8-C are complete. Therefore G8 is **CLOSED / FROZEN**.

No G8 runtime evidence is rerun because this status or memory changes.

---

## G9 approved product brief

Canonical brief: [`G9_PRODUCT_BRIEF_2026-08-19.md`](G9_PRODUCT_BRIEF_2026-08-19.md)

Approved decisions:

1. **Starter Lab by default + immediately available New Blank World**.
2. **All current M0 Matter visible from the start**; Discovery does not unlock Matter.
3. G9-A editor MVP:
   - Matter selection
   - Draw / Erase
   - brush size
   - Heat / Cool
   - Pause / Play / Single Step
   - x1 / x4 / x16
   - Reset
   - Pan / Zoom
   - preset load
   - Cell Inspector reuse
4. Discovery begins after the editor core works, within the same G9 milestone, as a phenomenon-level Research Note.
5. Save/Load and Rewind are deferred from the first acceptance slice. Rewind remains a future core experiment tool rather than being canceled.
6. First user validation is approximately 10–15 minutes of unguided play. Primary strong signals are a voluntary second experiment and a causal explanation without exact threshold knowledge.
7. The first Codex implementation task is **G9-A only** and stops at a user-testable candidate.

G9 does not authorize new Matter, recipe/unlock progression, final FX, speculative optimization, G8 recapture, `main` promotion or M0 `ACHIEVED`.

### G9-A implementation candidate

- product surface: canonical BAT/EXE no-argument launch and explicit `sandbox`/`play` open Sandbox in the same `powdergame-windows.exe`; explicit `gallery`/`normal` preserve the G8-B surface;
- presets: fully editable Starter Lab and immediate New Blank World, both reset/staged through the production GPU world and start paused;
- tools: nine canonical M0 Matter, Draw, Erase, four brush sizes, Heat, Cool, Pause/Play, Single Step, x1/x4/x16, Reset, Pan, Zoom and existing Cell Inspector;
- edit boundary: bounded/coalesced command batch before simulation ticks, exact Current/Next hygiene, affected chunk plus clipped neighbor halo wake, no CPU world truth or pointer-driven full-world readback;
- camera: one finite physical-pixel transform shared by rendering, picking and Inspector hover;
- direct review status: Draw/Ice/Steam/palette/Heat-Cool revisions were observed in the build, but the user still saw rapid Inspector on/off flicker; no broader USER ACCEPTED verdict is inferred;
- continuity v2: requested hover and presented sample are separate; rapid second/third hover changes retain one bounded previous sample with its original Cell/Material/tick/freshness; compact tooltip requires a fresh current-Cell sample; Ready/Held/Sampling use one fixed detailed panel geometry; reset/preset/epoch/failure clears the held sample immediately;
- validation: source `a00e39b...` Inspector `11/11` and Inspector UI `3/3`, affected check/clippy, strict policy audit and exactly one explicit Sandbox 3-frame release bounded launch check all pass;
- FULL: `0` because validation-plan classified it recommended, not required, and no engine/Core/fixture/Cargo graph/shared Simulation layout changed.

This is not user acceptance. The revised candidate was **USER RE-REVIEWED / REVISION REQUIRED**; continuity v2 requires direct user re-review.

Thermal follow-up is registered in [`THERMAL_TRANSPORT_IGNITION_CAUSALITY.md`](THERMAL_TRANSPORT_IGNITION_CAUSALITY.md). D-013/ADR-0005 lock Air as a separate mass/energy Environment field and distinguish Atmosphere, Vacuum, EMPTY and Void. TE-0R reuse survey, TE-0 design, TE-0A independent review and TE-0B reference proof are complete with Critical/High blocker zero. D-014/source `1a722d...` implements TE-1 state, canonical staging, occupancy hygiene, receiver-gated phase/Smoke spawn and exact blocked phase pressure. Air does not yet flow or exchange heat, and it does not couple pressure to Matter. TE-2 and later thermal/ignition gates are not started.

---

## 비공식 진단 artifact

Preserve pending a separate retention decision:

- `g8c-pilot-8ee1ae238c32-c64090539536`
- `g8c-pilot-8ee1ae238c32-6341f4f59218`
- `g8c-aggregation-replay-20260819T015515996891Z-fc408076b67a`

They are not official performance evidence. Do not prune them without a separate retention decision.

---

## 기술 blocker

**없음.**

## 다음 행동

1. run `run_powdergame.bat sandbox` and re-review Inspector continuity v2 on the new TE-1 descendant executable;
2. accept, revise or reject the continuity v2 editor experience;
3. authorize TE-2 separately only after the TE-1 evidence boundary is accepted;
4. do not start G9-B/C/D/E, Discovery, Save/Load, Rewind, broad presentation or optimization before those decisions.

## 아직 별도 결정인 것

- G9-A continuity v2 user re-review
- Thermal Environment TE-2 authorization; Air flow/thermal coefficients and later ignition-dose coefficients remain gate-owned
- later G9-B/C/D/E scope progression after the first candidate
- shared `main` promotion
- final M0 `ACHIEVED`

M0 `ACHIEVED`: **NO** — G9 direct product validation remains.
