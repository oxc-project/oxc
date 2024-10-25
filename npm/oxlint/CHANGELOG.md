# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.10.0] - 2024-10-18

### Features

- 6e3224d linter: Configure by category in config files (#6120) (DonIsaac)

## [0.9.9] - 2024-09-27

### Bug Fixes

- 01b9c4b npm/oxlint: Make bin/oxc_language_server an executable (#6066) (Boshen)

## [0.9.7] - 2024-09-23

### Refactor

- ba7b01f linter: Add `LinterBuilder` (#5714) (DonIsaac)

## [0.9.6] - 2024-09-18

### Refactor

- a438743 linter: Move `OxlintConfig` to `Oxlintrc` (#5707) (DonIsaac)

## [0.9.4] - 2024-09-12

### Features

- 023c160 linter: Impl `Serialize` for `OxlintConfig` (#5594) (DonIsaac)

## [0.9.3] - 2024-09-07

### Styling
- 694f032 Add trailing line breaks to `package.json` files (#5542) (overlookmotel)

## [0.8.0] - 2024-08-23

### Features

- a0effab linter: Support more flexible config.globals values (#4990) (Don Isaac)

## [0.7.2] - 2024-08-15

### Features

- 4d28d03 task/website: Support render `subschemas.all_of` (#4800) (mysteryven)

## [0.7.1] - 2024-08-12

### Features

- cc922f4 vscode: Provide config's schema to oxlint config files (#4826) (Don Isaac)

## [0.7.0] - 2024-08-05

### Bug Fixes

- 0fba738 npm: SyntaxError caused by optional chaining in low version node (#4650) (heygsc)

## [0.6.0] - 2024-07-11

### Features

- cc58614 linter: Better schemas for allow/warn/deny (#4150) (DonIsaac)

## [0.4.2] - 2024-05-28

### Bug Fixes

- 19bb1c0 website: Hack `schemars` to render code snippet in markdown (#3417) (Boshen)

## [0.3.2] - 2024-05-04

### Bug Fixes

- dcda1f6 cli: Update `--format` documentation (#3118) (Vasilii A)

## [0.3.0] - 2024-04-22

### Refactor

- 5241e1e cli: Improve `--help` documentation (Boshen)

## [0.2.8] - 2024-02-06

### Features

- 839e7c5 napi/parser: Add more linux-musl targets (Boshen)

## [0.2.7] - 2024-02-03

### Features

- 0ae28dd npm/oxlint: Display target triple when error is thrown (#2259) (Boshen)

## [0.2.4] - 2024-01-23

### Bug Fixes

- 382a187 npm: Fix bin script for musl / gnu (Boshen)

## [0.2.3] - 2024-01-23

### Features
- 20a34b5 Introduce --react-perf-plugin CLI flag, update rules to correctness (#2119) (Hulk)

## [0.0.17] - 2023-11-09

### Features

- d82ba5b cli: Run oxlint with no file arguments (#1201) (Boshen)

## [0.0.9] - 2023-08-21

### Bug Fixes

- e14dd06 npm: Add package.repository and other fields according to provernance (Boshen)
- f7d2675 npm: Fix github link according to provernance (Boshen)

