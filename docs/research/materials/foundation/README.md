---
title: Powdergame Foundation Materials
type: material-family-index
id: material-family-foundation
status: active
implementation_state: mixed
updated: 2026-08-17
last_verified: 2026-08-17
sources:
  - ../../../vision/USER_VISION.md
  - ../../../specs/MATERIAL_SPEC.md
  - ../../../specs/SIMULATION_SPEC.md
  - ../../../specs/REACTION_SPEC.md
  - ../../../planning/MILESTONES.md
  - ../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md
  - ../../encyclopedia/01A_FOUNDATION_CATALOG.md
tags:
  - foundation
  - material
  - family
  - world-grammar
---

# Foundation Materials

> 세계가 움직이고, 흐르고, 얼고, 타고, 자랄 수 있게 하는 가장 작은 공통 어휘.

Foundation은 모든 Material을 한꺼번에 구현하겠다는 목록이 아니다. 현재 M0 구현 브랜치에 등록·구현된 9개 identity와, 이후 세계 문법을 잇기 위해 보존하는 7개 초기 catalog 방향을 같은 개념 계약으로 읽기 위한 컬렉션이다.

## 상태 경계

### M0 baseline — 코드에서 확인된 9종

아래 9종은 구현 브랜치 `177879c2e2e916066f376a4465c00430a0cdd8ac`의 Registry와 subsystem test에서 identity와 behavior가 확인되었다. 따라서 `status: adopted`, `implementation_state: implemented`로 기록한다.

개별 Material이 모든 Product Gate를 독립적으로 통과했다는 뜻은 아니다. 전체 M0가 아직 진행 중이고 증거가 subsystem 단위이므로 어느 페이지도 `validated`로 올리지 않는다.

| Engine ID | Material | Movement | 개념 상태 | 구현 상태 | 노출 정책 |
|---:|---|---|---|---|---|
| 1 | [Boundary Block](boundary-block.md) | `STATIC` | `adopted` | `implemented` | world boundary only |
| 2 | [Stone](stone.md) | `STATIC` | `adopted` | `implemented` | Foundation source, player palette pending |
| 3 | [Sand](sand.md) | `POWDER` | `adopted` | `implemented` | Foundation source, player palette pending |
| 4 | [Water](water.md) | `LIQUID` | `adopted` | `implemented` | Foundation source, player palette pending |
| 5 | [Oil](oil.md) | `LIQUID` | `adopted` | `implemented` | Foundation source, player palette pending |
| 6 | [Steam](steam.md) | `GAS` | `adopted` | `implemented` | result identity, player palette pending |
| 7 | [Smoke](smoke.md) | `GAS` | `adopted` | `implemented` | result identity, player palette pending |
| 8 | [Ice](ice.md) | `STATIC` | `adopted` | `implemented` | phase identity, player palette pending |
| 9 | [Wood](wood.md) | `STATIC` | `adopted` | `implemented` | Foundation source, player palette pending |

`EMPTY == 0`은 이 ID 표의 첫 Material이 아니다. Cell에 Matter가 없음을 나타내는 부재 값이며 Registry에 등록되지 않는다. ID는 density, palette, 문서 나열 순서와 무관하다.

### Existing catalog direction — 아직 코드에 없는 7종

아래 항목은 [Material Specification](../../../specs/MATERIAL_SPEC.md)의 초기 catalog 방향과 research graph에 남아 있지만 현재 Registry에 없다. 따라서 `status: candidate`, `implementation_state: not_registered`를 유지한다.

| Material | Movement 방향 | 개념 상태 | 구현 상태 | 노출 정책 |
|---|---|---|---|---|
| [Acid](acid.md) | `LIQUID` | `candidate` | `not_registered` | adoption 전 보류 |
| [Seed](seed.md) | `POWDER` | `candidate` | `not_registered` | adoption 전 보류 |
| [Plant](plant.md) | `STATIC` | `candidate` | `not_registered` | 성장 결과로 발견 후 노출 후보 |
| [Salt](salt.md) | `POWDER` | `candidate` | `not_registered` | adoption 전 보류 |
| [Lava](lava.md) | `LIQUID` | `candidate` | `not_registered` | adoption 전 보류 |
| [Metal](metal.md) | `STATIC` | `candidate` | `not_registered` | adoption 전 보류 |
| [Glass](glass.md) | `STATIC` | `candidate` | `not_registered` | 제조 결과로 발견 후 노출 후보 |

P1 Rule Card가 Acid와 Lava를 입력으로 사용하더라도 Rule Card는 구현 계약이 아니다. identity 등록과 prototype evidence가 생기기 전에는 이 두 페이지를 `prototype`, `registered`, `implemented`로 승격하지 않는다.

## Foundation이 담당하는 세계 문법

| 문법 | 대표 Material | 플레이어가 읽을 핵심 |
|---|---|---|
| `STATIC` | Boundary Block, Stone, Ice, Wood; future Metal/Glass/Plant | 흐름을 막고 형태를 유지한다. STATIC도 열·연소·압력 같은 별도 법칙에는 반응할 수 있다. |
| `POWDER` | Sand; future Seed/Salt | 아래로 떨어지고 쌓이며 빈틈으로 무너진다. |
| `LIQUID` | Water, Oil; future Acid/Lava | 흐르고 층을 만들며 접촉면을 다음 반응으로 연결한다. |
| `GAS` | Steam, Smoke | 위·대각·옆으로 이동하고 밀도 차이와 제한된 수명을 드러낸다. |
| Temperature | Ice, Water, Steam, Wood, Oil; future Lava/Glass/Metal | 열은 상변화·연소·제조의 공통 원인이다. |
| Phase Transition | Ice ↔ Water ↔ Steam; future Sand → Glass, Metal → molten stage | 같은 조건이 Material마다 다른 전이를 선택한다. |
| Combustion | Wood, Oil → Heat + Smoke | Fire는 영구 주황색 Matter가 아니라 연소 state/phenomenon이다. |
| Pressure | Steam, Wood, Stone, Boundary Block | 막힌 상변화가 압력을 만들고 약한 구조·열린 경계와 연쇄된다. |
| Growth | future Seed → Plant | 움직이는 잠재 생명과 자리 잡은 성장을 구분한다. |
| Chemistry | future Acid, Salt | 선택적 용해와 용액 상태가 지질·금속·상변화에 새 조건을 만든다. |

## Family 관계

### Water family

```text
Ice ↔ Water ↔ Steam
```

[Water](water.md)는 phase family의 중심이면서 P1에서 [Dirt](../p1/dirt.md)/[Mud](../p1/mud.md), [Clay](../p1/clay.md)/[Wet Clay](../p1/wet-clay.md), [Lava](lava.md)/[Obsidian](../p1/obsidian.md)을 잇는다. 향후 [Salt](salt.md)와 만나 Brine을 만들고 [Seed](seed.md)/[Plant](plant.md)의 성장 조건이 될 수 있다.

### Terrain and mineral family

```text
Stone ─┬─ Basalt
       ├─ Obsidian
       └─ Limestone

Sand ──future Heat──> Glass
```

[Stone](stone.md)은 안정된 구조 baseline이다. [Basalt](../p1/basalt.md), [Obsidian](../p1/obsidian.md), [Limestone](../p1/limestone.md)은 각각 냉각 맥락, 화산유리, 산 반응이라는 별도 동사를 증명해야 독립 identity를 유지한다. [Sand](sand.md)는 낙하하는 Powder이고 [Glass](glass.md)는 향후 열로 제조되는 STATIC 결과이므로 같은 family 안에서도 역할이 다르다.

### Combustion family

```text
Wood / Oil
→ combustion state
→ Heat + Smoke
→ future Ash / ecology / chemistry inputs
```

[Wood](wood.md)는 구조를 가진 고체 연료, [Oil](oil.md)은 흐르며 Water 위에 층을 이루는 액체 연료다. [Smoke](smoke.md)는 연소의 이동 가능한 흔적이다. Fire 자체는 이 family의 Material이 아니다.

### Volcanic family

```text
Lava ──ordinary cooling──> Basalt
Lava ──rapid quench──────> Obsidian
Basalt / Obsidian ──extreme Heat──> Lava
```

[Lava](lava.md)는 P1의 source identity 후보이고 [Basalt](../p1/basalt.md)와 [Obsidian](../p1/obsidian.md)은 서로 다른 냉각 맥락을 기록하는 결과 후보다. 이 문서화가 해당 Rule의 구현을 의미하지 않는다.

### Soil, manufacture, and ecology

```text
Dirt + Water → Mud
Clay + Water → Wet Clay ──Heat──> Brick
Seed + suitable Dirt / Water → future Plant
Plant → future Tree / Wood
```

[Seed](seed.md)와 [Plant](plant.md)는 아직 생태 구현이 없는 Foundation 방향이다. P1의 [Dirt](../p1/dirt.md), [Mud](../p1/mud.md), [Clay](../p1/clay.md), [Wet Clay](../p1/wet-clay.md), [Brick](../p1/brick.md)은 Water와 Heat가 지형·제조에 쓰이는지 먼저 검증한다.

### Chemistry and engineering

```text
Limestone + Acid → future CO2 + neutralized liquid abstraction
Salt + Water → future Brine
Metal → future Iron / Copper / Lead / Mercury
```

[Acid](acid.md), [Salt](salt.md), [Metal](metal.md)은 넓은 family의 시작점이다. P1의 [Limestone](../p1/limestone.md)과 [Carbon Dioxide](../p1/carbon-dioxide.md)는 Acid가 단순 삭제액보다 나은 반응을 만들 수 있는지 검토한다.

## Placeholder와 concrete identity

Foundation의 [Stone](stone.md), [Metal](metal.md), [Plant](plant.md)는 의도적으로 넓은 abstraction이다.

- Stone은 안정된 STATIC baseline을 맡는다. Basalt, Obsidian, Limestone은 별도 행동을 얻을 때만 분리된다.
- Metal은 구조·열전달·용융·부식의 공통 입구다. Iron, Copper, Lead, Mercury는 각각 부식, 열전도, 밀도/저융점, 액체 금속이라는 기억할 동사를 증명해야 한다.
- Plant는 가장 단순한 성장 결과다. Tree, Vine, Moss, Fungus는 성숙, 표면 추종, 습윤 표면 식민, 유기물 분해가 실제 놀이가 될 때 분리한다.

Foundation 페이지를 만든 사실만으로 이런 future identity를 Registry나 SPEC에 자동 등록하지 않는다.

## Palette / Discovery 원칙

구현 브랜치에는 여러 진단용 renderer palette와 직접 scenario 구성 경로가 있지만, 일반 플레이어가 Material을 고르는 palette/brush UI는 아직 없다. 따라서 페이지의 `palette_policy`는 **현재 노출 사실이 아니라 향후 제품 정책**이다.

- 안정된 source Material은 Foundation palette 후보로 둘 수 있다.
- Steam, Smoke, Glass처럼 인과 사슬의 결과가 중요한 identity는 먼저 세계에서 관찰하게 하는 편을 우선한다.
- Boundary Block은 세계 가장자리 편집 계약이며 일반 source palette 항목으로 취급하지 않는다.
- 정확한 threshold, rule priority, 숨은 발견 개수는 플레이어 Dictionary에 공개하지 않는다.
- 개발용 Material Wiki와 플레이어가 관찰로 채우는 Dictionary는 분리한다.

## Material이 아닌 Foundation 개념

- [`EMPTY`](../../../specs/SIMULATION_SPEC.md#34-empty)는 Matter가 없는 상태다. 숨은 Air도, 열·압력 매질도 아니다.
- [Fire / Combustion](../../../specs/REACTION_SPEC.md#11-fire--combustion)은 phenomenon/state다. 별도 Phenomenon Wiki가 생기기 전까지 Material 페이지를 만들지 않는다.
- [Temperature](../../../specs/SIMULATION_SPEC.md#13-temperature)와 [Pressure](../../../specs/SIMULATION_SPEC.md#15-pressure)는 Field다.
- [ ] TODO: Fire / Combustion을 다룰 때는 permanent orange Matter가 아닌 별도 Phenomenon Wiki 계약으로 작성한다.

## 관련 문서

- [Material Wiki](../README.md)
- [P1 — Geology & Irreversible Manufacture](../p1/README.md)
- [User Vision](../../../vision/USER_VISION.md)
- [Material Specification](../../../specs/MATERIAL_SPEC.md)
- [Simulation Specification](../../../specs/SIMULATION_SPEC.md)
- [Reaction Specification](../../../specs/REACTION_SPEC.md)
- [M0 Evidence Gates](../../../planning/MILESTONES.md)
- [Interaction Graph & Catalog Decisions](../../derived/INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md)
- [Foundation Catalog](../../encyclopedia/01A_FOUNDATION_CATALOG.md)
- [Implementation registry at `177879c`](https://github.com/Eeevah/Powdergame/blob/177879c2e2e916066f376a4465c00430a0cdd8ac/engine/core/src/material.rs)
