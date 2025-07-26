# Dependency Map

This document summarises the direct Cargo dependencies for each crate in the workspace. Versions are pinned via `[workspace.dependencies]` in the root `Cargo.toml` where possible. This helps keep crates in sync and simplifies upgrades.

| Crate | Purpose | Key Dependencies |
|------|---------|-----------------|
| **bx_core** | shared data types (`Holding`) | `serde`, `candid` |
| **aggregator** | core logic for fetching holdings and rewards | `bx_core`, `ic-cdk`, `serde`, `tokio`, `tracing` |
| **aggregator_canister** | exposes `aggregator` as an IC canister | `aggregator`, `ic-cdk`, `serde_json`, `serde_bytes` |
| **mock_*_canister** | deterministic mock services for tests | `ic-cdk`, `serde` |

Workspace dependency versions:

```toml
[workspace.dependencies]
async-trait = "=0.1.88"
candid = "=0.9.11"
ic-agent = "=0.26.1"
ic-cdk = "=0.10.2"
ic-cdk-macros = "=0.7.1"
serde = { version = "=1.0.219", features = ["derive"] }
serde_json = "=1.0.140"
futures = "=0.3.31"
tokio = { version = "=1.46.0", features = ["macros", "rt", "time"] }
once_cell = "=1.19.0"
ic-cdk-timers = "=0.4.0"
num-traits = "=0.2.19"
tracing = "=0.1.41"
tracing-subscriber = "=0.3.18"
notify = "=6.1.1"
chrono = "=0.4.41"
serde_bytes = "=0.11.17"
```
