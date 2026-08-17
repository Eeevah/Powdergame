#!/usr/bin/env python3
"""Create one immutable external Sand Fall experiment packet.

Operational failures intentionally leave the unique run directory in place
without EXPERIMENT_RECEIPT.json. A failed run is never repaired or reused.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import re
import secrets
import subprocess
import sys
import tomllib
import zipfile
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Any, Iterable, Sequence


EXPERIMENT_ID = "g8b-sand-fall-v0"
SCENARIO = "sand-fall"
DEFAULT_ARTIFACT_ROOT = Path(r"C:\Users\mdkap\source\Powdergame-artifacts")

MANIFEST_SCHEMA = "powdergame-experiment-manifest-v0"
ANALYSIS_SCHEMA = "powdergame-experiment-analysis-v0"
FRAMES_SCHEMA = "powdergame-experiment-frames-v0"
TELEMETRY_SCHEMA = "powdergame-experiment-telemetry-v0"
REPORT_SCHEMA = "powdergame-experiment-report-v0"
RECEIPT_SCHEMA = "powdergame-experiment-receipt-v0"

WORLD_WIDTH = 256
WORLD_HEIGHT = 256
CHUNK_SIZE = 64
MAX_TICKS = 20_000
DIAGNOSTIC_INTERVAL = 8
CONSECUTIVE_ALL_SLEEP = 3
POST_SLEEP_TICKS = 180

RENDERER_WIDTH = 1_600
RENDERER_HEIGHT = 900
CROP_X = 420
CROP_Y = 60
CROP_WIDTH = 760
CROP_HEIGHT = 760

ALLOWED_VERDICTS = {"PASS", "FAIL", "NEEDS_HUMAN"}
PREDICATE_NAMES = {
    "actual_fall",
    "matter_conservation",
    "no_invalid_materials",
    "no_nonfinite_fields",
    "sleep_before_max",
    "post_sleep_stable",
    "exact_reset",
}
PREDICATE_STATUSES = {"pass", "fail", "unknown"}
HEX64 = re.compile(r"^[0-9a-fA-F]{64}$")
GIT_OID = re.compile(r"^(?:[0-9a-fA-F]{40}|[0-9a-fA-F]{64})$")
STATE_HASH = re.compile(r"^fnv1a64:[0-9a-f]{16}$")

MANIFEST_TOP_KEYS = {
    "schema_version",
    "experiment_id",
    "run_id",
    "scenario",
    "created_utc",
    "source",
    "binary",
    "artifact",
    "world",
    "experiment",
    "renderer",
    "commands",
}
MANIFEST_SECTION_KEYS = {
    "source": {"root", "branch", "sha", "git_state"},
    "binary": {"path", "sha256", "build_profile"},
    "artifact": {"root", "run_dir"},
    "world": {"width", "height", "chunk_size"},
    "experiment": {
        "max_ticks",
        "diagnostic_interval_ticks",
        "consecutive_all_sleep",
        "post_sleep_ticks",
    },
    "renderer": {
        "full_width",
        "full_height",
        "crop_x",
        "crop_y",
        "crop_width",
        "crop_height",
    },
    "commands": {"build", "worker"},
}


class ExperimentError(RuntimeError):
    """An operational or artifact-contract failure."""


@dataclass(frozen=True)
class SourceInfo:
    root: Path
    branch: str
    sha: str
    git_state: str = "clean"


@dataclass(frozen=True)
class ManifestData:
    run_id: str
    created_utc: str
    source: SourceInfo
    binary_path: Path
    binary_sha256: str
    artifact_root: Path
    run_dir: Path
    build_command: tuple[str, ...]
    worker_command: tuple[str, ...]

    def as_dict(self) -> dict[str, Any]:
        return {
            "schema_version": MANIFEST_SCHEMA,
            "experiment_id": EXPERIMENT_ID,
            "run_id": self.run_id,
            "scenario": SCENARIO,
            "created_utc": self.created_utc,
            "source": {
                "root": str(self.source.root),
                "branch": self.source.branch,
                "sha": self.source.sha,
                "git_state": self.source.git_state,
            },
            "binary": {
                "path": str(self.binary_path),
                "sha256": self.binary_sha256,
                "build_profile": "release",
            },
            "artifact": {
                "root": str(self.artifact_root),
                "run_dir": str(self.run_dir),
            },
            "world": {
                "width": WORLD_WIDTH,
                "height": WORLD_HEIGHT,
                "chunk_size": CHUNK_SIZE,
            },
            "experiment": {
                "max_ticks": MAX_TICKS,
                "diagnostic_interval_ticks": DIAGNOSTIC_INTERVAL,
                "consecutive_all_sleep": CONSECUTIVE_ALL_SLEEP,
                "post_sleep_ticks": POST_SLEEP_TICKS,
            },
            "renderer": {
                "full_width": RENDERER_WIDTH,
                "full_height": RENDERER_HEIGHT,
                "crop_x": CROP_X,
                "crop_y": CROP_Y,
                "crop_width": CROP_WIDTH,
                "crop_height": CROP_HEIGHT,
            },
            "commands": {
                "build": list(self.build_command),
                "worker": list(self.worker_command),
            },
        }


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


def format_utc(value: datetime) -> str:
    return value.astimezone(timezone.utc).isoformat(timespec="microseconds").replace(
        "+00:00", "Z"
    )


def generate_run_id(now: datetime | None = None) -> str:
    value = (now or utc_now()).astimezone(timezone.utc)
    stamp = value.strftime("%Y%m%dT%H%M%S") + f"{value.microsecond:06d}Z"
    return f"{EXPERIMENT_ID}-{stamp}-{secrets.token_hex(4)}"


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def write_new_bytes(
    path: Path, data: bytes, publication_log: list[str] | None = None
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        with path.open("xb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
    except FileExistsError as error:
        raise ExperimentError(f"refusing to overwrite existing artifact: {path}") from error
    if publication_log is not None:
        publication_log.append(path.name)


def write_new_text(
    path: Path, text: str, publication_log: list[str] | None = None
) -> None:
    write_new_bytes(path, text.encode("utf-8"), publication_log)


def create_run_directory(artifact_root: Path, run_id: str) -> Path:
    if not run_id or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", run_id):
        raise ExperimentError(f"invalid run ID: {run_id!r}")
    artifact_root.mkdir(parents=True, exist_ok=True)
    run_dir = artifact_root / run_id
    try:
        run_dir.mkdir()
    except FileExistsError as error:
        raise ExperimentError(f"run ID already exists and cannot be reused: {run_id}") from error
    return run_dir


def is_path_within(path: Path, parent: Path) -> bool:
    path_resolved = path.resolve()
    parent_resolved = parent.resolve()
    try:
        common = os.path.commonpath([str(path_resolved), str(parent_resolved)])
        return os.path.normcase(common) == os.path.normcase(
            str(parent_resolved)
        )
    except ValueError:
        return False


def validate_external_artifact_root(source_root: Path, artifact_root: Path) -> None:
    source = source_root.resolve()
    artifacts = artifact_root.resolve()
    if artifacts == source or is_path_within(artifacts, source):
        raise ExperimentError(
            f"artifact root must be outside the source repository: {artifacts}"
        )


def git_text(source_root: Path, *args: str) -> str:
    completed = subprocess.run(
        ["git", *args],
        cwd=source_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if completed.returncode != 0:
        detail = completed.stderr.decode("utf-8", errors="replace").strip()
        raise ExperimentError(f"git {' '.join(args)} failed: {detail}")
    return completed.stdout.decode("utf-8", errors="strict").strip()


def inspect_clean_named_source(source_root: Path) -> SourceInfo:
    source = source_root.resolve(strict=True)
    branch = git_text(source, "branch", "--show-current")
    if not branch or branch == "HEAD":
        raise ExperimentError("experiment source must be on a named branch, not detached HEAD")
    status = git_text(source, "status", "--porcelain", "--untracked-files=all")
    if status:
        raise ExperimentError("experiment source must be clean; dirty/untracked paths detected")
    sha = git_text(source, "rev-parse", "HEAD")
    if not GIT_OID.fullmatch(sha):
        raise ExperimentError(f"git returned an invalid source SHA: {sha!r}")
    return SourceInfo(root=source, branch=branch, sha=sha)


def run_logged(
    command: Sequence[str], cwd: Path, stdout_path: Path, stderr_path: Path
) -> int:
    completed = subprocess.run(
        list(command),
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    write_new_bytes(stdout_path, completed.stdout)
    write_new_bytes(stderr_path, completed.stderr)
    return completed.returncode


def toml_quote(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def toml_array(values: Sequence[str]) -> str:
    return "[" + ", ".join(toml_quote(value) for value in values) + "]"


def render_manifest(manifest: ManifestData) -> str:
    data = manifest.as_dict()
    lines = [
        f"schema_version = {toml_quote(data['schema_version'])}",
        f"experiment_id = {toml_quote(data['experiment_id'])}",
        f"run_id = {toml_quote(data['run_id'])}",
        f"scenario = {toml_quote(data['scenario'])}",
        f"created_utc = {toml_quote(data['created_utc'])}",
    ]
    for section in ("source", "binary", "artifact", "world", "experiment", "renderer"):
        lines.extend(["", f"[{section}]"])
        for key, value in data[section].items():
            if isinstance(value, str):
                lines.append(f"{key} = {toml_quote(value)}")
            elif isinstance(value, int) and not isinstance(value, bool):
                lines.append(f"{key} = {value}")
            else:
                raise ExperimentError(f"unsupported manifest value {section}.{key}")
    lines.extend(
        [
            "",
            "[commands]",
            f"build = {toml_array(data['commands']['build'])}",
            f"worker = {toml_array(data['commands']['worker'])}",
            "",
        ]
    )
    return "\n".join(lines)


def require_exact_keys(value: dict[str, Any], expected: set[str], label: str) -> None:
    actual = set(value)
    if actual != expected:
        missing = sorted(expected - actual)
        extra = sorted(actual - expected)
        raise ExperimentError(f"{label} keys mismatch; missing={missing}, extra={extra}")


def validate_manifest_dict(data: dict[str, Any]) -> None:
    require_exact_keys(data, MANIFEST_TOP_KEYS, "manifest")
    if data["schema_version"] != MANIFEST_SCHEMA:
        raise ExperimentError("manifest schema_version mismatch")
    if data["experiment_id"] != EXPERIMENT_ID or data["scenario"] != SCENARIO:
        raise ExperimentError("manifest experiment/scenario mismatch")
    if not isinstance(data["run_id"], str) or not re.fullmatch(
        r"[A-Za-z0-9][A-Za-z0-9._-]*", data["run_id"]
    ):
        raise ExperimentError("manifest run_id must be a safe non-empty identifier")
    if not isinstance(data["created_utc"], str):
        raise ExperimentError("manifest created_utc must be a string")
    try:
        created = datetime.fromisoformat(data["created_utc"].replace("Z", "+00:00"))
    except ValueError as error:
        raise ExperimentError("manifest created_utc must be an ISO-8601 timestamp") from error
    if created.tzinfo is None or created.utcoffset() != timezone.utc.utcoffset(created):
        raise ExperimentError("manifest created_utc must identify UTC")
    for section, expected in MANIFEST_SECTION_KEYS.items():
        value = data[section]
        if not isinstance(value, dict):
            raise ExperimentError(f"manifest [{section}] must be a table")
        require_exact_keys(value, expected, f"manifest [{section}]")
    if data["source"]["git_state"] != "clean":
        raise ExperimentError("manifest source must be clean")
    if not isinstance(data["source"]["branch"], str) or data["source"]["branch"] in {
        "",
        "HEAD",
    }:
        raise ExperimentError("manifest source branch must be named")
    if not isinstance(data["source"]["sha"], str) or not GIT_OID.fullmatch(
        data["source"]["sha"]
    ):
        raise ExperimentError("manifest source SHA must be a 40- or 64-character Git OID")
    if not isinstance(data["binary"]["sha256"], str) or not HEX64.fullmatch(
        data["binary"]["sha256"]
    ):
        raise ExperimentError("manifest binary SHA-256 must be 64 hexadecimal characters")
    if data["binary"]["build_profile"] != "release":
        raise ExperimentError("manifest build profile must be release")
    if data["world"] != {
        "width": WORLD_WIDTH,
        "height": WORLD_HEIGHT,
        "chunk_size": CHUNK_SIZE,
    }:
        raise ExperimentError("manifest world must be exactly 256x256 with chunk size 64")
    if data["experiment"] != {
        "max_ticks": MAX_TICKS,
        "diagnostic_interval_ticks": DIAGNOSTIC_INTERVAL,
        "consecutive_all_sleep": CONSECUTIVE_ALL_SLEEP,
        "post_sleep_ticks": POST_SLEEP_TICKS,
    }:
        raise ExperimentError("manifest experiment constants mismatch")
    if data["renderer"] != {
        "full_width": RENDERER_WIDTH,
        "full_height": RENDERER_HEIGHT,
        "crop_x": CROP_X,
        "crop_y": CROP_Y,
        "crop_width": CROP_WIDTH,
        "crop_height": CROP_HEIGHT,
    }:
        raise ExperimentError("manifest renderer/crop contract mismatch")
    path_values = (
        data["source"]["root"],
        data["binary"]["path"],
        data["artifact"]["root"],
        data["artifact"]["run_dir"],
    )
    if not all(isinstance(value, str) and value for value in path_values):
        raise ExperimentError("manifest paths must be non-empty strings")
    source_root, binary_path, artifact_root, run_dir = map(Path, path_values)
    for label, path in (
        ("source root", source_root),
        ("binary path", binary_path),
        ("artifact root", artifact_root),
        ("run directory", run_dir),
    ):
        if not path.is_absolute():
            raise ExperimentError(f"manifest {label} must be absolute")
    if run_dir.resolve() != (artifact_root / data["run_id"]).resolve():
        raise ExperimentError("manifest run directory must be artifact_root/run_id")
    validate_external_artifact_root(source_root, artifact_root)
    expected_binary = source_root / "target" / "release" / "powdergame-windows.exe"
    if binary_path.resolve() != expected_binary.resolve():
        raise ExperimentError("manifest binary path is not the locked release Windows binary")
    expected_build = [
        "cargo",
        "build",
        "--locked",
        "--release",
        "-p",
        "powdergame-windows",
    ]
    if data["commands"]["build"] != expected_build:
        raise ExperimentError("manifest build command mismatch")
    expected_worker = list(
        worker_command(binary_path, run_dir, data["run_id"], data["binary"]["sha256"])
    )
    if data["commands"]["worker"] != expected_worker:
        raise ExperimentError("manifest worker command mismatch")
    for name in ("build", "worker"):
        command = data["commands"][name]
        if not isinstance(command, list) or not command or not all(
            isinstance(part, str) and part for part in command
        ):
            raise ExperimentError(f"manifest command {name} must be a non-empty string array")


def read_and_validate_manifest(path: Path) -> dict[str, Any]:
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise ExperimentError(f"invalid experiment manifest: {error}") from error
    validate_manifest_dict(data)
    return data


def read_json(path: Path, label: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ExperimentError(f"invalid {label}: {error}") from error
    if not isinstance(value, dict):
        raise ExperimentError(f"{label} must contain one JSON object")
    return value


def read_jsonl(path: Path, label: str) -> list[dict[str, Any]]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError) as error:
        raise ExperimentError(f"cannot read {label}: {error}") from error
    records: list[dict[str, Any]] = []
    for line_number, line in enumerate(lines, 1):
        if not line.strip():
            raise ExperimentError(f"{label} contains blank line {line_number}")
        try:
            record = json.loads(line)
        except json.JSONDecodeError as error:
            raise ExperimentError(f"{label} line {line_number} is invalid JSON: {error}") from error
        if not isinstance(record, dict):
            raise ExperimentError(f"{label} line {line_number} must be an object")
        records.append(record)
    if not records:
        raise ExperimentError(f"{label} must contain at least one record")
    return records


def require_nonnegative_int(value: Any, label: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise ExperimentError(f"{label} must be a non-negative integer")
    return value


def validate_analysis(analysis: dict[str, Any], manifest: dict[str, Any]) -> None:
    expected_keys = {
        "schema_version",
        "experiment_id",
        "run_id",
        "scenario",
        "binary_sha256",
        "provenance",
        "world",
        "sleep",
        "lifecycle",
        "baseline",
        "metrics",
        "predicates",
        "verdict",
        "raw_frame_count",
    }
    require_exact_keys(analysis, expected_keys, "analysis")
    for key in ("experiment_id", "run_id", "scenario", "binary_sha256"):
        expected = (
            manifest["binary"]["sha256"] if key == "binary_sha256" else manifest[key]
        )
        if analysis[key] != expected:
            raise ExperimentError(f"analysis {key} does not match manifest")
    if analysis["schema_version"] != ANALYSIS_SCHEMA:
        raise ExperimentError("analysis schema_version mismatch")
    provenance = analysis["provenance"]
    if not isinstance(provenance, dict):
        raise ExperimentError("analysis provenance must be an object")
    require_exact_keys(
        provenance, {"source_sha", "git_state", "build_profile"}, "analysis provenance"
    )
    if provenance["source_sha"] != manifest["source"]["sha"]:
        raise ExperimentError("analysis provenance source_sha mismatch")
    if provenance["git_state"] != "clean":
        raise ExperimentError("analysis provenance git_state must be clean")
    if provenance["build_profile"] != "release":
        raise ExperimentError("analysis provenance build_profile must be release")
    if analysis["world"] != manifest["world"]:
        raise ExperimentError("analysis world does not match manifest")
    if not isinstance(analysis["sleep"], dict):
        raise ExperimentError("analysis sleep must be an object")
    require_exact_keys(analysis["sleep"], {"enabled", "threshold"}, "analysis sleep")
    if not isinstance(analysis["sleep"]["enabled"], bool):
        raise ExperimentError("analysis sleep enabled must be boolean")
    require_nonnegative_int(analysis["sleep"]["threshold"], "analysis sleep threshold")
    lifecycle = analysis["lifecycle"]
    if not isinstance(lifecycle, dict):
        raise ExperimentError("analysis lifecycle must be an object")
    lifecycle_keys = {
        "max_ticks",
        "diagnostic_interval_ticks",
        "all_sleep_consecutive_samples",
        "post_sleep_confirmation_ticks",
        "first_all_sleep_sim_tick",
        "first_all_sleep_diagnostic_sample_tick",
        "first_all_sleep_sample_sequence",
        "confirmed_all_sleep_sim_tick",
        "post_sleep_end_tick",
        "post_sleep_change_ticks",
        "post_sleep_wake_ticks",
        "sample_count",
    }
    require_exact_keys(lifecycle, lifecycle_keys, "analysis lifecycle")
    expected_lifecycle = {
        "max_ticks": MAX_TICKS,
        "diagnostic_interval_ticks": DIAGNOSTIC_INTERVAL,
        "all_sleep_consecutive_samples": CONSECUTIVE_ALL_SLEEP,
        "post_sleep_confirmation_ticks": POST_SLEEP_TICKS,
    }
    for key, expected in expected_lifecycle.items():
        if lifecycle.get(key) != expected:
            raise ExperimentError(f"analysis lifecycle {key} mismatch")
    optional_ticks = (
        "first_all_sleep_sim_tick",
        "first_all_sleep_diagnostic_sample_tick",
        "first_all_sleep_sample_sequence",
        "confirmed_all_sleep_sim_tick",
        "post_sleep_end_tick",
    )
    for key in optional_ticks:
        if lifecycle[key] is not None:
            require_nonnegative_int(lifecycle[key], f"analysis lifecycle {key}")
    for key in ("post_sleep_change_ticks", "post_sleep_wake_ticks", "sample_count"):
        require_nonnegative_int(lifecycle[key], f"analysis lifecycle {key}")
    if (
        lifecycle["first_all_sleep_diagnostic_sample_tick"]
        != lifecycle["first_all_sleep_sample_sequence"]
    ):
        raise ExperimentError("analysis first all-sleep sample identities disagree")

    baseline = analysis["baseline"]
    if not isinstance(baseline, dict):
        raise ExperimentError("analysis baseline must be an object")
    require_exact_keys(
        baseline, {"matter_count", "sand_count", "sand_y_sum"}, "analysis baseline"
    )
    for key, value in baseline.items():
        require_nonnegative_int(value, f"analysis baseline {key}")

    metrics = analysis["metrics"]
    if not isinstance(metrics, dict):
        raise ExperimentError("analysis metrics must be an object")
    metrics_keys = {
        "peak_active_cells",
        "peak_active_chunks",
        "first_sleeping_chunk_tick",
        "first_all_sleep_tick",
        "settling_duration",
        "post_sleep_state_changes",
        "post_sleep_spontaneous_wakes",
        "final_sleeping_chunks",
        "matter_count_delta",
        "reset_exact_equivalence",
    }
    require_exact_keys(metrics, metrics_keys, "analysis metrics")
    for key in (
        "peak_active_cells",
        "peak_active_chunks",
        "post_sleep_state_changes",
        "post_sleep_spontaneous_wakes",
        "final_sleeping_chunks",
    ):
        require_nonnegative_int(metrics[key], f"analysis metrics {key}")
    for key in ("first_sleeping_chunk_tick", "first_all_sleep_tick", "settling_duration"):
        if metrics[key] is not None:
            require_nonnegative_int(metrics[key], f"analysis metrics {key}")
    if isinstance(metrics["matter_count_delta"], bool) or not isinstance(
        metrics["matter_count_delta"], int
    ):
        raise ExperimentError("analysis metrics matter_count_delta must be an integer")
    if not isinstance(metrics["reset_exact_equivalence"], bool):
        raise ExperimentError("analysis metrics reset_exact_equivalence must be boolean")
    if metrics["first_all_sleep_tick"] != lifecycle["first_all_sleep_sim_tick"]:
        raise ExperimentError("analysis first_all_sleep_tick disagrees with lifecycle")
    if metrics["settling_duration"] != lifecycle["first_all_sleep_sim_tick"]:
        raise ExperimentError("analysis settling_duration disagrees with lifecycle")
    if metrics["post_sleep_state_changes"] != lifecycle["post_sleep_change_ticks"]:
        raise ExperimentError("analysis post-sleep state change metrics disagree")
    if metrics["post_sleep_spontaneous_wakes"] != lifecycle["post_sleep_wake_ticks"]:
        raise ExperimentError("analysis post-sleep wake metrics disagree")

    predicates = analysis["predicates"]
    if not isinstance(predicates, dict) or set(predicates) != PREDICATE_NAMES:
        raise ExperimentError("analysis predicates must contain the exact seven checks")
    for name, predicate in predicates.items():
        if not isinstance(predicate, dict):
            raise ExperimentError(f"analysis predicate {name} must be an object")
        if set(predicate) != {"status", "detail"}:
            raise ExperimentError(f"analysis predicate {name} keys mismatch")
        if predicate["status"] not in PREDICATE_STATUSES:
            raise ExperimentError(f"analysis predicate {name} has invalid status")
        if not isinstance(predicate["detail"], str):
            raise ExperimentError(f"analysis predicate {name} detail must be a string")
    if analysis["verdict"] not in ALLOWED_VERDICTS:
        raise ExperimentError("analysis verdict must be PASS, FAIL, or NEEDS_HUMAN")
    raw_frame_count = require_nonnegative_int(
        analysis["raw_frame_count"], "analysis raw_frame_count"
    )
    if not 6 <= raw_frame_count <= 10:
        raise ExperimentError("analysis raw_frame_count must be between 6 and 10")


def safe_relative_worker_path(run_dir: Path, value: Any) -> Path:
    if not isinstance(value, str) or not value:
        raise ExperimentError("frame relative_path must be a non-empty string")
    if "\\" in value:
        raise ExperimentError("frame relative_path must use forward slashes")
    relative = PurePosixPath(value)
    if relative.is_absolute() or ".." in relative.parts:
        raise ExperimentError(f"unsafe frame relative_path: {value}")
    if len(relative.parts) < 3 or relative.parts[:2] != ("work", "frames"):
        raise ExperimentError("raw frames must be under work/frames")
    path = run_dir.joinpath(*relative.parts)
    if not is_path_within(path, run_dir / "work" / "frames"):
        raise ExperimentError(f"frame path escaped work/frames: {value}")
    return path


def slugify_reason(value: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return (slug or "frame")[:48]


def screenshot_name(frame: dict[str, Any], crop: bool = False) -> str:
    ordinal = require_nonnegative_int(frame.get("ordinal"), "frame ordinal")
    sim_tick = require_nonnegative_int(frame.get("sim_tick"), "frame sim_tick")
    sample_sequence = require_nonnegative_int(
        frame.get("sample_sequence"), "frame sample_sequence"
    )
    reason = frame.get("reason")
    if not isinstance(reason, str) or not reason:
        raise ExperimentError("frame reason must be a non-empty string")
    suffix = "_crop" if crop else ""
    return (
        f"frame-{ordinal:03d}_sim-{sim_tick:06d}_sample-{sample_sequence:06d}_"
        f"{slugify_reason(reason)}{suffix}.png"
    )


def validate_frames(
    frames_doc: dict[str, Any], manifest: dict[str, Any], run_dir: Path
) -> list[dict[str, Any]]:
    required_top = {
        "schema_version",
        "experiment_id",
        "run_id",
        "scenario",
        "binary_sha256",
        "frame_count",
        "pixel_encoding",
        "frames",
    }
    require_exact_keys(frames_doc, required_top, "frames.json")
    if frames_doc["schema_version"] != FRAMES_SCHEMA:
        raise ExperimentError("frames schema_version mismatch")
    for key in ("experiment_id", "run_id", "scenario", "binary_sha256"):
        expected = (
            manifest["binary"]["sha256"] if key == "binary_sha256" else manifest[key]
        )
        if frames_doc[key] != expected:
            raise ExperimentError(f"frames {key} does not match manifest")
    frames = frames_doc["frames"]
    if not isinstance(frames, list) or not frames:
        raise ExperimentError("frames.json must contain at least one frame")
    if frames_doc["frame_count"] != len(frames):
        raise ExperimentError("frames frame_count does not match frames array")
    if frames_doc["pixel_encoding"] != "rgba8-tightly-packed":
        raise ExperimentError("frames pixel_encoding mismatch")
    required_frame = {
        "ordinal",
        "kind",
        "relative_path",
        "width",
        "height",
        "rgba_bytes",
        "reason",
        "sim_tick",
        "sample_sequence",
        "state_hash",
    }
    seen_paths: set[str] = set()
    seen_names: set[str] = set()
    for expected_ordinal, frame in enumerate(frames):
        if not isinstance(frame, dict):
            raise ExperimentError(f"frame {expected_ordinal} must be an object")
        require_exact_keys(frame, required_frame, f"frame {expected_ordinal}")
        if frame["ordinal"] != expected_ordinal:
            raise ExperimentError("frame ordinals must be contiguous and zero-based")
        if not isinstance(frame["kind"], str) or not frame["kind"]:
            raise ExperimentError(f"frame {expected_ordinal} kind must be non-empty")
        if frame["width"] != RENDERER_WIDTH or frame["height"] != RENDERER_HEIGHT:
            raise ExperimentError("raw frame dimensions must be exactly 1600x900")
        expected_bytes = RENDERER_WIDTH * RENDERER_HEIGHT * 4
        if frame["rgba_bytes"] != expected_bytes:
            raise ExperimentError("raw frame rgba_bytes does not match dimensions")
        require_nonnegative_int(frame["sim_tick"], "frame sim_tick")
        require_nonnegative_int(frame["sample_sequence"], "frame sample_sequence")
        if not isinstance(frame["reason"], str) or not frame["reason"]:
            raise ExperimentError("frame reason must be non-empty")
        if not isinstance(frame["state_hash"], str) or not STATE_HASH.fullmatch(
            frame["state_hash"]
        ):
            raise ExperimentError("frame state_hash must be fnv1a64:<16 lowercase hex>")
        raw_path = safe_relative_worker_path(run_dir, frame["relative_path"])
        if frame["relative_path"] in seen_paths:
            raise ExperimentError("duplicate raw frame relative_path")
        seen_paths.add(frame["relative_path"])
        if not raw_path.is_file() or raw_path.stat().st_size != expected_bytes:
            raise ExperimentError(f"raw RGBA size mismatch: {raw_path}")
        name = screenshot_name(frame)
        if name in seen_names:
            raise ExperimentError("duplicate derived screenshot name")
        seen_names.add(name)
    return frames


def validate_samples(samples: list[dict[str, Any]], manifest: dict[str, Any]) -> None:
    expected_keys = {
        "schema_version",
        "experiment_id",
        "run_id",
        "source_sha",
        "git_state",
        "build_profile",
        "binary_sha256",
        "sample_sequence",
        "sim_tick",
        "phase",
        "reason",
        "world",
        "sleep",
        "census",
        "material_counts_by_id",
        "matter_count",
        "sand_count",
        "sand_y_sum",
        "sand_min_y",
        "sand_max_y",
        "invalid_material_count",
        "nonfinite_temperature_count",
        "nonfinite_pressure_count",
        "changed_chunks",
        "wake_chunks",
        "wake_reason_or",
        "state_hash",
    }
    census_keys = {
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
    }
    for index, sample in enumerate(samples):
        require_exact_keys(sample, expected_keys, f"sample {index}")
        if sample["schema_version"] != TELEMETRY_SCHEMA:
            raise ExperimentError(f"sample {index} schema_version mismatch")
        identity = {
            "experiment_id": manifest["experiment_id"],
            "run_id": manifest["run_id"],
            "source_sha": manifest["source"]["sha"],
            "git_state": "clean",
            "build_profile": "release",
            "binary_sha256": manifest["binary"]["sha256"],
        }
        for key, expected in identity.items():
            if sample[key] != expected:
                raise ExperimentError(f"sample {index} {key} mismatch")
        sequence = require_nonnegative_int(sample["sample_sequence"], "sample sequence")
        if sequence != index:
            raise ExperimentError("sample_sequence must be contiguous and zero-based")
        require_nonnegative_int(sample["sim_tick"], "sample sim_tick")
        if not isinstance(sample["phase"], str) or not sample["phase"]:
            raise ExperimentError(f"sample {index} phase must be non-empty")
        if not isinstance(sample["reason"], str) or not sample["reason"]:
            raise ExperimentError(f"sample {index} reason must be non-empty")
        if sample["world"] != manifest["world"]:
            raise ExperimentError(f"sample {index} world mismatch")
        sleep = sample["sleep"]
        if not isinstance(sleep, dict):
            raise ExperimentError(f"sample {index} sleep must be an object")
        require_exact_keys(sleep, {"enabled", "threshold"}, f"sample {index} sleep")
        if not isinstance(sleep["enabled"], bool):
            raise ExperimentError(f"sample {index} sleep enabled must be boolean")
        require_nonnegative_int(sleep["threshold"], f"sample {index} sleep threshold")
        census = sample["census"]
        if not isinstance(census, dict):
            raise ExperimentError(f"sample {index} census must be an object")
        require_exact_keys(census, census_keys, f"sample {index} census")
        for key, value in census.items():
            require_nonnegative_int(value, f"sample {index} census {key}")
        if census["total_cells"] != WORLD_WIDTH * WORLD_HEIGHT:
            raise ExperimentError(f"sample {index} census total_cells mismatch")
        total_chunks = (WORLD_WIDTH // CHUNK_SIZE) * (WORLD_HEIGHT // CHUNK_SIZE)
        if census["total_chunks"] != total_chunks:
            raise ExperimentError(f"sample {index} census total_chunks mismatch")
        counts = sample["material_counts_by_id"]
        if not isinstance(counts, list) or len(counts) != 10:
            raise ExperimentError(f"sample {index} material_counts_by_id mismatch")
        for material_id, value in enumerate(counts):
            require_nonnegative_int(value, f"sample {index} material count {material_id}")
        for key in (
            "matter_count",
            "sand_count",
            "sand_y_sum",
            "invalid_material_count",
            "nonfinite_temperature_count",
            "nonfinite_pressure_count",
            "changed_chunks",
            "wake_chunks",
            "wake_reason_or",
        ):
            require_nonnegative_int(sample[key], f"sample {index} {key}")
        for key in ("sand_min_y", "sand_max_y"):
            if sample[key] is not None:
                require_nonnegative_int(sample[key], f"sample {index} {key}")
        if not isinstance(sample["state_hash"], str) or not STATE_HASH.fullmatch(
            sample["state_hash"]
        ):
            raise ExperimentError(f"sample {index} state_hash is invalid")


def validate_events(events: list[dict[str, Any]], manifest: dict[str, Any]) -> None:
    for index, event in enumerate(events):
        required = {
            "schema_version",
            "experiment_id",
            "run_id",
            "event_sequence",
            "event",
            "sim_tick",
            "sample_sequence",
            "detail",
        }
        require_exact_keys(event, required, f"event {index}")
        if event["schema_version"] != TELEMETRY_SCHEMA:
            raise ExperimentError(f"event {index} schema_version mismatch")
        if event["experiment_id"] != manifest["experiment_id"]:
            raise ExperimentError(f"event {index} experiment_id mismatch")
        if event["run_id"] != manifest["run_id"]:
            raise ExperimentError(f"event {index} run_id mismatch")
        sequence = require_nonnegative_int(event["event_sequence"], "event sequence")
        if sequence != index:
            raise ExperimentError("event_sequence must be contiguous and zero-based")
        require_nonnegative_int(event["sim_tick"], "event sim_tick")
        if event["sample_sequence"] is not None:
            require_nonnegative_int(event["sample_sequence"], "event sample_sequence")
        if not isinstance(event["event"], str) or not event["event"]:
            raise ExperimentError(f"event {index} event must be non-empty")
        if not isinstance(event["detail"], str):
            raise ExperimentError(f"event {index} detail must be a string")


def sample_is_all_sleep(sample: dict[str, Any]) -> bool:
    census = sample["census"]
    return (
        census["any_active_cells"] == 0
        and census["active_chunks"] == 0
        and census["runnable_chunks"] == 0
        and census["total_chunks"] > 0
        and census["sleeping_chunks"] == census["total_chunks"]
    )


def verdict_from_predicates(predicates: dict[str, Any]) -> str:
    statuses = {predicate["status"] for predicate in predicates.values()}
    if "fail" in statuses:
        return "FAIL"
    if "unknown" in statuses:
        return "NEEDS_HUMAN"
    return "PASS"


def first_confirmed_all_sleep_streak(
    samples: list[dict[str, Any]], required: int
) -> list[dict[str, Any]] | None:
    streak: list[dict[str, Any]] = []
    for sample in samples:
        if sample["phase"] != "settling" or sample["reason"] == "tick1":
            continue
        if sample_is_all_sleep(sample):
            streak.append(sample)
            if len(streak) == required:
                return streak
        else:
            streak.clear()
    return None


def validate_telemetry(
    run_dir: Path, manifest: dict[str, Any]
) -> tuple[dict[str, Any], dict[str, Any], list[dict[str, Any]], list[dict[str, Any]]]:
    required_logs = (
        run_dir / "stdout.log",
        run_dir / "stderr.log",
        run_dir / "logs" / "build.stdout.log",
        run_dir / "logs" / "build.stderr.log",
    )
    for path in required_logs:
        if not path.is_file():
            raise ExperimentError(f"required raw command log is missing: {path}")
    analysis = read_json(run_dir / "work" / "analysis.json", "analysis.json")
    frames_doc = read_json(run_dir / "work" / "frames.json", "frames.json")
    samples = read_jsonl(run_dir / "telemetry" / "samples.jsonl", "samples.jsonl")
    events = read_jsonl(run_dir / "telemetry" / "events.jsonl", "events.jsonl")
    validate_analysis(analysis, manifest)
    frames = validate_frames(frames_doc, manifest, run_dir)
    validate_samples(samples, manifest)
    validate_events(events, manifest)
    if analysis["raw_frame_count"] != len(frames):
        raise ExperimentError("analysis raw_frame_count does not match frames.json")
    sample_count = analysis["lifecycle"].get("sample_count")
    if sample_count != len(samples):
        raise ExperimentError("analysis lifecycle sample_count does not match samples.jsonl")
    recomputed_verdict = verdict_from_predicates(analysis["predicates"])
    if analysis["verdict"] != recomputed_verdict:
        raise ExperimentError("analysis verdict disagrees with its seven predicate statuses")
    for index, sample in enumerate(samples):
        if sample["sleep"] != analysis["sleep"]:
            raise ExperimentError(f"sample {index} sleep settings disagree with analysis")
    baseline = analysis["baseline"]
    tick0 = samples[0]
    expected_baseline = {
        "matter_count": tick0["matter_count"],
        "sand_count": tick0["sand_count"],
        "sand_y_sum": tick0["sand_y_sum"],
    }
    if baseline != expected_baseline:
        raise ExperimentError("analysis baseline does not match sample 0")
    metrics = analysis["metrics"]
    if metrics["peak_active_cells"] != max(
        sample["census"]["any_active_cells"] for sample in samples
    ):
        raise ExperimentError("analysis peak_active_cells does not match telemetry")
    if metrics["peak_active_chunks"] != max(
        sample["census"]["active_chunks"] for sample in samples
    ):
        raise ExperimentError("analysis peak_active_chunks does not match telemetry")
    first_sleeping = next(
        (sample for sample in samples if sample["census"]["sleeping_chunks"] > 0), None
    )
    first_sleeping_tick = None if first_sleeping is None else first_sleeping["sim_tick"]
    if metrics["first_sleeping_chunk_tick"] != first_sleeping_tick:
        raise ExperimentError("analysis first_sleeping_chunk_tick does not match telemetry")
    non_reset_samples = [sample for sample in samples if sample["phase"] != "reset"]
    if not non_reset_samples:
        raise ExperimentError("telemetry has no pre-reset samples")
    final_pre_reset = non_reset_samples[-1]
    if metrics["final_sleeping_chunks"] != final_pre_reset["census"]["sleeping_chunks"]:
        raise ExperimentError("analysis final_sleeping_chunks does not match telemetry")
    matter_delta = final_pre_reset["matter_count"] - tick0["matter_count"]
    if metrics["matter_count_delta"] != matter_delta:
        raise ExperimentError("analysis matter_count_delta does not match telemetry")

    lifecycle = analysis["lifecycle"]
    streak = first_confirmed_all_sleep_streak(samples, CONSECUTIVE_ALL_SLEEP)
    post_sleep = [
        sample for sample in samples if sample["phase"] == "post-sleep-confirmation"
    ]
    if streak is None:
        for key in (
            "first_all_sleep_sim_tick",
            "first_all_sleep_diagnostic_sample_tick",
            "first_all_sleep_sample_sequence",
            "confirmed_all_sleep_sim_tick",
            "post_sleep_end_tick",
        ):
            if lifecycle[key] is not None:
                raise ExperimentError(f"analysis {key} exists without a confirmed all-sleep streak")
        if post_sleep:
            raise ExperimentError("post-sleep samples exist without a confirmed all-sleep streak")
    else:
        first = streak[0]
        confirmed = streak[-1]
        expected_identity = {
            "first_all_sleep_sim_tick": first["sim_tick"],
            "first_all_sleep_diagnostic_sample_tick": first["sample_sequence"],
            "first_all_sleep_sample_sequence": first["sample_sequence"],
            "confirmed_all_sleep_sim_tick": confirmed["sim_tick"],
        }
        for key, expected in expected_identity.items():
            if lifecycle[key] != expected:
                raise ExperimentError(f"analysis lifecycle {key} does not match telemetry")
        if len(post_sleep) != POST_SLEEP_TICKS:
            raise ExperimentError("post-sleep telemetry does not contain exactly 180 samples")
        expected_ticks = list(
            range(confirmed["sim_tick"] + 1, confirmed["sim_tick"] + POST_SLEEP_TICKS + 1)
        )
        if [sample["sim_tick"] for sample in post_sleep] != expected_ticks:
            raise ExperimentError("post-sleep telemetry sim ticks are not contiguous")
        if lifecycle["post_sleep_end_tick"] != expected_ticks[-1]:
            raise ExperimentError("analysis post_sleep_end_tick does not match telemetry")
        if analysis["predicates"]["post_sleep_stable"]["status"] == "pass":
            stable_hash = confirmed["state_hash"]
            for sample in post_sleep:
                census = sample["census"]
                if (
                    sample["state_hash"] != stable_hash
                    or sample["changed_chunks"] != 0
                    or sample["wake_chunks"] != 0
                    or census["any_active_cells"] != 0
                    or census["active_chunks"] != 0
                    or census["runnable_chunks"] != 0
                    or census["sleeping_chunks"] != census["total_chunks"]
                ):
                    raise ExperimentError(
                        "PASS post-sleep telemetry contains a state change or wake"
                    )
            if (
                lifecycle["post_sleep_change_ticks"] != 0
                or lifecycle["post_sleep_wake_ticks"] != 0
            ):
                raise ExperimentError("PASS post-sleep lifecycle counts must both be zero")
    reset_samples = [sample for sample in samples if sample["phase"] == "reset"]
    if len(reset_samples) != 1 or reset_samples[0] is not samples[-1]:
        raise ExperimentError("telemetry must end with exactly one reset sample")
    if (
        analysis["predicates"]["exact_reset"]["status"] == "pass"
        and reset_samples[0]["state_hash"] != tick0["state_hash"]
    ):
        raise ExperimentError("PASS exact_reset has a different authoritative state hash")
    by_sequence = {sample["sample_sequence"]: sample for sample in samples}
    for frame in frames:
        sample = by_sequence.get(frame["sample_sequence"])
        if sample is None:
            raise ExperimentError("frame sample_sequence is absent from telemetry")
        if sample["sim_tick"] != frame["sim_tick"]:
            raise ExperimentError("frame sim_tick disagrees with its telemetry sample")
        if sample["state_hash"] != frame["state_hash"]:
            raise ExperimentError("frame state_hash disagrees with its telemetry sample")
    return analysis, frames_doc, samples, events


def pillow_modules() -> tuple[Any, Any, Any]:
    try:
        from PIL import Image, ImageDraw, ImageOps
    except ImportError as error:
        raise ExperimentError("Pillow is required to post-process experiment frames") from error
    return Image, ImageDraw, ImageOps


def png_bytes(image: Any) -> bytes:
    buffer = io.BytesIO()
    image.save(buffer, format="PNG", optimize=False, compress_level=9)
    return buffer.getvalue()


def create_screenshots(
    run_dir: Path, frames: list[dict[str, Any]], publication_log: list[str]
) -> list[dict[str, Any]]:
    Image, _, _ = pillow_modules()
    screenshots = run_dir / "screenshots"
    full_dir = screenshots / "full"
    crop_dir = screenshots / "crops"
    try:
        screenshots.mkdir()
        full_dir.mkdir()
        crop_dir.mkdir()
    except FileExistsError as error:
        raise ExperimentError("screenshot output directory already exists") from error

    output: list[dict[str, Any]] = []
    for frame in frames:
        raw_path = safe_relative_worker_path(run_dir, frame["relative_path"])
        raw = raw_path.read_bytes()
        image = Image.frombytes("RGBA", (RENDERER_WIDTH, RENDERER_HEIGHT), raw)
        full_name = screenshot_name(frame)
        crop_name = screenshot_name(frame, crop=True)
        full_path = full_dir / full_name
        crop_path = crop_dir / crop_name
        write_new_bytes(full_path, png_bytes(image), publication_log)
        crop = image.crop(
            (CROP_X, CROP_Y, CROP_X + CROP_WIDTH, CROP_Y + CROP_HEIGHT)
        )
        write_new_bytes(crop_path, png_bytes(crop), publication_log)
        output.append(
            {
                "ordinal": frame["ordinal"],
                "reason": frame["reason"],
                "sim_tick": frame["sim_tick"],
                "sample_sequence": frame["sample_sequence"],
                "state_hash": frame["state_hash"],
                "full_png": full_path.relative_to(run_dir).as_posix(),
                "crop_png": crop_path.relative_to(run_dir).as_posix(),
            }
        )
    return output


def create_contact_sheet_bytes(run_dir: Path, screenshots: list[dict[str, Any]]) -> bytes:
    Image, ImageDraw, ImageOps = pillow_modules()
    columns = 3
    panel_width = 420
    panel_height = 450
    rows = (len(screenshots) + columns - 1) // columns
    sheet = Image.new("RGB", (columns * panel_width, max(1, rows) * panel_height), "#11151c")
    draw = ImageDraw.Draw(sheet)
    for index, item in enumerate(screenshots):
        column = index % columns
        row = index // columns
        left = column * panel_width
        top = row * panel_height
        crop = Image.open(run_dir / item["crop_png"]).convert("RGB")
        thumb = ImageOps.contain(crop, (390, 390))
        x = left + (panel_width - thumb.width) // 2
        y = top + 8
        sheet.paste(thumb, (x, y))
        label = (
            f"#{item['ordinal']} {item['reason']} | sim {item['sim_tick']} | "
            f"sample {item['sample_sequence']}"
        )
        draw.text((left + 12, top + 410), label, fill="#f4f7fb")
        draw.rectangle(
            (left + 2, top + 2, left + panel_width - 3, top + panel_height - 3),
            outline="#506078",
            width=2,
        )
    return png_bytes(sheet)


def render_report_markdown(
    manifest: dict[str, Any],
    analysis: dict[str, Any],
    samples: list[dict[str, Any]],
    events: list[dict[str, Any]],
    screenshots: list[dict[str, Any]],
) -> str:
    lines = [
        "# Powdergame Sand Fall Experiment Report",
        "",
        f"- Experiment: `{manifest['experiment_id']}`",
        f"- Run ID: `{manifest['run_id']}`",
        f"- Source: `{manifest['source']['sha']}` on `{manifest['source']['branch']}` (`clean`)",
        f"- Binary SHA-256: `{manifest['binary']['sha256']}`",
        f"- Automatic verdict: **{analysis['verdict']}**",
        f"- Samples / events / frames: {len(samples)} / {len(events)} / {len(screenshots)}",
        "",
        "The automatic verdict is worker telemetry, not user acceptance or G8-B/G8-C closure.",
        "",
        "## Predicates",
        "",
        "| Predicate | Status | Detail |",
        "|---|---|---|",
    ]
    for name in sorted(PREDICATE_NAMES):
        predicate = analysis["predicates"][name]
        detail = predicate["detail"].replace("|", "\\|").replace("\n", " ")
        lines.append(f"| `{name}` | {predicate['status']} | {detail} |")
    lines.extend(
        [
            "",
            "## Frames",
            "",
            "| # | Reason | Sim tick | Sample | State hash | Full | World crop |",
            "|---:|---|---:|---:|---|---|---|",
        ]
    )
    for item in screenshots:
        lines.append(
            f"| {item['ordinal']} | {item['reason']} | {item['sim_tick']} | "
            f"{item['sample_sequence']} | `{item['state_hash']}` | "
            f"[{Path(item['full_png']).name}](../{item['full_png']}) | "
            f"[{Path(item['crop_png']).name}](../{item['crop_png']}) |"
        )
    lines.extend(
        [
            "",
            "## Boundaries",
            "",
            "- This experiment is Sand Fall only.",
            "- Water Flow and G8-C are outside scope.",
            "- Gallery rendering/diagnostics are not official benchmark timing.",
            "- Review packet generation does not contact an AI reviewer.",
            "",
        ]
    )
    return "\n".join(lines)


def render_review_prompt(manifest: dict[str, Any], analysis: dict[str, Any]) -> str:
    return f"""# ChatGPT Review Prompt — Powdergame Sand Fall Experiment

Review only the attached `REVIEW_PACKET.zip` for experiment `{manifest['experiment_id']}`,
run `{manifest['run_id']}`, source `{manifest['source']['sha']}`, binary
`{manifest['binary']['sha256']}`. The worker automatic verdict is
`{analysis['verdict']}`; treat it as a claim to check, not as a conclusion to repeat.

Inspect the manifest, raw logs, telemetry JSONL, REPORT.md/REPORT.json, full screenshots,
world crops, and contact sheet. Report concrete evidence, mismatches, missing data, and
unresolved questions. Do not infer Water Flow, G8-C performance, product readiness, or
G8-B closure from this Sand Fall run. Complete settling and all chunks sleeping are an
accepted successful Sand Fall outcome; do not recommend artificial perpetual activity
merely to keep the scene moving.

No action, code change, upload, or external message is authorized by this prompt.
"""


def packet_members(run_dir: Path) -> list[Path]:
    include_roots = {"logs", "telemetry", "report", "screenshots"}
    include_files = {"EXPERIMENT_MANIFEST.toml", "stdout.log", "stderr.log"}
    members: list[Path] = []
    for path in run_dir.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(run_dir)
        if relative.as_posix() == "report/REVIEW_PACKET.zip":
            continue
        if relative.as_posix() in include_files or relative.parts[0] in include_roots:
            members.append(path)
    return sorted(members, key=lambda path: path.relative_to(run_dir).as_posix())


def create_review_packet(run_dir: Path) -> Path:
    packet = run_dir / "report" / "REVIEW_PACKET.zip"
    try:
        with zipfile.ZipFile(
            packet, mode="x", compression=zipfile.ZIP_DEFLATED, compresslevel=9
        ) as archive:
            for path in packet_members(run_dir):
                archive.write(path, path.relative_to(run_dir).as_posix())
    except FileExistsError as error:
        raise ExperimentError(f"refusing to overwrite existing packet: {packet}") from error
    return packet


def hashable_files(run_dir: Path) -> list[Path]:
    excluded = {"HASHES.sha256", "EXPERIMENT_RECEIPT.json"}
    return sorted(
        (
            path
            for path in run_dir.rglob("*")
            if path.is_file() and path.name not in excluded
        ),
        key=lambda path: path.relative_to(run_dir).as_posix(),
    )


def render_hashes(run_dir: Path) -> str:
    return "".join(
        f"{sha256_file(path)}  {path.relative_to(run_dir).as_posix()}\n"
        for path in hashable_files(run_dir)
    )


def postprocess_run(run_dir: Path, publication_log: list[str] | None = None) -> Path:
    log = publication_log if publication_log is not None else []
    receipt_path = run_dir / "EXPERIMENT_RECEIPT.json"
    if receipt_path.exists():
        raise ExperimentError("completed run already has a receipt and cannot be reused")
    manifest_path = run_dir / "EXPERIMENT_MANIFEST.toml"
    manifest = read_and_validate_manifest(manifest_path)
    analysis, frames_doc, samples, events = validate_telemetry(run_dir, manifest)
    screenshots = create_screenshots(run_dir, frames_doc["frames"], log)

    report_dir = run_dir / "report"
    try:
        report_dir.mkdir()
    except FileExistsError as error:
        raise ExperimentError("report output directory already exists") from error
    contact_sheet_path = report_dir / "CONTACT_SHEET.png"
    write_new_bytes(
        contact_sheet_path, create_contact_sheet_bytes(run_dir, screenshots), log
    )

    report_json = {
        "schema_version": REPORT_SCHEMA,
        "experiment_id": manifest["experiment_id"],
        "run_id": manifest["run_id"],
        "scenario": manifest["scenario"],
        "source": manifest["source"],
        "binary": manifest["binary"],
        "automatic_verdict": analysis["verdict"],
        "predicates": analysis["predicates"],
        "analysis": analysis,
        "sample_count": len(samples),
        "event_count": len(events),
        "screenshots": screenshots,
        "contact_sheet": contact_sheet_path.relative_to(run_dir).as_posix(),
        "scope": {
            "sand_fall_only": True,
            "water_flow": False,
            "g8c": False,
            "ai_contacted": False,
        },
    }
    write_new_text(
        report_dir / "REPORT.json",
        json.dumps(report_json, indent=2, ensure_ascii=False, sort_keys=True) + "\n",
        log,
    )
    write_new_text(
        report_dir / "REPORT.md",
        render_report_markdown(manifest, analysis, samples, events, screenshots),
        log,
    )
    write_new_text(
        report_dir / "CHATGPT_REVIEW_PROMPT.md",
        render_review_prompt(manifest, analysis),
        log,
    )

    packet = create_review_packet(run_dir)
    log.append(packet.relative_to(run_dir).as_posix())
    hashes_path = run_dir / "HASHES.sha256"
    write_new_text(hashes_path, render_hashes(run_dir), log)

    receipt = {
        "schema_version": RECEIPT_SCHEMA,
        "experiment_id": manifest["experiment_id"],
        "run_id": manifest["run_id"],
        "scenario": manifest["scenario"],
        "source_sha": manifest["source"]["sha"],
        "binary_sha256": manifest["binary"]["sha256"],
        "automatic_verdict": analysis["verdict"],
        "completed_utc": format_utc(utc_now()),
        "manifest_sha256": sha256_file(manifest_path),
        "review_packet_sha256": sha256_file(packet),
        "hashes_sha256": sha256_file(hashes_path),
        "hash_entry_count": len(hashable_files(run_dir)),
        "receipt_is_final_publication_marker": True,
    }
    write_new_text(
        receipt_path,
        json.dumps(receipt, indent=2, ensure_ascii=False, sort_keys=True) + "\n",
        log,
    )
    # Publication invariant: this function performs no filesystem write after
    # the create-new receipt write above.
    return receipt_path


def worker_command(binary: Path, run_dir: Path, run_id: str, binary_sha256: str) -> tuple[str, ...]:
    return (
        str(binary),
        "--experiment-worker",
        SCENARIO,
        "--experiment-run-dir",
        str(run_dir),
        "--experiment-run-id",
        run_id,
        "--binary-sha256",
        binary_sha256,
        "--max-ticks",
        str(MAX_TICKS),
        "--diagnostic-interval",
        str(DIAGNOSTIC_INTERVAL),
        "--consecutive-all-sleep",
        str(CONSECUTIVE_ALL_SLEEP),
        "--post-sleep-ticks",
        str(POST_SLEEP_TICKS),
    )


def run_experiment(source_root: Path, artifact_root: Path, scenario: str) -> Path:
    if scenario != SCENARIO:
        raise ExperimentError(
            f"unsupported experiment scenario {scenario!r}; only {SCENARIO!r} is allowed"
        )
    validate_external_artifact_root(source_root, artifact_root)
    source = inspect_clean_named_source(source_root)
    run_id = generate_run_id()
    run_dir = create_run_directory(artifact_root.resolve(), run_id)
    logs = run_dir / "logs"
    logs.mkdir()

    build = ("cargo", "build", "--locked", "--release", "-p", "powdergame-windows")
    build_exit = run_logged(
        build,
        source.root,
        logs / "build.stdout.log",
        logs / "build.stderr.log",
    )
    if build_exit != 0:
        raise ExperimentError(f"release build failed with exit code {build_exit}; run preserved")

    binary = source.root / "target" / "release" / "powdergame-windows.exe"
    if not binary.is_file():
        raise ExperimentError(f"release binary was not produced: {binary}")
    binary_hash = sha256_file(binary)
    worker = worker_command(binary, run_dir, run_id, binary_hash)
    manifest = ManifestData(
        run_id=run_id,
        created_utc=format_utc(utc_now()),
        source=source,
        binary_path=binary,
        binary_sha256=binary_hash,
        artifact_root=artifact_root.resolve(),
        run_dir=run_dir.resolve(),
        build_command=build,
        worker_command=worker,
    )
    manifest_text = render_manifest(manifest)
    write_new_text(run_dir / "EXPERIMENT_MANIFEST.toml", manifest_text)
    read_and_validate_manifest(run_dir / "EXPERIMENT_MANIFEST.toml")

    worker_exit = run_logged(
        worker,
        source.root,
        run_dir / "stdout.log",
        run_dir / "stderr.log",
    )
    if worker_exit != 0:
        raise ExperimentError(
            f"experiment worker failed operationally with exit code {worker_exit}; "
            "run preserved without receipt"
        )
    return postprocess_run(run_dir)


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run one immutable Powdergame Sand Fall experiment."
    )
    parser.add_argument("scenario", help="must be exactly sand-fall")
    parser.add_argument(
        "--artifact-root",
        type=Path,
        default=DEFAULT_ARTIFACT_ROOT,
        help=f"external artifact root (default: {DEFAULT_ARTIFACT_ROOT})",
    )
    parser.add_argument(
        "--source-root",
        type=Path,
        default=Path(__file__).resolve().parents[2],
        help=argparse.SUPPRESS,
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        receipt = run_experiment(args.source_root, args.artifact_root, args.scenario)
    except ExperimentError as error:
        print(f"FATAL: {error}", file=sys.stderr)
        return 1
    print(f"Experiment receipt: {receipt}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
