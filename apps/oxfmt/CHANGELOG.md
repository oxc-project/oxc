# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.2.0] - 2025-09-16

### 💥 BREAKING CHANGES

- d90bebc oxfmt: [**BREAKING**] Change default behavior more `cargo fmt` like (#13794) (leaysgur)

### 🧪 Testing

- afa2297 oxfmt: Fix failing tests on Windows (#13801) (leaysgur)
- 5fbffcf oxfmt: Enable changing `cwd` during tests (#13797) (leaysgur)


## [0.1.0] - 2025-09-12

### 🚀 Features

- 1d72f8b oxfmt: Support --no-error-on-unmatched-pattern (#13671) (leaysgur)

### 🐛 Bug Fixes

- d6628bf oxfmt: Print sorted output (#13709) (leaysgur)
- 056f6de oxfmt: Set preserve_parens: false to prevent panic (#13666) (leaysgur)

### 🚜 Refactor

- 6b74078 formatter: Move `is_supported_source_type` to `oxc_formatter` crate (#13702) (Sysix)

### 🧪 Testing

- 83d735b oxfmt: Use normalized path separator (#13726) (leaysgur)
- 289ef9b oxfmt: Add tests setup (#13684) (leaysgur)



