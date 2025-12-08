# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.102.0] - 2025-12-08

### 💥 BREAKING CHANGES

- 083fea9 napi/parser: [**BREAKING**] Represent empty optional fields on JS side as `null` (#16411) (overlookmotel)

### 🐛 Bug Fixes

- 699406a napi/parser: Move `ExportEntry::module_request` field to first (#16412) (overlookmotel)
- 12bd794 napi/parser: Move `ExportEntry::module_request` field to last (#16403) (overlookmotel)

### ⚡ Performance

- 790beeb napi/parser: Do not remove extraneous options on JS side (#16447) (overlookmotel)

## [0.100.0] - 2025-12-01

### 💥 BREAKING CHANGES

- 74cf572 ast: [**BREAKING**] Make `source` field of `TSImportType` a `StringLiteral` (#16114) (copilot-swe-agent)
- 934d873 napi: [**BREAKING**] Drop `armv7-unknown-linux-musleabihf` support (#16105) (Boshen)

### 🚀 Features

- bab4bc8 napi/parser: Add type annotations to parse-raw-worker test (#15998) (camc314)

### 🐛 Bug Fixes

- 9149a26 linter/plugins, napi/parser: Deep freeze visitor keys (#16293) (overlookmotel)
- 9cc20a1 minifier: Avoid merging side effectful expressions to next assignment statement if the side effect may change the left hand side reference (#16165) (sapphi-red)
- 91eb3f2 ast/estree: Convert `TSImportType` `argument` field to `Literal` (#16109) (overlookmotel)

### ⚡ Performance

- 02bdf90 linter/plugins, napi/parser: Reuse arrays in visitor keys (#16294) (overlookmotel)

## [0.99.0] - 2025-11-24

### 💥 BREAKING CHANGES

- cbb27fd ast: [**BREAKING**] Add `TSGlobalDeclaration` type (#15712) (overlookmotel)

### 🐛 Bug Fixes

- 2bd3cb6 apps, editors, napi: Fix `oxlint-disable` comments (#16014) (overlookmotel)

## [0.98.0] - 2025-11-17

### 💥 BREAKING CHANGES

- ea51b0b napi: [**BREAKING**] Standardize function naming with sync suffixes (#15661) (Boshen)

### 🚀 Features

- f5ce55a napi: Export all options using wildcard exports (Boshen)

## [0.97.0] - 2025-11-11

### 🚀 Features

- 682dca2 parser: Add more helps to parser errors (#15186) (sapphi-red)

### 🐛 Bug Fixes

- 7f079ab ast/estree: Fix raw transfer deserializer for `AssignmentTargetPropertyIdentifier` (#15304) (overlookmotel)

### ⚡ Performance

- c82fab0 ast/estree: Remove pointless assignments from raw transfer deserializers (#15305) (overlookmotel)

## [0.96.0] - 2025-10-30

### 🚀 Features

- bec7a7d semantic: Add scope to `TSConstructorType` (#14676) (camc314)

### 🐛 Bug Fixes

- 597340e ast-tools: Use oxfmt to format generated code (#15064) (camc314)
- 84b2605 linter/plugins: Remove `parent` property from comments (#14624) (overlookmotel)

### 🚜 Refactor

- 14de671 linter/plugins: Simplify `comments` getter (#14728) (overlookmotel)
- 85a2743 linter/plugins, napi/parser: Remove extraneous code from raw transfer deserializers (#14683) (overlookmotel)
- 2b14abc napi/parser: Shorten raw transfer deserializer for `Comment` (#14623) (overlookmotel)


## [0.95.0] - 2025-10-15

### 🚀 Features

- 0ec0847 ci: Run napi tests on windows (#14383) (camc314)
- b3c5132 parser: Use typescript for raw transfer tests (#14390) (camc314)

### 🚜 Refactor

- 4f301de napi/parser, linter/plugins: Improve formatting of generated code (#14554) (overlookmotel)
- 68c0252 napi/parser, linter/plugins: Shorten generated raw transfer deserializer code (#14553) (overlookmotel)
- f6d890a napi/parser: Re-run `ast_tools` codegen (#14547) (overlookmotel)
- 52f35c6 napi/parser, linter/plugins: Rename `types.js` to `type_ids.js` (#14384) (overlookmotel)

### 🧪 Testing

- f293e3e napi/parser: Disable raw transfer tests on `antd.js` fixture (#14446) (overlookmotel)
- 994c099 napi/parser: Skip slow test (#14424) (overlookmotel)
- 0b076b4 napi/parser: Fix and clarify exclude tests logic (#14409) (overlookmotel)
- 56274a5 napi/parser: Increase timeout on slow test (#14391) (overlookmotel)


## [0.94.0] - 2025-10-06

### 🚀 Features

- a2914fe linter/plugins: Add `loc` field getter to all AST nodes (#14355) (overlookmotel)
- c8de6fe linter/plugins: Add `parent` field to AST nodes (#14345) (overlookmotel)
- 7e4d04f napi/parser: Add option to add `parent` prop to AST nodes with raw transfer (#14344) (overlookmotel)
- 6374065 napi/parser: Raw transfer support `range` field (#14319) (overlookmotel)

### 🐛 Bug Fixes

- a11bc9f napi/parser, linter/plugins: Add `range` field to `TemplateElement` (#14339) (overlookmotel)
- c65e782 napi/parser: Fix JSDoc comments (#14318) (overlookmotel)

### 🚜 Refactor

- 1489376 napi/parser, linter/plugins: Minify walker code (#14376) (overlookmotel)
- c8eeeb5 linter/plugins: Remove build-time dependency on `napi/parser` (#14374) (overlookmotel)
- 073167a napi/parser: Simplify raw transfer deserializer codegen (#14313) (overlookmotel)
- 34e1c0b napi/parser: Use minifier to generate JS/TS raw transfer deserializers from single source (#14312) (overlookmotel)
- a98757a napi/parser: Minify syntax in raw transfer deserializers (#14308) (overlookmotel)

### ⚡ Performance

- e75d42d napi/parser, linter/plugins: Remove runtime `preserveParens` option from raw transfer deserializers (#14338) (overlookmotel)


## [0.94.0] - 2025-10-06

### 🚀 Features

- a2914fe linter/plugins: Add `loc` field getter to all AST nodes (#14355) (overlookmotel)
- c8de6fe linter/plugins: Add `parent` field to AST nodes (#14345) (overlookmotel)
- 7e4d04f napi/parser: Add option to add `parent` prop to AST nodes with raw transfer (#14344) (overlookmotel)
- 6374065 napi/parser: Raw transfer support `range` field (#14319) (overlookmotel)

### 🐛 Bug Fixes

- a11bc9f napi/parser, linter/plugins: Add `range` field to `TemplateElement` (#14339) (overlookmotel)
- c65e782 napi/parser: Fix JSDoc comments (#14318) (overlookmotel)

### 🚜 Refactor

- 1489376 napi/parser, linter/plugins: Minify walker code (#14376) (overlookmotel)
- c8eeeb5 linter/plugins: Remove build-time dependency on `napi/parser` (#14374) (overlookmotel)
- 073167a napi/parser: Simplify raw transfer deserializer codegen (#14313) (overlookmotel)
- 34e1c0b napi/parser: Use minifier to generate JS/TS raw transfer deserializers from single source (#14312) (overlookmotel)
- a98757a napi/parser: Minify syntax in raw transfer deserializers (#14308) (overlookmotel)

### ⚡ Performance

- e75d42d napi/parser, linter/plugins: Remove runtime `preserveParens` option from raw transfer deserializers (#14338) (overlookmotel)


## [0.93.0] - 2025-09-28

### 🚜 Refactor

- c0c0c17 napi/parser: Remove unnecessary `#[estree(field_order)]` attr (#14144) (overlookmotel)

### 🧪 Testing

- 8c09592 napi/parser: Align fixture version with other tasks (#14146) (overlookmotel)


## [0.93.0] - 2025-09-28

### 🚜 Refactor

- c0c0c17 napi/parser: Remove unnecessary `#[estree(field_order)]` attr (#14144) (overlookmotel)

### 🧪 Testing

- 8c09592 napi/parser: Align fixture version with other tasks (#14146) (overlookmotel)


## [0.92.0] - 2025-09-24

### 💥 BREAKING CHANGES

- b1da5d5 napi/parser: [**BREAKING**] Change `oxc-parser` to ESM (#14042) (overlookmotel)

### 🐛 Bug Fixes

- 1f4be38 napi/parser: Generate `.d.mts` extension for types (#14038) (Daniel Roe)

### 🚜 Refactor

- cc0019f napi: Move scripts into `scripts` directory (#14048) (overlookmotel)
- bb040bc parser, linter: Replace `.mjs` files with `.js` (#14045) (overlookmotel)
- e0be08b napi/parser: Freeze `visitorKeys` export (#14012) (overlookmotel)

### 📚 Documentation

- 909a086 parser: Simplify example (#14044) (overlookmotel)

### 🧪 Testing

- 55cd2f3 parser: Remove unused import (#14043) (overlookmotel)


## [0.92.0] - 2025-09-24

### 💥 BREAKING CHANGES

- b1da5d5 napi/parser: [**BREAKING**] Change `oxc-parser` to ESM (#14042) (overlookmotel)

### 🐛 Bug Fixes

- 1f4be38 napi/parser: Generate `.d.mts` extension for types (#14038) (Daniel Roe)

### 🚜 Refactor

- cc0019f napi: Move scripts into `scripts` directory (#14048) (overlookmotel)
- bb040bc parser, linter: Replace `.mjs` files with `.js` (#14045) (overlookmotel)
- e0be08b napi/parser: Freeze `visitorKeys` export (#14012) (overlookmotel)

### 📚 Documentation

- 909a086 parser: Simplify example (#14044) (overlookmotel)

### 🧪 Testing

- 55cd2f3 parser: Remove unused import (#14043) (overlookmotel)


## [0.91.0] - 2025-09-22

### 🐛 Bug Fixes

- 0185984 napi/parser: Raw transfer `preserveParens` option apply to `TSParenthesizedType` (#13964) (overlookmotel)
- bd2fa90 napi/parser: Move `browser.js` file (#13918) (overlookmotel)
- 5ff91bc napi/parser: Fix example (#13887) (overlookmotel)
- 21bbf95 napi: Rebuild bindings file for NAPI packages (#13889) (overlookmotel)

### 🚜 Refactor

- 48062d0 napi/parser: Itemize files in `package.json` (#13978) (overlookmotel)
- 329b7e7 napi/parser: Move scripts to `scripts` directory (#13900) (overlookmotel)
- ac3e9e9 napi/parser: Move JS code into `src-js` directory (#13899) (overlookmotel)
- 2cef8ec napi/parser: Remove outdated comment (#13886) (overlookmotel)

### 📚 Documentation

- 7a50ad9 napi/parser, napi/transform: Correct README examples (#13934) (overlookmotel)

### 💼 Other

- fb347da crates: V0.91.0 (#13961) (Boshen)


## [0.91.0] - 2025-09-22

### 🐛 Bug Fixes

- 0185984 napi/parser: Raw transfer `preserveParens` option apply to `TSParenthesizedType` (#13964) (overlookmotel)
- bd2fa90 napi/parser: Move `browser.js` file (#13918) (overlookmotel)
- 5ff91bc napi/parser: Fix example (#13887) (overlookmotel)
- 21bbf95 napi: Rebuild bindings file for NAPI packages (#13889) (overlookmotel)

### 🚜 Refactor

- 48062d0 napi/parser: Itemize files in `package.json` (#13978) (overlookmotel)
- 329b7e7 napi/parser: Move scripts to `scripts` directory (#13900) (overlookmotel)
- ac3e9e9 napi/parser: Move JS code into `src-js` directory (#13899) (overlookmotel)
- 2cef8ec napi/parser: Remove outdated comment (#13886) (overlookmotel)

### 📚 Documentation

- 7a50ad9 napi/parser, napi/transform: Correct README examples (#13934) (overlookmotel)

### 💼 Other

- fb347da crates: V0.91.0 (#13961) (Boshen)


## [0.91.0] - 2025-09-21

### 🐛 Bug Fixes

- bd2fa90 napi/parser: Move `browser.js` file (#13918) (overlookmotel)
- 5ff91bc napi/parser: Fix example (#13887) (overlookmotel)
- 21bbf95 napi: Rebuild bindings file for NAPI packages (#13889) (overlookmotel)

### 🚜 Refactor

- 329b7e7 napi/parser: Move scripts to `scripts` directory (#13900) (overlookmotel)
- ac3e9e9 napi/parser: Move JS code into `src-js` directory (#13899) (overlookmotel)
- 2cef8ec napi/parser: Remove outdated comment (#13886) (overlookmotel)

### 📚 Documentation

- 7a50ad9 napi/parser, napi/transform: Correct README examples (#13934) (overlookmotel)


## [0.91.0] - 2025-09-21

### 🐛 Bug Fixes

- bd2fa90 napi/parser: Move `browser.js` file (#13918) (overlookmotel)
- 5ff91bc napi/parser: Fix example (#13887) (overlookmotel)
- 21bbf95 napi: Rebuild bindings file for NAPI packages (#13889) (overlookmotel)

### 🚜 Refactor

- 329b7e7 napi/parser: Move scripts to `scripts` directory (#13900) (overlookmotel)
- ac3e9e9 napi/parser: Move JS code into `src-js` directory (#13899) (overlookmotel)
- 2cef8ec napi/parser: Remove outdated comment (#13886) (overlookmotel)

### 📚 Documentation

- 7a50ad9 napi/parser, napi/transform: Correct README examples (#13934) (overlookmotel)


## [0.90.0] - 2025-09-18

### 🚀 Features

- b52389a node: Bump `engines` field to require Node.js 20.19.0+ for ESM support (#13879) (Copilot)
- 24a21b3 napi/parser: `preserveParens` option for raw transfer (#13855) (overlookmotel)

### 🐛 Bug Fixes

- 9796ec1 napi: Fix binding files (Boshen)

### 🚜 Refactor

- 42b1000 napi/parser: Export functions directly (#13854) (overlookmotel)


## [0.90.0] - 2025-09-18

### 🚀 Features

- b52389a node: Bump `engines` field to require Node.js 20.19.0+ for ESM support (#13879) (Copilot)
- 24a21b3 napi/parser: `preserveParens` option for raw transfer (#13855) (overlookmotel)

### 🐛 Bug Fixes

- 9796ec1 napi: Fix binding files (Boshen)

### 🚜 Refactor

- 42b1000 napi/parser: Export functions directly (#13854) (overlookmotel)


## [0.89.0] - 2025-09-15

### 🐛 Bug Fixes

- 341791e parser: Invalid `main` entrypoint in `package.json` (#13767) (Ari Perkkiö)


## [0.89.0] - 2025-09-15

### 🐛 Bug Fixes

- 341791e parser: Invalid `main` entrypoint in `package.json` (#13767) (Ari Perkkiö)


## [0.88.0] - 2025-09-15

### 💥 BREAKING CHANGES

- 4577b71 napi/parser: [**BREAKING**] Change `oxc-parser` to ESM (#13432) (Boshen)

### 🐛 Bug Fixes

- ab51394 raw_transfer: Disable layout assertions on some 32-bit platforms (#13716) (overlookmotel)

### 🚜 Refactor

- babbaca all: Remove `pub` from modules with no exports (#13618) (overlookmotel)


## [0.88.0] - 2025-09-15

### 💥 BREAKING CHANGES

- 4577b71 napi/parser: [**BREAKING**] Change `oxc-parser` to ESM (#13432) (Boshen)

### 🐛 Bug Fixes

- ab51394 raw_transfer: Disable layout assertions on some 32-bit platforms (#13716) (overlookmotel)

### 🚜 Refactor

- babbaca all: Remove `pub` from modules with no exports (#13618) (overlookmotel)












## [0.82.3] - 2025-08-20

### 🚜 Refactor

- 5baa4f1 napi/parser: Fix lint errors (#13185) (overlookmotel)


## [0.82.3] - 2025-08-20

### 🚜 Refactor

- 5baa4f1 napi/parser: Fix lint errors (#13185) (overlookmotel)


## [0.82.2] - 2025-08-17

### 🚜 Refactor

- f6ac2e6 allocator, napi/parser: Use `usize::is_multiple_of` (#13142) (overlookmotel)

### ⚡ Performance

- ecc9c60 napi/parser: Raw transfer: reduce maths complexity (#13145) (overlookmotel)


## [0.82.2] - 2025-08-17

### 🚜 Refactor

- f6ac2e6 allocator, napi/parser: Use `usize::is_multiple_of` (#13142) (overlookmotel)

### ⚡ Performance

- ecc9c60 napi/parser: Raw transfer: reduce maths complexity (#13145) (overlookmotel)




## [0.82.0] - 2025-08-12

### 🚜 Refactor

- bf9ae3b ast/estree: Remove custom serializer for `TSImportTypeQualifier` (#12961) (overlookmotel)


## [0.82.0] - 2025-08-12

### 🚜 Refactor

- bf9ae3b ast/estree: Remove custom serializer for `TSImportTypeQualifier` (#12961) (overlookmotel)


## [0.81.0] - 2025-08-06

### 💥 BREAKING CHANGES

- 2cc1001 ast: [**BREAKING**] Remove `ExportDefaultDeclaration` `exported` field (#12808) (overlookmotel)
- 50b91ac ast: [**BREAKING**] Remove `IdentifierReference` from `qualifier` field of `TSImportType` (#12799) (camc314)

### 🐛 Bug Fixes

- 47e2c59 estree: Fix serialization of `TSImportTypeQualifier` (#12801) (camc314)


## [0.81.0] - 2025-08-06

### 💥 BREAKING CHANGES

- 2cc1001 ast: [**BREAKING**] Remove `ExportDefaultDeclaration` `exported` field (#12808) (overlookmotel)
- 50b91ac ast: [**BREAKING**] Remove `IdentifierReference` from `qualifier` field of `TSImportType` (#12799) (camc314)

### 🐛 Bug Fixes

- 47e2c59 estree: Fix serialization of `TSImportTypeQualifier` (#12801) (camc314)


## [0.80.0] - 2025-08-03

### 💥 BREAKING CHANGES

- cd93174 ast: [**BREAKING**] Introduce `WithClauseKeyword` (#12741) (overlookmotel)
- 7332ae4 ast: [**BREAKING**] Box `rest` fields of `ArrayAssignmentTarget` and `ObjectAssignmentTarget` (#12698) (Copilot)

### 🧪 Testing

- 0ec214b napi: Compile tests in debug mode (#12750) (overlookmotel)
- 2f255a0 napi/parser: Ensure `target` dir exists (#12751) (overlookmotel)
- 02504b9 napi/parser: Disable raw transfer tests by default (#12742) (overlookmotel)


## [0.80.0] - 2025-08-03

### 💥 BREAKING CHANGES

- cd93174 ast: [**BREAKING**] Introduce `WithClauseKeyword` (#12741) (overlookmotel)
- 7332ae4 ast: [**BREAKING**] Box `rest` fields of `ArrayAssignmentTarget` and `ObjectAssignmentTarget` (#12698) (Copilot)

### 🧪 Testing

- 0ec214b napi: Compile tests in debug mode (#12750) (overlookmotel)
- 2f255a0 napi/parser: Ensure `target` dir exists (#12751) (overlookmotel)
- 02504b9 napi/parser: Disable raw transfer tests by default (#12742) (overlookmotel)




## [0.79.0] - 2025-07-30

### 🚜 Refactor

- 8717807 napi/oxlint: Make `types.js` importable (#12581) (overlookmotel)
- f0b1f0d napi/oxlint, napi/parser: Remove source length from `RawTransferMetadata` (#12483) (overlookmotel)

### ⚡ Performance

- 69f8b63 napi/parser, napi/oxlint: Lazy visit: faster check for exit visitor (#12496) (overlookmotel)


## [0.79.0] - 2025-07-30

### 🚜 Refactor

- 8717807 napi/oxlint: Make `types.js` importable (#12581) (overlookmotel)
- f0b1f0d napi/oxlint, napi/parser: Remove source length from `RawTransferMetadata` (#12483) (overlookmotel)

### ⚡ Performance

- 69f8b63 napi/parser, napi/oxlint: Lazy visit: faster check for exit visitor (#12496) (overlookmotel)




## [0.77.3] - 2025-07-20

### 🚀 Features

- bc0fbe5 allocator: `AllocatorPool` store IDs in `Allocator`s (#12310) (overlookmotel)

### 🚜 Refactor

- c5dff1e linter, napi/parser: Add `source_len` field to `RawTransferMetadata` (#12383) (overlookmotel)
- 5e3b415 linter: Duplicate `RawTransferMetadata` in `oxc_linter` crate (#12382) (overlookmotel)
- 319fc3b allocator/fixed-size: Store `alloc_ptr` in the memory block backing the allocator (#12380) (overlookmotel)
- 8fe1aec ast_tools, allocator, napi/parser: Rename vars (#12379) (overlookmotel)


## [0.77.3] - 2025-07-20

### 🚀 Features

- bc0fbe5 allocator: `AllocatorPool` store IDs in `Allocator`s (#12310) (overlookmotel)

### 🚜 Refactor

- c5dff1e linter, napi/parser: Add `source_len` field to `RawTransferMetadata` (#12383) (overlookmotel)
- 5e3b415 linter: Duplicate `RawTransferMetadata` in `oxc_linter` crate (#12382) (overlookmotel)
- 319fc3b allocator/fixed-size: Store `alloc_ptr` in the memory block backing the allocator (#12380) (overlookmotel)
- 8fe1aec ast_tools, allocator, napi/parser: Rename vars (#12379) (overlookmotel)


## [0.77.2] - 2025-07-17

### 🚜 Refactor

- 4517624 napi/parser: Use `sourceByteLen` for UTF8 source length (#12365) (overlookmotel)


## [0.77.2] - 2025-07-17

### 🚜 Refactor

- 4517624 napi/parser: Use `sourceByteLen` for UTF8 source length (#12365) (overlookmotel)


## [0.77.1] - 2025-07-16

### 🚀 Features

- 9b14fbc ast: Add `ThisExpression` to `TSTypeName` (#12156) (Boshen)

### 🚜 Refactor

- a2da682 napi/parser: Clarify pointer maths (#12300) (overlookmotel)
- 2f9bd11 allocator: Fixed size allocator leave space for metadata after arena (#12278) (overlookmotel)
- 5fba91c napi/parser: Raw transfer: introduce metadata struct (#12269) (overlookmotel)
- 39ef911 napi/parser, allocator: Raw transfer: store buffer size and align as consts (#12275) (overlookmotel)
- d009bdb napi/parser: Raw transfer: store offsets as consts (#12268) (overlookmotel)
- 43f61ed napi/parser: Correct comment about raw transfer buffer size (#12273) (overlookmotel)
- 28ed99b napi/parser: Do not compile raw transfer code on WASM (#12271) (overlookmotel)

### ⚡ Performance

- cc1e9fc napi/parser: Raw transfer: reduce size of buffer by 16 bytes (#12277) (overlookmotel)
- 28be5de napi/parser: Raw transfer: move check for supported platform (#12274) (overlookmotel)


## [0.77.1] - 2025-07-16

### 🚀 Features

- 9b14fbc ast: Add `ThisExpression` to `TSTypeName` (#12156) (Boshen)

### 🚜 Refactor

- a2da682 napi/parser: Clarify pointer maths (#12300) (overlookmotel)
- 2f9bd11 allocator: Fixed size allocator leave space for metadata after arena (#12278) (overlookmotel)
- 5fba91c napi/parser: Raw transfer: introduce metadata struct (#12269) (overlookmotel)
- 39ef911 napi/parser, allocator: Raw transfer: store buffer size and align as consts (#12275) (overlookmotel)
- d009bdb napi/parser: Raw transfer: store offsets as consts (#12268) (overlookmotel)
- 43f61ed napi/parser: Correct comment about raw transfer buffer size (#12273) (overlookmotel)
- 28ed99b napi/parser: Do not compile raw transfer code on WASM (#12271) (overlookmotel)

### ⚡ Performance

- cc1e9fc napi/parser: Raw transfer: reduce size of buffer by 16 bytes (#12277) (overlookmotel)
- 28be5de napi/parser: Raw transfer: move check for supported platform (#12274) (overlookmotel)


## [0.77.0] - 2025-07-12

### 🚀 Features

- 407429a napi/parser,napi/transform: Accept `lang=dts` (#12154) (Boshen)

### 🚜 Refactor

- baa3726 tests/napi: Add `build-test` script for tests (#12132) (camc314)

### ⚡ Performance

- 4c35f4a napi/parser: Optimize raw transfer deserializer for `TSClassImplements` (#12158) (overlookmotel)


## [0.77.0] - 2025-07-12

### 🚀 Features

- 407429a napi/parser,napi/transform: Accept `lang=dts` (#12154) (Boshen)

### 🚜 Refactor

- baa3726 tests/napi: Add `build-test` script for tests (#12132) (camc314)

### ⚡ Performance

- 4c35f4a napi/parser: Optimize raw transfer deserializer for `TSClassImplements` (#12158) (overlookmotel)


## [0.76.0] - 2025-07-08

### 🐛 Bug Fixes

- a490e00 napi/parser: Lazy visit: correct error messages (#12109) (overlookmotel)

### 🚜 Refactor

- c5e8d90 napi/parser: Lazy deser: `constructors.js` export static object (#12090) (overlookmotel)
- fe35285 napi/parser: Lazy deser: remove `construct` function (#12089) (overlookmotel)
- 9ae0815 napi/parser: Move files (#12088) (overlookmotel)


## [0.76.0] - 2025-07-08

### 🐛 Bug Fixes

- a490e00 napi/parser: Lazy visit: correct error messages (#12109) (overlookmotel)

### 🚜 Refactor

- c5e8d90 napi/parser: Lazy deser: `constructors.js` export static object (#12090) (overlookmotel)
- fe35285 napi/parser: Lazy deser: remove `construct` function (#12089) (overlookmotel)
- 9ae0815 napi/parser: Move files (#12088) (overlookmotel)


## [0.75.1] - 2025-07-03

### 🐛 Bug Fixes

- a3641d2 napi/parser: Remove non-existent methods from TS type defs (#12054) (overlookmotel)

### 🚜 Refactor

- 016634f ast/estree: Introduce `serialize_span` method (#12013) (overlookmotel)
- 754f01d ast/estree: Move `start` and `end` fields to last (#12012) (overlookmotel)
- 4597311 ast/estree: Remove temp vars for `ranges` from serializer (#12007) (overlookmotel)

### 📚 Documentation

- 4a408c3 napi/parser: Document options (#12008) (overlookmotel)


## [0.75.1] - 2025-07-03

### 🐛 Bug Fixes

- a3641d2 napi/parser: Remove non-existent methods from TS type defs (#12054) (overlookmotel)

### 🚜 Refactor

- 016634f ast/estree: Introduce `serialize_span` method (#12013) (overlookmotel)
- 754f01d ast/estree: Move `start` and `end` fields to last (#12012) (overlookmotel)
- 4597311 ast/estree: Remove temp vars for `ranges` from serializer (#12007) (overlookmotel)

### 📚 Documentation

- 4a408c3 napi/parser: Document options (#12008) (overlookmotel)


## [0.75.0] - 2025-06-25

### 💥 BREAKING CHANGES

- 9a2548a napi/parser: [**BREAKING**] Add `range` option (#11728) (Bacary Bruno Bodian)

### 🐛 Bug Fixes

- cf0e18a napi/parser: `NodeArray` allow setting large integer properties (#11883) (overlookmotel)

### 🚜 Refactor

- 0bf7815 napi/parser: Lazy visitor: pre-calculate count of node types (#11861) (overlookmotel)

### ⚡ Performance

- 84fa006 napi/parser: Lazy deser: faster construction of `NodeArray` iterators (#11870) (overlookmotel)
- fb02e6c napi/parser: Lazy deser: speed up creating `NodeArray`s (#11869) (overlookmotel)
- 58dfff8 napi/parser: Raw deser: remove `WeakMap` from `NodeArray` (#11868) (overlookmotel)
- 6c5ee78 napi/parser: Lazy visit: cheaper for loop (#11864) (overlookmotel)

### 🧪 Testing

- 54f9464 napi/parser: Add benchmarks for lazy visit alone (#11866) (overlookmotel)
- 97b671f napi/parser: Load internal modules with `require` in benchmarks (#11865) (overlookmotel)


## [0.75.0] - 2025-06-25

### 💥 BREAKING CHANGES

- 9a2548a napi/parser: [**BREAKING**] Add `range` option (#11728) (Bacary Bruno Bodian)

### 🐛 Bug Fixes

- cf0e18a napi/parser: `NodeArray` allow setting large integer properties (#11883) (overlookmotel)

### 🚜 Refactor

- 0bf7815 napi/parser: Lazy visitor: pre-calculate count of node types (#11861) (overlookmotel)

### ⚡ Performance

- 84fa006 napi/parser: Lazy deser: faster construction of `NodeArray` iterators (#11870) (overlookmotel)
- fb02e6c napi/parser: Lazy deser: speed up creating `NodeArray`s (#11869) (overlookmotel)
- 58dfff8 napi/parser: Raw deser: remove `WeakMap` from `NodeArray` (#11868) (overlookmotel)
- 6c5ee78 napi/parser: Lazy visit: cheaper for loop (#11864) (overlookmotel)

### 🧪 Testing

- 54f9464 napi/parser: Add benchmarks for lazy visit alone (#11866) (overlookmotel)
- 97b671f napi/parser: Load internal modules with `require` in benchmarks (#11865) (overlookmotel)


## [0.74.0] - 2025-06-23

### 🚀 Features

- 93069a5 napi/parser: Add experimental lazy visitor (#11837) (overlookmotel)

### 🚜 Refactor

- 0260308 ast_tools: Prepare lazy deserializer codegen for visitor (#11836) (overlookmotel)
- b544be8 napi/parser: Remove options amendment from `prepareRaw` (#11828) (overlookmotel)
- 9c960cd napi/parser: Re-order code (#11813) (overlookmotel)
- bfed7f2 napi/parser: Rename file (#11808) (overlookmotel)
- 08e666f ast/estree: Add `#[estree]` attrs to `RegExpFlagsAlias` (#11794) (overlookmotel)

### 📚 Documentation

- 4dc8a4e napi/parser: Add JSDoc comments to all functions (#11814) (overlookmotel)

### ⚡ Performance

- 6bbe048 napi/parser: Do not lazily create `TextEncoder` (#11817) (overlookmotel)
- aef1770 napi/parser: Destructure `bindings` on import (#11811) (overlookmotel)
- 3a0a673 napi/parser: Lazy-load raw transfer and lazy deser code (#11807) (overlookmotel)


## [0.74.0] - 2025-06-23

### 🚀 Features

- 93069a5 napi/parser: Add experimental lazy visitor (#11837) (overlookmotel)

### 🚜 Refactor

- 0260308 ast_tools: Prepare lazy deserializer codegen for visitor (#11836) (overlookmotel)
- b544be8 napi/parser: Remove options amendment from `prepareRaw` (#11828) (overlookmotel)
- 9c960cd napi/parser: Re-order code (#11813) (overlookmotel)
- bfed7f2 napi/parser: Rename file (#11808) (overlookmotel)
- 08e666f ast/estree: Add `#[estree]` attrs to `RegExpFlagsAlias` (#11794) (overlookmotel)

### 📚 Documentation

- 4dc8a4e napi/parser: Add JSDoc comments to all functions (#11814) (overlookmotel)

### ⚡ Performance

- 6bbe048 napi/parser: Do not lazily create `TextEncoder` (#11817) (overlookmotel)
- aef1770 napi/parser: Destructure `bindings` on import (#11811) (overlookmotel)
- 3a0a673 napi/parser: Lazy-load raw transfer and lazy deser code (#11807) (overlookmotel)


## [0.73.2] - 2025-06-18

### 🐛 Bug Fixes

- a47a6de napi/parser: Lazy deser: do not expose `getElement` method from `NodeArray` (#11777) (overlookmotel)

### ⚡ Performance

- 21c8852 napi/parser: Faster deserialization of `Vec`s in raw transfer (#11776) (overlookmotel)


## [0.73.2] - 2025-06-18

### 🐛 Bug Fixes

- a47a6de napi/parser: Lazy deser: do not expose `getElement` method from `NodeArray` (#11777) (overlookmotel)

### ⚡ Performance

- 21c8852 napi/parser: Faster deserialization of `Vec`s in raw transfer (#11776) (overlookmotel)


## [0.73.1] - 2025-06-17

### 🚀 Features

- 81ef443 napi: Add `aarch64-linux-android` target (#11769) (LongYinan)
- dfdebc2 napi/parser: Lazy deserializer `NodeArray` `slice` method (#11680) (overlookmotel)

### 🐛 Bug Fixes

- 6feab7e ast/estree: Remove custom serializer for `MethodDefinition` `key` field (#11763) (overlookmotel)
- fcb3084 napi/parser: Lazy deser: remove outdated comments (#11699) (overlookmotel)
- 2749931 napi/parser: Lazy deserializer block class constructors correctly (#11679) (overlookmotel)
- e523d86 napi/parser: Lazy deserializer locally cache all `Vec`s and strings (#11667) (overlookmotel)

### 🚜 Refactor

- d057652 regular-expression: Shorten Span construction (#11689) (Ulrich Stark)
- f1f3c30 napi/parser: Lazy deserializer: prefix local cache property keys with `$` (#11673) (overlookmotel)

### ⚡ Performance

- d136acd napi/parser: Lazy deser: remove `getInternal` function in `NodeArray` (#11698) (overlookmotel)
- a6a82f9 napi/parser: Lazy deser: avoid changing shape of `NodeArray` prototype (#11697) (overlookmotel)
- 60f754e napi/parser: Lazily deserialize `Vec`s (#11678) (overlookmotel)

### 🧪 Testing

- 20efcd4 napi/parser: Remove unnecessary `RUN_SIMPLE_LAZY_TESTS` env var (#11703) (overlookmotel)
- 6848b24 napi/parser: Lazy deser: tests for introspection of `NodeArray`s (#11702) (overlookmotel)


## [0.73.1] - 2025-06-17

### 🚀 Features

- 81ef443 napi: Add `aarch64-linux-android` target (#11769) (LongYinan)
- dfdebc2 napi/parser: Lazy deserializer `NodeArray` `slice` method (#11680) (overlookmotel)

### 🐛 Bug Fixes

- 6feab7e ast/estree: Remove custom serializer for `MethodDefinition` `key` field (#11763) (overlookmotel)
- fcb3084 napi/parser: Lazy deser: remove outdated comments (#11699) (overlookmotel)
- 2749931 napi/parser: Lazy deserializer block class constructors correctly (#11679) (overlookmotel)
- e523d86 napi/parser: Lazy deserializer locally cache all `Vec`s and strings (#11667) (overlookmotel)

### 🚜 Refactor

- d057652 regular-expression: Shorten Span construction (#11689) (Ulrich Stark)
- f1f3c30 napi/parser: Lazy deserializer: prefix local cache property keys with `$` (#11673) (overlookmotel)

### ⚡ Performance

- d136acd napi/parser: Lazy deser: remove `getInternal` function in `NodeArray` (#11698) (overlookmotel)
- a6a82f9 napi/parser: Lazy deser: avoid changing shape of `NodeArray` prototype (#11697) (overlookmotel)
- 60f754e napi/parser: Lazily deserialize `Vec`s (#11678) (overlookmotel)

### 🧪 Testing

- 20efcd4 napi/parser: Remove unnecessary `RUN_SIMPLE_LAZY_TESTS` env var (#11703) (overlookmotel)
- 6848b24 napi/parser: Lazy deser: tests for introspection of `NodeArray`s (#11702) (overlookmotel)


## [0.73.0] - 2025-06-13

### 💥 BREAKING CHANGES

- f3eaefb ast: [**BREAKING**] Add `value` field to `BigIntLiteral` (#11564) (overlookmotel)

### 🚀 Features

- 5860195 napi/parser: Improved `console.log` output for lazy deserialized AST (#11642) (overlookmotel)
- 5a55a58 napi/parser: Add lazy deserialization (#11595) (overlookmotel)
- 120b00f napi/parser: Support old versions of NodeJS (#11596) (overlookmotel)

### 🐛 Bug Fixes

- 931fc73 napi/parser: Cache nodes in lazy deserialization (#11637) (overlookmotel)

### 🚜 Refactor

- ff7111c napi/parser: Use "construct" instead of "deserialize" in lazy deserializer (#11616) (overlookmotel)
- 95ee174 napi/parser: Move raw transfer code into separate directory (#11583) (overlookmotel)
- 8e74e05 ast/estree: Remove dead code from generated raw transfer deserializer (#11579) (overlookmotel)

### ⚡ Performance

- 5271951 napi/parser: Remove function calls from lazy deserialization (#11615) (overlookmotel)
- 7c66637 napi/parser: Re-use `TypedArray` objects in raw transfer (#11585) (overlookmotel)

### 🧪 Testing

- 8cab72f napi/parser: Env var to run lazy deserialization tests (#11636) (overlookmotel)
- 8ad3061 napi/parser: Run raw transfer tests on multiple threads (#11611) (overlookmotel)
- c0027e0 ast/estree: Benchmark raw transfer deserialization in isolation (#11584) (overlookmotel)


## [0.73.0] - 2025-06-13

### 💥 BREAKING CHANGES

- f3eaefb ast: [**BREAKING**] Add `value` field to `BigIntLiteral` (#11564) (overlookmotel)

### 🚀 Features

- 5860195 napi/parser: Improved `console.log` output for lazy deserialized AST (#11642) (overlookmotel)
- 5a55a58 napi/parser: Add lazy deserialization (#11595) (overlookmotel)
- 120b00f napi/parser: Support old versions of NodeJS (#11596) (overlookmotel)

### 🐛 Bug Fixes

- 931fc73 napi/parser: Cache nodes in lazy deserialization (#11637) (overlookmotel)

### 🚜 Refactor

- ff7111c napi/parser: Use "construct" instead of "deserialize" in lazy deserializer (#11616) (overlookmotel)
- 95ee174 napi/parser: Move raw transfer code into separate directory (#11583) (overlookmotel)
- 8e74e05 ast/estree: Remove dead code from generated raw transfer deserializer (#11579) (overlookmotel)

### ⚡ Performance

- 5271951 napi/parser: Remove function calls from lazy deserialization (#11615) (overlookmotel)
- 7c66637 napi/parser: Re-use `TypedArray` objects in raw transfer (#11585) (overlookmotel)

### 🧪 Testing

- 8cab72f napi/parser: Env var to run lazy deserialization tests (#11636) (overlookmotel)
- 8ad3061 napi/parser: Run raw transfer tests on multiple threads (#11611) (overlookmotel)
- c0027e0 ast/estree: Benchmark raw transfer deserialization in isolation (#11584) (overlookmotel)


# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.72.3] - 2025-06-06

### Bug Fixes

- 8451bee ast/estree: Remove repeat fields from `BindingPattern` in TS-ESTree AST (#11500) (overlookmotel)
- 5c32b7c ast/estree: Make error objects via raw transfer match standard transfer (#11481) (overlookmotel)
- 953e61b ast/estree: Fix field order of `PropertyKey` constructor in raw transfer TS-ESTree AST (#11463) (overlookmotel)
- ec4fc83 estree: Use consistent field order in serialization (#11385) (Yuji Sugiura)
- ab0dd29 napi: Napi build cache problem (#11479) (LongYinan)

### Testing

- 97aa9cc ast/estree: Remove test skip list for TS raw transfer tests (#11478) (overlookmotel)
- 75e241c ast/estree: Test raw transfer on TS-ESTree (#11476) (overlookmotel)

## [0.72.3] - 2025-06-06

### Bug Fixes

- 8451bee ast/estree: Remove repeat fields from `BindingPattern` in TS-ESTree AST (#11500) (overlookmotel)
- 5c32b7c ast/estree: Make error objects via raw transfer match standard transfer (#11481) (overlookmotel)
- 953e61b ast/estree: Fix field order of `PropertyKey` constructor in raw transfer TS-ESTree AST (#11463) (overlookmotel)
- ec4fc83 estree: Use consistent field order in serialization (#11385) (Yuji Sugiura)
- ab0dd29 napi: Napi build cache problem (#11479) (LongYinan)

### Testing

- 97aa9cc ast/estree: Remove test skip list for TS raw transfer tests (#11478) (overlookmotel)
- 75e241c ast/estree: Test raw transfer on TS-ESTree (#11476) (overlookmotel)

## [0.72.2] - 2025-05-31

### Features

- 1df6732 ast/estree: Add decorators to ESTree AST (#11393) (overlookmotel)

### Refactor

- 12690a1 ast/estree: Re-order fields in visitation order (#11362) (overlookmotel)
- 75ee3a5 ast/estree: Do not put TS struct fields last (#11360) (overlookmotel)
- 1d1ebd6 ast_tools/estree: Order `type` and `span` fields first by default (#11361) (overlookmotel)

## [0.72.2] - 2025-05-31

### Features

- 1df6732 ast/estree: Add decorators to ESTree AST (#11393) (overlookmotel)

### Refactor

- 12690a1 ast/estree: Re-order fields in visitation order (#11362) (overlookmotel)
- 75ee3a5 ast/estree: Do not put TS struct fields last (#11360) (overlookmotel)
- 1d1ebd6 ast_tools/estree: Order `type` and `span` fields first by default (#11361) (overlookmotel)

## [0.72.1] - 2025-05-28

### Features

- b8aa4e3 napi/parser: `parseAsync` support raw transfer (#11335) (overlookmotel)

## [0.72.1] - 2025-05-28

### Features

- b8aa4e3 napi/parser: `parseAsync` support raw transfer (#11335) (overlookmotel)

## [0.72.0] - 2025-05-24

### Features

- 23182b8 ast/estree: Add `phase` field to `ImportExpression` in TS-ESTree AST (#11193) (overlookmotel)

## [0.72.0] - 2025-05-24

### Features

- 23182b8 ast/estree: Add `phase` field to `ImportExpression` in TS-ESTree AST (#11193) (overlookmotel)

## [0.71.0] - 2025-05-20

### Features

- c60382d allocator/vec2: Change `len` and `cap` fields from `usize` to `u32` (#10884) (Dunqing)
- d47b305 ast/estree: Add `phase` field to `ImportExpression` in ESTree AST (#11165) (overlookmotel)
- 1bc8d29 ast/estree: Add `phase` field to `ImportDeclaration` in ESTree AST (#11157) (overlookmotel)
- 9e90e00 ast_tools: Introduce `#[js_only]` attr for struct fields and converters (#11160) (overlookmotel)
- d67c9e5 napi: Bump napi to beta (#11159) (Boshen)

### Bug Fixes

- 3795eb6 ci: Use jsdelivr for all benchmark files (#11108) (Boshen)
- 963167d napi: Fix cfg feature on global_allocator (Boshen)

### Performance

- 5dcd0f1 allocator/vec2: Reorder `RawVec` fields (#11050) (Dunqing)
- 2b0a69f ast: Re-order struct fields to reduce padding (#11056) (overlookmotel)
- b9e51e2 ast: Reduce size of `Comment` to 16 bytes (#11062) (camchenry)

### Documentation

- e92bf1f napi: Update docs for `oxc-parser` (#11156) (overlookmotel)

### Refactor

- 9775585 regular_expression: Refactor `regexp-modifiers` support (#11142) (Yuji Sugiura)

## [0.71.0] - 2025-05-20

### Features

- c60382d allocator/vec2: Change `len` and `cap` fields from `usize` to `u32` (#10884) (Dunqing)
- d47b305 ast/estree: Add `phase` field to `ImportExpression` in ESTree AST (#11165) (overlookmotel)
- 1bc8d29 ast/estree: Add `phase` field to `ImportDeclaration` in ESTree AST (#11157) (overlookmotel)
- 9e90e00 ast_tools: Introduce `#[js_only]` attr for struct fields and converters (#11160) (overlookmotel)
- d67c9e5 napi: Bump napi to beta (#11159) (Boshen)

### Bug Fixes

- 3795eb6 ci: Use jsdelivr for all benchmark files (#11108) (Boshen)
- 963167d napi: Fix cfg feature on global_allocator (Boshen)

### Performance

- 5dcd0f1 allocator/vec2: Reorder `RawVec` fields (#11050) (Dunqing)
- 2b0a69f ast: Re-order struct fields to reduce padding (#11056) (overlookmotel)
- b9e51e2 ast: Reduce size of `Comment` to 16 bytes (#11062) (camchenry)

### Documentation

- e92bf1f napi: Update docs for `oxc-parser` (#11156) (overlookmotel)

### Refactor

- 9775585 regular_expression: Refactor `regexp-modifiers` support (#11142) (Yuji Sugiura)

## [0.70.0] - 2025-05-15

### Features

- 647b6f3 napi: Add arm musl (#10958) (Bernd Storath)

### Bug Fixes

- 6f3f9d7 ast/estree: Fix `raw_deser` for `TSMappedTypeOptional` serializer (#10971) (overlookmotel)
- 53329f8 ast/estree: Fix field order for `FormalParameter` (#10962) (overlookmotel)
- 8b8f78f ast/estree: Fix field order and type def for `RestElement` in `FormalParameters` (#10961) (overlookmotel)
- 2b76ab5 ast/estree: Fix `TSModuleDeclaration` raw deserializer (#10924) (overlookmotel)
- d036cf5 estree: Ensure the same key order for `AssignmentPattern` (#10953) (Yuji Sugiura)
- 635aa96 napi: Computed final source type from `lang` then `sourceType` (#11060) (Boshen)
- 584d8b9 napi: Enable mimalloc `no_opt_arch` feature on linux aarch64 (#11053) (Boshen)

### Performance

- a4b5716 ast/estree: Streamline raw deserializer for `WithClause` (#10974) (overlookmotel)

## [0.70.0] - 2025-05-15

### Features

- 647b6f3 napi: Add arm musl (#10958) (Bernd Storath)

### Bug Fixes

- 6f3f9d7 ast/estree: Fix `raw_deser` for `TSMappedTypeOptional` serializer (#10971) (overlookmotel)
- 53329f8 ast/estree: Fix field order for `FormalParameter` (#10962) (overlookmotel)
- 8b8f78f ast/estree: Fix field order and type def for `RestElement` in `FormalParameters` (#10961) (overlookmotel)
- 2b76ab5 ast/estree: Fix `TSModuleDeclaration` raw deserializer (#10924) (overlookmotel)
- d036cf5 estree: Ensure the same key order for `AssignmentPattern` (#10953) (Yuji Sugiura)
- 635aa96 napi: Computed final source type from `lang` then `sourceType` (#11060) (Boshen)
- 584d8b9 napi: Enable mimalloc `no_opt_arch` feature on linux aarch64 (#11053) (Boshen)

### Performance

- a4b5716 ast/estree: Streamline raw deserializer for `WithClause` (#10974) (overlookmotel)

## [0.69.0] - 2025-05-09

- 2b5d826 ast: [**BREAKING**] Fix field order for `TSTypeAssertion` (#10906) (overlookmotel)

- 1f35910 ast: [**BREAKING**] Fix field order for `TSNamedTupleMember` (#10905) (overlookmotel)

- 8a3bba8 ast: [**BREAKING**] Fix field order for `PropertyDefinition` (#10902) (overlookmotel)

- 5746d36 ast: [**BREAKING**] Fix field order for `NewExpression` (#10893) (overlookmotel)

- 0139793 ast: [**BREAKING**] Re-order fields of `TaggedTemplateExpression` (#10889) (overlookmotel)

- 6646b6b ast: [**BREAKING**] Fix field order for `JSXOpeningElement` (#10882) (overlookmotel)

- cc2ed21 ast: [**BREAKING**] Fix field order for `JSXElement` and `JSXFragment` (#10881) (overlookmotel)

- ad4fbf4 ast: [**BREAKING**] Simplify `RegExpPattern` (#10834) (overlookmotel)

### Features

- d066516 ast_tools: Support `#[estree(prepend_to)]` (#10849) (overlookmotel)
- 22ba60b napi: Add `s390x-unknown-linux-gnu` build (#10892) (Boshen)
- 308fe73 napi: Add `x86_64-unknown-freebsd` and `riscv64gc-unknown-linux-gnu` builds (#10886) (Boshen)
- 3cf867c napi/parser: Expose module record data for `export default interface` (#10894) (Boshen)

### Bug Fixes

- 2c09243 ast: Fix field order for `AccessorProperty` (#10878) (overlookmotel)
- e7228fa ast/estree: Fix `optional` field of `TSMappedType` in TS-ESTree AST (#10874) (overlookmotel)
- 6f0638a ast/estree: Remove `TSImportTypeOptions` custom serializer (#10873) (overlookmotel)
- e6657ae ast/estree: Reorder fields for TS `Identifier` types in TS-ESTree AST (#10864) (overlookmotel)
- 87fc903 napi/parser: Expose visitor keys files in NPM package (#10817) (overlookmotel)

### Performance

- 49a6f97 napi/parser: Faster fixup of `BigInt`s and `RegExp`s (#10820) (overlookmotel)
- 0905767 napi/parser: Simplify recursion and avoid function calls in fixup visitor (#10813) (overlookmotel)
- f85bda4 parser: Use visitor instead of JSON.parse reviver (#10791) (Arnaud Barré)

### Refactor

- b16331e ast/estree: Generalize concatenating fields with `Concat2` (#10848) (overlookmotel)
- daba0a7 estree: Remove regular expression types from ESTree AST (#10855) (overlookmotel)

### Styling

- 62c3a4a ast_tools: Add full stop to end of generated comments (#10809) (overlookmotel)

## [0.69.0] - 2025-05-09

- 2b5d826 ast: [**BREAKING**] Fix field order for `TSTypeAssertion` (#10906) (overlookmotel)

- 1f35910 ast: [**BREAKING**] Fix field order for `TSNamedTupleMember` (#10905) (overlookmotel)

- 8a3bba8 ast: [**BREAKING**] Fix field order for `PropertyDefinition` (#10902) (overlookmotel)

- 5746d36 ast: [**BREAKING**] Fix field order for `NewExpression` (#10893) (overlookmotel)

- 0139793 ast: [**BREAKING**] Re-order fields of `TaggedTemplateExpression` (#10889) (overlookmotel)

- 6646b6b ast: [**BREAKING**] Fix field order for `JSXOpeningElement` (#10882) (overlookmotel)

- cc2ed21 ast: [**BREAKING**] Fix field order for `JSXElement` and `JSXFragment` (#10881) (overlookmotel)

- ad4fbf4 ast: [**BREAKING**] Simplify `RegExpPattern` (#10834) (overlookmotel)

### Features

- d066516 ast_tools: Support `#[estree(prepend_to)]` (#10849) (overlookmotel)
- 22ba60b napi: Add `s390x-unknown-linux-gnu` build (#10892) (Boshen)
- 308fe73 napi: Add `x86_64-unknown-freebsd` and `riscv64gc-unknown-linux-gnu` builds (#10886) (Boshen)
- 3cf867c napi/parser: Expose module record data for `export default interface` (#10894) (Boshen)

### Bug Fixes

- 2c09243 ast: Fix field order for `AccessorProperty` (#10878) (overlookmotel)
- e7228fa ast/estree: Fix `optional` field of `TSMappedType` in TS-ESTree AST (#10874) (overlookmotel)
- 6f0638a ast/estree: Remove `TSImportTypeOptions` custom serializer (#10873) (overlookmotel)
- e6657ae ast/estree: Reorder fields for TS `Identifier` types in TS-ESTree AST (#10864) (overlookmotel)
- 87fc903 napi/parser: Expose visitor keys files in NPM package (#10817) (overlookmotel)

### Performance

- 49a6f97 napi/parser: Faster fixup of `BigInt`s and `RegExp`s (#10820) (overlookmotel)
- 0905767 napi/parser: Simplify recursion and avoid function calls in fixup visitor (#10813) (overlookmotel)
- f85bda4 parser: Use visitor instead of JSON.parse reviver (#10791) (Arnaud Barré)

### Refactor

- b16331e ast/estree: Generalize concatenating fields with `Concat2` (#10848) (overlookmotel)
- daba0a7 estree: Remove regular expression types from ESTree AST (#10855) (overlookmotel)

### Styling

- 62c3a4a ast_tools: Add full stop to end of generated comments (#10809) (overlookmotel)

## [0.68.1] - 2025-05-04

### Bug Fixes

- c33eb9c ast/estree: Fix raw deser for `TSTypeReference` (#10787) (overlookmotel)

## [0.68.1] - 2025-05-04

### Bug Fixes

- c33eb9c ast/estree: Fix raw deser for `TSTypeReference` (#10787) (overlookmotel)

## [0.68.0] - 2025-05-03

- 28ceb90 ast: [**BREAKING**] Remove `TSMappedTypeModifierOperator::None` variant (#10749) (overlookmotel)

### Bug Fixes

- 61d825b ast/estree: Rename `assert` to `with` in `TSImportType` `options` in TS-ESTree AST (#10681) (overlookmotel)
- c8005ad ast/estree: Add line comment for hashbang in ESTree AST (#10669) (overlookmotel)

### Performance

- d882eaa napi/parser: Lazy load raw transfer deserializers (#10482) (overlookmotel)

### Refactor

- 050ecd9 ast/estree: Remove custom serializer for `TSMappedTypeModifierOperator` (#10747) (overlookmotel)
- a2ba7c3 napi/parser: Add comments about hashbang comments (#10692) (overlookmotel)
- 3b6d52d napi/parser: Move generated deserializer files (#10481) (overlookmotel)

### Testing

- 14c4bbb ast/estree: Fix raw transfer tests (#10666) (overlookmotel)

## [0.68.0] - 2025-05-03

- 28ceb90 ast: [**BREAKING**] Remove `TSMappedTypeModifierOperator::None` variant (#10749) (overlookmotel)

### Bug Fixes

- 61d825b ast/estree: Rename `assert` to `with` in `TSImportType` `options` in TS-ESTree AST (#10681) (overlookmotel)
- c8005ad ast/estree: Add line comment for hashbang in ESTree AST (#10669) (overlookmotel)

### Performance

- d882eaa napi/parser: Lazy load raw transfer deserializers (#10482) (overlookmotel)

### Refactor

- 050ecd9 ast/estree: Remove custom serializer for `TSMappedTypeModifierOperator` (#10747) (overlookmotel)
- a2ba7c3 napi/parser: Add comments about hashbang comments (#10692) (overlookmotel)
- 3b6d52d napi/parser: Move generated deserializer files (#10481) (overlookmotel)

### Testing

- 14c4bbb ast/estree: Fix raw transfer tests (#10666) (overlookmotel)

## [0.67.0] - 2025-04-27

### Bug Fixes

- 24ab2f3 ast/estree: Convert `TSClassImplements::expression` to `MemberExpression` in TS-ESTree AST (#10607) (overlookmotel)
- 0825834 ast/estree: Correct `this` in `TSTypeName` in TS-ESTree AST (#10603) (overlookmotel)
- d1f5abb ast/estree: Fix TS-ESTree AST for `TSModuleDeclaration` (#10574) (overlookmotel)
- 66e384c ast/estree: Add missing fields to `ObjectPattern` in TS-ESTree AST (#10570) (overlookmotel)
- a9785e3 parser,linter: Consider typescript declarations for named exports (#10532) (Ulrich Stark)

### Refactor

- 936f885 napi/parser: Refactor `wrap` files (#10480) (overlookmotel)

## [0.67.0] - 2025-04-27

### Bug Fixes

- 24ab2f3 ast/estree: Convert `TSClassImplements::expression` to `MemberExpression` in TS-ESTree AST (#10607) (overlookmotel)
- 0825834 ast/estree: Correct `this` in `TSTypeName` in TS-ESTree AST (#10603) (overlookmotel)
- d1f5abb ast/estree: Fix TS-ESTree AST for `TSModuleDeclaration` (#10574) (overlookmotel)
- 66e384c ast/estree: Add missing fields to `ObjectPattern` in TS-ESTree AST (#10570) (overlookmotel)
- a9785e3 parser,linter: Consider typescript declarations for named exports (#10532) (Ulrich Stark)

### Refactor

- 936f885 napi/parser: Refactor `wrap` files (#10480) (overlookmotel)

## [0.66.0] - 2025-04-23

### Bug Fixes

- 43ad4e9 ast: Box `this_param` in `TSCallSignatureDeclaration` (#10558) (Yuji Sugiura)
- 8eb3c0a ast/estree: Fix raw deser for `FormalParameter` (#10548) (overlookmotel)
- f19b287 estree: Add `TSParameterProperty` (#10534) (Yuji Sugiura)

## [0.66.0] - 2025-04-23

### Bug Fixes

- 43ad4e9 ast: Box `this_param` in `TSCallSignatureDeclaration` (#10558) (Yuji Sugiura)
- 8eb3c0a ast/estree: Fix raw deser for `FormalParameter` (#10548) (overlookmotel)
- f19b287 estree: Add `TSParameterProperty` (#10534) (Yuji Sugiura)

## [0.65.0] - 2025-04-21

- 99d82db ast: [**BREAKING**] Move `type_parameters` field to before `extends` in `TSInterfaceDeclaration` (#10476) (overlookmotel)

- 7212803 ast: [**BREAKING**] Change `TSInterfaceDeclaration::extends` from `Option<Vec>` to `Vec` (#10472) (overlookmotel)

### Bug Fixes

- 1952e30 ast/estree: Serialize class constructor key as `Identifier` in TS-ESTree AST (#10471) (overlookmotel)
- fbf0ae2 estree: Add missing fixed fields to `AssignmentPattern` (#10490) (Yuji Sugiura)
- a42d85f estree: `FormalParameters` serializer for TS types (#10462) (Yuji Sugiura)
- 4f1343b parser: Fix missing type export in module information (#10516) (Ulrich Stark)

### Documentation

- 109cb21 napi/parser: Remove raw transfer from example (#10486) (overlookmotel)

### Refactor


## [0.65.0] - 2025-04-21

- 99d82db ast: [**BREAKING**] Move `type_parameters` field to before `extends` in `TSInterfaceDeclaration` (#10476) (overlookmotel)

- 7212803 ast: [**BREAKING**] Change `TSInterfaceDeclaration::extends` from `Option<Vec>` to `Vec` (#10472) (overlookmotel)

### Bug Fixes

- 1952e30 ast/estree: Serialize class constructor key as `Identifier` in TS-ESTree AST (#10471) (overlookmotel)
- fbf0ae2 estree: Add missing fixed fields to `AssignmentPattern` (#10490) (Yuji Sugiura)
- a42d85f estree: `FormalParameters` serializer for TS types (#10462) (Yuji Sugiura)
- 4f1343b parser: Fix missing type export in module information (#10516) (Ulrich Stark)

### Documentation

- 109cb21 napi/parser: Remove raw transfer from example (#10486) (overlookmotel)

### Refactor


## [0.64.0] - 2025-04-17

- c538efa ast: [**BREAKING**] `ImportExpression` only allows one option argument (#10432) (Boshen)

- 7284135 ast: [**BREAKING**] Remove `trailing_commas` from `ArrayExpression` and `ObjectExpression` (#10431) (Boshen)

- 771d50f ast: [**BREAKING**] Change `Class::implements` to `Vec<TSClassImplements>` (#10430) (Boshen)

- 521de23 ast: [**BREAKING**] Add `computed` property to `TSEnumMember` and `TSEnumMemberName::TemplateString` (#10092) (Yuji Sugiura)

- 49732ff ast: [**BREAKING**] Re-introduce `TSEnumBody` AST node (#10284) (Yuji Sugiura)

### Features

- 4c246fb ast: Add `override` field in `AccessorProperty` (#10415) (Yuji Sugiura)

### Bug Fixes

- f3ddefb ast/estree: Add missing fields to `AssignmentTargetRest` in TS-ESTree AST (#10456) (overlookmotel)
- 77b6f7e ast/estree: Fix start span of `Program` in TS-ESTree AST where first statement is `@dec export class C {}` (#10448) (overlookmotel)
- 4817c7e ast/estree: Add fields to `AssignmentTargetPattern` in TS-ESTree AST (#10423) (overlookmotel)
- b3094b3 ast/estree: Add `optional` field to `AssignmentTargetProperty` in TS-ESTree AST (#10412) (overlookmotel)
- a7fd30f ast/estree: Add fields to `BindingRestElement` in TS-ESTree AST (#10411) (overlookmotel)
- cc07efd ast/estree: Fix `JSXOpeningFragment` (#10208) (therewillbecode)
- 48ed6a1 ast/estree: Fix span for `TemplateElement` in TS AST (#10315) (overlookmotel)
- 2520b25 estree: Align `TSMappedType` fields (#10392) (Yuji Sugiura)
- 3ed3669 estree: Rename `JSDocXxxType` to `TSJSDocXxxType` (#10358) (Yuji Sugiura)
- b54fb3e estree: Rename `TSInstantiationExpression`.`type_parameters` to `type_arguments` (#10327) (Yuji Sugiura)

### Refactor

- 6e6c777 ast: Add `TSEnumMemberName` variant to replace `computed` field (#10346) (Yuji Sugiura)

## [0.64.0] - 2025-04-17

- c538efa ast: [**BREAKING**] `ImportExpression` only allows one option argument (#10432) (Boshen)

- 7284135 ast: [**BREAKING**] Remove `trailing_commas` from `ArrayExpression` and `ObjectExpression` (#10431) (Boshen)

- 771d50f ast: [**BREAKING**] Change `Class::implements` to `Vec<TSClassImplements>` (#10430) (Boshen)

- 521de23 ast: [**BREAKING**] Add `computed` property to `TSEnumMember` and `TSEnumMemberName::TemplateString` (#10092) (Yuji Sugiura)

- 49732ff ast: [**BREAKING**] Re-introduce `TSEnumBody` AST node (#10284) (Yuji Sugiura)

### Features

- 4c246fb ast: Add `override` field in `AccessorProperty` (#10415) (Yuji Sugiura)

### Bug Fixes

- f3ddefb ast/estree: Add missing fields to `AssignmentTargetRest` in TS-ESTree AST (#10456) (overlookmotel)
- 77b6f7e ast/estree: Fix start span of `Program` in TS-ESTree AST where first statement is `@dec export class C {}` (#10448) (overlookmotel)
- 4817c7e ast/estree: Add fields to `AssignmentTargetPattern` in TS-ESTree AST (#10423) (overlookmotel)
- b3094b3 ast/estree: Add `optional` field to `AssignmentTargetProperty` in TS-ESTree AST (#10412) (overlookmotel)
- a7fd30f ast/estree: Add fields to `BindingRestElement` in TS-ESTree AST (#10411) (overlookmotel)
- cc07efd ast/estree: Fix `JSXOpeningFragment` (#10208) (therewillbecode)
- 48ed6a1 ast/estree: Fix span for `TemplateElement` in TS AST (#10315) (overlookmotel)
- 2520b25 estree: Align `TSMappedType` fields (#10392) (Yuji Sugiura)
- 3ed3669 estree: Rename `JSDocXxxType` to `TSJSDocXxxType` (#10358) (Yuji Sugiura)
- b54fb3e estree: Rename `TSInstantiationExpression`.`type_parameters` to `type_arguments` (#10327) (Yuji Sugiura)

### Refactor

- 6e6c777 ast: Add `TSEnumMemberName` variant to replace `computed` field (#10346) (Yuji Sugiura)

## [0.63.0] - 2025-04-08

- a26fd34 ast: [**BREAKING**] Remove `JSXOpeningElement::self_closing` field (#10275) (overlookmotel)

### Bug Fixes

- e42c040 ast/estree: Add TS fields to `LabelIdentifier` (#10295) (overlookmotel)
- 06fc07c ast/estree: Fix `TSImportType` (#10200) (therewillbecode)
- 760188e ast/estree: Fix `BindingProperty` (#10193) (therewillbecode)
- f547d76 ast/estree: Add `TSEnumBody` to `TSEnumDeclaration.body` (#10017) (Yuji Sugiura)
- 34d5c00 ast/estree: Fix `ExportDefaultDeclaration` node (#10165) (therewillbecode)
- 498b479 ast/estree: Fix `AccessorProperty` node (#10067) (therewillbecode)
- bf90072 ast/estree: Fix `ObjectProperty` node (#10018) (therewillbecode)
- 27768a5 parser: Store lone surrogates in `TemplateElementValue` as escape sequence (#10182) (overlookmotel)
- 38d2bea parser: Fix parsing lone surrogates in `StringLiteral`s (#10180) (overlookmotel)
- 52f2a40 span/estree: Skip `ModuleKind::Unambiguous` varient for `estree` (#10146) (Dunqing)

### Refactor

- b662df4 ast/estree: Alter `Program` start span with converter (#10195) (overlookmotel)

### Testing

- bdded7e ast/estree: Add tests for JSX via raw transfer (#10241) (overlookmotel)

## [0.63.0] - 2025-04-08

- a26fd34 ast: [**BREAKING**] Remove `JSXOpeningElement::self_closing` field (#10275) (overlookmotel)

### Bug Fixes

- e42c040 ast/estree: Add TS fields to `LabelIdentifier` (#10295) (overlookmotel)
- 06fc07c ast/estree: Fix `TSImportType` (#10200) (therewillbecode)
- 760188e ast/estree: Fix `BindingProperty` (#10193) (therewillbecode)
- f547d76 ast/estree: Add `TSEnumBody` to `TSEnumDeclaration.body` (#10017) (Yuji Sugiura)
- 34d5c00 ast/estree: Fix `ExportDefaultDeclaration` node (#10165) (therewillbecode)
- 498b479 ast/estree: Fix `AccessorProperty` node (#10067) (therewillbecode)
- bf90072 ast/estree: Fix `ObjectProperty` node (#10018) (therewillbecode)
- 27768a5 parser: Store lone surrogates in `TemplateElementValue` as escape sequence (#10182) (overlookmotel)
- 38d2bea parser: Fix parsing lone surrogates in `StringLiteral`s (#10180) (overlookmotel)
- 52f2a40 span/estree: Skip `ModuleKind::Unambiguous` varient for `estree` (#10146) (Dunqing)

### Refactor

- b662df4 ast/estree: Alter `Program` start span with converter (#10195) (overlookmotel)

### Testing

- bdded7e ast/estree: Add tests for JSX via raw transfer (#10241) (overlookmotel)

## [0.62.0] - 2025-04-01

### Features

- 1ab8871 napi/parser: Auto download wasm binding on webcontainer (#10049) (Hiroshi Ogawa)

### Bug Fixes

- 95e69f6 ast/estree: Fix `StringLiteral`s containing lone surrogates (#10036) (overlookmotel)
- 8408606 ast/estree: Fix `TSMethodSignature` (#10032) (therewillbecode)
- 1a0bd7c ast/estree: Fix `TSPropertySignature` (#10031) (therewillbecode)
- 707a776 ast/estree: Fix TS type defs for `TSIndexSignature` and `TSIndexSignatureName` (#10003) (overlookmotel)
- c98d3f4 ast/estree: Add custom serializer for extends field of TSInterfaceDeclaration (#9996) (therewillbecode)
- f0e1510 parser: Store lone surrogates as escape sequence (#10041) (overlookmotel)

### Testing

- ab1a796 napi: Disable NAPI parser tests for TS files (#10028) (overlookmotel)

## [0.62.0] - 2025-04-01

### Features

- 1ab8871 napi/parser: Auto download wasm binding on webcontainer (#10049) (Hiroshi Ogawa)

### Bug Fixes

- 95e69f6 ast/estree: Fix `StringLiteral`s containing lone surrogates (#10036) (overlookmotel)
- 8408606 ast/estree: Fix `TSMethodSignature` (#10032) (therewillbecode)
- 1a0bd7c ast/estree: Fix `TSPropertySignature` (#10031) (therewillbecode)
- 707a776 ast/estree: Fix TS type defs for `TSIndexSignature` and `TSIndexSignatureName` (#10003) (overlookmotel)
- c98d3f4 ast/estree: Add custom serializer for extends field of TSInterfaceDeclaration (#9996) (therewillbecode)
- f0e1510 parser: Store lone surrogates as escape sequence (#10041) (overlookmotel)

### Testing

- ab1a796 napi: Disable NAPI parser tests for TS files (#10028) (overlookmotel)

## [0.61.2] - 2025-03-23

### Bug Fixes

- 89cb368 ast/estree: Add decorators field to `AssignmentPattern` (#9967) (therewillbecode)
- 4980b73 ast/estree: Add missing estree fields to `TSIndexSignature` and `TSIndexSignatureName` (#9968) (therewillbecode)
- b9f80b9 ast/estree: Fix `TSFunctionType` and `TSCallSignatureDeclaration`  (#9959) (therewillbecode)
- 0cdeedd ast/estree: Fix `ArrayPattern` (#9956) (therewillbecode)
- 6fcd342 ast/estree: Fix `FormalParameter` (#9954) (therewillbecode)
- 9d1035e ast/estree: Fix TS type def for `TSThisParameter` (#9942) (overlookmotel)
- 8228b74 ast/estree: Fix `Function.this_param` (#9913) (hi-ogawa)
- d69cc34 ast/estree: Fix `BindingIdentifier` (#9822) (hi-ogawa)
- 5631ebd ast/extree: Fix `TSModuleDeclaration.global` (#9941) (overlookmotel)

### Refactor

- db642eb ast/estree: Shorten raw deser code (#9944) (overlookmotel)

## [0.61.2] - 2025-03-23

### Bug Fixes

- 89cb368 ast/estree: Add decorators field to `AssignmentPattern` (#9967) (therewillbecode)
- 4980b73 ast/estree: Add missing estree fields to `TSIndexSignature` and `TSIndexSignatureName` (#9968) (therewillbecode)
- b9f80b9 ast/estree: Fix `TSFunctionType` and `TSCallSignatureDeclaration`  (#9959) (therewillbecode)
- 0cdeedd ast/estree: Fix `ArrayPattern` (#9956) (therewillbecode)
- 6fcd342 ast/estree: Fix `FormalParameter` (#9954) (therewillbecode)
- 9d1035e ast/estree: Fix TS type def for `TSThisParameter` (#9942) (overlookmotel)
- 8228b74 ast/estree: Fix `Function.this_param` (#9913) (hi-ogawa)
- d69cc34 ast/estree: Fix `BindingIdentifier` (#9822) (hi-ogawa)
- 5631ebd ast/extree: Fix `TSModuleDeclaration.global` (#9941) (overlookmotel)

### Refactor

- db642eb ast/estree: Shorten raw deser code (#9944) (overlookmotel)

## [0.61.1] - 2025-03-21

### Features

- 8e3b20d napi/parser: Add portable wasm browser build (#9901) (Hiroshi Ogawa)

## [0.61.1] - 2025-03-21

### Features

- 8e3b20d napi/parser: Add portable wasm browser build (#9901) (Hiroshi Ogawa)

## [0.61.0] - 2025-03-20

- c631291 parser: [**BREAKING**] Parse `TSImportAttributes` as `ObjectExpression` (#9902) (Boshen)

### Features

- 6565fc4 napi: Feature gate allocator (#9921) (Boshen)
- 2cedfe4 napi: Add codeframe to napi error (#9893) (Boshen)
- a9a47a6 parser: Add regex cargo feature to oxc_parser (#9879) (Toshit)
- 59c8f71 parser,codegen: Handle lone surrogate in string literal (#9918) (Boshen)

### Bug Fixes

- 28a2ed3 estree/ast: Fix `IdentifierName` and `IdentifierReference` (#9863) (hi-ogawa)

### Performance

- 5f97f28 ast/estree: Speed up raw deser for `JSXElement` (#9895) (overlookmotel)

### Documentation

- 590a258 napi/parser: Add stackblitz link for wasm build (Boshen)

### Refactor

- 961b95d napi: Move common code to `oxc_napi` (#9875) (Boshen)
- 233c1fc napi/playground: Add JSON.parse wrapper (#9880) (Hiroshi Ogawa)

### Testing

- 040e993 napi: Refactor NAPI parser benchmarks (#9911) (overlookmotel)
- e637e2e napi/parser: Tweak vitest config (#9878) (Hiroshi Ogawa)

## [0.61.0] - 2025-03-20

- c631291 parser: [**BREAKING**] Parse `TSImportAttributes` as `ObjectExpression` (#9902) (Boshen)

### Features

- 6565fc4 napi: Feature gate allocator (#9921) (Boshen)
- 2cedfe4 napi: Add codeframe to napi error (#9893) (Boshen)
- a9a47a6 parser: Add regex cargo feature to oxc_parser (#9879) (Toshit)
- 59c8f71 parser,codegen: Handle lone surrogate in string literal (#9918) (Boshen)

### Bug Fixes

- 28a2ed3 estree/ast: Fix `IdentifierName` and `IdentifierReference` (#9863) (hi-ogawa)

### Performance

- 5f97f28 ast/estree: Speed up raw deser for `JSXElement` (#9895) (overlookmotel)

### Documentation

- 590a258 napi/parser: Add stackblitz link for wasm build (Boshen)

### Refactor

- 961b95d napi: Move common code to `oxc_napi` (#9875) (Boshen)
- 233c1fc napi/playground: Add JSON.parse wrapper (#9880) (Hiroshi Ogawa)

### Testing

- 040e993 napi: Refactor NAPI parser benchmarks (#9911) (overlookmotel)
- e637e2e napi/parser: Tweak vitest config (#9878) (Hiroshi Ogawa)

## [0.60.0] - 2025-03-18

### Features

- aa3dff8 napi: Add mimalloc to parser and transformr (#9859) (Boshen)

### Performance

- 2d63704 ast: Re-order `VariableDeclarationKind` variants (#9853) (overlookmotel)

### Refactor

- 7106e5d napi: Disable unused browser fs (#9848) (hi-ogawa)

## [0.60.0] - 2025-03-18

### Features

- aa3dff8 napi: Add mimalloc to parser and transformr (#9859) (Boshen)

### Performance

- 2d63704 ast: Re-order `VariableDeclarationKind` variants (#9853) (overlookmotel)

### Refactor

- 7106e5d napi: Disable unused browser fs (#9848) (hi-ogawa)

## [0.59.0] - 2025-03-18

- 3d17860 ast: [**BREAKING**] Reorder fields of `TemplateElement` (#9821) (overlookmotel)

- ce6808a parser: [**BREAKING**] Rename `type_parameters` to `type_arguments` where needed  (#9815) (hi-ogawa)

### Features

- db946e6 ast/estree: Order TS fields last by default (#9820) (overlookmotel)
- 06518ae napi/parser: `JSON.parse` the returned AST in wasm (#9630) (Boshen)

### Bug Fixes

- 3f858c4 ast/estree: Add `directive` field to `ExpressionStatement` in TS AST (#9844) (overlookmotel)
- cd18358 ast/extree: Fix `Class.implements` (#9817) (hi-ogawa)

### Refactor


### Testing

- 48bac92 napi/parser: Test wasi browser (#9793) (Hiroshi Ogawa)

## [0.59.0] - 2025-03-18

- 3d17860 ast: [**BREAKING**] Reorder fields of `TemplateElement` (#9821) (overlookmotel)

- ce6808a parser: [**BREAKING**] Rename `type_parameters` to `type_arguments` where needed  (#9815) (hi-ogawa)

### Features

- db946e6 ast/estree: Order TS fields last by default (#9820) (overlookmotel)
- 06518ae napi/parser: `JSON.parse` the returned AST in wasm (#9630) (Boshen)

### Bug Fixes

- 3f858c4 ast/estree: Add `directive` field to `ExpressionStatement` in TS AST (#9844) (overlookmotel)
- cd18358 ast/extree: Fix `Class.implements` (#9817) (hi-ogawa)

### Refactor


### Testing

- 48bac92 napi/parser: Test wasi browser (#9793) (Hiroshi Ogawa)

## [0.58.1] - 2025-03-13

### Bug Fixes

- cd3f2fb ast/estree: Fix `JSXOpeningFragment` (#9747) (Hiroshi Ogawa)
- fecec56 ast/estree: Fix `JSXOpeningElement` field order (#9746) (hi-ogawa)

## [0.58.1] - 2025-03-13

### Bug Fixes

- cd3f2fb ast/estree: Fix `JSXOpeningFragment` (#9747) (Hiroshi Ogawa)
- fecec56 ast/estree: Fix `JSXOpeningElement` field order (#9746) (hi-ogawa)

## [0.58.0] - 2025-03-13

- 842edd8 ast: [**BREAKING**] Add `raw` property to `JSXText` node (#9641) (Yuji Sugiura)

### Features

- 446d11e ast/estree: Export `Node` union type (#9574) (hi-ogawa)

### Bug Fixes

- 475b48f ast: Change `ImportExpression::attributes` to `options` (#9665) (Boshen)

### Documentation

- a6c9b09 napi/minifier: Improve documentation (#9736) (Boshen)

## [0.58.0] - 2025-03-13

- 842edd8 ast: [**BREAKING**] Add `raw` property to `JSXText` node (#9641) (Yuji Sugiura)

### Features

- 446d11e ast/estree: Export `Node` union type (#9574) (hi-ogawa)

### Bug Fixes

- 475b48f ast: Change `ImportExpression::attributes` to `options` (#9665) (Boshen)

### Documentation

- a6c9b09 napi/minifier: Improve documentation (#9736) (Boshen)

## [0.57.0] - 2025-03-11

- 510446a parser: [**BREAKING**] Align JSXNamespacedName with ESTree (#9648) (Arnaud Barré)

### Features

- 638007b parser: Apply `preserveParens` to `TSParenthesizedType` (#9653) (Boshen)

### Bug Fixes

- eae1a41 ast: Align `TSImportType` field names with ts-eslint (#9664) (Boshen)
- 6ac3635 napi/parser: Disable raw transfer on unsupported platforms (#9651) (overlookmotel)

### Refactor

- c6edafe napi: Remove `npm/oxc-*/` npm packages (#9631) (Boshen)

## [0.57.0] - 2025-03-11

- 510446a parser: [**BREAKING**] Align JSXNamespacedName with ESTree (#9648) (Arnaud Barré)

### Features

- 638007b parser: Apply `preserveParens` to `TSParenthesizedType` (#9653) (Boshen)

### Bug Fixes

- eae1a41 ast: Align `TSImportType` field names with ts-eslint (#9664) (Boshen)
- 6ac3635 napi/parser: Disable raw transfer on unsupported platforms (#9651) (overlookmotel)

### Refactor

- c6edafe napi: Remove `npm/oxc-*/` npm packages (#9631) (Boshen)

## [0.56.4] - 2025-03-07

### Bug Fixes

- c08b7fc napi: Commit wasi files (Boshen)

## [0.56.3] - 2025-03-07

### Features

- 6b95d25 parser: Disallow `TSInstantiationExpression` in `SimpleAssignmentTarget` (#9586) (Boshen)

## [0.56.0] - 2025-03-06

### Bug Fixes

- 91c9932 napi: Do not support raw transfer on WASM32 (#9566) (overlookmotel)

## [0.55.0] - 2025-03-05

- 4056560 ast/estree: [**BREAKING**] Option to return JS-only AST (#9520) (overlookmotel)

### Features

- af02a87 ast/estree: `Property` have consistent field order (#9547) (overlookmotel)
- 3e4f909 ast/estree: ESTree AST `ExportNamedDeclaration` always have `attributes` field (#9546) (overlookmotel)
- d55dbe2 ast/estree: Raw transfer (experimental) (#9516) (overlookmotel)

### Bug Fixes

- a0f6f37 ast/estree: Raw transfer support `showSemanticErrors` option (#9522) (overlookmotel)

### Refactor

- c1a8cea ast/estree: Simplify serializing `RegExpLiteral`s (#9551) (overlookmotel)

### Testing

- 4378a66 ast/estree: Speed up raw transfer tests (#9521) (overlookmotel)

## [0.54.0] - 2025-03-04

- 355a4db napi/parser: [**BREAKING**] Remove `parse_without_return` API (#9455) (Boshen)

- a5cde10 visit_ast: [**BREAKING**] Add `oxc_visit_ast` crate (#9428) (Boshen)

### Features

- 68c77c8 napi/parser: Return semantic errors (#9460) (Boshen)

### Testing

- d129055 napi: Add tests for worker threads (#9408) (Boshen)
- 48d51e3 napi: Add tests for `hashbang` field (#9386) (overlookmotel)

## [0.53.0] - 2025-02-26

- 4a5a7cf napi/parser: [**BREAKING**] Remove magic string; enable utf16 span converter by default (#9291) (Boshen)

### Features


### Performance

- 61939ca ast/estree: Faster UTF-8 to UTF-16 span conversion (#9349) (overlookmotel)

### Refactor

- b09249c ast/estree: Rename serializers and serialization methods (#9284) (overlookmotel)

## [0.52.0] - 2025-02-21

- 216b33f ast/estree: [**BREAKING**] Replace `serde` with custom `ESTree` serializer (#9256) (overlookmotel)

### Features


### Bug Fixes

- b9c8a10 wasm: Transfer AST to JS as JSON string in `oxc-wasm` (#9269) (overlookmotel)
- 5acc6ec wasm: Transfer AST to JS as JSON string (#9259) (overlookmotel)

## [0.51.0] - 2025-02-15

### Bug Fixes

- 0937a55 napi/parser: Utf16 span for errors (#9112) (hi-ogawa)
- 15f23f1 napi/parser: Utf16 span for module record (#9093) (hi-ogawa)
- 9edfb1d napi/parser: Fix unicode comment panic (#9084) (hi-ogawa)

### Performance

- af59945 napi/parser: Do not convert comment spans twice (#9087) (overlookmotel)

### Testing

- eaff3d9 napi/parser: Split tests for `convertSpanUtf16` (#9113) (hi-ogawa)

## [0.50.0] - 2025-02-12

### Features

- 81c81a7 napi/parser: Add `convert_span_utf16` option (#8983) (hi-ogawa)

### Bug Fixes

- 41dba62 ast/estree: Set `value` for `BigIntLiteral`s and `RegExpLiteral`s on JS side (#9044) (overlookmotel)

### Testing

- ef553b9 napi: Add NAPI parser benchmark (#9045) (overlookmotel)

## [0.49.0] - 2025-02-10

### Bug Fixes

- a520986 ast: Estree compat `Program.sourceType` (#8919) (Hiroshi Ogawa)
- e30cf6a ast: Estree compat `MemberExpression` (#8921) (Hiroshi Ogawa)
- 0c55dd6 ast: Serialize `Function.params` like estree (#8772) (Hiroshi Ogawa)

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

### Testing

- 4803059 ast: Remove old ast snapshot tests (#8976) (hi-ogawa)

## [0.47.1] - 2025-01-19

### Features

- ee8ee55 napi/parser: Add `.hasChanged()` to `MagicString` (#8586) (Boshen)
- 1bef911 napi/parser: Add source map API (#8584) (Boshen)

## [0.47.0] - 2025-01-18

### Features

- c479a58 napi/parser: Expose dynamic import expressions (#8540) (Boshen)

