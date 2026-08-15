# Powdergame Material Specification

이 문서는 Powdergame의 Matter가 무엇이며 어떤 최소 정보로 움직이고 변하는지를 정의한다.

---

## 1. Material은 현실 물질 DB가 아니다

Material은 Powdergame 우주를 구성하는 어휘다.

현실의 물질을 참고할 수 있지만 현실에 존재해야 할 필요는 없다.

유효한 Material의 기준은:

- 플레이어가 성격을 이해할 수 있음
- 다른 Matter/Field와 의미 있는 상호작용을 함
- 게임 세계 안에서 규칙이 일관됨
- 충분히 저비용으로 실행 가능함

이다.

예:

```text
Vibranium
- Static
- 매우 높은 pressure resistance
- thermal response 낮음
- 대부분의 reaction에 inert
```

이런 가상 Matter도 완전히 유효하다.

---

## 2. Material Identity

Material은 안정적인 identity를 가진다.

개념적 필드:

```text
id
name
movement_class
properties
tags
transitions
interaction_rules
schema_version
```

Material ID는 density나 다른 물성 순서와 결합하지 않는다.

> **ID는 Identity, Density는 Property.**

물성 변경 때문에 Material ID를 재배치하는 구조를 피한다.

---

## 3. Registration State

콘텐츠 상태를 구분한다.

- `REGISTERED` — catalog에 존재
- `IMPLEMENTED` — engine rule이 동작
- `VALIDATED` — milestone/test/play validation을 통과

등록되어 있다는 이유만으로 구현/검증된 것으로 취급하지 않는다.

초기 element catalog는 넓게 등록할 수 있지만 M0는 일부만 VALIDATED하면 된다.

현재 알려진 초기 catalog 방향:

- Sand
- Water
- Stone
- Wood
- Oil
- Acid
- Steam
- Seed
- Plant
- Salt
- Ice
- Lava
- Metal
- Glass
- Smoke

Fire는 M0에서 단순 Matter보다 combustion phenomenon/state로 취급한다.

---

## 4. Movement Class

M0 기본 movement family:

```text
STATIC
POWDER
LIQUID
GAS
```

Material 하나하나에 완전히 별도 movement shader를 만들지 않는다.

공통 Behavior Class가 있고 Material은 작은 property 차이로 성격을 조절한다.

### STATIC

일반 gravity/density movement 없음.

### POWDER

아래/아래 대각선 중심의 local movement.

### LIQUID

아래 → 대각선 → lateral local movement.

### GAS

높은 mobility와 위/대각/측면 local movement. 단, 안정 bulk에서 무의미한 이동을 계속하지 않는다.

---

## 5. Density Rank

Density는 현실 kg/m³ 수치가 아니라 게임 내부의 **local displacement order**다.

정밀 float가 필요하지 않은 한 작은 integer rank를 사용한다.

핵심 연산:

```text
A > B
A == B
A < B
```

예시 값은 단지 설명용이다.

```text
Steam       20
Oil         70
Water       90
Sand        150
MoltenMetal 220
```

### 중요한 원칙

- density는 per-cell 저장하지 않는다.
- Material Registry/compact descriptor의 property다.
- 같은 rank의 Matter 사이에는 density 기반 displacement가 없다.
- STATIC은 일반 density swap 대상이 아니다.

> **부력을 계산하지 않는다. 정렬한다.**

---

## 6. Properties use Minimum Sufficient Representation

각 물성은 gameplay에 필요한 가장 싼 표현을 쓴다.

```text
정확한 연속값 필요 → f32 등 numeric
순서만 필요         → integer rank
Yes/No만 필요       → bit
몇 단계면 충분       → small enum
```

예시 방향:

```text
movement_class        small enum
movable               bit/implicit by class
density_rank          small integer
thermal_participation bit/class
conductivity          bit/rank if needed
pressure_resistance   rank/integer
flammable             bit/tag
optical traits        bits/small enum
radiation resistance  rank
```

정확한 bit width는 실제 packing/benchmark 단계에서 결정한다.

모든 물성을 무조건 f32로 만들지 않는다.

---

## 7. Temperature-related Properties

Temperature state 자체는 M0 baseline에서 Cell의 f32 Field다.

Material은 필요에 따라 다음과 같은 thermal property를 가질 수 있다.

- thermal participation
- conductivity class/value
- heat capacity class/value
- transition thresholds
- ignition threshold/condition

현실 단위를 그대로 고집하지 않는다.

Material 간 차이를 플레이어가 느끼는 데 필요한 정도만 표현한다.

---

## 8. Pressure-related Properties

Material은 필요에 따라:

- pressure resistance
- mobility
- breakable
- rupture threshold

등을 가진다.

Pressure를 받았을 때:

- movable Matter는 밀릴 수 있음
- resistant Matter는 버팀
- threshold를 넘으면 파열 가능

이 관계는 현실 재료 공학을 정확히 재현하기보다 게임에서 이해 가능한 강도 차이를 만든다.

---

## 9. Transitions

모든 Material은 transition rule을 가질 **가능성**이 있다.

현재 transition이 없으면 빈 목록이면 된다.

모든 Material에 현실적인 Solid/Liquid/Gas 세 상태를 강제하지 않는다.

예:

```text
Ice ↔ Water ↔ Steam
```

도 가능하고:

```text
Metal
→ Molten Metal
→ Burning Metal
→ Ash-like Matter
```

같은 가상 graph도 가능하다.

### Transition 조건

가능한 조건:

- Temperature
- Pressure
- local neighbor
- current state/tag
- future Field

### Transition 결과

- self Matter replacement
- state/flag change
- Heat/Pressure effect
- secondary Matter spawn request

등이 가능하다.

---

## 10. Interaction Ownership

현재 방향은 **Material이 자기 interaction 목록을 소유**하는 것이다.

예:

```text
Oil
- hot neighbor → ignite
- Acid neighbor → special response

Metal
- Acid neighbor → corrode
```

A+B 관계를 완벽하게 global Reaction Catalog 하나로 정규화하려고 엔진/콘텐츠 구조를 과도하게 복잡하게 만들지 않는다.

중복 가능성은 허용하며 coarse phase order와 First-Match로 처리한다.

자세한 규칙은 `REACTION_SPEC.md`를 따른다.

---

## 11. Tags

Rule authoring에서 Material 이름을 하드코딩하는 것을 줄이기 위해 property/tag를 사용할 수 있다.

예:

```text
flammable
organic
oxidizable
exotic
inert
```

하지만 tag system 자체가 무거운 runtime query engine이 되어서는 안 된다.

콘텐츠 로드/compile 단계에서 실제 Material별 small rule set으로 정리하는 방향을 선호한다.

---

## 12. No Universal Future State

미래 콘텐츠를 대비해 모든 Cell에 다음을 미리 넣지 않는다.

```text
oxidation_progress
wetness_progress
growth_progress
corrosion_progress
mana_progress
...
```

느린 변화는 가능하면 Material 단계로 표현한다.

```text
Copper
→ Weathered Copper
→ Oxidized Copper
```

정말 연속값이 게임적으로 중요해질 때만 해당 시스템용 state를 추가하고 benchmark한다.

---

## 13. M0 Validated Matter Set

현재 M0에서 실제 behavior를 검증할 최소 세트:

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

Fire/Combustion은 phenomenon/state.

이 세트의 목적은 콘텐츠량이 아니라 다음 세계 문법을 증명하는 것이다.

- Static
- Powder
- Liquid
- Gas
- density displacement
- temperature-driven transition
- cooling-driven transition
- combustion
- phase expansion
- pressure generation
- rupture/vent

---

## 14. Example: Material Definition Philosophy

가상의 `Cryosteel`을 만들 때 개발자는 핵심 성격을 결정할 수 있다.

```text
Cryosteel
- STATIC
- dense
- cold에서 매우 강함
- high heat에서 취약한 형태로 transition
- 특정 Matter와 열을 강하게 교환
```

엔진은 개별 결과를 모두 하드코딩하지 않는다.

예를 들어 Cryosteel이 Lava의 열을 빼앗고 Lava가 자기 transition rule에 의해 Stone이 되며 그 Stone이 flow를 막아 Pressure를 만든다면, 그 연쇄는 별도의 `Cryosteel causes pressure explosion` rule 없이 생길 수 있다.

이것이 Material 설계의 목표다.
