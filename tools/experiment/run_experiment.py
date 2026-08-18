#!/usr/bin/env python3
"""Create one immutable external G8-B scenario experiment packet.

Operational failures intentionally leave the unique run directory in place
without EXPERIMENT_RECEIPT.json. A failed run is never repaired or reused.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import math
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
from typing import Any, Callable, Iterable, Sequence


DEFAULT_ARTIFACT_ROOT = Path(r"C:\Users\mdkap\source\Powdergame-artifacts")

SOURCE_INPUT_MANIFEST_SCHEMA = "powdergame-source-input-manifest-v0"
SOURCE_INPUT_MANIFEST_NAME = "SOURCE_INPUT_MANIFEST.json"
FROZEN_BINARY_RELATIVE_PATH = PurePosixPath(
    "frozen-binary/powdergame-windows.exe"
)
AUDIT_BUNDLE_SUFFIX = ".AUDIT_BUNDLE.zip"
AUDIT_BUNDLE_SHA256_SUFFIX = ".AUDIT_BUNDLE_SHA256.txt"
PRESSURE_AUDIT_BUNDLE_MANIFEST_SCHEMA = (
    "powdergame-pressure-burst-audit-bundle-manifest-v1"
)
SOURCE_INPUT_EXACT_PATHS = frozenset(
    {
        "run_experiment.bat",
        "tools/experiment/run_experiment.py",
    }
)
SOURCE_EXTERNAL_BUILD_INPUTS = (
    ("windows-consolas-font", Path(r"C:\Windows\Fonts\consola.ttf")),
)

SAND_MANIFEST_SCHEMA = "powdergame-experiment-manifest-v0"
SAND_ANALYSIS_SCHEMA = "powdergame-experiment-analysis-v0"
FRAMES_SCHEMA = "powdergame-experiment-frames-v0"
SAND_TELEMETRY_SCHEMA = "powdergame-experiment-telemetry-v0"
SAND_REPORT_SCHEMA = "powdergame-experiment-report-v0"
SAND_RECEIPT_SCHEMA = "powdergame-experiment-receipt-v0"

WATER_MANIFEST_SCHEMA = "powdergame-experiment-manifest-v1"
WATER_ANALYSIS_SCHEMA = "powdergame-experiment-analysis-v2"
WATER_TELEMETRY_SCHEMA = "powdergame-experiment-telemetry-v2"
WATER_REPORT_SCHEMA = "powdergame-experiment-report-v2"
WATER_RECEIPT_SCHEMA = "powdergame-experiment-receipt-v2"

FIRE_MANIFEST_SCHEMA = "powdergame-fire-heat-manifest-v0"
FIRE_ANALYSIS_SCHEMA = "powdergame-fire-heat-analysis-v0"
FIRE_TELEMETRY_SCHEMA = "powdergame-fire-heat-telemetry-v0"
FIRE_REPORT_SCHEMA = "powdergame-fire-heat-report-v0"
FIRE_RECEIPT_SCHEMA = "powdergame-fire-heat-receipt-v0"

PRESSURE_MANIFEST_SCHEMA = "powdergame-pressure-burst-manifest-v0"
PRESSURE_ANALYSIS_SCHEMA = "powdergame-pressure-burst-analysis-v1"
PRESSURE_FRAMES_SCHEMA = "powdergame-pressure-burst-frames-v0"
PRESSURE_TELEMETRY_SCHEMA = "powdergame-pressure-burst-telemetry-v1"
PRESSURE_REPORT_SCHEMA = "powdergame-pressure-burst-report-v1"
PRESSURE_RECEIPT_SCHEMA = "powdergame-pressure-burst-receipt-v1"

WORLD_WIDTH = 256
WORLD_HEIGHT = 256
CHUNK_SIZE = 64
MAX_TICKS = 20_000
DIAGNOSTIC_INTERVAL = 8
CONSECUTIVE_ALL_SLEEP = 3
CONSECUTIVE_STABLE_PLATEAU = 8
POST_SLEEP_TICKS = 180
CONSECUTIVE_REACTION_ZERO = 3
POST_REACTION_TICKS = 180
CONSECUTIVE_PERSISTENT_OPENING = 3
POST_OPENING_TICKS = 180
TERMINAL_WINDOW_SAMPLES = 64
WOOD_RUPTURE_THRESHOLD = 80.0

RENDERER_WIDTH = 1_600
RENDERER_HEIGHT = 900
CROP_X = 420
CROP_Y = 60
CROP_WIDTH = 760
CROP_HEIGHT = 760

SAND_ALLOWED_VERDICTS = frozenset({"PASS", "FAIL", "NEEDS_HUMAN"})
SAND_PREDICATE_NAMES = frozenset({
    "actual_fall",
    "matter_conservation",
    "no_invalid_materials",
    "no_nonfinite_fields",
    "sleep_before_max",
    "post_sleep_stable",
    "exact_reset",
})
WATER_ALLOWED_VERDICTS = frozenset({"PASS", "FAIL", "NEEDS_HUMAN_REVIEW"})
WATER_PREDICATE_NAMES = frozenset({
    "actual_water_movement",
    "cross_chunk_flow",
    "destination_arrival",
    "water_conservation",
    "no_invalid_materials",
    "no_nonfinite_fields",
    "stable_bulk_before_max",
    "post_settle_stable",
    "exact_reset",
    "water_outside_outer_basin_cells",
})
FIRE_ALLOWED_VERDICTS = frozenset({"PASS", "FAIL", "NEEDS_HUMAN_REVIEW"})
FIRE_PREDICATE_NAMES = frozenset({
    "combustion_observed",
    "smoke_generated",
    "heat_propagated",
    "phase_work_observed",
    "fuel_consumed",
    "reaction_terminated_before_max",
    "post_reaction_no_restart",
    "thermal_tail_observed",
    "thermal_tail_decreased",
    "no_invalid_materials",
    "no_nonfinite_fields",
    "exact_reset",
})
PRESSURE_FIXTURE_CAUSALITY_CONFOUNDED_VERDICT = "FIXTURE_CAUSALITY_CONFOUNDED"
PRESSURE_ALLOWED_VERDICTS = frozenset({
    "PASS",
    "FAIL",
    "NEEDS_HUMAN_REVIEW",
    PRESSURE_FIXTURE_CAUSALITY_CONFOUNDED_VERDICT,
})
PRESSURE_CAUSAL_CLASSIFICATIONS = frozenset({
    "pressure_opening_precedes_combustion",
    "fixture_causality_confounded",
    "insufficient_causal_evidence",
})
PRESSURE_PREDICATE_NAMES = frozenset({
    "pressure_activity_observed",
    "relief_seam_damaged",
    "persistent_opening_created",
    "pressure_opening_precedes_combustion",
    "exterior_vent_observed",
    "post_opening_pressure_relieved",
    "terminal_pressure_not_runaway",
    "no_invalid_materials",
    "no_nonfinite_fields",
    "exact_reset",
})
WATER_ACTIVE_CLASSIFICATION_RULE = (
    "cardinal-4-in-bounds;water-oil-first;water-empty-second;other-remainder"
)
WATER_PHASES = frozenset(
    {"initial", "flowing", "post-settle-confirmation", "reset"}
)
WATER_REASONS = frozenset(
    {
        "tick0",
        "tick1",
        "early-flow",
        "diagnostic-cadence",
        "max-tick",
        "post-settle-tick",
        "programmatic-r-equivalent",
    }
)
WATER_ALWAYS_EVENTS = frozenset(
    {
        "lifecycle_started",
        "pristine_reset_completed",
        "tick0_captured",
        "tick1_captured",
        "terminal_selected",
        "reset_started",
        "reset_comparison_completed",
        "worker_completed",
    }
)
WATER_OPTIONAL_EVENTS = frozenset(
    {
        "post_settle_confirmation_completed",
        "water_movement_observed",
        "cross_chunk_flow_observed",
        "destination_arrival_observed",
        "new_peak_active",
        "new_max_destination_spread",
        "first_sleeping_chunk_observed",
        "all_sleep_observed",
        "all_sleep_streak_broken",
        "all_sleep_confirmed",
        "stable_plateau_observed",
        "stable_plateau_streak_broken",
        "stable_plateau_confirmed",
    }
)
WATER_FRAME_KINDS = frozenset(
    {
        "tick0",
        "tick1",
        "first-movement",
        "peak-active",
        "cross-chunk-flow",
        "destination-arrival",
        "max-destination-spread",
        "first-sleeping-chunk",
        "late",
        "terminal",
        "post-settle",
        "reset",
        "diagnostic-observation",
    }
)
FIRE_FRAME_KINDS = frozenset(
    {
        "tick0",
        "tick1",
        "first-combustion",
        "first-smoke",
        "peak-reaction",
        "peak-thermal",
        "first-phase-transition",
        "fuel-substantially-consumed",
        "reaction-zero",
        "post-reaction-tail",
        "terminal",
        "reset",
        "diagnostic-observation",
    }
)
PRESSURE_FRAME_BADGE_KINDS = (
    "tick0",
    "tick1",
    "first-pressure-activity",
    "first-wood-damage",
    "first-rupture",
    "persistent-opening",
    "opening-reseal",
    "first-exterior-steam",
    "peak-pressure",
    "peak-pressure-activity",
    "post-opening",
    "terminal",
    "diagnostic-observation",
    "reset",
)
PRESSURE_FRAME_BADGE_RANK = {
    kind: rank for rank, kind in enumerate(PRESSURE_FRAME_BADGE_KINDS)
}
PRESSURE_PHASE_REASONS = {
    "initial": frozenset({"tick0"}),
    "pressurizing": frozenset(
        {"tick1", "early-diagnostic", "diagnostic-cadence", "max-tick"}
    ),
    "post-opening-observation": frozenset(
        {"post-opening-tick", "post-opening-observation-complete", "max-tick"}
    ),
    "reset": frozenset({"programmatic-r-equivalent"}),
}
PRESSURE_ALWAYS_EVENTS = frozenset(
    {
        "lifecycle_started",
        "pristine_reset_completed",
        "tick0_captured",
        "tick1_captured",
        "terminal_selected",
        "reset_started",
        "reset_comparison_completed",
        "worker_completed",
    }
)
PRESSURE_OPTIONAL_EVENTS = frozenset(
    {
        "pressure_activity_observed",
        "relief_seam_damage_observed",
        "relief_seam_combustion_observed",
        "relief_seam_fuel_progress_observed",
        "rupture_observed",
        "persistent_opening_streak_started",
        "persistent_opening_streak_broken",
        "persistent_opening_confirmed",
        "relief_seam_steam_observed",
        "exterior_vent_observed",
        "new_peak_chamber_mean_pressure",
        "new_peak_chamber_max_pressure",
        "new_peak_pressure_activity",
        "post_confirmation_reseal_observed",
        "post_opening_observation_started",
        "post_opening_pressure_relief_observed",
        "post_opening_observation_completed",
    }
)
PREDICATE_STATUSES = {"pass", "fail", "unknown"}
HEX64 = re.compile(r"^[0-9a-fA-F]{64}$")
GIT_OID = re.compile(r"^(?:[0-9a-fA-F]{40}|[0-9a-fA-F]{64})$")
STATE_HASH = re.compile(r"^fnv1a64:[0-9a-f]{16}$")

SAND_MANIFEST_TOP_KEYS = {
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
# Public Sand v0 alias retained for the existing focused fixture suite.
MANIFEST_TOP_KEYS = SAND_MANIFEST_TOP_KEYS
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
class ScenarioContract:
    scenario: str
    experiment_id: str
    manifest_schema: str
    telemetry_schema: str
    analysis_schema: str
    frames_schema: str
    report_schema: str
    receipt_schema: str
    predicate_names: frozenset[str]
    allowed_verdicts: frozenset[str]
    needs_human_verdict: str
    title: str
    records_run_mode: bool = False


SAND_CONTRACT = ScenarioContract(
    scenario="sand-fall",
    experiment_id="g8b-sand-fall-v0",
    manifest_schema=SAND_MANIFEST_SCHEMA,
    telemetry_schema=SAND_TELEMETRY_SCHEMA,
    analysis_schema=SAND_ANALYSIS_SCHEMA,
    frames_schema=FRAMES_SCHEMA,
    report_schema=SAND_REPORT_SCHEMA,
    receipt_schema=SAND_RECEIPT_SCHEMA,
    predicate_names=SAND_PREDICATE_NAMES,
    allowed_verdicts=SAND_ALLOWED_VERDICTS,
    needs_human_verdict="NEEDS_HUMAN",
    title="Sand Fall",
)
WATER_CONTRACT = ScenarioContract(
    scenario="water-flow",
    experiment_id="g8b-water-flow-v0",
    manifest_schema=WATER_MANIFEST_SCHEMA,
    telemetry_schema=WATER_TELEMETRY_SCHEMA,
    analysis_schema=WATER_ANALYSIS_SCHEMA,
    frames_schema=FRAMES_SCHEMA,
    report_schema=WATER_REPORT_SCHEMA,
    receipt_schema=WATER_RECEIPT_SCHEMA,
    predicate_names=WATER_PREDICATE_NAMES,
    allowed_verdicts=WATER_ALLOWED_VERDICTS,
    needs_human_verdict="NEEDS_HUMAN_REVIEW",
    title="Water Flow",
    records_run_mode=True,
)
FIRE_CONTRACT = ScenarioContract(
    scenario="fire-heat",
    experiment_id="g8b-fire-heat-v0",
    manifest_schema=FIRE_MANIFEST_SCHEMA,
    telemetry_schema=FIRE_TELEMETRY_SCHEMA,
    analysis_schema=FIRE_ANALYSIS_SCHEMA,
    frames_schema=FRAMES_SCHEMA,
    report_schema=FIRE_REPORT_SCHEMA,
    receipt_schema=FIRE_RECEIPT_SCHEMA,
    predicate_names=FIRE_PREDICATE_NAMES,
    allowed_verdicts=FIRE_ALLOWED_VERDICTS,
    needs_human_verdict="NEEDS_HUMAN_REVIEW",
    title="Fire / Heat",
    records_run_mode=True,
)
PRESSURE_CONTRACT = ScenarioContract(
    scenario="pressure-burst",
    experiment_id="g8b-pressure-burst-v0",
    manifest_schema=PRESSURE_MANIFEST_SCHEMA,
    telemetry_schema=PRESSURE_TELEMETRY_SCHEMA,
    analysis_schema=PRESSURE_ANALYSIS_SCHEMA,
    frames_schema=PRESSURE_FRAMES_SCHEMA,
    report_schema=PRESSURE_REPORT_SCHEMA,
    receipt_schema=PRESSURE_RECEIPT_SCHEMA,
    predicate_names=PRESSURE_PREDICATE_NAMES,
    allowed_verdicts=PRESSURE_ALLOWED_VERDICTS,
    needs_human_verdict="NEEDS_HUMAN_REVIEW",
    title="Pressure Burst",
    records_run_mode=True,
)
SCENARIO_CONTRACTS = {
    SAND_CONTRACT.scenario: SAND_CONTRACT,
    WATER_CONTRACT.scenario: WATER_CONTRACT,
    FIRE_CONTRACT.scenario: FIRE_CONTRACT,
    PRESSURE_CONTRACT.scenario: PRESSURE_CONTRACT,
}
RUN_MODES = frozenset({"candidate", "scratch"})
WATER_FINDING_CLASSIFICATIONS = (
    "actual_physics_defect",
    "fixture_representativeness_issue",
    "expected_local_movement_artifact",
    "presentation_or_capture_issue",
    "insufficient_evidence",
)
WATER_VISUAL_QUESTIONS = (
    "Does Water visibly leave the source volume and follow the intended channel?",
    "Is flow continuous across chunk boundaries without a visible seam or freeze?",
    "Does Water reach and spread through the destination basin?",
    "Does the Water/Oil arrangement remain visually plausible for the staged fixture?",
    "Do the late, terminal, and post-settle frames support the telemetry classification?",
    "Do HUD values and visible state agree with the joined sample caption?",
    "Is any Water visibly outside the intended outer basin boundary?",
    "Does final residual activity visually align with the reported interface classes?",
)

# Sand v0 compatibility aliases are intentionally retained for existing callers,
# fixtures, and the already-published Sand contract.
EXPERIMENT_ID = SAND_CONTRACT.experiment_id
SCENARIO = SAND_CONTRACT.scenario
MANIFEST_SCHEMA = SAND_CONTRACT.manifest_schema
ANALYSIS_SCHEMA = SAND_CONTRACT.analysis_schema
TELEMETRY_SCHEMA = SAND_CONTRACT.telemetry_schema
REPORT_SCHEMA = SAND_CONTRACT.report_schema
RECEIPT_SCHEMA = SAND_CONTRACT.receipt_schema
ALLOWED_VERDICTS = set(SAND_CONTRACT.allowed_verdicts)
PREDICATE_NAMES = set(SAND_CONTRACT.predicate_names)


def contract_for_scenario(scenario: str) -> ScenarioContract:
    try:
        return SCENARIO_CONTRACTS[scenario]
    except KeyError as error:
        expected = ", ".join(sorted(SCENARIO_CONTRACTS))
        raise ExperimentError(
            f"unsupported experiment scenario {scenario!r}; expected one of: {expected}"
        ) from error


def validate_run_mode(contract: ScenarioContract, run_mode: str) -> None:
    if run_mode not in RUN_MODES:
        raise ExperimentError(
            f"unsupported run mode {run_mode!r}; expected candidate or scratch"
        )
    if contract is SAND_CONTRACT and run_mode != "candidate":
        raise ExperimentError("Sand Fall v0 supports only candidate mode")


def contract_for_manifest(data: dict[str, Any]) -> ScenarioContract:
    scenario = data.get("scenario")
    if not isinstance(scenario, str):
        raise ExperimentError("manifest scenario must be a string")
    contract = contract_for_scenario(scenario)
    if data.get("experiment_id") != contract.experiment_id:
        raise ExperimentError("manifest experiment/scenario mismatch")
    return contract


@dataclass(frozen=True)
class SourceInfo:
    root: Path
    branch: str
    sha: str
    git_state: str = "clean"
    tracked_state_sha256: str = ""


@dataclass(frozen=True)
class SourceSeal:
    source: SourceInfo
    manifest: dict[str, Any]


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
    contract: ScenarioContract = SAND_CONTRACT
    run_mode: str = "candidate"

    def as_dict(self) -> dict[str, Any]:
        validate_run_mode(self.contract, self.run_mode)
        if self.contract is FIRE_CONTRACT:
            experiment = {
                "max_ticks": MAX_TICKS,
                "diagnostic_interval_ticks": DIAGNOSTIC_INTERVAL,
                "consecutive_reaction_zero": CONSECUTIVE_REACTION_ZERO,
                "post_reaction_ticks": POST_REACTION_TICKS,
            }
        elif self.contract is PRESSURE_CONTRACT:
            experiment = {
                "max_ticks": MAX_TICKS,
                "diagnostic_interval_ticks": DIAGNOSTIC_INTERVAL,
                "consecutive_persistent_opening": CONSECUTIVE_PERSISTENT_OPENING,
                "post_opening_ticks": POST_OPENING_TICKS,
                "terminal_window_samples": TERMINAL_WINDOW_SAMPLES,
            }
        else:
            experiment = {
                "max_ticks": MAX_TICKS,
                "diagnostic_interval_ticks": DIAGNOSTIC_INTERVAL,
                "consecutive_all_sleep": CONSECUTIVE_ALL_SLEEP,
                "post_sleep_ticks": POST_SLEEP_TICKS,
            }
        if self.contract is WATER_CONTRACT:
            experiment["stable_plateau_consecutive_samples"] = (
                CONSECUTIVE_STABLE_PLATEAU
            )
        data = {
            "schema_version": self.contract.manifest_schema,
            "experiment_id": self.contract.experiment_id,
            "run_id": self.run_id,
            "scenario": self.contract.scenario,
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
            "experiment": experiment,
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
        if self.contract.records_run_mode:
            data["run_mode"] = self.run_mode
        return data


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


def format_utc(value: datetime) -> str:
    return value.astimezone(timezone.utc).isoformat(timespec="microseconds").replace(
        "+00:00", "Z"
    )


def generate_run_id(
    now: datetime | None = None,
    contract: ScenarioContract = SAND_CONTRACT,
    run_mode: str = "candidate",
) -> str:
    validate_run_mode(contract, run_mode)
    value = (now or utc_now()).astimezone(timezone.utc)
    stamp = value.strftime("%Y%m%dT%H%M%S") + f"{value.microsecond:06d}Z"
    mode_marker = "-scratch" if contract.records_run_mode and run_mode == "scratch" else ""
    return f"{contract.experiment_id}{mode_marker}-{stamp}-{secrets.token_hex(4)}"


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
    safe_root = str(source_root.resolve())
    completed = subprocess.run(
        ["git", "-c", f"safe.directory={safe_root}", *args],
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
    return inspect_named_source(source_root, allow_dirty_tracked=False)


def git_bytes(source_root: Path, *args: str) -> bytes:
    safe_root = str(source_root.resolve())
    completed = subprocess.run(
        ["git", "-c", f"safe.directory={safe_root}", *args],
        cwd=source_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if completed.returncode != 0:
        detail = completed.stderr.decode("utf-8", errors="replace").strip()
        raise ExperimentError(f"git {' '.join(args)} failed: {detail}")
    return completed.stdout


def inspect_named_source(
    source_root: Path, *, allow_dirty_tracked: bool
) -> SourceInfo:
    source = source_root.resolve(strict=True)
    branch = git_text(source, "branch", "--show-current")
    if not branch or branch == "HEAD":
        raise ExperimentError("experiment source must be on a named branch, not detached HEAD")
    status = git_bytes(
        source,
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
    )
    status_records = [record for record in status.split(b"\0") if record]
    untracked = [record for record in status_records if record.startswith(b"?? ")]
    if untracked:
        raise ExperimentError(
            "experiment source contains untracked paths; scratch permits tracked changes only"
        )
    if status and not allow_dirty_tracked:
        raise ExperimentError("experiment source must be clean; tracked changes detected")
    sha = git_text(source, "rev-parse", "HEAD")
    if not GIT_OID.fullmatch(sha):
        raise ExperimentError(f"git returned an invalid source SHA: {sha!r}")
    tracked_diff = git_bytes(
        source,
        "diff",
        "--binary",
        "--full-index",
        "--no-ext-diff",
        "--no-textconv",
        "HEAD",
        "--",
    )
    tracked_state = hashlib.sha256()
    tracked_state.update(b"git-status-porcelain-v1-z\0")
    tracked_state.update(status)
    tracked_state.update(b"\0git-diff-binary-full-index\0")
    tracked_state.update(tracked_diff)
    return SourceInfo(
        root=source,
        branch=branch,
        sha=sha,
        git_state="dirty" if status else "clean",
        tracked_state_sha256=tracked_state.hexdigest(),
    )


def is_source_input_path(relative: PurePosixPath) -> bool:
    value = relative.as_posix()
    return (
        value in SOURCE_INPUT_EXACT_PATHS
        or relative.name in {"Cargo.toml", "Cargo.lock", "build.rs"}
        or relative.suffix.lower() in {".rs", ".wgsl"}
    )


def tracked_source_input_paths(source_root: Path) -> list[PurePosixPath]:
    raw_paths = git_bytes(source_root, "ls-files", "-z", "--cached")
    paths: list[PurePosixPath] = []
    for raw_path in raw_paths.split(b"\0"):
        if not raw_path:
            continue
        try:
            text = raw_path.decode("utf-8", errors="strict")
        except UnicodeDecodeError as error:
            raise ExperimentError("tracked source input path is not valid UTF-8") from error
        relative = PurePosixPath(text)
        if relative.is_absolute() or ".." in relative.parts:
            raise ExperimentError(f"unsafe tracked source input path: {text!r}")
        if is_source_input_path(relative):
            paths.append(relative)
    paths.sort(key=PurePosixPath.as_posix)
    if not paths:
        raise ExperimentError("source input manifest selection is empty")
    required = {"Cargo.toml", "Cargo.lock", *SOURCE_INPUT_EXACT_PATHS}
    selected = {path.as_posix() for path in paths}
    missing = sorted(required - selected)
    if missing:
        raise ExperimentError(f"required source input paths are not tracked: {missing}")
    return paths


def capture_external_build_inputs() -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = []
    labels: set[str] = set()
    for label, configured_path in SOURCE_EXTERNAL_BUILD_INPUTS:
        if not re.fullmatch(r"[a-z0-9][a-z0-9._-]*", label) or label in labels:
            raise ExperimentError(f"invalid/duplicate external build input label: {label!r}")
        labels.add(label)
        path = Path(configured_path)
        if not path.is_absolute():
            raise ExperimentError(
                f"external build input must use an absolute path: {label}={path}"
            )
        try:
            resolved = path.resolve(strict=True)
        except OSError as error:
            raise ExperimentError(
                f"required external build input is missing: {label}={path}"
            ) from error
        if not resolved.is_file():
            raise ExperimentError(
                f"required external build input is not a file: {label}={resolved}"
            )
        before = resolved.stat()
        digest = sha256_file(resolved)
        after = resolved.stat()
        if (before.st_size, before.st_mtime_ns) != (
            after.st_size,
            after.st_mtime_ns,
        ):
            raise ExperimentError(
                f"external build input changed while hashing: {label}={resolved}"
            )
        entries.append(
            {
                "label": label,
                "path": str(resolved),
                "sha256": digest,
                "size_bytes": after.st_size,
            }
        )
    entries.sort(key=lambda entry: entry["label"])
    if not entries:
        raise ExperimentError("external build input seal configuration is empty")
    return entries


def capture_source_seal(
    source_root: Path, *, allow_dirty_tracked: bool = False
) -> SourceSeal:
    before = inspect_named_source(
        source_root, allow_dirty_tracked=allow_dirty_tracked
    )
    entries: list[dict[str, Any]] = []
    for relative in tracked_source_input_paths(before.root):
        path = before.root.joinpath(*relative.parts)
        if not path.is_file():
            raise ExperimentError(
                f"tracked source input is not a regular file: {relative.as_posix()}"
            )
        before_stat = path.stat()
        digest = sha256_file(path)
        after_stat = path.stat()
        if (before_stat.st_size, before_stat.st_mtime_ns) != (
            after_stat.st_size,
            after_stat.st_mtime_ns,
        ):
            raise ExperimentError(
                "tracked source input changed while hashing: "
                f"{relative.as_posix()}"
            )
        entries.append(
            {
                "path": relative.as_posix(),
                "sha256": digest,
                "size_bytes": after_stat.st_size,
            }
        )
    external_entries = capture_external_build_inputs()
    after = inspect_named_source(
        before.root, allow_dirty_tracked=allow_dirty_tracked
    )
    if after != before:
        raise ExperimentError("source identity changed while capturing input manifest")
    manifest = {
        "schema_version": SOURCE_INPUT_MANIFEST_SCHEMA,
        "source": {
            "root": str(after.root),
            "branch": after.branch,
            "head_sha": after.sha,
            "git_state": after.git_state,
        },
        "selection": {
            "tracked_only": True,
            "rules": [
                "Cargo.toml/Cargo.lock/build.rs",
                "Rust (*.rs)",
                "WGSL (*.wgsl)",
                "run_experiment.bat",
                "tools/experiment/run_experiment.py",
            ],
        },
        "file_count": len(entries),
        "files": entries,
        "external_file_count": len(external_entries),
        "external_files": external_entries,
    }
    return SourceSeal(source=after, manifest=manifest)


def render_source_input_manifest(seal: SourceSeal) -> str:
    return json.dumps(
        seal.manifest, indent=2, ensure_ascii=False, sort_keys=True
    ) + "\n"


def assert_source_manifest_artifact_unchanged(path: Path, expected: SourceSeal) -> None:
    try:
        observed = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise ExperimentError(f"source input manifest cannot be read: {error}") from error
    if observed != render_source_input_manifest(expected):
        raise ExperimentError(
            "source input manifest artifact changed; run preserved without receipt"
        )


def assert_source_seal_unchanged(
    source_root: Path, expected: SourceSeal, phase: str
) -> None:
    try:
        observed = capture_source_seal(
            source_root,
            allow_dirty_tracked=expected.source.git_state == "dirty",
        )
    except ExperimentError as error:
        raise ExperimentError(f"source seal check failed at {phase}: {error}") from error
    if observed != expected:
        raise ExperimentError(
            f"source input manifest drift detected at {phase}; "
            "run preserved without receipt"
        )


def copy_frozen_binary(source_binary: Path, run_dir: Path) -> tuple[Path, str]:
    destination = run_dir.joinpath(*FROZEN_BINARY_RELATIVE_PATH.parts)
    try:
        destination.parent.mkdir(parents=True, exist_ok=False)
        with source_binary.open("rb") as source_handle, destination.open("xb") as output:
            for block in iter(lambda: source_handle.read(1024 * 1024), b""):
                output.write(block)
            output.flush()
            os.fsync(output.fileno())
    except FileExistsError as error:
        raise ExperimentError(
            f"refusing to overwrite frozen experiment binary: {destination}"
        ) from error
    except OSError as error:
        raise ExperimentError(
            f"failed to freeze experiment binary {source_binary}: {error}"
        ) from error
    source_hash = sha256_file(source_binary)
    frozen_hash = sha256_file(destination)
    if frozen_hash != source_hash:
        raise ExperimentError("frozen experiment binary hash does not match release output")
    return destination.resolve(), frozen_hash


def assert_frozen_binary_unchanged(binary: Path, expected_sha256: str, phase: str) -> None:
    if not binary.is_file():
        raise ExperimentError(f"frozen experiment binary missing at {phase}: {binary}")
    observed = sha256_file(binary)
    if observed != expected_sha256:
        raise ExperimentError(
            f"frozen experiment binary drift detected at {phase}; "
            "run preserved without receipt"
        )


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
    ]
    if "run_mode" in data:
        lines.append(f"run_mode = {toml_quote(data['run_mode'])}")
    lines.append(f"created_utc = {toml_quote(data['created_utc'])}")
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
    contract = contract_for_manifest(data)
    expected_top = set(SAND_MANIFEST_TOP_KEYS)
    if contract.records_run_mode:
        expected_top.add("run_mode")
    require_exact_keys(data, expected_top, "manifest")
    if data["schema_version"] != contract.manifest_schema:
        raise ExperimentError("manifest schema_version mismatch")
    run_mode = data.get("run_mode", "candidate")
    if not isinstance(run_mode, str):
        raise ExperimentError("manifest run_mode must be a string")
    validate_run_mode(contract, run_mode)
    if contract.records_run_mode:
        scratch_marker_present = "-scratch-" in data["run_id"]
        if scratch_marker_present != (run_mode == "scratch"):
            raise ExperimentError("manifest run_id scratch marker disagrees with run_mode")
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
    expected_sections = {name: set(keys) for name, keys in MANIFEST_SECTION_KEYS.items()}
    if contract is FIRE_CONTRACT:
        expected_sections["experiment"] = {
            "max_ticks",
            "diagnostic_interval_ticks",
            "consecutive_reaction_zero",
            "post_reaction_ticks",
        }
    elif contract is PRESSURE_CONTRACT:
        expected_sections["experiment"] = {
            "max_ticks",
            "diagnostic_interval_ticks",
            "consecutive_persistent_opening",
            "post_opening_ticks",
            "terminal_window_samples",
        }
    if contract is WATER_CONTRACT:
        expected_sections["experiment"].add("stable_plateau_consecutive_samples")
    for section, expected in expected_sections.items():
        value = data[section]
        if not isinstance(value, dict):
            raise ExperimentError(f"manifest [{section}] must be a table")
        require_exact_keys(value, expected, f"manifest [{section}]")
    source_git_state = data["source"]["git_state"]
    if source_git_state not in {"clean", "dirty"}:
        raise ExperimentError("manifest source git_state must be clean or dirty")
    if source_git_state == "dirty" and not (
        contract.records_run_mode and run_mode == "scratch"
    ):
        raise ExperimentError(
            "dirty source is allowed only for scratch runs with an explicit run_mode"
        )
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
    if contract is FIRE_CONTRACT:
        expected_experiment = {
            "max_ticks": MAX_TICKS,
            "diagnostic_interval_ticks": DIAGNOSTIC_INTERVAL,
            "consecutive_reaction_zero": CONSECUTIVE_REACTION_ZERO,
            "post_reaction_ticks": POST_REACTION_TICKS,
        }
    elif contract is PRESSURE_CONTRACT:
        expected_experiment = {
            "max_ticks": MAX_TICKS,
            "diagnostic_interval_ticks": DIAGNOSTIC_INTERVAL,
            "consecutive_persistent_opening": CONSECUTIVE_PERSISTENT_OPENING,
            "post_opening_ticks": POST_OPENING_TICKS,
            "terminal_window_samples": TERMINAL_WINDOW_SAMPLES,
        }
    else:
        expected_experiment = {
            "max_ticks": MAX_TICKS,
            "diagnostic_interval_ticks": DIAGNOSTIC_INTERVAL,
            "consecutive_all_sleep": CONSECUTIVE_ALL_SLEEP,
            "post_sleep_ticks": POST_SLEEP_TICKS,
        }
    if contract is WATER_CONTRACT:
        expected_experiment["stable_plateau_consecutive_samples"] = (
            CONSECUTIVE_STABLE_PLATEAU
        )
    if data["experiment"] != expected_experiment:
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
    legacy_binary = source_root / "target" / "release" / "powdergame-windows.exe"
    frozen_binary = run_dir.joinpath(*FROZEN_BINARY_RELATIVE_PATH.parts)
    if contract in {FIRE_CONTRACT, PRESSURE_CONTRACT}:
        if binary_path.resolve() != frozen_binary.resolve():
            raise ExperimentError(
                f"{contract.title} manifest binary path must be the run-local frozen executable"
            )
    elif binary_path.resolve() not in {legacy_binary.resolve(), frozen_binary.resolve()}:
        raise ExperimentError(
            "Sand/Water manifest binary path is neither the historical release path "
            "nor the run-local frozen executable"
        )
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
        worker_command(
            binary_path,
            run_dir,
            data["run_id"],
            data["binary"]["sha256"],
            contract=contract,
            run_mode=run_mode,
        )
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


def require_finite_number(value: Any, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ExperimentError(f"{label} must be a finite number")
    converted = float(value)
    if not math.isfinite(converted):
        raise ExperimentError(f"{label} must be a finite number")
    return converted


def validate_sand_analysis(analysis: dict[str, Any], manifest: dict[str, Any]) -> None:
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
    if provenance["git_state"] != manifest["source"]["git_state"]:
        raise ExperimentError("analysis provenance git_state mismatch")
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
    # Sand v0 historical compatibility only: despite its old name,
    # first_all_sleep_diagnostic_sample_tick stores the diagnostic sample
    # sequence, never a simulation tick. Keep it as a deprecated alias and
    # reject any artifact in which it diverges from the explicit field.
    if (
        lifecycle["first_all_sleep_diagnostic_sample_tick"]
        != lifecycle["first_all_sleep_sample_sequence"]
    ):
        raise ExperimentError(
            "analysis deprecated first_all_sleep_diagnostic_sample_tick alias "
            "disagrees with first_all_sleep_sample_sequence"
        )

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


def require_optional_nonnegative_int(value: Any, label: str) -> int | None:
    if value is None:
        return None
    return require_nonnegative_int(value, label)


def require_optional_finite_number(value: Any, label: str) -> float | None:
    if value is None:
        return None
    return require_finite_number(value, label)


def require_optional_identity_pair(
    tick: Any, sample_sequence: Any, label: str
) -> tuple[int | None, int | None]:
    parsed_tick = require_optional_nonnegative_int(tick, f"{label} tick")
    parsed_sample = require_optional_nonnegative_int(
        sample_sequence, f"{label} sample_sequence"
    )
    if (parsed_tick is None) != (parsed_sample is None):
        raise ExperimentError(f"{label} tick/sample_sequence must both be null or integers")
    return parsed_tick, parsed_sample


def validate_water_analysis(analysis: dict[str, Any], manifest: dict[str, Any]) -> None:
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
        expected = manifest["binary"]["sha256"] if key == "binary_sha256" else manifest[key]
        if analysis[key] != expected:
            raise ExperimentError(f"analysis {key} does not match manifest")
    if analysis["schema_version"] != WATER_CONTRACT.analysis_schema:
        raise ExperimentError("analysis schema_version mismatch")

    provenance = analysis["provenance"]
    if not isinstance(provenance, dict):
        raise ExperimentError("analysis provenance must be an object")
    require_exact_keys(
        provenance, {"source_sha", "git_state", "build_profile"}, "analysis provenance"
    )
    if provenance != {
        "source_sha": manifest["source"]["sha"],
        "git_state": manifest["source"]["git_state"],
        "build_profile": "release",
    }:
        raise ExperimentError("analysis provenance mismatch")
    if analysis["world"] != manifest["world"]:
        raise ExperimentError("analysis world does not match manifest")
    sleep = analysis["sleep"]
    if not isinstance(sleep, dict):
        raise ExperimentError("analysis sleep must be an object")
    require_exact_keys(sleep, {"enabled", "threshold"}, "analysis sleep")
    if not isinstance(sleep["enabled"], bool):
        raise ExperimentError("analysis sleep enabled must be boolean")
    require_nonnegative_int(sleep["threshold"], "analysis sleep threshold")

    lifecycle = analysis["lifecycle"]
    if not isinstance(lifecycle, dict):
        raise ExperimentError("analysis lifecycle must be an object")
    lifecycle_keys = {
        "max_ticks",
        "diagnostic_interval_ticks",
        "all_sleep_consecutive_samples",
        "stable_plateau_consecutive_samples",
        "post_settle_confirmation_ticks",
        "terminal_reason",
        "first_all_sleep_sim_tick",
        "first_all_sleep_sample_sequence",
        "confirmed_all_sleep_sim_tick",
        "first_stable_plateau_sim_tick",
        "first_stable_plateau_sample_sequence",
        "confirmed_stable_plateau_sim_tick",
        "terminal_sim_tick",
        "terminal_sample_sequence",
        "post_settle_end_tick",
        "post_settle_change_ticks",
        "post_settle_wake_ticks",
        "sample_count",
    }
    require_exact_keys(lifecycle, lifecycle_keys, "analysis lifecycle")
    expected_lifecycle = {
        "max_ticks": MAX_TICKS,
        "diagnostic_interval_ticks": DIAGNOSTIC_INTERVAL,
        "all_sleep_consecutive_samples": CONSECUTIVE_ALL_SLEEP,
        "stable_plateau_consecutive_samples": CONSECUTIVE_STABLE_PLATEAU,
        "post_settle_confirmation_ticks": POST_SLEEP_TICKS,
    }
    for key, expected in expected_lifecycle.items():
        if lifecycle[key] != expected:
            raise ExperimentError(f"analysis lifecycle {key} mismatch")
    if lifecycle["terminal_reason"] not in {"all-sleep", "stable-plateau", "max-ticks"}:
        raise ExperimentError("analysis lifecycle terminal_reason is invalid")
    require_optional_identity_pair(
        lifecycle["first_all_sleep_sim_tick"],
        lifecycle["first_all_sleep_sample_sequence"],
        "analysis lifecycle first all-sleep",
    )
    require_optional_nonnegative_int(
        lifecycle["confirmed_all_sleep_sim_tick"],
        "analysis lifecycle confirmed_all_sleep_sim_tick",
    )
    require_optional_identity_pair(
        lifecycle["first_stable_plateau_sim_tick"],
        lifecycle["first_stable_plateau_sample_sequence"],
        "analysis lifecycle first stable plateau",
    )
    require_optional_nonnegative_int(
        lifecycle["confirmed_stable_plateau_sim_tick"],
        "analysis lifecycle confirmed_stable_plateau_sim_tick",
    )
    for key in (
        "terminal_sim_tick",
        "terminal_sample_sequence",
        "post_settle_change_ticks",
        "post_settle_wake_ticks",
        "sample_count",
    ):
        require_nonnegative_int(lifecycle[key], f"analysis lifecycle {key}")
    require_optional_nonnegative_int(
        lifecycle["post_settle_end_tick"], "analysis lifecycle post_settle_end_tick"
    )
    if lifecycle["terminal_reason"] == "max-ticks":
        if lifecycle["post_settle_end_tick"] is not None:
            raise ExperimentError("max-ticks terminal must not record post_settle_end_tick")
    elif lifecycle["post_settle_end_tick"] is None:
        raise ExperimentError("settled terminal must record post_settle_end_tick")

    baseline = analysis["baseline"]
    baseline_keys = {
        "matter_count",
        "water_count",
        "oil_count",
        "water_y_sum",
        "oil_y_sum",
        "water_occupied_chunks",
        "oil_occupied_chunks",
        "bottom_chunk_row_water_cells",
        "destination_water_cells",
        "destination_spread_x",
    }
    if not isinstance(baseline, dict):
        raise ExperimentError("analysis baseline must be an object")
    require_exact_keys(baseline, baseline_keys, "analysis baseline")
    for key, value in baseline.items():
        require_nonnegative_int(value, f"analysis baseline {key}")

    metrics = analysis["metrics"]
    metrics_keys = {
        "peak_active_cells",
        "peak_active_chunks",
        "peak_active_sim_tick",
        "peak_active_sample_sequence",
        "first_water_movement_tick",
        "first_water_movement_sample_sequence",
        "first_cross_chunk_flow_tick",
        "first_cross_chunk_flow_sample_sequence",
        "first_destination_arrival_tick",
        "first_destination_arrival_sample_sequence",
        "first_sleeping_chunk_tick",
        "first_sleeping_chunk_sample_sequence",
        "max_bottom_chunk_row_water_cells",
        "max_destination_water_cells",
        "max_destination_spread_x",
        "max_destination_spread_tick",
        "max_destination_spread_sample_sequence",
        "max_water_outside_outer_basin_cells",
        "final_matter_count",
        "final_water_count",
        "final_oil_count",
        "final_water_occupied_chunks",
        "final_oil_occupied_chunks",
        "final_sleeping_chunks",
        "final_water_outside_outer_basin_cells",
        "final_active_water_empty_surface_cells",
        "final_active_water_oil_interface_cells",
        "final_active_other_cells",
        "active_cell_classification_rule",
        "matter_count_delta",
        "water_count_delta",
        "oil_count_delta",
        "post_settle_state_changes",
        "post_settle_spontaneous_wakes",
        "reset_exact_equivalence",
    }
    if not isinstance(metrics, dict):
        raise ExperimentError("analysis metrics must be an object")
    require_exact_keys(metrics, metrics_keys, "analysis metrics")
    for key in (
        "peak_active_cells",
        "peak_active_chunks",
        "peak_active_sim_tick",
        "peak_active_sample_sequence",
        "max_bottom_chunk_row_water_cells",
        "max_destination_water_cells",
        "max_destination_spread_x",
        "max_water_outside_outer_basin_cells",
        "final_matter_count",
        "final_water_count",
        "final_oil_count",
        "final_water_occupied_chunks",
        "final_oil_occupied_chunks",
        "final_sleeping_chunks",
        "final_water_outside_outer_basin_cells",
        "final_active_water_empty_surface_cells",
        "final_active_water_oil_interface_cells",
        "final_active_other_cells",
        "post_settle_state_changes",
        "post_settle_spontaneous_wakes",
    ):
        require_nonnegative_int(metrics[key], f"analysis metrics {key}")
    for prefix in (
        "first_water_movement",
        "first_cross_chunk_flow",
        "first_destination_arrival",
        "first_sleeping_chunk",
        "max_destination_spread",
    ):
        require_optional_identity_pair(
            metrics[f"{prefix}_tick"],
            metrics[f"{prefix}_sample_sequence"],
            f"analysis metrics {prefix}",
        )
    if metrics["max_destination_spread_x"] == 0:
        if metrics["max_destination_spread_tick"] is not None:
            raise ExperimentError("zero max destination spread must have null identity")
    elif metrics["max_destination_spread_tick"] is None:
        raise ExperimentError("positive max destination spread must have an identity")
    for key in ("matter_count_delta", "water_count_delta", "oil_count_delta"):
        if isinstance(metrics[key], bool) or not isinstance(metrics[key], int):
            raise ExperimentError(f"analysis metrics {key} must be an integer")
    if not isinstance(metrics["reset_exact_equivalence"], bool):
        raise ExperimentError("analysis metrics reset_exact_equivalence must be boolean")
    if metrics["active_cell_classification_rule"] != WATER_ACTIVE_CLASSIFICATION_RULE:
        raise ExperimentError("analysis active-cell classification rule mismatch")

    predicates = analysis["predicates"]
    if not isinstance(predicates, dict) or set(predicates) != WATER_CONTRACT.predicate_names:
        raise ExperimentError("analysis predicates must contain the exact ten Water checks")
    for name, predicate in predicates.items():
        if not isinstance(predicate, dict) or set(predicate) != {"status", "detail"}:
            raise ExperimentError(f"analysis predicate {name} keys mismatch")
        if predicate["status"] not in PREDICATE_STATUSES:
            raise ExperimentError(f"analysis predicate {name} has invalid status")
        if not isinstance(predicate["detail"], str):
            raise ExperimentError(f"analysis predicate {name} detail must be a string")
    if analysis["verdict"] not in WATER_CONTRACT.allowed_verdicts:
        raise ExperimentError(
            "analysis verdict must be PASS, FAIL, or NEEDS_HUMAN_REVIEW"
        )
    raw_frame_count = require_nonnegative_int(
        analysis["raw_frame_count"], "analysis raw_frame_count"
    )
    if not 8 <= raw_frame_count <= 12:
        raise ExperimentError("Water analysis raw_frame_count must be between 8 and 12")


def validate_fire_analysis(analysis: dict[str, Any], manifest: dict[str, Any]) -> None:
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
    require_exact_keys(analysis, expected_keys, "Fire analysis")
    for key in ("experiment_id", "run_id", "scenario", "binary_sha256"):
        expected = manifest["binary"]["sha256"] if key == "binary_sha256" else manifest[key]
        if analysis[key] != expected:
            raise ExperimentError(f"Fire analysis {key} does not match manifest")
    if analysis["schema_version"] != FIRE_CONTRACT.analysis_schema:
        raise ExperimentError("Fire analysis schema_version mismatch")
    provenance = analysis["provenance"]
    if not isinstance(provenance, dict):
        raise ExperimentError("Fire analysis provenance must be an object")
    require_exact_keys(
        provenance, {"source_sha", "git_state", "build_profile"}, "Fire analysis provenance"
    )
    if provenance != {
        "source_sha": manifest["source"]["sha"],
        "git_state": manifest["source"]["git_state"],
        "build_profile": "release",
    }:
        raise ExperimentError("Fire analysis provenance mismatch")
    if analysis["world"] != manifest["world"]:
        raise ExperimentError("Fire analysis world does not match manifest")
    sleep = analysis["sleep"]
    if not isinstance(sleep, dict):
        raise ExperimentError("Fire analysis sleep must be an object")
    require_exact_keys(sleep, {"enabled", "threshold"}, "Fire analysis sleep")
    if not isinstance(sleep["enabled"], bool):
        raise ExperimentError("Fire analysis sleep enabled must be boolean")
    require_nonnegative_int(sleep["threshold"], "Fire analysis sleep threshold")

    lifecycle = analysis["lifecycle"]
    lifecycle_keys = {
        "max_ticks",
        "diagnostic_interval_ticks",
        "consecutive_reaction_zero_samples",
        "post_reaction_confirmation_ticks",
        "terminal_reason",
        "first_reaction_zero_sim_tick",
        "first_reaction_zero_sample_sequence",
        "confirmed_reaction_zero_sim_tick",
        "confirmed_reaction_zero_sample_sequence",
        "post_reaction_end_tick",
        "post_reaction_restart_samples",
        "sample_count",
    }
    if not isinstance(lifecycle, dict):
        raise ExperimentError("Fire analysis lifecycle must be an object")
    require_exact_keys(lifecycle, lifecycle_keys, "Fire analysis lifecycle")
    expected_lifecycle = {
        "max_ticks": MAX_TICKS,
        "diagnostic_interval_ticks": DIAGNOSTIC_INTERVAL,
        "consecutive_reaction_zero_samples": CONSECUTIVE_REACTION_ZERO,
        "post_reaction_confirmation_ticks": POST_REACTION_TICKS,
    }
    for key, expected in expected_lifecycle.items():
        if lifecycle[key] != expected:
            raise ExperimentError(f"Fire analysis lifecycle {key} mismatch")
    if lifecycle["terminal_reason"] not in {"reaction-zero", "max-ticks"}:
        raise ExperimentError("Fire analysis terminal_reason is invalid")
    require_optional_identity_pair(
        lifecycle["first_reaction_zero_sim_tick"],
        lifecycle["first_reaction_zero_sample_sequence"],
        "Fire analysis first reaction-zero",
    )
    require_optional_identity_pair(
        lifecycle["confirmed_reaction_zero_sim_tick"],
        lifecycle["confirmed_reaction_zero_sample_sequence"],
        "Fire analysis confirmed reaction-zero",
    )
    require_optional_nonnegative_int(
        lifecycle["post_reaction_end_tick"], "Fire analysis post_reaction_end_tick"
    )
    require_nonnegative_int(
        lifecycle["post_reaction_restart_samples"],
        "Fire analysis post_reaction_restart_samples",
    )
    require_nonnegative_int(lifecycle["sample_count"], "Fire analysis sample_count")

    baseline = analysis["baseline"]
    baseline_keys = {
        "matter_count",
        "wood_count",
        "oil_count",
        "smoke_count",
        "ice_count",
        "water_count",
        "steam_count",
        "fuel_count",
        "wood_fuel_progress_sum",
        "oil_fuel_progress_sum",
        "substantial_fuel_consumption_threshold",
        "substantial_fuel_remaining_threshold",
    }
    if not isinstance(baseline, dict):
        raise ExperimentError("Fire analysis baseline must be an object")
    require_exact_keys(baseline, baseline_keys, "Fire analysis baseline")
    for key, value in baseline.items():
        require_nonnegative_int(value, f"Fire analysis baseline {key}")

    metrics = analysis["metrics"]
    metrics_keys = {
        "first_combustion_tick",
        "first_combustion_sample_sequence",
        "first_smoke_tick",
        "first_smoke_sample_sequence",
        "first_phase_transition_tick",
        "first_phase_transition_sample_sequence",
        "fuel_substantially_consumed_tick",
        "fuel_substantially_consumed_sample_sequence",
        "peak_reaction_cells",
        "peak_reaction_tick",
        "peak_reaction_sample_sequence",
        "peak_thermal_cells",
        "peak_thermal_tick",
        "peak_thermal_sample_sequence",
        "peak_smoke_count",
        "peak_smoke_tick",
        "peak_smoke_sample_sequence",
        "max_heat_propagated_cells",
        "reaction_zero_tick",
        "confirmed_reaction_zero_tick",
        "post_reaction_thermal_cells",
        "post_reaction_final_thermal_cells",
        "post_reaction_min_thermal_cells",
        "post_reaction_thermal_decrease",
        "post_reaction_reaction_restart_ticks",
        "post_reaction_restart_samples",
        "final_matter_count",
        "final_wood_count",
        "final_oil_count",
        "final_smoke_count",
        "final_ice_count",
        "final_water_count",
        "final_steam_count",
        "wood_count_delta",
        "oil_count_delta",
        "fuel_count_delta",
        "fuel_consumed",
        "invalid_material_occurrences",
        "nonfinite_field_occurrences",
        "reset_exact_equivalence",
    }
    if not isinstance(metrics, dict):
        raise ExperimentError("Fire analysis metrics must be an object")
    require_exact_keys(metrics, metrics_keys, "Fire analysis metrics")
    for prefix in (
        "first_combustion",
        "first_smoke",
        "first_phase_transition",
        "fuel_substantially_consumed",
    ):
        require_optional_identity_pair(
            metrics[f"{prefix}_tick"],
            metrics[f"{prefix}_sample_sequence"],
            f"Fire analysis metrics {prefix}",
        )
    for prefix in ("peak_reaction", "peak_thermal"):
        require_nonnegative_int(metrics[f"{prefix}_cells"], f"Fire analysis {prefix}_cells")
        require_optional_identity_pair(
            metrics[f"{prefix}_tick"],
            metrics[f"{prefix}_sample_sequence"],
            f"Fire analysis {prefix}",
        )
    require_nonnegative_int(metrics["peak_smoke_count"], "Fire analysis peak_smoke_count")
    require_nonnegative_int(metrics["peak_smoke_tick"], "Fire analysis peak_smoke_tick")
    require_nonnegative_int(
        metrics["peak_smoke_sample_sequence"],
        "Fire analysis peak_smoke_sample_sequence",
    )
    for key in (
        "max_heat_propagated_cells",
        "post_reaction_thermal_cells",
        "post_reaction_final_thermal_cells",
        "post_reaction_min_thermal_cells",
        "post_reaction_reaction_restart_ticks",
        "post_reaction_restart_samples",
        "final_matter_count",
        "final_wood_count",
        "final_oil_count",
        "final_smoke_count",
        "final_ice_count",
        "final_water_count",
        "final_steam_count",
        "fuel_consumed",
        "invalid_material_occurrences",
        "nonfinite_field_occurrences",
    ):
        require_nonnegative_int(metrics[key], f"Fire analysis metrics {key}")
    for key in ("reaction_zero_tick", "confirmed_reaction_zero_tick"):
        require_optional_nonnegative_int(metrics[key], f"Fire analysis metrics {key}")
    for key in ("wood_count_delta", "oil_count_delta", "fuel_count_delta"):
        if isinstance(metrics[key], bool) or not isinstance(metrics[key], int):
            raise ExperimentError(f"Fire analysis metrics {key} must be an integer")
    for key in ("post_reaction_thermal_decrease", "reset_exact_equivalence"):
        if not isinstance(metrics[key], bool):
            raise ExperimentError(f"Fire analysis metrics {key} must be boolean")

    predicates = analysis["predicates"]
    if not isinstance(predicates, dict) or set(predicates) != FIRE_CONTRACT.predicate_names:
        raise ExperimentError("Fire analysis predicates must contain the exact twelve checks")
    for name, predicate in predicates.items():
        if not isinstance(predicate, dict) or set(predicate) != {"status", "detail"}:
            raise ExperimentError(f"Fire analysis predicate {name} keys mismatch")
        if predicate["status"] not in PREDICATE_STATUSES:
            raise ExperimentError(f"Fire analysis predicate {name} has invalid status")
        if not isinstance(predicate["detail"], str):
            raise ExperimentError(f"Fire analysis predicate {name} detail must be a string")
    if analysis["verdict"] not in FIRE_CONTRACT.allowed_verdicts:
        raise ExperimentError("Fire analysis verdict is invalid")
    raw_frame_count = require_nonnegative_int(
        analysis["raw_frame_count"], "Fire analysis raw_frame_count"
    )
    if not 8 <= raw_frame_count <= 12:
        raise ExperimentError("Fire analysis raw_frame_count must be between 8 and 12")


def validate_pressure_analysis(
    analysis: dict[str, Any], manifest: dict[str, Any]
) -> None:
    require_exact_keys(
        analysis,
        {
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
            "terminal_window",
            "review_flags",
            "causal_classification",
            "predicates",
            "verdict",
            "raw_frame_count",
        },
        "Pressure analysis",
    )
    expected_identity = {
        "schema_version": PRESSURE_CONTRACT.analysis_schema,
        "experiment_id": manifest["experiment_id"],
        "run_id": manifest["run_id"],
        "scenario": PRESSURE_CONTRACT.scenario,
        "binary_sha256": manifest["binary"]["sha256"],
    }
    for key, expected in expected_identity.items():
        if analysis[key] != expected:
            raise ExperimentError(f"Pressure analysis {key} mismatch")
    if analysis["provenance"] != {
        "source_sha": manifest["source"]["sha"],
        "git_state": manifest["source"]["git_state"],
        "build_profile": "release",
    }:
        raise ExperimentError("Pressure analysis provenance mismatch")
    if analysis["world"] != manifest["world"]:
        raise ExperimentError("Pressure analysis world mismatch")
    sleep = analysis["sleep"]
    if not isinstance(sleep, dict):
        raise ExperimentError("Pressure analysis sleep must be an object")
    require_exact_keys(sleep, {"enabled", "threshold"}, "Pressure analysis sleep")
    if not isinstance(sleep["enabled"], bool):
        raise ExperimentError("Pressure analysis sleep enabled must be boolean")
    require_nonnegative_int(sleep["threshold"], "Pressure analysis sleep threshold")

    lifecycle = analysis["lifecycle"]
    lifecycle_keys = {
        "max_ticks",
        "diagnostic_interval_ticks",
        "consecutive_persistent_opening_samples",
        "post_opening_ticks",
        "terminal_window_samples",
        "terminal_reason",
        "persistent_opening_start_sim_tick",
        "persistent_opening_start_sample_sequence",
        "persistent_opening_confirmed_sim_tick",
        "persistent_opening_confirmed_sample_sequence",
        "post_opening_end_tick",
        "sample_count",
    }
    if not isinstance(lifecycle, dict):
        raise ExperimentError("Pressure analysis lifecycle must be an object")
    require_exact_keys(lifecycle, lifecycle_keys, "Pressure analysis lifecycle")
    for key, expected in {
        "max_ticks": MAX_TICKS,
        "diagnostic_interval_ticks": DIAGNOSTIC_INTERVAL,
        "consecutive_persistent_opening_samples": CONSECUTIVE_PERSISTENT_OPENING,
        "post_opening_ticks": POST_OPENING_TICKS,
        "terminal_window_samples": TERMINAL_WINDOW_SAMPLES,
    }.items():
        if lifecycle[key] != expected:
            raise ExperimentError(f"Pressure analysis lifecycle {key} mismatch")
    if lifecycle["terminal_reason"] not in {
        "post-opening-observation-complete",
        "max-ticks",
    }:
        raise ExperimentError("Pressure analysis terminal_reason is invalid")
    require_optional_identity_pair(
        lifecycle["persistent_opening_start_sim_tick"],
        lifecycle["persistent_opening_start_sample_sequence"],
        "Pressure analysis persistent opening start",
    )
    require_optional_identity_pair(
        lifecycle["persistent_opening_confirmed_sim_tick"],
        lifecycle["persistent_opening_confirmed_sample_sequence"],
        "Pressure analysis persistent opening confirmation",
    )
    require_optional_nonnegative_int(
        lifecycle["post_opening_end_tick"], "Pressure analysis post_opening_end_tick"
    )
    require_nonnegative_int(lifecycle["sample_count"], "Pressure analysis sample_count")

    baseline = analysis["baseline"]
    baseline_keys = {
        "initial_matter_count",
        "initial_water_count",
        "initial_steam_count",
        "initial_relief_seam_wood_cells",
        "initial_top_relief_seam_wood_cells",
        "initial_bottom_relief_seam_wood_cells",
        "initial_chamber_pressure_cell_count",
        "initial_chamber_mean_pressure",
        "initial_chamber_max_pressure",
    }
    if not isinstance(baseline, dict):
        raise ExperimentError("Pressure analysis baseline must be an object")
    require_exact_keys(baseline, baseline_keys, "Pressure analysis baseline")
    for key in baseline_keys - {
        "initial_chamber_mean_pressure",
        "initial_chamber_max_pressure",
    }:
        require_nonnegative_int(baseline[key], f"Pressure analysis baseline {key}")
    for key in ("initial_chamber_mean_pressure", "initial_chamber_max_pressure"):
        require_finite_number(baseline[key], f"Pressure analysis baseline {key}")

    metrics = analysis["metrics"]
    metrics_keys = {
        "first_pressure_activity_tick",
        "first_pressure_activity_sample_sequence",
        "first_wood_damage_tick",
        "first_wood_damage_sample_sequence",
        "first_rupture_tick",
        "first_rupture_sample_sequence",
        "first_persistent_opening_tick",
        "first_persistent_opening_sample_sequence",
        "persistent_opening_confirmed_tick",
        "persistent_opening_confirmed_sample_sequence",
        "first_outside_chamber_steam_tick",
        "first_outside_chamber_steam_sample_sequence",
        "first_steam_in_relief_seam_tick",
        "first_steam_in_relief_seam_sample_sequence",
        "first_post_confirmation_reseal_tick",
        "first_post_confirmation_reseal_sample_sequence",
        "first_post_opening_relief_tick",
        "first_post_opening_relief_sample_sequence",
        "first_relief_seam_combustion_tick",
        "first_relief_seam_combustion_sample_sequence",
        "first_relief_seam_fuel_progress_tick",
        "first_relief_seam_fuel_progress_sample_sequence",
        "peak_chamber_mean_pressure",
        "peak_chamber_mean_pressure_tick",
        "peak_chamber_mean_pressure_sample_sequence",
        "peak_chamber_max_pressure",
        "peak_chamber_max_pressure_tick",
        "peak_chamber_max_pressure_sample_sequence",
        "peak_pressure_active_cells",
        "peak_pressure_active_tick",
        "peak_pressure_active_sample_sequence",
        "pre_opening_peak_chamber_mean_pressure",
        "pre_opening_peak_chamber_max_pressure",
        "vent_reference_chamber_mean_pressure",
        "vent_reference_chamber_max_pressure",
        "post_opening_chamber_mean_pressure",
        "post_opening_chamber_max_pressure",
        "terminal_chamber_mean_pressure",
        "terminal_chamber_max_pressure",
        "terminal_pressure_relieved",
        "through_opening_confirmation_relief_seam_combusting_cells_peak",
        "through_opening_confirmation_relief_seam_flame_event_cells_peak",
        "through_opening_confirmation_relief_seam_fuel_progress_sum_peak",
        "through_opening_confirmation_relief_seam_fuel_progress_max",
        "opening_confirmation_relief_seam_combusting_cells",
        "opening_confirmation_relief_seam_flame_event_cells",
        "opening_confirmation_relief_seam_fuel_progress_sum",
        "opening_confirmation_relief_seam_fuel_progress_max",
        "opening_confirmation_relief_seam_adjacent_pressure_medium_cells",
        "opening_confirmation_relief_seam_max_adjacent_pressure",
        "opening_confirmation_adjacent_pressure_at_or_above_wood_rupture_threshold",
        "first_opening_relief_seam_adjacent_pressure_medium_cells",
        "first_opening_relief_seam_max_adjacent_pressure",
        "first_opening_adjacent_pressure_at_or_above_wood_rupture_threshold",
        "wood_rupture_threshold",
        "final_relief_seam_wood_cells",
        "final_top_relief_seam_wood_cells",
        "final_bottom_relief_seam_wood_cells",
        "final_relief_seam_open_cells",
        "final_top_relief_seam_open_cells",
        "final_bottom_relief_seam_open_cells",
        "final_relief_seam_through_open_lanes",
        "final_top_relief_seam_through_open_lanes",
        "final_bottom_relief_seam_through_open_lanes",
        "final_steam_in_relief_seam_cells",
        "outside_chamber_steam_peak",
        "final_outside_chamber_steam_cells",
        "final_matter_count",
        "matter_count_delta",
        "final_water_count",
        "water_count_delta",
        "final_steam_count",
        "steam_count_delta",
        "final_pressure_active_cells",
        "final_thermal_active_cells",
        "final_reaction_active_cells",
        "invalid_material_occurrences",
        "nonfinite_field_occurrences",
        "reset_exact_equivalence",
    }
    if not isinstance(metrics, dict):
        raise ExperimentError("Pressure analysis metrics must be an object")
    require_exact_keys(metrics, metrics_keys, "Pressure analysis metrics")
    for prefix in (
        "first_pressure_activity",
        "first_wood_damage",
        "first_rupture",
        "first_persistent_opening",
        "persistent_opening_confirmed",
        "first_outside_chamber_steam",
        "first_steam_in_relief_seam",
        "first_post_confirmation_reseal",
        "first_post_opening_relief",
        "first_relief_seam_combustion",
        "first_relief_seam_fuel_progress",
    ):
        require_optional_identity_pair(
            metrics[f"{prefix}_tick"],
            metrics[f"{prefix}_sample_sequence"],
            f"Pressure analysis metrics {prefix}",
        )
    for prefix in (
        "peak_chamber_mean_pressure",
        "peak_chamber_max_pressure",
        "peak_pressure_active",
    ):
        require_nonnegative_int(metrics[f"{prefix}_tick"], f"Pressure {prefix} tick")
        require_nonnegative_int(
            metrics[f"{prefix}_sample_sequence"], f"Pressure {prefix} sample"
        )
    for key in (
        "peak_chamber_mean_pressure",
        "peak_chamber_max_pressure",
        "pre_opening_peak_chamber_mean_pressure",
        "pre_opening_peak_chamber_max_pressure",
    ):
        require_finite_number(metrics[key], f"Pressure analysis metrics {key}")
    for key in (
        "vent_reference_chamber_mean_pressure",
        "vent_reference_chamber_max_pressure",
        "post_opening_chamber_mean_pressure",
        "post_opening_chamber_max_pressure",
        "terminal_chamber_mean_pressure",
        "terminal_chamber_max_pressure",
        "opening_confirmation_relief_seam_max_adjacent_pressure",
        "first_opening_relief_seam_max_adjacent_pressure",
    ):
        require_optional_finite_number(metrics[key], f"Pressure analysis metrics {key}")
    rupture_threshold = require_finite_number(
        metrics["wood_rupture_threshold"],
        "Pressure analysis metrics wood_rupture_threshold",
    )
    if rupture_threshold < 0:
        raise ExperimentError(
            "Pressure analysis metrics wood_rupture_threshold must be nonnegative"
        )
    optional_integer_metrics = {
        "opening_confirmation_relief_seam_combusting_cells",
        "opening_confirmation_relief_seam_flame_event_cells",
        "opening_confirmation_relief_seam_fuel_progress_sum",
        "opening_confirmation_relief_seam_fuel_progress_max",
        "opening_confirmation_relief_seam_adjacent_pressure_medium_cells",
        "first_opening_relief_seam_adjacent_pressure_medium_cells",
    }
    for key in optional_integer_metrics:
        require_optional_nonnegative_int(
            metrics[key], f"Pressure analysis metrics {key}"
        )
    optional_boolean_metrics = {
        "opening_confirmation_adjacent_pressure_at_or_above_wood_rupture_threshold",
        "first_opening_adjacent_pressure_at_or_above_wood_rupture_threshold",
    }
    for key in optional_boolean_metrics:
        if metrics[key] is not None and not isinstance(metrics[key], bool):
            raise ExperimentError(
                f"Pressure analysis metrics {key} must be boolean or null"
            )
    integer_metrics = metrics_keys - {
        *{
            f"{prefix}_{suffix}"
            for prefix in (
                "first_pressure_activity",
                "first_wood_damage",
                "first_rupture",
                "first_persistent_opening",
                "persistent_opening_confirmed",
                "first_outside_chamber_steam",
                "first_steam_in_relief_seam",
                "first_post_confirmation_reseal",
                "first_post_opening_relief",
                "first_relief_seam_combustion",
                "first_relief_seam_fuel_progress",
            )
            for suffix in ("tick", "sample_sequence")
        },
        "peak_chamber_mean_pressure",
        "peak_chamber_max_pressure",
        "pre_opening_peak_chamber_mean_pressure",
        "pre_opening_peak_chamber_max_pressure",
        "vent_reference_chamber_mean_pressure",
        "vent_reference_chamber_max_pressure",
        "post_opening_chamber_mean_pressure",
        "post_opening_chamber_max_pressure",
        "terminal_chamber_mean_pressure",
        "terminal_chamber_max_pressure",
        "opening_confirmation_relief_seam_max_adjacent_pressure",
        "first_opening_relief_seam_max_adjacent_pressure",
        "wood_rupture_threshold",
        "terminal_pressure_relieved",
        "reset_exact_equivalence",
        *optional_integer_metrics,
        *optional_boolean_metrics,
        "matter_count_delta",
        "water_count_delta",
        "steam_count_delta",
    }
    for key in integer_metrics:
        require_nonnegative_int(metrics[key], f"Pressure analysis metrics {key}")
    for key in ("matter_count_delta", "water_count_delta", "steam_count_delta"):
        if isinstance(metrics[key], bool) or not isinstance(metrics[key], int):
            raise ExperimentError(f"Pressure analysis metrics {key} must be an integer")
    for key in ("terminal_pressure_relieved", "reset_exact_equivalence"):
        if not isinstance(metrics[key], bool):
            raise ExperimentError(f"Pressure analysis metrics {key} must be boolean")

    if analysis["causal_classification"] not in PRESSURE_CAUSAL_CLASSIFICATIONS:
        raise ExperimentError("Pressure analysis causal_classification is invalid")

    window = analysis["terminal_window"]
    window_keys = {
        "sample_count",
        "start_sim_tick",
        "end_sim_tick",
        "start_mean_pressure",
        "end_mean_pressure",
        "start_max_pressure",
        "end_max_pressure",
        "minimum_mean_pressure",
        "maximum_mean_pressure",
        "slope_per_sample",
        "positive_step_count",
        "positive_max_step_count",
        "mean_unbounded_growth",
        "max_unbounded_growth",
        "unbounded_growth",
    }
    if not isinstance(window, dict):
        raise ExperimentError("Pressure analysis terminal_window must be an object")
    require_exact_keys(window, window_keys, "Pressure analysis terminal_window")
    require_nonnegative_int(window["sample_count"], "Pressure terminal window sample_count")
    require_nonnegative_int(
        window["positive_step_count"], "Pressure terminal window positive_step_count"
    )
    require_nonnegative_int(
        window["positive_max_step_count"],
        "Pressure terminal window positive_max_step_count",
    )
    for key in ("start_sim_tick", "end_sim_tick"):
        require_optional_nonnegative_int(window[key], f"Pressure terminal window {key}")
    for key in (
        "start_mean_pressure",
        "end_mean_pressure",
        "start_max_pressure",
        "end_max_pressure",
        "minimum_mean_pressure",
        "maximum_mean_pressure",
        "slope_per_sample",
    ):
        require_optional_finite_number(window[key], f"Pressure terminal window {key}")
    for key in ("mean_unbounded_growth", "max_unbounded_growth", "unbounded_growth"):
        if not isinstance(window[key], bool):
            raise ExperimentError(f"Pressure terminal window {key} must be boolean")

    flags = analysis["review_flags"]
    flag_names = (
        "only_one_relief_seam_ruptured",
        "high_terminal_pressure_activity",
        "long_pressure_tail",
        "persistent_vent_plume",
        "terminal_activity_remains",
    )
    if not isinstance(flags, dict):
        raise ExperimentError("Pressure analysis review_flags must be an object")
    require_exact_keys(flags, set(flag_names) | {"reasons"}, "Pressure review_flags")
    for name in flag_names:
        if not isinstance(flags[name], bool):
            raise ExperimentError(f"Pressure review flag {name} must be boolean")
    if not isinstance(flags["reasons"], list) or not all(
        isinstance(reason, str) for reason in flags["reasons"]
    ):
        raise ExperimentError("Pressure review_flags reasons must be a string array")

    predicates = analysis["predicates"]
    if not isinstance(predicates, dict) or set(predicates) != PRESSURE_PREDICATE_NAMES:
        raise ExperimentError("Pressure analysis predicates must contain the exact ten checks")
    for name, predicate in predicates.items():
        if not isinstance(predicate, dict):
            raise ExperimentError(f"Pressure analysis predicate {name} must be an object")
        require_exact_keys(predicate, {"status", "detail"}, f"Pressure predicate {name}")
        if predicate["status"] not in PREDICATE_STATUSES:
            raise ExperimentError(f"Pressure analysis predicate {name} status is invalid")
        if not isinstance(predicate["detail"], str):
            raise ExperimentError(f"Pressure analysis predicate {name} detail must be a string")
    if analysis["verdict"] not in PRESSURE_ALLOWED_VERDICTS:
        raise ExperimentError("Pressure analysis verdict is invalid")
    raw_frame_count = require_nonnegative_int(
        analysis["raw_frame_count"], "Pressure analysis raw_frame_count"
    )
    if not 8 <= raw_frame_count <= 12:
        raise ExperimentError("Pressure analysis raw_frame_count must be between 8 and 12")


def validate_analysis(analysis: dict[str, Any], manifest: dict[str, Any]) -> None:
    contract = contract_for_manifest(manifest)
    if contract is SAND_CONTRACT:
        validate_sand_analysis(analysis, manifest)
    elif contract is WATER_CONTRACT:
        validate_water_analysis(analysis, manifest)
    elif contract is FIRE_CONTRACT:
        validate_fire_analysis(analysis, manifest)
    elif contract is PRESSURE_CONTRACT:
        validate_pressure_analysis(analysis, manifest)
    else:
        raise ExperimentError(f"unsupported analysis contract: {contract.scenario}")


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
    if reason is None and isinstance(frame.get("badges"), list):
        reason = "+".join(
            str(badge.get("kind", ""))
            for badge in frame["badges"]
            if isinstance(badge, dict)
        )
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
    contract = contract_for_manifest(manifest)
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
    if frames_doc["schema_version"] != contract.frames_schema:
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
    if contract in {WATER_CONTRACT, FIRE_CONTRACT, PRESSURE_CONTRACT} and not 8 <= len(frames) <= 12:
        raise ExperimentError(
            f"{contract.title} frames.json must contain between 8 and 12 frames"
        )
    if frames_doc["pixel_encoding"] != "rgba8-tightly-packed":
        raise ExperimentError("frames pixel_encoding mismatch")
    required_frame = {
        "ordinal",
        "relative_path",
        "width",
        "height",
        "rgba_bytes",
        "sim_tick",
        "sample_sequence",
        "state_hash",
    }
    if contract is PRESSURE_CONTRACT:
        required_frame.add("badges")
    else:
        required_frame.update({"kind", "reason"})
    seen_paths: set[str] = set()
    seen_names: set[str] = set()
    for expected_ordinal, frame in enumerate(frames):
        if not isinstance(frame, dict):
            raise ExperimentError(f"frame {expected_ordinal} must be an object")
        require_exact_keys(frame, required_frame, f"frame {expected_ordinal}")
        if frame["ordinal"] != expected_ordinal:
            raise ExperimentError("frame ordinals must be contiguous and zero-based")
        if contract is PRESSURE_CONTRACT:
            badges = frame["badges"]
            if not isinstance(badges, list) or not badges:
                raise ExperimentError(
                    f"Pressure frame {expected_ordinal} badges must be non-empty"
                )
            seen_badges: set[str] = set()
            previous_rank = -1
            for badge_index, badge in enumerate(badges):
                if not isinstance(badge, dict):
                    raise ExperimentError(
                        f"Pressure frame {expected_ordinal} badge {badge_index} must be an object"
                    )
                require_exact_keys(
                    badge,
                    {"kind", "reason"},
                    f"Pressure frame {expected_ordinal} badge {badge_index}",
                )
                kind = badge["kind"]
                if kind not in PRESSURE_FRAME_BADGE_RANK:
                    raise ExperimentError(
                        f"Pressure frame {expected_ordinal} badge kind {kind!r} is unsupported"
                    )
                if kind in seen_badges:
                    raise ExperimentError(
                        f"Pressure frame {expected_ordinal} contains duplicate badge {kind}"
                    )
                seen_badges.add(kind)
                rank = PRESSURE_FRAME_BADGE_RANK[kind]
                if rank <= previous_rank:
                    raise ExperimentError(
                        f"Pressure frame {expected_ordinal} badges are not in canonical order"
                    )
                previous_rank = rank
                if not isinstance(badge["reason"], str) or not badge["reason"]:
                    raise ExperimentError(
                        f"Pressure frame {expected_ordinal} badge reason must be non-empty"
                    )
        else:
            if not isinstance(frame["kind"], str) or not frame["kind"]:
                raise ExperimentError(f"frame {expected_ordinal} kind must be non-empty")
        if contract in {WATER_CONTRACT, FIRE_CONTRACT}:
            allowed_kinds = (
                WATER_FRAME_KINDS if contract is WATER_CONTRACT else FIRE_FRAME_KINDS
            )
            if frame["kind"] not in allowed_kinds:
                raise ExperimentError(
                    f"{contract.title} frame {expected_ordinal} has unsupported kind "
                    f"{frame['kind']!r}"
                )
            if (
                frame["kind"] == "diagnostic-observation"
                and frame["reason"] != "minimum-evidence-observation"
            ):
                raise ExperimentError(
                    "diagnostic-observation frame reason must be "
                    "minimum-evidence-observation"
                )
        if frame["width"] != RENDERER_WIDTH or frame["height"] != RENDERER_HEIGHT:
            raise ExperimentError("raw frame dimensions must be exactly 1600x900")
        expected_bytes = RENDERER_WIDTH * RENDERER_HEIGHT * 4
        if frame["rgba_bytes"] != expected_bytes:
            raise ExperimentError("raw frame rgba_bytes does not match dimensions")
        require_nonnegative_int(frame["sim_tick"], "frame sim_tick")
        require_nonnegative_int(frame["sample_sequence"], "frame sample_sequence")
        if contract is not PRESSURE_CONTRACT and (
            not isinstance(frame["reason"], str) or not frame["reason"]
        ):
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


def validate_sand_samples(samples: list[dict[str, Any]], manifest: dict[str, Any]) -> None:
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
            "git_state": manifest["source"]["git_state"],
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


def validate_water_samples(samples: list[dict[str, Any]], manifest: dict[str, Any]) -> None:
    expected_keys = {
        "schema_version",
        "experiment_id",
        "run_id",
        "scenario",
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
        "water_count",
        "oil_count",
        "water_y_sum",
        "water_min_y",
        "water_max_y",
        "oil_y_sum",
        "oil_min_y",
        "oil_max_y",
        "water_occupied_chunks",
        "oil_occupied_chunks",
        "water_outside_initial_mask",
        "water_outside_outer_basin_cells",
        "initial_water_cells_vacated",
        "bottom_chunk_row_water_cells",
        "destination_water_cells",
        "destination_spread_x",
        "invalid_material_count",
        "nonfinite_temperature_count",
        "nonfinite_pressure_count",
        "changed_chunks",
        "wake_chunks",
        "wake_reason_or",
        "state_hash",
        "physical_state_hash",
        "active_water_empty_surface_cells",
        "active_water_oil_interface_cells",
        "active_other_cells",
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
        if sample["schema_version"] != WATER_CONTRACT.telemetry_schema:
            raise ExperimentError(f"sample {index} schema_version mismatch")
        identity = {
            "experiment_id": manifest["experiment_id"],
            "run_id": manifest["run_id"],
            "scenario": WATER_CONTRACT.scenario,
            "source_sha": manifest["source"]["sha"],
            "git_state": manifest["source"]["git_state"],
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
        if sample["phase"] not in WATER_PHASES:
            raise ExperimentError(f"sample {index} phase is invalid")
        if sample["reason"] not in WATER_REASONS:
            raise ExperimentError(f"sample {index} reason is invalid")
        expected_reasons = {
            "initial": {"tick0"},
            "flowing": {"tick1", "early-flow", "diagnostic-cadence", "max-tick"},
            "post-settle-confirmation": {"post-settle-tick"},
            "reset": {"programmatic-r-equivalent"},
        }
        if sample["reason"] not in expected_reasons[sample["phase"]]:
            raise ExperimentError(f"sample {index} phase/reason mismatch")
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
        for key in ("active_chunks", "runnable_chunks", "sleeping_chunks"):
            if census[key] > total_chunks:
                raise ExperimentError(f"sample {index} census {key} exceeds total chunks")
        if census["runnable_chunks"] + census["sleeping_chunks"] != total_chunks:
            raise ExperimentError(f"sample {index} chunk-state census is incomplete")
        for key in (
            "matter_active_cells",
            "thermal_active_cells",
            "pressure_active_cells",
            "reaction_active_cells",
        ):
            if census[key] > census["any_active_cells"]:
                raise ExperimentError(f"sample {index} census {key} exceeds any-active cells")
        counts = sample["material_counts_by_id"]
        if not isinstance(counts, list) or len(counts) != 10:
            raise ExperimentError(f"sample {index} material_counts_by_id mismatch")
        for material_id, value in enumerate(counts):
            require_nonnegative_int(value, f"sample {index} material count {material_id}")
        scalar_counts = (
            "matter_count",
            "water_count",
            "oil_count",
            "water_y_sum",
            "oil_y_sum",
            "water_occupied_chunks",
            "oil_occupied_chunks",
            "water_outside_initial_mask",
            "water_outside_outer_basin_cells",
            "initial_water_cells_vacated",
            "bottom_chunk_row_water_cells",
            "destination_water_cells",
            "destination_spread_x",
            "invalid_material_count",
            "nonfinite_temperature_count",
            "nonfinite_pressure_count",
            "changed_chunks",
            "wake_chunks",
            "wake_reason_or",
            "active_water_empty_surface_cells",
            "active_water_oil_interface_cells",
            "active_other_cells",
        )
        for key in scalar_counts:
            require_nonnegative_int(sample[key], f"sample {index} {key}")
        if sample["matter_count"] != sum(counts[1:]):
            raise ExperimentError(f"sample {index} matter_count does not match material counts")
        if sample["water_count"] != counts[4] or sample["oil_count"] != counts[5]:
            raise ExperimentError(f"sample {index} Water/Oil counts do not match material counts")
        if sum(counts) + sample["invalid_material_count"] != WORLD_WIDTH * WORLD_HEIGHT:
            raise ExperimentError(f"sample {index} material census does not cover the world")
        for material in ("water", "oil"):
            count = sample[f"{material}_count"]
            minimum = sample[f"{material}_min_y"]
            maximum = sample[f"{material}_max_y"]
            require_optional_nonnegative_int(minimum, f"sample {index} {material}_min_y")
            require_optional_nonnegative_int(maximum, f"sample {index} {material}_max_y")
            if count == 0:
                if minimum is not None or maximum is not None:
                    raise ExperimentError(
                        f"sample {index} empty {material} census must have null y bounds"
                    )
            elif minimum is None or maximum is None or minimum > maximum or maximum >= WORLD_HEIGHT:
                raise ExperimentError(f"sample {index} {material} y bounds are invalid")
            else:
                y_sum = sample[f"{material}_y_sum"]
                if not minimum * count <= y_sum <= maximum * count:
                    raise ExperimentError(f"sample {index} {material} y sum is out of bounds")
        for key in ("water_occupied_chunks", "oil_occupied_chunks"):
            if sample[key] > total_chunks:
                raise ExperimentError(f"sample {index} {key} exceeds total chunks")
        for key in (
            "water_outside_initial_mask",
            "water_outside_outer_basin_cells",
            "bottom_chunk_row_water_cells",
            "destination_water_cells",
        ):
            if sample[key] > sample["water_count"]:
                raise ExperimentError(f"sample {index} {key} exceeds water_count")
        if sample["initial_water_cells_vacated"] > WORLD_WIDTH * WORLD_HEIGHT:
            raise ExperimentError(f"sample {index} vacated Water count exceeds the world")
        if sample["destination_water_cells"] == 0 and sample["destination_spread_x"] != 0:
            raise ExperimentError(f"sample {index} empty destination must have zero spread")
        if sample["destination_spread_x"] >= WORLD_WIDTH:
            raise ExperimentError(f"sample {index} destination spread is out of range")
        active_classified = (
            sample["active_water_empty_surface_cells"]
            + sample["active_water_oil_interface_cells"]
            + sample["active_other_cells"]
        )
        if active_classified != census["any_active_cells"]:
            raise ExperimentError(
                f"sample {index} active-cell classifications do not partition "
                "census any_active_cells"
            )
        if not isinstance(sample["state_hash"], str) or not STATE_HASH.fullmatch(
            sample["state_hash"]
        ):
            raise ExperimentError(f"sample {index} state_hash is invalid")
        if not isinstance(sample["physical_state_hash"], str) or not STATE_HASH.fullmatch(
            sample["physical_state_hash"]
        ):
            raise ExperimentError(f"sample {index} physical_state_hash is invalid")
        if (sample["wake_chunks"] == 0) != (sample["wake_reason_or"] == 0):
            raise ExperimentError(f"sample {index} wake census and wake_reason_or disagree")


FIRE_PHASES = frozenset({"initial", "reacting", "post-reaction-confirmation", "reset"})
FIRE_REASONS = frozenset(
    {
        "tick0",
        "tick1",
        "early-diagnostic",
        "diagnostic-cadence",
        "max-tick",
        "post-reaction-tick",
        "programmatic-r-equivalent",
    }
)


def validate_fire_samples(samples: list[dict[str, Any]], manifest: dict[str, Any]) -> None:
    expected_keys = {
        "schema_version",
        "experiment_id",
        "run_id",
        "scenario",
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
        "wood_count",
        "oil_count",
        "smoke_count",
        "ice_count",
        "water_count",
        "steam_count",
        "combusting_wood_cells",
        "combusting_oil_cells",
        "flame_event_wood_cells",
        "flame_event_oil_cells",
        "wood_fuel_progress_sum",
        "oil_fuel_progress_sum",
        "heat_propagated_cells",
        "phase_inventory_changed",
        "invalid_material_count",
        "nonfinite_temperature_count",
        "nonfinite_pressure_count",
        "changed_chunks",
        "wake_chunks",
        "wake_reason_or",
        "state_hash",
        "physical_state_hash",
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
    expected_reasons = {
        "initial": {"tick0"},
        "reacting": {"tick1", "early-diagnostic", "diagnostic-cadence", "max-tick"},
        "post-reaction-confirmation": {"post-reaction-tick"},
        "reset": {"programmatic-r-equivalent"},
    }
    total_chunks = (WORLD_WIDTH // CHUNK_SIZE) * (WORLD_HEIGHT // CHUNK_SIZE)
    for index, sample in enumerate(samples):
        require_exact_keys(sample, expected_keys, f"Fire sample {index}")
        if sample["schema_version"] != FIRE_CONTRACT.telemetry_schema:
            raise ExperimentError(f"Fire sample {index} schema_version mismatch")
        identity = {
            "experiment_id": manifest["experiment_id"],
            "run_id": manifest["run_id"],
            "scenario": FIRE_CONTRACT.scenario,
            "source_sha": manifest["source"]["sha"],
            "git_state": manifest["source"]["git_state"],
            "build_profile": "release",
            "binary_sha256": manifest["binary"]["sha256"],
        }
        for key, expected in identity.items():
            if sample[key] != expected:
                raise ExperimentError(f"Fire sample {index} {key} mismatch")
        if require_nonnegative_int(sample["sample_sequence"], "Fire sample sequence") != index:
            raise ExperimentError("Fire sample_sequence must be contiguous and zero-based")
        require_nonnegative_int(sample["sim_tick"], f"Fire sample {index} sim_tick")
        if sample["phase"] not in FIRE_PHASES:
            raise ExperimentError(f"Fire sample {index} phase is invalid")
        if sample["reason"] not in FIRE_REASONS:
            raise ExperimentError(f"Fire sample {index} reason is invalid")
        if sample["reason"] not in expected_reasons[sample["phase"]]:
            raise ExperimentError(f"Fire sample {index} phase/reason mismatch")
        if sample["world"] != manifest["world"]:
            raise ExperimentError(f"Fire sample {index} world mismatch")
        sleep = sample["sleep"]
        if not isinstance(sleep, dict):
            raise ExperimentError(f"Fire sample {index} sleep must be an object")
        require_exact_keys(sleep, {"enabled", "threshold"}, f"Fire sample {index} sleep")
        if not isinstance(sleep["enabled"], bool):
            raise ExperimentError(f"Fire sample {index} sleep enabled must be boolean")
        require_nonnegative_int(sleep["threshold"], f"Fire sample {index} sleep threshold")
        census = sample["census"]
        if not isinstance(census, dict):
            raise ExperimentError(f"Fire sample {index} census must be an object")
        require_exact_keys(census, census_keys, f"Fire sample {index} census")
        for key, value in census.items():
            require_nonnegative_int(value, f"Fire sample {index} census {key}")
        if census["total_cells"] != WORLD_WIDTH * WORLD_HEIGHT:
            raise ExperimentError(f"Fire sample {index} census total_cells mismatch")
        if census["total_chunks"] != total_chunks:
            raise ExperimentError(f"Fire sample {index} census total_chunks mismatch")
        for key in ("active_chunks", "runnable_chunks", "sleeping_chunks"):
            if census[key] > total_chunks:
                raise ExperimentError(f"Fire sample {index} census {key} exceeds total chunks")
        if census["runnable_chunks"] + census["sleeping_chunks"] != total_chunks:
            raise ExperimentError(f"Fire sample {index} chunk-state census is incomplete")
        for key in (
            "matter_active_cells",
            "thermal_active_cells",
            "pressure_active_cells",
            "reaction_active_cells",
        ):
            if census[key] > census["any_active_cells"]:
                raise ExperimentError(
                    f"Fire sample {index} census {key} exceeds any-active cells"
                )
        counts = sample["material_counts_by_id"]
        if not isinstance(counts, list) or len(counts) != 10:
            raise ExperimentError(f"Fire sample {index} material_counts_by_id mismatch")
        for material_id, value in enumerate(counts):
            require_nonnegative_int(value, f"Fire sample {index} material count {material_id}")
        scalar_keys = (
            "matter_count",
            "wood_count",
            "oil_count",
            "smoke_count",
            "ice_count",
            "water_count",
            "steam_count",
            "combusting_wood_cells",
            "combusting_oil_cells",
            "flame_event_wood_cells",
            "flame_event_oil_cells",
            "wood_fuel_progress_sum",
            "oil_fuel_progress_sum",
            "heat_propagated_cells",
            "invalid_material_count",
            "nonfinite_temperature_count",
            "nonfinite_pressure_count",
            "changed_chunks",
            "wake_chunks",
            "wake_reason_or",
        )
        for key in scalar_keys:
            require_nonnegative_int(sample[key], f"Fire sample {index} {key}")
        if not isinstance(sample["phase_inventory_changed"], bool):
            raise ExperimentError(f"Fire sample {index} phase_inventory_changed must be boolean")
        if sample["matter_count"] != sum(counts[1:]):
            raise ExperimentError(f"Fire sample {index} matter_count disagrees with material census")
        named_counts = {
            "water_count": counts[4],
            "oil_count": counts[5],
            "steam_count": counts[6],
            "smoke_count": counts[7],
            "ice_count": counts[8],
            "wood_count": counts[9],
        }
        for key, expected in named_counts.items():
            if sample[key] != expected:
                raise ExperimentError(f"Fire sample {index} {key} disagrees with material census")
        if sum(counts) + sample["invalid_material_count"] != WORLD_WIDTH * WORLD_HEIGHT:
            raise ExperimentError(f"Fire sample {index} material census does not cover the world")
        for flag_count, material_count in (
            ("combusting_wood_cells", "wood_count"),
            ("combusting_oil_cells", "oil_count"),
            ("flame_event_wood_cells", "wood_count"),
            ("flame_event_oil_cells", "oil_count"),
        ):
            if sample[flag_count] > sample[material_count]:
                raise ExperimentError(
                    f"Fire sample {index} {flag_count} exceeds {material_count}"
                )
        for key in ("state_hash", "physical_state_hash"):
            if not isinstance(sample[key], str) or not STATE_HASH.fullmatch(sample[key]):
                raise ExperimentError(f"Fire sample {index} {key} is invalid")
        if (sample["wake_chunks"] == 0) != (sample["wake_reason_or"] == 0):
            raise ExperimentError(f"Fire sample {index} wake census and wake_reason_or disagree")


def validate_pressure_samples(
    samples: list[dict[str, Any]], manifest: dict[str, Any]
) -> None:
    expected_keys = {
        "schema_version",
        "experiment_id",
        "run_id",
        "scenario",
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
        "water_count",
        "steam_count",
        "relief_seam_wood_cells",
        "top_relief_seam_wood_cells",
        "bottom_relief_seam_wood_cells",
        "relief_seam_open_cells",
        "top_relief_seam_open_cells",
        "bottom_relief_seam_open_cells",
        "relief_seam_through_open_lanes",
        "top_relief_seam_through_open_lanes",
        "bottom_relief_seam_through_open_lanes",
        "top_relief_seam_combusting_cells",
        "bottom_relief_seam_combusting_cells",
        "relief_seam_combusting_cells",
        "top_relief_seam_flame_event_cells",
        "bottom_relief_seam_flame_event_cells",
        "relief_seam_flame_event_cells",
        "top_relief_seam_fuel_progress_sum",
        "top_relief_seam_fuel_progress_max",
        "bottom_relief_seam_fuel_progress_sum",
        "bottom_relief_seam_fuel_progress_max",
        "relief_seam_fuel_progress_sum",
        "relief_seam_fuel_progress_max",
        "top_relief_seam_adjacent_pressure_medium_cells",
        "bottom_relief_seam_adjacent_pressure_medium_cells",
        "relief_seam_adjacent_pressure_medium_cells",
        "top_relief_seam_max_adjacent_pressure",
        "bottom_relief_seam_max_adjacent_pressure",
        "relief_seam_max_adjacent_pressure",
        "steam_in_relief_seam_cells",
        "outside_chamber_steam_cells",
        "chamber_pressure_cell_count",
        "chamber_mean_pressure",
        "chamber_max_pressure",
        "invalid_material_count",
        "nonfinite_temperature_count",
        "nonfinite_pressure_count",
        "changed_chunks",
        "wake_chunks",
        "wake_reason_or",
        "state_hash",
        "physical_state_hash",
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
    identity = {
        "schema_version": PRESSURE_TELEMETRY_SCHEMA,
        "experiment_id": manifest["experiment_id"],
        "run_id": manifest["run_id"],
        "scenario": PRESSURE_CONTRACT.scenario,
        "source_sha": manifest["source"]["sha"],
        "git_state": manifest["source"]["git_state"],
        "build_profile": "release",
        "binary_sha256": manifest["binary"]["sha256"],
    }
    total_chunks = (WORLD_WIDTH // CHUNK_SIZE) * (WORLD_HEIGHT // CHUNK_SIZE)
    for index, sample in enumerate(samples):
        require_exact_keys(sample, expected_keys, f"Pressure sample {index}")
        for key, expected in identity.items():
            if sample[key] != expected:
                raise ExperimentError(f"Pressure sample {index} {key} mismatch")
        if require_nonnegative_int(
            sample["sample_sequence"], f"Pressure sample {index} sequence"
        ) != index:
            raise ExperimentError("Pressure sample_sequence must be contiguous and zero-based")
        require_nonnegative_int(sample["sim_tick"], f"Pressure sample {index} sim_tick")
        phase = sample["phase"]
        reason = sample["reason"]
        if phase not in PRESSURE_PHASE_REASONS:
            raise ExperimentError(f"Pressure sample {index} phase is invalid")
        if reason not in PRESSURE_PHASE_REASONS[phase]:
            raise ExperimentError(f"Pressure sample {index} phase/reason mismatch")
        if sample["world"] != manifest["world"]:
            raise ExperimentError(f"Pressure sample {index} world mismatch")
        sleep = sample["sleep"]
        if not isinstance(sleep, dict):
            raise ExperimentError(f"Pressure sample {index} sleep must be an object")
        require_exact_keys(sleep, {"enabled", "threshold"}, f"Pressure sample {index} sleep")
        if not isinstance(sleep["enabled"], bool):
            raise ExperimentError(f"Pressure sample {index} sleep enabled must be boolean")
        require_nonnegative_int(sleep["threshold"], f"Pressure sample {index} sleep threshold")

        census = sample["census"]
        if not isinstance(census, dict):
            raise ExperimentError(f"Pressure sample {index} census must be an object")
        require_exact_keys(census, census_keys, f"Pressure sample {index} census")
        for key, value in census.items():
            require_nonnegative_int(value, f"Pressure sample {index} census {key}")
        if census["total_cells"] != WORLD_WIDTH * WORLD_HEIGHT:
            raise ExperimentError(f"Pressure sample {index} total_cells mismatch")
        if census["total_chunks"] != total_chunks:
            raise ExperimentError(f"Pressure sample {index} total_chunks mismatch")
        if census["runnable_chunks"] + census["sleeping_chunks"] != total_chunks:
            raise ExperimentError(f"Pressure sample {index} chunk-state census is incomplete")
        for key in ("active_chunks", "runnable_chunks", "sleeping_chunks"):
            if census[key] > total_chunks:
                raise ExperimentError(f"Pressure sample {index} {key} exceeds total chunks")
        for key in (
            "matter_active_cells",
            "thermal_active_cells",
            "pressure_active_cells",
            "reaction_active_cells",
        ):
            if census[key] > census["any_active_cells"]:
                raise ExperimentError(
                    f"Pressure sample {index} census {key} exceeds any_active_cells"
                )

        counts = sample["material_counts_by_id"]
        if not isinstance(counts, list) or len(counts) != 10:
            raise ExperimentError(f"Pressure sample {index} material_counts_by_id mismatch")
        for material_id, count in enumerate(counts):
            require_nonnegative_int(count, f"Pressure sample {index} material {material_id}")
        require_nonnegative_int(
            sample["invalid_material_count"],
            f"Pressure sample {index} invalid_material_count",
        )
        if sum(counts) + sample["invalid_material_count"] != WORLD_WIDTH * WORLD_HEIGHT:
            raise ExperimentError(f"Pressure sample {index} material census total mismatch")
        if sample["matter_count"] != sum(counts[1:]):
            raise ExperimentError(f"Pressure sample {index} matter_count mismatch")
        if sample["water_count"] != counts[4] or sample["steam_count"] != counts[6]:
            raise ExperimentError(f"Pressure sample {index} Water/Steam count mismatch")
        integer_keys = (
            "matter_count",
            "water_count",
            "steam_count",
            "relief_seam_wood_cells",
            "top_relief_seam_wood_cells",
            "bottom_relief_seam_wood_cells",
            "relief_seam_open_cells",
            "top_relief_seam_open_cells",
            "bottom_relief_seam_open_cells",
            "relief_seam_through_open_lanes",
            "top_relief_seam_through_open_lanes",
            "bottom_relief_seam_through_open_lanes",
            "top_relief_seam_combusting_cells",
            "bottom_relief_seam_combusting_cells",
            "relief_seam_combusting_cells",
            "top_relief_seam_flame_event_cells",
            "bottom_relief_seam_flame_event_cells",
            "relief_seam_flame_event_cells",
            "top_relief_seam_fuel_progress_sum",
            "top_relief_seam_fuel_progress_max",
            "bottom_relief_seam_fuel_progress_sum",
            "bottom_relief_seam_fuel_progress_max",
            "relief_seam_fuel_progress_sum",
            "relief_seam_fuel_progress_max",
            "top_relief_seam_adjacent_pressure_medium_cells",
            "bottom_relief_seam_adjacent_pressure_medium_cells",
            "relief_seam_adjacent_pressure_medium_cells",
            "steam_in_relief_seam_cells",
            "outside_chamber_steam_cells",
            "chamber_pressure_cell_count",
            "invalid_material_count",
            "nonfinite_temperature_count",
            "nonfinite_pressure_count",
            "changed_chunks",
            "wake_chunks",
            "wake_reason_or",
        )
        for key in integer_keys:
            require_nonnegative_int(sample[key], f"Pressure sample {index} {key}")
        if sample["relief_seam_wood_cells"] != (
            sample["top_relief_seam_wood_cells"]
            + sample["bottom_relief_seam_wood_cells"]
        ):
            raise ExperimentError(f"Pressure sample {index} seam Wood total mismatch")
        if sample["relief_seam_open_cells"] != (
            sample["top_relief_seam_open_cells"]
            + sample["bottom_relief_seam_open_cells"]
        ):
            raise ExperimentError(f"Pressure sample {index} seam open total mismatch")
        if sample["relief_seam_through_open_lanes"] != (
            sample["top_relief_seam_through_open_lanes"]
            + sample["bottom_relief_seam_through_open_lanes"]
        ):
            raise ExperimentError(
                f"Pressure sample {index} seam through-open lane total mismatch"
            )
        for suffix in ("combusting_cells", "flame_event_cells"):
            if sample[f"relief_seam_{suffix}"] != (
                sample[f"top_relief_seam_{suffix}"]
                + sample[f"bottom_relief_seam_{suffix}"]
            ):
                raise ExperimentError(
                    f"Pressure sample {index} combined seam {suffix} mismatch"
                )
        if sample["relief_seam_fuel_progress_sum"] != (
            sample["top_relief_seam_fuel_progress_sum"]
            + sample["bottom_relief_seam_fuel_progress_sum"]
        ):
            raise ExperimentError(
                f"Pressure sample {index} combined seam fuel-progress sum mismatch"
            )
        if sample["relief_seam_fuel_progress_max"] != max(
            sample["top_relief_seam_fuel_progress_max"],
            sample["bottom_relief_seam_fuel_progress_max"],
        ):
            raise ExperimentError(
                f"Pressure sample {index} combined seam fuel-progress max mismatch"
            )
        if sample["relief_seam_adjacent_pressure_medium_cells"] != (
            sample["top_relief_seam_adjacent_pressure_medium_cells"]
            + sample["bottom_relief_seam_adjacent_pressure_medium_cells"]
        ):
            raise ExperimentError(
                f"Pressure sample {index} combined adjacent pressure-medium count mismatch"
            )
        adjacent_pressures = {}
        for region in ("top", "bottom", ""):
            prefix = f"{region}_" if region else ""
            key = f"{prefix}relief_seam_max_adjacent_pressure"
            value = require_finite_number(
                sample[key], f"Pressure sample {index} {key}"
            )
            if value < 0:
                raise ExperimentError(f"Pressure sample {index} {key} is negative")
            adjacent_pressures[region] = value
        if not pressure_float_equal(
            adjacent_pressures[""],
            max(adjacent_pressures["top"], adjacent_pressures["bottom"]),
        ):
            raise ExperimentError(
                f"Pressure sample {index} combined max adjacent pressure mismatch"
            )
        if sample["relief_seam_wood_cells"] + sample["relief_seam_open_cells"] != 576:
            raise ExperimentError(f"Pressure sample {index} seam census does not total 576")
        if sample["top_relief_seam_wood_cells"] + sample["top_relief_seam_open_cells"] != 384:
            raise ExperimentError(f"Pressure sample {index} top seam census mismatch")
        if (
            sample["bottom_relief_seam_wood_cells"]
            + sample["bottom_relief_seam_open_cells"]
            != 192
        ):
            raise ExperimentError(f"Pressure sample {index} bottom seam census mismatch")
        if sample["steam_in_relief_seam_cells"] > sample["relief_seam_open_cells"]:
            raise ExperimentError(f"Pressure sample {index} seam Steam exceeds open cells")
        if sample["top_relief_seam_through_open_lanes"] > 48:
            raise ExperimentError(f"Pressure sample {index} top through-open lanes exceed 48")
        if sample["bottom_relief_seam_through_open_lanes"] > 24:
            raise ExperimentError(f"Pressure sample {index} bottom through-open lanes exceed 24")
        if sample["top_relief_seam_through_open_lanes"] * 8 > sample[
            "top_relief_seam_open_cells"
        ]:
            raise ExperimentError(
                f"Pressure sample {index} top through-open lanes exceed damaged cells"
            )
        if sample["bottom_relief_seam_through_open_lanes"] * 8 > sample[
            "bottom_relief_seam_open_cells"
        ]:
            raise ExperimentError(
                f"Pressure sample {index} bottom through-open lanes exceed damaged cells"
            )
        if sample["outside_chamber_steam_cells"] > sample["steam_count"]:
            raise ExperimentError(f"Pressure sample {index} exterior Steam exceeds inventory")
        if sample["chamber_pressure_cell_count"] != 29_920:
            raise ExperimentError(f"Pressure sample {index} chamber cell count mismatch")
        mean_pressure = require_finite_number(
            sample["chamber_mean_pressure"], f"Pressure sample {index} chamber mean"
        )
        max_pressure = require_finite_number(
            sample["chamber_max_pressure"], f"Pressure sample {index} chamber max"
        )
        if mean_pressure < 0 or max_pressure < mean_pressure:
            raise ExperimentError(f"Pressure sample {index} chamber pressure ordering is invalid")
        for key in ("state_hash", "physical_state_hash"):
            if not isinstance(sample[key], str) or not STATE_HASH.fullmatch(sample[key]):
                raise ExperimentError(f"Pressure sample {index} {key} is invalid")
        if (sample["wake_chunks"] == 0) != (sample["wake_reason_or"] == 0):
            raise ExperimentError(
                f"Pressure sample {index} wake census and wake_reason_or disagree"
            )


def validate_samples(samples: list[dict[str, Any]], manifest: dict[str, Any]) -> None:
    contract = contract_for_manifest(manifest)
    if contract is SAND_CONTRACT:
        validate_sand_samples(samples, manifest)
    elif contract is WATER_CONTRACT:
        validate_water_samples(samples, manifest)
    elif contract is FIRE_CONTRACT:
        validate_fire_samples(samples, manifest)
    elif contract is PRESSURE_CONTRACT:
        validate_pressure_samples(samples, manifest)
    else:
        raise ExperimentError(f"unsupported sample contract: {contract.scenario}")


def validate_events(events: list[dict[str, Any]], manifest: dict[str, Any]) -> None:
    contract = contract_for_manifest(manifest)
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
        if contract.records_run_mode:
            required.add("scenario")
        require_exact_keys(event, required, f"event {index}")
        if event["schema_version"] != contract.telemetry_schema:
            raise ExperimentError(f"event {index} schema_version mismatch")
        if event["experiment_id"] != manifest["experiment_id"]:
            raise ExperimentError(f"event {index} experiment_id mismatch")
        if event["run_id"] != manifest["run_id"]:
            raise ExperimentError(f"event {index} run_id mismatch")
        if contract.records_run_mode and event["scenario"] != contract.scenario:
            raise ExperimentError(f"event {index} scenario mismatch")
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


def verdict_from_predicates(
    predicates: dict[str, Any], contract: ScenarioContract = SAND_CONTRACT
) -> str:
    statuses = {predicate["status"] for predicate in predicates.values()}
    if "fail" in statuses:
        return "FAIL"
    if "unknown" in statuses:
        return contract.needs_human_verdict
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


def validate_sand_telemetry(
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
    recomputed_verdict = verdict_from_predicates(analysis["predicates"], SAND_CONTRACT)
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


WATER_DIAGNOSTIC_REASONS = frozenset(
    {"early-flow", "diagnostic-cadence", "max-tick"}
)


def water_diagnostic_samples(samples: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [
        sample
        for sample in samples
        if sample["phase"] == "flowing"
        and sample["reason"] in WATER_DIAGNOSTIC_REASONS
    ]


def first_matching(
    samples: Iterable[dict[str, Any]], predicate: Any
) -> dict[str, Any] | None:
    return next((sample for sample in samples if predicate(sample)), None)


def sample_identity(sample: dict[str, Any] | None) -> tuple[int | None, int | None]:
    if sample is None:
        return None, None
    return sample["sim_tick"], sample["sample_sequence"]


def confirmed_water_all_sleep_streak(
    samples: list[dict[str, Any]], required: int
) -> tuple[list[dict[str, Any]] | None, list[dict[str, Any]], list[dict[str, Any]]]:
    streak: list[dict[str, Any]] = []
    confirmed: list[dict[str, Any]] | None = None
    starts: list[dict[str, Any]] = []
    breaks: list[dict[str, Any]] = []
    for sample in water_diagnostic_samples(samples):
        if sample_is_all_sleep(sample):
            if not streak:
                starts.append(sample)
            streak.append(sample)
            if confirmed is None and len(streak) == required:
                confirmed = list(streak)
                break
        elif streak:
            breaks.append(sample)
            streak.clear()
    return confirmed, starts, breaks


def confirmed_water_plateau_streak(
    samples: list[dict[str, Any]], required: int
) -> tuple[list[dict[str, Any]] | None, list[dict[str, Any]], list[dict[str, Any]]]:
    streak: list[dict[str, Any]] = []
    confirmed: list[dict[str, Any]] | None = None
    starts: list[dict[str, Any]] = []
    breaks: list[dict[str, Any]] = []
    for sample in water_diagnostic_samples(samples):
        eligible = sample["changed_chunks"] == 0 and sample["wake_chunks"] == 0
        same_hash = bool(streak) and sample["state_hash"] == streak[-1]["state_hash"]
        if not eligible:
            if streak:
                breaks.append(sample)
                streak.clear()
            continue
        if streak and not same_hash:
            breaks.append(sample)
            streak.clear()
        if not streak:
            starts.append(sample)
        streak.append(sample)
        if confirmed is None and len(streak) == required:
            confirmed = list(streak)
            break
    return confirmed, starts, breaks


def require_water_predicate_statuses(
    predicates: dict[str, Any], expected: dict[str, str]
) -> None:
    for name, expected_status in expected.items():
        actual = predicates[name]["status"]
        if actual != expected_status:
            raise ExperimentError(
                f"Water predicate {name} status {actual!r} disagrees with raw telemetry "
                f"({expected_status!r})"
            )


def require_event_sample_identity(
    event: dict[str, Any], sample: dict[str, Any], label: str
) -> None:
    if (
        event["sim_tick"] != sample["sim_tick"]
        or event["sample_sequence"] != sample["sample_sequence"]
    ):
        raise ExperimentError(f"Water event {label} identity disagrees with telemetry")


def validate_water_event_contract(
    events: list[dict[str, Any]],
    samples: list[dict[str, Any]],
    lifecycle: dict[str, Any],
    first_movement: dict[str, Any] | None,
    first_cross: dict[str, Any] | None,
    first_destination: dict[str, Any] | None,
    first_sleeping: dict[str, Any] | None,
    peak_updates: list[dict[str, Any]],
    spread_updates: list[dict[str, Any]],
    all_sleep_starts: list[dict[str, Any]],
    all_sleep_breaks: list[dict[str, Any]],
    all_sleep_streak: list[dict[str, Any]] | None,
    plateau_starts: list[dict[str, Any]],
    plateau_breaks: list[dict[str, Any]],
    plateau_streak: list[dict[str, Any]] | None,
    terminal: dict[str, Any],
    post_settle: list[dict[str, Any]],
) -> None:
    allowed = WATER_ALWAYS_EVENTS | WATER_OPTIONAL_EVENTS
    by_name: dict[str, list[dict[str, Any]]] = {}
    by_sequence = {sample["sample_sequence"]: sample for sample in samples}
    for event in events:
        if event["event"] not in allowed:
            raise ExperimentError(f"unsupported Water event {event['event']!r}")
        by_name.setdefault(event["event"], []).append(event)
        sequence = event["sample_sequence"]
        if sequence is not None:
            sample = by_sequence.get(sequence)
            if sample is None or sample["sim_tick"] != event["sim_tick"]:
                raise ExperimentError(
                    f"Water event {event['event']} does not bind to its telemetry sample"
                )

    for name in WATER_ALWAYS_EVENTS:
        if len(by_name.get(name, [])) != 1:
            raise ExperimentError(f"Water event {name} must occur exactly once")
    mandatory_order = (
        "lifecycle_started",
        "pristine_reset_completed",
        "tick0_captured",
        "tick1_captured",
        "terminal_selected",
        "reset_started",
        "reset_comparison_completed",
        "worker_completed",
    )
    mandatory_positions = [by_name[name][0]["event_sequence"] for name in mandatory_order]
    if mandatory_positions != sorted(mandatory_positions):
        raise ExperimentError("Water lifecycle events are out of order")

    identity_events = {
        "tick0_captured": samples[0],
        "tick1_captured": samples[1],
        "terminal_selected": terminal,
        "reset_started": samples[-2],
        "reset_comparison_completed": samples[-1],
        "worker_completed": samples[-1],
    }
    for name, sample in identity_events.items():
        require_event_sample_identity(by_name[name][0], sample, name)
    for name in ("lifecycle_started", "pristine_reset_completed"):
        event = by_name[name][0]
        if event["sim_tick"] != 0 or event["sample_sequence"] is not None:
            raise ExperimentError(f"Water event {name} must be the pre-sample tick0 event")

    optional_first = {
        "water_movement_observed": first_movement,
        "cross_chunk_flow_observed": first_cross,
        "destination_arrival_observed": first_destination,
        "first_sleeping_chunk_observed": first_sleeping,
    }
    for name, sample in optional_first.items():
        found = by_name.get(name, [])
        if sample is None:
            if found:
                raise ExperimentError(f"Water event {name} exists without its signal")
        elif len(found) != 1:
            raise ExperimentError(f"Water event {name} must occur exactly once")
        else:
            require_event_sample_identity(found[0], sample, name)

    repeated = {
        "new_peak_active": peak_updates,
        "new_max_destination_spread": spread_updates,
        "all_sleep_observed": all_sleep_starts,
        "all_sleep_streak_broken": all_sleep_breaks,
        "stable_plateau_observed": plateau_starts,
        "stable_plateau_streak_broken": plateau_breaks,
    }
    for name, expected_samples in repeated.items():
        found = by_name.get(name, [])
        if len(found) != len(expected_samples):
            raise ExperimentError(f"Water event {name} cardinality disagrees with telemetry")
        for event, sample in zip(found, expected_samples, strict=True):
            require_event_sample_identity(event, sample, name)

    confirmation_events = {
        "all_sleep_confirmed": None if all_sleep_streak is None else all_sleep_streak[-1],
        "stable_plateau_confirmed": None if plateau_streak is None else plateau_streak[-1],
    }
    for name, sample in confirmation_events.items():
        found = by_name.get(name, [])
        if sample is None:
            if found:
                raise ExperimentError(f"Water event {name} exists without confirmation")
        elif len(found) != 1:
            raise ExperimentError(f"Water event {name} must occur exactly once")
        else:
            require_event_sample_identity(found[0], sample, name)

    post_events = by_name.get("post_settle_confirmation_completed", [])
    window_expected = lifecycle["terminal_reason"] != "max-ticks"
    if len(post_events) != int(window_expected):
        raise ExperimentError(
            "post_settle_confirmation_completed event disagrees with terminal reason"
        )
    if post_events:
        require_event_sample_identity(
            post_events[0], post_settle[-1], "post_settle_confirmation_completed"
        )
        if not (
            by_name["terminal_selected"][0]["event_sequence"]
            < post_events[0]["event_sequence"]
            < by_name["reset_started"][0]["event_sequence"]
        ):
            raise ExperimentError("Water post-settle completion event is out of order")


def validate_water_frame_contract(
    frames: list[dict[str, Any]],
    samples: list[dict[str, Any]],
    analysis: dict[str, Any],
    first_movement: dict[str, Any] | None,
    first_cross: dict[str, Any] | None,
    first_destination: dict[str, Any] | None,
    first_sleeping: dict[str, Any] | None,
    peak: dict[str, Any],
    max_spread: dict[str, Any] | None,
    terminal: dict[str, Any],
    post_settle: list[dict[str, Any]],
) -> None:
    by_kind: dict[str, list[dict[str, Any]]] = {}
    kinds_by_sequence: dict[int, list[str]] = {}
    by_sequence = {sample["sample_sequence"]: sample for sample in samples}
    for frame in frames:
        by_kind.setdefault(frame["kind"], []).append(frame)
        sequence = frame["sample_sequence"]
        kinds_by_sequence.setdefault(sequence, []).append(frame["kind"])
        sample = by_sequence.get(sequence)
        if sample is None:
            raise ExperimentError("Water frame sample_sequence is absent from telemetry")
        if sample["sim_tick"] != frame["sim_tick"] or sample["state_hash"] != frame["state_hash"]:
            raise ExperimentError("Water frame identity disagrees with telemetry")

    for kind, grouped in by_kind.items():
        if kind != "diagnostic-observation" and len(grouped) > 1:
            raise ExperimentError(f"Water named frame kind {kind} occurs more than once")
    for kinds in kinds_by_sequence.values():
        if len(kinds) == 1:
            continue
        if (
            len(kinds) != 2
            or "peak-active" not in kinds
            or "diagnostic-observation" in kinds
        ):
            raise ExperimentError(
                "Water frame sample identities must be unique except one analysis-bound "
                "peak-active/named-frame alias"
            )

    for required_kind in ("tick0", "tick1", "late", "terminal", "reset"):
        if len(by_kind.get(required_kind, [])) != 1:
            raise ExperimentError(f"Water frame kind {required_kind} must occur exactly once")
    expected_post_frames = 1 if post_settle else 0
    if len(by_kind.get("post-settle", [])) != expected_post_frames:
        raise ExperimentError("Water post-settle frame cardinality disagrees with lifecycle")

    expected_by_kind: dict[str, dict[str, Any] | None] = {
        "tick0": samples[0],
        "tick1": samples[1],
        "first-movement": first_movement,
        "peak-active": peak,
        "cross-chunk-flow": first_cross,
        "destination-arrival": first_destination,
        "max-destination-spread": max_spread,
        "first-sleeping-chunk": first_sleeping,
        "terminal": terminal,
        "post-settle": post_settle[-1] if post_settle else None,
        "reset": samples[-1],
    }
    diagnostics = water_diagnostic_samples(samples)
    terminal_index = next(
        (index for index, sample in enumerate(diagnostics) if sample is terminal), None
    )
    expected_by_kind["late"] = (
        diagnostics[terminal_index - 1]
        if terminal_index is not None and terminal_index > 0
        else None
    )
    for kind, expected_sample in expected_by_kind.items():
        for frame in by_kind.get(kind, []):
            if expected_sample is None:
                raise ExperimentError(f"Water frame kind {kind} exists without its milestone")
            if frame["sample_sequence"] != expected_sample["sample_sequence"]:
                raise ExperimentError(f"Water frame kind {kind} binds the wrong sample")

    fallback = by_kind.get("diagnostic-observation", [])
    if fallback:
        if analysis["verdict"] == "PASS" or len(frames) != 8:
            raise ExperimentError(
                "diagnostic-observation frames may only fill a non-PASS run to eight frames"
            )
        for frame in fallback:
            sample = by_sequence[frame["sample_sequence"]]
            if sample not in diagnostics:
                raise ExperimentError(
                    "diagnostic-observation frame must bind a flowing diagnostic sample"
                )


def validate_water_telemetry(
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
    validate_water_analysis(analysis, manifest)
    frames = validate_frames(frames_doc, manifest, run_dir)
    validate_water_samples(samples, manifest)
    validate_events(events, manifest)
    if analysis["raw_frame_count"] != len(frames):
        raise ExperimentError("analysis raw_frame_count does not match frames.json")
    if analysis["lifecycle"]["sample_count"] != len(samples):
        raise ExperimentError("analysis lifecycle sample_count does not match samples.jsonl")
    recomputed_verdict = verdict_from_predicates(
        analysis["predicates"], WATER_CONTRACT
    )
    if analysis["verdict"] != recomputed_verdict:
        raise ExperimentError(
            "analysis verdict disagrees with its ten Water predicate statuses"
        )
    for index, sample in enumerate(samples):
        if sample["sleep"] != analysis["sleep"]:
            raise ExperimentError(f"sample {index} sleep settings disagree with analysis")

    if len(samples) < 3:
        raise ExperimentError("Water telemetry must contain tick0, tick1, and reset samples")
    tick0 = samples[0]
    tick1 = samples[1]
    reset = samples[-1]
    if (tick0["sim_tick"], tick0["phase"], tick0["reason"]) != (0, "initial", "tick0"):
        raise ExperimentError("Water telemetry must begin with initial tick0")
    if (tick1["sim_tick"], tick1["phase"], tick1["reason"]) != (1, "flowing", "tick1"):
        raise ExperimentError("Water telemetry sample 1 must be flowing tick1")
    reset_samples = [sample for sample in samples if sample["phase"] == "reset"]
    if len(reset_samples) != 1 or reset is not reset_samples[0]:
        raise ExperimentError("Water telemetry must end with exactly one reset sample")
    if reset["sim_tick"] != 0 or reset["reason"] != "programmatic-r-equivalent":
        raise ExperimentError("Water reset sample must be the programmatic tick0 equivalent")
    pre_reset = samples[:-1]
    if any(sample["phase"] == "initial" for sample in pre_reset[1:]):
        raise ExperimentError("Water initial phase may only appear at sample 0")
    if any(
        later["sim_tick"] <= earlier["sim_tick"]
        for earlier, later in zip(pre_reset, pre_reset[1:])
    ):
        raise ExperimentError("Water pre-reset sim ticks must be strictly increasing")
    post_settle = [
        sample for sample in pre_reset if sample["phase"] == "post-settle-confirmation"
    ]
    if post_settle and any(
        sample["phase"] != "post-settle-confirmation"
        for sample in pre_reset[pre_reset.index(post_settle[0]) :]
    ):
        raise ExperimentError("Water post-settle phase must be contiguous and final")

    expected_baseline = {
        key: tick0[key]
        for key in (
            "matter_count",
            "water_count",
            "oil_count",
            "water_y_sum",
            "oil_y_sum",
            "water_occupied_chunks",
            "oil_occupied_chunks",
            "bottom_chunk_row_water_cells",
            "destination_water_cells",
            "destination_spread_x",
        )
    }
    if analysis["baseline"] != expected_baseline:
        raise ExperimentError("Water analysis baseline does not match tick0 telemetry")
    if any(
        sample["initial_water_cells_vacated"] > tick0["water_count"]
        for sample in pre_reset
    ):
        raise ExperimentError("Water vacated-cell count exceeds the tick0 Water mask")

    signal_samples = [
        sample for sample in pre_reset if sample["phase"] in {"initial", "flowing"}
    ]
    flowing = [sample for sample in signal_samples if sample["phase"] == "flowing"]
    first_movement = first_matching(
        flowing,
        lambda sample: sample["water_outside_initial_mask"] > 0
        and sample["initial_water_cells_vacated"] > 0,
    )
    first_cross = first_matching(
        flowing, lambda sample: sample["bottom_chunk_row_water_cells"] > 0
    )
    first_destination = first_matching(
        flowing, lambda sample: sample["destination_water_cells"] > 0
    )
    first_sleeping = first_matching(
        pre_reset, lambda sample: sample["census"]["sleeping_chunks"] > 0
    )

    peak = signal_samples[0]
    peak_updates: list[dict[str, Any]] = []
    for sample in signal_samples[1:]:
        if sample["census"]["any_active_cells"] > peak["census"]["any_active_cells"]:
            peak = sample
            peak_updates.append(sample)
    max_active_chunks = max(sample["census"]["active_chunks"] for sample in signal_samples)
    max_bottom = max(sample["bottom_chunk_row_water_cells"] for sample in signal_samples)
    max_destination = max(sample["destination_water_cells"] for sample in signal_samples)
    max_outside_outer_basin = max(
        sample["water_outside_outer_basin_cells"] for sample in pre_reset
    )
    spread_updates: list[dict[str, Any]] = []
    current_spread = tick0["destination_spread_x"]
    max_spread = tick0 if current_spread != 0 else None
    for sample in signal_samples[1:]:
        if sample["destination_spread_x"] > current_spread:
            current_spread = sample["destination_spread_x"]
            spread_updates.append(sample)
            max_spread = sample

    lifecycle = analysis["lifecycle"]
    by_sequence = {sample["sample_sequence"]: sample for sample in samples}
    terminal = by_sequence.get(lifecycle["terminal_sample_sequence"])
    if terminal is None or terminal["sim_tick"] != lifecycle["terminal_sim_tick"]:
        raise ExperimentError("Water terminal identity is absent from telemetry")
    if terminal["phase"] != "flowing" or terminal["reason"] not in WATER_DIAGNOSTIC_REASONS:
        raise ExperimentError("Water terminal must bind a flowing diagnostic sample")
    expected_diagnostic_ticks = [2]
    expected_diagnostic_ticks.extend(
        range(DIAGNOSTIC_INTERVAL, terminal["sim_tick"] + 1, DIAGNOSTIC_INTERVAL)
    )
    if terminal["sim_tick"] == MAX_TICKS and MAX_TICKS not in expected_diagnostic_ticks:
        expected_diagnostic_ticks.append(MAX_TICKS)
    expected_diagnostic_ticks = sorted(set(expected_diagnostic_ticks))
    actual_diagnostics = water_diagnostic_samples(pre_reset)
    if [sample["sim_tick"] for sample in actual_diagnostics] != expected_diagnostic_ticks:
        raise ExperimentError("Water diagnostic cadence is incomplete or contains extra samples")
    for sample in actual_diagnostics:
        expected_reason = (
            "early-flow"
            if sample["sim_tick"] == 2
            else "max-tick"
            if sample["sim_tick"] == MAX_TICKS
            else "diagnostic-cadence"
        )
        if sample["reason"] != expected_reason:
            raise ExperimentError("Water diagnostic reason disagrees with its sim tick")

    all_sleep_streak, all_sleep_starts, all_sleep_breaks = (
        confirmed_water_all_sleep_streak(pre_reset, CONSECUTIVE_ALL_SLEEP)
    )
    plateau_streak, plateau_starts, plateau_breaks = confirmed_water_plateau_streak(
        pre_reset, CONSECUTIVE_STABLE_PLATEAU
    )
    terminal_reason = lifecycle["terminal_reason"]
    if terminal_reason == "all-sleep":
        if all_sleep_streak is None or terminal is not all_sleep_streak[-1]:
            raise ExperimentError("all-sleep terminal disagrees with the three-sample streak")
        first_all_sleep = all_sleep_streak[0]
        if (
            lifecycle["first_all_sleep_sim_tick"],
            lifecycle["first_all_sleep_sample_sequence"],
            lifecycle["confirmed_all_sleep_sim_tick"],
        ) != (
            first_all_sleep["sim_tick"],
            first_all_sleep["sample_sequence"],
            terminal["sim_tick"],
        ):
            raise ExperimentError("Water all-sleep lifecycle identity mismatch")
    elif lifecycle["confirmed_all_sleep_sim_tick"] is not None:
        raise ExperimentError("non-all-sleep terminal records all-sleep confirmation")
    elif (
        lifecycle["first_all_sleep_sim_tick"] is not None
        or lifecycle["first_all_sleep_sample_sequence"] is not None
    ):
        raise ExperimentError("non-all-sleep terminal records an all-sleep streak identity")
    if terminal_reason != "all-sleep" and all_sleep_streak is not None:
        raise ExperimentError("confirmed all-sleep streak did not receive terminal precedence")
    if plateau_streak is not None:
        if terminal is not plateau_streak[-1] or terminal_reason not in {
            "all-sleep",
            "stable-plateau",
        }:
            raise ExperimentError(
                "plateau confirmation does not coincide with its selected terminal"
            )
        first_plateau = plateau_streak[0]
        if (
            lifecycle["first_stable_plateau_sim_tick"],
            lifecycle["first_stable_plateau_sample_sequence"],
            lifecycle["confirmed_stable_plateau_sim_tick"],
        ) != (
            first_plateau["sim_tick"],
            first_plateau["sample_sequence"],
            terminal["sim_tick"],
        ):
            raise ExperimentError("Water stable-plateau lifecycle identity mismatch")
    elif (
        lifecycle["confirmed_stable_plateau_sim_tick"] is not None
        or lifecycle["first_stable_plateau_sim_tick"] is not None
        or lifecycle["first_stable_plateau_sample_sequence"] is not None
    ):
        raise ExperimentError("Water lifecycle records plateau identity without confirmation")
    if terminal_reason == "stable-plateau" and plateau_streak is None:
        raise ExperimentError("stable-plateau terminal lacks the eight-sample streak")
    if terminal_reason == "max-ticks":
        if terminal["sim_tick"] != MAX_TICKS or terminal["reason"] != "max-tick":
            raise ExperimentError("max-ticks terminal does not bind max-tick telemetry")
        if post_settle:
            raise ExperimentError("max-ticks terminal must not have a post-settle window")
    else:
        if len(post_settle) != POST_SLEEP_TICKS:
            raise ExperimentError("settled Water terminal must have exactly 180 post samples")
        expected_ticks = list(
            range(terminal["sim_tick"] + 1, terminal["sim_tick"] + POST_SLEEP_TICKS + 1)
        )
        if [sample["sim_tick"] for sample in post_settle] != expected_ticks:
            raise ExperimentError("Water post-settle ticks must be contiguous")
        if lifecycle["post_settle_end_tick"] != expected_ticks[-1]:
            raise ExperimentError("Water post_settle_end_tick disagrees with telemetry")

    final_pre_reset = pre_reset[-1]
    raw_post_changes = sum(
        sample["physical_state_hash"] != terminal["physical_state_hash"]
        or sample["changed_chunks"] != 0
        for sample in post_settle
    )
    raw_post_wakes = sum(
        sample["wake_chunks"] != 0
        or (terminal_reason == "all-sleep" and not sample_is_all_sleep(sample))
        for sample in post_settle
    )
    if lifecycle["post_settle_change_ticks"] != raw_post_changes:
        raise ExperimentError("Water post-settle change count disagrees with raw telemetry")
    if lifecycle["post_settle_wake_ticks"] != raw_post_wakes:
        raise ExperimentError("Water post-settle wake count disagrees with telemetry")

    metrics = analysis["metrics"]
    expected_metrics = {
        "peak_active_cells": peak["census"]["any_active_cells"],
        "peak_active_chunks": max_active_chunks,
        "peak_active_sim_tick": peak["sim_tick"],
        "peak_active_sample_sequence": peak["sample_sequence"],
        "first_water_movement_tick": sample_identity(first_movement)[0],
        "first_water_movement_sample_sequence": sample_identity(first_movement)[1],
        "first_cross_chunk_flow_tick": sample_identity(first_cross)[0],
        "first_cross_chunk_flow_sample_sequence": sample_identity(first_cross)[1],
        "first_destination_arrival_tick": sample_identity(first_destination)[0],
        "first_destination_arrival_sample_sequence": sample_identity(first_destination)[1],
        "first_sleeping_chunk_tick": sample_identity(first_sleeping)[0],
        "first_sleeping_chunk_sample_sequence": sample_identity(first_sleeping)[1],
        "max_bottom_chunk_row_water_cells": max_bottom,
        "max_destination_water_cells": max_destination,
        "max_destination_spread_x": current_spread,
        "max_destination_spread_tick": sample_identity(max_spread)[0],
        "max_destination_spread_sample_sequence": sample_identity(max_spread)[1],
        "max_water_outside_outer_basin_cells": max_outside_outer_basin,
        "final_matter_count": final_pre_reset["matter_count"],
        "final_water_count": final_pre_reset["water_count"],
        "final_oil_count": final_pre_reset["oil_count"],
        "final_water_occupied_chunks": final_pre_reset["water_occupied_chunks"],
        "final_oil_occupied_chunks": final_pre_reset["oil_occupied_chunks"],
        "final_sleeping_chunks": final_pre_reset["census"]["sleeping_chunks"],
        "final_water_outside_outer_basin_cells": final_pre_reset[
            "water_outside_outer_basin_cells"
        ],
        "final_active_water_empty_surface_cells": final_pre_reset[
            "active_water_empty_surface_cells"
        ],
        "final_active_water_oil_interface_cells": final_pre_reset[
            "active_water_oil_interface_cells"
        ],
        "final_active_other_cells": final_pre_reset["active_other_cells"],
        "matter_count_delta": final_pre_reset["matter_count"] - tick0["matter_count"],
        "water_count_delta": final_pre_reset["water_count"] - tick0["water_count"],
        "oil_count_delta": final_pre_reset["oil_count"] - tick0["oil_count"],
        "post_settle_state_changes": raw_post_changes,
        "post_settle_spontaneous_wakes": lifecycle["post_settle_wake_ticks"],
    }
    for key, expected in expected_metrics.items():
        if metrics[key] != expected:
            raise ExperimentError(f"Water analysis metric {key} disagrees with telemetry")
    if metrics["active_cell_classification_rule"] != WATER_ACTIVE_CLASSIFICATION_RULE:
        raise ExperimentError("Water analysis active-cell classification rule mismatch")
    final_active_classified = (
        metrics["final_active_water_empty_surface_cells"]
        + metrics["final_active_water_oil_interface_cells"]
        + metrics["final_active_other_cells"]
    )
    if final_active_classified != final_pre_reset["census"]["any_active_cells"]:
        raise ExperimentError(
            "Water final active-cell classifications do not partition any_active_cells"
        )

    conserved = all(
        (sample["matter_count"], sample["water_count"], sample["oil_count"])
        == (tick0["matter_count"], tick0["water_count"], tick0["oil_count"])
        for sample in pre_reset
    )
    no_invalid = all(sample["invalid_material_count"] == 0 for sample in pre_reset)
    no_nonfinite = all(
        sample["nonfinite_temperature_count"] == 0
        and sample["nonfinite_pressure_count"] == 0
        for sample in pre_reset
    )
    settled_window = terminal_reason != "max-ticks"
    post_stable = (
        settled_window
        and len(post_settle) == POST_SLEEP_TICKS
        and lifecycle["post_settle_change_ticks"] == 0
        and lifecycle["post_settle_wake_ticks"] == 0
    )
    reset_observable_equal = all(
        reset[key] == tick0[key]
        for key in (
            "world",
            "sleep",
            "census",
            "material_counts_by_id",
            "matter_count",
            "water_count",
            "oil_count",
            "water_y_sum",
            "water_min_y",
            "water_max_y",
            "oil_y_sum",
            "oil_min_y",
            "oil_max_y",
            "water_occupied_chunks",
            "oil_occupied_chunks",
            "water_outside_initial_mask",
            "water_outside_outer_basin_cells",
            "initial_water_cells_vacated",
            "bottom_chunk_row_water_cells",
            "destination_water_cells",
            "destination_spread_x",
            "invalid_material_count",
            "nonfinite_temperature_count",
            "nonfinite_pressure_count",
            "state_hash",
            "physical_state_hash",
            "active_water_empty_surface_cells",
            "active_water_oil_interface_cells",
            "active_other_cells",
        )
    )
    if metrics["reset_exact_equivalence"] and not reset_observable_equal:
        raise ExperimentError("PASS exact Water reset disagrees with observable telemetry")
    expected_predicates = {
        "actual_water_movement": "pass" if first_movement is not None else "unknown",
        "cross_chunk_flow": "pass" if first_cross is not None else "unknown",
        "destination_arrival": "pass" if first_destination is not None else "unknown",
        "water_conservation": "pass" if conserved else "fail",
        "no_invalid_materials": "pass" if no_invalid else "fail",
        "no_nonfinite_fields": "pass" if no_nonfinite else "fail",
        "stable_bulk_before_max": (
            "pass"
            if terminal_reason == "all-sleep"
            and lifecycle["confirmed_all_sleep_sim_tick"] < MAX_TICKS
            else "unknown"
        ),
        "post_settle_stable": (
            "unknown" if not settled_window else "pass" if post_stable else "fail"
        ),
        "exact_reset": "pass" if metrics["reset_exact_equivalence"] else "fail",
        "water_outside_outer_basin_cells": (
            "pass" if max_outside_outer_basin == 0 else "fail"
        ),
    }
    require_water_predicate_statuses(analysis["predicates"], expected_predicates)

    validate_water_event_contract(
        events,
        samples,
        lifecycle,
        first_movement,
        first_cross,
        first_destination,
        first_sleeping,
        peak_updates,
        spread_updates,
        all_sleep_starts,
        all_sleep_breaks,
        all_sleep_streak,
        plateau_starts,
        plateau_breaks,
        plateau_streak,
        terminal,
        post_settle,
    )
    validate_water_frame_contract(
        frames,
        samples,
        analysis,
        first_movement,
        first_cross,
        first_destination,
        first_sleeping,
        peak,
        max_spread,
        terminal,
        post_settle,
    )
    return analysis, frames_doc, samples, events


FIRE_DIAGNOSTIC_REASONS = frozenset(
    {"early-diagnostic", "diagnostic-cadence", "max-tick"}
)
FIRE_ALWAYS_EVENTS = frozenset(
    {
        "lifecycle_started",
        "pristine_reset_completed",
        "tick0_captured",
        "tick1_captured",
        "terminal_selected",
        "reset_started",
        "reset_comparison_completed",
        "worker_completed",
    }
)
FIRE_OPTIONAL_EVENTS = frozenset(
    {
        "combustion_observed",
        "smoke_generated",
        "heat_propagated",
        "phase_transition_observed",
        "fuel_substantially_consumed",
        "new_peak_reaction",
        "new_peak_thermal",
        "reaction_zero_streak_started",
        "reaction_zero_streak_broken",
        "reaction_zero_confirmed",
        "post_reaction_confirmation_completed",
    }
)
FIRE_FRAME_REASONS = {
    "tick0": "pristine-reset",
    "tick1": "after-one-production-tick",
    "first-combustion": "both-fuels-production-combustion",
    "first-smoke": "smoke-count-above-tick0",
    "peak-reaction": "highest-observed-reaction-cells",
    "peak-thermal": "highest-observed-thermal-cells",
    "first-phase-transition": "phase-inventory-differs-from-tick0",
    "fuel-substantially-consumed": "at-least-25-percent-initial-fuel-consumed",
    "reaction-zero": "first-sample-of-confirmed-reaction-zero-streak",
    "post-reaction-tail": "post-reaction-confirmation-complete",
    "reset": "programmatic-r-equivalent",
    "diagnostic-observation": "minimum-evidence-observation",
}


def fire_diagnostic_samples(samples: Iterable[dict[str, Any]]) -> list[dict[str, Any]]:
    return [
        sample
        for sample in samples
        if sample["phase"] == "reacting" and sample["reason"] in FIRE_DIAGNOSTIC_REASONS
    ]


def confirmed_fire_reaction_zero_streak(
    samples: list[dict[str, Any]], first_combustion: dict[str, Any] | None
) -> tuple[list[dict[str, Any]] | None, list[dict[str, Any]], list[dict[str, Any]]]:
    if first_combustion is None:
        return None, [], []
    streak: list[dict[str, Any]] = []
    starts: list[dict[str, Any]] = []
    breaks: list[dict[str, Any]] = []
    for sample in fire_diagnostic_samples(samples):
        if sample["sim_tick"] < first_combustion["sim_tick"]:
            continue
        if sample["census"]["reaction_active_cells"] == 0:
            if not streak:
                starts.append(sample)
            streak.append(sample)
            if len(streak) == CONSECUTIVE_REACTION_ZERO:
                return list(streak), starts, breaks
        elif streak:
            breaks.append(sample)
            streak.clear()
    return None, starts, breaks


def require_fire_predicate_statuses(
    predicates: dict[str, Any], expected: dict[str, str]
) -> None:
    for name, expected_status in expected.items():
        actual = predicates[name]["status"]
        if actual != expected_status:
            raise ExperimentError(
                f"Fire predicate {name} status {actual!r} disagrees with raw telemetry "
                f"({expected_status!r})"
            )


def validate_fire_event_contract(
    events: list[dict[str, Any]],
    samples: list[dict[str, Any]],
    first_combustion: dict[str, Any] | None,
    first_smoke: dict[str, Any] | None,
    first_heat: dict[str, Any] | None,
    first_phase: dict[str, Any] | None,
    first_fuel: dict[str, Any] | None,
    reaction_peak_updates: list[dict[str, Any]],
    thermal_peak_updates: list[dict[str, Any]],
    streak_starts: list[dict[str, Any]],
    streak_breaks: list[dict[str, Any]],
    zero_streak: list[dict[str, Any]] | None,
    terminal: dict[str, Any],
    post_reaction: list[dict[str, Any]],
) -> None:
    allowed = FIRE_ALWAYS_EVENTS | FIRE_OPTIONAL_EVENTS
    by_name: dict[str, list[dict[str, Any]]] = {}
    by_sequence = {sample["sample_sequence"]: sample for sample in samples}
    for event in events:
        if event["event"] not in allowed:
            raise ExperimentError(f"unsupported Fire event {event['event']!r}")
        by_name.setdefault(event["event"], []).append(event)
        sequence = event["sample_sequence"]
        if sequence is not None:
            sample = by_sequence.get(sequence)
            if sample is None or sample["sim_tick"] != event["sim_tick"]:
                raise ExperimentError(
                    f"Fire event {event['event']} does not bind to its telemetry sample"
                )
    for name in FIRE_ALWAYS_EVENTS:
        if len(by_name.get(name, [])) != 1:
            raise ExperimentError(f"Fire event {name} must occur exactly once")
    mandatory_order = (
        "lifecycle_started",
        "pristine_reset_completed",
        "tick0_captured",
        "tick1_captured",
        "terminal_selected",
        "reset_started",
        "reset_comparison_completed",
        "worker_completed",
    )
    positions = [by_name[name][0]["event_sequence"] for name in mandatory_order]
    if positions != sorted(positions):
        raise ExperimentError("Fire lifecycle events are out of order")
    for name in ("lifecycle_started", "pristine_reset_completed"):
        event = by_name[name][0]
        if event["sim_tick"] != 0 or event["sample_sequence"] is not None:
            raise ExperimentError(f"Fire event {name} must be a pre-sample tick0 event")
    identities = {
        "tick0_captured": samples[0],
        "tick1_captured": samples[1],
        "terminal_selected": terminal,
        "reset_started": samples[-2],
        "reset_comparison_completed": samples[-1],
        "worker_completed": samples[-1],
    }
    for name, sample in identities.items():
        event = by_name[name][0]
        if (event["sim_tick"], event["sample_sequence"]) != sample_identity(sample):
            raise ExperimentError(f"Fire event {name} identity disagrees with telemetry")
    one_shot = {
        "combustion_observed": first_combustion,
        "smoke_generated": first_smoke,
        "heat_propagated": first_heat,
        "phase_transition_observed": first_phase,
        "fuel_substantially_consumed": first_fuel,
    }
    for name, sample in one_shot.items():
        found = by_name.get(name, [])
        if sample is None:
            if found:
                raise ExperimentError(f"Fire event {name} exists without its signal")
        elif len(found) != 1:
            raise ExperimentError(f"Fire event {name} must occur exactly once")
        elif (found[0]["sim_tick"], found[0]["sample_sequence"]) != sample_identity(sample):
            raise ExperimentError(f"Fire event {name} identity disagrees with telemetry")
    repeated = {
        "new_peak_reaction": reaction_peak_updates,
        "new_peak_thermal": thermal_peak_updates,
        "reaction_zero_streak_started": streak_starts,
        "reaction_zero_streak_broken": streak_breaks,
    }
    for name, expected_samples in repeated.items():
        found = by_name.get(name, [])
        if len(found) != len(expected_samples):
            raise ExperimentError(f"Fire event {name} cardinality disagrees with telemetry")
        for event, sample in zip(found, expected_samples, strict=True):
            if (event["sim_tick"], event["sample_sequence"]) != sample_identity(sample):
                raise ExperimentError(f"Fire event {name} identity disagrees with telemetry")
    confirmed_events = by_name.get("reaction_zero_confirmed", [])
    if zero_streak is None:
        if confirmed_events:
            raise ExperimentError("Fire reaction-zero event exists without confirmation")
    elif len(confirmed_events) != 1 or (
        confirmed_events[0]["sim_tick"], confirmed_events[0]["sample_sequence"]
    ) != sample_identity(zero_streak[-1]):
        raise ExperimentError("Fire reaction-zero confirmation event is invalid")
    post_events = by_name.get("post_reaction_confirmation_completed", [])
    if bool(post_reaction) != (len(post_events) == 1):
        raise ExperimentError("Fire post-reaction completion event disagrees with lifecycle")
    if post_reaction and (
        post_events[0]["sim_tick"], post_events[0]["sample_sequence"]
    ) != sample_identity(post_reaction[-1]):
        raise ExperimentError("Fire post-reaction completion event identity is invalid")


def validate_fire_frame_contract(
    frames: list[dict[str, Any]],
    samples: list[dict[str, Any]],
    analysis: dict[str, Any],
    expected_by_kind: dict[str, dict[str, Any] | None],
) -> None:
    by_kind: dict[str, list[dict[str, Any]]] = {}
    by_sequence = {sample["sample_sequence"]: sample for sample in samples}
    for frame in frames:
        by_kind.setdefault(frame["kind"], []).append(frame)
        sample = by_sequence.get(frame["sample_sequence"])
        if sample is None or (
            sample["sim_tick"] != frame["sim_tick"]
            or sample["state_hash"] != frame["state_hash"]
        ):
            raise ExperimentError("Fire frame identity disagrees with telemetry")
        expected_reason = (
            "reaction-zero-confirmed"
            if frame["kind"] == "terminal"
            and analysis["lifecycle"]["terminal_reason"] == "reaction-zero"
            else "max-tick-reached"
            if frame["kind"] == "terminal"
            else FIRE_FRAME_REASONS[frame["kind"]]
        )
        if frame["reason"] != expected_reason:
            raise ExperimentError(f"Fire frame {frame['kind']} reason mismatch")
    for kind, grouped in by_kind.items():
        if kind != "diagnostic-observation" and len(grouped) != 1:
            raise ExperimentError(f"Fire frame kind {kind} occurs more than once")
    for required in ("tick0", "tick1", "terminal", "reset"):
        if len(by_kind.get(required, [])) != 1:
            raise ExperimentError(f"Fire frame kind {required} must occur exactly once")
    for kind, expected_sample in expected_by_kind.items():
        found = by_kind.get(kind, [])
        if expected_sample is None:
            if found:
                raise ExperimentError(f"Fire frame kind {kind} exists without its milestone")
        elif len(found) != 1:
            raise ExperimentError(f"Fire frame kind {kind} must occur exactly once")
        elif found[0]["sample_sequence"] != expected_sample["sample_sequence"]:
            raise ExperimentError(f"Fire frame kind {kind} binds the wrong sample")
    fallback = by_kind.get("diagnostic-observation", [])
    missing_optional = any(
        sample is None
        for kind, sample in expected_by_kind.items()
        if kind not in {"tick0", "tick1", "terminal", "reset"}
    )
    if fallback and (not missing_optional or len(frames) != 8):
        raise ExperimentError(
            "Fire diagnostic-observation frames may only fill incomplete evidence to eight"
        )
    diagnostics = fire_diagnostic_samples(samples)
    for frame in fallback:
        if by_sequence[frame["sample_sequence"]] not in diagnostics:
            raise ExperimentError("Fire diagnostic-observation frame is not diagnostic telemetry")


def validate_fire_telemetry(
    run_dir: Path, manifest: dict[str, Any]
) -> tuple[dict[str, Any], dict[str, Any], list[dict[str, Any]], list[dict[str, Any]]]:
    for path in (
        run_dir / "stdout.log",
        run_dir / "stderr.log",
        run_dir / "logs" / "build.stdout.log",
        run_dir / "logs" / "build.stderr.log",
    ):
        if not path.is_file():
            raise ExperimentError(f"required raw command log is missing: {path}")
    analysis = read_json(run_dir / "work" / "analysis.json", "analysis.json")
    frames_doc = read_json(run_dir / "work" / "frames.json", "frames.json")
    samples = read_jsonl(run_dir / "telemetry" / "samples.jsonl", "samples.jsonl")
    events = read_jsonl(run_dir / "telemetry" / "events.jsonl", "events.jsonl")
    validate_fire_analysis(analysis, manifest)
    frames = validate_frames(frames_doc, manifest, run_dir)
    validate_fire_samples(samples, manifest)
    validate_events(events, manifest)
    if analysis["raw_frame_count"] != len(frames):
        raise ExperimentError("Fire analysis raw_frame_count does not match frames.json")
    if analysis["lifecycle"]["sample_count"] != len(samples):
        raise ExperimentError("Fire lifecycle sample_count does not match samples.jsonl")
    if analysis["verdict"] != verdict_from_predicates(analysis["predicates"], FIRE_CONTRACT):
        raise ExperimentError("Fire verdict disagrees with its twelve predicate statuses")
    for index, sample in enumerate(samples):
        if sample["sleep"] != analysis["sleep"]:
            raise ExperimentError(f"Fire sample {index} sleep settings disagree with analysis")
    if len(samples) < 4:
        raise ExperimentError("Fire telemetry must contain tick0, tick1, diagnostics, and reset")
    tick0, tick1, reset = samples[0], samples[1], samples[-1]
    if (tick0["sim_tick"], tick0["phase"], tick0["reason"]) != (0, "initial", "tick0"):
        raise ExperimentError("Fire telemetry must begin with initial tick0")
    if (tick1["sim_tick"], tick1["phase"], tick1["reason"]) != (1, "reacting", "tick1"):
        raise ExperimentError("Fire telemetry sample 1 must be reacting tick1")
    reset_samples = [sample for sample in samples if sample["phase"] == "reset"]
    if len(reset_samples) != 1 or reset_samples[0] is not reset:
        raise ExperimentError("Fire telemetry must end with exactly one reset sample")
    if reset["sim_tick"] != 0 or reset["reason"] != "programmatic-r-equivalent":
        raise ExperimentError("Fire reset sample must be the programmatic tick0 equivalent")
    pre_reset = samples[:-1]
    if any(
        later["sim_tick"] <= earlier["sim_tick"]
        for earlier, later in zip(pre_reset, pre_reset[1:])
    ):
        raise ExperimentError("Fire pre-reset sim ticks must be strictly increasing")
    post_reaction = [
        sample for sample in pre_reset if sample["phase"] == "post-reaction-confirmation"
    ]
    if post_reaction and any(
        sample["phase"] != "post-reaction-confirmation"
        for sample in pre_reset[pre_reset.index(post_reaction[0]) :]
    ):
        raise ExperimentError("Fire post-reaction phase must be contiguous and final")
    reacting = [sample for sample in pre_reset if sample["phase"] == "reacting"]
    if not reacting or reacting[0] is not tick1:
        raise ExperimentError("Fire telemetry lacks its reacting lifecycle")

    baseline_keys = (
        "matter_count",
        "wood_count",
        "oil_count",
        "smoke_count",
        "ice_count",
        "water_count",
        "steam_count",
        "wood_fuel_progress_sum",
        "oil_fuel_progress_sum",
    )
    expected_baseline = {key: tick0[key] for key in baseline_keys}
    initial_fuel = tick0["wood_count"] + tick0["oil_count"]
    threshold = (initial_fuel + 3) // 4
    expected_baseline.update(
        {
            "fuel_count": initial_fuel,
            "substantial_fuel_consumption_threshold": threshold,
            "substantial_fuel_remaining_threshold": initial_fuel - threshold,
        }
    )
    if analysis["baseline"] != expected_baseline:
        raise ExperimentError("Fire analysis baseline disagrees with tick0 telemetry")
    for sample in pre_reset:
        inventory_changed = (
            sample["ice_count"], sample["water_count"], sample["steam_count"]
        ) != (tick0["ice_count"], tick0["water_count"], tick0["steam_count"])
        if sample["phase_inventory_changed"] != inventory_changed:
            raise ExperimentError("Fire phase_inventory_changed disagrees with raw inventory")

    observed = pre_reset[1:]
    wood_seen = False
    oil_seen = False
    first_combustion = None
    for sample in reacting:
        wood_seen |= (
            sample["flame_event_wood_cells"] > 0
            or sample["wood_fuel_progress_sum"] > tick0["wood_fuel_progress_sum"]
        )
        oil_seen |= (
            sample["flame_event_oil_cells"] > 0
            or sample["oil_fuel_progress_sum"] > tick0["oil_fuel_progress_sum"]
        )
        if wood_seen and oil_seen:
            first_combustion = sample
            break
    first_smoke = first_matching(observed, lambda sample: sample["smoke_count"] > tick0["smoke_count"])
    first_heat = first_matching(observed, lambda sample: sample["heat_propagated_cells"] > 0)
    first_phase = first_matching(observed, lambda sample: sample["phase_inventory_changed"])
    first_fuel = first_matching(
        observed,
        lambda sample: sample["wood_count"] + sample["oil_count"]
        <= expected_baseline["substantial_fuel_remaining_threshold"],
    )

    reaction_peak = None
    thermal_peak = None
    reaction_peak_cells = 0
    thermal_peak_cells = 0
    reaction_peak_updates: list[dict[str, Any]] = []
    thermal_peak_updates: list[dict[str, Any]] = []
    for sample in observed:
        if sample["census"]["reaction_active_cells"] > reaction_peak_cells:
            reaction_peak = sample
            reaction_peak_cells = sample["census"]["reaction_active_cells"]
            reaction_peak_updates.append(sample)
        if sample["census"]["thermal_active_cells"] > thermal_peak_cells:
            thermal_peak = sample
            thermal_peak_cells = sample["census"]["thermal_active_cells"]
            thermal_peak_updates.append(sample)
    max_heat = max(sample["heat_propagated_cells"] for sample in observed)
    peak_smoke = tick0
    peak_smoke_count = tick0["smoke_count"]
    for sample in observed:
        if sample["smoke_count"] > peak_smoke_count:
            peak_smoke = sample
            peak_smoke_count = sample["smoke_count"]

    diagnostics = fire_diagnostic_samples(reacting)
    lifecycle = analysis["lifecycle"]
    terminal_reason = lifecycle["terminal_reason"]
    zero_streak, streak_starts, streak_breaks = confirmed_fire_reaction_zero_streak(
        reacting, first_combustion
    )
    if terminal_reason == "reaction-zero":
        if zero_streak is None:
            raise ExperimentError("Fire reaction-zero terminal lacks a three-sample streak")
        terminal = zero_streak[-1]
        if (
            lifecycle["first_reaction_zero_sim_tick"],
            lifecycle["first_reaction_zero_sample_sequence"],
            lifecycle["confirmed_reaction_zero_sim_tick"],
            lifecycle["confirmed_reaction_zero_sample_sequence"],
        ) != (
            zero_streak[0]["sim_tick"],
            zero_streak[0]["sample_sequence"],
            terminal["sim_tick"],
            terminal["sample_sequence"],
        ):
            raise ExperimentError("Fire reaction-zero lifecycle identity mismatch")
    else:
        terminal = diagnostics[-1] if diagnostics else None
        if terminal is None or terminal["sim_tick"] != MAX_TICKS or terminal["reason"] != "max-tick":
            raise ExperimentError("Fire max-ticks terminal is not bound to max-tick telemetry")
        if zero_streak is not None or any(
            lifecycle[key] is not None
            for key in (
                "first_reaction_zero_sim_tick",
                "first_reaction_zero_sample_sequence",
                "confirmed_reaction_zero_sim_tick",
                "confirmed_reaction_zero_sample_sequence",
            )
        ):
            raise ExperimentError("Fire max-ticks lifecycle records reaction-zero confirmation")

    expected_diagnostic_ticks = [2]
    expected_diagnostic_ticks.extend(
        range(DIAGNOSTIC_INTERVAL, terminal["sim_tick"] + 1, DIAGNOSTIC_INTERVAL)
    )
    if terminal["sim_tick"] == MAX_TICKS and MAX_TICKS not in expected_diagnostic_ticks:
        expected_diagnostic_ticks.append(MAX_TICKS)
    expected_diagnostic_ticks = sorted(set(expected_diagnostic_ticks))
    if [sample["sim_tick"] for sample in diagnostics] != expected_diagnostic_ticks:
        raise ExperimentError("Fire diagnostic cadence is incomplete or contains extra samples")
    for sample in diagnostics:
        expected_reason = (
            "early-diagnostic"
            if sample["sim_tick"] == 2
            else "max-tick"
            if sample["sim_tick"] == MAX_TICKS
            else "diagnostic-cadence"
        )
        if sample["reason"] != expected_reason:
            raise ExperimentError("Fire diagnostic reason disagrees with sim tick")

    if terminal_reason == "reaction-zero":
        if len(post_reaction) != POST_REACTION_TICKS:
            raise ExperimentError("Fire terminal must have exactly 180 post-reaction samples")
        expected_ticks = list(
            range(terminal["sim_tick"] + 1, terminal["sim_tick"] + POST_REACTION_TICKS + 1)
        )
        if [sample["sim_tick"] for sample in post_reaction] != expected_ticks:
            raise ExperimentError("Fire post-reaction ticks must be contiguous")
        if lifecycle["post_reaction_end_tick"] != expected_ticks[-1]:
            raise ExperimentError("Fire post_reaction_end_tick disagrees with telemetry")
    elif post_reaction or lifecycle["post_reaction_end_tick"] is not None:
        raise ExperimentError("Fire max-ticks terminal must not have a post-reaction window")

    restart_samples = [
        sample for sample in post_reaction if sample["census"]["reaction_active_cells"] > 0
    ]
    if lifecycle["post_reaction_restart_samples"] != len(restart_samples):
        raise ExperimentError("Fire lifecycle restart count disagrees with telemetry")
    final_pre_reset = pre_reset[-1]
    metrics = analysis["metrics"]
    post_start_thermal = (
        terminal["census"]["thermal_active_cells"] if post_reaction else 0
    )
    post_final_thermal = (
        post_reaction[-1]["census"]["thermal_active_cells"] if post_reaction else 0
    )
    post_min_thermal = (
        min(
            [terminal["census"]["thermal_active_cells"]]
            + [sample["census"]["thermal_active_cells"] for sample in post_reaction]
        )
        if post_reaction
        else 0
    )
    invalid_occurrences = sum(sample["invalid_material_count"] for sample in pre_reset)
    nonfinite_occurrences = sum(
        sample["nonfinite_temperature_count"] + sample["nonfinite_pressure_count"]
        for sample in pre_reset
    )
    expected_metrics = {
        "first_combustion_tick": sample_identity(first_combustion)[0],
        "first_combustion_sample_sequence": sample_identity(first_combustion)[1],
        "first_smoke_tick": sample_identity(first_smoke)[0],
        "first_smoke_sample_sequence": sample_identity(first_smoke)[1],
        "first_phase_transition_tick": sample_identity(first_phase)[0],
        "first_phase_transition_sample_sequence": sample_identity(first_phase)[1],
        "fuel_substantially_consumed_tick": sample_identity(first_fuel)[0],
        "fuel_substantially_consumed_sample_sequence": sample_identity(first_fuel)[1],
        "peak_reaction_cells": reaction_peak_cells,
        "peak_reaction_tick": sample_identity(reaction_peak)[0],
        "peak_reaction_sample_sequence": sample_identity(reaction_peak)[1],
        "peak_thermal_cells": thermal_peak_cells,
        "peak_thermal_tick": sample_identity(thermal_peak)[0],
        "peak_thermal_sample_sequence": sample_identity(thermal_peak)[1],
        "peak_smoke_count": peak_smoke_count,
        "peak_smoke_tick": sample_identity(peak_smoke)[0],
        "peak_smoke_sample_sequence": sample_identity(peak_smoke)[1],
        "max_heat_propagated_cells": max_heat,
        "reaction_zero_tick": sample_identity(None if zero_streak is None else zero_streak[0])[0],
        "confirmed_reaction_zero_tick": sample_identity(
            None if zero_streak is None else zero_streak[-1]
        )[0],
        "post_reaction_thermal_cells": post_start_thermal,
        "post_reaction_final_thermal_cells": post_final_thermal,
        "post_reaction_min_thermal_cells": post_min_thermal,
        "post_reaction_thermal_decrease": post_min_thermal < post_start_thermal,
        "post_reaction_reaction_restart_ticks": len(restart_samples),
        "post_reaction_restart_samples": len(restart_samples),
        "final_matter_count": final_pre_reset["matter_count"],
        "final_wood_count": final_pre_reset["wood_count"],
        "final_oil_count": final_pre_reset["oil_count"],
        "final_smoke_count": final_pre_reset["smoke_count"],
        "final_ice_count": final_pre_reset["ice_count"],
        "final_water_count": final_pre_reset["water_count"],
        "final_steam_count": final_pre_reset["steam_count"],
        "wood_count_delta": final_pre_reset["wood_count"] - tick0["wood_count"],
        "oil_count_delta": final_pre_reset["oil_count"] - tick0["oil_count"],
        "fuel_count_delta": (
            final_pre_reset["wood_count"]
            + final_pre_reset["oil_count"]
            - initial_fuel
        ),
        "fuel_consumed": max(
            0,
            initial_fuel - final_pre_reset["wood_count"] - final_pre_reset["oil_count"],
        ),
        "invalid_material_occurrences": invalid_occurrences,
        "nonfinite_field_occurrences": nonfinite_occurrences,
    }
    for key, expected in expected_metrics.items():
        if metrics[key] != expected:
            raise ExperimentError(f"Fire analysis metric {key} disagrees with telemetry")

    reset_keys = (
        "world",
        "sleep",
        "census",
        "material_counts_by_id",
        "matter_count",
        "wood_count",
        "oil_count",
        "smoke_count",
        "ice_count",
        "water_count",
        "steam_count",
        "combusting_wood_cells",
        "combusting_oil_cells",
        "flame_event_wood_cells",
        "flame_event_oil_cells",
        "wood_fuel_progress_sum",
        "oil_fuel_progress_sum",
        "heat_propagated_cells",
        "phase_inventory_changed",
        "invalid_material_count",
        "nonfinite_temperature_count",
        "nonfinite_pressure_count",
        "state_hash",
        "physical_state_hash",
    )
    reset_equal = all(reset[key] == tick0[key] for key in reset_keys)
    if metrics["reset_exact_equivalence"] and not reset_equal:
        raise ExperimentError("Fire exact reset claim disagrees with observable telemetry")
    no_invalid = invalid_occurrences == 0
    no_nonfinite = nonfinite_occurrences == 0
    completed_post = terminal_reason == "reaction-zero" and len(post_reaction) == POST_REACTION_TICKS
    expected_predicates = {
        "combustion_observed": "pass" if first_combustion is not None else "fail",
        "smoke_generated": "pass" if first_smoke is not None else "fail",
        "heat_propagated": "pass" if first_heat is not None else "fail",
        "phase_work_observed": "pass" if first_phase is not None else "fail",
        "fuel_consumed": "pass" if metrics["fuel_consumed"] > 0 else "fail",
        "reaction_terminated_before_max": (
            "pass"
            if terminal_reason == "reaction-zero" and terminal["sim_tick"] < MAX_TICKS
            else "fail"
        ),
        "post_reaction_no_restart": (
            "unknown"
            if not completed_post
            else "pass"
            if not restart_samples
            else "fail"
        ),
        "thermal_tail_observed": "pass" if post_start_thermal > 0 else "unknown",
        "thermal_tail_decreased": (
            "pass" if post_min_thermal < post_start_thermal else "unknown"
        ),
        "no_invalid_materials": "pass" if no_invalid else "fail",
        "no_nonfinite_fields": "pass" if no_nonfinite else "fail",
        "exact_reset": "pass" if metrics["reset_exact_equivalence"] else "fail",
    }
    require_fire_predicate_statuses(analysis["predicates"], expected_predicates)

    validate_fire_event_contract(
        events,
        samples,
        first_combustion,
        first_smoke,
        first_heat,
        first_phase,
        first_fuel,
        reaction_peak_updates,
        thermal_peak_updates,
        streak_starts,
        streak_breaks,
        zero_streak,
        terminal,
        post_reaction,
    )
    expected_frames = {
        "tick0": tick0,
        "tick1": tick1,
        "first-combustion": first_combustion,
        "first-smoke": first_smoke,
        "peak-reaction": reaction_peak,
        "peak-thermal": thermal_peak,
        "first-phase-transition": first_phase,
        "fuel-substantially-consumed": first_fuel,
        "reaction-zero": None if zero_streak is None else zero_streak[0],
        "post-reaction-tail": post_reaction[-1] if post_reaction else None,
        "terminal": terminal,
        "reset": reset,
    }
    validate_fire_frame_contract(frames, samples, analysis, expected_frames)
    return analysis, frames_doc, samples, events


def pressure_float_equal(recorded: Any, expected: float) -> bool:
    return isinstance(recorded, (int, float)) and not isinstance(recorded, bool) and math.isclose(
        # Worker values are canonicalized to nine decimal places. Half of one
        # output unit admits only final-decimal rounding for derived values
        # such as the trend slope; it cannot hide one telemetry output unit.
        float(recorded), float(expected), rel_tol=0.0, abs_tol=5.0e-10
    )


def pressure_diagnostic_samples(
    samples: Iterable[dict[str, Any]],
) -> list[dict[str, Any]]:
    return [
        sample
        for sample in samples
        if sample["phase"] == "pressurizing"
        and sample["reason"] in {"early-diagnostic", "diagnostic-cadence", "max-tick"}
    ]


def pressure_opening_streak(
    diagnostics: list[dict[str, Any]],
) -> tuple[
    list[dict[str, Any]] | None,
    list[dict[str, Any]],
    list[dict[str, Any]],
]:
    streak: list[dict[str, Any]] = []
    starts: list[dict[str, Any]] = []
    breaks: list[dict[str, Any]] = []
    for sample in diagnostics:
        if sample["relief_seam_through_open_lanes"] == 0:
            if streak:
                breaks.append(sample)
                streak = []
            continue
        if not streak:
            starts.append(sample)
        streak.append(sample)
        if len(streak) == CONSECUTIVE_PERSISTENT_OPENING:
            return list(streak), starts, breaks
    return None, starts, breaks


def pressure_terminal_trend(window_samples: list[dict[str, Any]]) -> dict[str, Any]:
    count = len(window_samples)
    means = [float(sample["chamber_mean_pressure"]) for sample in window_samples]
    maxima = [float(sample["chamber_max_pressure"]) for sample in window_samples]
    positive_steps = sum(
        right > left for left, right in zip(means, means[1:])
    )
    positive_max_steps = sum(
        right > left for left, right in zip(maxima, maxima[1:])
    )
    slope: float | None
    if count < 2:
        slope = None
    else:
        mean_x = (count - 1.0) / 2.0
        mean_y = sum(means) / count
        numerator = sum(
            (index - mean_x) * (value - mean_y)
            for index, value in enumerate(means)
        )
        denominator = sum((index - mean_x) ** 2 for index in range(count))
        slope = 0.0 if denominator == 0.0 else numerator / denominator
    mean_unbounded = (
        count >= 2
        and means[-1] > means[0] * 1.10 + 1.0
        and positive_steps * 4 >= (count - 1) * 3
    )
    max_unbounded = (
        count >= 2
        and maxima[-1] > maxima[0] * 1.10 + 1.0
        and positive_max_steps * 4 >= (count - 1) * 3
    )
    first = window_samples[0] if window_samples else None
    last = window_samples[-1] if window_samples else None
    return {
        "sample_count": count,
        "start_sim_tick": None if first is None else first["sim_tick"],
        "end_sim_tick": None if last is None else last["sim_tick"],
        "start_mean_pressure": None if first is None else first["chamber_mean_pressure"],
        "end_mean_pressure": None if last is None else last["chamber_mean_pressure"],
        "start_max_pressure": None if first is None else first["chamber_max_pressure"],
        "end_max_pressure": None if last is None else last["chamber_max_pressure"],
        "minimum_mean_pressure": None if not means else min(means),
        "maximum_mean_pressure": None if not means else max(means),
        "slope_per_sample": slope,
        "positive_step_count": positive_steps,
        "positive_max_step_count": positive_max_steps,
        "mean_unbounded_growth": mean_unbounded,
        "max_unbounded_growth": max_unbounded,
        "unbounded_growth": mean_unbounded or max_unbounded,
    }


def pressure_expected_verdict(
    predicate_statuses: dict[str, str],
    review_flags: dict[str, Any],
    causal_classification: str,
) -> str:
    if causal_classification == "fixture_causality_confounded":
        return PRESSURE_FIXTURE_CAUSALITY_CONFOUNDED_VERDICT
    if any(status == "fail" for status in predicate_statuses.values()):
        return "FAIL"
    if any(status == "unknown" for status in predicate_statuses.values()) or review_flags[
        "reasons"
    ]:
        return "NEEDS_HUMAN_REVIEW"
    return "PASS"


def pressure_causal_classification(
    *,
    opening_start: dict[str, Any] | None,
    confirmed: dict[str, Any] | None,
    first_combustion: dict[str, Any] | None,
    first_fuel_progress: dict[str, Any] | None,
    combusting_peak: int,
    flame_event_peak: int,
    fuel_progress_sum_peak: int,
    fuel_progress_max: int,
) -> str:
    opening_start_sequence = (
        None if opening_start is None else opening_start["sample_sequence"]
    )
    if opening_start_sequence is None:
        return "insufficient_causal_evidence"
    confounded = any(
        value > 0
        for value in (
            combusting_peak,
            flame_event_peak,
            fuel_progress_sum_peak,
            fuel_progress_max,
        )
    ) or any(
        sample is not None
        and opening_start_sequence is not None
        and sample["sample_sequence"] <= opening_start_sequence
        for sample in (first_combustion, first_fuel_progress)
    )
    if confounded:
        return "fixture_causality_confounded"
    if confirmed is None:
        return "insufficient_causal_evidence"
    return "pressure_opening_precedes_combustion"


def pressure_expected_frame_badges(
    tick0: dict[str, Any],
    tick1: dict[str, Any],
    first_pressure: dict[str, Any] | None,
    first_damage: dict[str, Any] | None,
    first_rupture: dict[str, Any] | None,
    opening_streak: list[dict[str, Any]] | None,
    first_reseal: dict[str, Any] | None,
    first_exterior: dict[str, Any] | None,
    peak_max: dict[str, Any],
    peak_activity: dict[str, Any],
    first_relief: dict[str, Any] | None,
    terminal: dict[str, Any],
    reset: dict[str, Any],
    diagnostic_fallbacks: list[dict[str, Any]],
    terminal_reason: str,
) -> list[dict[str, Any]]:
    milestone_specs: list[tuple[str, str, dict[str, Any] | None]] = [
        ("tick0", "pristine-reset", tick0),
        ("tick1", "after-one-production-tick", tick1),
        (
            "first-pressure-activity",
            "first-sampled-pressure-activity",
            first_pressure,
        ),
        (
            "first-wood-damage",
            "first-authored-relief-seam-wood-loss",
            first_damage,
        ),
        (
            "first-rupture",
            "first-eight-cell-through-open-relief-lane",
            first_rupture,
        ),
        (
            "persistent-opening",
            "three-consecutive-diagnostics-with-opening",
            None if opening_streak is None else opening_streak[-1],
        ),
        (
            "opening-reseal",
            "first-zero-through-lane-sample-after-persistent-confirmation",
            first_reseal,
        ),
        (
            "first-exterior-steam",
            "first-steam-outside-authored-chamber-after-opening",
            first_exterior,
        ),
        ("peak-pressure", "highest-observed-chamber-max-pressure", peak_max),
        (
            "peak-pressure-activity",
            "highest-observed-pressure-active-cells",
            peak_activity,
        ),
        (
            "post-opening",
            "first-post-vent-chamber-mean-and-max-pressure-relief",
            None if first_reseal is not None else first_relief,
        ),
        ("terminal", terminal_reason, terminal),
        ("reset", "programmatic-r-equivalent", reset),
    ]

    folded: dict[tuple[bool, int, str], dict[str, Any]] = {}

    def add_badge(kind: str, reason: str, sample: dict[str, Any]) -> None:
        is_reset = kind == "reset"
        key = (is_reset, sample["sim_tick"], sample["state_hash"])
        entry = folded.setdefault(
            key,
            {
                "sim_tick": sample["sim_tick"],
                "sample_sequence": sample["sample_sequence"],
                "state_hash": sample["state_hash"],
                "badges": [],
                "is_reset": is_reset,
            },
        )
        entry["sample_sequence"] = min(
            entry["sample_sequence"], sample["sample_sequence"]
        )
        if kind not in {badge["kind"] for badge in entry["badges"]}:
            entry["badges"].append({"kind": kind, "reason": reason})
            entry["badges"].sort(key=lambda badge: PRESSURE_FRAME_BADGE_RANK[badge["kind"]])

    for kind, reason, sample in milestone_specs:
        if sample is not None:
            add_badge(kind, reason, sample)
    for sample in diagnostic_fallbacks:
        if len(folded) >= 8:
            break
        add_badge("diagnostic-observation", "minimum-evidence-observation", sample)
    expected = sorted(
        folded.values(),
        key=lambda entry: (
            entry["is_reset"],
            entry["sim_tick"],
            entry["sample_sequence"],
            min(PRESSURE_FRAME_BADGE_RANK[badge["kind"]] for badge in entry["badges"]),
        ),
    )
    return expected


def validate_pressure_frame_contract(
    frames: list[dict[str, Any]], expected: list[dict[str, Any]]
) -> None:
    if len(frames) != len(expected):
        raise ExperimentError(
            f"Pressure physical frame count mismatch: recorded={len(frames)}, expected={len(expected)}"
        )
    for index, (recorded, wanted) in enumerate(zip(frames, expected, strict=True)):
        for key in ("sim_tick", "sample_sequence", "state_hash", "badges"):
            if recorded[key] != wanted[key]:
                raise ExperimentError(
                    f"Pressure frame {index} {key} disagrees with deterministic milestone folding"
                )
    non_reset = [frame for frame in frames if not any(
        badge["kind"] == "reset" for badge in frame["badges"]
    )]
    if any(
        later["sim_tick"] <= earlier["sim_tick"]
        for earlier, later in zip(non_reset, non_reset[1:])
    ):
        raise ExperimentError("Pressure non-reset frames must be strictly chronological")
    if not frames[-1]["badges"] or not any(
        badge["kind"] == "reset" for badge in frames[-1]["badges"]
    ):
        raise ExperimentError("Pressure reset frame must be last")
    if any(
        any(badge["kind"] == "reset" for badge in frame["badges"])
        for frame in frames[:-1]
    ):
        raise ExperimentError("Pressure reset badge may occur only on the last frame")


def validate_pressure_event_contract(
    events: list[dict[str, Any]], expected: list[tuple[str, int, int | None]]
) -> None:
    observed = [
        (event["event"], event["sim_tick"], event["sample_sequence"])
        for event in events
    ]
    if observed != expected:
        raise ExperimentError("Pressure event sequence disagrees with raw telemetry milestones")


def validate_pressure_telemetry(
    run_dir: Path, manifest: dict[str, Any]
) -> tuple[dict[str, Any], dict[str, Any], list[dict[str, Any]], list[dict[str, Any]]]:
    analysis = read_json(run_dir / "work" / "analysis.json", "Pressure analysis")
    frames_doc = read_json(run_dir / "work" / "frames.json", "Pressure frames")
    samples = read_jsonl(run_dir / "telemetry" / "samples.jsonl", "Pressure samples")
    events = read_jsonl(run_dir / "telemetry" / "events.jsonl", "Pressure events")
    validate_analysis(analysis, manifest)
    frames = validate_frames(frames_doc, manifest, run_dir)
    validate_samples(samples, manifest)
    validate_events(events, manifest)
    if analysis["raw_frame_count"] != len(frames):
        raise ExperimentError("Pressure analysis raw_frame_count disagrees with frames.json")
    if analysis["lifecycle"]["sample_count"] != len(samples):
        raise ExperimentError("Pressure analysis sample_count disagrees with samples.jsonl")
    for index, sample in enumerate(samples):
        if sample["sleep"] != analysis["sleep"]:
            raise ExperimentError(f"Pressure sample {index} sleep settings disagree with analysis")
    if len(samples) < 4:
        raise ExperimentError("Pressure telemetry must contain tick0, tick1, terminal, and reset")
    tick0, tick1, reset = samples[0], samples[1], samples[-1]
    if (tick0["sim_tick"], tick0["phase"], tick0["reason"]) != (0, "initial", "tick0"):
        raise ExperimentError("Pressure telemetry must begin with initial tick0")
    if (tick1["sim_tick"], tick1["phase"], tick1["reason"]) != (
        1,
        "pressurizing",
        "tick1",
    ):
        raise ExperimentError("Pressure telemetry sample 1 must be pressurizing tick1")
    if (reset["sim_tick"], reset["phase"], reset["reason"]) != (
        0,
        "reset",
        "programmatic-r-equivalent",
    ):
        raise ExperimentError("Pressure telemetry must end with the programmatic reset")
    if any(sample["phase"] == "reset" for sample in samples[:-1]):
        raise ExperimentError("Pressure reset sample may occur only once and last")
    pre_reset = samples[:-1]
    if any(
        later["sim_tick"] <= earlier["sim_tick"]
        for earlier, later in zip(pre_reset, pre_reset[1:])
    ):
        raise ExperimentError("Pressure pre-reset sim ticks must be strictly increasing")

    if (
        tick0["relief_seam_wood_cells"],
        tick0["top_relief_seam_wood_cells"],
        tick0["bottom_relief_seam_wood_cells"],
        tick0["relief_seam_open_cells"],
        tick0["relief_seam_through_open_lanes"],
        tick0["chamber_pressure_cell_count"],
    ) != (576, 384, 192, 0, 0, 29_920):
        raise ExperimentError("Pressure tick0 authored chamber/seam baseline mismatch")
    for key in (
        "relief_seam_combusting_cells",
        "relief_seam_flame_event_cells",
        "relief_seam_fuel_progress_sum",
        "relief_seam_fuel_progress_max",
    ):
        if tick0[key] != 0:
            raise ExperimentError(
                f"Pressure tick0 authored relief seam {key} must be zero"
            )
    expected_baseline = {
        "initial_matter_count": tick0["matter_count"],
        "initial_water_count": tick0["water_count"],
        "initial_steam_count": tick0["steam_count"],
        "initial_relief_seam_wood_cells": tick0["relief_seam_wood_cells"],
        "initial_top_relief_seam_wood_cells": tick0[
            "top_relief_seam_wood_cells"
        ],
        "initial_bottom_relief_seam_wood_cells": tick0[
            "bottom_relief_seam_wood_cells"
        ],
        "initial_chamber_pressure_cell_count": tick0["chamber_pressure_cell_count"],
        "initial_chamber_mean_pressure": tick0["chamber_mean_pressure"],
        "initial_chamber_max_pressure": tick0["chamber_max_pressure"],
    }
    if analysis["baseline"] != expected_baseline:
        raise ExperimentError("Pressure analysis baseline disagrees with tick0 telemetry")

    diagnostics = pressure_diagnostic_samples(pre_reset)
    opening_observations = [tick1, *diagnostics]
    opening_streak, streak_starts, streak_breaks = pressure_opening_streak(
        opening_observations
    )
    confirmed = None if opening_streak is None else opening_streak[-1]
    expected_diagnostic_ticks = [2]
    diagnostic_end = MAX_TICKS if confirmed is None else confirmed["sim_tick"]
    expected_diagnostic_ticks.extend(
        range(DIAGNOSTIC_INTERVAL, diagnostic_end + 1, DIAGNOSTIC_INTERVAL)
    )
    if diagnostic_end == MAX_TICKS and MAX_TICKS not in expected_diagnostic_ticks:
        expected_diagnostic_ticks.append(MAX_TICKS)
    expected_diagnostic_ticks = sorted(set(expected_diagnostic_ticks))
    if [sample["sim_tick"] for sample in diagnostics] != expected_diagnostic_ticks:
        raise ExperimentError("Pressure diagnostic cadence is incomplete or contains extras")
    for sample in diagnostics:
        expected_reason = (
            "early-diagnostic"
            if sample["sim_tick"] == 2
            else "max-tick"
            if sample["sim_tick"] == MAX_TICKS
            else "diagnostic-cadence"
        )
        if sample["reason"] != expected_reason:
            raise ExperimentError("Pressure diagnostic reason disagrees with sim tick")

    post_opening = [
        sample
        for sample in pre_reset
        if sample["phase"] == "post-opening-observation"
    ]
    lifecycle = analysis["lifecycle"]
    terminal_reason = lifecycle["terminal_reason"]
    if confirmed is None:
        if post_opening:
            raise ExperimentError("Pressure post-opening samples exist without confirmation")
        terminal = diagnostics[-1]
        if terminal["sim_tick"] != MAX_TICKS or terminal_reason != "max-ticks":
            raise ExperimentError("Pressure no-opening lifecycle must terminate at max-ticks")
        expected_post_end = None
    else:
        expected_post_ticks = list(
            range(
                confirmed["sim_tick"] + 1,
                min(MAX_TICKS, confirmed["sim_tick"] + POST_OPENING_TICKS) + 1,
            )
        )
        if [sample["sim_tick"] for sample in post_opening] != expected_post_ticks:
            raise ExperimentError("Pressure post-opening ticks must be contiguous")
        terminal = post_opening[-1] if post_opening else confirmed
        full_window = len(post_opening) == POST_OPENING_TICKS
        expected_terminal_reason = (
            "post-opening-observation-complete" if full_window else "max-ticks"
        )
        if terminal_reason != expected_terminal_reason:
            raise ExperimentError("Pressure terminal_reason disagrees with post-opening window")
        if post_opening:
            for sample in post_opening[:-1]:
                if sample["reason"] != "post-opening-tick":
                    raise ExperimentError("Pressure post-opening interior reason mismatch")
            expected_last_reason = (
                "post-opening-observation-complete"
                if full_window
                else "max-tick"
            )
            if post_opening[-1]["reason"] != expected_last_reason:
                raise ExperimentError("Pressure post-opening terminal reason mismatch")
        expected_post_end = terminal["sim_tick"] if full_window else None

    opening_start = None if opening_streak is None else opening_streak[0]
    expected_lifecycle = {
        "persistent_opening_start_sim_tick": sample_identity(opening_start)[0],
        "persistent_opening_start_sample_sequence": sample_identity(opening_start)[1],
        "persistent_opening_confirmed_sim_tick": sample_identity(confirmed)[0],
        "persistent_opening_confirmed_sample_sequence": sample_identity(confirmed)[1],
        "post_opening_end_tick": expected_post_end,
    }
    for key, expected in expected_lifecycle.items():
        if lifecycle[key] != expected:
            raise ExperimentError(f"Pressure lifecycle {key} disagrees with telemetry")

    observed = pre_reset[1:]
    first_pressure = first_matching(
        observed, lambda sample: sample["census"]["pressure_active_cells"] > 0
    )
    first_damage = first_matching(
        observed,
        lambda sample: sample["relief_seam_wood_cells"]
        < tick0["relief_seam_wood_cells"],
    )
    first_rupture = first_matching(
        observed,
        lambda sample: sample["relief_seam_through_open_lanes"] > 0,
    )
    first_seam_combustion = first_matching(
        observed,
        lambda sample: sample["relief_seam_combusting_cells"] > 0,
    )
    first_seam_fuel_progress = first_matching(
        observed,
        lambda sample: sample["relief_seam_fuel_progress_sum"] > 0,
    )
    through_confirmation_samples = (
        pre_reset
        if confirmed is None
        else [
            sample
            for sample in pre_reset
            if sample["sample_sequence"] <= confirmed["sample_sequence"]
        ]
    )
    through_confirmation_combusting_peak = max(
        sample["relief_seam_combusting_cells"]
        for sample in through_confirmation_samples
    )
    through_confirmation_flame_peak = max(
        sample["relief_seam_flame_event_cells"]
        for sample in through_confirmation_samples
    )
    through_confirmation_fuel_sum_peak = max(
        sample["relief_seam_fuel_progress_sum"]
        for sample in through_confirmation_samples
    )
    through_confirmation_fuel_max = max(
        sample["relief_seam_fuel_progress_max"]
        for sample in through_confirmation_samples
    )
    expected_causal_classification = pressure_causal_classification(
        opening_start=opening_start,
        confirmed=confirmed,
        first_combustion=first_seam_combustion,
        first_fuel_progress=first_seam_fuel_progress,
        combusting_peak=through_confirmation_combusting_peak,
        flame_event_peak=through_confirmation_flame_peak,
        fuel_progress_sum_peak=through_confirmation_fuel_sum_peak,
        fuel_progress_max=through_confirmation_fuel_max,
    )
    first_reseal = None
    first_seam_steam = None
    if confirmed is not None:
        first_reseal = first_matching(
            observed,
            lambda sample: sample["sample_sequence"]
            > confirmed["sample_sequence"]
            and sample["relief_seam_through_open_lanes"] == 0,
        )
        first_seam_steam = first_matching(
            observed,
            lambda sample: sample["sample_sequence"]
            >= confirmed["sample_sequence"]
            and sample["steam_in_relief_seam_cells"] > 0,
        )
    first_exterior = None
    if first_seam_steam is not None:
        first_exterior = first_matching(
            observed,
            lambda sample: sample["sample_sequence"]
            >= first_seam_steam["sample_sequence"]
            and sample["outside_chamber_steam_cells"] > 0,
        )

    peak_mean = tick0
    peak_max = tick0
    peak_activity = tick0
    pre_opening_samples = [
        tick0,
        *[
            sample
            for sample in observed
            if confirmed is None
            or sample["sample_sequence"] < confirmed["sample_sequence"]
        ],
    ]
    pre_opening_peak_mean = max(
        float(sample["chamber_mean_pressure"]) for sample in pre_opening_samples
    )
    pre_opening_peak_max = max(
        float(sample["chamber_max_pressure"]) for sample in pre_opening_samples
    )
    for sample in observed:
        if sample["chamber_mean_pressure"] > peak_mean["chamber_mean_pressure"]:
            peak_mean = sample
        if sample["chamber_max_pressure"] > peak_max["chamber_max_pressure"]:
            peak_max = sample
        if (
            sample["census"]["pressure_active_cells"]
            > peak_activity["census"]["pressure_active_cells"]
        ):
            peak_activity = sample
    first_post = confirmed
    vent_reference_mean = (
        None if first_exterior is None else float(first_exterior["chamber_mean_pressure"])
    )
    vent_reference_max = (
        None if first_exterior is None else float(first_exterior["chamber_max_pressure"])
    )
    first_relief = None
    if (
        first_exterior is not None
        and vent_reference_mean is not None
        and vent_reference_max is not None
    ):
        first_relief = first_matching(
            observed,
            lambda sample: sample["sample_sequence"]
            > first_exterior["sample_sequence"]
            and sample["chamber_mean_pressure"] < vent_reference_mean
            and sample["chamber_max_pressure"] < vent_reference_max,
        )

    if confirmed is None:
        terminal_source = [tick1, *diagnostics]
    else:
        terminal_source = [confirmed, *post_opening]
    terminal_window_samples = terminal_source[-TERMINAL_WINDOW_SAMPLES:]
    expected_window = pressure_terminal_trend(terminal_window_samples)
    recorded_window = analysis["terminal_window"]
    for key, expected in expected_window.items():
        recorded = recorded_window[key]
        if isinstance(expected, float):
            if not pressure_float_equal(recorded, expected):
                raise ExperimentError(f"Pressure terminal window {key} disagrees with telemetry")
        elif recorded != expected:
            raise ExperimentError(f"Pressure terminal window {key} disagrees with telemetry")

    final = pre_reset[-1]
    reset_keys = (
        "world",
        "sleep",
        "census",
        "material_counts_by_id",
        "matter_count",
        "water_count",
        "steam_count",
        "relief_seam_wood_cells",
        "top_relief_seam_wood_cells",
        "bottom_relief_seam_wood_cells",
        "relief_seam_open_cells",
        "top_relief_seam_open_cells",
        "bottom_relief_seam_open_cells",
        "relief_seam_through_open_lanes",
        "top_relief_seam_through_open_lanes",
        "bottom_relief_seam_through_open_lanes",
        "top_relief_seam_combusting_cells",
        "bottom_relief_seam_combusting_cells",
        "relief_seam_combusting_cells",
        "top_relief_seam_flame_event_cells",
        "bottom_relief_seam_flame_event_cells",
        "relief_seam_flame_event_cells",
        "top_relief_seam_fuel_progress_sum",
        "top_relief_seam_fuel_progress_max",
        "bottom_relief_seam_fuel_progress_sum",
        "bottom_relief_seam_fuel_progress_max",
        "relief_seam_fuel_progress_sum",
        "relief_seam_fuel_progress_max",
        "top_relief_seam_adjacent_pressure_medium_cells",
        "bottom_relief_seam_adjacent_pressure_medium_cells",
        "relief_seam_adjacent_pressure_medium_cells",
        "top_relief_seam_max_adjacent_pressure",
        "bottom_relief_seam_max_adjacent_pressure",
        "relief_seam_max_adjacent_pressure",
        "steam_in_relief_seam_cells",
        "outside_chamber_steam_cells",
        "chamber_pressure_cell_count",
        "chamber_mean_pressure",
        "chamber_max_pressure",
        "invalid_material_count",
        "nonfinite_temperature_count",
        "nonfinite_pressure_count",
        "state_hash",
        "physical_state_hash",
    )
    observable_reset_equal = all(reset[key] == tick0[key] for key in reset_keys)
    recorded_reset_exact = analysis["metrics"]["reset_exact_equivalence"]
    if recorded_reset_exact and not observable_reset_equal:
        raise ExperimentError(
            "Pressure exact-reset claim contradicts observable reset telemetry"
        )
    invalid_occurrences = sum(sample["invalid_material_count"] for sample in pre_reset)
    nonfinite_occurrences = sum(
        sample["nonfinite_temperature_count"] + sample["nonfinite_pressure_count"]
        for sample in pre_reset
    )
    end_mean = expected_window["end_mean_pressure"]
    end_max = expected_window["end_max_pressure"]
    pressure_relieved = (
        first_relief is not None
        and vent_reference_mean is not None
        and vent_reference_max is not None
        and end_mean is not None
        and end_max is not None
        and end_mean < vent_reference_mean
        and end_max < vent_reference_max
        and end_mean < pre_opening_peak_mean
        and end_max < pre_opening_peak_max
    )
    expected_metrics = {
        "first_pressure_activity_tick": sample_identity(first_pressure)[0],
        "first_pressure_activity_sample_sequence": sample_identity(first_pressure)[1],
        "first_wood_damage_tick": sample_identity(first_damage)[0],
        "first_wood_damage_sample_sequence": sample_identity(first_damage)[1],
        "first_rupture_tick": sample_identity(first_rupture)[0],
        "first_rupture_sample_sequence": sample_identity(first_rupture)[1],
        "first_persistent_opening_tick": sample_identity(opening_start)[0],
        "first_persistent_opening_sample_sequence": sample_identity(opening_start)[1],
        "persistent_opening_confirmed_tick": sample_identity(confirmed)[0],
        "persistent_opening_confirmed_sample_sequence": sample_identity(confirmed)[1],
        "first_steam_in_relief_seam_tick": sample_identity(first_seam_steam)[0],
        "first_steam_in_relief_seam_sample_sequence": sample_identity(
            first_seam_steam
        )[1],
        "first_outside_chamber_steam_tick": sample_identity(first_exterior)[0],
        "first_outside_chamber_steam_sample_sequence": sample_identity(first_exterior)[1],
        "first_post_confirmation_reseal_tick": sample_identity(first_reseal)[0],
        "first_post_confirmation_reseal_sample_sequence": sample_identity(
            first_reseal
        )[1],
        "first_post_opening_relief_tick": sample_identity(first_relief)[0],
        "first_post_opening_relief_sample_sequence": sample_identity(first_relief)[1],
        "first_relief_seam_combustion_tick": sample_identity(first_seam_combustion)[0],
        "first_relief_seam_combustion_sample_sequence": sample_identity(
            first_seam_combustion
        )[1],
        "first_relief_seam_fuel_progress_tick": sample_identity(
            first_seam_fuel_progress
        )[0],
        "first_relief_seam_fuel_progress_sample_sequence": sample_identity(
            first_seam_fuel_progress
        )[1],
        "peak_chamber_mean_pressure": peak_mean["chamber_mean_pressure"],
        "peak_chamber_mean_pressure_tick": peak_mean["sim_tick"],
        "peak_chamber_mean_pressure_sample_sequence": peak_mean["sample_sequence"],
        "peak_chamber_max_pressure": peak_max["chamber_max_pressure"],
        "peak_chamber_max_pressure_tick": peak_max["sim_tick"],
        "peak_chamber_max_pressure_sample_sequence": peak_max["sample_sequence"],
        "peak_pressure_active_cells": peak_activity["census"]["pressure_active_cells"],
        "peak_pressure_active_tick": peak_activity["sim_tick"],
        "peak_pressure_active_sample_sequence": peak_activity["sample_sequence"],
        "pre_opening_peak_chamber_mean_pressure": pre_opening_peak_mean,
        "pre_opening_peak_chamber_max_pressure": pre_opening_peak_max,
        "vent_reference_chamber_mean_pressure": vent_reference_mean,
        "vent_reference_chamber_max_pressure": vent_reference_max,
        "post_opening_chamber_mean_pressure": None
        if first_post is None
        else first_post["chamber_mean_pressure"],
        "post_opening_chamber_max_pressure": None
        if first_post is None
        else first_post["chamber_max_pressure"],
        "terminal_chamber_mean_pressure": end_mean,
        "terminal_chamber_max_pressure": end_max,
        "terminal_pressure_relieved": pressure_relieved,
        "through_opening_confirmation_relief_seam_combusting_cells_peak": (
            through_confirmation_combusting_peak
        ),
        "through_opening_confirmation_relief_seam_flame_event_cells_peak": (
            through_confirmation_flame_peak
        ),
        "through_opening_confirmation_relief_seam_fuel_progress_sum_peak": (
            through_confirmation_fuel_sum_peak
        ),
        "through_opening_confirmation_relief_seam_fuel_progress_max": (
            through_confirmation_fuel_max
        ),
        "opening_confirmation_relief_seam_combusting_cells": None
        if confirmed is None
        else confirmed["relief_seam_combusting_cells"],
        "opening_confirmation_relief_seam_flame_event_cells": None
        if confirmed is None
        else confirmed["relief_seam_flame_event_cells"],
        "opening_confirmation_relief_seam_fuel_progress_sum": None
        if confirmed is None
        else confirmed["relief_seam_fuel_progress_sum"],
        "opening_confirmation_relief_seam_fuel_progress_max": None
        if confirmed is None
        else confirmed["relief_seam_fuel_progress_max"],
        "opening_confirmation_relief_seam_adjacent_pressure_medium_cells": None
        if confirmed is None
        else confirmed["relief_seam_adjacent_pressure_medium_cells"],
        "opening_confirmation_relief_seam_max_adjacent_pressure": None
        if confirmed is None
        else confirmed["relief_seam_max_adjacent_pressure"],
        "opening_confirmation_adjacent_pressure_at_or_above_wood_rupture_threshold": None
        if confirmed is None
        else (
            confirmed["relief_seam_adjacent_pressure_medium_cells"] != 0
            and confirmed["relief_seam_max_adjacent_pressure"]
            >= WOOD_RUPTURE_THRESHOLD
        ),
        "first_opening_relief_seam_adjacent_pressure_medium_cells": None
        if first_rupture is None
        else first_rupture["relief_seam_adjacent_pressure_medium_cells"],
        "first_opening_relief_seam_max_adjacent_pressure": None
        if first_rupture is None
        else first_rupture["relief_seam_max_adjacent_pressure"],
        "first_opening_adjacent_pressure_at_or_above_wood_rupture_threshold": None
        if first_rupture is None
        else (
            first_rupture["relief_seam_adjacent_pressure_medium_cells"] != 0
            and first_rupture["relief_seam_max_adjacent_pressure"]
            >= WOOD_RUPTURE_THRESHOLD
        ),
        "wood_rupture_threshold": WOOD_RUPTURE_THRESHOLD,
        "final_relief_seam_wood_cells": final["relief_seam_wood_cells"],
        "final_top_relief_seam_wood_cells": final["top_relief_seam_wood_cells"],
        "final_bottom_relief_seam_wood_cells": final[
            "bottom_relief_seam_wood_cells"
        ],
        "final_relief_seam_open_cells": final["relief_seam_open_cells"],
        "final_top_relief_seam_open_cells": final["top_relief_seam_open_cells"],
        "final_bottom_relief_seam_open_cells": final[
            "bottom_relief_seam_open_cells"
        ],
        "final_relief_seam_through_open_lanes": final[
            "relief_seam_through_open_lanes"
        ],
        "final_top_relief_seam_through_open_lanes": final[
            "top_relief_seam_through_open_lanes"
        ],
        "final_bottom_relief_seam_through_open_lanes": final[
            "bottom_relief_seam_through_open_lanes"
        ],
        "final_steam_in_relief_seam_cells": final["steam_in_relief_seam_cells"],
        "outside_chamber_steam_peak": max(
            sample["outside_chamber_steam_cells"] for sample in pre_reset
        ),
        "final_outside_chamber_steam_cells": final["outside_chamber_steam_cells"],
        "final_matter_count": final["matter_count"],
        "matter_count_delta": final["matter_count"] - tick0["matter_count"],
        "final_water_count": final["water_count"],
        "water_count_delta": final["water_count"] - tick0["water_count"],
        "final_steam_count": final["steam_count"],
        "steam_count_delta": final["steam_count"] - tick0["steam_count"],
        "final_pressure_active_cells": final["census"]["pressure_active_cells"],
        "final_thermal_active_cells": final["census"]["thermal_active_cells"],
        "final_reaction_active_cells": final["census"]["reaction_active_cells"],
        "invalid_material_occurrences": invalid_occurrences,
        "nonfinite_field_occurrences": nonfinite_occurrences,
        # The worker compares the complete GPU snapshot. The raw telemetry is a
        # strict necessary-condition cross-check, but cannot prove equality of
        # hidden proposal/claim/activity/chunk/parameter buffers on its own.
        "reset_exact_equivalence": recorded_reset_exact,
    }
    metrics = analysis["metrics"]
    for key, expected in expected_metrics.items():
        recorded = metrics[key]
        if isinstance(expected, float):
            if not pressure_float_equal(recorded, expected):
                raise ExperimentError(f"Pressure analysis metric {key} disagrees with telemetry")
        elif recorded != expected:
            raise ExperimentError(f"Pressure analysis metric {key} disagrees with telemetry")

    expected_flag_values = {
        "only_one_relief_seam_ruptured": (
            any(
                sample["top_relief_seam_through_open_lanes"] > 0
                for sample in pre_reset
            )
            ^ any(
                sample["bottom_relief_seam_through_open_lanes"] > 0
                for sample in pre_reset
            )
        ),
        "high_terminal_pressure_activity": (
            final["census"]["pressure_active_cells"] >= 256
            and final["census"]["pressure_active_cells"] * 4
            > peak_activity["census"]["pressure_active_cells"]
        ),
        "long_pressure_tail": end_mean is not None
        and end_mean * 2.0 > tick0["chamber_mean_pressure"],
        "persistent_vent_plume": final["outside_chamber_steam_cells"] > 0,
        # Runnable chunks alone do not imply active work; Pressure explicitly
        # does not require whole-world all-sleep.
        "terminal_activity_remains": (
            final["census"]["any_active_cells"] > 0
            or final["census"]["active_chunks"] > 0
        ),
    }
    expected_flags = {
        **expected_flag_values,
        "reasons": [name for name, value in expected_flag_values.items() if value],
    }
    if analysis["review_flags"] != expected_flags:
        raise ExperimentError("Pressure review flags disagree with raw telemetry")
    if analysis["causal_classification"] != expected_causal_classification:
        raise ExperimentError(
            "Pressure causal classification disagrees with raw seam-confound telemetry"
        )

    expected_statuses = {
        "pressure_activity_observed": "pass" if first_pressure is not None else "fail",
        "relief_seam_damaged": "pass" if first_damage is not None else "fail",
        "persistent_opening_created": (
            "pass" if confirmed is not None and first_reseal is None else "fail"
        ),
        "pressure_opening_precedes_combustion": (
            "pass"
            if expected_causal_classification
            == "pressure_opening_precedes_combustion"
            else "fail"
        ),
        "exterior_vent_observed": "pass" if first_exterior is not None else "fail",
        "post_opening_pressure_relieved": (
            "pass" if pressure_relieved else "unknown"
        ),
        "terminal_pressure_not_runaway": (
            "unknown"
            if len(terminal_window_samples) < TERMINAL_WINDOW_SAMPLES
            else "fail"
            if expected_window["unbounded_growth"]
            else "pass"
        ),
        "no_invalid_materials": "pass" if invalid_occurrences == 0 else "fail",
        "no_nonfinite_fields": "pass" if nonfinite_occurrences == 0 else "fail",
        "exact_reset": "pass" if recorded_reset_exact else "fail",
    }
    recorded_statuses = {
        name: predicate["status"] for name, predicate in analysis["predicates"].items()
    }
    if recorded_statuses != expected_statuses:
        raise ExperimentError("Pressure predicate statuses disagree with raw telemetry")
    expected_verdict = pressure_expected_verdict(
        expected_statuses, expected_flags, expected_causal_classification
    )
    if analysis["verdict"] != expected_verdict:
        raise ExperimentError("Pressure verdict disagrees with predicates and review flags")

    expected_events: list[tuple[str, int, int | None]] = [
        ("lifecycle_started", 0, None),
        ("pristine_reset_completed", 0, None),
        ("tick0_captured", 0, tick0["sample_sequence"]),
    ]
    running_mean = tick0["chamber_mean_pressure"]
    running_max = tick0["chamber_max_pressure"]
    running_activity = tick0["census"]["pressure_active_cells"]

    def append_observation_events(sample: dict[str, Any]) -> None:
        nonlocal running_mean, running_max, running_activity
        identity = (sample["sim_tick"], sample["sample_sequence"])
        if sample is first_pressure:
            expected_events.append(("pressure_activity_observed", *identity))
        if sample is first_damage:
            expected_events.append(("relief_seam_damage_observed", *identity))
        if sample is first_rupture:
            expected_events.append(("rupture_observed", *identity))
        if sample is first_seam_combustion:
            expected_events.append(("relief_seam_combustion_observed", *identity))
        if sample is first_seam_fuel_progress:
            expected_events.append(("relief_seam_fuel_progress_observed", *identity))
        if sample is first_seam_steam:
            expected_events.append(("relief_seam_steam_observed", *identity))
        if sample is first_exterior:
            expected_events.append(("exterior_vent_observed", *identity))
        if sample is first_reseal:
            expected_events.append(("post_confirmation_reseal_observed", *identity))
        if sample["chamber_mean_pressure"] > running_mean:
            running_mean = sample["chamber_mean_pressure"]
            expected_events.append(("new_peak_chamber_mean_pressure", *identity))
        if sample["chamber_max_pressure"] > running_max:
            running_max = sample["chamber_max_pressure"]
            expected_events.append(("new_peak_chamber_max_pressure", *identity))
        if sample["census"]["pressure_active_cells"] > running_activity:
            running_activity = sample["census"]["pressure_active_cells"]
            expected_events.append(("new_peak_pressure_activity", *identity))
        if sample is first_relief:
            expected_events.append(("post_opening_pressure_relief_observed", *identity))

    append_observation_events(tick1)
    expected_events.append(
        ("tick1_captured", tick1["sim_tick"], tick1["sample_sequence"])
    )
    opening_streak_samples: list[dict[str, Any]] = []
    if tick1["relief_seam_through_open_lanes"] > 0:
        opening_streak_samples.append(tick1)
        expected_events.append(
            (
                "persistent_opening_streak_started",
                tick1["sim_tick"],
                tick1["sample_sequence"],
            )
        )
    confirmation_seen = False
    for sample in diagnostics:
        identity = (sample["sim_tick"], sample["sample_sequence"])
        if not confirmation_seen:
            if sample["relief_seam_through_open_lanes"] == 0:
                if opening_streak_samples:
                    expected_events.append(("persistent_opening_streak_broken", *identity))
                    opening_streak_samples = []
            else:
                if not opening_streak_samples:
                    expected_events.append(("persistent_opening_streak_started", *identity))
                opening_streak_samples.append(sample)
                if len(opening_streak_samples) == CONSECUTIVE_PERSISTENT_OPENING:
                    expected_events.append(("persistent_opening_confirmed", *identity))
                    expected_events.append(("post_opening_observation_started", *identity))
                    confirmation_seen = True
        append_observation_events(sample)
    for sample in post_opening:
        append_observation_events(sample)
        if (
            terminal_reason == "post-opening-observation-complete"
            and sample is terminal
        ):
            expected_events.append(
                (
                    "post_opening_observation_completed",
                    sample["sim_tick"],
                    sample["sample_sequence"],
                )
            )
    terminal_identity = (terminal["sim_tick"], terminal["sample_sequence"])
    expected_events.extend(
        [
            ("terminal_selected", *terminal_identity),
            ("reset_started", *terminal_identity),
            (
                "reset_comparison_completed",
                reset["sim_tick"],
                reset["sample_sequence"],
            ),
            ("worker_completed", reset["sim_tick"], reset["sample_sequence"]),
        ]
    )
    validate_pressure_event_contract(events, expected_events)

    fallback_candidates = [
        sample
        for sample in pre_reset
        if (
            sample["phase"] == "pressurizing"
            and sample["sim_tick"] % (DIAGNOSTIC_INTERVAL * 128) == 0
        )
        or (
            confirmed is not None
            and sample["phase"] == "post-opening-observation"
            and (sample["sim_tick"] - confirmed["sim_tick"]) % 32 == 0
        )
    ][-8:]
    expected_frames = pressure_expected_frame_badges(
        tick0,
        tick1,
        first_pressure,
        first_damage,
        first_rupture,
        opening_streak,
        first_reseal,
        first_exterior,
        peak_max,
        peak_activity,
        first_relief,
        terminal,
        reset,
        fallback_candidates,
        terminal_reason,
    )
    validate_pressure_frame_contract(frames, expected_frames)
    return analysis, frames_doc, samples, events


def validate_telemetry(
    run_dir: Path, manifest: dict[str, Any]
) -> tuple[dict[str, Any], dict[str, Any], list[dict[str, Any]], list[dict[str, Any]]]:
    contract = contract_for_manifest(manifest)
    if contract is SAND_CONTRACT:
        return validate_sand_telemetry(run_dir, manifest)
    if contract is WATER_CONTRACT:
        return validate_water_telemetry(run_dir, manifest)
    if contract is FIRE_CONTRACT:
        return validate_fire_telemetry(run_dir, manifest)
    if contract is PRESSURE_CONTRACT:
        return validate_pressure_telemetry(run_dir, manifest)
    raise ExperimentError(f"unsupported telemetry contract: {contract.scenario}")


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
        reason = frame.get("reason")
        if reason is None:
            reason = "+".join(badge["kind"] for badge in frame["badges"])
        item = {
                "ordinal": frame["ordinal"],
                "reason": reason,
                "sim_tick": frame["sim_tick"],
                "sample_sequence": frame["sample_sequence"],
                "state_hash": frame["state_hash"],
                "full_png": full_path.relative_to(run_dir).as_posix(),
                "crop_png": crop_path.relative_to(run_dir).as_posix(),
            }
        if "badges" in frame:
            item["badges"] = frame["badges"]
        output.append(item)
    return output


def contact_sheet_caption_lines(
    item: dict[str, Any], sample: dict[str, Any] | None
) -> tuple[str, ...]:
    reason = str(item["reason"])
    if len(reason) > 34:
        reason = reason[:31] + "..."
    identity = (
        f"#{item['ordinal']} {reason} | sim {item['sim_tick']} | "
        f"sample {item['sample_sequence']}"
    )
    if sample is None:
        return (identity,)
    if sample["sample_sequence"] != item["sample_sequence"]:
        raise ExperimentError("contact-sheet telemetry join has a sample identity mismatch")
    if "state_hash" in item and item["state_hash"] != sample["state_hash"]:
        raise ExperimentError("contact-sheet telemetry join has a state-hash mismatch")
    census = sample["census"]
    if sample.get("scenario") == FIRE_CONTRACT.scenario:
        return (
            identity,
            (
                f"Reaction {census['reaction_active_cells']} | "
                f"Thermal {census['thermal_active_cells']} | "
                f"Wood {sample['wood_count']} | Oil {sample['oil_count']}"
            ),
            (
                f"Smoke {sample['smoke_count']} | Ice/Water/Steam "
                f"{sample['ice_count']}/{sample['water_count']}/{sample['steam_count']}"
            ),
            f"State {sample['state_hash']}",
        )
    if sample.get("scenario") == PRESSURE_CONTRACT.scenario:
        return (
            identity,
            f"Pressure active {census['pressure_active_cells']}",
            (
                f"Chamber mean/max {sample['chamber_mean_pressure']:.3f}/"
                f"{sample['chamber_max_pressure']:.3f}"
            ),
            (
                "Seam Wood/open/through "
                f"{sample['relief_seam_wood_cells']}/"
                f"{sample['relief_seam_open_cells']}/"
                f"{sample['relief_seam_through_open_lanes']} | "
                f"Outside Steam {sample['outside_chamber_steam_cells']}"
            ),
            f"State {sample['state_hash']}",
        )
    return (
        identity,
        (
            f"Active cells {census['any_active_cells']} | "
            f"Runnable {census['runnable_chunks']} | Sleeping {census['sleeping_chunks']}"
        ),
        f"State {sample['state_hash']}",
    )


def create_contact_sheet_bytes(
    run_dir: Path,
    screenshots: list[dict[str, Any]],
    samples: list[dict[str, Any]] | None = None,
) -> bytes:
    Image, ImageDraw, ImageOps = pillow_modules()
    columns = 3
    panel_width = 420
    samples_by_sequence = (
        {} if samples is None else {sample["sample_sequence"]: sample for sample in samples}
    )
    captions: list[tuple[str, ...]] = []
    caption_top = 374
    caption_line_height = 18
    caption_bottom_padding = 10
    probe = ImageDraw.Draw(Image.new("RGB", (1, 1)))
    caption_bottom = 0
    for item in screenshots:
        sample = samples_by_sequence.get(item["sample_sequence"])
        if samples is not None and sample is None:
            raise ExperimentError("contact-sheet frame sample is absent from telemetry")
        lines = contact_sheet_caption_lines(item, sample)
        captions.append(lines)
        for line_index, label in enumerate(lines):
            bbox = probe.textbbox(
                (12, caption_top + line_index * caption_line_height), label
            )
            caption_bottom = max(caption_bottom, bbox[3])

    # Historical four-line layouts remain 450 px high. Pressure adds a fifth
    # required State-hash line, so size the panel from the actual Pillow text
    # bounds instead of clipping the final evidence caption.
    panel_height = max(450, caption_bottom + caption_bottom_padding)
    rows = (len(screenshots) + columns - 1) // columns
    sheet = Image.new(
        "RGB", (columns * panel_width, max(1, rows) * panel_height), "#11151c"
    )
    draw = ImageDraw.Draw(sheet)
    for index, item in enumerate(screenshots):
        column = index % columns
        row = index // columns
        left = column * panel_width
        top = row * panel_height
        crop = Image.open(run_dir / item["crop_png"]).convert("RGB")
        thumb = ImageOps.contain(crop, (390, 360))
        x = left + (panel_width - thumb.width) // 2
        y = top + 8
        sheet.paste(thumb, (x, y))
        for line_index, label in enumerate(captions[index]):
            draw.text(
                (left + 12, top + caption_top + line_index * caption_line_height),
                label,
                fill="#f4f7fb",
            )
        draw.rectangle(
            (left + 2, top + 2, left + panel_width - 3, top + panel_height - 3),
            outline="#506078",
            width=2,
        )
    return png_bytes(sheet)


def water_remediation_summary(analysis: dict[str, Any]) -> dict[str, Any]:
    metrics = analysis["metrics"]
    final_any_active_cells = (
        metrics["final_active_water_empty_surface_cells"]
        + metrics["final_active_water_oil_interface_cells"]
        + metrics["final_active_other_cells"]
    )
    return {
        "active_cell_classification_rule": metrics[
            "active_cell_classification_rule"
        ],
        "max_water_outside_outer_basin_cells": metrics[
            "max_water_outside_outer_basin_cells"
        ],
        "final_water_outside_outer_basin_cells": metrics[
            "final_water_outside_outer_basin_cells"
        ],
        "final_active_water_empty_surface_cells": metrics[
            "final_active_water_empty_surface_cells"
        ],
        "final_active_water_oil_interface_cells": metrics[
            "final_active_water_oil_interface_cells"
        ],
        "final_active_other_cells": metrics["final_active_other_cells"],
        "final_any_active_cells": final_any_active_cells,
    }


def fire_heat_summary(analysis: dict[str, Any]) -> dict[str, Any]:
    metrics = analysis["metrics"]
    lifecycle = analysis["lifecycle"]
    return {
        "first_combustion_tick": metrics["first_combustion_tick"],
        "first_smoke_tick": metrics["first_smoke_tick"],
        "first_phase_transition_tick": metrics["first_phase_transition_tick"],
        "fuel_substantially_consumed_tick": metrics[
            "fuel_substantially_consumed_tick"
        ],
        "peak_reaction_cells": metrics["peak_reaction_cells"],
        "peak_reaction_tick": metrics["peak_reaction_tick"],
        "peak_thermal_cells": metrics["peak_thermal_cells"],
        "peak_thermal_tick": metrics["peak_thermal_tick"],
        "peak_smoke_count": metrics["peak_smoke_count"],
        "peak_smoke_tick": metrics["peak_smoke_tick"],
        "terminal_reason": lifecycle["terminal_reason"],
        "reaction_zero_tick": metrics["reaction_zero_tick"],
        "confirmed_reaction_zero_tick": metrics["confirmed_reaction_zero_tick"],
        "post_reaction_thermal_cells": metrics["post_reaction_thermal_cells"],
        "post_reaction_final_thermal_cells": metrics[
            "post_reaction_final_thermal_cells"
        ],
        "post_reaction_min_thermal_cells": metrics["post_reaction_min_thermal_cells"],
        "post_reaction_restart_samples": metrics["post_reaction_restart_samples"],
        "fuel_consumed": metrics["fuel_consumed"],
        "wood_count_delta": metrics["wood_count_delta"],
        "oil_count_delta": metrics["oil_count_delta"],
        "reset_exact_equivalence": metrics["reset_exact_equivalence"],
    }


def pressure_candidate_blocker(analysis: dict[str, Any]) -> dict[str, Any]:
    causal_classification = analysis["causal_classification"]
    causal_blocked = causal_classification != "pressure_opening_precedes_combustion"
    failed_predicates = sorted(
        name
        for name in PRESSURE_PREDICATE_NAMES
        if analysis["predicates"][name]["status"] == "fail"
    )
    details: list[dict[str, Any]] = []
    if causal_blocked:
        details.append(
            {
                "kind": "causal_classification",
                "classification": causal_classification,
                "detail": analysis["predicates"][
                    "pressure_opening_precedes_combustion"
                ]["detail"],
            }
        )
    details.extend(
        {
            "kind": "hard_predicate_failure",
            "predicate": name,
            "detail": analysis["predicates"][name]["detail"],
        }
        for name in failed_predicates
    )
    blocker = causal_blocked or bool(failed_predicates)
    classification = (
        causal_classification
        if causal_blocked
        else "hard_predicate_failure"
        if failed_predicates
        else None
    )
    return {
        "candidate_blocker": blocker,
        "candidate_blocker_classification": classification,
        "candidate_blocker_details": details,
        "failed_hard_predicates": failed_predicates,
    }


def pressure_burst_summary(analysis: dict[str, Any]) -> dict[str, Any]:
    metrics = analysis["metrics"]
    lifecycle = analysis["lifecycle"]
    causal_classification = analysis["causal_classification"]
    blocker = pressure_candidate_blocker(analysis)
    return {
        "causal_classification": causal_classification,
        **blocker,
        # v1 compatibility aliases: these describe whether a completed scratch
        # run is eligible to advance to candidate publication.
        "scratch_candidate_blocker": blocker["candidate_blocker"],
        "scratch_blocker_classification": blocker[
            "candidate_blocker_classification"
        ],
        "terminal_reason": lifecycle["terminal_reason"],
        "first_pressure_activity_tick": metrics["first_pressure_activity_tick"],
        "first_wood_damage_tick": metrics["first_wood_damage_tick"],
        "first_rupture_tick": metrics["first_rupture_tick"],
        "persistent_opening_confirmed_tick": metrics[
            "persistent_opening_confirmed_tick"
        ],
        "first_steam_in_relief_seam_tick": metrics[
            "first_steam_in_relief_seam_tick"
        ],
        "first_outside_chamber_steam_tick": metrics[
            "first_outside_chamber_steam_tick"
        ],
        "first_post_confirmation_reseal_tick": metrics[
            "first_post_confirmation_reseal_tick"
        ],
        "first_post_opening_relief_tick": metrics[
            "first_post_opening_relief_tick"
        ],
        "first_relief_seam_combustion_tick": metrics[
            "first_relief_seam_combustion_tick"
        ],
        "first_relief_seam_fuel_progress_tick": metrics[
            "first_relief_seam_fuel_progress_tick"
        ],
        "through_opening_confirmation_combusting_peak": metrics[
            "through_opening_confirmation_relief_seam_combusting_cells_peak"
        ],
        "through_opening_confirmation_flame_event_peak": metrics[
            "through_opening_confirmation_relief_seam_flame_event_cells_peak"
        ],
        "through_opening_confirmation_fuel_progress_sum_peak": metrics[
            "through_opening_confirmation_relief_seam_fuel_progress_sum_peak"
        ],
        "through_opening_confirmation_fuel_progress_max": metrics[
            "through_opening_confirmation_relief_seam_fuel_progress_max"
        ],
        "opening_confirmation_combusting_cells": metrics[
            "opening_confirmation_relief_seam_combusting_cells"
        ],
        "opening_confirmation_flame_event_cells": metrics[
            "opening_confirmation_relief_seam_flame_event_cells"
        ],
        "opening_confirmation_fuel_progress_sum": metrics[
            "opening_confirmation_relief_seam_fuel_progress_sum"
        ],
        "opening_confirmation_fuel_progress_max": metrics[
            "opening_confirmation_relief_seam_fuel_progress_max"
        ],
        "opening_confirmation_adjacent_pressure_medium_cells": metrics[
            "opening_confirmation_relief_seam_adjacent_pressure_medium_cells"
        ],
        "opening_confirmation_max_adjacent_pressure": metrics[
            "opening_confirmation_relief_seam_max_adjacent_pressure"
        ],
        "opening_confirmation_pressure_at_or_above_rupture_threshold": metrics[
            "opening_confirmation_adjacent_pressure_at_or_above_wood_rupture_threshold"
        ],
        "first_opening_adjacent_pressure_medium_cells": metrics[
            "first_opening_relief_seam_adjacent_pressure_medium_cells"
        ],
        "first_opening_max_adjacent_pressure": metrics[
            "first_opening_relief_seam_max_adjacent_pressure"
        ],
        "first_opening_pressure_at_or_above_rupture_threshold": metrics[
            "first_opening_adjacent_pressure_at_or_above_wood_rupture_threshold"
        ],
        "wood_rupture_threshold": metrics["wood_rupture_threshold"],
        "vent_reference_chamber_mean_pressure": metrics[
            "vent_reference_chamber_mean_pressure"
        ],
        "vent_reference_chamber_max_pressure": metrics[
            "vent_reference_chamber_max_pressure"
        ],
        "peak_chamber_mean_pressure": metrics["peak_chamber_mean_pressure"],
        "peak_chamber_max_pressure": metrics["peak_chamber_max_pressure"],
        "peak_pressure_active_cells": metrics["peak_pressure_active_cells"],
        "terminal_chamber_mean_pressure": metrics[
            "terminal_chamber_mean_pressure"
        ],
        "terminal_chamber_max_pressure": metrics["terminal_chamber_max_pressure"],
        "terminal_pressure_relieved": metrics["terminal_pressure_relieved"],
        "final_relief_seam_open_cells": metrics["final_relief_seam_open_cells"],
        "final_top_relief_seam_open_cells": metrics[
            "final_top_relief_seam_open_cells"
        ],
        "final_bottom_relief_seam_open_cells": metrics[
            "final_bottom_relief_seam_open_cells"
        ],
        "final_relief_seam_through_open_lanes": metrics[
            "final_relief_seam_through_open_lanes"
        ],
        "final_top_relief_seam_through_open_lanes": metrics[
            "final_top_relief_seam_through_open_lanes"
        ],
        "final_bottom_relief_seam_through_open_lanes": metrics[
            "final_bottom_relief_seam_through_open_lanes"
        ],
        "outside_chamber_steam_peak": metrics["outside_chamber_steam_peak"],
        "final_outside_chamber_steam_cells": metrics[
            "final_outside_chamber_steam_cells"
        ],
        "terminal_window": analysis["terminal_window"],
        "review_flags": analysis["review_flags"],
        "reset_exact_equivalence": metrics["reset_exact_equivalence"],
    }


def render_report_markdown(
    manifest: dict[str, Any],
    analysis: dict[str, Any],
    samples: list[dict[str, Any]],
    events: list[dict[str, Any]],
    screenshots: list[dict[str, Any]],
) -> str:
    contract = contract_for_manifest(manifest)
    lines = [
        f"# Powdergame {contract.title} Experiment Report",
        "",
        f"- Experiment: `{manifest['experiment_id']}`",
        f"- Run ID: `{manifest['run_id']}`",
        f"- Source: `{manifest['source']['sha']}` on `{manifest['source']['branch']}` "
        f"(`{manifest['source']['git_state']}`)",
        f"- Binary SHA-256: `{manifest['binary']['sha256']}`",
        f"- Automatic verdict: **{analysis['verdict']}**",
        f"- Samples / events / frames: {len(samples)} / {len(events)} / {len(screenshots)}",
    ]
    if contract.records_run_mode:
        lines.append(f"- Run mode: `{manifest['run_mode']}`")
    lines.extend(
        [
            "",
            "The automatic verdict is worker telemetry, not user acceptance or G8-B/G8-C closure.",
            "",
            "## Predicates",
            "",
            "| Predicate | Status | Detail |",
            "|---|---|---|",
        ]
    )
    for name in sorted(contract.predicate_names):
        predicate = analysis["predicates"][name]
        detail = predicate["detail"].replace("|", "\\|").replace("\n", " ")
        lines.append(f"| `{name}` | {predicate['status']} | {detail} |")
    lines.extend(
        [
            "",
            "## Frames",
            "",
            (
                "| # | Kind | Reason | Sim tick | Sample | State hash | Full | World crop |"
                if contract.records_run_mode
                else "| # | Reason | Sim tick | Sample | State hash | Full | World crop |"
            ),
            (
                "|---:|---|---|---:|---:|---|---|---|"
                if contract.records_run_mode
                else "|---:|---|---:|---:|---|---|---|"
            ),
        ]
    )
    for item in screenshots:
        kind = f" {item['kind']} |" if contract.records_run_mode else ""
        lines.append(
            f"| {item['ordinal']} |{kind} {item['reason']} | {item['sim_tick']} | "
            f"{item['sample_sequence']} | `{item['state_hash']}` | "
            f"[{Path(item['full_png']).name}](../{item['full_png']}) | "
            f"[{Path(item['crop_png']).name}](../{item['crop_png']}) |"
        )
    if contract is WATER_CONTRACT:
        remediation = water_remediation_summary(analysis)
        lines.extend(
            [
                "",
                "## Water remediation telemetry",
                "",
                f"- Outside outer basin, maximum / final cells: "
                f"{remediation['max_water_outside_outer_basin_cells']} / "
                f"{remediation['final_water_outside_outer_basin_cells']}",
                f"- Final active Water/Empty surface cells: "
                f"{remediation['final_active_water_empty_surface_cells']}",
                f"- Final active Water/Oil interface cells: "
                f"{remediation['final_active_water_oil_interface_cells']}",
                f"- Final active Other cells: {remediation['final_active_other_cells']}",
                f"- Final any-active total: {remediation['final_any_active_cells']}",
                f"- Classification rule: `{remediation['active_cell_classification_rule']}`",
                "",
                "## Review classification guide",
                "",
                "These are cautious labels for a human reviewer, not findings declared by the runner.",
                "",
            ]
        )
        lines.extend(f"- `{category}`" for category in WATER_FINDING_CLASSIFICATIONS)
        lines.extend(["", "## Visual questions", ""])
        lines.extend(f"- {question}" for question in WATER_VISUAL_QUESTIONS)
    elif contract is FIRE_CONTRACT:
        summary = fire_heat_summary(analysis)
        lines.extend(
            [
                "",
                "## Fire / Heat telemetry",
                "",
                f"- First combustion / Smoke / phase transition ticks: "
                f"{summary['first_combustion_tick']} / {summary['first_smoke_tick']} / "
                f"{summary['first_phase_transition_tick']}",
                f"- Peak Reaction / Thermal cells: {summary['peak_reaction_cells']} at "
                f"{summary['peak_reaction_tick']} / {summary['peak_thermal_cells']} at "
                f"{summary['peak_thermal_tick']}",
                f"- Peak Smoke count: {summary['peak_smoke_count']} at "
                f"{summary['peak_smoke_tick']}",
                f"- Terminal / first zero / confirmed zero: "
                f"{summary['terminal_reason']} / {summary['reaction_zero_tick']} / "
                f"{summary['confirmed_reaction_zero_tick']}",
                f"- Thermal tail start / final / minimum: "
                f"{summary['post_reaction_thermal_cells']} / "
                f"{summary['post_reaction_final_thermal_cells']} / "
                f"{summary['post_reaction_min_thermal_cells']}",
                f"- Post-reaction restart samples: "
                f"{summary['post_reaction_restart_samples']}",
                f"- Fuel consumed; Wood / Oil deltas: {summary['fuel_consumed']}; "
                f"{summary['wood_count_delta']} / {summary['oil_count_delta']}",
                f"- Exact reset: {summary['reset_exact_equivalence']}",
            ]
        )
    elif contract is PRESSURE_CONTRACT:
        summary = pressure_burst_summary(analysis)
        lines.extend(
            [
                "",
                "## Pressure Burst telemetry",
                "",
                f"- Causal classification: {summary['causal_classification']}",
                "- Scratch candidate blocker / classification: "
                f"{summary['scratch_candidate_blocker']} / "
                f"{summary['scratch_blocker_classification']}",
                "- First activity / Wood damage / first through-lane rupture ticks: "
                f"{summary['first_pressure_activity_tick']} / "
                f"{summary['first_wood_damage_tick']} / {summary['first_rupture_tick']}",
                "- Persistent opening confirmation / causal exterior Steam ticks: "
                f"{summary['persistent_opening_confirmed_tick']} / "
                f"{summary['first_outside_chamber_steam_tick']}",
                "- Causal seam Steam / post-confirmation reseal / post-vent relief ticks: "
                f"{summary['first_steam_in_relief_seam_tick']} / "
                f"{summary['first_post_confirmation_reseal_tick']} / "
                f"{summary['first_post_opening_relief_tick']}",
                "- Causal vent milestones are confirmation-gated; any earlier raw Steam "
                "counts remain available in telemetry samples.",
                "- First relief-seam combustion / fuel-progress ticks: "
                f"{summary['first_relief_seam_combustion_tick']} / "
                f"{summary['first_relief_seam_fuel_progress_tick']}",
                "- Through-confirmation combustion / flame / fuel sum / fuel max peaks: "
                f"{summary['through_opening_confirmation_combusting_peak']} / "
                f"{summary['through_opening_confirmation_flame_event_peak']} / "
                f"{summary['through_opening_confirmation_fuel_progress_sum_peak']} / "
                f"{summary['through_opening_confirmation_fuel_progress_max']}",
                "- At confirmation combustion / flame / fuel sum / fuel max: "
                f"{summary['opening_confirmation_combusting_cells']} / "
                f"{summary['opening_confirmation_flame_event_cells']} / "
                f"{summary['opening_confirmation_fuel_progress_sum']} / "
                f"{summary['opening_confirmation_fuel_progress_max']}",
                "- First-opening adjacent Pressure-medium cells / max / threshold met: "
                f"{summary['first_opening_adjacent_pressure_medium_cells']} / "
                f"{summary['first_opening_max_adjacent_pressure']} / "
                f"{summary['first_opening_pressure_at_or_above_rupture_threshold']}",
                "- Confirmed-opening adjacent Pressure-medium cells / max / threshold met: "
                f"{summary['opening_confirmation_adjacent_pressure_medium_cells']} / "
                f"{summary['opening_confirmation_max_adjacent_pressure']} / "
                f"{summary['opening_confirmation_pressure_at_or_above_rupture_threshold']} "
                f"(Wood threshold {summary['wood_rupture_threshold']})",
                "- Vent-reference chamber mean / max: "
                f"{summary['vent_reference_chamber_mean_pressure']} / "
                f"{summary['vent_reference_chamber_max_pressure']}",
                "- Peak chamber mean / max: "
                f"{summary['peak_chamber_mean_pressure']} / "
                f"{summary['peak_chamber_max_pressure']}",
                f"- Peak Pressure-active cells: {summary['peak_pressure_active_cells']}",
                "- Terminal chamber mean / max: "
                f"{summary['terminal_chamber_mean_pressure']} / "
                f"{summary['terminal_chamber_max_pressure']}",
                f"- Sustained post-vent Pressure relief: "
                f"{summary['terminal_pressure_relieved']}",
                "- Final raw non-Wood seam cells (total/top/bottom): "
                f"{summary['final_relief_seam_open_cells']} / "
                f"{summary['final_top_relief_seam_open_cells']} / "
                f"{summary['final_bottom_relief_seam_open_cells']}",
                "- Final eight-cell through-open lanes (total/top/bottom): "
                f"{summary['final_relief_seam_through_open_lanes']} / "
                f"{summary['final_top_relief_seam_through_open_lanes']} / "
                f"{summary['final_bottom_relief_seam_through_open_lanes']}",
                "- Outside Steam peak / final: "
                f"{summary['outside_chamber_steam_peak']} / "
                f"{summary['final_outside_chamber_steam_cells']}",
                "- Review flags: "
                + (", ".join(summary["review_flags"]["reasons"]) or "none"),
                f"- Exact reset: {summary['reset_exact_equivalence']}",
            ]
        )
    lines.extend(
        [
            "",
            "## Boundaries",
            "",
            f"- This experiment is {contract.title} only.",
            (
                "- Water Flow and G8-C are outside scope."
                if contract is SAND_CONTRACT
                else "- Sand Fall regression, other Gallery scenarios, and G8-C are outside scope."
            ),
            "- Gallery rendering/diagnostics are not official benchmark timing.",
            "- Review packet generation does not contact an AI reviewer.",
            "",
        ]
    )
    return "\n".join(lines)


def render_review_prompt(manifest: dict[str, Any], analysis: dict[str, Any]) -> str:
    contract = contract_for_manifest(manifest)
    if contract is WATER_CONTRACT:
        remediation = water_remediation_summary(analysis)
        categories = "\n".join(
            f"- `{category}`" for category in WATER_FINDING_CLASSIFICATIONS
        )
        questions = "\n".join(f"- {question}" for question in WATER_VISUAL_QUESTIONS)
        return f"""# Human Review Prompt — Powdergame Water Flow Experiment

This prompt was generated locally and was not sent to an AI or any external service.
Review only the attached `REVIEW_PACKET.zip` for experiment `{manifest['experiment_id']}`,
run `{manifest['run_id']}` in `{manifest['run_mode']}` mode, source
`{manifest['source']['sha']}`, binary `{manifest['binary']['sha256']}`. The worker
automatic verdict is `{analysis['verdict']}`; treat it as a telemetry claim to check,
not as acceptance or a product conclusion.

`REVIEW_PACKET.zip` is a lightweight human-review packet. It does not contain the
frozen executable or source snapshot and cannot independently establish source/binary
forensic identity; use the candidate's sibling `AUDIT_BUNDLE.zip` for that purpose.

Water remediation telemetry reports maximum/final outside-outer-basin cells as
`{remediation['max_water_outside_outer_basin_cells']}` /
`{remediation['final_water_outside_outer_basin_cells']}`. Final active-cell classes are
Water/Empty `{remediation['final_active_water_empty_surface_cells']}`, Water/Oil
`{remediation['final_active_water_oil_interface_cells']}`, and Other
`{remediation['final_active_other_cells']}`, totaling
`{remediation['final_any_active_cells']}` under rule
`{remediation['active_cell_classification_rule']}`. The packet has aggregate counts,
not per-cell neighbor records; do not independently infer the class partition from pixels.

Classify each concrete observation cautiously using one of these review labels:
{categories}

Answer these visual questions from the full frames, crops, joined contact-sheet captions,
and matching telemetry samples:
{questions}

Report missing evidence and ambiguity explicitly. Do not infer other scenarios, G8-C,
performance readiness, or G8-B closure. No action, code change, upload, or external
message is authorized by this prompt.
"""
    if contract is FIRE_CONTRACT:
        summary = fire_heat_summary(analysis)
        return f"""# Human Review Prompt — Powdergame Fire / Heat Experiment

This prompt was generated locally and was not sent to an AI or external service.
Review only `REVIEW_PACKET.zip` for experiment `{manifest['experiment_id']}`, run
`{manifest['run_id']}` in `{manifest['run_mode']}` mode, source
`{manifest['source']['sha']}`, binary `{manifest['binary']['sha256']}`. Treat automatic
verdict `{analysis['verdict']}` as a telemetry claim, not user acceptance or closure.

`REVIEW_PACKET.zip` is a lightweight human-review packet. It does not contain the
frozen executable or source snapshot and cannot independently establish source/binary
forensic identity; use the candidate's sibling `AUDIT_BUNDLE.zip` for that purpose.

Check the causal sequence in the full frames, crops, contact sheet, raw samples, and events:
Wood/Oil production combustion, Smoke creation, heat propagation, Ice/Water/Steam phase
inventory work, finite fuel consumption, three diagnostic Reaction-zero samples, and the
180-tick post-reaction Thermal tail. Report whether the tail remains visible and decreases;
a remaining Thermal tail is not itself a failure. The recorded terminal is
`{summary['terminal_reason']}` with first/confirmed zero ticks
`{summary['reaction_zero_tick']}` / `{summary['confirmed_reaction_zero_tick']}` and tail
start/final/minimum cells `{summary['post_reaction_thermal_cells']}` /
`{summary['post_reaction_final_thermal_cells']}` /
`{summary['post_reaction_min_thermal_cells']}`.

Report concrete mismatches, missing evidence, and ambiguity. Do not infer other scenarios,
G8-C performance, product readiness, or G8-B closure. No upload, external message, code
change, or other action is authorized by this prompt.
"""
    if contract is PRESSURE_CONTRACT:
        summary = pressure_burst_summary(analysis)
        return f"""# Human Review Prompt — Powdergame Pressure Burst Experiment

This prompt was generated locally and was not sent to an AI or external service.
Review only `REVIEW_PACKET.zip` for experiment `{manifest['experiment_id']}`, run
`{manifest['run_id']}` in `{manifest['run_mode']}` mode, source
`{manifest['source']['sha']}`, binary `{manifest['binary']['sha256']}`. Treat automatic
verdict `{analysis['verdict']}` as a telemetry claim, not user acceptance or closure.

Check the causal sequence in the full frames, crops, contact sheet, raw samples, and events:
Pressure activity, authored relief-seam damage, the first complete eight-cell through-open
lane, three-sample persistent opening, zero relief-seam combustion/flame/fuel progress through
opening confirmation, and seam-adjacent Pressure evidence against the Wood rupture threshold.
The machine causal classification is `{summary['causal_classification']}`; first seam
combustion/fuel-progress ticks are `{summary['first_relief_seam_combustion_tick']}` /
`{summary['first_relief_seam_fuel_progress_tick']}`, and through-confirmation
combusting/flame/fuel-sum/fuel-max peaks are
`{summary['through_opening_confirmation_combusting_peak']}` /
`{summary['through_opening_confirmation_flame_event_peak']}` /
`{summary['through_opening_confirmation_fuel_progress_sum_peak']}` /
`{summary['through_opening_confirmation_fuel_progress_max']}`. Then verify Steam in the relief
seam at or before causal exterior venting,
any post-confirmation reseal, sustained post-vent mean/max relief, the 180-tick
post-opening window, and the 64-sample terminal mean/max Pressure trend. Frame badges
sharing one physical state are folded
onto one image. Review flags are `{', '.join(summary['review_flags']['reasons']) or 'none'}`;
these flags require human review and are not automatic defect findings.

Treat raw non-Wood seam `open_cells` as damage diagnostics only. Rupture, persistent
opening, reseal, per-seam opening, causal venting, and the post-opening timer require a
full eight-cell authored seam column made only of Empty, Steam, or Smoke.

Report concrete mismatches, missing evidence, and ambiguity. Do not infer other scenarios,
G8-C performance, product readiness, or G8-B closure. No upload, external message, code
change, or other action is authorized by this prompt.
"""
    return f"""# ChatGPT Review Prompt — Powdergame Sand Fall Experiment

Review only the attached `REVIEW_PACKET.zip` for experiment `{manifest['experiment_id']}`,
run `{manifest['run_id']}`, source `{manifest['source']['sha']}`, binary
`{manifest['binary']['sha256']}`. The worker automatic verdict is
`{analysis['verdict']}`; treat it as a claim to check, not as a conclusion to repeat.

`REVIEW_PACKET.zip` is a lightweight human-review packet. It does not contain the
frozen executable or source snapshot and cannot independently establish source/binary
forensic identity; use the candidate's sibling `AUDIT_BUNDLE.zip` for that purpose.

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


def parse_hash_manifest(path: Path) -> dict[str, str]:
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise ExperimentError(f"cannot read HASHES.sha256: {error}") from error
    if not text or not text.endswith("\n"):
        raise ExperimentError("HASHES.sha256 must be non-empty and newline-terminated")

    entries: dict[str, str] = {}
    previous_path: str | None = None
    for line_number, line in enumerate(text.splitlines(), 1):
        match = re.fullmatch(r"([0-9a-f]{64})  ([^\r\n]+)", line)
        if match is None:
            raise ExperimentError(f"HASHES.sha256 line {line_number} is malformed")
        digest, relative_text = match.groups()
        relative = PurePosixPath(relative_text)
        if (
            relative.is_absolute()
            or ".." in relative.parts
            or "\\" in relative_text
            or relative.as_posix() != relative_text
        ):
            raise ExperimentError(
                f"HASHES.sha256 line {line_number} has an unsafe/noncanonical path"
            )
        if relative_text in entries:
            raise ExperimentError(
                f"HASHES.sha256 contains duplicate path: {relative_text}"
            )
        if previous_path is not None and relative_text <= previous_path:
            raise ExperimentError("HASHES.sha256 paths must be strictly sorted")
        entries[relative_text] = digest
        previous_path = relative_text
    return entries


def validate_hash_inventory(run_dir: Path, hashes_path: Path) -> dict[str, str]:
    entries = parse_hash_manifest(hashes_path)
    expected_files = hashable_files(run_dir)
    expected = {
        path.relative_to(run_dir).as_posix(): path for path in expected_files
    }
    actual_paths = set(entries)
    expected_paths = set(expected)
    if actual_paths != expected_paths:
        missing = sorted(expected_paths - actual_paths)
        extra = sorted(actual_paths - expected_paths)
        raise ExperimentError(
            f"HASHES.sha256 inventory mismatch; missing={missing}, extra={extra}"
        )
    for relative, path in expected.items():
        if not path.is_file():
            raise ExperimentError(f"hash inventory path is not a file: {relative}")
        observed = sha256_file(path)
        if entries[relative] != observed:
            raise ExperimentError(
                f"HASHES.sha256 digest mismatch for {relative}: "
                f"recorded={entries[relative]}, observed={observed}"
            )
    return entries


def postprocess_run(
    run_dir: Path,
    publication_log: list[str] | None = None,
    final_guard: Callable[[], None] | None = None,
) -> Path:
    log = publication_log if publication_log is not None else []
    receipt_path = run_dir / "EXPERIMENT_RECEIPT.json"
    if receipt_path.exists():
        raise ExperimentError("completed run already has a receipt and cannot be reused")
    manifest_path = run_dir / "EXPERIMENT_MANIFEST.toml"
    manifest = read_and_validate_manifest(manifest_path)
    contract = contract_for_manifest(manifest)
    analysis, frames_doc, samples, events = validate_telemetry(run_dir, manifest)
    if contract is PRESSURE_CONTRACT and manifest["run_mode"] == "candidate":
        blocker = pressure_candidate_blocker(analysis)
        if blocker["candidate_blocker"]:
            classification = blocker["candidate_blocker_classification"]
            failed = ",".join(blocker["failed_hard_predicates"]) or "none"
            raise ExperimentError(
                "Pressure candidate publication blocked after telemetry validation: "
                f"classification={classification}; failed_hard_predicates={failed}; "
                "run preserved incomplete without report, receipt, or Audit Bundle"
            )
    screenshots = create_screenshots(run_dir, frames_doc["frames"], log)
    if contract.records_run_mode:
        for screenshot, frame in zip(screenshots, frames_doc["frames"], strict=True):
            if contract is PRESSURE_CONTRACT:
                screenshot["kind"] = "+".join(
                    badge["kind"] for badge in frame["badges"]
                )
            else:
                screenshot["kind"] = frame["kind"]

    report_dir = run_dir / "report"
    try:
        report_dir.mkdir()
    except FileExistsError as error:
        raise ExperimentError("report output directory already exists") from error
    contact_sheet_path = report_dir / "CONTACT_SHEET.png"
    write_new_bytes(
        contact_sheet_path,
        create_contact_sheet_bytes(run_dir, screenshots, samples),
        log,
    )

    report_json = {
        "schema_version": contract.report_schema,
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
            "sand_fall_only": contract is SAND_CONTRACT,
            "water_flow": contract is WATER_CONTRACT,
            "g8c": False,
            "ai_contacted": False,
            "review_packet_role": "lightweight_human_review",
            "review_packet_supports_source_binary_forensics": False,
            "candidate_forensics_delivery": "sibling AUDIT_BUNDLE",
        },
    }
    if contract is FIRE_CONTRACT:
        report_json["scope"]["fire_heat"] = True
    elif contract is PRESSURE_CONTRACT:
        report_json["scope"]["pressure_burst"] = True
    if contract.records_run_mode:
        report_json["run_mode"] = manifest["run_mode"]
    if contract is WATER_CONTRACT:
        report_json["water_remediation"] = water_remediation_summary(analysis)
        report_json["review_guidance"] = {
            "classification_categories": list(WATER_FINDING_CLASSIFICATIONS),
            "visual_questions": list(WATER_VISUAL_QUESTIONS),
            "categories_are_findings": False,
            "active_cell_classification_rule": WATER_ACTIVE_CLASSIFICATION_RULE,
            "active_cell_classes_are_worker_aggregates": True,
        }
    elif contract is FIRE_CONTRACT:
        report_json["fire_heat"] = fire_heat_summary(analysis)
        report_json["review_guidance"] = {
            "thermal_tail_is_not_failure_by_itself": True,
            "automatic_verdict_is_user_acceptance": False,
            "g8b_closed": False,
        }
    elif contract is PRESSURE_CONTRACT:
        report_json["pressure_burst"] = pressure_burst_summary(analysis)
        report_json["review_guidance"] = {
            "automatic_verdict_is_user_acceptance": False,
            "review_flags_force_human_review": True,
            "fixture_causality_confounded_is_candidate_blocker": True,
            "named_causal_chain_required": True,
            "g8b_closed": False,
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
        "schema_version": contract.receipt_schema,
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
    source_input_manifest_path = run_dir / SOURCE_INPUT_MANIFEST_NAME
    if source_input_manifest_path.is_file():
        receipt["source_input_manifest_sha256"] = sha256_file(
            source_input_manifest_path
        )
    binary_path = Path(manifest["binary"]["path"])
    if is_path_within(binary_path, run_dir):
        receipt["frozen_binary_path"] = binary_path.relative_to(run_dir).as_posix()
    if contract.records_run_mode:
        receipt["run_mode"] = manifest["run_mode"]
    if contract is WATER_CONTRACT:
        receipt["water_remediation"] = water_remediation_summary(analysis)
    elif contract is FIRE_CONTRACT:
        receipt["fire_heat"] = fire_heat_summary(analysis)
    elif contract is PRESSURE_CONTRACT:
        receipt["pressure_burst"] = pressure_burst_summary(analysis)
    if final_guard is not None:
        final_guard()
    write_new_text(
        receipt_path,
        json.dumps(receipt, indent=2, ensure_ascii=False, sort_keys=True) + "\n",
        log,
    )
    # Publication invariant: this function performs no filesystem write after
    # the create-new receipt write above.
    return receipt_path


def git_archive_zip_bytes(source_root: Path, source_sha: str) -> bytes:
    if not GIT_OID.fullmatch(source_sha):
        raise ExperimentError(f"cannot archive invalid source SHA: {source_sha!r}")
    safe_root = str(source_root.resolve())
    completed = subprocess.run(
        [
            "git",
            "-c",
            f"safe.directory={safe_root}",
            "archive",
            "--format=zip",
            "--prefix=source/",
            source_sha,
        ],
        cwd=source_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if completed.returncode != 0:
        detail = completed.stderr.decode("utf-8", errors="replace").strip()
        raise ExperimentError(f"git archive failed: {detail}")
    return completed.stdout


def exact_json_value_equal(recorded: Any, expected: Any) -> bool:
    if type(recorded) is not type(expected):
        return False
    if isinstance(expected, dict):
        return set(recorded) == set(expected) and all(
            exact_json_value_equal(recorded[key], expected[key]) for key in expected
        )
    if isinstance(expected, list):
        return len(recorded) == len(expected) and all(
            exact_json_value_equal(left, right)
            for left, right in zip(recorded, expected, strict=True)
        )
    return recorded == expected


def validate_audit_receipt(
    receipt: dict[str, Any],
    manifest: dict[str, Any],
    analysis: dict[str, Any],
    *,
    manifest_path: Path,
    hashes_path: Path,
    review_packet_path: Path,
    source_inputs_path: Path,
    binary_path: Path,
    hash_entry_count: int,
) -> None:
    contract = contract_for_manifest(manifest)
    expected: dict[str, Any] = {
        "schema_version": contract.receipt_schema,
        "experiment_id": manifest["experiment_id"],
        "run_id": manifest["run_id"],
        "scenario": manifest["scenario"],
        "source_sha": manifest["source"]["sha"],
        "binary_sha256": manifest["binary"]["sha256"],
        "automatic_verdict": analysis["verdict"],
        "manifest_sha256": sha256_file(manifest_path),
        "review_packet_sha256": sha256_file(review_packet_path),
        "hashes_sha256": sha256_file(hashes_path),
        "hash_entry_count": hash_entry_count,
        "receipt_is_final_publication_marker": True,
        "source_input_manifest_sha256": sha256_file(source_inputs_path),
        "frozen_binary_path": binary_path.relative_to(manifest_path.parent).as_posix(),
    }
    if contract.records_run_mode:
        expected["run_mode"] = manifest["run_mode"]
    if contract is WATER_CONTRACT:
        expected["water_remediation"] = water_remediation_summary(analysis)
    elif contract is FIRE_CONTRACT:
        expected["fire_heat"] = fire_heat_summary(analysis)
    elif contract is PRESSURE_CONTRACT:
        expected["pressure_burst"] = pressure_burst_summary(analysis)

    require_exact_keys(
        receipt, set(expected) | {"completed_utc"}, "experiment receipt"
    )
    completed_utc = receipt["completed_utc"]
    if not isinstance(completed_utc, str):
        raise ExperimentError("receipt completed_utc must be a string")
    try:
        completed = datetime.fromisoformat(completed_utc.replace("Z", "+00:00"))
    except ValueError as error:
        raise ExperimentError("receipt completed_utc must be an ISO-8601 timestamp") from error
    if completed.tzinfo is None or completed.utcoffset() != timezone.utc.utcoffset(completed):
        raise ExperimentError("receipt completed_utc must identify UTC")

    for field, expected_value in expected.items():
        recorded = receipt[field]
        if not exact_json_value_equal(recorded, expected_value):
            raise ExperimentError(
                f"receipt {field} contract mismatch: "
                f"recorded={recorded!r}, observed={expected_value!r}"
            )


def safe_bundle_member_name(value: str, label: str) -> str:
    relative = PurePosixPath(value)
    if (
        not value
        or relative.is_absolute()
        or ".." in relative.parts
        or "\\" in value
        or relative.as_posix() != value
    ):
        raise ExperimentError(f"{label} has an unsafe/noncanonical path: {value!r}")
    return value


def deterministic_zip_bytes(members: Iterable[tuple[str, bytes]]) -> bytes:
    ordered = sorted(members, key=lambda member: member[0])
    names = [safe_bundle_member_name(name, "ZIP member") for name, _ in ordered]
    if len(names) != len(set(names)):
        raise ExperimentError("ZIP member inventory contains duplicate paths")
    output = io.BytesIO()
    with zipfile.ZipFile(
        output, mode="w", compression=zipfile.ZIP_DEFLATED, compresslevel=9
    ) as archive:
        for name, data in ordered:
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            archive.writestr(info, data, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    return output.getvalue()


def zip_bytes_inventory(data: bytes, label: str) -> list[dict[str, Any]]:
    inventory: list[dict[str, Any]] = []
    try:
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            names: set[str] = set()
            for info in archive.infolist():
                if info.is_dir():
                    continue
                name = safe_bundle_member_name(info.filename, f"{label} member")
                if name in names:
                    raise ExperimentError(f"{label} contains duplicate member {name}")
                names.add(name)
                payload = archive.read(info)
                if len(payload) != info.file_size:
                    raise ExperimentError(f"{label} member size mismatch for {name}")
                inventory.append(
                    {
                        "path": name,
                        "size_bytes": len(payload),
                        "sha256": hashlib.sha256(payload).hexdigest(),
                    }
                )
    except (OSError, zipfile.BadZipFile, RuntimeError) as error:
        raise ExperimentError(f"invalid {label}: {error}") from error
    inventory.sort(key=lambda entry: entry["path"])
    return inventory


def pressure_source_input_bytes(
    source_inputs_path: Path,
    source_root: Path,
    manifest: dict[str, Any],
) -> tuple[bytes, list[dict[str, str]]]:
    source_inputs = read_json(source_inputs_path, "source input manifest")
    require_exact_keys(
        source_inputs,
        {
            "schema_version",
            "source",
            "selection",
            "file_count",
            "files",
            "external_file_count",
            "external_files",
        },
        "source input manifest",
    )
    if source_inputs["schema_version"] != SOURCE_INPUT_MANIFEST_SCHEMA:
        raise ExperimentError("source input manifest schema mismatch")
    if source_inputs["source"] != {
        "root": str(source_root.resolve()),
        "branch": manifest["source"]["branch"],
        "head_sha": manifest["source"]["sha"],
        "git_state": manifest["source"]["git_state"],
    }:
        raise ExperimentError("source input manifest source identity mismatch")
    files = source_inputs["files"]
    external_files = source_inputs["external_files"]
    if not isinstance(files, list) or source_inputs["file_count"] != len(files):
        raise ExperimentError("source input manifest tracked file_count mismatch")
    if (
        not isinstance(external_files, list)
        or source_inputs["external_file_count"] != len(external_files)
    ):
        raise ExperimentError("source input manifest external_file_count mismatch")

    zip_members: list[tuple[str, bytes]] = []
    mappings: list[dict[str, str]] = []
    seen_originals: set[str] = set()
    for index, entry in enumerate(files):
        if not isinstance(entry, dict):
            raise ExperimentError(f"source input manifest file {index} must be an object")
        require_exact_keys(
            entry, {"path", "sha256", "size_bytes"}, f"source input file {index}"
        )
        relative_text = safe_bundle_member_name(entry["path"], "source input path")
        if relative_text in seen_originals:
            raise ExperimentError(f"source input manifest duplicates {relative_text}")
        seen_originals.add(relative_text)
        relative = PurePosixPath(relative_text)
        path = source_root.joinpath(*relative.parts)
        if not path.is_file():
            raise ExperimentError(f"sealed source input is missing: {relative_text}")
        payload = path.read_bytes()
        if entry["size_bytes"] != len(payload) or entry["sha256"] != hashlib.sha256(
            payload
        ).hexdigest():
            raise ExperimentError(f"sealed source input bytes drifted: {relative_text}")
        bundle_path = f"repository/{relative_text}"
        zip_members.append((bundle_path, payload))
        mappings.append(
            {
                "original": relative_text,
                "bundle_path": f"SOURCE_INPUT_BYTES.zip!{bundle_path}",
            }
        )

    labels: set[str] = set()
    for index, entry in enumerate(external_files):
        if not isinstance(entry, dict):
            raise ExperimentError(f"external source input {index} must be an object")
        require_exact_keys(
            entry,
            {"label", "path", "sha256", "size_bytes"},
            f"external source input {index}",
        )
        label = entry["label"]
        if (
            not isinstance(label, str)
            or not re.fullmatch(r"[a-z0-9][a-z0-9._-]*", label)
            or label in labels
        ):
            raise ExperimentError(f"external source input label is invalid/duplicate: {label!r}")
        labels.add(label)
        path = Path(entry["path"])
        if not path.is_absolute() or not path.is_file():
            raise ExperimentError(f"external source input is missing: {entry['path']!r}")
        payload = path.read_bytes()
        if entry["size_bytes"] != len(payload) or entry["sha256"] != hashlib.sha256(
            payload
        ).hexdigest():
            raise ExperimentError(f"external source input bytes drifted: {label}")
        filename = path.name
        if not filename or filename in {".", ".."}:
            raise ExperimentError(f"external source input filename is invalid: {label}")
        bundle_path = f"external/{label}/{filename}"
        zip_members.append((bundle_path, payload))
        mappings.append(
            {
                "original": str(path.resolve()),
                "bundle_path": f"SOURCE_INPUT_BYTES.zip!{bundle_path}",
            }
        )
    mappings.sort(key=lambda entry: (entry["original"], entry["bundle_path"]))
    return deterministic_zip_bytes(zip_members), mappings


def render_bundle_hashes(members: dict[str, bytes]) -> bytes:
    return "".join(
        f"{hashlib.sha256(data).hexdigest()}  {name}\n"
        for name, data in sorted(members.items())
        if name != "AUDIT_BUNDLE_HASHES.sha256"
    ).encode("utf-8")


def create_pressure_audit_bundle_vnext(
    run_dir: Path, source_root: Path, expected_receipt_sha256: str
) -> tuple[Path, Path]:
    bundle_path = run_dir.parent / f"{run_dir.name}{AUDIT_BUNDLE_SUFFIX}"
    sidecar_path = run_dir.parent / f"{run_dir.name}{AUDIT_BUNDLE_SHA256_SUFFIX}"
    if bundle_path.exists() or sidecar_path.exists():
        raise ExperimentError(
            "refusing to overwrite existing Pressure audit bundle or SHA-256 sidecar"
        )

    manifest_path = run_dir / "EXPERIMENT_MANIFEST.toml"
    hashes_path = run_dir / "HASHES.sha256"
    receipt_path = run_dir / "EXPERIMENT_RECEIPT.json"
    source_inputs_path = run_dir / SOURCE_INPUT_MANIFEST_NAME
    review_packet_path = run_dir / "report" / "REVIEW_PACKET.zip"
    required = (
        manifest_path,
        hashes_path,
        receipt_path,
        source_inputs_path,
        review_packet_path,
    )
    missing = [path.relative_to(run_dir).as_posix() for path in required if not path.is_file()]
    if missing:
        raise ExperimentError(f"Pressure audit bundle inputs are incomplete: {missing}")
    if not isinstance(expected_receipt_sha256, str) or not HEX64.fullmatch(
        expected_receipt_sha256
    ):
        raise ExperimentError("expected receipt SHA-256 must be 64 hexadecimal characters")
    if sha256_file(receipt_path) != expected_receipt_sha256.lower():
        raise ExperimentError("Pressure receipt bytes changed after final publication")

    manifest = read_and_validate_manifest(manifest_path)
    if contract_for_manifest(manifest) is not PRESSURE_CONTRACT:
        raise ExperimentError("Pressure Audit Bundle vNext requires Pressure Burst")
    if manifest["run_mode"] != "candidate":
        raise ExperimentError("Pressure Audit Bundle vNext is candidate-only")
    binary_path = Path(manifest["binary"]["path"])
    expected_binary = run_dir.joinpath(*FROZEN_BINARY_RELATIVE_PATH.parts).resolve()
    if binary_path.resolve() != expected_binary or not binary_path.is_file():
        raise ExperimentError("Pressure audit bundle requires the run-local frozen executable")
    if sha256_file(binary_path) != manifest["binary"]["sha256"]:
        raise ExperimentError("frozen executable hash changed before Pressure audit bundling")

    analysis, _, _, _ = validate_telemetry(run_dir, manifest)
    hash_entries = validate_hash_inventory(run_dir, hashes_path)
    receipt = read_json(receipt_path, "experiment receipt")
    validate_audit_receipt(
        receipt,
        manifest,
        analysis,
        manifest_path=manifest_path,
        hashes_path=hashes_path,
        review_packet_path=review_packet_path,
        source_inputs_path=source_inputs_path,
        binary_path=binary_path,
        hash_entry_count=len(hash_entries),
    )

    source_input_zip, source_mappings = pressure_source_input_bytes(
        source_inputs_path, source_root.resolve(), manifest
    )
    # Unlike the historical bundles, Git archive failure is fatal for Pressure.
    git_archive = git_archive_zip_bytes(source_root.resolve(), manifest["source"]["sha"])
    review_packet = review_packet_path.read_bytes()
    nested_packet_inventory = zip_bytes_inventory(review_packet, "REVIEW_PACKET.zip")
    run_members: list[tuple[Path, str, str]] = [
        (review_packet_path, "REVIEW_PACKET.zip", "lightweight human review packet"),
        (manifest_path, "EXPERIMENT_MANIFEST.toml", "run contract and exact commands"),
        (hashes_path, "HASHES.sha256", "complete immutable run-file hash inventory"),
        (receipt_path, "EXPERIMENT_RECEIPT.json", "final run publication marker"),
        (
            source_inputs_path,
            SOURCE_INPUT_MANIFEST_NAME,
            "sealed build-input inventory",
        ),
        (
            binary_path,
            FROZEN_BINARY_RELATIVE_PATH.as_posix(),
            "run-local executable used by the worker",
        ),
    ]
    direct_members = [
        {
            "bundle_path": "AUDIT_BUNDLE_MANIFEST.json",
            "original": None,
            "role": "self-describing bundle inventory and verification scopes",
        },
        {
            "bundle_path": "AUDIT_BUNDLE_HASHES.sha256",
            "original": None,
            "role": "SHA-256 inventory of every other direct bundle member",
        },
        {
            "bundle_path": "SOURCE_INPUT_BYTES.zip",
            "original": SOURCE_INPUT_MANIFEST_NAME,
            "role": "byte-exact tracked and external sealed build inputs",
        },
        {
            "bundle_path": "GIT_SOURCE_ARCHIVE.zip",
            "original": manifest["source"]["sha"],
            "role": "commit-addressable full tracked source tree",
        },
    ]
    direct_members.extend(
        {
            "bundle_path": bundle_name,
            "original": path.relative_to(run_dir).as_posix(),
            "role": role,
        }
        for path, bundle_name, role in run_members
    )
    direct_members.sort(key=lambda entry: entry["bundle_path"])
    original_to_bundle = [
        {
            "original": path.relative_to(run_dir).as_posix(),
            "bundle_path": bundle_name,
        }
        for path, bundle_name, _ in run_members
    ]
    original_to_bundle.extend(source_mappings)
    original_to_bundle.extend(
        {
            "original": entry["path"],
            "bundle_path": f"REVIEW_PACKET.zip!{entry['path']}",
        }
        for entry in nested_packet_inventory
    )
    original_to_bundle.append(
        {
            "original": f"git:{manifest['source']['sha']}",
            "bundle_path": "GIT_SOURCE_ARCHIVE.zip!source/**",
        }
    )
    original_to_bundle.sort(key=lambda entry: (entry["original"], entry["bundle_path"]))
    bundle_manifest = {
        "schema_version": PRESSURE_AUDIT_BUNDLE_MANIFEST_SCHEMA,
        "experiment_id": manifest["experiment_id"],
        "run_id": manifest["run_id"],
        "scenario": manifest["scenario"],
        "source_sha": manifest["source"]["sha"],
        "binary_sha256": manifest["binary"]["sha256"],
        "receipt_sha256": expected_receipt_sha256.lower(),
        "direct_members": direct_members,
        "nested_review_packet_inventory": nested_packet_inventory,
        "original_to_bundle_mapping": original_to_bundle,
        "omitted_work": [
            {
                "path": "work/analysis.json",
                "reason": "worker intermediate is hash-bound and copied into report/REPORT.json",
            },
            {
                "path": "work/frames.json",
                "reason": "worker frame metadata is hash-bound and independently validated before packaging",
            },
            {
                "path": "work/frames/**",
                "reason": "raw RGBA intermediates are hash-bound; derived full PNGs are in REVIEW_PACKET.zip",
            },
        ],
        "verification_scopes": {
            "REVIEW_PACKET.zip": "human-visible report, telemetry, logs, and derived screenshots",
            "SOURCE_INPUT_BYTES.zip": "exact bytes enumerated by SOURCE_INPUT_MANIFEST.json, including external inputs",
            "GIT_SOURCE_ARCHIVE.zip": "tracked repository tree at source SHA only",
            FROZEN_BINARY_RELATIVE_PATH.as_posix(): "exact executable launched for this run",
            "HASHES.sha256": "run-directory files before the final receipt, excluding only HASHES and receipt",
            "AUDIT_BUNDLE_HASHES.sha256": "every other direct Audit Bundle member, excluding only this bundle-local hash inventory itself",
        },
        "archive_role_difference": {
            "SOURCE_INPUT_BYTES.zip": (
                "selected build inputs at capture time, including external font bytes"
            ),
            "GIT_SOURCE_ARCHIVE.zip": (
                "complete Git-tracked tree at source SHA; excludes external inputs"
            ),
        },
    }
    manifest_bytes = (
        json.dumps(bundle_manifest, indent=2, ensure_ascii=False, sort_keys=True) + "\n"
    ).encode("utf-8")
    members: dict[str, bytes] = {
        "AUDIT_BUNDLE_MANIFEST.json": manifest_bytes,
        "SOURCE_INPUT_BYTES.zip": source_input_zip,
        "GIT_SOURCE_ARCHIVE.zip": git_archive,
    }
    for path, bundle_name, _ in run_members:
        members[bundle_name] = path.read_bytes()
    members["AUDIT_BUNDLE_HASHES.sha256"] = render_bundle_hashes(members)
    expected_direct_paths = {entry["bundle_path"] for entry in direct_members}
    if set(members) != expected_direct_paths:
        raise ExperimentError("Pressure audit bundle direct-member inventory mismatch")
    bundle_bytes = deterministic_zip_bytes(members.items())
    write_new_bytes(bundle_path, bundle_bytes)
    bundle_hash = sha256_file(bundle_path)
    write_new_text(sidecar_path, f"{bundle_hash}  {bundle_path.name}\n")
    return bundle_path, sidecar_path


def create_audit_bundle(
    run_dir: Path, source_root: Path, expected_receipt_sha256: str
) -> tuple[Path, Path]:
    manifest_path = run_dir / "EXPERIMENT_MANIFEST.toml"
    hashes_path = run_dir / "HASHES.sha256"
    receipt_path = run_dir / "EXPERIMENT_RECEIPT.json"
    source_inputs_path = run_dir / SOURCE_INPUT_MANIFEST_NAME
    review_packet_path = run_dir / "report" / "REVIEW_PACKET.zip"
    required = (
        manifest_path,
        hashes_path,
        receipt_path,
        source_inputs_path,
        review_packet_path,
    )
    missing = [path.name for path in required if not path.is_file()]
    if missing:
        raise ExperimentError(f"audit bundle inputs are incomplete: {missing}")
    if not HEX64.fullmatch(expected_receipt_sha256):
        raise ExperimentError("expected receipt SHA-256 must be 64 hexadecimal characters")
    observed_receipt_sha256 = sha256_file(receipt_path)
    if observed_receipt_sha256 != expected_receipt_sha256.lower():
        raise ExperimentError(
            "receipt SHA-256 changed after final publication: "
            f"expected={expected_receipt_sha256.lower()}, "
            f"observed={observed_receipt_sha256}"
        )

    manifest = read_and_validate_manifest(manifest_path)
    run_mode = manifest.get("run_mode", "candidate")
    if run_mode != "candidate":
        raise ExperimentError("AUDIT_BUNDLE is candidate-only")
    binary_path = Path(manifest["binary"]["path"])
    expected_binary = run_dir.joinpath(*FROZEN_BINARY_RELATIVE_PATH.parts).resolve()
    if binary_path.resolve() != expected_binary or not binary_path.is_file():
        raise ExperimentError("audit bundle requires the run-local frozen executable")
    if sha256_file(binary_path) != manifest["binary"]["sha256"]:
        raise ExperimentError("frozen executable hash changed before audit bundling")

    analysis, _, _, _ = validate_telemetry(run_dir, manifest)
    hash_entries = validate_hash_inventory(run_dir, hashes_path)
    receipt = read_json(receipt_path, "experiment receipt")
    validate_audit_receipt(
        receipt,
        manifest,
        analysis,
        manifest_path=manifest_path,
        hashes_path=hashes_path,
        review_packet_path=review_packet_path,
        source_inputs_path=source_inputs_path,
        binary_path=binary_path,
        hash_entry_count=len(hash_entries),
    )

    bundle_path = run_dir.parent / f"{run_dir.name}{AUDIT_BUNDLE_SUFFIX}"
    sidecar_path = run_dir.parent / f"{run_dir.name}{AUDIT_BUNDLE_SHA256_SUFFIX}"
    members = (
        (review_packet_path, "REVIEW_PACKET.zip"),
        (manifest_path, "EXPERIMENT_MANIFEST.toml"),
        (hashes_path, "HASHES.sha256"),
        (receipt_path, "EXPERIMENT_RECEIPT.json"),
        (source_inputs_path, SOURCE_INPUT_MANIFEST_NAME),
        (binary_path, FROZEN_BINARY_RELATIVE_PATH.as_posix()),
    )
    try:
        with bundle_path.open("xb") as output:
            with zipfile.ZipFile(
                output, mode="w", compression=zipfile.ZIP_DEFLATED, compresslevel=9
            ) as archive:
                for path, archive_name in members:
                    archive.write(path, archive_name)
                try:
                    source_archive = git_archive_zip_bytes(
                        source_root, manifest["source"]["sha"]
                    )
                except ExperimentError as error:
                    archive.writestr(
                        "SOURCE_ARCHIVE_UNAVAILABLE.txt",
                        f"Git archive was unavailable: {error}\n",
                    )
                else:
                    archive.writestr("SOURCE_ARCHIVE.zip", source_archive)
            output.flush()
            os.fsync(output.fileno())
    except FileExistsError as error:
        raise ExperimentError(f"refusing to overwrite audit bundle: {bundle_path}") from error
    except OSError as error:
        raise ExperimentError(f"failed to create audit bundle: {error}") from error

    bundle_hash = sha256_file(bundle_path)
    write_new_text(sidecar_path, f"{bundle_hash}  {bundle_path.name}\n")
    return bundle_path, sidecar_path


def worker_command(
    binary: Path,
    run_dir: Path,
    run_id: str,
    binary_sha256: str,
    contract: ScenarioContract = SAND_CONTRACT,
    run_mode: str = "candidate",
) -> tuple[str, ...]:
    validate_run_mode(contract, run_mode)
    common = (
        str(binary),
        "--experiment-worker",
        contract.scenario,
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
    )
    if contract is FIRE_CONTRACT:
        return common + (
            "--consecutive-reaction-zero",
            str(CONSECUTIVE_REACTION_ZERO),
            "--post-reaction-ticks",
            str(POST_REACTION_TICKS),
        )
    if contract is PRESSURE_CONTRACT:
        return common + (
            "--consecutive-persistent-opening",
            str(CONSECUTIVE_PERSISTENT_OPENING),
            "--post-opening-ticks",
            str(POST_OPENING_TICKS),
            "--terminal-window-samples",
            str(TERMINAL_WINDOW_SAMPLES),
        )
    return common + (
        "--consecutive-all-sleep",
        str(CONSECUTIVE_ALL_SLEEP),
        "--post-sleep-ticks",
        str(POST_SLEEP_TICKS),
    )


def run_experiment(
    source_root: Path,
    artifact_root: Path,
    scenario: str,
    mode: str = "candidate",
) -> Path:
    contract = contract_for_scenario(scenario)
    validate_run_mode(contract, mode)
    validate_external_artifact_root(source_root, artifact_root)
    source_seal = capture_source_seal(
        source_root,
        allow_dirty_tracked=contract.records_run_mode and mode == "scratch",
    )
    source = source_seal.source
    run_id = generate_run_id(contract=contract, run_mode=mode)
    run_dir = create_run_directory(artifact_root.resolve(), run_id)
    source_manifest_path = run_dir / SOURCE_INPUT_MANIFEST_NAME
    write_new_text(source_manifest_path, render_source_input_manifest(source_seal))
    logs = run_dir / "logs"
    logs.mkdir()

    build = ("cargo", "build", "--locked", "--release", "-p", "powdergame-windows")
    build_exit = run_logged(
        build,
        source.root,
        logs / "build.stdout.log",
        logs / "build.stderr.log",
    )
    assert_source_seal_unchanged(source.root, source_seal, "post-build")
    assert_source_manifest_artifact_unchanged(source_manifest_path, source_seal)
    if build_exit != 0:
        raise ExperimentError(f"release build failed with exit code {build_exit}; run preserved")

    release_binary = source.root / "target" / "release" / "powdergame-windows.exe"
    if not release_binary.is_file():
        raise ExperimentError(f"release binary was not produced: {release_binary}")
    binary, binary_hash = copy_frozen_binary(release_binary, run_dir)
    assert_source_seal_unchanged(source.root, source_seal, "pre-worker")
    assert_source_manifest_artifact_unchanged(source_manifest_path, source_seal)
    assert_frozen_binary_unchanged(binary, binary_hash, "pre-worker")
    worker = worker_command(
        binary, run_dir, run_id, binary_hash, contract=contract, run_mode=mode
    )
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
        contract=contract,
        run_mode=mode,
    )
    manifest_text = render_manifest(manifest)
    write_new_text(run_dir / "EXPERIMENT_MANIFEST.toml", manifest_text)
    read_and_validate_manifest(run_dir / "EXPERIMENT_MANIFEST.toml")
    assert_source_seal_unchanged(source.root, source_seal, "worker-launch")
    assert_source_manifest_artifact_unchanged(source_manifest_path, source_seal)
    assert_frozen_binary_unchanged(binary, binary_hash, "worker-launch")

    worker_exit = run_logged(
        worker,
        source.root,
        run_dir / "stdout.log",
        run_dir / "stderr.log",
    )
    assert_source_seal_unchanged(source.root, source_seal, "post-worker")
    assert_source_manifest_artifact_unchanged(source_manifest_path, source_seal)
    assert_frozen_binary_unchanged(binary, binary_hash, "post-worker")
    if worker_exit != 0:
        raise ExperimentError(
            f"experiment worker failed operationally with exit code {worker_exit}; "
            "run preserved without receipt"
        )

    def final_publication_guard() -> None:
        assert_source_seal_unchanged(source.root, source_seal, "pre-receipt")
        assert_source_manifest_artifact_unchanged(source_manifest_path, source_seal)
        assert_frozen_binary_unchanged(binary, binary_hash, "pre-receipt")

    receipt = postprocess_run(run_dir, final_guard=final_publication_guard)
    if mode == "candidate":
        receipt_sha256 = sha256_file(receipt)
        if contract is PRESSURE_CONTRACT:
            create_pressure_audit_bundle_vnext(
                run_dir, source.root, receipt_sha256
            )
        else:
            create_audit_bundle(run_dir, source.root, receipt_sha256)
    return receipt


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run one immutable Powdergame G8-B scenario experiment."
    )
    parser.add_argument(
        "scenario", choices=sorted(SCENARIO_CONTRACTS), help="scenario experiment to run"
    )
    parser.add_argument(
        "--mode",
        choices=sorted(RUN_MODES),
        default="candidate",
        help="scenario publication mode (default: candidate)",
    )
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
        receipt = run_experiment(
            args.source_root, args.artifact_root, args.scenario, mode=args.mode
        )
    except ExperimentError as error:
        print(f"FATAL: {error}", file=sys.stderr)
        return 1
    print(f"Experiment receipt: {receipt}")
    if args.mode == "candidate":
        run_dir = receipt.parent
        bundle = run_dir.parent / f"{run_dir.name}{AUDIT_BUNDLE_SUFFIX}"
        sidecar = run_dir.parent / f"{run_dir.name}{AUDIT_BUNDLE_SHA256_SUFFIX}"
        print(f"Audit bundle: {bundle}")
        print(f"Audit bundle SHA-256: {sha256_file(bundle)} ({sidecar})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
