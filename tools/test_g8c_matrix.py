from __future__ import annotations

import csv
import contextlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
import zipfile

from tools import g8c_matrix as matrix
from tools import verify_g8c_matrix as verifier


def write_csv(path: Path, fieldnames: list[str], rows: list[dict[str, object]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


class ContractTests(unittest.TestCase):
    def test_profiles_are_exact(self) -> None:
        pilot = matrix.matrix_profile("pilot")
        self.assertEqual(
            (
                pilot["width"],
                pilot["trials"],
                pilot["mode_a_ticks"],
                pilot["mode_b_ticks"],
            ),
            (256, 1, 32, 16),
        )
        self.assertEqual(
            (
                pilot["overhead_ticks"],
                pilot["mode_c_measurement_frames"],
                pilot["mode_d_profile_frames"],
            ),
            (16, 60, 16),
        )
        official = matrix.matrix_profile("official")
        self.assertEqual(
            (
                official["width"],
                official["height"],
                official["trials"],
                official["mode_a_ticks"],
                official["mode_b_ticks"],
                official["overhead_ticks"],
            ),
            (2048, 2048, 3, 1024, 256, 256),
        )
        self.assertEqual(official["mode_c_measurement_secs"], 10.0)
        self.assertEqual(official["mode_d_profile_frames"], 256)
        self.assertEqual(official["present_mode"], "Fifo")

    def test_five_scenarios_are_required_once_in_order(self) -> None:
        matrix.validate_scenario_sequence(matrix.SCENARIOS)
        for invalid in (
            matrix.SCENARIOS[:-1],
            (*matrix.SCENARIOS, matrix.SCENARIOS[-1]),
            tuple(reversed(matrix.SCENARIOS)),
            (*matrix.SCENARIOS[:-1], "calibration"),
        ):
            with self.assertRaises(matrix.MatrixError):
                matrix.validate_scenario_sequence(invalid)

    def test_rust_compatible_nearest_percentile(self) -> None:
        self.assertEqual(matrix.nearest_percentile([1.0, 2.0], 50.0), 2.0)
        stats = matrix.numeric_stats([4.0, 1.0, 2.0, 3.0])
        self.assertEqual(stats["p50"], 3.0)
        self.assertEqual(stats["p95"], 4.0)
        self.assertEqual(matrix.window_numeric_stats([4.0, 1.0, 2.0, 3.0])["p50"], 3.0)
        with self.assertRaises(matrix.MatrixError):
            matrix.numeric_stats([])

    def test_commands_preserve_inner_schema_and_isolate_window_modes(self) -> None:
        profile = matrix.matrix_profile("pilot")
        benchmark = matrix.benchmark_command(
            Path("benchmark.exe"), "sand-fall", profile, Path("summary.csv")
        )
        self.assertIn("--throughput-ticks", benchmark)
        self.assertEqual(benchmark[benchmark.index("--throughput-ticks") + 1], "32")
        self.assertEqual(benchmark[benchmark.index("--profile-ticks") + 1], "16")
        self.assertNotIn("powdergame-g8c-headless-v1", benchmark)
        coexistence = matrix.windows_worker_command(
            Path("windows.exe"),
            "coexistence",
            "sand-fall",
            profile,
            "run",
            "a" * 64,
            Path("c.csv"),
            Path("c.json"),
        )
        render = matrix.windows_worker_command(
            Path("windows.exe"),
            "render-profile",
            "sand-fall",
            profile,
            "run",
            "a" * 64,
            Path("d.csv"),
            Path("d.json"),
        )
        self.assertIn("--measurement-frames", coexistence)
        self.assertNotIn("--profile-frames", coexistence)
        self.assertIn("--profile-frames", render)
        self.assertNotIn("--measurement-frames", render)
        self.assertEqual(render[render.index("--binary-sha256") + 1], "a" * 64)

    def test_cli_is_strict(self) -> None:
        self.assertEqual(matrix.parse_args(["pilot"]).mode, "pilot")
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
            matrix.parse_args(["pilot", "--unknown"])

    def test_worker_metadata_enforces_common_config_and_hardware(self) -> None:
        profile = matrix.matrix_profile("pilot")
        metadata = {
            "schema_version": matrix.COEXISTENCE_SCHEMA,
            "run_id": "run",
            "mode": "coexistence",
            "source_sha": "a" * 40,
            "git_state": "dirty",
            "build_profile": "release",
            "binary_sha256": "b" * 64,
            "scenario": "sand-fall",
            "requested_config": {
                "width": 256,
                "height": 256,
                "chunk_size": 64,
                "sleep_enabled": True,
                "sleep_threshold": 16,
                "prewarm_secs": 2.0,
                "trials": 1,
                "target_tps": 60,
                "measurement_secs": None,
                "measurement_frames": 60,
                "profile_frames": None,
            },
            "actual_surface": {
                "width": 1600,
                "height": 900,
                "format": "Bgra8UnormSrgb",
                "present_mode": "Fifo",
            },
            "window_lifecycle": {
                "required_width": 1600,
                "required_height": 900,
                "initial_live_width": 1600,
                "initial_live_height": 900,
                "last_live_width": 1600,
                "last_live_height": 900,
                "initial_live_size_confirmed": True,
                "canonical_noop_count": 0,
                "stale_payload_count": 1,
                "fatal_live_resize_count": 0,
                "event_count": 1,
                "events": [
                    {
                        "event_kind": "resized",
                        "classification": "stale_payload_ignored",
                        "payload_width": 2864,
                        "payload_height": 1560,
                        "live_width": 1600,
                        "live_height": 900,
                    }
                ],
            },
            "adapter": {
                "name": "NVIDIA GeForce RTX 5090",
                "vendor": 0x10DE,
                "device": 0x2B85,
                "backend": "Dx12",
                "driver": "fixture-driver",
                "driver_info": "fixture-driver-info",
            },
            "hud_enabled": False,
            "inspector_enabled": False,
            "text_diagnostics_enabled": False,
            "screenshot_readback_enabled": False,
            "timestamp_query_enabled": False,
            "device_error_count": 0,
            "device_errors": [],
            "surface_error_count": 0,
            "surface_errors": [],
            "raw_csv": "",
            "trials": [{"trial": 1}],
        }
        # Validate the object through a real JSON read, then mutate one common field.
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "metadata.json"
            metadata["raw_csv"] = str(path.with_suffix(".csv"))
            path.write_text(json.dumps(metadata), encoding="utf-8")
            matrix.validate_worker_metadata(
                path,
                schema=matrix.COEXISTENCE_SCHEMA,
                mode="coexistence",
                scenario="sand-fall",
                run_id="run",
                source_sha="a" * 40,
                source_git_state="dirty",
                binary_sha256="b" * 64,
                profile=profile,
            )
            metadata["requested_config"]["width"] = 255
            path.write_text(json.dumps(metadata), encoding="utf-8")
            with self.assertRaisesRegex(
                matrix.MatrixError, "requested config mismatch"
            ):
                matrix.validate_worker_metadata(
                    path,
                    schema=matrix.COEXISTENCE_SCHEMA,
                    mode="coexistence",
                    scenario="sand-fall",
                    run_id="run",
                    source_sha="a" * 40,
                    source_git_state="dirty",
                    binary_sha256="b" * 64,
                    profile=profile,
                )
            metadata["requested_config"]["width"] = 256
            metadata["device_error_count"] = 0
            metadata["device_errors"] = []
            lifecycle = metadata["window_lifecycle"]
            lifecycle["events"][0].update(
                classification="fatal_noncanonical_live_size",
                live_width=0,
                live_height=0,
            )
            lifecycle["stale_payload_count"] = 0
            lifecycle["fatal_live_resize_count"] = 1
            path.write_text(json.dumps(metadata), encoding="utf-8")
            with self.assertRaisesRegex(matrix.MatrixError, "noncanonical live size"):
                matrix.validate_worker_metadata(
                    path,
                    schema=matrix.COEXISTENCE_SCHEMA,
                    mode="coexistence",
                    scenario="sand-fall",
                    run_id="run",
                    source_sha="a" * 40,
                    source_git_state="dirty",
                    binary_sha256="b" * 64,
                    profile=profile,
                )
            lifecycle["events"][0].update(
                classification="stale_payload_ignored",
                live_width=1600,
                live_height=900,
            )
            lifecycle["fatal_live_resize_count"] = 0
            lifecycle["stale_payload_count"] = 2
            path.write_text(json.dumps(metadata), encoding="utf-8")
            with self.assertRaisesRegex(matrix.MatrixError, "counter mismatch"):
                matrix.validate_worker_metadata(
                    path,
                    schema=matrix.COEXISTENCE_SCHEMA,
                    mode="coexistence",
                    scenario="sand-fall",
                    run_id="run",
                    source_sha="a" * 40,
                    source_git_state="dirty",
                    binary_sha256="b" * 64,
                    profile=profile,
                )
            lifecycle["stale_payload_count"] = 1
            metadata["device_error_count"] = 1
            metadata["device_errors"] = ["injected"]
            path.write_text(json.dumps(metadata), encoding="utf-8")
            with self.assertRaisesRegex(
                matrix.MatrixError, "records device/surface errors"
            ):
                matrix.validate_worker_metadata(
                    path,
                    schema=matrix.COEXISTENCE_SCHEMA,
                    mode="coexistence",
                    scenario="sand-fall",
                    run_id="run",
                    source_sha="a" * 40,
                    source_git_state="dirty",
                    binary_sha256="b" * 64,
                    profile=profile,
                )

    def test_headless_manifest_rejects_cross_file_identity_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run_dir = Path(temporary)
            paths = matrix.headless_paths(run_dir, "sand-fall")
            fields = ["schema_version", "run_id", "commit_sha", "git_state"]
            stable = {
                "schema_version": matrix.INNER_HEADLESS_SCHEMA,
                "run_id": "inner-a",
                "commit_sha": "a" * 40,
                "git_state": "dirty",
            }
            for key in ("summary", "raw_ticks", "raw_cells", "raw_chunks"):
                row = dict(stable)
                if key == "raw_ticks":
                    row["run_id"] = "inner-b"
                write_csv(paths[key], fields, [row])
            with self.assertRaisesRegex(
                matrix.MatrixError, "raw-file identity mismatch"
            ):
                matrix.write_headless_manifest(
                    run_dir,
                    "sand-fall",
                    paths,
                    matrix.matrix_profile("pilot"),
                    "matrix-run",
                    {"sha": "a" * 40, "git_state": "dirty"},
                    {"path": "benchmark.exe", "sha256": "b" * 64},
                )


class SourceSealTests(unittest.TestCase):
    def init_repo(self, root: Path) -> None:
        subprocess.run(
            ["git", "init", "-b", matrix.REQUIRED_BRANCH],
            cwd=root,
            check=True,
            capture_output=True,
        )
        subprocess.run(
            ["git", "config", "user.email", "tests@example.invalid"],
            cwd=root,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "G8C Tests"], cwd=root, check=True
        )
        (root / "tracked.txt").write_bytes(b"one\r\ntwo\r\n")
        subprocess.run(["git", "add", "tracked.txt"], cwd=root, check=True)
        subprocess.run(
            ["git", "commit", "-m", "fixture"],
            cwd=root,
            check=True,
            capture_output=True,
        )

    def test_pilot_allows_tracked_dirty_but_rejects_untracked(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.init_repo(root)
            (root / "tracked.txt").write_bytes(b"changed\r\n")
            state = matrix.capture_source_state(root, "pilot")
            self.assertEqual(state["git_state"], "dirty")
            self.assertEqual(state["dirty_scope"], "tracked-only")
            (root / "untracked.txt").write_text("no", encoding="utf-8")
            with self.assertRaisesRegex(matrix.MatrixError, "untracked"):
                matrix.capture_source_state(root, "pilot")

    def test_exact_source_archive_preserves_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            source.mkdir()
            payload = b"line-one\r\nline-two\n\x00"
            (source / "input.txt").write_bytes(payload)
            entries = [
                {
                    "kind": "repository_tracked",
                    "source_path": "input.txt",
                    "archive_path": "repository/input.txt",
                    "size": len(payload),
                    "sha256": matrix.sha256_bytes(payload),
                }
            ]
            archive = root / "source.zip"
            matrix.write_source_archive(archive, source, entries)
            matrix.verify_source_archive(archive, entries)
            with zipfile.ZipFile(archive) as bundle:
                self.assertEqual(bundle.read("repository/input.txt"), payload)

    def test_official_requires_exact_origin_feature_upstream(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / "source"
            root.mkdir()
            remote = base / "remote.git"
            self.init_repo(root)
            subprocess.run(
                ["git", "init", "--bare", remote], check=True, capture_output=True
            )
            subprocess.run(
                ["git", "remote", "add", "origin", remote], cwd=root, check=True
            )
            subprocess.run(
                ["git", "push", "-u", "origin", matrix.REQUIRED_BRANCH],
                cwd=root,
                check=True,
                capture_output=True,
            )
            self.assertEqual(
                matrix.capture_source_state(root, "official")["upstream"],
                f"origin/{matrix.REQUIRED_BRANCH}",
            )
            subprocess.run(
                ["git", "remote", "add", "other", remote], cwd=root, check=True
            )
            subprocess.run(
                ["git", "fetch", "other"], cwd=root, check=True, capture_output=True
            )
            subprocess.run(
                [
                    "git",
                    "branch",
                    "--set-upstream-to",
                    f"other/{matrix.REQUIRED_BRANCH}",
                ],
                cwd=root,
                check=True,
                capture_output=True,
            )
            with self.assertRaisesRegex(matrix.MatrixError, "exact origin"):
                matrix.capture_source_state(root, "official")

    def test_source_based_run_id_and_no_overwrite(self) -> None:
        self.assertEqual(
            matrix.run_id_for("official", "a" * 40, "b" * 64),
            "g8c-official-matrix-aaaaaaaaaaaa-bbbbbbbbbbbb",
        )
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            matrix.create_run_directory(root, "official", "same")
            with self.assertRaisesRegex(matrix.MatrixError, "rerun is forbidden"):
                matrix.create_run_directory(root, "official", "same")

    def test_isolated_target_cleanup_is_confined_to_its_exact_prefix(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            artifact_root = Path(temporary)
            run_id = "g8c-pilot-fixture"
            isolated = Path(
                tempfile.mkdtemp(prefix=f".{run_id}-build-", dir=artifact_root)
            )
            (isolated / "payload").write_text("temporary", encoding="utf-8")
            matrix.remove_isolated_target(isolated, artifact_root, run_id)
            self.assertFalse(isolated.exists())
            unexpected = artifact_root / "unrelated-target"
            unexpected.mkdir()
            with self.assertRaisesRegex(
                matrix.MatrixError, "unexpected isolated target"
            ):
                matrix.remove_isolated_target(unexpected, artifact_root, run_id)
            self.assertTrue(unexpected.is_dir())


class AggregationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.profile = dict(matrix.matrix_profile("pilot"))
        self.profile.update(
            width=2,
            height=2,
            chunk_size=2,
            trials=1,
            mode_b_ticks=2,
            mode_c_measurement_frames=2,
            mode_d_profile_frames=2,
        )

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def producer_summary_row(self, **overrides: object) -> dict[str, object]:
        row: dict[str, object] = {field: "" for field in matrix.HEADLESS_SUMMARY_HEADER}
        row.update(
            schema_version=matrix.INNER_HEADLESS_SCHEMA,
            run_id="g8b-sand-fall-fixture",
            commit_sha="a" * 40,
            git_state="dirty",
            adapter_name="NVIDIA GeForce RTX 5090",
            vendor_id="0x10DE",
            device_id="0x2B85",
            device_type="DiscreteGpu",
            backend="Dx12",
            driver="fixture-driver",
            profiling_enabled="false",
            build_profile="release",
            width=2,
            height=2,
            chunk_size=2,
            sleep_enabled="true",
            sleep_threshold=16,
            prewarm_requested_secs="2.000000",
            prewarm_ticks=1,
            tick_start=0,
            tick_end=31,
            method_note="actual producer fixture; scenario=sand-fall",
        )
        row.update(overrides)
        return row

    def producer_summary_rows(self) -> list[dict[str, object]]:
        return [
            self.producer_summary_row(
                measurement_mode="production_throughput",
                selection="trial",
                trial=1,
                metric_type="throughput_trial",
                name="elapsed_wall",
                value="256.000000000",
                unit="ms",
            ),
            self.producer_summary_row(
                measurement_mode="production_throughput",
                selection="trial",
                trial=1,
                metric_type="throughput_trial",
                name="wall_per_tick",
                value="8.000000000",
                unit="ms/tick",
            ),
            self.producer_summary_row(
                measurement_mode="production_throughput",
                selection="trial",
                trial=1,
                metric_type="throughput_trial",
                name="sustained_tps",
                value="125.000000000",
                unit="ticks/s",
            ),
            self.producer_summary_row(
                measurement_mode="production_throughput",
                selection="all_trials",
                trial="all",
                metric_type="throughput_summary",
                name="wall_per_tick",
                count=1,
                p50="8.000000000",
                p95="8.000000000",
                mean="8.000000000",
                min="8.000000000",
                max="8.000000000",
                unit="ms/tick",
            ),
            self.producer_summary_row(
                measurement_mode="production_throughput",
                selection="all_trials",
                trial="all",
                metric_type="throughput_summary",
                name="sustained_tps",
                count=1,
                p50="125.000000000",
                p95="125.000000000",
                mean="125.000000000",
                min="125.000000000",
                max="125.000000000",
                unit="ticks/s",
            ),
            self.producer_summary_row(
                profiling_enabled="true",
                timestamp_period_ns="1.000000000",
                measurement_mode="isolated_profiled_tick",
                selection="snapshot",
                trial="n/a",
                tick_start=16,
                tick_end=16,
                metric_type="application_tracked_buffer_allocation",
                name="total_tracked",
                value="1024.000000000",
                unit="bytes",
            ),
        ]

    def headless_fixture(
        self, summary_rows: list[dict[str, object]] | None = None
    ) -> dict[str, Path]:
        paths = matrix.headless_paths(self.root, "sand-fall")
        write_csv(
            paths["summary"],
            list(matrix.HEADLESS_SUMMARY_HEADER),
            summary_rows if summary_rows is not None else self.producer_summary_rows(),
        )
        raw_fields = [
            *matrix.GROUP_FIELDS,
            "gpu_tick_envelope_ms",
            "gpu_pass_sum_ms",
            "residual_ms",
        ]
        raw_rows = [
            {field: index + 1 for index, field in enumerate(raw_fields)},
            {field: index + 2 for index, field in enumerate(raw_fields)},
        ]
        write_csv(paths["raw_ticks"], raw_fields, raw_rows)
        write_csv(
            paths["raw_cells"],
            ["activity_mask"],
            [{"activity_mask": value} for value in (0, 1, 2, 15)],
        )
        write_csv(
            paths["raw_chunks"],
            ["activity_mask", "chunk_state"],
            [{"activity_mask": 3, "chunk_state": 0}],
        )
        return paths

    def test_headless_aggregation_recounts_raw_without_summing_overlaps(self) -> None:
        result = matrix.aggregate_headless(
            self.headless_fixture(), self.profile, "sand-fall"
        )
        self.assertEqual(result["mode_a_tps"]["p50"], 125)
        self.assertEqual(result["mode_a_elapsed_wall_ms"]["p50"], 256)
        self.assertEqual(result["mode_a_wall_ms_per_tick"]["p50"], 8)
        self.assertEqual(result["census"]["total_cells"], 4)
        self.assertEqual(result["census"]["any_active_cells"], 3)
        self.assertEqual(result["census"]["matter_active_cells"], 2)
        self.assertEqual(result["census"]["thermal_active_cells"], 2)
        self.assertEqual(result["tracked_persistent_gpu_bytes"], 1024)

    def assert_headless_summary_rejected(
        self, rows: list[dict[str, object]], pattern: str
    ) -> None:
        with self.assertRaisesRegex(matrix.MatrixError, pattern):
            matrix.aggregate_headless(
                self.headless_fixture(rows), self.profile, "sand-fall"
            )

    def test_headless_actual_producer_name_missing_and_alias_are_rejected(self) -> None:
        rows = self.producer_summary_rows()
        rows = [
            row
            for row in rows
            if not (
                row["metric_type"] == "throughput_trial"
                and row["name"] == "wall_per_tick"
            )
        ]
        self.assert_headless_summary_rejected(rows, "row identity inventory mismatch")

        rows = self.producer_summary_rows()
        for row in rows:
            if row["name"] == "wall_per_tick":
                row["name"] = "wall_ms_per_tick"
        self.assert_headless_summary_rejected(rows, "internal alias.*forbidden")

    def test_headless_duplicate_wall_per_tick_is_rejected(self) -> None:
        rows = self.producer_summary_rows()
        duplicate = dict(
            next(
                row
                for row in rows
                if row["metric_type"] == "throughput_trial"
                and row["name"] == "wall_per_tick"
            )
        )
        rows.append(duplicate)
        self.assert_headless_summary_rejected(rows, "duplicate.*metric identity")

    def test_headless_wall_per_tick_unit_is_strict(self) -> None:
        for wrong_unit in ("s/tick", "us/tick", ""):
            with self.subTest(wrong_unit=wrong_unit):
                rows = self.producer_summary_rows()
                next(
                    row
                    for row in rows
                    if row["metric_type"] == "throughput_trial"
                    and row["name"] == "wall_per_tick"
                )["unit"] = wrong_unit
                self.assert_headless_summary_rejected(rows, "require unit 'ms/tick'")

    def test_headless_trial_row_is_not_confused_with_all_trials_summary(self) -> None:
        rows = self.producer_summary_rows()
        summary = next(
            row
            for row in rows
            if row["metric_type"] == "throughput_summary"
            and row["name"] == "wall_per_tick"
        )
        summary.update(metric_type="throughput_trial", selection="trial", trial=2)
        self.assert_headless_summary_rejected(rows, "row identity inventory mismatch")

    def test_headless_unexpected_trial_or_summary_identity_is_rejected(self) -> None:
        for extra in (
            self.producer_summary_row(
                measurement_mode="production_throughput",
                selection="trial",
                trial=99,
                metric_type="throughput_trial",
                name="wall_per_tick",
                value="8.000000000",
                unit="ms/tick",
            ),
            self.producer_summary_row(
                measurement_mode="production_throughput",
                selection="all_trials",
                trial="all",
                metric_type="throughput_summary",
                name="elapsed_wall",
                count=1,
                p50="256.000000000",
                p95="256.000000000",
                mean="256.000000000",
                min="256.000000000",
                max="256.000000000",
                unit="ms",
            ),
        ):
            with self.subTest(
                identity=(extra["metric_type"], extra["trial"], extra["name"])
            ):
                self.assert_headless_summary_rejected(
                    [*self.producer_summary_rows(), extra],
                    "row identity inventory mismatch",
                )

    def test_headless_scenario_identity_and_schema_are_strict(self) -> None:
        rows = self.producer_summary_rows()
        rows[0]["method_note"] = "actual producer fixture; scenario=water-flow"
        self.assert_headless_summary_rejected(rows, "scenario identity mismatch")

        rows = self.producer_summary_rows()
        rows[0]["schema_version"] = "invented-schema"
        self.assert_headless_summary_rejected(rows, "unexpected headless schema")

    def test_headless_nonfinite_and_reconstruction_mismatch_are_rejected(self) -> None:
        rows = self.producer_summary_rows()
        next(
            row
            for row in rows
            if row["metric_type"] == "throughput_trial"
            and row["name"] == "wall_per_tick"
        )["value"] = "NaN"
        self.assert_headless_summary_rejected(rows, "non-finite numeric field")

        rows = self.producer_summary_rows()
        next(
            row
            for row in rows
            if row["metric_type"] == "throughput_trial"
            and row["name"] == "elapsed_wall"
        )["value"] = "255.000000000"
        self.assert_headless_summary_rejected(rows, "elapsed wall.*mismatch")

    def test_headless_summary_statistics_are_reconstructed_from_trials(self) -> None:
        rows = self.producer_summary_rows()
        next(
            row
            for row in rows
            if row["metric_type"] == "throughput_summary"
            and row["name"] == "wall_per_tick"
        )["p50"] = "7.000000000"
        self.assert_headless_summary_rejected(rows, "wall_per_tick.p50.*mismatch")

    def coexistence_fixture(self) -> Path:
        path = self.root / "c.csv"
        fields = [
            "schema_version",
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
        ]
        rows = [
            {
                "schema_version": matrix.COEXISTENCE_SCHEMA,
                "trial": 1,
                "frame_index": index,
                "sim_tick": index + 1,
                "window_elapsed_ms": (index + 1) * 16.0,
                "frame_wall_ms": 16.0,
                "scheduled_sim_ticks": index + 1,
                "sim_ticks_executed": 1,
                "catch_up_ticks": 0,
                "missed_simulation_deadlines": int(index == 1),
                "presented": "1",
                "surface_error": "",
            }
            for index in range(2)
        ]
        write_csv(path, fields, rows)
        return path

    def render_fixture(self, *, error: bool = False) -> Path:
        path = self.root / "d.csv"
        fields = [
            "schema_version",
            "trial",
            "frame_index",
            "sim_tick",
            "presented",
            "gpu_start_tick",
            "gpu_end_tick",
            "gpu_render_ms",
            "timestamp_period_ns",
            "surface_error",
        ]
        rows = [
            {
                "schema_version": matrix.RENDER_PROFILE_SCHEMA,
                "trial": 1,
                "frame_index": index,
                "sim_tick": index + 10,
                "presented": "0" if error and index == 1 else "1",
                "gpu_start_tick": 100 * index + 1,
                "gpu_end_tick": 100 * index + 101,
                "gpu_render_ms": 0.0001,
                "timestamp_period_ns": 1,
                "surface_error": "Lost" if error and index == 1 else "",
            }
            for index in range(2)
        ]
        write_csv(path, fields, rows)
        return path

    def test_window_aggregates_and_mode_d_strictness(self) -> None:
        coexistence = matrix.aggregate_coexistence(
            self.coexistence_fixture(), self.profile
        )
        self.assertEqual(coexistence["presented_frames"], 2)
        self.assertEqual(coexistence["missed_deadline_ratio"], 0.5)
        render = matrix.aggregate_render_profile(self.render_fixture(), self.profile)
        self.assertAlmostEqual(render["gpu_render_ms"]["p50"], 0.0001)
        with self.assertRaisesRegex(matrix.MatrixError, "not successfully presented"):
            matrix.aggregate_render_profile(
                self.render_fixture(error=True), self.profile
            )

    def test_memory_guard_affects_recommendation(self) -> None:
        row = {
            "scenario": "sand-fall",
            "mode_a_tps_p50": 200.0,
            "mode_b_gpu_envelope_p95_ms": 1.0,
            "mode_c_simulation_tps": 60.0,
            "mode_c_render_fps": 60.0,
            "mode_c_missed_deadline_ratio": 0.0,
            "mode_c_frame_p95_ms": 16.7,
            "mode_d_gpu_render_p95_ms": 1.0,
            "mode_c_failed_surface_frames": 0,
            "mode_c_surface_errors": 0,
            "mode_c_device_errors": 0,
            "mode_d_surface_errors": 0,
            "mode_d_device_errors": 0,
            "rtx_5090_32gib_tracked_memory_ratio": 0.8,
        }
        decision, reasons = matrix.optimization_recommendation([row] * 5)
        self.assertEqual(decision, "OPTIMIZATION_REVIEW_REQUIRED")
        self.assertTrue(any("GPU bytes" in reason for reason in reasons))


class IndependentContractTests(unittest.TestCase):
    def test_frozen_verifier_command_binds_required_live_repo_root(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            command = matrix.verifier_command(
                root / "run/verification/frozen-verifier.py",
                root / "run",
                root / "run-delivery/G8C_MATRIX_PACKAGE.zip",
                root / "run-delivery/G8C_MATRIX_PACKAGE_SHA256.txt",
                root / "run-delivery/G8C_MATRIX_VERIFICATION.json",
                root / "source",
            )
            self.assertEqual(command[-2:], ["--repo-root", str(root / "source")])
            parsed = verifier._parse_args(command[3:])
            self.assertEqual(parsed.repo_root, root / "source")

    def fixture_inputs(
        self,
    ) -> tuple[
        dict[str, object], dict[str, object], dict[str, object], dict[str, object]
    ]:
        census = {
            "total_cells": 4,
            "any_active_cells": 3,
            "matter_active_cells": 2,
            "thermal_active_cells": 2,
            "pressure_active_cells": 1,
            "reaction_active_cells": 1,
            "total_chunks": 1,
            "active_chunks": 1,
            "runnable_chunks": 1,
            "sleeping_chunks": 0,
        }
        group_stats = {
            field: {"p50": float(index + 1)}
            for index, field in enumerate(matrix.GROUP_FIELDS)
        }
        coordinator_headless = {
            "mode_a_tps": {"p50": 120.0, "mean": 121.0, "min": 119.0, "max": 123.0},
            "mode_a_wall_ms_per_tick": {"p50": 8.0, "p95": 9.0},
            "mode_b": {
                **group_stats,
                "gpu_tick_envelope_ms": {"p50": 7.0, "p95": 8.0},
                "residual_ms": {"p50": 0.2},
            },
            "census": census,
            "tracked_persistent_gpu_bytes": 1024,
        }
        verifier_headless = {
            "mode_a": {
                "tps": coordinator_headless["mode_a_tps"],
                "wall_ms_per_tick": coordinator_headless["mode_a_wall_ms_per_tick"],
            },
            "mode_b": {
                "fields": {
                    **{
                        f"group_{name}_ms": group_stats[field]
                        for name, field in zip(verifier.GROUPS, matrix.GROUP_FIELDS)
                    },
                    "gpu_tick_envelope_ms": {"p50": 7.0, "p95": 8.0},
                    "residual_ms": {"p50": 0.2},
                }
            },
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
            "gpu_render_ms": {"p50": 3.0, "p95": 4.0, "count": 16},
            "surface_errors": 0,
            "device_errors": 0,
        }
        return coordinator_headless, verifier_headless, coexistence, render

    def test_coordinator_and_verifier_reconstruct_the_same_row_and_decision(
        self,
    ) -> None:
        coordinator_headless, verifier_headless, coexistence, render = (
            self.fixture_inputs()
        )
        coordinator_row = matrix.scenario_matrix_row(
            "sand-fall", "a" * 40, coordinator_headless, coexistence, render
        )
        verifier_row = verifier._scenario_matrix_row(
            "sand-fall", "a" * 40, verifier_headless, coexistence, render
        )
        self.assertEqual(coordinator_row, verifier_row)
        self.assertEqual(
            matrix.optimization_recommendation(
                [coordinator_row] * len(matrix.SCENARIOS)
            ),
            verifier._optimization_recommendation(
                [verifier_row] * len(verifier.SCENARIOS)
            ),
        )

    def test_coordinator_reports_match_independent_reconstruction(self) -> None:
        coordinator_headless, _, coexistence, render = self.fixture_inputs()
        rows = [
            matrix.scenario_matrix_row(
                scenario, "a" * 40, coordinator_headless, coexistence, render
            )
            for scenario in matrix.SCENARIOS
        ]
        for mode in ("pilot", "official"):
            if mode == "pilot":
                recommendation = "NEEDS_HUMAN_REVIEW"
                reasons = [
                    "non-evidence pilot validates orchestration only and must never be used for a G9 decision"
                ]
            else:
                recommendation, reasons = matrix.optimization_recommendation(rows)
            with tempfile.TemporaryDirectory() as temporary:
                run_dir = Path(temporary)
                reports = matrix.write_reports(
                    run_dir,
                    "matrix-fixture",
                    mode,
                    rows,
                    recommendation,
                    reasons,
                )
                verifier._validate_reports(
                    run_dir,
                    reports,
                    matrix_run_id="matrix-fixture",
                    run_mode=mode,
                    rows=rows,
                    recommendation=recommendation,
                    reasons=reasons,
                )


class PublicationTests(unittest.TestCase):
    def test_hash_inventory_uses_case_sensitive_posix_path_order(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run_dir = Path(temporary)
            for relative in ("Zeta/file.bin", "alpha.bin", "Beta/file.bin"):
                path = run_dir / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(relative.encode("utf-8"))
            hashes, entries = matrix.write_hash_inventory(run_dir)
            paths = [entry["path"] for entry in entries]
            self.assertEqual(paths, sorted(paths))
            parsed = verifier._parse_hash_inventory(
                hashes.read_bytes(), "fixture HASHES.sha256"
            )
            self.assertEqual(list(parsed), paths)

    def test_hashes_receipt_package_and_exact_delivery_names(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            parent = Path(temporary)
            run_dir = parent / "matrix-id"
            run_dir.mkdir()
            matrix.write_new_json(
                run_dir / "G8C_MATRIX_MANIFEST.json",
                {"schema_version": matrix.MATRIX_SCHEMA},
            )
            matrix.write_new_text(run_dir / "payload.txt", "payload\n")
            report_path = run_dir / "report" / "G8C_MATRIX.json"
            matrix.write_new_json(report_path, {"recommendation": "NEEDS_HUMAN_REVIEW"})
            hashes, entries = matrix.write_hash_inventory(run_dir)
            self.assertNotIn("HASHES.sha256", hashes.read_text(encoding="utf-8"))
            binaries = {
                "benchmark": {"path": "benchmark.exe", "sha256": "a" * 64},
                "windows": {"path": "windows.exe", "sha256": "b" * 64},
            }
            receipt = matrix.write_receipt(
                run_dir,
                "matrix-id",
                "pilot",
                {"sha": "c" * 40},
                "d" * 64,
                binaries,
                {"matrix_json": "report/G8C_MATRIX.json"},
                {"path": "verification/frozen-verifier.py", "sha256": "e" * 64},
                "NEEDS_HUMAN_REVIEW",
                entries,
            )
            self.assertTrue(receipt.is_file())
            package, sidecar, package_hash = matrix.create_package(run_dir)
            self.assertEqual(package.name, "G8C_MATRIX_PACKAGE.zip")
            self.assertEqual(sidecar.name, "G8C_MATRIX_PACKAGE_SHA256.txt")
            self.assertEqual(package.parent.name, "matrix-id-delivery")
            self.assertEqual(
                sidecar.read_text(encoding="utf-8"), f"{package_hash}  {package.name}\n"
            )
            with zipfile.ZipFile(package) as archive:
                self.assertIn("matrix-id/G8C_MATRIX_RECEIPT.json", archive.namelist())

    def test_logged_process_records_exact_command_and_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run_dir = Path(temporary) / "run"
            run_dir.mkdir()
            output = run_dir / "output.txt"
            code = "from pathlib import Path; Path(r'%s').write_text('ok')" % output
            record = matrix.run_logged(
                [sys.executable, "-c", code],
                cwd=run_dir,
                stdout_path=run_dir / "stdout.log",
                stderr_path=run_dir / "stderr.log",
                record_path=run_dir / "command.json",
                role="fixture",
                scenario=None,
                run_root=run_dir,
                expected_outputs=[output],
            )
            self.assertEqual(record["exit_code"], 0)
            self.assertEqual(record["expected_outputs"], ["output.txt"])
            self.assertEqual(
                json.loads((run_dir / "command.json").read_text())["argv"][0],
                sys.executable,
            )


class AggregationReplayContractTests(unittest.TestCase):
    def test_replay_rejects_any_unapproved_source_pilot_path(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / matrix.REPLAY_SOURCE_PILOT_ID
            source.mkdir()
            with self.assertRaisesRegex(
                matrix.MatrixError, "exact approved replacement"
            ):
                matrix.run_aggregation_replay(
                    Path(matrix.__file__).resolve().parents[1],
                    Path(temporary) / "artifacts",
                    source,
                )

    def test_replay_cli_requires_and_scopes_source_pilot(self) -> None:
        arguments = matrix.parse_args(
            ["aggregation-replay", "--source-pilot", "source-pilot"]
        )
        self.assertEqual(arguments.mode, "aggregation-replay")
        self.assertEqual(arguments.source_pilot, Path("source-pilot"))
        for invalid in (
            ["aggregation-replay"],
            ["pilot", "--source-pilot", "source-pilot"],
        ):
            with (
                contextlib.redirect_stderr(io.StringIO()),
                self.assertRaises(SystemExit),
            ):
                matrix.parse_args(invalid)

    def test_replay_inventory_and_copy_are_byte_exact_and_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            destination = root / "destination"
            (source / "nested").mkdir(parents=True)
            (source / "z.txt").write_bytes(b"z")
            (source / "nested" / "a.bin").write_bytes(b"abc")
            entries, digest, total = matrix.directory_byte_inventory(source)
            self.assertEqual(
                [entry["path"] for entry in entries], ["nested/a.bin", "z.txt"]
            )
            self.assertEqual(total, 4)
            matrix.copy_inventory_new(source, destination, entries)
            copied_entries, copied_digest, copied_total = (
                matrix.directory_byte_inventory(destination)
            )
            self.assertEqual(copied_entries, entries)
            self.assertEqual(copied_digest, digest)
            self.assertEqual(copied_total, total)
            (source / "z.txt").write_bytes(b"changed")
            _, changed_digest, _ = matrix.directory_byte_inventory(source)
            self.assertNotEqual(changed_digest, digest)

    def test_replay_inventory_uses_case_sensitive_posix_path_order(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for relative in ("Zeta/file.bin", "alpha.bin", "Beta/file.bin"):
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(relative.encode("utf-8"))
            entries, digest, _ = matrix.directory_byte_inventory(root)
            self.assertEqual(
                [entry["path"] for entry in entries],
                sorted(entry["path"] for entry in entries),
            )
            self.assertEqual(
                digest,
                verifier._inventory_digest(verifier._inventory_run(root)),
            )

    def test_replay_reports_are_explicitly_non_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run_dir = Path(temporary)
            rows = [
                {
                    "scenario": scenario,
                    "mode_a_tps_p50": 120.0,
                    "mode_b_gpu_envelope_p95_ms": 1.0,
                    "mode_c_simulation_tps": 60.0,
                    "mode_c_render_fps": 60.0,
                    "mode_c_frame_p95_ms": 16.7,
                    "mode_d_gpu_render_p95_ms": 1.0,
                    "tracked_persistent_gpu_gib": 0.1,
                    "rtx_5090_32gib_tracked_memory_ratio": 0.01,
                    "bottleneck_group": "Thermal",
                }
                for scenario in matrix.SCENARIOS
            ]
            reports = matrix.write_reports(
                run_dir,
                "g8c-aggregation-replay-fixture",
                "aggregation-replay",
                rows,
                "NEEDS_HUMAN_REVIEW",
                ["parser validation only"],
            )
            report = json.loads((run_dir / reports["matrix_json"]).read_text())
            self.assertFalse(report["official_evidence"])
            self.assertTrue(report["pilot_must_never_be_promoted"])
            decision = (run_dir / reports["optimization_decision"]).read_text()
            self.assertIn("NON-EVIDENCE PILOT", decision)

    def test_full_replay_publication_path_cannot_launch_a_process(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source_pilot = root / "approved-source-pilot"
            artifact_root = root / "artifacts"
            source_pilot.mkdir()
            source_sha = "a" * 40
            source_digest = "b" * 64
            source_state = {"sha": source_sha, "git_state": "dirty"}
            profile = matrix.matrix_profile("pilot")
            matrix.write_new_json(
                source_pilot / "SOURCE_INPUT_MANIFEST.json",
                {
                    "schema_version": matrix.SOURCE_SCHEMA,
                    "matrix_run_id": source_pilot.name,
                    "run_mode": "pilot",
                    "source_input_digest": source_digest,
                    "source": source_state,
                },
            )
            (source_pilot / "SOURCE_INPUT_BYTES.zip").write_bytes(b"source")
            (source_pilot / "GIT_SOURCE_ARCHIVE.zip").write_bytes(b"git")
            (source_pilot / "frozen-binary").mkdir()
            (source_pilot / "frozen-binary" / "powdergame-benchmark.exe").write_bytes(
                b"benchmark"
            )
            (source_pilot / "frozen-binary" / "powdergame-windows.exe").write_bytes(
                b"windows"
            )
            matrix.write_new_json(
                source_pilot / "build" / "COMMAND.json",
                {"schema_version": matrix.PROCESS_SCHEMA, "exit_code": 0},
            )
            for index in range(1, 16):
                matrix.write_new_json(
                    source_pilot / "process" / f"{index:02d}-capture.json",
                    {"schema_version": matrix.PROCESS_SCHEMA, "exit_code": 0},
                )
            for scenario in matrix.SCENARIOS:
                matrix.write_new_json(
                    source_pilot / "scenarios" / scenario / "HEADLESS_MANIFEST.json",
                    {
                        "matrix_run_id": source_pilot.name,
                        "scenario": scenario,
                        "common_config": profile,
                    },
                )
                headless = matrix.headless_paths(source_pilot, scenario)
                for key in ("summary", "raw_ticks", "raw_cells", "raw_chunks"):
                    headless[key].parent.mkdir(parents=True, exist_ok=True)
                    headless[key].write_bytes(key.encode("ascii"))
                for mode, directory, stem in (
                    ("coexistence", "coexistence", "mode-c-coexistence"),
                    ("render-profile", "render-profile", "mode-d-render-profile"),
                ):
                    raw = source_pilot / "raw" / directory / scenario / f"{stem}.csv"
                    raw.parent.mkdir(parents=True, exist_ok=True)
                    raw.write_text("fixture\n", encoding="utf-8")
                    matrix.write_new_json(
                        raw.with_suffix(".json"),
                        {
                            "mode": mode,
                            "scenario": scenario,
                            "run_id": source_pilot.name,
                            "raw_csv": str(raw),
                        },
                    )

            stats = {
                "count": 1,
                "p50": 1.0,
                "p95": 1.0,
                "p99": 1.0,
                "mean": 1.0,
                "min": 1.0,
                "max": 1.0,
            }
            headless_result = {
                "mode_a_tps": stats,
                "mode_a_wall_ms_per_tick": stats,
                "mode_b": {
                    **{field: stats for field in matrix.GROUP_FIELDS},
                    "gpu_tick_envelope_ms": stats,
                    "gpu_pass_sum_ms": stats,
                    "residual_ms": stats,
                },
                "census": {
                    "total_cells": 1,
                    "any_active_cells": 1,
                    "matter_active_cells": 1,
                    "thermal_active_cells": 1,
                    "pressure_active_cells": 1,
                    "reaction_active_cells": 1,
                    "total_chunks": 1,
                    "active_chunks": 1,
                    "runnable_chunks": 1,
                    "sleeping_chunks": 0,
                },
                "tracked_persistent_gpu_bytes": 1024,
                "adapter": {
                    "name": "NVIDIA GeForce RTX 5090",
                    "vendor_id": "0x10DE",
                    "device_id": "0x2B85",
                    "backend": "Dx12",
                },
            }
            coexistence_result = {
                "simulation_tps": stats,
                "actual_simulation_ticks": 60,
                "render_fps": stats,
                "presented_frames": 60,
                "frame_wall_ms": stats,
                "missed_deadline_ratio": 0.0,
                "missed_simulation_deadlines": 0,
                "catch_up_ticks": 0,
                "failed_surface_frames": 0,
                "surface_errors": 0,
                "device_errors": 0,
            }
            render_result = {
                "gpu_render_ms": stats,
                "surface_errors": 0,
                "device_errors": 0,
            }
            metadata_calls: list[tuple[Path, Path, str]] = []

            def validate_metadata(path: Path, **kwargs: object) -> dict[str, object]:
                recorded = Path(str(kwargs["recorded_raw_csv_path"])).resolve()
                metadata = json.loads(path.read_text(encoding="utf-8"))
                self.assertEqual(Path(metadata["raw_csv"]).resolve(), recorded)
                self.assertEqual(kwargs["run_id"], source_pilot.name)
                metadata_calls.append((path, recorded, str(kwargs["mode"])))
                return {
                    "adapter": {
                        "name": "NVIDIA GeForce RTX 5090",
                        "vendor": 0x10DE,
                        "device": 0x2B85,
                        "backend": "Dx12",
                    }
                }

            def fake_verifier(
                _verifier: Path,
                _run_dir: Path,
                package: Path,
                _sidecar: Path,
                _source_root: Path,
            ) -> Path:
                result = package.parent / "G8C_MATRIX_VERIFICATION.json"
                matrix.write_new_json(result, {"verified": True})
                return result

            forbidden = AssertionError("replay attempted to launch or build")
            replay_id = "g8c-aggregation-replay-fixture"
            with (
                mock.patch.object(matrix, "REPLAY_SOURCE_PILOT_ID", source_pilot.name),
                mock.patch.object(matrix, "REPLAY_SOURCE_PILOT_PATH", source_pilot),
                mock.patch.object(
                    matrix, "aggregation_replay_run_id", return_value=replay_id
                ),
                mock.patch.object(
                    matrix, "aggregate_headless", return_value=headless_result
                ),
                mock.patch.object(
                    matrix, "aggregate_coexistence", return_value=coexistence_result
                ),
                mock.patch.object(
                    matrix, "aggregate_render_profile", return_value=render_result
                ),
                mock.patch.object(
                    matrix, "validate_worker_metadata", side_effect=validate_metadata
                ),
                mock.patch.object(matrix, "validate_csv_identity"),
                mock.patch.object(
                    matrix,
                    "run_independent_verifier_in_process",
                    side_effect=fake_verifier,
                ),
                mock.patch.object(matrix, "build_and_freeze", side_effect=forbidden),
                mock.patch.object(matrix, "run_logged", side_effect=forbidden),
                mock.patch.object(matrix.subprocess, "run", side_effect=forbidden),
            ):
                result = matrix.run_aggregation_replay(
                    Path(matrix.__file__).resolve().parents[1],
                    artifact_root,
                    source_pilot,
                )
            self.assertEqual(len(metadata_calls), 10)
            self.assertEqual(result["measurement_subprocess_count"], 0)
            self.assertEqual(result["launched_process_count"], 0)
            self.assertTrue(Path(result["receipt"]).is_file())
            self.assertTrue(Path(result["package"]).is_file())
            self.assertTrue(Path(result["verification"]).is_file())
            outer_manifest = json.loads(
                (Path(result["run_dir"]) / "G8C_MATRIX_MANIFEST.json").read_text()
            )
            self.assertEqual(outer_manifest["command_record_paths"], [])
            self.assertIsNone(outer_manifest["build_command_record"])
            replay = outer_manifest["aggregation_replay"]
            self.assertEqual(replay["source_pilot_id"], source_pilot.name)
            self.assertEqual(replay["source_pilot_command_record_paths"].__len__(), 16)
            self.assertEqual(
                replay["source_pilot_command_record_paths"][0],
                "build/COMMAND.json",
            )
            self.assertTrue(
                all(
                    not path.startswith("source-pilot/")
                    for path in replay["source_pilot_command_record_paths"]
                )
            )
            self.assertEqual(replay["launched_process_count"], 0)
            self.assertEqual(
                replay["replay_implementation"]["coordinator"]["path"],
                "verification/frozen-coordinator.py",
            )
            with zipfile.ZipFile(result["package"]) as archive:
                self.assertIn(
                    f"{replay_id}/G8C_MATRIX_RECEIPT.json", archive.namelist()
                )


if __name__ == "__main__":
    unittest.main()
