# Checkpoint — TE-4D ignition-kinetics design authorized — 2026-08-22

## Repository coordinate
- Worktree: `C:\Users\mdkap\source\repos\Powdergame-g8b`
- Branch: `feature/m0-g9-first-playable`
- Session baseline: `1b0fb2c0328eba6a9cbeb824b30727ecc46675bd`
- Production physics: `41467219819c5d0cb3eab8ae22b652449da20480`
- Scene 1 app source: `e9f4a3744ea3bdab0fd70f0f78aa27cb7e9fa448`
- Working tree: D-028 docs/memory authorization closure in progress; runtime unchanged

## The story so far
The user directly accepted the complete pressure-decoupled TE-3 Water/Steam
phase cycle after Scenes 1–4 were observed consistent. D-027 records **USER
ACCEPTED WITH KNOWN FOLLOW-UP**. One phase-family Cell remains one Water-
equivalent quantity, Water creates no extra Steam and Water boiling creates no
phase pressure. The pressure-volume redesign remains deferred.

## Valid evidence
- `docs/evidence/THERMAL_ENVIRONMENT_TE_3_PHASE_CYCLE_2026-08-21.md` — TE3-F01–F15 and final-source FULL; valid only for production source `41467219819c5d0cb3eab8ae22b652449da20480` and its recorded toolchain/profile/configuration.
- `docs/evidence/TE3_DIRECT_REVIEW_SURFACE_REMEDIATION_2026-08-21.md` — Scenes 2–4 candidate staging and diagnostics at source `c2f4f2bb16b00801a72ff6e4a54726cc69674bad`.
- `docs/evidence/TE3_SANDBOX_PHASE_TRUTH_REMEDIATION_2026-08-22.md` — 24-byte Sandbox phase profile and observability at source `89d2400d677dec7e39cba76234c18d8b2363a496`.
- `docs/evidence/TE3_SCENE1_PHASE_CYCLE_REMEDIATION_2026-08-22.md` — actual Scene 1 ticks `72/320/320/336/552/584`, family `288`, source `e9f4a37...`, and direct user observation.
- Canonical reviewed artifact SHA-256 `6F2EF0BF49FC39AF550B2CF958DCC5A2F551AAE65ACD9F1735D208519E8E1C0E`, size `10,097,664` bytes.
- D-027 is a docs/memory-only human disposition and does not rebind or rerun any runtime evidence.

## Decided
- D-024 — pressure-decoupled one-Cell/one-quantity TE-3 is the active runtime model.
- D-025/D-026 — Scenes 1–4 direct observation consistent; Q-015 closed.
- D-027 — TE-3 **USER ACCEPTED WITH KNOWN FOLLOW-UP**.
- `WATER_STEAM_PRESSURE_VOLUME_REDESIGN` remains deferred/not started.

## Waiting on the user
None during the authorized TE-4D design program. Architecture acceptance,
Vacuum policy, implementation, TE-5 and G9-B each require a later user decision.

## Current authorization
- D-028 authorizes TE-4D docs/reference design only.
- Target: integrated excess-temperature dose, cooling decay, previous-snapshot
  orthogonal flame bonus and finite energy-like chemical heat.
- Required evidence: one preflighted/frozen reference execution and one
  fresh-context independent review.
- ADR-0012 must remain Proposed; TE-4 runtime, Vacuum policy, Oxygen, Ash, FX,
  Pressure redesign, TE-5/TE-6 and G9-B/C/D/E remain unauthorized/not started.

## Next first action
Complete the exact descriptor/flags/pass/writer inventory, predeclare and freeze
the reference model, execute it exactly once, then run the independent review.

## Tried
- Reused the source-bound runtime, artifact and direct-review receipts; no Cargo, GPU, FULL, build, launch or candidate rerun was needed for this docs-only acceptance.
- Preserved all blocked TE-5B/C/D/X and phase-packet attempts as history rather than treating TE-3 acceptance as their repair or revival.
