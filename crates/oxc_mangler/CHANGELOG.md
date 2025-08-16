# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).




## [0.80.0] - 2025-08-03

### 🚜 Refactor

- 77d397a mangler: Move `NodeId` lookup into `is_name_set_reference_node` (#12664) (overlookmotel)

### 📚 Documentation

- de1de35 rust: Add comprehensive README.md documentation for all Rust crates (#12706) (Copilot)

### ⚡ Performance

- 4adc1ed mangler: Remove unnecessary `AstNode` lookups (#12663) (overlookmotel)



## [0.79.0] - 2025-07-30

### 🚜 Refactor

- a696227 linter: Remove AstKind for SimpleAssignmentTarget (#12401) (Tyler Earls)





## [0.77.1] - 2025-07-16

### 🚜 Refactor

- ee761de ast: Remove `AstKind` for `AssignmentTarget` (#12252) (Tyler Earls)


## [0.77.0] - 2025-07-12

### ⚡ Performance

- c7889c3 semantic,linter: Simplify implementation and uses of ancestors iterators (#12164) (Ulrich Stark)


## [0.76.0] - 2025-07-08

### 🚜 Refactor

- 54cf5cb semantic: Remove Option from parent_* methods (#12087) (Ulrich Stark)


## [0.75.1] - 2025-07-03

### 🚜 Refactor

- f7a2ae4 ast: Add `AstKind` for `AssignmentTargetPropertyIdentifier`, `AssignmentTargetPropertyProperty` (#11985) (camc314)






## [0.73.0] - 2025-06-13

### 🚜 Refactor

- bfa05f2 mangler: Use bump allocator for internal storage (#11409) (camchenry)
- fa4966d mangler: Use external allocator (#11408) (camchenry)


# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.71.0] - 2025-05-20

- 65a6419 mangler: [**BREAKING**] `Mangler::build_with_semantic` take mut ref to `Semantic` (#11132) (overlookmotel)

### Performance


## [0.61.2] - 2025-03-23

### Features

- ea3de06 mangler: Support `keep_names` option (#9898) (sapphi-red)
- 9992559 mangler: Collect symbols that is used to set `name` property (#9897) (sapphi-red)

## [0.61.0] - 2025-03-20

### Performance

- b272893 mangler, minifier: Initialize a Vec with a specific value using `Vec::from_iter_in` combined with `repeat_with` (#9908) (Dunqing)

## [0.60.0] - 2025-03-18

- b3ce925 data_structures: [**BREAKING**] Put all parts behind features (#9849) (overlookmotel)

### Features


## [0.58.0] - 2025-03-13

### Documentation

- a6c9b09 napi/minifier: Improve documentation (#9736) (Boshen)

## [0.57.0] - 2025-03-11

- 3c6f140 semantic: [**BREAKING**] Make `Scoping` methods consistent (#9628) (Boshen)

- ef6e0cc semantic: [**BREAKING**] Combine `SymbolTable` and `ScopeTree` into `Scoping` (#9615) (Boshen)

- 7331656 semantic: [**BREAKING**] Rename `SymbolTable` and `ScopeTree` methods (#9613) (Boshen)

- 23738bf semantic: [**BREAKING**] Introduce `Scoping` (#9611) (Boshen)

### Refactor


## [0.55.0] - 2025-03-05

### Features

- 29041fb data_structures: Move `InlineString` into `oxc_data_structures` crate (#9549) (overlookmotel)

### Performance

- bc14ee5 mangler: Use shorter `InlineString` (#9552) (overlookmotel)

## [0.53.0] - 2025-02-26

### Refactor

- c31b53f mangler: Reduce scope of `unsafe` blocks (#9321) (overlookmotel)
- f10a6da mangler: Move base54 into seperate mod (#9278) (Cameron)

## [0.52.0] - 2025-02-21

### Features

- dde05e3 mangler: Opt-out of direct eval (#9191) (Boshen)

### Refactor

- 3f0b5b3 mangler: Use `iter::repeat_n` (#9272) (overlookmotel)

## [0.49.0] - 2025-02-10

### Bug Fixes

- f11dff0 mangler, prettier, ast_tools: Remove methods which are unstable in our MSRV (#8929) (overlookmotel)

## [0.48.2] - 2025-02-02

### Features

- 86b6219 mangler: Use characters in the order of their likely frequency (#8771) (sapphi-red)

### Performance

- 2e4ff91 manger: Revert "perf(manger): remove useless tmp_bindings (#8735)" (#8741) (Dunqing)

### Refactor

- 6aa2dde codegen: Accept SymbolTable instead of Mangler (#8829) (Daniel Bulant)

## [0.48.1] - 2025-01-26

### Features

- 6589c3b mangler: Reuse variable names (#8562) (翠 / green)

### Bug Fixes

- 33de70a mangler: Handle cases where a var is declared in a block scope (#8706) (翠 / green)

### Performance

- dc0b0f2 manger: Remove useless `tmp_bindings` (#8735) (Dunqing)
- e472ced mangler: Optimize handling of collecting lived scope ids (#8724) (Dunqing)

### Refactor

- 52a37d0 mangler: Simplify initialization of `slots` (#8734) (Dunqing)

## [0.48.0] - 2025-01-24

### Refactor

- ac4f98e span: Derive `Copy` on `Atom` (#8596) (branchseer)

## [0.47.0] - 2025-01-18

### Performance

- d17021c mangler: Optimize `base54` function (#8557) (overlookmotel)
- 6b52d7a mangler: Use a single allocation space for temporary vecs (#8495) (Boshen)

## [0.46.0] - 2025-01-14

### Performance

- 7a8200c mangler: Allocate base54 name without heap allocation (#8472) (Boshen)
- 31dac22 mangler: Allocate data in arena (#8471) (Boshen)
- 372eb09 minifier: Preallocate mangler's semantic data (#8451) (Boshen)

## [0.45.0] - 2025-01-11

### Bug Fixes

- 5c63414 mangler: Keep exported symbols for `top_level: true` (#7927) (翠 / green)

## [0.43.0] - 2024-12-21

### Performance

- 414e828 semantic: Allocate symbol data in Allocator (#8012) (Boshen)

### Refactor

- 02f968d semantic: Change `Bindings` to a plain `FxHashMap` (#8019) (Boshen)

## [0.42.0] - 2024-12-18

### Features

- db9e93b mangler: Mangle top level variables (#7907) (翠 / green)

## [0.33.0] - 2024-10-24

### Refactor

- e59b5d9 minifier: Dereference `SymbolId` as soon as possible (#6823) (overlookmotel)

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

