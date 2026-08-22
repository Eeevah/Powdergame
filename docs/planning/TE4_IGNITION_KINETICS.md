# TE-4D Ignition Kinetics Plan

Status: **V2 EVIDENCE REPAIR AUTHORIZED / RUNTIME NOT STARTED**.

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

Packed u6 is selected by D-029 and adds zero persistent/scratch bytes. Source
binding limits require two future logical passes, producing a conservative
42-pass/84-query projection. It
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

## Selected v2 identity

D-029 fixes Oil `48/2/50/6/1/2/4`, Wood `60/1/50/5/1/2/4`, packed u6,
non-Vacuum orthogonal EMPTY Air-face access, Oil/Wood gross Q `15/8`, and the
consume-before-emission final tick. A manifest-bound reference must execute 13
required fixtures while four production fixtures remain `NOT_ESTABLISHED`.

## Required next decision

Do not implement. After the exactly-once v2 reference and fresh independent
review, the user must accept or revise ADR-0012 before runtime work.

## Lesson promotion

`LESSON_PROMOTION: PROJECT_ONLY`. PG-L034 records the new coefficient-selection
identity/tie-policy preflight guard. The verified Wiki already contains the
general one-shot preflight rule, and its local checkout is user-dirty and was
required to remain read-only; no duplicate Wiki edit or PR is created.
