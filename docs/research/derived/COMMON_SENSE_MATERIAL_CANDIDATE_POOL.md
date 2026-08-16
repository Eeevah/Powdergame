# Powdergame Common-Sense Material Candidate Pool

## Status

- Type: `DERIVED / RESEARCH`
- Authority: non-authoritative candidate research
- Purpose: find materials a player would reasonably expect in an interaction sandbox, then keep only those with a memorable behavior verb.
- This document does **not** expand M0, register Materials, or change Evidence Gates.

> **The catalog should feel obvious in hindsight.**
>
> A player should look at the palette and think “of course that is here,” then discover that familiar materials combine into unfamiliar chains.

---

## 1. Selection principle

This pass intentionally avoids starting from fictional materials.

```text
common real-world material
→ memorable behavior?
→ multiple interaction partners?
→ cheap enough to express?
→ keep / reserve / merge

only after this pass:
interaction gap
→ historical / alchemical archetype
→ original Powdergame Matter if still necessary
```

A material does not earn a slot because it is famous, scientifically interesting, or visually different. It earns a slot because it adds a useful **verb**.

Examples:

```text
Iron      → RUST
Copper    → OXIDIZE / CONDUCT
Sponge    → ABSORB / SATURATE
Limestone → DISSOLVE / DEPOSIT
Fuse      → PROPAGATE IGNITION SLOWLY
Fungus    → DECOMPOSE / SPREAD
```

---

## 2. Cross-game lessons

This is not a copying exercise. References are used to discover recurring interaction grammar.

### Powder Game 2

Dan-Ball's official Powder Game 2 palette is compact but verb-heavy: Gunpowder explodes, Mud is wet Sand, Virus transforms dots, Nitroglycerin is a high-sensitivity explosive, Soapy Water creates bubbles, Metal conducts electricity, Vine grows around material, Mercury is a heavy liquid metal, and Fuse burns gradually.

Design lesson: **one element should feel like one toy.**

Reference: https://dan-ball.jp/en/javagame/dust2/

### Minecraft

Useful Minecraft material differentiation is behavioral rather than merely cosmetic. Copper visibly oxidizes over time; Amethyst grows crystal clusters from special substrate; Dripstone creates stalactite/stalagmite-like mineral structures; Powder Snow has a trapping/sinking identity; Tinted Glass changes light behavior.

Design lesson: **a familiar family can branch when each branch carries a remembered verb.**

References:

- https://feedback.minecraft.net/hc/en-us/articles/4402626897165-Minecraft-Caves-Cliffs-Part-1-1-17-Java
- https://edusupport.minecraft.net/hc/en-us/articles/4409173064084-What-s-new-in-the-GOAT-Update-version-1-17-30-5

### The Powder Toy

The Powder Toy's element vocabulary reinforces the usefulness of physical/chemical niches such as salt water, iron/corrosion, quartz/crystal behavior, ceramics, sponge, thermite and virus-like transformation.

Design lesson: **new interaction axis beats new rarity tier.**

Reference: https://powdertoy.co.uk/Wiki/

### Doodle God

Doodle God repeatedly uses familiar transformations such as Fire + Sand → Glass, Fire + Clay → Bricks, Limestone + Clay → Cement, and Saltpeter + Sulfur → Gunpowder.

Powdergame translation:

```text
symbolic recipe
→ observable world condition
→ local transformation
→ discoverable material / structure
```

Reference: https://gamefaqs.gamespot.com/pc/180182-doodle-god/faqs/64039

---

## 3. Decision states

- `FOUNDATION` — already part of the current foundation/catalog direction.
- `PROMOTE` — strong candidate for the first broad interaction catalog.
- `STRONG-RESERVE` — good material, but one more system or clearer contrast is needed.
- `RESULT/VARIANT` — should initially be a transformation result or variant rather than a new primary palette button.
- `FUTURE-SYSTEM` — valuable once Electricity / Light / Biology / Radiation / Agent systems justify it.
- `REFERENCE-ONLY` — useful idea but weak as an independent runtime Matter.

---

## 4. Foundation identities

Keep the current foundation/catalog vocabulary stable:

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
Acid
Seed
Plant
Salt
Lava
Metal
Glass
```

`Stone`, `Metal`, and `Plant` remain useful abstractions in the foundation. They can later decompose into specific materials without invalidating M0.

---

# 5. Common-sense candidate pool

## A. Soil, powder, sediment

| Candidate | Decision | Memorable verb | Why it matters |
|---|---|---|---|
| Dirt / Soil | `PROMOTE` | SUPPORT GROWTH / BECOME MUD | Gives Seed/Plant a substrate and closes an obvious terrain gap. |
| Mud | `RESULT/VARIANT` | FLOW SLOWLY / DRY | Prefer Dirt + Water result; promote only if movement is distinct enough. |
| Clay | `PROMOTE` | WET / SHAPE / FIRE → BRICK | Extremely strong transformation hub. |
| Gravel | `STRONG-RESERVE` | COARSE FALL / DRAIN | Useful if grain size becomes visible; otherwise overlaps Sand. |
| Ash | `STRONG-RESERVE` | COMBUSTION RESIDUE / SMOTHER | Strong byproduct identity; preferably emerge from burning. |
| Snow | `STRONG-RESERVE` | PACK / MELT / INSULATE | Familiar, but Ice already covers much of the thermal family. |
| Sawdust | `REFERENCE-ONLY` | LIGHT FLAMMABLE POWDER | Better as a Wood byproduct than core Matter. |
| Coal | `PROMOTE` | BURN SLOW / HEAT STRONGLY | Distinct solid fuel linking geology, fire and industry. |
| Charcoal | `RESULT/VARIANT` | PYROLYSIS PRODUCT / FUEL | Prefer Wood → Charcoal first. |
| Sulfur | `PROMOTE` | BURN / REACT / MAKE EXPLOSIVE | Real material plus strong historical/alchemical identity. |
| Nitrate Salt / Saltpeter | `PROMOTE` | OXIDIZER / EXPLOSIVE INGREDIENT | Gives explosives a recognizable precursor family. |
| Gunpowder | `PROMOTE` | FAST BURN → PRESSURE | Clear Heat → Combustion → Pressure bridge. |

## B. Rock, mineral, glass

| Candidate | Decision | Memorable verb | Why it matters |
|---|---|---|---|
| Basalt | `PROMOTE` | LAVA SLOW-COOL RESULT | Makes cooling history visible. |
| Obsidian | `PROMOTE` | RAPID-COOL VOLCANIC GLASS | Strong Lava + rapid cooling reward. |
| Limestone / Calcite | `PROMOTE` | DISSOLVE / PRECIPITATE | Connects Acid, Water, caves, deposition and construction. |
| Quartz / Crystal | `PROMOTE` | CRYSTALLIZE / GROW | Opens non-biological growth and future optical/electrical hooks. |
| Sandstone | `STRONG-RESERVE` | CEMENT SAND / ERODE | Good if sedimentation/compaction becomes visible. |
| Dripstone / Travertine | `STRONG-RESERVE` | DRIP / DEPOSIT / GROW | Great slow geology; needs dissolved-mineral semantics. |
| Granite | `REFERENCE-ONLY` | THERMAL MASS / STRUCTURE | Do not add just for geology variety. |
| Pumice | `STRONG-RESERVE` | FLOAT / POROUS ROCK | Promote if buoyancy/porosity makes it visibly unique. |
| Gypsum | `REFERENCE-ONLY` | HYDRATE / DEHYDRATE | Interesting but less readable than Limestone/Clay. |
| Tempered Glass | `RESULT/VARIANT` | FRACTURE DIFFERENTLY | Split only if fracture/thermal shock becomes gameplay. |

## C. Metals

Generic `Metal` should eventually become a foundation placeholder rather than the only metal in the world.

| Candidate | Decision | Memorable verb | Why it matters |
|---|---|---|---|
| Iron | `PROMOTE` | RUST / STRUCTURE / MELT | Best concrete default metal; corrosion creates world history. |
| Copper | `PROMOTE` | CONDUCT / OXIDIZE | Strong thermal identity now, electrical identity later. |
| Lead | `PROMOTE` | SINK / MELT EARLIER | Density makes it distinct before Radiation exists. |
| Mercury | `PROMOTE` | LIQUID METAL / VERY DENSE | Immediately unique. |
| Aluminum | `STRONG-RESERVE` | LIGHT METAL / THERMITE COMPONENT | Promote when low-density construction or thermite matters. |
| Zinc | `STRONG-RESERVE` | SACRIFICIAL CORROSION | Good chemistry identity but less obvious. |
| Gold | `STRONG-RESERVE` | DENSE / CORROSION-RESISTANT / CONDUCT | Needs more than rarity/value. |
| Tungsten | `FUTURE-SYSTEM` | EXTREME HEAT RESISTANCE | Strong with high-temperature engineering. |
| Titanium | `FUTURE-SYSTEM` | HIGH STRENGTH-TO-WEIGHT | Otherwise risks becoming “better Metal.” |
| Molten Metal | `RESULT/VARIANT` | LIQUID PHASE | Prefer phase state, not unrelated Matter. |
| Rust | `RESULT/VARIANT` | CORROSION FRONT | Important transformation product. |

## D. Liquids and soft matter

| Candidate | Decision | Memorable verb | Why it matters |
|---|---|---|---|
| Alcohol | `PROMOTE` | VOLATILE / FLAMMABLE / MIX WITH WATER | Familiar fuel distinct from Oil. |
| Brine | `PROMOTE` | CHANGE FREEZING / CORROSION | Strong Salt + Water + Ice + Metal bridge. |
| Soapy Water | `PROMOTE` | MAKE FOAM / TRAP GAS | Fills a major surface/foam gap. |
| Foam / Bubbles | `RESULT/VARIANT` | TRAP GAS / SLOW / COLLAPSE | Prefer as produced state. |
| Resin / Sap | `STRONG-RESERVE` | STICK / HARDEN / BURN | Links Plant/Wood to adhesion/manufacturing. |
| Tar | `STRONG-RESERVE` | SLOW FLOW / STICK / BURN LONG | Worthwhile if viscosity is visible. |
| Honey | `REFERENCE-ONLY` | STICK / SLOW | Same family as Resin/Tar; keep the strongest representative. |
| Base / Alkali archetype | `STRONG-RESERVE` | NEUTRALIZE ACID | Chemistry deserves an Acid counter. |
| Hydrogen Peroxide / oxidizer | `STRONG-RESERVE` | OXIDIZE / DECOMPOSE TO GAS | Reserve until chemistry grammar is clearer. |
| Nitroglycerin | `PROMOTE` | SHOCK-SENSITIVE LIQUID EXPLOSIVE | Crucially different trigger from Gunpowder. |

## E. Gases and atmosphere

| Candidate | Decision | Memorable verb | Why it matters |
|---|---|---|---|
| Methane | `PROMOTE` | ACCUMULATE / IGNITE | Makes confined gas a hazard. |
| Oxygen | `PROMOTE` | SUPPORT COMBUSTION | Valuable atmosphere ingredient; must not retroactively become an M0 prerequisite. |
| CO2 | `PROMOTE` | HEAVY GAS / SUPPRESS FIRE | Gives Dry Ice a destination and makes gas layering useful. |
| Hydrogen | `STRONG-RESERVE` | VERY LIGHT / FLAMMABLE | Strong but overlaps Methane until gas buoyancy is proven. |
| Ammonia | `STRONG-RESERVE` | TOXIC / WATER-SOLUBLE / LOW-BOILING | Strong industrial/space identity. |
| Fog / Cloud droplets | `FUTURE-SYSTEM` | CONDENSE / RAIN / SCATTER | Better as emergent atmosphere state. |
| Poison Gas archetype | `FUTURE-SYSTEM` | HARM BIOLOGY | Needs Biology/Agents. |

## F. Thermal and extreme real matter

| Candidate | Decision | Memorable verb | Why it matters |
|---|---|---|---|
| Dry Ice | `PROMOTE` | SUBLIMATE DIRECTLY TO CO2 | Extremely readable phase-change oddity. |
| Methane Clathrate | `STRONG-RESERVE` | WARM → RELEASE METHANE | Exceptional chain potential. |
| Aerogel | `STRONG-RESERVE` | INSULATE / BREAK EASILY | Clean engineering tradeoff. |
| Perchlorate Dust | `STRONG-RESERVE` | HEAT → OXIDIZING BEHAVIOR | Great Mars/regolith surprise. |
| Ammonia Ice | `STRONG-RESERVE` | HEAT → AMMONIA GAS | Useful outer-system ice family. |
| Tholin-like Tar | `REFERENCE-ONLY` | HAZE → ORGANIC TAR | Good world flavor, weaker immediate verb. |
| Regolith | `REFERENCE-ONLY` | ABRASIVE DRY DUST | Environment texture unless specialized. |

## G. Manufactured / construction

| Candidate | Decision | Memorable verb | Why it matters |
|---|---|---|---|
| Brick | `PROMOTE` | CLAY + HEAT → STRUCTURE | Excellent world-manufactured material discovery. |
| Cement | `STRONG-RESERVE` | BIND / REACT WITH WATER | Arrive with Limestone and curing semantics. |
| Concrete | `STRONG-RESERVE` | POUR / SET / HARDEN | Strong construction result once setting matters. |
| Ceramic | `STRONG-RESERVE` | FIRE CLAY / HEAT-RESIST / BRITTLE | Distinct if high-temperature structures matter. |
| Sponge | `PROMOTE` | ABSORB WATER / SATURATE | Perfect one-material-one-verb toy. |
| Fuse | `PROMOTE` | BURN SLOWLY / CARRY TRIGGER | Ideal manufactured causal line. |
| Rubber | `STRONG-RESERVE` | BOUNCE / INSULATE / BURN | Strong once elasticity/electricity exists. |
| Paper | `RESULT/VARIANT` | BURN FAST / ABSORB | Better as processed Wood/Plant result. |
| Fabric | `FUTURE-SYSTEM` | WICK / FLEX / BURN | Needs flexible/capillary semantics. |

## H. Biology / ecology

Biology should branch by **growth topology and ecological role**, not by species count.

| Candidate | Decision | Memorable verb | Why it matters |
|---|---|---|---|
| Tree | `PROMOTE` | GROW → PRODUCE WOOD | Makes Plant part of a material production cycle. |
| Vine | `PROMOTE` | CLIMB / FOLLOW SURFACES | Visibly different growth topology. |
| Moss | `PROMOTE` | COLONIZE WET STONE | Strong Water + Stone + Biology connector. |
| Fungus / Mold | `PROMOTE` | DECOMPOSE ORGANIC MATTER / SPREAD | Adds decomposition and reality-first contagion. |
| Algae | `STRONG-RESERVE` | GROW IN WATER | Stronger with Light/Oxygen. |
| Bacteria | `STRONG-RESERVE` | DECOMPOSE / FERMENT / INFECT | Needs biological eligibility rules. |
| Virus | `FUTURE-SYSTEM` | INFECT ELIGIBLE LIFE / PROPAGATE | Real virus should infect biology, not arbitrary Stone. |
| Blood | `FUTURE-SYSTEM` | FLOW / COAGULATE | Useful with Agents/life damage. |
| Bone | `FUTURE-SYSTEM` | STRUCTURE / CHAR / DISSOLVE | Later biological material. |
| Chitin / Shell | `FUTURE-SYSTEM` | LIGHT ARMOR / DECOMPOSE | Later biological manufacturing. |
| Fertilizer / Nutrient | `STRONG-RESERVE` | ACCELERATE GROWTH / OVERGROW | Valuable once growth consumes resources. |

---

# 6. Recommended first broad catalog

The first broad catalog should be a compact interaction vocabulary, not a periodic table.

### Current foundation: 16 identities

Keep the current foundation/catalog direction as the base layer.

### Promote after foundation proof: 29 named identities + one intentionally empty slot

```text
Dirt
Clay
Brick
Coal
Sulfur
Nitrate Salt / Saltpeter
Gunpowder
Alcohol
Methane
Dry Ice
CO2
Oxygen
Brine
Mercury
Obsidian
Iron
Copper
Lead
Basalt
Limestone / Calcite
Quartz / Crystal
Tree
Vine
Moss
Fungus / Mold
Sponge
Soapy Water
Nitroglycerin
Fuse

+ 1 slot intentionally left open after prototype evidence
```

That yields roughly **45–46 primary identities including the current foundation**, depending on whether staged results such as Foam or Tree stages become explicit palette entries.

The empty slot is deliberate. Do not fill it because the list looks incomplete; fill it only when prototype evidence exposes a missing interaction verb.

---

# 7. Strong reserve

Do not forget these, but do not rush them:

```text
Mud
Gravel
Ash
Snow
Charcoal
Sandstone
Dripstone / Travertine
Pumice
Aluminum
Zinc
Gold
Rust
Resin / Sap
Tar
Base / Alkali
Hydrogen Peroxide / oxidizer
Hydrogen
Ammonia
Methane Clathrate
Aerogel
Perchlorate Dust
Ammonia Ice
Cement
Concrete
Ceramic
Rubber
Algae
Bacteria
Fertilizer
```

---

# 8. Historical / alchemical layer

Historical concepts enrich discovery language without forcing every old “element” into Cell Matter.

### Classical elements

```text
Earth  → soil / rock / mineral family concept
Water  → liquid / ice / steam family
Air    → gas / atmosphere family
Fire   → phenomenon, not Matter
Aether → future exotic / cosmological category
```

### Alchemical trio

These are unusually valuable because the symbolic roles also correspond to strong real materials:

```text
Sulfur  → active / burning / transforming
Mercury → fluid / metallic / mutable
Salt    → fixed / crystalline / soluble
```

Use the trio as discovery/lore vocabulary layered on top of real behavior, not as fake physics.

---

# 9. Reality-first answer to the Virus gap

The desired fun is **contagious local transformation**, not necessarily a literal virus.

Try this ladder first:

```text
Rust
Metal → Rust front

Fungus
organic residue → Fungus growth

Crystal seed
solution → Crystal growth

Biological Virus
eligible living substrate → infected state
```

Only if these are too narrow for the sandbox fantasy should Powdergame add an original cross-material transmuting contamination. Then it has a proven reason to exist rather than merely imitating another game.

---

# 10. Original Matter budget after this audit

This broad real-material pass reduces the need for fictional Matter.

Current strongest gap-fillers remain:

```text
Pyrostor
Heat storage → capacity → delayed release

Phase-Wax
phase change ↔ strong heat buffering

Heat-Diode Material
strong directional heat transport

Vapor-Latch
store Steam/Gas → conditional release
```

Even these should compete with real engineering analogues before implementation.

---

# 11. Prototype order

Implement by **interaction bundle**, not by category.

### Bundle A — Terrain becomes material

```text
Dirt + Water → Mud-like state
Clay + Water → workable clay
Clay + Heat → Brick
```

### Bundle B — Volcanic geology

```text
Lava + ordinary cooling → Basalt
Lava + rapid cooling / Water → Obsidian + Steam
Limestone + Acid → dissolution
mineral Water + deposition → Limestone/Calcite growth later
```

### Bundle C — Metals become distinct

```text
Iron + moisture/Brine → Rust
Copper + time/environment → oxidation state
Lead → dense low-melting ballast
Mercury → dense liquid metal
```

### Bundle D — Fire becomes chemistry

```text
Coal → long solid fuel
Alcohol → volatile liquid fuel
Methane → confined flammable gas
Sulfur + Nitrate-family + carbon fuel → Gunpowder family
Nitroglycerin → shock-triggered liquid explosive
Fuse → delayed ignition path
```

### Bundle E — Atmosphere and suppression

```text
Dry Ice + Heat → CO2
CO2 → settle / suppress combustion
Oxygen → combustion support
Methane + combustion condition → fire / pressure chain
```

### Bundle F — Soft matter

```text
Sponge + Water → saturation
Soapy Water + Gas/agitation → Foam
Foam → traps gas / slows motion / collapses
```

### Bundle G — Ecology topology

```text
Seed/Plant → Tree → Wood
Vine → climb solids
Moss → wet Stone colonization
Fungus → consume dead organic material
```

Prototype evidence from these bundles should decide whether the final catalog is 40, 50, or 60 Materials. **The count is an output, not a target.**

---

# 12. Final rule

Before adding a Matter, ask:

```text
1. What does it DO that the player can see?
2. What existing Matter does it react with?
3. What chain starts after that reaction?
4. Could a familiar real material do the same job better?
5. Is it truly Matter, or should it be a state/result/phenomenon/device?
```

If questions 1–3 do not have strong answers, the material belongs in the encyclopedia, not the game palette.

> **The target is a small enough vocabulary that every word has meaning, and a rich enough reaction graph that the combinations feel endless.**
