#!/usr/bin/env python3
"""Capture one immutable G8-C performance matrix from raw worker evidence.

The historical G8-A benchmark files remain the inner Mode A/B contract.  This
coordinator adds G8-C wrapper identity, the two windowed workers, source and
binary binding, raw-first aggregation, and receipt-last publication.
"""

from __future__ import annotations

import argparse
import csv
import datetime as dt
import hashlib
import json
import math
import os
from pathlib import Path, PurePosixPath
import runpy
import shutil
import statistics
import subprocess
import sys
import tempfile
from typing import Any, Mapping, Sequence
import zipfile


MATRIX_SCHEMA = "powdergame-g8c-official-matrix-v1"
HEADLESS_SCHEMA = "powdergame-g8c-headless-v1"
INNER_HEADLESS_SCHEMA = "powdergame-g8b-fixture-v1"
COEXISTENCE_SCHEMA = "powdergame-g8c-coexistence-v1"
RENDER_PROFILE_SCHEMA = "powdergame-g8c-render-profile-v1"
SOURCE_SCHEMA = "powdergame-g8c-source-input-v1"
PROCESS_SCHEMA = "powdergame-g8c-process-v1"
RECEIPT_SCHEMA = "powdergame-g8c-matrix-receipt-v1"
REPORT_SCHEMA = "powdergame-g8c-matrix-report-v1"
REPLAY_INPUT_SCHEMA = "powdergame-g8c-aggregation-replay-input-v1"
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
GROUP_FIELDS = (
    "group_matter_movement_ms",
    "group_ownership_claim_ms",
    "group_thermal_conduction_ms",
    "group_reaction_phase_ms",
    "group_pressure_structure_ms",
    "group_active_sleep_management_ms",
)
GROUP_LABELS = {
    "group_matter_movement_ms": "Matter Movement",
    "group_ownership_claim_ms": "Claim / Resolve",
    "group_thermal_conduction_ms": "Thermal",
    "group_reaction_phase_ms": "Reaction / Phase",
    "group_pressure_structure_ms": "Pressure / Structure",
    "group_active_sleep_management_ms": "Active / Sleep",
}
HEADLESS_SUMMARY_HEADER = (
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
FONT_PATH = Path(r"C:\Windows\Fonts\consola.ttf")
FIXED_ZIP_TIME = (1980, 1, 1, 0, 0, 0)


class MatrixError(RuntimeError):
    """An operational or evidence-integrity failure."""


def utc_now() -> str:
    return (
        dt.datetime.now(dt.timezone.utc)
        .isoformat(timespec="microseconds")
        .replace("+00:00", "Z")
    )


def canonical_json_bytes(value: Any) -> bytes:
    return (
        json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    ).encode("utf-8")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def write_new_bytes(path: Path, value: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        with path.open("xb") as stream:
            stream.write(value)
            stream.flush()
            os.fsync(stream.fileno())
    except FileExistsError as error:
        raise MatrixError(f"refusing to overwrite existing output: {path}") from error


def write_new_json(path: Path, value: Any) -> None:
    write_new_bytes(path, canonical_json_bytes(value))


def write_new_text(path: Path, value: str) -> None:
    write_new_bytes(path, value.encode("utf-8"))


def safe_relative(path: Path, root: Path) -> str:
    try:
        relative = path.resolve().relative_to(root.resolve())
    except ValueError as error:
        raise MatrixError(f"path escapes matrix root: {path}") from error
    result = relative.as_posix()
    if result in ("", ".") or ".." in PurePosixPath(result).parts:
        raise MatrixError(f"unsafe matrix-relative path: {result}")
    return result


def _git(
    source_root: Path, *args: str, check: bool = True
) -> subprocess.CompletedProcess[bytes]:
    command = [
        "git",
        "-c",
        f"safe.directory={source_root.resolve().as_posix()}",
        *args,
    ]
    result = subprocess.run(command, cwd=source_root, capture_output=True, check=False)
    if check and result.returncode != 0:
        message = result.stderr.decode("utf-8", errors="replace").strip()
        raise MatrixError(f"git command failed ({' '.join(args)}): {message}")
    return result


def _git_text(source_root: Path, *args: str) -> str:
    return _git(source_root, *args).stdout.decode("utf-8", errors="strict").strip()


def _status_records(source_root: Path) -> tuple[str, ...]:
    raw = _git(
        source_root, "status", "--porcelain=v1", "-z", "--untracked-files=all"
    ).stdout
    return tuple(
        part.decode("utf-8", errors="surrogateescape")
        for part in raw.split(b"\0")
        if part
    )


def _tracked_paths(source_root: Path) -> tuple[str, ...]:
    raw = _git(source_root, "ls-files", "-z").stdout
    paths = tuple(
        part.decode("utf-8", errors="surrogateescape")
        for part in raw.split(b"\0")
        if part
    )
    if not paths or len(set(paths)) != len(paths):
        raise MatrixError(
            "tracked source inventory is empty or contains duplicate paths"
        )
    return tuple(sorted(paths))


def validate_scenario_sequence(scenarios: Sequence[str]) -> None:
    if tuple(scenarios) != SCENARIOS:
        missing = sorted(set(SCENARIOS) - set(scenarios))
        duplicates = sorted({item for item in scenarios if scenarios.count(item) > 1})
        extras = sorted(set(scenarios) - set(SCENARIOS))
        raise MatrixError(
            "G8-C requires each official scenario exactly once in canonical order; "
            f"missing={missing}, duplicates={duplicates}, extras={extras}"
        )


def matrix_profile(mode: str) -> dict[str, Any]:
    if mode == "pilot":
        return {
            "width": 256,
            "height": 256,
            "chunk_size": 64,
            "sleep_enabled": True,
            "sleep_threshold": 16,
            "prewarm_secs": 2.0,
            "trials": 1,
            "mode_a_ticks": 32,
            "mode_b_ticks": 16,
            "overhead_ticks": 16,
            "mode_c_measurement_secs": None,
            "mode_c_measurement_frames": 60,
            "mode_d_profile_frames": 16,
            "target_tps": 60,
            "render_width": 1600,
            "render_height": 900,
            "present_mode": "Fifo",
        }
    if mode == "official":
        return {
            "width": 2048,
            "height": 2048,
            "chunk_size": 64,
            "sleep_enabled": True,
            "sleep_threshold": 16,
            "prewarm_secs": 2.0,
            "trials": 3,
            "mode_a_ticks": 1024,
            "mode_b_ticks": 256,
            "overhead_ticks": 256,
            "mode_c_measurement_secs": 10.0,
            "mode_c_measurement_frames": None,
            "mode_d_profile_frames": 256,
            "target_tps": 60,
            "render_width": 1600,
            "render_height": 900,
            "present_mode": "Fifo",
        }
    raise MatrixError(f"unsupported matrix mode: {mode}")


def capture_source_state(source_root: Path, mode: str) -> dict[str, Any]:
    source_root = source_root.resolve()
    if not (source_root / ".git").exists():
        raise MatrixError(f"source root is not a Git worktree: {source_root}")
    sha = _git_text(source_root, "rev-parse", "HEAD")
    branch = _git_text(source_root, "branch", "--show-current")
    if branch != REQUIRED_BRANCH:
        raise MatrixError(
            f"G8-C capture requires branch {REQUIRED_BRANCH}, observed {branch or '(detached)'}"
        )
    records = _status_records(source_root)
    untracked = sorted(record[3:] for record in records if record.startswith("?? "))
    if untracked:
        raise MatrixError(f"untracked source inputs are forbidden: {untracked}")
    if mode == "official":
        if records:
            raise MatrixError(
                f"official capture requires a clean worktree: {list(records)}"
            )
        upstream = _git_text(
            source_root,
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        )
        expected_upstream = f"origin/{REQUIRED_BRANCH}"
        if upstream != expected_upstream:
            raise MatrixError(
                "official capture requires the exact origin feature-branch upstream; "
                f"observed {upstream!r}, expected {expected_upstream!r}"
            )
        upstream_sha = _git_text(source_root, "rev-parse", "@{upstream}")
        counts = _git_text(
            source_root, "rev-list", "--left-right", "--count", "HEAD...@{upstream}"
        ).split()
        if upstream_sha != sha or counts != ["0", "0"]:
            raise MatrixError(
                "official capture requires HEAD equal to upstream with ahead/behind 0/0; "
                f"HEAD={sha}, upstream={upstream_sha}, counts={counts}"
            )
    else:
        upstream = None
        upstream_sha = None
        counts = None
    return {
        "sha": sha,
        "branch": branch,
        "git_state": "clean" if not records else "dirty",
        "dirty_scope": None if not records else "tracked-only",
        "status_porcelain": list(records),
        "upstream": upstream,
        "upstream_sha": upstream_sha,
        "ahead_behind": counts,
    }


def collect_source_entries(source_root: Path) -> tuple[list[dict[str, Any]], str]:
    entries: list[dict[str, Any]] = []
    for relative in _tracked_paths(source_root):
        path = source_root / Path(relative)
        if not path.is_file():
            raise MatrixError(f"tracked build input is not a regular file: {relative}")
        normalized = relative.replace("\\", "/")
        entries.append(
            {
                "kind": "repository_tracked",
                "source_path": normalized,
                "archive_path": f"repository/{normalized}",
                "size": path.stat().st_size,
                "sha256": sha256_file(path),
            }
        )
    if not FONT_PATH.is_file():
        raise MatrixError(f"required external build input is missing: {FONT_PATH}")
    entries.append(
        {
            "kind": "external_build_input",
            "source_path": str(FONT_PATH),
            "archive_path": "external/Windows/Fonts/consola.ttf",
            "size": FONT_PATH.stat().st_size,
            "sha256": sha256_file(FONT_PATH),
        }
    )
    digest = hashlib.sha256()
    for entry in entries:
        digest.update(entry["archive_path"].encode("utf-8"))
        digest.update(b"\0")
        digest.update(str(entry["size"]).encode("ascii"))
        digest.update(b"\0")
        digest.update(entry["sha256"].encode("ascii"))
        digest.update(b"\n")
    return entries, digest.hexdigest()


def source_entry_path(source_root: Path, entry: Mapping[str, Any]) -> Path:
    if entry["kind"] == "repository_tracked":
        return source_root / Path(str(entry["source_path"]))
    return Path(str(entry["source_path"]))


def assert_source_unchanged(
    source_root: Path,
    source_state: Mapping[str, Any],
    entries: Sequence[Mapping[str, Any]],
    label: str,
) -> None:
    observed_sha = _git_text(source_root, "rev-parse", "HEAD")
    observed_branch = _git_text(source_root, "branch", "--show-current")
    observed_status = list(_status_records(source_root))
    if (
        observed_sha != source_state["sha"]
        or observed_branch != source_state["branch"]
        or observed_status != source_state["status_porcelain"]
    ):
        raise MatrixError(
            f"source Git identity changed at {label}: HEAD={observed_sha}, "
            f"branch={observed_branch!r}, status={observed_status}"
        )
    if source_state.get("upstream") is not None:
        observed_upstream = _git_text(
            source_root,
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        )
        observed_upstream_sha = _git_text(source_root, "rev-parse", "@{upstream}")
        observed_counts = _git_text(
            source_root, "rev-list", "--left-right", "--count", "HEAD...@{upstream}"
        ).split()
        if (
            observed_upstream != source_state["upstream"]
            or observed_upstream_sha != source_state["upstream_sha"]
            or observed_counts != source_state["ahead_behind"]
        ):
            raise MatrixError(
                f"source upstream identity changed at {label}: upstream={observed_upstream!r}, "
                f"SHA={observed_upstream_sha}, counts={observed_counts}"
            )
    for entry in entries:
        path = source_entry_path(source_root, entry)
        if (
            not path.is_file()
            or path.stat().st_size != entry["size"]
            or sha256_file(path) != entry["sha256"]
        ):
            raise MatrixError(
                f"exact source input changed at {label}: {entry['archive_path']}"
            )
    if (
        _git_text(source_root, "rev-parse", "HEAD") != source_state["sha"]
        or _git_text(source_root, "branch", "--show-current") != source_state["branch"]
        or list(_status_records(source_root)) != source_state["status_porcelain"]
    ):
        raise MatrixError(
            f"source Git identity changed during the {label} exact-byte scan"
        )


def _zip_stream_member(
    archive: zipfile.ZipFile, source: Path, archive_name: str
) -> None:
    info = zipfile.ZipInfo(archive_name, FIXED_ZIP_TIME)
    info.compress_type = zipfile.ZIP_DEFLATED
    info.external_attr = 0o100644 << 16
    with (
        source.open("rb") as input_stream,
        archive.open(info, "w", force_zip64=True) as output_stream,
    ):
        shutil.copyfileobj(input_stream, output_stream, length=1024 * 1024)


def write_source_archive(
    path: Path, source_root: Path, entries: Sequence[Mapping[str, Any]]
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        with zipfile.ZipFile(
            path, "x", compression=zipfile.ZIP_DEFLATED, allowZip64=True
        ) as archive:
            for entry in entries:
                _zip_stream_member(
                    archive,
                    source_entry_path(source_root, entry),
                    str(entry["archive_path"]),
                )
    except FileExistsError as error:
        raise MatrixError(f"refusing to overwrite source archive: {path}") from error


def verify_source_archive(path: Path, entries: Sequence[Mapping[str, Any]]) -> None:
    expected = {str(entry["archive_path"]): entry for entry in entries}
    with zipfile.ZipFile(path) as archive:
        observed = [info.filename for info in archive.infolist() if not info.is_dir()]
        if len(observed) != len(set(observed)) or set(observed) != set(expected):
            raise MatrixError(
                "exact source archive inventory does not match source manifest"
            )
        for name in observed:
            digest = hashlib.sha256()
            size = 0
            with archive.open(name) as stream:
                for block in iter(lambda: stream.read(1024 * 1024), b""):
                    digest.update(block)
                    size += len(block)
            entry = expected[name]
            if size != entry["size"] or digest.hexdigest() != entry["sha256"]:
                raise MatrixError(f"exact source archive byte mismatch: {name}")


def write_git_archive(path: Path, source_root: Path, source_sha: str) -> None:
    if path.exists():
        raise MatrixError(f"refusing to overwrite Git archive: {path}")
    result = _git(
        source_root,
        "archive",
        "--format=zip",
        f"--output={path}",
        source_sha,
        check=False,
    )
    if result.returncode != 0 or not path.is_file():
        message = result.stderr.decode("utf-8", errors="replace").strip()
        raise MatrixError(f"failed to create canonical Git archive: {message}")


def estimate_output_bytes(profile: Mapping[str, Any]) -> int:
    cells = int(profile["width"]) * int(profile["height"])
    chunks = math.ceil(int(profile["width"]) / int(profile["chunk_size"])) * math.ceil(
        int(profile["height"]) / int(profile["chunk_size"])
    )
    per_scenario = (
        cells * 135
        + chunks * 160
        + int(profile["trials"]) * int(profile["mode_b_ticks"]) * 8_000
    )
    source_and_binaries_allowance = 900 * 1024 * 1024
    return len(SCENARIOS) * per_scenario + source_and_binaries_allowance


def run_id_for(mode: str, source_sha: str, input_digest: str) -> str:
    prefix = "g8c-pilot" if mode == "pilot" else "g8c-official-matrix"
    return f"{prefix}-{source_sha[:12]}-{input_digest[:12]}"


def create_run_directory(artifact_root: Path, mode: str, run_id: str) -> Path:
    parent = artifact_root / "scratch" if mode == "pilot" else artifact_root
    parent.mkdir(parents=True, exist_ok=True)
    run_dir = parent / run_id
    try:
        run_dir.mkdir()
    except FileExistsError as error:
        raise MatrixError(
            f"capture already exists for this exact source seal; rerun is forbidden: {run_dir}"
        ) from error
    return run_dir


def run_logged(
    argv: Sequence[str],
    *,
    cwd: Path,
    stdout_path: Path,
    stderr_path: Path,
    record_path: Path,
    role: str,
    scenario: str | None,
    run_root: Path,
    expected_outputs: Sequence[Path],
) -> dict[str, Any]:
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stderr_path.parent.mkdir(parents=True, exist_ok=True)
    started = utc_now()
    environment = os.environ.copy()
    try:
        git_config_count = int(environment.get("GIT_CONFIG_COUNT", "0"))
    except ValueError:
        git_config_count = 0
    environment["GIT_CONFIG_COUNT"] = str(git_config_count + 1)
    environment[f"GIT_CONFIG_KEY_{git_config_count}"] = "safe.directory"
    environment[f"GIT_CONFIG_VALUE_{git_config_count}"] = cwd.resolve().as_posix()
    try:
        with (
            stdout_path.open("xb") as stdout_stream,
            stderr_path.open("xb") as stderr_stream,
        ):
            result = subprocess.run(
                list(argv),
                cwd=cwd,
                env=environment,
                stdout=stdout_stream,
                stderr=stderr_stream,
                check=False,
            )
            stdout_stream.flush()
            stderr_stream.flush()
            os.fsync(stdout_stream.fileno())
            os.fsync(stderr_stream.fileno())
    except FileExistsError as error:
        raise MatrixError(f"refusing to overwrite process log for {role}") from error
    record = {
        "schema_version": PROCESS_SCHEMA,
        "role": role,
        "scenario": scenario,
        "argv": [str(item) for item in argv],
        "cwd": str(cwd.resolve()),
        "started_at_utc": started,
        "ended_at_utc": utc_now(),
        "exit_code": result.returncode,
        "environment_overrides": {
            "GIT_CONFIG_COUNT": str(git_config_count + 1),
            f"GIT_CONFIG_KEY_{git_config_count}": "safe.directory",
            f"GIT_CONFIG_VALUE_{git_config_count}": cwd.resolve().as_posix(),
        },
        "stdout_path": safe_relative(stdout_path, run_root),
        "stderr_path": safe_relative(stderr_path, run_root),
        "expected_outputs": [
            safe_relative(path, run_root) for path in expected_outputs
        ],
    }
    write_new_json(record_path, record)
    if result.returncode != 0:
        raise MatrixError(
            f"{role} failed with exit code {result.returncode}; incomplete capture preserved without receipt"
        )
    missing = [str(path) for path in expected_outputs if not path.is_file()]
    if missing:
        raise MatrixError(f"{role} omitted expected outputs: {missing}")
    return record


def copy_new(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    try:
        with source.open("rb") as input_stream, destination.open("xb") as output_stream:
            shutil.copyfileobj(input_stream, output_stream, length=1024 * 1024)
            output_stream.flush()
            os.fsync(output_stream.fileno())
    except FileExistsError as error:
        raise MatrixError(
            f"refusing to overwrite frozen binary: {destination}"
        ) from error


def remove_isolated_target(path: Path, artifact_root: Path, run_id: str) -> None:
    resolved = path.resolve()
    root = artifact_root.resolve()
    expected_prefix = f".{run_id}-build-"
    if resolved.parent != root or not resolved.name.startswith(expected_prefix):
        raise MatrixError(
            f"refusing to remove unexpected isolated target path: {resolved}"
        )
    try:
        shutil.rmtree(resolved)
    except OSError as error:
        raise MatrixError(
            f"failed to remove isolated target {resolved}: {error}"
        ) from error
    if resolved.exists():
        raise MatrixError(f"isolated target still exists after cleanup: {resolved}")


def build_and_freeze(
    source_root: Path,
    artifact_root: Path,
    run_dir: Path,
    run_id: str,
) -> tuple[dict[str, dict[str, Any]], str]:
    cargo = shutil.which("cargo")
    if not cargo:
        raise MatrixError("cargo executable was not found")
    temporary_target = Path(
        tempfile.mkdtemp(prefix=f".{run_id}-build-", dir=artifact_root)
    )
    try:
        argv = [
            cargo,
            "build",
            "--locked",
            "--release",
            "--target-dir",
            str(temporary_target),
            "-p",
            "powdergame-benchmark",
            "-p",
            "powdergame-windows",
        ]
        benchmark_built = temporary_target / "release" / "powdergame-benchmark.exe"
        windows_built = temporary_target / "release" / "powdergame-windows.exe"
        run_logged(
            argv,
            cwd=source_root,
            stdout_path=run_dir / "build" / "stdout.log",
            stderr_path=run_dir / "build" / "stderr.log",
            record_path=run_dir / "build" / "COMMAND.json",
            role="isolated-locked-release-build",
            scenario=None,
            run_root=run_dir,
            expected_outputs=(),
        )
        if not benchmark_built.is_file() or not windows_built.is_file():
            raise MatrixError(
                "isolated build did not produce both required executables"
            )
        frozen = {
            "benchmark": run_dir / "frozen-binary" / "powdergame-benchmark.exe",
            "windows": run_dir / "frozen-binary" / "powdergame-windows.exe",
        }
        copy_new(benchmark_built, frozen["benchmark"])
        copy_new(windows_built, frozen["windows"])
        records: dict[str, dict[str, Any]] = {}
        for role, path in frozen.items():
            records[role] = {
                "path": safe_relative(path, run_dir),
                "size": path.stat().st_size,
                "sha256": sha256_file(path),
                "build_profile": "release",
            }
        return records, safe_relative(run_dir / "build" / "COMMAND.json", run_dir)
    finally:
        remove_isolated_target(temporary_target, artifact_root, run_id)


def headless_paths(run_dir: Path, scenario: str) -> dict[str, Path]:
    directory = run_dir / "raw" / "headless" / scenario
    summary = directory / "summary.csv"
    return {
        "summary": summary,
        "raw_ticks": directory / "summary_raw_ticks.csv",
        "raw_cells": directory / "summary_raw_cells.csv",
        "raw_chunks": directory / "summary_raw_chunks.csv",
        "manifest": run_dir / "scenarios" / scenario / "HEADLESS_MANIFEST.json",
    }


def benchmark_command(
    binary: Path, scenario: str, profile: Mapping[str, Any], summary: Path
) -> list[str]:
    return [
        str(binary),
        "--scenario",
        scenario,
        "--width",
        str(profile["width"]),
        "--height",
        str(profile["height"]),
        "--chunk",
        str(profile["chunk_size"]),
        "--sleep",
        "on" if profile["sleep_enabled"] else "off",
        "--threshold",
        str(profile["sleep_threshold"]),
        "--prewarm-secs",
        str(profile["prewarm_secs"]),
        "--throughput-ticks",
        str(profile["mode_a_ticks"]),
        "--profile-ticks",
        str(profile["mode_b_ticks"]),
        "--overhead-ticks",
        str(profile["overhead_ticks"]),
        "--trials",
        str(profile["trials"]),
        "--csv",
        str(summary),
    ]


def windows_worker_command(
    binary: Path,
    mode: str,
    scenario: str,
    profile: Mapping[str, Any],
    run_id: str,
    binary_sha256: str,
    raw_csv: Path,
    metadata_json: Path,
) -> list[str]:
    argv = [
        str(binary),
        "--g8c-worker",
        "--mode",
        mode,
        "--run-id",
        run_id,
        "--binary-sha256",
        binary_sha256,
        "--scenario",
        scenario,
        "--width",
        str(profile["width"]),
        "--height",
        str(profile["height"]),
        "--chunk",
        str(profile["chunk_size"]),
        "--sleep",
        "on" if profile["sleep_enabled"] else "off",
        "--threshold",
        str(profile["sleep_threshold"]),
        "--prewarm-secs",
        str(profile["prewarm_secs"]),
        "--trials",
        str(profile["trials"]),
        "--target-tps",
        str(profile["target_tps"]),
    ]
    if mode == "coexistence":
        if profile["mode_c_measurement_secs"] is not None:
            argv.extend(["--measurement-secs", str(profile["mode_c_measurement_secs"])])
        else:
            argv.extend(
                ["--measurement-frames", str(profile["mode_c_measurement_frames"])]
            )
    elif mode == "render-profile":
        argv.extend(["--profile-frames", str(profile["mode_d_profile_frames"])])
    else:
        raise MatrixError(f"invalid windowed worker mode: {mode}")
    argv.extend(["--raw-csv", str(raw_csv), "--metadata-json", str(metadata_json)])
    return argv


def read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise MatrixError(f"invalid JSON {path}: {error}") from error
    if not isinstance(value, dict):
        raise MatrixError(f"JSON root must be an object: {path}")
    return value


def validate_csv_identity(
    path: Path, schema: str, scenario: str
) -> tuple[str, str, str]:
    with path.open("r", encoding="utf-8-sig", newline="") as stream:
        reader = csv.DictReader(stream)
        first = next(reader, None)
    if first is None:
        raise MatrixError(f"CSV has no evidence rows: {path}")
    if first.get("schema_version") != schema:
        raise MatrixError(f"unexpected schema in {path}: {first.get('schema_version')}")
    if "scenario" in first and first.get("scenario") != scenario:
        raise MatrixError(f"unexpected scenario in {path}: {first.get('scenario')}")
    return (
        first.get("run_id", ""),
        first.get("commit_sha", ""),
        first.get("git_state", ""),
    )


def write_headless_manifest(
    run_dir: Path,
    scenario: str,
    paths: Mapping[str, Path],
    profile: Mapping[str, Any],
    matrix_run_id: str,
    source_state: Mapping[str, Any],
    benchmark_binary: Mapping[str, Any],
) -> dict[str, Any]:
    inner_run_id, inner_sha, inner_state = validate_csv_identity(
        paths["summary"], INNER_HEADLESS_SCHEMA, scenario
    )
    if inner_sha != source_state["sha"] or inner_state != source_state["git_state"]:
        raise MatrixError(
            f"headless provenance mismatch for {scenario}: SHA={inner_sha}, state={inner_state}"
        )
    for key in ("raw_ticks", "raw_cells", "raw_chunks"):
        observed_identity = validate_csv_identity(
            paths[key], INNER_HEADLESS_SCHEMA, scenario
        )
        if observed_identity != (inner_run_id, inner_sha, inner_state):
            raise MatrixError(
                f"headless raw-file identity mismatch for {scenario}: "
                f"{key}={observed_identity}, summary={(inner_run_id, inner_sha, inner_state)}"
            )
    files = {
        key: {
            "path": safe_relative(paths[key], run_dir),
            "size": paths[key].stat().st_size,
            "sha256": sha256_file(paths[key]),
        }
        for key in ("summary", "raw_ticks", "raw_cells", "raw_chunks")
    }
    manifest = {
        "schema_version": HEADLESS_SCHEMA,
        "matrix_run_id": matrix_run_id,
        "scenario": scenario,
        "inner_schema_version": INNER_HEADLESS_SCHEMA,
        "inner_run_id": inner_run_id,
        "source": {
            "sha": source_state["sha"],
            "git_state": source_state["git_state"],
        },
        "frozen_benchmark_binary": dict(benchmark_binary),
        "common_config": dict(profile),
        "files": files,
    }
    write_new_json(paths["manifest"], manifest)
    return manifest


def validate_worker_metadata(
    path: Path,
    *,
    schema: str,
    mode: str,
    scenario: str,
    run_id: str,
    source_sha: str,
    source_git_state: str,
    binary_sha256: str,
    profile: Mapping[str, Any],
    recorded_raw_csv_path: Path | None = None,
) -> dict[str, Any]:
    metadata = read_json(path)
    expected_fields = {
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
    if set(metadata) != expected_fields:
        raise MatrixError(
            f"{mode} metadata field inventory mismatch for {scenario}: "
            f"missing={sorted(expected_fields - set(metadata))}, "
            f"extra={sorted(set(metadata) - expected_fields)}"
        )
    expected = {
        "schema_version": schema,
        "run_id": run_id,
        "mode": mode,
        "scenario": scenario,
        "source_sha": source_sha,
        "git_state": source_git_state,
        "binary_sha256": binary_sha256,
        "build_profile": "release",
        "hud_enabled": False,
        "inspector_enabled": False,
        "text_diagnostics_enabled": False,
        "screenshot_readback_enabled": False,
    }
    for key, value in expected.items():
        if metadata.get(key) != value:
            raise MatrixError(
                f"{mode} metadata mismatch for {scenario}: {key}={metadata.get(key)!r}, expected {value!r}"
            )
    if bool(metadata.get("timestamp_query_enabled")) != (mode == "render-profile"):
        raise MatrixError(f"{mode} timestamp-query isolation mismatch for {scenario}")
    requested = metadata.get("requested_config", {})
    expected_requested = {
        "width": profile["width"],
        "height": profile["height"],
        "chunk_size": profile["chunk_size"],
        "sleep_enabled": profile["sleep_enabled"],
        "sleep_threshold": profile["sleep_threshold"],
        "prewarm_secs": profile["prewarm_secs"],
        "trials": profile["trials"],
        "target_tps": profile["target_tps"],
        "measurement_secs": profile["mode_c_measurement_secs"]
        if mode == "coexistence"
        else None,
        "measurement_frames": profile["mode_c_measurement_frames"]
        if mode == "coexistence"
        else None,
        "profile_frames": profile["mode_d_profile_frames"]
        if mode == "render-profile"
        else None,
    }
    if not isinstance(requested, dict) or set(requested) != set(expected_requested):
        raise MatrixError(
            f"{mode} requested config field inventory mismatch for {scenario}"
        )
    for key, value in expected_requested.items():
        observed = requested.get(key)
        if isinstance(value, float) and isinstance(observed, (int, float)):
            matches = math.isclose(float(observed), value, rel_tol=0.0, abs_tol=1e-9)
        else:
            matches = observed == value
        if not matches:
            raise MatrixError(
                f"{mode} requested config mismatch for {scenario}: {key}={observed!r}, expected {value!r}"
            )
    surface = metadata.get("actual_surface", {})
    if (
        not isinstance(surface, dict)
        or set(surface) != {"width", "height", "format", "present_mode"}
        or surface.get("width") != profile["render_width"]
        or surface.get("height") != profile["render_height"]
        or not str(surface.get("format", "")).strip()
        or str(surface.get("present_mode", "")).lower() != "fifo"
    ):
        raise MatrixError(
            f"{mode} actual surface contract mismatch for {scenario}: {surface}"
        )
    validate_window_lifecycle_metadata(
        metadata.get("window_lifecycle"),
        mode=mode,
        scenario=scenario,
        required_width=int(profile["render_width"]),
        required_height=int(profile["render_height"]),
    )
    adapter = metadata.get("adapter", {})
    if (
        not isinstance(adapter, dict)
        or set(adapter)
        != {"name", "vendor", "device", "backend", "driver", "driver_info"}
        or "RTX 5090" not in str(adapter.get("name", "")).upper()
        or adapter.get("vendor") != 0x10DE
        or not isinstance(adapter.get("device"), int)
        or adapter.get("device", 0) <= 0
        or str(adapter.get("backend", "")).lower() != "dx12"
    ):
        raise MatrixError(
            f"{mode} requires NVIDIA RTX 5090 / DX12 for official comparability: {adapter}"
        )
    expected_raw_csv = (
        path.with_suffix(".csv").resolve()
        if recorded_raw_csv_path is None
        else recorded_raw_csv_path.resolve()
    )
    try:
        observed_raw_csv = Path(str(metadata.get("raw_csv", ""))).resolve(strict=False)
    except OSError as error:
        raise MatrixError(
            f"{mode} raw CSV path is invalid for {scenario}: {error}"
        ) from error
    if observed_raw_csv != expected_raw_csv:
        raise MatrixError(
            f"{mode} raw CSV path mismatch for {scenario}: "
            f"observed {observed_raw_csv}, expected {expected_raw_csv}"
        )
    if (
        metadata.get("device_error_count") != 0
        or metadata.get("device_errors") != []
        or metadata.get("surface_error_count") != 0
        or metadata.get("surface_errors") != []
    ):
        raise MatrixError(
            f"{mode} successful metadata records device/surface errors for {scenario}"
        )
    trials = metadata.get("trials")
    if not isinstance(trials, list) or len(trials) != int(profile["trials"]):
        raise MatrixError(f"{mode} metadata trial inventory mismatch for {scenario}")
    return metadata


def validate_window_lifecycle_metadata(
    value: Any,
    *,
    mode: str,
    scenario: str,
    required_width: int,
    required_height: int,
) -> None:
    label = f"{mode} window lifecycle for {scenario}"
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
    if not isinstance(value, dict) or set(value) != expected_fields:
        observed_fields = set(value) if isinstance(value, dict) else set()
        raise MatrixError(
            f"{label} field inventory mismatch: "
            f"missing={sorted(expected_fields - observed_fields)}, "
            f"extra={sorted(observed_fields - expected_fields)}"
        )

    def nonnegative_integer(field: str) -> int:
        observed = value[field]
        if isinstance(observed, bool) or not isinstance(observed, int) or observed < 0:
            raise MatrixError(f"{label}.{field} must be a nonnegative integer")
        return observed

    required = (
        nonnegative_integer("required_width"),
        nonnegative_integer("required_height"),
    )
    initial_live = (
        nonnegative_integer("initial_live_width"),
        nonnegative_integer("initial_live_height"),
    )
    last_live = (
        nonnegative_integer("last_live_width"),
        nonnegative_integer("last_live_height"),
    )
    expected_size = (required_width, required_height)
    if (
        required != expected_size
        or initial_live != expected_size
        or last_live != expected_size
    ):
        raise MatrixError(
            f"{label} requires required/initial/last live size {required_width}x{required_height}: "
            f"required={required}, initial={initial_live}, last={last_live}"
        )
    if value["initial_live_size_confirmed"] is not True:
        raise MatrixError(f"{label}.initial_live_size_confirmed must be true")

    recorded_counts = {
        "canonical_no_op": nonnegative_integer("canonical_noop_count"),
        "stale_payload_ignored": nonnegative_integer("stale_payload_count"),
        "fatal_noncanonical_live_size": nonnegative_integer("fatal_live_resize_count"),
    }
    event_count = nonnegative_integer("event_count")
    events = value["events"]
    if not isinstance(events, list):
        raise MatrixError(f"{label}.events must be an array")
    if event_count != len(events):
        raise MatrixError(
            f"{label}.event_count mismatch: {event_count} != {len(events)}"
        )

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
    for index, event in enumerate(events):
        event_label = f"{label}.events[{index}]"
        if not isinstance(event, dict) or set(event) != expected_event_fields:
            observed_fields = set(event) if isinstance(event, dict) else set()
            raise MatrixError(
                f"{event_label} field inventory mismatch: "
                f"missing={sorted(expected_event_fields - observed_fields)}, "
                f"extra={sorted(observed_fields - expected_event_fields)}"
            )
        if event["event_kind"] not in allowed_event_kinds:
            raise MatrixError(
                f"{event_label}.event_kind is invalid: {event['event_kind']!r}"
            )

        dimensions: dict[str, int] = {}
        for field in (
            "payload_width",
            "payload_height",
            "live_width",
            "live_height",
        ):
            observed = event[field]
            if (
                isinstance(observed, bool)
                or not isinstance(observed, int)
                or observed < 0
            ):
                raise MatrixError(
                    f"{event_label}.{field} must be a nonnegative integer"
                )
            dimensions[field] = observed
        payload_size = (dimensions["payload_width"], dimensions["payload_height"])
        live_size = (dimensions["live_width"], dimensions["live_height"])
        if live_size != expected_size:
            expected_classification = "fatal_noncanonical_live_size"
        elif payload_size == expected_size:
            expected_classification = "canonical_no_op"
        else:
            expected_classification = "stale_payload_ignored"
        if (
            event["event_kind"] == "redraw_guard"
            and expected_classification != "fatal_noncanonical_live_size"
        ):
            raise MatrixError(
                f"{event_label} redraw_guard may only record a fatal live-size observation"
            )
        if event["classification"] != expected_classification:
            raise MatrixError(
                f"{event_label}.classification mismatch: "
                f"{event['classification']!r} != {expected_classification!r}"
            )
        recomputed_counts[expected_classification] += 1
        if live_size != expected_size:
            raise MatrixError(
                f"{event_label} records noncanonical live size {live_size}; "
                f"successful metadata requires {expected_size}"
            )

    if recorded_counts != recomputed_counts:
        raise MatrixError(
            f"{label} counter mismatch: recorded={recorded_counts}, "
            f"recomputed={recomputed_counts}"
        )
    if recorded_counts["fatal_noncanonical_live_size"] != 0:
        raise MatrixError(f"{label} records a fatal live resize")


def nearest_percentile(values: Sequence[float], percent: float) -> float:
    if not values:
        raise MatrixError("cannot calculate a percentile from an empty sample")
    ordered = sorted(values)
    position = percent / 100.0 * (len(ordered) - 1)
    index = math.floor(position + 0.5)
    return ordered[min(index, len(ordered) - 1)]


def numeric_stats(values: Sequence[float]) -> dict[str, float | int]:
    if not values or any(not math.isfinite(value) for value in values):
        raise MatrixError("statistics require finite non-empty values")
    return {
        "count": len(values),
        "p50": nearest_percentile(values, 50.0),
        "p95": nearest_percentile(values, 95.0),
        "p99": nearest_percentile(values, 99.0),
        "mean": statistics.fmean(values),
        "min": min(values),
        "max": max(values),
    }


def window_numeric_stats(values: Sequence[float]) -> dict[str, float | int]:
    # All four G8-C modes deliberately share the historical G8-A rule.
    return numeric_stats(values)


def csv_bool(value: str, *, field: str, path: Path) -> bool:
    normalized = value.strip().lower()
    if normalized in ("1", "true"):
        return True
    if normalized in ("0", "false"):
        return False
    raise MatrixError(f"invalid boolean field {field} in {path}: {value!r}")


def _float(row: Mapping[str, str], key: str, path: Path) -> float:
    try:
        value = float(row[key])
    except (KeyError, ValueError) as error:
        raise MatrixError(f"invalid numeric field {key} in {path}") from error
    if not math.isfinite(value):
        raise MatrixError(f"non-finite numeric field {key} in {path}")
    return value


def _headless_summary_rows(path: Path, scenario: str) -> list[dict[str, str]]:
    if scenario not in SCENARIOS:
        raise MatrixError(f"unsupported headless scenario identity: {scenario!r}")
    with path.open("r", encoding="utf-8-sig", newline="") as stream:
        reader = csv.DictReader(stream)
        if tuple(reader.fieldnames or ()) != HEADLESS_SUMMARY_HEADER:
            raise MatrixError(
                f"unexpected historical headless summary header in {path}: "
                f"{tuple(reader.fieldnames or ())!r}"
            )
        rows: list[dict[str, str]] = []
        identities: set[tuple[str, str, str, str, str]] = set()
        run_ids: set[str] = set()
        for row_number, raw in enumerate(reader, 2):
            if None in raw or any(value is None for value in raw.values()):
                raise MatrixError(
                    f"incomplete headless summary row {row_number} in {path}"
                )
            row = {str(key): str(value) for key, value in raw.items()}
            if row["schema_version"] != INNER_HEADLESS_SCHEMA:
                raise MatrixError(
                    f"unexpected headless schema at {path}:{row_number}: "
                    f"{row['schema_version']!r}"
                )
            run_ids.add(row["run_id"])
            scenario_tokens = {
                token.strip()
                for token in row["method_note"].split(";")
                if token.strip().startswith("scenario=")
            }
            if scenario_tokens != {f"scenario={scenario}"}:
                raise MatrixError(
                    f"headless scenario identity mismatch at {path}:{row_number}: "
                    f"{sorted(scenario_tokens)!r}"
                )
            if row["measurement_mode"] not in {
                "production_throughput",
                "isolated_profiled_tick",
            }:
                raise MatrixError(
                    f"unexpected measurement_mode at {path}:{row_number}: "
                    f"{row['measurement_mode']!r}"
                )
            identity = (
                row["measurement_mode"],
                row["metric_type"],
                row["selection"],
                row["trial"],
                row["name"],
            )
            if identity in identities:
                raise MatrixError(
                    f"duplicate headless summary metric identity in {path}: {identity!r}"
                )
            identities.add(identity)
            rows.append(row)
    if not rows:
        raise MatrixError(f"headless summary has no evidence rows: {path}")
    if len(run_ids) != 1:
        raise MatrixError(
            f"headless summary has inconsistent run IDs: {sorted(run_ids)!r}"
        )
    run_id = next(iter(run_ids))
    if not run_id.startswith(f"g8b-{scenario}-"):
        raise MatrixError(
            f"headless run ID is not bound to scenario {scenario}: {run_id!r}"
        )
    return rows


def _require_headless_row(
    rows: Sequence[Mapping[str, str]],
    *,
    path: Path,
    measurement_mode: str,
    metric_type: str,
    selection: str,
    trial: str,
    name: str,
    unit: str,
) -> Mapping[str, str]:
    matches = [
        row
        for row in rows
        if row["measurement_mode"] == measurement_mode
        and row["metric_type"] == metric_type
        and row["selection"] == selection
        and row["trial"] == trial
        and row["name"] == name
    ]
    identity = (measurement_mode, metric_type, selection, trial, name)
    if len(matches) != 1:
        raise MatrixError(
            f"headless summary requires exactly one row {identity!r}; "
            f"observed {len(matches)} in {path}"
        )
    row = matches[0]
    if row["unit"] != unit:
        raise MatrixError(
            f"headless summary unit mismatch for {identity!r}: "
            f"{row['unit']!r} != {unit!r}"
        )
    return row


def _assert_close(actual: float, expected: float, label: str) -> None:
    if not math.isclose(actual, expected, rel_tol=5.0e-7, abs_tol=5.0e-7):
        raise MatrixError(f"{label} mismatch: {actual} != {expected}")


def _headless_mode_a_adapter(
    rows: Sequence[Mapping[str, str]],
    *,
    path: Path,
    profile: Mapping[str, Any],
) -> dict[str, list[float]]:
    if any(row["name"] == "wall_ms_per_tick" for row in rows):
        raise MatrixError(
            "historical headless summary must use raw name 'wall_per_tick'; "
            "the internal alias 'wall_ms_per_tick' is forbidden"
        )
    wrong_units = sorted(
        {
            row["unit"]
            for row in rows
            if row["name"] == "wall_per_tick" and row["unit"] != "ms/tick"
        }
    )
    if wrong_units:
        raise MatrixError(
            f"historical wall_per_tick rows require unit 'ms/tick': {wrong_units!r}"
        )

    trial_count = int(profile["trials"])
    mode_a_ticks = int(profile["mode_a_ticks"])
    expected_identities = {
        ("throughput_trial", "trial", str(trial), name, unit)
        for trial in range(1, trial_count + 1)
        for name, unit in (
            ("elapsed_wall", "ms"),
            ("wall_per_tick", "ms/tick"),
            ("sustained_tps", "ticks/s"),
        )
    } | {
        ("throughput_summary", "all_trials", "all", name, unit)
        for name, unit in (
            ("wall_per_tick", "ms/tick"),
            ("sustained_tps", "ticks/s"),
        )
    }
    observed_identities = {
        (
            row["metric_type"],
            row["selection"],
            row["trial"],
            row["name"],
            row["unit"],
        )
        for row in rows
        if row["measurement_mode"] == "production_throughput"
    }
    if observed_identities != expected_identities:
        raise MatrixError(
            "historical throughput row identity inventory mismatch: "
            f"missing={sorted(expected_identities - observed_identities)!r}, "
            f"unexpected={sorted(observed_identities - expected_identities)!r}"
        )
    trial_values: dict[str, list[float]] = {
        "elapsed_wall_ms": [],
        "wall_ms_per_tick": [],
        "sustained_tps": [],
    }
    raw_specs = (
        ("elapsed_wall", "ms", "elapsed_wall_ms"),
        ("wall_per_tick", "ms/tick", "wall_ms_per_tick"),
        ("sustained_tps", "ticks/s", "sustained_tps"),
    )
    for trial_number in range(1, trial_count + 1):
        observed: dict[str, float] = {}
        for raw_name, unit, internal_name in raw_specs:
            row = _require_headless_row(
                rows,
                path=path,
                measurement_mode="production_throughput",
                metric_type="throughput_trial",
                selection="trial",
                trial=str(trial_number),
                name=raw_name,
                unit=unit,
            )
            value = _float(row, "value", path)
            if value <= 0.0:
                raise MatrixError(
                    f"Mode A trial {trial_number} {raw_name} must be positive in {path}"
                )
            observed[raw_name] = value
            trial_values[internal_name].append(value)
        _assert_close(
            observed["elapsed_wall"],
            observed["wall_per_tick"] * mode_a_ticks,
            f"Mode A trial {trial_number} elapsed wall",
        )
        _assert_close(
            observed["sustained_tps"],
            1000.0 / observed["wall_per_tick"],
            f"Mode A trial {trial_number} sustained TPS",
        )

    for raw_name, unit, internal_name in raw_specs[1:]:
        row = _require_headless_row(
            rows,
            path=path,
            measurement_mode="production_throughput",
            metric_type="throughput_summary",
            selection="all_trials",
            trial="all",
            name=raw_name,
            unit=unit,
        )
        if row["value"] != "":
            raise MatrixError(
                f"all-trials summary {raw_name} must not contain a trial value in {path}"
            )
        try:
            count = int(row["count"])
        except ValueError as error:
            raise MatrixError(
                f"invalid all-trials summary count for {raw_name} in {path}"
            ) from error
        if count != trial_count:
            raise MatrixError(
                f"all-trials summary count for {raw_name} is {count}, expected {trial_count}"
            )
        reconstructed = numeric_stats(trial_values[internal_name])
        for statistic in ("p50", "p95", "mean", "min", "max"):
            _assert_close(
                _float(row, statistic, path),
                float(reconstructed[statistic]),
                f"all-trials {raw_name}.{statistic}",
            )
    return trial_values


def aggregate_headless(
    paths: Mapping[str, Path], profile: Mapping[str, Any], scenario: str
) -> dict[str, Any]:
    rows = _headless_summary_rows(paths["summary"], scenario)
    throughput = _headless_mode_a_adapter(rows, path=paths["summary"], profile=profile)
    memory_total: int | None = None
    adapters: set[tuple[str, str, str, str]] = set()
    for row in rows:
        adapters.add(
            (
                row["adapter_name"],
                row["vendor_id"],
                row["device_id"],
                row["backend"],
            )
        )
    memory_row = _require_headless_row(
        rows,
        path=paths["summary"],
        measurement_mode="isolated_profiled_tick",
        metric_type="application_tracked_buffer_allocation",
        selection="snapshot",
        trial="n/a",
        name="total_tracked",
        unit="bytes",
    )
    memory_value = _float(memory_row, "value", paths["summary"])
    if memory_value < 0.0 or not memory_value.is_integer():
        raise MatrixError("total_tracked must be a nonnegative integral byte count")
    memory_total = int(memory_value)
    if len(adapters) != 1:
        raise MatrixError(f"headless adapter identity is inconsistent: {adapters}")
    adapter_name, vendor_id, device_id, backend = next(iter(adapters))
    if (
        "RTX 5090" not in adapter_name.upper()
        or vendor_id.upper() != "0X10DE"
        or backend.lower() != "dx12"
    ):
        raise MatrixError(
            f"headless modes require NVIDIA RTX 5090 / DX12: {(adapter_name, vendor_id, device_id, backend)}"
        )

    mode_b: dict[str, list[float]] = {
        field: []
        for field in (
            *GROUP_FIELDS,
            "gpu_tick_envelope_ms",
            "gpu_pass_sum_ms",
            "residual_ms",
        )
    }
    with paths["raw_ticks"].open("r", encoding="utf-8-sig", newline="") as stream:
        rows = 0
        for row in csv.DictReader(stream):
            rows += 1
            for field in mode_b:
                mode_b[field].append(_float(row, field, paths["raw_ticks"]))
    trials = int(profile["trials"])
    expected_ticks = trials * int(profile["mode_b_ticks"])
    if rows != expected_ticks:
        raise MatrixError(
            f"Mode B raw tick count mismatch: observed {rows}, expected {expected_ticks}"
        )

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
    with paths["raw_cells"].open("r", encoding="utf-8-sig", newline="") as stream:
        for row in csv.DictReader(stream):
            try:
                mask = int(row["activity_mask"])
            except (KeyError, ValueError) as error:
                raise MatrixError(
                    f"invalid cell census row in {paths['raw_cells']}"
                ) from error
            census["total_cells"] += 1
            census["any_active_cells"] += mask != 0
            census["matter_active_cells"] += (mask & 1) != 0
            census["thermal_active_cells"] += (mask & 2) != 0
            census["pressure_active_cells"] += (mask & 4) != 0
            census["reaction_active_cells"] += (mask & 8) != 0
    with paths["raw_chunks"].open("r", encoding="utf-8-sig", newline="") as stream:
        for row in csv.DictReader(stream):
            try:
                mask = int(row["activity_mask"])
                state = int(row["chunk_state"])
            except (KeyError, ValueError) as error:
                raise MatrixError(
                    f"invalid chunk census row in {paths['raw_chunks']}"
                ) from error
            census["total_chunks"] += 1
            census["active_chunks"] += mask != 0
            census["runnable_chunks"] += state == 0
            census["sleeping_chunks"] += state == 1
    expected_cells = int(profile["width"]) * int(profile["height"])
    expected_chunks = math.ceil(
        int(profile["width"]) / int(profile["chunk_size"])
    ) * math.ceil(int(profile["height"]) / int(profile["chunk_size"]))
    if (
        census["total_cells"] != expected_cells
        or census["total_chunks"] != expected_chunks
    ):
        raise MatrixError(
            f"raw census dimensions mismatch: cells={census['total_cells']}/{expected_cells}, "
            f"chunks={census['total_chunks']}/{expected_chunks}"
        )
    return {
        "mode_a_tps": numeric_stats(throughput["sustained_tps"]),
        "mode_a_elapsed_wall_ms": numeric_stats(throughput["elapsed_wall_ms"]),
        "mode_a_wall_ms_per_tick": numeric_stats(throughput["wall_ms_per_tick"]),
        "mode_b": {name: numeric_stats(values) for name, values in mode_b.items()},
        "census": census,
        "tracked_persistent_gpu_bytes": memory_total,
        "adapter": {
            "name": adapter_name,
            "vendor_id": vendor_id,
            "device_id": device_id,
            "backend": backend,
        },
    }


def aggregate_coexistence(
    path: Path,
    profile: Mapping[str, Any],
    metadata: Mapping[str, Any] | None = None,
) -> dict[str, Any]:
    per_trial: dict[int, dict[str, Any]] = {}
    frame_wall_values: list[float] = []
    with path.open("r", encoding="utf-8-sig", newline="") as stream:
        for row in csv.DictReader(stream):
            if row.get("schema_version") != COEXISTENCE_SCHEMA:
                raise MatrixError(f"unexpected Mode C row schema in {path}")
            trial = int(row["trial"])
            current = per_trial.setdefault(
                trial,
                {
                    "rows": 0,
                    "ticks": 0,
                    "presented": 0,
                    "scheduled": 0,
                    "missed": 0,
                    "catch_up": 0,
                    "failed": 0,
                    "elapsed_ms": 0.0,
                    "frame_indices": [],
                    "sim_ticks": [],
                    "frame_walls": [],
                    "surface_errors": 0,
                },
            )
            current["rows"] += 1
            current["ticks"] += int(row["sim_ticks_executed"])
            presented = csv_bool(row["presented"], field="presented", path=path)
            current["presented"] += int(presented)
            current["scheduled"] = max(
                current["scheduled"], int(row["scheduled_sim_ticks"])
            )
            current["missed"] += int(row["missed_simulation_deadlines"])
            current["catch_up"] += int(row["catch_up_ticks"])
            current["failed"] += int(not presented)
            current["surface_errors"] += int(bool(row.get("surface_error", "")))
            current["elapsed_ms"] = max(
                current["elapsed_ms"], _float(row, "window_elapsed_ms", path)
            )
            current["frame_indices"].append(int(row["frame_index"]))
            current["sim_ticks"].append(int(row["sim_tick"]))
            frame_wall = _float(row, "frame_wall_ms", path)
            current["frame_walls"].append(frame_wall)
            if presented:
                frame_wall_values.append(frame_wall)
    expected_trials = set(range(1, int(profile["trials"]) + 1))
    if set(per_trial) != expected_trials:
        raise MatrixError(
            f"Mode C trial identity mismatch in {path}: {sorted(per_trial)}"
        )
    if profile["mode_c_measurement_frames"] is not None:
        expected = int(profile["mode_c_measurement_frames"])
        if any(value["rows"] != expected for value in per_trial.values()):
            raise MatrixError(f"Mode C pilot frame count mismatch in {path}")
    for trial, value in per_trial.items():
        if value["frame_indices"] != list(range(value["rows"])):
            raise MatrixError(
                f"Mode C frame identity is not contiguous for trial {trial} in {path}"
            )
        if any(
            right < left
            for left, right in zip(value["sim_ticks"], value["sim_ticks"][1:])
        ):
            raise MatrixError(
                f"Mode C simulation tick regressed for trial {trial} in {path}"
            )
        if profile["mode_c_measurement_secs"] is not None:
            requested_ms = float(profile["mode_c_measurement_secs"]) * 1000.0
            if value["elapsed_ms"] < requested_ms:
                raise MatrixError(
                    f"Mode C measured duration is outside the fixed window for trial {trial}: {value['elapsed_ms']} ms"
                )
    sim_rates: list[float] = []
    frame_rates: list[float] = []
    for value in per_trial.values():
        seconds = value["elapsed_ms"] / 1000.0
        if seconds <= 0:
            raise MatrixError(f"Mode C trial has no positive duration in {path}")
        sim_rates.append(value["ticks"] / seconds)
        frame_rates.append(value["presented"] / seconds)
    scheduled = sum(
        max(value["scheduled"], value["ticks"]) for value in per_trial.values()
    )
    result = {
        "actual_simulation_ticks": sum(value["ticks"] for value in per_trial.values()),
        "simulation_tps": window_numeric_stats(sim_rates),
        "presented_frames": sum(value["presented"] for value in per_trial.values()),
        "render_fps": window_numeric_stats(frame_rates),
        "frame_wall_ms": window_numeric_stats(frame_wall_values),
        "missed_simulation_deadlines": sum(
            value["missed"] for value in per_trial.values()
        ),
        "missed_deadline_ratio": (
            sum(value["missed"] for value in per_trial.values()) / scheduled
            if scheduled
            else 0.0
        ),
        "catch_up_ticks": sum(value["catch_up"] for value in per_trial.values()),
        "failed_surface_frames": sum(value["failed"] for value in per_trial.values()),
        "surface_errors": sum(value["surface_errors"] for value in per_trial.values()),
        "device_errors": 0,
    }
    if metadata is not None:
        if int(metadata.get("surface_error_count", -1)) != result["surface_errors"]:
            raise MatrixError(f"Mode C metadata surface-error mismatch in {path}")
        result["device_errors"] = int(metadata.get("device_error_count", -1))
        if result["device_errors"] != 0:
            raise MatrixError(f"Mode C metadata records device errors in {path}")
        summaries = metadata.get("trials")
        if not isinstance(summaries, list) or len(summaries) != int(profile["trials"]):
            raise MatrixError(f"Mode C metadata trial inventory mismatch in {path}")
        for summary in summaries:
            trial = int(summary["trial"])
            raw = per_trial.get(trial)
            if raw is None:
                raise MatrixError(
                    f"Mode C metadata names unknown trial {trial} in {path}"
                )
            seconds = raw["elapsed_ms"] / 1000.0
            expected_values = {
                "elapsed_ms": raw["elapsed_ms"],
                "actual_simulation_ticks": raw["ticks"],
                "actual_simulation_tps": raw["ticks"] / seconds,
                "presented_frames": raw["presented"],
                "render_fps": raw["presented"] / seconds,
                "frame_p50_ms": nearest_percentile(raw["frame_walls"], 50.0),
                "frame_p95_ms": nearest_percentile(raw["frame_walls"], 95.0),
                "frame_p99_ms": nearest_percentile(raw["frame_walls"], 99.0),
                "missed_simulation_deadlines": raw["missed"],
                "missed_deadline_ratio": (
                    raw["missed"] / max(raw["scheduled"], raw["ticks"])
                    if max(raw["scheduled"], raw["ticks"])
                    else 0.0
                ),
                "catch_up_ticks": raw["catch_up"],
                "failed_surface_frames": raw["failed"],
                "surface_errors": raw["surface_errors"],
                "device_errors": 0,
            }
            for key, expected in expected_values.items():
                observed = summary.get(key)
                if isinstance(expected, float):
                    matches = isinstance(observed, (int, float)) and math.isclose(
                        float(observed), expected, rel_tol=1e-6, abs_tol=1e-6
                    )
                else:
                    matches = observed == expected
                if not matches:
                    raise MatrixError(
                        f"Mode C metadata arithmetic mismatch in {path}: "
                        f"trial={trial}, {key}={observed}, expected={expected}"
                    )
    return result


def aggregate_render_profile(
    path: Path,
    profile: Mapping[str, Any],
    metadata: Mapping[str, Any] | None = None,
) -> dict[str, Any]:
    values: list[float] = []
    rows_by_trial: dict[int, int] = {}
    frames_by_trial: dict[int, list[int]] = {}
    sim_ticks_by_trial: dict[int, list[int]] = {}
    gpu_by_trial: dict[int, list[float]] = {}
    with path.open("r", encoding="utf-8-sig", newline="") as stream:
        for row in csv.DictReader(stream):
            if row.get("schema_version") != RENDER_PROFILE_SCHEMA:
                raise MatrixError(f"unexpected Mode D row schema in {path}")
            trial = int(row["trial"])
            rows_by_trial[trial] = rows_by_trial.get(trial, 0) + 1
            frames_by_trial.setdefault(trial, []).append(int(row["frame_index"]))
            sim_ticks_by_trial.setdefault(trial, []).append(int(row["sim_tick"]))
            if not csv_bool(
                row.get("presented", ""), field="presented", path=path
            ) or row.get("surface_error", ""):
                raise MatrixError(
                    f"Mode D measured row was not successfully presented in {path}"
                )
            start = int(row["gpu_start_tick"])
            end = int(row["gpu_end_tick"])
            period = _float(row, "timestamp_period_ns", path)
            observed = _float(row, "gpu_render_ms", path)
            if end <= start or period <= 0:
                raise MatrixError(f"Mode D timestamp ordering failure in {path}")
            reconstructed = (end - start) * period / 1_000_000.0
            if not math.isclose(reconstructed, observed, rel_tol=1e-6, abs_tol=1e-6):
                raise MatrixError(
                    f"Mode D duration mismatch in {path}: observed={observed}, reconstructed={reconstructed}"
                )
            values.append(observed)
            gpu_by_trial.setdefault(trial, []).append(observed)
    expected_trials = set(range(1, int(profile["trials"]) + 1))
    expected_frames = int(profile["mode_d_profile_frames"])
    if set(rows_by_trial) != expected_trials or any(
        count != expected_frames for count in rows_by_trial.values()
    ):
        raise MatrixError(
            f"Mode D trial/frame identity mismatch in {path}: {rows_by_trial}"
        )
    for trial in expected_trials:
        if frames_by_trial[trial] != list(range(expected_frames)):
            raise MatrixError(
                f"Mode D frame identity is not contiguous for trial {trial} in {path}"
            )
        if any(
            right < left
            for left, right in zip(
                sim_ticks_by_trial[trial], sim_ticks_by_trial[trial][1:]
            )
        ):
            raise MatrixError(
                f"Mode D simulation tick regressed for trial {trial} in {path}"
            )
    result = {
        "gpu_render_ms": window_numeric_stats(values),
        "device_errors": 0,
        "surface_errors": 0,
    }
    if metadata is not None:
        result["device_errors"] = int(metadata.get("device_error_count", -1))
        result["surface_errors"] = int(metadata.get("surface_error_count", -1))
        if result["device_errors"] != 0 or result["surface_errors"] != 0:
            raise MatrixError(
                f"Mode D metadata records device/surface errors in {path}"
            )
        summaries = metadata.get("trials")
        if not isinstance(summaries, list) or len(summaries) != int(profile["trials"]):
            raise MatrixError(f"Mode D metadata trial inventory mismatch in {path}")
        for summary in summaries:
            trial = int(summary["trial"])
            trial_values = gpu_by_trial.get(trial)
            if trial_values is None:
                raise MatrixError(
                    f"Mode D metadata names unknown trial {trial} in {path}"
                )
            expected_values = {
                "presented_frames": expected_frames,
                "failed_surface_frames": 0,
                "device_errors": 0,
                "surface_errors": 0,
                "gpu_render_p50_ms": nearest_percentile(trial_values, 50.0),
                "gpu_render_p95_ms": nearest_percentile(trial_values, 95.0),
                "gpu_render_mean_ms": statistics.fmean(trial_values),
            }
            for key, expected in expected_values.items():
                observed = summary.get(key)
                if isinstance(expected, float):
                    matches = isinstance(observed, (int, float)) and math.isclose(
                        float(observed), expected, rel_tol=1e-6, abs_tol=1e-6
                    )
                else:
                    matches = observed == expected
                if not matches:
                    raise MatrixError(
                        f"Mode D metadata arithmetic mismatch in {path}: "
                        f"trial={trial}, {key}={observed}, expected={expected}"
                    )
    return result


def scenario_matrix_row(
    scenario: str,
    source_sha: str,
    headless: Mapping[str, Any],
    coexistence: Mapping[str, Any],
    render: Mapping[str, Any],
) -> dict[str, Any]:
    groups = {field: headless["mode_b"][field]["p50"] for field in GROUP_FIELDS}
    bottleneck_field = max(groups, key=groups.get)
    tracked_bytes = int(headless["tracked_persistent_gpu_bytes"])
    rtx_5090_bytes = 32 * 1024**3
    row = {
        "source_sha": source_sha,
        "scenario": scenario,
        "mode_a_tps_p50": headless["mode_a_tps"]["p50"],
        "mode_a_tps_mean": headless["mode_a_tps"]["mean"],
        "mode_a_tps_min": headless["mode_a_tps"]["min"],
        "mode_a_tps_max": headless["mode_a_tps"]["max"],
        "mode_a_wall_ms_tick_p50": headless["mode_a_wall_ms_per_tick"]["p50"],
        "mode_a_wall_ms_tick_p95": headless["mode_a_wall_ms_per_tick"]["p95"],
        "headroom_60_tps_ratio": headless["mode_a_tps"]["p50"] / 60.0,
        "mode_b_gpu_envelope_p50_ms": headless["mode_b"]["gpu_tick_envelope_ms"]["p50"],
        "mode_b_gpu_envelope_p95_ms": headless["mode_b"]["gpu_tick_envelope_ms"]["p95"],
        "matter_movement_p50_ms": groups["group_matter_movement_ms"],
        "claim_resolve_p50_ms": groups["group_ownership_claim_ms"],
        "thermal_p50_ms": groups["group_thermal_conduction_ms"],
        "reaction_phase_p50_ms": groups["group_reaction_phase_ms"],
        "pressure_structure_p50_ms": groups["group_pressure_structure_ms"],
        "active_sleep_p50_ms": groups["group_active_sleep_management_ms"],
        "residual_p50_ms": headless["mode_b"]["residual_ms"]["p50"],
        **headless["census"],
        "working_chunks": headless["census"]["runnable_chunks"],
        "tracked_persistent_gpu_bytes": headless["tracked_persistent_gpu_bytes"],
        "tracked_persistent_gpu_gib": tracked_bytes / 1024**3,
        "rtx_5090_32gib_tracked_memory_ratio": tracked_bytes / rtx_5090_bytes,
        "rtx_5090_32gib_tracked_memory_headroom_bytes": rtx_5090_bytes - tracked_bytes,
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
        "bottleneck_group": GROUP_LABELS[bottleneck_field],
    }
    return row


def optimization_recommendation(
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


def write_reports(
    run_dir: Path,
    run_id: str,
    mode: str,
    rows: Sequence[Mapping[str, Any]],
    recommendation: str,
    reasons: Sequence[str],
) -> dict[str, str]:
    non_evidence = mode != "official"
    report_dir = run_dir / "report"
    report_dir.mkdir(parents=True, exist_ok=True)
    csv_path = report_dir / "G8C_MATRIX.csv"
    try:
        with csv_path.open("x", encoding="utf-8", newline="") as stream:
            writer = csv.DictWriter(stream, fieldnames=list(rows[0]))
            writer.writeheader()
            writer.writerows(rows)
            stream.flush()
            os.fsync(stream.fileno())
    except FileExistsError as error:
        raise MatrixError(f"refusing to overwrite matrix CSV: {csv_path}") from error
    report_json = {
        "schema_version": REPORT_SCHEMA,
        "matrix_run_id": run_id,
        "run_mode": mode,
        "official_evidence": mode == "official",
        "pilot_must_never_be_promoted": non_evidence,
        "scenarios": list(rows),
        "recommendation": recommendation,
        "recommendation_reasons": list(reasons),
        "activity_count_note": "cell activity subsystem counts overlap and must not be summed",
    }
    json_path = report_dir / "G8C_MATRIX.json"
    write_new_json(json_path, report_json)
    title = (
        "G8-C Official Performance Matrix"
        if mode == "official"
        else "G8-C Matrix Pilot (NON-EVIDENCE)"
    )
    lines = [
        f"# {title}",
        "",
        f"- Matrix: `{run_id}`",
        f"- Run mode: `{mode}`",
        f"- Recommendation: **{recommendation}**",
        "- Fifo render FPS is interpreted with frame percentiles and deadline ratios, "
        "not an exact integer-60 comparison.",
        "- Cell activity subsystem counts overlap and are not summed.",
        "- Memory is app-tracked persistent GPU allocation, not total driver-resident "
        "VRAM; the guard uses the RTX 5090 32 GiB capacity.",
        "",
        "| Scenario | Mode A TPS P50 | Mode B envelope P95 ms | Mode C sim TPS | "
        "Mode C FPS | Frame P95 ms | Mode D render P95 ms | Tracked GiB | Bottleneck |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---|",
    ]
    for row in rows:
        lines.append(
            f"| {row['scenario']} | {row['mode_a_tps_p50']:.3f} | "
            f"{row['mode_b_gpu_envelope_p95_ms']:.3f} | {row['mode_c_simulation_tps']:.3f} | "
            f"{row['mode_c_render_fps']:.3f} | {row['mode_c_frame_p95_ms']:.3f} | "
            f"{row['mode_d_gpu_render_p95_ms']:.3f} | {row['tracked_persistent_gpu_gib']:.3f} | "
            f"{row['bottleneck_group']} |"
        )
    markdown_path = report_dir / "G8C_REPORT.md"
    write_new_text(markdown_path, "\n".join(lines) + "\n")
    bottleneck_path = report_dir / "BOTTLENECK_ANALYSIS.md"
    bottleneck_lines = [
        "# G8-C Bottleneck Analysis",
        "",
        *(
            ["**NON-EVIDENCE PILOT: never promote this output.**", ""]
            if non_evidence
            else []
        ),
    ]
    for row in rows:
        bottleneck_lines.append(
            f"- **{row['scenario']}**: {row['bottleneck_group']} is the largest Mode B grouped P50; "
            f"GPU envelope P95 is {row['mode_b_gpu_envelope_p95_ms']:.3f} ms; "
            f"app-tracked persistent memory is {row['tracked_persistent_gpu_gib']:.3f} GiB "
            f"({row['rtx_5090_32gib_tracked_memory_ratio']:.1%} of 32 GiB)."
        )
    write_new_text(bottleneck_path, "\n".join(bottleneck_lines) + "\n")
    decision_path = report_dir / "OPTIMIZATION_DECISION.md"
    decision_lines = [
        "# G8-C Optimization Decision",
        "",
        *(
            ["**NON-EVIDENCE PILOT: never promote this output.**", ""]
            if non_evidence
            else []
        ),
        f"Recommendation: **{recommendation}**",
        "",
        *[f"- {reason}" for reason in reasons],
        "",
        "This report does not authorize or begin optimization or G9 work.",
    ]
    write_new_text(decision_path, "\n".join(decision_lines) + "\n")
    return {
        "matrix_csv": safe_relative(csv_path, run_dir),
        "matrix_json": safe_relative(json_path, run_dir),
        "report": safe_relative(markdown_path, run_dir),
        "bottleneck_analysis": safe_relative(bottleneck_path, run_dir),
        "optimization_decision": safe_relative(decision_path, run_dir),
    }


def write_hash_inventory(run_dir: Path) -> tuple[Path, list[dict[str, Any]]]:
    hashes_path = run_dir / "HASHES.sha256"
    receipt_path = run_dir / "G8C_MATRIX_RECEIPT.json"
    entries: list[dict[str, Any]] = []
    regular_files = [
        (safe_relative(path, run_dir), path)
        for path in run_dir.rglob("*")
        if path.is_file()
    ]
    for relative, path in sorted(regular_files, key=lambda item: item[0]):
        if path in (hashes_path, receipt_path):
            continue
        entries.append(
            {
                "path": relative,
                "size": path.stat().st_size,
                "sha256": sha256_file(path),
            }
        )
    text = "".join(f"{entry['sha256']}  {entry['path']}\n" for entry in entries)
    write_new_text(hashes_path, text)
    return hashes_path, entries


def write_receipt(
    run_dir: Path,
    run_id: str,
    mode: str,
    source_state: Mapping[str, Any],
    source_digest: str,
    binaries: Mapping[str, Mapping[str, Any]],
    reports: Mapping[str, str],
    verifier: Mapping[str, Any],
    recommendation: str,
    hash_entries: Sequence[Mapping[str, Any]],
) -> Path:
    manifest_path = run_dir / "G8C_MATRIX_MANIFEST.json"
    hashes_path = run_dir / "HASHES.sha256"
    delivery_directory = f"{run_id}-delivery"
    package_name = "G8C_MATRIX_PACKAGE.zip"
    receipt = {
        "schema_version": RECEIPT_SCHEMA,
        "matrix_run_id": run_id,
        "run_mode": mode,
        "complete": True,
        "receipt_is_final_publication_marker": True,
        "published_at_utc": utc_now(),
        "source_sha": source_state["sha"],
        "source_input_digest": source_digest,
        "manifest_sha256": sha256_file(manifest_path),
        "hashes_sha256": sha256_file(hashes_path),
        "hash_entry_count": len(hash_entries),
        "frozen_binaries": binaries,
        "reports": {
            key: {"path": path, "sha256": sha256_file(run_dir / path)}
            for key, path in reports.items()
        },
        "independent_verifier": dict(verifier),
        "recommendation": recommendation,
        "delivery": {
            "sibling_directory": delivery_directory,
            "package_filename": package_name,
            "package_sha256_sidecar": "G8C_MATRIX_PACKAGE_SHA256.txt",
            "hash_binding": (
                "sibling sidecar is created after this receipt and hashes the ZIP64 "
                "package containing this receipt"
            ),
        },
    }
    receipt_path = run_dir / "G8C_MATRIX_RECEIPT.json"
    write_new_json(receipt_path, receipt)
    return receipt_path


def create_package(run_dir: Path) -> tuple[Path, Path, str]:
    delivery = run_dir.parent / f"{run_dir.name}-delivery"
    try:
        delivery.mkdir()
    except FileExistsError as error:
        raise MatrixError(
            f"refusing to overwrite delivery directory: {delivery}"
        ) from error
    package = delivery / "G8C_MATRIX_PACKAGE.zip"
    sidecar = delivery / "G8C_MATRIX_PACKAGE_SHA256.txt"
    try:
        with zipfile.ZipFile(
            package, "x", compression=zipfile.ZIP_DEFLATED, allowZip64=True
        ) as archive:
            for source in sorted(item for item in run_dir.rglob("*") if item.is_file()):
                archive_name = f"{run_dir.name}/{safe_relative(source, run_dir)}"
                _zip_stream_member(archive, source, archive_name)
    except FileExistsError as error:
        raise MatrixError(f"refusing to overwrite package: {package}") from error
    package_hash = sha256_file(package)
    write_new_text(sidecar, f"{package_hash}  {package.name}\n")
    return package, sidecar, package_hash


def verifier_command(
    verifier: Path,
    run_dir: Path,
    package: Path,
    sidecar: Path,
    result_path: Path,
    source_root: Path,
) -> list[str]:
    return [
        sys.executable,
        "-B",
        str(verifier),
        "--run-dir",
        str(run_dir),
        "--package",
        str(package),
        "--sidecar",
        str(sidecar),
        "--write-result",
        str(result_path),
        "--repo-root",
        str(source_root),
    ]


def run_independent_verifier(
    verifier: Path,
    run_dir: Path,
    package: Path,
    sidecar: Path,
    source_root: Path,
) -> Path:
    if not verifier.is_file():
        raise MatrixError(f"frozen independent verifier is missing: {verifier}")
    result_path = package.parent / "G8C_MATRIX_VERIFICATION.json"
    argv = verifier_command(
        verifier, run_dir, package, sidecar, result_path, source_root
    )
    environment = os.environ.copy()
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    result = subprocess.run(
        argv,
        cwd=run_dir.parent,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.stdout:
        print(result.stdout, end="")
    if result.returncode != 0:
        message = result.stderr.strip() or "independent verifier returned no diagnostic"
        raise MatrixError(f"independent verification failed: {message}")
    if not result_path.is_file():
        raise MatrixError("independent verifier did not publish its sibling result")
    return result_path


def directory_byte_inventory(root: Path) -> tuple[list[dict[str, Any]], str, int]:
    root = root.resolve()
    if not root.is_dir():
        raise MatrixError(f"inventory root is not a directory: {root}")
    entries: list[dict[str, Any]] = []
    total_bytes = 0
    regular_files = [
        (safe_relative(path, root), path) for path in root.rglob("*") if path.is_file()
    ]
    for relative, path in sorted(regular_files, key=lambda item: item[0]):
        size = path.stat().st_size
        entries.append({"path": relative, "size": size, "sha256": sha256_file(path)})
        total_bytes += size
    if not entries:
        raise MatrixError(f"inventory root contains no files: {root}")
    digest = hashlib.sha256()
    for entry in entries:
        digest.update(entry["path"].encode("utf-8"))
        digest.update(b"\0")
        digest.update(str(entry["size"]).encode("ascii"))
        digest.update(b"\0")
        digest.update(entry["sha256"].encode("ascii"))
        digest.update(b"\n")
    return entries, digest.hexdigest(), total_bytes


def copy_inventory_new(
    source_root: Path,
    destination_root: Path,
    entries: Sequence[Mapping[str, Any]],
) -> None:
    for entry in entries:
        relative = Path(str(entry["path"]))
        source = source_root / relative
        destination = destination_root / relative
        copy_new(source, destination)
        if (
            destination.stat().st_size != entry["size"]
            or sha256_file(destination) != entry["sha256"]
        ):
            raise MatrixError(
                f"aggregation replay input copy mismatch: {entry['path']}"
            )


def run_independent_verifier_in_process(
    verifier: Path,
    run_dir: Path,
    package: Path,
    sidecar: Path,
    source_root: Path,
) -> Path:
    if not sys.dont_write_bytecode:
        raise MatrixError(
            "aggregation replay requires Python -B for frozen verification"
        )
    result_path = package.parent / "G8C_MATRIX_VERIFICATION.json"
    argv = verifier_command(
        verifier, run_dir, package, sidecar, result_path, source_root
    )
    saved_argv = sys.argv
    try:
        sys.argv = argv[2:]
        try:
            runpy.run_path(str(verifier), run_name="__main__")
        except SystemExit as error:
            if error.code not in (None, 0):
                raise MatrixError(
                    f"in-process independent verifier failed with exit code {error.code}"
                ) from error
    finally:
        sys.argv = saved_argv
    if not result_path.is_file():
        raise MatrixError(
            "in-process independent verifier did not publish its sibling result"
        )
    return result_path


def aggregation_replay_run_id(input_digest: str) -> str:
    timestamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    return f"g8c-aggregation-replay-{timestamp}-{input_digest[:12]}"


def run_aggregation_replay(
    source_root: Path,
    artifact_root: Path,
    source_pilot: Path,
) -> dict[str, Any]:
    source_root = source_root.resolve()
    artifact_root = artifact_root.resolve()
    source_pilot = source_pilot.resolve()
    source_pilot_id = source_pilot.name
    if (
        source_pilot_id != REPLAY_SOURCE_PILOT_ID
        or source_pilot != REPLAY_SOURCE_PILOT_PATH.resolve()
    ):
        raise MatrixError(
            "aggregation replay requires the exact approved replacement pilot: "
            f"observed {source_pilot}, expected {REPLAY_SOURCE_PILOT_PATH.resolve()}"
        )
    forbidden_publication = [
        name
        for name in (
            "G8C_MATRIX_MANIFEST.json",
            "G8C_MATRIX_RECEIPT.json",
            "HASHES.sha256",
        )
        if (source_pilot / name).exists()
    ]
    if forbidden_publication:
        raise MatrixError(
            "aggregation replay requires incomplete pre-publication pilot inputs; "
            f"found {forbidden_publication!r}"
        )

    pre_entries, pre_digest, pre_total_bytes = directory_byte_inventory(source_pilot)
    run_id = aggregation_replay_run_id(pre_digest)
    run_dir = create_run_directory(artifact_root, "pilot", run_id)
    coordinator_source = Path(__file__).resolve()
    verifier_source = source_root / "tools" / "verify_g8c_matrix.py"
    if not verifier_source.is_file():
        raise MatrixError(f"independent verifier source is missing: {verifier_source}")
    frozen_coordinator = run_dir / "verification" / "frozen-coordinator.py"
    frozen_verifier = run_dir / "verification" / "frozen-verifier.py"
    copy_new(coordinator_source, frozen_coordinator)
    copy_new(verifier_source, frozen_verifier)
    replay_implementation = {
        "coordinator": {
            "path": safe_relative(frozen_coordinator, run_dir),
            "size": frozen_coordinator.stat().st_size,
            "sha256": sha256_file(frozen_coordinator),
        },
        "verifier": {
            "path": safe_relative(frozen_verifier, run_dir),
            "size": frozen_verifier.stat().st_size,
            "sha256": sha256_file(frozen_verifier),
        },
    }
    inputs_root = run_dir / "source-pilot"
    copy_inventory_new(source_pilot, inputs_root, pre_entries)

    source_manifest = read_json(inputs_root / "SOURCE_INPUT_MANIFEST.json")
    if (
        source_manifest.get("schema_version") != SOURCE_SCHEMA
        or source_manifest.get("matrix_run_id") != source_pilot_id
        or source_manifest.get("run_mode") != "pilot"
    ):
        raise MatrixError("source pilot input manifest identity is invalid")
    source_state = source_manifest.get("source")
    source_digest = source_manifest.get("source_input_digest")
    if not isinstance(source_state, dict) or not isinstance(source_digest, str):
        raise MatrixError("source pilot source identity is incomplete")

    first_headless_manifest = read_json(
        inputs_root / "scenarios" / SCENARIOS[0] / "HEADLESS_MANIFEST.json"
    )
    profile = first_headless_manifest.get("common_config")
    if not isinstance(profile, dict) or profile != matrix_profile("pilot"):
        raise MatrixError(
            "source pilot common config is not the canonical pilot profile"
        )
    validate_scenario_sequence(SCENARIOS)

    binaries: dict[str, dict[str, Any]] = {}
    for role, filename in (
        ("benchmark", "powdergame-benchmark.exe"),
        ("windows", "powdergame-windows.exe"),
    ):
        path = inputs_root / "frozen-binary" / filename
        if not path.is_file():
            raise MatrixError(f"source pilot frozen {role} binary is missing")
        binaries[role] = {
            "path": safe_relative(path, run_dir),
            "size": path.stat().st_size,
            "sha256": sha256_file(path),
            "build_profile": "release",
        }

    capture_process_paths = [
        safe_relative(inputs_root / "build" / "COMMAND.json", inputs_root),
        *[
            safe_relative(path, inputs_root)
            for path in sorted((inputs_root / "process").glob("*.json"))
        ],
    ]
    if len(capture_process_paths) != 16:
        raise MatrixError(
            "source pilot must contain one build and fifteen measurement process records"
        )
    for relative in capture_process_paths:
        record = read_json(inputs_root / relative)
        if (
            record.get("schema_version") != PROCESS_SCHEMA
            or record.get("exit_code") != 0
        ):
            raise MatrixError(
                f"source pilot process did not exit successfully: {relative}"
            )

    rows: list[dict[str, Any]] = []
    scenario_records: list[dict[str, Any]] = []
    matrix_adapter: dict[str, Any] | None = None
    for scenario in SCENARIOS:
        paths = headless_paths(inputs_root, scenario)
        headless_manifest = read_json(paths["manifest"])
        if (
            headless_manifest.get("matrix_run_id") != source_pilot_id
            or headless_manifest.get("scenario") != scenario
            or headless_manifest.get("common_config") != profile
        ):
            raise MatrixError(f"source pilot headless manifest mismatch for {scenario}")
        headless = aggregate_headless(paths, profile, scenario)

        coexistence_csv = (
            inputs_root / "raw" / "coexistence" / scenario / "mode-c-coexistence.csv"
        )
        coexistence_metadata_path = coexistence_csv.with_suffix(".json")
        validate_csv_identity(coexistence_csv, COEXISTENCE_SCHEMA, scenario)
        coexistence_metadata = validate_worker_metadata(
            coexistence_metadata_path,
            schema=COEXISTENCE_SCHEMA,
            mode="coexistence",
            scenario=scenario,
            run_id=source_pilot_id,
            source_sha=str(source_state["sha"]),
            source_git_state=str(source_state["git_state"]),
            binary_sha256=binaries["windows"]["sha256"],
            profile=profile,
            recorded_raw_csv_path=(
                source_pilot
                / "raw"
                / "coexistence"
                / scenario
                / "mode-c-coexistence.csv"
            ),
        )
        coexistence = aggregate_coexistence(
            coexistence_csv, profile, coexistence_metadata
        )

        render_csv = (
            inputs_root
            / "raw"
            / "render-profile"
            / scenario
            / "mode-d-render-profile.csv"
        )
        render_metadata_path = render_csv.with_suffix(".json")
        validate_csv_identity(render_csv, RENDER_PROFILE_SCHEMA, scenario)
        render_metadata = validate_worker_metadata(
            render_metadata_path,
            schema=RENDER_PROFILE_SCHEMA,
            mode="render-profile",
            scenario=scenario,
            run_id=source_pilot_id,
            source_sha=str(source_state["sha"]),
            source_git_state=str(source_state["git_state"]),
            binary_sha256=binaries["windows"]["sha256"],
            profile=profile,
            recorded_raw_csv_path=(
                source_pilot
                / "raw"
                / "render-profile"
                / scenario
                / "mode-d-render-profile.csv"
            ),
        )
        render = aggregate_render_profile(render_csv, profile, render_metadata)

        expected_adapter = headless["adapter"]
        if matrix_adapter is None:
            matrix_adapter = dict(expected_adapter)
        elif expected_adapter != matrix_adapter:
            raise MatrixError(f"headless adapter changed at replay scenario {scenario}")
        for label, metadata in (
            ("Mode C", coexistence_metadata),
            ("Mode D", render_metadata),
        ):
            observed = metadata["adapter"]
            if (
                observed.get("name") != expected_adapter["name"]
                or observed.get("vendor") != int(expected_adapter["vendor_id"], 16)
                or observed.get("device") != int(expected_adapter["device_id"], 16)
                or str(observed.get("backend", "")).lower()
                != expected_adapter["backend"].lower()
            ):
                raise MatrixError(
                    f"{scenario} {label} adapter differs from headless during replay"
                )
        rows.append(
            scenario_matrix_row(
                scenario, str(source_state["sha"]), headless, coexistence, render
            )
        )
        scenario_records.append(
            {
                "scenario": scenario,
                "headless_manifest": safe_relative(paths["manifest"], run_dir),
                "headless_summary": safe_relative(paths["summary"], run_dir),
                "raw_ticks": safe_relative(paths["raw_ticks"], run_dir),
                "raw_cells": safe_relative(paths["raw_cells"], run_dir),
                "raw_chunks": safe_relative(paths["raw_chunks"], run_dir),
                "coexistence_csv": safe_relative(coexistence_csv, run_dir),
                "coexistence_metadata": safe_relative(
                    coexistence_metadata_path, run_dir
                ),
                "render_profile_csv": safe_relative(render_csv, run_dir),
                "render_profile_metadata": safe_relative(render_metadata_path, run_dir),
            }
        )

    recommendation = "NEEDS_HUMAN_REVIEW"
    reasons = [
        "aggregation replay reuses non-evidence pilot measurements for parser validation only"
    ]
    for row in rows:
        row["total_recommendation_flag"] = recommendation
    reports = write_reports(
        run_dir, run_id, "aggregation-replay", rows, recommendation, reasons
    )

    if (
        sha256_file(coordinator_source)
        != replay_implementation["coordinator"]["sha256"]
        or sha256_file(verifier_source) != replay_implementation["verifier"]["sha256"]
    ):
        raise MatrixError("aggregation replay implementation changed during execution")
    delivery = run_dir.parent / f"{run_id}-delivery"
    expected_package = delivery / "G8C_MATRIX_PACKAGE.zip"
    expected_sidecar = delivery / "G8C_MATRIX_PACKAGE_SHA256.txt"
    expected_verification = delivery / "G8C_MATRIX_VERIFICATION.json"
    verifier_record = {
        "path": safe_relative(frozen_verifier, run_dir),
        "size": frozen_verifier.stat().st_size,
        "sha256": sha256_file(frozen_verifier),
        "expected_argv": verifier_command(
            frozen_verifier,
            run_dir,
            expected_package,
            expected_sidecar,
            expected_verification,
            source_root,
        ),
        "execution_timing": "after receipt and package; result is delivery sibling and does not mutate matrix run",
    }

    post_entries, post_digest, post_total_bytes = directory_byte_inventory(source_pilot)
    if (
        post_entries != pre_entries
        or post_digest != pre_digest
        or post_total_bytes != pre_total_bytes
    ):
        raise MatrixError("source pilot inputs changed during aggregation replay")
    inventory_value = {
        "schema_version": REPLAY_INPUT_SCHEMA,
        "replay_run_id": run_id,
        "source_pilot_id": source_pilot_id,
        "source_pilot_path": str(source_pilot),
        "inputs_root": "source-pilot",
        "pre_replay_digest": pre_digest,
        "post_aggregation_digest": post_digest,
        "unchanged": True,
        "entry_count": len(pre_entries),
        "total_bytes": pre_total_bytes,
        "entries": [
            {
                **entry,
                "replay_path": f"source-pilot/{entry['path']}",
            }
            for entry in pre_entries
        ],
    }
    inventory_path = run_dir / "SOURCE_PILOT_INPUT_MANIFEST.json"
    write_new_json(inventory_path, inventory_value)
    replay_contract = {
        "source_pilot_id": source_pilot_id,
        "source_pilot_path": str(source_pilot),
        "source_pilot_inventory_path": safe_relative(inventory_path, run_dir),
        "source_pilot_inventory_sha256": sha256_file(inventory_path),
        "source_pilot_inventory_digest": pre_digest,
        "source_pilot_file_count": len(pre_entries),
        "source_pilot_total_bytes": pre_total_bytes,
        "inputs_root": "source-pilot",
        "source_pilot_command_record_paths": capture_process_paths,
        "replay_implementation": replay_implementation,
        "non_evidence": True,
        "gpu_measurement_reused_for_parser_validation": True,
        "measurement_subprocess_count": 0,
        "executable_invocation_count": 0,
        "gpu_context_count": 0,
        "launched_process_count": 0,
    }
    manifest = {
        "schema_version": MATRIX_SCHEMA,
        "matrix_run_id": run_id,
        "run_mode": "aggregation-replay",
        "official_evidence": False,
        "pilot_must_never_be_promoted": True,
        "aggregation_replay": replay_contract,
        "source": {
            **source_state,
            "input_digest": source_digest,
            "input_manifest": "source-pilot/SOURCE_INPUT_MANIFEST.json",
            "exact_input_archive": "source-pilot/SOURCE_INPUT_BYTES.zip",
            "canonical_git_archive": "source-pilot/GIT_SOURCE_ARCHIVE.zip",
        },
        "common_config": profile,
        "hardware_policy": {
            "adapter": "NVIDIA RTX 5090",
            "vendor_id": "0x10DE",
            "backend": "Dx12",
            "tracked_memory_capacity_bytes": 32 * 1024**3,
            "tracked_memory_note": "application-tracked persistent GPU bytes, not total driver-resident VRAM",
        },
        "scenario_order": list(SCENARIOS),
        "frozen_binaries": binaries,
        "build_command_record": None,
        "independent_verifier": verifier_record,
        "command_record_paths": [],
        "scenarios": scenario_records,
        "reports": reports,
        "estimated_unpacked_bytes_before_capture": pre_total_bytes,
        "recommendation": recommendation,
    }
    write_new_json(run_dir / "G8C_MATRIX_MANIFEST.json", manifest)
    hashes_path, hash_entries = write_hash_inventory(run_dir)
    receipt = write_receipt(
        run_dir,
        run_id,
        "aggregation-replay",
        source_state,
        str(source_digest),
        binaries,
        reports,
        verifier_record,
        recommendation,
        hash_entries,
    )
    package, sidecar, package_hash = create_package(run_dir)
    verification = run_independent_verifier_in_process(
        frozen_verifier, run_dir, package, sidecar, source_root
    )
    return {
        "run_id": run_id,
        "run_dir": str(run_dir),
        "run_mode": "aggregation-replay",
        "source_pilot_id": source_pilot_id,
        "measurement_subprocess_count": 0,
        "executable_invocation_count": 0,
        "gpu_context_count": 0,
        "launched_process_count": 0,
        "receipt": str(receipt),
        "receipt_sha256": sha256_file(receipt),
        "hashes": str(hashes_path),
        "package": str(package),
        "package_sha256": package_hash,
        "sidecar": str(sidecar),
        "verification": str(verification),
        "verification_sha256": sha256_file(verification),
        "source_pilot_input_digest": pre_digest,
        "source_pilot_unchanged": True,
        "recommendation": recommendation,
    }


def run_matrix(source_root: Path, artifact_root: Path, mode: str) -> dict[str, Any]:
    source_root = source_root.resolve()
    artifact_root = artifact_root.resolve()
    try:
        artifact_root.relative_to(source_root)
    except ValueError:
        pass
    else:
        raise MatrixError("artifact root must be external to the source worktree")
    profile = matrix_profile(mode)
    validate_scenario_sequence(SCENARIOS)
    source_state = capture_source_state(source_root, mode)
    entries, source_digest = collect_source_entries(source_root)
    run_id = run_id_for(mode, source_state["sha"], source_digest)
    estimate = estimate_output_bytes(profile)
    print(
        f"Estimated unpacked output before capture: {estimate} bytes "
        f"({estimate / (1024**3):.2f} GiB)"
    )
    run_dir = create_run_directory(artifact_root, mode, run_id)
    source_manifest = {
        "schema_version": SOURCE_SCHEMA,
        "matrix_run_id": run_id,
        "run_mode": mode,
        "source": source_state,
        "source_input_digest": source_digest,
        "entry_count": len(entries),
        "entries": entries,
        "roles": {
            "SOURCE_INPUT_BYTES.zip": "exact working-tree build-input bytes plus external Consolas input",
            "GIT_SOURCE_ARCHIVE.zip": (
                "canonical Git archive for source SHA; pilot tracked changes intentionally differ"
            ),
        },
    }
    source_manifest_path = run_dir / "SOURCE_INPUT_MANIFEST.json"
    write_new_json(source_manifest_path, source_manifest)
    write_source_archive(run_dir / "SOURCE_INPUT_BYTES.zip", source_root, entries)
    verify_source_archive(run_dir / "SOURCE_INPUT_BYTES.zip", entries)
    write_git_archive(
        run_dir / "GIT_SOURCE_ARCHIVE.zip", source_root, source_state["sha"]
    )
    assert_source_unchanged(source_root, source_state, entries, "pre-build")

    binaries, build_record_path = build_and_freeze(
        source_root, artifact_root, run_dir, run_id
    )
    assert_source_unchanged(source_root, source_state, entries, "post-build")
    frozen_benchmark = run_dir / binaries["benchmark"]["path"]
    frozen_windows = run_dir / binaries["windows"]["path"]
    process_records: list[str] = [build_record_path]
    scenario_records: list[dict[str, Any]] = []
    scenario_runtime: dict[str, dict[str, Any]] = {}
    sequence = 0

    for scenario in SCENARIOS:
        sequence += 1
        paths = headless_paths(run_dir, scenario)
        paths["summary"].parent.mkdir(parents=True, exist_ok=True)
        prefix = run_dir / "process" / f"{sequence:02d}-{scenario}-headless"
        record_path = prefix.with_suffix(".json")
        run_logged(
            benchmark_command(frozen_benchmark, scenario, profile, paths["summary"]),
            cwd=source_root,
            stdout_path=prefix.with_suffix(".stdout.log"),
            stderr_path=prefix.with_suffix(".stderr.log"),
            record_path=record_path,
            role="headless-mode-a-b",
            scenario=scenario,
            run_root=run_dir,
            expected_outputs=[
                paths[key]
                for key in ("summary", "raw_ticks", "raw_cells", "raw_chunks")
            ],
        )
        process_records.append(safe_relative(record_path, run_dir))
        headless_manifest = write_headless_manifest(
            run_dir,
            scenario,
            paths,
            profile,
            run_id,
            source_state,
            binaries["benchmark"],
        )
        scenario_runtime[scenario] = {
            "headless_paths": paths,
            "headless_manifest": headless_manifest,
        }

    for scenario in SCENARIOS:
        sequence += 1
        directory = run_dir / "raw" / "coexistence" / scenario
        directory.mkdir(parents=True, exist_ok=True)
        raw_csv = directory / "mode-c-coexistence.csv"
        metadata_json = directory / "mode-c-coexistence.json"
        prefix = run_dir / "process" / f"{sequence:02d}-{scenario}-coexistence"
        record_path = prefix.with_suffix(".json")
        run_logged(
            windows_worker_command(
                frozen_windows,
                "coexistence",
                scenario,
                profile,
                run_id,
                binaries["windows"]["sha256"],
                raw_csv,
                metadata_json,
            ),
            cwd=source_root,
            stdout_path=prefix.with_suffix(".stdout.log"),
            stderr_path=prefix.with_suffix(".stderr.log"),
            record_path=record_path,
            role="windowed-production-coexistence",
            scenario=scenario,
            run_root=run_dir,
            expected_outputs=[raw_csv, metadata_json],
        )
        process_records.append(safe_relative(record_path, run_dir))
        validate_csv_identity(raw_csv, COEXISTENCE_SCHEMA, scenario)
        mode_c_metadata = validate_worker_metadata(
            metadata_json,
            schema=COEXISTENCE_SCHEMA,
            mode="coexistence",
            scenario=scenario,
            run_id=run_id,
            source_sha=source_state["sha"],
            source_git_state=source_state["git_state"],
            binary_sha256=binaries["windows"]["sha256"],
            profile=profile,
        )
        scenario_runtime[scenario].update(
            {
                "coexistence_csv": raw_csv,
                "coexistence_metadata_path": metadata_json,
                "coexistence_metadata": mode_c_metadata,
            }
        )

    for scenario in SCENARIOS:
        sequence += 1
        directory = run_dir / "raw" / "render-profile" / scenario
        directory.mkdir(parents=True, exist_ok=True)
        raw_csv = directory / "mode-d-render-profile.csv"
        metadata_json = directory / "mode-d-render-profile.json"
        prefix = run_dir / "process" / f"{sequence:02d}-{scenario}-render-profile"
        record_path = prefix.with_suffix(".json")
        run_logged(
            windows_worker_command(
                frozen_windows,
                "render-profile",
                scenario,
                profile,
                run_id,
                binaries["windows"]["sha256"],
                raw_csv,
                metadata_json,
            ),
            cwd=source_root,
            stdout_path=prefix.with_suffix(".stdout.log"),
            stderr_path=prefix.with_suffix(".stderr.log"),
            record_path=record_path,
            role="windowed-gpu-render-timing",
            scenario=scenario,
            run_root=run_dir,
            expected_outputs=[raw_csv, metadata_json],
        )
        process_records.append(safe_relative(record_path, run_dir))
        validate_csv_identity(raw_csv, RENDER_PROFILE_SCHEMA, scenario)
        mode_d_metadata = validate_worker_metadata(
            metadata_json,
            schema=RENDER_PROFILE_SCHEMA,
            mode="render-profile",
            scenario=scenario,
            run_id=run_id,
            source_sha=source_state["sha"],
            source_git_state=source_state["git_state"],
            binary_sha256=binaries["windows"]["sha256"],
            profile=profile,
        )
        scenario_runtime[scenario].update(
            {
                "render_csv": raw_csv,
                "render_metadata_path": metadata_json,
                "render_metadata": mode_d_metadata,
            }
        )

    rows: list[dict[str, Any]] = []
    matrix_adapter: dict[str, Any] | None = None
    for scenario in SCENARIOS:
        runtime = scenario_runtime[scenario]
        headless = aggregate_headless(runtime["headless_paths"], profile, scenario)
        coexistence = aggregate_coexistence(
            runtime["coexistence_csv"], profile, runtime["coexistence_metadata"]
        )
        render = aggregate_render_profile(
            runtime["render_csv"], profile, runtime["render_metadata"]
        )
        expected_adapter = headless["adapter"]
        if matrix_adapter is None:
            matrix_adapter = dict(expected_adapter)
        elif expected_adapter != matrix_adapter:
            raise MatrixError(
                f"headless adapter identity changed across scenarios: "
                f"baseline={matrix_adapter}, {scenario}={expected_adapter}"
            )
        for label, metadata in (
            ("Mode C", runtime["coexistence_metadata"]),
            ("Mode D", runtime["render_metadata"]),
        ):
            observed_adapter = metadata["adapter"]
            if (
                observed_adapter.get("name") != expected_adapter["name"]
                or observed_adapter.get("vendor")
                != int(expected_adapter["vendor_id"], 16)
                or observed_adapter.get("device")
                != int(expected_adapter["device_id"], 16)
                or str(observed_adapter.get("backend", "")).lower()
                != expected_adapter["backend"].lower()
            ):
                raise MatrixError(
                    f"{scenario} {label} adapter does not match the headless adapter: "
                    f"headless={expected_adapter}, windowed={observed_adapter}"
                )
        rows.append(
            scenario_matrix_row(
                scenario, source_state["sha"], headless, coexistence, render
            )
        )
        paths = runtime["headless_paths"]
        scenario_records.append(
            {
                "scenario": scenario,
                "headless_manifest": safe_relative(paths["manifest"], run_dir),
                "headless_summary": safe_relative(paths["summary"], run_dir),
                "raw_ticks": safe_relative(paths["raw_ticks"], run_dir),
                "raw_cells": safe_relative(paths["raw_cells"], run_dir),
                "raw_chunks": safe_relative(paths["raw_chunks"], run_dir),
                "coexistence_csv": safe_relative(runtime["coexistence_csv"], run_dir),
                "coexistence_metadata": safe_relative(
                    runtime["coexistence_metadata_path"], run_dir
                ),
                "render_profile_csv": safe_relative(runtime["render_csv"], run_dir),
                "render_profile_metadata": safe_relative(
                    runtime["render_metadata_path"], run_dir
                ),
            }
        )
    if mode == "pilot":
        recommendation, reasons = (
            "NEEDS_HUMAN_REVIEW",
            [
                "non-evidence pilot validates orchestration only and must never be used for a G9 decision"
            ],
        )
    else:
        recommendation, reasons = optimization_recommendation(rows)
    for row in rows:
        row["total_recommendation_flag"] = recommendation
    reports = write_reports(run_dir, run_id, mode, rows, recommendation, reasons)
    verifier_source = source_root / "tools" / "verify_g8c_matrix.py"
    if not verifier_source.is_file():
        raise MatrixError(f"independent verifier source is missing: {verifier_source}")
    frozen_verifier = run_dir / "verification" / "frozen-verifier.py"
    copy_new(verifier_source, frozen_verifier)
    delivery_directory = run_dir.parent / f"{run_id}-delivery"
    expected_package = delivery_directory / "G8C_MATRIX_PACKAGE.zip"
    expected_sidecar = delivery_directory / "G8C_MATRIX_PACKAGE_SHA256.txt"
    expected_verification = delivery_directory / "G8C_MATRIX_VERIFICATION.json"
    verifier_record = {
        "path": safe_relative(frozen_verifier, run_dir),
        "size": frozen_verifier.stat().st_size,
        "sha256": sha256_file(frozen_verifier),
        "expected_argv": verifier_command(
            frozen_verifier,
            run_dir,
            expected_package,
            expected_sidecar,
            expected_verification,
            source_root,
        ),
        "execution_timing": "after receipt and package; result is delivery sibling and does not mutate matrix run",
    }
    manifest = {
        "schema_version": MATRIX_SCHEMA,
        "matrix_run_id": run_id,
        "run_mode": mode,
        "official_evidence": mode == "official",
        "pilot_must_never_be_promoted": mode == "pilot",
        "source": {
            **source_state,
            "input_digest": source_digest,
            "input_manifest": safe_relative(source_manifest_path, run_dir),
            "exact_input_archive": "SOURCE_INPUT_BYTES.zip",
            "canonical_git_archive": "GIT_SOURCE_ARCHIVE.zip",
        },
        "common_config": profile,
        "hardware_policy": {
            "adapter": "NVIDIA RTX 5090",
            "vendor_id": "0x10DE",
            "backend": "Dx12",
            "tracked_memory_capacity_bytes": 32 * 1024**3,
            "tracked_memory_note": "application-tracked persistent GPU bytes, not total driver-resident VRAM",
        },
        "scenario_order": list(SCENARIOS),
        "frozen_binaries": binaries,
        "build_command_record": build_record_path,
        "independent_verifier": verifier_record,
        "command_record_paths": process_records,
        "scenarios": scenario_records,
        "reports": reports,
        "estimated_unpacked_bytes_before_capture": estimate,
        "recommendation": recommendation,
    }
    write_new_json(run_dir / "G8C_MATRIX_MANIFEST.json", manifest)
    assert_source_unchanged(source_root, source_state, entries, "pre-receipt")
    for role, record in binaries.items():
        frozen_path = run_dir / record["path"]
        if (
            not frozen_path.is_file()
            or frozen_path.stat().st_size != record["size"]
            or sha256_file(frozen_path) != record["sha256"]
        ):
            raise MatrixError(f"frozen {role} binary changed before receipt")
    if sha256_file(frozen_verifier) != verifier_record["sha256"]:
        raise MatrixError("frozen independent verifier changed before receipt")
    hashes_path, hash_entries = write_hash_inventory(run_dir)
    assert_source_unchanged(source_root, source_state, entries, "receipt-publication")
    receipt = write_receipt(
        run_dir,
        run_id,
        mode,
        source_state,
        source_digest,
        binaries,
        reports,
        verifier_record,
        recommendation,
        hash_entries,
    )
    package, sidecar, package_hash = create_package(run_dir)
    verification = run_independent_verifier(
        frozen_verifier, run_dir, package, sidecar, source_root
    )
    matrix_size = sum(
        path.stat().st_size for path in run_dir.rglob("*") if path.is_file()
    )
    return {
        "run_id": run_id,
        "run_dir": str(run_dir),
        "receipt": str(receipt),
        "receipt_sha256": sha256_file(receipt),
        "hashes": str(hashes_path),
        "package": str(package),
        "package_sha256": package_hash,
        "package_size": package.stat().st_size,
        "matrix_unpacked_size": matrix_size,
        "sidecar": str(sidecar),
        "verification": str(verification),
        "verification_sha256": sha256_file(verification),
        "frozen_binary_sha256": {
            role: record["sha256"] for role, record in binaries.items()
        },
        "recommendation": recommendation,
    }


def default_source_root() -> Path:
    return Path(__file__).resolve().parents[1]


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Capture one immutable Powdergame G8-C pilot or official matrix."
    )
    parser.add_argument("mode", choices=("pilot", "official", "aggregation-replay"))
    parser.add_argument("--source-root", type=Path, default=default_source_root())
    parser.add_argument("--source-pilot", type=Path)
    parser.add_argument(
        "--artifact-root",
        type=Path,
        default=Path(r"C:\Users\mdkap\source\Powdergame-artifacts"),
    )
    arguments = parser.parse_args(argv)
    if arguments.mode == "aggregation-replay" and arguments.source_pilot is None:
        parser.error("aggregation-replay requires --source-pilot")
    if arguments.mode != "aggregation-replay" and arguments.source_pilot is not None:
        parser.error("--source-pilot is only valid for aggregation-replay")
    return arguments


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        if args.mode == "aggregation-replay":
            result = run_aggregation_replay(
                args.source_root, args.artifact_root, args.source_pilot
            )
        else:
            result = run_matrix(args.source_root, args.artifact_root, args.mode)
    except MatrixError as error:
        print(f"FATAL: {error}", file=sys.stderr)
        return 1
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
