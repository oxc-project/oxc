# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

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

