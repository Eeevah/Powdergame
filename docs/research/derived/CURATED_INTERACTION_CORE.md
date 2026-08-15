# Powdergame Curated Interaction Core

## Status

- Type: `DERIVED`
- Authority: non-authoritative content shortlist
- Based on `MATERIAL_SELECTION_FRAMEWORK.md`
- Goal: produce a compact, interaction-heavy vocabulary instead of carrying hundreds of encyclopedia entries into implementation.
- M0 scope remains unchanged.

> **하나의 Material이 하나의 이름이 아니라 여러 실험의 시작점이어야 한다.**

---

## 1. Tier A — Current world grammar

현재 세계의 기본 문법을 증명하는 Matter는 그대로 유지한다.

```text
Boundary Block
Stone
Sand
Ice
Water
Steam
Smoke
Wood
Oil
```

Fire remains a combustion phenomenon/state.

이미 알려진 initial catalog direction:

```text
Acid
Seed
Plant
Salt
Lava
Metal
Glass
```

이 16종이 첫 기반이다.

---

## 2. Tier B — Real materials with very high interaction value

여기서는 `유명해서`가 아니라, 기존 Matter와 만나 여러 chain을 만들기 때문에 고른다.

### Dirt

**왜 남기는가:** 물·식물·열을 잇는 가장 기본적인 토양 Matter.

```text
Dirt + Water → Mud-like state/material
Dirt + Seed + Water → growth substrate
Drying → loose Dirt
```

### Clay

**왜 남기는가:** 자연물에서 제조재로 넘어가는 가장 직관적인 다리.

```text
Clay + Water → workable wet clay
Wet Clay + Heat → Brick
```

### Brick

**왜 남기는가:** 반응의 결과가 구조물 재료가 되는 첫 명확한 보상.

### Coal / Charcoal

**왜 남기는가:** 연료이면서 Gunpowder/제련/재(Ash) chain의 허브.

Charcoal과 Coal이 gameplay상 너무 비슷하면 하나를 variant/alias로 줄일 수 있다.

### Sulfur

**왜 남기는가:** 현실 물질, 강한 냄새/연소 이미지, Gunpowder chain, 연금술 상징을 동시에 가짐.

### Saltpeter / Nitrate

**왜 남기는가:** Gunpowder를 단순 `Coal + Sulfur = Explosion`이 아닌 생산 chain으로 만들어준다.

### Gunpowder

**왜 남기는가:** Powder movement, Heat, Combustion, Pressure, Smoke를 한 번에 연결하는 최고 수준의 interaction hub.

```text
Powder + ignition
→ rapid combustion
→ Heat + Pressure
→ Smoke / residue
```

### Alcohol

**왜 남기는가:** Water와 섞이는 액체인데 쉽게 타는 성격 때문에 Oil과 다른 실험을 만든다.

### Methane

**왜 남기는가:** 눈에 보이지 않는 공간/밀폐/환기가 gameplay가 되게 한다.

```text
Methane GAS
→ confined accumulation
→ ignition
→ Heat + Pressure
```

### Dry Ice

**왜 남기는가:** `고체 → 액체`라는 상식에서 벗어나 승화를 직관적으로 보여준다.

```text
Dry Ice + Heat → CO2 GAS
```

### CO2

**왜 남기는가:** 무거운 GAS + combustion suppression이라는 독립적인 역할.

### Oxygen

**왜 남기는가:** Fire 자체가 아니라 **다른 연소를 강화하는 물질**이라는 명확한 역할.

주의: Oxygen을 넣는다고 전체 atmosphere composition simulation을 자동 도입하지 않는다.

### Brine

**왜 남기는가:** Salt / Water / Ice / Metal을 연결하는 매우 높은 interaction yield.

```text
Salt + Water → Brine
Brine → altered freezing behavior
Brine + Metal → corrosion candidate
```

### Mercury

**왜 남기는가:** `액체 금속`이라는 한 문장만으로 정체성이 강하고, 무거운 액체/금속 반응/연금술 reference까지 연결한다.

### Rust

**왜 남기는가:** 시간이 남기는 세계 흔적. Metal의 변화가 영구적으로 보이게 한다.

### Limestone

**왜 남기는가:** Stone 계열에서 Cement/Concrete 제조로 넘어가는 원료 역할.

### Cement

**왜 남기는가:** `powder + water → curing`이라는 새로운 제조 동사를 만든다.

### Concrete

**왜 남기는가:** 유동/혼합 상태가 시간이 지나 구조물이 되는 강한 world-fabrication 결과.

### Obsidian

**왜 남기는가:** Lava + rapid cooling의 시각적으로 강한 결과물.

### Resin

**왜 남기는가:** Liquid → cured solid, combustion, trapping, Amber-like 결과를 하나의 chain에 연결할 수 있다.

---

## 3. Tier C — Real but exotic / space-real candidates

현실 기반이지만 평범한 소재보다 조금 늦게 공개하면 발견감이 커지는 후보.

### Methane Clathrate

**도감 정체성:** 얼음처럼 보이지만 불을 품은 고체.

```text
Heat
→ Methane release
→ confinement
→ ignition
→ Pressure accident
```

가장 강한 우주/심해 현실 후보 중 하나.

### Regolith

외계 Dirt/Sand 역할. 단순 palette swap이 되지 않도록 glassing, abrasive behavior 또는 embedded volatile 같은 차이를 가져야 한다.

### Perchlorate Dust

평범한 먼지처럼 보이다 충분히 가열하면 oxidizing behavior를 보이는 화성계 후보.

### Ammonia Ice

Water Ice와 다른 냄새/가스/상전이를 가진 outer-system ice family.

### Hydrocarbon Lake

물처럼 보이지만 Water와 섞이지 않고 잘 타는 `false water`.

### Aerogel

매우 가볍고 뛰어난 단열, 대신 취성/압축 실패라는 명확한 tradeoff.

---

## 4. Historical worldview layer

이 계층은 전부 Matter ID가 아니다.

### Classical elements

```text
Earth
Water
Air
Fire
Aether
```

권장 사용:

- Discovery category
- world era / philosophy collection
- tutorial metaphor
- special recipe family naming

현대 simulation layer와 대응할 때는:

```text
Earth  → Dirt / Stone / Mineral families
Water  → Water / Ice / Steam
Air    → gases / atmosphere future layer
Fire   → combustion phenomenon
Aether → exotic/future world primitive inspiration
```

### Alchemical trio

```text
Sulfur
Mercury
Salt
```

세 물질 모두 실제 Matter로도 강하고 역사적 상징성도 있어, **역사적 세계관과 실제 sandbox physics를 연결하는 이상적인 세트**다.

---

## 5. Tier D — Original Matter only where reality leaves a useful hole

여기서는 숫자를 아주 적게 유지한다.

### Pyrostor

**빈 자리:** 열을 `받는 즉시 전달`하는 물질은 많지만, 열을 저장했다가 나중에 사건으로 돌려주는 Matter가 부족하다.

```text
Heat absorption
→ capacity
→ delayed release
→ Heat / Pressure event
```

### Phase-Wax

**빈 자리:** 열을 이동 가능한 잠열 저장고로 쓰는 gameplay.

```text
solid ↔ liquid
→ absorb/release Heat
```

### Heat-Diode Material

**빈 자리:** conductivity는 있어도 열 흐름의 방향을 설계하는 construction Matter가 없다.

```text
Heat transfer strong one way
Heat transfer weak reverse way
```

### Vapor-Latch

**빈 자리:** GAS 자체를 `저장했다 조건에 따라 방출`하는 장치형 Matter.

```text
Steam/Gas capture
→ capacity
→ pressure condition
→ release
```

### Baroclast-style Material

**빈 자리:** Pressure가 파괴 외에도 구조를 움직이거나 변형하는 데 사용되도록 한다.

### Leak-Seal Material

**빈 자리:** Pressure difference 자체가 repair/cure trigger가 되는 자동수리 Matter.

---

## 6. What is deliberately not in the core

현재 core에서 제외하거나 늦춘다.

- `더 강한 Metal`만 반복하는 수많은 fictional alloys
- IP 고유 Matter의 직접 채택
- Time / Probability / observation-dependent matter
- arbitrary teleport / delete Matter
- Light-only decorative variants
- Biology variants that need full Agent simulation
- Electricity variants before an Electricity family exists
- multiple poisons/acids/fuels that differ only in damage strength

이들은 encyclopedia에는 남길 수 있지만 core vocabulary는 아니다.

---

## 7. Approximate shape of the first broad catalog

정확한 Material 수를 고정하지 않지만 방향은 다음 정도다.

```text
Current M0 + initial catalog      ~16
High-interaction real additions  ~15–20
Real exotic / space additions    ~5–8
Original gap-fillers              ~4–8
Historical concepts               separate discovery layer
```

즉 수백 종이 아니라 **대략 수십 종의 강한 Matter**로 먼저 풍부한 interaction graph를 만든다.

중요한 목표는 `30개 Material = 30개 기능`이 아니다.

```text
30~50개의 Matter
× 서로 다른 조건
× temperature
× pressure
× density
× phase
× combustion
× construction
= 수백 개의 발견 가능한 현상
```

이게 Powdergame 콘텐츠의 기본 스케일링 방식이다.

---

## 8. Core design sentence

> **현실의 강한 물질로 세계를 만들고, 역사적 자연관으로 세계에 의미를 붙이고, 현실이 제공하지 못하는 재미있는 빈칸에만 가상의 Matter를 발명한다.**
