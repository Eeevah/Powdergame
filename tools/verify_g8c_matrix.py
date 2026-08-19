#!/usr/bin/env python3
"""Independent verifier for one G8-C official-matrix run and ZIP package.

This module intentionally does not import the capture coordinator.  It parses
the immutable run, streams the large benchmark CSV files, reconstructs every
reported statistic, validates the receipt/hash/source/binary bindings, and
then checks that the ZIP64 delivery is an exact, safe copy of the run tree.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import math
import os
import re
import stat
import subprocess
import sys
import zipfile
from collections import Counter, defaultdict
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Any, BinaryIO, Iterator, Mapping, NoReturn, Sequence, TextIO


MATRIX_SCHEMA = "powdergame-g8c-official-matrix-v1"
HEADLESS_SCHEMA = "powdergame-g8c-headless-v1"
INNER_HEADLESS_SCHEMA = "powdergame-g8b-fixture-v1"
COEXISTENCE_SCHEMA = "powdergame-g8c-coexistence-v1"
RENDER_PROFILE_SCHEMA = "powdergame-g8c-render-profile-v1"
SOURCE_INPUT_SCHEMA = "powdergame-g8c-source-input-v1"
REPLAY_INPUT_SCHEMA = "powdergame-g8c-aggregation-replay-input-v1"
PROCESS_SCHEMA = "powdergame-g8c-process-v1"
RECEIPT_SCHEMA = "powdergame-g8c-matrix-receipt-v1"
VERIFICATION_SCHEMA = "powdergame-g8c-independent-verification-v1"
REPORT_SCHEMA = "powdergame-g8c-matrix-report-v1"
REQUIRED_BRANCH = "feature/m0-g8c-official-matrix"
REPLAY_SOURCE_PILOT_ID = "g8c-pilot-8ee1ae238c32-6341f4f59218"
REPLAY_SOURCE_PILOT_PATH = Path(
    r"C:\Users\mdkap\source\Powdergame-artifacts\scratch\g8c-pilot-8ee1ae238c32-6341f4f59218"
)

SCENARIOS = (
    "sand-fall",
    "water-flow",
    "fire-heat",
    "pressure-burst",
    "heavy-mixed-world",
)

PASS_NAMES = (
    "activity_wake",
    "movement_propose",
    "movement_claim",
    "movement_commit",
    "thermal",
    "phase_transition",
    "expansion_claim",
    "expansion_spawn_commit",
    "expansion_pressure",
    "decay",
    "combustion",
    "smoke_claim",
    "smoke_commit",
    "pressure",
    "rupture",
    "activity_propose",
    "activity_reduce",
)

GROUPS: Mapping[str, tuple[str, ...]] = {
    "matter_movement": ("movement_propose", "movement_commit"),
    "ownership_claim": ("movement_claim", "expansion_claim", "smoke_claim"),
    "thermal_conduction": ("thermal",),
    "reaction_phase": (
        "phase_transition",
        "expansion_spawn_commit",
        "expansion_pressure",
        "decay",
        "combustion",
        "smoke_commit",
    ),
    "pressure_structure": ("pressure", "rupture"),
    "active_sleep_management": (
        "activity_wake",
        "activity_propose",
        "activity_reduce",
    ),
}

GROUP_DEFINITION = (
    "matter_movement=movement_propose+movement_commit;"
    "ownership_claim=movement_claim+expansion_claim+smoke_claim;"
    "thermal_conduction=thermal;"
    "reaction_phase=phase_transition+expansion_spawn_commit+expansion_pressure+"
    "decay+combustion+smoke_commit;pressure_structure=pressure+rupture;"
    "active_sleep_management=activity_wake+activity_propose+activity_reduce"
)

GROUP_LABELS: Mapping[str, str] = {
    "matter_movement": "Matter Movement",
    "ownership_claim": "Claim / Resolve",
    "thermal_conduction": "Thermal",
    "reaction_phase": "Reaction / Phase",
    "pressure_structure": "Pressure / Structure",
    "active_sleep_management": "Active / Sleep",
}

SUMMARY_HEADER = (
    "schema_version",
    "run_id",
    "commit_sha",
    "git_state",
    "adapter_name",
    "vendor_id",
    "device_id",
    "device_type",
    "backend",
    "driver",
    "driver_info",
    "profiling_enabled",
    "timestamp_period_ns",
    "build_profile",
    "width",
    "height",
    "chunk_size",
    "sleep_enabled",
    "sleep_threshold",
    "prewarm_requested_secs",
    "prewarm_ticks",
    "measurement_mode",
    "selection",
    "trial",
    "tick_start",
    "tick_end",
    "metric_type",
    "name",
    "value",
    "count",
    "p50",
    "p95",
    "mean",
    "min",
    "max",
    "unit",
    "method_note",
)

RAW_TICK_HEADER = (
    SUMMARY_HEADER[:22]
    + ("trial", "sample_id", "tick_index", "tick_start", "tick_end")
    + tuple(
        item
        for name in PASS_NAMES
        for item in (f"{name}_start_tick", f"{name}_end_tick")
    )
    + tuple(f"pass_{name}_ms" for name in PASS_NAMES)
    + tuple(f"group_{name}_ms" for name in GROUPS)
    + (
        "gpu_pass_sum_ms",
        "gpu_tick_envelope_ms",
        "residual_ms",
        "timestamp_unit",
        "duration_unit",
        "group_definition",
    )
)

RAW_CELL_HEADER = (
    "schema_version",
    "run_id",
    "commit_sha",
    "git_state",
    "census_tick",
    "index",
    "activity_mask",
)

RAW_CHUNK_HEADER = RAW_CELL_HEADER + ("chunk_state",)

COEXISTENCE_HEADER = (
    "schema_version",
    "scenario",
    "trial",
    "frame_index",
    "sim_tick",
    "window_elapsed_ms",
    "frame_wall_ms",
    "scheduled_sim_ticks",
    "sim_ticks_executed",
    "catch_up_ticks",
    "missed_simulation_deadlines",
    "presented",
    "surface_error",
)

RENDER_PROFILE_HEADER = COEXISTENCE_HEADER[:-1] + (
    "gpu_start_tick",
    "gpu_end_tick",
    "gpu_render_ms",
    "timestamp_period_ns",
    "surface_error",
)

HEX64 = re.compile(r"^[0-9a-f]{64}$")
OID40 = re.compile(r"^[0-9a-f]{40}$")
HASH_LINE = re.compile(r"^([0-9a-f]{64})  ([^\r\n]+)$")
FLOAT_ABS_TOLERANCE = 5.0e-7
FLOAT_REL_TOLERANCE = 5.0e-7
ACTIVITY_BITS = (1, 2, 4, 8)


class VerificationError(RuntimeError):
    """Raised for any integrity, schema, or arithmetic mismatch."""


def _fail(message: str) -> NoReturn:
    raise VerificationError(message)


def _safe_relative(value: str, label: str = "path") -> str:
    if not isinstance(value, str) or not value or "\\" in value or "\x00" in value:
        _fail(f"{label} is not a canonical POSIX relative path: {value!r}")
    path = PurePosixPath(value)
    if (
        path.is_absolute()
        or path.as_posix() != value
        or any(part in {"", ".", ".."} for part in path.parts)
    ):
        _fail(f"{label} is not a canonical POSIX relative path: {value!r}")
    return value


def _sha256_stream(stream: BinaryIO) -> tuple[str, int]:
    digest = hashlib.sha256()
    size = 0
    while True:
        block = stream.read(1024 * 1024)
        if not block:
            break
        digest.update(block)
        size += len(block)
    return digest.hexdigest(), size


def sha256_file(path: Path) -> str:
    with path.open("rb") as stream:
        return _sha256_stream(stream)[0]


def _read_json_bytes(data: bytes, label: str) -> Mapping[str, Any]:
    try:
        value = json.loads(data.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        _fail(f"invalid UTF-8 JSON in {label}: {error}")
    if not isinstance(value, dict):
        _fail(f"{label} must contain one JSON object")
    return value


def _read_json_file(path: Path, label: str) -> Mapping[str, Any]:
    try:
        data = path.read_bytes()
    except OSError as error:
        _fail(f"cannot read {label} at {path}: {error}")
    return _read_json_bytes(data, label)


def _mapping(value: Any, label: str) -> Mapping[str, Any]:
    if not isinstance(value, dict):
        _fail(f"{label} must be an object")
    return value


def _sequence(value: Any, label: str) -> Sequence[Any]:
    if not isinstance(value, list):
        _fail(f"{label} must be an array")
    return value


def _string(value: Any, label: str, *, nonempty: bool = True) -> str:
    if not isinstance(value, str) or (nonempty and not value):
        _fail(f"{label} must be {'a nonempty ' if nonempty else 'a '}string")
    return value


def _integer(value: Any, label: str, *, minimum: int | None = None) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        _fail(f"{label} must be an integer")
    if minimum is not None and value < minimum:
        _fail(f"{label} must be >= {minimum}, got {value}")
    return value


def _number(value: Any, label: str, *, minimum: float | None = None) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        _fail(f"{label} must be numeric")
    result = float(value)
    if not math.isfinite(result):
        _fail(f"{label} must be finite")
    if minimum is not None and result < minimum:
        _fail(f"{label} must be >= {minimum}, got {result}")
    return result


def _boolean(value: Any, label: str) -> bool:
    if not isinstance(value, bool):
        _fail(f"{label} must be boolean")
    return value


def _field(record: Mapping[str, Any], names: Sequence[str], label: str) -> Any:
    present = [name for name in names if name in record]
    if len(present) != 1:
        _fail(f"{label} must contain exactly one of {list(names)}, found {present}")
    return record[present[0]]


def _path_field(record: Mapping[str, Any], names: Sequence[str], label: str) -> str:
    value = _field(record, names, label)
    if isinstance(value, dict):
        value = _mapping(value, label).get("path")
    return _safe_relative(_string(value, f"{label}.path"), f"{label}.path")


def _parse_rfc3339_utc(value: Any, label: str) -> datetime:
    text = _string(value, label)
    if not text.endswith("Z"):
        _fail(f"{label} must use RFC3339 UTC Z form")
    try:
        parsed = datetime.fromisoformat(text[:-1] + "+00:00")
    except ValueError as error:
        _fail(f"{label} is not a valid RFC3339 timestamp: {error}")
    if parsed.utcoffset() != timezone.utc.utcoffset(parsed):
        _fail(f"{label} must identify UTC")
    return parsed


def _parse_int(value: str, label: str, *, minimum: int | None = None) -> int:
    try:
        result = int(value, 10)
    except (TypeError, ValueError):
        _fail(f"{label} must be a base-10 integer, got {value!r}")
    if minimum is not None and result < minimum:
        _fail(f"{label} must be >= {minimum}, got {result}")
    return result


def _parse_float(value: str, label: str, *, minimum: float | None = None) -> float:
    try:
        result = float(value)
    except (TypeError, ValueError):
        _fail(f"{label} must be numeric, got {value!r}")
    if not math.isfinite(result):
        _fail(f"{label} must be finite")
    if minimum is not None and result < minimum:
        _fail(f"{label} must be >= {minimum}, got {result}")
    return result


def _parse_csv_bool(value: str, label: str) -> bool:
    if value == "true":
        return True
    if value == "false":
        return False
    _fail(f"{label} must be lowercase true/false, got {value!r}")


def _close(left: float, right: float) -> bool:
    return math.isclose(
        left,
        right,
        rel_tol=FLOAT_REL_TOLERANCE,
        abs_tol=FLOAT_ABS_TOLERANCE,
    )


def _assert_close(observed: float, expected: float, label: str) -> None:
    if not _close(observed, expected):
        _fail(f"{label} mismatch: observed={observed:.12g}, expected={expected:.12g}")


def _rust_percentile(values: Sequence[float], percentage: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    position = percentage / 100.0 * (len(ordered) - 1)
    # Rust f64::round rounds nonnegative half values away from zero.
    index = math.floor(position + 0.5)
    return ordered[min(index, len(ordered) - 1)]


def _stats(values: Sequence[float]) -> Mapping[str, float | int]:
    if not values:
        return {"count": 0, "p50": 0.0, "p95": 0.0, "mean": 0.0, "min": 0.0, "max": 0.0}
    return {
        "count": len(values),
        "p50": _rust_percentile(values, 50.0),
        "p95": _rust_percentile(values, 95.0),
        "mean": math.fsum(values) / len(values),
        "min": min(values),
        "max": max(values),
    }


def _stats_with_p99(values: Sequence[float]) -> Mapping[str, float | int]:
    result = dict(_stats(values))
    result["p99"] = _rust_percentile(values, 99.0)
    return result


def _check_header(
    reader: csv.DictReader[str], expected: Sequence[str], label: str
) -> None:
    if tuple(reader.fieldnames or ()) != tuple(expected):
        _fail(
            f"{label} header mismatch: observed={reader.fieldnames!r}, expected={list(expected)!r}"
        )


def _require_row_complete(row: Mapping[str | None, Any], label: str) -> None:
    if None in row:
        _fail(f"{label} contains extra CSV fields")
    missing = [key for key, value in row.items() if value is None]
    if missing:
        _fail(f"{label} is missing CSV fields: {missing}")


def _iter_regular_files(root: Path) -> Iterator[tuple[str, Path]]:
    for path in sorted(root.rglob("*"), key=lambda item: item.as_posix()):
        try:
            metadata = path.lstat()
        except OSError as error:
            _fail(f"cannot stat run member {path}: {error}")
        relative = _safe_relative(path.relative_to(root).as_posix(), "run member")
        if stat.S_ISLNK(metadata.st_mode):
            _fail(f"run directory contains a symlink: {relative}")
        if stat.S_ISDIR(metadata.st_mode):
            continue
        if not stat.S_ISREG(metadata.st_mode):
            _fail(f"run directory contains a non-regular file: {relative}")
        yield relative, path


def _parse_hash_inventory(data: bytes, label: str) -> Mapping[str, str]:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        _fail(f"{label} is not UTF-8: {error}")
    if not text or not text.endswith("\n") or "\r" in text:
        _fail(f"{label} must be nonempty canonical LF text ending in newline")
    result: dict[str, str] = {}
    previous = ""
    for line_number, line in enumerate(text.splitlines(), 1):
        match = HASH_LINE.fullmatch(line)
        if match is None:
            _fail(f"{label} line {line_number} is malformed")
        digest, relative = match.groups()
        relative = _safe_relative(relative, f"{label} line {line_number} path")
        if relative in result:
            _fail(f"{label} duplicates {relative}")
        if previous and relative <= previous:
            _fail(f"{label} paths are not strictly sorted at {relative}")
        previous = relative
        result[relative] = digest
    return result


def _parse_sidecar(path: Path, package_name: str) -> str:
    try:
        data = path.read_bytes()
    except OSError as error:
        _fail(f"cannot read package sidecar {path}: {error}")
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        _fail(f"package sidecar is not UTF-8: {error}")
    match = re.fullmatch(r"([0-9a-f]{64})  ([^\r\n]+)\n", text)
    if match is None:
        _fail("package sidecar must be one canonical '<sha256>  <filename>\\n' record")
    digest, recorded_name = match.groups()
    if recorded_name != package_name:
        _fail(
            f"package sidecar filename mismatch: recorded={recorded_name!r}, actual={package_name!r}"
        )
    return digest


@dataclass(frozen=True)
class PackageInventoryEntry:
    relative_path: str
    sha256: str
    size_bytes: int


class MatrixPackage:
    """Safe, streaming view over the one-top-level-directory ZIP package."""

    def __init__(self, package_path: Path, expected_root_name: str):
        self.path = package_path
        try:
            self.archive = zipfile.ZipFile(package_path, "r", allowZip64=True)
        except (OSError, zipfile.BadZipFile) as error:
            _fail(f"cannot open G8-C package {package_path}: {error}")
        self.prefix = expected_root_name + "/"
        self._members: dict[str, zipfile.ZipInfo] = {}
        for info in self.archive.infolist():
            name = info.filename
            if "\\" in name or "\x00" in name:
                _fail(f"package contains unsafe member name {name!r}")
            pure = PurePosixPath(name)
            if pure.is_absolute() or any(
                part in {"", ".", ".."} for part in pure.parts
            ):
                _fail(f"package contains unsafe member name {name!r}")
            if not name.startswith(self.prefix):
                _fail(
                    f"package member is outside required top-level {expected_root_name!r}: {name!r}"
                )
            relative = name[len(self.prefix) :]
            if not relative:
                if not info.is_dir():
                    _fail("package top-level root entry must be a directory")
                continue
            if info.is_dir():
                continue
            relative = _safe_relative(relative, "package member")
            unix_type = (info.external_attr >> 16) & 0o170000
            if unix_type == stat.S_IFLNK:
                _fail(f"package contains a symlink member: {relative}")
            if info.flag_bits & 0x1:
                _fail(f"package contains an encrypted member: {relative}")
            if relative in self._members:
                _fail(f"package contains duplicate member: {relative}")
            self._members[relative] = info
        if not self._members:
            _fail("package contains no regular files")

    def close(self) -> None:
        self.archive.close()

    def __enter__(self) -> "MatrixPackage":
        return self

    def __exit__(self, *_: Any) -> None:
        self.close()

    @property
    def members(self) -> set[str]:
        return set(self._members)

    def open_binary(self, relative: str) -> BinaryIO:
        relative = _safe_relative(relative, "package member lookup")
        info = self._members.get(relative)
        if info is None:
            _fail(f"package is missing {relative}")
        try:
            return self.archive.open(info, "r")
        except (OSError, RuntimeError, zipfile.BadZipFile) as error:
            _fail(f"cannot open package member {relative}: {error}")

    def open_text(self, relative: str) -> TextIO:
        return io.TextIOWrapper(
            self.open_binary(relative), encoding="utf-8", newline=""
        )

    def read_bytes(self, relative: str, *, maximum: int = 64 * 1024 * 1024) -> bytes:
        info = self._members.get(_safe_relative(relative, "package member lookup"))
        if info is None:
            _fail(f"package is missing {relative}")
        if info.file_size > maximum:
            _fail(f"package member {relative} exceeds in-memory read limit")
        with self.open_binary(relative) as stream:
            data = stream.read(maximum + 1)
        if len(data) != info.file_size or len(data) > maximum:
            _fail(f"package member {relative} size/read mismatch")
        return data

    def hash_member(self, relative: str) -> tuple[str, int]:
        try:
            with self.open_binary(relative) as stream:
                return _sha256_stream(stream)
        except (OSError, RuntimeError, zipfile.BadZipFile) as error:
            _fail(f"cannot hash package member {relative}: {error}")

    def inventory(self) -> Mapping[str, PackageInventoryEntry]:
        result: dict[str, PackageInventoryEntry] = {}
        for relative in sorted(self._members):
            digest, size = self.hash_member(relative)
            info = self._members[relative]
            if size != info.file_size:
                _fail(f"package member size mismatch for {relative}")
            result[relative] = PackageInventoryEntry(relative, digest, size)
        return result


@dataclass(frozen=True)
class RunInventoryEntry:
    relative_path: str
    path: Path
    sha256: str
    size_bytes: int


def _inventory_run(root: Path) -> Mapping[str, RunInventoryEntry]:
    result: dict[str, RunInventoryEntry] = {}
    for relative, path in _iter_regular_files(root):
        digest = sha256_file(path)
        result[relative] = RunInventoryEntry(
            relative, path, digest, path.stat().st_size
        )
    return result


def _inventory_digest(inventory: Mapping[str, RunInventoryEntry]) -> str:
    digest = hashlib.sha256()
    for relative in sorted(inventory):
        entry = inventory[relative]
        digest.update(relative.encode("utf-8"))
        digest.update(b"\0")
        digest.update(str(entry.size_bytes).encode("ascii"))
        digest.update(b"\0")
        digest.update(entry.sha256.encode("ascii"))
        digest.update(b"\n")
    return digest.hexdigest()


@dataclass(frozen=True)
class AggregationReplayBinding:
    source_pilot_id: str
    original_root: Path
    copied_root: Path
    original_inventory_before: Mapping[str, RunInventoryEntry]
    process_result: Mapping[str, Any]
    implementation: Mapping[str, Mapping[str, Any]]


def _validate_replay_implementation(
    run_dir: Path,
    value: Any,
    independent_verifier: Any,
) -> Mapping[str, Mapping[str, Any]]:
    records = _mapping(value, "aggregation_replay.replay_implementation")
    expected_paths = {
        "coordinator": "verification/frozen-coordinator.py",
        "verifier": "verification/frozen-verifier.py",
    }
    if set(records) != set(expected_paths):
        _fail("replay implementation role inventory mismatch")
    result: dict[str, Mapping[str, Any]] = {}
    for role, expected_path in expected_paths.items():
        record = _mapping(records[role], f"replay implementation {role}")
        if set(record) != {"path", "size", "sha256"}:
            _fail(f"replay implementation {role} field inventory mismatch")
        if record.get("path") != expected_path:
            _fail(f"replay implementation {role} path mismatch")
        path = run_dir.joinpath(*PurePosixPath(expected_path).parts)
        size = _integer(record.get("size"), f"replay {role} size", minimum=1)
        digest = _string(record.get("sha256"), f"replay {role} SHA-256")
        if (
            HEX64.fullmatch(digest) is None
            or not path.is_file()
            or path.stat().st_size != size
            or sha256_file(path) != digest
        ):
            _fail(f"replay implementation {role} size/hash mismatch")
        result[role] = dict(record)
    verifier_record = _mapping(independent_verifier, "independent_verifier")
    if {key: verifier_record.get(key) for key in ("path", "size", "sha256")} != result[
        "verifier"
    ]:
        _fail("replay verifier identity differs from independent_verifier binding")
    return result


def _validate_aggregation_replay(
    run_dir: Path,
    manifest: Mapping[str, Any],
    profile: Mapping[str, Any],
) -> AggregationReplayBinding:
    replay = _mapping(manifest.get("aggregation_replay"), "aggregation_replay")
    expected_fields = {
        "source_pilot_id",
        "source_pilot_path",
        "source_pilot_inventory_path",
        "source_pilot_inventory_sha256",
        "source_pilot_inventory_digest",
        "source_pilot_file_count",
        "source_pilot_total_bytes",
        "inputs_root",
        "source_pilot_command_record_paths",
        "non_evidence",
        "gpu_measurement_reused_for_parser_validation",
        "measurement_subprocess_count",
        "executable_invocation_count",
        "gpu_context_count",
        "launched_process_count",
        "replay_implementation",
    }
    if set(replay) != expected_fields:
        _fail(
            "aggregation_replay field inventory mismatch: "
            f"missing={sorted(expected_fields - set(replay))}, "
            f"extra={sorted(set(replay) - expected_fields)}"
        )
    source_pilot_id = _string(replay.get("source_pilot_id"), "source pilot ID")
    if source_pilot_id != REPLAY_SOURCE_PILOT_ID:
        _fail("aggregation replay source pilot ID is not the approved replacement")
    source_pilot_path_text = _string(
        replay.get("source_pilot_path"), "source pilot path"
    )
    source_pilot_path = Path(source_pilot_path_text)
    if not source_pilot_path.is_absolute():
        _fail("source pilot path must be absolute")
    try:
        original_root = source_pilot_path.resolve(strict=True)
    except OSError as error:
        _fail(f"source pilot path is missing or inaccessible: {error}")
    if (
        not original_root.is_dir()
        or str(original_root) != source_pilot_path_text
        or original_root.name != source_pilot_id
        or original_root != REPLAY_SOURCE_PILOT_PATH.resolve(strict=True)
    ):
        _fail("source pilot path is not the normalized approved replacement path")
    if replay.get("inputs_root") != "source-pilot":
        _fail("aggregation replay inputs_root must be source-pilot")
    copied_root = run_dir / "source-pilot"
    if not copied_root.is_dir() or copied_root.resolve() == original_root:
        _fail("aggregation replay copied input root is missing or aliases the original")

    if manifest.get("command_record_paths") != []:
        _fail("aggregation replay must have no outer command records")
    if manifest.get("build_command_record") is not None:
        _fail("aggregation replay must not claim an outer build command")
    if replay.get("non_evidence") is not True:
        _fail("aggregation replay must be explicitly non-evidence")
    if replay.get("gpu_measurement_reused_for_parser_validation") is not True:
        _fail("aggregation replay must identify reused GPU measurements")
    for field in (
        "measurement_subprocess_count",
        "executable_invocation_count",
        "gpu_context_count",
        "launched_process_count",
    ):
        if _integer(replay.get(field), f"aggregation_replay.{field}", minimum=0) != 0:
            _fail(f"aggregation replay {field} must be zero")

    inventory_relative = _safe_relative(
        _string(
            replay.get("source_pilot_inventory_path"),
            "source pilot inventory path",
        )
    )
    if inventory_relative != "SOURCE_PILOT_INPUT_MANIFEST.json":
        _fail("source pilot inventory path is not canonical")
    inventory_path = run_dir / inventory_relative
    inventory_sha = _string(
        replay.get("source_pilot_inventory_sha256"),
        "source pilot inventory SHA-256",
    )
    if (
        HEX64.fullmatch(inventory_sha) is None
        or sha256_file(inventory_path) != inventory_sha
    ):
        _fail("source pilot inventory file hash mismatch")
    inventory_manifest = _read_json_file(
        inventory_path, "SOURCE_PILOT_INPUT_MANIFEST.json"
    )
    expected_inventory_fields = {
        "schema_version",
        "replay_run_id",
        "source_pilot_id",
        "source_pilot_path",
        "inputs_root",
        "pre_replay_digest",
        "post_aggregation_digest",
        "unchanged",
        "entry_count",
        "total_bytes",
        "entries",
    }
    if set(inventory_manifest) != expected_inventory_fields:
        _fail("source pilot inventory manifest field inventory mismatch")
    expected_inventory_scalars = {
        "schema_version": REPLAY_INPUT_SCHEMA,
        "replay_run_id": manifest.get("matrix_run_id"),
        "source_pilot_id": source_pilot_id,
        "source_pilot_path": source_pilot_path_text,
        "inputs_root": "source-pilot",
        "unchanged": True,
    }
    for field, expected in expected_inventory_scalars.items():
        if inventory_manifest.get(field) != expected:
            _fail(f"source pilot inventory {field} mismatch")

    original_inventory = _inventory_run(original_root)
    copied_inventory = _inventory_run(copied_root)
    entries = _sequence(inventory_manifest.get("entries"), "source pilot entries")
    recorded: dict[str, tuple[str, int]] = {}
    previous = ""
    for index, value in enumerate(entries):
        entry = _mapping(value, f"source pilot entries[{index}]")
        if set(entry) != {"path", "replay_path", "size", "sha256"}:
            _fail("source pilot inventory entry field mismatch")
        relative = _safe_relative(
            _string(entry.get("path"), f"source pilot entries[{index}].path")
        )
        if previous and relative <= previous:
            _fail("source pilot inventory paths are not strictly sorted")
        previous = relative
        replay_path = _safe_relative(
            _string(
                entry.get("replay_path"),
                f"source pilot entries[{index}].replay_path",
            )
        )
        if replay_path != f"source-pilot/{relative}":
            _fail(f"source pilot replay path mismatch for {relative}")
        size = _integer(entry.get("size"), f"source pilot {relative}.size", minimum=0)
        digest = _string(entry.get("sha256"), f"source pilot {relative}.sha256")
        if HEX64.fullmatch(digest) is None or relative in recorded:
            _fail(f"source pilot inventory digest/identity invalid for {relative}")
        recorded[relative] = (digest, size)

    expected_count = len(original_inventory)
    expected_bytes = sum(entry.size_bytes for entry in original_inventory.values())
    if (
        _integer(replay.get("source_pilot_file_count"), "source pilot file count")
        != expected_count
        or _integer(replay.get("source_pilot_total_bytes"), "source pilot bytes")
        != expected_bytes
        or _integer(inventory_manifest.get("entry_count"), "inventory entry_count")
        != expected_count
        or _integer(inventory_manifest.get("total_bytes"), "inventory total_bytes")
        != expected_bytes
    ):
        _fail("source pilot inventory count/byte total mismatch")
    expected_records = {
        relative: (entry.sha256, entry.size_bytes)
        for relative, entry in original_inventory.items()
    }
    copied_records = {
        relative: (entry.sha256, entry.size_bytes)
        for relative, entry in copied_inventory.items()
    }
    if recorded != expected_records or copied_records != expected_records:
        _fail("source pilot original/inventory/copied bytes differ")
    digest = _inventory_digest(original_inventory)
    manifest_digest = _string(
        replay.get("source_pilot_inventory_digest"),
        "source pilot inventory digest",
    )
    if (
        manifest_digest != digest
        or inventory_manifest.get("pre_replay_digest") != digest
        or inventory_manifest.get("post_aggregation_digest") != digest
    ):
        _fail("source pilot inventory aggregate digest mismatch")

    command_paths = _sequence(
        replay.get("source_pilot_command_record_paths"),
        "source pilot command record paths",
    )
    capture_manifest = {
        "matrix_run_id": source_pilot_id,
        "build_command_record": "build/COMMAND.json",
        "command_record_paths": list(command_paths),
        "frozen_binaries": manifest.get("frozen_binaries"),
    }
    process_result = _validate_process_records(
        copied_root,
        capture_manifest,
        profile,
        command_record_paths=command_paths,
        recorded_run_root=original_root,
    )
    if process_result["count"] != 16:
        _fail("source pilot must contain one build plus fifteen measurement records")
    implementation = _validate_replay_implementation(
        run_dir,
        replay.get("replay_implementation"),
        manifest.get("independent_verifier"),
    )
    return AggregationReplayBinding(
        source_pilot_id=source_pilot_id,
        original_root=original_root,
        copied_root=copied_root,
        original_inventory_before=original_inventory,
        process_result=process_result,
        implementation=implementation,
    )


def _validate_replay_original_unchanged(binding: AggregationReplayBinding) -> None:
    after = _inventory_run(binding.original_root)
    before = {
        relative: (entry.sha256, entry.size_bytes)
        for relative, entry in binding.original_inventory_before.items()
    }
    observed = {
        relative: (entry.sha256, entry.size_bytes) for relative, entry in after.items()
    }
    if observed != before or _inventory_digest(after) != _inventory_digest(
        binding.original_inventory_before
    ):
        _fail(
            "source pilot changed while aggregation replay was independently verified"
        )


def _validate_run_hashes(
    run_dir: Path,
    inventory: Mapping[str, RunInventoryEntry],
    *,
    run_mode: str = "official",
) -> tuple[Mapping[str, str], str]:
    required = {
        "G8C_MATRIX_MANIFEST.json",
        "HASHES.sha256",
        "G8C_MATRIX_RECEIPT.json",
    }
    if run_mode == "aggregation-replay":
        required.update(
            {
                "SOURCE_PILOT_INPUT_MANIFEST.json",
                "source-pilot/SOURCE_INPUT_MANIFEST.json",
                "source-pilot/SOURCE_INPUT_BYTES.zip",
                "source-pilot/GIT_SOURCE_ARCHIVE.zip",
                "source-pilot/frozen-binary/powdergame-benchmark.exe",
                "source-pilot/frozen-binary/powdergame-windows.exe",
                "verification/frozen-coordinator.py",
                "verification/frozen-verifier.py",
            }
        )
    else:
        required.update(
            {
                "SOURCE_INPUT_MANIFEST.json",
                "SOURCE_INPUT_BYTES.zip",
                "GIT_SOURCE_ARCHIVE.zip",
                "frozen-binary/powdergame-benchmark.exe",
                "frozen-binary/powdergame-windows.exe",
            }
        )
    missing = sorted(required - set(inventory))
    if missing:
        _fail(f"run directory is missing required files: {missing}")
    hashes_path = run_dir / "HASHES.sha256"
    hashes_bytes = hashes_path.read_bytes()
    hashes = _parse_hash_inventory(hashes_bytes, "HASHES.sha256")
    expected_paths = set(inventory) - {"HASHES.sha256", "G8C_MATRIX_RECEIPT.json"}
    if set(hashes) != expected_paths:
        _fail(
            "HASHES.sha256 inventory mismatch: "
            f"missing={sorted(expected_paths - set(hashes))}, "
            f"extra={sorted(set(hashes) - expected_paths)}"
        )
    for relative, expected_digest in hashes.items():
        observed = inventory[relative].sha256
        if observed != expected_digest:
            _fail(
                f"HASHES.sha256 digest mismatch for {relative}: "
                f"recorded={expected_digest}, observed={observed}"
            )
    return hashes, hashlib.sha256(hashes_bytes).hexdigest()


def _validate_package_copy(
    run_dir: Path,
    package_path: Path,
    sidecar_path: Path,
    run_inventory: Mapping[str, RunInventoryEntry],
) -> tuple[str, int]:
    recorded_package_hash = _parse_sidecar(sidecar_path, package_path.name)
    observed_package_hash = sha256_file(package_path)
    if observed_package_hash != recorded_package_hash:
        _fail(
            "package SHA-256 mismatch: "
            f"recorded={recorded_package_hash}, observed={observed_package_hash}"
        )
    with MatrixPackage(package_path, run_dir.name) as package:
        package_inventory = package.inventory()
        if set(package_inventory) != set(run_inventory):
            _fail(
                "package/run inventory mismatch: "
                f"missing={sorted(set(run_inventory) - set(package_inventory))}, "
                f"extra={sorted(set(package_inventory) - set(run_inventory))}"
            )
        for relative, run_entry in run_inventory.items():
            packaged = package_inventory[relative]
            if (
                packaged.sha256 != run_entry.sha256
                or packaged.size_bytes != run_entry.size_bytes
            ):
                _fail(f"package is not a byte-exact copy of run member {relative}")
    return observed_package_hash, package_path.stat().st_size


def _git_bytes(repo_root: Path, *arguments: str) -> bytes:
    safe_root = str(repo_root.resolve())
    completed = subprocess.run(
        ["git", "-c", f"safe.directory={safe_root}", *arguments],
        cwd=repo_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if completed.returncode != 0:
        detail = completed.stderr.decode("utf-8", errors="replace").strip()
        _fail(f"git {' '.join(arguments)} failed: {detail}")
    return completed.stdout


def _git_text(repo_root: Path, *arguments: str) -> str:
    try:
        return (
            _git_bytes(repo_root, *arguments).decode("utf-8", errors="strict").strip()
        )
    except UnicodeDecodeError as error:
        _fail(f"git {' '.join(arguments)} returned non-UTF-8 text: {error}")


def _zip_inventory_bytes(
    data: bytes, label: str
) -> Mapping[str, tuple[str, int, bytes]]:
    result: dict[str, tuple[str, int, bytes]] = {}
    try:
        with zipfile.ZipFile(io.BytesIO(data), "r", allowZip64=True) as archive:
            for info in archive.infolist():
                if info.is_dir():
                    continue
                name = _safe_relative(info.filename, f"{label} member")
                unix_type = (info.external_attr >> 16) & 0o170000
                if unix_type == stat.S_IFLNK:
                    _fail(f"{label} contains symlink {name}")
                if info.flag_bits & 0x1:
                    _fail(f"{label} contains encrypted member {name}")
                if name in result:
                    _fail(f"{label} contains duplicate member {name}")
                payload = archive.read(info)
                if len(payload) != info.file_size:
                    _fail(f"{label} member size mismatch for {name}")
                result[name] = (
                    hashlib.sha256(payload).hexdigest(),
                    len(payload),
                    payload,
                )
    except (OSError, RuntimeError, zipfile.BadZipFile) as error:
        _fail(f"invalid {label}: {error}")
    if not result:
        _fail(f"{label} is empty")
    return result


def _git_archive_commit(data: bytes) -> str:
    """Return the commit identity embedded by `git archive --format=zip`."""
    try:
        with zipfile.ZipFile(io.BytesIO(data), "r", allowZip64=True) as archive:
            commit = archive.comment.decode("ascii", errors="strict")
    except (OSError, UnicodeError, zipfile.BadZipFile) as error:
        _fail(f"cannot read Git archive commit identity: {error}")
    if OID40.fullmatch(commit) is None:
        _fail(f"GIT_SOURCE_ARCHIVE.zip has invalid embedded commit {commit!r}")
    return commit


def _zip_inventory_file(
    path: Path, label: str
) -> tuple[Mapping[str, tuple[str, int]], bytes]:
    """Stream a nested ZIP inventory without loading ZIP64 members into memory."""
    result: dict[str, tuple[str, int]] = {}
    try:
        with zipfile.ZipFile(path, "r", allowZip64=True) as archive:
            comment = archive.comment
            for info in archive.infolist():
                if info.is_dir():
                    continue
                name = _safe_relative(info.filename, f"{label} member")
                unix_type = (info.external_attr >> 16) & 0o170000
                if unix_type == stat.S_IFLNK:
                    _fail(f"{label} contains symlink {name}")
                if info.flag_bits & 0x1:
                    _fail(f"{label} contains encrypted member {name}")
                if name in result:
                    _fail(f"{label} contains duplicate member {name}")
                with archive.open(info, "r") as stream:
                    digest, size = _sha256_stream(stream)
                if size != info.file_size:
                    _fail(f"{label} member size mismatch for {name}")
                result[name] = (digest, size)
    except (OSError, RuntimeError, zipfile.BadZipFile) as error:
        _fail(f"invalid {label}: {error}")
    if not result:
        _fail(f"{label} is empty")
    return result, comment


def _source_entry_records(
    source_manifest: Mapping[str, Any],
) -> tuple[list[Mapping[str, Any]], list[Mapping[str, Any]]]:
    if "files" in source_manifest or "external_files" in source_manifest:
        files = [
            _mapping(value, f"source files[{index}]")
            for index, value in enumerate(
                _sequence(source_manifest.get("files"), "source files")
            )
        ]
        external = [
            _mapping(value, f"external files[{index}]")
            for index, value in enumerate(
                _sequence(source_manifest.get("external_files"), "external files")
            )
        ]
        if "file_count" in source_manifest:
            if _integer(
                source_manifest["file_count"], "source file_count", minimum=0
            ) != len(files):
                _fail("source file_count does not match files length")
        if "external_file_count" in source_manifest:
            if _integer(
                source_manifest["external_file_count"],
                "source external_file_count",
                minimum=0,
            ) != len(external):
                _fail("source external_file_count does not match external_files length")
        return files, external
    entries = [
        _mapping(value, f"source entries[{index}]")
        for index, value in enumerate(
            _sequence(source_manifest.get("entries"), "source entries")
        )
    ]
    files = [
        entry
        for entry in entries
        if entry.get("kind") in {"repository", "repository_tracked"}
    ]
    external = [
        entry
        for entry in entries
        if entry.get("kind") in {"external", "external_build_input"}
    ]
    if len(files) + len(external) != len(entries):
        _fail("source entries contain an unknown kind")
    return files, external


def _source_record(
    entry: Mapping[str, Any], *, external: bool, index: int
) -> tuple[str, str, str, int]:
    label = f"{'external' if external else 'repository'} source entry {index}"
    original = _string(
        _field(entry, ("path", "source_path", "original_path"), label),
        f"{label}.path",
    )
    archive_path = _safe_relative(
        _string(
            _field(entry, ("archive_path", "member_path"), label),
            f"{label}.archive_path",
        ),
        f"{label}.archive_path",
    )
    digest = _string(entry.get("sha256"), f"{label}.sha256")
    if HEX64.fullmatch(digest) is None:
        _fail(f"{label}.sha256 is not lowercase SHA-256")
    size = _integer(
        _field(entry, ("size", "size_bytes"), label), f"{label}.size", minimum=0
    )
    if external:
        if archive_path != "external/Windows/Fonts/consola.ttf":
            _fail(f"unexpected external archive path: {archive_path}")
    else:
        original = _safe_relative(original, f"{label}.path")
        if archive_path != f"repository/{original}":
            _fail(f"repository source archive mapping mismatch for {original}")
    return original, archive_path, digest, size


def _validate_source_inputs(
    run_dir: Path,
    manifest: Mapping[str, Any],
    source_sha: str,
    run_mode: str,
    repo_root: Path | None,
) -> Mapping[str, Any]:
    source_manifest_path = run_dir / "SOURCE_INPUT_MANIFEST.json"
    source_manifest = _read_json_file(
        source_manifest_path, "SOURCE_INPUT_MANIFEST.json"
    )
    if source_manifest.get("schema_version") != SOURCE_INPUT_SCHEMA:
        _fail("SOURCE_INPUT_MANIFEST.json schema mismatch")
    if source_manifest.get("matrix_run_id") != manifest.get("matrix_run_id"):
        _fail("source input manifest matrix run ID mismatch")
    if source_manifest.get("run_mode") != run_mode:
        _fail("source input manifest run mode mismatch")
    source_identity = _mapping(source_manifest.get("source"), "source input identity")
    recorded_sha = _string(
        _field(source_identity, ("sha", "head_sha"), "source input identity"),
        "source input SHA",
    )
    if recorded_sha != source_sha:
        _fail("source input manifest SHA does not match matrix source SHA")
    recorded_state = _string(source_identity.get("git_state"), "source input git_state")
    if run_mode == "official" and recorded_state != "clean":
        _fail("official source input manifest must record clean Git state")
    if run_mode == "pilot" and recorded_state not in {"clean", "dirty"}:
        _fail("pilot source input manifest has invalid Git state")
    outer_source = _mapping(manifest.get("source"), "matrix source")
    source_identity_fields = (
        "sha",
        "branch",
        "git_state",
        "dirty_scope",
        "status_porcelain",
        "upstream",
        "upstream_sha",
        "ahead_behind",
    )
    expected_source_identity = {
        key: outer_source.get(key) for key in source_identity_fields
    }
    if source_identity != expected_source_identity:
        _fail("source input manifest Git identity differs from matrix manifest")

    files, external_files = _source_entry_records(source_manifest)
    if _integer(
        source_manifest.get("entry_count"), "source entry_count", minimum=0
    ) != (len(files) + len(external_files)):
        _fail("source entry_count does not match entries length")
    if source_manifest.get("roles") != {
        "SOURCE_INPUT_BYTES.zip": "exact working-tree build-input bytes plus external Consolas input",
        "GIT_SOURCE_ARCHIVE.zip": "canonical Git archive for source SHA; pilot tracked changes intentionally differ",
    }:
        _fail("source archive role contract mismatch")
    records: list[tuple[str, str, str, int, bool]] = []
    seen_original: set[str] = set()
    seen_archive: set[str] = set()
    for external, entries in ((False, files), (True, external_files)):
        for index, entry in enumerate(entries):
            original, archive_path, digest, size = _source_record(
                entry, external=external, index=index
            )
            if original in seen_original or archive_path in seen_archive:
                _fail(f"source input manifest duplicates {original!r}/{archive_path!r}")
            seen_original.add(original)
            seen_archive.add(archive_path)
            records.append((original, archive_path, digest, size, external))
    if len(external_files) != 1:
        _fail("source input manifest must contain exactly the Consolas external input")
    source_digest_state = hashlib.sha256()
    for _, archive_path, digest, size, _ in records:
        source_digest_state.update(archive_path.encode("utf-8"))
        source_digest_state.update(b"\0")
        source_digest_state.update(str(size).encode("ascii"))
        source_digest_state.update(b"\0")
        source_digest_state.update(digest.encode("ascii"))
        source_digest_state.update(b"\n")
    source_input_digest = source_digest_state.hexdigest()
    if source_manifest.get("source_input_digest") != source_input_digest:
        _fail("source input aggregate digest mismatch")
    if outer_source.get("input_digest") != source_input_digest:
        _fail("matrix source input_digest does not match exact source manifest")

    source_zip_path = run_dir / "SOURCE_INPUT_BYTES.zip"
    source_zip, _ = _zip_inventory_file(source_zip_path, "SOURCE_INPUT_BYTES.zip")
    if set(source_zip) != seen_archive:
        _fail(
            "SOURCE_INPUT_BYTES.zip inventory mismatch: "
            f"missing={sorted(seen_archive - set(source_zip))}, "
            f"extra={sorted(set(source_zip) - seen_archive)}"
        )
    for original, archive_path, digest, size, _ in records:
        observed_digest, observed_size = source_zip[archive_path]
        if observed_digest != digest or observed_size != size:
            _fail(f"exact source-input bytes mismatch for {original}")

    git_archive_path = run_dir / "GIT_SOURCE_ARCHIVE.zip"
    git_archive_inventory, git_comment = _zip_inventory_file(
        git_archive_path, "GIT_SOURCE_ARCHIVE.zip"
    )
    try:
        git_commit = git_comment.decode("ascii", errors="strict")
    except UnicodeError as error:
        _fail(f"GIT_SOURCE_ARCHIVE.zip commit comment is not ASCII: {error}")
    if git_commit != source_sha or OID40.fullmatch(git_commit) is None:
        _fail("GIT_SOURCE_ARCHIVE.zip embedded commit differs from source SHA")
    manifest_tracked_names = {
        original for original, _, _, _, external in records if not external
    }
    if run_mode == "official" and set(git_archive_inventory) != manifest_tracked_names:
        _fail(
            "GIT_SOURCE_ARCHIVE.zip tracked-path inventory mismatch: "
            f"missing={sorted(manifest_tracked_names - set(git_archive_inventory))}, "
            f"extra={sorted(set(git_archive_inventory) - manifest_tracked_names)}"
        )
    # Deliberately do not compare working-tree build-input bytes to Git blob
    # bytes: a clean checkout may materialize different EOL bytes.  Both
    # archives are independently inventoried and bound, preserving that
    # forensic boundary instead of silently normalizing it.

    live_result: dict[str, Any] = {"checked": False}
    if repo_root is not None:
        repo = repo_root.resolve(strict=True)
        branch = _git_text(repo, "branch", "--show-current")
        if branch != REQUIRED_BRANCH:
            _fail(f"live source is on unexpected branch {branch!r}")
        live_head = _git_text(repo, "rev-parse", "HEAD")
        if live_head != source_sha:
            _fail(
                f"live repository HEAD changed: expected {source_sha}, got {live_head}"
            )
        status_bytes = _git_bytes(
            repo, "status", "--porcelain=v1", "-z", "--untracked-files=all"
        )
        records_status = [value for value in status_bytes.split(b"\0") if value]
        untracked = [value for value in records_status if value.startswith(b"?? ")]
        if untracked:
            _fail("live repository contains untracked files during verification")
        try:
            live_status = [
                value.decode("utf-8", errors="strict") for value in records_status
            ]
        except UnicodeDecodeError as error:
            _fail(f"live Git status is not UTF-8: {error}")
        recorded_status = list(
            _sequence(
                source_identity.get("status_porcelain"),
                "source input status_porcelain",
            )
        )
        if live_status != recorded_status:
            _fail(
                "live Git status differs from the sealed source identity: "
                f"live={live_status!r}, recorded={recorded_status!r}"
            )
        live_state = "dirty" if status_bytes else "clean"
        if live_state != recorded_state:
            _fail(
                f"live Git state differs from source manifest: live={live_state}, recorded={recorded_state}"
            )
        if run_mode == "official":
            upstream_name = _git_text(
                repo,
                "rev-parse",
                "--abbrev-ref",
                "--symbolic-full-name",
                "@{upstream}",
            )
            expected_upstream = f"origin/{REQUIRED_BRANCH}"
            if upstream_name != expected_upstream:
                _fail(
                    "official live source has the wrong upstream: "
                    f"{upstream_name!r} != {expected_upstream!r}"
                )
            upstream_sha = _git_text(repo, "rev-parse", "@{upstream}")
            counts = _git_text(
                repo, "rev-list", "--left-right", "--count", "HEAD...@{upstream}"
            ).split()
            if upstream_sha != live_head or counts != ["0", "0"]:
                _fail("official source HEAD is not equal to its upstream")
        tracked_raw = _git_bytes(repo, "ls-files", "-z", "--cached")
        tracked = {
            value.decode("utf-8", errors="strict")
            for value in tracked_raw.split(b"\0")
            if value
        }
        manifest_tracked = {original for original, _, _, _, ext in records if not ext}
        if manifest_tracked != tracked:
            _fail(
                "source input manifest is not the complete tracked working tree: "
                f"missing={sorted(tracked - manifest_tracked)}, "
                f"extra={sorted(manifest_tracked - tracked)}"
            )
        for original, archive_path, digest, size, external in records:
            live_path = (
                Path(original)
                if external
                else repo.joinpath(*PurePosixPath(original).parts)
            )
            if external:
                # The manifest carries the absolute Windows font path.
                live_path = Path(original)
            if not live_path.is_file():
                _fail(f"sealed source input is missing from live machine: {original}")
            live_digest = sha256_file(live_path)
            if live_digest != digest or live_path.stat().st_size != size:
                _fail(f"live source-input bytes drifted for {original}")
            if source_zip[archive_path][0] != live_digest:
                _fail(f"source archive/live byte mismatch for {original}")
        live_git_archive = _git_bytes(repo, "archive", "--format=zip", source_sha)
        if hashlib.sha256(live_git_archive).hexdigest() != sha256_file(
            git_archive_path
        ):
            _fail("GIT_SOURCE_ARCHIVE.zip is not the canonical live git archive")
        live_result = {
            "checked": True,
            "head_sha": live_head,
            "git_state": live_state,
            "branch": branch,
            "status_porcelain": live_status,
            "upstream": (
                f"origin/{REQUIRED_BRANCH}" if run_mode == "official" else None
            ),
            "upstream_sha": live_head if run_mode == "official" else None,
            "ahead_behind": ["0", "0"] if run_mode == "official" else None,
            "tracked_file_count": len(tracked),
        }

    return {
        "manifest_sha256": sha256_file(source_manifest_path),
        "source_input_archive_sha256": sha256_file(source_zip_path),
        "git_archive_sha256": sha256_file(git_archive_path),
        "source_input_digest": source_input_digest,
        "tracked_file_count": len(files),
        "external_file_count": len(external_files),
        "live_git": live_result,
    }


def _normalize_role(value: str) -> str:
    aliases = {
        "build": "build",
        "isolated-build": "build",
        "isolated-locked-release-build": "build",
        "headless": "headless",
        "mode-a-b": "headless",
        "benchmark": "headless",
        "headless-mode-a-b": "headless",
        "coexistence": "coexistence",
        "mode-c": "coexistence",
        "windowed-production-coexistence": "coexistence",
        "render-profile": "render-profile",
        "mode-d": "render-profile",
        "windowed-gpu-render-timing": "render-profile",
    }
    result = aliases.get(value)
    if result is None:
        _fail(f"unknown process role {value!r}")
    return result


def _validate_process_records(
    run_dir: Path,
    manifest: Mapping[str, Any],
    common: Mapping[str, Any],
    *,
    command_record_paths: Sequence[Any] | None = None,
    recorded_run_root: Path | None = None,
) -> Mapping[str, Any]:
    path_values = (
        command_record_paths
        if command_record_paths is not None
        else _sequence(manifest.get("command_record_paths"), "command_record_paths")
    )
    paths = [
        _safe_relative(_string(value, f"command_record_paths[{index}]"))
        for index, value in enumerate(path_values)
    ]
    if len(paths) != len(set(paths)):
        _fail("command_record_paths contains duplicates")
    expected_path_order = ["build/COMMAND.json"]
    expected_path_order.extend(
        f"process/{index:02d}-{scenario}-headless.json"
        for index, scenario in enumerate(SCENARIOS, 1)
    )
    expected_path_order.extend(
        f"process/{index:02d}-{scenario}-coexistence.json"
        for index, scenario in enumerate(SCENARIOS, 6)
    )
    expected_path_order.extend(
        f"process/{index:02d}-{scenario}-render-profile.json"
        for index, scenario in enumerate(SCENARIOS, 11)
    )
    if paths != expected_path_order:
        _fail("command_record_paths order/layout mismatch")
    expected_roles = {("build", None)} | {
        (role, scenario)
        for scenario in SCENARIOS
        for role in ("headless", "coexistence", "render-profile")
    }
    observed_roles: set[tuple[str, str | None]] = set()
    outputs_seen: set[str] = set()
    command_results: list[Mapping[str, Any]] = []
    latest_process_end: datetime | None = None
    command_run_root = (
        recorded_run_root.resolve(strict=True)
        if recorded_run_root is not None
        else run_dir.resolve(strict=True)
    )
    for path_index, relative in enumerate(paths):
        record = _read_json_file(
            run_dir.joinpath(*PurePosixPath(relative).parts), relative
        )
        expected_record_fields = {
            "schema_version",
            "role",
            "scenario",
            "argv",
            "cwd",
            "started_at_utc",
            "ended_at_utc",
            "exit_code",
            "environment_overrides",
            "stdout_path",
            "stderr_path",
            "expected_outputs",
        }
        if set(record) != expected_record_fields:
            _fail(f"process record field inventory mismatch in {relative}")
        if record.get("schema_version") != PROCESS_SCHEMA:
            _fail(f"process record schema mismatch in {relative}")
        raw_role = _string(record.get("role"), f"{relative}.role")
        expected_raw_role = (
            "isolated-locked-release-build"
            if path_index == 0
            else "headless-mode-a-b"
            if path_index <= 5
            else "windowed-production-coexistence"
            if path_index <= 10
            else "windowed-gpu-render-timing"
        )
        if raw_role != expected_raw_role:
            _fail(f"process role contract mismatch in {relative}")
        role = _normalize_role(raw_role)
        raw_scenario = record.get("scenario")
        scenario: str | None
        if role == "build":
            if raw_scenario not in (None, "", "all"):
                _fail(f"build process record must not target one scenario: {relative}")
            scenario = None
        else:
            scenario = _string(raw_scenario, f"{relative}.scenario")
            if scenario not in SCENARIOS:
                _fail(f"process record has nonofficial scenario {scenario!r}")
        identity = (role, scenario)
        if identity in observed_roles:
            _fail(f"duplicate process role/scenario record {identity}")
        observed_roles.add(identity)

        argv = [
            _string(value, f"{relative}.argv[{index}]", nonempty=False)
            for index, value in enumerate(
                _sequence(record.get("argv"), f"{relative}.argv")
            )
        ]
        if not argv:
            _fail(f"{relative}.argv must be nonempty")
        cwd = _string(record.get("cwd"), f"{relative}.cwd")
        if not Path(cwd).is_absolute():
            _fail(f"{relative}.cwd must be absolute")
        environment = _mapping(
            record.get("environment_overrides"), f"{relative}.environment_overrides"
        )
        count = _string(
            environment.get("GIT_CONFIG_COUNT"), f"{relative}.GIT_CONFIG_COUNT"
        )
        try:
            count_value = int(count)
        except ValueError:
            _fail(f"{relative}.GIT_CONFIG_COUNT is not an integer")
        safe_key = f"GIT_CONFIG_KEY_{count_value - 1}"
        safe_value = f"GIT_CONFIG_VALUE_{count_value - 1}"
        if (
            count_value < 1
            or set(environment) != {"GIT_CONFIG_COUNT", safe_key, safe_value}
            or environment.get(safe_key) != "safe.directory"
            or environment.get(safe_value) != Path(cwd).resolve().as_posix()
        ):
            _fail(f"{relative} environment override contract mismatch")
        started = _parse_rfc3339_utc(
            _field(record, ("start_utc", "started_utc", "started_at_utc"), relative),
            f"{relative}.start_utc",
        )
        ended = _parse_rfc3339_utc(
            _field(record, ("end_utc", "ended_utc", "ended_at_utc"), relative),
            f"{relative}.end_utc",
        )
        if ended < started:
            _fail(f"process record ends before it starts: {relative}")
        latest_process_end = (
            ended if latest_process_end is None else max(latest_process_end, ended)
        )
        exit_code = _integer(record.get("exit_code"), f"{relative}.exit_code")
        if exit_code != 0:
            _fail(f"process record is not successful: {relative} exit={exit_code}")
        stdout_path = _path_field(record, ("stdout_path", "stdout"), relative)
        stderr_path = _path_field(record, ("stderr_path", "stderr"), relative)
        if role == "build":
            expected_stdout, expected_stderr = "build/stdout.log", "build/stderr.log"
        else:
            stem = relative[:-5]
            expected_stdout, expected_stderr = (
                f"{stem}.stdout.log",
                f"{stem}.stderr.log",
            )
        if (stdout_path, stderr_path) != (expected_stdout, expected_stderr):
            _fail(f"process log path contract mismatch in {relative}")
        for log_path in (stdout_path, stderr_path):
            if not run_dir.joinpath(*PurePosixPath(log_path).parts).is_file():
                _fail(f"process record refers to missing log {log_path}")
        expected_outputs = [
            _safe_relative(_string(value, f"{relative}.expected_outputs[{index}]"))
            for index, value in enumerate(
                _sequence(
                    record.get("expected_outputs"), f"{relative}.expected_outputs"
                )
            )
        ]
        if len(expected_outputs) != len(set(expected_outputs)):
            _fail(f"process record duplicates expected outputs: {relative}")
        if role == "build":
            required_outputs: set[str] = set()
        elif role == "headless":
            assert scenario is not None
            required_outputs = {
                f"raw/headless/{scenario}/summary.csv",
                f"raw/headless/{scenario}/summary_raw_ticks.csv",
                f"raw/headless/{scenario}/summary_raw_cells.csv",
                f"raw/headless/{scenario}/summary_raw_chunks.csv",
            }
        elif role == "coexistence":
            assert scenario is not None
            required_outputs = {
                f"raw/coexistence/{scenario}/mode-c-coexistence.csv",
                f"raw/coexistence/{scenario}/mode-c-coexistence.json",
            }
        else:
            assert scenario is not None
            required_outputs = {
                f"raw/render-profile/{scenario}/mode-d-render-profile.csv",
                f"raw/render-profile/{scenario}/mode-d-render-profile.json",
            }
        if set(expected_outputs) != required_outputs:
            _fail(
                f"process expected-output contract mismatch in {relative}: "
                f"observed={sorted(expected_outputs)}, expected={sorted(required_outputs)}"
            )
        for output in expected_outputs:
            if output in outputs_seen:
                _fail(f"multiple process records claim output {output}")
            outputs_seen.add(output)
            if not run_dir.joinpath(*PurePosixPath(output).parts).is_file():
                _fail(f"process expected output is missing: {output}")

        joined = "\0".join(argv)
        if role == "build":
            for required in (
                "build",
                "--locked",
                "--release",
                "powdergame-benchmark",
                "powdergame-windows",
                "--target-dir",
            ):
                if required not in argv:
                    _fail(f"isolated build argv is missing {required!r}")
            if (
                Path(argv[0]).name.lower() not in {"cargo", "cargo.exe"}
                or len(argv) != 10
                or argv[1:5] != ["build", "--locked", "--release", "--target-dir"]
                or not Path(argv[5]).is_absolute()
                or argv[6:]
                != [
                    "-p",
                    "powdergame-benchmark",
                    "-p",
                    "powdergame-windows",
                ]
            ):
                _fail("isolated build command is not the exact locked release contract")
            target_dir = Path(argv[5]).resolve(strict=False)
            source_dir = Path(cwd).resolve(strict=False)
            if target_dir.is_relative_to(source_dir) or not target_dir.name.startswith(
                f".{_string(manifest.get('matrix_run_id'), 'matrix run ID')}-build-"
            ):
                _fail(
                    "locked release build target was not isolated from the source worktree"
                )
        elif role == "headless":
            if "--scenario" not in argv or scenario not in argv or "--csv" not in argv:
                _fail(f"headless argv does not bind scenario/output: {relative}")
            expected_tail = [
                "--scenario",
                scenario,
                "--width",
                str(common["width"]),
                "--height",
                str(common["height"]),
                "--chunk",
                str(common["chunk_size"]),
                "--sleep",
                "on" if common["sleep_enabled"] else "off",
                "--threshold",
                str(common["sleep_threshold"]),
                "--prewarm-secs",
                str(common["prewarm_secs"]),
                "--throughput-ticks",
                str(common["mode_a_ticks"]),
                "--profile-ticks",
                str(common["mode_b_ticks"]),
                "--overhead-ticks",
                str(common["overhead_ticks"]),
                "--trials",
                str(common["trials"]),
                "--csv",
                str(command_run_root / f"raw/headless/{scenario}/summary.csv"),
            ]
            if (
                argv[0]
                != str(command_run_root / "frozen-binary/powdergame-benchmark.exe")
                or argv[1:] != expected_tail
            ):
                _fail(
                    f"headless command differs from the common measurement contract: {relative}"
                )
        else:
            if "--g8c-worker" not in argv or scenario not in argv:
                _fail(f"window worker argv does not bind G8-C/scenario: {relative}")
            expected_mode = "coexistence" if role == "coexistence" else "render-profile"
            if "--mode" not in argv or expected_mode not in argv:
                _fail(f"window worker argv has wrong measurement mode: {relative}")
            expected_tail = [
                "--g8c-worker",
                "--mode",
                expected_mode,
                "--run-id",
                _string(manifest.get("matrix_run_id"), "matrix run ID"),
                "--binary-sha256",
                _mapping(manifest.get("frozen_binaries"), "frozen_binaries")["windows"][
                    "sha256"
                ],
                "--scenario",
                scenario,
                "--width",
                str(common["width"]),
                "--height",
                str(common["height"]),
                "--chunk",
                str(common["chunk_size"]),
                "--sleep",
                "on" if common["sleep_enabled"] else "off",
                "--threshold",
                str(common["sleep_threshold"]),
                "--prewarm-secs",
                str(common["prewarm_secs"]),
                "--trials",
                str(common["trials"]),
                "--target-tps",
                str(common["target_tps"]),
            ]
            if role == "coexistence":
                if common["mode_c_measurement_secs"] is not None:
                    expected_tail.extend(
                        ["--measurement-secs", str(common["mode_c_measurement_secs"])]
                    )
                else:
                    expected_tail.extend(
                        [
                            "--measurement-frames",
                            str(common["mode_c_measurement_frames"]),
                        ]
                    )
                csv_relative = f"raw/coexistence/{scenario}/mode-c-coexistence.csv"
                metadata_relative = (
                    f"raw/coexistence/{scenario}/mode-c-coexistence.json"
                )
            else:
                expected_tail.extend(
                    ["--profile-frames", str(common["mode_d_profile_frames"])]
                )
                csv_relative = (
                    f"raw/render-profile/{scenario}/mode-d-render-profile.csv"
                )
                metadata_relative = (
                    f"raw/render-profile/{scenario}/mode-d-render-profile.json"
                )
            expected_tail.extend(
                [
                    "--raw-csv",
                    str(command_run_root / csv_relative),
                    "--metadata-json",
                    str(command_run_root / metadata_relative),
                ]
            )
            if (
                argv[0]
                != str(command_run_root / "frozen-binary/powdergame-windows.exe")
                or argv[1:] != expected_tail
            ):
                _fail(
                    f"window worker command differs from the common measurement contract: {relative}"
                )
        command_results.append(
            {
                "path": relative,
                "role": role,
                "scenario": scenario,
                "argv_sha256": hashlib.sha256(joined.encode("utf-8")).hexdigest(),
                "duration_seconds": (ended - started).total_seconds(),
            }
        )
    if observed_roles != expected_roles:
        _fail(
            "process role/scenario inventory mismatch: "
            f"missing={sorted(expected_roles - observed_roles, key=str)}, "
            f"extra={sorted(observed_roles - expected_roles, key=str)}"
        )
    build_record = _safe_relative(
        _string(manifest.get("build_command_record"), "build_command_record")
    )
    build_matches = [
        item["path"] for item in command_results if item["role"] == "build"
    ]
    if build_matches != [build_record]:
        _fail("build_command_record does not identify the unique build process record")
    assert latest_process_end is not None
    return {
        "count": len(command_results),
        "records": command_results,
        "latest_end_utc": latest_process_end,
    }


@dataclass(frozen=True)
class CommonConfig:
    width: int
    height: int
    chunk_size: int
    sleep_enabled: bool
    sleep_threshold: int
    prewarm_seconds: float
    trials: int
    mode_a_ticks: int
    mode_b_ticks: int
    overhead_ticks: int
    target_tps: float
    mode_c_seconds: float | None
    mode_c_frames: int | None
    mode_d_frames: int
    render_width: int
    render_height: int


def _config_value(record: Mapping[str, Any], names: Sequence[str], label: str) -> Any:
    return _field(record, names, f"common_config.{label}")


def _parse_common_config(record: Mapping[str, Any], run_mode: str) -> CommonConfig:
    width = _integer(
        _config_value(record, ("width", "world_width"), "width"),
        "common_config.width",
        minimum=1,
    )
    height = _integer(
        _config_value(record, ("height", "world_height"), "height"),
        "common_config.height",
        minimum=1,
    )
    chunk = _integer(
        _config_value(record, ("chunk", "chunk_size"), "chunk_size"),
        "common_config.chunk_size",
        minimum=1,
    )
    sleep_enabled = _boolean(
        _config_value(record, ("sleep", "sleep_enabled"), "sleep_enabled"),
        "common_config.sleep_enabled",
    )
    threshold = _integer(
        _config_value(record, ("threshold", "sleep_threshold"), "sleep_threshold"),
        "common_config.sleep_threshold",
        minimum=0,
    )
    prewarm = _number(
        _config_value(record, ("prewarm_secs", "prewarm_seconds"), "prewarm"),
        "common_config.prewarm_seconds",
        minimum=0.0,
    )
    trials = _integer(record.get("trials"), "common_config.trials", minimum=1)
    mode_a_ticks = _integer(
        _config_value(record, ("mode_a_ticks", "throughput_ticks"), "mode_a_ticks"),
        "common_config.mode_a_ticks",
        minimum=1,
    )
    mode_b_ticks = _integer(
        _config_value(record, ("mode_b_ticks", "profile_ticks"), "mode_b_ticks"),
        "common_config.mode_b_ticks",
        minimum=1,
    )
    overhead_ticks = _integer(
        record.get("overhead_ticks"), "common_config.overhead_ticks", minimum=1
    )
    target_tps = _number(
        _config_value(record, ("target_tps", "mode_c_target_tps"), "target_tps"),
        "common_config.target_tps",
        minimum=0.001,
    )
    mode_d_frames = _integer(
        _config_value(
            record,
            ("mode_d_frames", "profile_frames", "mode_d_profile_frames"),
            "mode_d_frames",
        ),
        "common_config.mode_d_frames",
        minimum=1,
    )
    render_width = _integer(
        _config_value(
            record,
            ("render_width", "physical_width", "surface_width"),
            "render_width",
        ),
        "common_config.render_width",
        minimum=1,
    )
    render_height = _integer(
        _config_value(
            record,
            ("render_height", "physical_height", "surface_height"),
            "render_height",
        ),
        "common_config.render_height",
        minimum=1,
    )
    mode_c_seconds: float | None = None
    mode_c_frames: int | None = None
    seconds_keys = [
        key
        for key in (
            "mode_c_seconds",
            "measurement_seconds",
            "mode_c_measurement_secs",
        )
        if key in record and record[key] is not None
    ]
    frame_keys = [
        key
        for key in (
            "mode_c_frames",
            "measurement_frames",
            "mode_c_measurement_frames",
        )
        if key in record and record[key] is not None
    ]
    if len(seconds_keys) + len(frame_keys) != 1:
        _fail("common_config must select exactly one Mode C seconds/frames bound")
    if seconds_keys:
        mode_c_seconds = _number(
            record[seconds_keys[0]], "common_config.mode_c_seconds", minimum=0.001
        )
    else:
        mode_c_frames = _integer(
            record[frame_keys[0]], "common_config.mode_c_frames", minimum=1
        )

    config = CommonConfig(
        width=width,
        height=height,
        chunk_size=chunk,
        sleep_enabled=sleep_enabled,
        sleep_threshold=threshold,
        prewarm_seconds=prewarm,
        trials=trials,
        mode_a_ticks=mode_a_ticks,
        mode_b_ticks=mode_b_ticks,
        overhead_ticks=overhead_ticks,
        target_tps=target_tps,
        mode_c_seconds=mode_c_seconds,
        mode_c_frames=mode_c_frames,
        mode_d_frames=mode_d_frames,
        render_width=render_width,
        render_height=render_height,
    )
    if run_mode == "official":
        expected = CommonConfig(
            width=2048,
            height=2048,
            chunk_size=64,
            sleep_enabled=True,
            sleep_threshold=16,
            prewarm_seconds=2.0,
            trials=3,
            mode_a_ticks=1024,
            mode_b_ticks=256,
            overhead_ticks=256,
            target_tps=60.0,
            mode_c_seconds=10.0,
            mode_c_frames=None,
            mode_d_frames=256,
            render_width=1600,
            render_height=900,
        )
        if config != expected:
            _fail(
                f"official common config mismatch: observed={config!r}, expected={expected!r}"
            )
    else:
        if (
            config.width,
            config.height,
            config.chunk_size,
            config.trials,
            config.mode_a_ticks,
            config.mode_b_ticks,
            config.overhead_ticks,
            config.mode_c_frames,
            config.mode_d_frames,
            config.render_width,
            config.render_height,
        ) != (256, 256, 64, 1, 32, 16, 16, 60, 16, 1600, 900):
            _fail(f"pilot common config mismatch: {config!r}")
    return config


@dataclass(frozen=True)
class ScenarioPaths:
    headless_manifest: str
    summary: str
    raw_ticks: str
    raw_cells: str
    raw_chunks: str
    coexistence_csv: str
    coexistence_metadata: str
    render_csv: str
    render_metadata: str


def _scenario_path_map(record: Mapping[str, Any]) -> Mapping[str, Any]:
    for key in ("paths", "artifacts", "files"):
        if key in record:
            return _mapping(record[key], f"scenario.{key}")
    return record


def _scenario_paths(
    record: Mapping[str, Any], scenario: str, *, path_prefix: str = ""
) -> ScenarioPaths:
    paths = _scenario_path_map(record)

    def path(names: Sequence[str], label: str) -> str:
        return _path_field(paths, names, f"{scenario}.{label}")

    observed = ScenarioPaths(
        headless_manifest=path(
            ("headless_manifest", "headless_manifest_path"), "headless_manifest"
        ),
        summary=path(
            ("headless_summary", "summary", "summary_path"), "headless_summary"
        ),
        raw_ticks=path(
            ("headless_raw_ticks", "raw_ticks", "raw_ticks_path"), "raw_ticks"
        ),
        raw_cells=path(
            ("headless_raw_cells", "raw_cells", "raw_cells_path"), "raw_cells"
        ),
        raw_chunks=path(
            ("headless_raw_chunks", "raw_chunks", "raw_chunks_path"), "raw_chunks"
        ),
        coexistence_csv=path(
            ("coexistence_csv", "mode_c_csv", "coexistence_csv_path"),
            "coexistence_csv",
        ),
        coexistence_metadata=path(
            (
                "coexistence_metadata",
                "mode_c_metadata",
                "coexistence_metadata_path",
            ),
            "coexistence_metadata",
        ),
        render_csv=path(
            ("render_profile_csv", "mode_d_csv", "render_csv"), "render_profile_csv"
        ),
        render_metadata=path(
            (
                "render_profile_metadata",
                "mode_d_metadata",
                "render_metadata",
            ),
            "render_profile_metadata",
        ),
    )
    canonical = ScenarioPaths(
        headless_manifest=f"scenarios/{scenario}/HEADLESS_MANIFEST.json",
        summary=f"raw/headless/{scenario}/summary.csv",
        raw_ticks=f"raw/headless/{scenario}/summary_raw_ticks.csv",
        raw_cells=f"raw/headless/{scenario}/summary_raw_cells.csv",
        raw_chunks=f"raw/headless/{scenario}/summary_raw_chunks.csv",
        coexistence_csv=f"raw/coexistence/{scenario}/mode-c-coexistence.csv",
        coexistence_metadata=f"raw/coexistence/{scenario}/mode-c-coexistence.json",
        render_csv=f"raw/render-profile/{scenario}/mode-d-render-profile.csv",
        render_metadata=f"raw/render-profile/{scenario}/mode-d-render-profile.json",
    )
    expected = ScenarioPaths(
        **{
            field: f"{path_prefix}{getattr(canonical, field)}"
            for field in canonical.__dataclass_fields__
        }
    )
    if observed != expected:
        _fail(f"scenario artifact layout mismatch for {scenario}: {observed!r}")
    return canonical


def _read_csv_rows(
    path: Path, expected_header: Sequence[str], label: str
) -> list[dict[str, str]]:
    try:
        stream = path.open("r", encoding="utf-8", newline="")
    except OSError as error:
        _fail(f"cannot open {label}: {error}")
    with stream:
        reader = csv.DictReader(stream)
        _check_header(reader, expected_header, label)
        rows: list[dict[str, str]] = []
        for row_number, raw in enumerate(reader, 2):
            _require_row_complete(raw, f"{label} row {row_number}")
            rows.append({str(key): str(value) for key, value in raw.items()})
    if not rows:
        _fail(f"{label} has no data rows")
    return rows


def _validate_headless_common_row(
    row: Mapping[str, str],
    *,
    scenario: str,
    source_sha: str,
    config: CommonConfig,
    run_id: str,
    row_label: str,
    source_state: str | None = None,
    require_scenario_note: bool = True,
) -> None:
    if row["schema_version"] != INNER_HEADLESS_SCHEMA:
        _fail(f"{row_label} inner schema mismatch")
    if row["run_id"] != run_id or row["commit_sha"] != source_sha:
        _fail(f"{row_label} provenance mismatch")
    if row["git_state"] not in {"clean", "dirty"}:
        _fail(f"{row_label} invalid git_state")
    if source_state is not None and row["git_state"] != source_state:
        _fail(f"{row_label} Git state differs from the sealed matrix source")
    if row["build_profile"] != "release":
        _fail(f"{row_label} must use release build profile")
    if (
        _parse_int(row["width"], f"{row_label}.width"),
        _parse_int(row["height"], f"{row_label}.height"),
        _parse_int(row["chunk_size"], f"{row_label}.chunk_size"),
        _parse_csv_bool(row["sleep_enabled"], f"{row_label}.sleep_enabled"),
        _parse_int(row["sleep_threshold"], f"{row_label}.sleep_threshold"),
    ) != (
        config.width,
        config.height,
        config.chunk_size,
        config.sleep_enabled,
        config.sleep_threshold,
    ):
        _fail(f"{row_label} WorldConfig mismatch")
    _assert_close(
        _parse_float(row["prewarm_requested_secs"], f"{row_label}.prewarm"),
        config.prewarm_seconds,
        f"{row_label}.prewarm",
    )
    if require_scenario_note:
        scenario_tokens = [
            token.strip()
            for token in row.get("method_note", "").split(";")
            if token.strip().startswith("scenario=")
        ]
        expected_token = f"scenario={scenario}"
        if scenario_tokens != [expected_token]:
            _fail(
                f"{row_label} is not bound exclusively to scenario {scenario}: "
                f"{scenario_tokens!r}"
            )


def _summary_index(
    rows: Sequence[Mapping[str, str]],
) -> Mapping[tuple[str, str, str], Mapping[str, str]]:
    result: dict[tuple[str, str, str], Mapping[str, str]] = {}
    for row in rows:
        key = (row["metric_type"], row["name"], row["trial"])
        if key in result:
            _fail(f"headless summary contains duplicate metric identity {key}")
        result[key] = row
    return result


def _validate_throughput_external_contract(
    rows: Sequence[Mapping[str, str]],
    *,
    scenario: str,
    config: CommonConfig,
) -> None:
    """Validate the historical Rust producer vocabulary before adapting it.

    The producer's external name is deliberately ``wall_per_tick`` with the
    unit ``ms/tick``.  ``wall_ms_per_tick`` is an internal matrix field and is
    never accepted as a raw CSV alias.
    """

    trial_units = {
        "elapsed_wall": "ms",
        "wall_per_tick": "ms/tick",
        "sustained_tps": "ticks/s",
    }
    summary_units = {
        "wall_per_tick": "ms/tick",
        "sustained_tps": "ticks/s",
    }
    expected_identities = {
        ("throughput_trial", name, str(trial))
        for trial in range(1, config.trials + 1)
        for name in trial_units
    }
    expected_identities.update(
        ("throughput_summary", name, "all") for name in summary_units
    )

    observed: dict[tuple[str, str, str], Mapping[str, str]] = {}
    for row_number, row in enumerate(rows, 2):
        metric_type = row["metric_type"]
        name = row["name"]
        if metric_type not in {"throughput_trial", "throughput_summary"}:
            continue
        if name == "wall_ms_per_tick":
            _fail(
                f"{scenario} summary row {row_number} uses internal raw alias "
                "wall_ms_per_tick; expected historical wall_per_tick"
            )
        identity = (metric_type, name, row["trial"])
        if identity not in expected_identities:
            _fail(
                f"{scenario} summary contains unexpected throughput identity "
                f"{identity!r}"
            )
        if identity in observed:
            _fail(f"headless summary contains duplicate metric identity {identity}")
        observed[identity] = row

        label = f"{scenario}.{identity}"
        if row["measurement_mode"] != "production_throughput":
            _fail(f"{label}.measurement_mode must be production_throughput")
        if _parse_csv_bool(row["profiling_enabled"], f"{label}.profiling_enabled"):
            _fail(f"{label}.profiling_enabled must be false")
        if row["timestamp_period_ns"]:
            _fail(f"{label}.timestamp_period_ns must be empty")
        if row["tick_start"] != "0" or row["tick_end"] != str(config.mode_a_ticks - 1):
            _fail(f"{label} measured tick window mismatch")

        if metric_type == "throughput_trial":
            if row["selection"] != "trial":
                _fail(f"{label}.selection must be trial")
            expected_unit = trial_units[name]
            if row["unit"] != expected_unit:
                _fail(f"{label}.unit must be {expected_unit!r}, got {row['unit']!r}")
            _parse_float(row["value"], f"{label}.value", minimum=0.0)
            populated_stats = [
                field
                for field in ("count", "p50", "p95", "mean", "min", "max")
                if row[field]
            ]
            if populated_stats:
                _fail(f"{label} trial row unexpectedly contains summary statistics")
        else:
            if row["selection"] != "all_trials":
                _fail(f"{label}.selection must be all_trials")
            expected_unit = summary_units[name]
            if row["unit"] != expected_unit:
                _fail(f"{label}.unit must be {expected_unit!r}, got {row['unit']!r}")
            if row["value"]:
                _fail(f"{label} all-trials summary must not contain a trial value")
            _parse_int(row["count"], f"{label}.count", minimum=1)
            for field in ("p50", "p95", "mean", "min", "max"):
                _parse_float(row[field], f"{label}.{field}", minimum=0.0)

    observed_identities = set(observed)
    if observed_identities != expected_identities:
        _fail(
            f"{scenario} throughput external contract mismatch: "
            f"missing={sorted(expected_identities - observed_identities)}, "
            f"extra={sorted(observed_identities - expected_identities)}"
        )


def _check_summary_stats(
    row: Mapping[str, str], expected: Mapping[str, float | int], label: str
) -> None:
    observed_count = _parse_int(row["count"], f"{label}.count", minimum=0)
    if observed_count != expected["count"]:
        _fail(f"{label}.count mismatch: {observed_count} != {expected['count']}")
    for name in ("p50", "p95", "mean", "min", "max"):
        _assert_close(
            _parse_float(row[name], f"{label}.{name}"),
            float(expected[name]),
            f"{label}.{name}",
        )


def _validate_headless_summary(
    path: Path,
    *,
    scenario: str,
    source_sha: str,
    config: CommonConfig,
    source_state: str | None = None,
) -> tuple[str, list[dict[str, str]], Mapping[str, Any]]:
    rows = _read_csv_rows(path, SUMMARY_HEADER, f"{scenario} headless summary")
    run_ids = {row["run_id"] for row in rows}
    if len(run_ids) != 1:
        _fail(f"{scenario} summary does not contain exactly one run ID")
    run_id = next(iter(run_ids))
    if not run_id.startswith(f"g8b-{scenario}-"):
        _fail(f"{scenario} inner benchmark run ID has wrong prefix: {run_id}")
    for index, row in enumerate(rows):
        _validate_headless_common_row(
            row,
            scenario=scenario,
            source_sha=source_sha,
            config=config,
            run_id=run_id,
            row_label=f"{scenario} summary row {index + 2}",
            source_state=source_state,
        )
    _validate_throughput_external_contract(
        rows,
        scenario=scenario,
        config=config,
    )
    index = _summary_index(rows)
    trial_ids = set(range(1, config.trials + 1))
    throughput: dict[int, dict[str, float]] = defaultdict(dict)
    for trial in trial_ids:
        for name in ("elapsed_wall", "wall_per_tick", "sustained_tps"):
            key = ("throughput_trial", name, str(trial))
            row = index.get(key)
            if row is None:
                _fail(f"{scenario} summary is missing {key}")
            throughput[trial][name] = _parse_float(
                row["value"], f"{scenario}.{key}.value", minimum=0.0
            )
        elapsed = throughput[trial]["elapsed_wall"]
        per_tick = throughput[trial]["wall_per_tick"]
        tps = throughput[trial]["sustained_tps"]
        if elapsed <= 0.0 or per_tick <= 0.0 or tps <= 0.0:
            _fail(f"{scenario} trial {trial} throughput values must be positive")
        _assert_close(
            elapsed, per_tick * config.mode_a_ticks, f"{scenario} trial {trial} elapsed"
        )
        _assert_close(tps, 1000.0 / per_tick, f"{scenario} trial {trial} TPS")
    wall_values = [throughput[trial]["wall_per_tick"] for trial in sorted(trial_ids)]
    tps_values = [throughput[trial]["sustained_tps"] for trial in sorted(trial_ids)]
    for name, values in (("wall_per_tick", wall_values), ("sustained_tps", tps_values)):
        key = ("throughput_summary", name, "all")
        row = index.get(key)
        if row is None:
            _fail(f"{scenario} summary is missing {key}")
        _check_summary_stats(row, _stats(values), f"{scenario}.{name}")
    production_adapter = {
        tuple(
            row[name]
            for name in (
                "adapter_name",
                "vendor_id",
                "device_id",
                "backend",
                "driver",
                "driver_info",
            )
        )
        for row in rows
        if row["measurement_mode"] == "production_throughput"
    }
    profiling_adapter = {
        tuple(
            row[name]
            for name in (
                "adapter_name",
                "vendor_id",
                "device_id",
                "backend",
                "driver",
                "driver_info",
            )
        )
        for row in rows
        if row["measurement_mode"] == "isolated_profiled_tick"
    }
    if len(production_adapter) != 1 or production_adapter != profiling_adapter:
        _fail(f"{scenario} production/profiling adapter identity mismatch")
    adapter_name, vendor_id, device_id, backend, driver, driver_info = next(
        iter(production_adapter)
    )
    if (
        "RTX 5090" not in adapter_name.upper()
        or vendor_id.upper() != "0X10DE"
        or backend.lower() != "dx12"
    ):
        _fail(
            f"{scenario} headless modes require NVIDIA RTX 5090 / DX12: "
            f"{(adapter_name, vendor_id, device_id, backend, driver, driver_info)!r}"
        )
    return (
        run_id,
        rows,
        {
            "tps": dict(_stats(tps_values)),
            "wall_ms_per_tick": dict(_stats(wall_values)),
            "trials": throughput,
            "adapter": {
                "name": adapter_name,
                "vendor": int(vendor_id, 0),
                "device": int(device_id, 0),
                "backend": backend,
                "driver": driver,
                "driver_info": driver_info,
            },
        },
    )


def _validate_raw_ticks(
    path: Path,
    *,
    scenario: str,
    source_sha: str,
    config: CommonConfig,
    run_id: str,
    summary_rows: Sequence[Mapping[str, str]],
    source_state: str | None = None,
) -> Mapping[str, Any]:
    per_trial: dict[int, dict[str, list[float]]] = {}
    all_values: dict[str, list[float]] = {
        **{f"pass_{name}_ms": [] for name in PASS_NAMES},
        **{f"group_{name}_ms": [] for name in GROUPS},
        **{f"group_{name}_envelope_pct": [] for name in GROUPS},
        "gpu_pass_sum_ms": [],
        "gpu_tick_envelope_ms": [],
        "residual_ms": [],
    }
    sample_ids: dict[int, list[int]] = defaultdict(list)
    try:
        stream = path.open("r", encoding="utf-8", newline="")
    except OSError as error:
        _fail(f"cannot open {scenario} raw ticks: {error}")
    row_count = 0
    with stream:
        reader = csv.DictReader(stream)
        _check_header(reader, RAW_TICK_HEADER, f"{scenario} raw ticks")
        for row_number, raw in enumerate(reader, 2):
            _require_row_complete(raw, f"{scenario} raw tick row {row_number}")
            row = {str(key): str(value) for key, value in raw.items()}
            _validate_headless_common_row(
                row,
                scenario=scenario,
                source_sha=source_sha,
                config=config,
                run_id=run_id,
                row_label=f"{scenario} raw tick row {row_number}",
                source_state=source_state,
                require_scenario_note=False,
            )
            if row["measurement_mode"] != "isolated_profiled_tick":
                _fail(f"{scenario} raw tick row {row_number} has wrong mode")
            if not _parse_csv_bool(
                row["profiling_enabled"], f"{scenario} raw row profiling_enabled"
            ):
                _fail(f"{scenario} raw ticks must be profiling enabled")
            period = _parse_float(
                row["timestamp_period_ns"], f"{scenario} timestamp period", minimum=0.0
            )
            if period <= 0:
                _fail(f"{scenario} timestamp period must be positive")
            trial = _parse_int(row["trial"], f"{scenario}.trial", minimum=1)
            if trial > config.trials:
                _fail(f"{scenario} raw tick has out-of-range trial {trial}")
            sample = _parse_int(row["sample_id"], f"{scenario}.sample_id", minimum=0)
            tick = _parse_int(row["tick_index"], f"{scenario}.tick_index", minimum=0)
            if (
                sample != tick
                or row["tick_start"] != row["tick_index"]
                or row["tick_end"] != row["tick_index"]
            ):
                _fail(
                    f"{scenario} raw tick/sample identity mismatch at row {row_number}"
                )
            sample_ids[trial].append(sample)
            raw_pairs: list[tuple[int, int]] = []
            pass_values: dict[str, float] = {}
            previous_end: int | None = None
            for pass_name in PASS_NAMES:
                start = _parse_int(
                    row[f"{pass_name}_start_tick"],
                    f"{scenario}.{pass_name}.start",
                    minimum=0,
                )
                end = _parse_int(
                    row[f"{pass_name}_end_tick"],
                    f"{scenario}.{pass_name}.end",
                    minimum=0,
                )
                if end <= start:
                    _fail(
                        f"{scenario} {pass_name} timestamps are not positive/in-order"
                    )
                if previous_end is not None and start < previous_end:
                    _fail(
                        f"{scenario} pass timestamps overlap/out-of-order at {pass_name}"
                    )
                previous_end = end
                observed_ms = _parse_float(
                    row[f"pass_{pass_name}_ms"],
                    f"{scenario}.pass_{pass_name}_ms",
                    minimum=0.0,
                )
                reconstructed_ms = (end - start) * period / 1_000_000.0
                _assert_close(
                    observed_ms,
                    reconstructed_ms,
                    f"{scenario} pass {pass_name} duration",
                )
                raw_pairs.append((start, end))
                pass_values[pass_name] = observed_ms
                all_values[f"pass_{pass_name}_ms"].append(observed_ms)
            group_values: dict[str, float] = {}
            for group_name, members in GROUPS.items():
                expected_group = math.fsum(pass_values[name] for name in members)
                observed_group = _parse_float(
                    row[f"group_{group_name}_ms"],
                    f"{scenario}.group_{group_name}_ms",
                    minimum=0.0,
                )
                _assert_close(
                    observed_group,
                    expected_group,
                    f"{scenario} grouped subsystem {group_name}",
                )
                group_values[group_name] = observed_group
                all_values[f"group_{group_name}_ms"].append(observed_group)
            pass_sum = _parse_float(
                row["gpu_pass_sum_ms"], f"{scenario}.gpu_pass_sum_ms", minimum=0.0
            )
            envelope = _parse_float(
                row["gpu_tick_envelope_ms"],
                f"{scenario}.gpu_tick_envelope_ms",
                minimum=0.0,
            )
            residual = _parse_float(row["residual_ms"], f"{scenario}.residual_ms")
            _assert_close(
                pass_sum, math.fsum(pass_values.values()), f"{scenario} pass sum"
            )
            _assert_close(
                envelope,
                (raw_pairs[-1][1] - raw_pairs[0][0]) * period / 1_000_000.0,
                f"{scenario} GPU envelope",
            )
            _assert_close(residual, envelope - pass_sum, f"{scenario} residual")
            if envelope <= 0.0:
                _fail(f"{scenario} cannot calculate grouped/envelope ratio from zero")
            ratio_values = {
                group_name: value / envelope * 100.0
                for group_name, value in group_values.items()
            }
            for group_name, value in ratio_values.items():
                all_values[f"group_{group_name}_envelope_pct"].append(value)
            if (
                row["timestamp_unit"] != "raw_gpu_tick"
                or row["duration_unit"] != "milliseconds"
            ):
                _fail(f"{scenario} raw tick units mismatch")
            if row["group_definition"] != GROUP_DEFINITION:
                _fail(f"{scenario} group definition mismatch")
            for key, value in (
                ("gpu_pass_sum_ms", pass_sum),
                ("gpu_tick_envelope_ms", envelope),
                ("residual_ms", residual),
            ):
                all_values[key].append(value)
            trial_values = per_trial.setdefault(trial, {key: [] for key in all_values})
            for pass_name, value in pass_values.items():
                trial_values[f"pass_{pass_name}_ms"].append(value)
            for group_name, value in group_values.items():
                trial_values[f"group_{group_name}_ms"].append(value)
            for group_name, value in ratio_values.items():
                trial_values[f"group_{group_name}_envelope_pct"].append(value)
            trial_values["gpu_pass_sum_ms"].append(pass_sum)
            trial_values["gpu_tick_envelope_ms"].append(envelope)
            trial_values["residual_ms"].append(residual)
            row_count += 1
    expected_trials = set(range(1, config.trials + 1))
    if set(per_trial) != expected_trials:
        _fail(f"{scenario} raw tick trial inventory mismatch")
    for trial in sorted(expected_trials):
        if sample_ids[trial] != list(range(config.mode_b_ticks)):
            _fail(f"{scenario} raw tick samples are not contiguous in trial {trial}")
    if row_count != config.trials * config.mode_b_ticks:
        _fail(f"{scenario} raw tick row count mismatch")

    summary = _summary_index(summary_rows)
    for trial, values_by_name in per_trial.items():
        for pass_name in PASS_NAMES:
            key = ("pass", pass_name, str(trial))
            if key not in summary:
                _fail(f"{scenario} summary missing Mode B pass {key}")
            _check_summary_stats(
                summary[key],
                _stats(values_by_name[f"pass_{pass_name}_ms"]),
                f"{scenario}.{trial}.pass.{pass_name}",
            )
        for group_name in GROUPS:
            key = ("grouped_subsystem", group_name, str(trial))
            if key not in summary:
                _fail(f"{scenario} summary missing Mode B group {key}")
            _check_summary_stats(
                summary[key],
                _stats(values_by_name[f"group_{group_name}_ms"]),
                f"{scenario}.{trial}.group.{group_name}",
            )
            ratio_key = ("grouped_envelope_ratio", group_name, str(trial))
            if ratio_key not in summary:
                _fail(f"{scenario} summary missing Mode B ratio {ratio_key}")
            _check_summary_stats(
                summary[ratio_key],
                _stats(values_by_name[f"group_{group_name}_envelope_pct"]),
                f"{scenario}.{trial}.group_ratio.{group_name}",
            )
        for metric_name, field_name in (
            ("gpu_tick_envelope", "gpu_tick_envelope_ms"),
            ("gpu_pass_sum", "gpu_pass_sum_ms"),
            ("diagnostic_residual", "residual_ms"),
        ):
            key = ("envelope", metric_name, str(trial))
            if key not in summary:
                _fail(f"{scenario} summary missing Mode B envelope {key}")
            _check_summary_stats(
                summary[key],
                _stats(values_by_name[field_name]),
                f"{scenario}.{trial}.{field_name}",
            )
    return {
        "row_count": row_count,
        "fields": {key: dict(_stats(values)) for key, values in all_values.items()},
    }


def _validate_census(
    cells_path: Path,
    chunks_path: Path,
    *,
    scenario: str,
    source_sha: str,
    run_id: str,
    config: CommonConfig,
    summary_rows: Sequence[Mapping[str, str]],
    source_state: str | None = None,
) -> Mapping[str, int]:
    census = {
        "total_cells": 0,
        "any_active_cells": 0,
        "matter_active_cells": 0,
        "thermal_active_cells": 0,
        "pressure_active_cells": 0,
        "reaction_active_cells": 0,
        "total_chunks": 0,
        "active_chunks": 0,
        "runnable_chunks": 0,
        "sleeping_chunks": 0,
    }
    census_ticks: set[int] = set()
    expected_state = None
    try:
        cell_stream = cells_path.open("r", encoding="utf-8", newline="")
    except OSError as error:
        _fail(f"cannot open {scenario} raw cell census: {error}")
    with cell_stream:
        reader = csv.DictReader(cell_stream)
        _check_header(reader, RAW_CELL_HEADER, f"{scenario} raw cells")
        for expected_index, raw in enumerate(reader):
            _require_row_complete(raw, f"{scenario} raw cell row {expected_index + 2}")
            row = {str(key): str(value) for key, value in raw.items()}
            if (
                row["schema_version"] != INNER_HEADLESS_SCHEMA
                or row["run_id"] != run_id
                or row["commit_sha"] != source_sha
            ):
                _fail(f"{scenario} raw cell provenance mismatch")
            if expected_state is None:
                expected_state = row["git_state"]
            elif row["git_state"] != expected_state:
                _fail(f"{scenario} raw cell Git state changed within file")
            if source_state is not None and row["git_state"] != source_state:
                _fail(f"{scenario} raw cell Git state differs from sealed source")
            if _parse_int(row["index"], f"{scenario} cell index") != expected_index:
                _fail(f"{scenario} raw cell index is not contiguous")
            census_ticks.add(
                _parse_int(row["census_tick"], f"{scenario} census tick", minimum=0)
            )
            mask = _parse_int(
                row["activity_mask"], f"{scenario} cell activity", minimum=0
            )
            if mask & ~0xF:
                _fail(f"{scenario} cell activity contains unknown bits: {mask}")
            census["total_cells"] += 1
            census["any_active_cells"] += int(mask != 0)
            census["matter_active_cells"] += int(bool(mask & 1))
            census["thermal_active_cells"] += int(bool(mask & 2))
            census["pressure_active_cells"] += int(bool(mask & 4))
            census["reaction_active_cells"] += int(bool(mask & 8))
    try:
        chunk_stream = chunks_path.open("r", encoding="utf-8", newline="")
    except OSError as error:
        _fail(f"cannot open {scenario} raw chunk census: {error}")
    with chunk_stream:
        reader = csv.DictReader(chunk_stream)
        _check_header(reader, RAW_CHUNK_HEADER, f"{scenario} raw chunks")
        for expected_index, raw in enumerate(reader):
            _require_row_complete(raw, f"{scenario} raw chunk row {expected_index + 2}")
            row = {str(key): str(value) for key, value in raw.items()}
            if (
                row["schema_version"] != INNER_HEADLESS_SCHEMA
                or row["run_id"] != run_id
                or row["commit_sha"] != source_sha
            ):
                _fail(f"{scenario} raw chunk provenance mismatch")
            if row["git_state"] != expected_state:
                _fail(f"{scenario} raw chunk Git state differs from cells")
            if _parse_int(row["index"], f"{scenario} chunk index") != expected_index:
                _fail(f"{scenario} raw chunk index is not contiguous")
            census_ticks.add(
                _parse_int(row["census_tick"], f"{scenario} census tick", minimum=0)
            )
            mask = _parse_int(
                row["activity_mask"], f"{scenario} chunk activity", minimum=0
            )
            if mask & ~0xF:
                _fail(f"{scenario} chunk activity contains unknown bits: {mask}")
            state = _parse_int(row["chunk_state"], f"{scenario} chunk state", minimum=0)
            if state not in (0, 1):
                _fail(f"{scenario} chunk state is outside runnable/sleeping: {state}")
            census["total_chunks"] += 1
            census["active_chunks"] += int(mask != 0)
            census["runnable_chunks"] += int(state == 0)
            census["sleeping_chunks"] += int(state == 1)
    expected_cells = config.width * config.height
    expected_chunks = math.ceil(config.width / config.chunk_size) * math.ceil(
        config.height / config.chunk_size
    )
    if (
        census["total_cells"] != expected_cells
        or census["total_chunks"] != expected_chunks
    ):
        _fail(
            f"{scenario} census dimensions mismatch: "
            f"cells={census['total_cells']}/{expected_cells}, "
            f"chunks={census['total_chunks']}/{expected_chunks}"
        )
    if len(census_ticks) != 1:
        _fail(f"{scenario} census rows do not share one source tick")
    summary = _summary_index(summary_rows)
    for name, expected in census.items():
        key = ("activity_census", name, "n/a")
        row = summary.get(key)
        if row is None:
            _fail(f"{scenario} summary is missing census metric {name}")
        observed = _parse_float(row["value"], f"{scenario}.census.{name}")
        _assert_close(observed, float(expected), f"{scenario}.census.{name}")
    return census


def _validate_memory(
    summary_rows: Sequence[Mapping[str, str]], scenario: str
) -> Mapping[str, int]:
    index = _summary_index(summary_rows)
    names = (
        "world_dense_state",
        "movement_scratch",
        "activity_scratch",
        "uniforms_and_tables",
        "profiler_resolve_and_readback",
        "total_tracked",
    )
    values: dict[str, int] = {}
    for name in names:
        key = ("application_tracked_buffer_allocation", name, "n/a")
        row = index.get(key)
        if row is None:
            _fail(f"{scenario} summary is missing memory metric {name}")
        raw = _parse_float(row["value"], f"{scenario}.memory.{name}", minimum=0.0)
        if raw != math.floor(raw):
            _fail(f"{scenario} memory metric {name} is not an integer byte count")
        values[name] = int(raw)
    components = math.fsum(values[name] for name in names[:-1])
    if int(components) != values["total_tracked"]:
        _fail(f"{scenario} tracked memory total does not equal its components")
    return values


def _validate_overhead_and_summary_inventory(
    summary_rows: Sequence[Mapping[str, str]],
    *,
    scenario: str,
    config: CommonConfig,
) -> Mapping[str, float]:
    """Validate the complete G8-A summary identity set and overhead arithmetic."""
    index = _summary_index(summary_rows)
    expected: set[tuple[str, str, str]] = set()
    for trial in range(1, config.trials + 1):
        trial_text = str(trial)
        expected.update(
            ("throughput_trial", name, trial_text)
            for name in ("elapsed_wall", "wall_per_tick", "sustained_tps")
        )
        expected.update(("pass", name, trial_text) for name in PASS_NAMES)
        expected.update(("grouped_subsystem", name, trial_text) for name in GROUPS)
        expected.update(("grouped_envelope_ratio", name, trial_text) for name in GROUPS)
        expected.update(
            ("envelope", name, trial_text)
            for name in (
                "gpu_tick_envelope",
                "gpu_pass_sum",
                "diagnostic_residual",
            )
        )
    expected.update(
        ("throughput_summary", name, "all")
        for name in ("wall_per_tick", "sustained_tps")
    )
    expected.update(
        ("application_tracked_buffer_allocation", name, "n/a")
        for name in (
            "world_dense_state",
            "movement_scratch",
            "activity_scratch",
            "uniforms_and_tables",
            "profiler_resolve_and_readback",
            "total_tracked",
        )
    )
    expected.update(
        ("activity_census", name, "n/a")
        for name in (
            "total_cells",
            "any_active_cells",
            "matter_active_cells",
            "thermal_active_cells",
            "pressure_active_cells",
            "reaction_active_cells",
            "total_chunks",
            "active_chunks",
            "runnable_chunks",
            "sleeping_chunks",
        )
    )
    overhead_names = (
        "batched_unprofiled_elapsed",
        "synchronized_unprofiled_elapsed",
        "synchronized_profiled_elapsed",
        "synchronization_overhead",
        "profiling_increment",
        "observed_profiled_path_overhead",
    )
    expected.update(("profiling_overhead", name, "n/a") for name in overhead_names)
    observed = set(index)
    if observed != expected:
        _fail(
            f"{scenario} headless summary metric inventory mismatch: "
            f"missing={sorted(expected - observed)}, extra={sorted(observed - expected)}"
        )

    overhead = {
        name: _parse_float(
            index[("profiling_overhead", name, "n/a")]["value"],
            f"{scenario}.overhead.{name}",
        )
        for name in overhead_names
    }
    batched = overhead["batched_unprofiled_elapsed"]
    synchronized = overhead["synchronized_unprofiled_elapsed"]
    profiled = overhead["synchronized_profiled_elapsed"]
    if batched <= 0.0 or synchronized <= 0.0:
        _fail(f"{scenario} profiling controls must have positive baselines")
    _assert_close(
        overhead["synchronization_overhead"],
        (synchronized - batched) / batched * 100.0,
        f"{scenario}.synchronization_overhead",
    )
    _assert_close(
        overhead["profiling_increment"],
        (profiled - synchronized) / synchronized * 100.0,
        f"{scenario}.profiling_increment",
    )
    _assert_close(
        overhead["observed_profiled_path_overhead"],
        (profiled - batched) / batched * 100.0,
        f"{scenario}.observed_profiled_path_overhead",
    )
    for name in overhead_names:
        row = index[("profiling_overhead", name, "n/a")]
        if (
            _parse_int(row["tick_start"], f"{scenario}.overhead.tick_start") != 0
            or _parse_int(row["tick_end"], f"{scenario}.overhead.tick_end")
            != config.overhead_ticks - 1
        ):
            _fail(f"{scenario} overhead row tick window mismatch for {name}")
    return overhead


def _validate_headless_manifest(
    path: Path,
    *,
    scenario: str,
    matrix_run_id: str,
    source_sha: str,
    source_state: str,
    binary: Mapping[str, Any],
    profile: Mapping[str, Any],
    paths: ScenarioPaths,
    run_dir: Path,
    inner_run_id: str,
) -> None:
    manifest = _read_json_file(path, f"{scenario} HEADLESS_MANIFEST.json")
    expected_scalars = {
        "schema_version": HEADLESS_SCHEMA,
        "matrix_run_id": matrix_run_id,
        "scenario": scenario,
        "inner_schema_version": INNER_HEADLESS_SCHEMA,
        "inner_run_id": inner_run_id,
    }
    for key, expected in expected_scalars.items():
        if manifest.get(key) != expected:
            _fail(f"{scenario} headless manifest {key} mismatch")
    if manifest.get("source") != {"sha": source_sha, "git_state": source_state}:
        _fail(f"{scenario} headless manifest source mismatch")
    if manifest.get("frozen_benchmark_binary") != dict(binary):
        _fail(f"{scenario} headless manifest binary mismatch")
    if manifest.get("common_config") != dict(profile):
        _fail(f"{scenario} headless manifest common config mismatch")
    files = _mapping(manifest.get("files"), f"{scenario} headless files")
    expected_paths = {
        "summary": paths.summary,
        "raw_ticks": paths.raw_ticks,
        "raw_cells": paths.raw_cells,
        "raw_chunks": paths.raw_chunks,
    }
    if set(files) != set(expected_paths):
        _fail(f"{scenario} headless manifest file roles mismatch")
    for role, relative in expected_paths.items():
        record = _mapping(files[role], f"{scenario} headless {role}")
        actual_path = run_dir.joinpath(*PurePosixPath(relative).parts)
        if record.get("path") != relative:
            _fail(f"{scenario} headless {role} path mismatch")
        if (
            _integer(record.get("size"), f"{scenario}.{role}.size", minimum=0)
            != actual_path.stat().st_size
        ):
            _fail(f"{scenario} headless {role} size mismatch")
        if record.get("sha256") != sha256_file(actual_path):
            _fail(f"{scenario} headless {role} hash mismatch")


def _validate_headless(
    run_dir: Path,
    *,
    scenario: str,
    paths: ScenarioPaths,
    matrix_run_id: str,
    source_sha: str,
    source_state: str,
    benchmark_binary: Mapping[str, Any],
    profile: Mapping[str, Any],
    config: CommonConfig,
) -> Mapping[str, Any]:
    summary_path = run_dir.joinpath(*PurePosixPath(paths.summary).parts)
    run_id, summary_rows, mode_a = _validate_headless_summary(
        summary_path,
        scenario=scenario,
        source_sha=source_sha,
        config=config,
        source_state=source_state,
    )
    mode_b = _validate_raw_ticks(
        run_dir.joinpath(*PurePosixPath(paths.raw_ticks).parts),
        scenario=scenario,
        source_sha=source_sha,
        config=config,
        run_id=run_id,
        summary_rows=summary_rows,
        source_state=source_state,
    )
    census = _validate_census(
        run_dir.joinpath(*PurePosixPath(paths.raw_cells).parts),
        run_dir.joinpath(*PurePosixPath(paths.raw_chunks).parts),
        scenario=scenario,
        source_sha=source_sha,
        run_id=run_id,
        config=config,
        summary_rows=summary_rows,
        source_state=source_state,
    )
    memory = _validate_memory(summary_rows, scenario)
    overhead = _validate_overhead_and_summary_inventory(
        summary_rows, scenario=scenario, config=config
    )
    _validate_headless_manifest(
        run_dir.joinpath(*PurePosixPath(paths.headless_manifest).parts),
        scenario=scenario,
        matrix_run_id=matrix_run_id,
        source_sha=source_sha,
        source_state=source_state,
        binary=benchmark_binary,
        profile=profile,
        paths=paths,
        run_dir=run_dir,
        inner_run_id=run_id,
    )
    return {
        "inner_run_id": run_id,
        "mode_a": mode_a,
        "mode_b": mode_b,
        "census": census,
        "memory": memory,
        "overhead": overhead,
    }


def _nearest_rank(values: Sequence[float], fraction: float) -> float:
    """G8-A/Rust percentile contract (name retained for internal compatibility)."""
    return _rust_percentile(values, fraction * 100.0)


def _parse_presented(value: str, label: str) -> bool:
    if value == "1":
        return True
    if value == "0":
        return False
    _fail(f"{label} must be canonical 0/1, got {value!r}")


@dataclass
class WindowTrial:
    trial: int
    rows: int = 0
    last_frame_index: int = -1
    last_sim_tick: int = 0
    last_elapsed_ms: float = 0.0
    last_scheduled: int = 0
    ticks: int = 0
    presented: int = 0
    catch_up: int = 0
    missed: int = 0
    failed: int = 0
    surface_errors: int = 0
    frame_wall_ms: list[float] | None = None
    gpu_ms: list[float] | None = None

    def __post_init__(self) -> None:
        self.frame_wall_ms = []
        self.gpu_ms = []


def _metadata_trial_expected(trial: WindowTrial) -> Mapping[str, Any]:
    if trial.last_elapsed_ms <= 0:
        _fail(f"window trial {trial.trial} has no positive elapsed duration")
    seconds = trial.last_elapsed_ms / 1000.0
    frame_values = trial.frame_wall_ms or []
    gpu_values = trial.gpu_ms or []
    scheduled = max(trial.last_scheduled, trial.ticks)
    return {
        "trial": trial.trial,
        "elapsed_ms": trial.last_elapsed_ms,
        "actual_simulation_ticks": trial.ticks,
        "actual_simulation_tps": trial.ticks / seconds,
        "presented_frames": trial.presented,
        "render_fps": trial.presented / seconds,
        "frame_p50_ms": _nearest_rank(frame_values, 0.50),
        "frame_p95_ms": _nearest_rank(frame_values, 0.95),
        "frame_p99_ms": _nearest_rank(frame_values, 0.99),
        "missed_simulation_deadlines": trial.missed,
        "missed_deadline_ratio": trial.missed / scheduled if scheduled else 0.0,
        "catch_up_ticks": trial.catch_up,
        "failed_surface_frames": trial.failed,
        "device_errors": 0,
        "surface_errors": trial.surface_errors,
        "gpu_render_p50_ms": _nearest_rank(gpu_values, 0.50) if gpu_values else None,
        "gpu_render_p95_ms": _nearest_rank(gpu_values, 0.95) if gpu_values else None,
        "gpu_render_mean_ms": math.fsum(gpu_values) / len(gpu_values)
        if gpu_values
        else None,
    }


def _check_metadata_trial(
    observed: Mapping[str, Any], expected: Mapping[str, Any], label: str
) -> None:
    if set(observed) != set(expected):
        _fail(
            f"{label} field inventory mismatch: "
            f"missing={sorted(set(expected) - set(observed))}, "
            f"extra={sorted(set(observed) - set(expected))}"
        )
    for key, expected_value in expected.items():
        value = observed[key]
        if expected_value is None:
            if value is not None:
                _fail(f"{label}.{key} must be null")
        elif isinstance(expected_value, float):
            _assert_close(
                _number(value, f"{label}.{key}"), expected_value, f"{label}.{key}"
            )
        elif value != expected_value:
            _fail(f"{label}.{key} mismatch: {value!r} != {expected_value!r}")


def _validate_window_lifecycle(
    value: Any,
    *,
    mode: str,
    scenario: str,
    required_width: int,
    required_height: int,
) -> None:
    label = f"{scenario}.{mode}.window_lifecycle"
    lifecycle = _mapping(value, label)
    expected_fields = {
        "required_width",
        "required_height",
        "initial_live_width",
        "initial_live_height",
        "last_live_width",
        "last_live_height",
        "initial_live_size_confirmed",
        "canonical_noop_count",
        "stale_payload_count",
        "fatal_live_resize_count",
        "event_count",
        "events",
    }
    if set(lifecycle) != expected_fields:
        _fail(
            f"{label} field inventory mismatch: "
            f"missing={sorted(expected_fields - set(lifecycle))}, "
            f"extra={sorted(set(lifecycle) - expected_fields)}"
        )

    expected_size = (required_width, required_height)
    required_size = (
        _integer(lifecycle["required_width"], f"{label}.required_width", minimum=0),
        _integer(lifecycle["required_height"], f"{label}.required_height", minimum=0),
    )
    initial_live_size = (
        _integer(
            lifecycle["initial_live_width"],
            f"{label}.initial_live_width",
            minimum=0,
        ),
        _integer(
            lifecycle["initial_live_height"],
            f"{label}.initial_live_height",
            minimum=0,
        ),
    )
    last_live_size = (
        _integer(
            lifecycle["last_live_width"],
            f"{label}.last_live_width",
            minimum=0,
        ),
        _integer(
            lifecycle["last_live_height"],
            f"{label}.last_live_height",
            minimum=0,
        ),
    )
    if (
        required_size != expected_size
        or initial_live_size != expected_size
        or last_live_size != expected_size
    ):
        _fail(
            f"{label} requires required/initial/last live size "
            f"{required_width}x{required_height}: required={required_size}, "
            f"initial={initial_live_size}, last={last_live_size}"
        )
    if not _boolean(
        lifecycle["initial_live_size_confirmed"],
        f"{label}.initial_live_size_confirmed",
    ):
        _fail(f"{label}.initial_live_size_confirmed must be true")

    recorded_counts = {
        "canonical_no_op": _integer(
            lifecycle["canonical_noop_count"],
            f"{label}.canonical_noop_count",
            minimum=0,
        ),
        "stale_payload_ignored": _integer(
            lifecycle["stale_payload_count"],
            f"{label}.stale_payload_count",
            minimum=0,
        ),
        "fatal_noncanonical_live_size": _integer(
            lifecycle["fatal_live_resize_count"],
            f"{label}.fatal_live_resize_count",
            minimum=0,
        ),
    }
    events = _sequence(lifecycle["events"], f"{label}.events")
    event_count = _integer(lifecycle["event_count"], f"{label}.event_count", minimum=0)
    if event_count != len(events):
        _fail(f"{label}.event_count mismatch: {event_count} != {len(events)}")

    expected_event_fields = {
        "event_kind",
        "classification",
        "payload_width",
        "payload_height",
        "live_width",
        "live_height",
    }
    allowed_event_kinds = {"resized", "scale_factor_changed", "redraw_guard"}
    recomputed_counts = {classification: 0 for classification in recorded_counts}
    for index, raw_event in enumerate(events):
        event_label = f"{label}.events[{index}]"
        event = _mapping(raw_event, event_label)
        if set(event) != expected_event_fields:
            _fail(
                f"{event_label} field inventory mismatch: "
                f"missing={sorted(expected_event_fields - set(event))}, "
                f"extra={sorted(set(event) - expected_event_fields)}"
            )
        event_kind = _string(event["event_kind"], f"{event_label}.event_kind")
        if event_kind not in allowed_event_kinds:
            _fail(f"{event_label}.event_kind is invalid: {event_kind!r}")
        payload_size = (
            _integer(
                event["payload_width"],
                f"{event_label}.payload_width",
                minimum=0,
            ),
            _integer(
                event["payload_height"],
                f"{event_label}.payload_height",
                minimum=0,
            ),
        )
        live_size = (
            _integer(event["live_width"], f"{event_label}.live_width", minimum=0),
            _integer(event["live_height"], f"{event_label}.live_height", minimum=0),
        )
        if live_size != expected_size:
            expected_classification = "fatal_noncanonical_live_size"
        elif payload_size == expected_size:
            expected_classification = "canonical_no_op"
        else:
            expected_classification = "stale_payload_ignored"
        if event_kind == "redraw_guard" and expected_classification != (
            "fatal_noncanonical_live_size"
        ):
            _fail(
                f"{event_label} redraw_guard may only record a fatal live-size observation"
            )
        classification = _string(
            event["classification"], f"{event_label}.classification"
        )
        if classification != expected_classification:
            _fail(
                f"{event_label}.classification mismatch: "
                f"{classification!r} != {expected_classification!r}"
            )
        recomputed_counts[expected_classification] += 1
        if live_size != expected_size:
            _fail(
                f"{event_label} records noncanonical live size {live_size}; "
                f"successful metadata requires {expected_size}"
            )

    if recorded_counts != recomputed_counts:
        _fail(
            f"{label} counter mismatch: recorded={recorded_counts}, "
            f"recomputed={recomputed_counts}"
        )
    if recorded_counts["fatal_noncanonical_live_size"] != 0:
        _fail(f"{label} records a fatal live resize")


def _validate_worker_metadata(
    path: Path,
    *,
    mode: str,
    schema: str,
    scenario: str,
    matrix_run_id: str,
    source_sha: str,
    source_state: str,
    windows_binary: Mapping[str, Any],
    profile: Mapping[str, Any],
    config: CommonConfig,
    trials: Mapping[int, WindowTrial],
    raw_csv_path: Path,
    recorded_raw_csv_path: Path | None = None,
) -> Mapping[str, Any]:
    metadata = _read_json_file(path, f"{scenario} {mode} metadata")
    expected_top_fields = {
        "schema_version",
        "run_id",
        "mode",
        "source_sha",
        "git_state",
        "build_profile",
        "binary_sha256",
        "scenario",
        "requested_config",
        "actual_surface",
        "window_lifecycle",
        "adapter",
        "hud_enabled",
        "inspector_enabled",
        "text_diagnostics_enabled",
        "screenshot_readback_enabled",
        "timestamp_query_enabled",
        "device_error_count",
        "device_errors",
        "surface_error_count",
        "surface_errors",
        "raw_csv",
        "trials",
    }
    if set(metadata) != expected_top_fields:
        _fail(
            f"{scenario} {mode} metadata field inventory mismatch: "
            f"missing={sorted(expected_top_fields - set(metadata))}, "
            f"extra={sorted(set(metadata) - expected_top_fields)}"
        )
    expected_scalars = {
        "schema_version": schema,
        "run_id": matrix_run_id,
        "mode": mode,
        "source_sha": source_sha,
        "git_state": source_state,
        "build_profile": "release",
        "binary_sha256": windows_binary["sha256"],
        "scenario": scenario,
        "hud_enabled": False,
        "inspector_enabled": False,
        "text_diagnostics_enabled": False,
        "screenshot_readback_enabled": False,
        "timestamp_query_enabled": mode == "render-profile",
        "device_error_count": 0,
        "surface_error_count": 0,
    }
    for key, expected in expected_scalars.items():
        if metadata.get(key) != expected:
            _fail(
                f"{scenario} {mode} metadata {key} mismatch: "
                f"{metadata.get(key)!r} != {expected!r}"
            )
    device_errors = _sequence(
        metadata.get("device_errors"), f"{scenario}.{mode}.device_errors"
    )
    if device_errors:
        _fail(f"{scenario} {mode} recorded device errors")
    surface_errors = _sequence(
        metadata.get("surface_errors"), f"{scenario}.{mode}.surface_errors"
    )
    if surface_errors:
        _fail(f"{scenario} {mode} recorded surface errors")
    requested = _mapping(
        metadata.get("requested_config"), f"{scenario}.{mode}.requested_config"
    )
    expected_requested = {
        "width": config.width,
        "height": config.height,
        "chunk_size": config.chunk_size,
        "sleep_enabled": config.sleep_enabled,
        "sleep_threshold": config.sleep_threshold,
        "prewarm_secs": config.prewarm_seconds,
        "trials": config.trials,
        "target_tps": int(config.target_tps),
        "measurement_secs": config.mode_c_seconds if mode == "coexistence" else None,
        "measurement_frames": config.mode_c_frames if mode == "coexistence" else None,
        "profile_frames": config.mode_d_frames if mode == "render-profile" else None,
    }
    if set(requested) != set(expected_requested):
        _fail(f"{scenario} {mode} requested config fields mismatch")
    for key, expected in expected_requested.items():
        observed = requested[key]
        if expected is None:
            if observed is not None:
                _fail(f"{scenario} {mode} requested {key} must be null")
        elif isinstance(expected, float):
            _assert_close(
                _number(observed, f"{scenario}.{mode}.{key}"),
                expected,
                f"{scenario}.{mode}.{key}",
            )
        elif observed != expected:
            _fail(f"{scenario} {mode} requested {key} mismatch")
    surface = _mapping(
        metadata.get("actual_surface"), f"{scenario}.{mode}.actual_surface"
    )
    if set(surface) != {"width", "height", "format", "present_mode"}:
        _fail(f"{scenario} {mode} actual surface fields mismatch")
    if (
        surface["width"] != config.render_width
        or surface["height"] != config.render_height
        or not _string(surface["format"], f"{scenario}.{mode}.surface.format")
        or str(surface["present_mode"]).lower() != "fifo"
    ):
        _fail(f"{scenario} {mode} actual surface contract mismatch")
    _validate_window_lifecycle(
        metadata.get("window_lifecycle"),
        mode=mode,
        scenario=scenario,
        required_width=config.render_width,
        required_height=config.render_height,
    )
    adapter = _mapping(metadata.get("adapter"), f"{scenario}.{mode}.adapter")
    if set(adapter) != {"name", "vendor", "device", "backend", "driver", "driver_info"}:
        _fail(f"{scenario} {mode} adapter field inventory mismatch")
    if (
        "RTX 5090"
        not in _string(adapter.get("name"), f"{scenario}.{mode}.adapter.name")
        or _integer(adapter.get("vendor"), f"{scenario}.{mode}.adapter.vendor")
        != 0x10DE
        or str(adapter.get("backend", "")).lower() != "dx12"
    ):
        _fail(f"{scenario} {mode} adapter is not NVIDIA RTX 5090 / DX12")
    recorded_raw = Path(_string(metadata.get("raw_csv"), f"{scenario}.{mode}.raw_csv"))
    expected_recorded_raw = recorded_raw_csv_path or raw_csv_path
    _validate_recorded_raw_csv_path(
        recorded_raw,
        expected_recorded_raw,
        f"{scenario} {mode}",
    )
    metadata_trials = [
        _mapping(value, f"{scenario}.{mode}.trials[{index}]")
        for index, value in enumerate(
            _sequence(metadata.get("trials"), f"{scenario}.{mode}.trials")
        )
    ]
    if len(metadata_trials) != config.trials:
        _fail(f"{scenario} {mode} metadata trial count mismatch")
    for observed in metadata_trials:
        trial_id = _integer(
            observed.get("trial"), f"{scenario}.{mode}.trial", minimum=1
        )
        trial = trials.get(trial_id)
        if trial is None:
            _fail(f"{scenario} {mode} metadata has unknown trial {trial_id}")
        _check_metadata_trial(
            observed,
            _metadata_trial_expected(trial),
            f"{scenario}.{mode}.trial[{trial_id}]",
        )
    return metadata


def _validate_recorded_raw_csv_path(
    recorded_raw: Path,
    expected_recorded_raw: Path,
    label: str,
) -> None:
    if recorded_raw.resolve() != expected_recorded_raw.resolve():
        _fail(f"{label} metadata raw_csv path mismatch")


def _validate_window_rows(
    path: Path,
    *,
    mode: str,
    scenario: str,
    config: CommonConfig,
) -> tuple[Mapping[int, WindowTrial], Mapping[str, Any]]:
    if mode == "coexistence":
        schema = COEXISTENCE_SCHEMA
        expected_header = COEXISTENCE_HEADER
    else:
        schema = RENDER_PROFILE_SCHEMA
        expected_header = RENDER_PROFILE_HEADER
    trials: dict[int, WindowTrial] = {}
    all_frame_ms: list[float] = []
    all_gpu_ms: list[float] = []
    timestamp_periods: set[float] = set()
    try:
        stream = path.open("r", encoding="utf-8", newline="")
    except OSError as error:
        _fail(f"cannot open {scenario} {mode} raw CSV: {error}")
    with stream:
        reader = csv.DictReader(stream)
        _check_header(reader, expected_header, f"{scenario} {mode} raw CSV")
        for row_number, raw in enumerate(reader, 2):
            _require_row_complete(raw, f"{scenario} {mode} row {row_number}")
            row = {str(key): str(value) for key, value in raw.items()}
            if row["schema_version"] != schema:
                _fail(f"{scenario} {mode} row schema mismatch at {row_number}")
            if row["scenario"] != scenario:
                _fail(
                    f"{scenario} {mode} row scenario identity mismatch at {row_number}"
                )
            trial_id = _parse_int(row["trial"], f"{scenario}.{mode}.trial", minimum=1)
            if trial_id > config.trials:
                _fail(f"{scenario} {mode} trial is out of range")
            trial = trials.setdefault(trial_id, WindowTrial(trial_id))
            frame = _parse_int(
                row["frame_index"], f"{scenario}.{mode}.frame", minimum=0
            )
            if frame != trial.rows or frame != trial.last_frame_index + 1:
                _fail(
                    f"{scenario} {mode} frame sequence is not contiguous in trial {trial_id}"
                )
            sim_tick = _parse_int(
                row["sim_tick"], f"{scenario}.{mode}.sim_tick", minimum=0
            )
            elapsed = _parse_float(
                row["window_elapsed_ms"], f"{scenario}.{mode}.elapsed", minimum=0.0
            )
            frame_wall = _parse_float(
                row["frame_wall_ms"], f"{scenario}.{mode}.frame_wall", minimum=0.0
            )
            if elapsed <= trial.last_elapsed_ms or frame_wall <= 0:
                _fail(
                    f"{scenario} {mode} elapsed/frame duration is not strictly positive"
                )
            _assert_close(
                frame_wall,
                elapsed - trial.last_elapsed_ms,
                f"{scenario} {mode} frame wall delta",
            )
            scheduled = _parse_int(
                row["scheduled_sim_ticks"], f"{scenario}.{mode}.scheduled", minimum=0
            )
            executed = _parse_int(
                row["sim_ticks_executed"], f"{scenario}.{mode}.executed", minimum=0
            )
            catch_up = _parse_int(
                row["catch_up_ticks"], f"{scenario}.{mode}.catch_up", minimum=0
            )
            missed = _parse_int(
                row["missed_simulation_deadlines"],
                f"{scenario}.{mode}.missed",
                minimum=0,
            )
            if (
                scheduled < trial.last_sim_tick
                or executed != scheduled - trial.last_sim_tick
            ):
                _fail(f"{scenario} {mode} scheduler backlog arithmetic mismatch")
            expected_behind = max(executed - 1, 0)
            if catch_up != expected_behind or missed != expected_behind:
                _fail(f"{scenario} {mode} catch-up/deadline arithmetic mismatch")
            if sim_tick != trial.last_sim_tick + executed or sim_tick != scheduled:
                _fail(f"{scenario} {mode} simulation tick accounting mismatch")
            if scheduled > math.floor(elapsed / 1000.0 * config.target_tps):
                _fail(f"{scenario} {mode} scheduled ticks exceed elapsed target")
            presented = _parse_presented(
                row["presented"], f"{scenario}.{mode}.presented"
            )
            surface_error = row["surface_error"]
            if presented == bool(surface_error):
                _fail(f"{scenario} {mode} presented/surface_error consistency failure")
            # Successful official workers currently abort on acquisition/render errors;
            # evidence rows must therefore all be presented and error-free.
            if not presented or surface_error:
                _fail(f"{scenario} {mode} contains a failed surface frame")
            trial.rows += 1
            trial.last_frame_index = frame
            trial.last_sim_tick = sim_tick
            trial.last_elapsed_ms = elapsed
            trial.last_scheduled = scheduled
            trial.ticks += executed
            trial.presented += int(presented)
            trial.catch_up += catch_up
            trial.missed += missed
            trial.failed += int(not presented)
            trial.surface_errors += int(bool(surface_error))
            assert trial.frame_wall_ms is not None
            trial.frame_wall_ms.append(frame_wall)
            all_frame_ms.append(frame_wall)
            if mode == "render-profile":
                start = _parse_int(
                    row["gpu_start_tick"], f"{scenario}.gpu_start_tick", minimum=0
                )
                end = _parse_int(
                    row["gpu_end_tick"], f"{scenario}.gpu_end_tick", minimum=0
                )
                if end <= start:
                    _fail(f"{scenario} Mode D raw timestamps are not positive/in-order")
                period = _parse_float(
                    row["timestamp_period_ns"],
                    f"{scenario}.timestamp_period_ns",
                    minimum=0.0,
                )
                if period <= 0:
                    _fail(f"{scenario} Mode D timestamp period must be positive")
                observed_gpu = _parse_float(
                    row["gpu_render_ms"], f"{scenario}.gpu_render_ms", minimum=0.0
                )
                reconstructed = (end - start) * period / 1_000_000.0
                _assert_close(
                    observed_gpu, reconstructed, f"{scenario} Mode D GPU time"
                )
                timestamp_periods.add(period)
                assert trial.gpu_ms is not None
                trial.gpu_ms.append(observed_gpu)
                all_gpu_ms.append(observed_gpu)
    expected_trials = set(range(1, config.trials + 1))
    if set(trials) != expected_trials:
        _fail(f"{scenario} {mode} trial inventory mismatch")
    for trial in trials.values():
        if trial.last_elapsed_ms <= 0.0:
            _fail(f"{scenario} {mode} trial has no positive measured duration")
        if mode == "coexistence" and config.mode_c_frames is not None:
            if trial.rows != config.mode_c_frames:
                _fail(f"{scenario} pilot Mode C frame count mismatch")
        elif mode == "coexistence" and config.mode_c_seconds is not None:
            if trial.last_elapsed_ms < config.mode_c_seconds * 1000.0:
                _fail(f"{scenario} official Mode C window ended early")
            if trial.last_elapsed_ms > config.mode_c_seconds * 1000.0 + 2_000.0:
                _fail(f"{scenario} official Mode C window exceeded its fixed bound")
        elif mode == "render-profile" and trial.rows != config.mode_d_frames:
            _fail(f"{scenario} Mode D frame count mismatch")
    if mode == "render-profile" and len(timestamp_periods) != 1:
        _fail(f"{scenario} Mode D does not use one timestamp period")
    sim_rates = [
        trial.ticks / (trial.last_elapsed_ms / 1000.0) for trial in trials.values()
    ]
    frame_rates = [
        trial.presented / (trial.last_elapsed_ms / 1000.0) for trial in trials.values()
    ]
    scheduled_total = sum(trial.last_scheduled for trial in trials.values())
    aggregate = {
        "actual_simulation_ticks": sum(trial.ticks for trial in trials.values()),
        "simulation_tps": dict(_stats_with_p99(sim_rates)),
        "presented_frames": sum(trial.presented for trial in trials.values()),
        "render_fps": dict(_stats_with_p99(frame_rates)),
        "frame_wall_ms": dict(_stats_with_p99(all_frame_ms)),
        "missed_simulation_deadlines": sum(trial.missed for trial in trials.values()),
        "missed_deadline_ratio": (
            sum(trial.missed for trial in trials.values()) / scheduled_total
            if scheduled_total
            else 0.0
        ),
        "catch_up_ticks": sum(trial.catch_up for trial in trials.values()),
        "failed_surface_frames": sum(trial.failed for trial in trials.values()),
        "surface_errors": sum(trial.surface_errors for trial in trials.values()),
        "device_errors": 0,
    }
    if mode == "render-profile":
        aggregate["gpu_render_ms"] = dict(_stats_with_p99(all_gpu_ms))
    return trials, aggregate


def _validate_window_mode(
    run_dir: Path,
    *,
    mode: str,
    scenario: str,
    paths: ScenarioPaths,
    matrix_run_id: str,
    source_sha: str,
    source_state: str,
    windows_binary: Mapping[str, Any],
    profile: Mapping[str, Any],
    config: CommonConfig,
    recorded_run_root: Path | None = None,
) -> Mapping[str, Any]:
    if mode == "coexistence":
        raw_relative = paths.coexistence_csv
        metadata_relative = paths.coexistence_metadata
        schema = COEXISTENCE_SCHEMA
    else:
        raw_relative = paths.render_csv
        metadata_relative = paths.render_metadata
        schema = RENDER_PROFILE_SCHEMA
    raw_path = run_dir.joinpath(*PurePosixPath(raw_relative).parts)
    recorded_raw_path = (
        recorded_run_root.joinpath(*PurePosixPath(raw_relative).parts)
        if recorded_run_root is not None
        else raw_path
    )
    trials, aggregate = _validate_window_rows(
        raw_path, mode=mode, scenario=scenario, config=config
    )
    metadata = _validate_worker_metadata(
        run_dir.joinpath(*PurePosixPath(metadata_relative).parts),
        mode=mode,
        schema=schema,
        scenario=scenario,
        matrix_run_id=matrix_run_id,
        source_sha=source_sha,
        source_state=source_state,
        windows_binary=windows_binary,
        profile=profile,
        config=config,
        trials=trials,
        raw_csv_path=raw_path,
        recorded_raw_csv_path=recorded_raw_path,
    )
    aggregate = dict(aggregate)
    aggregate["adapter"] = dict(_mapping(metadata["adapter"], "window adapter"))
    aggregate["surface"] = dict(
        _mapping(metadata["actual_surface"], "window actual surface")
    )
    return aggregate


def _expected_profile(run_mode: str) -> Mapping[str, Any]:
    reduced_profile = run_mode in {"pilot", "aggregation-replay"}
    common: dict[str, Any] = {
        "width": 256 if reduced_profile else 2048,
        "height": 256 if reduced_profile else 2048,
        "chunk_size": 64,
        "sleep_enabled": True,
        "sleep_threshold": 16,
        "prewarm_secs": 2.0,
        "trials": 1 if reduced_profile else 3,
        "mode_a_ticks": 32 if reduced_profile else 1024,
        "mode_b_ticks": 16 if reduced_profile else 256,
        "overhead_ticks": 16 if reduced_profile else 256,
        "mode_c_measurement_secs": None if reduced_profile else 10.0,
        "mode_c_measurement_frames": 60 if reduced_profile else None,
        "mode_d_profile_frames": 16 if reduced_profile else 256,
        "target_tps": 60,
        "render_width": 1600,
        "render_height": 900,
        "present_mode": "Fifo",
    }
    return common


def _validate_source_identity(
    source: Mapping[str, Any], run_mode: str
) -> tuple[str, str]:
    expected_fields = {
        "sha",
        "branch",
        "git_state",
        "dirty_scope",
        "status_porcelain",
        "upstream",
        "upstream_sha",
        "ahead_behind",
        "input_digest",
        "input_manifest",
        "exact_input_archive",
        "canonical_git_archive",
    }
    if set(source) != expected_fields:
        _fail(
            "matrix source field inventory mismatch: "
            f"missing={sorted(expected_fields - set(source))}, "
            f"extra={sorted(set(source) - expected_fields)}"
        )
    sha = _string(source.get("sha"), "matrix source SHA")
    if OID40.fullmatch(sha) is None:
        _fail("matrix source SHA is not a lowercase Git object ID")
    if source.get("branch") != REQUIRED_BRANCH:
        _fail(f"matrix source branch is not {REQUIRED_BRANCH}")
    state = _string(source.get("git_state"), "matrix Git state")
    if state not in {"clean", "dirty"}:
        _fail("matrix Git state must be clean or dirty")
    status = _sequence(source.get("status_porcelain"), "matrix status_porcelain")
    if any(not isinstance(value, str) or value.startswith("?? ") for value in status):
        _fail("matrix source status contains invalid or untracked entries")
    expected_scope = None if state == "clean" else "tracked-only"
    if source.get("dirty_scope") != expected_scope:
        _fail("matrix source dirty_scope is inconsistent with Git state")
    if (not status) != (state == "clean"):
        _fail("matrix status_porcelain is inconsistent with Git state")
    if run_mode == "official":
        if state != "clean" or status:
            _fail("official matrix source must be clean")
        if source.get("upstream") != f"origin/{REQUIRED_BRANCH}":
            _fail("official matrix source records the wrong upstream")
        if source.get("upstream_sha") != sha or source.get("ahead_behind") != [
            "0",
            "0",
        ]:
            _fail("official matrix source is not upstream-equal 0/0")
    else:
        if any(
            source.get(key) is not None
            for key in ("upstream", "upstream_sha", "ahead_behind")
        ):
            _fail("nonofficial source must not claim an upstream equality check")
    source_prefix = "source-pilot/" if run_mode == "aggregation-replay" else ""
    if source.get("input_manifest") != f"{source_prefix}SOURCE_INPUT_MANIFEST.json":
        _fail("matrix source input-manifest path mismatch")
    if source.get("exact_input_archive") != f"{source_prefix}SOURCE_INPUT_BYTES.zip":
        _fail("matrix exact-input archive path mismatch")
    if source.get("canonical_git_archive") != f"{source_prefix}GIT_SOURCE_ARCHIVE.zip":
        _fail("matrix Git archive path mismatch")
    digest = _string(source.get("input_digest"), "matrix source input digest")
    if HEX64.fullmatch(digest) is None:
        _fail("matrix source input digest is not SHA-256")
    return sha, state


def _validate_binary_records(
    run_dir: Path, value: Any, *, path_prefix: str = ""
) -> Mapping[str, Mapping[str, Any]]:
    binaries = _mapping(value, "frozen_binaries")
    if set(binaries) != {"benchmark", "windows"}:
        _fail("frozen binary role inventory mismatch")
    expected_paths = {
        "benchmark": f"{path_prefix}frozen-binary/powdergame-benchmark.exe",
        "windows": f"{path_prefix}frozen-binary/powdergame-windows.exe",
    }
    result: dict[str, Mapping[str, Any]] = {}
    for role, expected_path in expected_paths.items():
        record = _mapping(binaries[role], f"frozen binary {role}")
        if set(record) != {"path", "size", "sha256", "build_profile"}:
            _fail(f"frozen binary {role} field inventory mismatch")
        if (
            record.get("path") != expected_path
            or record.get("build_profile") != "release"
        ):
            _fail(f"frozen binary {role} path/profile mismatch")
        path = run_dir.joinpath(*PurePosixPath(expected_path).parts)
        if not path.is_file():
            _fail(f"frozen binary {role} is missing")
        size = _integer(record.get("size"), f"frozen binary {role} size", minimum=1)
        digest = _string(record.get("sha256"), f"frozen binary {role} SHA-256")
        if HEX64.fullmatch(digest) is None:
            _fail(f"frozen binary {role} SHA-256 is malformed")
        if path.stat().st_size != size or sha256_file(path) != digest:
            _fail(f"frozen binary {role} size/hash mismatch")
        result[role] = dict(record)
    if result["benchmark"]["sha256"] == result["windows"]["sha256"]:
        _fail("the two role-distinct frozen executables unexpectedly have one hash")
    return result


def _validate_frozen_verifier(
    run_dir: Path,
    package_path: Path,
    sidecar_path: Path,
    record_value: Any,
    write_result: Path | None,
    repo_root: Path,
) -> Mapping[str, Any]:
    record = _mapping(record_value, "independent_verifier")
    expected_fields = {"path", "size", "sha256", "expected_argv", "execution_timing"}
    if set(record) != expected_fields:
        _fail("independent verifier field inventory mismatch")
    if record.get("path") != "verification/frozen-verifier.py":
        _fail("independent verifier path mismatch")
    frozen = run_dir / "verification" / "frozen-verifier.py"
    if not frozen.is_file():
        _fail("receipt-bound frozen verifier is missing")
    size = _integer(record.get("size"), "frozen verifier size", minimum=1)
    digest = _string(record.get("sha256"), "frozen verifier SHA-256")
    if (
        HEX64.fullmatch(digest) is None
        or frozen.stat().st_size != size
        or sha256_file(frozen) != digest
    ):
        _fail("frozen verifier size/hash binding mismatch")
    executing = Path(__file__).resolve()
    if executing != frozen.resolve() or sha256_file(executing) != digest:
        _fail(
            "verification must be executed by the receipt-bound frozen-verifier.py copy"
        )
    if not sys.dont_write_bytecode:
        _fail("frozen verifier must execute with -B/PYTHONDONTWRITEBYTECODE")
    expected_result = package_path.parent / "G8C_MATRIX_VERIFICATION.json"
    argv = [
        _string(value, f"independent_verifier.expected_argv[{index}]", nonempty=False)
        for index, value in enumerate(
            _sequence(record.get("expected_argv"), "independent_verifier.expected_argv")
        )
    ]
    expected_argv = [
        argv[0] if argv else "",
        "-B",
        str(frozen),
        "--run-dir",
        str(run_dir),
        "--package",
        str(package_path),
        "--sidecar",
        str(sidecar_path),
        "--write-result",
        str(expected_result),
        "--repo-root",
        str(repo_root),
    ]
    if len(argv) != len(expected_argv) or argv != expected_argv:
        _fail("independent verifier expected argv binding mismatch")
    if [sys.executable, "-B", *sys.argv] != argv:
        _fail("actual frozen-verifier argv differs from its manifest binding")
    try:
        if Path(argv[0]).resolve() != Path(sys.executable).resolve():
            _fail("independent verifier Python executable binding mismatch")
    except OSError as error:
        _fail(f"cannot resolve independent verifier Python executable: {error}")
    if record.get("execution_timing") != (
        "after receipt and package; result is delivery sibling and does not mutate matrix run"
    ):
        _fail("independent verifier execution timing contract mismatch")
    if write_result is not None and write_result.resolve() != expected_result.resolve():
        _fail("--write-result path differs from receipt-bound sibling result path")
    return dict(record)


def _scenario_matrix_row(
    scenario: str,
    source_sha: str,
    headless: Mapping[str, Any],
    coexistence: Mapping[str, Any],
    render: Mapping[str, Any],
) -> dict[str, Any]:
    group_values = {
        group: headless["mode_b"]["fields"][f"group_{group}_ms"]["p50"]
        for group in GROUPS
    }
    bottleneck = max(group_values, key=group_values.get)
    tracked = int(headless["memory"]["total_tracked"])
    capacity = 32 * 1024**3
    row: dict[str, Any] = {
        "source_sha": source_sha,
        "scenario": scenario,
        "mode_a_tps_p50": headless["mode_a"]["tps"]["p50"],
        "mode_a_tps_mean": headless["mode_a"]["tps"]["mean"],
        "mode_a_tps_min": headless["mode_a"]["tps"]["min"],
        "mode_a_tps_max": headless["mode_a"]["tps"]["max"],
        "mode_a_wall_ms_tick_p50": headless["mode_a"]["wall_ms_per_tick"]["p50"],
        "mode_a_wall_ms_tick_p95": headless["mode_a"]["wall_ms_per_tick"]["p95"],
        "headroom_60_tps_ratio": headless["mode_a"]["tps"]["p50"] / 60.0,
        "mode_b_gpu_envelope_p50_ms": headless["mode_b"]["fields"][
            "gpu_tick_envelope_ms"
        ]["p50"],
        "mode_b_gpu_envelope_p95_ms": headless["mode_b"]["fields"][
            "gpu_tick_envelope_ms"
        ]["p95"],
        "matter_movement_p50_ms": group_values["matter_movement"],
        "claim_resolve_p50_ms": group_values["ownership_claim"],
        "thermal_p50_ms": group_values["thermal_conduction"],
        "reaction_phase_p50_ms": group_values["reaction_phase"],
        "pressure_structure_p50_ms": group_values["pressure_structure"],
        "active_sleep_p50_ms": group_values["active_sleep_management"],
        "residual_p50_ms": headless["mode_b"]["fields"]["residual_ms"]["p50"],
        **headless["census"],
        "working_chunks": headless["census"]["runnable_chunks"],
        "tracked_persistent_gpu_bytes": tracked,
        "tracked_persistent_gpu_gib": tracked / 1024**3,
        "rtx_5090_32gib_tracked_memory_ratio": tracked / capacity,
        "rtx_5090_32gib_tracked_memory_headroom_bytes": capacity - tracked,
        "mode_c_simulation_tps": coexistence["simulation_tps"]["p50"],
        "mode_c_actual_simulation_ticks": coexistence["actual_simulation_ticks"],
        "mode_c_render_fps": coexistence["render_fps"]["p50"],
        "mode_c_presented_frames": coexistence["presented_frames"],
        "mode_c_frame_p50_ms": coexistence["frame_wall_ms"]["p50"],
        "mode_c_frame_p95_ms": coexistence["frame_wall_ms"]["p95"],
        "mode_c_frame_p99_ms": coexistence["frame_wall_ms"]["p99"],
        "mode_c_missed_deadline_ratio": coexistence["missed_deadline_ratio"],
        "mode_c_missed_simulation_deadlines": coexistence[
            "missed_simulation_deadlines"
        ],
        "mode_c_catch_up_ticks": coexistence["catch_up_ticks"],
        "mode_c_failed_surface_frames": coexistence["failed_surface_frames"],
        "mode_c_surface_errors": coexistence["surface_errors"],
        "mode_c_device_errors": coexistence["device_errors"],
        "mode_d_gpu_render_p50_ms": render["gpu_render_ms"]["p50"],
        "mode_d_gpu_render_p95_ms": render["gpu_render_ms"]["p95"],
        "mode_d_measured_frames": render["gpu_render_ms"]["count"],
        "mode_d_surface_errors": render["surface_errors"],
        "mode_d_device_errors": render["device_errors"],
        "bottleneck_group": GROUP_LABELS[bottleneck],
    }
    return row


def _optimization_recommendation(
    rows: Sequence[Mapping[str, Any]],
) -> tuple[str, list[str]]:
    blockers: list[str] = []
    for row in rows:
        scenario = row["scenario"]
        if row["mode_a_tps_p50"] < 60.0:
            blockers.append(f"{scenario}: Mode A P50 is below 60 TPS")
        if row["mode_b_gpu_envelope_p95_ms"] > 16.667:
            blockers.append(f"{scenario}: Mode B envelope P95 exceeds 16.667 ms")
        if row["mode_c_simulation_tps"] < 57.0:
            blockers.append(
                f"{scenario}: Mode C simulation TPS is below the 5% Fifo tolerance"
            )
        if row["mode_c_render_fps"] < 57.0:
            blockers.append(
                f"{scenario}: Mode C render FPS is below the 5% Fifo tolerance"
            )
        if row["mode_c_missed_deadline_ratio"] > 0.05:
            blockers.append(f"{scenario}: Mode C missed-deadline ratio exceeds 5%")
        if row["mode_c_frame_p95_ms"] > 33.334:
            blockers.append(f"{scenario}: Mode C frame P95 exceeds two 60 Hz frames")
        if row["mode_d_gpu_render_p95_ms"] > 16.667:
            blockers.append(f"{scenario}: Mode D GPU render P95 exceeds 16.667 ms")
        if row["mode_c_failed_surface_frames"] > 0:
            blockers.append(f"{scenario}: Mode C observed failed surface frames")
        if row["mode_c_surface_errors"] > 0 or row["mode_c_device_errors"] > 0:
            blockers.append(f"{scenario}: Mode C observed surface/device errors")
        if row["mode_d_surface_errors"] > 0 or row["mode_d_device_errors"] > 0:
            blockers.append(f"{scenario}: Mode D observed surface/device errors")
        if row["rtx_5090_32gib_tracked_memory_ratio"] > 0.75:
            blockers.append(
                f"{scenario}: app-tracked persistent GPU bytes exceed 75% of RTX 5090 32 GiB"
            )
    if blockers:
        return "OPTIMIZATION_REVIEW_REQUIRED", blockers
    strong = all(
        row["mode_a_tps_p50"] >= 120.0
        and row["mode_b_gpu_envelope_p95_ms"] <= 8.3335
        and row["mode_c_simulation_tps"] >= 57.0
        and row["mode_c_render_fps"] >= 57.0
        and row["mode_c_missed_deadline_ratio"] <= 0.01
        and row["mode_c_frame_p95_ms"] <= 25.0
        and row["mode_d_gpu_render_p95_ms"] <= 8.3335
        and row["rtx_5090_32gib_tracked_memory_ratio"] <= 0.50
        for row in rows
    )
    if strong:
        return "PROCEED_TO_G9", [
            "all five workloads preserve strong simulation and render headroom"
        ]
    return "NEEDS_HUMAN_REVIEW", [
        "no hard optimization blocker was measured, but one or more headroom margins are borderline"
    ]


def _assert_json_equal(observed: Any, expected: Any, label: str) -> None:
    if isinstance(expected, Mapping):
        if not isinstance(observed, Mapping) or set(observed) != set(expected):
            _fail(f"{label} object field inventory mismatch")
        for key in expected:
            _assert_json_equal(observed[key], expected[key], f"{label}.{key}")
        return
    if isinstance(expected, list):
        if not isinstance(observed, list) or len(observed) != len(expected):
            _fail(f"{label} list shape mismatch")
        for index, (left, right) in enumerate(zip(observed, expected)):
            _assert_json_equal(left, right, f"{label}[{index}]")
        return
    if isinstance(expected, float):
        if isinstance(observed, bool) or not isinstance(observed, (int, float)):
            _fail(f"{label} must be numeric")
        _assert_close(float(observed), expected, label)
        return
    if observed != expected:
        _fail(f"{label} mismatch: observed={observed!r}, expected={expected!r}")


def _validate_matrix_csv(path: Path, rows: Sequence[Mapping[str, Any]]) -> None:
    try:
        stream = path.open("r", encoding="utf-8", newline="")
    except OSError as error:
        _fail(f"cannot open matrix CSV: {error}")
    with stream:
        reader = csv.DictReader(stream)
        expected_header = tuple(rows[0])
        _check_header(reader, expected_header, "G8C_MATRIX.csv")
        observed_rows = list(reader)
    if len(observed_rows) != len(rows):
        _fail("G8C_MATRIX.csv scenario row count mismatch")
    for row_index, (observed, expected) in enumerate(zip(observed_rows, rows)):
        _require_row_complete(observed, f"G8C_MATRIX.csv row {row_index + 2}")
        for key, expected_value in expected.items():
            value = observed[key]
            if isinstance(expected_value, (int, float)) and not isinstance(
                expected_value, bool
            ):
                _assert_close(
                    _parse_float(value, f"matrix CSV row {row_index + 2}.{key}"),
                    float(expected_value),
                    f"matrix CSV row {row_index + 2}.{key}",
                )
            elif value != str(expected_value):
                _fail(f"matrix CSV row {row_index + 2}.{key} mismatch")


def _validate_reports(
    run_dir: Path,
    manifest_reports_value: Any,
    *,
    matrix_run_id: str,
    run_mode: str,
    rows: Sequence[Mapping[str, Any]],
    recommendation: str,
    reasons: Sequence[str],
) -> Mapping[str, str]:
    reports = _mapping(manifest_reports_value, "matrix reports")
    expected_paths = {
        "matrix_csv": "report/G8C_MATRIX.csv",
        "matrix_json": "report/G8C_MATRIX.json",
        "report": "report/G8C_REPORT.md",
        "bottleneck_analysis": "report/BOTTLENECK_ANALYSIS.md",
        "optimization_decision": "report/OPTIMIZATION_DECISION.md",
    }
    if reports != expected_paths:
        _fail("matrix report path/role contract mismatch")
    paths = {
        role: run_dir.joinpath(*PurePosixPath(relative).parts)
        for role, relative in expected_paths.items()
    }
    if any(not path.is_file() for path in paths.values()):
        _fail("one or more required matrix reports are missing")
    _validate_matrix_csv(paths["matrix_csv"], rows)
    expected_json = {
        "schema_version": REPORT_SCHEMA,
        "matrix_run_id": matrix_run_id,
        "run_mode": run_mode,
        "official_evidence": run_mode == "official",
        "pilot_must_never_be_promoted": run_mode in {"pilot", "aggregation-replay"},
        "scenarios": list(rows),
        "recommendation": recommendation,
        "recommendation_reasons": list(reasons),
        "activity_count_note": "cell activity subsystem counts overlap and must not be summed",
    }
    _assert_json_equal(
        _read_json_file(paths["matrix_json"], "G8C_MATRIX.json"),
        expected_json,
        "G8C_MATRIX.json",
    )

    title = (
        "G8-C Official Performance Matrix"
        if run_mode == "official"
        else "G8-C Matrix Pilot (NON-EVIDENCE)"
    )
    report_lines = [
        f"# {title}",
        "",
        f"- Matrix: `{matrix_run_id}`",
        f"- Run mode: `{run_mode}`",
        f"- Recommendation: **{recommendation}**",
        "- Fifo render FPS is interpreted with frame percentiles and deadline ratios, not an exact integer-60 comparison.",
        "- Cell activity subsystem counts overlap and are not summed.",
        "- Memory is app-tracked persistent GPU allocation, not total driver-resident VRAM; the guard uses the RTX 5090 32 GiB capacity.",
        "",
        "| Scenario | Mode A TPS P50 | Mode B envelope P95 ms | Mode C sim TPS | Mode C FPS | Frame P95 ms | Mode D render P95 ms | Tracked GiB | Bottleneck |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---|",
    ]
    for row in rows:
        report_lines.append(
            f"| {row['scenario']} | {row['mode_a_tps_p50']:.3f} | "
            f"{row['mode_b_gpu_envelope_p95_ms']:.3f} | "
            f"{row['mode_c_simulation_tps']:.3f} | {row['mode_c_render_fps']:.3f} | "
            f"{row['mode_c_frame_p95_ms']:.3f} | "
            f"{row['mode_d_gpu_render_p95_ms']:.3f} | "
            f"{row['tracked_persistent_gpu_gib']:.3f} | {row['bottleneck_group']} |"
        )
    expected_report = "\n".join(report_lines) + "\n"
    pilot_warning = (
        ["**NON-EVIDENCE PILOT: never promote this output.**", ""]
        if run_mode in {"pilot", "aggregation-replay"}
        else []
    )
    bottleneck_lines = ["# G8-C Bottleneck Analysis", "", *pilot_warning]
    for row in rows:
        bottleneck_lines.append(
            f"- **{row['scenario']}**: {row['bottleneck_group']} is the largest Mode B grouped P50; "
            f"GPU envelope P95 is {row['mode_b_gpu_envelope_p95_ms']:.3f} ms; "
            f"app-tracked persistent memory is {row['tracked_persistent_gpu_gib']:.3f} GiB "
            f"({row['rtx_5090_32gib_tracked_memory_ratio']:.1%} of 32 GiB)."
        )
    expected_bottleneck = "\n".join(bottleneck_lines) + "\n"
    decision_lines = [
        "# G8-C Optimization Decision",
        "",
        *pilot_warning,
        f"Recommendation: **{recommendation}**",
        "",
        *[f"- {reason}" for reason in reasons],
        "",
        "This report does not authorize or begin optimization or G9 work.",
    ]
    expected_decision = "\n".join(decision_lines) + "\n"
    expected_text = {
        "report": expected_report,
        "bottleneck_analysis": expected_bottleneck,
        "optimization_decision": expected_decision,
    }
    for role, expected in expected_text.items():
        try:
            observed = paths[role].read_text(encoding="utf-8")
        except (OSError, UnicodeError) as error:
            _fail(f"cannot read {role}: {error}")
        if observed != expected:
            _fail(f"{role} does not match independently reconstructed content")
    return expected_paths


def _validate_receipt(
    run_dir: Path,
    package_path: Path,
    sidecar_path: Path,
    *,
    receipt_value: Mapping[str, Any],
    manifest: Mapping[str, Any],
    hashes: Mapping[str, str],
    hashes_sha256: str,
    binaries: Mapping[str, Mapping[str, Any]],
    reports: Mapping[str, str],
    verifier: Mapping[str, Any],
    source_sha: str,
    source_digest: str,
    recommendation: str,
    latest_process_end: datetime,
) -> None:
    expected_fields = {
        "schema_version",
        "matrix_run_id",
        "run_mode",
        "complete",
        "receipt_is_final_publication_marker",
        "published_at_utc",
        "source_sha",
        "source_input_digest",
        "manifest_sha256",
        "hashes_sha256",
        "hash_entry_count",
        "frozen_binaries",
        "reports",
        "independent_verifier",
        "recommendation",
        "delivery",
    }
    if set(receipt_value) != expected_fields:
        _fail("receipt field inventory mismatch")
    exact_scalars = {
        "schema_version": RECEIPT_SCHEMA,
        "matrix_run_id": manifest["matrix_run_id"],
        "run_mode": manifest["run_mode"],
        "complete": True,
        "receipt_is_final_publication_marker": True,
        "source_sha": source_sha,
        "source_input_digest": source_digest,
        "manifest_sha256": sha256_file(run_dir / "G8C_MATRIX_MANIFEST.json"),
        "hashes_sha256": hashes_sha256,
        "hash_entry_count": len(hashes),
        "recommendation": recommendation,
    }
    for key, expected in exact_scalars.items():
        if receipt_value.get(key) != expected:
            _fail(f"receipt {key} mismatch")
    published = _parse_rfc3339_utc(
        receipt_value.get("published_at_utc"), "receipt published_at_utc"
    )
    if published < latest_process_end:
        _fail("receipt publication predates a captured build/measurement process")
    if receipt_value.get("frozen_binaries") != binaries:
        _fail("receipt frozen-binary binding mismatch")
    if receipt_value.get("independent_verifier") != verifier:
        _fail("receipt frozen-verifier binding mismatch")
    expected_report_records = {
        role: {
            "path": relative,
            "sha256": sha256_file(run_dir.joinpath(*PurePosixPath(relative).parts)),
        }
        for role, relative in reports.items()
    }
    if receipt_value.get("reports") != expected_report_records:
        _fail("receipt report hash/path binding mismatch")
    delivery = _mapping(receipt_value.get("delivery"), "receipt delivery")
    expected_delivery = {
        "sibling_directory": f"{run_dir.name}-delivery",
        "package_filename": "G8C_MATRIX_PACKAGE.zip",
        "package_sha256_sidecar": "G8C_MATRIX_PACKAGE_SHA256.txt",
        "hash_binding": "sibling sidecar is created after this receipt and hashes the ZIP64 package containing this receipt",
    }
    if delivery != expected_delivery:
        _fail("receipt delivery contract mismatch")
    expected_delivery_dir = run_dir.parent / expected_delivery["sibling_directory"]
    if package_path != expected_delivery_dir / expected_delivery["package_filename"]:
        _fail("package is not in the receipt-bound sibling delivery directory")
    if (
        sidecar_path
        != expected_delivery_dir / expected_delivery["package_sha256_sidecar"]
    ):
        _fail("sidecar is not in the receipt-bound sibling delivery directory")


def _validate_scenario_sequence(values: Sequence[Any]) -> None:
    observed = [
        _string(
            _mapping(value, f"matrix scenarios[{index}]").get("scenario"),
            f"matrix scenarios[{index}].scenario",
        )
        for index, value in enumerate(values)
    ]
    if observed != list(SCENARIOS):
        counts = Counter(observed)
        missing = sorted(set(SCENARIOS) - set(observed))
        duplicates = sorted(name for name, count in counts.items() if count > 1)
        extras = sorted(set(observed) - set(SCENARIOS))
        _fail(
            "matrix scenarios must appear exactly once in canonical order: "
            f"missing={missing}, duplicates={duplicates}, extras={extras}"
        )


def verify_matrix(
    run_dir: Path,
    package_path: Path,
    sidecar_path: Path,
    *,
    repo_root: Path,
    write_result: Path | None = None,
) -> Mapping[str, Any]:
    """Independently verify one immutable matrix run and its ZIP64 delivery."""
    try:
        run_dir = run_dir.resolve(strict=True)
        package_path = package_path.resolve(strict=True)
        sidecar_path = sidecar_path.resolve(strict=True)
        repo_root = repo_root.resolve(strict=True)
    except OSError as error:
        _fail(f"verification input path is missing or inaccessible: {error}")
    if not run_dir.is_dir() or not package_path.is_file() or not sidecar_path.is_file():
        _fail(
            "verification inputs must be one run directory, package file, and sidecar file"
        )
    if package_path.name != "G8C_MATRIX_PACKAGE.zip":
        _fail("package filename is not canonical")
    if sidecar_path.name != "G8C_MATRIX_PACKAGE_SHA256.txt":
        _fail("package sidecar filename is not canonical")
    if write_result is not None:
        write_result = write_result.resolve(strict=False)
        if write_result.exists():
            _fail(f"refusing to overwrite existing verification result: {write_result}")

    inventory = _inventory_run(run_dir)
    manifest = _read_json_file(
        run_dir / "G8C_MATRIX_MANIFEST.json", "G8C_MATRIX_MANIFEST.json"
    )
    provisional_run_mode = _string(manifest.get("run_mode"), "matrix run_mode")
    hashes, hashes_sha256 = _validate_run_hashes(
        run_dir,
        inventory,
        run_mode=provisional_run_mode,
    )
    expected_manifest_fields = {
        "schema_version",
        "matrix_run_id",
        "run_mode",
        "official_evidence",
        "pilot_must_never_be_promoted",
        "source",
        "common_config",
        "hardware_policy",
        "scenario_order",
        "frozen_binaries",
        "build_command_record",
        "independent_verifier",
        "command_record_paths",
        "scenarios",
        "reports",
        "estimated_unpacked_bytes_before_capture",
        "recommendation",
    }
    if manifest.get("run_mode") == "aggregation-replay":
        expected_manifest_fields.add("aggregation_replay")
    if set(manifest) != expected_manifest_fields:
        _fail(
            "matrix manifest field inventory mismatch: "
            f"missing={sorted(expected_manifest_fields - set(manifest))}, "
            f"extra={sorted(set(manifest) - expected_manifest_fields)}"
        )
    if manifest.get("schema_version") != MATRIX_SCHEMA:
        _fail("matrix manifest schema mismatch")
    run_mode = _string(manifest.get("run_mode"), "matrix run_mode")
    if run_mode != provisional_run_mode:
        _fail("matrix run mode changed while its hash inventory was checked")
    if run_mode not in {"pilot", "official", "aggregation-replay"}:
        _fail("matrix run_mode must be pilot, official, or aggregation-replay")
    matrix_run_id = _string(manifest.get("matrix_run_id"), "matrix run ID")
    if matrix_run_id != run_dir.name:
        _fail("matrix run ID differs from its immutable run directory name")
    source = _mapping(manifest.get("source"), "matrix source")
    source_sha, source_state = _validate_source_identity(source, run_mode)
    source_digest = _string(source.get("input_digest"), "source input digest")
    if run_mode == "aggregation-replay":
        if not matrix_run_id.startswith("g8c-aggregation-replay-"):
            _fail("aggregation replay matrix run ID has the wrong prefix")
    else:
        expected_run_id = (
            "g8c-pilot" if run_mode == "pilot" else "g8c-official-matrix"
        ) + f"-{source_sha[:12]}-{source_digest[:12]}"
        if matrix_run_id != expected_run_id:
            _fail("matrix run ID is not derived from source SHA and exact-input digest")
    if manifest.get("official_evidence") is not (run_mode == "official"):
        _fail("matrix official_evidence flag mismatch")
    if manifest.get("pilot_must_never_be_promoted") is not (
        run_mode in {"pilot", "aggregation-replay"}
    ):
        _fail("matrix pilot non-promotion flag mismatch")

    profile = _mapping(manifest.get("common_config"), "matrix common_config")
    if profile != _expected_profile(run_mode):
        _fail("matrix common config does not match the frozen measurement profile")
    config = _parse_common_config(profile, run_mode)
    hardware = {
        "adapter": "NVIDIA RTX 5090",
        "vendor_id": "0x10DE",
        "backend": "Dx12",
        "tracked_memory_capacity_bytes": 32 * 1024**3,
        "tracked_memory_note": "application-tracked persistent GPU bytes, not total driver-resident VRAM",
    }
    if manifest.get("hardware_policy") != hardware:
        _fail("matrix hardware policy mismatch")
    if run_mode == "aggregation-replay":
        replay_for_estimate = _mapping(
            manifest.get("aggregation_replay"), "aggregation_replay"
        )
        expected_estimate = _integer(
            replay_for_estimate.get("source_pilot_total_bytes"),
            "aggregation replay source pilot bytes",
            minimum=1,
        )
    else:
        cells = config.width * config.height
        chunks = math.ceil(config.width / config.chunk_size) * math.ceil(
            config.height / config.chunk_size
        )
        expected_estimate = (
            len(SCENARIOS)
            * (cells * 135 + chunks * 160 + config.trials * config.mode_b_ticks * 8_000)
            + 900 * 1024 * 1024
        )
    if manifest.get("estimated_unpacked_bytes_before_capture") != expected_estimate:
        _fail("pre-capture storage estimate mismatch")

    replay_mode = run_mode == "aggregation-replay"
    artifact_prefix = "source-pilot/" if replay_mode else ""
    binaries = _validate_binary_records(
        run_dir,
        manifest.get("frozen_binaries"),
        path_prefix=artifact_prefix,
    )
    verifier_record = _validate_frozen_verifier(
        run_dir,
        package_path,
        sidecar_path,
        manifest.get("independent_verifier"),
        write_result,
        repo_root,
    )
    replay_binding: AggregationReplayBinding | None = None
    measurement_root = run_dir
    measurement_run_id = matrix_run_id
    recorded_measurement_root: Path | None = None
    measurement_binaries = binaries
    if replay_mode:
        replay_binding = _validate_aggregation_replay(run_dir, manifest, profile)
        measurement_root = replay_binding.copied_root
        measurement_run_id = replay_binding.source_pilot_id
        recorded_measurement_root = replay_binding.original_root
        measurement_binaries = {
            role: {
                **record,
                "path": _string(record["path"], f"{role}.path").removeprefix(
                    "source-pilot/"
                ),
            }
            for role, record in binaries.items()
        }
        pilot_source = dict(source)
        for field in (
            "input_manifest",
            "exact_input_archive",
            "canonical_git_archive",
        ):
            pilot_source[field] = _string(
                pilot_source[field], f"matrix source {field}"
            ).removeprefix("source-pilot/")
        source_result = _validate_source_inputs(
            measurement_root,
            {
                "matrix_run_id": measurement_run_id,
                "source": pilot_source,
            },
            source_sha,
            "pilot",
            None,
        )
    else:
        process_result = _validate_process_records(run_dir, manifest, profile)
        source_result = _validate_source_inputs(
            run_dir, manifest, source_sha, run_mode, repo_root
        )
        if source_result["live_git"].get("checked") is not True:
            _fail(
                "independent verification did not complete the required live Git check"
            )
    process_result = (
        replay_binding.process_result if replay_binding is not None else process_result
    )

    if manifest.get("scenario_order") != list(SCENARIOS):
        _fail("matrix scenario_order must contain the five scenarios exactly once")
    scenario_values = _sequence(manifest.get("scenarios"), "matrix scenarios")
    if len(scenario_values) != len(SCENARIOS):
        _fail("matrix scenario record count mismatch")
    _validate_scenario_sequence(scenario_values)
    scenario_results: list[dict[str, Any]] = []
    adapters: list[Mapping[str, Any]] = []
    expected_scenario_fields = {
        "scenario",
        "headless_manifest",
        "headless_summary",
        "raw_ticks",
        "raw_cells",
        "raw_chunks",
        "coexistence_csv",
        "coexistence_metadata",
        "render_profile_csv",
        "render_profile_metadata",
    }
    for expected_scenario, value in zip(SCENARIOS, scenario_values):
        record = _mapping(value, f"scenario record {expected_scenario}")
        if set(record) != expected_scenario_fields:
            _fail(f"scenario record field inventory mismatch for {expected_scenario}")
        if record.get("scenario") != expected_scenario:
            _fail("matrix scenarios are missing, duplicated, or out of canonical order")
        paths = _scenario_paths(
            record,
            expected_scenario,
            path_prefix=artifact_prefix,
        )
        headless = _validate_headless(
            measurement_root,
            scenario=expected_scenario,
            paths=paths,
            matrix_run_id=measurement_run_id,
            source_sha=source_sha,
            source_state=source_state,
            benchmark_binary=measurement_binaries["benchmark"],
            profile=profile,
            config=config,
        )
        coexistence = _validate_window_mode(
            measurement_root,
            mode="coexistence",
            scenario=expected_scenario,
            paths=paths,
            matrix_run_id=measurement_run_id,
            source_sha=source_sha,
            source_state=source_state,
            windows_binary=measurement_binaries["windows"],
            profile=profile,
            config=config,
            recorded_run_root=recorded_measurement_root,
        )
        render = _validate_window_mode(
            measurement_root,
            mode="render-profile",
            scenario=expected_scenario,
            paths=paths,
            matrix_run_id=measurement_run_id,
            source_sha=source_sha,
            source_state=source_state,
            windows_binary=measurement_binaries["windows"],
            profile=profile,
            config=config,
            recorded_run_root=recorded_measurement_root,
        )
        if coexistence["adapter"] != render["adapter"]:
            _fail(f"{expected_scenario} Mode C/D adapter identity mismatch")
        if headless["mode_a"]["adapter"] != coexistence["adapter"]:
            _fail(f"{expected_scenario} headless/windowed adapter identity mismatch")
        adapters.append(coexistence["adapter"])
        scenario_results.append(
            _scenario_matrix_row(
                expected_scenario, source_sha, headless, coexistence, render
            )
        )
    if any(adapter != adapters[0] for adapter in adapters[1:]):
        _fail("adapter identity changed across the five scenario captures")

    if run_mode == "pilot":
        recommendation, reasons = (
            "NEEDS_HUMAN_REVIEW",
            [
                "non-evidence pilot validates orchestration only and must never be used for a G9 decision"
            ],
        )
    elif run_mode == "aggregation-replay":
        recommendation, reasons = (
            "NEEDS_HUMAN_REVIEW",
            [
                "aggregation replay reuses non-evidence pilot measurements for parser validation only"
            ],
        )
    else:
        recommendation, reasons = _optimization_recommendation(scenario_results)
    for row in scenario_results:
        row["total_recommendation_flag"] = recommendation
    if manifest.get("recommendation") != recommendation:
        _fail("matrix manifest recommendation differs from raw-input reconstruction")
    reports = _validate_reports(
        run_dir,
        manifest.get("reports"),
        matrix_run_id=matrix_run_id,
        run_mode=run_mode,
        rows=scenario_results,
        recommendation=recommendation,
        reasons=reasons,
    )
    receipt = _read_json_file(
        run_dir / "G8C_MATRIX_RECEIPT.json", "G8C_MATRIX_RECEIPT.json"
    )
    _validate_receipt(
        run_dir,
        package_path,
        sidecar_path,
        receipt_value=receipt,
        manifest=manifest,
        hashes=hashes,
        hashes_sha256=hashes_sha256,
        binaries=binaries,
        reports=reports,
        verifier=verifier_record,
        source_sha=source_sha,
        source_digest=source_digest,
        recommendation=recommendation,
        latest_process_end=process_result["latest_end_utc"],
    )
    package_sha256, package_size = _validate_package_copy(
        run_dir, package_path, sidecar_path, inventory
    )
    replay_result: Mapping[str, Any] | None = None
    if replay_binding is not None:
        _validate_replay_original_unchanged(replay_binding)
        replay_result = {
            "source_pilot_id": replay_binding.source_pilot_id,
            "source_pilot_path": str(replay_binding.original_root),
            "source_pilot_file_count": len(replay_binding.original_inventory_before),
            "source_pilot_total_bytes": sum(
                entry.size_bytes
                for entry in replay_binding.original_inventory_before.values()
            ),
            "source_pilot_inventory_digest": _inventory_digest(
                replay_binding.original_inventory_before
            ),
            "source_pilot_capture_process_records": process_result["count"],
            "replay_launched_processes": 0,
            "replay_implementation": replay_binding.implementation,
            "original_unchanged": True,
            "copied_inputs_verified": True,
        }
    result: dict[str, Any] = {
        "schema_version": VERIFICATION_SCHEMA,
        "verified": True,
        "verified_at_utc": datetime.now(timezone.utc)
        .isoformat(timespec="microseconds")
        .replace("+00:00", "Z"),
        "matrix_run_id": matrix_run_id,
        "run_mode": run_mode,
        "source_sha": source_sha,
        "source_input_digest": source_digest,
        "manifest_sha256": sha256_file(run_dir / "G8C_MATRIX_MANIFEST.json"),
        "receipt_sha256": sha256_file(run_dir / "G8C_MATRIX_RECEIPT.json"),
        "hashes_sha256": hashes_sha256,
        "package_sha256": package_sha256,
        "package_size_bytes": package_size,
        "frozen_verifier": {
            "path": verifier_record["path"],
            "sha256": verifier_record["sha256"],
            "executed_path": str(Path(__file__).resolve()),
            "argv": [sys.executable, "-B", *sys.argv],
        },
        "scenario_order": list(SCENARIOS),
        "scenario_rows": scenario_results,
        "recommendation": recommendation,
        "recommendation_reasons": reasons,
        "checks": {
            "receipt_finality": "PASS",
            "source_input_and_git_archive": "PASS",
            "frozen_binaries": "PASS",
            "commands_and_logs": 0 if replay_mode else process_result["count"],
            "source_pilot_capture_records": (
                process_result["count"] if replay_mode else None
            ),
            "mode_a_b_c_d_reconstruction": "PASS",
            "matrix_reports": "PASS",
            "package_exact_copy": "PASS",
            "run_file_count": len(inventory),
        },
        "source_verification": source_result,
    }
    if replay_result is not None:
        result["aggregation_replay"] = replay_result
    return result


def _write_verification_result(path: Path, result: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = (
        json.dumps(result, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    ).encode("utf-8")
    try:
        with path.open("xb") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
    except FileExistsError:
        _fail(f"refusing to overwrite existing verification result: {path}")
    except OSError as error:
        _fail(f"cannot publish verification result {path}: {error}")


def _parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Independently verify one immutable Powdergame G8-C matrix package."
    )
    parser.add_argument("--run-dir", required=True, type=Path)
    parser.add_argument("--package", required=True, type=Path)
    parser.add_argument("--sidecar", required=True, type=Path)
    parser.add_argument("--write-result", type=Path)
    parser.add_argument("--repo-root", required=True, type=Path)
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    arguments = _parse_args(argv)
    try:
        result = verify_matrix(
            arguments.run_dir,
            arguments.package,
            arguments.sidecar,
            repo_root=arguments.repo_root,
            write_result=arguments.write_result,
        )
        if arguments.write_result is not None:
            _write_verification_result(arguments.write_result, result)
        print(
            json.dumps(
                {
                    "verified": True,
                    "matrix_run_id": result["matrix_run_id"],
                    "recommendation": result["recommendation"],
                    "package_sha256": result["package_sha256"],
                },
                sort_keys=True,
            )
        )
        return 0
    except VerificationError as error:
        print(f"G8-C independent verification failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
