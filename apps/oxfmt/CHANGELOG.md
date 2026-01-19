# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.25.0] - 2026-01-19

### 🚀 Features

- a95b9bb oxfmt: Support oxfmtrc `overrides` config (#18068) (leaysgur)
- 984d5c1 oxfmt/sort-imports: Support `options.customGroups` (#17576) (nilptr)
- fd2c792 formatter: Support css prop, styled jsx, and member/computed `styled.tags` (#17990) (magic-akari)
- 361a8f1 oxfmt: Upgrade `prettier` to 3.8.0 (#18024) (Dunqing)
- 873c683 oxfmt: Add more tracing logs (#18015) (Yuji Sugiura)
- cc3e74b oxfmt: Add Prettier specific fields in `Oxfmtrc` (#17981) (leaysgur)
- 6ffe315 oxfmt: Add more `Oxfmtrc` fields description (#17979) (leaysgur)

### 🐛 Bug Fixes

- 2a397f8 oxlint/lsp: Don't send `workspace/diagnostic/refresh` notification on watched file changes (#17885) (Sysix)
- efacb13 oxfmt: Do not wrap with `block_indent()` if `format_embedded` fails (#17975) (leaysgur)
- 9d0f551 oxfmt: Do not panic with subdirectry and config (#17955) (leaysgur)
- 9d96cc6 oxfmt: Use `std(out/err)._handle.setBlocking(true)` to handle `WouldBlock` error in Rust (#17950) (leaysgur)

## [0.24.0] - 2026-01-12

### 🚀 Features

- 2e03ebf oxfmt/lsp: Use `SourceFormatter` to support non-JS files and napi features (#17655) (leaysgur)
- 623f7eb oxfmt/sort_package_json: Use `options.sort_scripts` (#17740) (leaysgur)
- 86c0168 oxfmt/sort_package_json: Handle `oxfmtrc.sort_scripts` option (#17738) (leaysgur)
- 256636a oxfmt/lsp: Add `.editorconfig` to `get_watcher_patterns` (#17694) (leaysgur)
- 3f3db39 oxfmt/lsp: Use `ConfigResolver` to align with CLI (#17654) (leaysgur)

### 🐛 Bug Fixes

- 9e89389 formatter/tailwindcss: Nested class string doesn't respect `singleQuote: true` (#17838) (Dunqing)
- f0cedd4 formatter/tailwindcss: Class name is broken after sorting when its contains single quotes with `singleQuote: true` (#17790) (Dunqing)
- 1864142 oxfmt/tailwindcss: Bundle `prettier/plugins/*` (#17782) (leaysgur)
- 3a9d43b oxfmt: Ignore explicit positional path which is ignored by directory (#17732) (leaysgur)
- 0563217 formatter: Classes will be stripped out when both `experimentalTailwindcss` and `experimentalSortImports` are enabled (#17726) (Dunqing)

### 📚 Documentation

- 62b7a01 formatter: Clarify `experimentalTailwindcss` configuration comments (#17898) (Dunqing)

## [0.23.0] - 2026-01-06

### 🚀 Features

- a19cc93 oxfmt: Add debug logging to oxfmt LSP to troubleshoot resolved options at runtime (#17695) (Nicholas Rayburn)

### 🐛 Bug Fixes

- dcfdd41 formatter: Should not set up tailwindcss callback when no tailwindcss configuration is set (#17696) (Dunqing)

## [0.22.0] - 2026-01-05

### 🚀 Features

- 8fd4ea9 oxfmt: `options.embeddedLanguageFormatting` is now `"auto"` by default (#17649) (leaysgur)

### 🐛 Bug Fixes

- 174375d oxfmt,oxlint: Disable mimalloc for 32-bit Arm targets (#17473) (Yaksh Bariya)

### ⚡ Performance

- abb28dc oxfmt: Turn of pretty print from sort-package-json (#17452) (Boshen)

## [0.21.0] - 2025-12-29

### 🚀 Features

- 4df8063 oxfmt: Respect `.gitignore` in sub directries (#17352) (leaysgur)

### 🐛 Bug Fixes

- c6690d1 rust: Remove unsupported tokio io-std feature for WASM compatibility (#17311) (Boshen)

## [0.20.0] - 2025-12-22

### 🚀 Features

- 97a02d1 oxfmt: Add `insertFinalNewline` option (#17251) (leaysgur)
- a3f3c58 oxfmt: Support TOML(v1.0 only) files (#17113) (leaysgur)

### 🐛 Bug Fixes

- 7b810f4 oxfmt: Use correct root dir with ignore and overrides for nested cwd (#17244) (leaysgur)
- cdb80d4 oxfmt: Resolve `.editorconfig` root dir from `cwd` (#17093) (leaysgur)

## [0.19.0] - 2025-12-19

### 🚀 Features

- 15dfb55 oxfmt: Respect single nearest `.editorconfig` (#17043) (leaysgur)
- 8c33ff4 oxfmt: Expose Node.js API: `format(fileName, sourceText, options?)` (#16939) (leaysgur)

### 🐛 Bug Fixes

- d340c87 oxfmt: Update api `FormatOptions` type with `& Record<string, unknown>` (#17036) (leaysgur)
- 827a256 oxfmt: Place ignorePatterns at bottom of JSON in --migrate prettier (#16926) (Boshen)

## [0.18.0] - 2025-12-15

### 🚀 Features

- 5e3ceb8 oxfmt: Support `oxfmt --stdin-filepath` (#16868) (leaysgur)
- d4c0bb7 oxfmt: Support `oxfmt --migrate prettier` (JS side) (#16773) (leaysgur)
- 2b9c3fe oxfmt: Support `oxfmt --migrate [prettier]` (Rust side) (#16771) (leaysgur)
- 47c8710 oxfmt: Arrange cli mode and update help (#16728) (leaysgur)
- 559eff1 oxfmt: Support `oxfmt --init` (#16720) (leaysgur)
- 28e0682 oxfmt: Enable experimental `package.json` sorting by default (#16593) (leaysgur)
- feffe48 oxfmt: Trace which files are being formatted via `OXC_LOG=debug` (#16627) (Boshen)

### 🐛 Bug Fixes

- bc2e0f8 oxfmt: Report `exitCode` correctly (#16770) (leaysgur)
- d719988 oxfmt: Make Rust CLI as just formatting CLI (#16768) (leaysgur)
- 2577814 oxfmt: Remove `jsonc` parser override for `(j|t)sconfig(.*)?.json` (#16762) (leaysgur)
- 02f59ba oxfmt: Always respect ignored files even specified (#16632) (leaysgur)
- 37c1a06 oxfmt: Exclude lock files to be formatted (#16629) (leaysgur)

### ⚡ Performance

- 10b4f9f oxfmt: Make time measurement conditional (#16634) (Boshen)
- 6f3aaba oxfmt: Use `worker_threads` by `tinypool` for prettier formatting (#16618) (leaysgur)

## [0.17.0] - 2025-12-08

### 🚀 Features

- 3184f17 oxfmt: Pass filepath field to prettier formatting (#16591) (Yuji Sugiura)
- 7bb3304 oxfmt: Pass populated config to prettier formatting (#16584) (leaysgur)
- 69f84d2 oxfmt: Pass raw config to prettier formatting (#16582) (leaysgur)
- a83a2ec oxfmt: Expose `setupConfig(configJSON: string)` napi callback (#16579) (leaysgur)
- af76b0e oxfmt: Support formatting HTML, YAML, GraphQL, Handlerbars, Markdown, CSS files (#16524) (leaysgur)
- 66b64ef oxfmt: Support formatting JSON files (#16523) (leaysgur)
- 4767926 oxfmt: Prepare non-js/ts file support with prettier (#16480) (leaysgur)
- 2b4ce5d oxfmt: Use dedicated `format_by_xxx_formatter` method by `SourceType` (#16417) (leaysgur)
- 0867d2f oxfmt: Set up JS `formatFile()` function for Rust via napi (#16415) (leaysgur)
- b6feb66 oxfmt: Rename `embedded.ts` with preparing `formatFile()` function (#16414) (leaysgur)
- dd2cb62 oxfmt: Not error on explicit `--write` flag used (#16376) (leaysgur)

## [0.16.0] - 2025-12-01

### 🚀 Features

- 116e0d1 website: Auto generate oxfmt docs (#15985) (Boshen)

### 🐛 Bug Fixes

- 0faa978 oxfmt: JsFormatEmbeddedCb types (#16324) (Brooooooklyn)
- 653fa6c oxlint/oxfmt/lsp: Tell client the real tool name & version (#16212) (Sysix)
- 38b7bc4 oxfmt: Make no-napi build work (#16134) (leaysgur)
- 14b0a6a oxfmt: Fix JS-ish file detection (#16092) (leaysgur)

## [0.15.0] - 2025-11-24

### 🚀 Features

- f9a502c oxfmt: `oxfmt --lsp` support (#15765) (leaysgur)

## [0.14.0] - 2025-11-17

### 🚀 Features

- 99823ad oxfmt: Print nothing for default(write) mode (#15583) (leaysgur)

### ⚡ Performance

- d99a83f oxfmt: Use simdutf8 based read_to_string (#15614) (leaysgur)

### 📚 Documentation

- 3d15805 linter: Reformat doc comments (#15670) (overlookmotel)

## [0.12.0] - 2025-11-10

### 🚀 Features

- 3251000 oxfmt: Use `prettier` directly and bundle `prettier` (#15544) (Dunqing)
- 7b1e6f3 apps: Add pure rust binaries and release to github (#15469) (Boshen)
- 33ad374 oxfmt: Disable embedded formatting by default for alpha (#15402) (leaysgur)

### ⚡ Performance

- a6808a0 oxfmt: Use `AllocatorPool` to reuse allocator between threads (#15412) (leaysgur)


## [0.10.0] - 2025-11-04

### 🚀 Features

- b77f254 oxfmt,formatter: Support `embeddedLanguageFormatting` option (#15216) (leaysgur)
- 898d6fe oxfmt: Add embedded language formatting with Prettier integration (#14820) (Boshen)

### 🐛 Bug Fixes

- daacf85 oxfmt: Release build fails (#15262) (Dunqing)
- f5d0348 oxfmt: Sync `dependencies` with `npm/oxfmt` and `apps/oxfmt` (#15261) (leaysgur)

### 🚜 Refactor

- 27b4f36 diagnostic: Remove `path` from sender (#15130) (camc314)


## [0.9.0] - 2025-10-30

### 🚜 Refactor

- 5de99c2 formatter: Export unified way to get_parse_options (#15027) (leaysgur)

### 💼 Other

- aceff66 oxfmt: V0.9.0 (#15088) (Boshen)



## [0.8.0] - 2025-10-22

### 🚀 Features

- 006708d oxfmt: Support `ignorePatterns` in oxfmtrc (#14875) (leaysgur)


## [0.7.0] - 2025-10-21

### 🚀 Features

- 6dfcd80 oxfmt: Search both .json and .jsonc config file (#14848) (leaysgur)

### 🐛 Bug Fixes

- 7a420a1 oxfmt: Handle `.d.ts` file correctly (#14835) (leaysgur)

### 🚜 Refactor

- 6fa7420 oxfmt: Use custom ignore builder (#14850) (leaysgur)


## [0.6.0] - 2025-10-20

### 🚀 Features

- 7f91a26 oxfmt: Handle ignoring files (#14798) (leaysgur)
- 199a2c6 oxfmt: Support `--with-node-modules` option (#14713) (leaysgur)
- 26c5f5a oxfmt: Ignore VCS directories by default (#14616) (leaysgur)
- fec2ed9 oxfmt: Use Prettier style config key and value (#14612) (leaysgur)
- 1b58521 oxfmt,language_server: Enable JSX for all JS source type (#14605) (leaysgur)

### 🐛 Bug Fixes

- ef02760 oxfmt: Handle relative path starts with dot (#14708) (leaysgur)
- ee37f5d oxfmt: Handle default cwd correctly (#14704) (leaysgur)
- 0961c3a oxlint,oxfmt: Skip traversing `.git` directories (#14590) (Boshen)

### 🚜 Refactor

- b7926f3 oxfmt: Update CLI --help details (#14796) (leaysgur)
- 173168b oxfmt: Refactor walk.rs and format.rs relationship (#14795) (leaysgur)
- aea9d79 oxfmt: Pass `PathBuf` from walk.rs to service.rs (#14716) (leaysgur)

### 🧪 Testing

- 7c42ea0 oxfmt: Remove args from snapshot file name (#14800) (leaysgur)


## [0.5.0] - 2025-10-14

### 🚀 Features

- 51ddfa8 oxfmt: Support `.oxfmtrc.json(c)` config file (#14398) (leaysgur)

### 🐛 Bug Fixes

- 0f19be0 oxfmt: Normalize path delimiter on Windows (#14463) (leaysgur)



## [0.3.0] - 2025-09-19

### 🐛 Bug Fixes

- 55775ce oxfmt: Fix up the half-finished lines (#13840) (leaysgur)

### ⚡ Performance

- 59db021 oxfmt: Walk and format at the same time (#13838) (leaysgur)


## [0.2.0] - 2025-09-16

### 💥 BREAKING CHANGES

- d90bebc oxfmt: [**BREAKING**] Change default behavior more `cargo fmt` like (#13794) (leaysgur)

### 🧪 Testing

- afa2297 oxfmt: Fix failing tests on Windows (#13801) (leaysgur)
- 5fbffcf oxfmt: Enable changing `cwd` during tests (#13797) (leaysgur)


## [0.1.0] - 2025-09-12

### 🚀 Features

- 1d72f8b oxfmt: Support --no-error-on-unmatched-pattern (#13671) (leaysgur)

### 🐛 Bug Fixes

- d6628bf oxfmt: Print sorted output (#13709) (leaysgur)
- 056f6de oxfmt: Set preserve_parens: false to prevent panic (#13666) (leaysgur)

### 🚜 Refactor

- 6b74078 formatter: Move `is_supported_source_type` to `oxc_formatter` crate (#13702) (Sysix)

### 🧪 Testing

- 83d735b oxfmt: Use normalized path separator (#13726) (leaysgur)
- 289ef9b oxfmt: Add tests setup (#13684) (leaysgur)



