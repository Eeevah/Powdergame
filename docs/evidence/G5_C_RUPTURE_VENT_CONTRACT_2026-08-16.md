# G5-C Pressure Stress → Rupture → Opening → Vent — Contract

Date: 2026-08-16
Status: **PLANNED**

## Purpose

Complete the remaining G5 Pressure Chain without a boiler-specific explosion rule:

```text
Pressure in Liquid/Gas
→ adjacent structural Matter reads local pressure stress
→ generic Material rupture threshold exceeded
→ structural cell becomes EMPTY
→ opening exists in the normal Matter grid
→ Gas/Liquid movement uses that opening on following ticks
→ pressured medium can leave / pressure is released by the existing spatial-field rules
```

## Architecture rules

- Pressure stays a spatial scalar field owned by cells, not by Matter identity.
- Static structures do **not** become pressure media and do not store pressure merely because they are stressed.
- Structural stress is derived from neighboring pressure-medium cells.
- Rupture strength is Material descriptor data, not a material-name branch in WGSL.
- Boundary Block remains unbreakable in G5-C baseline.
- Stone and Wood may have distinct finite rupture thresholds.
- Rupture is `Read Neighbors → Write Self`: a structural cell only decides whether its own `material_next[self]` becomes `EMPTY`.
- No atomics or cross-cell writes are required for rupture itself.
- Rupture clears temperature/flags for the newly EMPTY cell, preserving the existing EMPTY invariant.
- No explosion impulse, fragment particles, radial damage, or presentation-only blast code is required for G5-C.
- Venting must emerge from the opening plus the already-existing movement/Pressure semantics.

## Required technical evidence

1. Sub-threshold neighboring pressure does not rupture a structure.
2. Threshold-exceeding pressure ruptures a finite-strength structure.
3. Boundary Block remains unbreakable under extreme pressure.
4. Rupture works across a 64-cell chunk boundary.
5. A sealed hot-water fixture can naturally produce confinement Pressure, rupture a weak wall, create an opening, and then release Matter/Pressure through that opening over subsequent ticks.
6. Existing G5-A and G5-B tests remain green.
7. All production WGSL parses without a GPU.
8. Full GPU regression remains green on RTX 5090 / DX12.

## User validation after technical pass

The final G5 user validation is not merely a unit-test result. A small interactive/reference boiler fixture should demonstrate that the chain looks causally understandable without a special `explode_boiler()` path.
