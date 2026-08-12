# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.63.0] - 2026-08-10

### 🚀 Features

- fd02a89 oxfmt: Dispatch yaml-in-css(frontmatter) to `oxc_formatter_yaml` (#25336) (leaysgur)
- ab12665 formatter_core: Add `hardlineWithoutBreakParent` equivalent IR (#25273) (leaysgur)

### 🐛 Bug Fixes

- ab52a59 formatter: Format xxx-in-js inside JSDoc js fence (#25414) (leaysgur)
- 1a2c64a formatter,oxfmt: Apply effective print width for JSDoc fence (#25413) (leaysgur)
- 2eaede9 formatter_core: Unify leading-BOM handlings (#25340) (leaysgur)
- c29b587 formatter_core: Measure decided-flat fill separator as flat during group re-measure (#25276) (leaysgur)

### 📚 Documentation

- 6eae5c9 formatter,oxfmt: Record embed-layer decisions in place (#25422) (leaysgur)

## [0.62.0] - 2026-08-03

### 🐛 Bug Fixes

- f56009a oxfmt: Correct prose about comment width in fits measurement (#25054) (leaysgur)

### 📚 Documentation

- eaa7c69 formatter_core: Extract FORMATTER_POLICY (#25233) (leaysgur)

## [0.61.0] - 2026-07-27

### ⚡ Performance

- bb73b23 formatter_core: Bound the thread-local scratch cache (#24793) (leaysgur)
- a5f7b15 formatter: Stage assignment-like left hand side on the heap (#24613) (leaysgur)
- 94de05f formatter: Accumulate JSX child-list builders on the heap (#24585) (leaysgur)
- 7810e8a formatter_core: Share one thread-cached scratch vector across staging buffers (#24583) (leaysgur)
- c191f51 formatter_core: Stage IR buffers on the heap to reduce arena memory (#24582) (leaysgur)

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

