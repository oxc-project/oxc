# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [1.63.0] - 2026-05-05

### 📚 Documentation

- cacbc4a linter: Fix jest settings docs. (#22127) (connorshea)

## [1.62.0] - 2026-04-27

### 🚀 Features

- 348f46c linter: Add `respectEslintDisableDirectives` option (#21384) (Christian Vuerings)

### 🐛 Bug Fixes

- 8c425db linter: Allow string for jest version in config schema (#21649) (camc314)

## [1.61.0] - 2026-04-20

### 🚀 Features

- 38d8090 linter/jest: Implemented jest `version` settings in config file. (#21522) (Said Atrahouch)

## [1.60.0] - 2026-04-13

### 📚 Documentation

- cfd8a4f linter: Don't rely on old eslint doc for available globals (#21334) (Nicolas Le Cam)

## [1.59.0] - 2026-04-06

### 🐛 Bug Fixes

- dd2df87 npm: Export package.json for oxlint and oxfmt (#20784) (kazuya kawaguchi)

## [1.58.0] - 2026-03-30

### 🚀 Features

- 16516de linter: Enhance types for `DummyRule` (#20751) (camc314)

### 📚 Documentation

- be3dcc1 linter: Add note about node version + custom TS plugin (#19381) (camc314)

## [1.55.0] - 2026-03-12

### 🐛 Bug Fixes

- bc20217 oxlint,oxfmt: Omit useless `| null` for `Option<T>` field from schema (#20273) (leaysgur)

### 📚 Documentation

- f339f10 linter/plugins: Promote JS plugins to alpha status (#20281) (overlookmotel)

## [1.54.0] - 2026-03-12

### 📚 Documentation

- 0c7da4f linter: Fix extra closing brace in example config. (#20253) (connorshea)

## [1.52.0] - 2026-03-09

### 🚀 Features

- 61bf388 linter: Add `options.reportUnusedDisableDirectives` to config file (#19799) (Peter Wagenet)
- 2919313 linter: Introduce denyWarnings config options (#19926) (camc314)
- a607119 linter: Introduce maxWarnings config option (#19777) (camc314)

### 📚 Documentation

- 6c0e0b5 linter: Add oxlint.config.ts to the config docs. (#19941) (connorshea)
- 160e423 linter: Add a note that the typeAware and typeCheck options require oxlint-tsgolint (#19940) (connorshea)

## [1.51.0] - 2026-03-02

### 🚀 Features

- f34f6fa linter: Introduce typeCheck config option (#19764) (camc314)
- 694be7d linter: Introduce typeAware as config options (#19614) (camc314)

### 🐛 Bug Fixes

- 04e6223 npm: Add `preferUnplugged` for Yarn PnP compatibility (#19829) (Boshen)

### 📚 Documentation

- 2fa936f README.md: Map npm package links to npmx.dev (#19666) (Boshen)

## [1.45.0] - 2026-02-10

### 🐛 Bug Fixes

- 1b2f354 ci: Add missing riscv64/s390x napi targets for oxfmt and oxlint (#19217) (Cameron)

## [1.44.0] - 2026-02-10

### 🚀 Features

- ee2925b oxlint/lsp: Enable JS plugins (#18834) (overlookmotel)
- 9788a96 oxlint,oxfmt: Add more native builds (#18853) (Boshen)

### 📚 Documentation

- 9561e7f linter/plugins: Alter JS plugins example (#18900) (overlookmotel)
- b425a0c linter: Document jsPlugins examples (#18671) (Cameron)
- df2b7fa linter: Expand settings example with reference to custom plugins (#18670) (camc314)

## [1.42.0] - 2026-01-26

### 🚀 Features

- 15d69dc linter: Implement react/display-name rule (#18426) (camchenry)

### 📚 Documentation

- 8ccd853 npm: Update package homepage URLs and add keywords (#18509) (Boshen)

## [1.41.0] - 2026-01-19

### 📚 Documentation

- 8a294d5 oxfmt, oxlint: Update logo (#18242) (Dunqing)

## [1.37.0] - 2026-01-05

### 💥 BREAKING CHANGES

- f7da875 oxlint: [**BREAKING**] Remove oxc_language_server binary (#17457) (Boshen)

### 📚 Documentation

- 7e5fc90 linter: Update list of plugins that are reserved. (#17516) (connorshea)

## [1.35.0] - 2025-12-22

### 🚀 Features

- 9e624c9 linter/react: Add `version` to `ReactPluginSettings` (#17169) (camc314)

## [1.34.0] - 2025-12-19

### 🚀 Features

- a0f74a0 linter/config: Allow aliasing plugin names to allow names the same as builtin plugins (#15569) (Cameron)

### 🐛 Bug Fixes

- 005ec25 linter: Permit `$schema` `.oxlintrc.json` struct (#17060) (Copilot)
- d446c43 linter: Prevent extra fields from being present on oxlint config file (#16874) (connorshea)

## [1.30.0] - 2025-11-24

### 🚀 Features

- 595867a oxlint: Generate markdownDescription fields for oxlint JSON schema. (#15959) (connorshea)

## [1.29.0] - 2025-11-17

### 🚀 Features

- 84de1ca oxlint,oxfmt: Allow comments and also commas for vscode-json-ls (#15612) (leaysgur)

## [1.26.0] - 2025-11-05

### 🚀 Features

- 26f24d5 linter: Permit comments in `.oxlintrc.json` via json schema file (#15249) (Martin Leduc)

### 🐛 Bug Fixes

- d6996d0 linter: Fix JSON schema to deny additional properties for categories enum. (#15257) (Connor Shea)
- 9304f9f linter: Fix JSON schema to deny additional properties for plugins enum. (#15259) (Connor Shea)

### 📚 Documentation

- 84ef5ab linter: Avoid linebreaks for markdown links and update plugins docs in the configuration schema. (#15246) (Connor Shea)


## [1.25.0] - 2025-10-30

### 🚀 Features

- bd74603 linter: Add support for vitest/valid-title rule (#12085) (Tyler Earls)


## [1.24.0] - 2025-10-22

### 🐛 Bug Fixes

- 28e76ec oxlint: Resolving JS plugin failing when `extends` is used (#14556) (camc314)




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

