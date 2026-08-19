# Powdergame Status

이 문서는 **현재 실제 상태와 다음 의사결정만** 기록한다. 장기 방향은 [`ROADMAP.md`](ROADMAP.md), Gate 완료 계약은 [`MILESTONES.md`](MILESTONES.md), 상세 실행 증거는 [`../evidence/`](../evidence/)를 따른다. 과거의 긴 상태 이력은 Git history와 각 evidence 문서에 보존한다.

---

## 현재 한눈에 보기

| 구간 | 상태 |
|---|---|
| M0 — First World | **IN_PROGRESS** |
| G0–G7 | **PASS / CLOSED / FROZEN** |
| G8-A Measurement Substrate | **V5 OFFICIAL CAPTURE + INDEPENDENT VERIFICATION COMPLETE / USER VISUAL DURABLE DISPOSITION PENDING** |
| G8-B Benchmark Scenario Suite | **CLOSED / FROZEN** — 다섯 official scenario와 Cell Inspector v0 사용자 승인 완료 |
| G8-C Official Performance Matrix | **OFFICIAL CAPTURE COMPLETE / INDEPENDENT VERIFICATION PASS / RECOMMENDATION `PROCEED_TO_G9`** |
| G8 전체 | **IN_PROGRESS** — G8-A same-SHA visual disposition만 별도 pending |
| G9 Playable First World | **PENDING USER PRODUCT BRIEF / NOT STARTED** |
| 최적화 구현 | **DEFERRED / NOT STARTED** |

## 현재 작업선

- Active branch: `feature/m0-g8c-official-matrix`
- Sealed G8-C source: `4653d7c2e09e93f80fb81eeb73458d992c86858f`
- Ballast integration merge: `6b5f0201f882f212f9916521aec689261d97b4a6`
- G8-B closure: `18391e6a9fc8f9bc7b2757f3504366f106c05435`
- Legacy launcher retirement: `8ee1ae238c324c1db1d7e2882af071fec179a8f1`
- Shared `main`: 이 상태로 승격되지 않음

---

## G8-B 최종 승인 상태

| Scenario / Tool | Human disposition | Automatic disposition |
|---|---|---|
| Sand Fall | **USER ACCEPTED** | `PASS` |
| Water Flow | **USER ACCEPTED WITH KNOWN FOLLOW-UP** | `NEEDS_HUMAN_REVIEW` 유지 |
| Fire / Heat | **USER ACCEPTED** | `PASS` 유지 |
| Pressure Burst | **USER ACCEPTED WITH KNOWN FOLLOW-UP** | `NEEDS_HUMAN_REVIEW` 유지 |
| Heavy Mixed World | **USER ACCEPTED WITH KNOWN FOLLOW-UP** | `NEEDS_HUMAN_REVIEW` 유지; 14/14 hard PASS; blocker false |
| Cell Inspector v0 | **USER ACCEPTED WITH KNOWN FOLLOW-UP** | 사용자 UX 기능; 최대 10 Hz / 100 ms hover delay 비차단 |

G8-B는 **CLOSED / FROZEN**이다. 이전 rejected/superseded candidate와 automatic verdict는 소급 변경하지 않는다. Water의 소수 free-surface 재배열, Pressure의 top-seam-only opening·작은 plume·넓은 terminal activity, Heavy의 감소 중인 broad Thermal tail은 G8-C workload에 포함되어 측정됐으며 production-physics defect로 판정되지 않았다.

핵심 evidence:

- [`G8_B_BENCHMARK_SCENARIO_GALLERY_2026-08-17.md`](../evidence/G8_B_BENCHMARK_SCENARIO_GALLERY_2026-08-17.md)
- [`G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md`](../evidence/G8_B_SAND_FALL_EXPERIMENT_HARNESS_V0_2026-08-17.md)
- [`G8_B_WATER_FLOW_HARNESS_CANDIDATE_2026-08-17.md`](../evidence/G8_B_WATER_FLOW_HARNESS_CANDIDATE_2026-08-17.md)
- [`G8_B_FIRE_HEAT_HARNESS_CANDIDATE_2026-08-17.md`](../evidence/G8_B_FIRE_HEAT_HARNESS_CANDIDATE_2026-08-17.md)
- [`G8_B_PRESSURE_BURST_HARNESS_CANDIDATE_2026-08-18.md`](../evidence/G8_B_PRESSURE_BURST_HARNESS_CANDIDATE_2026-08-18.md)
- [`G8_B_HEAVY_MIXED_WORLD_HARNESS_CANDIDATE_2026-08-19.md`](../evidence/G8_B_HEAVY_MIXED_WORLD_HARNESS_CANDIDATE_2026-08-19.md)

---

## G8-C Official Matrix

### Evidence identity

- Matrix ID: `g8c-official-matrix-4653d7c2e09e-64df60ba0d79`
- Source SHA: `4653d7c2e09e93f80fb81eeb73458d992c86858f`
- Benchmark EXE SHA-256: `29131418a091d1657960c8cf1307d533582fa69e140af330b69be530c4394ed5`
- Windows EXE SHA-256: `2c1670bff506cc9793da9e3708cafb28b6485d14bc577abbcb5faa04f897c4e5`
- Receipt SHA-256: `1fbf4599893cc29e99b6033996b42fcdf025aac0b421cb80b95b3e55807455f6`
- Package SHA-256: `92f8b85cc0e34ea6e71a9f6b4fc95b0f70704263a0f798a69a830cce1d40b729`
- Verification result SHA-256: `77c7e1c982296277c451de02c3dca68fa6d7d9a90e9fd5426c4dffa1abd9bb0d`
- Raw hash inventory SHA-256: `8ade901cc359c2cdfb750f01fff35f0fae463046757e6cee4ba44100c0b8c260`
- Independent verifier: **PASS** — 230 matrix fields raw 재계산, mismatch `0`

### 핵심 결과

- 다섯 workload 최소 Mode A P50: **931.602 TPS**
- 60-TPS 최소 headroom: **15.527×**
- 최대 Mode B GPU envelope P95: **1.046784 ms**
- 최소 Mode C simulation rate: **59.898580 TPS**
- Mode C missed deadlines / catch-up / dropped frames: **0 / 0 / 0**
- 최대 Mode C frame P95: **4.2005 ms**
- 최대 Mode D GPU render P95: **0.021280 ms**
- Persistent tracked GPU bytes: **184,576,672 bytes / scenario** (`~0.172 GiB`, RTX 5090 32 GiB의 `~0.537%`)
- 다섯 workload의 최대 grouped P50 subsystem: **Active / Sleep management**

### 판정

**Recommendation: `PROCEED_TO_G9`**

현재 M0의 60-TPS simulation, simulation+render coexistence, rendering, persistent GPU memory에 명백한 blocker가 없다. Active / Sleep이 가장 큰 measured group이지만 현 수치는 G7-C compaction, indirect dispatch, aggressive packing, f16 또는 다른 최적화를 G9보다 먼저 구현할 근거가 아니다.

상세 계약과 전체 표: [`../evidence/G8_C_OFFICIAL_MATRIX_2026-08-19.md`](../evidence/G8_C_OFFICIAL_MATRIX_2026-08-19.md)

---

## 비공식 진단 이력

다음은 official evidence가 아니며 보존된 진단 자료다.

- Lifecycle-failed pilot: `g8c-pilot-8ee1ae238c32-c64090539536`
- Aggregation-failed replacement pilot: `g8c-pilot-8ee1ae238c32-6341f4f59218`
- Passing aggregation-only replay: `g8c-aggregation-replay-20260819T015515996891Z-fc408076b67a`

Replay는 executable/GPU/measurement subprocess를 `0`회 실행한 `non_evidence=true` parser/publication 검증이다. 세 진단 artifact의 정리는 별도 retention 승인 전까지 수행하지 않는다.

---

## 남은 결정과 Blocker

### 기술 blocker

**없음.** G8-C measurement integrity와 independent verification은 완료됐다.

### 사용자 결정 필요

1. **G8-A same-SHA visual durable disposition** — 기존 v5 capture의 별도 사용자 판정을 승인·보류·supersede 중 하나로 명시해야 한다.
2. **G9 Product Brief** — Matrix의 `PROCEED_TO_G9`는 권고이며 G9 구현 자동 승인이 아니다. 첫 playable world의 시작 상태, 편집 도구, Discovery 범위, 저장/Rewind 범위를 사용자와 합의한 뒤 구현한다.
3. **Shared `main` promotion** — 현재 feature line을 `main`으로 승격하는 것은 별도 승인이다.

---

## 다음 행동

1. 사용자와 G9 요구사항을 먼저 확정한다.
2. G9 구현 전 별도 branch/작업 범위와 acceptance contract를 만든다.
3. G8-C official Matrix를 docs/memory 변경 때문에 재실행하지 않는다.
4. 명백한 새 성능 blocker가 생기기 전에는 최적화 작업을 시작하지 않는다.

M0 `ACHIEVED`: **NO** — 실제 sandbox product validation인 G9가 남아 있다.
