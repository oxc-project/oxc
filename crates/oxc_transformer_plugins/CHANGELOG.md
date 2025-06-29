# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).


## [0.74.0] - 2025-06-23

### 💥 BREAKING CHANGES

- 8ef1be2 traverse: [**BREAKING**] Introduce `TraverseCtx<'a, State>` (#11770) (Boshen)





# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.72.0] - 2025-05-24

### Features

- 03390ad allocator: `TakeIn` trait with `AllocatorAccessor` (#11201) (Boshen)

## [0.71.0] - 2025-05-20

- 5d9344f rust: [**BREAKING**] Clippy avoid-breaking-exported-api = false (#11088) (Boshen)

### Refactor


## [0.70.0] - 2025-05-15

### Features

- 1673ffb codegen: Rework printing normal / legal / annotation comments (#10997) (Boshen)

## [0.68.0] - 2025-05-03

- a0a37e0 ast: [**BREAKING**] `AstBuilder` methods require an `Atom` with correct lifetime (#10735) (overlookmotel)

- 315143a codegen: [**BREAKING**] Remove useless `CodeGenerator` type alias (#10702) (Boshen)

### Features

- b01cb45 codegen: A way to keep legal comments after minification (#10689) (Boshen)

### Bug Fixes

- 4795059 transformer_plugins: Provide reference data when identifiers are replaced (#10620) (Boshen)

### Performance

- 8d84cf5 transformer: Avoid copying string data (#10726) (overlookmotel)
- c753f75 transformer, linter: Use `format_compact_str!` (#10753) (overlookmotel)
- 699ab3e transformer/inject_global_variables: Do not search string twice (#10751) (overlookmotel)

### Refactor


## [0.67.0] - 2025-04-27

### Features

- 1962bc6 transformer_plugins: Split out `oxc_transformer_plugins` crate (#10617) (Boshen)

### Testing

- fbd6864 transformer_plugins: Apply DCE after replace plugin (#10619) (Boshen)

