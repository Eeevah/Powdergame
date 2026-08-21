# Checkpoint — TE-3 Scene 1 finite-cycle remediation ready — 2026-08-22 04:12 KST

## Repository coordinate
- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Runtime/app HEAD: `e9f4a3744ea3bdab0fd70f0f78aa27cb7e9fa448`
- Working tree: expected docs/memory closure files only
- Wiki remote fallback: `b8c22c1dc477f7d08f35b54e11ca95c6ad10d4c3`; local Wiki is user-dirty and untouched

## The story so far
The user's Scene 1 run reached tick 21,160 with no Steam because the 3,885-Cell
fill diluted a finite heater and equilibrated below boiling. App source
`e9f4a37...` replaces only that fixture with a 288-Cell finite one-shot beaker.
Production physics remains `4146721...`; Scenes 2–4, Sandbox Inspector, the
Desktop shortcut, and deferred pressure remain unchanged.

## Valid evidence
- `docs/evidence/THERMAL_ENVIRONMENT_TE_3_PHASE_CYCLE_2026-08-21.md` — production F01–F15/FULL receipt; valid only for `41467219819c5d0cb3eab8ae22b652449da20480`.
- `docs/evidence/TE3_SANDBOX_PHASE_TRUTH_REMEDIATION_2026-08-22.md` — app/profile receipt for source `89d2400...`.
- `docs/evidence/TE3_SCENE1_PHASE_CYCLE_REMEDIATION_2026-08-22.md` — valid for Scene 1 source `e9f4a37...`, exact fixture, RTX 5090/DX12, and recorded artifact identity.
- Actual Scene 1 ticks: boil `72`, Steam/rise `320`, condensation `336`, upper Water `552`, fall `584`; family `288` exact.
- Windows suite `174 passed / 1 unrelated ignored`; FULL `0`.
- EXE SHA-256 `6F2EF0BF49FC39AF550B2CF958DCC5A2F551AAE65ACD9F1735D208519E8E1C0E`, 10,097,664 bytes.

## Decided
- D-024 — pressure-decoupled one-Cell/one-quantity TE-3 remains active.
- D-025 — Scenes 2–4 direct observation consistent; Scene 1 pending; Sandbox phase truth implemented.
- Scene 1 is remediated but not user accepted; Q-015 remains open.

## Waiting on the user
Scene 1 direct re-review and the resulting accept/revise decision.

## Next first action
Open the Desktop shortcut, press `SPACE`, and confirm the visible one-shot sequence by about tick 584; press `R` to replay.

## Tried
- The original one-Cell cold lid began condensation but exhausted its finite heat capacity before completing within 900 ticks.
- A finite isolated 63-Cell Stone block behind the same direct lid face completed Water at tick 656 without changing phase rules or creating an infinite sink.
- Free-Air completion required a longer bounded horizon: partial at tick 63, Water at tick 1043.
- The original Scene 1 never boiled: at tick 21,160 it had Water 3,885, Steam 0, and surface 85.824 C / E=0.
