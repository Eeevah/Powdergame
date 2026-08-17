# Powdergame Commonsense Material Survey

## Status

- Type: `DERIVED`
- Authority: non-authoritative content research
- Purpose: identify familiar real-world materials that players reasonably expect in an interaction sandbox, then select only those with a distinct gameplay verb.
- This document does **not** expand M0, register runtime Materials, or change Evidence Gates.

> **상식적으로 있어야 하는 물질을 넓게 조사하되, 이름이 아니라 상호작용을 기준으로 남긴다.**

---

## 1. Research question

The encyclopedia became broad enough that the next risk is no longer “missing ideas.” It is **missing ordinary but high-value Matter** while exotic ideas occupy too much attention.

This survey therefore asks:

1. What real materials repeatedly appear in falling-sand / crafting / block sandboxes?
2. Which familiar real materials have a behavior that can be explained in one sentence?
3. Which of those behaviors are not already represented by the current Powdergame foundation?
4. Which candidates connect to several existing systems rather than becoming decorative palette swaps?

Reference pattern:

```text
common expectation
+ distinct verb
+ several interactions
+ cheap local rule
= strong Matter candidate
```

Sources used for comparison include DAN-BALL Powder Game 2, The Powder Toy documentation, official Minecraft material/block documentation, the existing Powdergame encyclopedia and candidate research.

---

## 2. What the comparison games teach

### Powder Game 2 — one element, one memorable verb

The official Powder Game 2 element list is unusually useful because many elements can be summarized by a single behavior:

```text
NITRO   → high-sensitivity explosive
SOAPY   → makes bubbles
VIRUS   → transforms nearby dots
METAL   → conducts electricity
ACID    → melts materials
VINE    → grows around material
MERCURY → heavy liquid metal
FUSE    → burns gradually
CLOUD   → rain / snow / thunder cloud
```

The lesson is not to copy the list. The lesson is that **a player remembers a material because it does something**.

### The Powder Toy — ordinary matter becomes interesting through interactions

The Powder Toy similarly gets a lot of mileage from familiar substances: Iron can rust, Sponge absorbs water, Dry Ice is a CO2 solid, Fuse propagates burning, and salt-water chemistry connects otherwise ordinary materials.

The lesson is that scientific familiarity is not a weakness. A familiar material becomes a good sandbox element when the simulation exposes the property people already associate with it.

### Minecraft — family diversity is earned by behavior

Minecraft provides a useful block-material lesson. Copper visibly oxidizes through stages; amethyst grows from a specific budding substrate; powder snow behaves unlike ordinary snow; pointed dripstone moves/accumulates water or lava over time; moss and vines turn surfaces into ecological substrates.

The lesson for Powdergame is:

```text
Stone ≠ every rock forever
Metal ≠ every metal forever
Plant ≠ every plant forever
```

But family members should split only when their behavior creates a different experiment.

---

## 3. Candidate hierarchy

Candidates are classified as:

- **A — Core-worthy:** strong enough to join the first broad interaction catalog after validation.
- **B — Strong reserve:** clearly useful, but the same gameplay family can initially work without it.
- **C — Variant / staged state:** valuable name or appearance, but not yet worth a separate Material identity.
- **D — Encyclopedia only:** real and interesting, but currently too weak, redundant, or dependent on absent systems.

---

## 4. Soil, granular and sediment family

### A — Dirt

**Verb:** supports life and changes consistency with water.

```text
Dirt + Water → Mud-like state
Dirt + Seed + Water → growth substrate
Heat / drying → loose Dirt
```

This is more fundamental than many exotic candidates because it connects terrain, Water and Biology.

### A — Clay

**Verb:** becomes formable when wet and permanently changes when fired.

```text
Clay + Water → Wet Clay
Wet Clay + Heat → Brick
```

### A — Gunpowder

**Verb:** converts ignition into rapid gas/pressure generation.

It should emerge from a production chain rather than behave as a magical generic explosive.

### B — Gravel

**Verb:** coarse granular material that is heavier and less fluid than Sand.

Only split from Sand if grain-scale movement can visibly differ without expensive simulation.

### B — Ash

**Verb:** light combustion residue that can be transported, mixed with Water, and later participate in soil/cement/fertilizer-like chains.

Ash is attractive because it turns combustion output into a new input.

### B — Snow

**Verb:** loose cold powder that compacts/melts and may become Ice/Water.

Snow earns separation from Ice if it behaves as POWDER rather than STATIC.

### B — Sawdust

**Verb:** very light combustible powder.

Its value is not “Wood but smaller”; it creates dust/fire transport behavior.

### C — Sandstone

Useful as a compacted Sand result, but defer a separate identity until erosion/breakage differs enough from generic Stone.

### C — Silt

Useful encyclopedia term; runtime identity only if sedimentation behavior becomes distinct.

---

## 5. Stone, mineral and crystal family

### A — Basalt

**Verb:** ordinary volcanic solidification product.

```text
Lava + moderate cooling → Basalt
Lava + rapid cooling → Obsidian
```

This immediately makes cooling rate meaningful.

### A — Obsidian

**Verb:** rapid cooling produces a glass-like brittle volcanic solid.

Already a strong candidate because it distinguishes cooling history rather than adding another cosmetic rock.

### A — Limestone / Calcite family

**Verb:** dissolves/reacts with Acid and supports precipitation/deposition chains.

Potential chains:

```text
Limestone + Acid → dissolution / gas candidate
mineral-rich Water + evaporation → Calcite deposit
Limestone → Cement family (later manufacturing)
```

### A — Quartz / Crystal

**Verb:** crystal growth / precipitation seed.

Quartz is preferable to a generic fantasy “Crystal” when reality can provide the required role.

### B — Pumice

**Verb:** stone that is unusually light because of trapped gas/pores.

Useful when buoyancy/density contrast between rocks matters.

### B — Dripstone / Mineral Deposit

**Verb:** slow deposition from mineral-bearing water.

This can provide geology that visibly grows without invoking Biology.

### B — Amethyst

**Verb:** visible staged crystal growth from a growth substrate.

Prefer it as a crystal-growth exemplar, not as a decorative gemstone category.

### C — Granite / Diorite / Andesite

Keep as encyclopedia/visual variants until each earns a unique reaction or mechanical profile. Three differently colored Stones are not three gameplay verbs.

---

## 6. Metal family — generic Metal is not enough forever

The first implementation can keep generic `Metal`, but the broader game should eventually decompose it.

### A — Iron

**Verb:** rusts/corrodes and acts as the baseline structural metal.

```text
Iron + Water/Oxygen → Rust candidate
Iron + Brine → faster corrosion
Heat → Molten Iron / metal transition
```

### A — Copper

**Verb:** strongly transports heat and develops an oxide/patina state.

Later, Electricity gives it a second natural role rather than inventing one.

### A — Lead

**Verb:** very dense metal with a comparatively low melting temperature for a structural metal.

This gives density sorting and easy metal melting a clear exemplar.

### A — Mercury

**Verb:** heavy liquid metal.

It breaks the intuitive rule that “metal = solid,” making it extremely readable in a sandbox.

### B — Aluminum

**Verb:** lightweight metal with useful heat behavior.

Add when weight/density engineering is rich enough to make the difference obvious.

### B — Zinc

**Verb:** sacrificial/protective corrosion role.

Very interesting once corrosion is mature, but premature before Iron/Rust exists.

### B — Gold

**Verb:** chemically resistant, very dense, conductive metal.

Strong future Electricity/corrosion material; weak before those systems exist.

### B — Tungsten

**Verb:** extreme high-temperature metal.

Useful as an engineering endpoint only after ordinary metal melting is already fun.

### C — Steel / Stainless Steel

Manufactured variants are valuable, but they should emerge after Iron, carbon/fuel and corrosion systems justify them.

---

## 7. Liquid family

### A — Alcohol

**Verb:** mixes with Water yet burns readily.

This is much more distinct from Oil than another hydrocarbon liquid would be.

### A — Brine

**Verb:** dissolved Salt changes Water behavior.

```text
Salt + Water → Brine
Brine + cold → altered freezing
Brine + Iron → corrosion
```

### A — Nitroglycerin-like Nitro

**Verb:** liquid explosive that is especially sensitive to shock/pressure rather than merely flame.

Use the scientific/gameplay concept; implementation should remain an abstract sandbox rule, not a real preparation recipe.

### A/B — Soapy Water

**Verb:** captures Gas and makes Foam/bubbles.

```text
Soap + Water → Soapy Water
Soapy Water + Gas/agitation → Foam
Foam + Heat/Oil → collapse candidate
```

This opens an entire surface/bubble family that the current roster lacks.

### B — Resin

**Verb:** sticky liquid that hardens into a solid and can burn.

It links Plant/Wood to adhesives, Amber and manufactured surfaces.

### B — Tar

**Verb:** very viscous heavy fuel that burns differently from Oil.

Keep only if slow flow + long smoky combustion is visibly distinct.

### B — Honey

**Verb:** intuitive high-viscosity liquid.

Good player expectation, but less system-dense than Resin unless food/biology exists.

---

## 8. Gas and atmosphere family

### A — Oxygen

**Verb:** does not burn itself; accelerates combustion.

It should amplify local combustion without forcing a full atmospheric-composition solver.

### A — CO2

**Verb:** heavy nonflammable gas that suppresses combustion and can become Dry Ice.

### A — Methane

**Verb:** combustible gas whose danger appears when confined.

### B — Hydrogen

**Verb:** extremely light combustible gas.

It is valuable once gas density/escape behavior is sufficiently visible.

### B — Ammonia

**Verb:** toxic/reactive gas that can also bridge to fertilizer/biology or refrigeration later.

### B — Fog / Cloud

**Verb:** suspended droplets that condense/evaporate and reveal airflow.

This may be better represented as a phenomenon/mixture than a canonical pure Matter.

---

## 9. Biology and ecological material family

### A — Vine

**Verb:** grows along surfaces rather than simply upward/outward.

### A — Moss

**Verb:** colonizes damp Stone surfaces.

This is a perfect bridge between Water, Stone and Biology.

### A — Fungus / Mold

**Verb:** consumes dead organic Matter and spreads.

This provides a real-world propagation mechanic before inventing a transmutation virus.

### A — Algae

**Verb:** grows in Water and turns aquatic environments into living substrates.

Later Light/Oxygen systems can deepen it, but Water-based growth is already distinct.

### B — Tree

**Verb:** turns growth into Wood production.

Tree may initially be a mature Plant state rather than a separate Material ID.

### B — Bacteria

**Verb:** invisible/fast biological conversion.

Potentially powerful but difficult to read visually; use only when the interaction payoff is strong.

### B — Virus

**Verb:** infects suitable living Matter and converts its state.

The PG2-style universal `Virus transforms dots` mechanic is fun, but Powdergame should first try to express contagion through real biology, corrosion, crystallization and fungus. A fictional universal transmuter should only be added if a genuine interaction gap remains.

---

## 10. Absorption, foam and surface family

### A — Sponge

**Verb:** absorbs Water until saturated.

```text
Dry Sponge + Water → Wet/Saturated Sponge
Heat / squeezing mechanism → releases Water candidate
```

This is one of the clearest missing “ordinary sandbox toys.”

### A — Foam

**Verb:** traps Gas in a fragile liquid/solid network.

Foam adds low-density insulation, buoyancy and collapse behavior with one readable visual state.

### B — Wax

**Verb:** low-temperature reversible solid↔liquid phase change and waterproof coating.

Real wax may cover part of the role currently assigned to fictional Phase-Wax; the fictional version should survive only if its heat-storage behavior is intentionally stronger/distinct.

### B — Rubber

**Verb:** elastic solid with bounce/insulation behavior.

Add when elasticity is represented cheaply enough to matter.

---

## 11. Combustion, explosive and signal materials

### A — Fuse

**Role:** manufactured Matter / structure rather than raw natural material.

**Verb:** propagates Fire slowly and predictably.

It is valuable because it lets the player design **time** using ordinary combustion rather than a timer UI.

### A/B — Thermite-like reactive mixture

**Verb:** produces intense localized heat capable of melting metal.

Excellent once Iron/Metal has meaningful melting. Keep the recipe abstract and game-level.

### B — Charcoal

Potentially distinct from Coal through faster/lighter combustion and production from Wood, but initially a Coal-family variant is enough.

### B — Candle/Wax fuel

Useful only if slow, stable combustion has a role distinct from Wood/Coal/Oil.

---

## 12. Manufactured material family

### A — Brick

Already justified by `Clay + Water + Heat`.

### B — Cement

**Verb:** reacts with Water to become a binder.

### B — Concrete

**Verb:** a manufactured structural material produced from mineral ingredients rather than mined as a finished block.

`Limestone → Cement → Concrete` is a very good second manufacturing chain after Clay → Brick.

### B — Ceramic / Porcelain

**Verb:** fired mineral material with heat/chemical resistance but brittleness.

### B — Aerogel

Already strong as a real exotic: extraordinary insulation paired with fragility.

---

## 13. Real exotic materials that feel fictional

These are valuable because they add surprise without consuming the fictional-material budget.

### A — Methane Clathrate

```text
solid that looks ice-like
+ Heat
→ Methane release
→ confinement
→ ignition
→ Pressure
```

### A — Dry Ice

Sublimation makes it an immediate phase-change lesson.

### A — Perchlorate Dust

Useful as a delayed oxidizer / planetary-soil hazard abstraction.

### B — Ferrofluid

**Verb:** liquid responds visibly to magnetic fields.

Reserve for a future magnetism family.

### B — Non-Newtonian / shear-thickening slurry

**Verb:** flows slowly under gentle motion but stiffens under impact.

Excellent once collision/impact stress can be represented cheaply.

### B — Shape-memory alloy

**Verb:** Heat restores a trained material state.

Future candidate for state-memory mechanics before more magical memory Matter.

---

## 14. Historical / proto-scientific concepts

These should enrich discovery and naming without pretending they are modern chemistry.

### Classical elements

```text
Earth → Dirt / Stone / Clay / Mineral families
Water → Water / Ice / Steam / Brine
Air   → Gas / Oxygen / CO2 / Methane
Fire  → combustion phenomenon, not ordinary Matter
Aether → future exotic / philosophical category
```

### Alchemical principles

`Salt / Sulfur / Mercury` are especially valuable because they are simultaneously real Materials and historically symbolic substances.

### Phlogiston

Good encyclopedia/history concept and possible whimsical future world-rule, but not needed as an early Matter because combustion already has a physical grammar.

### Quintessence / Aether

Useful as a late exotic category label. Do not use it to justify arbitrary magic before a real interaction gap exists.

---

## 15. Proposed expanded real/common candidate pool

The following is the recommended **research pool**, not the runtime roster.

### Foundation / already-directional

```text
Boundary Block, Stone, Sand, Ice, Water, Steam, Smoke, Wood, Oil,
Acid, Seed, Plant, Salt, Lava, Metal, Glass
```

### Strong ordinary additions

```text
Dirt, Mud/state, Clay, Brick, Gravel, Ash, Snow,
Coal, Sulfur, Saltpeter, Gunpowder,
Basalt, Obsidian, Limestone/Calcite, Quartz/Crystal,
Iron, Copper, Lead, Mercury,
Alcohol, Brine, Nitroglycerin-like Nitro, Soapy Water, Foam, Sponge,
Oxygen, CO2, Methane,
Vine, Moss, Fungus, Algae,
Fuse
```

### Strong reserve

```text
Sawdust, Pumice, Dripstone, Amethyst,
Aluminum, Zinc, Gold, Tungsten, Steel,
Resin, Tar, Honey, Wax, Rubber,
Hydrogen, Ammonia, Fog/Cloud,
Tree, Bacteria, Virus,
Cement, Concrete, Ceramic,
Thermite-like mixture
```

### Real exotic reserve

```text
Dry Ice, Methane Clathrate, Perchlorate Dust, Aerogel,
Ferrofluid, shear-thickening slurry, shape-memory alloy,
Regolith, Ammonia Ice, Tholin-like organics
```

This produces a broad **80-ish-name research pool**, but many names are states, variants, structures or future-system candidates. The final runtime catalog should remain much smaller.

---

## 16. Highest-value gaps discovered

The comparison reveals eight gameplay verbs that are weak or absent in the current first roster:

```text
ABSORB      → Sponge
FOAM        → Soapy Water / Foam
SHOCK-DETONATE → Nitro
PROPAGATE-SLOWLY → Fuse
CORRODE / WEATHER → Iron / Copper / Rust
CRYSTAL-GROW → Quartz/Amethyst/Calcite
DECOMPOSE-ORGANIC → Fungus
SURFACE-GROW → Moss / Vine
```

These are higher-priority gaps than adding more fictional super-materials.

---

## 17. Recommended next shortlist change

Do **not** immediately replace the 38-entry first roster. Treat it as a baseline and test the following additions as the next review set:

### Near-certain additions

```text
Iron
Copper
Basalt
Limestone / Calcite
Quartz / Crystal
Sponge
Soapy Water / Foam
Vine
Moss
Fungus
Fuse
```

### Strong experiments

```text
Lead
Nitro
Snow
Ash
Algae
Dripstone
```

### Keep generic until the split proves useful

```text
Metal → keep as M0 placeholder, later decompose to Iron/Copper/Lead/Mercury
Stone → keep as foundation, later decompose by geological behavior
Plant → keep as foundation, later decompose to Vine/Moss/Fungus/Algae/Tree
Glass → keep generic until optical/thermal variants earn distinct verbs
```

If these additions survive interaction-graph testing, a mature early catalog will likely land closer to **45–55 truly distinct Matter identities**, not hundreds.

---

## 18. Selection test

Every candidate must answer all five questions:

1. **What is its one-sentence identity?**
2. **What does it do that current Matter cannot?**
3. **What are at least two other Materials/Fields it meaningfully interacts with?**
4. **Can the behavior be expressed using current or clearly justified local state?**
5. **If removed, does the sandbox lose an experiment rather than merely a color/name?**

A candidate that cannot answer these stays in the encyclopedia.

---

## 19. Design conclusion

The desired catalog is neither “every real substance” nor “a small pure-physics set.” It should feel like a **curated cabinet of phenomena**.

The strongest source order is:

```text
1. familiar real matter with strong behavior
2. real strange matter that feels almost fictional
3. historical/alchemical concepts that organize discovery
4. only then fictional Matter that fills an uncovered verb
```

This preserves familiarity while keeping surprise.

The guiding question for future material research should remain:

> **If I drop this next to something the player already knows, will something worth watching happen?**
