# TE-4I Scene 4 Smoke observability remediation — 2026-08-23

## Coordinate and disposition

- Start/docs baseline: `4ae570d104268f267ce59cc2c1f8816c803721bb`
- Preserved production-physics source: `8d9e8cbe3b6ac651335b5a728ef491abeae4772a`
- Candidate observability source: `a7622bf2106a11e731a46018a4afe30d236b9304`
- Wiki authority: `bf3d2b1d585492f470b92f09342935a497093379`
- Disposition: **candidate observability failure; production physics unchanged**
- TE-4I: **REVISED IMPLEMENTATION CANDIDATE / SCENE 4 USER RE-REVIEW PENDING**
- ADR-0012: **PROPOSED**

The user's direct observation was valid: the original Scene 4 surface showed
the source changing fuel `0 -> 1`, Air `YES -> NO`, and burning `YES -> NO`,
but did not let the user see or directly identify the claimed Smoke Matter.

## Exact production-state diagnosis

The candidate geometry was reproduced without alteration:

| Role | Cell | Tick 0 |
|---|---:|---|
| source | `(208,110)` | Wood, burning, fuel `0`, sole positive-Air face |
| Smoke target | `(209,110)` | EMPTY, Air mass `1.0` |
| Environment receiver | `(209,111)` | EMPTY/Vacuum |

The exact candidate test now reads authoritative target Matter rather than
inferring creation from source Air loss.

- Tick 1: source fuel is `1`; target Current and Next are both exactly Smoke;
  total Smoke count is exactly `1`; target Air Current/Next are exact Vacuum;
  receiver Air Current/Next settle equally with the whole former target Air
  mass `1.0`. Receiver energy remains finite/positive after the preceding
  production TE-2 thermal work.
- Tick 2: source burning is false; fuel remains `1`; gross chemical Heat and
  new Flame/Smoke emission are zero; the existing Smoke count remains `1`.
- Bounded later horizon: authoritative Smoke decays to EMPTY, the marker
  disappears, source Air access recovers, and no additional Smoke is emitted.

No claim, receiver, commit, or settle blocker was found. Existing production
F08 independently reproduced the same target identity and receiver settle.
Core, GPU production WGSL, coefficients, and ignition predicates were not
modified.

The prior candidate test passed without proving Smoke because it sampled only
the source row and treated Air-access loss plus fuel progress as a proxy for
the target transaction. The one-cell target was also under the fixed right
diagnostic card in the fitted camera view.

## Candidate remediation

Scene 4 retains its four policy controls and adds two fixed authoritative rows:

- `Self-Smoke target`: `(209,110)`, Material, temperature, Smoke decay age,
  Air mass/energy, sample tick and freshness;
- `Air receiver`: `(209,111)`, Material, Air mass/energy, sample tick and
  freshness.

The persistent summary adds Smoke Matter count, source fuel progress, target
Material, receiver Air mass, and the causal labels `READY`, `EMITTED`,
`EXTINGUISHED`, and `DECAYED`. Scene 4 uses a candidate-only camera focus that
keeps source, target, and receiver to the left of the diagnostic panel. Three
thin pulsing outlines surround the actual target cell only while its sampled
authoritative Material is Smoke. The outline does not create, enlarge,
duplicate, recolor, or animate production Smoke and disappears when the
target is no longer Smoke.

The normal product Inspector remains unchanged at one 24-byte bounded batch
and at most 10 Hz. Reset and scene changes still invalidate candidate samples.
The existing launcher route and verified Desktop shortcut target/arguments are
unchanged.

## Validation receipt

Final candidate code passed:

- Windows package: `183 passed / 1 ignored`;
- exact Scene 4 production-state test, including later decay/Air recovery;
- Scene 4 camera/diagnostic-panel visibility test;
- N/F/I, reset, generation-safety, and normal Inspector regressions;
- production `te4i_f08_actual_sole_air_self_smoke_extinguishes_next_tick`;
- production Smoke receiver success and no-receiver rejection controls;
- Environment transport/current-next settle controls;
- affected all-target check and clippy with warnings denied;
- formatting, strict policy audit, and `git diff --check`.

Workspace FULL count is `0` because Engine/Core/production WGSL did not
change. Release build count is `1`. The canonical artifact is:

- path: `target/release/powdergame-windows.exe`
- size: `10,147,328` bytes
- SHA-256:
  `EABC00C3F803800EEFA9DD935DD15F3AEDEB507EEBD638364088E2BD324D6297`

One 60-frame bounded candidate launch completed and exited cleanly through
`run_powdergame.bat ignition-kinetics --smoke-frames 60`. It opened the
candidate's default Scene 1, so it proves artifact startup/exit only and is not
claimed as a Scene 4 visual proof. Scene 4 evidence comes from the exact actual
production-state test above; visual sufficiency remains for the user's direct
re-review.

## Direct Scene 4 re-review

1. Open the unchanged Desktop shortcut or run
   `run_powdergame.bat ignition-kinetics`.
2. Press `4`, then press `N` once.
3. Confirm `EMITTED`, target Material `Smoke`, Smoke count `1`, receiver Air
   mass `1.0`, and the outline on the real target cell.
4. Press `N` again and confirm `EXTINGUISHED`, fuel still `1`, and no second
   Smoke.
5. Confirm the outline follows only the authoritative Smoke target and is not
   shown after that target returns to EMPTY.

This receipt does not accept ADR-0012 or claim TE-4I user acceptance.

`LESSON_PROMOTION: REQUIRED` was evaluated. PG-L036 records the project guard;
the verified Wiki evidence/fixture-integrity contract already requires direct
source, target, receiver and authoritative-metric evidence, so no new Wiki
content or dirty-checkout mutation was required.
