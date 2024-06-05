# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.13.2] - 2024-06-03

### Features

* atom: get &str from Atom<'a> with lifetime of 'a (#3420)

## [0.13.0] - 2024-05-14

### Features

* transformer/jsx-source: get the correct lineNumber and columnNumber from the span. (#3142)

### Refactor

* ast: add `is_strict` methods (#3227)
* parser,diagnostic: one diagnostic struct to eliminate monomorphization of generic types (#3214)

## [0.12.5] - 2024-04-22

### Refactor

* napi: remove unnecessary custom `Serialize` impl for `Atom` (#3041)

## [0.10.0] - 2024-03-14

### Features

* span: `impl<'a> PartialEq<str> for Atom<'a>` (#2649)
* span: remove `From<String>` and `From<Cow>` API because they create memory leak (#2628)- merge features `serde` and `wasm` to `serialize` (#2716) |- miette v7 (#2465) |

### Refactor

* ast: import `Tsify` to shorten code (#2665)
* ast: shorten manual TS defs (#2638)
* span: change shape of `Language` (#2680)
* span: simplify `Atom` (#2630)
* span: remove `Atom::Compact` variant (#2629)
* span: disallow struct expression constructor for `Span` (#2625)- derive `SerAttrs` on all AST types (#2698) |- reduce `cfg_attr` boilerplate with `SerAttrs` derive (#2669) |- "wasm" feature enable "serde" feature (#2639) |- make `CompactStr` immutable (#2620) |- rename `CompactString` to `CompactStr` (#2619) |

## [0.9.0] - 2024-03-05

### Refactor

* span: remove `AtomImpl` (#2525)
* span: move base54 method to mangler (#2523)- replace InlinableString with CompactString for `Atom` (#2517) |

## [0.7.0] - 2024-02-09

### Features

* span: fix memory leak by implementing inlineable string for oxc_allocator (#2294)

## [0.6.0] - 2024-02-03

### Features

* ast: TypeScript definition for wasm target (#2158)

## [0.4.0] - 2023-12-08

### Refactor

* rust: move to workspace lint table (#1444)

## [0.3.0] - 2023-11-06

### Features

* linter: support react/no-render-return-value (#1042)

### Refactor

* transformer: add an empty SPAN utility for creating AST nodes (#1067)

### Testing

* semantic: add scoping test cases (#954)

## [0.2.0] - 2023-09-14

### Features

* minifier: constant addition expression folding (#882)

### Refactor
- improve code coverage in various places (#721) |

