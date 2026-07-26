#!/usr/bin/env bash
set -euo pipefail

# Builds ./oxfmt.tgz — a wasi-only oxfmt npm package for the WebContainer PoC.
# See _design-oxfmt-wasi-playground.md「再現手順」/「PoC の再現手順」.
#
# The manual steps here (copying the wasi trio into dist, adding the
# @napi-rs/wasm-runtime dependency) disappear once Phase 1 lands:
# scripts/build.js will copy the wasi artifacts, and the runtime dependency
# will be declared by the @oxfmt/binding-wasm32-wasi package instead.

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
OXFMT="$ROOT/apps/oxfmt"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

# `--config.verify-deps-before-run=false`: skip pnpm's pre-run install check,
# which can fail on unrelated registry issues.
PNPM="pnpm --config.verify-deps-before-run=false"

echo "[1/6] wasi release build (bin target skipped via --no-default-features --features napi)"
$PNPM --dir "$OXFMT" build-napi-wasi --release

echo "[2/6] JS dist build"
$PNPM --dir "$OXFMT" build-js

echo "[3/6] copy wasi trio into dist (scripts/build.js only copies .node)"
cp "$OXFMT/src-js/oxfmt.wasi.cjs" "$OXFMT/src-js/wasi-worker.mjs" \
  "$OXFMT/src-js/oxfmt.wasm32-wasi.wasm" "$OXFMT/dist/"
# The glue prefers the debug wasm when present — never ship it.
rm -f "$OXFMT/dist/oxfmt.wasm32-wasi.debug.wasm"

echo "[4/6] stage publishable layout (npm/oxfmt is the package.json template)"
cp "$ROOT/npm/oxfmt/package.json" "$ROOT/npm/oxfmt/configuration_schema.json" "$STAGE/"
cp -R "$ROOT/npm/oxfmt/bin" "$STAGE/"
cp -R "$OXFMT/dist" "$STAGE/dist"
rm -f "$STAGE"/dist/*.node # wasi-only package: no native bindings

echo "[5/6] add @napi-rs/wasm-runtime dependency (required by oxfmt.wasi.cjs at runtime)"
node -e '
const fs = require("node:fs");
const p = process.argv[1];
const pkg = JSON.parse(fs.readFileSync(p, "utf8"));
pkg.dependencies["@napi-rs/wasm-runtime"] = "^1.1.6";
fs.writeFileSync(p, JSON.stringify(pkg, null, 2));
' "$STAGE/package.json"

echo "[6/6] npm pack"
(cd "$STAGE" && npm pack --quiet)
mv "$STAGE"/oxfmt-*.tgz "$HERE/oxfmt.tgz"

echo "wrote $HERE/oxfmt.tgz ($(du -h "$HERE/oxfmt.tgz" | cut -f1))"
