# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.13.1] - 2024-05-22

### Features

* sourcemap: add Sourcemap#from_json method (#3361)

### Bug Fixes
- fix some nightly warnings |

## [0.13.0] - 2024-05-14

### Features

* sourcemap: add feature "sourcemap_concurrent"
* sourcemap: add "rayon" feature (#3198)

## [0.12.3] - 2024-04-11

### Features

* sourcemap: add x_google_ignoreList (#2928)
* sourcemap: add sourceRoot (#2926)

## [0.12.2] - 2024-04-08

### Features

* sourcemap: optional JSONSourceMap fileds (#2910)
* sourcemap: add methods to mutate SourceMap (#2909)
* sourcemap: add SourceMapBuilder file (#2908)

## [0.12.1] - 2024-04-03

### Bug Fixes

* sourcemap: using serde_json::to_string to quote sourcemap string (#2889)

## [0.11.0] - 2024-03-30

### Features
- add oxc sourcemap crate (#2825) |

### Bug Fixes

* codegen: sourcemap token name should be original name (#2843)

