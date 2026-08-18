# Checkpoint — initial Ballast memory pilot — 2026-08-19 01:04 KST

## Repository coordinate

- Worktree: isolated sibling Ballast pilot (repository-relative role; absolute path belongs in the local report)
- Target integration branch: `feature/m0-g8b-scenario-suite`
- Pilot branch: `agent/ballast-memory-pilot`
- Confirmed fetched remote Base SHA: `e43078737712862c9cc6ccdc4b7e56475bafc6ce`
- HEAD at checkpoint authoring: `e43078737712862c9cc6ccdc4b7e56475bafc6ce` (`docs: record the Heavy Mixed candidate`)
- Working tree at authoring: dirty only by the expected six new agent/memory paths; intended publication state: clean

## The story so far

This is a docs-only navigation/return pilot; it does not replace Powdergame authority or prove runtime behavior. The exact remote integration state was fetched before the worktree was created. `docs/planning/STATUS.md` and `docs/HANDOFF.md` describe M0 as `IN_PROGRESS`: G0-G7 are closed, G8-A technical evidence is verified with same-SHA visual validation pending, Scenarios 1-4 and Cell Inspector v0 are user accepted, and Heavy Mixed World is the sole current G8-B acceptance blocker. Heavy is automatic `NEEDS_HUMAN_REVIEW` only for `broad_terminal_tail`, with 14/14 hard predicates passing and `candidate_blocker=false`; G8-B is not closed and G8-C is forbidden.

Confirmed Git state: fetched `origin/feature/m0-g8b-scenario-suite` and the new pilot both began at the full Base SHA above. Existing documented state: the current status/handoff and evidence records listed below. Reported local state elsewhere: the task identifies the active worktree as protected ongoing user work; its initial `git status --short` was empty at branch/HEAD `feature/m0-g8b-scenario-suite` / `e43078737712862c9cc6ccdc4b7e56475bafc6ce`, and no file contents were inspected. Unknown/pending state: Heavy user acceptance, G8-A same-SHA visual validation, explicit G8-B closure, and later gate/publication choices.

Opened authority: `docs/HANDOFF.md`, `docs/development/QUICKSTART.md`, `docs/planning/STATUS.md`, `docs/development/VALIDATION_POLICY.md`, `docs/development/DEVELOPMENT_LEARNING_LOOP.md`, `docs/development/LESSONS_LEDGER.md`, `docs/development/TESTING.md`, `docs/development/WORKTREE_ARTIFACT_EXECUTABLE_POLICY.md`, `docs/planning/ROADMAP.md`, `docs/planning/MILESTONES.md`, the Gallery evidence record, all current G8-B scenario evidence linked by `STATUS.md`, and the G8-A record.

## Valid evidence

- Heavy Mixed machine publication/manual-review evidence — exact immutable tuple: clean source `07260fffab22e5b4513eb168f0baac36e374ab94`; `cargo build --locked --release -p powdergame-windows`; frozen binary SHA-256 `9b84db005942cf60ae9ef133521e9297413d49c93d72e7ae64133e29622f7583`; source-input manifest SHA-256 `d4cf97dba93a3bf108e6105c623bcfb506baeb4306be327143f993faeec28ff3`; Consolas input SHA-256 `cf00b507b3286870cc5064ebd0633c303f70b491a4af25eec2d32df413db0179`; worker `--experiment-worker heavy-mixed --max-ticks 20000 --diagnostic-interval 8`; release profile; 256×256 world, chunk 64, terminal window 64 samples, overlap minimum 3; RTX 5090 / DX12 hardware check; run `g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4`; Receipt SHA-256 `2abebdef7f9174e63abfd9c67ce4a48d24b48edde4e6c29fab49022e36a2dbd1`; Audit Bundle SHA-256 `bc44c66bd52b5d856decb2317389a455a56ac8ae1f8d67b1bfeb5446cfb5731b`. This tuple supports 14/14 hard PASS, `candidate_blocker=false`, and automatic `NEEDS_HUMAN_REVIEW` solely for `broad_terminal_tail`; it does not supply user acceptance.
- G8-A v5 technical evidence — exact documented tuple: clean source/upstream `9abec9ee632b9abe429b13cf0cfb2e3ae7eacefe`; official `apps/benchmark/capture-evidence.ps1 -Official` path with isolated locked Cargo release build; schema `powdergame-g8a-v5`; 2048×2048 world, chunk 64, sleep enabled at threshold 16; RTX 5090 device `0x10DE:0x2B85`, driver `32.0.15.9636`, DX12; capture `g8a-v5-9abec9e-20260817T032827206Z`; package SHA-256 `4b9f44f66c18235f80d33738d15f3418c65c98e68e254aa01d13fc3a66eb6ec8`; Receipt SHA-256 `084012de7549eb8742f8974e40f21407c7d70f5d8bee346c60384a878a0ccbf3`; verifier-record SHA-256 `143b628e4bcd59a77df94c33aa085d4d4144addb403275e89eff1c3097fd260b`. This tuple supports G8-A calibration/technical claims only, not G8-B/G8-C or user validation.
- Current acceptance records remain indexed rather than re-executed: Sand automatic `PASS` / user accepted; Water automatic `NEEDS_HUMAN_REVIEW` / user accepted with known follow-up; Fire automatic `PASS` / user accepted; Pressure automatic `NEEDS_HUMAN_REVIEW` / user accepted with known follow-up; Cell Inspector v0 user accepted with known follow-up. This compact checkpoint does not reproduce their complete command/toolchain/config tuples. Before reusing any underlying validation command, open the linked canonical evidence/receipt and match every `VALIDATION_POLICY.md` reuse key; absent an exact match, do not claim command-result reuse.

The immutable Heavy and G8-A historical claims remain valid only while every listed source/input/binary/config/hardware identity and receipt/package hash remains intact and complete. A source, fixture, analyzer/capture implementation, schema, build profile, relevant config, hardware/backend, or authenticated artifact-byte change invalidates transfer to a new claim. The canonical records do not expose every toolchain-version value in this compact checkpoint, so no new toolchain-sensitive run may be treated as reused from this file alone. Docs-only changes and branch-pointer movement do not invalidate the old exact-run evidence, and no result transfers runtime provenance to this pilot docs commit.

## Decided

- D-001 — Use a bounded isolated Ballast memory pilot that supplements existing authority and imports no active-worktree WIP.
- D-002 — Treat agent/memory-only updates as docs-only and reuse only evidence whose exact validity inputs remain unchanged.

## Waiting on the user

- Q-001 — Heavy Mixed World acceptance or a concrete finding; this is the current blocker.
- Q-002 — Same-SHA G8-A user visual validation remains pending.
- Q-003 — G8-B closure and every later gate/publication choice require a separate explicit decision.

## Next first action

The user reviews the immutable Heavy Mixed World Contact Sheet and the three recommended frames in the canonical Gallery/Cell Inspector, then records acceptance or a concrete finding without rerunning the candidate, changing its automatic verdict, or authorizing G8-C implicitly.

## Tried

- An initial privileged exact-fetch attempt stopped before mutation because Git rejected the ownership context. The successful retry used command-local `safe.directory`; do not change global Git configuration for this worktree.
- A same-process wrapper initially mistook the audit's intentional invalid-alias probe `$LASTEXITCODE` for audit failure even though the audit printed PASS. The separate `pwsh -NoProfile -File tools/dev.ps1 audit` process returned exit 0; use the process exit, not ambient `$LASTEXITCODE` left by an internal probe.
- Do not seed current memory from the older status snapshots in Quickstart, Roadmap, or the initial Gallery evidence; follow current `STATUS.md`/`HANDOFF.md` links.
- Do not run `tools/dev.ps1 validation-plan` for this new `AGENTS.md`/`memory/**` path set: the current machine patterns classify it as unknown. Use the explicit docs-only policy and six-path allowlist.
- Do not rerun accepted scenario candidates, G8-A capture, runtime/GPU tests, smoke, or user acceptance for this pilot.
