#!/usr/bin/env bash
set -euo pipefail

# Install dfx locally if it is not already available.
# Uses DFINITY's install script in non-interactive mode.

DFX_VERSION="${DFX_VERSION:-0.17.0}"
if command -v dfx >/dev/null 2>&1; then
    echo "dfx already installed: $(dfx --version)"
    exit 0
fi

if [ -n "${DFX_TARBALL:-}" ] && [ -f "$DFX_TARBALL" ]; then
    echo "Installing dfx from $DFX_TARBALL..."
    temp_dir=$(mktemp -d)
    tar -xzf "$DFX_TARBALL" -C "$temp_dir"
    "$temp_dir"/dfx-*/install.sh -y >/dev/null
    rm -rf "$temp_dir"
else
    echo "Installing dfx ${DFX_VERSION}..."
    DFXVM_INIT_YES=1 DFX_VERSION="$DFX_VERSION" sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)" >/dev/null
fi

echo "dfx $(dfx --version) installed"
