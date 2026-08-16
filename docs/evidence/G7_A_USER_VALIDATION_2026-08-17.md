# G7-A User Validation — Activity Observatory (2026-08-17)

Status: **USER VALIDATION APPROVED** for G7-A measurement baseline only.

This does **not** close G7. Actual sleep/work skipping and wake correctness remain G7-B/C.

## Observed run

User directly observed `--activity-demo` from reset through long-run FAST x16 execution.

Representative GPU-readback observations:

| Sim Tick | Sample | Matter | Thermal | Pressure | Reaction | Fully Stable | Max Stable Ticks | Wake Candidates |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 0 | 0 | 0 | 0 | 0 | 16 | 0 | 0 |
| 115 | 110 | 3 | 2 | 0 | 0 | 12 | 110 | 5 |
| 297 | 295 | 3 | 2 | 0 | 1 | 13 | 295 | 5 |
| 813 | 810 | 4 | 3 | 1 | 2 | 12 | 810 | 6 |
| 1536 | 1535 | 4 | 3 | 1 | 2 | 12 | 1535 | 6 |
| 5969 | 5937 | 2 | 2 | 0 | 0 | 13 | 5937 | 6 |
| 33425 | 33393 | 2 | 2 | 0 | 0 | 13 | 33393 | 6 |

FAST x16 showed approximately 960 sim TPS in this 256x256 observatory. This is an observation convenience value, **not a G8 performance claim**.

## User-observed evidence

- Stable chunks remained stable for tens of thousands of ticks; `Max Stable Ticks` reached 33393.
- Activity remained localized instead of spreading to all 16 chunks: typically only 2–4 Matter, 2–3 Thermal, 0–1 Pressure and 0–2 Reaction chunks were active.
- 12–13 of 16 chunks commonly remained fully stable while live frontiers continued elsewhere.
- Heatmap colors and HUD counters agreed visually with moving, thermal, pressure and reaction frontiers.
- Sampled wake candidates remained small and localized (5–6), rather than producing world-wide false wake.
- Long-run FAST x16 observation did not show activity-state corruption or runaway false activity.
- HUD text slightly exceeded its intended card bounds in places, but remained readable; this is a non-blocking presentation issue.

## G7-A verdict

**APPROVED / COMPLETE as the Activity Observatory and measurement baseline.**

What G7-A proves:

1. stable bulk can be identified and measured over long periods;
2. changeable frontiers remain spatially localized;
3. Matter/Thermal/Pressure/Reaction activity categories are usable for diagnostics;
4. wake-candidate measurement is bounded and observable;
5. the observatory remains usable during long fast-forward runs.

What G7-A does **not** prove:

- subsystem dispatch is actually skipped for sleeping chunks;
- sleeping chunks wake correctly from every external/internal trigger;
- sleep produces zero semantic divergence versus the always-active baseline;
- any production performance gain.

Those belong to G7-B/C. G7 overall remains **IN_PROGRESS**.
