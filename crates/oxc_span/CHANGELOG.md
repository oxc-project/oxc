# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

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

