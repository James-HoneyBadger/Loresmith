#!/usr/bin/env bash
# Mellivorex suite launcher
# Usage: ./Suite.sh [player|studio|workbench|console] [args...]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
command="${1:-player}"
if [ "$#" -gt 0 ]; then
  shift
fi

case "$command" in
  player)
    exec "$SCRIPT_DIR/Play.sh" "$@"
    ;;
  studio)
    exec "$SCRIPT_DIR/Studio.sh" "$@"
    ;;
  workbench)
    exec "$SCRIPT_DIR/Workbench.sh" "$@"
    ;;
  console)
    exec "$SCRIPT_DIR/Console.sh" "$@"
    ;;
  -h|--help|help)
    cat <<'EOF'
Mellivorex suite launcher

  ./Suite.sh player [adventure.json]   Standalone graphical player (default)
  ./Suite.sh studio                    Graphical adventure editor
  ./Suite.sh workbench                 Terminal adventure editor
  ./Suite.sh console [adventure.json]  Terminal player

Individual launchers: Play.sh, Studio.sh, Workbench.sh, Console.sh
EOF
    ;;
  *)
    echo "Unknown suite app: $command" >&2
    echo "Run ./Suite.sh --help for the list." >&2
    exit 2
    ;;
esac
