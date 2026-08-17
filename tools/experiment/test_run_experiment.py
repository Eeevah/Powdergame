from __future__ import annotations

import copy
import json
import tempfile
import tomllib
import unittest
import zipfile
from contextlib import ExitStack
from pathlib import Path
from unittest import mock

from tools.experiment import run_experiment as experiment


class ExperimentRunnerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.source = self.base / "source"
        self.source.mkdir()
        self.artifacts = self.base / "artifacts"

        self.contract_patches = ExitStack()
        self.addCleanup(self.contract_patches.close)
        for name, value in {
            "RENDERER_WIDTH": 32,
            "RENDERER_HEIGHT": 24,
            "CROP_X": 4,
            "CROP_Y": 2,
            "CROP_WIDTH": 16,
            "CROP_HEIGHT": 16,
        }.items():
            self.contract_patches.enter_context(mock.patch.object(experiment, name, value))

    def manifest_data(self, run_dir: Path) -> experiment.ManifestData:
        binary = self.source / "target" / "release" / "powdergame-windows.exe"
        binary_sha256 = "b" * 64
        return experiment.ManifestData(
            run_id=run_dir.name,
            created_utc="2026-08-17T06:00:00.000000Z",
            source=experiment.SourceInfo(
                root=self.source.resolve(),
                branch="feature/g8b-experiment-harness-v0",
                sha="a" * 40,
            ),
            binary_path=binary.resolve(),
            binary_sha256=binary_sha256,
            artifact_root=self.artifacts.resolve(),
            run_dir=run_dir.resolve(),
            build_command=(
                "cargo",
                "build",
                "--locked",
                "--release",
                "-p",
                "powdergame-windows",
            ),
            worker_command=experiment.worker_command(
                binary.resolve(), run_dir.resolve(), run_dir.name, binary_sha256
            ),
        )

    def create_manifest(self, run_id: str = "g8b-sand-fall-v0-test-run") -> tuple[Path, dict]:
        run_dir = experiment.create_run_directory(self.artifacts, run_id)
        manifest_path = run_dir / "EXPERIMENT_MANIFEST.toml"
        experiment.write_new_text(
            manifest_path, experiment.render_manifest(self.manifest_data(run_dir))
        )
        return run_dir, experiment.read_and_validate_manifest(manifest_path)

    def create_valid_worker_fixture(self, run_id: str = "g8b-sand-fall-v0-test-run") -> Path:
        run_dir, manifest = self.create_manifest(run_id)
        logs = run_dir / "logs"
        logs.mkdir()
        (logs / "build.stdout.log").write_bytes(b"build stdout\r\n")
        (logs / "build.stderr.log").write_bytes(b"")
        (run_dir / "stdout.log").write_bytes(b"worker stdout\r\n")
        (run_dir / "stderr.log").write_bytes(b"")

        telemetry = run_dir / "telemetry"
        frames_dir = run_dir / "work" / "frames"
        telemetry.mkdir()
        frames_dir.mkdir(parents=True)

        specs = [
            (0, "initial", "tick0", 10, 2, 16, 0, 20, "fnv1a64:0000000000000000"),
            (1, "settling", "tick1", 20, 3, 16, 0, 22, "fnv1a64:0000000000000001"),
            (2, "settling", "early-settling", 12, 2, 14, 2, 30, "fnv1a64:0000000000000002"),
            (8, "settling", "diagnostic-cadence", 0, 0, 0, 16, 35, "fnv1a64:0000000000000003"),
            (16, "settling", "diagnostic-cadence", 0, 0, 0, 16, 35, "fnv1a64:0000000000000003"),
            (24, "settling", "diagnostic-cadence", 0, 0, 0, 16, 35, "fnv1a64:0000000000000003"),
        ]
        specs.extend(
            (
                tick,
                "post-sleep-confirmation",
                "post-sleep-tick",
                0,
                0,
                0,
                16,
                35,
                "fnv1a64:0000000000000003",
            )
            for tick in range(25, 205)
        )
        specs.append(
            (0, "reset", "programmatic-r-equivalent", 10, 2, 16, 0, 20, "fnv1a64:0000000000000000")
        )
        samples = []
        for index, (
            tick,
            phase,
            reason,
            any_active,
            active_chunks,
            runnable_chunks,
            sleeping_chunks,
            sand_y_sum,
            state_hash,
        ) in enumerate(specs):
            samples.append(
                {
                    "schema_version": experiment.TELEMETRY_SCHEMA,
                    "experiment_id": experiment.EXPERIMENT_ID,
                    "run_id": run_dir.name,
                    "source_sha": manifest["source"]["sha"],
                    "git_state": "clean",
                    "build_profile": "release",
                    "binary_sha256": manifest["binary"]["sha256"],
                    "sample_sequence": index,
                    "sim_tick": tick,
                    "phase": phase,
                    "reason": reason,
                    "world": manifest["world"],
                    "sleep": {"enabled": True, "threshold": 3},
                    "census": {
                        "total_cells": experiment.WORLD_WIDTH * experiment.WORLD_HEIGHT,
                        "any_active_cells": any_active,
                        "matter_active_cells": any_active,
                        "thermal_active_cells": 0,
                        "pressure_active_cells": 0,
                        "reaction_active_cells": 0,
                        "total_chunks": 16,
                        "active_chunks": active_chunks,
                        "runnable_chunks": runnable_chunks,
                        "sleeping_chunks": sleeping_chunks,
                    },
                    "material_counts_by_id": [65526, 0, 0, 10, 0, 0, 0, 0, 0, 0],
                    "matter_count": 10,
                    "sand_count": 10,
                    "sand_y_sum": sand_y_sum,
                    "sand_min_y": 2,
                    "sand_max_y": 8,
                    "invalid_material_count": 0,
                    "nonfinite_temperature_count": 0,
                    "nonfinite_pressure_count": 0,
                    "changed_chunks": active_chunks,
                    "wake_chunks": 0,
                    "wake_reason_or": 0,
                    "state_hash": state_hash,
                }
            )
        frames = []
        raw_size = experiment.RENDERER_WIDTH * experiment.RENDERER_HEIGHT * 4
        for ordinal, sample_index in enumerate((0, 1, 2, 3, len(samples) - 2, len(samples) - 1)):
            sample = samples[sample_index]
            filename = f"{ordinal:02}-fixture.rgba"
            color = bytes(
                (
                    (ordinal * 31) % 256,
                    (ordinal * 47) % 256,
                    (ordinal * 67) % 256,
                    255,
                )
            )
            (frames_dir / filename).write_bytes(color * (raw_size // 4))
            frames.append(
                {
                    "ordinal": ordinal,
                    "kind": "fixture",
                    "relative_path": f"work/frames/{filename}",
                    "width": experiment.RENDERER_WIDTH,
                    "height": experiment.RENDERER_HEIGHT,
                    "rgba_bytes": raw_size,
                    "reason": sample["reason"],
                    "sim_tick": sample["sim_tick"],
                    "sample_sequence": sample["sample_sequence"],
                    "state_hash": sample["state_hash"],
                }
            )

        sample_text = "".join(json.dumps(item, separators=(",", ":")) + "\n" for item in samples)
        (telemetry / "samples.jsonl").write_text(sample_text, encoding="utf-8")
        events = [
            {
                "schema_version": experiment.TELEMETRY_SCHEMA,
                "experiment_id": experiment.EXPERIMENT_ID,
                "run_id": run_dir.name,
                "event_sequence": 0,
                "event": "lifecycle_started",
                "sim_tick": 0,
                "sample_sequence": None,
                "detail": "worker output opened",
            },
            {
                "schema_version": experiment.TELEMETRY_SCHEMA,
                "experiment_id": experiment.EXPERIMENT_ID,
                "run_id": run_dir.name,
                "event_sequence": 1,
                "event": "worker_completed",
                "sim_tick": 0,
                "sample_sequence": len(samples) - 1,
                "detail": "PASS",
            },
        ]
        event_text = "".join(json.dumps(item, separators=(",", ":")) + "\n" for item in events)
        (telemetry / "events.jsonl").write_text(event_text, encoding="utf-8")

        frames_doc = {
            "schema_version": experiment.FRAMES_SCHEMA,
            "experiment_id": experiment.EXPERIMENT_ID,
            "run_id": run_dir.name,
            "scenario": experiment.SCENARIO,
            "binary_sha256": manifest["binary"]["sha256"],
            "frame_count": len(frames),
            "pixel_encoding": "rgba8-tightly-packed",
            "frames": frames,
        }
        (run_dir / "work" / "frames.json").write_text(
            json.dumps(frames_doc), encoding="utf-8"
        )
        predicates = {
            name: {"status": "pass", "detail": f"fixture {name}"}
            for name in experiment.PREDICATE_NAMES
        }
        analysis = {
            "schema_version": experiment.ANALYSIS_SCHEMA,
            "experiment_id": experiment.EXPERIMENT_ID,
            "run_id": run_dir.name,
            "scenario": experiment.SCENARIO,
            "binary_sha256": manifest["binary"]["sha256"],
            "provenance": {
                "source_sha": manifest["source"]["sha"],
                "git_state": "clean",
                "build_profile": "release",
            },
            "world": manifest["world"],
            "sleep": {"enabled": True, "threshold": 3},
            "lifecycle": {
                "max_ticks": experiment.MAX_TICKS,
                "diagnostic_interval_ticks": experiment.DIAGNOSTIC_INTERVAL,
                "all_sleep_consecutive_samples": experiment.CONSECUTIVE_ALL_SLEEP,
                "post_sleep_confirmation_ticks": experiment.POST_SLEEP_TICKS,
                "first_all_sleep_sim_tick": 8,
                "first_all_sleep_diagnostic_sample_tick": 3,
                "first_all_sleep_sample_sequence": 3,
                "confirmed_all_sleep_sim_tick": 24,
                "post_sleep_end_tick": 204,
                "post_sleep_change_ticks": 0,
                "post_sleep_wake_ticks": 0,
                "sample_count": len(samples),
            },
            "baseline": {"matter_count": 10, "sand_count": 10, "sand_y_sum": 20},
            "metrics": {
                "peak_active_cells": 20,
                "peak_active_chunks": 3,
                "first_sleeping_chunk_tick": 2,
                "first_all_sleep_tick": 8,
                "settling_duration": 8,
                "post_sleep_state_changes": 0,
                "post_sleep_spontaneous_wakes": 0,
                "final_sleeping_chunks": 16,
                "matter_count_delta": 0,
                "reset_exact_equivalence": True,
            },
            "predicates": predicates,
            "verdict": "PASS",
            "raw_frame_count": len(frames),
        }
        (run_dir / "work" / "analysis.json").write_text(
            json.dumps(analysis), encoding="utf-8"
        )
        return run_dir

    def test_manifest_is_strict_and_round_trips(self) -> None:
        run_dir, manifest = self.create_manifest()
        self.assertEqual(set(manifest), experiment.MANIFEST_TOP_KEYS)
        self.assertEqual(manifest["source"]["sha"], "a" * 40)
        self.assertEqual(manifest["artifact"]["run_dir"], str(run_dir.resolve()))

        unexpected = copy.deepcopy(manifest)
        unexpected["unexpected"] = "not allowed"
        with self.assertRaises(experiment.ExperimentError):
            experiment.validate_manifest_dict(unexpected)
        missing = copy.deepcopy(manifest)
        del missing["renderer"]["crop_x"]
        with self.assertRaises(experiment.ExperimentError):
            experiment.validate_manifest_dict(missing)
        unlocked = copy.deepcopy(manifest)
        unlocked["commands"]["build"].remove("--locked")
        with self.assertRaisesRegex(experiment.ExperimentError, "build command mismatch"):
            experiment.validate_manifest_dict(unlocked)
        wrong_run_dir = copy.deepcopy(manifest)
        wrong_run_dir["artifact"]["run_dir"] = str(self.artifacts / "different")
        with self.assertRaisesRegex(experiment.ExperimentError, "artifact_root/run_id"):
            experiment.validate_manifest_dict(wrong_run_dir)

    def test_git_queries_scope_safe_directory_to_the_exact_source_root(self) -> None:
        completed = mock.Mock(returncode=0, stdout=b"feature/test\n", stderr=b"")
        with mock.patch.object(experiment.subprocess, "run", return_value=completed) as run:
            value = experiment.git_text(self.source, "branch", "--show-current")

        self.assertEqual(value, "feature/test")
        run.assert_called_once_with(
            [
                "git",
                "-c",
                f"safe.directory={self.source.resolve()}",
                "branch",
                "--show-current",
            ],
            cwd=self.source,
            stdout=experiment.subprocess.PIPE,
            stderr=experiment.subprocess.PIPE,
            check=False,
        )

    def test_external_root_and_create_new_contracts(self) -> None:
        self.assertEqual(
            str(experiment.DEFAULT_ARTIFACT_ROOT),
            r"C:\Users\mdkap\source\Powdergame-artifacts",
        )
        with self.assertRaises(experiment.ExperimentError):
            experiment.validate_external_artifact_root(self.source, self.source / "artifacts")
        experiment.validate_external_artifact_root(self.source, self.artifacts)

        run_dir = experiment.create_run_directory(self.artifacts, "unique-run")
        with self.assertRaises(experiment.ExperimentError):
            experiment.create_run_directory(self.artifacts, "unique-run")
        output = run_dir / "one.txt"
        experiment.write_new_text(output, "one")
        with self.assertRaises(experiment.ExperimentError):
            experiment.write_new_text(output, "two")

    def test_worker_command_and_scenario_rejection_are_exact(self) -> None:
        binary = Path(r"C:\source\powdergame-windows.exe")
        run_dir = Path(r"C:\artifacts\run")
        command = experiment.worker_command(binary, run_dir, "run-1", "c" * 64)
        self.assertEqual(
            command,
            (
                str(binary),
                "--experiment-worker",
                "sand-fall",
                "--experiment-run-dir",
                str(run_dir),
                "--experiment-run-id",
                "run-1",
                "--binary-sha256",
                "c" * 64,
                "--max-ticks",
                "20000",
                "--diagnostic-interval",
                "8",
                "--consecutive-all-sleep",
                "3",
                "--post-sleep-ticks",
                "180",
            ),
        )
        with self.assertRaises(experiment.ExperimentError):
            experiment.run_experiment(self.source, self.artifacts, "water-flow")

    def test_screenshot_name_contains_tick_sample_and_reason(self) -> None:
        frame = {
            "ordinal": 7,
            "sim_tick": 123,
            "sample_sequence": 9,
            "reason": "First All Sleep!",
        }
        self.assertEqual(
            experiment.screenshot_name(frame),
            "frame-007_sim-000123_sample-000009_first-all-sleep.png",
        )
        self.assertEqual(
            experiment.screenshot_name(frame, crop=True),
            "frame-007_sim-000123_sample-000009_first-all-sleep_crop.png",
        )

    def test_telemetry_validation_binds_provenance_and_frame_samples(self) -> None:
        run_dir = self.create_valid_worker_fixture()
        manifest = experiment.read_and_validate_manifest(run_dir / "EXPERIMENT_MANIFEST.toml")
        analysis, frames_doc, samples, events = experiment.validate_telemetry(run_dir, manifest)
        self.assertEqual(analysis["raw_frame_count"], 6)
        self.assertEqual(frames_doc["frame_count"], 6)
        self.assertEqual(len(samples), 187)
        self.assertEqual(len(events), 2)
        self.assertEqual(analysis["metrics"]["first_sleeping_chunk_tick"], 2)
        self.assertEqual(analysis["metrics"]["matter_count_delta"], 0)
        self.assertEqual(analysis["verdict"], "PASS")

        sample_path = run_dir / "telemetry" / "samples.jsonl"
        records = [json.loads(line) for line in sample_path.read_text(encoding="utf-8").splitlines()]
        records[2]["binary_sha256"] = "d" * 64
        sample_path.write_text(
            "".join(json.dumps(item) + "\n" for item in records), encoding="utf-8"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "binary_sha256 mismatch"):
            experiment.validate_telemetry(run_dir, manifest)

    def test_recomputation_rejects_inconsistent_verdict_and_post_sleep(self) -> None:
        run_dir = self.create_valid_worker_fixture("g8b-sand-fall-v0-recompute")
        manifest = experiment.read_and_validate_manifest(run_dir / "EXPERIMENT_MANIFEST.toml")
        analysis_path = run_dir / "work" / "analysis.json"
        original_analysis = json.loads(analysis_path.read_text(encoding="utf-8"))

        inconsistent = copy.deepcopy(original_analysis)
        inconsistent["verdict"] = "FAIL"
        analysis_path.write_text(json.dumps(inconsistent), encoding="utf-8")
        with self.assertRaisesRegex(experiment.ExperimentError, "seven predicate statuses"):
            experiment.validate_telemetry(run_dir, manifest)

        inconsistent = copy.deepcopy(original_analysis)
        inconsistent["lifecycle"]["first_all_sleep_sim_tick"] = 16
        inconsistent["metrics"]["first_all_sleep_tick"] = 16
        inconsistent["metrics"]["settling_duration"] = 16
        analysis_path.write_text(json.dumps(inconsistent), encoding="utf-8")
        with self.assertRaisesRegex(experiment.ExperimentError, "does not match telemetry"):
            experiment.validate_telemetry(run_dir, manifest)

        analysis_path.write_text(json.dumps(original_analysis), encoding="utf-8")
        sample_path = run_dir / "telemetry" / "samples.jsonl"
        records = [json.loads(line) for line in sample_path.read_text(encoding="utf-8").splitlines()]
        records[6]["state_hash"] = "fnv1a64:ffffffffffffffff"
        sample_path.write_text(
            "".join(json.dumps(item) + "\n" for item in records), encoding="utf-8"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "state change or wake"):
            experiment.validate_telemetry(run_dir, manifest)

    def test_contact_sheet_contains_each_crop(self) -> None:
        Image, _, _ = experiment.pillow_modules()
        first = self.base / "first.png"
        second = self.base / "second.png"
        Image.new("RGB", (20, 20), (220, 20, 30)).save(first)
        Image.new("RGB", (20, 20), (20, 80, 220)).save(second)
        screenshots = [
            {
                "ordinal": 0,
                "reason": "red",
                "sim_tick": 0,
                "sample_sequence": 0,
                "crop_png": first.relative_to(self.base).as_posix(),
            },
            {
                "ordinal": 1,
                "reason": "blue",
                "sim_tick": 1,
                "sample_sequence": 1,
                "crop_png": second.relative_to(self.base).as_posix(),
            },
        ]
        sheet_bytes = experiment.create_contact_sheet_bytes(self.base, screenshots)
        sheet_path = self.base / "sheet.png"
        sheet_path.write_bytes(sheet_bytes)
        with Image.open(sheet_path) as sheet:
            self.assertEqual(sheet.size, (1260, 450))
            self.assertEqual(sheet.getpixel((210, 200)), (220, 20, 30))
            self.assertEqual(sheet.getpixel((630, 200)), (20, 80, 220))

    def test_packet_contents_hashes_and_receipt_last(self) -> None:
        run_dir = self.create_valid_worker_fixture()
        publication_log: list[str] = []
        receipt_path = experiment.postprocess_run(run_dir, publication_log)
        self.assertEqual(publication_log[-1], "EXPERIMENT_RECEIPT.json")
        self.assertEqual(receipt_path, run_dir / "EXPERIMENT_RECEIPT.json")
        self.assertTrue((run_dir / "report" / "CHATGPT_REVIEW_PROMPT.md").is_file())
        self.assertTrue((run_dir / "report" / "CONTACT_SHEET.png").is_file())
        self.assertFalse((run_dir / "CHATGPT_REVIEW_PROMPT.md").exists())

        packet = run_dir / "report" / "REVIEW_PACKET.zip"
        with zipfile.ZipFile(packet) as archive:
            names = set(archive.namelist())
        expected = {
            "EXPERIMENT_MANIFEST.toml",
            "stdout.log",
            "stderr.log",
            "logs/build.stdout.log",
            "logs/build.stderr.log",
            "telemetry/samples.jsonl",
            "telemetry/events.jsonl",
            "report/REPORT.md",
            "report/REPORT.json",
            "report/CHATGPT_REVIEW_PROMPT.md",
            "report/CONTACT_SHEET.png",
        }
        self.assertTrue(expected.issubset(names))
        self.assertTrue(any(name.startswith("screenshots/full/") for name in names))
        self.assertTrue(any(name.startswith("screenshots/crops/") for name in names))
        self.assertFalse(any(name.startswith("work/") for name in names))
        self.assertNotIn("report/REVIEW_PACKET.zip", names)
        self.assertNotIn("HASHES.sha256", names)
        self.assertNotIn("EXPERIMENT_RECEIPT.json", names)

        hashes = (run_dir / "HASHES.sha256").read_text(encoding="utf-8")
        self.assertIn("  report/REVIEW_PACKET.zip\n", hashes)
        self.assertIn("  work/analysis.json\n", hashes)
        self.assertNotIn("EXPERIMENT_RECEIPT.json", hashes)
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        self.assertEqual(receipt["review_packet_sha256"], experiment.sha256_file(packet))
        self.assertTrue(receipt["receipt_is_final_publication_marker"])
        with self.assertRaises(experiment.ExperimentError):
            experiment.postprocess_run(run_dir)

    def test_invalid_worker_output_leaves_no_receipt(self) -> None:
        run_dir = self.create_valid_worker_fixture()
        analysis_path = run_dir / "work" / "analysis.json"
        analysis = json.loads(analysis_path.read_text(encoding="utf-8"))
        analysis["raw_frame_count"] = 5
        analysis_path.write_text(json.dumps(analysis), encoding="utf-8")
        with self.assertRaises(experiment.ExperimentError):
            experiment.postprocess_run(run_dir)
        self.assertFalse((run_dir / "EXPERIMENT_RECEIPT.json").exists())


if __name__ == "__main__":
    unittest.main()
