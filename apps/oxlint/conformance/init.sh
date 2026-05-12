#!/bin/bash
set -e

# Path to repos.json (resolved before any `cd` changes the working directory)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPOS_JSON="$SCRIPT_DIR/repos.json"

# Read a field from repos.json for a given repo key.
# Usage: repo_field <key> <field>
# e.g. `repo_field eslint commitSha`
repo_field() {
  node -p "require('$REPOS_JSON')['$1'].$2"
}

# Shallow clone a repo at a specific commit, and `cd` into the cloned directory.
# Reads the repo URL and commit SHA from repos.json using the given key.
# Git commands copied from `.github/scripts/clone-parallel.mjs`.
clone_repo() {
  local dir="$1"
  local url="$(repo_field "$dir" repoUrl).git"
  local ref="$(repo_field "$dir" commitSha)"

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
clone_repo eslint

# Install dependencies
pnpm install --ignore-workspace

# Return to `submodules` directory
cd ..

###############################################################################
# React
###############################################################################

# Clone React repo into `submodules/react`
clone_repo react

# Install dependencies
yarn

# Install `eslint-plugin-react-hooks` dependency
cd packages/eslint-plugin-react-hooks
yarn add eslint-plugin-react-hooks
cd ../..

# @overlookmotel says: @camc314 added the next block to this script, but it doesn't seem to work on my machine.
# Presumably it's because we're using different versions of `yarn`, but I can't track down the problem exactly.
# So I'm commenting it out again for now.
# @connorshea says: I also ran into the problem cam had, I'm using yarn v1 via homebrew.

# Ensure `eslint-plugin-react-hooks` can be resolved from the React tests directory.
# In recent React workspace setups this is already satisfied after `yarn`, and forcing
# `yarn add` here can fail with workspace invariant errors.
# if ! node -e "require.resolve('eslint-plugin-react-hooks/package.json')" >/dev/null 2>&1; then
#   cd packages/eslint-plugin-react-hooks
#   yarn add eslint-plugin-react-hooks
#   cd ../..
# fi

# Return to `submodules` directory
cd ..

###############################################################################
# Stylistic
###############################################################################

# Clone ESLint Stylistic repo into `submodules/stylistic`
clone_repo stylistic

# Install dependencies.
# No `--ignore-workspace` because `eslint-stylistic` has its own `pnpm-workspace.yaml`.
pnpm install

# Patch `package.json` files to add Node.js subpath imports.
# ESLint Stylistic uses TypeScript `paths` in `tsconfig.base.json` (e.g. `#test`),
# but Node/tsx doesn't respect tsconfig paths. It needs `imports` in `package.json`.
#
# Read path aliases from `tsconfig.base.json` and add them to `package.json` as `imports`.
node -e '
const fs = require("fs");
const path = require("path");

const pluginPkgPath = path.resolve("packages/eslint-plugin/package.json");
const pluginPkg = JSON.parse(fs.readFileSync(pluginPkgPath, "utf8"));

// Read path aliases from `tsconfig.base.json`
const tsconfig = JSON.parse(fs.readFileSync("tsconfig.base.json", "utf8"));
const tsPaths = tsconfig.compilerOptions.paths;

// Convert tsconfig paths format to package.json imports format
// tsconfig: { "#test": ["./shared/test-utils/index.ts"] }
// package.json: { "#test": "./shared/test-utils/index.ts" }
pluginPkg.imports = Object.fromEntries(
  Object.entries(tsPaths).map(([alias, targets]) => [alias, targets[0]]),
);

fs.writeFileSync(pluginPkgPath, JSON.stringify(pluginPkg, null, 2) + "\n");
'

# Node.js resolves `imports` from the nearest `package.json` with a `name` field,
# and subpath imports can only reference files within the package (no `../` allowed).
# So create a symlink to the `shared` directory within the `eslint-plugin` package.
ln -s ../../shared packages/eslint-plugin/shared

# Replace top-level await imports in `parsers-jsx.ts` with regular imports.
# `tsx` does not support TLA with CJS output format.
if [[ "$OSTYPE" == darwin* ]]; then
  sed -i '' '/^export const.*= await import/d' shared/test-utils/parsers-jsx.ts
else
  sed -i '/^export const.*= await import/d' shared/test-utils/parsers-jsx.ts
fi

cat >> shared/test-utils/parsers-jsx.ts << 'EOF'
import BABEL_ESLINT from '@babel/eslint-parser';
import TYPESCRIPT_ESLINT from '@typescript-eslint/parser';
export { BABEL_ESLINT, TYPESCRIPT_ESLINT };
EOF

# Return to `submodules` directory
cd ..

###############################################################################
# SonarJS
###############################################################################

# Clone SonarJS repo into `submodules/sonarjs`
clone_repo sonarjs

# Install dependencies
pnpm install --ignore-workspace

# Build
# (ignore errors, it's just typecheck fail)
pnpm run bbf || true

# The tests use `describe` and `it` from `node:test`, but we just need to use global `describe`,
# and make `it` behave like `describe`
PATTERN="s/import .* from 'node:test';/const it = describe;/"
if [[ "$OSTYPE" == darwin* ]]; then
  find packages/jsts/src/rules -name '*.test.ts' -exec sed -i '' "$PATTERN" {} \;
else
  find packages/jsts/src/rules -name '*.test.ts' -exec sed -i "$PATTERN" {} \;
fi

# Replace `import.meta.dirname` with `__dirname` (`import.meta.dirname` doesn't work after `tsx` transforms to CommonJS)
PATTERN2="s/import\.meta\.dirname/__dirname/g"
TESTER_PATH="packages/jsts/tests/tools/testers/rule-tester.ts"
if [[ "$OSTYPE" == darwin* ]]; then
  find packages/jsts/src/rules -name '*.test.ts' -exec sed -i '' "$PATTERN2" {} \;
  sed -i '' "$PATTERN2" "$TESTER_PATH"
else
  find packages/jsts/src/rules -name '*.test.ts' -exec sed -i "$PATTERN2" {} \;
  sed -i "$PATTERN2" "$TESTER_PATH"
fi

# Replace `import { it } from 'node:test';` with `const it = describe;` in comment-based checker
PATTERN3="s/import .* from 'node:test';/const it = describe;/"
CHECKER_PATH="packages/jsts/tests/tools/testers/comment-based/checker.ts"
if [[ "$OSTYPE" == darwin* ]]; then
  sed -i '' "$PATTERN3" "$CHECKER_PATH"
else
  sed -i "$PATTERN3" "$CHECKER_PATH"
fi

# Return to `submodules` directory
cd ..

###############################################################################
# E18E
###############################################################################

# Clone E18E ESLint plugin repo into `submodules/e18e`
clone_repo e18e

# Install dependencies
pnpm install --ignore-workspace

# Return to `submodules` directory
cd ..

###############################################################################
# Testing Library
###############################################################################

# Clone `eslint-plugin-testing-library` repo into `submodules/testing_library`
clone_repo testing_library

# Install dependencies
pnpm install --ignore-workspace

# Return to `submodules` directory
cd ..

###############################################################################
# Storybook
###############################################################################

# Clone `eslint-plugin-storybook` repo into `submodules/storybook`
clone_repo storybook

# Install dependencies
yarn install

# Return to `submodules` directory
cd ..

###############################################################################
# Playwright
###############################################################################

# Clone `eslint-plugin-playwright` repo into `submodules/playwright`
clone_repo playwright

# Install dependencies
yarn install

# Return to `submodules` directory
cd ..

###############################################################################
# Cypress
###############################################################################

# Clone `eslint-plugin-cypress` repo into `submodules/cypress`
clone_repo cypress

# Install dependencies
npm install

# Return to `submodules` directory
cd ..

###############################################################################
# Mocha
###############################################################################

# Clone `eslint-plugin-mocha` repo into `submodules/mocha`
clone_repo mocha

# Install dependencies
npm install

# Return to `submodules` directory
cd ..

###############################################################################
# Regexp
###############################################################################

# Clone `eslint-plugin-regexp` repo into `submodules/regexp`
clone_repo regexp

# Install dependencies
npm install

# Return to `submodules` directory
cd ..
