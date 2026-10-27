#!/usr/bin/env bash
# Loresmith Player — standalone graphical game player
# Usage: ./Play.sh [adventure_file.json]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

cargo run --bin loresmith_player -- "$@"
