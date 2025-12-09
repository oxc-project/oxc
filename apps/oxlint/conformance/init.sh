#!/bin/bash
set -e

ESLINT_SHA="e593aa0fd29f51edea787815ffc847aa723ef1f8"

# Delete existing `submodules` directory
rm -rf submodules

# Clone ESLint repo into `submodules/eslint`
git clone --single-branch --depth 1 https://github.com/eslint/eslint.git submodules/eslint
cd submodules/eslint
git fetch --depth 1 origin "$ESLINT_SHA"
git reset --hard "$ESLINT_SHA"
git clean -f -q

# Install dependencies
pnpm install --ignore-workspace

# Copy TS-ESLint parser shim into `node_modules/@typescript-eslint/parser`
rm node_modules/@typescript-eslint/parser
cp -r tools/typescript-eslint-parser node_modules/@typescript-eslint/parser
cd node_modules/@typescript-eslint/parser

# Install dependencies of TS-ESLint parser shim
pnpm install --ignore-workspace
