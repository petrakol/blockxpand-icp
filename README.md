# BlockXpand ICP Aggregator

This repository is organised as a Cargo workspace with two crates:

- `core` – shared models such as the `Holding` struct
- `aggregator` – canister logic exposing `get_holdings`

The fetcher implementations are mocked and wrapped in small `tokio::time::sleep` calls to mimic network latency. Results are cached in-canister for 60 s.

## Building

```bash
cargo build
# build the WASM canister
cargo build --target wasm32-unknown-unknown --release -p aggregator_wasm
```

All dependencies are pinned in `Cargo.toml` using workspace versions so that
local and CI builds produce identical WASM binaries.

## Testing

```bash
cargo test --all
```

## Deployment

The `deploy.sh` script illustrates deployment using `dfx` to a test subnet. CI includes a deploy step so reviewers can access the resulting canister ID. The repository includes a minimal `dfx.json` so integration tests can deploy the canister.

## Development workflow

1. Install Rust and `dfx`. Add the `wasm32-unknown-unknown` target with `rustup target add wasm32-unknown-unknown`. If Clippy is missing, install it via `rustup component add clippy`.
2. Run `cargo test --all` and `cargo clippy -- -D warnings` before pushing.
3. On pull requests the GitHub Actions workflow runs tests, clippy, and a test deployment via `deploy.sh`.
4. Integration tests that call `dfx` locally require the CLI to be installed.
