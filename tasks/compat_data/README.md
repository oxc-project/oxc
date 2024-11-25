# Compat Data

Get engine compatibility Data from https://github.com/compat-table/compat-table/

Code extracted from https://github.com/babel/babel/tree/v7.26.2/packages/babel-compat-data

## Adding a new feature

- Find the feature from https://github.com/compat-table/compat-table/blob/gh-pages/data-es2016plus.js
- Add the feature in `./es-features.js`
- `pnpm install`
- `cargo run -p oxc_compat_data`
