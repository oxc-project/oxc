# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.30.0] - 2024-09-23

### Testing

- 84b7d1a index: Add unit tests to `oxc_index` (#5979) (DonIsaac)

## [0.29.0] - 2024-09-13

- 71116a1 index: [**BREAKING**] Remove ability to index `IndexVec` with `usize` (#5733) (overlookmotel)

### Features

- a362f51 index: Add `IndexVec::shrink_to` (#5713) (overlookmotel)

### Performance

- 333e2e0 index: Remove `Idx` bounds-checks from `first` + `last` methods (#5726) (overlookmotel)

## [0.28.0] - 2024-09-11

### Refactor

- 2de6ea0 index, traverse: Remove unnecessary type annotations (#5650) (overlookmotel)- 26d9235 Enable clippy::ref_as_ptr  (#5577) (夕舞八弦)

## [0.27.0] - 2024-09-06

### Features

- 4cb63fe index: Impl rayon related to trait for IndexVec (#5421) (IWANABETHATGUY)

### Documentation
- 00511fd Use `oxc_index` instead of `index_vec` in doc comments (#5423) (IWANABETHATGUY)

## [0.24.3] - 2024-08-18

### Refactor

- 786bf07 index: Shorten code and correct comment (#4905) (overlookmotel)

## [0.13.0] - 2024-05-14

### Bug Fixes

- 51de41c index: Add `example_generated` to create the docs. (#3106) (Ali Rezvani)

## [0.10.0] - 2024-03-14

### Features
- 697b6b7 Merge features `serde` and `wasm` to `serialize` (#2716) (Boshen)

## [0.5.0] - 2024-01-12

### Features

- f1b433b playground: Visualize symbol (#1886) (Dunqing)

## [0.4.0] - 2023-12-08

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

