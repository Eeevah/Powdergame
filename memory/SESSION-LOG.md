# Session log

This is a terse append-only audit trail, not a transcript.

## 2026-08-19 · Initial isolated Ballast pilot

- Wiki/profile: synchronized `personal-infra-wiki` at `318276eebfbf913638d72f5d218ead2450361a01`; `install-ballast-codex.ps1 -Verify` passed; no reinstall was performed.
- Repository coordinate: exact fetch of `origin/feature/m0-g8b-scenario-suite` produced Base SHA `e43078737712862c9cc6ccdc4b7e56475bafc6ce`; isolated branch `agent/ballast-memory-pilot` was created from that commit.
- Initialized: `AGENTS.md`, `memory/00-INDEX.md`, `memory/DECISIONS.md`, `memory/OPEN-QUESTIONS.md`, `memory/SESSION-LOG.md`, and `memory/CHECKPOINT.md`; no runtime source was changed.
- Validation: six-path allowlist, intended-target/link review, stale-state review, guarded `tools/dev.ps1 audit`, and `git diff --check` passed. No Rust/GPU FULL, app run, smoke, experiment candidate, capture, or user acceptance was executed.
- Publication: commit `ba2b6406f6605882c51886b0a50bc64d10990a7f`; Draft PR #4 opened and left unmerged for evaluation.

## 2026-08-19 · User adoption and reversible cutover

- Pilot disposition: the user reported the trial useful and adopted Ballast as Powdergame's single active session-continuity workflow.
- Product decision synchronized into memory: Heavy Mixed World `USER ACCEPTED WITH KNOWN FOLLOW-UP`; automatic `NEEDS_HUMAN_REVIEW`, 14/14 hard PASS, `candidate_blocker=false`, immutable candidate identity, and non-blocking declining `broad_terminal_tail` preserved.
- Next-gate decision: G8-C Official Matrix authorized and already running in a separate active development context. The memory cutover does not interrupt or alter that task.
- Cutover: `AGENTS.md` and `memory/**` changed from pilot wording to active bounded project memory. Added `docs/development/BALLAST_MEMORY_CUTOVER.md` and an archived initial checkpoint.
- Authority: `memory/CHECKPOINT.md` is the sole live resume coordinate; domain/evidence/status/ADR/spec/validation documents remain authoritative in their own scopes; `docs/HANDOFF.md` is preserved as historical/domain reference rather than a parallel session checkpoint.
- Rollback: immediate disable through `BALLAST_DISABLE=1` or Hook untrust; project rollback reverts the active cutover commit and then pilot commit `ba2b6406f6605882c51886b0a50bc64d10990a7f`. Squash merge is forbidden.
- Integration safety: PR #4 remains unmerged while G8-C is actively writing. After the final G8-C report, refresh exact Git/evidence coordinates and integrate with commit boundaries preserved.
- Runtime: no app, Rust/GPU FULL, smoke, scenario rerun, capture, or evidence mutation was performed for the cutover.