# G5-B — Expansion / Confinement → Pressure

**Status:** VALIDATION  
**Branch:** `feature/m0-g5-expansion-confinement`  
**Implementation commit before this document:** `1d6f5c13fe99e88b8da88a25e54561423ce2ff0e`  
**Parent technical gate:** G5-A Pressure Field = TECHNICAL PASS / FROZEN at `c8fcb5e1c8106f6c67f57eba1c31bd256de14818`

This document fixes the G5-B contract before RTX 5090 / DX12 execution. It is not a G5-C rupture design and does not mark G5 as ACHIEVED.

## Claim

A phase transition can request more Matter cells than its source identity occupies. Spatial ownership decides whether that requested expansion can exist. Unresolved expansion becomes Pressure rather than disappearing or invoking special-case explosion code.

Minimum chain proven by this sub-gate:

```text
Water heated above boil threshold
→ Water transforms to Steam at self
→ phase rule requests one additional Steam cell
→ local EMPTY candidate exists?
   ├─ yes → Claim/Resolve → one additional Steam cell
   └─ no / claim lost → blocked expansion → scalar Pressure at source
→ normal G5-A pressure propagation continues later in the tick
```

## Material-owned phase metadata

Phase rules now include:

```text
target_material
matter_yield
blocked_pressure
```

Current G5-B baseline:

```text
Water → Ice:   yield 1, blocked_pressure 0
Ice → Water:   yield 1, blocked_pressure 0
Steam → Water: yield 1, blocked_pressure 0
Water → Steam: yield 2, blocked_pressure 100
```

`100` is a gameplay pressure impulse, not an SI unit.

The current ownership path intentionally supports at most one extra cell (`matter_yield <= 2`). A future larger yield must extend the ownership grammar explicitly rather than silently writing more neighbors.

## GPU ownership grammar

No permanent per-cell expansion field is added. Existing `proposal[]` / `claim[]` scratch is reused sequentially after the movement pass has consumed it.

### Phase / Proposal

Each source invocation:

- reads its own Material + Temperature + phase descriptor,
- writes its own phase result to `material_next[self]`,
- if `matter_yield == 2`, writes at most one local expansion proposal to `proposal[self]`.

Candidate search is local 8-neighbor First-Match with upward preference. There is no long-range scan.

### Expansion Claim

Each EMPTY destination invocation:

- reads local neighboring proposals,
- accepts at most one source,
- resolves contention by smallest source cell index,
- writes only `claim[self]`.

This is the same architectural principle used elsewhere: spatial ownership effects use Propose → Claim/Resolve → Commit instead of arbitrary neighbor writes.

### Expansion Spawn Commit

A winning destination:

- reads the source's already-computed phase result,
- writes the new Matter only to destination self,
- carries source Temperature,
- starts with cleared Matter-owned flags.

### Confinement Pressure

A source whose expanding phase rule cannot realize its requested extra cell because:

- no local EMPTY candidate exists, or
- another source wins the destination claim,

receives its rule's `blocked_pressure` impulse at source.

A successful expansion receives no confinement impulse.

The existing G5-A pressure pass remains responsible for spatial pressure propagation later in the same tick. G5-B does not implement rupture, structure damage, or a boiler-specific explosion.

## Causal order

```text
movement ownership
→ thermal conduction
→ phase transform + expansion proposal
→ expansion claim
→ expansion spawn commit
→ unresolved expansion → pressure impulse
→ phase/expansion state becomes Current
→ decay
→ combustion / smoke
→ G5-A scalar pressure propagation
```

## Technical validation already completed in CI

Windows hosted validation has passed:

- patch/application guards
- `cargo fmt --all`
- `cargo test -p powdergame-core` → 125 passed, 0 failed
- Naga parse of all production WGSL, including all three new expansion shaders
- `cargo check --workspace --all-targets`
- `git diff --check`

This proves source-level contracts and compilation only. Hosted CI is not the production RTX 5090 and therefore cannot close this sub-gate.

## Required RTX 5090 / DX12 evidence

At minimum:

1. Open space: boiling Water produces source Steam + one additional Steam with no confinement Pressure.
2. Fully sealed: boiling Water produces source Steam, no additional Matter, and `blocked_pressure` at source.
3. Contention: two expansions targeting one EMPTY cell produce one winner; loser becomes Pressure.
4. Expansion crosses a 64-cell chunk boundary.
5. Yield-1 phase transitions do not produce expansion Pressure.
6. Existing G5-A pressure tests remain green.
7. Existing G4 phase/thermal/combustion and earlier movement/world invariants remain green.
8. Adapter/backend evidence is RTX 5090 + DX12.

Only after this evidence is captured may G5-B be marked TECHNICAL PASS / FROZEN.

## Explicitly out of scope

- structure stress
- rupture threshold
- pressure-driven wall break
- boiler explosion presentation
- pressure pushing arbitrary movable Matter
- G5-C venting fixture

Those belong to G5-C.
