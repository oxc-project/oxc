# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.60.0] - 2026-07-20

### 🐛 Bug Fixes

- 33e32d8 formatter_css: Use `line_suffix` for EOL line comment (#24580) (leaysgur)

## [0.59.0] - 2026-07-13

### 🚀 Features

- 3a7fe74 formatter_css: Update oxc-css-parser to 0.0.7 (#24434) (leaysgur)
- 0173cd3 formatter_css: Format Less :extend and merge props (#24358) (leaysgur)

### 🐛 Bug Fixes

- fcc28df formatter_css: Keep glued-braket-value tight (#24352) (leaysgur)
- eeabc4a formatter_css: Bail on EOF-recovered parse errors (#24282) (leaysgur)

## [0.58.0] - 2026-07-06

### 🚀 Features

- 4f4313e formatter_css: Update oxc-css-parser 0.0.5 (#24120) (leaysgur)
- e0b35a1 formatter_css: Update `oxc-css-parser@0.0.3` (#23974) (leaysgur)

### 🐛 Bug Fixes

- 9af3833 formatter_css: Make scss formatter consistent (#24207) (leaysgur)
- 46d7194 formatter_css: Use fill IR for `@forward` members (#24206) (leaysgur)
- e31038f formatter_css: Keep comment inside sass config list (#24205) (leaysgur)
- 9bf4b4a formatter_css: Align CSS output to Prettier 3.9.1 (#24100) (leaysgur)
- cd2452e formatter_css: Align SCSS output to Prettier 3.9.1 (#24097) (leaysgur)
- 4ee8745 formatter_css: Keep selector value contain line-break without breaking line (#24055) (leaysgur)
- 903ab6e formatter_css: Preserve newlines in css-in-js selector list (#23992) (leaysgur)

## [0.57.0] - 2026-06-29

### 💥 BREAKING CHANGES

- accbc49 oxfmt: [**BREAKING**] Format `parser:css,less,scss` files + css-in-js by `oxc_formatter_css` (#23321) (leaysgur)

### 🚀 Features

- dffa4b3 formatter_css: Implement `oxc_formatter_css` (#23320) (leaysgur)

### 🐛 Bug Fixes

- 67325ae formatter_css: Handle frontmatter language (#23819) (leaysgur)
- 48e2d78 formatter_css: Improve major prettier diffs (#23327) (leaysgur)

### 📚 Documentation

- b4d0dc9 oxfmt,formatter,formatter_css,formatter_core: Update AGENTS.md (#23814) (leaysgur)

