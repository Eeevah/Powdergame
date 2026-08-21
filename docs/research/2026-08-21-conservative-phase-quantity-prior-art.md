# Conservative phase-quantity prior art — 2026-08-21

## Scope and clean-room boundary

This survey asks only whether explicit conservative phase fractions/packets
are established conceptual tools. It does not import an external solver,
discretization, source formula or code. Powdergame's integer half-packets,
local claim/receiver transactions and gameplay pressure law are authored from
its existing contracts. External copied/translated/vendored implementation is
`0 files / 0 lines`.

## Primary literature

| Source identity | Publication/license posture | Conceptual relevance | Fit and non-use |
|---|---|---|---|
| Hirt & Nichols, “Volume of Fluid (VOF) Method for the Dynamics of Free Boundaries,” *Journal of Computational Physics* 39(1), 1981, DOI [10.1016/0021-9991(81)90145-5](https://doi.org/10.1016/0021-9991(81)90145-5) | Elsevier journal article; no code license granted to this repository; historical paper, not maintained software | A conserved cell-associated phase amount can represent material occupancy without duplicating mass | Concept only. Powdergame does not adopt VOF advection, interface reconstruction or equations. |
| Olsson & Kreiss, “A conservative level set method for two phase flow,” *Journal of Computational Physics* 210(1), 2005, DOI [10.1016/j.jcp.2005.04.007](https://doi.org/10.1016/j.jcp.2005.04.007) | Elsevier journal article; no implementation license or dependency; publication identity is stable rather than maintained code | Conservation must be explicit when an interface representation moves | Concept only. No level-set PDE, reinitialization or numerical stencil is used. |
| Chiu & Lin, “A conservative phase field method for solving incompressible two-phase flows,” *Journal of Computational Physics* 230(1), 2011, DOI [10.1016/j.jcp.2010.09.021](https://doi.org/10.1016/j.jcp.2010.09.021) | Elsevier journal article; no code copied and no library dependency; publication identity is stable | Conservative phase representation can be separated from visual/interface identity | Concept only. No phase-field PDE, fluid coupling, mobility or chemical potential enters the candidate. |

## Internal reuse conclusion

The useful prior-art lesson is narrow: make the conserved amount explicit and
audit its transport. The implementation shape comes entirely from Powdergame:
integer units reuse Current/Next ownership; expansion reuses existing
proposal/claim and whole-parcel Environment receiver; contraction uses a new
bounded local claim/commit; pressure reuses the existing scalar/rupture grammar
but remains a separate component. The candidate is not a VOF, level-set,
phase-field or compressible-fluid solver.
