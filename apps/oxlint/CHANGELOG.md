# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

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

