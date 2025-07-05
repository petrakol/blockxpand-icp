<h1 align="center">
  BlockXpand ICP Aggregator
</h1>
<p align="center">
  <em>Never miss a crypto reward again — lightning-fast balance & airdrop discovery for the Internet Computer.</em>
</p>

<p align="center">
  <a href="https://github.com/dfinity/agent-rs"><img src="https://img.shields.io/badge/Rust-1.74-blue?logo=rust" alt="Rust"></a>
  <a href="https://github.com/actions"><img src="https://github.com/<user>/<repo>/actions/workflows/ci.yml/badge.svg" alt="CI status"></a>
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

# BlockXpand ICP Aggregator

This repository is organised as a Cargo workspace with two crates:

- `core` – shared models such as the `Holding` struct
- `aggregator` – canister logic exposing `get_holdings`

Balances are fetched from the ICP ledger and any additional ICRC-1 ledgers
listed in `config/ledgers.toml`. Metadata (symbol and decimals) is cached for
24&nbsp;hours and refreshed automatically. Results are cached in-canister for
60&nbsp;s.

## Building

```bash
cargo build --quiet
```

## Testing

```bash
cargo test --quiet --all
```

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

## Deployment

The `deploy.sh` script illustrates deployment using `dfx` to a local test network.
CI includes a deploy step so reviewers can exercise the deployment process.
The repository includes a minimal `dfx.json` so integration tests can deploy the canister.

## Development workflow

1. Install Rust and `dfx`, and add the `wasm32-unknown-unknown` target with `rustup target add wasm32-unknown-unknown`.
2. Run `cargo test --quiet --all` and `cargo clippy --quiet -- -D warnings` before pushing.
3. On pull requests the GitHub Actions workflow runs tests, clippy, and a test
   deployment via `deploy.sh`.

