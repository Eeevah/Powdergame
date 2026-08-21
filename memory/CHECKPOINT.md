# Checkpoint — TE-5X comparison authorized; model evaluation next — 2026-08-21

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Task baseline: `f5b146571f2cb95b89d56d8831b68ddbeb75f395`
- Authorization commit: pending first docs/memory commit
- Comparison/checkpoint commit: not yet created
- Wiki read-only fallback: `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3`
- Runtime source is unchanged; this task is docs/reference only.

## Current truth

G9-A and TE-2 remain **USER ACCEPTED WITH KNOWN FOLLOW-UP**. TE-3D remains
**ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS**. TE-5B, TE-5C and TE-5D's
fixed-depth reassignment are **REJECTED / DESIGN BLOCKED**. D-022 authorizes a
TE-5X comparison over exactly three models: exact persistent-extent matching,
shared connected gas-chamber capacity and a conservative Vapor-volume
Environment scalar. No candidate is accepted. ADR-0010 and the combined proof
do not yet exist. TE-3/TE-5 runtime remains **NOT STARTED**.

## Validation boundary

- One combined pre-registered external comparison may run exactly once.
- No Rust, WGSL, Cargo, runtime allocation, GPU/device, build, launch or FULL
  validation is authorized.
- Historical TE-5B/C/D proof results remain read-only and source-bound.
- External copied/translated/vendored implementation remains `0 files / 0 lines`.

## Next first action

Complete the primary-source algorithm survey and freeze the three candidate
contracts, common fixtures, production-cost projections and combined proof
before executing it once. Do not recommend a model before fresh review.
