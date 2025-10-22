# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.8.0] - 2025-10-22

### 🚀 Features

- 006708d oxfmt: Support `ignorePatterns` in oxfmtrc (#14875) (leaysgur)


## [0.7.0] - 2025-10-21

### 🚀 Features

- 6dfcd80 oxfmt: Search both .json and .jsonc config file (#14848) (leaysgur)

### 🐛 Bug Fixes

- 7a420a1 oxfmt: Handle `.d.ts` file correctly (#14835) (leaysgur)

### 🚜 Refactor

- 6fa7420 oxfmt: Use custom ignore builder (#14850) (leaysgur)


## [0.6.0] - 2025-10-20

### 🚀 Features

- 7f91a26 oxfmt: Handle ignoring files (#14798) (leaysgur)
- 199a2c6 oxfmt: Support `--with-node-modules` option (#14713) (leaysgur)
- 26c5f5a oxfmt: Ignore VCS directories by default (#14616) (leaysgur)
- fec2ed9 oxfmt: Use Prettier style config key and value (#14612) (leaysgur)
- 1b58521 oxfmt,language_server: Enable JSX for all JS source type (#14605) (leaysgur)

### 🐛 Bug Fixes

- ef02760 oxfmt: Handle relative path starts with dot (#14708) (leaysgur)
- ee37f5d oxfmt: Handle default cwd correctly (#14704) (leaysgur)
- 0961c3a oxlint,oxfmt: Skip traversing `.git` directories (#14590) (Boshen)

### 🚜 Refactor

- b7926f3 oxfmt: Update CLI --help details (#14796) (leaysgur)
- 173168b oxfmt: Refactor walk.rs and format.rs relationship (#14795) (leaysgur)
- aea9d79 oxfmt: Pass `PathBuf` from walk.rs to service.rs (#14716) (leaysgur)

### 🧪 Testing

- 7c42ea0 oxfmt: Remove args from snapshot file name (#14800) (leaysgur)


## [0.5.0] - 2025-10-14

### 🚀 Features

- 51ddfa8 oxfmt: Support `.oxfmtrc.json(c)` config file (#14398) (leaysgur)

### 🐛 Bug Fixes

- 0f19be0 oxfmt: Normalize path delimiter on Windows (#14463) (leaysgur)



## [0.3.0] - 2025-09-19

### 🐛 Bug Fixes

- 55775ce oxfmt: Fix up the half-finished lines (#13840) (leaysgur)

### ⚡ Performance

- 59db021 oxfmt: Walk and format at the same time (#13838) (leaysgur)


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



