# Powdergame Material Prototype Bundles

## Status

- Type: `DERIVED`
- Authority: non-authoritative research
- Scope: prototype planning only
- M0 Evidence Gates are unchanged.

This document turns the material shortlist into testable interaction bundles.

A bundle is preferred over an isolated Material when several Materials together prove that existing world rules compose correctly.

---

## Bundle A — Volatile Atmosphere

### Matter

```text
Dry Ice
CO2
Wood/Oil or another existing combustible Matter
```

### Intended chain

```text
Dry Ice
+ Heat
→ CO2
→ GAS movement
→ density stratification
→ CO2 accumulates low
→ combustion encounters CO2
→ combustion continuation is suppressed
```

### Existing grammar reused

- STATIC → GAS transition
- Temperature
- Gas movement
- Density Rank
- Combustion interaction

### New content/rule work

- Dry Ice material descriptor
- Dry Ice → CO2 thermal transition
- CO2 gas descriptor
- combustion-suppression rule/trait

### Must not introduce

- hidden atmospheric composition
- universal oxygen percentage
- fluid-mixture solver
- special “fire extinguisher” engine pass

### Prototype questions

1. Does Dry Ice visibly behave differently from Water Ice?
2. Does CO2 naturally settle relative to Steam/Smoke without special buoyancy code?
3. Can a player build a low CO2 pocket that changes fire behavior?
4. Does the result come from ordinary local rules rather than a scenario script?

### Pass evidence

- deterministic test: heated Dry Ice transitions to CO2
- density test: CO2 ordering relative to selected existing gases is stable
- combustion test: CO2 adjacency changes combustion continuation through compiled/local rules
- mixed-world smoke test: no invalid Matter IDs / NaN / corruption

### Prerequisite substrate

Movement + Density + Thermal/Transition + Combustion grammar.

---

## Bundle B — Trapped Fuel / Pressure Accident

### Matter

```text
Clathrate
Methane
existing wall / Boundary / Stone
existing ignition source or combustible heat source
```

### Intended chain

```text
Clathrate
+ Heat
→ Methane release
→ confined GAS accumulation
→ ignition
→ Heat + Pressure
→ pressure propagation
→ rupture / opening
→ vent / flame jet
```

### Why this bundle matters

This is the strongest convergence point in the current research because one ordinary-looking solid connects:

- Temperature
- Material transition
- spawn/expansion request
- Gas movement
- Combustion
- Pressure
- Rupture
- Venting

It tests the Powdergame thesis that a dramatic accident can emerge without a dedicated “clathrate explosion” system.

### New content/rule work

- Clathrate descriptor and thermal transition
- Methane gas/fuel descriptor
- gas release/yield rule
- methane combustion effect expressed as Heat + Pressure

### Must not introduce

- dedicated radial explosion solver
- hardcoded boiler/clathrate scenario logic
- non-local arbitrary blast deletion
- separate methane pressure field

### Prototype questions

1. Does heating Clathrate in open space look harmless enough to teach gas release?
2. Does the same experiment become dangerous in a sealed room naturally?
3. Does ignition produce a pressure event through existing pressure semantics?
4. Does opening a vent change the outcome without special casing?
5. Can the player understand the causal chain from the resulting world state?

### Pass evidence

- transition test: heated Clathrate requests/releases Methane
- ownership test: spawn/expansion preserves one-cell-one-Matter invariant
- combustion test: Methane converts ignition into Heat/Pressure through ordinary rule effects
- pressure integration test: sealed setup builds higher pressure than vented setup
- rupture test: pressure-resistant and breakable structures respond according to generic properties

### Prerequisite substrate

Thermal/Transition + Combustion + Pressure/Rupture.

This bundle should wait until the generic pressure chain exists rather than pulling Pressure earlier.

---

## Bundle C — World Fabrication

### Matter

```text
Clay
optional Wet Clay or Mud-like stage
Brick
Sand
Glass
Water
```

### Intended chains

```text
Clay
+ Water condition
→ workable wet state
+ Heat
→ Brick
```

and:

```text
Sand
+ extreme Heat
→ Glass
```

### Why this bundle matters

It changes the sandbox from “materials react” to “the world manufactures materials.”

The reward is spatial and visible; no crafting menu is required.

### New content/rule work

- Clay POWDER
- Brick STATIC
- optional staged wet-clay identity
- Sand → Glass high-temperature transition if adopted

### Must not introduce

- universal wetness float
- hidden recipe inventory
- crafting UI as a prerequisite

### Prototype questions

1. Is wet Clay mechanically distinguishable from Dirt/Mud enough to deserve its own identity?
2. Can Brick be understood as a result of Heat, not an arbitrary recipe?
3. Does Glass create a meaningful new structural/thermal behavior beyond color?
4. Are transitions readable from the world itself?

### Pass evidence

- phase/reaction tests for each transformation
- movement-class transition test POWDER/LIQUID-like stage → STATIC
- cooling/heating cycle stability
- mixed-world validation with no accidental reverse transforms

### Prerequisite substrate

Movement classes + Temperature + Material transitions/reactions.

---

## Bundle D — Thermal Engineering

### Matter

```text
Cryofluid
Ablative Char
Lava
Metal / hot Metal
Water / Ice
```

### Intended chains

```text
Cryofluid
+ hot neighbor
→ local Heat exchange
→ target reaches its own transition threshold
→ target transforms using its own rule
```

Examples:

```text
Water → Ice
Lava → cooled solid
Molten Metal → Metal
```

Ablative branch:

```text
extreme Heat
→ Ablative Char consumes/damages itself
→ less Heat reaches rear structure
→ spent residue
```

### Why this bundle matters

This bundle directly tests one of the central design principles:

> A special Material should alter a shared field; target Materials should own their consequences.

Cryofluid should not contain explicit rules such as “if Lava then Stone” or “if Water then Ice” unless generic interaction semantics prove insufficient.

### State-cost question

Ablative lifetime should first be represented as staged Material identity:

```text
Ablative Char
→ Damaged Ablative
→ Spent Char
```

Only add a generic `fatigue8`/capacity state if multiple Materials prove that the continuous value creates visible gameplay worth its memory/bandwidth cost.

### Pass evidence

- heat exchange is local and finite
- target transitions happen through target rules
- no magic temperature reset
- Ablative state degrades and eventually fails
- behind-wall thermal trace differs measurably with and without Ablative material

### Prerequisite substrate

Thermal propagation + transitions. Pressure is optional for the first prototype.

---

## Bundle E — Salt / Brine / Ice Network

### Matter

```text
Salt
Water
Brine
Ice
Metal later
```

### Intended chain

```text
Salt + Water relation
→ Brine
→ different density / freezing behavior
→ later corrosion interaction with Metal
```

### Design constraint

Do not represent this by adding universal continuous salinity to every Water cell first.

Prefer explicit staged Matter identity while it remains sufficient:

```text
Water
Brine
```

### Why this bundle matters

Salt stops being decorative Powder and becomes a connector across:

- Liquid movement
- Density
- Temperature / freezing
- later corrosion

### Prerequisite substrate

Density + Temperature + transitions/reactions.

---

## Relative prototype value

| Rank | Bundle | Interaction Yield | New engine substrate | Main risk |
|---|---|---:|---|---|
| 1 | B — Trapped Fuel | Very High | none beyond planned Pressure | accidentally inventing bespoke explosion logic |
| 2 | D — Thermal Engineering | Very High | none | magic cooling / universal fatigue state |
| 3 | A — Volatile Atmosphere | High | none | slipping into atmospheric simulation |
| 4 | C — World Fabrication | High | none | recipe-like special cases / redundant wet states |
| 5 | E — Salt Network | Medium-High | none | hidden mixture/salinity state |

---

## Prototype promotion rule

A bundle should not move from research to authoritative content merely because the rules can be coded.

Promotion requires evidence that:

1. the player can read the cause/effect,
2. the interaction creates more than one useful experiment,
3. the implementation reuses generic systems,
4. no heavy universal state was added for one Material,
5. the bundle survives mixed-world validation,
6. the resulting world is more interesting than the same number of simpler palette variants.

Only after that should individual Materials be proposed for `REGISTERED` / `IMPLEMENTED` / `VALIDATED` status through the normal docs authority path.
