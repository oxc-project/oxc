# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.31.0] - 2024-10-08

### Features

- 7566c2d data_structures: Add `as_slice` + `as_mut_slice` methods to stacks (#6216) (overlookmotel)
- c3c3447 data_structures: Add `oxc_data_structures` crate; add stack (#6206) (Boshen)

### Refactor

- cc57541 data_structures: `NonEmptyStack::len` hint that `len` is never 0 (#6220) (overlookmotel)
- 147a5d5 data_structures: Remove `is_empty` methods for non-empty stacks (#6219) (overlookmotel)
- 61805fd data_structures: Add debug assertion to `SparseStack` (#6218) (overlookmotel)
- dbfa0bc data_structures: Add `len` method to `StackCommon` trait (#6215) (overlookmotel)

