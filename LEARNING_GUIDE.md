# GPU Fleet Health Agent — Learning Guide

Use this file as the guided tour of the project. It explains what exists, why each part exists, where to read the code, and what we plan to learn next. The goal is for you to understand and implement the project incrementally—not to hide the work behind a generated solution.

> **Current state:** We are on the `feat/rust-agent-skeleton` branch. The first Rust scaffold is present, but it has not yet been validated or reviewed. The process is intended to parse configuration, log a heartbeat, and shut down on Ctrl-C. It does not yet read a GPU, evaluate faults, serve metrics, or communicate with Kubernetes.

## How to use this guide

Follow the numbered steps in order. For each step:

1. Read the listed file(s) and try to explain their purpose in your own words.
2. Trace how data or control moves from one file to another.
3. Run the small check or experiment listed for that step.
4. Ask questions or make a small change only after the current behavior makes sense.

File links below take you directly to the relevant project files.

### Learning-first collaboration agreement

Before I write or change code that depends on a new Rust concept, library, tool, or technology, I will first explain what you need to learn and why it matters. We will pause implementation while you study it; I will not assume you are ready or silently build ahead. Once you say you understand the topic and want to continue, we will make the smallest ticket-scoped change, then review and test it together. If a step uses only concepts you have already covered, I will still briefly explain the plan before editing.

## Current ticket: `feat/rust-agent-skeleton`

We will finish only this ticket before starting the next feature. Work stays on this branch, and we merge it into `main` only after its scope is complete, checks pass, and you have reviewed the changes.

### Included in this ticket

- A compiling Rust package with a pinned toolchain and reproducible dependency lockfile.
- Minimal configuration parsing and validation, with safe remediation defaults.
- Structured startup/shutdown logging, a placeholder heartbeat, and Ctrl-C handling.
- Telemetry and remediation data types/interfaces only, so later integrations have explicit boundaries.
- Focused tests for configuration behavior, plus formatting, tests, and Clippy passing locally and in CI.
- Documentation that accurately describes the starter's current limitations.

### Explicitly out of scope

- NVML calls, GPU telemetry sampling, GPU rentals, and real hardware fault testing.
- Fault evaluation, thresholds, persistence/hysteresis, or health state decisions.
- Prometheus metrics or an HTTP endpoint.
- Kubernetes API access, cordon/drain behavior, or any live remediation.
- DaemonSet/RBAC manifests, Grafana dashboards, or deployment automation.

If a proposed change implements one of these later capabilities, we will defer it to its own feature branch/ticket. For this ticket, completion means the included items are understood, implemented, tested, and reviewed—not that the entire product is complete.

## Part 1 — What is a Rust project?

### Step 1: Project metadata and repeatable tools

**Read:** [Cargo.toml](Cargo.toml), [Cargo.lock](Cargo.lock), and [rust-toolchain.toml](rust-toolchain.toml).

- `Cargo.toml` is the package manifest. It names the crate, sets the Rust edition and minimum compiler version, and lists external crates (Rust libraries) the program uses.
- `Cargo.lock` records the exact resolved dependency versions so local builds and CI can reproduce the same dependency graph. It belongs in Git for this application.
- `rust-toolchain.toml` asks Rustup to use a particular compiler and components. This project currently pins Rust 1.79.0, matching the detected local toolchain.
- `async-trait` enables async functions in traits; `clap` parses command-line options and environment variables; `thiserror` helps define errors; `tokio` provides the async runtime and signal handling; `tracing` provides structured logs.

**Questions to answer:** Which file lists a dependency by name? Which file locks its resolved version? What does the toolchain file select?

**Check:** `cargo metadata --no-deps` prints the crate metadata without building the program.

### Step 2: Library modules and the executable

**Read:** [src/lib.rs](src/lib.rs), then [src/main.rs](src/main.rs).

- `src/lib.rs` declares public modules. The library crate is named with underscores in Rust imports (`gpu_fleet_health_agent`) even though the package name in Cargo uses hyphens.
- `src/main.rs` is the executable entry point. `main` initializes logging, parses and validates configuration, logs startup, then waits for either a timer tick or Ctrl-C.
- `#[tokio::main]` starts Tokio's asynchronous runtime so `main` can use async-compatible APIs. The timer is currently only a heartbeat placeholder, not a telemetry poll.
- `tokio::select!` waits for whichever event occurs first. The Ctrl-C path allows a normal shutdown message instead of an abrupt endless loop.

**Questions to answer:** What function runs first? Why does the program need Tokio? What happens on a timer tick today—and what does not happen?

## Part 2 — Configuration and safe defaults

### Step 3: Parse settings, validate them, and test them

**Read:** [src/config.rs](src/config.rs).

- `Config` describes the settings accepted by the process. Clap's `#[arg]` annotations define command-line names, environment variable names, and defaults.
- `Config::parse()` is called in `main`. `Config::validate()` checks a rule that can be safely checked without a GPU or cluster: the collection interval must be nonzero.
- The tests show expected behavior. They check safe defaults, invalid interval rejection, and parsing explicit remediation options.
- Remediation defaults to disabled and dry-run defaults to enabled. These settings are intentionally conservative because future remediation can disrupt workloads.
- At this stage the remediation settings are only parsed and logged; no remediation can occur.

**Questions to answer:** What is the default polling interval? Which test proves remediation is off by default? What error occurs for an interval of zero?

**Check:** `cargo test config::tests` runs just the configuration tests.

## Part 3 — Design boundaries before hardware integrations

### Step 4: Telemetry data and provider interface

**Read:** [src/telemetry.rs](src/telemetry.rs).

- `GpuId`, `GpuTelemetry`, and `TelemetrySnapshot` describe the information a future collector may return.
- Measurement fields are `Option`s. `Some(value)` means a reading is available; `None` means it is unavailable or unsupported. That distinction matters: missing data must not be mistaken for a healthy GPU.
- `TelemetryProvider` is a trait, or contract, that future implementations must satisfy. One implementation can use NVML; a mock implementation can return made-up samples in unit tests.
- There is no real provider yet. This file defines the boundary only.

**Questions to answer:** Why are readings optional? Why is an empty GPU list different from a collection error? How could a mock avoid needing physical GPU hardware?

### Step 5: Remediation request and interface

**Read:** [src/remediation.rs](src/remediation.rs).

- `RemediationRequest` describes the node, the reason for the request, and whether it is a dry run.
- `RemediationOutcome` and `RemediationError` describe possible results.
- `Remediator` is an interface for a future implementation that talks to Kubernetes.
- This is not yet a working dry-run or Kubernetes integration. The safety rules in the README (cordon before graceful drain, respect PodDisruptionBudgets, avoid unsafe deletion, and never automatically uncordon) still need implementation and tests before the interface can be used operationally.

**Questions to answer:** What information does a remediation request carry? Why should a mock remediator be testable before adding Kubernetes credentials?

## Part 4 — Tooling, checks, and workflow

### Step 6: Formatting, tests, and linting

**Read:** [.github/workflows/ci.yml](.github/workflows/ci.yml) and [.gitlab-ci.yml](.gitlab-ci.yml).

The workflows are intended to run formatting checks, Clippy (Rust's linter), and tests when a Cargo manifest exists. A configured CI workflow does not prove the code passes; the local commands below still need to be run and any failures understood.

From the repository root, run one command at a time and read its output:

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

`cargo fmt` checks consistent formatting, `cargo test` compiles and runs tests, and Clippy reports common mistakes and quality issues. If formatting needs changes, inspect what would change before applying formatting. The first run may download dependencies.

## What is implemented—and what is not

| Capability | Current state | Main file(s) |
|---|---|---|
| Rust package and dependencies | Present | [Cargo.toml](Cargo.toml), [Cargo.lock](Cargo.lock) |
| Configuration defaults and validation | Present; config-only tests included | [src/config.rs](src/config.rs) |
| Structured startup and heartbeat logs | Present | [src/main.rs](src/main.rs) |
| Graceful Ctrl-C handling | Present | [src/main.rs](src/main.rs) |
| Telemetry and remediation interfaces | Types/contracts only; no implementations | [src/telemetry.rs](src/telemetry.rs), [src/remediation.rs](src/remediation.rs) |
| NVML collection | Not implemented | Future work behind `TelemetryProvider` |
| Fault thresholds and persistence/hysteresis | Not implemented | Future health evaluator module |
| Prometheus endpoint and metrics | Not implemented | Future metrics module |
| Kubernetes cordon and drain | Not implemented | Future `Remediator` implementation |
| DaemonSet, RBAC, Grafana dashboard | Not implemented | Later deployment/monitoring branch |

## Next learning steps

Do not jump to renting a GPU yet. These phases can be developed and tested locally:

1. **Understand and validate the skeleton.** Run the formatter, tests, and Clippy commands above. Read failures before changing code. Learn the current data types and test structure.
2. **Practice with a mock telemetry provider.** Write a fake provider that returns sample data. This teaches the trait boundary and async testing without NVML or hardware.
3. **Build a pure health evaluator.** Feed it simulated temperatures, ECC counts, and PCIe information. Add persistence and hysteresis tests. This logic should be deterministic and GPU-independent.
4. **Add metrics and a local HTTP endpoint.** Use mock samples and check the output locally; a physical GPU is not needed.
5. **Rent or borrow an NVIDIA GPU when implementing NVML.** This is the first stage that benefits from real GPU access. Verify which NVML readings the chosen GPU, driver, and environment actually expose. Start with read-only telemetry; don't intentionally create hardware faults.
6. **Test Kubernetes behavior in a disposable cluster.** Begin with a fake/mock Kubernetes client and dry-run. A real GPU-backed Kubernetes node is useful later to verify the DaemonSet's device access and the complete deployment path, but keep live cordon/drain tests in a disposable cluster.
7. **Add dashboards and operator documentation.** These can use mocked/local Prometheus metrics initially; GPU rental is not required.

Never use production workloads to test cordon/drain behavior. Keep remediation disabled until its safety behavior, RBAC, timeout handling, and recovery procedure are reviewed.

## Git workflow for this learning branch

We are working on `feat/rust-agent-skeleton`. Small work-in-progress commits may be pushed to this feature branch to show daily progress; that does not mean the ticket is complete or ready to merge. Merge into `main` only after you understand and review the ticket changes and all required checks pass. Start the next feature branch from an updated `main` after this one is merged.

## Daily progress log

### 2026-10-17 — Rust skeleton ticket started

- **Progress:** Defined this ticket's scope and exclusions; added a first-pass Rust crate skeleton, initial configuration tests, telemetry/remediation interfaces, and this learning guide. The guide records each file's responsibility and a learn-first sequence.
- **Learning status:** The Rust concepts and scaffold are not yet confirmed as understood by the learner. No learning completion is claimed; review the guide and ask about unfamiliar concepts before extending the Rust code.
- **Validation status:** Formatting, tests, and Clippy have not yet completed successfully. Treat the scaffold as work in progress; no merge is appropriate until the learner reviews it and all ticket checks pass.
- **Not done today:** No GPU was rented, no NVML or Kubernetes integration was added, and no real remediation is implemented.
- **Next session:** Start with Cargo metadata and Rust modules/types in Steps 1–3 above. Explain any unfamiliar tooling before running it; then review code and validate one check at a time.

### 2026-10-18 — First validation pass

- **Learning progress:** Reviewed the Cargo package metadata, library/executable split, configuration and error types, and telemetry/remediation interfaces. `cargo metadata --no-deps --format-version 1` succeeded.
- **Formatting:** `cargo fmt --all -- --check` initially found layout differences in `src/config.rs` and `src/main.rs`; those were corrected, and the check then passed.
- **Tests:** `cargo test --all-targets` is currently blocked before compilation/tests run: the resolved `clap_derive` version requires Cargo support for Edition 2024, which Cargo 1.79 does not have. This is a toolchain/dependency compatibility issue, not a test failure. We have not selected or applied a fix yet.
- **Still pending:** Choose a compatible minimum Rust version or dependency set after understanding the trade-off; then rerun tests and Clippy. The ticket remains incomplete and must not be merged yet.
- **Out of scope today:** No GPU, NVML, Kubernetes integration, Prometheus endpoint, or remediation behavior was added.
