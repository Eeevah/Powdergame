# Session log

This is a terse append-only audit trail, not a transcript.

## 2026-08-19 · Initial isolated Ballast pilot

- Wiki/profile: synchronized `personal-infra-wiki` at `318276eebfbf913638d72f5d218ead2450361a01`; local `HEAD == origin/main`; `install-ballast-codex.ps1 -Verify` passed; no reinstall was performed.
- Repository coordinate: exact fetch of `origin/feature/m0-g8b-scenario-suite` produced Base SHA `e43078737712862c9cc6ccdc4b7e56475bafc6ce`; isolated branch `agent/ballast-memory-pilot` was created from that commit.
- Isolation: the existing `Powdergame-g8b` worktree was observed at `feature/m0-g8b-scenario-suite` / `e43078737712862c9cc6ccdc4b7e56475bafc6ce` with an empty `git status --short`; only permitted Git metadata was read, and no uncommitted contents were inspected or copied.
- Initialized: `AGENTS.md`, `memory/00-INDEX.md`, `memory/DECISIONS.md`, `memory/OPEN-QUESTIONS.md`, `memory/SESSION-LOG.md`, and `memory/CHECKPOINT.md`; no other project file was changed.
- Validation: six-path allowlist passed; 21 intended targets and 28 relative Markdown links resolved; stale-status/SHA review passed; guarded `pwsh -NoProfile -File tools/dev.ps1 audit` returned PASS/exit 0 with only the two declared legacy-launcher warnings; `git diff --check` passed. No repository-owned link/stale checker exists, so those two checks were targeted manual reviews.
- Publication outcome for the completed pilot session: the primary initialization commit uses `docs: add isolated Ballast memory pilot`, only `origin/agent/ballast-memory-pilot` is pushed, and a Draft PR is opened against `feature/m0-g8b-scenario-suite` and left unmerged for user evaluation. The live commit SHA and PR number/URL are reported in the final local receipt because they cannot exist before the commit containing this entry is created.
- Runtime: no Rust/GPU FULL, application execution, smoke, experiment/fixture candidate, official capture, performance run, or user acceptance was executed.
- Next: user reviews the immutable Heavy Mixed World evidence and records acceptance or a concrete finding.
