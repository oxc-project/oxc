# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.141.0] - 2026-07-20

### 🚀 Features

- 2b097c4 str: Export `Str` as `ArenaStr` (#24604) (overlookmotel)
- 3d22307 parser: Add `ParseOptions::enable_ident_hashes` (#24491) (Boshen)

### ⚡ Performance

- ba65790 semantic, allocator: Branchless `clone_in` for semantic IDs (#24564) (overlookmotel)

## [0.138.0] - 2026-06-29

### 💥 BREAKING CHANGES

- 88f4455 str: [**BREAKING**] `Str` and `Ident` methods take `&GetAllocator` (#23781) (overlookmotel)

### 🚀 Features

- 2580eda str: Add `Str::from_str_in` and `Ident::from_str_in` methods (#23767) (overlookmotel)

## [0.125.0] - 2026-04-13

### 💥 BREAKING CHANGES

- 36cdc31 str: [**BREAKING**] Remove identity `FromIn` impl for `Ident` (#21251) (overlookmotel)
- c4aedfa str: [**BREAKING**] Add `static_ident!` macro (#21245) (overlookmotel)

### 🐛 Bug Fixes

- 04b3c2f str: Fix unsound casting const pointers to mut pointers (#21242) (overlookmotel)
- ceadf6c str: Make `Ident::from_raw` an unsafe function (#21241) (overlookmotel)

### 📚 Documentation

- 01bc269 str: Reformat `Ident` doc comments (#21240) (overlookmotel)

## [0.114.0] - 2026-02-16

### 📚 Documentation

- 569aa61 rust: Add missing rustdocs and remove missing_docs lint attrs (#19306) (Boshen)

## [0.113.0] - 2026-02-10

### ⚡ Performance

- ed8c054 oxc_str: Add precomputed hash to Ident for fast HashMap lookups (#19143) (Boshen)

