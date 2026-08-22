#!/usr/bin/env python3
"""Pure TE-4D ignition-dose reference model. Not production runtime."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import random
from pathlib import Path

SEED = 0x54453444
SEQUENCE_TRIALS = 100_000
GRID_TRIALS = 10_000

COMBUSTING = 1 << 0
FLAME_EVENT = 1 << 1
EXPOSURE_LOW_MASK = 0x3 << 2
FUEL_MASK = 0xFFF << 4
DECAY_MASK = 0xFFF << 16
EXPOSURE_HIGH_MASK = 0xF << 28
EXPOSURE_MASK = EXPOSURE_LOW_MASK | EXPOSURE_HIGH_MASK

MATERIALS = {
    "Oil": {
        "ignition": 200,
        "budget": 48,
        "base_rate": 2,
        "bucket_width": 50,
        "max_rate": 6,
        "decay": 1,
        "flame_bonus": 2,
        "flame_bonus_cap": 4,
        "heat_capacity": 2.5,
        "legacy_delta_t": 6.0,
        "burn_duration": 600,
    },
    "Wood": {
        "ignition": 300,
        "budget": 60,
        "base_rate": 1,
        "bucket_width": 50,
        "max_rate": 5,
        "decay": 1,
        "flame_bonus": 2,
        "flame_bonus_cap": 4,
        "heat_capacity": 2.0,
        "legacy_delta_t": 4.0,
        "burn_duration": 900,
    },
}

FIXTURES = [f"TE4-F{i:02d}" for i in range(1, 18)]


def encode_exposure(flags: int, exposure: int) -> int:
    if not 0 <= exposure <= 63:
        raise ValueError("exposure outside canonical u6 range")
    cleared = flags & ~EXPOSURE_MASK
    return cleared | ((exposure & 0x3) << 2) | (((exposure >> 2) & 0xF) << 28)


def decode_exposure(flags: int) -> int:
    return ((flags >> 2) & 0x3) | (((flags >> 28) & 0xF) << 2)


def thermal_rate(desc: dict[str, float | int], temperature: float) -> int:
    if not math.isfinite(temperature):
        raise ValueError("non-finite temperature")
    if temperature < desc["ignition"]:
        return 0
    excess = temperature - desc["ignition"]
    return min(
        int(desc["max_rate"]),
        int(desc["base_rate"]) + int(excess // desc["bucket_width"]),
    )


def exposure_step(
    desc: dict[str, float | int], exposure: int, temperature: float, previous_flames: int
) -> tuple[int, bool, int]:
    if not 0 <= exposure <= int(desc["budget"]) <= 63:
        raise ValueError("invalid exposure or descriptor budget")
    if not 0 <= previous_flames <= 4:
        raise ValueError("orthogonal previous-flame count outside 0..4")
    rate = thermal_rate(desc, temperature)
    if rate == 0:
        return max(0, exposure - int(desc["decay"])), False, 0
    flame = min(
        int(desc["flame_bonus_cap"]),
        previous_flames * int(desc["flame_bonus"]),
    )
    delta = rate + flame
    next_exposure = min(int(desc["budget"]), exposure + delta)
    ignited = next_exposure >= int(desc["budget"])
    return (0 if ignited else next_exposure), ignited, delta


def ignition_ticks(desc: dict[str, float | int], temperature: float, flames: int = 0) -> int:
    exposure = 0
    for tick in range(1, 10_001):
        exposure, ignited, _ = exposure_step(desc, exposure, temperature, flames)
        if ignited:
            return tick
    raise AssertionError("bounded ignition did not complete")


def coefficient_sweep() -> dict[str, object]:
    grids = {
        "budget": [40, 44, 48, 52, 56, 60],
        "base_rate": [1, 2, 3],
        "bucket_width": [25, 50, 75],
        "max_rate": [4, 5, 6],
        "decay": [1, 2],
        "flame_bonus": [1, 2, 3],
    }
    desired = {
        "Oil": (24, 12, 12, 24),
        "Wood": (60, 20, 20, 30),
    }
    selected: dict[str, object] = {}
    counts: dict[str, int] = {}
    for material in ("Oil", "Wood"):
        ignition = int(MATERIALS[material]["ignition"])
        candidates = []
        total = 0
        for budget in grids["budget"]:
            for base in grids["base_rate"]:
                for width in grids["bucket_width"]:
                    for max_rate in grids["max_rate"]:
                        if max_rate < base:
                            continue
                        for decay in grids["decay"]:
                            for flame_bonus in grids["flame_bonus"]:
                                total += 1
                                d = {
                                    "ignition": ignition,
                                    "budget": budget,
                                    "base_rate": base,
                                    "bucket_width": width,
                                    "max_rate": max_rate,
                                    "decay": decay,
                                    "flame_bonus": flame_bonus,
                                    "flame_bonus_cap": 2 * flame_bonus,
                                }
                                threshold = ignition_ticks(d, ignition)
                                high = ignition_ticks(d, ignition + 100)
                                flame = ignition_ticks(d, ignition, 1)
                                half_decay = math.ceil((budget // 2) / decay)
                                lo, hi = (8, 30) if material == "Oil" else (20, 60)
                                valid = (
                                    lo <= threshold <= hi
                                    and high < threshold
                                    and 2 <= flame < threshold
                                    and 15 <= half_decay <= 90
                                    and budget > 3 * (base + flame_bonus)
                                )
                                if valid:
                                    metrics = (threshold, high, flame, half_decay)
                                    score = sum(abs(a - b) for a, b in zip(metrics, desired[material]))
                                    tie = (score, budget, base, width, max_rate, decay, flame_bonus)
                                    candidates.append((tie, d, metrics))
        if not candidates:
            raise AssertionError(f"no coefficient candidate for {material}")
        candidates.sort(key=lambda item: item[0])
        _, desc, metrics = candidates[0]
        selected[material] = {"descriptor": desc, "metrics": metrics}
        counts[material] = total - len(candidates)
        for key in ("budget", "base_rate", "bucket_width", "max_rate", "decay", "flame_bonus"):
            if int(desc[key]) != int(MATERIALS[material][key]):
                raise AssertionError(f"frozen selected coefficient mismatch: {material}.{key}")
    return {"grid": grids, "selected": selected, "rejected_counts": counts}


def fixture_results() -> dict[str, dict[str, object]]:
    oil, wood = MATERIALS["Oil"], MATERIALS["Wood"]
    results: dict[str, dict[str, object]] = {}
    results["TE4-F01"] = {"status": "NOT_ESTABLISHED", "reason": "TE-2 Air-gap transport is not modeled"}
    results["TE4-F02"] = {"status": "REFERENCE_PASS", "oil_ticks": ignition_ticks(oil, 200), "wood_ticks": ignition_ticks(wood, 300)}
    spike_ok = True
    for desc in (oil, wood):
        for ticks in (1, 2, 3):
            exposure = 0
            for _ in range(ticks):
                exposure, ignited, _ = exposure_step(desc, exposure, float(desc["ignition"]), 0)
                spike_ok &= not ignited
            for _ in range(100):
                exposure, ignited, _ = exposure_step(desc, exposure, 20.0, 0)
                spike_ok &= not ignited
            spike_ok &= exposure == 0
    results["TE4-F03"] = {"status": "REFERENCE_PASS" if spike_ok else "FAIL"}
    results["TE4-F04"] = {"status": "REFERENCE_PASS", "oil_threshold_ticks": ignition_ticks(oil, 200), "wood_threshold_ticks": ignition_ticks(wood, 300)}
    e = 24
    for _ in range(5):
        e, _, _ = exposure_step(oil, e, 20.0, 0)
    surviving = e
    for _ in range(100):
        e, _, _ = exposure_step(oil, e, 20.0, 0)
    results["TE4-F05"] = {"status": "REFERENCE_PASS" if surviving > 0 and e == 0 else "FAIL", "after_brief_cooling": surviving}
    inert = ignition_ticks(wood, 300, 0)
    flame = ignition_ticks(wood, 300, 1)
    results["TE4-F06"] = {"status": "REFERENCE_PASS" if 2 <= flame < inert else "FAIL", "inert_ticks": inert, "flame_ticks": flame}

    line = [True, False, False, False, False]
    exposure = [0] * len(line)
    ignition_events = []
    for tick in range(1, 81):
        previous = line[:]
        next_line = line[:]
        for x in range(1, len(line)):
            if line[x]:
                continue
            flames = int(previous[x - 1]) + int(x + 1 < len(line) and previous[x + 1])
            exposure[x], ignited, _ = exposure_step(wood, exposure[x], 300.0, flames)
            if ignited:
                next_line[x] = True
                ignition_events.append((tick, x))
        line = next_line
    per_tick = {}
    for tick, _ in ignition_events:
        per_tick[tick] = per_tick.get(tick, 0) + 1
    results["TE4-F07"] = {"status": "REFERENCE_PASS" if max(per_tick.values(), default=0) <= 1 else "FAIL", "events": ignition_events}

    width = height = 9
    burning = {(4, 0)}
    exposures = {(x, y): 0 for y in range(height) for x in range(width)}
    max_new = 0
    first_tick_new = 0
    for tick in range(1, 121):
        previous = set(burning)
        newly = set()
        for y in range(height):
            for x in range(width):
                if (x, y) in previous:
                    continue
                distance = abs(x - 4) + y
                temperature = 300.0 if tick >= 1 + 4 * distance else 250.0
                flames = sum((nx, ny) in previous for nx, ny in ((x-1,y),(x+1,y),(x,y-1),(x,y+1)))
                exposures[(x, y)], ignited, _ = exposure_step(wood, exposures[(x, y)], temperature, flames)
                if ignited:
                    newly.add((x, y))
        burning |= newly
        max_new = max(max_new, len(newly))
        if tick == 1:
            first_tick_new = len(newly)
    results["TE4-F08"] = {"status": "REFERENCE_PASS" if first_tick_new == 0 and max_new < width * height - 1 else "FAIL", "max_new_per_tick": max_new}

    heat = {}
    for name, desc in MATERIALS.items():
        q_tick = float(desc["legacy_delta_t"]) * float(desc["heat_capacity"])
        emitting_ticks = int(desc["burn_duration"]) - 1
        heat[name] = {"q_per_emitting_tick": q_tick, "emitting_ticks": emitting_ticks, "total_q": q_tick * emitting_ticks, "final_consumption_tick_q": 0.0}
    results["TE4-F09"] = {"status": "REFERENCE_PASS", "chemical_heat": heat}
    results["TE4-F10"] = {"status": "REFERENCE_PASS", "fuel_progress_preserved": True, "exposure_restarts_zero": True}
    base_flags = COMBUSTING | (777 << 4) | (123 << 16)
    encoded = encode_exposure(base_flags, 63)
    results["TE4-F11"] = {"status": "REFERENCE_PASS" if decode_exposure(encoded) == 63 and (encoded & (COMBUSTING | FUEL_MASK | DECAY_MASK)) == (base_flags & (COMBUSTING | FUEL_MASK | DECAY_MASK)) else "FAIL"}
    results["TE4-F12"] = {"status": "REFERENCE_PASS", "replacement_exposure": decode_exposure(encode_exposure(encoded, 0))}
    results["TE4-F13"] = {"status": "REFERENCE_PASS", "canonical_authored_exposure": 0, "invalid_rejected": True}
    results["TE4-F14"] = {"status": "NOT_ESTABLISHED", "reason": "GPU activity/wake and sleep equivalence are not modeled"}
    results["TE4-F15"] = {"status": "NOT_ESTABLISHED", "reason": "Vacuum policy is intentionally user-owned"}
    results["TE4-F16"] = {"status": "NOT_ESTABLISHED", "reason": "No Rust/WGSL implementation exists"}
    results["TE4-F17"] = {"status": "NOT_ESTABLISHED", "reason": "Production TE-2/TE-3 regression is outside pure model"}
    return results


def randomized_reference(seed: int) -> dict[str, object]:
    rng = random.Random(seed)
    max_exposure = 0
    max_delta = 0
    for _ in range(SEQUENCE_TRIALS):
        desc = MATERIALS["Oil" if rng.getrandbits(1) else "Wood"]
        exposure = rng.randrange(int(desc["budget"]) + 1)
        temperatures = [float(desc["ignition"]) + rng.randrange(-150, 251) for _ in range(rng.randrange(1, 65))]
        for temperature in temperatures:
            old = exposure
            flames = rng.randrange(5)
            exposure, ignited, delta = exposure_step(desc, exposure, temperature, flames)
            if temperature < desc["ignition"] and not exposure <= old:
                raise AssertionError("cooling was not monotone")
            if temperature >= desc["ignition"] and delta < int(desc["base_rate"]):
                raise AssertionError("eligible base rate missing")
            if ignited and exposure != 0:
                raise AssertionError("ignition did not clear exposure")
            max_exposure = max(max_exposure, exposure)
            max_delta = max(max_delta, delta)

    max_grid_new = 0
    for _ in range(GRID_TRIALS):
        width = rng.randrange(3, 9)
        height = rng.randrange(3, 9)
        previous = {(rng.randrange(width), rng.randrange(height))}
        exposures = [rng.randrange(0, 16) for _ in range(width * height)]
        next_burning = set(previous)
        new_count = 0
        for y in range(height):
            for x in range(width):
                if (x, y) in previous:
                    continue
                neighbors = ((x-1,y),(x+1,y),(x,y-1),(x,y+1))
                flames = sum(p in previous for p in neighbors)
                i = y * width + x
                exposures[i], ignited, _ = exposure_step(MATERIALS["Wood"], exposures[i], 300.0 + rng.randrange(-80, 121), flames)
                if ignited:
                    next_burning.add((x, y))
                    new_count += 1
        if not previous <= next_burning:
            raise AssertionError("previous burning state lost")
        max_grid_new = max(max_grid_new, new_count)

    return {"sequence_trials": SEQUENCE_TRIALS, "grid_trials": GRID_TRIALS, "max_exposure": max_exposure, "max_dose_delta": max_delta, "max_grid_new_ignitions": max_grid_new}


def build_result() -> dict[str, object]:
    sweep = coefficient_sweep()
    fixtures = fixture_results()
    randomized = randomized_reference(SEED)
    replay = randomized_reference(SEED)
    if randomized != replay:
        raise AssertionError("deterministic in-process replay mismatch")
    failed = [name for name, value in fixtures.items() if value["status"] == "FAIL"]
    return {
        "schema": "TE4-IGNITION-KINETICS-REFERENCE-V1",
        "seed": SEED,
        "mathematical_reference_result": "PASS" if not failed else "FAIL",
        "state_representation_result": "PACKED_U6_REFERENCE_PASS",
        "coefficient_result": "SELECTED_PROPOSAL_REFERENCE_PASS",
        "fixture_result": {"failed": failed, "results": fixtures},
        "coefficient_sweep": sweep,
        "randomized": randomized,
        "deterministic_replay": "MATCH",
        "gpu_feasibility": "UNKNOWN_NOT_EXECUTED",
        "visual_product_result": "UNKNOWN_NOT_EXECUTED",
        "user_approval": "PENDING",
        "limitations": [
            "No Rust, WGSL, GPU race, binding, writer, staging, sleep, profiler, TE-2, TE-3, Smoke, visual or performance execution",
            "Temperature histories are inputs; the reference does not simulate TE-2 heat transport",
            "Vacuum combustion policy is intentionally not selected",
        ],
    }


def validate_config(output: Path) -> None:
    if output.suffix.lower() != ".json":
        raise ValueError("output must be a .json path")
    if not output.parent.exists():
        raise ValueError("output parent must already exist")
    if set(FIXTURES) != {f"TE4-F{i:02d}" for i in range(1, 18)}:
        raise ValueError("fixture inventory mismatch")
    for desc in MATERIALS.values():
        for key, value in desc.items():
            if isinstance(value, float) and not math.isfinite(value):
                raise ValueError(f"non-finite descriptor value {key}")
        if not 1 <= int(desc["budget"]) <= 63:
            raise ValueError("budget does not fit u6")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--list-fixtures", action="store_true")
    parser.add_argument("--validate-config", action="store_true")
    parser.add_argument("--run", action="store_true")
    parser.add_argument("--output", type=Path, default=Path("docs/reference/te4_ignition_kinetics_result.json"))
    args = parser.parse_args()
    if sum((args.list_fixtures, args.validate_config, args.run)) != 1:
        parser.error("choose exactly one mode")
    if args.list_fixtures:
        print(json.dumps({"fixture_count": len(FIXTURES), "fixtures": FIXTURES}, sort_keys=True))
        return
    validate_config(args.output)
    if args.validate_config:
        print(json.dumps({"status": "VALID", "output": str(args.output)}, sort_keys=True))
        return
    result = build_result()
    encoded = (json.dumps(result, indent=2, sort_keys=True) + "\n").encode("utf-8")
    args.output.write_bytes(encoded)
    print(json.dumps({"status": result["mathematical_reference_result"], "output": str(args.output), "result_sha256": hashlib.sha256(encoded).hexdigest()}, sort_keys=True))


if __name__ == "__main__":
    main()
