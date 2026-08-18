# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.64.0] - 2026-08-18

### 🐛 Bug Fixes

- f405789 formatter_yaml: Consistent chomped eos behavior (#25523) (leaysgur)
- a243311 formatter_yaml: Bump oxc-yaml-parser for contentless block scalar (#25519) (leaysgur)

## [0.63.0] - 2026-08-10

### 🚀 Features

- fd02a89 oxfmt: Dispatch yaml-in-css(frontmatter) to `oxc_formatter_yaml` (#25336) (leaysgur)

### 🐛 Bug Fixes

- 2eaede9 formatter_core: Unify leading-BOM handlings (#25340) (leaysgur)
- f3c6953 formatter_yaml: Don't rewrite overflowing key to implicit (#25274) (leaysgur)

### 📚 Documentation

- 51224a7 formatter_yaml: Pin EOF blank lines divergence (#25269) (leaysgur)

## [0.62.0] - 2026-08-03

### 🐛 Bug Fixes

- f56009a oxfmt: Correct prose about comment width in fits measurement (#25054) (leaysgur)
- ee344bd formatter_yaml: Break long keys off a block scalar header (#25014) (leaysgur)
- d5e3476 formatter_yaml: Let ancestor collection values claim end comments after block scalars (#24897) (leaysgur)
- ec05297 formatter_yaml: Align sequence container end comments to the dash width (#24891) (leaysgur)

### ⚡ Performance

- b91d5a8 formatter_css,formatter_graphql,formatter_yaml,formatter_json: Pre alloc IR buffers (#25234) (leaysgur)

### 📚 Documentation

- eaa7c69 formatter_core: Extract FORMATTER_POLICY (#25233) (leaysgur)

## [0.61.0] - 2026-07-27

### 🚀 Features

- 2357a10 formatter_yaml: Implement YAML formatter (#24534) (leaysgur)

### 🐛 Bug Fixes

- 07ff12f formatter_yaml: Fix indent, blank, and spacing issues (#24837) (leaysgur)

### 📚 Documentation

- 69126a0 formatter_css,fomatter_graphql,formatter_yaml: Update AGENTS.md (#24821) (leaysgur)

