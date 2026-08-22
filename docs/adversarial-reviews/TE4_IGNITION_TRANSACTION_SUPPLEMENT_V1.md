# TE-4D targeted transaction supplement — fresh adversarial review

- Date: 2026-08-22
- Reviewed identity: `TE4-IGNITION-TRANSACTION-SUPPLEMENT-V1`
- Disposition: **TE-4D TRANSACTION SUPPLEMENT BLOCKED**
- Findings: Critical `0` / High `3` / Medium `2` / Low `0`
- ADR-0012: **PROPOSED**
- TE-4 runtime: **NOT STARTED**

The reviewer authored none of the manifest, script, snapshots or result and
performed a read-only static review. The supplement, tests and application
were not executed by the reviewer. The frozen one-shot result is preserved as
a process receipt; its internal PASS fields do not override the findings below.

## High findings

### H-001 — F15B omits the required Matter/Air settle

`run_f15b` performs Smoke commit and Air displacement, then immediately derives
stage-N+1 Air access and runs combustion. Those mutations update `current` only;
there is no Matter/Air `next -> current` settle transaction. Snapshot records 60
and 61 consequently retain `current`/`next` disagreement for the Smoke target,
receiver and source fuel. The predicate is topology-derived, but it reads a
reduced world that is not the required settled post-Smoke WorldState. V3 H-001
is therefore not closed.

### H-002 — Transaction identity remains caller-classified

The harness supplies `spec_id` to `record` and the auditor selects the expected
transition by that value. Ignition, active burn, extinguish, reignition and fuel
consumption share one expected-combustion branch; decay, rupture, Void and
generic replacement share another. The manifest `kind` is not used to infer
the semantic class from the delta. Exact state equality is useful, but it does
not prevent one delta from being counted under a caller-selected semantic name.
V3 H-002 therefore remains open.

### H-003 — Air displacement lacks receiver topology and claim semantics

The Air audit accepts arithmetic transfer between arbitrary target/receiver
indices. It does not require adjacency, an eligible EMPTY receiver, a receiver
claim or linkage to the Smoke transaction. The semantic matrix even accepts a
diagonal 3x2 target-to-receiver transfer. Conservation alone therefore does not
establish the production Environment receiver transaction.

## Medium findings

### M-001 — Negative-control coverage is overstated

The manifest names 13 negative-control families, but records carry no family
identifier and the generator creates only one generic corruption plus one
removed-body control per transaction class. The 38 rejected records therefore
do not establish every named family. `negative_control_rejections:
ALL_REQUIRED` is an overclaim.

### M-002 — The named third-party re-audit is self-re-audit

Execution calls `reaudit` from the same script, which calls the same
`audit_transition` implementation. This validates parsing, record hashes and
self-consistency, but it is not a third-party independent semantic audit.

## Confirmed narrow results

- Air access is computed from orthogonal Current topology; no literal
  `air_access` or `next_air_access` input exists.
- The Oil and Wood lifecycles use the same state machine for 600 and 900 ticks.
- Snapshot-derived emitting totals are Oil `599 * 15 = 8,985` and Wood
  `899 * 8 = 7,192`; the final ticks consume with zero Heat/Flame/Smoke/Q.
- Below/crossing/at/above-cap records close `deposited + clipped = gross` and
  the above-cap control does not reduce temperature.
- The bundle contains 1,565 full reduced-model before/after records: 1,527
  accepted and 38 rejected by the script's auditor.
- Manifest/script/snapshot/result-file hashes are respectively
  `03549f3b...918295`, `6ee23ebc...d557162`, `56398994...7f8423` and
  `54bc5281...146a2e`.
- GPU and product remain `NOT_ESTABLISHED`; user status remains `PENDING`.

## Verdict and remaining boundary

**TE-4D TRANSACTION SUPPLEMENT BLOCKED — Critical 0 / High 3 / Medium 2.**
The supplement materially improves topology, lifecycle and snapshot receipts,
but it does not close v3 H-001/H-002/H-003. ADR-0012 remains Proposed and
TE-4 runtime remains Not Started. A later user decision must authorize a new
identity; this frozen supplement must not be patched or rerun.

`LESSON_PROMOTION: NONE` — the failures are already covered by the Wiki
Evidence/fixture integrity and Snapshot precondition lifetime contracts and do
not add a new reusable rule beyond PG-L034/PG-L035.
