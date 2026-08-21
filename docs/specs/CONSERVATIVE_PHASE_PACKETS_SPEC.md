# Conservative Phase Packets Specification

- **Status:** PROPOSED — DESIGN BLOCKED / ARCHITECTURE REVISION REQUIRED
- **ADR:** [`ADR-0011`](../architecture/decisions/ADR-0011-conservative-phase-packets.md)
- **Decision:** D-023
- **Runtime:** NOT STARTED

## 1. Scope and authority

This candidate replaces only ADR-0006's one-Cell/one-whole-quantity ontology.
Its TE-2 heat-transfer, local phase-energy, strict initiation, real-sink,
radius-two nucleation, Atmosphere/Vacuum and evidence boundaries remain in
force. ADR-0007 through ADR-0010 remain blocked history. No statement here is
runtime evidence or permission to implement.

## 2. State and valid combinations

```text
phase_units_current   : u32[Cell]
phase_units_next      : u32[Cell]
phase_energy_current  : f32[Cell]
phase_energy_next     : f32[Cell]
phase_pressure_current: f32[Cell]
phase_pressure_next   : f32[Cell]
```

| Matter | Units | Canonical E | Valid E |
|---|---:|---:|---:|
| EMPTY/non-phase | 0 | 0 | exactly 0 |
| Ice | 2 | -80 | [-80, 0] |
| Water | 2 | 0 | [-80, 480] |
| Steam | 1 | 240 | [0, 240] |
| Steam | 2 | 480 | [0, 480] |

No phase Matter has zero units; no value exceeds two; Water and Ice never have
one unit. Every value is finite and invalid combinations are invariant
failures. The Steam E range scales with quantity so per-quantity vapor latent
fraction remains consistent.

For `q=units/2`:

```text
S_ice(T)   = 2.0 * T
S_water(T) = 2.5 * T
S_steam(T) = 2.5 * 100 + 0.8 * (T - 100)
H          = q * S_material(T) + E
```

Every split/merge tolerance is
`max(1e-3, 2e-6*max(1,abs(H_before),abs(H_after)))` and covers both sides of
the transaction. The reference uses exact rational arithmetic before applying
that future-f32 tolerance.

## 3. Boil and split transaction

At Water/2 `E=480`, a current gas-facing crossing or initiated/ready Water
enters a provisional completion transaction. The phase pass may settle Steam/2
only after the same snapshot has produced either a valid targeted expansion
request or an explicit blocked request. Buried canonical Water without prior
positive E and non-gas-facing extreme Ice cannot use this route.

The targeted request uses the existing GAS-reachable EMPTY stencil and
stateless destination claim. It excludes occupied Matter, density swaps,
downward-only targets and Void. A winner must also receive the target's whole
Air parcel through the existing TE-1 receiver transaction.

On success:

```text
Water/2 -> source Steam/1 + target Steam/1
units: 2 -> 1+1
E: 480 -> 240+240 at the canonical endpoint
target Air is displaced by the existing whole-parcel receiver transaction
phase-pressure equilibrium at both Steam/1 sources is zero
```

On blocked request, claim loss or receiver failure:

```text
Water/2 -> source Steam/2
target Matter/Air unchanged
units and H unchanged
phase-pressure equilibrium becomes 100
```

An existing Steam/2 retries the same transaction on later ticks. Success
creates two Steam/1 packets; failure retains Steam/2. The transaction does not
reapply latent heat or generic blocked-expansion pressure. Generic non-family
yield-two rules keep their historical event consequence and cannot target a
phase-family destination without a future owned-state design.

## 4. Local contraction

Merge eligibility requires two orthogonally adjacent Steam/1 Cells that are
both endpoint-ready under the accepted condensation-work predicate. A packet
at E=0 may cool sensibly while waiting; it remains Steam/1 and finite. The
selected target order is orthogonal only, rotated deterministically by tick
parity and the existing internal coordinate-hash priority. Proposal and claim
are fully overwritten and one source can propose at most one partner; one Cell
can participate in at most one winning pair per tick.

Commit chooses the claim winner as Water/2 and the partner as EMPTY/0. It sums
units, E and H, then normalizes the winner into the valid Water/2 E/T range.
The loser writes units/E zero and canonical Vacuum Air mass/energy zero. No Air
parcel is moved to the occupied winner, no Air is invented, and later TE-2 Air
flow alone may refill the vacancy. Claim loss changes nothing.

Steam/2 at the condensation endpoint converts in place to Water/2. A lone
Steam/1 never becomes Water/1. This metastability may sleep when no thermal,
merge, movement or pressure work is runnable.

## 5. Movement, editing and reset

- EMPTY movement copies packet units/E to destination and writes zero at the
  vacancy; density swap exchanges each Matter's values.
- Void exit subtracts the exiting units from the finite-world audit and clears
  the source.
- Non-phase replacement and Erase write units/E zero.
- Draw stages Ice `(2,-80)`, Water `(2,0)`, Steam `(1,240)`, other/EMPTY
  `(0,0)` in both halves. Heat/Cool does not change units directly.
- Reset, presets, scenarios, benchmarks, Inspector staging and every bypass
  writer must stage Current/Next byte-identically.
- Phase pressure is never copied with Matter. Its spatial update reacts to the
  new current identity/units after settle.

## 6. Phase pressure

Let `p_eq=100` only at Steam/2, otherwise zero. Only Liquid/Gas pressure-media
Cells retain or diffuse the component:

```text
p_raw = p_current
      + 0.20 * (p_eq - p_current)
      + 0.05 * sum(orthogonal pressure-medium neighbour - p_current)
```

Inputs and output are sanitized to `[0,100]`. Non-media write zero. If the
finite update magnitude is below `0.01`, the Cell retains its current value;
it wakes when local source class or a neighbour changes. This finite settle
rule prevents an asymptotic activity leak without snapping a pressure gradient
to zero. An isolated Steam/2 rises `20,36,48.8,59.04,67.232,73.7856,79.02848,
83.222784`, crossing Wood `80` on update 8. After split or condensation the
source becomes zero and the same law declines it.

Generic `pressure[]` is neither relaxed nor erased by this pass. Rupture sees
`sanitize(generic_pressure + phase_pressure)` once. Phase pressure is spatial,
may diffuse through pressure media, and has no derived-Air/background term.

## 7. Pass, scratch and binding projection

The 50-pass/100-query design graph is:

```text
0       activity_wake
1..5    existing movement transaction
6       phase_units_reconcile_movement
7       phase_energy_reconcile_movement
8..11   TE-2 Air/thermal scale and commit
12      phase_context_propose
13      phase_packet_enthalpy_normalize (T/E/units Next)
14      phase_packet_identity_split_propose (Material Next/proposal)
15..17  expansion claim, Environment receiver claim, Matter spawn
18      phase_packet_split_commit
19..22  generic expansion pressure/hygiene/Environment reconcile
23      phase_packet_merge_propose (proposal full write)
24      phase_packet_merge_claim (claim full write)
25      phase_packet_merge_identity_temperature_commit
26      phase_packet_merge_units_energy_commit
27      phase_packet_merge_environment_reconcile
28..31  decay, flag/phase hygiene, Environment reconcile
32..38  combustion/Smoke transaction, flag/phase hygiene, Environment reconcile
39      generic pressure
40      phase_pressure
41..44  rupture, flag/phase hygiene, Environment reconcile
45      base activity_propose
46      phase_activity_propose
47      phase_pressure_activity_propose
48      environment_activity_propose
49      activity_reduce
```

Current/Next settles are encoder copies between the named windows, as in the
accepted TE-3 projection; they are not hidden compute dispatches or profiler
passes. Any source audit that cannot realize this exact order at 50 is an
architecture revision, not license to add a hidden pass.

Representative maximum storage bindings:

| Pass | RO | RW | Total |
|---|---:|---:|---:|
| units movement reconcile | 6 | 1 | 7 |
| enthalpy normalize | 5 | 3 | 8 |
| identity/split propose | 6 | 2 | 8 |
| existing expansion spawn | 5 | 3 | 8 |
| packet split commit | 4 | 4 | 8 |
| merge propose/claim | 6 | 1 | 7 |
| merge identity/T commit | 6 | 2 | 8 |
| merge units/E commit | 6 | 2 | 8 |
| merge Environment reconcile | 6 | 2 | 8 |
| phase pressure | 3 | 1 | 4 |
| rupture after descriptor-table packing | 5 | 3 | 8 |
| phase activity with units | 7 | 1 | 8 |
| phase-pressure activity | 4 | 1 | 5 |

Proposal and claim are full-written for expansion, then full-written for merge,
then later overwritten by Smoke. Environment receiver scratch remains live
through spawn/reconcile and is not reused. The existing base activity pass is
already at eight storage bindings, so phase-pressure activity is separate.

Tracked memory is 5,771,504 B at 256² and 369,127,280 B at 2048² including two
50-pass profiler read/resolve buffers (1,600 B). This is arithmetic only.

## 8. Invariants

- **PQ-INV-001:** finite-world phase-unit sum changes only at Void/destructive authoring.
- **PQ-INV-002:** Ice/Water are units 2; Steam is units 1 or 2.
- **PQ-INV-003:** boiling split preserves units and H.
- **PQ-INV-004:** blocked boil creates compressed Steam, not quantity.
- **PQ-INV-005:** delayed split consumes a real EMPTY Cell.
- **PQ-INV-006:** vacancy walking cannot manufacture capacity.
- **PQ-INV-007:** two Steam/1 contract to one Water/2.
- **PQ-INV-008:** no Water/1 or Ice/1 exists.
- **PQ-INV-009:** lone endpoint-ready Steam/1 is finite and explicit.
- **PQ-INV-010:** movement carries units/E with Matter.
- **PQ-INV-011:** phase-pressure source derives only from Steam/2.
- **PQ-INV-012:** rupture combines generic/phase pressure once.
- **PQ-INV-013:** split/merge Environment accounting is exact.
- **PQ-INV-014:** no global matching, CCL or remote withdrawal exists.
- **PQ-INV-015:** activity matches split, merge, latent and pressure work.
- **PQ-INV-016:** invalid identity/unit/E combinations fail closed.

## 9. Frozen-candidate disposition

Fresh independent review found Critical `0` / High `8` / Medium `1`. The
candidate is **DESIGN BLOCKED**. In particular, this frozen pass/sleep table
omits the phase-pressure `chunk_state` read; the local merge contract can
strand pairable packets; spatial pressure can be reset by Steam/2 movement or
remain rupture-eligible after its source disappears; and the exact f32 split,
generic-pressure exclusion and authoritative-writer transactions are not
closed. The reference PASS does not resolve these source-integration defects.
This section records disposition only and does not revise the frozen formulas.
