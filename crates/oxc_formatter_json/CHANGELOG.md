# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.58.0] - 2026-07-06

### 🐛 Bug Fixes

- 0a6b16c formatter_json: Preserve key and literal value for json-stringify (#23996) (leaysgur)

## [0.57.0] - 2026-06-29

### 🐛 Bug Fixes

- 8c07cad all: Enable `disable_old_builder` Cargo feature for `oxc_ast` crate in tests (#23888) (overlookmotel)

## [0.56.0] - 2026-06-22

### 🐛 Bug Fixes

- f21ed2c formatter_json: Normalize CRLF for suppressed text (#23702) (leaysgur)
- 8fa7394 formatter_json: Handle wrapped error span (#23472) (leaysgur)

## [0.55.0] - 2026-06-15

### 🚀 Features

- 8cc82c4 formatter_json: Implement json-stringify variant (#23192) (leaysgur)

### 🐛 Bug Fixes

- b0b5d39 formatter_json: Support JSON5/JSON6 value in json/jsonc/json5 (#23193) (leaysgur)

### 📚 Documentation

- 4986613 formatter_json: Update AGENTS.md (#23199) (leaysgur)

## [0.54.0] - 2026-06-08

### 🚀 Features

- 3da77e0 oxfmt: Format `parser:json5` files by `oxc_formatter_json` (#22990) (leaysgur)
- 27a6db8 formatter_json: Implement jsonc variant (#22912) (leaysgur)

### 🐛 Bug Fixes

- 01e0871 formatter,formatter_json: Handle PS/LS as line terminator (#22978) (leaysgur)
- 23902d9 formatter_json: Handle CR only line breaks (#22977) (leaysgur)
- 136b72b formatter_json: Use line_suffix for line comment outside array (#22931) (leaysgur)
- 44e40fa formatter_json: Expand line comment inside array (#22911) (leaysgur)
- 2c86896 formatter_json: Avoid example binary name collision (#22904) (camc314)

### 📚 Documentation

- cc69d8d formatter_json: Update AGENTS.md (#22981) (leaysgur)
- 0490721 formatter_json: Update AGENTS.md (#22976) (leaysgur)
- 7e514bf formatter_json: Update AGENTS.md (#22930) (leaysgur)

## [0.53.0] - 2026-06-01

### 🚀 Features

- 49db054 formatter_json: Implement `oxc_formatter_json` (json variant only) (#22641) (leaysgur)

### 📚 Documentation

- 845f393 oxfmt,formatter,formatter_json,formatter_core: Add/update AGENTS.md (#22873) (leaysgur)

