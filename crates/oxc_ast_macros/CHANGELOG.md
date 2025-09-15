# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).








## [0.82.3] - 2025-08-20

### 🐛 Bug Fixes

- d676e04 ast_macros: Do not panic in macro (#13102) (overlookmotel)


## [0.82.2] - 2025-08-17

### 🚜 Refactor

- c63c944 ast_macros: Simplify code (#13101) (overlookmotel)




## [0.81.0] - 2025-08-06

### 💥 BREAKING CHANGES

- 50b91ac ast: [**BREAKING**] Remove `IdentifierReference` from `qualifier` field of `TSImportType` (#12799) (camc314)


## [0.80.0] - 2025-08-03

### 💥 BREAKING CHANGES

- cd93174 ast: [**BREAKING**] Introduce `WithClauseKeyword` (#12741) (overlookmotel)

### 📚 Documentation

- de1de35 rust: Add comprehensive README.md documentation for all Rust crates (#12706) (Copilot)



## [0.79.0] - 2025-07-30

### 🚜 Refactor

- f0b1f0d napi/oxlint, napi/parser: Remove source length from `RawTransferMetadata` (#12483) (overlookmotel)


## [0.78.0] - 2025-07-24

### 🚀 Features

- dee25f4 ast: Add `pife` field to `Function` (#12469) (sapphi-red)


## [0.77.3] - 2025-07-20

### 🚀 Features

- 0920e98 codegen: Keep arrow function PIFEs (#12353) (sapphi-red)
- b0db2d7 allocator: `FixedSizeAllocator` store flag recording if owned by both Rust and JS (#12381) (overlookmotel)
- bc0fbe5 allocator: `AllocatorPool` store IDs in `Allocator`s (#12310) (overlookmotel)

### 🚜 Refactor

- c5dff1e linter, napi/parser: Add `source_len` field to `RawTransferMetadata` (#12383) (overlookmotel)
- 5e3b415 linter: Duplicate `RawTransferMetadata` in `oxc_linter` crate (#12382) (overlookmotel)
- 319fc3b allocator/fixed-size: Store `alloc_ptr` in the memory block backing the allocator (#12380) (overlookmotel)



## [0.77.1] - 2025-07-16

### 🚜 Refactor

- 5fba91c napi/parser: Raw transfer: introduce metadata struct (#12269) (overlookmotel)










# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.72.3] - 2025-06-06

### Bug Fixes

- 5c32b7c ast/estree: Make error objects via raw transfer match standard transfer (#11481) (overlookmotel)

## [0.72.0] - 2025-05-24

### Features

- c2c0268 syntax: Introduce `CommentNodeId` (#11214) (overlookmotel)

## [0.71.0] - 2025-05-20

### Features

- 9e90e00 ast_tools: Introduce `#[js_only]` attr for struct fields and converters (#11160) (overlookmotel)

### Bug Fixes

- ec96e76 ast_macros: Fix missing `syn` feature "clone-impls" (#11181) (Boshen)

### Performance

- 6571b9b ast: Use bitflags for storing comment newline state (#11096) (camchenry)
- 2b0a69f ast: Re-order struct fields to reduce padding (#11056) (overlookmotel)

## [0.70.0] - 2025-05-15

### Refactor

- c1e6c9b ast_macros: Move logic out of generated code (#11049) (overlookmotel)
- 2a403cb ast_macros: Rename generated file (#11047) (overlookmotel)

## [0.69.0] - 2025-05-09

### Styling

- 62c3a4a ast_tools: Add full stop to end of generated comments (#10809) (overlookmotel)

## [0.62.0] - 2025-04-01

### Features

- 8cd7430 allocator: `TakeIn` trait (#9969) (overlookmotel)

## [0.52.0] - 2025-02-21

### Features

- 3e7b21c ast_tools: Add `#[builder(default)]` attribute for structs and enums (#9203) (overlookmotel)

## [0.51.0] - 2025-02-15

### Features

- f74d462 ast_tools: Introduce meta types (#9117) (overlookmotel)

## [0.49.0] - 2025-02-10

- b7ff7e1 span: [**BREAKING**] Export `ContentEq` trait from root of `oxc_span` crate (#8869) (overlookmotel)

### Features

- 8f0b865 ast_tools: Generate mapping of trait name to crate in `oxc_ast_macros` (#8870) (overlookmotel)
- e693ff3 ast_tools: Generate list of helper attributes in `oxc_ast_macros` crate (#8852) (overlookmotel)

### Documentation

- 4cabc16 ast_macros: Update docs (#8853) (overlookmotel)

### Refactor

- 893339d ast: Record plural names in `#[plural]` attr (#8889) (overlookmotel)
- caa651c ast: `#[content_eq(skip)]` attr (#8875) (overlookmotel)

## [0.47.0] - 2025-01-18

- 7066d1c ast, span, syntax, regular_expression: [**BREAKING**] Remove `ContentHash` (#8512) (overlookmotel)

### Features


## [0.38.0] - 2024-11-26

### Features

- eb70219 ast: Derive `GetAddress` on all enum types (#7472) (overlookmotel)

### Documentation

- 63f4d6c ast_tools: Reformat docs for `#[ast]` proc macro (#7437) (overlookmotel)
- bc0e8bc ast_tools: Update and reformat docs for `#[ast]` proc macro (#7436) (overlookmotel)

### Refactor

- cf0b3bc ast_tools: Remove `tsify` helper attr from `Ast` derive macro (#7435) (overlookmotel)

## [0.35.0] - 2024-11-04

### Features

- 854870e ast: Label AST fields with #[ts] (#6987) (ottomated)

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

