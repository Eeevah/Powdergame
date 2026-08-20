# Chinese-community reuse survey intake — 2026-08-20

## Authority and provenance

This document records an external research input titled **“Powdergame
중화권 오픈소스·커뮤니티 자원 마일스톤 매핑 보고서”**, supplied by the user on
2026-08-20. It is not a Powdergame source of truth and does not reopen D-013,
D-014 or ADR-0005. No standalone report file, byte hash, public URL or exact
upstream revision was exposed to this workspace; those identifiers therefore
remain unavailable rather than being guessed.

External simulation code copied, translated or vendored: **0 files / 0
lines**.

## Exact independently resolved upstream identities

These identities are reused from the canonical TE-0R audit rather than
inferred from community naming:

| Source | Immutable identity | License | Use here |
|---|---|---|---|
| [The Powder Toy](https://github.com/The-Powder-Toy/The-Powder-Toy/commit/2e47966b84b0d2f1750af0f82643791803537ea5) | `2e47966b84b0d2f1750af0f82643791803537ea5` | GPL-3.0 | fixture/UX reference only; copying, translation and close porting forbidden |
| [sandspiel](https://github.com/MaxBittker/sandspiel/commit/dc77827b36adc5c04ea063515de4173ce28dbf2c) | `dc77827b36adc5c04ea063515de4173ce28dbf2c` | MIT | bounded fixture/product reference only |
| Cinder named in the supplied report | exact repository/revision unavailable | no license statement available | `REFERENCE_FIXTURE` only; no code or formula reuse |

If the report's exact source file or Cinder upstream is later supplied, this
intake must be revisited before claiming a stronger provenance identity.

## Adoption map

| Research item | Provenance / license | Adoption mode | Powdergame mapping | Revisit / invalidation condition |
|---|---|---|---|---|
| `SMALL_DELTA_THERMAL_CONVERGENCE` | User-supplied report; formula was independently specified in the task | `ADOPTED_TEST_CONTRACT` | TE-2 CPU reference and production GPU regression; deadband is a gate, never a subtractive flux | Revisit only if the canonical thermal predicate or numerical representation changes |
| `AIR_GAP_HEAT_TRANSPORT` | Research alias only | `PROVENANCE_ALIAS` | Existing TE-F05 and TE-F07; no duplicate production test | Revisit if those fixture meanings change |
| `SEALED_EDGE_NO_FLUX` | Research alias only | `PROVENANCE_ALIAS` | Existing TE-F09 and TE-F10; no duplicate production test | Revisit if the sealed correctness edge changes |
| `SOURCE_FREE_ENERGY_STABILITY` | Research alias only | `PROVENANCE_ALIAS` | Existing TE-F01–F03 and numerical invariants | Revisit if passive accounting/tolerance changes |
| Cinder | Exact repository/revision not identified; no license statement available in the supplied material | `REFERENCE_FIXTURE` only | Small-delta semantic fixture idea only; no implementation reuse | Exact upstream repository, immutable revision and compatible license must be established before any stronger use |
| `NO_REGION_WIDE_INSTANT_COMBUSTION` | Research backlog alias | `REFERENCE_ONLY` | TE-4 backlog | Revisit only at the TE-4 ignition gate |
| `FINITE_CORROSION_BUDGET` / `REACTION_ORIENTATION_INVARIANCE` | Research backlog aliases | `REFERENCE_ONLY` | M1 / G9-B backlog | Revisit only when that later scope is explicitly authorized |
| `SEMANTIC_EVENT_PRESENTATION_DECOUPLING` | Research backlog alias | `REFERENCE_ONLY` | G9-C / G9-D | Revisit only at the corresponding presentation gate |
| Noita Chinese corpus / The Powder Toy Chinese UX / Sandfall product UX | Community/product observations; not runtime formula sources | `RESEARCH_INPUT_ONLY` | Later gate product research | Revisit only when the relevant later gate starts and exact sources/licenses are recorded |

The exact upstream and GPL boundary for The Powder Toy, and other already
audited sources, remain in the canonical
[`thermal-environment reuse survey`](2026-08-20-thermal-environment-reuse-survey.md).
This intake does not broaden those reuse permissions.

## Scope boundary

No ADR-0006, ADR-0007 or ADR-0008 is created. `NO_REGION_WIDE_INSTANT_COMBUSTION`,
corrosion, reaction orientation and semantic-event presentation are not part
of the TE-2 runtime diff. No Oxygen, Ash, new Matter, final FX, CFD or
optimization is authorized here.
