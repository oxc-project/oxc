# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

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

