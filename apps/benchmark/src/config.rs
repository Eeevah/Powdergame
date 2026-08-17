//! Strict, testable command-line configuration for the G8 harness.

use std::path::PathBuf;
use std::str::FromStr;

use powdergame_core::WorldConfig;
use powdergame_scenarios::{validate_scenario_config, ScenarioId};

pub const G8A_EVIDENCE_SCHEMA_VERSION: &str = "powdergame-g8a-v5";
pub const G8B_EVIDENCE_SCHEMA_VERSION: &str = "powdergame-g8b-fixture-v1";

const SCENARIO_CHOICES: &str =
    "calibration|sand-fall|water-flow|fire-heat|pressure-burst|heavy-mixed-world|active-sleep-g7";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BenchmarkScenario {
    #[default]
    Calibration,
    Shared(ScenarioId),
}

impl BenchmarkScenario {
    pub const fn number(self) -> Option<u8> {
        match self {
            Self::Calibration => None,
            Self::Shared(scenario) => Some(scenario.number()),
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Calibration => "calibration",
            Self::Shared(scenario) => scenario.slug(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Calibration => "G8-A calibration",
            Self::Shared(scenario) => scenario.name(),
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Calibration => "legacy mixed calibration fixture",
            Self::Shared(scenario) => scenario.description(),
        }
    }

    pub const fn is_calibration(self) -> bool {
        matches!(self, Self::Calibration)
    }

    pub const fn evidence_schema_version(self) -> &'static str {
        match self {
            Self::Calibration => G8A_EVIDENCE_SCHEMA_VERSION,
            Self::Shared(_) => G8B_EVIDENCE_SCHEMA_VERSION,
        }
    }

    pub fn run_id_prefix(self) -> String {
        match self {
            Self::Calibration => "g8a".into(),
            Self::Shared(scenario) => format!("g8b-{}", scenario.slug()),
        }
    }
}

impl FromStr for BenchmarkScenario {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "calibration" {
            return Ok(Self::Calibration);
        }
        value.parse::<ScenarioId>().map(Self::Shared).map_err(|_| {
            format!("invalid value for --scenario: {value}; expected {SCENARIO_CHOICES}")
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkCliConfig {
    pub scenario: BenchmarkScenario,
    pub width: u32,
    pub height: u32,
    pub chunk_size: u32,
    pub sleep_enabled: bool,
    pub sleep_threshold: u32,
    pub prewarm_secs: f64,
    pub throughput_ticks: u32,
    pub profile_ticks: u32,
    pub overhead_ticks: u32,
    pub trials: u32,
    pub csv_output: PathBuf,
}

impl Default for BenchmarkCliConfig {
    fn default() -> Self {
        Self {
            scenario: BenchmarkScenario::Calibration,
            width: 2048,
            height: 2048,
            chunk_size: 64,
            sleep_enabled: true,
            sleep_threshold: 16,
            prewarm_secs: 2.0,
            throughput_ticks: 1024,
            profile_ticks: 256,
            overhead_ticks: 256,
            trials: 3,
            csv_output: PathBuf::from("target/calibration_report.csv"),
        }
    }
}

impl BenchmarkCliConfig {
    pub fn world_config(&self) -> Result<WorldConfig, String> {
        WorldConfig::new(self.width, self.height, self.chunk_size)
            .map_err(|error| format!("invalid WorldConfig: {error}"))
    }

    pub fn validate(&self) -> Result<(), String> {
        let world = self.world_config()?;
        if let BenchmarkScenario::Shared(scenario) = self.scenario {
            validate_scenario_config(scenario, &world).map_err(|error| error.to_string())?;
        }
        if !self.prewarm_secs.is_finite() || self.prewarm_secs < 0.0 {
            return Err(format!(
                "--prewarm-secs must be a finite non-negative number, got {}",
                self.prewarm_secs
            ));
        }
        for (name, value) in [
            ("--throughput-ticks", self.throughput_ticks),
            ("--profile-ticks", self.profile_ticks),
            ("--overhead-ticks", self.overhead_ticks),
            ("--trials", self.trials),
        ] {
            if value == 0 {
                return Err(format!("{name} must be greater than zero"));
            }
        }
        Ok(())
    }
}

fn parse_number<T: std::str::FromStr>(name: &str, value: Option<String>) -> Result<T, String> {
    let value = value.ok_or_else(|| format!("missing value for {name}"))?;
    value
        .parse::<T>()
        .map_err(|_| format!("invalid value for {name}: {value}"))
}

pub fn parse_cli_args() -> Result<BenchmarkCliConfig, String> {
    parse_cli_args_from(std::env::args().skip(1))
}

pub fn parse_cli_args_from<I, S>(args: I) -> Result<BenchmarkCliConfig, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut config = BenchmarkCliConfig::default();
    let mut csv_output_was_explicit = false;
    let mut args = args.into_iter().map(Into::into);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--scenario" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --scenario".to_string())?;
                config.scenario = value.parse()?;
            }
            "--width" => config.width = parse_number("--width", args.next())?,
            "--height" => config.height = parse_number("--height", args.next())?,
            "--chunk" => config.chunk_size = parse_number("--chunk", args.next())?,
            "--sleep" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --sleep".to_string())?;
                config.sleep_enabled = match value.to_ascii_lowercase().as_str() {
                    "on" | "true" => true,
                    "off" | "false" => false,
                    _ => {
                        return Err(format!(
                            "invalid value for --sleep: {value}; expected on/off or true/false"
                        ));
                    }
                };
            }
            "--threshold" => config.sleep_threshold = parse_number("--threshold", args.next())?,
            "--prewarm-secs" => config.prewarm_secs = parse_number("--prewarm-secs", args.next())?,
            "--throughput-ticks" => {
                config.throughput_ticks = parse_number("--throughput-ticks", args.next())?
            }
            "--profile-ticks" => {
                config.profile_ticks = parse_number("--profile-ticks", args.next())?
            }
            "--overhead-ticks" => {
                config.overhead_ticks = parse_number("--overhead-ticks", args.next())?
            }
            "--trials" => config.trials = parse_number("--trials", args.next())?,
            "--csv" => {
                config.csv_output = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "missing value for --csv".to_string())?,
                );
                csv_output_was_explicit = true;
            }
            _ => return Err(format!("unknown benchmark argument: {argument}")),
        }
    }
    if !csv_output_was_explicit {
        if let BenchmarkScenario::Shared(scenario) = config.scenario {
            config.csv_output = PathBuf::from(format!("target/{}_report.csv", scenario.slug()));
        }
    }
    config.validate()?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = parse_cli_args_from(Vec::<String>::new()).unwrap();
        assert_eq!(config, BenchmarkCliConfig::default());
        assert_eq!(config.scenario, BenchmarkScenario::Calibration);
        assert_eq!(
            config.csv_output,
            PathBuf::from("target/calibration_report.csv")
        );
        assert_eq!(
            config.scenario.evidence_schema_version(),
            "powdergame-g8a-v5"
        );
        assert_eq!(config.scenario.run_id_prefix(), "g8a");
    }

    #[test]
    fn accepts_every_documented_scenario_and_assigns_conditional_identity() {
        for slug in [
            "calibration",
            "sand-fall",
            "water-flow",
            "fire-heat",
            "pressure-burst",
            "heavy-mixed-world",
        ] {
            let config = parse_cli_args_from(["--scenario", slug]).unwrap();
            assert_eq!(config.scenario.slug(), slug);
        }

        let shared = parse_cli_args_from(["--scenario", "sand-fall"]).unwrap();
        assert_eq!(
            shared.scenario.evidence_schema_version(),
            "powdergame-g8b-fixture-v1"
        );
        assert_eq!(shared.scenario.run_id_prefix(), "g8b-sand-fall");
    }

    #[test]
    fn shared_scenarios_get_distinct_slug_scoped_default_csv_paths() {
        for slug in [
            "sand-fall",
            "water-flow",
            "fire-heat",
            "pressure-burst",
            "heavy-mixed-world",
        ] {
            let config = parse_cli_args_from(["--scenario", slug]).unwrap();
            assert_eq!(
                config.csv_output,
                PathBuf::from(format!("target/{slug}_report.csv"))
            );
        }

        let active_sleep = parse_cli_args_from([
            "--scenario",
            "active-sleep-g7",
            "--width",
            "256",
            "--height",
            "256",
            "--chunk",
            "64",
        ])
        .unwrap();
        assert_eq!(
            active_sleep.csv_output,
            PathBuf::from("target/active-sleep-g7_report.csv")
        );
    }

    #[test]
    fn explicit_csv_wins_regardless_of_argument_order() {
        for args in [
            vec!["--scenario", "sand-fall", "--csv", "target/custom.csv"],
            vec!["--csv", "target/custom.csv", "--scenario", "sand-fall"],
        ] {
            let config = parse_cli_args_from(args).unwrap();
            assert_eq!(config.csv_output, PathBuf::from("target/custom.csv"));
        }
    }

    #[test]
    fn active_sleep_g7_requires_its_exact_world_config() {
        assert!(parse_cli_args_from([
            "--scenario",
            "active-sleep-g7",
            "--width",
            "256",
            "--height",
            "256",
            "--chunk",
            "64",
        ])
        .is_ok());

        let error = parse_cli_args_from(["--scenario", "active-sleep-g7"]).unwrap_err();
        assert!(error.contains("256x256"));
        assert!(error.contains("chunk size 64"));
    }

    #[test]
    fn rejects_zero_and_malformed_values_before_gpu_initialization() {
        for args in [
            vec!["--chunk", "0"],
            vec!["--trials", "0"],
            vec!["--profile-ticks", "wat"],
            vec!["--prewarm-secs", "NaN"],
        ] {
            assert!(parse_cli_args_from(args).is_err());
        }
    }

    #[test]
    fn rejects_missing_unknown_and_invalid_boolean_arguments() {
        assert!(parse_cli_args_from(["--width"]).is_err());
        assert!(parse_cli_args_from(["--unknown", "1"]).is_err());
        assert!(parse_cli_args_from(["--sleep", "maybe"]).is_err());
        assert!(parse_cli_args_from(["--scenario", "not-a-scenario"]).is_err());
    }

    #[test]
    fn accepts_non_divisible_valid_world_dimensions() {
        let config = parse_cli_args_from([
            "--width", "257", "--height", "321", "--chunk", "64", "--sleep", "off",
        ])
        .unwrap();
        assert_eq!(config.world_config().unwrap().width, 257);
        assert!(!config.sleep_enabled);
    }
}
