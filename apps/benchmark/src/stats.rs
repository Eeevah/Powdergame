//! Pure statistical aggregation for G8 benchmark evidence.

use powdergame_gpu::{ProfiledTickReport, PASS_COUNT};

pub const GROUP_COUNT: usize = 6;

pub const GROUP_NAMES: [&str; GROUP_COUNT] = [
    "matter_movement",
    "ownership_claim",
    "thermal_conduction",
    "reaction_phase",
    "pressure_structure",
    "active_sleep_management",
];

pub const GROUP_LABELS: [&str; GROUP_COUNT] = [
    "Matter Movement (propose + commit)",
    "Ownership / Claim (move + expansion + smoke claims)",
    "Thermal Conduction",
    "Reaction & Phase (phase, expansion, decay, combustion)",
    "Pressure & Rupture",
    "Active / Sleep Management (wake, propose, reduce)",
];

/// Computes percentile (0..100) from a sorted f64 slice.
fn percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (pct / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Statistics for a series of numeric measurements.
#[derive(Debug, Clone, PartialEq)]
pub struct StatSummary {
    pub count: usize,
    pub p50: f64,
    pub p95: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
}

impl StatSummary {
    pub fn from_slice(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self {
                count: 0,
                p50: 0.0,
                p95: 0.0,
                mean: 0.0,
                min: 0.0,
                max: 0.0,
            };
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let sum: f64 = sorted.iter().sum();
        Self {
            count: sorted.len(),
            p50: percentile(&sorted, 50.0),
            p95: percentile(&sorted, 95.0),
            mean: sum / sorted.len() as f64,
            min: sorted[0],
            max: sorted[sorted.len() - 1],
        }
    }
}

/// All aggregate timing statistics for one profiled trial.
#[derive(Debug, Clone)]
pub struct ProfiledStatistics {
    pub pass_stats: [StatSummary; PASS_COUNT],
    pub grouped_stats: [StatSummary; GROUP_COUNT],
    /// Per-tick group/envelope percentages. These are percentiles of the
    /// reported ratios, not ratios of independently aggregated percentiles.
    pub grouped_envelope_pct_stats: [StatSummary; GROUP_COUNT],
    pub envelope_stats: StatSummary,
    pub pass_sum_stats: StatSummary,
    pub residual_stats: StatSummary,
}

pub fn grouped_values(report: &ProfiledTickReport) -> [f64; GROUP_COUNT] {
    let grouped = report.grouped_summary();
    [
        grouped.matter_movement_ms,
        grouped.ownership_claim_ms,
        grouped.thermal_ms,
        grouped.reaction_phase_ms,
        grouped.pressure_structure_ms,
        grouped.active_sleep_ms,
    ]
}

/// Aggregates reports by first deriving every per-tick quantity, then taking
/// percentiles of those samples. This ordering is required for mathematically
/// valid grouped subsystem percentiles.
pub fn summarize_profiled_reports(reports: &[ProfiledTickReport]) -> ProfiledStatistics {
    let pass_stats = std::array::from_fn(|pass_index| {
        let values: Vec<f64> = reports
            .iter()
            .map(|report| report.passes[pass_index].duration_ms)
            .collect();
        StatSummary::from_slice(&values)
    });

    let grouped_samples: [Vec<f64>; GROUP_COUNT] = std::array::from_fn(|group_index| {
        reports
            .iter()
            .map(|report| grouped_values(report)[group_index])
            .collect()
    });
    let grouped_stats =
        std::array::from_fn(|group_index| StatSummary::from_slice(&grouped_samples[group_index]));

    let grouped_envelope_pct_stats = std::array::from_fn(|group_index| {
        let values: Vec<f64> = reports
            .iter()
            .map(|report| {
                let grouped = grouped_values(report)[group_index];
                grouped / report.gpu_tick_envelope_ms * 100.0
            })
            .collect();
        StatSummary::from_slice(&values)
    });

    let envelope_values: Vec<f64> = reports
        .iter()
        .map(|report| report.gpu_tick_envelope_ms)
        .collect();
    let pass_sum_values: Vec<f64> = reports
        .iter()
        .map(|report| report.gpu_pass_sum_ms)
        .collect();
    let residual_values: Vec<f64> = reports.iter().map(|report| report.residual_ms).collect();

    ProfiledStatistics {
        pass_stats,
        grouped_stats,
        grouped_envelope_pct_stats,
        envelope_stats: StatSummary::from_slice(&envelope_values),
        pass_sum_stats: StatSummary::from_slice(&pass_sum_values),
        residual_stats: StatSummary::from_slice(&residual_values),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use powdergame_gpu::{PassTiming, ProfiledTickReport, PASS_NAMES};

    fn report_with_movement(tick_index: u64, propose: f64, commit: f64) -> ProfiledTickReport {
        let mut raw = [0u64; PASS_COUNT * 2];
        let mut cursor = 1u64;
        let passes = std::array::from_fn(|index| {
            let duration_ms = match index {
                1 => propose,
                3 => commit,
                _ => 1.0,
            };
            raw[index * 2] = cursor;
            cursor += 1;
            raw[index * 2 + 1] = cursor;
            cursor += 1;
            PassTiming {
                name: PASS_NAMES[index],
                raw_start: raw[index * 2],
                raw_end: raw[index * 2 + 1],
                duration_ns: duration_ms * 1_000_000.0,
                duration_ms,
            }
        });
        let pass_sum: f64 = passes.iter().map(|pass| pass.duration_ms).sum();
        ProfiledTickReport {
            tick_index,
            timestamp_period: 1.0,
            passes,
            raw_timestamps: raw,
            gpu_pass_sum_ms: pass_sum,
            gpu_tick_envelope_ms: pass_sum + 10.0,
            residual_ms: 10.0,
        }
    }

    #[test]
    fn grouped_p50_is_median_of_per_tick_sums() {
        let reports = [
            report_with_movement(0, 0.0, 100.0),
            report_with_movement(1, 1.0, 1.0),
            report_with_movement(2, 100.0, 0.0),
        ];
        let stats = summarize_profiled_reports(&reports);

        let incorrect_sum_of_pass_medians = stats.pass_stats[1].p50 + stats.pass_stats[3].p50;
        assert_eq!(incorrect_sum_of_pass_medians, 2.0);
        assert_eq!(stats.grouped_stats[0].p50, 103.0);
        assert_ne!(stats.grouped_stats[0].p50, incorrect_sum_of_pass_medians);
    }

    #[test]
    fn grouped_samples_partition_each_tick_pass_sum() {
        let report = report_with_movement(0, 4.0, 7.0);
        let grouped_sum: f64 = grouped_values(&report).iter().sum();
        assert!((grouped_sum - report.gpu_pass_sum_ms).abs() < 1.0e-12);
    }
}
