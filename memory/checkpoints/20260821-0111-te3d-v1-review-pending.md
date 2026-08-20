# Checkpoint — TE-3D phase-enthalpy design candidate / user review pending — 2026-08-20 23:28 KST

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Session start source: `94b152e85ff6f5481a033d885d38dca0dbc1043a`
- TE-2 closure commit: `fd97e8b89f277e1205c8b5bcd970002bfd87e7c4`
- Production TE-2 source: `fb7e568e21012b6067269f4e1b82c36c865023d0`
- Review-remediation source: `097728128343cf89383920c968a010b3dcf8e8c0`
- Working tree: docs/memory-only TE-3D candidate awaiting its design commit;
  runtime source unchanged

## The story so far

G9-A and TE-2 remain **USER ACCEPTED WITH KNOWN FOLLOW-UP**. D-017 preserves
the direct observations and authorizes only the docs/research/review/reference
phase-design program. The two non-blocking TE-2 follow-ups remain
`LONG_HORIZON_SEALED_AIR_DRIFT_BUDGET` and
`TE2_CANDIDATE_HUD_LABEL_POLISH`.

Proposed ADR-0006 now defines Hybrid A+C: one Water-equivalent Ice/Water/Steam
Cell, 1:1 identity transitions and two Matter-owned f32 phase-energy halves.
The design records reversible local enthalpy, surface boiling, cold-surface or
deterministic free-air condensation, exact writer/binding/reset contracts and
a 40-pass/80-query projection. It does not implement or activate runtime.

The pure reference proof passed exactly once. The independent fresh-context
review then found four High counterexamples; the final design resolves all
four through temporal partial-progress veto, evidence-order repair, atomic
G5/TE-5 activation and an Air-aware claim-backed phase-context pass. Final
review disposition is **INDEPENDENT DESIGN REVIEW PASS / USER ARCHITECTURE
REVIEW PENDING**, with zero unresolved Critical/High findings.

## Valid evidence

- Reference script SHA-256:
  `117439a84f1debdc4e4cca6007a4307903bc643cb1811f8c0d979dfecda05561`;
  result SHA-256:
  `6c1afe9f3734be51301562ee3363a94726a75c1f64c222c3dc824ed31d19e42e`.
- Fixed seed `0x54453344`, 50,000 generated enthalpy trials, 4,096 finite
  nucleation regions, maximum absolute H error `1.52587890625e-05`, and 100
  closed cycles ending with one Water quantity at 20°C/E=0.
- Independent review SHA-256:
  `1fbc4501f62b91042ff5658cc8a6b509042e0fba84a266f72e610ef686274d34`;
  18 findings, four High resolved, five Medium and two Low still open.
- The proof does not establish WGSL, bindings, movement, sleep, performance,
  appearance or user acceptance. No Cargo/GPU/FULL/build/launch/runtime test
  was run for this docs-only design.
- Local personal-infra Wiki remained user-dirty; verified remote
  `origin/main` `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3` was used read-only.

## Decided

- D-017 remains the latest user-confirmed decision; no D-018 was recorded.
- ADR-0006 is `Proposed — user architecture review pending`, not Accepted.
- Proposed constants are `Lf=80`, `Lv=480`, surface maximum 80°C, minimum
  delta 10°C and free-air maximum 70°C.
- Water yield 1 cannot become production/user-testable before a separately
  authorized same-source TE-5 replacement preserves the frozen G5 causal
  chain atomically; that replacement is not designed or authorized here.
- TE-3 runtime, Air-pressure force, TE-4 and G9-B/C/D/E remain **NOT STARTED**.

## Waiting on user

The user must accept or revise Hybrid A+C, the proposed constants,
condensation/nucleation behavior, the 32 MiB 2048² memory cost and the atomic
G5/TE-5 activation constraint. The review also leaves temporal initiation-rate
bounds, buried in-progress boiling, isolated supercooled Steam and
zero-conductivity sink semantics open.

## Next first action

Review ADR-0006 and record accept or revise for Hybrid A+C, constants,
condensation/nucleation, memory cost, and atomic G5/TE-5 activation constraint.

## Tried

- Audited existing phase, Current/Next, movement, Environment, thermal,
  activity, editor/reset, profiler and validation mechanisms before designing.
- Reused TE-2 Q transfer, thermal tables, claim/proposal scratch, movement
  ownership and joint-settle patterns; proposed only the two required phase
  energy halves and no full-world scratch.
- Rejected extra independent Steam Matter and owner-linked fragments because
  they expand the quantity/ownership surface and can duplicate Water units.
- Preserved all TE-1/TE-2 evidence at its original source and copied no
  external implementation code or formula.
