# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.9.10] - 2024-10-07

### Features

- f272137 editors/vscode: Clear diagnostics on file deletion (#6326) (dalaoshu)
- 1a5f293 editors/vscode: Update VSCode extention to use project's language server (#6132) (dalaoshu)

## [0.9.3] - 2024-09-07

### Styling

- 7414ff8 editors: Add trailing newline to `.prettierignore` (#5540) (overlookmotel)- 694f032 Add trailing line breaks to `package.json` files (#5542) (overlookmotel)

## [0.7.1] - 2024-08-12

### Features

- cc922f4 vscode: Provide config's schema to oxlint config files (#4826) (Don Isaac)

## [0.2.6] - 2024-01-26

### Features

- d5b378a vscode: Allow config path configuration (#2172) (Julien Tanay)

## [0.2.0] - 2024-01-12

### Features

- fe48bfa lsp: Support vue, astro and svelte (#1923) (IWANABETHATGUY)

## [0.1.1] - 2024-01-06

### Features

- 665f818 vscode: Support lint vue file (#1842) (Wenzhe Wang)

### Bug Fixes

- c1bac34 lsp: Make the server available in nvim-lspconfig (#1823) (IWANABETHATGUY)
- ff0d0e0 vscode: Change all names to oxc_language_server (Boshen)

## [0.0.22] - 2023-12-25

### Bug Fixes

- fc7c857 vscode: Don't lint files in .gitignore and .eslintignore (#1765) (IWANABETHATGUY)

## [0.0.21] - 2023-12-18

### Features

- 6a90cd4 linter: Add  jsx-a11y settings (#1668) (msdlisper)
- 37d5152 vscode: Use icon to represent enabled status (#1675) (IWANABETHATGUY)- e529b38 Add option to control enable/disable oxc linter (#1665) (IWANABETHATGUY)

### Bug Fixes

- ef08892 vscode: Report problem more accurately  (#1681) (IWANABETHATGUY)

## [0.0.20] - 2023-12-13

### Features

- e63576d vscode: Add a option to control oxc lint timing (#1659) (IWANABETHATGUY)

### Bug Fixes

- e2d5763 vscode: Fix the broken package path (Boshen)

## [0.0.19] - 2023-12-08

### Bug Fixes

- 8251a34 oxc_vscode: Vscode extension - check on file change (not on file save)  (#1525) (IWANABETHATGUY)

## [0.0.18] - 2023-11-22

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

