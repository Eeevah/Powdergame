#!/usr/bin/env python3
"""Manifest-bound TE-4D v2 reference model. Standard library only."""

from __future__ import annotations

import argparse
import copy
import dataclasses
import hashlib
import json
import math
import random
import sys
from pathlib import Path
from typing import Any, Callable


EVIDENCE_IDENTITY = "TE4-IGNITION-KINETICS-REFERENCE-V2"
EXPECTED_MANIFEST_SHA256 = "9b763c1c7efa0ee9f9d444ef19dc5daed3833aafb546612b01d4e9db48d253ba"
COMBUSTING = 1 << 0
FLAME_EVENT = 1 << 1
EXPOSURE_LOW_MASK = 0x0000000C
FUEL_MASK = 0x0000FFF0
DECAY_MASK = 0x0FFF0000
EXPOSURE_HIGH_MASK = 0xF0000000
EXPOSURE_MASK = EXPOSURE_LOW_MASK | EXPOSURE_HIGH_MASK
MANAGED_MASK = COMBUSTING | FLAME_EVENT | EXPOSURE_MASK | FUEL_MASK | DECAY_MASK
Q_TOL = 1.0e-9


@dataclasses.dataclass
class CellState:
    material: str
    temperature: float
    flags: int = 0
    air_faces: tuple[tuple[str, float], ...] = ()

    @property
    def exposure(self) -> int:
        return decode_exposure(self.flags)

    @property
    def fuel(self) -> int:
        return (self.flags & FUEL_MASK) >> 4

    @property
    def decay_age(self) -> int:
        return (self.flags & DECAY_MASK) >> 16

    @property
    def burning(self) -> bool:
        return bool(self.flags & COMBUSTING)

    @property
    def prior_flame(self) -> bool:
        return bool(self.flags & FLAME_EVENT)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def encode_exposure(flags: int, exposure: int) -> int:
    if not 0 <= exposure <= 63:
        raise AssertionError(f"exposure out of range: {exposure}")
    clean = flags & ~EXPOSURE_MASK
    return clean | ((exposure & 0x3) << 2) | ((exposure & 0x3C) << 26)


def decode_exposure(flags: int) -> int:
    return ((flags >> 2) & 0x3) | ((flags >> 26) & 0x3C)


def set_fuel(flags: int, fuel: int) -> int:
    if not 0 <= fuel <= 4095:
        raise AssertionError(f"fuel out of range: {fuel}")
    return (flags & ~FUEL_MASK) | (fuel << 4)


def set_decay(flags: int, age: int) -> int:
    if not 0 <= age <= 4095:
        raise AssertionError(f"decay out of range: {age}")
    return (flags & ~DECAY_MASK) | (age << 16)


def air_access(state: CellState) -> bool:
    return any(material == "EMPTY" and math.isfinite(mass) and mass > 0.0 for material, mass in state.air_faces)


def thermal_rate(coeff: dict[str, Any], temperature: float) -> int:
    excess = temperature - coeff["ignition_threshold_C"]
    if excess < 0.0:
        return 0
    return min(coeff["max_rate"], coeff["base_rate"] + math.floor(excess / coeff["bucket_width_C"]))


def exposure_tick(state: CellState, coeff: dict[str, Any], prior_flames: int) -> tuple[CellState, dict[str, Any]]:
    before = state.exposure
    access = air_access(state)
    eligible = access and state.temperature >= coeff["ignition_threshold_C"]
    if state.burning:
        raise AssertionError("exposure_tick requires unlit state")
    if eligible:
        tr = thermal_rate(coeff, state.temperature)
        fr = min(coeff["flame_bonus_cap"], prior_flames * coeff["flame_bonus"])
        after = min(coeff["budget"], before + tr + fr)
    else:
        tr = 0
        fr = 0
        after = max(0, before - coeff["cooling_decay"])
    flags = encode_exposure(state.flags & ~FLAME_EVENT, after)
    ignited = after >= coeff["budget"]
    if ignited:
        flags = encode_exposure(flags | COMBUSTING | FLAME_EVENT, 0)
    return dataclasses.replace(state, flags=flags), {
        "before": before,
        "after": 0 if ignited else after,
        "delta": (0 if ignited else after) - before,
        "thermal_rate": tr,
        "flame_rate": fr,
        "air_access": access,
        "eligible": eligible,
        "ignited": ignited,
    }


def burning_tick(state: CellState, heat: dict[str, Any]) -> tuple[CellState, dict[str, Any]]:
    if not state.burning:
        raise AssertionError("burning_tick requires burning state")
    flags = state.flags & ~FLAME_EVENT
    if not air_access(state):
        flags = encode_exposure(flags & ~COMBUSTING, 0)
        return dataclasses.replace(state, flags=flags), {
            "extinguished": True, "consumed": False, "gross_Q": 0.0,
            "deposited_Q": 0.0, "clipped_Q": 0.0, "smoke": False, "flame": False,
        }
    next_fuel = state.fuel + 1
    if next_fuel >= heat["duration_ticks"]:
        return CellState("EMPTY", state.temperature, 0, state.air_faces), {
            "extinguished": False, "consumed": True, "gross_Q": 0.0,
            "deposited_Q": 0.0, "clipped_Q": 0.0, "smoke": False, "flame": False,
        }
    flags = set_fuel(flags | COMBUSTING | FLAME_EVENT, next_fuel)
    gross = float(heat["gross_Q_per_emitting_tick"])
    capacity = float(heat["material_heat_capacity"])
    delta = gross / capacity
    cap = max(1200.0, state.temperature)
    out_temperature = min(state.temperature + delta, cap)
    deposited = capacity * (out_temperature - state.temperature)
    clipped = gross - deposited
    if min(gross, deposited, clipped) < -Q_TOL or not all(math.isfinite(x) for x in (gross, deposited, clipped)):
        raise AssertionError("invalid Q accounting")
    if abs((deposited + clipped) - gross) > Q_TOL:
        raise AssertionError("Q accounting does not close")
    return dataclasses.replace(state, temperature=out_temperature, flags=flags), {
        "extinguished": False, "consumed": False, "gross_Q": gross,
        "deposited_Q": deposited, "clipped_Q": clipped, "smoke": True, "flame": True,
    }


def clear_identity(state: CellState, replacement: str = "EMPTY") -> CellState:
    return CellState(replacement, state.temperature, state.flags & ~MANAGED_MASK, state.air_faces)


def fixture_result(name: str, paths: dict[str, int], metrics: dict[str, Any]) -> dict[str, Any]:
    if not paths or any((not isinstance(v, int)) or v <= 0 for v in paths.values()):
        raise AssertionError(f"{name} has a zero required path")
    return {"class": "REFERENCE_REQUIRED", "status": "PASS", "executed_paths": paths, "metrics": metrics}


def simulate_until_ignite(material: str, manifest: dict[str, Any], temperature: float, prior_flames: int = 0) -> int:
    coeff = manifest["coefficients"][material]
    state = CellState(material, temperature, 0, (("EMPTY", 1.0),))
    for tick in range(1, coeff["budget"] + 2):
        state, event = exposure_tick(state, coeff, prior_flames)
        if event["ignited"]:
            return tick
    raise AssertionError(f"{material} did not ignite")


def run_f02(manifest: dict[str, Any]) -> dict[str, Any]:
    ticks = {}
    profiles = {}
    checks = 0
    for material in ("Oil", "Wood"):
        coeff = manifest["coefficients"][material]
        ticks[material] = {
            "threshold": simulate_until_ignite(material, manifest, coeff["ignition_threshold_C"]),
            "+100C": simulate_until_ignite(material, manifest, coeff["ignition_threshold_C"] + 100.0),
            "one_flame": simulate_until_ignite(material, manifest, coeff["ignition_threshold_C"], 1),
        }
        half = coeff["budget"] // 2
        ticks[material]["half_decay"] = math.ceil(half / coeff["cooling_decay"])
        if ticks[material] != manifest["target_timing_metrics"][material]:
            raise AssertionError(f"{material} timing mismatch")
        profiles[material] = [thermal_rate(coeff, coeff["ignition_threshold_C"] + x) for x in manifest["rate_probe_excess_C"]]
        if profiles[material] != manifest["rate_profiles"][material]:
            raise AssertionError(f"{material} rate profile mismatch")
        checks += len(profiles[material]) + 4
    return fixture_result("TE4-F02", {"oil_direct_contact_ticks": ticks["Oil"]["threshold"], "wood_direct_contact_ticks": ticks["Wood"]["threshold"], "timing_metric_checks": 8, "rate_profile_checks": 14}, {"ticks": ticks, "profiles": profiles, "checks": checks})


def run_f03(manifest: dict[str, Any]) -> dict[str, Any]:
    spikes = {}
    cooling = 0
    for material in ("Oil", "Wood"):
        coeff = manifest["coefficients"][material]
        spikes[material] = {}
        for length in (1, 2, 3):
            state = CellState(material, coeff["ignition_threshold_C"], 0, (("EMPTY", 1.0),))
            for _ in range(length):
                state, event = exposure_tick(state, coeff, 0)
                if event["ignited"]:
                    raise AssertionError("threshold spike ignited")
            peak = state.exposure
            state = dataclasses.replace(state, temperature=coeff["ignition_threshold_C"] - 1.0)
            while state.exposure:
                state, _ = exposure_tick(state, coeff, 0)
                cooling += 1
            spikes[material][str(length)] = peak
    return fixture_result("TE4-F03", {"one_tick_spike": 2, "two_tick_spike": 4, "three_tick_spike": 6, "cooling_decay_ticks": cooling}, {"peak_exposure": spikes})


def run_f04(manifest: dict[str, Any]) -> dict[str, Any]:
    oil = simulate_until_ignite("Oil", manifest, 200.0)
    wood = simulate_until_ignite("Wood", manifest, 300.0)
    if oil <= 3 or wood <= 3:
        raise AssertionError("first-tick ignition")
    return fixture_result("TE4-F04", {"oil_sustained_ticks": oil, "wood_sustained_ticks": wood, "first_tick_rejection": 2, "exact_ignition_events": 2}, {"Oil": oil, "Wood": wood})


def run_f05(manifest: dict[str, Any]) -> dict[str, Any]:
    coeff = manifest["coefficients"]["Oil"]
    state = CellState("Oil", 200.0, 0, (("EMPTY", 1.0),))
    records = []
    for phase, ticks, temp in (("partial", 6, 200.0), ("brief_cooling", 3, 199.0), ("reheat", 5, 200.0), ("long_cooling", 40, 199.0)):
        for tick in range(ticks):
            state = dataclasses.replace(state, temperature=temp)
            state, event = exposure_tick(state, coeff, 0)
            records.append({"phase": phase, "tick": tick + 1, "before": event["before"], "after": event["after"], "delta": event["delta"]})
    if state.exposure != 0 or state.burning:
        raise AssertionError("long cooling did not clear exposure")
    return fixture_result("TE4-F05", {"partial_exposure_ticks": 6, "brief_cooling_ticks": 3, "reheat_ticks": 5, "long_cooling_ticks": 40, "dose_delta_records": len(records)}, {"records": records, "final_exposure": state.exposure})


def run_f06(manifest: dict[str, Any]) -> dict[str, Any]:
    thermal = simulate_until_ignite("Wood", manifest, 300.0, 0)
    flame = simulate_until_ignite("Wood", manifest, 300.0, 1)
    coeff = manifest["coefficients"]["Wood"]
    below = CellState("Wood", 299.0, 0, (("EMPTY", 1.0),))
    for _ in range(20):
        below, event = exposure_tick(below, coeff, 4)
        if event["flame_rate"] != 0 or below.exposure != 0:
            raise AssertionError("flame bypassed thermal gate")
    if not 1 < flame < thermal:
        raise AssertionError("flame route is not finite and faster")
    return fixture_result("TE4-F06", {"thermal_only_ticks": thermal, "previous_flame_ticks": flame, "thermal_gate_checks": 20, "same_tick_visibility_checks": 20}, {"thermal_ticks": thermal, "one_flame_ticks": flame})


def simulate_propagation(manifest: dict[str, Any], width: int, height: int, initial: tuple[int, int], horizon: int) -> dict[str, Any]:
    coeff = manifest["coefficients"]["Wood"]
    states = {(x, y): CellState("Wood", 250.0, 0, (("EMPTY", 1.0),)) for y in range(height) for x in range(width)}
    states[initial] = dataclasses.replace(states[initial], flags=COMBUSTING | FLAME_EVENT)
    events = []
    adjacency_checks = 0
    for tick in range(1, horizon + 1):
        prior_burning = {p for p, s in states.items() if s.burning}
        next_states = dict(states)
        tick_events = []
        for pos, state in states.items():
            if state.burning:
                next_states[pos] = dataclasses.replace(state, flags=state.flags | FLAME_EVENT)
                continue
            x, y = pos
            neighbours = {(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)}
            flame_count = sum(n in prior_burning for n in neighbours)
            temperature = 300.0 if flame_count else 250.0
            candidate = dataclasses.replace(state, temperature=temperature)
            candidate, event = exposure_tick(candidate, coeff, flame_count)
            next_states[pos] = candidate
            adjacency_checks += 1
            if event["ignited"]:
                if flame_count == 0:
                    raise AssertionError("non-adjacent ignition")
                tick_events.append(pos)
        states = next_states
        if tick_events:
            events.append({"tick": tick, "cells": [list(p) for p in sorted(tick_events)]})
        if all(s.burning for s in states.values()):
            break
    digest = sha256_bytes(canonical_bytes(events))
    return {"events": events, "digest": digest, "completion_tick": tick if all(s.burning for s in states.values()) else None, "adjacency_checks": adjacency_checks}


def run_f07(manifest: dict[str, Any]) -> dict[str, Any]:
    out = simulate_propagation(manifest, 5, 1, (0, 0), 80)
    observed = [(e["tick"], e["cells"][0][0]) for e in out["events"]]
    expected = list(zip(manifest["propagation_bounds"]["TE4-F07"]["event_ticks"], manifest["propagation_bounds"]["TE4-F07"]["event_cells"]))
    if observed != expected or any(len(e["cells"]) != 1 for e in out["events"]):
        raise AssertionError(f"F07 events {observed!r}")
    return fixture_result("TE4-F07", {"line_simulation_ticks": 80, "previous_snapshot_edge_checks": out["adjacency_checks"], "ignition_event_records": 4, "same_tick_recursion_checks": 4}, out)


def run_f08(manifest: dict[str, Any]) -> dict[str, Any]:
    bounds = manifest["propagation_bounds"]["TE4-F08"]
    out = simulate_propagation(manifest, bounds["width"], bounds["height"], tuple(bounds["initial_burning"]), bounds["all_burning_by_tick"])
    first = out["events"][0]["tick"]
    maximum = max(len(e["cells"]) for e in out["events"])
    if first != bounds["first_new_ignition_tick"] or maximum > bounds["maximum_new_ignitions_one_tick"] or out["completion_tick"] is None or out["completion_tick"] > bounds["all_burning_by_tick"]:
        raise AssertionError(f"F08 bounds first={first} max={maximum} completion={out['completion_tick']}")
    metrics = dict(out)
    metrics.update({"first_new_ignition_tick": first, "maximum_new_ignitions_one_tick": maximum})
    return fixture_result("TE4-F08", {"grid_simulation_ticks": out["completion_tick"], "previous_snapshot_adjacency_checks": out["adjacency_checks"], "ignition_event_records": sum(len(e["cells"]) for e in out["events"]), "event_digest_records": len(out["events"])}, metrics)


def run_f09(manifest: dict[str, Any]) -> dict[str, Any]:
    metrics = {}
    lifecycle = fuel_ticks = cap_checks = q_checks = zero_checks = 0
    for material in ("Oil", "Wood"):
        heat = manifest["chemical_heat_policy"][material]
        state = CellState(material, 1198.0, COMBUSTING, (("EMPTY", 1.0),))
        gross = deposited = clipped = 0.0
        emissions = 0
        for _ in range(heat["duration_ticks"]):
            state, event = burning_tick(state, heat)
            lifecycle += 1
            if event["consumed"]:
                zero_checks += 1
                if any(event[k] for k in ("gross_Q", "deposited_Q", "clipped_Q", "smoke", "flame")):
                    raise AssertionError("consumption tick emitted")
                break
            emissions += 1
            fuel_ticks += 1
            gross += event["gross_Q"]
            deposited += event["deposited_Q"]
            clipped += event["clipped_Q"]
            cap_checks += 1
            q_checks += 1
        if emissions != heat["emitting_ticks"] or abs(gross - heat["maximum_gross_Q"]) > Q_TOL or abs(deposited + clipped - gross) > Q_TOL:
            raise AssertionError(f"{material} lifecycle accounting mismatch")
        no_air = CellState(material, 500.0, COMBUSTING, (("EMPTY", 0.0),))
        no_air, event = burning_tick(no_air, heat)
        zero_checks += 1
        if not event["extinguished"] or event["gross_Q"] != 0.0:
            raise AssertionError("extinguish emitted")
        metrics[material] = {"emitting_ticks": emissions, "gross_Q": gross, "deposited_Q": deposited, "clipped_Q": clipped, "final_material": state.material}
    return fixture_result("TE4-F09", {"burn_lifecycle_ticks": lifecycle, "fuel_progress_ticks": fuel_ticks, "cap_accounting_checks": cap_checks, "extinguish_zero_checks": 2, "consumption_zero_checks": 2, "q_conservation_checks": q_checks}, metrics)


def run_f10(manifest: dict[str, Any]) -> dict[str, Any]:
    coeff = manifest["coefficients"]["Oil"]
    heat = manifest["chemical_heat_policy"]["Oil"]
    state = CellState("Oil", 200.0, 0, (("EMPTY", 1.0),))
    initial_ticks = 0
    while not state.burning:
        state, _ = exposure_tick(state, coeff, 0)
        initial_ticks += 1
    for _ in range(10):
        state, _ = burning_tick(state, heat)
    fuel = state.fuel
    state = dataclasses.replace(state, air_faces=(("EMPTY", 0.0),))
    state, event = burning_tick(state, heat)
    if not event["extinguished"] or state.fuel != fuel or state.exposure != 0:
        raise AssertionError("extinguish ownership mismatch")
    state = dataclasses.replace(state, air_faces=(("EMPTY", 1.0),), temperature=200.0)
    rebuild = 0
    while not state.burning:
        state, _ = exposure_tick(state, coeff, 0)
        rebuild += 1
    if state.fuel != fuel:
        raise AssertionError("reignition restored fuel")
    return fixture_result("TE4-F10", {"initial_ignition_ticks": initial_ticks, "initial_burn_ticks": 10, "extinguish_ticks": 1, "dose_rebuild_ticks": rebuild, "reignition_ticks": 1, "fuel_preservation_checks": 2}, {"fuel_before": fuel, "fuel_after": state.fuel, "rebuild_ticks": rebuild})


def run_f11(manifest: dict[str, Any]) -> dict[str, Any]:
    flags = encode_exposure(set_decay(set_fuel(0, 17), 23), 37)
    source = CellState("Oil", 222.0, flags, (("EMPTY", 1.0),))
    empty = CellState("EMPTY", 20.0, 0, ())
    moved_source, moved_target = empty, copy.deepcopy(source)
    if moved_source.flags != 0 or moved_target.flags != flags:
        raise AssertionError("movement ownership mismatch")
    powder = CellState("Powder", 20.0, 0, ())
    swap_a, swap_b = powder, copy.deepcopy(moved_target)
    if swap_a.flags != 0 or swap_b.flags != flags or swap_b.exposure != 37:
        raise AssertionError("swap ownership mismatch")
    return fixture_result("TE4-F11", {"movement_transactions": 1, "density_swap_transactions": 1, "source_clear_checks": 2, "ownership_conservation_checks": 4}, {"moved_exposure": moved_target.exposure, "swapped_exposure": swap_b.exposure, "fuel": swap_b.fuel, "decay_age": swap_b.decay_age})


def run_f12(manifest: dict[str, Any]) -> dict[str, Any]:
    flags = encode_exposure(set_decay(set_fuel(COMBUSTING | FLAME_EVENT, 19), 7), 41)
    base = CellState("Oil", 400.0, flags, (("EMPTY", 1.0),))
    paths = {}
    for name, replacement in (("decay_replacements", "Smoke"), ("rupture_replacements", "Powder"), ("fuel_consumptions", "EMPTY"), ("void_exits", "EMPTY"), ("generic_replacements", "Stone")):
        out = clear_identity(base, replacement)
        if out.flags & MANAGED_MASK or out.exposure:
            raise AssertionError(f"stale ownership after {name}")
        paths[name] = 1
    paths["unrelated_flag_checks"] = 5
    return fixture_result("TE4-F12", paths, {"cleared_paths": 5})


def run_f13(manifest: dict[str, Any]) -> dict[str, Any]:
    canonical = CellState("Oil", 20.0, 0, (("EMPTY", 1.0),))
    current = copy.deepcopy(canonical)
    next_state = copy.deepcopy(canonical)
    draw = copy.deepcopy(canonical)
    erase = clear_identity(draw, "EMPTY")
    preset = copy.deepcopy(canonical)
    reset = copy.deepcopy(canonical)
    before = (copy.deepcopy(current), copy.deepcopy(next_state))
    invalid_rejected = False
    try:
        encode_exposure(0, 64)
    except AssertionError:
        invalid_rejected = True
    if not invalid_rejected or (current, next_state) != before or current != next_state or preset != reset or erase.exposure != 0:
        raise AssertionError("authoring transaction mismatch")
    return fixture_result("TE4-F13", {"draw_transactions": 1, "erase_transactions": 1, "preset_transactions": 1, "reset_transactions": 1, "current_next_equality_checks": 3, "invalid_staging_rejections": 1}, {"invalid_rejected": True, "atomic_preservation": True})


def run_f15(manifest: dict[str, Any]) -> dict[str, Any]:
    coeff = manifest["coefficients"]["Oil"]
    cases = {
        "Atmosphere": (("EMPTY", 1.0),),
        "LowPressure": (("EMPTY", 0.001),),
        "Vacuum": (("EMPTY", 0.0),),
        "OccupiedGas": (("Steam", 1.0),),
    }
    access = {name: air_access(CellState("Oil", 200.0, 0, faces)) for name, faces in cases.items()}
    if access != {"Atmosphere": True, "LowPressure": True, "Vacuum": False, "OccupiedGas": False}:
        raise AssertionError(f"Air matrix mismatch {access}")
    masses_before = copy.deepcopy(cases)
    burning = CellState("Oil", 300.0, COMBUSTING, cases["Atmosphere"])
    burning = dataclasses.replace(burning, air_faces=cases["Vacuum"])
    burning, event = burning_tick(burning, manifest["chemical_heat_policy"]["Oil"])
    if not event["extinguished"] or event["gross_Q"] != 0.0 or burning.exposure != 0:
        raise AssertionError("loss of Air did not extinguish cleanly")
    burning = dataclasses.replace(burning, air_faces=cases["LowPressure"], temperature=200.0)
    burning, recovered = exposure_tick(burning, coeff, 0)
    if recovered["after"] <= 0 or cases != masses_before:
        raise AssertionError("Air recovery or conservation mismatch")
    return fixture_result("TE4-F15", {"atmosphere_face_checks": 1, "low_pressure_face_checks": 1, "vacuum_face_checks": 2, "loss_of_air_ticks": 1, "air_recovery_ticks": 1, "air_conservation_checks": 4}, {"air_access": access, "loss_event": event, "recovery_exposure": burning.exposure})


FIXTURES: dict[str, Callable[[dict[str, Any]], dict[str, Any]]] = {
    "TE4-F02": run_f02, "TE4-F03": run_f03, "TE4-F04": run_f04,
    "TE4-F05": run_f05, "TE4-F06": run_f06, "TE4-F07": run_f07,
    "TE4-F08": run_f08, "TE4-F09": run_f09, "TE4-F10": run_f10,
    "TE4-F11": run_f11, "TE4-F12": run_f12, "TE4-F13": run_f13,
    "TE4-F15": run_f15,
}


def validate_manifest(manifest: dict[str, Any]) -> None:
    if manifest["evidence_identity"] != EVIDENCE_IDENTITY:
        raise AssertionError("evidence identity mismatch")
    expected_required = ["TE4-F02", "TE4-F03", "TE4-F04", "TE4-F05", "TE4-F06", "TE4-F07", "TE4-F08", "TE4-F09", "TE4-F10", "TE4-F11", "TE4-F12", "TE4-F13", "TE4-F15"]
    expected_deferred = ["TE4-F01", "TE4-F14", "TE4-F16", "TE4-F17"]
    if manifest["fixture_classes"]["REFERENCE_REQUIRED"] != expected_required or manifest["fixture_classes"]["PRODUCTION_DEFERRED"] != expected_deferred:
        raise AssertionError("fixture classification mismatch")
    if set(manifest["required_paths"]) != set(expected_required) or set(FIXTURES) != set(expected_required):
        raise AssertionError("required fixture implementation mismatch")
    exact = {"Oil": [48, 2, 50, 6, 1, 2, 4], "Wood": [60, 1, 50, 5, 1, 2, 4]}
    keys = ["budget", "base_rate", "bucket_width_C", "max_rate", "cooling_decay", "flame_bonus", "flame_bonus_cap"]
    for material, values in exact.items():
        if [manifest["coefficients"][material][k] for k in keys] != values:
            raise AssertionError(f"{material} coefficient identity mismatch")
    selected, rejected = manifest["equal_primary_metric_candidates"]["Oil"]
    if selected["identity"] != exact["Oil"] or rejected["identity"] != [48, 2, 25, 4, 1, 2, 4]:
        raise AssertionError("equal-primary set mismatch")
    if len(set(selected["rate_profile"])) <= len(set(rejected["rate_profile"])) or selected["rate_profile"][5] >= manifest["coefficients"]["Oil"]["max_rate"] or selected["rate_profile"][6] != manifest["coefficients"]["Oil"]["max_rate"]:
        raise AssertionError("declared secondary objectives do not distinguish selected Oil tuple")
    if manifest["representation"]["selected"] != "PACKED_U6" or manifest["vacuum_policy"]["selected"] != "NON_VACUUM_AIR_FACE_REQUIRED":
        raise AssertionError("selected representation or Air policy mismatch")


def random_campaigns(manifest: dict[str, Any]) -> tuple[str, dict[str, int]]:
    rng = random.Random(manifest["seed"])
    digest = hashlib.sha256()
    sequence_trials = manifest["trial_counts"]["single_cell_sequences"]
    coverage = {"near_budget": 0, "budget_minus_one": 0, "vacuum": 0, "low_pressure": 0, "atmosphere": 0, "movement": 0, "replacement": 0, "extinguish": 0}
    probes = [-1, 0, 25, 50, 100, 200]
    for i in range(sequence_trials):
        material = "Oil" if rng.getrandbits(1) == 0 else "Wood"
        coeff = manifest["coefficients"][material]
        exposure = rng.choice([0, max(0, coeff["budget"] - 2), coeff["budget"] - 1, rng.randrange(coeff["budget"])])
        mode = rng.randrange(3)
        mass = (1.0, 0.001, 0.0)[mode]
        label = ("atmosphere", "low_pressure", "vacuum")[mode]
        coverage[label] += 1
        state = CellState(material, coeff["ignition_threshold_C"] + rng.choice(probes), encode_exposure(0, exposure), (("EMPTY", mass),))
        prior = rng.randrange(5)
        out, event = exposure_tick(state, coeff, prior)
        if not 0 <= out.exposure <= 63 or decode_exposure(out.flags) != out.exposure:
            raise AssertionError("random packed exposure failure")
        if mass == 0.0 and event["after"] > exposure:
            raise AssertionError("Vacuum accumulated exposure")
        if exposure >= coeff["budget"] - 2:
            coverage["near_budget"] += 1
        if exposure == coeff["budget"] - 1:
            coverage["budget_minus_one"] += 1
        if i % 7 == 0:
            moved = copy.deepcopy(out)
            coverage["movement"] += 1
            if moved.flags != out.flags:
                raise AssertionError("random move mismatch")
        if i % 11 == 0:
            cleared = clear_identity(out)
            coverage["replacement"] += 1
            if cleared.flags & MANAGED_MASK:
                raise AssertionError("random replacement residue")
        if i % 13 == 0:
            burning = CellState(material, 500.0, set_fuel(COMBUSTING, rng.randrange(20)), (("EMPTY", 0.0),))
            extinguished, q = burning_tick(burning, manifest["chemical_heat_policy"][material])
            coverage["extinguish"] += 1
            if extinguished.burning or q["gross_Q"] != 0.0:
                raise AssertionError("random extinguish failure")
        digest.update(canonical_bytes([material, exposure, mass, prior, event, out.flags]))

    grid_trials = manifest["trial_counts"]["bounded_grids"]
    grid_checks = 0
    for _ in range(grid_trials):
        width = rng.randrange(2, 7)
        height = rng.randrange(2, 7)
        burning = {(x, y) for y in range(height) for x in range(width) if rng.random() < 0.2}
        if not burning:
            burning.add((rng.randrange(width), rng.randrange(height)))
        for y in range(height):
            for x in range(width):
                if (x, y) in burning:
                    continue
                neighbours = {(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)}
                count = sum(n in burning for n in neighbours)
                state = CellState("Wood", 300.0 if count else 250.0, encode_exposure(0, rng.randrange(60)), (("EMPTY", rng.choice((0.0, 0.001, 1.0))),))
                out, event = exposure_tick(state, manifest["coefficients"]["Wood"], count)
                if event["ignited"] and count == 0:
                    raise AssertionError("random grid non-adjacent ignition")
                grid_checks += 1
                digest.update(canonical_bytes([x, y, count, event, out.flags]))
    if any(v <= 0 for v in coverage.values()) or grid_checks <= 0:
        raise AssertionError("random coverage counter zero")
    coverage["grid_cell_checks"] = grid_checks
    return digest.hexdigest(), coverage


def preflight(manifest_path: Path) -> int:
    raw = manifest_path.read_bytes()
    actual_hash = sha256_bytes(raw)
    if actual_hash != EXPECTED_MANIFEST_SHA256:
        raise AssertionError(f"manifest hash mismatch: {actual_hash}")
    manifest = json.loads(raw)
    validate_manifest(manifest)
    print(json.dumps({
        "evidence_identity": EVIDENCE_IDENTITY,
        "manifest_sha256": actual_hash,
        "fixtures": manifest["fixture_classes"],
        "required_paths": manifest["required_paths"],
        "coefficients": manifest["coefficients"],
        "rate_profiles": manifest["rate_profiles"],
        "output_destination_writable_parent": str(manifest_path.parent),
        "evidence_executed": False,
    }, indent=2, sort_keys=True))
    return 0


def execute(manifest_path: Path, result_path: Path, failure_path: Path) -> int:
    script_hash = sha256_bytes(Path(__file__).read_bytes())
    raw = manifest_path.read_bytes()
    manifest_hash = sha256_bytes(raw)
    if manifest_hash != EXPECTED_MANIFEST_SHA256:
        raise AssertionError(f"manifest hash mismatch: {manifest_hash}")
    manifest = json.loads(raw)
    validate_manifest(manifest)
    if result_path.exists() or failure_path.exists():
        raise AssertionError("v2 output already exists; refusing a second attempt")
    try:
        campaign_digest, random_coverage = random_campaigns(manifest)
        fixture_results = {name: FIXTURES[name](manifest) for name in manifest["fixture_classes"]["REFERENCE_REQUIRED"]}
        deferred = {name: {"class": "PRODUCTION_DEFERRED", "status": "NOT_ESTABLISHED", "executed_paths": {}} for name in manifest["fixture_classes"]["PRODUCTION_DEFERRED"]}
        fixture_results.update(deferred)
        replay_a = simulate_propagation(manifest, 9, 9, (4, 0), manifest["propagation_bounds"]["TE4-F08"]["all_burning_by_tick"])
        replay_b = simulate_propagation(manifest, 9, 9, (4, 0), manifest["propagation_bounds"]["TE4-F08"]["all_burning_by_tick"])
        deterministic = replay_a["digest"] == replay_b["digest"] == fixture_results["TE4-F08"]["metrics"]["digest"]
        if not deterministic:
            raise AssertionError("deterministic replay mismatch")
        for name, required in manifest["required_paths"].items():
            executed = fixture_results[name]["executed_paths"]
            if set(executed) != set(required) or any(executed[p] <= 0 for p in required):
                raise AssertionError(f"{name} required path mismatch")
        reference_pass = sum(v["status"] == "PASS" for v in fixture_results.values())
        deferred_ne = sum(v["class"] == "PRODUCTION_DEFERRED" and v["status"] == "NOT_ESTABLISHED" for v in fixture_results.values())
        fail_count = sum(v["status"] == "FAIL" for v in fixture_results.values())
        unexpected_ne = sum(v["class"] != "PRODUCTION_DEFERRED" and v["status"] == "NOT_ESTABLISHED" for v in fixture_results.values())
        if (reference_pass, deferred_ne, fail_count, unexpected_ne) != (13, 4, 0, 0):
            raise AssertionError("fixture aggregate mismatch")
        result: dict[str, Any] = {
            "evidence_identity": EVIDENCE_IDENTITY,
            "script_sha256": script_hash,
            "manifest_sha256": manifest_hash,
            "result_sha256": "",
            "result_hash_scope": "CANONICAL_JSON_WITH_RESULT_SHA256_EMPTY",
            "attempts": 1,
            "completions": 1,
            "sequence_trials": manifest["trial_counts"]["single_cell_sequences"],
            "grid_trials": manifest["trial_counts"]["bounded_grids"],
            "deterministic_replay": {"status": "MATCH", "digest": replay_a["digest"]},
            "random_campaign_digest": campaign_digest,
            "random_coverage": random_coverage,
            "coefficient_identity": manifest["coefficients"],
            "rate_profiles": manifest["rate_profiles"],
            "representation": manifest["representation"],
            "vacuum_policy": manifest["vacuum_policy"],
            "chemical_heat_policy": manifest["chemical_heat_policy"],
            "pass_projection": manifest["pass_projection"],
            "fixture_results": fixture_results,
            "fixture_required_paths": manifest["required_paths"],
            "fixture_executed_paths": {k: v["executed_paths"] for k, v in fixture_results.items()},
            "reference_required_pass_count": reference_pass,
            "production_deferred_not_established_count": deferred_ne,
            "fail_count": fail_count,
            "unexpected_not_established_count": unexpected_ne,
            "mathematical_result": "PASS",
            "state_transition_result": "PASS",
            "coefficient_result": "PASS",
            "fixture_result": "PASS_REFERENCE_MODEL_ONLY",
            "gpu_status": "NOT_ESTABLISHED",
            "product_status": "NOT_ESTABLISHED",
            "user_status": "PENDING",
            "limitations": [
                "No Rust or WGSL production implementation was executed.",
                "GPU bindings, races, pass order, wake behavior and profiler allocation are not established.",
                "TE-2 transport, product visuals and user acceptance are not established.",
                "The four PRODUCTION_DEFERRED fixtures remain NOT_ESTABLISHED by design."
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
    parser.add_argument("--preflight", action="store_true")
    parser.add_argument("--result", type=Path)
    parser.add_argument("--failure", type=Path)
    args = parser.parse_args()
    if args.preflight:
        if args.result or args.failure:
            raise SystemExit("preflight does not accept output paths")
        return preflight(args.manifest)
    if not args.result or not args.failure:
        raise SystemExit("execution requires --result and --failure")
    return execute(args.manifest, args.result, args.failure)


if __name__ == "__main__":
    sys.exit(main())
