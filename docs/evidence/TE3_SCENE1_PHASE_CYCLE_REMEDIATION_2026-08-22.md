# TE-3 Scene 1 Phase-Cycle Remediation — 2026-08-22

- **Disposition:** REVISED SCENE 1 CANDIDATE / USER RE-REVIEW PENDING
- **App source:** `e9f4a3744ea3bdab0fd70f0f78aa27cb7e9fa448`
- **Task baseline:** `a40caee00eec3d2357b6096dcf9c21dc76dc5cd7`
- **Production physics:** `41467219819c5d0cb3eab8ae22b652449da20480` — unchanged
- **Wiki fallback:** connected `origin/main` `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3`; the local checkout was user-dirty and was not modified

## Reported failure and cause

Direct review reached tick `21,160` with family `3,885`, Water `3,885`, Steam
`0`, and the surface probe at `85.824 C / E=0`. The old 3,885-Cell Water fill
shared one finite 64-Cell `800 C` Stone reservoir with the cold vessel and Air.
It equilibrated below boiling; waiting longer could not create Steam. This was
a candidate-fixture defect, not evidence for a production phase-rule change.

## Replacement fixture

Scene 1 now uses one shallow, finite-energy beaker:

- Water: `x=104..151`, `y=128..133`, `96 C`, 288 Cells;
- finite Stone heater/floor: `x=103..152`, `y=134`, `800 C`;
- cold Stone lid: `x=103..152`, `y=92`, `-20 C`;
- Stone sides: `x=103/152`, `y=92..134`;
- fixed probes: beaker surface `(128,128)`, rising route `(128,108)`, and the
  Cell below the cold lid `(128,93)`.

The HUD calls this a finite one-shot cycle and tells the reviewer to press `R`
to replay it. No identity is scripted. Production thermal exchange, phase
normalization, GAS movement, condensation, and Liquid movement produce every
observed change.

## Actual Simulation checkpoints

The candidate test sampled the authoritative grid every eight ticks for a
predeclared 3,600-tick bound:

| Event | First sampled tick |
|---|---:|
| Surface Water has positive boiling phase energy | 72 |
| First 1:1 Steam identity | 320 |
| Steam above the original Water surface | 320 |
| Risen Steam has partial condensation energy | 336 |
| Condensed Water appears in the upper chamber | 552 |
| Condensed Water reaches the two rows above the vessel | 584 |

Phase-family quantity remained exactly `288` at every sampled checkpoint.
The test fails if any event is absent, out of order, or changes family count.

## Validation

- `phase_cycle::tests`: `8 passed / 0 failed`;
- affected Windows suite: `174 passed / 0 failed / 1 unrelated ignored`;
- affected all-target check: PASS;
- affected clippy with warnings denied: PASS;
- formatting, strict policy audit, and `git diff --check`: PASS;
- workspace FULL: `0`;
- Engine/Core/production WGSL changes: `0`;
- external copied/translated/vendored implementation: `0 files / 0 lines`.

The canonical release binary compiled once. The launcher used for the single
bounded check performed an additional no-op release freshness check before
starting the same binary. The 60-frame check initialized RTX 5090/DX12, loaded
Scene 1, ran 14 ticks, and exited cleanly. It does not prove the phase cycle.

## Artifact and review boundary

- EXE: `target/release/powdergame-windows.exe`
- SHA-256: `6F2EF0BF49FC39AF550B2CF958DCC5A2F551AAE65ACD9F1735D208519E8E1C0E`
- Size: `10,097,664` bytes
- Shortcut: `C:\Users\mdkap\Desktop\Powdergame TE-3 Phase Cycle.lnk`
- Shortcut target/arguments: canonical EXE / `--phase-cycle-candidate`

Scene 1 is not user accepted by this automated result. Direct review must
confirm the visible sequence, stable family count, replay wording, and
deferred-pressure label. TE-5 Pressure redesign, TE-4, and G9-B/C/D/E remain
not started.
