#!/usr/bin/env bash
# Loresmith updater — installs the latest release binaries for this platform.
#
# Usage:
#   curl -L https://github.com/James-HoneyBadger/Loresmith/releases/latest/download/loresmith-update.sh | bash
#   ./loresmith-update.sh [--prefix DIR] [--tag vX.Y.Z] [--archive PATH|URL] [--dry-run]
#
# Environment:
#   LORESMITH_BIN_DIR   install directory (same as --prefix)
#   LORESMITH_REPO      owner/repo to query (default: James-HoneyBadger/Loresmith)
#   LORESMITH_TARGET    override platform detection (e.g. linux-x86_64)

set -euo pipefail

REPO="${LORESMITH_REPO:-James-HoneyBadger/Loresmith}"
BINARIES=(loresmith_player loresmith_console loresmith_workbench loresmith_studio)

prefix=""
tag=""
archive=""
dry_run=0

die() {
    printf 'loresmith-update: %s\n' "$*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Loresmith updater — installs the latest release binaries for this platform.

Usage:
  loresmith-update.sh [options]

Options:
  --prefix DIR        Install into DIR instead of /usr/local/bin or ~/.local/bin
  --tag vX.Y.Z        Install a specific release instead of the latest
  --archive PATH|URL  Install from an already-downloaded archive (offline install)
  --dry-run           Print what would happen without downloading or writing
  -h, --help          Show this help

Environment:
  LORESMITH_BIN_DIR   Install directory (same as --prefix)
  LORESMITH_REPO      owner/repo to query (default: James-HoneyBadger/Loresmith)
  LORESMITH_TARGET    Override platform detection (e.g. linux-x86_64)
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --prefix)
            [ "$#" -ge 2 ] || die "--prefix needs a directory"
            prefix="$2"
            shift 2
            ;;
        --prefix=*)
            prefix="${1#*=}"
            shift
            ;;
        --tag)
            [ "$#" -ge 2 ] || die "--tag needs a release tag such as v6.0.0"
            tag="$2"
            shift 2
            ;;
        --tag=*)
            tag="${1#*=}"
            shift
            ;;
        --archive)
            [ "$#" -ge 2 ] || die "--archive needs a path or URL"
            archive="$2"
            shift 2
            ;;
        --archive=*)
            archive="${1#*=}"
            shift
            ;;
        --dry-run)
            dry_run=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "unknown argument: $1 (try --help)"
            ;;
    esac
done

platform_target() {
    local os arch
    case "$(uname -s)" in
        Linux) os=linux ;;
        Darwin) os=macos ;;
        *) os="$(printf '%s' "$(uname -s)" | tr '[:upper:]' '[:lower:]')" ;;
    esac
    case "$(uname -m)" in
        x86_64|amd64) arch=x86_64 ;;
        arm64|aarch64) arch=aarch64 ;;
        *) arch="$(uname -m)" ;;
    esac
    printf '%s-%s' "$os" "$arch"
}

require_supported_target() {
    case "$1" in
        linux-x86_64|macos-aarch64) return 0 ;;
        mingw*|msys*|cygwin*)
            die "this script is for Linux and macOS. Windows users: download the zip from https://github.com/$REPO/releases/latest"
            ;;
        *)
            die "no prebuilt binaries for $1; build from source instead: https://github.com/$REPO"
            ;;
    esac
}

fetch_to_file() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$1" -o "$2"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$2" "$1"
    else
        die "curl or wget is required"
    fi
}

fetch_to_stdout() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$1"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$1"
    else
        die "curl or wget is required"
    fi
}

latest_archive_url() {
    local target="$1" api json url
    if [ -n "$tag" ]; then
        api="https://api.github.com/repos/$REPO/releases/tags/$tag"
    else
        api="https://api.github.com/repos/$REPO/releases/latest"
    fi
    json="$(fetch_to_stdout "$api")" || die "could not read $api"
    url="$(printf '%s\n' "$json" \
        | grep -o '"browser_download_url": *"[^"]*"' \
        | sed 's/.*"\(http[^"]*\)".*/\1/' \
        | grep -E -- "-${target}\.tar\.gz$" \
        | head -n 1 || true)"
    [ -n "$url" ] || die "no release archive for $target in ${tag:-the latest release} of $REPO"
    printf '%s' "$url"
}

choose_prefix() {
    if [ -n "$prefix" ]; then
        printf '%s' "$prefix"
    elif [ -n "${LORESMITH_BIN_DIR:-}" ]; then
        printf '%s' "${LORESMITH_BIN_DIR}"
    elif [ -w /usr/local/bin ] 2>/dev/null; then
        printf '/usr/local/bin'
    else
        printf '%s/.local/bin' "$HOME"
    fi
}

if [ -n "${LORESMITH_TARGET:-}" ]; then
    target="$LORESMITH_TARGET"
else
    target="$(platform_target)"
fi

dest="$(choose_prefix)"

if [ -n "$archive" ]; then
    source_label="$archive"
else
    require_supported_target "$target"
    source_label="$(latest_archive_url "$target")"
fi

if [ "$dry_run" -eq 1 ]; then
    printf 'platform: %s\n' "$target"
    printf 'source:   %s\n' "$source_label"
    printf 'target:   %s\n' "$dest"
    printf 'binaries: %s\n' "${BINARIES[*]}"
    exit 0
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

case "$source_label" in
    http://*|https://*)
        archive_file="$work/loresmith.tar.gz"
        printf 'Downloading %s\n' "$source_label"
        fetch_to_file "$source_label" "$archive_file"
        ;;
    *)
        archive_file="$source_label"
        [ -f "$archive_file" ] || die "no such archive: $archive_file"
        ;;
esac

mkdir -p "$work/extract"
tar -xzf "$archive_file" -C "$work/extract"

mkdir -p "$dest" || die "cannot create $dest (rerun with sudo or set LORESMITH_BIN_DIR)"

sources=()
for bin in "${BINARIES[@]}"; do
    src="$(find "$work/extract" -type f -name "$bin" -print -quit)"
    [ -n "$src" ] || die "archive does not contain $bin"
    sources+=("$src")
done

for i in "${!BINARIES[@]}"; do
    install -m 755 "${sources[$i]}" "$dest/${BINARIES[$i]}" \
        || die "cannot write $dest/${BINARIES[$i]} (rerun with sudo or set LORESMITH_BIN_DIR)"
    printf 'installed %s\n' "$dest/${BINARIES[$i]}"
done

if [ -x "$dest/loresmith_console" ]; then
    printf 'version: %s\n' "$("$dest/loresmith_console" --version 2>/dev/null || printf 'unknown')"
fi

case ":$PATH:" in
    *":$dest:"*) ;;
    *) printf 'note: %s is not on PATH\n' "$dest" >&2 ;;
esac
