# Thermal Environment Validation Contract

- **Status:** Canonical design validation; runtime implementation not started
- **Architecture:** ADR-0005 / `THERMAL_ENVIRONMENT_SPEC.md`
- **Principle:** automated evidence proves invariants and named causal claims, not fun or user acceptance

## 1. Validation layers

1. **Reference math** — finite algebra, symmetry, conservation and canonicalization on small CPU/Python states.
2. **Structural GPU validation** — WGSL parsing, binding/write ownership, Current/Next settle and adapter-limit guards.
3. **GPU semantic fixtures** — production WGSL in bounded worlds.
4. **Cross-pass causality** — named source/tick/Cell chains across movement, Environment, thermal, phase, combustion and pressure.
5. **Product observation** — user-only judgment in Sandbox.
6. **Performance** — only after correctness source sealing; historical G8/G8-C evidence is not rebound.

## 2. Required property classes

- passive local and global maximum principle;
- equal-and-opposite face energy-like balance;
- non-negative finite Air mass and energy;
- donor outflow bounded by available mass;
- coefficient-domain rejection and post-rounding non-negative donor guard;
- source-free Air mass and advected-energy conservation;
- exact-zero Vacuum canonicalization with no positive residual deletion;
- Volume Exchange and spawn-receiver conservation;
- no false standard Atmosphere/ordinary fluid differential;
- one reconcile per occupancy event and no duplicate Air advection/conduction;
- exact reset/staging of both Environment halves;
- sleep-on/off semantic equivalence within the declared tolerance.

Property generators use deterministic seeds, bounded cases/shrinking and persisted failures. A future `proptest` dev-dependency may supplement handwritten semantic fixtures only after license, Rust 1.85+ toolchain, default-feature and bounded-runtime review. It never replaces named fixtures.

## 3. Semantic fixture matrix

| ID | Fixture | Required result |
|---|---|---|
| TE-F01 | Two-Matter equalization | monotonic convergence; no source-free overshoot |
| TE-F02 | Hot Stone to Oil with combustion disabled | Oil never exceeds current participating maximum |
| TE-F03 | Four hot neighbors around cold center | local convexity and energy-like balance |
| TE-F04 | Source removal | target stops rising and cools after source removal |
| TE-F05 | 1/3/8-Cell Atmospheric gap | finite transfer with distance-dependent onset |
| TE-F06 | Same Vacuum gap | Air-mediated transfer exactly zero |
| TE-F07 | Direct contact versus Air gap | direct contact is faster under selected coefficients |
| TE-F08 | Ambient opening | hot Matter loses energy toward reservoir |
| TE-F09 | Sealed equivalent | retains more energy than TE-F08 |
| TE-F10 | One-Cell wall | Air mass leak zero; only wall conduction allowed |
| TE-F11 | One-Cell opening | Air/heat flux begins only after opening |
| TE-F12 | Atmosphere refill | finite non-negative Air enters connected Vacuum |
| TE-F13 | Vacuum vent | Air mass and derived pressure decrease |
| TE-F14 | Heated sealed Air | derived pressure rises; uniform faces have zero net force |
| TE-F15 | Atmosphere/Vacuum membrane | differential direction correct; no double count |
| TE-F16 | Steam in cool Air | Steam loses energy |
| TE-F17 | Cold-lid condensation | surface route is faster than free cloud route |
| TE-F18 | Open beaker | named boil→rise→cool→condense→fall chain |
| TE-F19 | Hot sealed beaker | Steam persists longer than TE-F18 |
| TE-F20 | Phase hysteresis/reversal | no ping-pong; pending energy returned/absorbed |
| TE-F21 | Brief Oil pulse | no ignition; exposure decays |
| TE-F22 | Sustained Oil preheat | bounded delayed surface ignition |
| TE-F23 | Brief Wood pulse | no ignition |
| TE-F24 | Sustained Wood preheat | bounded delayed surface ignition, slower than Oil baseline |
| TE-F25 | Flame contact | explicit flame exposure is distinguishable from inert hot Matter |
| TE-F26 | Buried fuel | no simultaneous whole-bulk ignition |
| TE-F27 | Combustion source | positive chemical source explains temperature above initiator |
| TE-F28 | Vacuum combustion placeholder | blocked until explicit user policy |
| TE-F29 | Movement heat carry | Matter temperature/flags/progress follow ownership edge |
| TE-F30 | Occupancy hygiene | movement, Draw, Erase, decay and rupture leave no stale Air |
| TE-F31 | 2048² equilibrium bulk | no broad active tail; tracked bytes and pass cost exact |
| TE-F32 | Standard Air / zero-gauge fluid | effective pressure difference zero |
| TE-F33 | Dense Steam/Smoke curtain | documented displacement; no silent leak; permeability question evidence-based |
| TE-F34 | Phase energy-like budget | progress, reversal, completion and yield bounded |
| TE-F35 | Reconcile inventory | every ownership path reconciles exactly once |
| TE-F36 | Spawn with receiver | target Air moves to the unique claimed receiver without loss |
| TE-F37 | Spawn without receiver | phase remains blocked; Smoke request rejected; target Air unchanged |
| TE-F38 | Expansion target wins but receiver fails | Matter/Air unchanged; existing blocked-expansion pressure source applied exactly once |

## 4. Named acceptance chains

The beaker contract requires a causally ordered chain with source/tick/region identity: heat input, surface phase progress, Steam creation, upward movement, measured cooling, condensation progress, latent release accepted by a sink, Water creation in the cooler region, and subsequent downward motion. Merely observing both Water and Steam is insufficient.

Ignition validation distinguishes a brief pulse, sustained preheat, source removal, first surface ignition, exposure progress, chemical heat source, passive outflow and fuel progress. A threshold crossing alone is not ignition evidence.

## 5. Structural machine guards

- Naga parses every new WGSL module.
- Writable bindings exactly match each pass's declared outputs.
- No compute stage exceeds the observed eight-storage-buffer ceiling.
- Each stage has an explicit settle boundary before later writers/readers.
- Ownership event and Environment reconcile counts match.
- Spawn receiver arbitration has at most one winner and blocks cleanly without one.
- receiver candidates exclude all same-stage Matter destinations and reject a
  whole parcel when mass or energy headroom is insufficient;
- every original expansion winner with a failed receiver gets exactly one
  existing blocked-expansion pressure source before pressure settle; ordinary
  losers/blocked proposals are not double-counted;
- one `u32` receiver scratch remains live independently of the original Matter
  claim; paired Matter/Environment commit and rollback are structural guards;
- material flag ownership is pinned to combustion bits 0–1/4–15, Smoke decay
  bits 16–27 and zero reserved bits; identity hygiene runs before joint settle;
- bilateral face cohorts prevent one-sided flow across sleeping chunk seams;
- sealed TE-2 runtime edges and fixed-reservoir fixture edges are not mixed;
- `proposal`/`claim` scratch reuse is permitted only after a live-range structure test.
- `cell_activity` is never treated as float scratch.
- profiler pass identity/query count/group reconstruction includes Environment work.
- one canonical staging helper covers world creation/reset, scenarios, benchmark calibration, direct test hooks and Sandbox presets.
- Inspector payload remains 24 bytes and cadence remains at most 10 Hz until separately approved.

## 6. Reference-proof status

The verified TE-0.2 package proof used fixed seed `1413820466` and 20,000 thermal plus 20,000 Air-flow randomized cases. It found zero upper/lower overshoot; maximum energy-like and advected-energy error `7.275957614183426e-12`; maximum mass error `7.105427357601002e-15`; non-negative outputs; zero false standard pressure difference; and exact fixed-case Volume Exchange preservation.

This proves only the candidate formulas inside the sampled domain. The design
review subsequently locked coefficient domains and exact-zero Vacuum because
the script did not prove those guards. It does not prove WGSL, GPU race
freedom, 2D four-face donors, walls/reservoirs, spawn arbitration,
phase/ignition coefficients, pass ordering, bindings, sleep, performance, fun,
or user acceptance. Those limitations are mandatory report content.

## 7. Gate-specific validation economy

- TE-0 is docs-only: no Cargo tests, FULL, GPU run, release build, bounded launch check, candidate or official capture.
- Engine/Core/WGSL/layout gates run targeted tests plus one final FULL only when required by `VALIDATION_POLICY.md`.
- Semantic candidates run at most once per explicitly authorized gate.
- G8 and G8-C artifacts remain historical and are never rerun or rewritten for this program.
- Same-source evidence is reused only with matching command, toolchain/profile, config, hardware/backend and artifact identity.

## 8. Performance evidence boundary

Correctness-source measurement records exact persistent bytes, any non-reused scratch, pass/bind-group inventory, per-pass P50/P95, active chunks/tail, 60-TPS responsiveness and reset/source identity. It measures 256² product and 2048² reference configurations.

No coarsening, packing, f16 or solver optimization begins until the full-resolution correctness baseline demonstrates a measured blocker and the user authorizes a bounded optimization review.
