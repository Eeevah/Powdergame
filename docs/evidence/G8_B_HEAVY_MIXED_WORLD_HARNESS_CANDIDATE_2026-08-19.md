# G8-B Heavy Mixed World Harness Candidate

Date: 2026-08-19
Branch: `feature/m0-g8b-scenario-suite`
Candidate source SHA: `07260fffab22e5b4513eb168f0baac36e374ab94`
Run ID: `g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4`
Status: **IMMUTABLE CANDIDATE / AUTOMATIC NEEDS_HUMAN_REVIEW / 14-OF-14 HARD PASS / CANDIDATE BLOCKER FALSE / USER ACCEPTANCE PENDING / G8-B NOT CLOSED / G8-C FORBIDDEN**

## Scope and closure boundary

This record closes only the implementation and machine-publication evidence for the fifth official G8-B fixture, Heavy Mixed World. It does not record user acceptance. Automatic telemetry, the candidate's publication contract, and a third read-only verification all pass, but the automatic verdict remains `NEEDS_HUMAN_REVIEW` solely because of the review-only `broad_terminal_tail` flag.

Cell Inspector v0 was already **USER ACCEPTED WITH KNOWN FOLLOW-UP** at tested source `3c342d25099683df53e303d1920cebe1f6578b74`. Its slight bounded new-Cell hover delay at no more than 10 Hz / 100 ms remains non-blocking. That acceptance allowed Heavy Mixed inspection but did not pre-approve Heavy Mixed, G8-B closure, or G8-C.

This docs-only closure does not modify or rerun the source, production physics, fixture, worker, candidate, Receipt, Review Packet, Audit Bundle, screenshots, telemetry, or any earlier Sand/Water/Fire/Pressure evidence. Heavy Mixed remains **USER ACCEPTANCE PENDING**. Therefore G8-B is **NOT CLOSED** and G8-C is **FORBIDDEN**.

## Immutable candidate identity

| Field | Exact value |
|---|---|
| Experiment ID | `g8b-heavy-mixed-v0` |
| Scenario slug | `heavy-mixed` |
| Display name | Heavy Mixed World |
| Run mode | `candidate` |
| Run ID | `g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4` |
| Created UTC | `2026-08-18T15:40:07.111119Z` |
| Completed UTC | `2026-08-18T15:40:37.601105Z` |
| Source branch | `feature/m0-g8b-scenario-suite` |
| Source SHA | `07260fffab22e5b4513eb168f0baac36e374ab94` |
| Source state | `clean` |
| Source commit subject | `feat: add Heavy Mixed experiment analysis` |
| World | `256 x 256`, chunk size `64` |
| Required max tick / cadence | `20,000 / 8` |
| Terminal trend window | `64` production samples |
| Meaningful overlap minimum | `3` sampled records with at least three active subsystems |

During the third read-only publication audit, local HEAD, local `origin/feature/m0-g8b-scenario-suite`, and the live GitHub branch were all exactly `07260fffab22e5b4513eb168f0baac36e374ab94`. The worktree was clean and local ahead/behind was `0 / 0`. Live remote equality was checked with a direct read-only remote query rather than inferred from the local remote-tracking ref alone.

## Schemas and command identity

| Artifact | Schema |
|---|---|
| Manifest | `powdergame-heavy-mixed-manifest-v0` |
| Telemetry and events | `powdergame-heavy-mixed-telemetry-v0` |
| Analysis | `powdergame-heavy-mixed-analysis-v0` |
| Frames | `powdergame-heavy-mixed-frames-v0` |
| Report | `powdergame-heavy-mixed-report-v0` |
| Receipt | `powdergame-heavy-mixed-receipt-v0` |
| Audit Bundle manifest | `powdergame-heavy-mixed-audit-bundle-manifest-v0` |

The worker command uses only the common Heavy lifecycle arguments: frozen executable, `--experiment-worker heavy-mixed`, run directory, run ID, binary SHA-256, `--max-ticks 20000`, and `--diagnostic-interval 8`. Legacy Sand/Water/Fire/Pressure lifecycle arguments are absent.

## Source and executable provenance

The source-input seal was recaptured during the third read-only audit and matched the published `SOURCE_INPUT_MANIFEST.json` exactly.

| Source-input fact | Exact value |
|---|---:|
| Tracked source-input files | `81` |
| Tracked source-input bytes | `2,445,689` |
| External build inputs | `1` |
| External build-input bytes | `453,088` |
| External input | `C:\Windows\Fonts\consola.ttf` |
| External input SHA-256 | `cf00b507b3286870cc5064ebd0633c303f70b491a4af25eec2d32df413db0179` |

Canonical and frozen executable paths:

- canonical: `C:\Users\mdkap\source\repos\Powdergame-g8b\target\release\powdergame-windows.exe`
- frozen: `C:\Users\mdkap\source\Powdergame-artifacts\g8b-heavy-mixed-v0-20260818T154006091598Z-22d9edc4\frozen-binary\powdergame-windows.exe`
- Receipt-relative frozen path: `frozen-binary/powdergame-windows.exe`
- executable bytes: `9,654,272` (`9.207 MiB`)
- executable SHA-256: `9b84db005942cf60ae9ef133521e9297413d49c93d72e7ae64133e29622f7583`

The canonical target executable, Run-local frozen executable, manifest/Receipt identity, and Audit Bundle member are byte-identical. The release build completed successfully; the worker reported RTX 5090 / DX12 hardware check PASS, zero-byte worker stderr, and clean exit.

## Publication hashes and exact sizes

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| Run directory | `98,299,191` (`93.745 MiB`), `60` files | hash inventory below |
| Frozen executable | `9,654,272` (`9.207 MiB`) | `9b84db005942cf60ae9ef133521e9297413d49c93d72e7ae64133e29622f7583` |
| `EXPERIMENT_MANIFEST.toml` | `1,765` | `50bd48c57e9175b3d6cfc53ccf3c787aa7e5246cfaa118468b3e4590a275eaa2` |
| `SOURCE_INPUT_MANIFEST.json` | `14,498` | `d4cf97dba93a3bf108e6105c623bcfb506baeb4306be327143f993faeec28ff3` |
| `report/REPORT.json` | `24,621` | `854318da721d82720ddea1534f1a2b69fa538b7ed002fdffc138ef0ec32dd1ac` |
| `report/REVIEW_PACKET.zip` | `1,154,021` (`1.101 MiB`) | `ca2fe05a1497f8417dd732de23c1a260569adfcccd6bd0f16bad180f2a8d1144` |
| `HASHES.sha256` | `6,981` | `a382f89a1cd2a557f8384785611f0a8bbe20aca5d69ef1fc7384570778b7f822` |
| `EXPERIMENT_RECEIPT.json` | `3,547` | `2abebdef7f9174e63abfd9c67ce4a48d24b48edde4e6c29fab49022e36a2dbd1` |
| Sibling Audit Bundle | `6,659,537` (`6.351 MiB`) | `bc44c66bd52b5d856decb2317389a455a56ac8ae1f8d67b1bfeb5446cfb5731b` |
| Audit Bundle sidecar | `134` | contains the exact Bundle hash and filename |

Run-directory byte breakdown:

| Top-level path | Files | Bytes |
|---|---:|---:|
| `frozen-binary/` | `1` | `9,654,272` |
| `logs/` | `2` | `62` |
| `report/` | `5` | `1,751,713` |
| `screenshots/` | `28` | `383,458` |
| `telemetry/` | `2` | `5,824,666` |
| `work/` | `16` | `80,656,634` |

The Run hash inventory contains `58` authenticated entries. `HASHES.sha256` and the final Receipt follow the receipt-last exclusion contract. The Receipt has `receipt_is_final_publication_marker=true`.

The Audit Bundle has `10` direct members, `39` nested Review Packet inventory entries, and `128` original-to-bundle path mappings. Every bundle-local hash passed. The bundle separately preserves `SOURCE_INPUT_BYTES.zip`, `GIT_SOURCE_ARCHIVE.zip`, the frozen executable, Receipt, Run hash inventory, source-input manifest, and Review Packet. The embedded executable and Receipt hashes equal the external canonical values above.

## Coordinator revalidation

The third audit invoked the current coordinator's read-only validation path only. It did not run the worker or create a new scratch/candidate.

| Validation fact | Result |
|---|---|
| Manifest validation | PASS |
| Raw telemetry recomputation | PASS |
| Samples / events / folded frames | `2,504 / 319 / 14` |
| Run hash inventory entries | `58`, all matched |
| Source-input seal recapture | PASS |
| Frozen executable recheck | PASS |
| Receipt contract | PASS |
| Report / Receipt / recomputed Heavy summary equality | PASS |
| Audit Bundle local hashes and mapping | PASS |
| Candidate blocker | `false` |
| Failed hard predicates | `[]` |
| Automatic verdict | `NEEDS_HUMAN_REVIEW` |
| Human verdict | **USER ACCEPTANCE PENDING** |

## Hard predicates

All fourteen hard predicates pass.

| Predicate | Status | Evidence summary |
|---|---|---|
| `matter_movement_observed` | pass | first tick `1`, sample `1` |
| `density_displacement_observed` | pass | first tick `8`, sample `3`; ordered Oil-above-Water plus liquid motion and interface |
| `thermal_activity_observed` | pass | first tick `1`, sample `1` |
| `phase_work_observed` | pass | first tick `1`, sample `1` |
| `combustion_observed` | pass | first post-tick dynamic work at tick `1`, sample `1`; authored tick-0 flags excluded |
| `smoke_work_observed` | pass | first new decay-age-zero Smoke at tick `1`, sample `1`; authored tick-0 Smoke excluded |
| `pressure_activity_observed` | pass | first tick `8`, sample `3` |
| `meaningful_multi_system_overlap` | pass | `1,986` consecutive sampled records with at least three active subsystems |
| `inventory_accounted` | pass | every sampled material delta fits the allowed transition model |
| `no_invalid_materials` | pass | occurrence count `0` |
| `no_nonfinite_fields` | pass | occurrence count `0` |
| `no_wake_anomalies` | pass | USER_EDIT/unknown-bit occurrence count `0` during production |
| `no_unbounded_runaway` | pass | terminal Temperature/Pressure maxima do not meet the runaway rule |
| `exact_reset` | pass | programmatic R-equivalent state exactly equals pristine tick 0 |

## Milestones and concurrency

| Metric | Exact value |
|---|---|
| First movement | tick `1` |
| First density displacement | tick `8` |
| First phase work | tick `1` |
| First dynamic combustion work | tick `1` |
| First newly generated Smoke | tick `1` |
| First Pressure activity | tick `8` |
| First relief damage | tick `1` |
| First pressure-qualified non-combusting rupture confirmation | tick `32` |
| First all-intended four-subsystem sample | tick `8` |
| Peak subsystem concurrency | `4` at tick `8` |
| Peak active cells | `40,301` at tick `3,528` |
| Longest at-least-three-subsystem window | `1,986` samples, ticks `1..15,872`, span `15,871` |

Final `material_count_deltas_by_id` is `[6799, 0, 0, 0, -11312, -4592, 16305, -880, -2232, -4088]`. Unexplained material-delta occurrences, invalid-material occurrences, non-finite-field occurrences, and wake-anomaly occurrences are all zero.

## Terminal state and review-only tail

Terminal reason is `max-ticks` at tick `20,000`, sample `2,502`. The reset sample is `2,503` and is the final folded frame.

| Terminal metric | Exact value |
|---|---:|
| Any active cells | `25,833` (`39.418030%` of `65,536`) |
| Matter active cells | `264` |
| Thermal active cells | `25,828` |
| Pressure active cells | `0` |
| Reaction active cells | `0` |
| Active subsystems | `2` |
| Active / runnable / sleeping chunks | `14 / 16 / 0` |
| Temperature min / max | `0 / 837.3743896484375` |
| Pressure min / max | `0 / 9.369529288960621e-5` |

The terminal 64-sample window spans ticks `19,496..20,000`:

- Temperature max: `844.7784423828125 -> 837.3743896484375`
- Pressure max: `9.780732943909243e-5 -> 9.369529288960621e-5`
- Temperature / Pressure positive steps: `0 / 0`
- Temperature runaway / Pressure runaway / global unbounded growth: `false / false / false`

Review flags:

- `broad_terminal_tail=true`
- `dominant_subsystem=false`
- dominant subsystem name/share: `thermal / 0.7821129285912836`
- `long_thermal_pressure_tail=false`
- reasons: `["broad_terminal_tail"]`

The broad tail is the sole reason for automatic `NEEDS_HUMAN_REVIEW`. It is not a hard failure, candidate blocker, user rejection, or established production-physics defect.

## Exterior Steam and raw-kind vocabulary residue

The candidate records:

- raw `first_vent_tick=3,920`
- event `first_vent_observed`
- frame badge `first-vent`
- first complete relief lane tick `3,960`

These `first_vent*` names are preserved schema/event/frame vocabulary from the immutable candidate. Their actual Heavy semantics are **first exterior Steam above relief**, independent of whether a complete lane is open. They are not opening-gated and do not prove that Steam crossed a complete relief lane. Because exterior Steam is first observed before the first complete lane in this run, docs and human review must not describe tick `3,920` as causal venting.

The Review Prompt and Report already state this boundary: optional raw exterior-Steam observations must not substitute for hard predicates or be presented as causal vent evidence. This docs closure preserves the raw names without rewriting the candidate.

## Human-facing wording audit

The published Report and Review Prompt correctly state that:

- automatic `NEEDS_HUMAN_REVIEW` is a telemetry claim, not user acceptance, product readiness, or G8-B/G8-C closure;
- authored tick-0 Smoke and combusting flags are not dynamic Smoke/combustion evidence;
- the Review Packet is lightweight human-review evidence, while the sibling Audit Bundle carries source/binary forensic identity;
- folded badges sharing one physical state must all be reviewed;
- optional relief damage, rupture, complete-lane, and exterior-Steam observations do not replace the required hard predicates;
- no upload, external message, code change, or other action is authorized by the inert prompt.

One non-blocking raw-log wording residue remains: the worker startup line writes `scenario=heavy-mixed-world`, while the manifest, schema, CLI, completion line, Report, and Receipt use the canonical slug `heavy-mixed`. Docs use `heavy-mixed` for the slug and Heavy Mixed World for the display name. This does not change machine identity or artifact validation.

## Recommended manual review

Use the canonical Gallery and Cell Inspector for at most these three candidate frames:

1. `frame-002_sim-000008_sample-000003_ordered-water-oil-displacement.png`: peak four-subsystem concurrency with density and Pressure evidence. Inspect Water edge `(134,90)`, Oil edge `(142,90)`, and chamber medium `(176,150)`; their approximate 1600 x 900 capture positions are `(819,329)`, `(843,329)`, and `(944,507)`.
2. `frame-005_sim-003920_sample-000492_exterior-steam-above-relief.png`: inspect detector-band cell `(163,134)`, relief seam `(176,144)`, and chamber side `(176,148)`; approximate capture positions are `(905,459)`, `(944,489)`, and `(944,501)`. Treat this only as first exterior Steam above relief, not proof of opening-gated causal venting.
3. `frame-012_sim-020000_sample-002502_max-tick-reached.png`: terminal broad Thermal tail. Inspect upper field `(128,80)` and chamber interior `(176,180)`; approximate capture positions are `(802,299)` and `(944,596)`.

The candidate Contact Sheet is `report/CONTACT_SHEET.png` inside the immutable Run and Review Packet. It is chronological, folds same-state milestones, keeps reset last, and labels tick `3,920` as `first-exterior-steam`.

## Acceptance state and next action

- Cell Inspector v0: **USER ACCEPTED WITH KNOWN FOLLOW-UP**.
- Heavy Mixed implementation and machine publication: complete for this immutable candidate.
- Heavy Mixed automatic verdict: `NEEDS_HUMAN_REVIEW`, unchanged.
- Heavy Mixed hard predicates: `14 / 14 PASS`.
- Heavy Mixed candidate blocker: `false`.
- Heavy Mixed human verdict: **USER ACCEPTANCE PENDING**.
- G8-B: **NOT CLOSED**.
- G8-C: **FORBIDDEN** until Heavy Mixed user acceptance and explicit G8-B closure.
- Production physics, fixture, source, executable, candidate, and artifacts must not be changed or rerun for this docs-only closure.
