# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.13.2] - 2024-06-02

### Features

- Add `TraverseCtx::generate_uid` (#3394)

### Bug Fixes

- Exit scope early if enter it late (#3493)

### Refactor

- Move scope from `TSModuleBlock` to `TSModuleDeclaration` (#3488)

## [0.13.2] - 2024-06-02

### Features

- Add `TraverseCtx::generate_uid` (#3394)

### Bug Fixes

- Exit scope early if enter it late (#3493)

### Refactor

- Move scope from `TSModuleBlock` to `TSModuleDeclaration` (#3488)

## [0.13.1] - 2024-05-22

### Features

- Mutable access to scopes tree + symbol table (#3314)
- Pass `&mut TraverseCtx` to visitors (#3312)

### Refactor

- Split context code into multiple files (#3367)
- Move `parent` method etc into `TraverseAncestry` (#3308)
- `Traverse` produce scopes tree using `Semantic` (#3304)
- Store `ScopeId` in AST nodes (#3302)

## [0.13.0] - 2024-05-14

### Features

- Add type to AccessorProperty to support TSAbractAccessorProperty (#3256)
- Add scope flags to `TraverseCtx` (#3229)
- `oxc_traverse` crate (#3169)

### Bug Fixes

- Create scopes for functions (#3273)
- Allow `TraverseCtx::find_ancestor` closure to return AST node (#3236)
- Create scope for function nested in class method (#3234)
- Correctly parse cls.fn<C> = x (#3208)

### Documentation

- Improve docs for `TraverseCtx::ancestors_depth` (#3194)

### Refactor

- Simplify build script (#3231)
- Order AST type fields in visitation order (#3228)
- Do not expose `TraverseCtx::new` (#3226)
- `retag_stack` use `AncestorType` (#3173)

