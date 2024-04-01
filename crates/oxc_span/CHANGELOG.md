# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.10.0] - 2024-03-14

### Features

- Merge features `serde` and `wasm` to `serialize` (#2716)
- `impl<'a> PartialEq<str> for Atom<'a>` (#2649)
- Miette v7 (#2465)
- Remove `From<String>` and `From<Cow>` API because they create memory leak (#2628)

### Refactor

- Derive `SerAttrs` on all AST types (#2698)
- Change shape of `Language` (#2680)
- Reduce `cfg_attr` boilerplate with `SerAttrs` derive (#2669)
- Import `Tsify` to shorten code (#2665)
- "wasm" feature enable "serde" feature (#2639)
- Shorten manual TS defs (#2638)
- Simplify `Atom` (#2630)
- Remove `Atom::Compact` variant (#2629)
- Disallow struct expression constructor for `Span` (#2625)
- Make `CompactStr` immutable (#2620)
- Rename `CompactString` to `CompactStr` (#2619)

## [0.9.0] - 2024-03-05

### Refactor

- Remove `AtomImpl` (#2525)
- Move base54 method to mangler (#2523)
- Replace InlinableString with CompactString for `Atom` (#2517)

## [0.7.0] - 2024-02-09

### Features

- Fix memory leak by implementing inlineable string for oxc_allocator (#2294)

## [0.6.0] - 2024-02-03

### Features

- TypeScript definition for wasm target (#2158)

## [0.4.0] - 2023-12-08

### Refactor

- Move to workspace lint table (#1444)

## [0.3.0] - 2023-11-06

### Features

- Support react/no-render-return-value (#1042)

### Refactor

- Add an empty SPAN utility for creating AST nodes (#1067)

### Testing

- Add scoping test cases (#954)

## [0.2.0] - 2023-09-14

### Features

- Constant addition expression folding (#882)

### Refactor

- Improve code coverage in various places (#721)

