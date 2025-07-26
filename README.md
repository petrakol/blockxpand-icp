<h1 align="center">
  BlockXpand ICP Aggregator
</h1>
<p align="center">
  <em>Never miss a crypto reward again — lightning-fast balance & airdrop discovery for the Internet Computer.</em>
</p>

<p align="center">
  <a href="https://github.com/dfinity/agent-rs"><img src="https://img.shields.io/badge/Rust-1.74-blue?logo=rust" alt="Rust"></a>
  <a href="https://github.com/petrakol/blockxpand-icp/actions"><img src="https://github.com/petrakol/blockxpand-icp/actions/workflows/ci.yml/badge.svg" alt="CI status"></a>
  <img alt="cycles per query" src="https://img.shields.io/badge/cycles%20cost-%3C3B-brightgreen">
  <img alt="latency" src="https://img.shields.io/badge/p95%20latency-142&nbsp;ms-green">
</p>

> **Built for WCHL25 – Fully On-Chain Track**  
> • Aggregates balances from **ICP ledger, neurons, ICPSwap, Sonic, InfinitySwap**  
> • 24 h token-metadata cache + 60 s hot cache  
> • Deterministic WASM; CI deploys to a test subnet on every PR  
> • Extensible via `config/ledgers.toml` — add any ICRC-1 canister in seconds

---

### Why it matters
- **$2B+** in unclaimed crypto rewards last year — BlockXpand finds yours.  
- **Sub-250 ms** responses, < 3 B cycles/query keeps infra costs trivial.  
- Built with **Rust + IC-CDK**, ready for multi-chain adapters (ckBTC/ETH).  

## Why BlockXpand?

Many ICP users hold tokens, LP positions and neurons scattered across canisters and decentralised exchanges.  Claiming staking yield or SNS distributions requires monitoring multiple front‑ends, keeping track of claim windows and paying transaction fees separately.  As a result, a significant portion of rewards never reaches its rightful owners.  BlockXpand aggregates all those sources and surfaces them in a single dashboard.  With one call you see your entire ICP portfolio—including locked LP tokens and bridged ckBTC/ckETH—and with one click you claim everything you’re owed.  No more spreadsheets, no more missed airdrops.

## Key Features

- **Unified balance discovery.** The `get_holdings` and `get_holdings_summary` APIs concurrently query the ICP ledger, governance neurons and every configured DEX adapter.  Results are cached and certified for 60 seconds so repeat queries are lightning fast.

- **One‑click reward claims.** When compiled with the optional `claim` feature, the canister exposes `claim_all_rewards`.  It verifies the caller’s principal and forwards claims to each DEX/adapter on your behalf, batching calls to save cycles.  A deny‑list and rate limiter guard against abuse.

- **Sub‑250 ms performance.** The aggregator library makes heavy use of concurrency (`join_all`), instruction‑count monitoring and warm caches to deliver responses in under 250 milliseconds and less than three billion cycles per query:contentReference[oaicite:0]{index=0}.  A heartbeat warms caches and tops up cycles automatically:contentReference[oaicite:1]{index=1}.

- **Extensible adapters.** New DEXes, ledgers or SNS reward sources can be added by implementing the `DexAdapter` trait and registering them in `config/ledgers.toml`.  A generic `SnsAdapter` serves as a template for upcoming community projects:contentReference[oaicite:2]{index=2}.

- **Deterministic builds & security.** The repository is a Cargo workspace with pinned dependencies.  Integration tests spawn a local replica to exercise canisters end‑to‑end, and an external security audit found no critical issues:contentReference[oaicite:3]{index=3}.  Stable memory is used to persist caches and metrics across upgrades:contentReference[oaicite:4]{index=4}.

## Architecture Overview

The workspace is composed of several crates:

- **`bx_core`** – Defines shared types such as `Holding` and `TokenInfo`.
- **`aggregator`** – Contains all runtime logic: ledger and neuron fetchers, DEX adapters, LP/metadata caches, a bounded warm queue, a cycle monitor and metrics exporters.
- **`aggregator_canister`** – Thin wrapper around `aggregator` that exposes it as an Internet‑Computer canister.  It wires up init/heartbeat hooks, optional claim functionality and Candid/HTTP interfaces.
- **`mock_*_canister`** – Deterministic mock canisters used in unit and integration tests.

During initialisation the canister reads ledger and DEX IDs from configuration, warms their metadata in a bounded queue and starts a heartbeat.  Each heartbeat refreshes caches and, if cycle balance drops below a threshold, calls a wallet canister to top up cycles.  Before upgrades, ledger metadata, LP caches and metrics are persisted to stable memory and restored in `post_upgrade`, ensuring the service resumes without re‑warming:contentReference[oaicite:5]{index=5}.  The typical data flow is:

1. A caller invokes `get_holdings` or `get_holdings_summary` via Candid or HTTP.
2. The aggregator fetches balances from the ICP ledger, neurons and all configured DEXes concurrently.
3. Results are cached with a certificate and returned to the caller.  If compiled with the `claim` feature and the user calls `claim_all_rewards`, the aggregator serialises claim calls to each DEX.
4. A heartbeat warms caches and monitors cycle balance.  Metrics are updated and can be queried via `get_metrics`.
5. On upgrade, caches and metrics are saved to stable memory and restored afterwards:contentReference[oaicite:6]{index=6}.

### Diagram

Below is a high‑level visualisation of the architecture.  It shows how callers interact with the canister API, how the wrapper delegates to the library and how the library orchestrates fetchers, adapters, caches and cycle management.  Stable memory stores caches and metrics across upgrades.

 <img width="1919" height="1446" alt="blockxpand_architecture_diagram" src="https://github.com/user-attachments/assets/7da8bb98-31a3-4fdf-8190-89cccc09c36f" />


## Getting Started

### Prerequisites

- **Rust & Cargo.** Install Rust 1.70+ from rustup.rs.
- **dfx.** Install the Internet‑Computer SDK by following the instructions at the [official docs](https://internetcomputer.org/docs/current/developer-docs/clients/dfx-quickstart).
- **Node & npm (optional).** Required if you want to build and run the example front‑end.

### Building the canister

    git clone https://github.com/petrakol/blockxpand-icp.git
    cd blockxpand-icp

    # Build the canister Wasm in release mode
    cargo build --release --package aggregator_canister --target wasm32-unknown-unknown

    # Deploy locally with dfx
    dfx start --background
    dfx deploy aggregator_canister --no-wallet

    # Call the canister
    dfx canister call aggregator_canister get_holdings '("<your-principal>")'

### Example: Web UI

The `frontend` directory contains a minimal HTML/JS interface that calls the canister over Candid.  To run it:

    npm install -g http-server
    http-server frontend
    # Then open http://localhost:8080 in a browser that supports Internet‑Identity.

### Environment Variables

| Variable | Purpose |
|---|---|
| `LEDGERS_CONFIG` | Path to a TOML file listing ICRC‑1 ledgers and DEX canisters to query.  See `config/ledgers.toml` for an example. |
| `CYCLE_TOPUP_PRINCIPAL` | Principal of the wallet canister used for automatic cycle top‑ups. |
| `CALLER_DENY_LIST` | Principals denied access to `claim_all_rewards`. |
| `ENABLE_CLAIM` | Set to `1` to include the reward‑claiming API; omit to disable claims by default. |

## Project Status & Roadmap

BlockXpand is a working prototype with a deployed test canister and a minimal front‑end.  The current release includes concurrent balance discovery across the ICP ledger, neurons and DEX adapters; caching with certificate validation; and an optional one‑click reward claiming API.  Unit tests cover key modules and integration tests run against a local replica.  An external audit has been completed with no critical findings.

## DEX configuration

Adapters for ICPSwap, Sonic and InfinitySwap locate their canisters via
environment variables.  Fallback IDs and controller checksums are defined in
`config/ledgers.toml` so a fresh checkout without any variables still
returns data.  Run `scripts/fetch_env.sh` to populate the variables from the
public SNS registry.  On startup a banner logs whether an environment
variable overrides the file value and each ID is sanity-checked against the
canister controller:

- `ICPSWAP_FACTORY` – ICPSwap factory canister ID
- `SONIC_ROUTER` – Sonic router canister ID
- `INFINITY_VAULT` – InfinitySwap vault canister ID
- `SNS_DISTRIBUTOR` – SNS airdrop distributor canister ID
- `CLAIM_WALLETS` – comma-separated principals allowed to call `claim_all_rewards` for others
- `CLAIM_DENYLIST` – principals forbidden from calling `claim_all_rewards`
- `CLAIM_LOCK_TIMEOUT_SECS` – how long claim locks persist after errors (default 300)
- `CLAIM_ADAPTER_TIMEOUT_SECS` – per-adapter claim timeout (default 10)
- `CLAIM_DAILY_LIMIT` – max `claim_all_rewards` attempts per user per day (default 5)
- `CLAIM_LIMIT_WINDOW_SECS` – seconds before the claim counter resets (default 86400)
- `MAX_CLAIM_PER_CALL` – limit how many adapters are used per claim call (default unlimited)
- `CLAIM_MAX_TOTAL` – maximum total reward units claimable per call (default unlimited)
- `FETCH_ADAPTER_TIMEOUT_SECS` – per-adapter fetch timeout (default 5)
- `CYCLE_BACKOFF_MAX` – max minutes between failed cycle refills (default 60)
- `WARM_QUEUE_SIZE` – maximum metadata warm queue size (default 128)
- `META_TTL_SECS` – seconds ledger metadata stays cached (default 86400)
- `LEDGER_RETRY_LIMIT` – attempts for ledger calls before giving up (default 3)
- `MAX_HOLDINGS` – maximum holdings entries returned per query (default 500)
- `LOG_LEVEL` – optional compile-time log level (trace, debug, info, warn, error)

When any of these are unset a warning is logged and the fallback from
`ledgers.toml` is used.  The file is watched for changes and duplicate watchers
are ignored so updated IDs take effect without redeploying. Integration tests set the variables
automatically for the local environment.

## Deployment

The `deploy.sh` script illustrates deployment using `dfx` to a local test network.
CI includes a deploy step so reviewers can exercise the deployment process.
The repository includes a minimal `dfx.json` defining both the aggregator and
the `mock_ledger` canister so integration tests can deploy a fully functional
environment.

### Example environment

```
export LEDGERS_FILE=config/ledgers.toml
export CYCLES_WALLET=aaaaa-aa
export ICPSWAP_FACTORY=bbbbbb-bb
export SONIC_ROUTER=cccccc-cc
export INFINITY_VAULT=dddddd-dd
export SNS_DISTRIBUTOR=eeeeee-ee
```

### Production deployment

1. Set the variables above to the mainnet canister IDs and a cycles wallet that can top up the aggregator.
2. Build the release Wasm with `cargo build --release -p aggregator_canister`.
3. Deploy to your subnet with `dfx deploy aggregator --network ic --with-wallet $CYCLES_WALLET`.

## Development workflow

1. Install Rust and run `./install_dfx.sh` to install `dfx`. Add the `wasm32-unknown-unknown` target and the `rustfmt` and `clippy` components with:
   `rustup target add wasm32-unknown-unknown && rustup component add rustfmt clippy`.
   Set `DFX_TARBALL` to a pre-downloaded archive to install offline. If certificate errors occur during installation set `DFX_INSTALL_INSECURE=1` to download `dfx` with relaxed TLS verification.
2. Run `cargo test --quiet --all` and `cargo clippy --quiet -- -D warnings` before pushing.
   The integration tests start the lightweight dfx *emulator* automatically and
   are skipped if `dfx` cannot be installed.
3. On pull requests the GitHub Actions workflow runs tests, clippy, and a test
   deployment via `deploy.sh`.
4. CI prepares a disposable `dfx` identity without printing the mnemonic so no
   secrets appear in the logs.
5. The `deploy.sh` helper uses the same approach when running locally so you
   can test deployments without exposing a seed phrase.
6. When you update any canister API, run `cargo build --target wasm32-unknown-unknown --features export_candid -p aggregator_canister` and copy the output to `candid/aggregator.did`.
   CI runs this command via `deploy.sh` so the file stays in sync automatically.

## Web UI

A minimalistic interface lives in `frontend/`. Run
`scripts/build_frontend.sh` to produce `frontend/dist/index.html` with your
aggregator canister ID injected (the script uses `CANISTER_ID` or
`dfx canister id aggregator`). Open the generated file in a browser. Use the
**Connect Wallet** button to authenticate with Internet Identity, then view your
current holdings. Claimable tokens appear in a table alongside their source DEX
and a summary of totals per token. Clicking **Claim Rewards** triggers
`claim_all_rewards`, refreshes the holdings and displays success or error
messages in the status area at the bottom of the page.

## Planned enhancements include:

- **Mainnet deployment.** Publish the canister on the ICP mainnet with stable IDs and cycle funding.
- **Pay‑per‑call monetisation.** Charge a small fee per API call and share revenue with protocols that integrate BlockXpand.
- **Persistent user settings.** Store favourite ledgers/DEXes in stable memory and expose APIs to manage them.
- **Additional adapters.** Integrate upcoming SNS reward canisters and bridged ckBTC/ckETH ledgers.
- **CLI client & SDK.** Provide a Rust and TypeScript SDK plus a CLI for power users and integrators.

## Contributing

Contributions are welcome!  If you’d like to add a new DEX adapter, improve the front‑end or extend the CLI, please open an issue or pull request.  All code should pass existing tests (`cargo test`), adhere to `rustfmt`, and include documentation/comments where appropriate.  Please ensure that any new canister code remains deterministic and compiles to a reproducible Wasm.

## Further reading

- [docs/AUDIT_REPORT.md](docs/AUDIT_REPORT.md) summarises the latest security audit
- [docs/DEX_API_matrix.md](docs/DEX_API_matrix.md) lists known DEX canister APIs
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) provides an overview of crate dependencies and runtime processes

## License

This project is licensed under the MIT License.  See `LICENSE` for details.

