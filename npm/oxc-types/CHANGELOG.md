# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.47.0] - 2025-01-18

- 19d3677 ast: [**BREAKING**] Always return `Array<ImportDeclarationSpecifier>` for `ImportDeclaration.specifiers` (#8560) (sapphi-red)

### Bug Fixes


## [0.44.0] - 2024-12-25

- ad2a620 ast: [**BREAKING**] Add missing `AssignmentTargetProperty::computed` (#8097) (Boshen)

### Bug Fixes


## [0.40.0] - 2024-12-10

- 72eab6c parser: [**BREAKING**] Stage 3 `import source` and `import defer` (#7706) (Boshen)

- ebc80f6 ast: [**BREAKING**] Change 'raw' from &str to Option<Atom> (#7547) (Song Gao)

### Features


### Bug Fixes

- 2179b93 estree: Make type of `BigIntLiteral::raw` prop in ESTree AST optional (#7663) (overlookmotel)
- cbba26c estree: `raw: null` in ESTree AST for generated `NullLiteral`s (#7662) (overlookmotel)
- 1d59fc8 estree: `raw: null` in ESTree AST for generated `BooleanLiteral`s (#7661) (overlookmotel)

### Refactor


## [0.39.0] - 2024-12-04

- b0e1c03 ast: [**BREAKING**] Add `StringLiteral::raw` field (#7393) (Boshen)

### Features


## [0.37.0] - 2024-11-21

- f059b0e ast: [**BREAKING**] Add missing `ChainExpression` from `TSNonNullExpression` (#7377) (Boshen)

### Features

- 897d3b1 ast: Serialize StringLiterals to ESTree without `raw` (#7263) (ottomated)

### Bug Fixes


### Performance

- c335f92 syntax: Reorder operator enum variants (#7351) (overlookmotel)

### Refactor

- de472ca ast: Move `StringLiteral` definition higher up (#7270) (overlookmotel)

## [0.36.0] - 2024-11-09

- b11ed2c ast: [**BREAKING**] Remove useless `ObjectProperty::init` field (#7220) (Boshen)

- 0e4adc1 ast: [**BREAKING**] Remove invalid expressions from `TSEnumMemberName` (#7219) (Boshen)

- 092de67 types: [**BREAKING**] Append `rest` field into `elements` for objects and arrays to align with estree (#7212) (ottomated)

### Features

- dc0215c ast_tools: Add #[estree(append_to)], remove some custom serialization code (#7149) (ottomated)
- 9d6cc9d estree: ESTree compatibility for all literals (#7152) (ottomated)

### Bug Fixes


### Refactor


## [0.35.0] - 2024-11-04

### Features

- 9725e3c ast_tools: Add #[estree(always_flatten)] to Span (#6935) (ottomated)

### Bug Fixes

- caaf00e parser: Fix incorrect parsed `TSIndexSignature` (#7016) (Boshen)

### Refactor

- 9926990 napi: Move custom types to bottom of file (#6930) (overlookmotel)
- 23157bd napi: Types file in root of types package (#6929) (overlookmotel)

## [0.34.0] - 2024-10-26

- 67a7bde napi/parser: [**BREAKING**] Add typings to napi/parser (#6796) (ottomated)

### Features

- 1145341 ast_tools: Output typescript to a separate package (#6755) (ottomated)

### Bug Fixes

- b075982 types: Change @oxc/types package name (#6874) (ottomated)

