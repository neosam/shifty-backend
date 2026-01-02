#!/usr/bin/env bash

set -euo pipefail

# Show echo commands being executed (for debugging)
set -x

NEW_VERSION="$1"
FOLLOWING_VERSION="${2}-dev"
BRANCH="${3:-main}"

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

