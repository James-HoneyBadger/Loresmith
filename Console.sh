#!/usr/bin/env bash
# Saga Foundry Console — classic terminal player
# Usage: ./Console.sh [adventure_file.json]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

cargo run --bin sagacraft_player -- "$@"
