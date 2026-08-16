# Powdergame Block Palette & Powder Game 2 Gap Review

## Status

- Type: `DERIVED / REFERENCE_REVIEW`
- Authority: non-authoritative research
- Purpose: identify interaction gaps in the current first roster by studying Minecraft's block-material differentiation and Powder Game 2's element vocabulary.
- This document does **not** expand M0, register runtime Materials, or copy source-game content wholesale.

> **같은 Stone, 같은 Metal, 같은 Plant라는 상위 이름 아래에서도 플레이어가 기억할 동사가 다르면 별도 Matter 후보가 될 수 있다.**

---

## 1. Why this review was needed

The first interaction roster was intentionally compressed around causal chains. That exposed a new weakness:

```text
Metal = one generic identity
Stone = one generic identity
Plant = one generic identity
```

This is too coarse for a world-building sandbox once M0 is proven.

The correction is **not** to copy dozens of decorative variants. Instead, borrow the stronger lesson from Minecraft and Powder Game 2:

```text
visual/material family
+ one memorable behavior verb
= distinct gameplay identity
```

A new block/material candidate should earn its place through behavior, not color alone.

---

## 2. Minecraft lesson — material palette, not decoration palette

Minecraft's useful lesson is not that it has many stone textures. Its stronger blocks often differ by a rule the player remembers.

Reference behaviors worth translating into Powdergame grammar include:

- Copper ages through oxidation stages.
- Budding Amethyst grows crystals from exposed faces.
- Powder Snow traps/freeze-interacts instead of behaving like ordinary snow.
- Pointed Dripstone transports/drips liquids and can grow mineral structures.
- Sponge absorbs Water until saturated.
- Honey is sticky and changes motion.
- Sculk spreads by consuming/converting nearby substrate.

Powdergame should extract the verbs:

```text
OXIDIZE
GROW_CRYSTAL
TRAP / INSULATE / FREEZE
DRIP / DEPOSIT
ABSORB / SATURATE
ADHERE / SLOW
CONSUME / PROPAGATE
```

not the Minecraft names or exact implementation.

---

## 3. Metal family — generic Metal should become a foundation placeholder

Current `Metal` is useful for M0 because it proves a movement/thermal/structural class without multiplying content.

After the core is stable, the broader catalog should consider decomposing it into a small number of metals with **clearly different sandbox roles**.

### Iron — HIGH PRIORITY

**Dictionary:** the ordinary structural metal that visibly loses itself to rust.

Distinct verbs:

```text
STRUCTURE
CONDUCT_HEAT
OXIDIZE / RUST
MELT / CAST
```

Interaction value:

- Water + oxygen/moisture conditions -> Rust candidate
- Brine accelerates corrosion
- high Heat -> Molten Iron/Metal stage
- later magnetism can attach naturally without redesigning its identity

Iron should eventually become the default concrete meaning behind today's generic `Metal`.

### Copper — HIGH PRIORITY

**Dictionary:** a conductor that records time on its own surface.

Distinct verbs:

```text
CONDUCT_HEAT strongly
OXIDIZE in stages
later: CONDUCT_ELECTRICITY
```

Why it earns a separate identity:

- oxidation is visible world history rather than a stat difference;
- it creates contrast with Iron corrosion;
- later Electricity reuses the same Matter instead of adding a new conductor.

### Lead — MEDIUM-HIGH

**Dictionary:** a surprisingly soft metal that is extremely heavy and melts comparatively easily.

Distinct verbs:

```text
SINK / BALLAST
LOWER-MELT METAL
later: SHIELD radiation
```

Useful because Density and Heat already make it feel different before Radiation exists.

### Aluminum — MEDIUM

**Dictionary:** metal without the expected weight.

Distinct verbs:

```text
LIGHT STRUCTURE
GOOD THERMAL CONDUCTION
MELT earlier than refractory metals
```

Keep only if low density meaningfully changes construction/transport gameplay; otherwise reserve.

### Gold — MEDIUM

**Dictionary:** very dense, soft, and reluctant to corrode.

Distinct verbs:

```text
DENSE
CORROSION-RESISTANT
later: GOOD ELECTRICAL CONDUCTOR
```

Cultural recognizability is strong, but gameplay must be more than value/rarity.

### Tungsten / Titanium — RESERVE

Both are iconic engineering metals, but `stronger/hotter Metal` alone is not enough.

Adopt only if a future high-temperature or structural family makes their difference experiential.

### Mercury — KEEP

Already justified because `LIQUID METAL + extreme density` is immediately unique.

### Recommended metal palette

First decomposition target:

```text
Metal (temporary foundation abstraction)
→ Iron
→ Copper
→ Lead
→ Mercury

Reserve:
Aluminum / Gold / Tungsten / Titanium
```

This is enough diversity without becoming a periodic table simulator.

---

## 4. Stone / mineral family — generic Stone needs several verbs

`Stone` should remain the foundation inert structural baseline. Other rocks/minerals should exist only where they produce a different interaction.

### Basalt — HIGH PRIORITY

**Dictionary:** Lava that has already chosen to become rock.

Distinct verbs:

```text
LAVA COOLING RESULT
HEAT-RESISTANT STRUCTURE
possible thermal fracture
```

It gives ordinary slow-cooled Lava a result distinct from rapid-cooled Obsidian.

### Obsidian — KEEP

**Dictionary:** rapidly cooled volcanic glass.

Distinct verbs:

```text
RAPID-COOL RESULT
GLASS-LIKE / BRITTLE STRUCTURE
```

This makes `Lava cooling rate` meaningful rather than cosmetic.

### Limestone / Calcite — HIGH PRIORITY

**Dictionary:** stone that Water can slowly build and Acid can quickly erase.

Distinct verbs:

```text
DISSOLVE in Acid / acidic conditions
PRECIPITATE / DEPOSIT from mineral Water
PROCESS → Lime/Cement family
```

Excellent bridge between geology, chemistry and construction.

### Sandstone — MEDIUM-HIGH

**Dictionary:** Sand that stopped flowing because pressure/mineral binding made it remember a shape.

Distinct verbs:

```text
GRANULAR → STRUCTURAL
ERODE / CRUMBLE
```

Useful if Powdergame wants sedimentation/compaction; otherwise Brick/Concrete already cover the manufacturing verb.

### Granite — MEDIUM

Use as a durable inert/high-thermal-mass stone only if it creates visible contrast with ordinary Stone. Do not include just for geology variety.

### Dripstone / Travertine-like mineral — HIGH INTEREST, LATER

**Dictionary:** Water carries invisible stone; dripping leaves the stone behind.

Distinct chain:

```text
mineral Water / dissolved mineral
→ slow drip
→ deposit
→ stalactite / stalagmite growth
```

This is an excellent Doodle-God-like emergent construction process, but it needs slow growth/deposition rules.

### Amethyst / Growing Crystal — HIGH INTEREST

**Dictionary:** rock that grows a crystal crop instead of a plant.

Distinct verb:

```text
NUCLEATE
GROW_CRYSTAL from exposed surface
HARVEST / REGROW
```

The important mechanic is crystal growth, not purple decoration.

### Recommended stone/mineral palette

```text
Stone                baseline
Basalt               slow volcanic cooling
Obsidian             rapid volcanic cooling / glass-like
Limestone/Calcite    dissolution + deposition + cement chain
Amethyst/Crystal     mineral growth

Reserve:
Sandstone / Granite / Dripstone
```

---

## 5. Glass should become a small family too

Current Glass is already a good catalog direction, but it can later branch by behavior.

### Ordinary Glass — KEEP

Transparent, brittle, high-temperature Sand-derived structure.

### Tempered / Reinforced Glass — REAL RESERVE

Do not add as merely `Glass with more HP`. It needs a fracture pattern or heat-shock distinction to justify itself.

### Tinted / Light-Blocking Glass — FUTURE

Useful once Light is gameplay-relevant. Before then it is mostly visual.

### Crystal Glass / Quartz — RESERVE

Could earn a role through thermal, optical or piezoelectric behavior later.

---

## 6. Plant family — one generic Plant is too narrow

`Seed` and `Plant` are excellent M0/early abstractions, but later biology should branch by **growth topology and ecological interaction**, not species count.

### Grass / Ground Cover — MEDIUM-HIGH

```text
spread across suitable Dirt surface
burn quickly
regrow after Water
```

### Vine — HIGH PRIORITY

```text
seek/support solid surfaces
climb / spread along walls
burn
```

Different growth topology from ordinary Plant makes it worthwhile.

### Moss — HIGH PRIORITY

```text
spread over wet/shaded Stone
retain moisture
slowly change substrate appearance/state
```

Excellent connector between Water + Stone + Biology.

### Tree / Wood-producing Plant — HIGH PRIORITY

A staged organism can connect:

```text
Seed → Plant → Tree/Wood
Water + Soil + time
Fire → Charcoal/Ash/Smoke
```

Whether Tree is a separate Agent or staged Plant should be decided later.

### Mushroom / Fungus — HIGH PRIORITY LATER

```text
consume organic residue
prefer moisture/darkness
spread spores
convert dead biomass
```

This adds decomposition, something generic Plant does not provide.

### Algae — MEDIUM-HIGH LATER

```text
grow in Water
respond to nutrients/light
later produce/consume gases
```

Good aquatic ecology bridge.

### Recommended plant ecology palette

```text
Seed / Plant          foundation abstraction
Vine                  surface/climbing growth
Moss                  wet mineral colonizer
Tree/Wood stage       biomass/structure/fuel bridge
Mushroom/Fungus       decomposition
Algae                 aquatic growth
```

Do not add dozens of flower/tree species unless they create new interaction verbs.

---

## 7. Powder Game 2 — what is actually worth stealing as design grammar

Dan-Ball's current Powder Game 2 element list contains a compact set of elements with very strong verbs: Gunpowder explodes, Mud is wet Sand, Stone is breakable, Virus transforms dots, Nitro is a sensitive explosive, Soapy Water creates bubbles, Metal conducts electricity, Vine grows around matter, Mercury is a heavy liquid metal, Fuse burns gradually, Cloud creates weather variants, Pump moves liquids/gases, and Conveyor moves touched objects.

The useful lesson is **one element = one toy-like verb**.

### Powder Game 2 ideas already covered well

- Gunpowder -> current shortlist
- Mud -> Dirt/Water staged result candidate
- Lava -> catalog direction
- Flammable Gas -> Methane
- Acid -> catalog direction
- Salt / seawater -> Salt/Brine
- Mercury -> shortlist
- Vine -> should be promoted in plant family

### Nitro / Nitroglycerin — STRONG REAL CANDIDATE

**Gap:** Gunpowder is ignition-sensitive; Nitroglycerin can represent **shock-sensitive explosive liquid**.

```text
LIQUID
+ impact / pressure shock
→ rapid Heat + Pressure
```

This is a genuinely different explosive verb, not just `bigger Gunpowder`.

Recommendation: strong reserve, especially after impact/stress semantics exist.

### Soapy Water / Foam — STRONG REAL CANDIDATE

**Gap:** current liquids do not strongly express surface/foam behavior.

Possible Powdergame translation:

```text
Soap + Water → Soapy Water
Soapy Water + Gas/agitation → Foam/Bubbles
Foam → traps Gas / slows movement / collapses with Heat or Oil
```

This can create a whole low-cost `foam / bubble / surfactant` family.

Recommendation: HIGH.

### Fuse — STRONG PRODUCT / STRUCTURE CANDIDATE

Not really a fundamental Matter family, but extremely useful as a **slow propagation line**.

```text
ignition
→ gradual burn along connected Fuse
→ delayed trigger
```

This is perfect for player-built causal machines. Treat as manufactured product/structure rather than natural Matter.

### Cloud — FUTURE PHENOMENON / GAS-DROPLET FAMILY

PG2's rain/snow/thunder-cloud idea suggests a useful future weather abstraction, but it should emerge from Water droplets/temperature/electricity rather than be a magical single-purpose dot if possible.

### Virus — VERY IMPORTANT MECHANIC REFERENCE

PG2 Virus is interesting because it is not merely destructive: it **acquires/transmits a transformation identity** and can convert large regions.

Do **not** copy arbitrary `touch anything -> turn it into X` as a real Virus.

Extract the family:

```text
CONTAGIOUS STATE
→ propagate locally
→ transform eligible substrate
→ inherit or carry a payload/state
```

Reality-first substitutes should be considered first:

- biological Virus -> infect living Agents/cells only
- Fungus / Mold -> consume organic Matter and propagate
- Rust / corrosion front -> transforms Metal
- Crystal seeding -> transforms supersaturated solution into Crystal

Only if these fail to create the desired cross-material contagion toy should Powdergame invent an original `transmuting contamination` Matter.

### Ant / Bird / Fish — AGENT REFERENCES

These demonstrate that emergent flocking/pathing can be fun, but they belong to a later Agent layer rather than ordinary Matter.

### Pump / Conveyor — DEVICE REFERENCES

Excellent sandbox tools, but they should be built from structures/forces rather than disguised as natural Matter where possible.

### Clone — META TOOL, NOT WORLD MATTER

Useful editor/sandbox tool, dangerous as ordinary content because it bypasses production and conservation loops.

---

## 8. New gaps exposed by this pass

The current roster is missing several **material families**, not merely individual names.

### Gap A — Metal diversity

Needed verbs:

```text
RUST / AGE
HIGH CONDUCTION
DENSE / LOW-MELT
LIQUID METAL
```

Candidates: Iron, Copper, Lead, Mercury.

### Gap B — Geological transformation

Needed verbs:

```text
SLOW COOL
RAPID COOL
DISSOLVE
DEPOSIT
CRYSTAL GROWTH
```

Candidates: Basalt, Obsidian, Limestone/Calcite, Amethyst/Crystal.

### Gap C — Adhesion / absorption / foam

Needed verbs:

```text
STICK
ABSORB / SATURATE
FOAM / TRAP GAS
```

Candidates: Resin/Honey-like sticky Matter, Sponge, Soapy Water/Foam.

### Gap D — Ecology topology

Needed verbs:

```text
CLIMB
COLONIZE WET SURFACE
DECOMPOSE
AQUATIC GROWTH
```

Candidates: Vine, Moss, Fungus, Algae.

### Gap E — Contagious transformation

Needed verbs:

```text
INFECT
PROPAGATE
CONVERT ELIGIBLE SUBSTRATE
```

First try real biological/ecological/corrosive/crystal mechanisms. Original transmuting Virus only if a gameplay gap remains.

### Gap F — Slow signal / delayed causality

Needed verb:

```text
PROPAGATE TRIGGER SLOWLY
```

Candidate: Fuse as manufactured Matter/product.

---

## 9. Recommended roster correction

Do **not** simply grow the 38 roster to 70.

Instead change the mental model from a flat roster to **families with representatives**.

### Foundation placeholders

```text
Stone
Metal
Plant
```

stay useful in M0.

### First decompositions after foundation proof

```text
METAL
  Iron
  Copper
  Lead
  Mercury

STONE / MINERAL
  Stone
  Basalt
  Obsidian
  Limestone/Calcite
  Amethyst/Crystal

PLANT / ECOLOGY
  Plant
  Vine
  Moss
  Fungus
  Algae

SPECIAL REAL INTERACTION
  Sponge
  Soapy Water/Foam
  Nitroglycerin
  Fuse
```

Not all must ship together. The point is that these are stronger next candidates than adding many more fictional exotic Matters.

---

## 10. Selection rule updated by this review

The previous rule remains valid but gains one more question:

```text
Is this a new material name?
    ↓
Does it create a new interaction verb?
    ↓ yes
Does an existing real material already express that verb well?
    ↓ yes
Prefer the real material.
    ↓ no
Can a historical/natural-philosophy archetype express it?
    ↓ no
Only then invent original Matter.
```

And for block/material palette specifically:

> **Minecraft에서 배울 것은 블록 수가 아니라, 같은 ‘돌·금속·식물’ 안에서도 기억할 만한 행동을 하나씩 주는 방법이다. Powder Game 2에서 배울 것은 한 요소를 하나의 장난감 같은 동사로 만드는 방법이다.**
