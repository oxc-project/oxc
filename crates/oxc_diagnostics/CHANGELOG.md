# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.13.1] - 2024-05-22

### Features

* cli: add `--silent` to disable all diagnostics printing (#3338)

### Refactor

* diagnostics: s/warning/warn

## [0.13.0] - 2024-05-14

### Features

* linter: add `--format github` for github check annotation (#3191)

### Bug Fixes

* diagnostics: need to escape strings for --format github

### Refactor

* diagnostics: remove export of `miette`
* diagnostics: remove thiserror
* diagnostics: s/OxcDiagnostic::new/OxcDiagnostic::error
* diagnostics: use a trait to implement the reporters (#3190)
* linter,diagnostic: one diagnostic struct to eliminate monomorphization of generic types (#3235)
* parser,diagnostic: one diagnostic struct to eliminate monomorphization of generic types (#3214)- clean up more diagnostics usages |

## [0.12.5] - 2024-04-22

### Features

* cli: implement `--format checkstyle` (#3044)
* cli: implement `--format unix` (#3039)

## [0.12.3] - 2024-04-11

### Bug Fixes

* cli: if format is json do not print summary information (#2899) (#2925)

## [0.10.0] - 2024-03-14

### Features
- miette v7 (#2465) |

## [0.8.0] - 2024-02-26

### Features

* cli,diagnostics: add json reporter (#2451)
* diagnostics: implement json reporter (#2452)

## [0.6.0] - 2024-02-03

### Refactor
- move all miette usages to `oxc_diagnostics` |

## [0.5.0] - 2024-01-12

### Bug Fixes

* diagnostics: always print without considering the `--max-warnings` option (#1996)

## [0.4.0] - 2023-12-08

### Refactor

* rust: move to workspace lint table (#1444)

## [0.2.0] - 2023-09-14

### Bug Fixes

* cli: fix race condition when resolving paths
* deps: use one version of `textwrap`

### Refactor

* cli,diagnostics: implement DiagnosticService (#762)
* cli,linter: move path processing logic from cli to linter (#766)

