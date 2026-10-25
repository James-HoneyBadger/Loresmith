#!/usr/bin/env bash
# Saga Foundry Workbench — terminal adventure editor
# Usage: ./Workbench.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

exec "$SCRIPT_DIR/Saga.sh"
