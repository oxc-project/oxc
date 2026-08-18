# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

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

