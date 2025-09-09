# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.87.0] - 2025-09-08

### 🚀 Features

- 9590b57 regular_expression: Detect regex pattern modifiers usage (#13471) (sapphi-red)

### 🐛 Bug Fixes

- 9184911 regular_expression: Detect usage of unsupported syntax recursively (#13470) (sapphi-red)
- 0eea7da regular_expression: Don't lower capture groups that are not named ones (#13469) (sapphi-red)

### 🚜 Refactor

- c17b80a regular_expression: Extract `has_unsupported_regular_expression_pattern` (#13468) (sapphi-red)
- 14c40fd ast: Implement `RegExpLiteral::parse_pattern` (#13467) (sapphi-red)











## [0.80.0] - 2025-08-03

### 📚 Documentation

- 514322c rust: Add minimal documentation to example files in crates directory (#12731) (Copilot)














## [0.73.1] - 2025-06-17

### 🚜 Refactor

- d057652 regular-expression: Shorten Span construction (#11689) (Ulrich Stark)



# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.71.0] - 2025-05-20

- 5d9344f rust: [**BREAKING**] Clippy avoid-breaking-exported-api = false (#11088) (Boshen)

### Features

- c60382d allocator/vec2: Change `len` and `cap` fields from `usize` to `u32` (#10884) (Dunqing)

### Performance

- 2b0a69f ast: Re-order struct fields to reduce padding (#11056) (overlookmotel)

### Refactor

- 9775585 regular_expression: Refactor `regexp-modifiers` support (#11142) (Yuji Sugiura)

## [0.70.0] - 2025-05-15

### Documentation

- a86cbb3 linter: Fix incorrect backticks of fenced code blocks (#10947) (Ulrich Stark)

## [0.69.0] - 2025-05-09

### Refactor

- daba0a7 estree: Remove regular expression types from ESTree AST (#10855) (overlookmotel)
- 6de5a43 regular_expression: Move `impl GetSpan` to separate file (#10846) (Yuji Sugiura)

### Styling

- 62c3a4a ast_tools: Add full stop to end of generated comments (#10809) (overlookmotel)

## [0.63.0] - 2025-04-08

### Performance

- 774f6ba regular_expression: Remove `write!` macro where unnecessary (#10233) (overlookmotel)

## [0.61.2] - 2025-03-23

### Performance

- a09bbcf regular_expression: Make all fieldless enums `Copy` (#9937) (overlookmotel)

### Refactor

- d3d7d98 ast: Shorten generated code for `CloneIn` (#9939) (overlookmotel)
- 28179cd ast_tools: Simplify `CloneIn` derive (#9938) (overlookmotel)

## [0.61.1] - 2025-03-21

### Features

- bc0670c tasks,oxc_allocator: Add new method clone_in_with_semantic_ids for `CloneIn` trait (#9894) (IWANABETHATGUY)

## [0.54.0] - 2025-03-04

### Performance

- b0a0a82 ast/estree: Reduce overhead serializing static strings (#9396) (overlookmotel)

### Refactor

- dcff40c ast_tools: Generate layout assertions in multiple crates (#9448) (overlookmotel)

## [0.53.0] - 2025-02-26

### Refactor

- 55ed1df ast/estree: Shorten `ESTree` impls for enums (#9275) (overlookmotel)

## [0.52.0] - 2025-02-21

- 216b33f ast/estree: [**BREAKING**] Replace `serde` with custom `ESTree` serializer (#9256) (overlookmotel)

### Features


### Refactor

- ef856f5 oxc: Apply `clippy::needless_pass_by_ref_mut` (#9253) (Boshen)
- 9f36181 rust: Apply `cllippy::nursery` rules (#9232) (Boshen)

## [0.49.0] - 2025-02-10

- b7ff7e1 span: [**BREAKING**] Export `ContentEq` trait from root of `oxc_span` crate (#8869) (overlookmotel)

### Bug Fixes

- 7e6a537 ast: Include `directives` in `body` (#8981) (hi-ogawa)

### Refactor

- cbb4e9c ast: Generated `Serialize` impls flatten struct fields (#8904) (overlookmotel)
- abfe5bf ast: Shorten generated code for numbers (#8864) (overlookmotel)
- f69de07 ast: Remove unneeded lint attrs from generated code (#8862) (overlookmotel)
- 6d1e1d8 ast: Make generated code consistent (#8872) (overlookmotel)

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

## [0.48.0] - 2025-01-24

### Refactor

- ac4f98e span: Derive `Copy` on `Atom` (#8596) (branchseer)

## [0.47.0] - 2025-01-18

- 7066d1c ast, span, syntax, regular_expression: [**BREAKING**] Remove `ContentHash` (#8512) (overlookmotel)

### Features


### Refactor

- 007e8c0 ast, regular_expression: Shorten `ContentEq` implementations (#8519) (overlookmotel)
- b5ed58e span: All methods take owned `Span` (#8297) (overlookmotel)

## [0.46.0] - 2025-01-14

- 7eb6ccd ast: [**BREAKING**] Remove unused and not useful `ContentHash` (#8483) (Boshen)

### Features


## [0.42.0] - 2024-12-18

### Refactor

- 3858221 global: Sort imports (#7883) (overlookmotel)

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.38.0] - 2024-11-26

### Features

- eb70219 ast: Derive `GetAddress` on all enum types (#7472) (overlookmotel)

## [0.36.0] - 2024-11-09

### Features

- dc0215c ast_tools: Add #[estree(append_to)], remove some custom serialization code (#7149) (ottomated)

## [0.35.0] - 2024-11-04

### Features

- ce5b609 ast: Remove explicit untagged marker on enums (#6915) (ottomated)
- 9725e3c ast_tools: Add #[estree(always_flatten)] to Span (#6935) (ottomated)
- 169fa22 ast_tools: Default enums to rename_all = "camelCase" (#6933) (ottomated)

## [0.34.0] - 2024-10-26

- 90c786c regular_expression: [**BREAKING**] Support ES2025 Duplicated named capture groups (#6847) (leaysgur)

### Features

- 1145341 ast_tools: Output typescript to a separate package (#6755) (ottomated)

### Refactor

- 423d54c rust: Remove the annoying `clippy::wildcard_imports` (#6860) (Boshen)

## [0.33.0] - 2024-10-24

- 8032813 regular_expression: [**BREAKING**] Migrate to new regexp parser API (#6741) (leaysgur)

### Features

- f8e1907 regular_expression: Intro `ConstructorParser`(and `LiteralParser`) to handle escape sequence in RegExp('pat') (#6635) (leaysgur)

### Bug Fixes


### Refactor

- 85e69a1 ast_tools: Add line breaks to generated code for `ESTree` derive (#6680) (overlookmotel)
- ad8e293 ast_tools: Shorten generated code for `impl Serialize` (#6684) (overlookmotel)
- 9ba2b0e ast_tools: Move `#[allow]` attrs to top of generated files (#6679) (overlookmotel)
- 11458a5 ast_tools: Shorten generated code by avoiding `ref` in matches (#6675) (overlookmotel)

## [0.32.0] - 2024-10-19

### Features

- e310e52 parser: Generate `Serialize` impls in ast_tools (#6404) (ottomated)
- b5b0af9 regular_expression: Support RegExp Modifiers (#6410) (leaysgur)

### Bug Fixes

- 9f9057b regular_expression: Fixed control Y regular expression (#6524) (Tapan Prakash)
- c822b48 regular_expression: Fix CharacterClass negative codegen (#6415) (leaysgur)
- 384d5be regular_expression: Flatten Spans on regex AST nodes (#6396) (ottomated)

### Performance

- 7c20056 regex: Reduce string allocations in `Display` impls (#6528) (DonIsaac)

### Styling

- fb916b2 regular_expression: Re-order dependencies in `Cargo.toml` (#6672) (overlookmotel)

## [0.31.0] - 2024-10-08

- 5a73a66 regular_expression: [**BREAKING**] Simplify public APIs (#6262) (leaysgur)

### Refactor

- acab777 regular_expression: Misc fixes (#6234) (leaysgur)

## [0.30.2] - 2024-09-27

### Features

- 8d026e1 regular_expression: Implement `GetSpan` for RegExp AST nodes (#6056) (camchenry)
- 7764793 regular_expression: Implement visitor pattern trait for regex AST (#6055) (camchenry)

## [0.28.0] - 2024-09-11

### Bug Fixes

- 304ce25 regular_expression: Keep LegacyOctalEscape raw digits for `to_string` (#5692) (leaysgur)
- 0511d55 regular_expression: Report more MayContainStrings error in (nested)class (#5661) (leaysgur)
- 41582ea regular_expression: Improve RegExp `to_string()` results (#5635) (leaysgur)
- 28aad28 regular_expression: Handle `-` in `/[\-]/u` as escaped character (#5631) (leaysgur)

### Refactor

- 0ac420d linter: Use meaningful names for diagnostic parameters (#5564) (Don Isaac)
- 2da42ef regular_expression: Improve AST docs with refactoring may_contain_strings (#5665) (leaysgur)
- dec1395 regular_expression: Align diagnostics (#5543) (leaysgur)

## [0.27.0] - 2024-09-06

### Features

- 90facd3 ast: Add `ContentHash` trait; remove noop `Hash` implementation from `Span` (#5451) (rzvxa)
- 23285f4 ast: Add `ContentEq` trait. (#5427) (rzvxa)
- 59abf27 ast, parser: Add `oxc_regular_expression` types to the parser and AST. (#5256) (rzvxa)

### Bug Fixes

- 9b984b3 regex: Panic on displaying surrogated `UnicodeEscape` characters. (#5469) (rzvxa)
- 88b7ddb regular_expression: Handle unterminated character class (#5523) (leaysgur)

### Refactor

- ccc8a27 ast, ast_tools: Use full method path for generated derives trait calls. (#5462) (rzvxa)
- e7bd49d regular_expression: Correct typo (#5429) (overlookmotel)

## [0.26.0] - 2024-09-03

### Features

- 46b641b regular_expression: Validate max quantifier value (#5218) (leaysgur)

### Bug Fixes

- cffce11 regular_expression: Prevent panic on too large number (#5282) (leaysgur)

