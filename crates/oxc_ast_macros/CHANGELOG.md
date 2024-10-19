# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.32.0] - 2024-10-19

### Features

- e310e52 parser: Generate `Serialize` impls in ast_tools (#6404) (ottomated)

## [0.30.0] - 2024-09-23

### Refactor

- 17cd903 ast: Move functions to top level in `ast` macro (#5793) (overlookmotel)
- cf97f6d ast: Import `syn` types in `ast` macro (#5792) (overlookmotel)
- dc10eaf ast: Split `ast` macro into multiple files (#5791) (overlookmotel)

## [0.27.0] - 2024-09-06

### Features

- 90facd3 ast: Add `ContentHash` trait; remove noop `Hash` implementation from `Span` (#5451) (rzvxa)
- 23285f4 ast: Add `ContentEq` trait. (#5427) (rzvxa)

### Refactor

- 9f6e0ed ast: Simplify `ContentEq` trait definition. (#5468) (rzvxa)
- b47aca0 syntax: Use `generate_derive` for `CloneIn` in types outside of `oxc_ast` crate. (#5280) (rzvxa)

## [0.24.3] - 2024-08-18

### Documentation

- 47c9552 ast, ast_macros, ast_tools: Better documentation for `Ast` helper attributes. (#4856) (rzvxa)

## [0.24.2] - 2024-08-12

### Refactor

- 0ea697b ast, ast_codegen: `CloneIn` implementations now initialize semantic related cells with `Default` value. (#4819) (rzvxa)

## [0.24.1] - 2024-08-10

### Bug Fixes

- f5eeebd ast_macros: Raise compile error on invalid `generate_derive` input. (#4766) (rzvxa)

### Refactor

- 7ea058d ast_codegen: Replace Windows-style line breaks with Unix-style (#4769) (overlookmotel)

## [0.24.0] - 2024-08-08

### Features

- eae401c ast, ast_macros: Apply stable repr to all `#[ast]` enums (#4373) (rzvxa)
- 2e91ad6 ast_codegen: Support for `generate_derive` marker. (#4728) (rzvxa)
- 6a36616 syntax: Derive `CloneIn` for the AST-related items. (#4730) (rzvxa)

### Bug Fixes

- 94d3c31 minifier: Avoid removing function declaration from `KeepVar` (#4722) (Boshen)
- f290191 oxc_ast_macros: Fix `syn` lacking features to build (Boshen)
- a40a217 parser: Parse `assert` keyword in `TSImportAttributes` (#4610) (Boshen)

### Refactor

- 3f53b6f ast: Make AST structs `repr(C)`. (#4614) (rzvxa)
- 452e0ee ast: Remove defunct `visit_as` + `visit_args` attrs from `#[ast]` macro (#4599) (overlookmotel)

## [0.23.1] - 2024-08-06

### Features

- eae401c ast, ast_macros: Apply stable repr to all `#[ast]` enums (#4373) (rzvxa)

### Bug Fixes

- a40a217 parser: Parse `assert` keyword in `TSImportAttributes` (#4610) (Boshen)

### Refactor

- 3f53b6f ast: Make AST structs `repr(C)`. (#4614) (rzvxa)
- 452e0ee ast: Remove defunct `visit_as` + `visit_args` attrs from `#[ast]` macro (#4599) (overlookmotel)

## [0.22.0] - 2024-07-23

### Refactor

- abfccbd ast: Reduce `#[cfg_attr]` boilerplate in AST type defs (#4375) (overlookmotel)
- 5f1c7ec ast: Rename the `visited_node` marker to `ast`. (#4289) (rzvxa)

## [0.17.0] - 2024-07-05

### Features

- 1854a52 ast_codegen: Introduce the `#[span]` hint. (#4012) (rzvxa)
- 7538af1 ast_codegen: Add visit generator (#3954) (rzvxa)

## [0.16.0] - 2024-06-26

### Refactor

- fcd21a6 traverse: Indicate scope entry point with `scope(enter_before)` attr (#3882) (overlookmotel)

## [0.13.0] - 2024-05-14

### Features

- be87ca8 transform: `oxc_traverse` crate (#3169) (overlookmotel)

