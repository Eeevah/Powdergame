# TE-4D v3 ignition-kinetics design — independent adversarial review

- Date: 2026-08-22
- Scope: frozen v3 manifest, independent oracle generator/data, evidence script/result, and live source feasibility
- Reviewer independence: fresh context; authored none of the reviewed artifacts
- Execution boundary: read-only static review; no evidence, runtime, Cargo, GPU, build or application execution
- Verdict: **DESIGN BLOCKED**
- Findings: Critical `0`; unresolved High `3`; Medium `1`; Low `1`

## Positive controls

- The frozen oracle contains complete F07/F08, near-budget and symmetric-tie event lists and is independently structured from the evidence model.
- Self-replay is labelled determinism-only; coefficient optimality is explicitly not claimed.
- Packed bits do not collide in the proposed representation, although future hygiene changes remain required.
- Both projected new passes remain at six storage bindings and the current proposal producer fully writes its output.
- Current burning activity and post-Smoke settle ordering provide a plausible future wake/visibility route.
- Environment receiver gating is conservation-shaped and no production runtime source changed.

## High findings

### H-001 — F15B next-snapshot Air loss is asserted, not derived

The stage-N+1 predicate is assigned as the literal
`stage_n1_snapshot_air = False`, followed by an unconditional extinguish
transaction. No topology query derives the next `COMBUSTION_STAGE_SNAPSHOT`
Air predicate from the post-Smoke WorldState. The fixture therefore cannot
establish that own-Smoke removal of the sole Air face causally triggers the
next-stage extinguish. The reported F15B PASS and `sole_air_face_snapshot_fixture`
aggregate are invalid as architecture evidence.

Resolution required: a future identity must derive both stage snapshots from
the same immutable WorldState topology/Air predicate used by the transition,
and the transition must branch on that derived value. V3 is frozen and may not
be repaired or rerun.

### H-002 — Auditor trusts SUT-supplied semantic identity

The auditor computes field deltas, but accepts the caller-provided transaction
name and the transaction-returned event ID (`required_event in events`). The
`MATTER_OWNED_*` ownership strings have no semantic audit branch. Consequently
an arbitrary allowed-field mutation plus a claimed event can masquerade as
exposure, ignition, burning, extinguish or reignition. Positive audited-path
counters therefore do not prove the named transaction semantics.

Resolution required: a future auditor must infer transaction class and event
meaning from before/after state plus an independent specification, without
trusting SUT-returned names/events as proof. V3's reported zero-path count does
not close the v2 counter blocker.

### H-003 — F09 chemical-Q/final-tick result is not state-machine-derived

F09 invokes one arbitrary `heat=10` burning mutation, then a separate
unconditional fuel-consumption transaction. Oil/Wood gross totals are literal
`15*599` and `8*899`, while `final_tick_emission=0` is a reported constant.
No duration-driven lifecycle selects the consumption tick, suppresses its
emission, or derives gross/deposited/clipped closure from the same transaction
state machine. Chemical-Q and consume-before-emission are therefore not
established by the mutation receipts.

Resolution required: a future identity must drive Oil and Wood through their
complete duration state machines and audit each emission/consumption delta.
V3 cannot be re-executed.

## Other findings

### M-001 — Published receipts omit immutable snapshots

The result publishes computed delta summaries but not the actual immutable
before/after snapshots. A third party cannot independently re-audit the
receipts from the result alone. Future evidence should include compact state
snapshots or content-addressed snapshot records.

### L-001 — Planning tail remained stale during review

The plan still requested the already-made D-030 decision and retained the old
v2 lesson disposition. Closure must replace that tail with the v3 blocked
result and `LESSON_PROMOTION: NONE`.

## Disposition

Unresolved Critical/High count is `3`. Per D-030:

```text
TE-4D v3: DESIGN BLOCKED
ADR-0012: PROPOSED
TE-4 runtime: NOT STARTED
```

The one v3 process receipt remains immutable narrow history: attempts/
completions `1/1`, reported reference/deferred `13/4`, and frozen exact oracle
matches. Those process facts are not design approval. No lesson promotion is
needed because the reusable precondition-lifetime pattern is already PG-L035
and in the verified Wiki workflow; the new defects are identity-specific
evidence-construction failures.
