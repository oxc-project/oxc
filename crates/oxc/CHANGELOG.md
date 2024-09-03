# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.26.0] - 2024-09-03

### Features

- be4642f semantic: Transform checker check child scope IDs (#5410) (overlookmotel)

### Refactor

- 3ae94b8 semantic: Change `build_module_record` to accept &Path instead of PathBuf (Boshen)

## [0.25.0] - 2024-08-23

- ce4d469 codegen: [**BREAKING**] Remove const generic `MINIFY` (#5001) (Boshen)

### Features

- 6800e69 oxc: Add `Compiler` and `CompilerInterface` (#4954) (Boshen)

### Refactor

- cd9cf5e oxc: Remove `remove_whitespace` (Boshen)
- b4407c4 oxc,mangler: `oxc` crate add mangler; mangler use options API (Boshen)
- 4fdf26d transform_conformance: Add driver (#4969) (Boshen)

## [0.21.0] - 2024-07-18

### Features

- 8a190eb oxc: Export `oxc_mangler` (Boshen)

## [0.16.0] - 2024-06-26

### Features

- 4fb90eb oxc: Export isolated-declarations (#3765) (Boshen)

## [0.13.0] - 2024-05-14

### Features

- f6daf0b sourcemap: Add feature "sourcemap_concurrent" (Boshen)

## [0.11.0] - 2024-03-30

### Features
- b199cb8 Add oxc sourcemap crate (#2825) (underfin)

## [0.10.0] - 2024-03-14

### Features
- 697b6b7 Merge features `serde` and `wasm` to `serialize` (#2716) (Boshen)

## [0.6.0] - 2024-02-03

### Features

- cd5026c ast: TypeScript definition for wasm target (#2158) (Nicholas Roberts)

## [0.5.0] - 2024-01-12

### Features

- f1b433b playground: Visualize symbol (#1886) (Dunqing)

### Refactor

- a6717db formatter,linter,codegen: Remove oxc_formatter (#1968) (Boshen)

## [0.4.0] - 2023-12-08

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.3.0] - 2023-11-06

### Features

- 2e2b758 playground: Add transform and minify (#993) (Boshen)

