# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.35.0] - 2026-02-23

### 🚀 Features

- 984dc07 oxfmt: Strip `"experimental"SortXxx` prefix (#19567) (leaysgur)

## [0.34.0] - 2026-02-19

### 🐛 Bug Fixes

- 6c61b70 oxfmt: Fix outdated `sortImports.groups` doc comments (#19513) (leaysgur)

## [0.33.0] - 2026-02-16

### 💥 BREAKING CHANGES

- 9c34f72 formatter/sort_imports: [**BREAKING**] Report invalid group name with renaming `side-effect` > `side_effect` (#19416) (leaysgur)

### 🚀 Features

- 4baebef formatter/sort_imports: Support `{ newlinesBetween: bool }` inside `groups` (#19358) (leaysgur)
- d1c2fb6 formatter/sort_imports: Support `customGroups` attributes(`selector` and `modifiers`) (#19356) (leaysgur)

## [0.30.0] - 2026-02-10

### 🐛 Bug Fixes

- 1b2f354 ci: Add missing riscv64/s390x napi targets for oxfmt and oxlint (#19217) (Cameron)

## [0.29.0] - 2026-02-10

### 💥 BREAKING CHANGES

- 856a01f formatter/sort_imports: [**BREAKING**] Replace prefix match with glob pattern in `customGroups.elementNamePattern` (#19066) (leaysgur)

### 🚀 Features

- 6ee2d59 oxfmt: Use `oxc_formatter` in js-in-xxx part (#18373) (leaysgur)
- 9788a96 oxlint,oxfmt: Add more native builds (#18853) (Boshen)

## [0.27.0] - 2026-01-26

### 📚 Documentation

- 8ccd853 npm: Update package homepage URLs and add keywords (#18509) (Boshen)

## [0.26.0] - 2026-01-19

### 📚 Documentation

- 8a294d5 oxfmt, oxlint: Update logo (#18242) (Dunqing)

## [0.25.0] - 2026-01-19

### 🚀 Features

- a95b9bb oxfmt: Support oxfmtrc `overrides` config (#18068) (leaysgur)
- 984d5c1 oxfmt/sort-imports: Support `options.customGroups` (#17576) (nilptr)
- cc3e74b oxfmt: Add Prettier specific fields in `Oxfmtrc` (#17981) (leaysgur)
- 6ffe315 oxfmt: Add more `Oxfmtrc` fields description (#17979) (leaysgur)

## [0.24.0] - 2026-01-12

### 🚀 Features

- 86c0168 oxfmt/sort_package_json: Handle `oxfmtrc.sort_scripts` option (#17738) (leaysgur)

### 📚 Documentation

- 62b7a01 formatter: Clarify `experimentalTailwindcss` configuration comments (#17898) (Dunqing)

## [0.22.0] - 2026-01-05

### 💥 BREAKING CHANGES

- f7da875 oxlint: [**BREAKING**] Remove oxc_language_server binary (#17457) (Boshen)

### 🚀 Features

- 8fd4ea9 oxfmt: `options.embeddedLanguageFormatting` is now `"auto"` by default (#17649) (leaysgur)

## [0.21.0] - 2025-12-29

### 🐛 Bug Fixes

- 0a39cba oxfmt: Update wrong doc comment (#17288) (leaysgur)

## [0.20.0] - 2025-12-22

### 🚀 Features

- 97a02d1 oxfmt: Add `insertFinalNewline` option (#17251) (leaysgur)

## [0.18.0] - 2025-12-15

### 🚀 Features

- afd6c44 oxfmt: Support `quoteProps: consistent` in `Oxfmtrc` (#16721) (leaysgur)
- 28e0682 oxfmt: Enable experimental `package.json` sorting by default (#16593) (leaysgur)

### ⚡ Performance

- 6f3aaba oxfmt: Use `worker_threads` by `tinypool` for prettier formatting (#16618) (leaysgur)

### 📚 Documentation

- 8babdf9 oxfmt: Improve docs for `.oxfmtrc.jsonc` config fields and add markdownDescription fields to JSON Schema (#16587) (connorshea)

## [0.17.0] - 2025-12-08

### 🚀 Features

- 7374856 formatter/sort-imports: Support `options.internalPattern` (#16372) (leaysgur)

## [0.16.0] - 2025-12-01

### 🐛 Bug Fixes

- 9706a1a oxfmt: Ignore unsupported options (#16085) (leaysgur)

## [0.15.0] - 2025-11-24

### 💥 BREAKING CHANGES

- a937890 formatter: [**BREAKING**] Default to `lineWidth: 100` (#15933) (leaysgur)

### 🚀 Features

- 7818e22 formatter/sort-imports: Support `options.groups` (#15831) (leaysgur)

## [0.14.0] - 2025-11-17

### 🚀 Features

- 84de1ca oxlint,oxfmt: Allow comments and also commas for vscode-json-ls (#15612) (leaysgur)

## [0.12.0] - 2025-11-10

### 🚀 Features

- 3251000 oxfmt: Use `prettier` directly and bundle `prettier` (#15544) (Dunqing)
- 5708126 formatter/sort_imports: Add `options.newlinesBetween` (#15369) (leaysgur)

## [0.11.0] - 2025-11-06

### 🐛 Bug Fixes

- 7e0c13e oxfmt: Just run dist/cli.js (#15355) (Yuji Sugiura)


## [0.10.0] - 2025-11-04

### 🚀 Features

- b77f254 oxfmt,formatter: Support `embeddedLanguageFormatting` option (#15216) (leaysgur)

### 🐛 Bug Fixes

- f5d0348 oxfmt: Sync `dependencies` with `npm/oxfmt` and `apps/oxfmt` (#15261) (leaysgur)


## [0.9.0] - 2025-10-30

### 💼 Other

- 6368793 oxfmt: V0.9.0 (#15091) (Boshen)
- aceff66 oxfmt: V0.9.0 (#15088) (Boshen)



## [0.8.0] - 2025-10-22

### 🚀 Features

- 381e08c oxfmt: More friendly JSON schema (#14879) (leaysgur)
- 006708d oxfmt: Support `ignorePatterns` in oxfmtrc (#14875) (leaysgur)



## [0.6.0] - 2025-10-20

### 🚀 Features

- fec2ed9 oxfmt: Use Prettier style config key and value (#14612) (leaysgur)


## [0.5.0] - 2025-10-14

### 🚀 Features

- 8077f9b oxfmt: Provide JSON schema for `.oxfmtrc.json` (#14399) (leaysgur)


## [0.4.0] - 2025-10-09

### 🐛 Bug Fixes

- 59dc17e oxfmt: Change bin script to ESM (#14263) (Boshen)

### 🚜 Refactor

- 226deee oxfmt: Rename build script to `.js` (#14046) (overlookmotel)


## [0.3.0] - 2025-09-19

### 🚀 Features

- b52389a node: Bump `engines` field to require Node.js 20.19.0+ for ESM support (#13879) (Copilot)
- 25437db npm/oxfmt: Convert to ES modules (#13877) (Boshen)

### 📚 Documentation

- 2a35745 oxfmt: The current version does not work; DO NOT USE. (#13844) (Boshen)



