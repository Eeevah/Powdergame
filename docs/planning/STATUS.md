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
| G9-A First Playable Sandbox | **USER ACCEPTED WITH KNOWN FOLLOW-UP** — Inspector continuity v2 **USER ACCEPTED**; atomic TE-3/TE-5 runtime evidence remains a separate prerequisite before G9-B |
| Thermal Environment / Ignition Causality | **TE-2 USER ACCEPTED WITH KNOWN FOLLOW-UP** — candidate source `0977281...`; **TE-3D ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS**; ADR-0006 accepted for future atomic implementation; **TE-5B DESIGN BLOCKED / REFERENCE ABSTRACTION PASS / FINITE-CAPACITY HIGH OPEN**; ADR-0007 Proposed; TE-3 and TE-5B runtime not started; G9-B prerequisite |
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
- TE-2 runtime source: `fb7e568e21012b6067269f4e1b82c36c865023d0`
- TE-2 review-remediation candidate source: `097728128343cf89383920c968a010b3dcf8e8c0`
- TE-3D accepted architecture / TE-5B design baseline: `d7500e219af6f670be05f830b50c232d2bb53077`
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
- direct review status: Draw/Ice/Steam/palette/Heat-Cool revisions were observed; continuity v2 was subsequently directly reviewed and **USER ACCEPTED**; G9-A overall is **USER ACCEPTED WITH KNOWN FOLLOW-UP**;
- continuity v2: requested hover and presented sample are separate; rapid second/third hover changes retain one bounded previous sample with its original Cell/Material/tick/freshness; compact tooltip requires a fresh current-Cell sample; Ready/Held/Sampling use one fixed detailed panel geometry; reset/preset/epoch/failure clears the held sample immediately;
- validation: source `a00e39b...` Inspector `11/11` and Inspector UI `3/3`, affected check/clippy, strict policy audit and exactly one explicit Sandbox 3-frame release bounded launch check all pass;
- FULL: `0` because validation-plan classified it recommended, not required, and no engine/Core/fixture/Cargo graph/shared Simulation layout changed.

The latest direct disposition accepts G9-A Inspector continuity and records G9-A overall **USER ACCEPTED WITH KNOWN FOLLOW-UP**. This does not close the separate Thermal Environment prerequisite or authorize G9-B.

Thermal follow-up is registered in [`THERMAL_TRANSPORT_IGNITION_CAUSALITY.md`](THERMAL_TRANSPORT_IGNITION_CAUSALITY.md). D-013/ADR-0005 lock Air as a separate mass/energy Environment field and distinguish Atmosphere, Vacuum, EMPTY and Void. D-014/source `1a722d...` implements TE-1 occupancy hygiene. D-015/source `fb7e568...` implements TE-2 full-resolution Air flow, donor-energy advection, unified passive thermal exchange, activity/wake and the atomic Celsius-like migration. The 30-case small-delta CPU/GPU regression pins the deadband as a shared work gate. Candidate source `0977281...` remediates only controls, bounded diagnostics and scene staging. Direct re-review confirmed F/N/I, all four scene contracts and reset/controls; D-017 records TE-2 **USER ACCEPTED WITH KNOWN FOLLOW-UP** while preserving every prior automated/runtime/performance boundary. `LONG_HORIZON_SEALED_AIR_DRIFT_BUDGET` and `TE2_CANDIDATE_HUD_LABEL_POLISH` are non-blocking. D-018 accepts the Water/Steam Hybrid A+C architecture with locked amendments in [`ADR-0006`](../architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md), [`PHASE_THERMODYNAMICS_SPEC`](../specs/PHASE_THERMODYNAMICS_SPEC.md) and [`PHASE_THERMODYNAMICS_VALIDATION`](../development/PHASE_THERMODYNAMICS_VALIDATION.md). The one-shot amended proof and fresh v2 review passed with unresolved Critical `0` / High `0`. D-019 authorized a docs/reference-only TE-5B design program around an exclusive local volume-relief token and the existing gauge-pressure grammar. Its fixed-seed pure arbitration/accounting model passed its only execution, but the model contained no grid or cross-tick vacancy conservation. Fresh-context review found that staggered endpoint timing lets 1:1 Steam movement pass one EMPTY vacancy down a sealed Water column, so finite headspace need never become blocked and the required G5 pressure chain is not guaranteed. ADR-0007 is **PROPOSED — DESIGN BLOCKED / USER ARCHITECTURE REVISION REQUIRED**. TE-3 and TE-5B runtime remain **NOT STARTED**.

---

## 비공식 진단 artifact

Preserve pending a separate retention decision:

- `g8c-pilot-8ee1ae238c32-c64090539536`
- `g8c-pilot-8ee1ae238c32-6341f4f59218`
- `g8c-aggregation-replay-20260819T015515996891Z-fc408076b67a`

They are not official performance evidence. Do not prune them without a separate retention decision.

---

## 기술 blocker

**TE-5B DESIGN BLOCKED / REFERENCE ABSTRACTION PASS / RUNTIME NOT STARTED.** D-018 closes the current `1 Water -> up to 2 Steam -> up to 2 Water` quantity defect with one Water-equivalent quantity per Cell plus two reversible phase-energy halves. D-019's bridge reused movement reachability, exclusive proposal/claim ownership and gauge-pressure consequences, but its non-mutating same-tick token does not consume volume. In a sealed stagger-heated column, Steam moves into the only EMPTY Cell and vacates its source; the next Water reaches the endpoint only after that vacancy arrives, so the vacancy walks through every later Water Cell with zero pressure and no simultaneous swap stop. This unresolved High makes F05/F11 and atomic G5 continuity unsatisfiable under the current candidate. The one-shot pure model's failed checks remain `0` only inside its narrower no-grid arbitration/accounting scope. Historical G5 evidence remains source-bound and cannot be rebound.

## 다음 행동

1. obtain a user architecture revision that explicitly owns finite-capacity consumption and says which no-state, non-mutation or 1:1 constraint, if any, may change;
2. do not reinterpret the existing abstraction PASS as clearing the vacancy-conservation High or rerun it in this task;
3. keep TE-3 and TE-5B runtime **NOT STARTED** and the current G5 Water path active until a later separately authorized same-source replacement exists;
4. do not start full TE-5 Air/background-pressure coupling, TE-4, G9-B/C/D/E, Discovery, Save/Load, Rewind, broad presentation or optimization.

## 아직 별도 결정인 것

- product edge mode, Vacuum combustion and later ignition-dose choices remain gate-owned
- user revision of finite-capacity ownership and the exclusive-relief-token model; occupancy-only Air relief, exact mode encoding, inherited pressure scalar 100 and finite-headspace causal fixture remain secondary decisions
- full TE-5 pressure law, atomic implementation authorization and source-bound runtime/user evidence
- later G9-B/C/D/E scope progression after the first candidate
- shared `main` promotion
- final M0 `ACHIEVED`

M0 `ACHIEVED`: **NO** — G9 direct product validation remains.
