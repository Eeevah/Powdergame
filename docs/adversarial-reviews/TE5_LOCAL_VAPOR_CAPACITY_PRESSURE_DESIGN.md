# Independent TE-5C Local Vapor Capacity and Gauge-Pressure Design Review

- **Date:** 2026-08-21
- **Reviewer role:** fresh-context independent adversarial reviewer; not a
  primary-author participant
- **Reviewed branch:** `feature/m0-g9-first-playable`
- **Design baseline:** `6a1c83fad702d18f2d24365a4fc747ab74225f5c`
- **Review-time HEAD:** `cceca63fc7597aa58d6053a15118f761964366e0`
- **Current unresolved:** Critical **0**, High **6**
- **Runtime/proof execution by this reviewer:** **0**
- **Verdict:** **TE-5C DESIGN BLOCKED / ADR-0008 PROPOSED /
  ARCHITECTURE REVISION REQUIRED / RUNTIME NOT STARTED**

## 1. Scope and evidence boundary

This review attacks the D-020-authorized docs/reference-only candidate
**LOCAL VAPOR CAPACITY SHARE + GAUGE-PRESSURE EQUILIBRIUM**. It directly read
D-018 through D-020, ADR-0006/0007/0008, both phase and local-capacity
specifications and validation contracts, the TE-5C plan, implementation gates,
G5 milestones, production inventory, and the current pressure, rupture,
thermal, phase, movement, Simulation, proposal/claim, activity and profiler
sources and tests.

The repository was already user-dirty. This review did not modify any existing
file, runtime source, proof artifact, test, build output, Wiki page, memory
record or status authority. This review file is the only reviewer-authored
write.

The external script and result were read, SHA-256 hashed and the JSON parsed.
They were not executed, regenerated or modified. No Cargo, Rust, WGSL, Naga,
GPU, build, launch, fixture, performance or product command was run.

## 2. Frozen input hashes

| Input | SHA-256 |
|---|---|
| ADR-0008 | `3d804573caf33d69554f8ebdf2eea9ece6af98651d44da4660eea0f7a5905239` |
| local capacity specification | `2a1db37829e88cab9dcaea3c6de66c8608729d7a19d626eed8863c81b5f966ee` |
| local capacity validation | `d31d0cd6b465761c9c634c0ae1fd25effa70caffcec6ef8f765c61cbddf274df` |
| TE-5C plan | `3c1446a8dc8fc83ccbaa214bcce3149baa28dac0396e22a2891a295d1fac5f12` |
| production inventory | `61741ab92a9a1a79028081a531d298d4c0c7b8ad853ae42933a9ec647da0a086` |
| ADR-0006 | `1a46fd444f76836f88353b04faa99be92994ed225baf68fea41004e3b6abaf2c` |
| phase specification | `51a69e30414603b4ac8aca6f088859702202fcff6740d7f81b3395fb1b2099e9` |
| implementation gates | `dd8aa826914a9c763079821eb57577817b144d146e113338d2b6c15ecdd21da9` |
| milestones | `33ad4174eaa322246eef634580ab2ebf2bbe2e1726651262210d8fe6d11d7bc8` |
| D-018..D-020 ledger | `8e4dfdb84800bf8832b864cbb5ec4a718d46c6fb054bbcff5c8820f134a7d3d8` |
| checkpoint | `1db168e7c6554d3ae70a8052be6880725504c59273d50e190d481814837bd9af` |
| external reference script | `f0b4cb155fcc0785c60ff6ff4c2ee9d18a439ed3ea0941e679140de4188af791` |
| external reference result | `59b98a3454e13a22742e66559e06cfa9b3552a37e18929fa3b71949afaf1e8e5` |

The parsed result reports `DESIGN_BLOCKED`, one process execution, fixed seed
`0x54453543`, 50,000 static cases, 10,000 multi-tick cases, two matching
replays, digest
`3f01a0cb3033f157ba2371c0c4b52dd8d32daecee638b53e4da61a3337565b76`,
and one reported failure: `reachable_capacity_no_false_pressure`.

## 3. Finding counts

| Severity | Open | Resolved in reviewed snapshot |
|---|---:|---:|
| Critical | **0** | 0 |
| High | **6** | 0 |

The verdict is controlled by any one unresolved High. H-001 independently
reproduces the predeclared blocker; H-002 through H-006 are additional.

## 4. Findings

### TE5C-H-001 — Proportional sharing discards reachable capacity

- **Severity:** High
- **Status:** **OPEN — PREDECLARED DESIGN BLOCKER INDEPENDENTLY REPRODUCED**
- **Witness:** Steam `B=(0,1)` and `A=(1,1)` each have demand 1. EMPTY
  `E1=(0,0)` is adjacent to both; EMPTY `E2=(2,1)` is adjacent only to A.
  The complete assignment `B->E1, A->E2` exists. The locked law gives B 0.5
  from E1 and A 0.5 from E1 plus 1 from E2. A is capped at 1 and discards its
  excess 0.5; B remains at capacity 0.5 and receives target 100.
- **Impact:** VC-INV-008 fails even though VC-INV-003 passes. A genuinely
  sufficient local opening can fabricate rupture-capable pressure. This is a
  semantic failure of the exact formula, not missing GPU evidence.
- **Disposition:** D-020's stop rule applies exactly: **TE-5C DESIGN BLOCKED**.
  No redistribution, matching, radius or curve substitution is authorized.

### TE5C-H-002 — Every internal EMPTY is both finite capacity and an infinite vent reservoir

- **Severity:** High
- **Status:** **OPEN — DESIGN BLOCKER**
- **Witness:** Put pressured Steam beside an EMPTY Cell inside an otherwise
  sealed Stone vessel. Section 2 gives that EMPTY only one unit of finite
  headspace capacity, but Section 4 simultaneously applies
  `0.20 * (0-p0)` every tick as though the same Cell were an external
  gauge-zero reservoir. No path from that EMPTY to Atmosphere, Vacuum or Void
  is required. The `air_mass`/`air_energy` state is not read.
- **Impact:** Sealed finite headspace leaks gauge pressure through its own
  interior vacancy. The F11 chain can be sustained by repeatedly injecting
  the target floor while the same headspace drains it, rather than by a
  conserved confinement/vent model. Generic non-family gauge pressure is also
  drained whenever it happens to border any internal EMPTY, changing the
  frozen G5 field semantics without source provenance.
- **Authority conflict:** The candidate calls EMPTY a finite capacity unit and
  says sealed pressure is stable, while production currently treats EMPTY as
  a non-medium/no-exchange boundary. A rupture-created opening is not
  distinguishable from pre-existing sealed headspace by Material alone.
- **Disposition:** The candidate needs an explicit reservoir/connectivity or
  pressure-component ownership model. Occupancy alone cannot establish both
  finite capacity and external vent identity.

### TE5C-H-003 — The pressure update cannot implement downward phase equilibrium

- **Severity:** High
- **Status:** **OPEN — DESIGN BLOCKER**
- **Witness:** A sealed compressed Steam region raises gauge pressure to 100.
  Cool/condense it until all phase demand and targets become zero while the
  region remains Liquid/Gas pressure medium and has no EMPTY face or pressure
  gradient. The locked update starts from
  `p0=max(sanitize(p_current),target)`, so the phase-created 100 remains
  forever. The result is no longer derived from current Vapor demand.
- **Impact:** Condensation can lower the computed target but cannot lower its
  own pressure contribution. Replacing `max` with direct relaxation would in
  turn erase unrelated generic expansion pressure because both sources share
  one undifferentiated persistent gauge field. The design therefore cannot
  simultaneously provide state-derived phase equilibrium and preserve exact
  generic pressure without provenance/state that it forbids.
- **Disposition:** F09's opening-only decline does not close this witness. A
  later architecture must own/separate phase pressure contribution or define
  an accounted relaxation law and revalidate generic G5 behavior.

### TE5C-H-004 — Chebyshev capacity includes EMPTY Cells Steam cannot reach

- **Severity:** High
- **Status:** **OPEN — DESIGN BLOCKER**
- **Witness:** Surround canonical Steam on up, both up-diagonals and both
  lateral sides, but leave only a down or down-diagonal Cell EMPTY. The locked
  eight-neighbour capacity law returns capacity 1 and target 0. Production GAS
  movement searches only up, up-diagonal and lateral; it never moves down.
  The pressure vent law is orthogonal only and no other volume solver exists.
- **Impact:** The candidate can report complete relief although neither the
  actual Steam owner nor the pressure field has an authorized route to use
  that capacity. Buried initiated Water may also complete through this false
  transaction. This breaks the required causal relation between capacity and
  the retained production movement system.
- **Disposition:** Radius-one geometry must be constrained to an explicitly
  realizable route or accompanied by an authorized solver/state. Changing the
  stencil is a locked formula change and is not permitted inside D-020.

### TE5C-H-005 — Activity/sleep cannot represent the new pressure work within the projected graph

- **Severity:** High
- **Status:** **OPEN — DESIGN/BINDING BLOCKER**
- **Witness A:** Production `activity_propose` explicitly ignores
  medium-to-EMPTY pressure edges. TE-5C makes that exact edge vent work, but
  specifies no corresponding base pressure-activity change. A uniform generic
  gauge region beside EMPTY can therefore be classified stable; once its
  chunk sleeps, production pressure's sleeping path self-copies pressure and
  the promised vent stops.
- **Witness B:** The projected phase-activity pass is already at eight storage
  bindings with Material, temperature, phase energy, Air mass/energy, chunk
  state, proposal `D_e`, and activity RW. It has no pressure binding. It can
  either mark every positive target active forever, violating VC-INV-017, or
  fail to know that `pressure_current < target` still requires target-injection
  work. Adding pressure would exceed the stated eight-storage ceiling unless
  another binding/pass/contract changes.
- **Ordering amplification:** Capacity `D_e` is computed before pressure and
  rupture, but phase activity consumes it after rupture against the later
  Material snapshot. A rupture-created EMPTY can therefore be interpreted
  with pre-rupture `D_e`, while pressure used the earlier occupancy. The
  claimed matching predicate is not one immutable snapshot.
- **Impact:** VC-INV-012, VC-INV-016, VC-INV-017 and sleep-on/off equivalence
  are not established by the 41-pass/82-query projection.
- **Disposition:** The activity producer/consumer snapshot, vent frontier and
  target-convergence predicate must be redesigned and the pass/binding counts
  recalculated. A future fixture alone cannot repair a missing data path.

### TE5C-H-006 — The one-shot receipt reports mandatory checks that the script does not perform

- **Severity:** High
- **Status:** **OPEN — EVIDENCE INTEGRITY BLOCKER**
- **Observed static facts:**
  - `F13_partition_reset_replay` is assigned literal `True`;
  - `partition_digest` and `partition_digest_2` hash the same `phase` object
    with the same expression, so no alternate logical partition is modeled;
  - `opening_peak_increases` is initialized to zero but has no increment or
    failure check;
  - F09 changes Solid to EMPTY but performs no condensation/phase-demand
    update, despite being reported as condensation/opening relief;
  - F10 tests only that sealed generic Gas pressure 100 remains 100; it does
    not test coexistence with a phase target, duplication, or the new EMPTY
    vent regression;
  - F06 is assigned the F05 boolean rather than an independent finite-headspace
    crossing check.
- **Impact:** The validation contract and receipt say F01-F13, logical chunk
  partition equality, opening behavior, condensation direction and generic
  separation passed. Those claims exceed the code that produced the JSON.
  Hash validity and deterministic replay authenticate the script/result bytes,
  not the absent properties.
- **Disposition:** Preserve the one allowed result exactly as required, but
  classify these properties **NOT ESTABLISHED**, not PASS. D-020 forbids
  patching/rerunning this proof; a later architecture needs a newly authorized
  evidence contract rather than rebinding this receipt.

## 5. Required attack coverage summary

| Attack | Result |
|---|---|
| proportional capacity double count | Per-EMPTY aggregate is bounded; per-Cell cap discards usable share, H-001 |
| capacity underuse / reachable assignment | **H-001 open** |
| radius false pressure / false relief | **H-004 open** |
| dense cloud / vacancy walk | Vacancy-walk abstract check passes; it does not close H-001/H-002 |
| partial E | Continuous target is modeled; pressure provenance on reversal is H-003 |
| condensation pressure clearing | **H-003 open; proof does not model it, H-006** |
| EMPTY vent generic-pressure regression / Air leak | **H-002 open** |
| non-negative/stability | Coefficient sum is at most 0.8 and clamp is non-negative in the abstract rule; this does not close semantics |
| scratch/binding/order | Capacity write window is statically available; activity snapshot/binding failure is H-005 |
| activity sleep | **H-005 open** |
| evidence rebinding | Historical evidence stays source-bound; one-shot receipt overclaim is H-006 |
| hidden solver/state | No solver exists to realize down-capacity; H-004. No source state exists to clear only phase pressure; H-003 |

## 6. Verification performed and omitted

Performed:

- selective Ballast recall and direct authority/source reads;
- Git HEAD/status inspection;
- independent arithmetic reproduction of the asymmetric capacity witness;
- external script/result SHA-256 reads and standards-compliant JSON parse;
- static producer/consumer, pass-order, binding, activity/sleep, pressure,
  rupture and movement-stencil inspection.

Intentionally omitted:

```text
proof process execution: 0
Rust/Cargo/test/check/clippy: 0
WGSL/Naga/GPU/device: 0
workspace FULL: 0
build/launch/runtime: 0
fixture/performance/product: 0
Wiki edits: 0
other repository writes: 0
```

## 7. Final disposition

Unresolved Critical: **0**

Unresolved High: **6**

The predeclared reachable-capacity failure alone invokes D-020's stop rule.
The internal-EMPTY vent conflation, irreversible phase-pressure provenance,
unreachable eight-neighbour capacity, activity/sleep data-path conflict and
one-shot evidence overclaim independently prevent an architecture pass.

Exact disposition:

**TE-5C DESIGN BLOCKED / ADR-0008 PROPOSED / ARCHITECTURE REVISION REQUIRED /
RUNTIME NOT STARTED.**

Per D-020, the next decision must explicitly permit persistent phase-volume
state. No third stateless token/impulse attempt, silent formula substitution,
runtime implementation, evidence rebinding or G5 gate advancement is
authorized by this review.
