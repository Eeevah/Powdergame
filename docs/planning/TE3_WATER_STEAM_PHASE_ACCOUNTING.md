# TE-3 Water / Steam Phase Accounting

- **Status:** ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS
- **Registered from:** direct Sandbox observation after TE-2 review
- **Audited production-physics source:** `fb7e568e21012b6067269f4e1b82c36c865023d0`
- **Design baseline:** `94b152e85ff6f5481a033d885d38dca0dbc1043a`
- **Runtime implementation authorized:** no

This page is the review entry point for the accepted TE-3D Water/Steam
architecture. Its normative contract is split across:

- [`ADR-0006`](../architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md)
  for option selection and consequences;
- [`PHASE_THERMODYNAMICS_SPEC`](../specs/PHASE_THERMODYNAMICS_SPEC.md) for the
  proposed state, equations, writers, pass order and invariants;
- [`PHASE_THERMODYNAMICS_VALIDATION`](../development/PHASE_THERMODYNAMICS_VALIDATION.md)
  for fixtures and the one-shot reference-math receipt;
- [`TE3_PHASE_ENTHALPY_DESIGN`](../adversarial-reviews/TE3_PHASE_ENTHALPY_DESIGN.md)
  for the independent adversarial disposition.
- [`TE5_PHASE_VOLUME_PRESSURE_BRIDGE`](TE5_PHASE_VOLUME_PRESSURE_BRIDGE.md)
  for the rejected/blocked token attempt;
- [`TE5_LOCAL_VAPOR_CAPACITY_PRESSURE`](TE5_LOCAL_VAPOR_CAPACITY_PRESSURE.md)
  for the D-020 replacement and its one-shot DESIGN BLOCKED result.

D-018 accepts Hybrid A+C, its constants, memory cost, no-sink metastability
and atomic TE-5 activation constraint. The v1 ADR text is not accepted
unchanged: real-sink, buried-completion, radius-2 nucleation, generic-target
and hash-provenance amendments required one new reference proof and fresh
independent review. The reference proof passed its only run and the fresh v2
review closed with unresolved Critical `0` / High `0`. No Rust, WGSL, Cargo
manifest, build, launch, runtime evidence or later gate changes occurred in
TE-3D.

## 1. Direct user observation and audited defect

Air transport and cooling are visibly active in Sandbox. Steam rises and can
cool into Water, so the causal direction is understandable. The resulting
motion is not acceptable phase presentation or accounting:

- rising Steam and falling Water interleave;
- large blue/white checkerboard clumps remain suspended in mid-air;
- the cause can be followed, but the resulting volume and shape are unnatural.

The audited production path is:

```text
1 Water above 100 C
-> source becomes Steam
-> matter_yield = 2 may create one additional independent Steam Cell
-> each Steam below 95 C independently becomes one Water
-> up to 2 Water
```

The source and spawned Cell preserve temperature. No latent-energy debit,
credit or phase-progress state exists. Therefore an available-space cycle can
gain Water-equivalent Cells. This is a TE-3 accounting defect, not a request to
retune TE-2 Air flow or passive-thermal coefficients.

## 2. Compared representations

| Option | Quantity/accounting model | Strength | Blocking cost or failure | Disposition |
|---|---|---|---|---|
| A — one Matter Cell plus Environment expansion | one Water-equivalent quantity remains one occupied Matter Cell; expansion is pressure/Environment response | smallest identity and conservation surface; removes duplicate Water creation | visible expansion cannot be faked by extra independent Steam; pressure response remains a later TE-5 integration | selected half of Hybrid A+C |
| B — primary Steam plus bounded expansion fragment | primary owns an explicit secondary occupied fragment | can show multi-Cell volume directly | adds ownership, movement, merge, reset and orphan hazards; a fragment can accidentally become a second Water quantity | rejected |
| C — dedicated bounded phase state | latent/progress state is separate from identity | makes partial, reversible transition energy explicit | a dense state has allocation, writer, movement and sleep cost | selected only as per-Cell phase enthalpy, not quantity/fragment state |
| existing yield-2 identity expansion | two independent Steam identities | already visible | violates closed-cycle quantity and causes checkerboard traffic | rejected for Water/Steam |

The candidate is **Hybrid A+C**:

1. one occupied Water/Steam Cell is exactly one Water-equivalent quantity;
2. Water's future phase yield is `1`, with no fragment or mixed-quantity state;
3. two full-world `f32` phase-energy Current/Next buffers carry reversible
   sensible/latent enthalpy while the Matter identity stays Water, Ice or Steam;
4. boiling volume/confinement is represented by Environment/pressure response,
   not a second independent Steam Matter Cell;
5. surface condensation is preferred; free-air condensation uses a bounded,
   deterministic local nucleation predicate so a whole cloud cannot flip at once.

## 3. Candidate accounting contract

For one Cell quantity, the conserved phase coordinate is:

```text
H = S_phase(T) + E_phase

S_ice(T)   = 2.0 * (T - 0)
S_water(T) = 2.5 * (T - 0)
S_steam(T) = 2.5 * 100 + 0.8 * (T - 100)
```

Locked latent coefficients are `L_f = 80` and `L_v = 480` energy units per
Cell. Canonical phase-energy endpoints are Ice `-80`, Water `0`, Steam `480`;
valid identity ranges are Ice `[-80, 0]`, Water `[-80, 480]`, Steam `[0, 480]`,
and all other/EMPTY Cells `0`. Partial progress remains owned by the current
source identity until an endpoint is reached. Reversal consumes that same
stored progress before temperature departs the plateau. Normalization is
local-only after TE-2 has transferred passive heat, so no neighbor exchange is
counted twice.

The architecture preserves the current initiation thresholds and hysteresis:
Ice melts above `2 C`, Water freezes below `-2 C`, Water boils above `100 C`,
and Steam becomes condensation-eligible below `95 C`. Initiated positive-E
Water remains Matter-owned after burial. It may heat or reverse, but completion
requires a current gas-facing surface or explicit acceptance from a future
TE-5 transaction. At E=480 without either, it remains vaporization-ready Water
and stores excess H as Water superheat. A real context later converts 1:1 from
the same H. Supercooled Water may accumulate freezing progress; other identity
changes remain endpoint-owned.

## 4. Condensation and appearance contract

Surface condensation requires an orthogonally adjacent compiled condensed
phase/non-empty non-GAS Matter Cell whose actual temperature is at most `80 C`,
at least `10 C` colder than Steam, has positive shared TE-2 conductance and
passes the exact TE-2 energy-removal work predicate. A cold K=0 Boundary is not
a sink or activity source. Atmosphere/Vacuum remain non-surface routes.

When no surface sink exists, free-air condensation is eligible only below
`70 C` and only at the strict minimum `(hash32, y, x)` key in a 5×5 Chebyshev
neighbourhood. Seed and active-partial veto radius are both exactly two.
Thermally runnable partial Steam closes the next-tick cascade; stalled no-work
progress retains E but does not reserve space. The hash reuses Powdergame's
existing internal arbitration finalizer/constants with a newly documented
x/y/TE-3-tag input mapping.

TE3-F08 predeclares that every sampled 30-tick window has new initiations no
greater than `max(4, ceil(peak eligible canonical Steam / 8))`. Radius 1/2/3
are disclosed by the amended reference sweep, but radius 2 is normative and a
failure blocks the design rather than silently choosing radius 3. Canonical
Steam without a positive-conductance removal face, or partial Steam without
runnable thermal work in either direction, may remain metastable indefinitely
and sleep; no spontaneous condensation is added.

This predicate chooses bounded local seeds, not a random deletion or global
cloud switch. The phase-energy state moves with its Matter owner, and the
validation contract rejects orphaned state, region-wide same-tick conversion,
persistent checkerboard regions and a permanently awake world.

## 5. GPU feasibility and cost projection

The proposed command graph adds exactly two dense `f32` buffers and no generic
scratch buffer. It projects 40 production passes, 80 timestamp queries and a
1,280-byte profiler readback (`+6` passes, `+12` queries and `+192` bytes from
TE-2). A seven-storage `phase_context_propose` pass reuses dead claim scratch to
freeze Matter/Air/surface/thermal-work markers, so Atmosphere and Vacuum are
distinguished without binding Air to the maxed phase pass. The following
phase-thermodynamics bind group has eight storage bindings plus two uniforms;
movement reconcile has seven storage bindings plus parameters; all other
proposed phase hygiene/activity passes are at or below the eight-storage
ceiling.

Projected tracked allocations are:

| World | Without profiler | With profiler |
|---:|---:|---:|
| 256² | 4,721,328 bytes | 4,722,608 bytes |
| 2048² | 302,016,816 bytes | 302,018,096 bytes |

The approximately 32 MiB increase at 2048² is the explicit price of two phase
energy halves. Reset/editor/scenario writers stage both halves canonically.
Movement reconciliation transfers the energy once with the Matter owner.
Identity-changing decay, combustion and phase writers clear or normalize it.
The context pass first fully overwrites dead `claim` with immutable `u32`
markers; the phase pass consumes them and fully overwrites `proposal` back to
its ordinary `u32` meaning
before the unchanged expansion chain: Ice/Water/Steam emit `NO_PROPOSAL`, while
the generic non-family `yield > 1` path remains representable only with a
non-phase target. A generic yield greater than one may not target Ice, Water or
Steam without a later approved destination phase-energy ownership/writer
design. No phase-energy
proposal aliases the earlier live `proposal`/`claim` float window.

Pressure remains a boundary: TE-3 proposes a `yield = 1` Water/Steam phase
transition and retains the existing blocked-expansion pressure path only for
other generic `yield > 1` materials. A new Steam pressure-volume force law is
not silently introduced; that integration remains TE-5-owned. To preserve the
frozen G5 boil/confinement/rupture/vent chain, the new Water rule cannot be
activated in any production/user-testable source until a separately authorized
TE-5 replacement is ready on that same source. Phase-only code could only be
disabled staging, not a candidate or temporary regression.

## 6. Design fixtures and evidence boundary

The validation contract now includes buried mid-progress/ready/reopen/TE-5
placeholder fixtures, a zero-conductivity Boundary control, isolated Steam,
radius-2 temporal movement/release, the 30-tick bound and non-phase generic
yield compatibility. The preserved v1 proof passed once over 50,000 randomized
enthalpy cases, 4,096 finite nucleation regions and 100 cycles; it is not
relabeled as v2. The separate amended fixed-seed reference then passed its only
run with status `PASS_AMENDED_REFERENCE_MATH_ONLY`, script SHA-256
`c3624e467638a62ef2b62f96c8b12954ceef70609feeac47da70eca69f84db23`,
maximum H error `1.52587890625e-05` and normative radius-2 maximum of 209 new
initiations in any sampled 30-tick window. Neither receipt is WGSL, GPU,
binding, performance or visual evidence.

## 7. D-018 disposition and remaining gate

Accepted: Hybrid A+C, one Cell/one quantity, 1:1 transitions, two phase-energy
halves, no fragment/quantity/mixed state, local H after TE-2 Q, constants
80/480/80/10/70, 32 MiB at 2048², isolated-Steam metastability and atomic
same-source TE-5 activation.

Locked into future implementation authority: the real-sink predicate,
vaporization-ready completion gate, radius-2 rule and 30-tick bound, generic
phase-target restriction, internal hash provenance, the passed new reference
receipt and the independent v2 disposition `Critical 0 / High 0`. This user
disposition does not retroactively strengthen TE-1/TE-2 evidence or authorize
TE-3/TE-5 runtime, TE-4, G9-B/C/D/E, build, launch, capture, merge or `main`
promotion.

D-019 later authorizes only the docs/reference TE-5B design program. Proposed
[`ADR-0007`](../architecture/decisions/ADR-0007-phase-volume-pressure-bridge.md)
uses one exclusive EMPTY-only GAS movement-opportunity claim and the existing
gauge-pressure consequence without changing this page's accepted phase
quantity or enthalpy decisions. Independent review found the proposal cannot
consume finite headspace: 1:1 Steam movement vacates the source, so one EMPTY
vacancy can walk down a sealed Water column and grant zero-pressure completion
repeatedly. ADR-0007 is therefore design-blocked and does not authorize runtime.

## 8. Stop boundary

TE-3D stops at **ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS** and ADR-0006
at **ACCEPTED FOR FUTURE ATOMIC IMPLEMENTATION**. TE-3 runtime remains **NOT
STARTED**; the TE-5B pressure-volume bridge is **ADR-0007 PROPOSED / DESIGN
BLOCKED / RUNTIME NOT STARTED**; full TE-5 Air/background-pressure
force, TE-4 and G9-B/C/D/E remain **NOT STARTED**.

D-020 rejects that token and records the replacement
[`TE5_LOCAL_VAPOR_CAPACITY_PRESSURE`](TE5_LOCAL_VAPOR_CAPACITY_PRESSURE.md).
Its population/capacity model passed the vacancy-walk control but failed a
predeclared two-Steam/two-EMPTY asymmetric open-capacity case: proportional
shares were underused after a per-Cell cap, producing false target `100`.
TE-5C is also **DESIGN BLOCKED**. No accepted phase rule changes here; the next
architecture decision must explicitly permit persistent phase-volume state.

D-021 makes that explicit and evaluates
[`TE5_PERSISTENT_VAPOR_EXTENT`](TE5_PERSISTENT_VAPOR_EXTENT.md). The persistent
extent closes the old vacancy-reuse representation gap, but its frozen
depth-six matching cannot settle every canonical persistent state that has a
complete assignment. The eight-source alternating-chain counterexample makes
TE-5D **DESIGN BLOCKED** on wider matching scope. The accepted TE-3 phase
architecture remains unchanged and still cannot activate alone.
