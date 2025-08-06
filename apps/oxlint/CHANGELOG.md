# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [1.10.0] - 2025-08-06

### 🚀 Features

- 9b35600 linter/jsx-a11y: Add support for mapped attributes in label association checks (#12805) (camc314)

### 🐛 Bug Fixes

- 45206dd linter: Apply fix span offset after fixing the section source text (#12758) (Sysix)

### 🚜 Refactor

- 030e397 linter: Simplify parsing CLI args (#12802) (overlookmotel)
- c0e224a linter: Store `ExternalRuleId` in `OxlintOverrides` not raw names (#12502) (camc314)

### ⚡ Performance

- 693673b linter: Reduce iterations when collecting directories for nested configs (#12329) (overlookmotel)

### 🎨 Styling

- c15da81 codegen, formatter, linter, minifier, transformer: Re-order imports (#12725) (Copilot)

### 🧪 Testing

- d8ccff7 oxlint: Add `Tester::test_fix` mehod (#12754) (Sysix)


## [1.9.0] - 2025-07-29

### 🚜 Refactor

- 543fd53 napi/oxlint: Rename `run` to `lintFile` (#12567) (overlookmotel)
- 491c401 linter: Remove `#[must_use]` from `LintService::with_*` methods (#12560) (overlookmotel)
- d44b0ac linter: Remove `Runner` trait (#12559) (overlookmotel)
- bea652f linter: Add `vue` and `regex` to `BuiltinLintPlugins` (#12542) (Sysix)
- 5c33fc7 diagnostics: Implement `Eq` and `Ord` for `InfoPosition` (#12505) (overlookmotel)
- 7a0da04 diagnostics: Remove Option wrapper from MPSC channel and sender field (#12467) (camc314)

### 🧪 Testing

- d31adcf linter: Improve sorting diagnostics (#12504) (overlookmotel)


## [1.8.0] - 2025-07-22

### 🐛 Bug Fixes

- 46e33d5 linter: Improve error handling in config file lookup (#12391) (camc314)


## [1.7.0] - 2025-07-16

### 🚀 Features

- 5e428a4 linter/eslint-plugin-next: No-html-link-for-pages rule addition (#12194) (Gabriel Díaz Aguilera)
- c551b8f linter: Report diagnostics from custom plugins (#12219) (camc314)
- d387729 linter: JS custom rules config (#12160) (camc314)
- 152e59d napi/oxlint: Read source text into start of allocator (#12122) (overlookmotel)
- d4ebd14 linter: Add `oxlint2`/`disable_oxlint2` feature flags (#12130) (camc314)
- a4dae73 linter: Introduce `LintPlugins` to store builtin + custom plugins (#12117) (camc314)

### 🐛 Bug Fixes

- 9720774 linter: Report implicit config parse errors (#12260) (Simon Buchan)
- 853d2bc linter, language_server: Correctly identify usage of `import` plugin (#12157) (overlookmotel)

### 🚜 Refactor

- 6e54645 language_server: Store `LintService` instead of `Linter` (#12016) (Sysix)
- 113cf8c linter: Move `LintServiceOptions.paths` to `LintService.with_paths` (#12015) (Sysix)
- acfac68 oxlint: Adjust ignore patterns by counting bytes instead of chars (#12209) (Sysix)
- 1d2eaca oxlint2: Introduce `force_test_reporter` feature for consistent graphical outputs (#12133) (camc314)
- f7c675d linter: Rename `LintPlugins` to `BuiltinLintPlugins` (#12116) (camc314)

### 🧪 Testing

- d1194e8 oxlint: Ignore test on windows (#12262) (camc314)


## [1.6.0] - 2025-07-07

### 🚀 Features

- f81d336 linter: Introduce `ExternalLinter` struct (#12052) (camc314)

### 🐛 Bug Fixes

- 5851d2c oxlint: Always follow symlinks; remove cli flag `--symlinks` (#12048) (Boshen)

### 🚜 Refactor

- 2f7cbda linter: Move napi bindings out of oxc_linter (#12072) (camc314)
- 9254252 linter: Move code (#12071) (overlookmotel)


## [1.5.0] - 2025-07-02

### 🐛 Bug Fixes

- 4b2c658 oxlint: Make `--version` exit code be `0` (#11986) (camc314)


## [1.4.0] - 2025-06-30

### 🚀 Features

- 9b19b40 napi: Add basic oxlint napi bindings (#11877) (camc314)
- f102cb1 linter: Add `import/prefer-default-export` rule (#11891) (yefan)

### 🐛 Bug Fixes

- d991fed linter: Fix `jsx-a11y/label-has-associated-control` default values (#11832) (Sysix)

### 🚜 Refactor

- 2cf9fa3 linter: Derive debug for `extensions` (#11938) (camc314)


## [1.3.0] - 2025-06-23

### 🚜 Refactor

- b39d1fa linter: Output smaller spans for unused disable directives with multiple rules (#11781) (Sysix)


## [1.2.0] - 2025-06-19

### 🚀 Features

- 38dc614 oxc_linter: Reuse allocators (#11736) (camc314)


## [1.1.0] - 2025-06-12

### 🚀 Features

- 1181018 linter: Add eslint/no-extra-bind rule (#11588) (yefan)


## [1.0.0] - 2025-06-10

## [0.18.0] - 2025-06-06

- bd9dd88 linter: [**BREAKING**] Add more info to json reporter (#11524) (camc314)

### Features


### Bug Fixes

- 0946dac linter: Correctly inherit categories when plugins are enabled (#11353) (Cameron)

## [0.17.0] - 2025-05-30

- ead5309 linter: [**BREAKING**] Remove react from default plugin set (#11382) (camc314)

### Bug Fixes

- f6424dd linter: Reflect react plugin is disabled by default in cli (#11397) (camc314)

### Documentation

- cd354d4 oxlint: Remove incorrect doc comment (#11326) (camc314)

### Testing

- c4f64aa linter: Explicitly disable correctness for clarity (#11327) (camc314)

## [0.16.12] - 2025-05-25

- 5d9344f rust: [**BREAKING**] Clippy avoid-breaking-exported-api = false (#11088) (Boshen)

### Features

- 12b0917 linter: Auto-generate docs for rule configs (#10629) (DonIsaac)

### Bug Fixes

- e8470d9 linter: Delay merging of oxlintrc configs (#10835) (camc314)

### Refactor

- 9f3a14a linter: Cleanup diagnostic and docs for `eslint/no-console` (#11101) (Ulrich Stark)

## [0.16.11] - 2025-05-16

- 4e5c73b span: [**BREAKING**] `SourceType::from_path(".js")` return js instead of jsx (#11038) (Boshen)

### Features

- 466c24a linter: Add gitlab reporter output format (#10927) (Connor Pearson)

### Bug Fixes

- c52a9ba linter: Fix plugins inside overrides not being applied (#11057) (camc314)
- b12bd48 linter: Fix rule config not being correctly applied (#11055) (camc314)
- 0961296 linter: Add `gitlab` to linter `--help` docs (#10932) (camc314)
- 584d8b9 napi: Enable mimalloc `no_opt_arch` feature on linux aarch64 (#11053) (Boshen)

### Refactor

- bb999a3 language_server: Avoid cloning linter by taking reference in LintService (#10907) (Ulrich Stark)

## [0.16.10] - 2025-05-09

### Features

- 4c62348 linter: Regex/no-useless-backreference (#10773) (camc314)

### Refactor

- 79819cc linter: Move around some config store logic (#10861) (camc314)
- e132aba linter: Extract nested config searching to a fn (#10860) (camc314)
- efb4fb8 oxlint: Avoid result unwrap (#10836) (camc314)

## [0.16.9] - 2025-05-02

### Features

- 63f02a8 linter: Add react/forward_ref_uses_ref (#10506) (x6eull)
- eac205f linter: Add unicorn/consistent-assert rule (#10653) (Shota Kitahara)

### Bug Fixes

- e7c2b32 linter: Move `consistent-assert` to `pedantic` (#10665) (camc314)

## [0.16.8] - 2025-04-27

### Bug Fixes

- 723b4c6 linter: Cross_module of LintService not being enabled despite enabled import plugin (#10597) (Ulrich Stark)
- 9a02066 oxlint: Current dir as arg (#9382) (Ben Jones)

## [0.16.7] - 2025-04-21

### Bug Fixes

- 4e1f536 linter: Config path resolution when path contains '..' syntax (#10367) (Florian Bopp)

### Refactor

- 5ab4d40 linter: Simplify error handling (#10404) (camchenry)

## [0.16.6] - 2025-04-14

### Bug Fixes

- 9aaba69 linter: Nested configuration directory resolution (#10157) (Sub)

### Testing

- aa6ccd2 oxlint: Add test for nested and extended configuration with import plugin (#10372) (Sysix)

## [0.16.5] - 2025-04-07

### Features

- 2f6810a editor: Add named fixes for code actions (#10203) (camchenry)

### Bug Fixes

- f2eff56 linter: Fix `rule_id` for some diagnostics formats (#10251) (Alexander S.)
- d691701 various: Unwrap `Result` of `write!` macro (#10228) (overlookmotel)

### Performance

- b34e876 linter: Avoid cloning filters by refactoring functions to take references (#10247) (Ulrich Stark)

### Styling

- 66a0001 all: Remove unnecessary semi-colons (#10198) (overlookmotel)

## [0.16.4] - 2025-04-01

### Features

- 370266c semantic: Check redeclaration of variable declaration and function declaration in the block scope (#10074) (Dunqing)

### Bug Fixes

- 2c80858 linter: Enable multi-file analysis for nested configs (#10089) (camchenry)

### Refactor

- d8e49a1 linter: Compute lintable extensions at compile time (#10090) (camchenry)

## [0.16.3] - 2025-03-25

### Refactor

- 6432707 rust: Use `lazy-regex` (#10004) (Boshen)

## [0.16.2] - 2025-03-21

### Bug Fixes

- f649fb3 linter: Reclassify `unicorn/no-document-cookie` as restriction (#9933) (camchenry)

### Documentation

- 46a12c6 linter: Tell about junit `--format` options (#9931) (Sysix)

## [0.16.1] - 2025-03-20

### Features

- 8e3d9be linter: Support `--report-unused-disable-directive` (#9223) (1zumii)

### Bug Fixes

- e6f7c74 linter: Import and fix tests for typescript::no_unnecessary_parameter_property_assignment (#9720) (Ulrich Stark)

### Performance

- 84fa538 minify: Use mimalloc-safe to replace mimalloc (#9810) (LongYinan)

### Refactor

- b34cf94 oxlint: Remove `jemallocator` (#9823) (Boshen)

## [0.16.0] - 2025-03-16

- 225e266 linter: [**BREAKING**] Enable `--experimental-nested-config` by default and add `--disable-nested-config` option (#9760) (camchenry)

### Features


### Bug Fixes

- 22f18ac linter: Improve `jsx-a11y/anchor-ambiguous-text` diagnostic message (#9789) (1zumii)

## [0.15.15] - 2025-03-12

### Features

- 474a57b linter: A new multi-file analysis runtime (#9383) (branchseer)

### Bug Fixes

- ab594f1 linter: Turn oxc/no-redundant-constructor-init into typescript/no-unnecessary-parameter-property-assignment (#9618) (Uli)

## [0.15.14] - 2025-03-11

### Features

- 41f32ea linter: Allow adding more overrides via `extends` configs (#9475) (camchenry)
- fb7cf10 linter: Allowing `plugins` to be extended with `extends` (#9473) (camchenry)
- fc74849 linter: Inherit `rules` via the extended config files (#9308) (camchenry)

### Bug Fixes

- 4ca62ab linter: Output right file line and column for `.vue`, `.astro` and `.svelte` files (#9484) (Sysix)
- 3105159 linter: Do not output number of rules with nested configs (#9476) (camchenry)
- 5ecda01 linter: Support nested extending (#9472) (camchenry)

### Refactor

- 62bffed rust: Allow a few annoying clippy rules (#9588) (Boshen)

### Testing

- 934a387 linter: Remove test dependency on oxlint (#9513) (camchenry)

## [0.15.13] - 2025-03-04

### Features

- 4ad328b linter: Add oxc/no-redundant-constructor-init (#9299) (Ben Jones)

## [0.15.12] - 2025-02-23

### Features

- 9bc3017 linter: Add support for nested config files (#9153) (camchenry)
- cc8dd48 linter: Add unicorn/no-invalid-fetch-options rule (#9212) (Mikhail Baev)
- af13b1b linter: Promote `eslint/no-eval` to `correctness` (#9231) (dalaoshu)
- 542bbd7 linter: Support `import-x` plugin name (#9074) (Sysix)
- cded0ad oxlint: Add `--experimental-nested-config` option (#9152) (camchenry)

### Bug Fixes

- 4ed9d76 linter: Do not use nested configs with `--config` option (#9155) (camchenry)

### Refactor

- 63bb214 oxc: Apply `clippy::redundant_clone` (#9252) (Boshen)
- 9f36181 rust: Apply `cllippy::nursery` rules (#9232) (Boshen)

### Testing

- e49c92d linter: Ensure CLI filters take precedence over nested configs (#9156) (camchenry)

## [0.15.11] - 2025-02-16

### Features

- 5d508a4 linter: Support `env` and `globals` in `overrides` configuration (#8915) (Sysix)

### Bug Fixes

- 47c1649 linter: Output line/column for `--format=stylish` instead of offset + length (#9136) (Sysix)

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

## [0.15.10] - 2025-02-06

### Features

- 7e8568b linter: Junit reporter (#8756) (Tapan Prakash)

### Bug Fixes

- baf3e4e linter: Correctly replace rule severity with duplicate rule name configurations (#8840) (dalaoshu)

## [0.15.9] - 2025-02-01

### Bug Fixes

- 8ce21d1 linter: Can't disable `no-nested-ternary` rule anymore (#8600) (dalaoshu)
- e929f26 linter: Output `LintCommandInfo` for `CliRunResult::LintNoFilesFound` (#8714) (Sysix)
- 9cc9d5f linter: `ignorePatterns` does not work when files are provided as command arguments (#8590) (dalaoshu)

### Refactor

- 194a5ff linter: Remove `LintResult` (#8712) (Sysix)
- 4a2f2a9 linter: Move default `all_rules` output to trait (#8710) (Sysix)
- 741fb40 linter: Move stdout outside LintRunner (#8694) (Sysix)
- 10e5920 linter: Move finishing default diagnostic message to `GraphicalReporter` (#8683) (Sysix)
- 9731c56 oxlint: Move output from `CliRunResult::InvalidOption` to outside and use more Enums for different invalid options (#8778) (Sysix)
- fe45bee oxlint: Create different `CliRunResult` instead of passing `ExitCode` to it (#8777) (Sysix)
- 2378fef oxlint: Move ConfigFileInit output outside CliRunResult, exit code 1 when it fails (#8776) (Sysix)
- f4cecb5 oxlint: Remove unused `CliRunResult::PathNotFound` (#8775) (Sysix)

### Testing

- ad35e82 linter: Use snapshot testing instead of LintResult (#8711) (Sysix)
- bf895eb linter: Add diagnostic format test snapshots (#8696) (Alexander S.)
- 34d3d72 linter: Add snapshot tester for cli (#8695) (Sysix)
- 0bf2bcf oxlint: Test two real rules with same name but from different plugins (#8821) (dalaoshu)
- 2b83b71 oxlint: Improve disabling "no-nested-ternary" tests (#8814) (Alexander S.)
- 45648e7 oxlint: Fix InvalidOptionTsConfig tests for windows (#8791) (Alexander S.)
- 48bfed9 oxlint: Ignore windows path mismatch (Boshen)
- 6f4a023 oxlint: Remove "--print-config" test (#8792) (Sysix)
- 55c2025 oxlint: Add `CliRunResult` to snapshot (#8780) (Sysix)

## [0.15.8] - 2025-01-24

### Features

- 4ae568e linter: Add DiagnosticResult to the Reporters for receiving a sub part result (#8666) (Alexander S.)
- 8a0eb2a oxlint: Add stylish formatter (#8607) (Andrew Powell)

### Bug Fixes

- 40316af linter: Fix github `endColumn` output (#8647) (Alexander S.)
- dc912fa linter: Added missing $schema property to default config (#8625) (Tapan Prakash)

## [0.15.7] - 2025-01-19

### Features

- 4ac2e99 oxlint: Implement `--init` cli option (#8453) (Tapan Prakash)

### Refactor

- b4c87e2 linter: Move DiagnosticsReporters to oxlint (#8454) (Alexander S.)

## [0.15.6] - 2025-01-13

### Refactor

- 43ed3e1 linter: Add output formatter (#8436) (Alexander S.)
- 4e05e66 linter: Remove glob for windows (#8390) (Alexander S.)
- 3c534ae linter: Refactor `LintBuilder` to prep for nested configs (#8034) (camc314)

## [0.15.5] - 2025-01-02

### Bug Fixes

- 2b14a6f linter: Fix `ignorePattern` config for windows  (#8214) (Alexander S.)

### Testing

- cb709c9 linter: Fix some oxlint tests on windows (#8204) (Cameron)

## [0.15.4] - 2024-12-30

### Bug Fixes

- f3050d4 linter: Exclude svelte files from `no_unused_vars` rule (#8170) (Yuichiro Yamashita)

### Refactor

- 6da0b21 oxlint: Remove unused `git.rs` (#7990) (Boshen)
- 58e7777 oxlint: Remove extra if check in `Walkdir` (#7989) (Boshen)

## [0.15.3] - 2024-12-17

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.15.0] - 2024-12-10

- 39b9c5d linter: [**BREAKING**] Remove unmaintained security plugin (#7773) (Boshen)

### Features


## [0.14.1] - 2024-12-06

### Features

- 275d625 linter: Output rules to json array (#7574) (camc314)

### Bug Fixes

- 9761e94 apps/oxlint: Incorrect matching in `.oxlintignore` (#7566) (dalaoshu)
- 29db060 linter: Detect typescript eslint alias rules (#7622) (Alexander S.)
- 810671a linter: Detect vitest jest alias rules (#7567) (Alexander S.)

## [0.14.0] - 2024-12-01

### Features

- 32f860d linter: Add support for ignorePatterns property within config file (#7092) (Nicholas Rayburn)

## [0.13.1] - 2024-11-23

### Features

- 9558087 oxlint: Auto detect config file in CLI (#7348) (Alexander S.)

### Bug Fixes

- 8507464 linter: Hanging when source has syntax/is flow (#7432) (Cameron)
- e88cf1b linter: Make `overrides` globs relative to config path (#7407) (camchenry)

## [0.13.0] - 2024-11-21

- 878189c parser,linter: [**BREAKING**] Add `ParserReturn::is_flow_language`; linter ignore flow error (#7373) (Boshen)

### Features


## [0.12.0] - 2024-11-20

### Features

- 2268a0e linter: Support `overrides` config field (#6974) (DonIsaac)
- d3a0119 oxlint: Add `cwd` property to `LintRunner` (#7352) (Alexander S.)

### Bug Fixes

- df5c535 linter: Revert unmatched rule error (#7257) (Cameron A McHenry)

## [0.11.0] - 2024-11-03

- 1f2a6c6 linter: [**BREAKING**] Report unmatched rules with error exit code (#7027) (camchenry)

### Features

- 2184588 linter: Do not bail for unmatched rules yet (#7093) (Boshen)

### Bug Fixes

- 38d1f78 linter: Remove confusing help text for now (#7081) (Cam McHenry)

### Refactor

- a8dc75d linter: Remove unused CLI result types (#7088) (camchenry)

## [0.10.3] - 2024-10-26

### Features

- 0acca58 linter: Support `--print-config all` to print config file for project (#6579) (mysteryven)

## [0.10.2] - 2024-10-22

### Refactor

- 6ffdcc0 oxlint: Lint/mod.rs -> lint.rs (#6746) (Boshen)

### Testing

- b03cec6 oxlint: Add `--fix` test case (#6747) (Boshen)

## [0.10.1] - 2024-10-21

### Refactor

- d6609e9 linter: Use `run_on_jest_node` for existing lint rules (#6722) (camchenry)

## [0.10.0] - 2024-10-18

- 80266d8 linter: [**BREAKING**] Support plugins in oxlint config files (#6088) (DonIsaac)

### Features


## [0.9.10] - 2024-10-07

### Bug Fixes

- 9e9808b linter: Fix regression when parsing ts in vue files (#6336) (Boshen)

### Refactor

- ea908f7 linter: Consolidate file loading logic (#6130) (DonIsaac)

## [0.9.7] - 2024-09-23

### Features

- d24985e linter: Add `oxc-security/api-keys` (#5906) (DonIsaac)

## [0.9.6] - 2024-09-18

### Refactor

- 026ee6a linter: Decouple module resolution from import plugin (#5829) (dalaoshu)

## [0.9.4] - 2024-09-12

### Refactor

- 9e9435f linter: Add `LintFilter` (#5685) (DonIsaac)
- 5ae9b48 linter: Start internal/external split of `OxlintOptions` (#5659) (DonIsaac)
- bac03e3 linter: Make fields of `LintServiceOptions` private (#5593) (DonIsaac)
- 20d0068 oxlint: Move cli-related exports to `cli` module (#5139) (DonIsaac)

## [0.9.3] - 2024-09-07

### Features

- 4473779 linter/node: Implement no-exports-assign (#5370) (dalaoshu)

### Styling
- d8b29e7 Add trailing line breaks to JSON files (#5544) (overlookmotel)

## [0.9.0] - 2024-08-26

- b894d3b linter: [**BREAKING**] Make `no-unused-vars` correctness (#5081) (DonIsaac)

### Features


## [0.7.2] - 2024-08-15

### Documentation

- 955a4b4 oxlint: Improve cli doc regarding fix and `-D all` (Boshen)

## [0.7.0] - 2024-08-05

### Features

- b952942 linter: Add eslint/no-unused-vars (⭐ attempt 3.2) (#4445) (DonIsaac)
- 7afa1f0 linter: Support suggestions and dangerous fixes (#4223) (DonIsaac)

### Bug Fixes

- fe1356d linter: Change no-unused-vars to nursery (#4588) (DonIsaac)
- 72337b1 linter: Change typescript-eslint/no-namespace to restriction (#4539) (Don Isaac)
- 732f4e2 linter: Fix `oxlint` allocator cfg (#4527) (overlookmotel)

## [0.6.1] - 2024-07-17

### Features

- 1f8968a linter: Add eslint-plugin-promise rules: avoid-new, no-new-statics, params-names (#4293) (Jelle van der Waa)

## [0.5.1] - 2024-06-29

### Bug Fixes

- 750cb43 oxlint: Gate custom allocators by feature flag (#3945) (Luca Bruno)

## [0.5.0] - 2024-06-27

### Features

- 328445b linter: Support `vitest/no-disabled-tests` (#3717) (mysteryven)

### Bug Fixes

- 5902331 oxlint: Properly report error (#3889) (Luca Bruno)

## [0.4.2] - 2024-05-28

### Refactor

- 21505e8 cli: Move crates/oxc_cli to apps/oxlint (#3413) (Boshen)

