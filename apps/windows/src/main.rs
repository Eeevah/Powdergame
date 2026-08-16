//! Powdergame — Windows executable.
//!
//! winit window → wgpu/DX12 → RTX 5090 → dense GPU world → frames.
//!
//! Default (and `--smoke-frames N`): reference 2048×2048 world, empty
//! clear/present (G0 baseline). Demo fixtures present a staged 128×128 world
//! through the read-only world view:
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
//! Forest scene is unused by the G3/G4/G5 demos.
//!
//! Demos start PAUSED so the untouched initial scene can be inspected:
//!   SPACE  play/pause toggle
//!   N      single simulation tick while paused
//!   R      reset the demo scene (re-staged through the validated edit hook)
//!   ESC    exit
//! Each demo runs at its own fixed observation rate, decoupled from the
//! render rate: Movement/Density = 15 TPS (approved fixtures, unchanged),
//! Thermal = 60 TPS. Bounded smoke runs start PLAYING so they exercise
//! ticks + presentation.
//!
//! G4-B note: Steam now condenses below 40.0, so demo Steam is staged at a
//! stable hot temperature (T = 80.0). G4-C note: Wood/Oil combustion is
//! driven by real thermal conduction from staged hot Stone reservoirs —
//! the demo never writes a Material ID mid-tick.
//!
//! The Simulation runs headless; the Renderer only reads/presents.

mod observatory;
mod renderer;
mod text_renderer;

use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use observatory::ObservatoryCollector;
use powdergame_core::{
    WorldConfig, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_OIL,
    MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD,
};
use powdergame_gpu::{verify_target_hardware, AdapterReport, GpuError, Simulation};

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
}

impl DemoState {
    fn new(base_title: &'static str, tps: u32, start_playing: bool) -> Self {
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
        format!(
            "{} | {state} | {controls} | tick {}",
            self.base_title, self.ticks
        )
    }
}

/// App state. Simulation and Renderer are kept separate: the simulation does
/// not know about the window; the renderer only presents frames.
struct App {
    window: Option<Arc<Window>>,
    simulation: Option<Simulation>,
    renderer: Option<Renderer>,
    observatory_collector: Option<ObservatoryCollector>,
    frames_rendered: u32,
    smoke_frames: Option<u32>,
    demo_mode: DemoMode,
    demo: Option<DemoState>,
}

impl App {
    fn new(smoke_frames: Option<u32>, demo_mode: DemoMode) -> Self {
        Self {
            window: None,
            simulation: None,
            renderer: None,
            observatory_collector: None,
            frames_rendered: 0,
            smoke_frames,
            demo_mode,
            demo: None,
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
            DemoMode::None => "Powdergame — G0 Runtime",
        };
        // The thermal and pressure observatories use a larger world (320×192 / 256×256),
        // so they get a 1600×900 window; the G2/G3 fixtures keep 1280×720.
        let (window_w, window_h) = if self.demo_mode == DemoMode::Thermal
            || self.demo_mode == DemoMode::Pressure
            || self.demo_mode == DemoMode::ParallelIntegrity
            || self.demo_mode == DemoMode::Activity
        {
            (1600.0, 900.0)
        } else {
            (1280.0, 720.0)
        };
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::WindowAttributes::default()
                        .with_title(base_title)
                        .with_inner_size(winit::dpi::LogicalSize::new(window_w, window_h)),
                )
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
                stage_activity_demo(&simulation)?;
                println!("[powdergame] activity demo: 4-panel active/sleep observatory staged");
            }
        }

        let world_view = (self.demo_mode != DemoMode::None).then_some(WorldViewSpec {
            material_buffer: &simulation.world.material_current,
            temperature_buffer: Some(&simulation.world.temperature_current),
            flags_buffer: Some(&simulation.world.flags_current),
            width: simulation.world.config.width,
            height: simulation.world.config.height,
            palette: match self.demo_mode {
                DemoMode::Density => PresentationPalette::Lab,
                DemoMode::Thermal | DemoMode::Pressure => PresentationPalette::ThermalLab,
                DemoMode::ParallelIntegrity => PresentationPalette::Integrity,
                DemoMode::Activity => PresentationPalette::Activity,
                _ => PresentationPalette::Forest,
            },
            chunk_activity_buffer: (self.demo_mode == DemoMode::Activity)
                .then_some(&simulation.world.chunk_activity),
            chunk_size: simulation.world.config.chunk_size,
        });
        let renderer = Renderer::new(
            &simulation.context.instance,
            &simulation.context.adapter,
            &simulation.context.device,
            &simulation.context.queue,
            window.clone(),
            world_view,
        )?;
        println!("[powdergame] surface format: {:?}", renderer.format());
        if self.demo_mode != DemoMode::None {
            // Interactive sessions start PAUSED so the initial scene is fully
            // visible; bounded smoke runs start PLAYING to exercise ticks.
            let start_playing = self.smoke_frames.is_some();
            let demo = DemoState::new(base_title, self.demo_mode.ticks_per_second(), start_playing);
            window.set_title(&demo.title());
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
        self.simulation = Some(simulation);
        self.renderer = Some(renderer);
        self.observatory_collector = observatory_collector;
        Ok(())
    }

    fn toggle_play(&mut self, window: &Window) {
        if let Some(demo) = &mut self.demo {
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
            if demo.playing {
                println!("[powdergame] demo: N ignored while playing (SPACE to pause)");
                return;
            }
            demo.step_pending = true;
            println!("[powdergame] demo: single step requested");
            window.set_title(&demo.title());
            window.request_redraw();
        }
    }

    fn request_fast_forward(&mut self, window: &Window) {
        // G6/G7 observatory demos: cycles 1x -> 4x -> 16x -> 1x.
        // `Simulation::tick` semantics are unchanged — the multiplier just
        // runs more sequential ticks per update opportunity. N always steps
        // exactly one tick.
        if !matches!(
            self.demo_mode,
            DemoMode::ParallelIntegrity | DemoMode::Activity
        ) {
            return;
        }
        if let Some(demo) = &mut self.demo {
            demo.fast = match demo.fast {
                1 => 4,
                4 => 16,
                _ => 1,
            };
            if demo.playing {
                demo.rate_ticks = 0;
                demo.rate_started = Some(Instant::now());
            }
            println!("[powdergame] demo: fast-forward x{}", demo.fast);
            window.set_title(&demo.title());
            window.request_redraw();
        }
    }

    fn request_reset(&mut self, window: &Window) {
        if let Some(demo) = &mut self.demo {
            demo.reset_pending = true;
            demo.playing = false;
            demo.last_tick = None;
            demo.ticks = 0;
            demo.fast = 1;
            demo.rate_ticks = 0;
            demo.rate_started = None;
            if let Some(collector) = &mut self.observatory_collector {
                collector.reset();
            }
            println!("[powdergame] demo: reset requested");
            window.set_title(&demo.title());
            window.request_redraw();
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

/// G7-A activity observatory: 256×256 (4×4 chunks), 2×2 panel layout with
/// Boundary-Block dividers at x 127..128 / y 127..128. Boundary Block has
/// conductivity 0, so the four experiments are thermally isolated — heat
/// cannot cross a zero-conductivity edge, and the corrected detector never
/// reports a THERMAL frontier across it.
///
///   [A] STABLE WATER BULK     — sealed tank (no EMPTY interface → no
///       movement frontier) beside a draining Water column in a sealed
///       shaft: MATTER frontier while falling/settling, fully contained in
///       chunk (1,1) so it can never contaminate panel C below.
///   [B] STABLE STEAM / GAS     — TRUE stable control: sealed Stone shell
///       AND Steam both at T=80 — no initial interface gradient, no EMPTY
///       interface, no staged pressure/reaction, Steam on the stable side
///       of its 40 condense threshold. Gas existence != Activity.
///   [C] STABLE DURATION / WAKE CANDIDATE — Sand source entirely in the
///       upper-right C chunk (cx=1, cy=2); the lower-right C chunk (cx=1,
///       cy=3) is stable first, then the real Sand frontier crosses the
///       y=192 seam through ordinary movement and resets its stable
///       counter. This is a wake-candidate observation — G7-A has no
///       sleeping chunk and no dedicated wake propagation (G7-B).
///   [D] SLOW ACTIVE WORLD      — Wood strip ignited by a hot Stone end
///       (reaction + heat front) + a boiling Water pot (pressure + steam).
///
/// Staging uses only the validated edit hook (material + temperature +
/// flags); everything after the first tick is the production GPU
/// simulation.
fn stage_activity_demo(simulation: &Simulation) -> Result<(), GpuError> {
    let q = &simulation.context.queue;
    let set = |x: i64, y: i64, id: u32| simulation.world.write_material(q, x, y, id);
    let set_t = |x: i64, y: i64, t: f32| simulation.world.write_temperature(q, x, y, t);
    let stone = MATERIAL_STONE;
    let fill = |x0: i64, y0: i64, x1: i64, y1: i64, id: u32| -> Result<(), GpuError> {
        for y in y0..=y1 {
            for x in x0..=x1 {
                set(x, y, id)?;
            }
        }
        Ok(())
    };
    let fill_t = |x0: i64, y0: i64, x1: i64, y1: i64, t: f32| -> Result<(), GpuError> {
        for y in y0..=y1 {
            for x in x0..=x1 {
                set_t(x, y, t)?;
            }
        }
        Ok(())
    };

    // Central cross dividers: MATERIAL_BOUNDARY_BLOCK (K=0), so the four
    // panels are thermally isolated — heat cannot cross a zero-conductivity
    // edge, and the corrected detector never reports THERMAL across it.
    // (G5/G6 fixtures keep their Stone crosses.)
    for y in 1..=254 {
        set(127, y, MATERIAL_BOUNDARY_BLOCK)?;
        set(128, y, MATERIAL_BOUNDARY_BLOCK)?;
    }
    for x in 1..=254 {
        set(x, 127, MATERIAL_BOUNDARY_BLOCK)?;
        set(x, 128, MATERIAL_BOUNDARY_BLOCK)?;
    }

    // [A] STABLE WATER BULK (top-left: x 1..126, y 1..126).
    // Sealed tank: water has no EMPTY interface on any stencil stage → the
    // bulk chunks report no movement frontier (existence != activity).
    fill(30, 40, 91, 105, stone)?; // tank shell
    fill(32, 42, 89, 103, MATERIAL_WATER)?;
    // Draining column in a SEALED stone shaft: it falls and settles in the
    // sealed basin — a genuine MATTER frontier while falling, contained
    // entirely in chunk (1,1) so it can never contaminate panel C below.
    fill(94, 44, 95, 121, stone)?; // shaft left wall
    fill(108, 44, 109, 121, stone)?; // shaft right wall
    fill(94, 119, 109, 121, stone)?; // sealed basin floor
    fill(100, 70, 103, 80, MATERIAL_WATER)?; // column inside the shaft

    // [B] STABLE STEAM / GAS BULK (top-right: x 129..254, y 1..126).
    // TRUE stable control: Stone shell AND Steam both at T=80 — no initial
    // interface gradient, no EMPTY interface, no staged pressure/reaction,
    // Steam on the stable side of its 40 condense threshold. Large Steam
    // existence alone must never be activity.
    fill(140, 40, 231, 92, stone)?; // chamber shell
    fill(143, 43, 228, 88, MATERIAL_STEAM)?;
    fill_t(140, 40, 231, 92, 80.0)?; // shell + steam uniform at 80

    // [C] STABLE DURATION / WAKE CANDIDATE (bottom-left: x 1..126, y 129..254).
    // Sand source sits ENTIRELY in the upper-right C chunk (cx=1, cy=2:
    // x 64..127, y 128..191) at tick 0; the lower-right C chunk (cx=1,
    // cy=3: x 64..127, y 192..255) starts empty except a distant landing
    // floor, accumulates stable ticks first, then the real Sand frontier
    // crosses the y=192 chunk seam through ordinary movement and resets its
    // stable counter. No timer/script — production movement only.
    fill(96, 245, 110, 247, stone)?; // distant landing floor (target chunk)
    fill(100, 150, 106, 165, MATERIAL_SAND)?; // source in upper-right chunk only

    // [D] SLOW ACTIVE WORLD (bottom-right: x 129..254, y 129..254).
    // Wood strip ignited at one end by a hot Stone reservoir.
    fill(140, 174, 149, 179, stone)?;
    fill_t(140, 174, 149, 179, 200.0)?;
    fill(150, 175, 200, 178, MATERIAL_WOOD)?;
    // Boiling Water pot (hot Stone under a lidded cup with a vent):
    // expansion steam rises; confinement pressure forms when the vent is
    // occupied — a genuine PRESSURE frontier source.
    fill(210, 231, 245, 236, stone)?;
    fill_t(210, 231, 245, 236, 200.0)?;
    fill(214, 229, 240, 230, stone)?; // cup floor
    fill(214, 210, 240, 211, stone)?; // lid
    fill(226, 210, 229, 211, MATERIAL_EMPTY)?; // vent
    fill(214, 212, 215, 228, stone)?; // cup wall L
    fill(239, 212, 240, 228, stone)?; // cup wall R
    fill(217, 216, 238, 228, MATERIAL_WATER)?;

    Ok(())
}

/// Resets the demo world to its pristine boundary-ring state and re-stages
/// the active demo scene, using only the validated edit hook.
fn reset_demo_world(simulation: &Simulation, mode: DemoMode) -> Result<(), GpuError> {
    let q = &simulation.context.queue;
    let w = i64::from(simulation.world.config.width);
    let h = i64::from(simulation.world.config.height);
    for y in 0..h {
        for x in 0..w {
            simulation.world.write_material(q, x, y, MATERIAL_EMPTY)?;
        }
    }
    for x in 0..w {
        simulation
            .world
            .write_material(q, x, 0, MATERIAL_BOUNDARY_BLOCK)?;
        simulation
            .world
            .write_material(q, x, h - 1, MATERIAL_BOUNDARY_BLOCK)?;
    }
    for y in 0..h {
        simulation
            .world
            .write_material(q, 0, y, MATERIAL_BOUNDARY_BLOCK)?;
        simulation
            .world
            .write_material(q, w - 1, y, MATERIAL_BOUNDARY_BLOCK)?;
    }
    match mode {
        DemoMode::Movement => stage_movement_demo(simulation),
        DemoMode::Density => stage_density_demo(simulation),
        DemoMode::Thermal => stage_thermal_demo(simulation),
        DemoMode::Pressure => stage_pressure_demo(simulation),
        DemoMode::ParallelIntegrity => stage_parallel_integrity_demo(simulation),
        DemoMode::Activity => stage_activity_demo(simulation),
        DemoMode::None => Ok(()),
    }
}

/// Advances the demo simulation: pending reset/step first, then the
/// fixed-rate play loop (mode TPS), decoupled from the render rate.
fn step_demo(
    simulation: &mut Simulation,
    demo: &mut DemoState,
    collector: &mut Option<ObservatoryCollector>,
    mode: DemoMode,
) {
    if demo.reset_pending {
        demo.reset_pending = false;
        if let Err(e) = reset_demo_world(simulation, mode) {
            eprintln!("[powdergame] demo reset error: {e}");
        } else {
            println!("[powdergame] demo: scene reset to initial state");
        }
        if let Some(col) = collector {
            col.reset();
        }
        demo.ticks = 0;
        demo.last_tick = None;
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
                        col.latch_first_tick_if_g6(simulation, demo.ticks, demo.fast);
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
                            col.latch_first_tick_if_g6(simulation, demo.ticks, demo.fast);
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
        col.update(simulation, demo.ticks, demo.fast);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        if let Err(e) = self.init(event_loop) {
            eprintln!("[powdergame] FATAL: {e}");
            event_loop.exit();
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
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed || event.repeat {
                    return;
                }
                match event.logical_key {
                    Key::Named(NamedKey::Space) => self.toggle_play(&window),
                    Key::Named(NamedKey::Escape) => {
                        println!("[powdergame] ESC pressed; exiting");
                        event_loop.exit();
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
                            self.demo_mode,
                        );
                    } else if let Err(e) = simulation.tick() {
                        eprintln!("[powdergame] tick error: {e}");
                    }
                }
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

/// Parses `--smoke-frames N` (or `POWDERGAME_SMOKE_FRAMES`) for bounded runs.
fn parse_smoke_frames() -> Option<u32> {
    let mut frames = std::env::var("POWDERGAME_SMOKE_FRAMES")
        .ok()
        .and_then(|v| v.parse().ok());
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--smoke-frames" {
            frames = args.next().and_then(|v| v.parse().ok());
        }
    }
    frames
}

/// Parses the demo mode: `--movement-demo` / `--density-demo` /
/// `--thermal-demo` / `--pressure-demo` (or their `POWDERGAME_*_DEMO=1` env equivalents).
fn parse_demo_mode() -> DemoMode {
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--movement-demo" => return DemoMode::Movement,
            "--density-demo" => return DemoMode::Density,
            "--thermal-demo" => return DemoMode::Thermal,
            "--pressure-demo" => return DemoMode::Pressure,
            "--parallel-integrity-demo" => return DemoMode::ParallelIntegrity,
            "--activity-demo" => return DemoMode::Activity,
            _ => {}
        }
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
    DemoMode::None
}

fn main() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

    let smoke_frames = parse_smoke_frames();
    if let Some(n) = smoke_frames {
        println!("[powdergame] smoke run: will exit after {n} frames");
    }
    let demo_mode = parse_demo_mode();
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
        DemoMode::None => {}
    }

    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = App::new(smoke_frames, demo_mode);
    event_loop.run_app(&mut app).expect("event loop failed");
    println!("[powdergame] exited cleanly");
}

#[cfg(test)]
mod tests {
    use super::stage_activity_demo;
    use powdergame_core::{WorldConfig, ACTIVITY_MATTER};
    use powdergame_gpu::Simulation;

    /// G7-A actual-fixture long-run correctness validation (RTX 5090 / DX12).
    /// Uses the real `stage_activity_demo` geometry with 3000+ production
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
        stage_activity_demo(&sim).expect("stage activity demo");

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
}
