# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [1.33.0] - 2025-12-15

### ⚡ Performance

- 5eb8b14 lsp: Avoid one clone in code actions request (#16543) (Sysix)

## [1.32.0] - 2025-12-08

### 🐛 Bug Fixes

- 0c14531 oxlint/lsp: Revalidate all known files after internal restart (#16407) (Sysix)
- 5fd3096 lsp: Skip (un)registering file watcher when not dynamic watchers are supported (#16406) (Sysix)
- 804144f lsp: Respect `null` as invalid settings for `workspace/didChangeConfiguration` (#16361) (Sysix)
- 114665d lsp: Watch for files when workspace folder is added & `workspace/configuration` is not supported (#16307) (Sysix)

### ⚡ Performance

- 67cfa58 lsp: Avoid some clones (#16519) (Sysix)
- 2a09665 oxlint/lsp: Store only `LinterCodeAction` instead of the complete `DiagnosticReport` (#16516) (Sysix)
- 0ca9754 lsp: Avoid `Uri` to `String` and back to `Uri` conversion (#16470) (Sysix)
- 2736636 lsp: Avoid cloning when revalidating diagnostics (#16469) (Sysix)

### 📚 Documentation

- c0766df linter: Add config option docs for `unicorn/filename-case` rule. (#16280) (connorshea)

## [1.31.0] - 2025-12-01

### 🐛 Bug Fixes

- 653fa6c oxlint/oxfmt/lsp: Tell client the real tool name & version (#16212) (Sysix)
- 135cee6 oxlint/lsp: Run diagnostics after delayed worker initialization (#16244) (Sysix)

## [1.30.0] - 2025-11-24

### 🐛 Bug Fixes

- e4ba07f language_server: Always write to memory file system (#15975) (Sysix)
- 619a226 oxlint/lsp: Don't register `textDocument/formatting` capability (#15882) (Sysix)

## [1.29.0] - 2025-11-17

### 🚀 Features

- ac5c2ab lsp/fmt: Support ignore files & patterns (#15647) (Sysix)
- 4c62ffa language_server: Add `run_server` helper for CLI tools (#15609) (Sysix)

### ⚡ Performance

- 7528db4 language_server: Pass file content as a referenced `String` (#15568) (Sysix)
- ccfd935 language_server: Avoid `Mutex` in `ServerLinter` (#15516) (Sysix)

## [1.27.0] - 2025-11-10

### 🐛 Bug Fixes

- ecf2f7b language_server: Fail gracefully when tsgolint executable not found (#15436) (camc314)

### ⚡ Performance

- 25d577e language_server: Start tools in parallel (#15500) (Sysix)

## [1.26.0] - 2025-11-05

### 🚀 Features

- 798216b language_server: Respect disable directives for type-aware rules (#15170) (Sysix)

### 🐛 Bug Fixes

- e0edaef language-server: Disable tsgolint test on big endian (#15331) (camc314)
- 86cfae1 language-server: Log error if tsgolint fails to run (#15228) (camc314)
- 80a187c linter: Add offset for parsing error in partial loading files (#15075) (Liang Mi)

### 🚜 Refactor

- 778b0b6 language_server: Remove `ServerLinterDiagnostics` (#15169) (Sysix)
- 10732e8 language_server: `Backend` checks the correct `LintOptions::Run` (#15166) (Sysix)
- e70a37f language_server: Use `LintRunner` (#14472) (Sysix)

### 🧪 Testing

- 48e646b oxlint/lsp: Update snapshot for invalid syntax test (#15191) (Sysix)


## [1.25.0] - 2025-10-30

### 💥 BREAKING CHANGES

- 659fd37 linter: [**BREAKING**] `tsgolint`: request fixes when necessary (#15048) (camchenry)

### 🚜 Refactor

- b1e1531 language_server: Extract library interface from main.rs (#15036) (Boshen)
- 5de99c2 formatter: Export unified way to get_parse_options (#15027) (leaysgur)
- b55df7f language_server: Move sub option for `flags` to the root + deprecate flags (#14933) (Sysix)


## [1.24.0] - 2025-10-22

### 🚀 Features

- e3d65c3 language_server: Search both .json and .jsonc config file (#14868) (Sysix)
- 2609c74 language_server: Watch for `fmt.configPath` file content change (#14509) (Sysix)
- fec2ed9 oxfmt: Use Prettier style config key and value (#14612) (leaysgur)
- 1b58521 oxfmt,language_server: Enable JSX for all JS source type (#14605) (leaysgur)

### 🐛 Bug Fixes

- 644dfd4 language_server: Make unused directives fixable again (#14872) (Sysix)
- e560cc1 language_server: Fix panic when "disable rule for this line" position is after error span (#14597) (Sysix)

### 🚜 Refactor

- 699c91c language_server: Improve file watching for different tools (#14645) (Sysix)
- 3bfb8e1 linter: Make `Message.span` public (#14601) (Sysix)
- 9a589ca language_server: Use Message span for Diagnostic's Range (#14600) (Sysix)
- b301795 linter: Remove unused allocator param in `Runtime` for LSP & tests (#14489) (Sysix)

### ⚡ Performance

- df48416 language_server: Return `Diagnostic` when relinting all files (#14737) (Sysix)
- 22ee19f language_server: Avoid creating HashMap in `Backend::did_change_configuration` (#14736) (Sysix)
- ed922ec language_server: Avoid creating HashMap in `Backend::did_change_watched_files` (#14735) (Sysix)


## [1.23.0] - 2025-10-13

### 🚀 Features

- 4fbdef2 language_server: Support `fmt.configPath` configuration (#14502) (Sysix)
- 4d3ce2e language_server: Autodetect root `.oxfmtrc.json` (#14466) (Sysix)

### 🐛 Bug Fixes

- fb4a20d language_server: Add whitespace for `// oxlint-disable-next-line` fix (#14356) (Sysix)

### 🚜 Refactor

- 6440cde linter: Remove lifetime of `Message` (#14481) (Sysix)
- f599bef language_server: Move `Message` to `DiagnosticReport` transformation to one place (#14447) (Sysix)
- a9cea7c language_server: Use `FxHashSet` for `ServerLinter::extended_paths` (#14517) (Sysix)
- f977700 language_server: Pass `LintOptions` to `Worker::refresh_server_linter` (#14510) (Sysix)

### ⚡ Performance

- b44a30e language_server: Transform `Message` to `DiagnosticReport` with one call (#14448) (Sysix)

### 🧪 Testing

- 33b6cde language_server: Add basic tests for `WorkspaceWorker::did_change_configuration` (#14531) (Sysix)
- bfe1ecd language_server: Add tests for `WorkspaceWorker::init_watchers` (#14516) (Sysix)



## [1.21.0] - 2025-10-08

### 🐛 Bug Fixes

- 493082c language_server: Use the first Span of the message as the primary Diagnostic range (#14057) (Sysix)
- 6e8d2f6 language_server: Ignore JS plugins (#14379) (overlookmotel)

### 🚜 Refactor

- 3374b8e linter/language_server: Move all lsp relevant code to `oxc_language_server` crate (#14430) (Sysix)
- d24b74e linter/language_server: `oxc_linter::TsgoLinter::run_source` returns `Message` (#14429) (Sysix)
- e5b7fb2 linter/language_server: `oxc_linter::Runtime::run_source` returns `Message` (#14428) (Sysix)


## [1.20.0] - 2025-10-06

### 🚀 Features

- d16df93 linter: Support disable directives for type aware rules (#14052) (camc314)

### 🐛 Bug Fixes

- adff069 language_server: Don't apply "ignore this rule" fixes for fixAll code action + command (#14243) (Sysix)

### 🚜 Refactor

- 891fc47 language_server: Share code for command `oxc.fixAll` and code action `source.fixAll.oxc` (#14244) (Sysix)
- 7fe930c language_server: Remove unused fixture files (#14246) (Sysix)
- 2b2c345 language-server: Move `generate_inverted_diagnostics` to `error_with_position` (#14118) (camc314)

### ⚡ Performance

- fa3712d language_server: Create less `ExternalPluginStore`s (#14378) (overlookmotel)


## [1.19.0] - 2025-09-29

### 🚀 Features

- 1472147 linter: Move `no-unused-expressions` to correctness (#14099) (camchenry)

### 🐛 Bug Fixes

- e37c435 language_server: Correct position for "ignore this rule for this file" in vue/astro/svelte files (#14187) (Sysix)
- d36d227 language_server: Don't lint file on code action when it is already ignored (#13976) (Sysix)
- 353bfe7 language_server: Check if tsconfig path is a file before starting the `LintService` (#14126) (Sysix)

### 🚜 Refactor

- 7a0eb57 language_server: Refactor ignore code action logic as a linter fix (#14183) (Sysix)

### 📚 Documentation

- b83b1bd language_server: Docs for `Backend` struct (#14172) (Sysix)
- 3106ba0 language_server: Docs for `WorkspaceWorker` (#14161) (Sysix)

### 🧪 Testing

- be58d6d language_server: Fix test for ServerFormatter in windows (#14210) (Sysix)
- d7041c1 language_server: Add linebreaks for formatter snapshot (#14173) (Sysix)



## [1.17.0] - 2025-09-23

### 🚀 Features

- a21ff54 language_server: Introduce `ServerFormatter` (#13700) (Sysix)

### 🐛 Bug Fixes

- b8790c2 language_server: Output correct position for parser & semantic errors (#14010) (Sysix)

### 🚜 Refactor

- 0c93f33 language_server: Use minimal text edit for `ServerFormatter` (#13960) (Sysix)
- 823cb57 langauge_server: Move `Backend` into own file (#13955) (Sysix)
- dc700f5 language_server: Introduce `LSPFileSystem` (#13731) (Sysix)
- 873e231 language_server: Move `needs_linter_restart` to `ServerLinter` (#13834) (Sysix)

### 🧪 Testing

- b807b6f language_server: Skip formatter test on windows (#13986) (Sysix)
- 2600858 language_server: Add Tester for ServerFormatter (#13969) (Sysix)
- 37aadf0 language_server: Add `test_and_snapshot_multiple_file` (#13966) (Sysix)


## [1.16.0] - 2025-09-16

### 🚜 Refactor

- 137896a language_server: Split options for linting and formatting (#13627) (Sysix)


## [1.15.0] - 2025-09-11

### 🐛 Bug Fixes

- fb9d0f4 language_server: Don't resend diagnostic on save, when `typeAware` is disabled and run is onType (#13604) (YongSeok Jang (장용석))
- b677376 language_server: Include the diagnostic of the other linter (#13490) (Sysix)
- 5990f17 linter: Change `typescript/no-confusing-void-expression` to pedantic (#13473) (Boshen)

### 🚜 Refactor

- 671e0fd language_server: Only store one instance of a diagnostic (#13514) (Sysix)


## [1.14.0] - 2025-08-30

### 🐛 Bug Fixes

- 902b39b language_server: Split run logic for `oxlint` and `tsgolint` (#13332) (Sysix)
- 2a910c5 language_server: Disable tsgolint test on big endian (#13313) (camc314)

### 🚜 Refactor

- 6431033 linter: Move ignore logic to `LintIgnoreMatcher` (#13222) (Sysix)

### 🧪 Testing

- 33e0e8b language_server: Dont run tsgolint related test on big-edian (#13342) (Sysix)


## [1.13.0] - 2025-08-26

### 🐛 Bug Fixes

- 648e939 linter: Parse `ignorePatterns` with gitignore syntax (#13221) (Sysix)


## [1.12.0] - 2025-08-17

### 🚀 Features

- aecacae linter: Support `ignorePatterns` for nested configs (#12210) (Sysix)

### 🐛 Bug Fixes

- b0558a4 linter: Prevent unsound use of `Allocator` across threads (#13032) (overlookmotel)

### 🚜 Refactor

- b638371 language_server: Do not check twice for supported extension (#13130) (Sysix)
- 6c5b8be linter: Create `AllocatorPool` in `Runtime::new` (#13106) (overlookmotel)




## [1.11.0] - 2025-08-07

### 🚜 Refactor

- 9b9249d oxc_language_server: Remove unnecessary globset (#12861) (shulaoda)


## [1.10.0] - 2025-08-06

### 🚀 Features

- 5475075 vscode/language_server: Add `tsConfigPath` option (#12484) (Sysix)

### 🚜 Refactor

- c0e224a linter: Store `ExternalRuleId` in `OxlintOverrides` not raw names (#12502) (camc314)

### 🎨 Styling

- c15da81 codegen, formatter, linter, minifier, transformer: Re-order imports (#12725) (Copilot)




## [1.7.0] - 2025-07-16

### 🚀 Features

- d387729 linter: JS custom rules config (#12160) (camc314)

### 🐛 Bug Fixes

- 1920c6b language_server: Respect the root `.oxlintrc.json` file for `ignorePatterns` (#12171) (Sysix)
- 853d2bc linter, language_server: Correctly identify usage of `import` plugin (#12157) (overlookmotel)

### 🚜 Refactor

- 6e54645 language_server: Store `LintService` instead of `Linter` (#12016) (Sysix)
- 113cf8c linter: Move `LintServiceOptions.paths` to `LintService.with_paths` (#12015) (Sysix)


## [1.6.0] - 2025-07-07

### 🚀 Features

- f81d336 linter: Introduce `ExternalLinter` struct (#12052) (camc314)

### 🐛 Bug Fixes

- 5851d2c oxlint: Always follow symlinks; remove cli flag `--symlinks` (#12048) (Boshen)

### 🚜 Refactor

- 8d1be94 language_server: Fix todo by avoiding allocation (#12096) (Ulrich Stark)
- 72418ca linter: `RuntimeFileSystem::write_file` take `&str` (#12075) (overlookmotel)




## [1.3.0] - 2025-06-23

### 🚀 Features

- 1a54184 linter: Add fix for unused disable directive (#11708) (Sysix)
- 816ff03 linter: Read source text into the arena (#11825) (camc314)

### 🚜 Refactor

- b39d1fa linter: Output smaller spans for unused disable directives with multiple rules (#11781) (Sysix)


## [1.2.0] - 2025-06-19

### 🚀 Features

- 38dc614 oxc_linter: Reuse allocators (#11736) (camc314)
- 094b81c language_server: Add `unusedDisableDirectives` option (#11645) (Sysix)

### 🚜 Refactor

- abdbaa9 language_server: Use rule name directly from OxcCode instead of parsing out of the stringified version of OxcCode (#11714) (Nicholas Rayburn)



## [1.0.0] - 2025-06-10

## [0.18.1] - 2025-06-09

### ⚡ Performance

- 7bf25cb language_server: Transform `MessageWithPosition` to `Diagnostic` with less allocations (#11561) (Sysix)

## [0.18.0] - 2025-06-06

### Refactor

- db0b099 language_server: Convert only once uri to path when creating `ServerLinter` (#11503) (Sysix)
- ccceb52 language_server: Simplify `workspace/didChangeConfiguration` call (#11462) (Sysix)

## [0.17.0] - 2025-05-30

### Features

- 2083d33 linter/language_server: Add second editor suggestion for `react/forward-ref-uses-ref` (#11375) (Sysix)

### Bug Fixes

- 7af5bb1 oxc_language_server: Include save option for text document sync capability (#11297) (Nicholas Rayburn)

### Performance

- 0ed6c1a language_server: Use `Arc<RwLock>` instead of `Mutex` for workspace workers (#11328) (Sysix)

### Refactor

- 042a3f3 linter: Use `PossibleFixes` instead of `Option<Fix>` (#11284) (Sysix)

## [0.16.12] - 2025-05-25

- 5d9344f rust: [**BREAKING**] Clippy avoid-breaking-exported-api = false (#11088) (Boshen)

### Features

- 0c1f382 language_server: Watch for files inside `.oxlintrc.json` extends (#11226) (Sysix)
- 1675b2c language_server: Tell clients to watch for .oxlintrc.json files (#11078) (Sysix)

### Bug Fixes

- 0df5147 language_server: Correctly disable nested config search (#11173) (Sysix)

### Refactor

- 0d192e8 language_server: Introduce `ServerLinter.extended_paths` property (#11223) (Sysix)
- ff8f519 language_server: Restructure `initialized` function (#11077) (Sysix)
- 6b68de0 language_server: Add intern capability for `didChangeWatchedFiles.dynamicRegistration` (#11075) (Sysix)
- 35761ae language_server/editor: Refresh file watchers without restarting the server (didChangeConfiguration) (#11112) (Sysix)
- d5fdf17 language_server/editor: Refresh file watchers without restarting the server (didChangeWorkspaceFolders) (#11094) (Sysix)
- 9f3a14a linter: Cleanup diagnostic and docs for `eslint/no-console` (#11101) (Ulrich Stark)

## [0.16.11] - 2025-05-16

### Features

- 078bf0b language_server: Better fallback handling when passing invalid `Options` values (#10930) (Sysix)
- be7f7e1 language_server/editor: Support multi workspace folders (#10875) (Sysix)

### Bug Fixes

- 89cc21b language_server: Normalize oxlintrc config path (#10982) (Sysix)
- 39063ce linter: Reword diagnostic message for no-control-regex (#10993) (camc314)

### Refactor

- 3cc1466 language_server: New configuration structure for `initialize` and `workspace/didChangeConfiguration` (#10890) (Sysix)
- bd2ef7d language_server: Use `Arc` for `diagnostic_report_map` (#10940) (Sysix)
- bb999a3 language_server: Avoid cloning linter by taking reference in LintService (#10907) (Ulrich Stark)

## [0.16.10] - 2025-05-09

### Features

- e1bc037 language_server: Request for workspace configuration when client did not send them in `initialize` (#10789) (Sysix)
- 3bd339b language_server: Provide commands / code actions for unopened files (#10815) (Sysix)

### Bug Fixes

- f3cc3a2 language_server: Request client for configuration when no configuration is passed in `workspace/didChangeConfiguration` (#10871) (Sysix)
- 24fcb1e language_server: Return server version `initialize` response (#10810) (Sysix)

### Performance

- 00ffbc9 language_server: Do not request for configuration when all workers are ready (#10897) (Sysix)
- 96cca22 language_server: Use `simdutf8` when reading files from file system (#10814) (Sysix)

### Refactor

- 553ab5b language_server: Remove `OnceCell` from `WorkspaceWorker.root_uri` (#10898) (Sysix)
- f43fd18 language_server: Move the initialization of `ServerLinter` into a separate call (#10776) (Sysix)
- 39e0463 language_server: Move `nested_configs` to `ServerLinter` (#10775) (Sysix)
- 9ec13f6 language_server: Move `gitignore_glob` to `ServerLinter` (#10762) (Sysix)
- 3d47159 language_server: Use `IsolatedLintHandlerFileSystem` (#10830) (Sysix)
- 3d794f6 language_server: Move functions related to `ServerLinter` to `ServerLinter` (#10761) (Sysix)
- 79819cc linter: Move around some config store logic (#10861) (camc314)

## [0.16.9] - 2025-05-02

### Bug Fixes

- 46665bd langage_server: Fix initialize nested configs (#10698) (Sysix)
- 98bcd5f lsp: Incorrect quick fix offset in vue files (#10742) (camc314)

### Testing

- 9ebf3d4 language_server: Refactor tester to use WorkspaceWorker (#10730) (Sysix)
- 5a709ad language_server: Add test for `init_nested_configs` (#10728) (Sysix)
- 2615758 language_server: Fix slow test (#10659) (Alexander S.)
- fd18aaa language_server: Skip slow test (#10658) (overlookmotel)
- f6f1c5c lsp: Include fixed content in lsp snapshots (#10744) (camc314)

## [0.16.8] - 2025-04-27

### Bug Fixes

- f3eac51 language_server: Fix max integer values for range position (#10623) (Alexander S.)
- d309e07 language_server: Fix panics when paths contains specials characters like `[` or `]` (#10622) (Alexander S.)
- 91ce77a language_server: Temporary ignore tests that panic on Windows (#10583) (Yuji Sugiura)

### Refactor

- f6c6969 language_server: Make linter independent of `Backend` (#10497) (Sysix)
- db05a15 language_server: Do not request for worspace configuration when the client does not support it (#10507) (Sysix)
- 9f9e0e5 language_server: Move code actions into own file (#10479) (Sysix)

### Testing

- 9f43a58 language_server: Fix broken tests in windows (#10600) (Sysix)

## [0.16.7] - 2025-04-21

### Features

- bb8a078 language_server: Use linter runtime (#10268) (Sysix)

### Bug Fixes

- df488d4 language_server: Workspace edits as one batch when `source.fixAll.oxc` is the context (#10428) (Sysix)

### Performance

- 21f3175 langage_server: Prebuild `IsolatedLintHandler` (#10406) (Sysix)

### Refactor

- 2935103 language_server: Report info as warning instead of error when falling back to default config (#10517) (Sysix)

## [0.16.6] - 2025-04-14

### Features

- 0370363 language_server: Switch `tower-lsp` to `tower-lsp-server` (#10298) (Boshen)

### Bug Fixes

- 664342b language_server: Diable nested configuration when config path is provided (#10385) (Sysix)

### Performance

- e0057c3 language_server: Only restart internal linter once when multiple config changes detected (#10256) (Sysix)

### Refactor

- a95ba40 language_server: Make server more error resistance by falling back to default config (#10257) (Sysix)

### Testing

- 4a6bb21 language_server: Add test for `import` plugin integration (#10364) (Sysix)

## [0.16.5] - 2025-04-07

### Features

- 2f6810a editor: Add named fixes for code actions (#10203) (camchenry)
- 32b9d1e language_server: Add `fix_kind` flag (#10226) (Sysix)
- dab1bd8 language_server: Search for nested configurations by initialization (#10120) (Sysix)

### Bug Fixes

- d691701 various: Unwrap `Result` of `write!` macro (#10228) (overlookmotel)

### Performance

- b34e876 linter: Avoid cloning filters by refactoring functions to take references (#10247) (Ulrich Stark)

### Styling

- 66a0001 all: Remove unnecessary semi-colons (#10198) (overlookmotel)

## [0.16.4] - 2025-04-01

- da6336c language_server: [**BREAKING**] Remove `enable` configuration, the client should shutdown the server instead (#9990) (Sysix)

### Bug Fixes

- 07f2a25 editor: Enable regex parsing in language server (#10035) (camchenry)
- fcf7702 language_server: Start from a default oxlint configuration + SafeFix for nested configuration (#10043) (Sysix)

### Refactor

- c0e5251 language_server: Set `null` as a default value for `configPath` (#10047) (Sysix)
- d8e49a1 linter: Compute lintable extensions at compile time (#10090) (camchenry)

## [0.16.3] - 2025-03-25

### Bug Fixes

- 2d7b0cf editor: Re-add tester and prevent empty inverted diagnostics  (#9991) (camchenry)

### Refactor

- ad06194 linter: Add fixer for `typescript-eslint/no-non-null-asserted-optional-chain` (#9993) (camchenry)

### Testing

- 71dce1f editor: Add end-to-end tests for linter output (#9979) (Cam McHenry)
- c2f1be0 editor: Add tests for `offset_to_position` (#9978) (camchenry)

## [0.16.1] - 2025-03-20

- b3ce925 data_structures: [**BREAKING**] Put all parts behind features (#9849) (overlookmotel)

### Features

- 0973356 editor: Support nested configs (#9743) (Nicholas Rayburn)
- ea7e3f0 oxc_language_server: Support nested configs (#9739) (Nicholas Rayburn)

### Documentation

- 8bc70b3 language_server: Tell about Initialization options + didChangeWatchedFiles for nested configuration (#9876) (Alexander S.)

## [0.15.14] - 2025-03-11

### Features

- fc74849 linter: Inherit `rules` via the extended config files (#9308) (camchenry)

## [0.15.12] - 2025-02-23

### Bug Fixes

- 94bd2d8 language_server: Fix `clippy::significant_drop_in_scrutinee` warning (#9234) (Boshen)

### Refactor

- 6ec81ef language_server: Use `papaya` instead of `dashmap` (#9220) (Boshen)

