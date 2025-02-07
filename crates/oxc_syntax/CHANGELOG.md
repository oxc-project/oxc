# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.48.2] - 2025-02-02

### Bug Fixes

- 8a6ae8a minifier: Do not change `const` to `let` if assignment to constant variable. (#8761) (Boshen)

### Refactor

- 0568210 ast: Remove excess line breaks from generated code (#8830) (overlookmotel)

## [0.48.0] - 2025-01-24

### Refactor

- 23b49a6 linter: Use `cow_to_ascii_lowercase` instead `cow_to_lowercase` (#8678) (Boshen)
- b8d9a51 span: Deal only in owned `Atom`s (#8641) (overlookmotel)

## [0.47.0] - 2025-01-18

- 7066d1c ast, span, syntax, regular_expression: [**BREAKING**] Remove `ContentHash` (#8512) (overlookmotel)

### Features

- c479a58 napi/parser: Expose dynamic import expressions (#8540) (Boshen)

## [0.46.0] - 2025-01-14

- 7eb6ccd ast: [**BREAKING**] Remove unused and not useful `ContentHash` (#8483) (Boshen)

### Features


## [0.45.0] - 2025-01-11

### Features

- 41ddf60 minfier: Add `CompressOptions::target` (#8179) (Boshen)
- ec88c68 minifier: Compress `a || (a = b)` to `a ||= b` (#8315) (sapphi-red)
- e6fe84d minifier: Compress `a = a + b` to `a += b` (#8314) (sapphi-red)
- 7f19211 minifier: Minimize unary expression statements (#8256) (camc314)
- 2331ea8 minifier: `typeof foo === 'number'` => `typeof foo == 'number'` (#8112) (Boshen)

### Bug Fixes

- c0a3dda minifier: `instanceof` has error throwing side effect (#8378) (Boshen)

## [0.44.0] - 2024-12-25

### Features

- be4feb4 syntax: Add `SymbolId::new` method (#8041) (overlookmotel)

## [0.43.0] - 2024-12-21

### Refactor

- e7476a1 semantic: Remove `serialize` (#8015) (Boshen)

## [0.42.0] - 2024-12-18

- 84b75a0 semantic: [**BREAKING**] Remove `ScopeFlags::Modifiers` (#7935) (overlookmotel)

### Refactor

- 3858221 global: Sort imports (#7883) (overlookmotel)
- 8cf9766 semantic, syntax, wasm: Remove `#![allow(non_snake_case)]` (#7863) (overlookmotel)

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.41.0] - 2024-12-13

### Refactor

- 0f367e5 semantic: Improve the logic of resolving references to be cleaner (#7829) (Dunqing)

## [0.40.0] - 2024-12-10

### Features

- 00fea92 napi/parser: Expose span positions of `import.meta` (#7677) (Boshen)
- b8dc333 syntax: Add `ExportEntry::is_type` (#7676) (Boshen)

### Refactor

- c6a19aa linter: Remove unused `serde` features (#7738) (Boshen)

## [0.39.0] - 2024-12-04

- c2ced15 parser,linter: [**BREAKING**] Use a different `ModuleRecord` for linter (#7554) (Boshen)

- 8a788b8 parser: [**BREAKING**] Build `ModuleRecord` directly in parser (#7546) (Boshen)

### Features

- 7c62a33 napi/parser: Return esm info (#7602) (Boshen)
- 33e5a49 syntax: Add statement span to `ImportEntry` and `ExportEntry` (#7583) (Boshen)

### Performance

- 4a98230 syntax: Use `FxDashMap` for exported bindings (#7525) (overlookmotel)

### Refactor

- 169b8bf linter, syntax: Introduce type alias `FxDashMap` (#7520) (overlookmotel)
- f0e7acc syntax: Change `ModuleRecord::not_esm` to `has_module_syntax` (#7579) (Boshen)
- 18519de syntax: Remove `ModuleRecord::export_default` (#7578) (Boshen)
- d476660 syntax: Remove `ModuleRecord::exported_bindings_duplicated` because it is a syntax error (#7577) (Boshen)
- 17663f5 syntax: Remove `ModuleRecord::export_default_duplicated` because it is a syntax error (#7576) (Boshen)
- 79014ff syntax: Clean up `ModuleRecord` (#7568) (Boshen)

## [0.38.0] - 2024-11-26

- 27b2268 semantic: [**BREAKING**] Remove `SymbolFlags::Export` (#7414) (Dunqing)

- 6f0fe38 semantic: [**BREAKING**] Correct all `ReferenceFlags::Write` according to the spec (#7388) (Dunqing)

### Bug Fixes


### Refactor

- d7d0735 semantic: Remove `SymbolFlags::TypeLiteral` (#7415) (Dunqing)

## [0.37.0] - 2024-11-21

### Features

- 2534cde syntax: Add `AssignmentOperator::to_logical_operator` and `to_binary_operator` methods (#7350) (overlookmotel)

### Performance

- c335f92 syntax: Reorder operator enum variants (#7351) (overlookmotel)

### Documentation

- 9b9d020 semantic: Document the meaning of `ReferenceFlags::Read` and `Write` (#7368) (Boshen)
- b3d5802 syntax: More comments for `ReferenceFlags` (#7392) (overlookmotel)

### Styling

- b5a2027 syntax: Improve formatting (#7349) (overlookmotel)

## [0.36.0] - 2024-11-09

### Features

- dc0215c ast_tools: Add #[estree(append_to)], remove some custom serialization code (#7149) (ottomated)

## [0.35.0] - 2024-11-04

### Features

- 169fa22 ast_tools: Default enums to rename_all = "camelCase" (#6933) (ottomated)

## [0.34.0] - 2024-10-26

### Features

- 1145341 ast_tools: Output typescript to a separate package (#6755) (ottomated)

### Refactor

- 423d54c rust: Remove the annoying `clippy::wildcard_imports` (#6860) (Boshen)

## [0.33.0] - 2024-10-24

### Refactor

- 85e69a1 ast_tools: Add line breaks to generated code for `ESTree` derive (#6680) (overlookmotel)
- ad8e293 ast_tools: Shorten generated code for `impl Serialize` (#6684) (overlookmotel)
- 9ba2b0e ast_tools: Move `#[allow]` attrs to top of generated files (#6679) (overlookmotel)

## [0.32.0] - 2024-10-19

### Features

- e310e52 parser: Generate `Serialize` impls in ast_tools (#6404) (ottomated)

### Bug Fixes

- 2ce3e5f identifier: Add `ZWSP` to `is_irregular_whitespace` (#6662) (Boshen)
- 1d3d256 transformer: Correctly trim JSX (#6639) (magic-akari)

### Documentation

- 335b7f2 syntax: Enable lint warnings on missing docs, and add a lot of documentation (#6611) (DonIsaac)

### Refactor

- 856cab5 ecmascript: Move ToInt32 from `oxc_syntax` to `oxc_ecmascript` (#6471) (Boshen)

## [0.31.0] - 2024-10-08

### Refactor

- 03bc041 syntax: Remove some unsafe code creating IDs (#6324) (overlookmotel)

## [0.30.3] - 2024-09-27

### Bug Fixes

- 933a743 semantic: Add interfaces and functions to `SymbolFlags::ClassExcludes`  (#6057) (DonIsaac)

## [0.30.2] - 2024-09-27

### Bug Fixes

- e0a8959 minifier: Compute `void number` as `undefined` (#6028) (Boshen)

## [0.30.0] - 2024-09-23

- c96b712 syntax: [**BREAKING**] Remove `SymbolFlags::ArrowFunction` (#5857) (overlookmotel)

### Documentation

- 1ccf290 semantic: Document `AstNode` and `AstNodes` (#5872) (DonIsaac)
- e04841c syntax: Add ModuleRecord documentation (#5818) (DonIsaac)

### Refactor


## [0.29.0] - 2024-09-13

### Bug Fixes

- 042afa9 syntax: Correctly check for valid `RedeclarationId`s (#5759) (overlookmotel)

### Refactor

- cc0408b semantic: S/AstNodeId/NodeId (#5740) (Boshen)

## [0.27.0] - 2024-09-06

### Features

- 90facd3 ast: Add `ContentHash` trait; remove noop `Hash` implementation from `Span` (#5451) (rzvxa)
- 23285f4 ast: Add `ContentEq` trait. (#5427) (rzvxa)

### Performance

- bfabd8f syntax: Further optimize `is_identifier_name` (#5426) (overlookmotel)
- aeda84f syntax: Optimize `is_identifier_name` (#5425) (overlookmotel)

### Refactor

- ccc8a27 ast, ast_tools: Use full method path for generated derives trait calls. (#5462) (rzvxa)
- e4ed41d semantic: Change the reference flag to `ReferenceFlags::Type` if it is used within a `TSTypeQuery` (#5444) (Dunqing)
- b47aca0 syntax: Use `generate_derive` for `CloneIn` in types outside of `oxc_ast` crate. (#5280) (rzvxa)

## [0.25.0] - 2024-08-23

- d262a58 syntax: [**BREAKING**] Rename `ReferenceFlag` to `ReferenceFlags` (#5023) (overlookmotel)

### Refactor


## [0.24.3] - 2024-08-18

### Features

- 48821c0 semantic,syntax: Add SymbolFlags::ArrowFunction (#4946) (DonIsaac)

### Documentation

- 0a01a47 semantic: Improve documentation (#4850) (DonIsaac)

### Refactor

- 48a1c32 syntax: Inline trivial bitflags methods (#4877) (overlookmotel)

## [0.24.0] - 2024-08-08

### Features

- 82e2f6b ast_codegen: Process AST-related `syntax` types. (#4694) (rzvxa)
- 6a36616 syntax: Derive `CloneIn` for the AST-related items. (#4730) (rzvxa)

### Bug Fixes

- 9f8f299 syntax: Prevent creating invalid u32 IDs (#4675) (overlookmotel)

### Refactor

- e24fb5b syntax: Add explicit enum discriminants to AST related types. (#4691) (rzvxa)
- 3f3cb62 syntax, span: Reduce #[cfg_attr] boilerplate in type defs (#4698) (overlookmotel)

## [0.23.1] - 2024-08-06

### Bug Fixes

- 9f8f299 syntax: Prevent creating invalid u32 IDs (#4675) (overlookmotel)

## [0.23.0] - 2024-08-01

### Features

- a558492 codegen: Implement `BinaryExpressionVisitor` (#4548) (Boshen)
- 35654e6 codegen: Align operator precedence with esbuild (#4509) (Boshen)
- b952942 linter: Add eslint/no-unused-vars (⭐ attempt 3.2) (#4445) (DonIsaac)

## [0.22.1] - 2024-07-27

### Bug Fixes

- 1667491 syntax: Correct `is_reserved_keyword_or_global_object`'s incorrect function calling. (#4484) (Ethan Goh)
- 82ba2a0 syntax: Fix unsound use of `NonZeroU32` (#4466) (overlookmotel)

### Performance

- 24beaeb semantic: Give `AstNodeId` a niche (#4469) (overlookmotel)
- 6a9f4db semantic: Reduce storage size for symbol redeclarations (#4463) (overlookmotel)

### Refactor

- c99b3eb syntax: Give `ScopeId` a niche (#4468) (overlookmotel)
- 96fc94f syntax: Use `NonMaxU32` for IDs (#4467) (overlookmotel)

## [0.22.0] - 2024-07-23

### Bug Fixes

- f8565ae transformer/typescript: Unexpectedly removed class binding from ExportNamedDeclaration (#4351) (Dunqing)

## [0.21.0] - 2024-07-18

### Features

- 92ee774 semantic: Add `ScopeFlags::CatchClause` for use in CatchClause (#4205) (Dunqing)

### Bug Fixes

- 95e15b6 semantic: Incorrect resolve references for `ExportSpecifier` (#4320) (Dunqing)
- 1108f2a semantic: Resolve references to the incorrect symbol (#4280) (Dunqing)

### Performance

- 8fad7db semantic: Reduce `AstNodeId` to `u32` (#4264) (overlookmotel)

### Refactor

- fc0b17d syntax: Turn the `AstNodeId::dummy` into a constant field. (#4308) (rzvxa)

## [0.16.3] - 2024-07-02

### Bug Fixes

- d995f94 semantic: Resolve reference incorrectly when a parameter references a parameter that hasn't been defined yet (#4004) (Dunqing)

## [0.16.2] - 2024-06-30

### Performance

- 0c81fbe syntax: Use `NonZeroU32` for `SymbolId` and `ReferenceId` (#3970) (Boshen)

## [0.16.0] - 2024-06-26

### Bug Fixes

- 99a40ce semantic: `export default foo` should have `ExportLocalName::Default(NameSpan)` entry (#3823) (Boshen)

## [0.13.4] - 2024-06-07

### Bug Fixes

- c00598b transformer: JSX set `reference_id` on refs to imports (#3524) (overlookmotel)

## [0.13.1] - 2024-05-22

### Features

- e2dd8ac syntax: Export `is_reserved_keyword` and `is_global_object` method (#3384) (Boshen)

### Bug Fixes

- b4fa27a transformer: Do no add __self when the jsx is inside constructor (#3258) (Dunqing)

## [0.13.0] - 2024-05-14

### Features

- f1ccbd4 syntax: Add `ToInt32` trait for f64 (#3132) (Boshen)
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

