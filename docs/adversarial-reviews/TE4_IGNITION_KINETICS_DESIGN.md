# TE-4D Ignition Kinetics — Fresh Independent Adversarial Review

- Review date: 2026-08-22
- Reviewer: fresh-context independent agent; primary author did not perform this review
- Reviewed source: authorization HEAD `42cca5e73da542188d06b8e26d7e4a2934375a9a` plus uncommitted TE-4D docs/reference candidate
- Runtime execution/edit by reviewer: none
- Final finding count: **Critical 0 / High 2 / Medium 0 / Low 0**
- Verdict: **TE-4D DESIGN BLOCKED / ADR-0012 PROPOSED / RUNTIME NOT STARTED**

## Unresolved findings

### H-001 — Required frozen reference completed no model work

The failure receipt records attempt `1`, completion `0`, and zero sequences,
grids and fixtures. Oil tuples `48/2/25/4/1/2` and preregistered
`48/2/50/6/1/2` both yield `24/12/12/24`; undeclared lexicographic secondary
ordering chose the former and the frozen assertion stopped. D-028 forbids
repair/rerun. A new direct user decision must authorize a new evidence identity
and define whether equal metrics are equivalent or need an explicit normative
secondary objective. No coefficient, packed state or fixture receives PASS.

### H-002 — The frozen script can overstate named fixture coverage

F10 records fuel/exposure booleans without executing their transition; F11
only encode/decodes flags rather than movement/swap; F12 merely rewrites the
exposure bits rather than executing decay/rupture/consumption/Void; F13 records
authoring constants. F01 and F14–F17 are explicitly `NOT_ESTABLISHED`, yet the
top-level builder only treats literal `FAIL` as a blocker. F08 would allow up
to 79 new ignitions in an 81-Cell grid.

A new reference identity must aggregate PASS/FAIL/NOT_ESTABLISHED, forbid a
top-level PASS while any required reference obligation is unexecuted, separate
production-only fixtures, execute state transitions for F05/F09–F13 and pin an
exact per-tick frontier bound for F07/F08 including near-budget regions.

## Resolved during review response

### H-003 — Chemical-Q cap/accounting and capacity input — resolved

The first review found that fixed total Q ignored the live 1200 C cap and that
the proposed GPU descriptor had no heat-capacity input. ADR-0012 now separates
gross, deposited and clipped gameplay-Q. Core owns gross Q; descriptor
compilation uses the same Material heat capacity to serialize derived
`chemical_delta_t=Q/C` into byte 12. The current cap remains, no ninth storage
binding is added, exact totals are labeled maximum gross budgets, and F09
requires real burn-tick/cap/no-double-source accounting. This resolves the
design contradiction, not future implementation evidence.

### M-001 — Packed descriptor validation incomplete — resolved

The candidate now fails closed on u6 budget, positive decay/base/bucket,
`base<=max`, u8 flame/cap, duration, finite Q/capacity/delta, reserved bits,
zero non-combustible sentinel and no integer truncation. The earlier zero-width,
zero-decay and 256-to-u8 counterexamples are rejected before upload.

## Other attacks

- No live owner collides with flags bits 2..3/28..31; current hygiene would
  erase them and the design correctly requires a future mask change.
- Existing Current/Next pass order can preserve previous-tick-only flame reads;
  no same-tick recursive path was found in the packed projection.
- Combustion/activity remain at eight storage bindings. Reusing the combustion
  uniform at activity binding 9 is structurally feasible; execution remains
  unestablished.
- The dedicated-buffer estimate is conservative, but its exact hygiene/order
  remains explicitly unestablished.
- Vacuum is user-owned; no Oxygen, Ash, FX or Pressure scope leak was found.
- No historical evidence was rebound and no external implementation entered.

## Required next decision

Keep TE-4D blocked. A later user decision must authorize a new evidence
identity, tie/aggregation/frontier rules and the remaining architecture/Vacuum
choices. Do not implement from this ADR.
