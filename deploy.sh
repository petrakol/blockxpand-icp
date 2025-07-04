#!/bin/bash
set -e

# Example deployment step using dfx
# Deploys to a small test subnet so reviewers can try the canister.
dfx deploy --network ci-test
