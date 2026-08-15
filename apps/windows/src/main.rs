//! Powdergame G0 — Windows executable.
//!
//! winit window → wgpu/DX12 → RTX 5090 → 2048x2048 GPU world → empty frame
//! clear/present. The Simulation runs headless; the Renderer only presents.

mod renderer;

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use powdergame_core::WorldConfig;
use powdergame_gpu::{verify_target_hardware, AdapterReport, GpuError, Simulation};

use renderer::Renderer;

/// App state. Simulation and Renderer are kept separate: the simulation does
/// not know about the window; the renderer only presents frames.
struct App {
    window: Option<Arc<Window>>,
    simulation: Option<Simulation>,
    renderer: Option<Renderer>,
    frames_rendered: u32,
    smoke_frames: Option<u32>,
}

impl App {
    fn new(smoke_frames: Option<u32>) -> Self {
        Self {
            window: None,
            simulation: None,
            renderer: None,
            frames_rendered: 0,
            smoke_frames,
        }
    }

    fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<(), GpuError> {
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::WindowAttributes::default()
                        .with_title("Powdergame — G0 Runtime")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0)),
                )
                .map_err(|e| GpuError::Other(format!("window create failed: {e}")))?,
        );

        // DX12 + high-performance adapter (G0: no fallback).
        let context = pollster::block_on(powdergame_gpu::GpuContext::new())?;

        println!("[powdergame] === G0 GPU context ===");
        println!(
            "[powdergame] {}",
            AdapterReport::from_info(&context.adapter_info)
        );
        match verify_target_hardware(&context.adapter_info) {
            Ok(()) => println!("[powdergame] hardware check: PASS (RTX 5090 / Dx12)"),
            Err(e) => println!("[powdergame] hardware check: UNEXPECTED — {e}"),
        }

        // Headless simulation over the reference world.
        let mut simulation = Simulation::with_context(context, WorldConfig::reference())?;
        println!("[powdergame] === G0 world allocation ===");
        println!("[powdergame] {}", simulation.world.allocation);
        println!("[powdergame] allocation: success");

        simulation.tick()?;
        println!(
            "[powdergame] tick ok (headless, no window); marker={}",
            simulation.read_marker()?
        );

        let renderer = Renderer::new(
            &simulation.context.instance,
            &simulation.context.adapter,
            &simulation.context.device,
            &simulation.context.queue,
            window.clone(),
        )?;
        println!("[powdergame] surface format: {:?}", renderer.format());
        println!("[powdergame] window + renderer ready; presenting frames");

        self.window = Some(window);
        self.simulation = Some(simulation);
        self.renderer = Some(renderer);
        Ok(())
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

fn main() {
    // Respect RUST_LOG (e.g. `RUST_LOG=warn` silences wgpu's Naga spam);
    // default to info when unset.
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

    let smoke_frames = parse_smoke_frames();
    if let Some(n) = smoke_frames {
        println!("[powdergame] smoke run: will exit after {n} frames");
    }

    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = App::new(smoke_frames);
    event_loop.run_app(&mut app).expect("event loop failed");
    println!("[powdergame] exited cleanly");
}
