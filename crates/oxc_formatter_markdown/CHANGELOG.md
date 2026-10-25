# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.74.0] - 2026-10-19

### 🐛 Bug Fixes

- e236de0 formatter_tests: Skip conformance spec calls with non-literal options (#27957) (leaysgur)
- 2645d2c formatter_markdown: Treat link destinations and footnote references as opaque to code span fences (#27948) (leaysgur)
- c78e80e formatter_core: Report parse error positions relative to the BOM-headed source (#27776) (leaysgur)

### ⚡ Performance

- e5fdc9c formatter_core: Write into one element stack through a concrete `Formatter` (#27559) (leaysgur)
- 1d3ef10 formatter_core: Keep the embedded IR when nothing in it is rewritten (#27673) (leaysgur)

## [0.73.0] - 2026-10-13

### 📚 Documentation

- 37b3c48 formatter_markdown: Fix DIVERGENCES.md examples and slug to match behavior (#27358) (leaysgur)

## [0.72.0] - 2026-10-05

### 💥 BREAKING CHANGES

- e2c68b1 oxfmt: [**BREAKING**] Format `parser:markdown` files by `oxc_formatter_markdown` (#27256) (leaysgur)

### 🐛 Bug Fixes

- 71e400b formatter_markdown: Fixed remaining issues found by fuzz (#27334) (leaysgur)
- 7d0e54d formatter_markdown: Preserve line breaks around Chinese/Japanese characters in all `proseWrap` (#27331) (leaysgur)
- 2445064 formatter_markdown: Keep a preserved line break before a delimiter row that opens no table (#27278) (leaysgur)
- dd72fbf formatter_markdown: Keep container columns as spaces under `useTabs` (#27277) (leaysgur)

### ⚡ Performance

- 72f42a3 formatter_markdown: Pre-allocate the IR buffer (#27333) (leaysgur)

### 📚 Documentation

- 656b81c formatter_markdown: Record unclosed fences closing at a directive's closer (#27279) (leaysgur)
- 66545bf formatter_markdown: Record setext heading wrap divergence (#27221) (leaysgur)

## [0.71.0] - 2026-09-28

### 🚀 Features

- e0b1f9f oxfmt: Bump bundled Prettier version to 3.9.9 (#27002) (leaysgur)
- 342527d oxfmt: Bump bundled Prettier version to 3.9.8 (#26999) (leaysgur)

### 🐛 Bug Fixes

- 7bcb807 formatter_markdown: Keep blank line between HTML and nested list (#27112) (leaysgur)
- fd695f4 formatter_markdown: Fix more mismatches found in ecosystem-ci repos (#27003) (leaysgur)
- 4e77d59 formatter_markdown: Keep a math span after a kept line break from opening a block (#27001) (leaysgur)
- 584b8b0 formatter_markdown: Keep a shape line after a multi-line inline node or link title (#27000) (leaysgur)
- b939645 formatter_markdown: Keep a line break before an inline liquid tag under preserve (#26998) (leaysgur)

## [0.69.0] - 2026-09-21

### 🚀 Features

- 4356dc2 formatter_markdown: Keep frontmatter verbatim (#26779) (leaysgur)
- 7717813 formatter_markdown: Implement markdown formatter (#26434) (leaysgur)

### 🐛 Bug Fixes

- 68ed6a2 formatter_markdown: Fix real-world missmatches (#26785) (leaysgur)

### 📚 Documentation

- 11ff089 formatter_markdown: Follow updated DIVERGENCES policy (#26776) (leaysgur)

