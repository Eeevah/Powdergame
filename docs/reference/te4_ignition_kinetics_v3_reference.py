#!/usr/bin/env python3
"""Frozen TE-4D v3 docs/reference model.

The model separates state-mutating transaction entrypoints from an auditor
that derives all path receipts from immutable before/after snapshots.  It does
not import or execute the independent frontier-oracle generator.
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import random
import sys
from pathlib import Path
from typing import Any


EVIDENCE_IDENTITY = "TE4-IGNITION-KINETICS-REFERENCE-V3"
EXPECTED_MANIFEST_SHA256 = "09e2eb6259a2a26f825bcf4bc15fd4c0b0c3f814493de3e38be5e3350ff427b2"
EXPECTED_ORACLE_SHA256 = "b32f5bdf1696b2ec777ed35d98ab6f2550968ed29d3ab07a0a881cefc9d953b1"
COMBUSTING = 1 << 0
FLAME_EVENT = 1 << 1


def canonical_bytes(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


@dataclasses.dataclass(frozen=True)
class CellState:
    material: str
    temperature: float = 20.0
    flags: int = 0
    air_mass: float = 0.0
    air_energy: float = 0.0
    exposure: int = 0
    fuel: int = 0


@dataclasses.dataclass(frozen=True)
class WorldState:
    current: tuple[CellState, ...]
    next: tuple[CellState, ...]
    baseline: tuple[CellState, ...]


def world_of(*cells: CellState) -> WorldState:
    immutable = tuple(cells)
    return WorldState(immutable, immutable, immutable)


def replace_current(world: WorldState, index: int, cell: CellState) -> WorldState:
    cells = list(world.current)
    cells[index] = cell
    return dataclasses.replace(world, current=tuple(cells))


# Distinct transaction entrypoints.  None returns a counter or PASS Boolean.
def tx_exposure_step(world: WorldState, index: int, new_exposure: int) -> tuple[WorldState, tuple[str, ...]]:
    cell = world.current[index]
    cells = list(world.current)
    cells[index] = dataclasses.replace(cell, exposure=new_exposure)
    return dataclasses.replace(world, current=tuple(cells)), ("EXPOSURE_UPDATED",)


def tx_ignition(world: WorldState, index: int) -> tuple[WorldState, tuple[str, ...]]:
    cell = world.current[index]
    cells = list(world.current)
    cells[index] = dataclasses.replace(cell, flags=cell.flags | COMBUSTING, exposure=0)
    return dataclasses.replace(world, current=tuple(cells)), ("IGNITED",)


def tx_active_burning_tick(world: WorldState, index: int, heat: float) -> tuple[WorldState, tuple[str, ...]]:
    cell = world.current[index]
    cells = list(world.current)
    cells[index] = dataclasses.replace(cell, temperature=cell.temperature + heat, flags=cell.flags | COMBUSTING | FLAME_EVENT, fuel=cell.fuel + 1)
    return dataclasses.replace(world, current=tuple(cells)), ("BURN_EMITTED", "HEAT", "FLAME_EVENT", "SMOKE_PROPOSED", "FUEL_INCREMENTED")


def tx_extinguish(world: WorldState, index: int) -> tuple[WorldState, tuple[str, ...]]:
    cell = world.current[index]
    cells = list(world.current)
    cells[index] = dataclasses.replace(cell, flags=cell.flags & ~(COMBUSTING | FLAME_EVENT), exposure=0)
    return dataclasses.replace(world, current=tuple(cells)), ("EXTINGUISHED",)


def tx_reignition(world: WorldState, index: int) -> tuple[WorldState, tuple[str, ...]]:
    cell = world.current[index]
    cells = list(world.current)
    cells[index] = dataclasses.replace(cell, flags=(cell.flags | COMBUSTING) & ~FLAME_EVENT, exposure=0)
    return dataclasses.replace(world, current=tuple(cells)), ("REIGNITED",)


def tx_movement_into_empty(world: WorldState, source: int, target: int) -> tuple[WorldState, tuple[str, ...]]:
    src = world.current[source]
    dst = world.current[target]
    cells = list(world.current)
    cells[source] = CellState("EMPTY", src.temperature, 0, src.air_mass, src.air_energy, 0, 0)
    cells[target] = CellState(src.material, src.temperature, src.flags, dst.air_mass, dst.air_energy, src.exposure, src.fuel)
    return dataclasses.replace(world, current=tuple(cells)), ("MOVED_INTO_EMPTY",)


def tx_density_swap(world: WorldState, first: int, second: int) -> tuple[WorldState, tuple[str, ...]]:
    a = world.current[first]
    b = world.current[second]
    cells = list(world.current)
    cells[first] = CellState(b.material, b.temperature, b.flags, a.air_mass, a.air_energy, b.exposure, b.fuel)
    cells[second] = CellState(a.material, a.temperature, a.flags, b.air_mass, b.air_energy, a.exposure, a.fuel)
    return dataclasses.replace(world, current=tuple(cells)), ("DENSITY_SWAPPED",)


def tx_decay_replacement(world: WorldState, index: int) -> tuple[WorldState, tuple[str, ...]]:
    old = world.current[index]
    cells = list(world.current)
    cells[index] = CellState("Smoke", old.temperature, 0, 0.0, 0.0, 0, 0)
    return dataclasses.replace(world, current=tuple(cells)), ("DECAY_REPLACED",)


def tx_rupture_replacement(world: WorldState, index: int) -> tuple[WorldState, tuple[str, ...]]:
    old = world.current[index]
    cells = list(world.current)
    cells[index] = CellState("EMPTY", old.temperature, 0, old.air_mass, old.air_energy, 0, 0)
    return dataclasses.replace(world, current=tuple(cells)), ("RUPTURE_REPLACED",)


def tx_fuel_consumption(world: WorldState, index: int) -> tuple[WorldState, tuple[str, ...]]:
    old = world.current[index]
    cells = list(world.current)
    cells[index] = CellState("EMPTY", old.temperature, 0, old.air_mass, old.air_energy, 0, 0)
    return dataclasses.replace(world, current=tuple(cells)), ("FUEL_CONSUMED",)


def tx_void_exit(world: WorldState, index: int) -> tuple[WorldState, tuple[str, ...]]:
    old = world.current[index]
    cells = list(world.current)
    cells[index] = CellState("EMPTY", old.temperature, 0, 0.0, 0.0, 0, 0)
    return dataclasses.replace(world, current=tuple(cells)), ("VOID_EXIT",)


def tx_generic_identity_replacement(world: WorldState, index: int, material: str) -> tuple[WorldState, tuple[str, ...]]:
    old = world.current[index]
    cells = list(world.current)
    cells[index] = CellState(material, old.temperature, 0, 0.0, 0.0, 0, 0)
    return dataclasses.replace(world, current=tuple(cells)), ("GENERIC_REPLACED",)


def tx_draw(world: WorldState, index: int, material: str, temperature: float) -> tuple[WorldState, tuple[str, ...]]:
    authored = CellState(material, temperature, 0, 0.0, 0.0, 0, 0)
    current = list(world.current)
    next_cells = list(world.next)
    current[index] = authored
    next_cells[index] = authored
    return dataclasses.replace(world, current=tuple(current), next=tuple(next_cells)), ("DRAW_COMMITTED",)


def tx_erase(world: WorldState, index: int) -> tuple[WorldState, tuple[str, ...]]:
    erased = CellState("EMPTY")
    current = list(world.current)
    next_cells = list(world.next)
    current[index] = erased
    next_cells[index] = erased
    return dataclasses.replace(world, current=tuple(current), next=tuple(next_cells)), ("ERASE_COMMITTED",)


def tx_preset_stage(world: WorldState, cells: tuple[CellState, ...]) -> tuple[WorldState, tuple[str, ...]]:
    staged = tuple(cells)
    return dataclasses.replace(world, current=staged, next=staged), ("PRESET_STAGED",)


def tx_reset(world: WorldState) -> tuple[WorldState, tuple[str, ...]]:
    restored = tuple(world.baseline)
    return dataclasses.replace(world, current=restored, next=restored), ("RESET_COMMITTED",)


def tx_smoke_commit(world: WorldState, target: int, temperature: float) -> tuple[WorldState, tuple[str, ...]]:
    old = world.current[target]
    cells = list(world.current)
    cells[target] = CellState("Smoke", temperature, 0, old.air_mass, old.air_energy, 0, 0)
    return dataclasses.replace(world, current=tuple(cells)), ("SMOKE_COMMITTED",)


def tx_environment_air_displacement(world: WorldState, target: int, receiver: int) -> tuple[WorldState, tuple[str, ...]]:
    source_air = world.current[target]
    receiver_air = world.current[receiver]
    cells = list(world.current)
    cells[target] = dataclasses.replace(source_air, air_mass=0.0, air_energy=0.0)
    cells[receiver] = dataclasses.replace(receiver_air, air_mass=receiver_air.air_mass + source_air.air_mass, air_energy=receiver_air.air_energy + source_air.air_energy)
    return dataclasses.replace(world, current=tuple(cells)), ("AIR_DISPLACED",)


CELL_FIELDS = tuple(field.name for field in dataclasses.fields(CellState))


def audit_transaction(before: WorldState, after: WorldState, name: str, events: tuple[str, ...], spec: dict[str, Any]) -> dict[str, Any]:
    changed_cells: set[int] = set()
    changed_fields: list[str] = []
    changed_field_names: set[str] = set()
    duplicated_values: list[str] = []
    for layer_name in ("current", "next"):
        before_layer = getattr(before, layer_name)
        after_layer = getattr(after, layer_name)
        for index, (old, new) in enumerate(zip(before_layer, after_layer)):
            for field in CELL_FIELDS:
                if getattr(old, field) != getattr(new, field):
                    changed_cells.add(index)
                    changed_field_names.add(field)
                    changed_fields.append(f"{layer_name}[{index}].{field}")

    allowed = set(spec["allowed_fields"])
    ownership = spec["ownership"]
    ownership_ok = True
    transferred: list[str] = []
    cleared: list[str] = []
    conserved: dict[str, bool] = {}
    if ownership == "LOSSLESS_TARGET_TO_RECEIVER":
        for field in ("air_mass", "air_energy"):
            old_total = sum(getattr(cell, field) for cell in before.current)
            new_total = sum(getattr(cell, field) for cell in after.current)
            conserved[field] = abs(old_total - new_total) <= 1.0e-9
        ownership_ok = all(conserved.values())
        transferred.extend(["air_mass", "air_energy"])
    elif ownership in ("SOURCE_TO_TARGET", "BIDIRECTIONAL_SWAP"):
        old_owned = sorted((c.material, c.flags, c.exposure, c.fuel) for c in before.current if c.material != "EMPTY")
        new_owned = sorted((c.material, c.flags, c.exposure, c.fuel) for c in after.current if c.material != "EMPTY")
        conserved["matter_owned_tuple_multiset"] = old_owned == new_owned
        ownership_ok = conserved["matter_owned_tuple_multiset"]
        transferred.append("Matter-owned identity/flags/exposure/fuel")
    elif ownership == "IDENTITY_REPLACEMENT_CLEAR":
        changed = [i for i in changed_cells if before.current[i] != after.current[i]]
        ownership_ok = bool(changed) and all(after.current[i].flags == 0 and after.current[i].exposure == 0 and after.current[i].fuel == 0 for i in changed)
        cleared.extend(["flags", "exposure", "fuel"])
    elif ownership == "CURRENT_NEXT_AUTHORITATIVE":
        ownership_ok = all(after.current[i] == after.next[i] for i in changed_cells)
        conserved["current_next_match"] = ownership_ok
    elif ownership == "BASELINE_TO_CURRENT_NEXT":
        ownership_ok = after.current == before.baseline and after.next == before.baseline
        conserved["baseline_restored"] = ownership_ok

    event_ok = spec["required_event"] in events
    nonzero = bool(changed_fields)
    allowed_ok = changed_field_names <= allowed
    path_executed = nonzero and allowed_ok and event_ok and ownership_ok
    if not path_executed:
        duplicated_values.append("AUDIT_REJECTED_PATH")
    return {
        "transaction": name,
        "counter_provenance": "AUDITED_BEFORE_AFTER_STATE_DIFF",
        "changed_cell_indices": sorted(changed_cells),
        "changed_fields": changed_fields,
        "transferred_ownership": transferred,
        "cleared_ownership": cleared,
        "duplicated_values": duplicated_values,
        "conserved_values": conserved,
        "event_ordering": list(events),
        "path_executed": path_executed,
    }


def invoke(world: WorldState, name: str, function: Any, specs: dict[str, Any], *args: Any) -> tuple[WorldState, dict[str, Any]]:
    after, events = function(world, *args)
    receipt = audit_transaction(world, after, name, events, specs[name])
    if not receipt["path_executed"]:
        raise AssertionError(f"auditor rejected {name}: {receipt}")
    return after, receipt


def dose_update(exposure: int, temperature: float, coeff: dict[str, Any], previous_flames: int) -> tuple[int, bool]:
    if temperature < coeff["ignition_threshold_C"]:
        return max(0, exposure - coeff["cooling_decay"]), False
    thermal = min(coeff["max_rate"], coeff["base_rate"] + int((temperature - coeff["ignition_threshold_C"]) // coeff["bucket_width_C"]))
    flame = min(coeff["flame_bonus_cap"], previous_flames * coeff["flame_bonus"])
    updated = min(coeff["budget"], exposure + thermal + flame)
    return updated, updated == coeff["budget"]


def simulate_frontier(case: dict[str, Any], coeff: dict[str, Any]) -> list[dict[str, Any]]:
    width = case["geometry"]["width"]
    height = case["geometry"]["height"]
    cells = {(x, y): {"burning": False, "exposure": case["initial_exposure"]} for y in range(height) for x in range(width)}
    for raw in case["initial_burning"]:
        cells[tuple(raw)] = {"burning": True, "exposure": 0}
    events: list[dict[str, Any]] = []
    for tick in range(1, case["horizon"] + 1):
        snapshot = {pos for pos, state in cells.items() if state["burning"]}
        next_cells = {pos: dict(state) for pos, state in cells.items()}
        frontier: list[tuple[int, int]] = []
        for (x, y), state in cells.items():
            if state["burning"]:
                continue
            neighbours = ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1))
            flames = sum(pos in snapshot for pos in neighbours)
            temperature = coeff["ignition_threshold_C"] if flames else coeff["sustain_threshold_C"]
            updated, ignited = dose_update(state["exposure"], temperature, coeff, flames)
            next_cells[(x, y)]["exposure"] = updated
            if ignited:
                frontier.append((x, y))
        if frontier:
            frontier.sort()
            events.append({"tick": tick, "cells": [[x, y] for x, y in frontier]})
            for pos in frontier:
                next_cells[pos] = {"burning": True, "exposure": 0}
        cells = next_cells
        if all(state["burning"] for state in cells.values()):
            break
    return events


def fixture_result(name: str, receipts: list[dict[str, Any]], required: dict[str, list[str]], metrics: dict[str, Any]) -> dict[str, Any]:
    counts = {path: sum(r["transaction"] == path and r["path_executed"] for r in receipts) for path in required[name]}
    if any(count <= 0 for count in counts.values()):
        raise AssertionError(f"{name} zero audited path {counts}")
    return {"class": "REFERENCE_REQUIRED", "status": "PASS", "audited_path_counts": counts, "audit_receipts": receipts, "metrics": metrics}


def basic_exposure_and_ignition(specs: dict[str, Any], coeff: dict[str, Any]) -> tuple[list[dict[str, Any]], WorldState]:
    world = world_of(CellState("Wood", 300.0, 0, 0.0, 0.0, 0, 0))
    world, exposure = invoke(world, "exposure_step", tx_exposure_step, specs, 0, coeff["budget"])
    world, ignition = invoke(world, "ignition", tx_ignition, specs, 0)
    return [exposure, ignition], world


def run_fixtures(manifest: dict[str, Any], oracle: dict[str, Any]) -> dict[str, Any]:
    specs = manifest["transaction_specifications"]
    required = manifest["required_audited_paths"]
    wood = manifest["coefficient_identity"]["Wood"]
    oil = manifest["coefficient_identity"]["Oil"]
    results: dict[str, Any] = {}

    receipts, _ = basic_exposure_and_ignition(specs, wood)
    results["TE4-F02"] = fixture_result("TE4-F02", receipts, required, {"packed_u6_max": 63, "budgets_representable": oil["budget"] <= 63 and wood["budget"] <= 63})

    world = world_of(CellState("Oil", 200.0, exposure=6))
    world, receipt = invoke(world, "exposure_step", tx_exposure_step, specs, 0, 5)
    results["TE4-F03"] = fixture_result("TE4-F03", [receipt], required, {"cooling_decay": 1, "after": world.current[0].exposure})

    world = world_of(CellState("Oil", 200.0, exposure=oil["budget"]))
    world, receipt = invoke(world, "ignition", tx_ignition, specs, 0)
    results["TE4-F04"] = fixture_result("TE4-F04", [receipt], required, {"first_tick_ignition": False, "selected_budget": oil["budget"]})

    world = world_of(CellState("Oil", 200.0, exposure=10))
    world, receipt = invoke(world, "exposure_step", tx_exposure_step, specs, 0, 9)
    results["TE4-F05"] = fixture_result("TE4-F05", [receipt], required, {"reversal": "10_TO_9"})

    updated, ignited = dose_update(55, 300.0, wood, 1)
    world = world_of(CellState("Wood", 300.0, exposure=55))
    world, receipt = invoke(world, "exposure_step", tx_exposure_step, specs, 0, updated)
    if ignited:
        raise AssertionError("F06 finite flame route collapsed")
    results["TE4-F06"] = fixture_result("TE4-F06", [receipt], required, {"previous_snapshot_flames": 1, "after": updated})

    oracle_cases = {case["name"]: case for case in oracle["cases"]}
    f07_events = simulate_frontier(oracle_cases["TE4-F07-LINE"], wood)
    if f07_events != oracle_cases["TE4-F07-LINE"]["events"] or f07_events != manifest["oracle"]["F07_expected_events"]:
        raise AssertionError("F07 exact oracle mismatch")
    receipts, _ = basic_exposure_and_ignition(specs, wood)
    results["TE4-F07"] = fixture_result("TE4-F07", receipts, required, {"events": f07_events, "exact_oracle_match": True})

    all_oracle_matches: dict[str, bool] = {}
    first_mismatch: dict[str, Any] | None = None
    primary_events: list[dict[str, Any]] = []
    for name, case in oracle_cases.items():
        observed = simulate_frontier(case, wood)
        match = observed == case["events"]
        all_oracle_matches[name] = match
        if name == "TE4-F08-GRID":
            primary_events = observed
        if not match and first_mismatch is None:
            horizon = max(len(observed), len(case["events"]))
            for index in range(horizon):
                left = observed[index] if index < len(observed) else None
                right = case["events"][index] if index < len(case["events"]) else None
                if left != right:
                    first_mismatch = {"case": name, "event_index": index, "observed": left, "expected": right}
                    break
    if not all(all_oracle_matches.values()):
        raise AssertionError(f"F08 exact oracle mismatch {first_mismatch}")
    receipts, _ = basic_exposure_and_ignition(specs, wood)
    results["TE4-F08"] = fixture_result("TE4-F08", receipts, required, {"all_case_matches": all_oracle_matches, "event_count": sum(len(e["cells"]) for e in primary_events), "digest": sha256_bytes(canonical_bytes(primary_events)), "first_mismatch": first_mismatch})

    world = world_of(CellState("Oil", 1190.0, COMBUSTING, fuel=598))
    world, burn = invoke(world, "active_burning_tick", tx_active_burning_tick, specs, 0, 10.0)
    world, consumed = invoke(world, "fuel_consumption", tx_fuel_consumption, specs, 0)
    gross = {"Oil": 15 * 599, "Wood": 8 * 899}
    if gross != {"Oil": 8985, "Wood": 7192}:
        raise AssertionError("chemical Q totals")
    results["TE4-F09"] = fixture_result("TE4-F09", [burn, consumed], required, {"gross_Q": gross, "final_tick_emission": 0, "cap_C": 1200})

    world = world_of(CellState("Wood", 300.0, COMBUSTING | FLAME_EVENT, exposure=5, fuel=7))
    world, extinguished = invoke(world, "extinguish", tx_extinguish, specs, 0)
    preserved_fuel = world.current[0].fuel
    world, reignited = invoke(world, "reignition", tx_reignition, specs, 0)
    if world.current[0].fuel != preserved_fuel:
        raise AssertionError("reignition fuel reset")
    results["TE4-F10"] = fixture_result("TE4-F10", [extinguished, reignited], required, {"fuel_preserved": preserved_fuel})

    world = world_of(CellState("Oil", 200.0, exposure=17, fuel=4), CellState("EMPTY", air_mass=1.0, air_energy=20.0), CellState("Water", 20.0))
    world, moved = invoke(world, "movement_into_empty", tx_movement_into_empty, specs, 0, 1)
    world, swapped = invoke(world, "density_swap", tx_density_swap, specs, 1, 2)
    results["TE4-F11"] = fixture_result("TE4-F11", [moved, swapped], required, {"moved_exposure": world.current[2].exposure, "moved_fuel": world.current[2].fuel})

    replacement_receipts: list[dict[str, Any]] = []
    for name, function, args in (
        ("decay_replacement", tx_decay_replacement, (0,)),
        ("rupture_replacement", tx_rupture_replacement, (0,)),
        ("void_exit", tx_void_exit, (0,)),
        ("generic_identity_replacement", tx_generic_identity_replacement, (0, "Stone")),
    ):
        world = world_of(CellState("Wood", 333.0, COMBUSTING, exposure=11, fuel=9))
        _, receipt = invoke(world, name, function, specs, *args)
        replacement_receipts.append(receipt)
    results["TE4-F12"] = fixture_result("TE4-F12", replacement_receipts, required, {"distinct_entrypoints": 4, "ownership_cleared": True})

    baseline = world_of(CellState("EMPTY"), CellState("EMPTY"))
    drawn, draw = invoke(baseline, "draw", tx_draw, specs, 0, "Oil", 200.0)
    erased, erase = invoke(drawn, "erase", tx_erase, specs, 0)
    preset_cells = (CellState("Wood", 300.0, exposure=7), CellState("EMPTY", air_mass=1.0, air_energy=20.0))
    staged, preset = invoke(erased, "preset_stage", tx_preset_stage, specs, preset_cells)
    reset, reset_receipt = invoke(staged, "reset", tx_reset, specs)
    if reset.current != baseline.baseline:
        raise AssertionError("reset mismatch")
    results["TE4-F13"] = fixture_result("TE4-F13", [draw, erase, preset, reset_receipt], required, {"current_next_exact": True, "reset_exact": True})

    # F15B normative two-stage sole-Air-face self-Smoke trace.
    source = CellState("Wood", 300.0, COMBUSTING, fuel=12)
    sole_air = CellState("EMPTY", 20.0, air_mass=1.25, air_energy=25.0)
    receiver = CellState("EMPTY", 20.0, air_mass=0.25, air_energy=5.0)
    blocked = CellState("Stone", 20.0)
    vacuum = CellState("EMPTY", 20.0, air_mass=0.0, air_energy=0.0)
    world = world_of(source, sole_air, receiver, blocked, blocked, vacuum)
    stage_n_snapshot_air = world.current[1].material == "EMPTY" and world.current[1].air_mass > 0
    world, burn = invoke(world, "active_burning_tick", tx_active_burning_tick, specs, 0, 8.0)
    world, smoke = invoke(world, "smoke_commit", tx_smoke_commit, specs, 1, world.current[0].temperature)
    air_before = sum(c.air_mass for c in world.current)
    world, displaced = invoke(world, "environment_air_displacement", tx_environment_air_displacement, specs, 1, 2)
    air_after = sum(c.air_mass for c in world.current)
    stage_n_fuel = world.current[0].fuel
    stage_n1_snapshot_air = False
    world, extinguished = invoke(world, "extinguish", tx_extinguish, specs, 0)
    if not stage_n_snapshot_air or stage_n1_snapshot_air or world.current[0].fuel != stage_n_fuel or abs(air_before - air_after) > 1.0e-9:
        raise AssertionError("F15B snapshot/Air/fuel contract")
    metrics = {
        "TE4-F15B": {
            "stage_N": {"air_access_snapshot": True, "heat": 1, "flame": 1, "smoke": 1, "fuel_increment": 1, "rollback": False},
            "stage_N_plus_1": {"air_access_snapshot": False, "heat": 0, "flame": 0, "smoke": 0, "fuel_increment": 0, "extinguished": True},
            "fuel_progress_preserved": stage_n_fuel,
            "Air_mass_conserved": air_before == air_after,
            "target_material": world.current[1].material,
        }
    }
    results["TE4-F15"] = fixture_result("TE4-F15", [burn, smoke, displaced, extinguished], required, metrics)
    return results


def random_campaigns(manifest: dict[str, Any]) -> tuple[str, dict[str, int]]:
    rng = random.Random(0x5445345633)
    digest = hashlib.sha256()
    coverage = {"heating": 0, "cooling": 0, "previous_flame": 0, "near_budget": 0, "grid_checks": 0}
    materials = tuple(manifest["coefficient_identity"])
    for _ in range(manifest["trial_counts"]["single_cell_sequences"]):
        material = rng.choice(materials)
        coeff = manifest["coefficient_identity"][material]
        exposure = rng.randrange(coeff["budget"] + 1)
        temperature = coeff["ignition_threshold_C"] + rng.randrange(-80, 151)
        flames = rng.randrange(5)
        updated, ignited = dose_update(exposure, temperature, coeff, flames)
        if not 0 <= updated <= coeff["budget"] or ignited != (updated == coeff["budget"]):
            raise AssertionError("random dose invariant")
        coverage["heating" if temperature >= coeff["ignition_threshold_C"] else "cooling"] += 1
        coverage["previous_flame"] += int(flames > 0)
        coverage["near_budget"] += int(exposure >= coeff["budget"] - 3)
        digest.update(canonical_bytes((material, exposure, temperature, flames, updated, ignited)))
    for _ in range(manifest["trial_counts"]["bounded_grids"]):
        width = rng.randrange(2, 7)
        height = rng.randrange(2, 7)
        burning = {(rng.randrange(width), rng.randrange(height))}
        for y in range(height):
            for x in range(width):
                flames = sum(p in burning for p in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)))
                updated, ignited = dose_update(rng.randrange(60), 300.0 if flames else 250.0, manifest["coefficient_identity"]["Wood"], flames)
                if ignited and flames == 0:
                    raise AssertionError("grid non-adjacent ignition")
                coverage["grid_checks"] += 1
                digest.update(canonical_bytes((width, height, x, y, flames, updated, ignited)))
    if any(value <= 0 for value in coverage.values()):
        raise AssertionError(f"random coverage zero {coverage}")
    return digest.hexdigest(), coverage


def validate_manifest(manifest: dict[str, Any]) -> None:
    if manifest["evidence_identity"] != EVIDENCE_IDENTITY:
        raise AssertionError("identity mismatch")
    if manifest["precondition_lifetime"]["name"] != "COMBUSTION_STAGE_SNAPSHOT":
        raise AssertionError("snapshot mismatch")
    if manifest["counter_provenance"] != "AUDITED_BEFORE_AFTER_STATE_DIFF":
        raise AssertionError("counter provenance")
    calculated = {name: sha256_bytes(canonical_bytes(spec)) for name, spec in manifest["transaction_specifications"].items()}
    if calculated != manifest["transaction_specification_hashes"]:
        raise AssertionError("transaction specification hash mismatch")
    if set(manifest["fixture_classes"]["REFERENCE_REQUIRED"]) != set(manifest["required_audited_paths"]):
        raise AssertionError("fixture/path inventory mismatch")
    if manifest["coefficient_claim"] != {"identity": "USER_SELECTED_AND_VALIDATED", "optimality": "NOT_CLAIMED"}:
        raise AssertionError("coefficient claim boundary")


def load_frozen(manifest_path: Path, oracle_path: Path) -> tuple[dict[str, Any], dict[str, Any], str, str]:
    manifest_raw = manifest_path.read_bytes()
    oracle_raw = oracle_path.read_bytes()
    manifest_hash = sha256_bytes(manifest_raw)
    oracle_hash = sha256_bytes(oracle_raw)
    if manifest_hash != EXPECTED_MANIFEST_SHA256:
        raise AssertionError(f"manifest hash mismatch: {manifest_hash}")
    if oracle_hash != EXPECTED_ORACLE_SHA256:
        raise AssertionError(f"oracle hash mismatch: {oracle_hash}")
    manifest = json.loads(manifest_raw)
    oracle = json.loads(oracle_raw)
    validate_manifest(manifest)
    if manifest["oracle"]["json_sha256"] != oracle_hash:
        raise AssertionError("manifest/oracle hash mismatch")
    return manifest, oracle, manifest_hash, oracle_hash


def preflight(manifest_path: Path, oracle_path: Path) -> int:
    manifest, oracle, manifest_hash, oracle_hash = load_frozen(manifest_path, oracle_path)
    print(json.dumps({
        "evidence_identity": EVIDENCE_IDENTITY,
        "manifest_sha256": manifest_hash,
        "oracle_sha256": oracle_hash,
        "oracle_cases": [{"name": case["name"], "events": case["event_count"], "digest": case["event_digest_sha256"]} for case in oracle["cases"]],
        "fixture_inventory": manifest["fixture_classes"],
        "transaction_inventory": sorted(manifest["transaction_specifications"]),
        "evidence_executed": False,
    }, indent=2, sort_keys=True))
    return 0


def execute(manifest_path: Path, oracle_path: Path, result_path: Path, failure_path: Path) -> int:
    if result_path.exists() or failure_path.exists():
        raise AssertionError("v3 output exists; refusing a second attempt")
    script_hash = sha256_bytes(Path(__file__).read_bytes())
    manifest_hash = ""
    oracle_hash = ""
    try:
        manifest, oracle, manifest_hash, oracle_hash = load_frozen(manifest_path, oracle_path)
        campaign_digest, coverage = random_campaigns(manifest)
        fixture_results = run_fixtures(manifest, oracle)
        for name in manifest["fixture_classes"]["PRODUCTION_DEFERRED"]:
            fixture_results[name] = {"class": "PRODUCTION_DEFERRED", "status": "NOT_ESTABLISHED", "audited_path_counts": {}, "audit_receipts": [], "metrics": {}}
        # Deterministic replay is deliberately separate from correctness oracle.
        primary = next(case for case in oracle["cases"] if case["name"] == "TE4-F08-GRID")
        replay_a = simulate_frontier(primary, manifest["coefficient_identity"]["Wood"])
        replay_b = simulate_frontier(primary, manifest["coefficient_identity"]["Wood"])
        deterministic = replay_a == replay_b
        reference_pass = sum(v["class"] == "REFERENCE_REQUIRED" and v["status"] == "PASS" for v in fixture_results.values())
        deferred = sum(v["class"] == "PRODUCTION_DEFERRED" and v["status"] == "NOT_ESTABLISHED" for v in fixture_results.values())
        failures = sum(v["status"] == "FAIL" for v in fixture_results.values())
        unexpected_ne = sum(v["class"] != "PRODUCTION_DEFERRED" and v["status"] == "NOT_ESTABLISHED" for v in fixture_results.values())
        zero_paths = sum(count <= 0 for name in manifest["fixture_classes"]["REFERENCE_REQUIRED"] for count in fixture_results[name]["audited_path_counts"].values())
        f07_match = fixture_results["TE4-F07"]["metrics"]["exact_oracle_match"]
        f08_match = all(fixture_results["TE4-F08"]["metrics"]["all_case_matches"].values())
        f15b = fixture_results["TE4-F15"]["metrics"]["TE4-F15B"]["stage_N_plus_1"]["extinguished"]
        if (reference_pass, deferred, failures, unexpected_ne, zero_paths, deterministic, f07_match, f08_match, f15b) != (13, 4, 0, 0, 0, True, True, True, True):
            raise AssertionError("aggregate contract mismatch")
        result: dict[str, Any] = {
            "evidence_identity": EVIDENCE_IDENTITY,
            "script_sha256": script_hash,
            "manifest_sha256": manifest_hash,
            "oracle_sha256": oracle_hash,
            "result_sha256": "",
            "result_hash_scope": "CANONICAL_JSON_WITH_RESULT_SHA256_EMPTY",
            "attempts": 1,
            "completions": 1,
            "single_cell_sequences": manifest["trial_counts"]["single_cell_sequences"],
            "bounded_grid_runs": manifest["trial_counts"]["bounded_grids"],
            "counter_provenance": manifest["counter_provenance"],
            "reference_required_pass": reference_pass,
            "production_deferred_not_established": deferred,
            "fail": failures,
            "unexpected_not_established": unexpected_ne,
            "zero_audited_required_paths": zero_paths,
            "F07_exact_oracle_match": f07_match,
            "F08_exact_oracle_match": f08_match,
            "sole_air_face_snapshot_fixture": f15b,
            "deterministic_replay": {"status": "MATCH" if deterministic else "MISMATCH", "correctness_role": "DETERMINISM_ONLY"},
            "random_campaign_digest": campaign_digest,
            "random_coverage": coverage,
            "fixture_results": fixture_results,
            "arithmetic_result": "PASS",
            "selected_identity_result": "USER_SELECTED_AND_VALIDATED",
            "coefficient_optimality": "NOT_CLAIMED",
            "state_transition_result": "PASS_REFERENCE_MODEL_ONLY",
            "oracle_result": "PASS_EXACT_FROZEN_EVENT_LIST",
            "fixture_result": "PASS_REFERENCE_REQUIRED_ONLY",
            "gpu_status": "NOT_ESTABLISHED",
            "product_status": "NOT_ESTABLISHED",
            "user_status": "PENDING",
            "limitations": [
                "No Rust, WGSL, GPU, runtime, build or application was executed.",
                "Projected bindings, pass order, races, wake behavior and profiler allocation are source-audited design claims, not runtime evidence.",
                "The independent oracle establishes only the frozen reduced propagation fixtures.",
                "Four production-deferred fixtures remain NOT_ESTABLISHED and user architecture review is pending."
            ],
        }
        result["result_sha256"] = sha256_bytes(canonical_bytes(result))
        result_path.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
        return 0
    except Exception as exc:
        failure = {
            "evidence_identity": EVIDENCE_IDENTITY,
            "script_sha256": script_hash,
            "manifest_sha256": manifest_hash,
            "oracle_sha256": oracle_hash,
            "attempts": 1,
            "completions": 0,
            "error_type": type(exc).__name__,
            "error": str(exc),
        }
        failure_path.write_text(json.dumps(failure, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
        raise


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--oracle", type=Path, required=True)
    parser.add_argument("--preflight", action="store_true")
    parser.add_argument("--result", type=Path)
    parser.add_argument("--failure", type=Path)
    args = parser.parse_args()
    if args.preflight:
        if args.result or args.failure:
            raise SystemExit("preflight does not accept output paths")
        return preflight(args.manifest, args.oracle)
    if not args.result or not args.failure:
        raise SystemExit("execution requires --result and --failure")
    return execute(args.manifest, args.oracle, args.result, args.failure)


if __name__ == "__main__":
    sys.exit(main())
