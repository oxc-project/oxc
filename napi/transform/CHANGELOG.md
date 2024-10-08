# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.31.0] - 2024-10-08

- 020bb80 codegen: [**BREAKING**] Change to `CodegenReturn::code` and `CodegenReturn::map` (#6310) (Boshen)

- 4f6bc79 transformer: [**BREAKING**] Remove `source_type` param from `Transformer::new` (#6251) (overlookmotel)

- afc3ccb napi/transform: [**BREAKING**] Rename `TransformOptions::react` to `jsx`. (#6211) (Boshen)

### Features

- abd3a9f napi/transform: Perform dce after define plugin (#6312) (Boshen)
- a0ccc26 napi/transform: Add `lang` option to change source type (#6309) (Boshen)
- f98e12c napi/transform: Add inject plugin (#6250) (Boshen)
- 291891e napi/transform: Add `define` option (#6212) (Boshen)
- 51a78d5 napi/transform: Rename all mention of React to Jsx; remove mention of `Binding` (#6198) (Boshen)
- 2f888ed oxc: Add napi transform options (#6268) (Boshen)
- 8729755 oxc,napi/transform: Napi/transform use oxc compiler pipeline (#6298) (Boshen)

### Bug Fixes

- 294da86 napi/transform: Fix index.d.ts (Boshen)

### Refactor

- 5b5daec napi: Use vitest (#6307) (Boshen)
- 58a8615 napi/transform: Remove context (#6306) (Boshen)
- 099ff3a napi/transform: Remove "Binding" from types; fix type error (#6260) (Boshen)
- 54c1c53 napi/transform: Remove a call on `TransformOptions::clone` (#6210) (Boshen)

## [0.30.5] - 2024-09-29

### Features

- 15552ac napi/transform: Display semantic error (#6160) (Boshen)
- f50fdcd napi/transform: Make react refresh option take a boolean (#6146) (Boshen)

### Bug Fixes

- f27d59f napi/transform: Remove confusing `jsx` option (#6159) (Boshen)

## [0.30.4] - 2024-09-28

### Bug Fixes

- 6f98aad sourcemap: Align sourcemap type with Rollup (#6133) (Boshen)

## [0.30.0] - 2024-09-23

### Features

- 84a5816 isolated_declarations: Add `stripInternal` (#5878) (Boshen)
- dfbde2c isolated_declarations: Print jsdoc comments (#5858) (Boshen)
- 3230ae5 semantic: Add `SemanticBuilder::with_excess_capacity` (#5762) (overlookmotel)

### Bug Fixes

- 127c881 napi/transform: Fix jsdoc links (#5886) (Boshen)
- 6c04fa1 napi/transform: Make isolated_declaration options optional (#5880) (Boshen)

## [0.29.0] - 2024-09-13

### Bug Fixes

- 608b7d3 napi/transformer: Refresh plugin doesn't work even after passing the refresh option (#5702) (Dunqing)

## [0.28.0] - 2024-09-11

- b060525 semantic: [**BREAKING**] Remove `source_type` argument from `SemanticBuilder::new` (#5553) (Boshen)

### Features

- e698418 napi/transform: Align output `SourceMap` with Rollup's `ExistingRawSourceMap` (#5657) (Boshen)
- aba9194 napi/transform: Export react refresh options (#5533) (underfin)

### Refactor


## [0.27.0] - 2024-09-06

### Bug Fixes

- ea7a52f napi/transform: Fix test (Boshen)

## [0.26.0] - 2024-09-03

- b1d0075 napi/transform: [**BREAKING**] Align output API `sourceText` -> `code` with babel (#5398) (Boshen)

### Features

- 72740b3 isolated_declaration: Support sourcemap option (#5170) (dalaoshu)
- 01c0c3e transformer: Add remaining options to transformer options (#5169) (Boshen)
- 0abfc50 transformer/typescript: Support `rewrite_import_extensions` option (#5399) (Dunqing)

## [0.25.0] - 2024-08-23

- ce4d469 codegen: [**BREAKING**] Remove const generic `MINIFY` (#5001) (Boshen)

### Features

- 4b49cf8 transformer: Always pass in symbols and scopes (#5087) (Boshen)

## [0.24.1] - 2024-08-10

### Bug Fixes

- 4d0b40a napi/transform: Fix wrong isolated declarations emit (Boshen)

## [0.24.0] - 2024-08-08

### Bug Fixes

- 01d85de napi/transform: Update napi files (Boshen)

### Refactor
- 9b51e04 Overhaul napi transformer package (#4592) (DonIsaac)

## [0.23.1] - 2024-08-06

### Refactor
- 9b51e04 Overhaul napi transformer package (#4592) (DonIsaac)

## [0.20.0] - 2024-07-11

### Features

- 725571a napi/transformer: Add `jsx` option to force parsing with jsx (#4133) (Boshen)

## [0.17.2] - 2024-07-08

### Features

- 720983a napi/transform: Allow setting `sourceType` to `transform` (#4113) (Boshen)

## [0.17.1] - 2024-07-06

### Bug Fixes

- 150f4d9 napi/transform: Display error with spanned messages (Boshen)

## [0.16.2] - 2024-06-30

### Refactor

- 5845057 transformer: Pass in symbols and scopes (#3978) (Boshen)

