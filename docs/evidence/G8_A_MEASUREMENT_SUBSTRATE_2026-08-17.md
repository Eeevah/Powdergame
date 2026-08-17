# G8-A — Performance Measurement Substrate Evidence (2026-08-17)

G8 — Performance Evidence gate, sub-step A (measurement substrate correction candidate).

- **G7 Frozen Baseline SHA**: `94babb2667c081b5588489e1b4e710cc6efa68be`
- **G8-A Remediation Base**: `a67abaf959aba0423627f35b79fce7c82d8ec9b5`
- **Source Branch**: `fix/g8a-evidence-remediation-v5`
- **Historical v4 Run State**: the CSV records `git_state=dirty`; no later source may be rebound to it
- **Primary Hardware**: NVIDIA GeForce RTX 5090 (`0x10DE:0x2B85`, driver `32.0.15.9636`)
- **Backend / Build**: DirectX 12 / Cargo release profile
- **Status**: `V5 SOURCE FREEZE / CLEAN CHECKPOINT, PUSH, OFFICIAL RECEIPT, AND INDEPENDENT VERIFICATION REQUIRED`
- **Scope**: measurement, validation, and evidence corrections only. No production physics optimization was performed. G7-C and the G8-B five-scenario suite are not implemented.

The v4 calibration below is retained as historical raw data. Its CSV arithmetic can be independently reconstructed, but the packet does not bind that run to the later source snapshot, executable, command, stdout/stderr, and exit code. It is therefore not evidence that the v5 remediation source produced those values.

---

## 1. Measurement Architecture

### 1.1 Production and Profiled Paths

`Simulation::tick()` and `Simulation::tick_profiled()` use the same `tick_internal()` pass orchestration. The profiled path adds timestamp writes and readback around the same 17-pass sequence; the ordinary production context does not request `wgpu::Features::TIMESTAMP_QUERY`.

The exact-equivalence integration test runs 50 ticks from identical fixtures and compares Material, Flags, Temperature, and Pressure byte-for-byte. This establishes observational semantic equivalence; Mode B timing is still an intentionally synchronized diagnostic path and is not interchangeable with Mode A sustained wall time.

### 1.2 Timestamp Integrity

- 17 compute passes, 34 raw timestamp queries per profiled tick.
- Query `2i` is pass `i` start and query `2i + 1` is its end.
- Raw ticks are retained in the tick-level CSV.
- Conversion rejects non-finite or non-positive timestamp periods, equal/inverted pass endpoints, cross-pass inversions, non-positive envelopes, and pass sums larger than the envelope.
- `gpu_tick_envelope_ms` spans query 0 through query 33.
- `gpu_pass_sum_ms` is the sum of the 17 measured pass durations.
- `residual_ms` is derived in the integer tick domain before conversion and covers work between timestamped passes, including intermediate copies and scheduling gaps.

The measured pass order is:

1. `activity_wake`
2. `movement_propose`
3. `movement_claim`
4. `movement_commit`
5. `thermal`
6. `phase_transition`
7. `expansion_claim`
8. `expansion_spawn_commit`
9. `expansion_pressure`
10. `decay`
11. `combustion`
12. `smoke_claim`
13. `smoke_commit`
14. `pressure`
15. `rupture`
16. `activity_propose`
17. `activity_reduce`

### 1.3 Two Deliberately Separate Modes

**Mode A — production sustained throughput**

- Uses a normal `GpuContext::new()` without timestamp-query capability.
- Performs one context-level prewarm, then independently resets/restages each trial.
- Flushes scheduled reset/fixture `Queue::write_buffer` transfers with an empty submission, then waits for that work to complete before starting each trial timer.
- Submits 1,024 ordinary ticks in a batch and waits once after the measured window.
- Reports total wall time, wall time per tick, and sustained TPS.

**Mode B — isolated GPU breakdown**

- Creates a separate `GpuContext::with_profiling()` and verifies the adapter matches Mode A.
- Has its own context-level prewarm; each trial waits for reset/staging completion before its first profiled sample.
- Profiles and synchronously reads back every tick to preserve sample identity and raw query data.
- Reports each pass, six per-tick subsystem sums, pass sum, envelope, and residual.

Grouped P50/P95 values are percentiles of the **per-tick grouped sums**. They are not sums of independently computed pass percentiles. Group-to-envelope percentages are also computed per tick before percentile aggregation.

### 1.4 Census and Memory Scope

The activity census runs outside timed loops. Cell categories overlap: a cell can contribute to more than one of Matter, Thermal, Pressure, and Reaction, so category counts must not be summed as a partition. Chunk counts describe their own states.

The memory report is **application-tracked requested persistent GPU buffer bytes**, not resident VRAM. It includes dense world, movement scratch, activity diagnostics, all persistent uniforms/tables, and profiler resolve/readback buffers. It excludes opaque query-set backing storage, transient census/world/marker staging buffers, pipelines, bind groups, shaders, driver/backend allocations, and `queue.write_buffer` internals.

---

## 2. Corrected Reference Calibration

Run ID: `g8a-1786916099569`

Configuration: 2,048×2,048 cells, 64×64 chunks, sleep enabled with threshold 16, 2-second requested context prewarm (Mode A 1,920 ticks; Mode B 1,920 ticks), release build.

### 2.1 Mode A — 1,024 Ticks × 3 Trials

| Trial | Wall Time | Wall / Tick | Sustained TPS |
|---|---:|---:|---:|
| 1 | 1,083.04 ms | 1.0577 ms | 945.5 |
| 2 | 1,079.12 ms | 1.0538 ms | 948.9 |
| 3 | 1,076.32 ms | 1.0511 ms | 951.4 |

- **TPS**: P50 948.9, mean 948.6, min 945.5, max 951.4
- **Wall / tick**: P50 1.0538 ms, mean 1.0542 ms, min 1.0511 ms, max 1.0577 ms

### 2.2 Mode B — 256 Ticks × 3 Trials

| Trial | Envelope P50 | Envelope P95 | Pass Sum P50 | Residual P50 |
|---|---:|---:|---:|---:|
| 1 | 1.0204 ms | 1.0239 ms | 0.7663 ms | 0.2542 ms |
| 2 | 1.0205 ms | 1.0244 ms | 0.7663 ms | 0.2542 ms |
| 3 | 1.0201 ms | 1.0239 ms | 0.7662 ms | 0.2539 ms |

Trial 1 is the selected median-envelope trial for the following detailed summaries.

| Pass | P50 | P95 | Mean |
|---|---:|---:|---:|
| `activity_wake` | 0.0035 ms | 0.0046 ms | 0.0038 ms |
| `movement_propose` | 0.0328 ms | 0.0330 ms | 0.0328 ms |
| `movement_claim` | 0.0331 ms | 0.0336 ms | 0.0331 ms |
| `movement_commit` | 0.0390 ms | 0.0404 ms | 0.0391 ms |
| `thermal` | 0.0319 ms | 0.0322 ms | 0.0319 ms |
| `phase_transition` | 0.0333 ms | 0.0335 ms | 0.0334 ms |
| `expansion_claim` | 0.0332 ms | 0.0350 ms | 0.0333 ms |
| `expansion_spawn_commit` | 0.0325 ms | 0.0327 ms | 0.0325 ms |
| `expansion_pressure` | 0.0303 ms | 0.0304 ms | 0.0303 ms |
| `decay` | 0.0414 ms | 0.0428 ms | 0.0414 ms |
| `combustion` | 0.0412 ms | 0.0428 ms | 0.0409 ms |
| `smoke_claim` | 0.0332 ms | 0.0352 ms | 0.0333 ms |
| `smoke_commit` | 0.0313 ms | 0.0324 ms | 0.0313 ms |
| `pressure` | 0.0316 ms | 0.0325 ms | 0.0318 ms |
| `rupture` | 0.0318 ms | 0.0320 ms | 0.0318 ms |
| `activity_propose` | 0.0383 ms | 0.0393 ms | 0.0384 ms |
| `activity_reduce` | 0.2473 ms | 0.2478 ms | 0.2473 ms |

### 2.3 Correct Grouped Subsystem Statistics

| Group | P50 | P95 | Mean | Per-Tick Envelope Ratio P50 |
|---|---:|---:|---:|---:|
| Matter Movement | 0.071840 ms | 0.073376 ms | 0.071946 ms | 7.04% |
| Ownership / Claim | 0.099424 ms | 0.104032 ms | 0.099674 ms | 9.74% |
| Thermal Conduction | 0.031872 ms | 0.032224 ms | 0.031869 ms | 3.12% |
| Reaction / Phase | 0.210016 ms | 0.212224 ms | 0.209732 ms | 20.58% |
| Pressure / Structure | 0.063456 ms | 0.064352 ms | 0.063569 ms | 6.22% |
| Active / Sleep Management | 0.289472 ms | 0.290848 ms | 0.289530 ms | 28.37% |

These six groups partition the 17 pass durations. Their sum reconstructs `gpu_pass_sum_ms`; the residual is outside the groups.

### 2.4 Activity Census at Tick 256

- Cells: total 4,194,304; any active 266,016; Matter 220,275; Thermal 79,795; Pressure 1,898; Reaction 66,504.
- Chunks: total 1,024; active 219; runnable 381; sleeping 643.
- The four subsystem cell counts overlap and are diagnostic, not additive.

### 2.5 Application-Tracked Requested Buffer Bytes

| Category | Bytes |
|---|---:|
| Dense world state | 134,217,728 |
| Movement scratch | 33,554,432 |
| Activity diagnostics | 16,801,792 |
| Persistent uniforms and tables | 2,176 |
| Profiler resolve + mapped readback buffers | 544 |
| **Total with profiler buffers** | **184,576,672** |

The 2,176-byte static inventory covers all 14 persistent simulation buffers. The 544 profiler bytes are two 272-byte buffers; the query set itself is excluded because `wgpu` does not expose an application-requested byte size for it.

### 2.6 Overhead Controls — 256 Ticks

| Path | Wall Time |
|---|---:|
| Batched unprofiled | 268.00 ms |
| Per-tick synchronized unprofiled | 340.36 ms |
| Per-tick synchronized profiled | 346.40 ms |

- Synchronizing every unprofiled tick versus batching: **+27.00%**.
- Profiling increment over the synchronized unprofiled control: **+1.77%**.
- Combined profiled path versus batched production path: **+29.25%**.

The combined number must not be attributed solely to timestamp profiling: it includes per-tick synchronization and readback behavior. Mode B exists for attribution and raw samples, not sustained-throughput claims.

---

## 3. Evidence Artifacts and Validation

Local ignored artifacts:

- `target/g8a_correction_calibration_v4.csv` — schema `powdergame-g8a-v3`, 129 aggregate rows.
- `target/g8a_correction_calibration_v4_raw_ticks.csv` — 768 unique tick samples with named start/end tick fields for all 17 passes, pass durations, six group sums, an explicit group definition, pass sum, envelope, and residual.
- The earlier local v2 and v3 artifacts are retained only as invalidated history. v3 waited without first submitting pending `write_buffer` transfers, so reset/staging copies could still enter Mode A and overhead windows.
- The v4 source/executable linkage is absent. The collected `main.rs` snapshot was modified after the v4 CSV timestamps, so the packet cannot establish whether v4 ran the later empty-submit fence implementation.

Independent reconstruction of the historical v4 CSV found:

- one consistent provenance set and 768 unique `(trial, sample_id, tick_index)` identities;
- strict positive/in-order query pairs for every tick;
- exact envelope reconstruction from raw ticks;
- maximum pass-sum reconstruction error `2.22e-16 ms`;
- maximum group-sum reconstruction error `5.55e-17 ms`;
- maximum residual reconstruction error `2.22e-16 ms`;
- the exact 184,576,672-byte memory total.

The v5 remediation source adds the following future-capture contract:

- schema `powdergame-g8a-v5` emits aggregate, raw tick, raw cell, and raw chunk CSV files;
- raw cell output has exactly one data row per `cell_activity` value; raw chunk output has exactly one data row per `(chunk_activity, chunk_state)` pair;
- the census aggregate is recomputed from the snapshot before any evidence file is written, and a mismatch aborts publication;
- all four files are staged, flushed, and synchronized before publication; publication order is raw cell, raw chunk, raw tick, then aggregate, and every final path is no-overwrite;
- the four CSV publications are not represented as a cross-file transaction under process termination, OS crash, or power loss;
- official `capture-evidence.ps1` rejects dirty/detached source, performs an isolated locked release build outside the source tree, records source/executable/command/log/exit/artifact hashes, and writes `CAPTURE_RECEIPT.json` last;
- receipt absence means incomplete capture; a failed Capture ID is preserved and never reused;
- package creation follows the receipt, and the ZIP SHA-256 is written outside the ZIP as sibling `PACKAGE_SHA256.txt`;
- independent verification uses `verify-evidence.ps1`, not the capture implementation.

The contract requires one fresh official capture after the clean source SHA is committed, checked, and pushed. The historical v4 values are not rewritten or rebound.

Narrow implementation checks executed before the full source checkpoint:

- pure non-GPU census recount unit test: 1 executed, exit code 0;
- raw cell/chunk rectangular writer test: 1 executed, exit code 0;
- raw snapshot to census aggregate recomputation test: 1 executed, exit code 0;
- staged-publication order/failure/no-overwrite tests: 4 executed, exit code 0;
- `cargo fmt --all -- --check`: exit code 0;
- benchmark-package clippy with `-D warnings`: exit code 0;
- PowerShell AST parse of `capture-evidence.ps1`: 0 parse errors;
- `git diff --check`: exit code 0 with existing LF-to-CRLF working-copy warnings.

The authoritative full workspace, GPU integration, Windows release smoke, source commit/push, official capture, package, and independent-verifier results are external checkpoint/capture records produced after this source text is frozen. They must not be inferred from the narrow checks above.

### 3.1 Superseded and Current Review Records

An earlier local review report is retained as historical provenance, but its claim that v4 was generated after the corrected fence is not supported by the later evidence packet.

- A user-supplied review of `Powdergame-evidence.zip` independently reconstructed the v4 timing and aggregate calculations without finding a numerical mismatch.
- The same review found that v4 lacks a run-time source/binary/log binding, that the aggregate census cannot be independently recounted without its three raw GPU arrays, and that summary/raw publication can leave a summary-only file on a normal write failure.
- It also noted two packet-delivery/format items: the sibling `PACKAGE_SHA256.txt` was not attached with the ZIP, and the generated review-target TSV contained two empty-path records. The sibling file exists locally and matches the ZIP; future packet inventories must reject empty paths.
- No further external review is requested by this correction. Review remains explicit-request-only.

---

## 4. Gate Declaration

- **G8-A source candidate**: scope is frozen on `fix/g8a-evidence-remediation-v5`; the current evidence candidate is whichever external v5 package has a complete official receipt and independent-verifier record for the final clean source SHA.
- **G8 final PASS**: no.
- **G8-B official five-scenario suite**: not started.
- **G7-C compact active lists / indirect dispatch**: not implemented.
- **Production physics changes**: none.
- **Publication boundary**: only source/test/docs belong in the v5 branch commit; generated CSVs, receipt, executable, logs, verifier output, ZIP, and package hash remain outside Git.
