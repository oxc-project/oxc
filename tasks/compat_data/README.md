# Compat Data

Get engine compatibility data from

- https://github.com/compat-table/compat-table/
- https://github.com/compat-table/node-compat-table

Code extracted from

- https://github.com/babel/babel/tree/v7.26.2/packages/babel-compat-data
- https://github.com/evanw/esbuild/blob/v0.27.2/compat-table/src/kangax.ts

## Adding a new feature

- Find the feature from https://github.com/compat-table/compat-table/blob/gh-pages/data-es2016plus.js
- Add the feature in `./es-features.js`
- `pnpm install`
- `cargo run -p oxc_compat_data`

For a proposal without an ES edition, add it to `custom-compat-data.js` with
`es: null` and only the engines that support it. An empty `targets` object requires
lowering for concrete targets. An unspecified target and `esnext` preserve proposals,
while `supports_es_feature` reports no known support.

Use `cargo run -p oxc_compat_data -- --generate-only` to regenerate the Rust table
from the checked-in `data.json` without refreshing upstream compatibility data.
Run `cargo fmt` afterward to match the checked-in Rust formatting.
