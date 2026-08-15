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
//!                       STEAM+SMOKE gas ordering). Forest scene is unused.
//!
//! Demos start PAUSED so the untouched initial scene can be inspected:
//!   SPACE  play/pause toggle
//!   N      single simulation tick while paused
//!   R      reset the demo scene (re-staged through the validated edit hook)
//!   ESC    exit
//! The demo simulation runs at a fixed observation rate (15 TPS), decoupled
//! from the render rate. Bounded smoke runs start PLAYING so they exercise
//! ticks + presentation.
//!
//! G4-B note: Steam now condenses below 40.0, so demo Steam is staged at a
//! stable hot temperature (T = 80.0).
//!
//! The Simulation runs headless; the Renderer only reads/presents.

mod renderer;

use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use powdergame_core::{
    WorldConfig, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_OIL, MATERIAL_SAND,
    MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER,
};
use powdergame_gpu::{verify_target_hardware, AdapterReport, GpuError, Simulation};

use renderer::{PresentationPalette, Renderer, WorldViewSpec};

/// Demo observation rate: independent of the render FPS.
const DEMO_TICKS_PER_SECOND: u32 = 15;
const DEMO_TICK_INTERVAL: Duration =
    Duration::from_nanos(1_000_000_000 / (DEMO_TICKS_PER_SECOND as u64));

/// Stable hot temperature for staged Steam (above the 40.0 condensation
/// threshold, G4-B).
const STEAM_STABLE_T: f32 = 80.0;

const MOVEMENT_DEMO_TITLE: &str = "Powdergame G2 Demo | SAND | WATER | OIL | STEAM | SMOKE";
const DENSITY_DEMO_TITLE: &str =
    "Powdergame G3 Density Demo | SAND+WATER | WATER+OIL | STEAM+SMOKE";

/// Which demo fixture (if any) the app presents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoMode {
    None,
    Movement,
    Density,
}

/// Demo runtime state (demo modes only).
struct DemoState {
    base_title: &'static str,
    playing: bool,
    ticks: u64,
    last_tick: Option<Instant>,
    step_pending: bool,
    reset_pending: bool,
}

impl DemoState {
    fn new(base_title: &'static str, start_playing: bool) -> Self {
        Self {
            base_title,
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
            format!("[PLAY {DEMO_TICKS_PER_SECOND} TPS] SPACE Pause")
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
            DemoMode::None => "Powdergame — G0 Runtime",
        };
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::WindowAttributes::default()
                        .with_title(base_title)
                        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0)),
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

        // Headless simulation. Demo modes use a 128×128 world staged with a
        // fixture scene through the validated edit hook; production stays
        // GPU-authoritative.
        let config = if self.demo_mode == DemoMode::None {
            WorldConfig::reference()
        } else {
            WorldConfig::new(128, 128, 64).expect("demo world config")
        };
        let mut simulation = Simulation::with_context(context, config)?;
        println!("[powdergame] === world allocation ===");
        println!("[powdergame] {}", simulation.world.allocation);
        println!("[powdergame] allocation: success");

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
        }

        let world_view = (self.demo_mode != DemoMode::None).then_some(WorldViewSpec {
            material_buffer: &simulation.world.material_current,
            width: simulation.world.config.width,
            height: simulation.world.config.height,
            palette: match self.demo_mode {
                DemoMode::Density => PresentationPalette::Lab,
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
            self.demo = Some(DemoState::new(base_title, start_playing));
            window.set_title(&self.demo.as_ref().unwrap().title());
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
        Ok(())
    }

    fn toggle_play(&mut self, window: &Window) {
        if let Some(demo) = &mut self.demo {
            demo.playing = !demo.playing;
            if demo.playing {
                demo.last_tick = None; // restart the 15 TPS clock on resume
                println!("[powdergame] demo: PLAY ({} TPS)", DEMO_TICKS_PER_SECOND);
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
            println!("[powdergame] demo: reset requested");
            window.set_title(&demo.title());
            window.request_redraw();
        }
    }
}

/// Stages the G2 stylized-forest movement scene on the 128×128 demo world.
///
/// Zones run left→right in the same order as the window title
/// (SAND | WATER | OIL | STEAM | SMOKE), separated by stone tree-trunk
/// dividers, plus a small separate Void-exit funnel at the bottom right.
fn stage_movement_demo(simulation: &Simulation) -> Result<(), GpuError> {
    let q = &simulation.context.queue;
    let set = |x: i64, y: i64, id: u32| simulation.world.write_material(q, x, y, id);
    let set_t = |x: i64, y: i64, t: f32| simulation.world.write_temperature(q, x, y, t);
    let stone = MATERIAL_STONE;

    // Tree dividers (zone separators): trunk column + stylized canopy.
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

    // ── Zone 1 — SAND: forest hill with trees; sand pours from the sky. ──
    for x in 6..=18 {
        set(x, 84, stone)?;
        set(x, 85, stone)?; // upper ledge
    }
    for x in 12..=14 {
        set(x, 82, stone)?; // bump on the ledge
    }
    for x in 3..=20 {
        set(x, 104, stone)?;
        set(x, 105, stone)?;
        set(x, 106, stone)?; // lower ground
    }
    // Trees standing on the ledge.
    for y in 70..=83 {
        set(9, y, stone)?;
        set(16, y, stone)?;
    }
    for y in 68..=69 {
        for x in 8..=10 {
            set(x, y, stone)?;
        }
        for x in 15..=17 {
            set(x, y, stone)?;
        }
    }
    // Sand pour: falls onto trees/ledge, spills down to the ground below.
    for y in 6..=8 {
        for x in 9..=15 {
            set(x, y, MATERIAL_SAND)?;
        }
    }

    // ── Zone 2 — WATER: cliff, mid ledge, basin; stream + two-step fall. ──
    for x in 26..=46 {
        for y in 30..=32 {
            set(x, y, stone)?; // high cliff
        }
    }
    for x in 24..=28 {
        for y in 52..=54 {
            set(x, y, stone)?; // mid ledge: left waterfall (falls at x=25) lands here
        }
    }
    for x in 26..=46 {
        for y in 88..=90 {
            set(x, y, stone)?; // basin floor
        }
    }
    // Trees rising from the basin floor.
    for y in 74..=87 {
        set(29, y, stone)?;
        set(43, y, stone)?;
    }
    for y in 72..=73 {
        for x in 28..=30 {
            set(x, y, stone)?;
        }
        for x in 42..=44 {
            set(x, y, stone)?;
        }
    }
    // Water source on the cliff: pours off both edges, streams down.
    for y in 20..=22 {
        for x in 32..=42 {
            set(x, y, MATERIAL_WATER)?;
        }
    }

    // ── Zone 3 — OIL: bowl; oil rains in and pools. ──
    for x in 52..=66 {
        for y in 60..=62 {
            set(x, y, stone)?; // bowl floor
        }
    }
    for y in 48..=60 {
        for x in 52..=54 {
            set(x, y, stone)?; // left wall
        }
        for x in 64..=66 {
            set(x, y, stone)?; // right wall
        }
    }
    for y in 24..=26 {
        for x in 56..=62 {
            set(x, y, MATERIAL_OIL)?;
        }
    }

    // ── Zone 4 — STEAM: geyser basin under a slab, rising through canopy. ──
    for x in 76..=88 {
        for y in 112..=114 {
            set(x, y, stone)?; // basin floor
        }
    }
    for y in 100..=112 {
        for x in 76..=78 {
            set(x, y, stone)?; // left wall
        }
        for x in 86..=88 {
            set(x, y, stone)?; // right wall
        }
    }
    for x in 78..=86 {
        for y in 96..=98 {
            set(x, y, stone)?; // slab above the basin: steam flows around it
        }
    }
    for y in 72..=74 {
        for x in 74..=77 {
            set(x, y, stone)?; // canopy left
        }
        for x in 87..=90 {
            set(x, y, stone)?; // canopy right
        }
    }
    for y in 106..=110 {
        for x in 80..=84 {
            set(x, y, MATERIAL_STEAM)?;
            set_t(x, y, STEAM_STABLE_T)?; // G4-B: hot Steam stays Steam
        }
    }

    // ── Zone 5 — SMOKE: pit with a canopy gap; smoke rises through. ──
    for x in 100..=112 {
        for y in 118..=120 {
            set(x, y, stone)?; // pit floor
        }
    }
    for y in 88..=90 {
        for x in 98..=103 {
            set(x, y, stone)?; // canopy left
        }
        for x in 109..=114 {
            set(x, y, stone)?; // canopy right
        }
    }
    for y in 110..=114 {
        for x in 104..=108 {
            set(x, y, MATERIAL_SMOKE)?;
        }
    }

    // ── Void zone (bottom right): funnel into an open boundary hole. ──
    for x in 120..=124 {
        for y in 118..=120 {
            set(x, y, stone)?; // small platform
        }
    }
    for y in 124..=126 {
        set(121, y, stone)?; // funnel walls guiding sand into the hole
        set(123, y, stone)?;
    }
    set(122, 127, MATERIAL_EMPTY)?; // open the boundary ring → Void exit
    for y in 121..=123 {
        set(122, y, MATERIAL_SAND)?; // sand stack drains into the hole
    }

    Ok(())
}

/// Stages the G3 laboratory density-validation scene on the 128×128 world.
///
/// Three large tanks, left→right matching the window title. No forest
/// dividers, trees, ledges, or other G2 ornaments. Walls are Stone only.
///   1. SAND + WATER — large sand block sitting on a deep water pool.
///   2. WATER + OIL  — inverted layers (water above, oil below).
///   3. STEAM + SMOKE — sealed chamber, inverted (smoke above, steam below).
fn stage_density_demo(simulation: &Simulation) -> Result<(), GpuError> {
    let q = &simulation.context.queue;
    let set = |x: i64, y: i64, id: u32| simulation.world.write_material(q, x, y, id);
    let set_t = |x: i64, y: i64, t: f32| simulation.world.write_temperature(q, x, y, t);
    let stone = MATERIAL_STONE;

    // Full-height tanks: left / right / bottom walls, 2 cells thick.
    // Tank 3 is sealed (top wall too) so gas cannot leave.
    let tanks = [
        (4i64, 39i64, false),  // SAND + WATER, open top
        (46i64, 81i64, false), // WATER + OIL, open top
        (88i64, 123i64, true), // STEAM + SMOKE, sealed
    ];
    let wall_top = 4i64;
    let wall_bot = 125i64;
    for &(x0, x1, sealed) in &tanks {
        for y in wall_top..=wall_bot {
            set(x0, y, stone)?;
            set(x0 + 1, y, stone)?;
            set(x1 - 1, y, stone)?;
            set(x1, y, stone)?;
        }
        for x in x0..=x1 {
            set(x, wall_bot - 1, stone)?;
            set(x, wall_bot, stone)?;
            if sealed {
                set(x, wall_top, stone)?;
                set(x, wall_top + 1, stone)?;
            }
        }
    }

    // ── Tank 1 — SAND + WATER ──
    // Water: 20 rows × 24 cols, filling the lower half of the inner tank.
    // Sand:  10 rows × 16 cols, a large block sitting on the water.
    // Paused frame must read as Sand / Water, not a thin pour.
    for y in 104..=123 {
        for x in 10..=33 {
            set(x, y, MATERIAL_WATER)?;
        }
    }
    for y in 94..=103 {
        for x in 14..=29 {
            set(x, y, MATERIAL_SAND)?;
        }
    }

    // ── Tank 2 — WATER + OIL (deliberately inverted) ──
    // Oil below, water above; each layer 12 rows × 28 cols.
    for y in 112..=123 {
        for x in 50..=77 {
            set(x, y, MATERIAL_OIL)?;
        }
    }
    for y in 100..=111 {
        for x in 50..=77 {
            set(x, y, MATERIAL_WATER)?;
        }
    }

    // ── Tank 3 — STEAM + SMOKE (sealed, inverted) ──
    // Smoke above, steam below; each layer 12 rows × 28 cols, mid-chamber
    // so the swap has empty space above and below.
    for y in 52..=63 {
        for x in 92..=119 {
            set(x, y, MATERIAL_SMOKE)?;
        }
    }
    for y in 64..=75 {
        for x in 92..=119 {
            set(x, y, MATERIAL_STEAM)?;
            set_t(x, y, STEAM_STABLE_T)?; // G4-B: hot Steam stays Steam
        }
    }

    Ok(())
}

/// Resets the demo world to its pristine boundary-ring state and re-stages
/// the active demo scene, using only the validated edit hook (never touching
/// the simulation internals). Current and Next stay consistent throughout.
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
        DemoMode::None => Ok(()),
    }
}

/// Advances the demo simulation: pending reset/step first, then the
/// fixed-rate play loop (15 TPS), decoupled from the render rate.
fn step_demo(simulation: &mut Simulation, demo: &mut DemoState, mode: DemoMode) {
    if demo.reset_pending {
        demo.reset_pending = false;
        if let Err(e) = reset_demo_world(simulation, mode) {
            eprintln!("[powdergame] demo reset error: {e}");
        } else {
            println!("[powdergame] demo: scene reset to initial state");
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
        let now = Instant::now();
        let prev = demo.last_tick.unwrap_or(now);
        let mut acc = now.duration_since(prev);
        while acc >= DEMO_TICK_INTERVAL {
            match simulation.tick() {
                Ok(()) => demo.ticks += 1,
                Err(e) => eprintln!("[powdergame] demo tick error: {e}"),
            }
            acc -= DEMO_TICK_INTERVAL;
        }
        // Keep the remainder so the rate does not drift over time.
        demo.last_tick = Some(now - acc);
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
                        step_demo(simulation, demo, self.demo_mode);
                    } else if let Err(e) = simulation.tick() {
                        eprintln!("[powdergame] tick error: {e}");
                    }
                }
                if let Some(renderer) = &mut self.renderer {
                    if let Err(e) = renderer.render() {
                        eprintln!("[powdergame] render error: {e}");
                        // Device-lost style errors abort the app.
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

/// Parses the demo mode: `--movement-demo` / `--density-demo` (or their
/// `POWDERGAME_*_DEMO=1` env equivalents).
fn parse_demo_mode() -> DemoMode {
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--movement-demo" => return DemoMode::Movement,
            "--density-demo" => return DemoMode::Density,
            _ => {}
        }
    }
    if std::env::var("POWDERGAME_MOVEMENT_DEMO").as_deref() == Ok("1") {
        return DemoMode::Movement;
    }
    if std::env::var("POWDERGAME_DENSITY_DEMO").as_deref() == Ok("1") {
        return DemoMode::Density;
    }
    DemoMode::None
}

fn main() {
    // Respect RUST_LOG (e.g. `RUST_LOG=warn` silences wgpu's Naga spam);
    // default to info when unset.
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
        DemoMode::None => {}
    }

    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = App::new(smoke_frames, demo_mode);
    event_loop.run_app(&mut app).expect("event loop failed");
    println!("[powdergame] exited cleanly");
}
