# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.31.0] - 2024-10-08

### Features

- f6e42b6 sourcemap: Add support for sourcemap debug IDs (#6221) (Tim Fish)

## [0.30.4] - 2024-09-28

### Bug Fixes

- 6f98aad sourcemap: Align sourcemap type with Rollup (#6133) (Boshen)

## [0.29.0] - 2024-09-13

### Performance

- d18c896 rust: Use `cow_utils` instead (#5664) (dalaoshu)

## [0.28.0] - 2024-09-11

### Documentation

- fefbbc1 sourcemap: Add trailing newline to README (#5539) (overlookmotel)

## [0.24.3] - 2024-08-18

### Refactor

- 5fd1701 sourcemap: Lower the `msrv`. (#4873) (rzvxa)

## [0.24.0] - 2024-08-08

### Features

- e42ac3a sourcemap: Add `ConcatSourceMapBuilder::from_sourcemaps` (#4639) (overlookmotel)

### Performance

- ff43dff sourcemap: Speed up VLQ encoding (#4633) (overlookmotel)
- a330773 sourcemap: Reduce string copying in `ConcatSourceMapBuilder` (#4638) (overlookmotel)
- 372316b sourcemap: `ConcatSourceMapBuilder` extend `source_contents` in separate loop (#4634) (overlookmotel)
- c7f1d48 sourcemap: Keep local copy of previous token in VLQ encode (#4596) (overlookmotel)
- 590d795 sourcemap: Shorten main loop encoding VLQ (#4586) (overlookmotel)

## [0.23.1] - 2024-08-06

### Features

- e42ac3a sourcemap: Add `ConcatSourceMapBuilder::from_sourcemaps` (#4639) (overlookmotel)

### Performance

- ff43dff sourcemap: Speed up VLQ encoding (#4633) (overlookmotel)
- a330773 sourcemap: Reduce string copying in `ConcatSourceMapBuilder` (#4638) (overlookmotel)
- 372316b sourcemap: `ConcatSourceMapBuilder` extend `source_contents` in separate loop (#4634) (overlookmotel)
- c7f1d48 sourcemap: Keep local copy of previous token in VLQ encode (#4596) (overlookmotel)
- 590d795 sourcemap: Shorten main loop encoding VLQ (#4586) (overlookmotel)

## [0.23.0] - 2024-08-01

- 27fd062 sourcemap: [**BREAKING**] Avoid passing `Result`s (#4541) (overlookmotel)

### Performance

- d00014e sourcemap: Elide bounds checks in VLQ encoding (#4583) (overlookmotel)
- 1fd9dd0 sourcemap: Use simd to escape JSON string (#4487) (Brooooooklyn)

### Refactor

- 7c42ffc sourcemap: Align Base64 chars lookup table to cache line (#4535) (overlookmotel)

## [0.22.1] - 2024-07-27

### Bug Fixes

- 5db7bed sourcemap: Fix pre-calculation of required segments for building JSON (#4490) (overlookmotel)

### Performance

- 705e19f sourcemap: Reduce memory copies encoding JSON (#4489) (overlookmotel)
- 4d10c6c sourcemap: Pre allocate String buf while encoding (#4476) (Brooooooklyn)

### Refactor

- c958a55 sourcemap: `push_list` method for building JSON (#4486) (overlookmotel)

## [0.22.0] - 2024-07-23

### Bug Fixes

- 4cd5df0 sourcemap: Avoid negative line if token_chunks has same prev_dst_line (#4348) (underfin)

## [0.21.0] - 2024-07-18

### Features

- 205c259 sourcemap: Support SourceMapBuilder#token_chunks (#4220) (underfin)

## [0.16.0] - 2024-06-26

### Features

- 01572f0 sourcemap: Impl `std::fmt::Display` for `Error` (#3902) (DonIsaac)- d3cd3ea Oxc transform binding (#3896) (underfin)

## [0.13.1] - 2024-05-22

### Features

- 90d2d09 sourcemap: Add Sourcemap#from_json method (#3361) (underfin)

### Bug Fixes
- 899a52b Fix some nightly warnings (Boshen)

## [0.13.0] - 2024-05-14

### Features

- f6daf0b sourcemap: Add feature "sourcemap_concurrent" (Boshen)
- 7363e14 sourcemap: Add "rayon" feature (#3198) (Boshen)

## [0.12.3] - 2024-04-11

### Features

- 8662f4f sourcemap: Add x_google_ignoreList (#2928) (underfin)
- 5cb3991 sourcemap: Add sourceRoot (#2926) (underfin)

## [0.12.2] - 2024-04-08

### Features

- 96f02e6 sourcemap: Optional JSONSourceMap fileds (#2910) (underfin)
- d87cf17 sourcemap: Add methods to mutate SourceMap (#2909) (underfin)
- 74aca1c sourcemap: Add SourceMapBuilder file (#2908) (underfin)

## [0.12.1] - 2024-04-03

### Bug Fixes

- 28fae2e sourcemap: Using serde_json::to_string to quote sourcemap string (#2889) (underfin)

## [0.11.0] - 2024-03-30

### Features
- b199cb8 Add oxc sourcemap crate (#2825) (underfin)

### Bug Fixes

- 6177c2f codegen: Sourcemap token name should be original name (#2843) (underfin)

