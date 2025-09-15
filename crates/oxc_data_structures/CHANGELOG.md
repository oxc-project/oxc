# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).


## [0.88.0] - 2025-09-15

### 💥 BREAKING CHANGES

- e433633 data_structures: [**BREAKING**] Make `NonEmptyStack::is_empty` a compile-time error (#13673) (overlookmotel)

### 🚀 Features

- 1a58e99 data_structures: Add `is_exhausted` method to `NonEmptyStack` and `SparseStack` (#13672) (overlookmotel)
- 2db32eb data_structures: Add `boxed_slice!` and `boxed_array!` macros (#13596) (overlookmotel)


## [0.87.0] - 2025-09-08

### 🐛 Bug Fixes

- 34d3cde rust: Fix clippy issues (#13540) (Boshen)


## [0.86.0] - 2025-08-31

### 💥 BREAKING CHANGES

- edeebc6 data_structures: [**BREAKING**] Rename `SliceIterExt` to `SliceIter` (#13439) (overlookmotel)

### 🚀 Features

- d0479e9 data_structures: Add `as_mut_slice` method to `IterMut` via `SliceIterMutExt` trait (#13437) (overlookmotel)
- 5b139aa data_structures: Add `ptr` and `end_ptr` methods to `SliceIterExt` (#13435) (overlookmotel)
- d8b027f data_structures: Add `SliceIterExt::peek` method (#13434) (overlookmotel)

### 🚜 Refactor

- 51919c2 data_structures: Rename lifetime in `SliceIterExt` (#13433) (overlookmotel)

### ⚡ Performance

- 475205f data_structures: Reduce `IterMut::advance_unchecked` to 1 instruction (#13438) (overlookmotel)








## [0.82.0] - 2025-08-12

### 💥 BREAKING CHANGES

- 128b527 data_structures: [**BREAKING**] Remove `PointerExt` trait (#12903) (overlookmotel)

### 📚 Documentation

- bb7838d data_structures: Add stacks to README (#12904) (overlookmotel)


## [0.81.0] - 2025-08-06

### ⚡ Performance

- e8ac1a5 codegen: Write indent in chunks of 32 bytes (#12745) (overlookmotel)


## [0.80.0] - 2025-08-03

### 🚀 Features

- af4d558 codegen: Add options to control indentation (#12691) (Copilot)

### 📚 Documentation

- 45e2fe8 rust: Fix typos and grammar mistakes in Rust documentation comments (#12715) (Copilot)
- de1de35 rust: Add comprehensive README.md documentation for all Rust crates (#12706) (Copilot)







## [0.77.1] - 2025-07-16

### 🚀 Features

- 7cb4d22 data_structures: `SliceIterExt` extension trait (#12294) (overlookmotel)










# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.71.0] - 2025-05-20

- 5d9344f rust: [**BREAKING**] Clippy avoid-breaking-exported-api = false (#11088) (Boshen)

### Features

- c79a7d0 data_structures: Introduce `PointerExt` trait (#11095) (overlookmotel)

### Performance

- f8fac4e data_structures: Inline all methods in `PointerExt` (#11149) (overlookmotel)

### Refactor

- bb8bde3 various: Update macros to use `expr` fragment specifier (#11113) (overlookmotel)

## [0.69.0] - 2025-05-09

### Features

- 05bf1be data_structures: Add `InlineString::as_mut_str` method (#10856) (overlookmotel)

### Performance

- 2d4b8c9 data_structures: Optimize `InlineString` (#10850) (overlookmotel)

## [0.61.0] - 2025-03-20

### Features

- 38ad787 data_structures: Add `assert_unchecked!` macro (#9885) (overlookmotel)

### Testing

- 9314147 data_structures: Enable doc tests for `oxc_data_structures` crate (#9884) (overlookmotel)

## [0.60.0] - 2025-03-18

- b3ce925 data_structures: [**BREAKING**] Put all parts behind features (#9849) (overlookmotel)

### Features


## [0.55.0] - 2025-03-05

### Features

- 29041fb data_structures: Move `InlineString` into `oxc_data_structures` crate (#9549) (overlookmotel)

## [0.53.0] - 2025-02-26

### Features

- f21740e data_structures: Add `CodeBuffer::print_bytes_iter_unchecked` method (#9337) (overlookmotel)

### Bug Fixes

- 54d59f1 data_structures: Stack types correctly report allocation size if allocation failure during grow (#9317) (overlookmotel)

### Documentation

- 8bd3e39 data_structures: Uppercase SAFETY comments (#9330) (overlookmotel)

### Refactor

- 9d98444 codegen, data_structures: Move `CodeBuffer` into `oxc_data_structures` crate (#9326) (overlookmotel)
- 6a4e892 data_structures: Add debug assertion to `CodeBuffer::peek_nth_char_back` and improve safety docs (#9336) (overlookmotel)
- fc46218 data_structures: `CodeBuffer::print_str` use `Vec::extend_from_slice` (#9332) (overlookmotel)
- 690bae5 data_structures: Stack types const assert `T` is not zero-size type (#9318) (overlookmotel)
- 10ba2ea data_structures: Reduce scope of `unsafe` blocks (#9316) (overlookmotel)
- beb8382 data_structures: `CodeBuffer::print_bytes_unchecked` take a byte slice (#9327) (overlookmotel)

## [0.49.0] - 2025-02-10

- bec8fee data_structures: [**BREAKING**] Rename `Stack::last_unchecked_mut` method (#8911) (overlookmotel)

### Features

- 0a74cf5 data_structures: Add `first` and `first_mut` methods to stack types (#8908) (overlookmotel)

### Documentation

- f6b6e70 data_structures: Correct doc comments for `SparseStack` (#8907) (overlookmotel)

### Refactor


### Testing

- 2d06260 data_structures: Add tests for `NonEmptyStack::as_slice` and `as_slice_mut` (#8912) (overlookmotel)

## [0.45.0] - 2025-01-11

### Documentation

- e0a09ab data_structures: Improve docs for stack types (#8356) (overlookmotel)

### Refactor

- 9c1844a data_structures: Remove `NonNull` shim (#8423) (overlookmotel)

## [0.42.0] - 2024-12-18

### Features

- 46e2e27 data_structures: Implement `Default` for `NonEmptyStack` (#7946) (overlookmotel)

### Styling

- fb897f6 data_structures: Add line break (#7882) (overlookmotel)
- 7fb9d47 rust: `cargo +nightly fmt` (#7877) (Boshen)

## [0.40.0] - 2024-12-10

### Features

- cf2ee06 data_structures: Add rope (#7764) (Boshen)

### Styling

- e97a954 data_structures: Line breaks (#7766) (overlookmotel)

## [0.39.0] - 2024-12-04

### Features

- defaf4b data_structures: Add `SparseStack::last_mut` method (#7528) (overlookmotel)

## [0.37.0] - 2024-11-21

### Features

- d135d3e data_structures: Add methods to `SparseStack` (#7305) (overlookmotel)

## [0.35.0] - 2024-11-04

### Performance

- c58ec89 data_structures: Optimize `NonEmptyStack::pop` (#7021) (overlookmotel)

### Refactor

- b021147 data_structures: Make all methods of `NonNull` shim `#[inline(always)]` (#7024) (overlookmotel)
- fb1710a data_structures: Add `#[repr(transparent)]` to `NonNull` shim (#7023) (overlookmotel)
- f1fc8db data_structures: Add `read` method to `NonNull` shim (#7022) (overlookmotel)

## [0.32.0] - 2024-10-19

### Bug Fixes

- 7cc05f1 data_structures: Fix compilation failure on older Rust versions (#6526) (overlookmotel)

### Documentation

- de22b81 data-structures: Enable lint warnings on missing docs, and add missing doc comments (#6612) (DonIsaac)

## [0.31.0] - 2024-10-08

### Features

- 7566c2d data_structures: Add `as_slice` + `as_mut_slice` methods to stacks (#6216) (overlookmotel)
- c3c3447 data_structures: Add `oxc_data_structures` crate; add stack (#6206) (Boshen)

### Refactor

- cc57541 data_structures: `NonEmptyStack::len` hint that `len` is never 0 (#6220) (overlookmotel)
- 147a5d5 data_structures: Remove `is_empty` methods for non-empty stacks (#6219) (overlookmotel)
- 61805fd data_structures: Add debug assertion to `SparseStack` (#6218) (overlookmotel)
- dbfa0bc data_structures: Add `len` method to `StackCommon` trait (#6215) (overlookmotel)

