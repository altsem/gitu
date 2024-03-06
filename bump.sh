#!/bin/sh

set -e

version=$(git cliff --bumped-version)

cargo set-version "$(echo "$version" | sed s/^v//)"
git cliff --tag "$version" > CHANGELOG.md

git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore(release): prepare for $version"
git tag "$version"
