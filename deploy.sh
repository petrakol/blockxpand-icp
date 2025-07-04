#!/bin/bash
set -e

# Example deployment step using dfx
# Starts a local replica and deploys the canister so CI can exercise the
# deployment process without needing access to a remote network.
dfx start --background --clean
trap 'dfx stop' EXIT
dfx deploy
