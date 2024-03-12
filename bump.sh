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

git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore(release): prepare for $version"
git -c core.commentChar="@" tag -am "$(git cliff --latest --strip header)" "$version"
