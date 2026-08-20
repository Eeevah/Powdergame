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
