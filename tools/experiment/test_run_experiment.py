from __future__ import annotations

import copy
import hashlib
import io
import json
import subprocess
import tempfile
import tomllib
import unittest
import zipfile
from contextlib import ExitStack
from dataclasses import replace
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
        self.external_font = self.base / "external-inputs" / "consola.ttf"
        self.external_font.parent.mkdir()
        self.external_font.write_bytes(b"temporary Consolas fixture bytes")

        self.contract_patches = ExitStack()
        self.addCleanup(self.contract_patches.close)
        self.contract_patches.enter_context(
            mock.patch.object(
                experiment,
                "SOURCE_EXTERNAL_BUILD_INPUTS",
                (("windows-consolas-font", self.external_font),),
            )
        )
        for name, value in {
            "RENDERER_WIDTH": 32,
            "RENDERER_HEIGHT": 24,
            "CROP_X": 4,
            "CROP_Y": 2,
            "CROP_WIDTH": 16,
            "CROP_HEIGHT": 16,
        }.items():
            self.contract_patches.enter_context(mock.patch.object(experiment, name, value))

    def initialize_source_repository(self) -> None:
        files = {
            "Cargo.toml": "[workspace]\nmembers = []\n",
            "Cargo.lock": "version = 4\n",
            "apps/windows/Cargo.toml": "[package]\nname = \"sealed-test\"\n",
            "apps/windows/build.rs": "fn main() {}\n",
            "apps/windows/src/main.rs": "fn main() {}\n",
            "apps/scenarios/src/fixture.rs": "pub fn fixture() {}\n",
            "engine/gpu/src/test.wgsl": "@compute @workgroup_size(1) fn main() {}\n",
            "run_experiment.bat": "@echo off\n",
            "tools/experiment/run_experiment.py": "print('runner')\n",
            "docs/ignored.md": "not a source input\n",
        }
        for relative, text in files.items():
            path = self.source / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8")
        commands = (
            ("git", "init"),
            ("git", "config", "user.email", "seal-test@example.invalid"),
            ("git", "config", "user.name", "Seal Test"),
            ("git", "checkout", "-b", "feature/source-seal-test"),
            ("git", "add", "."),
            ("git", "commit", "-m", "fixture"),
        )
        for command in commands:
            completed = subprocess.run(
                command,
                cwd=self.source,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            self.assertEqual(
                completed.returncode,
                0,
                completed.stderr.decode("utf-8", errors="replace"),
            )

    def manifest_data(
        self,
        run_dir: Path,
        contract: experiment.ScenarioContract = experiment.SAND_CONTRACT,
        mode: str = "candidate",
        git_state: str = "clean",
    ) -> experiment.ManifestData:
        binary = (
            run_dir.joinpath(*experiment.FROZEN_BINARY_RELATIVE_PATH.parts)
            if contract
            in {
                experiment.FIRE_CONTRACT,
                experiment.PRESSURE_CONTRACT,
                experiment.HEAVY_CONTRACT,
            }
            else self.source / "target" / "release" / "powdergame-windows.exe"
        )
        binary_sha256 = "b" * 64
        return experiment.ManifestData(
            run_id=run_dir.name,
            created_utc="2026-08-17T06:00:00.000000Z",
            source=experiment.SourceInfo(
                root=self.source.resolve(),
                branch="feature/g8b-experiment-harness-v0",
                sha="a" * 40,
                git_state=git_state,
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
        git_state: str = "clean",
    ) -> tuple[Path, dict]:
        run_dir = experiment.create_run_directory(self.artifacts, run_id)
        manifest_path = run_dir / "EXPERIMENT_MANIFEST.toml"
        experiment.write_new_text(
            manifest_path,
            experiment.render_manifest(
                self.manifest_data(run_dir, contract, mode, git_state)
            ),
        )
        return run_dir, experiment.read_and_validate_manifest(manifest_path)

    def create_sealed_delivery_fixture(
        self, run_id: str, mode: str = "candidate"
    ) -> Path:
        run_dir = self.create_valid_fire_worker_fixture(run_id, mode=mode)
        binary = run_dir.joinpath(*experiment.FROZEN_BINARY_RELATIVE_PATH.parts)
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"frozen executable fixture")
        binary_hash = experiment.sha256_file(binary)
        old_binary_hash = "b" * 64
        for path in run_dir.rglob("*"):
            if path.is_file() and path.suffix in {".json", ".jsonl", ".toml"}:
                text = path.read_text(encoding="utf-8")
                path.write_text(
                    text.replace(old_binary_hash, binary_hash), encoding="utf-8"
                )
        experiment.write_new_text(
            run_dir / experiment.SOURCE_INPUT_MANIFEST_NAME,
            json.dumps({"schema_version": experiment.SOURCE_INPUT_MANIFEST_SCHEMA})
            + "\n",
        )
        experiment.postprocess_run(run_dir)
        return run_dir

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

    def create_valid_fire_worker_fixture(
        self,
        run_id: str = "g8b-fire-heat-v0-test-run",
        mode: str = "candidate",
    ) -> Path:
        if mode == "scratch" and "-scratch-" not in run_id:
            run_id = run_id.replace("g8b-fire-heat-v0-", "g8b-fire-heat-v0-scratch-")
        run_dir, manifest = self.create_manifest(run_id, experiment.FIRE_CONTRACT, mode)
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

        def sample(
            tick: int,
            phase: str,
            reason: str,
            *,
            reaction: int,
            thermal: int,
            wood: int,
            oil: int,
            smoke: int,
            ice: int,
            water: int,
            steam: int,
            wood_flame: int = 0,
            oil_flame: int = 0,
            wood_progress: int = 0,
            oil_progress: int = 0,
            heat: int = 0,
            changed: int = 2,
            state_hash: str,
        ) -> dict:
            boundary = 1020
            stone = 3256
            sand = 0
            non_empty = boundary + stone + sand + water + oil + steam + smoke + ice + wood
            counts = [
                experiment.WORLD_WIDTH * experiment.WORLD_HEIGHT - non_empty,
                boundary,
                stone,
                sand,
                water,
                oil,
                steam,
                smoke,
                ice,
                wood,
            ]
            any_active = max(reaction, thermal)
            return {
                "schema_version": experiment.FIRE_TELEMETRY_SCHEMA,
                "experiment_id": experiment.FIRE_CONTRACT.experiment_id,
                "run_id": run_dir.name,
                "scenario": experiment.FIRE_CONTRACT.scenario,
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
                    "any_active_cells": any_active,
                    "matter_active_cells": reaction,
                    "thermal_active_cells": thermal,
                    "pressure_active_cells": 0,
                    "reaction_active_cells": reaction,
                    "total_chunks": 16,
                    "active_chunks": 4 if any_active else 0,
                    "runnable_chunks": 16,
                    "sleeping_chunks": 0,
                },
                "material_counts_by_id": counts,
                "matter_count": sum(counts[1:]),
                "wood_count": wood,
                "oil_count": oil,
                "smoke_count": smoke,
                "ice_count": ice,
                "water_count": water,
                "steam_count": steam,
                "combusting_wood_cells": min(wood, 544),
                "combusting_oil_cells": min(oil, 272),
                "flame_event_wood_cells": wood_flame,
                "flame_event_oil_cells": oil_flame,
                "wood_fuel_progress_sum": wood_progress,
                "oil_fuel_progress_sum": oil_progress,
                "heat_propagated_cells": heat,
                "phase_inventory_changed": (ice, water, steam) != (2240, 1536, 0),
                "invalid_material_count": 0,
                "nonfinite_temperature_count": 0,
                "nonfinite_pressure_count": 0,
                "changed_chunks": changed,
                "wake_chunks": 0,
                "wake_reason_or": 0,
                "state_hash": state_hash,
                "physical_state_hash": state_hash,
            }

        tick0 = sample(
            0,
            "initial",
            "tick0",
            reaction=884,
            thermal=3776,
            wood=10926,
            oil=1610,
            smoke=0,
            ice=2240,
            water=1536,
            steam=0,
            changed=0,
            state_hash="fnv1a64:0000000000001000",
        )
        tick1 = sample(
            1,
            "reacting",
            "tick1",
            reaction=100,
            thermal=200,
            wood=10920,
            oil=1608,
            smoke=1,
            ice=2240,
            water=1536,
            steam=0,
            wood_flame=10,
            oil_flame=5,
            wood_progress=10,
            oil_progress=5,
            heat=12,
            state_hash="fnv1a64:0000000000001001",
        )
        tick2 = sample(
            2,
            "reacting",
            "early-diagnostic",
            reaction=120,
            thermal=250,
            wood=10000,
            oil=1500,
            smoke=10,
            ice=2230,
            water=1540,
            steam=6,
            wood_progress=100,
            oil_progress=60,
            heat=30,
            state_hash="fnv1a64:0000000000001002",
        )
        zero8 = sample(
            8,
            "reacting",
            "diagnostic-cadence",
            reaction=0,
            thermal=220,
            wood=8000,
            oil=1300,
            smoke=20,
            ice=2230,
            water=1540,
            steam=6,
            wood_progress=300,
            oil_progress=100,
            heat=50,
            state_hash="fnv1a64:0000000000001008",
        )
        zero16 = copy.deepcopy(zero8)
        zero16.update({"sim_tick": 16, "state_hash": "fnv1a64:0000000000001010"})
        zero16["physical_state_hash"] = zero16["state_hash"]
        zero24 = copy.deepcopy(zero8)
        zero24.update(
            {
                "sim_tick": 24,
                "census": {**zero24["census"], "thermal_active_cells": 200, "any_active_cells": 200},
                "state_hash": "fnv1a64:0000000000001018",
            }
        )
        zero24["physical_state_hash"] = zero24["state_hash"]
        samples = [tick0, tick1, tick2, zero8, zero16, zero24]
        for offset in range(1, experiment.POST_REACTION_TICKS + 1):
            thermal = max(20, 200 - offset)
            post = sample(
                24 + offset,
                "post-reaction-confirmation",
                "post-reaction-tick",
                reaction=0,
                thermal=thermal,
                wood=8000,
                oil=1300,
                smoke=25,
                ice=2230,
                water=1540,
                steam=6,
                wood_progress=300,
                oil_progress=100,
                heat=50,
                state_hash=f"fnv1a64:{0x2000 + offset:016x}",
            )
            samples.append(post)
        reset = copy.deepcopy(tick0)
        reset.update(
            {
                "sim_tick": 0,
                "phase": "reset",
                "reason": "programmatic-r-equivalent",
            }
        )
        samples.append(reset)
        for sequence, item in enumerate(samples):
            item["sample_sequence"] = sequence
        (telemetry / "samples.jsonl").write_text(
            "".join(json.dumps(item, separators=(",", ":")) + "\n" for item in samples),
            encoding="utf-8",
        )

        def event(name: str, item: dict | None, detail: str = "fixture") -> dict:
            return {
                "schema_version": experiment.FIRE_TELEMETRY_SCHEMA,
                "experiment_id": experiment.FIRE_CONTRACT.experiment_id,
                "run_id": run_dir.name,
                "scenario": experiment.FIRE_CONTRACT.scenario,
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
            event("combustion_observed", tick1),
            event("smoke_generated", tick1),
            event("heat_propagated", tick1),
            event("new_peak_reaction", tick1),
            event("new_peak_thermal", tick1),
            event("phase_transition_observed", tick2),
            event("new_peak_reaction", tick2),
            event("new_peak_thermal", tick2),
            event("fuel_substantially_consumed", zero8),
            event("reaction_zero_streak_started", zero8),
            event("reaction_zero_confirmed", zero24),
            event("terminal_selected", zero24),
            event("post_reaction_confirmation_completed", samples[-2]),
            event("reset_started", samples[-2]),
            event("reset_comparison_completed", reset),
            event("worker_completed", reset, "PASS"),
        ]
        for sequence, item in enumerate(events):
            item["event_sequence"] = sequence
        (telemetry / "events.jsonl").write_text(
            "".join(json.dumps(item, separators=(",", ":")) + "\n" for item in events),
            encoding="utf-8",
        )

        frame_specs = (
            ("tick0", tick0, "pristine-reset"),
            ("tick1", tick1, "after-one-production-tick"),
            ("first-combustion", tick1, "both-fuels-production-combustion"),
            ("first-smoke", tick1, "smoke-count-above-tick0"),
            ("peak-reaction", tick2, "highest-observed-reaction-cells"),
            ("peak-thermal", tick2, "highest-observed-thermal-cells"),
            ("first-phase-transition", tick2, "phase-inventory-differs-from-tick0"),
            (
                "fuel-substantially-consumed",
                zero8,
                "at-least-25-percent-initial-fuel-consumed",
            ),
            ("reaction-zero", zero8, "first-sample-of-confirmed-reaction-zero-streak"),
            ("post-reaction-tail", samples[-2], "post-reaction-confirmation-complete"),
            ("terminal", zero24, "reaction-zero-confirmed"),
            ("reset", reset, "programmatic-r-equivalent"),
        )
        raw_size = experiment.RENDERER_WIDTH * experiment.RENDERER_HEIGHT * 4
        frames = []
        for ordinal, (kind, item, reason) in enumerate(frame_specs):
            filename = f"{ordinal:02}-{kind}.rgba"
            color = bytes(((ordinal * 29) % 256, (ordinal * 41) % 256, 96, 255))
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
        (run_dir / "work" / "frames.json").write_text(
            json.dumps(
                {
                    "schema_version": experiment.FRAMES_SCHEMA,
                    "experiment_id": experiment.FIRE_CONTRACT.experiment_id,
                    "run_id": run_dir.name,
                    "scenario": experiment.FIRE_CONTRACT.scenario,
                    "binary_sha256": manifest["binary"]["sha256"],
                    "frame_count": len(frames),
                    "pixel_encoding": "rgba8-tightly-packed",
                    "frames": frames,
                }
            ),
            encoding="utf-8",
        )

        initial_fuel = tick0["wood_count"] + tick0["oil_count"]
        threshold = (initial_fuel + 3) // 4
        final = samples[-2]
        predicates = {
            name: {"status": "pass", "detail": f"fixture {name}"}
            for name in experiment.FIRE_PREDICATE_NAMES
        }
        analysis = {
            "schema_version": experiment.FIRE_ANALYSIS_SCHEMA,
            "experiment_id": experiment.FIRE_CONTRACT.experiment_id,
            "run_id": run_dir.name,
            "scenario": experiment.FIRE_CONTRACT.scenario,
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
                "consecutive_reaction_zero_samples": experiment.CONSECUTIVE_REACTION_ZERO,
                "post_reaction_confirmation_ticks": experiment.POST_REACTION_TICKS,
                "terminal_reason": "reaction-zero",
                "first_reaction_zero_sim_tick": 8,
                "first_reaction_zero_sample_sequence": zero8["sample_sequence"],
                "confirmed_reaction_zero_sim_tick": 24,
                "confirmed_reaction_zero_sample_sequence": zero24["sample_sequence"],
                "post_reaction_end_tick": 204,
                "post_reaction_restart_samples": 0,
                "sample_count": len(samples),
            },
            "baseline": {
                "matter_count": tick0["matter_count"],
                "wood_count": tick0["wood_count"],
                "oil_count": tick0["oil_count"],
                "smoke_count": tick0["smoke_count"],
                "ice_count": tick0["ice_count"],
                "water_count": tick0["water_count"],
                "steam_count": tick0["steam_count"],
                "fuel_count": initial_fuel,
                "wood_fuel_progress_sum": 0,
                "oil_fuel_progress_sum": 0,
                "substantial_fuel_consumption_threshold": threshold,
                "substantial_fuel_remaining_threshold": initial_fuel - threshold,
            },
            "metrics": {
                "first_combustion_tick": 1,
                "first_combustion_sample_sequence": tick1["sample_sequence"],
                "first_smoke_tick": 1,
                "first_smoke_sample_sequence": tick1["sample_sequence"],
                "first_phase_transition_tick": 2,
                "first_phase_transition_sample_sequence": tick2["sample_sequence"],
                "fuel_substantially_consumed_tick": 8,
                "fuel_substantially_consumed_sample_sequence": zero8["sample_sequence"],
                "peak_reaction_cells": 120,
                "peak_reaction_tick": 2,
                "peak_reaction_sample_sequence": tick2["sample_sequence"],
                "peak_thermal_cells": 250,
                "peak_thermal_tick": 2,
                "peak_thermal_sample_sequence": tick2["sample_sequence"],
                "peak_smoke_count": 25,
                "peak_smoke_tick": 25,
                "peak_smoke_sample_sequence": samples[6]["sample_sequence"],
                "max_heat_propagated_cells": 50,
                "reaction_zero_tick": 8,
                "confirmed_reaction_zero_tick": 24,
                "post_reaction_thermal_cells": 200,
                "post_reaction_final_thermal_cells": 20,
                "post_reaction_min_thermal_cells": 20,
                "post_reaction_thermal_decrease": True,
                "post_reaction_reaction_restart_ticks": 0,
                "post_reaction_restart_samples": 0,
                "final_matter_count": final["matter_count"],
                "final_wood_count": final["wood_count"],
                "final_oil_count": final["oil_count"],
                "final_smoke_count": final["smoke_count"],
                "final_ice_count": final["ice_count"],
                "final_water_count": final["water_count"],
                "final_steam_count": final["steam_count"],
                "wood_count_delta": final["wood_count"] - tick0["wood_count"],
                "oil_count_delta": final["oil_count"] - tick0["oil_count"],
                "fuel_count_delta": final["wood_count"] + final["oil_count"] - initial_fuel,
                "fuel_consumed": initial_fuel - final["wood_count"] - final["oil_count"],
                "invalid_material_occurrences": 0,
                "nonfinite_field_occurrences": 0,
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

    def create_valid_pressure_worker_fixture(
        self,
        run_id: str = "g8b-pressure-burst-v0-test-run",
        mode: str = "candidate",
        git_state: str = "clean",
    ) -> Path:
        if mode == "scratch" and "-scratch-" not in run_id:
            run_id = run_id.replace(
                "g8b-pressure-burst-v0-", "g8b-pressure-burst-v0-scratch-"
            )
        run_dir, manifest = self.create_manifest(
            run_id, experiment.PRESSURE_CONTRACT, mode, git_state
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

        initial_water = 20_000
        initial_steam = 0
        initial_total_wood = 11_516

        def sample(
            tick: int,
            phase: str,
            reason: str,
            *,
            pressure_active: int,
            mean_pressure: float,
            max_pressure: float,
            top_wood: int,
            bottom_wood: int,
            water: int,
            steam: int,
            outside_steam: int,
            state_hash: str,
            top_through_lanes: int = 0,
            bottom_through_lanes: int = 0,
            seam_steam: int = 0,
            changed: int = 2,
            top_combusting: int = 0,
            bottom_combusting: int = 0,
            top_flame_event: int = 0,
            bottom_flame_event: int = 0,
            top_fuel_progress_sum: int = 0,
            top_fuel_progress_max: int = 0,
            bottom_fuel_progress_sum: int = 0,
            bottom_fuel_progress_max: int = 0,
            top_adjacent_pressure_medium: int | None = None,
            bottom_adjacent_pressure_medium: int | None = None,
            top_max_adjacent_pressure: float | None = None,
            bottom_max_adjacent_pressure: float | None = None,
        ) -> dict:
            seam_wood = top_wood + bottom_wood
            seam_lost = 576 - seam_wood
            wood = initial_total_wood - seam_lost
            boundary = 1020
            stone = 3000
            non_empty = boundary + stone + water + steam + wood
            counts = [
                experiment.WORLD_WIDTH * experiment.WORLD_HEIGHT - non_empty,
                boundary,
                stone,
                0,
                water,
                0,
                steam,
                0,
                0,
                wood,
            ]
            top_open = 384 - top_wood
            bottom_open = 192 - bottom_wood
            seam_open = top_open + bottom_open
            through_lanes = top_through_lanes + bottom_through_lanes
            top_adjacent_pressure_medium = (
                top_through_lanes
                if top_adjacent_pressure_medium is None
                else top_adjacent_pressure_medium
            )
            bottom_adjacent_pressure_medium = (
                bottom_through_lanes
                if bottom_adjacent_pressure_medium is None
                else bottom_adjacent_pressure_medium
            )
            top_max_adjacent_pressure = (
                max_pressure
                if top_max_adjacent_pressure is None and top_adjacent_pressure_medium > 0
                else 0.0
                if top_max_adjacent_pressure is None
                else top_max_adjacent_pressure
            )
            bottom_max_adjacent_pressure = (
                max_pressure
                if bottom_max_adjacent_pressure is None
                and bottom_adjacent_pressure_medium > 0
                else 0.0
                if bottom_max_adjacent_pressure is None
                else bottom_max_adjacent_pressure
            )
            return {
                "schema_version": experiment.PRESSURE_TELEMETRY_SCHEMA,
                "experiment_id": experiment.PRESSURE_CONTRACT.experiment_id,
                "run_id": run_dir.name,
                "scenario": experiment.PRESSURE_CONTRACT.scenario,
                "source_sha": manifest["source"]["sha"],
                "git_state": manifest["source"]["git_state"],
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
                    "any_active_cells": pressure_active,
                    "matter_active_cells": min(pressure_active, 20),
                    "thermal_active_cells": 0,
                    "pressure_active_cells": pressure_active,
                    "reaction_active_cells": 0,
                    "total_chunks": 16,
                    "active_chunks": 8 if pressure_active else 0,
                    "runnable_chunks": 16,
                    "sleeping_chunks": 0,
                },
                "material_counts_by_id": counts,
                "matter_count": sum(counts[1:]),
                "water_count": water,
                "steam_count": steam,
                "relief_seam_wood_cells": seam_wood,
                "top_relief_seam_wood_cells": top_wood,
                "bottom_relief_seam_wood_cells": bottom_wood,
                "relief_seam_open_cells": seam_open,
                "top_relief_seam_open_cells": top_open,
                "bottom_relief_seam_open_cells": bottom_open,
                "relief_seam_through_open_lanes": through_lanes,
                "top_relief_seam_through_open_lanes": top_through_lanes,
                "bottom_relief_seam_through_open_lanes": bottom_through_lanes,
                "top_relief_seam_combusting_cells": top_combusting,
                "bottom_relief_seam_combusting_cells": bottom_combusting,
                "relief_seam_combusting_cells": top_combusting + bottom_combusting,
                "top_relief_seam_flame_event_cells": top_flame_event,
                "bottom_relief_seam_flame_event_cells": bottom_flame_event,
                "relief_seam_flame_event_cells": top_flame_event
                + bottom_flame_event,
                "top_relief_seam_fuel_progress_sum": top_fuel_progress_sum,
                "top_relief_seam_fuel_progress_max": top_fuel_progress_max,
                "bottom_relief_seam_fuel_progress_sum": bottom_fuel_progress_sum,
                "bottom_relief_seam_fuel_progress_max": bottom_fuel_progress_max,
                "relief_seam_fuel_progress_sum": top_fuel_progress_sum
                + bottom_fuel_progress_sum,
                "relief_seam_fuel_progress_max": max(
                    top_fuel_progress_max, bottom_fuel_progress_max
                ),
                "top_relief_seam_adjacent_pressure_medium_cells": top_adjacent_pressure_medium,
                "bottom_relief_seam_adjacent_pressure_medium_cells": (
                    bottom_adjacent_pressure_medium
                ),
                "relief_seam_adjacent_pressure_medium_cells": top_adjacent_pressure_medium
                + bottom_adjacent_pressure_medium,
                "top_relief_seam_max_adjacent_pressure": top_max_adjacent_pressure,
                "bottom_relief_seam_max_adjacent_pressure": bottom_max_adjacent_pressure,
                "relief_seam_max_adjacent_pressure": max(
                    top_max_adjacent_pressure, bottom_max_adjacent_pressure
                ),
                "steam_in_relief_seam_cells": seam_steam,
                "outside_chamber_steam_cells": outside_steam,
                "chamber_pressure_cell_count": 29_920,
                "chamber_mean_pressure": mean_pressure,
                "chamber_max_pressure": max_pressure,
                "invalid_material_count": 0,
                "nonfinite_temperature_count": 0,
                "nonfinite_pressure_count": 0,
                "changed_chunks": changed,
                "wake_chunks": 0,
                "wake_reason_or": 0,
                "state_hash": state_hash,
                "physical_state_hash": state_hash,
            }

        tick0 = sample(
            0,
            "initial",
            "tick0",
            pressure_active=0,
            mean_pressure=100.0,
            max_pressure=180.0,
            top_wood=384,
            bottom_wood=192,
            water=initial_water,
            steam=initial_steam,
            outside_steam=0,
            state_hash="fnv1a64:0000000000003000",
            changed=0,
        )
        tick1 = sample(
            1,
            "pressurizing",
            "tick1",
            pressure_active=500,
            mean_pressure=110.0,
            max_pressure=200.0,
            # One complete inner layer is damaged in each eight-cell-thick seam,
            # but neither seam has a cavity-to-exterior through lane yet.
            top_wood=336,
            bottom_wood=168,
            water=initial_water,
            steam=initial_steam,
            outside_steam=0,
            state_hash="fnv1a64:0000000000003001",
        )
        tick2 = sample(
            2,
            "pressurizing",
            "early-diagnostic",
            pressure_active=1000,
            mean_pressure=130.0,
            max_pressure=220.0,
            top_wood=328,
            bottom_wood=160,
            water=19_990,
            steam=10,
            outside_steam=0,
            state_hash="fnv1a64:0000000000003002",
            top_through_lanes=1,
            bottom_through_lanes=1,
        )
        tick8 = sample(
            8,
            "pressurizing",
            "diagnostic-cadence",
            pressure_active=800,
            mean_pressure=120.0,
            max_pressure=210.0,
            top_wood=320,
            bottom_wood=152,
            water=19_980,
            steam=20,
            outside_steam=0,
            state_hash="fnv1a64:0000000000003008",
            top_through_lanes=1,
            bottom_through_lanes=1,
        )
        tick16 = sample(
            16,
            "pressurizing",
            "diagnostic-cadence",
            pressure_active=750,
            mean_pressure=115.0,
            max_pressure=205.0,
            top_wood=312,
            bottom_wood=144,
            water=19_980,
            steam=20,
            outside_steam=0,
            state_hash="fnv1a64:0000000000003010",
            top_through_lanes=1,
            bottom_through_lanes=1,
        )
        samples = [tick0, tick1, tick2, tick8, tick16]
        for offset in range(1, experiment.POST_OPENING_TICKS + 1):
            converted = min(100, 20 + offset)
            post = sample(
                16 + offset,
                "post-opening-observation",
                "post-opening-observation-complete"
                if offset == experiment.POST_OPENING_TICKS
                else "post-opening-tick",
                pressure_active=max(0, 800 - offset * 5),
                mean_pressure=float(max(40, 100 - offset)),
                max_pressure=float(max(80, 190 - offset)),
                top_wood=max(280, 312 - offset // 6),
                bottom_wood=max(128, 144 - offset // 18),
                water=initial_water - converted,
                steam=converted,
                outside_steam=max(0, 4 - offset // 40),
                state_hash=f"fnv1a64:{0x4000 + offset:016x}",
                top_through_lanes=1,
                bottom_through_lanes=1,
                seam_steam=min(10, converted),
            )
            samples.append(post)
        reset = copy.deepcopy(tick0)
        reset.update(
            {
                "phase": "reset",
                "reason": "programmatic-r-equivalent",
            }
        )
        samples.append(reset)
        for sequence, item in enumerate(samples):
            item["sample_sequence"] = sequence
        (telemetry / "samples.jsonl").write_text(
            "".join(json.dumps(item, separators=(",", ":")) + "\n" for item in samples),
            encoding="utf-8",
        )

        def event(name: str, item: dict | None, detail: str = "fixture") -> dict:
            return {
                "schema_version": experiment.PRESSURE_TELEMETRY_SCHEMA,
                "experiment_id": experiment.PRESSURE_CONTRACT.experiment_id,
                "run_id": run_dir.name,
                "scenario": experiment.PRESSURE_CONTRACT.scenario,
                "event_sequence": -1,
                "event": name,
                "sim_tick": 0 if item is None else item["sim_tick"],
                "sample_sequence": None if item is None else item["sample_sequence"],
                "detail": detail,
            }

        first_vent = samples[5]
        first_relief = samples[6]
        final = samples[-2]
        events = [
            event("lifecycle_started", None),
            event("pristine_reset_completed", None),
            event("tick0_captured", tick0),
            event("pressure_activity_observed", tick1),
            event("relief_seam_damage_observed", tick1),
            event("new_peak_chamber_mean_pressure", tick1),
            event("new_peak_chamber_max_pressure", tick1),
            event("new_peak_pressure_activity", tick1),
            event("tick1_captured", tick1),
            event("persistent_opening_streak_started", tick2),
            event("rupture_observed", tick2),
            event("new_peak_chamber_mean_pressure", tick2),
            event("new_peak_chamber_max_pressure", tick2),
            event("new_peak_pressure_activity", tick2),
            event("persistent_opening_confirmed", tick16),
            event("post_opening_observation_started", tick16),
            event("relief_seam_steam_observed", first_vent),
            event("exterior_vent_observed", first_vent),
            event("post_opening_pressure_relief_observed", first_relief),
            event("post_opening_observation_completed", final),
            event("terminal_selected", final),
            event("reset_started", final),
            event("reset_comparison_completed", reset),
            event("worker_completed", reset, "PASS"),
        ]
        for sequence, item in enumerate(events):
            item["event_sequence"] = sequence
        (telemetry / "events.jsonl").write_text(
            "".join(json.dumps(item, separators=(",", ":")) + "\n" for item in events),
            encoding="utf-8",
        )

        frame_specs = (
            (tick0, [("tick0", "pristine-reset")]),
            (
                tick1,
                [
                    ("tick1", "after-one-production-tick"),
                    ("first-pressure-activity", "first-sampled-pressure-activity"),
                    ("first-wood-damage", "first-authored-relief-seam-wood-loss"),
                ],
            ),
            (
                tick2,
                [
                    ("first-rupture", "first-eight-cell-through-open-relief-lane"),
                    ("peak-pressure", "highest-observed-chamber-max-pressure"),
                    (
                        "peak-pressure-activity",
                        "highest-observed-pressure-active-cells",
                    ),
                ],
            ),
            (
                tick16,
                [
                    (
                        "persistent-opening",
                        "three-consecutive-diagnostics-with-opening",
                    )
                ],
            ),
            (
                first_vent,
                [
                    (
                        "first-exterior-steam",
                        "first-steam-outside-authored-chamber-after-opening",
                    )
                ],
            ),
            (
                first_relief,
                [
                    (
                        "post-opening",
                        "first-post-vent-chamber-mean-and-max-pressure-relief",
                    )
                ],
            ),
            (final, [("terminal", "post-opening-observation-complete")]),
            (reset, [("reset", "programmatic-r-equivalent")]),
        )
        raw_size = experiment.RENDERER_WIDTH * experiment.RENDERER_HEIGHT * 4
        frames = []
        for ordinal, (item, badges) in enumerate(frame_specs):
            filename = f"{ordinal:02}-{badges[0][0]}.rgba"
            color = bytes(((ordinal * 31) % 256, (ordinal * 47) % 256, 128, 255))
            (frames_dir / filename).write_bytes(color * (raw_size // 4))
            frames.append(
                {
                    "ordinal": ordinal,
                    "relative_path": f"work/frames/{filename}",
                    "width": experiment.RENDERER_WIDTH,
                    "height": experiment.RENDERER_HEIGHT,
                    "rgba_bytes": raw_size,
                    "badges": [
                        {"kind": kind, "reason": reason} for kind, reason in badges
                    ],
                    "sim_tick": item["sim_tick"],
                    "sample_sequence": item["sample_sequence"],
                    "state_hash": item["state_hash"],
                }
            )
        (run_dir / "work" / "frames.json").write_text(
            json.dumps(
                {
                    "schema_version": experiment.PRESSURE_FRAMES_SCHEMA,
                    "experiment_id": experiment.PRESSURE_CONTRACT.experiment_id,
                    "run_id": run_dir.name,
                    "scenario": experiment.PRESSURE_CONTRACT.scenario,
                    "binary_sha256": manifest["binary"]["sha256"],
                    "frame_count": len(frames),
                    "pixel_encoding": "rgba8-tightly-packed",
                    "frames": frames,
                }
            ),
            encoding="utf-8",
        )

        terminal_samples = samples[-1 - experiment.TERMINAL_WINDOW_SAMPLES : -1]
        trend = experiment.pressure_terminal_trend(terminal_samples)
        predicates = {
            name: {"status": "pass", "detail": f"fixture {name}"}
            for name in experiment.PRESSURE_PREDICATE_NAMES
        }
        analysis = {
            "schema_version": experiment.PRESSURE_ANALYSIS_SCHEMA,
            "experiment_id": experiment.PRESSURE_CONTRACT.experiment_id,
            "run_id": run_dir.name,
            "scenario": experiment.PRESSURE_CONTRACT.scenario,
            "binary_sha256": manifest["binary"]["sha256"],
            "provenance": {
                "source_sha": manifest["source"]["sha"],
                "git_state": manifest["source"]["git_state"],
                "build_profile": "release",
            },
            "world": manifest["world"],
            "sleep": {"enabled": True, "threshold": 3},
            "lifecycle": {
                "max_ticks": experiment.MAX_TICKS,
                "diagnostic_interval_ticks": experiment.DIAGNOSTIC_INTERVAL,
                "consecutive_persistent_opening_samples": (
                    experiment.CONSECUTIVE_PERSISTENT_OPENING
                ),
                "post_opening_ticks": experiment.POST_OPENING_TICKS,
                "terminal_window_samples": experiment.TERMINAL_WINDOW_SAMPLES,
                "terminal_reason": "post-opening-observation-complete",
                "persistent_opening_start_sim_tick": 2,
                "persistent_opening_start_sample_sequence": tick2["sample_sequence"],
                "persistent_opening_confirmed_sim_tick": 16,
                "persistent_opening_confirmed_sample_sequence": tick16[
                    "sample_sequence"
                ],
                "post_opening_end_tick": final["sim_tick"],
                "sample_count": len(samples),
            },
            "baseline": {
                "initial_matter_count": tick0["matter_count"],
                "initial_water_count": tick0["water_count"],
                "initial_steam_count": tick0["steam_count"],
                "initial_relief_seam_wood_cells": 576,
                "initial_top_relief_seam_wood_cells": 384,
                "initial_bottom_relief_seam_wood_cells": 192,
                "initial_chamber_pressure_cell_count": 29_920,
                "initial_chamber_mean_pressure": 100.0,
                "initial_chamber_max_pressure": 180.0,
            },
            "metrics": {
                "first_pressure_activity_tick": 1,
                "first_pressure_activity_sample_sequence": tick1["sample_sequence"],
                "first_wood_damage_tick": 1,
                "first_wood_damage_sample_sequence": tick1["sample_sequence"],
                "first_rupture_tick": 2,
                "first_rupture_sample_sequence": tick2["sample_sequence"],
                "first_persistent_opening_tick": 2,
                "first_persistent_opening_sample_sequence": tick2["sample_sequence"],
                "persistent_opening_confirmed_tick": 16,
                "persistent_opening_confirmed_sample_sequence": tick16[
                    "sample_sequence"
                ],
                "first_steam_in_relief_seam_tick": first_vent["sim_tick"],
                "first_steam_in_relief_seam_sample_sequence": first_vent[
                    "sample_sequence"
                ],
                "first_outside_chamber_steam_tick": first_vent["sim_tick"],
                "first_outside_chamber_steam_sample_sequence": first_vent[
                    "sample_sequence"
                ],
                "first_post_confirmation_reseal_tick": None,
                "first_post_confirmation_reseal_sample_sequence": None,
                "first_post_opening_relief_tick": first_relief["sim_tick"],
                "first_post_opening_relief_sample_sequence": first_relief[
                    "sample_sequence"
                ],
                "first_relief_seam_combustion_tick": None,
                "first_relief_seam_combustion_sample_sequence": None,
                "first_relief_seam_fuel_progress_tick": None,
                "first_relief_seam_fuel_progress_sample_sequence": None,
                "peak_chamber_mean_pressure": 130.0,
                "peak_chamber_mean_pressure_tick": 2,
                "peak_chamber_mean_pressure_sample_sequence": tick2["sample_sequence"],
                "peak_chamber_max_pressure": 220.0,
                "peak_chamber_max_pressure_tick": 2,
                "peak_chamber_max_pressure_sample_sequence": tick2["sample_sequence"],
                "peak_pressure_active_cells": 1000,
                "peak_pressure_active_tick": 2,
                "peak_pressure_active_sample_sequence": tick2["sample_sequence"],
                "pre_opening_peak_chamber_mean_pressure": 130.0,
                "pre_opening_peak_chamber_max_pressure": 220.0,
                "vent_reference_chamber_mean_pressure": first_vent[
                    "chamber_mean_pressure"
                ],
                "vent_reference_chamber_max_pressure": first_vent[
                    "chamber_max_pressure"
                ],
                "post_opening_chamber_mean_pressure": tick16[
                    "chamber_mean_pressure"
                ],
                "post_opening_chamber_max_pressure": tick16[
                    "chamber_max_pressure"
                ],
                "terminal_chamber_mean_pressure": trend["end_mean_pressure"],
                "terminal_chamber_max_pressure": trend["end_max_pressure"],
                "terminal_pressure_relieved": True,
                "through_opening_confirmation_relief_seam_combusting_cells_peak": 0,
                "through_opening_confirmation_relief_seam_flame_event_cells_peak": 0,
                "through_opening_confirmation_relief_seam_fuel_progress_sum_peak": 0,
                "through_opening_confirmation_relief_seam_fuel_progress_max": 0,
                "opening_confirmation_relief_seam_combusting_cells": 0,
                "opening_confirmation_relief_seam_flame_event_cells": 0,
                "opening_confirmation_relief_seam_fuel_progress_sum": 0,
                "opening_confirmation_relief_seam_fuel_progress_max": 0,
                "opening_confirmation_relief_seam_adjacent_pressure_medium_cells": tick16[
                    "relief_seam_adjacent_pressure_medium_cells"
                ],
                "opening_confirmation_relief_seam_max_adjacent_pressure": tick16[
                    "relief_seam_max_adjacent_pressure"
                ],
                "opening_confirmation_adjacent_pressure_at_or_above_wood_rupture_threshold": True,
                "first_opening_relief_seam_adjacent_pressure_medium_cells": tick2[
                    "relief_seam_adjacent_pressure_medium_cells"
                ],
                "first_opening_relief_seam_max_adjacent_pressure": tick2[
                    "relief_seam_max_adjacent_pressure"
                ],
                "first_opening_adjacent_pressure_at_or_above_wood_rupture_threshold": True,
                "wood_rupture_threshold": 80.0,
                "final_relief_seam_wood_cells": final["relief_seam_wood_cells"],
                "final_top_relief_seam_wood_cells": final[
                    "top_relief_seam_wood_cells"
                ],
                "final_bottom_relief_seam_wood_cells": final[
                    "bottom_relief_seam_wood_cells"
                ],
                "final_relief_seam_open_cells": final["relief_seam_open_cells"],
                "final_top_relief_seam_open_cells": final[
                    "top_relief_seam_open_cells"
                ],
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
                "final_steam_in_relief_seam_cells": final[
                    "steam_in_relief_seam_cells"
                ],
                "outside_chamber_steam_peak": max(
                    item["outside_chamber_steam_cells"] for item in samples[:-1]
                ),
                "final_outside_chamber_steam_cells": final[
                    "outside_chamber_steam_cells"
                ],
                "final_matter_count": final["matter_count"],
                "matter_count_delta": final["matter_count"] - tick0["matter_count"],
                "final_water_count": final["water_count"],
                "water_count_delta": final["water_count"] - tick0["water_count"],
                "final_steam_count": final["steam_count"],
                "steam_count_delta": final["steam_count"] - tick0["steam_count"],
                "final_pressure_active_cells": final["census"][
                    "pressure_active_cells"
                ],
                "final_thermal_active_cells": 0,
                "final_reaction_active_cells": 0,
                "invalid_material_occurrences": 0,
                "nonfinite_field_occurrences": 0,
                "reset_exact_equivalence": True,
            },
            "terminal_window": trend,
            "review_flags": {
                "only_one_relief_seam_ruptured": False,
                "high_terminal_pressure_activity": False,
                "long_pressure_tail": False,
                "persistent_vent_plume": False,
                "terminal_activity_remains": False,
                "reasons": [],
            },
            "causal_classification": "pressure_opening_precedes_combustion",
            "predicates": predicates,
            "verdict": "PASS",
            "raw_frame_count": len(frames),
        }
        (run_dir / "work" / "analysis.json").write_text(
            json.dumps(analysis), encoding="utf-8"
        )
        return run_dir

    def make_pressure_fixture_combustion_confounded(
        self, run_dir: Path
    ) -> dict[str, object]:
        samples_path = run_dir / "telemetry" / "samples.jsonl"
        samples = experiment.read_jsonl(samples_path, "Pressure confound samples")
        confounded = samples[2]
        confounded.update(
            {
                "top_relief_seam_combusting_cells": 1,
                "relief_seam_combusting_cells": 1,
                "top_relief_seam_flame_event_cells": 1,
                "relief_seam_flame_event_cells": 1,
                "top_relief_seam_fuel_progress_sum": 900,
                "top_relief_seam_fuel_progress_max": 9,
                "relief_seam_fuel_progress_sum": 900,
                "relief_seam_fuel_progress_max": 9,
            }
        )
        samples_path.write_text(
            "".join(
                json.dumps(item, separators=(",", ":")) + "\n" for item in samples
            ),
            encoding="utf-8",
        )

        analysis_path = run_dir / "work" / "analysis.json"
        analysis = json.loads(analysis_path.read_text(encoding="utf-8"))
        analysis["metrics"].update(
            {
                "first_relief_seam_combustion_tick": confounded["sim_tick"],
                "first_relief_seam_combustion_sample_sequence": confounded[
                    "sample_sequence"
                ],
                "first_relief_seam_fuel_progress_tick": confounded["sim_tick"],
                "first_relief_seam_fuel_progress_sample_sequence": confounded[
                    "sample_sequence"
                ],
                "through_opening_confirmation_relief_seam_combusting_cells_peak": 1,
                "through_opening_confirmation_relief_seam_flame_event_cells_peak": 1,
                "through_opening_confirmation_relief_seam_fuel_progress_sum_peak": 900,
                "through_opening_confirmation_relief_seam_fuel_progress_max": 9,
            }
        )
        analysis["causal_classification"] = "fixture_causality_confounded"
        analysis["predicates"]["pressure_opening_precedes_combustion"][
            "status"
        ] = "fail"
        analysis["verdict"] = (
            experiment.PRESSURE_FIXTURE_CAUSALITY_CONFOUNDED_VERDICT
        )
        analysis_path.write_text(json.dumps(analysis), encoding="utf-8")

        events_path = run_dir / "telemetry" / "events.jsonl"
        events = experiment.read_jsonl(events_path, "Pressure confound events")
        rupture_index = next(
            index
            for index, event in enumerate(events)
            if event["event"] == "rupture_observed"
        )
        inserted = []
        for event_name in (
            "relief_seam_combustion_observed",
            "relief_seam_fuel_progress_observed",
        ):
            record = copy.deepcopy(events[rupture_index])
            record["event"] = event_name
            record["detail"] = "synthetic causal confound"
            inserted.append(record)
        events[rupture_index + 1 : rupture_index + 1] = inserted
        for sequence, event in enumerate(events):
            event["event_sequence"] = sequence
        events_path.write_text(
            "".join(
                json.dumps(item, separators=(",", ":")) + "\n" for item in events
            ),
            encoding="utf-8",
        )
        return analysis

    def create_pressure_sealed_delivery_fixture(
        self,
        run_id: str = "g8b-pressure-burst-v0-sealed",
        mode: str = "candidate",
    ) -> Path:
        self.initialize_source_repository()
        run_dir = self.create_valid_pressure_worker_fixture(run_id, mode=mode)
        binary = run_dir.joinpath(*experiment.FROZEN_BINARY_RELATIVE_PATH.parts)
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"frozen Pressure executable fixture")
        binary_hash = experiment.sha256_file(binary)
        actual_sha = experiment.git_text(self.source, "rev-parse", "HEAD")
        actual_branch = experiment.git_text(self.source, "branch", "--show-current")
        replacements = {
            "a" * 40: actual_sha,
            "feature/g8b-experiment-harness-v0": actual_branch,
            "b" * 64: binary_hash,
        }
        for path in run_dir.rglob("*"):
            if path.is_file() and path.suffix in {".json", ".jsonl", ".toml"}:
                text = path.read_text(encoding="utf-8")
                for old, new in replacements.items():
                    text = text.replace(old, new)
                path.write_text(text, encoding="utf-8")
        seal = experiment.capture_source_seal(self.source)
        experiment.write_new_text(
            run_dir / experiment.SOURCE_INPUT_MANIFEST_NAME,
            experiment.render_source_input_manifest(seal),
        )
        experiment.postprocess_run(run_dir)
        return run_dir

    def create_valid_heavy_worker_fixture(
        self,
        run_id: str = "g8b-heavy-mixed-v0-test-run",
        mode: str = "candidate",
    ) -> Path:
        run_dir, manifest = self.create_manifest(
            run_id, experiment.HEAVY_CONTRACT, mode
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

        tick0_counts = [30_000, 5_000, 5_000, 2_000, 6_000, 2_000, 3_000, 1_000, 500, 11_036]
        mixed_counts = list(tick0_counts)
        mixed_counts[4] -= 1
        mixed_counts[6] += 1
        mixed_counts[7] += 1
        mixed_counts[9] -= 1
        self.assertEqual(sum(tick0_counts), experiment.WORLD_WIDTH * experiment.WORLD_HEIGHT)
        ticks = [
            0,
            1,
            2,
            *range(
                experiment.DIAGNOSTIC_INTERVAL,
                experiment.MAX_TICKS + 1,
                experiment.DIAGNOSTIC_INTERVAL,
            ),
        ]
        sleep = {"enabled": True, "threshold": 8}
        world = manifest["world"]
        samples: list[dict] = []
        for sequence, tick in enumerate(ticks):
            counts = tick0_counts if tick == 0 else mixed_counts
            deltas = [
                count - baseline
                for count, baseline in zip(counts, tick0_counts, strict=True)
            ]
            if tick == 0:
                active = (0, 0, 0, 0)
            elif tick in {1, 2, 8}:
                active = (20, 20, 20, 20)
            elif tick == 16:
                # This zero arrives after first >=3 overlap and must not count in
                # zero_activity_before_overlap_samples.
                active = (0, 0, 0, 0)
            else:
                lane = sequence % 4
                active = tuple(20 if index == lane else 0 for index in range(4))
            any_active = max(active)
            census = {
                "total_cells": experiment.WORLD_WIDTH * experiment.WORLD_HEIGHT,
                "any_active_cells": any_active,
                "matter_active_cells": active[0],
                "thermal_active_cells": active[1],
                "pressure_active_cells": active[2],
                "reaction_active_cells": active[3],
                "total_chunks": 16,
                "active_chunks": 1 if any_active else 0,
                "runnable_chunks": 16,
                "sleeping_chunks": 0,
            }
            sample = {
                "schema_version": experiment.HEAVY_TELEMETRY_SCHEMA,
                "experiment_id": manifest["experiment_id"],
                "run_id": manifest["run_id"],
                "scenario": experiment.HEAVY_CONTRACT.scenario,
                "source_sha": manifest["source"]["sha"],
                "git_state": manifest["source"]["git_state"],
                "build_profile": "release",
                "binary_sha256": manifest["binary"]["sha256"],
                "sample_sequence": sequence,
                "sim_tick": tick,
                "phase": "initial" if tick == 0 else "mixed",
                "reason": (
                    "tick0"
                    if tick == 0
                    else "tick1"
                    if tick == 1
                    else "early-diagnostic"
                    if tick == 2
                    else "max-tick"
                    if tick == experiment.MAX_TICKS
                    else "diagnostic-cadence"
                ),
                "world": world,
                "sleep": sleep,
                "census": census,
                "subsystem_active_count": sum(value > 0 for value in active),
                "material_counts_by_id": list(counts),
                "matter_count": sum(counts[1:]),
                "sand_count": counts[3],
                "water_count": counts[4],
                "oil_count": counts[5],
                "wood_count": counts[9],
                "ice_count": counts[8],
                "steam_count": counts[6],
                "smoke_count": counts[7],
                "sand_position_changed_cells": 4 if tick == 1 else 0,
                "liquid_position_changed_cells": 4 if tick == 1 else 0,
                "water_oil_interface_edges": 2 if tick == 1 else 0,
                "density_ordered_pairs": 1 if tick == 1 else 0,
                "combusting_wood_cells": 8 if tick == 0 else 4,
                "combusting_oil_cells": 0,
                "flame_event_wood_cells": 0,
                "flame_event_oil_cells": 0,
                "wood_fuel_progress_sum": 0,
                "oil_fuel_progress_sum": 0 if tick == 0 else 1,
                "dynamic_combustion_work": tick > 0,
                "new_smoke_cells": 1 if tick == 1 else 0,
                "phase_inventory_changed": tick > 0,
                "relief_seam_wood_count": 224 if tick == 0 else 223,
                "relief_seam_combusting_cells": 0,
                "relief_seam_flame_event_cells": 0,
                "relief_seam_fuel_progress_sum": 0,
                "relief_seam_adjacent_pressure_medium_cells": 0 if tick == 0 else 1,
                "relief_seam_max_adjacent_pressure": 0.0 if tick == 0 else 80.0,
                "relief_open_lanes": 0,
                "exterior_steam_cells": 0,
                "temperature_min": 0.0,
                "temperature_max": 20.0 if tick == 0 else 100.0,
                "pressure_min": 0.0,
                "pressure_max": 0.0 if tick == 0 else 50.0,
                "phase_pool_count": counts[4] + counts[6] + counts[8],
                "fuel_count": counts[5] + counts[9],
                "material_count_deltas_by_id": deltas,
                "gross_inventory_delta_cells": sum(abs(value) for value in deltas) // 2,
                "explained_material_delta_cells": sum(abs(value) for value in deltas) // 2,
                "unexplained_material_delta_cells": 0,
                "inventory_accounted": True,
                "invalid_material_count": 0,
                "nonfinite_temperature_count": 0,
                "nonfinite_pressure_count": 0,
                "changed_chunks": 1 if tick else 0,
                "wake_chunks": 0,
                "wake_reason_or": 0,
                "wake_anomaly_chunks": 0,
                "state_hash": f"fnv1a64:{sequence:016x}",
                "physical_state_hash": f"fnv1a64:{sequence + 0x10000:016x}",
            }
            samples.append(sample)
        reset = copy.deepcopy(samples[0])
        reset["sample_sequence"] = len(samples)
        reset["phase"] = "reset"
        reset["reason"] = "programmatic-r-equivalent"
        samples.append(reset)
        tick0 = samples[0]
        terminal = samples[-2]

        milestone_badges = {
            1: [
                {"kind": "first-movement", "reason": "Sand-position-change"},
                {"kind": "first-density", "reason": "ordered-Water-Oil-displacement"},
                {"kind": "first-phase", "reason": "phase-inventory-change"},
                {"kind": "first-combustion", "reason": "post-tick-combustion-work"},
                {"kind": "first-smoke", "reason": "new-decay-age-zero-Smoke"},
                {"kind": "first-pressure", "reason": "Pressure-activity"},
                {
                    "kind": "first-rupture",
                    "reason": "pressure-threshold-noncombusting-relief-damage",
                },
                {"kind": "tick1", "reason": "after-one-production-tick"},
            ]
        }
        representative_targets = [2, 2_500, 5_000, 7_500, 10_000, 12_500, 15_000, 17_500]
        target_index = 0
        for sample in samples[1:-1]:
            badges = milestone_badges.setdefault(sample["sample_sequence"], [])
            if (
                target_index < len(representative_targets)
                and sample["sim_tick"] >= representative_targets[target_index]
            ):
                target = representative_targets[target_index]
                kind = (
                    "mid-run"
                    if target == experiment.MAX_TICKS // 2
                    else "late-run"
                    if target == experiment.MAX_TICKS * 3 // 4
                    else "representative"
                )
                reason = (
                    "representative-mid-run"
                    if kind == "mid-run"
                    else "representative-late-run"
                    if kind == "late-run"
                    else "scheduled-mixed-state"
                )
                badges.append({"kind": kind, "reason": reason})
                target_index += 1
            if sample is terminal:
                badges.append({"kind": "terminal", "reason": "max-tick-reached"})
            if not badges:
                del milestone_badges[sample["sample_sequence"]]

        expected_frames = experiment.heavy_expected_frames(
            samples, milestone_badges, samples[1], samples[1]
        )
        frames = []
        raw_size = experiment.RENDERER_WIDTH * experiment.RENDERER_HEIGHT * 4
        for ordinal, expected in enumerate(expected_frames):
            relative = f"work/frames/{ordinal:02}-{expected['kind']}.rgba"
            (run_dir / relative).write_bytes(bytes(raw_size))
            frames.append(
                {
                    "ordinal": ordinal,
                    "relative_path": relative,
                    "width": experiment.RENDERER_WIDTH,
                    "height": experiment.RENDERER_HEIGHT,
                    "rgba_bytes": raw_size,
                    **expected,
                }
            )

        firsts = {
            "first_movement": samples[1],
            "first_density_displacement": samples[1],
            "first_thermal_activity": samples[1],
            "first_phase_transition": samples[1],
            "first_combustion_work": samples[1],
            "first_smoke_generation": samples[1],
            "first_pressure_activity": samples[1],
            "first_relief_damage": samples[1],
            "first_rupture": samples[1],
            "first_opening": None,
            "first_vent": None,
            "first_three_subsystems": samples[1],
            "first_all_intended_subsystems": samples[1],
        }
        first_metrics = {
            f"{prefix}_{suffix}": (
                None
                if sample is None
                else sample["sim_tick"]
                if suffix == "tick"
                else sample["sample_sequence"]
            )
            for prefix, sample in firsts.items()
            for suffix in ("tick", "sample")
        }
        pre_reset = samples[:-1]
        subsystems = {
            name: experiment.heavy_subsystem_summary(pre_reset, key)
            for name, key in {
                "matter": "matter_active_cells",
                "thermal": "thermal_active_cells",
                "pressure": "pressure_active_cells",
                "reaction": "reaction_active_cells",
            }.items()
        }
        longest = experiment.heavy_longest_overlap(pre_reset)
        terminal_activity = {
            key: terminal["census"][key]
            for key in (
                "any_active_cells",
                "active_chunks",
                "runnable_chunks",
                "sleeping_chunks",
                "matter_active_cells",
                "thermal_active_cells",
                "pressure_active_cells",
                "reaction_active_cells",
            )
        }
        terminal_activity["subsystem_active_count"] = terminal["subsystem_active_count"]
        final_counts = terminal["material_counts_by_id"]
        metrics = {
            **first_metrics,
            "peak_active_cells": samples[1]["census"]["any_active_cells"],
            "peak_active_tick": 1,
            "peak_active_sample": 1,
            "peak_concurrent_subsystem_count": 4,
            "peak_concurrency_tick": 1,
            "peak_concurrency_sample": 1,
            **longest,
            "subsystems": subsystems,
            "initial_material_counts_by_id": tick0_counts,
            "final_material_counts_by_id": final_counts,
            "final_material_count_deltas_by_id": terminal["material_count_deltas_by_id"],
            "initial_matter": tick0["matter_count"],
            "final_matter": terminal["matter_count"],
            "matter_delta": terminal["matter_count"] - tick0["matter_count"],
            "initial_sand": tick0["sand_count"],
            "final_sand": terminal["sand_count"],
            "sand_delta": terminal["sand_count"] - tick0["sand_count"],
            "initial_water": tick0["water_count"],
            "final_water": terminal["water_count"],
            "water_delta": terminal["water_count"] - tick0["water_count"],
            "initial_oil": tick0["oil_count"],
            "final_oil": terminal["oil_count"],
            "oil_delta": terminal["oil_count"] - tick0["oil_count"],
            "initial_wood": tick0["wood_count"],
            "final_wood": terminal["wood_count"],
            "wood_delta": terminal["wood_count"] - tick0["wood_count"],
            "initial_ice": tick0["ice_count"],
            "final_ice": terminal["ice_count"],
            "ice_delta": terminal["ice_count"] - tick0["ice_count"],
            "initial_steam": tick0["steam_count"],
            "final_steam": terminal["steam_count"],
            "steam_delta": terminal["steam_count"] - tick0["steam_count"],
            "initial_smoke": tick0["smoke_count"],
            "smoke_peak": terminal["smoke_count"],
            "smoke_peak_tick": 1,
            "smoke_peak_sample": 1,
            "final_smoke": terminal["smoke_count"],
            "smoke_delta": terminal["smoke_count"] - tick0["smoke_count"],
            "initial_phase_pool": tick0["phase_pool_count"],
            "final_phase_pool": terminal["phase_pool_count"],
            "phase_pool_delta": terminal["phase_pool_count"] - tick0["phase_pool_count"],
            "initial_fuel": tick0["fuel_count"],
            "final_fuel": terminal["fuel_count"],
            "fuel_delta": terminal["fuel_count"] - tick0["fuel_count"],
            "gross_inventory_delta_cells": terminal["gross_inventory_delta_cells"],
            "explained_material_delta_cells": terminal["explained_material_delta_cells"],
            "unexplained_material_delta_cells": 0,
            "unexplained_material_delta_occurrences": 0,
            "terminal_activity": terminal_activity,
            "terminal_bounds": {
                key: terminal[key]
                for key in (
                    "temperature_min",
                    "temperature_max",
                    "pressure_min",
                    "pressure_max",
                )
            },
            "relief_seam_wood_final": terminal["relief_seam_wood_count"],
            "relief_open_lanes_final": 0,
            "exterior_steam_final": 0,
            "invalid_material_occurrences": 0,
            "nonfinite_field_occurrences": 0,
            "wake_anomaly_occurrences": 0,
            "zero_activity_before_overlap_samples": 0,
            "reset_exact_equivalence": True,
            "tick0_state_hash": tick0["state_hash"],
            "reset_state_hash": reset["state_hash"],
            "tick0_physical_state_hash": tick0["physical_state_hash"],
            "reset_physical_state_hash": reset["physical_state_hash"],
        }
        cumulative = [
            subsystems[name]["cumulative_active_cells"]
            for name in ("matter", "thermal", "pressure", "reaction")
        ]
        dominant_index = max(range(4), key=lambda index: (cumulative[index], index))
        dominant_share = cumulative[dominant_index] / sum(cumulative)
        review_flags = {
            "dominant_subsystem": False,
            "dominant_subsystem_name": ("matter", "thermal", "pressure", "reaction")[dominant_index],
            "dominant_subsystem_share": dominant_share,
            "broad_terminal_tail": False,
            "long_thermal_pressure_tail": False,
            "reasons": [],
        }
        baseline = {
            "material_counts_by_id": tick0_counts,
            "matter_count": tick0["matter_count"],
            "sand_count": tick0["sand_count"],
            "water_count": tick0["water_count"],
            "oil_count": tick0["oil_count"],
            "wood_count": tick0["wood_count"],
            "ice_count": tick0["ice_count"],
            "steam_count": tick0["steam_count"],
            "smoke_count": tick0["smoke_count"],
            "phase_pool_count": tick0["phase_pool_count"],
            "fuel_count": tick0["fuel_count"],
            "relief_seam_wood_count": tick0["relief_seam_wood_count"],
            "exterior_steam_cells": tick0["exterior_steam_cells"],
            "density_ordered_pairs": tick0["density_ordered_pairs"],
        }
        analysis = {
            "schema_version": experiment.HEAVY_ANALYSIS_SCHEMA,
            "experiment_id": manifest["experiment_id"],
            "run_id": manifest["run_id"],
            "scenario": experiment.HEAVY_CONTRACT.scenario,
            "source_sha": manifest["source"]["sha"],
            "git_state": manifest["source"]["git_state"],
            "build_profile": "release",
            "binary_sha256": manifest["binary"]["sha256"],
            "world": world,
            "sleep": sleep,
            "lifecycle": {
                "terminal_reason": "max-ticks",
                "terminal_tick": experiment.MAX_TICKS,
                "terminal_sample": terminal["sample_sequence"],
                "required_max_ticks": experiment.MAX_TICKS,
                "diagnostic_interval_ticks": experiment.DIAGNOSTIC_INTERVAL,
                "terminal_window_samples": experiment.HEAVY_TERMINAL_WINDOW_SAMPLES,
            },
            "baseline": baseline,
            "metrics": metrics,
            "terminal_trend": experiment.heavy_terminal_trend(
                pre_reset[1:][-experiment.HEAVY_TERMINAL_WINDOW_SAMPLES :]
            ),
            "predicates": {
                name: {"status": "pass", "detail": "synthetic valid fixture"}
                for name in experiment.HEAVY_PREDICATE_NAMES
            },
            "review_flags": review_flags,
            "verdict": "PASS",
            "sample_count": len(samples),
            "raw_frame_count": len(frames),
        }
        events_spec = [
            ("lifecycle_started", 0, None),
            ("pristine_reset_completed", 0, None),
            ("tick0_captured", 0, 0),
            ("first_movement_observed", 1, 1),
            ("first_density_displacement_observed", 1, 1),
            ("first_thermal_activity_observed", 1, 1),
            ("first_phase_transition_observed", 1, 1),
            ("first_combustion_work_observed", 1, 1),
            ("first_smoke_generation_observed", 1, 1),
            ("first_pressure_activity_observed", 1, 1),
            ("first_relief_damage_observed", 1, 1),
            ("first_rupture_observed", 1, 1),
            ("first_three_subsystems_observed", 1, 1),
            ("first_all_intended_subsystems_observed", 1, 1),
            ("new_peak_active", 1, 1),
            ("new_peak_concurrency", 1, 1),
            ("tick1_captured", 1, 1),
            ("terminal_selected", terminal["sim_tick"], terminal["sample_sequence"]),
            ("reset_started", terminal["sim_tick"], terminal["sample_sequence"]),
            ("reset_comparison_completed", 0, reset["sample_sequence"]),
            ("worker_completed", 0, reset["sample_sequence"]),
        ]
        events = [
            {
                "schema_version": experiment.HEAVY_TELEMETRY_SCHEMA,
                "experiment_id": manifest["experiment_id"],
                "run_id": manifest["run_id"],
                "scenario": experiment.HEAVY_CONTRACT.scenario,
                "event_sequence": sequence,
                "event": name,
                "sim_tick": tick,
                "sample_sequence": sample_sequence,
                "detail": "synthetic valid fixture",
            }
            for sequence, (name, tick, sample_sequence) in enumerate(events_spec)
        ]
        (run_dir / "work" / "analysis.json").write_text(
            json.dumps(analysis, separators=(",", ":")), encoding="utf-8"
        )
        (run_dir / "work" / "frames.json").write_text(
            json.dumps(
                {
                    "schema_version": experiment.HEAVY_FRAMES_SCHEMA,
                    "experiment_id": manifest["experiment_id"],
                    "run_id": manifest["run_id"],
                    "scenario": experiment.HEAVY_CONTRACT.scenario,
                    "binary_sha256": manifest["binary"]["sha256"],
                    "frame_count": len(frames),
                    "pixel_encoding": "rgba8-tightly-packed",
                    "frames": frames,
                },
                separators=(",", ":"),
            ),
            encoding="utf-8",
        )
        (telemetry / "samples.jsonl").write_text(
            "".join(json.dumps(sample, separators=(",", ":")) + "\n" for sample in samples),
            encoding="utf-8",
        )
        (telemetry / "events.jsonl").write_text(
            "".join(json.dumps(event, separators=(",", ":")) + "\n" for event in events),
            encoding="utf-8",
        )
        return run_dir

    def create_heavy_sealed_delivery_fixture(
        self,
        run_id: str = "g8b-heavy-mixed-v0-sealed",
        mode: str = "candidate",
    ) -> Path:
        self.initialize_source_repository()
        run_dir = self.create_valid_heavy_worker_fixture(run_id, mode=mode)
        binary = run_dir.joinpath(*experiment.FROZEN_BINARY_RELATIVE_PATH.parts)
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"frozen Heavy Mixed executable fixture")
        binary_hash = experiment.sha256_file(binary)
        actual_sha = experiment.git_text(self.source, "rev-parse", "HEAD")
        actual_branch = experiment.git_text(self.source, "branch", "--show-current")
        replacements = {
            "a" * 40: actual_sha,
            "feature/g8b-experiment-harness-v0": actual_branch,
            "b" * 64: binary_hash,
        }
        for path in run_dir.rglob("*"):
            if path.is_file() and path.suffix in {".json", ".jsonl", ".toml"}:
                text = path.read_text(encoding="utf-8")
                for old, new in replacements.items():
                    text = text.replace(old, new)
                path.write_text(text, encoding="utf-8")
        seal = experiment.capture_source_seal(self.source)
        experiment.write_new_text(
            run_dir / experiment.SOURCE_INPUT_MANIFEST_NAME,
            experiment.render_source_input_manifest(seal),
        )
        experiment.postprocess_run(run_dir)
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

    def test_source_input_manifest_is_tracked_deterministic_and_detects_drift(self) -> None:
        self.initialize_source_repository()
        seal = experiment.capture_source_seal(self.source)
        paths = [entry["path"] for entry in seal.manifest["files"]]
        self.assertEqual(paths, sorted(paths))
        self.assertIn("Cargo.toml", paths)
        self.assertIn("Cargo.lock", paths)
        self.assertIn("apps/windows/build.rs", paths)
        self.assertIn("apps/scenarios/src/fixture.rs", paths)
        self.assertIn("engine/gpu/src/test.wgsl", paths)
        self.assertIn("tools/experiment/run_experiment.py", paths)
        self.assertNotIn("docs/ignored.md", paths)
        self.assertEqual(seal.manifest["file_count"], len(paths))
        self.assertEqual(seal.manifest["external_file_count"], 1)
        self.assertEqual(
            seal.manifest["external_files"],
            [
                {
                    "label": "windows-consolas-font",
                    "path": str(self.external_font.resolve()),
                    "sha256": experiment.sha256_file(self.external_font),
                    "size_bytes": self.external_font.stat().st_size,
                }
            ],
        )

        manifest_path = self.artifacts / experiment.SOURCE_INPUT_MANIFEST_NAME
        experiment.write_new_text(
            manifest_path, experiment.render_source_input_manifest(seal)
        )
        experiment.assert_source_manifest_artifact_unchanged(manifest_path, seal)
        (self.source / "apps/windows/src/main.rs").write_text(
            "fn main() { panic!(\"drift\") }\n", encoding="utf-8"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "post-build"):
            experiment.assert_source_seal_unchanged(
                self.source, seal, "post-build"
            )

    def test_dirty_tracked_scratch_seal_is_exact_and_rejects_input_drift(self) -> None:
        self.initialize_source_repository()
        selected = self.source / "apps" / "windows" / "src" / "main.rs"
        selected.write_text("fn main() { println!(\"dirty scratch\"); }\n", encoding="utf-8")

        with self.assertRaisesRegex(experiment.ExperimentError, "must be clean"):
            experiment.capture_source_seal(self.source)
        seal = experiment.capture_source_seal(
            self.source, allow_dirty_tracked=True
        )
        self.assertEqual(seal.source.git_state, "dirty")
        self.assertTrue(experiment.HEX64.fullmatch(seal.source.tracked_state_sha256))
        self.assertEqual(seal.manifest["source"]["git_state"], "dirty")
        selected_entry = next(
            entry
            for entry in seal.manifest["files"]
            if entry["path"] == "apps/windows/src/main.rs"
        )
        self.assertEqual(selected_entry["sha256"], experiment.sha256_file(selected))

        selected.write_text(
            "fn main() { println!(\"drift after seal\"); }\n", encoding="utf-8"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "pre-worker"):
            experiment.assert_source_seal_unchanged(
                self.source, seal, "pre-worker"
            )

    def test_dirty_scratch_manifest_and_pressure_telemetry_are_accepted(self) -> None:
        run_dir = self.create_valid_pressure_worker_fixture(
            "g8b-pressure-burst-v0-dirty-scratch",
            mode="scratch",
            git_state="dirty",
        )
        manifest = experiment.read_and_validate_manifest(
            run_dir / "EXPERIMENT_MANIFEST.toml"
        )
        self.assertEqual(manifest["run_mode"], "scratch")
        self.assertEqual(manifest["source"]["git_state"], "dirty")
        analysis, _, samples, _ = experiment.validate_telemetry(run_dir, manifest)
        self.assertEqual(analysis["provenance"]["git_state"], "dirty")
        self.assertTrue(all(sample["git_state"] == "dirty" for sample in samples))
        experiment.postprocess_run(run_dir)
        report_markdown = (run_dir / "report" / "REPORT.md").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            f"- Source: `{manifest['source']['sha']}` on "
            f"`{manifest['source']['branch']}` (`dirty`)",
            report_markdown,
        )
        self.assertIn(
            "Causal vent milestones are confirmation-gated",
            report_markdown,
        )

    def test_dirty_candidate_manifest_is_rejected(self) -> None:
        run_dir = self.artifacts / "g8b-pressure-burst-v0-dirty-candidate"
        run_dir.mkdir(parents=True)
        manifest = self.manifest_data(
            run_dir,
            experiment.PRESSURE_CONTRACT,
            "candidate",
            "dirty",
        ).as_dict()
        with self.assertRaisesRegex(
            experiment.ExperimentError, "dirty source is allowed only for scratch"
        ):
            experiment.validate_manifest_dict(manifest)

    def test_source_seal_detects_external_font_drift_and_missing_input(self) -> None:
        self.initialize_source_repository()
        seal = experiment.capture_source_seal(self.source)
        self.external_font.write_bytes(b"mutated Consolas fixture bytes")
        with self.assertRaisesRegex(experiment.ExperimentError, "pre-worker"):
            experiment.assert_source_seal_unchanged(
                self.source, seal, "pre-worker"
            )
        self.external_font.unlink()
        with self.assertRaisesRegex(experiment.ExperimentError, "missing"):
            experiment.capture_source_seal(self.source)

    def test_frozen_binary_is_create_new_hashed_and_rechecked(self) -> None:
        release = self.source / "target" / "release" / "powdergame-windows.exe"
        release.parent.mkdir(parents=True)
        release.write_bytes(b"release executable bytes")
        run_dir = experiment.create_run_directory(self.artifacts, "binary-freeze-test")
        frozen, digest = experiment.copy_frozen_binary(release, run_dir)
        self.assertEqual(
            frozen,
            run_dir.joinpath(*experiment.FROZEN_BINARY_RELATIVE_PATH.parts).resolve(),
        )
        self.assertEqual(frozen.read_bytes(), release.read_bytes())
        self.assertEqual(digest, experiment.sha256_file(frozen))
        experiment.assert_frozen_binary_unchanged(frozen, digest, "worker-launch")
        with self.assertRaisesRegex(experiment.ExperimentError, "overwrite"):
            experiment.copy_frozen_binary(release, run_dir)
        frozen.write_bytes(b"mutated")
        with self.assertRaisesRegex(experiment.ExperimentError, "post-worker"):
            experiment.assert_frozen_binary_unchanged(frozen, digest, "post-worker")

    def test_final_source_guard_failure_leaves_run_without_receipt(self) -> None:
        run_dir = self.create_valid_worker_fixture("g8b-sand-fall-v0-guard-test")

        def reject_drift() -> None:
            raise experiment.ExperimentError("source drift at pre-receipt")

        with self.assertRaisesRegex(experiment.ExperimentError, "source drift"):
            experiment.postprocess_run(run_dir, final_guard=reject_drift)
        self.assertFalse((run_dir / "EXPERIMENT_RECEIPT.json").exists())

    def test_candidate_audit_bundle_is_sibling_complete_and_hashed(self) -> None:
        run_dir = self.create_sealed_delivery_fixture(
            "g8b-fire-heat-v0-audit-bundle-test"
        )
        before = {
            path.relative_to(run_dir).as_posix(): experiment.sha256_file(path)
            for path in run_dir.rglob("*")
            if path.is_file()
        }
        receipt_sha256 = experiment.sha256_file(
            run_dir / "EXPERIMENT_RECEIPT.json"
        )
        with mock.patch.object(
            experiment, "git_archive_zip_bytes", return_value=b"git archive fixture"
        ):
            bundle, sidecar = experiment.create_audit_bundle(
                run_dir, self.source, receipt_sha256
            )
        after = {
            path.relative_to(run_dir).as_posix(): experiment.sha256_file(path)
            for path in run_dir.rglob("*")
            if path.is_file()
        }
        self.assertEqual(before, after, "sibling bundle must not modify completed run")
        self.assertEqual(bundle.parent, run_dir.parent)
        self.assertEqual(sidecar.parent, run_dir.parent)
        with zipfile.ZipFile(bundle) as archive:
            self.assertEqual(
                set(archive.namelist()),
                {
                    "REVIEW_PACKET.zip",
                    "EXPERIMENT_MANIFEST.toml",
                    "HASHES.sha256",
                    "EXPERIMENT_RECEIPT.json",
                    experiment.SOURCE_INPUT_MANIFEST_NAME,
                    experiment.FROZEN_BINARY_RELATIVE_PATH.as_posix(),
                    "SOURCE_ARCHIVE.zip",
                },
            )
        self.assertEqual(
            sidecar.read_text(encoding="utf-8"),
            f"{experiment.sha256_file(bundle)}  {bundle.name}\n",
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "overwrite"):
            experiment.create_audit_bundle(run_dir, self.source, receipt_sha256)

    def test_audit_bundle_rejects_review_packet_mutation_after_receipt(self) -> None:
        run_dir = self.create_sealed_delivery_fixture(
            "g8b-fire-heat-v0-audit-mutation-test"
        )
        receipt_sha256 = experiment.sha256_file(
            run_dir / "EXPERIMENT_RECEIPT.json"
        )
        packet = run_dir / "report" / "REVIEW_PACKET.zip"
        with packet.open("ab") as handle:
            handle.write(b"post-receipt mutation")
        bundle = run_dir.parent / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SUFFIX}"
        sidecar = (
            run_dir.parent
            / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SHA256_SUFFIX}"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "digest mismatch"):
            experiment.create_audit_bundle(run_dir, self.source, receipt_sha256)
        self.assertFalse(bundle.exists())
        self.assertFalse(sidecar.exists())

    def test_audit_bundle_rejects_receipt_verdict_mutation_after_postprocess(self) -> None:
        run_dir = self.create_sealed_delivery_fixture(
            "g8b-fire-heat-v0-audit-receipt-verdict-test"
        )
        receipt_path = run_dir / "EXPERIMENT_RECEIPT.json"
        receipt_sha256 = experiment.sha256_file(receipt_path)
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        receipt["automatic_verdict"] = "FAIL"
        receipt_path.write_text(
            json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        bundle = run_dir.parent / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SUFFIX}"
        sidecar = (
            run_dir.parent
            / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SHA256_SUFFIX}"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "receipt SHA-256 changed"):
            experiment.create_audit_bundle(run_dir, self.source, receipt_sha256)
        self.assertFalse(bundle.exists())
        self.assertFalse(sidecar.exists())

    def test_audit_bundle_rejects_valid_receipt_timestamp_byte_mutation(self) -> None:
        run_dir = self.create_sealed_delivery_fixture(
            "g8b-fire-heat-v0-audit-receipt-time-test"
        )
        receipt_path = run_dir / "EXPERIMENT_RECEIPT.json"
        receipt_sha256 = experiment.sha256_file(receipt_path)
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        receipt["completed_utc"] = "2026-08-17T23:59:59.123456Z"
        receipt_path.write_text(
            json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        bundle = run_dir.parent / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SUFFIX}"
        sidecar = (
            run_dir.parent
            / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SHA256_SUFFIX}"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "receipt SHA-256 changed"):
            experiment.create_audit_bundle(run_dir, self.source, receipt_sha256)
        self.assertFalse(bundle.exists())
        self.assertFalse(sidecar.exists())

    def test_audit_bundle_rejects_numeric_receipt_final_marker(self) -> None:
        run_dir = self.create_sealed_delivery_fixture(
            "g8b-fire-heat-v0-audit-receipt-marker-type-test"
        )
        receipt_path = run_dir / "EXPERIMENT_RECEIPT.json"
        receipt_sha256 = experiment.sha256_file(receipt_path)
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        receipt["receipt_is_final_publication_marker"] = 1
        receipt_path.write_text(
            json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        bundle = run_dir.parent / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SUFFIX}"
        sidecar = (
            run_dir.parent
            / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SHA256_SUFFIX}"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "receipt SHA-256 changed"):
            experiment.create_audit_bundle(run_dir, self.source, receipt_sha256)
        self.assertFalse(bundle.exists())
        self.assertFalse(sidecar.exists())

    def test_audit_bundle_rejects_inventory_and_receipt_binding_mismatches(self) -> None:
        extra_dir = self.create_sealed_delivery_fixture(
            "g8b-fire-heat-v0-audit-extra-file-test"
        )
        (extra_dir / "unhashed-after-receipt.txt").write_text(
            "not in HASHES.sha256\n", encoding="utf-8"
        )
        extra_receipt_sha256 = experiment.sha256_file(
            extra_dir / "EXPERIMENT_RECEIPT.json"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "inventory mismatch"):
            experiment.create_audit_bundle(
                extra_dir, self.source, extra_receipt_sha256
            )

        binding_dir = self.create_sealed_delivery_fixture(
            "g8b-fire-heat-v0-audit-binding-test"
        )
        receipt_path = binding_dir / "EXPERIMENT_RECEIPT.json"
        binding_receipt_sha256 = experiment.sha256_file(receipt_path)
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        receipt["manifest_sha256"] = "0" * 64
        receipt_path.write_text(json.dumps(receipt, sort_keys=True) + "\n", encoding="utf-8")
        with self.assertRaisesRegex(experiment.ExperimentError, "receipt SHA-256 changed"):
            experiment.create_audit_bundle(
                binding_dir, self.source, binding_receipt_sha256
            )

    def test_scratch_run_is_not_eligible_for_audit_bundle(self) -> None:
        run_dir = self.create_sealed_delivery_fixture(
            "g8b-fire-heat-v0-scratch-audit-test", mode="scratch"
        )
        receipt_sha256 = experiment.sha256_file(
            run_dir / "EXPERIMENT_RECEIPT.json"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "candidate-only"):
            experiment.create_audit_bundle(run_dir, self.source, receipt_sha256)
        self.assertFalse(
            (run_dir.parent / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SUFFIX}").exists()
        )

    def test_runner_executes_frozen_copy_and_rechecks_each_seal_phase(self) -> None:
        release = self.source / "target" / "release" / "powdergame-windows.exe"
        release.parent.mkdir(parents=True)
        release.write_bytes(b"coordinator release binary")
        source = experiment.SourceInfo(
            root=self.source.resolve(),
            branch="feature/m0-g8b-scenario-suite",
            sha="a" * 40,
        )
        seal = experiment.SourceSeal(
            source=source,
            manifest={
                "schema_version": experiment.SOURCE_INPUT_MANIFEST_SCHEMA,
                "source": {
                    "root": str(source.root),
                    "branch": source.branch,
                    "head_sha": source.sha,
                    "git_state": "clean",
                },
                "selection": {"tracked_only": True, "rules": []},
                "file_count": 0,
                "files": [],
            },
        )
        commands: list[tuple[str, ...]] = []

        def fake_run_logged(command, cwd, stdout_path, stderr_path):
            del cwd, stdout_path, stderr_path
            command = tuple(command)
            commands.append(command)
            if command[0] != "cargo":
                self.assertEqual(
                    Path(command[0]).resolve(),
                    (self.artifacts / "sealed-run").joinpath(
                        *experiment.FROZEN_BINARY_RELATIVE_PATH.parts
                    ).resolve(),
                )
            return 0

        def fake_postprocess(run_dir, publication_log=None, final_guard=None):
            del publication_log
            self.assertIsNotNone(final_guard)
            final_guard()
            receipt = run_dir / "EXPERIMENT_RECEIPT.json"
            receipt.write_text("{}\n", encoding="utf-8")
            return receipt

        with (
            mock.patch.object(experiment, "capture_source_seal", return_value=seal),
            mock.patch.object(experiment, "generate_run_id", return_value="sealed-run"),
            mock.patch.object(experiment, "run_logged", side_effect=fake_run_logged),
            mock.patch.object(experiment, "assert_source_seal_unchanged") as source_guard,
            mock.patch.object(experiment, "postprocess_run", side_effect=fake_postprocess),
            mock.patch.object(experiment, "create_audit_bundle") as create_bundle,
        ):
            receipt = experiment.run_experiment(
                self.source, self.artifacts, "fire-heat", mode="candidate"
            )
        self.assertEqual(receipt, self.artifacts / "sealed-run" / "EXPERIMENT_RECEIPT.json")
        self.assertEqual(len(commands), 2)
        self.assertEqual(commands[0][0], "cargo")
        self.assertEqual(
            [call.args[2] for call in source_guard.call_args_list],
            ["post-build", "pre-worker", "worker-launch", "post-worker", "pre-receipt"],
        )
        create_bundle.assert_called_once_with(
            self.artifacts / "sealed-run",
            self.source.resolve(),
            experiment.sha256_file(receipt),
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
        self.assertIs(
            experiment.contract_for_scenario("pressure-burst"),
            experiment.PRESSURE_CONTRACT,
        )
        self.assertIs(
            experiment.contract_for_scenario("heavy-mixed"),
            experiment.HEAVY_CONTRACT,
        )
        heavy = experiment.worker_command(
            binary,
            run_dir,
            "run-heavy",
            "e" * 64,
            contract=experiment.HEAVY_CONTRACT,
        )
        self.assertEqual(
            heavy,
            (
                str(binary),
                "--experiment-worker",
                "heavy-mixed",
                "--experiment-run-dir",
                str(run_dir),
                "--experiment-run-id",
                "run-heavy",
                "--binary-sha256",
                "e" * 64,
                "--max-ticks",
                "20000",
                "--diagnostic-interval",
                "8",
            ),
        )
        with self.assertRaises(experiment.ExperimentError):
            experiment.contract_for_scenario("unknown-experiment")

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

    def test_fire_manifest_and_worker_command_are_scenario_specific(self) -> None:
        run_dir, manifest = self.create_manifest(
            "g8b-fire-heat-v0-manifest-test", experiment.FIRE_CONTRACT
        )
        self.assertEqual(manifest["schema_version"], experiment.FIRE_MANIFEST_SCHEMA)
        self.assertEqual(manifest["run_mode"], "candidate")
        self.assertEqual(
            manifest["experiment"],
            {
                "max_ticks": 20_000,
                "diagnostic_interval_ticks": 8,
                "consecutive_reaction_zero": 3,
                "post_reaction_ticks": 180,
            },
        )
        command = manifest["commands"]["worker"]
        expected_binary = run_dir.joinpath(
            *experiment.FROZEN_BINARY_RELATIVE_PATH.parts
        ).resolve()
        self.assertEqual(command[0], str(expected_binary))
        self.assertIn("fire-heat", command)
        self.assertEqual(command[-4:], ["--consecutive-reaction-zero", "3", "--post-reaction-ticks", "180"])
        self.assertNotIn("--consecutive-all-sleep", command)
        self.assertNotIn("--post-sleep-ticks", command)
        legacy_binary = (
            self.source / "target" / "release" / "powdergame-windows.exe"
        ).resolve()
        legacy = replace(
            self.manifest_data(run_dir, experiment.FIRE_CONTRACT),
            binary_path=legacy_binary,
            worker_command=experiment.worker_command(
                legacy_binary,
                run_dir.resolve(),
                run_dir.name,
                "b" * 64,
                contract=experiment.FIRE_CONTRACT,
            ),
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "run-local frozen"):
            experiment.validate_manifest_dict(legacy.as_dict())
        scratch = experiment.generate_run_id(
            contract=experiment.FIRE_CONTRACT, run_mode="scratch"
        )
        self.assertIn("g8b-fire-heat-v0-scratch-", scratch)
        self.assertEqual(run_dir.name, manifest["run_id"])

    def test_fire_telemetry_is_independently_recomputed(self) -> None:
        run_dir = self.create_valid_fire_worker_fixture()
        manifest = experiment.read_and_validate_manifest(run_dir / "EXPERIMENT_MANIFEST.toml")
        analysis, frames, samples, events = experiment.validate_telemetry(run_dir, manifest)
        self.assertEqual(analysis["verdict"], "PASS")
        self.assertEqual(analysis["metrics"]["first_combustion_tick"], 1)
        self.assertEqual(analysis["metrics"]["first_smoke_tick"], 1)
        self.assertEqual(analysis["metrics"]["first_phase_transition_tick"], 2)
        self.assertEqual(analysis["metrics"]["reaction_zero_tick"], 8)
        self.assertEqual(analysis["metrics"]["confirmed_reaction_zero_tick"], 24)
        self.assertEqual(analysis["metrics"]["peak_smoke_count"], 25)
        self.assertEqual(analysis["metrics"]["post_reaction_thermal_cells"], 200)
        self.assertEqual(analysis["metrics"]["post_reaction_final_thermal_cells"], 20)
        self.assertEqual(frames["frame_count"], 12)
        self.assertEqual(len(samples), 187)
        self.assertEqual(len(events), 20)

        analysis_path = run_dir / "work" / "analysis.json"
        mutated = copy.deepcopy(analysis)
        mutated["metrics"]["peak_smoke_count"] = 24
        analysis_path.write_text(json.dumps(mutated), encoding="utf-8")
        with self.assertRaisesRegex(experiment.ExperimentError, "peak_smoke_count"):
            experiment.validate_telemetry(run_dir, manifest)

    def test_fire_rejects_reaction_zero_and_tail_mutations(self) -> None:
        reaction_dir = self.create_valid_fire_worker_fixture(
            "g8b-fire-heat-v0-reaction-mutation"
        )
        manifest = experiment.read_and_validate_manifest(
            reaction_dir / "EXPERIMENT_MANIFEST.toml"
        )
        samples_path = reaction_dir / "telemetry" / "samples.jsonl"
        samples = [
            json.loads(line) for line in samples_path.read_text(encoding="utf-8").splitlines()
        ]
        tick16 = next(item for item in samples if item["sim_tick"] == 16)
        tick16["census"]["reaction_active_cells"] = 1
        tick16["census"]["matter_active_cells"] = 1
        samples_path.write_text(
            "".join(json.dumps(item, separators=(",", ":")) + "\n" for item in samples),
            encoding="utf-8",
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "reaction-zero"):
            experiment.validate_telemetry(reaction_dir, manifest)

        tail_dir = self.create_valid_fire_worker_fixture("g8b-fire-heat-v0-tail-mutation")
        tail_manifest = experiment.read_and_validate_manifest(
            tail_dir / "EXPERIMENT_MANIFEST.toml"
        )
        analysis_path = tail_dir / "work" / "analysis.json"
        tail_analysis = json.loads(analysis_path.read_text(encoding="utf-8"))
        tail_analysis["metrics"]["post_reaction_min_thermal_cells"] = 21
        analysis_path.write_text(json.dumps(tail_analysis), encoding="utf-8")
        with self.assertRaisesRegex(experiment.ExperimentError, "post_reaction_min"):
            experiment.validate_telemetry(tail_dir, tail_manifest)

    def test_fire_contact_sheet_caption_is_compact_and_direct(self) -> None:
        item = {
            "ordinal": 4,
            "reason": "highest-observed-reaction-cells",
            "sim_tick": 2,
            "sample_sequence": 2,
            "state_hash": "fnv1a64:0123456789abcdef",
        }
        sample = {
            "scenario": "fire-heat",
            "sample_sequence": 2,
            "state_hash": "fnv1a64:0123456789abcdef",
            "wood_count": 10000,
            "oil_count": 1500,
            "smoke_count": 10,
            "ice_count": 2230,
            "water_count": 1540,
            "steam_count": 6,
            "census": {"reaction_active_cells": 120, "thermal_active_cells": 250},
        }
        self.assertEqual(
            experiment.contact_sheet_caption_lines(item, sample),
            (
                "#4 highest-observed-reaction-cells | sim 2 | sample 2",
                "Reaction 120 | Thermal 250 | Wood 10000 | Oil 1500",
                "Smoke 10 | Ice/Water/Steam 2230/1540/6",
                "State fnv1a64:0123456789abcdef",
            ),
        )

    def test_fire_packet_and_receipt_remain_create_new_and_receipt_last(self) -> None:
        run_dir = self.create_valid_fire_worker_fixture("g8b-fire-heat-v0-packet-test")
        publication_log: list[str] = []
        receipt_path = experiment.postprocess_run(run_dir, publication_log)
        self.assertEqual(publication_log[-1], "EXPERIMENT_RECEIPT.json")
        report = json.loads((run_dir / "report" / "REPORT.json").read_text(encoding="utf-8"))
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        self.assertEqual(report["schema_version"], experiment.FIRE_REPORT_SCHEMA)
        self.assertEqual(receipt["schema_version"], experiment.FIRE_RECEIPT_SCHEMA)
        self.assertEqual(report["run_mode"], "candidate")
        self.assertEqual(receipt["run_mode"], "candidate")
        self.assertEqual(report["fire_heat"], receipt["fire_heat"])
        self.assertEqual(report["fire_heat"]["peak_smoke_count"], 25)
        self.assertTrue(report["scope"]["fire_heat"])
        self.assertFalse(report["review_guidance"]["g8b_closed"])
        prompt = (run_dir / "report" / "CHATGPT_REVIEW_PROMPT.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("was not sent to an AI", prompt)
        self.assertIn("remaining Thermal tail is not itself a failure", prompt)
        packet = run_dir / "report" / "REVIEW_PACKET.zip"
        with zipfile.ZipFile(packet) as archive:
            names = set(archive.namelist())
        self.assertIn("report/CONTACT_SHEET.png", names)
        self.assertIn("telemetry/samples.jsonl", names)
        self.assertNotIn("EXPERIMENT_RECEIPT.json", names)
        self.assertEqual(receipt["review_packet_sha256"], experiment.sha256_file(packet))
        with self.assertRaisesRegex(experiment.ExperimentError, "receipt"):
            experiment.postprocess_run(run_dir)

    def test_pressure_manifest_worker_command_and_old_schema_isolation(self) -> None:
        run_dir, manifest = self.create_manifest(
            "g8b-pressure-burst-v0-manifest-test", experiment.PRESSURE_CONTRACT
        )
        self.assertEqual(manifest["schema_version"], experiment.PRESSURE_MANIFEST_SCHEMA)
        self.assertEqual(
            manifest["experiment"],
            {
                "max_ticks": 20_000,
                "diagnostic_interval_ticks": 8,
                "consecutive_persistent_opening": 3,
                "post_opening_ticks": 180,
                "terminal_window_samples": 64,
            },
        )
        command = manifest["commands"]["worker"]
        self.assertEqual(
            command[-6:],
            [
                "--consecutive-persistent-opening",
                "3",
                "--post-opening-ticks",
                "180",
                "--terminal-window-samples",
                "64",
            ],
        )
        for forbidden in (
            "--consecutive-all-sleep",
            "--post-sleep-ticks",
            "--consecutive-reaction-zero",
            "--post-reaction-ticks",
        ):
            self.assertNotIn(forbidden, command)
        for contract in (
            experiment.SAND_CONTRACT,
            experiment.WATER_CONTRACT,
            experiment.FIRE_CONTRACT,
        ):
            _, legacy_manifest = self.create_manifest(
                f"{contract.experiment_id}-pressure-isolation", contract
            )
            self.assertTrue(
                {
                    "consecutive_persistent_opening",
                    "post_opening_ticks",
                    "terminal_window_samples",
                }.isdisjoint(legacy_manifest["experiment"])
            )
            legacy = experiment.worker_command(
                Path("legacy.exe"),
                Path("run"),
                "legacy-run",
                "a" * 64,
                contract=contract,
            )
            for pressure_only in (
                "--consecutive-persistent-opening",
                "--post-opening-ticks",
                "--terminal-window-samples",
            ):
                self.assertNotIn(pressure_only, legacy)
        self.assertEqual(
            Path(manifest["binary"]["path"]),
            run_dir.joinpath(*experiment.FROZEN_BINARY_RELATIVE_PATH.parts).resolve(),
        )
        legacy_runs = (
            self.create_valid_worker_fixture(
                "g8b-sand-fall-v0-pressure-through-lane-isolation"
            ),
            self.create_valid_water_worker_fixture(
                "g8b-water-flow-v0-pressure-through-lane-isolation"
            ),
            self.create_valid_fire_worker_fixture(
                "g8b-fire-heat-v0-pressure-through-lane-isolation"
            ),
        )
        for legacy_run in legacy_runs:
            legacy_sample = experiment.read_jsonl(
                legacy_run / "telemetry" / "samples.jsonl", "legacy samples"
            )[0]
            for pressure_only_field in (
                "relief_seam_through_open_lanes",
                "top_relief_seam_through_open_lanes",
                "bottom_relief_seam_through_open_lanes",
            ):
                self.assertNotIn(pressure_only_field, legacy_sample)

    def test_pressure_telemetry_contact_and_folded_frames_are_exact(self) -> None:
        run_dir = self.create_valid_pressure_worker_fixture()
        manifest = experiment.read_and_validate_manifest(
            run_dir / "EXPERIMENT_MANIFEST.toml"
        )
        analysis, frames_doc, samples, events = experiment.validate_telemetry(
            run_dir, manifest
        )
        self.assertEqual(analysis["verdict"], "PASS")
        self.assertEqual(
            analysis["causal_classification"],
            "pressure_opening_precedes_combustion",
        )
        self.assertEqual(
            analysis["predicates"]["pressure_opening_precedes_combustion"][
                "status"
            ],
            "pass",
        )
        self.assertEqual(
            analysis["schema_version"], "powdergame-pressure-burst-analysis-v1"
        )
        self.assertEqual(
            frames_doc["schema_version"], "powdergame-pressure-burst-frames-v0"
        )
        self.assertTrue(
            all(
                sample["schema_version"]
                == "powdergame-pressure-burst-telemetry-v1"
                for sample in samples
            )
        )
        self.assertEqual(analysis["metrics"]["persistent_opening_confirmed_tick"], 16)
        self.assertEqual(analysis["metrics"]["first_rupture_tick"], 2)
        self.assertEqual(analysis["metrics"]["first_outside_chamber_steam_tick"], 17)
        self.assertEqual(analysis["metrics"]["first_post_opening_relief_tick"], 18)
        self.assertEqual(analysis["lifecycle"]["post_opening_end_tick"], 196)
        self.assertEqual(samples[-2]["sim_tick"], 196)
        self.assertEqual(samples[1]["relief_seam_open_cells"], 72)
        self.assertEqual(samples[1]["top_relief_seam_open_cells"], 48)
        self.assertEqual(samples[1]["bottom_relief_seam_open_cells"], 24)
        self.assertEqual(samples[1]["relief_seam_through_open_lanes"], 0)
        self.assertEqual(samples[2]["relief_seam_through_open_lanes"], 2)
        self.assertEqual(samples[2]["relief_seam_combusting_cells"], 0)
        self.assertEqual(samples[2]["relief_seam_fuel_progress_sum"], 0)
        self.assertEqual(
            analysis["metrics"][
                "through_opening_confirmation_relief_seam_combusting_cells_peak"
            ],
            0,
        )
        self.assertEqual(analysis["terminal_window"]["sample_count"], 64)
        self.assertFalse(analysis["terminal_window"]["unbounded_growth"])
        self.assertEqual(len(frames_doc["frames"]), 8)
        self.assertEqual(
            [badge["kind"] for badge in frames_doc["frames"][1]["badges"]],
            [
                "tick1",
                "first-pressure-activity",
                "first-wood-damage",
            ],
        )
        self.assertEqual(frames_doc["frames"][-1]["badges"][0]["kind"], "reset")
        self.assertEqual(events[-1]["event"], "worker_completed")

        frame = frames_doc["frames"][2]
        sample = samples[frame["sample_sequence"]]
        item = {
            "ordinal": frame["ordinal"],
            "reason": "+".join(badge["kind"] for badge in frame["badges"]),
            "sim_tick": frame["sim_tick"],
            "sample_sequence": frame["sample_sequence"],
            "state_hash": frame["state_hash"],
        }
        self.assertEqual(
            experiment.contact_sheet_caption_lines(item, sample),
            (
                "#2 first-rupture+peak-pressure+pea... | sim 2 | sample 2",
                "Pressure active 1000",
                "Chamber mean/max 130.000/220.000",
                "Seam Wood/open/through 488/88/2 | Outside Steam 0",
                "State fnv1a64:0000000000003002",
            ),
        )

        publication_log: list[str] = []
        screenshots = experiment.create_screenshots(
            run_dir, frames_doc["frames"], publication_log
        )
        contact_sheet = experiment.create_contact_sheet_bytes(
            run_dir, screenshots, samples
        )
        Image, ImageDraw, _ = experiment.pillow_modules()
        with Image.open(io.BytesIO(contact_sheet)) as sheet:
            rows = (len(screenshots) + 2) // 3
            panel_height = sheet.height // rows
            state_bbox = ImageDraw.Draw(sheet).textbbox(
                (12, 374 + 4 * 18), "State fnv1a64:0000000000003002"
            )
            self.assertLess(state_bbox[3] + 10, panel_height + 1)
            self.assertGreater(panel_height, 450)

        expected_once = experiment.pressure_expected_frame_badges(
            samples[0],
            samples[1],
            samples[1],
            samples[1],
            samples[2],
            [samples[2], samples[3], samples[4]],
            None,
            samples[5],
            samples[2],
            samples[2],
            samples[6],
            samples[-2],
            samples[-1],
            [],
            "post-opening-observation-complete",
        )
        expected_twice = copy.deepcopy(expected_once)
        self.assertEqual(expected_once, expected_twice)

        frames_path = run_dir / "work" / "frames.json"
        mutated = copy.deepcopy(frames_doc)
        mutated["frames"][-2], mutated["frames"][-1] = (
            mutated["frames"][-1],
            mutated["frames"][-2],
        )
        for ordinal, record in enumerate(mutated["frames"]):
            record["ordinal"] = ordinal
        frames_path.write_text(json.dumps(mutated), encoding="utf-8")
        with self.assertRaisesRegex(experiment.ExperimentError, "reset frame must be last|deterministic"):
            experiment.validate_telemetry(run_dir, manifest)

    def test_pressure_opening_detector_and_review_verdict_are_tri_state(self) -> None:
        diagnostics = [
            {
                "relief_seam_open_cells": raw_open,
                "relief_seam_through_open_lanes": through_lanes,
                "sim_tick": tick,
            }
            for tick, raw_open, through_lanes in (
                (2, 48, 1),
                (8, 64, 0),
                (16, 72, 1),
                (24, 80, 1),
                (32, 88, 1),
            )
        ]
        streak, starts, breaks = experiment.pressure_opening_streak(diagnostics)
        self.assertEqual([item["sim_tick"] for item in streak or []], [16, 24, 32])
        self.assertEqual([item["sim_tick"] for item in starts], [2, 16])
        self.assertEqual([item["sim_tick"] for item in breaks], [8])
        tick1_streak, tick1_starts, _ = experiment.pressure_opening_streak(
            [
                {
                    "relief_seam_open_cells": 72,
                    "relief_seam_through_open_lanes": 0,
                    "sim_tick": 1,
                },
                {
                    "relief_seam_open_cells": 80,
                    "relief_seam_through_open_lanes": 1,
                    "sim_tick": 2,
                },
                {
                    "relief_seam_open_cells": 88,
                    "relief_seam_through_open_lanes": 1,
                    "sim_tick": 8,
                },
                {
                    "relief_seam_open_cells": 96,
                    "relief_seam_through_open_lanes": 1,
                    "sim_tick": 16,
                },
            ]
        )
        self.assertEqual(
            [item["sim_tick"] for item in tick1_streak or []], [2, 8, 16]
        )
        self.assertEqual([item["sim_tick"] for item in tick1_starts], [2])
        pass_statuses = {name: "pass" for name in experiment.PRESSURE_PREDICATE_NAMES}
        flags = {
            "only_one_relief_seam_ruptured": True,
            "high_terminal_pressure_activity": False,
            "long_pressure_tail": False,
            "persistent_vent_plume": False,
            "terminal_activity_remains": False,
            "reasons": ["only_one_relief_seam_ruptured"],
        }
        self.assertEqual(
            experiment.pressure_expected_verdict(
                pass_statuses, flags, "pressure_opening_precedes_combustion"
            ),
            "NEEDS_HUMAN_REVIEW",
        )
        fail_statuses = {**pass_statuses, "exact_reset": "fail"}
        self.assertEqual(
            experiment.pressure_expected_verdict(
                fail_statuses, flags, "pressure_opening_precedes_combustion"
            ),
            "FAIL",
        )
        terminal_activity_flags = {
            **flags,
            "only_one_relief_seam_ruptured": False,
            "terminal_activity_remains": True,
            "reasons": ["terminal_activity_remains"],
        }
        self.assertEqual(
            experiment.pressure_expected_verdict(
                pass_statuses,
                terminal_activity_flags,
                "pressure_opening_precedes_combustion",
            ),
            "NEEDS_HUMAN_REVIEW",
        )
        self.assertEqual(
            experiment.pressure_expected_verdict(
                fail_statuses, flags, "fixture_causality_confounded"
            ),
            experiment.PRESSURE_FIXTURE_CAUSALITY_CONFOUNDED_VERDICT,
        )

    def test_pressure_combustion_confound_is_rejected_and_pressure_first_is_accepted(
        self,
    ) -> None:
        opening_start = {"sample_sequence": 3}
        confirmed = {"sample_sequence": 5}
        self.assertEqual(
            experiment.pressure_causal_classification(
                opening_start=opening_start,
                confirmed=confirmed,
                first_combustion=None,
                first_fuel_progress=None,
                combusting_peak=0,
                flame_event_peak=0,
                fuel_progress_sum_peak=0,
                fuel_progress_max=0,
            ),
            "pressure_opening_precedes_combustion",
        )
        self.assertEqual(
            experiment.pressure_causal_classification(
                opening_start=opening_start,
                confirmed=confirmed,
                first_combustion={"sample_sequence": 3},
                first_fuel_progress={"sample_sequence": 3},
                combusting_peak=1,
                flame_event_peak=1,
                fuel_progress_sum_peak=900,
                fuel_progress_max=9,
            ),
            "fixture_causality_confounded",
        )
        self.assertEqual(
            experiment.pressure_causal_classification(
                opening_start=None,
                confirmed=None,
                first_combustion={"sample_sequence": 2},
                first_fuel_progress={"sample_sequence": 2},
                combusting_peak=1,
                flame_event_peak=1,
                fuel_progress_sum_peak=1,
                fuel_progress_max=1,
            ),
            "insufficient_causal_evidence",
        )

    def test_pressure_confounded_worker_fixture_is_explicitly_classified(self) -> None:
        run_dir = self.create_valid_pressure_worker_fixture(
            "g8b-pressure-burst-v0-scratch-combustion-confound", mode="scratch"
        )
        self.make_pressure_fixture_combustion_confounded(run_dir)
        manifest = experiment.read_and_validate_manifest(
            run_dir / "EXPERIMENT_MANIFEST.toml"
        )
        validated, _, _, _ = experiment.validate_telemetry(run_dir, manifest)
        self.assertEqual(
            validated["verdict"],
            experiment.PRESSURE_FIXTURE_CAUSALITY_CONFOUNDED_VERDICT,
        )
        self.assertEqual(
            validated["causal_classification"], "fixture_causality_confounded"
        )
        self.assertEqual(
            validated["predicates"]["pressure_opening_precedes_combustion"][
                "status"
            ],
            "fail",
        )
        receipt_path = experiment.postprocess_run(run_dir)
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        self.assertEqual(receipt["run_mode"], "scratch")
        self.assertEqual(
            receipt["automatic_verdict"],
            experiment.PRESSURE_FIXTURE_CAUSALITY_CONFOUNDED_VERDICT,
        )
        self.assertTrue(receipt["pressure_burst"]["candidate_blocker"])
        self.assertEqual(
            receipt["pressure_burst"]["candidate_blocker_classification"],
            "fixture_causality_confounded",
        )
        self.assertTrue(receipt["pressure_burst"]["candidate_blocker_details"])

    def test_pressure_first_with_failed_hard_predicates_is_candidate_blocked(self) -> None:
        predicates = {
            name: {"status": "pass", "detail": f"{name} passed"}
            for name in experiment.PRESSURE_PREDICATE_NAMES
        }
        for name in ("exterior_vent_observed", "no_invalid_materials", "exact_reset"):
            predicates[name] = {"status": "fail", "detail": f"{name} failed"}
        blocker = experiment.pressure_candidate_blocker(
            {
                "causal_classification": "pressure_opening_precedes_combustion",
                "predicates": predicates,
            }
        )
        self.assertTrue(blocker["candidate_blocker"])
        self.assertEqual(
            blocker["candidate_blocker_classification"], "hard_predicate_failure"
        )
        self.assertEqual(
            blocker["failed_hard_predicates"],
            ["exact_reset", "exterior_vent_observed", "no_invalid_materials"],
        )
        self.assertEqual(
            {detail["predicate"] for detail in blocker["candidate_blocker_details"]},
            {"exact_reset", "exterior_vent_observed", "no_invalid_materials"},
        )

    def test_pressure_candidate_blocker_leaves_no_receipt_report_or_bundle(self) -> None:
        run_dir = self.create_valid_pressure_worker_fixture(
            "g8b-pressure-burst-v0-candidate-blocker"
        )
        self.make_pressure_fixture_combustion_confounded(run_dir)
        with self.assertRaisesRegex(
            experiment.ExperimentError,
            "Pressure candidate publication blocked.*fixture_causality_confounded",
        ):
            experiment.postprocess_run(run_dir)
        self.assertFalse((run_dir / "EXPERIMENT_RECEIPT.json").exists())
        self.assertFalse((run_dir / "report").exists())
        self.assertFalse(
            (run_dir.parent / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SUFFIX}").exists()
        )
        self.assertFalse(
            (
                run_dir.parent
                / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SHA256_SUFFIX}"
            ).exists()
        )

    def test_pressure_causal_vent_reseal_and_max_runaway_are_raw_recomputed(
        self,
    ) -> None:
        self.assertTrue(experiment.pressure_float_equal(1.0, 1.0 + 4.0e-10))
        self.assertFalse(experiment.pressure_float_equal(1.0, 1.0 + 1.0e-9))

        causal_run = self.create_valid_pressure_worker_fixture(
            "g8b-pressure-burst-v0-causal-test"
        )
        manifest = experiment.read_and_validate_manifest(
            causal_run / "EXPERIMENT_MANIFEST.toml"
        )
        samples_path = causal_run / "telemetry" / "samples.jsonl"
        samples = experiment.read_jsonl(samples_path, "Pressure test samples")
        samples[2]["outside_chamber_steam_cells"] = 2
        samples_path.write_text(
            "".join(json.dumps(item, separators=(",", ":")) + "\n" for item in samples),
            encoding="utf-8",
        )
        analysis, _, _, _ = experiment.validate_telemetry(causal_run, manifest)
        self.assertEqual(analysis["metrics"]["first_outside_chamber_steam_tick"], 17)
        analysis_path = causal_run / "work" / "analysis.json"
        hidden_reset_failure = json.loads(analysis_path.read_text(encoding="utf-8"))
        hidden_reset_failure["metrics"]["reset_exact_equivalence"] = False
        hidden_reset_failure["predicates"]["exact_reset"]["status"] = "fail"
        hidden_reset_failure["verdict"] = "FAIL"
        analysis_path.write_text(json.dumps(hidden_reset_failure), encoding="utf-8")
        accepted_failure, _, _, _ = experiment.validate_telemetry(
            causal_run, manifest
        )
        self.assertFalse(
            accepted_failure["metrics"]["reset_exact_equivalence"]
        )

        reseal_run = self.create_valid_pressure_worker_fixture(
            "g8b-pressure-burst-v0-reseal-test"
        )
        reseal_manifest = experiment.read_and_validate_manifest(
            reseal_run / "EXPERIMENT_MANIFEST.toml"
        )
        reseal_path = reseal_run / "telemetry" / "samples.jsonl"
        reseal_samples = experiment.read_jsonl(reseal_path, "Pressure reseal samples")
        reseal = reseal_samples[6]
        reseal.update(
            {
                "relief_seam_through_open_lanes": 0,
                "top_relief_seam_through_open_lanes": 0,
                "bottom_relief_seam_through_open_lanes": 0,
            }
        )
        reseal_path.write_text(
            "".join(
                json.dumps(item, separators=(",", ":")) + "\n"
                for item in reseal_samples
            ),
            encoding="utf-8",
        )
        with self.assertRaisesRegex(
            experiment.ExperimentError, "first_post_confirmation_reseal"
        ):
            experiment.validate_telemetry(reseal_run, reseal_manifest)

        trend = experiment.pressure_terminal_trend(
            [
                {
                    "sim_tick": tick,
                    "chamber_mean_pressure": 10.0,
                    "chamber_max_pressure": float(10 + tick),
                }
                for tick in range(experiment.TERMINAL_WINDOW_SAMPLES)
            ]
        )
        self.assertFalse(trend["mean_unbounded_growth"])
        self.assertTrue(trend["max_unbounded_growth"])
        self.assertTrue(trend["unbounded_growth"])
        self.assertEqual(trend["positive_max_step_count"], 63)

        reseal_frames = experiment.pressure_expected_frame_badges(
            reseal_samples[0],
            reseal_samples[1],
            reseal_samples[1],
            reseal_samples[1],
            reseal_samples[2],
            [reseal_samples[2], reseal_samples[3], reseal_samples[4]],
            reseal_samples[6],
            reseal_samples[5],
            reseal_samples[2],
            reseal_samples[2],
            reseal_samples[6],
            reseal_samples[-2],
            reseal_samples[-1],
            [],
            "post-opening-observation-complete",
        )
        badge_kinds = {
            badge["kind"]
            for frame in reseal_frames
            for badge in frame["badges"]
        }
        self.assertIn("opening-reseal", badge_kinds)
        self.assertNotIn("post-opening", badge_kinds)

    def test_pressure_audit_bundle_vnext_is_exact_hashed_and_create_new(self) -> None:
        run_dir = self.create_pressure_sealed_delivery_fixture()
        report = json.loads(
            (run_dir / "report" / "REPORT.json").read_text(encoding="utf-8")
        )
        receipt = json.loads(
            (run_dir / "EXPERIMENT_RECEIPT.json").read_text(encoding="utf-8")
        )
        prompt = (run_dir / "report" / "CHATGPT_REVIEW_PROMPT.md").read_text(
            encoding="utf-8"
        )
        self.assertEqual(
            report["schema_version"], "powdergame-pressure-burst-report-v1"
        )
        self.assertEqual(
            receipt["schema_version"], "powdergame-pressure-burst-receipt-v1"
        )
        self.assertEqual(
            report["pressure_burst"]["causal_classification"],
            "pressure_opening_precedes_combustion",
        )
        self.assertFalse(report["pressure_burst"]["candidate_blocker"])
        self.assertIsNone(
            report["pressure_burst"]["candidate_blocker_classification"]
        )
        self.assertEqual(report["pressure_burst"]["candidate_blocker_details"], [])
        self.assertEqual(report["pressure_burst"]["failed_hard_predicates"], [])
        self.assertFalse(report["pressure_burst"]["scratch_candidate_blocker"])
        self.assertIsNone(
            report["pressure_burst"]["scratch_blocker_classification"]
        )
        self.assertEqual(report["pressure_burst"], receipt["pressure_burst"])
        self.assertTrue(
            report["review_guidance"][
                "fixture_causality_confounded_is_candidate_blocker"
            ]
        )
        self.assertIn("zero relief-seam combustion/flame/fuel progress", prompt)
        self.assertIn("pressure_opening_precedes_combustion", prompt)
        receipt_sha = experiment.sha256_file(run_dir / "EXPERIMENT_RECEIPT.json")
        bundle, sidecar = experiment.create_pressure_audit_bundle_vnext(
            run_dir, self.source, receipt_sha
        )
        self.assertTrue(bundle.is_file())
        self.assertEqual(
            sidecar.read_text(encoding="utf-8"),
            f"{experiment.sha256_file(bundle)}  {bundle.name}\n",
        )
        with zipfile.ZipFile(bundle) as archive:
            names = set(archive.namelist())
            audit_manifest = json.loads(
                archive.read("AUDIT_BUNDLE_MANIFEST.json").decode("utf-8")
            )
            audit_hashes = archive.read("AUDIT_BUNDLE_HASHES.sha256").decode("utf-8")
            source_input_zip = archive.read("SOURCE_INPUT_BYTES.zip")
            packet_bytes = archive.read("REVIEW_PACKET.zip")
            self.assertIn("GIT_SOURCE_ARCHIVE.zip", names)
            self.assertEqual(
                names,
                {entry["bundle_path"] for entry in audit_manifest["direct_members"]},
            )
            hash_entries = {}
            for line in audit_hashes.splitlines():
                digest, name = line.split("  ", 1)
                hash_entries[name] = digest
            self.assertEqual(set(hash_entries), names - {"AUDIT_BUNDLE_HASHES.sha256"})
            for name, digest in hash_entries.items():
                self.assertEqual(digest, hashlib.sha256(archive.read(name)).hexdigest())
        self.assertEqual(
            audit_manifest["verification_scopes"]["HASHES.sha256"],
            "run-directory files before the final receipt, excluding only HASHES and receipt",
        )
        self.assertEqual(
            audit_manifest["verification_scopes"]["AUDIT_BUNDLE_HASHES.sha256"],
            "every other direct Audit Bundle member, excluding only this bundle-local hash inventory itself",
        )
        self.assertEqual(
            audit_manifest["nested_review_packet_inventory"],
            experiment.zip_bytes_inventory(packet_bytes, "REVIEW_PACKET.zip"),
        )
        mappings = {
            (entry["original"], entry["bundle_path"])
            for entry in audit_manifest["original_to_bundle_mapping"]
        }
        for entry in audit_manifest["nested_review_packet_inventory"]:
            self.assertIn(
                (
                    entry["path"],
                    f"REVIEW_PACKET.zip!{entry['path']}",
                ),
                mappings,
            )
        self.assertIn(
            ("EXPERIMENT_MANIFEST.toml", "EXPERIMENT_MANIFEST.toml"), mappings
        )
        self.assertIn(
            (
                "EXPERIMENT_MANIFEST.toml",
                "REVIEW_PACKET.zip!EXPERIMENT_MANIFEST.toml",
            ),
            mappings,
        )
        self.assertEqual(
            [entry["path"] for entry in audit_manifest["omitted_work"]],
            ["work/analysis.json", "work/frames.json", "work/frames/**"],
        )
        source_manifest = json.loads(
            (run_dir / experiment.SOURCE_INPUT_MANIFEST_NAME).read_text(encoding="utf-8")
        )
        with zipfile.ZipFile(io.BytesIO(source_input_zip)) as source_archive:
            for entry in source_manifest["files"]:
                self.assertEqual(
                    source_archive.read(f"repository/{entry['path']}"),
                    (self.source / entry["path"]).read_bytes(),
                )
            for entry in source_manifest["external_files"]:
                self.assertEqual(
                    source_archive.read(
                        f"external/{entry['label']}/{Path(entry['path']).name}"
                    ),
                    Path(entry["path"]).read_bytes(),
                )
        with self.assertRaisesRegex(experiment.ExperimentError, "overwrite"):
            experiment.create_pressure_audit_bundle_vnext(
                run_dir, self.source, receipt_sha
            )

    def test_pressure_audit_bundle_rejects_tamper_and_git_archive_failure(self) -> None:
        run_dir = self.create_pressure_sealed_delivery_fixture(
            "g8b-pressure-burst-v0-tamper"
        )
        receipt_sha = experiment.sha256_file(run_dir / "EXPERIMENT_RECEIPT.json")
        packet = run_dir / "report" / "REVIEW_PACKET.zip"
        packet.write_bytes(packet.read_bytes() + b"tamper")
        with self.assertRaises(experiment.ExperimentError):
            experiment.create_pressure_audit_bundle_vnext(
                run_dir, self.source, receipt_sha
            )
        bundle = run_dir.parent / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SUFFIX}"
        sidecar = run_dir.parent / (
            f"{run_dir.name}{experiment.AUDIT_BUNDLE_SHA256_SUFFIX}"
        )
        self.assertFalse(bundle.exists())
        self.assertFalse(sidecar.exists())

        # Restore the exact packet bytes, then prove a Git archive failure is fatal.
        packet.write_bytes(packet.read_bytes()[:-6])
        with mock.patch.object(
            experiment,
            "git_archive_zip_bytes",
            side_effect=experiment.ExperimentError("fixture git archive failure"),
        ):
            with self.assertRaisesRegex(experiment.ExperimentError, "git archive failure"):
                experiment.create_pressure_audit_bundle_vnext(
                    run_dir, self.source, receipt_sha
                )
        self.assertFalse(bundle.exists())
        self.assertFalse(sidecar.exists())

    def test_pressure_scratch_run_has_no_audit_bundle_vnext(self) -> None:
        run_dir = self.create_pressure_sealed_delivery_fixture(
            "g8b-pressure-burst-v0-scratch-bundle-test", mode="scratch"
        )
        receipt_sha = experiment.sha256_file(run_dir / "EXPERIMENT_RECEIPT.json")
        with self.assertRaisesRegex(experiment.ExperimentError, "candidate-only"):
            experiment.create_pressure_audit_bundle_vnext(
                run_dir, self.source, receipt_sha
            )
        self.assertFalse(
            (run_dir.parent / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SUFFIX}").exists()
        )
        self.assertFalse(
            (
                run_dir.parent
                / f"{run_dir.name}{experiment.AUDIT_BUNDLE_SHA256_SUFFIX}"
            ).exists()
        )

    def test_invalid_worker_output_leaves_no_receipt(self) -> None:
        run_dir = self.create_valid_worker_fixture()
        analysis_path = run_dir / "work" / "analysis.json"
        analysis = json.loads(analysis_path.read_text(encoding="utf-8"))
        analysis["raw_frame_count"] = 5
        analysis_path.write_text(json.dumps(analysis), encoding="utf-8")
        with self.assertRaises(experiment.ExperimentError):
            experiment.postprocess_run(run_dir)
        self.assertFalse((run_dir / "EXPERIMENT_RECEIPT.json").exists())

    def test_sand_v0_deprecated_diagnostic_tick_alias_is_sample_sequence(self) -> None:
        run_dir = self.create_valid_worker_fixture(
            "g8b-sand-fall-v0-deprecated-alias-test"
        )
        manifest = experiment.read_and_validate_manifest(
            run_dir / "EXPERIMENT_MANIFEST.toml"
        )
        analysis_path = run_dir / "work" / "analysis.json"
        analysis = json.loads(analysis_path.read_text(encoding="utf-8"))
        self.assertEqual(
            analysis["lifecycle"]["first_all_sleep_diagnostic_sample_tick"],
            analysis["lifecycle"]["first_all_sleep_sample_sequence"],
        )
        self.assertNotEqual(
            analysis["lifecycle"]["first_all_sleep_diagnostic_sample_tick"],
            analysis["lifecycle"]["first_all_sleep_sim_tick"],
        )
        analysis["lifecycle"]["first_all_sleep_diagnostic_sample_tick"] += 1
        analysis_path.write_text(json.dumps(analysis), encoding="utf-8")
        with self.assertRaisesRegex(experiment.ExperimentError, "deprecated"):
            experiment.validate_telemetry(run_dir, manifest)

    def test_heavy_manifest_raw_recomputation_and_folded_frames_are_exact(self) -> None:
        run_dir = self.create_valid_heavy_worker_fixture()
        manifest = experiment.read_and_validate_manifest(
            run_dir / "EXPERIMENT_MANIFEST.toml"
        )
        self.assertEqual(manifest["schema_version"], experiment.HEAVY_MANIFEST_SCHEMA)
        self.assertEqual(
            manifest["experiment"],
            {
                "max_ticks": 20_000,
                "diagnostic_interval_ticks": 8,
                "terminal_window_samples": 64,
                "meaningful_overlap_samples": 3,
            },
        )
        self.assertEqual(
            manifest["commands"]["worker"][-4:],
            ["--max-ticks", "20000", "--diagnostic-interval", "8"],
        )
        for legacy_only in (
            "--consecutive-all-sleep",
            "--post-sleep-ticks",
            "--consecutive-reaction-zero",
            "--post-reaction-ticks",
            "--consecutive-persistent-opening",
            "--post-opening-ticks",
        ):
            self.assertNotIn(legacy_only, manifest["commands"]["worker"])

        analysis, frames_doc, samples, events = experiment.validate_telemetry(
            run_dir, manifest
        )
        self.assertEqual(analysis["verdict"], "PASS")
        self.assertEqual(analysis["metrics"]["first_combustion_work_tick"], 1)
        self.assertEqual(analysis["metrics"]["first_smoke_generation_tick"], 1)
        self.assertEqual(analysis["metrics"]["first_pressure_activity_tick"], 1)
        self.assertEqual(analysis["metrics"]["first_rupture_tick"], 1)
        self.assertEqual(analysis["metrics"]["first_three_subsystems_tick"], 1)
        self.assertEqual(analysis["metrics"]["longest_three_plus_window_samples"], 3)
        self.assertEqual(analysis["metrics"]["zero_activity_before_overlap_samples"], 0)
        self.assertEqual(analysis["metrics"]["wake_anomaly_occurrences"], 0)
        self.assertFalse(analysis["terminal_trend"]["unbounded_growth"])
        self.assertEqual(len(samples), 2_504)
        self.assertEqual(len(frames_doc["frames"]), 12)
        self.assertEqual(frames_doc["frames"][-1]["badges"][0]["kind"], "reset")
        self.assertEqual(events[7]["event"], "first_combustion_work_observed")
        self.assertEqual(events[8]["event"], "first_smoke_generation_observed")
        tick1_badges = [badge["kind"] for badge in frames_doc["frames"][1]["badges"]]
        self.assertIn("first-rupture", tick1_badges)
        self.assertIn("peak-concurrency", tick1_badges)

    def test_heavy_recomputation_rejects_inventory_dynamic_runaway_and_reset_mutations(
        self,
    ) -> None:
        inventory_dir = self.create_valid_heavy_worker_fixture(
            "g8b-heavy-mixed-v0-inventory-mutation"
        )
        inventory_manifest = experiment.read_and_validate_manifest(
            inventory_dir / "EXPERIMENT_MANIFEST.toml"
        )
        samples_path = inventory_dir / "telemetry" / "samples.jsonl"
        samples = experiment.read_jsonl(samples_path, "Heavy inventory mutation")
        samples[1]["unexplained_material_delta_cells"] = 1
        samples[1]["explained_material_delta_cells"] = 1
        samples[1]["inventory_accounted"] = False
        samples_path.write_text(
            "".join(json.dumps(sample) + "\n" for sample in samples), encoding="utf-8"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "sequential accounting"):
            experiment.validate_telemetry(inventory_dir, inventory_manifest)

        dynamic_dir = self.create_valid_heavy_worker_fixture(
            "g8b-heavy-mixed-v0-dynamic-mutation"
        )
        dynamic_manifest = experiment.read_and_validate_manifest(
            dynamic_dir / "EXPERIMENT_MANIFEST.toml"
        )
        samples_path = dynamic_dir / "telemetry" / "samples.jsonl"
        samples = experiment.read_jsonl(samples_path, "Heavy dynamic mutation")
        samples[1]["oil_fuel_progress_sum"] = 0
        samples[1]["dynamic_combustion_work"] = False
        samples[1]["new_smoke_cells"] = 0
        samples_path.write_text(
            "".join(json.dumps(sample) + "\n" for sample in samples), encoding="utf-8"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "sequential accounting"):
            experiment.validate_telemetry(dynamic_dir, dynamic_manifest)

        runaway_dir = self.create_valid_heavy_worker_fixture(
            "g8b-heavy-mixed-v0-runaway-mutation"
        )
        runaway_manifest = experiment.read_and_validate_manifest(
            runaway_dir / "EXPERIMENT_MANIFEST.toml"
        )
        samples_path = runaway_dir / "telemetry" / "samples.jsonl"
        samples = experiment.read_jsonl(samples_path, "Heavy runaway mutation")
        for offset, sample in enumerate(samples[-65:-1]):
            sample["temperature_max"] = 100.0 + offset * 3.0
        samples_path.write_text(
            "".join(json.dumps(sample) + "\n" for sample in samples), encoding="utf-8"
        )
        runaway_analysis_path = runaway_dir / "work" / "analysis.json"
        runaway_analysis = json.loads(
            runaway_analysis_path.read_text(encoding="utf-8")
        )
        runaway_analysis["metrics"]["terminal_bounds"]["temperature_max"] = samples[-2][
            "temperature_max"
        ]
        runaway_analysis_path.write_text(json.dumps(runaway_analysis), encoding="utf-8")
        with self.assertRaisesRegex(experiment.ExperimentError, "terminal trend"):
            experiment.validate_telemetry(runaway_dir, runaway_manifest)

        reset_dir = self.create_valid_heavy_worker_fixture(
            "g8b-heavy-mixed-v0-reset-mutation"
        )
        reset_manifest = experiment.read_and_validate_manifest(
            reset_dir / "EXPERIMENT_MANIFEST.toml"
        )
        samples_path = reset_dir / "telemetry" / "samples.jsonl"
        samples = experiment.read_jsonl(samples_path, "Heavy reset mutation")
        samples[-1]["state_hash"] = "fnv1a64:ffffffffffffffff"
        samples_path.write_text(
            "".join(json.dumps(sample) + "\n" for sample in samples), encoding="utf-8"
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "exact reset claim"):
            experiment.validate_telemetry(reset_dir, reset_manifest)

    def test_heavy_candidate_guard_report_receipt_prompt_and_full_badges(self) -> None:
        run_dir = self.create_valid_heavy_worker_fixture(
            "g8b-heavy-mixed-v0-report-test"
        )
        manifest = experiment.read_and_validate_manifest(
            run_dir / "EXPERIMENT_MANIFEST.toml"
        )
        analysis, frames_doc, samples, _ = experiment.validate_telemetry(
            run_dir, manifest
        )
        frame = frames_doc["frames"][1]
        item = {
            "ordinal": frame["ordinal"],
            "reason": "+".join(badge["kind"] for badge in frame["badges"]),
            "sim_tick": frame["sim_tick"],
            "sample_sequence": frame["sample_sequence"],
            "state_hash": frame["state_hash"],
            "badges": frame["badges"],
        }
        lines = experiment.contact_sheet_caption_lines(
            item, samples[frame["sample_sequence"]]
        )
        badge_lines = [line for line in lines if line.startswith("Badges:")]
        self.assertGreater(len(badge_lines), 1)
        joined_badges = "\n".join(badge_lines)
        self.assertNotIn("...", joined_badges)
        for badge in frame["badges"]:
            self.assertIn(badge["kind"], joined_badges)

        publication_log: list[str] = []
        receipt_path = experiment.postprocess_run(run_dir, publication_log)
        self.assertEqual(publication_log[-1], "EXPERIMENT_RECEIPT.json")
        report = json.loads(
            (run_dir / "report" / "REPORT.json").read_text(encoding="utf-8")
        )
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        prompt = (run_dir / "report" / "CHATGPT_REVIEW_PROMPT.md").read_text(
            encoding="utf-8"
        )
        self.assertEqual(report["schema_version"], experiment.HEAVY_REPORT_SCHEMA)
        self.assertEqual(receipt["schema_version"], experiment.HEAVY_RECEIPT_SCHEMA)
        self.assertTrue(report["scope"]["heavy_mixed"])
        self.assertEqual(report["heavy_mixed"], receipt["heavy_mixed"])
        self.assertIn("Heavy Mixed World Experiment", prompt)
        self.assertIn("Authored tick-0 Smoke", prompt)
        self.assertIn("wrapped `Badges:`", prompt)
        self.assertIn(experiment.HEAVY_EXTERIOR_STEAM_PRESENTATION, prompt)
        self.assertIn("they are not proof that Steam crossed a complete", prompt)
        self.assertIn("not user acceptance", prompt)
        self.assertIn("G8-B/G8-C closure", prompt)
        report_markdown = (run_dir / "report" / "REPORT.md").read_text(encoding="utf-8")
        self.assertIn(experiment.HEAVY_EXTERIOR_STEAM_PRESENTATION, report_markdown)
        self.assertNotIn("causal rupture / opening / vent", report_markdown)
        self.assertFalse(
            report["review_guidance"][
                "exterior_steam_above_relief_is_opening_gated"
            ]
        )
        self.assertFalse(report["review_guidance"]["g8b_closed"])
        self.assertFalse(experiment.heavy_candidate_blocker(analysis)["candidate_blocker"])

        presentation_badges = experiment.heavy_presentation_badges(
            [
                {"kind": "first-vent", "reason": "exterior-Steam-above-relief"},
                {"kind": "terminal", "reason": "max-tick-reached"},
            ]
        )
        self.assertEqual(
            presentation_badges,
            [
                {
                    "kind": "first-exterior-steam",
                    "reason": experiment.HEAVY_EXTERIOR_STEAM_PRESENTATION,
                },
                {"kind": "terminal", "reason": "max-tick-reached"},
            ],
        )

        blocked_dir = self.create_valid_heavy_worker_fixture(
            "g8b-heavy-mixed-v0-candidate-blocked"
        )
        blocked_manifest = experiment.read_and_validate_manifest(
            blocked_dir / "EXPERIMENT_MANIFEST.toml"
        )
        sample_path = blocked_dir / "telemetry" / "samples.jsonl"
        blocked_samples = experiment.read_jsonl(sample_path, "Heavy wake blocker")
        blocked_samples[2]["wake_chunks"] = 1
        blocked_samples[2]["wake_reason_or"] = 4
        blocked_samples[2]["wake_anomaly_chunks"] = 1
        sample_path.write_text(
            "".join(json.dumps(sample) + "\n" for sample in blocked_samples),
            encoding="utf-8",
        )
        analysis_path = blocked_dir / "work" / "analysis.json"
        blocked_analysis = json.loads(analysis_path.read_text(encoding="utf-8"))
        blocked_analysis["metrics"]["wake_anomaly_occurrences"] = 1
        blocked_analysis["predicates"]["no_wake_anomalies"]["status"] = "fail"
        blocked_analysis["verdict"] = "FAIL"
        analysis_path.write_text(json.dumps(blocked_analysis), encoding="utf-8")
        validated, _, _, _ = experiment.validate_telemetry(
            blocked_dir, blocked_manifest
        )
        self.assertEqual(
            experiment.heavy_candidate_blocker(validated)["failed_hard_predicates"],
            ["no_wake_anomalies"],
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "candidate publication blocked"):
            experiment.postprocess_run(blocked_dir)
        self.assertFalse((blocked_dir / "report").exists())
        self.assertFalse((blocked_dir / "EXPERIMENT_RECEIPT.json").exists())

    def test_heavy_generic_duplicate_frames_prune_only_above_evidence_floor(self) -> None:
        def sample(sequence: int, tick: int, state_hash: str) -> dict:
            return {
                "sample_sequence": sequence,
                "sim_tick": tick,
                "state_hash": state_hash,
                "census": {
                    "any_active_cells": 1,
                    "matter_active_cells": 1,
                    "thermal_active_cells": 0,
                    "pressure_active_cells": 0,
                    "reaction_active_cells": 0,
                },
                "subsystem_active_count": 1,
                "sand_count": 1,
                "water_count": 1,
                "oil_count": 1,
                "wood_count": 1,
                "ice_count": 1,
                "steam_count": 1,
                "smoke_count": 1,
            }

        samples = [sample(0, 0, "fnv1a64:0000000000000000")]
        samples.extend(
            sample(sequence, sequence * 8, "fnv1a64:00000000000000aa")
            for sequence in range(1, 11)
        )
        samples.append(sample(11, 0, "fnv1a64:0000000000000000"))
        milestones = {
            sequence: [{"kind": "representative", "reason": "scheduled-mixed-state"}]
            for sequence in range(1, 11)
        }
        frames = experiment.heavy_expected_frames(samples, milestones, None, None)
        self.assertEqual(len(frames), 10)
        self.assertEqual(frames[0]["badges"][0]["kind"], "tick0")
        self.assertEqual(frames[-1]["badges"][0]["kind"], "reset")
        self.assertTrue(
            all(
                frame["badges"][0]["kind"] == "representative"
                for frame in frames[1:-1]
            )
        )

    def test_heavy_audit_bundle_vnext_has_local_hashes_and_path_mapping(self) -> None:
        run_dir = self.create_heavy_sealed_delivery_fixture()
        receipt_sha = experiment.sha256_file(run_dir / "EXPERIMENT_RECEIPT.json")
        bundle, sidecar = experiment.create_heavy_audit_bundle_vnext(
            run_dir, self.source, receipt_sha
        )
        self.assertEqual(
            sidecar.read_text(encoding="utf-8"),
            f"{experiment.sha256_file(bundle)}  {bundle.name}\n",
        )
        with zipfile.ZipFile(bundle) as archive:
            names = set(archive.namelist())
            bundle_manifest = json.loads(
                archive.read("AUDIT_BUNDLE_MANIFEST.json").decode("utf-8")
            )
            bundle_hashes = archive.read("AUDIT_BUNDLE_HASHES.sha256").decode(
                "utf-8"
            )
            hash_entries = {
                line.split("  ", 1)[1]: line.split("  ", 1)[0]
                for line in bundle_hashes.splitlines()
            }
            self.assertEqual(set(hash_entries), names - {"AUDIT_BUNDLE_HASHES.sha256"})
            for name, expected in hash_entries.items():
                self.assertEqual(hashlib.sha256(archive.read(name)).hexdigest(), expected)
        self.assertEqual(
            bundle_manifest["schema_version"],
            experiment.HEAVY_AUDIT_BUNDLE_MANIFEST_SCHEMA,
        )
        self.assertEqual(bundle_manifest["scenario"], "heavy-mixed")
        self.assertIn("SOURCE_INPUT_BYTES.zip", names)
        self.assertIn("GIT_SOURCE_ARCHIVE.zip", names)
        self.assertIn(experiment.FROZEN_BINARY_RELATIVE_PATH.as_posix(), names)
        mappings = bundle_manifest["original_to_bundle_mapping"]
        self.assertTrue(
            any(
                mapping["original"] == "telemetry/samples.jsonl"
                and mapping["bundle_path"]
                == "REVIEW_PACKET.zip!telemetry/samples.jsonl"
                for mapping in mappings
            )
        )
        self.assertTrue(
            any(mapping["bundle_path"].startswith("SOURCE_INPUT_BYTES.zip!") for mapping in mappings)
        )
        with self.assertRaisesRegex(experiment.ExperimentError, "overwrite"):
            experiment.create_heavy_audit_bundle_vnext(
                run_dir, self.source, receipt_sha
            )

    def test_heavy_addition_preserves_legacy_manifest_and_worker_shapes(self) -> None:
        expected_experiments = {
            experiment.SAND_CONTRACT: {
                "max_ticks": 20_000,
                "diagnostic_interval_ticks": 8,
                "consecutive_all_sleep": 3,
                "post_sleep_ticks": 180,
            },
            experiment.WATER_CONTRACT: {
                "max_ticks": 20_000,
                "diagnostic_interval_ticks": 8,
                "consecutive_all_sleep": 3,
                "post_sleep_ticks": 180,
                "stable_plateau_consecutive_samples": 8,
            },
            experiment.FIRE_CONTRACT: {
                "max_ticks": 20_000,
                "diagnostic_interval_ticks": 8,
                "consecutive_reaction_zero": 3,
                "post_reaction_ticks": 180,
            },
            experiment.PRESSURE_CONTRACT: {
                "max_ticks": 20_000,
                "diagnostic_interval_ticks": 8,
                "consecutive_persistent_opening": 3,
                "post_opening_ticks": 180,
                "terminal_window_samples": 64,
            },
        }
        for contract, expected in expected_experiments.items():
            run_id = f"{contract.experiment_id}-heavy-isolation"
            _, manifest = self.create_manifest(run_id, contract)
            self.assertEqual(manifest["experiment"], expected)
            command = manifest["commands"]["worker"]
            self.assertNotIn("heavy-mixed", command)
            self.assertNotIn("--meaningful-overlap-samples", command)


if __name__ == "__main__":
    unittest.main()
