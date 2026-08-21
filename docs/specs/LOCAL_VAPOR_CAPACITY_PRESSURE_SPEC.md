# Local Vapor Capacity and Gauge-Pressure Specification

- **Status:** Candidate specification — DESIGN BLOCKED
- **ADR:** [`ADR-0008`](../architecture/decisions/ADR-0008-local-vapor-capacity-pressure.md)
- **Depends on:** accepted [`ADR-0006`](../architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md)
- **Replaces:** blocked ADR-0007 token only; generic expansion remains separate
- **Runtime:** NOT STARTED

## 1. State and demand

`phase_energy` is the only Vapor-demand source. `Lv=480` and
`STEAM_EXTRA_VOLUME=1.0`. For valid phase-family state:

```text
r(Ice) = 0
r(Water,E<=0) = 0
r(Water,0<E<=Lv) = E/Lv
r(Steam,E) = E/Lv
```

Non-phase Matter and EMPTY have `r=0`. Non-finite or out-of-range state is an
invariant failure. Runtime logic must not use a clamp to disguise invalid E.

## 2. Capacity law

Neighbourhoods use world-coordinate Chebyshev radius one. An eligible capacity
Cell is in-domain and `Material==EMPTY`; Atmosphere and Vacuum are identical
for this occupancy-only law. Each eligible EMPTY has capacity exactly one and
is not mutated.

For every eligible EMPTY `e` and adjacent phase Cell `i`:

```text
D_e = Σ r_j
a(e,i) = r_i / max(1,D_e)
Σ_i a(e,i) <= 1

capacity_i = min(r_i, Σ_e a(e,i))
deficit_i = max(0,r_i-capacity_i)
compression_i = select(0, deficit_i/r_i, r_i>0)
```

All sums use the same immutable current Material/phase-energy snapshot.
Capacity is recomputed for every current phase Cell every tick. It creates no
claim, reservation, owner, target write or extra Matter.

## 3. Phase-volume pressure target

```text
WATER_VAPORIZATION_CONFINEMENT_PRESSURE_MAX = 100.0
FULL_PRESSURE_COMPRESSION = 0.5
target_i = 100.0 * clamp(compression_i/0.5,0,1)
```

Reference points: Steam plus exclusive EMPTY gives zero; two canonical Steam
sharing one EMPTY give capacity `0.5` and target `100` each; blocked canonical
Steam gives `100`; `r=0` gives zero. This target is recomputed state, not an
additive completion event. A binary response and a smooth nonlinear response
are disclosure alternatives only; the linear response remains normative.

## 4. Pressure update and vent boundary

For Liquid/Gas only:

```text
p0 = max(sanitize(pressure_current), target_i)
p_next = sanitize(
  p0
  + 0.20 * Σ_pressure_medium_neighbors (p_neighbor-p0)
  + 0.20 * Σ_orthogonal_EMPTY_neighbors (0-p0)
)
```

Void, Static, Powder and Boundary contribute no exchange term. A non-pressure
Cell writes gauge zero. The participating-face coefficient sum is at most
`0.8`; non-negative inputs cannot create negative output. An isolated sealed
medium retains gauge pressure. Rupture-created EMPTY is a vent face on the
following pressure pass. Derived Air pressure is not read or added.

`max(current,target)` raises pressure to equilibrium demand but does not erase
stored generic or historical gauge pressure. Generic non-family expansion
continues to add its own event consequence before the later pressure pass.

## 5. TE-3 completion

Initiation remains ADR-0006-owned. Current gas-facing Water may initiate;
positive-E/ready Water retains owned progress. In the atomic TE-5C graph,
initiated or ready Water may complete 1:1 at `E=Lv` even while buried because
the current phase population then enters the state-derived transaction.
Canonical buried Water at `E=0` cannot initiate by capacity/pressure alone.
Completion produces no proposal token, no extra Matter and no fixed impulse.

## 6. Pass and scratch contract

After Smoke's proposal/claim/receiver consumers and joint settle:

1. `vapor_capacity_sum` fully overwrites every proposal Cell as f32 `D_e` for
   EMPTY, or exact zero for occupied/skip Cells.
2. pressure reads current Material/E, neighboring proposal `D_e`, current
   pressure, movement class and chunk state; it writes pressure Next.
3. pressure settles before rupture.
4. phase activity reads the still-live `D_e` and uses the same demand/capacity/
   target predicate as pressure.
5. next-tick movement fully overwrites proposal as u32 before any u32 consumer.

No old TE-5B mode bit is retained. Sleeping paths must self-copy pressure and
must not let stale `D_e` suppress required work. Exact sleep equivalence and
wake halo remain future implementation fixtures.

## 7. Invariants

- **VC-INV-001:** phase-family quantity remains 1:1.
- **VC-INV-002:** demand derives only from valid accepted phase state.
- **VC-INV-003:** each EMPTY contributes aggregate capacity at most one.
- **VC-INV-004:** all current Vapor demand is recomputed every tick.
- **VC-INV-005:** vacancy movement cannot reset a consumed event token; no token exists.
- **VC-INV-006:** no completion token, reservation owner or target mutation.
- **VC-INV-007:** no new persistent/full-world allocation.
- **VC-INV-008:** genuinely sufficient open capacity produces no false target.
- **VC-INV-009:** finite headspace eventually produces nonzero target.
- **VC-INV-010:** target is finite, non-negative and at most `100`.
- **VC-INV-011:** generic expansion pressure remains separate/exact.
- **VC-INV-012:** EMPTY venting is local and causal.
- **VC-INV-013:** sealed isolated gauge pressure is stable.
- **VC-INV-014:** rupture-created opening reduces later pressure.
- **VC-INV-015:** derived Air/background pressure is not counted.
- **VC-INV-016:** pressure and activity use matching demand/capacity predicates.
- **VC-INV-017:** equilibrium/open bulk can sleep.
- **VC-INV-018:** historical evidence remains source-bound.

## 8. Explicit evidence boundary

The grid/time proof may establish only its pure model. It cannot establish
WGSL bindings/races, actual movement, device sleep, profiler/allocation,
performance, visual quality or user acceptance. A failed required fixture is a
design blocker and cannot be repaired by silently changing the law.

## 9. Unsatisfied invariant and blocker

The one-shot proof passed VC-INV-003 but failed VC-INV-008. In the predeclared
two-Steam/two-EMPTY asymmetric graph, a complete adjacency-respecting
assignment exists, but independent per-EMPTY proportional shares allocate
`1.5` gross capacity to A and `0.5` to B. `capacity_A=min(1,1.5)` discards the
extra share instead of reallocating it to B, so B receives target `100` in a
locally sufficient open geometry.

This is a semantic counterexample to the locked law, not missing runtime
evidence. The specification remains historical candidate authority but is
**DESIGN BLOCKED**. No iterative redistribution, matching or larger radius is
authorized by D-020.

Independent review also leaves VC-INV-011 through VC-INV-017 unestablished:
the same EMPTY cannot be both finite sealed capacity and an external zero-
gauge reservoir; the undifferentiated gauge field has no provenance with which
to lower only phase-origin pressure after condensation; downward Chebyshev
capacity is not reachable by the retained GAS stencil; and the projected
activity/snapshot/binding contract cannot observe all new pressure work. See
[`TE5_LOCAL_VAPOR_CAPACITY_PRESSURE_DESIGN`](../adversarial-reviews/TE5_LOCAL_VAPOR_CAPACITY_PRESSURE_DESIGN.md).
