# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.61.2] - 2025-03-23

### Features

- ea3de06 mangler: Support `keep_names` option (#9898) (sapphi-red)

## [0.61.0] - 2025-03-20

### Features

- dcd356e minifier: Support `keep_names` option (#9867) (sapphi-red)
- 6565fc4 napi: Feature gate allocator (#9921) (Boshen)

### Testing

- e637e2e napi/parser: Tweak vitest config (#9878) (Hiroshi Ogawa)

## [0.60.0] - 2025-03-18

### Features

- aa3dff8 napi: Add mimalloc to parser and transformr (#9859) (Boshen)

### Refactor

- 7106e5d napi: Disable unused browser fs (#9848) (hi-ogawa)

## [0.59.0] - 2025-03-18

### Performance

- 84fa538 minify: Use mimalloc-safe to replace mimalloc (#9810) (LongYinan)

## [0.58.0] - 2025-03-13

### Documentation

- a6c9b09 napi/minifier: Improve documentation (#9736) (Boshen)

## [0.57.0] - 2025-03-11

- ef6e0cc semantic: [**BREAKING**] Combine `SymbolTable` and `ScopeTree` into `Scoping` (#9615) (Boshen)

### Refactor

- c6edafe napi: Remove `npm/oxc-*/` npm packages (#9631) (Boshen)

