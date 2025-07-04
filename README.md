# BlockXpand ICP Aggregator

This repository is organised as a Cargo workspace with two crates:

- `core` – shared models such as the `Holding` struct
- `aggregator` – canister logic exposing `get_holdings`

The fetcher implementations are placeholders but demonstrate how the workspace is laid out.

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```

## Deployment

The `deploy.sh` script illustrates how a canister could be deployed using `dfx`. It currently acts as a placeholder.

