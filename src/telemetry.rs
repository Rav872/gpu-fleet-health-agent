use async_trait::async_trait;
use thiserror::Error;

/// Stable identifier for a GPU on the current node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuId {
    pub index: u32,
    pub uuid: String,
}

/// One sample of GPU properties and counters available from the telemetry provider.
#[derive(Clone, Debug, PartialEq)]
pub struct GpuTelemetry {
    pub gpu: GpuId,
    pub temperature_celsius: Option<f64>,
    pub ecc_correctable_errors: Option<u64>,
    pub ecc_uncorrectable_errors: Option<u64>,
    pub pcie_link_generation: Option<u32>,
    pub pcie_link_width: Option<u32>,
    pub thermal_throttling: Option<bool>,
}

/// Collection result for a single polling cycle. Empty is distinct from failure.
#[derive(Clone, Debug, PartialEq)]
pub struct TelemetrySnapshot {
    pub gpus: Vec<GpuTelemetry>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TelemetryError {
    #[error("telemetry provider is not initialized")]
    NotInitialized,
    #[error("telemetry collection failed: {0}")]
    Collection(String),
}

/// Provider abstraction. The initial implementation intentionally has no NVML dependency.
#[async_trait]
pub trait TelemetryProvider: Send + Sync {
    async fn collect(&self) -> Result<TelemetrySnapshot, TelemetryError>;
}
