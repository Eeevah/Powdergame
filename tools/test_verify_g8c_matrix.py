#!/usr/bin/env python3
"""Focused integrity tests for the independent G8-C matrix verifier."""

from __future__ import annotations

import csv
import contextlib
import hashlib
import io
import json
import shutil
import subprocess
import tempfile
import unittest
import warnings
import zipfile
from pathlib import Path
from unittest import mock

try:
    from tools import g8c_matrix as coordinator
    from tools import verify_g8c_matrix as verifier
except ModuleNotFoundError:  # Direct `python tools/test_verify_g8c_matrix.py`.
    import g8c_matrix as coordinator
    import verify_g8c_matrix as verifier


def write_csv(
    path: Path, header: tuple[str, ...], rows: list[dict[str, object]]
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=header)
        writer.writeheader()
        writer.writerows(rows)


def tiny_config(
    *, mode_c_frames: int = 3, mode_d_frames: int = 2
) -> verifier.CommonConfig:
    return verifier.CommonConfig(
        width=2,
        height=2,
        chunk_size=2,
        sleep_enabled=True,
        sleep_threshold=16,
        prewarm_seconds=2.0,
        trials=1,
        mode_a_ticks=1,
        mode_b_ticks=1,
        overhead_ticks=1,
        target_tps=60.0,
        mode_c_seconds=None,
        mode_c_frames=mode_c_frames,
        mode_d_frames=mode_d_frames,
        render_width=1600,
        render_height=900,
    )


def valid_window_lifecycle() -> dict[str, object]:
    return {
        "required_width": 1600,
        "required_height": 900,
        "initial_live_width": 1600,
        "initial_live_height": 900,
        "last_live_width": 1600,
        "last_live_height": 900,
        "initial_live_size_confirmed": True,
        "canonical_noop_count": 1,
        "stale_payload_count": 1,
        "fatal_live_resize_count": 0,
        "event_count": 2,
        "events": [
            {
                "event_kind": "resized",
                "classification": "canonical_no_op",
                "payload_width": 1600,
                "payload_height": 900,
                "live_width": 1600,
                "live_height": 900,
            },
            {
                "event_kind": "scale_factor_changed",
                "classification": "stale_payload_ignored",
                "payload_width": 2864,
                "payload_height": 1560,
                "live_width": 1600,
                "live_height": 900,
            },
        ],
    }


def headless_common(
    *,
    scenario: str,
    source_sha: str,
    run_id: str,
    mode: str,
    profiling: bool,
) -> dict[str, object]:
    row: dict[str, object] = {name: "" for name in verifier.SUMMARY_HEADER}
    row.update(
        {
            "schema_version": verifier.INNER_HEADLESS_SCHEMA,
            "run_id": run_id,
            "commit_sha": source_sha,
            "git_state": "clean",
            "adapter_name": "NVIDIA GeForce RTX 5090",
            "vendor_id": "0x10DE",
            "device_id": "0x2B85",
            "device_type": "DiscreteGpu",
            "backend": "Dx12",
            "driver": "test-driver",
            "driver_info": "test-info",
            "profiling_enabled": "true" if profiling else "false",
            "timestamp_period_ns": "1000" if profiling else "",
            "build_profile": "release",
            "width": "2",
            "height": "2",
            "chunk_size": "2",
            "sleep_enabled": "true",
            "sleep_threshold": "16",
            "prewarm_requested_secs": "2",
            "prewarm_ticks": "1",
            "measurement_mode": mode,
            "method_note": f"fixture; scenario={scenario}",
        }
    )
    return row


def stats_summary_row(
    common: dict[str, object], metric_type: str, name: str, value: float
) -> dict[str, object]:
    row = dict(common)
    row.update(
        {
            "selection": "trial",
            "trial": "1",
            "tick_start": "0",
            "tick_end": "0",
            "metric_type": metric_type,
            "name": name,
            "value": "",
            "count": "1",
            "p50": str(value),
            "p95": str(value),
            "mean": str(value),
            "min": str(value),
            "max": str(value),
            "unit": "ms",
        }
    )
    return row


def actual_producer_summary_rows(
    *,
    scenario: str = "sand-fall",
    source_sha: str = "2" * 40,
    run_id: str | None = None,
    elapsed_wall: float = 10.0,
    wall_per_tick: float = 10.0,
    sustained_tps: float = 100.0,
) -> list[dict[str, object]]:
    """Build the exact historical producer header/vocabulary for Mode A."""

    run_id = run_id or f"g8b-{scenario}-test"
    production = headless_common(
        scenario=scenario,
        source_sha=source_sha,
        run_id=run_id,
        mode="production_throughput",
        profiling=False,
    )
    profiled = headless_common(
        scenario=scenario,
        source_sha=source_sha,
        run_id=run_id,
        mode="isolated_profiled_tick",
        profiling=True,
    )
    rows: list[dict[str, object]] = []
    for name, value, unit in (
        ("elapsed_wall", elapsed_wall, "ms"),
        ("wall_per_tick", wall_per_tick, "ms/tick"),
        ("sustained_tps", sustained_tps, "ticks/s"),
    ):
        row = dict(production)
        row.update(
            {
                "selection": "trial",
                "trial": "1",
                "tick_start": "0",
                "tick_end": "0",
                "metric_type": "throughput_trial",
                "name": name,
                "value": str(value),
                "unit": unit,
            }
        )
        rows.append(row)
    for name, value, unit in (
        ("wall_per_tick", wall_per_tick, "ms/tick"),
        ("sustained_tps", sustained_tps, "ticks/s"),
    ):
        row = stats_summary_row(production, "throughput_summary", name, value)
        row.update({"selection": "all_trials", "trial": "all", "unit": unit})
        rows.append(row)
    rows.append(stats_summary_row(profiled, "pass", "activity_wake", 0.001))
    return rows


def aggregation_replay_contract_fixture(
    root: Path,
) -> tuple[Path, dict[str, object], Path]:
    pilot_id = "g8c-pilot-111111111111-fixture"
    original = root / pilot_id
    original.mkdir()
    repo = root / "source-repo"
    repo.mkdir()
    profile = verifier._expected_profile("aggregation-replay")

    binary_payloads = {
        "benchmark": b"benchmark-binary",
        "windows": b"windows-binary",
    }
    binary_names = {
        "benchmark": "powdergame-benchmark.exe",
        "windows": "powdergame-windows.exe",
    }
    binaries: dict[str, dict[str, object]] = {}
    for role, payload in binary_payloads.items():
        relative = f"frozen-binary/{binary_names[role]}"
        path = original / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(payload)
        binaries[role] = {
            "path": f"source-pilot/{relative}",
            "size": len(payload),
            "sha256": hashlib.sha256(payload).hexdigest(),
            "build_profile": "release",
        }

    environment = {
        "GIT_CONFIG_COUNT": "1",
        "GIT_CONFIG_KEY_0": "safe.directory",
        "GIT_CONFIG_VALUE_0": repo.resolve().as_posix(),
    }
    started = "2026-08-18T00:00:00.000000Z"
    ended = "2026-08-18T00:00:01.000000Z"
    command_paths: list[str] = []

    def write_process(
        relative: str,
        *,
        role: str,
        scenario: str | None,
        argv: list[str],
        outputs: list[str],
    ) -> None:
        record_path = original / relative
        record_path.parent.mkdir(parents=True, exist_ok=True)
        stem = relative[:-5]
        stdout = (
            "build/stdout.log"
            if role == "isolated-locked-release-build"
            else f"{stem}.stdout.log"
        )
        stderr = (
            "build/stderr.log"
            if role == "isolated-locked-release-build"
            else f"{stem}.stderr.log"
        )
        for log in (stdout, stderr):
            log_path = original / log
            log_path.parent.mkdir(parents=True, exist_ok=True)
            log_path.write_bytes(b"")
        for output in outputs:
            output_path = original / output
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_bytes(f"fixture:{output}\n".encode("utf-8"))
        record = {
            "schema_version": verifier.PROCESS_SCHEMA,
            "role": role,
            "scenario": scenario,
            "argv": argv,
            "cwd": str(repo.resolve()),
            "started_at_utc": started,
            "ended_at_utc": ended,
            "exit_code": 0,
            "environment_overrides": environment,
            "stdout_path": stdout,
            "stderr_path": stderr,
            "expected_outputs": outputs,
        }
        record_path.write_text(json.dumps(record), encoding="utf-8")
        command_paths.append(relative)

    build_target = root / f".{pilot_id}-build-fixture"
    write_process(
        "build/COMMAND.json",
        role="isolated-locked-release-build",
        scenario=None,
        argv=[
            str(root / "cargo.exe"),
            "build",
            "--locked",
            "--release",
            "--target-dir",
            str(build_target.resolve()),
            "-p",
            "powdergame-benchmark",
            "-p",
            "powdergame-windows",
        ],
        outputs=[],
    )
    for index, scenario in enumerate(verifier.SCENARIOS, 1):
        outputs = [
            f"raw/headless/{scenario}/summary.csv",
            f"raw/headless/{scenario}/summary_raw_ticks.csv",
            f"raw/headless/{scenario}/summary_raw_cells.csv",
            f"raw/headless/{scenario}/summary_raw_chunks.csv",
        ]
        write_process(
            f"process/{index:02d}-{scenario}-headless.json",
            role="headless-mode-a-b",
            scenario=scenario,
            argv=[
                str(original / "frozen-binary/powdergame-benchmark.exe"),
                "--scenario",
                scenario,
                "--width",
                str(profile["width"]),
                "--height",
                str(profile["height"]),
                "--chunk",
                str(profile["chunk_size"]),
                "--sleep",
                "on",
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
                str(original / outputs[0]),
            ],
            outputs=outputs,
        )
    for offset, (role, mode) in enumerate(
        (
            ("windowed-production-coexistence", "coexistence"),
            ("windowed-gpu-render-timing", "render-profile"),
        )
    ):
        for scenario_index, scenario in enumerate(verifier.SCENARIOS, 1):
            process_index = 6 + offset * 5 + scenario_index - 1
            if mode == "coexistence":
                csv_relative = f"raw/coexistence/{scenario}/mode-c-coexistence.csv"
                metadata_relative = (
                    f"raw/coexistence/{scenario}/mode-c-coexistence.json"
                )
                measurement_args = [
                    "--measurement-frames",
                    str(profile["mode_c_measurement_frames"]),
                ]
            else:
                csv_relative = (
                    f"raw/render-profile/{scenario}/mode-d-render-profile.csv"
                )
                metadata_relative = (
                    f"raw/render-profile/{scenario}/mode-d-render-profile.json"
                )
                measurement_args = [
                    "--profile-frames",
                    str(profile["mode_d_profile_frames"]),
                ]
            write_process(
                f"process/{process_index:02d}-{scenario}-{mode}.json",
                role=role,
                scenario=scenario,
                argv=[
                    str(original / "frozen-binary/powdergame-windows.exe"),
                    "--g8c-worker",
                    "--mode",
                    mode,
                    "--run-id",
                    pilot_id,
                    "--binary-sha256",
                    str(binaries["windows"]["sha256"]),
                    "--scenario",
                    scenario,
                    "--width",
                    str(profile["width"]),
                    "--height",
                    str(profile["height"]),
                    "--chunk",
                    str(profile["chunk_size"]),
                    "--sleep",
                    "on",
                    "--threshold",
                    str(profile["sleep_threshold"]),
                    "--prewarm-secs",
                    str(profile["prewarm_secs"]),
                    "--trials",
                    str(profile["trials"]),
                    "--target-tps",
                    str(profile["target_tps"]),
                    *measurement_args,
                    "--raw-csv",
                    str(original / csv_relative),
                    "--metadata-json",
                    str(original / metadata_relative),
                ],
                outputs=[csv_relative, metadata_relative],
            )

    run = root / "g8c-aggregation-replay-fixture"
    run.mkdir()
    shutil.copytree(original, run / "source-pilot")
    implementation_records: dict[str, dict[str, object]] = {}
    for role, relative, payload in (
        ("coordinator", "verification/frozen-coordinator.py", b"coordinator"),
        ("verifier", "verification/frozen-verifier.py", b"verifier"),
    ):
        path = run / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(payload)
        implementation_records[role] = {
            "path": relative,
            "size": len(payload),
            "sha256": hashlib.sha256(payload).hexdigest(),
        }

    original_inventory = verifier._inventory_run(original)
    inventory_digest = verifier._inventory_digest(original_inventory)
    entries = [
        {
            "path": relative,
            "replay_path": f"source-pilot/{relative}",
            "size": entry.size_bytes,
            "sha256": entry.sha256,
        }
        for relative, entry in sorted(original_inventory.items())
    ]
    replay_id = run.name
    inventory_manifest = {
        "schema_version": verifier.REPLAY_INPUT_SCHEMA,
        "replay_run_id": replay_id,
        "source_pilot_id": pilot_id,
        "source_pilot_path": str(original.resolve()),
        "inputs_root": "source-pilot",
        "pre_replay_digest": inventory_digest,
        "post_aggregation_digest": inventory_digest,
        "unchanged": True,
        "entry_count": len(entries),
        "total_bytes": sum(entry.size_bytes for entry in original_inventory.values()),
        "entries": entries,
    }
    inventory_path = run / "SOURCE_PILOT_INPUT_MANIFEST.json"
    inventory_path.write_text(json.dumps(inventory_manifest), encoding="utf-8")
    manifest: dict[str, object] = {
        "matrix_run_id": replay_id,
        "build_command_record": None,
        "command_record_paths": [],
        "frozen_binaries": binaries,
        "independent_verifier": {
            **implementation_records["verifier"],
            "expected_argv": [],
            "execution_timing": "after receipt and package; result is delivery sibling and does not mutate matrix run",
        },
        "aggregation_replay": {
            "source_pilot_id": pilot_id,
            "source_pilot_path": str(original.resolve()),
            "source_pilot_inventory_path": "SOURCE_PILOT_INPUT_MANIFEST.json",
            "source_pilot_inventory_sha256": verifier.sha256_file(inventory_path),
            "source_pilot_inventory_digest": inventory_digest,
            "source_pilot_file_count": len(entries),
            "source_pilot_total_bytes": sum(
                entry.size_bytes for entry in original_inventory.values()
            ),
            "inputs_root": "source-pilot",
            "source_pilot_command_record_paths": command_paths,
            "non_evidence": True,
            "gpu_measurement_reused_for_parser_validation": True,
            "measurement_subprocess_count": 0,
            "executable_invocation_count": 0,
            "gpu_context_count": 0,
            "launched_process_count": 0,
            "replay_implementation": implementation_records,
        },
    }
    return run, manifest, original


def window_row(
    schema: str,
    scenario: str,
    frame: int,
    elapsed_ms: float,
    scheduled: int,
    executed: int,
    *,
    gpu: bool = False,
) -> dict[str, object]:
    row: dict[str, object] = {
        "schema_version": schema,
        "scenario": scenario,
        "trial": 1,
        "frame_index": frame,
        "sim_tick": scheduled,
        "window_elapsed_ms": elapsed_ms,
        "frame_wall_ms": elapsed_ms if frame == 0 else elapsed_ms / (frame + 1),
        "scheduled_sim_ticks": scheduled,
        "sim_ticks_executed": executed,
        "catch_up_ticks": max(executed - 1, 0),
        "missed_simulation_deadlines": max(executed - 1, 0),
        "presented": 1,
        "surface_error": "",
    }
    if gpu:
        row.update(
            {
                "gpu_start_tick": 100 + frame * 20,
                "gpu_end_tick": 110 + frame * 20,
                "gpu_render_ms": 0.01,
                "timestamp_period_ns": 1000.0,
            }
        )
    return row


class PercentileAndWindowTests(unittest.TestCase):
    def test_universal_g8a_percentile_is_half_up_index(self) -> None:
        values = [1.0, 2.0, 3.0, 4.0]
        self.assertEqual(verifier._rust_percentile(values, 50.0), 3.0)
        self.assertEqual(verifier._nearest_rank(values, 0.50), 3.0)

    def test_mode_c_cumulative_scheduled_denominator_is_not_summed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "mode-c.csv"
            rows = [
                window_row(
                    verifier.COEXISTENCE_SCHEMA,
                    "sand-fall",
                    frame,
                    float((frame + 1) * 100),
                    (frame + 1) * 6,
                    6,
                )
                for frame in range(3)
            ]
            write_csv(path, verifier.COEXISTENCE_HEADER, rows)
            _, aggregate = verifier._validate_window_rows(
                path,
                mode="coexistence",
                scenario="sand-fall",
                config=tiny_config(),
            )
            self.assertAlmostEqual(aggregate["missed_deadline_ratio"], 15 / 18)
            self.assertNotAlmostEqual(aggregate["missed_deadline_ratio"], 15 / 36)

    def test_mode_d_requires_positive_ordered_timestamps(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "mode-d.csv"
            rows = [
                window_row(
                    verifier.RENDER_PROFILE_SCHEMA,
                    "water-flow",
                    frame,
                    float((frame + 1) * 20),
                    frame + 1,
                    1,
                    gpu=True,
                )
                for frame in range(2)
            ]
            rows[1]["gpu_end_tick"] = rows[1]["gpu_start_tick"]
            write_csv(path, verifier.RENDER_PROFILE_HEADER, rows)
            with self.assertRaisesRegex(verifier.VerificationError, "timestamps"):
                verifier._validate_window_rows(
                    path,
                    mode="render-profile",
                    scenario="water-flow",
                    config=tiny_config(),
                )


class WindowLifecycleMetadataTests(unittest.TestCase):
    def validate(self, value: object, *, mode: str = "coexistence") -> None:
        verifier._validate_window_lifecycle(
            value,
            mode=mode,
            scenario="sand-fall",
            required_width=1600,
            required_height=900,
        )

    def test_canonical_and_stale_events_are_accepted_for_both_window_modes(
        self,
    ) -> None:
        for mode in ("coexistence", "render-profile"):
            with self.subTest(mode=mode):
                self.validate(valid_window_lifecycle(), mode=mode)

    def test_fatal_live_resize_is_rejected(self) -> None:
        lifecycle = valid_window_lifecycle()
        event = lifecycle["events"][1]
        event.update(
            event_kind="redraw_guard",
            classification="fatal_noncanonical_live_size",
            payload_width=0,
            payload_height=0,
            live_width=0,
            live_height=0,
        )
        lifecycle["stale_payload_count"] = 0
        lifecycle["fatal_live_resize_count"] = 1
        with self.assertRaisesRegex(
            verifier.VerificationError, "noncanonical live size"
        ):
            self.validate(lifecycle)

    def test_lifecycle_count_tamper_is_rejected(self) -> None:
        lifecycle = valid_window_lifecycle()
        lifecycle["stale_payload_count"] = 2
        with self.assertRaisesRegex(verifier.VerificationError, "counter mismatch"):
            self.validate(lifecycle)

    def test_boolean_integer_is_rejected(self) -> None:
        lifecycle = valid_window_lifecycle()
        lifecycle["event_count"] = True
        with self.assertRaisesRegex(verifier.VerificationError, "must be an integer"):
            self.validate(lifecycle)


class HeadlessArithmeticTests(unittest.TestCase):
    def validate_summary(
        self,
        rows: list[dict[str, object]],
        *,
        scenario: str = "sand-fall",
        source_sha: str = "2" * 40,
    ) -> dict[str, object]:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "summary.csv"
            write_csv(path, verifier.SUMMARY_HEADER, rows)
            _, _, aggregate = verifier._validate_headless_summary(
                path,
                scenario=scenario,
                source_sha=source_sha,
                config=tiny_config(),
            )
            return dict(aggregate)

    def test_mode_a_statistics_are_reconstructed(self) -> None:
        aggregate = self.validate_summary(actual_producer_summary_rows())
        self.assertEqual(aggregate["tps"]["p50"], 100.0)
        self.assertEqual(aggregate["wall_ms_per_tick"]["p95"], 10.0)
        self.assertEqual(
            aggregate["trials"],
            {
                1: {
                    "elapsed_wall": 10.0,
                    "wall_per_tick": 10.0,
                    "sustained_tps": 100.0,
                }
            },
        )

    def test_missing_wall_per_tick_is_rejected(self) -> None:
        rows = [
            row
            for row in actual_producer_summary_rows()
            if not (
                row["metric_type"] == "throughput_summary"
                and row["name"] == "wall_per_tick"
            )
        ]
        with self.assertRaisesRegex(
            verifier.VerificationError, "throughput external contract mismatch"
        ):
            self.validate_summary(rows)

    def test_duplicate_wall_per_tick_is_rejected(self) -> None:
        rows = actual_producer_summary_rows()
        duplicate = next(
            row
            for row in rows
            if row["metric_type"] == "throughput_summary"
            and row["name"] == "wall_per_tick"
        )
        rows.append(dict(duplicate))
        with self.assertRaisesRegex(verifier.VerificationError, "duplicate metric"):
            self.validate_summary(rows)

    def test_internal_wall_alias_is_rejected(self) -> None:
        rows = actual_producer_summary_rows()
        for row in rows:
            if (
                row["metric_type"] == "throughput_summary"
                and row["name"] == "wall_per_tick"
            ):
                row["name"] = "wall_ms_per_tick"
        with self.assertRaisesRegex(verifier.VerificationError, "internal raw alias"):
            self.validate_summary(rows)

    def test_wrong_wall_per_tick_units_are_rejected(self) -> None:
        for unit in ("s/tick", "us/tick", ""):
            with self.subTest(unit=unit):
                rows = actual_producer_summary_rows()
                for row in rows:
                    if row["name"] == "wall_per_tick":
                        row["unit"] = unit
                with self.assertRaisesRegex(verifier.VerificationError, "unit must be"):
                    self.validate_summary(rows)

    def test_trial_row_is_not_accepted_as_all_trials_summary(self) -> None:
        rows = actual_producer_summary_rows()
        for row in rows:
            if (
                row["metric_type"] == "throughput_summary"
                and row["name"] == "wall_per_tick"
            ):
                row.update(
                    metric_type="throughput_trial",
                    selection="trial",
                    trial="1",
                    value="10.0",
                    count="",
                    p50="",
                    p95="",
                    mean="",
                    min="",
                    max="",
                )
        with self.assertRaisesRegex(verifier.VerificationError, "duplicate metric"):
            self.validate_summary(rows)

    def test_wrong_scenario_row_is_rejected(self) -> None:
        rows = actual_producer_summary_rows()
        rows[0]["method_note"] = "fixture; scenario=water-flow"
        with self.assertRaisesRegex(verifier.VerificationError, "bound exclusively"):
            self.validate_summary(rows)

    def test_mixed_scenario_tags_are_rejected(self) -> None:
        rows = actual_producer_summary_rows()
        rows[0]["method_note"] = "fixture; scenario=sand-fall; scenario=water-flow"
        with self.assertRaisesRegex(verifier.VerificationError, "bound exclusively"):
            self.validate_summary(rows)

    def test_nonfinite_throughput_value_is_rejected(self) -> None:
        rows = actual_producer_summary_rows()
        for row in rows:
            if (
                row["metric_type"] == "throughput_trial"
                and row["name"] == "wall_per_tick"
            ):
                row["value"] = "nan"
        with self.assertRaisesRegex(verifier.VerificationError, "must be finite"):
            self.validate_summary(rows)

    def test_wrong_measurement_mode_and_selection_are_rejected(self) -> None:
        for field, value, pattern in (
            ("measurement_mode", "isolated_profiled_tick", "measurement_mode"),
            ("selection", "trial", "selection must be all_trials"),
        ):
            with self.subTest(field=field):
                rows = actual_producer_summary_rows()
                target = next(
                    row
                    for row in rows
                    if row["metric_type"] == "throughput_summary"
                    and row["name"] == "wall_per_tick"
                )
                target[field] = value
                with self.assertRaisesRegex(verifier.VerificationError, pattern):
                    self.validate_summary(rows)

    def test_mode_b_rejects_group_duration_tamper(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "raw-ticks.csv"
            scenario = "pressure-burst"
            sha = "3" * 40
            run_id = "g8b-pressure-burst-test"
            common = headless_common(
                scenario=scenario,
                source_sha=sha,
                run_id=run_id,
                mode="isolated_profiled_tick",
                profiling=True,
            )
            row: dict[str, object] = {name: "" for name in verifier.RAW_TICK_HEADER}
            row.update({key: value for key, value in common.items() if key in row})
            row.update(
                {
                    "trial": 1,
                    "sample_id": 0,
                    "tick_index": 0,
                    "tick_start": 0,
                    "tick_end": 0,
                    "timestamp_unit": "raw_gpu_tick",
                    "duration_unit": "milliseconds",
                    "group_definition": verifier.GROUP_DEFINITION,
                }
            )
            for index, name in enumerate(verifier.PASS_NAMES):
                row[f"{name}_start_tick"] = index
                row[f"{name}_end_tick"] = index + 1
                row[f"pass_{name}_ms"] = 0.001
            group_values = {
                name: len(members) * 0.001 for name, members in verifier.GROUPS.items()
            }
            for name, value in group_values.items():
                row[f"group_{name}_ms"] = value
            row["gpu_pass_sum_ms"] = 0.017
            row["gpu_tick_envelope_ms"] = 0.017
            row["residual_ms"] = 0.0
            summary_rows: list[dict[str, object]] = []
            for name in verifier.PASS_NAMES:
                summary_rows.append(stats_summary_row(common, "pass", name, 0.001))
            for name, value in group_values.items():
                summary_rows.append(
                    stats_summary_row(common, "grouped_subsystem", name, value)
                )
                summary_rows.append(
                    stats_summary_row(
                        common,
                        "grouped_envelope_ratio",
                        name,
                        value / 0.017 * 100.0,
                    )
                )
            for name, value in (
                ("gpu_tick_envelope", 0.017),
                ("gpu_pass_sum", 0.017),
                ("diagnostic_residual", 0.0),
            ):
                summary_rows.append(stats_summary_row(common, "envelope", name, value))
            row["group_thermal_conduction_ms"] = 0.5
            write_csv(path, verifier.RAW_TICK_HEADER, [row])
            with self.assertRaisesRegex(
                verifier.VerificationError, "grouped subsystem"
            ):
                verifier._validate_raw_ticks(
                    path,
                    scenario=scenario,
                    source_sha=sha,
                    config=tiny_config(),
                    run_id=run_id,
                    summary_rows=summary_rows,
                )

    def test_mode_d_rejects_unpresented_row_even_with_error_detail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "mode-d.csv"
            rows = [
                window_row(
                    verifier.RENDER_PROFILE_SCHEMA,
                    "fire-heat",
                    frame,
                    float((frame + 1) * 20),
                    frame + 1,
                    1,
                    gpu=True,
                )
                for frame in range(2)
            ]
            rows[0]["presented"] = 0
            rows[0]["surface_error"] = (
                "kind=timeout;reconfigured=false;fatal=false;message=timeout"
            )
            write_csv(path, verifier.RENDER_PROFILE_HEADER, rows)
            with self.assertRaisesRegex(verifier.VerificationError, "failed surface"):
                verifier._validate_window_rows(
                    path,
                    mode="render-profile",
                    scenario="fire-heat",
                    config=tiny_config(),
                )

    def test_schema_compatibility_rejects_historical_or_unknown_window_schema(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "mode-c.csv"
            rows = [
                window_row(
                    "powdergame-g8c-coexistence-v0",
                    "sand-fall",
                    frame,
                    float((frame + 1) * 20),
                    frame + 1,
                    1,
                )
                for frame in range(3)
            ]
            write_csv(path, verifier.COEXISTENCE_HEADER, rows)
            with self.assertRaisesRegex(verifier.VerificationError, "schema"):
                verifier._validate_window_rows(
                    path,
                    mode="coexistence",
                    scenario="sand-fall",
                    config=tiny_config(),
                )


class ScenarioAndCensusTests(unittest.TestCase):
    def test_duplicate_scenario_is_rejected(self) -> None:
        records = [{"scenario": value} for value in verifier.SCENARIOS]
        records[-1] = {"scenario": "sand-fall"}
        with self.assertRaisesRegex(verifier.VerificationError, "duplicates"):
            verifier._validate_scenario_sequence(records)

    def test_missing_scenario_is_rejected(self) -> None:
        records = [{"scenario": value} for value in verifier.SCENARIOS[:-1]]
        with self.assertRaisesRegex(verifier.VerificationError, "missing"):
            verifier._validate_scenario_sequence(records)

    def test_streamed_census_recounts_overlapping_activity_and_chunks(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            sha = "1" * 40
            run_id = "g8b-sand-fall-test"
            cells = root / "cells.csv"
            chunks = root / "chunks.csv"
            write_csv(
                cells,
                verifier.RAW_CELL_HEADER,
                [
                    {
                        "schema_version": verifier.INNER_HEADLESS_SCHEMA,
                        "run_id": run_id,
                        "commit_sha": sha,
                        "git_state": "clean",
                        "census_tick": 4,
                        "index": index,
                        "activity_mask": mask,
                    }
                    for index, mask in enumerate((0, 1, 3, 15))
                ],
            )
            write_csv(
                chunks,
                verifier.RAW_CHUNK_HEADER,
                [
                    {
                        "schema_version": verifier.INNER_HEADLESS_SCHEMA,
                        "run_id": run_id,
                        "commit_sha": sha,
                        "git_state": "clean",
                        "census_tick": 4,
                        "index": 0,
                        "activity_mask": 15,
                        "chunk_state": 0,
                    }
                ],
            )
            expected = {
                "total_cells": 4,
                "any_active_cells": 3,
                "matter_active_cells": 3,
                "thermal_active_cells": 2,
                "pressure_active_cells": 1,
                "reaction_active_cells": 1,
                "total_chunks": 1,
                "active_chunks": 1,
                "runnable_chunks": 1,
                "sleeping_chunks": 0,
            }
            summary = [
                {
                    "metric_type": "activity_census",
                    "name": name,
                    "trial": "n/a",
                    "value": str(value),
                }
                for name, value in expected.items()
            ]
            observed = verifier._validate_census(
                cells,
                chunks,
                scenario="sand-fall",
                source_sha=sha,
                run_id=run_id,
                config=tiny_config(),
                summary_rows=summary,
            )
            self.assertEqual(observed, expected)


class AggregationReplayVerifierTests(unittest.TestCase):
    def validate_replay(
        self,
        run: Path,
        manifest: dict[str, object],
    ) -> verifier.AggregationReplayBinding:
        replay = manifest["aggregation_replay"]
        source_pilot_id = str(replay["source_pilot_id"])
        source_pilot_path = Path(str(replay["source_pilot_path"]))
        with (
            mock.patch.object(verifier, "REPLAY_SOURCE_PILOT_ID", source_pilot_id),
            mock.patch.object(verifier, "REPLAY_SOURCE_PILOT_PATH", source_pilot_path),
        ):
            return verifier._validate_aggregation_replay(
                run,
                manifest,
                verifier._expected_profile("aggregation-replay"),
            )

    def test_coordinator_and_verifier_inventory_digest_use_posix_ordinal_order(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for relative, payload in (
                ("Z-last.txt", b"upper"),
                ("a-first.txt", b"lower"),
                ("Nested/Mixed.txt", b"nested"),
            ):
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(payload)
            coordinator_entries, coordinator_digest, _ = (
                coordinator.directory_byte_inventory(root)
            )
            verifier_inventory = verifier._inventory_run(root)
            self.assertEqual(
                [entry["path"] for entry in coordinator_entries],
                sorted(verifier_inventory),
            )
            self.assertEqual(
                coordinator_digest,
                verifier._inventory_digest(verifier_inventory),
            )

    def test_replay_binds_original_copy_processes_and_implementation_without_subprocess(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run, manifest, original = aggregation_replay_contract_fixture(
                Path(temporary)
            )
            with mock.patch.object(
                subprocess,
                "run",
                side_effect=AssertionError("replay verification must not spawn"),
            ) as spawned:
                binding = self.validate_replay(run, manifest)
                verifier._validate_replay_original_unchanged(binding)
            spawned.assert_not_called()
            self.assertEqual(binding.original_root, original.resolve())
            self.assertEqual(binding.process_result["count"], 16)
            self.assertEqual(set(binding.implementation), {"coordinator", "verifier"})

    def test_replay_rejects_copied_input_tamper(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run, manifest, _ = aggregation_replay_contract_fixture(Path(temporary))
            target = run / "source-pilot/raw/headless/sand-fall/summary.csv"
            target.write_bytes(b"tampered")
            with self.assertRaisesRegex(
                verifier.VerificationError, "original/inventory/copied bytes differ"
            ):
                self.validate_replay(run, manifest)

    def test_replay_rejects_unapproved_source_pilot_identity(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run, manifest, _ = aggregation_replay_contract_fixture(Path(temporary))
            with self.assertRaisesRegex(
                verifier.VerificationError, "approved replacement"
            ):
                verifier._validate_aggregation_replay(
                    run,
                    manifest,
                    verifier._expected_profile("aggregation-replay"),
                )

    def test_replay_detects_original_pilot_change_after_initial_validation(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run, manifest, original = aggregation_replay_contract_fixture(
                Path(temporary)
            )
            binding = self.validate_replay(run, manifest)
            (original / "late-change.txt").write_text("changed", encoding="utf-8")
            with self.assertRaisesRegex(
                verifier.VerificationError, "changed while aggregation replay"
            ):
                verifier._validate_replay_original_unchanged(binding)

    def test_replay_rejects_run_root_relative_capture_process_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run, manifest, _ = aggregation_replay_contract_fixture(Path(temporary))
            replay = manifest["aggregation_replay"]
            replay["source_pilot_command_record_paths"] = [
                f"source-pilot/{path}"
                for path in replay["source_pilot_command_record_paths"]
            ]
            with self.assertRaisesRegex(
                verifier.VerificationError, "order/layout mismatch"
            ):
                self.validate_replay(run, manifest)

    def test_replay_raw_metadata_must_record_original_not_copied_path(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            original = root / "pilot/raw/mode-c.csv"
            copied = root / "replay/source-pilot/raw/mode-c.csv"
            original.parent.mkdir(parents=True)
            copied.parent.mkdir(parents=True)
            original.write_bytes(b"same")
            copied.write_bytes(b"same")
            verifier._validate_recorded_raw_csv_path(
                original,
                original,
                "fixture",
            )
            with self.assertRaisesRegex(verifier.VerificationError, "raw_csv path"):
                verifier._validate_recorded_raw_csv_path(
                    copied,
                    original,
                    "fixture",
                )


class SourceAndPackageTests(unittest.TestCase):
    def _git(self, repo: Path, *arguments: str) -> subprocess.CompletedProcess[bytes]:
        result = subprocess.run(
            ["git", "-c", f"safe.directory={repo.resolve().as_posix()}", *arguments],
            cwd=repo,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        if result.returncode != 0:
            self.fail(result.stderr.decode("utf-8", errors="replace"))
        return result

    def _live_source_fixture(
        self, root: Path, *, mode: str, dirty: bool
    ) -> tuple[Path, dict[str, object], str, list[str]]:
        origin = root / "origin.git"
        subprocess.run(
            ["git", "init", "--bare", str(origin)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        )
        repo = root / "source"
        repo.mkdir()
        self._git(repo, "init", "-b", verifier.REQUIRED_BRANCH)
        self._git(repo, "config", "user.name", "Verifier Test")
        self._git(repo, "config", "user.email", "verifier@example.invalid")
        tracked = repo / "tracked.txt"
        tracked.write_bytes(b"committed bytes\n")
        self._git(repo, "add", "tracked.txt")
        self._git(repo, "commit", "-m", "fixture")
        self._git(repo, "remote", "add", "origin", str(origin))
        self._git(repo, "push", "-u", "origin", verifier.REQUIRED_BRANCH)
        sha = self._git(repo, "rev-parse", "HEAD").stdout.decode().strip()
        if dirty:
            tracked.write_bytes(b"dirty tracked build bytes\n")
        status_bytes = self._git(
            repo, "status", "--porcelain=v1", "-z", "--untracked-files=all"
        ).stdout
        status = [value.decode("utf-8") for value in status_bytes.split(b"\0") if value]
        font = root / "consola-fixture.ttf"
        font.write_bytes(b"external font bytes")
        entries = []
        for kind, source, archive_path in (
            ("repository_tracked", tracked, "repository/tracked.txt"),
            ("external_build_input", font, "external/Windows/Fonts/consola.ttf"),
        ):
            entries.append(
                {
                    "kind": kind,
                    "source_path": (
                        "tracked.txt" if kind == "repository_tracked" else str(source)
                    ),
                    "archive_path": archive_path,
                    "size": source.stat().st_size,
                    "sha256": verifier.sha256_file(source),
                }
            )
        digest_state = hashlib.sha256()
        for entry in entries:
            digest_state.update(str(entry["archive_path"]).encode())
            digest_state.update(b"\0")
            digest_state.update(str(entry["size"]).encode())
            digest_state.update(b"\0")
            digest_state.update(str(entry["sha256"]).encode())
            digest_state.update(b"\n")
        digest = digest_state.hexdigest()
        source_identity = {
            "sha": sha,
            "branch": verifier.REQUIRED_BRANCH,
            "git_state": "dirty" if dirty else "clean",
            "dirty_scope": "tracked-only" if dirty else None,
            "status_porcelain": status,
            "upstream": (
                f"origin/{verifier.REQUIRED_BRANCH}" if mode == "official" else None
            ),
            "upstream_sha": sha if mode == "official" else None,
            "ahead_behind": ["0", "0"] if mode == "official" else None,
        }
        run = root / "run"
        run.mkdir()
        matrix_run_id = f"matrix-{mode}-fixture"
        source_manifest = {
            "schema_version": verifier.SOURCE_INPUT_SCHEMA,
            "matrix_run_id": matrix_run_id,
            "run_mode": mode,
            "source": source_identity,
            "source_input_digest": digest,
            "entry_count": len(entries),
            "entries": entries,
            "roles": {
                "SOURCE_INPUT_BYTES.zip": "exact working-tree build-input bytes plus external Consolas input",
                "GIT_SOURCE_ARCHIVE.zip": "canonical Git archive for source SHA; pilot tracked changes intentionally differ",
            },
        }
        (run / "SOURCE_INPUT_MANIFEST.json").write_text(
            json.dumps(source_manifest), encoding="utf-8"
        )
        with zipfile.ZipFile(
            run / "SOURCE_INPUT_BYTES.zip", "w", allowZip64=True
        ) as archive:
            archive.writestr("repository/tracked.txt", tracked.read_bytes())
            archive.writestr("external/Windows/Fonts/consola.ttf", font.read_bytes())
        (run / "GIT_SOURCE_ARCHIVE.zip").write_bytes(
            self._git(repo, "archive", "--format=zip", sha).stdout
        )
        manifest = {
            "matrix_run_id": matrix_run_id,
            "source": {
                **source_identity,
                "input_digest": digest,
                "input_manifest": "SOURCE_INPUT_MANIFEST.json",
                "exact_input_archive": "SOURCE_INPUT_BYTES.zip",
                "canonical_git_archive": "GIT_SOURCE_ARCHIVE.zip",
            },
        }
        return repo, manifest, sha, status

    def _source_fixture(self, root: Path) -> tuple[dict[str, object], str]:
        sha = "a" * 40
        tracked = b"exact tracked bytes\n"
        font = b"font bytes"
        entries = [
            {
                "kind": "repository_tracked",
                "source_path": "tracked.txt",
                "archive_path": "repository/tracked.txt",
                "size": len(tracked),
                "sha256": hashlib.sha256(tracked).hexdigest(),
            },
            {
                "kind": "external_build_input",
                "source_path": "C:\\Windows\\Fonts\\consola.ttf",
                "archive_path": "external/Windows/Fonts/consola.ttf",
                "size": len(font),
                "sha256": hashlib.sha256(font).hexdigest(),
            },
        ]
        digest_state = hashlib.sha256()
        for entry in entries:
            digest_state.update(str(entry["archive_path"]).encode())
            digest_state.update(b"\0")
            digest_state.update(str(entry["size"]).encode())
            digest_state.update(b"\0")
            digest_state.update(str(entry["sha256"]).encode())
            digest_state.update(b"\n")
        digest = digest_state.hexdigest()
        source_identity = {
            "sha": sha,
            "branch": verifier.REQUIRED_BRANCH,
            "git_state": "clean",
            "dirty_scope": None,
            "status_porcelain": [],
            "upstream": f"origin/{verifier.REQUIRED_BRANCH}",
            "upstream_sha": sha,
            "ahead_behind": ["0", "0"],
        }
        source_manifest = {
            "schema_version": verifier.SOURCE_INPUT_SCHEMA,
            "matrix_run_id": f"g8c-official-matrix-{sha[:12]}-{digest[:12]}",
            "run_mode": "official",
            "source": source_identity,
            "source_input_digest": digest,
            "entry_count": 2,
            "entries": entries,
            "roles": {
                "SOURCE_INPUT_BYTES.zip": "exact working-tree build-input bytes plus external Consolas input",
                "GIT_SOURCE_ARCHIVE.zip": "canonical Git archive for source SHA; pilot tracked changes intentionally differ",
            },
        }
        (root / "SOURCE_INPUT_MANIFEST.json").write_text(
            json.dumps(source_manifest), encoding="utf-8"
        )
        with zipfile.ZipFile(
            root / "SOURCE_INPUT_BYTES.zip", "w", allowZip64=True
        ) as archive:
            archive.writestr("repository/tracked.txt", tracked)
            archive.writestr("external/Windows/Fonts/consola.ttf", font)
        with zipfile.ZipFile(
            root / "GIT_SOURCE_ARCHIVE.zip", "w", allowZip64=True
        ) as archive:
            archive.comment = sha.encode("ascii")
            archive.writestr("tracked.txt", tracked)
        manifest = {
            "matrix_run_id": source_manifest["matrix_run_id"],
            "source": {
                **source_identity,
                "input_digest": digest,
                "input_manifest": "SOURCE_INPUT_MANIFEST.json",
                "exact_input_archive": "SOURCE_INPUT_BYTES.zip",
                "canonical_git_archive": "GIT_SOURCE_ARCHIVE.zip",
            },
        }
        return manifest, sha

    def test_exact_source_bytes_and_git_archive_identity_pass(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest, sha = self._source_fixture(root)
            result = verifier._validate_source_inputs(
                root, manifest, sha, "official", None
            )
            self.assertEqual(result["tracked_file_count"], 1)
            self.assertEqual(result["external_file_count"], 1)

    def test_live_official_and_dirty_pilot_git_checks_reach_exact_source_bytes(
        self,
    ) -> None:
        for mode, dirty in (("official", False), ("pilot", True)):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                repo, manifest, sha, status = self._live_source_fixture(
                    root, mode=mode, dirty=dirty
                )
                result = verifier._validate_source_inputs(
                    root / "run", manifest, sha, mode, repo
                )
                self.assertTrue(result["live_git"]["checked"])
                self.assertEqual(result["live_git"]["head_sha"], sha)
                self.assertEqual(result["live_git"]["status_porcelain"], status)
                self.assertEqual(
                    result["live_git"]["git_state"], "dirty" if dirty else "clean"
                )

    def test_tampered_git_archive_commit_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest, sha = self._source_fixture(root)
            archive_path = root / "GIT_SOURCE_ARCHIVE.zip"
            with zipfile.ZipFile(archive_path, "a", allowZip64=True) as archive:
                archive.comment = b"b" * 40
            with self.assertRaisesRegex(verifier.VerificationError, "embedded commit"):
                verifier._validate_source_inputs(root, manifest, sha, "official", None)

    def test_pilot_allows_tracked_inventory_to_differ_from_head_archive(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest, sha = self._source_fixture(root)
            source_path = root / "SOURCE_INPUT_MANIFEST.json"
            source_manifest = json.loads(source_path.read_text(encoding="utf-8"))
            for identity in (source_manifest["source"], manifest["source"]):
                identity["git_state"] = "dirty"
                identity["dirty_scope"] = "tracked-only"
                identity["status_porcelain"] = ["A  tracked.txt"]
                identity["upstream"] = None
                identity["upstream_sha"] = None
                identity["ahead_behind"] = None
            source_manifest["run_mode"] = "pilot"
            source_path.write_text(json.dumps(source_manifest), encoding="utf-8")
            with zipfile.ZipFile(
                root / "GIT_SOURCE_ARCHIVE.zip", "w", allowZip64=True
            ) as archive:
                archive.comment = sha.encode("ascii")
                archive.writestr("historical.txt", b"HEAD bytes")
            result = verifier._validate_source_inputs(
                root, manifest, sha, "pilot", None
            )
            self.assertEqual(result["tracked_file_count"], 1)

    def test_source_schema_compatibility_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest, sha = self._source_fixture(root)
            path = root / "SOURCE_INPUT_MANIFEST.json"
            value = json.loads(path.read_text(encoding="utf-8"))
            value["schema_version"] = "powdergame-g8c-source-input-v0"
            path.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaisesRegex(verifier.VerificationError, "schema"):
                verifier._validate_source_inputs(root, manifest, sha, "official", None)

    def test_package_hash_sidecar_and_exact_copy(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            parent = Path(temporary)
            run = parent / "run-id"
            run.mkdir()
            (run / "evidence.txt").write_bytes(b"evidence")
            package = parent / "G8C_MATRIX_PACKAGE.zip"
            with zipfile.ZipFile(package, "w", allowZip64=True) as archive:
                archive.write(run / "evidence.txt", "run-id/evidence.txt")
            sidecar = parent / "G8C_MATRIX_PACKAGE_SHA256.txt"
            sidecar.write_bytes(
                f"{verifier.sha256_file(package)}  {package.name}\n".encode("utf-8")
            )
            digest, size = verifier._validate_package_copy(
                run, package, sidecar, verifier._inventory_run(run)
            )
            self.assertEqual(digest, verifier.sha256_file(package))
            self.assertEqual(size, package.stat().st_size)
            sidecar.write_bytes(f"{'0' * 64}  {package.name}\n".encode("utf-8"))
            with self.assertRaisesRegex(verifier.VerificationError, "package SHA-256"):
                verifier._validate_package_copy(
                    run, package, sidecar, verifier._inventory_run(run)
                )

    def test_duplicate_package_member_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            package = Path(temporary) / "duplicate.zip"
            with warnings.catch_warnings():
                warnings.simplefilter("ignore", UserWarning)
                with zipfile.ZipFile(package, "w", allowZip64=True) as archive:
                    archive.writestr("run/a.txt", b"one")
                    archive.writestr("run/a.txt", b"two")
            with self.assertRaisesRegex(verifier.VerificationError, "duplicate"):
                verifier.MatrixPackage(package, "run")

    def test_hash_inventory_rejects_late_post_receipt_member(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary)
            required = (
                "G8C_MATRIX_MANIFEST.json",
                "SOURCE_INPUT_MANIFEST.json",
                "SOURCE_INPUT_BYTES.zip",
                "GIT_SOURCE_ARCHIVE.zip",
                "frozen-binary/powdergame-benchmark.exe",
                "frozen-binary/powdergame-windows.exe",
            )
            for relative in required:
                path = run / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(relative.encode())
            inventory = verifier._inventory_run(run)
            hashes = "".join(
                f"{inventory[name].sha256}  {name}\n" for name in sorted(inventory)
            )
            (run / "HASHES.sha256").write_bytes(hashes.encode("utf-8"))
            (run / "G8C_MATRIX_RECEIPT.json").write_text("{}", encoding="utf-8")
            verifier._validate_run_hashes(run, verifier._inventory_run(run))
            (run / "late-after-receipt.txt").write_text("late", encoding="utf-8")
            with self.assertRaisesRegex(
                verifier.VerificationError, "inventory mismatch"
            ):
                verifier._validate_run_hashes(run, verifier._inventory_run(run))


class ReportAndDecisionTests(unittest.TestCase):
    def test_cli_routes_required_repo_root_into_live_verification(self) -> None:
        fake_result = {
            "matrix_run_id": "fixture",
            "recommendation": "NEEDS_HUMAN_REVIEW",
            "package_sha256": "a" * 64,
        }
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            arguments = [
                "--run-dir",
                str(root / "run"),
                "--package",
                str(root / "package.zip"),
                "--sidecar",
                str(root / "sidecar.txt"),
                "--repo-root",
                str(root / "source"),
            ]
            with (
                mock.patch.object(
                    verifier, "verify_matrix", return_value=fake_result
                ) as verify,
                contextlib.redirect_stdout(io.StringIO()),
            ):
                self.assertEqual(verifier.main(arguments), 0)
            verify.assert_called_once_with(
                root / "run",
                root / "package.zip",
                root / "sidecar.txt",
                repo_root=root / "source",
                write_result=None,
            )

    def test_matrix_csv_tamper_is_rejected(self) -> None:
        rows = [
            {
                "source_sha": "a" * 40,
                "scenario": "sand-fall",
                "mode_a_tps_p50": 120.0,
                "bottleneck_group": "Thermal",
            }
        ]
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "matrix.csv"
            write_csv(path, tuple(rows[0]), [{**rows[0], "mode_a_tps_p50": 119.0}])
            with self.assertRaisesRegex(verifier.VerificationError, "mismatch"):
                verifier._validate_matrix_csv(path, rows)

    def test_optimization_decision_recomputed_from_thresholds(self) -> None:
        base = {
            "scenario": "sand-fall",
            "mode_a_tps_p50": 120.0,
            "mode_b_gpu_envelope_p95_ms": 8.0,
            "mode_c_simulation_tps": 60.0,
            "mode_c_render_fps": 60.0,
            "mode_c_missed_deadline_ratio": 0.0,
            "mode_c_frame_p95_ms": 20.0,
            "mode_d_gpu_render_p95_ms": 8.0,
            "mode_c_failed_surface_frames": 0,
            "mode_c_surface_errors": 0,
            "mode_c_device_errors": 0,
            "mode_d_surface_errors": 0,
            "mode_d_device_errors": 0,
            "rtx_5090_32gib_tracked_memory_ratio": 0.1,
        }
        recommendation, _ = verifier._optimization_recommendation([base])
        self.assertEqual(recommendation, "PROCEED_TO_G9")
        blocked = {**base, "mode_c_frame_p95_ms": 40.0}
        recommendation, reasons = verifier._optimization_recommendation([blocked])
        self.assertEqual(recommendation, "OPTIMIZATION_REVIEW_REQUIRED")
        self.assertTrue(any("frame P95" in reason for reason in reasons))

    def test_scenario_row_includes_raw_accounting_and_working_chunk_fields(
        self,
    ) -> None:
        fields = {
            **{
                f"group_{name}_ms": {"p50": float(index + 1)}
                for index, name in enumerate(verifier.GROUPS)
            },
            "gpu_tick_envelope_ms": {"p50": 8.0, "p95": 9.0},
            "residual_ms": {"p50": 0.25},
        }
        census = {
            "total_cells": 4,
            "any_active_cells": 4,
            "matter_active_cells": 1,
            "thermal_active_cells": 1,
            "pressure_active_cells": 1,
            "reaction_active_cells": 1,
            "total_chunks": 1,
            "active_chunks": 1,
            "runnable_chunks": 1,
            "sleeping_chunks": 0,
        }
        headless = {
            "mode_a": {
                "tps": {"p50": 120.0, "mean": 121.0, "min": 119.0, "max": 122.0},
                "wall_ms_per_tick": {"p50": 8.0, "p95": 9.0},
            },
            "mode_b": {"fields": fields},
            "census": census,
            "memory": {"total_tracked": 1024},
        }
        coexistence = {
            "simulation_tps": {"p50": 60.0},
            "actual_simulation_ticks": 600,
            "render_fps": {"p50": 59.9},
            "presented_frames": 599,
            "frame_wall_ms": {"p50": 16.7, "p95": 17.0, "p99": 18.0},
            "missed_deadline_ratio": 0.01,
            "missed_simulation_deadlines": 6,
            "catch_up_ticks": 6,
            "failed_surface_frames": 0,
            "surface_errors": 0,
            "device_errors": 0,
        }
        render = {
            "gpu_render_ms": {"p50": 2.0, "p95": 3.0, "count": 16},
            "surface_errors": 0,
            "device_errors": 0,
        }
        row = verifier._scenario_matrix_row(
            "sand-fall", "a" * 40, headless, coexistence, render
        )
        self.assertEqual(row["working_chunks"], 1)
        self.assertEqual(row["mode_c_actual_simulation_ticks"], 600)
        self.assertEqual(row["mode_c_presented_frames"], 599)
        self.assertEqual(row["mode_c_missed_simulation_deadlines"], 6)
        self.assertEqual(row["mode_c_catch_up_ticks"], 6)
        self.assertEqual(row["mode_d_measured_frames"], 16)

    def test_render_fps_below_fifo_tolerance_is_a_hard_blocker(self) -> None:
        row = {
            "scenario": "sand-fall",
            "mode_a_tps_p50": 120.0,
            "mode_b_gpu_envelope_p95_ms": 8.0,
            "mode_c_simulation_tps": 60.0,
            "mode_c_render_fps": 56.9,
            "mode_c_missed_deadline_ratio": 0.0,
            "mode_c_frame_p95_ms": 20.0,
            "mode_d_gpu_render_p95_ms": 8.0,
            "mode_c_failed_surface_frames": 0,
            "mode_c_surface_errors": 0,
            "mode_c_device_errors": 0,
            "mode_d_surface_errors": 0,
            "mode_d_device_errors": 0,
            "rtx_5090_32gib_tracked_memory_ratio": 0.1,
        }
        recommendation, reasons = verifier._optimization_recommendation([row])
        self.assertEqual(recommendation, "OPTIMIZATION_REVIEW_REQUIRED")
        self.assertTrue(any("render FPS" in reason for reason in reasons))


if __name__ == "__main__":
    unittest.main(verbosity=2)
