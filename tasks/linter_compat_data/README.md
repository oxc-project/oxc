# linter_compat_data

Generates `crates/oxc_linter/src/utils/compat/compat_data.json`, the embedded
browser-compatibility dataset used by the `compat/compat` lint rule (a native
port of [eslint-plugin-compat](https://github.com/amilajack/eslint-plugin-compat)).

The dataset combines:

- **MDN data** via [`ast-metadata-inferer`](https://www.npmjs.com/package/ast-metadata-inferer)
  (itself derived from `@mdn/browser-compat-data`): per-API `protoChainId`,
  AST node types, ES-vs-web kind, and per-browser `version_added`.
- **caniuse-lite** feature tables for the features referenced by
  eslint-plugin-compat's CanIUse provider (fetch, promises, serviceworkers, ...).
- The [`globals`](https://www.npmjs.com/package/globals) package's browser
  globals list (used for case-insensitive lookups of browser globals in member
  expressions).

## Regenerating

```bash
cd tasks/linter_compat_data
pnpm install
node generate.mjs
```

Dependency versions are pinned to keep the dataset (and the lint rule's test
expectations) reproducible. Bumping them refreshes the compat data and may
require updating test expectations in
`crates/oxc_linter/src/rules/compat/compat.rs`.
