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

    def manifest_data(
        self,
        run_dir: Path,
        contract: experiment.ScenarioContract = experiment.SAND_CONTRACT,
        mode: str = "candidate",
    ) -> experiment.ManifestData:
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
                binary.resolve(),
                run_dir.resolve(),
                run_dir.name,
                binary_sha256,
                contract=contract,
                run_mode=mode,
            ),
            contract=contract,
            run_mode=mode,
        )

    def create_manifest(
        self,
        run_id: str = "g8b-sand-fall-v0-test-run",
        contract: experiment.ScenarioContract = experiment.SAND_CONTRACT,
        mode: str = "candidate",
    ) -> tuple[Path, dict]:
        run_dir = experiment.create_run_directory(self.artifacts, run_id)
        manifest_path = run_dir / "EXPERIMENT_MANIFEST.toml"
        experiment.write_new_text(
            manifest_path,
            experiment.render_manifest(self.manifest_data(run_dir, contract, mode)),
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

    def create_valid_water_worker_fixture(
        self,
        run_id: str = "g8b-water-flow-v0-test-run",
        mode: str = "candidate",
    ) -> Path:
        if mode == "scratch" and "-scratch-" not in run_id:
            run_id = run_id.replace("g8b-water-flow-v0-", "g8b-water-flow-v0-scratch-")
        run_dir, manifest = self.create_manifest(
            run_id, experiment.WATER_CONTRACT, mode
        )
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

        material_counts = [40144, 1020, 6888, 0, 15244, 2240, 0, 0, 0, 0]

        def sample(
            tick: int,
            phase: str,
            reason: str,
            *,
            active: int,
            active_chunks: int,
            runnable: int,
            sleeping: int,
            changed: int,
            state_hash: str,
            outside: int = 0,
            outside_outer_basin: int = 0,
            vacated: int = 0,
            bottom: int = 0,
            destination: int = 0,
            spread: int = 0,
            water_chunks: int = 10,
            water_y_sum: int = 1_000_000,
            active_water_empty: int | None = None,
            active_water_oil: int = 0,
            active_other: int = 0,
        ) -> dict:
            return {
                "schema_version": experiment.WATER_TELEMETRY_SCHEMA,
                "experiment_id": experiment.WATER_CONTRACT.experiment_id,
                "run_id": run_dir.name,
                "scenario": experiment.WATER_CONTRACT.scenario,
                "source_sha": manifest["source"]["sha"],
                "git_state": "clean",
                "build_profile": "release",
                "binary_sha256": manifest["binary"]["sha256"],
                "sample_sequence": -1,
                "sim_tick": tick,
                "phase": phase,
                "reason": reason,
                "world": manifest["world"],
                "sleep": {"enabled": True, "threshold": 3},
                "census": {
                    "total_cells": experiment.WORLD_WIDTH * experiment.WORLD_HEIGHT,
                    "any_active_cells": active,
                    "matter_active_cells": active,
                    "thermal_active_cells": 0,
                    "pressure_active_cells": 0,
                    "reaction_active_cells": 0,
                    "total_chunks": 16,
                    "active_chunks": active_chunks,
                    "runnable_chunks": runnable,
                    "sleeping_chunks": sleeping,
                },
                "material_counts_by_id": list(material_counts),
                "matter_count": 25392,
                "water_count": 15244,
                "oil_count": 2240,
                "water_y_sum": water_y_sum,
                "water_min_y": 8,
                "water_max_y": 220 if bottom else 188,
                "oil_y_sum": 200000,
                "oil_min_y": 48,
                "oil_max_y": 180,
                "water_occupied_chunks": water_chunks,
                "oil_occupied_chunks": 4,
                "water_outside_initial_mask": outside,
                "water_outside_outer_basin_cells": outside_outer_basin,
                "initial_water_cells_vacated": vacated,
                "bottom_chunk_row_water_cells": bottom,
                "destination_water_cells": destination,
                "destination_spread_x": spread,
                "invalid_material_count": 0,
                "nonfinite_temperature_count": 0,
                "nonfinite_pressure_count": 0,
                "changed_chunks": changed,
                "wake_chunks": 0,
                "wake_reason_or": 0,
                "state_hash": state_hash,
                "physical_state_hash": state_hash,
                "active_water_empty_surface_cells": (
                    active if active_water_empty is None else active_water_empty
                ),
                "active_water_oil_interface_cells": active_water_oil,
                "active_other_cells": active_other,
            }

        samples = [
            sample(
                0,
                "initial",
                "tick0",
                active=100,
                active_chunks=6,
                runnable=16,
                sleeping=0,
                changed=0,
                state_hash="fnv1a64:0000000000000100",
            ),
            sample(
                1,
                "flowing",
                "tick1",
                active=120,
                active_chunks=7,
                runnable=15,
                sleeping=1,
                changed=5,
                state_hash="fnv1a64:0000000000000101",
                outside=5,
                vacated=5,
                water_chunks=11,
                water_y_sum=1_000_100,
            ),
            sample(
                2,
                "flowing",
                "early-flow",
                active=110,
                active_chunks=8,
                runnable=14,
                sleeping=2,
                changed=4,
                state_hash="fnv1a64:0000000000000102",
                outside=20,
                vacated=20,
                bottom=7,
                water_chunks=12,
                water_y_sum=1_000_220,
            ),
        ]
        for tick in (8, 16, 24):
            samples.append(
                sample(
                    tick,
                    "flowing",
                    "diagnostic-cadence",
                    active=0,
                    active_chunks=0,
                    runnable=0,
                    sleeping=16,
                    changed=0,
                    state_hash="fnv1a64:0000000000000200",
                    outside=40,
                    vacated=40,
                    bottom=12,
                    destination=20,
                    spread=15,
                    water_chunks=13,
                    water_y_sum=1_000_400,
                )
            )
        for tick in range(25, 205):
            samples.append(
                sample(
                    tick,
                    "post-settle-confirmation",
                    "post-settle-tick",
                    active=0,
                    active_chunks=0,
                    runnable=0,
                    sleeping=16,
                    changed=0,
                    state_hash="fnv1a64:0000000000000200",
                    outside=40,
                    vacated=40,
                    bottom=12,
                    destination=20,
                    spread=15,
                    water_chunks=13,
                    water_y_sum=1_000_400,
                )
            )
        samples.append(copy.deepcopy(samples[0]))
        samples[-1].update(
            {
                "sim_tick": 0,
                "phase": "reset",
                "reason": "programmatic-r-equivalent",
            }
        )
        for sequence, item in enumerate(samples):
            item["sample_sequence"] = sequence

        sample_path = telemetry / "samples.jsonl"
        sample_path.write_text(
            "".join(json.dumps(item, separators=(",", ":")) + "\n" for item in samples),
            encoding="utf-8",
        )

        def event(name: str, item: dict | None, detail: str = "fixture") -> dict:
            return {
                "schema_version": experiment.WATER_TELEMETRY_SCHEMA,
                "experiment_id": experiment.WATER_CONTRACT.experiment_id,
                "run_id": run_dir.name,
                "scenario": experiment.WATER_CONTRACT.scenario,
                "event_sequence": -1,
                "event": name,
                "sim_tick": 0 if item is None else item["sim_tick"],
                "sample_sequence": None if item is None else item["sample_sequence"],
                "detail": detail,
            }

        events = [
            event("lifecycle_started", None),
            event("pristine_reset_completed", None),
            event("tick0_captured", samples[0]),
            event("tick1_captured", samples[1]),
            event("water_movement_observed", samples[1]),
            event("new_peak_active", samples[1]),
            event("cross_chunk_flow_observed", samples[2]),
            event("first_sleeping_chunk_observed", samples[1]),
            event("destination_arrival_observed", samples[3]),
            event("new_max_destination_spread", samples[3]),
            event("all_sleep_observed", samples[3]),
            event("stable_plateau_observed", samples[3]),
            event("all_sleep_confirmed", samples[5]),
            event("terminal_selected", samples[5]),
            event("post_settle_confirmation_completed", samples[-2]),
            event("reset_started", samples[-2]),
            event("reset_comparison_completed", samples[-1]),
            event("worker_completed", samples[-1], "PASS"),
        ]
        for sequence, item in enumerate(events):
            item["event_sequence"] = sequence
        (telemetry / "events.jsonl").write_text(
            "".join(json.dumps(item, separators=(",", ":")) + "\n" for item in events),
            encoding="utf-8",
        )

        frame_specs = (
            ("tick0", 0, "pristine-reset"),
            ("tick1", 1, "tick1"),
            ("cross-chunk-flow", 2, "cross-chunk-flow"),
            ("destination-arrival", 3, "destination-arrival"),
            ("late", 4, "observation-before-terminal-diagnostic"),
            ("terminal", 5, "all-sleep-terminal"),
            ("post-settle", len(samples) - 2, "post-settle"),
            ("reset", len(samples) - 1, "programmatic-reset"),
        )
        raw_size = experiment.RENDERER_WIDTH * experiment.RENDERER_HEIGHT * 4
        frames = []
        for ordinal, (kind, sample_index, reason) in enumerate(frame_specs):
            item = samples[sample_index]
            filename = f"{ordinal:02}-{kind}.rgba"
            color = bytes(((ordinal * 31) % 256, (ordinal * 47) % 256, 80, 255))
            (frames_dir / filename).write_bytes(color * (raw_size // 4))
            frames.append(
                {
                    "ordinal": ordinal,
                    "kind": kind,
                    "relative_path": f"work/frames/{filename}",
                    "width": experiment.RENDERER_WIDTH,
                    "height": experiment.RENDERER_HEIGHT,
                    "rgba_bytes": raw_size,
                    "reason": reason,
                    "sim_tick": item["sim_tick"],
                    "sample_sequence": item["sample_sequence"],
                    "state_hash": item["state_hash"],
                }
            )
        frames_doc = {
            "schema_version": experiment.FRAMES_SCHEMA,
            "experiment_id": experiment.WATER_CONTRACT.experiment_id,
            "run_id": run_dir.name,
            "scenario": experiment.WATER_CONTRACT.scenario,
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
            for name in experiment.WATER_PREDICATE_NAMES
        }
        analysis = {
            "schema_version": experiment.WATER_ANALYSIS_SCHEMA,
            "experiment_id": experiment.WATER_CONTRACT.experiment_id,
            "run_id": run_dir.name,
            "scenario": experiment.WATER_CONTRACT.scenario,
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
                "stable_plateau_consecutive_samples": experiment.CONSECUTIVE_STABLE_PLATEAU,
                "post_settle_confirmation_ticks": experiment.POST_SLEEP_TICKS,
                "terminal_reason": "all-sleep",
                "first_all_sleep_sim_tick": 8,
                "first_all_sleep_sample_sequence": 3,
                "confirmed_all_sleep_sim_tick": 24,
                "first_stable_plateau_sim_tick": None,
                "first_stable_plateau_sample_sequence": None,
                "confirmed_stable_plateau_sim_tick": None,
                "terminal_sim_tick": 24,
                "terminal_sample_sequence": 5,
                "post_settle_end_tick": 204,
                "post_settle_change_ticks": 0,
                "post_settle_wake_ticks": 0,
                "sample_count": len(samples),
            },
            "baseline": {
                "matter_count": 25392,
                "water_count": 15244,
                "oil_count": 2240,
                "water_y_sum": 1_000_000,
                "oil_y_sum": 200000,
                "water_occupied_chunks": 10,
                "oil_occupied_chunks": 4,
                "bottom_chunk_row_water_cells": 0,
                "destination_water_cells": 0,
                "destination_spread_x": 0,
            },
            "metrics": {
                "peak_active_cells": 120,
                "peak_active_chunks": 8,
                "peak_active_sim_tick": 1,
                "peak_active_sample_sequence": 1,
                "first_water_movement_tick": 1,
                "first_water_movement_sample_sequence": 1,
                "first_cross_chunk_flow_tick": 2,
                "first_cross_chunk_flow_sample_sequence": 2,
                "first_destination_arrival_tick": 8,
                "first_destination_arrival_sample_sequence": 3,
                "first_sleeping_chunk_tick": 1,
                "first_sleeping_chunk_sample_sequence": 1,
                "max_bottom_chunk_row_water_cells": 12,
                "max_destination_water_cells": 20,
                "max_destination_spread_x": 15,
                "max_destination_spread_tick": 8,
                "max_destination_spread_sample_sequence": 3,
                "max_water_outside_outer_basin_cells": 0,
                "final_matter_count": 25392,
                "final_water_count": 15244,
                "final_oil_count": 2240,
                "final_water_occupied_chunks": 13,
                "final_oil_occupied_chunks": 4,
                "final_sleeping_chunks": 16,
                "final_water_outside_outer_basin_cells": 0,
                "final_active_water_empty_surface_cells": 0,
                "final_active_water_oil_interface_cells": 0,
                "final_active_other_cells": 0,
                "active_cell_classification_rule": (
                    experiment.WATER_ACTIVE_CLASSIFICATION_RULE
                ),
                "matter_count_delta": 0,
                "water_count_delta": 0,
                "oil_count_delta": 0,
                "post_settle_state_changes": 0,
                "post_settle_spontaneous_wakes": 0,
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

    def test_water_manifest_modes_and_tri_state_verdict_are_exact(self) -> None:
        candidate_dir, candidate = self.create_manifest(
            "g8b-water-flow-v0-candidate-test",
            experiment.WATER_CONTRACT,
            "candidate",
        )
        self.assertEqual(candidate["schema_version"], experiment.WATER_MANIFEST_SCHEMA)
        self.assertEqual(candidate["run_mode"], "candidate")
        self.assertEqual(
            candidate["experiment"]["stable_plateau_consecutive_samples"], 8
        )
        self.assertEqual(candidate["commands"]["worker"][2], "water-flow")
        self.assertNotIn("--mode", candidate["commands"]["worker"])
        self.assertEqual(candidate["artifact"]["run_dir"], str(candidate_dir.resolve()))

        _, scratch = self.create_manifest(
            "g8b-water-flow-v0-scratch-test",
            experiment.WATER_CONTRACT,
            "scratch",
        )
        self.assertEqual(scratch["run_mode"], "scratch")
        self.assertIn("-scratch-", scratch["run_id"])
        mismatched = copy.deepcopy(scratch)
        mismatched["run_mode"] = "candidate"
        with self.assertRaisesRegex(experiment.ExperimentError, "scratch marker"):
            experiment.validate_manifest_dict(mismatched)
        with self.assertRaisesRegex(experiment.ExperimentError, "only candidate"):
            experiment.validate_run_mode(experiment.SAND_CONTRACT, "scratch")

        pass_predicates = {
            name: {"status": "pass", "detail": "fixture"}
            for name in experiment.WATER_PREDICATE_NAMES
        }
        self.assertEqual(
            experiment.verdict_from_predicates(
                pass_predicates, experiment.WATER_CONTRACT
            ),
            "PASS",
        )
        pass_predicates["destination_arrival"]["status"] = "unknown"
        self.assertEqual(
            experiment.verdict_from_predicates(
                pass_predicates, experiment.WATER_CONTRACT
            ),
            "NEEDS_HUMAN_REVIEW",
        )
        pass_predicates["exact_reset"]["status"] = "fail"
        self.assertEqual(
            experiment.verdict_from_predicates(
                pass_predicates, experiment.WATER_CONTRACT
            ),
            "FAIL",
        )

    def test_water_telemetry_recomputes_flow_sleep_and_reset_contract(self) -> None:
        run_dir = self.create_valid_water_worker_fixture()
        manifest = experiment.read_and_validate_manifest(
            run_dir / "EXPERIMENT_MANIFEST.toml"
        )
        analysis, frames, samples, events = experiment.validate_telemetry(
            run_dir, manifest
        )
        self.assertEqual(analysis["verdict"], "PASS")
        self.assertEqual(analysis["metrics"]["first_water_movement_tick"], 1)
        self.assertEqual(analysis["metrics"]["first_cross_chunk_flow_tick"], 2)
        self.assertEqual(analysis["metrics"]["first_destination_arrival_tick"], 8)
        self.assertEqual(analysis["lifecycle"]["first_all_sleep_sim_tick"], 8)
        self.assertEqual(analysis["lifecycle"]["confirmed_all_sleep_sim_tick"], 24)
        self.assertEqual(analysis["lifecycle"]["post_settle_end_tick"], 204)
        self.assertEqual(len(samples), 187)
        self.assertEqual(frames["frame_count"], 8)
        self.assertEqual(len(events), 18)
        self.assertEqual(
            analysis["schema_version"], "powdergame-experiment-analysis-v2"
        )
        self.assertEqual(
            samples[0]["schema_version"], "powdergame-experiment-telemetry-v2"
        )
        self.assertEqual(
            analysis["metrics"]["max_water_outside_outer_basin_cells"], 0
        )
        self.assertEqual(
            analysis["predicates"]["water_outside_outer_basin_cells"]["status"],
            "pass",
        )

    def test_water_outer_basin_hard_predicate_recomputes_zero_and_leak(self) -> None:
        zero_dir = self.create_valid_water_worker_fixture(
            "g8b-water-flow-v0-outer-zero-test"
        )
        zero_manifest = experiment.read_and_validate_manifest(
            zero_dir / "EXPERIMENT_MANIFEST.toml"
        )
        zero_analysis, _, _, _ = experiment.validate_telemetry(
            zero_dir, zero_manifest
        )
        self.assertEqual(
            zero_analysis["predicates"]["water_outside_outer_basin_cells"]["status"],
            "pass",
        )

        leak_dir = self.create_valid_water_worker_fixture(
            "g8b-water-flow-v0-outer-leak-test"
        )
        sample_path = leak_dir / "telemetry" / "samples.jsonl"
        samples = [
            json.loads(line)
            for line in sample_path.read_text(encoding="utf-8").splitlines()
        ]
        samples[2]["water_outside_outer_basin_cells"] = 3
        sample_path.write_text(
            "".join(json.dumps(item) + "\n" for item in samples), encoding="utf-8"
        )
        analysis_path = leak_dir / "work" / "analysis.json"
        analysis = json.loads(analysis_path.read_text(encoding="utf-8"))
        analysis["metrics"]["max_water_outside_outer_basin_cells"] = 3
        analysis["predicates"]["water_outside_outer_basin_cells"] = {
            "status": "fail",
            "detail": "fixture observed three Water cells outside the outer basin",
        }
        analysis["verdict"] = "FAIL"
        analysis_path.write_text(json.dumps(analysis), encoding="utf-8")
        leak_manifest = experiment.read_and_validate_manifest(
            leak_dir / "EXPERIMENT_MANIFEST.toml"
        )
        leak_analysis, _, _, _ = experiment.validate_telemetry(
            leak_dir, leak_manifest
        )
        self.assertEqual(
            leak_analysis["predicates"]["water_outside_outer_basin_cells"]["status"],
            "fail",
        )
        self.assertEqual(leak_analysis["verdict"], "FAIL")

    def test_water_outer_basin_analysis_max_and_final_are_bound(self) -> None:
        for key in (
            "max_water_outside_outer_basin_cells",
            "final_water_outside_outer_basin_cells",
        ):
            with self.subTest(metric=key):
                run_dir = self.create_valid_water_worker_fixture(
                    f"g8b-water-flow-v0-{key}-test"
                )
                analysis_path = run_dir / "work" / "analysis.json"
                analysis = json.loads(analysis_path.read_text(encoding="utf-8"))
                analysis["metrics"][key] = 1
                analysis_path.write_text(json.dumps(analysis), encoding="utf-8")
                manifest = experiment.read_and_validate_manifest(
                    run_dir / "EXPERIMENT_MANIFEST.toml"
                )
                with self.assertRaisesRegex(experiment.ExperimentError, key):
                    experiment.validate_telemetry(run_dir, manifest)

    def test_water_max_ticks_final_metrics_exclude_reset_sample(self) -> None:
        run_dir = self.create_valid_water_worker_fixture(
            "g8b-water-flow-v0-max-ticks-final-test"
        )
        manifest = experiment.read_and_validate_manifest(
            run_dir / "EXPERIMENT_MANIFEST.toml"
        )
        sample_path = run_dir / "telemetry" / "samples.jsonl"
        original_samples = [
            json.loads(line)
            for line in sample_path.read_text(encoding="utf-8").splitlines()
        ]
        tick0 = copy.deepcopy(original_samples[0])
        tick1 = copy.deepcopy(original_samples[1])
        tick2 = copy.deepcopy(original_samples[2])
        diagnostic_template = copy.deepcopy(original_samples[3])
        samples = [tick0, tick1, tick2]
        for tick in range(8, experiment.MAX_TICKS + 1, experiment.DIAGNOSTIC_INTERVAL):
            item = copy.deepcopy(diagnostic_template)
            item.update(
                {
                    "sim_tick": tick,
                    "phase": "flowing",
                    "reason": (
                        "max-tick"
                        if tick == experiment.MAX_TICKS
                        else "diagnostic-cadence"
                    ),
                    "water_outside_outer_basin_cells": (
                        7 if tick == experiment.MAX_TICKS else 0
                    ),
                    "active_water_empty_surface_cells": 6,
                    "active_water_oil_interface_cells": 4,
                    "active_other_cells": 2,
                    "changed_chunks": 1,
                    "state_hash": f"fnv1a64:{tick:016x}",
                    "physical_state_hash": f"fnv1a64:{tick:016x}",
                }
            )
            item["census"].update(
                {
                    "any_active_cells": 12,
                    "matter_active_cells": 12,
                    "active_chunks": 2,
                    "runnable_chunks": 15,
                    "sleeping_chunks": 1,
                }
            )
            samples.append(item)

        terminal = samples[-1]
        reset = copy.deepcopy(tick0)
        reset.update(
            {
                "sim_tick": 0,
                "phase": "reset",
                "reason": "programmatic-r-equivalent",
                "water_outside_outer_basin_cells": 999,
                "active_water_empty_surface_cells": 80,
                "active_water_oil_interface_cells": 15,
                "active_other_cells": 5,
                "state_hash": "fnv1a64:fffffffffffffffe",
                "physical_state_hash": "fnv1a64:fffffffffffffffe",
            }
        )
        samples.append(reset)
        for sequence, item in enumerate(samples):
            item["sample_sequence"] = sequence
        sample_path.write_text(
            "".join(json.dumps(item, separators=(",", ":")) + "\n" for item in samples),
            encoding="utf-8",
        )

        tick8 = samples[3]

        def event(name: str, item: dict | None, detail: str = "fixture") -> dict:
            return {
                "schema_version": experiment.WATER_TELEMETRY_SCHEMA,
                "experiment_id": experiment.WATER_CONTRACT.experiment_id,
                "run_id": run_dir.name,
                "scenario": experiment.WATER_CONTRACT.scenario,
                "event_sequence": -1,
                "event": name,
                "sim_tick": 0 if item is None else item["sim_tick"],
                "sample_sequence": None if item is None else item["sample_sequence"],
                "detail": detail,
            }

        events = [
            event("lifecycle_started", None),
            event("pristine_reset_completed", None),
            event("tick0_captured", tick0),
            event("tick1_captured", tick1),
            event("water_movement_observed", tick1),
            event("new_peak_active", tick1),
            event("cross_chunk_flow_observed", tick2),
            event("first_sleeping_chunk_observed", tick1),
            event("destination_arrival_observed", tick8),
            event("new_max_destination_spread", tick8),
            event("terminal_selected", terminal),
            event("reset_started", terminal),
            event("reset_comparison_completed", reset),
            event("worker_completed", reset, "FAIL"),
        ]
        for sequence, item in enumerate(events):
            item["event_sequence"] = sequence
        (run_dir / "telemetry" / "events.jsonl").write_text(
            "".join(json.dumps(item, separators=(",", ":")) + "\n" for item in events),
            encoding="utf-8",
        )

        frames_dir = run_dir / "work" / "frames"
        peak_path = frames_dir / "peak-alias.rgba"
        peak_path.write_bytes((frames_dir / "01-tick1.rgba").read_bytes())
        frame_specs = (
            ("tick0", tick0, "pristine-reset", "work/frames/00-tick0.rgba"),
            ("tick1", tick1, "tick1", "work/frames/01-tick1.rgba"),
            (
                "cross-chunk-flow",
                tick2,
                "cross-chunk-flow",
                "work/frames/02-cross-chunk-flow.rgba",
            ),
            (
                "destination-arrival",
                tick8,
                "destination-arrival",
                "work/frames/03-destination-arrival.rgba",
            ),
            (
                "peak-active",
                tick1,
                "highest-observed-active-cells",
                "work/frames/peak-alias.rgba",
            ),
            (
                "late",
                samples[-3],
                "observation-before-terminal-diagnostic",
                "work/frames/04-late.rgba",
            ),
            (
                "terminal",
                terminal,
                "max-tick-reached",
                "work/frames/05-terminal.rgba",
            ),
            (
                "reset",
                reset,
                "programmatic-reset",
                "work/frames/07-reset.rgba",
            ),
        )
        raw_size = experiment.RENDERER_WIDTH * experiment.RENDERER_HEIGHT * 4
        frames = [
            {
                "ordinal": ordinal,
                "kind": kind,
                "relative_path": relative_path,
                "width": experiment.RENDERER_WIDTH,
                "height": experiment.RENDERER_HEIGHT,
                "rgba_bytes": raw_size,
                "reason": reason,
                "sim_tick": item["sim_tick"],
                "sample_sequence": item["sample_sequence"],
                "state_hash": item["state_hash"],
            }
            for ordinal, (kind, item, reason, relative_path) in enumerate(frame_specs)
        ]
        frames_path = run_dir / "work" / "frames.json"
        frames_doc = json.loads(frames_path.read_text(encoding="utf-8"))
        frames_doc["frame_count"] = len(frames)
        frames_doc["frames"] = frames
        frames_path.write_text(json.dumps(frames_doc), encoding="utf-8")

        analysis_path = run_dir / "work" / "analysis.json"
        analysis = json.loads(analysis_path.read_text(encoding="utf-8"))
        analysis["lifecycle"].update(
            {
                "terminal_reason": "max-ticks",
                "first_all_sleep_sim_tick": None,
                "first_all_sleep_sample_sequence": None,
                "confirmed_all_sleep_sim_tick": None,
                "first_stable_plateau_sim_tick": None,
                "first_stable_plateau_sample_sequence": None,
                "confirmed_stable_plateau_sim_tick": None,
                "terminal_sim_tick": experiment.MAX_TICKS,
                "terminal_sample_sequence": terminal["sample_sequence"],
                "post_settle_end_tick": None,
                "post_settle_change_ticks": 0,
                "post_settle_wake_ticks": 0,
                "sample_count": len(samples),
            }
        )
        analysis["metrics"].update(
            {
                "max_water_outside_outer_basin_cells": 7,
                "final_water_outside_outer_basin_cells": 7,
                "final_active_water_empty_surface_cells": 6,
                "final_active_water_oil_interface_cells": 4,
                "final_active_other_cells": 2,
                "final_sleeping_chunks": 1,
                "post_settle_state_changes": 0,
                "post_settle_spontaneous_wakes": 0,
                "reset_exact_equivalence": False,
            }
        )
        predicate_statuses = {
            name: "pass" for name in experiment.WATER_PREDICATE_NAMES
        }
        predicate_statuses.update(
            {
                "water_outside_outer_basin_cells": "fail",
                "stable_bulk_before_max": "unknown",
                "post_settle_stable": "unknown",
                "exact_reset": "fail",
            }
        )
        analysis["predicates"] = {
            name: {"status": status, "detail": f"fixture {name} {status}"}
            for name, status in predicate_statuses.items()
        }
        analysis["verdict"] = "FAIL"
        analysis["raw_frame_count"] = len(frames)
        analysis_path.write_text(json.dumps(analysis), encoding="utf-8")

        validated, _, validated_samples, _ = experiment.validate_water_telemetry(
            run_dir, manifest
        )
        final_non_reset = validated_samples[-2]
        reset_sample = validated_samples[-1]
        metrics = validated["metrics"]
        self.assertEqual(validated["lifecycle"]["terminal_reason"], "max-ticks")
        self.assertEqual(final_non_reset["sim_tick"], experiment.MAX_TICKS)
        self.assertEqual(metrics["max_water_outside_outer_basin_cells"], 7)
        self.assertEqual(
            metrics["final_water_outside_outer_basin_cells"],
            final_non_reset["water_outside_outer_basin_cells"],
        )
        self.assertEqual(
            (
                metrics["final_active_water_empty_surface_cells"],
                metrics["final_active_water_oil_interface_cells"],
                metrics["final_active_other_cells"],
            ),
            (
                final_non_reset["active_water_empty_surface_cells"],
                final_non_reset["active_water_oil_interface_cells"],
                final_non_reset["active_other_cells"],
            ),
        )
        self.assertEqual(reset_sample["water_outside_outer_basin_cells"], 999)
        self.assertNotEqual(
            metrics["max_water_outside_outer_basin_cells"],
            reset_sample["water_outside_outer_basin_cells"],
        )
        self.assertNotEqual(
            metrics["final_active_other_cells"], reset_sample["active_other_cells"]
        )

    def test_water_final_active_classification_is_exactly_bound(self) -> None:
        run_dir = self.create_valid_water_worker_fixture(
            "g8b-water-flow-v0-active-class-test"
        )
        sample_path = run_dir / "telemetry" / "samples.jsonl"
        samples = [
            json.loads(line)
            for line in sample_path.read_text(encoding="utf-8").splitlines()
        ]
        samples[-2]["active_other_cells"] = 1
        sample_path.write_text(
            "".join(json.dumps(item) + "\n" for item in samples), encoding="utf-8"
        )
        manifest = experiment.read_and_validate_manifest(
            run_dir / "EXPERIMENT_MANIFEST.toml"
        )
        with self.assertRaisesRegex(
            experiment.ExperimentError, "active-cell classifications"
        ):
            experiment.validate_telemetry(run_dir, manifest)

        metric_dir = self.create_valid_water_worker_fixture(
            "g8b-water-flow-v0-active-class-metric-test"
        )
        analysis_path = metric_dir / "work" / "analysis.json"
        analysis = json.loads(analysis_path.read_text(encoding="utf-8"))
        analysis["metrics"]["final_active_other_cells"] = 1
        analysis_path.write_text(json.dumps(analysis), encoding="utf-8")
        metric_manifest = experiment.read_and_validate_manifest(
            metric_dir / "EXPERIMENT_MANIFEST.toml"
        )
        with self.assertRaisesRegex(
            experiment.ExperimentError, "final_active_other_cells"
        ):
            experiment.validate_telemetry(metric_dir, metric_manifest)

    def test_water_named_peak_frame_may_share_the_tick1_identity(self) -> None:
        run_dir = self.create_valid_water_worker_fixture(
            "g8b-water-flow-v0-peak-alias-test"
        )
        frames_path = run_dir / "work" / "frames.json"
        frames_doc = json.loads(frames_path.read_text(encoding="utf-8"))
        tick1 = next(frame for frame in frames_doc["frames"] if frame["kind"] == "tick1")
        peak = copy.deepcopy(tick1)
        peak["ordinal"] = len(frames_doc["frames"])
        peak["kind"] = "peak-active"
        peak["reason"] = "highest-observed-active-cells"
        peak["relative_path"] = "work/frames/peak-alias.rgba"
        source_raw = run_dir / Path(*Path(tick1["relative_path"]).parts)
        (run_dir / "work" / "frames" / "peak-alias.rgba").write_bytes(
            source_raw.read_bytes()
        )
        frames_doc["frames"].append(peak)
        frames_doc["frame_count"] += 1
        frames_path.write_text(json.dumps(frames_doc), encoding="utf-8")
        analysis_path = run_dir / "work" / "analysis.json"
        analysis = json.loads(analysis_path.read_text(encoding="utf-8"))
        analysis["raw_frame_count"] += 1
        analysis_path.write_text(json.dumps(analysis), encoding="utf-8")

        manifest = experiment.read_and_validate_manifest(
            run_dir / "EXPERIMENT_MANIFEST.toml"
        )
        experiment.validate_telemetry(run_dir, manifest)

    def test_water_conservation_is_recomputed_from_every_non_reset_sample(self) -> None:
        run_dir = self.create_valid_water_worker_fixture(
            "g8b-water-flow-v0-conservation-test"
        )
        sample_path = run_dir / "telemetry" / "samples.jsonl"
        samples = [
            json.loads(line)
            for line in sample_path.read_text(encoding="utf-8").splitlines()
        ]
        samples[10]["material_counts_by_id"][0] += 1
        samples[10]["material_counts_by_id"][4] -= 1
        samples[10]["matter_count"] -= 1
        samples[10]["water_count"] -= 1
        sample_path.write_text(
            "".join(json.dumps(item) + "\n" for item in samples), encoding="utf-8"
        )
        manifest = experiment.read_and_validate_manifest(
            run_dir / "EXPERIMENT_MANIFEST.toml"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "water_conservation"):
            experiment.validate_telemetry(run_dir, manifest)

    def test_water_destination_and_cross_chunk_signals_are_not_inferred(self) -> None:
        for label, fields, expected in (
            (
                "cross",
                {"bottom_chunk_row_water_cells": 0},
                "first_cross_chunk_flow",
            ),
            (
                "destination",
                {"destination_water_cells": 0, "destination_spread_x": 0},
                "first_destination_arrival",
            ),
        ):
            with self.subTest(signal=label):
                run_dir = self.create_valid_water_worker_fixture(
                    f"g8b-water-flow-v0-{label}-test"
                )
                sample_path = run_dir / "telemetry" / "samples.jsonl"
                samples = [
                    json.loads(line)
                    for line in sample_path.read_text(encoding="utf-8").splitlines()
                ]
                for item in samples:
                    if item["phase"] == "flowing":
                        item.update(fields)
                sample_path.write_text(
                    "".join(json.dumps(item) + "\n" for item in samples),
                    encoding="utf-8",
                )
                manifest = experiment.read_and_validate_manifest(
                    run_dir / "EXPERIMENT_MANIFEST.toml"
                )
                with self.assertRaisesRegex(experiment.ExperimentError, expected):
                    experiment.validate_telemetry(run_dir, manifest)

    def test_water_all_sleep_and_plateau_detectors_use_exact_diagnostic_rules(self) -> None:
        run_dir = self.create_valid_water_worker_fixture(
            "g8b-water-flow-v0-sleep-test"
        )
        sample_path = run_dir / "telemetry" / "samples.jsonl"
        samples = [
            json.loads(line)
            for line in sample_path.read_text(encoding="utf-8").splitlines()
        ]
        all_sleep, _, _ = experiment.confirmed_water_all_sleep_streak(samples, 3)
        self.assertEqual([item["sim_tick"] for item in all_sleep or []], [8, 16, 24])

        plateau_samples = [copy.deepcopy(samples[4]) for _ in range(8)]
        for sequence, item in enumerate(plateau_samples):
            item["sample_sequence"] = sequence
            item["sim_tick"] = 8 * (sequence + 1)
        plateau, starts, breaks = experiment.confirmed_water_plateau_streak(
            plateau_samples, 8
        )
        self.assertEqual(len(plateau or []), 8)
        self.assertEqual(len(starts), 1)
        self.assertEqual(breaks, [])
        plateau_samples[4]["wake_chunks"] = 1
        plateau, _, breaks = experiment.confirmed_water_plateau_streak(
            plateau_samples, 8
        )
        self.assertIsNone(plateau)
        self.assertEqual(len(breaks), 1)

        samples[5]["census"]["runnable_chunks"] = 1
        samples[5]["census"]["sleeping_chunks"] = 15
        sample_path.write_text(
            "".join(json.dumps(item) + "\n" for item in samples), encoding="utf-8"
        )
        manifest = experiment.read_and_validate_manifest(
            run_dir / "EXPERIMENT_MANIFEST.toml"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "three-sample streak"):
            experiment.validate_telemetry(run_dir, manifest)

    def test_water_post_settle_and_reset_evidence_are_bound(self) -> None:
        for label, mutate, expected in (
            (
                "post",
                lambda samples: samples[10].update(
                    {
                        "state_hash": "fnv1a64:ffffffffffffffff",
                        "physical_state_hash": "fnv1a64:ffffffffffffffff",
                    }
                ),
                "post-settle change count",
            ),
            (
                "reset",
                lambda samples: samples[-1].update(
                    {
                        "state_hash": "fnv1a64:eeeeeeeeeeeeeeee",
                        "physical_state_hash": "fnv1a64:eeeeeeeeeeeeeeee",
                    }
                ),
                "exact Water reset",
            ),
        ):
            with self.subTest(evidence=label):
                run_dir = self.create_valid_water_worker_fixture(
                    f"g8b-water-flow-v0-{label}-test"
                )
                sample_path = run_dir / "telemetry" / "samples.jsonl"
                samples = [
                    json.loads(line)
                    for line in sample_path.read_text(encoding="utf-8").splitlines()
                ]
                mutate(samples)
                sample_path.write_text(
                    "".join(json.dumps(item) + "\n" for item in samples),
                    encoding="utf-8",
                )
                manifest = experiment.read_and_validate_manifest(
                    run_dir / "EXPERIMENT_MANIFEST.toml"
                )
                with self.assertRaisesRegex(experiment.ExperimentError, expected):
                    experiment.validate_telemetry(run_dir, manifest)

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

    def test_worker_commands_and_scenario_rejection_are_exact(self) -> None:
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
        water = experiment.worker_command(
            binary,
            run_dir,
            "run-2",
            "d" * 64,
            contract=experiment.WATER_CONTRACT,
        )
        self.assertEqual(water[1:3], ("--experiment-worker", "water-flow"))
        self.assertNotIn("--mode", water)
        self.assertNotIn("stable-plateau", water)
        with self.assertRaises(experiment.ExperimentError):
            experiment.contract_for_scenario("pressure-burst")

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

    def test_contact_sheet_caption_joins_activity_and_state_hash(self) -> None:
        item = {
            "ordinal": 3,
            "reason": "destination-arrival",
            "sim_tick": 24,
            "sample_sequence": 7,
        }
        sample = {
            "sample_sequence": 7,
            "state_hash": "fnv1a64:0123456789abcdef",
            "census": {
                "any_active_cells": 123,
                "runnable_chunks": 8,
                "sleeping_chunks": 6,
            },
        }
        self.assertEqual(
            experiment.contact_sheet_caption_lines(item, sample),
            (
                "#3 destination-arrival | sim 24 | sample 7",
                "Active cells 123 | Runnable 8 | Sleeping 6",
                "State fnv1a64:0123456789abcdef",
            ),
        )
        mismatched = copy.deepcopy(sample)
        mismatched["sample_sequence"] = 8
        with self.assertRaisesRegex(experiment.ExperimentError, "identity mismatch"):
            experiment.contact_sheet_caption_lines(item, mismatched)

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
        report = json.loads((run_dir / "report" / "REPORT.json").read_text(encoding="utf-8"))
        self.assertEqual(report["schema_version"], experiment.SAND_REPORT_SCHEMA)
        self.assertNotIn("run_mode", report)
        self.assertEqual(receipt["schema_version"], experiment.SAND_RECEIPT_SCHEMA)
        self.assertNotIn("run_mode", receipt)
        self.assertEqual(receipt["review_packet_sha256"], experiment.sha256_file(packet))
        self.assertTrue(receipt["receipt_is_final_publication_marker"])
        with self.assertRaises(experiment.ExperimentError):
            experiment.postprocess_run(run_dir)

    def test_water_packet_is_create_new_and_receipt_last(self) -> None:
        run_dir = self.create_valid_water_worker_fixture(
            "g8b-water-flow-v0-packet-test"
        )
        publication_log: list[str] = []
        receipt_path = experiment.postprocess_run(run_dir, publication_log)
        self.assertEqual(publication_log[-1], "EXPERIMENT_RECEIPT.json")
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        report = json.loads(
            (run_dir / "report" / "REPORT.json").read_text(encoding="utf-8")
        )
        self.assertEqual(receipt["schema_version"], experiment.WATER_RECEIPT_SCHEMA)
        self.assertEqual(receipt["run_mode"], "candidate")
        self.assertEqual(report["schema_version"], experiment.WATER_REPORT_SCHEMA)
        self.assertEqual(report["run_mode"], "candidate")
        self.assertEqual(
            report["water_remediation"]["active_cell_classification_rule"],
            experiment.WATER_ACTIVE_CLASSIFICATION_RULE,
        )
        self.assertEqual(
            report["water_remediation"]["max_water_outside_outer_basin_cells"], 0
        )
        self.assertEqual(report["water_remediation"]["final_any_active_cells"], 0)
        self.assertEqual(report["water_remediation"], receipt["water_remediation"])
        self.assertEqual(report["screenshots"][0]["kind"], "tick0")
        self.assertFalse(report["scope"]["ai_contacted"])
        self.assertFalse(
            report["review_guidance"]["categories_are_findings"]
        )
        prompt = (run_dir / "report" / "CHATGPT_REVIEW_PROMPT.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("was not sent to an AI", prompt)
        self.assertIn("actual_physics_defect", prompt)
        self.assertIn("Does Water visibly leave", prompt)
        self.assertIn(experiment.WATER_ACTIVE_CLASSIFICATION_RULE, prompt)
        packet = run_dir / "report" / "REVIEW_PACKET.zip"
        with zipfile.ZipFile(packet) as archive:
            names = set(archive.namelist())
        self.assertIn("report/CONTACT_SHEET.png", names)
        self.assertIn("telemetry/samples.jsonl", names)
        self.assertNotIn("HASHES.sha256", names)
        self.assertNotIn("EXPERIMENT_RECEIPT.json", names)
        with self.assertRaisesRegex(experiment.ExperimentError, "receipt"):
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
