# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.13.2] - 2024-06-03

### Features

* oxc_codegen: preserve annotate comment (#3465)

## [0.13.1] - 2024-05-22

### Features

* syntax: export `is_reserved_keyword` and `is_global_object` method (#3384)

## [0.13.0] - 2024-05-14

### Refactor

* ast: squash nested enums (#3115)
* syntax: move number related functions to number module (#3130)

## [0.11.0] - 2024-03-30

### Refactor

* ast: add walk_mut functions (#2776)
* sourcemap: change sourcemap name to take a reference (#2779)

## [0.10.0] - 2024-03-14

### Features

* span: remove `From<String>` and `From<Cow>` API because they create memory leak (#2628)

### Refactor
- remove unused dependencies (#2718) |- make `CompactStr` immutable (#2620) |- rename `CompactString` to `CompactStr` (#2619) |

## [0.9.0] - 2024-03-05

### Refactor

* codegen: clean up API around building sourcemaps (#2602)
* span: move base54 method to mangler (#2523)

## [0.8.0] - 2024-02-26

### Features

* Codegen: Improve codegen (#2460)
* codegen: configurable typescript codegen (#2443)

### Refactor

* ast: s/NumberLiteral/NumericLiteral to align with estree
* ast: s/ArrowExpression/ArrowFunctionExpression to align estree- remove `panic!` from examples (#2454) |- remove global allocator from non-user facing apps (#2401) |

## [0.7.0] - 2024-02-09

### Refactor

* ast: fix BigInt memory leak by removing it (#2293)

## [0.6.0] - 2024-02-03

### Features

* codegen: move string test to codegen (#2150)
* minifier: handle more expressions for side effects (#2062)

### Bug Fixes

* codegen: add parenthesis in binary expression by precedence (#2067)

## [0.4.0] - 2023-12-08

### Features

* semantic: support scope descendents starting from a certain scope. (#1629)

### Refactor

* rust: move to workspace lint table (#1444)

## [0.3.0] - 2023-11-06

### Features

* codegen: implement the basics of non-minifying codegen (#987)
* codegen: move minifying printer to codegen crate (#985)
* minifier: re-enable mangler (#972)
* minifier: reenable minifier tests (#969)
* minifier: reenable mangler
* minifier: partially re-enable minifier (#963)
* parser: TypeScript 5.2 (#811)
* playground: add transform and minify (#993)
* transform_conformance: move Formatter to codegen (#986)
* transformer: ES2020 Nullish Coalescing Operator (#1004)
* transformer: finish 2016 exponentiation operator (#996)- adjust the order of print semicolon (#1003) |

### Refactor

* ast: clean up some methods
* ast: fix the lifetime annotations around Vist and VisitMut (#973)
* clippy: allow clippy::too_many_lines
* clippy: allow struct_excessive_bools
* minifier: make the minifier api only accept an ast (#990)
* rust: change `RefCell.clone().into_inner()` to `RefCell.get()`

## [0.2.0] - 2023-09-14

### Features

* minifier: constant addition expression folding (#882)
* minifier: initialize conditions folding (#658)
* semantic: add `node_id` to `Reference` (#689)

### Refactor

* ast: use `atom` for `Directive` and `Hashbang` (#701)

