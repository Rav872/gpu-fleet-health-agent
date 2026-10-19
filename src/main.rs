use std::time::Duration;

use clap::Parser;
use gpu_fleet_health_agent::config::Config;
use tokio::signal;
use tokio::time;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    let config = Config::parse();
    config.validate()?;

    info!(
        collection_interval_secs = config.collection_interval_secs,
        remediation_enabled = config.remediation_enabled,
        dry_run = config.dry_run,
        "GPU Fleet Health Agent starting"
    );
    warn!("GPU telemetry and Kubernetes remediation providers are not configured yet");

    let mut interval = time::interval(Duration::from_secs(config.collection_interval_secs));
    interval.tick().await;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                info!("agent heartbeat; telemetry collection is not implemented yet");
            }
            result = signal::ctrl_c() => {
                result?;
                info!("shutdown signal received");
                break;
            }
        }
    }

    info!("GPU Fleet Health Agent stopped");
    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
