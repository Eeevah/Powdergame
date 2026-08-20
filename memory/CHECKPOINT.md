# Checkpoint — TE-5B phase-volume bridge DESIGN BLOCKED — 2026-08-21 02:33 KST

## Repository coordinate

- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Start / TE-3D architecture source:
  `d7500e219af6f670be05f830b50c232d2bb53077`
- TE-2 production source:
  `fb7e568e21012b6067269f4e1b82c36c865023d0`
- D-019 authorization commit:
  `f1ca48cc01a906bfb4a997c72bc2744b81546ccd`
- Final design/checkpoint coordinate: the docs/memory commit containing this
  file; production Rust/WGSL/Cargo remain byte-identical to the start source

## Current truth

G9-A and TE-2 remain **USER ACCEPTED WITH KNOWN FOLLOW-UP**. TE-3D remains
**ARCHITECTURE ACCEPTED WITH LOCKED AMENDMENTS**, and ADR-0006 remains
**ACCEPTED FOR FUTURE ATOMIC IMPLEMENTATION**. Those statuses do not authorize
runtime.

D-019's named **EXCLUSIVE LOCAL VOLUME-RELIEF TOKEN + EXISTING GAUGE PRESSURE**
candidate is **DESIGN BLOCKED**. ADR-0007 remains **PROPOSED — DESIGN BLOCKED /
USER ARCHITECTURE REVISION REQUIRED**. TE-3 runtime, TE-5B runtime, full TE-5,
Air-pressure force, TE-4 and G9-B/C/D/E are **NOT STARTED**. The historical G5
Water path remains active.

## Exact blocker

In a sealed one-Cell-wide column, only the top Water is ready at `t0`; each
lower Water is initiated just below `Lv` and reaches the endpoint only after
ordinary movement brings the EMPTY vacancy above it. The first 1:1 Steam
completion wins zero pressure, then moves into the EMPTY Cell and vacates its
source. The next Water reaches `Lv`, wins that new up-EMPTY Cell and repeats.
Staggering avoids a simultaneous lower Steam-swap stop. The vacancy walks down
the column, so finite headspace never has to become blocked and every completion
can receive zero phase-volume pressure.

Same-tick exclusivity is therefore not cross-tick capacity ownership. Repair
requires capacity/reservation state, target or Environment mutation, additional
occupied quantity, or another pressure law. Each changes a locked premise or
reopens a rejected option. No replacement was silently selected. TE5B-F05,
TE5B-F11 and PV-INV-018 are unsatisfied by the current candidate.

## Review and evidence

- Fresh-context review:
  [`TE5_PHASE_VOLUME_PRESSURE_BRIDGE_DESIGN.md`](../docs/adversarial-reviews/TE5_PHASE_VOLUME_PRESSURE_BRIDGE_DESIGN.md),
  SHA-256 `78d2e70c852d26734d42e45fce091f308f53cbb8069c42080e066b02c83d2ce5`.
- Review findings: Critical `0`; High `3` recorded / `1` open; Medium `4`
  recorded / `3` open; Low `1` open; Info `1` evidence obligation.
- Completion-attempt ordering and production GAS First-Match/Void/swap
  mismatches were corrected and independently marked resolved. The vacancy-
  conservation High remains open.
- The fixed-seed pure model executed exactly once with seed `0x54453542`,
  `100,000` unique trials and two same-process deterministic replays. It returned
  `PASS_REFERENCE_MODEL_ONLY`, failed checks `0` and counterexample `null`.
- Script/result SHA-256:
  `6fd9276933822db850bd4ec3f9648cf64c45b8905f6b37d17cc88d03cb23a340` /
  `f53173af05199916b10d287a02c8193e9f86c40c853c019db8491cb86ff56e59`.
- That proof modeled encoding/arbitration/accounting only. It had no grid,
  vacancy conservation, production descriptor generation, WGSL, GPU, movement,
  pressure propagation, rupture, venting, sleep, performance or user evidence.
  It was not rerun after the independent counterexample.
- Static packing remains 40 passes, 80 queries, 1,280 profiler bytes, eight or
  fewer storage bindings and zero candidate memory delta, but only for the
  semantically blocked model. A replacement is not proven to preserve them.
- External implementation copied, translated or vendored: `0 files / 0 lines`.

## Repository and validation boundary

- Local personal-infra Wiki was user-dirty/behind and was not changed. Verified
  connected `origin/main` object:
  `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3`.
- Strict development-policy audit passed. Validation-plan classified every
  `docs/*` path docs-only; it classified Ballast `memory/*` as unknown, while
  the canonical Ballast contract explicitly says memory/docs-only changes do
  not trigger Rust/GPU FULL, smoke, candidates or user acceptance.
- Development session
  `20260820T164950636Z-te5b-phase-volume-bridge-design-d7500e21` closed PASS in
  `2573.21774 s`, with FULL `0`, candidate `0`, target delta `0` bytes and only
  the external session-summary artifact delta `836` bytes.
- Cargo test/check/clippy, GPU test/run, workspace FULL, release build,
  application launch, TE-3 candidate, TE-5B candidate, full TE-5 candidate and
  G8/G8-C execution counts are all `0`.

## Waiting on user

1. revise or replace the exclusive token and choose how finite capacity is
   owned, including which no-new-state, target-non-mutation or 1:1 constraint,
   if any, may change;
2. approve or revise equal occupancy-only treatment of Atmospheric EMPTY and
   Vacuum EMPTY;
3. approve or revise `00 none / 01 Matter / 10 relief / 11 invalid` encoding;
4. retain or revise the inherited confinement impulse `100.0`;
5. confirm or revise the finite-headspace F05/F11 product meaning.

World-edge mode, full TE-5 background/structure coupling, Vacuum combustion,
runtime authorization and source-bound runtime/user evidence remain separate.

## Next first action

Do not implement TE-3 or TE-5B. Resume only after the user supplies an
architecture revision for finite-capacity ownership. Then reassess the option
set, invariants, passes/state and proof/review obligations as a new authorized
design unit; do not reuse the abstract PASS as evidence for the replacement.
