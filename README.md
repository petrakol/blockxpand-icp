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

This repository is organised as a Cargo workspace with four crates:

- `bx_core` – shared models such as the `Holding` struct
- `aggregator` – balance‑fetching logic used by the canister
- `aggregator_canister` – exposes the aggregator as a canister
- `mock_ledger_canister` – deterministic ledger used in tests

Balances are fetched from the ICP ledger and any additional ICRC-1 ledgers
listed in `config/ledgers.toml`. Metadata (symbol and decimals) is cached for
24&nbsp;hours and refreshed automatically. Results are cached in-canister for
60&nbsp;s.

Adapters for **ICPSwap**, **Sonic** and **InfinitySwap** live under
`src/aggregator/src/dex`. Reward claiming APIs are gated by the optional
`claim` feature flag.

### Key features

- **Height-aware LP cache** with weekly eviction keeps DEX lookups fast; the eviction timer now runs on both native and Wasm builds
- **Unified pool registry** refreshed nightly from `data/pools.toml` using
  asynchronous file I/O on native builds (embedded on Wasm builds via the correct
  relative path) and exported via the `pools_graphql` endpoint. A timer schedules
  a nightly refresh on both targets; override the path on native builds via
  `POOLS_FILE`. For Wasm builds the file is embedded using a compile-time
  absolute path
- Optional **reward claiming** via `claim_all_rewards` behind the `claim`
  feature flag
- All DEX adapters now fetch **concurrently** via `join_all` for minimal latency
- The `get_holdings` query runs ledger, neuron, and DEX fetchers concurrently
  for the quickest possible response
- Cross-platform utilities provide a shared `now`, `format_amount` and `get_agent`
  helper used across adapters and the ledger fetcher, plus utilities for querying
  DEX block height and an `env_principal` helper for configuration. Invalid values
  now print a helpful error. Parsed principals are cached so lookups only happen
  once. The shared agent logs any root key error and is cloned after the first
  successful initialisation to avoid repeated network handshakes
- Includes built-in adapters for ICPSwap, Sonic and InfinitySwap
- Common constants like `MINUTE_NS`, `DAY_NS`, `WEEK_NS`, `DAY_SECS` and `WEEK_SECS` centralise refresh durations
- Adapter fetchers yield to the scheduler before starting requests, eliminating
  the previous fixed delay
- A heartbeat-driven queue deterministically warms ledger and DEX metadata
  caches across ticks so refreshes never exceed the 5 s execution limit
- Wasm builds compile cleanly with no warnings
- `deploy.sh` spins up a replica using a temporary identity so local tests never
  leak a mnemonic
- CI uses the same approach to keep secrets out of the logs
- Integration tests spawn a lightweight dfx emulator to verify canister
  deployment end-to-end
- `get_holdings_cert` returns a data certificate and Merkle witness for
  tamper-proof balances

## Building

```bash
cargo build --quiet
```

## Testing

```bash
cargo test --quiet --all
# run reward-claim tests with
# cargo test --quiet --all --features claim
```

## Performance instrumentation

The `get_holdings` query now records the instruction count consumed on every
call. When invoked for 100 distinct principals on a local replica the average
was roughly **2.6&nbsp;B** instructions (≈ cycles), comfortably under the 3 B
budget. The instruction count is printed using `ic_cdk::println!` for each
request so you can verify the cost yourself.

## Ledger configuration

The file `config/ledgers.toml` lists all ICRC-1 ledger canisters that should be
queried. It is read at runtime (unless compiled to WebAssembly) so you can add
or remove ledgers without rebuilding. Set `LEDGERS_FILE` to override the path.
Each entry under `[ledgers]` maps a human name to its canister ID:

```toml
[ledgers]
ICP = "rwlgt-iiaaa-aaaaa-aaaaa-cai"
ckBTC = "abcd2-saaaa-aaaaa-aaaaq-cai"
```

Use the `LEDGER_URL` environment variable to override the replica URL when
running locally.
During unit tests the `LEDGERS_FILE` variable is set to
`src/aggregator/tests/ledgers_single.toml`, which references the mock ledger
canister.

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

When any of these are unset a warning is logged and the fallback from
`ledgers.toml` is used.  The file is watched for changes so updated IDs take
effect without redeploying.  Integration tests set the variables
automatically for the local environment.

## Deployment

The `deploy.sh` script illustrates deployment using `dfx` to a local test network.
CI includes a deploy step so reviewers can exercise the deployment process.
The repository includes a minimal `dfx.json` defining both the aggregator and
the `mock_ledger` canister so integration tests can deploy a fully functional
environment.

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

## Further reading

- [docs/AUDIT_REPORT.md](docs/AUDIT_REPORT.md) summarises the latest security audit
- [docs/DEX_API_matrix.md](docs/DEX_API_matrix.md) lists known DEX canister APIs

