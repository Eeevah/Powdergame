# Powdergame Developer Quickstart

Read this first when entering the repo.

## Current gate

- M0: IN_PROGRESS
- G0-G7: PASS / CLOSED
- G7-A: USER VALIDATED / FROZEN
- G7-B: PASS / CLOSED / FROZEN
- G8: Performance Evidence (IN_PROGRESS; historical v4 remains unbound historical data)
- Current work: seal `fix/g8a-evidence-remediation-v5` as a clean-source, receipt-bound G8-A evidence candidate
- Out of scope here: Canonical Recovery, G8-B/G8-C/G9, new materials, optimization, and `main` merge

## Windows

Typical repo/worktree root:
`C:\Users\mdkap\source\repos\Powdergame*`

Use the gate-specific worktree when present. Never blindly pull/rebase a dirty worktree.

The preserved correction was attached without reset/stash/rebase/pull to `fix/g8a-evidence-remediation-v5` from base `a67abaf959aba0423627f35b79fce7c82d8ec9b5`. Do not repurpose this branch for product work or integration. Its only purpose is the G8-A scope frozen in `docs/evidence/G8_A_REMEDIATION_V5_SCOPE_2026-08-17.md`.

## Run current G7 demo

```bat
run_g7_activity_demo.bat
```

Direct:
```bat
cargo run --release -p powdergame-windows -- --activity-demo
```

Controls: `SPACE` play/pause · `N` one tick · `F` x1/x4/x16 · `R` reset · `ESC` quit.

## Validation policy

**FAST — normal iteration**
```bat
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test <targeted tests>
```

**FULL — once per gate/checkpoint round**
```bat
cargo test --workspace -- --test-threads=1
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```
Do not run broad or repetitive demo smoke matrices by default. Run only the smallest smoke that is genuinely required by the current change. If user testing later exposes a problem, reproduce and validate only that affected path.

**PERFORMANCE — manual only**
Do not run performance benchmarks during normal build/test loops. G8 is now the next gate where performance measurement is allowed/expected.

The historical G8-A v4 aggregate/raw timing CSVs can be numerically reconstructed, but they are not bound to the later dirty source snapshot or executed binary and do not contain raw census buffers. Do not label them an official baseline.

For the next auditable G8-A capture, use `apps/benchmark/capture-evidence.ps1 -Official` instead of invoking the benchmark binary or `cargo run` directly. Official mode requires an attached clean source SHA and a new empty destination outside the repository. It performs an isolated locked release build, records the source snapshot, exact command and raw logs, hashes the executed binary, and links aggregate/raw tick/raw cell/raw chunk CSVs through one run receipt. `CAPTURE_RECEIPT.json` is the final completion marker; without it the capture is incomplete. Never rerun a failed capture under the same Capture ID.

```powershell
pwsh -NoProfile -File .\apps\benchmark\capture-evidence.ps1 `
  -Official `
  -DestinationRoot <new-empty-directory-outside-the-repository>
```

The v5 remediation branch stops after source publication, one official capture, and independent verification. Canonical Recovery and later gates are separate work and are not implied by a v5 receipt.

**ADVERSARIAL REVIEW — opt-in only**

Do not automatically request, perform, or file an adversarial review. Only do so when the user explicitly requests it, following `docs/adversarial-reviews/README.md`. Do not send Powdergame code, diffs, artifacts, or review prompts to GPT Pro, Grok, or another external AI reviewer.

## Never forget

- GPU production simulation is authoritative.
- One Cell = Max One Matter; EMPTY is not hidden air.
- Demo/HUD/diagnostics must not silently change physics.
- Frozen G0-G6 physics needs explicit justification to change.
- Temperature is a relative gameplay scalar, not Celsius.
- `docs/planning/MATERIAL_CANDIDATES.md` is user-owned; do not touch it.
- AI/CI may reach VALIDATION; user approval is required to close a gate.

For details: `docs/planning/STATUS.md`, `docs/HANDOFF.md`.
