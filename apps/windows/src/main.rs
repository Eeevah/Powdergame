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

const MOVEMENT_DEMO_TITLE: &str = "Powdergame G2 Demo | SAND | WATER | OIL | STEAM | SMOKE";
const DENSITY_DEMO_TITLE: &str =
    "Powdergame G3 Density Demo | SAND+WATER | WATER+OIL | STEAM+SMOKE";
const THERMAL_DEMO_TITLE: &str =
    "Powdergame G4 Thermal Observatory | 4 Large Panels + Live Metrics";
const PRESSURE_DEMO_TITLE: &str =
    "Powdergame G5 Pressure Chain | WOOD RELIEF vs STONE SEALED | Heat → Steam → Pressure → Rupture → Vent";

/// Which demo fixture (if any) the app presents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoMode {
    None,
    Movement,
    Density,
    Thermal,
    Pressure,
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
        }
    }

    /// Human-readable state for the window title.
    fn title(&self) -> String {
        let state = if self.playing {
            format!("[PLAY {} TPS] SPACE Pause", self.tps)
        } else {
            "[PAUSED] SPACE Play | N Step | R Reset".to_string()
        };
        format!("{} | {state} | tick {}", self.base_title, self.ticks)
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
            DemoMode::None => "Powdergame — G0 Runtime",
        };
        // The thermal observatory uses a larger 320×192 world, so it gets a
        // 1600×900 window; the G2/G3 fixtures keep 1280×720.
        let (window_w, window_h) = if self.demo_mode == DemoMode::Thermal {
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
        // for the G4 thermal observatory); production stays
        // GPU-authoritative.
        let config = if self.demo_mode == DemoMode::None {
            WorldConfig::reference()
        } else {
            let (w, h) = match self.demo_mode {
                DemoMode::Thermal => (320, 192),
                DemoMode::Pressure => (128, 128),
                _ => (128, 128),
            };
            WorldConfig::new(w, h, 64).expect("demo world config")
        };
        let mut simulation = Simulation::with_context(context, config)?;
        println!("[powdergame] === world allocation ===");
        println!("[powdergame] {}", simulation.world.allocation);
        println!("[powdergame] allocation: success");

        let observatory_collector = if self.demo_mode == DemoMode::Thermal {
            Some(ObservatoryCollector::new(&simulation))
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
                println!("[powdergame] pressure demo: twin boilers staged (Wood relief vs Stone control)");
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
                _ => PresentationPalette::Forest,
            },
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
                println!("[powdergame] demo: PLAY ({} TPS)", demo.tps);
            } else {
                println!("[powdergame] demo: PAUSED");
            }
            window.set_title(&demo.title());
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

    fn request_reset(&mut self, window: &Window) {
        if let Some(demo) = &mut self.demo {
            demo.reset_pending = true;
            demo.playing = false;
            demo.last_tick = None;
            demo.ticks = 0;
            if let Some(collector) = &mut self.observatory_collector {
                collector.reset();
            }
            println!("[powdergame] demo: reset requested");
            window.set_title(&demo.title());
            window.request_redraw();
        }
    }
}

/// Stages the G5 twin-boiler user-validation scene on the 128×128 demo world.
///
/// This fixture does not inject Pressure and does not open any vent. Both
/// boilers start with the same dense Water charge at T=58, just below the
/// Water→Steam threshold. A real hot-Stone floor conducts heat into the
/// Water. The left boiler has a one-cell Wood relief plug; the right uses
/// Stone at the corresponding location as an unbreakable control.
///
/// Expected emergent chain on the left:
/// thermal conduction → Water boils → yield=2 expansion is blocked by dense
/// Matter → confinement Pressure accumulates/propagates → Wood threshold 80
/// is exceeded → Wood self-writes EMPTY → ordinary GAS movement vents Steam.
/// The right-hand Stone control should remain sealed under the same rules.
fn stage_pressure_demo(simulation: &Simulation) -> Result<(), GpuError> {
    let q = &simulation.context.queue;
    let set = |x: i64, y: i64, id: u32| simulation.world.write_material(q, x, y, id);
    let set_t = |x: i64, y: i64, t: f32| simulation.world.write_temperature(q, x, y, t);

    // Central divider / visual baseline.
    for y in 8..=119 {
        set(63, y, MATERIAL_STONE)?;
    }

    // Build one boiler. Geometry is identical except for the center roof plug.
    let build_boiler = |x0: i64, x1: i64, plug_material: u32| -> Result<(), GpuError> {
        let roof_y = 44i64;
        let bottom_y = 108i64;
        let plug_l = (x0 + x1) / 2 - 4;
        let plug_r = (x0 + x1) / 2 + 4;

        // Side walls and base shell.
        for y in roof_y..=bottom_y {
            set(x0, y, MATERIAL_STONE)?;
            set(x1, y, MATERIAL_STONE)?;
        }
        for x in x0..=x1 {
            set(x, bottom_y, MATERIAL_STONE)?;
            set_t(x, bottom_y, 150.0)?;
        }

        // One-cell roof. Only the 9-cell center plug differs between boilers.
        for x in (x0 + 1)..x1 {
            let mat = if x >= plug_l && x <= plug_r {
                plug_material
            } else {
                MATERIAL_STONE
            };
            set(x, roof_y, mat)?;
            set_t(x, roof_y, 20.0)?;
        }

        // Dense water charge. No EMPTY neighbor is available inside the shell,
        // so boiling yield requests must either win a newly opened plug or
        // become confinement Pressure.
        for y in (roof_y + 1)..bottom_y {
            for x in (x0 + 1)..x1 {
                set(x, y, MATERIAL_WATER)?;
                set_t(x, y, 58.0)?;
            }
        }

        // Chimney rails above the plug make the vent plume easy to read while
        // leaving the center fully EMPTY. They are presentation geometry only.
        for y in 8..roof_y {
            set(plug_l - 2, y, MATERIAL_STONE)?;
            set(plug_r + 2, y, MATERIAL_STONE)?;
        }
        Ok(())
    };

    // LEFT: weak Wood relief plug. RIGHT: Stone control.
    build_boiler(8, 57, MATERIAL_WOOD)?;
    build_boiler(70, 119, MATERIAL_STONE)?;

    // Two small pedestal marks distinguish the chambers even without text.
    for x in 24..=41 {
        set(x, 116, MATERIAL_WOOD)?;
    }
    for x in 86..=103 {
        set(x, 116, MATERIAL_STONE)?;
    }

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
            match simulation.tick() {
                Ok(()) => {
                    demo.ticks += 1;
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
            match simulation.tick() {
                Ok(()) => demo.ticks += 1,
                Err(e) => eprintln!("[powdergame] demo tick error: {e}"),
            }
            acc -= interval;
        }
        demo.last_tick = Some(now - acc);
    }
    if let Some(col) = collector {
        col.update(simulation, demo.ticks);
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
                    let thermal_hud = if self.demo_mode == DemoMode::Thermal {
                        self.observatory_collector.as_ref().map(|c| {
                            (
                                c.metrics(),
                                self.demo.as_ref().map(|d| d.ticks).unwrap_or(0),
                            )
                        })
                    } else {
                        None
                    };
                    if let Err(e) = renderer.render(thermal_hud) {
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
        DemoMode::None => {}
    }

    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = App::new(smoke_frames, demo_mode);
    event_loop.run_app(&mut app).expect("event loop failed");
    println!("[powdergame] exited cleanly");
}
