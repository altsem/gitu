#!/bin/sh

set -e

version=$(git cliff --bumped-version)
if git rev-parse "refs/tags/$version" >/dev/null 2>&1
then
  echo "tag $version exists"
  exit 1
fi

cargo set-version "$(echo "$version" | sed s/^v//)"
git cliff --tag "$version" > CHANGELOG.md
git cliff --unreleased --tag "$version" --strip header > .recent-changelog-entry

git add Cargo.toml Cargo.lock CHANGELOG.md .recent-changelog-entry
git commit --no-verify -m "chore(release): prepare for $version"
git tag -am "$version" "$version"
