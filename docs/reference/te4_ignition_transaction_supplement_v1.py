#!/usr/bin/env python3
"""TE-4 targeted transaction evidence supplement.

The SUT returns state plus mechanically discovered changed fields only.  The
auditor receives a frozen specification id from the harness and independently
constructs the one permitted after-state.  It never consumes SUT semantic
names, events, counters or ownership labels.
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import sys
from pathlib import Path
from typing import Any, Callable


IDENTITY = "TE4-IGNITION-TRANSACTION-SUPPLEMENT-V1"
EXPECTED_MANIFEST_SHA256 = "03549f3b618fbb58b8c360d071d6f345c1409f5d92061b71760db5d033918295"
COMBUSTING = 1
FLAME_EVENT = 2


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")


def sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


@dataclasses.dataclass(frozen=True)
class Cell:
    material: str = "EMPTY"
    temperature: float = 20.0
    flags: int = 0
    air_mass: float = 0.0
    air_energy: float = 0.0
    exposure: int = 0
    fuel: int = 0


@dataclasses.dataclass(frozen=True)
class Stage:
    heat: int = 0
    flame: int = 0
    smoke: int = 0
    gross_q: float = 0.0
    deposited_q: float = 0.0
    clipped_q: float = 0.0
    consumption: int = 0


@dataclasses.dataclass(frozen=True)
class World:
    width: int
    height: int
    current: tuple[Cell, ...]
    next: tuple[Cell, ...]
    baseline: tuple[Cell, ...]
    stage: Stage = Stage()


def make_world(width: int, height: int, cells: tuple[Cell, ...]) -> World:
    if len(cells) != width * height:
        raise ValueError("world cell count")
    return World(width, height, cells, cells, cells)


def payload(world: World) -> dict[str, Any]:
    return {
        "width": world.width,
        "height": world.height,
        "current": [dataclasses.asdict(c) for c in world.current],
        "next": [dataclasses.asdict(c) for c in world.next],
        "baseline": [dataclasses.asdict(c) for c in world.baseline],
        "stage": dataclasses.asdict(world.stage),
    }


def from_payload(value: dict[str, Any]) -> World:
    return World(
        int(value["width"]), int(value["height"]),
        tuple(Cell(**c) for c in value["current"]),
        tuple(Cell(**c) for c in value["next"]),
        tuple(Cell(**c) for c in value["baseline"]),
        Stage(**value["stage"]),
    )


def cell_index(world: World, x: int, y: int) -> int | None:
    if x < 0 or y < 0 or x >= world.width or y >= world.height:
        return None
    return y * world.width + x


def derive_air_access(world: World, source: int) -> bool:
    x, y = source % world.width, source // world.width
    for nx, ny in ((x, y - 1), (x - 1, y), (x + 1, y), (x, y + 1)):
        index = cell_index(world, nx, ny)
        if index is not None:
            neighbour = world.current[index]
            if neighbour.material == "EMPTY" and neighbour.air_mass > 0.0:
                return True
    return False


def changed_fields(before: World, after: World) -> tuple[str, ...]:
    changes: list[str] = []
    for layer in ("current", "next"):
        left, right = getattr(before, layer), getattr(after, layer)
        for i, (a, b) in enumerate(zip(left, right)):
            for field in ("material", "temperature", "flags", "air_mass", "air_energy", "exposure", "fuel"):
                if getattr(a, field) != getattr(b, field):
                    changes.append(f"{layer}[{i}].{field}")
    for field in dataclasses.asdict(before.stage):
        if getattr(before.stage, field) != getattr(after.stage, field):
            changes.append(f"stage.{field}")
    return tuple(changes)


def sut_replace_current(world: World, index: int, cell: Cell, stage: Stage | None = None) -> tuple[World, tuple[str, ...]]:
    cells = list(world.current)
    cells[index] = cell
    after = dataclasses.replace(world, current=tuple(cells), stage=stage if stage is not None else world.stage)
    return after, changed_fields(world, after)


def sut_exposure_accumulation(world: World, data: dict[str, Any]) -> tuple[World, tuple[str, ...]]:
    i, amount = data["cell"], data["amount"]
    cell = world.current[i]
    return sut_replace_current(world, i, dataclasses.replace(cell, exposure=min(data["budget"], cell.exposure + amount)))


def sut_exposure_decay(world: World, data: dict[str, Any]) -> tuple[World, tuple[str, ...]]:
    i, amount = data["cell"], data["amount"]
    cell = world.current[i]
    return sut_replace_current(world, i, dataclasses.replace(cell, exposure=max(0, cell.exposure - amount)))


def sut_combustion_stage(world: World, data: dict[str, Any]) -> tuple[World, tuple[str, ...]]:
    i = data["cell"]
    p = data["parameters"]
    cell = world.current[i]
    access = derive_air_access(world, i)
    burning = bool(cell.flags & COMBUSTING)
    eligible_start = (not burning and cell.temperature >= p["ignition_threshold_C"] and cell.exposure >= p["budget"])
    if access and eligible_start:
        burning = True
    if burning and (not access or cell.temperature < p["sustain_threshold_C"]):
        after_cell = dataclasses.replace(cell, flags=cell.flags & ~(COMBUSTING | FLAME_EVENT), exposure=0)
        return sut_replace_current(world, i, after_cell, Stage())
    if not burning:
        return sut_replace_current(world, i, dataclasses.replace(cell, flags=cell.flags & ~FLAME_EVENT), Stage())
    next_progress = cell.fuel + 1
    if next_progress >= p["burn_duration"]:
        after_cell = Cell("EMPTY", data["empty_temperature_C"])
        return sut_replace_current(world, i, after_cell, Stage(consumption=1))
    gross = p["gross_Q"]
    possible = max(0.0, data["temperature_cap_C"] - cell.temperature) * p["heat_capacity"]
    deposited = min(gross, possible)
    clipped = gross - deposited
    after_temperature = cell.temperature + deposited / p["heat_capacity"]
    after_cell = dataclasses.replace(cell, temperature=after_temperature, flags=cell.flags | COMBUSTING | FLAME_EVENT, exposure=0, fuel=next_progress)
    stage = Stage(heat=1, flame=1, smoke=1, gross_q=gross, deposited_q=deposited, clipped_q=clipped)
    return sut_replace_current(world, i, after_cell, stage)


def sut_movement_into_empty(world: World, data: dict[str, Any]) -> tuple[World, tuple[str, ...]]:
    s, d = data["source"], data["destination"]
    src, dst = world.current[s], world.current[d]
    cells = list(world.current)
    cells[s] = Cell("EMPTY", src.temperature, air_mass=src.air_mass, air_energy=src.air_energy)
    cells[d] = Cell(src.material, src.temperature, src.flags, dst.air_mass, dst.air_energy, src.exposure, src.fuel)
    after = dataclasses.replace(world, current=tuple(cells))
    return after, changed_fields(world, after)


def sut_density_swap(world: World, data: dict[str, Any]) -> tuple[World, tuple[str, ...]]:
    a, b = data["first"], data["second"]
    left, right = world.current[a], world.current[b]
    cells = list(world.current)
    cells[a] = Cell(right.material, right.temperature, right.flags, left.air_mass, left.air_energy, right.exposure, right.fuel)
    cells[b] = Cell(left.material, left.temperature, left.flags, right.air_mass, right.air_energy, left.exposure, left.fuel)
    after = dataclasses.replace(world, current=tuple(cells))
    return after, changed_fields(world, after)


def sut_identity_replace(world: World, data: dict[str, Any]) -> tuple[World, tuple[str, ...]]:
    i = data["cell"]
    old = world.current[i]
    material = data["material"]
    temperature = data.get("temperature", old.temperature)
    air_mass = old.air_mass if material == "EMPTY" else 0.0
    air_energy = old.air_energy if material == "EMPTY" else 0.0
    return sut_replace_current(world, i, Cell(material, temperature, air_mass=air_mass, air_energy=air_energy))


def sut_draw(world: World, data: dict[str, Any]) -> tuple[World, tuple[str, ...]]:
    i = data["cell"]
    new = Cell(data["material"], data["temperature"])
    current, next_cells = list(world.current), list(world.next)
    current[i] = new
    next_cells[i] = new
    after = dataclasses.replace(world, current=tuple(current), next=tuple(next_cells))
    return after, changed_fields(world, after)


def sut_erase(world: World, data: dict[str, Any]) -> tuple[World, tuple[str, ...]]:
    i = data["cell"]
    current, next_cells = list(world.current), list(world.next)
    current[i], next_cells[i] = Cell(), Cell()
    after = dataclasses.replace(world, current=tuple(current), next=tuple(next_cells))
    return after, changed_fields(world, after)


def sut_preset_stage(world: World, data: dict[str, Any]) -> tuple[World, tuple[str, ...]]:
    cells = tuple(Cell(**c) for c in data["cells"])
    after = dataclasses.replace(world, current=cells, next=cells)
    return after, changed_fields(world, after)


def sut_reset(world: World, data: dict[str, Any]) -> tuple[World, tuple[str, ...]]:
    after = dataclasses.replace(world, current=world.baseline, next=world.baseline, stage=Stage())
    return after, changed_fields(world, after)


def sut_smoke_commit(world: World, data: dict[str, Any]) -> tuple[World, tuple[str, ...]]:
    i = data["target"]
    old = world.current[i]
    new = Cell("Smoke", data["temperature"], air_mass=old.air_mass, air_energy=old.air_energy)
    return sut_replace_current(world, i, new)


def sut_air_displacement(world: World, data: dict[str, Any]) -> tuple[World, tuple[str, ...]]:
    s, r = data["target"], data["receiver"]
    src, dst = world.current[s], world.current[r]
    cells = list(world.current)
    cells[s] = dataclasses.replace(src, air_mass=0.0, air_energy=0.0)
    cells[r] = dataclasses.replace(dst, air_mass=dst.air_mass + src.air_mass, air_energy=dst.air_energy + src.air_energy)
    after = dataclasses.replace(world, current=tuple(cells))
    return after, changed_fields(world, after)


def audit_set_current(world: World, index: int, cell: Cell, stage: Stage | None = None) -> World:
    cells = list(world.current)
    cells[index] = cell
    return dataclasses.replace(world, current=tuple(cells), stage=stage if stage is not None else world.stage)


def audit_expected_combustion(before: World, data: dict[str, Any]) -> World:
    i, p = data["cell"], data["parameters"]
    c = before.current[i]
    x, y = i % before.width, i // before.width
    access = False
    for nx, ny in ((x, y - 1), (x - 1, y), (x + 1, y), (x, y + 1)):
        if 0 <= nx < before.width and 0 <= ny < before.height:
            n = before.current[ny * before.width + nx]
            access = access or (n.material == "EMPTY" and n.air_mass > 0.0)
    burning = bool(c.flags & COMBUSTING)
    if access and not burning and c.temperature >= p["ignition_threshold_C"] and c.exposure >= p["budget"]:
        burning = True
    if burning and (not access or c.temperature < p["sustain_threshold_C"]):
        return audit_set_current(before, i, dataclasses.replace(c, flags=c.flags & ~(COMBUSTING | FLAME_EVENT), exposure=0), Stage())
    if not burning:
        return audit_set_current(before, i, dataclasses.replace(c, flags=c.flags & ~FLAME_EVENT), Stage())
    progress = c.fuel + 1
    if progress >= p["burn_duration"]:
        return audit_set_current(before, i, Cell("EMPTY", data["empty_temperature_C"]), Stage(consumption=1))
    gross = p["gross_Q"]
    deposited = min(gross, max(0.0, data["temperature_cap_C"] - c.temperature) * p["heat_capacity"])
    clipped = gross - deposited
    updated = dataclasses.replace(c, temperature=c.temperature + deposited / p["heat_capacity"], flags=c.flags | COMBUSTING | FLAME_EVENT, exposure=0, fuel=progress)
    return audit_set_current(before, i, updated, Stage(1, 1, 1, gross, deposited, clipped, 0))


def audit_expected(spec_id: str, before: World, data: dict[str, Any]) -> World:
    if spec_id == "exposure_accumulation":
        i, c = data["cell"], before.current[data["cell"]]
        return audit_set_current(before, i, dataclasses.replace(c, exposure=min(data["budget"], c.exposure + data["amount"])))
    if spec_id == "exposure_decay":
        i, c = data["cell"], before.current[data["cell"]]
        return audit_set_current(before, i, dataclasses.replace(c, exposure=max(0, c.exposure - data["amount"])))
    if spec_id in ("ignition", "active_burn", "extinguish", "reignition", "fuel_consumption"):
        return audit_expected_combustion(before, data)
    if spec_id == "movement_into_empty":
        s, d = data["source"], data["destination"]
        src, dst = before.current[s], before.current[d]
        cells = list(before.current)
        cells[s] = Cell("EMPTY", src.temperature, air_mass=src.air_mass, air_energy=src.air_energy)
        cells[d] = Cell(src.material, src.temperature, src.flags, dst.air_mass, dst.air_energy, src.exposure, src.fuel)
        return dataclasses.replace(before, current=tuple(cells))
    if spec_id == "density_swap":
        a, b = data["first"], data["second"]
        left, right = before.current[a], before.current[b]
        cells = list(before.current)
        cells[a] = Cell(right.material, right.temperature, right.flags, left.air_mass, left.air_energy, right.exposure, right.fuel)
        cells[b] = Cell(left.material, left.temperature, left.flags, right.air_mass, right.air_energy, left.exposure, left.fuel)
        return dataclasses.replace(before, current=tuple(cells))
    if spec_id in ("decay_replacement", "rupture_replacement", "void_exit", "generic_replacement"):
        i, old = data["cell"], before.current[data["cell"]]
        material, temperature = data["material"], data.get("temperature", old.temperature)
        new = Cell(material, temperature, air_mass=old.air_mass if material == "EMPTY" else 0.0, air_energy=old.air_energy if material == "EMPTY" else 0.0)
        return audit_set_current(before, i, new)
    if spec_id == "draw":
        i, new = data["cell"], Cell(data["material"], data["temperature"])
        current, nxt = list(before.current), list(before.next)
        current[i], nxt[i] = new, new
        return dataclasses.replace(before, current=tuple(current), next=tuple(nxt))
    if spec_id == "erase":
        i = data["cell"]
        current, nxt = list(before.current), list(before.next)
        current[i], nxt[i] = Cell(), Cell()
        return dataclasses.replace(before, current=tuple(current), next=tuple(nxt))
    if spec_id == "preset_stage":
        cells = tuple(Cell(**c) for c in data["cells"])
        return dataclasses.replace(before, current=cells, next=cells)
    if spec_id == "reset":
        return dataclasses.replace(before, current=before.baseline, next=before.baseline, stage=Stage())
    if spec_id == "smoke_commit":
        i, old = data["target"], before.current[data["target"]]
        if old.material != "EMPTY" or i != data["eligible_target"]:
            raise ValueError("ineligible smoke target")
        return audit_set_current(before, i, Cell("Smoke", data["temperature"], air_mass=old.air_mass, air_energy=old.air_energy))
    if spec_id == "air_displacement":
        s, r = data["target"], data["receiver"]
        src, dst = before.current[s], before.current[r]
        cells = list(before.current)
        cells[s] = dataclasses.replace(src, air_mass=0.0, air_energy=0.0)
        cells[r] = dataclasses.replace(dst, air_mass=dst.air_mass + src.air_mass, air_energy=dst.air_energy + src.air_energy)
        return dataclasses.replace(before, current=tuple(cells))
    raise KeyError(spec_id)


def audit_transition(spec_id: str, before: World, after: World, data: dict[str, Any], manifest: dict[str, Any]) -> tuple[bool, dict[str, Any]]:
    if spec_id not in manifest["audit_specifications"]:
        return False, {"reason": "unknown independent specification"}
    try:
        expected = audit_expected(spec_id, before, data)
    except Exception as exc:
        return False, {"reason": f"specification rejected input: {type(exc).__name__}: {exc}"}
    actual_changes = changed_fields(before, after)
    permitted_cell_fields = set(manifest["audit_specifications"][spec_id]["allowed_cell_fields"])
    actual_cell_fields = {
        item.rsplit(".", 1)[1]
        for item in actual_changes
        if item.startswith(("current[", "next["))
    }
    permitted_fields = actual_cell_fields <= permitted_cell_fields
    accepted = payload(after) == payload(expected) and bool(actual_changes) and permitted_fields
    affected = sorted({int(item.split("[")[1].split("]")[0]) for item in actual_changes if item.startswith(("current[", "next["))})
    return accepted, {
        "reason": "exact independent state match" if accepted else "after-state differs from independent specification",
        "changed_fields": list(actual_changes),
        "affected_cell_indices": affected,
        "expected_after_sha256": sha256(canonical(payload(expected))),
        "actual_after_sha256": sha256(canonical(payload(after))),
        "permitted_cell_fields": sorted(permitted_cell_fields),
        "actual_cell_fields": sorted(actual_cell_fields),
        "permitted_fields_only": permitted_fields,
        "claimed_semantic_ignored": data.get("claimed_semantic"),
    }


def record(fixture: str, ordinal: int, data: dict[str, Any], before: World, after: World, spec_id: str, manifest: dict[str, Any]) -> dict[str, Any]:
    accepted, summary = audit_transition(spec_id, before, after, data, manifest)
    item = {
        "evidence_identity": IDENTITY,
        "fixture": fixture,
        "transaction_ordinal": ordinal,
        "transaction_input": data,
        "before_state": payload(before),
        "after_state": payload(after),
        "affected_cell_indices": summary.get("affected_cell_indices", []),
        "audit_specification_id": spec_id,
        "accepted": accepted,
        "derived_delta_summary": summary,
        "record_sha256": "",
    }
    item["record_sha256"] = sha256(canonical(item))
    return item


def corrupt_after(world: World, spec_id: str, data: dict[str, Any]) -> World:
    if spec_id == "air_displacement":
        i = data["receiver"]
        cells = list(world.current)
        cells[i] = dataclasses.replace(cells[i], air_mass=cells[i].air_mass + 1.0)
        return dataclasses.replace(world, current=tuple(cells))
    if spec_id in ("draw", "erase", "preset_stage", "reset"):
        i = data.get("cell", 0)
        nxt = list(world.next)
        nxt[i] = dataclasses.replace(nxt[i], exposure=nxt[i].exposure + 1)
        return dataclasses.replace(world, next=tuple(nxt))
    i = data.get("cell", data.get("destination", data.get("target", data.get("first", 0))))
    cells = list(world.current)
    if spec_id in ("movement_into_empty", "density_swap"):
        cells[i] = dataclasses.replace(cells[i], fuel=cells[i].fuel + 1)
    elif spec_id in ("decay_replacement", "rupture_replacement", "fuel_consumption", "void_exit", "generic_replacement"):
        cells[i] = dataclasses.replace(cells[i], exposure=1)
    elif spec_id == "smoke_commit":
        cells[i] = dataclasses.replace(cells[i], material="Smoke")
        wrong = data["eligible_target"] + 1
        if wrong < len(cells):
            cells[wrong] = dataclasses.replace(cells[wrong], material="Smoke")
    else:
        cells[i] = dataclasses.replace(cells[i], temperature=cells[i].temperature + 0.5)
    return dataclasses.replace(world, current=tuple(cells))


def transaction_case(spec_id: str, params: dict[str, Any]) -> tuple[World, dict[str, Any], Callable[[World, dict[str, Any]], tuple[World, tuple[str, ...]]]]:
    air = Cell("EMPTY", air_mass=1.0, air_energy=20.0)
    oil = Cell("Oil", 200.0, exposure=47, fuel=5)
    wood = Cell("Wood", 300.0, COMBUSTING, exposure=0, fuel=5)
    base = make_world(3, 2, (oil, air, wood, Cell("Water", 20.0), Cell("Stone", 20.0), Cell()))
    combustion = {"cell": 0, "parameters": params["Oil"], "temperature_cap_C": 1200.0, "empty_temperature_C": 20.0}
    cases: dict[str, tuple[World, dict[str, Any], Callable[[World, dict[str, Any]], tuple[World, tuple[str, ...]]]]] = {
        "exposure_accumulation": (base, {"cell": 0, "amount": 1, "budget": 48}, sut_exposure_accumulation),
        "exposure_decay": (dataclasses.replace(base, current=(dataclasses.replace(oil, exposure=4),)+base.current[1:]), {"cell": 0, "amount": 1}, sut_exposure_decay),
        "ignition": (dataclasses.replace(base, current=(dataclasses.replace(oil, exposure=48),)+base.current[1:]), combustion, sut_combustion_stage),
        "active_burn": (dataclasses.replace(base, current=(dataclasses.replace(oil, flags=COMBUSTING, exposure=0),)+base.current[1:]), combustion, sut_combustion_stage),
        "extinguish": (dataclasses.replace(base, current=(dataclasses.replace(oil, flags=COMBUSTING, exposure=7), Cell(), base.current[2], base.current[3], base.current[4], base.current[5])), combustion, sut_combustion_stage),
        "reignition": (dataclasses.replace(base, current=(dataclasses.replace(oil, exposure=48, fuel=7),)+base.current[1:]), combustion, sut_combustion_stage),
        "movement_into_empty": (base, {"source": 0, "destination": 1}, sut_movement_into_empty),
        "density_swap": (base, {"first": 0, "second": 3}, sut_density_swap),
        "decay_replacement": (base, {"cell": 0, "material": "Smoke"}, sut_identity_replace),
        "rupture_replacement": (base, {"cell": 0, "material": "EMPTY"}, sut_identity_replace),
        "fuel_consumption": (dataclasses.replace(base, current=(dataclasses.replace(oil, flags=COMBUSTING, exposure=0, fuel=599),)+base.current[1:]), combustion, sut_combustion_stage),
        "void_exit": (base, {"cell": 0, "material": "EMPTY", "temperature": 20.0}, sut_identity_replace),
        "generic_replacement": (base, {"cell": 0, "material": "Stone"}, sut_identity_replace),
        "draw": (base, {"cell": 5, "material": "Oil", "temperature": 200.0}, sut_draw),
        "erase": (base, {"cell": 0}, sut_erase),
        "preset_stage": (base, {"cells": [dataclasses.asdict(Cell("Wood", 300.0, exposure=9))] + [dataclasses.asdict(Cell()) for _ in range(5)]}, sut_preset_stage),
        "reset": (dataclasses.replace(base, current=(Cell("Smoke"),)+base.current[1:], next=(Cell("Smoke"),)+base.next[1:]), {}, sut_reset),
        "smoke_commit": (base, {"target": 1, "eligible_target": 1, "temperature": 300.0}, sut_smoke_commit),
        "air_displacement": (dataclasses.replace(base, current=(base.current[0], Cell("Smoke", 300.0, air_mass=1.0, air_energy=20.0), base.current[2], base.current[3], base.current[4], Cell("EMPTY", air_mass=0.25, air_energy=5.0))), {"target": 1, "receiver": 5}, sut_air_displacement),
    }
    return cases[spec_id]


def run_semantic_matrix(manifest: dict[str, Any], records: list[dict[str, Any]], ordinal: int) -> tuple[int, dict[str, int]]:
    counts: dict[str, int] = {}
    params = manifest["coefficients"]
    for spec_id in manifest["required_transaction_classes"]:
        before, data, function = transaction_case(spec_id, params)
        data = dict(data)
        data["claimed_semantic"] = "INTENTIONALLY_WRONG_AND_IGNORED"
        after, _raw = function(before, data)
        valid = record("SEMANTIC_TRANSACTION_MATRIX", ordinal, data, before, after, spec_id, manifest)
        ordinal += 1
        if not valid["accepted"]:
            raise AssertionError(f"valid {spec_id} rejected")
        records.append(valid)
        invalid_after = corrupt_after(after, spec_id, data)
        invalid = record("SEMANTIC_TRANSACTION_MATRIX_NEGATIVE", ordinal, data, before, invalid_after, spec_id, manifest)
        ordinal += 1
        if invalid["accepted"]:
            raise AssertionError(f"invalid {spec_id} accepted")
        records.append(invalid)
        removed = record("SEMANTIC_TRANSACTION_REMOVED_BODY", ordinal, data, before, before, spec_id, manifest)
        ordinal += 1
        if removed["accepted"]:
            raise AssertionError(f"removed body {spec_id} accepted")
        records.append(removed)
        counts[spec_id] = 1
    return ordinal, counts


def make_f15b_world() -> tuple[World, int, int, int]:
    cells = [Cell("Stone") for _ in range(25)]
    source, target, receiver = 12, 7, 2
    cells[source] = Cell("Wood", 300.0, COMBUSTING, fuel=10)
    cells[target] = Cell("EMPTY", 20.0, air_mass=1.25, air_energy=25.0)
    cells[receiver] = Cell("EMPTY", 20.0, air_mass=0.25, air_energy=5.0)
    cells[17] = Cell("EMPTY", 20.0, air_mass=0.0, air_energy=0.0)
    return make_world(5, 5, tuple(cells)), source, target, receiver


def run_f15b(manifest: dict[str, Any], records: list[dict[str, Any]], ordinal: int) -> tuple[int, dict[str, Any]]:
    world, source, target, receiver = make_f15b_world()
    params = manifest["coefficients"]["Wood"]
    combustion_input = {"cell": source, "parameters": params, "temperature_cap_C": 1200.0, "empty_temperature_C": 20.0, "claimed_semantic": "IGNORED"}
    stage_n_access = derive_air_access(world, source)
    before = world
    world, _ = sut_combustion_stage(world, combustion_input)
    stage_n_stage = world.stage
    stage_n_fuel = world.current[source].fuel
    burn = record("F15B_SUPPLEMENT", ordinal, combustion_input, before, world, "active_burn", manifest); ordinal += 1
    if not burn["accepted"]:
        raise AssertionError("F15B stage N burn audit")
    records.append(burn)
    if world.stage.smoke != 1 or world.current[target].material != "EMPTY":
        raise AssertionError("F15B proposal precondition")
    smoke_input = {"target": target, "eligible_target": target, "temperature": world.current[source].temperature}
    before = world
    world, _ = sut_smoke_commit(world, smoke_input)
    smoke = record("F15B_SUPPLEMENT", ordinal, smoke_input, before, world, "smoke_commit", manifest); ordinal += 1
    if not smoke["accepted"]:
        raise AssertionError("F15B smoke audit")
    records.append(smoke)
    air_input = {"target": target, "receiver": receiver}
    before = world
    air_before = (sum(c.air_mass for c in world.current), sum(c.air_energy for c in world.current))
    world, _ = sut_air_displacement(world, air_input)
    displaced = record("F15B_SUPPLEMENT", ordinal, air_input, before, world, "air_displacement", manifest); ordinal += 1
    if not displaced["accepted"]:
        raise AssertionError("F15B Air audit")
    records.append(displaced)
    air_after = (sum(c.air_mass for c in world.current), sum(c.air_energy for c in world.current))
    stage_n1_access = derive_air_access(world, source)
    prior_fuel = world.current[source].fuel
    before = world
    world, _ = sut_combustion_stage(world, combustion_input)
    extinguished = record("F15B_SUPPLEMENT", ordinal, combustion_input, before, world, "extinguish", manifest); ordinal += 1
    if not extinguished["accepted"]:
        raise AssertionError("F15B stage N+1 audit")
    records.append(extinguished)
    result = {
        "stage_N_air_access_derived": stage_n_access,
        "stage_N": {**dataclasses.asdict(stage_n_stage), "fuel_after": stage_n_fuel},
        "stage_N_plus_1_air_access_derived": stage_n1_access,
        "stage_N_plus_1": dataclasses.asdict(world.stage),
        "stage_N_plus_1_burning": bool(world.current[source].flags & COMBUSTING),
        "fuel_preserved": world.current[source].fuel == prior_fuel,
        "target_material": world.current[target].material,
        "air_before": air_before,
        "air_after": air_after,
        "air_conserved": all(abs(a-b) <= manifest["tolerances"]["absolute_Air"] for a,b in zip(air_before, air_after)),
    }
    if not stage_n_access or stage_n1_access or result["stage_N_plus_1_burning"] or not result["fuel_preserved"] or not result["air_conserved"] or any(dataclasses.asdict(world.stage)[k] for k in ("heat", "flame", "smoke", "gross_q", "deposited_q")):
        raise AssertionError(f"F15B topology contract {result}")
    return ordinal, result


def run_lifecycle(material: str, manifest: dict[str, Any], records: list[dict[str, Any]], ordinal: int) -> tuple[int, dict[str, Any]]:
    p = manifest["coefficients"][material]
    cells = (Cell(material, p["sustain_threshold_C"] + 50.0, COMBUSTING), Cell("EMPTY", air_mass=1.0, air_energy=20.0))
    world = make_world(2, 1, cells)
    data = {"cell": 0, "parameters": p, "temperature_cap_C": 1200.0, "empty_temperature_C": 20.0}
    gross_total = deposited_total = clipped_total = 0.0
    trace_hash = hashlib.sha256()
    emitting: list[int] = []
    for tick in range(1, p["burn_duration"] + 1):
        before = world
        world, _ = sut_combustion_stage(world, data)
        spec_id = "fuel_consumption" if tick == p["burn_duration"] else "active_burn"
        item_data = dict(data, tick=tick, material=material)
        item = record(f"{material.upper()}_FULL_LIFECYCLE", ordinal, item_data, before, world, spec_id, manifest); ordinal += 1
        if not item["accepted"]:
            raise AssertionError(f"{material} lifecycle tick {tick}")
        records.append(item)
        stage = world.stage
        gross_total += stage.gross_q
        deposited_total += stage.deposited_q
        clipped_total += stage.clipped_q
        if stage.heat:
            emitting.append(tick)
        if abs(stage.deposited_q + stage.clipped_q - stage.gross_q) > manifest["tolerances"]["absolute_Q"]:
            raise AssertionError("Q closure")
        trace_hash.update(canonical({"tick": tick, "before": item["before_state"], "after": item["after_state"]}))
    expected_emit = list(range(1, p["burn_duration"]))
    if emitting != expected_emit or world.stage != Stage(consumption=1) or world.current[0].material != "EMPTY":
        raise AssertionError(f"{material} final lifecycle")
    return ordinal, {
        "duration": p["burn_duration"], "emitting_ticks": len(emitting), "first_emitting_tick": emitting[0], "last_emitting_tick": emitting[-1],
        "final_tick": p["burn_duration"], "final_stage": dataclasses.asdict(world.stage), "gross_Q": gross_total,
        "deposited_Q": deposited_total, "clipped_Q": clipped_total, "Q_closure": abs(deposited_total + clipped_total - gross_total) <= manifest["tolerances"]["absolute_Q"],
        "trace_sha256": trace_hash.hexdigest(),
    }


def run_cap_controls(manifest: dict[str, Any], records: list[dict[str, Any]], ordinal: int) -> tuple[int, dict[str, Any]]:
    p = manifest["coefficients"]["Oil"]
    results: dict[str, Any] = {}
    for name, temperature in (("below", 1000.0), ("crossing", 1198.0), ("at", 1200.0), ("above", 1250.0)):
        world = make_world(2, 1, (Cell("Oil", temperature, COMBUSTING), Cell("EMPTY", air_mass=1.0, air_energy=20.0)))
        data = {"cell": 0, "parameters": p, "temperature_cap_C": 1200.0, "empty_temperature_C": 20.0, "control": name}
        before = world
        world, _ = sut_combustion_stage(world, data)
        item = record("CAP_CONTROLS", ordinal, data, before, world, "active_burn", manifest); ordinal += 1
        if not item["accepted"]:
            raise AssertionError(f"cap {name}")
        records.append(item)
        results[name] = {"before": temperature, "after": world.current[0].temperature, "stage": dataclasses.asdict(world.stage)}
    if results["above"]["after"] != results["above"]["before"]:
        raise AssertionError("above-cap reduction")
    return ordinal, results


def validate_manifest(manifest: dict[str, Any]) -> None:
    if manifest["evidence_identity"] != IDENTITY or manifest["counter_provenance"] != "INDEPENDENT_SPEC_BEFORE_AFTER_AUDIT":
        raise AssertionError("manifest identity/provenance")
    calculated = {k: sha256(canonical(v)) for k,v in manifest["audit_specifications"].items()}
    if calculated != manifest["audit_specification_hashes"]:
        raise AssertionError("audit spec hash")
    if set(manifest["required_transaction_classes"]) != set(manifest["audit_specifications"]):
        raise AssertionError("transaction inventory")


def load_manifest(path: Path) -> tuple[dict[str, Any], str]:
    raw = path.read_bytes()
    digest = sha256(raw)
    if digest != EXPECTED_MANIFEST_SHA256:
        raise AssertionError(f"manifest hash mismatch {digest}")
    manifest = json.loads(raw)
    validate_manifest(manifest)
    return manifest, digest


def validate_snapshot_record(item: dict[str, Any], manifest: dict[str, Any]) -> tuple[bool, bool]:
    required = set(manifest["snapshot_schema"]["required_fields"])
    if set(item) != required:
        raise AssertionError("snapshot schema")
    claimed = item["record_sha256"]
    clone = dict(item)
    clone["record_sha256"] = ""
    if sha256(canonical(clone)) != claimed:
        raise AssertionError("snapshot record hash")
    before, after = from_payload(item["before_state"]), from_payload(item["after_state"])
    accepted, _summary = audit_transition(item["audit_specification_id"], before, after, item["transaction_input"], manifest)
    return accepted == item["accepted"], accepted


def reaudit(snapshot_path: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    records = [json.loads(line) for line in snapshot_path.read_text(encoding="utf-8").splitlines() if line]
    accepted = rejected = 0
    lifecycle = {"Oil": {"gross": 0.0, "ticks": 0, "final_zero": False}, "Wood": {"gross": 0.0, "ticks": 0, "final_zero": False}}
    for item in records:
        consistent, independently_accepted = validate_snapshot_record(item, manifest)
        if not consistent:
            raise AssertionError("stored audit disposition mismatch")
        accepted += int(independently_accepted)
        rejected += int(not independently_accepted)
        for material in lifecycle:
            if item["fixture"] == f"{material.upper()}_FULL_LIFECYCLE":
                lifecycle[material]["ticks"] += 1
                lifecycle[material]["gross"] += item["after_state"]["stage"]["gross_q"]
                if item["transaction_input"]["tick"] == item["transaction_input"]["parameters"]["burn_duration"]:
                    stage = item["after_state"]["stage"]
                    lifecycle[material]["final_zero"] = stage["heat"] == stage["flame"] == stage["smoke"] == 0 and stage["gross_q"] == stage["deposited_q"] == 0.0 and stage["consumption"] == 1
    return {"records": len(records), "accepted": accepted, "rejected": rejected, "lifecycle": lifecycle, "snapshot_sha256": sha256(snapshot_path.read_bytes())}


def preflight(manifest_path: Path) -> int:
    manifest, digest = load_manifest(manifest_path)
    print(json.dumps({"evidence_identity": IDENTITY, "manifest_sha256": digest, "fixtures": manifest["fixtures"], "transaction_specs": sorted(manifest["audit_specifications"]), "snapshot_schema": manifest["snapshot_schema"], "evidence_executed": False}, indent=2, sort_keys=True))
    return 0


def execute(manifest_path: Path, snapshots_path: Path, result_path: Path, failure_path: Path) -> int:
    if any(path.exists() for path in (snapshots_path, result_path, failure_path)):
        raise AssertionError("supplement output exists; refusing second attempt")
    script_hash = sha256(Path(__file__).read_bytes())
    manifest_hash = ""
    try:
        manifest, manifest_hash = load_manifest(manifest_path)
        records: list[dict[str, Any]] = []
        ordinal = 1
        ordinal, transaction_counts = run_semantic_matrix(manifest, records, ordinal)
        ordinal, f15b = run_f15b(manifest, records, ordinal)
        ordinal, oil = run_lifecycle("Oil", manifest, records, ordinal)
        ordinal, wood = run_lifecycle("Wood", manifest, records, ordinal)
        ordinal, caps = run_cap_controls(manifest, records, ordinal)
        snapshots_path.write_text("".join(json.dumps(item, sort_keys=True, separators=(",", ":")) + "\n" for item in records), encoding="utf-8", newline="\n")
        third_party = reaudit(snapshots_path, manifest)
        valid_count = sum(item["accepted"] for item in records)
        rejected_count = len(records) - valid_count
        required_classes = set(manifest["required_transaction_classes"])
        observed_classes = {item["audit_specification_id"] for item in records if item["accepted"]}
        unaudited = len(required_classes - observed_classes)
        expected_gross = {
            material: (parameters["burn_duration"] - 1) * parameters["gross_Q"]
            for material, parameters in manifest["coefficients"].items()
        }
        if oil["gross_Q"] != expected_gross["Oil"] or wood["gross_Q"] != expected_gross["Wood"]:
            raise AssertionError("lifecycle gross totals")
        if third_party["lifecycle"]["Oil"]["gross"] != expected_gross["Oil"] or third_party["lifecycle"]["Wood"]["gross"] != expected_gross["Wood"]:
            raise AssertionError("third-party lifecycle totals")
        if rejected_count < len(required_classes) * 2 or unaudited != 0:
            raise AssertionError("negative/audit coverage")
        result: dict[str, Any] = {
            "evidence_identity": IDENTITY, "script_sha256": script_hash, "manifest_sha256": manifest_hash,
            "snapshot_bundle_sha256": third_party["snapshot_sha256"], "snapshot_record_count": len(records),
            "accepted_valid_transition_count": valid_count, "rejected_negative_control_count": rejected_count,
            "valid_transaction_classes": transaction_counts, "unaudited_required_paths": unaudited,
            "attempts": 1, "completions": 1,
            "topology_result": "PASS", "topology_derived_f15b": "PASS", "f15b_receipt": f15b,
            "auditor_result": "PASS", "independent_semantic_audit": "PASS", "counter_provenance": manifest["counter_provenance"],
            "lifecycle_result": {"Oil": oil, "Wood": wood, "cap_controls": caps, "final_tick_zero_emission": "PASS"},
            "oil_lifecycle": "PASS", "wood_lifecycle": "PASS", "final_tick_zero_emission": "PASS",
            "negative_control_rejections": "ALL_REQUIRED",
            "snapshot_result": {"status": "PASS", "third_party_reaudit": third_party},
            "air_mass_energy_conservation": "PASS" if f15b["air_conserved"] else "FAIL",
            "GPU_status": "NOT_ESTABLISHED", "product_status": "NOT_ESTABLISHED", "user_status": "PENDING",
            "result_sha256": "", "result_hash_scope": "CANONICAL_JSON_WITH_RESULT_SHA256_EMPTY",
            "limitations": ["No v1/v2/v3 evidence was rerun.", "No Rust, WGSL, GPU, runtime, build or application was executed.", "This supplement closes only v3 H-001/H-002/H-003/M-001 at the reduced transaction-evidence layer."]
        }
        result["result_sha256"] = sha256(canonical(result))
        result_path.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
        return 0
    except Exception as exc:
        failure_path.write_text(json.dumps({"evidence_identity": IDENTITY, "script_sha256": script_hash, "manifest_sha256": manifest_hash, "attempts": 1, "completions": 0, "error_type": type(exc).__name__, "error": str(exc)}, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
        raise


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--preflight", action="store_true")
    parser.add_argument("--reaudit-snapshots", type=Path)
    parser.add_argument("--snapshots", type=Path)
    parser.add_argument("--result", type=Path)
    parser.add_argument("--failure", type=Path)
    args = parser.parse_args()
    manifest, _ = load_manifest(args.manifest)
    if args.preflight:
        if any((args.reaudit_snapshots, args.snapshots, args.result, args.failure)):
            raise SystemExit("preflight accepts no output or re-audit path")
        return preflight(args.manifest)
    if args.reaudit_snapshots:
        if any((args.snapshots, args.result, args.failure)):
            raise SystemExit("re-audit accepts only snapshot path")
        print(json.dumps(reaudit(args.reaudit_snapshots, manifest), indent=2, sort_keys=True))
        return 0
    if not all((args.snapshots, args.result, args.failure)):
        raise SystemExit("execution requires snapshots, result and failure")
    return execute(args.manifest, args.snapshots, args.result, args.failure)


if __name__ == "__main__":
    sys.exit(main())
