# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.13.1] - 2024-05-22

### Features

* syntax: export `is_reserved_keyword` and `is_global_object` method (#3384)

### Bug Fixes

* transformer: do no add __self when the jsx is inside constructor (#3258)

## [0.13.0] - 2024-05-14

### Features

* syntax: add `ToJsInt32` trait for f64 (#3132)
* syntax: add `ToJsString` trait for f64 (#3131)
* traverse: add scope flags to `TraverseCtx` (#3229)

### Bug Fixes

* traverse: create scope for function nested in class method (#3234)

### Refactor

* syntax: move number related functions to number module (#3130)
* syntax: use `FxHashMap` for `ModuleRecord::request_modules` (#3124)

## [0.12.5] - 2024-04-22

### Features

* syntax: module graph visitor. (#3062)

### Bug Fixes

* semantic: correctly resolve identifiers inside parameter initializers (#3046)

### Refactor

* ast: implement same traits on all fieldless enums (#3031)

## [0.12.2] - 2024-04-08

### Bug Fixes

* semantic: symbols inside functions and classes incorrectly flagged as exported (#2896)

## [0.12.1] - 2024-04-03

### Features

* transformer: add compiler assumptions (#2872)

## [0.11.0] - 2024-03-30

### Features

* semantic: distinguish type imports in ModuleRecord (#2785)

### Bug Fixes

* linter/import: ignore export declaration in no-duplicates (#2863)

### Refactor

* semantic: distinguish whether requested_modules is type imports/exports (#2848)

## [0.10.0] - 2024-03-14

### Features

* linter: resolve ESM star exports (#2682)- merge features `serde` and `wasm` to `serialize` (#2716) |

### Refactor

* ast: import `Tsify` to shorten code (#2665)
* ast: shorten manual TS defs (#2638)- derive `SerAttrs` on all AST types (#2698) |- reduce `cfg_attr` boilerplate with `SerAttrs` derive (#2669) |- "wasm" feature enable "serde" feature (#2639) |- rename `CompactString` to `CompactStr` (#2619) |

## [0.9.0] - 2024-03-05

### Features

* linter: remove all commonjs logic for import plugin (#2537)

## [0.8.0] - 2024-02-26

### Features

* Codegen: Improve codegen (#2460)
* linter: handle cjs `module.exports.foo = bar` and `exports.foo = bar` (#2492)
* semantic: add static property, ElementKind::Getter, ElementKind::Setter in ClassTable (#2445)
* syntax: implement `Debug` for `ModuleRecord` (#2488)

### Bug Fixes

* linter: improve import/no-named-as-default (#2494)

### Refactor

* linter: get arrow expression by scope_id in no_render_return_value (#2424)

## [0.7.0] - 2024-02-09

### Bug Fixes

* semantic: remove unnecessary SymbolFlags::Import (#2311)

## [0.6.0] - 2024-02-03

### Features

* ast: remove serde skip for symbol_id and reference_id (#2220)
* ast: TypeScript definition for wasm target (#2158)
* transformer/typescript: remove import if only have type reference (#2001)

### Bug Fixes

* codegen: add parenthesis in binary expression by precedence (#2067)

### Refactor

* parser: make `is_identifier` methods consistent
* syntax: don't re-export `unicode_id_start`
* syntax: ASCII tables static not const (#2128)
* syntax: reformat identifier byte tables (#2111)

## [0.5.0] - 2024-01-12

### Features

* linter: add eslint-plugin-import(export) rule (#1654)
* playground: visualize symbol (#1886)
* semantic: allow reserved keyword defined in ts module block (#1907)
* semantic: improve check super implementation, reduce access nodes (#1827)
* semantic: add ClassTable (#1793)

### Refactor

* semantic: improve ClassTable implmention and merge properties and methods to elements (#1902)

## [0.4.0] - 2023-12-08

### Features

* prettier: binaryish expressions with parens (#1597)
* prettier: check parens for `(let)[a] = 1` (#1585)
* prettier: support quoteProps option in PropertyKey (#1578)

### Refactor

* rust: move to workspace lint table (#1444)

## [0.3.0] - 2023-11-06

### Features

* minifier: partially re-enable minifier (#963)
* semantic: check non-simple lhs expression of assignment expression (#994)
* transformer: logical assignment operators (#923)
* transformer/jsx: escape xhtml in jsx attributes (#1088)
* transformer_conformance: read plugins options from babel `options.json` (#1006)

## [0.2.0] - 2023-09-14

### Features

* ast: add `SymbolId` and `ReferenceId` (#755)
* minifier: initialize conditions folding (#658)
* syntax: add loaded_modules to ModuleRecord

### Performance

* linter: reduce mallocs (#654)

### Refactor
- improve code coverage a little bit |- improve code coverage in various places (#721) |

