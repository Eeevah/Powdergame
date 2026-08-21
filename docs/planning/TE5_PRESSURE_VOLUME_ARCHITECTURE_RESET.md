# TE-5X Pressure-Volume Architecture Reset

- **Authority:** D-022
- **ADR:** [ADR-0010](../architecture/decisions/ADR-0010-pressure-volume-model-selection.md), Proposed
- **Runtime:** not started
- **Current stop:** TE-5X DESIGN BLOCKED; one-shot comparison evidence incomplete

## Why the reset exists

TE-5B failed cross-tick capacity, TE-5C failed usable-capacity allocation and
reversible pressure, and TE-5D failed because a fixed matching depth cannot
certify maximum matching. The reset preserves their evidence and compares the
remaining representations instead of tuning another token, radius or grace
constant.

## Frozen comparison

| Candidate | Product meaning | Conservation mechanism | Main risk before execution |
|---|---|---|---|
| A exact persistent extent | every Vapor source owns an exclusive EMPTY extent or is certified unmatched | reciprocal link and exact maximum matching | variable global matching work and complex Air/movement/editor hygiene |
| B shared chamber | connected gas headspace shares aggregate EMPTY capacity | exact component labels and deterministic stats | narrow-neck equilibrium meaning plus high sort/reduction cost |
| C conservative field | added Vapor volume is a transported Environment scalar | TE-2-like Current/Next donor scaling | condensation cannot identify remotely transported volume to remove |

All use the common PVX-F01–F15 matrix in
[validation](../development/PRESSURE_VOLUME_MODEL_COMPARISON_VALIDATION.md).
The response formulas, random coverage, execution command, oracle version and
selection order are frozen before the only reference execution.

The only execution stopped at the NetworkX version guard before any candidate
or fixture ran. The failure receipt parsed and was hashed, but it is incomplete
evidence. A/B/C evaluation counts are all zero and the provisional ranking is
void. The task cannot patch and rerun its one-shot proof.

## User model-selection checklist

- Does B's chamber-equilibrium meaning match the intended readable boiler,
  including a narrow neck whose target changes globally but pressure relaxes
  over time?
- Is B's projected 160 MiB state/scratch delta and 188-pass conservative
  design envelope acceptable as an architecture candidate pending runtime
  optimization/evidence?
- Should A remain a fallback despite exact ownership, 64 MiB persistent cost
  and a much larger variable work bound?
- Does the C condensation counterexample justify permanent rejection without
  a fourth representation?
- Are Atmosphere/Vacuum capacity equality and deferred background pressure
  acceptable for this narrow bridge?
- No answer authorizes implementation; a separate atomic TE-3/TE-5 source and
  evidence decision remains required.

## Stop boundary

TE-5X stops **DESIGN BLOCKED** because the required combined evidence was not
produced, so no candidate's eligibility can be established. ADR-0010 stays
Proposed / comparison evidence incomplete. Do not rerun, create a fourth
candidate or begin implementation.
