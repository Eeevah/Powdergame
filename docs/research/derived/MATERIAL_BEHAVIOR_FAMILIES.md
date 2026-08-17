# Powdergame Material Behavior Families

## Status

- Type: `DERIVED`
- Authority: non-authoritative research
- Inputs:
  - `docs/research/raw/MATERIAL_CANDIDATES.md`
  - 2026-08-16 Original Matter research intake
  - 2026-08-16 expanded fictional matter intake
  - current `MATERIAL_SPEC.md` / `REACTION_SPEC.md`
- Purpose: compress candidate names into reusable local behavior grammar before selecting actual Materials.

This document does **not** add Materials to the registry and does not change M0 Evidence Gates.

---

## 1. Compression rule

Raw candidate count is not content value.

A candidate earns a distinct Material identity only when it creates a meaningfully different player prediction or interaction chain.

```text
many source names
    ↓
behavior family
    ↓
distinct local verb
    ↓
Powdergame representative Material
    ↓
counter / failure / byproduct
```

Examples:

```text
Vespene / Tibanna / Methane
→ combustible gas family

Xeno Blood / Thresher Maw Acid / Alkahest
→ corrosive / dissolving liquid family

Beskar / Phrik / Adamantium / Vibranium / Trinium
→ extreme structural material family
```

A palette, lore origin, or stronger number is not enough to justify a new Material.

---

## 2. Evaluation axes

Every family is evaluated by:

- **3-second readability** — can the player predict the first consequence?
- **Interaction Yield** — how many existing systems can it connect?
- **System Reuse** — can it run on Movement / Density / Temperature / Pressure / Combustion / Phase Transition?
- **Locality** — can it be expressed with local rules and existing fields?
- **State cost** — does it require new per-cell persistent state?
- **Field cost** — does it require a reusable new global field?
- **Duplicate risk** — is this just another skin for an existing verb?
- **Counterability** — is there a readable way to stop or reverse it?

Classification:

- `FOUNDATION-COMPATIBLE` — expressible with M0 world grammar; not automatically added to M0.
- `NEAR-TERM` — needs small rule/state support but no new world-scale field.
- `FUTURE-FAMILY` — only worth adding with a reusable future subsystem.
- `META/DEFER` — rule-changing, camera/history dependent, or destructive enough to bypass the sandbox.

---

## 3. The 24 behavior families

| ID | Family | Core local verb | Raw examples | Dependency | Decision |
|---|---|---|---|---|---|
| F01 | Granular / Moisture | wet, dry, compact, crumble | Dirt, Mud, Clay, Snow, Regolith | Movement + Temperature + neighbor rule | FOUNDATION-COMPATIBLE |
| F02 | Phase / Volatile | melt, freeze, sublime, condense | Dry Ice, Ammonia Ice, Carbonite, Comet Slush | Temperature + transition | FOUNDATION-COMPATIBLE |
| F03 | Combustible Fuel | ignite, burn, leave smoke/ash | Alcohol, Tar, Napalm, Hydrocarbon Lake | Combustion + Temperature | FOUNDATION-COMPATIBLE |
| F04 | Oxidizer / Suppressant | accelerate or suppress combustion | Oxygen, CO2, Perchlorate Dust, Phlogiston | Combustion neighbor rule | NEAR-TERM |
| F05 | Explosive / Pressure Source | convert heat/reaction into pressure | Gunpowder, Methane, Clathrate, Naquadria | Temperature + Combustion + Pressure | FOUNDATION-COMPATIBLE |
| F06 | Corrosion / Dissolution | consume or transform contacted material | Acid, Sulfuric Cloud, Xeno Blood, Alkahest | reaction rules + tags | FOUNDATION-COMPATIBLE |
| F07 | Viscosity / Adhesion | slow, stick, coat, trap | Honey, Slime, Tar, Resin, Hive Resin | Movement property + transition | NEAR-TERM |
| F08 | Density / Immiscibility | layer, sink, float, separate | Mercury, Brine, Hydrocarbon Lake, Blubber | Density Rank + movement | FOUNDATION-COMPATIBLE |
| F09 | Curing / Construction | powder/liquid becomes structure | Clay→Brick, Resin→Amber, Concrete, Hive Resin | transition + spawn/replace | FOUNDATION-COMPATIBLE |
| F10 | Thermal Control | conduct, insulate, absorb, cool | Metal, Cryofluid, Ablative Char, Minovsky Haze | Temperature + material thermal properties | FOUNDATION-COMPATIBLE |
| F11 | Gas Release / Capture | release stored gas or condense it | Clathrate, Ilmenite, Dry Ice, Vapor-Latch-like concepts | transition + spawn request | FOUNDATION-COMPATIBLE |
| F12 | Biological Growth | grow into neighboring valid substrate | Seed, Plant, Vine, Moss, Mushroom | slow local rule | NEAR-TERM |
| F13 | Biological Consume / Repair | consume target, heal or convert residue | Vex Milk, Necro Tissue, Omni-gel, Medigel | slow local rule + tags | FUTURE-FAMILY |
| F14 | Propagation / Contamination | spread while rewriting environment | Tiberium, Creep, Sculk, Protomolecule Goo | slow scheduler + conversion rules | FUTURE-FAMILY |
| F15 | Energy Storage / Discharge | accumulate heat/energy then release | Pyrostor-like mechanic, Naquadah, Ion Cube | persistent state or staged Material | NEAR-TERM |
| F16 | Impulse / Directed Movement | push, deflect, launch, bias motion | Jet Exhaust, Breeze Rod, Vector-Glass-like mechanic | movement claims + pressure | NEAR-TERM |
| F17 | Extreme Structure | resist selected hazards | Kyanite, Ceramite, Trinium, Beskar/Phrik references | registry properties | NEAR-TERM |
| F18 | Optical / Emission | glow, filter, refract, transmit | Kyber reference, Glowstone, Prismatrix-like concepts | future Light or presentation-only subset | FUTURE-FAMILY |
| F19 | Toxic / Radiation Hazard | damage/alter biological or material targets | Poison Gas, Radon, Kryptonite reference, Miasma | Biology and/or Radiation | FUTURE-FAMILY |
| F20 | Electrical / Signal | conduct, store, route, switch | Redstone reference, Electrum, Logic-Ferrite family | future Electricity/Information | FUTURE-FAMILY |
| F21 | Gravity / Weight Modifier | locally change movement weight/bias | Eezo, Unobtanium, Heavy Core | reusable gravity/force system | FUTURE-FAMILY |
| F22 | Spatial / Teleport / Sink | relocate or delete matter | Xen Crystal, Ender reference, Void Fluid, Red Matter | spatial ownership beyond local movement | META/DEFER |
| F23 | Time / History / Rule Rewrite | restore old state, delay world rules | Timeshift Glow, Chrono family, Warp Bleed | history/tick-mask/meta state | META/DEFER |
| F24 | Resource / Decorative / Lore Skin | reward, color, provenance, collectible | Helium-3 treasure, Phozon, many named alloys | presentation/economy | MERGE OR DEFER |

---

## 4. High-confidence family convergence across research

Several families recur independently in the research sources. These deserve more weight than a single cool name.

### Heat storage / delayed release

References converge on:

- Pyrostor
- Naquadah-like heat absorption and burst
- Phase-Wax / thermal buffer concepts
- expanded `capacity` / `fatigue` model

The useful mechanic is not any source name. It is:

```text
heat input
→ staged accumulation
→ threshold
→ release / failure
→ residue
```

This is a strong original-Matter seed because it connects Temperature, Pressure, Combustion and construction safety.

### Gas trapped in a solid

References converge on:

- Clathrate
- Vapor-Latch-like gas storage
- Bubble-Clathrate
- comet / volatile ice concepts

Useful mechanic:

```text
stable solid
+ heat or pressure change
→ gas release
→ expansion / pressure
→ optional ignition
```

This has extremely high Interaction Yield with existing world grammar.

### Corrosive liquid family

References converge on:

- Acid
- Xeno Blood reference
- Thresher Maw Acid reference
- Alkahest
- mineral-acid research

These should **not** become four independent “stronger acid” Materials.

Better structure:

```text
Acid = baseline corrosive liquid
future specialized corrosive = different target/selectivity/byproduct
```

A stronger numeric corrosion rate alone is not a new identity.

### Combustible gas family

References converge on:

- Methane
- Vespene reference
- Tibanna reference
- Pyretic-Gas family

Again, source names should collapse into behavior grammar.

Distinct Materials are justified only if the secondary behavior changes, e.g.:

- methane-like gas: ignition + pressure
- condensable fuel gas: temperature-dependent phase
- pyretic gas: unusual phase/fuel chain

### Self-propagating matter

References converge on:

- Creep
- Tiberium
- Sculk
- Protomolecule
- Litho-Mycelium
- multiple biological/contamination OM candidates

The core reusable family is:

```text
valid substrate nearby
+ local resource/condition
→ self propagation
→ substrate transformed
→ counter condition stops spread
```

This is powerful but should wait for a generic slow-rule substrate instead of one-off infection code.

---

## 5. IP/reference normalization rule

Direct names from identifiable creative works are `REFERENCE_ONLY`.

Do not ship a Material just because the source has a memorable name.

Normalize in three steps:

```text
source reference
→ mechanic sentence with all lore removed
→ compare against existing family
→ merge or create original Matter candidate
```

Examples:

| Reference | Lore-free mechanic | Powdergame destination |
|---|---|---|
| Xeno Blood | downward-penetrating corrosive liquid | F06; likely Acid variant, not a separate baseline Material |
| Vespene / Tibanna | combustible pressurizable gas | F03/F05; merge around gas-fuel grammar |
| Tiberium | self-growing crystal that degrades substrate | F14; future original propagation Matter |
| Naquadah | thermal/energy accumulator with dangerous discharge | F15; merge with Pyrostor mechanic research |
| Beskar / Phrik | high-heat-resistant structure | F17; one original advanced structure is enough |
| Ghost Matter | contact temporarily invalidates ordinary matter behavior | F22/F23; defer |
| Quantum Shard | movement depends on observation/availability state | F23; defer |
| Timeshift Glow | restores previous Material state | F23; defer |
| Red Matter / Void Fluid | universal sink/delete | F22; defer because it bypasses interaction chains |

---

## 6. What not to turn into separate Materials

### Palette-only variants

Examples in raw notes already identify several:

- Covenant Purple Metal
- Turian Metal
- Asari Athame Glass
- Goa'uld Gold

These belong to presentation, recipe provenance, or skin data unless a distinct local rule exists.

### Item or structure concepts

The raw list correctly marks examples such as:

- Stargate Naquadah Gate
- Ominous Bottle
- Trial Key
- Respawn Anchor Core

A structure or inventory item is not automatically Matter.

### “Final material” by numbers only

A material that is only:

```text
harder
more heat resistant
more expensive
```

does not deserve a new Material identity.

Advanced structures need a **different relationship**, not merely larger constants.

---

## 7. State-cost rule

The expanded research proposes `capacity8`, `fatigue8`, `purity8`, etc.

These are useful **design concepts**, but not universal Cell fields.

Before adding any persistent state:

1. Can the behavior be represented as a staged Material transition?
2. Can an existing `flags` bit represent the gameplay-visible state?
3. Can the value live in a subsystem-specific sparse/packed representation?
4. Is continuous accumulation actually visible to the player?
5. Does the same state unlock a family of Materials?

Example:

```text
Ablative Char
fresh → damaged → spent
```

may be better than adding universal `fatigue8`.

Likewise:

```text
Heat Battery
uncharged → warm → charged → unstable
```

may be sufficient before a numeric capacity field is justified.

---

## 8. Selection rule going forward

A new representative Material should ideally satisfy at least four:

- readable within ~3 seconds
- uses at least two existing systems
- creates a chain rather than a terminal effect
- has a natural counter
- leaves a meaningful residue/byproduct
- is not an IP skin
- does not need a unique engine pass
- does not add universal hidden state
- produces a visually distinctive world state

The preferred question remains:

> If this Material is placed next to something familiar, can the player immediately form a hypothesis — and can the world answer that hypothesis through existing rules?
