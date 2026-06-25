#!/usr/bin/env bash

set -euo pipefail

# Usage:
#   cli-update-version.sh                            # auto-derive YEAR.DAY.COUNTER from date + Cargo.toml
#   cli-update-version.sh <RELEASE> <NEXT> [BRANCH]  # explicit (legacy mode)
#
# Auto-derive rules (no args):
#   YEAR = current year
#   DAY  = day-of-year (1..366, leading zeros stripped)
#   COUNTER is read from the current version in Cargo.toml:
#     - current matches YYYY.DDD.X-dev and YYYY.DDD == today -> release X,    next X+1-dev
#     - current matches YYYY.DDD.X     and YYYY.DDD == today -> release X+1,  next X+2-dev
#     - otherwise                                            -> release 0,    next 1-dev
# The release is tagged on $BRANCH (default "main"); the -dev bump lands on $BRANCH afterwards.

if [ $# -eq 0 ]; then
    YEAR=$(date +%Y)
    DAY=$(date +%j | sed 's/^0*//')
    [ -z "$DAY" ] && DAY=0

    # Read current version from the workspace's main Cargo.toml.
    # Backend layout has shifty_bin/Cargo.toml; the frontend has Cargo.toml at script CWD.
    if [ -f shifty_bin/Cargo.toml ]; then
        VERSION_FILE="shifty_bin/Cargo.toml"
    elif [ -f Cargo.toml ]; then
        VERSION_FILE="Cargo.toml"
    else
        echo "ERROR: cannot find a Cargo.toml to read current version from" >&2
        exit 1
    fi
    CURRENT=$(grep -m1 '^version = ' "$VERSION_FILE" | sed -E 's/^version = "(.*)"$/\1/')

    if [[ "$CURRENT" =~ ^([0-9]{4})\.([0-9]+)\.([0-9]+)(-dev)?$ ]]; then
        CUR_YEAR="${BASH_REMATCH[1]}"
        CUR_DAY="${BASH_REMATCH[2]}"
        CUR_COUNTER="${BASH_REMATCH[3]}"
        HAS_DEV="${BASH_REMATCH[4]:-}"
        if [ "$CUR_YEAR" = "$YEAR" ] && [ "$CUR_DAY" = "$DAY" ]; then
            if [ -n "$HAS_DEV" ]; then
                COUNTER="$CUR_COUNTER"
            else
                COUNTER=$((CUR_COUNTER + 1))
            fi
        else
            COUNTER=0
        fi
    else
        COUNTER=0
    fi

    NEXT_COUNTER=$((COUNTER + 1))
    NEW_VERSION="${YEAR}.${DAY}.${COUNTER}"
    FOLLOWING_BASE="${YEAR}.${DAY}.${NEXT_COUNTER}"
    BRANCH="main"

    echo "Auto-derived release version: $NEW_VERSION"
    echo "Auto-derived next dev version: ${FOLLOWING_BASE}-dev"
    echo "Branch: $BRANCH"
else
    NEW_VERSION="$1"
    FOLLOWING_BASE="${2}"
    BRANCH="${3:-main}"
fi

FOLLOWING_VERSION="${FOLLOWING_BASE}-dev"

# Show subsequent commands being executed
set -x

./update_versions.sh "$NEW_VERSION"
cargo build
jj commit -m "Set version to $NEW_VERSION"
jj b m "$BRANCH" --to @-
jj git push
git tag -a "v$NEW_VERSION" "$BRANCH"
git push --tags

./update_versions.sh "$FOLLOWING_VERSION"
cargo build
jj commit -m "Set version to $FOLLOWING_VERSION"
jj b m "$BRANCH" --to @-
jj git push

echo New release version: $NEW_VERSION
