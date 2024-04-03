# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.11.1] - 2024-04-03

### Features

- Add compiler assumptions (#2872)

## [0.11.0] - 2024-03-30

### Features

- Distinguish type imports in ModuleRecord (#2785)

### Bug Fixes

- Ignore export declaration in no-duplicates (#2863)

### Refactor

- Distinguish whether requested_modules is type imports/exports (#2848)

## [0.10.0] - 2024-03-14

### Features

- Merge features `serde` and `wasm` to `serialize` (#2716)
- Resolve ESM star exports (#2682)

### Refactor

- Derive `SerAttrs` on all AST types (#2698)
- Reduce `cfg_attr` boilerplate with `SerAttrs` derive (#2669)
- Import `Tsify` to shorten code (#2665)
- "wasm" feature enable "serde" feature (#2639)
- Shorten manual TS defs (#2638)
- Rename `CompactString` to `CompactStr` (#2619)

## [0.9.0] - 2024-03-05

### Features

- Remove all commonjs logic for import plugin (#2537)

## [0.8.0] - 2024-02-26

### Features

- Handle cjs `module.exports.foo = bar` and `exports.foo = bar` (#2492)
- Implement `Debug` for `ModuleRecord` (#2488)
- Improve codegen (#2460)
- Add static property, ElementKind::Getter, ElementKind::Setter in ClassTable (#2445)

### Bug Fixes

- Improve import/no-named-as-default (#2494)

### Refactor

- Get arrow expression by scope_id in no_render_return_value (#2424)

## [0.7.0] - 2024-02-09

### Bug Fixes

- Remove unnecessary SymbolFlags::Import (#2311)

## [0.6.0] - 2024-02-03

### Features

- Remove serde skip for symbol_id and reference_id (#2220)
- TypeScript definition for wasm target (#2158)
- Remove import if only have type reference (#2001)

### Bug Fixes

- Add parenthesis in binary expression by precedence (#2067)

### Refactor

- Don't re-export `unicode_id_start`
- Make `is_identifier` methods consistent
- ASCII tables static not const (#2128)
- Reformat identifier byte tables (#2111)

## [0.5.0] - 2024-01-12

### Features

- Allow reserved keyword defined in ts module block (#1907)
- Visualize symbol (#1886)
- Improve check super implementation, reduce access nodes (#1827)
- Add ClassTable (#1793)
- Add eslint-plugin-import(export) rule (#1654)

### Refactor

- Improve ClassTable implmention and merge properties and methods to elements (#1902)

## [0.4.0] - 2023-12-08

### Features

- Binaryish expressions with parens (#1597)
- Check parens for `(let)[a] = 1` (#1585)
- Support quoteProps option in PropertyKey (#1578)

### Refactor

- Move to workspace lint table (#1444)

## [0.3.0] - 2023-11-06

### Features

- Escape xhtml in jsx attributes (#1088)
- Read plugins options from babel `options.json` (#1006)
- Check non-simple lhs expression of assignment expression (#994)
- Partially re-enable minifier (#963)
- Logical assignment operators (#923)

## [0.2.0] - 2023-09-14

### Features

- Add loaded_modules to ModuleRecord
- Add `SymbolId` and `ReferenceId` (#755)
- Initialize conditions folding (#658)

### Performance

- Reduce mallocs (#654)

### Refactor

- Improve code coverage a little bit
- Improve code coverage in various places (#721)

