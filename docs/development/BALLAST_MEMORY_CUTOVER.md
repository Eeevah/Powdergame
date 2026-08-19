# Ballast Memory Cutover and Rollback

Status: **USER-APPROVED / INTEGRATED / ACTIVE**

## Active workflow

Powdergame uses Ballast as its single active session-continuity workflow.

1. `memory/00-INDEX.md`
2. `memory/CHECKPOINT.md`
3. active entries in `memory/DECISIONS.md`
4. only the task-relevant canonical documents linked by the index

Domain authority is unchanged:

| Domain | Authority |
|---|---|
| Current session coordinate and one next action | `memory/CHECKPOINT.md` |
| User-confirmed decisions and supersession | `memory/DECISIONS.md` |
| Pending choices | `memory/OPEN-QUESTIONS.md` |
| Milestone/gate/evidence router | `docs/planning/STATUS.md` |
| Immutable evidence and human dispositions | `docs/evidence/**` |
| Architecture/specification contracts | ADRs, `docs/architecture/**`, `docs/specs/**` |
| Validation and evidence reuse | `docs/development/VALIDATION_POLICY.md` |
| Promoted lessons | `docs/development/LESSONS_LEDGER.md` |
| Pre-cutover recovery narrative | Git history of `docs/HANDOFF.md` |

Memory links to proof; it does not copy or replace proof. Live Git/runtime wins when a checkpoint is stale.

## Integrated Git boundary

- Initial pilot commit: `ba2b6406f6605882c51886b0a50bc64d10990a7f`
- Active cutover commit: `8d21756f3dfa5c6a743f0aa03108153bb4b206df`
- Final pre-integration memory head: `4f5e910f6a4f27548f7f0b41f21e69b80996ec93`
- Product/evidence first parent: G8-C sealed source `4653d7c2e09e93f80fb81eeb73458d992c86858f`
- Integration merge commit: `6b5f0201f882f212f9916521aec689261d97b4a6`
- Merge method: merge commit; **not squash**

Heavy Mixed acceptance, G8-B closure, G8-C source/evidence and later product decisions remain on the first-parent product line. They are not removed by reverting the Ballast merge.

## Normal update rule

- Routine session end: update `memory/CHECKPOINT.md`; append a compact `SESSION-LOG` entry only when useful.
- User decision: append to `memory/DECISIONS.md`, then propagate it to the relevant canonical domain document.
- Actual milestone/evidence change: update `docs/planning/STATUS.md` and the relevant evidence record.
- Major phase change: archive the replaced checkpoint under `memory/checkpoints/`.
- Never store credentials, generated artifacts, raw telemetry, or copied evidence in `memory/`.
- Memory/docs-only changes never trigger Rust/GPU FULL, smoke, scenario candidates, official capture, or user acceptance.

## Immediate disable

Stop Hook rule injection before changing Git:

```powershell
$env:BALLAST_DISABLE = "1"
```

Alternatively remove trust from the Ballast Hook in Codex `/hooks`.

## Project rollback

Use the exact sequence recorded in the synchronized `personal-infra-wiki` rollback page. General rule:

1. Revert Ballast-only post-integration checkpoint/policy commits, newest first, when any exist.
2. Revert the merge while keeping product/evidence first parent:

```powershell
git revert -m 1 6b5f0201f882f212f9916521aec689261d97b4a6
```

Do not revert G8-B/G8-C product or evidence commits merely to remove Ballast.

Because the pre-cutover domain documents remain in Git history and canonical evidence was never replaced by memory, rollback does not reconstruct product truth from memory.

## Global Codex rollback

Project rollback is normally sufficient. For cross-project failures:

1. set `BALLAST_DISABLE=1` or untrust the Hook;
2. close Codex;
3. restore the appropriate timestamp backups under `C:\Users\mdkap\.codex\`;
4. remove only Ballast-managed resources and rule IDs:

```text
C:\Users\mdkap\.agents\skills\ballast-*
C:\Users\mdkap\.codex\hooks\ballast-rules.mjs
Ballast handler entry in C:\Users\mdkap\.codex\hooks.json
Ballast managed AGENTS block
Ballast-owned IDs in C:\Users\mdkap\.claude\ballast.rules.json
```

Never delete an entire shared rules file when only Ballast-owned IDs must be removed.

## Rollback triggers

Disable first and evaluate rollback when any serious condition occurs:

- stale checkpoint outranks contradictory live Git/runtime;
- valid exact-source evidence exists but expensive FULL/GPU/candidates are repeated without an invalidating input change;
- an agent proposal is recorded as a user decision;
- memory maintenance becomes larger than the development task;
- Hook errors block valid work;
- memory hides a dirty worktree or induces destructive Git operations;
- `docs/HANDOFF.md` and `memory/CHECKPOINT.md` are maintained as two live checkpoint systems.

One severe safety/correctness failure is enough for immediate rollback review. Repeated non-blocking friction is enough to reconsider the cutover.

## Post-rollback verification

Confirm:

- project AGENTS no longer activates Ballast;
- `memory/` is absent or explicitly historical/non-active;
- the prior resume path is restored intentionally;
- `docs/planning/STATUS.md`, evidence, ADRs, specs, validation policy and product commits remain intact;
- no runtime validation or recapture ran solely because of rollback docs;
- reverted commits remain in Git history for audit.

Global installation and cross-project rollback authority: `personal-infra-wiki` → Codex Ballast workflow and Powdergame rollback troubleshooting page.
