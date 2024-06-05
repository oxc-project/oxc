# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.2.6] - 2024-01-26

### Features

* vscode: allow config path configuration (#2172)

## [0.2.0] - 2024-01-12

### Features

* lsp: support vue, astro and svelte (#1923)

## [0.1.1] - 2024-01-06

### Features

* vscode: support lint vue file (#1842)

### Bug Fixes

* lsp: make the server available in nvim-lspconfig (#1823)
* vscode: change all names to oxc_language_server

## [0.0.22] - 2023-12-25

### Bug Fixes

* vscode: don't lint files in .gitignore and .eslintignore (#1765)

## [0.0.21] - 2023-12-18

### Features

* linter: add  jsx-a11y settings (#1668)
* vscode: use icon to represent enabled status (#1675)- add option to control enable/disable oxc linter (#1665) |

### Bug Fixes

* vscode: report problem more accurately  (#1681)

## [0.0.20] - 2023-12-13

### Features

* vscode: add a option to control oxc lint timing (#1659)

### Bug Fixes

* vscode: fix the broken package path

## [0.0.19] - 2023-12-08

### Bug Fixes

* oxc_vscode: vscode extension - check on file change (not on file save)  (#1525)

## [0.0.18] - 2023-11-22

### Refactor

* rust: move to workspace lint table (#1444)

