# ADR-0005 — Separate foreground Matter from atmospheric and vacuum Environment

- **Status:** Accepted design contract; production implementation not started
- **Date:** 2026-08-20
- **Decision:** D-013
- **Implementation entry:** TE-1 only after the checklist in `THERMAL_ENVIRONMENT_IMPLEMENTATION_GATES.md`

## Context

The current runtime treats `EMPTY` as absence of Matter and as no thermal or pressure medium. That contract made the first GPU rules small and measurable, but direct G9-A review exposed a product gap: heat cannot cross ordinary open space, while adjacent hot Matter can drive threshold ignition with little legible exposure time.

Powdergame also needs ordinary atmosphere and future Vacuum to be observably different without turning Air into a selectable Matter or weakening **One Cell = Max One Matter**. The chosen solution must fit the existing GPU-authoritative Current/Next pipeline and the observed eight-storage-buffer compute-stage limit.

## Decision

### Occupancy and Environment are distinct

`material_id` answers only whether foreground Matter occupies a Cell.

```text
EMPTY
  = material_id == EMPTY
  = no foreground Matter

Atmospheric Empty
  = EMPTY + Environment Air present

Vacuum
  = EMPTY + Air at or below the canonical vacuum threshold

Void
  = outside the finite simulation domain
```

Air is an Environment Field. It is not a Material ID, Registry entry, palette item, density-displacement Matter, or a second foreground Matter beneath an occupied Cell. Steam and Smoke remain explicit GAS Matter. The initial slice has no same-cell Air/Matter mixture.

### Canonical state

The full-resolution correctness baseline is:

```text
air_mass_current
air_mass_next
air_energy_current
air_energy_next
```

Air temperature and absolute-like background pressure are derived from finite non-negative mass and energy. They are not separately authoritative.

```text
material_id != EMPTY
→ air_mass == 0
→ air_energy == 0

material_id == EMPTY
→ Atmosphere, low-pressure space, or Vacuum according to air_mass
```

The correctness baseline uses `VACUUM_THRESHOLD = 0`. Vacuum canonicalizes to
exact `(air_mass, air_energy) = (0, 0)`. Positive finite mass is conserved
low-pressure Air even when it is too small to present as ordinary Atmosphere;
it is never silently rounded away. A non-Vacuum Cell must have finite positive
specific energy and a finite derived temperature. A later nonzero numerical
cutoff requires a conservative residual route or a separately approved,
measured source/sink budget.

### Celsius-like gameplay temperature

The product temperature scale is Celsius-like:

```text
about 20  = ordinary room temperature
about 0   = Water / Ice anchor
about 100 = Water / Steam anchor
```

This is not an SI solver. Heat capacity, cell mass, latent heat, ignition dose, and pressure coupling remain gameplay scalars. Current runtime values remain unchanged during TE-0. `TEMPERATURE_REFERENCE`, phase thresholds, placement temperatures, Heat/Cool deltas, combustion thresholds and heat, clamps, authored fixture temperatures, Inspector copy, and UI unit labels must migrate atomically in a later implementation gate. Partial migration is forbidden.

### Volume Exchange

Matter moving from a source into an EMPTY destination exchanges volume with the destination Environment parcel:

```text
before: source Matter; destination EMPTY + Environment
after:  source EMPTY + destination Environment; destination Matter + (0, 0)
```

Matter-to-Matter swaps keep Air canonical zero. Matter leaving the domain makes its source a Vacuum Cell; a later Air-flow stage or boundary reservoir may refill it.

An EMPTY-to-Matter spawn whose source remains Matter requires a separate deterministic Environment receiver claim. The target Air parcel moves to one unclaimed orthogonal EMPTY receiver, combining mass and energy there. Candidate receivers exclude every winning Matter destination in the same stage and must accept the whole parcel without exceeding finite Air mass or energy maxima. If no receiver with full headroom exists, the Matter spawn does not commit:

- phase expansion becomes an explicit Environment-blocked outcome and a
  mandatory separate bounded pass translates that failed Matter expansion
  into the existing phase-pressure result without converting Air mass or
  energy into pressure;
- optional Smoke generation is rejected for that tick;
- no Air is deleted and no new Air-to-mechanical-pressure conversion is introduced in TE-1.

The TE-5 pressure design may later supersede that blocked rule with an explicitly accounted displacement-pressure mechanism. Until then, Air conservation wins over an unreceivable spawn.

The exact TE-1 transaction uses one new full-world `u32`
`environment_receiver_claim` scratch. A receiver invocation derives the one
preferred receiver of each neighbouring winning Matter target from the still
live source-to-target claim, excludes all claimed Matter destinations, applies
mass/energy headroom, and writes the lowest target index. Matter spawn commits
only when that receiver records its target identity. Environment reconcile
then uses the original Matter claim plus the receiver claim to move the entire
parcel. No clamp or partial transfer is permitted. Matter and Environment Next
are settled together; failure leaves both unchanged. The original
`proposal`/`claim` remain live until expansion-pressure or Smoke ownership has
consumed them, so they are not reused for this scratch.

Player Draw and Erase are explicit authoring sources/sinks, not physical conservation events. Draw clears Environment under accepted new Matter. Erase creates EMPTY and seeds the selected world's default Environment; a future Vacuum operation is separate from Erase and separate from the Matter palette.

### Stage and binding boundary

Environment buffers are not appended to current max-eight-storage-buffer passes. Each occupancy-changing causal stage is followed by a separate Environment reconcile stage and an explicit joint settle before the next causal reader. Expansion and Smoke use receiver arbitration, a receiver-gated Matter commit, an eight-storage Environment reconcile, and joint settle. After the existing expansion-pressure pass and before pressure settle, mandatory `environment_blocked_expansion_pressure` reads material, temperature, phase table, proposal, original claim, receiver claim and read/writes `pressure_next`—seven storage bindings plus uniform params. It adds exactly the existing blocked-expansion source when a Matter target won but its Environment receiver did not. It is not Air pressure and does not consume Air mass or energy.

One authoritative writer is required per settled causal stage. A field may have later writers only after the previous Next state has been settled to Current. A separate identity-hygiene pass, below the binding ceiling, sanitizes Matter-owned flags before joint settle; Environment reconcile never pretends to own flags.

Existing `proposal`/`claim` scratch may be wrapped and reused only when its live range has ended and a structural lifetime test proves the reuse. `cell_activity` is not Environment scratch. New passes extend the profiler; their work must not disappear into residual timing.

### Pressure meanings remain separate

The design distinguishes:

- atmospheric absolute-like pressure derived from Air mass and energy;
- Vacuum pressure, which is zero;
- existing mechanical/gauge overpressure in `pressure[]`;
- structure force from pressure differences across faces.

TE-1 through TE-4 do not blindly add Air pressure and `pressure[]`, and do not retune rupture. A standard Atmosphere next to ordinary Liquid/Gas at zero mechanical overpressure must produce no false differential. Coupling begins only at TE-5 after dedicated fixtures and a user decision on Vacuum combustion support and edge policy.

### Full-resolution baseline and memory

| World | One f32/u32 buffer | Four Environment buffers | Receiver-claim scratch | Existing tracked no-profiler state | New correctness baseline |
|---|---:|---:|---:|---:|---:|
| 256×256 | 262,144 B | 1,048,576 B | 262,144 B | 2,886,144 B | 4,196,864 B |
| 2048×2048 | 16,777,216 B | 67,108,864 B | 16,777,216 B | 184,576,128 B | 268,462,208 B |

The one extra `u32` scratch is included because the live original ownership
claim cannot be overwritten before the Matter and blocked-pressure consumers
finish. No second Environment scratch is authorized. Coarse Environment grids,
packing and f16 are optimization experiments, not correctness architecture.

## Supersession boundary

This ADR narrowly supersedes the earlier statements that ordinary `EMPTY` can never carry an Environment medium and that Air, if needed, must be a Matter. It does not erase the historical baseline or change the current runtime during TE-0.

Preserved decisions include One Cell = Max One Matter, finite Void, GPU authority, Read Neighbors → Write Self, bounded ownership arbitration, loose causal phases, minimum sufficient physics, and no speculative universal state.

The open Option A/Option B section in `planning/THERMAL_TRANSPORT_IGNITION_CAUSALITY.md` is superseded by D-013 and this ADR. Neither a lone hidden `temperature[]` value nor a standalone `ambient_temperature[]` is the selected architecture.

## Rejected alternatives

### Air as registered Matter

Rejected because it occupies the foreground Cell, infects every Matter movement with a normal Matter swap, and cannot represent Atmosphere/Vacuum thermal-pressure state without further hidden state.

### EMPTY temperature or one ambient-temperature field

Rejected because temperature alone cannot distinguish equal-temperature standard Atmosphere from low pressure or Vacuum, and moving temperature without mass can create energy from transport.

### Air pressure plus temperature as authoritative state

Rejected because vent/refill cannot distinguish heating from amount transfer cleanly and energy advection becomes ambiguous.

### Coarse Air grid or full CFD first

Rejected for the correctness baseline. Both combine architecture with optimization, complicate one-cell wall sealing, and exceed the approved initial scope.

## Consequences

The design can support open-space cooling, Vacuum insulation, Steam cooling/condensation, sealed heating, and later pressure differential using common local rules. It adds 64 MiB at 2048², new reconcile/settle passes, activity integration, staging/reset work, and new causal evidence requirements.

TE-1 is **READY / NOT STARTED** only after TE-0R, TE-0A and TE-0B are complete. This ADR does not authorize Rust, WGSL, layout, fixture, executable, or runtime changes.
