# Checkpoint — TE-4I implementation-first evidence authorized — 2026-08-23 00:08 KST

## Repository coordinate
- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- HEAD: `a19753ba087309e4f2a4863915d57b67750f1ad2`
- Working tree: expected D-032 docs/memory authorization files

## The story so far
TE-4D v1/v2/v3 and D-031 are blocked immutable history. D-032 ends the
synthetic-reference loop and authorizes the locked ignition semantics as an
implementation-first Core/GPU candidate. ADR-0012 is still Proposed and not
accepted.

## Valid evidence
- TE-3 production physics `41467219819c5d0cb3eab8ae22b652449da20480` — valid for accepted TE-3 only.
- Live-source inventory at `a19753b...` — immediate-threshold baseline and 40-pass graph.
- V1/v2/v3/supplement artifacts — immutable blocked history only; no runtime claim.

## Decided
- D-032 — implement and validate TE-4I through actual production paths.
- Packed u6, locked Oil/Wood coefficients, non-Vacuum Air face and finite chemical Q are fixed.
- No new persistent/full-world state and no pass above eight storage bindings.

## Waiting on the user
No decision is required during the authorized implementation. Final ADR/product
review remains pending after source-bound evidence and candidate delivery.

## Next first action
Audit the live Core/WGSL writer and pass inventory, then implement packed-u6
Core semantics before connecting production passes.

## Tried
- Synthetic v1/v2/v3/supplement evidence is not reused as runtime evidence.
- Wiki `origin/main` `57d7e2bdbab5b9cbc46a4448fd881e7493e12f74` verified; dirty local Wiki untouched.
