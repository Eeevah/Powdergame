from pathlib import Path

p = Path('docs/planning/STATUS.md')
s = p.read_text(encoding='utf-8')

pairs = [
(
"`IN_PROGRESS` — G0 (Runtime) PASS, G1 (World Integrity) PASS, G2 (Local Movement) PASS / CLOSED, G3 (Density / Displacement) PASS / CLOSED, G4 (Thermal / Phase / Combustion) PASS / CLOSED (G4-A thermal baseline TECHNICAL PASS, G4-B phase transition TECHNICAL PASS, G4-C combustion & finite fuel TECHNICAL PASS, Smoke decay lifecycle G4 integration hardening TECHNICAL PASS, G4 Large 4-Panel Thermal Observatory `--thermal-demo` User Validation APPROVED on 2026-08-16). G5 (Pressure Chain) IN_PROGRESS — G5-A Pressure Field TECHNICAL PASS / FROZEN, G5-B Expansion / Confinement → Pressure TECHNICAL PASS / FROZEN, G5-C Rupture / Opening / Vent implementation & validation in progress.",
"`IN_PROGRESS` — G0 (Runtime) PASS, G1 (World Integrity) PASS, G2 (Local Movement) PASS / CLOSED, G3 (Density / Displacement) PASS / CLOSED, G4 (Thermal / Phase / Combustion) PASS / CLOSED (G4-A thermal baseline TECHNICAL PASS, G4-B phase transition TECHNICAL PASS, G4-C combustion & finite fuel TECHNICAL PASS, Smoke decay lifecycle G4 integration hardening TECHNICAL PASS, G4 Large 4-Panel Thermal Observatory `--thermal-demo` User Validation APPROVED on 2026-08-16). G5 (Pressure Chain) TECHNICAL CHAIN FROZEN — G5-A Pressure Field TECHNICAL PASS / FROZEN, G5-B Expansion / Confinement → Pressure TECHNICAL PASS / FROZEN, G5-C Rupture / Opening / Vent TECHNICAL PASS / FROZEN; final visible boiler-chain User Validation pending."
),
(
"**G0 — Runtime: PASS. G1 — World Integrity: PASS. G2 — Local Movement: PASS / CLOSED. G3 — Density / Displacement: PASS / CLOSED. G4 — Thermal / Phase / Combustion: PASS / CLOSED (G4-A thermal baseline TECHNICAL PASS, G4-B phase transition TECHNICAL PASS, G4-C combustion TECHNICAL PASS, Smoke decay G4 integration hardening TECHNICAL PASS, G4 4-Panel Thermal Observatory `--thermal-demo` (320×192) User Validation APPROVED on 2026-08-16). G5 — Pressure Chain: IN_PROGRESS (G5-A Pressure Field TECHNICAL PASS / FROZEN; G5-B Expansion / Confinement → Pressure TECHNICAL PASS / FROZEN; G5-C Rupture / Opening / Vent IN_PROGRESS).**",
"**G0 — Runtime: PASS. G1 — World Integrity: PASS. G2 — Local Movement: PASS / CLOSED. G3 — Density / Displacement: PASS / CLOSED. G4 — Thermal / Phase / Combustion: PASS / CLOSED (G4-A thermal baseline TECHNICAL PASS, G4-B phase transition TECHNICAL PASS, G4-C combustion TECHNICAL PASS, Smoke decay G4 integration hardening TECHNICAL PASS, G4 4-Panel Thermal Observatory `--thermal-demo` (320×192) User Validation APPROVED on 2026-08-16). G5 — Pressure Chain: TECHNICAL CHAIN FROZEN (G5-A Pressure Field TECHNICAL PASS / FROZEN; G5-B Expansion / Confinement → Pressure TECHNICAL PASS / FROZEN; G5-C Rupture / Opening / Vent TECHNICAL PASS / FROZEN); visible boiler-chain User Validation pending before G5 PASS / CLOSED.**"
),
(
"현재 **G5 — Pressure Chain** 진행 중이다. G5-A scalar Pressure propagation과 G5-B Phase expansion / confinement → Pressure generation은 RTX 5090 / DX12 실기 검증으로 TECHNICAL PASS / FROZEN이며, 다음 sub-gate는 **G5-C — Pressure stress → rupture → opening → venting**이다.",
"현재 **G5 — Pressure Chain**은 기술 체인 구현이 완료되었다. G5-A scalar Pressure propagation, G5-B Phase expansion / confinement → Pressure generation, G5-C Pressure stress → rupture → opening → venting이 모두 RTX 5090 / DX12 실기 검증으로 **TECHNICAL PASS / FROZEN**이다. 남은 G5 gate는 특별 explosion 코드 없이 `가열 → Steam → confinement Pressure → weak Wood rupture → opening → ordinary GAS vent`가 화면에서 하나의 자연스러운 사건으로 읽히는지 확인하는 **visible boiler-chain User Validation**이다."
),
(
"1. G0 (Runtime), G1 (World Integrity), G2 (Local Movement), G3 (Density / Displacement), **G4 (Thermal / Phase / Combustion)**: **ALL PASS / CLOSED** (2026-08-16).\n2. 다음 마일스톤 게이트 착수: **G5 — Pressure Chain** (Phase expansion / yield / Pressure / rupture / vent).\n3. M0 First World 마일스톤 전체는 G5~G9 완료 시까지 `IN_PROGRESS` 유지.",
"1. G0 (Runtime), G1 (World Integrity), G2 (Local Movement), G3 (Density / Displacement), **G4 (Thermal / Phase / Combustion)**: **ALL PASS / CLOSED** (2026-08-16).\n2. **G5 — Pressure Chain technical chain: G5-A / G5-B / G5-C ALL TECHNICAL PASS / FROZEN** on RTX 5090 / DX12. Next: visible boiler-chain User Validation; do not mark G5 PASS / CLOSED before user approval.\n3. After G5 User Validation APPROVED, advance to G6. M0 First World remains `IN_PROGRESS` until G5~G9 and final M0 approval are complete."
),
(
"M0 implementation: **IN_PROGRESS** — G0/G1/G2/G3/G4 PASS / CLOSED (G2/G3/G4 User Validation 모두 APPROVED 완료), G5~G9 + 최종 M0 승인 남음",
"M0 implementation: **IN_PROGRESS** — G0/G1/G2/G3/G4 PASS / CLOSED (G2/G3/G4 User Validation APPROVED); G5-A/G5-B/G5-C TECHNICAL PASS / FROZEN, G5 visible boiler-chain User Validation + G6~G9 + final M0 approval remaining"
),
(
"m0_status: IN_PROGRESS (G0 complete, G1 complete, G2 PASS/CLOSED, G3 PASS/CLOSED, G4 PASS/CLOSED incl. User Validation APPROVED 2026-08-16, G5 pending)",
"m0_status: IN_PROGRESS (G0 complete, G1 complete, G2 PASS/CLOSED, G3 PASS/CLOSED, G4 PASS/CLOSED incl. User Validation APPROVED 2026-08-16; G5-A/B/C TECHNICAL PASS/FROZEN on RTX 5090/DX12; G5 visible User Validation pending; G6-G9 pending)"
),
]

for old, new in pairs:
    if old not in s:
        raise SystemExit('missing STATUS anchor: ' + old[:120])
    s = s.replace(old, new, 1)

anchor = "---\n\n### Known Artifacts & Deferred Items"
block = r'''---

### G5 Pressure Chain Technical Evidence — RTX 5090 / DX12 (2026-08-16)

#### G5-A Pressure Field — TECHNICAL PASS / FROZEN
- Scalar spatial `f32` Pressure Field; Liquid/Gas are pressure media, EMPTY/Static/Powder are not.
- 4-neighbor local propagation; no arbitrary time decay; chunk-boundary propagation verified.
- Frozen regression suite after G5-C: **8 passed, 0 failed**.

#### G5-B Expansion / Confinement → Pressure — TECHNICAL PASS / FROZEN
- Water boiling uses Matter yield=2: open space spawns a second Steam; blocked/ownership-lost expansion becomes confinement Pressure (`100.0`).
- Deterministic ownership and 64-cell chunk-boundary expansion verified.
- Frozen regression suite after G5-C: **5 passed, 0 failed**.

#### G5-C Pressure Stress → Rupture → Opening → Vent — TECHNICAL PASS / FROZEN
```text
Tested implementation SHA: 5187d9980f9067cced1edb0b6a8f79ab56147a0c
Validation worktree: C:\Users\mdkap\source\repos\Powdergame-g5c-validation
Adapter: NVIDIA GeForce RTX 5090
Vendor: 0x10DE
Backend: wgpu::Backend::Dx12
WGSL parse: 1 passed, 0 failed (rupture.wgsl included)
G5-C rupture: 5 passed, 0 failed
G5-B expansion regression: 5 passed, 0 failed
G5-A pressure regression: 8 passed, 0 failed
G4-B phase regression: 16 passed, 0 failed
Full GPU integration: 143 passed, 0 failed, 1 ignored (controlled_reference_world_perf)
Core: 130 passed, 0 failed
Workspace all-target check: 0 errors, 0 warnings
Git diff check: clean
Validation worktree: clean
Original user MATERIAL_CANDIDATES.md: preserved untouched
```

G5-C structural baseline: Wood rupture threshold `80.0`; Stone/Boundary are unbreakable M0 reference walls. A fully blocked Water→Steam expansion generates `100.0` Pressure. The causal-chain test verified: Tick 1 `boil → blocked expansion → Pressure → Wood rupture → EMPTY opening`, Tick 2 ordinary GAS movement vents Steam through that opening and vacated spatial Pressure returns to reference. No boiler-specific/radial explosion code is used.

**Gate state:** G5-A/B/C technical chain is frozen. **G5 itself is not PASS / CLOSED yet**; final visible boiler-chain User Validation is required. Detailed G5-C evidence: `docs/planning/G5_C_RUPTURE_VENT.md`.

---

### Known Artifacts & Deferred Items'''

if anchor not in s:
    raise SystemExit('missing Known Artifacts anchor')
s = s.replace(anchor, block, 1)
p.write_text(s, encoding='utf-8', newline='\n')
print('STATUS updated for frozen G5 technical chain')
