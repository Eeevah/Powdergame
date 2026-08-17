# G8-B Scenario 3 — Fire / Heat Harness Candidate

Date: 2026-08-17
Status: **SEALED CANDIDATE — AUTOMATIC PASS / USER ACCEPTANCE PENDING**
Branch: `feature/m0-g8b-scenario-suite`
Starting SHA: `0f5585ba34ec901224a82f4329624abcb66b796b`
Candidate source SHA: `1635fdb9f562192123c92846e137b125c684ede9`

## 1. Scope and frozen predecessors

This work extends the existing one-command Experiment Evidence Harness to
`run_experiment.bat fire-heat`. It reuses the Sand/Water coordinator,
build-bound provenance, external unique Run directory, create-new writes,
renderer-output frames, reports, packet, hashes, and receipt-last publication.

- Sand Fall remains user accepted and its immutable pilot is unchanged.
- Water Flow remains human `ACCEPTED WITH KNOWN FOLLOW-UP`; its automatic
  `NEEDS_HUMAN_REVIEW`, candidate Run ID, packet, receipt, fixture, physics,
  and artifacts are unchanged.
- Fire / Heat staging and production physics are not tuned before the first
  candidate.
- Pressure Burst, Heavy Mixed World, G8-C, new Material work, physics changes,
  build optimization, `main`, and PR work are outside this task.

The Harness is an acceptance-evidence path. Its rendering, screenshots,
readback, and diagnostic samples are not part of any G8-C timed benchmark.

## 2. Audited finite tick-0 fixture

The shared `powdergame-scenarios` fixture is staged through the same
`reset_and_stage_scenario(..., ScenarioId::FireHeat)` call used by the Windows
Gallery and headless benchmark. The authored 256×256×64 image is finite and
contains no fuel or heat source that is replenished by scenario code.

| Authored region | Half-open rectangle | Initial state |
|---|---|---|
| Stone floor | `[12,244) × [222,232)` | Stone |
| Wood slab | `[24,222) × [154,214)` | Wood |
| Left Oil pocket | `[32,78) × [205,222)` | Oil |
| Right Oil pocket | `[180,226) × [204,222)` | Oil |
| Hot Stone column | `[14,26) × [144,222)` | Stone, 260 °C |
| Hot seed | `[24,42) × [168,202)` | 500 °C, authored `COMBUSTING` |
| Oil seed | `[32,48) × [205,222)` | 180 °C, authored `COMBUSTING` |
| Ice block | `[88,168) × [90,118)` | Ice, -20 °C |
| Water block | `[96,160) × [120,144)` | Water, -20 °C |

Exact tick-0 Material counts are Empty 44,948; Boundary 1,020; Stone 3,256;
Water 1,536; Oil 1,610; Ice 2,240; Wood 10,926; Sand/Steam/Smoke 0. Total
non-empty Matter is 20,588 and finite Wood+Oil fuel is 12,536.

Temperature counts are `-20=3,776`, `0=60,008`, `180=272`, `260=868`, and
`500=612`. Pressure is reference-zero everywhere. Authored combustion flags
cover 884 cells: Wood 544, Oil 272, and Stone 68. The 68 Stone cells are the
geometric overlap between the hot seed and pre-existing Stone column. They are
authenticated input, not evidence of real combustion; production clears
non-combustible combustion bits. Therefore tick-0 flags alone must never be
reported as first combustion.

## 3. Expected production causal chain

The worker advances only `Simulation::tick()` and stages no scripted result.
The production order is movement, thermal conduction, phase transition,
decay, combustion, Smoke commit, then pressure work.

The candidate observes, rather than assumes, this chain:

1. post-tick Wood and Oil expose genuine flame/fuel-progress signals;
2. finite Wood/Oil fuel is consumed;
3. Smoke is generated and later participates in reaction/decay work;
4. temperature changes propagate outside the authored hot mask;
5. the global `(Ice, Water, Steam)` inventory departs from tick 0;
6. reaction activity reaches a confirmed zero streak;
7. a separate post-reaction thermal tail is observed and checked for decrease;
8. programmatic reset reproduces the full pristine state.

The initial Ice and Water are exactly -20 °C while phase thresholds are strict,
and Empty gaps do not conduct. Phase work therefore depends on later
production interactions and is evidence to collect, not a staging guarantee.
`ACTIVITY_REACTION` also includes Smoke decay, so reaction-zero can occur later
than the last visible flame. `ACTIVITY_THERMAL` is reported as thermal activity,
not as an energy measurement.

## 4. Lifecycle and keyframes

The fixed candidate contract uses max tick 20,000, an eight-tick diagnostic
cadence, three consecutive reaction-zero diagnostic samples, and 180 contiguous
post-reaction production ticks. Whole-world all-sleep is not required.

Candidate keyframe roles are tick 0, tick 1, first genuine combustion, first
Smoke, sampled peak reaction, sampled peak thermal, first phase-inventory
change, 25%-combined-fuel-consumed, first reaction-zero, post-reaction thermal
tail, terminal observation, and exact reset. When multiple roles share one
sample, aliases remain explicit. A missing milestone is never relabelled;
honest diagnostic observations fill only the minimum evidence count.

The report distinguishes simulation tick from diagnostic sample sequence.
First/peak values are first-observed or sampled values at the declared cadence,
not claims about unobserved ticks between readbacks.

## 5. Telemetry and automatic verdict

Every sample binds source SHA, clean state, release profile, binary SHA-256,
Run ID, WorldConfig, sleep settings, simulation tick, diagnostic sequence,
activity census, chunk census, material inventory, invalid IDs, non-finite
fields, wake/change diagnostics, and state hashes. Fire-specific fields include
Wood/Oil/Smoke/Ice/Water/Steam counts, Wood/Oil combustion and fuel-progress
signals, propagated-heat cells, phase-inventory change, Smoke/reaction/thermal
peaks, fuel deltas, reaction-zero identities, and post-reaction tail values.

The exact automatic predicates are:

- `combustion_observed`
- `smoke_generated`
- `heat_propagated`
- `phase_work_observed`
- `fuel_consumed`
- `reaction_terminated_before_max`
- `post_reaction_no_restart`
- `thermal_tail_observed`
- `thermal_tail_decreased`
- `no_invalid_materials`
- `no_nonfinite_fields`
- `exact_reset`

Permanent reaction, absent fuel consumption, absent Smoke, absent phase work,
invalid Material, non-finite fields, or reset mismatch are concrete automatic
finding candidates. The two thermal-tail predicates can remain unknown and
produce `NEEDS_HUMAN_REVIEW`; a residual thermal tail is not itself a failure.
Ambiguous visual evidence also remains for human review. An automatic result
does not accept Scenario 3 or close G8-B.

## 6. Source seal, artifact, and publication contract

Artifacts are written outside Git under
`C:\Users\mdkap\source\Powdergame-artifacts\<unique-run-id>`. Run IDs are never
reused, existing files are never overwritten, failed runs remain without a
receipt, and `EXPERIMENT_RECEIPT.json` is the final publication marker and final
filesystem write inside the Run directory. The packet includes logs, telemetry,
report, full PNGs, crops, and Contact Sheet. Worker raw RGBA and analysis remain
in the complete Run directory and hash inventory even when excluded from the
curated packet.
No generated artifact is committed.

Before the build, the coordinator records a source-input manifest binding the
clean named branch/HEAD, tracked Cargo, Rust, WGSL, `build.rs`, scenario and
runner inputs, plus the absolute `C:\Windows\Fonts\consola.ttf` build input by
path, size, and SHA-256. It recomputes the same manifest after build,
immediately before worker launch, and after worker exit. The release output is
copied with
create-new + flush/fsync into the unique Run directory, hashed there, and that
frozen copy—not the mutable `target/release` path—is executed. The worker hashes
its own `current_exe()` before creating a window/GPU and rejects a mismatch.
Any source or binary drift leaves the Run incomplete without a receipt.

`REVIEW_PACKET.zip` is a lightweight human-review packet. It does not claim to
provide complete source/binary forensic verification. Candidate mode therefore
creates sibling delivery artifacts only after the immutable run receipt exists:
`<Run ID>.AUDIT_BUNDLE.zip` and `<Run ID>.AUDIT_BUNDLE_SHA256.txt`. The bundle
contains the Review Packet, manifest, hashes, receipt, source-input manifest,
frozen executed binary, and a commit-addressed Git archive when available.
Sibling creation does not modify the receipted Run directory; scratch mode does
not require an Audit Bundle.

## 7. Validation and result

| Item | State |
|---|---|
| Fixture audit and pure geometry/field pin | PASS — 1 test, 7 filtered |
| Bounded production combustion/Smoke/reset GPU test | PASS — 64 production ticks, 1 test, 2 filtered |
| Rust Fire worker and CLI | IMPLEMENTED; Fire 4/4 and worker CLI 4/4 PASS |
| Python coordinator/independent recomputation | PASS — final Experiment suite 41/41 |
| Targeted fmt/check/tests | PASS — workspace fmt/check; Windows 59 passed / 1 explicit long-run ignored; strict launcher misuse exits 2 before build |
| Workspace clippy | PASS — all targets with `-D warnings` |
| Clean source seal and push | PASS — `1635fdb9f562192123c92846e137b125c684ede9`, upstream 0/0 |
| One post-seal workspace test checkpoint | PASS — 559.745 s; 3 explicit manual/long-run ignores |
| One final-SHA Gallery release smoke | PASS — 60 frames, RTX 5090 / DX12, 5.818 s |
| One Fire candidate | COMPLETE — 20.788 s, no rerun |
| Automatic verdict / Run ID / packet / receipt | `PASS`; identities below |
| User acceptance | PENDING |

## 8. Sealed candidate result

The single candidate run is
`g8b-fire-heat-v0-20260817T133938546075Z-0e6aa901`. Its automatic verdict is
`PASS`; this is not Scenario 3 user acceptance and does not close G8-B.

- genuine Wood and Oil combustion: first observed tick `1`;
- Smoke: first tick `1`, sampled peak `12,070` cells at tick `7,864`, final `0`;
- phase inventory work: first observed tick `712`;
- sampled Reaction peak: `12,997` cells at tick `7,808`;
- sampled Thermal peak: `22,577` cells at tick `6,696`;
- finite fuel: Wood `10,926 → 0`, Oil `1,610 → 475`, total consumed `12,061`;
- Reaction zero: first sample tick `11,448`, confirmed tick `11,464`;
- post-Reaction window: 180 ticks to `11,644`, restart samples `0`;
- Thermal tail: `9,783` at start, `9,773` final, sampled minimum `9,768`;
- invalid material and non-finite field occurrences: `0 / 0`;
- programmatic reset exact equivalence: `true`.

Artifact identities:

- Review Packet: `report/REVIEW_PACKET.zip`, SHA-256
  `2a8e99d14bf0647b71e7ef32e3840655117e93b9f20ad1360af97d62a69eb940`;
- Receipt: `EXPERIMENT_RECEIPT.json`, SHA-256
  `ed17e75f7515d155f8b6e5a41a0aeb751b2876ec573658a6e49eb6dd72108aff`;
- sibling Audit Bundle: SHA-256
  `1c1df01dfa9004b9273bc45e4b01d3c784d5c377f98a9417bc0b7594c6a83706`;
- frozen executed binary: SHA-256
  `0338dfedbfd226f041cfda1b3ee4a81131ba27a2d5b8035abd4e653b552edbb9`;
- source-input manifest: SHA-256
  `d962ee6218eee83a64176d586789f89390e0913b8f9e43222bb46dcdcc73bb52`.

Independent read-only verification found no inventory, digest, source/binary,
telemetry recomputation, PNG/crop/contact-sheet, Review Packet, Audit Bundle, or
receipt-last mismatch. The Run contains 54 files / 81,996,443 bytes; including
the sibling bundle and sidecar, delivery size is 86,967,938 bytes. Pressure
Burst, Heavy Mixed World, and G8-C remain stopped. The unresolved decision is
human review and acceptance of the Fire / Heat scenes.
