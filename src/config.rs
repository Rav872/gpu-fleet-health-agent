use clap::Parser;
use thiserror::Error;

/// Command-line and environment configuration for the agent process.
#[derive(Clone, Debug, Parser)]
#[command(version, about)]
pub struct Config {
    /// Interval between GPU telemetry collection attempts, in seconds.
    #[arg(
        long,
        env = "GPU_FLEET_COLLECTION_INTERVAL_SECS",
        default_value_t = 30
    )]
    pub collection_interval_secs: u64,

    /// Allow the agent to perform Kubernetes node remediation.
    #[arg(
        long,
        env = "GPU_FLEET_REMEDIATION_ENABLED",
        default_value_t = false,
        action = clap::ArgAction::Set
    )]
    pub remediation_enabled: bool,

    /// Log proposed remediation without changing cluster state.
    #[arg(
        long,
        env = "GPU_FLEET_DRY_RUN",
        default_value_t = true,
        action = clap::ArgAction::Set
    )]
    pub dry_run: bool,
}

impl Config {
    /// Validate configuration values that are safe to check without hardware context.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.collection_interval_secs == 0 {
            return Err(ConfigError::ZeroCollectionInterval);
        }

        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("collection interval must be greater than zero seconds")]
    ZeroCollectionInterval,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Config, ConfigError};

    #[test]
    fn defaults_are_safe() {
        let config = Config::try_parse_from(["gpu-fleet-health-agent"])
            .expect("default configuration should parse");

        assert_eq!(config.collection_interval_secs, 30);
        assert!(!config.remediation_enabled);
        assert!(config.dry_run);
        assert_eq!(config.validate(), Ok(()));
    }

    #[test]
    fn zero_collection_interval_is_rejected() {
        let config = Config::try_parse_from([
            "gpu-fleet-health-agent",
            "--collection-interval-secs",
            "0",
        ])
        .expect("zero is syntactically a valid integer");

        assert_eq!(config.validate(), Err(ConfigError::ZeroCollectionInterval));
    }

    #[test]
    fn remediation_options_can_be_explicitly_configured() {
        let config = Config::try_parse_from([
            "gpu-fleet-health-agent",
            "--remediation-enabled",
            "true",
            "--dry-run",
            "false",
        ])
        .expect("boolean options should accept explicit values");

        assert!(config.remediation_enabled);
        assert!(!config.dry_run);
    }
}
