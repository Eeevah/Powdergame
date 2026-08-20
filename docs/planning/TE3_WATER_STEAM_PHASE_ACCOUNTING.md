# TE-3 Water / Steam Phase Accounting

- **Status:** PHASE-ENTHALPY DESIGN CANDIDATE / USER ARCHITECTURE REVIEW PENDING
- **Registered from:** direct Sandbox observation after TE-2 review
- **Audited production-physics source:** `fb7e568e21012b6067269f4e1b82c36c865023d0`
- **Design baseline:** `94b152e85ff6f5481a033d885d38dca0dbc1043a`
- **Runtime implementation authorized:** no

This page is the review entry point for the TE-3D Water/Steam design. The
normative candidate is split across:

- [`ADR-0006`](../architecture/decisions/ADR-0006-water-steam-phase-enthalpy.md)
  for option selection and consequences;
- [`PHASE_THERMODYNAMICS_SPEC`](../specs/PHASE_THERMODYNAMICS_SPEC.md) for the
  proposed state, equations, writers, pass order and invariants;
- [`PHASE_THERMODYNAMICS_VALIDATION`](../development/PHASE_THERMODYNAMICS_VALIDATION.md)
  for fixtures and the one-shot reference-math receipt;
- [`TE3_PHASE_ENTHALPY_DESIGN`](../adversarial-reviews/TE3_PHASE_ENTHALPY_DESIGN.md)
  for the independent adversarial disposition.

The proposal is reviewable, not accepted. No Rust, WGSL, Cargo manifest,
build, launch, runtime evidence or later gate changed in TE-3D.

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

Proposed latent coefficients are `L_f = 80` and `L_v = 480` energy units per
Cell. Canonical phase-energy endpoints are Ice `-80`, Water `0`, Steam `480`;
valid identity ranges are Ice `[-80, 0]`, Water `[-80, 480]`, Steam `[0, 480]`,
and all other/EMPTY Cells `0`. Partial progress remains owned by the current
source identity until an endpoint is reached. Reversal consumes that same
stored progress before temperature departs the plateau. Normalization is
local-only after TE-2 has transferred passive heat, so no neighbor exchange is
counted twice.

The candidate preserves the current initiation thresholds and hysteresis:
Ice melts above `2 C`, Water freezes below `-2 C`, Water boils above `100 C`,
and Steam becomes condensation-eligible below `95 C`. Buried superheated Water
may accumulate boiling progress but cannot mint volume. Supercooled Water may
accumulate freezing progress; identity changes only at the endpoint.

## 4. Condensation and appearance contract

Surface condensation requires an orthogonally adjacent compiled condensed
phase/non-empty non-GAS Matter Cell whose actual temperature is at most `80 C`
and at least `10 C` colder than the Steam Cell. When no such surface exists,
free-air condensation is eligible only below `70 C` and only at the strict
minimum `(hash32, y, x)` key among the eligible eight-neighbor region. A seed
must have a real TE-2 energy-removal face. Thermally runnable partial Steam
vetoes adjacent new seeds, closing the next-tick cascade; stalled partial
progress retains E but does not reserve its neighbours forever. The hash seed
and mixer are fixed in the specification.

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
the generic non-family `yield > 1` path remains representable. No phase-energy
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

The validation contract names deterministic fixtures for closed-cycle
quantity, reversal, surface boiling, sealed response, cold-surface and free-air
condensation, movement, editor/reset hygiene, sleeping seams, generic-yield
regression, pass/binding limits and TE-2 non-regression. The pure reference
proof passed once over 50,000 randomized enthalpy cases, 4,096 finite
nucleation regions and 100 reversal cycles. It is math-only evidence: no WGSL,
GPU, binding, movement, sleep, performance, visual or user-acceptance claim was
made.

## 7. User architecture review checklist

The next decision is to accept or revise the candidate. Review these choices:

- Hybrid A+C: one Cell equals one quantity; no expansion fragment;
- `L_f = 80`, `L_v = 480` and the canonical enthalpy ranges;
- cold-surface limits `80 C` / `10 C` and free-air threshold `70 C`;
- deterministic local nucleation and its expected visual texture;
- two dense phase-energy buffers and the approximately 32 MiB 2048² cost;
- the explicit deferral of Steam pressure-volume force to TE-5;
- the atomic-activation constraint that prevents a temporary G5
  expansion/confinement regression;
- the proposed pass/writer/reset/sleep contract.

Acceptance of this design would authorize only the separately stated next
gate. It would not retroactively strengthen TE-1/TE-2 evidence or authorize
TE-4, TE-5, G9-B/C/D/E, build, launch, capture, merge or `main` promotion.

## 8. Stop boundary

TE-3D stops at **PHASE-ENTHALPY DESIGN CANDIDATE / USER ARCHITECTURE REVIEW
PENDING**. TE-3 runtime, Air-pressure force, TE-4 ignition, TE-5 integration
and G9-B/C/D/E remain **NOT STARTED**.
