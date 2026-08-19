# G8-C Official Performance Matrix — Verified Evidence Closure

Date: 2026-08-19  
Branch: `feature/m0-g8c-official-matrix`  
Sealed source: `4653d7c2e09e93f80fb81eeb73458d992c86858f`  
Matrix ID: `g8c-official-matrix-4653d7c2e09e-64df60ba0d79`  
Status: **OFFICIAL CAPTURE COMPLETE / INDEPENDENT VERIFICATION PASS / RECOMMENDATION `PROCEED_TO_G9`**

## 1. Scope

G8-C measures the five user-accepted G8-B workloads under one common official contract and answers one product question:

> Does the current M0 engine have a simulation, rendering, coexistence, or memory blocker that justifies optimization before the first playable sandbox?

This closure does not start G9, implement optimization, close G8-A user visual validation, promote `main`, or declare M0 achieved.

Official scenarios:

1. Sand Fall
2. Water Flow
3. Fire / Heat
4. Pressure Burst
5. Heavy Mixed World

Common official configuration:

- World: `2048 × 2048`
- Chunk: `64`
- Sleep: ON, threshold `16`
- Hardware: NVIDIA RTX 5090 / DX12
- Build: isolated locked Cargo release
- Mode C/D surface: physical `1600 × 900`, `Bgra8UnormSrgb`, `PresentMode::Fifo`
- HUD / Cell Inspector / text diagnostics / screenshot readback: OFF

## 2. Measurement roles

- **Mode A — Headless production throughput:** ordinary non-profiled context, batch production ticks, one completion wait after the measured window.
- **Mode B — Headless GPU breakdown:** separate timestamp-enabled context, synchronized per-tick 17-pass timing, six subsystem groups, envelope/pass-sum/residual and raw identity.
- **Mode C — Windowed production coexistence:** 60-TPS simulation target and normal rendering together; records simulation rate, FPS, frame percentiles, deadlines, catch-up, drops and surface/device errors.
- **Mode D — Windowed GPU render profile:** separate timestamp-enabled window context; measures render-pass GPU time without folding profiling overhead into Mode C product responsiveness.

Mode B is diagnostic and must not be reported as sustained throughput. Mode D GPU render time is not the complete wall-frame cost.

## 3. Source, binaries and publication identity

| Item | Exact identity |
|---|---|
| Sealed source | `4653d7c2e09e93f80fb81eeb73458d992c86858f` |
| Commit subject | `fix: complete the G8-C measurement aggregation contract` |
| Benchmark executable SHA-256 | `29131418a091d1657960c8cf1307d533582fa69e140af330b69be530c4394ed5` |
| Windows executable SHA-256 | `2c1670bff506cc9793da9e3708cafb28b6485d14bc577abbcb5faa04f897c4e5` |
| Matrix Receipt SHA-256 | `1fbf4599893cc29e99b6033996b42fcdf025aac0b421cb80b95b3e55807455f6` |
| Matrix Package SHA-256 | `92f8b85cc0e34ea6e71a9f6b4fc95b0f70704263a0f798a69a830cce1d40b729` |
| Verification result SHA-256 | `77c7e1c982296277c451de02c3dca68fa6d7d9a90e9fd5426c4dffa1abd9bb0d` |
| `HASHES.sha256` SHA-256 | `8ade901cc359c2cdfb750f01fff35f0fae463046757e6cee4ba44100c0b8c260` |

Official execution counts:

- isolated build: `1`
- Headless A/B scenario processes: `5`
- Mode C scenario processes: `5`
- Mode D scenario processes: `5`
- official capture: `1`
- independent verification: `1`
- matrix package: `1`
- workspace FULL: `0`
- G8-B candidate reruns: `0`

Every official subprocess exited `0`.

## 4. Independent verification

The independent verifier reported `verified: true` and reconstructed **230 matrix fields** from raw inputs with mismatch count `0`.

The verifier independently checked:

- exact source-input and Git identity
- both frozen binary hashes
- command/log/exit identities
- five scenarios exactly once under the common config
- Mode A throughput and wall statistics
- Mode B pass/group/envelope/residual and grouped-percentile method
- activity census and tracked memory
- Mode C simulation/frame/deadline accounting
- Mode D timestamp ordering and render percentiles
- matrix aggregation, Receipt-last, hash inventory and package identity

Capture summaries were not accepted as proof without raw reconstruction.

## 5. Five-scenario headless results

| Scenario | Mode A TPS P50 | Mean | Min | Max | Wall P50 / P95 ms | Mode B envelope P50 / P95 ms | 60-TPS headroom |
|---|---:|---:|---:|---:|---:|---:|---:|
| Sand Fall | 957.317 | 957.839 | 957.310 | 958.890 | 1.044586 / 1.044594 | 1.017440 / 1.022848 | 15.955× |
| Water Flow | 943.442 | 944.477 | 941.024 | 948.965 | 1.059949 / 1.062672 | 1.028832 / 1.034880 | 15.724× |
| Fire / Heat | 950.815 | 950.649 | 947.248 | 953.885 | 1.051729 / 1.055690 | 1.022752 / 1.026560 | 15.847× |
| Pressure Burst | 931.602 | 933.032 | 930.153 | 937.340 | 1.073419 / 1.075092 | 1.040960 / 1.046784 | 15.527× |
| Heavy Mixed World | 938.956 | 940.537 | 937.518 | 945.136 | 1.065013 / 1.066646 | 1.029248 / 1.034784 | 15.649× |

The largest Mode B grouped P50 subsystem in every scenario is **Active / Sleep management**. This is a measured attribution, not by itself an optimization mandate.

Persistent application-tracked GPU allocation is `184,576,672 bytes` per scenario (`~0.172 GiB`, `~0.537%` of RTX 5090 32 GiB). It is requested persistent buffer accounting, not driver-reported resident VRAM.

## 6. Windowed coexistence and render timing

| Scenario | Mode C sim TPS | Mode C FPS | Frame P50 / P95 / P99 ms | Mode D GPU P50 / P95 / mean ms |
|---|---:|---:|---:|---:|
| Sand Fall | 59.898583 | 239.094345 | 4.1838 / 4.1988 / 4.2100 | 0.009856 / 0.020768 / 0.012613 |
| Water Flow | 59.898708 | 239.094843 | 4.1837 / 4.2001 / 4.2133 | 0.009824 / 0.019872 / 0.012362 |
| Fire / Heat | 59.898580 | 239.094333 | 4.1837 / 4.2005 / 4.2136 | 0.009920 / 0.021280 / 0.012769 |
| Pressure Burst | 59.898709 | 239.094848 | 4.1838 / 4.1975 / 4.2079 | 0.009888 / 0.019520 / 0.012307 |
| Heavy Mixed World | 59.898765 | 239.095070 | 4.1838 / 4.1995 / 4.2122 | 0.009888 / 0.019776 / 0.012390 |

All ten Mode C/D workers recorded:

- initial and final live physical size `1600 × 900`
- total canonical no-op events `20`
- total stale initial `2864 × 1560` payloads safely ignored `10`
- fatal live resize `0`
- missed deadlines / catch-up ticks / dropped frames `0 / 0 / 0`
- surface errors / device errors `0 / 0`
- Mode D timestamp-order errors `0`

A stale event payload is ignored only when current live `window.inner_size()` remains exactly canonical. Genuine noncanonical or zero live size remains fatal; arbitrary resolution and adaptive renderer resizing are not permitted by the official contract.

## 7. Adapter and aggregation integrity

The historical benchmark producer remains unchanged:

```text
external metric name = wall_per_tick
unit                 = ms/tick
```

The G8-C coordinator and verifier use an explicit adapter to map this external vocabulary to the internal field `wall_ms_per_tick`. Raw alias `wall_ms_per_tick`, wrong units, missing or duplicate rows, wrong scenario/mode/selection/trial, trial-summary confusion and non-finite values are rejected.

Tests use the actual producer's 37-column CSV shape rather than an invented internal vocabulary.

The passing aggregation-only replay:

- ID: `g8c-aggregation-replay-20260819T015515996891Z-fc408076b67a`
- source pilot: `g8c-pilot-8ee1ae238c32-6341f4f59218`
- launched executable/process/GPU context/measurement subprocess: `0`
- source pilot inputs: 98 files / 57,021,663 bytes, byte-identical before and after
- `non_evidence=true`

The replay validates parser, aggregation, publication and verifier behavior only. It is not official performance evidence.

## 8. Retained diagnostic pilots

Two failed pilots are preserved and are not repaired or promoted:

1. `g8c-pilot-8ee1ae238c32-c64090539536`
   - failed first Sand Mode C on a stale initial window-size payload
   - no final Receipt/package/report/verifier
2. `g8c-pilot-8ee1ae238c32-6341f4f59218`
   - all 15 measurement subprocesses exited `0`
   - final aggregation failed on historical CSV vocabulary mismatch
   - no official Matrix publication

Their binary hashes and raw measurements remain pilot-only diagnostics. Pruning requires a separate retention decision.

## 9. Optimization decision

**Official recommendation: `PROCEED_TO_G9`**

Evidence basis:

- minimum Mode A P50: `931.602 TPS`
- maximum Mode B P95: `1.046784 ms`
- minimum Mode C simulation rate: `59.898580 TPS`
- zero Mode C deadline misses, catch-up ticks and dropped frames
- maximum Mode C frame P95: `4.2005 ms`
- maximum Mode D render P95: `0.021280 ms`
- persistent tracked memory approximately `0.54%` of the primary GPU's 32 GiB
- no measurement-integrity mismatch

No representative workload blocks the current 60-TPS M0 target or simulation/render coexistence. No current evidence establishes a world-scale or content-budget blocker. Therefore G7-C compaction, indirect dispatch, packing, f16 and other aggressive optimization remain deferred.

This recommendation is not automatic user authorization. G9 starts only after a user-approved product brief and scope.

## 10. Remaining closure items

- G8-A same-SHA user visual durable disposition remains pending and separate.
- G8 overall closure requires an explicit user decision after resolving that disposition.
- G9, optimization and `main` promotion were not started by the official Matrix task.
- M0 remains `IN_PROGRESS`; G9 product validation is still required for `ACHIEVED`.
