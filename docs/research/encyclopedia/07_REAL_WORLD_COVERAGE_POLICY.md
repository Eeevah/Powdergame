# Volume 07 — Real-World Material Coverage Policy

## Status

- Type: `DERIVED` encyclopedia policy
- Authority: non-authoritative research
- Goal: make the Powdergame encyclopedia broad enough that a player can reasonably expect familiar real-world materials to exist somewhere in the corpus, without turning the runtime into a chemistry database.

> **사전은 넓게, 런타임은 필요한 만큼 구체적으로.**

## 1. What “include real materials broadly” means

Literal coverage of every real chemical compound, mineral species, alloy grade, polymer formulation, isotope and commercial mixture is neither finite nor useful for a game.

Powdergame instead aims to cover **player-meaningful real-world material archetypes**.

A real substance deserves its own encyclopedia entry when at least one of the following is true:

- a player is likely to recognize or search for it by name;
- it creates a visibly different local behavior;
- it participates in a useful reaction/production chain;
- it anchors a family of fictional/original Materials;
- it is culturally or scientifically iconic enough to inspire experiments.

An encyclopedia entry does **not** automatically imply a unique runtime Material ID.

```text
real-world name
→ encyclopedia entry
→ behavior-family comparison
→ one of:
   unique Material
   alias / variant
   staged transition
   recipe/result only
   reference-only
```

## 2. Coverage families

### A. Foundational environment

High-priority broad coverage:

- Air / atmospheric gas mixture
- Oxygen
- Nitrogen
- Carbon dioxide
- Water / Ice / Steam
- Snow
- Fog
- Soil / Dirt
- Mud
- Clay
- Sand
- Gravel
- Stone
- Basalt
- Granite
- Limestone
- Quartz
- Obsidian
- Volcanic ash
- Regolith

These are world-building vocabulary rather than niche chemistry.

### B. Metals and ores

Candidate encyclopedia coverage should include at least:

- Iron
- Steel
- Copper
- Aluminum
- Gold
- Silver
- Lead
- Tin
- Zinc
- Nickel
- Titanium
- Magnesium
- Mercury
- Tungsten
- Chromium
- Cobalt
- Uranium
- Plutonium
- Iron ore
- Bauxite
- Hematite
- Magnetite
- Ilmenite

Runtime individuality depends on visible traits such as melting behavior, thermal conductivity, corrosion, density order, magnetism, radiation or structural strength.

### C. Minerals, salts and ceramics

- Salt / sodium chloride archetype
- Brine
- Saltpeter / nitrate salt
- Sulfur
- Gypsum
- Chalk
- Silica
- Alumina
- Ceramic
- Porcelain
- Brick
- Cement
- Concrete
- Glass
- Fiberglass
- Aerogel

These are especially valuable because they naturally create `raw material → process → manufactured material` chains.

### D. Carbon and fuels

- Carbon / soot
- Coal
- Charcoal
- Coke
- Graphite
- Diamond
- Wood
- Sawdust
- Paper
- Peat
- Oil / petroleum archetype
- Kerosene-like fuel
- Gasoline-like fuel
- Diesel-like fuel
- Tar / bitumen
- Asphalt
- Alcohol / ethanol archetype
- Methane
- Propane-like fuel
- Hydrogen
- Gunpowder

The game should prefer recognizable behavior differences rather than pretending to reproduce refinery chemistry exactly.

### E. Common chemicals

Broad encyclopedia candidates:

- Acid archetype
- Sulfuric acid
- Hydrochloric acid
- Nitric acid
- Acetic acid / vinegar
- Alkali / base archetype
- Sodium hydroxide
- Ammonia
- Hydrogen peroxide
- Chlorine
- Bleach-like oxidizer
- Carbon monoxide
- Sulfur dioxide
- Methane
- Oxygen
- Hydrogen

Exact concentration chemistry is optional. A single `Acid` Material can remain the early gameplay archetype while named acids exist in the encyclopedia as future variants.

### F. Polymers, gels and soft materials

- Rubber
- Latex
- Resin
- Epoxy-like resin
- Plastic
- Polyethylene-like plastic
- Foam
- Gel
- Silicone
- Wax
- Paraffin
- Soap
- Grease
- Lubricating oil
- Adhesive

This family is valuable for viscosity, adhesion, curing, insulation and combustion gameplay.

### G. Biological materials

Material-level candidates:

- Blood
- Bone
- Fat
- Flesh / tissue archetype
- Chitin
- Shell
- Keratin
- Leather
- Wool
- Cotton
- Cellulose
- Starch
- Sugar
- Sap
- Resin
- Honey
- Milk
- Egg
- Seed
- Plant matter
- Algae
- Fungal biomass

Living organisms themselves belong to the future `Agent/Biology` layer rather than being forced into ordinary Matter.

### H. Geological / extreme / space materials

- Lava / magma archetype
- Dry Ice
- Ammonia Ice
- Methane clathrate
- Hydrocarbon lake liquid
- Tholin-like organic sediment
- Perchlorate-bearing regolith
- Comet slush
- Basalt dust
- Micrometeorite dust
- Ablative char
- Plasma as phenomenon/state rather than ordinary Matter
- Superionic ice as future exotic state

These are especially good bridges from reality into SF-feeling experiments.

## 3. When two real materials should remain distinct

Keep separate runtime identities when the player can learn a different rule.

Examples:

```text
Water vs Oil
→ immiscibility + density + combustion difference

Iron/Steel vs Copper
→ corrosion / structural / thermal or future electrical difference

Glass vs Ceramic
→ brittleness / thermal behavior / corrosion resistance

CO2 vs Methane
→ both GAS, but one suppresses combustion and the other fuels it

Salt vs Saltpeter
→ both POWDER, but one changes water/ice/corrosion while the other can feed oxidizing/explosive chains
```

If two entries differ only by lore or color, keep them as encyclopedia aliases or palette variants until a real gameplay difference exists.

## 4. Real-world values are anchors, not contracts

The game does not need SI-perfect simulation.

```text
real density
→ local density ordering

real melting/boiling tendency
→ readable transition threshold

real thermal conductivity
→ cheap conductivity class/value

real corrosion/chemical behavior
→ local interaction rule
```

This keeps the world learnable without making Powdergame a chemical-process simulator.

## 5. Recommended coverage target

The encyclopedia should eventually be comfortable holding **hundreds of real-world names and archetypes**.

The runtime catalog can still be much smaller because entries may normalize to:

- the same behavior family;
- a variant of one descriptor;
- a staged product;
- a future-system candidate;
- an alias/reference entry.

The goal is not a fixed count. The goal is that when a player thinks “what about copper / chlorine / rubber / dry ice / limestone / methane?”, the design corpus already has somewhere sensible to put it.

## 6. Adoption rule

Real-world familiarity is a strong reason for encyclopedia inclusion, but not an automatic reason for implementation.

A runtime candidate should still answer:

1. What does it do in the first three seconds?
2. What existing systems does it connect?
3. What other Material does it meaningfully differ from?
4. What can stop/counter/reverse it?
5. Does it require a new Field or per-cell state?

If those answers are weak, preserve the entry and defer implementation rather than deleting the material from the universe.