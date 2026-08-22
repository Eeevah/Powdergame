# Adversarial Review Archive

This directory preserves adversarial reviews that were already produced. Adversarial review is optional and is not a required Powdergame closeout step.

## Policy

- Do not automatically request, perform, or file an adversarial review.
- Perform and report one only when the user explicitly requests it.
- Do **not** send Powdergame code, diffs, evidence, or review prompts to GPT Pro, Grok, or any other external AI reviewer.
- Existing reports are historical, non-blocking records; they do not require user disposition before normal development continues.
- A report is evidence for the user's decision. It does not authorize a commit, push, PR, release, deletion, or gate closure.
- Preserve exact provenance: HEAD SHA, clean/dirty state, source snapshot/full dirty diff hash, executed binary hash, exact argv/stdout/stderr/exit code, benchmark run ID, artifact paths/hashes, and the validation boundary. For G8 evidence, use the capture receipt rather than inferring linkage from commit SHA plus `dirty` state.
- Do not hide unresolved findings or convert missing evidence into PASS.
- Respect the minimal-test policy: run only checks genuinely required by the change and state what was not run.

## Report Shape When Explicitly Requested

Each report must include:

1. task and reviewed scope;
2. worktree provenance;
3. current A/B/C/D finding counts;
4. adversarial failure modes examined;
5. findings fixed during the task;
6. verification performed and intentionally omitted;
7. remaining risks, missing evidence, and user decisions;
8. final verdict.

Use `YYYY-MM-DD_<GATE_OR_SCOPE>.md` filenames. If a requested report later becomes stale, create a new report and mark the earlier one superseded rather than silently rewriting its conclusion.

## Severity

- **A** — correctness, data-loss, or evidence-integrity blocker;
- **B** — material reliability risk that blocks acceptance;
- **C** — real but non-blocking defect or operational weakness;
- **D** — clarity, maintainability, or documentation issue.

## Explicitly requested design reviews

- [`TE3_PHASE_ENTHALPY_DESIGN.md`](TE3_PHASE_ENTHALPY_DESIGN.md) — preserved v1
  review plus fresh-context D-018 v2 review; the non-date filename is the
  user's explicit requested output name. Current disposition: **INDEPENDENT V2
  DESIGN REVIEW PASS — UNRESOLVED CRITICAL 0 / HIGH 0**. TE-3D is subsequently
  recorded as **ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS**.
- [`TE5_PHASE_VOLUME_PRESSURE_BRIDGE_DESIGN.md`](TE5_PHASE_VOLUME_PRESSURE_BRIDGE_DESIGN.md)
  — D-019 fresh-context review of the exclusive local volume-relief-token
  candidate. Current disposition: **TE-5B DESIGN BLOCKED — UNRESOLVED CRITICAL
  0 / HIGH 1**. The staggered-heating vacancy-walk counterexample prevents the
  non-mutating 1:1 token from guaranteeing finite-headspace exhaustion.
- [`TE5_LOCAL_VAPOR_CAPACITY_PRESSURE_DESIGN.md`](TE5_LOCAL_VAPOR_CAPACITY_PRESSURE_DESIGN.md)
  — D-020 fresh-context review of the final no-new-persistent-state capacity-
  pressure candidate. Current disposition: **TE-5C DESIGN BLOCKED —
  UNRESOLVED CRITICAL 0 / HIGH 6**. The locked sharing law fails an open-
  capacity witness, and five additional architecture/evidence blockers remain.
- [`TE5_PRESSURE_VOLUME_MODEL_COMPARISON.md`](TE5_PRESSURE_VOLUME_MODEL_COMPARISON.md)
  — D-022 fresh-context comparison review. Current disposition: **TE-5X DESIGN
  BLOCKED — UNRESOLVED CRITICAL 0 / HIGH 11**. The combined proof completed no
  candidate evaluation, and A/B/C each retain independent architecture blockers;
  there is no Recommendation or Retained fallback.
- [`TE3Q_CONSERVATIVE_PHASE_PACKETS_DESIGN.md`](TE3Q_CONSERVATIVE_PHASE_PACKETS_DESIGN.md)
- [`TE4_IGNITION_KINETICS_DESIGN.md`](TE4_IGNITION_KINETICS_DESIGN.md)
- [`TE4_IGNITION_KINETICS_DESIGN_V2.md`](TE4_IGNITION_KINETICS_DESIGN_V2.md)
- [`TE4_IGNITION_KINETICS_DESIGN_V3.md`](TE4_IGNITION_KINETICS_DESIGN_V3.md)
  — D-029 fresh-context review. **Critical 0 / unresolved High 3 / Medium 1**;
  positive path counters, same-tick Smoke/Air-face loss and post-run F08 digest
  block the v2 design. Runtime remains not started.
  — D-023 fresh-context packet review. Current disposition: **TE-3Q / TE-5Q
  DESIGN BLOCKED — UNRESOLVED CRITICAL 0 / HIGH 8 / MEDIUM 1**. The reduced
  proof under-models named fixtures; local contraction and spatial pressure
  retain independent causal/source-integration blockers.
