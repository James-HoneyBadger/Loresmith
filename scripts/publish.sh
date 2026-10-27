#!/usr/bin/env bash
# Publish the Loresmith crates to crates.io in dependency order.
#
# Usage:
#   scripts/publish.sh [--dry-run]
#
# Environment:
#   CARGO_REGISTRY_TOKEN   crates.io API token (or run `cargo login` first)

set -euo pipefail

# Dependency order: the libraries first, then the applications that pin them.
CRATES=(loresmith_rs loresmith_ui loresmith_console loresmith_workbench loresmith_player loresmith_studio)

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

dry_run=0
case "${1:-}" in
    --dry-run) dry_run=1 ;;
    -h|--help)
        sed -n '2,9p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
        exit 0
        ;;
    "") ;;
    *)
        printf 'publish: unknown argument: %s (try --help)\n' "$1" >&2
        exit 2
        ;;
esac

die() {
    printf 'publish: %s\n' "$*" >&2
    exit 1
}

version="$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
[ -n "$version" ] || die "could not read the workspace version from Cargo.toml"

if [ "$dry_run" -eq 0 ] \
    && [ -z "${CARGO_REGISTRY_TOKEN:-}" ] \
    && [ ! -s "$HOME/.cargo/credentials.toml" ]; then
    die "no crates.io credentials: set CARGO_REGISTRY_TOKEN or run 'cargo login'"
fi

published() {
    curl -fsS -A loresmith-publish \
        "https://crates.io/api/v1/crates/$1/$2" >/dev/null 2>&1
}

wait_for_registry() {
    local crate="$1" attempts=0
    until published "$crate" "$version"; do
        attempts=$((attempts + 1))
        if [ "$attempts" -ge 60 ]; then
            die "$crate $version did not appear on crates.io within 5 minutes"
        fi
        sleep 5
    done
}

printf 'Publishing Loresmith %s%s\n' "$version" "$([ "$dry_run" -eq 1 ] && printf ' (dry run)')"

for crate in "${CRATES[@]}"; do
    if published "$crate" "$version"; then
        printf '  %-22s already published, skipping\n' "$crate"
        continue
    fi

    if [ "$dry_run" -eq 1 ]; then
        if [ "$crate" = "loresmith_rs" ] || [ "$crate" = "loresmith_ui" ]; then
            cargo publish --dry-run -p "$crate"
        else
            # The applications pin the libraries by version; cargo refuses to
            # package them until those versions exist on crates.io.
            printf '  %-22s skipped (needs loresmith_rs/loresmith_ui on crates.io)\n' "$crate"
            continue
        fi
        printf '  %-22s dry run verified\n' "$crate"
        continue
    fi

    cargo publish -p "$crate"
    printf '  %-22s published\n' "$crate"

    # Later crates pin this one by version; wait until the index serves it.
    wait_for_registry "$crate"
done

printf 'Done: Loresmith %s\n' "$version"
