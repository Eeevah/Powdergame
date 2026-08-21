# TE-3Q / TE-5Q Conservative Phase Packets

- **Decision:** D-023
- **ADR:** [`ADR-0011`](../architecture/decisions/ADR-0011-conservative-phase-packets.md), Proposed
- **Runtime:** NOT STARTED
- **User gate:** DESIGN BLOCKED — architecture revision required

## Why this program exists

The four preserved TE-5B/C/D/X failures all retained one whole quantity per
foreground phase Cell. D-023 changes that ontology directly: visible Steam
expansion creates two real half-quantity Steam packets while conserving the
Water-equivalent total. This avoids a remote capacity ledger, matching solver,
owner extent and volume field.

## Locked candidate

- explicit units scale 2;
- Ice/Water units 2, Steam units 1 or 2;
- quantity-scaled local H and latent energy;
- actual existing expansion/receiver transaction for boil and later split;
- deterministic orthogonal-only local condensation pairing;
- no half-Water; lone half-Steam metastability is explicit;
- dedicated spatial phase pressure, separate from generic pressure and Air;
- 96 MiB persistent increment over TE-2 at 2048²;
- projected 50 passes / 100 timestamp queries / max eight storage bindings;
- no additional full-world scratch or external implementation.

## Decision checkpoints

1. Freeze the ADR/spec/validation, seed, coefficients, merge order and fixtures.
2. Syntax/import/list-check the standard-library script without model work.
3. Hash it and execute exactly once into a new non-existing result.
4. Freeze authority hashes and obtain fresh-context independent review.
5. If Critical/High is nonzero, stop DESIGN BLOCKED. Otherwise request user
   review; do not mark ADR-0011 Accepted.

## User architecture-review checklist

- Is a visible Steam Cell representing half a Water-equivalent acceptable?
- Is Water/Ice remaining whole while Steam alone packets into halves clear?
- Is a lone condensation-ready Steam/1 metastable state acceptable?
- Is orthogonal-only deterministic pairing preferable to diagonal or
  cold-ranked pairing?
- Do phase-pressure rise at tick 8 and finite relaxation after relief preserve
  the intended Wood-rupture chain?
- Are 96 MiB additional state and the static 50-pass projection acceptable
  before implementation evidence?
- Do open-beaker, cold-lid and finite-boiler traces match product meaning?

## Stop boundary

Maximum success is **CONSERVATIVE PHASE PACKET DESIGN CANDIDATE / INDEPENDENT
REVIEW PASS / USER ARCHITECTURE REVIEW PENDING**. Rust, WGSL, Cargo, allocation,
runtime, build, launch, TE-4, G9-B/C/D/E, optimization, PR and main merge remain
not started. Failure does not authorize another synthesized model.

## Frozen result

The one-shot reference returned mathematical PASS, but fresh independent
review found Critical `0` / High `8` / Medium `1`. Final stop is **TE-3Q /
TE-5Q DESIGN BLOCKED / ADR-0011 PROPOSED / RUNTIME NOT STARTED**. No success
stop or user-acceptance request is available from this evidence identity.
