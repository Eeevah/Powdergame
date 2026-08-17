# Powdergame First Expansion Material Shortlist

## Status

- Type: `DERIVED`
- Authority: non-authoritative research
- This is **not** an M0 milestone change.
- Goal: identify the first Materials worth prototyping after the current M0 world grammar is proven.

Inputs include `MATERIAL_CANDIDATES.md`, the Original Matter research, the expanded capacity/fatigue research, and current Material/Reaction specs.

---

## 1. Separate three sets first

### A. Current M0 validated set

Do not expand here:

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

Fire/Combustion remains a phenomenon/state.

### B. Already-known initial catalog direction

These are **not new discoveries from MATERIAL_CANDIDATES.md**:

```text
Acid
Seed
Plant
Salt
Lava
Metal
Glass
```

They are catalog-direction Materials that may be implemented/validated later.

### C. Truly new shortlist

The following are additional candidates chosen for behavior value, not name count.

---

## 2. Tier 1 — strongest low-system-cost candidates

### 1. Gunpowder

```text
POWDER
+ sufficient Heat / combustion trigger
→ rapid reaction
→ Heat + Pressure
→ Smoke / residue
```

Why:

- visually obvious
- uses Powder movement
- converts Combustion into Pressure
- supports sealed-vs-open experiments
- explosion can remain existing Heat + Pressure semantics rather than a dedicated solver

Risk:

- do not implement as bespoke radial explosion code.

**Recommendation: PROTOTYPE FIRST.**

---

### 2. Dry Ice

```text
STATIC
+ Heat
→ CO2 GAS
```

Why:

- introduces sublimation without a new physics system
- clearly differs from Ice
- naturally pairs with CO2
- teaches that “ice” does not imply Water

**Recommendation: PROTOTYPE FIRST.**

---

### 3. CO2

```text
heavy GAS
+ combustion neighbor
→ combustion suppression / reduced continuation
```

Why:

- density ordering gives immediate visual identity
- creates a fire-extinguisher experiment
- pairs with Dry Ice
- no hidden air-composition model required

Risk:

- do not turn atmospheric simulation into a requirement.

**Recommendation: PROTOTYPE WITH DRY ICE.**

---

### 4. Methane

```text
light GAS
+ ignition
→ Heat + Pressure + combustion products
```

Why:

- gas accumulation matters
- sealed space matters
- Temperature, Combustion, Pressure and Gas movement connect
- familiar enough for strong player prediction

**Recommendation: PROTOTYPE FIRST.**

---

### 5. Clathrate

```text
STATIC volatile solid
+ Heat
→ Methane spawn/release
→ possible Pressure
→ possible combustion chain
```

Why:

- one of the highest Interaction Yield candidates
- looks inert until temperature changes
- turns phase/thermal work into a delayed gas/fire hazard
- strong space/ocean-world identity without exotic physics

**Recommendation: PROTOTYPE AFTER METHANE.**

---

### 6. Cryofluid

Prefer a generic/original coolant identity rather than a lore name.

```text
LIQUID
+ hot neighbor
→ remove Heat from neighbor / gain Heat
```

Then existing rules cause:

```text
Water → Ice
Lava → cooled solid
hot Metal → cool Metal
```

Why:

- does not hardcode outcomes
- lets target Materials own their transitions
- demonstrates the design principle “one simple relation creates many consequences”

Risk:

- avoid magical fixed-temperature reset.

**Recommendation: HIGH PRIORITY.**

---

### 7. Ablative Char

```text
STATIC sacrificial heat shield
+ extreme Heat
→ consumes/transitions itself
→ reduces heat reaching material behind
→ spent residue
```

Why:

- “strong because it dies” is better than an invincible wall
- works with Temperature and staged transition
- immediately useful around Lava / hot gas

**Recommendation: HIGH PRIORITY.**

---

### 8. Clay → Brick

```text
Clay POWDER
+ Water / wet condition
→ workable wet state
+ Heat
→ Brick STATIC
```

Why:

- extremely readable
- turns simulation into fabrication
- gives a strong “I made a material from the world” payoff
- supports later construction without a crafting UI

Risk:

- Mud and wet Clay must remain mechanically distinct enough or be merged.

**Recommendation: HIGH PRIORITY CONTENT CHAIN.**

---

### 9. Obsidian

```text
Lava
+ strong cooling / Water contact
→ Obsidian
```

Why:

- recognizable visual result
- rewards interaction rather than menu recipe
- gives Lava a second cooling outcome if differentiated from ordinary Stone

Decision condition:

Obsidian should exist only if its resulting behavior differs enough from Stone/Glass, e.g. brittle/high thermal shock.

**Recommendation: CONDITIONAL HIGH PRIORITY.**

---

### 10. Brine

```text
Water + Salt relation
→ Brine LIQUID
```

Potential visible effects:

- different freezing threshold
- different density rank
- future corrosion acceleration

Why:

- makes Salt a network material rather than white Sand
- joins Temperature, Density and later corrosion

Risk:

- do not add invisible salinity float to Water by default.

**Recommendation: STAGED-MATERIAL APPROACH.**

---

## 3. Tier 2 — good after the first chain works

### Dirt ↔ Mud

Strong basic-world vocabulary, especially once Plant exists.

Keep only if Mud's movement is recognizably different from Water and Dirt changes growth/substrate behavior.

### Snow

A Powder that thermally becomes Water is simple and readable. Good variant of phase grammar without a new field.

### Tar

Useful only if it has a distinct combination:

```text
very slow liquid
+ adhesion / movement inhibition
+ long smoky combustion
```

If it is merely “slower Oil”, merge it.

### Alcohol

Useful as a fast-igniting water-miscible fuel. Defer if mixture behavior would require hidden composition state.

### Resin → Amber

Strong slow transition / preservation chain, but becomes more valuable after biological matter exists.

### Regolith

Good theme, weak mechanics by itself. Keep only with a differentiator such as volatile release, abrasion, or processing chain.

### Perchlorate Dust

Mechanically stronger than plain Regolith:

```text
ordinary-looking POWDER
+ Heat
→ oxidizing gas/effect
→ nearby combustion becomes more intense
```

This is a good “hidden hazard becomes legible through experiment” candidate, but oxidizer semantics should be added as a family, not a one-off.

---

## 4. Registered catalog candidates that deserve early attention

### Lava

High value because it anchors:

- extreme Temperature
- cooling transition
- Steam generation around Water
- Stone/Obsidian production
- ignition
- later structural/thermal tests

### Metal / Molten Metal

High value because it anchors:

- strong thermal conduction
- Static ↔ Liquid phase behavior
- density contrast
- later corrosion/electricity/structure

### Glass

High value as:

- Sand transformation reward
- brittle/static material
- Acid-resistance contrast if chosen
- future optics substrate

### Acid

Implement as the **baseline corrosion grammar**, not as a universal delete fluid.

Prefer:

```text
target sees Acid
→ target transforms/corrodes
```

over Acid directly deleting arbitrary neighbors.

### Salt

High value because it can connect Water, Brine, Ice and later corrosion.

### Seed / Plant

High value only when generic slow-rule scheduling is ready. Avoid making Biology a one-off special pass.

---

## 5. Reference-only candidates that should be mined, not copied

| Raw reference cluster | Keep the mechanic | Do not keep by default |
|---|---|---|
| Xeno Blood / Thresher Maw Acid | penetrating/selective corrosion | franchise names; merely faster Acid |
| Vespene / Tibanna | combustible gas / propellant | separate gases differing only by color |
| Naquadah / Naquadria | heat storage vs unstable discharge | franchise identity |
| Beskar / Phrik / Adamantium / Vibranium | specialized hazard resistance | five “super metals” |
| Tiberium / Creep / Sculk / Protomolecule | self-propagating substrate conversion | four separate infection systems |
| Ghost Matter / Red Matter / Void Fluid | unusual sink/absence mechanic | universal deletion that bypasses sandbox |
| Quantum / Timeshift references | rule/history manipulation | observation-based or history-heavy M0 logic |

The source is valuable as **mechanic archaeology**, not as a licensing/content list.

---

## 6. First prototype bundles

Do not prototype isolated Materials where a small bundle proves a chain better.

### Bundle A — volatile atmosphere

```text
Dry Ice
→ CO2
→ heavy gas layer
→ fire suppression
```

Proof:

- solid→gas transition
- gas density
- combustion interaction

### Bundle B — trapped fuel

```text
Clathrate
→ heat
→ Methane
→ sealed accumulation
→ ignition
→ Heat + Pressure
→ rupture/vent
```

Proof:

- phase/transition
- gas spawn
- combustion
- pressure
- structural consequence

This is the strongest candidate bundle in the current research.

### Bundle C — world fabrication

```text
Clay
+ Water / wet state
→ wet clay
+ Heat
→ Brick
```

and:

```text
Sand
+ extreme Heat
→ Glass
```

Proof:

- player makes structures through world rules, not recipe UI.

### Bundle D — thermal engineering

```text
Lava / hot Metal
+ Cryofluid
→ target follows its own cooling transition
```

plus:

```text
Ablative Char
→ sacrifices itself
→ protects rear structure
```

Proof:

- thermal properties create engineering, not just damage.

---

## 7. Explicit defer list

Defer even if visually exciting:

- observation-dependent Quantum Shard
- Timeshift / previous-state restoration
- universal Void Fluid / Red Matter delete
- generic gravity reversal Eezo/Unobtanium
- direct teleport materials
- full Light / Radiation / Electricity candidates before those families justify a subsystem
- one-off “cannot be damaged” ultimate metals
- lore-only treasure materials
- palette swaps

These are not rejected forever. They are poor **first expansion** choices because they either bypass existing causal chains or require a new reusable engine substrate.

---

## 8. Recommended first expansion order

This is research priority, not roadmap authority.

```text
1  existing registered: Lava
2  existing registered: Metal / Molten Metal
3  existing registered: Glass
4  existing registered: Salt
5  Dry Ice
6  CO2
7  Methane
8  Clathrate
9  Gunpowder
10 Cryofluid
11 Ablative Char
12 Clay / Brick
13 Brine
14 Obsidian
15 Dirt / Mud
16 Snow
17 Perchlorate Dust
18 Tar
```

The order deliberately front-loads Materials that make multiple existing systems talk to each other.

The goal is not “18 more elements”.

The goal is to make chains such as:

```text
Heat
→ volatile release
→ gas density
→ combustion
→ pressure
→ rupture
→ cooling
→ residue
→ construction
```

feel like one coherent world.
