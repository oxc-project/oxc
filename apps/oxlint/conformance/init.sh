#!/bin/bash
set -e

ESLINT_SHA="8f360ad6a7a743d33a83eed8973ee4a50731e55b" # 10.0.0-rc.0
REACT_SHA="612e371fb215498edde4c853bd1e0c8e9203808f" # 19.2.3

# Shallow clone a repo at a specific commit.
# Git commands copied from `.github/scripts/clone-parallel.mjs`.
clone() {
  local dir="$1"
  local url="$2"
  local ref="$3"

  git clone --single-branch --depth 1 "$url" "$dir"
  cd "$dir"
  git fetch --quiet --depth 1 origin "$ref"
  git reset --hard "$ref"
  git clean -f -q
}

# Delete existing `submodules` directory
rm -rf submodules
mkdir submodules
cd submodules

###############################################################################
# ESLint
###############################################################################

# Clone ESLint repo into `submodules/eslint`
clone eslint https://github.com/eslint/eslint.git "$ESLINT_SHA"

# Install dependencies
pnpm install --ignore-workspace

# Copy TS-ESLint parser shim into `node_modules/@typescript-eslint/parser`
rm node_modules/@typescript-eslint/parser
cp -r tools/typescript-eslint-parser node_modules/@typescript-eslint/parser
cd node_modules/@typescript-eslint/parser

# Install dependencies of TS-ESLint parser shim
pnpm install --ignore-workspace

# Return to `submodules` directory
cd ../../../..

###############################################################################
# React
###############################################################################

# Clone React repo into `submodules/react`
clone react https://github.com/facebook/react.git "$REACT_SHA"

# Install dependencies
yarn

# Install `eslint-plugin-react-hooks` dependency
cd packages/eslint-plugin-react-hooks
yarn add eslint-plugin-react-hooks

# Return to `submodules` directory
cd ../../..
