# ADR-0009: Persistent Vapor extent and dedicated phase pressure

- **Status:** Proposed — DESIGN BLOCKED / user architecture revision required
- **Decision owner:** user
- **Program authority:** D-021
- **Runtime:** not started
- **Supersedes for design selection:** ADR-0007 and ADR-0008, both preserved as blocked history

## Context

ADR-0007's exclusive completion token owned an EMPTY only inside one tick, so
ordinary 1:1 Steam movement could walk the same vacancy through a sealed Water
column. ADR-0008's stateless proportional sharing could discard usable
capacity even when a complete assignment existed, and mixed capacity with an
EMPTY vent meaning. Neither candidate conserved finite Vapor capacity across
ticks.

D-021 therefore permits one persistent phase-volume Current/Next pair and
reservation-target Environment mutation. It does not relax the accepted
one-Cell/one-quantity or 1:1 phase-family rules.

## Decision candidate

Add one logical per-Cell pair:

```text
PhaseVolumeState {
    link: u32,
    phase_pressure: f32,
}
```

The two physical buffers are `phase_volume_state_current` and
`phase_volume_state_next`. The link is either NONE, a Steam source's reserved
EMPTY target, the EMPTY target's reciprocal owner, or a compressed-Steam age.
`phase_pressure` is a dedicated gauge component and never aliases the existing
generic `pressure[]`.

The exact candidate encoding is:

```text
MODE_MASK            = 0xC0000000
PAYLOAD_MASK         = 0x3FFFFFFF
PV_NONE              = 0x00000000
PV_SOURCE_RESERVED   = 0x40000000 | (target_index + 1)
PV_TARGET_RESERVED   = 0x80000000 | (owner_index + 1)
PV_SOURCE_COMPRESSED = 0xC0000000 | min(compressed_age, PAYLOAD_MASK)
```

The strict `cell_count < 1 << 30` bound makes every indexed payload nonzero and
keeps it disjoint from the compressed-age mode. Invalid mode/material,
out-of-range payload and nonreciprocal pairs fail closed and are validation
errors; they are not repaired by guessing an owner.

## Spatial ownership

A reserved target remains Material `EMPTY`, but is not free space. It blocks
non-owner Matter movement and Draw, has exact zero Air mass/energy and zero
phase pressure, and is not an Air-flow or thermal node. Acquisition first uses
the existing TE-1 whole-parcel Environment receiver transaction. It commits
only when the target's complete Air parcel can move to one eligible receiver;
otherwise Matter, Air and links remain byte-identical. Release leaves an
ordinary Vacuum EMPTY which TE-2 may refill later.

The approved reservation targets are the in-domain, unreserved EMPTY subset
of the resulting Steam's current five-position GAS movement order: up, the two
parity-ordered up diagonals and the two parity-ordered lateral Cells. Void,
downward, occupied, density-swap and already reserved Cells are excluded.

## Completion, movement and release

Water reaching `E=Lv` completes 1:1 to Steam. A successful reservation writes
one reciprocal pair and equilibrium source zero. Failure produces
`SOURCE_COMPRESSED(age=0)` and equilibrium source 100. A compressed Steam
retries; an already reserved Steam never requests a second target.

Movement semantics are fixed as follows:

- moving into the owner's extent swaps the owner and extent positions;
- moving into another unreserved EMPTY relocates the extent to the vacated
  source, releases the old extent, and moves the destination Air parcel into
  the old extent; the new extent remains zero-Air;
- a density swap keeps the old extent and updates its backlink to the Steam's
  destination owner index;
- Void exit and Steam-to-Water condensation release only the reciprocal extent
  owned by that Steam;
- phase energy and the source-side link follow the Steam identity.

The three-Cell unreserved-EMPTY move requires a phase-volume movement-context
full write before the existing Environment movement reconcile. That context
may reuse `environment_receiver_claim` only after its preceding lifetime is
closed and must be fully overwritten before its later expansion lifetime.

## Matching candidate and hard gate

First-Match and scarcity-only greedy selection are rejected. The candidate is
deterministic scarcity/source ordering plus atomic augmenting-path
reassignment with:

```text
MATCH_SETTLE_TICKS       = 6
MAX_REASSIGNMENT_DEPTH   = 6 source vertices
approved neighbourhood  = five-position in-domain GAS EMPTY stencil above
```

Within a tick, an augment succeeds only if it reaches a free target at or
before the depth bound; a failed search changes no link. Across ticks,
compressed sources retry in descending age, then source-index order. The
existing internal arbitration finalizer breaks otherwise equal choices.

This bound is deliberately frozen before proof. The product condition is not
"usually good matching": whenever a complete matching exists in the approved
domain, no source may reach Wood's threshold before the algorithm finds it or
proves none exists. Failure of that condition blocks this ADR; pressure tuning
must not conceal it.

## Dedicated phase pressure

The frozen coefficient candidate is:

```text
relaxation = 0.10
diffusion  = 0.025
p_eq       = 100 for compressed Steam, otherwise 0

p_next = p_current
       + relaxation * (p_eq - p_current)
       + diffusion * sum(eligible_neighbor - p_current)
```

`0.10 + 4*0.025 = 0.20 <= 1`, so the explicit step is a convex combination of
the source equilibrium, eligible Liquid/Gas neighbours and the retained local
value. EMPTY and non-pressure Matter hold exact zero. A compressed isolated
source first exceeds Wood threshold 80 on tick 16; a match settled within six
ticks remains below the threshold. Acquiring an extent or condensing changes
`p_eq` to zero and permits decline.

Rupture reads exactly one sanitized sum:

```text
effective_gauge_stress = sanitize(generic_pressure + phase_pressure)
```

Generic pressure keeps its own propagation and lifetime. Derived Air pressure
is not included. The future rupture shader can remain at eight storage
bindings only by combining the current rupture-threshold and movement-class
material tables into one descriptor binding; this is a static projection, not
runtime evidence.

## Options compared

| Option | Result | Reason |
|---|---|---|
| Rigid adjacent extent | rejected | owner motion is unnecessarily blocked and does not reuse the zero-Air vacated source |
| Relocate extent to vacated source | primary | closes Matter, extent and Air accounting locally for EMPTY movement |
| Persistent nonlocal link with bounded rehoming | retained only for density swap | avoids inventing capacity where no EMPTY was created |
| First-Match | rejected | asymmetric A→E1/E2, B→E1 produces avoidable compression |
| Scarcity ordering only | rejected | longer alternating paths still require reassignment |
| Bounded augmenting reassignment | proof candidate | fixed pass budget, but must pass the hard matching condition |
| Unbounded exact maximum matching | not authorized | no fixed production pass bound or source-proven scratch/liveness contract |

## Allocation and pass projection

The pair adds 16 bytes per Cell: 1,048,576 bytes at 256² and 67,108,864 bytes
at 2048². TE-3 plus TE-5D would project to 369,125,680 tracked bytes at 2048²
before profiler or any other allocation change. These are checked design
arithmetic only.

The accepted TE-3 projection is 40 passes. A fixed-depth design can be
expressed as a projected 62 passes / 124 timestamp queries: one movement
context/reconcile extension, eighteen proposal/claim/augment rounds, one
Environment receiver pass, one reservation commit and one phase-pressure
pass. Air/thermal masking is added to their scale/predicate passes; commit
passes infer zero transfer from the scale and canonical zero-Air target. No
new full-world scratch is assumed. Exact maximum matching over arbitrary world
components is not represented by this fixed projection.

## Consequences and stop boundary

The candidate preserves phase-family quantity and makes capacity a real,
cross-tick spatial ownership relation. It costs 64 MiB at 2048² and introduces
substantial link hygiene, matching and activity obligations. It is not a
second foreground Matter: Material remains the sole occupancy identity and the
link can only reference one auxiliary EMPTY extent.

ADR-0009 remains Proposed. TE-3/TE-5 runtime, buffer allocation, full
background-Air pressure, structure differential, product edge mode, Vacuum
combustion and TE-4 remain not started. An unresolved Critical/High proof or
review finding makes TE-5D DESIGN BLOCKED.

## Reference-proof disposition

The frozen one-shot proof returned `DESIGN_BLOCKED`. Its fresh-start matching
model passed 50,000 randomized graphs, 10,000 abstract multi-tick grids and
the modeled coefficient/link checks, but it did not directly enumerate all
`2^36` labeled 6×6 graphs. More importantly, that model did not admit an
arbitrary already-persistent reciprocal matching as initial state.

A legal persistent counterexample is an alternating chain with sources
`U0..U7`, targets `V0..V7`, existing pairs `Ui→Vi` for `i=0..6`, unmatched
`U7`, edge `U7→V0`, and edges `Ui→{Vi,V(i+1)}` for `i=0..6`. The complete
matching `U7→V0, Ui→V(i+1)` exists, but its only augmenting path contains eight
source vertices. The frozen depth-six atomic retry makes no change, so the
unmatched source can reach phase pressure 80 despite sufficient capacity.

This violates PVX-INV-011 and the hard product condition. The required repair
is **wider matching scope**. A fixed-budget GPU realization may also require a
dedicated full-world frontier/predecessor scratch or unbounded pass count, but
this task does not silently authorize either. The 1:1 quantity and volume
representation are not disproved by this counterexample.
