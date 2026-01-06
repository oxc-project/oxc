<div align="center">

# sort-package-json

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]

[![MIT licensed][license-badge]][license-url]
[![Build Status][ci-badge]][ci-url]
[![Code Coverage][code-coverage-badge]][code-coverage-url]
[![CodSpeed Badge][codspeed-badge]][codspeed-url]
[![Sponsors][sponsors-badge]][sponsors-url]
[![Discord chat][discord-badge]][discord-url]

</div>

A Rust implementation that sorts package.json files according to well-established npm conventions.

> **Note on Compatibility:** This crate is **not compatible** with the original [sort-package-json](https://github.com/keithamus/sort-package-json) npm package. While both tools sort package.json files, this Rust implementation uses different sorting groupings that we believe are clearer and easier to navigate. The field order is inspired by both the original sort-package-json and Prettier's package.json sorting, but organized into more intuitive logical groups.

## Features

- **Sorts top-level fields** according to npm ecosystem conventions (138 predefined fields)
- **Preserves all data** - only reorders fields, never modifies values
- **Fast and safe** - pure Rust implementation with no unsafe code
- **Idempotent** - sorting multiple times produces the same result
- **Handles edge cases** - unknown fields sorted alphabetically, private fields (starting with `_`) sorted last

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
sort-package-json = "0.0.5"
```

### Library API

```rust
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string("package.json")?;
    let sorted = sort_package_json::sort_package_json(&contents)?;
    fs::write("package.json", sorted)?;
    Ok(())
}
```

With custom options:

```rust
use sort_package_json::{sort_package_json_with_options, SortOptions};

let options = SortOptions { pretty: false };
let sorted = sort_package_json_with_options(&contents, &options)?;
```

### Running the Example

To test on a repository, run the included example which recursively finds and sorts all `package.json` files:

```bash
cargo run --example simple [PATH]
```

If no path is provided, it defaults to the current directory.

### Example

Given an unsorted package.json:

```json
{
  "version": "1.0.0",
  "dependencies": { "foo": "1.0.0" },
  "name": "my-package",
  "scripts": { "test": "vitest" }
}
```

After sorting:

```json
{
  "name": "my-package",
  "version": "1.0.0",
  "scripts": { "test": "vitest" },
  "dependencies": { "foo": "1.0.0" }
}
```

## Field Ordering

Fields are sorted into 12 logical groups, followed by unknown fields alphabetically, then private fields (starting with `_`) at the end. The complete field order is based on both the [original sort-package-json](https://github.com/keithamus/sort-package-json/blob/main/index.js) and [prettier's package.json sorting](https://github.com/un-ts/prettier/blob/master/packages/pkg/src/rules/sort.ts) implementations.

```jsonc
{
  // 1. Core Package Metadata
  "$schema": "https://json.schemastore.org/package.json",
  "name": "my-package",
  "displayName": "My Package",
  "version": "1.0.0",
  "private": true,
  "description": "A sample package",
  "categories": ["linters", "formatters"],
  "keywords": ["sample", "test"],
  "homepage": "https://example.com",
  "bugs": { "url": "https://github.com/user/repo/issues", "email": "support@example.com" },

  // 2. License & People
  "license": "MIT",
  "author": { "name": "Author", "email": "author@example.com", "url": "https://example.com" },
  "maintainers": [{ "name": "Maintainer", "email": "maintainer@example.com" }],
  "contributors": [{ "name": "Contributor", "email": "contributor@example.com" }],

  // 3. Repository & Funding
  "repository": { "type": "git", "url": "https://github.com/user/repo.git" },
  "funding": { "type": "github", "url": "https://github.com/sponsors/user" },

  // 4. Package Content & Distribution
  "bin": { "my-cli": "./bin/cli.js" },
  "directories": { "lib": "lib", "bin": "bin", "man": "man", "doc": "doc" },
  "workspaces": ["packages/*"],
  "files": ["dist", "lib", "src/index.js"],
  "os": ["darwin", "linux"],
  "cpu": ["x64", "arm64"],

  // 5. Package Entry Points
  "type": "module",
  "sideEffects": false,
  "main": "./dist/index.cjs",
  "module": "./dist/index.mjs",
  "browser": "./dist/browser.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.mjs",
      "require": "./dist/index.cjs",
      "default": "./dist/index.mjs",
    },
  },
  "publishConfig": { "access": "public", "registry": "https://registry.npmjs.org" },

  // 6. Scripts
  "scripts": {
    "build": "tsup",
    "test": "vitest",
    "lint": "eslint .",
  },

  // 7. Dependencies
  "dependencies": { "lodash": "^4.17.21" },
  "devDependencies": { "typescript": "^5.0.0", "vitest": "^1.0.0" },
  "peerDependencies": { "react": ">=18.0.0" },
  "peerDependenciesMeta": { "react": { "optional": true } },
  "optionalDependencies": { "fsevents": "^2.3.0" },
  "bundledDependencies": ["internal-lib"],
  "overrides": { "semver": "^7.5.4" },

  // 8. Git Hooks & Commit Tools
  "simple-git-hooks": { "pre-commit": "npx lint-staged" },
  "lint-staged": { "*.ts": ["eslint --fix", "prettier --write"] },
  "commitlint": { "extends": ["@commitlint/config-conventional"] },

  // 9. VSCode Extension Specific
  "contributes": { "commands": [] },
  "activationEvents": ["onLanguage:javascript"],
  "icon": "icon.png",

  // 10. Build & Tool Configuration
  "browserslist": ["> 1%", "last 2 versions"],
  "prettier": { "semi": false, "singleQuote": true },
  "eslintConfig": { "extends": ["eslint:recommended"] },

  // 11. Testing
  "jest": { "testEnvironment": "node" },
  "c8": { "include": ["src/**"] },

  // 12. Runtime & Package Manager
  "engines": { "node": ">=18.0.0" },
  "packageManager": "pnpm@8.0.0",
  "pnpm": { "overrides": {} },

  // Unknown fields (sorted alphabetically)
  "customField": "value",
  "myConfig": {},

  // Private fields (sorted alphabetically, always last)
  "_internal": "hidden",
  "_private": "data",
}
```

## Why Not simd-json?

We use serde_json instead of [simd-json](https://github.com/simd-lite/simd-json) because:

- **No preserve_order support** - simd-json can't maintain custom field insertion order (required for our sorting)
- **Platform issues** - simd-json doesn't work on big-endian architectures ([#437](https://github.com/simd-lite/simd-json/issues/437))

## Development

### Building

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

Tests use snapshot testing via [insta](https://insta.rs/). To review and accept snapshot changes:

```bash
cargo insta review
```

Or to accept all changes:

```bash
cargo insta accept
```

### Test Coverage

- **Field ordering test** - verifies correct sorting of all field types
- **Idempotency test** - ensures sorting is stable (sorting twice = sorting once)

## License

MIT

## References

- [Original sort-package-json (JavaScript)](https://github.com/keithamus/sort-package-json)
- [simd-json issue #437 - Big Endian Compatibility](https://github.com/simd-lite/simd-json/issues/437)
- [Surprises in the Rust JSON Ecosystem](https://ecton.dev/rust-json-ecosystem/)

## [Sponsored By](https://github.com/sponsors/Boshen)

<p align="center">
  <a href="https://github.com/sponsors/Boshen">
    <img src="https://raw.githubusercontent.com/Boshen/sponsors/main/sponsors.svg" alt="My sponsors" />
  </a>
</p>

[discord-badge]: https://img.shields.io/discord/1079625926024900739?logo=discord&label=Discord
[discord-url]: https://discord.gg/9uXCAwqQZW
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-url]: https://github.com/oxc-project/sort-package-json/blob/main/LICENSE
[ci-badge]: https://github.com/oxc-project/sort-package-json/actions/workflows/ci.yml/badge.svg?event=push&branch=main
[ci-url]: https://github.com/oxc-project/sort-package-json/actions/workflows/ci.yml?query=event%3Apush+branch%3Amain
[code-coverage-badge]: https://codecov.io/github/oxc-project/sort-package-json/branch/main/graph/badge.svg
[code-coverage-url]: https://codecov.io/gh/oxc-project/sort-package-json
[sponsors-badge]: https://img.shields.io/github/sponsors/Boshen
[sponsors-url]: https://github.com/sponsors/Boshen
[codspeed-badge]: https://img.shields.io/endpoint?url=https://codspeed.io/badge.json
[codspeed-url]: https://codspeed.io/oxc-project/sort-package-json
[crates-badge]: https://img.shields.io/crates/d/sort-package-json?label=crates.io
[crates-url]: https://crates.io/crates/sort-package-json
[docs-badge]: https://img.shields.io/docsrs/sort-package-json
[docs-url]: https://docs.rs/sort-package-json
