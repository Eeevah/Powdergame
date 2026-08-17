//! G8-A GPU timestamp profiling substrate.
//!
//! Provides observational timestamp profiling for all 17 simulation passes
//! using `wgpu::Features::TIMESTAMP_QUERY` and per-compute-pass `timestamp_writes`.
//!
//! Architectural rules (see `docs/specs/SIMULATION_SPEC.md`):
//! - Profiling observes the exact production pipeline; it never redesigns or perturbs it.
//! - Raw pass timings are primary and authoritative.
//! - Secondary grouped subsystem summaries provide convenient roll-ups without double-counting.
//! - The GPU tick envelope measures from the beginning of `activity_wake` to the end of `activity_reduce`.

use crate::context::{GpuContext, GpuError};

/// Total number of distinct compute passes in a single simulation tick.
pub const PASS_COUNT: usize = 17;

/// Total number of timestamp queries per tick (start + end per pass).
pub const QUERY_COUNT: u32 = 34;

/// Canonical names of all 17 simulation passes in exact execution order.
pub const PASS_NAMES: [&str; 17] = [
    "activity_wake",
    "movement_propose",
    "movement_claim",
    "movement_commit",
    "thermal",
    "phase_transition",
    "expansion_claim",
    "expansion_spawn_commit",
    "expansion_pressure",
    "decay",
    "combustion",
    "smoke_claim",
    "smoke_commit",
    "pressure",
    "rupture",
    "activity_propose",
    "activity_reduce",
];

/// Measured timing for a single compute pass.
#[derive(Debug, Clone, PartialEq)]
pub struct PassTiming {
    pub name: &'static str,
    pub raw_start: u64,
    pub raw_end: u64,
    pub duration_ns: f64,
    pub duration_ms: f64,
}

/// Secondary grouped subsystem roll-up (derived from raw pass timings without double counting).
#[derive(Debug, Clone, PartialEq)]
pub struct GroupedSubsystemSummary {
    /// `movement_propose` + `movement_commit`
    pub matter_movement_ms: f64,
    /// `movement_claim` + `expansion_claim` + `smoke_claim`
    pub ownership_claim_ms: f64,
    /// `thermal`
    pub thermal_ms: f64,
    /// `phase_transition` + `expansion_spawn_commit` + `expansion_pressure` + `decay` + `combustion` + `smoke_commit`
    pub reaction_phase_ms: f64,
    /// `pressure` + `rupture`
    pub pressure_structure_ms: f64,
    /// `activity_wake` + `activity_propose` + `activity_reduce`
    pub active_sleep_ms: f64,
}

/// Complete profiling report for a single simulation tick.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfiledTickReport {
    pub tick_index: u64,
    pub timestamp_period: f32,
    pub passes: [PassTiming; 17],
    pub raw_timestamps: [u64; 34],
    /// Sum of all 17 individual pass durations.
    pub gpu_pass_sum_ms: f64,
    /// Total tick envelope duration: from start of activity_wake (query 0) to end of activity_reduce (query 33).
    pub gpu_tick_envelope_ms: f64,
    /// Diagnostic residual (`envelope - pass_sum`). Note: this is a diagnostic residual, not strict additive copy cost.
    pub residual_ms: f64,
}

impl ProfiledTickReport {
    /// Computes the secondary non-overlapping subsystem group summaries.
    pub fn grouped_summary(&self) -> GroupedSubsystemSummary {
        GroupedSubsystemSummary {
            matter_movement_ms: self.passes[1].duration_ms + self.passes[3].duration_ms,
            ownership_claim_ms: self.passes[2].duration_ms
                + self.passes[6].duration_ms
                + self.passes[11].duration_ms,
            thermal_ms: self.passes[4].duration_ms,
            reaction_phase_ms: self.passes[5].duration_ms
                + self.passes[7].duration_ms
                + self.passes[8].duration_ms
                + self.passes[9].duration_ms
                + self.passes[10].duration_ms
                + self.passes[12].duration_ms,
            pressure_structure_ms: self.passes[13].duration_ms + self.passes[14].duration_ms,
            active_sleep_ms: self.passes[0].duration_ms
                + self.passes[15].duration_ms
                + self.passes[16].duration_ms,
        }
    }
}

/// GPU-side profiler resources (QuerySet, resolve buffer, readback buffer).
pub struct GpuProfiler {
    pub query_set: wgpu::QuerySet,
    pub resolve_buffer: wgpu::Buffer,
    pub readback_buffer: wgpu::Buffer,
}

impl GpuProfiler {
    /// Creates a new profiler allocating the required QuerySet and resolve/readback buffers.
    pub fn new(context: &GpuContext) -> Result<Self, GpuError> {
        if !context.profiling_enabled {
            return Err(GpuError::Other(
                "cannot allocate GpuProfiler on GpuContext without profiling enabled".into(),
            ));
        }

        let query_set = context.device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("powdergame/profiler/query-set"),
            count: QUERY_COUNT,
            ty: wgpu::QueryType::Timestamp,
        });

        let byte_size = (QUERY_COUNT as u64) * 8;
        let resolve_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("powdergame/profiler/resolve-buffer"),
            size: byte_size,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("powdergame/profiler/readback-buffer"),
            size: byte_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Ok(Self {
            query_set,
            resolve_buffer,
            readback_buffer,
        })
    }

    /// Returns the exact application-tracked GPU buffer allocation bytes for profiler resources.
    pub fn tracked_gpu_allocation_bytes(&self) -> u64 {
        (QUERY_COUNT as u64) * 8 * 2
    }

    /// Reads back timestamp query results from GPU, maps the staging buffer, and constructs a ProfiledTickReport.
    pub fn readback_report(
        &self,
        device: &wgpu::Device,
        tick_index: u64,
        timestamp_period: f32,
    ) -> Result<ProfiledTickReport, GpuError> {
        let slice = self.readback_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        let _ = device.poll(wgpu::PollType::Wait);
        rx.recv()
            .map_err(|e| GpuError::ReadbackFailed(format!("profiler map callback lost: {e}")))?
            .map_err(|e| GpuError::ReadbackFailed(e.to_string()))?;

        let mapped = slice.get_mapped_range();
        let mut raw = [0u64; 34];
        for i in 0..34 {
            raw[i] = u64::from_ne_bytes(mapped[i * 8..(i + 1) * 8].try_into().unwrap());
        }
        drop(mapped);
        self.readback_buffer.unmap();

        report_from_raw_timestamps(tick_index, timestamp_period, raw)
    }
}

/// Validates one resolved timestamp set before deriving durations.
///
/// Timestamp zero is valid: the only required invariants are a positive finite
/// period, positive duration for every pass, production-order monotonicity, and
/// an envelope large enough to contain the non-overlapping pass intervals.
fn report_from_raw_timestamps(
    tick_index: u64,
    timestamp_period: f32,
    raw: [u64; 34],
) -> Result<ProfiledTickReport, GpuError> {
    if !timestamp_period.is_finite() || timestamp_period <= 0.0 {
        return Err(GpuError::ReadbackFailed(format!(
            "invalid profiler timestamp period {timestamp_period:?}; expected a finite positive value"
        )));
    }

    let mut pass_ticks = [0u64; PASS_COUNT];
    for i in 0..PASS_COUNT {
        let start_query = i * 2;
        let end_query = start_query + 1;
        let raw_start = raw[start_query];
        let raw_end = raw[end_query];

        if raw_end <= raw_start {
            return Err(GpuError::ReadbackFailed(format!(
                "invalid profiler timestamp data: pass={}, start_query={} raw_start={}, end_query={} raw_end={}; expected end > start",
                PASS_NAMES[i], start_query, raw_start, end_query, raw_end
            )));
        }

        if i > 0 {
            let previous_end_query = start_query - 1;
            let previous_end = raw[previous_end_query];
            if raw_start < previous_end {
                return Err(GpuError::ReadbackFailed(format!(
                    "invalid profiler timestamp order: previous_pass={} end_query={} raw_end={}, current_pass={} start_query={} raw_start={}; expected current start >= previous end",
                    PASS_NAMES[i - 1],
                    previous_end_query,
                    previous_end,
                    PASS_NAMES[i],
                    start_query,
                    raw_start
                )));
            }
        }

        pass_ticks[i] = raw_end - raw_start;
    }

    let envelope_ticks = raw[33].checked_sub(raw[0]).ok_or_else(|| {
        GpuError::ReadbackFailed(format!(
            "invalid profiler envelope: start_query=0 raw_start={}, end_query=33 raw_end={}; expected end > start",
            raw[0], raw[33]
        ))
    })?;
    if envelope_ticks == 0 {
        return Err(GpuError::ReadbackFailed(format!(
            "invalid profiler envelope: start_query=0 raw_start={}, end_query=33 raw_end={}; expected end > start",
            raw[0], raw[33]
        )));
    }

    let pass_sum_ticks = pass_ticks.iter().try_fold(0u64, |sum, &ticks| {
        sum.checked_add(ticks).ok_or_else(|| {
            GpuError::ReadbackFailed("profiler pass timestamp delta sum overflowed u64".into())
        })
    })?;
    if pass_sum_ticks > envelope_ticks {
        return Err(GpuError::ReadbackFailed(format!(
            "invalid profiler timestamp accounting: pass_sum_ticks={pass_sum_ticks} exceeds envelope_ticks={envelope_ticks}"
        )));
    }
    let residual_ticks = envelope_ticks - pass_sum_ticks;

    let period_ns = timestamp_period as f64;
    let ticks_to_ns = |ticks: u64| (ticks as f64) * period_ns;
    let passes = std::array::from_fn(|i| {
        let duration_ns = ticks_to_ns(pass_ticks[i]);
        PassTiming {
            name: PASS_NAMES[i],
            raw_start: raw[i * 2],
            raw_end: raw[i * 2 + 1],
            duration_ns,
            duration_ms: duration_ns / 1_000_000.0,
        }
    });
    let gpu_pass_sum_ms = ticks_to_ns(pass_sum_ticks) / 1_000_000.0;
    let gpu_tick_envelope_ms = ticks_to_ns(envelope_ticks) / 1_000_000.0;
    let residual_ms = ticks_to_ns(residual_ticks) / 1_000_000.0;

    if !passes
        .iter()
        .all(|pass| pass.duration_ns.is_finite() && pass.duration_ms.is_finite())
        || !gpu_pass_sum_ms.is_finite()
        || !gpu_tick_envelope_ms.is_finite()
        || !residual_ms.is_finite()
    {
        return Err(GpuError::ReadbackFailed(
            "profiler timestamp conversion produced a non-finite duration".into(),
        ));
    }

    Ok(ProfiledTickReport {
        tick_index,
        timestamp_period,
        passes,
        raw_timestamps: raw,
        gpu_pass_sum_ms,
        gpu_tick_envelope_ms,
        residual_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_raw_timestamps() -> [u64; 34] {
        let mut raw = [0u64; 34];
        let mut cursor = 0u64;
        for i in 0..PASS_COUNT {
            raw[i * 2] = cursor;
            cursor += (i as u64) + 1;
            raw[i * 2 + 1] = cursor;
            cursor += 3;
        }
        raw
    }

    #[test]
    fn raw_timestamp_validation_accepts_zero_epoch_and_preserves_groups() {
        let report = report_from_raw_timestamps(7, 1.0, valid_raw_timestamps()).unwrap();
        assert_eq!(report.raw_timestamps[0], 0);
        assert_eq!(report.tick_index, 7);

        let grouped = report.grouped_summary();
        let grouped_sum = grouped.matter_movement_ms
            + grouped.ownership_claim_ms
            + grouped.thermal_ms
            + grouped.reaction_phase_ms
            + grouped.pressure_structure_ms
            + grouped.active_sleep_ms;
        assert!((grouped_sum - report.gpu_pass_sum_ms).abs() < 1.0e-12);
    }

    #[test]
    fn raw_timestamp_validation_rejects_equal_or_inverted_pass_bounds() {
        let mut equal = valid_raw_timestamps();
        equal[5] = equal[4];
        let equal_error = report_from_raw_timestamps(0, 1.0, equal)
            .unwrap_err()
            .to_string();
        assert!(equal_error.contains("pass=movement_claim"));
        assert!(equal_error.contains("end > start"));

        let mut inverted = valid_raw_timestamps();
        inverted[7] = inverted[6] - 1;
        let inverted_error = report_from_raw_timestamps(0, 1.0, inverted)
            .unwrap_err()
            .to_string();
        assert!(inverted_error.contains("pass=movement_commit"));
    }

    #[test]
    fn raw_timestamp_validation_rejects_cross_pass_reordering() {
        let mut raw = valid_raw_timestamps();
        raw[4] = raw[3] - 1;
        raw[5] = raw[3] + 1;
        let error = report_from_raw_timestamps(0, 1.0, raw)
            .unwrap_err()
            .to_string();
        assert!(error.contains("previous_pass=movement_propose"));
        assert!(error.contains("current_pass=movement_claim"));
    }

    #[test]
    fn raw_timestamp_validation_rejects_invalid_periods() {
        for period in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            let error = report_from_raw_timestamps(0, period, valid_raw_timestamps())
                .unwrap_err()
                .to_string();
            assert!(error.contains("timestamp period"));
        }
    }
}
