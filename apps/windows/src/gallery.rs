//! G8-B Windows Gallery state and out-of-band diagnostic presentation data.
//!
//! Nothing in this module advances or mutates simulation physics. The Gallery
//! samples the already-produced activity buffers at a bounded cadence and
//! labels every sample with the simulation tick that produced it.

use powdergame_gpu::ActivityCensusReport;
use powdergame_scenarios::{ScenarioId, GALLERY_SCENARIOS};

use crate::inspector::{InspectorHudData, ScreenRect};

pub const GALLERY_DIAGNOSTIC_INTERVAL_TICKS: u64 = 30;
pub const GALLERY_CONTROLS: &str =
    "1-6 Scenario | SPACE Play/Pause | N One Tick | R Reset | F x1/x4/x16 | I Inspector details ON/OFF | ESC Quit";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitState {
    Clean,
    Dirty,
    Unavailable,
}

impl GitState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Dirty => "dirty",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeProvenance {
    pub source_sha: String,
    pub git_state: GitState,
    pub build_profile: &'static str,
}

impl RuntimeProvenance {
    /// Returns provenance embedded by `build.rs`. These values describe the
    /// source that produced this executable and cannot drift when the same
    /// EXE is later launched from a different or newer checkout.
    pub fn from_build() -> Self {
        Self {
            source_sha: env!("POWDERGAME_BUILD_SOURCE_SHA").to_string(),
            git_state: parse_git_state(env!("POWDERGAME_BUILD_GIT_STATE")),
            build_profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
        }
    }
}

fn parse_git_state(value: &str) -> GitState {
    match value {
        "clean" => GitState::Clean,
        "dirty" => GitState::Dirty,
        _ => GitState::Unavailable,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GalleryDiagnosticSample {
    pub sequence: u64,
    pub source_tick: u64,
    pub census: ActivityCensusReport,
}

/// Transaction state for a Gallery reset. `scenario` and the last diagnostic
/// sample remain committed to the previous successfully staged world until a
/// pending reset completes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GalleryTransition {
    Ready,
    Pending {
        requested: ScenarioId,
    },
    Failed {
        requested: ScenarioId,
        message: String,
    },
}

#[derive(Clone, Debug)]
pub struct GalleryState {
    scenario: ScenarioId,
    sample_sequence: u64,
    next_sample_tick: u64,
    diagnostic_sample: Option<GalleryDiagnosticSample>,
    transition: GalleryTransition,
}

impl GalleryState {
    pub fn new() -> Self {
        Self {
            scenario: GALLERY_SCENARIOS[0],
            sample_sequence: 0,
            next_sample_tick: 0,
            diagnostic_sample: None,
            transition: GalleryTransition::Ready,
        }
    }

    pub const fn scenario(&self) -> ScenarioId {
        self.scenario
    }

    pub fn scenario_number(&self) -> usize {
        GALLERY_SCENARIOS
            .iter()
            .position(|candidate| *candidate == self.scenario)
            .map_or(1, |index| index + 1)
    }

    /// Requests a one-based Gallery slot without changing the committed world
    /// attribution. The caller commits it only after shared staging succeeds.
    pub fn request_number(&mut self, number: u8) -> Option<ScenarioId> {
        let scenario = number
            .checked_sub(1)
            .and_then(|index| GALLERY_SCENARIOS.get(index as usize))
            .copied()?;
        self.transition = GalleryTransition::Pending {
            requested: scenario,
        };
        Some(scenario)
    }

    pub fn request_current_reset(&mut self) -> ScenarioId {
        let requested = self.scenario;
        self.transition = GalleryTransition::Pending { requested };
        requested
    }

    pub fn reset_target(&self) -> Option<ScenarioId> {
        match &self.transition {
            GalleryTransition::Pending { requested } => Some(*requested),
            GalleryTransition::Ready | GalleryTransition::Failed { .. } => None,
        }
    }

    pub fn commit_reset_success(&mut self) -> Option<ScenarioId> {
        let requested = self.reset_target()?;
        self.scenario = requested;
        self.transition = GalleryTransition::Ready;
        self.reset_diagnostics();
        Some(requested)
    }

    pub fn commit_reset_failure(&mut self, message: String) -> ScenarioId {
        let requested = self.reset_target().unwrap_or(self.scenario);
        self.transition = GalleryTransition::Failed { requested, message };
        requested
    }

    pub const fn transition(&self) -> &GalleryTransition {
        &self.transition
    }

    pub const fn is_ready(&self) -> bool {
        matches!(&self.transition, GalleryTransition::Ready)
    }

    pub fn reset_diagnostics(&mut self) {
        self.sample_sequence = 0;
        self.next_sample_tick = 0;
        self.diagnostic_sample = None;
    }

    pub fn should_sample(&self, simulation_tick: u64) -> bool {
        self.is_ready() && simulation_tick >= self.next_sample_tick
    }

    pub fn record_sample(&mut self, source_tick: u64, census: ActivityCensusReport) {
        if !self.is_ready() {
            return;
        }
        self.sample_sequence += 1;
        self.next_sample_tick = source_tick.saturating_add(GALLERY_DIAGNOSTIC_INTERVAL_TICKS);
        self.diagnostic_sample = Some(GalleryDiagnosticSample {
            sequence: self.sample_sequence,
            source_tick,
            census,
        });
    }

    pub fn defer_failed_sample(&mut self, source_tick: u64) {
        self.next_sample_tick = source_tick.saturating_add(GALLERY_DIAGNOSTIC_INTERVAL_TICKS);
    }

    pub fn diagnostic_sample(&self) -> Option<&GalleryDiagnosticSample> {
        self.diagnostic_sample.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct GalleryHudData {
    pub source_sha: String,
    pub git_state: &'static str,
    pub build_profile: &'static str,
    pub scenario_number: usize,
    pub scenario_name: &'static str,
    pub scenario_description: &'static str,
    pub world_width: u32,
    pub world_height: u32,
    pub chunk_size: u32,
    pub sleep_enabled: bool,
    pub sleep_threshold: u32,
    pub playing: bool,
    pub fast: u32,
    pub simulation_tick: Option<u64>,
    pub diagnostic_sample: Option<GalleryDiagnosticSample>,
    pub transition: GalleryTransition,
    pub inspector: Option<InspectorHudData>,
    pub inspector_cursor: Option<[f32; 2]>,
    pub world_viewport: Option<ScreenRect>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report() -> ActivityCensusReport {
        ActivityCensusReport {
            total_cells: 64,
            any_active_cells: 7,
            matter_active_cells: 3,
            thermal_active_cells: 2,
            pressure_active_cells: 1,
            reaction_active_cells: 1,
            total_chunks: 4,
            active_chunks: 2,
            runnable_chunks: 3,
            sleeping_chunks: 1,
        }
    }

    #[test]
    fn one_based_selection_commits_only_after_reset_success() {
        let mut state = GalleryState::new();
        for (index, scenario) in GALLERY_SCENARIOS.iter().copied().enumerate() {
            state.record_sample(99, sample_report());
            let previously_committed = state.scenario();
            assert_eq!(state.request_number((index + 1) as u8), Some(scenario));
            assert_eq!(state.scenario(), previously_committed);
            assert!(state.diagnostic_sample().is_some());
            assert!(!state.should_sample(1000));
            assert_eq!(state.reset_target(), Some(scenario));
            assert_eq!(state.commit_reset_success(), Some(scenario));
            assert_eq!(state.scenario(), scenario);
            assert_eq!(state.scenario_number(), index + 1);
            assert!(state.diagnostic_sample().is_none());
            assert!(state.should_sample(0));
        }
        assert_eq!(state.request_number(0), None);
        assert_eq!(state.request_number(7), None);
    }

    #[test]
    fn diagnostic_census_is_bounded_and_keeps_source_tick_separate() {
        let mut state = GalleryState::new();
        assert!(state.should_sample(0));
        state.record_sample(4, sample_report());
        assert!(!state.should_sample(33));
        assert!(state.should_sample(34));
        let sample = state.diagnostic_sample().unwrap();
        assert_eq!(sample.sequence, 1);
        assert_eq!(sample.source_tick, 4);
    }

    #[test]
    fn failed_diagnostic_attempt_is_also_rate_limited() {
        let mut state = GalleryState::new();
        state.defer_failed_sample(8);
        assert!(!state.should_sample(37));
        assert!(state.should_sample(38));
        assert!(state.diagnostic_sample().is_none());
    }

    #[test]
    fn failed_reset_keeps_committed_attribution_and_suppresses_sampling() {
        let mut state = GalleryState::new();
        state.record_sample(12, sample_report());
        let committed = state.scenario();
        assert_eq!(state.request_number(6), Some(ScenarioId::ActiveSleepG7));
        assert_eq!(
            state.commit_reset_failure("injected staging failure".to_string()),
            ScenarioId::ActiveSleepG7
        );
        assert_eq!(state.scenario(), committed);
        assert_eq!(state.diagnostic_sample().unwrap().source_tick, 12);
        assert!(!state.should_sample(10_000));
        assert!(matches!(
            state.transition(),
            GalleryTransition::Failed {
                requested: ScenarioId::ActiveSleepG7,
                ..
            }
        ));
    }

    #[test]
    fn build_git_state_uses_only_embedded_value_vocabulary() {
        assert_eq!(parse_git_state("clean"), GitState::Clean);
        assert_eq!(parse_git_state("dirty"), GitState::Dirty);
        assert_eq!(parse_git_state("unavailable"), GitState::Unavailable);
        assert_eq!(
            parse_git_state("later-checkout-value"),
            GitState::Unavailable
        );
    }

    #[test]
    fn gallery_controls_advertise_the_inspector_without_changing_existing_keys() {
        for control in [
            "1-6 Scenario",
            "SPACE Play/Pause",
            "N One Tick",
            "R Reset",
            "F x1/x4/x16",
            "I Inspector details ON/OFF",
            "ESC Quit",
        ] {
            assert!(GALLERY_CONTROLS.contains(control), "missing {control}");
        }
        assert!(GALLERY_CONTROLS.is_ascii());
    }
}
