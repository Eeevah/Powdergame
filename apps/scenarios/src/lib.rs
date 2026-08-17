//! Shared deterministic scenario construction and GPU staging.
//!
//! This crate contains authored fixture data only. It does not alter the
//! production simulation pass graph or physics rules.

mod fixture;
mod stage;

pub use fixture::{
    validate_scenario_config, ScenarioError, ScenarioFixture, ScenarioId, GALLERY_SCENARIOS,
    OFFICIAL_G8B_SCENARIOS,
};
pub use stage::{reset_and_stage_scenario, stage_scenario};
