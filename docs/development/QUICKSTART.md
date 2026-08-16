# Powdergame Developer Quickstart

Read this first when entering the repo.

## Current gate

- M0: IN_PROGRESS
- G0-G6: PASS / CLOSED
- G7-A: USER VALIDATED measurement baseline
- Next: G7-B actual sleep / wake correctness

## Windows

Typical repo/worktree root:
`C:\Users\mdkap\source\repos\Powdergame*`

Use the gate-specific worktree when present. Never blindly pull/rebase a dirty worktree.

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
Then run only the required demo smoke matrix.

**PERFORMANCE — manual only**
Do not run performance benchmarks during normal build/test loops. Run only when explicitly requested or at G8.

## Never forget

- GPU production simulation is authoritative.
- One Cell = Max One Matter; EMPTY is not hidden air.
- Demo/HUD/diagnostics must not silently change physics.
- Frozen G0-G6 physics needs explicit justification to change.
- Temperature is a relative gameplay scalar, not Celsius.
- `docs/planning/MATERIAL_CANDIDATES.md` is user-owned; do not touch it.
- AI/CI may reach VALIDATION; user approval is required to close a gate.

For details: `docs/planning/STATUS.md`, `docs/HANDOFF.md`.
