# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.146.0] - 2026-08-19

### 🐛 Bug Fixes

- 526c2b3 codegen: Remove deprecated test APIs (#25889) (camc314)
- 344a9b2 codegen: Align enum template sourcemaps (#25872) (camc314)

### ⚡ Performance

- 8783524 packages/codegen: Remove loop from `growMappingBuffer` (#25886) (overlookmotel)
- b6b14c0 packages/codegen: Remove redundant branch (#25885) (overlookmotel)
- be6adf0 packages/sourcemap: Make `location` passed to `recordSourceMapping` always a number (#25884) (overlookmotel)
- f720f15 packages/codegen: Take named-mapping only code off hot path in `recordSourceMapping` (#25883) (overlookmotel)
- a734af5 packages/codegen: Remove redundant setting of `last` in `printImportAttributes` and `printIf` (#25882) (overlookmotel)
- c6e44c6 packages/codegen: Combine 2 writes in `printJSXElement` (#25881) (overlookmotel)
- 8801662 packages/codegen: Remove redundant mapping from `printParams` (#25880) (overlookmotel)

### 📚 Documentation

- 5ad330e packages/codegen: Reformat comments (#25876) (overlookmotel)

## [0.145.0] - 2026-08-18

### 💥 BREAKING CHANGES

- 365274e packages/codegen: [**BREAKING**] `printSync` return an object (#25720) (overlookmotel)

### 🚀 Features

- 6a7eb60 packages/codegen: Alter capitalization of `sourceFilename` option (#25854) (overlookmotel)
- 300763f packages/codegen: Add full source map support (#25585) (camc314)
- a4478e9 codegen: Add `oxc-codegen` package (#25488) (overlookmotel)

### 🐛 Bug Fixes

- 0fbcf64 codegen: Validate sourcemap options (#25860) (camc314)
- 4cc7ea4 codegen: Reject invalid indent options (#25807) (camc314)
- 0c68b7f estree: Emit `decorators` on `FormalParameterRest` (#25582) (camc314)
- 59e8895 codegen: Validate starting indent level (#25550) (camc314)

### ⚡ Performance

- 5444cbf codegen: Skip escaping harmless `<` tokens (#25564) (camc314)

### 📚 Documentation

- 58f7ab9 packages/codegen: Reformat docs and comments (#25848) (overlookmotel)
- ce03ac1 packages/codegen: Fix JSDoc comments for `printSync` (#25722) (overlookmotel)
- 386a699 packages/codegen: Correct JSDoc comment (#25716) (overlookmotel)
- fd62354 codegen: Revamp package readme (#25560) (camc314)
- ffa3153 codegen: Clarify raw transfer Node requirement (#25565) (camc314)
- c5d4063 codegen: Clarify binary walk allocations (#25562) (camc314)
- 5703d7a codegen: Correct stale printer comments (#25561) (camc314)

