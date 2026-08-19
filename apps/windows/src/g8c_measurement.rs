//! G8-C windowed production-coexistence and render-profile worker.
//!
//! This module is an explicit developer/evidence path. It is dispatched
//! before the user-facing app routing, owns no Gallery HUD or Inspector, and
//! never enables timestamp queries for Mode C. Mode D creates a separate
//! timestamp-enabled context and resolves one bounded query batch only after
//! each measured frame window.

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use powdergame_core::WorldConfig;
use powdergame_gpu::{GpuContext, Simulation};
use powdergame_scenarios::{reset_and_stage_scenario, validate_scenario_config, ScenarioId};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use crate::experiment::verify_current_executable_sha256;
use crate::renderer::{
    MeasurementFrameStatus, MeasurementSurfaceFailure, PresentationPalette, RenderTimestampBatch,
    RenderTimestampSample, Renderer, SurfaceInfo, WorldViewSpec,
};

pub const COEXISTENCE_SCHEMA: &str = "powdergame-g8c-coexistence-v1";
pub const RENDER_PROFILE_SCHEMA: &str = "powdergame-g8c-render-profile-v1";
pub const COEXISTENCE_CSV_HEADER: &str = "schema_version,scenario,trial,frame_index,sim_tick,window_elapsed_ms,frame_wall_ms,scheduled_sim_ticks,sim_ticks_executed,catch_up_ticks,missed_simulation_deadlines,presented,surface_error";
pub const RENDER_PROFILE_CSV_HEADER: &str = "schema_version,scenario,trial,frame_index,sim_tick,window_elapsed_ms,frame_wall_ms,scheduled_sim_ticks,sim_ticks_executed,catch_up_ticks,missed_simulation_deadlines,presented,gpu_start_tick,gpu_end_tick,gpu_render_ms,timestamp_period_ns,surface_error";

const PHYSICAL_WIDTH: u32 = 1600;
const PHYSICAL_HEIGHT: u32 = 900;

fn required_physical_size() -> PhysicalSize<u32> {
    PhysicalSize::new(PHYSICAL_WIDTH, PHYSICAL_HEIGHT)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WindowSizeEventClassification {
    CanonicalNoOp,
    StalePayloadIgnored,
    FatalNoncanonicalLiveSize,
}

impl WindowSizeEventClassification {
    fn as_str(self) -> &'static str {
        match self {
            Self::CanonicalNoOp => "canonical_no_op",
            Self::StalePayloadIgnored => "stale_payload_ignored",
            Self::FatalNoncanonicalLiveSize => "fatal_noncanonical_live_size",
        }
    }

    /// G8-C never adapts the fixed measurement surface to a resize event.
    fn should_resize_renderer(self) -> bool {
        false
    }
}

fn classify_window_size_event(
    required_size: PhysicalSize<u32>,
    payload_size: PhysicalSize<u32>,
    live_size: PhysicalSize<u32>,
) -> WindowSizeEventClassification {
    if live_size != required_size {
        WindowSizeEventClassification::FatalNoncanonicalLiveSize
    } else if payload_size == required_size {
        WindowSizeEventClassification::CanonicalNoOp
    } else {
        WindowSizeEventClassification::StalePayloadIgnored
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WindowSizeObservationSource {
    Resized,
    ScaleFactorChanged,
    RedrawGuard,
}

impl WindowSizeObservationSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Resized => "resized",
            Self::ScaleFactorChanged => "scale_factor_changed",
            Self::RedrawGuard => "redraw_guard",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WindowSizeEventDiagnostic {
    source: WindowSizeObservationSource,
    payload_size: PhysicalSize<u32>,
    live_size: PhysicalSize<u32>,
    classification: WindowSizeEventClassification,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct WindowLifecycleDiagnostics {
    initial_live_size: Option<PhysicalSize<u32>>,
    last_live_size: Option<PhysicalSize<u32>>,
    initial_live_size_confirmed: bool,
    canonical_noop_count: u32,
    stale_payload_count: u32,
    fatal_live_resize_count: u32,
    events: Vec<WindowSizeEventDiagnostic>,
}

impl WindowLifecycleDiagnostics {
    fn confirm_initial_live_size(&mut self, live_size: PhysicalSize<u32>) -> Result<(), String> {
        self.initial_live_size = Some(live_size);
        self.last_live_size = Some(live_size);
        if live_size != required_physical_size() {
            return Err(format!(
                "G8-C requires an initial live window size of {PHYSICAL_WIDTH}x{PHYSICAL_HEIGHT}, got {}x{}",
                live_size.width, live_size.height
            ));
        }
        self.initial_live_size_confirmed = true;
        Ok(())
    }

    fn measurement_can_start(&self) -> bool {
        self.initial_live_size_confirmed && self.fatal_live_resize_count == 0
    }

    fn record_event(
        &mut self,
        source: WindowSizeObservationSource,
        payload_size: PhysicalSize<u32>,
        live_size: PhysicalSize<u32>,
    ) -> Result<WindowSizeEventClassification, String> {
        let classification =
            classify_window_size_event(required_physical_size(), payload_size, live_size);
        self.last_live_size = Some(live_size);
        match classification {
            WindowSizeEventClassification::CanonicalNoOp => {
                self.canonical_noop_count =
                    self.canonical_noop_count.checked_add(1).ok_or_else(|| {
                        "G8-C canonical window-size event counter overflow".to_string()
                    })?;
            }
            WindowSizeEventClassification::StalePayloadIgnored => {
                self.stale_payload_count = self
                    .stale_payload_count
                    .checked_add(1)
                    .ok_or_else(|| "G8-C stale window-size payload counter overflow".to_string())?;
            }
            WindowSizeEventClassification::FatalNoncanonicalLiveSize => {
                self.fatal_live_resize_count = self
                    .fatal_live_resize_count
                    .checked_add(1)
                    .ok_or_else(|| "G8-C fatal live-resize counter overflow".to_string())?;
            }
        }
        self.events.push(WindowSizeEventDiagnostic {
            source,
            payload_size,
            live_size,
            classification,
        });
        Ok(classification)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum G8cMode {
    Coexistence,
    RenderProfile,
}

impl G8cMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "coexistence" => Ok(Self::Coexistence),
            "render-profile" => Ok(Self::RenderProfile),
            _ => Err(format!(
                "invalid --mode '{value}'; expected coexistence or render-profile"
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Coexistence => "coexistence",
            Self::RenderProfile => "render-profile",
        }
    }

    fn schema(self) -> &'static str {
        match self {
            Self::Coexistence => COEXISTENCE_SCHEMA,
            Self::RenderProfile => RENDER_PROFILE_SCHEMA,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CoexistenceWindow {
    Seconds(f64),
    Frames(u32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct G8cWorkerConfig {
    pub mode: G8cMode,
    pub scenario: ScenarioId,
    pub width: u32,
    pub height: u32,
    pub chunk_size: u32,
    pub sleep_enabled: bool,
    pub sleep_threshold: u32,
    pub prewarm_secs: f64,
    pub trials: u32,
    pub target_tps: u32,
    pub coexistence_window: Option<CoexistenceWindow>,
    pub profile_frames: Option<u32>,
    pub run_id: String,
    pub binary_sha256: String,
    pub raw_csv: PathBuf,
    pub metadata_json: PathBuf,
}

impl G8cWorkerConfig {
    fn world_config(&self) -> Result<WorldConfig, String> {
        WorldConfig::new(self.width, self.height, self.chunk_size)
            .map_err(|error| format!("invalid G8-C WorldConfig: {error}"))
    }

    fn validate(&self) -> Result<(), String> {
        if !self.scenario.is_official_g8b() {
            return Err(format!(
                "G8-C supports only the five official scenarios, got {}",
                self.scenario.slug()
            ));
        }
        let world = self.world_config()?;
        validate_scenario_config(self.scenario, &world).map_err(|error| error.to_string())?;
        if !self.prewarm_secs.is_finite() || self.prewarm_secs < 0.0 {
            return Err(format!(
                "--prewarm-secs must be finite and non-negative, got {}",
                self.prewarm_secs
            ));
        }
        if self.trials == 0 {
            return Err("--trials must be greater than zero".into());
        }
        if self.target_tps == 0 {
            return Err("--target-tps must be greater than zero".into());
        }
        match (self.mode, self.coexistence_window, self.profile_frames) {
            (G8cMode::Coexistence, Some(CoexistenceWindow::Seconds(value)), None)
                if value.is_finite() && value > 0.0 => {}
            (G8cMode::Coexistence, Some(CoexistenceWindow::Frames(value)), None) if value > 0 => {}
            (G8cMode::RenderProfile, None, Some(value)) if value > 0 => {}
            (G8cMode::Coexistence, _, _) => {
                return Err("coexistence mode requires exactly one positive --measurement-secs or --measurement-frames and rejects --profile-frames".into());
            }
            (G8cMode::RenderProfile, _, _) => {
                return Err("render-profile mode requires positive --profile-frames and rejects coexistence measurement options".into());
            }
        }
        if !is_safe_id(&self.run_id) {
            return Err(format!(
                "--run-id must contain only ASCII letters, digits, '.', '_', or '-', got '{}'",
                self.run_id
            ));
        }
        if self.binary_sha256.len() != 64
            || !self
                .binary_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err("--binary-sha256 must be exactly 64 hexadecimal characters".into());
        }
        if self.raw_csv == self.metadata_json {
            return Err("--raw-csv and --metadata-json must be different paths".into());
        }
        Ok(())
    }
}

fn is_safe_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn value<T: std::str::FromStr>(name: &str, next: Option<String>) -> Result<T, String> {
    let raw = next.ok_or_else(|| format!("missing value for {name}"))?;
    raw.parse()
        .map_err(|_| format!("invalid value for {name}: {raw}"))
}

/// Returns `None` without consuming normal app routing when the explicit
/// `--g8c-worker` marker is absent.
pub fn worker_from_args<I, S>(args: I) -> Result<Option<G8cWorkerConfig>, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|argument| argument.as_ref().to_string())
        .collect::<Vec<_>>();
    if !args.iter().any(|argument| argument == "--g8c-worker") {
        return Ok(None);
    }

    let mut mode = None;
    let mut scenario = None;
    let mut width = None;
    let mut height = None;
    let mut chunk_size = None;
    let mut sleep_enabled = None;
    let mut sleep_threshold = None;
    let mut prewarm_secs = None;
    let mut trials = None;
    let mut target_tps = None;
    let mut measurement_secs = None;
    let mut measurement_frames = None;
    let mut profile_frames = None;
    let mut run_id = None;
    let mut binary_sha256 = None;
    let mut raw_csv = None;
    let mut metadata_json = None;
    let mut worker_seen = false;

    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        macro_rules! unique {
            ($slot:expr, $name:literal, $parsed:expr) => {{
                if $slot.is_some() {
                    return Err(format!("duplicate {} option", $name));
                }
                $slot = Some($parsed);
            }};
        }
        match argument.as_str() {
            "--g8c-worker" => {
                if worker_seen {
                    return Err("duplicate --g8c-worker option".into());
                }
                worker_seen = true;
            }
            "--mode" => unique!(
                mode,
                "--mode",
                G8cMode::parse(
                    &args
                        .next()
                        .ok_or_else(|| "missing value for --mode".to_string())?
                )?
            ),
            "--scenario" => unique!(
                scenario,
                "--scenario",
                args.next()
                    .ok_or_else(|| "missing value for --scenario".to_string())?
                    .parse::<ScenarioId>()
                    .map_err(|error| error.to_string())?
            ),
            "--width" => unique!(width, "--width", value("--width", args.next())?),
            "--height" => unique!(height, "--height", value("--height", args.next())?),
            "--chunk" => unique!(chunk_size, "--chunk", value("--chunk", args.next())?),
            "--sleep" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "missing value for --sleep".to_string())?;
                let parsed = match raw.as_str() {
                    "on" => true,
                    "off" => false,
                    _ => return Err(format!("invalid --sleep '{raw}'; expected on or off")),
                };
                unique!(sleep_enabled, "--sleep", parsed);
            }
            "--threshold" => unique!(
                sleep_threshold,
                "--threshold",
                value("--threshold", args.next())?
            ),
            "--prewarm-secs" => unique!(
                prewarm_secs,
                "--prewarm-secs",
                value("--prewarm-secs", args.next())?
            ),
            "--trials" => unique!(trials, "--trials", value("--trials", args.next())?),
            "--target-tps" => unique!(
                target_tps,
                "--target-tps",
                value("--target-tps", args.next())?
            ),
            "--measurement-secs" => unique!(
                measurement_secs,
                "--measurement-secs",
                value("--measurement-secs", args.next())?
            ),
            "--measurement-frames" => unique!(
                measurement_frames,
                "--measurement-frames",
                value("--measurement-frames", args.next())?
            ),
            "--profile-frames" => unique!(
                profile_frames,
                "--profile-frames",
                value("--profile-frames", args.next())?
            ),
            "--run-id" => unique!(
                run_id,
                "--run-id",
                args.next()
                    .ok_or_else(|| "missing value for --run-id".to_string())?
            ),
            "--binary-sha256" => unique!(
                binary_sha256,
                "--binary-sha256",
                args.next()
                    .ok_or_else(|| "missing value for --binary-sha256".to_string())?
                    .to_ascii_lowercase()
            ),
            "--raw-csv" => unique!(
                raw_csv,
                "--raw-csv",
                PathBuf::from(
                    args.next()
                        .ok_or_else(|| "missing value for --raw-csv".to_string())?
                )
            ),
            "--metadata-json" => unique!(
                metadata_json,
                "--metadata-json",
                PathBuf::from(
                    args.next()
                        .ok_or_else(|| "missing value for --metadata-json".to_string())?
                )
            ),
            _ => return Err(format!("unknown G8-C worker argument '{argument}'")),
        }
    }

    let coexistence_window = match (measurement_secs, measurement_frames) {
        (Some(_), Some(_)) => {
            return Err(
                "--measurement-secs and --measurement-frames are mutually exclusive".into(),
            );
        }
        (Some(seconds), None) => Some(CoexistenceWindow::Seconds(seconds)),
        (None, Some(frames)) => Some(CoexistenceWindow::Frames(frames)),
        (None, None) => None,
    };
    let config = G8cWorkerConfig {
        mode: mode.ok_or_else(|| "missing --mode".to_string())?,
        scenario: scenario.ok_or_else(|| "missing --scenario".to_string())?,
        width: width.ok_or_else(|| "missing --width".to_string())?,
        height: height.ok_or_else(|| "missing --height".to_string())?,
        chunk_size: chunk_size.ok_or_else(|| "missing --chunk".to_string())?,
        sleep_enabled: sleep_enabled.ok_or_else(|| "missing --sleep".to_string())?,
        sleep_threshold: sleep_threshold.ok_or_else(|| "missing --threshold".to_string())?,
        prewarm_secs: prewarm_secs.ok_or_else(|| "missing --prewarm-secs".to_string())?,
        trials: trials.ok_or_else(|| "missing --trials".to_string())?,
        target_tps: target_tps.ok_or_else(|| "missing --target-tps".to_string())?,
        coexistence_window,
        profile_frames,
        run_id: run_id.ok_or_else(|| "missing --run-id".to_string())?,
        binary_sha256: binary_sha256.ok_or_else(|| "missing --binary-sha256".to_string())?,
        raw_csv: raw_csv.ok_or_else(|| "missing --raw-csv".to_string())?,
        metadata_json: metadata_json.ok_or_else(|| "missing --metadata-json".to_string())?,
    };
    config.validate()?;
    Ok(Some(config))
}

struct OutputReservation {
    raw_csv: File,
    metadata_json: File,
}

impl OutputReservation {
    fn create(config: &G8cWorkerConfig) -> Result<Self, String> {
        let raw_csv = reserve_output(&config.raw_csv, "--raw-csv")?;
        let metadata_json = reserve_output(&config.metadata_json, "--metadata-json")?;
        Ok(Self {
            raw_csv,
            metadata_json,
        })
    }
}

fn reserve_output(path: &Path, label: &str) -> Result<File, String> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| format!("{label} requires an explicit parent directory"))?;
    if !parent.is_dir() {
        return Err(format!(
            "{label} parent directory does not exist: {}",
            parent.display()
        ));
    }
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            format!(
                "cannot reserve no-overwrite {label} {}: {error}",
                path.display()
            )
        })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TickPlan {
    scheduled: u64,
    execute: u64,
    catch_up: u64,
    missed_deadlines: u64,
}

fn tick_plan(elapsed: Duration, target_tps: u32, completed: u64) -> TickPlan {
    let scheduled = (elapsed.as_secs_f64() * f64::from(target_tps)).floor() as u64;
    let execute = scheduled.saturating_sub(completed);
    let behind = execute.saturating_sub(1);
    TickPlan {
        scheduled,
        execute,
        catch_up: behind,
        missed_deadlines: behind,
    }
}

fn validate_reset_boundary_count(observed: u32, trials: u32) -> Result<(), String> {
    let expected = trials
        .checked_add(1)
        .ok_or_else(|| format!("G8-C reset-boundary expectation overflows for {trials} trials"))?;
    if observed != expected {
        return Err(format!(
            "G8-C official publication requires exactly {expected} successful reset boundaries (prewarm + {trials} trials), observed {observed}"
        ));
    }
    Ok(())
}

fn validate_surface_contract(surface: &SurfaceInfo) -> Result<(), String> {
    if surface.width != PHYSICAL_WIDTH || surface.height != PHYSICAL_HEIGHT {
        return Err(format!(
            "G8-C requires an actual {PHYSICAL_WIDTH}x{PHYSICAL_HEIGHT} physical surface, got {}x{}",
            surface.width, surface.height
        ));
    }
    if surface.present_mode != wgpu::PresentMode::Fifo {
        return Err(format!(
            "G8-C requires PresentMode::Fifo, got {:?}",
            surface.present_mode
        ));
    }
    Ok(())
}

fn validate_window_lifecycle_for_publication(
    diagnostics: &WindowLifecycleDiagnostics,
) -> Result<(), String> {
    if !diagnostics.initial_live_size_confirmed
        || diagnostics.initial_live_size != Some(required_physical_size())
    {
        return Err("G8-C publication requires a canonical initial live window size".into());
    }
    if diagnostics.last_live_size != Some(required_physical_size()) {
        return Err("G8-C publication requires a canonical final live window size".into());
    }
    if diagnostics.fatal_live_resize_count != 0 {
        return Err(format!(
            "G8-C publication requires zero fatal live resizes, observed {}",
            diagnostics.fatal_live_resize_count
        ));
    }
    let canonical_noop_count = diagnostics
        .events
        .iter()
        .filter(|event| event.classification == WindowSizeEventClassification::CanonicalNoOp)
        .count() as u32;
    let stale_payload_count = diagnostics
        .events
        .iter()
        .filter(|event| event.classification == WindowSizeEventClassification::StalePayloadIgnored)
        .count() as u32;
    let fatal_live_resize_count = diagnostics
        .events
        .iter()
        .filter(|event| {
            event.classification == WindowSizeEventClassification::FatalNoncanonicalLiveSize
        })
        .count() as u32;
    if diagnostics.canonical_noop_count != canonical_noop_count
        || diagnostics.stale_payload_count != stale_payload_count
        || diagnostics.fatal_live_resize_count != fatal_live_resize_count
    {
        return Err("G8-C window lifecycle counters do not match recorded events".into());
    }
    if diagnostics.events.iter().any(|event| {
        event.live_size != required_physical_size()
            || event.classification == WindowSizeEventClassification::FatalNoncanonicalLiveSize
    }) {
        return Err("G8-C publication contains a noncanonical live window-size event".into());
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct FrameRow {
    trial: u32,
    frame_index: u32,
    sim_tick: u64,
    window_elapsed_ms: f64,
    frame_wall_ms: f64,
    scheduled_sim_ticks: u64,
    sim_ticks_executed: u64,
    catch_up_ticks: u64,
    missed_simulation_deadlines: u64,
    presented: bool,
    timestamp: Option<RenderTimestampSample>,
    surface_error: String,
}

#[derive(Clone, Debug)]
struct TrialSummary {
    trial: u32,
    elapsed_ms: f64,
    actual_simulation_ticks: u64,
    actual_simulation_tps: f64,
    presented_frames: u32,
    render_fps: f64,
    frame_p50_ms: f64,
    frame_p95_ms: f64,
    frame_p99_ms: f64,
    missed_simulation_deadlines: u64,
    missed_deadline_ratio: f64,
    catch_up_ticks: u64,
    failed_surface_frames: u32,
    device_errors: u32,
    surface_errors: u32,
    gpu_render_p50_ms: Option<f64>,
    gpu_render_p95_ms: Option<f64>,
    gpu_render_mean_ms: Option<f64>,
}

#[derive(Debug)]
enum RunPhase {
    Uninitialized,
    Prewarm {
        started: Instant,
        completed_ticks: u64,
    },
    Trial {
        trial: u32,
        started: Instant,
        last_frame_end: Instant,
        completed_ticks: u64,
        frame_index: u32,
        row_start: usize,
        device_error_start: usize,
        surface_error_start: usize,
    },
    Finished,
}

struct G8cApp {
    config: G8cWorkerConfig,
    outputs: Option<OutputReservation>,
    window: Option<Arc<Window>>,
    simulation: Option<Simulation>,
    renderer: Option<Renderer>,
    surface: Option<SurfaceInfo>,
    timestamp_period_ns: f32,
    timestamp_batch: Option<RenderTimestampBatch>,
    phase: RunPhase,
    rows: Vec<FrameRow>,
    summaries: Vec<TrialSummary>,
    device_errors: Arc<Mutex<Vec<String>>>,
    surface_errors: Vec<String>,
    successful_reset_boundaries: u32,
    window_lifecycle: WindowLifecycleDiagnostics,
    fatal_error: Option<String>,
    published: bool,
}

impl G8cApp {
    fn new(config: G8cWorkerConfig, outputs: OutputReservation) -> Self {
        Self {
            config,
            outputs: Some(outputs),
            window: None,
            simulation: None,
            renderer: None,
            surface: None,
            timestamp_period_ns: 0.0,
            timestamp_batch: None,
            phase: RunPhase::Uninitialized,
            rows: Vec::new(),
            summaries: Vec::new(),
            device_errors: Arc::new(Mutex::new(Vec::new())),
            surface_errors: Vec::new(),
            successful_reset_boundaries: 0,
            window_lifecycle: WindowLifecycleDiagnostics::default(),
            fatal_error: None,
            published: false,
        }
    }

    fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::WindowAttributes::default()
                        .with_title(format!(
                            "Powdergame G8-C {} | {}",
                            self.config.mode.as_str(),
                            self.config.scenario.name()
                        ))
                        .with_inner_size(winit::dpi::PhysicalSize::new(
                            PHYSICAL_WIDTH,
                            PHYSICAL_HEIGHT,
                        ))
                        .with_visible(true),
                )
                .map_err(|error| format!("G8-C window creation failed: {error}"))?,
        );
        let context = match self.config.mode {
            G8cMode::Coexistence => pollster::block_on(GpuContext::new()),
            G8cMode::RenderProfile => pollster::block_on(GpuContext::with_profiling()),
        }
        .map_err(|error| format!("G8-C GPU context initialization failed: {error}"))?;
        if context.profiling_enabled != (self.config.mode == G8cMode::RenderProfile) {
            return Err("G8-C context profiling feature does not match selected mode".into());
        }
        let device_errors = Arc::clone(&self.device_errors);
        context.device.on_uncaptured_error(Box::new(move |error| {
            let message = error.to_string();
            eprintln!("[powdergame][g8c] uncaptured device error: {message}");
            if let Ok(mut errors) = device_errors.lock() {
                errors.push(message);
            }
        }));
        self.timestamp_period_ns = context.timestamp_period;
        let world = self.config.world_config()?;
        let mut simulation = Simulation::with_context(context, world)
            .map_err(|error| format!("G8-C Simulation creation failed: {error}"))?;
        simulation.set_sleep_enabled(self.config.sleep_enabled);
        simulation.set_sleep_threshold(self.config.sleep_threshold);
        simulation.update_uniforms();

        let world_view = WorldViewSpec {
            material_buffer: &simulation.world.material_current,
            temperature_buffer: Some(&simulation.world.temperature_current),
            pressure_buffer: Some(&simulation.world.pressure_current),
            flags_buffer: Some(&simulation.world.flags_current),
            chunk_activity_buffer: None,
            chunk_size: simulation.world.config.chunk_size,
            width: simulation.world.config.width,
            height: simulation.world.config.height,
            palette: PresentationPalette::Gallery,
        };
        let renderer = Renderer::new(
            &simulation.context.instance,
            &simulation.context.adapter,
            &simulation.context.device,
            &simulation.context.queue,
            window.clone(),
            Some(world_view),
        )
        .map_err(|error| format!("G8-C renderer creation failed: {error}"))?;
        let surface = renderer.surface_info();
        validate_surface_contract(&surface)?;

        reset_stage_and_wait(&mut simulation, self.config.scenario, "prewarm")?;
        // The surface and the live OS window must both be canonical immediately
        // before prewarm or either measurement mode is allowed to start.
        self.window_lifecycle
            .confirm_initial_live_size(window.inner_size())?;
        self.record_successful_reset_boundary()?;
        self.phase = RunPhase::Prewarm {
            started: Instant::now(),
            completed_ticks: 0,
        };
        self.surface = Some(surface);
        self.window = Some(window);
        self.simulation = Some(simulation);
        self.renderer = Some(renderer);
        Ok(())
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        self.require_canonical_live_window_size()?;
        match self.phase {
            RunPhase::Prewarm {
                started,
                completed_ticks,
            } => self.redraw_prewarm(started, completed_ticks)?,
            RunPhase::Trial {
                trial,
                started,
                last_frame_end,
                completed_ticks,
                frame_index,
                row_start,
                device_error_start,
                surface_error_start,
            } => self.redraw_trial(
                event_loop,
                trial,
                started,
                last_frame_end,
                completed_ticks,
                frame_index,
                row_start,
                device_error_start,
                surface_error_start,
            )?,
            RunPhase::Finished | RunPhase::Uninitialized => {}
        }
        Ok(())
    }

    fn require_canonical_live_window_size(&mut self) -> Result<(), String> {
        if !self.window_lifecycle.measurement_can_start() {
            return Err(
                "G8-C measurement cannot run before canonical live window-size confirmation".into(),
            );
        }
        let live_size = self
            .window
            .as_ref()
            .ok_or_else(|| {
                "G8-C live window-size guard requires an initialized window".to_string()
            })?
            .inner_size();
        self.window_lifecycle.last_live_size = Some(live_size);
        if live_size == required_physical_size() {
            return Ok(());
        }

        let classification = self.window_lifecycle.record_event(
            WindowSizeObservationSource::RedrawGuard,
            live_size,
            live_size,
        )?;
        debug_assert_eq!(
            classification,
            WindowSizeEventClassification::FatalNoncanonicalLiveSize
        );
        Err(window_size_failure_message(
            WindowSizeObservationSource::RedrawGuard,
            live_size,
            live_size,
            classification,
        ))
    }

    fn handle_window_size_observation(
        &mut self,
        source: WindowSizeObservationSource,
        payload_size: PhysicalSize<u32>,
        live_size: PhysicalSize<u32>,
    ) -> Result<(), String> {
        let classification = self
            .window_lifecycle
            .record_event(source, payload_size, live_size)?;
        debug_assert!(!classification.should_resize_renderer());
        eprintln!(
            "[powdergame][g8c][window-lifecycle] source={};classification={};required={}x{};payload={}x{};live={}x{};stale_payload_count={};fatal_live_resize_count={}",
            source.as_str(),
            classification.as_str(),
            PHYSICAL_WIDTH,
            PHYSICAL_HEIGHT,
            payload_size.width,
            payload_size.height,
            live_size.width,
            live_size.height,
            self.window_lifecycle.stale_payload_count,
            self.window_lifecycle.fatal_live_resize_count,
        );
        if classification == WindowSizeEventClassification::FatalNoncanonicalLiveSize {
            return Err(window_size_failure_message(
                source,
                payload_size,
                live_size,
                classification,
            ));
        }
        Ok(())
    }

    fn redraw_prewarm(&mut self, started: Instant, completed_ticks: u64) -> Result<(), String> {
        let now = Instant::now();
        let plan = tick_plan(
            now.duration_since(started),
            self.config.target_tps,
            completed_ticks,
        );
        self.execute_ticks(plan.execute, "prewarm")?;
        self.renderer
            .as_mut()
            .expect("initialized renderer")
            .render(None)
            .map_err(|error| format!("G8-C prewarm render failed: {error}"))?;
        let completed_ticks = completed_ticks + plan.execute;
        if started.elapsed().as_secs_f64() >= self.config.prewarm_secs {
            self.start_trial(1)?;
        } else {
            self.phase = RunPhase::Prewarm {
                started,
                completed_ticks,
            };
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn redraw_trial(
        &mut self,
        event_loop: &ActiveEventLoop,
        trial: u32,
        started: Instant,
        last_frame_end: Instant,
        completed_ticks: u64,
        frame_index: u32,
        row_start: usize,
        device_error_start: usize,
        surface_error_start: usize,
    ) -> Result<(), String> {
        let frame_start = Instant::now();
        let plan = tick_plan(
            frame_start.duration_since(started),
            self.config.target_tps,
            completed_ticks,
        );
        self.execute_ticks(plan.execute, &format!("trial {trial}"))?;
        let frame_status = match self.config.mode {
            G8cMode::Coexistence => self
                .renderer
                .as_mut()
                .expect("initialized renderer")
                .render_measurement(),
            G8cMode::RenderProfile => {
                let renderer = self.renderer.as_mut().expect("initialized renderer");
                let batch = self
                    .timestamp_batch
                    .as_mut()
                    .expect("Mode D trial owns timestamp batch");
                renderer
                    .render_timestamped(batch)
                    .map_err(|error| format!("G8-C Mode D trial {trial} render failed: {error}"))?
            }
        };
        let frame_end = Instant::now();
        let completed_ticks = completed_ticks + plan.execute;
        let (presented_frame, surface_error, fatal_surface_error) = match frame_status {
            MeasurementFrameStatus::Presented => (true, String::new(), false),
            MeasurementFrameStatus::Dropped(error) => {
                let message = measurement_surface_error(&error);
                self.surface_errors.push(message.clone());
                if self.config.mode == G8cMode::RenderProfile {
                    return self.stop_failed_profile_trial(
                        event_loop,
                        trial,
                        started,
                        row_start,
                        device_error_start,
                        surface_error_start,
                        error,
                    );
                }
                (false, message, error.fatal)
            }
        };
        self.rows.push(FrameRow {
            trial,
            frame_index,
            sim_tick: self
                .simulation
                .as_ref()
                .expect("initialized simulation")
                .tick_count,
            window_elapsed_ms: frame_end.duration_since(started).as_secs_f64() * 1000.0,
            frame_wall_ms: frame_end.duration_since(last_frame_end).as_secs_f64() * 1000.0,
            scheduled_sim_ticks: plan.scheduled,
            sim_ticks_executed: plan.execute,
            catch_up_ticks: plan.catch_up,
            missed_simulation_deadlines: plan.missed_deadlines,
            presented: presented_frame,
            timestamp: None,
            surface_error,
        });

        let presented = frame_index + 1;
        let complete = fatal_surface_error
            || match self.config.mode {
                G8cMode::Coexistence => match self
                    .config
                    .coexistence_window
                    .expect("validated coexistence window")
                {
                    CoexistenceWindow::Seconds(seconds) => {
                        frame_end.duration_since(started).as_secs_f64() >= seconds
                    }
                    CoexistenceWindow::Frames(frames) => presented >= frames,
                },
                G8cMode::RenderProfile => {
                    presented
                        >= self
                            .config
                            .profile_frames
                            .expect("validated profile frames")
                }
            };
        if complete {
            self.finish_trial(
                trial,
                row_start,
                device_error_start,
                surface_error_start,
                frame_end.duration_since(started),
            )?;
            if fatal_surface_error {
                self.write_failure_diagnostic_outputs()?;
                self.phase = RunPhase::Finished;
                self.published = true;
                self.fatal_error = Some(
                    "G8-C Mode C encountered fatal surface out-of-memory; partial diagnostic metadata was published"
                        .into(),
                );
                event_loop.exit();
            } else if trial < self.config.trials {
                self.start_trial(trial + 1)?;
            } else {
                self.publish()?;
                self.phase = RunPhase::Finished;
                self.published = true;
                let device_errors = self.device_error_count();
                let surface_errors = self.surface_errors.len();
                if device_errors != 0 || surface_errors != 0 {
                    self.fatal_error = Some(format!(
                        "G8-C captured {device_errors} uncaught device errors and {surface_errors} surface errors; metadata was published for diagnosis"
                    ));
                }
                event_loop.exit();
            }
        } else {
            self.phase = RunPhase::Trial {
                trial,
                started,
                last_frame_end: frame_end,
                completed_ticks,
                frame_index: presented,
                row_start,
                device_error_start,
                surface_error_start,
            };
        }
        Ok(())
    }

    fn execute_ticks(&mut self, count: u64, label: &str) -> Result<(), String> {
        let simulation = self.simulation.as_mut().expect("initialized simulation");
        for index in 0..count {
            simulation.tick().map_err(|error| {
                format!("G8-C {label} simulation tick {index}/{count} failed: {error}")
            })?;
        }
        Ok(())
    }

    fn start_trial(&mut self, trial: u32) -> Result<(), String> {
        let simulation = self.simulation.as_mut().expect("initialized simulation");
        reset_stage_and_wait(simulation, self.config.scenario, &format!("trial {trial}"))?;
        self.record_successful_reset_boundary()?;
        self.timestamp_batch = match self.config.mode {
            G8cMode::Coexistence => None,
            G8cMode::RenderProfile => Some(
                self.renderer
                    .as_ref()
                    .expect("initialized renderer")
                    .begin_render_timestamp_batch(
                        self.config
                            .profile_frames
                            .expect("validated profile frames"),
                    )
                    .map_err(|error| {
                        format!("G8-C Mode D trial {trial} timestamp setup failed: {error}")
                    })?,
            ),
        };
        let started = Instant::now();
        let device_error_start = self.device_error_count();
        let surface_error_start = self.surface_errors.len();
        self.phase = RunPhase::Trial {
            trial,
            started,
            last_frame_end: started,
            completed_ticks: 0,
            frame_index: 0,
            row_start: self.rows.len(),
            device_error_start,
            surface_error_start,
        };
        Ok(())
    }

    fn finish_trial(
        &mut self,
        trial: u32,
        row_start: usize,
        device_error_start: usize,
        surface_error_start: usize,
        elapsed: Duration,
    ) -> Result<(), String> {
        if self.config.mode == G8cMode::RenderProfile {
            let batch = self
                .timestamp_batch
                .take()
                .expect("Mode D trial owns timestamp batch");
            let timestamps = self
                .renderer
                .as_ref()
                .expect("initialized renderer")
                .finish_render_timestamp_batch(batch, self.timestamp_period_ns)
                .map_err(|error| {
                    format!("G8-C Mode D trial {trial} timestamp readback failed: {error}")
                })?;
            let rows = &mut self.rows[row_start..];
            if timestamps.len() != rows.len() {
                return Err(format!(
                    "G8-C Mode D trial {trial} timestamp/frame mismatch: {} vs {}",
                    timestamps.len(),
                    rows.len()
                ));
            }
            for (row, timestamp) in rows.iter_mut().zip(timestamps) {
                row.timestamp = Some(timestamp);
            }
        }
        self.simulation
            .as_ref()
            .expect("initialized simulation")
            .context
            .device
            .poll(wgpu::PollType::Wait)
            .map_err(|error| format!("G8-C trial {trial} final GPU wait failed: {error}"))?;
        let trial_device_errors = self.device_error_count().saturating_sub(device_error_start);
        let trial_surface_errors = self
            .surface_errors
            .len()
            .saturating_sub(surface_error_start);
        self.summaries.push(summarize_trial(
            trial,
            &self.rows[row_start..],
            elapsed,
            trial_device_errors as u32,
            trial_surface_errors as u32,
        ));
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn stop_failed_profile_trial(
        &mut self,
        event_loop: &ActiveEventLoop,
        trial: u32,
        started: Instant,
        row_start: usize,
        device_error_start: usize,
        surface_error_start: usize,
        error: MeasurementSurfaceFailure,
    ) -> Result<(), String> {
        let elapsed = started.elapsed();
        if self.rows.len() > row_start {
            self.finish_trial(
                trial,
                row_start,
                device_error_start,
                surface_error_start,
                elapsed,
            )?;
        } else {
            self.timestamp_batch.take();
            let trial_device_errors = self.device_error_count().saturating_sub(device_error_start);
            let trial_surface_errors = self
                .surface_errors
                .len()
                .saturating_sub(surface_error_start);
            self.summaries.push(summarize_trial(
                trial,
                &[],
                elapsed,
                trial_device_errors as u32,
                trial_surface_errors as u32,
            ));
        }
        self.write_failure_diagnostic_outputs()?;
        self.phase = RunPhase::Finished;
        self.published = true;
        self.fatal_error = Some(format!(
            "G8-C Mode D surface acquisition failed ({}) and the partial diagnostic was published",
            measurement_surface_error(&error)
        ));
        event_loop.exit();
        Ok(())
    }

    fn device_error_count(&self) -> usize {
        self.device_errors.lock().map_or(0, |errors| errors.len())
    }

    fn record_successful_reset_boundary(&mut self) -> Result<(), String> {
        self.successful_reset_boundaries = self
            .successful_reset_boundaries
            .checked_add(1)
            .ok_or_else(|| "G8-C successful reset-boundary counter overflow".to_string())?;
        Ok(())
    }

    fn publish(&mut self) -> Result<(), String> {
        self.require_canonical_live_window_size()?;
        validate_window_lifecycle_for_publication(&self.window_lifecycle)?;
        validate_reset_boundary_count(self.successful_reset_boundaries, self.config.trials)?;
        self.write_outputs()
    }

    fn write_failure_diagnostic_outputs(&mut self) -> Result<(), String> {
        self.write_outputs()
    }

    fn write_outputs(&mut self) -> Result<(), String> {
        let outputs = self
            .outputs
            .take()
            .ok_or_else(|| "G8-C outputs were already published".to_string())?;
        write_raw_csv(
            outputs.raw_csv,
            self.config.mode,
            self.config.scenario,
            self.timestamp_period_ns,
            &self.rows,
        )?;
        let device_errors = self
            .device_errors
            .lock()
            .map_err(|_| "G8-C device error ledger lock poisoned".to_string())?;
        write_metadata_json(
            outputs.metadata_json,
            &self.config,
            self.surface.expect("initialized surface"),
            self.simulation.as_ref().expect("initialized simulation"),
            MetadataDiagnostics {
                summaries: &self.summaries,
                device_errors: &device_errors,
                surface_errors: &self.surface_errors,
                window_lifecycle: &self.window_lifecycle,
            },
        )?;
        Ok(())
    }
}

impl ApplicationHandler for G8cApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        if let Err(error) = self.init(event_loop) {
            self.fatal_error = Some(error);
            event_loop.exit();
            return;
        }
        self.window
            .as_ref()
            .expect("initialized window")
            .request_redraw();
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
                self.fatal_error = Some("G8-C measurement window closed before completion".into());
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.redraw(event_loop) {
                    self.fatal_error = Some(error);
                    event_loop.exit();
                    return;
                }
                if !matches!(self.phase, RunPhase::Finished) {
                    window.request_redraw();
                }
            }
            WindowEvent::Resized(payload_size) => {
                let live_size = window.inner_size();
                if let Err(error) = self.handle_window_size_observation(
                    WindowSizeObservationSource::Resized,
                    payload_size,
                    live_size,
                ) {
                    self.fatal_error = Some(error);
                    event_loop.exit();
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                // winit does not expose a stable final-size payload on this
                // variant. The authoritative live size is routed through the
                // same classifier as Resized.
                let live_size = window.inner_size();
                if let Err(error) = self.handle_window_size_observation(
                    WindowSizeObservationSource::ScaleFactorChanged,
                    live_size,
                    live_size,
                ) {
                    self.fatal_error = Some(error);
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

pub fn run_worker(config: G8cWorkerConfig) -> Result<(), String> {
    config.validate()?;
    verify_current_executable_sha256(&config.binary_sha256)
        .map_err(|error| format!("G8-C binary authentication failed: {error}"))?;
    let outputs = OutputReservation::create(&config)?;
    let event_loop =
        EventLoop::new().map_err(|error| format!("event loop creation failed: {error}"))?;
    let mut app = G8cApp::new(config, outputs);
    event_loop
        .run_app(&mut app)
        .map_err(|error| format!("G8-C event loop failed: {error}"))?;
    if let Some(error) = app.fatal_error {
        return Err(error);
    }
    if !app.published {
        return Err("G8-C worker exited without publishing outputs".into());
    }
    Ok(())
}

fn reset_stage_and_wait(
    simulation: &mut Simulation,
    scenario: ScenarioId,
    label: &str,
) -> Result<(), String> {
    reset_and_stage_scenario(simulation, scenario)
        .map_err(|error| format!("G8-C {label} reset/stage failed: {error}"))?;
    simulation.context.queue.submit([]);
    simulation
        .context
        .device
        .poll(wgpu::PollType::Wait)
        .map(|_| ())
        .map_err(|error| format!("G8-C {label} reset/stage GPU wait failed: {error}"))
}

fn measurement_surface_error(error: &MeasurementSurfaceFailure) -> String {
    format!(
        "kind={};reconfigured={};fatal={};message={}",
        error.kind, error.reconfigured, error.fatal, error.message
    )
}

fn window_size_failure_message(
    source: WindowSizeObservationSource,
    payload_size: PhysicalSize<u32>,
    live_size: PhysicalSize<u32>,
    classification: WindowSizeEventClassification,
) -> String {
    format!(
        "G8-C canonical window-size guard failed: source={}; classification={}; required={}x{}; payload={}x{}; live={}x{}",
        source.as_str(),
        classification.as_str(),
        PHYSICAL_WIDTH,
        PHYSICAL_HEIGHT,
        payload_size.width,
        payload_size.height,
        live_size.width,
        live_size.height,
    )
}

fn summarize_trial(
    trial: u32,
    rows: &[FrameRow],
    elapsed: Duration,
    device_errors: u32,
    surface_errors: u32,
) -> TrialSummary {
    let elapsed_seconds = elapsed.as_secs_f64();
    let actual_simulation_ticks = rows.iter().map(|row| row.sim_ticks_executed).sum::<u64>();
    let presented_frames = rows.iter().filter(|row| row.presented).count() as u32;
    let missed_simulation_deadlines = rows
        .iter()
        .map(|row| row.missed_simulation_deadlines)
        .sum::<u64>();
    let catch_up_ticks = rows.iter().map(|row| row.catch_up_ticks).sum::<u64>();
    let scheduled = rows.last().map_or(0, |row| {
        row.scheduled_sim_ticks.max(actual_simulation_ticks)
    });
    let frame_times = rows
        .iter()
        .filter(|row| row.presented)
        .map(|row| row.frame_wall_ms)
        .collect::<Vec<_>>();
    let gpu_times = rows
        .iter()
        .filter_map(|row| row.timestamp.map(|sample| sample.duration_ms))
        .collect::<Vec<_>>();
    TrialSummary {
        trial,
        elapsed_ms: elapsed_seconds * 1000.0,
        actual_simulation_ticks,
        actual_simulation_tps: actual_simulation_ticks as f64 / elapsed_seconds,
        presented_frames,
        render_fps: f64::from(presented_frames) / elapsed_seconds,
        frame_p50_ms: percentile(&frame_times, 0.50),
        frame_p95_ms: percentile(&frame_times, 0.95),
        frame_p99_ms: percentile(&frame_times, 0.99),
        missed_simulation_deadlines,
        missed_deadline_ratio: if scheduled == 0 {
            0.0
        } else {
            missed_simulation_deadlines as f64 / scheduled as f64
        },
        catch_up_ticks,
        failed_surface_frames: rows.iter().filter(|row| !row.presented).count() as u32,
        device_errors,
        surface_errors,
        gpu_render_p50_ms: (!gpu_times.is_empty()).then(|| percentile(&gpu_times, 0.50)),
        gpu_render_p95_ms: (!gpu_times.is_empty()).then(|| percentile(&gpu_times, 0.95)),
        gpu_render_mean_ms: (!gpu_times.is_empty())
            .then(|| gpu_times.iter().sum::<f64>() / gpu_times.len() as f64),
    }
}

/// G8-A-compatible percentile over raw samples: round `p * (n - 1)` to the
/// nearest index (positive values therefore use `floor(x + 0.5)`).
fn percentile(values: &[f64], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    let index = (percentile * (ordered.len() - 1) as f64 + 0.5).floor() as usize;
    ordered[index.min(ordered.len() - 1)]
}

fn write_raw_csv(
    file: File,
    mode: G8cMode,
    scenario: ScenarioId,
    timestamp_period_ns: f32,
    rows: &[FrameRow],
) -> Result<(), String> {
    let mut writer = BufWriter::new(file);
    match mode {
        G8cMode::Coexistence => writeln!(writer, "{COEXISTENCE_CSV_HEADER}"),
        G8cMode::RenderProfile => writeln!(writer, "{RENDER_PROFILE_CSV_HEADER}"),
    }
    .map_err(|error| format!("cannot write G8-C raw CSV header: {error}"))?;
    for row in rows {
        let prefix = format!(
            "{},{},{},{},{},{:.9},{:.9},{},{},{},{},{},",
            mode.schema(),
            scenario.slug(),
            row.trial,
            row.frame_index,
            row.sim_tick,
            row.window_elapsed_ms,
            row.frame_wall_ms,
            row.scheduled_sim_ticks,
            row.sim_ticks_executed,
            row.catch_up_ticks,
            row.missed_simulation_deadlines,
            u8::from(row.presented),
        );
        match mode {
            G8cMode::Coexistence => {
                writeln!(writer, "{}{}", prefix, csv_escape(&row.surface_error))
            }
            G8cMode::RenderProfile => {
                let timestamp = row.timestamp.ok_or_else(|| {
                    format!(
                        "Mode D row trial {} frame {} lacks timestamp identity",
                        row.trial, row.frame_index
                    )
                })?;
                writeln!(
                    writer,
                    "{}{},{},{:.9},{:.9},{}",
                    prefix,
                    timestamp.start_tick,
                    timestamp.end_tick,
                    timestamp.duration_ms,
                    timestamp_period_ns,
                    csv_escape(&row.surface_error)
                )
            }
        }
        .map_err(|error| format!("cannot write G8-C raw CSV row: {error}"))?;
    }
    writer
        .flush()
        .map_err(|error| format!("cannot flush G8-C raw CSV: {error}"))
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

struct MetadataDiagnostics<'a> {
    summaries: &'a [TrialSummary],
    device_errors: &'a [String],
    surface_errors: &'a [String],
    window_lifecycle: &'a WindowLifecycleDiagnostics,
}

fn write_metadata_json(
    file: File,
    config: &G8cWorkerConfig,
    surface: SurfaceInfo,
    simulation: &Simulation,
    diagnostics: MetadataDiagnostics<'_>,
) -> Result<(), String> {
    let MetadataDiagnostics {
        summaries,
        device_errors,
        surface_errors,
        window_lifecycle,
    } = diagnostics;
    let mut writer = BufWriter::new(file);
    let adapter = &simulation.context.adapter_info;
    let build_profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    writeln!(writer, "{{").map_err(io_error)?;
    json_string_field(&mut writer, 2, "schema_version", config.mode.schema(), true)?;
    json_string_field(&mut writer, 2, "run_id", &config.run_id, true)?;
    json_string_field(&mut writer, 2, "mode", config.mode.as_str(), true)?;
    json_string_field(
        &mut writer,
        2,
        "source_sha",
        env!("POWDERGAME_BUILD_SOURCE_SHA"),
        true,
    )?;
    json_string_field(
        &mut writer,
        2,
        "git_state",
        env!("POWDERGAME_BUILD_GIT_STATE"),
        true,
    )?;
    json_string_field(&mut writer, 2, "build_profile", build_profile, true)?;
    json_string_field(&mut writer, 2, "binary_sha256", &config.binary_sha256, true)?;
    json_string_field(&mut writer, 2, "scenario", config.scenario.slug(), true)?;
    writeln!(writer, "  \"requested_config\": {{").map_err(io_error)?;
    writeln!(writer, "    \"width\": {},", config.width).map_err(io_error)?;
    writeln!(writer, "    \"height\": {},", config.height).map_err(io_error)?;
    writeln!(writer, "    \"chunk_size\": {},", config.chunk_size).map_err(io_error)?;
    writeln!(writer, "    \"sleep_enabled\": {},", config.sleep_enabled).map_err(io_error)?;
    writeln!(
        writer,
        "    \"sleep_threshold\": {},",
        config.sleep_threshold
    )
    .map_err(io_error)?;
    writeln!(writer, "    \"prewarm_secs\": {:.9},", config.prewarm_secs).map_err(io_error)?;
    writeln!(writer, "    \"trials\": {},", config.trials).map_err(io_error)?;
    writeln!(writer, "    \"target_tps\": {},", config.target_tps).map_err(io_error)?;
    let measurement_secs = match config.coexistence_window {
        Some(CoexistenceWindow::Seconds(value)) => value.to_string(),
        _ => "null".into(),
    };
    let measurement_frames = match config.coexistence_window {
        Some(CoexistenceWindow::Frames(value)) => value.to_string(),
        _ => "null".into(),
    };
    let profile_frames = config
        .profile_frames
        .map_or_else(|| "null".into(), |value| value.to_string());
    writeln!(writer, "    \"measurement_secs\": {measurement_secs},").map_err(io_error)?;
    writeln!(writer, "    \"measurement_frames\": {measurement_frames},").map_err(io_error)?;
    writeln!(writer, "    \"profile_frames\": {profile_frames}").map_err(io_error)?;
    writeln!(writer, "  }},").map_err(io_error)?;
    writeln!(writer, "  \"actual_surface\": {{").map_err(io_error)?;
    writeln!(writer, "    \"width\": {},", surface.width).map_err(io_error)?;
    writeln!(writer, "    \"height\": {},", surface.height).map_err(io_error)?;
    json_string_field(
        &mut writer,
        4,
        "format",
        &format!("{:?}", surface.format),
        true,
    )?;
    json_string_field(
        &mut writer,
        4,
        "present_mode",
        &format!("{:?}", surface.present_mode),
        false,
    )?;
    writeln!(writer, "  }},").map_err(io_error)?;
    let initial_live_size = window_lifecycle.initial_live_size.ok_or_else(|| {
        "cannot publish G8-C metadata without an initial live window-size observation".to_string()
    })?;
    let last_live_size = window_lifecycle.last_live_size.ok_or_else(|| {
        "cannot publish G8-C metadata without a final live window-size observation".to_string()
    })?;
    writeln!(writer, "  \"window_lifecycle\": {{").map_err(io_error)?;
    writeln!(writer, "    \"required_width\": {PHYSICAL_WIDTH},").map_err(io_error)?;
    writeln!(writer, "    \"required_height\": {PHYSICAL_HEIGHT},").map_err(io_error)?;
    writeln!(
        writer,
        "    \"initial_live_width\": {},",
        initial_live_size.width
    )
    .map_err(io_error)?;
    writeln!(
        writer,
        "    \"initial_live_height\": {},",
        initial_live_size.height
    )
    .map_err(io_error)?;
    writeln!(writer, "    \"last_live_width\": {},", last_live_size.width).map_err(io_error)?;
    writeln!(
        writer,
        "    \"last_live_height\": {},",
        last_live_size.height
    )
    .map_err(io_error)?;
    writeln!(
        writer,
        "    \"initial_live_size_confirmed\": {},",
        window_lifecycle.initial_live_size_confirmed
    )
    .map_err(io_error)?;
    writeln!(
        writer,
        "    \"canonical_noop_count\": {},",
        window_lifecycle.canonical_noop_count
    )
    .map_err(io_error)?;
    writeln!(
        writer,
        "    \"stale_payload_count\": {},",
        window_lifecycle.stale_payload_count
    )
    .map_err(io_error)?;
    writeln!(
        writer,
        "    \"fatal_live_resize_count\": {},",
        window_lifecycle.fatal_live_resize_count
    )
    .map_err(io_error)?;
    writeln!(
        writer,
        "    \"event_count\": {},",
        window_lifecycle.events.len()
    )
    .map_err(io_error)?;
    writeln!(writer, "    \"events\": [").map_err(io_error)?;
    for (index, event) in window_lifecycle.events.iter().enumerate() {
        writeln!(writer, "      {{").map_err(io_error)?;
        json_string_field(&mut writer, 8, "event_kind", event.source.as_str(), true)?;
        json_string_field(
            &mut writer,
            8,
            "classification",
            event.classification.as_str(),
            true,
        )?;
        writeln!(
            writer,
            "        \"payload_width\": {},",
            event.payload_size.width
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "        \"payload_height\": {},",
            event.payload_size.height
        )
        .map_err(io_error)?;
        writeln!(writer, "        \"live_width\": {},", event.live_size.width).map_err(io_error)?;
        writeln!(
            writer,
            "        \"live_height\": {}",
            event.live_size.height
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      }}{}",
            if index + 1 == window_lifecycle.events.len() {
                ""
            } else {
                ","
            }
        )
        .map_err(io_error)?;
    }
    writeln!(writer, "    ]").map_err(io_error)?;
    writeln!(writer, "  }},").map_err(io_error)?;
    writeln!(writer, "  \"adapter\": {{").map_err(io_error)?;
    json_string_field(&mut writer, 4, "name", &adapter.name, true)?;
    writeln!(writer, "    \"vendor\": {},", adapter.vendor).map_err(io_error)?;
    writeln!(writer, "    \"device\": {},", adapter.device).map_err(io_error)?;
    json_string_field(
        &mut writer,
        4,
        "backend",
        &format!("{:?}", adapter.backend),
        true,
    )?;
    json_string_field(&mut writer, 4, "driver", &adapter.driver, true)?;
    json_string_field(&mut writer, 4, "driver_info", &adapter.driver_info, false)?;
    writeln!(writer, "  }},").map_err(io_error)?;
    for (name, value) in [
        ("hud_enabled", false),
        ("inspector_enabled", false),
        ("text_diagnostics_enabled", false),
        ("screenshot_readback_enabled", false),
        (
            "timestamp_query_enabled",
            config.mode == G8cMode::RenderProfile,
        ),
    ] {
        writeln!(writer, "  \"{name}\": {value},").map_err(io_error)?;
    }
    writeln!(writer, "  \"device_error_count\": {},", device_errors.len()).map_err(io_error)?;
    writeln!(writer, "  \"device_errors\": [").map_err(io_error)?;
    for (index, error) in device_errors.iter().enumerate() {
        writeln!(
            writer,
            "    \"{}\"{}",
            json_escape(error),
            if index + 1 == device_errors.len() {
                ""
            } else {
                ","
            }
        )
        .map_err(io_error)?;
    }
    writeln!(writer, "  ],").map_err(io_error)?;
    writeln!(
        writer,
        "  \"surface_error_count\": {},",
        surface_errors.len()
    )
    .map_err(io_error)?;
    writeln!(writer, "  \"surface_errors\": [").map_err(io_error)?;
    for (index, error) in surface_errors.iter().enumerate() {
        writeln!(
            writer,
            "    \"{}\"{}",
            json_escape(error),
            if index + 1 == surface_errors.len() {
                ""
            } else {
                ","
            }
        )
        .map_err(io_error)?;
    }
    writeln!(writer, "  ],").map_err(io_error)?;
    json_string_field(
        &mut writer,
        2,
        "raw_csv",
        &config.raw_csv.to_string_lossy(),
        true,
    )?;
    writeln!(writer, "  \"trials\": [").map_err(io_error)?;
    for (index, summary) in summaries.iter().enumerate() {
        writeln!(writer, "    {{").map_err(io_error)?;
        writeln!(writer, "      \"trial\": {},", summary.trial).map_err(io_error)?;
        writeln!(writer, "      \"elapsed_ms\": {:.9},", summary.elapsed_ms).map_err(io_error)?;
        writeln!(
            writer,
            "      \"actual_simulation_ticks\": {},",
            summary.actual_simulation_ticks
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      \"actual_simulation_tps\": {:.9},",
            summary.actual_simulation_tps
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      \"presented_frames\": {},",
            summary.presented_frames
        )
        .map_err(io_error)?;
        writeln!(writer, "      \"render_fps\": {:.9},", summary.render_fps).map_err(io_error)?;
        writeln!(
            writer,
            "      \"frame_p50_ms\": {:.9},",
            summary.frame_p50_ms
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      \"frame_p95_ms\": {:.9},",
            summary.frame_p95_ms
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      \"frame_p99_ms\": {:.9},",
            summary.frame_p99_ms
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      \"missed_simulation_deadlines\": {},",
            summary.missed_simulation_deadlines
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      \"missed_deadline_ratio\": {:.9},",
            summary.missed_deadline_ratio
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      \"catch_up_ticks\": {},",
            summary.catch_up_ticks
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      \"failed_surface_frames\": {},",
            summary.failed_surface_frames
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      \"device_errors\": {},",
            summary.device_errors
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      \"surface_errors\": {},",
            summary.surface_errors
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      \"gpu_render_p50_ms\": {},",
            json_optional_number(summary.gpu_render_p50_ms)
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      \"gpu_render_p95_ms\": {},",
            json_optional_number(summary.gpu_render_p95_ms)
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "      \"gpu_render_mean_ms\": {}",
            json_optional_number(summary.gpu_render_mean_ms)
        )
        .map_err(io_error)?;
        writeln!(
            writer,
            "    }}{}",
            if index + 1 == summaries.len() {
                ""
            } else {
                ","
            }
        )
        .map_err(io_error)?;
    }
    writeln!(writer, "  ]").map_err(io_error)?;
    writeln!(writer, "}}").map_err(io_error)?;
    writer
        .flush()
        .map_err(|error| format!("cannot flush G8-C metadata JSON: {error}"))
}

fn json_string_field(
    writer: &mut impl Write,
    indent: usize,
    name: &str,
    value: &str,
    comma: bool,
) -> Result<(), String> {
    writeln!(
        writer,
        "{}\"{}\": \"{}\"{}",
        " ".repeat(indent),
        json_escape(name),
        json_escape(value),
        if comma { "," } else { "" }
    )
    .map_err(io_error)
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped
}

fn json_optional_number(value: Option<f64>) -> String {
    value.map_or_else(|| "null".into(), |number| format!("{number:.9}"))
}

fn io_error(error: std::io::Error) -> String {
    format!("cannot write G8-C metadata JSON: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_args(mode: &str) -> Vec<String> {
        vec![
            "--g8c-worker",
            "--mode",
            mode,
            "--scenario",
            "sand-fall",
            "--width",
            "256",
            "--height",
            "256",
            "--chunk",
            "64",
            "--sleep",
            "on",
            "--threshold",
            "16",
            "--prewarm-secs",
            "0",
            "--trials",
            "1",
            "--target-tps",
            "60",
            "--run-id",
            "pilot-1",
            "--binary-sha256",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "--raw-csv",
            "target/g8c.csv",
            "--metadata-json",
            "target/g8c.json",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }

    #[test]
    fn no_marker_preserves_existing_routing() {
        assert_eq!(
            worker_from_args(["--benchmark-gallery", "--smoke-frames", "3"]).unwrap(),
            None
        );
    }

    #[test]
    fn raw_schema_headers_are_frozen_for_independent_verification() {
        assert_eq!(COEXISTENCE_CSV_HEADER.split(',').count(), 13);
        assert_eq!(RENDER_PROFILE_CSV_HEADER.split(',').count(), 17);
        assert!(COEXISTENCE_CSV_HEADER.ends_with("presented,surface_error"));
        assert!(RENDER_PROFILE_CSV_HEADER.ends_with(
            "presented,gpu_start_tick,gpu_end_tick,gpu_render_ms,timestamp_period_ns,surface_error"
        ));
    }

    #[test]
    fn window_size_classifier_uses_live_size_as_final_authority() {
        let required = required_physical_size();
        let noncanonical = PhysicalSize::new(2864, 1560);
        assert_eq!(
            classify_window_size_event(required, required, required),
            WindowSizeEventClassification::CanonicalNoOp
        );
        assert_eq!(
            classify_window_size_event(required, noncanonical, required),
            WindowSizeEventClassification::StalePayloadIgnored
        );
        assert_eq!(
            classify_window_size_event(required, required, noncanonical),
            WindowSizeEventClassification::FatalNoncanonicalLiveSize
        );
        assert_eq!(
            classify_window_size_event(required, noncanonical, noncanonical),
            WindowSizeEventClassification::FatalNoncanonicalLiveSize
        );
    }

    #[test]
    fn repeated_stale_initial_payloads_are_ignored_and_measurement_remains_ready() {
        let required = required_physical_size();
        let stale_payload = PhysicalSize::new(2864, 1560);
        let mut diagnostics = WindowLifecycleDiagnostics::default();
        assert!(!diagnostics.measurement_can_start());
        diagnostics.confirm_initial_live_size(required).unwrap();
        for _ in 0..3 {
            assert_eq!(
                diagnostics
                    .record_event(
                        WindowSizeObservationSource::Resized,
                        stale_payload,
                        required,
                    )
                    .unwrap(),
                WindowSizeEventClassification::StalePayloadIgnored
            );
        }
        assert!(diagnostics.measurement_can_start());
        assert_eq!(diagnostics.canonical_noop_count, 0);
        assert_eq!(diagnostics.stale_payload_count, 3);
        assert_eq!(diagnostics.fatal_live_resize_count, 0);
        assert_eq!(diagnostics.events.len(), 3);
        assert!(validate_window_lifecycle_for_publication(&diagnostics).is_ok());
    }

    #[test]
    fn zero_or_noncanonical_live_size_is_fatal_without_renderer_resize() {
        let required = required_physical_size();
        for (payload, live) in [
            (required, PhysicalSize::new(0, 0)),
            (PhysicalSize::new(2864, 1560), PhysicalSize::new(1920, 1080)),
        ] {
            let classification = classify_window_size_event(required, payload, live);
            assert_eq!(
                classification,
                WindowSizeEventClassification::FatalNoncanonicalLiveSize
            );
            assert!(!classification.should_resize_renderer());
        }

        let mut diagnostics = WindowLifecycleDiagnostics::default();
        diagnostics.confirm_initial_live_size(required).unwrap();
        diagnostics
            .record_event(
                WindowSizeObservationSource::Resized,
                required,
                PhysicalSize::new(0, 0),
            )
            .unwrap();
        assert!(!diagnostics.measurement_can_start());
        assert_eq!(diagnostics.fatal_live_resize_count, 1);
        assert!(validate_window_lifecycle_for_publication(&diagnostics).is_err());
    }

    #[test]
    fn mode_c_and_mode_d_share_the_mode_independent_window_classifier() {
        let required = required_physical_size();
        for mode in [G8cMode::Coexistence, G8cMode::RenderProfile] {
            assert!(matches!(
                mode,
                G8cMode::Coexistence | G8cMode::RenderProfile
            ));
            let classification =
                classify_window_size_event(required, PhysicalSize::new(2864, 1560), required);
            assert_eq!(
                classification,
                WindowSizeEventClassification::StalePayloadIgnored
            );
            assert!(!classification.should_resize_renderer());
        }
    }

    #[test]
    fn measurement_start_is_gated_on_canonical_live_size_confirmation() {
        let mut diagnostics = WindowLifecycleDiagnostics::default();
        assert!(!diagnostics.measurement_can_start());
        assert!(diagnostics
            .confirm_initial_live_size(PhysicalSize::new(2864, 1560))
            .is_err());
        assert!(!diagnostics.measurement_can_start());

        let mut canonical = WindowLifecycleDiagnostics::default();
        canonical
            .confirm_initial_live_size(required_physical_size())
            .unwrap();
        assert!(canonical.measurement_can_start());
    }

    #[test]
    fn fixed_surface_size_and_fifo_checks_remain_enforced() {
        let canonical = SurfaceInfo {
            width: PHYSICAL_WIDTH,
            height: PHYSICAL_HEIGHT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            present_mode: wgpu::PresentMode::Fifo,
        };
        assert!(validate_surface_contract(&canonical).is_ok());

        let wrong_size = SurfaceInfo {
            width: 1920,
            ..canonical
        };
        assert!(validate_surface_contract(&wrong_size).is_err());

        let wrong_present_mode = SurfaceInfo {
            present_mode: wgpu::PresentMode::Immediate,
            ..canonical
        };
        assert!(validate_surface_contract(&wrong_present_mode).is_err());
    }

    #[test]
    fn typed_surface_failure_keeps_kind_recovery_and_fatal_identity() {
        let failure = MeasurementSurfaceFailure {
            kind: "lost",
            message: "swap chain lost".into(),
            reconfigured: true,
            fatal: false,
        };
        assert_eq!(
            measurement_surface_error(&failure),
            "kind=lost;reconfigured=true;fatal=false;message=swap chain lost"
        );
    }

    #[test]
    fn strict_cli_accepts_each_mode_and_all_five_scenarios() {
        for scenario in [
            "sand-fall",
            "water-flow",
            "fire-heat",
            "pressure-burst",
            "heavy-mixed-world",
        ] {
            let mut args = base_args("coexistence");
            let position = args.iter().position(|arg| arg == "sand-fall").unwrap();
            args[position] = scenario.into();
            args.extend(["--measurement-frames".into(), "60".into()]);
            let parsed = worker_from_args(args).unwrap().unwrap();
            assert_eq!(parsed.scenario.slug(), scenario);
            assert_eq!(parsed.mode, G8cMode::Coexistence);
        }

        let mut args = base_args("render-profile");
        args.extend(["--profile-frames".into(), "16".into()]);
        let parsed = worker_from_args(args).unwrap().unwrap();
        assert_eq!(parsed.mode, G8cMode::RenderProfile);
        assert_eq!(parsed.profile_frames, Some(16));
    }

    #[test]
    fn strict_cli_rejects_nonofficial_duplicates_unknown_and_cross_mode_options() {
        let mut g7 = base_args("coexistence");
        let position = g7.iter().position(|arg| arg == "sand-fall").unwrap();
        g7[position] = "active-sleep-g7".into();
        g7.extend(["--measurement-frames".into(), "60".into()]);
        assert!(worker_from_args(g7).is_err());

        let mut duplicate = base_args("coexistence");
        duplicate.extend([
            "--measurement-frames".into(),
            "60".into(),
            "--trials".into(),
            "2".into(),
        ]);
        assert!(worker_from_args(duplicate).is_err());

        let mut unknown = base_args("coexistence");
        unknown.extend([
            "--measurement-frames".into(),
            "60".into(),
            "--mystery".into(),
        ]);
        assert!(worker_from_args(unknown).is_err());

        let mut cross_mode = base_args("render-profile");
        cross_mode.extend([
            "--profile-frames".into(),
            "16".into(),
            "--measurement-secs".into(),
            "10".into(),
        ]);
        assert!(worker_from_args(cross_mode).is_err());
    }

    #[test]
    fn scheduler_accounts_for_on_time_tick_catchup_and_deadline_debt() {
        assert_eq!(
            tick_plan(Duration::from_millis(0), 60, 0),
            TickPlan {
                scheduled: 0,
                execute: 0,
                catch_up: 0,
                missed_deadlines: 0,
            }
        );
        assert_eq!(
            tick_plan(Duration::from_millis(17), 60, 0),
            TickPlan {
                scheduled: 1,
                execute: 1,
                catch_up: 0,
                missed_deadlines: 0,
            }
        );
        assert_eq!(
            tick_plan(Duration::from_millis(51), 60, 0),
            TickPlan {
                scheduled: 3,
                execute: 3,
                catch_up: 2,
                missed_deadlines: 2,
            }
        );
        assert_eq!(tick_plan(Duration::from_millis(51), 60, 3).execute, 0);
    }

    #[test]
    fn publication_requires_pre_warm_and_every_trial_reset_boundary() {
        assert!(validate_reset_boundary_count(2, 1).is_ok());
        assert!(validate_reset_boundary_count(4, 3).is_ok());
        assert!(validate_reset_boundary_count(1, 1).is_err());
        assert!(validate_reset_boundary_count(3, 3).is_err());
        assert!(validate_reset_boundary_count(5, 3).is_err());
        assert!(validate_reset_boundary_count(u32::MAX, u32::MAX).is_err());
    }

    #[test]
    fn g8a_percentiles_and_trial_accounting_reconstruct_from_rows() {
        let rows = [10.0, 20.0, 30.0, 40.0, 50.0]
            .into_iter()
            .enumerate()
            .map(|(index, frame_wall_ms)| FrameRow {
                trial: 1,
                frame_index: index as u32,
                sim_tick: index as u64 + 1,
                window_elapsed_ms: (index + 1) as f64 * 10.0,
                frame_wall_ms,
                scheduled_sim_ticks: index as u64 + 1,
                sim_ticks_executed: 1,
                catch_up_ticks: u64::from(index == 3),
                missed_simulation_deadlines: u64::from(index == 3),
                presented: true,
                timestamp: None,
                surface_error: String::new(),
            })
            .collect::<Vec<_>>();
        let summary = summarize_trial(1, &rows, Duration::from_millis(50), 0, 0);
        assert_eq!(summary.actual_simulation_ticks, 5);
        assert_eq!(summary.presented_frames, 5);
        assert_eq!(summary.frame_p50_ms, 30.0);
        assert_eq!(summary.frame_p95_ms, 50.0);
        assert_eq!(summary.frame_p99_ms, 50.0);
        assert_eq!(summary.catch_up_ticks, 1);
        assert_eq!(summary.missed_simulation_deadlines, 1);

        let mut with_surface_failure = rows.clone();
        with_surface_failure[0].presented = false;
        with_surface_failure[0].surface_error = "SurfaceFrameAcquireFailed: injected".into();
        let failed = summarize_trial(1, &with_surface_failure, Duration::from_millis(50), 0, 1);
        assert_eq!(failed.presented_frames, 4);
        assert_eq!(failed.failed_surface_frames, 1);
        assert_eq!(failed.surface_errors, 1);
        assert_eq!(percentile(&[10.0, 20.0, 30.0, 40.0], 0.50), 30.0);
    }

    #[test]
    fn output_reservation_is_no_overwrite() {
        let unique = format!(
            "powdergame-g8c-output-test-{}-{}",
            std::process::id(),
            Instant::now().elapsed().as_nanos()
        );
        let directory = std::env::temp_dir().join(unique);
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("raw.csv");
        let file = reserve_output(&path, "test").unwrap();
        drop(file);
        assert!(reserve_output(&path, "test").is_err());
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
