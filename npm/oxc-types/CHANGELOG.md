# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.60.0] - 2025-03-18

### Bug Fixes

- 7b711f0 ast/estree: Make TS-only fields optional in TS type defs (#9846) (overlookmotel)

### Performance

- 2d63704 ast: Re-order `VariableDeclarationKind` variants (#9853) (overlookmotel)

## [0.59.0] - 2025-03-18

- ce6808a parser: [**BREAKING**] Rename `type_parameters` to `type_arguments` where needed  (#9815) (hi-ogawa)

### Features

- db946e6 ast/estree: Order TS fields last by default (#9820) (overlookmotel)

### Bug Fixes

- 3f858c4 ast/estree: Add `directive` field to `ExpressionStatement` in TS AST (#9844) (overlookmotel)
- cd18358 ast/extree: Fix `Class.implements` (#9817) (hi-ogawa)

## [0.58.1] - 2025-03-13

### Bug Fixes

- cd3f2fb ast/estree: Fix `JSXOpeningFragment` (#9747) (Hiroshi Ogawa)
- fecec56 ast/estree: Fix `JSXOpeningElement` field order (#9746) (hi-ogawa)

## [0.58.0] - 2025-03-13

- 842edd8 ast: [**BREAKING**] Add `raw` property to `JSXText` node (#9641) (Yuji Sugiura)

### Features

- 446d11e ast/estree: Export `Node` union type (#9574) (hi-ogawa)

## [0.57.0] - 2025-03-11

- 510446a parser: [**BREAKING**] Align JSXNamespacedName with ESTree (#9648) (Arnaud Barré)

### Bug Fixes

- eae1a41 ast: Align `TSImportType` field names with ts-eslint (#9664) (Boshen)

## [0.56.3] - 2025-03-07

### Features

- 6b95d25 parser: Disallow `TSInstantiationExpression` in `SimpleAssignmentTarget` (#9586) (Boshen)

## [0.55.0] - 2025-03-05

### Features

- af02a87 ast/estree: `Property` have consistent field order (#9547) (overlookmotel)

## [0.54.0] - 2025-03-04

### Features

- 2a08b14 parser: Support V8 intrinsics (#9379) (injuly)

## [0.53.0] - 2025-02-26

### Bug Fixes

- e303767 ast/estree: Fix ESTree AST for imports and exports (#9282) (overlookmotel)

## [0.52.0] - 2025-02-21

### Bug Fixes

- 72bab88 ast/estree: Remove unused TS type def for `WithClause` (#9250) (overlookmotel)

### Refactor

- 97cc1c8 ast: Remove `TSLiteral::NullLiteral` (replaced by `TSNullKeyword`) (#9147) (Boshen)

## [0.51.0] - 2025-02-15

- 21a9476 ast: [**BREAKING**] Remove `TSLiteral::RegExpLiteral` (#9056) (Dunqing)

- 9091387 ast: [**BREAKING**] Remove `TSType::TSQualifiedName` (#9051) (Dunqing)

### Features


### Bug Fixes

- 38f81af ast/estree: Order fields same as Acorn (#9128) (overlookmotel)
- 67f8932 ast/estree: `CatchParameter` do not include `type` and `Span` twice (#9125) (overlookmotel)
- 1b02fe0 ast/estree: `FormalParameter` do not include `Span` twice (#9124) (overlookmotel)
- d3b5fb0 ast/estree: Fix TS type for `AssignmentTargetPropertyIdentifier` (#9092) (overlookmotel)
- d8d80a9 ast/estree: Fix TS types for `BigIntLiteral` and `RegExpLiteral` (#9091) (overlookmotel)

## [0.50.0] - 2025-02-12

- d9189f1 ast: [**BREAKING**] Remove `PrivateInExpression::operator` field (#9041) (overlookmotel)

### Bug Fixes

- 22d93be ast: Estree compat `AssignmentTargetPropertyIdentifier` (#9006) (hi-ogawa)
- cd2e199 ast/estree: Fix serializing `RegExpLiteral` (#9043) (overlookmotel)
- f705c64 ast/estree: Serialize `PrivateInExpression` as `BinaryExpression` (#9033) (hi-ogawa)
- 2948804 ast/estree: Fix `ExportAllDeclaration` attributes (#9032) (hi-ogawa)
- 2371dd4 ast/estree: Fix serializing import and export `attributes` (#9030) (hi-ogawa)
- e75e1d2 ast/estree: Fix serializing `PrivateFieldExpression` (#9025) (overlookmotel)
- fcb5490 ast/estree: Fix serializing `ImportExpression`s (#9024) (overlookmotel)
- 9427007 ast/estree: Use `#[estree(append_to)]` for `TSModuleBlock` (#9020) (overlookmotel)

### Refactor

- cb3240c ast/estree: Remove redundant `ts_type` (#9037) (hi-ogawa)

## [0.49.0] - 2025-02-10

### Features

- 53ee053 ast: Improve TS type definitions for appended fields (#8882) (overlookmotel)

### Bug Fixes

- 1daa8fe ast: Estree compat `AssignmentTargetPropertyProperty` (#9005) (hi-ogawa)
- e0646d7 ast: Estree compat `ArrayAssignmentTarget` (#8998) (hi-ogawa)
- d7802a7 ast: Serialize `ArrowFunctionExpression.body: FunctionBody | Expression` (#8988) (Hiroshi Ogawa)
- 7e6a537 ast: Include `directives` in `body` (#8981) (hi-ogawa)
- ec1d593 ast: Add missing estree props for `ArrowFunctionExpression` (#8980) (camchenry)
- 8eccdec ast: Estree compat `CatchClause` (#8975) (hi-ogawa)
- 2ee1d6c ast: Estree compat `Property` (#8974) (hi-ogawa)
- 801d78e ast: Estree compat `UnaryExpression` (#8973) (hi-ogawa)
- a2883b1 ast: Estree compat `Function` (#8972) (hi-ogawa)
- a520986 ast: Estree compat `Program.sourceType` (#8919) (Hiroshi Ogawa)
- e30cf6a ast: Estree compat `MemberExpression` (#8921) (Hiroshi Ogawa)
- 0c55dd6 ast: Serialize `Function.params` like estree (#8772) (Hiroshi Ogawa)

### Refactor

- 640db88 ast: Introduce `#[estree(ts_alias)]` attr and use it on `Elision` (#8939) (overlookmotel)
- 66f0afb ast: Shorten TS type definitions for enums (#8938) (overlookmotel)
- a6884e4 ast: Simplify serializing literal types (#8937) (overlookmotel)
- c58f785 ast: Simplify serializing `SourceType` (#8936) (overlookmotel)
- 223eb8d ast: Override TS type defs with `#[estree(custom_ts_def)]` attribute on type (#8897) (overlookmotel)
- f6f92db ast: Re-order generated code (#8863) (overlookmotel)

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

