# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.59.0] - 2026-07-13

### 🚀 Features

- a9a5cd6 formatter_core: Expose `SourceText::as_str()` (#24281) (leaysgur)

### ⚡ Performance

- eeb1913 formatter_core: Avoid per-call `Vec` work-stack in soft-line removal (#23775) (Marius Schulz)

## [0.58.0] - 2026-07-06

### 🚀 Features

- 89ec3d9 formatter_core: Add literal line and root indention primitives (#24051) (leaysgur)
- 213a96b formatter_core: Add no-expand-parent for multiline text (#24050) (leaysgur)

### ⚡ Performance

- 468e1e3 formatter_core: Make printer queues cursor-based (#24098) (Boshen)
- c59f2fe rust: Return impl ExactSizeIterator from slice-backed accessors (#24144) (Boshen)
- c292fb2 formatter: Inline fits element dispatcher (#23982) (camc314)

## [0.57.0] - 2026-06-29

### ⚡ Performance

- 4ddcba0 formatter_core: Add printable-ASCII fast path to TextWidth (#23913) (Lawrence Lin)

### 📚 Documentation

- b4d0dc9 oxfmt,formatter,formatter_css,formatter_core: Update AGENTS.md (#23814) (leaysgur)

## [0.56.0] - 2026-06-22

### 💥 BREAKING CHANGES

- 36009dd allocator: [**BREAKING**] `GetAllocator::allocator` take `&self` (#23676) (overlookmotel)

## [0.54.0] - 2026-06-08

### 🚀 Features

- 27a6db8 formatter_json: Implement jsonc variant (#22912) (leaysgur)

### 🐛 Bug Fixes

- 01e0871 formatter,formatter_json: Handle PS/LS as line terminator (#22978) (leaysgur)

## [0.53.0] - 2026-06-01

### 📚 Documentation

- 845f393 oxfmt,formatter,formatter_json,formatter_core: Add/update AGENTS.md (#22873) (leaysgur)

