//! Strict, testable command-line configuration for the G8 harness.

use std::path::PathBuf;

use powdergame_core::WorldConfig;

#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkCliConfig {
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
        self.world_config()?;
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
    let mut args = args.into_iter().map(Into::into);
    while let Some(argument) = args.next() {
        match argument.as_str() {
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
                )
            }
            _ => return Err(format!("unknown benchmark argument: {argument}")),
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
