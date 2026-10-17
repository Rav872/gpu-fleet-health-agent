//! Building blocks for the GPU Fleet Health Agent.
//!
//! Hardware and Kubernetes integrations are intentionally not implemented yet.
//! Telemetry and remediation are represented by interfaces so those integrations
//! can be added and tested independently.

pub mod config;
pub mod remediation;
pub mod telemetry;
