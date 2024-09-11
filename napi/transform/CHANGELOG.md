# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

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

