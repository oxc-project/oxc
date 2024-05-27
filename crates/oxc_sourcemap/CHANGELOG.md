# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.13.1] - 2024-05-22

### Features

- Add Sourcemap#from_json method (#3361)

### Bug Fixes

- Fix some nightly warnings

## [0.13.0] - 2024-05-14

### Features

- Add feature "sourcemap_concurrent"
- Add "rayon" feature (#3198)

## [0.12.3] - 2024-04-11

### Features

- Add x_google_ignoreList (#2928)
- Add sourceRoot (#2926)

## [0.12.2] - 2024-04-08

### Features

- Optional JSONSourceMap fileds (#2910)
- Add methods to mutate SourceMap (#2909)
- Add SourceMapBuilder file (#2908)

## [0.11.1] - 2024-04-03

### Bug Fixes

- Using serde_json::to_string to quote sourcemap string (#2889)

## [0.11.0] - 2024-03-30

### Features

- Add oxc sourcemap crate (#2825)

### Bug Fixes

- Sourcemap token name should be original name (#2843)

