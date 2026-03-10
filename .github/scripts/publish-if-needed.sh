#!/usr/bin/env bash
# Publish an npm package only if its version is not already on the registry.
# Usage: bash .github/scripts/publish-if-needed.sh <pkg_dir> [publish_flags...]
# Example: bash .github/scripts/publish-if-needed.sh npm/oxlint --provenance --access public --no-git-checks

set -euo pipefail

pkg_dir="$1"
shift

read -r name version < <(node -e "const p=require('./${pkg_dir}/package.json'); console.log(p.name+' '+p.version)")

npm_stderr=$(mktemp)
if published=$(npm view "${name}@${version}" version 2>"$npm_stderr"); then
  if [ "$published" = "$version" ]; then
    rm -f "$npm_stderr"
    echo "⏭ ${name}@${version} already published, skipping."
    exit 0
  fi
else
  # npm view failed — log the error but continue to publish (which will
  # produce a clear error if the registry is actually unreachable).
  echo "⚠ npm view ${name}@${version} failed:"
  cat "$npm_stderr" >&2
fi
rm -f "$npm_stderr"

pnpm publish "${pkg_dir}/" "$@"
