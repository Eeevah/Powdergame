# Ballast Memory Cutover and Rollback

Status: **USER-APPROVED ACTIVE WORKFLOW / INTEGRATION STAGED IN PR #4**

## 1. Decision

Powdergame adopts Ballast as the single active **session-continuity** workflow.

Canonical resume order:

1. `memory/00-INDEX.md`
2. `memory/CHECKPOINT.md`
3. active entries in `memory/DECISIONS.md`
4. only the task-relevant canonical documents linked by the index

This cutover replaces the old practice of maintaining current session coordinates in several places. It does **not** replace the project's domain authorities.

| Domain | Authority after cutover |
|---|---|
| Current session coordinate and one next action | `memory/CHECKPOINT.md` |
| User-confirmed decisions and supersession | `memory/DECISIONS.md` |
| Pending choices | `memory/OPEN-QUESTIONS.md` |
| Milestone/gate/evidence router | `docs/planning/STATUS.md` |
| Immutable evidence and human dispositions | `docs/evidence/**` |
| Architecture and implementation contracts | ADRs, `docs/architecture/**`, `docs/specs/**` |
| Validation and evidence reuse | `docs/development/VALIDATION_POLICY.md` |
| Promoted technical/process lessons | `docs/development/LESSONS_LEDGER.md` |
| Historical/domain recovery narrative | `docs/HANDOFF.md` |

`docs/HANDOFF.md` is preserved. It is no longer updated after every session as a competing checkpoint. Existing domain/history content remains available for targeted reading.

## 2. Commit boundary

Initial pilot commit:

```text
ba2b6406f6605882c51886b0a50bc64d10990a7f
# docs: add isolated Ballast memory pilot
```

Active cutover is a later separate commit with subject:

```text
docs: adopt Ballast as primary project memory
```

Canonical product/evidence decisions, including Heavy Mixed acceptance, are not part of the Ballast rollback unit. They remain on the product line even if Ballast is removed.

### Merge rule

**Squash merge is forbidden.** It destroys selective rollback.

Allowed:

- rebase-and-merge that preserves the pilot and active-cutover commits;
- a merge commit that preserves both commits.

The cutover PR must not be merged while another writer is actively advancing the target branch without first exact-fetching and reconciling the live history.

## 3. Normal update rule

For an ordinary development session:

- update `memory/CHECKPOINT.md` when a substantial unit ends or the next action changes;
- append user decisions immediately to `memory/DECISIONS.md` and then propagate them to the relevant canonical domain document;
- update `docs/planning/STATUS.md` only when milestone/evidence state actually changes;
- append compact history to `memory/SESSION-LOG.md` rather than growing `docs/HANDOFF.md` as a live session diary;
- archive the replaced checkpoint under `memory/checkpoints/` when a major phase changes;
- never copy raw evidence, credentials, generated artifacts, or complete telemetry into memory.

## 4. Immediate disable — seconds, no Git change

If rule injection or a Hook causes trouble, disable it before diagnosing Git state.

PowerShell:

```powershell
$env:BALLAST_DISABLE = "1"
```

Alternatively, remove trust from the Ballast Hook in Codex `/hooks`.

This stops new Ballast rule injection without modifying project files or evidence.

## 5. Project rollback

After stopping the Hook, revert the project cutover in reverse order.

```powershell
git revert <SHA-of-commit-with-subject-docs-adopt-Ballast-as-primary-project-memory>
git revert ba2b6406f6605882c51886b0a50bc64d10990a7f
```

The first revert removes the active-workflow contract and restores the prior HANDOFF/resume expectations. The second removes the pilot `AGENTS.md` and `memory/**` initialization.

Do **not** revert unrelated canonical product/evidence commits. Heavy Mixed acceptance, G8-B/G8-C results, and other domain decisions remain intact.

Because the old domain documents were never deleted, rollback does not reconstruct them from memory.

## 6. Global Codex rollback

Project rollback is usually enough. Remove the global installation only if Ballast causes cross-project problems.

Fastest global stop:

```powershell
$env:BALLAST_DISABLE = "1"
```

Then close Codex and restore the appropriate timestamp backup, including the known installation backup family such as:

```text
C:\Users\mdkap\.codex\AGENTS.md.backup.<timestamp>
```

Review and remove only the Ballast-managed resources:

```text
C:\Users\mdkap\.agents\skills\ballast-*
C:\Users\mdkap\.codex\hooks\ballast-rules.mjs
Ballast handler entry in C:\Users\mdkap\.codex\hooks.json
Ballast managed block in AGENTS.md or AGENTS.override.md
Ballast-owned rule IDs in C:\Users\mdkap\.claude\ballast.rules.json
```

Do not delete an entire shared rules file when only specific Ballast-owned IDs need removal. Use the timestamp backups and the Wiki installation workflow as the source of truth.

## 7. Rollback triggers

Immediately disable the Hook and evaluate rollback if any serious event occurs:

- a new session follows an older document instead of the current checkpoint;
- valid same-SHA receipts exist but FULL/GPU/candidates are repeated without a changed validity input;
- a stale checkpoint is presented as current despite contradictory live Git/runtime state;
- an agent proposal is recorded as a user decision;
- memory maintenance becomes larger than the development task it supports;
- Hook errors block otherwise valid Codex requests;
- memory causes destructive Git operations or hides a dirty worktree;
- two live session-coordinate systems are being maintained in parallel.

One severe correctness/safety failure is enough for immediate rollback review. Repeated non-blocking friction is enough to revert the active cutover after preserving any useful product-domain commits.

## 8. Verification after rollback

Confirm:

- `AGENTS.md` no longer injects the project Ballast workflow;
- `memory/` is absent or intentionally retained only as historical, non-active material;
- `docs/HANDOFF.md` again identifies the active resume path;
- `docs/planning/STATUS.md`, evidence, ADRs, specs, validation policy, and lessons remain unchanged unless independently intended;
- no runtime test or evidence recapture was triggered solely by rollback docs;
- Git history still contains the reverted commits for auditability.

## 9. Related Wiki authority

The installation, supply-chain lock, backups, and cross-project rollback procedure are maintained in `personal-infra-wiki` under the Codex Ballast workflow and Powdergame adoption decision. The Wiki is the source of truth for the global installation; this repository is the source of truth for the Powdergame-specific cutover.