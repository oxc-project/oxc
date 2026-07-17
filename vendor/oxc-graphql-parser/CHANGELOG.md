# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.7](https://github.com/oxc-project/oxc-graphql-parser/compare/oxc-graphql-parser-v0.0.6...oxc-graphql-parser-v0.0.7) - 2026-07-12

### Fixed

- string values ending in an escaped quote lose the quote ([#53](https://github.com/oxc-project/oxc-graphql-parser/pull/53))
- selection set loop spins forever on invalid selection start ([#52](https://github.com/oxc-project/oxc-graphql-parser/pull/52))

### Other

- extract definition_start helper for description-aware spans ([#54](https://github.com/oxc-project/oxc-graphql-parser/pull/54))
- use Peekable::next_if_eq for CRLF normalization ([#50](https://github.com/oxc-project/oxc-graphql-parser/pull/50))
- hint cold error paths with std::hint::cold_path ([#49](https://github.com/oxc-project/oxc-graphql-parser/pull/49))
- lower MSRV to Rust 1.95.0 ([#48](https://github.com/oxc-project/oxc-graphql-parser/pull/48))

## [0.0.6](https://github.com/oxc-project/oxc-graphql-parser/compare/oxc-graphql-parser-v0.0.5...oxc-graphql-parser-v0.0.6) - 2026-07-08

### Fixed

- guard object value parsing against unbounded recursion ([#36](https://github.com/oxc-project/oxc-graphql-parser/pull/36))

### Other

- move tests to integration tests ([#42](https://github.com/oxc-project/oxc-graphql-parser/pull/42))
- inline arena constructors ([#41](https://github.com/oxc-project/oxc-graphql-parser/pull/41))
- [**breaking**] remove unused trait derives ([#40](https://github.com/oxc-project/oxc-graphql-parser/pull/40))

## [0.0.5](https://github.com/oxc-project/oxc-graphql-parser/compare/oxc-graphql-parser-v0.0.4...oxc-graphql-parser-v0.0.5) - 2026-07-03

### Added

- [**breaking**] fix up parser options to align with `graphql-js` ([#22](https://github.com/oxc-project/oxc-graphql-parser/pull/22))
- support extend directive extensions ([#21](https://github.com/oxc-project/oxc-graphql-parser/pull/21))
- support directives on directives ([#20](https://github.com/oxc-project/oxc-graphql-parser/pull/20))
- support gragment arguments ([#19](https://github.com/oxc-project/oxc-graphql-parser/pull/19))

### Fixed

- more strict definition parsing ([#23](https://github.com/oxc-project/oxc-graphql-parser/pull/23))

### Other

- remove the no-op MissingNameContext trait ([#34](https://github.com/oxc-project/oxc-graphql-parser/pull/34))
- [**breaking**] match oxc_ast AST shape: box enum arms and optional fat fields ([#33](https://github.com/oxc-project/oxc-graphql-parser/pull/33))
- [**breaking**] store descriptions and selection sets as arena references ([#30](https://github.com/oxc-project/oxc-graphql-parser/pull/30))
- rewrite lexer dispatch, skip trivia in cursor, shrink spans, exact-size AST lists ([#27](https://github.com/oxc-project/oxc-graphql-parser/pull/27))

## [0.0.4](https://github.com/oxc-project/oxc-graphql-parser/compare/oxc-graphql-parser-v0.0.3...oxc-graphql-parser-v0.0.4) - 2026-07-02

### Added

- collect comment and expose ([#24](https://github.com/oxc-project/oxc-graphql-parser/pull/24))

### Other

- implement span getter for enum ([#25](https://github.com/oxc-project/oxc-graphql-parser/pull/25))

## [0.0.3](https://github.com/oxc-project/oxc-graphql-parser/compare/oxc-graphql-parser-v0.0.2...oxc-graphql-parser-v0.0.3) - 2026-07-02

### Fixed

- fragment definition bug ([#17](https://github.com/oxc-project/oxc-graphql-parser/pull/17))

## [0.0.2](https://github.com/oxc-project/oxc-graphql-parser/compare/oxc-graphql-parser-v0.0.1...oxc-graphql-parser-v0.0.2) - 2026-07-01

### Fixed

- borrow allocator for oxc_allocator 0.138 new_in API

### Other

- scan whitespace runs in a tight loop ([#12](https://github.com/oxc-project/oxc-graphql-parser/pull/12))
- avoid token clones in the parser ([#13](https://github.com/oxc-project/oxc-graphql-parser/pull/13))
- scan name tokens in a tight loop ([#11](https://github.com/oxc-project/oxc-graphql-parser/pull/11))
- inline lexer iterator next ([#10](https://github.com/oxc-project/oxc-graphql-parser/pull/10))
- inline lexer token completion ([#9](https://github.com/oxc-project/oxc-graphql-parser/pull/9))
- avoid cloning lexer errors ([#8](https://github.com/oxc-project/oxc-graphql-parser/pull/8))
- avoid definition selector allocation ([#7](https://github.com/oxc-project/oxc-graphql-parser/pull/7))
- use byte cursor in lexer ([#5](https://github.com/oxc-project/oxc-graphql-parser/pull/5))
- allocate ast with oxc_allocator ([#4](https://github.com/oxc-project/oxc-graphql-parser/pull/4))
- replace cst with direct ast ([#3](https://github.com/oxc-project/oxc-graphql-parser/pull/3))
- port shared workflows
- add workspace lint config
- move benchmarks to workspace root
- add codspeed benchmarks and rust 2024
- remove parser error screenshot
