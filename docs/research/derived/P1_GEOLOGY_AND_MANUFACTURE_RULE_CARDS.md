# P1 Geology & Irreversible Manufacture Rule Cards

## Status

- Type: `DERIVED / PROTOTYPE CONTRACT CANDIDATE`
- Authority: non-authoritative research. This document does **not** modify `MATERIAL_SPEC.md`, `REACTION_SPEC.md`, `SIMULATION_SPEC.md`, M0 scope, Roadmap Phase 1, or Evidence Gates.
- Purpose: turn the first interaction graph's **P1 geology and manufacture bundle** into concrete Material identities, rule ownership, tuning seeds, fixtures and promotion evidence.
- Naming note: `P1` here means **Prototype Bundle 1**, not Roadmap Phase 1.

> **The prototype must prove that familiar Matter can create new terrain and manufactured Matter through shared world rules, without adding universal wetness, cooling-history or reaction-progress state to every Cell.**

Inputs:

- `../../specs/SIMULATION_SPEC.md`
- `../../specs/MATERIAL_SPEC.md`
- `../../specs/REACTION_SPEC.md`
- `../../architecture/decisions/ADR-0003-minimum-sufficient-physics.md`
- `../../development/TESTING.md`
- `INTERACTION_GRAPH_AND_CATALOG_DECISIONS.md`

---

## 1. Prototype claim

P1 must demonstrate four readable causal grammars:

```text
Dirt + Water
→ Mud
→ drying
→ Dirt
```

```text
Clay + Water
→ Wet Clay
→ Heat
→ Brick
```

```text
Lava + ordinary cooling
→ Basalt

Lava + rapid local quench
→ Obsidian
```

```text
Limestone + Acid
→ CO2 + neutralized liquid abstraction
```

The required player-facing conclusions are:

1. Water changes terrain consistency.
2. Heat can make an irreversible manufactured material instead of merely damaging Matter.
3. the local cooling condition can change a geological result;
4. Acid can reveal a gas-producing mineral reaction;
5. all four chains are compositions of generic movement, temperature and Material-owned local rules.

---

## 2. Scope boundary

### In scope

- new prototype Material identities;
- descriptor-level movement/density/thermal/pressure properties;
- Material-owned self transitions;
- existing Temperature propagation;
- existing Water/Steam/Lava/Acid interactions needed by the fixtures;
- semantic Discovery events;
- small deterministic fixtures and GPU scenario tests;
- descriptor-level slow update tiers.

### Explicitly out of scope

- changing M0 completion criteria;
- universal `wetness`, `cooling_rate`, `dissolution_progress` or `firing_progress` fields;
- exact soil moisture or chemical stoichiometry;
- a global chemistry solver;
- realistic sediment transport;
- concrete/cement industry;
- light, electricity, radiation or Agent systems;
- dripstone growth;
- exact global mass/energy accounting;
- a production Rule DSL editor.

---

## 3. Identity and palette decisions

Material identity and palette exposure are separate decisions.

| ID | Display name | Role | Movement | Player palette policy |
|---|---|---|---|---|
| `dirt` | Dirt | primary terrain candidate | `POWDER` | visible after content adoption |
| `mud` | Mud | staged result | `LIQUID`, low mobility | hidden until discovered; debug-spawnable |
| `clay` | Clay | primary mineral powder candidate | `POWDER` | visible after content adoption |
| `wet_clay` | Wet Clay | staged result | `STATIC` baseline | hidden until discovered; debug-spawnable |
| `brick` | Brick | manufactured result | `STATIC` | unlock/display after observed firing |
| `basalt` | Basalt | ordinary Lava solidification result | `STATIC` | unlock/display after observed cooling |
| `obsidian` | Obsidian | rapid-quench volcanic glass | `STATIC` | unlock/display after observed quench |
| `limestone` | Limestone / Calcite family | reactive rock candidate | `STATIC` | visible after content adoption |
| `carbon_dioxide` | CO2 | reaction support / future atmosphere identity | `GAS` | debug-spawnable in P1; player exposure later |

Existing identities used by P1:

```text
Water
Steam
Lava
Acid
Stone
Sand
EMPTY
```

### Why staged identities are preferred

```text
Dirt → Mud
Clay → Wet Clay
```

are represented as Material transitions rather than universal per-cell moisture values.

This keeps Cell cost independent of future content count and follows the current `No Universal Progress Field` contract.

### Wet Clay baseline decision

`Wet Clay` begins as weak `STATIC` Matter.

Reason:

- it produces an immediately readable `loose powder → shape-holding material` change;
- it requires no new cohesion solver;
- the player can shape it through the editor/brush before firing;
- a low-mobility LIQUID alternative remains available if static Wet Clay feels too dead.

Do not add a general cohesion field merely to make Wet Clay slump.

---

## 4. Descriptor tuning seeds

All values in this section are `TUNING_SEED`, not approved engine contracts.

### 4.1 Movable density ordering

Lower rank means lighter in the current explanatory convention.

```text
Steam          20   existing explanatory anchor
Smoke          30   provisional comparison anchor
CO2            45
Oil            70   existing explanatory anchor
Water          90   existing explanatory anchor
Mud           120
Dirt          135
Sand          150   existing explanatory anchor
Clay          160
Lava          180
Molten Metal  220   existing explanatory anchor
```

Required relationships, regardless of final numbers:

```text
Steam < CO2 < Oil < Water < Mud < Dirt < Sand < Clay < Lava
```

Notes:

- `STATIC` Matter does not participate in normal density swap even if the registry retains a reference density.
- CO2 must be heavier than Steam/Smoke in gas ordering, but remains a GAS using the common gas movement family.
- Mud must tend to sink through Water but flow more slowly than Water.

### 4.2 Cheap property classes

| Material | Conductivity | Heat capacity | Pressure resistance | Important tags |
|---|---|---|---|---|
| Dirt | low | medium | N/A | `soil`, `water_affine`, `organic_substrate` |
| Mud | medium | high | N/A | `soil`, `wet`, `slow_liquid` |
| Clay | low | medium | N/A | `mineral_powder`, `water_affine`, `fireable` |
| Wet Clay | medium | high | weak | `wet`, `shape_holding`, `fireable` |
| Brick | low-medium | medium | medium-high | `manufactured`, `ceramic`, `breakable` |
| Basalt | medium-low | medium-high | high | `volcanic`, `rock`, `remeltable` |
| Obsidian | low-medium | medium | medium, brittle | `volcanic`, `glasslike`, `brittle`, `remeltable` |
| Limestone | low-medium | medium | medium-low | `rock`, `carbonate`, `acid_reactive` |
| CO2 | very low | low | N/A | `gas`, `heavy_gas`, `nonflammable` |

Classes should compile to compact descriptors. They are not per-cell state.

### 4.3 Mobility seed

- `Mud`: shared LIQUID stencil with `mobility_tier = LOW`.
- `Wet Clay`: STATIC baseline.
- all other new movable Matter uses an existing shared movement family.

If the implementation has no descriptor-level mobility tier, P1 may initially run Mud as ordinary LIQUID for correctness, but product validation must explicitly judge whether the result is too watery.

---

## 5. Temperature tuning seed

Symbolic threshold names are the actual prototype interface. Numeric values are only starting points if the current authoring scale remains Celsius-like.

| Symbol | Celsius-like seed | Required ordering / purpose |
|---|---:|---|
| `mud_dry_threshold` | 60 | warm enough to dry exposed Mud slowly |
| `wet_clay_dry_threshold` | 80 | below firing, above ordinary ambient |
| `clay_fire_threshold` | 650 | irreversible Clay/Wet Clay → Brick |
| `lava_solidify_threshold` | 850 | Lava chooses volcanic solid result |
| `volcanic_remelt_threshold` | 1050 | Basalt/Obsidian → Lava |
| `rapid_quench_delta` | 400 | large local Temperature contrast seed |

Required relationship:

```text
ambient
< mud_dry_threshold
< wet_clay_dry_threshold
< water_boiling_threshold
< clay_fire_threshold
< lava_solidify_threshold
< volcanic_remelt_threshold
```

If existing Water/Lava tuning uses a different scale, keep the ordering and re-fit the numeric values. Do not silently reinterpret these as scientific constants.

---

## 6. Coarse rule order for P1

Within a Material's precompiled rule range:

```text
Phase Transition
→ Special Reaction
→ State Change
```

P1-specific consequences:

1. Clay firing wins over Water wetting when both conditions are present.
2. Wet Clay firing wins over ordinary drying.
3. Lava rapid-quench solidification is tested before ordinary solidification.
4. Acid/Limestone special reaction happens before any cosmetic state change.

No runtime sorting is required.

---

# 7. Rule cards

## P1-DIRT-001 — Water wetting

- **Owner:** `dirt`
- **Phase:** `State Change`
- **Rate tier:** `MEDIUM`
- **Neighborhood:** cardinal 4-neighbor
- **Condition:** at least one neighbor is `Water`; self temperature is below Water's boiling transition condition
- **Primary effect:** `TransformSelf(mud)`
- **Neighbor write:** none
- **Discovery event:** `TransformationObserved(dirt → mud, cause=water_contact)`
- **Counter/reverse:** P1-MUD-001
- **Cost note:** one small neighbor material check; no wetness state

Player sentence:

> 물은 흙을 없애지 않는다. 흙이 흐르게 만든다.

## P1-MUD-001 — Drying

- **Owner:** `mud`
- **Phase:** `State Change`
- **Rate tier:** `SLOW`
- **Neighborhood:** cardinal 4-neighbor
- **Condition:** no cardinal Water neighbor; self temperature >= `mud_dry_threshold`
- **Primary effect:** `TransformSelf(dirt)`
- **Neighbor write:** none
- **Discovery event:** `TransformationObserved(mud → dirt, cause=drying)`
- **Cost note:** no moisture progress; the slow scheduler provides temporal readability

Prototype abstraction:

- Water stored inside Mud is not globally accounted.
- P1 does not require Steam spawn on drying.
- If direct Mud → Dirt feels too abrupt, first adjust rate scheduling and visual feedback; do not add a float progress field by default.

## P1-CLAY-001 — Water plasticization

- **Owner:** `clay`
- **Phase:** `State Change`
- **Rate tier:** `MEDIUM`
- **Neighborhood:** cardinal 4-neighbor
- **Condition:** at least one neighbor is Water; firing rule did not match
- **Primary effect:** `TransformSelf(wet_clay)`
- **Neighbor write:** none
- **Discovery event:** `TransformationObserved(clay → wet_clay, cause=water_contact)`
- **Cost note:** reuses the Dirt wetting grammar but produces a different movement/structural result

## P1-CLAY-002 — Direct firing

- **Owner:** `clay`
- **Phase:** `Phase Transition`
- **Rate tier:** `FAST`
- **Condition:** self temperature >= `clay_fire_threshold`
- **Primary effect:** `TransformSelf(brick)`
- **Secondary effect:** optional small `EmitSimulationEvent(steam_or_dust_release)` for presentation only
- **Discovery event:** `TransformationObserved(clay → brick, cause=sustained_heat, semantic=manufacture)`
- **Counter/reverse:** none under ordinary P1 conditions

Direct dry Clay firing is allowed. Water is useful because it changes handling/shape, not because it is a mandatory recipe token.

## P1-WET-CLAY-001 — Air/heat drying

- **Owner:** `wet_clay`
- **Phase:** `State Change`
- **Rate tier:** `SLOW`
- **Neighborhood:** cardinal 4-neighbor
- **Condition:** self temperature >= `wet_clay_dry_threshold`; self temperature < `clay_fire_threshold`; no cardinal Water neighbor
- **Primary effect:** `TransformSelf(clay)`
- **Discovery event:** `TransformationObserved(wet_clay → clay, cause=drying)`

## P1-WET-CLAY-002 — Firing

- **Owner:** `wet_clay`
- **Phase:** `Phase Transition`
- **Rate tier:** `FAST`
- **Condition:** self temperature >= `clay_fire_threshold`
- **Primary effect:** `TransformSelf(brick)`
- **Secondary effect:** optional Steam presentation/spawn request only if an existing generic yield mechanism supports it cheaply
- **Discovery event:** `TransformationObserved(wet_clay → brick, cause=heat, semantic=irreversible_manufacture)`

Do not require a custom kiln entity. A hot enough local environment is the kiln.

## P1-LAVA-001 — Rapid quench

- **Owner:** `lava`
- **Phase:** `Phase Transition`
- **Rule order:** before P1-LAVA-002
- **Rate tier:** `FAST`
- **Neighborhood:** cardinal 4-neighbor Temperature + Material checks
- **Condition:** self temperature <= `lava_solidify_threshold`, and at least one is true:
  - cardinal neighbor is Water or Ice;
  - maximum local Temperature difference >= `rapid_quench_delta`.
- **Primary effect:** `TransformSelf(obsidian)`
- **Discovery event:** `TransformationObserved(lava → obsidian, cause=rapid_quench)`
- **Cost note:** uses current local conditions; no cooling-history state

This is an intentionally cheap proxy for cooling rate.

## P1-LAVA-002 — Ordinary solidification

- **Owner:** `lava`
- **Phase:** `Phase Transition`
- **Rule order:** after rapid quench
- **Rate tier:** `FAST`
- **Condition:** self temperature <= `lava_solidify_threshold`
- **Primary effect:** `TransformSelf(basalt)`
- **Discovery event:** `TransformationObserved(lava → basalt, cause=cooling)`

The same Lava identity creates two results through ordered conditions rather than separate scripted recipes.

## P1-BASALT-001 — Remelting

- **Owner:** `basalt`
- **Phase:** `Phase Transition`
- **Rate tier:** `FAST`
- **Condition:** self temperature >= `volcanic_remelt_threshold`
- **Primary effect:** `TransformSelf(lava)`
- **Discovery event:** `TransformationObserved(basalt → lava, cause=extreme_heat)`

## P1-OBSIDIAN-001 — Remelting

- **Owner:** `obsidian`
- **Phase:** `Phase Transition`
- **Rate tier:** `FAST`
- **Condition:** self temperature >= `volcanic_remelt_threshold`
- **Primary effect:** `TransformSelf(lava)`
- **Discovery event:** `TransformationObserved(obsidian → lava, cause=extreme_heat)`

P1 does not yet require an Obsidian thermal-shock shatter rule. Its first visible differences are origin, appearance and brittle pressure class.

## P1-LIMESTONE-001 — Acid carbonate response

- **Owner:** `limestone`
- **Phase:** `Special Reaction`
- **Rate tier:** `FAST` while the contact region is active
- **Neighborhood:** cardinal 4-neighbor
- **Condition:** at least one neighbor is `Acid`
- **Primary effect:** `TransformSelf(carbon_dioxide)`
- **Discovery event:** `ReactionObserved(limestone + acid, outputs=[carbon_dioxide, neutralized_liquid], semantic=gas_generation)`
- **Cost note:** self-write only; the solid Cell becomes the visible gas output

This is cell-level reaction bookkeeping, not molecule tracing.

## P1-ACID-001 — Carbonate neutralization abstraction

- **Owner:** `acid`
- **Phase:** `Special Reaction`
- **Rate tier:** `FAST`
- **Neighborhood:** cardinal 4-neighbor
- **Condition:** at least one neighbor is `Limestone`
- **Primary effect:** `TransformSelf(water)`
- **Discovery event:** emitted only once per semantic reaction cluster if event deduplication exists; otherwise Limestone owns the visible discovery event

P1 intentionally omits dissolved calcium salt as a separate Matter.

Approximate pair result:

```text
Limestone Cell + Acid Cell
→ CO2 Cell + Water Cell
```

### Pair fan-out risk

With pure self-write rules, several Limestone Cells can observe the same Acid Cell in one Tick and all transform before the Acid becomes Water.

Baseline policy:

1. accept this cheap approximation for the first fixture;
2. restrict the rule to cardinal neighbors;
3. inspect whether one Acid Cell erases an implausibly large front;
4. only if the artifact is clearly harmful, escalate this reaction to a pair Claim/Resolve path.

Do not add a universal Acid capacity byte to solve this one interaction.

---

## 8. Material-owned rule ledger

| Owner | Ordered rules |
|---|---|
| Dirt | water contact → Mud |
| Mud | warm + no Water → Dirt |
| Clay | high Heat → Brick; else Water contact → Wet Clay |
| Wet Clay | high Heat → Brick; else warm + no Water → Clay |
| Lava | rapid quench → Obsidian; else solidify → Basalt |
| Basalt | extreme Heat → Lava |
| Obsidian | extreme Heat → Lava |
| Limestone | Acid contact → CO2 |
| Acid | Limestone contact → Water |
| Brick | none in P1 |
| CO2 | no reaction rule in P1; shared GAS movement only |

CO2 fire suppression belongs to the later combustion-control prototype, not this P1 pass.

---

## 9. Discovery metadata

The in-game Dictionary should reveal phenomena, not numeric thresholds.

Suggested semantic events:

```text
wetting_observed
slow_drying_observed
irreversible_firing_observed
ordinary_volcanic_solidification_observed
rapid_quench_observed
acid_mineral_gas_generation_observed
remelting_observed
```

Example player-facing discoveries:

- **Mud:** “Water can make loose earth flow.”
- **Brick:** “Enough Heat permanently changes Clay into a structural material.”
- **Basalt:** “Lava normally cools into dark volcanic rock.”
- **Obsidian:** “Rapid cooling makes Lava choose a glass-like path.”
- **CO2 reaction:** “Some pale rocks release a heavy gas when Acid reaches them.”

Do not reveal `650`, `850`, `rapid_quench_delta`, exact rule priority or hidden remaining discovery counts.

---

## 10. Prototype fixtures

All fixtures should be constructible by a small headless/reference hook and viewable in the production GPU simulation.

### P1-F1 — Soil wetting tray

**World seed:** small rectangular tray, Dirt slope, Water reservoir, warm plate region.

Expected sequence:

```text
Water reaches Dirt
→ interface becomes Mud
→ Mud moves more slowly than Water and settles below it
→ isolated warm Mud becomes Dirt again
```

Required observations:

- Mud is visibly distinct from Water;
- Dirt does not need a per-cell wetness field;
- the stable dried result can sleep;
- Water does not mutate Dirt by neighbor write.

Failure signals:

- Mud moves exactly like Water and adds no new toy;
- the whole Dirt mass changes instantly from one distant Water Cell;
- drying flickers Dirt↔Mud every Tick;
- stable terrain remains permanently active.

### P1-F2 — Clay kiln

**World seed:** two Clay chambers, a Water inlet, a moderate warm zone and a high-Heat zone.

Expected sequence:

```text
Clay + Water → Wet Clay
Wet Clay + moderate drying → Clay
Clay/Wet Clay + high Heat → Brick
```

Required observations:

- Wet Clay holds a shape distinct from Clay powder;
- Brick remains STATIC when Water is reintroduced;
- Brick is visually and structurally distinct from generic Stone;
- no dedicated kiln object or crafting menu is required.

Failure signals:

- Water contact makes Clay permanently indestructible;
- Brick is merely recolored Stone with no manufacturing meaning;
- Wet Clay needs a universal cohesion solver;
- high Heat cannot consistently win over drying/wetting rules.

### P1-F3 — Lava cooling fork

**World seed:** equal Lava sources enter two otherwise equivalent channels.

- channel A: ordinary cooling against Stone/ambient Matter;
- channel B: Water/Ice quench and large local Temperature contrast.

Expected sequence:

```text
A → predominantly Basalt
B → predominantly Obsidian
```

Required observations:

- result difference is readable without storing previous Temperature;
- rule order prevents normal Basalt transition from stealing rapid-quench cases;
- reheating either solid can return it to Lava;
- Water may independently become Steam through its own existing transition.

Failure signals:

- both channels produce the same result;
- a single cold diagonal Cell changes a huge remote Lava region;
- Obsidian requires a bespoke “Lava + Water recipe” outside generic transition rules;
- solidification creates invalid ownership or duplicate Matter.

### P1-F4 — Carbonate chamber

**World seed:** Limestone floor/wall, controlled Acid droplets, collection basin below.

Expected sequence:

```text
Acid contacts Limestone
→ Limestone Cells become CO2
→ Acid Cells become Water
→ CO2 moves as a heavy Gas
```

Required observations:

- visible gas generation;
- Acid is not an infinite permanent catalyst after contact;
- no hidden Air is required;
- reaction does not corrupt neighboring unrelated Cells.

Failure signals:

- one Acid Cell deletes a macroscopic cave wall in one Tick;
- pair self-write produces oscillation or invalid IDs;
- CO2 behaves like Steam despite a different density rank;
- the reaction requires a full chemistry pass.

### P1-F5 — Combined world vignette

A small product-validation scene should combine:

```text
Water cuts through Dirt into Mud
Clay near a hot chamber becomes Brick
Lava produces Basalt and Obsidian in different cooling zones
Acid reaches Limestone and releases CO2
```

The goal is not a scripted puzzle. The scene should remain editable and continue producing understandable follow-up interactions after the player changes it.

---

## 11. Automated semantic assertions

Exact pixel layouts are not required.

Minimum assertions:

### Dirt / Mud

- Dirt adjacent to Water eventually produces at least one Mud Cell.
- Mud isolated from Water in a warm region eventually produces Dirt.
- Mud density ordering tends below Water.

### Clay / Brick

- Clay or Wet Clay above firing threshold eventually becomes Brick.
- Brick does not revert through ordinary Water contact or moderate cooling.
- Wet Clay below firing threshold can dry back to Clay.

### Lava results

- rapid-quench fixture produces Obsidian.
- ordinary-cooling fixture produces Basalt.
- the two outcomes use the same Lava identity and ordered rules.
- Basalt and Obsidian can remelt under extreme Heat.

### Limestone / Acid

- Acid/Limestone contact produces CO2 and neutralizes at least one Acid Cell.
- all output Material IDs remain valid.
- one contact cluster does not create duplicate ownership.

### Invariants

- One Cell = Max One Matter.
- no out-of-bounds write.
- no NaN/Infinity Temperature or Pressure.
- no reaction requires a full-world runtime sort.
- stable final structures can become inactive/sleep candidates.

---

## 12. Required telemetry

Record at least:

```text
rule evaluation count by card ID
rule match count by card ID
Material transition count
active Cell count
active Chunk count
simulation tick time
reaction cost
Temperature cost
Claim/Resolve cost if pair escalation is tested
invalid ID / invariant violations
```

For product review, capture:

- before/after fixture screenshots;
- a short run showing each chain;
- final Material counts;
- commit SHA, build/config, hardware and world seed.

No absolute performance pass/fail number is introduced by this research document.

---

## 13. Promotion gate

P1 may become an adopted content proposal only when all are true:

1. **Mud earns its identity:** it is visibly different from Water and Dirt.
2. **Brick earns its identity:** the player can create it through Heat and it behaves as manufactured structure.
3. **Basalt and Obsidian earn separate identities:** cooling context changes the observed result.
4. **Limestone earns its identity:** Acid contact produces a memorable gas-generating reaction.
5. **Rule ownership remains local:** standard paths are Read Neighbors, Write Self.
6. **Cell cost remains disciplined:** no universal future state is introduced.
7. **Stable results can sleep.**
8. **The user finds at least one fixture worth experimenting with beyond the expected result.**

Failure of one candidate does not fail the entire bundle. Demote or merge the weak identity and preserve the successful grammar.

---

## 14. Fallback decisions

| Problem | First response | Do not do first |
|---|---|---|
| Mud is too watery | reduce descriptor mobility / schedule movement less often | add viscosity float to every Cell |
| Wet Clay is too static | test shared low-mobility LIQUID behavior | add universal cohesion solver |
| drying flickers | add temperature hysteresis or slower rate tier | add wetness progress globally |
| Basalt/Obsidian split is unreliable | tune local delta and neighbor conditions | store cooling history per Cell |
| Acid dissolves too many Cells | use pair Claim/Resolve for this reaction | add generic reaction-capacity byte to all Matter |
| Brick feels like Stone recolor | change rupture/thermal classes and visual feedback | invent multiple decorative Brick types |
| CO2 adds no value in P1 | keep it as support identity for P2 | force fire suppression into P1 gate |

---

## 15. Implementation order

After M0 foundation proof, implement this bundle in the following order:

```text
1. Register prototype identities and debug colors
2. Add descriptor seeds and density ordering
3. Implement Dirt/Mud and Clay/Wet Clay/Brick self transitions
4. Implement Lava ordered solidification and remelting
5. Implement Limestone/Acid paired self-write approximation
6. Add semantic Discovery events
7. Build four isolated fixtures
8. Build combined editable vignette
9. Record semantic + performance evidence
10. Promote, merge or demote each identity independently
```

Do not update `MATERIAL_SPEC`, `REACTION_SPEC`, Roadmap status or Milestone status merely because these cards exist. Adoption follows implementation evidence and explicit user review.
