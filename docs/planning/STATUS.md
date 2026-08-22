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
| G9-A First Playable Sandbox | **USER ACCEPTED WITH KNOWN FOLLOW-UP** — Inspector continuity v2 **USER ACCEPTED**; G9-B remains separately gated |
| Thermal Environment / Ignition Causality | **TE-2 USER ACCEPTED WITH KNOWN FOLLOW-UP**; **TE-3 USER ACCEPTED WITH KNOWN FOLLOW-UP**; **TE-4I USER ACCEPTED WITH KNOWN FOLLOW-UP**; Scenes 1–4 **DIRECT OBSERVATION CONSISTENT**; **TE-4D v1/v2/v3 AND D-031 BLOCKED / IMMUTABLE**; runtime `8d9e8cb...`, observability `a7622bf...`; **ADR-0012 ACCEPTED FOR CURRENT IMPLEMENTATION**; TE-5R0 **LOCAL RELAXING PHASE-LOAD PRESSURE DESIGN BLOCKED / CRITICAL 0 / HIGH 3 / RUNTIME NOT STARTED** |
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
- TE-5C replacement-design baseline: `6a1c83fad702d18f2d24365a4fc747ab74225f5c`
- TE-3 pressure-decoupled runtime source: `41467219819c5d0cb3eab8ae22b652449da20480`
- TE-3 direct-review surface candidate: `c2f4f2bb16b00801a72ff6e4a54726cc69674bad`
- TE-3 Sandbox phase-truth candidate: `89d2400d677dec7e39cba76234c18d8b2363a496`
- TE-4I final runtime source: `8d9e8cbe3b6ac651335b5a728ef491abeae4772a`
- TE-4I Scene 4 observability source: `a7622bf2106a11e731a46018a4afe30d236b9304`
- Shared `main`: 이 상태로 승격되지 않음

TE-4D v1's only frozen reference process completed zero trials after an
equal-metric coefficient tie selected a different Oil tuple than the
preregistered identity. It remains immutable blocked history. D-029 authorizes
the separate `TE4-IGNITION-KINETICS-REFERENCE-V2` identity and selects exact
coefficients, packed u6 exposure, non-Vacuum Air-face access, chemical-Q cap
accounting and consume-before-emission. The manifest-bound v2 process completed
`1/1`: 13 reference fixtures PASS and exactly four production fixtures remain
`NOT_ESTABLISHED`. Fresh review found Critical `0` / unresolved High `3` /
Medium `1` and invalidated the broad transaction/frontier claim. TE-4D v2 is
**DESIGN BLOCKED** and TE-4 runtime remains **NOT STARTED**.

D-030 preserves v1/v2 as immutable blocked history and authorizes the distinct
`TE4-IGNITION-KINETICS-REFERENCE-V3` identity. V3 fixes the precondition
lifetime as `COMBUSTION_STAGE_SNAPSHOT`, requires mutation-derived transaction
receipts and a frozen independent complete F07/F08 oracle, and narrows the
coefficient claim to user-selected identity validation without optimality.
The source-audited future projection remains 42 passes / 84 queries / 1,344
profiler bytes / zero persistent or scratch bytes. Evidence and independent
review are pending; TE-4 runtime remains **NOT STARTED**.

The one v3 process completed `1/1` and reported the locked `13/4/0/0/0`
aggregate plus exact frozen-oracle matches. Fresh-context review nevertheless
found Critical `0` / unresolved High `3` / Medium `1` / Low `1`: F15B's next
Air snapshot is hardcoded, the auditor trusts SUT semantic labels, and F09 is
not one duration-driven chemical-heat/final-tick state machine. V3 is therefore
**DESIGN BLOCKED**; its files are immutable and TE-4 runtime is **NOT STARTED**.

D-031 authorizes a distinct transaction-only supplement for topology-derived
F15B, independent semantic auditing, full Oil/Wood fuel lifecycles and
re-auditable snapshots. It does not reopen or rerun v3's broad campaigns,
oracle or coefficients. The one process completed `1/1`, but fresh review found
Critical `0` / High `3` / Medium `2`: missing F15B settle, caller-classified
semantic counters and topology-free Air receiver transfer. The supplement is
**BLOCKED** and immutable; runtime remains **NOT STARTED**.

D-032 ends further synthetic reference repair and authorizes the locked
ignition semantics as an implementation-first production candidate. ADR-0012
is **PROPOSED — IMPLEMENTATION EVIDENCE AVAILABLE / USER ARCHITECTURE REVIEW
PENDING**. Final source `8d9e8cb...` passes F01..F17 production fixtures and
the canonical FULL after two recorded invalid-source attempts. The canonical
candidate and artifact are available, but user acceptance is not claimed. See
[`TE-4I evidence`](../evidence/THERMAL_ENVIRONMENT_TE_4_IGNITION_KINETICS_2026-08-23.md).

The direct user review found that Scene 4's source-side changes did not make
the one-cell Smoke target visible. D-033 preserves the production physics and
requires direct target/receiver evidence. Candidate source `a7622bf...` now
shows fixed target and receiver rows, Smoke count and a target-identity-driven
outline. Exact actual-state tests pass. D-034 records the user's direct
`READY -> EMITTED -> EXTINGUISHED -> DECAYED` observation at ticks
`0/1/2/1184`, accepts Scenes 1–4 as consistent, accepts TE-4I with known
follow-up, accepts ADR-0012 for the current implementation, and closes Q-017.
See the
[`Scene 4 remediation receipt`](../evidence/TE4_SCENE4_SMOKE_OBSERVABILITY_REMEDIATION_2026-08-23.md).

D-035 preserves TE-5B/C/D/X/Q as blocked immutable history, supersedes their
exact extent/zero-pressure/recovery/certification/permanent-sealed-pressure
constraints, and authorizes a docs-only **LOCAL RELAXING PHASE-LOAD PRESSURE**
candidate. The candidate derives Air background pressure, reuses the existing
dynamic pressure pair, and may not add token, matching, CCL, phase packet,
Vapor-volume field or runtime state. Independent review found three High
source-contract contradictions: unavailable pre-transition phase context,
unavailable/inconsistent fresh generic impulse, and legacy activity that
cannot sleep at exact nonuniform equilibrium. ADR-0013 therefore remains
**PROPOSED — DESIGN BLOCKED / ARCHITECTURE REVISION REQUIRED**; TE-5 runtime
remains not started. Canonical blocked authorities are
[`ADR-0013`](../architecture/decisions/ADR-0013-local-relaxing-phase-load-pressure.md),
the [specification](../specs/PRESSURE_VACUUM_COUPLING_SPEC.md), the
[validation contract](../development/PRESSURE_VACUUM_COUPLING_VALIDATION.md)
and the [TE-5R0 plan](TE5R_PRESSURE_VACUUM_REENTRY.md).

D-036 conserves GitHub Actions through 2026-09-01. TE-5R0 closes locally with
no Wiki branch/PR/CI/merge and no intermediate Powdergame push. The stale Wiki
project snapshot is temporarily acknowledged. At most one final `[skip ci]`
feature push is allowed only after local workflow-trigger inspection proves
that no relevant hosted workflow can run; otherwise the coherent commit stays
local for deferred publication.

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

D-020 rejects that token and evaluates [`TE-5C`](TE5_LOCAL_VAPOR_CAPACITY_PRESSURE.md).
Its one-shot grid/time result reported the vacancy-walk and F01–F13 checks as
passed but failed the predeclared asymmetric open-capacity control; fresh
review found that several reported checks do not implement their named
obligations and recorded unresolved Critical `0` / High `6`. ADR-0008 is
**PROPOSED — DESIGN BLOCKED**; the next decision must explicitly permit
persistent phase-volume state. No TE-3/TE-5 runtime has started.

D-021 preserves both failed stateless candidates and authorizes a
docs/reference-only TE-5D persistent Vapor extent plus dedicated phase-pressure
design program. The one-shot proof returned DESIGN BLOCKED: it did not meet the
all-labeled 6×6 obligation, and static analysis found a legal eight-source
persistent alternating chain beyond the frozen depth-six reassignment. Wider
matching scope is required; runtime remains unauthorized.

Fresh independent review confirmed the matching counterexample and found five
additional High issues: proof overclaim, receiver-blind matching, missing
editor reciprocal cleanup, pass/binding/scratch understatement and undefined
phase-pressure movement ownership. Final counts are Critical `0` / High `6` /
Medium `2`; review SHA-256 is
`73adaf56bea1589d425d89ba9430a7f50f3d0b9cf50f5b8fdc2155f263968ed6`.

D-022 preserves all three blocked designs and authorizes a docs/reference-only
TE-5X comparison of exact persistent-extent matching, shared gas-chamber
capacity and a conservative Vapor-volume Environment field. The only combined
process exited at the NetworkX version guard before candidate evaluation, so
all A/B/C and fixture counts are zero. The failure receipt is parseable but is
not proof evidence; the one-shot rule forbids a rerun. No model is accepted,
ranked or recommended and TE-5X is DESIGN BLOCKED. Fresh comparative review
ended at Critical `0` / High `11` / Medium `0`; review SHA-256 is
`c424c8336d3b34784f6a3ffbb37421ceca8888608c198da45793774b49ffb579`.

D-023's packet candidate is also preserved as blocked history. D-024 rejects
ADR-0011 as the active representation, restores ADR-0006 one-Cell/one-quantity
semantics and supersedes the atomic TE-3/TE-5 activation constraint. Runtime
source `41467219819c5d0cb3eab8ae22b652449da20480` implements only the two
phase-energy buffers, passes actual TE3-F01–F15 fixtures and final-source FULL,
and exposes the paused `phase-cycle`/`te3` candidate. Water creates no second
Steam and no phase expansion/blocked pressure. Historical G5 evidence remains
source-bound. D-025 records Scenes 2–4 as direct-observation consistent and
keeps Scene 1 pending. App source `89d2400...` preserves the 24-byte Inspector
while showing authoritative phase energy in Sandbox, discloses the sealed
world/no external ambient sink, and provides a local-only one-click shortcut.
Scene 1 source `e9f4a37...` replaces the below-boiling 3,885-Cell fixture with
a finite one-shot 288-Cell beaker. Actual ticks record boiling progress at 72,
Steam/rise at 320, condensation progress at 336, upper Water at 552 and fall at
584 while family quantity stays exact.
`WATER_STEAM_PRESSURE_VOLUME_REDESIGN` is deferred/not started. Receipts:
[`THERMAL_ENVIRONMENT_TE_3_PHASE_CYCLE_2026-08-21`](../evidence/THERMAL_ENVIRONMENT_TE_3_PHASE_CYCLE_2026-08-21.md).
[`TE3_SANDBOX_PHASE_TRUTH_REMEDIATION_2026-08-22`](../evidence/TE3_SANDBOX_PHASE_TRUTH_REMEDIATION_2026-08-22.md).
[`TE3_SCENE1_PHASE_CYCLE_REMEDIATION_2026-08-22`](../evidence/TE3_SCENE1_PHASE_CYCLE_REMEDIATION_2026-08-22.md).

---

## 비공식 진단 artifact

Preserve pending a separate retention decision:

- `g8c-pilot-8ee1ae238c32-c64090539536`
- `g8c-pilot-8ee1ae238c32-6341f4f59218`
- `g8c-aggregation-replay-20260819T015515996891Z-fc408076b67a`

They are not official performance evidence. Do not prune them without a separate retention decision.

---

## 현재 설계 작업

**TE-3 USER ACCEPTED WITH KNOWN FOLLOW-UP.** D-027 preserves D-024, which
rejects ADR-0011 and preserves all TE-5/packet failures as history. Runtime
Source `4146721...` implements the accepted one-Cell/one-quantity enthalpy
model without Water phase pressure. D-025 records Scenes 2–4 as consistent;
the remediated Scene 1 is direct-observation consistent under D-026. D-027
records the user's whole-candidate acceptance while retaining
`WATER_STEAM_PRESSURE_VOLUME_REDESIGN` as a known deferred follow-up. No TE-5
pressure-volume architecture is selected or started.

## 다음 행동

1. authorize **TE-5 Pressure/Vacuum architecture re-entry** only through a new
   direct decision;
2. begin that decision by reading every immutable TE-5B/C/D/X/Q blocker and
   deciding which architecture constraints may change;
3. keep TE-5 implementation, TE-6 and G9-B/C/D/E **NOT STARTED**.

## 아직 별도 결정인 것

- product edge mode, Vacuum combustion and later ignition-dose choices remain gate-owned
- user revision of finite-capacity ownership and the exclusive-relief-token model; occupancy-only Air relief, exact mode encoding, inherited pressure scalar 100 and finite-headspace causal fixture remain secondary decisions
- full TE-5 pressure law and source-bound runtime/user evidence
- later G9-B/C/D/E scope progression after the first candidate
- shared `main` promotion
- final M0 `ACHIEVED`

M0 `ACHIEVED`: **NO** — G9 direct product validation remains.
