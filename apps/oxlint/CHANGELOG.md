# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

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

