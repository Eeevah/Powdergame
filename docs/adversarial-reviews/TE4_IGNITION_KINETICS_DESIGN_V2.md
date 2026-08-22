# TE-4D v2 ignition-kinetics independent adversarial review

- Date: 2026-08-22
- Reviewer: fresh-context agent; did not author the v2 manifest, script, result or primary design response
- Review mode: read-only static inspection; no script, test or evidence execution
- Evidence identity: `TE4-IGNITION-KINETICS-REFERENCE-V2`
- Verdict: **TE-4D v2 DESIGN BLOCKED / ADR-0012 PROPOSED / RUNTIME NOT STARTED**
- Findings: Critical `0` / unresolved High `3` / Medium `1` / Low `0`

## Bound inputs

The reviewer inspected D-029, ADR-0012, the specification, validation and plan,
the production pass/binding/ownership source, the immutable v1 script/failure,
the frozen v2 manifest/script/result and the verified Wiki evidence/fixture
integrity contract. V1 remained attempt/completion `1/0` and executed no model
work. The v2 files were not changed or executed during review.

## H-001 — path counters do not prove distinct state transactions

Severity: **High / unresolved**.

`fixture_result()` accepts a fixture when every supplied counter is a positive
integer. Several counters are constants rather than values derived from the
named transaction. F06/F07 same-tick checks are constants; F11 models movement
and density swap as assignment/deep-copy; F12 gives five replacement names to
the same `clear_identity()` helper; F13 models Draw, preset and reset as copies
of one canonical object; F15's Air-conservation count compares an immutable
tuple. The aggregate at the end then reports 13 PASS and
`state_transition_result=PASS` from those statuses.

Smallest reproduction: replace or remove the distinct F11–F13 transaction body
while retaining the positive return counters. The aggregate still passes.
This violates the Wiki named-fixture contract and leaves F06/F11/F12/F13/F15
partly unestablished. Because v2 is frozen after its sole execution, repairing
the script or rerunning under this identity is forbidden. A future identity
must derive each path counter from a distinct pure transaction or downgrade it
to `NOT_ESTABLISHED`.

## H-002 — downstream Smoke can invalidate the only Air face in the same tick

Severity: **High / unresolved**.

D-029 says that a burning Cell losing Air access emits no Heat, Flame or Smoke
that tick. The projection determines Air access in
`ignition_exposure_propose`, before combustion and Smoke commit. Production
Smoke targeting can select an EMPTY Air face without reading Air mass; commit
then occupies that Cell and Environment reconcile moves its Air away. With
exactly one positive-Air EMPTY neighbour, combustion may emit based on the
early snapshot and the same transaction may remove the only qualifying face by
tick end. F15 covers access removed before combustion, not self-Smoke removal.

The user must choose one meaning in a later decision: start-of-combustion
snapshot access with next-tick extinguish; protect the last qualifying face
from Smoke; or recheck/cancel atomically after Smoke. The latter choices may
change proposal encoding or pass order, so the 42-pass/no-new-state projection
cannot be retained without a new audit.

## H-003 — F08 digest is a post-run self-replay, not a frozen oracle

Severity: **High / unresolved**.

The manifest freezes F08 first tick 20, maximum simultaneous count 5 and
completion bound 173, but not the exact event digest or per-tick coordinate
frontier. The evidence computes a digest after execution and compares two calls
to the same implementation. A changed yet deterministic frontier can keep the
three aggregate values and still pass. Thus the recorded digest proves replay
stability, not agreement with a predeclared independent oracle.

A future identity must freeze the complete F08 event digest or exact per-tick
frontier before execution, including the near-budget frontier family. V2's
observed digest remains a receipt, not an oracle.

## M-001 — secondary-objective completeness is not established

Severity: **Medium / unresolved risk; not the blocking reason by itself**.

The script checks the two manifest-listed Oil identities and some profile
properties. It does not enumerate a frozen candidate domain, prove the equal-
primary set complete, calculate the first timing-distance objective, examine
Wood ties or execute the final ambiguity branch. D-029 directly selects the
exact tuple, so identity remains fixed, but `coefficient_result=PASS` must not
be read as proof that no hidden tie exists.

## Attack matrix disposition

- Manifest hash, exact tuple and fixed identity: no mismatch found.
- Packed u6 numeric encode/decode and 64 rejection: no collision/wrap found;
  movement/hygiene/replacement authority remains blocked by H-001.
- Atmosphere/LowPressure/Vacuum/occupied-GAS modeled matrix: consistent; actual
  production predicate remains deferred.
- Q gross/deposited/clipped closure, cap and final consumption tick: no reduced-
  model counterexample found. TE-2 double-source remains production-deferred.
- Six-storage projected new passes, eight-storage ceiling and proposal lifetime:
  arithmetically/source-order consistent, but H-002 may require redesign.
- Historical evidence rebind and runtime leakage: none found. GPU/product/user
  status remains `NOT_ESTABLISHED`/`NOT_ESTABLISHED`/`PENDING`.

## Stop disposition

D-029 requires zero unresolved Critical/High findings. With three unresolved
High findings, the valid stop is **TE-4D v2 DESIGN BLOCKED**. The completed
reference receipt is preserved as narrow historical evidence; it cannot be
relabeled as independent-review PASS or used to authorize TE-4 runtime.

`LESSON_PROMOTION: REQUIRED — DEFERRED / NOT AUTHORIZED FOR WIKI WRITE`. H-002
is a new reusable same-tick precondition-invalidation pattern. PG-L035 records
it in the project ledger; the verified Wiki checkout remains read-only because
this task authorizes only Powdergame docs/memory publication.
