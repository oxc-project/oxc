# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

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

