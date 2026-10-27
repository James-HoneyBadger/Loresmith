#!/usr/bin/env bash
# Loresmith Workbench — terminal adventure editor
# Usage: ./Workbench.sh

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Run the terminal editor
cargo run --bin loresmith_workbench
