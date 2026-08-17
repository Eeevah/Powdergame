# G8-B — Sand Fall Experiment Evidence Harness v0 (2026-08-17)

## 1. Status and authority boundary

- **Experiment ID**: `g8b-sand-fall-v0`
- **Accepted scenario contract**: Scenario 1 `sand-fall` — **USER ACCEPTED**
- **Accepted success interpretation**: Sand가 실제로 낙하한 뒤 완전히 정착하고 모든 chunk가 sleep 상태로 수렴하는 것
- **G8-B checkpoint**: `feature/m0-g8b-scenario-suite` at `e77d102febb1e3c497c2b669efe0140408bd99d7`
- **Harness development line**: `feature/g8b-experiment-harness-v0`, stacked on `e77d102`
- **Harness state**: validated pilot **PASS**; `HARNESS REVIEW OUTPUT APPROVED`; documentation closure follows the experiment source commit
- **Gate state**: G8-B **USER ACCEPTANCE PENDING / NOT CLOSED**; G8-C not started

The harness automates one immutable evidence run for the already accepted Sand Fall behavior. It does not redefine the fixture, add a new physics result, or substitute automated telemetry for the remaining Scenario 2–5 user acceptance.

An automatic `PASS` means only that every hard Sand Fall predicate in this document evaluated true for that run. It does **not** close G8-B, approve Water Flow, establish G8-C performance, or authorize branch publication or `main` promotion.

---

## 2. Scope

Included:

- the shared `powdergame-scenarios::reset_and_stage_scenario` Sand Fall tick-0 fixture;
- production `Simulation::tick()` execution without an alternate physics path;
- out-of-band GPU state/activity readback;
- renderer-path full-frame capture and derived crops;
- machine-readable telemetry, hard-predicate evaluation, human-readable reports, hashes, review packet, and a receipt-last completion marker;
- a unique external run directory that is never overwritten or reused.

Explicitly excluded:

- Water Flow inspection, correction, retuning, or evidence;
- any production physics, WGSL/pass-graph, Material, sleep-threshold, or accepted Sand geometry change;
- G8-C throughput, GPU timing, rendering-cost, or coexistence measurement;
- G9, Interaction Lab, G7-C compaction, indirect dispatch, or another optimization;
- automatic upload, external AI contact, review request, commit, push, merge, or Gate closure.

---

## 3. One-command runner

From the repository root:

```bat
run_experiment.bat sand-fall
```

The runner accepts only `sand-fall`. Its production artifact root is fixed to:

```text
C:\Users\mdkap\source\Powdergame-artifacts
```

The runner requires a clean named source branch, creates a unique run ID and directory, performs a locked release build, records the executed binary SHA-256, launches the strict Windows experiment worker, validates the worker output, derives PNG/report artifacts, builds the review packet, hashes the run, and writes the receipt last.

The validated pilot was produced from clean experiment source `9e1fdac44aa14a546c7fe5ad6ceba49e71777eb5`. The later docs-only closure commit records the result and must not be reported as experiment provenance. Sand Fall and the Harness pilot must not be rerun for this closure.

---

## 4. Fixed v0 configuration

| Field | v0 value |
|---|---:|
| Scenario | `sand-fall` |
| World | 256 × 256 |
| Chunk size | 64 |
| Sleep | enabled |
| Diagnostic interval | 8 simulation ticks |
| Consecutive all-sleep samples | 3 |
| Post-sleep confirmation | 180 production simulation ticks |
| Maximum lifecycle tick | 20,000 |
| Full capture size | 1600 × 900 RGBA |
| Final semantic frames | 6–10 |

The lifecycle maximum comes from the v0 manifest/runner contract. The worker does not use the older hard-coded tick-1600 assumption.

---

## 5. Lifecycle and identity

The worker preserves simulation tick and diagnostic sample identity as separate values. `first_all_sleep_sim_tick` is the simulation tick attached to the first observed all-sleep sample in the confirmed three-sample streak; it is not the diagnostic sample sequence and is not presented as an unsampled exact transition time.

Lifecycle:

1. shared pristine Sand Fall reset/stage;
2. tick-0 state sample and frame;
3. exactly one production tick, followed by tick-1 sample and frame;
4. bounded settling samples and retrospective peak-active selection;
5. first observed sleeping-chunk frame;
6. late-settling frame immediately before the confirmed all-sleep streak;
7. first-all-sleep frame and confirmation after three consecutive qualifying samples;
8. 180 additional production ticks, with state-change and wake checks on every tick;
9. programmatic `R`-equivalent shared reset/stage and exact tick-0 comparison;
10. reset frame, worker finalization, and exit.

A diagnostic sample qualifies as all-sleep only when all four conditions are simultaneously true:

```text
active cells   = 0
active chunks  = 0
runnable chunks = 0
sleeping chunks = total chunks
```

Peak-active and late-settling images are retained from already observed renderer frames. The harness does not replay the world, script coordinates to manufacture results, or mutate the simulation to reproduce a past frame.

---

## 6. Automatic verdict

The worker records exactly seven hard predicates:

1. `actual_fall` — Sand vertical-position aggregate increased from pristine tick 0;
2. `matter_conservation` — registered non-empty Matter count remained equal to the tick-0 baseline;
3. `no_invalid_materials` — invalid Material count stayed zero;
4. `no_nonfinite_fields` — non-finite Temperature/Pressure count stayed zero;
5. `sleep_before_max` — three-sample all-sleep confirmation completed before the maximum lifecycle tick;
6. `post_sleep_stable` — the 180-tick confirmation window recorded zero authoritative state changes and zero wake/activity recurrence;
7. `exact_reset` — the programmatic shared reset snapshot exactly matched pristine tick 0 across the compared authoritative Current/Next and activity/edit-wake state.

Verdict mapping:

- `PASS`: all seven hard predicates are `pass`;
- `FAIL`: one or more hard predicates are `fail`;
- `NEEDS_HUMAN`: no predicate is a definite failure, but one or more required predicate values are unknown or ambiguous.

The presence of `CHATGPT_REVIEW_PROMPT.md` does not downgrade an otherwise automatic `PASS`. It is a local, inert review aid and is not sent anywhere by the runner.

---

## 7. External artifact layout

Every run is created directly under the fixed external artifact root:

```text
C:\Users\mdkap\source\Powdergame-artifacts\<unique-run-id>\
├─ EXPERIMENT_MANIFEST.toml
├─ EXPERIMENT_RECEIPT.json          # written last; absent means incomplete
├─ HASHES.sha256
├─ stdout.log
├─ stderr.log
├─ logs\
│  ├─ build.stdout.log
│  └─ build.stderr.log
├─ telemetry\
│  ├─ samples.jsonl
│  └─ events.jsonl
├─ work\
│  ├─ analysis.json
│  ├─ frames.json
│  └─ frames\*.rgba                # tightly packed semantic RGBA frames
├─ screenshots\
│  ├─ full\*.png
│  └─ crops\*.png                  # derived only from corresponding full PNG
├─ report\
│  ├─ REPORT.md
│  ├─ REPORT.json
│  ├─ CONTACT_SHEET.png
│  ├─ CHATGPT_REVIEW_PROMPT.md
│  └─ REVIEW_PACKET.zip
```

`samples.jsonl` includes raw sample metrics needed to inspect the hard predicates: separate sample/simulation identities, activity census, Material counts, Sand position aggregates, invalid/non-finite counts, chunk change/wake values, and authoritative state hashes. `events.jsonl` records lifecycle transitions and their attached identities. `analysis.json` is the worker predicate/verdict output; `REPORT.json` and `REPORT.md` are coordinator renderings, not replacements for raw telemetry.

Full PNGs are authoritative renderer captures. Every crop and the contact sheet is derived from those PNGs after the run; they are not CPU re-renders of simulation state.

---

## 8. No-overwrite and receipt-last publication

- The runner atomically reserves a new run directory. An existing run ID or output path is a nonzero failure.
- A failed run directory is preserved and never repaired, overwritten, or reused.
- Worker JSONL/log files are created once and closed before post-processing.
- Reports, screenshots, crops, contact sheet, and review prompt are published with create-new semantics.
- `REVIEW_PACKET.zip` is created before the hash manifest. It includes the manifest, worker/build logs, telemetry, report/prompt/contact sheet, and screenshots. Worker-only `work/` data remains in the immutable run but is excluded from this curated packet; the packet also excludes itself, `HASHES.sha256`, and the receipt, avoiding a circular package definition.
- `HASHES.sha256` covers every other regular pre-receipt run file, including the completed review packet.
- The coordinator validates manifest/run identity, telemetry and frame inventories, hard-predicate schema, file paths, and hashes before publication completion.
- `EXPERIMENT_RECEIPT.json` is the final write and final publication marker. No file inside the run directory may change afterward.

Receipt absence means an incomplete run even if some logs, telemetry, frames, reports, or a packet exist. A semantic `FAIL` or `NEEDS_HUMAN` may still be a structurally complete receipted run; an operational build/GPU/I/O/validation failure remains preserved without a receipt.

Generated run artifacts are external evidence and must never be added to Git. Only source, tests, tools, launchers, and documentation belong to the repository.

---

## 9. Validated pilot and closure boundary

The Sand Fall pilot and its preserved external artifacts were validated without rerunning the experiment:

```text
run_id: g8b-sand-fall-v0-20260817T065311878587Z-3ebd7505
experiment_source_sha: 9e1fdac44aa14a546c7fe5ad6ceba49e71777eb5
automatic_verdict: PASS
first_all_sleep_sim_tick: 1096
first_all_sleep_diagnostic_sample_sequence: 139
all_sleep_confirmed_sim_tick: 1112
post_sleep_ticks: 180
post_sleep_state_changes: 0
post_sleep_wakes: 0
matter_delta: 0
invalid_material_ids: 0
nan_or_inf_values: 0
exact_reset: true
review_packet_sha256: feffc180f81d36558b8139f5436a2a0eed6422617dd9a3207153b1cb62af1323
receipt_sha256: 42bbacf77ca80356996a53fb2d0a56a5aba18215b5b85bda637350522c95e033
harness_review_output: HARNESS REVIEW OUTPUT APPROVED
```

Contact Sheet review approved the keyframe selection. The all-sleep-related frames that look the same are retained because they prove distinct semantic milestones: the observation immediately before the confirmed streak, the first observed all-sleep sample within that streak, and completion of the 180-tick post-sleep stability window. The exact-reset frame separately proves restoration of the pristine tick-0 state. Compact per-tile metric captions are a non-blocking future improvement.

The documentation closure commit is the later docs-only commit that first contains this result and is distinct from experiment source `9e1fdac44aa14a546c7fe5ad6ceba49e71777eb5`; report its exact SHA after that commit exists. Sand Fall and the Harness pilot must not be rerun for this closure.

G8-B remains **USER ACCEPTANCE PENDING / NOT CLOSED** because Water Flow, Fire / Heat, Pressure Burst, and Heavy Mixed World remain pending. G8-C is also pending and must not begin from this closure.
