#!/usr/bin/env bash

# Check if version argument is provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 <new_version>"
    echo "Example: $0 0.12.0"
    exit 1
fi

NEW_VERSION=$1

# Update version in each Cargo.toml file
for cargo_file in */Cargo.toml; do
    if [ -f "$cargo_file" ]; then
        echo "Updating version in $cargo_file"
        # Use sed to replace only the first occurrence of version line
        sed -i "0,/^version = \".*\"/{s/^version = \".*\"/version = \"$NEW_VERSION\"/}" "$cargo_file"
        sed -i "0,/^version = .*/{s/^version = .*/version = \"$NEW_VERSION\"/}" "$cargo_file"
    fi
done

sed -i "0,/version = \".*\"/{s/version = \".*\"/version = \"$NEW_VERSION\"/}" "default.nix"

echo "Version update complete"
