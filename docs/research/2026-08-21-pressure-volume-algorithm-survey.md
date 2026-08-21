# Pressure-volume algorithm and reuse survey — 2026-08-21

- **Gate:** TE-5X docs/reference comparison
- **Repository baseline:** `f5b146571f2cb95b89d56d8831b68ddbeb75f395`
- **Decision authority:** D-022
- **Runtime/dependency change:** none
- **External implementation copied, translated or vendored:** **0 files / 0 lines**

This survey compares mathematical and API mechanisms; it does not authorize a
runtime dependency or import source. Links point to primary papers, official
upstreams or official API documentation.

## Internal reuse inventory

| Powdergame mechanism | TE-5X use | Boundary |
|---|---|---|
| accepted TE-3 `phase_energy` and descriptor tables | derive phase-vapor demand without a quantity field | no new latent-energy rule or Water-name shader branch |
| Current/Next settlement | stage reversible phase pressure or a conservative field | every writer and settle point remains explicit |
| proposal/claim after Smoke | Candidate A frontier/predecessor or Candidate B labels | full overwrite before reinterpretation; no simultaneous lifetime |
| TE-1 Environment receiver/reconcile | Candidate A reserved-target Air transaction | receiver feasibility is part of the matching edge, not a post-match surprise |
| TE-2 Air transport and unified thermal passes | Candidate C transport-pattern reference | no Air mass/energy aliasing and no second generic solver framework |
| generic `pressure[]` and rupture | common gauge boundary and Wood threshold 80 | phase-volume pressure stays distinguishable; historical G5 evidence is not rebound |
| activity/wake and profiler | separate phase-volume work and named variable/fixed groups | current base activity already uses eight storage bindings |
| checked allocation accounting | 256²/2048² byte projections | projections are not runtime evidence |

## External and official sources

| Source | Immutable identity/version | License | Maintenance signal | Examined mechanism | Use/decision | Runtime dependency / copied code | Revisit condition |
|---|---|---|---|---|---|---|---|
| [Hopcroft and Karp, SIAM J. Comput. 2(4), 1973](https://doi.org/10.1137/0202019) | DOI `10.1137/0202019`, pp. 225–231 | publisher copyright; algorithmic reference | canonical published algorithm | layered augmenting paths and `O((m+n)sqrt(n))` sequential bound | `REFERENCE_ONLY`; informs Candidate A exactness and certificate language | none / `0 files, 0 lines` | revisit only if Candidate A graph ontology changes |
| [NetworkX 3.6.1](https://github.com/networkx/networkx/releases/tag/networkx-3.6.1) | tag object `9fca49c5bc01e2f3f0faf1c32da895c98695c7e5`; `hopcroft_karp_matching` | BSD-3-Clause; the matching module also records upstream CC-BY-SA/public-domain provenance | signed 3.6.1 release, active upstream | maintained CPU maximum-cardinality matching oracle | `REFERENCE_TOOL_ONLY`; temporary external proof environment, never committed or linked to product | no Cargo/runtime dependency; package not copied into repository; `0 files, 0 lines` | re-audit exact version/license if the proof is rerun or a dependency is proposed |
| [Shiloach–Vishkin, Journal of Algorithms 3(1), 1982](https://doi.org/10.1016/0196-6774(82)90008-6) | DOI `10.1016/0196-6774(82)90008-6`, pp. 57–67 | publisher copyright; algorithmic reference | canonical parallel-connectivity paper | tree hooking and pointer jumping with logarithmic PRAM rounds | `REFERENCE_ONLY`; Candidate B clean-room feasibility pattern, not a WGSL port | none / `0 files, 0 lines` | a production proposal must mechanically prove its exact bounded variant |
| [NVIDIA RAPIDS cuGraph 26.08](https://docs.rapids.ai/api/cugraph/stable/api_docs/api/cugraph/cugraph.connected_components/) | tag `v26.08.00`, object `cdda1b792e347dd5e92cc221a49496b019482718` | Apache-2.0 | current stable 26.08 official docs | maintained GPU connected-component API, 32-bit vertices, CSR/graph input and per-vertex labels | `REFERENCE_ONLY`; confirms the mechanism is standard but CUDA/CSR stack does not fit dense wgpu state | no dependency; no CUDA/RAPIDS ingress; `0 files, 0 lines` | revisit only after a separately approved backend/dependency decision |
| [wgpu 26.0.1](https://docs.rs/wgpu/26.0.1/wgpu/struct.Limits.html) | Cargo.lock `wgpu 26.0.1`, checksum `70b6ff82bbf6e9206828e1a3178e851f8c20f1c9028e74dd3a8090741ccd5798`; upstream tag `v26.0.1` object `0c978d0d46c7eaf68f6d1ecf2ea1bd03a96b7c47` | MIT OR Apache-2.0 | project-pinned supported API family | default eight storage buffers per shader stage and 128 MiB storage-binding size | `REUSE_EXISTING_API`; all candidate passes stay at or below eight bindings | existing dependency only; no change / `0 files, 0 lines` | re-audit on wgpu major/limit/profile change |
| [Powdergame TE-2 source](../../engine/gpu/src/air_flow_scale.wgsl) | repository baseline `f5b146571f2cb95b89d56d8831b68ddbeb75f395` | repository license | current accepted production source | donor scaling, Current/Next conservative Air movement, activity and profiler pattern | `REUSE_PATTERN`; Candidate C comparison only | no new dependency / external copy none | invalidate if TE-2 transport ownership or settlement changes |
| [Powdergame thermal reuse survey](2026-08-20-thermal-environment-reuse-survey.md) | source-bound 2026-08-20 survey | mixed, per recorded source | current repository research authority | clean-room boundary for Powder Toy, sandspiel and solver libraries | `REUSE_RECORD`; no new formula/code authority | `0 files, 0 lines` | revisit only at its recorded gates |
| [Chinese-community intake](2026-08-20-chinese-community-reuse-survey.md) | user intake recorded 2026-08-20 | per-source/unknown as recorded | preserved research input | fixture aliases only | `REFERENCE_INPUT_ONLY`; no matching/CCL/transport implementation authority | `0 files, 0 lines` | exact upstream and license required before stronger use |

## Architecture fit findings

Maximum matching is the exact mathematical answer to Candidate A's discrete
source/extent ontology, but the paper/library does not supply a bounded wgpu
path-flip protocol, Air receiver transaction or activity contract. The CPU
oracle therefore validates cardinality only.

Connected components are a standard shared-chamber primitive. cuGraph proves
that maintained GPU implementations exist, not that CUDA CSR code can be
copied or that its performance transfers to a full-resolution changing grid.
Candidate B must own label construction, deterministic component reduction,
pressure relaxation and all memory/pass costs in Powdergame terms.

Powdergame already has the relevant conservative Current/Next transport
grammar for Candidate C. That reuse does not solve the ownership question:
after Vapor volume diffuses away from a condensing phase Cell, a local sink may
be too small. Clamping loses volume, signed debt violates the non-negative
field contract, and component-wide withdrawal becomes Candidate B.

No surveyed source changes the common 1:1 phase-family quantity or supplies a
fourth model. No external implementation text or formula has entered runtime.
