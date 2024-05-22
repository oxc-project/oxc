# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.13.1] - 2024-05-22

### Features

- Add `--silent` to disable all diagnostics printing (#3338)

### Refactor

- S/warning/warn

## [0.13.0] - 2024-05-14

### Features

- Add `--format github` for github check annotation (#3191)

### Bug Fixes

- Need to escape strings for --format github

### Refactor

- Clean up more diagnostics usages
- Remove export of `miette`
- Remove thiserror
- S/OxcDiagnostic::new/OxcDiagnostic::error
- One diagnostic struct to eliminate monomorphization of generic types (#3235)
- One diagnostic struct to eliminate monomorphization of generic types (#3214)
- Use a trait to implement the reporters (#3190)

## [0.12.5] - 2024-04-22

### Features

- Implement `--format checkstyle` (#3044)
- Implement `--format unix` (#3039)

## [0.12.3] - 2024-04-11

### Bug Fixes

- If format is json do not print summary information (#2899) (#2925)

## [0.10.0] - 2024-03-14

### Features

- Miette v7 (#2465)

## [0.8.0] - 2024-02-26

### Features

- Implement json reporter (#2452)
- Add json reporter (#2451)

## [0.6.0] - 2024-02-03

### Refactor

- Move all miette usages to `oxc_diagnostics`

## [0.5.0] - 2024-01-12

### Bug Fixes

- Always print without considering the `--max-warnings` option (#1996)

## [0.4.0] - 2023-12-08

### Refactor

- Move to workspace lint table (#1444)

## [0.2.0] - 2023-09-14

### Bug Fixes

- Use one version of `textwrap`
- Fix race condition when resolving paths

### Refactor

- Move path processing logic from cli to linter (#766)
- Implement DiagnosticService (#762)

