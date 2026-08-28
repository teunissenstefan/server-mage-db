#!/bin/sh
# Rust has no equivalent of Go's link-time `-X main.version` injection: the
# binary reports whatever Cargo.toml said when it was compiled. So the tag is
# not the single source of truth here, and a release cut from a stale
# Cargo.toml would ship a binary that lies about its own version — and an
# update notice that never fires. Fail the release instead.
#
# Called from .goreleaser.yaml as: scripts/check-version.sh {{ .Version }} {{ .IsSnapshot }}
set -eu

tag="$1"
snapshot="${2:-false}"

# Snapshot versions are derived from the commit, not from Cargo.toml, so
# there is nothing meaningful to compare on a local dry run.
if [ "$snapshot" = "true" ]; then
    exit 0
fi

have=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)

if [ "$tag" != "$have" ]; then
    echo "Cargo.toml has version $have but the tag says $tag." >&2
    echo "Bump Cargo.toml, commit and push, then tag v$have — or tag the version Cargo.toml already has." >&2
    exit 1
fi
