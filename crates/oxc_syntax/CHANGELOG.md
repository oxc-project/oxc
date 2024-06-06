# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.13.1] - 2024-05-22

### Features

- e2dd8ac syntax: Export `is_reserved_keyword` and `is_global_object` method (#3384) (Boshen)

### Bug Fixes

- b4fa27a transformer: Do no add __self when the jsx is inside constructor (#3258) (Dunqing)

## [0.13.0] - 2024-05-14

### Features

- f1ccbd4 syntax: Add `ToJsInt32` trait for f64 (#3132) (Boshen)
- 870d11f syntax: Add `ToJsString` trait for f64 (#3131) (Boshen)
- 46c02ae traverse: Add scope flags to `TraverseCtx` (#3229) (overlookmotel)

### Bug Fixes

- 4e20b04 traverse: Create scope for function nested in class method (#3234) (overlookmotel)

### Refactor

- a8af5de syntax: Move number related functions to number module (#3130) (Boshen)
- ae65613 syntax: Use `FxHashMap` for `ModuleRecord::request_modules` (#3124) (Boshen)

## [0.12.5] - 2024-04-22

### Features

- e6d11c6 syntax: Module graph visitor. (#3062) (Ali Rezvani)

### Bug Fixes

- 1f7033e semantic: Correctly resolve identifiers inside parameter initializers (#3046) (Boshen)

### Refactor

- 1249c6c ast: Implement same traits on all fieldless enums (#3031) (overlookmotel)

## [0.12.2] - 2024-04-08

### Bug Fixes

- 1ea24ea semantic: Symbols inside functions and classes incorrectly flagged as exported (#2896) (Don Isaac)

## [0.12.1] - 2024-04-03

### Features

- 7710d8c transformer: Add compiler assumptions (#2872) (Boshen)

## [0.11.0] - 2024-03-30

### Features

- 712b3d2 semantic: Distinguish type imports in ModuleRecord (#2785) (Dunqing)

### Bug Fixes

- df62828 linter/import: Ignore export declaration in no-duplicates (#2863) (Dunqing)

### Refactor

- 1b5e544 semantic: Distinguish whether requested_modules is type imports/exports (#2848) (Dunqing)

## [0.10.0] - 2024-03-14

### Features

- 366a879 linter: Resolve ESM star exports (#2682) (Boshen)- 697b6b7 Merge features `serde` and `wasm` to `serialize` (#2716) (Boshen)

### Refactor

- cba1e2f ast: Import `Tsify` to shorten code (#2665) (overlookmotel)
- 6b5723c ast: Shorten manual TS defs (#2638) (overlookmotel)- 89e8d15 Derive `SerAttrs` on all AST types (#2698) (overlookmotel)- 3c1e0db Reduce `cfg_attr` boilerplate with `SerAttrs` derive (#2669) (overlookmotel)- d76ee6b "wasm" feature enable "serde" feature (#2639) (overlookmotel)- 0646bf3 Rename `CompactString` to `CompactStr` (#2619) (overlookmotel)

## [0.9.0] - 2024-03-05

### Features

- d41dcc3 linter: Remove all commonjs logic for import plugin (#2537) (Boshen)

## [0.8.0] - 2024-02-26

### Features

- 6b3b260 Codegen: Improve codegen (#2460) (Andrew McClenaghan)
- f64c7e0 linter: Handle cjs `module.exports.foo = bar` and `exports.foo = bar` (#2492) (Boshen)
- 950298d semantic: Add static property, ElementKind::Getter, ElementKind::Setter in ClassTable (#2445) (Dunqing)
- 3e1794d syntax: Implement `Debug` for `ModuleRecord` (#2488) (Boshen)

### Bug Fixes

- fba66dc linter: Improve import/no-named-as-default (#2494) (Boshen)

### Refactor

- 67d7a46 linter: Get arrow expression by scope_id in no_render_return_value (#2424) (Dunqing)

## [0.7.0] - 2024-02-09

### Bug Fixes

- 540b2a0 semantic: Remove unnecessary SymbolFlags::Import (#2311) (Dunqing)

## [0.6.0] - 2024-02-03

### Features

- f673e41 ast: Remove serde skip for symbol_id and reference_id (#2220) (Dunqing)
- cd5026c ast: TypeScript definition for wasm target (#2158) (Nicholas Roberts)
- ead4e8d transformer/typescript: Remove import if only have type reference (#2001) (Dunqing)

### Bug Fixes

- 29dc5e6 codegen: Add parenthesis in binary expression by precedence (#2067) (Wenzhe Wang)

### Refactor

- bc7ea0b parser: Make `is_identifier` methods consistent (overlookmotel)
- 0dc1804 syntax: Don't re-export `unicode_id_start` (overlookmotel)
- 27aaff2 syntax: ASCII tables static not const (#2128) (overlookmotel)
- 4f59c4f syntax: Reformat identifier byte tables (#2111) (overlookmotel)

## [0.5.0] - 2024-01-12

### Features

- 90524c8 linter: Add eslint-plugin-import(export) rule (#1654) (Wenzhe Wang)
- f1b433b playground: Visualize symbol (#1886) (Dunqing)
- 3b4fe0e semantic: Allow reserved keyword defined in ts module block (#1907) (Dunqing)
- b9bdf36 semantic: Improve check super implementation, reduce access nodes (#1827) (Dunqing)
- ca04312 semantic: Add ClassTable (#1793) (Dunqing)

### Refactor

- 6c5b22f semantic: Improve ClassTable implmention and merge properties and methods to elements (#1902) (Dunqing)

## [0.4.0] - 2023-12-08

### Features

- da87b9b prettier: Binaryish expressions with parens (#1597) (Boshen)
- 1bd1c5b prettier: Check parens for `(let)[a] = 1` (#1585) (Boshen)
- f19032e prettier: Support quoteProps option in PropertyKey (#1578) (Dunqing)

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.3.0] - 2023-11-06

### Features

- 55b2f03 minifier: Partially re-enable minifier (#963) (Boshen)
- 1661385 semantic: Check non-simple lhs expression of assignment expression (#994) (Boshen)
- 5863f8f transformer: Logical assignment operators (#923) (Boshen)
- 1051f15 transformer/jsx: Escape xhtml in jsx attributes (#1088) (Boshen)
- 1b3b100 transformer_conformance: Read plugins options from babel `options.json` (#1006) (Boshen)

## [0.2.0] - 2023-09-14

### Features

- e7c2313 ast: Add `SymbolId` and `ReferenceId` (#755) (Yunfei He)
- e090b56 minifier: Initialize conditions folding (#658) (阿良仔)
- 75d928a syntax: Add loaded_modules to ModuleRecord (Boshen)

### Performance

- 6628fc8 linter: Reduce mallocs (#654) (Don Isaac)

### Refactor
- 12798e0 Improve code coverage a little bit (Boshen)- fdf288c Improve code coverage in various places (#721) (Boshen)

