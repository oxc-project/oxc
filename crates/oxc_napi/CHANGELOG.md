# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).











## [0.80.0] - 2025-08-03

### 📚 Documentation

- de1de35 rust: Add comprehensive README.md documentation for all Rust crates (#12706) (Copilot)








## [0.77.0] - 2025-07-12

### 🚀 Features

- 407429a napi/parser,napi/transform: Accept `lang=dts` (#12154) (Boshen)









# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.70.0] - 2025-05-15

### Bug Fixes

- 635aa96 napi: Computed final source type from `lang` then `sourceType` (#11060) (Boshen)

## [0.68.0] - 2025-05-03

### Refactor

- 06fde2a ast/estree: `convert_utf8_to_utf16` take `&mut` slice of errors, not `&mut Vec` (#10672) (overlookmotel)

## [0.61.0] - 2025-03-20

### Features

- 2cedfe4 napi: Add codeframe to napi error (#9893) (Boshen)

### Refactor

- 961b95d napi: Move common code to `oxc_napi` (#9875) (Boshen)

## [0.50.0] - 2025-02-12

### Bug Fixes

- ad93ece oxc_napi: Add napi build.rs (#9038) (LongYinan)

## [0.49.0] - 2025-02-10

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

## [0.42.0] - 2024-12-18

### Refactor

- 3858221 global: Sort imports (#7883) (overlookmotel)

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.40.1] - 2024-12-10

### Bug Fixes

- 18d0ce3 napi: Rename `Error` to `OxcError` to avoid name collision (#7780) (Boshen)

## [0.40.0] - 2024-12-10

### Features

- 85eec3c napi/transform,napi/parser: Return structured error object (#7724) (Boshen)

