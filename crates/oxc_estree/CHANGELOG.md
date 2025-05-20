# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.71.0] - 2025-05-20

### Features

- 9e90e00 ast_tools: Introduce `#[js_only]` attr for struct fields and converters (#11160) (overlookmotel)
- c79a7d0 data_structures: Introduce `PointerExt` trait (#11095) (overlookmotel)

### Performance

- 8f8d823 ast/estree: Optimize serializing strings to JSON (#11070) (overlookmotel)

## [0.69.0] - 2025-05-09

### Features

- d066516 ast_tools: Support `#[estree(prepend_to)]` (#10849) (overlookmotel)

### Performance

- 49a6f97 napi/parser: Faster fixup of `BigInt`s and `RegExp`s (#10820) (overlookmotel)

### Refactor

- 5645684 ast/estree: Print header and footer on JSON AST with fixes on separate lines (#10869) (overlookmotel)
- b16331e ast/estree: Generalize concatenating fields with `Concat2` (#10848) (overlookmotel)

## [0.63.0] - 2025-04-08

### Performance

- b5f8e38 ast/estree: Faster checking if bytes are ASCII (#10183) (overlookmotel)

## [0.62.0] - 2025-04-01

### Bug Fixes

- f0e1510 parser: Store lone surrogates as escape sequence (#10041) (overlookmotel)

## [0.61.2] - 2025-03-23

### Bug Fixes

- 8228b74 ast/estree: Fix `Function.this_param` (#9913) (hi-ogawa)

### Refactor

- dc3e725 ast/estree: Expose `INCLUDE_TS_FIELDS` constant on `Serializer` (#9943) (overlookmotel)

## [0.60.0] - 2025-03-18

- b3ce925 data_structures: [**BREAKING**] Put all parts behind features (#9849) (overlookmotel)

### Features


## [0.54.0] - 2025-03-04

### Performance

- b0a0a82 ast/estree: Reduce overhead serializing static strings (#9396) (overlookmotel)

## [0.53.0] - 2025-02-26

### Features

- 5c775ea ast/estree: Enable serialization without TS fields (#9285) (overlookmotel)

### Performance

- 1bfc459 ast/estree: Pre-allocate `CodeBuffer` for JSON output (#9340) (overlookmotel)
- 018c523 ast/estree: `ESTree` serializer use `CodeBuffer` (#9331) (overlookmotel)

### Refactor

- b09249c ast/estree: Rename serializers and serialization methods (#9284) (overlookmotel)
- 2faabe1 estree: Make `itoa` dependency optional (#9338) (overlookmotel)

## [0.52.0] - 2025-02-21

- 216b33f ast/estree: [**BREAKING**] Replace `serde` with custom `ESTree` serializer (#9256) (overlookmotel)

### Features


## [0.49.0] - 2025-02-10

### Bug Fixes

- 7e6a537 ast: Include `directives` in `body` (#8981) (hi-ogawa)

## [0.36.0] - 2024-11-09

- 092de67 types: [**BREAKING**] Append `rest` field into `elements` for objects and arrays to align with estree (#7212) (ottomated)

### Features

- dc0215c ast_tools: Add #[estree(append_to)], remove some custom serialization code (#7149) (ottomated)

### Bug Fixes


## [0.32.0] - 2024-10-19

### Features

- e310e52 parser: Generate `Serialize` impls in ast_tools (#6404) (ottomated)

