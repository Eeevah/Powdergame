#!/usr/bin/env python3
"""Independent exhaustive frontier oracle for the TE-4D v3 design.

This preflight-only generator intentionally imports no TE-4 reference model.
It scans every unlit coordinate from the previous tick's burning set and emits
the complete sorted ignition frontier.  The evidence program consumes only the
frozen JSON output; it never imports or executes this file.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


WOOD = {
    "budget": 60,
    "base_rate": 1,
    "bucket_width_C": 50,
    "max_rate": 5,
    "cooling_decay": 1,
    "flame_bonus": 2,
    "flame_bonus_cap": 4,
    "ignition_threshold_C": 300,
}

CASES = (
    {"name": "TE4-F07-LINE", "width": 5, "height": 1, "initial_burning": ((0, 0),), "initial_exposure": 0, "horizon": 80},
    {"name": "TE4-F08-GRID", "width": 9, "height": 9, "initial_burning": ((4, 0),), "initial_exposure": 0, "horizon": 173},
    {"name": "TE4-F08-NEAR-BUDGET-56", "width": 9, "height": 9, "initial_burning": ((4, 0),), "initial_exposure": 56, "horizon": 20},
    {"name": "TE4-F08-NEAR-BUDGET-59", "width": 9, "height": 9, "initial_burning": ((4, 0),), "initial_exposure": 59, "horizon": 20},
    {"name": "TE4-F08-SYMMETRIC-TIE", "width": 5, "height": 5, "initial_burning": ((2, 2),), "initial_exposure": 0, "horizon": 100},
)


def canonical_bytes(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")


def exhaustive_frontiers(case: dict[str, object]) -> dict[str, object]:
    width = int(case["width"])
    height = int(case["height"])
    burning = {tuple(p) for p in case["initial_burning"]}
    exposure = {
        (x, y): int(case["initial_exposure"])
        for y in range(height)
        for x in range(width)
        if (x, y) not in burning
    }
    events: list[dict[str, object]] = []
    completion_tick: int | None = None

    for tick in range(1, int(case["horizon"]) + 1):
        snapshot_burning = frozenset(burning)
        frontier: list[tuple[int, int]] = []
        next_exposure = dict(exposure)
        for y in range(height):
            for x in range(width):
                position = (x, y)
                if position in snapshot_burning:
                    continue
                orthogonal = ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1))
                prior_flames = sum(neighbour in snapshot_burning for neighbour in orthogonal)
                if prior_flames:
                    thermal_rate = WOOD["base_rate"]
                    flame_rate = min(WOOD["flame_bonus_cap"], prior_flames * WOOD["flame_bonus"])
                    updated = min(WOOD["budget"], exposure[position] + thermal_rate + flame_rate)
                else:
                    updated = max(0, exposure[position] - WOOD["cooling_decay"])
                next_exposure[position] = updated
                if updated == WOOD["budget"]:
                    frontier.append(position)

        if frontier:
            frontier.sort()
            events.append({"tick": tick, "cells": [[x, y] for x, y in frontier]})
            for position in frontier:
                burning.add(position)
                next_exposure.pop(position, None)
        exposure = next_exposure
        if len(burning) == width * height:
            completion_tick = tick
            break

    digest = hashlib.sha256(canonical_bytes(events)).hexdigest()
    return {
        "name": case["name"],
        "geometry": {"width": width, "height": height},
        "initial_burning": [list(p) for p in case["initial_burning"]],
        "initial_exposure": case["initial_exposure"],
        "horizon": case["horizon"],
        "visibility": "PREVIOUS_SNAPSHOT_ORTHOGONAL_ONLY",
        "events": events,
        "event_count": sum(len(event["cells"]) for event in events),
        "frontier_tick_count": len(events),
        "completion_tick": completion_tick,
        "event_digest_sha256": digest,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    results = [exhaustive_frontiers(case) for case in CASES]
    payload = {
        "schema": "powdergame.te4.ignition.v3.frontier-oracle/1",
        "generator_role": "PREFLIGHT_ONLY_INDEPENDENT_EXHAUSTIVE_ORACLE",
        "coefficient_identity": {"Wood": WOOD},
        "cases": results,
    }
    args.output.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    for result in results:
        print(f"{result['name']}: events={result['event_count']} frontiers={result['frontier_tick_count']} completion={result['completion_tick']} digest={result['event_digest_sha256']}")
        for event in result["events"]:
            print(f"  tick {event['tick']:>3}: {event['cells']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
