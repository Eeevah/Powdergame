# TE-3 Direct Review Surface Remediation — 2026-08-21

- **Disposition:** DIRECT OBSERVATION CONSISTENT / TE-3 USER ACCEPTED WITH KNOWN FOLLOW-UP (D-027)
- **Candidate source:** `c2f4f2bb16b00801a72ff6e4a54726cc69674bad`
- **Production-physics source:** `41467219819c5d0cb3eab8ae22b652449da20480` — unchanged
- **Task baseline:** `6a7598a4fe6bcec98a73bf83356accba436cc646`
- **Prior evidence:** [`THERMAL_ENVIRONMENT_TE_3_PHASE_CYCLE_2026-08-21`](THERMAL_ENVIRONMENT_TE_3_PHASE_CYCLE_2026-08-21.md)

## Boundary

Only `apps/windows` candidate staging, fixed diagnostics, HUD integration and
their tests changed. Engine/Core, engine/gpu, production WGSL, phase constants,
Air/thermal coefficients, pressure and the product Inspector did not change.
The final-source FULL and TE3-F01–F15 evidence remain bound to `4146721...`.
This remediation ran no workspace FULL and does not relabel `c2f4f2b...` as a
new production-physics FULL source.

## Scene 2 — surface, buried and reveal

Two Water controls begin at identical `T=100 C`, `E=480`. The surface Cell is
`(72,112)` with one gas-facing EMPTY face; its other movement candidates are
occupied. The buried Cell is `(172,112)` with all eight surrounding cells
occupied at `100 C`, satisfying the required four orthogonal blockers and
preventing diagonal drift. Scene 2 uses uniform `100 C` Atmosphere so this is a
completion-permission comparison rather than a cooling comparison.

At the predeclared tick 24, candidate staging changes only `(172,111)` from
Stone to EMPTY with `100 C` Air. It does not write the buried Cell's identity,
temperature or phase energy.

Observed production-tick checkpoints:

| Tick | Surface | Buried/exposed | Family count |
|---:|---|---|---:|
| 0 | Water, 100, E=480 | Water, 100, E=480 | 2 |
| 1 | Steam, 100, E=480 | Water, 100.000008, E=480 | 2 |
| 24 | opening EMPTY | Steam, 100, E=480 | 2 |

## Scene 3 — lid, free Air and K=0 Boundary

All three controls begin as canonical Steam at `T=94 C`, `E=480`.

- Lid Steam `(60,102)` is motionless and directly adjacent to `20 C` Stone.
- Free-Air lane is nine Steam Cells `(124..132,102)`. Boundary blocks every
  up, up-diagonal and lateral GAS movement target, while each lower face is
  EMPTY Air. TE-2 can cool through those Air faces without special movement.
- Boundary-control Steam `(196,102)` is enclosed by `20 C` K=0 Boundary.

Tick 1 values are lid `T=100/E=474.312012`, free-Air diagnostic
`T=93.444977/E=480`, and Boundary `T=94.000015/E=480`. Free-Air radius-two
nucleation first starts at tick 63: the diagnostic seed is
`T=100/E=455.974670`, and only 2 of 9 lane Cells are partial on that tick. The
K=0 control remains canonical. This demonstrates surface work before free-Air
work and rejects one-tick whole-lane conversion.

## Scene 4 — reversal and no-sink

- Boiling reversal `(60,108)`: Water `T=100/E=240`, enclosed against movement,
  with one `20 C` Stone cooling face.
- Condensation reversal `(128,108)`: Steam `T=100/E=240`, enclosed against
  movement, with one `300 C` Stone heating face.
- No-sink `(196,108)`: Steam `T=60/E=480`, enclosed by eight K=0 Boundary
  Cells. At tick 32 only the upper Boundary becomes `20 C` Stone.

Tick 1 values are boiling `T=100/E=235.200012`, condensation
`T=100/E=242.399994`, and no-sink `T=60/E=480`. No-sink remains byte-stable
through tick 31. At tick 32 the restored Stone face wakes real work and the
Steam reaches `T=100/E=447.520020`. Family count remains 3.

## Fixed diagnostics and controls

Every scene has exactly three named fixed rows. Scenes 2–4 use the labels:

- `Surface Water`, `Buried Water`, `Exposed-after-open result`;
- `Lid Steam`, `Free-Air Steam`, `Boundary-control Steam`;
- `Boiling reversal`, `Condensation reversal`, `No-sink Steam`.

Each row reports Cell, Material, temperature, phase energy, semantic progress,
sample tick and `Fresh`/`Sampling`/`Failed`. A generation/sequence token rejects
late results after reset or scene change. `N` performs one production tick and
forces one new sample; playing stays at the eight-tick cadence; `I` only
expands/collapses fixed rows; `F` retains x1/x4/x16. The persistent summary
keeps scene, play/pause, speed, simulation/sample ticks, family counts and the
pressure-deferred label. The pointer-driven product Inspector remains exactly
24 bytes at no more than 10 Hz.

## Validation and artifact

- Phase-cycle semantic tests: 6 passed, actual `Simulation` ticks.
- Affected Windows binary suite: 170 passed, 1 unrelated long GPU test ignored.
- Candidate I/F routing and explicit phase-cycle route: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p powdergame-windows --all-targets`: passed.
- `cargo clippy -p powdergame-windows --all-targets -- -D warnings`: passed.
- Strict development-policy audit and `git diff --check`: passed.
- Workspace FULL: 0, because no Engine/Core/WGSL/shared state changed and the
  affected app/readback surface is closed by the package and semantic tests.
- Release build: 1.
- TE-3 candidate bounded launch check: 1.

The only launch command was:

```text
run_powdergame.bat phase-cycle --smoke-frames 60
```

It built and started the canonical application on RTX 5090 / DX12, loaded the
TE-3 candidate, produced three diagnostic rows and exited cleanly after 60
frames. Its 14 simulation ticks do not prove a phase cycle.

- EXE: `target/release/powdergame-windows.exe`
- SHA-256: `F15B8B1198443935CB233A0FA526256563F400A0775ECC246542BB195938F966`
- Size: 10,095,104 bytes

Pressure redesign remains deferred/not started. TE-4 and G9-B/C/D/E remain
not started. At this receipt boundary direct user review was still required;
D-025, D-026 and D-027 later close that review without rebinding this evidence.

## 2026-08-22 partial direct review and observability follow-up

D-025 records Scenes 2–4 as direct-observation consistent while Scene 1 stays
pending. App source `89d2400d677dec7e39cba76234c18d8b2363a496`
supersedes this receipt only for current candidate presentation: it adds the
Sandbox phase-energy Inspector profile, sealed-world disclosure, truthful
probe labels and a finite cold-lid reservoir. Production physics remains
`4146721...`. Current receipt:
[`TE3_SANDBOX_PHASE_TRUTH_REMEDIATION_2026-08-22`](TE3_SANDBOX_PHASE_TRUTH_REMEDIATION_2026-08-22.md).

## Final direct disposition

D-025 records Scenes 2–4 as direct-observation consistent. After the separate
Scene 1 remediation and review, D-027 records the whole TE-3 candidate as
**USER ACCEPTED WITH KNOWN FOLLOW-UP**. This later disposition does not rewrite
the source-bound tests, artifact or launch limitations above. Pressure redesign
remains deferred/not started.
