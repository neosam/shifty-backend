#!/usr/bin/env bash

set -euo pipefail

# Usage:
#   cli-update-version.sh [-m tag-message] [-b branch] <RELEASE> [NEXT]
#
# Mechanical release executor: bumps every crate + nix file to RELEASE, builds,
# commits, tags v$RELEASE, pushes, then bumps to the next -dev version. It does
# NOT decide the version number — the caller (e.g. the release-version skill)
# derives it (from the GSD milestone in .planning/STATE.md + existing tags) and
# passes it in explicitly.
#
# Arguments:
#   RELEASE  (required)  SemVer version to release, e.g. 2.0.0 (must be X.Y.Z).
#   NEXT     (optional)  Base for the next in-development version. Defaults to
#                        RELEASE with the patch incremented (2.0.0 -> 2.0.1).
#                        A "-dev" suffix is always appended to it.
#
# Options:
#   -m tag-message  Annotated-tag message (e.g. release notes). Without it, `git tag -a`
#                   would open an interactive editor and block non-interactive runs.
#   -b branch       Branch to tag and push the -dev bump onto (default "main").
#
# Examples:
#   cli-update-version.sh -m "Release notes" 2.0.0        # next dev auto -> 2.0.1-dev
#   cli-update-version.sh -m "Release notes" 2.0.0 2.1.0  # next dev explicit -> 2.1.0-dev

TAG_MESSAGE=""
BRANCH_OPT=""

usage() {
    echo "Usage: $0 [-m tag-message] [-b branch] <RELEASE> [NEXT]" >&2
    exit 1
}

while getopts "m:b:" opt; do
    case $opt in
        m) TAG_MESSAGE="$OPTARG" ;;
        b) BRANCH_OPT="$OPTARG" ;;
        *) usage ;;
    esac
done
shift $((OPTIND - 1))

if [ $# -lt 1 ] || [ $# -gt 2 ]; then
    usage
fi

NEW_VERSION="$1"

# RELEASE must be a plain X.Y.Z SemVer (no -dev, no build metadata).
if [[ ! "$NEW_VERSION" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
    echo "ERROR: RELEASE must be a SemVer X.Y.Z (got '$NEW_VERSION')" >&2
    exit 1
fi
MAJOR="${BASH_REMATCH[1]}"
MINOR="${BASH_REMATCH[2]}"
PATCH="${BASH_REMATCH[3]}"

# NEXT defaults to RELEASE with the patch incremented.
if [ $# -eq 2 ]; then
    FOLLOWING_BASE="$2"
else
    FOLLOWING_BASE="${MAJOR}.${MINOR}.$((PATCH + 1))"
fi

BRANCH="${BRANCH_OPT:-main}"
FOLLOWING_VERSION="${FOLLOWING_BASE}-dev"

echo "Release version:  $NEW_VERSION"
echo "Next dev version: $FOLLOWING_VERSION"
echo "Branch:           $BRANCH"

# Build gate: backend workspace + the (excluded) frontend WASM crate.
# `cargo build` at the root only covers the backend workspace because
# shifty-dioxus is `exclude`d; the frontend's documented build gate is a
# wasm32 build, so run it explicitly whenever the frontend crate is present.
build_all() {
    cargo build
    if [ -f shifty-dioxus/Cargo.toml ]; then
        ( cd shifty-dioxus && cargo build --target wasm32-unknown-unknown )
    fi
}

# Show subsequent commands being executed
set -x

./update_versions.sh "$NEW_VERSION"
build_all
jj commit -m "Set version to $NEW_VERSION"
jj b m "$BRANCH" --to @-
jj git push
if [ -n "$TAG_MESSAGE" ]; then
    git tag -a "v$NEW_VERSION" -m "$TAG_MESSAGE" "$BRANCH"
else
    git tag -a "v$NEW_VERSION" "$BRANCH"
fi
git push --tags

./update_versions.sh "$FOLLOWING_VERSION"
build_all
jj commit -m "Set version to $FOLLOWING_VERSION"
jj b m "$BRANCH" --to @-
jj git push

echo New release version: $NEW_VERSION
