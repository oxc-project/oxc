# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.48.0] - 2025-01-24

### Features

- 2a2ad53 allocator: Add `Allocator::capacity` and `used_bytes` methods (#8621) (overlookmotel)
- 6801c81 allocator: Add `Allocator::new` and `with_capacity` methods (#8620) (overlookmotel)

### Performance

- 787aaad allocator: Make `String` non-drop (#8617) (overlookmotel)

### Documentation

- c1d243b allocator: Improve docs for `Allocator` (#8623) (overlookmotel)
- 01a5e5d allocator: Improve docs for `HashMap` (#8616) (overlookmotel)
- 87568a1 allocator: Reformat docs (#8615) (overlookmotel)

### Refactor

- ae8db53 allocator: Move `Allocator` into own module (#8656) (overlookmotel)
- 0f85bc6 allocator: Reduce repeat code to prevent `Drop` types in arena (#8655) (overlookmotel)
- de76eb1 allocator: Reorder `Box` methods (#8654) (overlookmotel)

## [0.47.0] - 2025-01-18

- fae4cd2 allocator: [**BREAKING**] Remove `Vec::into_string` (#8571) (overlookmotel)

- 95bc0d7 allocator: [**BREAKING**] `Allocator` do not deref to `bumpalo::Bump` (#8569) (overlookmotel)

### Features

- bf4e5e1 allocator: Add `HashMap` (#8553) (overlookmotel)

### Bug Fixes

- e87c001 allocator: Statically prevent memory leaks in allocator (#8570) (overlookmotel)

### Performance

- 76ea52b allocator: Inline `Box` methods (#8572) (overlookmotel)
- 93df57f allocator: `#[inline(always)]` methods of `Vec` which just delegate to `allocator_api2` (#8567) (overlookmotel)
- 5a28d68 allocator: `#[inline(always)]` methods of `HashMap` which just delegate to `hashbrown` (#8565) (overlookmotel)

### Documentation

- fa1a6d5 allocator: Update docs for `Vec` (#8555) (overlookmotel)

### Refactor

- ac05134 allocator: `String` type (#8568) (overlookmotel)
- 68fab81 allocator: Rename inner `Vec` type (#8566) (overlookmotel)

## [0.45.0] - 2025-01-11

### Features

- 6c7acac allocator: Implement `IntoIterator` for `&mut Vec` (#8389) (overlookmotel)
- 06e1780 minifier: Improve `StatementFusion` (#8194) (Boshen)

### Bug Fixes

- eb25bc0 allocator: Fix lifetimes on `IntoIterator` for `Vec` (#8388) (overlookmotel)

## [0.43.0] - 2024-12-21

### Features

- 75b775c allocator: `Vec<u8>::into_string` (#8017) (overlookmotel)
- 8547e02 ast: Implement `allocator_api2` for `Allocator` (#8043) (Boshen)

### Performance

- 414e828 semantic: Allocate symbol data in Allocator (#8012) (Boshen)

## [0.39.0] - 2024-12-04

### Bug Fixes

- 896ff86 minifier: Do not fold if statement block with lexical declaration (#7519) (Boshen)

## [0.37.0] - 2024-11-21

### Features

- 39afb48 allocator: Introduce `Vec::from_array_in` (#7331) (overlookmotel)

## [0.34.0] - 2024-10-26

### Features

- 419343b traverse: Implement `GetAddress` for `Ancestor` (#6877) (overlookmotel)

### Refactor

- adb5039 allocator: Add `impl GetAddress for Address` (#6891) (overlookmotel)

## [0.33.0] - 2024-10-24

- e1c2d30 allocator: [**BREAKING**] Make `Vec` non-drop (#6623) (overlookmotel)

### Bug Fixes


### Refactor

- ab8aa2f allocator: Move `GetAddress` trait into `oxc_allocator` (#6738) (overlookmotel)

## [0.32.0] - 2024-10-19

### Features

- 5ee1ef3 allocator: Add `Vec::into_boxed_slice` (#6195) (DonIsaac)

### Documentation

- 9f555d7 allocator: Clarify docs for `Box` (#6625) (overlookmotel)
- 06e75b0 allocator: Enable lint warnings on missing docs, and add missing doc comments (#6613) (DonIsaac)

## [0.31.0] - 2024-10-08

### Performance

- 5db9b30 allocator: Use lower bound of size hint when creating Vecs from an iterator (#6194) (DonIsaac)

### Refactor

- f7d1136 allocator: Remove unnecessary `Vec` impl (#6213) (overlookmotel)

## [0.30.2] - 2024-09-27

### Documentation

- 3099709 allocator: Document `oxc_allocator` crate (#6037) (DonIsaac)

## [0.27.0] - 2024-09-06

### Features

- e8bdd12 allocator: Add `AsMut` impl for `Box` (#5515) (overlookmotel)

## [0.25.0] - 2024-08-23

### Refactor

- a4247e9 allocator: Move `Box` and `Vec` into separate files (#5034) (overlookmotel)

## [0.24.3] - 2024-08-18

### Refactor

- a6967b3 allocator: Correct code comment (#4904) (overlookmotel)
- 90d0b2b allocator, ast, span, ast_tools: Use `allocator` as var name for `Allocator` (#4900) (overlookmotel)

## [0.24.2] - 2024-08-12

### Features

- 8e10e25 allocator: Introduce `Address` (#4810) (overlookmotel)

## [0.24.0] - 2024-08-08

### Features

- 23b0040 allocator: Introduce `CloneIn` trait. (#4726) (rzvxa)

## [0.23.0] - 2024-08-01

### Performance

- 4c6d19d allocator: Use capacity hint (#4584) (Luca Bruno)

## [0.22.0] - 2024-07-23

### Refactor

- 504daed allocator: Rename fn params for `Box::new_in` (#4431) (overlookmotel)

## [0.17.2] - 2024-07-08

### Features

- 115ac3b allocator: Introduce `FromIn` and `IntoIn` traits. (#4088) (rzvxa)

## [0.15.0] - 2024-06-18

### Features

- 8f5655d linter: Add eslint/no-useless-constructor (#3594) (Don Isaac)

## [0.13.0] - 2024-05-14

### Refactor

- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)

## [0.12.5] - 2024-04-22

### Refactor

- 6bc18e1 bench: Reuse allocator in parser + lexer benchmarks (#3053) (overlookmotel)

## [0.12.4] - 2024-04-19

### Features

- 063b281 allocator: Make `Box`'s PhantomData own the passed in `T` (#2952) (Boshen)

## [0.6.0] - 2024-02-03

### Documentation

- a1271af allocator: Document behaviour of `Box` (Boshen)

## [0.5.0] - 2024-01-12

### Features

- a6d9356 allocator: Add `From` API (#1908) (Boshen)

## [0.4.0] - 2023-12-08

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.2.0] - 2023-09-14

### Refactor
- 12798e0 Improve code coverage a little bit (Boshen)- fdf288c Improve code coverage in various places (#721) (Boshen)

