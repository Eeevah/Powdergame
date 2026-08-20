# Thermal Environment reuse and prior-art survey — 2026-08-20

- **Gate:** TE-0R
- **Repository source:** `f5c7ac8e76867f769cdf19d7f420432d8fef4509`
- **Decision rule:** reuse internal mechanisms first; import no external implementation during TE-0
- **Copied, translated, vendored external code:** **0 files / 0 lines**

## 1. Internal reuse inventory

| Powdergame mechanism | Classification | Thermal Environment use | Boundary |
|---|---|---|---|
| checked zeroed GPU buffer helper and `world_usage()` | `REUSE_AS_IS` | allocate Current/Next Environment buffers with existing storage/copy flags | no new allocation abstraction |
| `WorldLayout` checked byte arithmetic | `EXTEND` | add four buffers and exact tracked totals | keep historical reports versioned |
| Current/Next stage→copy→stage ordering | `REUSE_AS_IS` | Environment reconcile and joint settle follow the established causal pattern | no unsettled multi-writer Next |
| movement proposal/claim identity | `WRAP` | identify move/swap/Void Volume Exchange | never add Air bindings to maxed movement commit |
| phase/Smoke claim identity | `WRAP` | pair Matter target with a separate Environment receiver arbitration | current claim alone cannot receive displaced Air |
| reset and scenario/preset staging | `EXTEND` | stage exact Environment image beside existing Matter fields | one canonical staging API must cover every bypass path |
| Sandbox coalesced edit boundary and wake halo | `EXTEND` | add Draw/Erase Environment hygiene | preserve bounded command/submission behavior |
| activity/wake pipeline | `REFERENCE_ONLY` in TE-1; `EXTEND` in TE-2 | TE-2 adds Air frontier/wake semantics in a separate budgeted path | current activity pass already has eight storage bindings |
| timestamp profiler | `EXTEND` | assign every new reconcile/flow/thermal pass an identity and group | do not hide work in residual |
| persistent GPU allocation report | `EXTEND` | add exact Environment and proven scratch bytes | exclude transient driver-opaque storage as currently documented |
| Naga WGSL parse/write-contract tests | `EXTEND` | parse new shaders and pin writable bindings | handwritten semantic tests remain required |
| CPU movement reference rules | `EXTEND` | small-grid Volume Exchange and receiver arbitration reference | not a bit-exact GPU oracle |
| current CPU thermal rule | `REFERENCE_ONLY` | current 4-neighbor direction and sanitization are baseline evidence | it is Matter-only and not pair-conservative |
| 24-byte, ≤10-Hz Inspector collector | `WRAP` | preserve current product contract; use test-only Environment readback in TE-1 | payload expansion needs separate approval |
| `tools/dev.ps1 validation-plan` | `REUSE_AS_IS` | route later Engine/Core/WGSL changes to required validation | TE-0 remains docs-only |
| existing `pressure[]` | `REFERENCE_ONLY` | retain gauge-overpressure meaning | not Air mass, Air pressure, or an unconditional additive term |
| Erase as Vacuum | `NOT_APPLICABLE` | none | Erase and future Vacuum operation have different product semantics |

## 2. External survey

| Source | Exact commit/version | License | Maintenance signal | Mechanism examined | Powdergame use | Decision | Reason | Copied code? | Future revisit condition |
|---|---|---|---|---|---|---|---|---|---|
| [The Powder Toy](https://github.com/The-Powder-Toy/The-Powder-Toy/commit/2e47966b84b0d2f1750af0f82643791803537ea5) | HEAD `2e47966b84b0d2f1750af0f82643791803537ea5`; stable `v100.1.400` at `d768aeb89acad986bd252d7e904bf44bb374545f` | GPL-3.0 | stable release 2026-08-08 and later commit 2026-08-14 | separate pressure/velocity/ambient-heat grids, block maps, edge behavior, save-state fields | clean-room state separation, wall/edge/save/reset fixture ideas | `REFERENCE_FIXTURE` | coarse CPU grids differ from full-resolution GPU Current/Next; GPL code copying, translation and close porting are forbidden | no | only with separately documented clean-room formula/fixture provenance |
| [MaxBittker/sandspiel](https://github.com/MaxBittker/sandspiel/commit/dc77827b36adc5c04ea063515de4173ce28dbf2c) | `dc77827b36adc5c04ea063515de4173ce28dbf2c` | MIT | 2026-01 activity; no formal release discipline | separate Cell/Wind/Burn state, generation identity, fixed-seed path, EMPTY-only painting | edit and generation fixture ideas | `REFERENCE_FIXTURE` | CPU/WASM in-place state, JS/WebGL readback and mixed RNG do not fit Powdergame authority; fluid provenance also points to PavelDoGreat | no | bounded UI/edit fixture design only |
| [gfx-rs/wgpu](https://github.com/gfx-rs/wgpu/commit/6f8edda9b180efada6a24b6d40c567c57f59e9ea) | major-26 latest patch `v26.0.6`, commit `6f8edda9b180efada6a24b6d40c567c57f59e9ea`; trunk `bbac60d...` | MIT OR Apache-2.0 | active trunk 2026-08-19 | storage binding modes, ordered submissions, staging copies, error scopes, timestamp resolve | existing wgpu 26 APIs and Powdergame wrappers | `REUSE_DIRECT` | same maintained API family and Windows/DX12 path; adapt official contracts through existing abstractions rather than copy examples | no | verify `Cargo.lock` patch and migration impact before a same-major update |
| [proptest](https://github.com/proptest-rs/proptest/commit/7f1367f9a4dc8440c47b93166a38ed064f63ea8c) | `1.11.0`, commit `7f1367f9a4dc8440c47b93166a38ed064f63ea8c`; main `a4ad984...` | MIT OR Apache-2.0 | passive maintenance; release 2026-03 and soundness fix on main 2026-07 | generated states, shrinking, fixed seed, persisted failures, bounded case/time config | TE-1 dev-dependency candidate for algebraic properties | `REUSE_DIRECT` | suitable as a supplement, not a replacement for semantic fixtures; requires Rust 1.85 and bounded deterministic config | no | use fixed seed/case/shrink limits and disable `hardware-rng` until a release contains the audited fix |
| [WebGL Fluid Simulation](https://github.com/PavelDoGreat/WebGL-Fluid-Simulation/commit/a2d292931f19d9b3b9f564e23e6c32729d2121c3) | `a2d292931f19d9b3b9f564e23e6c32729d2121c3` | MIT | last commit 2024-11 | semi-Lagrangian advection, divergence, pressure projection, ping-pong fields, boundaries | conceptual comparison only | `REFERENCE_ONLY` | full incompressible visual CFD and velocity field exceed TE scope | no | only after a separately approved velocity/CFD program |
| [dimforge/salva](https://github.com/dimforge/salva/commit/51c153e05974836eea17db445e30f1ff64bb5b32) | `v0.10.0`, commit `51c153e05974836eea17db445e30f1ff64bb5b32` | Apache-2.0 | active release 2026-08-08 | SPH/PBF particle fluids, pressure/viscosity/surface tension | none in current grid architecture | `REJECT` | particle neighbor search, CPU/nalgebra/Rapier orientation and pre-1.0 API do not fit dense 2048² wgpu state | no | only if Powdergame explicitly adopts a particle-fluid subsystem |
| [bevy_eulerian_fluid](https://github.com/narasan49/bevy_eulerian_fluid/commit/105f3a2702b681cae4edd6d693508bcc55543f78) | `0.4.0`, commit `105f3a2702b681cae4edd6d693508bcc55543f78` | MIT OR Apache-2.0 | 2026-04 release; 2026-05 HEAD activity | GPU incompressible grid solver and pressure iteration | architecture comparison only | `REFERENCE_ONLY` | Bevy dependency stack and many-pass full solver are larger than the approved mass/energy slice | no | only after an explicit full velocity/pressure solver decision |
| [hatoo/apd](https://github.com/hatoo/apd/commit/922ef43e47e6e0169ebba80f10fc0f78c1f2d570) | `v0.0.1`, commit `922ef43e47e6e0169ebba80f10fc0f78c1f2d570` | MIT | dormant since 2021 | CPU ndarray Advect/Project/Diffuse primitives | standard-algorithm comparison only | `REFERENCE_ONLY` | no current GPU/wgpu integration or maintenance | no | formula comparison only, never as runtime dependency |
| [stroemung](https://github.com/wickedchicken/stroemung/commit/369a4dc9983922bdeb9b2bc3032baaf884829cf5) | `v0.1.2`, commit `369a4dc9983922bdeb9b2bc3032baaf884829cf5` | MIT | small 2025 project; upstream marks implementation/tests WIP | CPU 2D CFD, boundaries and snapshots | general boundary-fixture idea | `REJECT` | WIP CPU solver and pressure architecture do not fit the local Environment contract | no | only if maintenance, GPU path and determinism contracts materially change |
| [`fluid_core` crate](https://crates.io/crates/fluid_core) | `0.1.1`; no auditable source commit/repository | registry says MIT | recent but only 46 observed downloads and no upstream binding | advertised wgpu fluid core | none | `REJECT` | exact source, provenance, API and tests cannot be audited | no | public versioned upstream plus license/test provenance required |

## 3. Adopted reuse boundary

Direct reuse is limited to Powdergame's existing mechanisms and the official wgpu 26 API pattern already in use. `proptest 1.11.0` is only a future audited dev-dependency candidate. The Powder Toy and sandspiel contribute clean-room fixture and state-separation ideas, not code. CFD and particle crates are deliberately rejected or reference-only because they solve a larger and different problem.

The survey does not authorize a Cargo dependency, a runtime implementation, or a solver choice. Any later dependency must record the exact release/commit, license, feature set, MSRV, maintenance/security status, rollback cost and evidence impact at the implementation source boundary.

## 4. Later Chinese-community research intake

The user-supplied Chinese-community milestone mapping is recorded separately in
[`2026-08-20-chinese-community-reuse-survey.md`](2026-08-20-chinese-community-reuse-survey.md).
It contributes one adopted TE-2 regression contract and provenance aliases for
existing fixtures, with copied code fixed at `0 files / 0 lines`. It does not
reopen this survey's architecture or license decisions.
