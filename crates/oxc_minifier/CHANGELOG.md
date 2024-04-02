# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.11.0] - 2024-03-30

### Refactor

- Add walk_mut functions (#2776)
- Change sourcemap name to take a reference (#2779)

## [0.10.0] - 2024-03-14

### Features

- Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628)

### Refactor

- Remove unused dependencies (#2718)
- Make `CompactStr` immutable (#2620)
- Rename `CompactString` to `CompactStr` (#2619)

## [0.9.0] - 2024-03-05

### Refactor

- Clean up API around building sourcemaps (#2602)
- Move base54 method to mangler (#2523)

## [0.8.0] - 2024-02-26

### Features

- Improve codegen (#2460)
- Configurable typescript codegen (#2443)

### Refactor

- S/NumberLiteral/NumericLiteral to align with estree
- S/ArrowExpression/ArrowFunctionExpression to align estree
- Remove `panic!` from examples (#2454)
- Remove global allocator from non-user facing apps (#2401)

## [0.7.0] - 2024-02-09

### Refactor

- Fix BigInt memory leak by removing it (#2293)

## [0.6.0] - 2024-02-03

### Features

- Move string test to codegen (#2150)
- Handle more expressions for side effects (#2062)

### Bug Fixes

- Add parenthesis in binary expression by precedence (#2067)

## [0.4.0] - 2023-12-08

### Features

- Support scope descendents starting from a certain scope. (#1629)

### Refactor

- Move to workspace lint table (#1444)

## [0.3.0] - 2023-11-06

### Features

- ES2020 Nullish Coalescing Operator (#1004)
- Adjust the order of print semicolon (#1003)
- Finish 2016 exponentiation operator (#996)
- Add transform and minify (#993)
- Implement the basics of non-minifying codegen (#987)
- Move Formatter to codegen (#986)
- Move minifying printer to codegen crate (#985)
- Re-enable mangler (#972)
- Reenable minifier tests (#969)
- Reenable mangler
- Partially re-enable minifier (#963)
- TypeScript 5.2 (#811)

### Refactor

- Allow clippy::too_many_lines
- Allow struct_excessive_bools
- Change `RefCell.clone().into_inner()` to `RefCell.get()`
- Clean up some methods
- Make the minifier api only accept an ast (#990)
- Fix the lifetime annotations around Vist and VisitMut (#973)

## [0.2.0] - 2023-09-14

### Features

- Constant addition expression folding (#882)
- Add `node_id` to `Reference` (#689)
- Initialize conditions folding (#658)

### Refactor

- Use `atom` for `Directive` and `Hashbang` (#701)

