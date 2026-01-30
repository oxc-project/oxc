#!/bin/bash
set -e

ESLINT_SHA="8f360ad6a7a743d33a83eed8973ee4a50731e55b" # 10.0.0-rc.0
REACT_SHA="612e371fb215498edde4c853bd1e0c8e9203808f" # 19.2.3
STYLISTIC_SHA="5c4b512a225a314fa5f41eead9fdc4d51fc243d7" # 5.7.1
SONAR_SHA="8852e2593390e00f9d9aea764b0b0b9a503d1f08" # 3.0.6

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

###############################################################################
# Stylistic
###############################################################################

# Clone ESLint Stylistic repo into `submodules/stylistic`
clone stylistic https://github.com/eslint-stylistic/eslint-stylistic.git "$STYLISTIC_SHA"

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
clone sonarjs https://github.com/SonarSource/SonarJS.git "$SONAR_SHA"

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
