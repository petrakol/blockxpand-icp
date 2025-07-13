# Architecture Overview

This repository is a Cargo workspace composed of several crates that work together to fetch balances from multiple ledgers and DEXes and expose them through an Internet Computer canister.

## Crates

- **bx_core** – Shared data structures such as the `Holding` type.
- **aggregator** – Library containing all runtime logic:
  - ledger and neuron fetchers
  - DEX adapters
  - cycle top‑ups and heartbeat driven warm queue
  - LP cache and operational metrics
- **aggregator_canister** – Thin wrapper that exposes `aggregator` as a canister. It wires up init, heartbeat and upgrade hooks and optionally exports the Candid interface.
- **mock_*_canister** – Deterministic mock canisters used in unit and integration tests.

## Processes

1. **Warm queue** – On init the queue loads ledger and DEX IDs and gradually warms their metadata. The queue is bounded and deduplicates entries to avoid unbounded growth.
2. **Cycle monitor** – Every heartbeat checks the cycle balance and calls a wallet canister to top up when needed. Failures trigger exponential backoff and each event is logged in stable memory.
3. **Metrics** – Query and heartbeat counts plus cycle balance are tracked and can be queried via the `get_metrics` endpoint. Metrics state is preserved across upgrades.
4. **Upgrade flow** – Before upgrades the cycle log, ledger metadata and LP caches and metrics are saved to stable memory. They are restored in `post_upgrade` so the canister resumes operation without warming up again.

The [README](../README.md) explains how to configure environment variables and run the deployment script. The integration tests under `tests/` launch a local replica to exercise these processes end‑to‑end.
