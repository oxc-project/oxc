# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).




## [1.10.0] - 2025-08-06

### 🚀 Features

- 5475075 vscode/language_server: Add `tsConfigPath` option (#12484) (Sysix)


## [1.9.0] - 2025-07-29

### 🐛 Bug Fixes

- cc19c8b vscode: Fix statusbar icon order (#12544) (Christian Fehmer)







## [1.3.0] - 2025-06-23

### 🚀 Features

- 1a54184 linter: Add fix for unused disable directive (#11708) (Sysix)


## [1.2.0] - 2025-06-19

### 🚀 Features

- 0b4261b vscode: Add `oxc.requireConfig` configuration (#11700) (Sysix)
- 094b81c language_server: Add `unusedDisableDirectives` option (#11645) (Sysix)



## [1.0.0] - 2025-06-10

## [0.18.1] - 2025-06-09

### 🚜 Refactor

- 7ab84c2 editor: Use pattern for textDocument filter (#11559) (Sysix)

### 📚 Documentation

- e13ed51 editor/vscode: Fix typo in README (#11572) (David)

## [0.18.0] - 2025-06-06

### Testing

- 2ba2893 editor: Fix test for auto `fixAll` on save (#11448) (Sysix)

## [0.16.12] - 2025-05-25

### Features

- 0c1f382 language_server: Watch for files inside `.oxlintrc.json` extends (#11226) (Sysix)
- 1675b2c language_server: Tell clients to watch for .oxlintrc.json files (#11078) (Sysix)

### Refactor

- a28fe1e editor: Use always the provided language server (#11115) (Sysix)
- 35761ae language_server/editor: Refresh file watchers without restarting the server (didChangeConfiguration) (#11112) (Sysix)
- d5fdf17 language_server/editor: Refresh file watchers without restarting the server (didChangeWorkspaceFolders) (#11094) (Sysix)

## [0.16.11] - 2025-05-16

### Features

- 078bf0b language_server: Better fallback handling when passing invalid `Options` values (#10930) (Sysix)
- be7f7e1 language_server/editor: Support multi workspace folders (#10875) (Sysix)

### Bug Fixes

- 87bf2a8 editor: Send only `workspace/didChangeConfiguration` when some workspace configuration is effected (#11017) (Sysix)
- ed5708d editor: Detect all workspaces config path changes (#11016) (Sysix)

### Refactor

- 3cc1466 language_server: New configuration structure for `initialize` and `workspace/didChangeConfiguration` (#10890) (Sysix)

### Testing

- 76b6b33 editor: Add tests for multi workspace folder setup (#10904) (Sysix)

## [0.16.10] - 2025-05-09

### Features

- e1bc037 language_server: Request for workspace configuration when client did not send them in `initialize` (#10789) (Sysix)
- 3bd339b language_server: Provide commands / code actions for unopened files (#10815) (Sysix)

## [0.16.9] - 2025-05-02

### Bug Fixes

- 4ee95ec editor: Activate extension when astro files are opened too (#10725) (Sysix)

### Documentation

- 275fe71 editor: `oxc.flags` are not related to `oxlint` (#10645) (Sysix)

### Testing

- 1c4f90f editor: Add test for nested config serverity (#10697) (Sysix)

## [0.16.8] - 2025-04-27

### Bug Fixes

- 966fb03 editor: Fix memory leaks when server or watchers restarted (#10628) (Sysix)

### Performance

- 3c27d0d editor: Avoid sending `workspace/didChangeConfiguration` request when the server needs a restarts (#10550) (Sysix)

### Refactor

- e903ba2 editor: Split Config to VSCodeConfig and WorkspaceConfig (#10572) (Sysix)

## [0.16.7] - 2025-04-21

### Features

- bb8a078 language_server: Use linter runtime (#10268) (Sysix)

### Bug Fixes

- df488d4 language_server: Workspace edits as one batch when `source.fixAll.oxc` is the context (#10428) (Sysix)

### Refactor

- 8731f14 editor: Output error when custom server path is not accessible (#10518) (Sysix)

### Testing

- 83baf8b editor: Correct test diagnostic for import plugin (#10453) (Sysix)

## [0.16.6] - 2025-04-14

### Testing

- 62f7d76 editor: Refactor tests to use fixtures (#10381) (Sysix)

## [0.16.5] - 2025-04-07

### Features

- 2f6810a editor: Add named fixes for code actions (#10203) (camchenry)
- 32b9d1e language_server: Add `fix_kind` flag (#10226) (Sysix)
- dab1bd8 language_server: Search for nested configurations by initialization (#10120) (Sysix)

### Documentation

- f115f71 editor: Add readme block for possible configurations (#10243) (Sysix)

### Testing

- 297d07f editor: Add e2e tests for creating oxlint configurations on the fly (#10138) (Sysix)
- 29be469 editor: Add test for code actions (#10168) (camchenry)
- ba817a9 editor: Add E2E Diagnostics test (#10133) (Sysix)

## [0.16.4] - 2025-04-01

- da6336c language_server: [**BREAKING**] Remove `enable` configuration, the client should shutdown the server instead (#9990) (Sysix)

### Bug Fixes

- 0a33e27 editor: Update `initializationOptions` for a possible restart (#10121) (Sysix)
- ac780a2 editor: Repair filewatchers when no custom config provided (#10104) (Sysix)
- 4303ace editor: Dont send `didChangeConfiguration` request to the server when it is shutdown (#10084) (Sysix)

### Refactor

- 327be53 editor: `LanguageClient` can be undefined (#10112) (Sysix)
- 5ec477c editor: Make `onConfigChange` async (#10110) (Sysix)
- a278d73 editor: Use warning background when the plugin is deactived (#10085) (Sysix)
- c0e5251 language_server: Set `null` as a default value for `configPath` (#10047) (Sysix)

### Testing

- 410b8d6 editor: Use debug build of the language server (#10083) (Sysix)
- 500add0 editor: Add test for `oxc.fixAll` command (#10045) (Sysix)

## [0.16.3] - 2025-03-25

### Testing

- 878bec6 editor: Add test for `oxc.toggleEnable` command (#9987) (Sysix)
- 093e7e5 editor: Add `oxc.showOutputChannel` command test (#9986) (Sysix)
- c3af9a4 editor: Add tests for listing all oxc commands (#9930) (Sysix)

## [0.16.2] - 2025-03-21

- bfb416c editor: [**BREAKING**] Enable nested configuration by default (#9929) (Sysix)

### Bug Fixes


## [0.16.1] - 2025-03-20

### Features

- 0973356 editor: Support nested configs (#9743) (Nicholas Rayburn)

## [0.16.0] - 2025-03-16

### Features

- 27d6e9b editor: Only watch .oxlintrc.json or user supplied config paths (#9731) (Nicholas Rayburn)

## [0.15.11] - 2025-02-16

### Bug Fixes

- bcd4e49 editors/vscode: Fix `no-useless-call` warning (Boshen)

## [0.15.10] - 2025-02-06

### Features

- f4662a9 oxc_language_server: Implement `oxc.fixAll` workspace command (#8858) (Marek Vospel)

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

