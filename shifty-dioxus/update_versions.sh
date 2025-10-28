#!/usr/bin/env bash

# Check if version argument is provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 <new_version>"
    echo "Example: $0 0.12.0"
    exit 1
fi

NEW_VERSION=$1


sed -i "0,/version = \".*\"/{s/version = \".*\"/version = \"$NEW_VERSION\"/}" "Cargo.toml"
sed -i "0,/version = \".*\"/{s/version = \".*\"/version = \"$NEW_VERSION\"/}" "default.nix"
sed -i "0,/version = \".*\"/{s/version = \".*\"/version = \"$NEW_VERSION\"/}" "flake.nix"

echo "Version update complete"
