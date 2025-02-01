# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.15.9] - 2025-02-01

### Bug Fixes

- 5041cb3 vscode: Fix commands by reverting commit `259a47b` (#8819) (Alexander S.)

## [0.15.8] - 2025-01-24

### Refactor

- 259a47b vscode: Move commands and `findBinary` to separate files (#8605) (Alexander S.)

## [0.15.4] - 2024-12-30

### Bug Fixes

- 0794bee editor/vscode: Set minimum supported ver. to `^1.93.0` (#8182) (Pavel Smirnov)

### Refactor

- de8246b language_server: Move structs into own file (#8026) (Alexander S.)

## [0.15.1] - 2024-12-13

### Features

- 38b1c2e editor: Create a command to apply all auto-fixes for the current active text editor (#7672) (Nicholas Rayburn)

## [0.13.2] - 2024-11-26

- b04041d vscode: [**BREAKING**] Use `.oxlintrc.json` as default value for `oxc.configPath` (#7442) (Alexander S.)

### Bug Fixes


## [0.13.1] - 2024-11-23

### Testing

- 779f479 editor: Check if workspace configuration is updated (#7403) (Alexander S.)

## [0.13.0] - 2024-11-21

### Refactor

- 466f395 vscode: Split `ConfigService` and `Config` (#7376) (Alexander S.)

## [0.12.0] - 2024-11-20

### Bug Fixes

- ba0b2ff editor: Reload workspace configuration after change (#7302) (Alexander S.)

### Documentation

- 4c124a8 editor/vscode: Update VS Code readme with installation instructions and available features (#7306) (Nicholas Rayburn)

### Testing

- 5190b7f editor: Add test setup (#7361) (Alexander S.)

## [0.11.1] - 2024-11-09

### Features

- 4dd9b60 editor/vscode: Replace existing output channel and trace output channel with a single LogOutputChannel (#7196) (Nicholas Rayburn)

### Bug Fixes

- eea8879 editor/vscode: Update language client id to fix the resolution of the oxc.trace.server setting (#7181) (Nicholas Rayburn)

## [0.11.0] - 2024-11-03

### Features

- 6b619da editor: Listen to config file changes and trigger a didChangeConfiguration update (#6964) (Nicholas Rayburn)
- 7872927 editor/vscode: Support window/showMessage event (#7085) (Nicholas Rayburn)

### Bug Fixes

- ebf3753 editor: Fix onConfigChange to send the correct config for didChangeConfiguration notification (#6962) (Nicholas Rayburn)

## [0.10.1] - 2024-10-21

### Bug Fixes

- 1bcd707 editor: Update config sent to language server (#6724) (Nicholas Rayburn)

## [0.10.0] - 2024-10-18

- 7f6b219 editor/vscode: [**BREAKING**] Unify configuration logic (#6630) (DonIsaac)

### Features


### Bug Fixes

- cf92730 editor: Use human-readable output channel names (#6629) (DonIsaac)
- d9159a2 editor: Misaligned command prefixes (#6628) (DonIsaac)
- b9c94bb editors/vscode: Temporarily solve oxc_language_server issue on windows (#6384) (dalaoshu)

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

