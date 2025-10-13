# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).



## [1.21.0] - 2025-10-08

### 🐛 Bug Fixes

- 6e8d2f6 language_server: Ignore JS plugins (#14379) (overlookmotel)



## [1.19.0] - 2025-09-29

### 🚀 Features

- b4d716f linter/plugins: Move custom JS plugin config to `jsPlugins` (#14133) (overlookmotel)

### 🐛 Bug Fixes

- 8879b5a linter/plugins: Add types export to `npm/oxlint` (#14219) (overlookmotel)



## [1.17.0] - 2025-09-23

### 🚀 Features

- 3e117c6 linter/plugins: Add `defineRule` API (#13945) (overlookmotel)
- a14aa79 npm/oxlint: Convert to ES modules (#13876) (Boshen)
- b52389a node: Bump `engines` field to require Node.js 20.19.0+ for ESM support (#13879) (Copilot)
- 53d04dd linter: Convert `oxlint` to NAPI app (#13723) (overlookmotel)

### 🚜 Refactor

- bb040bc parser, linter: Replace `.mjs` files with `.js` (#14045) (overlookmotel)
- 7e0d736 linter/plugins: Rename `--experimental-js-plugins` to `--js-plugins` (#13860) (overlookmotel)




## [1.14.0] - 2025-08-30

### 🚀 Features

- 7fc4aef npm/oxlint: 'oxlint-tsgolint': '>=0.1.4' (Boshen)


## [1.13.0] - 2025-08-26

### 🐛 Bug Fixes

- 02c779f npm/oxlint: Make `oxlint-tsgolint` truly optional (#13153) (Boshen)




## [1.11.1] - 2025-08-09

### 🐛 Bug Fixes

- 8c57153 npm/oxlint: Fix `oxlint-tsgolint` version range for yarn (Boshen)

### 🚜 Refactor

- 238b183 linter: Use `fast-glob` instead of `globset` for `GlobSet` (#12870) (shulaoda)


## [1.11.0] - 2025-08-07

### 🚀 Features

- ac46347 oxlint: Add `tsgolint` integration (#12485) (camchenry)


## [1.10.0] - 2025-08-06

### 🚀 Features

- 9b35600 linter/jsx-a11y: Add support for mapped attributes in label association checks (#12805) (camc314)


## [1.9.0] - 2025-07-29

### 🚜 Refactor

- bea652f linter: Add `vue` and `regex` to `BuiltinLintPlugins` (#12542) (Sysix)



## [1.7.0] - 2025-07-16

### 🚀 Features

- a4dae73 linter: Introduce `LintPlugins` to store builtin + custom plugins (#12117) (camc314)










# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.16.12] - 2025-05-25

### Features

- 6a7018e linter: Generate stricter json schema for lint plugins (#11219) (camc314)

### Bug Fixes

- e8470d9 linter: Delay merging of oxlintrc configs (#10835) (camc314)

## [0.15.14] - 2025-03-11

### Features

- 3fce826 linter: Add support for `extends` property in oxlintrc (#9217) (camchenry)

## [0.15.13] - 2025-03-04

### Documentation

- 24850e7 linter: Add example of how configure rule (#9469) (Cédric DIRAND)

## [0.15.11] - 2025-02-16

### Features

- 5d508a4 linter: Support `env` and `globals` in `overrides` configuration (#8915) (Sysix)

## [0.15.8] - 2025-01-24

### Features

- 79ba9b5 linter: Added support to run in Node.JS legacy versions (#8648) (Luiz Felipe Weber)

## [0.15.7] - 2025-01-19

### Features

- 538b24a linter: Format the configuration documentation correctly (#8583) (Tapan Prakash)

## [0.14.0] - 2024-12-01

### Features

- 32f860d linter: Add support for ignorePatterns property within config file (#7092) (Nicholas Rayburn)

### Documentation

- a6b0100 linter: Fix config example headings (#7562) (Boshen)

## [0.13.0] - 2024-11-21

### Documentation

- df143ca linter: Add docs for config settings (#4827) (DonIsaac)

## [0.12.0] - 2024-11-20

### Features

- 2268a0e linter: Support `overrides` config field (#6974) (DonIsaac)

## [0.11.0] - 2024-11-03

### Documentation

- 4551baa linter: Document `rules` (#6983) (Boshen)

## [0.10.3] - 2024-10-26

### Documentation

- 3923e63 linter: Add schema to config examples (#6838) (Dmitry Zakharov)

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

