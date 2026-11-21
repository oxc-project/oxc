# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.159.0] - 2026-11-16

### 🚀 Features

- e375dff lexer: Complete NEON support for aarch64 (#29390) (Shanon Jackson)
- 523a063 lexer: Publish lexer and SIMD crates (#29264) (Boshen)

### 🐛 Bug Fixes

- 83fb644 lexer: Combine surrogate pairs written with any escape form (#29658) (Shanon Jackson)
- 075b74f lexer: Do not drop UTF-8 validation errors in TSX type arguments (#29409) (overlookmotel)
- 59cbbbf lexer: Fix source length assertion (#29408) (overlookmotel)
- 59aed23 parser: Highlight invalid numeric separators (#29117) (camc314)

### ⚡ Performance

- 762bbcf lexer: Cut NEON lexing cost in carve, coalesce and compress (#29414) (Shanon Jackson)

### 📚 Documentation

- 306b96a lexer: Correct safety comment on `TokensView::new` (#29231) (overlookmotel)
- 44c65f0 lexer: Add comments (#29226) (overlookmotel)

