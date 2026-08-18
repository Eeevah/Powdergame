//! Read-only Cell Inspector model and bounded single-cell GPU readback.
//!
//! The collector copies exactly six four-byte values into one persistent
//! staging buffer. Cursor motion only changes the requested Cell; GPU work is
//! issued from the redraw loop at a maximum cadence of 10 Hz and never blocks
//! a production simulation tick.

use std::fmt;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

use powdergame_core::{
    combustion_descriptor, fuel_progress, registry_lookup, WorldConfig, ACTIVITY_ALL_BITS,
    ACTIVITY_MATTER, ACTIVITY_PRESSURE, ACTIVITY_REACTION, ACTIVITY_THERMAL, CHUNK_STATE_RUNNABLE,
    CHUNK_STATE_SLEEPING, FLAG_COMBUSTING, FLAG_FLAME_EVENT, MATERIAL_EMPTY, MATERIAL_ICE,
    MATERIAL_STEAM, MATERIAL_WATER,
};
use powdergame_gpu::Simulation;

const FIELD_BYTES: u64 = 4;
pub(crate) const INSPECTOR_READBACK_BYTES: u64 = FIELD_BYTES * 6;
pub(crate) const INSPECTOR_SAMPLE_INTERVAL: Duration = Duration::from_millis(100);

const MATERIAL_OFFSET: u64 = 0;
const TEMPERATURE_OFFSET: u64 = 4;
const PRESSURE_OFFSET: u64 = 8;
const FLAGS_OFFSET: u64 = 12;
const CELL_ACTIVITY_OFFSET: u64 = 16;
const CHUNK_STATE_OFFSET: u64 = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReadbackSource {
    Material,
    Temperature,
    Pressure,
    Flags,
    CellActivity,
    ChunkState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReadbackCopy {
    source: ReadbackSource,
    source_offset: u64,
    destination_offset: u64,
    size: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CellCoordinate {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScreenRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ScreenRect {
    pub fn right(self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(self) -> f32 {
        self.y + self.height
    }

    fn is_usable(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
            && self.width > 0.0
            && self.height > 0.0
    }
}

/// Places the compact tooltip near the cursor and wholly inside the rendered
/// world rectangle. Near the right/bottom edge it flips to the other side.
pub(crate) fn tooltip_rect(
    cursor: [f32; 2],
    tooltip_size: [f32; 2],
    world: ScreenRect,
) -> Option<ScreenRect> {
    if !world.is_usable()
        || !cursor[0].is_finite()
        || !cursor[1].is_finite()
        || !tooltip_size[0].is_finite()
        || !tooltip_size[1].is_finite()
        || tooltip_size[0] <= 0.0
        || tooltip_size[1] <= 0.0
        || cursor[0] < world.x
        || cursor[0] >= world.right()
        || cursor[1] < world.y
        || cursor[1] >= world.bottom()
    {
        return None;
    }

    let width = tooltip_size[0].min(world.width);
    let height = tooltip_size[1].min(world.height);
    let inset_x = 6.0_f32.min(((world.width - width) * 0.5).max(0.0));
    let inset_y = 6.0_f32.min(((world.height - height) * 0.5).max(0.0));
    let min_x = world.x + inset_x;
    let max_x = world.right() - width - inset_x;
    let min_y = world.y + inset_y;
    let max_y = world.bottom() - height - inset_y;

    let preferred_x = cursor[0] + 16.0;
    let flipped_x = cursor[0] - width - 16.0;
    let x = if preferred_x + width <= world.right() - inset_x {
        preferred_x
    } else {
        flipped_x
    }
    .clamp(min_x, max_x);

    let preferred_y = cursor[1] + 20.0;
    let flipped_y = cursor[1] - height - 14.0;
    let y = if preferred_y + height <= world.bottom() - inset_y {
        preferred_y
    } else {
        flipped_y
    }
    .clamp(min_y, max_y);

    Some(ScreenRect {
        x,
        y,
        width,
        height,
    })
}

/// Fixed detail region inside the otherwise unused lower portion of the
/// Gallery's left card. The rectangle is clamped to the physical surface.
pub(crate) fn detail_panel_rect(
    surface_width: f32,
    surface_height: f32,
    content_top: f32,
) -> Option<ScreenRect> {
    if !surface_width.is_finite()
        || !surface_height.is_finite()
        || !content_top.is_finite()
        || surface_width <= 0.0
        || surface_height <= 0.0
    {
        return None;
    }
    let margin = 8.0;
    let card_left = 18.0;
    let card_width = (390.0_f32 - 28.0).min((surface_width - card_left - margin).max(0.0));
    let x = (card_left + margin).clamp(0.0, surface_width);
    let width = (card_width - margin * 2.0).min(surface_width - x).max(0.0);
    let bottom = (surface_height - 66.0).max(0.0);
    let y = content_top.max(0.0).min(bottom);
    let height = (bottom - y).clamp(0.0, 292.0);
    (width >= 120.0 && height >= 80.0).then_some(ScreenRect {
        x,
        y,
        width,
        height,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CellInspectorSample {
    pub cell: CellCoordinate,
    pub chunk: CellCoordinate,
    pub material_id: u32,
    pub temperature: f32,
    pub pressure: f32,
    pub flags: u32,
    pub cell_activity: u32,
    pub chunk_state: u32,
    pub simulation_tick: u64,
    pub diagnostic_sequence: u64,
    pub request_generation: u64,
    pub world_epoch: u64,
    completed_at: Instant,
}

#[cfg(test)]
impl CellInspectorSample {
    pub(crate) fn fixture(material_id: u32, flags: u32) -> Self {
        Self {
            cell: CellCoordinate { x: 143, y: 207 },
            chunk: CellCoordinate { x: 2, y: 3 },
            material_id,
            temperature: 72.4,
            pressure: 53.5,
            flags,
            cell_activity: ACTIVITY_MATTER | ACTIVITY_THERMAL | ACTIVITY_PRESSURE,
            chunk_state: CHUNK_STATE_RUNNABLE,
            simulation_tick: 7412,
            diagnostic_sequence: 928,
            request_generation: 4,
            world_epoch: 2,
            completed_at: Instant::now(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InspectorDisplayState {
    Hidden,
    Pending,
    Ready,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InspectorHudData {
    pub display_state: InspectorDisplayState,
    pub details_visible: bool,
    pub hovered_cell: Option<CellCoordinate>,
    pub sample: Option<CellInspectorSample>,
    pub error_message: Option<String>,
    pub current_simulation_tick: u64,
    pub sample_age_ticks: Option<u64>,
    pub sample_age_millis: Option<u64>,
    pub sample_tick_is_future: bool,
}

fn inspector_display_state(
    hovered_cell: Option<CellCoordinate>,
    world_ready: bool,
    has_matching_sample: bool,
    has_readback_failure: bool,
) -> InspectorDisplayState {
    if hovered_cell.is_none() {
        InspectorDisplayState::Hidden
    } else if has_readback_failure {
        InspectorDisplayState::Failed
    } else if !world_ready || !has_matching_sample {
        InspectorDisplayState::Pending
    } else {
        InspectorDisplayState::Ready
    }
}

pub(crate) fn material_display_name(id: u32) -> String {
    if id == MATERIAL_EMPTY {
        "Empty".to_string()
    } else if let Some(descriptor) = registry_lookup(id) {
        descriptor.name.to_string()
    } else {
        format!("Invalid Material {id}")
    }
}

pub(crate) fn activity_display(mask: u32) -> String {
    let unknown = mask & !ACTIVITY_ALL_BITS;
    if unknown != 0 {
        return format!("Invalid Activity 0x{mask:08X}");
    }
    let mut parts = Vec::with_capacity(4);
    if mask & ACTIVITY_MATTER != 0 {
        parts.push("Matter");
    }
    if mask & ACTIVITY_THERMAL != 0 {
        parts.push("Thermal");
    }
    if mask & ACTIVITY_PRESSURE != 0 {
        parts.push("Pressure");
    }
    if mask & ACTIVITY_REACTION != 0 {
        parts.push("Reaction");
    }
    if parts.is_empty() {
        "None".to_string()
    } else {
        parts.join(" | ")
    }
}

pub(crate) fn chunk_state_display(state: u32) -> String {
    match state {
        CHUNK_STATE_RUNNABLE => "Runnable".to_string(),
        CHUNK_STATE_SLEEPING => "Sleeping".to_string(),
        value => format!("Invalid Chunk State {value}"),
    }
}

pub(crate) fn field_display(value: f32) -> String {
    if value.is_finite() {
        format!("{value:.1}")
    } else if value.is_nan() {
        "Diagnostic error: NaN".to_string()
    } else if value.is_sign_positive() {
        "Diagnostic error: +Inf".to_string()
    } else {
        "Diagnostic error: -Inf".to_string()
    }
}

pub(crate) fn flags_display(material_id: u32, flags: u32) -> Option<String> {
    let progress = fuel_progress(flags);
    let descriptor = combustion_descriptor(material_id);
    let mut parts = Vec::with_capacity(3);
    if flags & FLAG_COMBUSTING != 0 {
        parts.push("Combusting".to_string());
    }
    if flags & FLAG_FLAME_EVENT != 0 {
        parts.push("Flame event".to_string());
    }
    if let Some(descriptor) = descriptor {
        if progress != 0 || flags & (FLAG_COMBUSTING | FLAG_FLAME_EVENT) != 0 {
            parts.push(format!(
                "Fuel {progress} / {}",
                descriptor.burn_duration_ticks
            ));
        }
    } else if progress != 0 || flags & (FLAG_COMBUSTING | FLAG_FLAME_EVENT) != 0 {
        parts.push("Invalid combustion flags".to_string());
    }
    (!parts.is_empty()).then(|| parts.join(" | "))
}

pub(crate) fn phase_identity_display(material_id: u32) -> Option<&'static str> {
    matches!(material_id, MATERIAL_WATER | MATERIAL_STEAM | MATERIAL_ICE)
        .then_some("Water | Ice | Steam")
}

pub(crate) fn compact_sample_label(sample: &CellInspectorSample) -> String {
    let mut label = material_display_name(sample.material_id);
    if sample.flags & FLAG_COMBUSTING != 0 {
        label.push_str(" | Combusting");
    } else if sample.flags & FLAG_FLAME_EVENT != 0 {
        label.push_str(" | Flame event");
    }
    label
}

pub(crate) fn freshness_display(data: &InspectorHudData) -> String {
    if data.sample_tick_is_future {
        return "Invalid sample identity: future tick".to_string();
    }
    match (data.sample_age_ticks, data.sample_age_millis) {
        (Some(0), Some(milliseconds)) if milliseconds <= 250 => {
            format!("Fresh | 0 ticks old | {milliseconds} ms")
        }
        (Some(ticks), Some(milliseconds)) => {
            format!("Latest diagnostic | {ticks} ticks old | {milliseconds} ms")
        }
        _ => "Sample freshness unavailable".to_string(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RequestIdentity {
    cell: CellCoordinate,
    chunk: CellCoordinate,
    simulation_tick: u64,
    diagnostic_sequence: u64,
    request_generation: u64,
    selection_generation: u64,
    world_epoch: u64,
}

fn request_identity_is_current(
    identity: RequestIdentity,
    hovered_cell: Option<CellCoordinate>,
    world_epoch: u64,
    selection_generation: u64,
    latest_request_generation: u64,
    world_ready: bool,
) -> bool {
    world_ready
        && Some(identity.cell) == hovered_cell
        && identity.world_epoch == world_epoch
        && identity.selection_generation == selection_generation
        && identity.request_generation == latest_request_generation
}

fn readback_copy_plan(
    config: &WorldConfig,
    cell: CellCoordinate,
) -> Result<(CellCoordinate, [ReadbackCopy; 6]), CellInspectorReadbackError> {
    if cell.x >= config.width || cell.y >= config.height {
        return Err(CellInspectorReadbackError::CoordinateOutOfRange(cell));
    }
    let cell_index = u64::from(cell.y) * u64::from(config.width) + u64::from(cell.x);
    let chunks_x = config.width.div_ceil(config.chunk_size);
    let chunk = CellCoordinate {
        x: cell.x / config.chunk_size,
        y: cell.y / config.chunk_size,
    };
    let chunk_index = u64::from(chunk.y) * u64::from(chunks_x) + u64::from(chunk.x);
    let cell_offset = cell_index * FIELD_BYTES;
    let chunk_offset = chunk_index * FIELD_BYTES;
    Ok((
        chunk,
        [
            ReadbackCopy {
                source: ReadbackSource::Material,
                source_offset: cell_offset,
                destination_offset: MATERIAL_OFFSET,
                size: FIELD_BYTES,
            },
            ReadbackCopy {
                source: ReadbackSource::Temperature,
                source_offset: cell_offset,
                destination_offset: TEMPERATURE_OFFSET,
                size: FIELD_BYTES,
            },
            ReadbackCopy {
                source: ReadbackSource::Pressure,
                source_offset: cell_offset,
                destination_offset: PRESSURE_OFFSET,
                size: FIELD_BYTES,
            },
            ReadbackCopy {
                source: ReadbackSource::Flags,
                source_offset: cell_offset,
                destination_offset: FLAGS_OFFSET,
                size: FIELD_BYTES,
            },
            ReadbackCopy {
                source: ReadbackSource::CellActivity,
                source_offset: cell_offset,
                destination_offset: CELL_ACTIVITY_OFFSET,
                size: FIELD_BYTES,
            },
            ReadbackCopy {
                source: ReadbackSource::ChunkState,
                source_offset: chunk_offset,
                destination_offset: CHUNK_STATE_OFFSET,
                size: FIELD_BYTES,
            },
        ],
    ))
}

struct PendingReadback {
    identity: RequestIdentity,
    receiver: Receiver<Result<(), String>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CallbackProgress {
    Pending,
    Ready,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CellInspectorReadbackError {
    ReadbackAlreadyPending,
    DevicePollFailed(String),
    MapFailed(String),
    CallbackChannelDisconnected,
    MappedDataTruncated { expected: usize, actual: usize },
    CoordinateOutOfRange(CellCoordinate),
}

impl fmt::Display for CellInspectorReadbackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadbackAlreadyPending => {
                formatter.write_str("Inspector readback already pending")
            }
            Self::DevicePollFailed(message) => {
                write!(formatter, "Inspector device poll failed: {message}")
            }
            Self::MapFailed(message) => write!(formatter, "Inspector map failed: {message}"),
            Self::CallbackChannelDisconnected => {
                formatter.write_str("Inspector map callback channel disconnected")
            }
            Self::MappedDataTruncated { expected, actual } => write!(
                formatter,
                "Inspector mapped data truncated: expected {expected} bytes, got {actual}"
            ),
            Self::CoordinateOutOfRange(cell) => write!(
                formatter,
                "Inspector Cell {},{} is outside the simulation world",
                cell.x, cell.y
            ),
        }
    }
}

impl std::error::Error for CellInspectorReadbackError {}

fn classify_callback(
    callback: Result<Result<(), String>, TryRecvError>,
) -> Result<CallbackProgress, CellInspectorReadbackError> {
    match callback {
        Err(TryRecvError::Empty) => Ok(CallbackProgress::Pending),
        Err(TryRecvError::Disconnected) => {
            Err(CellInspectorReadbackError::CallbackChannelDisconnected)
        }
        Ok(Err(message)) => Err(CellInspectorReadbackError::MapFailed(message)),
        Ok(Ok(())) => Ok(CallbackProgress::Ready),
    }
}

pub(crate) struct CellInspectorCollector {
    staging: wgpu::Buffer,
    pending: Option<PendingReadback>,
    hovered_cell: Option<CellCoordinate>,
    latest_sample: Option<CellInspectorSample>,
    details_visible: bool,
    world_ready: bool,
    failure_message: Option<String>,
    world_epoch: u64,
    selection_generation: u64,
    next_request_generation: u64,
    completed_sequence: u64,
    last_request_at: Option<Instant>,
}

impl CellInspectorCollector {
    pub(crate) fn new(simulation: &Simulation) -> Self {
        let staging = simulation
            .context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("cell-inspector/single-cell-readback"),
                size: INSPECTOR_READBACK_BYTES,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
        Self {
            staging,
            pending: None,
            hovered_cell: None,
            latest_sample: None,
            details_visible: false,
            world_ready: true,
            failure_message: None,
            world_epoch: 0,
            selection_generation: 0,
            next_request_generation: 0,
            completed_sequence: 0,
            last_request_at: None,
        }
    }

    pub(crate) fn set_hover(&mut self, hovered_cell: Option<CellCoordinate>) {
        if self.hovered_cell == hovered_cell {
            return;
        }
        self.hovered_cell = hovered_cell;
        self.selection_generation = self.selection_generation.wrapping_add(1);
        self.latest_sample = None;
    }

    pub(crate) fn toggle_details(&mut self) -> bool {
        self.details_visible = !self.details_visible;
        self.details_visible
    }

    pub(crate) fn begin_world_change(&mut self) {
        self.invalidate_for_pending_world();
    }

    pub(crate) fn mark_ready(&mut self) {
        self.world_ready = true;
        self.failure_message = None;
        self.latest_sample = None;
        self.last_request_at = None;
    }

    pub(crate) fn mark_unavailable(&mut self, message: impl Into<String>) {
        // World/scenario staging unavailability is not an Inspector readback
        // failure. Its structured error is logged by the caller, while the
        // presentation remains silent until a world is ready to sample again.
        let _ = message.into();
        self.invalidate_for_pending_world();
    }

    fn invalidate_for_pending_world(&mut self) {
        self.world_epoch = self.world_epoch.wrapping_add(1);
        self.selection_generation = self.selection_generation.wrapping_add(1);
        self.world_ready = false;
        self.failure_message = None;
        self.latest_sample = None;
        self.last_request_at = None;
        self.cancel_pending();
    }

    fn record_readback_failure(&mut self, error: &CellInspectorReadbackError) {
        self.world_epoch = self.world_epoch.wrapping_add(1);
        self.selection_generation = self.selection_generation.wrapping_add(1);
        self.world_ready = false;
        self.failure_message = Some(format!("Inspector unavailable: {error}"));
        self.latest_sample = None;
        self.last_request_at = None;
        self.cancel_pending();
    }

    pub(crate) fn shutdown(&mut self) {
        self.mark_unavailable("Inspector unavailable: app shutdown");
        self.hovered_cell = None;
    }

    pub(crate) fn hud_data(&self, current_simulation_tick: u64, now: Instant) -> InspectorHudData {
        let matching_sample = self.latest_sample.as_ref().filter(|sample| {
            Some(sample.cell) == self.hovered_cell && sample.world_epoch == self.world_epoch
        });
        let sample_tick_is_future =
            matching_sample.is_some_and(|sample| sample.simulation_tick > current_simulation_tick);
        let sample_age_ticks = matching_sample
            .and_then(|sample| current_simulation_tick.checked_sub(sample.simulation_tick));
        let sample_age_millis = matching_sample.map(|sample| {
            u64::try_from(
                now.saturating_duration_since(sample.completed_at)
                    .as_millis(),
            )
            .unwrap_or(u64::MAX)
        });
        let display_state = inspector_display_state(
            self.hovered_cell,
            self.world_ready,
            matching_sample.is_some(),
            self.failure_message.is_some(),
        );
        InspectorHudData {
            display_state,
            details_visible: self.details_visible,
            hovered_cell: self.hovered_cell,
            sample: matching_sample.cloned(),
            error_message: self.failure_message.clone(),
            current_simulation_tick,
            sample_age_ticks,
            sample_age_millis,
            sample_tick_is_future,
        }
    }

    pub(crate) fn update(
        &mut self,
        simulation: &Simulation,
        current_simulation_tick: u64,
        now: Instant,
    ) -> Result<(), CellInspectorReadbackError> {
        if let Err(error) = self.poll_pending(simulation, now) {
            self.record_readback_failure(&error);
            return Err(error);
        }

        let Some(cell) = self.hovered_cell else {
            return Ok(());
        };
        if !self.world_ready || self.pending.is_some() {
            return Ok(());
        }
        let needs_sample = self.latest_sample.as_ref().is_none_or(|sample| {
            sample.cell != cell
                || sample.world_epoch != self.world_epoch
                || sample.simulation_tick != current_simulation_tick
        });
        if !needs_sample {
            return Ok(());
        }
        let cadence_elapsed = self
            .last_request_at
            .is_none_or(|last| now.saturating_duration_since(last) >= INSPECTOR_SAMPLE_INTERVAL);
        if cadence_elapsed {
            if let Err(error) =
                self.request_readback(simulation, cell, current_simulation_tick, now)
            {
                self.record_readback_failure(&error);
                return Err(error);
            }
        }
        Ok(())
    }

    fn request_readback(
        &mut self,
        simulation: &Simulation,
        cell: CellCoordinate,
        simulation_tick: u64,
        now: Instant,
    ) -> Result<(), CellInspectorReadbackError> {
        if self.pending.is_some() {
            return Err(CellInspectorReadbackError::ReadbackAlreadyPending);
        }
        let (chunk, copy_plan) = readback_copy_plan(&simulation.world.config, cell)?;

        let device = &simulation.context.device;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("cell-inspector/single-cell-copy"),
        });
        for copy in copy_plan {
            let source = match copy.source {
                ReadbackSource::Material => &simulation.world.material_current,
                ReadbackSource::Temperature => &simulation.world.temperature_current,
                ReadbackSource::Pressure => &simulation.world.pressure_current,
                ReadbackSource::Flags => &simulation.world.flags_current,
                ReadbackSource::CellActivity => &simulation.world.cell_activity,
                ReadbackSource::ChunkState => &simulation.world.chunk_state,
            };
            encoder.copy_buffer_to_buffer(
                source,
                copy.source_offset,
                &self.staging,
                copy.destination_offset,
                copy.size,
            );
        }
        simulation.context.queue.submit([encoder.finish()]);

        let (sender, receiver) = mpsc::channel::<Result<(), String>>();
        self.staging
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result.map_err(|error| error.to_string()));
            });

        self.next_request_generation = self.next_request_generation.wrapping_add(1);
        self.pending = Some(PendingReadback {
            identity: RequestIdentity {
                cell,
                chunk,
                simulation_tick,
                diagnostic_sequence: self.completed_sequence.wrapping_add(1),
                request_generation: self.next_request_generation,
                selection_generation: self.selection_generation,
                world_epoch: self.world_epoch,
            },
            receiver,
        });
        self.last_request_at = Some(now);
        Ok(())
    }

    fn poll_pending(
        &mut self,
        simulation: &Simulation,
        now: Instant,
    ) -> Result<(), CellInspectorReadbackError> {
        let Some(pending) = self.pending.as_ref() else {
            return Ok(());
        };
        let mut callback = pending.receiver.try_recv();
        if matches!(callback, Err(TryRecvError::Empty)) {
            simulation
                .context
                .device
                .poll(wgpu::PollType::Poll)
                .map_err(|error| {
                    self.cancel_pending();
                    CellInspectorReadbackError::DevicePollFailed(error.to_string())
                })?;
            callback = self
                .pending
                .as_ref()
                .expect("pending Inspector readback remains after successful poll")
                .receiver
                .try_recv();
        }

        match classify_callback(callback) {
            Ok(CallbackProgress::Pending) => Ok(()),
            Err(CellInspectorReadbackError::CallbackChannelDisconnected) => {
                self.cancel_pending();
                Err(CellInspectorReadbackError::CallbackChannelDisconnected)
            }
            Err(CellInspectorReadbackError::MapFailed(message)) => {
                self.cancel_pending();
                Err(CellInspectorReadbackError::MapFailed(message))
            }
            Ok(CallbackProgress::Ready) => self.consume_mapped(now),
            Err(error) => Err(error),
        }
    }

    fn consume_mapped(&mut self, now: Instant) -> Result<(), CellInspectorReadbackError> {
        let pending = self
            .pending
            .take()
            .expect("mapped Inspector callback requires pending identity");
        let mapped = self.staging.slice(..).get_mapped_range();
        let expected = INSPECTOR_READBACK_BYTES as usize;
        let parsed = if mapped.len() < expected {
            Err(CellInspectorReadbackError::MappedDataTruncated {
                expected,
                actual: mapped.len(),
            })
        } else {
            let u32_at = |offset: usize| {
                u32::from_ne_bytes(mapped[offset..offset + 4].try_into().expect("four bytes"))
            };
            let f32_at = |offset: usize| {
                f32::from_ne_bytes(mapped[offset..offset + 4].try_into().expect("four bytes"))
            };
            Ok((
                u32_at(MATERIAL_OFFSET as usize),
                f32_at(TEMPERATURE_OFFSET as usize),
                f32_at(PRESSURE_OFFSET as usize),
                u32_at(FLAGS_OFFSET as usize),
                u32_at(CELL_ACTIVITY_OFFSET as usize),
                u32_at(CHUNK_STATE_OFFSET as usize),
            ))
        };
        drop(mapped);
        self.staging.unmap();
        let (material_id, temperature, pressure, flags, cell_activity, chunk_state) = parsed?;

        let identity = pending.identity;
        let identity_is_current = request_identity_is_current(
            identity,
            self.hovered_cell,
            self.world_epoch,
            self.selection_generation,
            self.next_request_generation,
            self.world_ready,
        );
        if !identity_is_current {
            return Ok(());
        }
        self.completed_sequence = identity.diagnostic_sequence;
        self.latest_sample = Some(CellInspectorSample {
            cell: identity.cell,
            chunk: identity.chunk,
            material_id,
            temperature,
            pressure,
            flags,
            cell_activity,
            chunk_state,
            simulation_tick: identity.simulation_tick,
            diagnostic_sequence: identity.diagnostic_sequence,
            request_generation: identity.request_generation,
            world_epoch: identity.world_epoch,
            completed_at: now,
        });
        Ok(())
    }

    fn cancel_pending(&mut self) {
        if self.pending.take().is_some() {
            // `unmap` also cancels a not-yet-completed map request. Any late
            // callback targets a receiver that has already been dropped.
            self.staging.unmap();
        }
    }
}

impl Drop for CellInspectorCollector {
    fn drop(&mut self) {
        self.cancel_pending();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use powdergame_core::{
        with_fuel_progress, MATERIAL_BOUNDARY_BLOCK, MATERIAL_OIL, MATERIAL_SAND, MATERIAL_SMOKE,
        MATERIAL_STONE, MATERIAL_WOOD,
    };

    fn sample(material_id: u32, flags: u32) -> CellInspectorSample {
        CellInspectorSample::fixture(material_id, flags)
    }

    #[test]
    fn material_names_cover_empty_registry_and_invalid_fallback() {
        let expected = [
            "Empty",
            "Boundary Block",
            "Stone",
            "Sand",
            "Water",
            "Oil",
            "Steam",
            "Smoke",
            "Ice",
            "Wood",
        ];
        for (id, expected_name) in expected.into_iter().enumerate() {
            assert_eq!(material_display_name(id as u32), expected_name);
        }
        assert_eq!(material_display_name(42), "Invalid Material 42");
        assert_eq!(
            material_display_name(u32::MAX),
            "Invalid Material 4294967295"
        );
        assert_eq!(
            material_display_name(MATERIAL_BOUNDARY_BLOCK),
            "Boundary Block"
        );
    }

    #[test]
    fn activity_chunk_and_fields_format_without_panics() {
        assert_eq!(activity_display(0), "None");
        assert_eq!(activity_display(ACTIVITY_MATTER), "Matter");
        assert_eq!(
            activity_display(ACTIVITY_ALL_BITS),
            "Matter | Thermal | Pressure | Reaction"
        );
        assert_eq!(activity_display(1 << 8), "Invalid Activity 0x00000100");
        assert_eq!(chunk_state_display(CHUNK_STATE_RUNNABLE), "Runnable");
        assert_eq!(chunk_state_display(CHUNK_STATE_SLEEPING), "Sleeping");
        assert_eq!(chunk_state_display(7), "Invalid Chunk State 7");
        assert_eq!(field_display(72.44), "72.4");
        assert_eq!(field_display(f32::NAN), "Diagnostic error: NaN");
        assert_eq!(field_display(f32::INFINITY), "Diagnostic error: +Inf");
        assert_eq!(field_display(f32::NEG_INFINITY), "Diagnostic error: -Inf");
    }

    #[test]
    fn combustion_flame_and_fuel_use_authoritative_helpers() {
        let wood_flags = with_fuel_progress(FLAG_COMBUSTING | FLAG_FLAME_EVENT, 438);
        assert_eq!(
            flags_display(MATERIAL_WOOD, wood_flags).as_deref(),
            Some("Combusting | Flame event | Fuel 438 / 900")
        );
        let oil_flags = with_fuel_progress(0, 31);
        assert_eq!(
            flags_display(MATERIAL_OIL, oil_flags).as_deref(),
            Some("Fuel 31 / 600")
        );
        assert_eq!(
            flags_display(MATERIAL_STONE, FLAG_COMBUSTING).as_deref(),
            Some("Combusting | Invalid combustion flags")
        );
        assert_eq!(
            compact_sample_label(&sample(MATERIAL_WOOD, wood_flags)),
            "Wood | Combusting"
        );
        assert_eq!(
            phase_identity_display(MATERIAL_WATER),
            Some("Water | Ice | Steam")
        );
        assert_eq!(phase_identity_display(MATERIAL_SAND), None);
    }

    #[test]
    fn tooltip_and_detail_panel_are_clamped_deterministically() {
        let world = ScreenRect {
            x: 420.0,
            y: 60.0,
            width: 760.0,
            height: 760.0,
        };
        assert!(tooltip_rect([419.9, 100.0], [120.0, 34.0], world).is_none());
        assert!(tooltip_rect([1180.0, 100.0], [120.0, 34.0], world).is_none());
        assert!(tooltip_rect([800.0, 59.9], [120.0, 34.0], world).is_none());
        assert!(tooltip_rect([800.0, 820.0], [120.0, 34.0], world).is_none());
        let top_left = tooltip_rect([420.0, 60.0], [120.0, 34.0], world).unwrap();
        assert!(top_left.x >= world.x && top_left.y >= world.y);
        let bottom_right = tooltip_rect([1179.9, 819.9], [160.0, 50.0], world).unwrap();
        assert!(bottom_right.right() <= world.right());
        assert!(bottom_right.bottom() <= world.bottom());
        assert_eq!(
            tooltip_rect([800.0, 400.0], [120.0, 34.0], world),
            tooltip_rect([800.0, 400.0], [120.0, 34.0], world)
        );
        let tiny_world = ScreenRect {
            x: 10.0,
            y: 20.0,
            width: 24.0,
            height: 12.0,
        };
        let tiny = tooltip_rect([11.0, 21.0], [80.0, 34.0], tiny_world).unwrap();
        assert_eq!(tiny, tiny_world);
        assert!(tiny.x >= tiny_world.x && tiny.right() <= tiny_world.right());
        assert!(tiny.y >= tiny_world.y && tiny.bottom() <= tiny_world.bottom());
        let panel = detail_panel_rect(1600.0, 900.0, 530.0).unwrap();
        assert!(panel.x >= 0.0 && panel.right() <= 1600.0);
        assert!(panel.y >= 530.0 && panel.bottom() <= 900.0);
        assert!(detail_panel_rect(100.0, 70.0, 60.0).is_none());
    }

    #[test]
    fn freshness_keeps_sim_tick_and_diagnostic_sequence_distinct() {
        let now = Instant::now();
        let mut ready = InspectorHudData {
            display_state: InspectorDisplayState::Ready,
            details_visible: true,
            hovered_cell: Some(CellCoordinate { x: 1, y: 2 }),
            sample: Some(sample(MATERIAL_WATER, 0)),
            error_message: None,
            current_simulation_tick: 7412,
            sample_age_ticks: Some(0),
            sample_age_millis: Some(25),
            sample_tick_is_future: false,
        };
        assert_eq!(freshness_display(&ready), "Fresh | 0 ticks old | 25 ms");
        ready.current_simulation_tick = 7420;
        ready.sample_age_ticks = Some(8);
        ready.sample_age_millis = Some(140);
        assert_eq!(
            freshness_display(&ready),
            "Latest diagnostic | 8 ticks old | 140 ms"
        );
        ready.sample_tick_is_future = true;
        assert_eq!(
            freshness_display(&ready),
            "Invalid sample identity: future tick"
        );
        let _ = now;
    }

    #[test]
    fn presentation_state_separates_silent_pending_from_readback_failure() {
        let cell = Some(CellCoordinate { x: 7, y: 9 });
        assert_eq!(
            inspector_display_state(None, true, false, false),
            InspectorDisplayState::Hidden
        );
        assert_eq!(
            inspector_display_state(cell, true, false, false),
            InspectorDisplayState::Pending
        );
        assert_eq!(
            inspector_display_state(cell, false, false, false),
            InspectorDisplayState::Pending
        );
        assert_eq!(
            inspector_display_state(cell, true, true, false),
            InspectorDisplayState::Ready
        );
        assert_eq!(
            inspector_display_state(cell, false, false, true),
            InspectorDisplayState::Failed
        );
        assert_eq!(
            inspector_display_state(None, false, false, true),
            InspectorDisplayState::Hidden
        );
    }

    #[test]
    fn readback_contract_is_one_mapped_batch_of_six_four_byte_fields() {
        assert_eq!(INSPECTOR_READBACK_BYTES, 24);
        assert_eq!(INSPECTOR_SAMPLE_INTERVAL, Duration::from_millis(100));
        let config = WorldConfig::new(128, 128, 64).expect("world config");
        let cell = CellCoordinate { x: 70, y: 90 };
        let (chunk, plan) = readback_copy_plan(&config, cell).expect("copy plan");
        assert_eq!(chunk, CellCoordinate { x: 1, y: 1 });
        assert_eq!(plan.len(), 6);
        assert_eq!(
            plan.map(|copy| copy.source),
            [
                ReadbackSource::Material,
                ReadbackSource::Temperature,
                ReadbackSource::Pressure,
                ReadbackSource::Flags,
                ReadbackSource::CellActivity,
                ReadbackSource::ChunkState,
            ]
        );
        assert!(plan.iter().all(|copy| copy.size == FIELD_BYTES));
        assert_eq!(
            plan.map(|copy| copy.destination_offset),
            [0, 4, 8, 12, 16, 20]
        );
        let cell_offset = (u64::from(cell.y) * 128 + u64::from(cell.x)) * FIELD_BYTES;
        assert!(plan[..5]
            .iter()
            .all(|copy| copy.source_offset == cell_offset));
        assert_eq!(plan[5].source_offset, 3 * FIELD_BYTES);
        assert_eq!(
            plan.last().unwrap().destination_offset + plan.last().unwrap().size,
            INSPECTOR_READBACK_BYTES
        );
        assert_eq!(
            readback_copy_plan(&config, CellCoordinate { x: 128, y: 0 }),
            Err(CellInspectorReadbackError::CoordinateOutOfRange(
                CellCoordinate { x: 128, y: 0 }
            ))
        );
    }

    #[test]
    fn request_identity_requires_cell_epoch_selection_request_and_ready_world() {
        let identity = RequestIdentity {
            cell: CellCoordinate { x: 7, y: 9 },
            chunk: CellCoordinate { x: 0, y: 0 },
            simulation_tick: 18,
            diagnostic_sequence: 2,
            request_generation: 4,
            selection_generation: 3,
            world_epoch: 5,
        };
        let current = |candidate: RequestIdentity,
                       hovered_cell: Option<CellCoordinate>,
                       world_epoch: u64,
                       selection_generation: u64,
                       request_generation: u64,
                       world_ready: bool| {
            request_identity_is_current(
                candidate,
                hovered_cell,
                world_epoch,
                selection_generation,
                request_generation,
                world_ready,
            )
        };
        assert!(current(identity, Some(identity.cell), 5, 3, 4, true));
        assert!(!current(
            identity,
            Some(CellCoordinate { x: 8, y: 9 }),
            5,
            3,
            4,
            true
        ));
        assert!(!current(identity, Some(identity.cell), 6, 3, 4, true));
        assert!(!current(identity, Some(identity.cell), 5, 4, 4, true));
        assert!(!current(identity, Some(identity.cell), 5, 3, 5, true));
        assert!(!current(identity, Some(identity.cell), 5, 3, 4, false));
    }

    #[test]
    fn map_failure_and_disconnect_are_structured() {
        assert_eq!(
            classify_callback(Err(TryRecvError::Empty)),
            Ok(CallbackProgress::Pending)
        );
        assert_eq!(classify_callback(Ok(Ok(()))), Ok(CallbackProgress::Ready));
        assert_eq!(
            classify_callback(Ok(Err("injected".to_string()))),
            Err(CellInspectorReadbackError::MapFailed(
                "injected".to_string()
            ))
        );
        assert_eq!(
            classify_callback(Err(TryRecvError::Disconnected)),
            Err(CellInspectorReadbackError::CallbackChannelDisconnected)
        );
        assert_eq!(
            CellInspectorReadbackError::MapFailed("injected".to_string()).to_string(),
            "Inspector map failed: injected"
        );
    }

    #[test]
    fn gpu_collector_reads_one_identity_discards_stale_and_reuses_after_reset() {
        #[derive(Clone, Copy)]
        struct ProbeValues {
            material: u32,
            temperature: f32,
            pressure: f32,
            flags: u32,
            activity: u32,
            chunk_state: u32,
        }

        fn write_probe(simulation: &Simulation, cell: CellCoordinate, values: ProbeValues) {
            let config = &simulation.world.config;
            let cell_index = u64::from(cell.y) * u64::from(config.width) + u64::from(cell.x);
            let cell_offset = cell_index * FIELD_BYTES;
            let chunks_x = config.width.div_ceil(config.chunk_size);
            let chunk_index = u64::from(cell.y / config.chunk_size) * u64::from(chunks_x)
                + u64::from(cell.x / config.chunk_size);
            let chunk_offset = chunk_index * FIELD_BYTES;
            let queue = &simulation.context.queue;
            queue.write_buffer(
                &simulation.world.material_current,
                cell_offset,
                &values.material.to_ne_bytes(),
            );
            queue.write_buffer(
                &simulation.world.temperature_current,
                cell_offset,
                &values.temperature.to_ne_bytes(),
            );
            queue.write_buffer(
                &simulation.world.pressure_current,
                cell_offset,
                &values.pressure.to_ne_bytes(),
            );
            queue.write_buffer(
                &simulation.world.flags_current,
                cell_offset,
                &values.flags.to_ne_bytes(),
            );
            queue.write_buffer(
                &simulation.world.cell_activity,
                cell_offset,
                &values.activity.to_ne_bytes(),
            );
            queue.write_buffer(
                &simulation.world.chunk_state,
                chunk_offset,
                &values.chunk_state.to_ne_bytes(),
            );
        }

        let context = pollster::block_on(powdergame_gpu::GpuContext::new()).expect("GPU context");
        let simulation = Simulation::with_context(
            context,
            powdergame_core::WorldConfig::new(128, 128, 64).expect("world config"),
        )
        .expect("simulation");
        let first = CellCoordinate { x: 70, y: 90 };
        let second = CellCoordinate { x: 12, y: 34 };
        let third = CellCoordinate { x: 96, y: 17 };
        let wood_flags = with_fuel_progress(FLAG_COMBUSTING | FLAG_FLAME_EVENT, 438);
        write_probe(
            &simulation,
            first,
            ProbeValues {
                material: MATERIAL_WOOD,
                temperature: 164.25,
                pressure: 83.5,
                flags: wood_flags,
                activity: ACTIVITY_ALL_BITS,
                chunk_state: CHUNK_STATE_RUNNABLE,
            },
        );
        write_probe(
            &simulation,
            second,
            ProbeValues {
                material: MATERIAL_WATER,
                temperature: 72.4,
                pressure: 53.5,
                flags: 0,
                activity: ACTIVITY_MATTER | ACTIVITY_THERMAL | ACTIVITY_PRESSURE,
                chunk_state: CHUNK_STATE_SLEEPING,
            },
        );
        write_probe(
            &simulation,
            third,
            ProbeValues {
                material: MATERIAL_SAND,
                temperature: 18.0,
                pressure: 2.0,
                flags: 0,
                activity: ACTIVITY_MATTER,
                chunk_state: CHUNK_STATE_RUNNABLE,
            },
        );

        let started = Instant::now();
        let mut collector = CellInspectorCollector::new(&simulation);
        assert!(!collector.hud_data(0, started).details_visible);
        assert!(collector.toggle_details());
        assert!(collector.hud_data(0, started).details_visible);
        collector.set_hover(Some(first));
        let initial_pending = collector.hud_data(0, started);
        assert_eq!(
            initial_pending.display_state,
            InspectorDisplayState::Pending
        );
        assert!(initial_pending.details_visible);
        assert!(initial_pending.sample.is_none());
        collector
            .update(&simulation, 0, started)
            .expect("paused tick-0 request");
        assert!(
            collector.pending.is_some(),
            "exactly one request is pending"
        );
        assert_eq!(
            collector.request_readback(&simulation, first, 0, started),
            Err(CellInspectorReadbackError::ReadbackAlreadyPending)
        );
        simulation
            .context
            .device
            .poll(wgpu::PollType::Wait)
            .expect("map completion");
        collector
            .update(&simulation, 0, started + Duration::from_millis(1))
            .expect("map success");
        let sample = collector.latest_sample.as_ref().expect("published sample");
        assert_eq!(sample.cell, first);
        assert_eq!(sample.chunk, CellCoordinate { x: 1, y: 1 });
        assert_eq!(sample.material_id, MATERIAL_WOOD);
        assert_eq!(sample.temperature.to_bits(), 164.25_f32.to_bits());
        assert_eq!(sample.pressure.to_bits(), 83.5_f32.to_bits());
        assert_eq!(sample.flags, wood_flags);
        assert_eq!(sample.cell_activity, ACTIVITY_ALL_BITS);
        assert_eq!(sample.chunk_state, CHUNK_STATE_RUNNABLE);
        assert_eq!((sample.simulation_tick, sample.diagnostic_sequence), (0, 1));
        assert_eq!(
            collector.hud_data(0, started).display_state,
            InspectorDisplayState::Ready
        );

        // A periodic refresh for the same Cell keeps the last matching sample
        // visible until its replacement arrives.
        collector
            .update(&simulation, 1, started + Duration::from_millis(101))
            .expect("same-Cell periodic refresh");
        assert!(collector.pending.is_some());
        assert_eq!(
            collector.hud_data(1, started).display_state,
            InspectorDisplayState::Ready
        );

        // A request may finish after hover changes, but its old identity must
        // never be published for the new Cell.
        collector.set_hover(Some(second));
        assert_eq!(
            collector.hud_data(8, started).display_state,
            InspectorDisplayState::Pending
        );
        assert!(collector.pending.is_some());
        collector.set_hover(Some(third));
        assert_eq!(
            collector.hud_data(8, started).display_state,
            InspectorDisplayState::Pending
        );
        simulation
            .context
            .device
            .poll(wgpu::PollType::Wait)
            .expect("stale map completion");
        collector
            .update(&simulation, 8, started + Duration::from_millis(102))
            .expect("stale completion is discarded");
        assert!(collector.pending.is_none());
        assert!(collector.latest_sample.is_none());
        assert_eq!(
            collector.hud_data(8, started).display_state,
            InspectorDisplayState::Pending
        );
        collector
            .update(&simulation, 8, started + Duration::from_millis(202))
            .expect("fresh request after cadence");
        simulation
            .context
            .device
            .poll(wgpu::PollType::Wait)
            .expect("fresh map completion");
        collector
            .update(&simulation, 8, started + Duration::from_millis(203))
            .expect("fresh completion");
        let sample = collector.latest_sample.as_ref().expect("new Cell sample");
        assert_eq!(sample.cell, third);
        assert_eq!(sample.material_id, MATERIAL_SAND);
        assert_eq!(sample.simulation_tick, 8);
        assert_eq!(sample.diagnostic_sequence, 2);
        assert_eq!(
            collector.hud_data(8, started).display_state,
            InspectorDisplayState::Ready
        );

        // Reset/scenario invalidation cancels a pending map, is idempotent,
        // and permits an immediate request on the same persistent staging
        // buffer once the new world is committed.
        collector.set_hover(Some(first));
        collector
            .update(&simulation, 9, started + Duration::from_millis(303))
            .expect("pre-reset request");
        assert!(collector.pending.is_some());
        collector.begin_world_change();
        collector.begin_world_change();
        assert!(collector.pending.is_none());
        assert!(collector.latest_sample.is_none());
        assert_eq!(
            collector.hud_data(0, started).display_state,
            InspectorDisplayState::Pending
        );
        let reset_pending = collector.hud_data(0, started);
        assert!(reset_pending.details_visible);
        assert!(reset_pending.error_message.is_none());
        collector.mark_ready();
        assert_eq!(
            collector.hud_data(0, started).display_state,
            InspectorDisplayState::Pending
        );
        assert!(collector.hud_data(0, started).details_visible);
        collector
            .update(&simulation, 0, started + Duration::from_millis(303))
            .expect("immediate post-reset request");
        assert!(collector.pending.is_some());
        simulation
            .context
            .device
            .poll(wgpu::PollType::Wait)
            .expect("post-reset map completion");
        collector
            .update(&simulation, 0, started + Duration::from_millis(304))
            .expect("post-reset completion");
        let sample = collector.latest_sample.as_ref().expect("post-reset sample");
        assert_eq!(sample.cell, first);
        assert_eq!(sample.simulation_tick, 0);
        assert_eq!(sample.world_epoch, collector.world_epoch);
        assert_eq!(sample.diagnostic_sequence, 3);
        assert_eq!(
            collector.hud_data(0, started).display_state,
            InspectorDisplayState::Ready
        );

        let injected_failure = CellInspectorReadbackError::MapFailed("injected".to_string());
        collector.record_readback_failure(&injected_failure);
        let failed = collector.hud_data(0, started);
        assert_eq!(failed.display_state, InspectorDisplayState::Failed);
        assert!(failed.details_visible);
        assert!(failed.sample.is_none());
        assert!(failed
            .error_message
            .as_deref()
            .is_some_and(|message| message.contains("Inspector map failed: injected")));

        collector.mark_unavailable("Inspector unavailable: staging failed");
        assert!(collector.latest_sample.is_none());
        assert_eq!(
            collector.hud_data(0, started).display_state,
            InspectorDisplayState::Pending
        );
        assert!(collector.hud_data(0, started).error_message.is_none());
        collector.shutdown();
        collector.shutdown();
        assert!(collector.pending.is_none());
        assert_eq!(
            collector.hud_data(0, started).display_state,
            InspectorDisplayState::Hidden
        );
    }

    #[test]
    fn registry_material_constants_used_by_examples_stay_distinct() {
        let ids = [
            MATERIAL_EMPTY,
            MATERIAL_BOUNDARY_BLOCK,
            MATERIAL_STONE,
            MATERIAL_SAND,
            MATERIAL_WATER,
            MATERIAL_OIL,
            MATERIAL_STEAM,
            MATERIAL_SMOKE,
            MATERIAL_ICE,
            MATERIAL_WOOD,
        ];
        for pair in ids.windows(2) {
            assert_ne!(pair[0], pair[1]);
        }
    }
}
