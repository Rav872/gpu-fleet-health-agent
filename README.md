# GPU Fleet Health Agent

A Kubernetes node agent that monitors NVIDIA GPU health and telemetry, exports Prometheus metrics, and can cordon and drain nodes when configurable fault conditions are met.

> **Project status:** Planning / early development. The repository currently contains the product requirements and GitLab starter configuration; the agent, Helm/Kubernetes manifests, and Grafana dashboards are not implemented yet.

## Goals

- Detect GPU ECC errors, thermal throttling, and PCIe link degradation on each GPU-equipped Kubernetes node.
- Make node-health decisions using configurable thresholds, persistence windows, and recovery rules rather than reacting to one transient sample.
- Expose GPU and agent health metrics for Prometheus and provide Grafana dashboard definitions.
- Optionally cordon and drain a node through the Kubernetes API when a sustained, actionable fault is detected.
- Make automated remediation observable, auditable, and safe to disable or run in dry-run mode.

## Non-goals for the initial release

- GPU workload scheduling, rescheduling policies, or repair of GPU hardware.
- Replacing NVIDIA DCGM Exporter, Kubernetes device plugins, or cluster-level monitoring. The agent focuses on health evaluation and remediation; existing telemetry components may complement it.
- Automatically rebooting or power-cycling nodes.
- Supporting non-NVIDIA accelerators in the first version.

## Planned architecture

1. **Node agent (Rust):** Runs on GPU nodes as a Kubernetes DaemonSet. Collects telemetry through NVIDIA's Management Library (NVML), evaluates health rules, and serves a Prometheus scrape endpoint.
2. **Health evaluator:** Applies configurable thresholds, consecutive-sample/persistence windows, and cooldowns. Distinguishes warning conditions from faults eligible for remediation.
3. **Remediator:** When enabled, requests a Kubernetes cordon followed by a graceful drain. Records the action and its outcome as metrics and logs. It does not uncordon automatically in the initial release.
4. **Prometheus and Grafana:** Prometheus scrapes each agent; dashboards show GPU health, node status, and remediation history. Alert rules can notify operators independently of remediation.

## Requirements

### Functional requirements

- Collect, per GPU where available:
  - Correctable and uncorrectable ECC error counts, including volatile and aggregate counts if NVML exposes them.
  - GPU temperature and thermal-throttling / performance-state indicators.
  - PCIe link generation and width, with a configurable way to compare current link state to the expected or observed baseline.
  - Collection success, GPU identity, and last successful sample time.
- Handle missing GPUs, unsupported NVML queries, driver/library errors, and device resets without crashing the agent. Report unavailable measurements explicitly rather than treating them as healthy.
- Support configurable collection intervals, fault thresholds, persistence windows, dry-run mode, remediation enablement, and drain timeout.
- Avoid repeated drain requests for the same node fault; expose clear state transitions and remediation outcomes.
- Export Prometheus metrics and provide a health/readiness endpoint for Kubernetes probes.
- Emit structured logs for detected conditions and every remediation decision/result.

### Safety and remediation requirements

- Remediation is **disabled by default**; dry-run is available before enabling real actions.
- Require a sustained fault (configurable consecutive samples or duration) and use hysteresis/cooldowns to avoid flapping.
- Cordon before drain. Drain must honor PodDisruptionBudgets and use a configurable grace period and timeout; report blocked or incomplete drains without silently forcing deletion.
- Never delete DaemonSet-managed pods or mirror pods as part of a drain. Any handling of local-storage pods must be explicit and configurable.
- Do not automatically uncordon nodes or alter workloads beyond the configured node drain operation.
- Make remediation idempotent, bounded, and auditable. Provide a documented operator recovery path.

### Non-functional requirements

- Rust stable toolchain; follow `cargo fmt` and Clippy with warnings treated as errors in CI.
- Low overhead on production nodes, bounded memory use, and no GPU telemetry polling more frequently than configured.
- Graceful shutdown and Kubernetes API retry/backoff with timeouts.
- Secure defaults: least-privilege Kubernetes RBAC, no credentials embedded in images/configuration, non-root container where compatible with NVML device access, and no host mounts beyond those required for GPU access.
- Unit-test health rules without a physical GPU; keep NVML and Kubernetes API interactions behind testable interfaces.
- Support Kubernetes clusters with NVIDIA GPUs and a compatible NVIDIA driver/NVML installation. The exact supported Kubernetes, driver, and Rust versions will be pinned before the first release.

## Initial configuration (proposed)

Configuration names and formats are not finalized. The first implementation should cover equivalents of:

| Setting | Purpose | Safe initial behavior |
|---|---|---|
| `collection_interval` | Time between telemetry samples | Conservative interval; configurable |
| `ecc_uncorrectable_threshold` | Uncorrectable ECC fault threshold | Alert; remediation requires persistence |
| `ecc_correctable_threshold` | Correctable ECC count/rate threshold | Alert; configurable policy |
| `max_temperature_celsius` | Thermal warning/fault threshold | Alert; require persistence for remediation |
| `expected_pcie_link` | Expected PCIe generation/width or baseline policy | Report degradation; do not assume unsupported fields are healthy |
| `fault_persistence` | How long/samples a condition must persist | Required before remediation |
| `remediation_enabled` | Permit cordon/drain actions | `false` |
| `dry_run` | Log intended actions without changing cluster state | `true` |
| `drain_timeout` / `pod_grace_period` | Bound graceful drain behavior | Explicitly configured |

Threshold values should be validated against vendor guidance and representative hardware before production use. Fault-specific defaults should not be guessed for safety-critical remediation.

## Prometheus metrics (planned)

Metric names and labels are provisional. Keep labels bounded; do not use pod names, error text, or other unbounded values as labels.

- `gpu_fleet_agent_up` — agent health.
- `gpu_fleet_gpu_temperature_celsius` — GPU temperature.
- `gpu_fleet_gpu_ecc_errors_total{kind,scope}` — ECC error counters, where supported.
- `gpu_fleet_gpu_pcie_link_info{generation,width}` — observed PCIe link information (info-style gauge).
- `gpu_fleet_gpu_throttle_active{reason}` — active throttling indicator, where supported.
- `gpu_fleet_gpu_telemetry_collection_errors_total{source}` — telemetry collection failures by bounded source category.
- `gpu_fleet_node_health_status{state}` — current evaluated health state.
- `gpu_fleet_remediation_actions_total{action,result}` — attempted/succeeded/failed/dry-run actions.
- `gpu_fleet_remediation_in_progress` — whether a node remediation is in progress.

The final metric contract should document units, types, labels, and meanings and avoid duplicating NVIDIA exporter metrics unnecessarily.

## Grafana dashboard (planned)

The initial dashboard should include fleet/node health overview, per-GPU ECC and temperature trends, thermal throttling and PCIe status, telemetry collection failures, and cordon/drain action outcomes. Include dashboard JSON and a documented Prometheus data source requirement when the dashboard is implemented.

## Development prerequisites

- Rust stable with `rustup`, `cargo`, `rustfmt`, and Clippy.
- Git and access to a GitLab project.
- For hardware/integration testing: a Kubernetes cluster with NVIDIA GPUs, NVIDIA drivers and NVML available to the agent container, and suitable cluster permissions.
- For monitoring integration: Prometheus and Grafana (a local development cluster is sufficient).
- A GPU is not required for unit tests; mock NVML and Kubernetes API boundaries.

## Development and testing (once the Rust crate is added)

Run from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

The GitLab pipeline runs these checks when a Cargo manifest is present. Hardware-dependent integration tests should be isolated and explicitly enabled only on suitable runners.

## GitLab setup

1. Create a **blank project** in GitLab named `gpu-fleet-health-agent`. If the repository is already initialized locally, do not ask GitLab to add a README or other starter files.
2. Commit the starter files, then add the GitLab project URL as `origin` and push the default branch. Use the URL GitLab displays for your project (SSH or HTTPS); do not commit access tokens.
3. In GitLab, verify the default branch, enable pipeline execution, and protect the default branch. Require merge requests and successful pipeline checks before merging.
4. Keep credentials in GitLab CI/CD variables or a trusted identity mechanism. Mask/protect sensitive variables; never put tokens, kubeconfigs, or cluster credentials in this repository.
5. Add a GPU-capable, appropriately isolated GitLab Runner later for hardware integration tests. Do not expose production cluster credentials to general-purpose shared runners.

Example local Git setup (substitute your GitLab URL):

```sh
git init -b main
git add README.md .gitignore .gitlab-ci.yml
git commit -m "docs: define GPU fleet health agent requirements"
git remote add origin <your-gitlab-project-url>
git push -u origin main
```

If this folder is already a Git repository, keep its existing history and add only the missing remote rather than running `git init` again. The commands above create a local repository and push it; they do not create a GitLab account or project for you.

## Delivery roadmap

- [ ] Confirm initial fault semantics and safe thresholds against target NVIDIA hardware.
- [ ] Create the Rust crate, configuration schema, logging, and testable NVML/Kubernetes interfaces.
- [ ] Implement telemetry collection and health-state evaluation with unit tests and simulated telemetry.
- [ ] Implement Prometheus metrics, health probes, and structured event logs.
- [ ] Implement opt-in dry-run remediation, then guarded cordon/drain with RBAC and integration tests.
- [ ] Add DaemonSet deployment manifests, resource/security settings, and installation documentation.
- [ ] Add Prometheus scrape/alert examples and Grafana dashboard JSON.
- [ ] Validate on a GPU test cluster, document supported version matrix and operator recovery procedure.

## Contributing

Changes should include tests for health rules and remediation behavior, keep destructive actions opt-in, and update documentation when configuration, metrics, RBAC, or supported versions change. Do not test real cordon/drain actions against production clusters.

## License

License not selected yet. Choose a license before publishing or accepting contributions.
