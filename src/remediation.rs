use async_trait::async_trait;
use thiserror::Error;

/// Request to remediate the Kubernetes node hosting this agent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemediationRequest {
    pub node_name: String,
    pub reason: String,
    pub dry_run: bool,
}

/// Result of a remediation request. Detailed lifecycle states will be added with Kubernetes integration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RemediationOutcome {
    DryRun,
    Completed,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RemediationError {
    #[error("remediation is disabled")]
    Disabled,
    #[error("remediation failed: {0}")]
    Failed(String),
}

/// Remediator abstraction. Implementations must enforce safe cordon/drain semantics.
#[async_trait]
pub trait Remediator: Send + Sync {
    async fn remediate(
        &self,
        request: RemediationRequest,
    ) -> Result<RemediationOutcome, RemediationError>;
}
