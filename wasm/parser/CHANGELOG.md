# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.53.0] - 2025-02-26

### Features

- 835ee95 wasm: Return estree with utf16 span offsets (#9376) (Boshen)

### Refactor

- b09249c ast/estree: Rename serializers and serialization methods (#9284) (overlookmotel)

## [0.52.0] - 2025-02-21

- 216b33f ast/estree: [**BREAKING**] Replace `serde` with custom `ESTree` serializer (#9256) (overlookmotel)

### Features

- ac1f622 wasm/parser: Expose comments (#9175) (Kevin Deng 三咲智子)

### Bug Fixes

- b9c8a10 wasm: Transfer AST to JS as JSON string in `oxc-wasm` (#9269) (overlookmotel)
- 5acc6ec wasm: Transfer AST to JS as JSON string (#9259) (overlookmotel)

### Documentation

- d04c4b0 wasm: Correct and update docs (#9260) (overlookmotel)

## [0.49.0] - 2025-02-10

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

## [0.42.0] - 2024-12-18

### Refactor

- 8cf9766 semantic, syntax, wasm: Remove `#![expect(non_snake_case)]` (#7863) (overlookmotel)

## [0.35.0] - 2024-11-04

### Bug Fixes

- 7d12669 types: Move @oxc-project/types to dependencies (#6909) (ottomated)

## [0.34.0] - 2024-10-26

### Features

- 1145341 ast_tools: Output typescript to a separate package (#6755) (ottomated)

### Bug Fixes

- b075982 types: Change @oxc/types package name (#6874) (ottomated)

## [0.26.0] - 2024-09-03

### Refactor

- b39c0d6 wasm: Add `source_type` for parser, replace class options with plain object (#5217) (Kevin Deng 三咲智子)

## [0.13.0] - 2024-05-14

### Refactor

- 2064ae9 parser,diagnostic: One diagnostic struct to eliminate monomorphization of generic types (#3214) (Boshen)

## [0.10.0] - 2024-03-14

### Features
- 697b6b7 Merge features `serde` and `wasm` to `serialize` (#2716) (Boshen)

## [0.6.0] - 2024-02-03

### Features

- 9c6c17b wasm/parser: Improve FFI (#2232) (Boshen)- 5ac61f0 Setup wasm parser for npm (#2221) (Boshen)

