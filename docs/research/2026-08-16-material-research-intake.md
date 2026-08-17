# 2026-08-16 Material Research Intake

## Status

**Type:** Research intake / non-authoritative  
**Adoption state:** `REFERENCE + CANDIDATE`  
**Authoritative implementation contract:** existing ADR / SPEC / MILESTONES

이 문서는 2026-08-16에 전달받은 세 개의 대형 재료 연구 결과를 Powdergame 저장소 안에서 추적하기 위한 intake 기록이다.

목적은 조사 결과를 버리지 않되, 조사 보고서의 숫자·설계 제안·IP 해석을 현재 구현 계약과 섞지 않는 것이다.

---

## 1. Source Manifest

### Source A — Material & Phenomenon Encyclopedia

**Original upload name:** `붙여넣은 마크다운(1)(1).md`  
**Title:** `Powdergame Material & Phenomenon Encyclopedia 심층 연구 보고서`  
**Original size:** 95,857 bytes  
**SHA-256:** `5e270324753a0224e88a24e522fffdc76a2951dafd6d0f1208e49dd8d60e23b0`

주요 범위:

- M0/Tier B Material archetype resolution
- 현실 물성 reference
- density rank / thermal class / pressure resistance 제안
- 상변화 및 combustion/pressure 현상 사전
- Material 후보와 machine-readable schema 제안

가장 직접적인 활용 영역:

- 현실 물성 reference layer
- Material archetype 선택
- 게임값 normalization 후보
- M0 이후 Material content authoring

### Source B — General Game Material Encyclopedia

**Original upload name:** `게임용 종합 재료 백과사전: 실재 물질·기존 창작물·오리지널 판타지/SF 소재`  
**Original container format:** ChatGPT research widget JSON  
**Original container size:** 1,451,602 bytes  
**Original container SHA-256:** `65734b120fcac059355e2b2dac0eed06b00de2db34df3b19eb986a87193e6c58`

위 JSON에서 최종 보고서 본문을 별도로 추출할 수 있다.

**Extracted report size:** 52,435 bytes  
**Extracted report SHA-256:** `c99ff484418b38dbf601e6bb1e46d1e522863b5c89264fbee173ef399e00d177`

주요 범위:

- 현실 금속 / 광물 / 화학 / 고분자 / 핵재료
- 기존 창작물 소재
- 오리지널 판타지/SF 소재
- 채집 → 정제 → 제작 → 재활용 경제 모델
- 재료 속성을 하나의 내구 수치가 아니라 기능 축으로 분리하는 아이디어

가장 직접적인 활용 영역:

- 장기 Material catalog 확장
- 현실 특이 소재 후보
- 기능별 재료 차별화 아이디어
- 향후 제작/경제 시스템을 실제로 채택할 경우 reference

현재 Powdergame은 이 보고서가 가정한 crafting/equipment game과 동일하지 않으므로 경제/제작 티어는 자동 채택하지 않는다.

### Source C — Fictional Matter Dynamics

**Original upload name:** `붙여넣은 마크다운(2).md`  
**Title:** `가상 물질 동역학 및 창작 소재 설계를 위한 분자 오토마타 아키텍처 연구 보고서`  
**Original size:** 136,447 bytes  
**SHA-256:** `91a2ec51b57af0551ee991efb9e858719bdb50a5e4ebbfd317b98758aedda980`

주요 범위:

- 기존 게임/SF/판타지/신화 소재 100종의 mechanic 추출
- 10개 Behavior Grammar taxonomy
- 실제 자연의 이상 물성 abstraction
- Original Matter 100종
- Interaction Yield / Emergence / Runtime Cheapness 평가

가장 직접적인 활용 영역:

- Powdergame 고유의 fictional Matter 설계
- 유명 IP의 이름이 아니라 행동 문법을 추출하는 reference
- 미래 Field/System 후보의 가치 평가
- M0-compatible content 후보 탐색

---

## 2. Current Powdergame Contract

연구자료를 해석할 때 현재 승인된 핵심 계약은 다음과 같다.

```text
M0 — First World

Matter
- Boundary Block
- Stone
- Sand
- Ice
- Water
- Steam
- Smoke
- Wood
- Oil

Field / Phenomenon
- Temperature
- Combustion / Fire
- Pressure

Movement
- STATIC
- POWDER
- LIQUID
- GAS

Core
- One Cell = Max One Matter
- EMPTY is not Matter
- Density Rank = local displacement order
- Read Neighbors / Write Self
- spatial ownership only → Claim/Resolve
- Minimum Sufficient Physics
- approximate / stable behavior, not scientific exactness
```

연구 결과는 이 계약을 바꾸는 권한을 갖지 않는다.

---

## 3. High-value Findings

세 연구 결과에서 공통적으로 가치가 높은 부분은 다음과 같다.

### A. Reality → Archetype → Game Value를 분리한다

`Stone`, `Oil`, `Wood`, `Smoke`, `Metal` 등은 현실에서 하나의 고정 물질이 아니다.

따라서 향후 Material data는 가능하면 다음 세 층을 구분한다.

```text
REAL REFERENCE
→ GAME ARCHETYPE
→ POWDERGAME VALUE
```

예:

```text
real basalt density / thermal data
→ Stone의 대표 archetype으로 basalt 채택 여부 검토
→ density_rank / thermal class / transition threshold는 게임값으로 별도 결정
```

이는 현재 `MATERIAL_SPEC.md`의 `Material is not a real-world material DB` 원칙과 잘 맞는다.

### B. Mechanic-first fictional Matter

가상 Matter는 이름이나 lore보다 다음을 먼저 정의하는 편이 가치가 높다.

```text
Input
→ cheap local rule
→ state / field change
→ secondary system
→ emergent chain
```

예:

```text
Pressure absorb
→ Heat accumulation
→ nearby Water heats
→ Steam
→ Pressure
→ rupture
```

이 구조는 Powdergame의 핵심인 `작은 규칙 → 큰 사건`과 직접 연결된다.

### C. Interaction Yield

좋은 Material은 스탯이 높은 Material이 아니라 적은 구현 비용으로 여러 기존 세계 법칙에 연결되는 Material이다.

연구에서 사용한 다음 관점은 계속 유지할 가치가 있다.

```text
Interaction Yield
≈ useful interactions / implementation + runtime cost
```

정확한 공식 점수로 사용할 필요는 없으나 콘텐츠 우선순위 판단 기준으로 유용하다.

### D. 현실의 이상 물성은 매우 좋은 아이디어 원천이다

특히 다음 범주는 Powdergame형 local rule로 변환 가치가 높다.

- phase-change materials
- non-Newtonian fluid
- shape-memory materials
- aerogel
- hydrophobic powder
- gallium-like low-temperature melting / embrittlement
- piezoelectric materials
- ferrofluid
- sublimating solids
- exothermic hydration
- crystallization / self-propagating patterns

현실을 정확히 복제하는 것이 아니라 **자연이 이미 발명한 이상한 규칙을 게임 문법으로 재해석**하는 것이 목적이다.

---

## 4. Do Not Adopt Directly

### 4.1 Exact proposed numeric values

다음은 모두 후보이며 현재 계약이 아니다.

- provisional `density_rank`
- exact ignition temperature
- exact melting/boiling threshold
- thermal class boundary
- pressure resistance rank
- smoke lifetime
- rupture threshold
- spawn count / expansion ratio

실제 게임값은 Production Simulation과 play/benchmark 결과를 보고 조정한다.

### 4.2 EMPTY thermal medium proposal

Source A는 EMPTY 영역에 배경 온도 감쇠를 두는 방안을 제안한다.

현재 M0 설계는:

> `EMPTY`는 숨은 Air/Thermal Matter가 아니다.

를 명시하고 있다.

따라서 이 제안은 **현재 계약과 충돌하는 research idea**다. 필요성이 실제 구현에서 증명되기 전에는 채택하지 않는다.

### 4.3 Per-cell progress/state explosion

Research Material 중 일부는 다음과 같은 상태를 암묵적으로 요구한다.

- age
- stored energy
- previous pressure
- resonance history
- infection progress
- memory
- local time scale
- probability bias

현재 `MATERIAL_SPEC`은 미래 가능성을 이유로 universal per-cell progress state를 미리 넣지 않는 방향이다.

따라서 후보마다:

```text
기존 Temperature / Pressure / Matter ID / small flags로 표현 가능?
```

을 먼저 확인한다.

### 4.4 IP names and lore

Vibranium, Adamantium, Beskar, Kyber, Dilithium, Redstone 등은 **reference material**이다.

실제 Powdergame 콘텐츠에서는 기본적으로:

```text
original fictional name/lore
X

abstract mechanic
→ transformed original Powdergame Matter
```

방식을 사용한다.

Research 문서의 `PUBLIC_DOMAIN_CANDIDATE` 표기도 법적 판정을 의미하지 않는다. 실제 사용 전 별도 권리 검토가 필요하다.

### 4.5 Crafting/economy assumptions

Source B는 채집·정제·장비·상점가·티어 중심의 게임을 가정한다.

현재 Powdergame의 최상위 비전은 우선 **세계 창조 / 실험 / 발견 sandbox**다.

따라서 금속 티어, 가격, 장비 내구, 공급망, 제작설비 등은 장기 reference로만 보존한다.

---

## 5. Candidate Buckets

연구를 실제 콘텐츠로 바꿀 때 다음 버킷으로 분류한다.

### Bucket A — M0 Reference

현재 M0 Matter의 archetype 및 물성 참고.

```text
Stone
Sand
Ice
Water
Steam
Smoke
Wood
Oil
```

첫 적용 우선순위가 가장 높다.

### Bucket B — M0-system-compatible Future Matter

새 대형 subsystem 없이 Temperature / Pressure / Movement / Phase / Combustion / local interaction으로 표현 가능한 후보.

연구자료에서 특히 검토 가치가 있는 예:

- phase-change thermal buffer
- endothermic cooling powder
- pressure-expanding mineral
- temperature-seeking gas
- pressure/resonance-fragile crystal
- sub-zero liquid coolant
- heat-expanding powder/foam
- dry-ice-like sublimating solid
- quicklime-like water-reactive powder
- hydrophobic powder

이 목록은 **등록 결정이 아니라 후보군**이다.

### Bucket C — Near-term Chemistry / Material Diversity

현재 catalog 방향과 연결하기 쉬운 후보.

- Acid
- Salt / Saltwater decision
- Glass
- Lava
- generic Metal / later metal families
- Dirt / Mud
- Ash / Char
- reactive mineral / corrosion products

### Bucket D — Future System Drivers

새 subsystem을 정당화할 정도로 여러 콘텐츠를 열 수 있을 때만 검토한다.

```text
Electricity
→ conductor / resistor / piezo / electrolysis / logic materials

Light
→ transparent / absorb / emit / reflect / photosensitive materials

Radiation
→ source / absorber / mutation / shielding

Magnetism
→ magnetic powder / ferrofluid / field-guided movement

Biology
→ nutrient / growth / infection / symbiosis

Information
→ signal / memory / delay / logic

Space / Time / Probability
→ 매우 장기적인 exotic layer
```

Matter 하나를 위해 subsystem 하나를 만들지 않는다.

---

## 6. Strong Transformation Direction

향후 이 세 research source를 그대로 Material Registry로 옮기지 않는다.

다음 네 개의 **derived artifact**로 다시 만드는 것이 권장된다.

### 1. `REAL_MATERIAL_REFERENCE`

현실 물성 및 archetype reference.

예:

```text
water
real reference
- density
- heat capacity
- thermal conductivity
- phase reference

No game rank here.
```

### 2. `MECHANIC_LIBRARY`

기존 창작물/자연현상에서 추출한 이름 없는 행동 문법.

예:

```text
KINETIC_TO_HEAT
PRESSURE_TRIGGERED_EXPANSION
HEAT_SEEKING_GAS
SUBSTRATE_CONVERSION
DELAYED_RELEASE
SELECTIVE_CORROSION
PHASE_LOCK
```

IP 이름은 provenance에만 남긴다.

### 3. `ORIGINAL_MATTER_CANDIDATES`

Powdergame 고유 Material 후보.

각 후보는 최소 다음을 가진다.

```text
identity
movement_class
existing systems used
new state required
local rules
emergent chains
interaction yield
implementation cost
runtime cost
milestone bucket
provenance / inspirations
```

### 4. `M0_MATERIAL_TUNING`

M0 실제 구현이 시작된 뒤 benchmark/play evidence를 바탕으로만 만드는 게임값 문서.

```text
density_rank
thermal coefficients/classes
transition thresholds
combustion rates
pressure/rupture values
```

Research 수치를 초기 seed로 사용할 수는 있지만, 최종값은 실제 world behavior가 결정한다.

---

## 7. Recommended Next Research Conversion

구현 전에 대규모 콘텐츠를 확정하지 않는다.

가장 가치가 높은 다음 변환 작업은:

1. Source A에서 **M0 8종의 현실 reference만 추출**
2. 연구 보고서가 제안한 게임값을 `provisional`로 별도 분리
3. Source C의 100개 mechanic을 **IP 이름 없는 mechanic taxonomy**로 재정리
4. 그 taxonomy에서 현재 M0 시스템만 쓰는 Original Matter 후보를 소수 선정
5. M0 구현/benchmark 이후 실제 추가 Material을 결정

즉 지금의 research는 콘텐츠를 바로 확정하는 문서가 아니라 앞으로 콘텐츠를 빠르고 일관되게 만드는 **재료 풀 + 행동 문법 풀**로 사용한다.

---

## 8. Source Preservation Note

이번 intake에서는 세 원문의 파일명, 크기와 SHA-256을 보존했다.

현재 사용 중인 GitHub connector의 repository write action은 UTF-8 `content`를 받는 방식이며 로컬 첨부 파일 경로 자체를 repository file로 전달하는 file parameter가 없다. 따라서 대형 원문 전체를 이 intake 커밋에서 byte-for-byte 복제하지 않고, 우선 추적 가능한 manifest와 설계 triage를 저장했다.

원문 내용을 실제 authoring data로 변환할 때는 위 SHA-256과 제목을 기준으로 source를 확인하고, derived artifact에는 어느 source/section에서 왔는지 provenance를 남긴다.

원문 자체가 authoritative contract가 아니라는 점은 이 제한과 무관하게 유지된다.
