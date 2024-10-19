# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.32.0] - 2024-10-19

### Features

- e310e52 parser: Generate `Serialize` impls in ast_tools (#6404) (ottomated)

### Documentation

- de22b81 data-structures: Enable lint warnings on missing docs, and add missing doc comments (#6612) (DonIsaac)
- 9e9fa9e span: Enable lint warnings on missing docs (#6617) (overlookmotel)
- 6a194f9 span: Document validity of `ModuleKind::Unambiguous` (#6423) (Boshen)

### Refactor

- 3faee66 span: Remove unused `ContentHash::content_hash_slice` (#6609) (DonIsaac)

## [0.30.4] - 2024-09-28

### Bug Fixes

- fd6798f parser: Remove unintended `pub Kind` (#6109) (Boshen)

## [0.30.0] - 2024-09-23

### Features

- a5f2e9a span: Impl `From<Atom<'a>>` for `Atom` (#5809) (DonIsaac)
- a07f03a transformer: Sync `Program::source_type` after transform (#5887) (Boshen)

## [0.28.0] - 2024-09-11

- 4a8aec1 span: [**BREAKING**] Change `SourceType::js` to `SourceType::cjs` and `SourceType::mjs` (#5606) (Boshen)

- 603817b oxc: [**BREAKING**] Add `SourceType::Unambiguous`; parse `.js` as unambiguous (#5557) (Boshen)

### Features

- b3cbd56 span: `format_compact_str!` macro (#5610) (overlookmotel)

### Bug Fixes

- 28b934c coverage: Apply `always_strict` to test262 and typescript per the specifcation (#5555) (Boshen)
- f49e6eb span: Treat `.js` as `module` file (reverts the previous breaking change) (#5612) (Boshen)

### Refactor

- 3d190a5 span: Move `CompactStr` into separate file (#5609) (overlookmotel)
- 5532628 span: Put types and impl in the same mod file (Boshen)

## [0.27.0] - 2024-09-06

### Features

- 90facd3 ast: Add `ContentHash` trait; remove noop `Hash` implementation from `Span` (#5451) (rzvxa)
- 23285f4 ast: Add `ContentEq` trait. (#5427) (rzvxa)

### Documentation

- 3f204a9 span: Update docs about `ContentEq` `Vec` comparison speed (#5478) (overlookmotel)

### Refactor

- 9f6e0ed ast: Simplify `ContentEq` trait definition. (#5468) (rzvxa)
- 94a6ac6 span: Use `Hasher` from `std` (#5476) (overlookmotel)

## [0.26.0] - 2024-09-03

### Features

- f81e8a1 linter: Add `oxc/no-async-endpoint-handlers` (#5364) (DonIsaac)
- 1b20ceb span: Add `CompactStr::to_compact_string` method (#5385) (Boshen)
- 5a137f0 span/source-type: Add SourceType factory methods (#5242) (DonIsaac)
- f5e05db span/source-type: Impl `Display` and `Error` for `UnknownExtension` (#5240) (DonIsaac)

### Bug Fixes

- a6bb3b1 span/source-type: Consider `.cjs` and `.cts` files as `ModuleKind::Script` (#5239) (DonIsaac)

### Refactor

- 94f60e7 span/source-type: Make `SourceType` factories `const` (#5241) (DonIsaac)

## [0.25.0] - 2024-08-23

### Refactor

- 7706523 span: Clarify `Atom` conversion methods lifetimes (#4978) (overlookmotel)

## [0.24.3] - 2024-08-18

### Refactor

- 90d0b2b allocator, ast, span, ast_tools: Use `allocator` as var name for `Allocator` (#4900) (overlookmotel)

## [0.24.2] - 2024-08-12

### Refactor

- 096ac7b linter: Clean up jsx-a11y/anchor-is-valid (#4831) (DonIsaac)

## [0.24.1] - 2024-08-10

### Features

- b3c3125 linter: Overhaul unicorn/no-useless-spread (#4791) (DonIsaac)

## [0.24.0] - 2024-08-08

### Features

- 54047e0 ast: `GetSpanMut` trait (#4609) (overlookmotel)
- 07607d3 ast_codegen, span: Process `Span` through ast_codegen (#4703) (overlookmotel)
- 125c5fd ast_codegen, span: Process `SourceType` through ast_codegen. (#4696) (rzvxa)
- 2e63618 span: Implement `CloneIn` for the AST-related items. (#4729) (rzvxa)

### Performance

- 6ff200d linter: Change react rules and utils to use `Cow` and `CompactStr` instead of `String`  (#4603) (DonIsaac)

### Refactor

- e1429e5 span: Reduce #[cfg_attr] boilerplate in type defs (#4702) (overlookmotel)

## [0.23.1] - 2024-08-06

### Features

- 54047e0 ast: `GetSpanMut` trait (#4609) (overlookmotel)

### Performance

- 6ff200d linter: Change react rules and utils to use `Cow` and `CompactStr` instead of `String`  (#4603) (DonIsaac)

## [0.22.1] - 2024-07-27

### Features

- e2735ca span: Add `contains_inclusive` method (#4491) (DonIsaac)

## [0.22.0] - 2024-07-23

### Bug Fixes
- ea33f94 Impl PartialEq<str> for CompactStr (#4352) (DonIsaac)

### Performance
- a207923 Replace some CompactStr usages with Cows (#4377) (DonIsaac)

## [0.18.0] - 2024-07-09

### Features

- 44c7fe3 span: Add various implementations of `FromIn` for `Atom`. (#4090) (rzvxa)

## [0.16.1] - 2024-06-29

### Refactor

- 2705df9 linter: Improve diagnostic labeling (#3960) (DonIsaac)

## [0.14.0] - 2024-06-12

### Features

- 129f91e span: Port over more methods from TextRange (#3592) (Don Isaac)

### Bug Fixes

- d65202d span: Correct doc comments (#3608) (overlookmotel)
- 9e8f4d6 transformer: Do not add `__source` for generated nodes (#3614) (overlookmotel)

### Refactor

- f98f777 linter: Add rule fixer (#3589) (Don Isaac)

## [0.13.4] - 2024-06-07

### Performance

- 9f467b8 transformer: Avoid fragment update where possible (#3535) (overlookmotel)

### Documentation

- 1d3c0d7 span: Add doc comments to `oxc_span::Span` (#3543) (Don Isaac)

## [0.13.2] - 2024-06-03

### Features

- 679495c atom: Get &str from Atom<'a> with lifetime of 'a (#3420) (Don Isaac)

## [0.13.0] - 2024-05-14

### Features

- a52e321 transformer/jsx-source: Get the correct lineNumber and columnNumber from the span. (#3142) (Dunqing)

### Refactor

- c84c116 ast: Add `is_strict` methods (#3227) (overlookmotel)
- 2064ae9 parser,diagnostic: One diagnostic struct to eliminate monomorphization of generic types (#3214) (Boshen)

## [0.12.5] - 2024-04-22

### Refactor

- 27102df napi: Remove unnecessary custom `Serialize` impl for `Atom` (#3041) (overlookmotel)

## [0.10.0] - 2024-03-14

### Features

- 8b3de77 span: `impl<'a> PartialEq<str> for Atom<'a>` (#2649) (Boshen)
- 4f9dd98 span: Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628) (Boshen)- 697b6b7 Merge features `serde` and `wasm` to `serialize` (#2716) (Boshen)- 265b2fb Miette v7 (#2465) (Boshen)

### Refactor

- cba1e2f ast: Import `Tsify` to shorten code (#2665) (overlookmotel)
- 6b5723c ast: Shorten manual TS defs (#2638) (overlookmotel)
- 75ae563 span: Change shape of `Language` (#2680) (overlookmotel)
- b2de57a span: Simplify `Atom` (#2630) (overlookmotel)
- cb4e054 span: Remove `Atom::Compact` variant (#2629) (Boshen)
- 798a6df span: Disallow struct expression constructor for `Span` (#2625) (Boshen)- 89e8d15 Derive `SerAttrs` on all AST types (#2698) (overlookmotel)- 3c1e0db Reduce `cfg_attr` boilerplate with `SerAttrs` derive (#2669) (overlookmotel)- d76ee6b "wasm" feature enable "serde" feature (#2639) (overlookmotel)- 8001b2f Make `CompactStr` immutable (#2620) (overlookmotel)- 0646bf3 Rename `CompactString` to `CompactStr` (#2619) (overlookmotel)

## [0.9.0] - 2024-03-05

### Refactor

- 27052eb span: Remove `AtomImpl` (#2525) (Boshen)
- 903f17c span: Move base54 method to mangler (#2523) (Boshen)- c56b6cb Replace InlinableString with CompactString for `Atom` (#2517) (Boshen)

## [0.7.0] - 2024-02-09

### Features

- 6002560 span: Fix memory leak by implementing inlineable string for oxc_allocator (#2294) (Boshen)

## [0.6.0] - 2024-02-03

### Features

- cd5026c ast: TypeScript definition for wasm target (#2158) (Nicholas Roberts)

## [0.4.0] - 2023-12-08

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.3.0] - 2023-11-06

### Features

- d8f07ca linter: Support react/no-render-return-value (#1042) (Dunqing)

### Refactor

- d9ba532 transformer: Add an empty SPAN utility for creating AST nodes (#1067) (Boshen)

### Testing

- b4b39b8 semantic: Add scoping test cases (#954) (Don Isaac)

## [0.2.0] - 2023-09-14

### Features

- 027a67d minifier: Constant addition expression folding (#882) (Don Isaac)

### Refactor
- fdf288c Improve code coverage in various places (#721) (Boshen)

