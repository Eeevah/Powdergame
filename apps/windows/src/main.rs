//! Powdergame — Windows executable.
//!
//! winit window → wgpu/DX12 → RTX 5090 → dense GPU world → frames.
//!
//! Default (and `--smoke-frames N`): G8-B Benchmark Scenario Gallery.
//! `--runtime-baseline` explicitly selects the reference 2048×2048 world and
//! empty clear/present path (G0 technical baseline). Demo fixtures present a
//! staged world through the read-only world view:
//!   `--movement-demo` — G2 stylized forest scene (approved by the user),
//!   `--density-demo`  — G3 laboratory tanks (3 large chambers:
//!                       SAND+WATER sinking, WATER+OIL layer separation,
//!                       STEAM+SMOKE gas ordering),
//!   `--thermal-demo`  — G4 thermal lab (PHASE: sequential Ice melt by
//!                       distance from the hot source; HEAT FLOW: sealed
//!                       Water vs Oil conduction tubes; COMBUSTION: Wood
//!                       ignition front travelling along a strip),
//!   `--pressure-demo` — G5 twin-boiler user-validation scene: identical
//!                       heated Water chambers, weak Wood relief plug on the
//!                       left and unbreakable Stone control on the right.
//!   `--benchmark-gallery` — G8-B paused six-scenario inspection Gallery,
//!                           sharing deterministic staging with the headless
//!                           benchmark harness.
//! Forest scene is unused by the G3/G4/G5 demos.
//!
//! Demos start PAUSED so the untouched initial scene can be inspected:
//!   SPACE  play/pause toggle
//!   N      single simulation tick while paused
//!   R      reset the demo scene (re-staged through the validated edit hook)
//!   F      x1 / x4 / x16 sequential tick multiplier (G6/G7/G8 Gallery)
//!   1-6    select and pristine-reset a Gallery scenario
//!   ESC    exit
//! Each demo runs at its own fixed observation rate, decoupled from the
//! render rate: Movement/Density = 15 TPS (approved fixtures, unchanged),
//! Thermal = 60 TPS. Existing demo smoke runs start PLAYING so they exercise
//! ticks + presentation; the Gallery remains PAUSED by contract.
//!
//! G4-B note: Steam now condenses below 40.0, so demo Steam is staged at a
//! stable hot temperature (T = 80.0). G4-C note: Wood/Oil combustion is
//! driven by real thermal conduction from staged hot Stone reservoirs —
//! the demo never writes a Material ID mid-tick.
//!
//! The Simulation runs headless; the Renderer only reads/presents.

mod experiment;
mod gallery;
mod inspector;
mod observatory;
mod renderer;
mod text_renderer;

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{path::PathBuf, process};

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use experiment::{
    run_fire_heat_experiment, run_pressure_burst_experiment, run_sand_fall_experiment,
    run_water_flow_experiment, verify_current_executable_sha256, ExperimentWorkerConfig,
    EXPERIMENT_ID, FIRE_EXPERIMENT_ID, PRESSURE_EXPERIMENT_ID, WATER_EXPERIMENT_ID,
};
use gallery::{
    GalleryHudData, GalleryState, GalleryTransition, RuntimeProvenance, GALLERY_CONTROLS,
};
use inspector::{CellCoordinate, CellInspectorCollector, InspectorHudData, ScreenRect};
use observatory::ObservatoryCollector;
use powdergame_core::{
    WorldConfig, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_OIL, MATERIAL_SAND, MATERIAL_SMOKE,
    MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD,
};
use powdergame_gpu::{verify_target_hardware, AdapterReport, GpuError, Simulation};
use powdergame_scenarios::{reset_and_stage_scenario, ScenarioId};

use renderer::{PresentationPalette, Renderer, WorldViewSpec};

/// Demo observation rates: independent of the render FPS. Movement/Density
/// keep the approved 15 TPS fixture timing; Thermal runs at 60 TPS so the
/// heat/phase/combustion chain reads at a natural speed.
const MOVEMENT_DEMO_TPS: u32 = 15;
const DENSITY_DEMO_TPS: u32 = 15;
const THERMAL_DEMO_TPS: u32 = 60;
const PRESSURE_DEMO_TPS: u32 = 60;
const PARALLEL_INTEGRITY_DEMO_TPS: u32 = 60;
const ACTIVITY_DEMO_TPS: u32 = 60;
const GALLERY_TPS: u32 = 60;

const MOVEMENT_DEMO_TITLE: &str = "Powdergame G2 Demo | SAND | WATER | OIL | STEAM | SMOKE";
const DENSITY_DEMO_TITLE: &str =
    "Powdergame G3 Density Demo | SAND+WATER | WATER+OIL | STEAM+SMOKE";
const THERMAL_DEMO_TITLE: &str =
    "Powdergame G4 Thermal Observatory | 4 Large Panels + Live Metrics";
const PRESSURE_DEMO_TITLE: &str =
    "Powdergame G5 Pressure Multi-Boiler Lab | 2x2 Standard vs Extreme Overdrive | Heat → Steam → Confinement → Rupture → Vent";
const PARALLEL_INTEGRITY_DEMO_TITLE: &str =
    "Powdergame G6 Parallel Integrity Lab | Contention + Chunk Boundary + Ownership Stress";
const ACTIVITY_DEMO_TITLE: &str =
    "Powdergame G7 Active/Sleep Observatory | Stable Bulk vs Active Frontier";
const GALLERY_TITLE: &str = "Powdergame G8-B Benchmark Scenario Gallery";

/// Which demo fixture (if any) the app presents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoMode {
    None,
    Movement,
    Density,
    Thermal,
    Pressure,
    ParallelIntegrity,
    Activity,
    Gallery,
}

impl DemoMode {
    /// Observation rate for this mode (production `Simulation::tick`
    /// semantics and the 60 TPS target are unchanged — this only throttles
    /// the demo runtime loop).
    fn ticks_per_second(self) -> u32 {
        match self {
            DemoMode::None => 60,
            DemoMode::Movement => MOVEMENT_DEMO_TPS,
            DemoMode::Density => DENSITY_DEMO_TPS,
            DemoMode::Thermal => THERMAL_DEMO_TPS,
            DemoMode::Pressure => PRESSURE_DEMO_TPS,
            DemoMode::ParallelIntegrity => PARALLEL_INTEGRITY_DEMO_TPS,
            DemoMode::Activity => ACTIVITY_DEMO_TPS,
            DemoMode::Gallery => GALLERY_TPS,
        }
    }

    fn tick_interval(self) -> Duration {
        Duration::from_nanos(1_000_000_000 / (self.ticks_per_second() as u64))
    }
}

/// Demo runtime state (demo modes only).
struct DemoState {
    base_title: &'static str,
    tps: u32,
    playing: bool,
    ticks: u64,
    last_tick: Option<Instant>,
    step_pending: bool,
    reset_pending: bool,
    /// Observatory fast-forward multiplier (1 / 4 / 16) for the G6
    /// parallel-integrity and G7-A activity demos. N always steps exactly
    /// one tick; F cycles the play multiplier; R resets it to 1.
    fast: u32,
    /// Tick counter for the measured simulation-TPS estimate (reset when play
    /// starts or the world resets).
    rate_ticks: u64,
    rate_started: Option<Instant>,
    /// Present only for G8-B. Scenario selection and diagnostic sample state
    /// are presentation/runtime concerns; physics remains in Simulation.
    gallery: Option<GalleryState>,
}

impl DemoState {
    fn new(
        base_title: &'static str,
        tps: u32,
        start_playing: bool,
        gallery: Option<GalleryState>,
    ) -> Self {
        Self {
            base_title,
            tps,
            playing: start_playing,
            ticks: 0,
            last_tick: None,
            step_pending: false,
            reset_pending: false,
            fast: 1,
            rate_ticks: 0,
            rate_started: None,
            gallery,
        }
    }

    /// Queues a reset without changing committed tick/sample attribution.
    /// Those values are committed only after shared reset/staging succeeds.
    fn queue_pristine_reset(&mut self) {
        self.reset_pending = true;
        self.step_pending = false;
        self.playing = false;
        self.last_tick = None;
        self.fast = 1;
        self.rate_ticks = 0;
        self.rate_started = None;
    }

    fn commit_pristine_reset(&mut self) {
        self.ticks = 0;
        self.last_tick = None;
    }

    fn gallery_ready_to_advance(&self) -> bool {
        self.gallery.as_ref().is_none_or(GalleryState::is_ready)
    }

    fn queue_single_step(&mut self) -> bool {
        if self.playing || !self.gallery_ready_to_advance() {
            return false;
        }
        self.step_pending = true;
        true
    }

    fn cycle_fast(&mut self) {
        self.fast = match self.fast {
            1 => 4,
            4 => 16,
            _ => 1,
        };
        if self.playing {
            self.rate_ticks = 0;
            self.rate_started = Some(Instant::now());
        }
    }

    /// Human-readable state for the window title.
    fn title(&self) -> String {
        let mut state = if self.playing {
            let fast_suffix = if self.fast > 1 {
                format!(" | FAST x{}", self.fast)
            } else {
                String::new()
            };
            format!("[PLAY {} TPS{fast_suffix}]", self.tps)
        } else {
            let fast_suffix = if self.fast > 1 {
                format!(" | FAST x{}", self.fast)
            } else {
                String::new()
            };
            format!("[PAUSED{fast_suffix}]",)
        };
        // Measured actual simulation throughput while playing (coarse wall-
        // clock estimate, not a GPU timestamp benchmark).
        if self.playing {
            if let (Some(start), Some(elapsed)) =
                (self.rate_started, self.rate_started.map(|s| s.elapsed()))
            {
                let _ = start;
                if elapsed.as_secs_f64() >= 0.5 && self.rate_ticks > 0 {
                    let measured = self.rate_ticks as f64 / elapsed.as_secs_f64();
                    state.push_str(&format!(" | sim ~{measured:.0} TPS"));
                }
            }
        }
        let controls = if self.playing {
            "SPACE Pause".to_string()
        } else {
            "SPACE Play | N Step | R Reset".to_string()
        };
        let gallery_label = self.gallery.as_ref().map_or_else(String::new, |gallery| {
            let transition = match gallery.transition() {
                GalleryTransition::Ready => String::new(),
                GalleryTransition::Pending { requested } => {
                    format!(
                        " | RESET PENDING -> {}/6 {}",
                        requested.number(),
                        requested.name()
                    )
                }
                GalleryTransition::Failed { requested, .. } => {
                    format!(
                        " | RESET FAILED -> {}/6 {}",
                        requested.number(),
                        requested.name()
                    )
                }
            };
            format!(
                " | {}/6 {}",
                gallery.scenario_number(),
                gallery.scenario().name()
            ) + &transition
        });
        format!(
            "{}{} | {state} | {controls} | tick {}",
            self.base_title, gallery_label, self.ticks
        )
    }
}

/// App state. Simulation and Renderer are kept separate: the simulation does
/// not know about the window; the renderer only presents frames.
struct App {
    window: Option<Arc<Window>>,
    // Declared before Simulation so a pending map is cancelled/unmapped before
    // the GPU context is dropped during App teardown.
    cell_inspector: Option<CellInspectorCollector>,
    simulation: Option<Simulation>,
    renderer: Option<Renderer>,
    observatory_collector: Option<ObservatoryCollector>,
    frames_rendered: u32,
    smoke_frames: Option<u32>,
    demo_mode: DemoMode,
    demo: Option<DemoState>,
    gallery_provenance: Option<RuntimeProvenance>,
    experiment: Option<ExperimentWorkerConfig>,
    cursor_position: Option<PhysicalPosition<f64>>,
    fatal_error: Option<String>,
}

impl App {
    fn new(
        smoke_frames: Option<u32>,
        demo_mode: DemoMode,
        experiment: Option<ExperimentWorkerConfig>,
    ) -> Self {
        Self {
            window: None,
            cell_inspector: None,
            simulation: None,
            renderer: None,
            observatory_collector: None,
            frames_rendered: 0,
            smoke_frames,
            demo_mode,
            demo: None,
            gallery_provenance: None,
            experiment,
            cursor_position: None,
            fatal_error: None,
        }
    }

    fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<(), GpuError> {
        let base_title = match self.demo_mode {
            DemoMode::Movement => MOVEMENT_DEMO_TITLE,
            DemoMode::Density => DENSITY_DEMO_TITLE,
            DemoMode::Thermal => THERMAL_DEMO_TITLE,
            DemoMode::Pressure => PRESSURE_DEMO_TITLE,
            DemoMode::ParallelIntegrity => PARALLEL_INTEGRITY_DEMO_TITLE,
            DemoMode::Activity => ACTIVITY_DEMO_TITLE,
            DemoMode::Gallery => GALLERY_TITLE,
            DemoMode::None => "Powdergame — G0 Runtime",
        };
        // The thermal and pressure observatories use a larger world (320×192 / 256×256),
        // so they get a 1600×900 window; the G2/G3 fixtures keep 1280×720.
        let (window_w, window_h) = if self.demo_mode == DemoMode::Thermal
            || self.demo_mode == DemoMode::Pressure
            || self.demo_mode == DemoMode::ParallelIntegrity
            || self.demo_mode == DemoMode::Activity
            || self.demo_mode == DemoMode::Gallery
        {
            (1600.0, 900.0)
        } else {
            (1280.0, 720.0)
        };
        let window_attributes = if self.experiment.is_some() {
            winit::window::WindowAttributes::default()
                .with_title(base_title)
                .with_inner_size(winit::dpi::PhysicalSize::new(1600, 900))
                .with_visible(false)
        } else {
            winit::window::WindowAttributes::default()
                .with_title(base_title)
                .with_inner_size(winit::dpi::LogicalSize::new(window_w, window_h))
        };
        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .map_err(|e| GpuError::Other(format!("window create failed: {e}")))?,
        );

        // DX12 + high-performance adapter (G0: no fallback).
        let context = pollster::block_on(powdergame_gpu::GpuContext::new())?;

        println!("[powdergame] === GPU context ===");
        println!(
            "[powdergame] {}",
            AdapterReport::from_info(&context.adapter_info)
        );
        match verify_target_hardware(&context.adapter_info) {
            Ok(()) => println!("[powdergame] hardware check: PASS (RTX 5090 / Dx12)"),
            Err(e) => println!("[powdergame] hardware check: UNEXPECTED — {e}"),
        }

        // Headless simulation. Demo modes use a small staged world through
        // the validated edit hook (128×128 for the G2/G3 fixtures, 320×192
        // for the G4 thermal observatory, 256×256 for the G5 pressure lab); production stays
        // GPU-authoritative.
        let config = if self.demo_mode == DemoMode::None {
            WorldConfig::reference()
        } else {
            let (w, h) = match self.demo_mode {
                DemoMode::Thermal => (320, 192),
                DemoMode::Pressure => (256, 256),
                DemoMode::ParallelIntegrity => (256, 256),
                DemoMode::Activity => (256, 256),
                DemoMode::Gallery => (256, 256),
                _ => (128, 128),
            };
            WorldConfig::new(w, h, 64).expect("demo world config")
        };
        let mut simulation = Simulation::with_context(context, config)?;
        println!("[powdergame] === world allocation ===");
        println!("[powdergame] {}", simulation.world.allocation);
        println!("[powdergame] allocation: success");

        let observatory_collector = if self.demo_mode == DemoMode::Thermal
            || self.demo_mode == DemoMode::Pressure
            || self.demo_mode == DemoMode::ParallelIntegrity
            || self.demo_mode == DemoMode::Activity
        {
            Some(ObservatoryCollector::new(
                &simulation,
                self.demo_mode == DemoMode::ParallelIntegrity,
                self.demo_mode == DemoMode::Activity,
            ))
        } else {
            None
        };
        // The hidden experiment worker also routes through Gallery mode, so
        // the interactive Inspector must be gated on the worker being absent.
        let cell_inspector = if cell_inspector_is_enabled(self.demo_mode, self.experiment.is_some())
        {
            Some(CellInspectorCollector::new(&simulation))
        } else {
            None
        };

        match self.demo_mode {
            DemoMode::None => {
                // G0 evidence: headless tick before the window exists.
                simulation.tick()?;
                println!(
                    "[powdergame] tick ok (headless, no window); marker={}",
                    simulation.read_marker()?
                );
            }
            DemoMode::Movement => {
                stage_movement_demo(&simulation)?;
                println!("[powdergame] movement demo: scene staged (one-time edit hook)");
            }
            DemoMode::Density => {
                stage_density_demo(&simulation)?;
                println!("[powdergame] density demo: scene staged (one-time edit hook)");
            }
            DemoMode::Thermal => {
                stage_thermal_demo(&simulation)?;
                println!("[powdergame] thermal demo: 4-panel large observatory staged");
            }
            DemoMode::Pressure => {
                stage_pressure_demo(&simulation)?;
                println!("[powdergame] pressure demo: 2x2 multi-boiler lab staged (Standard vs Extreme Overdrive)");
            }
            DemoMode::ParallelIntegrity => {
                stage_parallel_integrity_demo(&simulation)?;
                println!("[powdergame] parallel integrity demo: 2x2 contention lab staged");
            }
            DemoMode::Activity => {
                reset_and_stage_scenario(&mut simulation, ScenarioId::ActiveSleepG7).map_err(
                    |error| {
                        GpuError::Other(format!("shared ActiveSleepG7 staging failed: {error}"))
                    },
                )?;
                println!("[powdergame] activity demo: shared ActiveSleepG7 fixture staged");
            }
            DemoMode::Gallery => {
                if let Some(experiment) = self.experiment.as_ref() {
                    println!(
                        "[powdergame][experiment] worker owns the pristine shared {} reset/stage",
                        experiment.scenario.name()
                    );
                } else {
                    let initial = GalleryState::new().scenario();
                    reset_and_stage_scenario(&mut simulation, initial).map_err(|error| {
                        GpuError::Other(format!("shared Gallery staging failed: {error}"))
                    })?;
                    println!(
                        "[powdergame] G8-B Gallery: scenario 1/6 {} staged through shared benchmark fixture",
                        initial.name()
                    );
                }
            }
        }

        let world_view = (self.demo_mode != DemoMode::None).then_some(WorldViewSpec {
            material_buffer: &simulation.world.material_current,
            temperature_buffer: Some(&simulation.world.temperature_current),
            pressure_buffer: Some(&simulation.world.pressure_current),
            flags_buffer: Some(&simulation.world.flags_current),
            width: simulation.world.config.width,
            height: simulation.world.config.height,
            palette: match self.demo_mode {
                DemoMode::Density => PresentationPalette::Lab,
                DemoMode::Thermal | DemoMode::Pressure => PresentationPalette::ThermalLab,
                DemoMode::ParallelIntegrity => PresentationPalette::Integrity,
                DemoMode::Activity => PresentationPalette::Activity,
                DemoMode::Gallery => PresentationPalette::Gallery,
                _ => PresentationPalette::Forest,
            },
            chunk_activity_buffer: (self.demo_mode == DemoMode::Activity)
                .then_some(&simulation.world.chunk_activity),
            chunk_size: simulation.world.config.chunk_size,
        });
        let mut renderer = Renderer::new(
            &simulation.context.instance,
            &simulation.context.adapter,
            &simulation.context.device,
            &simulation.context.queue,
            window.clone(),
            world_view,
        )?;
        println!("[powdergame] surface format: {:?}", renderer.format());

        if let Some(config) = self.experiment.as_ref() {
            let provenance = RuntimeProvenance::from_build();
            if config.scenario != ScenarioId::SandFall {
                println!(
                    "[powdergame][experiment] starting experiment_id={} scenario={} run_id={} source_sha={} git_state={} build_profile={}",
                    config.experiment_id,
                    config.scenario.slug(),
                    config.run_id,
                    provenance.source_sha,
                    provenance.git_state.as_str(),
                    provenance.build_profile
                );
            } else {
                // Preserve the sealed Sand Fall stdout contract exactly.
                println!(
                    "[powdergame][experiment] starting experiment_id={} run_id={} source_sha={} git_state={} build_profile={}",
                    config.experiment_id,
                    config.run_id,
                    provenance.source_sha,
                    provenance.git_state.as_str(),
                    provenance.build_profile
                );
            }
            let outcome = match config.scenario {
                ScenarioId::SandFall => {
                    run_sand_fall_experiment(&mut simulation, &mut renderer, &provenance, config)
                }
                ScenarioId::WaterFlow => {
                    run_water_flow_experiment(&mut simulation, &mut renderer, &provenance, config)
                }
                ScenarioId::FireHeat => {
                    run_fire_heat_experiment(&mut simulation, &mut renderer, &provenance, config)
                }
                ScenarioId::PressureBurst => run_pressure_burst_experiment(
                    &mut simulation,
                    &mut renderer,
                    &provenance,
                    config,
                ),
                scenario => Err(format!("no experiment worker is registered for {scenario}")),
            }
            .map_err(|error| GpuError::Other(format!("experiment worker failed: {error}")))?;
            if config.scenario == ScenarioId::FireHeat {
                println!(
                    "[powdergame][experiment] completed run_id={} scenario=fire-heat verdict={} samples={} raw_frames={} post_reaction_end_tick={}",
                    outcome.run_id,
                    outcome.verdict.as_str(),
                    outcome.sample_count,
                    outcome.raw_frame_count,
                    outcome
                        .post_sleep_end_tick
                        .map_or_else(|| "null".to_string(), |value| value.to_string())
                );
            } else if config.scenario == ScenarioId::PressureBurst {
                println!(
                    "[powdergame][experiment] completed run_id={} scenario=pressure-burst verdict={} samples={} raw_frames={} terminal_tick={}",
                    outcome.run_id,
                    outcome.verdict.as_str(),
                    outcome.sample_count,
                    outcome.raw_frame_count,
                    outcome
                        .post_sleep_end_tick
                        .map_or_else(|| "null".to_string(), |value| value.to_string())
                );
            } else {
                println!(
                    "[powdergame][experiment] completed run_id={} verdict={} samples={} raw_frames={} first_all_sleep_sim_tick={} first_all_sleep_diagnostic_sample_tick={}",
                    outcome.run_id,
                    outcome.verdict.as_str(),
                    outcome.sample_count,
                    outcome.raw_frame_count,
                    outcome
                        .first_all_sleep_sim_tick
                        .map_or_else(|| "null".to_string(), |value| value.to_string()),
                    outcome
                        .first_all_sleep_sample_sequence
                        .map_or_else(|| "null".to_string(), |value| value.to_string())
                );
            }
            self.window = Some(window);
            self.cell_inspector = cell_inspector;
            self.simulation = Some(simulation);
            self.renderer = Some(renderer);
            self.observatory_collector = observatory_collector;
            event_loop.exit();
            return Ok(());
        }

        if self.demo_mode != DemoMode::None {
            // Interactive sessions start PAUSED so the initial scene is fully
            // visible; bounded smoke runs start PLAYING to exercise ticks.
            // Gallery is an inspection surface and always starts PAUSED, even
            // when a bounded presentation run is requested.
            let start_playing = self.smoke_frames.is_some() && self.demo_mode != DemoMode::Gallery;
            let gallery = (self.demo_mode == DemoMode::Gallery).then(GalleryState::new);
            let demo = DemoState::new(
                base_title,
                self.demo_mode.ticks_per_second(),
                start_playing,
                gallery,
            );
            window.set_title(&demo.title());
            if self.demo_mode == DemoMode::Gallery {
                let provenance = RuntimeProvenance::from_build();
                let scenario = demo.gallery.as_ref().expect("Gallery state").scenario();
                print_gallery_runtime_context(&simulation, &demo, &provenance, scenario);
                self.gallery_provenance = Some(provenance);
                println!(
                    "[powdergame][inspector] compact hover ON | details OFF | {}-byte readback | max 10 Hz",
                    inspector::INSPECTOR_READBACK_BYTES
                );
            }
            self.demo = Some(demo);
            println!(
                "[powdergame] window + world view ready; demo {}",
                if start_playing {
                    "PLAYING (bounded smoke run)"
                } else {
                    "PAUSED — SPACE play | N step | R reset | ESC quit"
                }
            );
        } else {
            println!("[powdergame] window + renderer ready; presenting frames");
        }

        self.window = Some(window);
        self.cell_inspector = cell_inspector;
        self.simulation = Some(simulation);
        self.renderer = Some(renderer);
        self.observatory_collector = observatory_collector;
        Ok(())
    }

    fn toggle_play(&mut self, window: &Window) {
        if let Some(demo) = &mut self.demo {
            if !demo.gallery_ready_to_advance() {
                println!(
                    "[powdergame][gallery] SPACE ignored until the pending/failed reset is recovered with R or 1-6"
                );
                return;
            }
            demo.playing = !demo.playing;
            if demo.playing {
                demo.last_tick = None; // restart the tick clock on resume
                demo.rate_ticks = 0;
                demo.rate_started = Some(Instant::now());
                println!("[powdergame] demo: PLAY ({} TPS)", demo.tps);
            } else {
                println!("[powdergame] demo: PAUSED");
                demo.rate_started = None;
            }
            window.set_title(&demo.title());
            window.request_redraw();
        }
    }

    fn request_step(&mut self, window: &Window) {
        if let Some(demo) = &mut self.demo {
            if !demo.queue_single_step() {
                println!(
                    "[powdergame] demo: N ignored while playing or while Gallery reset is not ready"
                );
                return;
            }
            println!("[powdergame] demo: single step requested");
            window.set_title(&demo.title());
            window.request_redraw();
        }
    }

    fn request_fast_forward(&mut self, window: &Window) {
        // G6/G7/G8 observatory demos: cycles 1x -> 4x -> 16x -> 1x.
        // `Simulation::tick` semantics are unchanged — the multiplier just
        // runs more sequential ticks per update opportunity. N always steps
        // exactly one tick.
        if !matches!(
            self.demo_mode,
            DemoMode::ParallelIntegrity | DemoMode::Activity | DemoMode::Gallery
        ) {
            return;
        }
        if let Some(demo) = &mut self.demo {
            demo.cycle_fast();
            println!("[powdergame] demo: fast-forward x{}", demo.fast);
            window.set_title(&demo.title());
            window.request_redraw();
        }
    }

    fn request_reset(&mut self, window: &Window) {
        if let Some(demo) = &mut self.demo {
            if let Some(gallery) = &mut demo.gallery {
                let scenario = gallery.request_current_reset();
                println!(
                    "[powdergame][gallery] transactional reset requested for committed {}/6 {}",
                    scenario.number(),
                    scenario.name()
                );
            }
            demo.queue_pristine_reset();
            if let Some(inspector) = &mut self.cell_inspector {
                inspector.begin_world_change();
            }
            if let Some(collector) = &mut self.observatory_collector {
                collector.reset();
            }
            println!("[powdergame] demo: reset requested");
            window.set_title(&demo.title());
            window.request_redraw();
        }
    }

    fn select_gallery_scenario(&mut self, number: u8, window: &Window) {
        if self.demo_mode != DemoMode::Gallery {
            return;
        }
        let Some(demo) = &mut self.demo else {
            return;
        };
        let Some(gallery) = &mut demo.gallery else {
            return;
        };
        let Some(requested) = gallery.request_number(number) else {
            return;
        };
        let committed = gallery.scenario();
        demo.queue_pristine_reset();
        if let Some(inspector) = &mut self.cell_inspector {
            inspector.begin_world_change();
        }
        println!(
            "[powdergame][gallery] requested {}/6 {} ({}) — PAUSED; committed attribution remains {}/6 {} until shared reset succeeds",
            requested.number(),
            requested.name(),
            requested.slug(),
            committed.number(),
            committed.name()
        );
        window.set_title(&demo.title());
        window.request_redraw();
    }

    fn toggle_sleep(&mut self, window: &Window) {
        if let Some(sim) = &mut self.simulation {
            let next_state = !sim.sleep_enabled;
            sim.set_sleep_enabled(next_state);
            println!(
                "[powdergame] sleep optimization: {}",
                if next_state {
                    "ENABLED (Sparse Work)"
                } else {
                    "DISABLED (Always-Active Reference)"
                }
            );
            window.request_redraw();
        }
    }

    fn adjust_sleep_threshold(&mut self, delta: i32, window: &Window) {
        if let Some(sim) = &mut self.simulation {
            let cur = sim.sleep_threshold as i32;
            let next = (cur + delta).clamp(1, 64) as u32;
            sim.set_sleep_threshold(next);
            println!("[powdergame] sleep settling threshold: {} ticks", next);
            window.request_redraw();
        }
    }

    fn toggle_cell_inspector_details(&mut self, window: &Window) {
        if !cell_inspector_is_enabled(self.demo_mode, self.experiment.is_some()) {
            return;
        }
        if let Some(inspector) = &mut self.cell_inspector {
            let visible = inspector.toggle_details();
            println!(
                "[powdergame][inspector] details {}",
                if visible { "ON" } else { "OFF" }
            );
            window.request_redraw();
        }
    }

    fn refresh_cell_inspector(&mut self, now: Instant) {
        if !cell_inspector_is_enabled(self.demo_mode, self.experiment.is_some()) {
            return;
        }
        let hovered = self
            .cursor_position
            .and_then(|cursor| self.renderer.as_ref()?.world_cell_at(cursor))
            .map(|(x, y)| CellCoordinate { x, y });
        let Some(inspector) = &mut self.cell_inspector else {
            return;
        };
        inspector.set_hover(hovered);
        let Some(simulation) = &self.simulation else {
            inspector.mark_unavailable("Inspector unavailable: simulation missing");
            return;
        };
        if let Err(error) = inspector.update(simulation, simulation.tick_count, now) {
            eprintln!("[powdergame][inspector] readback error: {error}");
        }
    }

    fn shutdown_cell_inspector(&mut self) {
        if let Some(inspector) = &mut self.cell_inspector {
            inspector.shutdown();
        }
    }
}

/// Unified staging configuration for boiler stress experiment chambers.
pub struct BoilerStagingConfig {
    pub x0: i64,
    pub x1: i64,
    pub roof_y: i64,
    pub bottom_y: i64,
    pub floor_heater_rows: i64,
    pub floor_heater_temp: f32,
    pub upper_heater_temp: f32,
    pub water_temp: f32,
    pub roof_relief: Option<(i64, i64)>, // (plug_left, plug_right)
    pub side_seam: Option<(i64, i64)>,   // (seam_top, seam_bottom) on right wall x1
    pub chimney_rails: bool,
    pub exhaust_duct: bool,
}

fn stage_boiler(
    simulation: &Simulation,
    cfg: &BoilerStagingConfig,
    stone: u32,
) -> Result<(), GpuError> {
    let q = &simulation.context.queue;
    let set = |x: i64, y: i64, id: u32| simulation.world.write_material(q, x, y, id);
    let set_t = |x: i64, y: i64, t: f32| simulation.world.write_temperature(q, x, y, t);

    // 1. Left Wall (Stone)
    for y in cfg.roof_y..=cfg.bottom_y {
        set(cfg.x0, y, stone)?;
    }

    // 2. Right Wall (Stone or Weak Seam)
    for y in cfg.roof_y..=(cfg.bottom_y - cfg.floor_heater_rows) {
        if let Some((s_top, s_bot)) = cfg.side_seam {
            if y >= s_top && y <= s_bot {
                set(cfg.x1, y, MATERIAL_WOOD)?;
                set_t(cfg.x1, y, 20.0)?;
                continue;
            }
        }
        set(cfg.x1, y, stone)?;
    }

    // 3. Floor Heaters
    for y in (cfg.bottom_y - cfg.floor_heater_rows + 1)..=cfg.bottom_y {
        for x in cfg.x0..=cfg.x1 {
            set(x, y, stone)?;
            set_t(x, y, cfg.floor_heater_temp)?;
        }
    }

    // 4. Roof (Stone or Roof Relief Plug)
    for x in (cfg.x0 + 1)..cfg.x1 {
        let is_plug = if let Some((p_l, p_r)) = cfg.roof_relief {
            x >= p_l && x <= p_r
        } else {
            false
        };
        let mat = if is_plug { MATERIAL_WOOD } else { stone };
        set(x, cfg.roof_y, mat)?;
        set_t(x, cfg.roof_y, 20.0)?;
    }

    // 5. Interior Water Fill
    for y in (cfg.roof_y + 1)..(cfg.bottom_y - cfg.floor_heater_rows + 1) {
        for x in (cfg.x0 + 1)..cfg.x1 {
            set(x, y, MATERIAL_WATER)?;
            set_t(x, y, cfg.water_temp)?;
        }
    }

    // 6. Upper Heater Plate (centered in chamber, 6 cells below roof)
    let center_x = (cfg.x0 + cfg.x1) / 2;
    let heater_y = cfg.roof_y + 6;
    for x in (center_x - 6)..=(center_x + 6) {
        set(x, heater_y, stone)?;
        set_t(x, heater_y, cfg.upper_heater_temp)?;
    }

    // 7. Optional Top Chimney Rails
    if cfg.chimney_rails {
        let chimney_top = if cfg.roof_y < 100 { 8i64 } else { 130i64 };
        for y in chimney_top..cfg.roof_y {
            set(center_x - 6, y, stone)?;
            set(center_x + 6, y, stone)?;
        }
    }

    // 8. Optional Side Exhaust Duct
    if cfg.exhaust_duct {
        if let Some((s_top, s_bot)) = cfg.side_seam {
            for y in (s_top - 4)..=(s_bot + 4) {
                for x in (cfg.x1 + 1)..=(cfg.x1 + 10) {
                    if y == s_top - 4 || y == s_bot + 4 {
                        set(x, y, stone)?;
                    } else {
                        set(x, y, MATERIAL_EMPTY)?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Stages the G5 2x2 Multi-Boiler Pressure Lab on the 256x256 demo world.
fn stage_pressure_demo(simulation: &Simulation) -> Result<(), GpuError> {
    let q = &simulation.context.queue;
    let set = |x: i64, y: i64, id: u32| simulation.world.write_material(q, x, y, id);
    let stone = MATERIAL_STONE;

    // ─── Central Vertical Divider (x 126..129) ───
    for y in 4..=250 {
        set(126, y, stone)?;
        set(127, y, stone)?;
        set(128, y, stone)?;
        set(129, y, stone)?;
    }

    // ─── Central Horizontal Divider (y 118..124) ───
    for x in 4..=251 {
        for y in 118..=124 {
            set(x, y, stone)?;
        }
    }

    // ─── TOP-LEFT (Panel A: WOOD RELIEF CANONICAL STANDARD) ───
    stage_boiler(
        simulation,
        &BoilerStagingConfig {
            x0: 14,
            x1: 114,
            roof_y: 44,
            bottom_y: 108,
            floor_heater_rows: 1,
            floor_heater_temp: 150.0,
            upper_heater_temp: 110.0,
            water_temp: 58.0,
            roof_relief: Some((60, 68)),
            side_seam: None,
            chimney_rails: true,
            exhaust_duct: false,
        },
        stone,
    )?;

    // ─── TOP-RIGHT (Panel B: STONE SEALED STANDARD CONTROL) ───
    stage_boiler(
        simulation,
        &BoilerStagingConfig {
            x0: 142,
            x1: 242,
            roof_y: 44,
            bottom_y: 108,
            floor_heater_rows: 1,
            floor_heater_temp: 150.0,
            upper_heater_temp: 110.0,
            water_temp: 58.0,
            roof_relief: None,
            side_seam: None,
            chimney_rails: true,
            exhaust_duct: false,
        },
        stone,
    )?;

    // ─── BOTTOM-LEFT (Panel C: WOOD RELIEF EXTREME OVERDRIVE) ───
    stage_boiler(
        simulation,
        &BoilerStagingConfig {
            x0: 14,
            x1: 114,
            roof_y: 170,
            bottom_y: 236,
            floor_heater_rows: 3,
            floor_heater_temp: 220.0,
            upper_heater_temp: 130.0,
            water_temp: 58.0,
            roof_relief: Some((60, 68)),
            side_seam: None,
            chimney_rails: true,
            exhaust_duct: false,
        },
        stone,
    )?;

    // ─── BOTTOM-RIGHT (Panel D: STONE SEALED EXTREME -> DELAYED PRESSURE BREACH) ───
    stage_boiler(
        simulation,
        &BoilerStagingConfig {
            x0: 142,
            x1: 242,
            roof_y: 170,
            bottom_y: 236,
            floor_heater_rows: 3,
            floor_heater_temp: 220.0,
            upper_heater_temp: 130.0,
            water_temp: 58.0,
            roof_relief: None,
            side_seam: Some((214, 222)),
            chimney_rails: false,
            exhaust_duct: true,
        },
        stone,
    )?;

    Ok(())
}

/// Stages the G2 stylized-forest movement scene on the 128×128 demo world.
fn stage_movement_demo(simulation: &Simulation) -> Result<(), GpuError> {
    let q = &simulation.context.queue;
    let set = |x: i64, y: i64, id: u32| simulation.world.write_material(q, x, y, id);
    let stone = MATERIAL_STONE;

    for dx in [22i64, 48, 70, 94] {
        for y in 4..=126 {
            set(dx, y, stone)?;
        }
        for y in 1..=3 {
            for x in (dx - 1)..=(dx + 1) {
                set(x, y, stone)?;
            }
        }
    }

    for x in 6..=18 {
        set(x, 84, stone)?;
        set(x, 85, stone)?;
    }
    for x in 12..=14 {
        set(x, 82, stone)?;
    }
    for x in 2..=21 {
        for y in 100..=126 {
            set(x, y, stone)?;
        }
    }
    for y in 4..=36 {
        for x in 6..=16 {
            set(x, y, MATERIAL_SAND)?;
        }
    }

    for x in 24..=46 {
        for y in 90..=126 {
            set(x, y, stone)?;
        }
    }
    for x in 25..=45 {
        for y in 90..=98 {
            set(x, y, MATERIAL_EMPTY)?;
        }
    }
    for y in 4..=36 {
        for x in 28..=42 {
            set(x, y, MATERIAL_WATER)?;
        }
    }

    for x in 50..=68 {
        for y in 90..=126 {
            set(x, y, stone)?;
        }
    }
    for x in 51..=67 {
        for y in 90..=98 {
            set(x, y, MATERIAL_EMPTY)?;
        }
    }
    for y in 4..=36 {
        for x in 54..=64 {
            set(x, y, MATERIAL_OIL)?;
        }
    }

    for x in 72..=92 {
        for y in 100..=126 {
            set(x, y, stone)?;
        }
    }
    for x in 76..=88 {
        for y in 105..=115 {
            set(x, y, MATERIAL_STEAM)?;
            simulation.world.write_temperature(q, x, y, 80.0)?;
        }
    }

    for x in 96..=116 {
        for y in 100..=126 {
            set(x, y, stone)?;
        }
    }
    for x in 100..=112 {
        for y in 105..=115 {
            set(x, y, MATERIAL_SMOKE)?;
        }
    }

    for y in 110..=126 {
        for x in 120..=121 {
            set(x, y, stone)?;
        }
        for x in 126..=127 {
            set(x, y, stone)?;
        }
    }
    for x in 122..=125 {
        set(x, 127, MATERIAL_EMPTY)?;
    }
    for y in 70..=95 {
        for x in 122..=125 {
            set(x, y, MATERIAL_SAND)?;
        }
    }

    Ok(())
}

/// Stages the G3 laboratory tanks demo on the 128×128 world.
fn stage_density_demo(simulation: &Simulation) -> Result<(), GpuError> {
    let q = &simulation.context.queue;
    let set = |x: i64, y: i64, id: u32| simulation.world.write_material(q, x, y, id);
    let stone = MATERIAL_STONE;

    for dx in [42i64, 84] {
        for y in 4..=126 {
            for x in (dx - 1)..=(dx + 1) {
                set(x, y, stone)?;
            }
        }
    }

    for x in 2..=40 {
        for y in 124..=126 {
            set(x, y, stone)?;
        }
    }
    for x in 44..=82 {
        for y in 124..=126 {
            set(x, y, stone)?;
        }
    }
    for x in 86..=126 {
        for y in 124..=126 {
            set(x, y, stone)?;
        }
    }

    for y in 56..=123 {
        for x in 5..=38 {
            set(x, y, MATERIAL_WATER)?;
        }
    }
    for y in 8..=48 {
        for x in 8..=35 {
            set(x, y, MATERIAL_SAND)?;
        }
    }

    for y in 56..=123 {
        for x in 47..=80 {
            set(x, y, MATERIAL_OIL)?;
        }
    }
    for y in 8..=48 {
        for x in 50..=77 {
            set(x, y, MATERIAL_WATER)?;
        }
    }

    for y in 8..=56 {
        for x in 89..=123 {
            set(x, y, MATERIAL_SMOKE)?;
        }
    }
    for y in 68..=120 {
        for x in 92..=120 {
            set(x, y, MATERIAL_STEAM)?;
            simulation.world.write_temperature(q, x, y, 80.0)?;
        }
    }

    Ok(())
}

/// Stages the 4-panel large G4 Thermal Observatory on the 320×192 world.
///
/// Layout (2×2 large chambers):
///   Top-Left (x 1..157, y 1..93):   A. PHASE HEATING (Ice sequential melt → Water → Steam)
///   Top-Right (x 162..318, y 1..93):  B. PHASE COOLING (Steam → Water condensation → Ice freezing)
///   Bottom-Left (x 1..157, y 98..190): C. HEAT COMPARISON (Identical sealed Water vs Oil tubes)
///   Bottom-Right (x 162..318, y 98..190): D. COMBUSTION (Wood strip ignition, spread, finite burn to EMPTY)
fn stage_thermal_demo(simulation: &Simulation) -> Result<(), GpuError> {
    let q = &simulation.context.queue;
    let set = |x: i64, y: i64, id: u32| simulation.world.write_material(q, x, y, id);
    let set_t = |x: i64, y: i64, t: f32| simulation.world.write_temperature(q, x, y, t);
    let stone = MATERIAL_STONE;

    // ─── Central Vertical Divider with Void Chimney (x 158..161) ───
    for y in 1..=190 {
        set(157, y, stone)?;
        set(158, y, stone)?;
        set(159, y, MATERIAL_EMPTY)?; // chimney column → Void
        set(160, y, MATERIAL_EMPTY)?;
        set(161, y, stone)?;
        set(162, y, stone)?;
    }
    set(159, 0, MATERIAL_EMPTY)?; // top chimney opening
    set(160, 0, MATERIAL_EMPTY)?;

    // ─── Central Horizontal Divider (y 94..97) ───
    for x in 1..=318 {
        set(x, 94, stone)?;
        set(x, 95, stone)?;
        set(x, 96, stone)?;
        set(x, 97, stone)?;
    }

    // ─── A — PHASE HEATING (Top-Left: x 1..157, y 1..93) ───
    // Hot Stone source at bottom-left (T=250). Stone conductor bar along floor.
    // Three Ice masses at increasing distances. Steam rises to top vent.
    for x in 3..=155 {
        for y in 88..=93 {
            set(x, y, stone)?;
        }
    }
    for x in 6..=24 {
        for y in 76..=93 {
            set(x, y, stone)?;
            set_t(x, y, 250.0)?; // hot left span
        }
    }
    // Conductor bed
    for x in 25..=150 {
        for y in 86..=88 {
            set(x, y, stone)?;
        }
    }
    // Three Ice blocks at increasing distance from heat source:
    // Ice 1: close (melts first)
    for y in 78..=85 {
        for x in 32..=46 {
            set(x, y, MATERIAL_ICE)?;
            set_t(x, y, -30.0)?;
        }
    }
    // Ice 2: mid distance
    for y in 78..=85 {
        for x in 75..=89 {
            set(x, y, MATERIAL_ICE)?;
            set_t(x, y, -30.0)?;
        }
    }
    // Ice 3: far distance
    for y in 78..=85 {
        for x in 120..=134 {
            set(x, y, MATERIAL_ICE)?;
            set_t(x, y, -30.0)?;
        }
    }
    // Top steam vent to Void
    for x in 60..=100 {
        set(x, 0, MATERIAL_EMPTY)?;
    }

    // ─── B — PHASE COOLING (Top-Right: x 162..318, y 1..93) ───
    // Direct thermal contact: Cold ceiling (T=-40) + cold fins (T=-40) in upper Steam cavity (T=80)
    // → Steam condenses to Water → Drips through center gap → Freeze basin bottom (T=-100) → Freezes to Ice.
    // Initial Ice count = 0.
    for x in 164..=316 {
        for y in 88..=93 {
            set(x, y, stone)?;
        }
    }
    // Cold ceiling at top of panel
    for x in 164..=316 {
        for y in 1..=3 {
            set(x, y, stone)?;
            set_t(x, y, -40.0)?;
        }
    }
    // Downward cold fins extending from ceiling into steam
    let fin_ranges = [184..=186, 210..=212, 238..=240, 266..=268, 294..=296];
    for fin in &fin_ranges {
        for x in fin.clone() {
            for y in 4..=26 {
                set(x, y, stone)?;
                set_t(x, y, -40.0)?;
            }
        }
    }
    // Hot Steam mass (T=80) filling upper cavity between fins (direct 4-neighbor contact)
    for y in 4..=26 {
        for x in 166..=314 {
            let in_fin = fin_ranges.iter().any(|r| r.contains(&x));
            if !in_fin {
                set(x, y, MATERIAL_STEAM)?;
                set_t(x, y, 80.0)?;
            }
        }
    }
    // Mid condensation/guide shelves (T=-40) with center drip gap (x 201..279 open)
    for y in 48..=50 {
        for x in 164..=200 {
            set(x, y, stone)?;
            set_t(x, y, -40.0)?;
        }
        for x in 280..=316 {
            set(x, y, stone)?;
            set_t(x, y, -40.0)?;
        }
    }
    // Bottom Freeze basin (T=-100, well below -20.0 freeze threshold)
    for x in 190..=290 {
        for y in 78..=85 {
            set(x, y, stone)?;
            set_t(x, y, -100.0)?;
        }
    }
    for y in 64..=78 {
        for x in 190..=194 {
            set(x, y, stone)?;
            set_t(x, y, -100.0)?;
        }
        for x in 286..=290 {
            set(x, y, stone)?;
            set_t(x, y, -100.0)?;
        }
    }

    // ─── C — HEAT COMPARISON (Bottom-Left: x 1..157, y 98..190) ───
    // Shared bottom hot Stone reservoir (T=50) under two identical sealed tubes (Water vs Oil).
    for x in 15..=145 {
        for y in 178..=186 {
            set(x, y, stone)?;
            set_t(x, y, 50.0)?; // bottom warm reservoir
        }
    }
    // Tube 1 (WATER): Interior x 25..65, y 112..174
    for y in 110..=176 {
        for x in 23..=24 {
            set(x, y, stone)?;
        }
        for x in 66..=67 {
            set(x, y, stone)?;
        }
    }
    for y in 110..=111 {
        for x in 23..=67 {
            set(x, y, stone)?;
        } // top seal
    }
    for y in 112..=174 {
        for x in 25..=65 {
            set(x, y, MATERIAL_WATER)?;
            set_t(x, y, 0.0)?;
        }
    }

    // Tube 2 (OIL): Interior x 90..130, y 112..174
    for y in 110..=176 {
        for x in 88..=89 {
            set(x, y, stone)?;
        }
        for x in 131..=132 {
            set(x, y, stone)?;
        }
    }
    for y in 110..=111 {
        for x in 88..=132 {
            set(x, y, stone)?;
        } // top seal
    }
    for y in 112..=174 {
        for x in 90..=130 {
            set(x, y, MATERIAL_OIL)?;
            set_t(x, y, 0.0)?;
        }
    }

    // ─── D — COMBUSTION (Bottom-Right: x 162..318, y 98..190) ───
    // Hot Stone igniter on left (T=200). Large Wood strip (length 81, height 8).
    for x in 164..=316 {
        for y in 186..=190 {
            set(x, y, stone)?;
        }
    }
    // Hot Stone igniter
    for y in 144..=155 {
        for x in 192..=199 {
            set(x, y, stone)?;
            set_t(x, y, 200.0)?;
        }
    }
    // Large Wood strip
    for y in 146..=153 {
        for x in 200..=280 {
            set(x, y, MATERIAL_WOOD)?;
            set_t(x, y, 0.0)?;
        }
    }
    // Chimney vent opening to central chimney
    for y in 100..=120 {
        for x in 163..=166 {
            set(x, y, MATERIAL_EMPTY)?;
        }
    }

    Ok(())
}

fn stage_parallel_integrity_demo(simulation: &Simulation) -> Result<(), GpuError> {
    let q = &simulation.context.queue;
    let set = |x: i64, y: i64, id: u32| simulation.world.write_material(q, x, y, id);
    let set_t = |x: i64, y: i64, t: f32| simulation.world.write_temperature(q, x, y, t);
    let stone = MATERIAL_STONE;

    // Central cross dividers (same as pressure demo)
    // Vertical: x 127..128
    for y in 1..=254 {
        set(127, y, stone)?;
        set(128, y, stone)?;
    }
    // Horizontal: y 127..128
    for x in 1..=254 {
        set(x, 127, stone)?;
        set(x, 128, stone)?;
    }

    // [A] MOVEMENT CONTENTION (top-left: x 1..126, y 1..126)
    // Dense falling columns of Sand + Water + Oil competing for the same landing zones
    // Multiple columns dumping into narrow funnels to force contention
    for col in 0..6 {
        let cx = 10 + col * 20;
        // Alternating material columns
        let mat = match col % 3 {
            0 => MATERIAL_SAND,
            1 => MATERIAL_WATER,
            _ => MATERIAL_OIL,
        };
        // Tall source column (height 30)
        for y in 5..35 {
            set(cx, y, mat)?;
            set(cx + 1, y, mat)?;
        }
    }
    // Stone shelf with narrow gaps to create landing contention
    for x in 2..126 {
        set(x, 60, stone)?;
    }
    // Gaps every 10 cells
    for gap in 0..12 {
        let gx = 8 + gap * 10;
        set(gx, 60, MATERIAL_EMPTY)?;
    }
    // Second batch below shelf
    for col in 0..6 {
        let cx = 15 + col * 18;
        let mat = match col % 3 {
            0 => MATERIAL_OIL,
            1 => MATERIAL_SAND,
            _ => MATERIAL_WATER,
        };
        for y in 65..80 {
            set(cx, y, mat)?;
        }
    }

    // [B] CHUNK BOUNDARY CONTENTION (top-right: x 129..254, y 1..126)
    // Place matter exactly at chunk boundaries (x=192 is boundary between chunk 2 and 3)
    // Vertical columns straddling x=192
    for y in 5..50 {
        set(191, y, MATERIAL_SAND)?;
        set(192, y, MATERIAL_SAND)?;
        set(193, y, MATERIAL_WATER)?;
        set(194, y, MATERIAL_WATER)?;
    }
    // Horizontal layer straddling y=64 (chunk boundary)
    for x in 135..250 {
        set(x, 62, MATERIAL_WATER)?;
        set(x, 63, MATERIAL_WATER)?;
        set(x, 64, MATERIAL_WATER)?;
        set(x, 65, MATERIAL_WATER)?;
    }
    // Falling sand onto boundary region
    for x in 140..200 {
        set(x, 10, MATERIAL_SAND)?;
        set(x, 11, MATERIAL_SAND)?;
    }
    // Oil pool crossing boundary
    for x in 200..248 {
        for y in 90..110 {
            set(x, y, MATERIAL_OIL)?;
        }
    }

    // [C] EXPANSION + SMOKE OWNERSHIP (bottom-left: x 1..126, y 129..254)
    // One-tick ownership instrument (latched by `evaluate_integrity_state` on
    // the first post-tick readback). Three sub-fixtures run in the SAME tick
    // so the movement -> expansion -> smoke proposal/claim scratch reuse is
    // exercised together:
    //   LEFT   EXPANSION CONTENTION — 3 boiling Water sources with exactly one
    //          shared EMPTY destination (every other neighbor is Stone). Each
    //          source proposes the target; the claim pass picks exactly one
    //          winner (target -> Steam) and the 2 losers receive confinement
    //          Pressure. No staged Steam, no staged Pressure.
    //   CENTER MOVEMENT FIXTURE — one Sand cell above EMPTY (falls 1 cell on
    //          tick 1) so the movement pass provably ran in the same tick.
    //   RIGHT  SMOKE CONTENTION — 3 burning Wood sources with exactly one
    //          shared EMPTY Smoke target (all other neighbors Stone). One
    //          winner spawns Smoke (decay age 0); all 3 Woods are preserved.

    // --- Expansion contention (LEFT) ---
    let (x0, y0, x1, y1) = observatory::EXP_REGION;
    for y in y0..=y1 {
        for x in x0..=x1 {
            set(i64::from(x), i64::from(y), stone)?;
            set_t(i64::from(x), i64::from(y), 100.0)?;
        }
    }
    set(
        i64::from(observatory::EXP_TARGET.0),
        i64::from(observatory::EXP_TARGET.1),
        MATERIAL_EMPTY,
    )?;
    for &(sx, sy) in &observatory::EXP_SOURCES {
        set(i64::from(sx), i64::from(sy), MATERIAL_WATER)?;
        set_t(i64::from(sx), i64::from(sy), 100.0)?;
    }

    // --- Movement fixture (CENTER) ---
    let (mx0, my0, mx1, my1) = observatory::MOVE_REGION;
    for y in my0..=my1 {
        for x in mx0..=mx1 {
            set(i64::from(x), i64::from(y), stone)?;
        }
    }
    set(
        i64::from(observatory::MOVE_SRC.0),
        i64::from(observatory::MOVE_SRC.1),
        MATERIAL_SAND,
    )?;
    set(
        i64::from(observatory::MOVE_DST.0),
        i64::from(observatory::MOVE_DST.1),
        MATERIAL_EMPTY,
    )?;

    // --- Smoke contention (RIGHT) ---
    let (qx0, qy0, qx1, qy1) = observatory::SMOKE_REGION;
    for y in qy0..=qy1 {
        for x in qx0..=qx1 {
            set(i64::from(x), i64::from(y), stone)?;
            set_t(i64::from(x), i64::from(y), 110.0)?;
        }
    }
    set(
        i64::from(observatory::SMOKE_TARGET.0),
        i64::from(observatory::SMOKE_TARGET.1),
        MATERIAL_EMPTY,
    )?;
    for &(wx, wy) in &observatory::SMOKE_SOURCES {
        set(i64::from(wx), i64::from(wy), MATERIAL_WOOD)?;
        set_t(i64::from(wx), i64::from(wy), 100.0)?;
    }

    // [D] HEAVY MIXED PARALLEL STRESS (bottom-right: x 129..254, y 129..254)
    // Everything at once: sand, water, oil, steam, smoke, hot/cold, wood combustion, pressure

    // Sand heap
    for x in 135..165 {
        for y in 135..160 {
            set(x, y, MATERIAL_SAND)?;
        }
    }
    // Water body
    for x in 170..210 {
        for y in 180..220 {
            set(x, y, MATERIAL_WATER)?;
        }
    }
    // Oil body
    for x in 215..245 {
        for y in 180..220 {
            set(x, y, MATERIAL_OIL)?;
        }
    }
    // Steam pocket (hot)
    for x in 135..160 {
        for y in 165..180 {
            set(x, y, MATERIAL_STEAM)?;
            set_t(x, y, 120.0)?;
        }
    }
    // Smoke wisps
    for x in 165..185 {
        for y in 135..155 {
            set(x, y, MATERIAL_SMOKE)?;
        }
    }
    // Hot stone plate (heater for phase transitions)
    for x in 170..240 {
        set(x, 225, stone)?;
        set_t(x, 225, 180.0)?;
    }
    // Wood with igniter (combustion)
    for x in 200..240 {
        for y in 135..160 {
            set(x, y, MATERIAL_WOOD)?;
        }
    }
    set(199, 147, stone)?;
    set_t(199, 147, 250.0)?; // Igniter
                             // Cold region (ice)
    for x in 135..155 {
        for y in 230..250 {
            set(x, y, MATERIAL_ICE)?;
            set_t(x, y, -30.0)?;
        }
    }
    // Water near cold region (will freeze)
    for x in 155..175 {
        for y in 235..250 {
            set(x, y, MATERIAL_WATER)?;
            set_t(x, y, 5.0)?;
        }
    }

    Ok(())
}

/// Resets the demo world to its pristine boundary-ring state and re-stages
/// the active demo scene, using bulk initialization and validated edit hooks.
fn reset_demo_world(
    simulation: &mut Simulation,
    mode: DemoMode,
    gallery_scenario: Option<ScenarioId>,
) -> Result<(), GpuError> {
    if mode == DemoMode::Activity {
        return reset_and_stage_scenario(simulation, ScenarioId::ActiveSleepG7).map_err(|error| {
            GpuError::Other(format!(
                "shared ActiveSleepG7 reset/staging failed: {error}"
            ))
        });
    }
    if mode == DemoMode::Gallery {
        let scenario = gallery_scenario.ok_or_else(|| {
            GpuError::Other("Gallery reset requested without a selected scenario".to_string())
        })?;
        return reset_and_stage_scenario(simulation, scenario).map_err(|error| {
            GpuError::Other(format!("shared Gallery reset/staging failed: {error}"))
        });
    }

    simulation.reset()?;
    match mode {
        DemoMode::Movement => stage_movement_demo(simulation),
        DemoMode::Density => stage_density_demo(simulation),
        DemoMode::Thermal => stage_thermal_demo(simulation),
        DemoMode::Pressure => stage_pressure_demo(simulation),
        DemoMode::ParallelIntegrity => stage_parallel_integrity_demo(simulation),
        DemoMode::Activity | DemoMode::Gallery => unreachable!("handled above"),
        DemoMode::None => Ok(()),
    }
}

/// Advances the demo simulation: pending reset/step first, then the
/// fixed-rate play loop (mode TPS), decoupled from the render rate.
fn step_demo(
    simulation: &mut Simulation,
    demo: &mut DemoState,
    collector: &mut Option<ObservatoryCollector>,
    cell_inspector: &mut Option<CellInspectorCollector>,
    mode: DemoMode,
) {
    if demo.reset_pending {
        demo.reset_pending = false;
        demo.step_pending = false;
        let gallery_scenario = demo.gallery.as_ref().and_then(GalleryState::reset_target);
        match reset_demo_world(simulation, mode, gallery_scenario) {
            Ok(()) => {
                let committed_gallery = demo
                    .gallery
                    .as_mut()
                    .and_then(GalleryState::commit_reset_success);
                demo.commit_pristine_reset();
                if let Some(scenario) = committed_gallery {
                    println!(
                        "[powdergame][gallery] transactional shared reset committed: {}/6 {} | SIM TICK 0 | DIAGNOSTIC SAMPLE pending",
                        scenario.number(),
                        scenario.name()
                    );
                } else {
                    println!("[powdergame] demo: scene reset to initial state");
                }
                if let Some(col) = collector {
                    col.reset();
                }
                if let Some(inspector) = cell_inspector {
                    inspector.mark_ready();
                }
            }
            Err(error) => {
                if let Some(gallery) = &mut demo.gallery {
                    let requested = gallery.commit_reset_failure(error.to_string());
                    eprintln!(
                        "[powdergame][gallery] RESET FAILED for requested {}/6 {}: {error}; prior scenario/tick/sample attribution retained; diagnostic sampling suppressed",
                        requested.number(),
                        requested.name()
                    );
                } else {
                    eprintln!("[powdergame] demo reset error: {error}");
                }
                if let Some(inspector) = cell_inspector {
                    inspector.mark_unavailable(format!(
                        "Inspector unavailable: scenario staging failed: {error}"
                    ));
                }
            }
        }
    }
    if demo.step_pending {
        demo.step_pending = false;
        if !demo.playing {
            // N always advances EXACTLY ONE tick — unaffected by the
            // fast-forward multiplier.
            match simulation.tick() {
                Ok(()) => {
                    demo.ticks += 1;
                    demo.rate_ticks += 1;
                    if let Some(col) = collector {
                        if let Err(error) =
                            col.latch_first_tick_if_g6(simulation, demo.ticks, demo.fast)
                        {
                            eprintln!(
                                "[powdergame] observatory synchronous readback error: {error}; demo paused and collector reset"
                            );
                            demo.playing = false;
                            col.reset();
                            return;
                        }
                    }
                    println!("[powdergame] demo: stepped to tick {}", demo.ticks);
                }
                Err(e) => eprintln!("[powdergame] demo step error: {e}"),
            }
        }
    }
    if demo.playing {
        let interval = mode.tick_interval();
        let now = Instant::now();
        let prev = demo.last_tick.unwrap_or(now);
        let mut acc = now.duration_since(prev);
        while acc >= interval {
            // Fast-forward runs the production tick sequentially `fast` times
            // per beat — identical ticks, just more of them per opportunity.
            for _ in 0..demo.fast {
                match simulation.tick() {
                    Ok(()) => {
                        demo.ticks += 1;
                        demo.rate_ticks += 1;
                        if let Some(col) = collector {
                            if let Err(error) =
                                col.latch_first_tick_if_g6(simulation, demo.ticks, demo.fast)
                            {
                                eprintln!(
                                    "[powdergame] observatory synchronous readback error: {error}; demo paused and collector reset"
                                );
                                demo.playing = false;
                                col.reset();
                                return;
                            }
                        }
                    }
                    Err(e) => eprintln!("[powdergame] demo tick error: {e}"),
                }
            }
            acc -= interval;
        }
        demo.last_tick = Some(now - acc);
    }
    if let Some(col) = collector {
        if let Err(error) = col.update(simulation, demo.ticks, demo.fast) {
            eprintln!(
                "[powdergame] observatory asynchronous readback error: {error}; demo paused and collector reset"
            );
            demo.playing = false;
            col.reset();
        }
    }
    if mode == DemoMode::Gallery {
        sample_gallery_diagnostics(simulation, demo);
    }
}

fn sample_gallery_diagnostics(simulation: &Simulation, demo: &mut DemoState) {
    let source_tick = simulation.tick_count;
    let Some(gallery) = &mut demo.gallery else {
        return;
    };
    if !gallery.should_sample(source_tick) {
        return;
    }
    match simulation.activity_census() {
        Ok(census) => {
            gallery.record_sample(source_tick, census);
            let sample = gallery
                .diagnostic_sample()
                .expect("sample was just recorded");
            println!(
                "[powdergame][gallery][diagnostic] SAMPLE #{} | SOURCE TICK {} | active cells {}/{} (M {} T {} P {} R {}) | active chunks {}/{} | runnable {} | sleeping {} | out-of-band readback",
                sample.sequence,
                sample.source_tick,
                sample.census.any_active_cells,
                sample.census.total_cells,
                sample.census.matter_active_cells,
                sample.census.thermal_active_cells,
                sample.census.pressure_active_cells,
                sample.census.reaction_active_cells,
                sample.census.active_chunks,
                sample.census.total_chunks,
                sample.census.runnable_chunks,
                sample.census.sleeping_chunks,
            );
        }
        Err(error) => {
            gallery.defer_failed_sample(source_tick);
            eprintln!(
                "[powdergame][gallery][diagnostic] census failed at SOURCE TICK {source_tick}: {error}"
            );
        }
    }
}

fn print_gallery_runtime_context(
    simulation: &Simulation,
    demo: &DemoState,
    provenance: &RuntimeProvenance,
    scenario: ScenarioId,
) {
    let config = &simulation.world.config;
    println!("[powdergame][gallery] === build-bound provenance ===");
    println!(
        "[powdergame][gallery] Build source SHA: {}",
        provenance.source_sha
    );
    println!(
        "[powdergame][gallery] Build Git state:  {}",
        provenance.git_state.as_str()
    );
    println!(
        "[powdergame][gallery] Build profile: {}",
        provenance.build_profile
    );
    println!(
        "[powdergame][gallery] Scenario:      {}/6 {} ({})",
        scenario.number(),
        scenario.name(),
        scenario.slug()
    );
    println!(
        "[powdergame][gallery] WorldConfig:   {}x{} | chunk size {}",
        config.width, config.height, config.chunk_size
    );
    println!(
        "[powdergame][gallery] Sleep:         {} | threshold {}",
        if simulation.sleep_enabled {
            "ON"
        } else {
            "OFF"
        },
        simulation.sleep_threshold
    );
    println!(
        "[powdergame][gallery] SIM TICK:      {}",
        simulation.tick_count
    );
    let sample = demo
        .gallery
        .as_ref()
        .and_then(GalleryState::diagnostic_sample);
    match sample {
        Some(sample) => println!(
            "[powdergame][gallery] DIAGNOSTIC SAMPLE #{} | SOURCE TICK {}",
            sample.sequence, sample.source_tick
        ),
        None => println!("[powdergame][gallery] DIAGNOSTIC SAMPLE: pending"),
    }
    println!("[powdergame][gallery] Controls:      {GALLERY_CONTROLS}");
    println!("[powdergame][gallery] Starts PAUSED; diagnostics are outside timed benchmark paths");
}

fn gallery_hud_data(
    simulation: &Simulation,
    demo: &DemoState,
    provenance: &RuntimeProvenance,
    inspector: Option<InspectorHudData>,
    inspector_cursor: Option<[f32; 2]>,
    world_viewport: Option<ScreenRect>,
) -> Option<GalleryHudData> {
    let gallery = demo.gallery.as_ref()?;
    let scenario = gallery.scenario();
    Some(GalleryHudData {
        source_sha: provenance.source_sha.clone(),
        git_state: provenance.git_state.as_str(),
        build_profile: provenance.build_profile,
        scenario_number: gallery.scenario_number(),
        scenario_name: scenario.name(),
        scenario_description: scenario.description(),
        world_width: simulation.world.config.width,
        world_height: simulation.world.config.height,
        chunk_size: simulation.world.config.chunk_size,
        sleep_enabled: simulation.sleep_enabled,
        sleep_threshold: simulation.sleep_threshold,
        playing: demo.playing,
        fast: demo.fast,
        simulation_tick: gallery.is_ready().then_some(simulation.tick_count),
        diagnostic_sample: gallery.diagnostic_sample().cloned(),
        transition: gallery.transition().clone(),
        inspector,
        inspector_cursor,
        world_viewport,
    })
}

fn should_toggle_gallery_inspector(
    mode: DemoMode,
    experiment_worker: bool,
    state: ElementState,
    repeat: bool,
    character: Option<&str>,
) -> bool {
    cell_inspector_is_enabled(mode, experiment_worker)
        && state == ElementState::Pressed
        && !repeat
        && character.is_some_and(|value| value.eq_ignore_ascii_case("i"))
}

fn cell_inspector_is_enabled(mode: DemoMode, experiment_worker: bool) -> bool {
    mode == DemoMode::Gallery && !experiment_worker
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        if let Err(e) = self.init(event_loop) {
            eprintln!("[powdergame] FATAL: {e}");
            self.fatal_error = Some(e.to_string());
            event_loop.exit();
            return;
        }
        if self.experiment.is_some() {
            return;
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.clone() else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("[powdergame] close requested; exiting");
                self.shutdown_cell_inspector();
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let character = match &event.logical_key {
                    Key::Character(value) => Some(value.as_str()),
                    _ => None,
                };
                if should_toggle_gallery_inspector(
                    self.demo_mode,
                    self.experiment.is_some(),
                    event.state,
                    event.repeat,
                    character,
                ) {
                    self.toggle_cell_inspector_details(&window);
                    return;
                }
                if event.state != ElementState::Pressed || event.repeat {
                    return;
                }
                match event.logical_key {
                    Key::Named(NamedKey::Space) => self.toggle_play(&window),
                    Key::Named(NamedKey::Escape) => {
                        println!("[powdergame] ESC pressed; exiting");
                        self.shutdown_cell_inspector();
                        event_loop.exit();
                    }
                    Key::Character(ref c)
                        if self.demo_mode == DemoMode::Gallery
                            && matches!(c.as_str(), "1" | "2" | "3" | "4" | "5" | "6") =>
                    {
                        let number = c.as_bytes()[0] - b'0';
                        self.select_gallery_scenario(number, &window);
                    }
                    Key::Character(ref c)
                        if self.demo_mode != DemoMode::Gallery && c.eq_ignore_ascii_case("s") =>
                    {
                        self.toggle_sleep(&window);
                    }
                    Key::Character(ref c) if self.demo_mode != DemoMode::Gallery && c == "[" => {
                        self.adjust_sleep_threshold(-1, &window);
                    }
                    Key::Character(ref c) if self.demo_mode != DemoMode::Gallery && c == "]" => {
                        self.adjust_sleep_threshold(1, &window);
                    }
                    Key::Character(ref c) if c.eq_ignore_ascii_case("n") => {
                        self.request_step(&window);
                    }
                    Key::Character(ref c) if c.eq_ignore_ascii_case("f") => {
                        self.request_fast_forward(&window);
                    }
                    Key::Character(ref c) if c.eq_ignore_ascii_case("r") => {
                        self.request_reset(&window);
                    }
                    _ => {}
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.cell_inspector.is_some() {
                    self.cursor_position = Some(position);
                    window.request_redraw();
                }
            }
            WindowEvent::CursorLeft { .. } => {
                if let Some(inspector) = &mut self.cell_inspector {
                    self.cursor_position = None;
                    inspector.set_hover(None);
                    window.request_redraw();
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if let Some(simulation) = &mut self.simulation {
                    if let Some(demo) = &mut self.demo {
                        step_demo(
                            simulation,
                            demo,
                            &mut self.observatory_collector,
                            &mut self.cell_inspector,
                            self.demo_mode,
                        );
                    } else if let Err(e) = simulation.tick() {
                        eprintln!("[powdergame] tick error: {e}");
                    }
                }
                let inspector_now = Instant::now();
                self.refresh_cell_inspector(inspector_now);
                let inspector_hud = self.cell_inspector.as_ref().and_then(|inspector| {
                    self.simulation
                        .as_ref()
                        .map(|simulation| inspector.hud_data(simulation.tick_count, inspector_now))
                });
                let inspector_cursor = self
                    .cursor_position
                    .map(|position| [position.x as f32, position.y as f32]);
                let world_viewport = self
                    .renderer
                    .as_ref()
                    .and_then(Renderer::world_viewport)
                    .map(|viewport| ScreenRect {
                        x: viewport.x,
                        y: viewport.y,
                        width: viewport.width,
                        height: viewport.height,
                    });
                let gallery_hud = if self.demo_mode == DemoMode::Gallery {
                    self.simulation.as_ref().and_then(|simulation| {
                        self.demo.as_ref().and_then(|demo| {
                            self.gallery_provenance.as_ref().and_then(|provenance| {
                                gallery_hud_data(
                                    simulation,
                                    demo,
                                    provenance,
                                    inspector_hud.clone(),
                                    inspector_cursor,
                                    world_viewport,
                                )
                            })
                        })
                    })
                } else {
                    None
                };
                if let Some(renderer) = &mut self.renderer {
                    let hud_data = match self.demo_mode {
                        DemoMode::Thermal => self.observatory_collector.as_ref().map(|c| {
                            renderer::HudData::Thermal(
                                c.metrics(),
                                self.demo.as_ref().map(|d| d.ticks).unwrap_or(0),
                            )
                        }),
                        DemoMode::Pressure => self.observatory_collector.as_ref().map(|c| {
                            renderer::HudData::Pressure(
                                c.pressure_metrics(),
                                self.demo.as_ref().map(|d| d.ticks).unwrap_or(0),
                            )
                        }),
                        DemoMode::ParallelIntegrity => {
                            self.observatory_collector.as_ref().map(|c| {
                                renderer::HudData::ParallelIntegrity(
                                    c.integrity_metrics(),
                                    self.demo.as_ref().map(|d| d.ticks).unwrap_or(0),
                                )
                            })
                        }
                        DemoMode::Activity => self.observatory_collector.as_ref().map(|c| {
                            renderer::HudData::Activity(
                                c.activity_metrics(),
                                self.demo.as_ref().map(|d| d.ticks).unwrap_or(0),
                            )
                        }),
                        DemoMode::Gallery => gallery_hud.as_ref().map(renderer::HudData::Gallery),
                        _ => None,
                    };
                    if let Err(e) = renderer.render(hud_data) {
                        eprintln!("[powdergame] render error: {e}");
                        event_loop.exit();
                        return;
                    }
                }

                if self.demo_mode != DemoMode::None {
                    if let Some(demo) = &self.demo {
                        window.set_title(&demo.title());
                    }
                }

                self.frames_rendered += 1;
                if let Some(smoke) = self.smoke_frames {
                    if self.frames_rendered >= smoke {
                        println!(
                            "[powdergame] smoke run complete after {} frames; exiting",
                            self.frames_rendered
                        );
                        event_loop.exit();
                        return;
                    }
                }
                window.request_redraw();
            }
            _ => {}
        }
    }
}

/// Strictly parses `--smoke-frames N` or `POWDERGAME_SMOKE_FRAMES`.
///
/// CLI and environment configuration are mutually exclusive so a bounded
/// validation run cannot silently inherit or override a different limit.
fn smoke_frames_from_args<I, S>(
    args: I,
    environment_value: Option<&str>,
) -> Result<Option<u32>, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    fn positive_frames(value: &str, source: &str) -> Result<u32, String> {
        let parsed = value
            .parse::<u32>()
            .map_err(|error| format!("invalid {source} value '{value}': {error}"))?;
        if parsed == 0 {
            return Err(format!("{source} must be greater than zero"));
        }
        Ok(parsed)
    }

    let args = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();
    let mut cli_value = None;
    let mut index = 0usize;
    while index < args.len() {
        if args[index] == "--smoke-frames" {
            if cli_value.is_some() {
                return Err("duplicate --smoke-frames option".to_string());
            }
            let raw = args
                .get(index + 1)
                .filter(|value| !value.starts_with("--"))
                .ok_or_else(|| "missing value after --smoke-frames".to_string())?;
            cli_value = Some(positive_frames(raw, "--smoke-frames")?);
            index += 2;
            continue;
        }
        index += 1;
    }

    let environment_value = environment_value
        .map(|value| positive_frames(value, "POWDERGAME_SMOKE_FRAMES"))
        .transpose()?;
    match (cli_value, environment_value) {
        (Some(_), Some(_)) => Err(
            "--smoke-frames conflicts with POWDERGAME_SMOKE_FRAMES; set exactly one".to_string(),
        ),
        (Some(value), None) | (None, Some(value)) => Ok(Some(value)),
        (None, None) => Ok(None),
    }
}

fn parse_smoke_frames() -> Result<Option<u32>, String> {
    let environment_value = match std::env::var("POWDERGAME_SMOKE_FRAMES") {
        Ok(value) => Some(value),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err("POWDERGAME_SMOKE_FRAMES must be a Unicode positive integer".to_string());
        }
    };
    smoke_frames_from_args(std::env::args().skip(1), environment_value.as_deref())
}

/// Parses the user-facing mode. With no explicit mode, the canonical app opens
/// the Gallery; the empty G0 runtime remains available only through
/// `--runtime-baseline` (or the pre-existing demo environment variables).
fn parse_demo_mode() -> DemoMode {
    if let Some(cli_mode) = explicit_demo_mode_from_args(std::env::args().skip(1)) {
        return cli_mode;
    }
    if std::env::var("POWDERGAME_MOVEMENT_DEMO").as_deref() == Ok("1") {
        return DemoMode::Movement;
    }
    if std::env::var("POWDERGAME_DENSITY_DEMO").as_deref() == Ok("1") {
        return DemoMode::Density;
    }
    if std::env::var("POWDERGAME_THERMAL_DEMO").as_deref() == Ok("1") {
        return DemoMode::Thermal;
    }
    if std::env::var("POWDERGAME_PRESSURE_DEMO").as_deref() == Ok("1") {
        return DemoMode::Pressure;
    }
    DemoMode::Gallery
}

#[cfg(test)]
fn demo_mode_from_args<I, S>(args: I) -> DemoMode
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    explicit_demo_mode_from_args(args).unwrap_or(DemoMode::Gallery)
}

fn explicit_demo_mode_from_args<I, S>(args: I) -> Option<DemoMode>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    for arg in args {
        match arg.as_ref() {
            "--runtime-baseline" => return Some(DemoMode::None),
            "--movement-demo" => return Some(DemoMode::Movement),
            "--density-demo" => return Some(DemoMode::Density),
            "--thermal-demo" => return Some(DemoMode::Thermal),
            "--pressure-demo" => return Some(DemoMode::Pressure),
            "--parallel-integrity-demo" => return Some(DemoMode::ParallelIntegrity),
            "--activity-demo" => return Some(DemoMode::Activity),
            "--benchmark-gallery" => return Some(DemoMode::Gallery),
            _ => {}
        }
    }
    None
}

/// Experiment workers own their hidden Gallery-sized presentation surface and
/// must never be redirected by the user-facing default-mode policy.
fn mode_for_launch(experiment_worker: bool, requested_mode: DemoMode) -> DemoMode {
    if experiment_worker {
        DemoMode::Gallery
    } else {
        requested_mode
    }
}

fn experiment_worker_from_args<I, S>(args: I) -> Result<Option<ExperimentWorkerConfig>, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect();
    let worker_requested = args.iter().any(|arg| arg == "--experiment-worker");
    let experiment_option_present = args.iter().any(|arg| {
        matches!(
            arg.as_str(),
            "--experiment-run-dir"
                | "--experiment-run-id"
                | "--binary-sha256"
                | "--max-ticks"
                | "--diagnostic-interval"
                | "--consecutive-all-sleep"
                | "--post-sleep-ticks"
                | "--consecutive-reaction-zero"
                | "--post-reaction-ticks"
                | "--consecutive-persistent-opening"
                | "--post-opening-ticks"
                | "--terminal-window-samples"
        )
    });
    if !worker_requested {
        if experiment_option_present {
            return Err(
                "experiment options require '--experiment-worker sand-fall|water-flow|fire-heat|pressure-burst'"
                    .to_string(),
            );
        }
        return Ok(None);
    }

    let mut scenario = None;
    let mut run_dir = None;
    let mut run_id = None;
    let mut binary_sha256 = None;
    let mut max_ticks = None;
    let mut diagnostic_interval_ticks = None;
    let mut consecutive_all_sleep = None;
    let mut post_sleep_ticks = None;
    let mut consecutive_reaction_zero = None;
    let mut post_reaction_ticks = None;
    let mut consecutive_persistent_opening = None;
    let mut post_opening_ticks = None;
    let mut terminal_window_samples = None;
    let mut index = 0usize;
    while index < args.len() {
        let option = &args[index];
        index += 1;
        let value = |index: &mut usize| -> Result<String, String> {
            let value = args
                .get(*index)
                .ok_or_else(|| format!("missing value for {option}"))?
                .clone();
            *index += 1;
            Ok(value)
        };
        match option.as_str() {
            "--experiment-worker" => {
                if scenario.is_some() {
                    return Err("duplicate --experiment-worker".to_string());
                }
                let selected = value(&mut index)?;
                scenario = Some(match selected.as_str() {
                    "sand-fall" => ScenarioId::SandFall,
                    "water-flow" => ScenarioId::WaterFlow,
                    "fire-heat" => ScenarioId::FireHeat,
                    "pressure-burst" => ScenarioId::PressureBurst,
                    _ => {
                        return Err(format!(
                            "experiment worker supports only 'sand-fall', 'water-flow', 'fire-heat', or 'pressure-burst', got '{selected}'"
                        ));
                    }
                });
            }
            "--experiment-run-dir" => {
                if run_dir.is_some() {
                    return Err("duplicate --experiment-run-dir".to_string());
                }
                run_dir = Some(PathBuf::from(value(&mut index)?));
            }
            "--experiment-run-id" => {
                if run_id.is_some() {
                    return Err("duplicate --experiment-run-id".to_string());
                }
                run_id = Some(value(&mut index)?);
            }
            "--binary-sha256" => {
                if binary_sha256.is_some() {
                    return Err("duplicate --binary-sha256".to_string());
                }
                binary_sha256 = Some(value(&mut index)?);
            }
            "--max-ticks" => {
                if max_ticks.is_some() {
                    return Err("duplicate --max-ticks".to_string());
                }
                let raw = value(&mut index)?;
                max_ticks = Some(
                    raw.parse::<u64>()
                        .map_err(|error| format!("invalid --max-ticks '{raw}': {error}"))?,
                );
            }
            "--diagnostic-interval" => {
                if diagnostic_interval_ticks.is_some() {
                    return Err("duplicate --diagnostic-interval".to_string());
                }
                let raw = value(&mut index)?;
                diagnostic_interval_ticks =
                    Some(raw.parse::<u64>().map_err(|error| {
                        format!("invalid --diagnostic-interval '{raw}': {error}")
                    })?);
            }
            "--consecutive-all-sleep" => {
                if consecutive_all_sleep.is_some() {
                    return Err("duplicate --consecutive-all-sleep".to_string());
                }
                let raw = value(&mut index)?;
                consecutive_all_sleep = Some(raw.parse::<u32>().map_err(|error| {
                    format!("invalid --consecutive-all-sleep '{raw}': {error}")
                })?);
            }
            "--post-sleep-ticks" => {
                if post_sleep_ticks.is_some() {
                    return Err("duplicate --post-sleep-ticks".to_string());
                }
                let raw = value(&mut index)?;
                post_sleep_ticks = Some(
                    raw.parse::<u32>()
                        .map_err(|error| format!("invalid --post-sleep-ticks '{raw}': {error}"))?,
                );
            }
            "--consecutive-reaction-zero" => {
                if consecutive_reaction_zero.is_some() {
                    return Err("duplicate --consecutive-reaction-zero".to_string());
                }
                let raw = value(&mut index)?;
                consecutive_reaction_zero = Some(raw.parse::<u32>().map_err(|error| {
                    format!("invalid --consecutive-reaction-zero '{raw}': {error}")
                })?);
            }
            "--post-reaction-ticks" => {
                if post_reaction_ticks.is_some() {
                    return Err("duplicate --post-reaction-ticks".to_string());
                }
                let raw = value(&mut index)?;
                post_reaction_ticks =
                    Some(raw.parse::<u32>().map_err(|error| {
                        format!("invalid --post-reaction-ticks '{raw}': {error}")
                    })?);
            }
            "--consecutive-persistent-opening" => {
                if consecutive_persistent_opening.is_some() {
                    return Err("duplicate --consecutive-persistent-opening".to_string());
                }
                let raw = value(&mut index)?;
                consecutive_persistent_opening = Some(raw.parse::<u32>().map_err(|error| {
                    format!("invalid --consecutive-persistent-opening '{raw}': {error}")
                })?);
            }
            "--post-opening-ticks" => {
                if post_opening_ticks.is_some() {
                    return Err("duplicate --post-opening-ticks".to_string());
                }
                let raw = value(&mut index)?;
                post_opening_ticks =
                    Some(raw.parse::<u32>().map_err(|error| {
                        format!("invalid --post-opening-ticks '{raw}': {error}")
                    })?);
            }
            "--terminal-window-samples" => {
                if terminal_window_samples.is_some() {
                    return Err("duplicate --terminal-window-samples".to_string());
                }
                let raw = value(&mut index)?;
                terminal_window_samples = Some(raw.parse::<u32>().map_err(|error| {
                    format!("invalid --terminal-window-samples '{raw}': {error}")
                })?);
            }
            _ => {
                return Err(format!("unknown experiment worker argument '{option}'"));
            }
        }
    }

    let scenario = scenario.expect("worker marker was observed");
    let experiment_id = match scenario {
        ScenarioId::SandFall => EXPERIMENT_ID,
        ScenarioId::WaterFlow => WATER_EXPERIMENT_ID,
        ScenarioId::FireHeat => FIRE_EXPERIMENT_ID,
        ScenarioId::PressureBurst => PRESSURE_EXPERIMENT_ID,
        _ => unreachable!("worker parser accepts four scenarios"),
    };
    let (
        consecutive_all_sleep,
        post_sleep_ticks,
        consecutive_reaction_zero,
        post_reaction_ticks,
        consecutive_persistent_opening,
        post_opening_ticks,
        terminal_window_samples,
    ) = match scenario {
        ScenarioId::SandFall | ScenarioId::WaterFlow => {
            if consecutive_reaction_zero.is_some()
                || post_reaction_ticks.is_some()
                || consecutive_persistent_opening.is_some()
                || post_opening_ticks.is_some()
                || terminal_window_samples.is_some()
            {
                return Err(
                    "Fire/Pressure lifecycle options are not valid for Sand/Water".to_string(),
                );
            }
            (
                consecutive_all_sleep
                    .ok_or_else(|| "missing --consecutive-all-sleep".to_string())?,
                post_sleep_ticks.ok_or_else(|| "missing --post-sleep-ticks".to_string())?,
                0,
                0,
                0,
                0,
                0,
            )
        }
        ScenarioId::FireHeat => {
            if consecutive_all_sleep.is_some()
                || post_sleep_ticks.is_some()
                || consecutive_persistent_opening.is_some()
                || post_opening_ticks.is_some()
                || terminal_window_samples.is_some()
            {
                return Err(
                    "Sand/Water/Pressure lifecycle options are not valid for Fire / Heat"
                        .to_string(),
                );
            }
            (
                0,
                0,
                consecutive_reaction_zero
                    .ok_or_else(|| "missing --consecutive-reaction-zero".to_string())?,
                post_reaction_ticks.ok_or_else(|| "missing --post-reaction-ticks".to_string())?,
                0,
                0,
                0,
            )
        }
        ScenarioId::PressureBurst => {
            if consecutive_all_sleep.is_some()
                || post_sleep_ticks.is_some()
                || consecutive_reaction_zero.is_some()
                || post_reaction_ticks.is_some()
            {
                return Err(
                    "Sand/Water/Fire lifecycle options are not valid for Pressure Burst"
                        .to_string(),
                );
            }
            (
                0,
                0,
                0,
                0,
                consecutive_persistent_opening
                    .ok_or_else(|| "missing --consecutive-persistent-opening".to_string())?,
                post_opening_ticks.ok_or_else(|| "missing --post-opening-ticks".to_string())?,
                terminal_window_samples
                    .ok_or_else(|| "missing --terminal-window-samples".to_string())?,
            )
        }
        _ => unreachable!("worker parser accepts four scenarios"),
    };
    Ok(Some(ExperimentWorkerConfig {
        experiment_id: experiment_id.to_string(),
        run_id: run_id.ok_or_else(|| "missing --experiment-run-id".to_string())?,
        run_dir: run_dir.ok_or_else(|| "missing --experiment-run-dir".to_string())?,
        scenario,
        binary_sha256: binary_sha256.ok_or_else(|| "missing --binary-sha256".to_string())?,
        max_ticks: max_ticks.ok_or_else(|| "missing --max-ticks".to_string())?,
        diagnostic_interval_ticks: diagnostic_interval_ticks
            .ok_or_else(|| "missing --diagnostic-interval".to_string())?,
        consecutive_all_sleep,
        post_sleep_ticks,
        consecutive_reaction_zero,
        post_reaction_ticks,
        consecutive_persistent_opening,
        post_opening_ticks,
        terminal_window_samples,
    }))
}

fn main() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

    let experiment = match experiment_worker_from_args(std::env::args().skip(1)) {
        Ok(experiment) => experiment,
        Err(error) => {
            eprintln!("[powdergame][experiment] argument error: {error}");
            process::exit(2);
        }
    };
    if let Some(config) = experiment.as_ref() {
        if let Err(error) = verify_current_executable_sha256(&config.binary_sha256) {
            eprintln!("[powdergame][experiment] binary authentication error: {error}");
            process::exit(2);
        }
    }
    let smoke_frames = match parse_smoke_frames() {
        Ok(frames) => frames,
        Err(error) => {
            eprintln!("[powdergame][smoke] argument error: {error}");
            process::exit(2);
        }
    };
    if let Some(n) = smoke_frames {
        println!("[powdergame] smoke run: will exit after {n} frames");
    }
    let requested_mode = parse_demo_mode();
    let demo_mode = mode_for_launch(experiment.is_some(), requested_mode);
    match demo_mode {
        DemoMode::Movement => println!(
            "[powdergame] movement demo: 128×128 stylized-forest scene, \
             starts PAUSED (SPACE play | N step | R reset | ESC quit)"
        ),
        DemoMode::Density => println!(
            "[powdergame] density demo: 128×128 laboratory tanks \
             (SAND+WATER | WATER+OIL | STEAM+SMOKE), starts PAUSED \
             (SPACE play | N step | R reset | ESC quit)"
        ),
        DemoMode::Thermal => println!(
            "[powdergame] thermal demo: 320×192 thermal observatory \
             (4 large panels + live diagnostic metrics), 60 TPS, starts PAUSED \
             (SPACE play | N step | R reset | ESC quit)"
        ),
        DemoMode::Pressure => println!(
            "[powdergame] pressure demo: 128×128 twin boilers, 60 TPS. \
             LEFT Wood relief plug should rupture/vent; RIGHT Stone control stays sealed. \
             Starts PAUSED (SPACE play | N step | R reset | ESC quit)"
        ),
        DemoMode::Activity => println!(
            "[powdergame] activity demo: 256x256 G7 active/sleep observatory, 60 TPS. \
             A stable water bulk | B true stable steam bulk | C stable-duration/wake-candidate | D slow active. \
             Starts PAUSED (SPACE play | F fast x1/x4/x16 | N step | R reset | ESC quit)"
        ),
        DemoMode::ParallelIntegrity => println!(
            "[powdergame] parallel integrity demo: 256x256 2x2 contention lab, 60 TPS. \
             Starts PAUSED (SPACE play | F fast x1/x4/x16 | N step | R reset | ESC quit)"
        ),
        DemoMode::Gallery => println!(
            "[powdergame] G8-B benchmark scenario Gallery: 256x256, six shared headless fixtures. \
             Starts PAUSED ({GALLERY_CONTROLS}). Diagnostic census is bounded and outside timed benchmark paths."
        ),
        DemoMode::None => println!(
            "[powdergame] G0 Runtime: 2048x2048 empty technical baseline (explicit diagnostic mode)"
        ),
    }

    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = App::new(smoke_frames, demo_mode, experiment);
    if let Err(error) = event_loop.run_app(&mut app) {
        eprintln!("[powdergame] event loop failed: {error}");
        process::exit(1);
    }
    if let Some(error) = app.fatal_error {
        eprintln!("[powdergame] incomplete run: {error}");
        process::exit(1);
    }
    println!("[powdergame] exited cleanly");
}

#[cfg(test)]
mod tests {
    use super::{
        cell_inspector_is_enabled, demo_mode_from_args, experiment_worker_from_args,
        mode_for_launch, should_toggle_gallery_inspector, smoke_frames_from_args, DemoMode,
        DemoState,
    };
    use powdergame_core::{WorldConfig, ACTIVITY_MATTER};
    use powdergame_gpu::{ActivityCensusReport, Simulation};
    use powdergame_scenarios::{reset_and_stage_scenario, ScenarioId};
    use winit::event::ElementState;

    #[test]
    fn smoke_frames_accepts_exactly_one_positive_cli_or_environment_value() {
        assert_eq!(
            smoke_frames_from_args(["--smoke-frames", "60"], None).unwrap(),
            Some(60)
        );
        assert_eq!(
            smoke_frames_from_args(["--benchmark-gallery"], Some("120")).unwrap(),
            Some(120)
        );
        assert_eq!(
            smoke_frames_from_args(["--benchmark-gallery"], None).unwrap(),
            None
        );
    }

    #[test]
    fn smoke_frames_rejects_missing_invalid_overflow_zero_and_duplicate_values() {
        assert!(smoke_frames_from_args(["--smoke-frames"], None).is_err());
        assert!(smoke_frames_from_args(["--smoke-frames", "--benchmark-gallery"], None).is_err());
        assert!(smoke_frames_from_args(["--smoke-frames", "many"], None).is_err());
        assert!(smoke_frames_from_args(["--smoke-frames", "4294967296"], None).is_err());
        assert!(smoke_frames_from_args(["--smoke-frames", "0"], None).is_err());
        assert!(
            smoke_frames_from_args(["--smoke-frames", "1", "--smoke-frames", "2"], None).is_err()
        );
        assert!(smoke_frames_from_args(["--benchmark-gallery"], Some("many")).is_err());
        assert!(smoke_frames_from_args(["--benchmark-gallery"], Some("4294967296")).is_err());
        assert!(smoke_frames_from_args(["--benchmark-gallery"], Some("0")).is_err());
    }

    #[test]
    fn smoke_frames_rejects_cli_environment_conflict() {
        let error = smoke_frames_from_args(["--smoke-frames", "60"], Some("60"))
            .expect_err("CLI and environment must not silently override one another");
        assert!(error.contains("conflicts"));
    }

    #[test]
    fn user_mode_defaults_to_gallery_with_no_args_or_smoke_only() {
        assert_eq!(
            demo_mode_from_args(std::iter::empty::<&str>()),
            DemoMode::Gallery
        );
        assert_eq!(
            demo_mode_from_args(["--smoke-frames", "3"]),
            DemoMode::Gallery
        );
        assert_eq!(
            smoke_frames_from_args(["--smoke-frames", "3"], None).unwrap(),
            Some(3)
        );
    }

    #[test]
    fn runtime_baseline_is_explicit_and_remains_bounded_with_smoke_frames() {
        assert_eq!(demo_mode_from_args(["--runtime-baseline"]), DemoMode::None);
        assert_eq!(
            demo_mode_from_args(["--runtime-baseline", "--smoke-frames", "3"]),
            DemoMode::None
        );
        assert_eq!(
            smoke_frames_from_args(["--runtime-baseline", "--smoke-frames", "3"], None).unwrap(),
            Some(3)
        );
    }

    #[test]
    fn existing_explicit_demo_flags_keep_their_modes() {
        for (flag, expected) in [
            ("--movement-demo", DemoMode::Movement),
            ("--density-demo", DemoMode::Density),
            ("--thermal-demo", DemoMode::Thermal),
            ("--pressure-demo", DemoMode::Pressure),
            ("--parallel-integrity-demo", DemoMode::ParallelIntegrity),
            ("--activity-demo", DemoMode::Activity),
            ("--benchmark-gallery", DemoMode::Gallery),
        ] {
            assert_eq!(demo_mode_from_args([flag]), expected, "flag={flag}");
        }
    }

    #[test]
    fn gallery_inspector_toggle_is_pressed_once_and_never_enters_worker_or_other_modes() {
        assert!(cell_inspector_is_enabled(DemoMode::Gallery, false));
        assert!(!cell_inspector_is_enabled(DemoMode::Gallery, true));
        assert!(!cell_inspector_is_enabled(DemoMode::None, false));
        for character in ["i", "I"] {
            assert!(should_toggle_gallery_inspector(
                DemoMode::Gallery,
                false,
                ElementState::Pressed,
                false,
                Some(character),
            ));
        }
        for (mode, worker, state, repeat, character) in [
            (
                DemoMode::Gallery,
                false,
                ElementState::Released,
                false,
                Some("i"),
            ),
            (
                DemoMode::Gallery,
                false,
                ElementState::Pressed,
                true,
                Some("i"),
            ),
            (
                DemoMode::Gallery,
                false,
                ElementState::Pressed,
                false,
                Some("n"),
            ),
            (DemoMode::Gallery, false, ElementState::Pressed, false, None),
            (
                DemoMode::Gallery,
                true,
                ElementState::Pressed,
                false,
                Some("i"),
            ),
            (
                DemoMode::None,
                false,
                ElementState::Pressed,
                false,
                Some("i"),
            ),
            (
                DemoMode::Activity,
                false,
                ElementState::Pressed,
                false,
                Some("i"),
            ),
        ] {
            assert!(!should_toggle_gallery_inspector(
                mode, worker, state, repeat, character,
            ));
        }
    }

    #[test]
    fn experiment_worker_routing_stays_ahead_of_user_mode_selection() {
        assert_eq!(
            mode_for_launch(true, DemoMode::None),
            DemoMode::Gallery,
            "workers retain the hidden Gallery-sized presentation surface"
        );
        assert_eq!(
            mode_for_launch(true, DemoMode::Movement),
            DemoMode::Gallery,
            "explicit user modes cannot redirect an experiment worker"
        );
        assert_eq!(mode_for_launch(false, DemoMode::None), DemoMode::None);
    }

    #[test]
    fn experiment_worker_cli_is_strict_and_complete() {
        let run_dir = r"C:\outside\sand-run";
        let hash = "a".repeat(64);
        let args = vec![
            "--experiment-worker".to_string(),
            "sand-fall".to_string(),
            "--experiment-run-dir".to_string(),
            run_dir.to_string(),
            "--experiment-run-id".to_string(),
            "g8b-sand-fall-v0-test".to_string(),
            "--binary-sha256".to_string(),
            hash.clone(),
            "--max-ticks".to_string(),
            "20000".to_string(),
            "--diagnostic-interval".to_string(),
            "8".to_string(),
            "--consecutive-all-sleep".to_string(),
            "3".to_string(),
            "--post-sleep-ticks".to_string(),
            "180".to_string(),
        ];
        let parsed = experiment_worker_from_args(args)
            .expect("valid worker arguments")
            .expect("worker selected");
        assert_eq!(parsed.scenario, ScenarioId::SandFall);
        assert_eq!(parsed.run_dir.to_string_lossy(), run_dir);
        assert_eq!(parsed.run_id, "g8b-sand-fall-v0-test");
        assert_eq!(parsed.binary_sha256, hash);
        assert_eq!(parsed.max_ticks, 20_000);
        assert_eq!(parsed.diagnostic_interval_ticks, 8);
        assert_eq!(parsed.consecutive_all_sleep, 3);
        assert_eq!(parsed.post_sleep_ticks, 180);
        assert_eq!(parsed.consecutive_reaction_zero, 0);
        assert_eq!(parsed.post_reaction_ticks, 0);
        assert_eq!(parsed.consecutive_persistent_opening, 0);
        assert_eq!(parsed.post_opening_ticks, 0);
        assert_eq!(parsed.terminal_window_samples, 0);
    }

    #[test]
    fn experiment_worker_cli_selects_water_contract_without_changing_sand_defaults() {
        let parsed = experiment_worker_from_args([
            "--experiment-worker",
            "water-flow",
            "--experiment-run-dir",
            r"C:\outside\water-run",
            "--experiment-run-id",
            "g8b-water-flow-v0-test",
            "--binary-sha256",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "--max-ticks",
            "20000",
            "--diagnostic-interval",
            "8",
            "--consecutive-all-sleep",
            "3",
            "--post-sleep-ticks",
            "180",
        ])
        .expect("valid Water worker arguments")
        .expect("Water worker selected");
        assert_eq!(parsed.experiment_id, "g8b-water-flow-v0");
        assert_eq!(parsed.scenario, ScenarioId::WaterFlow);
        assert_eq!(parsed.run_id, "g8b-water-flow-v0-test");
        assert_eq!(parsed.consecutive_reaction_zero, 0);
        assert_eq!(parsed.post_reaction_ticks, 0);
        assert_eq!(parsed.consecutive_persistent_opening, 0);
        assert_eq!(parsed.post_opening_ticks, 0);
        assert_eq!(parsed.terminal_window_samples, 0);
    }

    #[test]
    fn experiment_worker_cli_selects_fire_specific_lifecycle() {
        let parsed = experiment_worker_from_args([
            "--experiment-worker",
            "fire-heat",
            "--experiment-run-dir",
            r"C:\outside\fire-run",
            "--experiment-run-id",
            "g8b-fire-heat-v0-test",
            "--binary-sha256",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "--max-ticks",
            "20000",
            "--diagnostic-interval",
            "8",
            "--consecutive-reaction-zero",
            "3",
            "--post-reaction-ticks",
            "180",
        ])
        .expect("valid Fire worker arguments")
        .expect("Fire worker selected");
        assert_eq!(parsed.experiment_id, "g8b-fire-heat-v0");
        assert_eq!(parsed.scenario, ScenarioId::FireHeat);
        assert_eq!(parsed.consecutive_all_sleep, 0);
        assert_eq!(parsed.post_sleep_ticks, 0);
        assert_eq!(parsed.consecutive_reaction_zero, 3);
        assert_eq!(parsed.post_reaction_ticks, 180);
        assert_eq!(parsed.consecutive_persistent_opening, 0);
        assert_eq!(parsed.post_opening_ticks, 0);
        assert_eq!(parsed.terminal_window_samples, 0);
    }

    #[test]
    fn experiment_worker_cli_selects_pressure_specific_lifecycle() {
        let parsed = experiment_worker_from_args([
            "--experiment-worker",
            "pressure-burst",
            "--experiment-run-dir",
            r"C:\outside\pressure-run",
            "--experiment-run-id",
            "g8b-pressure-burst-v0-test",
            "--binary-sha256",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "--max-ticks",
            "20000",
            "--diagnostic-interval",
            "8",
            "--consecutive-persistent-opening",
            "3",
            "--post-opening-ticks",
            "180",
            "--terminal-window-samples",
            "64",
        ])
        .expect("valid Pressure worker arguments")
        .expect("Pressure worker selected");
        assert_eq!(parsed.experiment_id, "g8b-pressure-burst-v0");
        assert_eq!(parsed.scenario, ScenarioId::PressureBurst);
        assert_eq!(parsed.consecutive_all_sleep, 0);
        assert_eq!(parsed.post_sleep_ticks, 0);
        assert_eq!(parsed.consecutive_reaction_zero, 0);
        assert_eq!(parsed.post_reaction_ticks, 0);
        assert_eq!(parsed.consecutive_persistent_opening, 3);
        assert_eq!(parsed.post_opening_ticks, 180);
        assert_eq!(parsed.terminal_window_samples, 64);
    }

    #[test]
    fn experiment_worker_cli_rejects_missing_unknown_and_unsupported_modes() {
        assert!(experiment_worker_from_args(["--experiment-run-id", "orphan"]).is_err());
        assert!(experiment_worker_from_args(["--experiment-worker", "heavy-mixed-world"]).is_err());
        assert!(
            experiment_worker_from_args(["--experiment-worker", "sand-fall", "--unknown"]).is_err()
        );
        assert!(experiment_worker_from_args([
            "--experiment-worker",
            "fire-heat",
            "--consecutive-all-sleep",
            "3"
        ])
        .is_err());
        assert!(experiment_worker_from_args([
            "--experiment-worker",
            "pressure-burst",
            "--consecutive-reaction-zero",
            "3"
        ])
        .is_err());
        assert!(experiment_worker_from_args([
            "--experiment-worker",
            "water-flow",
            "--post-opening-ticks",
            "180"
        ])
        .is_err());
        assert!(experiment_worker_from_args([
            "--experiment-worker",
            "fire-heat",
            "--terminal-window-samples",
            "64"
        ])
        .is_err());
        assert!(experiment_worker_from_args(["--benchmark-gallery"])
            .expect("non-worker arguments are unchanged")
            .is_none());
    }

    /// G7-A actual-fixture long-run correctness validation (RTX 5090 / DX12).
    /// Uses the shared `ActiveSleepG7` geometry with 3000+ production
    /// `Simulation::tick()` calls. This is correctness evidence — NOT a
    /// performance benchmark (see G8).
    #[test]
    #[ignore = "RTX 5090 / DX12 3000-tick G7-A fixture validation"]
    fn activity_demo_long_run_3000_ticks() {
        let context =
            pollster::block_on(powdergame_gpu::GpuContext::new()).expect("DX12 GPU context");
        let mut sim = Simulation::with_context(
            context,
            WorldConfig::new(256, 256, 64).expect("world config"),
        )
        .expect("simulation init");
        reset_and_stage_scenario(&mut sim, ScenarioId::ActiveSleepG7)
            .expect("stage shared activity demo");

        let cidx = |cx: u32, cy: u32| -> usize { (cy * 4 + cx) as usize };
        // Panel B (top-right quadrant): all four chunks.
        let b = [cidx(2, 0), cidx(3, 0), cidx(2, 1), cidx(3, 1)];
        // Panel A sealed-control chunks (no frontier expected; the draining
        // shaft is contained in chunk (1,1)).
        let a_control = [cidx(0, 0), cidx(1, 0), cidx(0, 1)];
        // Panel C target lower-right chunk (stable first, then Sand arrives).
        let c_target = cidx(1, 3);

        let acts = |s: &Simulation| -> Vec<u32> {
            s.world
                .read_chunk_activity_all(&s.context.device, &s.context.queue)
                .expect("chunk activity readback")
        };
        let stables = |s: &Simulation| -> Vec<u32> {
            s.world
                .read_chunk_stable_all(&s.context.device, &s.context.queue)
                .expect("chunk stable readback")
        };

        let mut c_pre_arrival: u32 = 0;
        let mut c_arrival: Option<u64> = None;
        let mut c_arrival_stable_zero = false;
        let mut b_clean_at: Vec<u64> = Vec::new();

        for t in 1..=3000u64 {
            sim.tick().expect("production tick");
            let sample = t <= 200 || t % 50 == 0 || t >= 2900;
            if !sample {
                continue;
            }
            let a = acts(&sim);
            let s = stables(&sim);

            // C: capture the target chunk's stable ticks before the Sand
            // frontier arrives (first clean sample of the stable chunk).
            if c_arrival.is_none() && a[c_target] & ACTIVITY_MATTER == 0 && c_pre_arrival == 0 {
                c_pre_arrival = s[c_target];
            }
            // C: first MATTER arrival resets the target's stable counter.
            if c_arrival.is_none() && a[c_target] & ACTIVITY_MATTER != 0 {
                c_arrival = Some(t);
                c_arrival_stable_zero = s[c_target] == 0;
            }
            // B: fully stable on every sampled late tick.
            if t % 50 == 0 && b.iter().all(|&i| a[i] == 0) {
                b_clean_at.push(t);
            }
        }

        // B: exact masks and monotonic stable counters at the late state.
        let a = acts(&sim);
        let s = stables(&sim);
        for &i in &b {
            assert_eq!(
                a[i], 0,
                "B chunk {i} must be fully stable at t=3000 (mask {:#x})",
                a[i]
            );
            let b_stable = s[i];
            assert!(
                b_stable >= 1000,
                "B chunk {i} stable ticks {b_stable} must be monotonic across the long run"
            );
        }
        assert!(
            b_clean_at.len() >= 3,
            "B must be fully stable at multiple late samples (got {b_clean_at:?})"
        );
        // A: no cross-panel THERMAL contamination — sealed controls stay clean.
        for &i in &a_control {
            assert_eq!(
                a[i], 0,
                "A control chunk {i} must have no frontier at t=3000 (mask {:#x})",
                a[i]
            );
        }
        // C: target was stable first, then a real Sand frontier arrived and
        // reset the stable counter to 0.
        assert!(
            c_pre_arrival > 0,
            "C target chunk must accumulate stable ticks before Sand arrival"
        );
        let arrival = c_arrival.expect("C target chunk must receive MATTER activity");
        assert!(
            c_arrival_stable_zero,
            "C target stable counter must reset to 0 on arrival"
        );
        println!(
            "[powdergame][G7-A] long-run: C pre-arrival stable={c_pre_arrival}              first MATTER arrival tick={arrival} reset_to_zero={c_arrival_stable_zero}"
        );
    }

    /// Validates that `reset_demo_world` returns the simulation to the exact pristine
    /// staged state byte-for-byte across all GPU buffers, resets tick_count to 0,
    /// preserves sleep optimization settings, is idempotent across repeated resets,
    /// and matches a freshly constructed fixture on tick 1.
    #[test]
    fn activity_demo_reset_exact_equivalence() {
        let context1 =
            pollster::block_on(powdergame_gpu::GpuContext::new()).expect("DX12 GPU context");
        let mut sim_reset = Simulation::with_context(
            context1,
            WorldConfig::new(256, 256, 64).expect("world config"),
        )
        .expect("simulation init");
        sim_reset.set_sleep_enabled(true);
        sim_reset.set_sleep_threshold(7);
        reset_and_stage_scenario(&mut sim_reset, ScenarioId::ActiveSleepG7)
            .expect("stage shared activity demo");

        let context2 =
            pollster::block_on(powdergame_gpu::GpuContext::new()).expect("DX12 GPU context");
        let mut sim_fresh = Simulation::with_context(
            context2,
            WorldConfig::new(256, 256, 64).expect("world config"),
        )
        .expect("simulation init");
        sim_fresh.set_sleep_enabled(true);
        sim_fresh.set_sleep_threshold(7);
        reset_and_stage_scenario(&mut sim_fresh, ScenarioId::ActiveSleepG7)
            .expect("stage shared fresh demo");

        // Run sim_reset for 50 ticks to diverge all physics, combustion, movement, and activity state
        for _ in 0..50 {
            sim_reset.tick().expect("tick");
        }
        assert_eq!(sim_reset.tick_count, 50);

        // Reset sim_reset
        super::reset_demo_world(&mut sim_reset, super::DemoMode::Activity, None)
            .expect("reset demo world");

        // 1. tick_count == 0
        assert_eq!(sim_reset.tick_count, 0, "tick_count must reset to 0");
        // 2. sleep settings preserved
        assert!(sim_reset.sleep_enabled, "sleep_enabled must be preserved");
        assert_eq!(
            sim_reset.sleep_threshold, 7,
            "sleep_threshold must be preserved"
        );

        // 3. Exact buffer match with freshly staged world
        let m_r = sim_reset
            .world
            .read_material_all(&sim_reset.context.device, &sim_reset.context.queue)
            .unwrap();
        let m_f = sim_fresh
            .world
            .read_material_all(&sim_fresh.context.device, &sim_fresh.context.queue)
            .unwrap();
        assert_eq!(m_r, m_f, "material buffer mismatch after reset");

        let t_r = sim_reset
            .world
            .read_temperature_all(&sim_reset.context.device, &sim_reset.context.queue)
            .unwrap();
        let t_f = sim_fresh
            .world
            .read_temperature_all(&sim_fresh.context.device, &sim_fresh.context.queue)
            .unwrap();
        assert_eq!(t_r, t_f, "temperature buffer mismatch after reset");

        let p_r = sim_reset
            .world
            .read_pressure_all(&sim_reset.context.device, &sim_reset.context.queue)
            .unwrap();
        let p_f = sim_fresh
            .world
            .read_pressure_all(&sim_fresh.context.device, &sim_fresh.context.queue)
            .unwrap();
        assert_eq!(p_r, p_f, "pressure buffer mismatch after reset");

        let fl_r = sim_reset
            .world
            .read_flags_all(&sim_reset.context.device, &sim_reset.context.queue)
            .unwrap();
        let fl_f = sim_fresh
            .world
            .read_flags_all(&sim_fresh.context.device, &sim_fresh.context.queue)
            .unwrap();
        assert_eq!(fl_r, fl_f, "flags buffer mismatch after reset");

        let act_r = sim_reset
            .world
            .read_chunk_activity_all(&sim_reset.context.device, &sim_reset.context.queue)
            .unwrap();
        let act_f = sim_fresh
            .world
            .read_chunk_activity_all(&sim_fresh.context.device, &sim_fresh.context.queue)
            .unwrap();
        assert_eq!(act_r, act_f, "chunk_activity mismatch after reset");

        let st_r = sim_reset
            .world
            .read_chunk_state_all(&sim_reset.context.device, &sim_reset.context.queue)
            .unwrap();
        let st_f = sim_fresh
            .world
            .read_chunk_state_all(&sim_fresh.context.device, &sim_fresh.context.queue)
            .unwrap();
        assert_eq!(st_r, st_f, "chunk_state mismatch after reset");

        let stb_r = sim_reset
            .world
            .read_chunk_stable_all(&sim_reset.context.device, &sim_reset.context.queue)
            .unwrap();
        let stb_f = sim_fresh
            .world
            .read_chunk_stable_all(&sim_fresh.context.device, &sim_fresh.context.queue)
            .unwrap();
        assert_eq!(stb_r, stb_f, "chunk_stable mismatch after reset");

        let rsn_r = sim_reset
            .world
            .read_chunk_wake_reason_all(&sim_reset.context.device, &sim_reset.context.queue)
            .unwrap();
        let rsn_f = sim_fresh
            .world
            .read_chunk_wake_reason_all(&sim_fresh.context.device, &sim_fresh.context.queue)
            .unwrap();
        assert_eq!(rsn_r, rsn_f, "chunk_wake_reason mismatch after reset");

        let ew_r = sim_reset
            .world
            .read_chunk_edit_wake_all(&sim_reset.context.device, &sim_reset.context.queue)
            .unwrap();
        let ew_f = sim_fresh
            .world
            .read_chunk_edit_wake_all(&sim_fresh.context.device, &sim_fresh.context.queue)
            .unwrap();
        assert_eq!(ew_r, ew_f, "chunk_edit_wake mismatch after reset");

        // 4. Repeated reset idempotency: run for 20 ticks, reset again
        for _ in 0..20 {
            sim_reset.tick().expect("tick");
        }
        super::reset_demo_world(&mut sim_reset, super::DemoMode::Activity, None)
            .expect("repeat reset");
        assert_eq!(sim_reset.tick_count, 0);
        let m_r2 = sim_reset
            .world
            .read_material_all(&sim_reset.context.device, &sim_reset.context.queue)
            .unwrap();
        assert_eq!(m_r2, m_f, "material mismatch on second reset");

        // 5. Next tick is tick 1 and matches fresh tick 1 exactly
        sim_reset.tick().expect("tick 1 reset");
        sim_fresh.tick().expect("tick 1 fresh");
        assert_eq!(sim_reset.tick_count, 1);
        let m_r_t1 = sim_reset
            .world
            .read_material_all(&sim_reset.context.device, &sim_reset.context.queue)
            .unwrap();
        let m_f_t1 = sim_fresh
            .world
            .read_material_all(&sim_fresh.context.device, &sim_fresh.context.queue)
            .unwrap();
        assert_eq!(
            m_r_t1, m_f_t1,
            "tick 1 material outcome mismatch between reset and fresh"
        );
    }

    #[test]
    fn benchmark_gallery_argument_selects_gallery_mode() {
        assert_eq!(
            demo_mode_from_args(["--benchmark-gallery"]),
            DemoMode::Gallery
        );
    }

    #[test]
    fn gallery_control_state_preserves_one_tick_and_pristine_reset_contracts() {
        let mut demo = DemoState::new(
            super::GALLERY_TITLE,
            super::GALLERY_TPS,
            false,
            Some(super::GalleryState::new()),
        );
        assert!(!demo.playing, "Gallery must start paused");
        assert!(demo.queue_single_step());
        assert!(demo.step_pending);

        demo.step_pending = false;
        demo.playing = true;
        assert!(!demo.queue_single_step(), "N is ignored while playing");
        assert!(!demo.step_pending);

        demo.cycle_fast();
        assert_eq!(demo.fast, 4);
        demo.cycle_fast();
        assert_eq!(demo.fast, 16);
        demo.cycle_fast();
        assert_eq!(demo.fast, 1);

        demo.ticks = 42;
        demo.fast = 16;
        demo.playing = true;
        demo.gallery.as_mut().unwrap().record_sample(
            42,
            ActivityCensusReport {
                total_cells: 1,
                any_active_cells: 0,
                matter_active_cells: 0,
                thermal_active_cells: 0,
                pressure_active_cells: 0,
                reaction_active_cells: 0,
                total_chunks: 1,
                active_chunks: 0,
                runnable_chunks: 1,
                sleeping_chunks: 0,
            },
        );
        assert!(demo.gallery.as_ref().unwrap().diagnostic_sample().is_some());
        let committed = demo.gallery.as_ref().unwrap().scenario();
        assert_eq!(
            demo.gallery.as_mut().unwrap().request_number(6),
            Some(ScenarioId::ActiveSleepG7)
        );
        demo.queue_pristine_reset();
        assert!(!demo.playing);
        assert_eq!(demo.ticks, 42, "tick attribution is not committed early");
        assert_eq!(demo.fast, 1);
        assert!(demo.reset_pending);
        assert!(demo.gallery.as_ref().unwrap().diagnostic_sample().is_some());
        assert_eq!(demo.gallery.as_ref().unwrap().scenario(), committed);
        assert!(!demo.gallery_ready_to_advance());
        assert!(!demo.queue_single_step());

        demo.gallery
            .as_mut()
            .unwrap()
            .commit_reset_failure("injected".to_string());
        assert!(!demo.gallery_ready_to_advance());
        assert_eq!(demo.ticks, 42);
        assert!(demo.gallery.as_ref().unwrap().diagnostic_sample().is_some());

        demo.gallery.as_mut().unwrap().request_number(6);
        assert_eq!(
            demo.gallery.as_mut().unwrap().commit_reset_success(),
            Some(ScenarioId::ActiveSleepG7)
        );
        demo.commit_pristine_reset();
        assert_eq!(demo.ticks, 0);
        assert!(demo.gallery.as_ref().unwrap().diagnostic_sample().is_none());
        assert!(demo.gallery_ready_to_advance());
        assert_eq!(
            demo.gallery.as_ref().unwrap().scenario(),
            ScenarioId::ActiveSleepG7
        );
    }
}
