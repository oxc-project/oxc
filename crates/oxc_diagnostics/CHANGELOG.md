# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.48.2] - 2025-02-02

### Testing

- ad35e82 linter: Use snapshot testing instead of LintResult (#8711) (Sysix)

## [0.48.1] - 2025-01-26

### Refactor

- 10e5920 linter: Move finishing default diagnostic message to `GraphicalReporter` (#8683) (Sysix)

## [0.48.0] - 2025-01-24

### Features

- 4ae568e linter: Add DiagnosticResult to the Reporters for receiving a sub part result (#8666) (Alexander S.)

### Bug Fixes

- 40316af linter: Fix github `endColumn` output (#8647) (Alexander S.)

## [0.47.0] - 2025-01-18

### Refactor

- b4c87e2 linter: Move DiagnosticsReporters to oxlint (#8454) (Alexander S.)

## [0.42.0] - 2024-12-18

### Styling

- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.40.0] - 2024-12-10

### Features

- 85eec3c napi/transform,napi/parser: Return structured error object (#7724) (Boshen)

## [0.37.0] - 2024-11-21

### Features

- 2268a0e linter: Support `overrides` config field (#6974) (DonIsaac)

## [0.36.0] - 2024-11-09

### Features

- 22898c8 transformer: Warn BigInt when targeting < ES2020 (#7184) (Boshen)

## [0.35.0] - 2024-11-04

### Features

- 6d97af4 rust: Use `oxc-miette` (#6938) (Boshen)

## [0.30.2] - 2024-09-27

### Bug Fixes

- a88504c diagnostics: Check for terminal when displaying links (#6018) (Boshen)

## [0.30.1] - 2024-09-24

### Performance

- 2b17003 linter, prettier, diagnostics: Use `FxHashMap` instead of `std::collections::HashMap` (#5993) (camchenry)

## [0.30.0] - 2024-09-23

### Documentation

- 83ca7f5 diagnostics: Fully document `oxc_diagnostics` (#5865) (DonIsaac)

### Refactor

- 6dd6f7c ast: Change `Comment` struct (#5783) (Boshen)

## [0.27.0] - 2024-09-06

### Features

- 91b39c4 oxc_diagnostic: Impl DerefMut for OxcDiagnostic (#5474) (IWANABETHATGUY)

### Bug Fixes

- fce549e diagnostics: Ignore `Interrupted` and `BrokenPipe` errors while printing (#5526) (Boshen)

## [0.26.0] - 2024-09-03

### Features

- 9c22ce9 linter: Add hyperlinks to diagnostic messages (#5318) (DonIsaac)

### Bug Fixes

- ff7fa98 diagnostics: Improve "file is too long to fit on the screen" (#5120) (Boshen)

### Refactor

- cd63336 diagnostic: Change how diagnostic codes are rendered (#5317) (DonIsaac)

## [0.22.0] - 2024-07-23

### Features
- 6068e6b Add error codes to OxcDiagnostic (#4334) (DonIsaac)

### Refactor

- 7a75e0f linter: Use diagnostic codes in lint rules (#4349) (DonIsaac)
- a2eabe1 parser: Use error codes for ts diagnostics (#4335) (DonIsaac)

## [0.20.0] - 2024-07-11

### Performance

- ddfa343 diagnostic: Use `Cow<'static, str>` over `String` (#4175) (DonIsaac)

## [0.16.1] - 2024-06-29

### Refactor

- 2705df9 linter: Improve diagnostic labeling (#3960) (DonIsaac)

## [0.16.0] - 2024-06-26

### Performance

- 92c21b2 diagnostics: Optimize string-buffer reallocations (#3897) (Luca Bruno)

## [0.14.0] - 2024-06-12

### Bug Fixes

- e6ad3fb diagnostics: Do not print ansi color codes in non-TTYs (#3624) (Boshen)

## [0.13.1] - 2024-05-22

### Features

- 17f4b19 cli: Add `--silent` to disable all diagnostics printing (#3338) (Boshen)

### Refactor

- c9d84af diagnostics: S/warning/warn (Boshen)

## [0.13.0] - 2024-05-14

### Features

- ed3fa39 linter: Add `--format github` for github check annotation (#3191) (Boshen)

### Bug Fixes

- b86ef7d diagnostics: Need to escape strings for --format github (Boshen)

### Refactor

- dbde5b3 diagnostics: Remove export of `miette` (Boshen)
- 551632a diagnostics: Remove thiserror (Boshen)
- 312f74b diagnostics: S/OxcDiagnostic::new/OxcDiagnostic::error (Boshen)
- 82bd97d diagnostics: Use a trait to implement the reporters (#3190) (Boshen)
- f6f7adc linter,diagnostic: One diagnostic struct to eliminate monomorphization of generic types (#3235) (Boshen)
- 2064ae9 parser,diagnostic: One diagnostic struct to eliminate monomorphization of generic types (#3214) (Boshen)- 893af23 Clean up more diagnostics usages (Boshen)

## [0.12.5] - 2024-04-22

### Features

- ee1c0e5 cli: Implement `--format checkstyle` (#3044) (Boshen)
- 4425b96 cli: Implement `--format unix` (#3039) (Boshen)

## [0.12.3] - 2024-04-11

### Bug Fixes

- 6eba02f cli: If format is json do not print summary information (#2899) (#2925) (Kalven Schraut)

## [0.10.0] - 2024-03-14

### Features
- 265b2fb Miette v7 (#2465) (Boshen)

## [0.8.0] - 2024-02-26

### Features

- 195d76e cli,diagnostics: Add json reporter (#2451) (Boshen)
- d0d0d9d diagnostics: Implement json reporter (#2452) (Boshen)

## [0.6.0] - 2024-02-03

### Refactor
- 87b9978 Move all miette usages to `oxc_diagnostics` (Boshen)

## [0.5.0] - 2024-01-12

### Bug Fixes

- ea22d3c diagnostics: Always print without considering the `--max-warnings` option (#1996) (Boshen)

## [0.4.0] - 2023-12-08

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.2.0] - 2023-09-14

### Bug Fixes

- de7735d cli: Fix race condition when resolving paths (Boshen)
- ba8ef7b deps: Use one version of `textwrap` (Boshen)

### Refactor

- 2751240 cli,diagnostics: Implement DiagnosticService (#762) (Boshen)
- a9a6bb8 cli,linter: Move path processing logic from cli to linter (#766) (Boshen)

