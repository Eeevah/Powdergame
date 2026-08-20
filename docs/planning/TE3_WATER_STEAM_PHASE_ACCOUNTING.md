# TE-3 Water / Steam Phase Accounting

- **Status:** DESIGN REQUIRED / NOT STARTED
- **Registered from:** direct Sandbox observation after TE-2 review
- **Audited runtime source:** `fb7e568e21012b6067269f4e1b82c36c865023d0`
- **Runtime implementation authorized:** no

This document registers the Water/Steam design blocker that must close before
TE-3 runtime work. It does not retune thresholds, change yield, add phase state,
or select an implementation representation.

## 1. Direct user observation

Air transport and cooling are visibly active in Sandbox. Steam rises and can
cool into Water, so the causal direction is understandable. The resulting
motion is not acceptable phase presentation or accounting:

- rising Steam and falling Water interleave;
- large blue/white checkerboard clumps remain suspended in mid-air;
- the cause can be followed, but the resulting volume and shape are unnatural.

This is TE-3 design input. It is not a request to retune TE-2 Air-flow or
thermal coefficients.

## 2. Current round-trip audit

The current production path is:

```text
Water above 100 C
-> the source identity becomes Steam
-> matter_yield = 2 requests one additional local Steam Cell
-> with an available receiver, the result is up to 2 Steam Cells

Each Steam below 95 C
-> its own identity becomes one Water Cell
```

`engine/core/src/phase.rs` owns the thresholds and yield metadata.
`phase_transition.wgsl` performs the source identity transition and requests
one expansion receiver. `expansion_spawn_commit.wgsl` copies the resulting
Steam identity into that receiver. The identity transition preserves the
source temperature, and the expansion Cell receives the source temperature.
There is no latent-heat debit/credit and no per-Cell phase-progress state.

Therefore, when local space is available:

```text
1 Water -> up to 2 Steam -> up to 2 Water
```

A closed boil/condense cycle can increase Water-equivalent Cell count. That
violates the required closed-cycle accounting contract and can contribute to
the observed persistent checkerboard volume.

## 3. Blocking TE-3 requirements

TE-3 must define and test all of the following before implementation approval:

- `WATER_STEAM_CLOSED_CYCLE_NO_NET_MATTER_GAIN`
- `WATER_STEAM_REVERSAL_ACCOUNTING`
- `BOILING_VOLUME_EXPANSION_ACCOUNTING`
- `CONDENSATION_VOLUME_CONTRACTION`
- `LATENT_ENERGY_REVERSAL`
- `SURFACE_BOILING`
- `COLD_SURFACE_CONDENSATION`
- `CONDENSATION_NUCLEATION`
- `NO_MIDAIR_PHASE_TRAFFIC_JAM`

The accounting must distinguish conserved Water-equivalent quantity from
occupied Cell volume. Expansion and contraction cannot be represented as
unpaired Matter creation and independent 1:1 condensation.

## 4. Representations that must be compared

No option is selected by this registration.

### A. One Matter Cell plus Environment expansion

Use a 1:1 Water-to-Steam Matter identity transition. Represent boiling volume
expansion through pressure and/or Environment accounting rather than an extra
independent Steam Matter Cell. The design must show how visible expansion,
confinement pressure, reversal, and condensation volume are recovered without
losing quantity.

### B. Primary Steam plus explicit expansion fragment

Keep one primary Steam Matter Cell and represent extra occupied volume with an
explicit bounded expansion-fragment state that remains owned by the primary
quantity. Condensation must contract or merge that state deterministically.
The fragment cannot become an untracked second Water quantity.

### C. Dedicated bounded phase quantity/state

Represent a bounded phase quantity or progress state separately from occupied
Matter identity. Boiling, expansion, nucleation, condensation, contraction,
and latent-energy reversal would consume and restore that accounted quantity.
The proposal must justify storage, ownership, reset, sleep/wake, GPU pass, and
Inspector costs before adoption.

## 5. Required design fixtures

Before TE-3 implementation, the selected proposal must define deterministic
fixtures for at least:

1. one Water quantity completing repeated boil/condense cycles with no net
   Water-equivalent gain;
2. open-surface boiling with accounted visible expansion;
3. sealed boiling with accounted pressure/volume behavior;
4. cooling Steam contracting back to the original quantity;
5. condensation on a cold surface rather than arbitrary mid-air traffic jams;
6. bounded free-air nucleation where no cold surface is available;
7. latent-energy debit on boiling and matching credit on reversal;
8. no negative, non-finite, duplicated, orphaned, or permanently checkerboarded
   phase state;
9. exact reset and Current/Next ownership;
10. CPU reference and production GPU semantic agreement.

## 6. Forbidden shortcuts

Do not use any of these as a TE-3 fix:

- threshold-only retuning;
- random Steam deletion;
- random spreading;
- a movement special-case;
- fake droplet presentation;
- an output clamp;
- hidden global damping;
- disabling condensation;
- changing boiling yield to `1` without closing pressure, volume, and reversal
  accounting.

## 7. Stop boundary

TE-3 remains **DESIGN REQUIRED / NOT STARTED**. This registration authorizes no
phase runtime, latent heat, phase progress, ignition, TE-4, Air-pressure force,
or G9-B/C/D/E implementation.
