# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.55.0] - 2025-03-05

- 4056560 ast/estree: [**BREAKING**] Option to return JS-only AST (#9520) (overlookmotel)

### Features

- af02a87 ast/estree: `Property` have consistent field order (#9547) (overlookmotel)
- 3e4f909 ast/estree: ESTree AST `ExportNamedDeclaration` always have `attributes` field (#9546) (overlookmotel)
- d55dbe2 ast/estree: Raw transfer (experimental) (#9516) (overlookmotel)

### Bug Fixes

- a0f6f37 ast/estree: Raw transfer support `showSemanticErrors` option (#9522) (overlookmotel)

### Refactor

- c1a8cea ast/estree: Simplify serializing `RegExpLiteral`s (#9551) (overlookmotel)

### Testing

- 4378a66 ast/estree: Speed up raw transfer tests (#9521) (overlookmotel)

## [0.54.0] - 2025-03-04

- 355a4db napi/parser: [**BREAKING**] Remove `parse_without_return` API (#9455) (Boshen)

- a5cde10 visit_ast: [**BREAKING**] Add `oxc_visit_ast` crate (#9428) (Boshen)

### Features

- 68c77c8 napi/parser: Return semantic errors (#9460) (Boshen)

### Testing

- d129055 napi: Add tests for worker threads (#9408) (Boshen)
- 48d51e3 napi: Add tests for `hashbang` field (#9386) (overlookmotel)

## [0.53.0] - 2025-02-26

- 4a5a7cf napi/parser: [**BREAKING**] Remove magic string; enable utf16 span converter by default (#9291) (Boshen)

### Features


### Performance

- 61939ca ast/estree: Faster UTF-8 to UTF-16 span conversion (#9349) (overlookmotel)

### Refactor

- b09249c ast/estree: Rename serializers and serialization methods (#9284) (overlookmotel)

## [0.52.0] - 2025-02-21

- 216b33f ast/estree: [**BREAKING**] Replace `serde` with custom `ESTree` serializer (#9256) (overlookmotel)

### Features


### Bug Fixes

- b9c8a10 wasm: Transfer AST to JS as JSON string in `oxc-wasm` (#9269) (overlookmotel)
- 5acc6ec wasm: Transfer AST to JS as JSON string (#9259) (overlookmotel)

## [0.51.0] - 2025-02-15

### Bug Fixes

- 0937a55 napi/parser: Utf16 span for errors (#9112) (hi-ogawa)
- 15f23f1 napi/parser: Utf16 span for module record (#9093) (hi-ogawa)
- 9edfb1d napi/parser: Fix unicode comment panic (#9084) (hi-ogawa)

### Performance

- af59945 napi/parser: Do not convert comment spans twice (#9087) (overlookmotel)

### Testing

- eaff3d9 napi/parser: Split tests for `convertSpanUtf16` (#9113) (hi-ogawa)

## [0.50.0] - 2025-02-12

### Features

- 81c81a7 napi/parser: Add `convert_span_utf16` option (#8983) (hi-ogawa)

### Bug Fixes

- 41dba62 ast/estree: Set `value` for `BigIntLiteral`s and `RegExpLiteral`s on JS side (#9044) (overlookmotel)

### Testing

- ef553b9 napi: Add NAPI parser benchmark (#9045) (overlookmotel)

## [0.49.0] - 2025-02-10

### Bug Fixes

- a520986 ast: Estree compat `Program.sourceType` (#8919) (Hiroshi Ogawa)
- e30cf6a ast: Estree compat `MemberExpression` (#8921) (Hiroshi Ogawa)
- 0c55dd6 ast: Serialize `Function.params` like estree (#8772) (Hiroshi Ogawa)

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

### Testing

- 4803059 ast: Remove old ast snapshot tests (#8976) (hi-ogawa)

## [0.47.1] - 2025-01-19

### Features

- ee8ee55 napi/parser: Add `.hasChanged()` to `MagicString` (#8586) (Boshen)
- 1bef911 napi/parser: Add source map API (#8584) (Boshen)

## [0.47.0] - 2025-01-18

### Features

- c479a58 napi/parser: Expose dynamic import expressions (#8540) (Boshen)

