# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.30.0] - 2024-09-23

### Bug Fixes

- 362c427 mangler,codegen: Do not mangle top level symbols (#5965) (Boshen)

### Performance

- c477424 mangler: Use `sort_unstable_by_key` instead of `sort_by` (#5948) (Boshen)

## [0.28.0] - 2024-09-11

- b060525 semantic: [**BREAKING**] Remove `source_type` argument from `SemanticBuilder::new` (#5553) (Boshen)

### Refactor


## [0.25.0] - 2024-08-23

- 5f4c9ab semantic: [**BREAKING**] Rename `SymbolTable::get_flag` to `get_flags` (#5030) (overlookmotel)

### Refactor

- ca70cc7 linter, mangler, parser, semantic, transformer, traverse, wasm: Rename various `flag` vars to `flags` (#5028) (overlookmotel)
- b4407c4 oxc,mangler: `oxc` crate add mangler; mangler use options API (Boshen)

## [0.22.1] - 2024-07-27

### Performance

- 963a2d1 mangler: Reduce unnecessary allocation (#4498) (Dunqing)

### Refactor

- 7cd53f3 semantic: Var hoisting (#4379) (Dunqing)
- c99b3eb syntax: Give `ScopeId` a niche (#4468) (overlookmotel)

## [0.22.0] - 2024-07-23

### Bug Fixes

- 3d88f20 codegen: Print shorthand for all `{ x }` variants (#4374) (Boshen)

## [0.21.0] - 2024-07-18

### Features

- 5d17675 mangler: Add debug mode (#4314) (Boshen)
- e3e663b mangler: Initialize crate and integrate into minifier (#4197) (Boshen)

### Bug Fixes

- 3df9e69 mangler: No shorthand `BindingProperty`; handle var hoisting and export variables (#4319) (Boshen)

