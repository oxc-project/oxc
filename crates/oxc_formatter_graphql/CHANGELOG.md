# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.63.0] - 2026-08-10

### 🐛 Bug Fixes

- 2eaede9 formatter_core: Unify leading-BOM handlings (#25340) (leaysgur)

## [0.62.0] - 2026-08-03

### ⚡ Performance

- b91d5a8 formatter_css,formatter_graphql,formatter_yaml,formatter_json: Pre alloc IR buffers (#25234) (leaysgur)

### 📚 Documentation

- eaa7c69 formatter_core: Extract FORMATTER_POLICY (#25233) (leaysgur)

## [0.61.0] - 2026-07-27

### 📚 Documentation

- 69126a0 formatter_css,fomatter_graphql,formatter_yaml: Update AGENTS.md (#24821) (leaysgur)

## [0.60.0] - 2026-07-20

### 🐛 Bug Fixes

- 5f76998 formatter_graphql: Keep same line comments pending across intervening tokens (#24579) (leaysgur)

## [0.58.0] - 2026-07-06

### 🚀 Features

- 0ccd8a1 formatter_graphql: Update oxc-graphql-parser 0.0.5 (#24106) (leaysgur)
- 0e5bcc9 formatter_graphql: Update oxc-graphql-parser 0.0.4 (#24039) (leaysgur)

### 🐛 Bug Fixes

- e1ece97 formatter_graphql: Break `implements` list by print-width (#23997) (leaysgur)

## [0.57.0] - 2026-06-29

### 💥 BREAKING CHANGES

- 259e0cd oxfmt,formatter_graphql: [**BREAKING**] Support draft syntax with removing prettier fallback (#23326) (leaysgur)

### 🚀 Features

- 4e66212 formatter_graphql: Implement oxc_formatter_graphql (#23317) (leaysgur)

### 🐛 Bug Fixes

- 3f355e5 formatter_graphql: Improve major prettier diffs (#23419) (leaysgur)

