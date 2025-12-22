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
