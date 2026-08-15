# 2026-08-16 MATERIAL_CANDIDATES Analysis

## Status

- Type: derived research analysis / non-authoritative
- Source: `docs/research/raw/MATERIAL_CANDIDATES.md`
- Source size: 33,259 bytes
- Source SHA-256: `792c3797fdea62333533ee8b21ce35b08ffc4a5018786b2b35d94cd008b94649`
- Adoption state: `REFERENCE + CANDIDATE`
- Authority: current Vision / ADR / SPEC / MILESTONES remain authoritative

이 문서는 전달받은 `MATERIAL_CANDIDATES.md`를 현재 Powdergame 계약에 맞춰 해석한 분석 메모다. 원문을 수정하거나 후보를 자동 채택하지 않는다.

---

## 1. What this source is good at

이 노트의 가장 좋은 기준은 마지막에 적힌 다음 질문이다.

> 이 물질이 옆에 있으면, 플레이어가 3초 안에 무슨 일이 벌어질지 상상되는가?

이 기준은 현재 Powdergame의 `World Consistency over Scientific Correctness`, `Minimum Sufficient Physics`, `small local rule -> emergent chain` 철학과 잘 맞는다.

또한 원문은 다음 원칙을 스스로 지키려 한다.

- 이미 있는 Matter를 중복 생성하지 않음
- Fire를 Matter가 아니라 combustion phenomenon으로 취급
- 우주/SF 소재도 새 범용 물리축을 무조건 추가하지 않고 기존 Temperature / Density / Combustion / Pressure에 붙이려 함
- 팔레트 스왑이나 장식뿐인 후보는 스스로 `스킵` 또는 통합 대상으로 표시
- 유사 역할은 하나만 남기려 함

따라서 이 자료는 완성된 Material Registry보다 **human-readable candidate quarry**로 가치가 높다.

---

## 2. Inventory snapshot

Markdown의 굵은 항목 기준으로 약 230개 후보/메모 항목이 있다.

대략적인 섹션 분포:

- 흙 / 가루: 9
- 액체: 10
- 기체: 6
- 단단한 것: 7
- 생명: 7
- 우주 / 과학: 15
- 창작물 reference: 140
- 판타지 / 연금술: 36

정확히 중복 표기된 이름도 있다.

- Mithril
- Ichor
- Soul Sand
- Sculk

그 외에도 원문 자체가 `통합`, `스킵`, `겹침`이라고 적은 항목이 많으므로 실제 독립 mechanics 수는 230보다 상당히 작다.

---

## 3. Current catalog mismatch

원문의 첫 문장은 `게임에 아직 없는 물질 후보`라고 되어 있지만, 현재 `MATERIAL_SPEC.md` 기준으로 다음은 이미 초기 catalog 방향에 들어 있다.

- Acid
- Seed
- Plant
- Salt
- Lava
- Metal
- Glass

이들은 `새 후보`가 아니라 **REGISTERED direction / not necessarily M0 VALIDATED**로 해석해야 한다.

현재 M0 validated 최소 세트는 다음 9종이다.

- Boundary Block
- Stone
- Sand
- Ice
- Water
- Steam
- Smoke
- Wood
- Oil

따라서 앞으로 candidate tooling에서는 `not in M0`와 `not registered`를 구분해야 한다.

추천 상태:

```text
KNOWN_M0
REGISTERED_FUTURE
NEW_CANDIDATE
REFERENCE_ONLY
REJECT_OR_MERGE
```

---

## 4. Strong near-term candidates

다음 후보는 새로운 범용 Field 없이도 현재/근접한 Movement + Temperature + Pressure + Combustion + Phase Transition + small local rules로 강한 정체성을 만들 수 있다.

### Tier A — especially strong

#### Gunpowder

```text
POWDER
+ ignition / sufficient heat
-> fast combustion
-> Heat + Pressure + Smoke
```

장점:

- M0 세계 문법을 거의 그대로 재사용
- 밀폐/개방에 따라 결과가 자연스럽게 달라짐
- Pressure gate를 플레이어가 즉시 이해하게 함
- 별도 폭발 시스템보다 combustion + pressure chain으로 구현할 여지가 큼

대표적인 high Interaction Yield 후보.

#### Dry Ice

```text
STATIC
+ heat
-> CO2 GAS
```

장점:

- Ice와 다른 `solid -> gas` 문법을 보여줌
- CO2를 무거운 gas + combustion suppressor로 만들면 density/combustion을 함께 활용
- 승화라는 현상이 매우 읽기 쉬움

#### Clathrate

```text
cold STATIC
+ heat / pressure release
-> gas release
-> Methane + Pressure / combustion hazard
```

장점:

- Ice처럼 보여도 위험한 내부 가스를 가진다는 발견성이 강함
- Temperature -> Gas -> Pressure -> Combustion chain이 가능
- 우주/얼음 위성 theme와도 강하게 연결

실제 구현에서는 한 셀이 여러 Matter를 동시에 보유하지 않으므로 decomposition 결과는 self transition + spawn request 형태로 설계해야 한다.

#### CO2

```text
heavy GAS
+ combusting neighbor
-> combustion suppression / heat reduction candidate
```

Oxygen을 M0의 필수 연소 자원으로 만들지 않아도 독립적인 소화 가스로 사용할 수 있다.

#### Cryofluid

```text
LIQUID
+ hot neighbor
-> RemoveHeat
-> neighbor's normal transition rules do the rest
```

이 후보의 좋은 점은 `Cryofluid turns Lava into Stone` 같은 bespoke pair code가 아니라 **열을 빼앗는 일반 성격**으로 설계할 수 있다는 것이다.

#### Ablative Char

```text
STATIC
+ high heat
-> self degradation / destruction
-> absorbs or removes heat
```

강한 방어재가 아니라 `자기를 희생해 뒤를 지키는 Matter`라서 interaction grammar가 명확하다.

### Tier B — good with a little more content support

- Snow — POWDER -> Water thermal transition. 압축 눈은 별도 상태 없이 후속 Matter로 처리 가능.
- Brine — Water와 다른 freeze threshold를 가진 LIQUID. Salinity per-cell float보다 별도 Matter가 현재 철학에 더 잘 맞음.
- Clay / Brick — wetting + heating 생산 chain. 단, universal wetness state는 피하고 discrete Matter transition을 선호.
- Obsidian — Lava quench 결과 후보. Glass/Stone과 충분히 다른 rupture/thermal 성격이 있어야 독립 Material 가치가 생김.
- Methane — GAS + combustion + pressure. Oil과 다른 공간 점유/축적 위험이 명확함.
- Tar — slow LIQUID + long combustion + Smoke. viscosity를 별도 f32로 넣기보다 movement mobility class/rank로 표현할 수 있는지 검토.
- Regolith — 단순 `외계 Sand`만으로는 약하지만 heat reaction 또는 extraction chain이 있으면 가치가 생김.
- Perchlorate Dust — 강한 heat/combustion amplifier로 재해석 가능. explicit Oxygen simulation을 요구하지 않도록 설계할 수 있음.

---

## 5. Candidates that should stay future-system drivers

다음 후보는 좋은 아이디어지만 지금 구현하면 M0 범위를 크게 흔든다.

### Biology / Agent dependence

- Blood
- Poison Gas의 생체 피해
- Spore Cloud
- Vine / Moss / Mushroom / Algae
- Flood Biomass
- Kharaa Pustule
- Necro Tissue
- ADAM / Medigel 등

Seed/Plant의 최소 growth grammar와 일반 Biology system은 구분해야 한다.

### Electricity / Information

- Eezo Dust의 electric trigger
- Redstone Dust
- Logic-like Sculk variants
- Ancient Nano / repair materials if machine state is required

### Light / Radiation

- Kyber-style light output
- Glowstone / Luminite
- Kryptonite / Radon / Chiral Crystal
- radioactive Artifact variants

### Space / Gravity / Observer rules

- Ghost Matter
- Quantum Shard
- Xen Crystal
- Unobtanium gravity effect
- Void Fluid
- Red Matter
- Ender Pearl / Chorus warp family

특히 `Quantum Shard`의 `보고 있을 때/안 볼 때`는 rendering visibility 또는 player observation을 simulation truth에 연결하게 되므로 현재 GPU-authoritative local simulation과 궁합이 매우 나쁘다. 좋은 fiction reference이지만 낮은 우선순위다.

### Time / history restoration

- Timeshift Glow
- old-state restoration variants

이런 Matter는 prior state/history storage를 요구할 가능성이 높으므로 매우 늦게 다룬다.

---

## 6. Not really Matter / architecture conflicts

### Null Atmosphere

원문도 스스로 `진공이 물질이 될 수는 없다`고 지적한다.

현재 One Cell = max one Matter 모델에서는 `vacuum`을 가짜 GAS Matter로 채우는 것보다 EMPTY / environment semantics로 남기는 편이 맞다.

### Stargate Naquadah Gate

원문 판단대로 structure/object이지 Matter 자체가 아니다.

### Indoctrination Hum

현상/효과이지 Matter가 아니다.

### Pure presentation residues

`Plasmid Glow`, 일부 paint/scoring/visual-only 후보는 Simulation Matter보다 presentation effect / decal / visual state가 더 적합할 수 있다.

Material Registry를 시각 효과 catalog로 만들지 않는다.

---

## 7. Duplicate mechanic families

### Dissolver family

- Acid
- Xeno Blood
- Thresher Maw Acid
- Alkahest

추천:

- Acid = baseline corrosive liquid
- 한 개의 exotic selective/extreme dissolver만 추가
- 나머지는 reference mechanic으로 병합

### Flammable liquid family

- Oil
- Hydrocarbon Lake
- Alcohol
- Tar
- Napalm
- Promethium
- Ichthyic Oil
- Nuka Coolant

각각 독립 Material이 되려면 최소 한 가지 **다른 world grammar**가 필요하다.

예:

- Oil: light liquid + ordinary combustion
- Tar: very low mobility + long smoke-rich combustion
- Alcohol: mixes/dilutes with Water가 실제로 구현될 때만 가치 증가
- Napalm-like: adhesion이라는 독립 movement grammar가 있을 때만

### Flammable gas family

- Methane
- Vespene
- Tibanna

기본 mechanics는 하나로 충분하다. franchise names는 reference-only로 두고 Powdergame 고유 gas를 설계하는 편이 낫다.

### Extreme structural material family

- Trinium
- Beskar
- Phrik
- Adamantium
- Adamantine
- Forerunner Alloy
- Ceramite
- Mithril
- Vibranium

단순히 `더 안 깨짐`으로 여러 개 만들면 행동 다양성이 없다.

독립화하려면 예를 들어:

```text
Ablative -> 자기 희생으로 Heat 차단
Resonant -> 특정 Pressure pattern에만 취약
Thermal conductor -> Heat를 매우 빨리 분산
Thermal isolator -> Heat를 차단
Pressure absorber -> Pressure를 Heat로 변환
```

처럼 **강도 이외의 동사**가 필요하다.

### Infection / self-propagation family

- Protomolecule Goo
- Tiberium
- Flood Biomass
- Creep
- Necro Tissue
- Reaper Nanite
- Sculk
- Infection Orange

이쪽은 특히 하나의 generic self-propagation grammar를 여러 Matter가 공유할 가능성이 높다. 다만 Biology/Information 범위를 열기 전에는 보류.

### Annihilation / anomaly family

- Void Fluid
- Red Matter
- Ghost Matter

`무엇이든 지운다`는 효과는 쉽게 다른 Matter의 의미를 파괴한다. 적어도 target restriction, capacity, byproduct 또는 recoverable counter가 필요하다.

---

## 8. IP / provenance handling

이 파일은 유명 작품의 고유 명칭을 의도적으로 많이 사용한다.

현재 research archive 원칙에 따라 이런 이름은 **REFERENCE_ONLY**로 취급한다.

실제 Powdergame content 후보로 승격할 때는:

```text
source expression / franchise name
-> extract behavior grammar
-> compare with real/natural mechanic
-> combine with Powdergame systems
-> original identity / name / visual language
```

순으로 변환하는 것이 기본이다.

예:

```text
Vespene / Tibanna
-> buoyant or stratifying combustible gas
-> pressure-sensitive fuel gas
-> original Powdergame Matter
```

원전 고유명과 lore를 그대로 catalog에 넣는 것과, 그 행동 원리를 연구재료로 쓰는 것은 분리한다.

---

## 9. Recommended first derived shortlist

현재 engine grammar에 맞춰 이 파일에서 먼저 정식 candidate sheet로 승격할 가치가 높은 묶음:

```text
Gunpowder
Dry Ice
CO2
Clathrate
Methane
Cryofluid
Ablative Char
Snow
Brine
Clay
Brick
Obsidian
Tar
Regolith
Perchlorate Dust
```

그리고 이미 초기 catalog에 있는 다음 항목은 별도 `new candidate`가 아니라 기존 catalog 구체화 대상으로 연결한다.

```text
Acid
Salt
Lava
Metal
Glass
Seed
Plant
```

이 15 + 7을 먼저 다루면 현실/우주 테마와 현재 M0 physics 사이의 간격을 가장 적은 새 시스템으로 연결할 수 있다.

---

## 10. Best next transformation

이 원문을 그대로 Material Registry로 변환하지 않는다.

다음 단계는 후보마다 한 줄의 lore를 늘리는 것이 아니라 아래 compact sheet로 압축하는 것이다.

```text
candidate_name
provenance
status
movement_class
core_verb
input
self_change
field_effect
spawn_request
counter
byproduct
required_systems
interaction_yield
merge_family
```

특히 `core_verb`와 `merge_family`가 중요하다.

예:

```text
Gunpowder
core_verb: DETONATE_FROM_HEAT
input: Temperature / combusting neighbor
output: Heat + Pressure + Smoke
required_systems: existing M0 grammar
merge_family: explosive_powder
```

```text
Tibanna
core_verb: BURN_AS_STORED_GAS
merge_family: combustible_gas
provenance: FICTION_IP / REFERENCE_ONLY
```

이렇게 하면 230개의 이름이 아니라 **실제로 서로 다른 몇 개의 행동 문법이 있는지** 볼 수 있다.

---

## Conclusion

이 파일은 바로 구현할 Material 목록이라기보다, 현재까지 받은 research 중 **플레이어 관점의 직관성과 테마를 가장 잘 보여주는 후보 채석장**이다.

가장 강한 부분은 현실/우주/창작물을 섞으면서도 `옆에 두면 무슨 일이 생기나`를 계속 묻는 점이다.

가장 큰 위험은:

- 등록 상태와 M0 상태 혼동
- franchise-specific names의 직접 채택
- 같은 mechanic의 palette variant 증가
- future Field를 한 후보 때문에 성급히 추가
- annihilation / invulnerability 같은 absolute effect 남발

이다.

따라서 이 자료는 앞으로 `mechanic family -> original Powdergame candidate -> SPEC review -> registration` 파이프라인의 입력으로 사용한다.
