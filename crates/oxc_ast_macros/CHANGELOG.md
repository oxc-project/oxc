# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.22.0] - 2024-07-22

### Refactor

- abfccbd ast: Reduce `#[cfg_attr]` boilerplate in AST type defs (#4375) (overlookmotel)
- 5f1c7ec ast: Rename the `visited_node` marker to `ast`. (#4289) (rzvxa)

## [0.17.0] - 2024-07-05

### Features

- 1854a52 ast_codegen: Introduce the `#[span]` hint. (#4012) (rzvxa)
- 7538af1 ast_codegen: Add visit generator (#3954) (rzvxa)

## [0.16.0] - 2024-06-26

### Refactor

- fcd21a6 traverse: Indicate scope entry point with `scope(enter_before)` attr (#3882) (overlookmotel)

## [0.13.0] - 2024-05-14

### Features

- be87ca8 transform: `oxc_traverse` crate (#3169) (overlookmotel)

