# Powdergame Handoff — Retired Live Entry Point

Status: **HISTORICAL / DOMAIN REFERENCE ONLY**  
Retired as a live per-session checkpoint by Ballast integration merge `6b5f0201f882f212f9916521aec689261d97b4a6`.

## Canonical resume order

New, reset, or compacted sessions must start here instead:

1. [`../memory/00-INDEX.md`](../memory/00-INDEX.md)
2. [`../memory/CHECKPOINT.md`](../memory/CHECKPOINT.md)
3. active entries in [`../memory/DECISIONS.md`](../memory/DECISIONS.md)
4. only the task-relevant canonical documents linked by the index

`memory/CHECKPOINT.md` is the sole current session coordinate. Live Git/runtime remains authoritative if the checkpoint is stale.

## What this retirement does not change

- [`planning/STATUS.md`](planning/STATUS.md) remains the current milestone/evidence router.
- [`planning/MILESTONES.md`](planning/MILESTONES.md) remains the Gate-completion contract.
- [`evidence/`](evidence/) remains the immutable evidence and human-disposition source.
- ADRs, architecture, specs, validation policy and lessons keep their existing authority.
- Product/evidence commits and artifacts are not copied into Ballast memory.

## Historical handoff content

The full pre-cutover handoff remains auditable in Git at first-parent product commit:

```text
4653d7c2e09e93f80fb81eeb73458d992c86858f:docs/HANDOFF.md
```

Open that historical version only for older recovery narrative or domain context. Do not update it as a second live checkpoint.

## Current product direction

The verified G8-C Matrix recommends `PROCEED_TO_G9`; no current simulation, coexistence, rendering, or persistent-memory blocker requires optimization before the first playable sandbox. The product brief is user approved and G9-A source `f9a7087249bf6ffa0b6d47ad7568ba1798f591a3` is an **IMPLEMENTATION CANDIDATE / USER ACCEPTANCE PENDING**. G9-B/C/D/E, Discovery, Save/Load, Rewind and optimization remain not started. Live session continuity remains in `memory/CHECKPOINT.md`. See:

- [`planning/STATUS.md`](planning/STATUS.md)
- [`evidence/G8_C_OFFICIAL_MATRIX_2026-08-19.md`](evidence/G8_C_OFFICIAL_MATRIX_2026-08-19.md)
- [`vision/FIRST_PLAYABLE_WORLD.md`](vision/FIRST_PLAYABLE_WORLD.md)
- [`vision/UI_DIRECTION.md`](vision/UI_DIRECTION.md)

## Ballast rollback

Immediate stop:

```powershell
$env:BALLAST_DISABLE = "1"
```

Project rollback contract and exact integrated merge identity:

- [`development/BALLAST_MEMORY_CUTOVER.md`](development/BALLAST_MEMORY_CUTOVER.md)

After a deliberate rollback, restore a single prior resume path; do not leave HANDOFF and CHECKPOINT active in parallel.
