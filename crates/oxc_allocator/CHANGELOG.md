# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.31.0] - 2024-10-08

### Performance

- 5db9b30 allocator: Use lower bound of size hint when creating Vecs from an iterator (#6194) (DonIsaac)

### Refactor

- f7d1136 allocator: Remove unnecessary `Vec` impl (#6213) (overlookmotel)

## [0.30.2] - 2024-09-27

### Documentation

- 3099709 allocator: Document `oxc_allocator` crate (#6037) (DonIsaac)

## [0.27.0] - 2024-09-06

### Features

- e8bdd12 allocator: Add `AsMut` impl for `Box` (#5515) (overlookmotel)

## [0.25.0] - 2024-08-23

### Refactor

- a4247e9 allocator: Move `Box` and `Vec` into separate files (#5034) (overlookmotel)

## [0.24.3] - 2024-08-18

### Refactor

- a6967b3 allocator: Correct code comment (#4904) (overlookmotel)
- 90d0b2b allocator, ast, span, ast_tools: Use `allocator` as var name for `Allocator` (#4900) (overlookmotel)

## [0.24.2] - 2024-08-12

### Features

- 8e10e25 allocator: Introduce `Address` (#4810) (overlookmotel)

## [0.24.0] - 2024-08-08

### Features

- 23b0040 allocator: Introduce `CloneIn` trait. (#4726) (rzvxa)

## [0.23.0] - 2024-08-01

### Performance

- 4c6d19d allocator: Use capacity hint (#4584) (Luca Bruno)

## [0.22.0] - 2024-07-23

### Refactor

- 504daed allocator: Rename fn params for `Box::new_in` (#4431) (overlookmotel)

## [0.17.2] - 2024-07-08

### Features

- 115ac3b allocator: Introduce `FromIn` and `IntoIn` traits. (#4088) (rzvxa)

## [0.15.0] - 2024-06-18

### Features

- 8f5655d linter: Add eslint/no-useless-constructor (#3594) (Don Isaac)

## [0.13.0] - 2024-05-14

### Refactor

- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)

## [0.12.5] - 2024-04-22

### Refactor

- 6bc18e1 bench: Reuse allocator in parser + lexer benchmarks (#3053) (overlookmotel)

## [0.12.4] - 2024-04-19

### Features

- 063b281 allocator: Make `Box`'s PhantomData own the passed in `T` (#2952) (Boshen)

## [0.6.0] - 2024-02-03

### Documentation

- a1271af allocator: Document behaviour of `Box` (Boshen)

## [0.5.0] - 2024-01-12

### Features

- a6d9356 allocator: Add `From` API (#1908) (Boshen)

## [0.4.0] - 2023-12-08

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.2.0] - 2023-09-14

### Refactor
- 12798e0 Improve code coverage a little bit (Boshen)- fdf288c Improve code coverage in various places (#721) (Boshen)

