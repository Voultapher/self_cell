#!/usr/bin/env bash

set -euxo pipefail

NEW_VERSION="${1}"

# Ensure Cargo.toml has been updated.
CARGO_TOML_VERSION=$(grep "${NEW_VERSION}" Cargo.toml)

# Verify we are on main
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "${CURRENT_BRANCH}" != "main" ]; then
    echo "Not on main"
    exit 1
fi

git fetch origin

# DIFF_TO_ORIGIN=$(git diff origin/main)
# if [ "${DIFF_TO_ORIGIN}" != "" ]; then
#     echo "Out of sync with origin/main"
#     exit 1
# fi

# Run tests as sanity, nothing should be released which doesn't pass CI.
#./test.sh

git tag "v${NEW_VERSION}"
git push --tags

cargo publish