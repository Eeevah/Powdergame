# ADR-0012: Integrated ignition exposure and finite chemical heat

- Status: **PROPOSED — DESIGN BLOCKED / USER ARCHITECTURE REVIEW PENDING**
- Date: 2026-08-22
- Decision authority: D-028

## Context

Production currently ignites unlit Oil/Wood on the first tick their own
authoritative temperature reaches `200 C`/`300 C`. Burning continues above
`150 C`/`250 C`, advances the Matter-owned u12 fuel counter once, adds local
temperature `6`/`4`, emits `FLAME_EVENT` and makes at most one Smoke request.
It has no Oxygen input. The final tick that reaches fuel duration consumes the
Cell before heat, flame or Smoke emission.

The live `flags` ownership is bit 0 `COMBUSTING`, bit 1 `FLAME_EVENT`, bits
4..15 fuel progress and bits 16..27 Smoke decay age. Bits 2..3 and 28..31 are
unowned. The combustion GPU table is 16 entries at a serialized 32-byte stride
(512 bytes); the Rust logical upload descriptor has five fields/20 bytes and
the WGSL record has 12 explicit padding bytes. The combustion and base activity
passes each already bind eight storage buffers.

## Proposed decision

Adopt, only after a new evidence authorization, a generic Material-driven rule:

```text
unlit combustible and own T >= ignition threshold
  thermal_rate = min(max_rate,
                     base_rate + floor((T - threshold) / bucket_width))
  flame_rate = min(flame_bonus_cap,
                   previous-snapshot orthogonal FLAME_EVENT count * flame_bonus)
  exposure' = min(budget, exposure + thermal_rate + flame_rate)

unlit combustible and own T < ignition threshold
  exposure' = max(0, exposure - cooling_decay)

exposure' >= budget
  ignite this tick; set COMBUSTING and FLAME_EVENT; exposure = 0
```

The flame term never bypasses the target Cell's own threshold. It reads only
`flags_current` and therefore cannot recurse through newly written same-tick
flames. Connectivity alone has no ignition effect.

The preferred storage candidate is a canonical u6 split across bits 2..3 and
28..31. It adds zero persistent bytes and moves with existing Matter flags.
Every identity replacement, EMPTY, Void, decay, rupture, consumption, Draw,
Erase, preset and reset must clear it. The current `0x0000FFF3` combustible
hygiene mask would erase it and therefore must be deliberately revised in a
future implementation. Reusing fuel progress is rejected because exposure
reverses while fuel progress is irreversible consumed-fuel state.

The preferred chemical accounting converts the Core Material property to a
gameplay energy-like gross source while keeping a prederived GPU delta:

```text
Q_gross_tick = legacy_delta_T * material_heat_capacity
gpu_delta_T = Q_gross_tick / material_heat_capacity
T_next = min(T + gpu_delta_T, max(1200, T))
Q_deposited_tick = material_heat_capacity * (T_next - T)
Q_clipped_tick = Q_gross_tick - Q_deposited_tick
```

Thus Oil proposes gross `15` and Wood gross `8` gameplay-Q per emitting tick,
preserving the existing uncapped self delta. At/above the cap, gross source is
still finite but its clipped portion is not deposited. Preserve the existing
final-tick rule: duration tick 600/900 consumes before emission, so emitting
ticks are 599/899 and maximum **gross** source totals are Oil `8,985`, Wood
`7,192`; deposited totals are bounded by those values and depend on temperature
history. TE-2 never injects gross Q; it may conduct only deposited sensible
heat on later thermal ticks.

## Options

| Option | Disposition |
|---|---|
| Above-threshold tick counter | Rejected as primary: temperature excess has no effect. |
| Integrated excess-temperature dose | Proposed semantic model. |
| Packed u6 in flags | Preferred representation, but evidence not established. |
| Dedicated u32 Current/Next pair | Retained fallback; adds 512 KiB at 256² and 32 MiB at 2048². |
| Reuse fuel progress | Rejected: ambiguous reversible/irreversible ownership. |
| Stateless hysteresis/local threshold | Rejected: cannot express retained dose and decay. |

## GPU feasibility projection

Packed state can remain inside the existing 40-pass/80-query graph. The
combustion pass already reads current material, temperature, flags and writes
next temperature, flags, proposal and material, so orthogonal previous-flame
reads and exposure writes add no binding. Its 12 descriptor padding bytes can
hold three packed u32 metadata words without growing the 512-byte table:

| Byte | Proposed field | Encoding |
|---:|---|---|
| 0 | `is_combustible` | u32; zero is the fail-closed sentinel |
| 4 | `ignition_threshold` | finite f32 |
| 8 | `sustain_threshold` | finite f32 |
| 12 | `chemical_delta_t` | finite non-negative f32 prederived from Core `Q_gross/C`; preserves cap path |
| 16 | `burn_duration_ticks` | u32, 1..4095 |
| 20 | `dose_budget_decay` | budget bits 0..7, decay 8..15, reserved 16..31 zero |
| 24 | `thermal_rates` | base 0..7, max 8..15, bucket-C 16..23, reserved 24..31 zero |
| 28 | `flame_rates` | per-neighbor 0..7, cap 8..15, reserved 16..31 zero |

The future Core `CombustionDescriptor` stores finite `chemical_q_per_tick` as
the Material property. Descriptor compilation looks up that same Material's
positive finite heat capacity and serializes `chemical_delta_t=Q/C` at byte 12;
the GPU therefore needs no ninth storage binding. The current Rust GPU logical descriptor is 20 bytes/alignment 4 in the audited
build but has Rust representation and is manually serialized; it is not the
GPU ABI. The proposed explicit upload record is `#[repr(C)]`, 32 bytes,
alignment 4, and serializes the table exactly in the offsets above. A future
`size_of`/`align_of`, byte-hash and WGSL layout test is mandatory. The activity
pass can bind the existing 512-byte combustion uniform at binding 9 while
retaining its eight storage bindings; no duplicate table allocation is needed.
It uses settled material/temperature/flags, exposure>0, the same descriptor
predicate and orthogonal current flame frontier to keep the next tick runnable.

Compilation fails closed unless: combustible budget is 1..63; decay,
base-rate and bucket width are 1..255; max-rate is base..255; flame per-neighbor
and cap are 0..255 with cap at least the per-neighbor value; duration is
1..4095; ignition/sustain/Q/heat-capacity/derived-delta are finite, Q is
non-negative and heat capacity positive; and every reserved bit is zero.
Non-combustible entries require `is_combustible=0` and all kinetics words,
duration and source values zero. No u8 field is truncated during packing.

The dedicated pair cannot be added directly to either the eight-storage
movement-commit or combustion pass. A viable conservative projection needs at
least an exposure movement-reconcile pass and an exposure/proposal pass before
combustion: 42 passes/84 queries, plus two buffers. It would reuse fully written
`proposal` as an ignition request which combustion consumes and overwrites for
Smoke. Exact identity-hygiene fusion and binding rows remain
`NOT_ESTABLISHED`; this fallback is not implementation-ready.

## Blocker and consequences

The frozen one-shot reference process completed zero trials. Its coefficient
sweep found a metric tie and selected Oil `bucket_width=25,max_rate=4` before
the preregistered `50,6`; the frozen assertion stopped with
`Oil.bucket_width`. The script cannot be repaired or rerun under D-028.
Accordingly no coefficients, state representation or fixtures are accepted,
and TE-4D is **DESIGN BLOCKED**.

Fresh independent review finished at Critical `0` / unresolved High `2`.
Besides the zero-completion blocker, the immutable script can label several
named fixtures without executing their ownership transactions and can produce
a top-level PASS while required items remain `NOT_ESTABLISHED`. Chemical-Q cap
accounting and descriptor range validation findings were resolved in this
document, but they do not repair or rerun the frozen reference.

Vacuum policy is also intentionally undecided: preserve Air-independent
combustion, or add a minimal non-Vacuum Air predicate. Neither means Oxygen.
No runtime work may begin until the user selects a policy and authorizes a new
reference identity that closes the coefficient-selection blocker.
