#!/usr/bin/env bash
# Saga Foundry Workbench — terminal adventure editor
# Usage: ./Saga.sh

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Run the terminal editor
cargo run --bin sagacraft_ide_tui
