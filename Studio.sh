#!/usr/bin/env bash
# Saga Foundry Studio — graphical adventure editor
# Usage: ./Studio.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

cargo run --bin sagacraft_ide_gui
