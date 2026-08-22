# TE-4D Ignition Kinetics Plan

Status: **DESIGN BLOCKED / RUNTIME NOT STARTED**.

## Reuse and exact inventory

- Immediate-threshold production behavior and Oil/Wood constants remain live.
- `flags`: combustion 0..1 and 4..15; decay 16..27; candidate u6 2..3 + 28..31.
- Movement commit carries flags but is already 8-storage; identity edits clear
  both flag halves; current combustible hygiene mask would erase candidate bits.
- Combustion: 8 storage + 2 uniform bindings, pass 24 of 40; `proposal` is fully
  overwritten then consumed by Smoke claim/receiver/commit.
- Activity propose: 8 storage + 1 uniform, pass 36; property table binding 8 may
  grow without a new binding.
- Descriptor upload: Rust logical 20 bytes, manual 32-byte stride, WGSL 32
  bytes, 16 entries/512 bytes; 12 bytes padding available.
- Profiler: 40 passes / 80 timestamp queries.
- External copied/translated/vendored implementation: 0 files / 0 lines.

## Candidate layouts

Packed u6 keeps 40/80 and adds 0 persistent/scratch bytes and 0 passes. It
requires exact mask, movement, identity, authoring, activity and Inspector
tests. Descriptor padding is proposed as three u32 words: dose budget; packed
base/max/bucket metadata; packed decay/flame/cap metadata. ADR-0012 fixes their
byte offsets and 8-bit subfields. The base activity pass remains at eight
storage bindings and adds the existing combustion table as a second uniform;
the table allocation is reused, not duplicated. Finite/range validation,
serialized table hash and bounded candidate-only exposure diagnostics remain
future implementation work; the normal Inspector contract is not expanded.
Core chemical Q is compiled with the Material heat capacity into the existing
GPU delta-T slot. This avoids a capacity binding and preserves the current cap;
validation must distinguish finite gross Q from deposited and clipped Q.

Dedicated u32 Current/Next adds 524,288 bytes at 256² and 33,554,432 bytes at
2048². Because movement commit and combustion are already at eight storage
bindings, the conservative minimum projection is 42 passes/84 queries using a
movement-reconcile pass and a pre-combustion exposure pass. The latter writes
exposure Next and a fully overwritten ignition request into `proposal`; the
existing combustion pass consumes it before overwriting `proposal` for Smoke.
Identity-hygiene fusion, activity visibility and all binding rows remain
unproven, so this is only a fallback estimate.

## Required next decision

Do not implement. A new user decision must authorize a new reference identity,
resolve equal-metric coefficient tie semantics, choose packed or dedicated
state, accept/revise dose and chemical-Q rules, decide final-tick behavior and
select Air-independent or minimal-non-Vacuum combustion. Then all TE4-F01–F17
must execute against their named layers before a user-testable candidate.

## Lesson promotion

`LESSON_PROMOTION: PROJECT_ONLY`. PG-L034 records the new coefficient-selection
identity/tie-policy preflight guard. The verified Wiki already contains the
general one-shot preflight rule, and its local checkout is user-dirty and was
required to remain read-only; no duplicate Wiki edit or PR is created.
