# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.64.0] - 2026-08-18

### 🐛 Bug Fixes

- 5c0a5ff formatter_css: Fill own-line block comments inside space-separated values (#25578) (leaysgur)
- 53debfa formatter_css: Keep trailing comma and indent after comment-preceded last map value (#25577) (leaysgur)
- 40d38a0 formatter_css: Keep multi value function args intact after leading comment (#25518) (leaysgur)

## [0.63.0] - 2026-08-10

### 🚀 Features

- fd02a89 oxfmt: Dispatch yaml-in-css(frontmatter) to `oxc_formatter_yaml` (#25336) (leaysgur)

### 🐛 Bug Fixes

- 2eaede9 formatter_core: Unify leading-BOM handlings (#25340) (leaysgur)
- e23dccf formatter_css: Bump oxc-css-parser to accept unknown at-rule with interpolated (#25277) (leaysgur)

## [0.62.0] - 2026-08-03

### 🐛 Bug Fixes

- 68146bd formatter_css: Preserve glued plugin words in grid values (#25225) (leaysgur)
- dd28596 formatter_css: Bump oxc-css-parser for substituted at-rule preludes (#25104) (leaysgur)
- f56009a oxfmt: Correct prose about comment width in fits measurement (#25054) (leaysgur)

### ⚡ Performance

- b91d5a8 formatter_css,formatter_graphql,formatter_yaml,formatter_json: Pre alloc IR buffers (#25234) (leaysgur)

### 📚 Documentation

- eaa7c69 formatter_core: Extract FORMATTER_POLICY (#25233) (leaysgur)

## [0.61.0] - 2026-07-27

### 🐛 Bug Fixes

- 761a882 formatter_css: Keep leading `+` in An+B to enable idempotency check (#24973) (leaysgur)
- ae1a39b formatter_css: Align wrapped selector-arg indent (#24971) (leaysgur)
- fb34196 formatter_css: Keep non-ASCII strings with quotes (#24870) (leaysgur)
- 31783e9 formatter_css: Print nested SCSS map with consistent indent (#24789) (leaysgur)

### 📚 Documentation

- 69126a0 formatter_css,fomatter_graphql,formatter_yaml: Update AGENTS.md (#24821) (leaysgur)

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

