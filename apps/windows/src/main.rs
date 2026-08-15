//! Powdergame — Windows executable.
//!
//! winit window → wgpu/DX12 → RTX 5090 → dense GPU world → frames.
//!
//! Default (and `--smoke-frames N`): reference 2048×2048 world, empty
//! clear/present (G0 baseline). `--movement-demo`: small 256×256 world with
//! a staged local-movement scene presented through the read-only world view
//! (G2 user validation fixture — not a gameplay UI).
//!
//! The Simulation runs headless; the Renderer only reads/presents.

mod renderer;

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use powdergame_core::{
    WorldConfig, MATERIAL_EMPTY, MATERIAL_OIL, MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STEAM,
    MATERIAL_STONE, MATERIAL_WATER,
};
use powdergame_gpu::{verify_target_hardware, AdapterReport, GpuError, Simulation};

use renderer::{Renderer, WorldViewSpec};

/// App state. Simulation and Renderer are kept separate: the simulation does
/// not know about the window; the renderer only presents frames.
struct App {
    window: Option<Arc<Window>>,
    simulation: Option<Simulation>,
    renderer: Option<Renderer>,
    frames_rendered: u32,
    smoke_frames: Option<u32>,
    movement_demo: bool,
}

impl App {
    fn new(smoke_frames: Option<u32>, movement_demo: bool) -> Self {
        Self {
            window: None,
            simulation: None,
            renderer: None,
            frames_rendered: 0,
            smoke_frames,
            movement_demo,
        }
    }

    fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<(), GpuError> {
        let title = if self.movement_demo {
            "Powdergame — Local Movement Demo"
        } else {
            "Powdergame — G0 Runtime"
        };
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::WindowAttributes::default()
                        .with_title(title)
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

        // Headless simulation. Demo mode stages a small movement scene once
        // through the validated edit hook; production stays GPU-authoritative.
        let config = if self.movement_demo {
            WorldConfig::new(256, 256, 64).expect("demo world config")
        } else {
            WorldConfig::reference()
        };
        let mut simulation = Simulation::with_context(context, config)?;
        println!("[powdergame] === world allocation ===");
        println!("[powdergame] {}", simulation.world.allocation);
        println!("[powdergame] allocation: success");

        if self.movement_demo {
            stage_movement_demo(&simulation)?;
            println!("[powdergame] movement demo: scene staged (one-time edit hook)");
        }

        simulation.tick()?;
        println!(
            "[powdergame] tick ok (headless, no window); marker={}",
            simulation.read_marker()?
        );

        let world_view = self.movement_demo.then_some(WorldViewSpec {
            material_buffer: &simulation.world.material_current,
            width: simulation.world.config.width,
            height: simulation.world.config.height,
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
        if self.movement_demo {
            println!("[powdergame] window + world view ready; presenting movement demo");
        } else {
            println!("[powdergame] window + renderer ready; presenting frames");
        }

        self.window = Some(window);
        self.simulation = Some(simulation);
        self.renderer = Some(renderer);
        Ok(())
    }
}

/// Stages a one-time local-movement scene on the 256×256 demo world:
/// sand fall, water over a stone obstacle, oil pool, steam/smoke rise, and
/// an open boundary with sand exiting into Void.
fn stage_movement_demo(simulation: &Simulation) -> Result<(), GpuError> {
    let q = &simulation.context.queue;
    let set = |x: i64, y: i64, id: u32| simulation.world.write_material(q, x, y, id);

    // Sand column (falls and piles).
    for x in 20..=26 {
        set(x, 12, MATERIAL_SAND)?;
    }
    // Water over a stone obstacle (shelf + bump; water pools and flows off).
    for x in 60..=80 {
        set(x, 44, MATERIAL_STONE)?;
    }
    for x in 66..=72 {
        set(x, 40, MATERIAL_STONE)?;
    }
    for x in 64..=76 {
        set(x, 22, MATERIAL_WATER)?;
    }
    // Oil pool on a stone shelf.
    for x in 100..=112 {
        set(x, 44, MATERIAL_STONE)?;
    }
    for x in 104..=108 {
        set(x, 22, MATERIAL_OIL)?;
    }
    // Steam and smoke rise from the lower area.
    for x in 140..=148 {
        set(x, 230, MATERIAL_STEAM)?;
    }
    for x in 170..=178 {
        set(x, 230, MATERIAL_SMOKE)?;
    }
    // Open boundary: erase a bottom ring block and put sand above it so it
    // falls out of the world into Void (G2 Void movement demo).
    set(128, 255, MATERIAL_EMPTY)?;
    set(127, 254, MATERIAL_SAND)?;
    set(128, 254, MATERIAL_SAND)?;
    set(129, 254, MATERIAL_SAND)?;
    Ok(())
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
        let Some(window) = &self.window else {
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
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                // Simulation ticks independently of rendering; both run here.
                if let Some(simulation) = &mut self.simulation {
                    if let Err(e) = simulation.tick() {
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

/// Parses `--movement-demo` (or `POWDERGAME_MOVEMENT_DEMO=1`).
fn parse_movement_demo() -> bool {
    if std::env::var("POWDERGAME_MOVEMENT_DEMO").as_deref() == Ok("1") {
        return true;
    }
    std::env::args().skip(1).any(|arg| arg == "--movement-demo")
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
    let movement_demo = parse_movement_demo();
    if movement_demo {
        println!("[powdergame] movement demo: 256×256 staged scene + read-only world view");
    }

    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = App::new(smoke_frames, movement_demo);
    event_loop.run_app(&mut app).expect("event loop failed");
    println!("[powdergame] exited cleanly");
}
