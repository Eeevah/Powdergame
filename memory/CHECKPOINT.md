# Checkpoint — TE-3 phase-cycle candidate ready for user review — 2026-08-21

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Task baseline: `b5a3405bffba5f757f966e44d0cae0077f57d285`
- Wiki read-only fallback: `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3`
- Runtime source: `41467219819c5d0cb3eab8ae22b652449da20480`

## Current truth

D-024 is implemented as a pressure-decoupled ADR-0006 candidate. One family
Cell is one Water-equivalent quantity; transitions are 1:1. Water produces no
second Steam, expansion proposal or blocked pressure. ADR-0011 and TE-5B/C/D/X
remain blocked history. Historical G5 evidence is source-bound. TE-5 Pressure
redesign is DEFERRED / NOT STARTED.

## Scope and boundary

- Persistent TE-3 state is exactly the phase-energy Current/Next pair: 32 MiB
  at 2048 squared. No packets, units, phase pressure/volume or new scratch.
- The graph is 40 passes / 80 queries and every pass is at most 8 storage
  bindings.
- Candidate routes are `run_powdergame.bat phase-cycle` and `te3`; no-argument
  Sandbox is unchanged. Candidate starts paused and labels pressure deferred.

## Validation boundary

- Actual TE3-F01–F15 fixtures and targeted suites pass.
- Final-source canonical FULL passes at `4146721...`.
- Release build, bounded launch and bounded measurement counts are each one.
- EXE SHA-256 is
  `99745D13A7F5D7323EB5961A3A462A965C446C10CDA4CA9AF04495B0537C87BE`.
- G8/G8-C, TE-4, G9-B/C/D/E and official capture counts remain zero.

## Next first action

Run `run_powdergame.bat phase-cycle` for direct user review. Do not claim user
acceptance or start TE-5/TE-4/G9-B work without a new direct decision.
