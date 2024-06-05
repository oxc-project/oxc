# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.14.0] - 2024-06-05

### Bug Fixes

* transformer: JSX set `reference_id` on refs to imports (#3524)

### Refactor

* transformer/typescript: replace reference collector with symbols references (#3533)

## [0.13.3] - 2024-06-04

### Refactor

* traverse: `generate_uid` return `SymbolId` (#3520)

## [0.13.2] - 2024-06-03

### Bug Fixes

* traverse: exit scope early if enter it late (#3493)

### Refactor

* ast: move scope from `TSModuleBlock` to `TSModuleDeclaration` (#3488)

### Features

* transformer: add `TraverseCtx::generate_uid` (#3394)

## [0.13.1] - 2024-05-22

### Refactor

* ast: store `ScopeId` in AST nodes (#3302)
* traverse: split context code into multiple files (#3367)
* traverse: move `parent` method etc into `TraverseAncestry` (#3308)
* traverse: `Traverse` produce scopes tree using `Semantic` (#3304)

### Features

* traverse: mutable access to scopes tree + symbol table (#3314)
* traverse: pass `&mut TraverseCtx` to visitors (#3312)

## [0.13.0] - 2024-05-14

### Bug Fixes

* parser: correctly parse cls.fn<C> = x (#3208)
* traverse: create scopes for functions (#3273)
* traverse: allow `TraverseCtx::find_ancestor` closure to return AST node (#3236)
* traverse: create scope for function nested in class method (#3234)

### Features

* ast: add type to AccessorProperty to support TSAbractAccessorProperty (#3256)
* transform: `oxc_traverse` crate (#3169)
* traverse: add scope flags to `TraverseCtx` (#3229)

### Refactor

* ast: order AST type fields in visitation order (#3228)
* transform: `retag_stack` use `AncestorType` (#3173)
* traverse: simplify build script (#3231)
* traverse: do not expose `TraverseCtx::new` (#3226)

### Documentation

* transform: improve docs for `TraverseCtx::ancestors_depth` (#3194)

