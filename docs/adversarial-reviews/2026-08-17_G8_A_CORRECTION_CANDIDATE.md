# Adversarial Review Report — G8-A Correction Candidate

> **Superseded provenance note (2026-08-17):** A later user-supplied review of the collected evidence packet found that the v4 CSVs are not bound to the later source snapshot or executed binary. The packet also lacks the three raw census arrays. Statements below that v4 was generated after the final queue-fence correction must not be used as evidence of that linkage. The original text is otherwise retained as a historical record.

- **Date**: 2026-08-17
- **Recipient**: Powdergame user/owner
- **Reviewer**: Codex local worktree review
- **External AI used for this report**: no
- **Record status**: historical and non-blocking; no user response is required
- **Worktree HEAD**: `a67abaf959aba0423627f35b79fce7c82d8ec9b5`
- **State**: detached HEAD with dirty, uncommitted correction changes
- **G7 frozen baseline**: `94babb2667c081b5588489e1b4e710cc6efa68be`
- **Calibration run**: `g8a-1786916099569`

## Historical Verdict (Superseded for Provenance)

**PASS AS A LOCAL CORRECTION CANDIDATE**

This report originally recorded unresolved findings as **A/B/C/D = 0/0/0/0**. That statement predates the later packet review and is not the current evidence state.

This is not an official frozen baseline. Immutable provenance remains pending, and no commit, push, PR, or release was created. The report itself requires no user disposition.

## Reviewed Scope

- production/profiled tick orchestration and timestamp integrity;
- persistent tracked-buffer accounting;
- benchmark CLI and dimension-safe fixture construction;
- Mode A/Mode B timing boundaries;
- per-tick grouped statistics and raw evidence schema;
- fatal evidence-write behavior;
- v4 aggregate/raw artifacts and documentation claims.

## Adversarial Findings Resolved During the Task

### 1. Pending fixture uploads entered measured windows — resolved

`Queue::write_buffer` transfers do not begin until a queue submission. The earlier helper called `device.poll(Wait)` without first submitting the scheduled reset/fixture writes, allowing roughly 272 MiB of setup copies to enter the first timed tick.

Resolution:

- `reset_stage_and_wait()` now calls `queue.submit([])` before `PollType::Wait`;
- every Mode A, Mode B, and overhead-control window uses that helper before timing;
- v2 and v3 measurements are explicitly invalidated;
- this historical report stated that v4 was generated after the corrected fence; the later packet does not contain the source/binary/log linkage needed to establish that statement.

### 2. Earlier measurement-integrity defects — resolved

- all 14 persistent simulation buffers are included in the 2,176-byte static inventory;
- evidence directory creation, file creation, writes, flushes, and canonicalization propagate failures;
- timestamp conversion rejects invalid periods and equal, inverted, or cross-pass-disordered raw timestamps;
- envelope, pass sum, and residual use checked integer-domain reconstruction;
- group percentiles are computed from per-tick group sums rather than sums of pass medians;
- the fixture scales safely and clears aliased fire state at small supported dimensions;
- raw schema v3 names every pass start/end pair and records explicit group membership.

## v4 Evidence Check

- Aggregate CSV: 129 rows, SHA-256 `D25689FB23DA2E0FDBDB9157EA13A49096A3EDBDF68566CD725BD56AD60CC144`.
- Raw CSV: 768 rows and 768 unique sample identities, SHA-256 `3F38944753CD726A68555F384794C3FA8FD89404A958E84E2419352D66863D2A`.
- One schema and run provenance set.
- All 17 pass timestamp pairs are positive and ordered within and across passes.
- Maximum envelope reconstruction error: `0 ms`.
- Maximum pass-sum and residual reconstruction error: `2.22e-16 ms`.
- Maximum group reconstruction error: `5.55e-17 ms`.

Reference results:

- Mode A P50: 948.9 TPS / 1.0538 ms per tick.
- Mode B selected trial: envelope P50 1.0204 ms; pass-sum P50 0.7663 ms; residual P50 0.2542 ms.
- Application-tracked persistent buffers with profiler buffers: 184,576,672 bytes.
- Overhead controls: synchronization +27.00%; profiling increment +1.77%; combined path +29.25%.

## Verification Boundary

Final closeout checks:

- `cargo check -p powdergame-benchmark`: pass;
- `cargo fmt --all -- --check`: pass;
- `git diff --check`: pass;
- one required corrected v4 calibration run and local artifact reconstruction: pass.

Earlier in the correction round, the benchmark tests passed 15/15 and targeted clippy checks passed. No additional broad smoke matrix was run, in accordance with the user's minimal-test policy.

## Remaining Risks and User Decisions

- The worktree is dirty and detached, so v4 is local evidence rather than an immutable official baseline.
- G8-B's five official scenarios and G8-C's immutable matrix measurement are not started.
- Gate closure remains a user decision.
- Any commit, push, PR, or release requires explicit user authorization.

## Historical Note

Before the user changed the review policy, GPT Pro was consulted during this work session. Its record remains historical evidence only. The user subsequently removed adversarial review from the default workflow; no further review or report is required unless explicitly requested.
