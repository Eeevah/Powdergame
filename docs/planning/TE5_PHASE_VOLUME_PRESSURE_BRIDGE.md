# TE-5B Phase-Volume Relief / Confinement Bridge

- **State:** DESIGN BLOCKED / pure reference abstraction PASS / runtime NOT STARTED
- **Decision:** D-019
- **ADR:** [`ADR-0007`](../architecture/decisions/ADR-0007-phase-volume-pressure-bridge.md) — Proposed / DESIGN BLOCKED
- **Normative spec:** [`PHASE_VOLUME_PRESSURE_BRIDGE_SPEC`](../specs/PHASE_VOLUME_PRESSURE_BRIDGE_SPEC.md)
- **Validation:** [`PHASE_VOLUME_PRESSURE_BRIDGE_VALIDATION`](../development/PHASE_VOLUME_PRESSURE_BRIDGE_VALIDATION.md)
- **Design baseline:** `d7500e219af6f670be05f830b50c232d2bb53077`
- **Authorization source:** `f1ca48cc01a906bfb4a997c72bc2744b81546ccd`
- **Runtime:** TE-3 / TE-5B / full TE-5 NOT STARTED

## 1. Why this bridge exists

The D-018 phase architecture fixes the current Water/Steam quantity defect by
changing the future Water path from extra Steam yield to one 1:1 identity.
That path cannot activate alone because the G5 product contract still requires
boiling, confinement, gauge pressure, rupture, opening and venting to form one
readable causal chain.

TE-5B is the smallest bridge between those accepted contracts. When initiated
or gas-facing phase work reaches the Water-to-Steam endpoint, it asks whether
the accepted completion wins one local EMPTY movement opportunity or that
opportunity is blocked/contended. The answer selects zero or `100.0` gauge-
pressure consequence before joint settle. A Void-first edge attempt is not
accepted by this bridge and remains ready Water pending full TE-5 edge meaning.
The bridge does not try to solve general gas volume or background pressure.

## 2. Reuse-first audit

| Need | Existing mechanism reused | New runtime state/pass |
|---|---|---:|
| Steam-reachable local geometry | GAS up / parity up-diagonal / parity lateral stencil, restricted to EMPTY | 0 |
| exclusive volume availability | G5-B stateless proposal/claim and `edge_priority` | 0 |
| index/sentinel space | strict `cell_count < 1 << 30` | 0 |
| scratch lifetime | TE-3 proposal/claim window after TE-2 context use | 0 |
| target Air isolation | TE-1 separate Environment receiver transaction | 0 |
| failed consequence | G5-B direct and Environment-blocked pressure separation | 0 |
| pressure transport | existing non-negative scalar gauge-pressure propagation | 0 |
| structure failure | existing generic rupture threshold grammar | 0 |

The bridge adds mode semantics and fixtures only. It adds `0` persistent
buffers, `0` full-world scratch buffers, `0` production passes and `0`
timestamp queries relative to the accepted TE-3 projection. External code or
formulas copied/translated/vendored: `0 files / 0 lines`.

## 3. Options and disposition

### A. Unconditional completion pressure

Rejected. It creates pressure in clearly open boiling and can cross a nearby
Wood threshold before the new Steam has a normal movement opportunity.

### B. Non-exclusive “any EMPTY” relief

Rejected. Two or more completions can count the same EMPTY Cell. It removes the
existing winner/loser consequence grammar and makes same-tick quantity-volume
accounting order-dependent.

### C. Volume fraction, fragments or dedicated volume field

Rejected for TE-5B. It creates a second quantity/ownership system and new
world-scale state before evidence requires it.

### D. Exclusive local volume-relief token

Evaluated primary candidate. One source requests at most one movement-reachable EMPTY
Cell; one target grants one claim across both request modes; the winning relief
target remains unchanged; failed relief becomes gauge pressure exactly once.
Independent review found that this same non-mutation prevents finite headspace
from being consumed across ticks, so option D is not an acceptable final bridge
under the current constraints.

## 4. Candidate in one transaction

```text
eligible endpoint attempt (already initiated Water E>0, or current gas-facing)
-> replay resulting-Steam GAS First-Match order
   up, parity up-diagonals, parity laterals
-> EMPTY: accept provisional Steam + VOLUME_RELIEF(target)
-> earlier legal density swap / no EMPTY: accept provisional Steam + VOLUME_RELIEF(blocked)
-> Void first: defer; retain Water/H + NONE + zero pressure
-> shared Matter/relief claim arbitration
-> winner: settle 1:1 Steam, target unchanged, pressure +0
-> blocked/loser: settle 1:1 Steam, pressure +100 once
```

Downward Cells, density swaps, occupied GAS, Void and long search are excluded
as relief targets. Occupied non-swappable candidates are skipped exactly as
ordinary GAS movement; an earlier legal swap stops as blocked; Void stops as
edge-deferred rather than being skipped to a false lateral target. Atmospheric
and Vacuum Empty both qualify by occupancy. Air pressure is not an input.
Ordinary later GAS movement, not the bridge, occupies headspace.

### Blocking conservation trace

In a sealed one-Cell-wide column `[Stone cap][one EMPTY][ready Water][Water
just below endpoint]...`, only the top Water is ready at `t0`. Its completion
wins the only EMPTY target and receives zero pressure. The next ordinary move
occupies that Cell and vacates its source; during that tick's thermal/phase
sequence the next Water reaches the endpoint and wins the new up-EMPTY Cell.
Staggering readiness prevents a lower same-tick request from stopping on an
earlier Steam density swap. The EMPTY vacancy walks downward; every Water can
complete 1:1 with no blocked/losing request and no pressure. A token target that
is not used can also remain available for later ticks because the bridge
neither reserves nor mutates it.

This is a model counterexample, not missing runtime evidence. Same-tick claim
exclusivity is not a finite-volume ledger. Repair requires new capacity state,
a target/Environment mutation, additional occupied quantity, or another
pressure law, all outside or contrary to the authorized candidate. No alternate
option is silently adopted, and TE-5B stops **DESIGN BLOCKED**.

## 5. Encoding lock candidate

The source audit found no collision, so the candidate uses:

```text
00 = none only
01 = Matter expansion
10 = volume relief
11 = invalid/reserved
low 30 bits = target/source index + 1; zero = blocked for a request mode
```

The maximum legal payload remains within `0x3fffffff`. Invalid/reserved words,
out-of-range payloads and mode mismatches fail closed. Both valid modes share
one deterministic `edge_priority` destination domain.

The pure proof must pass this exact encoding. A failure blocks design; it does
not authorize a silent alternate encoding.

## 6. Consequence ownership

| Mode/result | Matter target | Environment receiver/displacement | Pressure owner |
|---|---|---|---|
| relief winner | none | none | zero |
| relief blocked/loser | none | none | source `+100.0` once |
| generic direct blocked/loser | historical generic behavior | none | generic source rule once |
| generic winner, receiver succeeds | historical spawn | historical receiver/reconcile | zero |
| generic winner, receiver fails | no committed spawn | receiver failure only | generic source rule once in Environment-blocked pass |

Relief is rejected by receiver, spawn and Environment-blocked pressure. A
mixed-mode target still has only one winner. A Matter winner whose receiver
later fails does not reopen claim arbitration for a relief loser.

## 7. TE-3 completion integration

The bridge observes an eligible endpoint attempt before output settles, not a
post-completion Water-name or temperature reconstruction. Already initiated
positive-E/ready Water may enter while buried; canonical buried Water may not.
A finite extreme Ice input can enter only with current gas-facing context. The
valid request word is the explicit acceptance token; provisional Steam and the
mode consequence settle together, preserving exactly-once behavior.

The existing phase descriptor pressure slot carries `100.0` for every source
family identity that can cross that endpoint after attempt eligibility passes.
Its existing trait word also carries registry-derived Steam swap eligibility
for First-Match replay. Phase-family yield remains one; the scalar is read only
when the accepted mode is relief. Subsequent Steam ticks emit `NONE`, so
pressure cannot repeat from identity persistence.

No phase H, E, Lf, Lv, quantity, surface-initiation, buried-ready or
metastability rule is reopened.

## 8. Pass feasibility

The combined projection remains:

- 40 timestamped production passes;
- 80 timestamp queries;
- two 640-byte profiler buffers, 1,280 bytes total;
- 4,722,608 tracked bytes at 256² with profiler;
- 302,018,096 tracked bytes at 2048² with profiler;
- TE-5B persistent/full-world memory delta `0`.

`phase_thermodynamics`, expansion spawn and expansion pressure remain at the
eight-storage ceiling but need only arithmetic/mode guards and reuse already-
bound data. Claim, Environment receiver and Environment-blocked pressure stay
below the ceiling. No extra pass is required.

Fixed writer/settle order:

```text
TE-2 settles
-> phase context fully writes claim
-> phase fully writes proposal
-> expansion claim fully writes claim
-> Matter-only receiver/spawn
-> direct generic/relief pressure
-> Matter-only Environment-blocked pressure
-> identity/phase/Environment/pressure settle
-> existing pressure propagation and settle
-> existing rupture
```

Later combustion and Smoke producers fully overwrite proposal/claim before
their consumers. Activity/wake and profiler grouping remain existing contracts;
their exact future behavior needs structural/GPU evidence.

## 9. Fixture program

The validation contract predeclares all twelve implementation fixtures:

| Fixture | Primary contract |
|---|---|
| F01 | unique open relief; target/Air unchanged; zero pressure |
| F02 | fully blocked completion; one Steam; exact pressure 100 |
| F03 | two sources/one target; one winner; loser pressure 100 |
| F04 | ordinary next-tick GAS movement consumes controlled route |
| F05 | **blocked:** finite headspace must change from early relief to later confinement, but the vacancy-walk trace prevents the current model from guaranteeing it |
| F06 | generic non-family expansion/Environment compatibility |
| F07 | mixed-mode one-winner contention and source-owned consequences |
| F08 | generic receiver failure exactly once; relief isolated |
| F09 | completion-tick-only pressure and exact reset replay |
| F10 | open boiler produces no false phase-volume rupture |
| F11 | **blocked:** full atomic G5 heating-to-vent trace cannot require pressure after finite relief under the current model |
| F12 | full writes, invalid modes, seam, sleep/wake and reset |

F11 must use ordinary phase, movement, gauge pressure and rupture rules. It
cannot use a boiler-specific explosion, combustion-created opening or rebinding
of historical evidence.

## 10. Reference-proof receipt and boundary

The predeclared fixed seed was `0x54453542`; unique randomized trial count was
`100,000`; two deterministic replays occurred inside one process. The script
was executed exactly once and returned `PASS_REFERENCE_MODEL_ONLY` with failed
checks `0` and smallest counterexample `null`.

Script/result SHA-256:

```text
6fd9276933822db850bd4ec3f9648cf64c45b8905f6b37d17cc88d03cb23a340
f53173af05199916b10d287a02c8193e9f86c40c853c019db8491cb86ff56e59
```

The deterministic digest was
`001968b462d75865851e159c35167e6ace04c27c46d12a7f77511823ab378d80`.
There were 169,945 mixed-mode targets, 250,245 relief winners and 411,612
blocked/losing relief requests; the latter exactly matched 411,612 pressure
consequences. Mode/index boundaries, invalid failure, one winner, one request,
unchanged relief target, generic separation, quantity, completion exactly once,
reset and scratch overwrite all passed. The JSON parsed and its embedded script
hash matched the actual script.

The extreme-Ice result was only an identity-labelled pure function call plus a
constant check; it did not generate or inspect a production phase descriptor,
run enthalpy normalization, or exercise ready Water `E = Lv`, `T = 100` after
surface context changes. Those remain future semantic/structural fixtures.
Likewise, abstract mode-mismatch rejection does not mean receiver/spawn
independently compare proposal and claim: the projected graph trusts
`expansion_claim` to reject bad inputs and fully write one constructed valid
claim or zero, while downstream consumers validate only the claim fields they
already bind.

It does not cover WGSL, GPU, actual movement, grid/vacancy conservation,
finite-capacity exhaustion, sleep, propagation, rupture, venting, performance,
appearance or acceptance. The exact command and hashes are recorded in the
validation receipt. The PASS therefore does not clear the finite-headspace
counterexample and the proof was not rerun.

## 11. Atomic G5 activation rule

TE-3 and TE-5B may be implemented only after this High is resolved by a new
user-dispositioned architecture and a later explicit authorization.
They may not become production/user-testable separately. One future source
must pass the new phase fixtures, TE5B F01–F12 and the F11 causal trace, then
activate Water yield 1 and the bridge together while replacing the old Water
extra-yield path.

Until then, the current production Water behavior remains active. Historical
G5 and TE-2 receipts retain their original source identity and limitations.

## 12. Explicitly deferred

- derived Air pressure as a force;
- Atmosphere background pressure and Vacuum pressure;
- structure face differential;
- product world-edge pressure mode;
- Vacuum combustion;
- a general compressible-fluid or volume solver;
- runtime pass/binding/performance evidence;
- TE-4 and G9-B/C/D/E.

## 13. User architecture-review checklist

The blocked candidate must stop for an explicit user architecture revision on:

- choose how finite capacity is consumed and which current no-state,
  non-mutation or 1:1 constraint, if any, may change;
- revise or replace the exclusive-relief-token model;
- approve/revise occupancy-only relief for Atmospheric and Vacuum Empty;
- approve/revise the exact two-mode encoding;
- retain/revise inherited confinement impulse `100.0` in the new atomic
  fixture;
- confirm/revise the finite-headspace F05/F11 product meaning.

Full TE-5 background/structure coupling, edge mode, Vacuum combustion and
runtime authority remain separate even if all five candidate items are
approved.

## 14. Stop boundary

The one-shot abstraction passed, but independent review left the finite-
capacity High open. The actual stop is:

```text
TE-5B: DESIGN BLOCKED
ADR-0007: PROPOSED — DESIGN BLOCKED / USER ARCHITECTURE REVISION REQUIRED
TE-3 runtime: NOT STARTED
TE-5B runtime: NOT STARTED
Full TE-5: NOT STARTED
```
